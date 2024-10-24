use std::{
    collections::HashMap,
    process::{Child, Command, Stdio},
};

use chrono::{Datelike, Utc};
use thirtyfour::prelude::*;
use tiny_bail::prelude::*;
use url::Url;

use crate::{job::Job, job_board::JobBoard};

#[derive(Default)]
pub struct Bot {
    server: Option<Child>,
    pub driver: Option<WebDriver>,
    pub job_boards: Vec<JobBoard>,
    pub jobs: HashMap<Url, Job>,
}

impl Bot {
    const JOBS_FILE_PATH: &str = "data/jobs.ron";
    const JOBS_BACKUP_FILE_PATH: &str = "data/jobs.ron.backup";
    const JOB_BOARDS_FILE_PATH: &str = "data/job_boards.ron";

    pub fn new() -> Self {
        Self::default()
    }

    pub async fn init(&mut self) -> WebDriverResult<()> {
        assert!(self.server.is_none() && self.driver.is_none());

        let server = Command::new("geckodriver")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()?;

        let mut caps = DesiredCapabilities::firefox();
        caps.set_headless()?;
        let driver = WebDriver::new("http://localhost:4444", caps).await?;

        self.server = Some(server);
        self.driver = Some(driver);

        Ok(())
    }

    pub async fn quit(self) -> WebDriverResult<()> {
        self.driver.unwrap().quit().await?;
        self.server.unwrap().kill()?;
        Ok(())
    }

    pub fn load(&mut self) {
        self.load_jobs();
        self.load_job_boards();
    }

    pub fn save(&mut self) {
        self.save_jobs();
    }

    pub fn load_jobs(&mut self) {
        let jobs_str = r!(std::fs::read_to_string(Self::JOBS_FILE_PATH));
        self.jobs = r!(ron::from_str(&jobs_str));
    }

    // Re-parse jobs from their titles. Useful for when parsing logic changes.
    pub fn fix_jobs(&mut self) {
        for job in self.jobs.values_mut() {
            let timestamp = job.timestamp;
            *job = Job::new(&job.title).with_source(&job.source);
            job.timestamp = timestamp;
        }
    }

    pub fn load_job_boards(&mut self) {
        let job_boards_str = r!(std::fs::read_to_string(Self::JOB_BOARDS_FILE_PATH));
        self.job_boards = r!(ron::from_str(&job_boards_str));
    }

    pub fn save_jobs(&self) {
        r!(std::fs::copy(
            Self::JOBS_FILE_PATH,
            Self::JOBS_BACKUP_FILE_PATH,
        ));
        let jobs_str = r!(ron::to_string(&self.jobs));
        r!(std::fs::write(Self::JOBS_FILE_PATH, jobs_str));
    }

    pub fn list_jobs(&self) {
        let today = Utc::now().num_days_from_ce();
        for (url, job) in sorted(&self.jobs) {
            println!(
                "Seen {} days ago: {}[{}] {} ({})",
                today - job.timestamp.num_days_from_ce(),
                job.prefix(),
                job.source,
                job,
                url
            );
        }
    }

    pub async fn update_jobs(&mut self) {
        let mut jobs = HashMap::with_capacity(2 * self.jobs.len());
        for i in 0..self.job_boards.len() {
            jobs.extend(c!(self.scrape_job_board(i).await));
        }
        self.jobs = jobs;
    }

    pub async fn scrape_job_board(&self, idx: usize) -> WebDriverResult<HashMap<Url, Job>> {
        // Scrape job board.
        let job_board = &self.job_boards[idx];
        let mut jobs = job_board.scrape2(self.driver.as_ref().unwrap()).await?;

        // Fix timestamps of already-known jobs.
        for (url, job) in &mut jobs {
            if let Some(old) = self.jobs.get(&url) {
                job.timestamp = old.timestamp;
                continue;
            }
        }

        // Log removed jobs.
        for (url, job) in sorted(&self.jobs) {
            cq!(job.source == job_board.name && !jobs.contains_key(url));
            log::info!("{}[{}] Missing: {} ({})", job.prefix(), job_board, job, url);
        }

        // Log added jobs.
        for (url, job) in sorted(&jobs) {
            cq!(!self.jobs.contains_key(url));
            log::info!("{}[{}] New: {} ({})", job.prefix(), job_board, job, url);
        }

        Ok(jobs)
    }
}

fn sorted(jobs: &HashMap<Url, Job>) -> impl IntoIterator<Item = (&Url, &Job)> {
    let mut urls = jobs.keys().collect::<Vec<_>>();
    urls.sort_by_key(|&url| {
        let job = &jobs[url];
        (
            job.is_good(),
            &job.source,
            job.timestamp.timestamp(),
            &job.title,
        )
    });
    urls.into_iter().map(|url| (url, &jobs[url]))
}
