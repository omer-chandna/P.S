use ps_core::models::ListedModel;
use ps_core::operation::OperationGuard;
use ps_core::optimizer::OptimizationStatus;
use ps_core::settings::AppSettings;
use ps_core::workflow::PendingReplacement;
use serde::Serialize;
use std::sync::{Arc, Mutex};
use tauri_plugin_global_shortcut::Shortcut;
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

pub struct PullJob {
    pub id: Uuid,
    pub cancel: CancellationToken,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OptimizeOutcome {
    pub operation_id: String,
    pub mode: ps_core::OptimizationMode,
    pub model: String,
    pub prompt_version: String,
    pub status: OptimizationStatus,
    pub original_text: String,
    pub rewritten_prompt: String,
    pub delivery: ps_core::workflow::DeliveryPlan,
    pub warnings: Vec<String>,
    pub duration_ms: u64,
    pub prompt_tokens: Option<u64>,
    pub completion_tokens: Option<u64>,
    pub suspicious: bool,
}

/// In-memory review request. Prompt text is not written to disk.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Attention {
    pub outcome: OptimizeOutcome,
    pub pending_id: Option<Uuid>,
    pub note: String,
}

pub struct AppState {
    pub settings: Mutex<AppSettings>,
    pub operations: Arc<OperationGuard>,
    pub pending: Mutex<Option<PendingReplacement>>,
    pub attention: Mutex<Option<Attention>>,
    pub pull: Mutex<Option<PullJob>>,
    pub models: Mutex<Vec<ListedModel>>,
    pub registered_improve: Mutex<Option<Shortcut>>,
    pub registered_compact: Mutex<Option<Shortcut>>,
}

impl AppState {
    pub fn new(settings: AppSettings) -> Self {
        Self {
            settings: Mutex::new(settings),
            operations: Arc::new(OperationGuard::default()),
            pending: Mutex::new(None),
            attention: Mutex::new(None),
            pull: Mutex::new(None),
            models: Mutex::new(Vec::new()),
            registered_improve: Mutex::new(None),
            registered_compact: Mutex::new(None),
        }
    }

    pub fn settings_snapshot(&self) -> AppSettings {
        self.settings
            .lock()
            .unwrap_or_else(|poison| poison.into_inner())
            .clone()
    }

    pub fn update_settings(&self, settings: AppSettings) {
        *self
            .settings
            .lock()
            .unwrap_or_else(|poison| poison.into_inner()) = settings;
    }

    pub fn set_models(&self, models: Vec<ListedModel>) {
        *self
            .models
            .lock()
            .unwrap_or_else(|poison| poison.into_inner()) = models;
    }

    pub fn models_snapshot(&self) -> Vec<ListedModel> {
        self.models
            .lock()
            .unwrap_or_else(|poison| poison.into_inner())
            .clone()
    }
}
