mod epub;
mod fetch;
mod parser;
mod progress;
mod validation;

use anyhow::Result;
use clap::Parser;
use epub::build_epub;
use fetch::fetch_html_body;
use parser::{extract_article_content, extract_article_links};
use progress::Progress;
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
        help = "Output directory for generated EPUBs (Ex: ./downloads)",
        default_value = "."
    )]
    output: PathBuf,

    #[arg(
        short,
        long,
        help = "Delay between calls to magazine source in milliseconds (ex: 1000 = 1 second)",
        default_value_t = 3000
    )]
    delay: u64,

    #[arg(
        short,
        long,
        help = "Overwrite the output file if it already exists",
        default_value_t = false
    )]
    force: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let url = args.url;
    let output = args.output;
    let delay = args.delay;
    let force = args.force;

    let mut progress = Progress::new();

    if !output.exists() {
        std::fs::create_dir_all(&output).expect("Failed to create output directory");
    }

    progress.next("Fetching issue HTML…");
    let doc = fetch_html_body(&url, &delay)?;
    let (links, title, css_sheet, image_uri) = extract_article_links(&doc);

    let output_path = output.join(format!("{}.epub", title));
    if output_path.exists() && !force {
        return Err(anyhow::anyhow!(
            "File '{}' already exists. Use --force to overwrite.",
            output_path.display()
        ));
    }

    let article_length = links.len();
    progress.next(&format!("Extracting {} articles…", article_length));

    let mut articles = Vec::new();
    for (i, link) in links.iter().enumerate() {
        progress.substep(i, article_length);
        let article_doc = fetch_html_body(&link, &delay)?;
        let (title, body) = extract_article_content(&article_doc);
        articles.push((title, body));
    }

    build_epub(
        &mut progress,
        &title,
        &output,
        articles,
        &css_sheet,
        &image_uri,
    )?;
    Ok(())
}
