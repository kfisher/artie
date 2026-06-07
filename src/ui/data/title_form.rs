// Copyright 2026 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! GObject containing the core form elements for editing title information.
//!
//! There are a couple different widgets that are used for editing title information. This object
//! provides the common functionality such as validation, restricting certain fields to numbers,
//! and hiding show specific elements when movie is the selected type.

use gtk::{DropDown, Entry, Widget};
use gtk::glib::{self, Object};
use gtk::prelude::*;
use gtk::glib::subclass::prelude::*;

use crate::drive::CopyFormData;
use crate::models::{CopyParamaters, MediaType};
use crate::ui::helpers;

glib::wrapper! {
    pub struct TitleFormObject(ObjectSubclass<imp::TitleFormObject>);
}

impl TitleFormObject {
    /// Create a new builder instance for this object.
    ///
    /// # Panics
    ///
    /// This will panic if the GObject cannot be created.
    pub fn builder() -> TitleFormBuilder {
        TitleFormBuilder::new()
    }

    /// Clears the form's values.
    pub fn clear(&self) {
        let imp = self.imp();

        imp.title_entry
            .borrow()
            .set_text("");

        imp.year_entry
            .borrow()
            .set_text("");

        imp.disc_number_entry
            .borrow()
            .set_text("");

        imp.season_number_entry
            .borrow()
            .set_text("");

        imp.location_entry
            .borrow()
            .set_text("");

        imp.memo_entry
            .borrow()
            .set_text("");
    }

    /// Gets the media type dropdown widget.
    pub fn media_type_dropdown(&self) -> DropDown {
        self.imp().media_type_dropdown
            .borrow()
            .clone()
    }

    /// Gets the title entry widget.
    pub fn title_entry(&self) -> Entry {
        self.imp().title_entry
            .borrow()
            .clone()
    }

    /// Gets the year entry widget.
    pub fn year_entry(&self) -> Entry {
        self.imp().year_entry
            .borrow()
            .clone()
    }

    /// Gets the disc number entry widget.
    pub fn disc_number_entry(&self) -> Entry {
        self.imp().disc_number_entry
            .borrow()
            .clone()
    }

    /// Gets the season number widget.
    pub fn season_number_entry(&self) -> Entry {
        self.imp().season_number_entry
            .borrow()
            .clone()
    }

    /// Gets the location entry widget.
    pub fn location_entry(&self) -> Entry {
        self.imp().location_entry
            .borrow()
            .clone()
    }

    /// Gets the memo entry widget.
    pub fn memo_entry(&self) -> Entry {
        self.imp().memo_entry
            .borrow()
            .clone()
    }

    /// Subscribe to changes to the media type.
    ///
    /// # Args
    ///
    /// `f`:  Callback function called when the media type changes. Will only be called when the
    /// selected media type is valid.
    pub fn connect_media_type_changed<F>(&self, f: F)
    where
        F: Fn(MediaType) + 'static
    {
        self.imp().media_type_dropdown
            .borrow()
            .connect_selected_notify(move |type_dropdown| {
                if let Some(media_type) = MediaType::from_index(type_dropdown.selected()) {
                    f(media_type);
                }
            });
    }

    /// Subscribe to changes to the title.
    ///
    /// # Args
    ///
    /// `f`:  Callback function called when the title changes.
    ///
    /// # Panics
    ///
    /// This will panic if the delegate for the entry is `None`.
    pub fn connect_title_changed<F>(&self, f: F)
    where
        F: Fn(&str) + 'static
    {
        self.imp().title_entry
            .borrow()
            .delegate()
            .unwrap()
            .connect_text_notify(move |entry| {
                f(&entry.text());
            });
    }

    /// Subscribe to changes to the year.
    ///
    /// # Args
    ///
    /// `f`:  Callback function called when the year changes.
    ///
    /// # Panics
    ///
    /// This will panic if the delegate for the entry is `None`.
    pub fn connect_year_changed<F>(&self, f: F)
    where
        F: Fn(&str) + 'static
    {
        self.imp().year_entry
            .borrow()
            .delegate()
            .unwrap()
            .connect_text_notify(move |entry| {
                f(&entry.text());
            });
    }

    /// Subscribe to changes to the disc number.
    ///
    /// # Args
    ///
    /// `f`:  Callback function called when the disc number changes.
    ///
    /// # Panics
    ///
    /// This will panic if the delegate for the entry is `None`.
    pub fn connect_disc_number_changed<F>(&self, f: F)
    where
        F: Fn(&str) + 'static
    {
        self.imp().disc_number_entry
            .borrow()
            .delegate()
            .unwrap()
            .connect_text_notify(move |entry| {
                f(&entry.text());
            });
    }

