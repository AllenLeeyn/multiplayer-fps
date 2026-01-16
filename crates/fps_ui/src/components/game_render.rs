use std::any::Any;
use std::collections::{HashMap, HashSet};
use std::fmt::Debug;
use std::time::{SystemTime, UNIX_EPOCH};

use fps_levels::maze::Maze;
use winit::event::ElementState;
use winit::keyboard::{KeyCode, PhysicalKey};

use super::super::{
    Color, Component, ComponentUpdate, LayoutMetrics, Rect, UIEvent, UIMainContext, WindowEvent, draw_point
};

const FOV: f32 = std::f32::consts::FRAC_PI_3; // 60°
const BOX_SIZE: f32 = 100.0;
const PLAYER_SIZE: f32 = 50.0;

#[derive(Debug)]
pub struct Player {
    pub id: String,
    pub pos: (f32, f32, f32),
    pub is_invincible: bool,
}

#[derive(Debug)]
pub struct Bullet {
    pub pos: (f32, f32, f32),
}

#[derive(Debug)]
pub struct GameRender {
    id: String,
    bounds: Rect,
    needs_redraw: bool,
    pressed_keys: HashSet<String>,
    maze: Option<Maze>,
    players: HashMap<String, Player>,
    bullets: Vec<Bullet>,
    current_player: Option<Player>
}

impl GameRender {
    pub fn new(id: String, bounds: Rect) -> Self {
        Self {
            id,
            bounds,
            needs_redraw: true,
            pressed_keys: HashSet::new(),
            maze: None,
            players: HashMap::new(),
            bullets: Vec::new(),
            current_player: None
        }
    }

    /// Check if a key is currently held down
    pub fn is_key_down(&self, key: &str) -> bool {
        self.pressed_keys.contains(key)
    }

    /// Get a snapshot of all currently pressed keys
    pub fn pressed_keys(&self) -> HashSet<String> {
        self.pressed_keys.clone()
    }

    fn draw_world(
        &self,
        frame: &mut [u8],
        context: &mut UIMainContext,
        maze: &Maze,
        camera: &Player,
        z_buffer: &mut [f32],
        dist_to_plane: f32,
    ) {
        let (px, py, angle) = camera.pos;
        let (screen_w, screen_h) = context.layout.size_as_u32();
        let half_fov = FOV * 0.5;

        let floor_tex = context.get_texture("tile_big").unwrap();
        let ceil_tex = context.get_texture("tile_big").unwrap();
        let wall_tex = context.get_texture("tile_big").unwrap();

        let cos_a = angle.cos();
        let sin_a = angle.sin();

        for x in 0..screen_w {
            let camera_x = 2.0 * x as f32 / screen_w as f32 - 1.0;
            let dir_x = cos_a + (-sin_a * camera_x * half_fov.tan());
            let dir_y = sin_a + (cos_a * camera_x * half_fov.tan());

            let (dist, side, (hx, hy)) = cast_ray_dda(px, py, dir_x, dir_y, maze);
            let corrected_dist = dist * (cos_a * dir_x + sin_a * dir_y);
            z_buffer[x as usize] = corrected_dist;

            let wall_height = (BOX_SIZE * dist_to_plane / corrected_dist) as i32;
            let draw_start = screen_h as i32 / 2 - wall_height / 2;
            let draw_end = screen_h as i32 / 2 + wall_height / 2;

            let wall_u = if side == 0 { hy % BOX_SIZE } else { hx % BOX_SIZE } / BOX_SIZE;

            for y in 0..screen_h as i32 {
                if y < draw_start {
                    // CEILING
                    let p = (screen_h as f32 * 0.5) - y as f32;
                    let row_dist = (BOX_SIZE * 0.5 * dist_to_plane) / p;
                    let tx = ((px + row_dist * dir_x) / BOX_SIZE).rem_euclid(1.0);
                    let ty = ((py + row_dist * dir_y) / BOX_SIZE).rem_euclid(1.0);
                    draw_point(frame, screen_w, screen_h, x, y as u32, ceil_tex.sample(tx, ty));
                } else if y >= draw_end {
                    // FLOOR
                    let p = y as f32 - (screen_h as f32 * 0.5);
                    let row_dist = (BOX_SIZE * 0.5 * dist_to_plane) / p;
                    let tx = ((px + row_dist * dir_x) / BOX_SIZE).rem_euclid(1.0);
                    let ty = ((py + row_dist * dir_y) / BOX_SIZE).rem_euclid(1.0);
                    draw_point(frame, screen_w, screen_h, x, y as u32, floor_tex.sample(tx, ty));
                } else {
                    // WALL
                    let wall_v = (y - draw_start) as f32 / wall_height as f32;
                    let color = wall_tex.sample(wall_u, wall_v);
                    let intensity = if side == 1 { 0.7 } else { 1.0 };
                    let distance_fade = (1.0 - (corrected_dist / (BOX_SIZE * 15.0))).max(0.15);
                    draw_point(frame, screen_w, screen_h, x, y as u32, color.darken(intensity * distance_fade));
                }
            }
        }
    }

