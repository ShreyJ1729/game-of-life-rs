use bevy::core_pipeline::clear_color::ClearColorConfig;
use bevy::diagnostic::FrameTimeDiagnosticsPlugin;
use bevy::diagnostic::LogDiagnosticsPlugin;
use bevy::ecs::query;
use bevy::prelude::*;
use bevy::sprite::MaterialMesh2dBundle;
use bevy::ui::update;
use bevy::utils::hashbrown::HashMap;
use rand::Rng;

// boid bundle
// position, x/y float
// direction, angle (rad) float
// render (iscoscele triangle as mesh)

// systems
// update position from speed/direction
// update direction from separation
// update direction from alignment
// update direction from cohesion
// update direction from bounds

// consts
const PI: f32 = std::f32::consts::PI;
const SCREEN_WIDTH: f32 = 960.0;
const SCREEN_HEIGHT: f32 = SCREEN_WIDTH * 9.0 / 16.0;
const BG_COLOR: Color = Color::DARK_GRAY;
const BOID_SIZE: f32 = 15.0;
const BOID_COLOR: Color = Color::TURQUOISE;
const NUM_BOIDS: usize = 30;
const SPEED: f32 = 3.0;
const SEPARATION_DISTANCE: f32 = 50.0;
const SEPARATION_SENSITIVITY: f32 = 0.1;
const ALIGNMENT_DISTANCE: f32 = 70.0;
const ALIGNMENT_SENSITIVITY: f32 = 0.00;
const COHESION_DISTANCE: f32 = 100.0;
const COHESION_SENSITIVITY: f32 = 0.00;

#[derive(Component)]
struct Boid {
    id: u16,
    position: (f32, f32),
    velocity: (f32, f32),
    direction: f32,
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle {
        camera_2d: Camera2d {
            clear_color: ClearColorConfig::Custom(BG_COLOR),
        },
        ..Default::default()
    });
}

fn setup_boids(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let mut rng = rand::thread_rng();
    for _ in 0..NUM_BOIDS {
        // gen random position and direction
        let position = (
            rng.gen_range(0.0..SCREEN_WIDTH) - SCREEN_WIDTH / 2.0,
            rng.gen_range(0.0..SCREEN_HEIGHT) - SCREEN_HEIGHT / 2.0,
        );
        let direction = rng.gen_range(0.0..PI * 2.0);

        // create boid bundle
        let boid_bundle = Boid {
            id: rng.gen(),
            position,
            velocity: (0.0, 0.0),
            direction,
        };

        // create render bundle
        let render_bundle = MaterialMesh2dBundle {
            mesh: meshes
                .add(shape::RegularPolygon::new(BOID_SIZE, 3).into())
                .into(),
            material: materials.add(ColorMaterial::from(BOID_COLOR)),
            transform: Transform::from_translation(Vec3::new(position.0, position.1, 0.0))
                * Transform::from_scale(Vec3::new(0.618, 1.0, 1.0))
                * Transform::from_rotation(Quat::from_rotation_z(direction - PI / 2.0)),
            ..default()
        };

        // merge bundles
        let bundle = (boid_bundle, render_bundle);

        // spawn boid
        commands.spawn(bundle);
    }

    commands.spawn(MaterialMesh2dBundle {
        mesh: meshes
            .add(shape::Circle::new(SEPARATION_DISTANCE).into())
            .into(),
        material: materials.add(ColorMaterial::from(Color::PURPLE)),
        // topleft
        transform: Transform::from_translation(Vec3::new(
            -SCREEN_WIDTH / 2.0 - SEPARATION_DISTANCE,
            SCREEN_HEIGHT / 2.0 + SEPARATION_DISTANCE,
            0.0,
        )),
        ..default()
    });

    // draw rectangle showing the screen bounds
    // white borders, transparent fill
    commands.spawn(MaterialMesh2dBundle {
        mesh: meshes
            .add(shape::Quad::new(Vec2::new(SCREEN_WIDTH, SCREEN_HEIGHT)).into())
            .into(),
        material: materials.add(ColorMaterial::from(Color::GRAY)),
        transform: Transform::from_translation(Vec3::new(0.0, 0.0, -1.0)),
        ..default()
    });
}

