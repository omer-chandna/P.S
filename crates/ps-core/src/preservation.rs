use regex::Regex;
use serde::{Deserialize, Serialize};
use std::sync::LazyLock;

static CODE_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"`([^`\n]{1,200})`").unwrap());
static FENCE_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?s)```[^\n]*\n(.*?)```").unwrap());
static URL_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r#"https?://[^\s<>"'`)]+"#).unwrap());
static PATH_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r#"(?x)
        (?:
            [A-Za-z]:[\\/](?:[\w.\-]+[\\/])+[\w.\-]+
            |
            (?:[\w.\-]+/){1,12}[\w.\-]+\.[\w]{1,8}
        )
        "#,
    )
    .unwrap()
});
static VERSION_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r#"(?i)\b(?:v\d+\.\d+(?:\.\d+)?|(?:node|python|rust|go|java|macos|windows|ubuntu)\s+\d+(?:\.\d+)*)\b"#,
    )
    .unwrap()
});
static LIMIT_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r#"(?i)\b\d+(?:\.\d+)?\s?(?:ms|milliseconds|seconds|minutes|hours|kb|mb|gb|tb|%|px)\b"#,
    )
    .unwrap()
});
static QUOTED_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#""([^"\n]{4,240})"|'([^'\n]{4,240})'"#).unwrap());
static EXCLUSION_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"(?i)(?:do not|don't|never|must not|without|avoid)\s+([^.\n]{3,140})"#).unwrap()
});

const GENERIC_FILLER: &[&str] = &[
    "think step by step",
    "act as a senior",
    "as a senior engineer",
    "be extremely thorough",
    "follow all best practices",
    "make everything production-ready",
    "provide an extensive explanation",
];

const STOPWORDS: &[&str] = &[
    "the", "and", "for", "with", "that", "this", "from", "into", "your", "any", "all", "are",
    "was", "were", "not", "but", "you", "its", "it's", "than", "then", "them", "they", "have",
    "has", "had", "will", "just", "only", "also", "when", "while", "where", "which", "what", "who",
    "how", "why", "can", "could", "should", "would", "about", "into", "over", "under", "after",
    "before", "again", "there", "their", "been", "being", "does", "doing", "each", "such", "using",
    "used", "use", "our", "out", "off", "per",
];

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PreservationReport {
    pub missing_identifiers: Vec<String>,
    pub missing_paths: Vec<String>,
    pub missing_urls: Vec<String>,
    pub missing_versions: Vec<String>,
    pub missing_limits: Vec<String>,
    pub missing_quotes: Vec<String>,
    pub damaged_code: Vec<String>,
    pub weakened_exclusions: Vec<String>,
    pub unexpected_paths: Vec<String>,
    pub unexpected_versions: Vec<String>,
    pub generic_additions: Vec<String>,
    pub expansion_notes: Vec<String>,
    pub suspicious: bool,
}

impl PreservationReport {
    pub fn clean() -> Self {
        Self {
            missing_identifiers: vec![],
            missing_paths: vec![],
            missing_urls: vec![],
            missing_versions: vec![],
            missing_limits: vec![],
            missing_quotes: vec![],
            damaged_code: vec![],
            weakened_exclusions: vec![],
            unexpected_paths: vec![],
            unexpected_versions: vec![],
            generic_additions: vec![],
            expansion_notes: vec![],
            suspicious: false,
        }
    }

    pub fn warnings(&self) -> Vec<String> {
        let mut warnings = Vec::new();
        push_group(
            &mut warnings,
            "Missing identifier",
            &self.missing_identifiers,
        );
        push_group(&mut warnings, "Missing path", &self.missing_paths);
        push_group(&mut warnings, "Missing URL", &self.missing_urls);
        push_group(&mut warnings, "Missing version", &self.missing_versions);
        push_group(&mut warnings, "Missing limit", &self.missing_limits);
        push_group(&mut warnings, "Missing quoted text", &self.missing_quotes);
        push_group(&mut warnings, "Changed code block", &self.damaged_code);
        push_group(
            &mut warnings,
            "Weakened constraint",
            &self.weakened_exclusions,
        );
        push_group(&mut warnings, "Unexpected path", &self.unexpected_paths);
        push_group(
            &mut warnings,
            "Unexpected version",
            &self.unexpected_versions,
        );
        push_group(&mut warnings, "Generic addition", &self.generic_additions);
        warnings.extend(self.expansion_notes.clone());
        warnings
    }

