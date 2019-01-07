// Copyright 2018 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::chunk_store;
use config_file_handler;
use maidsafe_utilities::serialisation::SerialisationError;
use routing::messaging;
use routing::ClientError;
use routing::{InterfaceError, MessageId, Request, Response, RoutingError};
use serde_json;
use std::io;
use quick_error::quick_error;

quick_error! {
    #[cfg_attr(feature = "cargo-clippy", allow(large_enum_variant))]
    #[derive(Debug)]
    pub enum InternalError {
        ChunkStore(error: chunk_store::Error) {
            from()
        }
        FailedToFindCachedRequest(message_id: MessageId)
        FileHandler(error: config_file_handler::Error) {
            from()
        }
        Io(error: io::Error) {
            from()
        }
        MpidMessaging(error: messaging::Error) {
            from()
        }
        Routing(error: InterfaceError) {
            from()
        }
        RoutingClient(error: ClientError) {
            from()
        }
        RoutingInternal(error: RoutingError) {
            from()
        }
        Serialisation(error: SerialisationError) {
            from()
        }
        JsonSerialisation(error: serde_json::Error) {
            from()
        }
        UnknownRequestType(request: Request)
        UnknownResponseType(response: Response)
        InvalidMessage
        NoSuchAccount
    }
}
