use crate::config::*;
use crate::genome::{Genome, GenomeExecutor, Sensors, Word};
use crate::plant::{Plant, PlantScent};
use bevy::prelude::*;
use rand::Rng;

/// Animal component with energy and age
#[derive(Component)]
pub struct Animal {
    pub energy: u32,
    pub age: f32,
}

impl Animal {
    pub fn new(energy: u32) -> Self {
        Self { energy, age: 0.0 }
    }

    pub fn consume_energy(&mut self, amount: u32) {
        self.energy = self.energy.saturating_sub(amount);
    }

    pub fn add_energy(&mut self, amount: u32) {
        self.energy += amount;
    }
}

/// Timer for animal metabolism
#[derive(Resource)]
pub struct MetabolismTimer(pub Timer);

/// System to spawn initial test animals
pub fn spawn_test_animals(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    spawn_seed_animals(
        &mut commands,
        &mut meshes,
        &mut materials,
        INITIAL_ANIMAL_COUNT,
        STARTING_ANIMAL_ENERGY,
    );
}

/// Helper function to spawn animals with the deterministic seed genome
pub fn spawn_seed_animals(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    count: usize,
    energy: u32,
) {
    let mut rng = rand::thread_rng();

    for _ in 0..count {
        let x = rng.gen_range(-ANIMAL_SPAWN_RANGE..ANIMAL_SPAWN_RANGE);
        let y = rng.gen_range(-ANIMAL_SPAWN_RANGE..ANIMAL_SPAWN_RANGE);
        let rotation = rng.gen_range(0.0..std::f32::consts::TAU);

        commands.spawn((
            Animal::new(energy),
            Genome::seed(),
            GenomeExecutor::new(energy),
            Sensors::default(),
            Mesh2d(meshes.add(Circle::new(10.0))),
            MeshMaterial2d(materials.add(ColorMaterial::from_color(Color::srgb(0.9, 0.3, 0.2)))),
            Transform::from_xyz(x, y, 0.0).with_rotation(Quat::from_rotation_z(rotation)),
        ));
    }
}

/// System to update sensors for all animals (4 directional smell sensors)
pub fn update_sensors(
    mut animals: Query<(&Transform, &mut Sensors), With<Animal>>,
    plants: Query<&Transform, With<PlantScent>>,
) {
    for (animal_transform, mut sensors) in animals.iter_mut() {
        let animal_pos = animal_transform.translation.truncate();

        // Get animal's forward and right vectors
        let forward = (animal_transform.rotation * Vec3::Y).truncate();
        let right = (animal_transform.rotation * Vec3::X).truncate();

        // Initialize sensors
        sensors.smell_front = None;
        sensors.smell_back = None;
        sensors.smell_left = None;
        sensors.smell_right = None;

        // Check each plant and categorize by quadrant
        for plant_transform in plants.iter() {
            let plant_pos = plant_transform.translation.truncate();
            let to_plant = plant_pos - animal_pos;
            let distance = to_plant.length();

            // Determine which quadrant the plant is in
            let forward_dot = to_plant.dot(forward);
            let right_dot = to_plant.dot(right);

            // Use dot products to determine quadrant
            if forward_dot.abs() > right_dot.abs() {
                // Front or back
                if forward_dot > 0.0 {
                    // Front
                    sensors.smell_front = Some(match sensors.smell_front {
                        None => distance,
                        Some(d) => d.min(distance),
                    });
                } else {
                    // Back
                    sensors.smell_back = Some(match sensors.smell_back {
                        None => distance,
                        Some(d) => d.min(distance),
                    });
                }
            } else {
                // Left or right
                if right_dot > 0.0 {
                    // Right
                    sensors.smell_right = Some(match sensors.smell_right {
                        None => distance,
                        Some(d) => d.min(distance),
                    });
                } else {
                    // Left
                    sensors.smell_left = Some(match sensors.smell_left {
                        None => distance,
                        Some(d) => d.min(distance),
                    });
                }
            }
        }
    }
}

/// Marker component for animals that need to split
#[derive(Component)]
pub struct PendingSplit;

