use crate::events::ControllerEvent;
use crate::midi_graph::{MidiEndpoint, MidiEndpointId, MidiGraph};
use eframe::egui;
use sdl2::controller::{Axis, Button};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::sync::mpsc;
use std::sync::Arc;
use std::time::{Duration, Instant};

pub struct ControllerApp {
    controller_rx: mpsc::Receiver<ControllerEvent>,
    midi_tx: mpsc::Sender<ControllerEvent>,
    midi_graph: Arc<MidiGraph>,
    last_pitch_bend: u16,
    last_tilt: f32,
    button_states: HashMap<Button, bool>,
    axis_states: HashMap<Axis, i16>,
    controller_name: Option<String>,
    controller_mapping: Option<String>,
    available_buttons: Vec<Button>,
    available_axes: Vec<Axis>,
    last_event_at: Option<Instant>,
    endpoints: Vec<MidiEndpoint>,
    selected_src: Option<usize>,
    selected_dst: Option<usize>,
    status: Option<String>,
}

fn configure_fonts(ctx: &egui::Context) {
    let key = egui::Id::new("fonts_configured");
    let already = ctx.data(|d| d.get_temp::<bool>(key).unwrap_or(false));
    if already {
        return;
    }

    if let Ok(path) = env::var("PITCH_CONTROLLER_FONT") {
        if let Ok(bytes) = fs::read(&path) {
            let mut fonts = egui::FontDefinitions::default();
            fonts
                .font_data
                .insert("ipafont".to_owned(), egui::FontData::from_owned(bytes));
            fonts
                .families
                .entry(egui::FontFamily::Proportional)
                .or_default()
                .insert(0, "ipafont".to_owned());
            fonts
                .families
                .entry(egui::FontFamily::Monospace)
                .or_default()
                .insert(0, "ipafont".to_owned());
            ctx.set_fonts(fonts);
        }
    }

    ctx.data_mut(|d| d.insert_temp(key, true));
}

impl ControllerApp {
    pub fn new(
        controller_rx: mpsc::Receiver<ControllerEvent>,
        midi_tx: mpsc::Sender<ControllerEvent>,
        midi_graph: Arc<MidiGraph>,
    ) -> Self {
        Self {
            controller_rx,
            midi_tx,
            midi_graph,
            last_pitch_bend: 8192,
            last_tilt: 0.0,
            button_states: HashMap::new(),
            axis_states: HashMap::new(),
            controller_name: None,
            controller_mapping: None,
            available_buttons: Vec::new(),
            available_axes: Vec::new(),
            last_event_at: None,
            endpoints: Vec::new(),
            selected_src: None,
            selected_dst: None,
            status: None,
        }
    }

    fn refresh_endpoints(&mut self) {
        match self.midi_graph.list_endpoints() {
            Ok(list) => {
                self.endpoints = list;
                self.selected_src = None;
                self.selected_dst = None;
                self.status = Some("端点一覧を更新しました".to_string());
            }
            Err(e) => {
                self.status = Some(format!("端点の取得に失敗しました: {}", e));
            }
        }
    }

    fn selected_pair(&self) -> Option<(MidiEndpointId, MidiEndpointId)> {
        let src_idx = self.selected_src?;
        let dst_idx = self.selected_dst?;
        let src = self.endpoints.get(src_idx)?;
        let dst = self.endpoints.get(dst_idx)?;
        Some((src.id.clone(), dst.id.clone()))
    }

    fn handle_event(&mut self, event: ControllerEvent) {
        match event {
            ControllerEvent::ButtonDown => {
                let _ = self.midi_tx.send(ControllerEvent::ButtonDown);
            }
            ControllerEvent::ButtonUp => {
                let _ = self.midi_tx.send(ControllerEvent::ButtonUp);
            }
            ControllerEvent::PitchBend(value) => {
                self.last_pitch_bend = value;
                // Convert 0..16383 (center 8192) to -1.0..1.0
                let tilt = (value as f32 - 8192.0) / 8192.0;
                self.last_tilt = tilt.clamp(-1.0, 1.0);
                let _ = self.midi_tx.send(ControllerEvent::PitchBend(value));
            }
            ControllerEvent::RawButton { button, pressed } => {
                self.button_states.insert(button, pressed);
            }
            ControllerEvent::RawAxis { axis, value } => {
                self.axis_states.insert(axis, value);
            }
            ControllerEvent::ControllerInfo {
                name,
                mapping,
                buttons,
                axes,
            } => {
                self.controller_name = Some(name);
                self.controller_mapping = Some(mapping);
                self.available_buttons = buttons;
                self.available_axes = axes;
            }
        }

        self.last_event_at = Some(Instant::now());
    }
}

