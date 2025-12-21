use fps_levels::{
    config::{Difficulty, MazeConfig, MazeSize},
    generator::generate_maze,
};

fn main() {
    let config = MazeConfig {
        size: MazeSize::Small,
        difficulty: Difficulty::Hard,
    };

    let maze = generate_maze(&config, "Debug Maze".into());

    println!("Maze: {}", maze.name);
    println!("Size: {} x {}", maze.width, maze.height);
    println!("Rooms: {} x {}", config.room_count(), config.room_count());
    println!("Spawn points: {}", maze.spawn_points.len());
    println!();

    println!("{}", maze);

    println!("Spawn coordinates:");
    for (i, (x, y)) in maze.spawn_points.iter().enumerate() {
        println!("  {}: ({}, {})", i, x, y);
    }
}
