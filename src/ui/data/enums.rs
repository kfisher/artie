// Copyright 2026 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! GObject enumerations.

use gtk::glib;

/// GObject representation of [`crate::models::MediaType`].
#[derive(Default, Debug, Eq, PartialEq, Clone, Copy, glib::Enum)]
#[repr(u8)]
#[enum_type(name = "ArtieMediaType")]
pub enum MediaType {
    #[default]
    Movie = 0,
    Show = 1,
}

impl MediaType {
    pub fn to_model(&self) -> crate::models::MediaType {
        match self {
            MediaType::Movie => crate::models::MediaType::Movie,
            MediaType::Show => crate::models::MediaType::Show,
        }
    }
}

impl From<crate::models::MediaType> for MediaType {
    fn from(value: crate::models::MediaType) -> Self {
        match value {
            crate::models::MediaType::Movie => MediaType::Movie,
            crate::models::MediaType::Show => MediaType::Show,
        }
    }
}

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
