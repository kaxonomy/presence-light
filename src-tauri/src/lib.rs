use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct DesktopConfig {
    ws_url: String,
    token: String,
    can_control: bool,
}

#[tauri::command]
fn desktop_config() -> DesktopConfig {
    DesktopConfig {
        ws_url: std::env::var("PRESENCE_WS_URL").unwrap_or_default(),
        token: std::env::var("PRESENCE_TOKEN").unwrap_or_default(),
        can_control: std::env::var("CAN_CONTROL")
            .is_ok_and(|value| value.eq_ignore_ascii_case("true")),
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_process::init())
        .setup(|app| {
            app.handle()
                .plugin(tauri_plugin_global_shortcut::Builder::new().build())?;
            #[cfg(target_os = "macos")]
            app.handle()
                .set_activation_policy(tauri::ActivationPolicy::Accessory)?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![desktop_config])
        .run(tauri::generate_context!())
        .expect("error while running Presence Light");
}
