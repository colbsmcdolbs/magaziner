mod epub;
mod fetch;
mod parser;
mod validation;

use anyhow::Result;
use clap::Parser;
use epub::build_epub;
use fetch::fetch_html_body;
use parser::{extract_article_content, extract_article_links};
use std::path::PathBuf;
use validation::validate_lrb_url;

#[derive(Parser, Debug)]
#[command(
    name = "magaziner",
    version,
    about = "Generate epub files from Magazine archives",
    long_about = None
)]

struct Args {
    #[arg(
        short,
        long,
        value_parser = validate_lrb_url,
        help = "Magazine archive URL"
    )]
    url: String,

    #[arg(
        long,
        short,
        help = "Output directory for generated EPUBs (Ex: ./downloads",
        default_value = "."
    )]
    pub output: PathBuf,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let url = args.url;
    let output = args.output;

    if !output.exists() {
        std::fs::create_dir_all(&output).expect("Failed to create output directory");
    }

    let doc = fetch_html_body(&url)?;
    println!("Parsed HTML body");

    let (links, title, css_sheet, image_uri) = extract_article_links(&doc);
    println!("Extracted links");

    let mut articles = Vec::new();

    for link in links {
        let article_doc = fetch_html_body(&link)?;
        let (title, body) = extract_article_content(&article_doc);
        println!("Extracted article content");
        articles.push((title, body));
    }

    build_epub(&title, &output, articles, &css_sheet, &image_uri)?;
    println!("Epub Completed");
    Ok(())
}
