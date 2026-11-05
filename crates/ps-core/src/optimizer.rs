use crate::clipboard::selection_matches;
use crate::error::{PsError, PsResult};
use crate::mode::OptimizationMode;
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
    NeedsClarification,
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
    pub clarification_question: String,
    pub preservation: PreservationReport,
    pub suspicious: bool,
    pub duration_ms: u128,
    pub prompt_tokens: Option<u64>,
    pub completion_tokens: Option<u64>,
    pub prompt_version: String,
    pub model: String,
    pub mode: OptimizationMode,
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

pub fn build_chat_request(
    model: &str,
    original: &str,
    mode: OptimizationMode,
    config: &OptimizeConfig,
) -> ChatRequest {
    ChatRequest {
        model: model.to_string(),
        messages: vec![
            ChatMessage {
                role: "system".into(),
                content: system_prompt(mode),
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
    mode: OptimizationMode,
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
    let request = build_chat_request(model, original, mode, config);
    let response = client.chat(&request, config.timeout, cancel).await?;
    interpret_chat_response(original, mode, model, response, started.elapsed())
}

pub fn interpret_chat_response(
    original: &str,
    mode: OptimizationMode,
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
        clarification_question: parsed.clarification_question,
        preservation,
        suspicious,
        duration_ms: elapsed.as_millis(),
        prompt_tokens: response.prompt_eval_count,
        completion_tokens: response.eval_count,
        prompt_version: prompt_version().to_string(),
        model: model.to_string(),
        mode,
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ParsedOptimization {
    status: OptimizationStatus,
    rewritten_prompt: String,
    clarification_question: String,
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
    let expected = ["status", "rewritten_prompt", "clarification_question"];
    if object.len() != expected.len() || expected.iter().any(|key| !object.contains_key(*key)) {
        return Err(PsError::MalformedOutput {
            detail: "Structured output did not contain exactly the required fields.".into(),
        });
    }
    let status = required_string(object, "status")?;
    let rewritten = required_string(object, "rewritten_prompt")?;
    let question = required_string(object, "clarification_question")?;

    match status {
        "rewritten" => {
            if !question.trim().is_empty() {
                return Err(PsError::MalformedOutput {
                    detail: "A rewritten result included a clarification question.".into(),
                });
            }
            if rewritten.trim().is_empty() {
                return Err(PsError::MalformedOutput {
                    detail: "A rewritten result did not include a prompt.".into(),
                });
            }
            if selection_matches(rewritten, original) {
                return Ok(unchanged(original));
            }
            Ok(ParsedOptimization {
                status: OptimizationStatus::Rewritten,
                rewritten_prompt: rewritten.to_string(),
                clarification_question: String::new(),
            })
        }
        "unchanged" => {
            if !question.trim().is_empty() {
                return Err(PsError::MalformedOutput {
                    detail: "An unchanged result included a clarification question.".into(),
                });
            }
            if !selection_matches(rewritten, original) {
                return Err(PsError::MalformedOutput {
                    detail: "The model marked the prompt unchanged but returned different text."
                        .into(),
                });
            }
            Ok(unchanged(original))
        }
        "needs_clarification" => {
            let question = question.trim();
            if question.is_empty() {
                return Err(PsError::MalformedOutput {
                    detail: "A clarification result did not include a question.".into(),
                });
            }
            if question.chars().count() > 500 {
                return Err(PsError::MalformedOutput {
                    detail: "The clarification question was unusably long.".into(),
                });
            }
            if !rewritten.trim().is_empty() {
                return Err(PsError::MalformedOutput {
                    detail: "A clarification result also included replacement text.".into(),
                });
            }
            Ok(ParsedOptimization {
                status: OptimizationStatus::NeedsClarification,
                rewritten_prompt: String::new(),
                clarification_question: question.to_string(),
            })
        }
        _ => Err(PsError::MalformedOutput {
            detail: "The model returned an unknown result status.".into(),
        }),
    }
}

fn unchanged(original: &str) -> ParsedOptimization {
    ParsedOptimization {
        status: OptimizationStatus::Unchanged,
        rewritten_prompt: original.to_string(),
        clarification_question: String::new(),
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
        let request =
            build_chat_request("qwen3.5:4b", original, OptimizationMode::Improve, &config());
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
    fn parses_rewritten_unchanged_and_clarification() {
        let original = "fix login it breaks on refresh";
        let rewritten = parse_model_output(
            r#"{"status":"rewritten","rewritten_prompt":"Investigate the login failure after a page refresh. Identify the root cause, implement a focused fix, and verify that the expected login state is preserved.","clarification_question":""}"#,
            original,
        )
        .unwrap();
        assert_eq!(rewritten.status, OptimizationStatus::Rewritten);

        let same = parse_model_output(
            &format!(
                r#"{{"status":"rewritten","rewritten_prompt":{},"clarification_question":""}}"#,
                serde_json::to_string(original).unwrap()
            ),
            original,
        )
        .unwrap();
        assert_eq!(same.status, OptimizationStatus::Unchanged);
        assert_eq!(same.rewritten_prompt, original);

        let unchanged = parse_model_output(
            &format!(
                r#"{{"status":"unchanged","rewritten_prompt":{},"clarification_question":""}}"#,
                serde_json::to_string(original).unwrap()
            ),
            original,
        )
        .unwrap();
        assert_eq!(unchanged.rewritten_prompt, original);

        let question = parse_model_output(
            r#"{"status":"needs_clarification","rewritten_prompt":"","clarification_question":"Should user deletion be permanent or reversible?"}"#,
            "Implement user deletion, but I haven't decided whether it should be permanent or reversible.",
        )
        .unwrap();
        assert_eq!(question.status, OptimizationStatus::NeedsClarification);
        assert!(question.rewritten_prompt.is_empty());
        assert_eq!(
            question.clarification_question,
            "Should user deletion be permanent or reversible?"
        );
    }

    #[test]
    fn rejects_contradictory_empty_and_truncated_payloads() {
        let original =
            "Add a loading spinner to the Submit button while the existing request is in progress.";
        assert!(parse_model_output(
            r#"{"status":"rewritten","rewritten_prompt":"","clarification_question":""}"#,
            original
        )
        .is_err());
        assert!(parse_model_output(
            r#"{"status":"unchanged","rewritten_prompt":"different","clarification_question":""}"#,
            original
        )
        .is_err());
        assert!(parse_model_output(
            r#"{"status":"needs_clarification","rewritten_prompt":"do this","clarification_question":"Which?"}"#,
            original
        )
        .is_err());
        assert!(parse_model_output(
            r#"{"status":"needs_clarification","rewritten_prompt":"","clarification_question":""}"#,
            original
        )
        .is_err());
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
        assert!(parse_model_output(
            r#"{"status":"ship_it","rewritten_prompt":"x","clarification_question":""}"#,
            original
        )
        .is_err());
        assert!(parse_model_output("```json\n{\"status\":\"unchanged\"}\n```", original).is_err());
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
            interpret_chat_response(
                "hello",
                OptimizationMode::Compact,
                "m",
                response,
                Duration::from_millis(5)
            ),
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
            interpret_chat_response(
                "hello",
                OptimizationMode::Improve,
                "m",
                empty,
                Duration::ZERO
            ),
            Err(PsError::EmptyResponse)
        ));
    }
}
