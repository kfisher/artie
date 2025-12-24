// Copyright 2025 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! Defines the copy page widget.
//!
//! The copy page is the page used to initiate, monitor, and terminate copy operations for all 
//! connected optical drives.

use gtk::{
    Align,
    ListItem,
    ListView,
    NoSelection,
    Orientation,
    PolicyType,
    ScrolledWindow,
    SignalListItemFactory
};
use gtk::gio;
use gtk::gio::ListStore;
use gtk::glib;
use gtk::glib::Object;
use gtk::prelude::*;
use gtk::subclass::prelude::*;

use crate::drive::glib::OpticalDriveObject;
use crate::ui::widget::DriveWidget;
use crate::ui::ContextObject;

glib::wrapper! {
    /// Widget used to initiate, monitor, and terminate copy operations.
    pub struct CopyPageWidget(ObjectSubclass<imp::CopyPageWidget>)
        @extends gtk::Box,
                 gtk::Widget,
        @implements gtk::Accessible,
                    gtk::Buildable,
                    gtk::ConstraintTarget,
                    gtk::Orientable;
}

impl CopyPageWidget {
    /// Creates a new [`CopyPageWidget`] instance.
    pub fn new(context: &ContextObject) -> Self {
        Object::builder()
            .property("context", context)
            .build()
    }                      

    /// Builds the user interface.
    ///
    /// It is expected that this will be called as part of the underlying widget's construction.
    /// See [`imp::CopyPageWidget::constructed`].
    fn build_ui(&self) {
        let list_view = ListView::builder()
            .valign(Align::Start)
            .build();

        let scroll = ScrolledWindow::builder()
            .child(&list_view)
            .hscrollbar_policy(PolicyType::Never)
            .hexpand(true)
            .vexpand(true)
            .build();

        self.append(&scroll);

        self.set_halign(Align::Center);
        self.set_margin_bottom(16);
        self.set_margin_top(16);
        self.set_orientation(Orientation::Vertical);
        self.set_spacing(16);

        let imp = self.imp();
        imp.drive_list_view.replace(Some(list_view));
    }

    /// Configures the model used in the drive list view.
    ///
    /// It is expected that this will be called as part of the underlying widget's construction.
    /// See [`imp::CopyPageWidget::constructed`]. 
    fn setup_model(&self) {
        let context = self.context().expect("context not set");
        self.imp().drive_list_view
            .borrow()
            .as_ref()
            .expect("drive_list_view is should not be None")
            .set_model(Some(&NoSelection::new(context.drives_store())));
    }

    /// Configures the factory used in the drive list view.
    ///
    /// It is expected that this will be called as part of the underlying widget's construction.
    /// See [`imp::CopyPageWidget::constructed`]. 
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
                .and_downcast::<OpticalDriveObject>()
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

        let imp = self.imp();
        imp.drive_list_view
            .borrow()
            .as_ref()
            .expect("drive_list_view should not be None")
            .set_factory(Some(&factory));
    }

    /// Configures the signals and callbacks.
    ///
    /// It is expected that this will be called as part of the underlying widget's construction.
    /// See [`imp::CopyPageWidget::constructed`]. 
    fn setup_callbacks(&self) {
    }
}

mod imp {
    //! Implemenation for the copy page widget.

    use std::cell::RefCell;

    use gtk::{Box, ListView};
    use gtk::gio::ListStore;
    use gtk::glib;
    use gtk::glib::Properties;
    use gtk::prelude::*;
    use gtk::subclass::prelude::*;

    use crate::ui::ContextObject;

    /// Implemenation for [`super::CopyPageWidget`].
    #[derive(Default, Properties)]
    #[properties(wrapper_type = super::CopyPageWidget)]
    pub struct CopyPageWidget {
        /// List view for displaying a list of available drives.
        pub(super) drive_list_view: RefCell<Option<ListView>>,

        /// The application context.
        #[property(get, set = Self::set_context, construct_only)]
        pub(super) context: RefCell<Option<ContextObject>>,
    }

    impl CopyPageWidget {
        /// Sets the application context.
        fn set_context(&self, context: Option<ContextObject>) {
            self.context.replace(context);
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for CopyPageWidget {
        const NAME: &'static str = "ArtieCopyPageWidget";
        type Type = super::CopyPageWidget;
        type ParentType = Box;
    }

    #[glib::derived_properties]
    impl ObjectImpl for CopyPageWidget {
        fn constructed(&self) {
            self.parent_constructed();

            let obj = self.obj();
            obj.build_ui();
            obj.setup_model();
            obj.setup_factory();
            obj.setup_callbacks();
        }
    }

    impl WidgetImpl for CopyPageWidget {}

    impl BoxImpl for CopyPageWidget {}
}
