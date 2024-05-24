pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
}

impl Color {
    // Create a new Color
    pub fn new(r: f32, g: f32, b: f32) -> Self {
        Color { r, g, b }
    }

    // Method to increment the color around the color wheel
    pub fn increment(&mut self, step: f32) {
        if self.r == 1.0 && self.g < 1.0 && self.b == 0.0 {
            // Red to Yellow (increment green)
            self.g = (self.g + step).min(1.0);
        } else if self.g == 1.0 && self.r > 0.0 && self.b == 0.0 {
            // Yellow to Green (decrement red)
            self.r = (self.r - step).max(0.0);
        } else if self.g == 1.0 && self.b < 1.0 && self.r == 0.0 {
            // Green to Cyan (increment blue)
            self.b = (self.b + step).min(1.0);
        } else if self.b == 1.0 && self.g > 0.0 && self.r == 0.0 {
            // Cyan to Blue (decrement green)
            self.g = (self.g - step).max(0.0);
        } else if self.b == 1.0 && self.r < 1.0 && self.g == 0.0 {
            // Blue to Magenta (increment red)
            self.r = (self.r + step).min(1.0);
        } else if self.r == 1.0 && self.b > 0.0 && self.g == 0.0 {
            // Magenta to Red (decrement blue)
            self.b = (self.b - step).max(0.0);
        }
    }
}
