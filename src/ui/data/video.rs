// Copyright 2026 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! GObject representation of a video.

use gtk::gio::ListStore;
use gtk::glib::{self, Object};

use crate::path;
use crate::models::Video;
use crate::ui::data::{AudioTrackObject, SubtitleTrackObject, TitleObject, VideoTrackObject};
use crate::ui::helpers;


glib::wrapper! {
    pub struct VideoObject(ObjectSubclass<imp::VideoObject>);
}

impl VideoObject {
    /// Creates a new video object instance from a [`models::Video`].
    ///
    /// # Panics
    ///
    /// This will panic if the GObject cannot be created.
    pub fn new(video: &Video) -> Self {
        let title = video.title.value
            .as_ref()
            .map(|t| TitleObject::new(t.as_ref()));

        let audio_tracks = video.audio_tracks
            .iter()
            .map(|t| AudioTrackObject::new(t));
        let audio_tracks = ListStore::from_iter(audio_tracks);

        let subtitle_tracks = video.subtitle_tracks
            .iter()
            .map(|t| SubtitleTrackObject::new(t));
        let subtitle_tracks = ListStore::from_iter(subtitle_tracks);

        let video_tracks = video.video_tracks
            .iter()
            .map(|t| VideoTrackObject::new(t));
        let video_tracks = ListStore::from_iter(video_tracks);

        let path = path::location_path(&video.location)
            .unwrap_or_default();

        Object::builder()
            .property("id", video.id)
            .property("title", title)
            .property("duration", helpers::format_duration(&video.duration))
            .property("path", path)
            .property("audio-tracks", audio_tracks)
            .property("subtitle-tracks", subtitle_tracks)
            .property("video-tracks", video_tracks)
            .build()
    }
}

mod imp {
    use std::cell::{Cell, RefCell};

    use gtk::glib::{self, Object, Properties};
    use gtk::gio::ListStore;
    use gtk::prelude::*;
    use gtk::subclass::prelude::*;

    use crate::ui::data::TitleObject;

    #[derive(Default, Properties)]
    #[properties(wrapper_type = super::VideoObject)]
    pub struct VideoObject {
        /// Unique id of the video (primary key).
        #[property(name = "id", get, set, type = u32)]
        pub(super) id: Cell<u32>,

        /// Unique id of the video (primary key).
        #[property(name = "title", get, set, type = Object)]
        pub(super) title: RefCell<Option<TitleObject>>,

        /// The video's runtime.
        #[property(name = "duration", get, set, type = String)]
        pub(super) duration: RefCell<String>,

        /// The path to the video.
        #[property(name = "path", get, set, type = String)]
        pub(super) path: RefCell<String>,

        /// List of audio tracks.
        ///
        /// Each item should be a [`crate::ui::data::AudioTrackObject`] instance.
        #[property(name = "audio-tracks", get, set)]
        pub(super) audio_tracks: RefCell<Option<ListStore>>,

        /// List of sutitle tracks.
        ///
        /// Each item should be a [`crate::ui::data::SubtitleTrackObject`] instance.
        #[property(name = "subtitle-tracks", get, set)]
        pub(super) subtitle_tracks: RefCell<Option<ListStore>>,

        /// List of video tracks.
        ///
        /// Each item should be a [`crate::ui::data::VideoTrackObject`] instance.
        #[property(name = "video-tracks", get, set)]
        pub(super) video_tracks: RefCell<Option<ListStore>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for VideoObject {
        const NAME: &'static str = "ArtieVideoObject";
        type Type = super::VideoObject;
    }

    #[glib::derived_properties]
    impl ObjectImpl for VideoObject {}
}

#[cfg(test)]
mod tests {
    // TODO[TESTS]
}

