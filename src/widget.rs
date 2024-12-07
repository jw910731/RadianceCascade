use std::f32::consts::PI;

use egui::{Slider, TextEdit};

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
            ui.add(Slider::new(&mut state.eye_pos_distance, 0.3..=3.0));
        });
}
