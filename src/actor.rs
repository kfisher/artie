// Copyright 2026 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! Actor types and utilities.

use tokio::sync::mpsc::{self, Receiver, Sender};
use tokio::sync::oneshot;

use crate::{Error, Result};
use crate::task;

/// Default number of requests that can be queued in an actor's channel.
pub const BUFFER_SIZE: usize = 10;

/// Responses to a request sent to an actor.
pub type Response<T> = oneshot::Sender<Result<T>>;

/// Processes actor messages.
pub trait MessageProcessor<Message> {
    /// Processes a request for the actor.
    ///
    /// The process handler is expected to be a quick operation to avoid blocking the task handling
    /// messages for too long. Heavy operations should spawn a separate task and use the response
    /// mechanism to relay the response when complete.
    ///
    /// # Args
    ///
    /// `msg`:  The message containing the request.
    ///
    /// # Errors
    ///
    /// The errors that might be returned will very by the request. Refer to the specific messages
    /// for more information.
    fn process(&mut self, msg: Message) -> impl std::future::Future<Output = Result<()>> + Send;
}

/// Application actor.
///
/// The application makes use of the [actor model](https://en.wikipedia.org/wiki/Actor_model).
/// Actors receive requests, provide a response, and manage their own state. Each actor runs in its
/// own async task. Each actor will have an associated actor ([`Handle`]) used to send messages to
/// the actor.
///
/// Communication between actors is handled using multi-producer, single-consumer queues routed
/// thru the message bus (see: [`crate::bus`]).
pub struct Actor<Message, Processor>
where
    Processor: MessageProcessor<Message>
{
    /// Name of the actor instance.
    ///
    /// The name is used to help distinguish one actor instance from another such as when logging
    /// messages. Therefore, these should ideally be unique.
    name: String,

    /// Receiving end of the channel used to send requests to the actor.
    chan_rx: Receiver<Message>,

    /// Processor used to process messages sent to the actor.
    msg_proc: Processor,
}

impl<Message, Processor> Actor<Message, Processor> 
where
    Processor: MessageProcessor<Message>
{
    /// Create a new actor instance.
    pub fn new(
        name: &str,
        chan_rx: Receiver<Message>,
        msg_proc: Processor
    ) -> Self {
        Self {
            name: name.to_owned(),
            chan_rx,
            msg_proc 
        }
    }
}

/// Handle used to communicate with an actor.
#[derive(Debug)]
pub struct Handle<Message> {
    /// Transmission end of the channel used to send requests to the associated actor.
    chan_tx: Sender<Message>
}

impl<Message> Handle<Message> {
    /// Create a new instance of the handle.
    ///
    /// # Args
    /// 
    /// `chan_tx`:  Transmission end of the channel used to send requests to the associated actor.
    pub fn new(chan_tx: Sender<Message>) -> Self {
        Self { chan_tx }
    }

    /// Send a message to the associated actor.
    ///
    /// # Args
    ///
    /// `msg`:  The message to send.
    ///
    /// # Errors
    ///
    /// [`Error::ChannelSend`] if the message could not be sent.
    pub async fn send<T>(&self, msg: T) -> Result<()> 
    where
        T: Into<Message>,
        Error: From<mpsc::error::SendError<Message>>
    {
        self.chan_tx.send(msg.into()).await?;
        Ok(())
    }
}

impl<Message> Clone for Handle<Message> {
    fn clone(&self) -> Self {
        Self { chan_tx: self.chan_tx.clone() }
    }
}

/// Create an actor and start its message processing task.
///
/// # Args
///                                                      
/// `name`:  The name of the actor instance. Used to help distinguish one actor instance from
/// another such as when logging.
///
/// `msg_processor`:  Processor used to process messages send to the actor.
pub fn create_and_run<Message, Processor>(name: &str, msg_processor: Processor) -> Handle<Message>
where
    Message: std::marker::Send + 'static,
    Processor: MessageProcessor<Message> + std::marker::Send + 'static
{
    let (tx, rx) = mpsc::channel(BUFFER_SIZE);

    let actor = Actor::new(name, rx, msg_processor);
    task::spawn(run(actor));

    Handle::new(tx)
}

/// Task for processing requests for an actor.
///
/// This async task will run until all senders for the message channel is closed or the actor closes
/// the channel's receiver.
///
/// # Args
///
/// `actor`:  The actor instance which will be processing requests.
pub async fn run<Message, Processor>(mut actor: Actor<Message, Processor>) 
where
    Processor: MessageProcessor<Message>
{
    tracing::info!(actor=actor.name, "actor processing started");

    while let Some(msg) = actor.chan_rx.recv().await {
        if let Err(err) = actor.msg_proc.process(msg).await {
            tracing::error!(actor=actor.name, ?err, "failed to process request");
        }
    }

    tracing::info!(actor=actor.name, "actor processing stopped");
}

#[cfg(test)]
mod tests {
    // TODO
}
