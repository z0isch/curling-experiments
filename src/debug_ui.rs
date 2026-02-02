use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};

use crate::hex_grid::Facing;

#[derive(Resource, Clone, Debug)]
pub struct DebugUIState {
    pub drag_coefficient: f32,
    pub stone_velocity_magnitude: f32,
    pub stone_facing: Facing,
    pub min_sweep_distance: f32,
    pub hex_radius: f32,
    pub stone_radius: f32,
    pub slow_down_factor: f32,
    pub rotation_factor: f32,
}

impl Default for DebugUIState {
    fn default() -> Self {
        Self {
            drag_coefficient: 0.0005,
            stone_velocity_magnitude: 200.0,
            stone_facing: Facing::UpRight,
            min_sweep_distance: 2.0,
            hex_radius: 20.0,
            stone_radius: 10.0,
            slow_down_factor: 100.0,
            rotation_factor: 0.017,
        }
    }
}

pub fn ui(mut contexts: EguiContexts, mut debug_ui_state: ResMut<DebugUIState>) -> Result {
    egui::Window::new("").show(contexts.ctx_mut()?, |ui| {
        ui.add(egui::Label::new("R to restart"));
        ui.add(egui::Label::new("Space to pause/resume"));
        ui.add(egui::Slider::new(&mut debug_ui_state.hex_radius, 10.0..=80.0).text("Hex Radius"));
        ui.add(
            egui::Slider::new(&mut debug_ui_state.stone_radius, 10.0..=30.0).text("Stone Radius"),
        );
        ui.add(
            egui::Slider::new(&mut debug_ui_state.min_sweep_distance, 0.0..=200.0)
                .text("Min Sweep Distance"),
        );
        ui.add(
            egui::Slider::new(&mut debug_ui_state.drag_coefficient, 0.0001..=0.001)
                .text("Drag Coefficient"),
        );
        ui.add(
            egui::Slider::new(&mut debug_ui_state.stone_velocity_magnitude, 0.0..=500.0)
                .text("Stone Velocity Magnitude"),
        );
        ui.add(
            egui::Slider::new(&mut debug_ui_state.slow_down_factor, 1.0..=500.0)
                .text("Slow Down Factor"),
        );
        ui.add(
            egui::Slider::new(&mut debug_ui_state.rotation_factor, 0.001..=0.1)
                .text("Rotation Factor"),
        );
        egui::ComboBox::from_label("Stone Facing")
            .selected_text(format!("{:?}", debug_ui_state.stone_facing))
            .show_ui(ui, |ui| {
                for facing in Facing::iterator() {
                    ui.selectable_value(
                        &mut debug_ui_state.stone_facing,
                        facing.clone(),
                        facing.to_string(),
                    );
                }
            });
    });
    Ok(())
}
