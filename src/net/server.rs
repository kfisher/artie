// Copyright 2026 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! Runs a server for a worker node.
//!
//! TODO

use tokio::net::TcpListener;
use tokio::sync::mpsc;

use crate::Error;
use crate::bus;
use crate::net::{self, Handle, OutgoingMessage};
use crate::task;

/// Create the the server actor.
///
/// This will create the actor, spawn the task for processing requests from the application and
/// spawn the task for listening for and processing messages from the control node,
///
/// # Args
///
/// `bus`:  Handle used to send messages to other actors via the message bus.
pub fn init(bus: &bus::Handle) -> Handle {
    let (handle, net_rx) = net::actor::init("server", bus);

    // TODO: Listen address and port should be configurable.
    let handle_clone = handle.clone();
    task::spawn(async move {
        listen("127.0.0.1:7878", handle_clone, net_rx).await;
    });

    handle
}

/// Listen for connections from the control node.
///
/// # Args
///
/// `addr`:  The address and port to listen on.
///
/// `server`:  Handle used to send messages to the server actor.
///
/// `net_rx`:  Receiving end of the channel used by the server message processor to send messages
/// to the connected control node.
async fn listen(addr: &str, server: Handle, mut net_rx: mpsc::Receiver<OutgoingMessage>) {
    let listener = match TcpListener::bind(addr).await {
        Ok(listener) => listener,
        Err(error) => {
            tracing::error!(?error, ?addr, "failed to listen for connections");
            return;
        }
    };

    tracing::info!(?addr, "waiting for connection");
    loop {
        tokio::select! {
            result = listener.accept() => {
                match result {
                    Ok((stream, peer_addr)) => {
                        // We only support a single connected client at a time since there should
                        // only ever be one control node.
                        tracing::info!(?peer_addr, "client connected");
                        let peer_addr = peer_addr.to_string();
                        net::process_stream(stream, &peer_addr, &server, &mut net_rx).await;
                        tracing::info!("client disconnected");
                    },
                    Err(error) => {
                        tracing::error!(?error, ?addr, "failed to accept connection");
                    }
                }
            }
            result = net_rx.recv() => {
                match result {
                    Some(msg) => {
                        tracing::trace!("attempted to send message when disconnected");
                        let _ = msg.response.send(Err(Error::Disconnected))
                            .inspect_err(|_| tracing::error!("failed to send response"));
                    },
                    None => {
                        tracing::info!("server channel closed. exiting");
                        break;
                    }
                }
            }
        }
    }
}

