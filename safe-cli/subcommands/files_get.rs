// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// http://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

use super::{
    helpers::{div_or, pluralize, prompt_user},
    OutputFmt,
};
use console::Term;
use indicatif::{MultiProgress, ProgressBar, ProgressDrawTarget, ProgressStyle, TickTimeLimit};
use log::{debug, info, trace, warn};
use safe_api::{
    fetch::Range,
    files::{FilesMap, ProcessedFiles},
    xorurl::{SafeDataType, XorUrl, XorUrlEncoder},
    Error, Result as ApiResult, Safe,
};
use std::fs;
use std::io::BufWriter;
use std::io::Write;
use std::path::Path;
use std::time::Duration;

/// # Retrieval/write status for current file and overall transfer.
#[derive(Debug, Clone)]
pub struct FilesGetStatus<'a, 'b> {
    pub path_remote: &'a Path,
    pub path_local: &'b Path,
    pub total_files: u64,
    pub current_file: u64,
    pub total_transfer_bytes: u64,
    pub transfer_bytes_written: u64,
    pub file_size: u64,
    pub file_bytes_written: u64,
}

/// # Action to perform when downloading if a file already exists.
#[derive(Debug)]
pub enum FileExistsAction {
    Overwrite,
    Preserve,
    Ask,
}

/// Default action is Ask
impl Default for FileExistsAction {
    fn default() -> Self {
        FileExistsAction::Ask
    }
}

// implement FromStr for parsing "--exists" arg.
impl std::str::FromStr for FileExistsAction {
    type Err = String;
    fn from_str(str: &str) -> Result<Self, String> {
        match str {
            "overwrite" => Ok(Self::Overwrite),
            "preserve" => Ok(Self::Preserve),
            "ask" => Ok(Self::Ask),
            other => Err(format!(
                "'{}' not supported. Supported values are ask, preserve, and overwrite",
                other
            )),
        }
    }
}

// What type of Progress Indicator to display.
#[derive(Debug)]
pub enum ProgressIndicator {
    Bars,
    Text,
    None,
}

// implement FromStr for parsing "--exists" arg.
impl std::str::FromStr for ProgressIndicator {
    type Err = String;
    fn from_str(str: &str) -> Result<Self, String> {
        match str {
            "bars" => Ok(Self::Bars),
            "text" => Ok(Self::Text),
            "none" => Ok(Self::None),
            other => Err(format!(
                "'{}' not supported. Supported values are bars, text, and none",
                other
            )),
        }
    }
}

/// Default progress indicator is Bars
impl Default for ProgressIndicator {
    fn default() -> Self {
        ProgressIndicator::Bars
    }
}

