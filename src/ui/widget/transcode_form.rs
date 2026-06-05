// Copyright 2025 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! Widget for entering transcode parameters.

use gtk::{
    Align,
    CheckButton,
    ColumnView,
    ColumnViewColumn,
    Grid,
    Label,
    ListItem,
    NoSelection,
    Orientation,
    SignalListItemFactory,
};
use gtk::gio::ListStore;
use gtk::glib::{self, Object};
use gtk::prelude::*;
use gtk::subclass::prelude::*;

use crate::ui::data::{AudioTrackObject, SubtitleTrackObject, VideoTrackObject};

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
        // let audio_tracks = video.audio_tracks()
        //     .unwrap();

        // let subtitle_tracks = video.subtitle_tracks()
        //     .unwrap();

        // let video_tracks = video.video_tracks()
        //     .unwrap();

        let video_track_header = create_header("Video Tracks");
        let video_track_table  = create_video_track_table();

        let audio_track_header = create_header("Audio Tracks");
        let audio_track_table  = create_audio_track_table();

        let subtitle_track_header = create_header("Subtitle Tracks");
        let subtitle_track_table  = create_subtitle_track_table();

        let grid = Grid::builder()
            .hexpand(true)
            .build();
        //                                  +---------- Column
        //                                  |, +------- Row
        //                                  |, |  +---- Width
        //                                  |, |  |  +- Height
        grid.attach(&video_track_header,    0, 0, 1, 1);
        grid.attach(&video_track_table,     0, 1, 1, 1);
        grid.attach(&audio_track_header,    0, 2, 1, 1);
        grid.attach(&audio_track_table,     0, 3, 1, 1);
        grid.attach(&subtitle_track_header, 0, 4, 1, 1);
        grid.attach(&subtitle_track_table,  0, 5, 1, 1);

        self.append(&grid);

        self.add_css_class("transcode-form-widget");
        self.set_hexpand(true);
        self.set_vexpand(true);
        self.set_orientation(Orientation::Vertical);

        let imp = self.imp();
        imp.grid.replace(Some(grid));
        imp.audio_track_table.replace(Some(audio_track_table));
        imp.subtitle_track_table.replace(Some(subtitle_track_table));
        imp.video_track_table.replace(Some(video_track_table));
    }
}

impl Default for TranscodeFormWidget {
    fn default() -> Self {
        Self::new()
    }
}

/// Create the table for displaying audio track information.
fn create_audio_track_table() -> ColumnView {
    let empty_list = ListStore::new::<AudioTrackObject>();
    let selection_model = NoSelection::new(Some(empty_list));
    let column_view = ColumnView::builder()
        .model(&selection_model)
        .build();

    let group_leader = CheckButton::new();
    let preview_factory = SignalListItemFactory::new();
    preview_factory.connect_setup(move |_, obj| {
        let button = CheckButton::builder()
            .halign(Align::Center)
            .hexpand(false)
            .group(&group_leader)
            .build();
        obj.downcast_ref::<ListItem>()
            .unwrap()
            .set_child(Some(&button));
    });

    let preview_column = ColumnViewColumn::builder()
        .title("Preview")
        .factory(&preview_factory)
        .build();

    let name_factory = SignalListItemFactory::new();
    name_factory.connect_setup(|_, obj| {
        let label = Label::builder()
            .halign(Align::Start)
            .hexpand(true)
            .build();
        obj.downcast_ref::<ListItem>()
            .unwrap()
            .set_child(Some(&label));
    });
    name_factory.connect_bind(|_, obj| {
        let item = obj.downcast_ref::<ListItem>().unwrap();
        let track = item.item()
            .unwrap()
            .downcast::<AudioTrackObject>()
            .unwrap();
        item.child()
            .unwrap()
            .downcast::<Label>()
            .unwrap()
            .set_label(&track.name());
    });

    let name_column = ColumnViewColumn::builder()
        .title("Name")
        .expand(true)
        .factory(&name_factory)
        .build();

    let codec_factory = SignalListItemFactory::new();
    codec_factory.connect_setup(|_, obj| {
        obj.downcast_ref::<ListItem>()
            .unwrap()
            .set_child(Some(&Label::new(None)));
    });
    codec_factory.connect_bind(|_, obj| {
        let item = obj.downcast_ref::<ListItem>().unwrap();
        let track = item.item()
            .unwrap()
            .downcast::<AudioTrackObject>()
            .unwrap();
        item.child()
            .unwrap()
            .downcast::<Label>()
            .unwrap()
            .set_label(&track.codec());
    });

    let codec_column = ColumnViewColumn::builder()
        .title("Codec")
        .factory(&codec_factory)
        .build();

    let language_factory = SignalListItemFactory::new();
    language_factory.connect_setup(|_, obj| {
        obj.downcast_ref::<ListItem>()
            .unwrap()
            .set_child(Some(&Label::new(None)));
    });
    language_factory.connect_bind(|_, obj| {
        let item = obj.downcast_ref::<ListItem>().unwrap();
        let track = item.item()
            .unwrap()
            .downcast::<AudioTrackObject>()
            .unwrap();
        item.child()
            .unwrap()
            .downcast::<Label>()
            .unwrap()
            .set_label(&track.language());
    });

    let language_column = ColumnViewColumn::builder()
        .title("Language")
        .factory(&language_factory)
        .build();

    let layout_factory = SignalListItemFactory::new();
    layout_factory.connect_setup(|_, obj| {
        obj.downcast_ref::<ListItem>()
            .unwrap()
            .set_child(Some(&Label::new(None)));
    });
    layout_factory.connect_bind(|_, obj| {
        let item = obj.downcast_ref::<ListItem>().unwrap();
        let track = item.item()
            .unwrap()
            .downcast::<AudioTrackObject>()
            .unwrap();
        item.child()
            .unwrap()
            .downcast::<Label>()
            .unwrap()
            .set_label(&track.layout());
    });

    let layout_column = ColumnViewColumn::builder()
        .title("Layout")
        .factory(&layout_factory)
        .build();

    column_view.append_column(&preview_column);
    column_view.append_column(&name_column);
    column_view.append_column(&codec_column);
    column_view.append_column(&language_column);
    column_view.append_column(&layout_column);

    column_view
}