    /// Subscribe to changes to the season number.
    ///
    /// # Args
    ///
    /// `f`:  Callback function called when the season number changes.
    ///
    /// # Panics
    ///
    /// This will panic if the delegate for the entry is `None`.
    pub fn connect_season_number_changed<F>(&self, f: F)
    where
        F: Fn(&str) + 'static
    {
        self.imp().season_number_entry
            .borrow()
            .delegate()
            .unwrap()
            .connect_text_notify(move |entry| {
                f(&entry.text());
            });
    }

    /// Subscribe to changes to the storage location.
    ///
    /// # Args
    ///
    /// `f`:  Callback function called when the storage location changes.
    ///
    /// # Panics
    ///
    /// This will panic if the delegate for the entry is `None`.
    pub fn connect_location_changed<F>(&self, f: F)
    where
        F: Fn(&str) + 'static
    {
        self.imp().location_entry
            .borrow()
            .delegate()
            .unwrap()
            .connect_text_notify(move |entry| {
                f(&entry.text());
            });
    }

    /// Subscribe to changes to the memo.
    ///
    /// # Args
    ///
    /// `f`:  Callback function called when the memo changes.
    ///
    /// # Panics
    ///
    /// This will panic if the delegate for the entry is `None`.
    pub fn connect_memo_changed<F>(&self, f: F)
    where
        F: Fn(&str) + 'static
    {
        self.imp().memo_entry
            .borrow()
            .delegate()
            .unwrap()
            .connect_text_notify(move |entry| {
                f(&entry.text());
            });
    }

    /// Gets the copy parameters based off the current form values.
    pub fn get_copy_parameters(&self) -> CopyParamaters {
        let imp = self.imp();

        let media_type = MediaType::from_index(imp.media_type_dropdown.borrow().selected())
            .unwrap_or_default();

        let title = imp.title_entry
            .borrow()
            .text();

        let release_year = imp.year_entry
            .borrow()
            .text()
            .parse::<u16>()
            .unwrap_or_default();

        let season_number = imp.season_number_entry
            .borrow()
            .text()
            .parse::<u16>()
            .unwrap_or_default();

        let disc_number = imp.disc_number_entry
            .borrow()
            .text()
            .parse::<u16>()
            .unwrap_or_default();

        let location = imp.location_entry
            .borrow()
            .text();

        let memo = imp.memo_entry
            .borrow()
            .text();

        CopyParamaters {
            media_type,
            title: title.into(),
            release_year,
            season_number,
            disc_number,
            location: location.into(),
            memo: memo.into(),
        }
    }

    /// Sets the current values of the form to the provided data.
    pub fn update_from_copy_form_data(&self, data: &CopyFormData) {
        let imp = self.imp();

        if let Some(media_type) = MediaType::from_string(&data.media_type) {
            imp.media_type_dropdown.borrow().set_selected(media_type.as_index());
        };

        imp.title_entry
            .borrow()
            .set_text(&data.title);

        imp.year_entry
            .borrow()
            .set_text(&data.year);
        
        imp.disc_number_entry
            .borrow()
            .set_text(&data.disc_number);

        imp.season_number_entry
            .borrow()
            .set_text(&data.season_number);

        imp.location_entry
            .borrow()
            .set_text(&data.storage_location);

        imp.memo_entry
            .borrow()
            .set_text(&data.memo);
    }

    /// Validates the form returning true if valid or false if invalid.
    ///
    /// This will also update the widget's display based on the validity so that the user knows
    /// which fields are invalid.
    pub fn validate(&self) -> bool {
        let valid = [
            self.validate_title(),
            self.validate_release_year(),
            self.validate_disc_number(),
            self.validate_season_number(),
            self.validate_location(),
            self.validate_memo(),
        ];

        valid.iter().all(|v| *v)
    }

    /// Validates the title and return the result.
    ///
    /// This will update the entry's CSS to reflect is validly.
    fn validate_title(&self) -> bool {
        let entry = self.imp()
            .title_entry
            .borrow();
        let valid = !entry
            .text()
            .trim()
            .is_empty();

        helpers::update_validity_style(&entry, valid);
        valid
    }

    /// Validates the release year and return the result.
    ///
    /// This will update the entry's CSS to reflect is validly.
    fn validate_release_year(&self) -> bool {
        let entry = self.imp()
            .year_entry
            .borrow();
        if let Ok(year) = entry.text().parse::<u16>() && (1000..=9999).contains(&year) {
            helpers::update_validity_style(&entry, true);
            return true;
        };

        helpers::update_validity_style(&entry, false);
        false
    }

