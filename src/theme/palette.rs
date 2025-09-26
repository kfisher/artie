// Copyright 2025 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

use crate::theme::color::Color;

/// A set of colors that make up the application's color palette.
///
/// This pallet was structured to mimic the [catppuccin](https://catppuccin.com/palette/) color
/// pallet.
pub struct Palette {
    pub base: Color,
    pub blue: Color,
    pub crust: Color,
    pub flamingo: Color,
    pub green: Color,
    pub lavender: Color,
    pub mantle: Color,
    pub maroon: Color,
    pub mauve: Color,
    pub overlay_0: Color,
    pub overlay_1: Color,
    pub overlay_2: Color,
    pub peach: Color,
    pub pink: Color,
    pub red: Color,
    pub rosewater: Color,
    pub sapphire: Color,
    pub sky: Color,
    pub subtext_0: Color,
    pub subtext_1: Color,
    pub surface_0: Color,
    pub surface_1: Color,
    pub surface_2: Color,
    pub teal: Color,
    pub text: Color,
    pub yellow: Color,

    pub primary: Color,
    pub border: Color,

    pub is_dark: bool,
}

/// Helper macro to populate the palette from a module.
macro_rules! palette {
    ($p:ident, $d:expr) => {
        Palette {
            base: $p::BASE,
            blue: $p::BLUE,
            crust: $p::CRUST,
            flamingo: $p::FLAMINGO,
            green: $p::GREEN,
            lavender: $p::LAVENDER,
            mantle: $p::MANTLE,
            maroon: $p::MAROON,
            mauve: $p::MAUVE,
            overlay_0: $p::OVERLAY_0,
            overlay_1: $p::OVERLAY_1,
            overlay_2: $p::OVERLAY_2,
            peach: $p::PEACH,
            pink: $p::PINK,
            red: $p::RED,
            rosewater: $p::ROSEWATER,
            sapphire: $p::SAPPHIRE,
            sky: $p::SKY,
            subtext_0: $p::SUBTEXT_0,
            subtext_1: $p::SUBTEXT_1,
            surface_0: $p::SURFACE_0,
            surface_1: $p::SURFACE_1,
            surface_2: $p::SURFACE_2,
            teal: $p::TEAL,
            text: $p::TEXT,
            yellow: $p::YELLOW,

            primary: $p::BLUE,
            border: $p::OVERLAY_0,

            is_dark: $d,
        }
    }
}

impl Palette {
    /// The dark theme palette.
    pub const DARK: Self = palette!(mocha, true);

    /// The light theme palette.
    pub const LIGHT: Self = palette!(latte, false);
}

// NOTE: I could have defined the following constants directly in the palette constants. Defining
//       them in modules will make it easier to switch out the pallets in the future. Once things
//       stabilize a bit, may want to consider definining them directly in the palette constants.

#[allow(dead_code)]
mod latte {
    use crate::theme::color::Color;

    pub const BASE:      Color = Color::new(220.0000, 0.2308, 0.9490); // #EFF1F5
    pub const BLUE:      Color = Color::new(219.907,  0.9149, 0.5392); // #1E66F5
    pub const CRUST:     Color = Color::new(220.0000, 0.2069, 0.8863); // #DCE0E8
    pub const FLAMINGO:  Color = Color::new(  0.0000, 0.5976, 0.6686); // #DD7878
    pub const GREEN:     Color = Color::new(109.2308, 0.5764, 0.3980); // #40A02B
    pub const LAVENDER:  Color = Color::new(230.9353, 0.9720, 0.7196); // #7287FD
    pub const MANTLE:    Color = Color::new(220.0000, 0.2195, 0.9196); // #E6E9EF
    pub const MAROON:    Color = Color::new(354.7826, 0.7630, 0.5863); // #E64553
    pub const MAUVE:     Color = Color::new(266.044,  0.8505, 0.5804); // #8839EF
    pub const OVERLAY_0: Color = Color::new(228.0000, 0.1124, 0.6510); // #9CA0B0
    pub const OVERLAY_1: Color = Color::new(231.4286, 0.1005, 0.5902); // #8C8FA1
    pub const OVERLAY_2: Color = Color::new(232.1739, 0.0962, 0.5314); // #7C7F93
    pub const PEACH:     Color = Color::new( 21.9753, 0.9918, 0.5196); // #FE640B
    pub const PINK:      Color = Color::new(316.0345, 0.7342, 0.6902); // #EA76CB
    pub const RED:       Color = Color::new(347.0769, 0.8667, 0.4412); // #D20F39
    pub const ROSEWATER: Color = Color::new( 10.8000, 0.5882, 0.6667); // #DC8A78
    pub const SAPPHIRE:  Color = Color::new(188.8591, 0.6995, 0.4176); // #209FB5
    pub const SKY:       Color = Color::new(197.0667, 0.9657, 0.4569); // #04A5E5
    pub const SUBTEXT_0: Color = Color::new(232.8,    0.1037, 0.4725); // #6C6F85
    pub const SUBTEXT_1: Color = Color::new(233.3333, 0.1280, 0.4137); // #5C5F77
    pub const SURFACE_0: Color = Color::new(222.8571, 0.1591, 0.8275); // #CCD0DA
    pub const SURFACE_1: Color = Color::new(225.0000, 0.1356, 0.7686); // #BCC0CC
    pub const SURFACE_2: Color = Color::new(226.6667, 0.1216, 0.7098); // #ACB0BE
    pub const TEAL:      Color = Color::new(183.2308, 0.7386, 0.3451); // #179299
    pub const TEXT:      Color = Color::new(233.7931, 0.1602, 0.3549); // #4C4F69
    pub const YELLOW:    Color = Color::new( 34.9485, 0.7698, 0.4941); // #DF8E1D
}

