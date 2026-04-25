// Copyright 2026 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! Handles network communication with worker node.
//!
//! TODO

pub mod manager;

use std::time::Duration;

use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
use tokio::sync::mpsc;

use crate::Result;
use crate::actor;
use crate::net::{Handle, Message};
use crate::task;

/// Maxium number of messages for a client actor that can be queued.
const CLIENT_CHANNEL_BUFFER_SIZE: usize = 10;

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

// TODO
enum TmpMsg {
}

/// Create the client actor.
///
/// This will create the actor and spawn the tasks for handling communication with the application
/// and handling communication with the worker node.
///
/// # Args
pub fn init(addr: &str) -> Handle {
    // Channel used for the client actor to send messages over the network.
    let (net_tx, net_rx) = mpsc::channel(CLIENT_CHANNEL_BUFFER_SIZE);

    let msg_processor = MessageProcessor::new(net_tx);

    let name = format!("client {}", &addr);
    let handle = actor::create_and_run(&name, msg_processor);

    let addr = addr.to_owned();
    let handle_clone = handle.clone();
    task::spawn(async move {
        connect(addr, handle_clone, net_rx).await
    });

    handle
}

/// Processes messages sent to the client actor.
struct MessageProcessor {
    /// Transmission end of the channel used to send messages to the connected worker node.
    net_tx: mpsc::Sender<TmpMsg>,
}

impl MessageProcessor {
    /// Create a new instance of the message processor.
    ///
    /// # Args
    ///
    /// `net_tx`:  Transmission end of the channel used to send messages to the connected worker.
    /// node.
    fn new(net_tx: mpsc::Sender<TmpMsg>) -> Self {
        Self { net_tx }
    }
}

impl actor::MessageProcessor<Message> for MessageProcessor {
    async fn process(&mut self, _msg: Message) -> Result<()> {
        todo!()
    }
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
async fn connect(addr: String, client: Handle, mut net_rx: mpsc::Receiver<TmpMsg>) {
    let mut attempt: u32 = 0;

    loop {
        match TcpStream::connect(&addr).await {
            Ok(stream) => {
                attempt = 0;

                tracing::info!(?addr, "client connected");
                process_stream(stream, &addr, &client, &mut net_rx).await;
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

/// Process communication with a connected worker node.
///
/// # Args
///
/// This will run until the worker drops the connection or the client actor closes the channel it
/// uses to send messages.
///
/// `stream`:  The stream for the connected worker.
///
/// `addr`:  The address of the connected worker.
///
/// `client`:  Handle to the client actor. Used to forward messages from the connected worker node
/// to the actor for processing.
///
/// `net_rx`:  Receiving end of the channel used by the client actor to send messages to the
/// connected worker node.
async fn process_stream(
    stream: TcpStream,
    addr: &str,
    _client: &Handle,
    net_rx: &mut mpsc::Receiver<TmpMsg>,
) {
    let (reader, mut writer) = stream.into_split();

    let mut reader = BufReader::new(reader);
    let mut line = String::new();

    loop {
        tokio::select! {
            result = reader.read_line(&mut line) => {
                match result {
                    Ok(0) => {
                        tracing::info!(?addr, "connection close by remote");
                        break;
                    },
                    Ok(_) => {
                        // TODO: Parse message and then relay it to the server via the handle.
                        print!("Received: {line}");
                        line.clear();
                    },
                    Err(error) => {
                        tracing::error!(?addr, ?error, "failed to read message");
                        break;
                    },
                }
            }
            msg = net_rx.recv() => {
                match msg {
                    Some(_) => {
                        // TODO: Serialize the message and send to the client.
                        if let Err(error) = writer.write_all("TODO".as_bytes()).await {
                            tracing::error!(?addr, ?error, "failed to send message");
                            break;
                        }
                    },
                    None => {
                        tracing::info!("client channel closed. exiting");
                        break;
                    }
                }
            }

        }
    }
}

