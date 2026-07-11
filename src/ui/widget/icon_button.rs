// Copyright 2026 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! Button widget for displaying both an icon and a label.

use glib::Object;
use gtk::{Align, Box, Image, Label, Orientation};
use gtk::glib;
use gtk::prelude::*;

glib::wrapper! {
    pub struct IconButton(ObjectSubclass<imp::IconButton>)
        @extends gtk::Button,
                 gtk::Widget,
        @implements gtk::Accessible,
                    gtk::Actionable,
                    gtk::Buildable,
                    gtk::ConstraintTarget;

}

impl IconButton {
    /// Create a builder instance for creating instances of [`IconButton`].
    pub fn builder() -> Builder {
        Builder::new()
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
    pub fn new(icon_name: &str, label: &str) -> Self {
        Object::builder()
            .property("icon-name", icon_name)
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
    pub fn icon_only(icon_name: &str) -> Self {
        Object::builder()
            .property("icon-name", icon_name)
            .property("spacing", 0)
            .build()
    }

    /// Builds the widget.
    ///
    /// Called by the implementation ([`imp::IconButton`]) when constructed.
    fn build_ui(&self) {
        let icon = Image::builder()
            .build();
        self.bind_property("icon-name", &icon, "icon-name").sync_create().build();

        let label = Label::builder()
            .build();
        self.bind_property("label", &label, "label").sync_create().build();

        let layout = Box::builder()
            .orientation(Orientation::Horizontal)
            .build();
        self.bind_property("spacing", &layout, "spacing").sync_create().build();
        layout.append(&icon);
        layout.append(&label);

        self.set_child(Some(&layout));
    }
}

impl Default for IconButton {
    fn default() -> Self {
        Self::new("", "")
    }
}

pub struct Builder {
    /// The name of the icon. This is the name of the SVG file without the path or file extention.
    icon_name: Option<String>,

    /// The button's text.
    label: Option<String>,

    /// The button's CSS class.
    css_class: String,

    /// The button should use the ghost variant.
    is_ghost: bool,
}

impl Builder {
    /// Create a new [`Builder`] instance.
    fn new() -> Self {
        Self {
            icon_name: None,
            label: None,
            css_class: String::from("default"),
            is_ghost: false,
        }
    }

    /// Configure the button using the danger color style.
    pub fn danger_button(mut self) -> Self {
        self.css_class = String::from("danger");
        self
    }

    /// Use the ghost form of the button.
    ///
    /// Ghost form buttons don't have a background and instead are icon only.
    pub fn ghost_button(mut self) -> Self {
        self.is_ghost = true;
        self
    }

    /// Configure the button using the primary color style.
    pub fn primary_button(mut self) -> Self {
        self.css_class = String::from("primary");
        self
    }

    /// Configure the button using the secondary color style.
    pub fn secondary_button(mut self) -> Self {
        self.css_class = String::from("secondary");
        self
    }

    /// Set the icon-name for the button.
    ///
    /// # Args
    ///
    /// `icon_name`:  The name of the icon. This is the name of the SVG file without the path or
    /// file extension.
    pub fn icon_name(mut self, label: &str) -> Self {
        self.icon_name = Some(label.to_owned());
        self
    }

    /// Set the label for the dropdown.
    ///
    /// # Args
    ///
    /// `label`:  The button text.
    pub fn label(mut self, label: &str) -> Self {
        self.label = Some(label.to_owned());
        self
    }

    /// Build a [`IconButton`] instance based off the parameters provided to the builder.
    pub fn build(self) -> IconButton {
        let icon_name = self.icon_name
            .unwrap_or_default();
        let button = match self.label {
            Some(label) => IconButton::new(&icon_name, &label),
            None => IconButton::icon_only(&icon_name),
        };
        button.add_css_class(&self.css_class);
        button.set_halign(Align::Start);
        button.set_hexpand(false);
        button.set_vexpand(false);
        button.set_valign(Align::Start);

        if self.is_ghost {
            button.add_css_class("ghost");
        }

        button
    }
}

#[cfg(test)]
mod tests {
    // TODO[TESTS]
}

mod imp {
    use std::cell::RefCell;

    use gtk::Button;
    use gtk::glib::{self, Properties};
    use gtk::prelude::*;
    use gtk::subclass::prelude::*;

    #[derive(Default, Properties)]
    #[properties(wrapper_type = super::IconButton)]
    pub struct IconButton {
        /// The icon name.
        ///
        /// This is the name of the SVG file without the extension.
        #[property(get, set)]
        icon_name: RefCell<String>,

        /// The button's text.
        #[property(get, set)]
        label: RefCell<String>,

        /// Spacing between the icon and label.
        #[property(get, set)]
        spacing: RefCell<i32>,
    }

    impl IconButton {
    }

    #[glib::object_subclass]
    impl ObjectSubclass for IconButton {
        const NAME: &'static str = "ArtieIconButton";
        type Type = super::IconButton;
        type ParentType = Button;
    }

    #[glib::derived_properties]
    impl ObjectImpl for IconButton {
        fn constructed(&self) {
            self.parent_constructed();
            self.obj().build_ui();
        }
    }

    impl WidgetImpl for IconButton {}

    impl ButtonImpl for IconButton {}
}
