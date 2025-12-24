// Copyright 2025 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! Defines the optical drive widget.
//!
//! The optical drive widget is used to initiate, monitor, and terminate copy operations.

use gtk::glib::property::PropertySet;
use gtk::{Align, Box, Label, Orientation};
use gtk::glib;
use gtk::glib::Object;
use gtk::prelude::*;
use gtk::subclass::prelude::*;

use crate::drive::glib::OpticalDriveObject;

glib::wrapper! {
    /// Widget used to initiate, monitor, and terminate copy operations for an optical drive.
    pub struct DriveWidget(ObjectSubclass<imp::DriveWidget>)
        @extends gtk::Box,
                 gtk::Widget,
        @implements gtk::Accessible,
                    gtk::Buildable, gtk::ConstraintTarget,
                    gtk::Orientable;
}

impl DriveWidget {
    /// Creates a new [`DriveWidget`] instance.
    pub fn new() -> Self {
        Object::builder().build()
    }                      

    /// Builds the user interface.
    ///
    /// It is expected that this will be called as part of the underlying widget's construction.
    /// See [`imp::DriveWidget::constructed`].
    fn build_ui(&self) {
        let imp = self.imp();

        let name_label = imp.name_label
            .borrow()
            .clone();
        name_label.add_css_class("drive-widget-label");

        let header_row = Box::builder()
            .margin_bottom(8)
            .margin_end(8)
            .margin_start(8)
            .margin_top(8)
            .orientation(Orientation::Horizontal)
            .build();
        header_row.append(&name_label);
        header_row.add_css_class("drive-widget-header");

        let placeholder = Label::builder()
            .label("PLACEHOLDER")
            .hexpand(true)
            .build();

        let content_row = Box::builder()
            .orientation(Orientation::Horizontal)
            .width_request(600)
            .build();
        content_row.append(&placeholder);
        content_row.add_css_class("drive-widget-content");

        let disc_label = imp.disc_label
            .borrow()
            .clone();
        disc_label.set_halign(Align::Start);
        disc_label.set_hexpand(true);

        let path_label = imp.path_label
            .borrow()
            .clone();
        path_label.set_hexpand(false);

        let serial_number_label = imp.serial_number_label
            .borrow()
            .clone();
        serial_number_label.set_hexpand(false);

        let footer_row = Box::builder()
            .orientation(Orientation::Horizontal)
            .margin_bottom(4)
            .margin_end(8)
            .margin_start(8)
            .margin_top(4)
            .build();
        footer_row.append(&disc_label);
        footer_row.append(&path_label);
        footer_row.append(&serial_number_label);
        footer_row.add_css_class("drive-widget-footer");

        self.add_css_class("drive-widget");
        self.set_orientation(Orientation::Vertical);
        self.set_spacing(0);
        self.append(&header_row);
        self.append(&content_row);
        self.append(&footer_row);
    }

    /// Binds the widget to the provided optical drive object.
    pub fn bind(&self, drive_object: &OpticalDriveObject) {
        let imp = self.imp();

        let mut bindings = imp.bindings.borrow_mut();

        let name_label = imp.name_label.borrow();
        let name_binding = drive_object
            .bind_property("name", &name_label.clone(), "label")
            .sync_create()
            .build();
        bindings.push(name_binding);

        let path_label = imp.path_label.borrow();
        let path_binding = drive_object
            .bind_property("path", &path_label.clone(), "label")
            .transform_to(|_, d: String| {
                Some(format!("[ {} ]", d).to_value())
            })
            .sync_create()
            .build();
        bindings.push(path_binding);

        let serial_number_label = imp.serial_number_label.borrow();
        let serial_number_binding = drive_object
            .bind_property("serial_number", &serial_number_label.clone(), "label")
            .transform_to(|_, d: String| {
                Some(format!("[ {} ]", d).to_value())
            })
            .sync_create()
            .build();
        bindings.push(serial_number_binding);

        let disc_label = imp.disc_label.borrow();
        let disc_binding = drive_object
            .bind_property("disc_label", &disc_label.clone(), "label")
            .transform_to(|_, d: String| {
                if d.is_empty() {
                    Some(String::from("No Disc"))
                } else {
                    Some(d)
                }
            })
            .sync_create()
            .build();
        bindings.push(disc_binding);
    }

    /// Unbinds the drive widget from the optical drive object which was bound when
    /// [`DriveWidget::bind`] was called.
    pub fn unbind(&self) {
        for binding in self.imp().bindings.borrow_mut().drain(..) {
            binding.unbind();
        }
    }
}

mod imp {
    //! Implemenation for the optical drive widget.

    use std::cell::RefCell;

    use gtk::{Box, Label};
    use gtk::glib;
    use gtk::glib::Binding;
    use gtk::subclass::prelude::*;

    /// Implemenation for [`super::DriveWidget`].
    #[derive(Default)]
    pub struct DriveWidget {
        /// Label widget for displaying the name of the drive.
        pub(super) name_label: RefCell<Label>,

        /// Label widget for displaying the device path of the drive.
        pub(super) path_label: RefCell<Label>,

        /// Label widget for displaying the serial number of the drive.
        pub(super) serial_number_label: RefCell<Label>,

        /// Label widget for displaying the disc label of the disc inserted into the drive.
        pub(super) disc_label: RefCell<Label>,

        /// The widget's bindings.
        ///
        /// This is populated when the widget is bound to a optical drive object
        /// and cleared when unbound. See ([`super::DriveWidget::bind`]) and
        /// ([`super::DriveWidget::unbind`]) for more information.
        pub bindings: RefCell<Vec<Binding>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for DriveWidget {
        const NAME: &'static str = "ArtieDriveWidget";
        type Type = super::DriveWidget;
        type ParentType = Box;
    }

    impl ObjectImpl for DriveWidget {
        fn constructed(&self) {
            self.parent_constructed();
            self.obj().build_ui();
        }
    }

    impl WidgetImpl for DriveWidget {}

    impl BoxImpl for DriveWidget {}
}
