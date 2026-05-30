// Copyright 2026 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! Defines the transcode list item widget.
//!
//! The transcode list item widget is responsible for displaying an entry in the list of videos
//! that are available to be transcoded.

use gtk::prelude::{BoxExt, OrientableExt};
use gtk::{
    Align,
    Box,
    Label,
    Orientation,
};
use gtk::glib::{self, Object};
use gtk::prelude::*;
use gtk::subclass::prelude::*;

use crate::ui::data::VideoObject;

glib::wrapper! {
    pub struct TranscodeListItemWidget(ObjectSubclass<imp::TranscodeListItemWidget>)
        @extends gtk::Box,
                 gtk::Widget,
        @implements gtk::Accessible,
                    gtk::Buildable,
                    gtk::ConstraintTarget,
                    gtk::Orientable;
}

impl TranscodeListItemWidget {

    /// Creates a new drive widget instance.
    ///
    /// # Panics
    ///
    /// This will panic if the GObject cannot be created.
    pub fn new() -> Self {
        Object::builder().build()
    }

    /// Binds the widget to the provided video object.
    ///
    /// # Args
    ///
    /// `video_object`:  The video object to bind to.
    pub fn bind(&self, video_object: &VideoObject) {
        let imp = self.imp();
        let mut bindings = imp.bindings.borrow_mut();

        let title_object = video_object.title();

        let title = imp.title.borrow().clone();
        let title_binding = title_object
            .bind_property("title", &title, "label")
            .sync_create()
            .build();
        bindings.push(title_binding);

        let duration = imp.duration.borrow().clone();
        let duration_binding = video_object
            .bind_property("duration", &duration, "label")
            .sync_create()
            .build();
        bindings.push(duration_binding);

        let season_number = imp.season_number.borrow().clone();
        let season_number_binding = title_object
            .bind_property("season-number", &season_number, "label")
            .transform_to(|_, value: u32| {
                let text = if value != 0 {
                    format!("[ season: {:02} ]", value)
                } else {
                    String::from("[ season: - ]")
                };
                Some(text)
            })
            .sync_create()
            .build();
        bindings.push(season_number_binding);

        let disc_number = imp.disc_number.borrow().clone();
        let disc_number_binding = title_object
            .bind_property("disc-number", &disc_number, "label")
            .transform_to(|_, value: u32| {
                Some(format!("[ disc: {:02} ]", value))
            })
            .sync_create()
            .build();
        bindings.push(disc_number_binding);
    }

    /// Sets the selection state of the item.
    ///
    /// This will add or remove the "selected" CSS class based on the provided value.
    ///
    /// # Args
    ///
    /// `selected`  Indicates if the item is selected or not.
    pub fn set_selected(&self, selected: bool) {
        if selected {
            self.add_css_class("selected");
        } else {
            self.remove_css_class("selected");
        }
    }

    /// Unbinds the drive widget from the optical drive object which was bound when
    /// [`TranscodeListItemWidget::bind`] was called.
    pub fn unbind(&self) {
        for binding in self.imp().bindings.borrow_mut().drain(..) {
            binding.unbind();
        }
    }

    /// Builds the widget.
    fn build_ui(&self) {
        let title = Label::builder()
            .halign(Align::Start)
            .hexpand(true)
            .build();
        title.add_css_class("video-title");

        let duration = Label::builder()
            .label("00:00:00")
            .build();
        duration.add_css_class("video-duration");

        let row_0 = Box::builder()
            .hexpand(true)
            .orientation(Orientation::Horizontal)
            .build();
        row_0.append(&title);
        row_0.append(&duration);

        let season_number = Label::builder()
            .build();

        let disc_number = Label::builder()
            .build();

        let row_1 = Box::builder()
            .build();
        row_1.append(&season_number);
        row_1.append(&disc_number);
        row_1.add_css_class("video-info");

        self.set_orientation(Orientation::Vertical);
        self.append(&row_0);
        self.append(&row_1);
        self.add_css_class("transcode-list-item");

        let imp = self.imp();
        imp.title.replace(title);
        imp.duration.replace(duration);
        imp.season_number.replace(season_number);
        imp.disc_number.replace(disc_number);
    }
}

mod imp {
    //! Implementation for the optical drive widget.

    use std::cell::RefCell;

    use gtk::{Box, Label};
    use gtk::glib::{self, Binding};
    use gtk::subclass::prelude::*;

    /// Implementation for [`super::TranscodeListItemWidget`].
    #[derive(Default)]
    pub struct TranscodeListItemWidget {
        /// The widget's bindings.
        pub(super) bindings: RefCell<Vec<Binding>>,

        /// The title of the video.
        pub(super) title: RefCell<Label>,

        /// The duration of the video displayed in hours, minutes, and seconds.
        pub(super) duration: RefCell<Label>,

        /// The season number display of the video.
        ///
        /// For movies, this will be an empty string.
        pub(super) season_number: RefCell<Label>,

        /// The disc number display of the video.
        ///
        /// This is the disc the video was copied from or the disc of the source video this video
        /// was transcoded from.
        pub(super) disc_number: RefCell<Label>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for TranscodeListItemWidget {
        const NAME: &'static str = "ArtieTranscodeListItemWidget";
        type Type = super::TranscodeListItemWidget;
        type ParentType = Box;
    }

    impl ObjectImpl for TranscodeListItemWidget {
        fn constructed(&self) {
            self.parent_constructed();
            self.obj().build_ui();
        }
    }

    impl WidgetImpl for TranscodeListItemWidget {}

    impl BoxImpl for TranscodeListItemWidget {}
}

#[cfg(test)]
mod tests {
    // TODO[TESTS]
}
