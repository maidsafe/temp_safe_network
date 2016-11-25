// Copyright 2016 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under (1) the MaidSafe.net
// Commercial License, version 1.0 or later, or (2) The General Public License
// (GPL), version 3, depending on which licence you accepted on initial access
// to the Software (the "Licences").
//
// By contributing code to the SAFE Network Software, or to this project
// generally, you agree to be bound by the terms of the MaidSafe Contributor
// Agreement, version 1.0.
// This, along with the Licenses can be found in the root directory of this
// project at LICENSE, COPYING and CONTRIBUTOR.
//
// Unless required by applicable law or agreed to in writing, the SAFE Network
// Software distributed under the GPL Licence is distributed on an "AS IS"
// BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or
// implied.
//
// Please review the Licences for the specific language governing permissions
// and limitations relating to use of the SAFE Network Software.

use core::{CoreError, CoreMsg, CoreMsgTx, NetworkTx};
use core::event::{CoreEvent, NetworkEvent};
use routing::{Event, MessageId, Response};
use std::sync::mpsc::Receiver;

/// Run the routing event loop - this will receive messages from routing.
pub fn run<T>(routing_rx: Receiver<Event>, mut core_tx: CoreMsgTx<T>, mut net_tx: NetworkTx)
    where T: 'static
{
    for it in routing_rx.iter() {
        trace!("Received Routing Event: {:?}", it);
        match it {
            Event::Response { response, .. } => {
                let (msg_id, event) = match get_core_event(response) {
                    Ok(val) => val,
                    Err(_) => break,
                };
                if !fire(&mut core_tx, msg_id, event) {
                    break;
                }
            }
            Event::RestartRequired => {
                if net_tx.send(NetworkEvent::Disconnected).is_err() {
                    break;
                }

                let msg = {
                    let _core_tx = core_tx.clone();
                    let _net_tx = net_tx.clone();
                    CoreMsg::new(move |_client, _| {
                        // TODO(nbaksalyar) uncomment
                        // client.restart_routing(core_tx, net_tx);
                        None
                    })
                };

                if core_tx.send(msg).is_err() {
                    break;
                }
            }
            Event::Terminate => break,
            x => {
                debug!("Routing Event {:?} is not handled in context of routing event loop.",
                       x);
            }
        }
    }
}

fn get_core_event(res: Response) -> Result<(MessageId, CoreEvent), CoreError> {
    Ok(match res {
        Response::GetIData { res, msg_id } => {
            (msg_id, CoreEvent::GetIData(res.map_err(CoreError::from)))
        }
        Response::PutIData { res, msg_id } => {
            (msg_id, CoreEvent::PutIData(res.map_err(CoreError::from)))
        }
        Response::PutMData { res, msg_id } => {
            (msg_id, CoreEvent::PutMData(res.map_err(CoreError::from)))
        }
        Response::GetMDataValue { res, msg_id } => {
            (msg_id, CoreEvent::GetMDataValue(res.map_err(CoreError::from)))
        }
        _ => return Err(CoreError::Unexpected("Invalid response type".to_owned())),
    })
}

/*
pub fn parse_get_err(reason_raw: &[u8]) -> GetError {
    match deserialise(&reason_raw) {
        Ok(elt) => elt,
        Err(e) => {
            let err_msg = format!("Couldn't obtain get failure reason: {:?}", e);
            warn!("{}", err_msg);
            GetError::NetworkOther(err_msg)
        }
    }
}

pub fn parse_mutation_err(reason_raw: &[u8]) -> MutationError {
    match deserialise(&reason_raw) {
        Ok(elt) => elt,
        Err(e) => {
            let err_msg = format!("Couldn't obtain mutation failure reason: {:?}", e);
            warn!("{}", err_msg);
            MutationError::NetworkOther(err_msg)
        }
    }
}
*/
/// Fire completion event to the core event loop. If the receiver in core event
/// loop has hung up or sending fails for some other reason, treat it as an
/// exit condition. The return value thus signifies if the firing was
/// successful.
fn fire<T>(core_tx: &mut CoreMsgTx<T>, msg_id: MessageId, event: CoreEvent) -> bool {
    let msg = CoreMsg::new(move |client, _| {
        client.fire_hook(&msg_id, event);
        None
    });

    core_tx.send(msg).is_ok()
}
