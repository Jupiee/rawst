use std::sync::LazyLock;

use fern::colors::Color;
use fern::colors::ColoredLevelConfig;

use crate::cli::args::Arguments;
use crate::core::config::Config;

fn default_colors() -> ColoredLevelConfig {
    ColoredLevelConfig::default()
        .trace(Color::BrightMagenta)
        .debug(Color::BrightCyan)
}
static COLORS: LazyLock<ColoredLevelConfig> = LazyLock::new(default_colors);

fn no_colors() -> ColoredLevelConfig {
    ColoredLevelConfig::new()
        .trace(Color::Black)
        .debug(Color::Black)
        .info(Color::Black)
        .warn(Color::Black)
        .error(Color::Black)
}
static NO_COLORS: LazyLock<ColoredLevelConfig> = LazyLock::new(no_colors);

pub fn init(config: &Config, args: &Arguments) -> Result<(), fern::InitError> {
    let log_file_path = config.log_file_path();

    println!("Initialising logger ({:?})...", log_file_path);

    let colors: &ColoredLevelConfig = match args.color.color {
        concolor_clap::ColorChoice::Never => &NO_COLORS,
        _ => &COLORS,
    };

    fern::Dispatch::new()
        .chain(
            // Log file
            fern::Dispatch::new()
                .level(args.log_verbosity.unwrap_or(log::LevelFilter::Debug))
                .format(|out, message, record| {
                    out.finish(format_args!(
                        "[{} {} {}] {}",
                        humantime::format_rfc3339_seconds(std::time::SystemTime::now()),
                        record.level(),
                        record.target(),
                        message
                    ))
                })
                .chain(fern::log_file(log_file_path)?),
        )
        .chain(
            // stdout
            fern::Dispatch::new()
                .level(args.verbosity.unwrap_or(log::LevelFilter::Warn))
                .format(|out, message, record| {
                    out.finish(format_args!(
                        "[{} {} {}] {}",
                        humantime::format_rfc3339_seconds(std::time::SystemTime::now()),
                        colors.color(record.level()),
                        record.target(),
                        message
                    ))
                })
                .chain(std::io::stdout()),
        )
        .apply()?;

    Ok(())
}
