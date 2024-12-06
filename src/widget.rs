use crate::{window::egui_tools::EguiRenderer, AppState};

pub fn widget_show(state: &mut AppState, renderer: &EguiRenderer) {
    egui::Window::new("winit + egui + wgpu says hello!")
        .default_open(false)
        .show(renderer.context(), |ui| {
            ui.label("Label!");

            if ui.button("Button!").clicked() {
                println!("boom!")
            }

            ui.separator();
            ui.horizontal(|ui| {
                ui.label(format!(
                    "Pixels per point: {}",
                    renderer.context().pixels_per_point()
                ));
                if ui.button("-").clicked() {
                    state.scale_factor = (state.scale_factor - 0.1).max(0.3);
                }
                if ui.button("+").clicked() {
                    state.scale_factor = (state.scale_factor + 0.1).min(3.0);
                }
            });
        });
}
