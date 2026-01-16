use rand::rng;
use rand::{Rng, seq::SliceRandom};
use std::time::{Duration, Instant};

use crate::app::{
    BOX_SIZE,
    PLAYER_SIZE,
    PLAYER_BACKSTEP_FACTOR,
    PLAYER_RUN_SPD,
    PLAYER_SIDESTEP_FACTOR,
    PLAYER_WALK_SPD,
    BULLET_SIZE, BULLET_SPD,
    PlayerAction
};

use super::{
    Client, ClientList, Pos, ClientStatus,
    game_structs::{Bullet, BulletStatus}
};

use fps_levels::maze::Maze;

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

pub fn update_player(client: &mut Client, maze: &Maze, bullets: &mut Vec<Bullet>, dt: f32) {
    if let ClientStatus::Invincible(ref mut remaining_time) = client.status {
        *remaining_time -= dt;
        
        // Once time runs out, transition back to Normal status
        if *remaining_time <= 0.0 {
            client.status = ClientStatus::Normal;
            println!("[Game] {} is no longer invincible", client.id);
        }
    }
    
    let Some(input) = &client.last_input else {
        return;
    };

    client.pos.rotate(input.mouse_dx * dt);

    let spd_modifier = if client.is_invincible() { 1.5 } else { 1.0 };
    let base_speed = player_speed(input.is_running);
    let move_step = base_speed * dt * spd_modifier;

    let (orig_x, orig_y, _) = client.pos.to_tuple();

    for raw in &input.actions {
        let Some(action) = PlayerAction::from_byte(*raw) else {
            continue;
        };

        match action {
            PlayerAction::MoveUp => 
                client.pos.move_forward(move_step),
            PlayerAction::MoveDown => 
                client.pos.move_backward(move_step * PLAYER_BACKSTEP_FACTOR),
            PlayerAction::MoveRight => 
                client.pos.strafe(move_step * PLAYER_SIDESTEP_FACTOR),
            PlayerAction::MoveLeft => 
                client.pos.strafe(-move_step * PLAYER_SIDESTEP_FACTOR),
            PlayerAction::Shoot => {
                let now = Instant::now();
                let cooldown = Duration::from_millis(300); // 0.3 second cooldown

                if now.duration_since(client.last_shot_time) >= cooldown {
                    bullets.push(Bullet {
                        owner_id: client.id.clone(),
                        pos: client.pos,
                    });
                    client.last_shot_time = now;
                }
            }
            _ => {}
        }
    }

    // Half-size radius for collision checks
    let half_size = PLAYER_SIZE / 2.0;

    // --- X-axis collision ---
    if !maze.is_walkable(client.pos.x - half_size, orig_y - half_size, BOX_SIZE)
        || !maze.is_walkable(client.pos.x + half_size, orig_y - half_size, BOX_SIZE)
        || !maze.is_walkable(client.pos.x - half_size, orig_y + half_size, BOX_SIZE)
        || !maze.is_walkable(client.pos.x + half_size, orig_y + half_size, BOX_SIZE)
    {
        client.pos.x = orig_x; // undo X if blocked
    }

    // --- Y-axis collision ---
    if !maze.is_walkable(client.pos.x - half_size, client.pos.y - half_size, BOX_SIZE)
        || !maze.is_walkable(client.pos.x + half_size, client.pos.y - half_size, BOX_SIZE)
        || !maze.is_walkable(client.pos.x - half_size, client.pos.y + half_size, BOX_SIZE)
        || !maze.is_walkable(client.pos.x + half_size, client.pos.y + half_size, BOX_SIZE)
    {
        client.pos.y = orig_y; // undo Y if blocked
    }

}

#[inline]
fn player_speed(is_running: bool) -> f32 {
    if is_running {
        PLAYER_RUN_SPD
    } else {
        PLAYER_WALK_SPD
    }
}

pub fn update_bullet(bullet: &mut Bullet, maze: &Maze, clients: &mut ClientList, dt: f32) -> BulletStatus {
    // 1. Move the bullet forward
    bullet.pos.move_forward(BULLET_SPD * dt);
    
    let bullet_radius = BULLET_SIZE / 2.0;
    let player_radius = PLAYER_SIZE / 2.0;
    let hit_radius_sq = (player_radius + bullet_radius).powi(2);

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

    // If we found any hits, return the closest one
    if let Some((id, _)) = closest_hit {
        return BulletStatus::HitPlayer(id);
    }

    BulletStatus::Active
}

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
            victim.score = victim.score.saturating_sub(5);
            
            // Set 2 seconds of invincibility
            victim.status = ClientStatus::Invincible(2.0);
            
            println!("[SERVER] {} hit {}. Victim score: {}", attacker_id, victim_id, victim.score);
        }
    }

    let mut score = 0;
    // 2. Reward Attacker
    if let Some(attacker) = clients.iter_mut().find(|c| c.id == attacker_id) {
        attacker.score += 10;
        score = attacker.score;
    }
    score
}