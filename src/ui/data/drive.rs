// Copyright 2025 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! Defines the GObject containing drive data.

use glib::Object;
use gtk::glib;

glib::wrapper! {
    pub struct DriveObject(ObjectSubclass<imp::DriveObject>);
}

impl DriveObject {
    pub fn new(name: &str, path: &str, serial_number: &str, disc: &str) -> Self {
        Object::builder()
            .property("name", name.to_owned())
            .property("path", path.to_owned())
            .property("serial-number", serial_number.to_owned())
            .property("disc", disc.to_owned())
            .build()
    }
}

#[derive(Default)]
pub struct DriveData {
    pub name: String,
    pub path: String,
    pub serial_number: String,
    pub disc: String,
}

mod imp {
    use std::cell::RefCell;

    use gtk::{Box, CompositeTemplate, Label};
    use gtk::glib;
    use gtk::glib::Properties;
    use gtk::glib::subclass::InitializingObject;
    use gtk::prelude::*;
    use gtk::subclass::prelude::*;

    #[derive(Properties, Default)]
    #[properties(wrapper_type = super::DriveObject)]
    pub struct DriveObject {
        #[property(name = "name", get, set, type = String, member = name)]
        #[property(name = "path", get, set, type = String, member = path)]
        #[property(name = "serial-number", get, set, type = String, member = serial_number)]
        #[property(name = "disc", get, set, type = String, member = disc)]
        pub data: RefCell<super::DriveData>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for DriveObject {
        const NAME: &'static str = "ArtieDriveObject";
        type Type = super::DriveObject;
    }

    // Trait shared by all GObjects
    #[glib::derived_properties]
    impl ObjectImpl for DriveObject {}
}

