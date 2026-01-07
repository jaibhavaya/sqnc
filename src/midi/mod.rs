/// MIDI output using midir
use midir::{MidiOutput, MidiOutputConnection};

pub struct MidiOutputDevice {
    connection: Option<MidiOutputConnection>,
}

impl MidiOutputDevice {
    pub fn new() -> Self {
        Self { connection: None }
    }

    pub fn available_ports() -> Vec<String> {
        if let Ok(midi_out) = MidiOutput::new("SQNC MIDI Output") {
            midi_out
                .ports()
                .iter()
                .filter_map(|p| midi_out.port_name(p).ok())
                .collect()
        } else {
            vec![]
        }
    }

    pub fn connect(&mut self, port_index: usize) -> Result<(), String> {
        let midi_out = MidiOutput::new("SQNC MIDI Output")
            .map_err(|e| format!("Failed to create MIDI output: {}", e))?;
        
        let ports = midi_out.ports();
        let port = ports
            .get(port_index)
            .ok_or_else(|| "Invalid port index".to_string())?;
        
        let connection = midi_out
            .connect(port, "sqnc")
            .map_err(|e| format!("Failed to connect: {}", e))?;
        
        self.connection = Some(connection);
        Ok(())
    }

    pub fn is_connected(&self) -> bool {
        self.connection.is_some()
    }

    pub fn send_note_on(&mut self, note: u8, velocity: u8) -> Result<(), String> {
        if let Some(ref mut conn) = self.connection {
            conn.send(&[0x90, note, velocity])
                .map_err(|e| format!("Failed to send note on: {}", e))?;
        }
        Ok(())
    }

    pub fn send_note_off(&mut self, note: u8) -> Result<(), String> {
        if let Some(ref mut conn) = self.connection {
            conn.send(&[0x80, note, 0])
                .map_err(|e| format!("Failed to send note off: {}", e))?;
        }
        Ok(())
    }

    pub fn disconnect(&mut self) {
        self.connection = None;
    }
}

impl Default for MidiOutputDevice {
    fn default() -> Self {
        Self::new()
    }
}

pub fn midi_note_name(note: u8) -> String {
    let note_names = ["C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B"];
    let octave = (note / 12) as i32 - 1;
    let note_index = (note % 12) as usize;
    format!("{}{}", note_names[note_index], octave)
}

