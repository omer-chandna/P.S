use crate::state::AppState;
use crate::{global, show_main};
use ps_core::OptimizationMode;
use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::{MouseButton, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Emitter, Manager};

pub fn install(app: &AppHandle) -> tauri::Result<()> {
    let menu = build_menu(app)?;
    let icon = tauri::image::Image::from_bytes(include_bytes!("../icons/tray.png"))?;
    let builder = TrayIconBuilder::with_id("ps")
        .menu(&menu)
        .tooltip("P.S. Local Prompt Optimizer")
        .icon(icon)
        .on_menu_event(|app, event| handle_menu(app, event.id.as_ref()))
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                ..
            } = event
            {
                show_main(tray.app_handle());
            }
        });
    #[cfg(target_os = "macos")]
    let builder = builder.icon_as_template(true);
    builder.build(app)?;
    Ok(())
}

pub fn rebuild(app: &AppHandle) {
    let Ok(menu) = build_menu(app) else {
        return;
    };
    if let Some(tray) = app.tray_by_id("ps") {
        let _ = tray.set_menu(Some(menu));
    }
}

pub fn set_tooltip(app: &AppHandle, text: &str) {
    if let Some(tray) = app.tray_by_id("ps") {
        let _ = tray.set_tooltip(Some(text));
    }
}

fn build_menu(app: &AppHandle) -> tauri::Result<Menu<tauri::Wry>> {
    let settings = app.state::<AppState>().settings_snapshot();
    let pause_label = if settings.shortcuts_paused {
        "Resume shortcuts"
    } else {
        "Pause shortcuts"
    };
    let open = MenuItem::with_id(app, "open", "Open P.S.", true, None::<&str>)?;
    let improve = MenuItem::with_id(app, "improve", "Improve selected text", true, None::<&str>)?;
    let compact = MenuItem::with_id(app, "compact", "Compact selected text", true, None::<&str>)?;
    let models = MenuItem::with_id(app, "models", "Change active model", true, None::<&str>)?;
    let pause = MenuItem::with_id(app, "pause", pause_label, true, None::<&str>)?;
    let settings_item = MenuItem::with_id(app, "settings", "Settings", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Quit P.S.", true, None::<&str>)?;
    let separator = PredefinedMenuItem::separator(app)?;
    Menu::with_items(
        app,
        &[
            &open,
            &improve,
            &compact,
            &models,
            &pause,
            &settings_item,
            &separator,
            &quit,
        ],
    )
}

fn handle_menu(app: &AppHandle, id: &str) {
    match id {
        "open" => show_main(app),
        "improve" => global::spawn(app.clone(), OptimizationMode::Improve),
        "compact" => global::spawn(app.clone(), OptimizationMode::Compact),
        "models" => navigate(app, "models"),
        "settings" => navigate(app, "settings"),
        "pause" => toggle_pause(app),
        "quit" => app.exit(0),
        _ => {}
    }
}

fn navigate(app: &AppHandle, page: &str) {
    let _ = app.emit("ps://navigate", page);
    show_main(app);
}

fn toggle_pause(app: &AppHandle) {
    let state = app.state::<AppState>();
    let mut settings = state.settings_snapshot();
    settings.shortcuts_paused = !settings.shortcuts_paused;
    if crate::apply_settings(app, settings).is_ok() {
        rebuild(app);
    }
}
