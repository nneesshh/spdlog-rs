use std::{fmt::Write, marker::PhantomData};

use crate::{
    formatter::{
        local_time_cacher::LOCAL_TIME_CACHER,
        pattern_formatter::{Pattern, PatternContext},
    },
    Error, Record, StringBuf,
};

/// A pattern that writes the abbreviated weekday name of log records into the
/// output. Example: `Mon`, `Tue`.
#[derive(Clone, Default)]
pub struct AbbrWeekdayName;

impl Pattern for AbbrWeekdayName {
    fn format(
        &self,
        record: &Record,
        dest: &mut StringBuf,
        _ctx: &mut PatternContext,
    ) -> crate::Result<()> {
        let name = LOCAL_TIME_CACHER
            .lock()
            .get(record.time())
            .weekday_name()
            .short;

        dest.push_str(name);
        Ok(())
    }
}

/// A pattern that writes the weekday name of log records into the output.
/// Example: `Monday`, `Tuesday`.
#[derive(Clone, Default)]
pub struct WeekdayName;

impl Pattern for WeekdayName {
    fn format(
        &self,
        record: &Record,
        dest: &mut StringBuf,
        _ctx: &mut PatternContext,
    ) -> crate::Result<()> {
        let name = LOCAL_TIME_CACHER
            .lock()
            .get(record.time())
            .weekday_name()
            .full;

        dest.push_str(name);
        Ok(())
    }
}

/// A pattern that writes the abbreviated month name of log records into the
/// output. Example: `Jan`, `Feb`.
#[derive(Clone, Default)]
pub struct AbbrMonthName;

impl Pattern for AbbrMonthName {
    fn format(
        &self,
        record: &Record,
        dest: &mut StringBuf,
        _ctx: &mut PatternContext,
    ) -> crate::Result<()> {
        let name = LOCAL_TIME_CACHER
            .lock()
            .get(record.time())
            .month_name()
            .short;

        dest.push_str(name);
        Ok(())
    }
}

/// A pattern that writes the month name of log records into the output.
/// Example: `January`, `February`.
#[derive(Clone, Default)]
pub struct MonthName;

impl Pattern for MonthName {
    fn format(
        &self,
        record: &Record,
        dest: &mut StringBuf,
        _ctx: &mut PatternContext,
    ) -> crate::Result<()> {
        let name = LOCAL_TIME_CACHER
            .lock()
            .get(record.time())
            .month_name()
            .full;

        dest.push_str(name);
        Ok(())
    }
}

/// A pattern that writes the full date time of log records into the output.
/// Example: `Thu Aug 23 15:35:46 2014`.
#[derive(Clone, Default)]
pub struct FullDateTime;

impl Pattern for FullDateTime {
    fn format(
        &self,
        record: &Record,
        dest: &mut StringBuf,
        _ctx: &mut PatternContext,
    ) -> crate::Result<()> {
        let (
            abbr_weekday_name,
            abbr_month_name,
            day_str,
            hour_str,
            minute_str,
            second_str,
            year_str,
        ) = {
            let mut time_cacher_lock = LOCAL_TIME_CACHER.lock();
            let cached_time = time_cacher_lock.get(record.time());
            (
                cached_time.weekday_name().short,
                cached_time.month_name().short,
                cached_time.day_str(),
                cached_time.hour_str(),
                cached_time.minute_str(),
                cached_time.second_str(),
                cached_time.year_str(),
            )
        };

        (|| {
            dest.push_str(abbr_weekday_name);
            dest.push(' ');
            dest.push_str(abbr_month_name);
            dest.push(' ');
            dest.push_str(&day_str);
            dest.push(' ');
            dest.push_str(&hour_str);
            dest.push(':');
            dest.push_str(&minute_str);
            dest.push(':');
            dest.push_str(&second_str);
            dest.push(' ');
            dest.push_str(&year_str)
        })();
        Ok(())
    }
}

/// A pattern that writes the short year of log records into the output.
/// Examples: `22`, `20`.
#[derive(Clone, Default)]
pub struct ShortYear;

