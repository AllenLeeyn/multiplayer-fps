use ab_glyph::{ Font, ScaleFont};

use super::super::{
    Color,
    Bounds,
    calculate_absolute_rect,
    UIMainContext,
    Component,
    ComponentUpdate,
    UIEvent,
    LayoutMetrics,
    WindowEvent,
    ElementState,
    MouseButton,
    IntRect,
    KeyCode,
    PhysicalKey
};

/// A standard single-line text input field component.
#[derive(Debug)]
pub struct TextInput {
    id: String,
    bounds: Bounds,
    layout: LayoutMetrics,
    
    // State
    pub text: String,
    pub placeholder: String,
    pub max_input: usize,
    // Note: Focus state is ultimately controlled by UIManager, but component needs to know.
    pub is_focused: bool, 
    cursor_index: usize,
    cursor_visible: bool,
    redraw_required: bool,
    
    // Appearance
    font_size: f32,
    text_color: Color,
    background_color: Color,
    focus_color: Color,
}

impl TextInput {
    /// Creates a new TextInput component.
    pub fn new(id: String, bounds: Bounds, layout: LayoutMetrics, placeholder: String, max_input: usize, color: Color, bg_color: Color) -> Self {
        TextInput {
            id,
            bounds,
            layout,
            text: String::new(),
            placeholder,
            max_input,
            is_focused: false,
            cursor_index: 0,
            cursor_visible: true,
            redraw_required: true,
            font_size: 30.0,
            text_color: color,
            background_color: bg_color,
            focus_color: bg_color,
        }
    }

    fn text_to_draw(&self) -> &str { 
        // Only show placeholder if text is empty AND placeholder is not empty
        if self.text.is_empty() && !self.placeholder.is_empty() {
            self.placeholder.as_str()
        } else {
            self.text.as_str()
        }
    }
}

impl Component for TextInput {
    fn id(&self) -> &str {
        &self.id
    }

    fn bounds(&self) -> Bounds {
        self.bounds
    }

    fn layout_metrics(&self) -> LayoutMetrics {
        self.layout
    }
    
    fn requires_redraw(&mut self) -> bool {
        let needs_redraw = self.redraw_required;
        self.redraw_required = false;

        // Force redraw when focused to toggle cursor visibility
        if self.is_focused {
            self.cursor_visible = !self.cursor_visible; 
            return true;
        }
        needs_redraw
    }

    fn handle_input(&mut self, event: &WindowEvent) -> Vec<UIEvent> {
        let mut events = Vec::new();

        if !self.is_focused {
            return events;
        }

        self.cursor_visible = true;

        match event {
            WindowEvent::KeyboardInput { 
                event, // The winit::event::KeyEvent struct
                is_synthetic: _, 
                .. 
            } => {
                // We typically only want to process input/control on the 'Pressed' state
                if event.state == ElementState::Pressed { 
                    self.redraw_required = true;
                    
                    // --- 1. Handle TEXT INSERTION (Replaces ReceivedCharacter) ---
                    // Check the optional `text` field for the actual characters typed
                    if self.text.len() < self.max_input{
                        if let Some(text_to_insert) = event.text.as_ref() {
                            // Insert the characters one by one (SmolStr/String implements Iterator<Item=char>)
                            for c in text_to_insert.chars() {
                                // Filter out control characters (like Enter, Tab, etc.) 
                                // as they should be handled by the physical key below.
                                if c.is_ascii() && !c.is_control() {
                                    self.text.insert(self.cursor_index, c);
                                    self.cursor_index += 1;
                                    self.cursor_visible = true;
                                }
                            }
                        }
                    }

                    // --- 2. Handle CONTROL KEYS (Physical Keys) ---
                    match event.physical_key {
                        PhysicalKey::Code(KeyCode::Backspace) => { 
                            if self.cursor_index > 0 {
                                self.text.remove(self.cursor_index - 1);
                                self.cursor_index -= 1;
                            }
                        },
                        PhysicalKey::Code(KeyCode::Delete) => {
                            if self.cursor_index < self.text.len() {
                                self.text.remove(self.cursor_index);
                            }
                        },
                        PhysicalKey::Code(KeyCode::Enter) => { 
                            self.redraw_required = true;
                            events.push(UIEvent::TextSubmitted(self.id.clone(), self.text.clone()));
                        },
                        // Cursor Movement
                        PhysicalKey::Code(KeyCode::ArrowLeft) => { 
                            self.cursor_index = self.cursor_index.saturating_sub(1);
                            self.cursor_visible = true;
                        },
                        PhysicalKey::Code(KeyCode::ArrowRight) => { 
                            self.cursor_index = (self.cursor_index + 1).min(self.text.len());
                            self.cursor_visible = true;
                        },
                        _ => {} // Ignore other key presses
                    }
                }
            },
            
            // 3. Winit::MouseInput (for click-based focus handling)
            WindowEvent::MouseInput { state: ElementState::Pressed, button, .. } => {
                if *button == MouseButton::Left {
                    self.cursor_index = self.text.len(); 
                    self.cursor_visible = true;
                    self.redraw_required = true;
                }
            }
            
            _ => {} // Ignore other input events
        }
        
        events
    }

