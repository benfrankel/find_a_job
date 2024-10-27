use std::{collections::HashMap, fmt::Display, time::Duration};

use html_escape::decode_html_entities;
use regex::Regex;
use serde::{Deserialize, Serialize};
use thirtyfour::{
    error::{WebDriverError, WebDriverResult},
    prelude::{ElementQueryable as _, ElementWaitable as _},
    By, WebDriver, WebElement,
};
use tiny_bail::prelude::*;
use url::Url;

use crate::job::Job;

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct JobSource {
    pub name: String,
    url: Url,
    /// An optional iframe index to parse within.
    #[serde(default)]
    iframe: Option<u16>,
    /// A sequence of sub-DOMs to enter to get to the meat.
    #[serde(default)]
    sub_doms: Vec<SubDom>,
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
    /// An optional regex to capture the job company.
    #[serde(with = "serde_regex", default)]
    job_company_re: Option<Regex>,
    /// A regex to capture the job title.
    #[serde(with = "serde_regex")]
    job_title_re: Regex,
    /// A regex to capture the job URL.
    #[serde(with = "serde_regex")]
    job_url_re: Regex,
    /// An optional CSS selector to close a popup before going to the next page.
    #[serde(default)]
    close_popup: Option<String>,
    /// An optional CSS selector to navigate to the next page.
    #[serde(default)]
    next_page: Option<String>,
}

impl Display for JobSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl JobSource {
    pub async fn scrape(&self, driver: &WebDriver) -> WebDriverResult<HashMap<Url, Job>> {
        let mut jobs = HashMap::new();

        let mut url = self.url.clone();
        for page in 0.. {
            // Load the next page.
            log::debug!("[{}] Page {}: {}", self.name, page, url);
            driver.goto(url.as_str()).await?;
            if let Some(css) = &self.wait_for {
                log::debug!("[{}] Page {}: Waiting for {}", self.name, page, css);
                driver.query(By::Css(css)).first().await?;
            }

            // Find the root element.
            let mut root = driver.query(By::Css("*")).nowait().first().await?;
            if !self.sub_doms.is_empty() {
                driver.enter_default_frame().await?;
                for sub_dom in &self.sub_doms {
                    root = sub_dom.enter(&driver, &root).await?;
                }
            }
            let page_html = root.outer_html().await?;

            // Parse jobs from page HTML.
            let prev_num_jobs = jobs.len();
            jobs.extend(self.parse_page(&page_html));
            log::debug!(
                "[{}] Page {}: Found {} jobs ({} total)",
                self.name,
                page,
                jobs.len() - prev_num_jobs,
                jobs.len(),
            );

            // Go to the next page.
            let next_page = bq!(self.next_page.as_ref());
            if let Some(css) = &self.close_popup {
                if let Ok(elem) = root.query(By::Css(css)).nowait().first().await {
                    if let Ok(true) = elem.is_clickable().await {
                        // This is `next_page.scroll_into_view()` but with instant scrolling.
                        driver.execute(r#"arguments[0].scrollIntoView({block: "center", inline: "center", behavior: "instant"});"#, vec![elem.to_json()?]).await?;
                        elem.click().await?;
                    }
                }
            }
            let next_page = bq!(root.query(By::Css(next_page)).nowait().first().await);
            log::debug!("[{}] Page {}: Next page...", self.name, page);
            let old_url = driver.current_url().await?;
            next_page.wait_until().clickable().await?;
            // This is `next_page.scroll_into_view()` but with instant scrolling.
            driver.execute(r#"arguments[0].scrollIntoView({block: "center", inline: "center", behavior: "instant"});"#, vec![next_page.to_json()?]).await?;
            next_page.click().await?;
            for i in 0..80 {
                tokio::time::sleep(Duration::from_millis(100)).await;
                url = driver.current_url().await?;
                if url != old_url {
                    break;
                }
                if i == 79 {
                    return Err(WebDriverError::Timeout("waiting for next page".to_string()));
                }
            }
        }

        Ok(jobs)
    }

    // TODO: Return `Result`.
    /// Extracts a collection of jobs from HTML.
    fn parse_page(&self, page_html: &str) -> HashMap<Url, Job> {
        let mut jobs = HashMap::new();

        // Determine the slice of HTML that contains the list of jobs.
        let start = self
            .start_re
            .as_ref()
            .and_then(|x| x.find(page_html))
            .map(|x| x.end())
            .unwrap_or_default();
        let end = self
            .end_re
            .as_ref()
            .and_then(|x| x.find(&page_html[start..]))
            .map(|x| start + x.start())
            .unwrap_or(page_html.len());

        // Split the slice into individual jobs.
        for job_html in self.next_job_re.split(&page_html[start..end]).skip(1) {
            // Extract the job's company.
            let company = if let Some(job_company_re) = &self.job_company_re {
                let captures = cq!(job_company_re.captures(job_html));
                let raw_company = c!(captures.get(1)).as_str();
                let raw_company = decode_html_entities(raw_company);
                raw_company.trim().to_string()
            } else {
                self.name.clone()
            };

            // Extract the job's title.
            let captures = cq!(self.job_title_re.captures(job_html));
            let raw_title = c!(captures.get(1)).as_str();
            let raw_title = decode_html_entities(raw_title);
            let raw_title = raw_title.trim();

            // Extract the job's URL.
            let captures = cq!(self.job_url_re.captures(job_html));
            let raw_url = c!(captures.get(1)).as_str();
            let raw_url = decode_html_entities(raw_url);
            let url = c!(self.url.join(&raw_url));

            // Save the job by its URL.
            jobs.insert(
                url,
                Job::new(raw_title)
                    .with_source(&self.name)
                    .with_company(company),
            );
        }

        jobs
    }
}

#[derive(Serialize, Deserialize, Debug)]
enum SubDom {
    Frame(String),
    Shadow(String),
}

impl SubDom {
    async fn enter(&self, driver: &WebDriver, root: &WebElement) -> WebDriverResult<WebElement> {
        match self {
            SubDom::Frame(css) => {
                root.query(By::Css(css))
                    .nowait()
                    .first()
                    .await?
                    .enter_frame()
                    .await?;
                driver.query(By::Css("*")).nowait().first().await
            }
            SubDom::Shadow(css) => {
                root.query(By::Css(css))
                    .nowait()
                    .first()
                    .await?
                    .get_shadow_root()
                    .await
            }
        }
    }
}
