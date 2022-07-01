// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use assert_fs::TempDir;
use criterion::{BatchSize, Criterion, SamplingMode, Throughput};
use duct::cmd;
use sn_api::files::FilesMapChange;
use sn_api::Safe;
use sn_cmd_test_utilities::util::get_bin_location;
use std::{fs, path::PathBuf, time::Duration};
use tokio::runtime::Runtime;

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
fn main() {
    let mut criterion = custom_criterion();
    criterion = criterion.measurement_time(Duration::from_millis(10000));
    // warmup time can mess up the `files` vec
    criterion = criterion.warm_up_time(Duration::from_millis(1));
    bench_cli_files(&mut criterion);
}

/// write random_content to file with its xorname as the filename
fn write_random_content(
    safe: &Safe,
    runtime: &Runtime,
    tmp_dir: &TempDir,
    size: usize,
) -> Result<(PathBuf, String), String> {
    let random_content: String = (0..size).map(|_| rand::random::<char>()).collect();
    let tmp_path = tmp_dir.path().join("temp");
    fs::write(&tmp_path, random_content).map_err(|_| "Error writing random content".to_string())?;

    let (_, processed_files, _) = runtime
        .block_on(safe.files_container_create_from(&tmp_path, None, false, false))
        .map_err(|_| "Cannot get files_container".to_string())?;

    let (_, change) = processed_files
        .iter()
        .next()
        .ok_or_else(|| "Should be present".to_string())?;
    let safe_url = match change {
        FilesMapChange::Failed(err) => return Err(format!("{:?}", err)),
        FilesMapChange::Added(link)
        | FilesMapChange::Updated(link)
        | FilesMapChange::Removed(link) => link.to_string(),
    };

    let file_name: String = safe_url.chars().skip(7).collect();
    let path = tmp_dir.path().join(file_name);
    fs::copy(&tmp_path, &path).map_err(|_| "Error copying file".to_string())?;

    Ok((path, safe_url))
}

fn bench_cli_files(c: &mut Criterion) {
    let mut group = c.benchmark_group("bench_cli_put");
    // since the put command takes ~21 seconds to complete
    group.sampling_mode(SamplingMode::Flat);

    let runtime = Runtime::new().unwrap();
    let safe = Safe::dry_runner(None);
    let tmp_dir = TempDir::new().unwrap();

    for size in [TINY_FILE, SIZE_100KB, SIZE_250KB, SIZE_500KB, SIZE_1MB].iter() {
        let data: String = (0..*size).map(|_| rand::random::<char>()).collect();
        group.throughput(Throughput::Bytes(data.len() as u64));
        let mut files: Vec<String> = Vec::new();

        group.bench_function(format!("cli put {:?} bytes", size), |b| {
            b.iter_batched(
                || {
                    let (file_path, file_url) =
                        write_random_content(&safe, &runtime, &tmp_dir, *size).unwrap();
                    println!("PUT {:?}", file_url);
                    files.push(file_url);
                    file_path
                },
                |file_path| {
                    //  use the safe command, so for bench it has to be installed
                    let _ = cmd!(get_bin_location().unwrap(), "files", "put", file_path)
                        .read()
                        .unwrap();
                },
                BatchSize::SmallInput,
            )
        });

        group.bench_function(format!("cli cat {:?} bytes", size), |b| {
            b.iter_batched(
                || {
                    let file_url = files.pop().unwrap();
                    let file_name: String = file_url.chars().skip(7).collect();
                    let data = fs::read(tmp_dir.path().join(file_name)).unwrap();
                    println!("CAT {:?}", file_url);
                    (file_url, data)
                },
                |(file_url, data)| {
                    let cat_data = cmd!(get_bin_location().unwrap(), "cat", file_url)
                        .read()
                        .unwrap();
                    assert_eq!(data.as_slice(), cat_data.as_bytes());
                },
                BatchSize::SmallInput,
            )
        });
    }
    group.finish();
}
