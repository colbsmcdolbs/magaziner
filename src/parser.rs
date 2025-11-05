use scraper::{Html, Selector};

pub fn extract_article_links(doc: &Html) -> (Vec<String>, String, String) {
    let selector = Selector::parse("a.toc-item").unwrap();
    let title_selector = Selector::parse("title").unwrap();
    let css_selector = Selector::parse("style").unwrap();
    let mut css_content = String::new();

    let links = doc
        .select(&selector)
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

    for el in doc.select(&css_selector) {
        css_content.push_str(&el.text().collect::<String>());
    }

    (links, title, css_content)
}

pub fn extract_article_content(doc: &Html) -> (String, String) {
    let title_selector = Selector::parse("title").unwrap();
    let body_selector = Selector::parse("div.article-copy").unwrap();

    let title = doc
        .select(&title_selector)
        .next()
        .map(|el| el.text().collect::<String>())
        .unwrap_or_else(|| "Untitled".into());

    let body = doc
        .select(&body_selector)
        .map(|el| el.inner_html())
        .collect::<Vec<_>>()
        .join("\n\n");

    (title, body)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn load_html_fixture(path: &str) -> Html {
        let html = fs::read_to_string(path)
            .unwrap_or_else(|_| panic!("Failed to read fixture at {}", path));
        Html::parse_document(&html)
    }

    #[test]
    fn test_extract_article_links_from_issue() {
        let doc = load_html_fixture("src/test/files/lrb/issue.html");
        let (links, title, css_sheet) = extract_article_links(&doc);

        assert!(!links.is_empty(), "Expected at least one article link");
        assert!(title == "Vol.1 No. 1 · 25 October 1979");
        assert!(
            links.iter().all(|l| l.starts_with("https://www.lrb.co.uk")),
            "All links should be absolute LRB URLs"
        );
        assert!(!css_sheet.is_empty());
        // Uncomment for debugging
        // println!("{}", title);
        // println!("Parsed {} links:", links.len());
        // for link in links {
        //     println!("{}", link);
        // }
    }

    #[test]
    fn test_extract_article_content_from_article() {
        let doc = load_html_fixture("src/test/files/lrb/article.html");
        let (title, body) = extract_article_content(&doc);

        assert!(!title.is_empty(), "Article should have a title");
        assert!(body.len() > 100, "Article body should be long enough");
        // Uncomment for debugging
        // println!("{}", body);
        // println!("{}", title);
    }
}
