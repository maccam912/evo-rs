use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use crate::config::*;

/// Marker component for the currently selected entity
#[derive(Component)]
pub struct Selected;

/// Resource to track the currently selected entity
#[derive(Resource, Default)]
pub struct SelectedEntity {
    pub entity: Option<Entity>,
}

/// System to handle entity selection via mouse clicks
pub fn handle_selection(
    mouse_button: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    mut selected_entity: ResMut<SelectedEntity>,
    mut commands: Commands,
    // Query all entities that can be selected (have Transform and any selectable component)
    selectable_query: Query<(Entity, &Transform), Or<(With<crate::plant::Plant>, With<crate::animal::Animal>)>>,
    // Query entities that are currently selected
    currently_selected: Query<Entity, With<Selected>>,
) {
    // Only process on left mouse button click
    if !mouse_button.just_pressed(MouseButton::Left) {
        return;
    }

    let window = windows.single();
    let (camera, camera_transform) = camera_query.single();

    // Get cursor position
    if let Some(cursor_pos) = window.cursor_position() {
        // Convert screen coordinates to world coordinates
        if let Ok(world_pos) = camera.viewport_to_world_2d(camera_transform, cursor_pos) {
            // Find the entity closest to the click position
            let mut closest_entity: Option<(Entity, f32)> = None;

            for (entity, transform) in selectable_query.iter() {
                let entity_pos = Vec2::new(transform.translation.x, transform.translation.y);
                let distance = world_pos.distance(entity_pos);

                if distance <= SELECTION_RADIUS {
                    match closest_entity {
                        None => closest_entity = Some((entity, distance)),
                        Some((_, closest_dist)) if distance < closest_dist => {
                            closest_entity = Some((entity, distance));
                        }
                        _ => {}
                    }
                }
            }

            // Clear previous selection
            for entity in currently_selected.iter() {
                commands.entity(entity).remove::<Selected>();
            }

            // Set new selection
            if let Some((entity, _)) = closest_entity {
                commands.entity(entity).insert(Selected);
                selected_entity.entity = Some(entity);
            } else {
                selected_entity.entity = None;
            }
        }
    }
}

/// System to add visual indicator to selected entities
pub fn update_selection_visuals(
    _selected_query: Query<&Sprite, (With<Selected>, Changed<Selected>)>,
) {
    // Visual indicators are now handled by the outline system
    // This system is kept as a placeholder for future enhancements
}
