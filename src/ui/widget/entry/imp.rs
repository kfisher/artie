// Copyright 2026 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! Widget implementation.

use std::cell::{Cell, RefCell};

use glib::{self, Properties};
use gtk::{Align, Box, Entry, Label, Orientation};
use gtk::prelude::*;
use gtk::subclass::prelude::*;

use crate::ui::validators;

#[derive(Default, Properties)]
#[properties(wrapper_type = super::EntryWidget)]
pub struct EntryWidget {
    /// The label for the entry.
    #[property(get, set)]
    pub(super) label: RefCell<Option<String>>,

    /// Indicates if the entry has a valid value.
    #[property(get)]
    is_valid: Cell<bool>,

    /// The underlying [`Entry`] GTK widget.
    pub(super) entry: RefCell<Option<Entry>>,

    /// List of validation functions.
    validators: RefCell<Vec<validators::string::Validator>>,
}

impl EntryWidget {
    /// Set the validation functions.
    ///
    /// This will replace any existing validation functions. This is expected to only be called by
    /// the widget's builder.
    pub(super) fn set_validators(&self, validators: Vec<validators::string::Validator>) {
        self.validators.replace(validators);
    }

    /// Builds the widget.
    fn build_ui(&self) {
        let obj = self.obj();
        obj.set_orientation(Orientation::Vertical);

        let entry = Entry::builder()
            .build();
        obj.append(&entry);

        let this = self;
        entry.delegate()
            .unwrap()
            .connect_text_notify(glib::clone!(
                #[weak]
                this,
                move |_| {
                    this.validate()
                }
            ));

        let label = Label::builder()
            .halign(Align::Start)
            .build();
        label.add_css_class("field-label");
        label.add_css_class("bottom");
        obj.bind_property("label", &label, "label")
            .sync_create()
            .build();
        obj.append(&label);

        self.entry.replace(Some(entry));
    }

    /// Sets the validation property.
    fn set_is_valid(&self, valid: bool) {
        if self.is_valid.get() != valid {
            self.is_valid.set(valid);
            self.obj().notify_is_valid();
        }
    }

    /// Validates the entry based on the validators it was configured with.
    fn validate(&self) {
        let text = self.entry
            .borrow()
            .as_ref()
            .map(|entry| entry.text());

        let Some(text) = text else {
            self.set_is_valid(false);
            return;
        };

        for validate in self.validators.borrow().iter() {
            if !validate(&text) {
                self.set_is_valid(false);
                return;
            }
        }

        self.set_is_valid(true);
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
