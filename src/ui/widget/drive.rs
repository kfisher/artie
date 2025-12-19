// Copyright 2025 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! TODO

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

    use gtk::{Box, CompositeTemplate};
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
        drive_name: RefCell<String>,
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
            // self.drive_name.set_text("HELLO FRIEND");
        }
    }

    impl WidgetImpl for DriveWidget {}

    impl BoxImpl for DriveWidget {}
}
