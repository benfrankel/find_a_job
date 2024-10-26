use find_a_job::{init_logger, Bot};
use thirtyfour::error::WebDriverResult;

#[tokio::main]
async fn main() -> WebDriverResult<()> {
    init_logger(log::LevelFilter::Debug);
    let mut bot = Bot::new();
    bot.init().await?;
    bot.load();
    bot.update_jobs().await;
    bot.save();
    bot.quit().await
}
