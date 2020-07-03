
// Copyright 2019 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

mod blob_storage;
mod map_storage;
mod sequence_storage;
mod elder_stores;
mod blob_register;
mod reading;
mod writing;

use reading::Reading;
use writing::Writing;
use crate::{action::Action, rpc::Rpc, utils, node::Init, Config, Result};
use blob_register::BlobRegister;
use blob_storage::BlobStorage;
use elder_stores::ElderStores;
use map_storage::MapStorage;
use routing::{Node, SrcLocation};
use sequence_storage::SequenceStorage;

use log::{debug, error, trace};
use safe_nd::{
    IDataAddress, Read, Write, BlobRead, BlobWrite, MapRead, MapWrite, SequenceRead, SequenceWrite, 
    MessageId, NodePublicId, PublicId, Request, NodeRequest, Response, XorName,
};
use threshold_crypto::{Signature, SignatureShare};

use std::{
    cell::{Cell, RefCell},
    collections::BTreeSet,
    fmt::{self, Display, Formatter},
    rc::Rc,
};


