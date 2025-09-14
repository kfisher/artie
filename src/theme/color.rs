// Copyright 2025 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

/// Represents a color using hue, saturation, and lightness with an alpha channel.
#[derive(Clone, Copy, Debug)]
pub struct Color {
    /// Degree on the color wheel [0 ... 360].
    hue: f32,

    /// Intensity of the color [0 ... 1].
    saturation: f32,

    /// How much light to give the color [0 ... 1].
    lightness: f32,

    /// Transparency of the color [0 .... 1].
    alpha: f32,
}

impl Color {
    pub const MIN_HUE: f32 = 0.0;
    pub const MAX_HUE: f32 = 360.0;

    pub const MIN_SATURATION: f32 = 0.0;
    pub const MAX_SATURATION: f32 = 1.0;

    pub const MIN_LIGHTNESS: f32 = 0.0;
    pub const MAX_LIGHTNESS: f32 = 1.0;

    pub const MIN_ALPHA: f32 = 0.0;
    pub const MAX_ALPHA: f32 = 1.0;

    /// Creates a new [`Color`] using HSL values with an alpha value of 1.
    pub const fn new(hue: f32, saturation: f32, lightness: f32) -> Self {
        Self::with_alpha(hue, saturation, lightness, 1.0)
    }

    /// Creates a new [`Color`] using HSLA values.
    pub const fn with_alpha(hue: f32, saturation: f32, lightness: f32, alpha: f32) -> Self {
        Self {
            hue: hue.clamp(Color::MIN_HUE, Color::MAX_HUE),
            saturation: saturation.clamp(Color::MIN_SATURATION, Color::MAX_SATURATION),
            lightness: lightness.clamp(Color::MIN_LIGHTNESS, Color::MAX_LIGHTNESS),
            alpha: alpha.clamp(Color::MIN_ALPHA, Color::MAX_ALPHA),
        }
    }

    /// Consumes and returns the color updated with the provided alpha value.
    pub const fn alpha(mut self, value: f32) -> Self {
        self.alpha = value.clamp(Color::MIN_ALPHA, Color::MAX_ALPHA);
        self
    }

    /// Darkens the color by the provided amount.
    pub const fn darken(mut self, amount: f32) -> Self {
        self.lightness = (self.lightness + amount)
            .clamp(Color::MIN_LIGHTNESS, Color::MAX_LIGHTNESS);
        self
    }

    /// Lightens the color by the provided amount.
    pub const fn lighten(mut self, amount: f32) -> Self {
        self.lightness = (self.lightness - amount)
            .clamp(Color::MIN_LIGHTNESS, Color::MAX_LIGHTNESS);
        self
    }

    /// Returns the RGB values of the color.
    pub const fn rgb(&self) -> (f32, f32, f32) {
        //
        // NOTE: The following algorithm was generated using ChatGPT. I have no idea whats its
        //       doing, but it seems to work and aligns with other routines I've examined online.
        // 

        const fn hue_to_rgb(p: f32, q: f32, t: f32) -> f32 {
            let mut t = t;
            if t < 0.0 { 
                t += 1.0; 
            }
            if t > 1.0 { 
                t -= 1.0; 
            }

            if t < 1.0/6.0 { 
                p + (q - p) * 6.0 * t
            } else if t < 1.0/2.0 { 
                q
            } else if t < 2.0/3.0 {
                p + (q - p) * (2.0/3.0 - t) * 6.0
            } else {
                p
            }
        }

        let (r, g, b);

        if self.saturation.abs() < f32::EPSILON {
            r = self.lightness;
            g = self.lightness;
            b = self.lightness;
        } else {
            let q = if self.lightness < 0.5 {
                self.lightness * (1.0 + self.saturation)
            } else {
                self.lightness + self.saturation - self.lightness * self.saturation
            };

            let p = 2.0 * self.lightness - q;

            let h = self.hue / 360.0;

            r = hue_to_rgb(p, q, h + 1.0/3.0);
            g = hue_to_rgb(p, q, h);
            b = hue_to_rgb(p, q, h - 1.0/3.0);
        }

        (r, g, b)
    }
}

impl From<Color> for iced::Color {
    fn from(color: Color) -> Self {
        let rgb = color.rgb();
        iced::Color::from_rgba(rgb.0, rgb.1, rgb.2, color.alpha)
    }
}

impl From<Color> for iced::Background {
    fn from(color: Color) -> Self {
        let rgb = color.rgb();
        iced::Color::from_rgba(rgb.0, rgb.1, rgb.2, color.alpha).into()
    }
}

// TODO: Need to write tests.
