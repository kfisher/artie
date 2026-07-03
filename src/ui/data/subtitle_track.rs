// Copyright 2026 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! GObject representation of an subtitle track.

use gtk::glib::{self, Object};

use crate::models::SubtitleTrack;
use crate::utilities;

glib::wrapper! {
    pub struct SubtitleTrackObject(ObjectSubclass<imp::SubtitleTrackObject>);
}

impl SubtitleTrackObject {
    /// Creates a new subtitle track object instance from a [`SubtitleTrack`].
    ///
    /// # Panics
    ///
    /// This will panic if the GObject cannot be created.
    pub fn new(subtitle_track: &SubtitleTrack) -> Self {
        Object::builder()
            .property("subtitle-track-index", subtitle_track.subtitle_index)
            .property("codec", subtitle_track.codec.to_string())
            .property("language", &subtitle_track.language_code)
            .build()
    }

    /// Return the display text to use in the track selection dropdown.
    pub fn selector_display(&self) -> String {
        format!(
            "({}) {}",
            self.subtitle_track_index(),
            utilities::expand_language_code(self.language().as_str()),
        )
    }
}

mod imp {
    use std::cell::{Cell, RefCell};

    use gtk::glib::{self, Properties};
    use gtk::prelude::*;
    use gtk::subclass::prelude::*;

    use crate::ui::data::TrackPreviewObject;

    #[derive(Default, Properties)]
    #[properties(wrapper_type = super::SubtitleTrackObject)]
    pub struct SubtitleTrackObject {
        /// The subtitle track index (starting at 1).
        #[property(name = "subtitle-track-index", get, set, type = u8)]
        pub subtitle_track_index: Cell<u8>,

        /// The subtitle's codec.
        ///
        /// This will be the string representation of the track's [`crate::models::SubtitleCodec`]
        /// value.
        #[property(name = "codec", get, set, type = String)]
        pub codec: RefCell<String>,

        /// The track's language.
        #[property(name = "language", get, set, type = String)]
        pub language: RefCell<String>,

        /// Preview data for the track.
        ///
        /// This will only be valid when the video is selected for preview. 
        #[property(name = "preview", get, set)]
        pub(super) preview: RefCell<TrackPreviewObject>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SubtitleTrackObject {
        const NAME: &'static str = "ArtieSubtitleTrackObject";
        type Type = super::SubtitleTrackObject;
    }

    #[glib::derived_properties]
    impl ObjectImpl for SubtitleTrackObject {}
}

#[cfg(test)]
mod tests {
    // TODO[TESTS]
}



