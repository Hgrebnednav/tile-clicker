use bevy::input::touch::TouchPhase;
use bevy::{prelude::*, window::PrimaryWindow};

pub struct InputPlugin<S> {
    state: S,
    size: UVec2,
    field: (Vec2, Vec2),
}

impl<S> InputPlugin<S> {
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
            );
    }
}

#[derive(Debug, Resource)]
struct Grid {
    size: UVec2,
    field: (Vec2, Vec2),
}

#[derive(Debug, Event)]
pub struct ClickEvent {
    pub tile_x: u16,
    pub tile_y: u16,
}

fn to_tile_pos(grid: &Grid, world_pos: Vec2) -> Option<ClickEvent> {
    let field_width = grid.field.1.x - grid.field.0.x;
    let field_height = grid.field.1.y - grid.field.0.y;
    let field_size = Vec2::new(field_width, field_height);
    let tile_size = field_size / grid.size.as_vec2();
    let top_offset = grid.field.0.y;

    // Shift position up such that all coordinates are positive and
    // the top region is negative
    let shifted = world_pos + field_size / 2.0 - Vec2::new(0.0, top_offset / 2.0);
    let scaled = dbg!(shifted / tile_size);
    if scaled.y >= grid.size.y as f32 || scaled.y < 0.0 {
        return None;
    }
    Some(ClickEvent {
        tile_x: scaled.x.trunc() as u16,
        tile_y: scaled.y.trunc() as u16,
    })
}

fn handle_click_input(
    mouse_btn: Res<Input<MouseButton>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    q_camera: Query<(&Camera, &GlobalTransform)>,
    mut event: EventWriter<ClickEvent>,
    settings: Res<Grid>,
) {
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

        if let Some(tile) = to_tile_pos(&settings, dbg!(world_pos)) {
            event.send(tile);
        }
    }
}

fn handle_touch_input(
    mut touches: EventReader<TouchInput>,
    windows: Query<&Window, With<PrimaryWindow>>,
    q_camera: Query<(&Camera, &GlobalTransform)>,
    mut click_event: EventWriter<ClickEvent>,
    settings: Res<Grid>,
) {
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
