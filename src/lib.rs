use std::{
    collections::HashMap,
    fmt::Display,
    sync::{
        atomic::{AtomicU8, Ordering},
        RwLock,
    },
    thread::ThreadId,
};

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

impl Colorize {
    fn use_color(&self) -> bool {
        match self {
            Colorize::BlackWhite => false,
            Colorize::Color => true,
            Colorize::ColorIf(stream) => supports_color(*stream),
        }
    }
}

impl Format {
    /// Creates a blank `Format` that prints without colors and no thread names
    pub fn new() -> Self {
        Self {
            colorize: Colorize::BlackWhite,
            color_threads: false,
            thread_names: false,
        }
    }

    /// Enable printing with colors if the given stream supports it.
    pub fn color_if_supported(mut self, stream: Stream) -> Self {
        self.colorize = Colorize::ColorIf(stream);
        self
    }

    /// Force enable colors
    pub fn force_colors(mut self) -> Self {
        self.colorize = Colorize::Color;
        self
    }

    /// Print thread names/id
    pub fn log_thread_names(mut self) -> Self {
        self.thread_names = true;
        self
    }

    /// Give each thread its own color on their printed names
    pub fn uniquely_color_threads(mut self) -> Self {
        self.color_threads = true;
        self.log_thread_names()
    }

    pub fn callback(
        self,
    ) -> impl Fn(fern::FormatCallback<'_>, &std::fmt::Arguments<'_>, &log::Record<'_>)
    {
        let use_color = self.colorize.use_color();
        let now = Time::new();
        let thread_name =
            ThreadName::new(use_color && self.color_threads, self.thread_names);

        move |out, message, record| {
            let msg = Message::new(use_color, record.level(), message);
            let level = Level::new(record.level(), use_color);

            out.finish(format_args!(
                "{}{}{} {}:{}",
                now,
                thread_name,
                level,
                record.target(),
                msg,
            ))
        }
    }
}

// TODO: organize into modules

struct Message<'a> {
    colorize: bool,
    level: log::Level,
    message: &'a std::fmt::Arguments<'a>,
}

impl<'a> Message<'a> {
    fn new(
        colorize: bool,
        level: log::Level,
        message: &'a std::fmt::Arguments<'a>,
    ) -> Self {
        Self {
            colorize,
            level,
            message,
        }
    }
}

impl<'a> Display for Message<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let style = if self.colorize {
            level_style(self.level)
        } else {
            Style::new()
        };

        write!(f, " {}", self.message.style(style))
    }
}

struct Time {
    offset: UtcOffset,
}

impl Time {
    fn new() -> Self {
        let offset = match UtcOffset::current_local_offset() {
            Ok(offset) => offset,
            Err(e) => {
                eprintln!("Failed to get the current UTC offset: {e:?}");
                UtcOffset::UTC
            }
        };
        Self { offset }
    }
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

struct ThreadName {
    colorize: bool,
    print: bool,
    thread_colors: RwLock<HashMap<ThreadId, Style>>,
    index: AtomicU8,
}

impl ThreadName {
    fn new(colorize: bool, print: bool) -> Self {
        let thread_colors = RwLock::new(HashMap::<ThreadId, Style>::new());
        let index = 0.into();
        Self {
            colorize,
            print,
            thread_colors,
            index,
        }
    }
}

impl Display for ThreadName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if !self.print {
            return Ok(());
        }

        let cur = std::thread::current();
        let thread_style = if self.colorize {
            let id = cur.id();
            match {
                let thread_colors = self.thread_colors.read().unwrap();
                thread_colors.get(&id).copied()
            } {
                Some(style) => style,
                None => {
                    let mut thread_colors = self.thread_colors.write().unwrap();
                    if let Some(style) = thread_colors.get(&id).copied() {
                        style
                    } else {
                        let i = self.index.fetch_add(1, Ordering::SeqCst);
                        let style = gen_color(i);
                        thread_colors.insert(id, style);
                        style
                    }
                }
            }
        } else {
            Style::new()
        };

        if let Some(name) = cur.name() {
            write!(f, " {}", format_args!("({})", name).style(thread_style))?;
        } else {
            write!(
                f,
                " {}",
                format_args!("({})", threadid_as_u64(cur.id())).style(thread_style)
            )?;
        }
        Ok(())
    }
}

struct Level {
    level: log::Level,
    use_color: bool,
}

impl Level {
    fn new(level: log::Level, use_color: bool) -> Self {
        Self { level, use_color }
    }
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

fn gen_color(i: u8) -> Style {
    const BOLD: u8 = 2;
    const COLOR: u8 = 7;
    const ITALIC: u8 = 2;
    let total = BOLD * COLOR * ITALIC;

    let style = Style::new();

    let i = i % total;
    let total = total / ITALIC;
    let style = match i / total {
        0 => style,
        1 => style.italic(),
        _ => unreachable!(),
    };

    let i = i % total;
    let total = total / BOLD;
    let style = match i / total {
        0 => style,
        1 => style.bold(),
        _ => unreachable!(),
    };

    let i = i % total;
    let total = total / COLOR;
    let style = match i / total {
        0 => style.bright_white(),
        1 => style.bright_blue(),
        2 => style.bright_yellow(),
        3 => style.bright_cyan(),
        4 => style.bright_purple(),
        5 => style.bright_green(),
        6 => style.bright_red(),
        _ => unreachable!(),
    };

    style
}
