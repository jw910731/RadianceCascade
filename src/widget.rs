use std::f32::consts::PI;

use egui::{Checkbox, Slider, TextEdit};

use crate::{window::egui_tools::EguiRenderer, AppState};

pub fn widget_show(state: &mut AppState, renderer: &EguiRenderer) {
    egui::Window::new("Camera Control")
        .default_open(false)
        .show(renderer.context(), |ui| {
            ui.label("Polar angle");
            ui.add(Slider::new(&mut state.eye_rotation_vertical, -1.4..=1.4));
            ui.separator();
            ui.label("Looking Angle");
            ui.add(Slider::new(&mut state.eye_rotation_horizontal, -PI..=PI));
            ui.separator();
            ui.horizontal(|ui| {
                ui.label("Eye position");
                ui.add(Slider::new(&mut state.delta.x, -3.0..=3.0));
                ui.add(Slider::new(&mut state.delta.y, -3.0..=3.0));
                ui.add(Slider::new(&mut state.delta.z, -3.0..=3.0));
            });
            ui.separator();
            ui.horizontal(|ui| {
                ui.label("Light position");
                ui.add_enabled_ui(!state.given_light_position, |ui| {
                    state
                        .light_input
                        .iter_mut()
                        .zip(state.light_position.iter_mut())
                        .for_each(|(input, position)| {
                            if ui.add(TextEdit::singleline(input).char_limit(5)).changed() {
                                *position = input.parse().unwrap_or(*position);
                            }
                        });
                });
            });
            ui.separator();
            ui.label("Distance");
            ui.add(Slider::new(&mut state.eye_pos_distance, 0.01..=3.0));
            ui.separator();
            ui.add(Checkbox::new(
                &mut state.enable_normal_map,
                "Enable normal map",
            ));
        });
}