impl Pattern for ShortYear {
    fn format(
        &self,
        record: &Record,
        dest: &mut StringBuf,
        _ctx: &mut PatternContext,
    ) -> crate::Result<()> {
        let year_short_str = LOCAL_TIME_CACHER.lock().get(record.time()).year_short_str();
        dest.push_str(&year_short_str);
        Ok(())
    }
}

/// A pattern that writes the year of log records into the output.
/// Examples: `2022`, `2021`.
#[derive(Clone, Default)]
pub struct Year;

impl Pattern for Year {
    fn format(
        &self,
        record: &Record,
        dest: &mut StringBuf,
        _ctx: &mut PatternContext,
    ) -> crate::Result<()> {
        let year_str = LOCAL_TIME_CACHER.lock().get(record.time()).year_str();
        dest.push_str(&year_str);
        Ok(())
    }
}

/// A pattern that writes the date of log records in `YYYY-MM-DD` format (ISO
/// 8601) into the output. Examples: `2022-04-01`, `2021-12-31`.
#[derive(Clone, Default)]
pub struct Date;

impl Pattern for Date {
    fn format(
        &self,
        record: &Record,
        dest: &mut StringBuf,
        _ctx: &mut PatternContext,
    ) -> crate::Result<()> {
        let (month_str, day_str, year_str) = {
            let mut local_cacher_lock = LOCAL_TIME_CACHER.lock();
            let cached_time = local_cacher_lock.get(record.time());
            (
                cached_time.month_str(),
                cached_time.day_str(),
                cached_time.year_str(),
            )
        };

        (|| {
            dest.push_str(&year_str);
            dest.push('-');
            dest.push_str(&month_str);
            dest.push('-');
            dest.push_str(&day_str)
        })();
        Ok(())
    }
}

/// A pattern that writes the short date of log records in `MM/DD/YY` format
/// into the output. Examples: `04/01/22`, `12/31/21`.
#[derive(Clone, Default)]
pub struct ShortDate;

impl Pattern for ShortDate {
    fn format(
        &self,
        record: &Record,
        dest: &mut StringBuf,
        _ctx: &mut PatternContext,
    ) -> crate::Result<()> {
        let (month_str, day_str, year_short_str) = {
            let mut local_cacher_lock = LOCAL_TIME_CACHER.lock();
            let cached_time = local_cacher_lock.get(record.time());
            (
                cached_time.month_str(),
                cached_time.day_str(),
                cached_time.year_short_str(),
            )
        };

        (|| {
            dest.push_str(&month_str);
            dest.push('/');
            dest.push_str(&day_str);
            dest.push('/');
            dest.push_str(&year_short_str)
        })();
        Ok(())
    }
}

/// A pattern that writes the month of log records into the output.
/// Examples: `01`, `12`.
#[derive(Clone, Default)]
pub struct Month;

impl Pattern for Month {
    fn format(
        &self,
        record: &Record,
        dest: &mut StringBuf,
        _ctx: &mut PatternContext,
    ) -> crate::Result<()> {
        let month_str = LOCAL_TIME_CACHER.lock().get(record.time()).month_str();
        dest.push_str(&month_str);
        Ok(())
    }
}

/// A pattern that writes the day of log records into the output.
/// Examples: `01`, `12`, `31`, `30`.
#[derive(Clone, Default)]
pub struct Day;

impl Pattern for Day {
    fn format(
        &self,
        record: &Record,
        dest: &mut StringBuf,
        _ctx: &mut PatternContext,
    ) -> crate::Result<()> {
        let day_str = LOCAL_TIME_CACHER.lock().get(record.time()).day_str();
        dest.push_str(&day_str);
        Ok(())
    }
}

/// A pattern that writes the hour of log records into the output. Examples:
/// `01`, `12`, `23`.
#[derive(Clone, Default)]
pub struct Hour;

