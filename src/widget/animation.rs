// Copyright 2025 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! Animation widgets and utilities.

/// Animation for rotating an object.
pub struct RotationAnimation {
    /// Number of times the object should rotate a full 360 degrees per second.
    pub rotations_per_second: f32,

    /// The current rotation in degrees.
    pub degrees: f32,

    /// Indicates if the animation is enabled.
    pub enabled: bool,
}

impl RotationAnimation {
    /// Creates a new [`RotationAnimation`] instance that started disabled with a rotation of zero
    /// degrees.
    pub fn new(rotations_per_second: f32) -> Self {
        Self {
            rotations_per_second,
            degrees: 0.0,
            enabled: false,
        }
    }

    /// Updates the rotation based on the elapsed time in seconds.
    pub fn tick(&mut self, delta_time: f32) {
        self.degrees += 360.0 * self.rotations_per_second * delta_time;
        self.degrees %= 360.0;
    }

    /// Enable the animation.
    pub fn start(&mut self) {
        self.enabled = true;
    }

    /// Disable the animation.
    pub fn stop(&mut self) {
        self.enabled = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tick() {
        let mut rotation = RotationAnimation::new(2.0);
        
        // After 0.5 seconds, should complete 1 full rotation (360°)
        rotation.tick(0.5);
        assert!((rotation.degrees - 0.0).abs() < 0.001);
        
        // After another 0.25 seconds, should be at 180° (half rotation)
        rotation.tick(0.25);
        assert!((rotation.degrees - 180.0).abs() < 0.001);
        
        // After another 0.25 seconds, should be back at 0° (complete 2 rotations)
        rotation.tick(0.25);
        assert!((rotation.degrees - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_tick_wraps_around() {
        let mut rotation = RotationAnimation::new(2.0);
        rotation.degrees = 350.0;
        
        rotation.tick(0.05);
        assert!((rotation.degrees - 26.0).abs() < 0.001);
    }
}