    fn finish(mut self) -> Self {
        self.suspicious = !self.warnings().is_empty();
        self
    }
}

fn push_group(warnings: &mut Vec<String>, label: &str, items: &[String]) {
    for item in items {
        warnings.push(format!("{label}: {item}"));
    }
}

pub fn check_preservation(original: &str, rewritten: &str) -> PreservationReport {
    let mut report = PreservationReport::clean();
    let original_without_urls = URL_RE.replace_all(original, " ");
    let rewritten_without_urls = URL_RE.replace_all(rewritten, " ");

    for identifier in unique_captures(&CODE_RE, original, 1) {
        if !rewritten.contains(&identifier) {
            report.missing_identifiers.push(identifier);
        }
    }
    for block in unique_captures(&FENCE_RE, original, 1) {
        if !rewritten.contains(block.trim()) {
            let preview: String = block.chars().take(80).collect();
            report.damaged_code.push(preview);
        }
    }
    for url in unique_matches(&URL_RE, original) {
        let cleaned = url.trim_end_matches(['.', ',', ';']);
        if !rewritten.contains(cleaned) {
            report.missing_urls.push(cleaned.to_string());
        }
    }
    for path in unique_matches(&PATH_RE, &original_without_urls) {
        if !rewritten.contains(path) {
            report.missing_paths.push(path.to_string());
        }
    }
    for version in unique_matches(&VERSION_RE, original) {
        if !contains_ignore_ascii_case(rewritten, version) {
            report.missing_versions.push(version.to_string());
        }
    }
    for limit in unique_matches(&LIMIT_RE, original) {
        if !contains_ignore_ascii_case(rewritten, limit) {
            report.missing_limits.push(limit.to_string());
        }
    }
    for quote in quoted_requirements(original) {
        if !rewritten.contains(&quote) {
            report.missing_quotes.push(quote);
        }
    }
    for exclusion in exclusion_phrases(original) {
        if !exclusion_preserved(&exclusion, rewritten) {
            report.weakened_exclusions.push(exclusion);
        }
    }

    for path in unique_matches(&PATH_RE, &rewritten_without_urls) {
        if !original.contains(path) {
            report.unexpected_paths.push(path.to_string());
        }
    }
    for version in unique_matches(&VERSION_RE, rewritten) {
        if !contains_ignore_ascii_case(original, version) {
            report.unexpected_versions.push(version.to_string());
        }
    }
    for phrase in GENERIC_FILLER {
        if contains_ignore_ascii_case(rewritten, phrase)
            && !contains_ignore_ascii_case(original, phrase)
        {
            report.generic_additions.push((*phrase).to_string());
        }
    }

    let original_len = original.chars().count();
    let rewritten_len = rewritten.chars().count();
    if rewritten_len > original_len {
        let added = rewritten_len - original_len;
        let ratio = rewritten_len as f32 / (original_len.max(1) as f32);
        if (original_len < 500 && added > 800) || (ratio > 2.5 && added > 1200) {
            report.expansion_notes.push(
                "Rewrite expands the prompt substantially beyond the original request.".into(),
            );
        }
    }

    report.finish()
}

fn unique_matches<'a>(regex: &Regex, text: &'a str) -> Vec<&'a str> {
    let mut found = Vec::new();
    for matched in regex.find_iter(text) {
        let value = matched.as_str();
        if !found.contains(&value) {
            found.push(value);
        }
    }
    found
}

fn unique_captures(regex: &Regex, text: &str, group: usize) -> Vec<String> {
    let mut found = Vec::new();
    for captures in regex.captures_iter(text) {
        if let Some(value) = captures.get(group) {
            let value = value.as_str().to_string();
            if !value.trim().is_empty() && !found.contains(&value) {
                found.push(value);
            }
        }
    }
    found
}

fn quoted_requirements(original: &str) -> Vec<String> {
    let mut found = Vec::new();
    for captures in QUOTED_RE.captures_iter(original) {
        let quote = captures
            .get(1)
            .or_else(|| captures.get(2))
            .map(|item| item.as_str())
            .unwrap_or("");
        if quote_is_technical(quote) && !found.iter().any(|existing: &String| existing == quote) {
            found.push(quote.to_string());
        }
    }
    found
}

