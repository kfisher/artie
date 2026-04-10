// Copyright 2026 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

use tokio::sync::mpsc;

use crate::Result;
use crate::error::Error;
use crate::db::Database;
use crate::drive;
use crate::drive::OpticalDrive;
use crate::drive::actor::handle::DriveActorHandle;
use crate::drive::actor::local;
use crate::drive::actor::worker;
use crate::fs::FileSystem;
use crate::task;

/// Maximum number of requests for the manager that can be queued.
const CHANNEL_BUFFER_SIZE: usize = 10;

/// Handle for interfacing with the drive actor manager.
pub struct DriveActorManagerHandle {
    /// Transmission end of the channel used to send requests to the drive actor handle.
    tx: mpsc::Sender<Message>,
}

impl DriveActorManagerHandle {
    /// Create a new instance of the drive actor manager handle.
    ///
    /// # Args
    ///
    /// `tx`:  Transmission end of the channel used to send requests to the drive actor handle.
    fn new(tx: mpsc::Sender<Message>) -> Self {
        Self { tx }
    }
}

/// Creates an instance of the drive manager for the control node and returns a handle for
/// communicating with the manager.
///
/// When the actor manager is created, a new task will be spawned to begin processing requests from
/// the application.
///
/// This factory method will initialize with a list of local drive actors for the drives connected
/// to the machine the application is running on. Besides that, there is no difference between the
/// factory methods.
///
/// # Args
///
/// `fs`:  Information about where where relevent files can be found or should be written to when
///        performing copy operation (e.g. where to copy titles to).
///
/// `db`:  Interface for reading and writing to the database.
///
/// # Errors
///
/// The only errors that can be raised are ones that happen when running the OS command to get
/// information about the system's optical drives.
pub fn create_control(fs: FileSystem, db: Database) -> Result<DriveActorManagerHandle> {
    create(|drive| local::create_actor(drive, fs.clone(), db.clone()))
}

/// Create an instance of the drive manager for a worker node and returns a handle for
/// communicating with the manager.
///
/// When the actor manager is created, a new task will be spawned to begin processing requests from
/// the application.
///
/// This factory method will initialize with a list of worker drive actors for the drives connected
/// to the machine the application is running on. Besides that, there is no difference between the
/// factory methods.
///
/// # Errors
///
/// The only errors that can be raised are ones that happen when running the OS command to get
/// information about the system's optical drives.
pub fn create_worker() -> Result<DriveActorManagerHandle> {
    create(|drive| worker::create_worker_actor(drive))
}

/// Messages for communicating with the drive actor manager.
enum Message {
}

/// Actor responsible for managing the drive actor instances.
struct DriveActorManager {
    /// List of handles for communicating with the drive actors.
    ///
    /// These handles can be associated with any of the drive actor types. They also do not all
    /// have to be associated with the same type.
    actors: Vec<DriveActorHandle>,

    /// Transmission end of the channel used to send requests to the drive actor manager.
    ///
    /// This isn't used directly. It serves as a "prototype" and cloned when creating new handles.
    tx: mpsc::Sender<Message>,

    /// Receiving end of the channel used to send requests to the client.
    ///
    /// This channel is used by the rest of the application to communicate with the client actor.
    rx: mpsc::Receiver<Message>,
}

impl DriveActorManager {
    /// Creates a new manager instance.
    ///
    /// # Args
    ///
    /// `actors`:  List of handles for communicating with the drive actors. The handles can be
    /// associated with any of the drive actor types. Also, they do not all have to be associated
    /// with the same type.
    fn new(actors: Vec<DriveActorHandle>,
        tx: mpsc::Sender<Message>,
        rx: mpsc::Receiver<Message>,
    ) -> DriveActorManager {
        Self { actors, tx, rx }
    }

    /// Creates a handle for communicating with the drive actor manager.
    fn create_handle(&self) -> DriveActorManagerHandle {
        DriveActorManagerHandle::new(self.tx.clone())
    }

    /// Process a request from the application.
    ///
    /// # Args
    ///
    /// `msg`:  Message containing the request.
    ///
    /// # Errors
    ///
    /// Will return an error if the manager is unable to process the request. The specific error
    /// will depend on the request type.
    fn proc_msg(&mut self, _msg: Message) -> Result<()> {
        todo!()
    }

    /// Get the next request in the queue.
    ///
    /// This will return `None` when the message channel is closed and does not contain any queued
    /// messages. If the message queue is empty, but the channel is not closed, this will sleep
    /// until a message is sent or the channel is closed.
    async fn recv_msg(&mut self) -> Option<Message> {
        todo!()
    }
}

/// Create an instance of the drive manager for a worker node and returns a handle for
/// communicating with the manager.
///
/// When the actor manager is created, a new task will be spawned to begin processing requests from
/// the application.
///
/// # Args
///
/// `create_actor`:  Function that should be used to create drive actors for the machines
///                  discovered on the local machine.
///
/// # Errors
///
/// The only errors that can be raised are ones that happen when running the OS command to get
/// information about the system's optical drives.
fn create<T>(create_actor: T) -> Result<DriveActorManagerHandle>
where 
    T: Fn(&OpticalDrive) -> DriveActorHandle
{
    let actors = drive::get_optical_drives()?
        .iter()
        .map(create_actor)
        .collect();

    // Channel used for the rest of the application to communicate with the drive actor manager.
    let (tx, rx) = mpsc::channel(CHANNEL_BUFFER_SIZE);

    let mgr = DriveActorManager::new(actors, tx.clone(), rx);
    task::spawn(run(mgr));

    Ok(DriveActorManagerHandle::new(tx))
}

/// Task for processing requests for the drive actor manager.
///
/// This async task will run until the message channel used for sending requests is closed.
///
/// # Args
///
/// `mgr`:  The drive actor manager instance which will be processing requests.
async fn run(mut mgr: DriveActorManager) {
    tracing::info!("drive actor manager started");

    while let Some(msg) = mgr.recv_msg().await {
        if let Err(error) = mgr.proc_msg(msg) {
            tracing::error!(?error, "failed to process drive actor manager request");
        }
    }

    tracing::info!("drive actor manager exited");
}
