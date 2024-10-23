use std::{collections::HashMap, fmt::Display};

use html_escape::decode_html_entities;
use regex::Regex;
use serde::{Deserialize, Serialize};
use tiny_bail::prelude::*;
use url::Url;

use crate::job::Job;

#[derive(Serialize, Deserialize, Debug)]
pub struct JobBoard {
    pub name: String,
    url: Url,
    #[serde(with = "serde_regex")]
    next_job_re: Regex,
    #[serde(with = "serde_regex")]
    job_title_re: Regex,
    #[serde(with = "serde_regex")]
    job_url_re: Regex,
    #[serde(with = "serde_regex")]
    next_page_re: Option<Regex>,
}

impl Display for JobBoard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", &self.name)
    }
}

impl JobBoard {
    pub fn scrape(&self) -> HashMap<Url, Job> {
        let mut jobs = HashMap::new();

        let mut url = self.url.clone();
        loop {
            // Make an HTTP request to the current URL.
            let page_response = r!(reqwest::blocking::get(url));
            let page_html = r!(page_response.text());

            // Extract a list of jobs and a URL to the next page from the HTML.
            let (new_jobs, new_url) = self.parse_page(&page_html);
            jobs.extend(new_jobs);
            url = bq!(new_url);

            // TODO: Sleep between requests?
        }

        jobs
    }

    fn parse_page(&self, page_html: &str) -> (HashMap<Url, Job>, Option<Url>) {
        let mut jobs = HashMap::new();

        // Parse jobs from the HTML.
        for job_html in self.next_job_re.split(&page_html).skip(1) {
            let captures = c!(self.job_title_re.captures(job_html));
            let raw_title = c!(captures.get(1)).as_str();

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