    fn draw_players(
        &self,
        frame: &mut [u8],
        context: &mut UIMainContext,
        camera: &Player,
        z_buffer: &[f32],
        dist_to_plane: f32,
    ) {
        let (px, py, cam_angle) = camera.pos;
        let (screen_w, screen_h) = context.layout.size_as_u32();
        let half_fov_tan = (FOV * 0.5).tan();

        // 1. Sort players by distance (Back-to-Front)
        let mut sorted_players: Vec<_> = self.players.values().filter(|p| p.id != camera.id).collect();
        sorted_players.sort_by(|a, b| {
            let da = (a.pos.0 - px).powi(2) + (a.pos.1 - py).powi(2);
            let db = (b.pos.0 - px).powi(2) + (b.pos.1 - py).powi(2);
            db.partial_cmp(&da).unwrap()
        });

        let cos_a = cam_angle.cos();
        let sin_a = cam_angle.sin();

        for player in sorted_players {
            if player.is_invincible {
                // Use system time or a frame counter to toggle visibility
                let time_ms = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap().as_millis();
                
                if (time_ms / 100) % 2 == 0 { continue; } 
            }

            let dx = player.pos.0 - px;
            let dy = player.pos.1 - py;

            // Transform world coordinates to camera space
            let transform_x = cos_a * dy - sin_a * dx;
            let transform_y = sin_a * dy + cos_a * dx;

            if transform_y <= 0.1 { continue; }

            let screen_x = (screen_w as f32 * 0.5) * (1.0 + transform_x / (transform_y * half_fov_tan));
            let full_wall_height = (BOX_SIZE * dist_to_plane / transform_y) as i32;
            let sprite_size = (PLAYER_SIZE * dist_to_plane / transform_y) as i32;
            let v_offset = screen_h as i32 / 2;

            // --- PRE-CALCULATE TEXTURE CONSTANTS ---
            let view_angle = dy.atan2(dx);
            let relative_rot = player.pos.2 - view_angle + std::f32::consts::PI;
            let sphere_tex = context.textures.get("eye").expect("Texture 'eye' missing");

            // --- DRAW SHADOW ---
            let shadow_y_screen = v_offset + (full_wall_height / 2);
            let shadow_w = sprite_size;
            let shadow_h = sprite_size / 6;
            
            let sx_min = (screen_x as i32 - shadow_w / 2).max(0);
            let sx_max = (screen_x as i32 + shadow_w / 2).min(screen_w as i32);

            for sx in sx_min..sx_max {
                if transform_y < z_buffer[sx as usize] {
                    let sy_min = (shadow_y_screen - shadow_h / 2).max(0);
                    let sy_max = (shadow_y_screen + shadow_h / 2).min(screen_h as i32);
                    for sy in sy_min..sy_max {
                        let ux = (sx as f32 - screen_x) / (shadow_w as f32 * 0.5);
                        let uy = (sy as f32 - shadow_y_screen as f32) / (shadow_h as f32 * 0.5);
                        let dist_sq = ux * ux + uy * uy;
                        if dist_sq <= 1.0 {
                            draw_point_darken(frame, screen_w, screen_h, sx as u32, sy as u32, 0.5 * (1.0 - dist_sq));
                        }
                    }
                }
            }

            // --- DRAW PLAYER SPHERE (Cylindrical Mapping) ---
            let x_min = (screen_x as i32 - sprite_size / 2).max(0);
            let x_max = (screen_x as i32 + sprite_size / 2).min(screen_w as i32);
            let y_min_sprite = v_offset - sprite_size / 2;

            for sx in x_min..x_max {
                if transform_y < z_buffer[sx as usize] {
                    let y_min = (v_offset - sprite_size / 2).max(0);
                    let y_max = (v_offset + sprite_size / 2).min(screen_h as i32);
                    for sy in y_min..y_max {
                        let ux = (sx as f32 - screen_x) / (sprite_size as f32 * 0.5);
                        let uy = (sy as f32 - v_offset as f32) / (sprite_size as f32 * 0.5);
                        let dist_sq = ux * ux + uy * uy;

                        if dist_sq <= 1.0 {
                            // Horizontal mapping
                            let sprite_angle_offset = ux * (std::f32::consts::PI / 2.0);
                            let tu = (0.5 + (relative_rot + sprite_angle_offset) / (2.0 * std::f32::consts::PI)).rem_euclid(1.0);
                            let tv = (uy + 1.0) * 0.5;

                            let color = sphere_tex.sample(tu, tv);
                            
                            // Lighting/Shading
                            let shading = (1.0 - dist_sq).sqrt() * 0.8 + 0.2;
                            let highlight = (1.0 - ((ux - 0.3).powi(2) + (uy + 0.3).powi(2)).sqrt()).max(0.0).powi(4);
                            
                            draw_point(frame, screen_w, screen_h, sx as u32, sy as u32, color.darken(shading + highlight));
                        }
                    }
                }
            }

            // --- DRAW USERNAME ---
            let font_size = (16.0 * (dist_to_plane / transform_y)).clamp(8.0, 24.0);
            let text_width = context.font_manager.measure_text_width(&player.id, font_size);
            let text_x = screen_x as f64 - (text_width / 2.0);
            let text_y = y_min_sprite as f64 - (font_size as f64 * 0.5) - 5.0;

            if text_y > 0.0 && transform_y < z_buffer[screen_x as usize % screen_w as usize] {
                context.font_manager.draw_text(
                    frame, &player.id, font_size, Color::RED,
                    text_x, text_y, screen_w, screen_h,
                );
            }
        }
    }

