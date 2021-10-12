// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::node::{Error, Result};
use std::fmt::{self, Formatter};

pub(crate) use adult_role::AdultRole;
pub(crate) use elder_role::ElderRole;

mod adult_role;
mod elder_role;

#[allow(clippy::large_enum_variant)]
#[derive(custom_debug::Debug)]
pub(crate) enum Role {
    Adult(#[debug(skip)] AdultRole),
    Elder(#[debug(skip)] ElderRole),
}

impl Role {
    pub(crate) fn as_adult(&self) -> Result<&AdultRole> {
        match self {
            Self::Adult(adult) => Ok(adult),
            _ => Err(Error::NotAnAdult),
        }
    }

    pub(crate) fn as_elder(&self) -> Result<&ElderRole> {
        match self {
            Self::Elder(elder) => Ok(elder),
            _ => Err(Error::NotAnElder),
        }
    }
}

impl fmt::Display for Role {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Elder(_) => write!(f, "Elder"),
            Self::Adult(_) => write!(f, "Adult"),
        }
    }
}
