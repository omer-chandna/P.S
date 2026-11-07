use crate::platform::{self, host};
use crate::state::{AppState, Attention, OptimizeOutcome};
use crate::tones::{self, Tone};
use crate::tray;
use ps_core::clipboard::selection_matches;
use ps_core::mode::OptimizationMode;
use ps_core::models::validate_model_name;
use ps_core::ollama::OllamaClient;
use ps_core::optimizer::{optimize_prompt, OptimizationAnalysis, OptimizeConfig};
use ps_core::settings::AppSettings;
use ps_core::workflow::{plan_delivery, DeliveryContext, DeliveryPlan, PendingReplacement};
use ps_core::{PsError, PsResult};
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_notification::NotificationExt;
use uuid::Uuid;

pub fn spawn(app: AppHandle, mode: OptimizationMode) {
    tauri::async_runtime::spawn(async move {
        if let Err(error) = run_global(&app, mode).await {
            tones::play(Tone::End);
            notify(&app, &error.to_string());
            let _ = app.emit("ps://notice", error.to_string());
        }
        tray::set_tooltip(&app, "P.S. Local Prompt Optimizer");
    });
}

pub async fn optimize_manual(
    app: &AppHandle,
    text: String,
    mode: OptimizationMode,
    model_override: Option<String>,
) -> PsResult<OptimizeOutcome> {
    let state = app.state::<AppState>();
    let lease = state.operations.try_begin()?;
    let settings = state.settings_snapshot();
    let model = resolve_model(&settings, model_override)?;
    let client = OllamaClient::new(&settings.ollama_endpoint)?;
    let analysis = optimize_prompt(
        &client,
        &model,
        &text,
        mode,
        &OptimizeConfig::from(&settings),
        &lease.cancellation(),
    )
    .await?;
    let delivery = plan_delivery(
        analysis.status,
        analysis.suspicious,
        &DeliveryContext {
            manual: true,
            auto_replace: false,
            preview_before_replace: true,
            target_still_valid: None,
            selection_still_matches: None,
        },
    );
    Ok(to_outcome(lease.id(), &text, &analysis, delivery))
}

pub fn current_attention(app: &AppHandle) -> Option<Attention> {
    app.state::<AppState>()
        .attention
        .lock()
        .unwrap_or_else(|poison| poison.into_inner())
        .clone()
}

pub fn dismiss_attention(app: &AppHandle) {
    let state = app.state::<AppState>();
    *state
        .attention
        .lock()
        .unwrap_or_else(|poison| poison.into_inner()) = None;
    *state
        .pending
        .lock()
        .unwrap_or_else(|poison| poison.into_inner()) = None;
}

pub async fn accept_attention(app: &AppHandle, id: Uuid) -> PsResult<()> {
    let state = app.state::<AppState>();
    let pending = state
        .pending
        .lock()
        .unwrap_or_else(|poison| poison.into_inner())
        .clone()
        .filter(|item| item.id == id)
        .ok_or(PsError::SelectionChanged)?;
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.hide();
    }
    let pending_for_paste = pending.clone();
    let pasted = tauri::async_runtime::spawn_blocking(move || paste_pending(&pending_for_paste))
        .await
        .map_err(|_| PsError::Shutdown)?;
    if let Err(error) = pasted {
        crate::show_main(app);
        return Err(error);
    }
    dismiss_attention(app);
    notify(app, "Prompt updated");
    Ok(())
}

fn paste_pending(pending: &PendingReplacement) -> PsResult<()> {
    let host = host();
    std::thread::sleep(Duration::from_millis(160));
    host.focus(&pending.target)?;
    std::thread::sleep(Duration::from_millis(180));
    let current = host.current_window()?;
    if !current.same_target(&pending.target) {
        return Err(PsError::FocusChanged);
    }
    let selected = platform::capture_selection(host)?;
    if !selection_matches(&pending.original_text, &selected) {
        return Err(PsError::SelectionChanged);
    }
    platform::paste_text(host, &pending.rewritten_prompt)
}

