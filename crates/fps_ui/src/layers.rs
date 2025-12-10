use std::collections::HashMap;

use super::{
    UIMainContext,
    Component,
    ComponentUpdate,
    UIEvent,
    WindowEvent,
    ElementState,
    MouseButton
};

/// A container for UI components that shares a common Z-order and visibility state.
#[derive(Debug)]
pub struct UILayer {
    pub id: String,
    pub z_index: u32,
    pub is_visible: bool,
    pub is_modal: bool,
    pub components: Vec<Box<dyn Component>>,
}

/// The main entry point for the UI system.
/// Manages the hierarchy of layers and components, routes input, and tracks state.
#[derive(Debug, Default)]
pub struct UIManager {
    layers: HashMap<String, UILayer>,
    focused_component_id: Option<String>,
    hovered_component_id: Option<String>,
    width: f64,
    height: f64,
    last_cursor_position: (f64, f64),
}

impl UIManager {
    /// Creates a new, empty UIManager.
    pub fn new(width: u32, height: u32) -> Self {
        let mut m = UIManager::default();
        m.width = width as f64;
        m.height = height as f64;
        m
    }

    /// Adds a layer to the manager. Returns an error if the ID already exists.
    pub fn add_layer(&mut self, layer: UILayer) -> Result<(), String> {
        if self.layers.contains_key(&layer.id) {
            return Err(format!("Layer with ID '{}' already exists.", layer.id));
        }
        self.layers.insert(layer.id.clone(), layer);
        Ok(())
    }

    /// Processes an abstract input event by dispatching it to dedicated handlers.
    pub fn process_input(&mut self, input_event: &WindowEvent) -> Vec<UIEvent> {
        use winit::event::WindowEvent::*;

        match input_event {
            // High-priority text and key routing to the focused component
            KeyboardInput { .. }  => {
                self.handle_keyboard_input(input_event)
            },

            // Mouse movement and hover
            WindowEvent::CursorMoved { position, .. } => {
                self.handle_cursor_movement(position.x, position.y)
            },
            
            // Mouse button for focus change and click routing
            MouseInput { state, button, .. } => {
                let (x, y) = self.last_cursor_position;
                self.handle_mouse_button(x, y, state, button, input_event)
            },

            // Ignore other input types for now (e.g., MouseWheel, Touch)
            _ => Vec::new(),
        }
    }

    /// Applies updates from the application to the corresponding components and layers.
    /// Returns true if any component triggered a redraw flag.
    pub fn apply_updates(&mut self, updates: Vec<ComponentUpdate>) -> bool {
        let mut needs_redraw = false;

        for update in updates {
            
            // 1. Check for Component Updates
            if let Some(target_id) = update.get_target_id() { 
                if let Some(component) = self.find_component_by_id_mut(target_id) {
                    // component.apply_update now correctly returns a bool
                    if component.apply_update(&update) { 
                        needs_redraw = true;
                    }
                }
            } 
            
            // 2. Check for Layer Updates (SetLayerVisibility)
            else if let Some(layer_id) = update.get_layer_id() {
                if let Some(layer) = self.layers.get_mut(layer_id) {
                    if let ComponentUpdate::SetLayerVisibility(_, is_visible) = update {
                        layer.is_visible = is_visible;
                        needs_redraw = true;
                    }
                }
            }
        }
        needs_redraw
    }

    /// Draws all visible layers and their components onto the pixel frame buffer.
    /// Called by the AppDriver during the RedrawRequested event.
    pub fn draw(&mut self, frame: &mut [u8], context: &mut UIMainContext) {
        let mut sorted_layers: Vec<&mut UILayer> = self.layers.values_mut().collect();
        sorted_layers.sort_by_key(|layer| layer.z_index);

        for layer in sorted_layers.iter_mut().filter(|l| l.is_visible) {
            for component in layer.components.iter_mut() {
                    component.draw(frame, context, self.width as u32, self.height as u32);
            }
        }
    }

