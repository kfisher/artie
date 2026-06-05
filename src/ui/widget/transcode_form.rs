// Copyright 2025 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! Widget for entering transcode parameters.

use gtk::{
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

use crate::ui::data::{AudioTrackObject, SubtitleTrackObject, VideoObject, VideoTrackObject};

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
        self.create_and_replace_grid(&None);

        self.add_css_class("transcode-form-widget");
        self.set_hexpand(true);
        self.set_vexpand(true);
        self.set_orientation(Orientation::Vertical);
    }

    fn create_and_replace_grid(&self, video: &Option<VideoObject>) {
        let imp = self.imp();

        if let Some(old_grid) = imp.grid.borrow().as_ref() {
            self.remove(old_grid);
        }

        let new_grid = Grid::builder()
            .hexpand(true)
            .build();

        if let Some(video) = video {
            populate_grid(&new_grid, video);
        }

        self.append(&new_grid);
        imp.grid.replace(Some(new_grid));
    }
}

impl Default for TranscodeFormWidget {
    fn default() -> Self {
        Self::new()
    }
}

const HEADER_COL_SPAN: i32 = 1;

// TODO
fn build_audio_track_ui(grid: &Grid, starting_row: i32, audio_tracks: &ListStore) -> i32 {
    let audio_header = Label::builder()
        .hexpand(true)
        .label("Audio Tracks")
        .build();
    audio_header.add_css_class("header");

    let mut next_row = starting_row;
    grid.attach(&audio_header, 0, next_row, HEADER_COL_SPAN, 1);
    next_row += 1;

    let selection_model = NoSelection::new(Some(audio_tracks.clone()));
    let column_view = ColumnView::new(Some(selection_model));

    let name_factory = SignalListItemFactory::new();
    name_factory.connect_setup(|_, obj| {
        obj.downcast_ref::<ListItem>()
            .unwrap()
            .set_child(Some(&Label::new(None)));
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

    column_view.append_column(&ColumnViewColumn::new(Some("Name"), Some(name_factory)));
    column_view.append_column(&ColumnViewColumn::new(Some("Codec"), Some(codec_factory)));
    column_view.append_column(&ColumnViewColumn::new(Some("Language"), Some(language_factory)));
    column_view.append_column(&ColumnViewColumn::new(Some("Layout"), Some(layout_factory)));

    grid.attach(&column_view, 0, next_row, 1, 1);
    next_row += 1;

    next_row
}

// TODO
fn build_subtitle_track_ui(grid: &Grid, starting_row: i32, subtitle_tracks: &ListStore) -> i32 {
    let subtitle_header = Label::builder()
        .label("Subtitle Tracks")
        .build();
    subtitle_header.add_css_class("header");

    let mut next_row = starting_row;
    grid.attach(&subtitle_header, 0, next_row, HEADER_COL_SPAN, 1);
    next_row += 1;

    let selection_model = NoSelection::new(Some(subtitle_tracks.clone()));
    let column_view = ColumnView::new(Some(selection_model));

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

    let language_factory = SignalListItemFactory::new();
    language_factory.connect_setup(|_, obj| {
        obj.downcast_ref::<ListItem>()
            .unwrap()
            .set_child(Some(&Label::new(None)));
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

    column_view.append_column(&ColumnViewColumn::new(Some("Codec"), Some(codec_factory)));
    column_view.append_column(&ColumnViewColumn::new(Some("Language"), Some(language_factory)));

    grid.attach(&column_view, 0, next_row, 1, 1);
    next_row += 1;

    next_row
}

// TODO
fn build_video_track_ui(grid: &Grid, starting_row: i32, video_tracks: &ListStore) -> i32 {
    let video_header = Label::builder()
        .label("Video Tracks")
        .build();
    video_header.add_css_class("header");

    let mut next_row = starting_row;
    grid.attach(&video_header, 0, next_row, HEADER_COL_SPAN, 1);
    next_row += 1;

    let selection_model = NoSelection::new(Some(video_tracks.clone()));
    let column_view = ColumnView::new(Some(selection_model));

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
            .downcast::<VideoTrackObject>()
            .unwrap();
        item.child()
            .unwrap()
            .downcast::<Label>()
            .unwrap()
            .set_label(&track.codec());
    });

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

    column_view.append_column(&ColumnViewColumn::new(Some("Codec"), Some(codec_factory)));
    column_view.append_column(&ColumnViewColumn::new(Some("Size"), Some(size_factory)));
    column_view.append_column(&ColumnViewColumn::new(Some("Aspect Ratio"), Some(aspect_ratio_factory)));

    grid.attach(&column_view, 0, next_row, 1, 1);
    next_row += 1;

    next_row
}

// TODO
fn populate_grid(grid: &Grid, video: &VideoObject) {
    let audio_tracks = video.audio_tracks()
        .unwrap();

    let subtitle_tracks = video.subtitle_tracks()
        .unwrap();

    let video_tracks = video.video_tracks()
        .unwrap();

    let next_row = build_video_track_ui(&grid, 0, &video_tracks);
    let next_row = build_audio_track_ui(&grid, next_row, &audio_tracks);
    build_subtitle_track_ui(&grid, next_row, &subtitle_tracks);
}

mod imp {
    use std::cell::RefCell;

    use gtk::{Box, Grid};
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

        // TODO
        pub(super) grid: RefCell<Option<Grid>>,
    }

    impl TranscodeFormWidget {
        fn set_video(&self, video: Option<VideoObject>) {
            self.obj().create_and_replace_grid(&video);
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
