use colored::{ColoredString, Colorize};
use env_logger::fmt::Formatter;
use log::{Level, Record};
use std::io::Write;

pub fn custom_log_format(buf: &mut Formatter, record: &Record) -> std::io::Result<()> {
    let level = record.level();

    let level_prefix = match level {
        Level::Error => "[!]".red(),
        Level::Warn => "[⚠️]".yellow(),
        Level::Info => ColoredString::from("[*]"),
        _ => ColoredString::from(""),
    };

    let msg_with_prefixes: String = record.args().to_string().replace("\n", "\n | ");

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
        use colored::Colorize;
        let message = format!($($arg)*).replace("\n", "\n |  ");
        println!("{} {}", "[✔]".green(), message.green());
    }};
}
