use crate::config::*;
use bevy::prelude::*;
use rand::Rng;

/// Plant component that stores energy
#[derive(Component)]
pub struct Plant {
    pub energy: u32,
}

/// Marker component indicating this entity emits plant scent
#[derive(Component)]
pub struct PlantScent;

impl Plant {
    pub const MAX_ENERGY: u32 = PLANT_MAX_ENERGY;

    pub fn new() -> Self {
        Self { energy: 0 }
    }

    pub fn add_energy(&mut self, amount: u32) {
        self.energy = (self.energy + amount).min(Self::MAX_ENERGY);
    }

    pub fn consume_energy(&mut self, amount: u32) {
        self.energy = self.energy.saturating_sub(amount);
    }
}

/// Configuration resource for plant spawning and growth
#[derive(Resource)]
pub struct PlantConfig {
    pub world_bounds: f32,
}

impl Default for PlantConfig {
    fn default() -> Self {
        Self {
            world_bounds: WORLD_BOUNDS,
        }
    }
}

/// Timer resource for plant spawning
#[derive(Resource)]
pub struct PlantSpawnTimer(pub Timer);

/// Timer resource for plant growth
#[derive(Resource)]
pub struct PlantGrowthTimer(pub Timer);

/// System to spawn new plants at regular intervals
pub fn spawn_plants(
    time: Res<Time>,
    mut timer: ResMut<PlantSpawnTimer>,
    config: Res<PlantConfig>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    if timer.0.tick(time.delta()).just_finished() {
        let mut rng = rand::thread_rng();

        // Random position within world bounds
        let x = rng.gen_range(-config.world_bounds..config.world_bounds);
        let y = rng.gen_range(-config.world_bounds..config.world_bounds);

        // Spawn plant entity
        commands.spawn((
            Plant::new(),
            PlantScent,
            Mesh2d(meshes.add(Circle::new(8.0))),
            MeshMaterial2d(materials.add(ColorMaterial::from_color(Color::srgb(0.2, 0.8, 0.2)))),
            Transform::from_xyz(x, y, 0.0),
        ));
    }
}

/// System to grow existing plants (increment energy)
pub fn grow_plants(
    time: Res<Time>,
    mut timer: ResMut<PlantGrowthTimer>,
    mut plants: Query<&mut Plant>,
) {
    if timer.0.tick(time.delta()).just_finished() {
        for mut plant in plants.iter_mut() {
            plant.add_energy(PLANT_GROWTH_AMOUNT);
        }
    }
}

/// System to update plant visual representation based on energy
pub fn update_plant_visuals(mut plants: Query<(&Plant, &mut Transform), Changed<Plant>>) {
    for (plant, mut transform) in plants.iter_mut() {
        // Scale plant based on energy (0-100 maps to 0.5-1.5 scale)
        let scale = 0.5 + (plant.energy as f32 / Plant::MAX_ENERGY as f32) * 1.0;
        transform.scale = Vec3::splat(scale);
    }
}
