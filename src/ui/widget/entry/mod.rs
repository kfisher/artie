// Copyright 2026 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! Widget for inputting text.

mod imp;

use glib::{self, GString, Object, SignalHandlerId};

use gtk::prelude::*;
use gtk::subclass::prelude::*;

use crate::ui::validators::Validator;

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
    ///
    /// # Args
    ///
    /// `f`:  The function to call when the entry's text changes.
    pub fn connect_text_notify<F>(&self, f: F) -> SignalHandlerId
    where
        F: Fn(&Self) + 'static
    {
        let this = self;
        self.imp().entry
            .borrow()
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
            .text()
    }

    /// Set the entry's text.
    ///
    /// # Args
    ///
    /// `text`:  The new value for the entry's text.
    pub fn set_text(&self, text: &str) {
        self.imp().entry
            .borrow()
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

    /// Validator to use when validating the entry's current value.
    validator: Validator,
}

impl Builder {
    /// Create a new [`Builder`] instance.
    fn new() -> Self {
        Self {
            label: None,
            validator: Validator::default(),
        }
    }

    /// Set the label for the entry.
    ///
    /// # Args
    ///
    /// `label`  The dropdown's label.
    pub fn label(mut self, label: &str) -> Self {
        self.label = Some(label.to_owned());
        self
    }

    /// Build a [`EntryWidget`] instance based off the parameters provided to the builder.
    pub fn build(self) -> EntryWidget {
        Object::builder()
            .property("label", self.label)
            .build()
    }
}
