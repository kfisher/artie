// Copyright 2026 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! Field for configuring the audio tracks for a transcode operation.

mod imp;

use glib::{self, Object};

glib::wrapper! {
    pub struct AudioTrackFieldWidget(ObjectSubclass<imp::AudioTrackFieldWidget>)
        @extends gtk::Box,
                 gtk::Widget,
        @implements gtk::Accessible,
                    gtk::Buildable,
                    gtk::ConstraintTarget,
                    gtk::Orientable;
}

impl AudioTrackFieldWidget {
    /// Create a builder instance for creating instances of [`AudioTrackFieldWidget`].
    pub fn builder() -> Builder {
        Builder::new()
    }
}

/// Provides a builder-pattern for creating instances of [`AudioTrackFieldWidget`].
pub struct Builder {
}

impl Builder {
    /// Create a new [`Builder`] instance.
    fn new() -> Self {
        Self {
        }
    }

    /// Build a [`AudioTrackFieldWidget`] instance based off the parameters provided to the builder.
    pub fn build(&self) -> AudioTrackFieldWidget {
        Object::builder()
            .build()
    }
}