    /// Validates the disc number and return the result.
    ///
    /// This will update the entry's CSS to reflect is validly.
    fn validate_disc_number(&self) -> bool {
        let entry = self.imp()
            .disc_number_entry
            .borrow();
        if let Ok(disc_number) = entry.text().parse::<u16>() && disc_number > 0 {
            helpers::update_validity_style(&entry, true);
            return true;
        };

        helpers::update_validity_style(&entry, false);
        false
    }

    /// Validates the season number and return the result.
    ///
    /// This will update the entry's CSS to reflect is validly.
    fn validate_season_number(&self) -> bool {
        let imp = self.imp();

        let entry = imp.season_number_entry
            .borrow();

        let media_type_dropdown = imp.media_type_dropdown
            .borrow();

        if let Some(media_type) = MediaType::from_index(media_type_dropdown.selected()) && media_type != MediaType::Show {
            helpers::update_validity_style(&entry, true);
            return true
        }

        if let Ok(season_number) = entry.text().parse::<u16>() && season_number > 0 {
            helpers::update_validity_style(&entry, true);
            return true;
        };

        helpers::update_validity_style(&entry, false);
        false
    }

    /// Validates the location field and return the result.
    ///
    /// This will update the entry's CSS to reflect is validly.
    fn validate_location(&self) -> bool {
        let entry = self.imp().location_entry
            .borrow();
        let valid = !entry
            .text()
            .trim()
            .is_empty();

        helpers::update_validity_style(&entry, valid);
        valid
    }

    /// Validates the memo field and return the result.
    ///
    /// This will update the entry's CSS to reflect is validly.
    fn validate_memo(&self) -> bool {
        // The memo is optional so it is always valid. This function was created anyways should we
        // want to add requirements to the memo that would need checked in the future.
        true
    }
}

#[derive(Default)]
pub struct TitleFormBuilder {
    /// Dropdown used to select the type of media.
    media_type_dropdown: Option<DropDown>,

    /// The entry for the movie title.
    title_entry: Option<Entry>,

    /// The entry for the release year.
    year_entry: Option<Entry>,

    /// The entry for the disc number.
    disc_number_entry: Option<Entry>,

    /// The entry for the season number.
    season_number_entry: Option<Entry>,

    /// The entry for the location.
    location_entry: Option<Entry>,

    /// The entry for the meoy.
    memo_entry: Option<Entry>,

    /// List of widgets that should be hidden if the selected media type is movie.
    hide_if_movie: Vec<Widget>,
}

