// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// http://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

#[macro_use]
extern crate duct;
use criterion::Criterion;
use std::time::Duration;

extern crate sn_cmd_test_utilities;

use sn_api::Error;
use sn_cmd_test_utilities::{create_wallet_with_balance, get_bin_location};

// sample size is _NOT_ the number of times the command is run...
// https://bheisler.github.io/criterion.rs/book/analysis.html#measurement
const SAMPLE_SIZE: usize = 10;

fn custom_criterion() -> Criterion {
    Criterion::default().sample_size(SAMPLE_SIZE)
}

fn main() {
    let mut criterion = custom_criterion();
    criterion = criterion.measurement_time(Duration::from_millis(20_000));

    bench_cli_wallet(&mut criterion).unwrap();
}

fn bench_cli_wallet(c: &mut Criterion) -> Result<(), Error> {
    let (wallet_from, _pk, _sk) = create_wallet_with_balance("1600.000000001", None)?; // we need 1 nano to pay for the costs of creation
    let (wallet_to, _pk, _sk) = create_wallet_with_balance("5.000000001", None)?; // we need 1 nano to pay for the costs of creation
    c.bench_function("performing transfers", |b| {
        b.iter(|| {
            let result = cmd!(
                get_bin_location(),
                "wallet",
                "transfer",
                "1",
                "--from",
                &wallet_from,
                "--to",
                &wallet_to
            )
            .read()
            .unwrap();

            assert!(result.contains("Success"))
        })
    });
    Ok(())
}
