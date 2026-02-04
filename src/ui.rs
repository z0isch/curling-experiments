use bevy::prelude::*;

use crate::{
    LevelStart, OnLevel, PhysicsPaused, StoneStopped,
    level::CurrentLevel,
    tile::{CurrentDragTileType, TileType},
};

#[derive(Component)]
struct CountdownText;

#[derive(Component)]
struct CountdownUI;

#[derive(Component)]
struct BroomTypeText;

#[derive(Component)]
struct StoneStoppedUI;

#[derive(Component)]
struct TipUI;

#[derive(Resource)]
struct Countdown {
    pub timer: Timer,
    pub count: u32,
}

#[derive(Component)]
pub struct BroomUI;

#[derive(Component)]
pub struct Level0UI;

#[derive(Component)]
pub struct BottomLeftUI;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Startup, setup)
        .add_systems(Update, (update_broom_type_ui, update_countdown))
        .add_observer(on_level_start)
        .add_observer(on_stone_stopped);
}

fn setup(mut commands: Commands) {
    commands.spawn(bottom_left_ui());
    commands.spawn(countdown_ui());
    commands.spawn(stone_stopped_ui());
    commands.spawn(broom_type_ui());

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
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)),
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

fn bottom_left_ui() -> impl Bundle {
    (
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(16.0),
            bottom: Val::Px(16.0),
            padding: UiRect::all(Val::Px(8.0)),
            ..default()
        },
        Pickable::IGNORE,
        BottomLeftUI,
        children![(
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(20.0),
                ..default()
            },
            children![
                (
                    TipUI,
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                        flex_direction: FlexDirection::Column,
                        ..default()
                    },
                    BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.5)),
                ),
                (
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(5.0),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)),
                    children![
                        (
                            Text::new("Controls"),
                            TextFont {
                                font_size: 25.0,
                                ..default()
                            },
                            TextColor(Color::WHITE),
                            Pickable::IGNORE
                        ),
                        (
                            Text::new("R: Restart Level"),
                            TextFont {
                                font_size: 20.0,
                                ..default()
                            },
                            TextColor(Color::WHITE),
                            Pickable::IGNORE
                        ),
                        (
                            Text::new("1-3: Switch Brooms"),
                            TextFont {
                                font_size: 20.0,
                                ..default()
                            },
                            TextColor(Color::WHITE),
                            Pickable::IGNORE
                        )
                    ]
                )
            ]
        )],
    )
}

fn level_0_ui() -> impl Bundle {
    (
        Node {
            width: Val::Percent(100.0),
            justify_content: JustifyContent::Center,
            padding: UiRect::all(Val::Px(50.0)),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(20.0),
            ..default()
        },
        Pickable::IGNORE,
        Level0UI,
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)),
        children![
            (
                Text::new("Let's practice that sweeping technique!"),
                TextFont {
                    font_size: 30.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                Pickable::IGNORE,
            ),
            (
                Text::new("Click and drag on the tile to make it smooth"),
                TextFont {
                    font_size: 30.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                Pickable::IGNORE,
            )
        ],
    )
}

fn level_1_tip_ui() -> Vec<Text> {
    [
        Text::new("You can't effect the stone directly"),
        Text::new("Sweep tiles to control the speed"),
    ]
    .to_vec()
}

fn level_2_tip_ui() -> Vec<Text> {
    [
        Text::new("Number keys change broom types"),
        Text::new("#2 sure might be handy"),
    ]
    .to_vec()
}

fn level_3_tip_ui() -> Vec<Text> {
    [
        Text::new("I wonder what the #3 broom does..."),
        Text::new("Remember that you can hit R at any time to restart"),
    ]
    .to_vec()
}

fn level_4_tip_ui() -> Vec<Text> {
    [Text::new("Let's get BOOSTING!")].to_vec()
}

fn level_5_tip_ui() -> Vec<Text> {
    [Text::new("Good luck with this one ;)")].to_vec()
}

fn tip_ui(current_level: &CurrentLevel) -> Option<Vec<impl Bundle>> {
    let lines = match current_level {
        CurrentLevel::Level0 => vec![],
        CurrentLevel::Level1 => level_1_tip_ui(),
        CurrentLevel::Level2 => level_2_tip_ui(),
        CurrentLevel::Level3 => level_3_tip_ui(),
        CurrentLevel::Level4 => level_4_tip_ui(),
        CurrentLevel::Level5 => level_5_tip_ui(),
    };
    let mut bundles = Vec::new();
    for line in lines {
        bundles.push((
            line,
            TextFont {
                font_size: 20.,
                ..default()
            },
            TextColor(Color::BLACK),
            Pickable::IGNORE,
        ))
    }
    Some(bundles)
}

fn broom_type_ui() -> impl Bundle {
    (
        Node {
            width: Val::Percent(100.0),
            justify_content: JustifyContent::Center,
            padding: UiRect::all(Val::Px(50.0)),
            ..default()
        },
        BroomUI,
        Pickable::IGNORE,
        children![(
            BroomTypeText,
            Text::new(""),
            TextFont {
                font_size: 30.0,
                ..default()
            },
            TextColor(Color::WHITE),
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)),
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
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)),
            Text::new("Too bad! Press R to retry."),
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
            **text = format!(
                "Broom: {}",
                match current_drag_tile_type.0 {
                    TileType::MaintainSpeed => "Straight",
                    TileType::TurnCounterclockwise => "Counterclockwise",
                    TileType::TurnClockwise => "Clockwise",

                    //Shouldn't be able to drag these
                    TileType::SlowDown => "SlowDown",
                    TileType::Goal => "Goal",
                    TileType::Wall => "Wall",
                    TileType::SpeedUp(_) => "SpeedUp",
                }
            );
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
    on_level: Res<OnLevel>,
    mut countdown_text: Single<&mut Text, With<CountdownText>>,
    tip_ui_entity: Single<Entity, With<TipUI>>,
    children_query: Query<&Children>,
    broom_ui_entity: Single<Entity, With<BroomUI>>,
    level_0_ui_entity: Query<Entity, With<Level0UI>>,
    bottom_left_ui_entity: Single<Entity, With<BottomLeftUI>>,
) {
    log::info!("Level start: {:?}", on_level.0.current_level);
    let level = &on_level.0;
    match level.current_level {
        CurrentLevel::Level0 => {
            commands.entity(*broom_ui_entity).insert(Visibility::Hidden);
            commands
                .entity(*bottom_left_ui_entity)
                .insert(Visibility::Hidden);
            commands.spawn(level_0_ui());
        }
        _ => {
            for entity in level_0_ui_entity.iter() {
                commands.entity(entity).despawn();
            }
            commands
                .entity(*bottom_left_ui_entity)
                .insert(Visibility::Visible);
            commands
                .entity(*broom_ui_entity)
                .insert(Visibility::Visible);
        }
    }

    if let Some(c) = level.countdown {
        countdown.count = c;
        countdown.timer.reset();
        **countdown_text = Text(c.to_string());
        commands.entity(*countdown_ui).insert(Visibility::Visible);

        commands
            .entity(*stone_stopped_ui)
            .insert(Visibility::Hidden);
    }

    if let Ok(children) = children_query.get(*tip_ui_entity) {
        for child in children.iter() {
            commands.entity(child).despawn();
        }
    }
    if let Some(tip_ui) = tip_ui(&level.current_level) {
        commands.entity(*tip_ui_entity).with_children(|parent| {
            for bundle in tip_ui {
                parent.spawn(bundle);
            }
        });
    }
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