// processes the `safe files get` command.  called by files.rs
//
// dest is a local path.  defaults to "."
//   Path will be created if not existing, else error.
//
// TODO: _preserve file attributes is not yet implemented, we need them
//   stored in metadata first.
//
// TBD: how should we handle OutputFmt?  Presently, we are displaying
// progress bars, and also [possibly] prompting user about overwrites.
// We have a list of processed files that we could present as json
// or in table form.  But if stdout format is json and we prompt user,
// then any process parsing output as json will break.  So possibly
// --exists=ask and --json should conflict and not be allowed.
//
// This command is really similar to cp or scp, and people are fine
// using those without a report.  So it doesn't seem especially urgent.
pub async fn process_get_command(
    safe: &mut Safe,
    source: XorUrl,
    dest: Option<String>,
    exists: FileExistsAction,
    progress: ProgressIndicator,
    _preserve: bool,
    _output_fmt: OutputFmt,
) -> Result<(), String> {
    let str_path = dest.unwrap_or_else(|| ".".to_string());
    let path = Path::new(&str_path);

    let (mp, bars) = create_progress_bars();

    let mut overwrites: u64 = 0;
    let mut preserves: u64 = 0;

    let (_version, processed_files) =
        files_container_get_files(&safe, &source, &str_path, |status| {
            let mut overwrite = true;
            let mut mystatus = status.clone();

            if status.file_bytes_written == 0 {
                // It is an error/warning if the dest path attempts to use
                // an existing file as a directory. But other files should
                // still be written.  eg:
                // $ mkdir -p /tmp/a/b/c && touch /tmp/a/file.txt
                // $ mkdir /tmp/target && touch /tmp/target/b   (b is a file)
                // $ cp -r /tmp/a/* /tmp/target
                //    cp: cannot overwrite non-directory '/tmp/target/b' with directory '/tmp/a/b'
                // $ ls -l /tmp/target/
                //      total 0
                //      -rw-rw-r-- 1 user user 0 Mar 31 14:38 b         (b still a file)
                //      -rw-rw-r-- 1 user user 0 Mar 31 14:38 file.txt  (other file written)
                //
                // TBD: Should FileExistsAction apply to this case?
                //      unix cp does not provide any flag/option/prompt to permit this
                //      and it always emits a warning.  So I am satisfied with this
                //      working the same way, at least for now.
                if let Some(parent) = status.path_local.parent() {
                    if let Some(filepath) = path_contains_file(&parent) {
                        let msg = format!(
                            "cannot overwrite non-directory '{}' with directory in '{}'",
                            filepath.display(),
                            status.path_local.display()
                        );
                        let err = Error::FileSystemError(msg.clone());

                        warn!("Skipping file \"{}\". {}", status.path_local.display(), err);
                        if isatty::stderr_isatty() {
                            eprintln!("Warning: {}", msg);
                        }
                        overwrite = false;
                    }
                }
                if status.path_local.exists() && overwrite {
                    overwrite = match exists {
                        FileExistsAction::Overwrite => true,
                        FileExistsAction::Preserve => false,
                        FileExistsAction::Ask => {
                            let prompt = format!(
                                "overwrite '{}'? ",
                                status.path_local.display().to_string()
                            );
                            prompt_yes_no(&prompt, "Y")
                        }
                    };
                    if overwrite {
                        overwrites += 1;
                    } else {
                        preserves += 1;
                        mystatus.total_transfer_bytes -= mystatus.file_size;
                    }
                }
            }
            if overwrite {
                match progress {
                    ProgressIndicator::Bars => {
                        update_progress_bars(&mp, &bars, &mystatus);
                    }
                    ProgressIndicator::Text => {
                        print_status(status);
                    }
                    ProgressIndicator::None => {}
                }
            }
            overwrite
        })
        .await?;

    if processed_files.is_empty() && preserves == 0 {
        return Err("Path not found".to_string());
    } else {
        print_results(&processed_files, path, overwrites, preserves);
    }

    Ok(())
}

// detects if a path contains a file at any level.
//   eg    /tmp/foo/somefile/bar/other
//   if somefile exists and is a file, it will be returned.
fn path_contains_file(path: &Path) -> Option<&Path> {
    let mut p: &Path = path;

    loop {
        if p.is_file() {
            return Some(p);
        }
        match p.parent() {
            Some(parent) => {
                p = parent;
            }
            None => break,
        }
    }
    None
}

// prints results/summary of GET transfer
fn print_results(processed_files: &ProcessedFiles, path: &Path, overwrites: u64, preserves: u64) {
    if overwrites > 0 || preserves > 0 {
        println!(
            "Done. Retrieved {} {} to {}.\n  pre-existing: {}   (overwritten: {}  preserved: {})",
            processed_files.len(),
            pluralize("file", "files", processed_files.len() as u64),
            path.display(),
            overwrites + preserves,
            overwrites,
            preserves
        );
    } else {
        println!(
            "Done. Retrieved {} files to {}",
            processed_files.len(),
            path.display()
        );
    }
}

fn print_status(status: &FilesGetStatus) {
    // TBD: This is displaying pretty much all progress info, and it might be
    // information overload.
    println!(
        "{} - files: {} of {} ({:.0}%). transfer: {} of {} ({:.0}%), file: {} of {} ({:.0}%)",
        status.path_remote.display(),
        status.current_file,
        status.total_files,
        div_or(status.current_file as f64, status.total_files as f64, 1.0) * 100.0,
        status.transfer_bytes_written,
        status.total_transfer_bytes,
        div_or(
            status.transfer_bytes_written as f64,
            status.total_transfer_bytes as f64,
            1.0
        ) * 100.0,
        status.file_bytes_written,
        status.file_size,
        div_or(
            status.file_bytes_written as f64,
            status.file_size as f64,
            1.0
        ) * 100.0
    );
}

