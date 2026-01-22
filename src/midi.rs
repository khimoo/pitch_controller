use crate::events::ControllerEvent;
use portmidi as pm;
use std::sync::mpsc;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

const NOTE: u8 = 60; // Middle C
const VELOCITY: u8 = 100;

pub fn start_midi_worker(
    context: Arc<pm::PortMidi>,
    output_device_id: pm::PortMidiDeviceId,
    rx: mpsc::Receiver<ControllerEvent>,
    channel: u8,
) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        let out_port = context
            .output_port(context.device(output_device_id).unwrap(), 1024)
            .unwrap();

        println!(
            "Playing... Connect Virt Out 1 to Virt In 1 to see midi messages on screen..."
        );
        println!("Press Ctrl-C to abort...");

        handle_controller_midi(out_port, rx, channel);
    })
}

pub fn spawn_input_logger(
    context: Arc<pm::PortMidi>,
    input_device_id: pm::PortMidiDeviceId,
    timeout: Duration,
) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        let in_port = context
            .input_port(context.device(input_device_id).unwrap(), 1024)
            .unwrap();

        while let Ok(_) = in_port.poll() {
            if let Ok(Some(event)) = in_port.read_n(1024) {
                println!("{:?}", event);
            }
            thread::sleep(timeout);
        }
    })
}

fn handle_controller_midi(
    mut out_port: pm::OutputPort,
    rx: mpsc::Receiver<ControllerEvent>,
    channel: u8,
) {
    loop {
        match rx.recv() {
            Ok(ControllerEvent::ButtonDown) => {
                let note_on = pm::MidiMessage {
                    status: 0x90 + channel,
                    data1: NOTE,
                    data2: VELOCITY,
                    data3: 0,
                };
                println!("Note On: {:?}", note_on);
                let _ = out_port.write_message(note_on);
            }
            Ok(ControllerEvent::ButtonUp) => {
                let note_off = pm::MidiMessage {
                    status: 0x80 + channel,
                    data1: NOTE,
                    data2: VELOCITY,
                    data3: 0,
                };
                println!("Note Off: {:?}", note_off);
                let _ = out_port.write_message(note_off);
            }
            Ok(ControllerEvent::PitchBend(value)) => {
                let pitch_bend = pm::MidiMessage {
                    status: 0xE0 + channel,
                    data1: (value & 0x7F) as u8,
                    data2: ((value >> 7) & 0x7F) as u8,
                    data3: 0,
                };
                println!("Pitch Bend: {:?}", pitch_bend);
                let _ = out_port.write_message(pitch_bend);
            }
            Err(_) => break,
        }
    }
}
