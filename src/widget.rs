use std::f32::consts::PI;

use egui::{Checkbox, Slider, TextEdit};

use crate::{window::egui_tools::EguiRenderer, AppState};

pub fn widget_show(state: &mut AppState, renderer: &EguiRenderer) {
    egui::Window::new("Camera Control")
        .default_open(false)
        .show(renderer.context(), |ui| {
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
            state.normal_map_changed = ui
                .add(Checkbox::new(
                    &mut state.enable_normal_map,
                    "Enable normal map",
                ))
                .changed();
        });
}
