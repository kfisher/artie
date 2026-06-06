// Copyright 2026 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! GObject representation of a video.

use gst::glib::object::{Cast, CastNone};
use gtk::gio::ListStore;
use gtk::gio::prelude::ListModelExt;
use gtk::glib::{self, Object};
use gtk::subclass::prelude::*;

use crate::path;
use crate::models::Video;
use crate::ui::data::{AudioTrackObject, SubtitleTrackObject, TitleObject, VideoTrackObject};
use crate::ui::helpers;


glib::wrapper! {
    pub struct VideoObject(ObjectSubclass<imp::VideoObject>);
}

impl VideoObject {
    /// Creates a new video object instance from a [`models::Video`].
    ///
    /// # Panics
    ///
    /// This will panic if the GObject cannot be created.
    pub fn new(video: &Video) -> Self {
        let title = video.title.value
            .as_ref()
            .map(|t| TitleObject::new(t.as_ref()));

        let audio_tracks = video.audio_tracks
            .iter()
            .map(|t| AudioTrackObject::new(t));
        let audio_tracks = ListStore::from_iter(audio_tracks);

        let subtitle_tracks = video.subtitle_tracks
            .iter()
            .map(|t| SubtitleTrackObject::new(t));
        let subtitle_tracks = ListStore::from_iter(subtitle_tracks);

        let video_tracks = video.video_tracks
            .iter()
            .map(|t| VideoTrackObject::new(t));
        let video_tracks = ListStore::from_iter(video_tracks);

        let path = path::location_path(&video.location)
            .unwrap_or_default();

        let obj: VideoObject = Object::builder()
            .property("id", video.id)
            .property("title", title)
            .property("duration", helpers::format_duration(&video.duration))
            .property("path", path)
            .property("audio-tracks", audio_tracks)
            .property("subtitle-tracks", subtitle_tracks)
            .property("video-tracks", video_tracks)
            .build();

        obj.setup_bindings();

        obj
    }

    /// Get an audio track.
    ///
    /// # Args
    ///
    /// `index`:  The track's index (zero indexed)
    pub fn get_audio_track(&self, index: u32) -> Option<AudioTrackObject> {
        self.imp().audio_tracks
            .borrow()
            .as_ref()
            .and_then(|list_store| list_store.item(index).and_downcast())
    }

    /// Get an subtitle track.
    ///
    /// # Args
    ///
    /// `index`:  The track's index (zero indexed)
    pub fn get_subtitle_track(&self, index: u32) -> Option<SubtitleTrackObject> {
        self.imp().subtitle_tracks
            .borrow()
            .as_ref()
            .and_then(|list_store| list_store.item(index).and_downcast())
    }

    /// Get an video track.
    ///
    /// # Args
    ///
    /// `index`:  The track's index (zero indexed)
    pub fn get_video_track(&self, index: u32) -> Option<VideoTrackObject> {
        self.imp().video_tracks
            .borrow()
            .as_ref()
            .and_then(|list_store| list_store.item(index).and_downcast())
    }

    /// Resets the preview data back to default values.
    pub fn reset_preview_data(&self) {
        let audio_tracks = self.audio_tracks().unwrap();
        for audio_track in &audio_tracks {
            audio_track.unwrap()
                .downcast_ref::<AudioTrackObject>()
                .unwrap()
                .preview()
                .reset();
        }

        let subtitle_tracks = self.subtitle_tracks().unwrap();
        for subtitle_track in &subtitle_tracks {
            subtitle_track.unwrap()
                .downcast_ref::<SubtitleTrackObject>()
                .unwrap()
                .preview()
                .reset();
        }

        let video_tracks = self.video_tracks().unwrap();
        for video_track in &video_tracks {
            video_track.unwrap()
                .downcast_ref::<VideoTrackObject>()
                .unwrap()
                .preview()
                .reset();
        }

        let imp = self.imp();
        imp.clear_preview_tracks();
    }

    /// Configure the bindings for the video object.
    fn setup_bindings(&self) {
        let audio_tracks = self.audio_tracks()
            .unwrap();
        for audio_track in &audio_tracks {
            let audio_track = audio_track
                .unwrap()
                .downcast::<AudioTrackObject>()
                .unwrap();
            self.bind_audio_track_object(audio_track);
        }
    }

    /// Bind to an audio track.
    ///
    /// # Args
    ///
    /// `audio_track`:  The audio track to bind to.
    fn bind_audio_track_object(&self, audio_track: AudioTrackObject) {
        let video = self;
        audio_track.preview().connect_selected_notify(glib::clone!(
            #[weak]
            audio_track,
            #[weak]
            video,
            move |preview| {
                video.preview_audio_track(audio_track, preview.selected());
            }
        ));
    }

