//! # Mini Map Component
//!
//! A top-down minimap component that displays the maze layout and player position.
//! Uses cached rendering for efficient updates when only player position changes.

use std::any::Any;
use super::super::draw_triangle;

use fps_levels::maze::{Cell, Maze};

use super::super::{
    Color, Component, ComponentUpdate, LayoutMetrics, Rect, UIMainContext, WindowEvent,
    calculate_absolute_rect, draw_filled_bordered_box,
};

/// A top-down minimap component showing maze layout and player position.
///
/// The minimap caches the maze rendering for efficiency, only redrawing when
/// the maze changes. Player position updates are fast as they only require
/// drawing a triangle indicator.
///
/// # Features
///
/// - Cached maze rendering for performance
/// - Player position indicator (red triangle)
/// - Configurable cell and player sizes
/// - Efficient updates when only position changes
///
/// # Example
///
/// ```rust,no_run
/// use fps_ui::components::MiniMap;
/// use fps_levels::{maze::Maze, generator::generate_maze, config::MazeConfig};
/// use fps_ui::{Rect, layout::LayoutMetrics};
///
/// let config = MazeConfig::new();
/// let maze = generate_maze(&config, "Level 1".to_string());
///
/// let minimap = MiniMap::new(
///     "minimap",
///     maze,
///     Rect::new(600.0, 10.0, 150.0, 150.0),
///     LayoutMetrics::default(),
///     5.0,   // cell size in pixels
///     2.5,   // player size in pixels
/// );
/// ```
#[derive(Debug)]
pub struct MiniMap {
    id: String,
    bounds: Rect,
    layout: LayoutMetrics,

    maze_size: u32,
    maze_cache: Vec<u8>,  // RGBA pixels
    maze_width_px: u32,
    maze_height_px: u32,

    player_pos: (f32, f32, f32),

    cell_px: f32,
    player_px: f32,

    dirty: bool,
}

impl MiniMap {
    /// Creates a new minimap component.
    ///
    /// # Arguments
    ///
    /// * `id` - Unique identifier for the component
    /// * `maze` - The maze to display
    /// * `bounds` - Component's bounding rectangle (relative coordinates)
    /// * `layout` - Layout metrics for positioning
    /// * `cell_px` - Size of each cell in pixels
    /// * `player_px` - Size of the player indicator in pixels
    pub fn new(
        id: impl Into<String>,
        maze: Maze,
        bounds: Rect,
        layout: LayoutMetrics,
        cell_px: f32,
        player_px: f32,
    ) -> Self {
        
        let cell_px_u32 = cell_px.round() as u32;

        let (maze_cache, w, h, maze_size, cell_size) =
            Self::build_maze_cache(&maze, cell_px_u32);

        Self {
            id: id.into(),
            bounds,
            layout,
            maze_size,
            maze_cache,
            maze_width_px: w,
            maze_height_px: h,
            player_pos: (0.0, 0.0, 0.0),
            cell_px: cell_size as f32,
            player_px,
            dirty: true,
        }
    }
    
    /// Builds a cached pixel buffer of the maze for efficient rendering.
    ///
    /// The cache is only rebuilt when the maze changes, allowing fast updates
    /// when only the player position changes.
    ///
    /// # Arguments
    ///
    /// * `maze` - The maze to cache
    /// * `cell_px` - Size of each cell in pixels
    ///
    /// # Returns
    ///
    /// A tuple of `(pixel_buffer, width, height, maze_size, cell_size)`.
    fn build_maze_cache(
        maze: &Maze,
        cell_px: u32,
    ) -> (Vec<u8>, u32, u32, u32, u32) {
        const BYTES_PER_PIXEL: u32 = 4;
        let grid_color = Color::BLACK;

        let width_px = maze.width as u32 * cell_px;
        let height_px = maze.height as u32 * cell_px;

        let mut buffer = vec![0; (width_px * height_px * BYTES_PER_PIXEL) as usize];

        for y in 0..maze.height as u32 {
            for x in 0..maze.width as u32 {
                let fill_color = match maze.get(x as usize, y as usize) {
                    Cell::Wall => Color::DARK_GRAY,
                    Cell::Empty => Color::LIME,
                };

                let px0 = x * cell_px;
                let py0 = y * cell_px;

                draw_filled_bordered_box(
                    &mut buffer,
                    width_px,
                    height_px,
                    px0,
                    py0,
                    cell_px,
                    cell_px,
                    fill_color,
                    grid_color,
                );
            }
        }

        let maze_size = maze.config.size.grid_size() as u32; 
        let cell_size = width_px/maze_size;
        (buffer, width_px, height_px, maze_size, cell_size)
    }
}