fn update_positions(mut boids: Query<&mut Boid>) {
    for mut boid in boids.iter_mut() {
        boid.position = (
            boid.position.0 + boid.velocity.0,
            boid.position.1 + boid.velocity.1,
        );
    }
}

fn update_velocities(mut boids: Query<&mut Boid>) {
    for mut boid in boids.iter_mut() {
        boid.velocity = (boid.direction.cos() * SPEED, boid.direction.sin() * SPEED);
    }
}

fn update_directions_from_separation(mut boids: Query<&mut Boid>) {
    // to store updates
    let mut direction_updates = HashMap::new();

    // compute position updates
    for boid1 in boids.iter() {
        for boid2 in boids.iter() {
            if boid1.id == boid2.id {
                continue;
            }

            let distance = ((boid1.position.0 - boid2.position.0).powi(2)
                + (boid1.position.1 - boid2.position.1).powi(2))
            .sqrt();

            if distance < SEPARATION_DISTANCE + BOID_SIZE {
                let towards_collision = (boid1.position.1 - boid2.position.1)
                    .atan2(boid1.position.0 - boid2.position.0);
                let away_from_collision = towards_collision + PI;
                let direction = if (away_from_collision - boid1.direction).abs() < PI {
                    boid1.direction
                        - (away_from_collision - boid1.direction) * SEPARATION_SENSITIVITY
                            / distance.cbrt()
                } else {
                    boid1.direction
                        - (boid1.direction - away_from_collision) * SEPARATION_SENSITIVITY
                            / distance.cbrt()
                };
                direction_updates.insert(boid1.id, direction);
            }
        }
    }

    // execute position updates
    for mut boid in boids.iter_mut() {
        if let Some(new_direction) = direction_updates.get(&boid.id) {
            boid.direction = *new_direction;
        }
    }
}

fn update_directions_from_alignment(mut boids: Query<&mut Boid>) {
    let mut direction_updates = HashMap::new();

    for boid1 in boids.iter() {
        let mut average_direction = 0.0;
        let mut count = 0;

        for boid2 in boids.iter() {
            if boid1.id == boid2.id {
                continue;
            }

            let distance = ((boid1.position.0 - boid2.position.0).powi(2)
                + (boid1.position.1 - boid2.position.1).powi(2))
            .sqrt();
            let distance = distance - BOID_SIZE;

            if distance < ALIGNMENT_DISTANCE {
                average_direction += boid2.direction;
                count += 1;
            }
        }

        if count > 0 {
            let towards_alignment = average_direction / count as f32;

            let direction = if (towards_alignment - boid1.direction).abs() < PI {
                boid1.direction + (towards_alignment - boid1.direction) * ALIGNMENT_SENSITIVITY
            } else {
                boid1.direction + (boid1.direction - towards_alignment) * ALIGNMENT_SENSITIVITY
            };
            direction_updates.insert(boid1.id, direction);
        }
    }

    for mut boid in boids.iter_mut() {
        if let Some(new_direction) = direction_updates.get(&boid.id) {
            boid.direction = *new_direction;
        }
    }
}

