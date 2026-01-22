#[derive(Debug, Clone, Copy)]
pub enum ControllerEvent {
    ButtonDown,
    ButtonUp,
    PitchBend(u16),
}
