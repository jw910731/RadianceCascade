use std::f32::consts::PI;

use egui::Slider;

use crate::{window::egui_tools::EguiRenderer, AppState};

pub fn widget_show(state: &mut AppState, renderer: &EguiRenderer) {
    egui::Window::new("Camera Control")
        .default_open(false)
        .show(renderer.context(), |ui| {
            ui.label("Polar angle");
            ui.add(Slider::new(&mut state.look_at_y, -1.0..=1.0));
            ui.separator();
            ui.label("Looking Angle");
            ui.add(Slider::new(&mut state.eye_pos_rotation, -PI..=PI));
        });
}
