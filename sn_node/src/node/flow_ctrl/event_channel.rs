// Copyright 2022 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::node::Event;
use tokio::sync::mpsc;

pub(crate) fn new(channel_size: usize) -> (EventSender, EventReceiver) {
    let (event_sender, event_receiver) = mpsc::channel(channel_size);
    (
        EventSender { event_sender },
        EventReceiver { event_receiver },
    )
}

/// Sender for [`crate::node::Node`] events
#[derive(Clone)]
pub(crate) struct EventSender {
    event_sender: mpsc::Sender<Event>,
}

impl EventSender {
    /// Sends the event.
    /// Currently ignoring if receiver end is closed or dropped.
    pub(crate) async fn send(&self, event: Event) {
        if self.event_sender.send(event).await.is_err() {
            error!("Event receiver has been closed");
        }
    }
}

/// Receiver of events
#[allow(missing_debug_implementations)]
pub struct EventReceiver {
    event_receiver: mpsc::Receiver<Event>,
}

impl EventReceiver {
    /// Waits for, and then returns next event
    pub async fn next(&mut self) -> Option<Event> {
        self.event_receiver.recv().await
    }

    /// Returns next event if any, else None
    pub fn try_next(&mut self) -> Option<Event> {
        if let Ok(event) = self.event_receiver.try_recv() {
            Some(event)
        } else {
            None
        }
    }
}
