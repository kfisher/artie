// Copyright 2026 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! General (non-widget) GLib objects.

mod enums;
mod optical_drive;
mod title;
mod video;

pub use enums::{MediaType, OpticalDriveState};
pub use optical_drive::OpticalDriveObject;
pub use title::TitleObject;
pub use video::VideoObject;
