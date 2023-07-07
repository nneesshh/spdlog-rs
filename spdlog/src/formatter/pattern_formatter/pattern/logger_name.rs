use crate::{
    formatter::pattern_formatter::{Pattern, PatternContext},
    Record, StringBuf,
};

/// A pattern that writes the logger's name into the output. Example:
/// `my-logger`.
#[derive(Clone, Default)]
pub struct LoggerName;

impl Pattern for LoggerName {
    fn format(
        &self,
        record: &Record,
        dest: &mut StringBuf,
        _ctx: &mut PatternContext,
    ) -> crate::Result<()> {
        dest.push_str(record.logger_name().unwrap_or(""));
        Ok(())
    }
}
