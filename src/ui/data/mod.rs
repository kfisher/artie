// Copyright 2026 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! General (non-widget) GLib objects.

mod audio_track;
mod enums;
mod optical_drive;
mod subtitle_track;
mod title;
mod track_object;
mod video;
mod video_track;

pub use audio_track::AudioTrackObject;
pub use enums::{MediaType, OpticalDriveState};
pub use optical_drive::OpticalDriveObject;
pub use subtitle_track::SubtitleTrackObject;
pub use title::TitleObject;
pub use track_object::{TrackObject, TrackObjectImpl};
pub use video::VideoObject;
pub use video_track::VideoTrackObject;
