use serde::{Deserialize, Serialize};

/// Hardware facts used only to phrase conservative model suggestions.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HardwareProfile {
    pub os: String,
    pub arch: String,
    pub total_memory_bytes: u64,
    pub available_memory_bytes: u64,
    pub gpu_name: Option<String>,
    pub gpu_memory_bytes: Option<u64>,
    pub apple_silicon: bool,
    pub detection_notes: Vec<String>,
}

impl HardwareProfile {
    pub fn memory_known(&self) -> bool {
        self.total_memory_bytes > 0
    }

    /// Budget recommendations against available memory.
    /// Downloaded file size is not treated as the runtime requirement.
    pub fn recommendation_budget_bytes(&self) -> Option<u64> {
        if self.available_memory_bytes > 0 {
            Some(self.available_memory_bytes)
        } else if self.total_memory_bytes > 0 {
            Some(self.total_memory_bytes.saturating_mul(6) / 10)
        } else {
            None
        }
    }
}

pub fn detect_hardware() -> HardwareProfile {
    use sysinfo::{MemoryRefreshKind, RefreshKind, System};

    let system = System::new_with_specifics(
        RefreshKind::nothing().with_memory(MemoryRefreshKind::everything()),
    );
    let apple_silicon = cfg!(all(target_os = "macos", target_arch = "aarch64"));
    let mut notes = vec![
        "Recommendations use available system memory and stay conservative.".to_string(),
        "A model's download size is smaller than the memory it needs while running.".to_string(),
    ];
    if apple_silicon {
        notes.push(
            "Apple Silicon uses unified memory. Available memory is the budget for local models."
                .to_string(),
        );
    } else {
        notes.push(
            "Discrete GPU memory is included only when the platform can read it reliably. Otherwise choose a model manually."
                .to_string(),
        );
    }
    if system.total_memory() == 0 {
        notes.push(
            "Memory information could not be read. Suggestions below are not based on this machine."
                .to_string(),
        );
    }

    HardwareProfile {
        os: std::env::consts::OS.to_string(),
        arch: std::env::consts::ARCH.to_string(),
        total_memory_bytes: system.total_memory(),
        available_memory_bytes: system.available_memory(),
        gpu_name: None,
        gpu_memory_bytes: None,
        apple_silicon,
        detection_notes: notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unknown_memory_has_no_budget() {
        let profile = HardwareProfile {
            os: "test".into(),
            arch: "test".into(),
            total_memory_bytes: 0,
            available_memory_bytes: 0,
            gpu_name: None,
            gpu_memory_bytes: None,
            apple_silicon: false,
            detection_notes: vec![],
        };
        assert!(profile.recommendation_budget_bytes().is_none());
    }

    #[test]
    fn prefers_available_memory_over_a_fraction_of_total() {
        let profile = HardwareProfile {
            os: "macos".into(),
            arch: "aarch64".into(),
            total_memory_bytes: 16 * 1024 * 1024 * 1024,
            available_memory_bytes: 9 * 1024 * 1024 * 1024,
            gpu_name: None,
            gpu_memory_bytes: None,
            apple_silicon: true,
            detection_notes: vec![],
        };
        assert_eq!(
            profile.recommendation_budget_bytes(),
            Some(9 * 1024 * 1024 * 1024)
        );
    }
}
