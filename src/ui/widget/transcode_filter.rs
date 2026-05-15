// Copyright 2026 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! Defines the transcode filter widget.
//!
//! The transcode filter widget is used to filter the list of titles on the transcode page.

use gtk::{
    Orientation,
};
use gtk::glib::{self, Object};
use gtk::prelude::*;

glib::wrapper! {
    pub struct TranscodeFilterWidget(ObjectSubclass<imp::TranscodeFilterWidget>)
        @extends gtk::Box,
                 gtk::Widget,
        @implements gtk::Accessible,
                    gtk::Buildable,
                    gtk::ConstraintTarget,
                    gtk::Orientable;
}

impl TranscodeFilterWidget {
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
    /// Called by the implementation ([`imp::TranscodeFilterWidget`]) when constructed.
    fn build_ui(&self) {
        let placeholder = gtk::Label::builder()
            .label("Filter Placeholder")
            .build();
        self.append(&placeholder);
        self.set_orientation(Orientation::Horizontal);
    }
}

mod imp {
    //! Implemenation for the copy page widget.

    use std::cell::RefCell;

    use gtk::Box;

    use gtk::glib::{self, Properties};
    use gtk::prelude::*;
    use gtk::subclass::prelude::*;

    /// Implemenation for [`super::TranscodeFilterWidget`].
    #[derive(Default, Properties)]
    #[properties(wrapper_type = super::TranscodeFilterWidget)]
    pub struct TranscodeFilterWidget {
        #[property(name = "subtask", get, set, type = String)]
        pub(super) subtask: RefCell<String>,
    }

    impl TranscodeFilterWidget {
    }

    #[glib::object_subclass]
    impl ObjectSubclass for TranscodeFilterWidget {
        const NAME: &'static str = "ArtieTranscodeFilterWidget";
        type Type = super::TranscodeFilterWidget;
        type ParentType = Box;
    }

    #[glib::derived_properties]
    impl ObjectImpl for TranscodeFilterWidget {
        fn constructed(&self) {
            self.parent_constructed();

            let obj = self.obj();
            obj.build_ui();
        }
    }

    impl WidgetImpl for TranscodeFilterWidget {}

    impl BoxImpl for TranscodeFilterWidget {}
}

#[cfg(test)]
mod tests {
    // TODO[TESTS]
}
