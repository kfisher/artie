// Copyright 2025 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! Widget for entering copy parameters.

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

use crate::drive::CopyFormData;
use crate::models::{CopyParamaters, MediaType};
use crate::ui::data::{TitleFormObject, TitleFormType};

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
        Object::builder()
            .build()
    }

    /// Clears the form's values.
    pub fn clear(&self) {
        self.imp().form_data
            .borrow()
            .as_ref()
            .unwrap()
            .clear()
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
        self.imp().form_data
            .borrow()
            .as_ref()
            .unwrap()
            .connect_media_type_changed(f);
    }

    /// Subscribe to changes to the title.
    ///
    /// # Args
    ///
    /// `f`:  Callback function called when the title changes.
    pub fn connect_title_changed<F>(&self, f: F)
    where
        F: Fn(&str) + 'static
    {
        self.imp().form_data
            .borrow()
            .as_ref()
            .unwrap()
            .connect_title_changed(f);
    }

    /// Subscribe to changes to the year.
    ///
    /// # Args
    ///
    /// `f`:  Callback function called when the year changes.
    pub fn connect_year_changed<F>(&self, f: F)
    where
        F: Fn(&str) + 'static
    {
        self.imp().form_data
            .borrow()
            .as_ref()
            .unwrap()
            .connect_year_changed(f);
    }

    /// Subscribe to changes to the disc number.
    ///
    /// # Args
    ///
    /// `f`:  Callback function called when the disc number changes.
    pub fn connect_disc_number_changed<F>(&self, f: F)
    where
        F: Fn(&str) + 'static
    {
        self.imp().form_data
            .borrow()
            .as_ref()
            .unwrap()
            .connect_disc_number_changed(f);
    }

    /// Subscribe to changes to the season number.
    ///
    /// # Args
    ///
    /// `f`:  Callback function called when the season number changes.
    pub fn connect_season_number_changed<F>(&self, f: F)
    where
        F: Fn(&str) + 'static
    {
        self.imp().form_data
            .borrow()
            .as_ref()
            .unwrap()
            .connect_season_number_changed(f);
    }

    /// Subscribe to changes to the storage location.
    ///
    /// # Args
    ///
    /// `f`:  Callback function called when the storage location changes.
    pub fn connect_location_changed<F>(&self, f: F)
    where
        F: Fn(&str) + 'static
    {
        self.imp().form_data
            .borrow()
            .as_ref()
            .unwrap()
            .connect_location_changed(f);
    }

    /// Subscribe to changes to the memo.
    ///
    /// # Args
    ///
    /// `f`:  Callback function called when the memo changes.
    pub fn connect_memo_changed<F>(&self, f: F)
    where
        F: Fn(&str) + 'static
    {
        self.imp().form_data
            .borrow()
            .as_ref()
            .unwrap()
            .connect_memo_changed(f);
    }

    /// Gets the copy parameters based off the current form values.
    pub fn get_copy_parameters(&self) -> CopyParamaters {
        self.imp().form_data
            .borrow()
            .as_ref()
            .unwrap()
            .get_copy_parameters()
    }

    /// Sets the current values of the form to the provided data.
    pub fn set_form_data(&self, data: &CopyFormData) {
        self.imp().form_data
            .borrow()
            .as_ref()
            .unwrap()
            .update_from_copy_form_data(data);
    }

    /// Validates the form returning true if valid or false if invalid.
    ///
    /// This will also update the widget's display based on the validity so that the user knows
    /// which fields are invalid.
    pub fn validate(&self) -> bool {
        self.imp().form_data
            .borrow()
            .as_ref()
            .unwrap()
            .validate()
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

        let title_form = TitleFormObject::builder(TitleFormType::CopyOnly)
            .media_type_dropdown(&type_dropdown)
            .title_entry(&title_entry)
            .year_entry(&year_entry)
            .disc_number_entry(&disc_number_entry)
            .season_number_entry(&season_number_entry)
            .location_entry(&location_entry)
            .memo_entry(&memo_entry)
            .build();

        let imp = self.imp();
        imp.form_data.replace(Some(title_form));
        imp.season_number_field.replace(season_number_field);
    }
}

impl Default for CopyFormWidget {
    fn default() -> Self {
        Self::new()
    }
}

mod imp {
    use std::cell::RefCell;

    use gtk::Box;
    use gtk::glib;
    use gtk::subclass::prelude::*;

    use crate::ui::data::TitleFormObject;

    #[derive(Default)]
    pub struct CopyFormWidget {
        /// The label and entry for the season number.
        pub(super) season_number_field: RefCell<Box>,

        /// Container for the core form elements.
        ///
        /// This contains each input widget used in the form and is responsible for handing
        /// validation.
        pub(super) form_data: RefCell<Option<TitleFormObject>>,
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
        }
    }

    impl WidgetImpl for CopyFormWidget {}

    impl BoxImpl for CopyFormWidget {}
}

#[cfg(test)]
mod tests {
    // TODO[TESTS]
}
