// Copyright 2025 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! TODO

use tokio::sync::mpsc;
use tokio::sync::oneshot;

use crate::{Error, Result};
use crate::error::ChannelError;

use super::{DiscState, Drive, DriveState, DriveStatus};

/// Specifies the messages and responses for the optical drive actor.
///
/// Each message will have a channel that will be used by the actor to send the response. This will 
/// be a `oneshot` channel for a single response or a `mpsc` channel for streaming data such as
/// operation process information.
#[derive(Debug)]
pub enum DriveActorMessage {
    /// Request the current status of the optical drive.
    GetStatus {
        response: oneshot::Sender<DriveStatus>,
    },
}

// TODO
#[derive(Debug)]
pub enum DriveDirectorMessage {
}

/// Handle used to communicate with a [`DriveActor`] instance.
#[derive(Clone, Debug)]
pub struct DriveActorHandle {
    /// Transmission end of the channel used to send requests to the actor.
    tx: mpsc::Sender<DriveActorMessage>,
}

impl DriveActorHandle {
    /// Create a new [`DriveActorHandle`] instance.
    fn new(tx: mpsc::Sender<DriveActorMessage>) -> Self {
        Self { tx }
    }

    /// Get the current status of the optical drive.
    pub async fn get_status(&self) -> Result<DriveStatus> {
        let (tx, rx) = oneshot::channel();

        let msg = DriveActorMessage::GetStatus { response: tx };

        self.tx.send(msg).await
            .map_err(send_error)?;

        rx.await.map_err(oneshot_recv_error)
    }

    // TODO: Copy Request
    // - Message could have a full channel for sending back progress information
    // - Message would also require the copy perameters passed in via function.
    // - Need to figure out how to integrate with iced.
}

// TODO
#[derive(Clone, Debug)]
pub struct DriveDirectorHandle {
    /// Transmission end of the channel used to send requests to the director.
    tx: mpsc::Sender<DriveDirectorMessage>,
}

impl DriveDirectorHandle {
    /// Create a new [`DriveDirectorHandle`] instance.
    fn new(tx: mpsc::Sender<DriveDirectorMessage>) -> Self {
        Self { tx }
    }
}

/// Create a [`DriveActor`] instance, start it, and return its handle.
pub fn create_actor(serial_number: &str) -> DriveActorHandle {
    DriveActor::create(serial_number.to_owned())
}

pub fn director_worker() {
}

/// Actor used to perform copy operations and monitor the state of an optical 
#[derive(Debug)]
struct DriveActor {
    /// The serial number of the optical drive this actor is associated with.
    serial_number: String,

    /// The current state of the drive.
    state: DriveState,

    /// Receiving end of the channel used to send requests to the actor.
    rx: mpsc::Receiver<DriveActorMessage>,
}

impl DriveActor {
    /// Create a [`DriveActor`] instance.
    fn new(serial_number: String, rx: mpsc::Receiver<DriveActorMessage>) -> Self {
        Self { serial_number, state: DriveState::Idle, rx }
    }

    /// Create a [`DriveActor`] instance, start it, and return its handle.
    fn create(serial_number: String) -> DriveActorHandle {
        let (tx, rx) = mpsc::channel(10);

        let actor = DriveActor::new(serial_number, rx);
        tokio::spawn(run_actor(actor));

        DriveActorHandle::new(tx)
    }

    /// Process a message that was received from the actor's communication channel.
    fn handle_message(&mut self, msg: DriveActorMessage) -> Result<()> {
        match msg {
            DriveActorMessage::GetStatus { response } => self.get_state(response),
        }
    }

    /// Process the request for getting the current state of the actor.
    fn get_state(&self, tx: oneshot::Sender<DriveStatus>) -> Result<()> {
        let status = match super::get_optical_drive(&self.serial_number)? {
            Some(drive) => DriveStatus::new(drive.disc, self.state.clone()),
            None => DriveStatus::new(DiscState::None, DriveState::Disconnected),
        };

        tx.send(status).map_err(|_| Error::DriveActorChannel { error: ChannelError::OneShotSend })?;

        Ok(())
    }
}

// TODO
#[derive(Debug)]
struct DriveDirector {
    /// Receiving end of the channel used to send requests to the director.
    rx: mpsc::Receiver<DriveDirectorMessage>,
}

impl DriveDirector {
    /// Create a [`DriveDirector`] instance.
    fn new(rx: mpsc::Receiver<DriveDirectorMessage>) -> Self {
        Self { rx }
    }

    /// Create a [`DriveDirector`] instance, start it, and return its handle.
    fn create() -> DriveDirectorHandle {
        let (tx, rx) = mpsc::channel(10);

        let director = DriveDirector::new(rx);
        tokio::spawn(run_director(director));

        DriveDirectorHandle::new(tx)
    }

    /// Process a message that was received from the actor's communication channel.
    fn handle_message(&mut self, msg: DriveDirectorMessage) -> Result<()> {
        match msg {
        }
    }
}

fn create_director() -> DriveDirectorHandle {
    todo!()
}

/// Create an application error from the provided tokio error.
fn oneshot_recv_error(e: oneshot::error::RecvError) -> Error {
    Error::DriveActorChannel { error: ChannelError::OneShotRecv(e) }
}

/// Runs the processing loop for the provided actor.
async fn run_actor(mut actor: DriveActor) {
    tracing::info!(sn=actor.serial_number, "message processing started");

    while let Some(msg) = actor.rx.recv().await {
        if let Err(error) = actor.handle_message(msg) {
            tracing::trace!(sn=actor.serial_number, ?error, "Failed to process message.");
        }
    }

    tracing::info!(sn=actor.serial_number, "message processing ended");
}

/// Runs the processing loop for the provided actor.
async fn run_director(mut director: DriveDirector) {
    tracing::info!("message processing started");

    while let Some(msg) = director.rx.recv().await {
        if let Err(error) = director.handle_message(msg) {
            tracing::trace!(?error, "failed to process message");
        }
    }

    tracing::info!("message processing ended");
}

/// Create an application error from the provided tokio error.
fn send_error(e: mpsc::error::SendError<DriveActorMessage>) -> Error {
    Error::DriveActorChannel { error: ChannelError::Send(e) }
}

// TODO: TESTING
