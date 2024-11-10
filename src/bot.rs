use std::{
    collections::HashMap,
    process::{Child, Command, Stdio},
    sync::Arc,
    time::Duration,
};

use chrono::Utc;
use colored::{Color, Colorize as _};
use thirtyfour::{
    common::config::WebDriverConfig, extensions::query::ElementPollerWithTimeout, prelude::*,
    AlertBehaviour,
};
use tiny_bail::prelude::*;

use crate::{job::Job, job_source::JobSource};

#[derive(Default)]
pub struct Bot {
    server: Option<Child>,
    pub driver: Option<WebDriver>,
    pub job_sources: Vec<JobSource>,
    pub jobs: HashMap<String, Job>,
}

impl Bot {
    const JOBS_FILE_PATH: &str = "data/jobs.ron";
    const JOBS_BACKUP_FILE_PATH: &str = "data/jobs.backup.ron";
    const JOB_SOURCES_FILE_PATH: &str = "data/job_sources.ron";

    pub fn new() -> Self {
        Self::default()
    }

    pub async fn init(&mut self) -> WebDriverResult<()> {
        self.init_helper(true).await
    }

    pub async fn init_no_headless(&mut self) -> WebDriverResult<()> {
        self.init_helper(false).await
    }

    async fn init_helper(&mut self, headless: bool) -> WebDriverResult<()> {
        assert!(self.server.is_none() && self.driver.is_none());

        // Spawn WebDriver server as a child process.
        let server = Command::new("geckodriver")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()?;
        std::thread::sleep(Duration::from_millis(100));

        // Connect to WebDriver server.
        let mut caps = DesiredCapabilities::firefox();
        if headless {
            caps.set_headless()?;
        }
        caps.set_unexpected_alert_behaviour(AlertBehaviour::Dismiss)?;
        let config = WebDriverConfig::builder()
            .poller(Arc::new(ElementPollerWithTimeout::new(
                Duration::from_secs(8),
                Duration::from_millis(100),
            )))
            .build()?;
        let driver = WebDriver::new_with_config("http://localhost:4444", caps, config).await?;

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
        self.load_job_sources();
    }

    pub fn save(&mut self) {
        self.save_jobs();
    }

    pub fn load_jobs(&mut self) {
        let jobs_str = r!(std::fs::read_to_string(Self::JOBS_FILE_PATH));
        self.jobs = r!(ron::from_str(&jobs_str));
    }

    // Re-parse jobs from their titles. Useful when parsing logic changes.
    pub fn fix_jobs(&mut self) {
        for job in self.jobs.values_mut() {
            job.reparse();
        }
    }

    pub fn load_job_sources(&mut self) {
        let job_sources_str = r!(std::fs::read_to_string(Self::JOB_SOURCES_FILE_PATH));
        self.job_sources = ron::from_str(&job_sources_str).unwrap();
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
        let now = Utc::now();
        for (_, job) in sorted(&self.jobs) {
            let age = (now - job.timestamp).num_days();
            // Ugly code makes pretty colors.
            println!(
                "{} {} {} {}",
                format!("{:>2} days ago", age.to_string().bold()).color(if age == 0 {
                    Color::Cyan
                } else if age < 7 {
                    Color::TrueColor {
                        r: 200,
                        g: 150,
                        b: 60,
                    }
                } else {
                    Color::Red
                }),
                format!("{:12}", job.company.chars().take(12).collect::<String>()),
                format!(
                    "{:64}",
                    job.to_string().chars().take(64).collect::<String>(),
                )
                .color(if job.score() > 0 {
                    Color::Green
                } else {
                    Color::Red
                }),
                format!("({})", job.url).italic().dimmed(),
            );
        }
    }

    pub async fn update_jobs(&mut self) {
        let mut jobs = HashMap::with_capacity(2 * self.jobs.len());
        for i in 0..self.job_sources.len() {
            jobs.extend(c!(self.scrape_job_source(i).await));
        }
        self.jobs = jobs;
    }

    pub async fn scrape_job_source(&self, idx: usize) -> WebDriverResult<HashMap<String, Job>> {
        // Scrape job source.
        let job_source = &self.job_sources[idx];
        let mut jobs = job_source.scrape(self.driver.as_ref().unwrap()).await?;

        // Fix timestamps of already-known jobs.
        for (id, job) in &mut jobs {
            if let Some(old) = self.jobs.get(id) {
                job.timestamp = old.timestamp;
                continue;
            }
        }

        // Log removed jobs.
        let now = Utc::now();
        for (id, job) in sorted(&self.jobs) {
            cq!(job.source == job_source.name && !jobs.contains_key(id));
            let age = (now - job.timestamp).num_days();
            log::info!(
                "{}[{}] Removed after {} days: {} ({})",
                job.prefix(),
                job.company,
                age,
                job,
                job.url,
            );
        }

        // Log added jobs.
        for (id, job) in sorted(&jobs) {
            cq!(!self.jobs.contains_key(id));
            log::info!(
                "{}[{}] New: {} ({})",
                job.prefix(),
                job.company,
                job,
                job.url,
            );
        }

        Ok(jobs)
    }
}

fn sorted(jobs: &HashMap<String, Job>) -> impl IntoIterator<Item = (&String, &Job)> {
    let mut ids = jobs.keys().collect::<Vec<_>>();
    let now = Utc::now();
    ids.sort_by_key(|&id| {
        let job = &jobs[id];
        let age = (now - job.timestamp).num_days() as i32;
        (
            job.score() > 0,
            age == 0,
            age < 7,
            job.score() - age,
            &job.company,
            &job.title,
        )
    });
    ids.into_iter().map(|id| (id, &jobs[id]))
}
