// Copyright 2026 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! Defines the transcode queue widget.
//!
//! TODO

use gtk::{
    Orientation,
};
use gtk::glib::{self, Object};
use gtk::prelude::*;
// use gtk::subclass::prelude::*;

glib::wrapper! {
    pub struct TranscodeQueueWidget(ObjectSubclass<imp::TranscodeQueueWidget>)
        @extends gtk::Box,
                 gtk::Widget,
        @implements gtk::Accessible,
                    gtk::Buildable,
                    gtk::ConstraintTarget,
                    gtk::Orientable;
}

impl TranscodeQueueWidget {
    /// Creates a new copy page instance.
    ///
    /// # Panics
    ///
    /// This will panic if the GObject cannot be created.
    pub fn new() -> Self {
        Object::builder()
            .build()
    }

    /// Builds the widget.
    ///
    /// Called by the implementation ([`imp::TranscodeQueueWidget`]) when constructed.
    fn build_ui(&self) {
        self.set_hexpand(false);
        self.set_orientation(Orientation::Vertical);
        self.add_css_class("transcode-queue");
        self.set_width_request(300);
    }
}

mod imp {
    //! Implemenation for the copy page widget.

    use gtk::Box;

    use gtk::glib::{self, Properties};
    use gtk::subclass::prelude::*;

    /// Implemenation for [`super::TranscodeQueueWidget`].
    #[derive(Default, Properties)]
    #[properties(wrapper_type = super::TranscodeQueueWidget)]
    pub struct TranscodeQueueWidget {
    }

    impl TranscodeQueueWidget {
    }

    #[glib::object_subclass]
    impl ObjectSubclass for TranscodeQueueWidget {
        const NAME: &'static str = "ArtieTranscodeQueueWidget";
        type Type = super::TranscodeQueueWidget;
        type ParentType = Box;
    }

    #[glib::derived_properties]
    impl ObjectImpl for TranscodeQueueWidget {
        fn constructed(&self) {
            self.parent_constructed();

            let obj = self.obj();
            obj.build_ui();
        }
    }

    impl WidgetImpl for TranscodeQueueWidget {}

    impl BoxImpl for TranscodeQueueWidget {}
}

#[cfg(test)]
mod tests {
    // TODO[TESTS]
}
