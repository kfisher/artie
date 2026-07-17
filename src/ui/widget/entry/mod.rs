// Copyright 2026 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! Widget for inputting text.
//!
//! This is basically a wrapper around GTK's [`gtk::Entry`] widget that adds some application
//! specific convenience items. Use the [`Builder`] provided by [`EntryWidget::builder`] to
//! configure and create [`EntryWidget`] instances.

mod imp;

use glib::{self, GString, Object, SignalHandlerId};

use gtk::prelude::*;
use gtk::subclass::prelude::*;

use crate::ui::validators;

glib::wrapper! {
    pub struct EntryWidget(ObjectSubclass<imp::EntryWidget>)
        @extends gtk::Box,
                 gtk::Widget,
        @implements gtk::Accessible,
                    gtk::Buildable,
                    gtk::ConstraintTarget,
                    gtk::Orientable;
}

impl EntryWidget {
    /// Create a builder instance for creating instances of [`EntryWidget`].
    pub fn builder() -> Builder {
        Builder::new()
    }

    /// Connect to the notification signal emitted when the entry's text changes.
    pub fn connect_text_notify<F>(&self, f: F) -> SignalHandlerId
    where
        F: Fn(&Self) + 'static
    {
        let this = self;
        self.imp().entry
            .borrow()
            .as_ref()
            .unwrap()
            .delegate()
            .unwrap()
            .connect_text_notify(glib::clone!(
                #[weak]
                this,
                move |_| {
                    f(&this)
                }
            ))
    }

    /// Get the entry's text.
    pub fn text(&self) -> GString {
        self.imp().entry
            .borrow()
            .as_ref()
            .unwrap()
            .text()
    }

    /// Set the entry's text.
    pub fn set_text(&self, text: &str) {
        self.imp().entry
            .borrow()
            .as_ref()
            .unwrap()
            .set_text(text);
    }
}

impl Default for EntryWidget {
    fn default() -> Self {
        Builder::new().build()
    }
}

/// Provides a builder-pattern for creating instances of [`EntryWidget`].
pub struct Builder {
    /// The entry's label.
    label: Option<String>,

    /// List of validation functions.
    validators: Vec<validators::string::Validator>,

    /// If true, the entry will be set to expand horizontally.
    hexpand: bool,
}

impl Builder {
    /// Create a new [`Builder`] instance.
    fn new() -> Self {
        Self {
            label: None,
            validators: Vec::default(),
            hexpand: false,
        }
    }

    /// Indicate if the entry should expand the entire available horizontal space (default: false).
    pub fn hexpand(mut self, expand: bool) -> Self {
        self.hexpand = expand;
        self
    }

    /// Set the label for the entry.
    pub fn label(mut self, label: &str) -> Self {
        self.label = Some(label.to_owned());
        self
    }

    /// Configures the entry so that it cannot be considered valid if its empty or only contains
    /// whitespace.
    pub fn not_empty(mut self) -> Self {
        self.validators.push(validators::string::not_empty_or_only_whitespace);
        self
    }

    /// Build a [`EntryWidget`] instance based off the parameters provided to the builder.
    pub fn build(self) -> EntryWidget {
        let obj: EntryWidget = Object::builder()
            .property("label", self.label)
            .property("hexpand", self.hexpand)
            .build();

        let imp = obj.imp();
        imp.set_validators(self.validators);

        obj
    }
}
