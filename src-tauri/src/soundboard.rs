use rodio::{
    cpal::{
        self,
        traits::{DeviceTrait, HostTrait, StreamTrait},
        FromSample, Sample,
    },
    Decoder, DeviceSinkBuilder, MixerDeviceSink, Player, Source,
};
use serde::Serialize;
use std::{
    io::Cursor,
    sync::{
        atomic::{AtomicBool, AtomicU64, Ordering},
        Arc, LazyLock, Mutex,
    },
    time::Duration,
};

static CHIME: Mutex<Option<Arc<Player>>> = Mutex::new(None);
static PLAYBACK_GENERATION: AtomicU64 = AtomicU64::new(0);
static ROUTE: Mutex<Option<SoundboardRoute>> = Mutex::new(None);
static CONFIGURATION_GENERATION: AtomicU64 = AtomicU64::new(0);
static ROUTING_ENABLED: AtomicBool = AtomicBool::new(false);
static MICROPHONE_MUTED: LazyLock<Arc<AtomicBool>> =
    LazyLock::new(|| Arc::new(AtomicBool::new(false)));
const CHIME_BYTES: &[u8] = include_bytes!("../../public/chime1.mp3");
type StreamError = Arc<Mutex<Option<String>>>;

struct SoundboardRoute {
    input_id: String,
    output_id: String,
    // Keep both streams alive, and stop capture before dropping the mixer.
    _input: Option<cpal::Stream>,
    output: MixerDeviceSink,
    input_error: StreamError,
    output_error: StreamError,
}

#[derive(Serialize)]
pub struct AudioDevice {
    id: String,
    name: String,
}

fn audio_devices(input: bool) -> Result<Vec<AudioDevice>, String> {
    #[cfg(target_os = "linux")]
    super::audio_setup::ensure_linux_audio_devices()?;
    let host = cpal::default_host();
    let devices = if input {
        host.input_devices()
    } else {
        host.output_devices()
    }
    .map_err(|error| format!("Cannot list audio devices: {error}"))?;
    Ok(devices
        .filter_map(|device| {
            Some(AudioDevice {
                id: device.id().ok()?.to_string(),
                name: device.description().ok()?.name().to_string(),
            })
        })
        .collect())
}

#[tauri::command]
pub async fn soundboard_outputs() -> Result<Vec<AudioDevice>, String> {
    tauri::async_runtime::spawn_blocking(|| audio_devices(false))
        .await
        .map_err(|error| error.to_string())?
}

#[tauri::command]
pub async fn soundboard_inputs() -> Result<Vec<AudioDevice>, String> {
    tauri::async_runtime::spawn_blocking(|| audio_devices(true))
        .await
        .map_err(|error| error.to_string())?
}

fn find_device(device_id: &str, input: bool) -> Result<cpal::Device, String> {
    let host = cpal::default_host();
    let mut devices = if input {
        host.input_devices()
    } else {
        host.output_devices()
    }
    .map_err(|error| format!("Cannot list audio devices: {error}"))?;
    devices
        .find(|device| device.id().is_ok_and(|id| id.to_string() == device_id))
        .ok_or_else(|| format!("The selected {} is unavailable. Reconnect it, refresh devices, and choose it again in Audio setup.", if input { "microphone" } else { "cable output" }))
}

fn is_feedback_pair(input: &str, output: &str) -> bool {
    let input = input.to_lowercase();
    let output = output.to_lowercase();
    [
        "blackhole",
        "vb-audio virtual cable",
        "voicemeeter",
        "presence light",
        "presence_light",
    ]
    .iter()
    .any(|cable| input.contains(cable) && output.contains(cable))
        || (input.contains("cable output") && output.contains("cable input"))
}

fn stream_error(error: &StreamError) -> Result<(), String> {
    match error
        .lock()
        .map_err(|_| "The audio error lock is unavailable.")?
        .as_ref()
    {
        Some(error) => Err(error.clone()),
        None => Ok(()),
    }
}

fn error_callback(
    error: StreamError,
    device: &'static str,
) -> impl FnMut(cpal::StreamError) + Send + 'static {
    move |cause| {
        if let Ok(mut error) = error.lock() {
            *error = Some(format!(
                "The {device} stopped: {cause}. Reconnect it and click Refresh devices in the controller configuration."
            ));
        }
    }
}

fn open_output(device: cpal::Device, error: StreamError) -> Result<MixerDeviceSink, String> {
    let mut output = DeviceSinkBuilder::from_device(device)
        .map_err(|error| format!("Cannot configure the cable output: {error}"))?
        .with_error_callback(error_callback(error, "cable output"))
        .open_stream()
        .map_err(|error| format!("Cannot open the cable output: {error}"))?;
    output.log_on_drop(false);
    Ok(output)
}

