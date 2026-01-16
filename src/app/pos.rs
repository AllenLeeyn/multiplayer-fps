//! # Position Module
//!
//! Provides a position structure for representing 2D coordinates and rotation
//! in the game world. Includes utilities for movement and rotation.

/// Represents a position and rotation in the game world.
///
/// Uses a 2D coordinate system (x, y) with an angle for rotation.
/// All values are in world units (meters or arbitrary units).
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Pos {
    /// X coordinate in world space.
    pub x: f32,
    
    /// Y coordinate in world space.
    pub y: f32,
    
    /// Rotation angle in radians (0 = facing right, increases counterclockwise).
    pub angle: f32,
}

impl Pos {
    /// Resets position and rotation to origin.
    ///
    /// Sets x and y to 0.0 and angle to 0.0 (facing right).
    pub fn reset(&mut self) {
        self.x = 0.0;
        self.y = 0.0;
        self.angle = 0.0;
    }

    /// Converts position to a tuple format.
    ///
    /// Useful for serialization or passing to functions expecting tuples.
    ///
    /// # Returns
    ///
    /// A tuple of `(x, y, angle)`.
    pub fn to_tuple(&self) -> (f32, f32, f32) {
        (self.x, self.y, self.angle)
    }

    /// Moves forward in the direction of the current angle.
    ///
    /// # Arguments
    ///
    /// * `distance` - Distance to move in world units
    pub fn move_forward(&mut self, distance: f32) {
        self.x += self.angle.cos() * distance;
        self.y += self.angle.sin() * distance;
    }

    /// Moves backward (opposite to the current angle).
    ///
    /// # Arguments
    ///
    /// * `distance` - Distance to move in world units
    pub fn move_backward(&mut self, distance: f32) {
        self.x -= self.angle.cos() * distance;
        self.y -= self.angle.sin() * distance;
    }

    /// Moves sideways (perpendicular to the current angle).
    ///
    /// Strafe movement is commonly used for sidestepping.
    /// Moves 90 degrees to the right of the current facing direction.
    ///
    /// # Arguments
    ///
    /// * `distance` - Distance to move in world units (positive = right, negative = left)
    pub fn strafe(&mut self, distance: f32) {
        let a = self.angle + std::f32::consts::FRAC_PI_2;
        self.x += a.cos() * distance;
        self.y += a.sin() * distance;
    }

    /// Rotates the position by a delta angle.
    ///
    /// Applies rotation sensitivity scaling and normalizes the angle
    /// to the range [0, 2π).
    ///
    /// # Arguments
    ///
    /// * `delta` - Raw rotation delta (typically mouse movement)
    pub fn rotate(&mut self, delta: f32) {
        use crate::app::constants::player::ROTATION_SENSITIVITY;
        self.angle += delta * ROTATION_SENSITIVITY;

        // Normalize angle to [0, 2π)
        if self.angle < 0.0 {
            self.angle += std::f32::consts::TAU;
        } else if self.angle >= std::f32::consts::TAU {
            self.angle -= std::f32::consts::TAU;
        }
    }
}
