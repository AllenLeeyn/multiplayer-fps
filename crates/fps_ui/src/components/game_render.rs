//! # Game Render Component
//!
//! A 3D raycasted game world renderer using DDA (Digital Differential Analyzer) raycasting.
//! Renders walls, floor, ceiling, players, and bullets from a first-person perspective.
//!
//! ## Setup
//!
//! **Construction:** Create with `GameRender::new(id, bounds)`. The component has no maze or
//! camera until the app sends updates.
//!
//! **Data flow:** The app drives all game state via `ComponentUpdate`:
//! - `SetMaze(id, maze)` — set the maze (grid of walls) to raycast against.
//! - `SetGameRender(id, players_map, bullets_list)` — set other players (id → (x, y, angle, is_invincible))
//!   and bullets ((x, y, angle) list). Called each time the app has a new game snapshot.
//! - `SetGameRenderCamera(id, (player_id, x, y, angle, is_invincible))` — set the local camera
//!   (the player whose view we render).
//!
//! The app typically pushes these updates every frame or every network tick so the view stays
//! in sync with the game state.
//!
//! ## Input handling
//!
//! **This component only handles the Escape key.** When the user presses Escape, it emits
//! `UIEvent::ButtonClicked(component_id)` so the app can open the pause menu or leave the game.
//! All other player input (WASD, mouse look, shoot, etc.) is captured in the **app loop**, not
//! inside `GameRender`. The app updates game state and then sends that state back into
//! `GameRender` via the `SetGameRender` / `SetGameRenderCamera` updates above.
//!
//! ## Draw world (raycasting and perspective)
//!
//! **Raycasting:** For each screen column we cast one ray from the camera in the direction
//! corresponding to that column (using the FOV). `cast_ray_dda` uses a DDA (Digital Differential
//! Analyzer) step through the maze grid: we step along the ray in fixed increments (one cell
//! at a time in X or Y) and stop at the first wall. The result is per-column distance and hit
//! position, plus which side of the cell was hit (X-side or Y-side) for texture and shading.
//!
//! **Perspective:** The projection plane is at distance `dist_to_plane` in front of the camera
//! (derived from screen width and FOV). Wall height on screen is computed as
//! `wall_height = (BOX_SIZE * dist_to_plane) / corrected_dist`, so farther walls appear
//! shorter. Fish-eye is reduced by using `corrected_dist = dist * (cos_a*dir_x + sin_a*dir_y)`
//! (distance along the camera forward direction).
//!
//! **Ceiling and floor:** For each pixel column we already have the wall strip (from `draw_start`
//! to `draw_end`). Above the strip we draw the ceiling, below it the floor. For a given screen
//! row we compute the 3D ray that would hit that row: `row_dist = (BOX_SIZE * 0.5 * dist_to_plane) / p`
//! where `p` is the vertical offset from screen center. We then sample the ceiling/floor texture
//! at the world position `(cam + row_dist * dir)` (tiled by `BOX_SIZE`) so ceiling and floor
//! use the same perspective and tiling as the world.
//!
//! **Walls:** The wall strip is texture-mapped using hit position: `wall_u` from the cell edge
//! (hy or hx mod BOX_SIZE), `wall_v` from the vertical position within the strip. Shading uses
//! side (Y-side darker) and distance fade so far walls are dimmer.
//!
//! ## Player drawing (circle sprite with cylinder mapping)
//!
//! Other players are drawn as **circle sprites** (a disc on screen) with **cylinder mapping** so
//! the texture wraps horizontally around the “cylinder” of the sprite. World position is
//! transformed to camera space; we project to screen X using the same FOV formula as walls.
//! Screen size scales with depth: `sprite_size = (PLAYER_SIZE * dist_to_plane) / transform_y`.
//! For each pixel in the sprite’s screen rectangle we compute normalized disc coordinates
//! `(ux, uy)`; if `ux² + uy² ≤ 1` we’re inside the circle. Texture coordinates: horizontally we
//! use the player’s facing angle plus an offset from `ux` (cylinder wrap); vertically we map
//! `uy` to `tv`. A simple shading term (distance from center + highlight) is applied so the
//! circle looks rounded. A soft shadow is drawn below the sprite; both shadow and sprite are
//! depth-tested against the z-buffer filled by the raycast.
//!
//! ## Bullet drawing
//!
//! Bullets are drawn as **small filled quads** (no texture). For each bullet we transform to
//! camera space and skip if behind the camera. We project to screen X with the same FOV math;
//! we depth-test against the z-buffer and skip if occluded. On-screen size scales with
//! distance: `sprite_size = (5.0 * dist_to_plane) / transform_y`. We draw a small rectangle
//! of fixed yellow color centered at the projected position and vertical center; no circle
//! or cylinder mapping, just a perspective-scaled quad.

use std::any::Any;
use std::collections::{HashMap, HashSet};
use std::time::{SystemTime, UNIX_EPOCH};

use fps_levels::maze::Maze;
use winit::event::ElementState;
use winit::keyboard::{KeyCode, PhysicalKey};

