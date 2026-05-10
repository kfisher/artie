// Copyright 2026 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! Handles network communication with worker node.
//!
//! The client can be initialized by calling [`init`]. This will start the task used to process
//! requests for the actor (for sending messages to the worker node) and the task for connecting to
//! and processing requests from the worker node. The connection task will automatically handle
//! retrying connects if the connection fails or the network connection is broken. This will be
//! called by the client manager actor during application startup for each configured node or when
//! a new node is added to the configuration.
//!
//! Messages can be sent to the control node by using one of the helper methods in the
//! [`crate::net`] module.

pub mod manager;

use std::time::Duration;

use tokio::net::TcpStream;
use tokio::sync::mpsc;

use crate::bus;
use crate::net::{self, Handle, OutgoingMessage};
use crate::task;

/// Initial amount of time to wait before attempting to connect to a worker node after a failed
/// attempt or disconnection.
const BASE_DELAY: Duration = Duration::from_secs(1);

/// The maximum amount of time to wait before attempting to connect to a worker node after a failed
/// attempt or disconnection.
const MAX_DELAY: Duration = Duration::from_secs(60);

/// The number of attempts before the max delay is reached.
///
/// This value is the attempt number where the delay calculation would result in exceeding the
/// maximum delay. `2^6=64` which is greater than the [`MAX_DELAY`] of 60 seconds.
const MAX_BACKOFF_COUNT: u32 = 6;

/// Create the client actor.
///
/// This will create the actor and spawn the tasks for handling communication with the application
/// and handling communication with the worker node.
///
/// # Args
///
/// `bus`:  Handle used to send messages to other actors via the message bus.
///
/// `addr`:  The address to connect to.
pub fn init(bus: &bus::Handle, addr: &str) -> Handle {
    let name = format!("client {}", &addr);
    let (handle, net_rx) = net::actor::init(&name, bus);

    let addr = addr.to_owned();
    let handle_clone = handle.clone();
    task::spawn(async move {
        connect(addr, handle_clone, net_rx).await
    });

    handle
}

/// Connect to the worker node.
///
/// # Args
///
/// `addr`:  The address of the node to connect to.
///
/// `client`:  Handle for the client instance.
///
/// `net_rx`:  Receiving end of the channel used by the server actor to send messages to the
/// connected client.
async fn connect(addr: String, client: Handle, mut net_rx: mpsc::Receiver<OutgoingMessage>) {
    let mut attempt: u32 = 0;

    loop {
        match TcpStream::connect(&addr).await {
            Ok(stream) => {
                attempt = 0;

                tracing::info!(?addr, "client connected");
                net::process_stream(stream, &addr, &client, &mut net_rx).await;
                tracing::warn!(?addr, "connection lost, will attempt to reconnect");
            }
            Err(error) => {
                tracing::error!(?error, ?addr, attempt, "failed to connect");
            }
        }

        attempt = attempt.saturating_add(1);
        let delay = if attempt >= MAX_BACKOFF_COUNT {
            MAX_DELAY
        } else {
            BASE_DELAY * 2u32.saturating_pow(attempt)
        };

        tracing::trace!(?addr, attempt, ?delay, "reconnecting after delay");
        tokio::time::sleep(delay).await;
    }
}


#[cfg(test)]
mod tests {
    // TODO[TESTS]
}
