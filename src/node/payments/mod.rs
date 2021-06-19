// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

//pub mod elder_signing;
mod reward_calc;
mod reward_wallets;

use self::reward_calc::distribute_rewards;
pub use self::reward_wallets::RewardWallets;
use crate::{
    messaging::{
        client::{
            ClientMsg, ClientSig, CmdError, DebitableCmd, Error as ErrorMsg, GuaranteedQuote,
            PaymentError, PaymentQuote, ProcessMsg, QueryResponse,
        },
        Aggregation, EndUser, MessageId, SrcLocation,
    },
    node::{
        capacity::StoreCost as StoreCostCalc,
        node_ops::{MsgType, NodeDuty, OutgoingMsg},
        Error, Result,
    },
    types::{NodeAge, PublicKey, Token},
};
use log::info;
use sn_dbc::{KeyManager, Mint, ReissueRequest};
use std::collections::{BTreeMap, BTreeSet};
use xor_name::{Prefix, XorName};

type Payment = BTreeMap<PublicKey, sn_dbc::Dbc>;

/// The management of section funds,
/// via the usage of a distributed AT2 Actor.
#[allow(clippy::large_enum_variant)]
#[derive(Clone)]
pub struct Payments<K: KeyManager> {
    store_cost: StoreCostCalc,
    wallets: RewardWallets,
    mint: Mint<K>,
}

impl<K: KeyManager> Payments<K> {
    ///
    pub fn new(store_cost: StoreCostCalc, wallets: RewardWallets, mint: Mint<K>) -> Self {
        Self {
            store_cost,
            wallets,
            mint,
        }
    }

    /// Returns registered wallet key of a node.
    #[allow(unused)]
    pub fn get_node_wallet(&self, node_name: &XorName) -> Option<PublicKey> {
        let (_, key) = self.wallets.get(node_name)?;
        Some(key)
    }

    /// Returns node wallet keys of registered nodes.
    #[allow(unused)]
    pub fn node_wallets(&self) -> BTreeMap<XorName, (NodeAge, PublicKey)> {
        self.wallets.node_wallets()
    }

    /// Nodes register/updates wallets for future reward payouts.
    #[allow(unused)]
    pub fn set_node_wallet(&self, node_id: XorName, wallet: PublicKey, age: u8) {
        self.wallets.set_node_wallet(node_id, age, wallet)
    }

    /// When the section becomes aware that a node has left,
    /// its reward key is removed.
    #[allow(unused)]
    pub fn remove_node_wallet(&self, node_name: XorName) {
        self.wallets.remove_wallet(node_name)
    }

    /// When the section becomes aware that a node has left,
    /// its reward key is removed.
    #[allow(unused)]
    pub fn keep_wallets_of(&self, prefix: Prefix) {
        self.wallets.keep_wallets_of(prefix)
    }



    /// Get latest StoreCost for the given number of bytes.
    #[allow(unused)]
    pub async fn get_store_cost(
        &mut self,
        data: BTreeMap<bls::PublicKey, BTreeMap<XorName, u64>>,
        msg_id: MessageId,
        origin: SrcLocation,
    ) -> NodeDuty {
        let result = self
            .store_cost(data)
            .await
            .map_err(|e| crate::messaging::client::Error::InvalidOperation(e.to_string()));
        NodeDuty::Send(OutgoingMsg {
            msg: MsgType::Client(ClientMsg::Process(ProcessMsg::QueryResponse {
                response: QueryResponse::GetStoreCost(result),
                id: MessageId::in_response_to(&msg_id),
                correlation_id: msg_id,
            })),
            section_source: false, // strictly this is not correct, but we don't expect responses to a response..
            dst: origin.to_dst(),
            aggregation: Aggregation::None, // TODO: to_be_aggregated: Aggregation::AtDestination,
        })
    }

