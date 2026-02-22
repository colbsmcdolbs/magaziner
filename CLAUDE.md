# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Commands

```bash
# Build
cargo build

# Build release binary
cargo build --release

# Run
cargo run -- --url https://www.lrb.co.uk/the-paper/v47/n06

# Run with options
cargo run -- --url <URL> --output ./downloads --delay 1000 --force

# Run all tests
cargo test

# Run a single test
cargo test test_extract_article_links_from_issue
cargo test test_valid_lrb_url_should_pass

# Lint
cargo clippy
```

## Architecture

`magaziner` is a CLI tool that downloads a London Review of Books (LRB) issue and generates an EPUB file from it.

**Pipeline (orchestrated in `main.rs`):**
1. Fetch the issue page HTML (`fetch.rs`)
2. Parse article links, title, CSS, and cover image URI from the issue page (`parser.rs`)
3. Fetch each article page and extract its title and body HTML (`parser.rs`)
4. Build and write an EPUB file (`epub.rs`)

**Modules:**
- `main.rs` — CLI args via `clap` (derive), orchestrates the pipeline, manages output path and `--force` flag
- `fetch.rs` — blocking HTTP requests via `reqwest`; applies the configurable delay before each fetch to rate-limit requests
- `parser.rs` — HTML parsing with `scraper`; `extract_article_links` targets `a.toc-item` for article links and `div.article-issue-cover-image img` for the cover; `extract_article_content` targets `div.article-copy` for body content
- `epub.rs` — builds the EPUB using `epub-builder`; sanitizes scraped HTML for XHTML compliance (named entity replacement, self-closing tags, iframe/image stripping) before embedding
- `validation.rs` — clap `value_parser` that validates the URL matches `https://www.lrb.co.uk/the-paper/v\d{2}/n\d{2}`
- `progress.rs` — simple step counter (hardcoded total of 5 steps) that prints `[n/5]` progress lines

**Tests:**
- `parser.rs` has integration-style tests using HTML fixture files at `src/test/lrb/issue.html` and `src/test/lrb/article.html`
- `validation.rs` has unit tests for URL format validation

The tool is hardcoded to LRB (`lrb.co.uk`) — CSS selectors, URL validation, and EPUB metadata all target that site specifically.
