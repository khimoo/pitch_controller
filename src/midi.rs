use crate::events::ControllerEvent;
use portmidi as pm;
use std::sync::mpsc::{self, TryRecvError};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

const NOTE: u8 = 60; // Middle C
const VELOCITY: u8 = 100;

pub fn start_midi_worker(
    context: Arc<pm::PortMidi>,
    input_device_id: Option<pm::PortMidiDeviceId>,
    output_device_id: pm::PortMidiDeviceId,
    rx: mpsc::Receiver<ControllerEvent>,
    channel: u8,
) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        let out_port = context
            .output_port(context.device(output_device_id).unwrap(), 1024)
            .unwrap();

        let mut in_port = input_device_id
            .and_then(|id| context.device(id).ok())
            .and_then(|dev| context.input_port(dev, 1024).ok());

        println!(
            "Playing... Connect Virt Out 1 to Virt In 1 to see midi messages on screen..."
        );
        println!("Press Ctrl-C to abort...");

        handle_controller_and_passthrough(out_port, in_port.take(), rx, channel);
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

fn handle_controller_and_passthrough(
    mut out_port: pm::OutputPort,
    mut in_port: Option<pm::InputPort>,
    rx: mpsc::Receiver<ControllerEvent>,
    channel: u8,
) {
    const IDLE_SLEEP: Duration = Duration::from_millis(2);

    loop {
        let mut idle = true;

        if let Some(port) = in_port.as_mut() {
            if let Ok(_) = port.poll() {
                if let Ok(Some(events)) = port.read_n(1024) {
                    idle = false;
                    for event in events {
                        println!("MIDI In: {:?}", event.message);
                        let _ = out_port.write_message(event.message);
                    }
                }
            }
        }

        match rx.try_recv() {
            Ok(event) => {
                idle = false;
                handle_controller_event(&mut out_port, event, channel);
            }
            Err(TryRecvError::Empty) => {}
            Err(TryRecvError::Disconnected) => break,
        }

        if idle {
            thread::sleep(IDLE_SLEEP);
        }
    }
}

fn handle_controller_event(out_port: &mut pm::OutputPort, event: ControllerEvent, channel: u8) {
    match event {
        ControllerEvent::ButtonDown => {
            let note_on = pm::MidiMessage {
                status: 0x90 + channel,
                data1: NOTE,
                data2: VELOCITY,
                data3: 0,
            };
            println!("Note On: {:?}", note_on);
            let _ = out_port.write_message(note_on);
        }
        ControllerEvent::ButtonUp => {
            let note_off = pm::MidiMessage {
                status: 0x80 + channel,
                data1: NOTE,
                data2: VELOCITY,
                data3: 0,
            };
            println!("Note Off: {:?}", note_off);
            let _ = out_port.write_message(note_off);
        }
        ControllerEvent::PitchBend(value) => {
            let pitch_bend = pm::MidiMessage {
                status: 0xE0 + channel,
                data1: (value & 0x7F) as u8,
                data2: ((value >> 7) & 0x7F) as u8,
                data3: 0,
            };
            println!("Pitch Bend: {:?}", pitch_bend);
            let _ = out_port.write_message(pitch_bend);
        }
        ControllerEvent::RawButton { .. }
        | ControllerEvent::RawAxis { .. }
        | ControllerEvent::ControllerInfo { .. } => {
            // MIDI worker ignores raw/UI-only events
        }
    }
}
