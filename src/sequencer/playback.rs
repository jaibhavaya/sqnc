/// Playback engine - coordinates timing and triggers
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{channel, Sender, Receiver};
use std::thread;
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub enum PlaybackEvent {
    StepAdvanced(usize),
    NoteOn(u8, u8),  // note, velocity
    NoteOff(u8),     // note
}

pub struct PlaybackEngine {
    sender: Sender<PlaybackEvent>,
    receiver: Receiver<PlaybackEvent>,
    is_running: Arc<Mutex<bool>>,
}

impl PlaybackEngine {
    pub fn new() -> Self {
        let (sender, receiver) = channel();
        
        Self {
            sender,
            receiver,
            is_running: Arc::new(Mutex::new(false)),
        }
    }

    pub fn start(
        &mut self,
        bpm: f32,
        grid_width: usize,
        grid_height: usize,
        grid_state: Arc<Mutex<Vec<bool>>>,
        note: u8,
    ) {
        if *self.is_running.lock().unwrap() {
            return;
        }

        *self.is_running.lock().unwrap() = true;

        let is_running = Arc::clone(&self.is_running);
        let sender = self.sender.clone();

        thread::spawn(move || {
            let step_duration = Duration::from_secs_f32(60.0 / bpm / 4.0);
            let note_duration = step_duration / 2;
            let total_steps = grid_width * grid_height;
            let mut current_step = 0;
            let mut last_step_time = Instant::now();

            while *is_running.lock().unwrap() {
                let now = Instant::now();

                if now.duration_since(last_step_time) >= step_duration {
                    // Notify that step advanced
                    let _ = sender.send(PlaybackEvent::StepAdvanced(current_step));

                    // Check if step should trigger
                    let should_trigger = {
                        let grid_lock = grid_state.lock().unwrap();
                        current_step < grid_lock.len() && grid_lock[current_step]
                    };

                    if should_trigger {
                        // Send note on
                        let _ = sender.send(PlaybackEvent::NoteOn(note, 100));

                        // Schedule note off
                        let sender_clone = sender.clone();
                        thread::spawn(move || {
                            thread::sleep(note_duration);
                            let _ = sender_clone.send(PlaybackEvent::NoteOff(note));
                        });
                    }

                    current_step = (current_step + 1) % total_steps;
                    last_step_time = now;
                }

                thread::sleep(Duration::from_millis(1));
            }
        });
    }

    pub fn stop(&mut self) {
        *self.is_running.lock().unwrap() = false;
    }

    pub fn is_running(&self) -> bool {
        *self.is_running.lock().unwrap()
    }

    pub fn poll_events(&self) -> Vec<PlaybackEvent> {
        let mut events = Vec::new();
        while let Ok(event) = self.receiver.try_recv() {
            events.push(event);
        }
        events
    }
}

impl Default for PlaybackEngine {
    fn default() -> Self {
        Self::new()
    }
}