fn update_directions_from_cohesion(mut boids: Query<&mut Boid>) {
    let mut direction_updates = HashMap::new();

    for boid1 in boids.iter() {
        let mut average_position = (0.0, 0.0);
        let mut count = 0;

        for boid2 in boids.iter() {
            if boid1.id == boid2.id {
                continue;
            }

            let distance = ((boid1.position.0 - boid2.position.0).powi(2)
                + (boid1.position.1 - boid2.position.1).powi(2))
            .sqrt();
            let distance = distance - BOID_SIZE;

            if distance < COHESION_DISTANCE {
                average_position.0 += boid2.position.0;
                average_position.1 += boid2.position.1;
                count += 1;
            }
        }

        if count > 0 {
            average_position.0 /= count as f32;
            average_position.1 /= count as f32;
            let towards_center = (average_position.1 - boid1.position.1)
                .atan2(average_position.0 - boid1.position.0);

            let direction = if (towards_center - boid1.direction).abs() < PI {
                boid1.direction + (towards_center - boid1.direction) * COHESION_SENSITIVITY
            } else {
                boid1.direction + (boid1.direction - towards_center) * COHESION_SENSITIVITY
            };
            direction_updates.insert(boid1.id, direction);
        }
    }

    for mut boid in boids.iter_mut() {
        if let Some(new_direction) = direction_updates.get(&boid.id) {
            boid.direction = *new_direction;
        }
    }
}

fn stay_away_from_borders(mut boids: Query<&mut Boid>) {
    let mut direction_updates = HashMap::new();

    for boid in boids.iter() {
        // compute distance from each border
        let distance_from_left = boid.position.0 + SCREEN_WIDTH / 2.0;
        let distance_from_right = SCREEN_WIDTH / 2.0 - boid.position.0;
        let distance_from_top = SCREEN_HEIGHT / 2.0 - boid.position.1;
        let distance_from_bottom = boid.position.1 + SCREEN_HEIGHT / 2.0;

        if distance_from_bottom < SEPARATION_DISTANCE {
            let away_from_bottom = PI / 2.0;
            let direction = if (away_from_bottom - boid.direction).abs() < PI {
                boid.direction
                    + (away_from_bottom - boid.direction) * SEPARATION_SENSITIVITY
                        / distance_from_bottom.cbrt()
            } else {
                boid.direction
                    + (boid.direction - away_from_bottom) * SEPARATION_SENSITIVITY
                        / distance_from_bottom.cbrt()
            };
            direction_updates.insert(boid.id, direction);
        }

        if distance_from_top < SEPARATION_DISTANCE {
            let away_from_top = 3.0 * PI / 2.0;
            let direction = if (away_from_top - boid.direction).abs() < PI {
                boid.direction
                    + (away_from_top - boid.direction) * SEPARATION_SENSITIVITY
                        / distance_from_top.cbrt()
            } else {
                boid.direction
                    + (boid.direction - away_from_top) * SEPARATION_SENSITIVITY
                        / distance_from_top.cbrt()
            };
            direction_updates.insert(boid.id, direction);
        }

        if distance_from_left < SEPARATION_DISTANCE {
            let away_from_left = 0.0;
            let direction = if (away_from_left - boid.direction).abs() < PI {
                boid.direction
                    + (away_from_left - boid.direction) * SEPARATION_SENSITIVITY
                        / distance_from_left.cbrt()
            } else {
                boid.direction
                    + (boid.direction - away_from_left) * SEPARATION_SENSITIVITY
                        / distance_from_left.cbrt()
            };
            direction_updates.insert(boid.id, direction);
        }
    }

    for mut boid in boids.iter_mut() {
        if let Some(new_direction) = direction_updates.get(&boid.id) {
            boid.direction = *new_direction;
        }
    }
}

fn update_boids(mut query: Query<(&Boid, &mut Transform)>) {
    for (boid, mut transform) in query.iter_mut() {
        transform.translation = Vec3::new(boid.position.0, boid.position.1, 0.0);
        transform.rotation = Quat::from_rotation_z(boid.direction - PI / 2.0);
    }
}

fn wait() {
    std::thread::sleep(std::time::Duration::from_millis(20));
}

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            FrameTimeDiagnosticsPlugin::default(),
            LogDiagnosticsPlugin::default(),
        ))
        .add_systems(Startup, (setup_camera, setup_boids).chain())
        .add_systems(
            Update,
            (
                update_directions_from_separation,
                update_directions_from_alignment,
                update_directions_from_cohesion,
                stay_away_from_borders,
                update_velocities,
                update_positions,
                update_boids,
                wait,
            )
                .chain(),
        )
        .run();
}
