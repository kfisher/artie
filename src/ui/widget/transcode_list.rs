// Copyright 2026 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! Defines the transcode list widget.
//!
//! The transcode list widget is responsible for displaying the list of titles that are available
//! to be transcoded.

use gtk::{
    ListView,
    Orientation,
    PolicyType,
    ScrolledWindow,
};
use gtk::glib::{self, Object};
use gtk::prelude::*;

use crate::ui::widget::TranscodeFilterWidget;

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
    pub fn new() -> Self {
        Object::builder()
            .build()
    }

    /// Builds the widget.
    ///
    /// Called by the implementation ([`imp::TranscodeListWidget`]) when constructed.
    fn build_ui(&self) {
        let filter = TranscodeFilterWidget::new();

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

        self.set_orientation(Orientation::Vertical);
    }
}

mod imp {
    //! Implemenation for the copy page widget.

    use std::cell::RefCell;

    use gtk::Box;

    use gtk::glib::{self, Properties};
    use gtk::prelude::*;
    use gtk::subclass::prelude::*;

    /// Implemenation for [`super::TranscodeListWidget`].
    #[derive(Default, Properties)]
    #[properties(wrapper_type = super::TranscodeListWidget)]
    pub struct TranscodeListWidget {
        #[property(name = "subtask", get, set, type = String)]
        pub(super) subtask: RefCell<String>,
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

