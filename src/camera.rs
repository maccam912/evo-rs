use bevy::input::mouse::{MouseMotion, MouseWheel};
use bevy::input::touch::Touches;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

#[derive(Component)]
pub struct MainCamera;

#[derive(Resource)]
pub struct CameraState {
    pub zoom: f32,
    pub position: Vec2,
    pub is_panning: bool,
    pub primary_touch_id: Option<u64>,
    pub secondary_touch_id: Option<u64>,
    pub last_pinch_distance: Option<f32>,
}

impl Default for CameraState {
    fn default() -> Self {
        Self {
            zoom: 1.0,
            position: Vec2::ZERO,
            is_panning: false,
            primary_touch_id: None,
            secondary_touch_id: None,
            last_pinch_distance: None,
        }
    }
}

pub fn setup_camera(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        MainCamera,
        Transform::from_xyz(0.0, 0.0, 0.0),
        OrthographicProjection {
            scale: 1.0,
            ..OrthographicProjection::default_2d()
        },
    ));
}

pub fn camera_zoom(
    mut scroll_events: EventReader<MouseWheel>,
    mut camera_state: ResMut<CameraState>,
    mut query: Query<&mut OrthographicProjection, With<MainCamera>>,
) {
    for event in scroll_events.read() {
        // Zoom in/out with mouse wheel
        let zoom_delta = -event.y * 0.1;
        camera_state.zoom = (camera_state.zoom + zoom_delta).clamp(0.1, 10.0);

        if let Ok(mut projection) = query.get_single_mut() {
            projection.scale = camera_state.zoom;
        }
    }
}

pub fn camera_pan(
    mouse_button: Res<ButtonInput<MouseButton>>,
    mut motion_events: EventReader<MouseMotion>,
    mut camera_state: ResMut<CameraState>,
    mut query: Query<&mut Transform, With<MainCamera>>,
    _window_query: Query<&Window, With<PrimaryWindow>>,
) {
    // Check if middle mouse button is pressed
    if mouse_button.just_pressed(MouseButton::Middle) {
        camera_state.is_panning = true;
    }
    if mouse_button.just_released(MouseButton::Middle) {
        camera_state.is_panning = false;
    }

    if camera_state.is_panning {
        for event in motion_events.read() {
            if let Ok(mut transform) = query.get_single_mut() {
                // Pan the camera - invert Y because screen coords go down but world goes up
                let pan_delta = Vec2::new(-event.delta.x, event.delta.y) * camera_state.zoom;
                camera_state.position += pan_delta;
                transform.translation.x = camera_state.position.x;
                transform.translation.y = camera_state.position.y;
            }
        }
    }
}

pub fn camera_touch_controls(
    touches: Res<Touches>,
    mut camera_state: ResMut<CameraState>,
    mut camera_query: Query<(&mut Transform, &mut OrthographicProjection), With<MainCamera>>,
) {
    // Track the active touches so that the same finger continues to control the camera.
    for touch in touches.iter_just_pressed() {
        let id = touch.id();
        if camera_state.primary_touch_id.is_none() {
            camera_state.primary_touch_id = Some(id);
        } else if camera_state.secondary_touch_id.is_none()
            && camera_state.primary_touch_id != Some(id)
        {
            camera_state.secondary_touch_id = Some(id);
            camera_state.last_pinch_distance = None;
        }
    }

    // Clear touch tracking when fingers lift or get cancelled.
    for touch in touches.iter_just_released() {
        clear_touch(&mut camera_state, touch.id());
    }
    for touch in touches.iter_just_canceled() {
        clear_touch(&mut camera_state, touch.id());
    }

    if let Ok((mut transform, mut projection)) = camera_query.get_single_mut() {
        // Handle pinch zoom when two touches are active.
        if let (Some(primary_id), Some(secondary_id)) = (
            camera_state.primary_touch_id,
            camera_state.secondary_touch_id,
        ) {
            if let (Some(primary_touch), Some(secondary_touch)) = (
                touches.get_pressed(primary_id),
                touches.get_pressed(secondary_id),
            ) {
                let current_distance = primary_touch
                    .position()
                    .distance(secondary_touch.position());

                if let Some(previous_distance) = camera_state.last_pinch_distance {
                    let distance_delta = current_distance - previous_distance;
                    if distance_delta.abs() > f32::EPSILON {
                        camera_state.zoom =
                            (camera_state.zoom - distance_delta * 0.003).clamp(0.1, 10.0);
                        projection.scale = camera_state.zoom;
                    }
                }

                camera_state.last_pinch_distance = Some(current_distance);
            }
        } else {
            camera_state.last_pinch_distance = None;
        }

        // Handle swipe panning when a single touch is active.
        if camera_state.secondary_touch_id.is_none() {
            if let Some(primary_id) = camera_state.primary_touch_id {
                if let Some(primary_touch) = touches.get_pressed(primary_id) {
                    let delta = primary_touch.delta();
                    if delta.length_squared() > 0.0 {
                        let pan_delta = Vec2::new(-delta.x, delta.y) * camera_state.zoom;
                        camera_state.position += pan_delta;
                        transform.translation.x = camera_state.position.x;
                        transform.translation.y = camera_state.position.y;
                    }
                }
            }
        }
    }
}

fn clear_touch(camera_state: &mut CameraState, id: u64) {
    if camera_state.primary_touch_id == Some(id) {
        camera_state.primary_touch_id = camera_state.secondary_touch_id;
        camera_state.secondary_touch_id = None;
        camera_state.last_pinch_distance = None;
    } else if camera_state.secondary_touch_id == Some(id) {
        camera_state.secondary_touch_id = None;
        camera_state.last_pinch_distance = None;
    }

    if camera_state.primary_touch_id.is_none() {
        camera_state.last_pinch_distance = None;
    }
}
