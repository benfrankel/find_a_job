use std::collections::HashMap;

use find_a_job::{init_logger, Bot, Job};

#[tokio::main]
async fn main() {
    init_logger(log::LevelFilter::Info);
    let mut bot = Bot::new();
    bot.load_jobs();
    bot.fix_jobs();
    bot.save_jobs();
}

/// A helper function for transitioning a job source off of using URLs as IDs.
#[allow(unused)]
fn url_to_id<'a>(jobs: impl IntoIterator<Item = &'a mut Job>, source: impl AsRef<str>) {
    let jobs_by_url_str = std::fs::read_to_string("data/jobs.backup.ron").unwrap();
    let jobs_by_url: HashMap<String, Job> = ron::from_str(&jobs_by_url_str).unwrap();
    for job in jobs.into_iter() {
        if !job.source.starts_with(source.as_ref()) {
            continue;
        }
        job.timestamp = jobs_by_url[job.url.as_str()].timestamp;
    }
}