    fn draw_bullets(
        &self,
        frame: &mut [u8],
        context: &mut UIMainContext,
        camera: &Player,
        z_buffer: &[f32],
        dist_to_plane: f32,
    ) {
        let (px, py, cam_angle) = camera.pos;
        let (screen_w, screen_h) = context.layout.size_as_u32();
        let half_fov_tan = (FOV * 0.5).tan();
        let cos_a = cam_angle.cos();
        let sin_a = cam_angle.sin();

        for bullet in &self.bullets {
            let dx = bullet.pos.0 - px;
            let dy = bullet.pos.1 - py;

            // Transform to camera space
            let transform_x = cos_a * dy - sin_a * dx;
            let transform_y = sin_a * dy + cos_a * dx;

            // 1. CLIP: Bullet is behind the camera
            if transform_y <= 5.0 { continue; } 

            // 2. PROJECT: Determine screen position
            let screen_x = (screen_w as f32 * 0.5) * (1.0 + transform_x / (transform_y * half_fov_tan));
            
            // 3. DEPTH CHECK: Check against Z-Buffer
            // Ensure z_buffer and transform_y are in the same units (world units)
            let sx = screen_x as i32;
            if sx < 0 || sx >= screen_w as i32 || transform_y > z_buffer[sx as usize] {
                continue; 
            }

            // 4. DRAW: Size scales inversely with distance
            let sprite_size = (5.0 * dist_to_plane / transform_y) as i32; 
            let v_offset = screen_h as i32 / 2;

            let x_start = (sx - sprite_size / 2).max(0);
            let x_end = (sx + sprite_size / 2).min(screen_w as i32);
            let y_start = (v_offset - sprite_size / 2).max(0);
            let y_end = (v_offset + sprite_size / 2).min(screen_h as i32);

            for x in x_start..x_end {
                for y in y_start..y_end {
                    draw_point(frame, screen_w, screen_h, x as u32, y as u32, Color::new(255, 255, 0, 255));
                }
            }
        }
    }

}

