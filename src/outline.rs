use bevy::prelude::*;
use crate::selection::Selected;

/// Component that marks an outline entity linked to a selected entity
#[derive(Component)]
pub struct SelectionOutline {
    pub parent: Entity,
}

/// System to add/remove outlines for selected entities
pub fn manage_selection_outlines(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    // Newly selected entities
    added_selection: Query<(Entity, &Transform), Added<Selected>>,
    // Entities that lost selection
    mut removed_selection: RemovedComponents<Selected>,
    // Existing outlines
    outlines: Query<(Entity, &SelectionOutline)>,
) {
    // Add outlines to newly selected entities
    for (entity, transform) in added_selection.iter() {
        // Spawn an outline circle slightly larger than the entity
        commands.spawn((
            SelectionOutline { parent: entity },
            Mesh2d(meshes.add(Circle::new(12.0))), // Slightly larger than plant (8.0)
            MeshMaterial2d(materials.add(ColorMaterial::from_color(
                Color::srgba(1.0, 1.0, 0.0, 0.6) // Yellow with transparency
            ))),
            Transform::from_xyz(transform.translation.x, transform.translation.y, -0.1),
        ));
    }

    // Remove outlines for deselected entities
    for removed_entity in removed_selection.read() {
        for (outline_entity, outline) in outlines.iter() {
            if outline.parent == removed_entity {
                commands.entity(outline_entity).despawn();
            }
        }
    }
}

/// System to update outline positions to follow their parent entities
pub fn update_outline_positions(
    selected_entities: Query<(Entity, &Transform), With<Selected>>,
    mut outlines: Query<(&SelectionOutline, &mut Transform), Without<Selected>>,
) {
    for (outline, mut outline_transform) in outlines.iter_mut() {
        if let Ok((_, parent_transform)) = selected_entities.get(outline.parent) {
            outline_transform.translation.x = parent_transform.translation.x;
            outline_transform.translation.y = parent_transform.translation.y;
            // Scale the outline to match the parent's scale
            outline_transform.scale = parent_transform.scale;
        }
    }
}
