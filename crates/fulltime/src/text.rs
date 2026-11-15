use scraper::{ElementRef, Selector};

use crate::error::{PageError, PageKind};

pub(crate) fn selector(page: PageKind, css: &'static str) -> Result<Selector, PageError> {
    Selector::parse(css)
        .map_err(|parse_error| PageError::invalid_selector(page, css, parse_error.to_string()))
}

pub(crate) fn collapse_whitespace(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

pub(crate) fn element_text(element: ElementRef<'_>) -> String {
    collapse_whitespace(&element.text().collect::<Vec<_>>().join(" "))
}

pub(crate) fn first_text(element: ElementRef<'_>, selector: &Selector) -> Option<String> {
    element
        .select(selector)
        .next()
        .map(element_text)
        .filter(|text| !text.is_empty())
}

pub(crate) fn first_href<'element>(
    element: ElementRef<'element>,
    selector: &Selector,
) -> Option<&'element str> {
    element
        .select(selector)
        .find_map(|link| link.value().attr("href"))
}

pub fn normalised_name(text: &str) -> String {
    collapse_whitespace(text).to_uppercase()
}

pub(crate) fn query_value<'href>(href: &'href str, name: &str) -> Option<&'href str> {
    let (_, query_and_fragment) = href.split_once('?')?;
    let query = query_and_fragment
        .split_once('#')
        .map_or(query_and_fragment, |(query, _)| query);
    query
        .split('&')
        .filter_map(|pair| pair.split_once('='))
        .find(|(key, _)| *key == name)
        .map(|(_, value)| value)
        .filter(|value| !value.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalised_names_ignore_case_and_spacing() {
        assert_eq!(normalised_name("  ethan   hurren "), "ETHAN HURREN");
        assert_eq!(normalised_name("Ethan Hurren"), "ETHAN HURREN");
    }

    #[test]
    fn query_values_are_read_by_name() {
        let href = "/displayTeam.html?divisionseason=780926466&teamID=296920867";
        assert_eq!(query_value(href, "teamID"), Some("296920867"));
        assert_eq!(query_value(href, "divisionseason"), Some("780926466"));
        assert_eq!(query_value(href, "league"), None);
        assert_eq!(
            query_value("/displayCountyFixture.html?id=1&league=", "league"),
            None
        );
        assert_eq!(
            query_value(
                "/displayTeam.html?teamID=296920867&sortAsc=false#tab-2",
                "sortAsc"
            ),
            Some("false")
        );
        assert_eq!(
            query_value("/displayTeam.html?teamID=296920867#tab-2", "teamID"),
            Some("296920867")
        );
    }
}
