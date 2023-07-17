use std::fmt::Write;

///
#[derive(Clone)]
pub struct CommlibFormatter {
    with_eol: bool,
}

impl CommlibFormatter {
    /// Constructs a `CommlibFormatter`.
    #[must_use]
    pub fn new() -> CommlibFormatter {
        CommlibFormatter { with_eol: true }
    }

    ///
    #[must_use]
    pub fn without_eol() -> Self {
        Self { with_eol: false }
    }

    fn format_impl(
        &self,
        record: &crate::Record,
        dest: &mut crate::StringBuf,
    ) -> Result<crate::formatter::FmtExtraInfo, std::fmt::Error> {
        cfg_if::cfg_if! {
            if #[cfg(not(feature = "flexible-string"))] {
                dest.reserve(crate::string_buf::RESERVE_SIZE);
            }
        }

        // Datetime
        {
            let mut local_time_cacher = crate::formatter::LOCAL_TIME_CACHER.lock();
            let time = local_time_cacher.get(record.time());
            dest.push_str("[");
            dest.push_str(&&time.full_iso_8601_str());
            dest.push_str("] ");
        }

        // Level
        let style_range_begin = dest.len();

        dest.push_str(record.level().as_str());

        let style_range_end = dest.len();
        dest.push_str(": ");

        // Payload
        dest.push_str(record.payload());

        // Source location
        if let Some(srcloc) = record.source_location() {
            dest.push_str(" [");
            dest.push_str(srcloc.file_name());
            dest.push_str(":");
            write!(dest, "{}", srcloc.line())?;
            dest.push(']');
        }

        // Thread id
        dest.push(' ');
        write!(dest, "{}", record.tid())?;

        if self.with_eol {
            dest.push_str(crate::EOL);
        }

        Ok(crate::formatter::FmtExtraInfo {
            style_range: Some(style_range_begin..style_range_end),
        })
    }
}

impl crate::formatter::Formatter for CommlibFormatter {
    fn format(
        &self,
        record: &crate::Record,
        dest: &mut crate::StringBuf,
    ) -> crate::Result<crate::formatter::FmtExtraInfo> {
        self.format_impl(record, dest)
            .map_err(crate::Error::FormatRecord)
    }

    fn clone_box(&self) -> Box<dyn crate::formatter::Formatter> {
        Box::new(self.clone())
    }
}

impl Default for CommlibFormatter {
    fn default() -> CommlibFormatter {
        CommlibFormatter::new()
    }
}

#[cfg(test)]
mod tests {
    use chrono::prelude::*;

    use super::*;
    use crate::formatter::Formatter;
    use crate::{Level, EOL, get_current_tid};

    #[test]
    fn format() {
        let record = crate::Record::new(Level::Warn, "test log content");
        let mut buf = crate::StringBuf::new();
        let extra_info = CommlibFormatter::new().format(&record, &mut buf).unwrap();

        let local_time: DateTime<Local> = record.time().into();

        //ISO 8601 / RFC 3339 %+: Same as %Y-%m-%dT%H:%M:%S.%9f%:z
        assert_eq!(
            format!(
                "[{}] warn: test log content {}{}",
                local_time.format("%Y-%m-%dT%H:%M:%S.%9f%:z"),
                get_current_tid(),
                EOL
            ),
            buf
        );
        assert_eq!(Some(38..42), extra_info.style_range());
    }
}
