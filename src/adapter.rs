use crate::progress::Progress;
use scraper::Html;

pub struct IssueData {
    pub links: Vec<String>,
    pub title: String,
    pub css: String,
    pub cover_image_uri: String,
    pub publication_name: String,
    pub date: Option<String>,
}

pub struct ArticleData {
    pub title: String,
    pub body: String,
}

pub trait MagazineAdapter {
    fn extract_issue(&self, doc: &Html, progress: &Progress) -> IssueData;
    fn extract_article(&self, doc: &Html, progress: &Progress) -> ArticleData;
}
