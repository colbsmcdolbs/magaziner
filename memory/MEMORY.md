# magaziner Project Memory

## Architecture
- Multi-publication CLI tool; generates EPUB from LRB or Harper's issues
- Adapter pattern: `MagazineAdapter` trait in `adapter.rs`; concrete impls in `london_review_adapter.rs` and `harpers_adapter.rs`
- URL regex detection in `validation.rs` → `MagazineSource` enum → selects adapter at runtime
- `fetch.rs` builds a `reqwest::blocking::Client` via `make_client(cookie)` — all fetch fns take `&Client`
- `HARPERS_COOKIE` env var: raw Cookie header string for Harper's subscriber access (warns if unset)

## Key selectors
- LRB issue: `a.toc-item` (links), `div.article-issue-cover-image img` (cover)
- LRB article: `div.article-copy` (body), `div.reviewed-items` (prefix)
- Harper's issue: `div.article-card a` deduped (links), `div.issue-cover img.cover-img` (cover), title from `<title>` split on " | "
- Harper's article: `h1.article-title` (title), `div.wysiwyg-content.entry-content` (body)

## URL formats
- LRB: `https://www.lrb.co.uk/the-paper/vNN/nNN`
- Harper's: `https://harpers.org/archive/YYYY/MM`

## Test fixtures
- `src/test/lrb/issue.html`, `src/test/lrb/article.html`
- `src/test/harpers/issue.html`, `src/test/harpers/article.html`
