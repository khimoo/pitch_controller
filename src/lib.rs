extern crate sdl2;

use std::sync::mpsc;

pub enum ControllerEvent {
    ButtonDown,
    ButtonUp,
    PitchBend(u16),
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
                let pitch_bend_value = ((-(value) / 4) + 8192).clamp(0, 16383) as u16; // valueはもともとi16
                tx.send(ControllerEvent::PitchBend(pitch_bend_value)).expect("Failed to send pitch bend event");
            }
            Event::Quit { .. } => break,
            _ => (),
        }
    }

    Ok(())
}
