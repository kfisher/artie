// Copyright 2026 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! GObject representation of an subtitle track.

use gtk::glib::{self, Object};

use crate::models::SubtitleTrack;

glib::wrapper! {
    pub struct SubtitleTrackObject(ObjectSubclass<imp::SubtitleTrackObject>)
        @extends super::TrackObject;
}

impl SubtitleTrackObject {
    /// Creates a new subtitle track object instance from a [`SubtitleTrack`].
    ///
    /// # Panics
    ///
    /// This will panic if the GObject cannot be created.
    pub fn new(subtitle_track: &SubtitleTrack) -> Self {
        Object::builder()
            .property("codec", subtitle_track.codec.to_string())
            .property("language", &subtitle_track.language)
            .build()
    }
}

mod imp {
    use std::cell::RefCell;

    use gtk::glib::{self, Properties};
    use gtk::prelude::*;
    use gtk::subclass::prelude::*;

    #[derive(Default, Properties)]
    #[properties(wrapper_type = super::SubtitleTrackObject)]
    pub struct SubtitleTrackObject {
        /// The subtitle's codec.
        ///
        /// This will be the string representation of the track's [`crate::models::SubtitleCodec`]
        /// value.
        #[property(name = "codec", get, set, type = String)]
        pub codec: RefCell<String>,

        /// The track's language.
        #[property(name = "language", get, set, type = String)]
        pub language: RefCell<String>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SubtitleTrackObject {
        const NAME: &'static str = "ArtieSubtitleTrackObject";
        type Type = super::SubtitleTrackObject;
        type ParentType = super::super::TrackObject;
    }

    #[glib::derived_properties]
    impl ObjectImpl for SubtitleTrackObject {}

    impl super::super::TrackObjectImpl for SubtitleTrackObject {}
}

#[cfg(test)]
mod tests {
    // TODO[TESTS]
}



