// Copyright 2026 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! Actor for managing client connections to worker nodes.
//!
//! TODO

use crate::Result;
use crate::actor;
use crate::bus;
use crate::net::{Handle, Message, Settings};
use crate::client;

/// Create the client manager actor.
///
/// This will create the actor and spawn the task for processing requests. It will also initialize
/// the client for each of the configured worker nodes.
///
/// # Args
///
/// `bus`:  Handle used to send messages to other actors via the message bus.
/// 
/// `settings`:  Settings containing the list of worker nodes to connect to.
pub fn init(bus: &bus::Handle, settings: &Settings) -> Handle {
    let msg_processor = MessageProcessor::new(bus.clone(), &settings.workers);
    let handle = actor::create_and_run("client manager", msg_processor);
    handle
}

/// Handle for interfacing with a client actor.
///
/// This is basically a wrapper around the standard actor handle, but with additional metadata such
/// as the client address.
struct ClientHandle {
    /// The network address of the worker node the client is connected to.
    addr: String,

    /// The underlying actor handle.
    actor: Handle,
}

/// Processes messages sent to the client manager.
///
/// The client manager actor may receive messages for itself or one of the clients. Any message for
/// a client, the message will be forwarded to that client.
struct MessageProcessor {
    /// Handle used to send messages to other actors via the message bus.
    bus: bus::Handle,

    /// List of handles for all available clients.
    /// 
    /// The client actors will remain active even if they cannot connect to their associated worker
    /// nodes. They automatically handle reconnect attemps and any request sent to them while
    /// disconnect will fail.
    clients: Vec<ClientHandle>,
}

impl MessageProcessor {
    /// Create a new instance of the message processor.
    /// 
    /// # Args
    ///
    /// `bus`:  Handle used to send messages to other actors via the message bus.
    ///
    /// `clients`:  List of network addresses for the worker nodes to create clients for.
    fn new(bus: bus::Handle, clients: &[String]) -> Self {
        let clients = clients.iter()
            .map(|addr| create_client(&bus, addr))
            .collect();
        Self { bus, clients }
    }
}

impl actor::MessageProcessor<Message> for MessageProcessor {
    async fn process(&mut self, _msg: Message) -> Result<()> {
        todo!()
    }
}

/// Create a client actor.
///
/// This will create and initialize the client actor which will start tasks for processing requests
/// from the application and sending/receiving messages to/from the associated worker node.
///
/// # Args
///
/// `bus`:  Handle used to send messages to other actors via the message bus.
///
/// `addr`:  The address of the client.
fn create_client(bus: &bus::Handle, addr: &String) -> ClientHandle {
    ClientHandle {
        addr: addr.to_owned(),
        actor: client::init(bus, addr),
    }
}
