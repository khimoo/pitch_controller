extern crate sdl2;

use crate::events::ControllerEvent;
use std::sync::mpsc;

fn normalize_axis(raw: i16) -> f32 {
    // Promote to i32 and clamp to symmetric range to avoid -32768 overflow/asymmetry
    let clamped = (raw as i32).clamp(-32767, 32767) as f32;
    (clamped / 32767.0).clamp(-1.0, 1.0) // now in [-1.0, 1.0]
}

fn pitch_bend_from_norm(norm: f32) -> u16 {
    // Map [-1.0, 1.0] to [0, 16383] with center 8192
    let v = ((norm + 1.0) * 8191.5).round();
    v.clamp(0.0, 16383.0) as u16
}

pub fn start_controller(tx: mpsc::Sender<ControllerEvent>) -> Result<(), String> {
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
    println!("Press A button to play MIDI note");
    println!("Use left stick up/down to control pitch bend");

    // Main event loop
    for event in sdl_context.event_pump()?.wait_iter() {
        use sdl2::controller::Button;
        use sdl2::controller::Axis;
        use sdl2::event::Event;

        match event {
            Event::ControllerButtonDown { button: Button::A, .. } => {
                println!("A Button pressed - sending MIDI note on");
                tx.send(ControllerEvent::ButtonDown).expect("Failed to send event");
            }
            Event::ControllerButtonUp { button: Button::A, .. } => {
                println!("A Button released - sending MIDI note off");
                tx.send(ControllerEvent::ButtonUp).expect("Failed to send event");
            }
            Event::ControllerAxisMotion { axis: Axis::LeftY, value, .. } => {
                // Normalize to [-1, 1], invert to make up positive
                let norm = -normalize_axis(value);
                let pitch_bend_value = pitch_bend_from_norm(norm);
                tx.send(ControllerEvent::PitchBend(pitch_bend_value)).expect("Failed to send pitch bend event");
            }
            Event::Quit { .. } => break,
            _ => (),
        }
    }

    Ok(())
}