/// Create the table for displaying subtitle track information.
fn create_subtitle_track_table() -> ColumnView {
    let empty_list = ListStore::new::<SubtitleTrackObject>();
    let selection_model = NoSelection::new(Some(empty_list));
    let column_view = ColumnView::new(Some(selection_model));

    let group_leader = CheckButton::new();
    let preview_factory = SignalListItemFactory::new();
    preview_factory.connect_setup(move |_, obj| {
        let button = CheckButton::builder()
            .halign(Align::Center)
            .hexpand(false)
            .group(&group_leader)
            .build();
        obj.downcast_ref::<ListItem>()
            .unwrap()
            .set_child(Some(&button));
    });

    let preview_column = ColumnViewColumn::builder()
        .title("Preview")
        .factory(&preview_factory)
        .build();

    let codec_factory = SignalListItemFactory::new();
    codec_factory.connect_setup(|_, obj| {
        obj.downcast_ref::<ListItem>()
            .unwrap()
            .set_child(Some(&Label::new(None)));
    });
    codec_factory.connect_bind(|_, obj| {
        let item = obj.downcast_ref::<ListItem>().unwrap();
        let track = item.item()
            .unwrap()
            .downcast::<SubtitleTrackObject>()
            .unwrap();
        item.child()
            .unwrap()
            .downcast::<Label>()
            .unwrap()
            .set_label(&track.codec());
    });

    let codec_column = ColumnViewColumn::builder()
        .title("Codec")
        .factory(&codec_factory)
        .build();

    let language_factory = SignalListItemFactory::new();
    language_factory.connect_setup(|_, obj| {
        let label = Label::builder()
            .halign(Align::Start)
            .build();
        obj.downcast_ref::<ListItem>()
            .unwrap()
            .set_child(Some(&label));
    });
    language_factory.connect_bind(|_, obj| {
        let item = obj.downcast_ref::<ListItem>()
            .unwrap();
        let track = item.item()
            .unwrap()
            .downcast::<SubtitleTrackObject>()
            .unwrap();
        item.child()
            .unwrap()
            .downcast::<Label>()
            .unwrap()
            .set_label(&track.language());
    });

    let language_column = ColumnViewColumn::builder()
        .expand(true)
        .title("Language")
        .factory(&language_factory)
        .build();

    column_view.append_column(&preview_column);
    column_view.append_column(&language_column);
    column_view.append_column(&codec_column);

    column_view
}

