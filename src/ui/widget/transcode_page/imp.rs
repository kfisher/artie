// Copyright 2026 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! Widget implementation.

use std::cell::RefCell;

use glib::{self, Properties};

use gtk::{Box, ListView, Orientation};

use gtk::prelude::*;
use gtk::subclass::prelude::*;

use crate::ui::ContextObject;
use crate::ui::data::VideoObject;
use crate::ui::widget::{
    TitleFormWidget,
    TranscodeFormWidget,
    TranscodeListWidget,
    TranscodeQueueWidget,
    VideoPlayerWidget,
};

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
    /// Builds the widget.
    fn build_ui(&self) {
        let obj = self.obj();
        obj.set_vexpand(true);
        obj.set_hexpand(true);
        obj.set_orientation(Orientation::Horizontal);
        obj.set_spacing(8);

        obj.append(&self.create_video_list());
        obj.append(&self.create_main_section());
        obj.append(&self.create_transcode_queue());
    }

    /// Creates the main section (center column) of the transcode page.
    fn create_main_section(&self) -> Box {
        let main_section = Box::builder()
            .hexpand(true)
            .orientation(Orientation::Vertical)
            .build();

        let main_section_row_0 = Box::builder()
            .hexpand(true)
            .margin_top(8)
            .orientation(Orientation::Horizontal)
            .spacing(8)
            .valign(gtk::Align::Start)
            .vexpand(false)
            .build();
        main_section.append(&main_section_row_0);

        let video_player = VideoPlayerWidget::new();
        main_section_row_0.append(&video_player);

        self.obj().bind_property("selected-video", &video_player, "video")
            .sync_create()
            .build();

        let context = self.context
            .borrow()
            .clone()
            .unwrap();

        let title_form = TitleFormWidget::new(&context);
        title_form.set_hexpand(true);
        title_form.set_halign(gtk::Align::Fill);
        title_form.set_vexpand(true);
        title_form.set_valign(gtk::Align::Fill);
        main_section_row_0.append(&title_form);

        self.obj().bind_property("selected-video", &title_form, "video")
            .sync_create()
            .build();

        let main_section_row_1 = Box::builder()
            .hexpand(true)
            .margin_bottom(8)
            .margin_top(8)
            .orientation(Orientation::Horizontal)
            .spacing(8)
            .vexpand(true)
            .build();
        main_section.append(&main_section_row_1);

        let transcode_form = TranscodeFormWidget::builder()
            .build();
        main_section_row_1.append(&transcode_form);

        self.obj().bind_property("selected-video", &transcode_form, "video")
            .sync_create()
            .build();

        self.title_form.replace(Some(title_form));
        self.video_player.replace(Some(video_player));

        main_section
    }

    /// Create the transcode queue section (right column) of the transcode page.
    fn create_transcode_queue(&self) -> TranscodeQueueWidget {
        let transcode_queue = TranscodeQueueWidget::new();
        transcode_queue
    }

    /// Create the video list section (left column) of the transcode page.
    fn create_video_list(&self) -> TranscodeListWidget {
        let context = self.context
            .borrow()
            .clone()
            .unwrap();
        let transcode_list = TranscodeListWidget::new(&context);

        let obj = self.obj();
        transcode_list.connect_video_selected(glib::clone!(
            #[weak]
            obj,
            move |video| {
                obj.set_selected_video(Some(video.clone()));
            }
        ));

        transcode_list
    }
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
        self.build_ui();
    }
}

impl WidgetImpl for TranscodePageWidget {}

impl BoxImpl for TranscodePageWidget {}
