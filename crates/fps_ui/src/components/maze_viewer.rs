use fps_levels::maze::{Cell, Maze};
use std::any::Any;

use super::super::{
    Color, Component, ComponentUpdate, IntRect, LayoutMetrics, Rect, UIMainContext, WindowEvent,
    calculate_absolute_rect, draw_filled_bordered_box,
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
        let bounds_f64 = calculate_absolute_rect(&self.layout, &self.bounds, &context.layout);
        let abs_rect: IntRect = bounds_f64.to_int_rect();

        let start_x = abs_rect.x as u32;
        let start_y = abs_rect.y as u32;

        let cell_px = self.cell_px.round() as u32;
        let (draw_width, draw_height) = context.layout.size_as_u32();

        let grid_color = Color::BLACK;

        for y in 0..self.maze.height as u32 {
            for x in 0..self.maze.width as u32 {
                let fill_color = match self.maze.get(x as usize, y as usize) {
                    Cell::Wall => Color::DARK_GRAY,
                    Cell::Empty => Color::LIME,
                };

                let px0 = start_x + x * cell_px;
                let py0 = start_y + y * cell_px;

                draw_filled_bordered_box(
                    frame,
                    draw_width,
                    draw_height,
                    px0,
                    py0,
                    cell_px,
                    cell_px,
                    fill_color,
                    grid_color,
                );
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
