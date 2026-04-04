// Copyright 2026 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! TODO: DOC

use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc;

use crate::Result;
use crate::task;

/// Maxium number of messages for a drive actor that can be queued.
const SERVER_CHANNEL_BUFFER_SIZE: usize = 10;

/// Specifies the messages and responses for the server actor.
pub enum ServerMessage {
}

/// Handle used to communicate with the [`Server`] instance.
pub struct ServerHandle {
    /// Transmission end of the channel used to send requests to the server.
    tx: mpsc::Sender<ServerMessage>,
}

impl ServerHandle {
    /// Create a new [`ServerHandle`] instance.
    pub fn new(tx: mpsc::Sender<ServerMessage>) -> Self {
        Self { tx }
    }
}

/// Create the [`Server`] instance, start its processing task, and return a handle for it.
pub fn create_and_run_server() -> ServerHandle {
    let (tx, rx) = mpsc::channel(SERVER_CHANNEL_BUFFER_SIZE);

    let server = Server::new(tx.clone(), rx);
    task::spawn(run_server(server));

    ServerHandle::new(tx)
}

/// Handles the worker side of network communication.
struct Server {
    /// Transmission end of the channel used to send requests to the server instance.
    ///
    /// This isn't used directly by the server. It serves as a "prototype" and cloned when creating
    /// new handles for the server.
    tx: mpsc::Sender<ServerMessage>,

    /// Receiving end of the channel used to send requests to the server.
    rx: mpsc::Receiver<ServerMessage>,
}

impl Server {
    /// Create a [`Server`] instance.
    ///
    /// `tx`:  Transmission end of the channel used to send requests to the server instance.
    ///
    /// `rx`:  Receiving end of the channel used to send requests to the server.
    fn new(tx: mpsc::Sender<ServerMessage>, rx: mpsc::Receiver<ServerMessage>) -> Self {
        Self { tx, rx }
    }

    /// Create and return a [`ServerHandle`] instance for communicating with this server instance.
    fn create_handle(&self) -> ServerHandle {
        ServerHandle::new(self.tx.clone())
    }

    /// Processes a request for the server.
    ///
    /// `msg`:  Message containing the request for the server.
    fn proc_msg(&mut self, _msg: ServerMessage) -> Result<()> {
        todo!()
    }

    /// Get the next request for the server.
    ///
    /// This will return `None` when the message channel is closed and does not contain any queued
    /// messages. If the message queue is empty, but the channel is not closed, this will sleep
    /// until a message is sent or the channel is closed.
    async fn recv_msg(&mut self) -> Option<ServerMessage> {
        self.rx.recv().await
    }
}

/// Listen for connections from the control node.
///
/// **Note** This will run indefinately until the application exits. Its expected to be run as a 
/// seperate async task.
///
/// `server`:  Handle for the server instance.
///
/// `addr`:  The address and port to listen on.
async fn listen(_server: ServerHandle, addr: &str) {
    let listener = match TcpListener::bind(addr).await {
        Ok(listener) => listener,
        Err(error) => {
            tracing::error!(?error, ?addr, "failed to listen for connections");
            return;
        }
    };

    tracing::info!(?addr, "waiting for connection");
    loop {
        let _stream = match listener.accept().await {
            Ok((stream, peer_addr)) => {
                tracing::info!(?peer_addr, "client connected");
                stream
            },
            Err(error) => {
                tracing::error!(?error, ?addr, "failed to accept connection");
                continue;
            }
        };

        // TODO: What to do here.
    }
}

/// Runs the server.
///
/// This will start the listen task to handle connections from the control node. When a control
/// node connects, it will start handling the communcation.
///
/// After the listen task is started, it will begin processing local requests.
///
/// This will run until the server's message channel is closed.
async fn run_server(mut server: Server) {
    tracing::info!("server started");

    // Start listening for connections from the control node.
    // TODO: Port and maybe the address should be configurable.
    let handle = server.create_handle();
    task::spawn(async move {
        listen(handle, "127.0.0.1:7878").await;
    });

    while let Some(msg) = server.recv_msg().await {
        if let Err(error) = server.proc_msg(msg) {
            tracing::error!(?error, "failed to process server request");
        }
    }

    tracing::info!("server exited");
}

