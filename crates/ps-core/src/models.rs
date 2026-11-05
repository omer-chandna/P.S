use crate::error::{PsError, PsResult};
use crate::hardware::HardwareProfile;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PresetCategory {
    Fast,
    Balanced,
    Quality,
}

impl PresetCategory {
    pub fn label(self) -> &'static str {
        match self {
            Self::Fast => "Fast",
            Self::Balanced => "Balanced",
            Self::Quality => "Quality",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelPreset {
    pub category: PresetCategory,
    pub model: &'static str,
    /// Approximate registry download size. Not a runtime memory measurement.
    pub download_size_label: &'static str,
    /// Conservative runtime memory budget in bytes.
    pub runtime_memory_bytes: u64,
    pub summary: &'static str,
}

const MIB: u64 = 1024 * 1024;
const GIB: u64 = 1024 * MIB;

/// Initial candidates, not a benchmark ranking.
pub fn recommended_presets() -> [ModelPreset; 3] {
    [
        ModelPreset {
            category: PresetCategory::Fast,
            model: "qwen3.5:2b",
            download_size_label: "about 2 GB",
            runtime_memory_bytes: 3 * GIB,
            summary: "Lower memory use and quicker rewrites.",
        },
        ModelPreset {
            category: PresetCategory::Balanced,
            model: "qwen3.5:4b",
            download_size_label: "about 3 GB",
            runtime_memory_bytes: 6 * GIB,
            summary: "Default suggestion. Preserves instructions with moderate latency.",
        },
        ModelPreset {
            category: PresetCategory::Quality,
            model: "qwen3.5:9b",
            download_size_label: "about 6 GB",
            runtime_memory_bytes: 10 * GIB,
            summary: "Higher capacity for long, constraint-heavy prompts.",
        },
    ]
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MemoryFit {
    Comfortable,
    Tight,
    Unlikely,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PresetRecommendation {
    pub category: PresetCategory,
    pub model: String,
    pub download_size_label: String,
    pub runtime_memory_bytes: u64,
    pub summary: String,
    pub fit: MemoryFit,
    pub suggested: bool,
    pub note: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecommendationSet {
    pub presets: Vec<PresetRecommendation>,
    pub suggested_model: Option<String>,
    pub hardware_limited: bool,
    pub notes: Vec<String>,
}

pub fn recommend(profile: &HardwareProfile) -> RecommendationSet {
    let budget = profile.recommendation_budget_bytes();
    let mut presets = Vec::new();
    for preset in recommended_presets() {
        let (fit, note) = match budget {
            None => (
                MemoryFit::Unknown,
                "Memory could not be measured, so this is not a hardware match.".to_string(),
            ),
            Some(bytes) if bytes >= preset.runtime_memory_bytes => (
                MemoryFit::Comfortable,
                "Available memory covers the conservative runtime budget.".to_string(),
            ),
            Some(bytes) if bytes >= preset.runtime_memory_bytes.saturating_mul(3) / 4 => (
                MemoryFit::Tight,
                "Available memory is close to the conservative runtime budget.".to_string(),
            ),
            Some(_) => (
                MemoryFit::Unlikely,
                "Available memory is below the conservative runtime budget.".to_string(),
            ),
        };
        presets.push(PresetRecommendation {
            category: preset.category,
            model: preset.model.to_string(),
            download_size_label: preset.download_size_label.to_string(),
            runtime_memory_bytes: preset.runtime_memory_bytes,
            summary: preset.summary.to_string(),
            fit,
            suggested: false,
            note,
        });
    }

    let suggested_index = if budget.is_none() {
        presets
            .iter()
            .position(|preset| preset.category == PresetCategory::Balanced)
    } else {
        pick_suggestion(&presets)
    };
    let mut suggested_model = None;
    if let Some(index) = suggested_index {
        presets[index].suggested = true;
        suggested_model = Some(presets[index].model.clone());
    }

    let mut notes = profile.detection_notes.clone();
    if let Some(gpu_bytes) = profile.gpu_memory_bytes {
        if let Some(name) = &profile.gpu_name {
            notes.push(format!(
                "Detected {name} with about {} GB of GPU memory. P.S. still recommends from system memory so a model is not selected on an uncertain GPU figure.",
                gpu_bytes / GIB
            ));
        }
    }
    notes.push("Suggestions are optional. Any compatible local model can be selected.".to_string());

    RecommendationSet {
        presets,
        suggested_model,
        hardware_limited: budget.is_none(),
        notes,
    }
}

fn pick_suggestion(presets: &[PresetRecommendation]) -> Option<usize> {
    let comfortable = |category: PresetCategory| {
        presets.iter().position(|preset| {
            preset.category == category && matches!(preset.fit, MemoryFit::Comfortable)
        })
    };
    comfortable(PresetCategory::Balanced)
        .or_else(|| comfortable(PresetCategory::Fast))
        .or_else(|| comfortable(PresetCategory::Quality))
        .or_else(|| {
            presets.iter().position(|preset| {
                preset.category == PresetCategory::Fast
                    && matches!(preset.fit, MemoryFit::Tight | MemoryFit::Comfortable)
            })
        })
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListedModel {
    pub name: String,
    pub size_bytes: u64,
    pub digest: Option<String>,
    pub parameter_size: Option<String>,
    pub quantization: Option<String>,
    pub family: Option<String>,
    pub remote_host: Option<String>,
    pub remote_model: Option<String>,
    pub cloud: bool,
    pub runnable: bool,
}

pub fn is_cloud_model(name: &str, remote_host: Option<&str>, remote_model: Option<&str>) -> bool {
    if remote_host.is_some_and(|value| !value.trim().is_empty()) {
        return true;
    }
    if remote_model.is_some_and(|value| !value.trim().is_empty()) {
        return true;
    }
    let lower = name.to_ascii_lowercase();
    lower.ends_with("-cloud")
        || lower.contains(":cloud")
        || lower.contains("-cloud:")
        || lower.contains(".cloud")
}

#[derive(Debug, Clone, Default)]
pub struct ModelParts {
    pub name: String,
    pub size_bytes: u64,
    pub digest: Option<String>,
    pub parameter_size: Option<String>,
    pub quantization: Option<String>,
    pub family: Option<String>,
    pub remote_host: Option<String>,
    pub remote_model: Option<String>,
}

pub fn classify_model(parts: ModelParts) -> ListedModel {
    let cloud = is_cloud_model(
        &parts.name,
        parts.remote_host.as_deref(),
        parts.remote_model.as_deref(),
    );
    ListedModel {
        name: parts.name,
        size_bytes: parts.size_bytes,
        digest: parts.digest,
        parameter_size: parts.parameter_size,
        quantization: parts.quantization,
        family: parts.family,
        remote_host: parts.remote_host,
        remote_model: parts.remote_model,
        cloud,
        runnable: !cloud,
    }
}

pub fn validate_model_name(name: &str) -> PsResult<()> {
    let name = name.trim();
    if name.is_empty() || name.len() > 128 {
        return Err(PsError::InvalidModelName {
            detail: "Model names must be between 1 and 128 characters.".into(),
        });
    }
    if is_cloud_model(name, None, None) {
        return Err(PsError::CloudModelRejected {
            model: name.to_string(),
        });
    }
    let characters_ok = name
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, ':' | '.' | '_' | '-' | '/'));
    if !characters_ok
        || name.contains("..")
        || name.contains("//")
        || name.starts_with('/')
        || name.ends_with('/')
        || name.starts_with(':')
        || name.ends_with(':')
    {
        return Err(PsError::InvalidModelName {
            detail: "Use a local Ollama model name such as qwen3.5:4b.".into(),
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn profile(available_gib: u64, total_gib: u64) -> HardwareProfile {
        HardwareProfile {
            os: "macos".into(),
            arch: "aarch64".into(),
            total_memory_bytes: total_gib * GIB,
            available_memory_bytes: available_gib * GIB,
            gpu_name: None,
            gpu_memory_bytes: None,
            apple_silicon: true,
            detection_notes: vec!["Apple Silicon uses unified memory.".into()],
        }
    }

    #[test]
    fn balanced_is_the_default_when_memory_allows_it() {
        let set = recommend(&profile(12, 16));
        assert_eq!(set.suggested_model.as_deref(), Some("qwen3.5:4b"));
        let balanced = set
            .presets
            .iter()
            .find(|preset| preset.category == PresetCategory::Balanced)
            .unwrap();
        assert!(balanced.suggested);
        assert_eq!(balanced.fit, MemoryFit::Comfortable);
    }

    #[test]
    fn small_memory_suggests_fast_and_marks_quality_unlikely() {
        let set = recommend(&profile(4, 8));
        assert_eq!(set.suggested_model.as_deref(), Some("qwen3.5:2b"));
        let quality = set
            .presets
            .iter()
            .find(|preset| preset.category == PresetCategory::Quality)
            .unwrap();
        assert_eq!(quality.fit, MemoryFit::Unlikely);
        assert!(!quality.suggested);
    }

    #[test]
    fn unknown_hardware_explains_the_limit_and_still_highlights_balanced() {
        let set = recommend(&profile(0, 0));
        assert!(set.hardware_limited);
        assert_eq!(set.suggested_model.as_deref(), Some("qwen3.5:4b"));
        assert!(set
            .presets
            .iter()
            .all(|preset| preset.fit == MemoryFit::Unknown));
    }

    #[test]
    fn cloud_names_are_not_runnable() {
        let model = classify_model(ModelParts {
            name: "gpt-oss:120b-cloud".into(),
            ..ModelParts::default()
        });
        assert!(model.cloud);
        assert!(!model.runnable);
        let remote = classify_model(ModelParts {
            name: "llama3.2".into(),
            size_bytes: 10,
            remote_host: Some("https://ollama.com".into()),
            ..ModelParts::default()
        });
        assert!(remote.cloud);
        assert!(validate_model_name("qwen3.5:4b").is_ok());
        assert!(validate_model_name("qwen3.5:9b-cloud").is_err());
        assert!(validate_model_name("http://evil").is_err());
        assert!(validate_model_name("../secret").is_err());
    }
}
