// Copyright 2026 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! GObject containing the core form elements for editing title information.
//!
//! There are a couple different widgets that are used for editing title information. This object
//! provides the common functionality such as validation, restricting certain fields to numbers,
//! and hiding show specific elements when movie is the selected type.

use gtk::{DropDown, Entry};
use gtk::glib::{self, Object};
use gtk::prelude::*;
use gtk::glib::subclass::prelude::*;

use crate::drive::CopyFormData;
use crate::models::{CopyParamaters, MediaType, SpecialFeatureType};
use crate::ui::data::TitleObject;
use crate::ui::helpers;

glib::wrapper! {
    pub struct TitleFormObject(ObjectSubclass<imp::TitleFormObject>);
}

/// Specifies the use case for the form.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum TitleFormType {
    /// The form is being used to enter copy parameters for a disc.
    ///
    /// In this case, only the items required for the initial copy will be required and validated.
    CopyOnly,

    /// The form is being used to edit all title information.
    #[default]
    Full,
}

impl TitleFormObject {
    /// Create a new builder instance for this object.
    ///
    /// # Args
    ///
    /// `form_type`:  Indicates what this form is being used for. Will effect the required elements
    /// to be specified by the builder as well as what elements are validated.
    ///
    /// # Panics
    ///
    /// This will panic if the GObject cannot be created.
    pub fn builder(form_type: TitleFormType) -> TitleFormBuilder {
        TitleFormBuilder::new(form_type)
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

        imp.episode_number_entry
            .borrow()
            .set_text("");

        imp.episode_count_entry
            .borrow()
            .set_text("");

        imp.special_feature_type_dropdown
            .borrow()
            .set_selected(0);

        imp.special_feature_name_entry
            .borrow()
            .set_text("");

        imp.version_entry
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

    /// Update the form data from the provided copy form data.
    ///
    /// # Args
    ///
    /// `data`  The copy form data.
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

    /// Update the form data from a title data object.
    ///
    /// # Args
    ///
    /// `title`  The title data object.
    pub fn update_from_title(&self, title: &TitleObject) {
        let imp = self.imp();

        imp.media_type_dropdown
            .borrow()
            .set_selected(title.media_type().to_model().as_index());

        imp.title_entry
            .borrow()
            .set_text(&title.title());

        imp.year_entry
            .borrow()
            .set_text(&format!("{}", &title.year()));

        imp.disc_number_entry
            .borrow()
            .set_text(&format!("{}", &title.disc_number()));

        imp.season_number_entry
            .borrow()
            .set_text(&format!("{}", &title.season_number()));

        imp.location_entry
            .borrow()
            .set_text(&title.location());

        imp.memo_entry
            .borrow()
            .set_text(&title.memo());

        let episode_number = title.episode_number();
        if episode_number != 0 {
            imp.episode_number_entry
                .borrow()
                .set_text(&format!("{}", &title.episode_number()));
        } else {
            imp.episode_number_entry
                .borrow()
                .set_text("");
        }

        let episode_count = title.episode_count();
        if episode_count != 0 {
            imp.episode_count_entry
                .borrow()
                .set_text(&format!("{}", &title.episode_number()));
        } else {
            imp.episode_count_entry
                .borrow()
                .set_text(&format!("{}", 1));
        }

        imp.special_feature_type_dropdown
            .borrow()
            .set_selected(title.special_feature_type().to_model().as_index());

        imp.special_feature_name_entry
            .borrow()
            .set_text(&title.special_feature_name());

        imp.version_entry
            .borrow()
            .set_text(&title.version());
    }

    /// Validates the form returning true if valid or false if invalid.
    ///
    /// This will also update the widget's display based on the validity so that the user knows
    /// which fields are invalid.
    pub fn validate(&self) -> bool {
        let mut valid = vec![
            self.validate_title(),
            self.validate_release_year(),
            self.validate_disc_number(),
            self.validate_season_number(),
            self.validate_location(),
            self.validate_memo(),
        ];

        if self.is_copy_only() {
            valid.push(self.validate_episode_number());
            valid.push(self.validate_episode_count());
            valid.push(self.validate_special_feature());
            valid.push(self.validate_version());
        }

        valid.iter().all(|v| *v)
    }

    /// Returns true if this form is configured for copy data only.
    pub fn is_copy_only(&self) -> bool {
        self.imp().form_type.get() == TitleFormType::CopyOnly
    }

    /// Returns true if the type is currently set to show.
    pub fn is_show(&self) -> bool {
        let dropdown = self.imp().media_type_dropdown
            .borrow();
        match MediaType::from_index(dropdown.selected()) {
            Some(media_type) => media_type == MediaType::Show,
            None => false,
        }
    }

    /// Returns true if the special feature type is set to something other then None.
    pub fn is_special_feature(&self) -> bool {
        let dropdown = self.imp().special_feature_type_dropdown
            .borrow();
        match SpecialFeatureType::from_index(dropdown.selected()) {
            Some(special_feature_type) => special_feature_type != SpecialFeatureType::None,
            None => false,
        }
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

        // Only applicable if the title is a show.
        if !self.is_show() {
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

    /// Validates the episode number field and return the result.
    ///
    /// This will update the entry's CSS to reflect is validly.
    fn validate_episode_number(&self) -> bool {
        let imp = self.imp();

        let entry = imp.episode_number_entry
            .borrow();

        // Only applicable if the title is a show and not a special feature.
        if !self.is_show() || self.is_special_feature() {
            helpers::update_validity_style(&entry, true);
            return true
        }

        if let Ok(episode_number) = entry.text().parse::<u16>() && episode_number > 0 {
            helpers::update_validity_style(&entry, true);
            return true;
        };

        helpers::update_validity_style(&entry, false);
        false
    }

    /// Validates the episode count field and return the result.
    ///
    /// This will update the entry's CSS to reflect is validly.
    fn validate_episode_count(&self) -> bool {
        let imp = self.imp();

        let entry = imp.episode_count_entry
            .borrow();

        // Only applicable if the title is a show and not a special feature.
        if !self.is_show() || self.is_special_feature() {
            helpers::update_validity_style(&entry, true);
            return true
        }

        if let Ok(episode_count) = entry.text().parse::<u16>() && episode_count > 0 {
            helpers::update_validity_style(&entry, true);
            return true;
        };

        helpers::update_validity_style(&entry, false);
        false
    }

    /// Validates the special feature type and name fields and return the result.
    ///
    /// This will update the entry's CSS to reflect is validly.
    fn validate_special_feature(&self) -> bool {
        let imp = self.imp();

        let feature_name_entry = imp.special_feature_name_entry
            .borrow();

        if !self.is_special_feature() {
            helpers::update_validity_style(&feature_name_entry, true);
            return true
        }

        let valid = !feature_name_entry
            .text()
            .trim()
            .is_empty();

        helpers::update_validity_style(&feature_name_entry, valid);
        valid
    }

    /// Validates the version field and return the result.
    ///
    /// This will update the entry's CSS to reflect is validly.
    fn validate_version(&self) -> bool {
        // The version field is optional so it is always valid. This function was created anyways
        // should we want to add requirements to the version that would need checked in the future.
        true
    }
}

pub struct TitleFormBuilder {
    /// Indicates what this form is being used for.
    ///
    /// This will effect what elements are required by the builder and will control what elements
    /// will be validated.
    form_type: TitleFormType,

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

    /// The entry for the memo.
    memo_entry: Option<Entry>,

    /// The entry for the episode number
    episode_number_entry: Option<Entry>,

    /// The entry for the episode count
    episode_count_entry: Option<Entry>,

    /// The dropdown for the special feature type.
    special_feature_type_dropdown: Option<DropDown>,

    /// The entry for the special feature name.
    special_feature_name_entry: Option<Entry>,

    /// The entry for the version.
    version_entry: Option<Entry>,
}

impl TitleFormBuilder {
    ///
    /// `form_type`:  Indicates what this form is being used for. Will effect the required elements
    /// to be specified by the builder as well as what elements are validated.
    ///
    pub fn new(form_type: TitleFormType) -> Self {
        Self {
            form_type,
            media_type_dropdown: None,
            title_entry: None,
            year_entry: None,
            disc_number_entry: None,
            season_number_entry: None,
            location_entry: None,
            memo_entry: None,
            episode_number_entry: None,
            episode_count_entry: None,
            special_feature_type_dropdown: None,
            special_feature_name_entry: None,
            version_entry: None,
        }
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
    ///
    /// If the form type is [`FormType::Full`], then it will also panic if any of the following are
    /// not specified:
    /// - episode_number_entry
    /// - episode_count_entry
    /// - special_feature_type_dropdown
    /// - special_feature_name_entry
    /// - version_entry
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

        let is_copy_only = self.form_type == TitleFormType::CopyOnly;

        if let Some(episode_number_entry) = self.episode_number_entry {
            imp.episode_number_entry.replace(episode_number_entry);
        } else if !is_copy_only {
            panic!("The builder expects episode_number_entry to be specified");
        }

        if let Some(episode_count_entry) = self.episode_count_entry {
            imp.episode_count_entry.replace(episode_count_entry);
        } else if !is_copy_only {
            panic!("The builder expects episode_count_entry to be specified");
        }

        if let Some(special_feature_type_dropdown) = self.special_feature_type_dropdown {
            imp.special_feature_type_dropdown.replace(special_feature_type_dropdown);
        } else if !is_copy_only {
            panic!("The builder expects special_feature_type_dropdown to be specified");
        }

        if let Some(special_feature_name_entry) = self.special_feature_name_entry {
            imp.special_feature_name_entry.replace(special_feature_name_entry);
        } else if !is_copy_only {
            panic!("The builder expects special_feature_name_entry to be specified");
        }

        if let Some(version_entry) = self.version_entry {
            imp.version_entry.replace(version_entry);
        } else if !is_copy_only {
            panic!("The builder expects version_entry to be specified");
        }

        imp.form_type.replace(self.form_type);

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

    /// Sets the entry used for entering the episode number.
    ///
    /// # Args
    ///
    /// `entry`:  The entry widget.
    pub fn episode_number_entry(mut self, entry: &Entry) -> Self {
        self.episode_number_entry = Some(entry.clone());
        self
    }

    /// Sets the entry used for entering the episode count.
    ///
    /// # Args
    ///
    /// `entry`:  The entry widget.
    pub fn episode_count_entry(mut self, entry: &Entry) -> Self {
        self.episode_count_entry = Some(entry.clone());
        self
    }

    /// Sets the dropdown used for entering the special feature type.
    ///
    /// # Args
    ///
    /// `dropdown`:  The dropdown widget.
    pub fn special_feature_type_dropdown(mut self, dropdown: &DropDown) -> Self {
        self.special_feature_type_dropdown = Some(dropdown.clone());
        self
    }

    /// Sets the entry used for entering the special feature name.
    ///
    /// # Args
    ///
    /// `entry`:  The entry widget.
    pub fn special_feature_name_entry(mut self, entry: &Entry) -> Self {
        self.special_feature_name_entry = Some(entry.clone());
        self
    }

    /// Sets the entry used for entering the version.
    ///
    /// # Args
    ///
    /// `entry`:  The entry widget.
    pub fn version_entry(mut self, entry: &Entry) -> Self {
        self.version_entry = Some(entry.clone());
        self
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
    use std::cell::{Cell, RefCell};

    use gtk::{DropDown, Entry};
    use gtk::glib::{self, Properties};
    use gtk::subclass::prelude::*;

    use crate::ui::data::TitleFormType;

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

        /// The entry for entering the title's episode number.
        ///
        /// For title's that span multiple episodes, this is the number for the first episode.
        pub(super) episode_number_entry: RefCell<Entry>,

        /// The entry for entering the number of episodes the title covers.
        pub(super) episode_count_entry: RefCell<Entry>,

        /// The dropdown for selecting the special feature type.
        pub(super) special_feature_type_dropdown: RefCell<DropDown>,

        /// The entry for entering the special feature name.
        pub(super) special_feature_name_entry: RefCell<Entry>,

        /// The entry for entering the title version (e.g. directors cut).
        pub(super) version_entry: RefCell<Entry>,

        /// Specifies the use case for the form.
        pub(super) form_type: Cell<TitleFormType>,
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
