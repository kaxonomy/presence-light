use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};
use tauri::AppHandle;
use tauri_plugin_autostart::ManagerExt;

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(default, rename_all = "camelCase")]
struct SavedConfig {
    worker_url: String,
    token: String,
    can_control: bool,
    autostart: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct DesktopConfig {
    worker_url: String,
    token: String,
    can_control: bool,
    autostart: bool,
    configured: bool,
    start_minimized: bool,
}

fn config_path() -> Result<PathBuf, String> {
    std::env::current_exe()
        .map_err(|error| format!("The application cannot find its executable: {error}"))?
        .parent()
        .map(|directory| directory.join("config.yml"))
        .ok_or_else(|| "The application cannot find the configuration directory.".to_string())
}

fn load_config() -> Result<SavedConfig, String> {
    let path = config_path()?;
    if path.exists() {
        let text = fs::read_to_string(&path)
            .map_err(|error| format!("The application cannot read {}: {error}", path.display()))?;
        return serde_yaml::from_str(&text).map_err(|error| {
            format!(
                "The configuration in {} is not valid: {error}",
                path.display()
            )
        });
    }

    Ok(SavedConfig {
        worker_url: std::env::var("PRESENCE_WS_URL").unwrap_or_default(),
        token: std::env::var("PRESENCE_TOKEN").unwrap_or_default(),
        can_control: std::env::var("CAN_CONTROL")
            .is_ok_and(|value| value.eq_ignore_ascii_case("true")),
        autostart: false,
    })
}

fn valid_worker_url(value: &str) -> bool {
    value
        .strip_prefix("ws://")
        .or_else(|| value.strip_prefix("wss://"))
        .is_some_and(|address| !address.trim().is_empty())
}

#[tauri::command]
fn desktop_config(app: AppHandle) -> Result<DesktopConfig, String> {
    let config = load_config()?;
    let configured = valid_worker_url(&config.worker_url) && !config.token.trim().is_empty();
    let autostart = app.autolaunch().is_enabled().unwrap_or(config.autostart);

    Ok(DesktopConfig {
        worker_url: config.worker_url,
        token: config.token,
        can_control: config.can_control,
        autostart,
        configured,
        start_minimized: std::env::args().any(|argument| argument == "--minimized"),
    })
}

#[tauri::command]
fn save_desktop_config(
    app: AppHandle,
    worker_url: String,
    token: String,
    can_control: bool,
    autostart: bool,
) -> Result<(), String> {
    let worker_url = worker_url.trim().to_string();
    let token = token.trim().to_string();
    if !valid_worker_url(&worker_url) {
        return Err("Enter a WebSocket URL that starts with ws:// or wss://.".to_string());
    }
    if token.is_empty() {
        return Err("Enter the private token for this device.".to_string());
    }

    if autostart {
        app.autolaunch().enable()
    } else {
        app.autolaunch().disable()
    }
    .map_err(|error| format!("The application cannot change the startup option: {error}"))?;

    let path = config_path()?;
    let text = serde_yaml::to_string(&SavedConfig {
        worker_url,
        token,
        can_control,
        autostart,
    })
    .map_err(|error| format!("The application cannot create the configuration: {error}"))?;
    fs::write(&path, text)
        .map_err(|error| format!("The application cannot write {}: {error}", path.display()))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_process::init())
        .setup(|app| {
            app.handle().plugin(tauri_plugin_autostart::init(
                tauri_plugin_autostart::MacosLauncher::LaunchAgent,
                Some(vec!["--minimized"]),
            ))?;
            app.handle()
                .plugin(tauri_plugin_global_shortcut::Builder::new().build())?;
            #[cfg(target_os = "macos")]
            app.handle()
                .set_activation_policy(tauri::ActivationPolicy::Accessory)?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            desktop_config,
            save_desktop_config
        ])
        .run(tauri::generate_context!())
        .expect("error while running Presence Light");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn configuration_round_trip() {
        let text = serde_yaml::to_string(&SavedConfig {
            worker_url: "wss://preview-worker.example.workers.dev/ws/friends".into(),
            token: "secret".into(),
            can_control: false,
            autostart: true,
        })
        .unwrap();
        let config: SavedConfig = serde_yaml::from_str(&text).unwrap();

        assert!(valid_worker_url(&config.worker_url));
        assert!(config.autostart);
        assert!(!config.can_control);
    }
}
