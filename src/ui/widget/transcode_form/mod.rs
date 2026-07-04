// Copyright 2026 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! Widget for initiating a transcode operation.

mod imp;

use glib::{self, Object};

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
    /// Creates a new builder instance which can be used to create instances of
    /// [`TranscodeFormWidget`].
    pub fn builder() -> Builder {
        Builder::default()
    }
}

/// Builder for [`TranscodeFormWidget`].
#[derive(Default)]
pub struct Builder {
}

impl Builder {
    /// Build a [`TranscodeFormWidget`] instance based on builder parameters.
    pub fn build(&self) -> TranscodeFormWidget {
        Object::builder()
            .build()
    }
}
