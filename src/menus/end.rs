//! The credits menu.

use bevy::{input::common_conditions::input_just_pressed, prelude::*};

use crate::menus::{Menu, settings::btn};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Menu::End), spawn_end_menu);
    app.add_systems(
        Update,
        go_back.run_if(in_state(Menu::End).and(input_just_pressed(KeyCode::Escape))),
    );
}

fn spawn_end_menu(mut commands: Commands) {
    commands
        .spawn((
            GlobalZIndex(2),
            DespawnOnExit(Menu::End),
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
            Pickable::IGNORE,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("Thanks for playing!"),
                TextFont {
                    font_size: 20.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)),
            ));
            parent.spawn((
                Node {
                    justify_content: JustifyContent::SpaceAround,
                    width: Val::Percent(100.0),
                    ..Default::default()
                },
                children![btn("End game", go_back_on_click),],
            ));
        });
}

fn go_back_on_click(_: On<Pointer<Click>>, mut next_menu: ResMut<NextState<Menu>>) {
    next_menu.set(Menu::Main);
}

fn go_back(mut next_menu: ResMut<NextState<Menu>>) {
    next_menu.set(Menu::Main);
}
