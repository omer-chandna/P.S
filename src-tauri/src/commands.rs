use crate::global::{self, notify};
use crate::platform::{self, host, PermissionState};
use crate::state::{AppState, Attention, OptimizeOutcome, PullJob};
use crate::tray;
use ps_core::hardware::HardwareProfile;
use ps_core::mode::OptimizationMode;
use ps_core::models::{validate_model_name, ListedModel, RecommendationSet};
use ps_core::ollama::OllamaClient;
use ps_core::settings::AppSettings;
use ps_core::shortcuts::{display_accelerator, shortcuts_conflict};
use ps_core::{PsError, PsResult};
use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager};
use uuid::Uuid;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OptimizeRequest {
    pub text: String,
    pub mode: OptimizationMode,
    pub model: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionStatus {
    pub connected: bool,
    pub endpoint: String,
    pub message: String,
    pub runnable_models: usize,
    pub cloud_models: usize,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HardwareResponse {
    pub hardware: HardwareProfile,
    pub recommendations: RecommendationSet,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ShortcutInfo {
    pub improve_accelerator: String,
    pub improve_display: String,
    pub improve_enabled: bool,
    pub compact_accelerator: String,
    pub compact_display: String,
    pub compact_enabled: bool,
    pub paused: bool,
    pub conflict: Option<String>,
    pub macos: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PermissionInfo {
    pub state: &'static str,
    pub title: String,
    pub detail: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppInfo {
    pub name: &'static str,
    pub version: String,
    pub prompt_version: String,
    pub os: &'static str,
    pub arch: &'static str,
    pub supported_platform: bool,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct PullNotice {
    model: String,
    status: String,
    percent: Option<u8>,
    done: bool,
    error: Option<String>,
}

#[tauri::command]
pub fn get_settings(app: AppHandle) -> AppSettings {
    app.state::<AppState>().settings_snapshot()
}

#[tauri::command]
pub fn update_settings(app: AppHandle, settings: AppSettings) -> PsResult<AppSettings> {
    crate::apply_settings(&app, settings)
}

#[tauri::command]
pub async fn ollama_status(app: AppHandle) -> ConnectionStatus {
    let endpoint = app.state::<AppState>().settings_snapshot().ollama_endpoint;
    match refresh_models(&app).await {
        Ok(models) => ConnectionStatus {
            connected: true,
            endpoint,
            message: "Connected to local Ollama.".into(),
            runnable_models: models.iter().filter(|model| model.runnable).count(),
            cloud_models: models.iter().filter(|model| model.cloud).count(),
        },
        Err(error) => ConnectionStatus {
            connected: false,
            endpoint,
            message: error.to_string(),
            runnable_models: 0,
            cloud_models: 0,
        },
    }
}

#[tauri::command]
pub async fn list_models(app: AppHandle) -> PsResult<Vec<ListedModel>> {
    refresh_models(&app).await
}

#[tauri::command]
pub fn activate_model(app: AppHandle, model: String) -> PsResult<AppSettings> {
    ensure_runnable(&app, &model)?;
    let mut settings = app.state::<AppState>().settings_snapshot();
    settings.active_model = Some(model);
    crate::apply_settings(&app, settings)
}

#[tauri::command]
pub async fn delete_model(app: AppHandle, model: String) -> PsResult<Vec<ListedModel>> {
    validate_model_name(&model)?;
    let settings = app.state::<AppState>().settings_snapshot();
    let client = OllamaClient::new(&settings.ollama_endpoint)?;
    client.delete_model(&model).await?;
    let state = app.state::<AppState>();
    let mut current = state.settings_snapshot();
    if current.active_model.as_deref() == Some(model.as_str()) {
        current.active_model = None;
        let _ = crate::apply_settings(&app, current);
    }
    refresh_models(&app).await
}

#[tauri::command]
pub async fn pull_model(app: AppHandle, model: String) -> PsResult<()> {
    validate_model_name(&model)?;
    let state = app.state::<AppState>();
    let job = PullJob {
        id: Uuid::new_v4(),
        cancel: tokio_util::sync::CancellationToken::new(),
    };
    let cancel = job.cancel.clone();
    let id = job.id;
    {
        let mut slot = state
            .pull
            .lock()
            .unwrap_or_else(|poison| poison.into_inner());
        if let Some(existing) = slot.replace(job) {
            existing.cancel.cancel();
        }
    }
    let model_name = model.clone();
    tauri::async_runtime::spawn(async move {
        let settings = app.state::<AppState>().settings_snapshot();
        let result = match OllamaClient::new(&settings.ollama_endpoint) {
            Ok(client) => {
                let progress_app = app.clone();
                let progress_model = model_name.clone();
                client
                    .pull(&model_name, &cancel, move |progress| {
                        emit_pull(
                            &progress_app,
                            &progress_model,
                            &progress.status,
                            progress.percent,
                            progress.done,
                            progress.error,
                        );
                    })
                    .await
            }
            Err(error) => Err(error),
        };
        match result {
            Ok(()) => {
                emit_pull(&app, &model_name, "success", Some(100), true, None);
                let _ = refresh_models(&app).await;
                notify(&app, "Model download finished");
            }
            Err(PsError::Cancelled) => {
                emit_pull(
                    &app,
                    &model_name,
                    "cancelled",
                    None,
                    true,
                    Some("Download cancelled.".into()),
                );
            }
            Err(error) => {
                emit_pull(
                    &app,
                    &model_name,
                    "error",
                    None,
                    true,
                    Some(error.to_string()),
                );
            }
        }
        clear_pull(&app, id);
    });
    Ok(())
}

#[tauri::command]
pub fn cancel_pull(app: AppHandle) -> bool {
    let state = app.state::<AppState>();
    let slot = state
        .pull
        .lock()
        .unwrap_or_else(|poison| poison.into_inner());
    if let Some(job) = slot.as_ref() {
        job.cancel.cancel();
        true
    } else {
        false
    }
}

#[tauri::command]
pub fn hardware_recommendations() -> HardwareResponse {
    let mut hardware = ps_core::hardware::detect_hardware();
    platform::enrich_hardware(&mut hardware);
    let recommendations = ps_core::models::recommend(&hardware);
    HardwareResponse {
        hardware,
        recommendations,
    }
}

#[tauri::command]
pub async fn optimize_text(app: AppHandle, request: OptimizeRequest) -> PsResult<OptimizeOutcome> {
    global::optimize_manual(&app, request.text, request.mode, request.model).await
}

#[tauri::command]
pub fn cancel_optimization(app: AppHandle) -> bool {
    app.state::<AppState>().operations.cancel_current()
}

#[tauri::command]
pub fn attention(app: AppHandle) -> Option<Attention> {
    global::current_attention(&app)
}

#[tauri::command]
pub fn dismiss_attention(app: AppHandle) {
    global::dismiss_attention(&app);
}

#[tauri::command]
pub async fn accept_attention(app: AppHandle, id: Uuid) -> PsResult<()> {
    global::accept_attention(&app, id).await
}

#[tauri::command]
pub fn shortcut_info(app: AppHandle) -> ShortcutInfo {
    let settings = app.state::<AppState>().settings_snapshot();
    let macos = cfg!(target_os = "macos");
    let conflict = if shortcuts_conflict(&settings.improve_shortcut, &settings.compact_shortcut) {
        Some("Improve and Compact are using the same shortcut.".to_string())
    } else {
        None
    };
    ShortcutInfo {
        improve_accelerator: settings.improve_shortcut.accelerator.clone(),
        improve_display: display_accelerator(&settings.improve_shortcut.accelerator, macos),
        improve_enabled: settings.improve_shortcut.enabled,
        compact_accelerator: settings.compact_shortcut.accelerator.clone(),
        compact_display: display_accelerator(&settings.compact_shortcut.accelerator, macos),
        compact_enabled: settings.compact_shortcut.enabled,
        paused: settings.shortcuts_paused,
        conflict,
        macos,
    }
}

#[tauri::command]
pub fn permission_info() -> PermissionInfo {
    match host().accessibility() {
        PermissionState::Granted => PermissionInfo {
            state: "granted",
            title: "Accessibility available".into(),
            detail: "P.S. can read the active window and simulate copy and paste.".into(),
        },
        PermissionState::Missing => PermissionInfo {
            state: "missing",
            title: "Accessibility permission needed".into(),
            detail: "macOS needs Accessibility permission so P.S. can copy and replace the selected text. Open System Settings, enable P.S., then retry.".into(),
        },
        PermissionState::Unsupported => PermissionInfo {
            state: "unsupported",
            title: "This system is not a supported target".into(),
            detail: "Selection capture is implemented for macOS and Windows 11. Manual optimization still runs against a local Ollama server.".into(),
        },
    }
}

#[tauri::command]
pub fn open_permission_settings() -> PsResult<()> {
    host().open_accessibility_settings()
}

#[tauri::command]
pub fn open_external(app: AppHandle, url: String) -> PsResult<()> {
    if !allowed_https(&url) {
        return Err(PsError::InvalidSettings {
            detail: "That address is not in the allowed documentation list.".into(),
        });
    }
    use tauri_plugin_opener::OpenerExt;
    app.opener()
        .open_url(url, None::<&str>)
        .map_err(|_| PsError::InvalidSettings {
            detail: "The link could not be opened.".into(),
        })
}

#[tauri::command]
pub fn app_info(app: AppHandle) -> AppInfo {
    let supported_platform = cfg!(all(target_os = "macos", target_arch = "aarch64"))
        || cfg!(all(target_os = "windows", target_arch = "x86_64"));
    AppInfo {
        name: "P.S.",
        version: app.package_info().version.to_string(),
        prompt_version: ps_core::prompts::prompt_version().to_string(),
        os: std::env::consts::OS,
        arch: std::env::consts::ARCH,
        supported_platform,
    }
}

#[tauri::command]
pub fn finish_onboarding(app: AppHandle) -> PsResult<AppSettings> {
    let mut settings = app.state::<AppState>().settings_snapshot();
    settings.onboarding_completed = true;
    settings.onboarding_step = 5;
    let saved = crate::apply_settings(&app, settings)?;
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.hide();
    }
    Ok(saved)
}

#[tauri::command]
pub fn copy_text(text: String) -> PsResult<()> {
    host().write_clipboard_text(&text)
}

async fn refresh_models(app: &AppHandle) -> PsResult<Vec<ListedModel>> {
    let settings = app.state::<AppState>().settings_snapshot();
    let client = OllamaClient::new(&settings.ollama_endpoint)?;
    let models = client.list_models().await?;
    app.state::<AppState>().set_models(models.clone());
    tray::rebuild(app);
    Ok(models)
}

fn ensure_runnable(app: &AppHandle, model: &str) -> PsResult<()> {
    validate_model_name(model)?;
    let models = app.state::<AppState>().models_snapshot();
    if models.is_empty() {
        return Ok(());
    }
    match models.iter().find(|candidate| candidate.name == model) {
        Some(candidate) if candidate.runnable => Ok(()),
        Some(_) => Err(PsError::CloudModelRejected {
            model: model.to_string(),
        }),
        None => Err(PsError::ModelUnavailable {
            model: model.to_string(),
        }),
    }
}

fn allowed_https(value: &str) -> bool {
    let Ok(url) = reqwest_url(value) else {
        return false;
    };
    if url.scheme() != "https" {
        return false;
    }
    let host = url.host_str().unwrap_or("");
    let path = url.path();
    match host {
        "ollama.com" | "www.ollama.com" => {
            path == "/" || path.starts_with("/download") || path.starts_with("/library")
        }
        "docs.ollama.com" => true,
        "github.com" => path.starts_with("/ollama/ollama"),
        _ => false,
    }
}

fn reqwest_url(value: &str) -> Result<url::Url, ()> {
    url::Url::parse(value).map_err(|_| ())
}

fn emit_pull(
    app: &AppHandle,
    model: &str,
    status: &str,
    percent: Option<u8>,
    done: bool,
    error: Option<String>,
) {
    let _ = app.emit(
        "ps://pull-progress",
        PullNotice {
            model: model.to_string(),
            status: status.to_string(),
            percent,
            done,
            error,
        },
    );
}

fn clear_pull(app: &AppHandle, id: Uuid) {
    let state = app.state::<AppState>();
    let mut slot = state
        .pull
        .lock()
        .unwrap_or_else(|poison| poison.into_inner());
    if slot.as_ref().is_some_and(|job| job.id == id) {
        *slot = None;
    }
}
