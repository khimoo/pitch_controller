extern crate portmidi as pm;
use pitch_shifter::{ControllerEvent, start_controller};

use pm::MidiMessage;
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use std::sync::mpsc;

static CHANNEL: u8 = 0;

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
            Ok(ControllerEvent::PitchBend(value)) => {
                let pitch_bend = MidiMessage {
                    status: 0xE0 + CHANNEL,
                    data1: (value & 0x7F) as u8, // 上7bit
                    data2: ((value >> 7) & 0x7F) as u8, // 下7bit
                    data3: 0,
                };
                println!("Pitch Bend: {:?}", pitch_bend);
                let _ = out_port.write_message(pitch_bend);
            },
            Err(_) => break,
        }
    }
}
