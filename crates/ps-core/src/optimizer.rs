use crate::clipboard::selection_matches;
use crate::error::{PsError, PsResult};
use crate::ollama::{ChatMessage, ChatOptions, ChatRequest, ChatResponse, OllamaClient};
use crate::preservation::{check_preservation, PreservationReport};
use crate::prompts::{output_schema, prompt_version, system_prompt, user_message};
use crate::settings::AppSettings;
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};
use tokio_util::sync::CancellationToken;

pub const MAX_PROMPT_CHARS: usize = 50_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OptimizationStatus {
    Rewritten,
    Unchanged,
}

#[derive(Debug, Clone)]
pub struct OptimizeConfig {
    pub temperature: f32,
    pub top_p: f32,
    pub output_token_limit: Option<u32>,
    pub timeout: Duration,
    pub reasoning_enabled: bool,
    pub keep_alive: String,
}

impl From<&AppSettings> for OptimizeConfig {
    fn from(settings: &AppSettings) -> Self {
        Self {
            temperature: settings.temperature,
            top_p: settings.top_p,
            output_token_limit: settings.output_token_limit,
            timeout: Duration::from_secs(settings.request_timeout_secs),
            reasoning_enabled: settings.reasoning_enabled,
            keep_alive: settings.keep_alive.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OptimizationAnalysis {
    pub status: OptimizationStatus,
    pub rewritten_prompt: String,
    pub preservation: PreservationReport,
    pub suspicious: bool,
    pub duration_ms: u128,
    pub prompt_tokens: Option<u64>,
    pub completion_tokens: Option<u64>,
    pub prompt_version: String,
    pub model: String,
}

pub fn select_output_budget(input_chars: usize, configured: Option<u32>) -> u32 {
    if let Some(limit) = configured {
        if (128..=32_768).contains(&limit) {
            return limit;
        }
    }
    let input_tokens = ((input_chars as u32).saturating_add(3)) / 4;
    let budget = input_tokens.saturating_mul(3).saturating_add(512);
    budget.clamp(512, 16_384)
}

pub fn build_chat_request(model: &str, original: &str, config: &OptimizeConfig) -> ChatRequest {
    ChatRequest {
        model: model.to_string(),
        messages: vec![
            ChatMessage {
                role: "system".into(),
                content: system_prompt(),
            },
            ChatMessage {
                role: "user".into(),
                content: user_message(original),
            },
        ],
        stream: false,
        format: output_schema(),
        keep_alive: config.keep_alive.clone(),
        think: config.reasoning_enabled,
        options: ChatOptions {
            temperature: config.temperature,
            top_p: config.top_p,
            num_predict: select_output_budget(original.chars().count(), config.output_token_limit),
        },
    }
}

pub async fn optimize_prompt(
    client: &OllamaClient,
    model: &str,
    original: &str,
    config: &OptimizeConfig,
    cancel: &CancellationToken,
) -> PsResult<OptimizationAnalysis> {
    if original.trim().is_empty() {
        return Err(PsError::NoSelection);
    }
    if original.chars().count() > MAX_PROMPT_CHARS {
        return Err(PsError::PromptTooLong);
    }
    let started = Instant::now();
    let request = build_chat_request(model, original, config);
    let response = client.chat(&request, config.timeout, cancel).await?;
    interpret_chat_response(original, model, response, started.elapsed())
}

pub fn interpret_chat_response(
    original: &str,
    model: &str,
    response: ChatResponse,
    elapsed: Duration,
) -> PsResult<OptimizationAnalysis> {
    if let Some(error) = response.error.as_deref() {
        let lower = error.to_ascii_lowercase();
        if lower.contains("not found") {
            return Err(PsError::ModelUnavailable {
                model: model.to_string(),
            });
        }
        return Err(PsError::MalformedOutput {
            detail: "Ollama reported an inference error.".into(),
        });
    }
    if response.done_reason.as_deref() == Some("length") {
        return Err(PsError::TruncatedOutput);
    }
    let message = response.message.unwrap_or(crate::ollama::ChatMessageBody {
        role: "assistant".into(),
        content: String::new(),
        thinking: None,
    });
    if message.content.trim().is_empty() {
        return Err(PsError::EmptyResponse);
    }
    let parsed = parse_model_output(&message.content, original)?;
    let preservation = if parsed.status == OptimizationStatus::Rewritten {
        check_preservation(original, &parsed.rewritten_prompt)
    } else {
        PreservationReport::clean()
    };
    let suspicious = preservation.suspicious;
    Ok(OptimizationAnalysis {
        status: parsed.status,
        rewritten_prompt: parsed.rewritten_prompt,
        preservation,
        suspicious,
        duration_ms: elapsed.as_millis(),
        prompt_tokens: response.prompt_eval_count,
        completion_tokens: response.eval_count,
        prompt_version: prompt_version().to_string(),
        model: model.to_string(),
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ParsedOptimization {
    status: OptimizationStatus,
    rewritten_prompt: String,
}

pub(crate) fn parse_model_output(raw: &str, original: &str) -> PsResult<ParsedOptimization> {
    let extracted = extract_json(raw);
    if extracted.is_empty() {
        return Err(PsError::EmptyResponse);
    }
    if !extracted.ends_with('}') {
        return Err(PsError::TruncatedOutput);
    }
    let value: serde_json::Value =
        serde_json::from_str(extracted).map_err(|_| PsError::MalformedOutput {
            detail: "Structured output was not valid JSON.".into(),
        })?;
    let object = value.as_object().ok_or_else(|| PsError::MalformedOutput {
        detail: "Structured output must be a JSON object.".into(),
    })?;
    const ALLOWED: &[&str] = &["status", "rewritten_prompt", "clarification_question"];
    if !object.contains_key("status") || !object.contains_key("rewritten_prompt") {
        return Err(PsError::MalformedOutput {
            detail: "Structured output did not contain exactly the required fields.".into(),
        });
    }
    if object.keys().any(|key| !ALLOWED.contains(&key.as_str())) {
        return Err(PsError::MalformedOutput {
            detail: "Structured output did not contain exactly the required fields.".into(),
        });
    }
    let status = required_string(object, "status")?;
    let rewritten = required_string(object, "rewritten_prompt")?;
    // Older models still emit clarification_question. It is never shown.
    match status {
        "rewritten" => settle_rewritten(rewritten, original),
        "unchanged" => settle_unchanged(rewritten, original),
        "needs_clarification" => Ok(settle_legacy_question(rewritten, original)),
        _ => Err(PsError::MalformedOutput {
            detail: "The model returned an unknown result status.".into(),
        }),
    }
}

fn settle_rewritten(rewritten: &str, original: &str) -> PsResult<ParsedOptimization> {
    let rewritten = rewritten.trim();
    if rewritten.is_empty() {
        return Err(PsError::MalformedOutput {
            detail: "A rewritten result did not include a prompt.".into(),
        });
    }
    Ok(accept_or_leave(rewritten, original))
}

/// `unchanged` is a decision to keep the selection. Small models often rewrite the
/// text anyway. That reply is not an error, and the rewritten text is not used.
fn settle_unchanged(_rewritten: &str, original: &str) -> PsResult<ParsedOptimization> {
    Ok(unchanged(original))
}

/// A model that still asks a question does not stop the shortcut.
/// A usable rewrite is kept. A question by itself leaves the selection alone.
fn settle_legacy_question(rewritten: &str, original: &str) -> ParsedOptimization {
    let rewritten = rewritten.trim();
    if let Some(prompt) = usable_prompt(rewritten, original) {
        rewritten_result(prompt)
    } else {
        unchanged(original)
    }
}

fn accept_or_leave(rewritten: &str, original: &str) -> ParsedOptimization {
    if selection_matches(rewritten, original)
        || is_cosmetic_edit(rewritten, original)
        || is_user_facing_question(rewritten)
        || is_placeholder_prompt(rewritten)
    {
        unchanged(original)
    } else {
        rewritten_result(rewritten)
    }
}

/// Case, spacing, wrapping quotes, and a trailing sentence mark are not a rewrite.
/// Internal punctuation stays significant so versions, paths, and flags are kept.
fn is_cosmetic_edit(rewritten: &str, original: &str) -> bool {
    let left = cosmetic_key(original);
    let right = cosmetic_key(rewritten);
    !left.is_empty() && left == right
}

fn cosmetic_key(text: &str) -> String {
    let collapsed = text.split_whitespace().collect::<Vec<_>>().join(" ");
    let trimmed = collapsed.trim().trim_matches(['"', '\'', '`']);
    trimmed.trim_end_matches(['.', '!', '?']).to_lowercase()
}

fn rewritten_result(prompt: &str) -> ParsedOptimization {
    ParsedOptimization {
        status: OptimizationStatus::Rewritten,
        rewritten_prompt: prompt.to_string(),
    }
}

fn usable_prompt<'a>(rewritten: &'a str, original: &str) -> Option<&'a str> {
    if rewritten.chars().count() < 24
        || selection_matches(rewritten, original)
        || is_cosmetic_edit(rewritten, original)
        || is_placeholder_prompt(rewritten)
        || is_user_facing_question(rewritten)
    {
        return None;
    }
    Some(rewritten)
}

fn is_placeholder_prompt(prompt: &str) -> bool {
    let stripped = prompt
        .trim()
        .trim_matches(|ch: char| matches!(ch, '"' | '\'' | '`' | '.' | '!' | '?' | ' '));
    let lower = stripped.to_ascii_lowercase();
    matches!(
        lower.as_str(),
        "" | "none"
            | "n/a"
            | "na"
            | "null"
            | "nil"
            | "undefined"
            | "no"
            | "nope"
            | "nothing"
            | "empty"
            | "-"
            | "—"
            | "..."
            | "no clarification"
            | "no clarification needed"
            | "no clarification required"
            | "no clarification necessary"
            | "none needed"
            | "not needed"
            | "not applicable"
            | "no question"
            | "no questions"
            | "no follow-up"
            | "no follow up"
            | "no followup"
            | "false"
    )
}

/// A short question aimed at the user, rather than a prompt for the coding agent.
fn is_user_facing_question(text: &str) -> bool {
    let trimmed = text.trim();
    if trimmed.chars().count() > 280 || !trimmed.contains('?') {
        return false;
    }
    let lower = trimmed.to_ascii_lowercase();
    let first = lower.lines().next().unwrap_or(&lower).trim();
    const LEADS: &[&str] = &[
        "could you",
        "can you",
        "would you",
        "please clarify",
        "please specify",
        "please provide",
        "please tell",
        "what specifically",
        "which file",
        "which option",
        "which approach",
        "which one",
        "do you ",
        "did you ",
        "are you ",
        "should i",
        "should we",
        "should this",
        "should it",
        "should the",
        "should user",
        "i need more",
        "i need to know",
        "before i ",
        "before proceeding",
        "to clarify",
        "clarification:",
        "quick question",
        "one question",
    ];
    if !LEADS.iter().any(|lead| first.starts_with(lead)) {
        return false;
    }
    let remainder = first
        .split_once('?')
        .map(|(_, rest)| rest.trim())
        .unwrap_or("");
    remainder.chars().count() <= 40
}

fn unchanged(original: &str) -> ParsedOptimization {
    ParsedOptimization {
        status: OptimizationStatus::Unchanged,
        rewritten_prompt: original.to_string(),
    }
}

fn required_string<'a>(
    object: &'a serde_json::Map<String, serde_json::Value>,
    key: &str,
) -> PsResult<&'a str> {
    object
        .get(key)
        .and_then(|value| value.as_str())
        .ok_or_else(|| PsError::MalformedOutput {
            detail: "A required structured-output field was not text.".into(),
        })
}

fn extract_json(raw: &str) -> &str {
    let trimmed = raw.trim().trim_start_matches('\u{feff}');
    let Some(rest) = trimmed.strip_prefix("```") else {
        return trimmed;
    };
    let rest = rest.trim_start_matches(|ch: char| ch != '\n');
    let rest = rest.trim_start_matches('\n').trim();
    rest.strip_suffix("```").unwrap_or(rest).trim()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config() -> OptimizeConfig {
        OptimizeConfig::from(&AppSettings::default())
    }

    #[test]
    fn output_budget_grows_with_the_prompt_and_respects_an_explicit_limit() {
        assert_eq!(select_output_budget(0, None), 512);
        assert!(select_output_budget(10_000, None) > 2_000);
        assert_eq!(select_output_budget(10_000, Some(1024)), 1024);
        assert_eq!(select_output_budget(100_000, None), 16_384);
    }

    #[test]
    fn request_keeps_the_original_out_of_the_system_message() {
        let original = "UNIQUE_SELECTED_PROMPT with `file.ts`";
        let request = build_chat_request("qwen3.5:4b", original, &config());
        assert!(request.messages[0].content.contains("Clarify"));
        assert!(request.messages[0].content.contains("Condense"));
        assert!(!request.messages[0].content.contains("MODE:"));
        assert!(!request.stream);
        assert!(!request.think);
        assert!((request.options.temperature - 0.2).abs() < 0.0001);
        assert_eq!(request.options.top_p, 0.9);
        assert_eq!(request.keep_alive, "10m");
        assert_eq!(request.messages[0].role, "system");
        assert_eq!(request.messages[1].role, "user");
        assert!(!request.messages[0]
            .content
            .contains("UNIQUE_SELECTED_PROMPT"));
        assert!(request.messages[1]
            .content
            .contains("UNIQUE_SELECTED_PROMPT"));
        assert_eq!(request.format["additionalProperties"], false);
    }

    #[test]
    fn parses_a_rewrite_and_an_unchanged_prompt() {
        let original = "fix login it breaks on refresh";
        let rewritten = parse_model_output(
            r#"{"status":"rewritten","rewritten_prompt":"Investigate the login failure after a page refresh. Identify the root cause, implement a focused fix, and verify that the expected login state is preserved."}"#,
            original,
        )
        .unwrap();
        assert_eq!(rewritten.status, OptimizationStatus::Rewritten);

        let same = parse_model_output(
            &format!(
                r#"{{"status":"rewritten","rewritten_prompt":{}}}"#,
                serde_json::to_string(original).unwrap()
            ),
            original,
        )
        .unwrap();
        assert_eq!(same.status, OptimizationStatus::Unchanged);
        assert_eq!(same.rewritten_prompt, original);

        let unchanged = parse_model_output(
            &format!(
                r#"{{"status":"unchanged","rewritten_prompt":{}}}"#,
                serde_json::to_string(original).unwrap()
            ),
            original,
        )
        .unwrap();
        assert_eq!(unchanged.status, OptimizationStatus::Unchanged);
        assert_eq!(unchanged.rewritten_prompt, original);
    }

    #[test]
    fn cosmetic_edits_keep_the_original_text() {
        let original =
            "Add a loading spinner to the Submit button while the existing request is in progress.";
        let capitalized = parse_model_output(
            &format!(
                r#"{{"status":"rewritten","rewritten_prompt":{}}}"#,
                serde_json::to_string(&original.to_uppercase()).unwrap()
            ),
            original,
        )
        .unwrap();
        assert_eq!(capitalized.status, OptimizationStatus::Unchanged);
        assert_eq!(capitalized.rewritten_prompt, original);

        let punctuated = parse_model_output(
            &format!(
                r#"{{"status":"unchanged","rewritten_prompt":{}}}"#,
                serde_json::to_string(&format!("{original}.")).unwrap()
            ),
            original,
        )
        .unwrap();
        assert_eq!(punctuated.status, OptimizationStatus::Unchanged);
        assert_eq!(punctuated.rewritten_prompt, original);

        let clarified = parse_model_output(
            r#"{"status":"rewritten","rewritten_prompt":"Investigate the login failure after a page refresh. Identify the root cause and fix it."}"#,
            "fix login it breaks on refresh",
        )
        .unwrap();
        assert_eq!(clarified.status, OptimizationStatus::Rewritten);

        let version = parse_model_output(
            r#"{"status":"rewritten","rewritten_prompt":"Keep compatibility with Node 2."}"#,
            "Keep compatibility with Node 22.",
        )
        .unwrap();
        assert_eq!(version.status, OptimizationStatus::Rewritten);
        assert!(version.rewritten_prompt.contains("Node 2."));

        let path = parse_model_output(
            r#"{"status":"rewritten","rewritten_prompt":"Update src/auth/sessionts without changing the public API."}"#,
            "Update src/auth/session.ts without changing the public API.",
        )
        .unwrap();
        assert_eq!(path.status, OptimizationStatus::Rewritten);
        assert!(path.rewritten_prompt.contains("sessionts"));
    }

    #[test]
    fn a_question_leaves_the_selection_and_a_rewrite_is_kept() {
        let original = "fix login it breaks on refresh";
        let only_question = parse_model_output(
            r#"{"status":"needs_clarification","rewritten_prompt":"","clarification_question":"Should user deletion be permanent or reversible?"}"#,
            original,
        )
        .unwrap();
        assert_eq!(only_question.status, OptimizationStatus::Unchanged);
        assert_eq!(only_question.rewritten_prompt, original);

        let shaped = parse_model_output(
            r#"{"status":"rewritten","rewritten_prompt":"Could you clarify which file contains the login bug?"}"#,
            original,
        )
        .unwrap();
        assert_eq!(shaped.status, OptimizationStatus::Unchanged);
        assert_eq!(shaped.rewritten_prompt, original);

        let with_rewrite = parse_model_output(
            r#"{"status":"needs_clarification","rewritten_prompt":"Investigate the login failure after a page refresh. Identify the root cause, implement a focused fix, and verify that the expected login state is preserved.","clarification_question":"What should the expected behavior be?"}"#,
            original,
        )
        .unwrap();
        assert_eq!(with_rewrite.status, OptimizationStatus::Rewritten);
        assert!(with_rewrite.rewritten_prompt.contains("login failure"));

        let stray = parse_model_output(
            r#"{"status":"rewritten","rewritten_prompt":"Investigate the login failure after a page refresh and fix the session.","clarification_question":"Which file contains the login bug?"}"#,
            original,
        )
        .unwrap();
        assert_eq!(stray.status, OptimizationStatus::Rewritten);
        assert!(stray.rewritten_prompt.contains("fix the session"));
    }

    #[test]
    fn rejects_contradictory_empty_and_truncated_payloads() {
        let original =
            "Add a loading spinner to the Submit button while the existing request is in progress.";
        assert!(
            parse_model_output(r#"{"status":"rewritten","rewritten_prompt":""}"#, original)
                .is_err()
        );
        let question_only = parse_model_output(
            r#"{"status":"needs_clarification","rewritten_prompt":"do this","clarification_question":"Which?"}"#,
            original,
        )
        .unwrap();
        assert_eq!(question_only.status, OptimizationStatus::Unchanged);
        assert_eq!(question_only.rewritten_prompt, original);
        let empty_question = parse_model_output(
            r#"{"status":"needs_clarification","rewritten_prompt":"","clarification_question":""}"#,
            original,
        )
        .unwrap();
        assert_eq!(empty_question.status, OptimizationStatus::Unchanged);
        assert!(matches!(
            parse_model_output(
                r#"{"status":"rewritten","rewritten_prompt":"partial"#,
                original
            ),
            Err(PsError::TruncatedOutput)
        ));
        assert!(matches!(
            parse_model_output("   ", original),
            Err(PsError::EmptyResponse)
        ));
        assert!(
            parse_model_output(r#"{"status":"ship_it","rewritten_prompt":"x"}"#, original).is_err()
        );
        assert!(parse_model_output("```json\n{\"status\":\"unchanged\"}\n```", original).is_err());
    }

    #[test]
    fn unchanged_status_keeps_the_original_when_the_model_returns_different_text() {
        let original = "is the readme good eonugh for an open source project?";
        let parsed = parse_model_output(
            r#"{"status":"unchanged","rewritten_prompt":"Is the README good enough for an open source project?"}"#,
            original,
        )
        .unwrap();
        assert_eq!(parsed.status, OptimizationStatus::Unchanged);
        assert_eq!(parsed.rewritten_prompt, original);
    }

    #[test]
    fn length_stop_and_empty_content_do_not_become_rewrites() {
        let response = ChatResponse {
            message: Some(crate::ollama::ChatMessageBody {
                role: "assistant".into(),
                content: "{\"status\":\"rewritten\"}".into(),
                thinking: None,
            }),
            done: true,
            done_reason: Some("length".into()),
            prompt_eval_count: Some(3),
            eval_count: Some(9),
            error: None,
        };
        assert!(matches!(
            interpret_chat_response("hello", "m", response, Duration::from_millis(5)),
            Err(PsError::TruncatedOutput)
        ));

        let empty = ChatResponse {
            message: Some(crate::ollama::ChatMessageBody {
                role: "assistant".into(),
                content: String::new(),
                thinking: Some("hidden".into()),
            }),
            done: true,
            done_reason: Some("stop".into()),
            prompt_eval_count: None,
            eval_count: None,
            error: None,
        };
        assert!(matches!(
            interpret_chat_response("hello", "m", empty, Duration::ZERO),
            Err(PsError::EmptyResponse)
        ));
    }
}