// --- Component Trait Implementation ---

impl Component for GameRender {
    fn id(&self) -> &str {
        &self.id
    }

    fn bounds(&self) -> Rect {
        self.bounds
    }

    fn layout_metrics(&self) -> LayoutMetrics {
        LayoutMetrics::default()
    }

    fn handle_input(
        &mut self,
        event: &WindowEvent,
        _cursor_pos: Option<(f64, f64)>,
    ) -> Vec<UIEvent> {
        if let WindowEvent::KeyboardInput { event, .. } = event {
            if event.state == ElementState::Pressed {
                if let PhysicalKey::Code(KeyCode::Escape) = event.physical_key {
                    return vec![UIEvent::ButtonClicked(self.id.clone())];
                }
            }
        }

        Vec::new()
    }

    fn draw(&self, frame: &mut [u8], context: &mut UIMainContext) {
        let Some(maze) = &self.maze else { return };
        let Some(camera) = &self.current_player else { return };
        let (screen_w, _screen_h) = context.layout.size_as_u32();

        // Setup shared constants
        let half_fov = FOV * 0.5;
        let dist_to_plane = (screen_w as f32 * 0.5) / half_fov.tan();
        let mut z_buffer = vec![f32::MAX; screen_w as usize];

        // Draw World (Floor, Ceil, Walls) and fill Z-Buffer
        self.draw_world(frame, context, maze, camera, &mut z_buffer, dist_to_plane);

        // Draw Players as Spheres
        self.draw_players(frame, context, camera, &z_buffer, dist_to_plane);

        self.draw_bullets(frame, context, camera, &z_buffer, dist_to_plane);

        // Invincibility Pulse Effect
        if camera.is_invincible {
            draw_flash(frame);
        }
    }

    fn apply_update(&mut self, update: &ComponentUpdate) -> bool {
        match update {
            ComponentUpdate::SetPosition(_, x, y) => {
                self.bounds.x = *x;
                self.bounds.y = *y;
                self.needs_redraw = true;
                true
            }

            ComponentUpdate::Resize { width, height } => {
                // Bounds::new is inclusive, so it creates the new Rect (0.0, 0.0, width, height)
                self.bounds = Rect::new(0.0, 0.0, *width, *height);
                self.needs_redraw = true;
                true
            }

            ComponentUpdate::SetMaze(id, maze ) if *id == self.id => {
                self.maze = Some(maze.clone());
                self.needs_redraw = true;

                false
            }

            ComponentUpdate::SetGameRender(
                id,
                players,
                bullets,
            ) if *id == self.id => {
                // 1. Update Players
                self.players = players.iter()
                    .map(|(id, &(x, y, angle, is_invincible))| {
                        (id.clone(), Player {
                            id: id.clone(),
                            pos: (x, y, angle),
                            is_invincible,
                        }) // score optional
                    }).collect();

                // 2. Update Bullets
                self.bullets = bullets.iter()
                    .map(|&(x, y, angle)| Bullet {
                        pos: (x, y, angle),
                    })
                    .collect();

                self.needs_redraw = true;
                true
            }

            ComponentUpdate::SetGameRenderCamera(
                id,
                player
            ) if *id == self.id => {
                self.current_player = Some(
                    Player {
                        id: player.0.clone(),
                        pos: (player.1, player.2, player.3),
                        is_invincible: player.4
                    }
                );

                self.needs_redraw = true;
                true
            }

            _ => false,
        }
    }

