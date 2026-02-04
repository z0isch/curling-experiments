use bevy::prelude::*;

use crate::{
    GameStart, LevelStart, OnLevel, PhysicsPaused, StoneStopped,
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

#[derive(Component)]
pub struct MainUI;

#[derive(Component)]
pub struct TitleScreenUI;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Startup, setup)
        .add_systems(Update, (update_broom_type_ui, update_countdown))
        .add_observer(on_level_start)
        .add_observer(on_stone_stopped);
}

fn setup(mut commands: Commands) {
    commands.insert_resource(Countdown {
        timer: Timer::from_seconds(1.0, TimerMode::Repeating),
        count: 0,
    });
}

pub fn spawn_title_screen_ui(mut commands: Commands) {
    commands
        .spawn((
            TitleScreenUI,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(50.0),
                ..default()
            },
            Visibility::default(),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("CURLING"),
                TextFont {
                    font_size: 100.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)),
            ));

            parent
                .spawn((
                    BorderColor::all(Color::BLACK),
                    BackgroundColor(Color::WHITE),
                    Node {
                        width: px(150.0),
                        height: px(65.0),
                        border: UiRect::all(px(5.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    Visibility::default(),
                ))
                .with_children(|p2| {
                    p2.spawn((
                        Text::new("Play"),
                        TextColor(Color::BLACK),
                        TextFont::default().with_font_size(40.0),
                    ));
                })
                .observe(
                    |_ev: On<Pointer<Click>>,
                     mut commands: Commands,
                     title_screen_query: Query<Entity, With<TitleScreenUI>>| {
                        commands.trigger(GameStart);
                        for e in title_screen_query.iter() {
                            commands.entity(e).despawn();
                        }
                    },
                );
        });
}

fn countdown_ui(time_left: u32) -> impl Bundle {
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
        MainUI,
        Visibility::Visible,
        children![(
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)),
            CountdownText,
            Text::new(time_left.to_string()),
            TextFont {
                font_size: 120.0,
                ..default()
            },
            TextColor(Color::srgba(1.0, 0.9, 0.2, 0.9)),
            Pickable::IGNORE,
        )],
    )
}

fn spawn_bottom_left_ui(mut commands: Commands, current_level: &CurrentLevel) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(16.0),
                bottom: Val::Px(16.0),
                padding: UiRect::all(Val::Px(8.0)),
                ..default()
            },
            Pickable::IGNORE,
            BottomLeftUI,
            Visibility::default(),
            MainUI,
        ))
        .with_children(|p1| {
            p1.spawn((
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(20.0),
                    ..default()
                },
                Visibility::default(),
            ))
            .with_children(|p2| {
                p2.spawn((
                    TipUI,
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                        flex_direction: FlexDirection::Column,
                        ..default()
                    },
                    BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.5)),
                ))
                .with_children(|p3| {
                    if let Some(tips) = tip_ui(current_level) {
                        for tip in tips {
                            p3.spawn(tip);
                        }
                    }
                });

                p2.spawn((
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(5.0),
                        ..default()
                    },
                    Visibility::default(),
                    BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)),
                ))
                .with_children(|p3| {
                    p3.spawn((
                        Text::new("Controls"),
                        TextFont {
                            font_size: 25.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                        Pickable::IGNORE,
                    ));
                    p3.spawn((
                        Text::new("R: Restart Level"),
                        TextFont {
                            font_size: 20.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                        Pickable::IGNORE,
                    ));
                    p3.spawn((
                        Text::new("1-3: Switch Brooms"),
                        TextFont {
                            font_size: 20.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                        Pickable::IGNORE,
                    ));
                });
            });
        });
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
        Text::new("I wonder what the #3 does..."),
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

fn broom_type_ui(tile_type: &TileType) -> impl Bundle {
    (
        Node {
            width: Val::Percent(100.0),
            justify_content: JustifyContent::Center,
            padding: UiRect::all(Val::Px(50.0)),
            ..default()
        },
        MainUI,
        BroomUI,
        Pickable::IGNORE,
        children![(
            BroomTypeText,
            Text::new(get_broom_type_text(tile_type)),
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
        MainUI,
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

fn get_broom_type_text(tile_type: &TileType) -> String {
    format!(
        "Broom: {}",
        match tile_type {
            TileType::MaintainSpeed => "Straight",
            TileType::TurnCounterclockwise => "Counterclockwise",
            TileType::TurnClockwise => "Clockwise",

            //Shouldn't be able to drag these
            TileType::SlowDown => "SlowDown",
            TileType::Goal => "Goal",
            TileType::Wall => "Wall",
            TileType::SpeedUp(_) => "SpeedUp",
        }
    )
}

fn update_broom_type_ui(
    current_drag_tile_type: Res<CurrentDragTileType>,
    mut text_query: Query<&mut Text, With<BroomTypeText>>,
) {
    if current_drag_tile_type.is_changed() {
        for mut text in &mut text_query {
            **text = get_broom_type_text(&current_drag_tile_type.0)
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
            commands.entity(*countdown_ui_query).despawn();
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
    mut countdown: ResMut<Countdown>,
    on_level: Res<OnLevel>,
    level_0_ui_entity: Query<Entity, With<Level0UI>>,
    main_ui_entity: Query<Entity, With<MainUI>>,
    current_drag_tile_type: Res<CurrentDragTileType>,
) {
    for entity in main_ui_entity.iter() {
        commands.entity(entity).despawn();
    }

    for entity in level_0_ui_entity.iter() {
        commands.entity(entity).despawn();
    }

    let level = &on_level.0;
    match level.current_level {
        CurrentLevel::Level0 => {
            commands.spawn(level_0_ui());
        }
        _ => {
            commands.spawn(broom_type_ui(&current_drag_tile_type.0));
            if let Some(c) = level.countdown {
                countdown.count = c;
                countdown.timer.reset();
                commands.spawn(countdown_ui(c));
            }
            spawn_bottom_left_ui(commands, &level.current_level);
        }
    }
}

fn on_stone_stopped(
    mut _ev: On<StoneStopped>,
    mut commands: Commands,
    stone_stopped_ui_entity: Query<Entity, With<StoneStoppedUI>>,
) {
    if stone_stopped_ui_entity.is_empty() {
        commands.spawn(stone_stopped_ui());
    }
}
