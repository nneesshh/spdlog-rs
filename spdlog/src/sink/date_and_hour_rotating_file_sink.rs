//! Provides a date and hour rotating file sink.

use std::{
    fs::File,
    io::{BufWriter, Write},
    path::{Path, PathBuf},
    time::SystemTime,
};

use chrono::prelude::*;

use crate::{
    sink::{helper, Sink},
    sync::*,
    utils, Error, Record, Result, StringBuf,
};

trait Rotator {
    #[allow(clippy::ptr_arg)]
    fn log(&self, record: &Record, string_buf: &StringBuf) -> Result<()>;
    fn flush(&self) -> Result<()>;
    fn drop_flush(&mut self) -> Result<()> {
        self.flush()
    }
}

struct RotatorTimePoint {
    base_path: PathBuf,
    inner: SpinMutex<RotatorTimePointInner>,
}

struct RotatorTimePointInner {
    file: BufWriter<File>,
    rotation_time_point: SystemTime,
}

/// A sink with a collection of files as the target, rotating according to the
/// rotation policy.
///
/// A service program that runs for a long time in an environment with limited
/// hard disk space may continue to write messages to the log file and
/// eventually run out of hard disk space. `DateAndHourRotatingFileSink` is designed for
/// such a usage scenario. It splits log messages into one or more log files
/// and may be configured to delete old log files automatically to save disk
/// space. The operation that splits log messages into multiple log files and
/// optionally creates and deletes log files is called a **rotation**. The
/// **rotation policy** determines when and how log files are created or
/// deleted, and how log messages are written to different log files.
///
/// # Parameters
///
/// A rotating file sink can be created with 3 parameters: the **base path**,
/// the **maximum number of log files**, and the **rotation policy**.
///
/// ## The Base Path
///
/// Each rotating file sink requires a **base path** which serves as a template
/// to form log file paths. You can set the base path with
/// [`DateAndHourRotatingFileSinkBuilder::base_path`] when building a rotating file sink.
/// Different rotation policy may use different file name patterns based on the
/// base path. For more information about the base path, see the documentation
/// of [`DateAndHourRotatingFileSinkBuilder::base_path`].
///
/// # Rotation Policy
///
/// The only policy is rotating file with both date and hour
///
/// # Examples
///
/// See [./examples] directory.
///
/// [./examples]: https://github.com/SpriteOvO/spdlog-rs/tree/main/spdlog/examples
pub struct DateAndHourRotatingFileSink {
    common_impl: helper::CommonImpl,
    rotator: RotatorTimePoint,
}

/// The builder of [`DateAndHourRotatingFileSink`].
#[doc = include_str!("../include/doc/generic-builder-note.md")]
/// # Examples
///
/// - Building a [`DateAndHourRotatingFileSink`].
///
///   ```no_run
///   use spdlog::sink::{DateAndHourRotatingFileSink};
///
///   # fn main() -> Result<(), spdlog::Error> {
///   let sink: DateAndHourRotatingFileSink = DateAndHourRotatingFileSink::builder()
///       .base_path("/path/to/base_log_file") // required
///       // .rotate_on_open(true) // optional, defaults to `false`
///       .build()?;
///   # Ok(()) }
///   ```
///
/// - If any required parameters are missing, a compile-time error will be
///   raised.
///
///   ```compile_fail,E0061
///   use spdlog::sink::{DateAndHourRotatingFileSink};
///
///   # fn main() -> Result<(), spdlog::Error> {
///   let sink: DateAndHourRotatingFileSink = DateAndHourRotatingFileSink::builder()
///       // .base_path("/path/to/base_log_file") // required
///       .rotate_on_open(true) // optional, defaults to `false`
///       .build()?;
///   # Ok(()) }
///   ```
///
///   ```compile_fail,E0061
///   use spdlog::sink::{DateAndHourRotatingFileSink};
///
///   # fn main() -> Result<(), spdlog::Error> {
///   let sink: DateAndHourRotatingFileSink = DateAndHourRotatingFileSink::builder()
///       .base_path("/path/to/base_log_file") // required
///       .rotate_on_open(true) // optional, defaults to `false`
///       .build()?;
///   # Ok(()) }
///   ```
pub struct DateAndHourRotatingFileSinkBuilder<ArgBP> {
    common_builder_impl: helper::CommonBuilderImpl,
    base_path: ArgBP,
    rotate_on_open: bool,
}

