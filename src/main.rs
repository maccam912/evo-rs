mod camera;
mod plant;
mod selection;
mod outline;
mod genome;
mod animal;
mod config;

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPlugin};
use camera::{CameraState, MainCamera, camera_pan, camera_zoom, setup_camera};
use plant::{PlantConfig, PlantSpawnTimer, PlantGrowthTimer, spawn_plants, grow_plants, update_plant_visuals, Plant};
use selection::{Selected, SelectedEntity, handle_selection, update_selection_visuals};
use outline::{manage_selection_outlines, update_outline_positions};
use animal::{Animal, MetabolismTimer, spawn_test_animals, spawn_random_animals, update_sensors, execute_genomes, animal_metabolism, remove_dead_animals, split_animals, population_failsafe};
use genome::{Genome, GenomeExecutor, Sensors, WordCategory};
use config::*;

/// Resource to control simulation state
#[derive(Resource, PartialEq, Eq, Clone, Copy)]
pub enum SimulationState {
    Running,
    Paused,
}

impl Default for SimulationState {
    fn default() -> Self {
        SimulationState::Running
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Evolution Ecology Simulator".to_string(),
                resolution: (1280.0, 720.0).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(EguiPlugin)
        .init_resource::<CameraState>()
        .init_resource::<PlantConfig>()
        .init_resource::<SelectedEntity>()
        .init_resource::<SimulationState>()
        .insert_resource(PlantSpawnTimer(Timer::from_seconds(PLANT_SPAWN_INTERVAL, TimerMode::Repeating)))
        .insert_resource(PlantGrowthTimer(Timer::from_seconds(PLANT_GROWTH_INTERVAL, TimerMode::Repeating)))
        .insert_resource(MetabolismTimer(Timer::from_seconds(METABOLISM_INTERVAL, TimerMode::Repeating)))
        .add_systems(Startup, (setup_camera, spawn_test_animals))
        .add_systems(Update, (
            // Always run (even when paused)
            camera_zoom,
            camera_pan,
            handle_selection,
            update_selection_visuals,
            manage_selection_outlines,
            update_outline_positions,
            ui_system,
        ))
        .add_systems(Update, (
            // Only run when simulation is running
            spawn_plants,
            grow_plants,
            update_plant_visuals,
            update_sensors,
            execute_genomes,
            split_animals,
            animal_metabolism,
            remove_dead_animals,
            population_failsafe,
        ).run_if(|state: Res<SimulationState>| *state == SimulationState::Running))
        .run();
}


fn ui_system(
    mut commands: Commands,
    mut contexts: EguiContexts,
    camera_state: Res<CameraState>,
    mut simulation_state: ResMut<SimulationState>,
    selected_entity: Res<SelectedEntity>,
    _query: Query<&Transform, With<MainCamera>>,
    plants: Query<&Plant>,
    animals: Query<&Animal>,
    selected_plants: Query<(&Plant, &Transform), With<Selected>>,
    selected_animals: Query<(&Animal, &Genome, &GenomeExecutor, &Sensors, &Transform), With<Selected>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    egui::Window::new("Simulation Info")
        .default_pos(egui::pos2(10.0, 10.0))
        .show(contexts.ctx_mut(), |ui| {
            // Pause/Resume and Spawn buttons
            ui.horizontal(|ui| {
                let button_text = if *simulation_state == SimulationState::Running {
                    "⏸ Pause"
                } else {
                    "▶ Resume"
                };

                if ui.button(button_text).clicked() {
                    *simulation_state = if *simulation_state == SimulationState::Running {
                        SimulationState::Paused
                    } else {
                        SimulationState::Running
                    };
                }

                let state_text = if *simulation_state == SimulationState::Running {
                    "Running"
                } else {
                    "Paused"
                };
                ui.label(format!("State: {}", state_text));
            });

            ui.horizontal(|ui| {
                if ui.button(format!("➕ Spawn {} Animals", MANUAL_SPAWN_COUNT)).clicked() {
                    spawn_random_animals(
                        &mut commands,
                        &mut meshes,
                        &mut materials,
                        MANUAL_SPAWN_COUNT,
                        STARTING_ANIMAL_ENERGY,
                    );
                }
            });

            ui.separator();
            ui.heading("Camera Controls");
            ui.separator();

            ui.label(format!("Zoom: {:.2}x", camera_state.zoom));
            ui.label(format!("Position: ({:.1}, {:.1})",
                camera_state.position.x,
                camera_state.position.y
            ));

            ui.separator();
            ui.label("Controls:");
            ui.label("• Mouse Wheel - Zoom in/out");
            ui.label("• Middle Mouse - Pan camera");
            ui.label("• Left Click - Select entity");

            ui.separator();
            ui.heading("Ecology Stats");
            ui.separator();

            let plant_count = plants.iter().count();
            let animal_count = animals.iter().count();

            ui.label(format!("Plants: {}", plant_count));
            ui.label(format!("Animals: {}", animal_count));

            if plant_count > 0 {
                let total_energy: u32 = plants.iter().map(|p| p.energy).sum();
                let avg_energy = total_energy as f32 / plant_count as f32;
                ui.label(format!("Plant Total Energy: {}", total_energy));
                ui.label(format!("Plant Avg Energy: {:.1}", avg_energy));
            }

            if animal_count > 0 {
                let total_energy: u32 = animals.iter().map(|a| a.energy).sum();
                let avg_energy = total_energy as f32 / animal_count as f32;
                ui.label(format!("Animal Total Energy: {}", total_energy));
                ui.label(format!("Animal Avg Energy: {:.1}", avg_energy));
            }
        });

    // Show selected entity stats
    if selected_entity.entity.is_some() {
        egui::Window::new("Selected Entity")
            .default_pos(egui::pos2(10.0, 300.0))
            .show(contexts.ctx_mut(), |ui| {
                // Check if it's a plant
                if let Ok((plant, transform)) = selected_plants.get_single() {
                    ui.heading("Plant");
                    ui.separator();

                    ui.label(format!("Energy: {} / {}", plant.energy, Plant::MAX_ENERGY));

                    // Progress bar for energy
                    let energy_ratio = plant.energy as f32 / Plant::MAX_ENERGY as f32;
                    let progress_bar = egui::ProgressBar::new(energy_ratio)
                        .text(format!("{}%", (energy_ratio * 100.0) as u32));
                    ui.add(progress_bar);

                    ui.separator();
                    ui.label(format!("Position: ({:.1}, {:.1})",
                        transform.translation.x,
                        transform.translation.y
                    ));
                } else if let Ok((animal, genome, executor, sensors, transform)) = selected_animals.get_single() {
                    ui.heading("Animal");
                    ui.separator();

                    ui.label(format!("Energy: {}", animal.energy));

                    ui.separator();
                    ui.label(format!("Position: ({:.1}, {:.1})",
                        transform.translation.x,
                        transform.translation.y
                    ));

                    let rotation_degrees = transform.rotation.to_euler(EulerRot::ZXY).0.to_degrees();
                    ui.label(format!("Facing: {:.1}°", rotation_degrees));

                    ui.separator();
                    ui.label("Sensors:");
                    if let Some(dist) = sensors.smell_front {
                        ui.label(format!("  Front: {:.1}", dist));
                    } else {
                        ui.label("  Front: None");
                    }
                    if let Some(dist) = sensors.smell_back {
                        ui.label(format!("  Back: {:.1}", dist));
                    } else {
                        ui.label("  Back: None");
                    }
                    if let Some(dist) = sensors.smell_left {
                        ui.label(format!("  Left: {:.1}", dist));
                    } else {
                        ui.label("  Left: None");
                    }
                    if let Some(dist) = sensors.smell_right {
                        ui.label(format!("  Right: {:.1}", dist));
                    } else {
                        ui.label("  Right: None");
                    }

                    ui.separator();
                    ui.label("Genome:");
                    ui.label(format!("  Words: {}", genome.words.len()));
                    ui.label(format!("  Current IP: {}", executor.instruction_pointer));
                    ui.label(format!("  Stack Size: {}", executor.stack.len()));
                    ui.label(format!("  Executed: {} / {}",
                        executor.instructions_executed_this_frame,
                        executor.max_instructions_per_frame
                    ));
                } else {
                    ui.label("Unknown entity type");
                }
            });
    }

    // Show genome viewer for selected animals
    if selected_entity.entity.is_some() {
        if let Ok((animal, genome, executor, _sensors, _transform)) = selected_animals.get_single() {
            egui::Window::new("Genome Viewer")
                .default_pos(egui::pos2(300.0, 10.0))
                .default_size(egui::vec2(500.0, 600.0))
                .show(contexts.ctx_mut(), |ui| {
                    ui.heading(format!("Stack Machine Genome ({} words)", genome.words.len()));
                    ui.separator();

                    ui.label(format!("Energy: {} | IP: {} | Executed: {}/{}",
                        animal.energy,
                        executor.instruction_pointer,
                        executor.instructions_executed_this_frame,
                        executor.max_instructions_per_frame
                    ));

                    ui.separator();

                    // Stack visualization
                    ui.heading("Stack");
                    if executor.stack.is_empty() {
                        ui.colored_label(egui::Color32::GRAY, "  (empty)");
                    } else {
                        // Display stack top-to-bottom (reversed)
                        for (i, value) in executor.stack.iter().enumerate().rev() {
                            let is_top = i == executor.stack.len() - 1;
                            let prefix = if is_top { "▶ " } else { "  " };
                            ui.monospace(format!("{}[{}] {}", prefix, i, value));
                        }
                    }

                    ui.separator();
                    ui.heading("Program");

                    // Scrollable area for words
                    egui::ScrollArea::vertical()
                        .auto_shrink([false; 2])
                        .show(ui, |ui| {
                            for (index, word) in genome.words.iter().enumerate() {
                                // Check if this is the currently executing word
                                let is_current = index == executor.instruction_pointer;

                                // Get word category for color
                                let category = word.category();
                                let text_color = match category {
                                    WordCategory::Stack => egui::Color32::from_rgb(100, 150, 255),    // Blue
                                    WordCategory::Sensor => egui::Color32::from_rgb(200, 100, 255),   // Purple
                                    WordCategory::Arithmetic => egui::Color32::from_rgb(255, 220, 100), // Yellow
                                    WordCategory::Control => egui::Color32::from_rgb(255, 150, 50),   // Orange
                                    WordCategory::Action => egui::Color32::from_rgb(100, 255, 100),   // Green
                                    WordCategory::Special => egui::Color32::from_rgb(150, 150, 150),  // Gray
                                };

                                // Create the word text with stack effect
                                let text = format!("{:3}: {}  {}", index, word, word.stack_effect());

                                // Draw with background highlight if current word
                                if is_current {
                                    let (rect, response) = ui.allocate_exact_size(
                                        egui::vec2(ui.available_width(), 18.0),
                                        egui::Sense::hover()
                                    );

                                    // Draw highlight background
                                    ui.painter().rect_filled(
                                        rect,
                                        egui::Rounding::same(2.0),
                                        egui::Color32::from_rgba_unmultiplied(255, 255, 0, 80) // Yellow highlight
                                    );

                                    // Draw text on top
                                    ui.painter().text(
                                        rect.left_center() + egui::vec2(5.0, 0.0),
                                        egui::Align2::LEFT_CENTER,
                                        &text,
                                        egui::FontId::monospace(11.0),
                                        text_color
                                    );

                                    response
                                } else {
                                    ui.add(egui::Label::new(
                                        egui::RichText::new(text)
                                            .color(text_color)
                                            .font(egui::FontId::monospace(11.0))
                                    ))
                                };
                            }
                        });
                });
        }
    }
}
