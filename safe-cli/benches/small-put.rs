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

use criterion::{BatchSize, Criterion};

const TEST_FILE_RANDOM_CONTENT: &str = "test_file_random_content.txt";

const SAMPLE_SIZE: usize = 20;
// random data limits to generate a file of size:
const FOUR_MEGABYTE: usize = 1_000_000;
const EIGHT_MEGABYTE: usize = 2_000_000;
const ONE_MEGABYTE: usize = 250_000;
const HALF_MEGABYTE: usize = 125_000;
const TINY_FILE: usize = 10;

fn custom_criterion() -> Criterion {
    Criterion::default().sample_size(SAMPLE_SIZE)
}
fn main() {
    let mut criterion = custom_criterion();

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
                cmd!("safe", "files", "put", TEST_FILE_RANDOM_CONTENT)
                    .read()
                    .unwrap()
            },
            BatchSize::SmallInput,
        )
    });
    c.bench_function("cli put random 1/2 meg data", |b| {
        b.iter_batched(
            || put_random_content(HALF_MEGABYTE),
            |_| {
                //  use the safe command, so for bench it has to be installed
                cmd!("safe", "files", "put", TEST_FILE_RANDOM_CONTENT)
                    .read()
                    .unwrap()
            },
            BatchSize::SmallInput,
        )
    });
    c.bench_function("cli put random 1 meg data", |b| {
        b.iter_batched(
            || put_random_content(ONE_MEGABYTE),
            |_| {
                //  use the safe command, so for bench it has to be installed
                cmd!("safe", "files", "put", TEST_FILE_RANDOM_CONTENT)
                    .read()
                    .unwrap()
            },
            BatchSize::SmallInput,
        )
    });
    c.bench_function("cli put random 4 meg data", |b| {
        b.iter_batched(
            || put_random_content(FOUR_MEGABYTE),
            |_| {
                //  use the safe command, so for bench it has to be installed
                cmd!("safe", "files", "put", TEST_FILE_RANDOM_CONTENT)
                    .read()
                    .unwrap()
            },
            BatchSize::SmallInput,
        )
    });
    c.bench_function("cli put random 8 meg data", |b| {
        b.iter_batched(
            || put_random_content(EIGHT_MEGABYTE),
            |_| {
                //  use the safe command, so for bench it has to be installed
                cmd!("safe", "files", "put", TEST_FILE_RANDOM_CONTENT)
                    .read()
                    .unwrap()
            },
            BatchSize::SmallInput,
        )
    });
}