// Capture must never block the output callback: a missing or muted microphone
// contributes silence while the separate chime source continues playing.
struct MicrophoneSource {
    samples: rtrb::Consumer<f32>,
    sample_rate: rodio::SampleRate,
    muted: Arc<AtomicBool>,
}

impl Iterator for MicrophoneSource {
    type Item = f32;

    fn next(&mut self) -> Option<f32> {
        let sample = self.samples.pop().unwrap_or(0.0);
        Some(if self.muted.load(Ordering::Relaxed) {
            0.0
        } else {
            sample
        })
    }
}

impl Source for MicrophoneSource {
    fn current_span_len(&self) -> Option<usize> {
        None
    }
    fn channels(&self) -> rodio::ChannelCount {
        rodio::ChannelCount::MIN
    }
    fn sample_rate(&self) -> rodio::SampleRate {
        self.sample_rate
    }
    fn total_duration(&self) -> Option<Duration> {
        None
    }
}

fn capture<T>(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    mut samples: rtrb::Producer<f32>,
    error: StreamError,
) -> Result<cpal::Stream, String>
where
    T: cpal::SizedSample,
    f32: FromSample<T>,
{
    let channels = usize::from(config.channels);
    device.build_input_stream::<T, _, _>(config, move |data, _| {
        for frame in data.chunks_exact(channels) {
            let mono = if MICROPHONE_MUTED.load(Ordering::Relaxed) {
                0.0
            } else {
                frame.iter().map(|sample| f32::from_sample(*sample)).sum::<f32>() / channels as f32
            };
            // Bound latency instead of allowing stale speech to accumulate.
            let _ = samples.push(mono);
        }
    }, error_callback(error, "microphone"), None)
        .map_err(|error| format!("Cannot open the microphone: {error}. Check microphone access in system privacy settings."))
}

fn open_input(
    device: &cpal::Device,
    error: StreamError,
) -> Result<(cpal::Stream, MicrophoneSource), String> {
    let supported = device
        .default_input_config()
        .map_err(|error| format!("Cannot configure the microphone: {error}"))?;
    let config: cpal::StreamConfig = supported.clone().into();
    let sample_rate = rodio::SampleRate::new(config.sample_rate)
        .ok_or("The microphone reported an invalid sample rate.")?;
    if config.channels == 0 {
        return Err("The microphone reported no input channels.".into());
    }
    let (mut tx, rx) = rtrb::RingBuffer::new((config.sample_rate / 10).max(1) as usize);
    // A small initial cushion absorbs independent input/output callback timing.
    for _ in 0..config.sample_rate / 50 {
        let _ = tx.push(0.0);
    }
    macro_rules! capture_formats {
        ($($format:ident => $sample:ty),+ $(,)?) => {
            match supported.sample_format() {
                $(cpal::SampleFormat::$format => capture::<$sample>(device, &config, tx, error),)+
                format => Err(format!("The microphone sample format {format} is unsupported.")),
            }
        };
    }
    let stream = capture_formats!(
        F32 => f32, F64 => f64, I8 => i8, I16 => i16, I24 => cpal::I24,
        I32 => i32, I64 => i64, U8 => u8, U16 => u16, U24 => cpal::U24,
        U32 => u32, U64 => u64,
    )?;
    stream
        .play()
        .map_err(|error| format!("Cannot start the microphone: {error}"))?;
    Ok((
        stream,
        MicrophoneSource {
            samples: rx,
            sample_rate,
            muted: Arc::clone(&MICROPHONE_MUTED),
        },
    ))
}

