use crate::endpoint::{self, DEFAULT_OLLAMA_ENDPOINT};
use crate::error::{PsError, PsResult};
use crate::mode::OptimizationMode;
use crate::prompts::prompt_version;
use crate::shortcuts::{
    default_compact_accelerator, default_improve_accelerator, normalize_accelerator,
    shortcuts_conflict, ShortcutBinding,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Theme {
    Dark,
    Light,
    System,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    pub version: u32,
    pub theme: Theme,
    pub launch_at_startup: bool,
    pub run_in_background: bool,
    pub notifications: bool,
    pub shortcuts_paused: bool,
    pub improve_shortcut: ShortcutBinding,
    pub compact_shortcut: ShortcutBinding,
    pub default_mode: OptimizationMode,
    pub auto_replace: bool,
    pub preview_before_replace: bool,
    pub prompt_version: String,
    pub ollama_endpoint: String,
    pub active_model: Option<String>,
    pub temperature: f32,
    pub top_p: f32,
    /// `None` selects a budget from the input length.
    pub output_token_limit: Option<u32>,
    pub request_timeout_secs: u64,
    pub reasoning_enabled: bool,
    pub keep_alive: String,
    pub onboarding_completed: bool,
    pub onboarding_step: u8,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            version: 1,
            theme: Theme::Dark,
            launch_at_startup: false,
            run_in_background: true,
            notifications: true,
            shortcuts_paused: false,
            improve_shortcut: ShortcutBinding::new(true, default_improve_accelerator()),
            compact_shortcut: ShortcutBinding::new(true, default_compact_accelerator()),
            default_mode: OptimizationMode::Improve,
            auto_replace: true,
            preview_before_replace: false,
            prompt_version: prompt_version().to_string(),
            ollama_endpoint: DEFAULT_OLLAMA_ENDPOINT.to_string(),
            active_model: None,
            temperature: 0.2,
            top_p: 0.9,
            output_token_limit: None,
            request_timeout_secs: 90,
            reasoning_enabled: false,
            keep_alive: "10m".to_string(),
            onboarding_completed: false,
            onboarding_step: 0,
        }
    }
}

impl AppSettings {
    pub fn validate_and_normalize(&mut self) -> PsResult<()> {
        self.ollama_endpoint = endpoint::normalize_local_endpoint(&self.ollama_endpoint)?;
        self.improve_shortcut.accelerator =
            normalize_accelerator(&self.improve_shortcut.accelerator)?;
        self.compact_shortcut.accelerator =
            normalize_accelerator(&self.compact_shortcut.accelerator)?;
        if shortcuts_conflict(&self.improve_shortcut, &self.compact_shortcut) {
            return Err(PsError::ShortcutConflict {
                shortcut: self.improve_shortcut.accelerator.clone(),
                detail: "Improve and Compact cannot use the same shortcut.".into(),
            });
        }
        if !(0.0..=2.0).contains(&self.temperature) {
            return Err(PsError::InvalidSettings {
                detail: "Temperature must be between 0 and 2.".into(),
            });
        }
        if !(0.0..=1.0).contains(&self.top_p) {
            return Err(PsError::InvalidSettings {
                detail: "Top-p must be between 0 and 1.".into(),
            });
        }
        if !(5..=600).contains(&self.request_timeout_secs) {
            return Err(PsError::InvalidSettings {
                detail: "Timeout must be between 5 and 600 seconds.".into(),
            });
        }
        if let Some(limit) = self.output_token_limit {
            if !(128..=32_768).contains(&limit) {
                return Err(PsError::InvalidSettings {
                    detail: "Output token limit must be between 128 and 32768, or automatic."
                        .into(),
                });
            }
        }
        if !valid_keep_alive(&self.keep_alive) {
            return Err(PsError::InvalidSettings {
                detail: "Keep-alive must look like 10m, 30s, or 1h.".into(),
            });
        }
        self.prompt_version = prompt_version().to_string();
        self.onboarding_step = self.onboarding_step.min(5);
        Ok(())
    }
}

fn valid_keep_alive(value: &str) -> bool {
    let value = value.trim();
    let Some((amount, unit)) = value.split_at_checked(value.len().saturating_sub(1)) else {
        return false;
    };
    matches!(unit, "s" | "m" | "h")
        && !amount.is_empty()
        && amount.chars().all(|ch| ch.is_ascii_digit())
        && amount.len() <= 4
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_are_valid_and_round_trip() {
        let mut settings = AppSettings::default();
        settings.validate_and_normalize().unwrap();
        assert_eq!(settings.ollama_endpoint, DEFAULT_OLLAMA_ENDPOINT);
        assert!(!settings.preview_before_replace);
        assert!(settings.auto_replace);
        assert!((settings.temperature - 0.2).abs() < f32::EPSILON);
        let json = serde_json::to_string(&settings).unwrap();
        let restored: AppSettings = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.improve_shortcut.accelerator, "Ctrl+Alt+P");
        assert_eq!(restored.default_mode, OptimizationMode::Improve);
        assert_eq!(restored.theme, Theme::Dark);
    }

    #[test]
    fn rejects_remote_endpoints_and_shortcut_conflicts() {
        let mut settings = AppSettings {
            ollama_endpoint: "https://example.com".into(),
            ..AppSettings::default()
        };
        assert!(settings.validate_and_normalize().is_err());

        let mut settings = AppSettings {
            compact_shortcut: ShortcutBinding::new(true, "Ctrl+Alt+P"),
            ..AppSettings::default()
        };
        assert!(matches!(
            settings.validate_and_normalize(),
            Err(PsError::ShortcutConflict { .. })
        ));
    }
}
