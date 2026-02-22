use crate::progress::Progress;
use anyhow::Result;
use reqwest::blocking;
use scraper::Html;
use std::fs::File;
use std::io::copy;
use std::{thread, time::Duration};

pub fn fetch_html_body(url: &str, delay: &u64, progress: &Progress) -> Result<Html> {
    let body = fetch_html_raw(url, delay, progress)?;
    Ok(Html::parse_document(&body))
}

pub fn fetch_html_raw(url: &str, delay: &u64, progress: &Progress) -> Result<String> {
    progress.verbose(&format!("GET {}", url));
    thread::sleep(Duration::from_millis(*delay));
    let body = blocking::get(url)?.text()?;
    progress.verbose(&format!("{} bytes received", body.len()));
    Ok(body)
}

pub fn download_image(url: &str, output_path: &str, progress: &Progress) -> Result<()> {
    progress.verbose(&format!("Downloading image: {}", url));
    let response = reqwest::blocking::get(url)?;
    let mut file = File::create(output_path)?;
    let mut content = std::io::Cursor::new(response.bytes()?);
    copy(&mut content, &mut file)?;
    Ok(())
}
