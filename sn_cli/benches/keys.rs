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

use sn_cmd_test_utilities::get_bin_location;
use std::time::Duration;

// sample size is _NOT_ the number of times the command is run...
// https://bheisler.github.io/criterion.rs/book/analysis.html#measurement
const SAMPLE_SIZE: usize = 10;

fn custom_criterion() -> Criterion {
    Criterion::default().sample_size(SAMPLE_SIZE)
}

fn main() {
    let mut criterion = custom_criterion();
    criterion = criterion.measurement_time(Duration::from_millis(20_000));

    bench_cli_keys(&mut criterion);
}

fn bench_cli_keys(c: &mut Criterion) {
    c.bench_function("generating keys", |b| {
        b.iter(|| {
            //  use the safe command, so for bench it has to be installed
            cmd!(get_bin_location(), "keys", "create", "--test-coins")
                .read()
                .unwrap()
        })
    });
}
