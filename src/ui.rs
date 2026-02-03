use bevy::prelude::*;

use crate::{Countdown, PhysicsPaused, tile::CurrentDragTileType};

#[derive(Component)]
pub struct CountdownText;
#[derive(Component)]
pub struct CountdownUI;

#[derive(Component)]
pub struct BroomTypeText;

pub fn spawn_countdown(commands: &mut Commands, countdown: u32) {
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            Pickable::IGNORE,
            CountdownUI,
        ))
        .with_children(|parent| {
            parent.spawn((
                CountdownText,
                Text::new(countdown.to_string()),
                TextFont {
                    font_size: 120.0,
                    ..default()
                },
                TextColor(Color::srgba(1.0, 0.9, 0.2, 0.9)),
                Pickable::IGNORE,
            ));
        });
}

pub fn spawn_broom_type_ui(commands: &mut Commands) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(16.0),
                bottom: Val::Px(16.0),
                padding: UiRect::all(Val::Px(8.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)),
            Pickable::IGNORE,
        ))
        .with_children(|parent| {
            parent.spawn((
                BroomTypeText,
                Text::new("Broom: MaintainSpeed"),
                TextFont {
                    font_size: 20.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                Pickable::IGNORE,
            ));
        });
}

pub fn update_broom_type_ui(
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
pub fn update_countdown(
    mut commands: Commands,
    time: Res<Time>,
    mut countdown: ResMut<Countdown>,
    mut paused: ResMut<PhysicsPaused>,
    mut text_query: Query<&mut Text, With<CountdownText>>,
    mut countdown_ui_query: Query<Entity, With<CountdownUI>>,
) {
    // Only run countdown while physics is paused and countdown is active
    if !paused.0 || countdown.count == 0 {
        return;
    }

    countdown.timer.tick(time.delta());

    if countdown.timer.just_finished() {
        countdown.count = countdown.count.saturating_sub(1);

        if countdown.count == 0 {
            for entity in &mut countdown_ui_query {
                commands.entity(entity).despawn();
            }
            paused.0 = false;
        } else {
            // Update the countdown text
            for mut text in &mut text_query {
                **text = countdown.count.to_string();
            }
        }
    }
}