    #[allow(unused)]
    async fn store_cost(
        &self,
        data: BTreeMap<bls::PublicKey, BTreeMap<XorName, u64>>,
    ) -> Result<PaymentQuote> {
        let bytes = data.values().map(|c| c.values().sum()).sum();

        if data.is_empty() || bytes == 0 {
            Err(Error::InvalidOperation("Cannot store 0 bytes".to_string()))
        } else {
            let store_cost = self.store_cost.from(bytes).await?;
            info!("Store cost for {:?} bytes: {}", bytes, store_cost);
            let payable: BTreeMap<_, _> = distribute_rewards(store_cost, self.node_wallets())
                .iter()
                .map(|(_, (_, key, amount))| (*key, *amount))
                .collect();

            data.iter().map(|(section_key, data)| {
                PaymentQuote {
                    bytes: data.values().sum(),
                    data: data.keys().copied().collect(),
                    payable,
                }
            })
            Ok(PaymentQuote {
                bytes,
                data: data.keys().copied().collect(),
                payable,
            })
        }
    }

    // pub async fn reissue(&self, req: ReissueRequest) -> Result<NodeDuty> {
    //     let inputs_belonging_to_mints = req
    //         .transaction
    //         .inputs
    //         .iter()
    //         .filter(|dbc| is_close(dbc.name()))
    //         .map(|dbc| dbc.name())
    //         .collect();

    //     match self
    //         .mint
    //         .reissue(req.clone(), inputs_belonging_to_mints)
    //         .await
    //     {
    //         Ok((tx, tx_sigs)) => {
    //             // // TODO: store these somewhere, for nodes to claim
    //             // let _output_dbcs: Vec<_> = payment
    //             //     .transaction
    //             //     .outputs
    //             //     .into_iter()
    //             //     .map(|content| Dbc {
    //             //         content,
    //             //         transaction: tx.clone(),
    //             //         transaction_sigs: tx_sigs.clone(),
    //             //     })
    //             //     .collect();

    //             info!("Payment: forwarding data..");
    //             Ok(NodeDuty::Send(OutgoingMsg {
    //                 msg: MsgType::Node(NodeMsg::NodeCmd {
    //                     cmd: NodeCmd::Metadata {
    //                         cmd: data_cmd.clone(),
    //                         client_sig,
    //                         origin,
    //                     },
    //                     id: MessageId::in_response_to(&msg_id),
    //                 }),
    //                 section_source: true, // i.e. errors go to our section
    //                 dst: DstLocation::Section(data_cmd.dst_address()),
    //                 aggregation: Aggregation::AtDestination,
    //             }))
    //         }
    //         Err(e) => {
    //             warn!("Payment failed: {:?}", e);
    //             Ok(NodeDuty::Send(OutgoingMsg {
    //                 msg: MsgType::Client(ClientMsg::Process(ProcessMsg::CmdError {
    //                     error: CmdError::Payment(e.into()),
    //                     id: MessageId::in_response_to(&msg_id),
    //                     correlation_id: msg_id,
    //                 })),
    //                 section_source: false, // strictly this is not correct, but we don't expect responses to an error..
    //                 dst: SrcLocation::EndUser(origin).to_dst(),
    //                 aggregation: Aggregation::None, // TODO: to_be_aggregated: Aggregation::AtDestination,
    //             }))
    //         }
    //     }
    // }

    /// Makes sure the payment contained
    /// within a data write, is credited
    /// to the nodes.
    pub async fn process_payment(
        &mut self,
        cmd: NetworkCmd,
        client_sig: ClientSig,
        msg_id: MessageId,
        origin: EndUser,
    ) -> Result<NodeDuty> {
        let (_data_cmd, _payment) = match self.validate_payment(cmd, client_sig).await {
            Ok(result) => result,
            Err(e) => {
                return Ok(NodeDuty::Send(OutgoingMsg {
                    msg: MsgType::Client(ClientMsg::Process(ProcessMsg::CmdError {
                        error: CmdError::Payment(PaymentError(ErrorMsg::InvalidOperation(
                            e.to_string(),
                        ))),
                        id: MessageId::in_response_to(&msg_id),
                        correlation_id: msg_id,
                    })),
                    section_source: false, // strictly this is not correct, but we don't expect responses to a response..
                    dst: SrcLocation::EndUser(origin).to_dst(),
                    aggregation: Aggregation::None, // TODO: to_be_aggregated: Aggregation::AtDestination,
                }));
            }
        };
        // info!("Payment: forwarding data..");
        // Ok(NodeDuty::Send(OutgoingMsg {
        //     msg: MsgType::Node(NodeMsg::NodeCmd {
        //         cmd: NodeCmd::Metadata {
        //             cmd: data_cmd.clone(),
        //             client_sig,
        //             origin,
        //         },
        //         id: MessageId::in_response_to(&msg_id),
        //     }),
        //     section_source: true, // i.e. errors go to our section
        //     dst: DstLocation::Section(data_cmd.dst_address()),
        //     aggregation: Aggregation::AtDestination,
        // }))
        Ok(NodeDuty::NoOp)
    }

