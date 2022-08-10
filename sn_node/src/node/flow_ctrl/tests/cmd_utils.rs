use crate::node::{
    flow_ctrl::dispatcher::Dispatcher,
    messaging::{OutgoingMsg, Peers},
    Cmd,
};
use assert_matches::assert_matches;
use eyre::Result;
use sn_interface::{
    messaging::{
        system::{JoinResponse, MembershipState, RelocateDetails, SystemMsg},
        MsgType,
    },
    network_knowledge::{test_utils::*, NodeState, SectionAuthorityProvider},
    types::{Peer, SecretKeySet},
};
use std::collections::BTreeSet;

pub(crate) struct HandleOnlineStatus {
    pub(crate) node_approval_sent: bool,
    pub(crate) relocate_details: Option<RelocateDetails>,
}

pub(crate) async fn handle_online_cmd(
    peer: &Peer,
    sk_set: &SecretKeySet,
    dispatcher: &Dispatcher,
    section_auth: &SectionAuthorityProvider,
) -> Result<HandleOnlineStatus> {
    let node_state = NodeState::joined(*peer, None);
    let membership_decision = section_decision(sk_set, node_state.to_msg())?;

    let all_cmds = run_and_collect_cmds(
        Cmd::HandleMembershipDecision(membership_decision),
        dispatcher,
    )
    .await?;

    let mut status = HandleOnlineStatus {
        node_approval_sent: false,
        relocate_details: None,
    };

    for cmd in all_cmds {
        let (msg, recipients) = match cmd {
            Cmd::SendMsg {
                recipients,
                msg: OutgoingMsg::System(msg),
                ..
            } => (msg, recipients),
            _ => continue,
        };

        match msg {
            SystemMsg::JoinResponse(response) => {
                if let JoinResponse::Approved {
                    section_auth: signed_sap,
                    ..
                } = *response
                {
                    assert_eq!(signed_sap.value, section_auth.clone().to_msg());
                    assert_matches!(recipients, Peers::Multiple(peers) => {
                        assert_eq!(peers, BTreeSet::from([*peer]));
                    });
                    status.node_approval_sent = true;
                }
            }
            SystemMsg::Propose {
                proposal: sn_interface::messaging::system::Proposal::Offline(node_state),
                ..
            } => {
                if let MembershipState::Relocated(details) = node_state.state {
                    if details.previous_name != peer.name() {
                        continue;
                    }
                    status.relocate_details = Some(*details.clone());
                }
            }
            _ => continue,
        }
    }

    Ok(status)
}

pub(crate) async fn run_and_collect_cmds(cmd: Cmd, dispatcher: &Dispatcher) -> Result<Vec<Cmd>> {
    let mut all_cmds = vec![];

    let mut cmds = dispatcher.process_cmd(cmd).await?;

    while !cmds.is_empty() {
        all_cmds.extend(cmds.clone());
        let mut new_cmds = vec![];
        for cmd in cmds {
            if !matches!(cmd, Cmd::SendMsg { .. }) {
                new_cmds.extend(dispatcher.process_cmd(cmd).await?);
            }
        }
        cmds = new_cmds;
    }

    Ok(all_cmds)
}
