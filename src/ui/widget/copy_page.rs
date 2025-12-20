// Copyright 2025 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! Defines the widget for the copy page.
//!
//! The copy page is used to copy media from connected optical drives. Each drive will have a 
//! [`super::drive::DriveWidget`] instance created.

use glib::Object;
use gtk::glib;

glib::wrapper! {
    pub struct CopyPageWidget(ObjectSubclass<imp::CopyPageWidget>)
        @extends gtk::Box,
                 gtk::Widget,
        @implements gtk::Accessible,
                    gtk::Buildable,
                    gtk::ConstraintTarget,
                    gtk::Orientable;
}

impl CopyPageWidget {
    pub fn new() -> Self {
        Object::builder().build()
    }                      
}

mod imp {
    use std::cell::RefCell;

    use gtk::{Box, CompositeTemplate};
    use gtk::glib;
    use gtk::glib::subclass::InitializingObject;
    use gtk::prelude::*;
    use gtk::subclass::prelude::*;

    use crate::ui::widget::drive::DriveWidget;

    #[derive(CompositeTemplate, Default)]
    #[template(resource = "/org/example/artie/ui/copy_page.ui")]
    pub struct CopyPageWidget {
    }

    #[glib::object_subclass]
    impl ObjectSubclass for CopyPageWidget {
        const NAME: &'static str = "ArtieCopyPageWidget";
        type Type = super::CopyPageWidget;
        type ParentType = Box;

        fn class_init(klass: &mut Self::Class) {
            DriveWidget::ensure_type();

            klass.bind_template();
            // klass.bind_template_callbacks();
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for CopyPageWidget {
        fn constructed(&self) {
            self.parent_constructed();
        }
    }

    impl WidgetImpl for CopyPageWidget {}

    impl BoxImpl for CopyPageWidget {}
}

