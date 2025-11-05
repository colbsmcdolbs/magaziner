mod epub;
mod fetch;
mod parser;

use anyhow::Result;
use clap::Parser;
use epub::build_epub;
use fetch::fetch_html_body;
use parser::{extract_article_content, extract_article_links};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    url: String,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let url = args.url;

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

    build_epub(&title, articles, &css_sheet, &image_uri)?;
    println!("Epub Completed");
    Ok(())
}
