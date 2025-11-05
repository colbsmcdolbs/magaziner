use crate::fetch::download_image;
use anyhow::Result;
use epub_builder::{EpubBuilder, EpubContent, ReferenceType, ZipLibrary};
use regex::Regex;
use std::fs::{File, remove_file};

pub fn build_epub(
    title: &str,
    articles: Vec<(String, String)>,
    css_sheet: &str,
    image_uri: &str,
) -> Result<()> {
    let mut epub = EpubBuilder::new(ZipLibrary::new()?)?;
    epub.metadata("title", title)?
        .metadata("author", "London Review of Books")?;

    if !image_uri.trim().is_empty() && image_uri.starts_with("http") {
        let cover_path = "cover.jpg";
        download_image(image_uri, cover_path)?;

        {
            let mut cover_file = File::open(cover_path)?;
            epub.add_cover_image("cover.jpg", &mut cover_file, "image/jpeg")?;
            println!("Successfully added cover image to epub")
        }

        remove_file(cover_path)?;
    }

    epub.stylesheet(css_sheet.as_bytes())?;

    let title_page = format!(
        r#"<?xml version="1.0" encoding="utf-8"?>
<!DOCTYPE html>
<html xmlns="http://www.w3.org/1999/xhtml">
  <head><title>{}</title></head>
  <body style="text-align: center; margin-top: 40%;">
    <h1>{}</h1>
    <h3>London Review of Books</h3>
  </body>
</html>"#,
        title, title
    );

    epub.add_content(
        EpubContent::new("title.xhtml", title_page.as_bytes())
            .title("Title Page")
            .reftype(ReferenceType::Cover),
    )?;

    let toc_html = {
        let mut list_items = String::new();
        for (i, (article_title, _)) in articles.iter().enumerate() {
            list_items.push_str(&format!(
                r#"<li><a href="article{}.xhtml">{}</a></li>"#,
                i, article_title
            ));
        }

        format!(
            r#"<?xml version="1.0" encoding="utf-8"?>
<!DOCTYPE html>
<html xmlns="http://www.w3.org/1999/xhtml">
  <head><title>Table of Contents</title></head>
  <body>
    <h2>Table of Contents</h2>
    <ol>
      {}
    </ol>
  </body>
</html>"#,
            list_items
        )
    };

    epub.add_content(
        EpubContent::new("toc.xhtml", toc_html.as_bytes())
            .title("Table of Contents")
            .reftype(ReferenceType::Toc),
    )?;

    for (i, (article_title, body)) in articles.into_iter().enumerate() {
        let filename = format!("article{}.xhtml", i);
        let safe_body = sanitize_html_for_epub(&body);

        let xhtml = format!(
            r#"<?xml version="1.0" encoding="utf-8"?>
<!DOCTYPE html>
<html xmlns="http://www.w3.org/1999/xhtml">
  <head>
    <title>{}</title>
    <style>
      body {{
        font-family: serif;
        margin: 2em;
      }}
      h1.article-title {{
        font-size: 2em;
        text-align: center;
        margin-top: 1em;
        margin-bottom: 1.5em;
      }}
    </style>
  </head>
  <body>
    <h1 class="article-title">{}</h1>
    {}
  </body>
</html>"#,
            article_title, article_title, safe_body
        );

        epub.add_content(
            EpubContent::new(filename, xhtml.as_bytes())
                .title(&article_title)
                .reftype(ReferenceType::Text),
        )?;
    }

    let file = File::create(format!("{}.epub", title))?;
    epub.generate(file)?;
    Ok(())
}

fn sanitize_html_for_epub(html: &str) -> String {
    let body = html
        .replace("&nbsp;", "&#160;")
        .replace("&mdash;", "&#8212;")
        .replace("&ndash;", "&#8211;")
        .replace("&lsquo;", "&#8216;")
        .replace("&rsquo;", "&#8217;")
        .replace("&ldquo;", "&#8220;")
        .replace("&rdquo;", "&#8221;")
        .replace("&hellip;", "&#8230;")
        .replace("<br>", "<br />")
        .replace("<hr>", "<hr />")
        .replace("<img ", "<img ")
        .replace("<br/>", "<br />")
        .replace("<hr/>", "<hr />");

    let iframe_regex = Regex::new(r"(?is)<iframe.*?</iframe>").unwrap();
    let stripped = iframe_regex.replace_all(&body, "").into_owned();

    let img_regex = Regex::new(r"(?is)<img[^>]*>").unwrap();
    img_regex.replace_all(&stripped, "").to_string()
}
