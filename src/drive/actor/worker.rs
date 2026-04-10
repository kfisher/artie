// Copyright 2026 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

use tokio::sync::mpsc;
use tokio::sync::oneshot;

use crate::Result;
use crate::db::Database;
use crate::drive::OpticalDrive;
use crate::drive::actor;
use crate::drive::actor::{DriveActor, DriveActorMessage, OpticalDriveState, OpticalDriveStatus};
use crate::drive::actor::handle::DriveActorHandle;
use crate::fs::FileSystem;
use crate::task;

/// Create a [`ControlActor`] instance, start it, and return a [`DriveActorHandle`] instance for
/// interfacing with it.
///
/// `drive`:  The information for the optical drive that this actor is associated with.
pub fn create_control_actor(drive: &OpticalDrive) -> DriveActorHandle {
    let (tx, rx) = mpsc::channel(actor::ACTOR_CHANNEL_BUFFER_SIZE);

    let actor = ControlActor::new(drive.clone(), tx.clone(), rx);
    task::spawn(actor::run_actor(actor));

    DriveActorHandle::new(tx)
}

/// Create a [`WorkerActor`] instance, start it, and return a [`DriveActorHandle`] instance for
/// interfacing with it.
///
/// `drive`:  The information for the optical drive that this actor is associated with.
pub fn create_worker_actor(drive: &OpticalDrive) -> DriveActorHandle {
    let (tx, rx) = mpsc::channel(actor::ACTOR_CHANNEL_BUFFER_SIZE);

    let actor = ControlActor::new(drive.clone(), tx.clone(), rx);
    task::spawn(actor::run_actor(actor));

    DriveActorHandle::new(tx)
}

/// Actor used to perform copy operations and monitor the state of an optical drive on a worker
/// node instance (control side).
struct ControlActor {
    /// The optical drive this actor is associated with.
    ///
    /// Each actor instance is associated with a single drive. In the case of this actor type, that
    /// drive is attached to the host where the [`WorkerActor`] instance is attached to that this
    /// actor is paired with.
    ///
    /// Additionally, a drive should not have more than one actor instance where a [`ControlActor`]
    /// /[`WorkerActor`] pair is considered a single instance.
    drive: OpticalDrive,

    /// The current state of the drive.
    state: OpticalDriveState,

    /// Transmission end of the channel used to send requests to the actor.
    ///
    /// This isn't used directly by the actor. It is cloned when creating new handle instance from
    /// the actor.
    tx: mpsc::Sender<DriveActorMessage>,

    /// Receiving end of the channel used to send requests to the actor.
    rx: mpsc::Receiver<DriveActorMessage>,
}

impl ControlActor {
    /// Creates a new [`ControlActor`] instance.
    ///
    /// `drive`:  The information for the optical drive that this actor is associated with.
    ///
    /// `tx`:  Transmission end of the channel used to send requests to the actor. This is used as
    /// a 'prototype' instance to create copied when creating new handles for the actor.
    ///
    /// `rx`:  Receiving end of the channel used to send requests to the actor.
    fn new(
        drive: OpticalDrive,
        tx: mpsc::Sender<DriveActorMessage>,
        rx: mpsc::Receiver<DriveActorMessage>,
    ) -> Self {
        Self { drive, state: OpticalDriveState::Disconnected, tx, rx }
    }

    // TODO
    fn get_state(&self, _response: oneshot::Sender<OpticalDriveStatus>) -> Result<()> {

        // - If worker not connected, return disconnected state.
        // - Send request to worker.
        // - Wait for response.
        // - Return state.

        todo!()
    }
}

impl DriveActor for ControlActor {
    fn proc_msg(&mut self, msg: DriveActorMessage) -> Result<()> {
        match msg {
            DriveActorMessage::GetStatus { response } => self.get_state(response),
            _ => todo!()
        }
    }

    fn serial_number(&self) -> &str {
        &self.drive.serial_number
    }

    async fn recv_msg(&mut self) -> Option<DriveActorMessage> {
        self.rx.recv().await
    }
}

/// Actor used to perform copy operations and monitor the state of an optical drive on a worker
/// node instance (worker side).
struct WorkerActor {
    /// The optical drive this actor is associated with.
    ///
    /// Each actor instance is associated with a single drive. In the case of this actor type, that
    /// drive is attached to the host where this instance is running Additionally, a drive should
    /// not have more than one actor instance where a [`ControlActor`] / [`WorkerActor`] pair is
    /// considered a single instance.
    drive: OpticalDrive,

    /// Transmission end of the channel used to send requests to the actor.
    ///
    /// This isn't used directly by the actor. It is cloned when creating new handle instance from
    /// the actor.
    tx: mpsc::Sender<DriveActorMessage>,

    /// Receiving end of the channel used to send requests to the actor.
    rx: mpsc::Receiver<DriveActorMessage>,
}

impl WorkerActor {
    /// Creates a new [`WorkerActor`] instance.
    ///
    /// `drive`:  The information for the optical drive that this actor is associated with.
    ///
    /// `tx`:  Transmission end of the channel used to send requests to the actor. This is used as
    /// a 'prototype' instance to create copied when creating new handles for the actor.
    ///
    /// `rx`:  Receiving end of the channel used to send requests to the actor.
    fn new(
        drive: OpticalDrive,
        tx: mpsc::Sender<DriveActorMessage>,
        rx: mpsc::Receiver<DriveActorMessage>,
    ) -> Self {
        Self { drive, tx, rx }
    }
}

impl DriveActor for WorkerActor {
    fn proc_msg(&mut self, msg: DriveActorMessage) -> Result<()> {
        match msg {
            _ => todo!()
        }
    }

    fn serial_number(&self) -> &str {
        &self.drive.serial_number
    }

    async fn recv_msg(&mut self) -> Option<DriveActorMessage> {
        self.rx.recv().await
    }
}
