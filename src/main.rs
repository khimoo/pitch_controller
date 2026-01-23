extern crate portmidi as pm;

use pitch_controller::controller::ControllerConfig;
use pitch_controller::{start_controller, start_midi_worker, spawn_input_logger, ControllerApp, MidiGraph};
use std::sync::mpsc;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

static CHANNEL: u8 = 0;

fn main() -> Result<(), eframe::Error> {
    // initialize the PortMidi context.
    let context = pm::PortMidi::new().unwrap();
    let context = Arc::new(context);
    let timeout = Duration::from_millis(10);

    let midi_graph = Arc::new(MidiGraph::new().unwrap_or_else(|e| {
        eprintln!("Failed to initialize ALSA MIDI graph: {}", e);
        std::process::exit(1);
    }));

    let v_in = context.create_virtual_input("Virt In 1").unwrap();
    let v_out = context.create_virtual_output("Virt Out 1").unwrap();

    // MIDI output thread
    let (midi_tx, midi_rx) = mpsc::channel();
    let _midi_handle = start_midi_worker(Arc::clone(&context), v_out.id(), midi_rx, CHANNEL);

    // MIDI input logger thread (unchanged)
    let _logger_handle = spawn_input_logger(Arc::clone(&context), v_in.id(), timeout);

    // Controller thread (SDL2 loop). It only sends events to the GUI thread; the GUI forwards them to MIDI.
    let (controller_tx, controller_rx) = mpsc::channel();
    let controller_config = ControllerConfig::default();
    thread::spawn(move || {
        if let Err(e) = start_controller(controller_tx, controller_config) {
            eprintln!("Controller thread error: {}", e);
        }
    });

    // Run egui app that visualizes the stick tilt and forwards events to MIDI
    let native_options = eframe::NativeOptions::default();
    let app = move |_: &eframe::CreationContext<'_>| -> Box<dyn eframe::App> {
        Box::new(ControllerApp::new(
            controller_rx,
            midi_tx.clone(),
            Arc::clone(&midi_graph),
        ))
    };
    eframe::run_native("Pitch Controller Monitor", native_options, Box::new(app))
}
