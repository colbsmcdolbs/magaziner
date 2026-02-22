use crate::progress::Progress;
use anyhow::Result;
use reqwest::blocking::Client;
use reqwest::header::{HeaderMap, HeaderValue, COOKIE};
use scraper::Html;
use std::fs::File;
use std::io::copy;
use std::{thread, time::Duration};

pub fn make_client(cookie: Option<&str>) -> Result<Client> {
    let mut headers = HeaderMap::new();
    if let Some(cookie_str) = cookie
        && !cookie_str.is_empty()
    {
        headers.insert(COOKIE, HeaderValue::from_str(cookie_str)?);
    }
    Ok(Client::builder().default_headers(headers).build()?)
}

pub fn fetch_html_body(
    client: &Client,
    url: &str,
    delay: &u64,
    progress: &Progress,
) -> Result<Html> {
    let body = fetch_html_raw(client, url, delay, progress)?;
    Ok(Html::parse_document(&body))
}

pub fn fetch_html_raw(
    client: &Client,
    url: &str,
    delay: &u64,
    progress: &Progress,
) -> Result<String> {
    progress.verbose(&format!("GET {}", url));
    thread::sleep(Duration::from_millis(*delay));
    let body = client.get(url).send()?.text()?;
    progress.verbose(&format!("{} bytes received", body.len()));
    Ok(body)
}

pub fn download_image(
    client: &Client,
    url: &str,
    output_path: &str,
    progress: &Progress,
) -> Result<()> {
    progress.verbose(&format!("Downloading image: {}", url));
    let response = client.get(url).send()?;
    let mut file = File::create(output_path)?;
    let mut content = std::io::Cursor::new(response.bytes()?);
    copy(&mut content, &mut file)?;
    Ok(())
}
