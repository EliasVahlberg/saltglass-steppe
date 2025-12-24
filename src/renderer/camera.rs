//! Camera system for smooth viewport management

/// Manages camera position and smooth movement
pub struct Camera {
    target_x: f32,
    target_y: f32,
    current_x: f32,
    current_y: f32,
    smoothing: f32,
}

impl Camera {
    /// Create a new camera
    pub fn new() -> Self {
        Self {
            target_x: 0.0,
            target_y: 0.0,
            current_x: 0.0,
            current_y: 0.0,
            smoothing: 1.0, // No smoothing - instant camera movement
        }
    }

    /// Update camera to follow a target position
    pub fn update(&mut self, target_x: i32, target_y: i32, view_width: i32, view_height: i32) {
        // Set target position (centered on target)
        // Use integer division to avoid fractional centering that causes jitter
        let half_width = view_width / 2;
        let half_height = view_height / 2;
        
        self.target_x = (target_x - half_width) as f32;
        self.target_y = (target_y - half_height) as f32;

        // Smooth interpolation towards target
        let dx = self.target_x - self.current_x;
        let dy = self.target_y - self.current_y;
        
        self.current_x += dx * self.smoothing;
        self.current_y += dy * self.smoothing;

        // Snap to target if very close (prevents infinite tiny movements)
        if dx.abs() < 0.01 {
            self.current_x = self.target_x;
        }
        if dy.abs() < 0.01 {
            self.current_y = self.target_y;
        }
    }

    /// Get current camera position (top-left corner of viewport)
    pub fn position(&self) -> (i32, i32) {
        (self.current_x as i32, self.current_y as i32)
    }

    /// Get smooth camera position (for smooth scrolling)
    pub fn smooth_position(&self) -> (f32, f32) {
        (self.current_x, self.current_y)
    }

    /// Set smoothing factor
    pub fn set_smoothing(&mut self, smoothing: f32) {
        self.smoothing = smoothing.clamp(0.0, 1.0);
    }

    /// Instantly snap camera to target (no smoothing)
    pub fn snap_to_target(&mut self) {
        self.current_x = self.target_x;
        self.current_y = self.target_y;
    }
}

impl Default for Camera {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_camera_movement() {
        let mut camera = Camera::new();
        
        // Initial position should be 0,0
        assert_eq!(camera.position(), (0, 0));
        
        // Update to new target
        camera.update(10, 10, 20, 20);
        
        // Should move towards target but not instantly (due to smoothing)
        let (x, y) = camera.position();
        assert!(x > 0 && x <= 10);
        assert!(y > 0 && y <= 10);
        
        // After many updates, should reach target
        for _ in 0..100 {
            camera.update(10, 10, 20, 20);
        }
        assert_eq!(camera.position(), (10, 10));
    }

    #[test]
    fn test_camera_snap() {
        let mut camera = Camera::new();
        
        camera.update(10, 10, 20, 20);
        camera.snap_to_target();
        
        assert_eq!(camera.position(), (10, 10));
    }

    #[test]
    fn test_smoothing_factor() {
        let mut camera = Camera::new();
        
        // No smoothing - should move instantly
        camera.set_smoothing(1.0);
        camera.update(10, 10, 20, 20);
        assert_eq!(camera.position(), (10, 10));
        
        // Reset and test with no smoothing
        camera = Camera::new();
        camera.set_smoothing(0.0);
        camera.update(10, 10, 20, 20);
        assert_eq!(camera.position(), (0, 0)); // Should not move at all
    }
}
