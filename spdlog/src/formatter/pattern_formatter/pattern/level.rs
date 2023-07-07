use crate::{
    formatter::pattern_formatter::{Pattern, PatternContext},
    Record, StringBuf,
};

/// A pattern that writes the level of a log record into the output. Examples:
/// `critical`, `error`, `warn`.
#[derive(Clone, Default)]
pub struct Level;

impl Pattern for Level {
    fn format(
        &self,
        record: &Record,
        dest: &mut StringBuf,
        _ctx: &mut PatternContext,
    ) -> crate::Result<()> {
        dest.push_str(record.level().as_str());
        Ok(())
    }
}

/// A pattern that writes the level in a shorter form of a log record into the
/// output. Examples: `C`, `E`, `W`.
#[derive(Clone, Default)]
pub struct ShortLevel;

impl Pattern for ShortLevel {
    fn format(
        &self,
        record: &Record,
        dest: &mut StringBuf,
        _ctx: &mut PatternContext,
    ) -> crate::Result<()> {
        dest.push_str(record.level().as_short_str());
        Ok(())
    }
}
