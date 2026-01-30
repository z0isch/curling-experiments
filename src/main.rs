use avian3d::prelude::*;
use bevy::{
    camera_controller::free_camera::{FreeCamera, FreeCameraPlugin},
    prelude::*,
};
use bevy_egui::{EguiContexts, EguiPlugin, EguiPrimaryContextPass, egui};

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            FreeCameraPlugin,
            PhysicsPlugins::default(),
            EguiPlugin::default(),
            PhysicsDebugPlugin,
        ))
        .add_systems(Startup, setup)
        .add_systems(Update, spawn_stone)
        .add_systems(EguiPrimaryContextPass, ui_example_system)
        .run();
}

fn ui_example_system(mut contexts: EguiContexts, mut ui_state: ResMut<UiState>) -> Result {
    egui::SidePanel::left("side_panel")
        .default_width(200.0)
        .show(contexts.ctx_mut()?, |ui| {
            ui.vertical(|ui| {
                ui.label("Press Space to spawn a stone");
                ui.label("Press R to remove all stones");
                ui.label("Press F to toggle free camera");
                ui.add(
                    egui::Slider::new(&mut ui_state.inner_radius, 0.1..=4.0)
                        .text("stone inner radius"),
                );
                ui.add(
                    egui::Slider::new(&mut ui_state.outer_radius, 0.1..=4.0)
                        .text("stone outer radius"),
                );
                ui.add(
                    egui::Slider::new(&mut ui_state.stone_height, 0.1..=4.0).text("stone height"),
                );
                ui.add(egui::Slider::new(&mut ui_state.friction, 0.0..=5.0).text("friction"));
                ui.add(egui::Slider::new(&mut ui_state.velocity_x, 0.0..=20.0).text("x velocity"));
                ui.add(egui::Slider::new(&mut ui_state.velocity_z, 0.0..=20.0).text("z velocity"));
                ui.add(
                    egui::Slider::new(&mut ui_state.angular_velocity_around_y, 0.0..=50.0)
                        .text("angular velocity around y"),
                );
            });
        });
    Ok(())
}

#[derive(Resource)]
struct UiState {
    free_camera: bool,
    inner_radius: f32,
    outer_radius: f32,
    stone_height: f32,
    friction: f32,
    velocity_x: f32,
    velocity_z: f32,
    angular_velocity_around_y: f32,
}

#[derive(Component)]
struct Stone;

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        RigidBody::Static,
        Collider::cylinder(100.0, 0.1),
        Friction::new(0.),
        Mesh3d(meshes.add(Cylinder::new(100.0, 0.1))),
        MeshMaterial3d(materials.add(Color::WHITE)),
    ));

    // Light
    commands.spawn((
        PointLight { ..default() },
        Transform::from_xyz(4.0, 8.0, 4.0),
    ));

    // Camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0., 30., 0.).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    commands.insert_resource(UiState {
        free_camera: false,
        inner_radius: 0.2,
        outer_radius: 0.3,
        stone_height: 0.1,
        friction: 2.,
        velocity_x: 15.,
        velocity_z: 0.,
        angular_velocity_around_y: 60.,
    });
}

fn spawn_stone(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    keys: Res<ButtonInput<KeyCode>>,
    mut ui_state: ResMut<UiState>,
    stones: Query<Entity, With<Stone>>,
    camera: Single<(Entity, &Transform), With<Camera3d>>,
) {
    log::info!("camera: {:?}", camera.1.translation);
    let mesh_handle = meshes.add(
        Mesh::from(Extrusion::new(
            Annulus::new(ui_state.inner_radius, ui_state.outer_radius),
            ui_state.stone_height,
        ))
        .rotated_by(Quat::from_rotation_x(std::f32::consts::FRAC_PI_2)),
    );
    let mesh = meshes.get(&mesh_handle).unwrap();

    if keys.just_pressed(KeyCode::KeyR) {
        for stone in stones.iter() {
            commands.entity(stone).despawn();
        }
    }
    if keys.just_pressed(KeyCode::KeyF) {
        let mut c = commands.get_entity(camera.0).unwrap();
        ui_state.free_camera = !ui_state.free_camera;
        if ui_state.free_camera {
            c.insert(FreeCamera::default());
        } else {
            c.remove::<FreeCamera>();
        }
    }
    if keys.just_pressed(KeyCode::Space) {
        commands.spawn((
            Stone,
            Friction::new(ui_state.friction),
            RigidBody::Dynamic,
            Collider::convex_decomposition_from_mesh(mesh).unwrap(),
            LinearVelocity(Vec3::new(ui_state.velocity_x, 0., ui_state.velocity_z)),
            AngularVelocity(Vec3::new(0., ui_state.angular_velocity_around_y, 0.)),
            Mesh3d(meshes.add(Cylinder::new(ui_state.outer_radius, 0.1))),
            Transform::from_xyz(
                -15. + ui_state.outer_radius,
                ui_state.stone_height / 2.0 + 0.1,
                -5.0,
            ),
        ));
    }
}