/// Create the table for displaying video track information.
fn create_video_track_table() -> ColumnView {
    let empty_list = ListStore::new::<VideoTrackObject>();
    let selection_model = NoSelection::new(Some(empty_list));
    let column_view = ColumnView::new(Some(selection_model));

    let group_leader = CheckButton::new();
    let preview_factory = SignalListItemFactory::new();
    preview_factory.connect_setup(move |_, obj| {
        let button = CheckButton::builder()
            .halign(Align::Center)
            .hexpand(false)
            .group(&group_leader)
            .build();
        obj.downcast_ref::<ListItem>()
            .unwrap()
            .set_child(Some(&button));
    });

    let preview_column = ColumnViewColumn::builder()
        .title("Preview")
        .factory(&preview_factory)
        .build();

    let codec_factory = SignalListItemFactory::new();
    codec_factory.connect_setup(|_, obj| {
        let label = Label::builder()
            .halign(Align::Start)
            .build();
        obj.downcast_ref::<ListItem>()
            .unwrap()
            .set_child(Some(&label));
    });
    codec_factory.connect_bind(|_, obj| {
        let item = obj.downcast_ref::<ListItem>().unwrap();
        let track = item.item()
            .unwrap()
            .downcast::<VideoTrackObject>()
            .unwrap();
        item.child()
            .unwrap()
            .downcast::<Label>()
            .unwrap()
            .set_label(&track.codec());
    });

    let codec_column = ColumnViewColumn::builder()
        .expand(true)
        .title("Codec")
        .factory(&codec_factory)
        .build();

    let size_factory = SignalListItemFactory::new();
    size_factory.connect_setup(|_, obj| {
        obj.downcast_ref::<ListItem>()
            .unwrap()
            .set_child(Some(&Label::new(None)));
    });
    size_factory.connect_bind(|_, obj| {
        let item = obj.downcast_ref::<ListItem>().unwrap();
        let track = item.item()
            .unwrap()
            .downcast::<VideoTrackObject>()
            .unwrap();
        item.child()
            .unwrap()
            .downcast::<Label>()
            .unwrap()
            .set_label(&track.size());
    });

    let size_column = ColumnViewColumn::builder()
        .title("Size")
        .factory(&size_factory)
        .build();

    let aspect_ratio_factory = SignalListItemFactory::new();
    aspect_ratio_factory.connect_setup(|_, obj| {
        obj.downcast_ref::<ListItem>()
            .unwrap()
            .set_child(Some(&Label::new(None)));
    });
    aspect_ratio_factory.connect_bind(|_, obj| {
        let item = obj.downcast_ref::<ListItem>().unwrap();
        let track = item.item()
            .unwrap()
            .downcast::<VideoTrackObject>()
            .unwrap();
        item.child()
            .unwrap()
            .downcast::<Label>()
            .unwrap()
            .set_label(&track.aspect_ratio());
    });

    let aspect_ratio_column = ColumnViewColumn::builder()
        .title("Aspect Ratio")
        .factory(&aspect_ratio_factory)
        .build();

    column_view.append_column(&preview_column);
    column_view.append_column(&codec_column);
    column_view.append_column(&size_column);
    column_view.append_column(&aspect_ratio_column);

    column_view
}

/// Create a header row.
///
/// # Args
///
/// `text`:  The header text.
fn create_header(text: &str) -> Label {
    let audio_header = Label::builder()
        .hexpand(true)
        .label(text)
        .build();
    audio_header.add_css_class("header");

    audio_header
}

mod imp {
    use std::cell::RefCell;

    use gtk::{Box, ColumnView, Grid, NoSelection};
    use gtk::gio::ListStore;
    use gtk::glib::{self, Properties};
    use gtk::prelude::*;
    use gtk::subclass::prelude::*;

    use crate::ui::data::VideoObject;

    #[derive(Default, Properties)]
    #[properties(wrapper_type = super::TranscodeFormWidget)]
    pub struct TranscodeFormWidget {
        /// The active video.
        #[property(get, set = Self::set_video, nullable)]
        pub(super) video: RefCell<Option<VideoObject>>,

        /// The base grid layout.
        pub(super) grid: RefCell<Option<Grid>>,

        /// The table used to display audio track information and controls.
        pub(super) audio_track_table: RefCell<Option<ColumnView>>,

        /// The table used to display subtitle track information and controls.
        pub(super) subtitle_track_table: RefCell<Option<ColumnView>>,

        /// The table used to display video track information and controls.
        pub(super) video_track_table: RefCell<Option<ColumnView>>,
    }

    impl TranscodeFormWidget {
        /// Setter for the active video.
        ///
        /// This will also update the audio, subtitle, and video track tables.
        ///
        /// # Args
        ///
        /// `video`:  The new selected video. `None` if no video is selected or a video was
        /// unselected without selecting another.
        fn set_video(&self, video: Option<VideoObject>) {
            if let Some(video) = &video {
                self.update_audio_model(video.audio_tracks());
                self.update_subtitle_model(video.subtitle_tracks());
                self.update_video_model(video.video_tracks());
            } else {
                self.update_audio_model(None);
                self.update_subtitle_model(None);
                self.update_video_model(None);
            }

            self.video.replace(video);
        }

        /// Update the audio track table.
        ///
        /// # Args
        ///
        /// `audio_tracks`:  The audio track data. `None` if a video is not selected.
        fn update_audio_model(&self, audio_tracks: Option<ListStore>) {
            self.audio_track_table
                .borrow()
                .as_ref()
                .unwrap()
                .set_model(Some(&NoSelection::new(audio_tracks)));
        }

        /// Update the subtitle track table.
        ///
        /// # Args
        ///
        /// `subtitle_tracks`:  The subtitle track data. `None` if a video is not selected.
        fn update_subtitle_model(&self, subtitle_tracks: Option<ListStore>) {
            self.subtitle_track_table
                .borrow()
                .as_ref()
                .unwrap()
                .set_model(Some(&NoSelection::new(subtitle_tracks)));
        }

        /// Update the video track table.
        ///
        /// # Args
        ///
        /// `video_tracks`:  The video track data. `None` if a video is not selected.
        fn update_video_model(&self, video_tracks: Option<ListStore>) {
            self.video_track_table
                .borrow()
                .as_ref()
                .unwrap()
                .set_model(Some(&NoSelection::new(video_tracks)));
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
