mod commands;
mod global;
mod hotkeys;
mod persist;
mod platform;
mod state;
mod tones;
mod tray;

use commands::{
    accept_attention, activate_model, app_info, attention, cancel_optimization, cancel_pull,
    copy_text, delete_model, dismiss_attention, finish_onboarding, get_settings,
    hardware_recommendations, list_models, ollama_status, open_external, open_permission_settings,
    optimize_text, permission_info, pull_model, shortcut_info, update_settings,
};
use ps_core::settings::AppSettings;
use ps_core::PsResult;
use state::AppState;
use tauri::Manager;
use tauri_plugin_autostart::ManagerExt;

pub fn apply_settings(app: &tauri::AppHandle, mut incoming: AppSettings) -> PsResult<AppSettings> {
    incoming.validate_and_normalize()?;
    let state = app.state::<AppState>();
    let previous = state.settings_snapshot();
    state.update_settings(incoming.clone());
    if let Err(error) = hotkeys::register(app, &incoming) {
        state.update_settings(previous.clone());
        let _ = hotkeys::register(app, &previous);
        let _ = persist::save_settings(app, &previous);
        tray::rebuild(app);
        return Err(error);
    }
    persist::save_settings(app, &incoming)?;
    apply_autostart(app, incoming.launch_at_startup);
    tray::rebuild(app);
    Ok(incoming)
}

fn apply_autostart(app: &tauri::AppHandle, enabled: bool) {
    let manager = app.autolaunch();
    let result = if enabled {
        manager.enable()
    } else {
        manager.disable()
    };
    if let Err(error) = result {
        eprintln!("launch-at-startup could not be updated: {error}");
    }
}

pub fn show_main(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.unminimize();
        let _ = window.show();
        let _ = window.set_focus();
    }
}

#[doc(hidden)]
pub mod test_support {
    /// Runs a closure on the macOS main queue. Used to prove keyboard-layout
    /// lookups no longer abort when capture starts on a worker thread.
    #[cfg(target_os = "macos")]
    pub fn run_on_main_thread<T: Send + 'static>(work: impl FnOnce() -> T + Send + 'static) -> T {
        crate::platform::run_on_main_thread(work)
    }
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec!["--background"]),
        ))
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            show_main(app);
        }))
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(hotkeys::on_shortcut)
                .build(),
        )
        .setup(|app| {
            let settings = persist::load_settings(app.handle());
            app.manage(AppState::new(settings.clone()));
            #[cfg(target_os = "macos")]
            {
                let _ = app.set_activation_policy(tauri::ActivationPolicy::Accessory);
            }
            if let Err(error) = tray::install(app.handle()) {
                eprintln!("menu bar icon was not created: {error}");
            }
            if let Err(error) = hotkeys::register(app.handle(), &settings) {
                eprintln!("shortcuts were not registered: {error}");
            }
            apply_autostart(app.handle(), settings.launch_at_startup);
            let background = std::env::args().any(|arg| arg == "--background");
            if !background {
                show_main(app.handle());
            }
            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = window.hide();
            }
        })
        .invoke_handler(tauri::generate_handler![
            get_settings,
            update_settings,
            ollama_status,
            list_models,
            activate_model,
            delete_model,
            pull_model,
            cancel_pull,
            hardware_recommendations,
            optimize_text,
            cancel_optimization,
            attention,
            dismiss_attention,
            accept_attention,
            shortcut_info,
            permission_info,
            open_permission_settings,
            open_external,
            app_info,
            finish_onboarding,
            copy_text
        ])
        .build(tauri::generate_context!())
        .expect("failed to start P.S.")
        .run(|app, event| {
            if let tauri::RunEvent::Exit = event {
                if let Some(state) = app.try_state::<AppState>() {
                    state.operations.cancel_current();
                    if let Some(job) = state
                        .pull
                        .lock()
                        .unwrap_or_else(|poison| poison.into_inner())
                        .as_ref()
                    {
                        job.cancel.cancel();
                    }
                }
            }
        });
}
