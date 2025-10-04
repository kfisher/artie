// Copyright 2025 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: BSD-3-Clause

//! Color utilities.

use iced::Color;

pub struct Oklch {
    pub l: f32,
    pub c: f32,
    pub h: f32,
    pub a: f32,
}

impl Oklch {
    /// Create Oklch from sRGB Color
    fn from_rgb(color: &Color) -> Self {
        // Step 1: sRGB to linear RGB
        let lr = srgb_to_linear(color.r);
        let lg = srgb_to_linear(color.g);
        let lb = srgb_to_linear(color.b);

        // Step 2: Linear RGB to Oklab
        let l_oklab = 0.4122214708 * lr + 0.5363325363 * lg + 0.0514459929 * lb;
        let m = 0.2119034982 * lr + 0.6806995451 * lg + 0.1073969566 * lb;
        let s = 0.0883024619 * lr + 0.2817188376 * lg + 0.6299787005 * lb;

        let l_ = l_oklab.cbrt();
        let m_ = m.cbrt();
        let s_ = s.cbrt();

        let l = 0.2104542553 * l_ + 0.7936177850 * m_ - 0.0040720468 * s_;
        let a = 1.9779984951 * l_ - 2.4285922050 * m_ + 0.4505937099 * s_;
        let b = 0.0259040371 * l_ + 0.7827717662 * m_ - 0.8086757660 * s_;

        // Step 3: Oklab to Oklch
        let c = (a * a + b * b).sqrt();
        let h = b.atan2(a).to_degrees();
        let h = if h < 0.0 { h + 360.0 } else { h };

        Oklch {
            l,
            c,
            h,
            a: color.a,
        }
    }

    /// Convert Oklch to sRGB Color
    fn to_rgb(&self) -> Color {
        // Step 1: Oklch to Oklab
        let h_rad = self.h.to_radians();
        let a = self.c * h_rad.cos();
        let b = self.c * h_rad.sin();

        // Step 2: Oklab to linear RGB
        let l_ = self.l + 0.3963377774 * a + 0.2158037573 * b;
        let m_ = self.l - 0.1055613458 * a - 0.0638541728 * b;
        let s_ = self.l - 0.0894841775 * a - 1.2914855480 * b;

        let l_lin = l_ * l_ * l_;
        let m_lin = m_ * m_ * m_;
        let s_lin = s_ * s_ * s_;

        let lr = 4.0767416621 * l_lin - 3.3077115913 * m_lin + 0.2309699292 * s_lin;
        let lg = -1.2684380046 * l_lin + 2.6097574011 * m_lin - 0.3413193965 * s_lin;
        let lb = -0.0041960863 * l_lin - 0.7034186147 * m_lin + 1.7076147010 * s_lin;

        // Step 3: Linear RGB to sRGB
        Color {
            r: linear_to_srgb(lr),
            g: linear_to_srgb(lg),
            b: linear_to_srgb(lb),
            a: self.a,
        }
    }

    /// Lighten the color by increasing lightness
    pub const fn lighten(mut self, amount: f32) -> Self {
        self.l = (self.l + amount).min(1.0);
        self
    }

    /// Darken the color by decreasing lightness
    pub const fn darken(mut self, amount: f32) -> Self {
        self.l = (self.l - amount).max(0.0);
        self
    }

    /// Set lightness to a specific value (0.0 to 1.0)
    pub const fn with_lightness(mut self, lightness: f32) -> Self {
        self.l = lightness.clamp(0.0, 1.0);
        self
    }
}

/// Lighten the color by the provided amount.
///
/// `amount` should be a value between 0.0 and 1.0.
pub fn lighten(color: &Color, amount: f32) -> Color {
    Oklch::from_rgb(&color)
        .lighten(amount)
        .to_rgb()
}

/// Darken the color by the provided amount.
///
/// `amount` should be a value between 0.0 and 1.0.
pub fn darken(color: &Color, amount: f32) -> Color {
    Oklch::from_rgb(&color)
        .darken(amount)
        .to_rgb()
}

/// Convert sRGB component to linear RGB
fn srgb_to_linear(c: f32) -> f32 {
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

/// Convert linear RGB component to sRGB
fn linear_to_srgb(c: f32) -> f32 {
    if c <= 0.0031308 {
        c * 12.92
    } else {
        1.055 * c.powf(1.0 / 2.4) - 0.055
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rgb_to_oklch_white() {
        let white = Color { r: 1.0, g: 1.0, b: 1.0, a: 1.0 };
        let oklch = Oklch::from_rgb(&white);
        
        assert!((oklch.l - 1.0).abs() < 0.01);
        assert!(oklch.c < 0.01);
    }

    #[test]
    fn test_rgb_to_oklch_black() {
        let black = Color { r: 0.0, g: 0.0, b: 0.0, a: 1.0 };
        let oklch = Oklch::from_rgb(&black);
        
        assert!(oklch.l < 0.01);
        assert!(oklch.c < 0.01);
    }

    #[test]
    fn test_roundtrip() {
        let original = Color { r: 0.5, g: 0.3, b: 0.8, a: 1.0 };
        let oklch = Oklch::from_rgb(&original);
        let back = oklch.to_rgb();
        
        assert!((original.r - back.r).abs() < 0.001);
        assert!((original.g - back.g).abs() < 0.001);
        assert!((original.b - back.b).abs() < 0.001);
        assert!((original.a - back.a).abs() < 0.001);
    }

    #[test]
    fn test_lighten() {
        let color = Color { r: 0.5, g: 0.5, b: 0.5, a: 1.0 };
        let oklch = Oklch::from_rgb(&color);
        let lighter = Oklch::from_rgb(&color).lighten(0.2);
        
        assert!(lighter.l > oklch.l);
        assert_eq!(lighter.c, oklch.c);
        assert_eq!(lighter.h, oklch.h);
    }

    #[test]
    fn test_darken() {
        let color = Color { r: 0.5, g: 0.5, b: 0.5, a: 1.0 };
        let oklch = Oklch::from_rgb(&color);
        let darker = Oklch::from_rgb(&color).darken(0.2);
        
        assert!(darker.l < oklch.l);
        assert_eq!(darker.c, oklch.c);
        assert_eq!(darker.h, oklch.h);
    }

    #[test]
    fn test_with_lightness() {
        let color = Color { r: 0.5, g: 0.3, b: 0.8, a: 1.0 };
        let oklch = Oklch::from_rgb(&color);
        let adjusted = Oklch::from_rgb(&color).with_lightness(0.75);
        
        assert_eq!(adjusted.l, 0.75);
        assert_eq!(adjusted.c, oklch.c);
        assert_eq!(adjusted.h, oklch.h);
    }
}
