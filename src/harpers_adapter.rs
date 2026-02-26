use crate::adapter::{ArticleData, IssueData, MagazineAdapter};
use crate::progress::Progress;
use scraper::{Html, Selector};
use std::collections::HashSet;

pub struct HarpersAdapter;

impl MagazineAdapter for HarpersAdapter {
    fn extract_issue(&self, doc: &Html, progress: &Progress) -> IssueData {
        let issue_article_sel =
            Selector::parse("section.issue-articles div.issue-article").unwrap();
        let reading_item_sel = Selector::parse("section.issue-readings div.reading-item").unwrap();
        let a_sel = Selector::parse("a").unwrap();
        let ac_tax_sel = Selector::parse("span.ac-tax").unwrap();
        let title_selector = Selector::parse("title").unwrap();
        let cover_selector = Selector::parse("div.issue-cover img.cover-img").unwrap();

        let mut seen: HashSet<String> = HashSet::new();

        // Collect Readings section links in DOM order
        let reading_links: Vec<String> = doc
            .select(&reading_item_sel)
            .filter_map(|item| {
                item.select(&a_sel)
                    .filter_map(|a| a.value().attr("href"))
                    .find(|href| {
                        let parts: Vec<&str> = href.trim_matches('/').split('/').collect();
                        parts.len() >= 4 && parts.first() == Some(&"archive")
                    })
                    .map(|href| format!("https://harpers.org{}", href))
            })
            .collect();

        // Process issue-articles in DOM order (already in magazine sequence).
        // Insert Readings after the Harper's Index card.
        let mut links: Vec<String> = Vec::new();
        let mut readings_inserted = false;

        for article_el in doc.select(&issue_article_sel) {
            let is_index = article_el
                .select(&ac_tax_sel)
                .next()
                .map(|el| el.text().collect::<String>().contains("Index"))
                .unwrap_or(false);

            // Prefer /archive/ link; fall back to /harpers-index/ for the Index card.
            let href_opt = article_el
                .select(&a_sel)
                .filter_map(|a| a.value().attr("href"))
                .find(|href| {
                    let parts: Vec<&str> = href.trim_matches('/').split('/').collect();
                    parts.len() >= 4 && parts.first() == Some(&"archive")
                })
                .or_else(|| {
                    if is_index {
                        article_el
                            .select(&a_sel)
                            .filter_map(|a| a.value().attr("href"))
                            .find(|href| href.contains("/harpers-index/"))
                    } else {
                        None
                    }
                });

            if let Some(href) = href_opt {
                let url = format!("https://harpers.org{}", href);
                if seen.insert(url.clone()) {
                    links.push(url);
                }
            }

            // Insert all Readings articles immediately after Harper's Index.
            if is_index && !readings_inserted {
                readings_inserted = true;
                for r_url in &reading_links {
                    if seen.insert(r_url.clone()) {
                        links.push(r_url.clone());
                    }
                }
            }
        }

        let title = doc
            .select(&title_selector)
            .next()
            .map(|el| el.text().collect::<String>())
            .map(|t| {
                // "February 2026 | Harper's Magazine" → "February 2026"
                t.split_once(" | ")
                    .map(|(month, _)| month.trim().to_string())
                    .unwrap_or(t)
            })
            .unwrap_or_else(|| "Untitled".into());

        let cover_image_uri = doc
            .select(&cover_selector)
            .next()
            .and_then(|el| el.value().attr("src"))
            .unwrap_or("")
            .to_string();

        progress.verbose(&format!("Found {} article links", links.len()));
        progress.verbose(&format!("Issue title: {}", title));
        progress.verbose(&format!("Cover image: {}", cover_image_uri));

        IssueData {
            links,
            title,
            css: String::new(),
            cover_image_uri,
            publication_name: "Harper's Magazine".to_string(),
        }
    }

    fn extract_article(&self, doc: &Html, progress: &Progress) -> ArticleData {
        let title_selector = Selector::parse("h1.article-title").unwrap();
        let fallback_title_selector = Selector::parse("title").unwrap();
        let body_selector = Selector::parse("div.wysiwyg-content.entry-content").unwrap();
        let header_meta_sel = Selector::parse("div.header-meta").unwrap();

        let title = doc
            .select(&title_selector)
            .next()
            .map(|el| el.text().collect::<String>().trim().to_string())
            .filter(|t| !t.is_empty())
            .or_else(|| {
                doc.select(&fallback_title_selector)
                    .next()
                    .map(|el| el.text().collect::<String>())
            })
            .unwrap_or_else(|| "Untitled".into());

        let body = doc
            .select(&body_selector)
            .map(|el| {
                let raw = el.inner_html();
                // Remove the "Adjust / Share" UI controls embedded at the top of the body.
                if let Some(header_el) = el.select(&header_meta_sel).next() {
                    raw.replacen(&header_el.html(), "", 1)
                } else {
                    raw
                }
            })
            .collect::<Vec<_>>()
            .join("\n\n");

        progress.verbose(&format!("Extracted: {}", title));

        ArticleData { title, body }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::progress::Verbosity;
    use scraper::Html;
    use std::fs;

    fn load_html_fixture(path: &str) -> Html {
        let html = fs::read_to_string(path)
            .unwrap_or_else(|_| panic!("Failed to read fixture at {}", path));
        Html::parse_document(&html)
    }

    #[test]
    fn test_extract_article_links_from_harpers_issue() {
        let doc = load_html_fixture("src/test/harpers/issue.html");
        let progress = Progress::new(Verbosity::Quiet);
        let adapter = HarpersAdapter;
        let issue = adapter.extract_issue(&doc, &progress);

        assert!(
            !issue.links.is_empty(),
            "Expected at least one article link"
        );
        assert_eq!(issue.title, "February 2026");
        assert!(
            issue.links.iter().all(|l| {
                l.starts_with("https://harpers.org/archive/")
                    || l.starts_with("https://harpers.org/harpers-index/")
            }),
            "All links should be absolute Harper's article URLs"
        );
        assert!(!issue.cover_image_uri.is_empty());
    }

    #[test]
    fn test_extract_article_content_from_harpers_article() {
        let doc = load_html_fixture("src/test/harpers/article.html");
        let progress = Progress::new(Verbosity::Quiet);
        let adapter = HarpersAdapter;
        let article = adapter.extract_article(&doc, &progress);

        assert!(!article.title.is_empty(), "Article should have a title");
        assert!(
            article.body.len() > 100,
            "Article body should be long enough"
        );
    }

    #[test]
    fn test_article_body_excludes_adjust_share_controls() {
        let doc = load_html_fixture("src/test/harpers/article.html");
        let progress = Progress::new(Verbosity::Quiet);
        let adapter = HarpersAdapter;
        let article = adapter.extract_article(&doc, &progress);

        assert!(
            !article.body.contains("header-meta"),
            "Article body should not contain the Adjust/Share UI controls"
        );
    }
}
