//! The main menu (seen on the title screen).

use bevy::prelude::*;

use crate::{
    asset_tracking::ResourceHandles,
    menus::{Menu, settings::btn},
    screens::Screen,
};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Menu::Main), spawn_main_menu);
}

fn spawn_main_menu(mut commands: Commands) {
    commands
        .spawn((
            GlobalZIndex(2),
            DespawnOnExit(Menu::Main),
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
                Text::new("Broom Shakalaka"),
                TextFont {
                    font_size: 100.0,
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
                children![btn("Play", enter_loading_or_gameplay_screen),],
            ));

            parent.spawn((
                Node {
                    justify_content: JustifyContent::SpaceAround,
                    width: Val::Percent(100.0),
                    ..Default::default()
                },
                children![btn("Settings", open_settings_menu),],
            ));
            parent.spawn((
                Node {
                    justify_content: JustifyContent::SpaceAround,
                    width: Val::Percent(100.0),
                    ..Default::default()
                },
                children![btn("Credits", open_credits_menu),],
            ));
        });
}

fn enter_loading_or_gameplay_screen(
    _: On<Pointer<Click>>,
    resource_handles: Res<ResourceHandles>,
    mut next_screen: ResMut<NextState<Screen>>,
) {
    if resource_handles.is_all_done() {
        next_screen.set(Screen::Gameplay);
    } else {
        next_screen.set(Screen::Loading);
    }
}

fn open_settings_menu(_: On<Pointer<Click>>, mut next_menu: ResMut<NextState<Menu>>) {
    next_menu.set(Menu::Settings);
}

fn open_credits_menu(_: On<Pointer<Click>>, mut next_menu: ResMut<NextState<Menu>>) {
    next_menu.set(Menu::Credits);
}