async fn run_global(app: &AppHandle, mode: OptimizationMode) -> PsResult<()> {
    let state = app.state::<AppState>();
    let settings = state.settings_snapshot();
    if settings.shortcuts_paused {
        notify(app, "Shortcuts are paused.");
        return Ok(());
    }
    let lease = match state.operations.try_begin() {
        Ok(lease) => lease,
        Err(error) => {
            notify(app, &error.to_string());
            return Ok(());
        }
    };
    tray::set_tooltip(app, "P.S. is optimizing…");

    let captured = tauri::async_runtime::spawn_blocking(|| {
        let host = host();
        let target = host.current_window()?;
        let text = platform::capture_selection(host)?;
        Ok::<_, PsError>((target, text))
    })
    .await
    .map_err(|_| PsError::Shutdown)??;

    if lease.is_cancelled() {
        notify(app, "Optimization cancelled.");
        tones::play(Tone::End);
        return Ok(());
    }

    let (target, original) = captured;
    let model = resolve_model(&settings, None)?;
    let client = OllamaClient::new(&settings.ollama_endpoint)?;
    let analysis = optimize_prompt(
        &client,
        &model,
        &original,
        mode,
        &OptimizeConfig::from(&settings),
        &lease.cancellation(),
    )
    .await?;

    if lease.is_cancelled() || !state.operations.is_current(lease.id()) {
        notify(app, "Optimization cancelled.");
        tones::play(Tone::End);
        return Ok(());
    }

    let mut context = DeliveryContext {
        manual: false,
        auto_replace: settings.auto_replace,
        preview_before_replace: settings.preview_before_replace,
        target_still_valid: None,
        selection_still_matches: None,
    };
    if let Ok(current) = host().current_window() {
        context.target_still_valid = Some(current.same_target(&target));
    }

    let might_replace = analysis.status == ps_core::OptimizationStatus::Rewritten
        && !analysis.suspicious
        && settings.auto_replace
        && !settings.preview_before_replace
        && context.target_still_valid == Some(true);
    if might_replace {
        let original_for_check = original.clone();
        let matches = tauri::async_runtime::spawn_blocking(move || {
            let selected = platform::capture_selection(host())?;
            Ok::<_, PsError>(selection_matches(&original_for_check, &selected))
        })
        .await
        .map_err(|_| PsError::Shutdown)?;
        context.selection_still_matches = Some(matches.unwrap_or(false));
    }

    let delivery = plan_delivery(analysis.status, analysis.suspicious, &context);
    let outcome = to_outcome(lease.id(), &original, &analysis, delivery);
    match delivery {
        DeliveryPlan::Replace => {
            if let Err(error) = platform::paste_text(host(), &analysis.rewritten_prompt) {
                stage_review(
                    app,
                    outcome,
                    Some(pending_from(&target, &original, &analysis, lease.id())),
                    "The rewrite could not be pasted. Copy it from P.S. instead.",
                );
                notify(app, &error.to_string());
                tones::play(Tone::End);
            } else {
                notify(app, "Prompt updated");
                tones::play(Tone::Finish);
            }
        }
        DeliveryPlan::LeaveUnchanged => {
            notify(app, "Already clear");
            tones::play(Tone::End);
        }
        DeliveryPlan::Preview => {
            let note = preview_note(&analysis, &context);
            stage_review(
                app,
                outcome,
                Some(pending_from(&target, &original, &analysis, lease.id())),
                &note,
            );
            notify(app, "Review the rewrite");
            tones::play(Tone::Finish);
        }
        DeliveryPlan::Abort => {
            notify(app, "The optimization was not applied.");
            tones::play(Tone::End);
        }
    }
    Ok(())
}

fn stage_review(
    app: &AppHandle,
    outcome: OptimizeOutcome,
    pending: Option<PendingReplacement>,
    note: &str,
) {
    let state = app.state::<AppState>();
    let pending_id = pending.as_ref().map(|item| item.id);
    *state
        .pending
        .lock()
        .unwrap_or_else(|poison| poison.into_inner()) = pending;
    let attention = Attention {
        outcome,
        pending_id,
        note: note.to_string(),
    };
    *state
        .attention
        .lock()
        .unwrap_or_else(|poison| poison.into_inner()) = Some(attention.clone());
    let _ = app.emit("ps://attention", &attention);
    crate::show_main(app);
}

fn pending_from(
    target: &ps_core::workflow::WindowTarget,
    original: &str,
    analysis: &OptimizationAnalysis,
    id: Uuid,
) -> PendingReplacement {
    PendingReplacement {
        id,
        original_text: original.to_string(),
        rewritten_prompt: analysis.rewritten_prompt.clone(),
        target: target.clone(),
        mode: analysis.mode,
        model: analysis.model.clone(),
    }
}

fn preview_note(analysis: &OptimizationAnalysis, context: &DeliveryContext) -> String {
    if analysis.suspicious {
        return "This rewrite needs a look before it replaces anything.".into();
    }
    if context.target_still_valid == Some(false) {
        return "The original window changed, so nothing was pasted.".into();
    }
    if context.selection_still_matches == Some(false) {
        return "The selection changed, so nothing was pasted.".into();
    }
    if !context.auto_replace || context.preview_before_replace {
        return "Review the rewrite, then accept it to replace the selection.".into();
    }
    "Review this rewrite before replacing the selection.".into()
}

fn to_outcome(
    id: Uuid,
    original: &str,
    analysis: &OptimizationAnalysis,
    delivery: DeliveryPlan,
) -> OptimizeOutcome {
    OptimizeOutcome {
        operation_id: id.to_string(),
        mode: analysis.mode,
        model: analysis.model.clone(),
        prompt_version: analysis.prompt_version.clone(),
        status: analysis.status,
        original_text: original.to_string(),
        rewritten_prompt: analysis.rewritten_prompt.clone(),
        delivery,
        warnings: analysis.preservation.warnings(),
        duration_ms: u64::try_from(analysis.duration_ms).unwrap_or(u64::MAX),
        prompt_tokens: analysis.prompt_tokens,
        completion_tokens: analysis.completion_tokens,
        suspicious: analysis.suspicious,
    }
}

fn resolve_model(settings: &AppSettings, override_model: Option<String>) -> PsResult<String> {
    let model = override_model
        .or_else(|| settings.active_model.clone())
        .ok_or_else(|| PsError::InvalidSettings {
            detail: "Choose a local model before optimizing.".into(),
        })?;
    validate_model_name(&model)?;
    Ok(model)
}

pub fn notify(app: &AppHandle, body: &str) {
    let settings = app.state::<AppState>().settings_snapshot();
    if !settings.notifications {
        return;
    }
    let _ = app.notification().builder().title("P.S.").body(body).show();
}
