use anyhow::Result;
use reqwest::blocking;
use scraper::Html;
use std::{thread, time::Duration};


pub fn fetch_html_body(url: &str) -> Result<Html> {
    thread::sleep(Duration::from_secs(5));

    let body = blocking::get(url)?.text()?;
    Ok(Html::parse_document(&body))
}
