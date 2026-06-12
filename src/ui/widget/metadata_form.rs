// Copyright 2025 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! Widget for entering information about a title.

// TODO: Should this be renamed to TitleFormWidget?

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
use gtk::subclass::prelude::*;

use crate::models::{MediaType, SpecialFeatureType};
use crate::ui::data::{TitleFormObject, TitleFormType, TitleObject, VideoObject};
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
    /// Constructs a new metadata form instance.
    ///
    /// # Panics
    ///
    /// This will panic if the GObject cannot be created.
    pub fn new() -> Self {
        Object::builder().build()
    }

    /// Update the form data from a title data object.
    ///
    /// # Args
    ///
    /// `title`  The title data object.
    pub fn update_from_title(&self, title: &TitleObject) {
        self.imp().form_data
            .borrow()
            .as_ref()
            .unwrap()
            .update_from_title(title);
    }

    /// Update the form data from a video data object.
    ///
    /// # Args
    ///
    /// `video`  The video data object.
    pub fn update_from_video(&self, video: &VideoObject) {
        let title = video.title()
            .downcast::<TitleObject>()
            .unwrap();
        self.update_from_title(&title);
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
            .row_spacing(3)
            .vexpand(true)
            .hexpand(true)
            .build();

        // To make things easier to update in the future, use a variable to represent the current
        // row and increment between each row. This will allow items to be moved around or inserted
        // without having to update all row values.
        let mut current_row: i32 = 0;

        let media_type_label = Label::builder()
            .halign(Align::End)
            .label("Type")
            .build();
        layout.attach(&media_type_label, 0, current_row, 1, 1);

        let media_type_model = StringList::new(&[
            MediaType::Movie.as_str(),
            MediaType::Show.as_str(),
        ]);
        let media_type_dropdown = DropDown::builder()
            .hexpand(true)
            .model(&media_type_model)
            .build();
        layout.attach(&media_type_dropdown, 1, current_row, 1, 1);

        current_row += 1;

        let title_label = Label::builder()
            .halign(Align::End)
            .label("Title")
            .build();
        layout.attach(&title_label, 0, current_row, 1, 1);

        let title_entry = Entry::builder()
            .build();
        layout.attach(&title_entry, 1, current_row, 1, 1);

        current_row += 1;

        // TODO: We will want help icons (with popup)
        // TODO: Additional information displayed for episode number / count

        let year_label = Label::builder()
            .halign(Align::End)
            .label("Year")
            .build();
        layout.attach(&year_label, 0, current_row, 1, 1);

        let year_entry = Entry::builder()
            .max_length(4)
            .max_width_chars(12)
            .build();
        layout.attach(&year_entry, 1, current_row, 1, 1);

        current_row += 1;

        let season_number_label = Label::builder()
            .halign(Align::End)
            .label("Season Number")
            .build();
        layout.attach(&season_number_label, 0, current_row, 1, 1);

        let season_number_entry = Entry::builder()
            .max_length(2)
            .build();
        layout.attach(&season_number_entry, 1, current_row, 1, 1);

        current_row += 1;

        let episode_number_label = Label::builder()
            .halign(Align::End)
            .label("Episode Number")
            .build();
        layout.attach(&episode_number_label, 0, current_row, 1, 1);

        let episode_number_entry = Entry::builder()
            .max_length(2)
            .build();
        layout.attach(&episode_number_entry, 1, current_row, 1, 1);

        current_row += 1;

        let episode_count_label = Label::builder()
            .halign(Align::End)
            .label("Episode Count")
            .build();
        layout.attach(&episode_count_label, 0, current_row, 1, 1);

        let episode_count_entry = Entry::builder()
            .max_length(2)
            .build();
        layout.attach(&episode_count_entry, 1, current_row, 1, 1);

        current_row += 1;

        let special_feature_label = Label::builder()
            .halign(Align::End)
            .label("Special Feature")
            .build();
        layout.attach(&special_feature_label, 0, current_row, 1, 1);

        let special_feature_model = StringList::new(&[
            "N/A", // SpecialFeatureType::None,
            SpecialFeatureType::BehindTheScenes.as_str(),
            SpecialFeatureType::DeletedScenes.as_str(),
            SpecialFeatureType::Interviews.as_str(),
            SpecialFeatureType::Scenes.as_str(),
            SpecialFeatureType::Samples.as_str(),
            SpecialFeatureType::Shorts.as_str(),
            SpecialFeatureType::Featurettes.as_str(),
            SpecialFeatureType::Clips.as_str(),
            SpecialFeatureType::Extras.as_str(),
            SpecialFeatureType::Trailers.as_str(),
        ]);
        let special_feature_type_dropdown = DropDown::builder()
            .model(&special_feature_model)
            .build();
        layout.attach(&special_feature_type_dropdown, 1, current_row, 1, 1);

        current_row += 1;

        // let special_feature_name_label = Label::builder()
        //     .halign(Align::End)
        //     .label("Special Feature Name")
        //     .build();
        // layout.attach(&special_feature_name_label, 0, current_row, 1, 1);

        let special_feature_name_entry = Entry::builder()
            .build();
        layout.attach(&special_feature_name_entry, 1, current_row, 1, 1);

        current_row += 1;

        let version_label = Label::builder()
            .halign(Align::End)
            .label("Version")
            .build();
        layout.attach(&version_label, 0, current_row, 1, 1);

        let version_entry = Entry::builder()
            .build();
        layout.attach(&version_entry, 1, current_row, 1, 1);

        current_row += 1;

        let disc_number_label = Label::builder()
            .halign(Align::End)
            .label("Disc Number")
            .build();
        layout.attach(&disc_number_label, 0, current_row, 1, 1);

        let disc_number_entry = Entry::builder()
            .max_length(2)
            .max_width_chars(8)
            .build();
        layout.attach(&disc_number_entry, 1, current_row, 1, 1);

        current_row += 1;

        let location_label = Label::builder()
            .halign(Align::End)
            .label("Storage Location")
            .build();
        layout.attach(&location_label, 0, current_row, 1, 1);

        let location_entry = Entry::builder()
            .build();
        layout.attach(&location_entry, 1, current_row, 1, 1);

        current_row += 1;

        let memo_label = Label::builder()
            .halign(Align::End)
            .label("Memo")
            .build();
        layout.attach(&memo_label, 0, current_row, 1, 1);

        let memo_entry = Entry::builder()
            .build();
        layout.attach(&memo_entry, 1, current_row, 1, 1);

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

        let title_form = TitleFormObject::builder(TitleFormType::Full)
            .media_type_dropdown(&media_type_dropdown)
            .title_entry(&title_entry)
            .year_entry(&year_entry)
            .disc_number_entry(&disc_number_entry)
            .season_number_entry(&season_number_entry)
            .location_entry(&location_entry)
            .memo_entry(&memo_entry)
            .episode_number_entry(&episode_number_entry)
            .episode_count_entry(&episode_count_entry)
            .special_feature_type_dropdown(&special_feature_type_dropdown)
            .special_feature_name_entry(&special_feature_name_entry)
            .version_entry(&version_entry)
            .hide_when_movie(season_number_label.upcast_ref())
            .hide_when_movie(season_number_entry.upcast_ref())
            .hide_when_movie(episode_number_label.upcast_ref())
            .hide_when_movie(episode_number_entry.upcast_ref())
            .hide_when_movie(episode_count_label.upcast_ref())
            .hide_when_movie(episode_count_entry.upcast_ref())
            .hide_when_main_feature(special_feature_name_entry.upcast_ref())
            .hide_when_special_feature(episode_number_label.upcast_ref())
            .hide_when_special_feature(episode_number_entry.upcast_ref())
            .hide_when_special_feature(episode_count_label.upcast_ref())
            .hide_when_special_feature(episode_count_entry.upcast_ref())
            .build();

        let imp = self.imp();
        imp.form_data.replace(Some(title_form));
    }
}

impl Default for MetadataFormWidget {
    fn default() -> Self {
        Self::new()
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

mod imp {
    use std::cell::RefCell;

    use gtk::Box;
    use gtk::glib;
    use gtk::subclass::prelude::*;

    use crate::ui::data::TitleFormObject;

    #[derive(Default)]
    pub struct MetadataFormWidget {
        /// Container for the core form elements.
        ///
        /// This contains each input widget used in the form and is responsible for handing
        /// validation.
        pub(super) form_data: RefCell<Option<TitleFormObject>>,
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
