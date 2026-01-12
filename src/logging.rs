use colored::Colorize;
use std::io::IsTerminal;
use std::io::Write;

pub fn setup_log(level: Option<log::LevelFilter>) {
    let use_color = std::io::stdout().is_terminal();

    env_logger::Builder::from_default_env()
        .format(move |buf, record| {
            let level_text = record.level().to_string();
            let level = if use_color {
                match record.level() {
                    log::Level::Error => level_text.red(),
                    log::Level::Warn => level_text.yellow(),
                    log::Level::Info => level_text.green(),
                    log::Level::Debug => level_text.blue(),
                    log::Level::Trace => level_text.magenta(),
                }
            } else {
                level_text.normal()
            };
            writeln!(
                buf,
                "{:<5} [{}:{}] {}",
                level,
                record.file().unwrap_or("?"),
                record.line().unwrap_or(0),
                record.args()
            )
        })
        .filter_level(level.unwrap_or(log::LevelFilter::Info))
        .init();
}
