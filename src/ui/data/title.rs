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
            .property("disc-number", title.disc as u32)
            .property("id", title.id)
            .property("index", title.index)
            .property("location", &title.location)
            .property("media-type", MediaType::from(title.media_type))
            .property("memo", &title.memo)
            .property("season-number", title.season as u32)
            .property("title", &title.title)
            .property("year", title.year as u32)
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

        /// The release year.
        ///
        /// For television shows, this is the release year of the first season.
        #[property(name = "year", get, set, type = u32)]
        pub year: Cell<u32>,

        /// The disc number the title was copied from.
        #[property(name = "disc-number", get, set, type = u32)]
        pub disc_number: Cell<u32>,

        /// The season number.
        ///
        /// Only valid for television shows. For movies, should be set to zero.
        #[property(name = "season-number", get, set, type = u32)]
        pub season_number: Cell<u32>,

        /// The physical location of the disc being copied.
        #[property(name = "location", get, set, type = String)]
        pub(super) location: RefCell<String>,

        /// Additional information/context provided by the user.
        #[property(name = "memo", get, set, type = String)]
        pub(super) memo: RefCell<String>,
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