#[tauri::command]
pub async fn configure_soundboard(
    input_device_id: String,
    output_device_id: String,
) -> Result<(), String> {
    let generation = CONFIGURATION_GENERATION.fetch_add(1, Ordering::Relaxed) + 1;
    tauri::async_runtime::spawn_blocking(move || {
        // Same lock order as set_microphone_muted; changing routes cannot leave
        // the virtual microphone muted by an earlier system-level mute.
        let mut restore = super::MICROPHONE_RESTORE
            .lock()
            .map_err(|_| "The microphone mute lock is unavailable.")?;
        let mut active = ROUTE
            .lock()
            .map_err(|_| "The audio routing lock is unavailable.")?;
        // A newer command may have disabled capture while this worker waited.
        if CONFIGURATION_GENERATION.load(Ordering::Relaxed) != generation {
            return Ok(());
        }
        ROUTING_ENABLED.store(!output_device_id.is_empty(), Ordering::Relaxed);
        if output_device_id.is_empty() {
            PLAYBACK_GENERATION.fetch_add(1, Ordering::Relaxed);
            *active = None;
            return Ok(());
        }
        super::restore_microphone_state(&mut restore)?;
        if let Some(route) = active.as_ref() {
            if route.input_id == input_device_id
                && route.output_id == output_device_id
                && stream_error(&route.input_error).is_ok()
                && stream_error(&route.output_error).is_ok()
            {
                return Ok(());
            }
        }
        // Close old capture before switching or reporting a disconnected device.
        PLAYBACK_GENERATION.fetch_add(1, Ordering::Relaxed);
        *active = None;
        if !input_device_id.is_empty() && input_device_id == output_device_id {
            return Err("Choose your physical microphone as input, not the virtual cable.".into());
        }
        #[cfg(target_os = "linux")]
        prepare_linux_output(&output_device_id)?;
        let output_device = find_device(&output_device_id, false)?;
        let input_device = if input_device_id.is_empty() {
            None
        } else {
            let input = find_device(&input_device_id, true)?;
            let input_name = input.description().map_err(|error| error.to_string())?;
            let output_name = output_device.description().map_err(|error| error.to_string())?;
            if is_feedback_pair(input_name.name(), output_name.name()) {
                return Err("Choose your physical microphone as input. Using the virtual cable as both input and output would create feedback.".into());
            }
            Some(input)
        };
        let input_error = Arc::new(Mutex::new(None));
        let output_error = Arc::new(Mutex::new(None));
        let output = open_output(output_device, Arc::clone(&output_error))?;
        let input = if let Some(input_device) = input_device {
            let (input, source) = open_input(&input_device, Arc::clone(&input_error))?;
            if CONFIGURATION_GENERATION.load(Ordering::Relaxed) != generation {
                return Ok(());
            }
            output.mixer().add(source);
            Some(input)
        } else {
            None // Chime-only mode still isolates the cable from system muting.
        };
        if CONFIGURATION_GENERATION.load(Ordering::Relaxed) != generation {
            return Ok(());
        }
        *active = Some(SoundboardRoute {
            input_id: input_device_id,
            output_id: output_device_id,
            _input: input,
            output,
            input_error,
            output_error,
        });
        Ok(())
    })
    .await
    .map_err(|error| error.to_string())?
}

pub fn mute_microphone_if_routed(muted: bool) -> Result<bool, String> {
    MICROPHONE_MUTED.store(muted, Ordering::Relaxed);
    // An unavailable requested route must not fall back to muting the call's
    // virtual microphone. Only explicit disable restores system mute behavior.
    Ok(ROUTING_ENABLED.load(Ordering::Relaxed))
}

#[cfg(target_os = "linux")]
fn prepare_linux_output(device_id: &str) -> Result<(), String> {
    if device_id.ends_with(":presence_light_cable") {
        super::audio_setup::ensure_linux_cable()
    } else {
        super::audio_setup::ensure_linux_audio_devices()
    }
}

pub fn stop_soundboard() -> Result<(), String> {
    CONFIGURATION_GENERATION.fetch_add(1, Ordering::Relaxed);
    stop_soundboard_chime()?;
    ROUTING_ENABLED.store(false, Ordering::Relaxed);
    *ROUTE
        .lock()
        .map_err(|_| "The audio routing lock is unavailable.")? = None;
    Ok(())
}

fn validate_output(device_id: &str, volume: f32) -> Result<(), String> {
    if device_id.trim().is_empty() {
        return Err("Choose a virtual audio cable output in Audio setup.".into());
    }
    if !volume.is_finite() || !(0.0..=1.0).contains(&volume) {
        return Err("Choose a sound volume from 0% to 100%.".into());
    }
    Ok(())
}

