//! # The game
//! A game session consists of four phases
//! - Setup, [`RunningState::Paused`]
//! - Running, [`RunningState::Running`] game can be played
//! - Finished (menu), [`RunningState::Finished`] game has finished menu is open
//! - Cleanup, after pressing a menu button the game is set to a "clean" state for restarting or
//! going back to the main menu

use bevy::prelude::*;
use bevy::time::Stopwatch;

pub const FIELD_SIZE_X: f32 = 500.0;
pub const FIELD_SIZE_Y: f32 = 500.0;
pub const SCORE_HEIGHT: f32 = 80.0;
pub const TILE_NUM_X: usize = 5;
pub const TILE_NUM_Y: usize = TILE_NUM_X;
pub const TILE_SIZE_X: f32 = FIELD_SIZE_X / TILE_NUM_X as f32;
pub const TILE_SIZE_Y: f32 = FIELD_SIZE_Y / TILE_NUM_Y as f32;
pub const BASE_DELAY: f32 = 0.8;
/// Total time in seconds the game lasts
pub const GAME_DURATION: f32 = 30.0;

mod input;
mod loading;

use crate::despawn_screen;
use crate::main_menu::{HOVERED_BUTTON, NORMAL_BUTTON, PRESSED_BUTTON};
use input::ClickEvent;
pub use loading::{Assets, LoadingPlugin};

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum GameState {
    #[default]
    Loading,
    Menu,
    Game,
}

/// Indicate the state during [`GameState::Game`].
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
enum RunningState {
    /// Not used except for starting state
    #[default]
    Paused,
    /// Game is running, player can interact with tiles
    Running,
    /// Game is finished, only interact with menu
    Finished,
}

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        let top_left = Vec2::new(0.0, SCORE_HEIGHT);
        let bottom_right = Vec2::new(FIELD_SIZE_X, FIELD_SIZE_Y + SCORE_HEIGHT);
        let input_plugin = input::InputPlugin::new(
            RunningState::Running,
            UVec2::new(TILE_NUM_X as u32, TILE_NUM_Y as u32),
            top_left,
            bottom_right,
        );
        app.add_state::<GameState>()
            .add_state::<RunningState>()
            .add_plugins(input_plugin)
            .add_event::<FinishedEvent>()
            .add_event::<SpawnNewEvent>()
            .add_event::<SoundEvent>()
            .insert_resource(Msaa::Off)
            .add_systems(OnEnter(GameState::Game), setup_game)
            .add_systems(PostUpdate, play_sound.run_if(in_state(GameState::Game)))
            .add_systems(
                OnExit(GameState::Game),
                (
                    despawn_screen::<OnGameScreen>,
                    despawn_screen::<OnSessionScreen>,
                    cleanup,
                ),
            )
            .add_systems(OnEnter(RunningState::Running), setup_session)
            .add_systems(
                PreUpdate,
                update_game_time.run_if(in_state(RunningState::Running)),
            )
            .add_systems(
                PostUpdate,
                (check_finished, spawn_tile).run_if(in_state(RunningState::Running)),
            )
            .add_systems(
                Update,
                (click, update_score, tile_spawn_timer, update_tile_points)
                    .run_if(in_state(RunningState::Running)),
            )
            .add_systems(OnEnter(RunningState::Finished), setup_menu)
            .add_systems(
                Update,
                button_system.run_if(in_state(RunningState::Finished)),
            )
            .add_systems(
                OnExit(RunningState::Finished),
                (despawn_screen::<OnSessionScreen>, cleanup_session),
            );
    }
}

/// The elapsed time of a game.
#[derive(Debug, Resource)]
struct GameTime(Stopwatch);

/// Timer driving the spawn time of tiles
#[derive(Debug, Resource)]
struct SpawnTimer(Timer);

/// Timer to disable first 0.8s of menu interaction
#[derive(Debug, Resource)]
struct MenuActiveDelay(Timer);

/// The score of player
#[derive(Debug, Default, Resource)]
struct Score(usize);

/// Tile position of the last spawned tile
#[derive(Debug, Default, Resource)]
struct LastSpawn(UVec2);

