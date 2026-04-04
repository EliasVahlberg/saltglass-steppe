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
}
