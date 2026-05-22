// Copyright 2026 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! Button widget for displaying both an icon and a label.

use glib::Object;
use gtk::{Box, Image, Label, Orientation};
use gtk::glib;
use gtk::prelude::*;
// use gtk::subclass::prelude::*;

glib::wrapper! {
    pub struct IconToggleButton(ObjectSubclass<imp::IconToggleButton>)
        @extends gtk::ToggleButton,
                 gtk::Button,
                 gtk::Widget,
        @implements gtk::Accessible,
                    gtk::Actionable,
                    gtk::Buildable,
                    gtk::ConstraintTarget;

}

impl IconToggleButton {
    /// Creates a builder instance for the button.
    pub fn builder() -> IconToggleButtonBuilder {
        IconToggleButtonBuilder::default()
    }

    /// Creates a new button instance.
    ///
    /// # Args
    ///
    /// `icon_name`:  The name of the icon. This is the name of the SVG file without the path or
    /// file extension.
    ///
    /// `label`:  The button text.
    ///
    /// # Panics
    ///
    /// This will panic if the GObject cannot be created.
    fn new(icon_name: String, label: String) -> Self {
        Object::builder()
            .property("icon-name", icon_name)
            .property("label", label)
            .build()
    }

    /// Builds the widget.
    ///
    /// Called by the implementation ([`imp::IconToggleButton`]) when constructed.
    fn build_ui(&self) {
        let icon = Image::builder()
            .build();

        self.bind_property("icon-name", &icon, "icon-name")
            .sync_create()
            .build();
        let label = Label::builder()
            .build();
        self.bind_property("label", &label, "label")
            .sync_create()
            .build();

        let layout = Box::builder()
            .orientation(Orientation::Horizontal)
            .spacing(4)
            .build();
        layout.append(&icon);
        layout.append(&label);

        self.set_child(Some(&layout));
    }
}

impl Default for IconToggleButton {
    fn default() -> Self {
        Self::new(String::default(), String::default())
    }
}

/// Builder used to create instances of the toggle button.
#[derive(Default)]
pub struct IconToggleButtonBuilder {
    /// The name of the icon. This is the name of the SVG file without the path or file extension.
    icon_name: Option<String>,

    /// The text for the button's label.
    label: Option<String>,
}

impl IconToggleButtonBuilder {
    /// Set the text for the button.
    ///
    /// # Args
    ///
    /// `label`  The text for the button's label.
    pub fn label(mut self, label: &str) -> Self {
        self.label = Some(label.to_owned());
        self
    }

    /// Set the icon for the button.
    ///
    /// # Args
    ///
    /// `name`  The name of the icon. This is the name of the SVG file without the path or file
    /// extension.
    pub fn icon_name(mut self, name: &str) -> Self {
        self.icon_name = Some(name.to_owned());
        self
    }

    /// Creates the widget using the builder's current configuration.
    ///
    /// # Panics
    ///
    /// This will panic if the GObject cannot be created.
    pub fn build(self) -> IconToggleButton {
        IconToggleButton::new(
            self.icon_name.unwrap_or_default(),
            self.label.unwrap_or_default(),
        )
    }
}

#[cfg(test)]
mod tests {
    // TODO[TESTS]
}

mod imp {
    use std::cell::RefCell;

    use gtk::ToggleButton;
    use gtk::glib::{self, Properties};
    use gtk::prelude::*;
    use gtk::subclass::prelude::*;

    #[derive(Default, Properties)]
    #[properties(wrapper_type = super::IconToggleButton)]
    pub struct IconToggleButton {
        /// The icon name.
        ///
        /// This is the name of the SVG file without the extension.
        #[property(get, set)]
        pub(super) icon_name: RefCell<String>,

        /// The button's text.
        #[property(get, set)]
        pub(super) label: RefCell<String>,
    }

    impl IconToggleButton {
    }

    #[glib::object_subclass]
    impl ObjectSubclass for IconToggleButton {
        const NAME: &'static str = "ArtieIconToggleButton";
        type Type = super::IconToggleButton;
        type ParentType = ToggleButton;
    }

    #[glib::derived_properties]
    impl ObjectImpl for IconToggleButton {
        fn constructed(&self) {
            self.parent_constructed();
            self.obj().build_ui();
        }
    }

    impl WidgetImpl for IconToggleButton {}

    impl ButtonImpl for IconToggleButton {}

    impl ToggleButtonImpl for IconToggleButton {}
}

