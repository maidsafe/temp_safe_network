// Copyright 2022 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::log_sleep;
use crate::node::{
    flow_ctrl::{
        cmds::{Cmd, CmdJob},
        dispatcher::Dispatcher,
        event_channel::EventSender,
    },
    CmdProcessEvent, Error, Event, RateLimits, Result,
};

use custom_debug::Debug;
use priority_queue::PriorityQueue;
use std::time::SystemTime;
use std::{
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
    time::Duration,
};
use tokio::{sync::RwLock, time::Instant};

type Priority = i32;

const SLEEP_TIME: Duration = Duration::from_millis(10);

const ORDER: Ordering = Ordering::SeqCst;

/// A module for enhanced flow control.
///
/// Orders the incoming cmds (work) according to their priority,
/// and executes them in a controlled way, taking into
/// account the rate limits of our system load monitoring.
///
/// Stacking up direct method calls in the async runtime is sort of
/// like saying to a node "do everything everyone is asking of you, now".
/// We're now saying to a node "do as much as you can without choking,
/// and start with the most important things first".
pub(crate) struct CmdCtrl {
    cmd_queue: PriorityQueue<CmdJob, Priority>,
    attempted: CmdThroughput,
    monitoring: RateLimits,
    stopped: bool,
    pub(crate) dispatcher: Arc<Dispatcher>,
    id_counter: Arc<AtomicUsize>,
    event_sender: EventSender,
}

impl CmdCtrl {
    pub(crate) fn new(
        dispatcher: Dispatcher,
        monitoring: RateLimits,
        event_sender: EventSender,
    ) -> Self {
        Self {
            cmd_queue: PriorityQueue::new(),
            attempted: CmdThroughput::default(),
            monitoring,
            stopped: false,
            dispatcher: Arc::new(dispatcher),
            id_counter: Arc::new(AtomicUsize::new(0)),
            event_sender,
        }

        // let session_clone = session.clone();
        // let _ = tokio::task::spawn_local(async move { session_clone.keep_processing().await });

        // session
    }

    pub(crate) fn node(&self) -> Arc<RwLock<crate::node::Node>> {
        self.dispatcher.node()
    }

    pub(crate) async fn push(&mut self, cmd: Cmd) -> Result<()> {
        self.push_internal(cmd, None).await
    }
    pub(crate) fn q_len(&self) -> usize {
        self.cmd_queue.len()
    }

    // consume self
    // NB that clones could still exist, however they would be in the disconnected state
    #[allow(unused)]
    pub(crate) async fn stop(mut self) {
        self.stopped = true;
        self.cmd_queue.clear();
    }

    async fn extend(&mut self, cmds: Vec<Cmd>, parent_id: usize) -> Vec<Result<()>> {
        let mut results = vec![];

        for cmd in cmds {
            results.push(self.push_internal(cmd, Some(parent_id)).await);
        }

        results
    }

    async fn push_internal(&mut self, cmd: Cmd, parent_id: Option<usize>) -> Result<()> {
        debug!("pushing cmd {cmd:?}");
        if self.stopped().await {
            // should not happen (be reachable)
            return Err(Error::InvalidState);
        }

        let job = CmdJob::new(
            self.id_counter.fetch_add(1, ORDER),
            parent_id,
            cmd,
            SystemTime::now(),
        );

        let prio = job.priority();
        let _ = self.cmd_queue.push(job, prio);
        debug!("cmd pushed to q");
        Ok(())
    }

    // could be accessed via a clone
    async fn stopped(&self) -> bool {
        self.stopped
    }

    async fn notify(&self, event: Event) {
        self.event_sender.send(event).await
    }

    /// Get the next Cmd going off of priority
    pub(crate) fn next_cmd(&mut self) -> Option<CmdJob> {
        self.cmd_queue.pop().map(|(job, _prio)| job)
    }

    // TODO: process a _specific_ cmd,
    // pass in notifier.
    // put this whole thing in another local thread??
    // It needs to be untied from the normal loop?

    /// Processes the next priority cmd
    pub(crate) async fn process_cmd(
        &self,
        cmd_job_id: usize,
        cmd: Cmd,
        cmd_process_api: tokio::sync::mpsc::Sender<(Cmd, Option<usize>)>,
    ) -> Result<()> {
        debug!("processing next cmd");
        // loop {
        if self.stopped().await {
            return Err(Error::CmdCtrlStopped);
        }

        // let expected_rate = self.monitoring.max_cmds_per_s().await;
        // let actual_rate = self.attempted.value();
        // if actual_rate > expected_rate {
        //     let diff = actual_rate - expected_rate;
        //     log_sleep!(Duration::from_millis((diff * 10_f64) as u64));
        //     // continue;
        // } else if self.cmd_queue.is_empty() {
        //     log_sleep!(Duration::from_millis(SLEEP_TIME));
        //     continue;
        // }

        // #[cfg(feature = "test-utils")]
        // {
        //     let queue = self.cmd_queue;
        //     debug!("Cmd queue length: {}", queue.len());
        // }

        debug!("q lennn {}", self.cmd_queue.len());
        // let queue_res = self.cmd_queue.pop();
        // if let Some((job, _prio)) = queue_res {
        // let cmd_ctrl = self;
        self
            .notify(Event::CmdProcessing(CmdProcessEvent::Started {
                job_id: cmd_job_id,
                time: SystemTime::now(),
            }))
            .await;

        // let node = self.dispatcher.node().clone();
        // let comm = self.dispatcher.comm();
        // let dkg_timeout = self.dispatcher.dkg_timeout().clone();

        let dispatcher = self.dispatcher.clone();
        let _ = tokio::task::spawn_local(async move{
        match dispatcher.process_cmd(cmd).await {
            Ok(cmds) => {
                // self.monitoring.increment_cmds().await;

                // evaluate: handle these watchers?
                for cmd in cmds {
                    cmd_process_api.send((cmd, Some(cmd_job_id))).await;
                }
                // self
                //     .notify(Event::CmdProcessing(CmdProcessEvent::Finished {
                //         job: job,
                //         time: SystemTime::now(),
                //     }))
                //     .await;
            }
            Err(error) => {
                // self
                //     .notify(Event::CmdProcessing(CmdProcessEvent::Failed {
                //         job: job.clone(),
                //         time: SystemTime::now(),
                //         error: format!("{:?}", error),
                //     }))
                //     .await;
            }
        }
        // self.attempted.increment(); // both on fail and success
                                    });
                                    // }

        Ok(())
        // }
    }
}

#[derive(Clone, Debug)]
struct CmdThroughput {
    msgs: Arc<AtomicUsize>,
    since: Instant,
}

impl Default for CmdThroughput {
    fn default() -> Self {
        Self {
            msgs: Arc::new(AtomicUsize::new(0)),
            since: Instant::now(),
        }
    }
}

impl CmdThroughput {
    fn increment(&self) {
        let _ = self.msgs.fetch_add(1, Ordering::SeqCst);
    }

    // msgs / s
    fn value(&self) -> f64 {
        let msgs = self.msgs.load(Ordering::SeqCst);
        let seconds = (Instant::now() - self.since).as_secs();
        msgs as f64 / f64::max(1.0, seconds as f64)
    }
}

// #[derive(Debug)]
// pub(crate) struct EnqueuedJob {
//     job: CmdJob,
// }

// impl PartialEq for EnqueuedJob {
//     fn eq(&self, other: &Self) -> bool {
//         self.job.id() == other.job.id()
//     }
// }

// impl Eq for EnqueuedJob {}

// impl std::hash::Hash for EnqueuedJob {
//     fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
//         self.job.id().hash(state);
//     }
// }
