use bevy::prelude::*;

use crate::despawn_on_screen;
use crate::game::{Assets, GameState, OnGameScreen, OnSessionScreen, RunningState};

pub const NORMAL_BUTTON: Color = Color::rgb(0.15, 0.15, 0.15);
pub const HOVERED_BUTTON: Color = Color::rgb(0.25, 0.25, 0.25);
pub const PRESSED_BUTTON: Color = Color::rgb(0.35, 0.75, 0.35);

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Menu), setup_main_menu)
            .add_systems(
                Update,
                main_menu_button_system.run_if(in_state(GameState::Menu)),
            )
            .add_systems(OnExit(GameState::Menu), despawn_on_screen::<OnMainMenu>)
            .add_systems(OnEnter(RunningState::Finished), setup_game_menu)
            .add_systems(
                Update,
                game_menu_button_system.run_if(in_state(RunningState::Finished)),
            );
    }
}

/// Tag for indicating entities wich belong the the main menu screen
#[derive(Debug, Component)]
pub struct OnMainMenu;

/// Style used in buttons
macro_rules! BUTTON_STYLE {
    () => {
        Style {
            width: Val::Percent(50.0),
            height: Val::Percent(15.0),
            margin: UiRect::new(Val::Auto, Val::Auto, Val::Px(10.0), Val::Px(10.0)),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            border: UiRect::all(Val::Px(2.0)),
            ..Default::default()
        }
    };
}

/// Buttons in the menu
#[derive(Debug, Clone, Copy, Component)]
enum MainMenuButton {
    Start,
}

impl MainMenuButton {
    /// All buttons to be displayed in the menu
    const ALL: &'static [Self] = &[Self::Start];
}

/// Timer to disable first 0.8s of menu interaction
#[derive(Debug, Resource)]
struct MenuActiveDelay(Timer);

/// Kinds of buttons in menu
#[derive(Debug, Clone, Copy, PartialEq, Eq, Component)]
enum GameMenuButton {
    /// Go back to "main menu"
    Menu,
    /// Restart the game
    Restart,
}

impl GameMenuButton {
    /// All buttons for the menu
    const ALL: &'static [Self] = &[Self::Menu, Self::Restart];
}

/// Create the menu
fn setup_main_menu(mut commands: Commands, assets: Res<Assets>) {
    commands.spawn(Camera2dBundle::default()).insert(OnMainMenu);

    let menu_node = commands
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
        .insert(OnMainMenu)
        .id();
    for button in MainMenuButton::ALL.iter() {
        let button_frame = commands
            .spawn(ButtonBundle {
                style: BUTTON_STYLE!(),
                background_color: Color::rgb(0.2, 0.2, 0.2).into(),
                border_color: Color::rgb(0.5, 0.2, 0.2).into(),
                ..default()
            })
            .insert(*button)
            .set_parent(menu_node)
            .id();
        commands
            .spawn(TextBundle::from_section(
                format!("{:?}", button),
                TextStyle {
                    font: assets.font.clone(),
                    font_size: 40.0,
                    color: Color::rgb(0.9, 0.9, 0.9),
                },
            ))
            .set_parent(button_frame);
    }
}

/// Setup a menu
fn setup_game_menu(mut commands: Commands, assets: Res<Assets>) {
    // Prevent accidental clicking on menu item just after the game has ended
    commands.insert_resource(MenuActiveDelay(Timer::from_seconds(0.8, TimerMode::Once)));
    let ui_node = commands
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
        .id();
    for button in GameMenuButton::ALL.iter() {
        let button_frame = commands
            .spawn(ButtonBundle {
                style: BUTTON_STYLE!(),
                background_color: Color::rgb(0.2, 0.2, 0.2).into(),
                border_color: Color::rgb(0.5, 0.2, 0.2).into(),
                ..default()
            })
            .insert(*button)
            .set_parent(ui_node)
            .id();
        commands
            .spawn(TextBundle::from_section(
                format!("{:?}", button),
                TextStyle {
                    font: assets.font.clone(),
                    font_size: 40.0,
                    color: Color::rgb(0.9, 0.9, 0.9),
                },
            ))
            .set_parent(button_frame);
    }
}

/// Handle the menu buttons
#[allow(clippy::type_complexity)]
fn main_menu_button_system(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor, &MainMenuButton),
        Changed<Interaction>,
    >,
    mut app_state: ResMut<NextState<GameState>>,
) {
    for (interaction, mut color, button) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *color = PRESSED_BUTTON.into();
                info!("Entry selected: {:?}", button);
                match button {
                    MainMenuButton::Start => app_state.set(GameState::Game),
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

/// Handle the button interactions of the menu.
/// Possible options are defined inside [`GameMenuButton`].
fn game_menu_button_system(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor, &GameMenuButton),
        Changed<Interaction>,
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
                    GameMenuButton::Restart => {
                        running_state.set(RunningState::Running);
                    }
                    GameMenuButton::Menu => game_state.set(GameState::Menu),
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