// Update the progress bars.
// Called once before each file starts downloading,
// and again after each chunk until it finishes.
// Current state can be determined by
//    checking transfer_bvtes_written and file_bytes_written
fn update_progress_bars(m: &MultiProgress, bars: &[ProgressBar], status: &FilesGetStatus) {
    let b_onefile = status.file_size == status.total_transfer_bytes;

    // do some setup if the transfer is just starting.
    if status.transfer_bytes_written == 0 {
        // Hide bar1 if we are only downloading 1 file in this transfer.
        if b_onefile {
            bars[1].finish_and_clear();
        }
    }

    // set bar 1 length and reset elapsed time when starting a new file.
    if status.file_bytes_written == 0 {
        bars[0].set_length(status.file_size);
        bars[0].reset();

        // We re-set this for each file because it can change if a pre-existing file is preserved.
        bars[2].set_length(status.total_transfer_bytes);
        // thread::sleep(Duration::from_millis(1000));

        if !b_onefile {
            bars[1].set_length(status.file_size);
            bars[1].reset();
        }
    }

    let msg = format!(
        "File [{} of {}]: {}",
        status.current_file,
        status.total_files,
        status.path_remote.display().to_string().trim_matches('/')
    );

    // set bar 0 message, and set position for all 3 bars.
    bars[0].set_message(&msg);
    bars[0].set_position(status.file_bytes_written);
    bars[1].set_position(status.file_bytes_written);
    bars[2].set_position(status.transfer_bytes_written);

    // finish bars 0 and 1 when current file has been downloaded.
    if status.file_size == status.file_bytes_written {
        bars[0].finish_at_current_pos();
        bars[1].finish_at_current_pos();
    }

    // Hide bars 0 and 1 when transfer is done.
    if status.total_transfer_bytes == status.transfer_bytes_written {
        bars[0].finish_and_clear();
        bars[1].finish_and_clear();
        bars[2].finish_at_current_pos();
    }
    // tell bar to update/display.
    m.tick(TickTimeLimit::Timeout(Duration::from_millis(50)))
        .unwrap_or(());
}

// Creates and inits the progress bars.
// We use 3. The first just reports the path.
// The 2nd and 3rd represent File progress and Transfer progress respectively.
fn create_progress_bars() -> (MultiProgress, Vec<ProgressBar>) {
    let m = MultiProgress::with_draw_target(ProgressDrawTarget::stdout_nohz());
    let sty_file = ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta})  File")
        .progress_chars("##-");
    let sty_transfer = ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta})  Transfer")
        .progress_chars("##-");
    let sty_path = ProgressStyle::default_bar().template("{msg}");

    // first bar just prints path of file downloading.
    // second bar is for single file progress, in bytes.
    // third bar is for entire transfer progress, in bytes.
    let b1 = m.add(ProgressBar::new(100));
    let b2 = m.add(ProgressBar::new(100));
    let b3 = m.add(ProgressBar::new(100));

    b1.set_style(sty_path);
    b2.set_style(sty_file);
    b3.set_style(sty_transfer);

    let bars = vec![b1, b2, b3];

    (m, bars)
}

// Prompts user for [Y/n] input.
// TODO: make i18n friendly.
fn prompt_yes_no(prompt_msg: &str, default: &str) -> bool {
    let yes_no = "[Y/n]";
    let msg = format!("{}{}: ", prompt_msg, yes_no);
    loop {
        let result = prompt_user(&msg, "");
        let choice = match result {
            Ok(input) => input.to_uppercase(),
            Err(input) => {
                if input.is_empty() {
                    default.to_string()
                } else {
                    input
                }
            }
        };
        match choice.as_str() {
            "Y" => {
                return true;
            }
            "N" => {
                return false;
            }
            _ => {}
        };
        // prevent scrolling after user hits Enter.
        // This is a partially successful attempt to keep progress bar
        // painting from getting screwed up.
        Term::stdout().clear_last_lines(1).unwrap_or(())
    }
}

