// Copyright 2026 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! GObject containing information for previewing an audio, subtitle, or video track.

use gst::glib::object::ObjectExt;
use gtk::CheckButton;
use gtk::glib::{self, Object};
use gtk::glib::subclass::prelude::*;

glib::wrapper! {
    pub struct TrackPreviewObject(ObjectSubclass<imp::TrackPreviewObject>);
}

impl TrackPreviewObject {
    pub fn new() -> Self {
        Object::builder()
            .build()
    }

    // TODO
    pub fn bind(&self, button: &CheckButton) {
        let imp = self.imp();

        if imp.binding.borrow().is_some() {
            self.unbind();
        }

        let binding = self.bind_property("selected", button, "active")
            .bidirectional()
            .sync_create()
            .build();
        imp.binding.replace(Some(binding));
    }

    // TODO
    pub fn unbind(&self) {
        let imp = self.imp();
        if let Some(binding) = imp.binding.replace(None) {
            binding.unbind();
        }
    }

    // TODO
    pub fn reset(&self) {
        self.unbind();

        self.set_selected(false);
        self.set_stream_id(None::<String>);
    }
}

impl Default for TrackPreviewObject {
    fn default() -> Self {
        Self::new()
    }
}

mod imp {
    use std::cell::{Cell, RefCell};

    use gtk::glib::{self, Binding, Properties};
    use gtk::prelude::*;
    use gtk::subclass::prelude::*;

    #[derive(Default, Properties)]
    #[properties(wrapper_type = super::TrackPreviewObject)]
    pub struct TrackPreviewObject {
        /// The GStreamer stream identifier for this track.
        #[property(name = "stream-id", get, set, nullable)]
        pub stream_id: RefCell<Option<String>>,

        /// Whether this track is selected for preview.
        #[property(name = "selected", get, set)]
        pub selected: Cell<bool>,

        // TODO
        pub(super) binding: RefCell<Option<Binding>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for TrackPreviewObject {
        const NAME: &'static str = "ArtieTrackPreviewObject";
        type Type = super::TrackPreviewObject;
    }

    #[glib::derived_properties]
    impl ObjectImpl for TrackPreviewObject {}
}

#[cfg(test)]
mod tests {
    // TODO[TESTS]
}
