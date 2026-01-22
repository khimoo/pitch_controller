use crate::events::ControllerEvent;
use eframe::egui;
use std::sync::mpsc;
use std::time::{Duration, Instant};

pub struct ControllerApp {
    controller_rx: mpsc::Receiver<ControllerEvent>,
    midi_tx: mpsc::Sender<ControllerEvent>,
    last_pitch_bend: u16,
    last_tilt: f32,
    last_button: Option<bool>,
    last_event_at: Option<Instant>,
}

impl ControllerApp {
    pub fn new(
        controller_rx: mpsc::Receiver<ControllerEvent>,
        midi_tx: mpsc::Sender<ControllerEvent>,
    ) -> Self {
        Self {
            controller_rx,
            midi_tx,
            last_pitch_bend: 8192,
            last_tilt: 0.0,
            last_button: None,
            last_event_at: None,
        }
    }

    fn handle_event(&mut self, event: ControllerEvent) {
        match event {
            ControllerEvent::ButtonDown => {
                self.last_button = Some(true);
                let _ = self.midi_tx.send(ControllerEvent::ButtonDown);
            }
            ControllerEvent::ButtonUp => {
                self.last_button = Some(false);
                let _ = self.midi_tx.send(ControllerEvent::ButtonUp);
            }
            ControllerEvent::PitchBend(value) => {
                self.last_pitch_bend = value;
                // Convert 0..16383 (center 8192) to -1.0..1.0
                let tilt = (value as f32 - 8192.0) / 8192.0;
                self.last_tilt = tilt.clamp(-1.0, 1.0);
                let _ = self.midi_tx.send(ControllerEvent::PitchBend(value));
            }
        }

        self.last_event_at = Some(Instant::now());
    }
}

impl eframe::App for ControllerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Drain any pending controller events
        while let Ok(event) = self.controller_rx.try_recv() {
            self.handle_event(event);
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Pitch Controller Monitor");
            if let Some(last) = self.last_event_at {
                let ago = last.elapsed().as_millis();
                ui.label(format!("Last event: {} ms ago", ago));
            } else {
                ui.label("Waiting for controller input...");
            }

            ui.separator();
            ui.label("Left Stick Tilt (Y axis)");
            let progress = (self.last_tilt + 1.0) / 2.0; // map -1..1 to 0..1
            ui.add(egui::ProgressBar::new(progress).text(format!("{:+.2}", self.last_tilt)));
            ui.label(format!("Pitch bend value: {} (0-16383)", self.last_pitch_bend));

            ui.separator();
            match self.last_button {
                Some(true) => ui.label("Button A: pressed"),
                Some(false) => ui.label("Button A: released"),
                None => ui.label("Button A: --"),
            };
        });

        // Request a repaint to keep UI responsive
        ctx.request_repaint_after(Duration::from_millis(16));
    }
}
