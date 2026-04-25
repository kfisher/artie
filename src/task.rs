// Copyright 2026 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! Utilities for running asynchronous tasks.
//!
//! These functions essentially wrap a number of the tokio task functions to ensure they are called
//! with the tokio runtime. The `tokio::main` macro cannot be used because it interferes with the
//! GTK runtime.

use std::sync::OnceLock;

use tokio::runtime::Runtime;
use tokio::task::JoinHandle;

/// Spawns a new asynchronous task returning the join handle for it.
///
/// This is essentially just a drop-in for the tokio::spawn method which can't be used because
/// the runtime is manually setup instead of using `tokio::main` macro.
pub fn spawn<F>(future: F) -> JoinHandle<F::Output>
where
    F: Future + Send + 'static,
    F::Output: Send + 'static,
{
    runtime().spawn(future)
}

/// Runs the provided closure on a thread where blocking is acceptable.
///
/// This is essentially just a drop-in for the `tokio::spawn_blocking` method which can't be
/// used because the runtime is manually setup instead of using `tokio::main` macro.
pub fn spawn_blocking<F, R>(func: F) -> JoinHandle<R>
where
    F: FnOnce() -> R + Send + 'static,
    R: Send + 'static,
{
    runtime().spawn_blocking(func)
}

/// Runs a future blocking until it completes.
pub fn block_on<F>(future: F) -> F::Output
where
    F: Future,
{
    Runtime::new().expect("Failed to create blocking runtime").block_on(future)
}

/// Gets the tokio runtime.
///
/// On the first call, the runtime will be initialized.
fn runtime() -> &'static Runtime {
    static RUNTIME: OnceLock<Runtime> = OnceLock::new();
    RUNTIME.get_or_init(|| {
        Runtime::new().expect("Failed to init runtime")
    })
}