impl Pattern for Hour {
    fn format(
        &self,
        record: &Record,
        dest: &mut StringBuf,
        _ctx: &mut PatternContext,
    ) -> crate::Result<()> {
        let hour_str = LOCAL_TIME_CACHER.lock().get(record.time()).hour_str();
        dest.push_str(&hour_str);
        Ok(())
    }
}

/// A pattern that writes the hour in 12-hour format of log records into the
/// output. Examples: `01`, `12`.
#[derive(Clone, Default)]
pub struct Hour12;

impl Pattern for Hour12 {
    fn format(
        &self,
        record: &Record,
        dest: &mut StringBuf,
        _ctx: &mut PatternContext,
    ) -> crate::Result<()> {
        let hour_12_str = LOCAL_TIME_CACHER.lock().get(record.time()).hour12_str();
        dest.push_str(&hour_12_str);
        Ok(())
    }
}

/// A pattern that writes the minute of log records into the output. Examples:
/// `00`, `05`, `59`.
#[derive(Clone, Default)]
pub struct Minute;

impl Pattern for Minute {
    fn format(
        &self,
        record: &Record,
        dest: &mut StringBuf,
        _ctx: &mut PatternContext,
    ) -> crate::Result<()> {
        let minute_str = LOCAL_TIME_CACHER.lock().get(record.time()).minute_str();
        dest.push_str(&minute_str);
        Ok(())
    }
}

/// A pattern that writes the second of log records into the output. Examples:
/// `00`, `05`, `59`.
#[derive(Clone, Default)]
pub struct Second;

impl Pattern for Second {
    fn format(
        &self,
        record: &Record,
        dest: &mut StringBuf,
        _ctx: &mut PatternContext,
    ) -> crate::Result<()> {
        let second_str = LOCAL_TIME_CACHER.lock().get(record.time()).second_str();
        dest.push_str(&second_str);
        Ok(())
    }
}

/// A pattern that writes the millisecond part within a second of the timestamp
/// of a log record into the output. Example: `231`.
#[derive(Clone, Default)]
pub struct Millisecond {
    /// This field prevents users from creating `Millisecond` objects
    /// literally.
    _phantom: PhantomData<()>,
}

impl Pattern for Millisecond {
    fn format(
        &self,
        record: &Record,
        dest: &mut StringBuf,
        _ctx: &mut PatternContext,
    ) -> crate::Result<()> {
        let millisecond = LOCAL_TIME_CACHER.lock().get(record.time()).millisecond();
        write!(dest, "{:03}", millisecond).map_err(Error::FormatRecord)
    }
}

/// A pattern that writes the microsecond part within a second of the timestamp
/// of a log record into the output. Example: `372152`.
#[derive(Clone, Default)]
pub struct Microsecond;

impl Pattern for Microsecond {
    fn format(
        &self,
        record: &Record,
        dest: &mut StringBuf,
        _ctx: &mut PatternContext,
    ) -> crate::Result<()> {
        let nanosecond = LOCAL_TIME_CACHER.lock().get(record.time()).nanosecond();
        write!(dest, "{:06}", nanosecond / 1_000).map_err(Error::FormatRecord)
    }
}

/// A pattern that writes the nanosecond part within a second of the timestamp
/// of a log record into the output. Example: `482930154`.
#[derive(Clone, Default)]
pub struct Nanosecond;

impl Pattern for Nanosecond {
    fn format(
        &self,
        record: &Record,
        dest: &mut StringBuf,
        _ctx: &mut PatternContext,
    ) -> crate::Result<()> {
        let nanosecond = LOCAL_TIME_CACHER.lock().get(record.time()).nanosecond();
        write!(dest, "{:09}", nanosecond).map_err(Error::FormatRecord)
    }
}

/// A pattern that writes "AM" or "PM" into the output according to the
/// timestamp of a log record. Example: `AM`, `PM`.
#[derive(Clone, Default)]
pub struct AmPm;

