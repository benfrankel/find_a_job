use find_a_job::Bot;

fn main() {
    let mut bot = Bot::new();
    bot.init();
    bot.scrape_job_boards();
    bot.save_jobs();
}