/// System to execute genome words (stack-based)
pub fn execute_genomes(
    mut commands: Commands,
    mut animals: Query<
        (
            Entity,
            &mut Animal,
            &Genome,
            &mut GenomeExecutor,
            &Sensors,
            &mut Transform,
        ),
        Without<PendingSplit>,
    >,
    mut plants: Query<(Entity, &mut Plant, &Transform), Without<Animal>>,
) {
    for (entity, mut animal, genome, mut executor, sensors, mut transform) in animals.iter_mut() {
        executor.reset_for_frame(animal.energy);
        executor.build_jump_table(genome);
        executor.build_label_table(genome); // Build label table for jumps

        let mut should_despawn = false;
        let mut should_split = false;

        // Circular execution: only stops when instruction budget runs out
        while executor.can_execute() {
            // Ensure IP is within bounds (wrap if necessary)
            if executor.instruction_pointer >= genome.words.len() {
                executor.instruction_pointer = 0;
            }

            let word = genome.words[executor.instruction_pointer];

            // Handle Split as a special case (requires energy check before execution)
            if matches!(word, Word::Split) {
                if animal.energy >= SPLIT_ENERGY_COST {
                    should_split = true;
                    executor.advance(genome.words.len());
                    break; // Stop execution this frame
                } else {
                    // Not enough energy, treat as Nop
                    executor.advance(genome.words.len());
                    continue;
                }
            }

            match execute_word(
                word,
                &mut executor,
                &mut animal,
                sensors,
                &mut transform,
                &mut plants,
                &mut commands,
            ) {
                Ok(ExecutionResult::Continue) => {
                    executor.advance(genome.words.len());
                }
                Ok(ExecutionResult::Jump(target)) => {
                    // Ensure target is within bounds (wrap if necessary)
                    executor.instruction_pointer = target % genome.words.len();
                    executor.instructions_executed_this_frame += 1;
                }
                Ok(ExecutionResult::Skip) => {
                    // Stack underflow or type mismatch - skip instruction
                    executor.advance(genome.words.len());
                }
                Err(_) => {
                    // Fatal error - kill the animal
                    should_despawn = true;
                    break;
                }
            }
        }

        // Apply deferred actions after iteration completes
        if should_despawn || animal.energy == 0 {
            // Either fatal error or out of energy - despawn
            commands.entity(entity).despawn();
        } else if should_split {
            // Only insert PendingSplit if entity is still alive
            commands.entity(entity).insert(PendingSplit);
        }
    }
}

/// Execution result for word execution
enum ExecutionResult {
    Continue,    // Continue to next word
    Jump(usize), // Jump to specific position (for control flow)
    Skip,        // Skip this word (stack error)
}