    async fn validate_payment(
        &self,
        cmd: NetworkCmd,
        _client_sig: ClientSig,
    ) -> Result<(DebitableCmd, BTreeMap<PublicKey, sn_dbc::Dbc>)> {
        if cmd.payment.amount() > cmd.quote.amount() {
            Ok((cmd.op, cmd.payment))
        } else {
            return Err(Error::InvalidOperation(format!(
                "Too low payment: {}, expected: {}",
                cmd.payment.amount(),
                cmd.quote.amount(),
            )));
        }
    }

    // async fn validate_payment(
    //     &self,
    //     msg: ProcessMsg,
    // ) -> Result<(ReissueRequest, DataCmd, ClientSig)> {
    //     let (payment, quote, data_cmd, client_sig) = match msg {
    //         ProcessMsg::Cmd {
    //             cmd:
    //                 Cmd::Debitable(NetworkCmd {
    //                     op: DebitableCmd::Data(cmd),
    //                     quote,
    //                     payment,
    //                 }),
    //             client_sig,
    //             ..
    //         } => (payment, quote, cmd, client_sig),
    //         _ => {
    //             return Err(Error::InvalidOperation(
    //                 "Payment is only needed for data writes.".to_string(),
    //             ))
    //         }
    //     };

    //     let total_cost = quote.amount();
    //     if total_cost > payment.amount() {
    //         return Err(Error::InvalidOperation(format!(
    //             "Too low payment: {}, expected: {}",
    //             payment.amount(),
    //             total_cost
    //         )));
    //     }
    //     // // verify that each dbc amount is for an existing node,
    //     // // and that the amount is proportional to the its age.
    //     // for out in &payment.transaction.outputs {
    //     //     // TODO: let node_wallet = out.owner_key;
    //     //     let node_wallet = get_random_pk();
    //     //     match quote.payable.get(&node_wallet) {
    //     //         Some(expected_amount) => {
    //     //             if expected_amount.as_nano() > out.amount {
    //     //                 return Err(Error::InvalidOperation(format!(
    //     //                     "Too low payment for {}: {}, expected {}",
    //     //                     node_wallet,
    //     //                     expected_amount,
    //     //                     Token::from_nano(out.amount),
    //     //                 )));
    //     //             }
    //     //         }
    //     //         None => return Err(Error::InvalidOwner(node_wallet)),
    //     //     }
    //     // }

    //     info!(
    //         "Payment: OK. (Store cost: {}, paid amount: {}.)",
    //         total_cost,
    //         payment.amount()
    //     );

    //     Ok((payment, data_cmd, client_sig))
    // }
}

trait Amount {
    fn amount(&self) -> Token;
}

impl Amount for ReissueRequest {
    fn amount(&self) -> Token {
        Token::from_nano(self.transaction.inputs.iter().map(|dbc| dbc.amount()).sum())
    }
}

impl Amount for Payment {
    fn amount(&self) -> Token {
        Token::from_nano(self.values().map(|v| v.amount()).sum())
    }
}

impl Amount for PaymentQuote {
    fn amount(&self) -> Token {
        Token::from_nano(self.payable.values().map(|v| v.as_nano()).sum())
    }
}

impl Amount for GuaranteedQuote {
    fn amount(&self) -> Token {
        self.quote.amount()
    }
}
