# SQNC

A modular step sequencer built in Rust with audio and MIDI output.

## Requirements

- Rust (install from [rustup.rs](https://rustup.rs/))
- Nightly toolchain: `rustup install nightly && rustup override set nightly`

## Running

```bash
cargo run
```

## Building

```bash
# Development build
cargo build

# Optimized release build
cargo build --release
```

## Usage

1. Click **Play** to start the sequencer
2. Click step buttons to toggle them on/off
3. Adjust **BPM** and **Note** as desired
4. Select a **MIDI Output** port to send MIDI (optional - audio plays by default)

Changes to steps update in real-time while playing.
