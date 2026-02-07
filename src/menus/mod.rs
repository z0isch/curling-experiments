//! The game's menus and transitions between them.

mod credits;
mod end;
mod main;
mod pause;
mod settings;

use bevy::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.init_state::<Menu>();

    app.add_plugins((
        main::plugin,
        settings::plugin,
        pause::plugin,
        credits::plugin,
        end::plugin,
    ));
}

#[derive(States, Copy, Clone, Eq, PartialEq, Hash, Debug, Default)]
pub enum Menu {
    #[default]
    None,
    Main,
    Settings,
    Credits,
    Pause,
    End,
}
