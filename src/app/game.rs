//! # Game Logic Module
//!
//! Core game logic functions for player movement, bullet physics, collisions,
//! and scoring. These functions operate on game entities and are used by both
//! client (for prediction) and server (for authoritative updates).

use rand::{Rng, rng, seq::SliceRandom};
use std::time::{Duration, Instant};

use crate::app::{
    constants::{
        physics::{BOX_SIZE, PLAYER_SIZE, BULLET_SIZE},
        player::{WALK_SPEED, RUN_SPEED, SIDESTEP_FACTOR, BACKSTEP_FACTOR, INVINCIBLE_SPEED_MULTIPLIER},
        bullet::{SPEED as BULLET_SPEED, SHOOT_COOLDOWN_MS, INVINCIBILITY_DURATION_SECONDS, SUBSTEPS as BULLET_SUBSTEPS},
        scoring::{HIT_PENALTY, HIT_REWARD},
    },
    PlayerAction
};

use super::{
    Client, ClientList, Pos, ClientStatus,
    game_structs::{Bullet, BulletStatus}
};

use fps_levels::maze::Maze;

/// Initializes all players at random spawn points when a game starts.
///
/// Assigns each player to a random spawn point from the maze's spawn points
/// with a random rotation angle. Used when transitioning from lobby to game.
///
/// # Arguments
///
/// * `players` - The list of clients to spawn
/// * `maze` - The maze containing spawn points
pub fn game_on_init(players: &mut ClientList, maze: &Maze) {
    let mut rng = rng();

    let mut spawns = maze.spawn_points.clone();
    spawns.shuffle(&mut rng);

    for (client, &(sx, sy)) in players.iter_mut().zip(spawns.iter()) {
        client.pos = Pos {
            x: (sx as f32 + 0.5) * BOX_SIZE,
            y: (sy as f32 + 0.5) * BOX_SIZE,
            angle: rng.random_range(0.0..std::f32::consts::TAU),
        };
    }
}

/// Spawns a single client at a random spawn point.
///
/// Used when a client joins mid-game. Selects a random spawn point and
/// assigns the client to it with a random rotation angle.
///
/// # Arguments
///
/// * `client` - The client to spawn
/// * `maze` - The maze containing spawn points
pub fn spawn_client_randomly(client: &mut Client, maze: &Maze) {
    use rand::Rng;

    let mut rng = rand::rng();

    if maze.spawn_points.is_empty() {
        return;
    }

    let (sx, sy) = maze.spawn_points[rng.random_range(0..maze.spawn_points.len())];

    client.pos = Pos {
        x: (sx as f32 + 0.5) * BOX_SIZE,
        y: (sy as f32 + 0.5) * BOX_SIZE,
        angle: rng.random_range(0.0..std::f32::consts::TAU),
    };
}

/// Updates invincibility status based on elapsed time.
///
/// Decrements the remaining invincibility time and transitions back to
/// normal status when the time expires.
///
/// # Arguments
///
/// * `client` - The client whose invincibility to update
/// * `dt` - Elapsed time in seconds since last update
fn update_invincibility(client: &mut Client, dt: f32) {
    if let ClientStatus::Invincible(ref mut remaining_time) = client.status {
        *remaining_time -= dt;
        
        // Once time runs out, transition back to Normal status
        if *remaining_time <= 0.0 {
            client.status = ClientStatus::Normal;
            println!("[Game] {} is no longer invincible", client.id);
        }
    }
}

/// Handles player movement based on input actions.
///
/// Processes movement actions and applies them to the client's position.
///
/// # Arguments
///
/// * `client` - The client to update
/// * `actions` - Set of active player actions (serialized as bytes)
/// * `_is_running` - Whether the player is running (currently unused but kept for future use)
/// * `move_step` - Distance to move this frame
fn handle_player_movement(
    client: &mut Client,
    actions: &std::collections::HashSet<u8>,
    _is_running: bool,
    move_step: f32,
) {
    for raw in actions {
        let Some(action) = PlayerAction::from_byte(*raw) else {
            continue;
        };

        match action {
            PlayerAction::MoveUp => 
                client.pos.move_forward(move_step),
            PlayerAction::MoveDown => 
                client.pos.move_backward(move_step * BACKSTEP_FACTOR),
            PlayerAction::MoveRight => 
                client.pos.strafe(move_step * SIDESTEP_FACTOR),
            PlayerAction::MoveLeft => 
                client.pos.strafe(-move_step * SIDESTEP_FACTOR),
            _ => {}
        }
    }
}

