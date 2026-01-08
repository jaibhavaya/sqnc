#[cfg(feature = "gui")]
use eframe::egui;

#[cfg(feature = "gui")]
use sqnc::{
    midi_note_name, AudioOutput, MidiOutputDevice, PlaybackEngine, PlaybackEvent, Sequencer,
};

#[cfg(feature = "gui")]
fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([800.0, 800.0])
            .with_title("SQNC - Step Sequencer"),
        ..Default::default()
    };

    eframe::run_native(
        "SQNC",
        options,
        Box::new(|_cc| Ok(Box::new(SequencerApp::new()))),
    )
}

#[cfg(not(feature = "gui"))]
fn main() {
    eprintln!("This binary requires the 'gui' feature to be enabled");
    std::process::exit(1);
}

#[cfg(feature = "gui")]
struct SequencerApp {
    sequencer: Sequencer,
    audio_output: AudioOutput,
    midi_output: MidiOutputDevice,
    playback_engine: PlaybackEngine,

    // UI state
    available_midi_ports: Vec<String>,
    selected_port: Option<usize>,
    current_visual_step: usize,
}

#[cfg(feature = "gui")]
impl SequencerApp {
    fn new() -> Self {
        let available_midi_ports = MidiOutputDevice::available_ports();

        Self {
            sequencer: Sequencer::new(8, 8), // Start with 16x1 for compatibility
            audio_output: AudioOutput::default(),
            midi_output: MidiOutputDevice::new(),
            playback_engine: PlaybackEngine::new(),
            available_midi_ports,
            selected_port: None,
            current_visual_step: 0,
        }
    }

    fn handle_playback_events(&mut self) {
        let events = self.playback_engine.poll_events();

        for event in events {
            match event {
                PlaybackEvent::StepAdvanced(step) => {
                    self.current_visual_step = step;
                    self.sequencer.set_current_position(step);
                }
                PlaybackEvent::NoteOn(note, velocity) => {
                    self.audio_output.trigger_note(note);
                    let _ = self.midi_output.send_note_on(note, velocity);
                }
                PlaybackEvent::NoteOff(note) => {
                    self.audio_output.stop_note();
                    let _ = self.midi_output.send_note_off(note);
                }
            }
        }
    }

    fn start_playback(&mut self) {
        let grid = self.sequencer.grid();
        self.playback_engine.start(
            self.sequencer.bpm(),
            grid.width(),
            grid.height(),
            self.sequencer.grid_state().clone(),
            self.sequencer.note(),
        );
    }

    fn stop_playback(&mut self) {
        self.playback_engine.stop();
        self.audio_output.stop_note();
        let _ = self.midi_output.send_note_off(self.sequencer.note());
    }
}

#[cfg(feature = "gui")]
impl eframe::App for SequencerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.request_repaint();

        self.handle_playback_events();

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("SQNC - Step Sequencer");
            ui.add_space(10.0);

            // MIDI Port Selection
            let mut selected_port_changed = None;
            ui.horizontal(|ui| {
                ui.label("MIDI Output:");
                if self.available_midi_ports.is_empty() {
                    ui.label("No MIDI ports available");
                } else {
                    egui::ComboBox::from_label("")
                        .selected_text(
                            self.selected_port
                                .map(|i| self.available_midi_ports[i].as_str())
                                .unwrap_or("Select port..."),
                        )
                        .show_ui(ui, |ui| {
                            for (i, port_name) in self.available_midi_ports.iter().enumerate() {
                                if ui
                                    .selectable_label(self.selected_port == Some(i), port_name)
                                    .clicked()
                                {
                                    selected_port_changed = Some(i);
                                }
                            }
                        });
                }
            });

            if let Some(port_idx) = selected_port_changed {
                if let Ok(()) = self.midi_output.connect(port_idx) {
                    self.selected_port = Some(port_idx);
                }
            }

            ui.add_space(10.0);

            // Transport controls
            ui.horizontal(|ui| {
                let is_playing = self.playback_engine.is_running();

                if is_playing {
                    if ui.button("⏸ Stop").clicked() {
                        self.stop_playback();
                    }
                } else {
                    if ui.button("▶ Play").clicked() {
                        self.start_playback();
                    }
                }

                ui.add_space(20.0);

                ui.label("BPM:");
                let mut bpm = self.sequencer.bpm();
                if ui
                    .add(egui::Slider::new(&mut bpm, 40.0..=240.0).step_by(1.0))
                    .changed()
                {
                    self.sequencer.set_bpm(bpm);
                }

                ui.add_space(20.0);

                ui.label("Note:");
                let mut note = self.sequencer.note();
                if ui
                    .add(egui::Slider::new(&mut note, 0..=127).step_by(1.0))
                    .changed()
                {
                    self.sequencer.set_note(note);
                }
                ui.label(format!("({})", midi_note_name(note)));
            });

            ui.add_space(20.0);

            // Step grid (16 steps in 2 rows of 8)
            ui.label("Steps:");
            ui.add_space(5.0);

            let is_playing = self.playback_engine.is_running();

            ui.horizontal(|ui| {
                for i in 1..8 {
                    ui.vertical(|ui| {
                        for j in 1..8 {
                            let is_current = is_playing && self.current_visual_step == i;
                            let number = i * j
                            let button_text = if is_current {
                                format!("● {}", number)
                            } else {
                                format!("{}", number)
                            };

                            let step_enabled = self.sequencer.grid_mut().get(i, j);

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
                                self.sequencer.grid_mut().toggle(i, j);
                                self.sequencer.update_grid_state();
                            }
                        }
                    });
                }
            });

            // Info
            ui.separator();
            ui.label("Click steps to enable/disable them");
            if !self.midi_output.is_connected() {
                ui.colored_label(
                    egui::Color32::YELLOW,
                    "⚠ No MIDI output connected - audio playback only",
                );
            }
        });
    }
}
