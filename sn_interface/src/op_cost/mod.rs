// Copyright 2023 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use self_encryption::MAX_CHUNK_SIZE;
use sn_dbc::Token;
use tracing::debug;

/// The conversion from token to raw value.
const TOKEN_TO_RAW_CONVERSION: u64 = 1_000_000_000;
/// The maximum supply of SNT, also the larges value that can be represented by a single `Token`.
const MAX_SUPPLY: u64 = (u32::max_value() as u64 + 1) * TOKEN_TO_RAW_CONVERSION;

/// Calculates the required tokens of write operations, for a certain number of bytes,
/// given the network size (current section prefix length), the number of storage
/// nodes in our section, and percent filled.
///
/// This uses an algorithm to calculate a fee with supply/demand adjusting properties
/// (although indirect and therefore arguably sluggish) based on:
/// 1. Network size (contributing to the deflationary character of SNT).
/// 2. Storage used.
/// 3. Requirements for space margin (1/3).
/// 4. Requirement for as low fee as possible for as long as possible.
/// (Point 3. and 4. are achieved by the specific design of the required tokens curve.)
///
/// It is assumed that a larger network means the token has a greater value, and therefore
/// the required number of tokens per operation is also lower, following a curve as per above.
/// The constants used may seem arbitrary but have been carefully chosen as to model the desired behaviour.
pub fn required_tokens(
    bytes: usize,
    prefix_len: usize,
    num_storage_nodes: u8,
    percent_filled: f64,
) -> Token {
    debug!(
        "required_tokens input values; bytes: {bytes}, prefix_len: {prefix_len}, num_storage_nodes: {num_storage_nodes}, percent_filled: {percent_filled}",
    );
    let available_nodes = num_storage_nodes as f64;
    let supply_demand_factor =
        0.001 + (1_f64 / (20_f64 * available_nodes)).powf(8_f64) + percent_filled.powf(3_f64);
    let byte_size_share = bytes as f64 / MAX_CHUNK_SIZE as f64;
    let data_size_factor = byte_size_share + byte_size_share.powf(2_f64);
    let steepness_reductor = prefix_len as f64 + 1_f64;
    let supply_share = max_supply_share_per_section(prefix_len) as f64;
    let token_source = steepness_reductor * supply_share.powf(0.5_f64);
    let required_tokens = (token_source * data_size_factor * supply_demand_factor).round() as u64;
    Token::from_nano(u64::max(1, required_tokens)) // always return > 0
}

