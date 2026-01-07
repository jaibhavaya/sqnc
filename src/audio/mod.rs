/// Audio output using cpal
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::{Arc, Mutex};

pub struct AudioOutput {
    _stream: Option<cpal::Stream>,
    phase: Arc<Mutex<f32>>,
    trigger: Arc<Mutex<Option<f32>>>,
}

impl AudioOutput {
    pub fn new() -> Option<Self> {
        let phase = Arc::new(Mutex::new(0.0));
        let trigger = Arc::new(Mutex::new(None));
        
        let phase_clone = Arc::clone(&phase);
        let trigger_clone = Arc::clone(&trigger);
        
        let stream = Self::setup_audio_stream(phase_clone, trigger_clone)?;
        
        Some(Self {
            _stream: Some(stream),
            phase,
            trigger,
        })
    }

    fn setup_audio_stream(
        phase: Arc<Mutex<f32>>,
        trigger: Arc<Mutex<Option<f32>>>,
    ) -> Option<cpal::Stream> {
        let host = cpal::default_host();
        let device = host.default_output_device()?;
        let config = device.default_output_config().ok()?;
        
        let sample_rate = config.sample_rate().0 as f32;
        
        let stream = match config.sample_format() {
            cpal::SampleFormat::F32 => {
                device.build_output_stream(
                    &config.into(),
                    move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                        let mut phase_lock = phase.lock().unwrap();
                        let mut trigger_lock = trigger.lock().unwrap();
                        
                        for sample in data.iter_mut() {
                            if let Some(frequency) = *trigger_lock {
                                let phase_increment = frequency / sample_rate;
                                *sample = (*phase_lock * 2.0 * std::f32::consts::PI).sin() * 0.2;
                                *phase_lock += phase_increment;
                                if *phase_lock >= 1.0 {
                                    *phase_lock -= 1.0;
                                }
                            } else {
                                *sample = 0.0;
                                *phase_lock = 0.0;
                            }
                        }
                    },
                    |err| eprintln!("Audio stream error: {}", err),
                    None,
                )
            }
            _ => return None,
        };
        
        if let Ok(stream) = stream {
            let _ = stream.play();
            Some(stream)
        } else {
            None
        }
    }

    pub fn trigger_note(&mut self, note: u8) {
        let frequency = midi_note_to_frequency(note);
        *self.trigger.lock().unwrap() = Some(frequency);
    }

    pub fn stop_note(&mut self) {
        *self.trigger.lock().unwrap() = None;
    }
}

impl Default for AudioOutput {
    fn default() -> Self {
        Self::new().unwrap_or_else(|| Self {
            _stream: None,
            phase: Arc::new(Mutex::new(0.0)),
            trigger: Arc::new(Mutex::new(None)),
        })
    }
}

fn midi_note_to_frequency(note: u8) -> f32 {
    440.0 * 2.0_f32.powf((note as f32 - 69.0) / 12.0)
}

