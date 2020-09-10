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
use std::fs;
use std::time::Duration;

use criterion::{BatchSize, Criterion};

const TEST_FILE_RANDOM_CONTENT: &str = "test_file_random_content.txt";

// sample size is _NOT_ the number of times the command is run...
// https://bheisler.github.io/criterion.rs/book/analysis.html#measurement
const SAMPLE_SIZE: usize = 10;
// random data limits to generate a file of size (in bytes):
const SIZE_1MB: usize = 1_000_000;
const SIZE_500KB: usize = 500_000;
const SIZE_250KB: usize = 250_000;
const SIZE_100KB: usize = 100_000;
const TINY_FILE: usize = 10;

use sn_cmd_test_utilities::get_bin_location;

fn custom_criterion() -> Criterion {
    Criterion::default().sample_size(SAMPLE_SIZE)
}
fn main() {
    let mut criterion = custom_criterion();
    criterion = criterion.measurement_time(Duration::from_millis(10000));

    bench_cli_put(&mut criterion);
}

fn put_random_content(size: usize) -> Result<(), String> {
    let random_content: String = (0..size).map(|_| rand::random::<char>()).collect();
    fs::write(TEST_FILE_RANDOM_CONTENT, random_content)
        .map_err(|_| "Error writing random content".to_string())?;

    Ok(())
}

fn bench_cli_put(c: &mut Criterion) {
    c.bench_function("cli put random tiny file", |b| {
        b.iter_batched(
            || put_random_content(TINY_FILE),
            |_| {
                //  use the safe command, so for bench it has to be installed
                cmd!(get_bin_location(), "files", "put", TEST_FILE_RANDOM_CONTENT)
                    .read()
                    .unwrap()
            },
            BatchSize::SmallInput,
        )
    });
    c.bench_function("cli put random 100 KB data", |b| {
        b.iter_batched(
            || put_random_content(SIZE_100KB),
            |_| {
                //  use the safe command, so for bench it has to be installed
                cmd!(get_bin_location(), "files", "put", TEST_FILE_RANDOM_CONTENT)
                    .read()
                    .unwrap()
            },
            BatchSize::SmallInput,
        )
    });

    c.bench_function("cli put random 250 KB data", |b| {
        b.iter_batched(
            || put_random_content(SIZE_250KB),
            |_| {
                //  use the safe command, so for bench it has to be installed
                cmd!(get_bin_location(), "files", "put", TEST_FILE_RANDOM_CONTENT)
                    .read()
                    .unwrap()
            },
            BatchSize::SmallInput,
        )
    });
    c.bench_function("cli put random 500 KB data", |b| {
        b.iter_batched(
            || put_random_content(SIZE_500KB),
            |_| {
                //  use the safe command, so for bench it has to be installed
                cmd!(get_bin_location(), "files", "put", TEST_FILE_RANDOM_CONTENT)
                    .read()
                    .unwrap()
            },
            BatchSize::SmallInput,
        )
    });
    c.bench_function("cli put random 1 meg data", |b| {
        b.iter_batched(
            || put_random_content(SIZE_1MB),
            |_| {
                //  use the safe command, so for bench it has to be installed
                cmd!(get_bin_location(), "files", "put", TEST_FILE_RANDOM_CONTENT)
                    .read()
                    .unwrap()
            },
            BatchSize::SmallInput,
        )
    });
}
