// Copyright 2026 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! Field for configuring the subtitle tracks for a transcode operation.

mod imp;

use glib::{self, Object};

glib::wrapper! {
    pub struct SubtitleTrackFieldWidget(ObjectSubclass<imp::SubtitleTrackFieldWidget>)
        @extends gtk::Box,
                 gtk::Widget,
        @implements gtk::Accessible,
                    gtk::Buildable,
                    gtk::ConstraintTarget,
                    gtk::Orientable;
}

impl SubtitleTrackFieldWidget {
    /// Create a builder instance for creating instances of [`SubtitleTrackFieldWidget`].
    pub fn builder() -> Builder {
        Builder::new()
    }
}

/// Provides a builder-pattern for creating instances of [`SubtitleTrackFieldWidget`].
pub struct Builder {
}

impl Builder {
    /// Create a new [`Builder`] instance.
    fn new() -> Self {
        Self {
        }
    }

    /// Build a [`SubtitleTrackFieldWidget`] instance based off the parameters provided to the
    /// builder.
    pub fn build(&self) -> SubtitleTrackFieldWidget {
        Object::builder()
            .build()
    }
}
