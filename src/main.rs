mod adapter;
mod epub;
mod fetch;
mod harpers_adapter;
mod london_review_adapter;
mod progress;
mod validation;

use adapter::MagazineAdapter;
use anyhow::Result;
use clap::Parser;
use epub::build_epub;
use fetch::{fetch_html_body, make_client};
use harpers_adapter::HarpersAdapter;
use london_review_adapter::LondonReviewAdapter;
use progress::{Progress, Verbosity};
use std::path::PathBuf;
use validation::{detect_source, validate_magazine_url, MagazineSource};

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
        value_parser = validate_magazine_url,
        help = "Magazine archive URL (LRB or Harper's)"
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

    #[arg(
        short,
        long,
        help = "Print detailed network and parsing logs",
        conflicts_with = "quiet"
    )]
    verbose: bool,

    #[arg(
        short,
        long,
        help = "Suppress all output for script automation",
        conflicts_with = "verbose"
    )]
    quiet: bool,

    #[arg(
        short,
        long,
        help = "Custom output filename without extension (ex: --name \"My Issue\")"
    )]
    name: Option<String>,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let url = args.url;
    let output = args.output;
    let delay = args.delay;
    let force = args.force;

    let verbosity = if args.verbose {
        Verbosity::Verbose
    } else if args.quiet {
        Verbosity::Quiet
    } else {
        Verbosity::Normal
    };

    let mut progress = Progress::new(verbosity);

    let source = detect_source(&url).expect("URL already validated by clap");

    // HARPERS_COOKIE should be set to the raw Cookie header value from an authenticated
    // browser session (e.g. "wordpress_logged_in_xxx=abc123; other_cookie=value").
    let harpers_cookie = std::env::var("HARPERS_COOKIE").ok();
    if matches!(source, MagazineSource::Harpers) && harpers_cookie.is_none() {
        eprintln!(
            "Warning: HARPERS_COOKIE env var not set; subscriber content may be inaccessible."
        );
    }

    let cookie = match source {
        MagazineSource::Harpers => harpers_cookie.as_deref(),
        MagazineSource::LondonReview => None,
    };

    let client = make_client(cookie)?;

    let adapter: Box<dyn MagazineAdapter> = match source {
        MagazineSource::LondonReview => Box::new(LondonReviewAdapter),
        MagazineSource::Harpers => Box::new(HarpersAdapter),
    };

    if !output.exists() {
        std::fs::create_dir_all(&output)?;
    }

    progress.next("Fetching issue HTML…");
    let doc = fetch_html_body(&client, &url, &delay, &progress)?;
    let issue = adapter.extract_issue(&doc, &progress);

    let magazine_prefix = match source {
        MagazineSource::Harpers => "Harpers",
        MagazineSource::LondonReview => "LRB",
    };
    let filename = args
        .name
        .unwrap_or_else(|| format!("{} - {}", magazine_prefix, issue.title));

    let output_path = output.join(format!("{}.epub", filename));
    if output_path.exists() && !force {
        return Err(anyhow::anyhow!(
            "File '{}' already exists. Use --force to overwrite.",
            output_path.display()
        ));
    }

    let article_length = issue.links.len();
    progress.next(&format!("Extracting {} articles…", article_length));

    let mut articles = Vec::new();
    for (i, link) in issue.links.iter().enumerate() {
        progress.substep(i, article_length);
        let article_doc = fetch_html_body(&client, link, &delay, &progress)?;
        let article = adapter.extract_article(&article_doc, &progress);
        articles.push((article.title, article.body));
    }

    build_epub(
        &mut progress,
        &issue.title,
        &issue.publication_name,
        &filename,
        &output,
        articles,
        &issue.css,
        &issue.cover_image_uri,
        &client,
    )?;

    Ok(())
}
