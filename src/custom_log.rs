use colored::{ColoredString, Colorize};
use env_logger::fmt::Formatter;
use log::{Level, Record};
use spinoff::{spinners, Color, Spinner};
use std::io::Write;

fn with_prefix(msg: String) -> String {
    msg.replace("\n", "\n |  ")
}

pub fn success_message(msg: String) -> String {
    format!("{} {}", "[✔]".green(), with_prefix(msg).green())
}

fn error_message(msg: String) -> String {
    format!("{} {}", "[!]".red(), with_prefix(msg).red())
}

pub fn custom_log_format(buf: &mut Formatter, record: &Record) -> std::io::Result<()> {
    let level = record.level();

    let level_prefix = match level {
        Level::Error => "[!]".red(),
        Level::Warn => "[⚠️]".yellow(),
        Level::Info => ColoredString::from("[*]"),
        _ => ColoredString::from(""),
    };

    let msg_with_prefixes: String = with_prefix(record.args().to_string());

    let message = match level {
        Level::Error => msg_with_prefixes.red(),
        Level::Warn => msg_with_prefixes.yellow(),
        _ => ColoredString::from(msg_with_prefixes),
    };

    writeln!(buf, "{} {}", level_prefix, message)
}

#[macro_export]
macro_rules! success {
    ($($arg:tt)*) => {{
        use $crate::custom_log::success_message;
        let message = format!($($arg)*);
        println!("{}", success_message(message));
    }};
}

pub struct OlSpinner {
    spinner: Spinner,
}

impl OlSpinner {
    pub fn new(message: &str) -> Self {
        let spinner = Spinner::new(spinners::Aesthetic, message.to_owned(), Color::White);
        OlSpinner { spinner }
    }

    pub fn stop_with_success(&mut self, message: &str) {
        self.spinner
            .stop_with_message(success_message(message.to_owned()).as_str());
    }

    pub fn stop_with_error(&mut self, message: &str) {
        self.spinner
            .stop_with_message(error_message(message.to_owned()).as_str());
    }
}
