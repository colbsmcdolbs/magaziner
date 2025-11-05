use anyhow::Result;
use reqwest::blocking;
use scraper::Html;
use std::fs::File;
use std::io::copy;
use std::{thread, time::Duration};

pub fn fetch_html_body(url: &str) -> Result<Html> {
    let body = fetch_html_raw(url)?;
    Ok(Html::parse_document(&body))
}

pub fn fetch_html_raw(url: &str) -> Result<String> {
    thread::sleep(Duration::from_secs(5));

    let body = blocking::get(url)?.text()?;
    Ok(body)
}

pub fn download_image(url: &str, output_path: &str) -> Result<()> {
    let response = reqwest::blocking::get(url)?;
    let mut file = File::create(output_path)?;
    let mut content = std::io::Cursor::new(response.bytes()?);
    copy(&mut content, &mut file)?;
    Ok(())
}