impl DateAndHourRotatingFileSink {
    /// Constructs a builder of `DateAndHourRotatingFileSink`.
    #[must_use]
    pub fn builder() -> DateAndHourRotatingFileSinkBuilder<()> {
        DateAndHourRotatingFileSinkBuilder {
            common_builder_impl: helper::CommonBuilderImpl::new(),
            base_path: (),
            rotate_on_open: false,
        }
    }
}

impl Sink for DateAndHourRotatingFileSink {
    fn log(&self, record: &Record) -> Result<()> {
        if !self.should_log(record.level()) {
            return Ok(());
        }

        let mut string_buf = StringBuf::new();
        self.common_impl
            .formatter
            .read()
            .format(record, &mut string_buf)?;

        self.rotator.log(record, &string_buf)
    }

    fn flush(&self) -> Result<()> {
        self.rotator.flush()
    }

    helper::common_impl!(@Sink: common_impl);
}

impl Drop for DateAndHourRotatingFileSink {
    fn drop(&mut self) {
        if let Err(err) = self.rotator.drop_flush() {
            self.common_impl
                .non_returnable_error("DateAndHourRotatingFileSink", err)
        }
    }
}

impl RotatorTimePoint {
    fn new(base_path: PathBuf, truncate: bool) -> Result<Self> {
        let now = SystemTime::now();
        let file_path = Self::calc_file_path(base_path.as_path(), now);
        let file = utils::open_file(file_path, truncate)?;

        let inner = RotatorTimePointInner {
            file: BufWriter::new(file),
            rotation_time_point: Self::next_rotation_time_point(now),
        };

        let res = Self {
            base_path,
            inner: SpinMutex::new(inner),
        };

        Ok(res)
    }

    // a little expensive, should only be called when rotation is needed or in
    // constructor.
    #[must_use]
    fn next_rotation_time_point(now: SystemTime) -> SystemTime {
        let now: DateTime<Utc> = now.into();

        let mut rotation_time = now
            .with_minute(0)
            .unwrap()
            .with_second(0)
            .unwrap()
            .with_nanosecond(0)
            .unwrap();

        rotation_time = rotation_time
            .checked_add_signed(chrono::Duration::hours(1))
            .unwrap();

        rotation_time.into()
    }

    #[must_use]
    fn calc_file_path(base_path: impl AsRef<Path>, system_time: SystemTime) -> PathBuf {
        let base_path = base_path.as_ref();
        let mut file_name = base_path.file_stem().unwrap().to_owned();
        let externsion = base_path.extension();

        let local_time: chrono::DateTime<chrono::Local> = system_time.into();

        // append yyyymmdd to base_path
        let date_path = format!(
            "{:04}{:02}{:02}",
            local_time.year(),
            local_time.month(),
            local_time.day()
        );

        // append hour to filename
        file_name.push(format!("_{:02}", local_time.hour()));

        let mut path = base_path.to_owned();
        path.pop();
        path.push(date_path.as_str());
        path.push(file_name);

        if let Some(externsion) = externsion {
            path.set_extension(externsion);
        }
        path
    }
}

impl Rotator for RotatorTimePoint {
    fn log(&self, record: &Record, string_buf: &StringBuf) -> Result<()> {
        let mut inner = self.inner.lock();

        let record_time = record.time();
        let should_rotate = record_time >= inner.rotation_time_point;

        if should_rotate {
            let file_path = Some(Self::calc_file_path(&self.base_path, record_time));
            inner.file = BufWriter::new(utils::open_file(file_path.as_ref().unwrap(), true)?);
            inner.rotation_time_point = Self::next_rotation_time_point(record_time);
        }

        inner
            .file
            .write_all(string_buf.as_bytes())
            .map_err(Error::WriteRecord)?;

        Ok(())
    }