/// Checks if a position collides with walls.
///
/// Performs collision detection by checking the four corners of a bounding box
/// against the maze. Used for player collision detection.
///
/// # Arguments
///
/// * `maze` - The maze to check against
/// * `x` - X coordinate (world units)
/// * `y` - Y coordinate (world units)
/// * `half_size` - Half the size of the bounding box (player radius)
///
/// # Returns
///
/// `true` if all corners are walkable (no collision), `false` if any corner hits a wall.
fn check_collision_at(maze: &Maze, x: f32, y: f32, half_size: f32) -> bool {
    maze.is_walkable(x - half_size, y - half_size, BOX_SIZE)
        && maze.is_walkable(x + half_size, y - half_size, BOX_SIZE)
        && maze.is_walkable(x - half_size, y + half_size, BOX_SIZE)
        && maze.is_walkable(x + half_size, y + half_size, BOX_SIZE)
}

/// Resolves collisions by reverting position if blocked.
///
/// Checks X and Y axis movement separately and reverts movement on the
/// blocked axis. This allows sliding along walls instead of getting stuck.
///
/// # Arguments
///
/// * `client` - The client whose position to check
/// * `maze` - The maze to check against
/// * `orig_x` - Original X position before movement
/// * `orig_y` - Original Y position before movement
fn resolve_collisions(client: &mut Client, maze: &Maze, orig_x: f32, orig_y: f32) {
    let half_size = PLAYER_SIZE / 2.0;

    // --- X-axis collision ---
    if !check_collision_at(maze, client.pos.x, orig_y, half_size) {
        client.pos.x = orig_x; // undo X if blocked
    }

    // --- Y-axis collision ---
    if !check_collision_at(maze, client.pos.x, client.pos.y, half_size) {
        client.pos.y = orig_y; // undo Y if blocked
    }
}

/// Updates a player's state for one frame.
///
/// Processes invincibility countdown, applies input-based movement and rotation,
/// handles shooting, and resolves collisions. This is the main player update function
/// called each server tick.
///
/// # Arguments
///
/// * `client` - The client to update
/// * `maze` - The maze for collision detection
/// * `bullets` - List of bullets to add new shots to
/// * `dt` - Elapsed time in seconds since last update
pub fn update_player(client: &mut Client, maze: &Maze, bullets: &mut Vec<Bullet>, dt: f32) {
    update_invincibility(client, dt);
    
    let Some(input) = &client.last_input else {
        return;
    };

    // Extract data from input to avoid borrow conflicts
    let actions = input.actions.clone();
    let is_running = input.is_running;
    let mouse_dx = input.mouse_dx;

    client.pos.rotate(mouse_dx * dt);

    let spd_modifier = if client.is_invincible() { INVINCIBLE_SPEED_MULTIPLIER } else { 1.0 };
    let base_speed = player_speed(is_running);
    let move_step = base_speed * dt * spd_modifier;

    let (orig_x, orig_y, _) = client.pos.to_tuple();

    // Handle movement first
    handle_player_movement(client, &actions, is_running, move_step);
    
    // Resolve collisions (may revert position if hitting a wall)
    resolve_collisions(client, maze, orig_x, orig_y);
    
    // Handle shooting AFTER collision resolution so bullets spawn from correct final position
    // Check if shoot action is present and update last_shot_time
    let should_shoot = actions.iter().any(|&raw| {
        PlayerAction::from_byte(raw) == Some(PlayerAction::Shoot)
    });
    
        if should_shoot {
            let now = Instant::now();
            let cooldown = Duration::from_millis(SHOOT_COOLDOWN_MS);
            
            if now.duration_since(client.last_shot_time) >= cooldown {
                bullets.push(Bullet {
                    owner_id: client.id.clone(),
                    pos: client.pos,
                });
                client.last_shot_time = now;
            }
        }
}

/// Gets the base player movement speed.
///
/// # Arguments
///
/// * `is_running` - Whether the player is holding the run key
///
/// # Returns
///
/// The movement speed in world units per second.
#[inline]
fn player_speed(is_running: bool) -> f32 {
    if is_running {
        RUN_SPEED
    } else {
        WALK_SPEED
    }
}