/// Execute a single word
fn execute_word(
    word: Word,
    executor: &mut GenomeExecutor,
    animal: &mut Animal,
    sensors: &Sensors,
    transform: &mut Transform,
    plants: &mut Query<(Entity, &mut Plant, &Transform), Without<Animal>>,
    commands: &mut Commands,
) -> Result<ExecutionResult, ()> {
    match word {
        // Stack Manipulation
        Word::Dup => {
            if let Some(&val) = executor.peek() {
                executor.stack.push(val);
                Ok(ExecutionResult::Continue)
            } else {
                Ok(ExecutionResult::Skip)
            }
        }
        Word::Drop => {
            executor.pop();
            Ok(ExecutionResult::Continue)
        }
        Word::Swap => {
            if let (Some(b), Some(a)) = (executor.pop(), executor.pop()) {
                executor.stack.push(b);
                executor.stack.push(a);
                Ok(ExecutionResult::Continue)
            } else {
                Ok(ExecutionResult::Skip)
            }
        }
        Word::Over => {
            if executor.stack.len() >= 2 {
                let val = executor.stack[executor.stack.len() - 2];
                executor.stack.push(val);
                Ok(ExecutionResult::Continue)
            } else {
                Ok(ExecutionResult::Skip)
            }
        }
        Word::Rot => {
            if executor.stack.len() >= 3 {
                let c = executor.pop().unwrap();
                let b = executor.pop().unwrap();
                let a = executor.pop().unwrap();
                executor.stack.push(b);
                executor.stack.push(c);
                executor.stack.push(a);
                Ok(ExecutionResult::Continue)
            } else {
                Ok(ExecutionResult::Skip)
            }
        }

        // Literals
        Word::PushFloat(val) => {
            executor.push_float(val);
            Ok(ExecutionResult::Continue)
        }
        Word::PushBool(val) => {
            executor.push_bool(val);
            Ok(ExecutionResult::Continue)
        }

        // Sensor Operations
        Word::SmellFront => {
            let distance = sensors.smell_front.unwrap_or(999999.0);
            executor.push_float(distance);
            Ok(ExecutionResult::Continue)
        }
        Word::SmellBack => {
            let distance = sensors.smell_back.unwrap_or(999999.0);
            executor.push_float(distance);
            Ok(ExecutionResult::Continue)
        }
        Word::SmellLeft => {
            let distance = sensors.smell_left.unwrap_or(999999.0);
            executor.push_float(distance);
            Ok(ExecutionResult::Continue)
        }
        Word::SmellRight => {
            let distance = sensors.smell_right.unwrap_or(999999.0);
            executor.push_float(distance);
            Ok(ExecutionResult::Continue)
        }
        Word::Energy => {
            executor.push_float(animal.energy as f32);
            Ok(ExecutionResult::Continue)
        }

        // Arithmetic Operations
        Word::Add => {
            if let (Some(b), Some(a)) = (executor.pop_float(), executor.pop_float()) {
                executor.push_float(a + b);
                Ok(ExecutionResult::Continue)
            } else {
                Ok(ExecutionResult::Skip)
            }
        }
        Word::Sub => {
            if let (Some(b), Some(a)) = (executor.pop_float(), executor.pop_float()) {
                executor.push_float(a - b);
                Ok(ExecutionResult::Continue)
            } else {
                Ok(ExecutionResult::Skip)
            }
        }
        Word::Mul => {
            if let (Some(b), Some(a)) = (executor.pop_float(), executor.pop_float()) {
                executor.push_float(a * b);
                Ok(ExecutionResult::Continue)
            } else {
                Ok(ExecutionResult::Skip)
            }
        }
        Word::Div => {
            if let (Some(b), Some(a)) = (executor.pop_float(), executor.pop_float()) {
                if b != 0.0 {
                    executor.push_float(a / b);
                } else {
                    executor.push_float(0.0); // Division by zero returns 0
                }
                Ok(ExecutionResult::Continue)
            } else {
                Ok(ExecutionResult::Skip)
            }
        }

        // Comparison Operations
        Word::Lt => {
            if let (Some(b), Some(a)) = (executor.pop_float(), executor.pop_float()) {
                executor.push_bool(a < b);
                Ok(ExecutionResult::Continue)
            } else {
                Ok(ExecutionResult::Skip)
            }
        }
        Word::Gt => {
            if let (Some(b), Some(a)) = (executor.pop_float(), executor.pop_float()) {
                executor.push_bool(a > b);
                Ok(ExecutionResult::Continue)
            } else {
                Ok(ExecutionResult::Skip)
            }
        }
        Word::Eq => {
            if let (Some(b), Some(a)) = (executor.pop_float(), executor.pop_float()) {
                executor.push_bool((a - b).abs() < 0.001); // Float equality with tolerance
                Ok(ExecutionResult::Continue)
            } else {
                Ok(ExecutionResult::Skip)
            }
        }

        // Logic Operations
        Word::And => {
            if let (Some(b), Some(a)) = (executor.pop_bool(), executor.pop_bool()) {
                executor.push_bool(a && b);
                Ok(ExecutionResult::Continue)
            } else {
                Ok(ExecutionResult::Skip)
            }
        }
        Word::Or => {
            if let (Some(b), Some(a)) = (executor.pop_bool(), executor.pop_bool()) {
                executor.push_bool(a || b);
                Ok(ExecutionResult::Continue)
            } else {
                Ok(ExecutionResult::Skip)
            }
        }
        Word::Not => {
            if let Some(a) = executor.pop_bool() {
                executor.push_bool(!a);
                Ok(ExecutionResult::Continue)
            } else {
                Ok(ExecutionResult::Skip)
            }
        }

        // Control Flow
        Word::If => {
            let condition = executor.pop_bool().unwrap_or(false);
            let current_pos = executor.instruction_pointer;

            // Find matching Then/Else in jump table
            if let Some((_, else_pos, then_pos)) = executor
                .jump_table
                .iter()
                .find(|(if_pos, _, _)| *if_pos == current_pos)
            {
                if !condition {
                    // Jump to else or then
                    if let Some(else_target) = else_pos {
                        Ok(ExecutionResult::Jump(*else_target + 1))
                    } else {
                        Ok(ExecutionResult::Jump(*then_pos + 1))
                    }
                } else {
                    // Continue to next word (execute if branch)
                    Ok(ExecutionResult::Continue)
                }
            } else {
                // No matching then found, skip
                Ok(ExecutionResult::Continue)
            }
        }
        Word::Else => {
            // When we hit else, we came from the IF branch, so skip to THEN
            let current_pos = executor.instruction_pointer;

            // Find the IF that this ELSE belongs to
            for (_if_pos, else_pos, then_pos) in &executor.jump_table {
                if *else_pos == Some(current_pos) {
                    return Ok(ExecutionResult::Jump(*then_pos + 1));
                }
            }

            Ok(ExecutionResult::Continue)
        }
        Word::Then => {
            // Then is just a marker, continue execution
            Ok(ExecutionResult::Continue)
        }

        // Movement Actions
        Word::MoveForward => {
            if let Some(distance) = executor.pop_float() {
                let clamped_distance =
                    (distance * 0.01).clamp(-MAX_MOVEMENT_SPEED, MAX_MOVEMENT_SPEED);
                let forward = transform.rotation * Vec3::Y;
                transform.translation += forward * clamped_distance;
                Ok(ExecutionResult::Continue)
            } else {
                Ok(ExecutionResult::Skip)
            }
        }
        Word::MoveBackward => {
            if let Some(distance) = executor.pop_float() {
                let clamped_distance =
                    (distance * 0.01).clamp(-MAX_MOVEMENT_SPEED, MAX_MOVEMENT_SPEED);
                let backward = transform.rotation * Vec3::NEG_Y;
                transform.translation += backward * clamped_distance;
                Ok(ExecutionResult::Continue)
            } else {
                Ok(ExecutionResult::Skip)
            }
        }
        Word::TurnLeft => {
            if let Some(degrees) = executor.pop_float() {
                let clamped_degrees =
                    (degrees * 0.01).clamp(-MAX_ANGULAR_VELOCITY, MAX_ANGULAR_VELOCITY);
                let rotation = Quat::from_rotation_z(clamped_degrees.to_radians());
                transform.rotation = rotation * transform.rotation;
                Ok(ExecutionResult::Continue)
            } else {
                Ok(ExecutionResult::Skip)
            }
        }
        Word::TurnRight => {
            if let Some(degrees) = executor.pop_float() {
                let clamped_degrees =
                    (degrees * 0.01).clamp(-MAX_ANGULAR_VELOCITY, MAX_ANGULAR_VELOCITY);
                let rotation = Quat::from_rotation_z(-clamped_degrees.to_radians());
                transform.rotation = rotation * transform.rotation;
                Ok(ExecutionResult::Continue)
            } else {
                Ok(ExecutionResult::Skip)
            }
        }

        // Resource Actions
        Word::Eat => {
            let animal_pos = transform.translation.truncate();

            // Find plant within eating distance
            for (plant_entity, mut plant, plant_transform) in plants.iter_mut() {
                let plant_pos = plant_transform.translation.truncate();
                if animal_pos.distance(plant_pos) <= EAT_DISTANCE {
                    // Transfer energy from plant to animal
                    let energy_to_transfer = plant.energy.min(EAT_AMOUNT);
                    plant.consume_energy(energy_to_transfer);
                    animal.add_energy(energy_to_transfer);

                    // If plant is depleted, remove it
                    if plant.energy == 0 {
                        commands.entity(plant_entity).despawn();
                    }
                    break;
                }
            }
            Ok(ExecutionResult::Continue)
        }
        Word::Split => {
            // Should never reach here (handled in execute_genomes)
            Ok(ExecutionResult::Continue)
        }

        // Labels (just markers, act like Nop)
        Word::Label0 | Word::Label1 | Word::Label2 | Word::Label3 => Ok(ExecutionResult::Continue),

        // Jumps (jump to label position)
        Word::Jump0 => {
            if let Some(target) = executor.label_table[0] {
                Ok(ExecutionResult::Jump(target))
            } else {
                // Label not found, treat as Nop
                Ok(ExecutionResult::Continue)
            }
        }
        Word::Jump1 => {
            if let Some(target) = executor.label_table[1] {
                Ok(ExecutionResult::Jump(target))
            } else {
                Ok(ExecutionResult::Continue)
            }
        }
        Word::Jump2 => {
            if let Some(target) = executor.label_table[2] {
                Ok(ExecutionResult::Jump(target))
            } else {
                Ok(ExecutionResult::Continue)
            }
        }
        Word::Jump3 => {
            if let Some(target) = executor.label_table[3] {
                Ok(ExecutionResult::Jump(target))
            } else {
                Ok(ExecutionResult::Continue)
            }
        }

        // Special
        Word::Nop => Ok(ExecutionResult::Continue),
    }
}

