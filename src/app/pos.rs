#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Pos {
    pub x: f32,
    pub y: f32,
    pub angle: f32,
}

impl Pos {
    pub fn reset(&mut self) {
        self.x = 0.0;
        self.y = 0.0;
        self.angle = 0.0;
    }
    
    pub fn to_tuple(&self) -> (f32, f32, f32) {
        (self.x, self.y, self.angle)
    }

    pub fn move_forward(&mut self, distance: f32) {
        self.x += self.angle.cos() * distance;
        self.y += self.angle.sin() * distance;
    }

    pub fn move_backward(&mut self, distance: f32) {
        self.x -= self.angle.cos() * distance;
        self.y -= self.angle.sin() * distance;
    }

    pub fn strafe(&mut self, distance: f32) {
        let a = self.angle + std::f32::consts::FRAC_PI_2;
        self.x += a.cos() * distance;
        self.y += a.sin() * distance;
    }

    pub fn rotate(&mut self, delta: f32) {
        self.angle += delta * 0.05;

        // Optional normalization
        if self.angle < 0.0 {
            self.angle += std::f32::consts::TAU;
        } else if self.angle >= std::f32::consts::TAU {
            self.angle -= std::f32::consts::TAU;
        }
    }
}