impl eframe::App for ControllerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        configure_fonts(ctx);
        // Drain any pending controller events
        while let Ok(event) = self.controller_rx.try_recv() {
            self.handle_event(event);
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Pitch Controller Monitor");
            if let Some(name) = &self.controller_name {
                ui.label(format!("Controller: {}", name));
            }
            if let Some(mapping) = &self.controller_mapping {
                ui.label(format!("Mapping: {}", mapping));
            }
            if let Some(last) = self.last_event_at {
                let ago = last.elapsed().as_millis();
                ui.label(format!("Last event: {} ms ago", ago));
            } else {
                ui.label("Waiting for controller input...");
            }

            ui.separator();
            ui.label("Pitch axis (mapped)");
            let progress = (self.last_tilt + 1.0) / 2.0; // map -1..1 to 0..1
            ui.add(egui::ProgressBar::new(progress).text(format!("{:+.2}", self.last_tilt)));
            ui.label(format!("Pitch bend value: {} (0-16383)", self.last_pitch_bend));

            ui.separator();
            ui.heading("Buttons");
            if self.available_buttons.is_empty() {
                ui.label("No buttons detected yet.");
            } else {
                for b in &self.available_buttons {
                    let pressed = self.button_states.get(b).copied().unwrap_or(false);
                    ui.horizontal(|ui| {
                        ui.label(format!("{:?}", b));
                        ui.colored_label(
                            if pressed { egui::Color32::LIGHT_GREEN } else { egui::Color32::GRAY },
                            if pressed { "pressed" } else { "released" },
                        );
                    });
                }
            }

            ui.separator();
            ui.heading("Axes");
            if self.available_axes.is_empty() {
                ui.label("No axes detected yet.");
            } else {
                for axis in &self.available_axes {
                    let raw = self.axis_states.get(axis).copied().unwrap_or(0);
                    let norm = (raw as f32 / 32767.0).clamp(-1.0, 1.0);
                    let progress = (norm + 1.0) / 2.0;
                    ui.horizontal(|ui| {
                        ui.label(format!("{:?}", axis));
                        ui.add(egui::ProgressBar::new(progress).text(format!("{:+.2}", norm)));
                        ui.label(format!("raw: {}", raw));
                    });
                }
            }

            ui.separator();
            ui.heading("MIDI接続 (ALSA sequencer)");
            if ui.button("端点を更新").clicked() {
                self.refresh_endpoints();
            }

            let src_label = self
                .selected_src
                .and_then(|i| self.endpoints.get(i))
                .map(|e| e.name.clone())
                .unwrap_or_else(|| "選択なし".to_string());
            let dst_label = self
                .selected_dst
                .and_then(|i| self.endpoints.get(i))
                .map(|e| e.name.clone())
                .unwrap_or_else(|| "選択なし".to_string());

            ui.horizontal(|ui| {
                egui::ComboBox::from_label("送信元")
                    .selected_text(src_label)
                    .show_ui(ui, |ui| {
                        for (idx, ep) in self.endpoints.iter().enumerate() {
                            if !ep.can_write {
                                continue;
                            }
                            ui.selectable_value(&mut self.selected_src, Some(idx), &ep.name);
                        }
                    });

                egui::ComboBox::from_label("送信先")
                    .selected_text(dst_label)
                    .show_ui(ui, |ui| {
                        for (idx, ep) in self.endpoints.iter().enumerate() {
                            if !ep.can_read {
                                continue;
                            }
                            ui.selectable_value(&mut self.selected_dst, Some(idx), &ep.name);
                        }
                    });
            });

            ui.horizontal(|ui| {
                if ui.button("接続").clicked() {
                    if let Some((src, dst)) = self.selected_pair() {
                        match self.midi_graph.connect(&src, &dst) {
                            Ok(_) => self.status = Some("接続しました".to_string()),
                            Err(e) => self.status = Some(format!("接続に失敗しました: {}", e)),
                        }
                    }
                }
                if ui.button("切断").clicked() {
                    if let Some((src, dst)) = self.selected_pair() {
                        match self.midi_graph.disconnect(&src, &dst) {
                            Ok(_) => self.status = Some("切断しました".to_string()),
                            Err(e) => self.status = Some(format!("切断に失敗しました: {}", e)),
                        }
                    }
                }
            });

            if let Some(status) = &self.status {
                ui.label(status);
            }
        });

        // Request a repaint to keep UI responsive
        ctx.request_repaint_after(Duration::from_millis(16));
    }
}
