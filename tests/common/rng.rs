// Copyright 2019 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use rand::{rngs::OsRng, CryptoRng, Error, Rng, RngCore, SeedableRng};
use rand_chacha::ChaChaRng;

pub struct TestRng(ChaChaRng);

impl RngCore for TestRng {
    fn next_u32(&mut self) -> u32 {
        self.0.next_u32()
    }

    fn next_u64(&mut self) -> u64 {
        self.0.next_u64()
    }

    fn fill_bytes(&mut self, dest: &mut [u8]) {
        self.0.fill_bytes(dest)
    }

    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), Error> {
        self.0.try_fill_bytes(dest)
    }
}

impl CryptoRng for TestRng {}

// Compatibility with routing.
// TODO: remove this when we update rand to the same version that routing uses.
impl routing_rand_core::RngCore for TestRng {
    fn next_u32(&mut self) -> u32 {
        self.0.next_u32()
    }

    fn next_u64(&mut self) -> u64 {
        self.0.next_u64()
    }

    fn fill_bytes(&mut self, dest: &mut [u8]) {
        self.0.fill_bytes(dest)
    }

    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), routing_rand_core::Error> {
        self.0.fill_bytes(dest);
        Ok(())
    }
}

// Create new random number generator using random seed.
pub fn new() -> TestRng {
    let mut rng = OsRng::new().expect("Failed to create OS RNG");
    from_seed(rng.gen())
}

pub fn from_rng<R: RngCore>(rng: &mut R) -> TestRng {
    TestRng(ChaChaRng::from_seed(rng.gen()))
}

pub fn from_seed(seed: u64) -> TestRng {
    TestRng(ChaChaRng::seed_from_u64(seed))
}
