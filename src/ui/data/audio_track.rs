// Copyright 2026 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! GObject representation of an audio track.

use gtk::glib::{self, Object};

use crate::models::AudioTrack;

glib::wrapper! {
    pub struct AudioTrackObject(ObjectSubclass<imp::AudioTrackObject>)
        @extends super::TrackObject;
}

impl AudioTrackObject {
    /// Creates a new audio track object instance from a [`AudioTrack`].
    ///
    /// # Panics
    ///
    /// This will panic if the GObject cannot be created.
    pub fn new(audio_track: &AudioTrack) -> Self {
        Object::builder()
            .property("name", &audio_track.name)
            .property("codec", &audio_track.codec.to_string())
            .property("language", &audio_track.language)
            .property("layout", &audio_track.channel_layout)
            .build()
    }
}

mod imp {
    use std::cell::RefCell;

    use gtk::glib::{self, Properties};
    use gtk::prelude::*;
    use gtk::subclass::prelude::*;

    #[derive(Default, Properties)]
    #[properties(wrapper_type = super::AudioTrackObject)]
    pub struct AudioTrackObject {
        /// The name of the audio track.
        #[property(name = "name", get, set, type = String)]
        pub name: RefCell<String>,

        /// The track's audio codec.
        ///
        /// This will be the string representation of the track's [`crate::models::AudioCodec`]
        /// value.
        #[property(name = "codec", get, set, type = String)]
        pub codec: RefCell<String>,

        /// The track's language.
        #[property(name = "language", get, set, type = String)]
        pub language: RefCell<String>,

        /// The audio channel layout for the channel.
        #[property(name = "layout", get, set, type = String)]
        pub layout: RefCell<String>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for AudioTrackObject {
        const NAME: &'static str = "ArtieAudioTrackObject";
        type Type = super::AudioTrackObject;
        type ParentType = super::super::TrackObject;
    }

    #[glib::derived_properties]
    impl ObjectImpl for AudioTrackObject {}

    impl super::super::TrackObjectImpl for AudioTrackObject {}
}

#[cfg(test)]
mod tests {
    // TODO[TESTS]
}