use super::super::{
    Color, Component, ComponentUpdate, LayoutMetrics, Rect, UIEvent, UIMainContext, WindowEvent, draw_point, draw_filled_box
};

// 110 Degrees (Wide / Fast-paced FOV)
const FOV: f32 = 110.0 * (std::f32::consts::PI / 180.0);

/// Size of each maze cell in world units.
const BOX_SIZE: f32 = 100.0;

/// Size of player sprites in world units.
const PLAYER_SIZE: f32 = 50.0;

/// Camera offset distance behind the player (in world units).
/// This moves the camera back slightly to improve visibility and feel.
const CAMERA_OFFSET_DISTANCE: f32 = 15.0;

/// Represents a player in the game world.
#[derive(Debug)]
pub struct Player {
    pub id: String,
    pub pos: (f32, f32, f32),
    pub is_invincible: bool,
}

/// Represents a bullet in the game world.
#[derive(Debug)]
pub struct Bullet {
    /// Bullet position: (x, y, angle)
    pub pos: (f32, f32, f32),
}

/// A 3D raycasted game world renderer.
///
/// Renders a first-person view of the game world using raycasting techniques.
/// Supports rendering walls, floor, ceiling, players, and bullets with proper
/// depth sorting and perspective projection.
///
/// # Rendering Features
///
/// - Raycasted walls with texture mapping
/// - Textured floor and ceiling
/// - Player sprites with depth sorting
/// - Bullet rendering
/// - Distance-based lighting and fog
/// - Field of view (FOV) support
///
/// # Example
///
/// ```rust,no_run
/// use fps_ui::components::GameRender;
/// use fps_ui::Rect;
///
/// let renderer = GameRender::new(
///     "game_view".to_string(),
///     Rect::new(0.0, 0.0, 800.0, 600.0),
/// );
/// ```
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
    /// Creates a new game render component.
    ///
    /// # Arguments
    ///
    /// * `id` - Unique identifier for the component
    /// * `bounds` - Rendering area bounds (relative coordinates)
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

    /// Checks if a key is currently held down.
    ///
    /// # Arguments
    ///
    /// * `key` - Key identifier string
    ///
    /// # Returns
    ///
    /// `true` if the key is pressed, `false` otherwise.
    pub fn is_key_down(&self, key: &str) -> bool {
        self.pressed_keys.contains(key)
    }

    /// Gets a snapshot of all currently pressed keys.
    ///
    /// # Returns
    ///
    /// A cloned set of pressed key identifiers.
    pub fn pressed_keys(&self) -> HashSet<String> {
        self.pressed_keys.clone()
    }

    /// Draws the 3D world using raycasting.
    ///
    /// Renders walls, floor, and ceiling with texture mapping. Fills the Z-buffer
    /// for depth testing of sprites.
    ///
    /// # Arguments
    ///
    /// * `frame` - Frame buffer to draw into
    /// * `context` - UI context for textures and layout
    /// * `maze` - The maze to render
    /// * `camera` - Camera/player position and angle
    /// * `z_buffer` - Depth buffer (filled by this function)
    /// * `dist_to_plane` - Distance to projection plane
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
        
        // Offset camera position backward for better visibility
        let cam_x = px - CAMERA_OFFSET_DISTANCE * cos_a;
        let cam_y = py - CAMERA_OFFSET_DISTANCE * sin_a;

        for x in 0..screen_w {
            let camera_x = 2.0 * x as f32 / screen_w as f32 - 1.0;
            let dir_x = cos_a + (-sin_a * camera_x * half_fov.tan());
            let dir_y = sin_a + (cos_a * camera_x * half_fov.tan());

            let (dist, side, (hx, hy)) = cast_ray_dda(cam_x, cam_y, dir_x, dir_y, maze);
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
                    let tx = ((cam_x + row_dist * dir_x) / BOX_SIZE).rem_euclid(1.0);
                    let ty = ((cam_y + row_dist * dir_y) / BOX_SIZE).rem_euclid(1.0);
                    draw_point(frame, screen_w, screen_h, x, y as u32, ceil_tex.sample(tx, ty));
                } else if y >= draw_end {
                    // FLOOR
                    let p = y as f32 - (screen_h as f32 * 0.5);
                    let row_dist = (BOX_SIZE * 0.5 * dist_to_plane) / p;
                    let tx = ((cam_x + row_dist * dir_x) / BOX_SIZE).rem_euclid(1.0);
                    let ty = ((cam_y + row_dist * dir_y) / BOX_SIZE).rem_euclid(1.0);
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

    /// Draws player sprites with depth sorting and perspective projection.
    ///
    /// Players are rendered as textured spheres with proper depth testing against
    /// walls. Includes player names above sprites.
    ///
    /// # Arguments
    ///
    /// * `frame` - Frame buffer to draw into
    /// * `context` - UI context for textures and fonts
    /// * `camera` - Camera/player position and angle
    /// * `z_buffer` - Depth buffer for occlusion testing
    /// * `dist_to_plane` - Distance to projection plane
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

        let cos_a = cam_angle.cos();
        let sin_a = cam_angle.sin();
        
        // Offset camera position backward for better visibility
        let cam_x = px - CAMERA_OFFSET_DISTANCE * cos_a;
        let cam_y = py - CAMERA_OFFSET_DISTANCE * sin_a;

        // 1. Sort players by distance (Back-to-Front)
        let mut sorted_players: Vec<_> = self.players.values().filter(|p| p.id != camera.id).collect();
        sorted_players.sort_by(|a, b| {
            let da = (a.pos.0 - cam_x).powi(2) + (a.pos.1 - cam_y).powi(2);
            let db = (b.pos.0 - cam_x).powi(2) + (b.pos.1 - cam_y).powi(2);
            db.partial_cmp(&da).unwrap()
        });

        for player in sorted_players {
            if player.is_invincible {
                // Use system time or a frame counter to toggle visibility
                /* let time_ms = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap().as_millis();
                
                if (time_ms / 100) % 2 == 0 { continue; }  */

                // just hide the player
                continue;
            }

            let dx = player.pos.0 - cam_x;
            let dy = player.pos.1 - cam_y;

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

    /// Draws bullet sprites with perspective projection.
    ///
    /// Bullets are rendered as small yellow circles that scale with distance.
    /// Only visible bullets (in front of camera, not occluded) are drawn.
    ///
    /// # Arguments
    ///
    /// * `frame` - Frame buffer to draw into
    /// * `context` - UI context for layout information
    /// * `camera` - Camera/player position and angle
    /// * `z_buffer` - Depth buffer for occlusion testing
    /// * `dist_to_plane` - Distance to projection plane
    /// Draws a crosshair at the center of the screen.
    ///
    /// The crosshair is a simple plus sign consisting of continuous horizontal
    /// and vertical lines intersecting at the screen center.
    ///
    /// # Arguments
    ///
    /// * `frame` - Frame buffer to draw into
    /// * `context` - UI context for layout information
    fn draw_crosshair(&self, frame: &mut [u8], context: &mut UIMainContext) {
        let (screen_w, screen_h) = context.layout.size_as_u32();
        let center_x = screen_w / 2;
        let center_y = screen_h / 2;

        // Crosshair dimensions
        const CROSSHAIR_LENGTH: u32 = 20;  // Length of each arm (half-length)
        const CROSSHAIR_THICKNESS: u32 = 2;  // Thickness of the lines

        let crosshair_color = Color::WHITE;

        // Draw horizontal line (full width through center)
        let h_start = center_x.saturating_sub(CROSSHAIR_LENGTH);
        let h_width = (CROSSHAIR_LENGTH * 2).min(screen_w.saturating_sub(h_start));
        draw_filled_box(
            frame,
            screen_w,
            screen_h,
            h_start,
            center_y.saturating_sub(CROSSHAIR_THICKNESS / 2),
            h_width,
            CROSSHAIR_THICKNESS,
            crosshair_color,
        );

        // Draw vertical line (full height through center)
        let v_start = center_y.saturating_sub(CROSSHAIR_LENGTH);
        let v_height = (CROSSHAIR_LENGTH * 2).min(screen_h.saturating_sub(v_start));
        draw_filled_box(
            frame,
            screen_w,
            screen_h,
            center_x.saturating_sub(CROSSHAIR_THICKNESS / 2),
            v_start,
            CROSSHAIR_THICKNESS,
            v_height,
            crosshair_color,
        );
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
        
        // Offset camera position backward for better visibility
        let cam_x = px - CAMERA_OFFSET_DISTANCE * cos_a;
        let cam_y = py - CAMERA_OFFSET_DISTANCE * sin_a;

        for bullet in &self.bullets {
            let dx = bullet.pos.0 - cam_x;
            let dy = bullet.pos.1 - cam_y;

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

        // Draw crosshair at center
        self.draw_crosshair(frame, context);
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

/// Casts a ray using DDA (Digital Differential Analyzer) algorithm.
///
/// Finds the first wall intersection along a ray from the given position and direction.
/// Used for raycasted 3D rendering to determine wall distances and hit positions.
///
/// # Arguments
///
/// * `px` - Starting X position in world coordinates
/// * `py` - Starting Y position in world coordinates
/// * `dir_x` - Ray direction X component (normalized)
/// * `dir_y` - Ray direction Y component (normalized)
/// * `maze` - The maze to cast the ray through
///
/// # Returns
///
/// A tuple of:
/// - `f32`: Distance to the wall
/// - `i32`: Side hit (0 = X-side, 1 = Y-side)
/// - `(f32, f32)`: Hit position (x, y) in world coordinates
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

/// Draws a full-screen color overlay.
///
/// Used for screen effects like invincibility flash or damage indicators.
///
/// # Arguments
///
/// * `frame` - Frame buffer to draw into
/// * `color` - Overlay color (typically semi-transparent)
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

/// Draws an invincibility flash effect.
///
/// Creates a pulsing white overlay to indicate invincibility status.
/// Uses a time-based intensity calculation for the pulse effect.
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