#[tauri::command]
pub async fn play_soundboard_chime(device_id: String, volume: f32) -> Result<(), String> {
    validate_output(&device_id, volume)?;
    let generation = PLAYBACK_GENERATION.fetch_add(1, Ordering::Relaxed) + 1;
    tauri::async_runtime::spawn_blocking(move || {
        let mut active = CHIME.lock().map_err(|_| "The chime lock is unavailable.")?;
        // Stop must also cancel a play command queued on the blocking pool.
        if PLAYBACK_GENERATION.load(Ordering::Relaxed) != generation {
            return Ok(());
        }
        if let Some(previous) = active.take() {
            previous.stop();
        }
        let route = ROUTE
            .lock()
            .map_err(|_| "The audio routing lock is unavailable.")?;
        let mut standalone_output = None;
        let (mixer, error) =
            if let Some(route) = route.as_ref().filter(|route| route.output_id == device_id) {
                // The mic and chime have independent gain; muting only the mic
                // never changes the shared output or the virtual cable endpoint.
                (
                    route.output.mixer().clone(),
                    Arc::clone(&route.output_error),
                )
            } else {
                #[cfg(target_os = "linux")]
                prepare_linux_output(&device_id)?;
                let error = Arc::new(Mutex::new(None));
                // Never fall back to speakers when the chosen cable is missing.
                let output = open_output(find_device(&device_id, false)?, Arc::clone(&error))?;
                let mixer = output.mixer().clone();
                standalone_output = Some(output);
                (mixer, error)
            };
        drop(route);
        let source = Decoder::try_from(Cursor::new(CHIME_BYTES))
            .map_err(|error| format!("Cannot read the chime: {error}"))?;
        if PLAYBACK_GENERATION.load(Ordering::Relaxed) != generation {
            return Ok(());
        }
        let player = Arc::new(Player::connect_new(&mixer));
        player.set_volume(volume);
        player.append(source);
        *active = Some(Arc::clone(&player));
        drop(active);
        // Poll errors as well as completion so unplugging an output cannot
        // leave a test command waiting forever on an abandoned audio stream.
        while !player.empty() {
            if PLAYBACK_GENERATION.load(Ordering::Relaxed) != generation {
                player.stop();
                return Ok(());
            }
            if let Err(error) = stream_error(&error) {
                player.stop();
                return Err(error);
            }
            std::thread::sleep(Duration::from_millis(20));
        }
        drop(standalone_output);
        Ok(())
    })
    .await
    .map_err(|error| error.to_string())?
}

#[tauri::command]
pub fn stop_soundboard_chime() -> Result<(), String> {
    PLAYBACK_GENERATION.fetch_add(1, Ordering::Relaxed);
    if let Some(player) = CHIME
        .lock()
        .map_err(|_| "The chime lock is unavailable.")?
        .take()
    {
        player.stop();
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn requires_an_explicit_output_and_valid_volume() {
        assert!(validate_output("", 0.5).is_err());
        assert!(validate_output("  ", 0.5).is_err());
        for volume in [f32::NAN, f32::INFINITY, -0.1, 1.1] {
            assert!(validate_output("cable", volume).is_err());
        }
        assert!(validate_output("cable", 0.5).is_ok());
    }

    #[test]
    fn rejects_cable_feedback_without_rejecting_a_physical_microphone() {
        assert!(is_feedback_pair(
            "CABLE Output (VB-Audio Virtual Cable)",
            "CABLE Input (VB-Audio Virtual Cable)"
        ));
        assert!(is_feedback_pair("BlackHole 2ch", "BlackHole 2ch"));
        assert!(is_feedback_pair(
            "Presence Light Microphone",
            "Presence Light Cable"
        ));
        assert!(!is_feedback_pair(
            "Headset Microphone",
            "CABLE Input (VB-Audio Virtual Cable)"
        ));
        assert!(!is_feedback_pair("NVIDIA Broadcast", "BlackHole 2ch"));
    }

    #[test]
    fn bundled_chime_decodes_to_audible_samples() {
        let source = Decoder::try_from(Cursor::new(CHIME_BYTES)).unwrap();
        assert!(source.into_iter().any(|sample| sample.abs() > 0.01));
    }

    #[test]
    fn muted_or_disconnected_microphone_keeps_chime_playing_and_drains_speech() {
        let (mut tx, rx) = rtrb::RingBuffer::new(8);
        let muted = Arc::new(AtomicBool::new(true));
        let mut microphone = MicrophoneSource {
            samples: rx,
            sample_rate: rodio::SampleRate::new(48_000).unwrap(),
            muted: Arc::clone(&muted),
        };
        tx.push(0.8).unwrap();
        assert_eq!(microphone.next(), Some(0.0));
        muted.store(false, Ordering::Relaxed);
        assert_eq!(microphone.next(), Some(0.0), "muted speech was consumed");
        tx.push(0.3).unwrap();
        assert_eq!(microphone.next(), Some(0.3));
        drop(tx);
        let chime = rodio::buffer::SamplesBuffer::new(
            rodio::ChannelCount::MIN,
            microphone.sample_rate(),
            vec![0.5, 0.25],
        );
        let mut mixed = microphone.mix(chime);
        assert_eq!(
            mixed.next(),
            Some(0.5),
            "missing mic cannot block the chime"
        );
        assert_eq!(mixed.next(), Some(0.25));
    }
}
