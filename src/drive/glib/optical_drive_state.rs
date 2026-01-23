// Copyright 2025 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! Defines the GObject representation of an optical drive's state.

use gtk::glib;


/// GObject representation of the optical drive's state.
///
/// This state combines the [`crate::drive::DiscState`] and [`crate::drive::OpticalDriveState`]
/// rust types into a single enumeration that can be used as a GObject property.
#[derive(Default, Debug, Eq, PartialEq, Clone, Copy, glib::Enum)]
#[repr(u8)]
#[enum_type(name = "ArtieOpticalDriveState")]
pub enum OpticalDriveState {
    #[default]
    Disconnected = 0,
    Empty = 1,
    Idle = 2,
    Copying = 3,
    Success = 4,
    Failed = 5,
}
