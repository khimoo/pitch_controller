use sdl2::controller::{Axis, Button};

#[derive(Debug, Clone)]
pub enum ControllerEvent {
    // High-level, already-mapped musical intents (consumed by MIDI worker)
    ButtonDown, // mapped note-on button
    ButtonUp,   // mapped note-off button
    PitchBend(u16),

    // Raw, device-level input for UI/learning/configuration
    RawButton { button: Button, pressed: bool },
    RawAxis { axis: Axis, value: i16 },

    // Metadata about a connected controller so UI can populate controls
    ControllerInfo {
        name: String,
        mapping: String,
        buttons: Vec<Button>,
        axes: Vec<Axis>,
    },
}
