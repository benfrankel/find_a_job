use find_a_job::Bot;

fn main() {
    let mut bot = Bot::new();
    bot.init();
    bot.scrape();
    bot.save();
}
