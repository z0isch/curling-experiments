use bevy::prelude::*;

use crate::{LevelStart, OnLevel, PhysicsPaused, StoneStopped, tile::CurrentDragTileType};

#[derive(Component)]
struct CountdownText;

#[derive(Component)]
struct CountdownUI;

#[derive(Component)]
struct BroomTypeText;

#[derive(Component)]
struct StoneStoppedUI;

#[derive(Resource)]
struct Countdown {
    pub timer: Timer,
    pub count: u32,
}

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Startup, setup)
        .add_systems(Update, (update_broom_type_ui, update_countdown))
        .add_observer(on_level_start)
        .add_observer(on_stone_stopped);
}

fn setup(mut commands: Commands) {
    commands.spawn(broom_type_ui());
    commands.spawn(countdown_ui());
    commands.spawn(stone_stopped_ui());

    commands.insert_resource(Countdown {
        timer: Timer::from_seconds(1.0, TimerMode::Repeating),
        count: 0,
    });
}

fn countdown_ui() -> impl Bundle {
    (
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        Pickable::IGNORE,
        CountdownUI,
        Visibility::Hidden,
        children![(
            CountdownText,
            Text::new(""),
            TextFont {
                font_size: 120.0,
                ..default()
            },
            TextColor(Color::srgba(1.0, 0.9, 0.2, 0.9)),
            Pickable::IGNORE,
        )],
    )
}

fn broom_type_ui() -> impl Bundle {
    (
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(16.0),
            bottom: Val::Px(16.0),
            padding: UiRect::all(Val::Px(8.0)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)),
        Pickable::IGNORE,
        children![(
            BroomTypeText,
            Text::new("Broom: MaintainSpeed"),
            TextFont {
                font_size: 20.0,
                ..default()
            },
            TextColor(Color::WHITE),
            Pickable::IGNORE,
        )],
    )
}

fn stone_stopped_ui() -> impl Bundle {
    (
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        Pickable::IGNORE,
        StoneStoppedUI,
        Visibility::Hidden,
        children![(
            Text::new("Too bad! Press `R` to retry."),
            TextFont {
                font_size: 50.0,
                ..default()
            },
            TextColor(Color::srgba(1.0, 0.9, 0.2, 0.9)),
            Pickable::IGNORE,
        )],
    )
}

fn update_broom_type_ui(
    current_drag_tile_type: Res<CurrentDragTileType>,
    mut text_query: Query<&mut Text, With<BroomTypeText>>,
) {
    if current_drag_tile_type.is_changed() {
        for mut text in &mut text_query {
            **text = format!("Broom: {:?}", current_drag_tile_type.0);
        }
    }
}

/// System that updates the countdown and starts physics when it reaches zero
fn update_countdown(
    mut commands: Commands,
    time: Res<Time>,
    mut countdown: ResMut<Countdown>,
    mut paused: ResMut<PhysicsPaused>,
    mut text_query: Query<&mut Text, With<CountdownText>>,
    countdown_ui_query: Single<Entity, With<CountdownUI>>,
) {
    // Only run countdown while physics is paused and countdown is active
    if !paused.0 || countdown.count == 0 {
        return;
    }

    countdown.timer.tick(time.delta());

    if countdown.timer.just_finished() {
        countdown.count = countdown.count.saturating_sub(1);

        if countdown.count == 0 {
            commands
                .entity(*countdown_ui_query)
                .insert(Visibility::Hidden);
            paused.0 = false;
        } else {
            // Update the countdown text
            for mut text in &mut text_query {
                **text = countdown.count.to_string();
            }
        }
    }
}

// Observers

fn on_level_start(
    mut _ev: On<LevelStart>,
    mut commands: Commands,
    stone_stopped_ui: Single<Entity, With<StoneStoppedUI>>,
    countdown_ui: Single<Entity, With<CountdownUI>>,
    mut countdown: ResMut<Countdown>,
    level: Res<OnLevel>,
) {
    countdown.count = level.0.countdown;
    countdown.timer.reset();

    commands.entity(*countdown_ui).insert(Visibility::Visible);
    commands
        .entity(*stone_stopped_ui)
        .insert(Visibility::Hidden);
}

fn on_stone_stopped(
    mut _ev: On<StoneStopped>,
    mut commands: Commands,
    stone_stopped_ui: Single<Entity, With<StoneStoppedUI>>,
) {
    commands
        .entity(*stone_stopped_ui)
        .insert(Visibility::Visible);
}
