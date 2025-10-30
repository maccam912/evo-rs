/// Configuration constants for the evolution simulator

// ============================================================================
// GENOME SETTINGS
// ============================================================================

/// Number of instructions in a newly generated genome
pub const BASE_GENOME_LENGTH: usize = 100;

/// Mutation rate: 1% chance per instruction to be replaced with random instruction
pub const MUTATION_RATE: u32 = 1;

/// Duplication rate: 1% chance per instruction to be duplicated (inserted after)
pub const DUPLICATION_RATE: u32 = 1;

/// Deletion rate: 1% chance per instruction to be deleted
pub const DELETION_RATE: u32 = 1;

/// Energy cost to execute the Split instruction
pub const SPLIT_ENERGY_COST: u32 = 10;

// ============================================================================
// SPAWN SETTINGS
// ============================================================================

/// Number of animals spawned at game start
pub const INITIAL_ANIMAL_COUNT: usize = 500;

/// Starting energy for each animal at spawn
pub const STARTING_ANIMAL_ENERGY: u32 = 10;

/// Number of animals respawned by failsafe when population reaches zero
pub const FAILSAFE_RESPAWN_COUNT: usize = 500;

/// Number of animals spawned by manual spawn button
pub const MANUAL_SPAWN_COUNT: usize = 500;

// ============================================================================
// METABOLISM & TIMING
// ============================================================================

/// Interval in seconds between metabolism ticks (energy drain)
pub const METABOLISM_INTERVAL: f32 = 1.0;

/// Energy drained from each animal per metabolism tick
pub const METABOLISM_COST: u32 = 1;

/// Maximum lifespan of an animal in seconds (animals die when age >= this value)
pub const MAX_LIFESPAN: f32 = 60.0;

/// Interval in seconds between plant spawns
pub const PLANT_SPAWN_INTERVAL: f32 = 1.0;

/// Interval in seconds between plant growth ticks
pub const PLANT_GROWTH_INTERVAL: f32 = 1.0;

/// Energy added to each plant per growth tick
pub const PLANT_GROWTH_AMOUNT: u32 = 1;

/// Maximum energy a plant can store
pub const PLANT_MAX_ENERGY: u32 = 100;

// ============================================================================
// WORLD & INTERACTION SETTINGS
// ============================================================================

/// World bounds for plant spawning (plants spawn within ±WORLD_BOUNDS)
pub const WORLD_BOUNDS: f32 = 500.0;

/// Range for animal spawning (animals spawn within ±ANIMAL_SPAWN_RANGE)
pub const ANIMAL_SPAWN_RANGE: f32 = 200.0;

/// Maximum distance at which an animal can eat a plant
pub const EAT_DISTANCE: f32 = 10.0;

/// Maximum energy transferred from plant to animal per eat action
pub const EAT_AMOUNT: u32 = 20;

/// Maximum distance for selecting entities with mouse
pub const SELECTION_RADIUS: f32 = 20.0;

// ============================================================================
// MOVEMENT LIMITS
// ============================================================================

/// Maximum movement speed per instruction (distance units)
pub const MAX_MOVEMENT_SPEED: f32 = 0.5;

/// Maximum rotation speed per instruction (degrees)
pub const MAX_ANGULAR_VELOCITY: f32 = 5.0;

/// Maximum number of instructions an animal can execute per frame (prevents high-energy animals from moving too fast)
pub const MAX_INSTRUCTIONS_PER_FRAME: u32 = 10;
