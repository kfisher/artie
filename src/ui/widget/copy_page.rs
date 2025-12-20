// Copyright 2025 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! Defines the widget for the copy page.
//!
//! The copy page is used to copy media from connected optical drives. Each drive will have a 
//! [`super::drive::DriveWidget`] instance created.

use gio::ListStore;
use glib::Object;
use gtk::{ListItem, NoSelection};
use gtk::gio;
use gtk::glib;
use gtk::glib::object::Cast;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::SignalListItemFactory;

use crate::ui::data::drive::DriveObject;
use crate::ui::widget::drive::DriveWidget;

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

    fn drives(&self) -> ListStore {
        self.imp()
            .drive_list
            .borrow()
            .clone()
            .expect("Failed to get drives.")
    }

    fn setup_callbacks(&self) {
    }

    fn setup_drives(&self) {
        let model = ListStore::new::<DriveObject>();

        self.imp().drive_list.replace(Some(model));

        let selection_model = NoSelection::new(Some(self.drives()));
        self.imp().drive_list_view.set_model(Some(&selection_model));

        let d0 = DriveObject::new("Drive X", "/dev/ph01", "PH001", "FAUX_MOVIE");
        self.drives().append(&d0);

        let d1 = DriveObject::new("Drive Y", "/dev/ph02", "PH002", "FAUX_MOVIE");
        self.drives().append(&d1);

        let d2 = DriveObject::new("Drive Z", "/dev/ph03", "PH003", "No Disc");
        self.drives().append(&d2);
    }

    fn setup_factory(&self) {
        let factory = SignalListItemFactory::new();

        factory.connect_setup(move |_, list_item| {
            let drive_widget = DriveWidget::new();
            list_item
                .downcast_ref::<ListItem>()
                .expect("list_item needs to be a ListItem")
                .set_child(Some(&drive_widget))
        });

        factory.connect_bind(move |_, list_item| {
            let drive_object = list_item
                .downcast_ref::<ListItem>()
                .expect("list_item needs to be a ListItem")
                .item()
                .and_downcast::<DriveObject>()
                .expect("list_item needs to be a DriveObject");
            let drive_widget = list_item
                .downcast_ref::<ListItem>()
                .expect("list_item needs to be a ListItem")
                .child()
                .and_downcast::<DriveWidget>()
                .expect("list_item child needs to be a DriveWidget");
            drive_widget.bind(&drive_object);
        });

        factory.connect_unbind(move |_, list_item| {
            let drive_widget = list_item
                .downcast_ref::<ListItem>()
                .expect("list_item needs to be a ListItem")
                .child()
                .and_downcast::<DriveWidget>()
                .expect("list_item child needs to be a DriveWidget");
            drive_widget.unbind();
        });

        self.imp().drive_list_view.set_factory(Some(&factory));
    }
}

mod imp {
    use std::cell::RefCell;

    use gtk::{Box, CompositeTemplate, ListView};
    use gtk::gio;
    use gtk::glib;
    use gtk::glib::subclass::InitializingObject;
    use gtk::prelude::*;
    use gtk::subclass::prelude::*;

    use crate::ui::widget::drive::DriveWidget;

    #[derive(CompositeTemplate, Default)]
    #[template(resource = "/org/example/artie/ui/copy_page.ui")]
    pub struct CopyPageWidget {
        #[template_child]
        pub drive_list_view: TemplateChild<ListView>,

        pub drive_list: RefCell<Option<gio::ListStore>>,
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

            let obj = self.obj();
            obj.setup_drives();
            obj.setup_callbacks();
            obj.setup_factory();
        }
    }

    impl WidgetImpl for CopyPageWidget {}

    impl BoxImpl for CopyPageWidget {}
}