    fn flush(&self) -> Result<()> {
        self.inner.lock().file.flush().map_err(Error::FlushBuffer)
    }
}

impl<ArgBP> DateAndHourRotatingFileSinkBuilder<ArgBP> {
    /// Specifies the base path of the log file.
    ///
    /// The path needs to be suffixed with an extension, if you expect the
    /// rotated eventual file names to contain the extension.
    ///
    /// If there is an extension, the different rotation policies will insert
    /// relevant information in the front of the extension. If there is not
    /// an extension, it will be appended to the end.
    ///
    /// Supposes the given base path is `/path/to/base_file.log`, the eventual
    /// file names may look like the following:
    ///
    /// - `/path/to/base_file_1.log`
    /// - `/path/to/base_file_2.log`
    /// - `/path/to/base_file_2022-03-23.log`
    /// - `/path/to/base_file_2022-03-24.log`
    /// - `/path/to/base_file_2022-03-23_03.log`
    /// - `/path/to/base_file_2022-03-23_04.log`
    ///
    /// This parameter is **required**.
    #[must_use]
    pub fn base_path<P>(self, base_path: P) -> DateAndHourRotatingFileSinkBuilder<PathBuf>
    where
        P: Into<PathBuf>,
    {
        DateAndHourRotatingFileSinkBuilder {
            common_builder_impl: self.common_builder_impl,
            base_path: base_path.into(),
            rotate_on_open: self.rotate_on_open,
        }
    }

    /// Specifies whether to rotate files once when constructing
    /// `DateAndHourRotatingFileSink`.
    ///
    /// It may truncate the contents of the existing file if the parameter is `true`
    /// , since the file name is a time point and not an index.
    ///
    /// This parameter is **optional**, and defaults to `false`.
    #[must_use]
    pub fn rotate_on_open(mut self, rotate_on_open: bool) -> Self {
        self.rotate_on_open = rotate_on_open;
        self
    }

    helper::common_impl!(@SinkBuilder: common_builder_impl);
}

