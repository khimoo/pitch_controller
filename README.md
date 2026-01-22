# Pitch Controller with Game Controller

A MIDI controller application that allows you to control pitch bend using a game controller. Press the A button to play notes and use the left stick to control pitch bend.

[![Pitch Controller Demo](https://img.youtube.com/vi/xuWjXmqUC6k/0.jpg)](https://youtu.be/xuWjXmqUC6k)

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
The binary will be created at `target/release/pitch_controller`

## Usage

1. Connect your game controller to your computer
2. Run the application:
```bash
./target/release/pitch_controller
```

3. Connect the virtual MIDI output to your desired MIDI input:
   - On Linux: Use aconnect or similar MIDI routing software
   - On Windows: Use a virtual MIDI cable software
