// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

/// Common utility functions for writing test cases.
#[cfg(any(test, feature = "test-utils"))]
pub mod test_utils;

use rand::distributions::Alphanumeric;
use rand::rngs::OsRng;
use rand::{self, Rng};
use rayon::current_num_threads;

/// Generates a `String` from `length` random UTF-8 `char`s.  Note that the NULL character will be
/// excluded to allow conversion to a `CString` if required, and that the actual `len()` of the
/// returned `String` will likely be around `4 * length` as most of the randomly-generated `char`s
/// will consume 4 elements of the `String`.
pub fn generate_random_string(length: usize) -> String {
    let mut rng = OsRng;
    ::std::iter::repeat(())
        .map(|()| rng.gen::<char>())
        .filter(|c| *c != '\u{0}')
        .take(length)
        .collect()
}

/// Generates a readable `String` using provided `length` and only ASCII characters.
pub fn generate_readable_string(length: usize) -> String {
    let mut rng = OsRng;
    ::std::iter::repeat(())
        .map(|()| rng.sample(Alphanumeric))
        .take(length)
        .collect()
}

/// Generates a random vector using provided `length`.
pub fn generate_random_vector(length: usize) -> Vec<u8> {
    use rayon::prelude::*;
    let threads = current_num_threads();

    if threads > length {
        let mut rng = OsRng;
        return ::std::iter::repeat(())
            .map(|()| rng.gen::<u8>())
            .take(length)
            .collect();
    }

    let per_thread = length / threads;
    let remainder = length % threads;

    let mut bytes: Vec<u8> = (0..threads)
        .par_bridge()
        .map(|_| vec![0u8; per_thread])
        .map(|mut bytes| {
            let bytes = bytes.as_mut_slice();
            rand::thread_rng().fill(bytes);
            bytes.to_owned()
        })
        .flatten()
        .collect();

    bytes.extend(vec![0u8; remainder]);

    bytes
}

/// Convert binary data to a diplay-able format
#[inline]
pub fn bin_data_format(data: &[u8]) -> String {
    let len = data.len();
    if len < 8 {
        return format!("[ {:?} ]", data);
    }

    format!(
        "[ {:02x} {:02x} {:02x} {:02x}..{:02x} {:02x} {:02x} {:02x} ]",
        data[0],
        data[1],
        data[2],
        data[3],
        data[len - 4],
        data[len - 3],
        data[len - 2],
        data[len - 1]
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    const SIZE: usize = 10;

    // Test `generate_random_string` and that the results are not repeated.
    #[test]
    fn random_string() {
        let str0 = generate_random_string(SIZE);
        let str1 = generate_random_string(SIZE);
        let str2 = generate_random_string(SIZE);

        assert_ne!(str0, str1);
        assert_ne!(str0, str2);
        assert_ne!(str1, str2);

        assert_eq!(str0.chars().count(), SIZE);
        assert_eq!(str1.chars().count(), SIZE);
        assert_eq!(str2.chars().count(), SIZE);
    }

    // Test `generate_random_vector` and that the results are not repeated.
    #[test]
    fn random_vector() {
        let vec0 = generate_random_vector(SIZE);
        let vec1 = generate_random_vector(SIZE);
        let vec2 = generate_random_vector(SIZE);

        assert_ne!(vec0, vec1);
        assert_ne!(vec0, vec2);
        assert_ne!(vec1, vec2);

        assert_eq!(vec0.len(), SIZE);
        assert_eq!(vec1.len(), SIZE);
        assert_eq!(vec2.len(), SIZE);
    }
}
