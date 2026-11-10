use ps_core::settings::{settings_from_stored, stored_settings_need_migration};
use ps_core::{AppSettings, PsError, PsResult};
use tauri::AppHandle;
use tauri_plugin_store::StoreExt;

const STORE_PATH: &str = "settings.json";
const SETTINGS_KEY: &str = "settings";

pub fn load_settings(app: &AppHandle) -> AppSettings {
    let Ok(store) = app.store(STORE_PATH) else {
        return AppSettings::default();
    };
    let Some(value) = store.get(SETTINGS_KEY) else {
        return AppSettings::default();
    };
    let migrate = stored_settings_need_migration(&value);
    let Ok(settings) = settings_from_stored(value) else {
        return AppSettings::default();
    };
    if migrate {
        let _ = save_settings(app, &settings);
    }
    settings
}

pub fn save_settings(app: &AppHandle, settings: &AppSettings) -> PsResult<()> {
    let store = app
        .store(STORE_PATH)
        .map_err(|_| PsError::InvalidSettings {
            detail: "Settings could not be opened.".into(),
        })?;
    let value = serde_json::to_value(settings).map_err(|_| PsError::InvalidSettings {
        detail: "Settings could not be encoded.".into(),
    })?;
    store.set(SETTINGS_KEY, value);
    store.save().map_err(|_| PsError::InvalidSettings {
        detail: "Settings could not be saved.".into(),
    })?;
    Ok(())
}
