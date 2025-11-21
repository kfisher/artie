// Copyright 2025 Kevin Fisher. All rights reserved.
// SPDX-License-Identifier: GPL-3.0-only

//! TODO: DOC

//-]use tokio::sync::mpsc::{Receiver, Sender};
//-]
//-]
//-]pub enum Message {
//-]}
//-]
//-]
//-]/// Actor used for performing copy operation.
//-]///
//-]/// Each actor will be associated with a single optical drive and each configured optical drive
//-]/// will be associated with a single actor instance.
//-]pub struct Actor {
//-]    /// Receiver for receiving requests from the application.
//-]    receiver: Receiver<Message>,
//-]}
//-]
//-]pub struct Builder {
//-]    /// The serial number of the drive.
//-]    serial_number: String,
//-]}
//-]
//-]pub struct Handle {
//-]    /// Sender for sending requests to the actor.
//-]    sender: Sender<Message>,
//-]}
//-]
//-]impl Handle {
//-]}

// impl Builder {
//     /// Create a [`Builder`] instance.
//     pub fn new(serial_number: &str) -> Self {
//         Self {
//             serial_number: serial_number.to_owned(),
//         }
//     }
// 
//     // TODO: DOC
//     pub build(self) -> Handle {
//     }
// }
