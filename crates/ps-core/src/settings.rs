use crate::endpoint::{self, DEFAULT_OLLAMA_ENDPOINT};
use crate::error::{PsError, PsResult};
use crate::prompts::prompt_version;
use crate::shortcuts::{default_accelerator, normalize_accelerator, ShortcutBinding};
use serde::{Deserialize, Serialize};

pub const SETTINGS_VERSION: u32 = 2;

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
    pub shortcut: ShortcutBinding,
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
            version: SETTINGS_VERSION,
            theme: Theme::Dark,
            launch_at_startup: false,
            run_in_background: true,
            notifications: true,
            shortcuts_paused: false,
            shortcut: ShortcutBinding::new(true, default_accelerator()),
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
        self.shortcut.accelerator = normalize_accelerator(&self.shortcut.accelerator)?;
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
        self.version = SETTINGS_VERSION;
        self.prompt_version = prompt_version().to_string();
        self.onboarding_step = self.onboarding_step.min(5);
        Ok(())
    }
}

/// Read settings saved by this version or by the earlier Improve/Compact build.
pub fn settings_from_stored(value: serde_json::Value) -> PsResult<AppSettings> {
    let value = migrate_stored_settings(value);
    let mut settings: AppSettings =
        serde_json::from_value(value).map_err(|_| PsError::InvalidSettings {
            detail: "Stored settings could not be read.".into(),
        })?;
    settings.validate_and_normalize()?;
    Ok(settings)
}

pub fn stored_settings_need_migration(value: &serde_json::Value) -> bool {
    value.as_object().is_some_and(|object| {
        object.contains_key("improveShortcut")
            || object.contains_key("compactShortcut")
            || object.contains_key("defaultMode")
            || !object.contains_key("shortcut")
    })
}

fn migrate_stored_settings(mut value: serde_json::Value) -> serde_json::Value {
    let Some(object) = value.as_object_mut() else {
        return value;
    };
    if !object.contains_key("shortcut") {
        let improve = object.get("improveShortcut").cloned();
        let compact = object.get("compactShortcut").cloned();
        if let Some(shortcut) = legacy_shortcut(improve, compact) {
            object.insert("shortcut".to_string(), shortcut);
        }
    }
    object.remove("improveShortcut");
    object.remove("compactShortcut");
    object.remove("defaultMode");
    value
}

fn legacy_shortcut(
    improve: Option<serde_json::Value>,
    compact: Option<serde_json::Value>,
) -> Option<serde_json::Value> {
    let improve_on = improve.as_ref().is_some_and(binding_enabled);
    let compact_on = compact.as_ref().is_some_and(binding_enabled);
    if improve_on {
        return improve;
    }
    if compact_on {
        return compact;
    }
    improve.or(compact)
}

fn binding_enabled(value: &serde_json::Value) -> bool {
    value
        .get("enabled")
        .and_then(|flag| flag.as_bool())
        .unwrap_or(false)
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
        assert_eq!(settings.version, SETTINGS_VERSION);
        assert_eq!(settings.ollama_endpoint, DEFAULT_OLLAMA_ENDPOINT);
        assert!(!settings.preview_before_replace);
        assert!(settings.auto_replace);
        assert!((settings.temperature - 0.2).abs() < f32::EPSILON);
        let json = serde_json::to_value(&settings).unwrap();
        assert!(json.get("improveShortcut").is_none());
        assert!(json.get("compactShortcut").is_none());
        assert!(json.get("defaultMode").is_none());
        let restored: AppSettings = serde_json::from_value(json).unwrap();
        assert_eq!(restored.shortcut.accelerator, "Ctrl+Alt+P");
        assert!(restored.shortcut.enabled);
        assert_eq!(restored.theme, Theme::Dark);
        assert_eq!(restored.prompt_version, "1.3.0");
    }

    #[test]
    fn rejects_remote_endpoints_and_bare_shortcuts() {
        let mut settings = AppSettings {
            ollama_endpoint: "https://example.com".into(),
            ..AppSettings::default()
        };
        assert!(settings.validate_and_normalize().is_err());

        let mut settings = AppSettings {
            shortcut: ShortcutBinding::new(true, "P"),
            ..AppSettings::default()
        };
        assert!(matches!(
            settings.validate_and_normalize(),
            Err(PsError::InvalidShortcut { .. })
        ));
    }

    #[test]
    fn legacy_settings_keep_one_shortcut_and_the_selected_model() {
        let mut stored = serde_json::to_value(AppSettings::default()).unwrap();
        let object = stored.as_object_mut().unwrap();
        object.remove("shortcut");
        object.insert("version".into(), serde_json::json!(1));
        object.insert(
            "improveShortcut".into(),
            serde_json::json!({"enabled": true, "accelerator": "Ctrl+Alt+K"}),
        );
        object.insert(
            "compactShortcut".into(),
            serde_json::json!({"enabled": true, "accelerator": "Ctrl+Alt+Shift+P"}),
        );
        object.insert("defaultMode".into(), serde_json::json!("compact"));
        object.insert("activeModel".into(), serde_json::json!("qwen3.5:4b"));
        object.insert("onboardingCompleted".into(), serde_json::json!(true));
        assert!(stored_settings_need_migration(&stored));

        let settings = settings_from_stored(stored).unwrap();
        assert_eq!(settings.shortcut.accelerator, "Ctrl+Alt+K");
        assert!(settings.shortcut.enabled);
        assert_eq!(settings.active_model.as_deref(), Some("qwen3.5:4b"));
        assert!(settings.onboarding_completed);
        assert_eq!(settings.version, SETTINGS_VERSION);
        assert_eq!(settings.prompt_version, "1.3.0");

        let mut disabled_improve = serde_json::to_value(AppSettings::default()).unwrap();
        let object = disabled_improve.as_object_mut().unwrap();
        object.remove("shortcut");
        object.insert(
            "improveShortcut".into(),
            serde_json::json!({"enabled": false, "accelerator": "Ctrl+Alt+K"}),
        );
        object.insert(
            "compactShortcut".into(),
            serde_json::json!({"enabled": true, "accelerator": "option+control+j"}),
        );
        let settings = settings_from_stored(disabled_improve).unwrap();
        assert_eq!(settings.shortcut.accelerator, "Ctrl+Alt+J");
        assert!(settings.shortcut.enabled);

        let mut both_off = serde_json::to_value(AppSettings::default()).unwrap();
        let object = both_off.as_object_mut().unwrap();
        object.remove("shortcut");
        object.insert(
            "improveShortcut".into(),
            serde_json::json!({"enabled": false, "accelerator": "Ctrl+Alt+K"}),
        );
        object.insert(
            "compactShortcut".into(),
            serde_json::json!({"enabled": false, "accelerator": "Ctrl+Alt+J"}),
        );
        let settings = settings_from_stored(both_off).unwrap();
        assert_eq!(settings.shortcut.accelerator, "Ctrl+Alt+K");
        assert!(!settings.shortcut.enabled);
    }
}
