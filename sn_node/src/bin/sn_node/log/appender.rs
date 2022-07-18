use file_rotate::{
    compression::Compression,
    suffix::{AppendTimestamp, FileLimit},
    ContentLimit, FileRotate,
};
use std::{fmt::Debug, io, io::Write, path::Path};

/// `FileRotateAppender` is a `tracing_appender` with extra logrotate features:
///  - most recent logfile name re-used to support following (e.g. 'tail -f=logfile')
///  - numbered rotation (logfile.1, logfile.2 etc)
///  - limit logfile by size, lines or time
///  - limit maximum number of logfiles
///  - optional compression of rotated logfiles
//
// The above functionality is provided using crate file_rotation
pub struct FileRotateAppender {
    writer: FileRotate<AppendTimestamp>,
}

impl FileRotateAppender {
    /// Create default `FileRotateAppender`
    pub fn new(directory: impl AsRef<Path>, file_name_prefix: impl AsRef<Path>) -> Self {
        let log_directory = directory.as_ref().to_str().unwrap();
        let log_filename_prefix = file_name_prefix.as_ref().to_str().unwrap();
        let path = Path::new(&log_directory).join(&log_filename_prefix);
        let writer = FileRotate::new(
            &Path::new(&path),
            AppendTimestamp::default(FileLimit::MaxFiles(9)),
            ContentLimit::Bytes(10 * 1024 * 1024),
            Compression::OnRotate(1),
        );

        Self { writer }
    }

    /// Create `FileRotateAppender` using parameters
    pub fn make_rotate_appender(
        directory: impl AsRef<Path>,
        file_name_prefix: impl AsRef<Path>,
        file_limit: AppendTimestamp,
        max_log_size: ContentLimit,
        compression: Compression,
    ) -> Self {
        let log_directory = directory.as_ref().to_str().unwrap();
        let log_filename_prefix = file_name_prefix.as_ref().to_str().unwrap();
        let path = Path::new(&log_directory).join(&log_filename_prefix);
        let writer = FileRotate::new(&Path::new(&path), file_limit, max_log_size, compression);

        Self { writer }
    }
}

impl Write for FileRotateAppender {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.writer.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.writer.flush()
    }
}

use std::fmt;

impl Debug for FileRotateAppender {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FileRotateAppender").finish()
    }
}