impl DateAndHourRotatingFileSinkBuilder<PathBuf> {
    /// Builds a [`DateAndHourRotatingFileSink`].
    ///
    /// # Errors
    ///
    /// If an error occurs opening the file, [`Error::CreateDirectory`] or [`Error::OpenFile`]
    /// will be returned.
    pub fn build(self) -> Result<DateAndHourRotatingFileSink> {
        let rotator = RotatorTimePoint::new(self.base_path, self.rotate_on_open)?;

        let res = DateAndHourRotatingFileSink {
            common_impl: helper::CommonImpl::from_builder(self.common_builder_impl),
            rotator,
        };

        Ok(res)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{prelude::*, test_utils::*, Level, Record};

    static BASE_LOGS_PATH: Lazy<PathBuf> = Lazy::new(|| {
        let path = TEST_LOGS_PATH.join("rotating_file_sink");
        fs::create_dir_all(&path).unwrap();
        path
    });

    mod policy_time_point {
        use super::*;

        static LOGS_PATH: Lazy<PathBuf> = Lazy::new(|| {
            let path = BASE_LOGS_PATH.join("policy_time_point");
            fs::create_dir_all(&path).unwrap();
            path
        });

        #[test]
        fn calc_file_path() {
            let system_time = Local.with_ymd_and_hms(2012, 3, 4, 5, 6, 7).unwrap().into();

            let calc_date_and_hour = |base_path| {
                RotatorTimePoint::calc_file_path(base_path, system_time)
                    .to_str()
                    .unwrap()
                    .to_string()
            };

            #[cfg(not(windows))]
            let run = || {
                assert_eq!(
                    calc_date_and_hour("/tmp/test.log"),
                    "/tmp/20120304/test_05.log"
                );
                assert_eq!(calc_date_and_hour("/tmp/test"), "/tmp/20120304/test_05");
            };

            #[cfg(windows)]
            #[rustfmt::skip]
            let run = || {
                assert_eq!(calc_date_and_hour("D:\\tmp\\test.txt"), "D:\\tmp\\20120304\\test_05.txt");
                assert_eq!(calc_date_and_hour("D:\\tmp\\test"), "D:\\tmp\\20120304\\test_05");
            };

            run();
        }

        #[test]
        fn rotate() {
            let build = |rotate_on_open| {
                fs::remove_dir_all(LOGS_PATH.as_path()).unwrap();
                fs::create_dir(LOGS_PATH.as_path()).unwrap();

                let hourly_sink = DateAndHourRotatingFileSink::builder()
                    .base_path(LOGS_PATH.join("hourly.log"))
                    .rotate_on_open(rotate_on_open)
                    .build()
                    .unwrap();

                let local_time_now = Local::now();
                let daily_sink = DateAndHourRotatingFileSink::builder()
                    .base_path(LOGS_PATH.join("daily.log"))
                    .rotate_on_open(rotate_on_open)
                    .build()
                    .unwrap();

                let sinks: [Arc<dyn Sink>; 2] = [Arc::new(hourly_sink), Arc::new(daily_sink)];
                let logger = test_logger_builder().sinks(sinks).build().unwrap();
                logger.set_level_filter(LevelFilter::All);
                logger
            };

            let exist_files = |file_name_prefix| {
                let paths = fs::read_dir(LOGS_PATH.clone()).unwrap();

                paths.fold(0_usize, |count, entry| {
                    if entry
                        .unwrap()
                        .file_name()
                        .to_string_lossy()
                        .starts_with(file_name_prefix)
                    {
                        count + 1
                    } else {
                        count
                    }
                })
            };

            let exist_hourly_files = || exist_files("hourly");
            let exist_daily_files = || exist_files("daily");

            const SECOND_1: Duration = Duration::from_secs(1);
            const HOUR_1: Duration = Duration::from_secs(60 * 60);
            const DAY_1: Duration = Duration::from_secs(60 * 60 * 24);

            {
                let logger = build(true);
                let mut record = Record::new(Level::Info, "test log message");
                let initial_time = record.time();

                assert_eq!(exist_hourly_files(), 1);
                assert_eq!(exist_daily_files(), 1);

                logger.log(&record);
                assert_eq!(exist_hourly_files(), 1);
                assert_eq!(exist_daily_files(), 1);

                record.set_time(record.time() + HOUR_1 + SECOND_1);
                logger.log(&record);
                assert_eq!(exist_hourly_files(), 2);
                assert_eq!(exist_daily_files(), 1);

                record.set_time(record.time() + HOUR_1 + SECOND_1);
                logger.log(&record);
                assert_eq!(exist_hourly_files(), 3);
                assert_eq!(exist_daily_files(), 1);

                record.set_time(record.time() + SECOND_1);
                logger.log(&record);
                assert_eq!(exist_hourly_files(), 3);
                assert_eq!(exist_daily_files(), 1);

                record.set_time(initial_time + DAY_1 + SECOND_1);
                logger.log(&record);
                assert_eq!(exist_hourly_files(), 4);
                assert_eq!(exist_daily_files(), 2);
            }
        }
    }

    #[test]
    fn test_builder_optional_params() {
        // workaround for the missing `no_run` attribute
        let _ = || {
            let _: Result<DateAndHourRotatingFileSink> = DateAndHourRotatingFileSink::builder()
                .base_path("/path/to/base_log_file")
                // .rotate_on_open(true)
                .build();

            let _: Result<DateAndHourRotatingFileSink> = DateAndHourRotatingFileSink::builder()
                .base_path("/path/to/base_log_file")
                // .rotate_on_open(true)
                .build();

            let _: Result<DateAndHourRotatingFileSink> = DateAndHourRotatingFileSink::builder()
                .base_path("/path/to/base_log_file")
                .rotate_on_open(true)
                .build();

            let _: Result<DateAndHourRotatingFileSink> = DateAndHourRotatingFileSink::builder()
                .base_path("/path/to/base_log_file")
                .rotate_on_open(true)
                .build();
        };
    }
}
