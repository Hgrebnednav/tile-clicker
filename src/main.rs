use bevy::prelude::*;
use bevy::window::WindowResolution;
mod game;
mod main_menu;

use game::{FIELD_SIZE_X, FIELD_SIZE_Y, SCORE_HEIGHT};

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    resolution: WindowResolution::new(FIELD_SIZE_X, FIELD_SIZE_Y + SCORE_HEIGHT)
                        .with_scale_factor_override(1.0),
                    ..default()
                }),
                ..default()
            }),
        )
        .add_plugins(main_menu::MainMenuPlugin)
        .add_plugins(game::GamePlugin)
        .run()
}

/// Generic system that takes a component as a parameter, and will despawn all entities with that component
fn despawn_screen<T: Component>(to_despawn: Query<Entity, With<T>>, mut commands: Commands) {
    for entity in &to_despawn {
        commands.entity(entity).despawn_recursive();
    }
}
