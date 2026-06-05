// Copyright 2026 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! GObject base class for track objects.

use gtk::glib;
use gtk::subclass::prelude::*;

glib::wrapper! {
    pub struct TrackObject(ObjectSubclass<imp::TrackObject>);
}

pub trait TrackObjectImpl: ObjectImpl {}

unsafe impl<T: TrackObjectImpl> IsSubclassable<T> for TrackObject {}

mod imp {
    use std::cell::{Cell, RefCell};

    use gtk::glib::{self, Properties};
    use gtk::prelude::*;
    use gtk::subclass::prelude::*;

    #[derive(Default, Properties)]
    #[properties(wrapper_type = super::TrackObject)]
    pub struct TrackObject {
        /// The GStreamer stream identifier for this track.
        #[property(name = "stream-id", get, set, type = Option<String>)]
        pub stream_id: RefCell<Option<String>>,

        /// Whether this track is selected for preview.
        #[property(name = "preview", get, set, type = bool)]
        pub preview: Cell<bool>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for TrackObject {
        const NAME: &'static str = "ArtieTrackObject";
        type Type = super::TrackObject;
    }

    #[glib::derived_properties]
    impl ObjectImpl for TrackObject {}
}

#[cfg(test)]
mod tests {
    // TODO[TESTS]
}
