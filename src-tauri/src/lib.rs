use serde::{Deserialize, Serialize};
use std::{
    fs::{self, File, OpenOptions},
    io::Write,
    path::PathBuf,
    sync::{Mutex, OnceLock},
    time::{SystemTime, UNIX_EPOCH},
};
use tauri::{AppHandle, Manager};
use tauri_plugin_autostart::ManagerExt;

static DEBUG_LOG: OnceLock<Mutex<File>> = OnceLock::new();
static CONFIG_LOCK: Mutex<()> = Mutex::new(());

#[derive(Debug, Deserialize, Serialize)]
#[serde(default, rename_all = "camelCase")]
struct SavedConfig {
    worker_url: String,
    token: String,
    can_control: bool,
    autostart: bool,
    animations: bool,
    opacity: f64,
    dot_size: u8,
    sound_enabled: bool,
    sound_volume: f64,
    status_shortcut: String,
    visibility_shortcut: String,
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
            dot_size: 22,
            sound_enabled: true,
            sound_volume: 0.5,
            status_shortcut: "CommandOrControl+Shift+KeyP".into(),
            visibility_shortcut: "CommandOrControl+Shift+KeyO".into(),
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
    dot_size: u8,
    sound_enabled: bool,
    sound_volume: f64,
    status_shortcut: String,
    visibility_shortcut: String,
    position_x: Option<i32>,
    position_y: Option<i32>,
    configured: bool,
    start_minimized: bool,
}

fn legacy_config_path() -> Result<PathBuf, String> {
    std::env::current_exe()
        .map_err(|error| format!("The application cannot find its executable: {error}"))?
        .parent()
        .map(|directory| directory.join("config.yml"))
        .ok_or_else(|| "The application cannot find the configuration directory.".to_string())
}

fn config_path(app: &AppHandle) -> Result<PathBuf, String> {
    app.path()
        .app_config_dir()
        .map(|directory| directory.join("settings.yml"))
        .map_err(|error| {
            format!("The application cannot find its configuration directory: {error}")
        })
}

fn load_config(app: &AppHandle) -> Result<SavedConfig, String> {
    let current_path = config_path(app)?;
    let legacy_path = legacy_config_path()?;
    let path = if current_path.exists() {
        current_path
    } else if legacy_path.exists() {
        legacy_path
    } else {
        current_path
    };
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

fn write_config(app: &AppHandle, config: &SavedConfig) -> Result<(), String> {
    let path = config_path(app)?;
    if let Some(directory) = path.parent() {
        fs::create_dir_all(directory).map_err(|error| {
            format!(
                "The application cannot create {}: {error}",
                directory.display()
            )
        })?;
    }
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

fn init_debug_log(app: &AppHandle) {
    let Ok(path) = config_path(app).map(|path| path.with_file_name("latest.txt")) else {
        return;
    };
    if let Some(directory) = path.parent() {
        let _ = fs::create_dir_all(directory);
    }
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

fn parse_shortcut(value: &str) -> Result<tauri_plugin_global_shortcut::Shortcut, String> {
    value
        .trim()
        .parse()
        .map_err(|error| format!("Choose a valid shortcut: {error}"))
}

#[tauri::command]
fn desktop_config(app: AppHandle) -> Result<DesktopConfig, String> {
    let _lock = CONFIG_LOCK
        .lock()
        .map_err(|_| "The configuration lock is unavailable.".to_string())?;
    let config = load_config(&app)?;
    let configured = valid_worker_url(&config.worker_url) && !config.token.trim().is_empty();
    let autostart = app.autolaunch().is_enabled().unwrap_or(config.autostart);

    Ok(DesktopConfig {
        worker_url: config.worker_url,
        token: config.token,
        can_control: config.can_control,
        autostart,
        animations: config.animations,
        opacity: config.opacity,
        dot_size: config.dot_size,
        sound_enabled: config.sound_enabled,
        sound_volume: config.sound_volume,
        status_shortcut: config.status_shortcut,
        visibility_shortcut: config.visibility_shortcut,
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
    dot_size: u8,
    sound_enabled: bool,
    sound_volume: f64,
    status_shortcut: String,
    visibility_shortcut: String,
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
    if !(14..=40).contains(&dot_size) {
        return Err("Choose a dot size from 14px to 40px.".to_string());
    }
    if !sound_volume.is_finite() || !(0.0..=1.0).contains(&sound_volume) {
        return Err("Choose a sound volume from 0% to 100%.".to_string());
    }
    if status_shortcut.trim().is_empty() || visibility_shortcut.trim().is_empty() {
        return Err("Choose both shortcuts.".to_string());
    }
    let parsed_status_shortcut = parse_shortcut(&status_shortcut)?;
    let parsed_visibility_shortcut = parse_shortcut(&visibility_shortcut)?;
    if can_control && parsed_status_shortcut.id() == parsed_visibility_shortcut.id() {
        return Err("Choose different shortcuts for status and visibility.".to_string());
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

    let _lock = CONFIG_LOCK
        .lock()
        .map_err(|_| "The configuration lock is unavailable.".to_string())?;
    let previous = load_config(&app)?;
    write_config(
        &app,
        &SavedConfig {
            worker_url,
            token,
            can_control,
            autostart,
            animations,
            opacity,
            dot_size,
            sound_enabled,
            sound_volume,
            status_shortcut,
            visibility_shortcut,
            position_x: previous.position_x,
            position_y: previous.position_y,
        },
    )
}

#[tauri::command]
fn save_overlay_position(app: AppHandle, position_x: i32, position_y: i32) -> Result<(), String> {
    let _lock = CONFIG_LOCK
        .lock()
        .map_err(|_| "The configuration lock is unavailable.".to_string())?;
    let mut config = load_config(&app)?;
    config.position_x = Some(position_x);
    config.position_y = Some(position_y);
    write_config(&app, &config)
}

#[tauri::command]
fn debug_log(level: String, message: String) {
    write_debug_log(&level, &message);
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let app = tauri::Builder::default()
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec!["--minimized"]),
        ))
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .setup(|app| {
            init_debug_log(app.handle());
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
            dot_size: 30,
            sound_enabled: true,
            sound_volume: 0.5,
            status_shortcut: "CommandOrControl+Shift+P".into(),
            visibility_shortcut: "CommandOrControl+Shift+O".into(),
            position_x: Some(20),
            position_y: Some(30),
        })
        .unwrap();
        let config: SavedConfig = serde_yaml::from_str(&text).unwrap();

        assert!(valid_worker_url(&config.worker_url));
        assert!(config.autostart);
        assert!(config.animations);
        assert_eq!(config.opacity, 0.75);
        assert_eq!(config.dot_size, 30);
        assert!(config.sound_enabled);
        assert_eq!(config.sound_volume, 0.5);
        assert_eq!(config.status_shortcut, "CommandOrControl+Shift+P");
        assert_eq!(
            parse_shortcut("CommandOrControl+Shift+P").unwrap().id(),
            parse_shortcut("CommandOrControl+Shift+KeyP").unwrap().id()
        );
        assert_eq!(config.position_x, Some(20));
        assert!(!config.can_control);

        let legacy: SavedConfig =
            serde_yaml::from_str("workerUrl: wss://example.com\ntoken: secret\n").unwrap();
        assert!(legacy.animations);
        assert_eq!(legacy.opacity, 1.0);
        assert_eq!(legacy.dot_size, 22);
        assert!(legacy.sound_enabled);
        assert_eq!(legacy.sound_volume, 0.5);
        assert_eq!(legacy.visibility_shortcut, "CommandOrControl+Shift+KeyO");
    }
}
