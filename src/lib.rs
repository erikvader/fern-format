use owo_colors::{OwoColorize, Style};
use time::{OffsetDateTime, UtcOffset};

pub use supports_color::Stream;

pub struct Format {
    /// How to decide if colors should be used at all
    colorize: Colorize,

    /// If thread names should be colored uniquely
    color_threads: bool,

    /// If thread names should be logged
    thread_names: bool,
}

enum Colorize {
    BlackWhite,
    Color,
    ColorIf(Stream),
}

impl Format {
    pub fn new() -> Self {
        Self {
            colorize: Colorize::BlackWhite,
            color_threads: false,
            thread_names: false,
        }
    }

    pub fn color_if_supported(mut self, stream: Stream) -> Self {
        self.colorize = Colorize::ColorIf(stream);
        self
    }

    pub fn force_colors(mut self) -> Self {
        self.colorize = Colorize::Color;
        self
    }

    pub fn log_thread_names(mut self) -> Self {
        self.thread_names = true;
        self
    }

    pub fn uniquely_color_threads(mut self) -> Self {
        self.color_threads = true;
        self.log_thread_names()
    }

    pub fn callback(
        self,
    ) -> impl Fn(fern::FormatCallback<'_>, &std::fmt::Arguments<'_>, &log::Record<'_>)
    {
        let utc_offset = match UtcOffset::current_local_offset() {
            Ok(offset) => offset,
            Err(e) => {
                eprintln!("Failed to get the current UTC offset: {e:?}");
                UtcOffset::UTC
            }
        };
        let date_format = time::macros::format_description!(
            "[hour repr:24]:[minute]:[second].[subsecond digits:6]"
        );

        let use_color = match self.colorize {
            Colorize::BlackWhite => false,
            Colorize::Color => true,
            Colorize::ColorIf(stream) => supports_color(stream),
        };

        move |out, message, record| {
            // TODO: transform each of these if-cases to a struct that implements Display
            // to avoid allocating so many temporary strings.
            let now = OffsetDateTime::now_utc()
                .to_offset(utc_offset)
                .time()
                .format(date_format)
                .unwrap_or_else(|_| "??:??:??.??????".into());

            let style = if use_color {
                level_style(record.level())
            } else {
                Style::new()
            };

            let thread_name = if self.thread_names {
                // TODO: print ID if name is not available
                format!(" ({})", std::thread::current().name().unwrap_or("??"))
            } else {
                String::new()
            };

            let level = if use_color {
                String::new()
            } else {
                format!(" [{}]", record.level())
            };

            out.finish(format_args!(
                "{}{}{} {}: {}",
                now,
                thread_name,
                level,
                record.target(),
                message.style(style),
            ))
        }
    }
}

fn supports_color(stream: Stream) -> bool {
    supports_color::on(stream).is_some_and(|support| support.has_basic)
}

/// Mimics the color style of journald
fn level_style(level: log::Level) -> Style {
    match level {
        log::Level::Error => Style::new().bright_red().bold(),
        log::Level::Warn => Style::new().bright_yellow().bold(),
        log::Level::Info => Style::new().bright_white().bold(),
        log::Level::Debug => Style::new().white(),
        log::Level::Trace => Style::new().dimmed(),
    }
}