impl Pattern for AmPm {
    fn format(
        &self,
        record: &Record,
        dest: &mut StringBuf,
        _ctx: &mut PatternContext,
    ) -> crate::Result<()> {
        let am_pm_str = LOCAL_TIME_CACHER.lock().get(record.time()).am_pm_str();
        dest.push_str(am_pm_str);
        Ok(())
    }
}

/// A pattern that writes the time of log records in 12-hour format into the
/// output. Examples: `02:55:02 PM`.
#[derive(Clone, Default)]
pub struct Time12;

impl Pattern for Time12 {
    fn format(
        &self,
        record: &Record,
        dest: &mut StringBuf,
        _ctx: &mut PatternContext,
    ) -> crate::Result<()> {
        let (hour_str, minute_str, second_str, am_pm_str) = {
            let mut time_cacher_lock = LOCAL_TIME_CACHER.lock();
            let cached_time = time_cacher_lock.get(record.time());
            (
                cached_time.hour12_str(),
                cached_time.minute_str(),
                cached_time.second_str(),
                cached_time.am_pm_str(),
            )
        };

        (|| {
            dest.push_str(&hour_str);
            dest.push(':');
            dest.push_str(&minute_str);
            dest.push(':');
            dest.push_str(&second_str);
            dest.push(' ');
            dest.push_str(am_pm_str)
        })();
        Ok(())
    }
}

/// A pattern that writes the short time of log records into the output.
/// Examples: `22:28`, `09:53`.
#[derive(Clone, Default)]
pub struct ShortTime;

impl Pattern for ShortTime {
    fn format(
        &self,
        record: &Record,
        dest: &mut StringBuf,
        _ctx: &mut PatternContext,
    ) -> crate::Result<()> {
        let (hour_str, minute_str) = {
            let mut time_cacher_lock = LOCAL_TIME_CACHER.lock();
            let cached_time = time_cacher_lock.get(record.time());
            (cached_time.hour_str(), cached_time.minute_str())
        };

        (|| {
            dest.push_str(&hour_str);
            dest.push(':');
            dest.push_str(&minute_str)
        })();
        Ok(())
    }
}

/// A pattern that writes the time of log records into the output. Examples:
/// `22:28:02`, `09:53:41`.
#[derive(Clone, Default)]
pub struct Time;

impl Pattern for Time {
    fn format(
        &self,
        record: &Record,
        dest: &mut StringBuf,
        _ctx: &mut PatternContext,
    ) -> crate::Result<()> {
        let (hour_str, minute_str, second_str) = {
            let mut time_cacher_lock = LOCAL_TIME_CACHER.lock();
            let cached_time = time_cacher_lock.get(record.time());
            (
                cached_time.hour_str(),
                cached_time.minute_str(),
                cached_time.second_str(),
            )
        };

        (|| {
            dest.push_str(&hour_str);
            dest.push(':');
            dest.push_str(&minute_str);
            dest.push(':');
            dest.push_str(&second_str)
        })();
        Ok(())
    }
}

/// A pattern that writes the timezone offset of log records into the output.
/// Examples: `+08:00`, `+00:00`, `-06:00`.
#[derive(Clone, Default)]
pub struct TzOffset;

impl Pattern for TzOffset {
    fn format(
        &self,
        record: &Record,
        dest: &mut StringBuf,
        _ctx: &mut PatternContext,
    ) -> crate::Result<()> {
        let tz_offset_str = LOCAL_TIME_CACHER.lock().get(record.time()).tz_offset_str();
        dest.push_str(&tz_offset_str);
        Ok(())
    }
}

/// A pattern that writes the unix timestamp of log records into the output.
/// Examples: `1528834770`.
#[derive(Clone, Default)]
pub struct UnixTimestamp;

impl Pattern for UnixTimestamp {
    fn format(
        &self,
        record: &Record,
        dest: &mut StringBuf,
        _ctx: &mut PatternContext,
    ) -> crate::Result<()> {
        let unix_timestamp_str = LOCAL_TIME_CACHER
            .lock()
            .get(record.time())
            .unix_timestamp_str();
        dest.push_str(&unix_timestamp_str);
        Ok(())
    }
}
