// Copyright 2026 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! Defines the transcode page widget.
//!
//! The transcode page is the page used to initiate, monitor, and terminate transcode operations.

mod imp;

use glib::{self, Object};

use crate::ui::ContextObject;

glib::wrapper! {
    pub struct TranscodePageWidget(ObjectSubclass<imp::TranscodePageWidget>)
        @extends gtk::Box,
                 gtk::Widget,
        @implements gtk::Accessible,
                    gtk::Buildable,
                    gtk::ConstraintTarget,
                    gtk::Orientable;
}

impl TranscodePageWidget {
    /// Create a builder instance for creating instances of [`TranscodePageWidget`].
    pub fn builder() -> Builder {
        Builder::new()
    }
}

/// Provides a builder-pattern for creating instances of [`TranscodePageWidget`].
pub struct Builder {
    /// The application context for the UI.
    context: Option<ContextObject>,
}

impl Builder {
    fn new() -> Self {
        Self {
            context: None,
        }
    }

    /// Build a [`TranscodePageWidget`] instance based off the parameters provided to the builder.
    pub fn build(self) -> TranscodePageWidget {
        Object::builder()
            .property("context", self.context.unwrap())
            .build()
    }

    /// Set the application context.
    pub fn context(mut self, context: &ContextObject) -> Self {
        self.context = Some(context.clone());
        self
    }
}

