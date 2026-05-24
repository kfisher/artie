// Copyright 2026 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! Defines the transcode list widget.
//!
//! The transcode list widget is responsible for displaying the list of videos that are available
//! to be transcoded.

use gtk::{
    ListItem,
    ListView,
    NoSelection,
    Orientation,
    PolicyType,
    ScrolledWindow,
    SignalListItemFactory,
};
use gtk::gio::ListStore;
use gtk::glib::{self, Object};
use gtk::prelude::*;
use gtk::subclass::prelude::*;

use crate::bus::Handle;
use crate::db;
use crate::ui::context::ContextObject;
use crate::ui::widget::{TranscodeListFilterWidget, TranscodeListItemWidget};
use crate::ui::data::VideoObject;

glib::wrapper! {
    pub struct TranscodeListWidget(ObjectSubclass<imp::TranscodeListWidget>)
        @extends gtk::Box,
                 gtk::Widget,
        @implements gtk::Accessible,
                    gtk::Buildable,
                    gtk::ConstraintTarget,
                    gtk::Orientable;
}

impl TranscodeListWidget {
    /// Creates a new copy page instance.
    ///
    /// # Panics
    ///
    /// This will panic if the GObject cannot be created.
    pub fn new(context: &ContextObject) -> Self {
        let obj: Self = Object::builder()
            .build();
        obj.imp().context.replace(Some(context.clone()));
        obj.setup_model();
        obj.setup_factory();
        obj.refresh();

        obj
    }

    /// Builds the widget.
    ///
    /// Called by the implementation ([`imp::TranscodeListWidget`]) when constructed.
    fn build_ui(&self) {
        let filter = TranscodeListFilterWidget::new();

        let list_view = ListView::builder()
            .build();

        let scroll = ScrolledWindow::builder()
            .child(&list_view)
            .hscrollbar_policy(PolicyType::Never)
            .vexpand(true)
            .vscrollbar_policy(PolicyType::Automatic)
            .build();

        self.append(&filter);
        self.append(&scroll);

        self.set_hexpand(false);
        self.set_orientation(Orientation::Vertical);
        self.add_css_class("transcode-list");

        let imp = self.imp();
        imp.video_list_view.replace(Some(list_view));
    }

    /// Refresh the list of videos.
    fn refresh(&self) {
        let imp = self.imp();

        let bus = imp.context
            .borrow()
            .as_ref()
            .expect("context was None")
            .bus()
            .expect("bus was None");

        let mut video_store = imp.video_store
            .borrow()
            .clone()
            .expect("video_store was None");

        glib::spawn_future_local(async move {
            refresh_video_store(&bus, &mut video_store).await;
        });
    }

    /// Configures the model used in the video list view.
    fn setup_model(&self) {
        let video_store = ListStore::new::<VideoObject>();
        let model = NoSelection::new(Some(video_store.clone()));

        let imp = self.imp();

        imp.video_list_view
            .borrow()
            .as_ref()
            .expect("video_list_view was None")
            .set_model(Some(&model));

        imp.video_store.replace(Some(video_store));
    }

    /// Configures the factory used in the video list view.
    fn setup_factory(&self) {
        let factory = SignalListItemFactory::new();

        factory.connect_setup(move |_, list_item| {
            let widget = TranscodeListItemWidget::new();
            list_item
                .downcast_ref::<ListItem>()
                .expect("list_item needs to be a ListItem")
                .set_child(Some(&widget));
        });

        factory.connect_bind(move |_, list_item| {
            let video_object = list_item
                .downcast_ref::<ListItem>()
                .expect("list_item not a ListItem")
                .item()
                .and_downcast::<VideoObject>()
                .expect("list_item not a VideoObject");
            let widget = list_item
                .downcast_ref::<ListItem>()
                .expect("list_item not a ListItem")
                .child()
                .and_downcast::<TranscodeListItemWidget>()
                .expect("list_item child not a TranscodeListItemWidget");
            widget.bind(&video_object);
        });

        factory.connect_unbind(move |_, list_item| {
            let widget = list_item
                .downcast_ref::<ListItem>()
                .expect("list_item not a ListItem")
                .child()
                .and_downcast::<TranscodeListItemWidget>()
                .expect("list_item child not a TranscodeListItemWidget");
            widget.unbind();
        });

        let imp = self.imp();
        imp.video_list_view
            .borrow()
            .as_ref()
            .expect("video_list_view was None")
            .set_factory(Some(&factory));
    }
}

/// Refresh the list of videos.
///
/// # Args
///
/// `bus`:  Handle for messages to the various application actors.
///
/// `store`  The video list store that will be updated.
async fn refresh_video_store(bus: &Handle, store: &mut ListStore) {
    let conn = match db::connect(&bus).await {
        Ok(conn) => conn,
        Err(error) => {
            tracing::error!(?error, "database connection failed");
            return;
        }
    };

    let videos = db::video::inbox_videos(&conn)
        .inspect_err(|error| tracing::error!(?error, "failed to get video data"))
        .unwrap_or_default()
        .into_iter()
        .map(|video| VideoObject::new(&video));

    store.remove_all();
    store.extend(videos);
}

mod imp {
    //! Implemenation for the copy page widget.

    use std::cell::RefCell;

    use gtk::{Box, ListView};

    use gtk::gio::ListStore;
    use gtk::glib::{self, Properties};
    // use gtk::prelude::*;
    use gtk::subclass::prelude::*;

    use crate::ui::ContextObject;

    /// Implemenation for [`super::TranscodeListWidget`].
    #[derive(Default, Properties)]
    #[properties(wrapper_type = super::TranscodeListWidget)]
    pub struct TranscodeListWidget {
        /// The application context.
        pub(super) context: RefCell<Option<ContextObject>>,

        /// List view for displaying a list of videos.
        pub(super) video_list_view: RefCell<Option<ListView>>,

        /// List of [`crate::ui::data::VideoObject`] instances containing the information for
        /// videos that can be transcoded.
        pub(super) video_store: RefCell<Option<ListStore>>,
    }

    impl TranscodeListWidget {
    }

    #[glib::object_subclass]
    impl ObjectSubclass for TranscodeListWidget {
        const NAME: &'static str = "ArtieTranscodeListWidget";
        type Type = super::TranscodeListWidget;
        type ParentType = Box;
    }

    #[glib::derived_properties]
    impl ObjectImpl for TranscodeListWidget {
        fn constructed(&self) {
            self.parent_constructed();

            let obj = self.obj();
            obj.build_ui();
        }
    }

    impl WidgetImpl for TranscodeListWidget {}

    impl BoxImpl for TranscodeListWidget {}
}

#[cfg(test)]
mod tests {
    // TODO[TESTS]
}

