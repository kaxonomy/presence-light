use std::process::Command;

#[tauri::command]
pub fn soundboard_platform() -> &'static str {
    std::env::consts::OS
}

#[tauri::command]
pub async fn setup_soundboard_cable() -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(setup_cable)
        .await
        .map_err(|error| format!("Audio cable setup stopped: {error}"))?
}

#[tauri::command]
pub fn open_microphone_settings() -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        return Command::new("control.exe")
            .arg("mmsys.cpl,,1")
            .creation_flags(0x08000000)
            .spawn()
            .map(|_| ())
            .map_err(|error| format!("Cannot open the microphone controls: {error}"));
    }
    #[cfg(not(target_os = "windows"))]
    Err("Windows microphone controls are unavailable on this platform.".into())
}

#[cfg(target_os = "windows")]
fn setup_cable() -> Result<(), String> {
    use std::os::windows::process::CommandExt;
    Command::new("rundll32.exe")
        .args(["url.dll,FileProtocolHandler", "https://vb-audio.com/Cable/"])
        .creation_flags(0x08000000)
        .spawn()
        .map(|_| ())
        .map_err(|error| format!("Cannot open the VB-CABLE installer page: {error}"))
}

#[cfg(target_os = "macos")]
fn setup_cable() -> Result<(), String> {
    let output = Command::new("open")
        .arg("https://existential.audio/blackhole/")
        .output()
        .map_err(|error| format!("Cannot open the BlackHole installer page: {error}"))?;
    if output.status.success() {
        Ok(())
    } else {
        Err(format!(
            "Cannot open the BlackHole installer page: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ))
    }
}

#[cfg(target_os = "linux")]
pub(super) fn pactl(args: &[&str]) -> Result<String, String> {
    let output = Command::new("pactl").args(args).output().map_err(|error| {
        format!("Install your distribution's PulseAudio utilities (pactl), then try again: {error}")
    })?;
    if !output.status.success() {
        return Err(format!(
            "Cannot configure the audio cable. Check that PulseAudio or PipeWire with PulseAudio support is running: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

#[cfg(target_os = "linux")]
fn has_audio_device(list: &str, name: &str) -> bool {
    list.lines()
        .any(|line| line.split_whitespace().nth(1) == Some(name))
}

#[cfg(target_os = "linux")]
fn setup_cable() -> Result<(), String> {
    ensure_linux_cable()
}

#[cfg(target_os = "linux")]
pub(super) fn ensure_linux_cable() -> Result<(), String> {
    static SETUP_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
    let _lock = SETUP_LOCK
        .lock()
        .map_err(|_| "Audio cable setup is unavailable. Restart Presence Light.")?;
    create_linux_cable(pactl)?;
    ensure_linux_audio_devices()
}

// CPAL uses ALSA on Linux. PulseAudio devices need named ALSA pulse-plugin
// entries so selecting a cable does not depend on the system's default output.
#[cfg(target_os = "linux")]
pub(super) fn ensure_linux_audio_devices() -> Result<(), String> {
    use std::{fs, os::unix::fs::DirBuilderExt, path::PathBuf, sync::Mutex};

    static CONFIG: Mutex<Option<(PathBuf, String)>> = Mutex::new(None);
    let mut config = CONFIG
        .lock()
        .map_err(|_| "Audio device configuration is unavailable. Restart Presence Light.")?;
    let sinks = pactl(&["list", "short", "sinks"])?;
    let sources = pactl(&["list", "short", "sources"])?;
    let aliases = linux_alsa_aliases(&sinks, &sources);

    if config.is_none() {
        let base = std::env::var("ALSA_CONFIG_PATH")
            .unwrap_or_else(|_| "/usr/share/alsa/alsa.conf".into());
        if base.contains(['<', '>', '\n', '\r']) || !std::path::Path::new(&base).is_file() {
            return Err("Cannot find the system ALSA configuration. Install ALSA and its PulseAudio plugin, then restart Presence Light.".into());
        }
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|error| error.to_string())?
            .as_nanos();
        let directory = std::env::temp_dir().join(format!(
            "presence-light-alsa-{}-{unique}",
            std::process::id()
        ));
        fs::DirBuilder::new()
            .mode(0o700)
            .create(&directory)
            .map_err(|error| format!("Cannot prepare the audio device configuration: {error}"))?;
        *config = Some((directory.join("alsa.conf"), base));
    }

    let (path, base) = config.as_ref().unwrap();
    let contents =
        format!("<{base}>\n# Presence Light audio devices; only used by this process.\n{aliases}");
    if fs::read_to_string(path).ok().as_deref() != Some(&contents) {
        let next = path.with_extension("next");
        fs::write(&next, contents)
            .and_then(|_| fs::rename(next, path))
            .map_err(|error| format!("Cannot save the audio device configuration: {error}"))?;
    }
    // All CPAL callers initialize this before enumerating or opening devices.
    // The included system configuration still reads the user's own .asoundrc.
    if std::env::var_os("ALSA_CONFIG_PATH").as_deref() != Some(path.as_os_str()) {
        std::env::set_var("ALSA_CONFIG_PATH", path);
    }
    Ok(())
}

#[cfg(target_os = "linux")]
fn linux_alsa_aliases(sinks: &str, sources: &str) -> String {
    use std::fmt::Write;
    let mut config = String::new();
    for (devices, direction, kind) in [(sinks, "Output", "sink"), (sources, "Input", "source")] {
        for name in devices
            .lines()
            .filter_map(|line| line.split_whitespace().nth(1))
        {
            if direction == "Input"
                && (name.ends_with(".monitor") || name == "presence_light_microphone")
            {
                continue;
            }
            let managed_cable = direction == "Output" && name == "presence_light_cable";
            let (id, description) = if managed_cable {
                (name.to_string(), "Presence Light Cable")
            } else {
                let mut id = format!("presence_light_{kind}_");
                for byte in name.bytes() {
                    write!(id, "{byte:02x}").unwrap();
                }
                (id, name)
            };
            let escape = |text: &str| text.replace('\\', "\\\\").replace('"', "\\\"");
            // ! replaces a stale copy of our reserved alias if the base config
            // already includes one; no user file is written or rewritten.
            writeln!(config, "pcm.!{id} {{\n  type pulse\n  device \"{}\"\n  hint {{ show on description \"{}\" ioid \"{direction}\" }}\n}}", escape(name), escape(description)).unwrap();
        }
    }
    config
}

#[cfg(target_os = "linux")]
fn create_linux_cable(
    mut run: impl FnMut(&[&str]) -> Result<String, String>,
) -> Result<(), String> {
    let sinks = run(&["list", "short", "sinks"])?;
    let sources = run(&["list", "short", "sources"])?;
    let created_sink = if has_audio_device(&sinks, "presence_light_cable") {
        None
    } else {
        let id = run(&[
            "load-module",
            "module-null-sink",
            "sink_name=presence_light_cable",
            r#"sink_properties="device.description='Presence Light Cable'""#,
            "rate=48000",
            "channels=2",
        ])?;
        Some(id.parse::<u32>().map_err(|_| {
            "Audio cable setup received an invalid module identifier. Restart your audio server and try again."
        })?)
    };
    if !has_audio_device(&sources, "presence_light_microphone") {
        if let Err(error) = run(&[
            "load-module",
            "module-remap-source",
            "master=presence_light_cable.monitor",
            "source_name=presence_light_microphone",
            r#"source_properties="device.description='Presence Light Microphone'""#,
        ]) {
            if let Some(id) = created_sink {
                if let Err(cleanup) = run(&["unload-module", &id.to_string()]) {
                    return Err(format!(
                        "{error} The incomplete cable could not be removed: {cleanup}"
                    ));
                }
            }
            return Err(error);
        }
    }
    Ok(())
}

#[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
fn setup_cable() -> Result<(), String> {
    Err("Audio cable setup is supported on Windows, macOS, and Linux.".into())
}

#[cfg(all(test, target_os = "linux"))]
mod tests {
    use super::*;

    #[test]
    fn alsa_aliases_keep_explicit_routes_and_exclude_feedback_inputs() {
        let aliases = linux_alsa_aliases(
            "1\tpresence_light_cable\tdriver\n2\tother-output\tdriver",
            "3\tmicrophone\tdriver\n4\tpresence_light_cable.monitor\tdriver\n5\tpresence_light_microphone\tdriver",
        );
        assert!(aliases.contains("pcm.!presence_light_cable {"));
        assert!(aliases.contains("device \"presence_light_cable\""));
        assert!(aliases.contains("description \"Presence Light Cable\""));
        assert!(aliases.contains("device \"other-output\""));
        assert!(aliases.contains("device \"microphone\""));
        assert_eq!(aliases.matches("ioid \"Input\"").count(), 1);
        assert_eq!(aliases.matches("ioid \"Output\"").count(), 2);
        assert!(!aliases.contains(".monitor"));
        assert!(!aliases.contains("device \"presence_light_microphone\""));
    }

    #[test]
    fn setup_is_idempotent_and_rolls_back_only_new_devices() {
        let mut calls = Vec::new();
        create_linux_cable(|args| {
            calls.push(args.join(" "));
            match args {
                ["list", "short", "sinks"] => Ok("1\tpresence_light_cable\tdriver\n".into()),
                ["list", "short", "sources"] => Ok("2\tpresence_light_microphone\tdriver".into()),
                _ => panic!("Existing cable should not change: {args:?}"),
            }
        })
        .unwrap();
        assert_eq!(calls.len(), 2);

        for sink_exists in [false, true] {
            calls.clear();
            let result = create_linux_cable(|args| {
                calls.push(args.join(" "));
                match args {
                    ["list", "short", "sinks"] if sink_exists => {
                        Ok("1\tpresence_light_cable\tdriver".into())
                    }
                    ["list", "short", _] => Ok(String::new()),
                    ["load-module", "module-null-sink", ..] => Ok("42".into()),
                    ["load-module", "module-remap-source", ..] => Err("Source failed".into()),
                    ["unload-module", "42"] => Ok(String::new()),
                    _ => panic!("Unexpected setup command: {args:?}"),
                }
            });
            assert_eq!(result.unwrap_err(), "Source failed");
            assert_eq!(
                calls.iter().any(|call| call == "unload-module 42"),
                !sink_exists
            );
        }
        assert!(!has_audio_device(
            "1\tpresence_light_cable_extra\tdriver",
            "presence_light_cable"
        ));
    }
}
