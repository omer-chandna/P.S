fn main() {
    let attributes =
        tauri_build::Attributes::new().app_manifest(tauri_build::AppManifest::new().commands(&[
            "get_settings",
            "update_settings",
            "ollama_status",
            "list_models",
            "activate_model",
            "delete_model",
            "pull_model",
            "cancel_pull",
            "hardware_recommendations",
            "optimize_text",
            "cancel_optimization",
            "attention",
            "dismiss_attention",
            "accept_attention",
            "shortcut_info",
            "permission_info",
            "open_permission_settings",
            "open_external",
            "app_info",
            "finish_onboarding",
            "copy_text",
        ]));
    if let Err(error) = tauri_build::try_build(attributes) {
        panic!("failed to run tauri build script: {error:#}");
    }
}
