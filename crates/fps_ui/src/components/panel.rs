//! # Panel Component
//!
//! A basic rectangular panel component used for backgrounds, containers, and full-screen overlays.
//! Supports solid colors, textures, and custom rendering functions.

use std::any::Any;
use std::fmt::{Debug, Formatter, Result};

use super::super::{
    Color, Component, ComponentUpdate, IntRect, LayoutMetrics, Rect, UIEvent, UIMainContext, WindowEvent, calculate_absolute_rect, draw_filled_box
};

/// Defines the rendering source for a panel.
///
/// Panels can render from solid colors, loaded textures, or custom functions.
pub enum RenderSource {
    /// Fills the panel with a solid color.
    SolidColor(Color),

    /// Renders a texture loaded in `UIMainContext`.
    ///
    /// The string is the texture ID used when loading the texture.
    Image(String),

    /// Renders using a custom function.
    ///
    /// The function receives normalized coordinates (0.0 to 1.0) relative to
    /// the panel's bounds and returns a color for that position.
    Function(Box<dyn Fn(f64, f64) -> Color + Send + Sync>),
}

impl Debug for RenderSource {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            RenderSource::SolidColor(c) => f.debug_tuple("SolidColor").field(c).finish(),
            RenderSource::Image(path) => f.debug_tuple("Image").field(path).finish(),
            // The closure itself cannot be debugged, so we use a placeholder description.
            RenderSource::Function(_) => f.write_str("Function(<dynamic>)"),
        }
    }
}

/// A rectangular panel component for backgrounds and containers.
///
/// Panels are simple rectangular components that can render solid colors,
/// textures, or custom-generated content. They're typically used as backgrounds
/// or container elements.
///
/// # Example
///
/// ```rust,no_run
/// use fps_ui::components::{Panel, RenderSource};
/// use fps_ui::{Color, Rect};
///
/// // Solid color background
/// let panel = Panel::new(
///     "background".to_string(),
///     RenderSource::SolidColor(Color::BLUE),
///     Rect::new(0.0, 0.0, 800.0, 600.0),
/// );
///
/// // Texture background
/// let texture_panel = Panel::new(
///     "bg_texture".to_string(),
///     RenderSource::Image("background_texture".to_string()),
///     Rect::new(0.0, 0.0, 800.0, 600.0),
/// );
/// ```
#[derive(Debug)]
pub struct Panel {
    id: String,
    bounds: Rect,
    layout: LayoutMetrics,
    source: RenderSource,
    needs_redraw: bool,
}

impl Panel {
    /// Creates a new panel component.
    ///
    /// # Arguments
    ///
    /// * `id` - Unique identifier for the panel
    /// * `source` - Rendering source (color, texture, or function)
    /// * `bounds` - Panel's bounding rectangle (relative coordinates)
    pub fn new(id: String, source: RenderSource, bounds: Rect) -> Self {
        Panel {
            id,
            bounds,
            layout: LayoutMetrics::default(),
            source,
            needs_redraw: true,
        }
    }

    /// Creates a new panel component with custom layout metrics.
    ///
    /// # Arguments
    ///
    /// * `id` - Unique identifier for the panel
    /// * `source` - Rendering source (color, texture, or function)
    /// * `bounds` - Panel's bounding rectangle (relative coordinates)
    /// * `layout` - Layout metrics for positioning
    pub fn new_with_layout(id: String, source: RenderSource, bounds: Rect, layout: LayoutMetrics) -> Self {
        Panel {
            id,
            bounds,
            layout,
            source,
            needs_redraw: true,
        }
    }
}

// --- Component Trait Implementation ---

impl Component for Panel {
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
    ) -> Vec<UIEvent> {
        Vec::new()
    }

    fn draw(&self, frame: &mut [u8], context: &mut UIMainContext) {
        let bounds_f64 = calculate_absolute_rect(&self.layout_metrics(), &self.bounds, &context.layout);
        let abs_rect: IntRect = bounds_f64.to_int_rect();
        let (logical_width, logical_height) = context.layout.size_as_u32();

        let color = match &self.source {
            RenderSource::SolidColor(c) => *c,
            _ => Color::RED,
        };

        draw_filled_box(
            frame,
            logical_width,
            logical_height,
            abs_rect.x as u32,
            abs_rect.y as u32,
            abs_rect.w as u32,
            abs_rect.h as u32,
            color,
        );
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
