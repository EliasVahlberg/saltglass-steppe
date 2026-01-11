//! Performance optimization utilities for the renderer

use std::time::{Duration, Instant};

/// Frame rate limiter to prevent excessive CPU usage
pub struct FrameLimiter {
    target_fps: u32,
    frame_duration: Duration,
    last_frame: Instant,
}

impl FrameLimiter {
    /// Create a new frame limiter with target FPS
    pub fn new(target_fps: u32) -> Self {
        Self {
            target_fps,
            frame_duration: Duration::from_nanos(1_000_000_000 / target_fps as u64),
            last_frame: Instant::now(),
        }
    }

    /// Wait until it's time for the next frame
    pub fn limit(&mut self) {
        let elapsed = self.last_frame.elapsed();
        if elapsed < self.frame_duration {
            std::thread::sleep(self.frame_duration - elapsed);
        }
        self.last_frame = Instant::now();
    }

    /// Set new target FPS
    pub fn set_fps(&mut self, fps: u32) {
        self.target_fps = fps;
        self.frame_duration = Duration::from_nanos(1_000_000_000 / fps as u64);
    }

    /// Get current target FPS
    pub fn fps(&self) -> u32 {
        self.target_fps
    }
}

/// Viewport culling utilities
pub struct ViewportCuller {
    last_cam_x: i32,
    last_cam_y: i32,
    last_width: i32,
    last_height: i32,
    cached_bounds: Option<(i32, i32, i32, i32)>, // min_x, min_y, max_x, max_y
}

impl ViewportCuller {
    pub fn new() -> Self {
        Self {
            last_cam_x: i32::MIN,
            last_cam_y: i32::MIN,
            last_width: 0,
            last_height: 0,
            cached_bounds: None,
        }
    }

    /// Get viewport bounds, using cache if camera hasn't moved
    pub fn get_bounds(
        &mut self,
        cam_x: i32,
        cam_y: i32,
        width: i32,
        height: i32,
    ) -> (i32, i32, i32, i32) {
        // Check if we can use cached bounds
        if let Some(bounds) = self.cached_bounds {
            if cam_x == self.last_cam_x
                && cam_y == self.last_cam_y
                && width == self.last_width
                && height == self.last_height
            {
                return bounds;
            }
        }

        // Calculate new bounds with small buffer for smooth scrolling
        let buffer = 2;
        let bounds = (
            cam_x - buffer,
            cam_y - buffer,
            cam_x + width + buffer,
            cam_y + height + buffer,
        );

        // Cache the results
        self.last_cam_x = cam_x;
        self.last_cam_y = cam_y;
        self.last_width = width;
        self.last_height = height;
        self.cached_bounds = Some(bounds);

        bounds
    }

    /// Check if a point is within the viewport bounds
    pub fn is_in_bounds(&self, x: i32, y: i32, bounds: (i32, i32, i32, i32)) -> bool {
        let (min_x, min_y, max_x, max_y) = bounds;
        x >= min_x && x < max_x && y >= min_y && y < max_y
    }
}

impl Default for ViewportCuller {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_frame_limiter() {
        let mut limiter = FrameLimiter::new(60);
        assert_eq!(limiter.fps(), 60);

        limiter.set_fps(30);
        assert_eq!(limiter.fps(), 30);
    }

    #[test]
    fn test_viewport_culler() {
        let mut culler = ViewportCuller::new();

        let bounds = culler.get_bounds(10, 10, 20, 20);
        assert_eq!(bounds, (8, 8, 32, 32)); // With buffer

        // Test caching
        let bounds2 = culler.get_bounds(10, 10, 20, 20);
        assert_eq!(bounds, bounds2);

        // Test bounds checking
        assert!(culler.is_in_bounds(15, 15, bounds));
        assert!(!culler.is_in_bounds(50, 50, bounds));
    }
}
