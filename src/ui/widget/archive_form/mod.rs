// Copyright 2026 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! Widget for initiating a transcode operation.

mod imp;

use glib::{self, Object};

glib::wrapper! {
    pub struct ArchiveFormWidget(ObjectSubclass<imp::ArchiveFormWidget>)
        @extends gtk::Box,
                 gtk::Widget,
        @implements gtk::Accessible,
                    gtk::Buildable,
                    gtk::ConstraintTarget,
                    gtk::Orientable;
}

impl ArchiveFormWidget {
    /// Creates a new builder instance which can be used to create instances of
    /// [`ArchiveFormWidget`].
    pub fn builder() -> Builder {
        Builder::default()
    }
}

/// Builder for [`ArchiveFormWidget`].
#[derive(Default)]
pub struct Builder {
}

impl Builder {
    /// Build a [`ArchiveFormWidget`] instance based on builder parameters.
    pub fn build(&self) -> ArchiveFormWidget {
        Object::builder()
            .build()
    }
}

