// Copyright 2026 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! GObject representation of a video.

use gtk::glib::{self, Object};

use crate::models::Video;
use crate::ui::data::TitleObject;
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

        Object::builder()
            .property("id", video.id)
            .property("title", title)
            .property("duration", helpers::format_elapsed_time(&video.duration))
            .build()
    }
}

mod imp {
    use std::cell::{Cell, RefCell};

    use gtk::glib::{self, Object, Properties};
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

