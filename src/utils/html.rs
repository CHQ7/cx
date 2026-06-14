use scraper::{Html, Selector};

/// Simplify HTML by removing hidden elements, scripts, styles, etc.
/// Mirrors Python's simphtml.py functionality
pub fn simplify_html(html: &str) -> String {
    let document = Html::parse_document(html);

    // Remove script and style tags
    let mut text = document.root_element().text().collect::<String>();

    // Clean up whitespace
    text = text.split_whitespace().collect::<Vec<_>>().join(" ");

    // Truncate if too long
    if text.len() > 10000 {
        text.truncate(10000);
        text.push_str("...[truncated]");
    }

    text
}

/// Extract visible text from HTML, preserving structure
pub fn extract_text(html: &str) -> String {
    let document = Html::parse_document(html);
    let selector = Selector::parse("body").unwrap();

    let mut result = String::new();

    if let Some(body) = document.select(&selector).next() {
        for node in body.text() {
            let trimmed = node.trim();
            if !trimmed.is_empty() {
                if !result.is_empty() {
                    result.push(' ');
                }
                result.push_str(trimmed);
            }
        }
    }

    result
}

/// Remove common tracking and analytics elements
pub fn clean_tracking(html: &str) -> String {
    let document = Html::parse_document(html);
    let mut result = document.root_element().html();

    // Remove common tracking attributes
    let tracking_attrs = ["data-tracking", "data-analytics", "data-gtm"];
    for attr in &tracking_attrs {
        result = result.replace(&format!("{}=\"", attr), "");
        result = result.replace(&format!("{}='", attr), "");
    }

    result
}
