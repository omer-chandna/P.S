use serde::Serialize;

/// Recoverable failures. Messages are safe to show in the interface and must not
/// contain selected prompt text.
#[derive(Debug, thiserror::Error, Clone, PartialEq, Eq)]
pub enum PsError {
    #[error("Ollama is not reachable at {endpoint}. {detail}")]
    OllamaUnavailable { endpoint: String, detail: String },

    #[error("The model \"{model}\" is not available locally.")]
    ModelUnavailable { model: String },

    #[error("The optimization timed out before a result was ready.")]
    Timeout,

    #[error("The model returned an empty response.")]
    EmptyResponse,

    #[error("The model response could not be used. {detail}")]
    MalformedOutput { detail: String },

    #[error("The model response was truncated before it finished.")]
    TruncatedOutput,

    #[error("The rewrite failed preservation checks. {detail}")]
    PreservationFailed { detail: String },

    #[error("No selected text was captured.")]
    NoSelection,

    #[error("The clipboard could not be used. {detail}")]
    ClipboardUnavailable { detail: String },

    #[error("The clipboard changed before replacement could finish.")]
    ClipboardChanged,

    #[error("The original window is no longer focused, so the text was not replaced.")]
    FocusChanged,

    #[error("The selection changed, so the text was not replaced.")]
    SelectionChanged,

    #[error("P.S. needs {permission} permission to continue.")]
    MissingPermissions { permission: String },

    #[error("The shortcut {shortcut} could not be registered. {detail}")]
    ShortcutConflict { shortcut: String, detail: String },

    #[error("{detail}")]
    InvalidShortcut { detail: String },

    #[error("An optimization is already running.")]
    AlreadyRunning,

    #[error("The optimization was cancelled.")]
    Cancelled,

    #[error("{detail}")]
    InvalidEndpoint { detail: String },

    #[error("\"{model}\" is a cloud model. P.S. only runs local models.")]
    CloudModelRejected { model: String },

    #[error("{detail}")]
    InvalidModelName { detail: String },

    #[error("{detail}")]
    InvalidSettings { detail: String },

    #[error("The selection is too long to optimize safely.")]
    PromptTooLong,

    #[error("Text capture and replacement are available on macOS and Windows only.")]
    UnsupportedPlatform,

    #[error("P.S. closed before the optimization finished. The original text was left unchanged.")]
    Shutdown,

    #[error("{detail}")]
    ModelDownload { detail: String },
}

impl PsError {
    pub fn category(&self) -> &'static str {
        match self {
            Self::OllamaUnavailable { .. } => "ollama_unavailable",
            Self::ModelUnavailable { .. } => "model_unavailable",
            Self::Timeout => "timeout",
            Self::EmptyResponse => "empty_response",
            Self::MalformedOutput { .. } => "malformed_output",
            Self::TruncatedOutput => "truncated_output",
            Self::PreservationFailed { .. } => "preservation_failed",
            Self::NoSelection => "no_selection",
            Self::ClipboardUnavailable { .. } => "clipboard_unavailable",
            Self::ClipboardChanged => "clipboard_changed",
            Self::FocusChanged => "focus_changed",
            Self::SelectionChanged => "selection_changed",
            Self::MissingPermissions { .. } => "missing_permissions",
            Self::ShortcutConflict { .. } => "shortcut_conflict",
            Self::InvalidShortcut { .. } => "invalid_shortcut",
            Self::AlreadyRunning => "already_running",
            Self::Cancelled => "cancelled",
            Self::InvalidEndpoint { .. } => "invalid_endpoint",
            Self::CloudModelRejected { .. } => "cloud_model",
            Self::InvalidModelName { .. } => "invalid_model_name",
            Self::InvalidSettings { .. } => "invalid_settings",
            Self::PromptTooLong => "prompt_too_long",
            Self::UnsupportedPlatform => "unsupported_platform",
            Self::Shutdown => "shutdown",
            Self::ModelDownload { .. } => "model_download",
        }
    }
}

impl Serialize for PsError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("PsError", 2)?;
        state.serialize_field("category", self.category())?;
        state.serialize_field("message", &self.to_string())?;
        state.end()
    }
}

pub type PsResult<T> = Result<T, PsError>;