// The proportion of total supply "available" per section,
// given a certain network size (i.e. prefix len).
// This is not an actual allocation, just a theoretical value used for the calc of required tokens.
fn max_supply_share_per_section(prefix_len: usize) -> u64 {
    (MAX_SUPPLY as f64 / 2_f64.powf(prefix_len as f64)).floor() as u64
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::messaging::data::DataCmd;
    use std::mem;

    #[test]
    fn calculates_required_tokens() {
        let bytes = MAX_CHUNK_SIZE;
        let prefix_len = 4;
        let num_storage_nodes = 24;
        let percent_filled = 3_f64 / num_storage_nodes as f64;
        let required_tokens =
            required_tokens(bytes, prefix_len, num_storage_nodes, percent_filled).as_nano();
        println!("required_tokens: {required_tokens}");
        assert_eq!(required_tokens, 15_300_364); // 0.01500364 tokens
    }

    #[test]
    fn section_share_of_max_supply_decreases_as_network_grows() {
        // Prefix zero is one section so low usage (lots available).
        let first_section_nanos = max_supply_share_per_section(0);
        assert_eq!(MAX_SUPPLY, first_section_nanos);
        // First split leads to each section "sharing" half the token supply.
        let first_split_nanos = max_supply_share_per_section(1);
        assert_eq!(MAX_SUPPLY / 2, first_split_nanos);
        // At least one token available in up to 2.3 * 10^18 sections, (which is more than one billion times one billion sections).
        let last_split_nanos = max_supply_share_per_section(61);
        assert!(last_split_nanos > 0);
    }

    // -------------------------------------------------------------
    // --------------- Required Tokens Common Sense ---------------------
    // -------------------------------------------------------------
    // Test various different comparisons of the OpCost.
    // These tests are of the type 'all things being equal, then ...'
    // Thanks to @IanColeman for these constributions.

    #[test]
    fn smaller_chunks_require_fewer_tokens() {
        let max_chunk_size = MAX_CHUNK_SIZE;
        let prefix_len = 0;
        let num_storage_nodes = 8;
        let percent_filled = 7_f64 / num_storage_nodes as f64;
        let standard_fee = required_tokens(
            max_chunk_size,
            prefix_len,
            num_storage_nodes,
            percent_filled,
        )
        .as_nano();

        // smaller chunks require fewer tokens
        let max_chunk_size_less_one_byte = max_chunk_size - 1;
        let small_chunk_fee = required_tokens(
            max_chunk_size_less_one_byte,
            prefix_len,
            num_storage_nodes,
            percent_filled,
        )
        .as_nano();
        assert!(
            small_chunk_fee <= standard_fee,
            "small chunks don't require fewer tokens, expect {} <= {}",
            small_chunk_fee,
            standard_fee
        );
    }

    #[test]
    fn fewer_tokens_required_in_larger_net() {
        let max_chunk_size = MAX_CHUNK_SIZE;
        let prefix_len = 2; // first couple of sections see an increase in required tokens, whereafter it is strictly decreasing
        let num_storage_nodes = 8;
        let percent_filled = 7_f64 / num_storage_nodes as f64;
        let standard_fee = required_tokens(
            max_chunk_size,
            prefix_len,
            num_storage_nodes,
            percent_filled,
        )
        .as_nano();
        // ops require fewer tokens in a larger network than in a smaller network
        let larger_prefix = prefix_len + 1;
        let large_network_fee = required_tokens(
            max_chunk_size,
            larger_prefix,
            num_storage_nodes,
            percent_filled,
        )
        .as_nano();
        assert!(
            large_network_fee <= standard_fee,
            "larger network is not cheaper, expect {} <= {}",
            large_network_fee,
            standard_fee
        );
    }

    #[test]
    fn emptier_section_requires_fewer_tokens() {
        let max_chunk_size = MAX_CHUNK_SIZE;
        let prefix_len = 0;
        let num_storage_nodes = 8;
        let percent_filled = 7_f64 / num_storage_nodes as f64;
        let standard_fee = required_tokens(
            max_chunk_size,
            prefix_len,
            num_storage_nodes,
            percent_filled,
        )
        .as_nano();
        // less filled section require fewer tokens than more filled section
        let less_percent_filled = 6_f64 / num_storage_nodes as f64;
        let lower_fee = required_tokens(
            max_chunk_size,
            prefix_len,
            num_storage_nodes,
            less_percent_filled,
        )
        .as_nano();
        assert!(
            lower_fee <= standard_fee,
            "less filled section does not require fewer tokens, expect {} <= {}",
            lower_fee,
            standard_fee
        );
    }

    #[test]
    fn splitting_into_multiple_chunks_require_fewer_tokens_than_same_bytes_in_single_chunk() {
        // we encourage more granularity in data chunking
        let max_chunk_size = MAX_CHUNK_SIZE;
        let prefix_len = 2;
        let num_storage_nodes = 8;
        let percent_filled = 7_f64 / num_storage_nodes as f64;
        let standard_fee = required_tokens(
            max_chunk_size,
            prefix_len,
            num_storage_nodes,
            percent_filled,
        )
        .as_nano();
        // many tiny chunks require fewer tokens than the same bytes in one big chunk
        let one_kb_bytes = 1024;
        let reduced_fee =
            required_tokens(one_kb_bytes, prefix_len, num_storage_nodes, percent_filled).as_nano();
        let combined_fee = reduced_fee * (MAX_CHUNK_SIZE / one_kb_bytes) as u64;
        assert!(
            combined_fee <= standard_fee,
            "many small chunks does not require fewer tokens than one big chunk, expect {} <= {}",
            combined_fee,
            standard_fee,
        );
    }

    #[test]
    fn tokens_are_required_up_to_max_network_size() {
        // The size of the actual DataCmd is used for OpCost calc.
        // In general, the size of a type is not stable across compilations,
        // but it is close enough for our purposes here.
        let minimum_storage_bytes = mem::size_of::<DataCmd>();
        let big_prefix_len = 256;
        let num_storage_nodes = 20;
        let percent_filled = 10_f64 / num_storage_nodes as f64;
        // tokens are required up to 2.3 * 10^78 nodes.
        let required_tokens = required_tokens(
            minimum_storage_bytes,
            big_prefix_len,
            num_storage_nodes,
            percent_filled,
        )
        .as_nano();
        assert!(required_tokens > 0, "tokens are not always required",);
    }
}