/// Updates a bullet's position and checks for collisions.
///
/// Moves the bullet forward using substeps for accuracy at lower tick rates,
/// checks for wall and player collisions, and returns the bullet's new status.
/// Used each server tick for all active bullets.
///
/// # Arguments
///
/// * `bullet` - The bullet to update
/// * `maze` - The maze for wall collision detection
/// * `clients` - The list of clients for player collision detection
/// * `dt` - Elapsed time in seconds since last update
///
/// # Returns
///
/// The new bullet status:
/// - `Active` if the bullet is still moving
/// - `HitWall` if the bullet hit a wall
/// - `HitPlayer(id)` if the bullet hit a player (returns player ID)
pub fn update_bullet(bullet: &mut Bullet, maze: &Maze, clients: &mut ClientList, dt: f32) -> BulletStatus {
    // Calculate substeps: number of physics steps = substeps + 1
    // 0 substeps (60Hz) = 1 step, 2 substeps (20Hz) = 3 steps
    let num_steps = BULLET_SUBSTEPS + 1;
    let substep_dt = dt / (num_steps as f32);
    
    let bullet_radius = BULLET_SIZE / 2.0;
    let player_radius = PLAYER_SIZE / 2.0;
    let hit_radius_sq = (player_radius + bullet_radius).powi(2);

    // Perform movement and collision checks in substeps
    for _step in 0..num_steps {
        // 1. Move the bullet forward by substep_dt
        bullet.pos.move_forward(BULLET_SPEED * substep_dt);
        
        // 2. Wall Collision Check (Immediate return as walls are static)
        if !maze.is_walkable(bullet.pos.x, bullet.pos.y, BOX_SIZE) {
            return BulletStatus::HitWall;
        }

        // 3. Player Collision Check (Find closest)
        let mut closest_hit: Option<(String, f32)> = None;

        for client in clients.iter() {
            if client.id == bullet.owner_id { continue; }
            
            // Skip invincible players entirely
            if client.is_invincible() { continue; }

            let dx = client.pos.x - bullet.pos.x;
            let dy = client.pos.y - bullet.pos.y;
            let dist_sq = dx * dx + dy * dy;

            if dist_sq < hit_radius_sq {
                // If we haven't hit anyone yet, or this player is closer than the previous find
                match closest_hit {
                    None => closest_hit = Some((client.id.clone(), dist_sq)),
                    Some((_, best_dist_sq)) if dist_sq < best_dist_sq => {
                        closest_hit = Some((client.id.clone(), dist_sq));
                    }
                    _ => {}
                }
            }
        }

        // If we found any hits, return the closest one immediately
        if let Some((id, _)) = closest_hit {
            return BulletStatus::HitPlayer(id);
        }
    }

    BulletStatus::Active
}

/// Handles the result of a bullet hitting a player.
///
/// Applies scoring changes (reward to attacker, penalty to victim) and sets
/// invincibility on the victim. This is the server's authoritative handling
/// of a successful hit.
///
/// # Arguments
///
/// * `attacker_id` - ID of the player who shot the bullet
/// * `victim_id` - ID of the player who was hit
/// * `clients` - The list of clients to update
///
/// # Returns
///
/// The attacker's new score after the hit.
pub fn handle_bullet_hit(
    attacker_id: &str,
    victim_id: &str,
    clients: &mut ClientList,
) -> u32 {
    // 1. Find victim
    if let Some(victim) = clients.iter_mut().find(|c| c.id == victim_id) {
        // Double check status (though update_bullet filters this, it's safe)
        if !victim.is_invincible() {
            // Apply Penalty
            victim.score = victim.score.saturating_sub(HIT_PENALTY);
            
            // Set invincibility duration
            victim.status = ClientStatus::Invincible(INVINCIBILITY_DURATION_SECONDS);
            
            println!("[SERVER] {} hit {}. Victim score: {}", attacker_id, victim_id, victim.score);
        }
    }

    let mut score = 0;
    // 2. Reward Attacker
    if let Some(attacker) = clients.iter_mut().find(|c| c.id == attacker_id) {
        attacker.score += HIT_REWARD;
        score = attacker.score;
    }
    score
}