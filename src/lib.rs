// Copyright (c) 2016 creato
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

#![warn(missing_docs)]

//! A basic publish/subscribe channel.
//!
//! # Usage
//!
//! Add to crate dependencies:
//! 
//! ```toml
//! [dependencies]
//! pub-sub = "*"
//! ```
//! Import in crate root:
//!
//! ```
//! extern crate pub_sub;
//! ```
//!
//! # Example
//!
//! ```
//! extern crate pub_sub;
//! extern crate uuid;
//! 
//! use std::thread;
//! use uuid::Uuid;
//! 
//! fn main() {
//!     let (send, recv) = pub_sub::new();
//!     // send: pub_sub::Sender<Uuid>
//!     // recv: pub_sub::Receiver<Uuid>
//! 
//!     for _ in 0..16 {
//!         let recv = recv.clone();
//! 
//!         thread::spawn(move || {
//!             while let Ok(msg) = recv.recv() {
//!                 println!("recevied {}", msg);
//!             }
//!         });
//!     }
//! 
//!     for _ in 0..16 {
//!         let send = send.clone();
//! 
//!         thread::spawn(move || {
//!             let msg_id = Uuid::new_v4();
//!             println!("    sent {}", msg_id);
//!             send.send(msg_id);
//!         });
//!     }
//! }
//! ```

#[macro_use]
extern crate log;
extern crate uuid;

use std::sync::{mpsc, Arc, Mutex};
use std::collections::HashMap;


/// Sending component of a pub/sub channel.
#[derive(Clone)]
pub struct Sender<T: Clone> {
    senders: Arc<Mutex<HashMap<uuid::Uuid, mpsc::Sender<T>>>>,
}

/// Receiver component of a pub/sub channel.
pub struct Receiver<T: Clone> {
    receiver: mpsc::Receiver<T>,
    senders: Arc<Mutex<HashMap<uuid::Uuid, mpsc::Sender<T>>>>,
    id: uuid::Uuid,
}

impl<T: Clone> Sender<T> {
    /// Attempts to broadcast
    pub fn send(&self, it: T) -> Result<(), mpsc::SendError<T>> {
        let senders = self.senders.lock().unwrap();

        for (_, sender) in senders.iter() {
            match sender.send(it.clone()) {
                Ok(_) => {}
                Err(err) => return Err(err),
            }
        }

        Ok(())
    }
}

impl<T: Clone> Receiver<T> {
    /// Receives a single message. Blocks until a message is available.
    pub fn recv(&self) -> Result<T, mpsc::RecvError> {
        self.receiver.recv()
    }

    /// Tries to receive a single message, not blocking if one is not available.
    pub fn try_recv(&self) -> Result<T, mpsc::TryRecvError> {
        self.receiver.try_recv()
    }

    /// Creates an iterator that will block waiting for messages.
    pub fn iter(&self) -> mpsc::Iter<T> {
        self.receiver.iter()
    }
}


impl<T: Clone> Clone for Receiver<T> {
    /// Create a new receiver associated with the sender.
    fn clone(&self) -> Self {
        let id = uuid::Uuid::new_v4();
        let (send, recv) = mpsc::channel();

        {
            let mut senders = self.senders.lock().unwrap();
            senders.insert(id, send);
        }

        Receiver {
            receiver: recv,
            senders: self.senders.clone(),
            id: id,
        }
    }
}

impl<T: Clone> Drop for Receiver<T> {
    /// Remove our sender ID from the sender list.
    fn drop(&mut self) {
        let mut senders = self.senders.lock().unwrap();
        senders.remove(&self.id);
    }
}

/// Create a pub/sub channel
pub fn new<T: Clone>() -> (Sender<T>, Receiver<T>) {
    let mut senders = HashMap::new();

    let initial_id = uuid::Uuid::new_v4();
    let (send, recv) = mpsc::channel();

    senders.insert(initial_id, send);

    let senders = Arc::new(Mutex::new(senders));

    (Sender { senders: senders.clone() },
     Receiver {
        senders: senders.clone(),
        id: initial_id,
        receiver: recv,
    })
}

#[cfg(test)]
extern crate env_logger;

#[cfg(test)]
mod tests {
    use std;

    use super::*;

    fn pre() {
        use env_logger;
        env_logger::init().unwrap();
    }

    #[test]
    fn many_senders() {
        use std::sync::atomic::{AtomicUsize, Ordering};
        pre();

        let (send, recv) = new();

        let threads = 5;
        let pulses = 50;

        let received = std::sync::Arc::new(AtomicUsize::new(0));

        for _ in 0..threads {
            let recv = recv.clone();
            let received = received.clone();
            std::thread::spawn(move || {
                while let Ok(_) = recv.recv() {
                    received.fetch_add(1, Ordering::AcqRel);
                }
            });
        }


        let mut accum = 0;

        for _ in 0..pulses {
            accum += 1;
            debug!("pulse {}", accum);
            send.send(accum).unwrap();
        }

        std::thread::sleep(std::time::Duration::from_millis(75));
        assert_eq!(received.load(Ordering::Acquire), threads * pulses);
    }
}
