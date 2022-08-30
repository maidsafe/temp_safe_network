// Copyright 2022 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

//! Instrumentation for statemaps: https://github.com/TritonDataCenter/statemap

use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::json;

use super::core::Node;

pub(crate) enum State {
    Idle,
    ProcessCmd,
    Comms,
    Validation,
    Dysfunction,
    SystemMsg,
    ServiceMsg,
    Dkg,
    Agreement,
    Membership,
    Handover,
    Replication,
}

impl Node {
    pub(crate) fn statemap_log_metadata(&self) {
        let start_dur = match self.start_time.duration_since(UNIX_EPOCH) {
            Ok(dur) => dur,
            Err(e) => {
                error!("STATEMAP: failed to calculate start time {e:?}");
                return;
            }
        };
        let secs = start_dur.as_secs();
        let nanos = start_dur.subsec_nanos();
        let name = self.name();
        let states = json!({
            "Idle": {"value": State::Idle as usize, "color": "#f9f9f9"},
            "ProcessCmd": {"value": State::ProcessCmd as usize, "color": "#a0522d"},
            "Comms": {"value": State::Comms as usize, "color": "#008000"},
            "Validation": {"value": State::Validation as usize, "color": "#4b0082"},
            "Dysfunction": {"value": State::Dysfunction as usize, "color": "#ff0000"},
            "SystemMsg": {"value": State::SystemMsg as usize, "color": "#ffd700"},
            "ServiceMsg": {"value": State::ServiceMsg as usize, "color": "#7fff00"},
            "Dkg": {"value": State::Dkg as usize, "color": "#00ffff"},
            "Agreement": {"value": State::Agreement as usize, "color": "#0000ff"},
            "Membership": {"value": State::Membership as usize, "color": "#ff00ff"},
            "Handover": {"value": State::Handover as usize, "color": "#6495ed"},
            "Replication": {"value": State::Replication as usize, "color": "#ff69b4"},
        });

        let metadata = json!({
            "title": format!("The various states safe_network"),
            "start": [secs, nanos],
            "host": name,
            "states": states
        });

        trace!("STATEMAP_METADATA: {metadata}");
    }

    pub(crate) fn statemap_log_state(&self, state: State) {
        // { "time": "1579579142", "entity": "<xorname>", "state": 6 }
        let time = match SystemTime::now().duration_since(self.start_time) {
            Ok(t) => t.as_nanos(),
            Err(e) => {
                error!("STATEMAP: failed to read system time: {e:?}");
                return;
            }
        };
        let name = self.name();
        let entry = json!({
            "time": format!("{}", time),
            "entity": name,
            "state": state as usize,
        });

        info!("STATEMAP_ENTRY: {entry}")
    }
}