    fn apply_update(&mut self, update: &ComponentUpdate) -> bool {
        match update {
            ComponentUpdate::SetText(_, new_text) => {
                self.text = new_text.clone();
                // Move cursor to the end when text is programmatically set
                self.cursor_index = self.text.len(); 
                self.redraw_required = true;
                true
            },
            ComponentUpdate::SetFocus(_, is_focused) => {
                if self.is_focused != *is_focused {
                    self.is_focused = *is_focused;
                    self.redraw_required = true;

                    // Reset cursor state when gaining focus
                    if self.is_focused {
                        self.cursor_visible = true;
                        self.cursor_index = self.text.len(); 
                    }
                    return true;
                }
                false
            },
            _ => false
        }
    }

    fn draw(&self, frame: &mut [u8], context: &mut UIMainContext, screen_width: u32, screen_height: u32) {
        
        const BPP: usize = 4;
        let draw_width = screen_width as usize;
        
        // Use the geometry module to get the final screen bounds
        let bounds_f64 = calculate_absolute_rect(
            &self.layout, 
            &self.bounds, 
            screen_width as f64, 
            screen_height as f64
        );
        
        // Pixel-snap the bounds for drawing
        let abs_rect: IntRect = bounds_f64.to_int_rect();
        
        // 1. Determine Colors and Text
        let bg_color = if self.is_focused { self.focus_color } else { self.background_color };
        let text_to_draw = if self.text.is_empty() && !self.is_focused { 
            self.placeholder.as_str()
        } else { 
            self.text.as_str() 
        };
        let text_color = if self.text.is_empty() && !self.is_focused {
            // Use a lighter gray for placeholder text
            Color::new(150, 150, 150, 255) 
        } else { 
            self.text_color 
        };
        
        // 2. Draw Background
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
        
        // 3. Draw Text
        let padding_x = 5.0; // Left padding
        let text_start_x = bounds_f64.x + padding_x;

        // Calculate baseline for vertical centering
        let (ascent, line_height) = {
            let scale = context.font_manager.calculate_scale(self.font_size, context.global_scale_factor);
            
            // This creates an immutable reference to the FontArc inside FontManager.
            let scaled_font = context.font_manager.get_primary_font().as_scaled(scale);
            
            // Extract the required f64 values immediately.
            let ascent = scaled_font.ascent() as f64;
            let line_height = (scaled_font.height() as f64) * context.global_scale_factor as f64;
            
            // `scaled_font` drops here, releasing the immutable borrow.
            (ascent, line_height)
        };
        let text_baseline_y = bounds_f64.y - ascent + line_height;

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
        
        // 4. Draw Cursor (If focused and visible, and not drawing placeholder)
        if self.is_focused && self.cursor_visible && self.text_to_draw() == self.text.as_str() {
            
            // Measure the text up to the cursor index using the new FontManager method
            let text_before_cursor = &self.text[0..self.cursor_index];
            let advance_x = context.font_manager.measure_text_width(
                text_before_cursor, 
                self.font_size, 
                context.global_scale_factor
            );
            
            // Cursor position
            let cursor_x = (text_start_x + advance_x) as i32;
            let cursor_w = 2; // Cursor width in pixels
            
            let cursor_h = (line_height * 0.8) as i32; // 80% of line height
            
            let cursor_y_center = abs_rect.y + abs_rect.h as i32 / 2;
            let cursor_y = cursor_y_center - cursor_h / 2;
            
            let cursor_color = Color::WHITE;
            
            // Simplified Cursor Drawing
            for y in cursor_y..(cursor_y + cursor_h) {
                for x in cursor_x..(cursor_x + cursor_w) {
                    // Check bounds against the component's visible area
                    if x >= abs_rect.x && x < (abs_rect.x + abs_rect.w as i32) &&
                       y >= abs_rect.y && y < (abs_rect.y + abs_rect.h as i32) {
                        
                        let offset = (y as usize * draw_width + x as usize) * BPP;
                        
                        // Draw opaque white cursor
                        frame[offset] = cursor_color.r;
                        frame[offset + 1] = cursor_color.g;
                        frame[offset + 2] = cursor_color.b;
                        frame[offset + 3] = 255; 
                    }
                }
            }
        }
    }
}
