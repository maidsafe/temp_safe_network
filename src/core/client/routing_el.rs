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
use maidsafe_utilities::serialisation::deserialise;
use routing::{Event, MessageId, Response};
use routing::client_errors::{GetError, MutationError};
use std::sync::mpsc::Receiver;

/// Run the routing event loop - this will receive messages from routing.
pub fn run<T>(routing_rx: Receiver<Event>, core_tx: CoreMsgTx<T>, net_tx: NetworkTx)
    where T: 'static
{
    for it in routing_rx.iter() {
        trace!("Received Routing Event: {:?}", it);
        match it {
            Event::Response { response, .. } => {
                let (id, event) = handle_resp(response);
                if !fire(&core_tx, id, event) {
                    break;
                }
            }
            Event::RestartRequired => {
                if net_tx.send(NetworkEvent::Disconnected).is_err() {
                    break;
                }

                let msg = {
                    let core_tx = core_tx.clone();
                    let net_tx = net_tx.clone();
                    CoreMsg::new(move |client, _| {
                        client.restart_routing(core_tx, net_tx);
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

fn handle_resp(resp: Response) -> (MessageId, CoreEvent) {
    match resp {
        Response::GetSuccess(data, id) => (id, CoreEvent::Get(Ok(data))),
        Response::GetFailure { id, data_id, external_error_indicator } => {
            let reason = parse_get_err(&external_error_indicator);
            let e = CoreError::GetFailure {
                data_id: data_id,
                reason: reason,
            };
            (id, CoreEvent::Get(Err(e)))
        }
        Response::PutSuccess(_, id) |
        Response::PostSuccess(_, id) |
        Response::DeleteSuccess(_, id) |
        Response::AppendSuccess(_, id) => (id, CoreEvent::Mutation(Ok(()))),
        Response::PutFailure { id, data_id, external_error_indicator } |
        Response::PostFailure { id, data_id, external_error_indicator } |
        Response::DeleteFailure { id, data_id, external_error_indicator } |
        Response::AppendFailure { id, data_id, external_error_indicator } => {
            let reason = parse_mutation_err(&external_error_indicator);
            let e = CoreError::MutationFailure {
                data_id: data_id,
                reason: reason,
            };
            (id, CoreEvent::Mutation(Err(e)))
        }
        Response::GetAccountInfoSuccess { id, data_stored, space_available } => {
            (id, CoreEvent::AccountInfo(Ok((data_stored, space_available))))
        }
        Response::GetAccountInfoFailure { id, external_error_indicator } => {
            let reason = parse_get_err(&external_error_indicator);
            let e = CoreError::GetAccountInfoFailure { reason: reason };
            (id, CoreEvent::AccountInfo(Err(e)))
        }
    }
}

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

/// Fire completion event to the core event loop. If the receiver in core event
/// loop has hung up or sending fails for some other reason, treat it as an
/// exit condition. The return value thus signifies if the firing was
/// successful.
fn fire<T>(core_tx: &CoreMsgTx<T>, id: MessageId, event: CoreEvent) -> bool {
    let msg = CoreMsg::new(move |client, _| {
        client.fire_hook(&id, event);
        None
    });

    core_tx.send(msg).is_ok()
}
