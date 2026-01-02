use std::any::Any;

use super::super::{
    Color, Component, ComponentUpdate, IntRect, LayoutMetrics, Rect, UIEvent, UIMainContext,
    WindowEvent, calculate_absolute_rect,
};

#[derive(Debug)]
pub struct TextBox {
    id: String,
    bounds: Rect,
    layout: LayoutMetrics,

    // Content
    lines: Vec<String>,

    // Scrolling
    scroll_offset: f32,
    line_height: f32,

    // State
    hovered: bool,
    redraw_required: bool,

    // Appearance
    font_size: f32,
    text_color: Color,
    background_color: Color,
}

impl TextBox {
    pub fn new(
        id: String,
        bounds: Rect,
        layout: LayoutMetrics,
        font_size: f32,
        text_color: Color,
        background_color: Color,
    ) -> Self {
        Self {
            id,
            bounds,
            layout,
            lines: Vec::new(),
            scroll_offset: 0.0,
            line_height: font_size * 1.3, // sane default
            hovered: false,
            redraw_required: true,
            font_size,
            text_color,
            background_color,
        }
    }

    fn max_scroll(&self, visible_height: f32) -> f32 {
        let content_height = self.lines.len() as f32 * self.line_height;
        (content_height - visible_height).max(0.0)
    }
}

impl Component for TextBox {
    fn id(&self) -> &str {
        &self.id
    }

    fn bounds(&self) -> Rect {
        self.bounds
    }

    fn layout_metrics(&self) -> LayoutMetrics {
        self.layout
    }

    fn requires_redraw(&mut self) -> bool {
        let r = self.redraw_required;
        self.redraw_required = false;
        r
    }

    fn handle_input(
        &mut self,
        event: &WindowEvent,
        _cursor_pos: Option<(f64, f64)>,
    ) -> Vec<UIEvent> {
        match event {
            WindowEvent::MouseWheel { delta, .. } if self.hovered => {
                let scroll_delta = match delta {
                    winit::event::MouseScrollDelta::LineDelta(_, y) => y * self.line_height,
                    winit::event::MouseScrollDelta::PixelDelta(p) => p.y as f32,
                };

                self.scroll_offset -= scroll_delta;
                self.redraw_required = true;
            }
            _ => {}
        }

        Vec::new()
    }

    fn apply_update(&mut self, update: &ComponentUpdate) -> bool {
        let mut _changed = false;

        match update {
            ComponentUpdate::SetText(_, text) => {
                self.lines = text.lines().map(|s| s.to_string()).collect();
                _changed = true;
            }

            ComponentUpdate::SetTextVec(_, lines) => {
                self.lines = lines.clone();
                _changed = true;
            }

            ComponentUpdate::AppendText(_, text) => {
                self.lines.extend(text.lines().map(|s| s.to_string()));
                _changed = true;
            }

            ComponentUpdate::AppendTextVec(_, lines) => {
                self.lines.extend(lines.iter().cloned());
                _changed = true;
            }

            ComponentUpdate::SetHovered(_, hovered) => {
                self.hovered = *hovered;
                return false;
            }

            _ => return false,
        }

        if _changed {
            // Scroll to bottom (actual clamp happens in draw())
            self.scroll_offset = f32::MAX;
            self.redraw_required = true;
        }

        true
    }

    fn draw(&self, frame: &mut [u8], context: &mut UIMainContext) {
        const BPP: usize = 4;

        let bounds_f64 = calculate_absolute_rect(&self.layout, &self.bounds, &context.layout);
        let abs_rect: IntRect = bounds_f64.to_int_rect();

        let (logical_width, logical_height) = context.layout.size_as_u32();
        let draw_width = logical_width as usize;

        // --- Background ---
        for y in abs_rect.y..(abs_rect.y + abs_rect.h as i32) {
            for x in abs_rect.x..(abs_rect.x + abs_rect.w as i32) {
                if x >= 0 && y >= 0 && x < logical_width as i32 && y < logical_height as i32 {
                    let o = (y as usize * draw_width + x as usize) * BPP;
                    frame[o] = self.background_color.r;
                    frame[o + 1] = self.background_color.g;
                    frame[o + 2] = self.background_color.b;
                    frame[o + 3] = 255;
                }
            }
        }

        // --- Clamp scroll ---
        let max_scroll = self.max_scroll(bounds_f64.h as f32);
        let scroll = self.scroll_offset.clamp(0.0, max_scroll);

        // --- Visible range ---
        let line_height = self.line_height as f64;
        let first_line = (scroll / self.line_height).ceil() as usize;
        let y_offset = -(scroll % self.line_height) as f64;

        // --- Draw lines ---
        for (i, line) in self.lines.iter().skip(first_line).enumerate() {
            let y = bounds_f64.y + y_offset + i as f64 * line_height;
            if y + line_height as f64 > bounds_f64.y + bounds_f64.h {
                break;
            }

            context.font_manager.draw_text(
                frame,
                line,
                self.font_size,
                self.text_color,
                bounds_f64.x + 6.0,
                y + self.font_size as f64,
                logical_width,
                logical_height,
            );
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