/// Indicate how the game ended
#[derive(Debug, Event)]
enum FinishedEvent {
    Lost,
    Finished,
}

/// Spawn a new tile
#[derive(Debug, Event)]
enum SpawnNewEvent {
    /// Normal game tile
    Normal,
    /// Tile with position indicating a misclick
    Error((u32, u32)),
}

/// Possible sounds to play
#[derive(Debug, Event)]
enum SoundEvent {
    Normal,
    Error,
}

/// Tag for entities in [`GameState::Game`]
#[derive(Debug, Default, Component)]
struct OnGameScreen;

/// Tag for entities in [`RunningState::Running`]
#[derive(Debug, Default, Component)]
struct OnSessionScreen;

/// Tag indicating score text
#[derive(Debug, Component)]
struct ScoreText;

/// Kinds of buttons in menu
#[derive(Debug, Clone, Copy, PartialEq, Eq, Component)]
enum Button {
    /// Go back to "main menu"
    Menu,
    /// Restart the game
    Restart,
}

impl Button {
    /// All buttons for the menu
    const ALL: &'static [Self] = &[Self::Menu, Self::Restart];
}

/// Grid configuration for the game
type GameGrid = Grid<TILE_NUM_X, TILE_NUM_Y>;

/// A grid indicating which tiles exist.
/// The grid has X elements in X direction and Y elements in Y direction.
/// The [`bevy::ecs::entity::Entity`] is the entity containing all the components of the Tile.
/// The [`bevy::time::Timer`] is for tracking how many points a tile is worth.
#[derive(Debug, Clone, Resource)]
struct Grid<const X: usize, const Y: usize> {
    tiles: [[Option<(Entity, Timer)>; X]; Y],
}

impl<const X: usize, const Y: usize> std::ops::Index<[usize; 2]> for Grid<X, Y> {
    type Output = Option<(Entity, Timer)>;
    fn index(&self, index: [usize; 2]) -> &Self::Output {
        &self.tiles[index[0]][index[1]]
    }
}

impl<const X: usize, const Y: usize> std::ops::IndexMut<[usize; 2]> for Grid<X, Y> {
    fn index_mut(&mut self, index: [usize; 2]) -> &mut Self::Output {
        &mut self.tiles[index[0]][index[1]]
    }
}

impl<const X: usize, const Y: usize> Grid<X, Y> {
    /// New instance of an empty grid
    fn new() -> Self {
        let tiles = std::array::from_fn(|_| std::array::from_fn(|_| None));
        Self { tiles }
    }

    /// Set a tile.
    fn set(&mut self, x: usize, y: usize, entity: Entity) {
        let x = x.min(X - 1);
        let y = y.min(Y - 1);
        self.tiles[y][x] = Some((entity, Timer::from_seconds(5.0, TimerMode::Once)));
    }

    /// Remove a tile from the grid.
    /// Returns the entity and the score of the tile.
    fn take(&mut self, x: usize, y: usize) -> Option<(Entity, usize)> {
        let x = x.min(X - 1);
        let y = y.min(Y - 1);
        let score = |t: Timer| (t.remaining_secs() * 2.0).trunc() as usize;
        self.tiles[y][x].take().map(|(e, t)| (e, score(t)))
    }

    /// Update the timers of the tiles.
    /// This should be called every frame until the game ends.
    fn tick(&mut self, delta: bevy::utils::Duration) {
        let iter = self.tiles.iter_mut().flat_map(|row| row.iter_mut());
        for tile in iter {
            tile.as_mut().map(|(_, t)| t.tick(delta));
        }
    }

    /// Check if all tiles are occupied.
    fn is_full(&self) -> bool {
        self.free_tiles() == 0
    }

    /// Number of free tiles available on the grid.
    fn free_tiles(&self) -> usize {
        self.tiles
            .iter()
            .flat_map(|row| row.iter())
            .filter(|tile| tile.is_none())
            .count()
    }

    /// Number of occupied tiles on the grid.
    fn filled_tiles(&self) -> usize {
        (X * Y) - self.free_tiles()
    }
}

