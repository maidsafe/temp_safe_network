// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use assert_fs::TempDir;
use color_eyre::Result;
use criterion::{BatchSize, BenchmarkId, Criterion, Throughput};
use rand::{distributions::Alphanumeric, Rng};
use sn_cmd_test_utilities::util::{parse_files_put_or_sync_output, safe_cmd_stdout};
use std::{fs, path::PathBuf, time::Duration};

// sample size is _NOT_ the number of times the command is run...
// https://bheisler.github.io/criterion.rs/book/analysis.html#measurement
const SAMPLE_SIZE: usize = 10;
// random data limits to generate a file of size (in bytes):
const SIZE_1MB: usize = 1_000_000;
const SIZE_500KB: usize = 500_000;
const SIZE_250KB: usize = 250_000;
const SIZE_100KB: usize = 100_000;
const TINY_FILE: usize = 10;

fn custom_criterion() -> Criterion {
    Criterion::default().sample_size(SAMPLE_SIZE)
}
fn main() -> Result<()> {
    let mut criterion = custom_criterion();
    criterion = criterion.measurement_time(Duration::from_millis(10000));
    bench_cli_put(&mut criterion)?;
    bench_cli_cat(&mut criterion)?;

    Ok(())
}

fn bench_cli_put(c: &mut Criterion) -> Result<()> {
    let mut group = c.benchmark_group("cli_put");
    let tmp_dir = TempDir::new().unwrap();

    for size in &[TINY_FILE, SIZE_100KB, SIZE_250KB, SIZE_500KB, SIZE_1MB] {
        group.throughput(Throughput::Bytes(random_data(*size).len() as u64));

        group.bench_function(BenchmarkId::new("put", size), |b| {
            b.iter_batched(
                || write_random_content(&tmp_dir, *size),
                |file_path| {
                    let _ =
                        safe_cmd_stdout(["files", "put", &file_path.to_string_lossy()], Some(0))
                            .unwrap();
                },
                BatchSize::SmallInput,
            )
        });
    }

    Ok(())
}

fn bench_cli_cat(c: &mut Criterion) -> Result<()> {
    let mut group = c.benchmark_group("cli_cat");
    let tmp_dir = TempDir::new().unwrap();

    for size in &[TINY_FILE, SIZE_100KB, SIZE_250KB, SIZE_500KB, SIZE_1MB] {
        group.throughput(Throughput::Bytes(random_data(*size).len() as u64));

        group.bench_function(BenchmarkId::new("cat", size), |b| {
            b.iter_batched(
                || {
                    // put file and return its safe_url
                    let file_path = write_random_content(&tmp_dir, *size);
                    let output = safe_cmd_stdout(
                        ["files", "put", &file_path.to_string_lossy(), "--json"],
                        Some(0),
                    )
                    .unwrap();
                    let (_, processed) = parse_files_put_or_sync_output(&output).unwrap();
                    let safe_url = processed[&file_path].link().unwrap();
                    safe_url.clone()
                },
                |safe_url| {
                    let _ = safe_cmd_stdout(["cat", &safe_url], Some(0)).unwrap();
                },
                BatchSize::SmallInput,
            )
        });
    }

    Ok(())
}

fn random_data(size: usize) -> String {
    (0..size).map(|_| rand::random::<char>()).collect()
}
fn write_random_content(tmp_dir: &TempDir, size: usize) -> PathBuf {
    let file_name: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(20)
        .map(char::from)
        .collect();
    let file_path = tmp_dir.path().join(file_name);
    fs::write(&file_path, random_data(size))
        .map_err(|_| "Error writing random content".to_string())
        .unwrap();
    file_path
}
