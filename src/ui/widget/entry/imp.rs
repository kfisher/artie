// Copyright 2026 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! Widget implementation.

use std::cell::RefCell;

use glib::{self, Properties};
use gtk::{Align, Box, Entry, Label, Orientation};
use gtk::prelude::*;
use gtk::subclass::prelude::*;

use crate::ui::validators::Validator;

#[derive(Default, Properties)]
#[properties(wrapper_type = super::EntryWidget)]
pub struct EntryWidget {
    /// The label for the entry.
    #[property(get, set)]
    pub(super) label: RefCell<Option<String>>,

    /// Validator to use when validating the entry's current value.
    validator: RefCell<Validator>,
}

impl EntryWidget {
    /// Set the validator used to validate the dropdown. 
    ///
    /// # Args
    ///
    /// `validator`  The new validator.
    pub(super) fn set_validator(&self, validator: Validator) {
        self.validator.replace(validator);
    }

    /// Builds the widget.
    fn build_ui(&self) {
        let obj = self.obj();
        obj.set_orientation(Orientation::Vertical);

        let entry = Entry::builder()
            .build();
        obj.append(&entry);

        let label = Label::builder()
            .halign(Align::Start)
            .build();
        label.add_css_class("field-label");
        label.add_css_class("bottom");
        obj.bind_property("label", &label, "label")
            .sync_create()
            .build();
        obj.append(&label);
    }
}

#[glib::object_subclass]
impl ObjectSubclass for EntryWidget {
    const NAME: &'static str = "ArtieEntryWidget";
    type Type = super::EntryWidget;
    type ParentType = Box;
}

#[glib::derived_properties]
impl ObjectImpl for EntryWidget {
    fn constructed(&self) {
        self.parent_constructed();
        self.build_ui();
    }
}

impl WidgetImpl for EntryWidget {
}

impl BoxImpl for EntryWidget {
}
