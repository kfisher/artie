// Copyright 2026 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! Defines the transcode page widget.
//!
//! The transcode page is the page used to initiate, monitor, and terminate transcode operations.

use gtk::{
    Box,
    Orientation,
};
use gtk::glib::{self, Object};
use gtk::prelude::*;

use crate::ui::ContextObject;
use crate::ui::widget::{TranscodeListWidget, TranscodeQueueWidget, VideoPlayerWidget};

glib::wrapper! {
    pub struct TranscodePageWidget(ObjectSubclass<imp::TranscodePageWidget>)
        @extends gtk::Box,
                 gtk::Widget,
        @implements gtk::Accessible,
                    gtk::Buildable,
                    gtk::ConstraintTarget,
                    gtk::Orientable;
}

impl TranscodePageWidget {
    /// Creates a new transcode page instance.
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
    /// Called by the implementation ([`imp::TranscodePageWidget`]) when constructed.
    fn build_ui(&self) {
        let context = self.context()
            .expect("context was None");
        let transcode_list = TranscodeListWidget::new(&context);

        let video_preview = VideoPlayerWidget::new();

        let main_section = Box::builder()
            .hexpand(true)
            .orientation(Orientation::Vertical)
            .build();
        main_section.append(&video_preview);

        let transcode_queue = TranscodeQueueWidget::new();

        self.append(&transcode_list);
        self.append(&main_section);
        self.append(&transcode_queue);

        self.set_vexpand(true);
        self.set_hexpand(true);
        self.set_orientation(Orientation::Horizontal);
        self.set_spacing(16);
    }
}

mod imp {
    //! Implemenation for the transcode page widget.

    use std::cell::RefCell;

    use gtk::{Box, ListView};

    use gtk::glib::{self, Properties};
    use gtk::prelude::*;
    use gtk::subclass::prelude::*;

    use crate::ui::ContextObject;

    /// Implemenation for [`super::TranscodePageWidget`].
    #[derive(Default, Properties)]
    #[properties(wrapper_type = super::TranscodePageWidget)]
    pub struct TranscodePageWidget {
        /// List view for displaying a list of available drives.
        pub(super) drive_list_view: RefCell<Option<ListView>>,

        /// The application context.
        #[property(get, construct_only)]
        pub(super) context: RefCell<Option<ContextObject>>,
    }

    impl TranscodePageWidget {
    }

    #[glib::object_subclass]
    impl ObjectSubclass for TranscodePageWidget {
        const NAME: &'static str = "ArtieTranscodePageWidget";
        type Type = super::TranscodePageWidget;
        type ParentType = Box;
    }

    #[glib::derived_properties]
    impl ObjectImpl for TranscodePageWidget {
        fn constructed(&self) {
            self.parent_constructed();

            let obj = self.obj();
            obj.build_ui();
        }
    }

    impl WidgetImpl for TranscodePageWidget {}

    impl BoxImpl for TranscodePageWidget {}
}

#[cfg(test)]
mod tests {
    // TODO[TESTS]
}
