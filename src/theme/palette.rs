// Copyright 2025 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! Application color palette.

use std::sync::LazyLock;

use iced::Color;

use crate::theme::color;

/// Defines a color group.
///
/// A color group consists of a base color plus several variants for things like borders and 
/// hovering. It also includes the color for displaying text on elements whose background is the 
/// base color. This is mainly done to make things easier to keep consitant while also allow 
/// tweaking to individual groups if needed. 
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ColorSet {
    /// The base color.
    pub color: Color,

    /// The color to use for text where the background is the base color.
    pub text: Color,

    /// The color to use for the border for an element whose background is the base color.
    pub border: Color,

    /// The hover variant of the base color.
    pub hover: Color,
}

/// Color pallet for the application.
pub struct Palette {
    pub accent: ColorSet,
    pub background: ColorSet,
    pub danger: ColorSet,
    pub primary: ColorSet,
    pub secondary: ColorSet,
    pub selection: ColorSet,
    pub subtext: ColorSet,
    pub success: ColorSet,
    pub surface_0: ColorSet,
    pub surface_1: ColorSet,
    pub surface_2: ColorSet,
    pub surface_3: ColorSet,
    pub text: ColorSet,
    pub modal: Color,
}

// NOTE: The following is based of the Nord color palette.
//       https://www.nordtheme.com/

// TODO: The `colors::TODO` items below are bright green so that they stand out when attempting to 
//       use them. This way, the color will be set when there is a use case to test against.

pub static DARK_PALETTE: LazyLock<Palette> = LazyLock::new(|| {
    Palette {
        accent: ColorSet {
            color:  colors::NORD07,
            text:   color::darken(&colors::NORD07, 0.42),
            border: color::darken(&colors::NORD07, 0.125),
            hover:  color::darken(&colors::NORD07, 0.1),
        },
        background: ColorSet {
            color:  colors::NORD00,
            text:   colors::TODO,
            border: color::darken(&colors::NORD00, 0.125),
            hover:  colors::TODO,
        },
        danger: ColorSet {
            color:  colors::NORD11,
            text:   color::darken(&colors::NORD11, 0.42),
            border: color::darken(&colors::NORD11, 0.125),
            hover:  color::darken(&colors::NORD11, 0.1),
        },
        primary: ColorSet {
            color:  colors::NORD08,
            text:   color::darken(&colors::NORD08, 0.42),
            border: color::darken(&colors::NORD08, 0.125),
            hover:  color::darken(&colors::NORD08, 0.1),
        },
        secondary: ColorSet {
            color:  colors::NORD09,
            text:   color::darken(&colors::NORD09, 0.42),
            border: color::darken(&colors::NORD09, 0.125),
            hover:  color::darken(&colors::NORD09, 0.1),
        },
        subtext: ColorSet {
            color:  colors::NORD04,
            text:   colors::TODO,
            border: colors::TODO,
            hover:  colors::TODO,
        },
        success: ColorSet {
            color:  colors::NORD14,
            text:   color::darken(&colors::NORD14, 0.42),
            border: color::darken(&colors::NORD14, 0.125),
            hover:  color::darken(&colors::NORD14, 0.1),
        },
        selection: ColorSet {
            color:  colors::NORD10,
            text:   colors::NORD06,
            border: colors::TODO,
            hover:  colors::TODO,
        },
        surface_0: ColorSet {
            color:  colors::NORD00,
            text:   color::lighten(&colors::NORD00, 0.42),
            border: color::darken(&colors::NORD00, 0.125),
            hover:  color::darken(&colors::NORD00, 0.1),
        },
        surface_1: ColorSet {
            color:  colors::NORD01,
            text:   color::lighten(&colors::NORD01, 0.42),
            border: color::darken(&colors::NORD01, 0.125),
            hover:  color::darken(&colors::NORD01, 0.1),
        },
        surface_2: ColorSet {
            color:  colors::NORD02,
            text:   color::lighten(&colors::NORD02, 0.42),
            border: color::darken(&colors::NORD02, 0.125),
            hover:  color::darken(&colors::NORD02, 0.1),
        },
        surface_3: ColorSet {
            color:  colors::NORD03,
            text:   color::lighten(&colors::NORD03, 0.42),
            border: color::darken(&colors::NORD03, 0.125),
            hover:  color::darken(&colors::NORD03, 0.1),
        },
        text: ColorSet {
            color:  colors::NORD06,
            text:   colors::TODO,
            border: colors::TODO,
            hover:  colors::TODO,
        },
        modal: Color::from_rgba8(0x00, 0x00, 0x00, 0.50),
    }
});

#[allow(dead_code)]
pub mod colors {
    use iced::Color;

    pub const NORD00: Color = Color::from_rgb8(0x2E, 0x34, 0x40);
    pub const NORD01: Color = Color::from_rgb8(0x3B, 0x42, 0x52);
    pub const NORD02: Color = Color::from_rgb8(0x43, 0x4C, 0x5E);
    pub const NORD03: Color = Color::from_rgb8(0x4C, 0x56, 0x6A);
    pub const NORD04: Color = Color::from_rgb8(0xD8, 0xDE, 0xE9);
    pub const NORD05: Color = Color::from_rgb8(0xE5, 0xE9, 0xF0);
    pub const NORD06: Color = Color::from_rgb8(0xEC, 0xEF, 0xF4);
    pub const NORD07: Color = Color::from_rgb8(0x8F, 0xBC, 0xBB);
    pub const NORD08: Color = Color::from_rgb8(0x88, 0xC0, 0xD0);
    pub const NORD09: Color = Color::from_rgb8(0x81, 0xA1, 0xC1);
    pub const NORD10: Color = Color::from_rgb8(0x5E, 0x81, 0xAC);
    pub const NORD11: Color = Color::from_rgb8(0xBF, 0x61, 0x6A);
    pub const NORD12: Color = Color::from_rgb8(0xD0, 0x87, 0x70);
    pub const NORD13: Color = Color::from_rgb8(0xEB, 0xCB, 0x8B);
    pub const NORD14: Color = Color::from_rgb8(0xA3, 0xBE, 0x8C);
    pub const NORD15: Color = Color::from_rgb8(0xB4, 0x8E, 0xAD);

    pub const TODO: Color = Color {
        r: 0.0,
        g: 1.0,
        b: 0.0,
        a: 1.0,
    };
}