/// # Downloads files within a FilesContainer that match the xorurl path component and writes them to disk, preserving paths.
///
/// TODO: In the future, this will have options for preserving symlinks and
/// file attributes.
/// ```
async fn files_container_get_matching(safe: &Safe, url: &str) -> ApiResult<(u64, FilesMap)> {
    let (version, files_map) = safe.files_container_get(&url).await?;

    let filtered_filesmap = filter_files_map_by_xorurl_path(&files_map, &url, |_, _| None)?;

    Ok((version, filtered_filesmap))
}

/// # Downloads all files within a FilesContainer and writes them to disk, preserving paths.
///
/// TODO: In the future, this will have options for preserving symlinks and
/// file attributes.
async fn files_container_get_files(
    safe: &Safe,
    url: &str,
    dirpath: &str,
    callback: impl FnMut(&FilesGetStatus) -> bool,
) -> ApiResult<(u64, ProcessedFiles)> {
    let (xorurl_encoder, nrs_encoder) = safe.parse_and_resolve_url(url).await?;

    // if nrs_encoder is not None, then 'url' is an NRS link and
    // we must append the nrs_url path to the xor_url path
    // to obtain full path within the FileContainer.
    // Else we just use the xor_url path.
    let resolved_url_encoder = match nrs_encoder {
        Some(p) => {
            let mut s = xorurl_encoder.path_decoded()?;
            s.push_str(&p.path_decoded()?);
            let mut e = xorurl_encoder.clone();
            e.set_path(&s);
            e
        }
        None => xorurl_encoder.clone(),
    };

    let resolved_url = resolved_url_encoder.to_string();

    // note: files_container_get_matching() also calls safe.parse_and_resolve_url().
    // We should somehow modify the API so this redundancy can be removed as it requires
    // unnecessary network requests.
    let (version, files_map) = files_container_get_matching(&safe, &resolved_url).await?;

    debug!("Getting files in container {:?}", url);
    debug!("resolved url is {:?}", &resolved_url);

    // Todo: This test will need to be modified once we support empty directories.
    let is_single_file = files_map.len() == 1;

    let urlpath = resolved_url_encoder.path_decoded()?;

    let root = find_root_path(&dirpath, &urlpath, is_single_file)?;

    // This is a constraint to verify that parent of dirpath exists.
    // Without this check, files_map_get_files() will happily create
    // any missing dirs, which "might" be ok.  However, unix 'cp'
    // enforces that parent dir exists, so we will do the same to avoid
    // surprising users.
    ensure_parent_dir_exists(&root)?;

    let processed_files = files_map_get_files(&safe, &files_map, &urlpath, &root, callback).await?;
    Ok((version, processed_files))
}

// Determines the root (translated) path to download files to.
// The root path is determined as per the follow matrix:
/*

source     |source type| dest                      | dest exists | dest type | translated
---------------------------------------------------------------------------------------
testdata   | dir       | /tmp/testdata             | Y           | dir       | /tmp/testdata/testdata
testdata   | dir       | /tmp/testdata             | Y           | file      | error:  cannot overwrite non-directory '/tmp/testdata' with directory '../testdata/'
testdata   | dir       | /tmp/testdata             | N           | --        | /tmp/testdata

testdata   | dir       | /tmp/newname              | Y           | dir       | /tmp/newname/testdata
testdata   | dir       | /tmp/newname              | Y           | file      | error:  cannot overwrite non-directory '/tmp/testdata' with directory '../testdata/'
testdata   | dir       | /tmp/newname              | N           | --        | /tmp/newname

-- source is a file --

testdata   | file      | /tmp/testdata             | Y           | dir       | /tmp/testdata/testdata
testdata   | file      | /tmp/testdata             | Y           | file      | /tmp/testdata
testdata   | file      | /tmp/testdata             | N           | --        | /tmp/testdata

testdata   | file      | /tmp/newname              | Y           | dir       | /tmp/newname/testdata
testdata   | file      | /tmp/newname              | Y           | file      | /tmp/newname
testdata   | file      | /tmp/newname              | N           | --        | /tmp/newname
*/
#[allow(clippy::collapsible_if)]
fn find_root_path(
    destpath: &str,
    sourcepath: &str,
    source_is_single_file: bool,
) -> ApiResult<String> {
    // Note: The if+else clauses could be combined to be more
    // compact, but I am leaving it in expanded form to be more easily
    // understood in context of the path matrix in the fn comment.

    let mut root = Path::new(destpath).to_path_buf();
    if source_is_single_file {
        if root.exists() {
            if root.is_dir() {
                let p = Path::new(sourcepath);
                if let Some(fname) = p.file_name() {
                    root.push(fname);
                }
            }
        }
    } else {
        if root.exists() {
            if root.is_dir() {
                let p = Path::new(sourcepath);
                if let Some(fname) = p.file_name() {
                    root.push(fname);
                }
            } else {
                let msg = format!(
                    "cannot overwrite non-directory '{}' with a directory",
                    destpath
                );
                return Err(Error::FileSystemError(msg));
            }
        }
    }
    Ok(root.display().to_string())
}

