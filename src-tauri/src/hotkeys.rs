use crate::state::AppState;
use ps_core::settings::AppSettings;
use ps_core::shortcuts::ShortcutBinding;
use ps_core::{PsError, PsResult};
use tauri::AppHandle;
use tauri::Manager;
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutEvent, ShortcutState};

pub fn register(app: &AppHandle, settings: &AppSettings) -> PsResult<()> {
    let shortcuts = app.global_shortcut();
    shortcuts
        .unregister_all()
        .map_err(|_| PsError::ShortcutConflict {
            shortcut: settings.improve_shortcut.accelerator.clone(),
            detail: "Existing shortcuts could not be cleared.".into(),
        })?;

    let state = app.state::<AppState>();
    *state
        .registered_improve
        .lock()
        .unwrap_or_else(|poison| poison.into_inner()) = None;
    *state
        .registered_compact
        .lock()
        .unwrap_or_else(|poison| poison.into_inner()) = None;

    if settings.shortcuts_paused {
        return Ok(());
    }

    let improve = register_one(app, &settings.improve_shortcut)?;
    let compact = register_one(app, &settings.compact_shortcut)?;
    *state
        .registered_improve
        .lock()
        .unwrap_or_else(|poison| poison.into_inner()) = improve;
    *state
        .registered_compact
        .lock()
        .unwrap_or_else(|poison| poison.into_inner()) = compact;
    Ok(())
}

fn register_one(app: &AppHandle, binding: &ShortcutBinding) -> PsResult<Option<Shortcut>> {
    if !binding.enabled {
        return Ok(None);
    }
    let parsed = binding
        .accelerator
        .parse::<Shortcut>()
        .map_err(|_| PsError::InvalidShortcut {
            detail: format!("{} is not a valid shortcut.", binding.accelerator),
        })?;
    app.global_shortcut()
        .register(parsed)
        .map_err(|_| PsError::ShortcutConflict {
            shortcut: binding.accelerator.clone(),
            detail: "The operating system already uses this shortcut or rejected it.".into(),
        })?;
    Ok(Some(parsed))
}

pub fn on_shortcut(app: &AppHandle, shortcut: &Shortcut, event: ShortcutEvent) {
    if event.state != ShortcutState::Pressed {
        return;
    }
    let Some(state) = app.try_state::<AppState>() else {
        return;
    };
    let settings = state.settings_snapshot();
    if settings.shortcuts_paused {
        return;
    }
    let improve = *state
        .registered_improve
        .lock()
        .unwrap_or_else(|poison| poison.into_inner());
    let compact = *state
        .registered_compact
        .lock()
        .unwrap_or_else(|poison| poison.into_inner());
    let mode = if improve.as_ref() == Some(shortcut) {
        Some(ps_core::OptimizationMode::Improve)
    } else if compact.as_ref() == Some(shortcut) {
        Some(ps_core::OptimizationMode::Compact)
    } else {
        None
    };
    if let Some(mode) = mode {
        crate::global::spawn(app.clone(), mode);
    }
}
