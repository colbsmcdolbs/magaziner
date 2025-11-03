mod fetch;
mod parser;
mod epub;

use anyhow::Result;
use fetch::fetch_html_body;
use parser::{extract_article_links, extract_article_content};
use epub::build_epub;

fn main() -> Result<()> {
    let url = "https://www.lrb.co.uk/the-paper/v43/n01";
    let doc = fetch_html_body(url)?;
    println!("Parsed HTML body");
    let (links, title) = extract_article_links(&doc);
    println!("Extracted links");

    let mut articles = Vec::new();

    for link in links {
        let article_doc = fetch_html_body(&link)?;
        let (title, body) = extract_article_content(&article_doc);
        println!("Extracted article content");
        articles.push((title, body));
    }

    build_epub(&title, articles)?;
    println!("Epub Completed");
    Ok(())
}