/// Configure all game things not associated with an specific session
fn setup_game(
    mut commands: Commands,
    assets: Res<Assets>,
    mut state: ResMut<NextState<RunningState>>,
) {
    info!("Setup Gam");
    commands.spawn((Camera2dBundle::default(), OnGameScreen));

    // Fill field with tile pattern
    // TODO: try shader for tile pattern
    for y in 0..TILE_NUM_Y {
        for x in 0..TILE_NUM_X {
            tile::<OnGameScreen>(
                &mut commands,
                UVec3::new(x as u32, y as u32, 0),
                Color::rgb(0.8, 0.8, 0.8),
            );
        }
    }
    // Score and remaining time UI
    let font: Handle<Font> = assets.font.clone();
    fn text_section(s: &str, font: Handle<Font>) -> TextSection {
        TextSection {
            value: s.into(),
            style: TextStyle {
                font_size: 52.0,
                color: Color::WHITE,
                font,
            },
        }
    }
    commands
        .spawn(TextBundle {
            text: Text::from_sections([
                text_section("Score: ", font.clone()),
                text_section("0", font.clone()),
                text_section(" Time: ", font.clone()),
                text_section(&GAME_DURATION.to_string(), font.clone()),
            ]),
            ..default()
        })
        .insert(ScoreText)
        .insert(OnGameScreen);

    // Transision from Paused to Running state
    state.set(RunningState::Running);
}

/// Setup session specific resources
fn setup_session(mut commands: Commands, mut time: ResMut<Time<Virtual>>) {
    info!("Setup Session");
    commands.insert_resource(GameGrid::new());
    commands.insert_resource(GameTime(Stopwatch::new()));
    commands.insert_resource(Score(0));
    commands.insert_resource(LastSpawn(UVec2::default()));
    commands.insert_resource(SpawnTimer(Timer::from_seconds(
        BASE_DELAY,
        TimerMode::Repeating,
    )));
    time.unpause();
    time.set_relative_speed(1.0);
}

/// Setup a menu
fn setup_menu(mut commands: Commands, assets: Res<Assets>) {
    // Prevent accidental clicking on menu item just after the game has ended
    commands.insert_resource(MenuActiveDelay(Timer::from_seconds(0.8, TimerMode::Once)));
    commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(90.0),
                padding: UiRect::new(Val::Auto, Val::Auto, Val::Px(10.0), Val::Px(10.0)),
                justify_content: JustifyContent::Center,
                flex_direction: FlexDirection::Column,
                ..default()
            },
            background_color: Color::NONE.into(),
            ..default()
        })
        .insert(OnSessionScreen)
        .insert(OnGameScreen)
        .with_children(|parent| {
            for button in Button::ALL.iter() {
                parent
                    .spawn(ButtonBundle {
                        style: Style {
                            width: Val::Percent(50.0),
                            height: Val::Percent(15.0),
                            margin: UiRect::new(Val::Auto, Val::Auto, Val::Px(10.0), Val::Px(10.0)),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            border: UiRect::all(Val::Px(2.0)),
                            ..default()
                        },
                        background_color: Color::rgb(0.2, 0.2, 0.2).into(),
                        border_color: Color::rgb(0.5, 0.2, 0.2).into(),
                        ..default()
                    })
                    .insert(*button)
                    .insert(OnGameScreen)
                    .insert(OnSessionScreen)
                    .with_children(|parent| {
                        parent
                            .spawn(TextBundle::from_section(
                                format!("{:?}", button),
                                TextStyle {
                                    font: assets.font.clone(),
                                    font_size: 40.0,
                                    color: Color::rgb(0.9, 0.9, 0.9),
                                },
                            ))
                            .insert(OnSessionScreen)
                            .insert(OnGameScreen);
                    });
            }
        });
}

/// Set to clean state after a session
fn cleanup(mut state: ResMut<NextState<RunningState>>, mut clicks: EventReader<ClickEvent>) {
    state.set(RunningState::Paused);
    clicks.clear();
}

/// Set to clean state after a session
fn cleanup_session(mut clicks: EventReader<ClickEvent>) {
    clicks.clear();
}

