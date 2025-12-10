use std::fmt::Debug;

// --- The UI Output Contract (Sent to Application) ---

#[derive(Debug, Clone)]
pub enum UIEvent {
    ButtonClicked(String),
    ButtonToggled(String, bool),
    TextSubmitted(String, String),
    ValueChanged(String, f32),
    ExitRequested,
}

#[derive(Debug, Clone)]
pub enum ComponentUpdate {
    SetText(String, String),            // (Component ID, New Text)
    SetValue(String, f32),              // (Component ID, New Value)
    SetLayerVisibility(String, bool),   // (Layer ID, Is Visible)
    SetPosition(String, f64, f64),      // (Component ID, New X, New Y)
    Resize {width: f64, height: f64},
    SetHovered(String, bool),
    SetFocus(String, bool),
}

impl ComponentUpdate {
    /// Returns the target component ID if the update is for a component, otherwise None.
    pub fn get_target_id(&self) -> Option<&str> {
        match self {
            // These variants target a specific component ID
            ComponentUpdate::SetText(id, _) |
            ComponentUpdate::SetValue(id, _) |
            ComponentUpdate::SetPosition(id, _, _) |
            ComponentUpdate::SetFocus(id, _) |
            ComponentUpdate::SetHovered(id, _) => Some(id),
            
            // These variants target layers or are global, and should be handled separately
            ComponentUpdate::SetLayerVisibility(_, _) |
            ComponentUpdate::Resize { .. } => None,
        }
    }
    
    /// Returns the target layer ID if the update is for a layer, otherwise None.
    pub fn get_layer_id(&self) -> Option<&str> {
        match self {
            ComponentUpdate::SetLayerVisibility(id, _) => Some(id),
            _ => None,
        }
    }
}

/// A snapshot of the UI state changes that need to be applied.
#[derive(Debug, Clone)]
pub struct UIStateSnapshot {
    pub updates: Vec<ComponentUpdate>,
}