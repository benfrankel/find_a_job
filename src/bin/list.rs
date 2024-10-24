use find_a_job::{init_logger, Bot};

#[tokio::main]
async fn main() {
    init_logger(log::LevelFilter::Info);
    let mut bot = Bot::new();
    bot.load_jobs();
    bot.list_jobs();
}