impl TitleFormBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    /// Builds the [`TitleFormObject`] instance consuming self in the process.
    ///
    /// # Panics
    ///
    /// This will panic if any one of the following were not specified before calling build:
    /// - media_type_dropdown
    /// - title_entry
    /// - year_entry
    /// - disc_number_entry
    /// - season_number_entry
    /// - location_entry
    /// - memo_entry
    pub fn build(self) -> TitleFormObject {
        let obj: TitleFormObject = Object::builder()
            .build();

        let imp = obj.imp();

        if let Some(media_type_dropdown) = self.media_type_dropdown {
            imp.media_type_dropdown.replace(media_type_dropdown);
        } else {
            panic!("The builder expects media_type_dropdown to be specified");
        }

        if let Some(title_entry) = self.title_entry {
            imp.title_entry.replace(title_entry);
        } else {
            panic!("The builder expects title_entry to be specified");
        }

        if let Some(year_entry) = self.year_entry {
            imp.year_entry.replace(year_entry);
        } else {
            panic!("The builder expects year_entry to be specified");
        }

        if let Some(disc_number_entry) = self.disc_number_entry {
            imp.disc_number_entry.replace(disc_number_entry);
        } else {
            panic!("The builder expects disc_number_entry to be specified");
        }

        if let Some(season_number_entry) = self.season_number_entry {
            imp.season_number_entry.replace(season_number_entry);
        } else {
            panic!("The builder expects season_number_entry to be specified");
        }

        if let Some(location_entry) = self.location_entry {
            imp.location_entry.replace(location_entry);
        } else {
            panic!("The builder expects location_entry to be specified");
        }

        if let Some(memo_entry) = self.memo_entry {
            imp.memo_entry.replace(memo_entry);
        } else {
            panic!("The builder expects memo_entry to be specified");
        }

        for widget in self.hide_if_movie {
            imp.media_type_dropdown
                .borrow()
                .bind_property("selected", &widget, "visible")
                .transform_to(|_, selected: u32| hide_if_movie(selected))
                .sync_create()
                .build();
        }

        obj
    }

    // TODO: There is a hidden dependency/assumption in how the dropdown was created.
    // TODO: The year, disc, and season fields also have a minor assumption about max number of
    //       digits. Minor issue.

    /// Sets the dropdown used to represent the media type dropdown.
    ///
    /// # Args
    ///
    /// `dropdown`:  The dropdown widget.
    pub fn media_type_dropdown(mut self, dropdown: &DropDown) -> Self {
        self.media_type_dropdown = Some(dropdown.clone());
        self
    }

    /// Sets the entry used for entering the title's name.
    ///
    /// # Args
    ///
    /// `entry`:  The entry widget.
    pub fn title_entry(mut self, entry: &Entry) -> Self {
        self.title_entry = Some(entry.clone());
        self
    }

    /// Sets the entry used for entering the title's release year.
    ///
    /// This entry will automatically be restricted to only allow numbers to be entered.
    ///
    /// # Args
    ///
    /// `entry`:  The entry widget.
    pub fn year_entry(mut self, entry: &Entry) -> Self {
        restrict_to_numbers(entry);
        self.year_entry = Some(entry.clone());
        self
    }

    /// Sets the entry used for entering the number of the disc the title was copied from.
    ///
    /// This entry will automatically be restricted to only allow numbers to be entered.
    ///
    /// # Args
    ///
    /// `entry`:  The entry widget.
    pub fn disc_number_entry(mut self, entry: &Entry) -> Self {
        restrict_to_numbers(entry);
        self.disc_number_entry = Some(entry.clone());
        self
    }

    /// Sets the entry used for entering the season number for the title.
    ///
    /// This entry will automatically be restricted to only allow numbers to be entered.
    ///
    /// # Args
    ///
    /// `entry`:  The entry widget.
    pub fn season_number_entry(mut self, entry: &Entry) -> Self {
        restrict_to_numbers(entry);
        self.season_number_entry = Some(entry.clone());
        self
    }

    /// Sets the entry used for entering the location where the disc is stored that the title was
    /// copied from.
    ///
    /// # Args
    ///
    /// `entry`:  The entry widget.
    pub fn location_entry(mut self, entry: &Entry) -> Self {
        self.location_entry = Some(entry.clone());
        self
    }

    /// Sets the entry used for entering the memo.
    ///
    /// # Args
    ///
    /// `entry`:  The entry widget.
    pub fn memo_entry(mut self, entry: &Entry) -> Self {
        self.memo_entry = Some(entry.clone());
        self
    }

    /// Bind the provided widget's "visible" property to the state of the media type selection so
    /// that it will be hidden when the selected type is movie.
    ///
    /// # Args
    ///
    /// `widget`:  The widget to bind.
    pub fn hide_if_movie(mut self, widget: &Widget) -> Self {
        self.hide_if_movie.push(widget.clone());
        self
    }
}

/// Returns `true` if the provided media type is a show indicating the associated item should be
/// visible or `false` if the media type is something else (e.g. a show) to indicate the item
/// should be hidden.
///
/// # Args
///
/// `selected`:  The numeric (index) representation of the media type.
fn hide_if_movie(selected: u32) -> Option<bool>  {
    match MediaType::from_index(selected) {
        Some(media_type) => Some(media_type == MediaType::Show),
        None => Some(false),
    }
}

/// Adds a signal connection that will restrict the entry's values to numbers only.
///
/// # Args
///
/// `entry`:  The entry that should only allow numbers to be entered.
fn restrict_to_numbers(entry: &Entry) {
    entry.delegate()
        .unwrap()
        .connect_insert_text(helpers::number_only_insert_text);
}

mod imp {
    use std::cell::RefCell;

    use gtk::{DropDown, Entry};
    use gtk::glib::{self, Properties};
    use gtk::subclass::prelude::*;

    #[derive(Default, Properties)]
    #[properties(wrapper_type = super::TitleFormObject)]
    pub struct TitleFormObject {
        /// Dropdown used to select the type of media.
        pub(super) media_type_dropdown: RefCell<DropDown>,

        /// The entry for the movie title.
        pub(super) title_entry: RefCell<Entry>,

        /// The entry for the release year.
        pub(super) year_entry: RefCell<Entry>,

        /// The entry for the disc number.
        pub(super) disc_number_entry: RefCell<Entry>,

        /// The entry for the season number.
        pub(super) season_number_entry: RefCell<Entry>,

        /// The entry for the location.
        pub(super) location_entry: RefCell<Entry>,

        /// The entry for the meoy.
        pub(super) memo_entry: RefCell<Entry>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for TitleFormObject {
        const NAME: &'static str = "ArtieTitleFormObject";
        type Type = super::TitleFormObject;
    }

    #[glib::derived_properties]
    impl ObjectImpl for TitleFormObject {}
}

#[cfg(test)]
mod tests {
    // TODO[TESTS]
}
