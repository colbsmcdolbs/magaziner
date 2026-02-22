use crate::adapter::{ArticleData, IssueData, MagazineAdapter};
use crate::progress::Progress;
use scraper::{Html, Selector};
use std::collections::HashSet;

pub struct HarpersAdapter;

impl MagazineAdapter for HarpersAdapter {
    fn extract_issue(&self, doc: &Html, progress: &Progress) -> IssueData {
        let link_selector = Selector::parse("div.article-card a").unwrap();
        let title_selector = Selector::parse("title").unwrap();
        let cover_selector = Selector::parse("div.issue-cover img.cover-img").unwrap();

        let mut seen = HashSet::new();
        let links: Vec<String> = doc
            .select(&link_selector)
            .filter_map(|el| el.value().attr("href"))
            .filter(|href| {
                // Keep only article URLs (/archive/YYYY/MM/slug), not issue URLs (/archive/YYYY/MM)
                let parts: Vec<&str> = href.trim_matches('/').split('/').collect();
                parts.len() >= 4 && parts.first() == Some(&"archive")
            })
            .map(|href| format!("https://harpers.org{}", href))
            .filter(|url| seen.insert(url.clone()))
            .collect();

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
            .map(|el| el.inner_html())
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

        assert!(!issue.links.is_empty(), "Expected at least one article link");
        assert_eq!(issue.title, "February 2026");
        assert!(
            issue.links.iter().all(|l| l.starts_with("https://harpers.org/archive/")),
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
        assert!(article.body.len() > 100, "Article body should be long enough");
    }
}
