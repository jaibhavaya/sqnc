/// SQNC - A modular step sequencer library
/// 
/// This library provides the core components for building step sequencers:
/// - Grid-based sequencing with flexible grid sizes
/// - Audio output for testing
/// - MIDI output for production use
/// - Playback engine for timing and coordination

pub mod sequencer;
pub mod audio;
pub mod midi;

// Re-export commonly used types
pub use sequencer::{Grid, Sequencer};
pub use sequencer::playback::{PlaybackEngine, PlaybackEvent};
pub use audio::AudioOutput;
pub use midi::{MidiOutputDevice, midi_note_name};

