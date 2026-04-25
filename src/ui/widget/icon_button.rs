// Copyright 2026 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! Button widget for displaying both an icon and a label.

use glib::Object;
use gtk::{Box, Image, Label, Orientation};
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
            .spacing(4)
            .build();
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
