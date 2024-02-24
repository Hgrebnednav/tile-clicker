use bevy::prelude::*;

use super::GameState;
use crate::despawn_on_screen;

/// Plugin for loading assets
pub struct LoadingPlugin;

impl Plugin for LoadingPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<AssetsLoading>()
            .add_systems(OnEnter(GameState::Loading), splash_setup)
            .add_systems(
                Update,
                check_assets_ready.run_if(in_state(GameState::Loading)),
            )
            .add_systems(
                OnExit(GameState::Loading),
                despawn_on_screen::<OnSplashScreen>,
            );
    }
}

/// Tag for marking entities belonging to the splash screen
/// TODO: add logo/icon during startup to not have this being useless
#[derive(Component)]
struct OnSplashScreen;

/// List of assets to track the loading state
#[derive(Debug, Default, Resource)]
pub struct AssetsLoading(pub Vec<UntypedHandle>);

/// Timer to specify minimum splash screen display time
#[derive(Resource)]
struct SplashTimer(Timer);

/// All assets used in the game.
/// Keep assets loaded to not reload them during runtime
#[derive(Resource)]
pub struct Assets {
    pub font: Handle<Font>,
    pub hit: Handle<AudioSource>,
    pub error: Handle<AudioSource>,
}

/// Setup splash screen and start loading assets
fn splash_setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut loading: ResMut<AssetsLoading>,
) {
    //let icon = asset_server.load("bevy_icon.png");

    //commands
    //    .spawn((
    //        NodeBundle {
    //            style: Style {
    //                align_items: AlignItems::Center,
    //                justify_content: JustifyContent::Center,
    //                width: Val::Percent(100.0),
    //                height: Val::Percent(100.0),
    //                ..default()
    //            },
    //            ..default()
    //        },
    //        OnSplashScreen,
    //    ))
    //    .with_children(|parent| {
    //        parent.spawn(ImageBundle {
    //            style: Style {
    //                // This will set the logo to be 200px wide, and auto adjust its height
    //                width: Val::Px(200.0),
    //                ..default()
    //            },
    //            image: UiImage::new(icon),
    //            ..default()
    //        });
    //    });
    // Load assets
    let font = asset_server.load("fonts/EBGaramond-Regular.ttf");
    loading.0.push(font.clone().untyped());
    let hit = asset_server.load("test.wav");
    loading.0.push(hit.clone().untyped());
    let error = asset_server.load("test2.wav");
    loading.0.push(error.clone().untyped());
    let assets = Assets { font, hit, error };
    commands.insert_resource(assets);
    commands.insert_resource(SplashTimer(Timer::from_seconds(1.0, TimerMode::Once)));
}

/// Check when the assets are ready and transition state
fn check_assets_ready(
    mut commands: Commands,
    server: Res<AssetServer>,
    loading: Res<AssetsLoading>,
    mut game_state: ResMut<NextState<GameState>>,
    time: Res<Time>,
    mut timer: ResMut<SplashTimer>,
) {
    use bevy::asset::LoadState;

    let mut ready_count = 0;
    for handle in loading.0.iter() {
        match server.load_state(handle.id()) {
            LoadState::Failed => {
                println!("Failed loading asset {:?}", server.get_path(handle.id()));
            }
            LoadState::Loaded => {
                ready_count += 1;
            }
            _ => {
                // Item still loading
            }
        }
    }
    timer.0.tick(time.delta());
    if ready_count == loading.0.len() && timer.0.finished() {
        info!("Finished loading");
        commands.remove_resource::<AssetsLoading>();
        commands.remove_resource::<SplashTimer>();
        game_state.set(GameState::Menu);
    }
}
