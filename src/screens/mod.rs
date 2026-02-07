mod end;
mod gameplay;
mod loading;
mod title;

use bevy::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.init_state::<Screen>();
    app.init_resource::<LoadingDestination>();

    app.add_plugins((
        gameplay::plugin,
        loading::plugin,
        title::plugin,
        end::plugin,
    ));
}

#[derive(Resource, Copy, Clone, Eq, PartialEq, Hash, Debug, Default)]
pub enum LoadingDestination {
    #[default]
    Gameplay,
}

/// The game's main screen states.
#[derive(States, Copy, Clone, Eq, PartialEq, Hash, Debug, Default)]
pub enum Screen {
    #[default]
    Title,
    Loading,
    Gameplay,
    End,
}
