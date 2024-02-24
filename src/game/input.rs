use bevy::input::touch::TouchPhase;
use bevy::{prelude::*, window::PrimaryWindow};

/// Plugin which convert input clicks to a tile clicked event
pub struct InputPlugin<S> {
    state: S,
    size: UVec2,
    field: (Vec2, Vec2),
}

impl<S> InputPlugin<S> {
    /// Create a new instance of the plugin with the region where the tiles are located.
    pub fn new(state: S, size: UVec2, top_left: Vec2, bottom_right: Vec2) -> Self {
        Self {
            state,
            size,
            field: (top_left, bottom_right),
        }
    }
}

impl<S> Plugin for InputPlugin<S>
where
    S: Copy + Send + Sync + States + 'static,
{
    fn build(&self, app: &mut App) {
        app.add_event::<ClickEvent>()
            .insert_resource(Grid {
                size: self.size,
                field: self.field,
            })
            .add_systems(
                Update,
                (handle_click_input, handle_touch_input).run_if(in_state(self.state)),
            )
            .add_systems(PreUpdate, update_time.run_if(in_state(self.state)))
            .add_systems(OnEnter(self.state), setup);
    }
}

/// The grid settings
#[derive(Debug, Resource)]
struct Grid {
    size: UVec2,
    field: (Vec2, Vec2),
}

/// Event with clicked tile positions
#[derive(Debug, Event)]
pub struct ClickEvent {
    pub tile_x: usize,
    pub tile_y: usize,
}

/// Delay to prevent click events being send for 0.4s after the game stars
#[derive(Debug, Resource)]
struct ClickDelay(Timer);

/// Setup plugin resources
fn setup(mut commands: Commands) {
    commands.insert_resource(ClickDelay(Timer::from_seconds(0.4, TimerMode::Once)));
}

/// Update time of the [`ClickDelay`] resource
fn update_time(time: Res<Time<Real>>, mut delay: ResMut<ClickDelay>) {
    delay.0.tick(time.delta());
}

/// Convert a world position to an [`ClickEvent`] if the click is inside the region
fn to_tile_pos(grid: &Grid, world_pos: Vec2) -> Option<ClickEvent> {
    let field_width = grid.field.1.x - grid.field.0.x;
    let field_height = grid.field.1.y - grid.field.0.y;
    let field_size = Vec2::new(field_width, field_height);
    let tile_size = field_size / grid.size.as_vec2();
    let top_offset = grid.field.0.y;

    // Shift position up such that all coordinates are positive and
    // the top region is negative
    let shifted = world_pos + field_size / 2.0 - Vec2::new(0.0, top_offset / 2.0);
    let scaled = shifted / tile_size;
    if scaled.y >= grid.size.y as f32 || scaled.y < 0.0 {
        return None;
    }
    Some(ClickEvent {
        tile_x: scaled.x.trunc() as usize,
        tile_y: scaled.y.trunc() as usize,
    })
}

/// Handle mouse clicks
/// Convert screen position to a tile position
fn handle_click_input(
    mouse_btn: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    q_camera: Query<(&Camera, &GlobalTransform)>,
    mut event: EventWriter<ClickEvent>,
    settings: Res<Grid>,
    start_delay: Res<ClickDelay>,
) {
    if !start_delay.0.finished() {
        return;
    }
    let Ok(win) = windows.get_single() else {
        panic!("No primary window found");
    };
    if mouse_btn.just_pressed(MouseButton::Left) {
        let Some(pos) = win.cursor_position() else {
            return;
        };
        let (camera, camera_transform) = q_camera.single();

        let width = win.width();
        let height = win.height();

        let window_size = Vec2::new(width, height);

        // convert screen position [0..resolution] to ndc [-1..1] (gpu coordinates)
        let ndc = (pos / window_size) * 2.0 - Vec2::ONE;
        // Flip vertical
        let ndc = Vec2::new(1.0, -1.0) * ndc;

        // matrix for undoing the projection and camera transform
        let ndc_to_world = camera_transform.compute_matrix() * camera.projection_matrix().inverse();

        // use it to convert ndc to world-space coordinates
        let world_pos = ndc_to_world.project_point3(ndc.extend(-1.0));

        // Flip Y axis and reduce to 2d vector
        let world_pos = world_pos.truncate() * Vec2::new(1.0, -1.0);

        if let Some(tile) = to_tile_pos(&settings, world_pos) {
            event.send(tile);
        }
    }
}

/// Handle touch inputs
/// Convert screen position to a tile position
/// TODO: Does this work? It does not work in WASM
fn handle_touch_input(
    mut touches: EventReader<TouchInput>,
    windows: Query<&Window, With<PrimaryWindow>>,
    q_camera: Query<(&Camera, &GlobalTransform)>,
    mut click_event: EventWriter<ClickEvent>,
    settings: Res<Grid>,
    start_delay: Res<ClickDelay>,
) {
    if !start_delay.0.finished() {
        return;
    }
    let Ok(win) = windows.get_single() else {
        panic!("No primary window found");
    };
    for event in touches.read() {
        let TouchInput {
            phase, position, ..
        } = event;
        info!("Touch registered");
        match phase {
            TouchPhase::Ended => (),
            _ => return,
        }

        let (camera, camera_transform) = q_camera.single();

        let width = win.width();
        let height = win.height();

        let window_size = Vec2::new(width, height);

        // convert screen position [0..resolution] to ndc [-1..1] (gpu coordinates)
        let ndc = (*position / window_size) * 2.0 - Vec2::ONE;
        // Flip vertical
        let ndc = Vec2::new(1.0, -1.0) * ndc;

        // matrix for undoing the projection and camera transform
        let ndc_to_world = camera_transform.compute_matrix() * camera.projection_matrix().inverse();

        // use it to convert ndc to world-space coordinates
        let world_pos = ndc_to_world.project_point3(ndc.extend(-1.0));

        // Flip Y axis and reduce to 2d vector
        let world_pos = world_pos.truncate() * Vec2::new(1.0, -1.0);

        if let Some(tile) = to_tile_pos(&settings, dbg!(world_pos)) {
            click_event.send(tile);
        }
    }
}