/// Create a new tile at `pos`
/// `S` is a marker component for marking a tile as either [`OnGameScreen`] or [`OnSessionScreen`]
fn tile<S: Default + Component>(commands: &mut Commands, pos: UVec3, color: Color) -> Entity {
    let x = -(FIELD_SIZE_X - TILE_SIZE_X) / 2.0 + pos.x as f32 * TILE_SIZE_X;
    let y = -(FIELD_SIZE_Y - TILE_SIZE_Y) / 2.0 + pos.y as f32 * TILE_SIZE_Y;
    let y = -y - SCORE_HEIGHT / 2.0;
    let translation = Vec3::new(x, y, pos.z as f32);
    commands
        .spawn((
            SpriteBundle {
                sprite: Sprite {
                    color,
                    custom_size: Some(Vec2::new(TILE_SIZE_X, TILE_SIZE_Y) * 0.95),
                    ..default()
                },
                transform: Transform::from_translation(translation),
                ..default()
            },
            <S as Default>::default(),
        ))
        .id()
}

/// Send an event to spawn a new tile when the timer finishes
fn tile_spawn_timer(timer: Res<SpawnTimer>, mut events: EventWriter<SpawnNewEvent>) {
    if timer.0.finished() {
        events.send(SpawnNewEvent::Normal);
    }
}

/// Spawn the tiles received from the event reader
fn spawn_tile(
    mut commands: Commands,
    mut tiles: ResMut<GameGrid>,
    mut events: EventReader<SpawnNewEvent>,
    mut timer: ResMut<SpawnTimer>,
    mut last_spawn: ResMut<LastSpawn>,
) {
    use rand::{thread_rng, Rng};
    const SPAWN_DISTANCE: isize = 2;
    for e in events.read().take(1) {
        let mut rng = thread_rng();

        match e {
            SpawnNewEvent::Normal => {
                let color = Color::rgb(0.1, 0.1, 0.1);
                loop {
                    if tiles.is_full() {
                        break;
                    }
                    let x = rng.gen_range(0..TILE_NUM_X);
                    let y = rng.gen_range(0..TILE_NUM_Y);
                    let dx = x as isize - last_spawn.0.x as isize;
                    let dy = y as isize - last_spawn.0.y as isize;
                    let dx = dx.abs().min(SPAWN_DISTANCE) * dx.signum();
                    let dy = dy.abs().min(SPAWN_DISTANCE) * dy.signum();
                    // Limit the distance of new spawned tiles from the last spawned tile
                    let x = (last_spawn.0.x as usize)
                        .saturating_add_signed(dx)
                        .min(TILE_NUM_X - 1);
                    let y = (last_spawn.0.y as usize)
                        .saturating_add_signed(dy)
                        .min(TILE_NUM_Y - 1);
                    let pos = UVec2::new(x as u32, y as u32);
                    if pos == last_spawn.0 {
                        continue;
                    }
                    if tiles[[x, y]].is_none() {
                        let entity = tile::<OnSessionScreen>(&mut commands, pos.extend(1), color);
                        last_spawn.0 = pos;
                        tiles.set(x, y, entity);
                        timer.0.reset();
                        break;
                    }
                }
            }
            SpawnNewEvent::Error((x, y)) => {
                let color = Color::rgb(0.9, 0.1, 0.1);
                tile::<OnSessionScreen>(&mut commands, UVec3::new(*x, *y, 2), color);
            }
        }
    }
    events.clear();
}

/// Update the timer of the tiles on the grid to reduce points.
/// Using [`Time<Virtual>`] makes the tiles lose points faster as the game progresses.
fn update_tile_points(time: Res<Time<Virtual>>, mut tiles: ResMut<GameGrid>) {
    tiles.tick(time.delta());
}

