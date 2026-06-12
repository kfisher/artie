// Copyright 2025 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! Title related utilities and operations.

use crate::Result;
use crate::bus::Handle;
use crate::db;
use crate::models::Title;

/// Update a title's data in the database.
///
/// # Args
///
/// `bus`:  Handle for communicating with the various application actors.
///
/// `title`:  The title with the updated information.
///
/// # Errors
///
/// See [`db::connect`] for the errors that can arise if opening the database connection fails.
///
/// [`crate::Error::Database`] if the update database operation fails.
pub async fn update_title(bus: &Handle, title: &Title) -> Result<()> {
    let conn = db::connect(&bus).await
        .inspect_err(|error| tracing::error!(?error, "database connection error"))?;

    db::title::update(&conn, title)
        .inspect(|_| tracing::info!(id=title.id, "title updated"))
        .inspect_err(|error| tracing::error!(?error, "database update failed"))
}

