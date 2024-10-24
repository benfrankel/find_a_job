mod bot;
mod job;
mod job_board;

pub use bot::Bot;

pub fn init_logger(default_level: log::LevelFilter) {
    pretty_env_logger::formatted_timed_builder()
        .filter_level(default_level)
        .parse_default_env()
        .init();
}