    fn requires_redraw(&mut self) -> bool {
        if self.needs_redraw {
            self.needs_redraw = false;
            true
        } else {
            false
        }
    }

    fn get_text(&self) -> &str {
        &self.id
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

pub fn cast_ray_dda(px: f32, py: f32, dir_x: f32, dir_y: f32, maze: &Maze) -> (f32, i32, (f32, f32)) {
    let mut map_x = (px / BOX_SIZE).floor() as i32;
    let mut map_y = (py / BOX_SIZE).floor() as i32;

    let delta_dist_x = (1.0 / dir_x).abs();
    let delta_dist_y = (1.0 / dir_y).abs();

    let (step_x, mut side_dist_x) = if dir_x < 0.0 {
        (-1, (px / BOX_SIZE - map_x as f32) * delta_dist_x)
    } else {
        (1, (map_x as i32 as f32 + 1.0 - px / BOX_SIZE) * delta_dist_x)
    };

    let (step_y, mut side_dist_y) = if dir_y < 0.0 {
        (-1, (py / BOX_SIZE - map_y as f32) * delta_dist_y)
    } else {
        (1, (map_y as i32 as f32 + 1.0 - py / BOX_SIZE) * delta_dist_y)
    };

    let mut side = 0; // 0 for X, 1 for Y
    let mut hit = false;
    let mut depth = 0;

    while !hit && depth < 100 {
        if side_dist_x < side_dist_y {
            side_dist_x += delta_dist_x;
            map_x += step_x;
            side = 0;
        } else {
            side_dist_y += delta_dist_y;
            map_y += step_y;
            side = 1;
        }
        
        if maze.get_cell_safe(map_x, map_y) { hit = true; }
        depth += 1;
    }

    let dist = if side == 0 { side_dist_x - delta_dist_x } else { side_dist_y - delta_dist_y };
    let hit_x = px + (dist * dir_x * BOX_SIZE);
    let hit_y = py + (dist * dir_y * BOX_SIZE);

    (dist * BOX_SIZE, side, (hit_x, hit_y))
}

fn draw_point_darken(frame: &mut [u8], width: u32, _height: u32, x: u32, y: u32, intensity: f32) {
    let idx = ((y * width + x) * 4) as usize;
    if idx + 3 >= frame.len() { return; }

    // Multiplier: 1.0 means no change, 0.0 means black. 
    // We subtract the shadow intensity from 1.0.
    let multiplier = (1.0 - intensity).clamp(0.0, 1.0);

    frame[idx]     = (frame[idx] as f32 * multiplier) as u8;     // R
    frame[idx + 1] = (frame[idx + 1] as f32 * multiplier) as u8; // G
    frame[idx + 2] = (frame[idx + 2] as f32 * multiplier) as u8; // B
}

fn draw_screen_overlay(frame: &mut [u8], color: Color) {
    let alpha = color.value[3] as f32 / 255.0;
    let r = color.value[0] as f32;
    let g = color.value[1] as f32;
    let b = color.value[2] as f32;

    for i in (0..frame.len()).step_by(4) {
        // Simple Alpha Blending: Result = (Color * Alpha) + (Background * (1 - Alpha))
        frame[i]     = (r * alpha + frame[i] as f32 * (1.0 - alpha)) as u8;
        frame[i + 1] = (g * alpha + frame[i + 1] as f32 * (1.0 - alpha)) as u8;
        frame[i + 2] = (b * alpha + frame[i + 2] as f32 * (1.0 - alpha)) as u8;
    }
}

fn draw_flash(frame: &mut [u8]) {
    let time_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();

    // Flash every 200ms (5 times per second)
    // If the remainder of (time / 100) is even, we show the flash.
    // This creates a 100ms ON, 100ms OFF pattern.
    if (time_ms / 100) % 2 == 0 {
        // High alpha (100) for a sharp, noticeable warning
        let flash_color = Color::new(255, 0, 0, 100);
        draw_screen_overlay(frame, flash_color);
    }
}