use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OptimizationMode {
    Improve,
    Compact,
}

impl OptimizationMode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Improve => "improve",
            Self::Compact => "compact",
        }
    }
}
