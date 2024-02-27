use std::{collections::HashMap, fmt::Display, thread::ThreadId};

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

        let use_color = match self.colorize {
            Colorize::BlackWhite => false,
            Colorize::Color => true,
            Colorize::ColorIf(stream) => supports_color(stream),
        };

        let thread_colors = HashMap::<ThreadId, ()>::new();

        move |out, message, record| {
            let now = Time { offset: utc_offset };
            let style = if use_color {
                level_style(record.level())
            } else {
                Style::new()
            };

            let thread_name = ThreadName { format: &self };
            let level = Level {
                level: record.level(),
                use_color,
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

struct Time {
    offset: UtcOffset,
}

impl Display for Time {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        const DATE_FORMAT: &[time::format_description::FormatItem<'_>] = time::macros::format_description!(
            "[hour repr:24]:[minute]:[second].[subsecond digits:6]"
        );

        let now = OffsetDateTime::now_utc()
            .to_offset(self.offset)
            .time()
            // TODO: figure out how to format this directly into the formatter using
            // format_into
            .format(DATE_FORMAT)
            .unwrap_or_else(|_| "??:??:??.??????".into());
        write!(f, "{}", now)
    }
}

struct ThreadName<'a> {
    format: &'a Format,
}

impl Display for ThreadName<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.format.thread_names {
            let cur = std::thread::current();
            if let Some(name) = cur.name() {
                write!(f, " ({})", name)?;
            } else {
                write!(f, " ({})", threadid_as_u64(cur.id()))?;
            }
        }
        Ok(())
    }
}

struct Level {
    level: log::Level,
    use_color: bool,
}

impl Display for Level {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if !self.use_color {
            write!(f, " [{}]", self.level)?;
        }

        Ok(())
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

// https://github.com/rust-lang/rust/issues/67939
// TODO: use the stabilzed function when and if it is stabilized
// TODO: error handling?
fn threadid_as_u64(id: ThreadId) -> u64 {
    let string = format!("{:?}", id);
    let string = string.strip_prefix("ThreadId(").unwrap();
    let string = string.strip_suffix(")").unwrap();
    string.parse().unwrap()
}