/// System for animal metabolism - drains energy at configured rate and increments age
pub fn animal_metabolism(
    time: Res<Time>,
    mut timer: ResMut<MetabolismTimer>,
    mut animals: Query<&mut Animal>,
) {
    // Increment age continuously for all animals
    let delta = time.delta_secs();
    for mut animal in animals.iter_mut() {
        animal.age += delta;
    }

    // Drain energy at regular intervals
    if timer.0.tick(time.delta()).just_finished() {
        for mut animal in animals.iter_mut() {
            animal.consume_energy(METABOLISM_COST);
        }
    }
}

/// System to remove dead animals (zero energy or exceeded max lifespan)
pub fn remove_dead_animals(mut commands: Commands, animals: Query<(Entity, &Animal)>) {
    for (entity, animal) in animals.iter() {
        if animal.energy == 0 || animal.age >= MAX_LIFESPAN {
            commands.entity(entity).despawn();
        }
    }
}

/// System to handle animal splitting/reproduction
pub fn split_animals(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    splitting_animals: Query<(Entity, &Animal, &Genome, &Transform), With<PendingSplit>>,
) {
    for (entity, animal, genome, transform) in splitting_animals.iter() {
        // Consume energy for split
        let remaining_energy = animal.energy.saturating_sub(SPLIT_ENERGY_COST);
        let offspring_energy = remaining_energy / 2;

        // Create two offspring with mutated genomes
        for _ in 0..2 {
            let mutated_genome = genome.mutate();
            let position = transform.translation.truncate();
            let rotation = transform.rotation;

            commands.spawn((
                Animal::new(offspring_energy),
                mutated_genome,
                GenomeExecutor::new(offspring_energy),
                Sensors::default(),
                Mesh2d(meshes.add(Circle::new(10.0))),
                MeshMaterial2d(
                    materials.add(ColorMaterial::from_color(Color::srgb(0.9, 0.3, 0.2))),
                ),
                Transform::from_xyz(position.x, position.y, 0.0).with_rotation(rotation),
            ));
        }

        // Despawn the parent
        commands.entity(entity).despawn();
    }
}

/// System to respawn animals when population reaches zero
pub fn population_failsafe(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    animals: Query<&Animal>,
) {
    let count = animals.iter().count();

    if count == 0 {
        spawn_seed_animals(
            &mut commands,
            &mut meshes,
            &mut materials,
            FAILSAFE_RESPAWN_COUNT,
            STARTING_ANIMAL_ENERGY,
        );
    }
}
