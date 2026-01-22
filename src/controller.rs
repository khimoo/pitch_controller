extern crate sdl2;

use crate::events::ControllerEvent;
use sdl2::controller::{Axis, Button, GameController};
use sdl2::event::Event;
use std::collections::HashMap;
use std::sync::mpsc;

/// User-configurable mapping for musical actions and axis processing.
#[derive(Debug, Clone)]
pub struct ControllerConfig {
    pub note_button: Button,
    pub pitch_axis: Axis,
    pub invert_pitch: bool,
    pub deadzone: i16,
}

impl Default for ControllerConfig {
    fn default() -> Self {
        Self {
            note_button: Button::A,
            pitch_axis: Axis::LeftY,
            invert_pitch: true, // LeftY is inverted (up = negative) on most controllers
            deadzone: 2_000,    // small default deadzone to mask minor drift
        }
    }
}

fn normalize_axis(raw: i16, invert: bool) -> f32 {
    // Promote to i32 and clamp to symmetric range to avoid -32768 overflow/asymmetry
    let clamped = (raw as i32).clamp(-32767, 32767) as f32;
    let mut norm = (clamped / 32767.0).clamp(-1.0, 1.0); // now in [-1.0, 1.0]
    if invert {
        norm = -norm;
    }
    norm
}

fn apply_deadzone(norm: f32, deadzone: f32) -> f32 {
    if norm.abs() < deadzone {
        0.0
    } else {
        // rescale so that the remaining range maps back to [-1,1]
        let sign = norm.signum();
        let magnitude = (norm.abs() - deadzone) / (1.0 - deadzone);
        sign * magnitude.clamp(0.0, 1.0)
    }
}

fn pitch_bend_from_norm(norm: f32) -> u16 {
    // Map [-1.0, 1.0] to [0, 16383] with center 8192
    let v = ((norm + 1.0) * 8191.5).round();
    v.clamp(0.0, 16383.0) as u16
}

fn known_buttons() -> Vec<Button> {
    vec![
        Button::A,
        Button::B,
        Button::X,
        Button::Y,
        Button::Back,
        Button::Guide,
        Button::Start,
        Button::LeftStick,
        Button::RightStick,
        Button::LeftShoulder,
        Button::RightShoulder,
        Button::DPadUp,
        Button::DPadDown,
        Button::DPadLeft,
        Button::DPadRight,
        Button::Misc1,
        Button::Paddle1,
        Button::Paddle2,
        Button::Paddle3,
        Button::Paddle4,
        Button::Touchpad,
    ]
}

fn known_axes() -> Vec<Axis> {
    vec![
        Axis::LeftX,
        Axis::LeftY,
        Axis::RightX,
        Axis::RightY,
        Axis::TriggerLeft,
        Axis::TriggerRight,
    ]
}

fn collect_present_inputs(_controller: &GameController) -> (Vec<Button>, Vec<Axis>) {
    // SDL's GameController API always exposes the normalized Xbox-like layout.
    // Some entries may be inert for a given device, but enumerating the known set
    // keeps the UI consistent and lets us show values as they arrive.
    (known_buttons(), known_axes())
}

pub fn start_controller(
    tx: mpsc::Sender<ControllerEvent>,
    config: ControllerConfig,
) -> Result<(), String> {
    // Required for certain controllers to work on Windows
    sdl2::hint::set("SDL_JOYSTICK_THREAD", "1");

    let sdl_context = sdl2::init()?;
    let game_controller_subsystem = sdl_context.game_controller()?;

    let available = game_controller_subsystem
        .num_joysticks()
        .map_err(|e| format!("can't enumerate joysticks: {}", e))?;

    println!("{} joysticks available", available);

    // Find and open the first available game controller
    let controller = (0..available)
        .find_map(|id| {
            if !game_controller_subsystem.is_game_controller(id) {
                println!("{} is not a game controller", id);
                return None;
            }

            println!("Attempting to open controller {}", id);

            match game_controller_subsystem.open(id) {
                Ok(c) => {
                    println!("Success: opened \"{}\"", c.name());
                    Some(c)
                }
                Err(e) => {
                    println!("failed: {:?}", e);
                    None
                }
            }
        });

    // If no controller is found, return
    let controller = match controller {
        Some(c) => c,
        None => {
            println!("No controller found, MIDI will still play without controller input");
            return Ok(());
        }
    };

    println!("Controller mapping: {}", controller.mapping());
    println!("Configured note button: {:?}", config.note_button);
    println!("Configured pitch axis: {:?}", config.pitch_axis);

    let (present_buttons, present_axes) = collect_present_inputs(&controller);
    let _ = tx.send(ControllerEvent::ControllerInfo {
        name: controller.name(),
        mapping: controller.mapping(),
        buttons: present_buttons.clone(),
        axes: present_axes.clone(),
    });

    // Track last raw states to avoid spamming identical events to UI
    let mut button_state: HashMap<Button, bool> = HashMap::new();
    let mut axis_state: HashMap<Axis, i16> = HashMap::new();

    // Main event loop
    for event in sdl_context.event_pump()?.wait_iter() {
        match event {
            Event::ControllerButtonDown { button, .. } => {
                button_state.insert(button, true);
                let _ = tx.send(ControllerEvent::RawButton { button, pressed: true });

                if button == config.note_button {
                    println!("Button {:?} pressed - sending MIDI note on", button);
                    let _ = tx.send(ControllerEvent::ButtonDown);
                }
            }
            Event::ControllerButtonUp { button, .. } => {
                button_state.insert(button, false);
                let _ = tx.send(ControllerEvent::RawButton { button, pressed: false });

                if button == config.note_button {
                    println!("Button {:?} released - sending MIDI note off", button);
                    let _ = tx.send(ControllerEvent::ButtonUp);
                }
            }
            Event::ControllerAxisMotion { axis, value, .. } => {
                axis_state.insert(axis, value);
                let _ = tx.send(ControllerEvent::RawAxis { axis, value });

                if axis == config.pitch_axis {
                    let mut norm = normalize_axis(value, config.invert_pitch);
                    norm = apply_deadzone(norm, (config.deadzone as f32) / 32767.0);
                    let pitch_bend_value = pitch_bend_from_norm(norm);
                    let _ = tx.send(ControllerEvent::PitchBend(pitch_bend_value));
                }
            }
            Event::ControllerDeviceAdded { which, .. } => {
                println!("Controller {} added (hotplug not fully handled yet)", which);
            }
            Event::ControllerDeviceRemoved { which, .. } => {
                println!("Controller {} removed", which);
                break;
            }
            Event::Quit { .. } => break,
            _ => (),
        }
    }

    Ok(())
}
