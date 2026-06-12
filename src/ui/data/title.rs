// Copyright 2026 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! GObject representation of a title.

use gtk::glib::{self, Object};

use crate::models::Title;
use crate::ui::data::{MediaType, SpecialFeatureType};

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
        let (special_feature_type, special_feature_name) = match &title.special_feature {
            Some(special_feature) => {
                (SpecialFeatureType::from(special_feature.kind), special_feature.name.clone())
            },
            None => {
                (SpecialFeatureType::None, String::default())
            }
        };

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
            .property("episode-number", title.episode_number as u32)
            .property("episode-count", title.episode_count as u32)
            .property("special-feature-type", special_feature_type)
            .property("special-feature-name", special_feature_name)
            .property("version", &title.version)
            .build()
    }

    /// Converts the GLib object to the standard model type.
    pub fn to_model(&self) -> Title {
        Title {
            id: self.id(),
            index: self.index(),
            media_type: self.media_type().to_model(),
            title: self.title(),
            year: self.year() as u16,
            season: self.season_number() as u16,
            episode_number: self.episode_number() as u16,
            episode_count: self.episode_count() as u16,
            special_feature: self.special_feature_type().to_special_feature(&self.special_feature_name()),
            version: self.version(),
            disc: self.disc_number() as u16,
            location: self.location(),
            memo: self.memo(),
            videos: None,
        }
    }
}

mod imp {
    use std::cell::{Cell, RefCell};

    use gtk::glib::{self, Properties};
    use gtk::prelude::*;
    use gtk::subclass::prelude::*;

    use crate::ui::data::{MediaType, SpecialFeatureType};

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
            builder(MediaType::Movie)
        )]
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

        /// The episode number.
        ///
        /// In the case that this video covers multiple episodes, this will be the number for the
        /// first episode.
        ///
        /// Only valid for television shows. For movies, should be set to zero.
        #[property(name = "episode-number", get, set, type = u32)]
        pub(super) episode_number: Cell<u32>,

        /// The number of episodes this title covers.
        ///
        /// Only valid for television shows. For movies, should be set to zero.
        #[property(name = "episode-count", get, set, type = u32)]
        pub(super) episode_count: Cell<u32>,

        /// The type of the special feature (if applicable).
        #[property(
            name = "special-feature-type",
            get,
            set,
            type = SpecialFeatureType,
            builder(SpecialFeatureType::None)
        )]
        pub(super) special_feature_type: Cell<SpecialFeatureType>,

        /// The name of the special feature (if applicable).
        #[property(name = "special-feature-name", get, set, type = String)]
        pub(super) special_feature_name: RefCell<String>,

        /// The version of the title (e.g. Directors Cut, 1080p, etc.)
        ///
        /// This should be empty for the default version of the title.
        #[property(name = "version", get, set, type = String)]
        pub(super) version: RefCell<String>,
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

