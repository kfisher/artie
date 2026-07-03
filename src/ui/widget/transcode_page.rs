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
use gtk::subclass::prelude::*;

use crate::ui::ContextObject;
use crate::ui::widget::{
    TitleFormWidget,
    TranscodeFormWidget,
    TranscodeListWidget,
    TranscodeQueueWidget,
    VideoPlayerWidget,
};

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
        self.set_vexpand(true);
        self.set_hexpand(true);
        self.set_orientation(Orientation::Horizontal);
        self.set_spacing(8);

        let context = self.context()
            .expect("context was None");

        let transcode_list = TranscodeListWidget::new(&context);
        self.append(&transcode_list);

        let this = self.clone();
        transcode_list.connect_video_selected(glib::clone!(
            #[weak]
            this,
            move |video| {
                this.set_selected_video(Some(video.clone()));
            }
        ));

        let main_section = Box::builder()
            .hexpand(true)
            .orientation(Orientation::Vertical)
            .build();
        self.append(&main_section);

        let main_section_row_0 = Box::builder()
            .hexpand(true)
            .valign(gtk::Align::Start)
            .vexpand(false)
            .orientation(Orientation::Horizontal)
            .spacing(8)
            .margin_top(8)
            .build();
        main_section.append(&main_section_row_0);

        let video_player = VideoPlayerWidget::new();
        main_section_row_0.append(&video_player);

        self.bind_property("selected-video", &video_player, "video")
            .sync_create()
            .build();

        let title_form = TitleFormWidget::new(&context);
        title_form.set_hexpand(true);
        title_form.set_halign(gtk::Align::Fill);
        title_form.set_vexpand(true);
        title_form.set_valign(gtk::Align::Fill);
        main_section_row_0.append(&title_form);

        let transcode_queue = TranscodeQueueWidget::new();
        self.append(&transcode_queue);

        self.bind_property("selected-video", &title_form, "video")
            .sync_create()
            .build();

        let imp = self.imp();
        imp.title_form.replace(Some(title_form));
        imp.video_player.replace(Some(video_player));
    }
}


mod imp {
    use std::cell::RefCell;

    use gtk::{Box, ListView};

    use gtk::glib::{self, Properties};
    use gtk::prelude::*;
    use gtk::subclass::prelude::*;

    use crate::ui::ContextObject;
    use crate::ui::data::VideoObject;
    use crate::ui::widget::{TitleFormWidget, TranscodeFormWidget, VideoPlayerWidget};

    /// Implemenation for [`super::TranscodePageWidget`].
    #[derive(Default, Properties)]
    #[properties(wrapper_type = super::TranscodePageWidget)]
    pub struct TranscodePageWidget {
        /// The application context.
        #[property(get, construct_only)]
        pub(super) context: RefCell<Option<ContextObject>>,

        /// The currently selected video.
        #[property(get, set, nullable)]
        pub(super) selected_video: RefCell<Option<VideoObject>>,

        /// List view for displaying a list of available drives.
        pub(super) drive_list_view: RefCell<Option<ListView>>,

        /// Form use to edit information about the active title.
        pub(super) title_form: RefCell<Option<TitleFormWidget>>,

        /// The widget used to play the video preview.
        pub(super) video_player: RefCell<Option<VideoPlayerWidget>>,
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
