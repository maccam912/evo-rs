use bevy::prelude::*;
use bevy::input::mouse::{MouseWheel, MouseMotion};
use bevy::window::PrimaryWindow;

#[derive(Component)]
pub struct MainCamera;

#[derive(Resource)]
pub struct CameraState {
    pub zoom: f32,
    pub position: Vec2,
    pub is_panning: bool,
}

impl Default for CameraState {
    fn default() -> Self {
        Self {
            zoom: 1.0,
            position: Vec2::ZERO,
            is_panning: false,
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
