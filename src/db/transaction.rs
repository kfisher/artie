// Copyright 2026 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! Utility functions for starting and committing database transactions.
//!
//! Transactions can be used anywhere a [`Connection`] parameter can be used. Transactions allow
//! multiple database operations to be performed, but not saved until committed. If not committed,
//! the operations will be be rolled back when the transaction object is dropped.

use rusqlite::{Connection, Transaction};

use crate::Result;

/// Starts a new database transaction.
///
/// # Args
///
/// `conn`:  The database connection.
///
/// # Errors
///
/// [`crate::Error::Database`] if there is an issue creating the transaction.
pub fn start(conn: &mut Connection) -> Result<Transaction<'_>> {
    conn.transaction().map_err(|e| e.into())
}

/// Commits the transaction.
///
/// # Args
///
/// `conn`:  The database connection.
///
/// # Errors
///
/// [`crate::Error::Database`] if there is an issue creating the transaction.
pub fn commit(transaction: Transaction) -> Result<()> {
    transaction.commit().map_err(|e| e.into())
}
