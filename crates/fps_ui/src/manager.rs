use std::collections::HashMap;

use super::{
    Component, ComponentUpdate, ElementState, MouseButton, UIEvent, UIMainContext, WindowEvent,
    calculate_absolute_rect,
};

/// A container for UI components that shares a common Z-order and visibility state.
#[derive(Debug)]
pub struct Layer {
    pub id: String,
    pub z_index: u32,
    pub is_visible: bool,
    pub is_modal: bool,
    pub components: Vec<Box<dyn Component>>,
}

/// The main entry point for the UI system.
/// Manages the hierarchy of layers and components, routes input, and tracks state.
#[derive(Debug)]
pub struct UIManager {
    context: UIMainContext,
    layers: HashMap<String, Layer>,
    focused_component_id: Option<String>,
    hovered_component_id: Option<String>,
    last_cursor_position: (f64, f64),
}

impl UIManager {
    /// Creates a new UIManager with a reference to the global context.
    pub fn new(context: UIMainContext) -> Self {
        Self {
            context,
            layers: HashMap::new(),
            focused_component_id: None,
            hovered_component_id: None,
            last_cursor_position: (0.0, 0.0),
        }
    }

    /// Adds a layer to the manager. Returns an error if the ID already exists.
    pub fn add_layer(&mut self, layer: Layer) -> Result<(), String> {
        if self.layers.contains_key(&layer.id) {
            return Err(format!("Layer with ID '{}' already exists.", layer.id));
        }
        self.layers.insert(layer.id.clone(), layer);
        Ok(())
    }

    /// Processes input events using **logical coordinates** from AppDriver.
    pub fn process_input(
        &mut self,
        event: &WindowEvent,
        logical_cursor: Option<(f64, f64)>,
    ) -> Vec<UIEvent> {
        use winit::event::WindowEvent::*;

        match event {
            KeyboardInput { .. } => self.handle_keyboard_input(event),
            CursorMoved { .. } => {
                if let Some((lx, ly)) = logical_cursor {
                    self.handle_cursor_movement(lx, ly)
                } else {
                    Vec::new()
                }
            }
            MouseInput { state, button, .. } => {
                if let Some((lx, ly)) = logical_cursor {
                    self.handle_mouse_button(lx, ly, state, button, event)
                } else {
                    Vec::new()
                }
            }
            _ => Vec::new(),
        }
    }

    /// Applies updates from the application to components and layers.
    pub fn apply_updates(&mut self, updates: Vec<ComponentUpdate>) -> bool {
        let mut needs_redraw = false;

        for update in updates {
            if let Some(target_id) = update.get_target_id() {
                if let Some(component) = self.find_component_by_id_mut(target_id) {
                    if component.apply_update(&update) {
                        needs_redraw = true;
                    }
                }
            } else if let Some(layer_id) = update.get_layer_id() {
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

    pub fn update_components(&mut self) {
        for layer in self.layers.values_mut() {
            if layer.is_visible {
                for component in layer.components.iter_mut() {
                    component.update(); // FpsComponent updates itself here
                }
            }
        }
    }

    /// Draw all visible layers using the logical size from context.
    pub fn draw(&mut self, frame: &mut [u8]) {
        let mut sorted_layers: Vec<&mut Layer> = self.layers.values_mut().collect();
        sorted_layers.sort_by_key(|layer| layer.z_index);

        for layer in sorted_layers.iter_mut().filter(|l| l.is_visible) {
            for component in layer.components.iter_mut() {
                component.draw(frame, &mut self.context);
            }
        }
    }

    // --- Internal Helpers ---

    /// Helper to find a mutable reference to a component by its ID.
    pub fn find_component_by_id_mut(&mut self, id: &str) -> Option<&mut Box<dyn Component>> {
        for layer in self.layers.values_mut() {
            if let Some(component) = layer.components.iter_mut().find(|c| c.id() == id) {
                return Some(component);
            }
        }
        None
    }

    fn hit_test_component(&self, x: f64, y: f64) -> Option<String> {
        let mut sorted_layers: Vec<&Layer> = self.layers.values().collect();
        sorted_layers.sort_by_key(|layer| std::cmp::Reverse(layer.z_index));

        for layer in sorted_layers.iter().filter(|l| l.is_visible) {
            for component in layer.components.iter().rev() {
                let abs_rect = calculate_absolute_rect(
                    &component.layout_metrics(),
                    &component.bounds(),
                    &self.context.layout,
                );

                if abs_rect.contains(x, y) {
                    return Some(component.id().to_string());
                }
            }
        }
        None
    }

    fn handle_keyboard_input(&mut self, event: &WindowEvent) -> Vec<UIEvent> {
        let mut events = Vec::new();

        if let Some(focused_id) = &self.focused_component_id.clone() {
            if let Some(comp) = self.find_component_by_id_mut(focused_id) {
                events.extend(comp.handle_input(event));
            } else {
                self.focused_component_id = None;
            }
        }

        events
    }

    fn handle_cursor_movement(&mut self, x: f64, y: f64) -> Vec<UIEvent> {
        let new_hover_id = self.hit_test_component(x, y);
        self.last_cursor_position = (x, y);

        if self.hovered_component_id.as_ref() != new_hover_id.as_ref() {
            if let Some(old_id) = self.hovered_component_id.take() {
                if let Some(comp) = self.find_component_by_id_mut(&old_id) {
                    comp.set_hovered(false);
                }
            }

            if let Some(new_id) = new_hover_id.clone() {
                self.hovered_component_id = Some(new_id.clone());
                if let Some(comp) = self.find_component_by_id_mut(&new_id) {
                    comp.set_hovered(true);
                }
            }
        }

        Vec::new()
    }

    fn handle_mouse_button(
        &mut self,
        x: f64,
        y: f64,
        _state: &ElementState,
        button: &MouseButton,
        raw_event: &WindowEvent,
    ) -> Vec<UIEvent> {
        let mut events = Vec::new();

        if *button == MouseButton::Left {
            let current_focus = self.focused_component_id.clone();
            let hit_id = self.hit_test_component(x, y);

            if hit_id != current_focus {
                if let Some(old_id) = current_focus {
                    if let Some(comp) = self.find_component_by_id_mut(&old_id) {
                        comp.set_focus(false);
                    }
                }

                if let Some(new_id) = hit_id.clone() {
                    self.focused_component_id = Some(new_id.clone());
                    if let Some(comp) = self.find_component_by_id_mut(&new_id) {
                        comp.set_focus(true);
                    }
                } else {
                    self.focused_component_id = None;
                }
            }

            if let Some(hit_id) = hit_id {
                if let Some(comp) = self.find_component_by_id_mut(&hit_id) {
                    events.extend(comp.handle_input(raw_event));
                }
            }
        }

        events
    }

    pub fn set_layer_visibility(&mut self, layer_id: &str, visible: bool) {
        if let Some(layer) = self.layers.get_mut(layer_id) {
            layer.is_visible = visible;
        }
    }
}
