// Copyright 2026 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! GObject representation of an video track.

use gtk::glib::{self, Object};

use crate::models::VideoTrack;

glib::wrapper! {
    pub struct VideoTrackObject(ObjectSubclass<imp::VideoTrackObject>)
        @extends super::TrackObject;
}

impl VideoTrackObject {
    /// Creates a new video track object instance from a [`VideoTrack`].
    ///
    /// # Panics
    ///
    /// This will panic if the GObject cannot be created.
    pub fn new(video_track: &VideoTrack) -> Self {
        Object::builder()
            .property("codec", video_track.codec.to_string())
            .property("size", &video_track.size)
            .property("aspect-ratio", &video_track.aspect_ratio)
            .build()
    }
}

mod imp {
    use std::cell::RefCell;

    use gtk::glib::{self, Properties};
    use gtk::prelude::*;
    use gtk::subclass::prelude::*;

    #[derive(Default, Properties)]
    #[properties(wrapper_type = super::VideoTrackObject)]
    pub struct VideoTrackObject {
        /// The video's codec.
        ///
        /// This will be the string representation of the track's [`crate::models::VideoCodec`]
        /// value.
        #[property(name = "codec", get, set, type = String)]
        pub codec: RefCell<String>,

        /// The resolution of the video (e.g. 720 x 480).
        #[property(name = "size", get, set, type = String)]
        pub size: RefCell<String>,

        /// The video's aspect ratio (e.g. 16:9)
        #[property(name = "aspect-ratio", get, set, type = String)]
        pub aspect_ratio: RefCell<String>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for VideoTrackObject {
        const NAME: &'static str = "ArtieVideoTrackObject";
        type Type = super::VideoTrackObject;
        type ParentType = super::super::TrackObject;
    }

    #[glib::derived_properties]
    impl ObjectImpl for VideoTrackObject {}

    impl super::super::TrackObjectImpl for VideoTrackObject {}
}

#[cfg(test)]
mod tests {
    // TODO[TESTS]
}



