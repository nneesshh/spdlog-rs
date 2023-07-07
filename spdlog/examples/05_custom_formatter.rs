fn main() {
    spdlog::info!("default format by `FullFormatter`");

    // There are two ways to set up custom formats

    // 1. This is the easiest and most convenient way
    use_pattern_formatter();

    // 2. When you need to implement more complex formatting logic
    impl_manually();

    // 3.
    use_pattern_formatter_by_async_sink();
}

fn use_pattern_formatter() {
    use spdlog::{
        formatter::{pattern, PatternFormatter},
        prelude::*,
    };

    // Building a pattern formatter with a pattern.
    //
    // The `pattern!` macro will parse the template string at compile-time.
    // See the documentation of `pattern!` macro for more usage.
    let new_formatter: Box<PatternFormatter<_>> = Box::new(PatternFormatter::new(pattern!(
        "{datetime} - {^{level}} - {payload}{eol}"
    )));

    // Setting the new formatter for each sink of the default logger.
    for sink in spdlog::default_logger().sinks() {
        sink.set_formatter(new_formatter.clone())
    }

    info!("format by `PatternFormatter`");
}

fn impl_manually() {
    use std::fmt::Write;

    use spdlog::{
        formatter::{FmtExtraInfo, Formatter},
        prelude::*,
        Record, StringBuf,
    };

    #[derive(Clone, Default)]
    struct MyFormatter;

    impl Formatter for MyFormatter {
        fn format(&self, record: &Record, dest: &mut StringBuf) -> spdlog::Result<FmtExtraInfo> {
            let style_range_begin: usize = dest.len();

            dest.write_str(&record.level().as_str().to_ascii_uppercase())
                .map_err(spdlog::Error::FormatRecord)?;

            let style_range_end: usize = dest.len();

            writeln!(dest, " {}", record.payload()).map_err(spdlog::Error::FormatRecord)?;

            Ok(FmtExtraInfo::builder()
                .style_range(style_range_begin..style_range_end)
                .build())
        }

        fn clone_box(&self) -> Box<dyn Formatter> {
            Box::new(self.clone())
        }
    }

    // Building a custom formatter.
    let new_formatter: Box<MyFormatter> = Box::default();

    // Setting the new formatter for each sink of the default logger.
    for sink in spdlog::default_logger().sinks() {
        sink.set_formatter(new_formatter.clone())
    }

    info!("format by `MyFormatter` (impl manually)");
}

fn use_pattern_formatter_by_async_sink() {
    use spdlog::{
        formatter::{pattern, PatternFormatter},
        prelude::*,
    };

    let sss_sink: std::sync::Arc<dyn spdlog::sink::Sink> = std::sync::Arc::new(
        spdlog::sink::StdStreamSink::builder()
            .std_stream(spdlog::sink::StdStream::Stdout)
            .style_mode(spdlog::terminal_style::StyleMode::Never)
            .build()
            .unwrap(),
    );

    let async_sink = std::sync::Arc::new(
        spdlog::sink::AsyncPoolSink::builder()
            .sink(sss_sink)
            .build()
            .unwrap(),
    );

    let logger: std::sync::Arc<spdlog::Logger> =
        std::sync::Arc::new(spdlog::Logger::builder().sink(async_sink).build().unwrap());

    spdlog::set_default_logger(logger);

    // Building a pattern formatter with a pattern.
    //
    // The `pattern!` macro will parse the template string at compile-time.
    // See the documentation of `pattern!` macro for more usage.
    let new_formatter: Box<PatternFormatter<_>> = Box::new(PatternFormatter::new(pattern!(
        "{datetime} - {^{level}} - {payload}{eol}{tid}"
    )));

    // Setting the new formatter for each sink of the default logger.
    for sink in spdlog::default_logger().sinks() {
        sink.set_formatter(new_formatter.clone())
    }

    info!("format by `PatternFormatter`");
    spdlog::default_logger().flush();

    for _ in 1.. {
        std::thread::sleep(std::time::Duration::from_millis(100));
        spdlog::default_logger().flush();
    }
}
