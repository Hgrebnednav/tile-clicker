//! # Tile Clicker Game
//! Try to click tiles when they spawn.
//! The game lasts 30 seconds.
//!
//! A tile spawn after a timer reaches zero or if the only tile is clicked.
//! The timer starts with a time of 0.8s at the end of the game this becomes (0.8/3)s.
//! This is done by modifying the relative speed of [`bevy::time::Time<Virtual>`].
//! The relative speed is given by (2/30²)t_e² + 1, where t_e is the elapsed real time
//! since the start of the game.
//!
//! Points are given when a tile is clicked in a timely manner.
//! The longer it takes to click a tile the less points are received.
//! At the start of the game it takes 5s before a tile is worth zero points, at the end of the game
//! it takes (5/3)s until a tile is worth zero points.
//! The time is decreased each frame with [`bevy::time::Time<Virtual>::delta()`].
//!
//! The game is over after 30s or when clicking on an empty tile.

use bevy::prelude::*;
use bevy::window::WindowResolution;
mod game;
mod ui;

use game::{FIELD_SIZE_X, FIELD_SIZE_Y, SCORE_HEIGHT};

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Tile Clicker".to_owned(),
                    resolution: WindowResolution::new(FIELD_SIZE_X, FIELD_SIZE_Y + SCORE_HEIGHT)
                        .with_scale_factor_override(1.0),
                    resizable: false,
                    canvas: Some("#tile-clicker-canvas".into()),
                    ..default()
                }),
                ..default()
            }),
        )
        .add_plugins(game::LoadingPlugin)
        .add_plugins(ui::UiPlugin)
        .add_plugins(game::GamePlugin)
        .run()
}

/// Generic system that takes a component as a parameter, and will despawn all entities with that component
fn despawn_on_screen<T: Component>(to_despawn: Query<Entity, With<T>>, mut commands: Commands) {
    for entity in &to_despawn {
        commands.entity(entity).despawn_recursive();
    }
}
