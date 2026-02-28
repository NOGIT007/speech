use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::{Arc, Mutex};

use anyhow::{Context, Result};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::SampleFormat;
use hound::{SampleFormat as HoundSampleFormat, WavSpec, WavWriter};

/// Thread-safe shared state for audio recording.
/// The cpal Stream is not Send, so we keep it behind an Arc and
/// only access it from the audio thread. The Tauri commands interact
/// with the shared atomic/mutex state.
pub struct AudioState {
    /// Current RMS audio level (0.0-1.0), stored as f32 bits
    pub level: Arc<AtomicU32>,
    /// Whether we are currently recording
    pub is_recording: Arc<AtomicBool>,
    /// Path to the current recording file
    pub recording_path: Arc<Mutex<Option<PathBuf>>>,
    /// WAV writer (shared with the audio callback thread)
    pub writer: Arc<Mutex<Option<WavWriter<std::io::BufWriter<std::fs::File>>>>>,
    /// Signal to stop recording (the audio thread checks this)
    pub stop_signal: Arc<AtomicBool>,
}

impl AudioState {
    pub fn new() -> Self {
        Self {
            level: Arc::new(AtomicU32::new(0)),
            is_recording: Arc::new(AtomicBool::new(false)),
            recording_path: Arc::new(Mutex::new(None)),
            writer: Arc::new(Mutex::new(None)),
            stop_signal: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Get the current audio RMS level (0.0 to 1.0).
    pub fn current_level(&self) -> f32 {
        f32::from_bits(self.level.load(Ordering::Relaxed))
    }
}

/// Start recording audio on a background thread.
/// Returns the path to the WAV file being written.
pub fn start_recording(state: &AudioState) -> Result<PathBuf> {
    if state.is_recording.load(Ordering::Relaxed) {
        anyhow::bail!("Already recording");
    }

    let host = cpal::default_host();
    let device = host
        .default_input_device()
        .context("No audio input device available")?;

    let config = device
        .default_input_config()
        .context("Failed to get default input config")?;

    let device_sample_rate = config.sample_rate().0;
    let device_channels = config.channels() as usize;

    tracing::info!(
        "Audio device: {} ({}Hz, {}ch, {:?})",
        device.name().unwrap_or_default(),
        device_sample_rate,
        device_channels,
        config.sample_format()
    );

    // Create temp file for recording
    let temp_dir = std::env::temp_dir();
    let filename = format!("speech_recording_{}.wav", uuid::Uuid::new_v4());
    let path = temp_dir.join(&filename);

    // Set up WAV writer: 16kHz mono 16-bit PCM (matching AudioRecorder.swift:56-63)
    let spec = WavSpec {
        channels: 1,
        sample_rate: 16000,
        bits_per_sample: 16,
        sample_format: HoundSampleFormat::Int,
    };

    let writer = WavWriter::create(&path, spec).context("Failed to create WAV file")?;
    *state.writer.lock().unwrap() = Some(writer);
    *state.recording_path.lock().unwrap() = Some(path.clone());
    state.stop_signal.store(false, Ordering::Relaxed);
    state.is_recording.store(true, Ordering::Relaxed);

    let writer_ref = Arc::clone(&state.writer);
    let level_ref = Arc::clone(&state.level);
    let stop_ref = Arc::clone(&state.stop_signal);
    let is_recording_ref = Arc::clone(&state.is_recording);
    let target_rate = 16000u32;

    // Spawn a dedicated thread for the audio stream (cpal::Stream is !Send on macOS)
    std::thread::spawn(move || {
        let stream_result = match config.sample_format() {
            SampleFormat::F32 => {
                let stream_config = config.into();
                device.build_input_stream(
                    &stream_config,
                    {
                        let writer_ref = Arc::clone(&writer_ref);
                        let level_ref = Arc::clone(&level_ref);
                        move |data: &[f32], _| {
                            process_audio_f32(
                                data,
                                device_sample_rate,
                                device_channels,
                                target_rate,
                                &writer_ref,
                                &level_ref,
                            );
                        }
                    },
                    |err| tracing::error!("Audio stream error: {}", err),
                    None,
                )
            }
            SampleFormat::I16 => {
                let stream_config = config.into();
                device.build_input_stream(
                    &stream_config,
                    {
                        let writer_ref = Arc::clone(&writer_ref);
                        let level_ref = Arc::clone(&level_ref);
                        move |data: &[i16], _| {
                            let float_data: Vec<f32> =
                                data.iter().map(|&s| s as f32 / i16::MAX as f32).collect();
                            process_audio_f32(
                                &float_data,
                                device_sample_rate,
                                device_channels,
                                target_rate,
                                &writer_ref,
                                &level_ref,
                            );
                        }
                    },
                    |err| tracing::error!("Audio stream error: {}", err),
                    None,
                )
            }
            SampleFormat::I32 => {
                let stream_config = config.into();
                device.build_input_stream(
                    &stream_config,
                    {
                        let writer_ref = Arc::clone(&writer_ref);
                        let level_ref = Arc::clone(&level_ref);
                        move |data: &[i32], _| {
                            let float_data: Vec<f32> =
                                data.iter().map(|&s| s as f32 / i32::MAX as f32).collect();
                            process_audio_f32(
                                &float_data,
                                device_sample_rate,
                                device_channels,
                                target_rate,
                                &writer_ref,
                                &level_ref,
                            );
                        }
                    },
                    |err| tracing::error!("Audio stream error: {}", err),
                    None,
                )
            }
            format => {
                tracing::error!("Unsupported sample format: {:?}", format);
                is_recording_ref.store(false, Ordering::Relaxed);
                return;
            }
        };

        match stream_result {
            Ok(stream) => {
                if let Err(e) = stream.play() {
                    tracing::error!("Failed to start audio stream: {}", e);
                    is_recording_ref.store(false, Ordering::Relaxed);
                    return;
                }

                tracing::info!("Audio stream started, waiting for stop signal");

                // Keep the stream alive until stop signal
                while !stop_ref.load(Ordering::Relaxed) {
                    std::thread::sleep(std::time::Duration::from_millis(10));
                }

                // Stream is dropped here, stopping capture
                drop(stream);
                tracing::info!("Audio stream stopped");
            }
            Err(e) => {
                tracing::error!("Failed to build audio stream: {}", e);
                is_recording_ref.store(false, Ordering::Relaxed);
            }
        }
    });

    tracing::info!("Recording started: {}", path.display());
    Ok(path)
}

/// Stop recording and finalize the WAV file.
/// Returns the path to the completed WAV file.
pub fn stop_recording(state: &AudioState) -> Result<PathBuf> {
    if !state.is_recording.load(Ordering::Relaxed) {
        anyhow::bail!("Not recording");
    }

    // Signal the audio thread to stop
    state.stop_signal.store(true, Ordering::Relaxed);
    state.is_recording.store(false, Ordering::Relaxed);

    // Give the audio thread a moment to stop
    std::thread::sleep(std::time::Duration::from_millis(50));

    // Finalize WAV file
    if let Some(writer) = state.writer.lock().unwrap().take() {
        writer.finalize().context("Failed to finalize WAV file")?;
    }

    let path = state
        .recording_path
        .lock()
        .unwrap()
        .take()
        .context("No recording path found")?;

    // Reset level
    state.level.store(0, Ordering::Relaxed);

    tracing::info!("Recording stopped: {}", path.display());
    Ok(path)
}

/// Process audio data: compute RMS, downsample to mono 16kHz, write to WAV.
/// Matches the audio pipeline in AudioRecorder.swift:74-114.
fn process_audio_f32(
    data: &[f32],
    device_rate: u32,
    device_channels: usize,
    target_rate: u32,
    writer: &Arc<Mutex<Option<WavWriter<std::io::BufWriter<std::fs::File>>>>>,
    level: &Arc<AtomicU32>,
) {
    if data.is_empty() {
        return;
    }

    // Mix to mono (take first channel)
    let mono: Vec<f32> = data.iter().step_by(device_channels).copied().collect();

    // Compute RMS level (matching AudioRecorder.swift:81-86)
    let sum: f32 = mono.iter().map(|s| s * s).sum();
    let rms = (sum / mono.len() as f32).sqrt();
    let normalized = (rms * 12.0).min(1.0);
    level.store(normalized.to_bits(), Ordering::Relaxed);

    // Resample to target rate using simple linear interpolation
    let resampled = if device_rate != target_rate {
        let ratio = device_rate as f64 / target_rate as f64;
        let output_len = (mono.len() as f64 / ratio) as usize;
        let mut output = Vec::with_capacity(output_len);
        for i in 0..output_len {
            let src_idx = i as f64 * ratio;
            let idx = src_idx as usize;
            let frac = src_idx - idx as f64;
            let sample = if idx + 1 < mono.len() {
                mono[idx] as f64 * (1.0 - frac) + mono[idx + 1] as f64 * frac
            } else {
                mono[idx.min(mono.len() - 1)] as f64
            };
            output.push(sample as f32);
        }
        output
    } else {
        mono
    };

    // Write 16-bit PCM samples to WAV
    if let Ok(mut guard) = writer.lock() {
        if let Some(ref mut w) = *guard {
            for sample in &resampled {
                let s16 =
                    (*sample * i16::MAX as f32).clamp(i16::MIN as f32, i16::MAX as f32) as i16;
                let _ = w.write_sample(s16);
            }
        }
    }
}
