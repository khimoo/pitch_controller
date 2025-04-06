extern crate portmidi as pm;
extern crate sdl2;

use pm::MidiMessage;
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use std::sync::mpsc;

static CHANNEL: u8 = 0;
static MELODY: [(u8, u32); 42] = [
    (60, 1),
    (60, 1),
    (67, 1),
    (67, 1),
    (69, 1),
    (69, 1),
    (67, 2),
    (65, 1),
    (65, 1),
    (64, 1),
    (64, 1),
    (62, 1),
    (62, 1),
    (60, 2),
    (67, 1),
    (67, 1),
    (65, 1),
    (65, 1),
    (64, 1),
    (64, 1),
    (62, 2),
    (67, 1),
    (67, 1),
    (65, 1),
    (65, 1),
    (64, 1),
    (64, 1),
    (62, 2),
    (60, 1),
    (60, 1),
    (67, 1),
    (67, 1),
    (69, 1),
    (69, 1),
    (67, 2),
    (65, 1),
    (65, 1),
    (64, 1),
    (64, 1),
    (62, 1),
    (62, 1),
    (60, 2),
];

fn main() {
    // initialize the PortMidi context.
    let context = pm::PortMidi::new().unwrap();
    let context = Arc::new(context);
    let timeout = Duration::from_millis(10);

    let v_in = context.create_virtual_input("Virt In 1").unwrap();
    let v_out = context.create_virtual_output("Virt Out 1").unwrap();

    // Create a channel for sending controller events to the MIDI thread
    let (tx, rx) = mpsc::channel();

    // Start MIDI output thread
    let con2 = Arc::clone(&context);
    thread::spawn(move || {
        let out_port = con2
            .output_port(con2.device(v_out.id()).unwrap(), 1024)
            .unwrap();
        
        println!("Playing... Connect Virt Out 1 to Virt In 1 to see midi messages on screen...");
        println!("(Note: Windows not supported: midi devices do have to be implemented drivers)");
        println!("Press Crtl-C to abort...");
        
        // Handle controller events for MIDI output
        handle_controller_midi(out_port, rx);
    });

    // Start MIDI input thread
    let con3 = Arc::clone(&context);
    thread::spawn(move || {
        let in_port = con3
            .input_port(con3.device(v_in.id()).unwrap(), 1024)
            .unwrap();

        while let Ok(_) = in_port.poll() {
            if let Ok(Some(event)) = in_port.read_n(1024) {
                println!("{:?}", event);
            }
            thread::sleep(timeout);
        }
    });

    // Start controller handling
    start_controller(tx).expect("Failed to initialize controller");
}

fn play(mut out_port: pm::OutputPort, verbose: bool) -> pm::Result<()> {
    for &(note, dur) in MELODY.iter().cycle() {
        let note_on = MidiMessage {
            status: 0x90 + CHANNEL,
            data1: note,
            data2: 100,
            data3: 0,
        };
        if verbose {
            println!("Note On: {:?}", note_on);
        }
        out_port.write_message(note_on)?;

        // note hold time before sending pitch bend
        thread::sleep(Duration::from_millis(dur as u64 * 400));

        // ピッチベンドの値を設定 (0x2000 が中央位置)
        let pitch_bend_value: u16 = 0x3000;
        let lsb = (pitch_bend_value & 0x7F) as u8; // LSB
        let msb = ((pitch_bend_value >> 7) & 0x7F) as u8; // MSB

        let pitch_wheel = MidiMessage {
            status: 0xE0 + CHANNEL,
            data1: lsb,
            data2: msb,
            data3: 0,
        };
        if verbose {
            println!("Pitch Bend: {:?}", pitch_wheel);
        }
        out_port.write_message(pitch_wheel)?;

        thread::sleep(Duration::from_millis(dur as u64 * 200));

        // ピッチベンドの値をもとにもどす
        let pitch_bend_value: u16 = 0x2000;
        let lsb = (pitch_bend_value & 0x7F) as u8; // LSB
        let msb = ((pitch_bend_value >> 7) & 0x7F) as u8; // MSB

        let pitch_wheel = MidiMessage {
            status: 0xE0 + CHANNEL,
            data1: lsb,
            data2: msb,
            data3: 0,
        };
        if verbose {
            println!("Pitch Bend: {:?}", pitch_wheel);
        }
        out_port.write_message(pitch_wheel)?;


        let note_off = MidiMessage {
            status: 0x80 + CHANNEL,
            data1: note,
            data2: 100,
            data3: 0,
        };
        if verbose {
            println!("Note Off: {:?}", note_off);
        }
        out_port.write_message(note_off)?;

        // short pause
        thread::sleep(Duration::from_millis(100));
    }
    Ok(())
}

// New function to handle controller input
fn start_controller(tx: mpsc::Sender<ControllerEvent>) -> Result<(), String> {
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
    println!("Press A button to play MIDI notes");

    // Main event loop
    for event in sdl_context.event_pump()?.wait_iter() {
        use sdl2::controller::Button;
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
            Event::Quit { .. } => break,
            _ => (),
        }
    }

    Ok(())
}

// Controller event enum
enum ControllerEvent {
    ButtonDown,
    ButtonUp,
}

// Function to handle MIDI output based on controller events
fn handle_controller_midi(mut out_port: pm::OutputPort, rx: mpsc::Receiver<ControllerEvent>) {
    const NOTE: u8 = 60; // Middle C
    const VELOCITY: u8 = 100;
    
    loop {
        match rx.recv() {
            Ok(ControllerEvent::ButtonDown) => {
                let note_on = MidiMessage {
                    status: 0x90 + CHANNEL,
                    data1: NOTE,
                    data2: VELOCITY,
                    data3: 0,
                };
                println!("Note On: {:?}", note_on);
                let _ = out_port.write_message(note_on);
            },
            Ok(ControllerEvent::ButtonUp) => {
                let note_off = MidiMessage {
                    status: 0x80 + CHANNEL,
                    data1: NOTE,
                    data2: VELOCITY,
                    data3: 0,
                };
                println!("Note Off: {:?}", note_off);
                let _ = out_port.write_message(note_off);
            },
            Err(_) => break,
        }
    }
}
