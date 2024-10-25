use std::{collections::HashMap, fmt::Display};

use html_escape::decode_html_entities;
use regex::Regex;
use serde::{Deserialize, Serialize};
use thirtyfour::{error::WebDriverResult, prelude::ElementQueryable as _, By, WebDriver};
use tiny_bail::prelude::*;
use url::Url;

use crate::job::Job;

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct JobBoard {
    pub name: String,
    url: Url,
    /// An optional CSS selector to wait for before parsing the HTML.
    #[serde(default)]
    wait_for: Option<String>,
    /// An optional regex to ignore some initial HTML.
    #[serde(with = "serde_regex", default)]
    start_re: Option<Regex>,
    /// An optional regex to ignore some final HTML.
    #[serde(with = "serde_regex", default)]
    end_re: Option<Regex>,
    /// A regex to jump to the next job in the list.
    #[serde(with = "serde_regex")]
    next_job_re: Regex,
    /// A regex to capture the nearest job title.
    #[serde(with = "serde_regex")]
    job_title_re: Regex,
    /// A regex to capture the nearest job URL.
    #[serde(with = "serde_regex")]
    job_url_re: Regex,
    /// An optional regex to capture the URL of the next page.
    #[serde(with = "serde_regex", default)]
    next_page_re: Option<Regex>,
}

impl Display for JobBoard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", &self.name)
    }
}

impl JobBoard {
    pub async fn scrape(&self, driver: &WebDriver) -> WebDriverResult<HashMap<Url, Job>> {
        let mut jobs = HashMap::new();

        let mut url = self.url.clone();
        for i in 0.. {
            // Make an HTTP request to get the current page.
            log::debug!("[{}] Page {}: Going to {}", self.name, i, url);
            driver.goto(url.as_str()).await?;

            // Wait for the page to be ready before reading its HTML.
            if let Some(css) = &self.wait_for {
                log::debug!("[{}] Page {}: Waiting for jobs...", self.name, i);
                driver.query(By::Css(css)).first().await?;
            }
            let page_html = driver.source().await?;

            // Extract a list of jobs and the URL to the next page from the HTML.
            let (new_jobs, new_url) = self.parse_page(&page_html);
            log::debug!("[{}] Page {}: Found {} jobs", self.name, i, new_jobs.len());
            jobs.extend(new_jobs);
            url = bq!(new_url);

            // TODO: Sleep between requests?
        }

        Ok(jobs)
    }

    // TODO: Return `Result`.
    fn parse_page(&self, page_html: &str) -> (HashMap<Url, Job>, Option<Url>) {
        let mut jobs = HashMap::new();

        // Parse jobs from the HTML.
        let start = self
            .start_re
            .as_ref()
            .and_then(|x| x.find(&page_html))
            .map(|x| x.end())
            .unwrap_or_default();
        let end = self
            .end_re
            .as_ref()
            .and_then(|x| x.find(&page_html[start..]))
            .map(|x| start + x.start())
            .unwrap_or(page_html.len());
        for job_html in self.next_job_re.split(&page_html[start..end]).skip(1) {
            let captures = c!(self.job_title_re.captures(job_html));
            let raw_title = c!(captures.get(1)).as_str().trim();
            let raw_title = decode_html_entities(raw_title);

            let captures = c!(self.job_url_re.captures(job_html));
            let raw_url = c!(captures.get(1)).as_str();
            let raw_url = decode_html_entities(raw_url);
            let url = c!(self.url.join(&raw_url));

            jobs.insert(url, Job::new(raw_title).with_source(&self.name));
        }

        // Find a URL to the next page if there is one.
        let next_page_url = self.next_page_re.as_ref().and_then(|x| {
            let captures = rq!(x.captures(page_html));
            let raw_url = r!(captures.get(1)).as_str();
            let raw_url = decode_html_entities(raw_url);
            let url = r!(self.url.join(&raw_url));
            Some(url)
        });

        (jobs, next_page_url)
    }
}
