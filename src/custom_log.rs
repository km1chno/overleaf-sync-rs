use colored::Colorize;
use env_logger::fmt::Formatter;
use log::{Level, Record};
use std::io::Write;

pub fn custom_log_format(buf: &mut Formatter, record: &Record) -> std::io::Result<()> {
    let level = record.level();

    let level_prefix = match level {
        Level::Error => "[❌]".red(),
        Level::Warn => "[⚠️]".yellow(),
        Level::Info => "[*]".bright_white(),
        _ => "".clear(),
    };

    let msg_with_prefixes: String = record.args().to_string().replace("\n", "\n | ");

    let message = match level {
        Level::Error => msg_with_prefixes.red(),
        Level::Warn => msg_with_prefixes.yellow(),
        _ => msg_with_prefixes.bright_white(),
    };

    writeln!(buf, "{} {}", level_prefix, message)
}

#[macro_export]
macro_rules! success {
    ($($arg:tt)*) => {{
        use colored::Colorize;
        let message = format!($($arg)*).replace("\n", "\n | ");
        println!("{} {}", "[✔]".green(), message.green());
    }};
}
