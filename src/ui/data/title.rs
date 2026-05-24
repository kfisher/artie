// Copyright 2026 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! GObject representation of a title.

use gtk::glib::{self, Object};

use crate::models::Title;
use crate::ui::data::MediaType;

glib::wrapper! {
    pub struct TitleObject(ObjectSubclass<imp::TitleObject>);
}

impl TitleObject {
    /// Creates a new title object instance from a [`models::Title`].
    ///
    /// # Panics
    ///
    /// This will panic if the GObject cannot be created.
    pub fn new(title: &Title) -> Self {
        Object::builder()
            .property("id", title.id)
            .property("index", title.index)
            .property("media-type", MediaType::from(title.media_type))
            .property("title", &title.title)
            .property("season-number", title.season as u32)
            .property("disc-number", title.disc as u32)
            .build()
    }
}

mod imp {
    use std::cell::{Cell, RefCell};

    use gtk::glib::{self, Properties};
    use gtk::prelude::*;
    use gtk::subclass::prelude::*;

    use crate::ui::data::MediaType;

    #[derive(Default, Properties)]
    #[properties(wrapper_type = super::TitleObject)]
    pub struct TitleObject {
        /// Unique id of the title (primary key).
        #[property(name = "id", get, set, type = u32)]
        pub(super) id: Cell<u32>,

        /// The index of the title within the DVD or Blu-ray.
        #[property(name = "index", get, set, type = u8)]
        pub(super) index: Cell<u8>,

        /// The type of media the title is associated with.
        #[property(
            name = "media-type",
            get,
            set,
            type = MediaType,
            builder(MediaType::Movie))
        ]
        pub(super) media_type: Cell<MediaType>,

        /// The movie or show title.
        #[property(name = "title", get, set, type = String)]
        pub(super) title: RefCell<String>,

        /// The disc number the title was copied from.
        #[property(name = "disc-number", get, set, type = u32)]
        pub disc_number: Cell<u32>,

        /// The season number.
        ///
        /// Only valid for television shows. For movies, should be set to zero.
        #[property(name = "season-number", get, set, type = u32)]
        pub season_number: Cell<u32>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for TitleObject {
        const NAME: &'static str = "ArtieTitleObject";
        type Type = super::TitleObject;
    }

    #[glib::derived_properties]
    impl ObjectImpl for TitleObject {}
}

#[cfg(test)]
mod tests {
    // TODO[TESTS]
}