/// Handle click events
fn click(
    mut commands: Commands,
    mut clicks: EventReader<ClickEvent>,
    mut tiles: ResMut<GameGrid>,
    mut finished: EventWriter<FinishedEvent>,
    mut score: ResMut<Score>,
    mut new_tile: EventWriter<SpawnNewEvent>,
    mut sound: EventWriter<SoundEvent>,
) {
    for event in clicks.read() {
        let x = (event.tile_x as usize).min(TILE_NUM_X - 1);
        let y = (event.tile_y as usize).min(TILE_NUM_Y - 1);
        if let Some((entity, s)) = tiles.take(x, y) {
            commands.entity(entity).despawn_recursive();
            score.0 += s;
            if tiles.filled_tiles() == 0 {
                new_tile.send(SpawnNewEvent::Normal);
            }
            sound.send(SoundEvent::Normal);
        } else {
            new_tile.send(SpawnNewEvent::Error((x as u32, y as u32)));
            finished.send(FinishedEvent::Lost);
            sound.send(SoundEvent::Error);
        }
    }
}

/// Update score and time text in UI
fn update_score(mut q: Query<&mut Text, With<ScoreText>>, score: Res<Score>, time: Res<GameTime>) {
    use std::fmt::Write;
    let Some(mut text) = q.iter_mut().next() else {
        return;
    };
    text.sections[1].value.clear();
    write!(&mut text.sections[1].value, "{}", score.0).unwrap();
    text.sections[3].value.clear();
    write!(
        &mut text.sections[3].value,
        "{:.1}",
        (GAME_DURATION - time.0.elapsed_secs()).max(0.0)
    )
    .unwrap();
}

/// Play a sound
fn play_sound(mut commands: Commands, assets: Res<Assets>, mut events: EventReader<SoundEvent>) {
    for sound in events.read() {
        let audio = match sound {
            SoundEvent::Normal => assets.hit.clone(),
            SoundEvent::Error => assets.error.clone(),
        };
        commands.spawn(AudioBundle {
            source: audio,
            settings: PlaybackSettings::DESPAWN,
        });
        info!("Playing sound");
    }
}

/// Check if the game has finished and transition to the next state
fn check_finished(
    mut events: EventReader<FinishedEvent>,
    mut state: ResMut<NextState<RunningState>>,
) {
    for event in events.read() {
        match event {
            FinishedEvent::Lost => (),
            FinishedEvent::Finished => (),
        }
        state.set(RunningState::Finished);
    }
}

/// Update timers and stopwatches and modify virtual time relative speed.
/// The virtual time relative speed increases as the game progresses.
/// This is a linear course over the duration of the game session.
fn update_game_time(
    mut stopwatch: ResMut<GameTime>,
    mut spawn_time: ResMut<SpawnTimer>,
    real_time: Res<Time<Real>>,
    mut time: ResMut<Time<Virtual>>,
    mut events: EventWriter<FinishedEvent>,
) {
    stopwatch.0.tick(real_time.delta());
    spawn_time.0.tick(time.delta());
    time.set_relative_speed(1.0 + stopwatch.0.elapsed_secs() / GAME_DURATION);
    if stopwatch.0.elapsed_secs() > GAME_DURATION {
        info!("Time {} elapsed, finished", stopwatch.0.elapsed_secs());
        events.send(FinishedEvent::Finished);
    }
}

/// Handle the button interactions of the menu.
/// Possible options are defined inside [`Button`].
#[allow(clippy::type_complexity)]
fn button_system(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor, &Button),
        (Changed<Interaction>, With<Button>),
    >,
    mut game_state: ResMut<NextState<GameState>>,
    mut running_state: ResMut<NextState<RunningState>>,
    mut delay: ResMut<MenuActiveDelay>,
    time: ResMut<Time<Real>>,
) {
    delay.0.tick(time.delta());
    if !delay.0.finished() {
        return;
    }
    for (interaction, mut color, button) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *color = PRESSED_BUTTON.into();
                info!("Entry selected: {:?}", button);
                match button {
                    Button::Restart => {
                        running_state.set(RunningState::Running);
                    }
                    Button::Menu => game_state.set(GameState::Menu),
                }
            }
            Interaction::Hovered => {
                *color = HOVERED_BUTTON.into();
            }
            Interaction::None => {
                *color = NORMAL_BUTTON.into();
            }
        }
    }
}
