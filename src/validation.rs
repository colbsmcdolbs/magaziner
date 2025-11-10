use regex::Regex;
use url::Url;

pub fn validate_lrb_url(s: &str) -> Result<String, String> {
    let parsed = Url::parse(s).map_err(|_| format!("Invalid URL format: {}", s))?;

    let re = Regex::new(r"^https://www\.lrb\.co\.uk/the-paper/v\d{2}/n\d{2}/?$")
        .expect("regex should compile");

    if !re.is_match(parsed.as_str()) {
        return Err(format!(
            "Invalid LRB issue URL: {}\nExpected format like: https://www.lrb.co.uk/the-paper/v43/n01",
            s
        ));
    }

    Ok(s.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_lrb_url_should_pass() {
        let url = "https://www.lrb.co.uk/the-paper/v47/n06";
        let result = validate_lrb_url(url);
        assert!(result.is_ok(), "Expected valid URL to pass");
        assert_eq!(result.unwrap(), url);
    }

    #[test]
    fn test_valid_lrb_url_should_pass_old_url() {
        let url = "https://www.lrb.co.uk/the-paper/v01/n01";
        let result = validate_lrb_url(url);
        assert!(result.is_ok(), "Expected valid URL to pass");
        assert_eq!(result.unwrap(), url);
    }

    #[test]
    fn test_invalid_url_should_fail() {
        let url = "https://www.google.com";
        let result = validate_lrb_url(url);
        assert!(result.is_err(), "Expected non-LRB URL to fail");
        println!("Error: {}", result.unwrap_err());
    }

    #[test]
    fn test_invalid_lrb_style_url_should_fail() {
        let url = "https://www.lrb.co.uk/the-paper/v47/n06/article-title";
        let result = validate_lrb_url(url);
        assert!(result.is_err(), "Expected extra path segment to fail");
        println!("Error: {}", result.unwrap_err());
    }

    #[test]
    fn test_invalid_protocol_should_fail() {
        let url = "http://www.lrb.co.uk/the-paper/v47/n06";
        let result = validate_lrb_url(url);
        assert!(result.is_err(), "Expected http:// to fail (must be https://)");
    }

    #[test]
    fn test_invalid_numbers_should_fail() {
        let url = "https://www.lrb.co.uk/the-paper/vab/n01";
        let result = validate_lrb_url(url);
        assert!(result.is_err(), "Expected malformed version to fail");
    }
}
