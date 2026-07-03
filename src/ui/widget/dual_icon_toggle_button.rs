// Copyright 2026 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! Button widgets for displaying both an icon and a label.

use glib::Object;
use gtk::Image;
use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;

glib::wrapper! {
    pub struct DuelIconToggleButton(ObjectSubclass<imp::DuelIconToggleButton>)
        @extends gtk::ToggleButton,
                 gtk::Button,
                 gtk::Widget,
        @implements gtk::Accessible,
                    gtk::Actionable,
                    gtk::Buildable,
                    gtk::ConstraintTarget;

}

impl DuelIconToggleButton {
    /// Creates a builder instance for the button.
    pub fn builder() -> DuelIconToggleButtonBuilder {
        DuelIconToggleButtonBuilder::default()
    }

    /// Creates a new button instance.
    ///
    /// # Args
    ///
    /// `active_icon_name`:  The name of the icon when active. This is the name of the SVG file
    /// without the path or file extension.
    ///
    /// `inactive_icon_name`:  The name of the icon when inactive. This is the name of the SVG file
    /// without the path or file extension.
    ///
    /// # Panics
    ///
    /// This will panic if the GObject cannot be created.
    fn new(active_icon_name: &str, inactive_icon_name: &str) -> Self {
        Object::builder()
            .property("active-icon-name", active_icon_name)
            .property("inactive-icon-name", inactive_icon_name)
            .build()
    }

    /// Builds the widget.
    ///
    /// Called by the implementation ([`imp::DuelIconToggleButton`]) when constructed.
    fn build_ui(&self) {
        let icon = Image::builder()
            .pixel_size(20)
            .build();

        self.connect_active_notify(|button| {
            button.update_icon();
        });

        self.set_child(Some(&icon));
    }

    /// Updates the displayed icon based off the active/inactive state.
    fn update_icon(&self) {
        let imp = self.imp();

        let icon_name = if self.is_active() {
            &imp.active_icon_name
        } else {
            &imp.inactive_icon_name
        };

        self.child()
            .and_downcast_ref::<Image>()
            .unwrap()
            .set_icon_name(Some(icon_name.borrow().as_str()));
    }
}

impl Default for DuelIconToggleButton {
    fn default() -> Self {
        Self::new("", "")
    }
}

/// Builder used to create instances of the toggle button.
#[derive(Default)]
pub struct DuelIconToggleButtonBuilder {
    /// The name of the icon when active.
    ///
    /// This is the name of the SVG file without the path or file extension.
    active_icon_name: Option<String>,

    /// The name of the icon when inactive.
    ///
    /// This is the name of the SVG file without the path or file extension.
    inactive_icon_name: Option<String>,

    /// If true, the active and inactive states will have the same appearance.
    no_highlight: bool,

    /// If true, the button will initialize in the active state.
    active: bool,
}

impl DuelIconToggleButtonBuilder {
    /// The created button will initialize in the active state.
    pub fn active(mut self) -> Self {
        self.active = true;
        self
    }

    /// Set the icon for the button when active.
    ///
    /// # Args
    ///
    /// `name`  The name of the icon. This is the name of the SVG file without the path or file
    /// extension.
    pub fn active_icon_name(mut self, name: &str) -> Self {
        self.active_icon_name = Some(name.to_owned());
        self
    }

    /// Set the icon for the button when inactive.
    ///
    /// # Args
    ///
    /// `name`  The name of the icon. This is the name of the SVG file without the path or file
    /// extension.
    pub fn inactive_icon_name(mut self, name: &str) -> Self {
        self.inactive_icon_name = Some(name.to_owned());
        self
    }

    /// Configure the button so that the button has the same general appearance in both the active
    /// and inactive states.
    pub fn no_highlight(mut self) -> Self {
        self.no_highlight = true;
        self
    }

    /// Creates the widget using the builder's current configuration.
    ///
    /// # Panics
    ///
    /// This will panic if the GObject cannot be created.
    pub fn build(self) -> DuelIconToggleButton {
        let icon = DuelIconToggleButton::new(
            &self.active_icon_name.unwrap(),
            &self.inactive_icon_name.unwrap(),
        );

        if self.no_highlight {
            icon.add_css_class("no-highlight");
        }

        icon.update_icon();

        if self.active {
            icon.set_active(true);
        }

        icon
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
    #[properties(wrapper_type = super::DuelIconToggleButton)]
    pub struct DuelIconToggleButton {
        /// The name of the icon when active.
        ///
        /// This is the name of the SVG file without the extension.
        #[property(name="active-icon-name", get, set)]
        pub(super) active_icon_name: RefCell<String>,

        /// The name of the icon when inactive.
        ///
        /// This is the name of the SVG file without the extension.
        #[property(name="inactive-icon-name", get, set)]
        pub(super) inactive_icon_name: RefCell<String>,
    }

    impl DuelIconToggleButton {
    }

    #[glib::object_subclass]
    impl ObjectSubclass for DuelIconToggleButton {
        const NAME: &'static str = "ArtieDuelIconToggleButton";
        type Type = super::DuelIconToggleButton;
        type ParentType = ToggleButton;
    }

    #[glib::derived_properties]
    impl ObjectImpl for DuelIconToggleButton {
        fn constructed(&self) {
            self.parent_constructed();
            self.obj().build_ui();
        }
    }

    impl WidgetImpl for DuelIconToggleButton {}

    impl ButtonImpl for DuelIconToggleButton {}

    impl ToggleButtonImpl for DuelIconToggleButton {}
}