fn quote_is_technical(quote: &str) -> bool {
    let lower = quote.to_ascii_lowercase();
    lower.contains("error")
        || lower.contains("exception")
        || lower.contains("fail")
        || lower.contains("undefined")
        || lower.contains("cannot")
        || lower.contains("panic")
        || lower.contains("traceback")
        || PATH_RE.is_match(quote)
}

fn exclusion_phrases(original: &str) -> Vec<String> {
    let mut found = Vec::new();
    for captures in EXCLUSION_RE.captures_iter(original) {
        if let Some(phrase) = captures.get(1) {
            let phrase = phrase.as_str().trim().trim_end_matches(['.', ',', ';']);
            if !phrase.is_empty() && !found.iter().any(|existing: &String| existing == phrase) {
                found.push(phrase.to_string());
            }
        }
    }
    found
}

fn exclusion_preserved(phrase: &str, rewritten: &str) -> bool {
    let tokens = significant_tokens(phrase);
    if tokens.is_empty() {
        return true;
    }
    let present = tokens
        .iter()
        .filter(|token| contains_ignore_ascii_case(rewritten, token))
        .count();
    let required = (tokens.len() * 2).div_ceil(3);
    present >= required.max(1)
}

fn significant_tokens(phrase: &str) -> Vec<String> {
    phrase
        .split(|ch: char| !ch.is_alphanumeric() && ch != '_' && ch != '-')
        .filter(|token| {
            token.len() >= 3 && !STOPWORDS.contains(&token.to_ascii_lowercase().as_str())
        })
        .map(ToOwned::to_owned)
        .collect()
}

fn contains_ignore_ascii_case(haystack: &str, needle: &str) -> bool {
    haystack
        .to_ascii_lowercase()
        .contains(&needle.to_ascii_lowercase())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_a_faithful_rewrite_of_an_explicit_constraint() {
        let original = "Update src/auth/session.ts without changing the public API. Keep compatibility with Node 22.";
        let rewritten = "Update src/auth/session.ts. Keep the public API unchanged and preserve compatibility with Node 22.";
        let report = check_preservation(original, rewritten);
        assert!(!report.suspicious, "{:?}", report.warnings());
    }

    #[test]
    fn flags_a_dropped_path_version_and_constraint() {
        let original = "Update src/auth/session.ts without changing the public API. Keep compatibility with Node 22.";
        let rewritten = "Update the session helper and modernize the API.";
        let report = check_preservation(original, rewritten);
        assert!(report.suspicious);
        assert!(report
            .missing_paths
            .iter()
            .any(|path| path.contains("session.ts")));
        assert!(report
            .missing_versions
            .iter()
            .any(|version| version.contains("22")));
        assert!(!report.weakened_exclusions.is_empty());
    }

    #[test]
    fn flags_new_paths_versions_and_generic_filler() {
        let original = "Fix the timeout in the retry helper.";
        let rewritten = "Fix the timeout in src/net/retry.ts for Node 24. Think step by step and act as a senior engineer.";
        let report = check_preservation(original, rewritten);
        assert!(report.suspicious);
        assert!(!report.unexpected_paths.is_empty());
        assert!(!report.unexpected_versions.is_empty());
        assert!(report.generic_additions.len() >= 2);
    }

    #[test]
    fn does_not_treat_ordinary_rephrasing_as_suspicious() {
        let original = "fix login it breaks on refresh";
        let rewritten = "Investigate the login failure after a page refresh. Identify the root cause, implement a focused fix, and verify that the expected login state is preserved.";
        let report = check_preservation(original, rewritten);
        assert!(!report.suspicious, "{:?}", report.warnings());
    }

    #[test]
    fn preserves_backticks_urls_limits_and_code_fences() {
        let original = "Call `refreshToken` at https://example.com/callback within 30 seconds.\n```\nconst code = 1;\n```\nThe console says \"TypeError: refreshToken is undefined\".";
        let rewritten = "Call `refreshToken` at https://example.com/callback within 30 seconds and inspect the existing flow.\n```\nconst code = 1;\n```\nKeep the reported error \"TypeError: refreshToken is undefined\" in mind.";
        let report = check_preservation(original, rewritten);
        assert!(!report.suspicious, "{:?}", report.warnings());

        let damaged = "Call the token helper within 30 seconds.";
        let damaged_report = check_preservation(original, damaged);
        assert!(damaged_report.suspicious);
        assert!(!damaged_report.missing_identifiers.is_empty());
        assert!(!damaged_report.missing_urls.is_empty());
    }
}
