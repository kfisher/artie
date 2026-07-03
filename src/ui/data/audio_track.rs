// Copyright 2026 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! GObject representation of an audio track.

use gtk::glib::{self, Object};

use crate::models::AudioTrack;
use crate::utilities;

glib::wrapper! {
    pub struct AudioTrackObject(ObjectSubclass<imp::AudioTrackObject>);
}

impl AudioTrackObject {
    /// Creates a new audio track object instance from a [`AudioTrack`].
    ///
    /// # Panics
    ///
    /// This will panic if the GObject cannot be created.
    pub fn new(audio_track: &AudioTrack) -> Self {
        Object::builder()
            .property("audio-track-index", audio_track.audio_index)
            .property("name", &audio_track.name)
            .property("codec", &audio_track.codec.to_string())
            .property("language", &audio_track.language_code)
            .property("layout", &audio_track.channel_layout)
            .build()
    }

    /// Return the display text to use in the track selection dropdown.
    pub fn selector_display(&self) -> String {
        format!(
            "({}) {} - {}",
            self.audio_track_index(),
            self.name(),
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
    #[properties(wrapper_type = super::AudioTrackObject)]
    pub struct AudioTrackObject {
        /// The audio track index (starting at 1).
        #[property(name = "audio-track-index", get, set, type = u8)]
        pub audio_track_index: Cell<u8>,

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

        /// Preview data for the track.
        ///
        /// This will only be valid when the video is selected for preview. 
        #[property(name = "preview", get, set)]
        pub(super) preview: RefCell<TrackPreviewObject>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for AudioTrackObject {
        const NAME: &'static str = "ArtieAudioTrackObject";
        type Type = super::AudioTrackObject;
    }

    #[glib::derived_properties]
    impl ObjectImpl for AudioTrackObject {}
}

#[cfg(test)]
mod tests {
    // TODO[TESTS]
}


