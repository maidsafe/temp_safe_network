// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::{
    client::{Error, Result},
    types::Chunk,
};
use async_trait::async_trait;
use bincode::serialize;
use futures::future::join_all;

#[allow(unused)]
pub(crate) struct ChunkUploader<U: Uploader> {
    uploader: U,
}

#[async_trait]
pub(crate) trait Uploader: Clone {
    async fn upload(&self, bytes: &[u8]) -> Result<()>;
}

impl<U: Uploader + Send + Sync + 'static> ChunkUploader<U> {
    #[allow(unused)]
    pub(crate) fn new(uploader: U) -> Self {
        Self { uploader }
    }

    #[allow(unused)]
    pub(crate) async fn store(&self, chunks: Vec<Chunk>) -> Result<()> {
        let handles =
            chunks
                .into_iter()
                .map(|c| (c, self.uploader.clone()))
                .map(|(chunk, uploader)| {
                    tokio::spawn(async move {
                        let serialized_chunk = serialize(&chunk)?;
                        uploader.upload(&serialized_chunk).await
                    })
                });

        let results = join_all(handles).await;

        for res1 in results {
            match res1 {
                Ok(res2) => {
                    if res2.is_err() {
                        return res2;
                    }
                }
                Err(e) => return Err(Error::Generic(e.to_string())),
            }
        }

        Ok(())
    }
}
