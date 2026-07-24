// Copyright 2026 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! Defines the transcode queue widget.
//!
//! TODO

mod imp;

use gtk::glib::{self, Object};

glib::wrapper! {
    pub struct TranscodeQueueWidget(ObjectSubclass<imp::TranscodeQueueWidget>)
        @extends gtk::Box,
                 gtk::Widget,
        @implements gtk::Accessible,
                    gtk::Buildable,
                    gtk::ConstraintTarget,
                    gtk::Orientable;
}

impl TranscodeQueueWidget {
    /// Create a builder instance for creating instances of [`TranscodeQueueWidget`].
    pub fn builder() -> Builder {
        Builder::new()
    }
}

pub struct Builder {
}

impl Builder {
    /// Create a new [`Builder`] instance.
    pub fn new() -> Self {
        Self {
        }
    }

    /// Build a [`TranscodeQueueWidget`] instance based off the parameters provided to the builder.
    pub fn build(self) -> TranscodeQueueWidget {
        let obj: TranscodeQueueWidget = Object::builder()
            .build();
        obj
    }
}

