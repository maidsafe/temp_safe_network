// Copyright 2023 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use thiserror::Error;

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    IoError(#[from] std::io::Error),
    #[error("Invalid download URL: {0}")]
    InvalidDownloadUrl(String),
    #[error("Invalid target path: {0}")]
    InvalidTargetPath(String),
    #[error("Could not parse version number from release version '{0}'")]
    InvalidReleaseVersionFormat(String),
    #[error(transparent)]
    SelfUpdate(#[from] self_update::errors::Error),
    #[error("Failed to update binary: {0}")]
    UpdateFailed(String),
    #[error(transparent)]
    Url(#[from] url::ParseError),
}