#[allow(dead_code)]
mod frappe {
    use crate::theme::color::Color;

    pub const BASE:      Color = Color::new(229.0909, 0.1864, 0.2314); // #303446
    pub const BLUE:      Color = Color::new(221.6327, 0.7424, 0.7412); // #8CAAEE
    pub const CRUST:     Color = Color::new(229.4118, 0.1954, 0.1706); // #232634
    pub const FLAMINGO:  Color = Color::new(  0.0000, 0.5854, 0.8392); // #EEBEBE
    pub const GREEN:     Color = Color::new( 95.8333, 0.4390, 0.6784); // #A6D189
    pub const LAVENDER:  Color = Color::new(238.9091, 0.6627, 0.8373); // #BABBF1
    pub const MANTLE:    Color = Color::new(230.5263, 0.1881, 0.1980); // #292C3C
    pub const MAROON:    Color = Color::new(357.7778, 0.6585, 0.7588); // #EA999C
    pub const MAUVE:     Color = Color::new(276.6667, 0.5902, 0.7608); // #CA9EE6
    pub const OVERLAY_0: Color = Color::new(229.0909, 0.1336, 0.5157); // #737994
    pub const OVERLAY_1: Color = Color::new(226.6667, 0.1698, 0.5843); // #838BA7
    pub const OVERLAY_2: Color = Color::new(227.6923, 0.2229, 0.6569); // #949CBB
    pub const PEACH:     Color = Color::new( 20.3306, 0.7908, 0.7000); // #EF9F76
    pub const PINK:      Color = Color::new(316.0000, 0.7317, 0.8392); // #F4B8E4
    pub const RED:       Color = Color::new(358.8119, 0.6779, 0.7078); // #E78284
    pub const ROSEWATER: Color = Color::new( 10.2857, 0.5738, 0.8804); // #F2D5CF
    pub const SAPPHIRE:  Color = Color::new(198.6207, 0.5541, 0.6922); // #85C1DC
    pub const SKY:       Color = Color::new(189.0909, 0.4783, 0.7294); // #99D1DB
    pub const SUBTEXT_0: Color = Color::new(228.2927, 0.2950, 0.7275); // #A5ADCE
    pub const SUBTEXT_1: Color = Color::new(226.6667, 0.4369, 0.7980); // #B5BFE2
    pub const SURFACE_0: Color = Color::new(230.0000, 0.1558, 0.3020); // #414559
    pub const SURFACE_1: Color = Color::new(227.1429, 0.1474, 0.3725); // #51576D
    pub const SURFACE_2: Color = Color::new(228.0000, 0.1327, 0.4431); // #626880
    pub const TEAL:      Color = Color::new(171.5493, 0.3923, 0.6451); // #81C8BE
    pub const TEXT:      Color = Color::new(227.234,  0.7015, 0.8686); // #C6D0F5
    pub const YELLOW:    Color = Color::new( 39.5294, 0.6204, 0.7314); // #E5C890
}

#[allow(dead_code)]
mod macchiato {
    use crate::theme::color::Color;

