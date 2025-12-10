use ab_glyph::{ Font, ScaleFont};
use winit::event::{WindowEvent, ElementState as WinitElementState, MouseButton as WinitMouseButton};

use crate::context::UIMainContext;
use crate::components::Component;
use crate::events::{ComponentUpdate, UIEvent};
use crate::geometry::{Bounds, calculate_absolute_rect, IntRect};
use crate::layout::{LayoutMetrics};
use crate::Color;

/// A standard interactive button component.
#[derive(Debug)]
pub struct Button {
    id: String,
    bounds: Bounds,
    layout: LayoutMetrics,
    
    // Visual State
    text: String,
    font_size: f32,
    background_color: Color,
    hover_color: Color,
    text_color: Color,
    
    // Interaction State
    is_hovered: bool,
    is_pressed: bool,
    redraw_required: bool,
}

impl Button {
    pub fn new(id: &str, text: &str, bounds: Bounds, layout: LayoutMetrics, text_color: Color, background_color: Color, hover_color: Color) -> Self {
        Button {
            id: id.to_string(),
            bounds,
            layout,
            text: text.to_string(),
            font_size: 24.0,
            background_color,
            hover_color,
            text_color,
            is_hovered: false,
            is_pressed: false,
            redraw_required: true,
        }
    }

    /// Helper to get the current drawing color based on interaction state.
    fn current_bg_color(&self) -> Color {
        if self.is_hovered {
            self.hover_color
        } else {
            self.background_color
        }
    }
    
    /// Helper to update hover state. Used by UIManager's hit-test/hover logic.
    pub fn set_hovered(&mut self, is_hovered: bool) {
        if self.is_hovered != is_hovered {
            self.is_hovered = is_hovered;
            self.redraw_required = true;
        }
    }
}

impl Component for Button {
    fn id(&self) -> &str {
        &self.id
    }

    fn bounds(&self) -> Bounds {
        self.bounds
    }

    fn layout_metrics(&self) -> LayoutMetrics {
        self.layout
    }

    fn set_focus(&mut self, _is_focused: bool) -> bool {
        false
    }

    fn handle_input(&mut self, event: &WindowEvent) -> Vec<UIEvent> {
        let mut events = Vec::new();

        match event {
            WindowEvent::MouseInput { state, button, .. } => {
                if *button == WinitMouseButton::Left {
                    let old_pressed = self.is_pressed;
                    
                    println!("{} is pressed", self.id);
                    // The button must be hovered to be pressed
                    self.is_pressed = *state == WinitElementState::Pressed && self.is_hovered;

                    // If released and was pressed, and is still hovered, generate a Click event
                    if old_pressed && *state == WinitElementState::Released && self.is_hovered {
                        events.push(UIEvent::ButtonClicked(self.id.clone()));
                    }
                    
                    if old_pressed != self.is_pressed {
                         self.redraw_required = true;
                    }
                }
            },
            _ => {}
        }

        events
    }

    fn apply_update(&mut self, update: &ComponentUpdate) -> bool {
        match update {
            ComponentUpdate::SetText(id, new_text) if id == &self.id => {
                self.text = new_text.clone();
                self.redraw_required = true;
                true
            }
            ComponentUpdate::SetHovered(_id, is_hovered) => {
                if self.is_hovered != *is_hovered {
                    self.is_hovered = *is_hovered;
                    self.redraw_required = true;
                    true
                } else {
                    false
                }
            }
            _ => false,
        }
    }

    fn requires_redraw(&mut self) -> bool {
        // Clear flag after checking
        let required = self.redraw_required;
        self.redraw_required = false; 
        required
    }

    fn draw(&self, frame: &mut [u8], context: &mut UIMainContext, screen_width: u32, screen_height: u32) {
        
        const BPP: usize = 4;
        let draw_width = screen_width as usize;
        
        let bounds_f64 = calculate_absolute_rect(
            &self.layout, 
            &self.bounds, 
            screen_width as f64, 
            screen_height as f64
        );
        
        let abs_rect: IntRect = bounds_f64.to_int_rect();
        let bg_color = self.current_bg_color();
        let text_color = self.text_color;
        
        // 1. Draw Background
        for y in abs_rect.y..(abs_rect.y + abs_rect.h as i32) {
            for x in abs_rect.x..(abs_rect.x + abs_rect.w as i32) {
                if x >= 0 && x < screen_width as i32 && y >= 0 && y < screen_height as i32 {
                    let offset = (y as usize * draw_width + x as usize) * BPP;
                    frame[offset] = bg_color.r;
                    frame[offset + 1] = bg_color.g;
                    frame[offset + 2] = bg_color.b;
                    frame[offset + 3] = 255; 
                }
            }
        }
        
        // 2. Draw Text (Vertically and Horizontally Centered)
        let text_to_draw = self.text.as_str();
        
        let (scaled_ascent, scaled_descent) = {
            let scale = context.font_manager.calculate_scale(self.font_size, context.global_scale_factor);
            let scaled_font = context.font_manager.get_primary_font().as_scaled(scale);
            
            let ascent = (scaled_font.ascent() * context.global_scale_factor) as f64;
            let descent = (scaled_font.descent() * context.global_scale_factor) as f64; 
            (ascent, descent)
        };
        
        // Vertical Centering: Center of box - (offset from baseline to font center)
        let font_center_offset = (scaled_ascent + scaled_descent) / 2.0;
        let component_center_y = bounds_f64.y + bounds_f64.h / 2.0;
        let text_baseline_y = component_center_y - font_center_offset;

        // Horizontal Centering:
        let text_width = context.font_manager.measure_text_width(
            text_to_draw, 
            self.font_size, 
            context.global_scale_factor
        );
        let text_start_x = bounds_f64.x + (bounds_f64.w / 2.0) - (text_width / 2.0);


        context.font_manager.draw_text(
            frame,
            text_to_draw,
            self.font_size,
            text_color,
            text_start_x,
            text_baseline_y,
            context.global_scale_factor,
            screen_width,
            screen_height,
        );
    }
}