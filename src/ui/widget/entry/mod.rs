// Copyright 2026 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! Widget for inputting text.

mod imp;

use glib::{self, Object};

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