// Verifies that parent directory of a given path exists.
fn ensure_parent_dir_exists(path: &str) -> ApiResult<()> {
    let p = Path::new(path);

    // a relative path such as '.' or 'somedir' or 'somefile'
    // has an implicit parent.
    if p.is_relative() && p.components().count() == 1 {
        return Ok(());
    }

    if let Some(pa) = p.parent() {
        if pa.is_dir() {
            return Ok(());
        } else {
            return Err(Error::FileSystemError(format!(
                "No such directory: \"{}\"",
                pa.display()
            )));
        }
    }
    // This should never happen.
    Err(Error::FileSystemError(format!(
        "Parent directory not found for: \"{}\"",
        p.display()
    )))
}

/// # Downloads files within a FilesMap and writes them to disk, preserving paths.
///
/// TODO: In the future, this will have options for preserving symlinks and
/// file attributes.
async fn files_map_get_files(
    safe: &Safe,
    files_map: &FilesMap,
    sourcepath: &str,
    dirpath: &str,
    mut callback: impl FnMut(&FilesGetStatus) -> bool,
) -> ApiResult<ProcessedFiles> {
    trace!("Fetching files from FilesMap");

    let dpath = Path::new(dirpath);

    let mut processed_files = ProcessedFiles::new();
    let mut transfer_bytes_written = 0;

    // We need to calc total_transfer_bytes in advance for status callback
    let mut total_transfer_bytes = files_map
        .iter()
        .map(|(_path, details)| &details["size"])
        .fold(0, |tot, size| tot + size.parse().unwrap_or(0));

    // Loop through files map and download each file.
    // caller may cancel individual files, but not entire transfer.
    for (idx, (path, details)) in files_map.iter().enumerate() {
        // fetch xorurl and write to path.  directory paths created if needed.

        // Here, we rely/assume that the FilesMap has already been filtered
        // so that only files matching source_relpath prefix are present.
        let source_relpath = &path[sourcepath.len()..];

        let xorurl = &details["link"];
        let abspath = if !source_relpath.is_empty() {
            dpath.join(source_relpath.trim_matches('/'))
        } else {
            dpath.to_path_buf()
        };
        trace!("file target path: {}", abspath.display());

        // determine the file size from metadata.  string must be parsed.
        let size: u64 = details["size"].parse().map_err(|err| {
            Error::Unexpected(format!(
                "Invalid file size: {} for {}.  {:?}",
                details["size"], xorurl, err
            ))
        })?;

        // Setup status to notify our caller of progress in callback.
        let mut status = FilesGetStatus {
            path_remote: &Path::new(path),
            path_local: &abspath.as_path(),
            total_files: files_map.len() as u64,
            current_file: idx as u64 + 1,
            total_transfer_bytes,
            transfer_bytes_written,
            file_size: size,
            file_bytes_written: 0,
        };

        // status callback before file download begins.
        let b_write = callback(&status);
        if !b_write {
            // If caller decides not to download this file, then we need to
            // deduct the file size from total bytes in transfer.
            total_transfer_bytes -= size;
            continue;
        }

        // Download file.  We handle callback from download_file_from_net()
        // and our handler calls a callback supplied by our caller.
        match download_file_from_net(
            &safe,
            xorurl,
            abspath.as_path(),
            size,
            |_path, _file_size, file_bytes_written, last_write: u64| {
                transfer_bytes_written += last_write;
                status.transfer_bytes_written = transfer_bytes_written;
                status.file_bytes_written = file_bytes_written;

                // status callback for each chunk of file downloaded.
                callback(&status);
                true
            },
        )
        .await
        {
            Ok(_bytes) => {
                processed_files.insert(path.to_string(), ("+".to_string(), xorurl.to_string()));
            }
            Err(err) => {
                processed_files.insert(path.to_string(), ("E".to_string(), format!("<{}>", err)));
                info!("Skipping file \"{}\". {}", path, err);
            }
        };
    }

    Ok(processed_files)
}

