// Copyright 2025 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! Widget for entering transcode parameters.

use gtk::{
    Label,
    Orientation,
};
use gtk::glib::{self, Object};
use gtk::prelude::*;
// use gtk::subclass::prelude::*;

glib::wrapper! {
    pub struct TranscodeFormWidget(ObjectSubclass<imp::TranscodeFormWidget>)
        @extends gtk::Box,
                 gtk::Widget,
        @implements gtk::Accessible,
                    gtk::Buildable,
                    gtk::ConstraintTarget,
                    gtk::Orientable;
}

impl TranscodeFormWidget {
    /// Constructs a new transcode form instance.
    ///
    /// # Panics
    ///
    /// This will panic if the GObject cannot be created.
    pub fn new() -> Self {
        Object::builder().build()
    }

    /// Builds the widget.
    ///
    /// Called by the implementation ([`imp::TranscodeFormWidget`]) when constructed.
    fn build_ui(&self) {
        let audio_header = Label::builder()
            .label("Audio")
            .build();
        audio_header.add_css_class("header");

        let subtitle_header = Label::builder()
            .label("Subtitles")
            .build();
        subtitle_header.add_css_class("header");

        self.append(&audio_header);
        self.append(&subtitle_header);

        self.add_css_class("transcode-form-widget");
        self.set_hexpand(true);
        self.set_vexpand(true);
        self.set_orientation(Orientation::Vertical);
    }
}

impl Default for TranscodeFormWidget {
    fn default() -> Self {
        Self::new()
    }
}

mod imp {
    use gtk::Box;
    use gtk::glib;
    use gtk::subclass::prelude::*;

    #[derive(Default)]
    pub struct TranscodeFormWidget {
    }

    #[glib::object_subclass]
    impl ObjectSubclass for TranscodeFormWidget {
        const NAME: &'static str = "ArtieTranscodeFormWidget";
        type Type = super::TranscodeFormWidget;
        type ParentType = Box;
    }

    impl ObjectImpl for TranscodeFormWidget {
        fn constructed(&self) {
            self.parent_constructed();
            let obj = self.obj();
            obj.build_ui();
        }
    }

    impl WidgetImpl for TranscodeFormWidget {}

    impl BoxImpl for TranscodeFormWidget {}
}

#[cfg(test)]
mod tests {
    // TODO[TESTS]
}
