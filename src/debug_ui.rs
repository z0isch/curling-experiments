use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};

use crate::{
    PhysicsPaused,
    hex_grid::HexGrid,
    level::{CurrentLevel, Facing},
    restart_game,
    stone::Stone,
    tile::ScratchOffMaterial,
};

#[derive(Clone, Debug)]
pub struct StoneUIConfig {
    pub velocity_magnitude: f32,
    pub facing: Facing,
}

#[derive(Resource, Clone, Debug)]
pub struct DebugUIState {
    pub drag_coefficient: f32,
    pub stone_configs: Vec<StoneUIConfig>,
    pub min_sweep_distance: f32,
    pub hex_radius: f32,
    pub stone_radius: f32,
    pub slow_down_factor: f32,
    pub rotation_factor: f32,
    pub snap_distance: f32,
    pub snap_velocity: f32,
    pub current_level: CurrentLevel,
    pub speed_up_factor: f32,
}

pub fn debug_ui(mut contexts: EguiContexts, mut debug_ui_state: ResMut<DebugUIState>) -> Result {
    egui::Window::new("Debug")
        .default_open(false)
        .show(contexts.ctx_mut()?, |debug_ui| {
            debug_ui.add(egui::Label::new("R to restart"));
            debug_ui.add(egui::Label::new("Space to pause/resume"));

            debug_ui.separator();
            debug_ui.add(egui::Label::new("Level Selection"));
            egui::ComboBox::from_id_salt("level_selector")
                .selected_text(format!("{}", debug_ui_state.current_level))
                .show_ui(debug_ui, |debug_ui| {
                    for level in CurrentLevel::iterator() {
                        debug_ui.selectable_value(
                            &mut debug_ui_state.current_level,
                            *level,
                            level.to_string(),
                        );
                    }
                });
            debug_ui.add(
                egui::Slider::new(&mut debug_ui_state.hex_radius, 10.0..=80.0).text("Hex Radius"),
            );
            debug_ui.add(
                egui::Slider::new(&mut debug_ui_state.stone_radius, 10.0..=30.0)
                    .text("Stone Radius"),
            );
            debug_ui.add(
                egui::Slider::new(&mut debug_ui_state.min_sweep_distance, 0.0..=400.0)
                    .text("Min Sweep Distance"),
            );
            debug_ui.add(
                egui::Slider::new(&mut debug_ui_state.drag_coefficient, 0.001..=0.01)
                    .text("Drag Coefficient"),
            );
            debug_ui.add(
                egui::Slider::new(&mut debug_ui_state.slow_down_factor, 1.0..=500.0)
                    .text("Slow Down Factor"),
            );
            debug_ui.add(
                egui::Slider::new(&mut debug_ui_state.rotation_factor, 0.001..=0.1)
                    .text("Rotation Factor"),
            );
            debug_ui.add(
                egui::Slider::new(&mut debug_ui_state.snap_distance, 1.0..=50.0)
                    .text("Snap Distance"),
            );
            debug_ui.add(
                egui::Slider::new(&mut debug_ui_state.snap_velocity, 5.0..=100.0)
                    .text("Snap Velocity"),
            );
            debug_ui.add(
                egui::Slider::new(&mut debug_ui_state.speed_up_factor, 0.0..=500.0)
                    .text("Speed Up Factor"),
            );

            debug_ui.separator();
            debug_ui.add(egui::Label::new("Stone Configurations"));

            for (i, stone_config) in debug_ui_state.stone_configs.iter_mut().enumerate() {
                debug_ui.collapsing(format!("Stone {}", i + 1), |debug_ui| {
                    debug_ui.add(
                        egui::Slider::new(&mut stone_config.velocity_magnitude, 0.0..=500.0)
                            .text("Velocity"),
                    );
                    egui::ComboBox::from_id_salt(format!("stone_facing_{}", i))
                        .selected_text(format!("{:?}", stone_config.facing))
                        .show_ui(debug_ui, |debug_ui| {
                            for facing in Facing::iterator() {
                                debug_ui.selectable_value(
                                    &mut stone_config.facing,
                                    facing.clone(),
                                    facing.to_string(),
                                );
                            }
                        });
                });
            }
        });
    Ok(())
}

pub fn on_debug_ui_level_change(
    mut maybe_old_level: Local<Option<CurrentLevel>>,
    debug_ui_state: Res<DebugUIState>,
    commands: Commands,
    grid: Single<Entity, With<HexGrid>>,
    stone_query: Query<Entity, With<Stone>>,
    paused: ResMut<PhysicsPaused>,
    meshes: ResMut<Assets<Mesh>>,
    materials: ResMut<Assets<ColorMaterial>>,
    scratch_materials: ResMut<Assets<ScratchOffMaterial>>,
) {
    let old_level = maybe_old_level.replace(debug_ui_state.current_level);
    if old_level == Some(debug_ui_state.current_level) || old_level.is_none() {
        return;
    }
    restart_game(
        commands,
        grid,
        debug_ui_state,
        stone_query,
        paused,
        meshes,
        materials,
        scratch_materials,
        None,
    );
}
