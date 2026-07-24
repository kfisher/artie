// Copyright 2026 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! Defines the transcode queue widget.
//!
//! TODO

mod imp;

use gtk::glib::{self, Object};

use crate::ui::ContextObject;

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
    /// The application context for the UI.
    context: Option<ContextObject>,
}

impl Builder {
    /// Create a new [`Builder`] instance.
    pub fn new() -> Self {
        Self {
            context: None,
        }
    }

    /// Build a [`TranscodeQueueWidget`] instance based off the parameters provided to the builder.
    pub fn build(self) -> TranscodeQueueWidget {
        let obj: TranscodeQueueWidget = Object::builder()
            .property("context", self.context.unwrap())
            .build();
        obj
    }

    /// Set the application context.
    pub fn context(mut self, context: &ContextObject) -> Self {
        self.context = Some(context.clone());
        self
    }
}

