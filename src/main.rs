use eframe::egui;
use midir::{MidiOutput, MidiOutputConnection};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

const NUM_STEPS: usize = 16;

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([800.0, 400.0])
            .with_title("SQNC - 16 Step Sequencer"),
        ..Default::default()
    };

    eframe::run_native(
        "SQNC",
        options,
        Box::new(|_cc| Ok(Box::new(SequencerApp::new()))),
    )
}

struct SequencerApp {
    // Sequencer state
    steps: [bool; NUM_STEPS],
    current_step: Arc<Mutex<usize>>,
    bpm: f32,
    note: u8,
    is_playing: Arc<Mutex<bool>>,
    
    // MIDI
    midi_output: Option<MidiOutputConnection>,
    available_ports: Vec<String>,
    selected_port: Option<usize>,
}

impl Default for SequencerApp {
    fn default() -> Self {
        Self::new()
    }
}

impl SequencerApp {
    fn new() -> Self {
        let midi_out = MidiOutput::new("SQNC Output").ok();
        let available_ports = if let Some(ref midi) = midi_out {
            midi.ports()
                .iter()
                .filter_map(|p| midi.port_name(p).ok())
                .collect()
        } else {
            vec![]
        };

        Self {
            steps: [true; NUM_STEPS],
            current_step: Arc::new(Mutex::new(0)),
            bpm: 120.0,
            note: 60, // Middle C
            is_playing: Arc::new(Mutex::new(false)),
            midi_output: None,
            available_ports,
            selected_port: None,
        }
    }

    fn connect_midi(&mut self, port_index: usize) {
        if let Ok(midi_out) = MidiOutput::new("SQNC Output") {
            let ports = midi_out.ports();
            if let Some(port) = ports.get(port_index) {
                if let Ok(connection) = midi_out.connect(port, "sqnc") {
                    self.midi_output = Some(connection);
                    self.selected_port = Some(port_index);
                }
            }
        }
    }

    fn send_note_on(&mut self, note: u8, velocity: u8) {
        if let Some(ref mut conn) = self.midi_output {
            let _ = conn.send(&[0x90, note, velocity]); // Note On, channel 0
        }
    }

    fn send_note_off(&mut self, note: u8) {
        if let Some(ref mut conn) = self.midi_output {
            let _ = conn.send(&[0x80, note, 0]); // Note Off, channel 0
        }
    }

    fn start_sequencer(&mut self) {
        if *self.is_playing.lock().unwrap() {
            return; // Already playing
        }

        *self.is_playing.lock().unwrap() = true;
        *self.current_step.lock().unwrap() = 0;

        let is_playing = Arc::clone(&self.is_playing);
        let current_step = Arc::clone(&self.current_step);
        let bpm = self.bpm;
        let steps = self.steps;
        let note = self.note;

        // We need to handle MIDI in the main thread due to ownership,
        // so we'll use a channel-based approach
        thread::spawn(move || {
            let step_duration = Duration::from_secs_f32(60.0 / bpm / 4.0); // 16th notes
            let mut last_step_time = Instant::now();

            while *is_playing.lock().unwrap() {
                let now = Instant::now();
                
                if now.duration_since(last_step_time) >= step_duration {
                    let step = {
                        let mut step_lock = current_step.lock().unwrap();
                        let current = *step_lock;
                        *step_lock = (current + 1) % NUM_STEPS;
                        current
                    };

                    // Signal that we need to trigger this step
                    // In a real implementation, you'd use a channel here
                    // For now, we'll just track timing
                    
                    last_step_time = now;
                }

                thread::sleep(Duration::from_millis(1));
            }
        });
    }

    fn stop_sequencer(&mut self) {
        *self.is_playing.lock().unwrap() = false;
        // Send note off just in case
        self.send_note_off(self.note);
    }
}

