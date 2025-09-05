use crate::math_utils::{Vec3, Color, Lerp};

pub struct Skybox {
    pub top_color: Color,
    pub horizon_color: Color,
    pub bottom_color: Color,
    pub sun_direction: Vec3,
    pub sun_color: Color,
    pub sun_size: f32,
}

impl Skybox {
    pub fn new() -> Self {
        Self {
            top_color: Color::new(0.5, 0.7, 1.0),     // Light blue
            horizon_color: Color::new(0.8, 0.9, 1.0), // Nearly white
            bottom_color: Color::new(0.6, 0.8, 0.9),  // Pale blue
            sun_direction: Vec3::new(0.3, 0.6, 0.4).normalize(),
            sun_color: Color::new(1.0, 0.9, 0.7),     // Warm yellow
            sun_size: 0.02,
        }
    }
    
    pub fn sample(&self, direction: Vec3) -> Color {
        let dir = direction.normalize();
        
        // Calculate the vertical gradient
        let t = (dir.y + 1.0) * 0.5; // Map from [-1, 1] to [0, 1]
        
        let sky_color = if t > 0.5 {
            // Upper hemisphere: interpolate between horizon and top
            let upper_t = (t - 0.5) * 2.0;
            self.horizon_color.lerp(&self.top_color, upper_t)
        } else {
            // Lower hemisphere: interpolate between bottom and horizon
            let lower_t = t * 2.0;
            self.bottom_color.lerp(&self.horizon_color, lower_t)
        };
        
        // Add sun
        let sun_dot = dir.dot(&self.sun_direction);
        if sun_dot > (1.0 - self.sun_size) {
            let sun_intensity = ((sun_dot - (1.0 - self.sun_size)) / self.sun_size).powf(2.0);
            sky_color.lerp(&self.sun_color, sun_intensity)
        } else {
            // Add sun glow
            let glow_size = self.sun_size * 3.0;
            if sun_dot > (1.0 - glow_size) {
                let glow_intensity = ((sun_dot - (1.0 - glow_size)) / glow_size).powf(0.5) * 0.3;
                sky_color.lerp(&self.sun_color, glow_intensity)
            } else {
                sky_color
            }
        }
    }
}

impl Default for Skybox {
    fn default() -> Self {
        Self::new()
    }
}
