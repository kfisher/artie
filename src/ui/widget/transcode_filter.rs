// Copyright 2026 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! Defines the transcode filter widget.
//!
//! The transcode filter widget is used to filter the list of titles on the transcode page.

use gtk::{
    Align,
    Button,
    Box,
    DropDown,
    Entry,
    Grid,
    Label,
    Orientation,
    Revealer,
    RevealerTransitionType,
    SearchEntry,
    Widget,
};
use gtk::glib::{self, Object};
use gtk::prelude::*;
use gtk::subclass::prelude::*;

use crate::ui::widget::IconToggleButton;

glib::wrapper! {
    pub struct TranscodeFilterWidget(ObjectSubclass<imp::TranscodeFilterWidget>)
        @extends gtk::Box,
                 gtk::Widget,
        @implements gtk::Accessible,
                    gtk::Buildable,
                    gtk::ConstraintTarget,
                    gtk::Orientable;
}

impl TranscodeFilterWidget {
    /// Creates a new copy page instance.
    ///
    /// # Panics
    ///
    /// This will panic if the GObject cannot be created.
    pub fn new() -> Self {
        Object::builder()
            .build()
    }

    /// Builds the widget.
    ///
    /// Called by the implementation ([`imp::TranscodeFilterWidget`]) when constructed.
    fn build_ui(&self) {
        let filter_button = IconToggleButton::builder()
            .icon_name("fontawesome.v7.solid.filter-symbolic")
            .label("Filter")
            .build();
        filter_button.add_css_class("default");
        filter_button.set_active(true);

        let search_entry = SearchEntry::builder()
            .hexpand(true)
            .build();

        let search_row = Box::builder()
            .spacing(2)
            .orientation(Orientation::Horizontal)
            .build();
        search_row.add_css_class("search-row");
        search_row.append(&filter_button);
        search_row.append(&search_entry);


        let filters = Grid::builder()
            .row_spacing(2)
            .column_spacing(4)
            .build();
        filters.add_css_class("filter-row");

        let min_runtime_label = Label::builder()
            .halign(Align::Start)
            .label("Min Runtime:")
            .build();
        let min_runtime = Entry::builder()
            .halign(Align::Start)
            .hexpand(false)
            .max_width_chars(8)
            .build();
        min_runtime.add_css_class("runtime-entry");
        filters.attach(&min_runtime_label, 0, 0, 1, 1);
        filters.attach(&min_runtime, 1, 0, 1, 1);

        let max_runtime_label = Label::builder()
            .halign(Align::Start)
            .label("Max Runtime:")
            .build();
        let max_runtime = Entry::builder()
            .halign(Align::Start)
            .hexpand(false)
            .max_width_chars(8)
            .build();
        max_runtime.add_css_class("runtime-entry");
        filters.attach(&max_runtime_label, 0, 1, 1, 1);
        filters.attach(&max_runtime, 1, 1, 1, 1);

        let title_label = Label::builder()
            .halign(Align::Start)
            .label("Title:")
            .build();
        let title = DropDown::builder()
            .hexpand(true)
            .build();
        filters.attach(&title_label, 0, 2, 1, 1);
        filters.attach(&title, 1, 2, 1, 1);

        let season_label = Label::builder()
            .halign(Align::Start)
            .label("Season:")
            .build();
        let season = Entry::builder()
            .halign(Align::Start)
            .hexpand(false)
            .max_width_chars(8)
            .build();
        season.add_css_class("runtime-entry");
        filters.attach(&season_label, 0, 3, 1, 1);
        filters.attach(&season, 1, 3, 1, 1);

        let transcode_state_label = Label::builder()
            .halign(Align::Start)
            .label("State:")
            .build();
        let transcode_state = DropDown::builder()
            .hexpand(true)
            .build();
        filters.attach(&transcode_state_label, 0, 4, 1, 1);
        filters.attach(&transcode_state, 1, 4, 1, 1);

        let clear_button = Button::builder()
            .hexpand(true)
            .label("Clear")
            .build();
        clear_button.add_css_class("danger");

        let apply_button = Button::builder()
            .hexpand(true)
            .label("Apply")
            .build();
        apply_button.add_css_class("success");

        let filter_controls = Box::builder()
            .orientation(Orientation::Horizontal)
            .spacing(8)
            .build();
        filter_controls.append(&clear_button);
        filter_controls.append(&apply_button);
        filter_controls.add_css_class("filter-controls");
        filters.attach(&filter_controls, 0, 5, 2, 1);

        let revealer = Revealer::builder()
            .child(&filters)
            .reveal_child(true)
            .transition_type(RevealerTransitionType::SlideDown)
            .build();

        filter_button.bind_property("active", &revealer, "reveal-child")
            .sync_create()
            .build();

        self.append(&search_row);
        self.append(&revealer);
        self.set_spacing(2);
        self.set_orientation(Orientation::Vertical);
        self.add_css_class("filter");

        let imp = self.imp();
        imp.revealer.replace(revealer);
    }
}

/// Creates a row for the advance filters.
///
/// # Args
///
/// `label`   The label text to use for the widget.
///
///
/// `widget`  The filter's edit widget.
fn build_filter_row(label: &str, widget: &impl IsA<Widget>) -> Box {
    widget.set_halign(Align::Start);
    widget.set_hexpand(true);

    let label = Label::builder()
        .halign(Align::End)
        .hexpand(true)
        .label(label)
        .build();

    let row = Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(2)
        .build();
    row.append(&label);
    row.append(widget);

    row
}

mod imp {
    //! Implemenation for the copy page widget.

    use std::cell::RefCell;

    use gtk::{Box, Revealer};

    use gtk::glib::{self, Properties};
    use gtk::subclass::prelude::*;

    /// Implemenation for [`super::TranscodeFilterWidget`].
    #[derive(Default, Properties)]
    #[properties(wrapper_type = super::TranscodeFilterWidget)]
    pub struct TranscodeFilterWidget {
        // TODO
        pub(super) revealer: RefCell<Revealer>,
    }

    impl TranscodeFilterWidget {
    }

    #[glib::object_subclass]
    impl ObjectSubclass for TranscodeFilterWidget {
        const NAME: &'static str = "ArtieTranscodeFilterWidget";
        type Type = super::TranscodeFilterWidget;
        type ParentType = Box;
    }

    #[glib::derived_properties]
    impl ObjectImpl for TranscodeFilterWidget {
        fn constructed(&self) {
            self.parent_constructed();

            let obj = self.obj();
            obj.build_ui();
        }
    }

    impl WidgetImpl for TranscodeFilterWidget {}

    impl BoxImpl for TranscodeFilterWidget {}
}

#[cfg(test)]
mod tests {
    // TODO[TESTS]
}
