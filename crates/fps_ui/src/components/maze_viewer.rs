use fps_levels::maze::{Cell, Maze};
use std::any::Any;

use super::super::{
    Color, Component, ComponentUpdate, IntRect, LayoutMetrics, Rect, UIMainContext, WindowEvent,
    calculate_absolute_rect,
};

#[derive(Debug)]
pub struct MazeView {
    id: String,
    bounds: Rect,
    layout: LayoutMetrics,

    maze: Maze,
    cell_px: f64,

    dirty: bool,
}

impl MazeView {
    pub fn new(
        id: impl Into<String>,
        maze: Maze,
        bounds: Rect,
        layout: LayoutMetrics,
        cell_px: f64,
    ) -> Self {
        Self {
            id: id.into(),
            bounds,
            layout,
            maze,
            cell_px,
            dirty: true,
        }
    }
}

impl Component for MazeView {
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
        // Read-only: no input handling
        Vec::new()
    }

    fn apply_update(&mut self, update: &ComponentUpdate) -> bool {
        match update {
            ComponentUpdate::SetMaze(id, maze) if *id == self.id => {
                self.maze = maze.clone();
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

        let bounds_f64 = calculate_absolute_rect(&self.layout, &self.bounds, &context.layout);
        let abs_rect: IntRect = bounds_f64.to_int_rect();

        let start_x = abs_rect.x as isize;
        let start_y = abs_rect.y as isize;

        let cell_px = self.cell_px.round() as isize;
        let (draw_width, draw_height) = context.layout.size_as_usize();

        let grid_color = Color::BLACK;

        for y in 0..self.maze.height as isize {
            for x in 0..self.maze.width as isize {
                let fill_color = match self.maze.get(x as usize, y as usize) {
                    Cell::Wall => Color::DARK_GRAY,
                    Cell::Empty => Color::LIME,
                };

                let px0 = start_x + x * cell_px;
                let py0 = start_y + y * cell_px;

                for py in 0..cell_px {
                    let sy = py0 + py;
                    if sy < 0 || sy >= draw_height as isize {
                        continue;
                    }

                    for px in 0..cell_px {
                        let sx = px0 + px;
                        if sx < 0 || sx >= draw_width as isize {
                            continue;
                        }

                        let color = if px == cell_px - 1 || py == cell_px - 1 {
                            grid_color
                        } else {
                            fill_color
                        };

                        let offset = (sy as usize * draw_width + sx as usize) * BYTES_PER_PIXEL;

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
