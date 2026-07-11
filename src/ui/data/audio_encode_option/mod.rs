// Copyright 2026 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! GObject containing the parameters for encoding an audio track when transcoding a video.

mod imp;

use gtk::glib::{self, Object};

use crate::ui::data::AudioTrackObject;

glib::wrapper! {
    pub struct AudioEncodeOptionObject(ObjectSubclass<imp::AudioEncodeOptionObject>);
}

impl AudioEncodeOptionObject {
    /// Create a new instance of [`AudioEncodeOptionObject`].
    pub fn new(
        track_number: u8,
        source_track: &AudioTrackObject,
        encoder: &str,
        name: &str,
    ) -> Self {
        Object::builder()
            .property("track-number", track_number)
            .property("source-track", source_track)
            .property("encoder", encoder)
            .property("name", name)
            .build()
    }
}

