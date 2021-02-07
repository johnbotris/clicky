use log::LevelFilter::{self, *};

pub fn init_logging(quiet: i8, verbose: i8) {
    match simple_logger::SimpleLogger::new()
        .with_level(get_log_level(quiet, verbose))
        .init()
    {
        Ok(_) => log::trace!("Logging initialized"),
        Err(e) => eprintln!("Failed to initialize logging: {}", e),
    }
}

fn get_log_level(quiet: i8, verbose: i8) -> LevelFilter {
    let default = 3;
    let level = (default + verbose - quiet).min(5).max(0);
    assert!(0 <= level && level <= 5);
    [Off, Error, Warn, Info, Debug, Trace][level as usize]
}
