# magaziner

A fast, self-hosted CLI tool that downloads a magazine issue from your subscription and generates a clean, portable **EPUB** file — ready to load on any e-reader.

Currently supports:
- **London Review of Books** (`lrb.co.uk`)
- **Harper's Magazine** (`harpers.org`)

---

## Why

Magazine websites are noisy, require a live internet connection, and often restrict offline access. `magaziner` fetches every article in an issue, strips navigation and ads, and bundles the result into a standards-compliant EPUB. Read your subscription on a Kindle, Kobo, or any e-reader app — without a browser.

---

## Features

- Generates a complete EPUB from a single issue URL
- Fetches the cover image and embeds it
- Builds a linked table of contents
- Configurable per-request delay for polite rate limiting
- Verbose and quiet output modes for scripting
- Adapter-based architecture — adding a new publication is self-contained
- Authenticated fetching via cookie passthrough (Harper's)

---

## Installation

### Prerequisites

- [Rust](https://rustup.rs/) (edition 2024, stable toolchain)

### Build from source

```bash
git clone https://github.com/colbsmcdolbs/magaziner
cd magaziner
cargo build --release
```

The binary will be at `./target/release/magaziner`. You can copy it anywhere on your `$PATH`:

```bash
cp target/release/magaziner ~/.local/bin/
```

---

## Usage

```
magaziner --url <URL> [OPTIONS]

Options:
  -u, --url <URL>        Magazine archive URL (LRB or Harper's)
  -o, --output <OUTPUT>  Output directory for generated EPUBs [default: .]
  -d, --delay <DELAY>    Delay between requests in milliseconds [default: 3000]
  -f, --force            Overwrite the output file if it already exists
  -v, --verbose          Print detailed network and parsing logs
  -q, --quiet            Suppress all output (for scripting)
  -h, --help             Print help
  -V, --version          Print version
```

### London Review of Books

```bash
magaziner --url https://www.lrb.co.uk/the-paper/v47/n06
```

The URL must match the format `https://www.lrb.co.uk/the-paper/vNN/nNN` — the issue index page, not an individual article.

### Harper's Magazine

Harper's requires an active subscription to access full article content. Export your session cookie from a logged-in browser and set the `HARPERS_COOKIE` environment variable before running:

```bash
export HARPERS_COOKIE="your_session_cookie_string_here"
magaziner --url https://harpers.org/archive/2026/02
```

The URL must match the format `https://harpers.org/archive/YYYY/MM`.

> **Getting your cookie:** In Chrome or Firefox, open DevTools → Application → Cookies while logged in to `harpers.org`, then copy the full cookie string from the `Cookie` request header (visible in the Network tab on any page request).

---

## Examples

Download an LRB issue to the current directory:

```bash
magaziner --url https://www.lrb.co.uk/the-paper/v47/n06
```

Download to a specific folder, overwriting if it exists:

```bash
magaziner --url https://www.lrb.co.uk/the-paper/v47/n06 --output ~/Books --force
```

Download a Harper's issue with a faster request cadence and verbose logging:

```bash
HARPERS_COOKIE="..." magaziner \
  --url https://harpers.org/archive/2026/02 \
  --output ~/Books \
  --delay 1000 \
  --verbose
```

Use in a shell script (quiet mode, exits non-zero on error):

```bash
HARPERS_COOKIE="..." magaziner \
  --url https://harpers.org/archive/2026/02 \
  --output ~/Books \
  --quiet
```

---

## Output

The generated file is named after the issue and written to the output directory:

```
~/Books/February 2026.epub
~/Books/Vol. 47 No. 6 · 20 March 2025.epub
```

The EPUB includes:
- A cover image (fetched from the issue page)
- A title page
- A linked table of contents
- All articles, formatted for e-readers

---

## Supported URL Formats

| Publication | URL Format | Example |
|---|---|---|
| London Review of Books | `https://www.lrb.co.uk/the-paper/vNN/nNN` | `https://www.lrb.co.uk/the-paper/v47/n06` |
| Harper's Magazine | `https://harpers.org/archive/YYYY/MM` | `https://harpers.org/archive/2026/02` |

`magaziner` validates the URL at startup and will clearly tell you what format is expected if it doesn't match.

---

## Architecture

```
src/
├── main.rs                   # CLI args (clap), pipeline orchestration
├── adapter.rs                # MagazineAdapter trait + IssueData/ArticleData structs
├── london_review_adapter.rs  # LRB HTML parsing
├── harpers_adapter.rs        # Harper's HTML parsing
├── fetch.rs                  # HTTP client (reqwest blocking), cookie injection
├── epub.rs                   # EPUB assembly (epub-builder), HTML sanitization
├── validation.rs             # URL regex validation, MagazineSource detection
└── progress.rs               # Progress output (normal / verbose / quiet)
```

### Pipeline

1. **Validate URL** — regex match determines which adapter to use
2. **Build HTTP client** — `reqwest::blocking::Client` with optional `Cookie` header
3. **Fetch issue page** — parse article links, title, CSS, and cover image URL
4. **Fetch each article** — extract title and body HTML, respecting the configured delay
5. **Build EPUB** — sanitize HTML for XHTML compliance, assemble with cover and TOC

### Adding a new publication

Implement `MagazineAdapter` for your new source:

```rust
pub trait MagazineAdapter {
    fn extract_issue(&self, doc: &Html, progress: &Progress) -> IssueData;
    fn extract_article(&self, doc: &Html, progress: &Progress) -> ArticleData;
}
```

Then add a regex branch to `detect_source()` in `validation.rs` and wire up the adapter in `main.rs`. No other files need to change.

---

## Environment Variables

| Variable | Description |
|---|---|
| `HARPERS_COOKIE` | Raw `Cookie` header value for an authenticated Harper's session. Required for full subscriber access. If unset, a warning is printed and only free-tier content will be available. |

---

## Development

```bash
# Run all tests
cargo test

# Run a specific test
cargo test test_extract_article_links_from_harpers_issue

# Lint
cargo clippy

# Build optimized release binary
cargo build --release
```

### Test fixtures

Integration-style tests use saved HTML fixture files rather than live network requests:

```
src/test/
├── lrb/
│   ├── issue.html    # LRB issue index page
│   └── article.html  # LRB article page
└── harpers/
    ├── issue.html    # Harper's issue index page
    └── article.html  # Harper's article page
```

---

## Dependencies

| Crate | Purpose |
|---|---|
| [`clap`](https://crates.io/crates/clap) | CLI argument parsing |
| [`reqwest`](https://crates.io/crates/reqwest) | Blocking HTTP client |
| [`scraper`](https://crates.io/crates/scraper) | HTML parsing via CSS selectors |
| [`epub-builder`](https://crates.io/crates/epub-builder) | EPUB file generation |
| [`regex`](https://crates.io/crates/regex) | URL validation |
| [`url`](https://crates.io/crates/url) | URL parsing |
| [`anyhow`](https://crates.io/crates/anyhow) | Ergonomic error handling |

---

## Legal

This tool is intended for personal use with content you have a valid subscription to access. Downloading and archiving articles for personal offline reading is consistent with fair use principles in many jurisdictions, but you are responsible for complying with each publication's terms of service and applicable law in your country.