impl Component for MiniMap {
    fn id(&self) -> &str {
        &self.id
    }

    fn bounds(&self) -> Rect {
        self.bounds
    }

    fn layout_metrics(&self) -> LayoutMetrics {
        self.layout
    }

    fn handle_input(
        &mut self,
        _event: &WindowEvent,
        _cursor_pos: Option<(f64, f64)>,
    ) -> Vec<super::super::UIEvent> {
        Vec::new()
    }

    fn apply_update(&mut self, update: &ComponentUpdate) -> bool {
        match update {
            ComponentUpdate::SetMiniMapLayout(id, layout) if *id == self.id => {
                self.bounds = layout.0;
                self.cell_px = layout.1;
                self.player_px = layout.2;

                self.dirty = true;
                true
            }

            ComponentUpdate::SetMaze(id, maze) if *id == self.id => {
                let cell_px = self.cell_px.round() as u32;
                let (buf, w, h, maze_size, cell_size) = Self::build_maze_cache(maze, cell_px);

                self.cell_px = cell_size as f32;
                self.maze_size = maze_size;
                self.maze_cache = buf;
                self.maze_width_px = w;
                self.maze_height_px = h;

                self.dirty = true;
                true
            }

            ComponentUpdate::SetMiniMapPlayer(id, pos) if *id == self.id => {
                self.player_pos = *pos;
                self.dirty = true;
                true
            }

            _ => false,
        }
    }

    fn requires_redraw(&mut self) -> bool {
        let redraw = self.dirty;
        self.dirty = false;
        redraw
    }

    fn draw(&self, frame: &mut [u8], context: &mut UIMainContext) {
        const BYTES_PER_PIXEL: usize = 4;

        let abs = calculate_absolute_rect(&self.layout, &self.bounds, &context.layout);
        let rect = abs.to_int_rect();

        let (frame_width, frame_height) = context.layout.size_as_u32();

        // ---- blit maze cache ----
        for y in 0..self.maze_height_px as usize {
            let dst_y = rect.y as usize + y;
            let src_row = y * self.maze_width_px as usize * BYTES_PER_PIXEL;
            let dst_row = dst_y * frame_width as usize * BYTES_PER_PIXEL
                + rect.x as usize * BYTES_PER_PIXEL;

            let len = self.maze_width_px as usize * BYTES_PER_PIXEL;

            if dst_row + len <= frame.len() {
                frame[dst_row..dst_row + len]
                    .copy_from_slice(&self.maze_cache[src_row..src_row + len]);
            }
        }

        // ---- draw player triangle ----
        let (x, y, angle) = self.player_pos;
        let norm_x = x / 100.0;
        let norm_y = y / 100.0;

        // minimap pixel coordinates
        let cx = rect.x as f32 + norm_x * self.cell_px;
        let cy = rect.y as f32 + norm_y * self.cell_px;

        draw_triangle(
            frame,
            frame_width,
            frame_height,
            cx,
            cy,
            angle,
            self.cell_px/ 2.0,
            Color::RED,
        );
    }

    
    fn get_text(&self) -> &str {
        ""
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
