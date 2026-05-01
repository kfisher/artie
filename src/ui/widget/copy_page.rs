// Copyright 2026 Kevin Fisher. All rights reserved.
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
use gtk::glib::{self, Object};
use gtk::prelude::*;
use gtk::subclass::prelude::*;

use crate::ui::ContextObject;
use crate::ui::data::OpticalDriveObject;
use crate::ui::widget::DriveWidget;

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
    /// Creates a new copy page instance.
    ///
    /// # Args
    ///
    /// `context`:  The application context fo the UI.
    ///
    /// # Panics
    ///
    /// This will panic if the GObject cannot be created.
    pub fn new(context: &ContextObject) -> Self {
        Object::builder()
            .property("context", context)
            .build()
    }

    /// Builds the widget.
    ///
    /// Called by the implementation ([`imp::CopyPageWidget`]) when constructed.
    fn build_ui(&self) {
        let list_view = ListView::builder()
            .halign(Align::Center)
            .valign(Align::Start)
            .build();
        list_view.add_css_class("drive-list-widget");

        let scroll = ScrolledWindow::builder()
            .child(&list_view)
            .hscrollbar_policy(PolicyType::Never)
            .hexpand(true)
            .margin_bottom(16)
            .margin_top(16)
            .vexpand(true)
            .build();

        self.append(&scroll);

        self.set_vexpand(true);
        self.set_hexpand(true);
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
            .expect("drive_list_view should not be None")
            .set_model(Some(&NoSelection::new(context.drive_store())));
    }

    /// Configures the factory used in the drive list view.
    ///
    /// Called by the implementation ([`imp::CopyPageWidget`]) when constructed.
    fn setup_factory(&self) {
        let factory = SignalListItemFactory::new();

        factory.connect_setup(move |_, list_item| {
            let drive_widget = DriveWidget::new();
            list_item
                .downcast_ref::<ListItem>()
                .expect("list_item needs to be a ListItem")
                .set_child(Some(&drive_widget));
        });

        factory.connect_bind(move |_, list_item| {
            let drive_object = list_item
                .downcast_ref::<ListItem>()
                .expect("list_item not a ListItem")
                .item()
                .and_downcast::<OpticalDriveObject>()
                .expect("list_item not a OpticalDriveObject");
            let drive_widget = list_item
                .downcast_ref::<ListItem>()
                .expect("list_item not a ListItem")
                .child()
                .and_downcast::<DriveWidget>()
                .expect("list_item child not a DriveWidget");
            drive_widget.bind(&drive_object);
        });

        factory.connect_unbind(move |_, list_item| {
            let drive_widget = list_item
                .downcast_ref::<ListItem>()
                .expect("list_item not a ListItem")
                .child()
                .and_downcast::<DriveWidget>()
                .expect("list_item child not a DriveWidget");
            drive_widget.unbind();
        });

        let imp = self.imp();
        imp.drive_list_view
            .borrow()
            .as_ref()
            .expect("drive_list_view was None")
            .set_factory(Some(&factory));
    }
}

mod imp {
    //! Implemenation for the copy page widget.

    use std::cell::RefCell;

    use gtk::{Box, ListView};

    use gtk::glib::{self, Properties};
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
        }
    }

    impl WidgetImpl for CopyPageWidget {}

    impl BoxImpl for CopyPageWidget {}
}

#[cfg(test)]
mod tests {
    // TODO
}
