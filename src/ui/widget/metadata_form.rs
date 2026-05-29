// Copyright 2025 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! Widget for entering information about a title.

use gtk::{
    Align,
    Box,
    DropDown,
    Entry,
    Label,
    Orientation,
    StringList,
    Grid,
};
use gtk::glib::{self, Object};
use gtk::prelude::*;
// use gtk::subclass::prelude::*;

use crate::models::MediaType;
use crate::ui::widget::IconButton;

glib::wrapper! {
    pub struct MetadataFormWidget(ObjectSubclass<imp::MetadataFormWidget>)
        @extends gtk::Box,
                 gtk::Widget,
        @implements gtk::Accessible,
                    gtk::Buildable,
                    gtk::ConstraintTarget,
                    gtk::Orientable;
}

impl MetadataFormWidget {
    /// Constructs a new copy form instance.
    ///
    /// # Panics
    ///
    /// This will panic if the GObject cannot be created.
    pub fn new() -> Self {
        Object::builder().build()
    }

    /// Builds the widget.
    ///
    /// Called by the implementation ([`imp::MetadataFormWidget`]) when constructed.
    fn build_ui(&self) {
        let header = Label::builder()
            .label("Title Info")
            .build();
        header.add_css_class("header");

        let layout = Grid::builder()
            .column_spacing(8)
            .row_spacing(2)
            .vexpand(true)
            .build();

        let media_type_label = Label::builder()
            .halign(Align::End)
            .label("Type")
            .build();
        layout.attach(&media_type_label, 0, 0, 1, 1);

        let media_type_model = StringList::new(&[
            MediaType::Movie.as_str(),
            MediaType::Show.as_str(),
        ]);
        let media_type_dropdown = DropDown::builder()
            .model(&media_type_model)
            .width_request(116)
            .build();
        layout.attach(&media_type_dropdown, 1, 0, 1, 1);

        let title_label = Label::builder()
            .halign(Align::End)
            .label("Title")
            .build();
        layout.attach(&title_label, 0, 1, 1, 1);

        let title_entry = Entry::builder()
            .build();
        layout.attach(&title_entry, 1, 1, 1, 1);

        // TODO: We will want help icons (with popup)
        // TODO: Additional information displayed for episode number / count

        let year_label = Label::builder()
            .halign(Align::End)
            .label("Year")
            .build();
        layout.attach(&year_label, 0, 2, 1, 1);

        let year_entry = Entry::builder()
            .max_length(4)
            .max_width_chars(12)
            .build();
        layout.attach(&year_entry, 1, 2, 1, 1);

        let season_number_label = Label::builder()
            .halign(Align::End)
            .label("Season Number")
            .build();
        layout.attach(&season_number_label, 0, 3, 1, 1);

        let season_number_entry = Entry::builder()
            .max_length(2)
            .build();
        layout.attach(&season_number_entry, 1, 3, 1, 1);

        let episode_number_label = Label::builder()
            .halign(Align::End)
            .label("Episode Number")
            .build();
        layout.attach(&episode_number_label, 0, 4, 1, 1);

        let episode_number_entry = Entry::builder()
            .max_length(2)
            .build();
        layout.attach(&episode_number_entry, 1, 4, 1, 1);

        let episode_count_label = Label::builder()
            .halign(Align::End)
            .label("Episode Count")
            .build();
        layout.attach(&episode_count_label, 0, 5, 1, 1);

        let episode_count_entry = Entry::builder()
            .max_length(2)
            .build();
        layout.attach(&episode_count_entry, 1, 5, 1, 1);

        let special_feature_label = Label::builder()
            .halign(Align::End)
            .label("Special Feature")
            .build();
        layout.attach(&special_feature_label, 0, 6, 1, 1);

        let special_feature_model = StringList::new(&[
            "-",
        ]);
        let special_feature_dropdown = DropDown::builder()
            .model(&special_feature_model)
            .build();
        layout.attach(&special_feature_dropdown, 1, 6, 1, 1);

        let version_label = Label::builder()
            .halign(Align::End)
            .label("Version")
            .build();
        layout.attach(&version_label, 0, 7, 1, 1);

        let version_entry = Entry::builder()
            .build();
        layout.attach(&version_entry, 1, 7, 1, 1);

        let disc_number_label = Label::builder()
            .halign(Align::End)
            .label("Disc Number")
            .build();
        layout.attach(&disc_number_label, 0, 8, 1, 1);

        let disc_number_entry = Entry::builder()
            .max_length(2)
            .max_width_chars(8)
            .build();
        layout.attach(&disc_number_entry, 1, 8, 1, 1);

        let location_label = Label::builder()
            .halign(Align::End)
            .label("Location")
            .build();
        layout.attach(&location_label, 0, 9, 1, 1);

        let location_entry = Entry::builder()
            .build();
        layout.attach(&location_entry, 1, 9, 1, 1);

        let memo_label = Label::builder()
            .halign(Align::End)
            .label("Memo")
            .build();
        layout.attach(&memo_label, 0, 10, 1, 1);

        let memo_entry = Entry::builder()
            .build();
        layout.attach(&memo_entry, 1, 10, 1, 1);

        let revert_button = IconButton::new(
            "fontawesome.v7.solid.rotate-left",
            "Revert",
        );
        revert_button.add_css_class("default");
        revert_button.set_halign(Align::End);
        revert_button.set_hexpand(true);

        let apply_button = IconButton::new(
            "fontawesome.v7.solid.save",
            "Apply",
        );
        apply_button.add_css_class("secondary");

        let controls = Box::builder()
            .orientation(Orientation::Horizontal)
            .spacing(8)
            .build();
        controls.append(&revert_button);
        controls.append(&apply_button);
        controls.add_css_class("controls");
        controls.set_hexpand(true);

        self.append(&header);
        self.append(&layout);
        self.append(&controls);
        self.add_css_class("metadata-form-widget");
        self.set_halign(Align::Start);
        self.set_hexpand(false);
        self.set_valign(Align::Start);
        self.set_vexpand(false);
        self.set_orientation(Orientation::Vertical);
    }
}

impl Default for MetadataFormWidget {
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
    pub struct MetadataFormWidget {
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
    impl ObjectSubclass for MetadataFormWidget {
        const NAME: &'static str = "ArtieMetadataFormWidget";
        type Type = super::MetadataFormWidget;
        type ParentType = Box;
    }

    impl ObjectImpl for MetadataFormWidget {
        fn constructed(&self) {
            self.parent_constructed();

            let obj = self.obj();
            obj.build_ui();
        }
    }

    impl WidgetImpl for MetadataFormWidget {}

    impl BoxImpl for MetadataFormWidget {}
}

#[cfg(test)]
mod tests {
    // TODO[TESTS]
}
