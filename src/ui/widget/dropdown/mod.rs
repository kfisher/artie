// Copyright 2026 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! Widget for selecting an item from a list of options.
//!
//! This is a wrapper around the built-in [`gtk::DropDown`] widget that provides application
//! specific customizations.

mod imp;

use glib::{self, Object};

use gtk::StringList;
use gtk::subclass::prelude::*;

use crate::ui::validators::Validator;

glib::wrapper! {
    pub struct DropDownWidget(ObjectSubclass<imp::DropDownWidget>)
        @extends gtk::Box,
                 gtk::Widget,
        @implements gtk::Accessible,
                    gtk::Buildable,
                    gtk::ConstraintTarget,
                    gtk::Orientable;
}

impl DropDownWidget {
    /// Create a builder instance for creating instances of [`DropDownWidget`].
    pub fn builder() -> Builder {
        Builder::new()
    }
}

impl Default for DropDownWidget {
    fn default() -> Self {
        Builder::new().build()
    }
}

/// Provides a builder-pattern for creating instances of [`DropDownWidget`].
pub struct Builder {
    /// The dropdown's label.
    label: Option<String>,

    /// List of options.
    options: Vec<String>,

    /// Validator to use when validating the dropdown's current value.
    validator: Validator,
}

impl Builder {
    /// Create a new [`Builder`] instance.
    fn new() -> Self {
        Self {
            label: None,
            options: Vec::default(),
            validator: Validator::default(),
        }
    }

    /// Set the label for the dropdown.
    ///
    /// # Args
    ///
    /// `label`  The dropdown's label.
    pub fn label(mut self, label: &str) -> Self {
        self.label = Some(label.to_owned());
        self
    }

    /// Add an option to the dropdown.
    ///
    /// # Args
    ///
    /// `option`  Option that will be appended to the list of options.
    pub fn option(mut self, option: &str) -> Self {
        self.options.push(option.to_owned());
        self
    }

    /// Add a list of options to the dropdown.
    ///
    /// # Args
    ///
    /// `options`:  List of options to add. They will be appended to the current list of options if
    /// any were provided to the builder before this call in the same order as they are provided.
    pub fn options(mut self, options: &[&str]) -> Self {
        self.options.extend(options.iter().map(|&s| s.to_owned()));
        self
    }

    /// Build a [`DropDownWidget`] instance based off the parameters provided to the builder.
    pub fn build(self) -> DropDownWidget {
        let obj: DropDownWidget = Object::builder()
            .property("label", self.label)
            .property("model", StringList::from_iter(self.options))
            .build();

        let imp = obj.imp();
        imp.set_validator(self.validator);

        obj
    }
}
