# Pitch Shifter

A MIDI controller application that allows you to control pitch bend using a game controller. Use the left stick to control pitch bend.

## Features

- Gui setting tool
- Keyconfig

## Requirements
- portmidi
- sdl2

## Installation

```bash
cargo build --release
```
The binary will be created at `target/release/pitch_shifter`

## Usage

1. Connect your game controller to your computer
2. Run the application:
```bash
./target/release/pitch_shifter
```

3. Connect the virtual MIDI output to your desired MIDI input:
   - On Linux: Use aconnect or similar MIDI routing software
   - On Windows: Use a virtual MIDI cable software
