use crate::error::{PsError, PsResult};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShortcutBinding {
    pub enabled: bool,
    pub accelerator: String,
}

impl ShortcutBinding {
    pub fn new(enabled: bool, accelerator: impl Into<String>) -> Self {
        Self {
            enabled,
            accelerator: accelerator.into(),
        }
    }
}

const MOD_ORDER: [&str; 5] = ["Ctrl", "Alt", "Shift", "Command", "Super"];

pub fn normalize_accelerator(input: &str) -> PsResult<String> {
    let parts: Vec<&str> = input
        .split('+')
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .collect();
    if parts.len() < 2 {
        return Err(PsError::InvalidShortcut {
            detail: "Use at least one modifier and a key, such as Ctrl+Alt+P.".into(),
        });
    }

    let (key_raw, modifiers) = parts.split_last().expect("at least two parts");
    let mut normalized_mods = Vec::new();
    for modifier in modifiers {
        normalized_mods.push(normalize_modifier(modifier)?);
    }
    if normalized_mods.is_empty() {
        return Err(PsError::InvalidShortcut {
            detail: "A global shortcut needs a modifier.".into(),
        });
    }
    normalized_mods.sort_by_key(|modifier| {
        MOD_ORDER
            .iter()
            .position(|candidate| candidate == modifier)
            .unwrap_or(MOD_ORDER.len())
    });
    normalized_mods.dedup();

    let key = normalize_key(key_raw)?;
    if MOD_ORDER.contains(&key.as_str()) {
        return Err(PsError::InvalidShortcut {
            detail: "The shortcut must end with a non-modifier key.".into(),
        });
    }

    Ok(format!("{}+{key}", normalized_mods.join("+")))
}

fn normalize_modifier(raw: &str) -> PsResult<String> {
    match raw.to_ascii_lowercase().as_str() {
        "ctrl" | "control" => Ok("Ctrl".to_string()),
        "alt" | "option" | "opt" => Ok("Alt".to_string()),
        "shift" => Ok("Shift".to_string()),
        "cmd" | "command" | "meta" => Ok("Command".to_string()),
        "super" | "win" | "windows" => Ok("Super".to_string()),
        _ => Err(PsError::InvalidShortcut {
            detail: format!("\"{raw}\" is not a supported modifier."),
        }),
    }
}

fn normalize_key(raw: &str) -> PsResult<String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err(PsError::InvalidShortcut {
            detail: "The shortcut is missing its key.".into(),
        });
    }
    if let Some(function) = trimmed
        .strip_prefix(['f', 'F'])
        .filter(|rest| !rest.is_empty() && rest.chars().all(|ch| ch.is_ascii_digit()))
    {
        let number: u8 = function.parse().unwrap_or(0);
        if (1..=24).contains(&number) {
            return Ok(format!("F{number}"));
        }
    }
    let mut chars = trimmed.chars();
    let first = chars.next().unwrap();
    if chars.next().is_none() && first.is_ascii_alphanumeric() {
        return Ok(first.to_ascii_uppercase().to_string());
    }
    Err(PsError::InvalidShortcut {
        detail: format!("\"{trimmed}\" is not a supported shortcut key."),
    })
}

pub fn display_accelerator(canonical: &str, macos: bool) -> String {
    if !macos {
        return canonical.to_string();
    }
    canonical
        .split('+')
        .map(|part| match part {
            "Ctrl" => "Control",
            "Alt" => "Option",
            "Command" => "Command",
            other => other,
        })
        .collect::<Vec<_>>()
        .join("+")
}

pub fn default_accelerator() -> &'static str {
    "Ctrl+Alt+P"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_modifier_order_and_aliases() {
        assert_eq!(
            normalize_accelerator("option+control+p").unwrap(),
            "Ctrl+Alt+P"
        );
        assert_eq!(
            normalize_accelerator("Shift+Alt+Ctrl+P").unwrap(),
            "Ctrl+Alt+Shift+P"
        );
        assert_eq!(
            normalize_accelerator("cmd+shift+k").unwrap(),
            "Shift+Command+K"
        );
    }

    #[test]
    fn rejects_bare_keys_and_unknown_tokens() {
        assert!(normalize_accelerator("P").is_err());
        assert!(normalize_accelerator("Ctrl+Nope").is_err());
        assert!(normalize_accelerator("Ctrl+Alt").is_err());
    }

    #[test]
    fn macos_display_uses_control_and_option() {
        assert_eq!(display_accelerator("Ctrl+Alt+P", true), "Control+Option+P");
        assert_eq!(display_accelerator("Ctrl+Alt+P", false), "Ctrl+Alt+P");
    }
}