/// # Filter out file items outside of xorurl path
///
/// This function accepts a callback/closure that can optionally
/// translate paths in the filtered FilesMap that is returned.
///
/// The callback accepts 1: xorurl_path, and 2: path from input FilesMap.
/// It returns a modified path, or None.
///
/// The xorurl_path is the optional path component of an XorUrl, eg in
/// safe://hnyynyiy3wc3ciagtspu9ntb78rce5r994pjhhbx9jo9cjedkchu6ug9zqbnc/testdata/subfolder
/// the xorurl_path is /testdata/subfolder
///
/// Todo: this API should/will interpret and filter by wildcard, and accept
/// ranges and sets, as bash does, eg *.txt, photo{1,3,5}.jpg, photo{1-3}.jpg
///
pub fn filter_files_map_by_xorurl_path(
    files_map: &FilesMap,
    target_url: &str,
    mut callback: impl FnMut(&str, &str) -> Option<String>,
) -> ApiResult<FilesMap> {
    let xorurl_encoder = Safe::parse_url(target_url)?;

    let path = xorurl_encoder.path_decoded()?;

    Ok(filter_files_map_by_path(files_map, &path, |fmpath| {
        callback(&path, &fmpath)
    }))
}

/// # Filter out file items outside of path
///
/// This function accepts a callback/closure that can optionally
/// translate paths in the filtered FilesMap that is returned.
///
/// The callback accepts 1: xorurl_path, and 2: path from input FilesMap.
/// It returns a modified path, or None.
///
/// The xorurl_path is the optional path component of an XorUrl, eg in
/// safe://hnyynyiy3wc3ciagtspu9ntb78rce5r994pjhhbx9jo9cjedkchu6ug9zqbnc/testdata/subfolder
/// the xorurl_path is /testdata/subfolder
///
/// Todo: this API should/will interpret and filter by wildcard, and accept
/// ranges and sets, as bash does, eg *.txt, photo{1,3,5}.jpg, photo{1-3}.jpg
fn filter_files_map_by_path(
    files_map: &FilesMap,
    path: &str,
    mut callback: impl FnMut(&str) -> Option<String>,
) -> FilesMap {
    let mut filtered_filesmap = FilesMap::default();

    files_map.iter().for_each(|(filepath, fileitem)| {
        if filepath
            .trim_end_matches('/')
            .starts_with(&path.trim_end_matches('/'))
        {
            let filtered_path = match callback(&filepath) {
                Some(p) => p,
                None => filepath.to_string(),
            };
            filtered_filesmap.insert(filtered_path, fileitem.clone());
        }
    });
    filtered_filesmap
}

