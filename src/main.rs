// Support configuring Bevy lints within code.
#![cfg_attr(bevy_lint, feature(register_tool), register_tool(bevy))]
// Disable console on Windows for non-dev builds.
#![cfg_attr(not(feature = "dev"), windows_subsystem = "windows")]

mod asset_tracking;
mod confetti;
mod crt_postprocess;
//mod debug_ui;
#[cfg(feature = "dev")]
mod dev_tools;
mod fire_trail;
mod gameplay;
mod hex_grid;
mod intersection;
mod level;
mod menus;
mod screens;
mod stone;
mod tile;
mod ui;

use bevy::prelude::*;
use bevy::{asset::AssetMetaCheck, window::WindowResolution};
use bevy_egui::EguiPlugin;
use bevy_rand::{
    plugin::EntropyPlugin,
    prelude::{ChaCha8Rng, WyRand},
};
use bevy_seedling::SeedlingPlugin;
use crt_postprocess::{CrtPostProcessPlugin, CrtSettings, update_crt_time};

fn main() -> AppExit {
    App::new().add_plugins(AppPlugin).run()
}

pub struct AppPlugin;

impl Plugin for AppPlugin {
    fn build(&self, app: &mut App) {
        // Add Bevy plugins.
        app.add_plugins(
            DefaultPlugins
                .set(AssetPlugin {
                    // Wasm builds will check for meta files (that don't exist) if this isn't set.
                    // This causes errors and even panics on web build on itch.
                    // See https://github.com/bevyengine/bevy_github_ci_template/issues/48.
                    meta_check: AssetMetaCheck::Never,
                    ..default()
                })
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        resolution: WindowResolution::new(1024, 768),
                        resizable: false,
                        title: "Hexagon Grid".into(),
                        ..default()
                    }),
                    ..default()
                }),
        )
        .add_plugins(SeedlingPlugin::default())
        .add_plugins(EguiPlugin::default())
        .add_plugins(MeshPickingPlugin)
        .add_plugins((
            EntropyPlugin::<ChaCha8Rng>::default(),
            EntropyPlugin::<WyRand>::default(),
        ))
        .add_plugins(CrtPostProcessPlugin);

        // Add other plugins.
        app.add_plugins((
            asset_tracking::plugin,
            #[cfg(feature = "dev")]
            dev_tools::plugin,
            menus::plugin,
            screens::plugin,
            gameplay::plugin,
        ));

        // Set up the `Pause` state.
        app.init_state::<Pause>();
        app.configure_sets(FixedUpdate, PausableSystems.run_if(in_state(Pause(false))));
        app.configure_sets(Update, PausableSystems.run_if(in_state(Pause(false))));

        // Spawn the main camera.
        app.add_systems(Startup, spawn_camera);

        app.add_systems(Update, update_crt_time);
    }
}

/// Whether or not the game is paused.
#[derive(States, Copy, Clone, Eq, PartialEq, Hash, Debug, Default)]
struct Pause(pub bool);

/// A system set for systems that shouldn't run while the game is paused.
#[derive(SystemSet, Copy, Clone, Eq, PartialEq, Hash, Debug)]
struct PausableSystems;

fn spawn_camera(mut commands: Commands, mut mesh_picking_settings: ResMut<MeshPickingSettings>) {
    commands.spawn((
        Name::new("Camera"),
        Camera2d,
        CrtSettings::default(),
        MeshPickingCamera,
    ));
    mesh_picking_settings.require_markers = true;
}
