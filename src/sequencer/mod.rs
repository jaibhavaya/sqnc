/// Core sequencer logic - grid state and step management
/// This is grid-agnostic and can work with any grid size
use std::sync::{Arc, Mutex};
pub mod playback;

#[derive(Debug, Clone)]
pub struct Grid {
    cells: Vec<Vec<bool>>,
    width: usize,
    height: usize,
}

impl Grid {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            cells: vec![vec![true; width]; height],
            width,
            height,
        }
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn get(&self, x: usize, y: usize) -> bool {
        self.cells
            .get(y)
            .and_then(|row| row.get(x))
            .copied()
            .unwrap_or(false)
    }

    pub fn set(&mut self, x: usize, y: usize, value: bool) {
        if let Some(row) = self.cells.get_mut(y) {
            if let Some(cell) = row.get_mut(x) {
                *cell = value;
            }
        }
    }

    pub fn toggle(&mut self, x: usize, y: usize) {
        let current = self.get(x, y);
        self.set(x, y, !current);
    }

    pub fn clear(&mut self) {
        for row in &mut self.cells {
            for cell in row {
                *cell = false;
            }
        }
    }

    pub fn fill(&mut self) {
        for row in &mut self.cells {
            for cell in row {
                *cell = true;
            }
        }
    }
}

pub struct Sequencer {
    grid: Grid,
    grid_state: Arc<Mutex<Vec<Vec<bool>>>>,
    current_position: usize,
    bpm: f32,
    note: u8,
    is_playing: bool,
}

impl Sequencer {
    pub fn new(width: usize, height: usize) -> Self {
        let grid = Grid::new(width, height);
        let initial_state = grid.cells.clone();

        Self {
            grid: Grid::new(width, height),
            grid_state: Arc::new(Mutex::new(initial_state)),
            current_position: 0,
            bpm: 120.0,
            note: 60, // Middle C
            is_playing: false,
        }
    }

    pub fn grid(&self) -> &Grid {
        &self.grid
    }

    pub fn grid_state(&self) -> &Arc<Mutex<Vec<Vec<bool>>>> {
        &self.grid_state
    }

    pub fn grid_mut(&mut self) -> &mut Grid {
        &mut self.grid
    }

    pub fn current_position(&self) -> usize {
        self.current_position
    }

    pub fn set_current_position(&mut self, pos: usize) {
        self.current_position = pos;
    }

    pub fn advance_position(&mut self) -> usize {
        let total_steps = self.grid.width() * self.grid.height();
        self.current_position = (self.current_position + 1) % total_steps;
        self.current_position
    }

    pub fn bpm(&self) -> f32 {
        self.bpm
    }

    pub fn set_bpm(&mut self, bpm: f32) {
        self.bpm = bpm.clamp(40.0, 240.0);
    }

    pub fn note(&self) -> u8 {
        self.note
    }

    pub fn set_note(&mut self, note: u8) {
        self.note = note.clamp(0, 127);
    }

    pub fn is_playing(&self) -> bool {
        self.is_playing
    }

    pub fn start(&mut self) {
        self.is_playing = true;
    }

    pub fn stop(&mut self) {
        self.is_playing = false;
    }

    pub fn toggle_play(&mut self) {
        self.is_playing = !self.is_playing;
    }

    /// Check if the current step should trigger a note
    pub fn should_trigger(&self) -> bool {
        let total_steps = self.grid.width() * self.grid.height();
        if self.current_position >= total_steps {
            return false;
        }

        let x = self.current_position % self.grid.width();
        let y = self.current_position / self.grid.width();
        self.grid.get(x, y)
    }

    /// Calculate step duration in milliseconds
    pub fn step_duration_ms(&self) -> u64 {
        let steps_per_beat = 4.0; // 16th notes
        let beats_per_second = self.bpm / 60.0;
        let steps_per_second = beats_per_second * steps_per_beat;
        (1000.0 / steps_per_second) as u64
    }

    pub fn update_grid_state(&mut self) {
        let mut shared = self.grid_state.lock().unwrap();
        *shared = self.grid.cells.clone();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grid_creation() {
        let grid = Grid::new(8, 8);
        assert_eq!(grid.width(), 8);
        assert_eq!(grid.height(), 8);
    }

    #[test]
    fn test_grid_toggle() {
        let mut grid = Grid::new(4, 4);
        assert!(grid.get(0, 0));
        grid.toggle(0, 0);
        assert!(!grid.get(0, 0));
    }

    #[test]
    fn test_sequencer_advance() {
        let mut seq = Sequencer::new(4, 4);
        assert_eq!(seq.current_position(), 0);
        seq.advance_position();
        assert_eq!(seq.current_position(), 1);
    }
}