    // --- Internal Helpers ---

    /// Helper to find a mutable reference to a component by its ID.
    fn find_component_by_id_mut(&mut self, id: &str) -> Option<&mut Box<dyn Component>> {
        for layer in self.layers.values_mut() {
            if let Some(component) = layer.components.iter_mut().find(|c| c.id() == id) {
                return Some(component);
            }
        }
        None
    }

    /// Helper to perform a reverse Z-order hit test to find the top-most component at (x, y).
    /// This method assumes it has access to the current screen dimensions (e.g., from AppDriver).
    fn hit_test_component(&self, x: f64, y: f64) -> Option<String> {
        let mut sorted_layers: Vec<&UILayer> = self.layers.values().collect();
        // Sort in reverse Z-order (highest Z-index first)
        sorted_layers.sort_by_key(|layer| std::cmp::Reverse(layer.z_index));

        for layer in sorted_layers.iter().filter(|l| l.is_visible) {
            // Components are hit-tested in reverse order of definition (last added is on top)
            for component in layer.components.iter().rev() {
                let bounds = component.bounds();
                let layout = component.layout_metrics();
                
                let abs_rect = crate::geometry::calculate_absolute_rect(
                    &layout, 
                    &bounds, 
                    self.width, 
                    self.height,
                );
                
                // Use Rect/Bounds contains method from geometry.rs
                if abs_rect.contains(x, y) {
                    return Some(component.id().to_string());
                }
            }
        }
        None
    }

    /// Handles all KeyboardInput and ReceivedCharacter events by routing to the focused component.
    fn handle_keyboard_input(&mut self, event: &WindowEvent) -> Vec<UIEvent> {
        let mut generated_events = Vec::new();
        
        if let Some(focused_id) = &self.focused_component_id.clone() {
            if let Some(component) = self.find_component_by_id_mut(focused_id) {
                generated_events.extend(component.handle_input(event));
            } else {
                // Focused component disappeared, clear focus
                self.focused_component_id = None;
            }
        }
        
        generated_events
    }

    /// Handles CursorMoved events, managing hover state and routing the event.
    fn handle_cursor_movement(&mut self, x: f64, y: f64) -> Vec<UIEvent> {
        // Find the top-most component at (x, y)
        let hit_target_id = self.hit_test_component(x, y);
        self.last_cursor_position = (x, y);
        self.hovered_component_id = hit_target_id;
        Vec::new()
    }

    /// Handles MouseButton events, managing focus and routing the click.
    fn handle_mouse_button(&mut self, x: f64, y: f64, state: &ElementState, button: &MouseButton, raw_event:&WindowEvent ) -> Vec<UIEvent> {
        let mut generated_events = Vec::new();

        println!("click on {} {}", x, y);
        if *state == ElementState::Pressed && *button == MouseButton::Left {
            let current_focus_id = self.focused_component_id.clone();
            let hit_target_id = self.hit_test_component(x, y);

            if hit_target_id.as_ref() != current_focus_id.as_ref() {
                if let Some(old_id) = current_focus_id {
                    if let Some(old_component) = self.find_component_by_id_mut(&old_id) {
                        old_component.set_focus(false);
                    }
                }
                
                // Focus new
                if let Some(new_id) = hit_target_id.clone() {
                    self.focused_component_id = Some(new_id.clone()); 
                    if let Some(new_component) = self.find_component_by_id_mut(&new_id) {
                        new_component.set_focus(true);
                    }
                } else {
                    // Clicked outside
                    self.focused_component_id = None;
                }
            }
            // B. Route the click event to the component that was hit (if one exists)

            if let Some(hit_id) = hit_target_id {
                if let Some(component) = self.find_component_by_id_mut(&hit_id) {
                    generated_events.extend(component.handle_input(raw_event));
                }
            }
        }
        
        generated_events
    }
}
