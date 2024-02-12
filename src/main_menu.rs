use bevy::prelude::*;

use crate::despawn_screen;
use crate::game::GameState;

pub const NORMAL_BUTTON: Color = Color::rgb(0.15, 0.15, 0.15);
pub const HOVERED_BUTTON: Color = Color::rgb(0.25, 0.25, 0.25);
pub const PRESSED_BUTTON: Color = Color::rgb(0.35, 0.75, 0.35);

pub struct MainMenuPlugin;

impl Plugin for MainMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Menu), setup_main_menu)
            .add_systems(Update, button_system.run_if(in_state(GameState::Menu)))
            .add_systems(OnExit(GameState::Menu), despawn_screen::<OnMainMenu>);
    }
}

#[derive(Debug, Component)]
pub struct OnMainMenu;

#[derive(Debug, Clone, Copy, Component)]
enum Button {
    Start,
}

impl Button {
    const ALL: &'static [Self] = &[Button::Start];
}

fn setup_main_menu(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle::default()).insert(OnMainMenu);

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
        .insert(OnMainMenu)
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
                    .insert(OnMainMenu)
                    .with_children(|parent| {
                        parent
                            .spawn(TextBundle::from_section(
                                format!("{:?}", button),
                                TextStyle {
                                    font: asset_server.load("fonts/EBGaramond-Regular.ttf"),
                                    font_size: 40.0,
                                    color: Color::rgb(0.9, 0.9, 0.9),
                                },
                            ))
                            .insert(OnMainMenu);
                    });
            }
        });
}

#[allow(clippy::type_complexity)]
fn button_system(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor, &Button),
        (Changed<Interaction>, With<Button>),
    >,
    mut app_state: ResMut<NextState<GameState>>,
) {
    for (interaction, mut color, button) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *color = PRESSED_BUTTON.into();
                info!("Entry selected: {:?}", button);
                match button {
                    Button::Start => app_state.set(GameState::Game),
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