// Downloads a file from the network to a given file path
// Data is downloaded and written to filesystem in 64k chunks.
// xorurl must point to immutable data
// size (in bytes) must be provided
// A callback/closure is called after each chunk is downloaded.
async fn download_file_from_net(
    safe: &Safe,
    xorurl: &str,
    path: &Path,
    size: u64,
    //Path, file_size, file_bytes_written, bytes_written.  return false to cancel download.
    mut callback: impl FnMut(&Path, u64, u64, u64) -> bool,
) -> ApiResult<u64> {
    debug!("downloading file {} to {}", xorurl, path.display());

    // get directory path
    let mut dir_path = path.to_path_buf();
    dir_path.pop();

    create_dir_all(&dir_path.as_path())?;

    // chunk_size based on https://stackoverflow.com/questions/8803515/optimal-buffer-size-for-write2
    // originally it was 4096 to match common disk block size, but that seems a bit small for the
    // network, so I multiplied by 16.  Perhaps should make it a param so caller can decide.
    let chunk_size: u64 = 65536;
    let mut rcvd: u64 = 0;
    let mut bytes_written: u64 = 0;

    let fh = file_create(path)?;
    let mut stream = BufWriter::new(fh);

    // stream and write the file in chunk_size pieces
    while rcvd < size {
        let start = rcvd;
        let end = if rcvd + chunk_size < size {
            rcvd + chunk_size
        } else {
            size
        };
        let range = Some((Some(start), Some(end)));
        // gets public or private, based on xorurl type
        let filedata = files_get_immutable(&safe, &xorurl, range).await?;
        bytes_written += stream_write(&mut stream, &filedata, &path)? as u64;
        rcvd += filedata.len() as u64;
        trace!(
            "received {} bytes of {}.  chunk start: {}, end: {}",
            rcvd,
            size,
            start,
            end
        );

        // invoke callback if present, with status info.
        //        if let Some(cb) = callback {
        let b_continue = callback(path, size, bytes_written, filedata.len() as u64);
        if !b_continue {
            trace!("download cancelled by callback");
            break;
        }
    }

    // Close may generate an error, so we do a flush/sync first to detect such.
    // see https://github.com/rust-lang/rust/pull/63410#issuecomment-519965351
    let fh = bufwriter_into_inner(stream, &path)?;
    file_sync_all(&fh, &path)?;

    Ok(bytes_written as u64)
}

// syncs file to filesystem.
fn file_sync_all(f: &fs::File, path: &Path) -> ApiResult<()> {
    f.sync_all().map_err(|err| {
        Error::FileSystemError(format!(
            "Error syncing file: \"{}\" {:?}",
            path.display(),
            err
        ))
    })
}

// causes BufWriter to flush() file.
fn bufwriter_into_inner<W: Write>(w: BufWriter<W>, path: &Path) -> ApiResult<W> {
    w.into_inner().map_err(|err| {
        Error::FileSystemError(format!(
            "Error flushing file: \"{}\" {}",
            path.display(),
            err.to_string()
        ))
    })
}

// Writes data to a file/stream.
fn stream_write(writer: &mut dyn Write, data: &[u8], path: &Path) -> ApiResult<usize> {
    writer.write(&data).map_err(|err| {
        Error::FileSystemError(format!(
            "Error writing to file: \"{}\" {:?}",
            path.display(),
            err
        ))
    })
}

// Creates a file, ready for writing.
fn file_create(path: &Path) -> ApiResult<fs::File> {
    fs::File::create(path).map_err(|err| {
        Error::FileSystemError(format!(
            "Couldn't create file: \"{}\" {:?}",
            path.display(),
            err
        ))
    })
}

// create all directories in path if possible.
fn create_dir_all(dir_path: &Path) -> ApiResult<()> {
    if dir_path.is_file() {
        return Err(Error::FileSystemError(format!(
            "cannot overwrite non-directory '{}' with a directory.",
            dir_path.display()
        )));
    }
    fs::create_dir_all(&dir_path).map_err(|err| {
        Error::FileSystemError(format!(
            "Couldn't create path: \"{}\" {:?}",
            dir_path.display(),
            err
        ))
    })
}

/// # Get Unpublished ImmutableData
/// Get unpublished immutable data blobs from the network.
///
async fn files_get_unpublished_immutable(
    _safe: &Safe,
    _url: &str,
    _range: Range,
) -> ApiResult<Vec<u8>> {
    unimplemented!();
}

/// # Get Published or Unpublished ImmutableData
/// Get immutable data blobs from the network.
///
pub async fn files_get_immutable(safe: &Safe, url: &str, range: Range) -> ApiResult<Vec<u8>> {
    match XorUrlEncoder::from_url(&url)?.data_type() {
        SafeDataType::PublishedImmutableData => {
            safe.files_get_published_immutable(&url, range).await
        }
        SafeDataType::UnpublishedImmutableData => {
            files_get_unpublished_immutable(&safe, &url, range).await
        }
        _ => Err(Error::InvalidInput(
            "URL target is not immutable data.".to_string(),
        )),
    }
}
