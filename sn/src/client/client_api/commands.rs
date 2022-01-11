// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::Client;
use crate::at_least_one_correct_elder;
use crate::client::Error;
use crate::messaging::{
    data::{DataCmd, ServiceMsg},
    ServiceAuth, WireMsg,
};
use crate::types::{PublicKey, Signature};
use bytes::Bytes;
use xor_name::XorName;

impl Client {
    /// Send a signed DataCmd to the network.
    /// This is to be part of a public API, for the user to
    /// provide the serialised and already signed command.
    pub async fn send_signed_command(
        &self,
        dst_address: XorName,
        client_pk: PublicKey,
        serialised_cmd: Bytes,
        signature: Signature,
        targets: usize,
    ) -> Result<(), Error> {
        let auth = ServiceAuth {
            public_key: client_pk,
            signature,
        };

        self.session
            .send_cmd(dst_address, auth, serialised_cmd, targets)
            .await
    }

    // Send a DataCmd to the network without awaiting for a response.
    // This function is a helper private to this module.
    #[instrument(skip_all, level = "debug", name = "client-api send cmd")]
    pub(crate) async fn send_cmd(&self, cmd: DataCmd) -> Result<(), Error> {
        let client_pk = self.public_key();
        let dst_name = cmd.dst_name();

        let targets = at_least_one_correct_elder(); // stored at Adults, so only 1 correctly functioning Elder need to relay

        let serialised_cmd = {
            let msg = ServiceMsg::Cmd(cmd);
            WireMsg::serialize_msg_payload(&msg)?
        };
        let signature = self.keypair.sign(&serialised_cmd);

        self.send_signed_command(dst_name, client_pk, serialised_cmd, signature, targets)
            .await
    }
}
