// Copyright 2026 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! Button widget for displaying both an icon and a label.

use glib::Object;
use gtk::{Box, Image, Label, Orientation};
use gtk::glib;
use gtk::prelude::*;

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
            .property("icon-size", -1)
            .property("label", label)
            .property("spacing", 4)
            .build()
    }

    /// Creates a button with an icon only.
    ///
    /// # Args
    ///
    /// `icon_name`:  The name of the icon. This is the name of the SVG file without the path or
    /// file extension.
    ///
    /// # Panics
    ///
    /// This will panic if the GObject cannot be created.
    fn icon_only(icon_name: &str) -> Self {
        let obj: IconToggleButton = Object::builder()
            .property("icon-name", icon_name)
            .property("icon-size", 24)
            .property("spacing", 0)
            .build();

        obj.add_css_class("icon-only");

        obj
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
        self.bind_property("icon-size", &icon, "pixel-size")
            .sync_create()
            .build();
        let label = Label::builder()
            .build();
        self.bind_property("label", &label, "label")
            .sync_create()
            .build();

        let layout = Box::builder()
            .orientation(Orientation::Horizontal)
            .build();
        self.bind_property("spacing", &layout, "spacing").sync_create().build();
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
        if self.icon_name.is_some() && self.label.is_some() {
            IconToggleButton::new(self.icon_name.unwrap(), self.label.unwrap())
        } else if self.icon_name.is_some() {
            IconToggleButton::icon_only(&self.icon_name.unwrap())
        } else {
            IconToggleButton::new(String::default(), String::default())
        }
    }
}

#[cfg(test)]
mod tests {
    // TODO[TESTS]
}

mod imp {
    use std::cell::{Cell, RefCell};

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
        #[property(name="icon-name", get, set)]
        pub(super) icon_name: RefCell<String>,

        /// The button's text.
        #[property(name="label", get, set)]
        pub(super) label: RefCell<String>,

        /// Spacing between the icon and label.
        #[property(name="spacing", get, set)]
        spacing: Cell<i32>,

        /// The size of the icon.
        #[property(name="icon-size", get, set)]
        icon_size: Cell<i32>,
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