    /// Update the audio track that will be previewed.
    ///
    /// # Args
    ///
    /// `audio_track`:  The audio track to preview. It is assumed that this audio track belongs to
    /// this video and the video's stream_id has been set.
    fn preview_audio_track(&self, audio_track: AudioTrackObject, selected: bool) {
        let imp = self.imp();
        if selected {
            imp.set_preview_audio_track(audio_track);
            self.preview_changed();
        }
    }

    // TODO
    fn preview_changed(&self) {
        tracing::warn!("TODO: UPDATE VIDEO")
    }
}

mod imp {
    use std::cell::{Cell, RefCell};

    use gtk::glib::{self, Object, Properties};
    use gtk::gio::ListStore;
    use gtk::prelude::*;
    use gtk::subclass::prelude::*;

    use crate::ui::data::{
        AudioTrackObject,
        SubtitleTrackObject,
        TitleObject,
        VideoTrackObject,
    };

    #[derive(Default, Properties)]
    #[properties(wrapper_type = super::VideoObject)]
    pub struct VideoObject {
        /// Unique id of the video (primary key).
        #[property(name = "id", get, set, type = u32)]
        pub(super) id: Cell<u32>,

        /// Unique id of the video (primary key).
        #[property(name = "title", get, set, type = Object)]
        pub(super) title: RefCell<Option<TitleObject>>,

        /// The video's runtime.
        #[property(name = "duration", get, set, type = String)]
        pub(super) duration: RefCell<String>,

        /// The path to the video.
        #[property(name = "path", get, set, type = String)]
        pub(super) path: RefCell<String>,

        /// List of audio tracks.
        ///
        /// Each item should be a [`crate::ui::data::AudioTrackObject`] instance.
        #[property(name = "audio-tracks", get, set)]
        pub(super) audio_tracks: RefCell<Option<ListStore>>,

        /// List of sutitle tracks.
        ///
        /// Each item should be a [`crate::ui::data::SubtitleTrackObject`] instance.
        #[property(name = "subtitle-tracks", get, set)]
        pub(super) subtitle_tracks: RefCell<Option<ListStore>>,

        /// List of video tracks.
        ///
        /// Each item should be a [`crate::ui::data::VideoTrackObject`] instance.
        #[property(name = "video-tracks", get, set)]
        pub(super) video_tracks: RefCell<Option<ListStore>>,

        /// The audio track currently being previewed.
        ///
        /// This is only applicable in the context of the transcode page when the video is
        /// selected.
        pub(super) preview_audio_track: RefCell<Option<AudioTrackObject>>,

        /// The subtitle track currently being previewed.
        ///
        /// This is only applicable in the context of the transcode page when the video is
        /// selected.
        pub(super) preview_subtitle_track: RefCell<Option<SubtitleTrackObject>>,

        /// The video track currently being previewed.
        ///
        /// This is only applicable in the context of the transcode page when the video is
        /// selected.
        pub(super) preview_video_track: RefCell<Option<VideoTrackObject>>,
    }

    impl VideoObject {
        /// Resets the preview track fields back to `None`.
        pub(super) fn clear_preview_tracks(&self) {
            self.preview_audio_track.replace(None);
            self.preview_subtitle_track.replace(None);
            self.preview_video_track.replace(None);
        }

        /// Sets the preview audio track.
        ///
        /// # Args
        ///
        /// `audio_track`:  The new selected audio track.
        pub(super) fn set_preview_audio_track(&self, audio_track: AudioTrackObject) {
            tracing::trace!(track=audio_track.name(), "preview audio track changed");
            self.preview_audio_track.replace(Some(audio_track));
        }

        /// Sets the preview subtitle track.
        ///
        /// # Args
        ///
        /// `subtitle_track`:  The new selected subtitle track.
        pub(super) fn set_preview_subtitle_track(&self, subtitle_track: SubtitleTrackObject) {
            tracing::trace!(language=subtitle_track.language(), "preview subtitle track changed");
            self.preview_subtitle_track.replace(Some(subtitle_track));
        }

        /// Sets the preview video track.
        ///
        /// # Args
        ///
        /// `video_track`:  The new selected video track.
        pub(super) fn set_preview_video_track(&self, video_track: VideoTrackObject) {
            tracing::trace!("preview video track changed");
            self.preview_video_track.replace(Some(video_track));
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for VideoObject {
        const NAME: &'static str = "ArtieVideoObject";
        type Type = super::VideoObject;
    }

    #[glib::derived_properties]
    impl ObjectImpl for VideoObject {}
}

#[cfg(test)]
mod tests {
    // TODO[TESTS]
}

