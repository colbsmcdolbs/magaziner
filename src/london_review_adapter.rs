use crate::adapter::{ArticleData, IssueData, MagazineAdapter};
use crate::progress::Progress;
use scraper::{Html, Selector};

pub struct LondonReviewAdapter;

impl MagazineAdapter for LondonReviewAdapter {
    fn extract_issue(&self, doc: &Html, progress: &Progress) -> IssueData {
        let articles_selector = Selector::parse("a.toc-item").unwrap();
        let title_selector = Selector::parse("title").unwrap();
        let css_selector = Selector::parse("style").unwrap();
        let cover_selector = Selector::parse("div.article-issue-cover-image img").unwrap();

        let links: Vec<String> = doc
            .select(&articles_selector)
            .filter_map(|el| el.value().attr("href"))
            .map(|s| format!("https://www.lrb.co.uk{}", s))
            .collect();

        let title = doc
            .select(&title_selector)
            .next()
            .map(|el| el.text().collect::<String>())
            .map(|t| {
                t.split_once("Vol.")
                    .map(|(_, rest)| format!("Vol.{}", rest.trim()))
                    .unwrap_or(t)
            })
            .unwrap_or_else(|| "Untitled".into());

        let css = doc
            .select(&css_selector)
            .map(|el| el.text().collect::<String>().trim().to_string())
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
            .join("\n");

        let img = doc.select(&cover_selector).next().unwrap();
        let cover_image_uri = img
            .value()
            .attr("data-appsrc")
            .or_else(|| img.value().attr("srcset"))
            .or_else(|| img.value().attr("src"))
            .map(|url| url.split_whitespace().next().unwrap_or("").to_string())
            .unwrap_or_else(|| "".into());

        progress.verbose(&format!("Found {} article links", links.len()));
        progress.verbose(&format!("Issue title: {}", title));
        progress.verbose(&format!("Cover image: {}", cover_image_uri));

        IssueData {
            links,
            title,
            css,
            cover_image_uri,
            publication_name: "London Review of Books".to_string(),
        }
    }

    fn extract_article(&self, doc: &Html, progress: &Progress) -> ArticleData {
        let title_selector = Selector::parse("title").unwrap();
        let reviewed_items_selector = Selector::parse("div.reviewed-items").unwrap();
        let body_selector = Selector::parse("div.article-copy").unwrap();

        let title = doc
            .select(&title_selector)
            .next()
            .map(|el| el.text().collect::<String>())
            .unwrap_or_else(|| "Untitled".into());

        let reviewed_items = doc
            .select(&reviewed_items_selector)
            .map(|el| el.inner_html())
            .collect::<Vec<_>>()
            .join("\n\n");

        let body = doc
            .select(&body_selector)
            .map(|el| el.inner_html())
            .collect::<Vec<_>>()
            .join("\n\n");

        let complete_article = format!("{reviewed_items}{body}");

        progress.verbose(&format!("Extracted: {}", title));

        ArticleData {
            title,
            body: complete_article,
        }
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
    fn test_extract_article_links_from_issue() {
        let doc = load_html_fixture("src/test/lrb/issue.html");
        let progress = Progress::new(Verbosity::Quiet);
        let adapter = LondonReviewAdapter;
        let issue = adapter.extract_issue(&doc, &progress);

        assert!(!issue.links.is_empty(), "Expected at least one article link");
        assert_eq!(issue.title, "Vol.99 No. 3 · 15 March 2025");
        assert!(
            issue.links.iter().all(|l| l.starts_with("https://www.lrb.co.uk")),
            "All links should be absolute LRB URLs"
        );
        assert!(!issue.css.is_empty());
        assert!(!issue.cover_image_uri.is_empty());
    }

    #[test]
    fn test_extract_article_content_from_article() {
        let doc = load_html_fixture("src/test/lrb/article.html");
        let progress = Progress::new(Verbosity::Quiet);
        let adapter = LondonReviewAdapter;
        let article = adapter.extract_article(&doc, &progress);

        assert!(!article.title.is_empty(), "Article should have a title");
        assert!(article.body.len() > 100, "Article body should be long enough");
    }
}
