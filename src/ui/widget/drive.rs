// Copyright 2025 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! Defines the optical drive widget.
//!
//! The optical drive widget is used to initiate, monitor, and terminate copy operations.

use glib::Object;
use gtk::glib;

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
}

mod imp {
    use std::cell::RefCell;

    use gtk::{Box, CompositeTemplate, Label};
    use gtk::glib;
    use gtk::glib::Properties;
    use gtk::glib::subclass::InitializingObject;
    use gtk::prelude::*;
    use gtk::subclass::prelude::*;

    #[derive(CompositeTemplate, Default, Properties)]
    #[template(resource = "/org/example/artie/ui/drive.ui")]
    #[properties(wrapper_type = super::DriveWidget)]
    pub struct DriveWidget {
        #[property(get, set)]
        name: RefCell<String>,

        #[property(get, set)]
        device: RefCell<String>,

        #[property(get, set)]
        serial_number: RefCell<String>,

        #[property(get, set)]
        disc: RefCell<String>,

        #[template_child]
        pub name_label: TemplateChild<Label>,

        #[template_child]
        pub device_label: TemplateChild<Label>,

        #[template_child]
        pub serial_number_label: TemplateChild<Label>,

        #[template_child]
        pub disc_label: TemplateChild<Label>,
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

    #[glib::derived_properties]
    impl ObjectImpl for DriveWidget {
        fn constructed(&self) {
            self.parent_constructed();

            let obj = self.obj();
            obj.bind_property("name", &*self.name_label, "label")
                .sync_create()
                .build();
            obj.bind_property("device", &*self.device_label, "label")
                .transform_to(|_, d: String| {
                    Some(format!("[ {} ]", d).to_value())
                })
                .sync_create()
                .build();
            obj.bind_property("serial_number", &*self.serial_number_label, "label")
                .transform_to(|_, d: String| {
                    Some(format!("[ {} ]", d).to_value())
                })
                .sync_create()
                .build();
            obj.bind_property("disc", &*self.disc_label, "label")
                .sync_create()
                .build();
        }
    }

    impl WidgetImpl for DriveWidget {}

    impl BoxImpl for DriveWidget {}
}
