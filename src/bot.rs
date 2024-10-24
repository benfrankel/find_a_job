use std::{collections::HashMap, time::Duration};

use reqwest::{blocking::Client, header};
use tiny_bail::prelude::*;
use url::Url;

use crate::{job::Job, job_board::JobBoard};

#[derive(Default)]
pub struct Bot {
    pub client: Client,
    pub job_boards: Vec<JobBoard>,
    pub jobs: HashMap<Url, Job>,
}

impl Bot {
    const JOBS_FILE_PATH: &str = "data/jobs.ron";
    const JOBS_BACKUP_FILE_PATH: &str = "data/jobs.ron.backup";
    const JOB_BOARDS_FILE_PATH: &str = "data/job_boards.ron";

    pub fn new() -> Self {
        let headers = header::HeaderMap::from_iter([
            (
                header::ACCEPT,
                header::HeaderValue::from_static("text/html,application/xhtml+xml,application/xml;q=0.9"),
            ),
            (
                header::ACCEPT_ENCODING,
                header::HeaderValue::from_static("gzip, deflate, br"),
            ),
            (
                header::ACCEPT_LANGUAGE,
                header::HeaderValue::from_static("en-US,en;q=0.5"),
            ),
            (
                header::CONNECTION,
                header::HeaderValue::from_static("keep-alive"),
            ),
            (
                header::DNT,
                header::HeaderValue::from_static("1"),
            ),
            (
                header::REFERER,
                header::HeaderValue::from_static("http://www.google.com/"),
            ),
            (
                header::UPGRADE_INSECURE_REQUESTS,
                header::HeaderValue::from_static("1"),
            ),
            (
                header::USER_AGENT,
                header::HeaderValue::from_static("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/129.0.0.0 Safari/537.3"),
            )
        ]);

        Self {
            client: Client::builder()
                .default_headers(headers)
                .timeout(Duration::from_secs(8))
                .build()
                .unwrap(),
            job_boards: Vec::new(),
            jobs: HashMap::new(),
        }
    }

    pub fn init(&mut self) {
        init_logger();
        self.load();
    }

    pub fn load(&mut self) {
        self.load_jobs();
        self.load_job_boards();
    }

    pub fn save(&mut self) {
        self.save_jobs();
    }

    pub fn scrape(&mut self) {
        let mut new_jobs = HashMap::new();

        for job_board in &self.job_boards {
            // Scrape job board.
            let mut jobs = job_board.scrape(&self.client);
            log::info!("[{}] Scraped {} jobs", job_board, jobs.len());

            // Log added jobs and update existing jobs.
            for (url, job) in &mut jobs {
                if let Some(old) = self.jobs.get(&url) {
                    job.first_seen = old.first_seen;
                    continue;
                }
                log::log!(
                    job.log_level(),
                    "[{}] Added: \"{}\" at {}",
                    job_board,
                    job,
                    url,
                );
            }

            // Log removed jobs.
            for (url, job) in &self.jobs {
                cq!(job.source == job_board.name && !jobs.contains_key(url));
                log::log!(
                    job.log_level(),
                    "[{}] Removed: \"{}\" at {}",
                    job_board,
                    job,
                    url,
                );
            }

            new_jobs.extend(jobs);
        }

        self.jobs = new_jobs;
    }

    pub fn log_jobs(&self) {
        // TODO: Sort by job source.
        for (url, job) in &self.jobs {
            // TODO: Use first_seen to include "from X days ago" in the message.
            // TODO: Sort by first_seen so new jobs are at the top.
            log::log!(
                job.log_level(),
                "[{}] Job: \"{}\" at {}",
                job.source,
                job,
                url,
            );
        }
    }

    pub fn load_jobs(&mut self) {
        let jobs_str = r!(std::fs::read_to_string(Self::JOBS_FILE_PATH));
        self.jobs = r!(ron::from_str(&jobs_str));
    }

    pub fn save_jobs(&self) {
        r!(std::fs::copy(
            Self::JOBS_FILE_PATH,
            Self::JOBS_BACKUP_FILE_PATH,
        ));
        let jobs_str = r!(ron::to_string(&self.jobs));
        r!(std::fs::write(Self::JOBS_FILE_PATH, jobs_str));
    }

    pub fn load_job_boards(&mut self) {
        let job_boards_str = r!(std::fs::read_to_string(Self::JOB_BOARDS_FILE_PATH));
        self.job_boards = r!(ron::from_str(&job_boards_str));
    }
}

fn init_logger() {
    pretty_env_logger::formatted_timed_builder()
        .filter_level(log::LevelFilter::Info)
        .parse_default_env()
        .init();
}