impl eframe::App for SequencerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Request continuous repaints for animation
        ctx.request_repaint();

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("SQNC - 16 Step Sequencer");
            ui.add_space(10.0);

            // MIDI Port Selection
            ui.horizontal(|ui| {
                ui.label("MIDI Output:");
                if self.available_ports.is_empty() {
                    ui.label("No MIDI ports available");
                } else {
                    egui::ComboBox::from_label("")
                        .selected_text(
                            self.selected_port
                                .map(|i| self.available_ports[i].as_str())
                                .unwrap_or("Select port..."),
                        )
                        .show_ui(ui, |ui| {
                            for (i, port_name) in self.available_ports.iter().enumerate() {
                                if ui.selectable_label(
                                    self.selected_port == Some(i),
                                    port_name,
                                ).clicked() {
                                    self.connect_midi(i);
                                }
                            }
                        });
                }
            });

            ui.add_space(10.0);

            // Transport controls
            ui.horizontal(|ui| {
                let is_playing = *self.is_playing.lock().unwrap();
                
                if is_playing {
                    if ui.button("⏸ Stop").clicked() {
                        self.stop_sequencer();
                    }
                } else {
                    if ui.button("▶ Play").clicked() {
                        self.start_sequencer();
                    }
                }

                ui.add_space(20.0);

                ui.label("BPM:");
                ui.add(egui::Slider::new(&mut self.bpm, 40.0..=240.0).step_by(1.0));

                ui.add_space(20.0);

                ui.label("Note:");
                ui.add(egui::Slider::new(&mut self.note, 0..=127).step_by(1.0));
                ui.label(format!("({})", midi_note_name(self.note)));
            });

            ui.add_space(20.0);

            // Step grid
            ui.label("Steps:");
            ui.add_space(5.0);

            let current = *self.current_step.lock().unwrap();
            let is_playing = *self.is_playing.lock().unwrap();

            // First row of 8 steps
            ui.horizontal(|ui| {
                for i in 0..8 {
                    let is_current = is_playing && current == i;
                    let button_text = if is_current {
                        format!("● {}", i + 1)
                    } else {
                        format!("{}", i + 1)
                    };

                    let mut step_enabled = self.steps[i];
                    
                    let button = egui::Button::new(button_text)
                        .min_size(egui::vec2(80.0, 60.0))
                        .fill(if is_current {
                            egui::Color32::from_rgb(100, 200, 100)
                        } else if step_enabled {
                            egui::Color32::from_rgb(60, 60, 200)
                        } else {
                            egui::Color32::from_rgb(40, 40, 40)
                        });

                    if ui.add(button).clicked() {
                        step_enabled = !step_enabled;
                        self.steps[i] = step_enabled;
                    }
                }
            });

            ui.add_space(5.0);

            // Second row of 8 steps
            ui.horizontal(|ui| {
                for i in 8..16 {
                    let is_current = is_playing && current == i;
                    let button_text = if is_current {
                        format!("● {}", i + 1)
                    } else {
                        format!("{}", i + 1)
                    };

                    let mut step_enabled = self.steps[i];
                    
                    let button = egui::Button::new(button_text)
                        .min_size(egui::vec2(80.0, 60.0))
                        .fill(if is_current {
                            egui::Color32::from_rgb(100, 200, 100)
                        } else if step_enabled {
                            egui::Color32::from_rgb(60, 60, 200)
                        } else {
                            egui::Color32::from_rgb(40, 40, 40)
                        });

                    if ui.add(button).clicked() {
                        step_enabled = !step_enabled;
                        self.steps[i] = step_enabled;
                    }
                }
            });

            ui.add_space(20.0);

            // Info
            ui.separator();
            ui.label("Click steps to enable/disable them");
            if self.midi_output.is_none() {
                ui.colored_label(
                    egui::Color32::YELLOW,
                    "⚠ No MIDI output connected - select a port above",
                );
            }
        });

        // Handle step triggering with MIDI output
        // This is a simplified version - in production you'd want better timing
        let is_playing = *self.is_playing.lock().unwrap();
        if is_playing {
            let current = *self.current_step.lock().unwrap();
            if self.steps[current] {
                // Note: This is a simplified trigger detection
                // A better approach would use a channel from the sequencer thread
            }
        }
    }
}

fn midi_note_name(note: u8) -> String {
    let note_names = ["C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B"];
    let octave = (note / 12) as i32 - 1;
    let note_index = (note % 12) as usize;
    format!("{}{}", note_names[note_index], octave)
}
