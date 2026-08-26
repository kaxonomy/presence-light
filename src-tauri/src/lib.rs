use serde::{Deserialize, Serialize};
use std::{
    fs::{self, File, OpenOptions},
    io::Write,
    path::PathBuf,
    sync::{Mutex, OnceLock},
    time::{SystemTime, UNIX_EPOCH},
};
use tauri::AppHandle;
use tauri_plugin_autostart::ManagerExt;

static DEBUG_LOG: OnceLock<Mutex<File>> = OnceLock::new();

#[derive(Debug, Deserialize, Serialize)]
#[serde(default, rename_all = "camelCase")]
struct SavedConfig {
    worker_url: String,
    token: String,
    can_control: bool,
    autostart: bool,
    animations: bool,
    opacity: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    position_x: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    position_y: Option<i32>,
}

impl Default for SavedConfig {
    fn default() -> Self {
        Self {
            worker_url: String::new(),
            token: String::new(),
            can_control: false,
            autostart: false,
            animations: true,
            opacity: 1.0,
            position_x: None,
            position_y: None,
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct DesktopConfig {
    worker_url: String,
    token: String,
    can_control: bool,
    autostart: bool,
    animations: bool,
    opacity: f64,
    position_x: Option<i32>,
    position_y: Option<i32>,
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
        ..SavedConfig::default()
    })
}

fn write_config(config: &SavedConfig) -> Result<(), String> {
    let path = config_path()?;
    let text = serde_yaml::to_string(config)
        .map_err(|error| format!("The application cannot create the configuration: {error}"))?;
    fs::write(&path, text)
        .map_err(|error| format!("The application cannot write {}: {error}", path.display()))
}

fn write_debug_log(level: &str, message: &str) {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_secs());
    if let Some(log) = DEBUG_LOG.get() {
        if let Ok(mut file) = log.try_lock() {
            let _ = writeln!(file, "{timestamp} [{level}] {message}");
            let _ = file.flush();
        }
    }
}

fn init_debug_log() {
    let Ok(path) = config_path().map(|path| path.with_file_name("latest.txt")) else {
        return;
    };
    let Ok(file) = OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(path)
    else {
        return;
    };
    let _ = DEBUG_LOG.set(Mutex::new(file));
    write_debug_log("INFO", "Presence Light started");

    let previous_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic| {
        write_debug_log("PANIC", &panic.to_string());
        previous_hook(panic);
    }));
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
        animations: config.animations,
        opacity: config.opacity,
        position_x: config.position_x,
        position_y: config.position_y,
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
    animations: bool,
    opacity: f64,
) -> Result<(), String> {
    let worker_url = worker_url.trim().to_string();
    let token = token.trim().to_string();
    if !valid_worker_url(&worker_url) {
        return Err("Enter a WebSocket URL that starts with ws:// or wss://.".to_string());
    }
    if token.is_empty() {
        return Err("Enter the private token for this device.".to_string());
    }
    if !opacity.is_finite() || !(0.1..=1.0).contains(&opacity) {
        return Err("Choose an opacity from 10% to 100%.".to_string());
    }

    let autolaunch = app.autolaunch();
    let current_autostart = autolaunch.is_enabled().unwrap_or(false);
    if autostart != current_autostart {
        if autostart {
            autolaunch.enable()
        } else {
            autolaunch.disable()
        }
        .map_err(|error| format!("The application cannot change the startup option: {error}"))?;
    }

    let previous = load_config()?;
    write_config(&SavedConfig {
        worker_url,
        token,
        can_control,
        autostart,
        animations,
        opacity,
        position_x: previous.position_x,
        position_y: previous.position_y,
    })
}

#[tauri::command]
fn save_overlay_position(position_x: i32, position_y: i32) -> Result<(), String> {
    let mut config = load_config()?;
    config.position_x = Some(position_x);
    config.position_y = Some(position_y);
    write_config(&config)
}

#[tauri::command]
fn debug_log(level: String, message: String) {
    write_debug_log(&level, &message);
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    init_debug_log();
    let app = tauri::Builder::default()
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec!["--minimized"]),
        ))
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .setup(|app| {
            #[cfg(target_os = "macos")]
            app.handle()
                .set_activation_policy(tauri::ActivationPolicy::Accessory)?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            desktop_config,
            save_desktop_config,
            save_overlay_position,
            debug_log
        ])
        .build(tauri::generate_context!())
        .expect("error while building Presence Light");
    app.run(|_, event| {
        if matches!(event, tauri::RunEvent::Exit) {
            write_debug_log("INFO", "Presence Light stopped");
        }
    });
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
            animations: true,
            opacity: 0.75,
            position_x: Some(20),
            position_y: Some(30),
        })
        .unwrap();
        let config: SavedConfig = serde_yaml::from_str(&text).unwrap();

        assert!(valid_worker_url(&config.worker_url));
        assert!(config.autostart);
        assert!(config.animations);
        assert_eq!(config.opacity, 0.75);
        assert_eq!(config.position_x, Some(20));
        assert!(!config.can_control);

        let legacy: SavedConfig =
            serde_yaml::from_str("workerUrl: wss://example.com\ntoken: secret\n").unwrap();
        assert!(legacy.animations);
        assert_eq!(legacy.opacity, 1.0);
    }
}
