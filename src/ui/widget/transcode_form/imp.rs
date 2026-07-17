// Copyright 2026 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! Widget implementation.

use std::cell::RefCell;

use glib::{self, Properties};

use gtk::{Box, Label, Orientation};
use gtk::prelude::*;
use gtk::subclass::prelude::*;

use handbrake;

use crate::ui::data::VideoObject;
use crate::ui::widget::{AudioTrackFieldWidget, DropDownWidget, IconButton};

#[derive(Default, Properties)]
#[properties(wrapper_type = super::TranscodeFormWidget)]
pub struct TranscodeFormWidget {
    /// The active video.
    #[property(get, set = Self::set_video, nullable)]
    pub(super) video: RefCell<Option<VideoObject>>,

    /// Dropdown for selecting the HandBrake preset to use when transcoding.
    pub(super) preset_dropdown: RefCell<DropDownWidget>,

    /// Dropdown for selecting the video encoder to use.
    pub(super) video_encoder_dropdown: RefCell<DropDownWidget>,
}

impl TranscodeFormWidget {
    /// Builds the widget.
    fn build_ui(&self) {
        let obj = self.obj();
        obj.set_hexpand(true);
        obj.set_orientation(Orientation::Vertical);
        obj.add_css_class("transcode-form-widget");

        let row_0 = self.create_header();
        obj.append(&row_0);

        let row_1 = Box::builder()
            .orientation(Orientation::Horizontal)
            .spacing(8)
            .build();
        obj.append(&row_1);

        let column_0 = Box::builder()
            .hexpand(true)
            .orientation(Orientation::Vertical)
            .spacing(8)
            .build();
        column_0.append(&self.create_preset_field());
        column_0.append(&self.create_video_encoder_field());
        column_0.add_css_class("column");
        row_1.append(&column_0);

        let column_1 = Box::builder()
            .hexpand(true)
            .orientation(Orientation::Vertical)
            .spacing(8)
            .build();
        column_1.append(&self.create_audio_track_field());
        column_1.add_css_class("column");
        row_1.append(&column_1);

        let column_2 = Box::builder()
            .hexpand(true)
            .orientation(Orientation::Vertical)
            .spacing(8)
            .build();
        column_2.add_css_class("column");
        row_1.append(&column_2);

        let row_2 = self.create_footer();
        obj.append(&row_2);
    }

    /// Creates the widget used for editing audio tracks.
    fn create_audio_track_field(&self) -> AudioTrackFieldWidget {
        let widget = AudioTrackFieldWidget::builder()
            .build();

        self.obj().bind_property("video", &widget, "video")
            .sync_create()
            .build();

        widget
    }

    /// Creates the footer for the transcode form.
    fn create_footer(&self) -> Box {
        let footer = Box::builder()
            .hexpand(true)
            .spacing(8)
            .orientation(Orientation::Horizontal)
            .build();
        footer.add_css_class("footer");

        // let delete_button = IconButton::builder()
        //     .icon_name("fontawesome.v7.solid.trash")
        //     .label("Delete Video")
        //     .danger_button()
        //     .build();
        // footer.append(&delete_button);

        let spacer = Box::builder()
            .hexpand(true)
            .orientation(Orientation::Horizontal)
            .build();
        footer.append(&spacer);

        // let archive_button = IconButton::builder()
        //     .icon_name("fontawesome.v7.solid.archive")
        //     .label("Archive Video")
        //     .build();
        // footer.append(&archive_button);

        let queue_button = IconButton::builder()
            .icon_name("fontawesome.v7.solid.film")
            .label("Queue Transcode")
            .secondary_button()
            .build();
        footer.append(&queue_button);

        footer
    }

    /// Creates the header for the transcode form.
    fn create_header(&self) -> Box {
        let header = Box::builder()
            .orientation(Orientation::Horizontal)
            .build();
        header.add_css_class("header");

        let title = Label::builder()
            .hexpand(true)
            .label("Transcode")
            .build();
        header.append(&title);

        header
    }

    /// Create the field for selecting the HandBrake preset.
    fn create_preset_field(&self) -> DropDownWidget {
        let dropdown = DropDownWidget::builder()
            .label("HandBrake Preset")
            .options(handbrake::presets())
            .build();
        self.preset_dropdown.replace(dropdown.clone());
        dropdown
    }

    /// Create the field for selecting the video encoder to use.
    fn create_video_encoder_field(&self) -> DropDownWidget {
        let dropdown = DropDownWidget::builder()
            .label("Video Encoder")
            .options(handbrake::video_encoders())
            .build();
        self.video_encoder_dropdown.replace(dropdown.clone());
        dropdown
    }

    /// Setter for the active video.
    ///
    /// # Args
    ///
    /// `video`:  The newly selected video. If `Some`, the form will be updated to reflect the
    /// provided video. If `None`, the form's values will be reset back to default. In both cases,
    /// any user provided changes will be reset.
    fn set_video(&self, video: Option<VideoObject>) {
        self.video.replace(video);
    }
}

#[glib::object_subclass]
impl ObjectSubclass for TranscodeFormWidget {
    const NAME: &'static str = "ArtieTranscodeFormWidget";
    type Type = super::TranscodeFormWidget;
    type ParentType = Box;
}

#[glib::derived_properties]
impl ObjectImpl for TranscodeFormWidget {
    fn constructed(&self) {
        self.parent_constructed();
        self.build_ui();
    }
}

impl WidgetImpl for TranscodeFormWidget {
}

impl BoxImpl for TranscodeFormWidget {
}

