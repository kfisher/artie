// Copyright 2025 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! Defines the GObject wrappers the drive module.

mod optical_drive;
mod optical_drive_state;

pub use optical_drive::OpticalDriveObject;
pub use optical_drive_state::OpticalDriveState;