    pub const BASE:      Color = Color::new(231.8182, 0.2340, 0.1843); // #24273A
    pub const BLUE:      Color = Color::new(220.1887, 0.8281, 0.7490); // #8AADF4
    pub const CRUST:     Color = Color::new(235.7143, 0.2258, 0.1216); // #181926
    pub const FLAMINGO:  Color = Color::new(  0.0000, 0.5833, 0.8588); // #F0C6C6
    pub const GREEN:     Color = Color::new(105.2174, 0.4825, 0.7196); // #A6DA95
    pub const LAVENDER:  Color = Color::new(234.4615, 0.8228, 0.8451); // #B7BDF8
    pub const MANTLE:    Color = Color::new(233.3333, 0.2308, 0.1529); // #1E2030
    pub const MAROON:    Color = Color::new(355.0588, 0.7143, 0.7667); // #EE99A0
    pub const MAUVE:     Color = Color::new(266.5116, 0.8269, 0.7961); // #C6A0F6
    pub const OVERLAY_0: Color = Color::new(230.3226, 0.1235, 0.4922); // #6E738D
    pub const OVERLAY_1: Color = Color::new(227.6471, 0.1545, 0.5686); // #8087A2
    pub const OVERLAY_2: Color = Color::new(228.3333, 0.2000, 0.6471); // #939AB7
    pub const PEACH:     Color = Color::new( 21.3559, 0.8551, 0.7294); // #F5A97F
    pub const PINK:      Color = Color::new(316.0714, 0.7368, 0.8510); // #F5BDE6
    pub const RED:       Color = Color::new(351.1765, 0.7391, 0.7294); // #ED8796
    pub const ROSEWATER: Color = Color::new( 10.0000, 0.5769, 0.8980); // #F4DBD6
    pub const SAPPHIRE:  Color = Color::new(198.6408, 0.6561, 0.6922); // #7DC4E4
    pub const SKY:       Color = Color::new(188.7805, 0.5942, 0.7294); // #91D7E3
    pub const SUBTEXT_0: Color = Color::new(227.3684, 0.2676, 0.7216); // #A5ADCB
    pub const SUBTEXT_1: Color = Color::new(228.0000, 0.3922, 0.8000); // #B8C0E0
    pub const SURFACE_0: Color = Color::new(230.4,    0.1880, 0.2608); // #363A4F
    pub const SURFACE_1: Color = Color::new(231.1111, 0.1561, 0.3392); // #494D64
    pub const SURFACE_2: Color = Color::new(229.6552, 0.1374, 0.4137); // #5B6078
    pub const TEAL:      Color = Color::new(171.0811, 0.4684, 0.6902); // #8BD5CA
    pub const TEXT:      Color = Color::new(227.4419, 0.6825, 0.8765); // #CAD3F5
    pub const YELLOW:    Color = Color::new( 40.2532, 0.6991, 0.7784); // #EED49F
}

#[allow(dead_code)]
pub mod mocha {
    use crate::theme::color::Color;

    pub const BASE:      Color = Color::new(240.0000, 0.2105, 0.1490); // #1E1E2E
    pub const BLUE:      Color = Color::new(217.1681, 0.9187, 0.7588); // #89B4FA
    pub const CRUST:     Color = Color::new(240.0000, 0.2273, 0.0863); // #11111B
    pub const FLAMINGO:  Color = Color::new(  0.0000, 0.5873, 0.8765); // #F2CDCD
    pub const GREEN:     Color = Color::new(115.4545, 0.5410, 0.7608); // #A6E3A1
    pub const LAVENDER:  Color = Color::new(231.8919, 0.9737, 0.8510); // #B4BEFE
    pub const MANTLE:    Color = Color::new(240.0000, 0.2131, 0.1196); // #181825
    pub const MAROON:    Color = Color::new(350.4,    0.6522, 0.7745); // #EBA0AC
    pub const MAUVE:     Color = Color::new(267.4074, 0.8351, 0.8098); // #CBA6F7
    pub const OVERLAY_0: Color = Color::new(230.7692, 0.1074, 0.4745); // #6C7086
    pub const OVERLAY_1: Color = Color::new(229.6552, 0.1278, 0.5549); // #7F849C
    pub const OVERLAY_2: Color = Color::new(228.3871, 0.1676, 0.6373); // #9399B2
    pub const PEACH:     Color = Color::new( 22.9565, 0.9200, 0.7549); // #FAB387
    pub const PINK:      Color = Color::new(316.4706, 0.7183, 0.8608); // #F5C2E7
    pub const RED:       Color = Color::new(343.2692, 0.8125, 0.7490); // #F38BA8
    pub const ROSEWATER: Color = Color::new(  9.6000, 0.5556, 0.9118); // #F5E0DC
    pub const SAPPHIRE:  Color = Color::new(198.5,    0.7595, 0.6902); // #74C7EC
    pub const SKY:       Color = Color::new(189.1837, 0.7101, 0.7294); // #89DCEB
    pub const SUBTEXT_0: Color = Color::new(227.6471, 0.2361, 0.7176); // #A6ADC8
    pub const SUBTEXT_1: Color = Color::new(226.6667, 0.3529, 0.8000); // #BAC2DE
    pub const SURFACE_0: Color = Color::new(236.8421, 0.1624, 0.2294); // #313244
    pub const SURFACE_1: Color = Color::new(234.2857, 0.1321, 0.3118); // #45475A
    pub const SURFACE_2: Color = Color::new(232.5,    0.1200, 0.3922); // #585B70
    pub const TEAL:      Color = Color::new(170.0000, 0.5735, 0.7333); // #94E2D5
    pub const TEXT:      Color = Color::new(226.1538, 0.6393, 0.8804); // #CDD6F4
    pub const YELLOW:    Color = Color::new( 41.3514, 0.8605, 0.8314); // #F9E2AF
}

