use std::collections::HashMap;

use crate::context::UIMainContext;
use crate::components::Component;
use crate::events::{UIInputEvent, ComponentUpdate, UIEvent, ElementState};
use crate::events::UIInputEvent::{CursorMoved, MouseButton, KeyboardInput};

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
}

impl UIManager {
    /// Creates a new, empty UIManager.
    pub fn new() -> Self {
        UIManager::default()
    }

    /// Adds a layer to the manager. Returns an error if the ID already exists.
    pub fn add_layer(&mut self, layer: UILayer) -> Result<(), String> {
        if self.layers.contains_key(&layer.id) {
            return Err(format!("Layer with ID '{}' already exists.", layer.id));
        }
        self.layers.insert(layer.id.clone(), layer);
        Ok(())
    }

    /// Processes an abstract input event, performing focus checks, hit-testing, and routing.
    pub fn process_input(&mut self, input_event: &UIInputEvent) -> Vec<UIEvent> {
        let mut generated_events = Vec::new();
        let mut input_consumed = false;
        
        // --- 1. KEYBOARD FOCUS CHECK (High Priority) ---
        if let Some(focused_id) = &self.focused_component_id.clone() {
            if let Some(component) = self.find_component_by_id_mut(focused_id) {
                let events = component.handle_input(input_event);
                generated_events.extend(events);
                
                if matches!(input_event, KeyboardInput { .. }) {
                    input_consumed = true;
                }
            } else {
                self.focused_component_id = None;
            }
        }

        // --- 2. MOUSE EVENT HIT-TESTING & BUBBLING ---
        let mouse_coords = match input_event {
            CursorMoved { x, y } | MouseButton { x, y, .. } => Some((*x, *y)),
            _ => None,
        };

        if let Some((mx, my)) = mouse_coords {
            // Sort layers by Z-index to process input from top to bottom
            let mut sorted_layers: Vec<&mut UILayer> = self.layers.values_mut().collect();
            sorted_layers.sort_by_key(|layer| layer.z_index);
            sorted_layers.reverse();

            for layer in sorted_layers.iter_mut().filter(|l| l.is_visible) {
                // Components within a layer are processed back-to-front (rev)
                for component in layer.components.iter_mut().rev() {
                    let bounds = component.bounds();
                    
                    // FIX: Convert f32 (mx, my) to f64 to match bounds.contains signature
                    if bounds.contains(mx as f64, my as f64) { 
                        self.hovered_component_id = Some(component.id().to_string());
                        
                        // Pass input to the top-most hit component if not already consumed
                        if !input_consumed {
                            let events = component.handle_input(input_event);
                            generated_events.extend(events);
                            input_consumed = true;
                            
                            // Basic Focus Logic: A mouse press on a component gives it focus
                            if matches!(input_event, MouseButton { state, .. } if *state == ElementState::Pressed) {
                                self.focused_component_id = Some(component.id().to_string());
                            }
                        }
                    }
                }
                
                // If the top visible layer is modal OR input was consumed, stop processing layers below.
                if input_consumed || layer.is_modal {
                    break; 
                }
            }
        }
        
        // --- 3. GLOBAL INPUT / UNFOCUS LOGIC ---
        // If a mouse click happened but didn't hit any component, clear focus.
        if !input_consumed && matches!(input_event, MouseButton { state, .. } if *state == ElementState::Pressed) {
             self.focused_component_id = None;
        }

        generated_events
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
    pub fn draw(&mut self, frame: &mut [u8], context: &UIMainContext, width: u32, height: u32) {
        let mut sorted_layers: Vec<&mut UILayer> = self.layers.values_mut().collect();
        sorted_layers.sort_by_key(|layer| layer.z_index);

        for layer in sorted_layers.iter_mut().filter(|l| l.is_visible) {
            for component in layer.components.iter_mut() {
                    component.draw(frame, context, width, height);
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
}
