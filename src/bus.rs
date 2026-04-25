// Copyright 2026 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! Application messaging bus.

use tokio::sync::mpsc::{self, Receiver};
use tokio::task::JoinHandle;

use crate::Result;
use crate::actor::{self, Actor};
use crate::app;
use crate::db;
use crate::drive;
use crate::task;
use crate::ui;

/// Handle used to communicate with the message bus.
pub type Handle = actor::Handle<Message>;

/// Specifies the types of messages that can be sent on the message bus.
#[derive(Debug)]
pub enum Message {
    /// General application messages.
    App(app::Message),

    /// Messages for sending requests to the database actor.
    Database(db::Message),

    /// Messages for sending requests to a drive actor.
    Drive(drive::Message),

    /// Messages for sending requests to the message bus itself.
    MessageBus(MessageBusMessage),

    /// Messages for sending requests to the UI.
    UI(ui::Message),
}

impl From<db::Message> for Message {
    fn from(value: db::Message) -> Self {
        Message::Database(value)
    }
}

impl From<drive::Message> for Message {
    fn from(value: drive::Message) -> Self {
        Message::Drive(value)
    }
}

/// Messages used to send requests to the message bus itself.
#[derive(Debug)]
pub enum MessageBusMessage {
}

/// Create's the channel used to send messages to the message bus.
///
/// This will return both the transmission (as a handle) and receiving end of the channel. The
/// receiving end should be passed to [`init_processor`] when the message processing is started.
pub fn init_channel() -> (Handle, Receiver<Message>) {
    let (tx, rx) = mpsc::channel(actor::BUFFER_SIZE);
    (Handle::new(tx), rx)
}

/// Create's the message processor for the message bus and start its processing task so it can
/// begin handling requests.
///
/// # Args
///
/// `db`:  Handle used to send messages to the database actor.
///
/// `drive_mgr`:  Handle used to send messages to the drive manager actor and drive actors.
///
/// `bus_send`:  The transmission end of the message bus communication channel.
pub fn init_processor(
    db: db::Handle,
    drive_mgr: drive::Handle,
    bus_recv: Receiver<Message>,
) -> JoinHandle<()> {
    let msg_processor = MessageBus::new(db, drive_mgr);
    let actor = Actor::new("message bus", bus_recv, msg_processor);

    // Unlike other actors, return the JoinHandle so that headless mode (no GUI) has something to
    // wait on an not exit immediately.
    task::spawn(actor::run(actor))
}

/// Creates the message bus and spawn its processing task so it can begin handling requests.
///
/// The returned handle can be used to send requests to the bus. The message bus will run its
/// processing task until the application exits.
///
/// # Args
///
/// `db`:  Handle used to send messages to the database actor.
///
/// `drive_mgr`:  Handle used to send messages to the drive manager actor and drive actors.
pub fn init(db: db::Handle, drive_mgr: drive::Handle) -> Handle {
    let msg_processor = MessageBus::new(db, drive_mgr);
    actor::create_and_run("message bus", msg_processor)
}

/// Messaging bus used for inner-application communication.
///
/// The application is made up of multiple actors, each with their own responsibility. To avoid
/// having to pass around a lot of state or handles to the different actors, the bus is used for
/// one actor to make a request of another. For example, if the UI needs an updated status of a
/// drive, it can send a request to the actor responsible for interfacing with that drive which
/// would then send the status as its response.
struct MessageBus {
    /// Handle used to send messages to the database actor..
    db: db::Handle,

    /// Handle used to send messages to the drive manager actor and drive actors.
    drive_mgr: drive::Handle,
}

impl MessageBus {
    /// Creates a new instance of the message bus.
    ///
    /// # Args
    ///
    /// `db`:  Handle used to send messages to the database actor. All [`Message::Database`]
    /// messages will be forwarded to this handle.
    ///
    /// `drive_mgr`:  Handle used to send messages to the drive manager actor. All [`Message::Drive`]
    /// messages will be forwarded to this handle.
    fn new(db: db::Handle, drive_mgr: drive::Handle) -> Self {
        Self { db, drive_mgr }
    }
}

impl actor::MessageProcessor<Message> for MessageBus {
    async fn process(&mut self, msg: Message) -> Result<()> {
        match msg {
            Message::App(_) => Ok(()),
            Message::Database(msg) => {
                self.db.send(msg).await
            },
            Message::Drive(msg) => {
                self.drive_mgr.send(msg).await
            },
            Message::MessageBus(_) => Ok(()),
            Message::UI(_) => Ok(()),
        }
    }
}

#[cfg(test)]
mod tests {
    // TODO
}
