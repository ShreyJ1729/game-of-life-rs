use bevy::diagnostic::FrameTimeDiagnosticsPlugin;
use bevy::diagnostic::LogDiagnosticsPlugin;
use bevy::prelude::*;
use rand::Rng;

const GRID_WIDTH: usize = 300;
const GRID_HEIGHT: usize = GRID_WIDTH * 9 / 16;
const CELL_SIZE: f32 = 3.0;

#[derive(Resource)]
struct Grid([[bool; GRID_WIDTH]; GRID_HEIGHT]);

#[derive(Component)]
struct Cell {
    row: usize,
    col: usize,
}

fn setup(mut commands: Commands, mut grid: ResMut<Grid>) {
    commands.spawn(Camera2dBundle::default());

    let mut rng = rand::thread_rng();
    for i in 0..grid.0.len() {
        for j in 0..grid.0[i].len() {
            grid.0[i][j] = rng.gen_bool(0.1);
        }
    }

    grid.0 = [[false; GRID_WIDTH]; GRID_HEIGHT];

    // glider gun
    let positions = vec![
        (5, 1),
        (5, 2),
        (6, 1),
        (6, 2),
        (5, 11),
        (6, 11),
        (7, 11),
        (4, 12),
        (3, 13),
        (3, 14),
        (8, 12),
        (9, 13),
        (9, 14),
        (6, 15),
        (4, 16),
        (5, 17),
        (6, 17),
        (7, 17),
        (6, 18),
        (8, 16),
        (3, 21),
        (4, 21),
        (5, 21),
        (3, 22),
        (4, 22),
        (5, 22),
        (2, 23),
        (6, 23),
        (1, 25),
        (2, 25),
        (6, 25),
        (7, 25),
        (3, 35),
        (4, 35),
        (3, 36),
        (4, 36),
    ];

    for (row, col) in positions {
        grid.0[row][col] = true;
    }
}

fn render_cells(mut commands: Commands, grid: ResMut<Grid>) {
    for row in 0..grid.0.len() {
        for col in 0..grid.0[0].len() {
            // compute position, size and color
            let position = Vec3::new(
                (col as f32 - grid.0[0].len() as f32 / 2.0) * CELL_SIZE,
                (row as f32 - grid.0.len() as f32 / 2.0) * CELL_SIZE,
                0.0,
            );

            let size = Vec2::new(CELL_SIZE, CELL_SIZE);

            let color = if grid.0[row][col] {
                Color::BLACK
            } else {
                Color::WHITE
            };

            let spritebundle = SpriteBundle {
                sprite: Sprite {
                    color,
                    custom_size: Some(size),
                    ..default()
                },
                transform: Transform::from_translation(position),
                ..default()
            };

            let cell = Cell { row, col };

            // spawn cell in world
            commands.spawn((cell, spritebundle));
        }
    }
}

fn compute_next_generation(mut grid: ResMut<Grid>) {
    // update grid resource with next generation

    // copy grid
    let mut new_grid = [[false; GRID_WIDTH]; GRID_HEIGHT];
    for row in 0..grid.0.len() {
        for col in 0..grid.0[0].len() {
            new_grid[row][col] = grid.0[row][col];
        }
    }

    for row in 0..grid.0.len() {
        for col in 0..grid.0[0].len() {
            let mut count = 0;

            for delta_row in -1..=1 {
                for delta_col in -1..=1 {
                    if delta_row == 0 && delta_col == 0 {
                        continue;
                    }
                    let new_row = row as i32 + delta_row;
                    let new_col = col as i32 + delta_col;
                    if new_row < 0 || new_row >= grid.0.len() as i32 {
                        continue;
                    }
                    if new_col < 0 || new_col >= grid.0[0].len() as i32 {
                        continue;
                    }
                    if grid.0[new_row as usize][new_col as usize] {
                        count += 1;
                    }
                }
            }

            if grid.0[row][col] {
                if count < 2 || count > 3 {
                    new_grid[row][col] = false;
                }
            } else {
                if count == 3 {
                    new_grid[row][col] = true;
                }
            }
        }
    }

    grid.0 = new_grid;
}

fn update_cell_color(mut query: Query<(&Cell, &mut Sprite)>, grid: Res<Grid>) {
    for (cell, mut sprite) in query.iter_mut() {
        if grid.0[cell.row][cell.col] {
            sprite.color = Color::BLACK;
        } else {
            sprite.color = Color::WHITE;
        }
    }
}

fn wait() {
    std::thread::sleep(std::time::Duration::from_millis(33));
}

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            FrameTimeDiagnosticsPlugin::default(),
            LogDiagnosticsPlugin::default(),
        ))
        .insert_resource(Grid {
            0: [[false; GRID_WIDTH]; GRID_HEIGHT],
        })
        .add_systems(Startup, (setup, render_cells).chain())
        .add_systems(
            Update,
            (compute_next_generation, update_cell_color, wait).chain(),
        )
        .run();
}
