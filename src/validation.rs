use regex::Regex;
use url::Url;

#[derive(Debug, Clone, PartialEq)]
pub enum MagazineSource {
    LondonReview,
    Harpers,
}

pub fn detect_source(url: &str) -> Option<MagazineSource> {
    let lrb_re =
        Regex::new(r"^https://www\.lrb\.co\.uk/the-paper/v\d{2}/n\d{2}/?$").unwrap();
    let harpers_re = Regex::new(r"^https://harpers\.org/archive/\d{4}/\d{2}/?$").unwrap();

    if lrb_re.is_match(url) {
        Some(MagazineSource::LondonReview)
    } else if harpers_re.is_match(url) {
        Some(MagazineSource::Harpers)
    } else {
        None
    }
}

pub fn validate_magazine_url(s: &str) -> Result<String, String> {
    let _ = Url::parse(s).map_err(|_| format!("Invalid URL format: {}", s))?;

    if detect_source(s).is_some() {
        Ok(s.to_string())
    } else {
        Err(format!(
            "Unsupported URL: {}\nSupported formats:\n  LRB:      https://www.lrb.co.uk/the-paper/v47/n06\n  Harper's: https://harpers.org/archive/2026/02",
            s
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_lrb_url_should_pass() {
        let url = "https://www.lrb.co.uk/the-paper/v47/n06";
        let result = validate_magazine_url(url);
        assert!(result.is_ok(), "Expected valid URL to pass");
        assert_eq!(result.unwrap(), url);
    }

    #[test]
    fn test_valid_lrb_url_should_pass_old_url() {
        let url = "https://www.lrb.co.uk/the-paper/v01/n01";
        let result = validate_magazine_url(url);
        assert!(result.is_ok(), "Expected valid URL to pass");
        assert_eq!(result.unwrap(), url);
    }

    #[test]
    fn test_valid_harpers_url_should_pass() {
        let url = "https://harpers.org/archive/2026/02";
        let result = validate_magazine_url(url);
        assert!(result.is_ok(), "Expected valid Harper's URL to pass");
        assert_eq!(result.unwrap(), url);
    }

    #[test]
    fn test_invalid_url_should_fail() {
        let url = "https://www.google.com";
        let result = validate_magazine_url(url);
        assert!(result.is_err(), "Expected non-supported URL to fail");
    }

    #[test]
    fn test_invalid_lrb_style_url_should_fail() {
        let url = "https://www.lrb.co.uk/the-paper/v47/n06/article-title";
        let result = validate_magazine_url(url);
        assert!(result.is_err(), "Expected extra path segment to fail");
    }

    #[test]
    fn test_invalid_protocol_should_fail() {
        let url = "http://www.lrb.co.uk/the-paper/v47/n06";
        let result = validate_magazine_url(url);
        assert!(result.is_err(), "Expected http:// to fail (must be https://)");
    }

    #[test]
    fn test_invalid_numbers_should_fail() {
        let url = "https://www.lrb.co.uk/the-paper/vab/n01";
        let result = validate_magazine_url(url);
        assert!(result.is_err(), "Expected malformed version to fail");
    }

    #[test]
    fn test_detect_lrb_source() {
        assert_eq!(
            detect_source("https://www.lrb.co.uk/the-paper/v47/n06"),
            Some(MagazineSource::LondonReview)
        );
    }

    #[test]
    fn test_detect_harpers_source() {
        assert_eq!(
            detect_source("https://harpers.org/archive/2026/02"),
            Some(MagazineSource::Harpers)
        );
    }

    #[test]
    fn test_detect_unknown_returns_none() {
        assert_eq!(detect_source("https://example.com"), None);
    }
}
