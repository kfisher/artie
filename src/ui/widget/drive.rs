// Copyright 2025 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! Defines the optical drive widget.
//!
//! The optical drive widget is used to initiate, monitor, and terminate copy operations.

use glib::Object;
use gtk::glib;
use gtk::glib::object::ObjectExt;
use gtk::prelude::*;
use gtk::subclass::prelude::*;

use crate::ui::data::drive::DriveObject;

glib::wrapper! {
    pub struct DriveWidget(ObjectSubclass<imp::DriveWidget>)
        @extends gtk::Box,
                 gtk::Widget,
        @implements gtk::Accessible,
                    gtk::Buildable,
                    gtk::ConstraintTarget,
                    gtk::Orientable;
}

impl DriveWidget {
    pub fn new() -> Self {
        Object::builder().build()
    }                      

    pub fn bind(&self, drive_object: &DriveObject) {
        let mut bindings = self.imp().bindings.borrow_mut();

        let name = self.imp().name.get();
        let name_binding = drive_object
            .bind_property("name", &name, "label")
            .sync_create()
            .build();
        bindings.push(name_binding);

        let path = self.imp().path.get();
        let path_binding = drive_object
            .bind_property("path", &path, "label")
            .transform_to(|_, d: String| {
                Some(format!("[ {} ]", d).to_value())
            })
            .sync_create()
            .build();
        bindings.push(path_binding);

        let serial_number = self.imp().serial_number.get();
        let serial_number_binding = drive_object
            .bind_property("serial_number", &serial_number, "label")
            .transform_to(|_, d: String| {
                Some(format!("[ {} ]", d).to_value())
            })
            .sync_create()
            .build();
        bindings.push(serial_number_binding);

        let disc = self.imp().disc.get();
        let disc_binding = drive_object
            .bind_property("disc", &disc, "label")
            .sync_create()
            .build();
        bindings.push(disc_binding);
    }

    pub fn unbind(&self) {
        for binding in self.imp().bindings.borrow_mut().drain(..) {
            binding.unbind();
        }
    }
}

mod imp {
    use std::cell::RefCell;

    use gtk::{Box, CompositeTemplate, Label};
    use gtk::glib;
    use gtk::glib::{Binding, Properties};
    use gtk::glib::subclass::InitializingObject;
    use gtk::prelude::*;
    use gtk::subclass::prelude::*;

    #[derive(CompositeTemplate, Default)]
    #[template(resource = "/org/example/artie/ui/drive.ui")]
    pub struct DriveWidget {
        #[template_child]
        pub name: TemplateChild<Label>,

        #[template_child]
        pub path: TemplateChild<Label>,

        #[template_child]
        pub serial_number: TemplateChild<Label>,

        #[template_child]
        pub disc: TemplateChild<Label>,

        pub bindings: RefCell<Vec<Binding>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for DriveWidget {
        const NAME: &'static str = "ArtieDriveWidget";
        type Type = super::DriveWidget;
        type ParentType = Box;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for DriveWidget {}

    impl WidgetImpl for DriveWidget {}

    impl BoxImpl for DriveWidget {}
}
