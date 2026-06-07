// Copyright 2026 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! General (non-widget) GLib objects.

mod audio_track;
mod enums;
mod optical_drive;
mod subtitle_track;
mod title;
mod track_preview;
mod video;
mod video_track;
mod title_form;

pub use audio_track::AudioTrackObject;
pub use enums::{MediaType, OpticalDriveState, SpecialFeatureType};
pub use optical_drive::OpticalDriveObject;
pub use subtitle_track::SubtitleTrackObject;
pub use title::TitleObject;
pub use track_preview::TrackPreviewObject;
pub use video::VideoObject;
pub use video_track::VideoTrackObject;
pub use title_form::{TitleFormObject, TitleFormType};
