use fps_levels::{
    config::MazeSize,
    maze::{Cell, Maze},
};
use std::any::Any;

use super::super::{
    Color, Component, ComponentUpdate, ElementState, IntRect, LayoutMetrics, MouseButton, Rect,
    UIEvent, UIMainContext, WindowEvent, calculate_absolute_rect,
};

#[derive(Debug)]
pub struct MazeEditor {
    id: String,
    bounds: Rect,
    layout: LayoutMetrics,

    maze: Maze,

    cell_px: f64, // pixel size per cell
    is_focused: bool,
    dirty: bool,

    mouse_down: bool,
    last_cell: Option<(usize, usize)>,
}

impl MazeEditor {
    pub fn new(
        id: impl Into<String>,
        size: MazeSize,
        bounds: Rect,
        layout: LayoutMetrics,
        cell_px: f64,
    ) -> Self {
        let grid = size.room_count() * 3;

        Self {
            id: id.into(),
            bounds,
            layout,
            maze: Maze::new("edited_maze".into(), grid, grid),
            cell_px,
            is_focused: false,
            dirty: true,
            mouse_down: false,
            last_cell: None,
        }
    }

    /// Read-only access for saving into config
    pub fn maze(&self) -> &Maze {
        &self.maze
    }

    fn cell_from_cursor(&self, cursor_x: f64, cursor_y: f64) -> Option<(usize, usize)> {
        let local_x = cursor_x - self.bounds.x;
        let local_y = cursor_y - self.bounds.y;

        if local_x < 0.0 || local_y < 0.0 || local_x >= self.bounds.w || local_y >= self.bounds.h {
            return None;
        }

        let cx = (local_x / self.cell_px) as isize;
        let cy = (local_y / self.cell_px) as isize;

        if self.maze.in_bounds(cx, cy) {
            Some((cx as usize, cy as usize))
        } else {
            None
        }
    }

    fn paint_at_cursor(&mut self, cursor_x: f64, cursor_y: f64) {
        let cell = match self.cell_from_cursor(cursor_x, cursor_y) {
            Some(c) => c,
            None => return,
        };

        // Prevent repeated toggling of the same cell
        if self.last_cell == Some(cell) {
            return;
        }

        self.toggle_cell_at(cell.0, cell.1);
        self.last_cell = Some(cell);
    }

    fn toggle_cell_at(&mut self, x: usize, y: usize) {
        let cell = self.maze.get(x, y);
        let new_cell = match cell {
            Cell::Wall => Cell::Empty,
            Cell::Empty => Cell::Wall,
        };
        self.maze.set(x, y, new_cell);
        self.dirty = true;
    }
}

impl Component for MazeEditor {
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
        event: &WindowEvent,
        cursor_pos: Option<(f64, f64)>,
    ) -> Vec<UIEvent> {
        match event {
            WindowEvent::MouseInput {
                state: ElementState::Pressed,
                button: MouseButton::Left,
                ..
            } => {
                self.mouse_down = true;
                self.last_cell = None;

                if let Some((x, y)) = cursor_pos {
                    self.paint_at_cursor(x, y);
                }
            }

            WindowEvent::MouseInput {
                state: ElementState::Released,
                button: MouseButton::Left,
                ..
            } => {
                self.mouse_down = false;
                self.last_cell = None;
            }

            WindowEvent::CursorMoved { .. } => {
                if self.mouse_down {
                    if let Some((x, y)) = cursor_pos {
                        self.paint_at_cursor(x, y);
                    }
                }
            }

            _ => {}
        }

        Vec::new()
    }

    fn apply_update(&mut self, update: &ComponentUpdate) -> bool {
        match update {
            ComponentUpdate::SetMazeSize(id, size) if *id == self.id => {
                let grid = size.room_count() * 3;
                self.maze = Maze::new("edited_maze".into(), grid, grid);
                self.dirty = true;
                true
            }

            ComponentUpdate::SetFocus(id, focused) if *id == self.id => {
                self.is_focused = *focused;
                false
            }

            ComponentUpdate::SetMaze(id, maze) if *id == self.id => {
                self.maze = maze.clone();
                false
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

        let bounds_f64 = calculate_absolute_rect(&self.layout, &self.bounds, &context.layout);
        let abs_rect: IntRect = bounds_f64.to_int_rect();
        let start_x = abs_rect.x as isize;
        let start_y = abs_rect.y as isize;

        let cell_px = self.cell_px.round() as isize;

        let (draw_width, draw_height) = context.layout.size_as_usize();

        let grid_color = Color::BLACK;

        for my in 0..self.maze.height as isize {
            for mx in 0..self.maze.width as isize {
                let fill_color = match self.maze.get(mx as usize, my as usize) {
                    Cell::Wall => Color::DARK_GRAY,
                    Cell::Empty => Color::LIME,
                };

                let px0 = start_x + mx * cell_px;
                let py0 = start_y + my * cell_px;

                for py in 0..cell_px {
                    let y = py0 + py;
                    if y < 0 || y >= draw_height as isize {
                        continue;
                    }

                    for px in 0..cell_px {
                        let x = px0 + px;
                        if x < 0 || x >= draw_width as isize {
                            continue;
                        }

                        let color = if px == cell_px - 1 || py == cell_px - 1 {
                            grid_color
                        } else {
                            fill_color
                        };

                        let offset = (y as usize * draw_width + x as usize) * BYTES_PER_PIXEL;

                        if offset + 3 < frame.len() {
                            frame[offset] = color.r;
                            frame[offset + 1] = color.g;
                            frame[offset + 2] = color.b;
                            frame[offset + 3] = color.a;
                        }
                    }
                }
            }
        }
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
