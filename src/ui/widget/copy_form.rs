// Copyright 2025 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! Widget for entering copy parameters.

use std::fmt::{self, Display, Formatter};

use gtk::{
    Align,
    Box,
    DropDown,
    Entry,
    Label,
    Orientation,
    StringList
};
use gtk::glib::{self, Object};
use gtk::prelude::*;
use gtk::subclass::prelude::*;

use crate::drive::FormData;
use crate::models::{CopyParamaters, MediaType};
use crate::ui::helpers;

glib::wrapper! {
    pub struct CopyFormWidget(ObjectSubclass<imp::CopyFormWidget>)
        @extends gtk::Box,
                 gtk::Widget,
        @implements gtk::Accessible,
                    gtk::Buildable,
                    gtk::ConstraintTarget,
                    gtk::Orientable;
}

impl CopyFormWidget {
    /// Constructs a new copy form instance.
    ///
    /// # Panics
    ///
    /// This will panic if the GObject cannot be created.
    pub fn new() -> Self {
        Object::builder().build()
    }

    /// Clears the form's values.
    pub fn clear(&self) {
        let imp = self.imp();
        imp.title_entry.borrow().set_text("");
        imp.year_entry.borrow().set_text("");
        imp.disc_number_entry.borrow().set_text("");
        imp.season_number_entry.borrow().set_text("");
        imp.location_entry.borrow().set_text("");
        imp.memo_entry.borrow().set_text("");
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
        self.imp().type_dropdown.borrow().connect_selected_notify(move |type_dropdown| {
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
        let title_entry = self.imp().title_entry
            .borrow()
            .clone();
        if let Some(delegate) = title_entry.delegate() {
            delegate.connect_text_notify(move |entry| {
                f(&entry.text());
            });
        } else {
            panic!("failed to get delegate for title");
        }
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
        let year_entry = self.imp().year_entry
            .borrow()
            .clone();
        if let Some(delegate) = year_entry.delegate() {
            delegate.connect_text_notify(move |entry| {
                f(&entry.text());
            });
        } else {
            panic!("failed to get delegate for year");
        }
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
        let disc_number_entry = self.imp().disc_number_entry
            .borrow()
            .clone();
        if let Some(delegate) = disc_number_entry.delegate() {
            delegate.connect_text_notify(move |entry| {
                f(&entry.text());
            });
        } else {
            panic!("failed to get delegate for disc_number");
        }
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
        let season_number_entry = self.imp().season_number_entry
            .borrow()
            .clone();
        if let Some(delegate) = season_number_entry.delegate() {
            delegate.connect_text_notify(move |entry| {
                f(&entry.text());
            });
        } else {
            panic!("failed to get delegate for season_number");
        }
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
        let location_entry = self.imp().location_entry
            .borrow()
            .clone();
        if let Some(delegate) = location_entry.delegate() {
            delegate.connect_text_notify(move |entry| {
                f(&entry.text());
            });
        } else {
            panic!("failed to get delegate for location");
        }
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
        let memo_entry = self.imp().memo_entry
            .borrow()
            .clone();
        if let Some(delegate) = memo_entry.delegate() {
            delegate.connect_text_notify(move |entry| {
                f(&entry.text());
            });
        } else {
            panic!("failed to get delegate for memo");
        }
    }

    /// Gets the copy parameters based off the current form values.
    pub fn get_copy_parameters(&self) -> CopyParamaters {
        let imp = self.imp();

        let media_type = MediaType::from_index(imp.type_dropdown.borrow().selected())
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
    pub fn set_form_data(&self, form_data: &FormData) {
        let imp = self.imp();
        if let Some(media_type) = MediaType::from_string(&form_data.media_type) {
            imp.type_dropdown.borrow().set_selected(media_type.as_index());
        };
        imp.title_entry.borrow().set_text(&form_data.title);
        imp.year_entry.borrow().set_text(&form_data.year);
        imp.disc_number_entry.borrow().set_text(&form_data.disc_number);
        imp.season_number_entry.borrow().set_text(&form_data.season_number);
        imp.location_entry.borrow().set_text(&form_data.storage_location);
        imp.memo_entry.borrow().set_text(&form_data.memo);
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

    /// Builds the widget.
    ///
    /// Called by the implementation ([`imp::CopyFormWidget`]) when constructed.
    fn build_ui(&self) {
        let type_model = StringList::new(&[
            MediaType::Movie.as_str(),
            MediaType::Show.as_str(),
        ]);

        let type_dropdown = DropDown::builder()
            .model(&type_model)
            .width_request(116)
            .build();

        let type_label = Label::builder()
            .halign(Align::Start)
            .label("Type")
            .margin_start(8)
            .build();

        let type_field = Box::builder()
            .orientation(Orientation::Vertical)
            .build();
        type_field.append(&type_dropdown);
        type_field.append(&type_label);

        let title_entry = Entry::builder()
            .build();

        let title_label = Label::builder()
            .halign(Align::Start)
            .label("Title")
            .margin_start(8)
            .build();

        let title_field = Box::builder()
            .hexpand(true)
            .orientation(Orientation::Vertical)
            .build();
        title_field.append(&title_entry);
        title_field.append(&title_label);

        let year_entry = Entry::builder()
            .max_length(4)
            .max_width_chars(12)
            .build();

        let release_year_label = Label::builder()
            .halign(Align::Start)
            .label("Release Year")
            .margin_start(8)
            .build();

        let release_year_field = Box::builder()
            .orientation(Orientation::Vertical)
            .build();
        release_year_field.append(&year_entry);
        release_year_field.append(&release_year_label);

        let season_number_entry = Entry::builder()
            .max_length(2)
            .max_width_chars(8)
            .build();

        let season_number_label = Label::builder()
            .halign(Align::Start)
            .label("Season #")
            .margin_start(8)
            .build();

        let season_number_field = Box::builder()
            .orientation(Orientation::Vertical)
            .build();
        season_number_field.append(&season_number_entry);
        season_number_field.append(&season_number_label);

        let disc_number_entry = Entry::builder()
            .max_length(2)
            .max_width_chars(8)
            .build();

        let disc_number_label = Label::builder()
            .halign(Align::Start)
            .label("Disc #")
            .margin_start(8)
            .build();

        let disc_number_field = Box::builder()
            .orientation(Orientation::Vertical)
            .build();
        disc_number_field.append(&disc_number_entry);
        disc_number_field.append(&disc_number_label);

        let location_entry = Entry::builder()
            .build();

        let location_label = Label::builder()
            .halign(Align::Start)
            .label("Storage Location")
            .margin_start(8)
            .build();

        let location_field = Box::builder()
            .hexpand(true)
            .orientation(Orientation::Vertical)
            .build();
        location_field.append(&location_entry);
        location_field.append(&location_label);

        let memo_entry = Entry::builder()
            .build();

        let memo_label = Label::builder()
            .halign(Align::Start)
            .label("Memo (optional)")
            .margin_start(8)
            .build();

        let memo_field = Box::builder()
            .hexpand(true)
            .orientation(Orientation::Vertical)
            .build();
        memo_field.append(&memo_entry);
        memo_field.append(&memo_label);

        let form_row_0 = Box::builder()
            .orientation(Orientation::Horizontal)
            .spacing(8)
            .build();
        form_row_0.append(&type_field);
        form_row_0.append(&title_field);
        form_row_0.append(&release_year_field);

        let form_row_1 = Box::builder()
            .orientation(Orientation::Horizontal)
            .spacing(8)
            .build();
        form_row_1.append(&season_number_field);
        form_row_1.append(&disc_number_field);
        form_row_1.append(&location_field);
        form_row_1.append(&memo_field);

        self.set_margin_bottom(8);
        self.set_margin_end(8);
        self.set_margin_start(8);
        self.set_margin_top(8);
        self.set_orientation(Orientation::Vertical);
        self.set_hexpand(true);
        self.set_vexpand(true);
        self.set_spacing(8);
        self.append(&form_row_0);
        self.append(&form_row_1);

        let imp = self.imp();
        imp.type_dropdown.replace(type_dropdown);
        imp.title_entry.replace(title_entry);
        imp.year_entry.replace(year_entry);
        imp.disc_number_entry.replace(disc_number_entry);
        imp.season_number_field.replace(season_number_field);
        imp.season_number_entry.replace(season_number_entry);
        imp.location_entry.replace(location_entry);
        imp.memo_entry.replace(memo_entry);
    }

    /// Configures the bindings.
    ///
    /// Called by the implementation ([`imp::CopyFormWidget`]) when constructed.
    fn setup_bindings(&self) {
        let imp = self.imp();

        let type_dropdown = imp.type_dropdown
            .borrow();
        let season_number_field = imp.season_number_field
            .borrow()
            .clone();
        type_dropdown
            .bind_property("selected", &season_number_field, "visible")
            .transform_to(|_, selected: u32| {
                match MediaType::from_index(selected) {
                    Some(media_type) => Some(media_type == MediaType::Show),
                    None => Some(false),
                }
            })
            .sync_create()
            .build();
    }

    /// Configures the signals and callbacks.
    ///
    /// Called by the implementation ([`imp::CopyFormWidget`]) when constructed.
    ///
    /// # Panics
    ///
    /// This will panic if any of the required delegates are `None`.
    fn setup_callbacks(&self) {
        let imp = self.imp();

        let year_entry = imp.year_entry
            .borrow()
            .clone();
        if let Some(delegate) = year_entry.delegate() {
            delegate.connect_insert_text(number_only_insert_text);
        } else {
            panic!("failed to get delegate for year");
        }

        let disc_number_entry = imp.disc_number_entry
            .borrow();
        if let Some(delegate) = disc_number_entry.delegate() {
            delegate.connect_insert_text(number_only_insert_text);
        } else {
            panic!("failed to get delegate for disc number");
        }

        let season_number_entry = imp.season_number_entry
            .borrow();
        if let Some(delegate) = season_number_entry.delegate() {
            delegate.connect_insert_text(number_only_insert_text);
        } else {
            panic!("failed to get delegate for season number");
        }
    }

    /// Validates the title and return the result.
    ///
    /// This will update the entry's CSS to reflect is validly.
    fn validate_title(&self) -> bool {
        let entry = self.imp().title_entry.borrow();
        let valid = !entry.text().trim().is_empty();

        helpers::update_validity_style(&entry, valid);

        valid
    }

    /// Validates the release year and return the result.
    ///
    /// This will update the entry's CSS to reflect is validly.
    fn validate_release_year(&self) -> bool {
        let entry = self.imp().year_entry.borrow();
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
        let entry = self.imp().disc_number_entry.borrow();
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
        let entry = self.imp().season_number_entry.borrow();

        let type_dropdown = self.imp().type_dropdown.borrow();
        if let Some(media_type) = MediaType::from_index(type_dropdown.selected()) && media_type != MediaType::Show {
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
        let entry = self.imp().location_entry.borrow();
        let valid = !entry.text().trim().is_empty();

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

impl Display for CopyFormWidget {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let imp = self.imp();
        write!(
            f,
            "{{ type: '{}', title: '{}', year: '{}', disc: '{}', season: '{}', location: '{}', memo: '{}'}}",
            MediaType::from_index(imp.type_dropdown.borrow().selected())
                .map_or_else(|| "", |media_type| media_type.as_str()),
            imp.title_entry.borrow().text(),
            imp.year_entry.borrow().text(),
            imp.disc_number_entry.borrow().text(),
            imp.season_number_entry.borrow().text(),
            imp.location_entry.borrow().text(),
            imp.memo_entry.borrow().text(),
        )
    }
}

impl Default for CopyFormWidget {
    fn default() -> Self {
        Self::new()
    }
}

/// insert-text signal handler that restricts input to numbers only.
fn number_only_insert_text(entry: &gtk::Editable, text: &str, _position: &mut i32) {
    const NUMBERS: &str = "0123456789";
    let filtered: String = text.chars()
        .filter(|c| NUMBERS.contains(*c))
        .collect();
    if filtered != text {
        glib::signal::signal_stop_emission_by_name(entry, "insert-text");
    }
}

mod imp {
    use std::cell::RefCell;

    use gtk::{Box, Entry, DropDown};
    use gtk::glib;
    use gtk::subclass::prelude::*;

    #[derive(Default)]
    pub struct CopyFormWidget {
        /// Dropdown used to select the type of media.
        pub(super) type_dropdown: RefCell<DropDown>,

        /// The entry for the movie title.
        pub(super) title_entry: RefCell<Entry>,

        /// The entry for the release year.
        pub(super) year_entry: RefCell<Entry>,

        /// The entry for the disc number.
        pub(super) disc_number_entry: RefCell<Entry>,

        /// The label and entry for the season number.
        pub(super) season_number_field: RefCell<Box>,

        /// The entry for the season number.
        pub(super) season_number_entry: RefCell<Entry>,

        /// The entry for the location.
        pub(super) location_entry: RefCell<Entry>,

        /// The entry for the meoy.
        pub(super) memo_entry: RefCell<Entry>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for CopyFormWidget {
        const NAME: &'static str = "ArtieCopyFormWidget";
        type Type = super::CopyFormWidget;
        type ParentType = Box;
    }

    impl ObjectImpl for CopyFormWidget {
        fn constructed(&self) {
            self.parent_constructed();

            let obj = self.obj();
            obj.build_ui();
            obj.setup_bindings();
            obj.setup_callbacks();
        }
    }

    impl WidgetImpl for CopyFormWidget {}

    impl BoxImpl for CopyFormWidget {}
}

#[cfg(test)]
mod tests {
    // TODO
}
