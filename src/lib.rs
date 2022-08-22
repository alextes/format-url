//! Format URLs for fetch requests using templates and substitution values.
//!
//! This library is currently considering a pure function based design where the data required is
//! essentially constructed in the function call, and a builder pattern design.
//!
//! ## Usage - fn pattern
//! ```
//! use format_url::format_url;
//!
//! let url = format_url(
//!     "https://api.example.com/",
//!     "/user",
//!     Some(vec![("id", "alex+tes")]),
//!     None,
//! ).unwrap();
//!
//! assert_eq!(url, "https://api.example.com/user?id=alex%2Btes");
//! ```
//!
//! ## Usage - builder pattern
//! ```
//! use format_url::FormatUrlV2;
//!
//! let url = FormatUrlV2::new("https://api.example.com/")
//!     .with_path_template("/user/:name")
//!     .with_substitutes(vec![("name", "alex")])
//!     .with_query_params(vec![("active", "true")])
//!     .format_url()
//!     .unwrap();
//!
//! assert_eq!(url, "https://api.example.com/user/alex?active=true");
//! ```
//!
//! ## Wishlist
//! * Support for lists and nested values. (serde_urlencoded -> serde_qs)

use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
use serde::Serialize;

type SubstitutePairs<'a> = Vec<(&'a str, &'a str)>;

fn strip_double_slash<'a>(base_url: &str, route_template: &'a str) -> &'a str {
    if base_url.ends_with("/") && route_template.starts_with("/") {
        &route_template[1..]
    } else {
        route_template
    }
}

fn format_path(route_template: &str, substitutes: &SubstitutePairs) -> String {
    substitutes
        .iter()
        .fold(route_template.to_owned(), |route, (key, value)| {
            route.replace(
                &format!(":{}", key),
                &utf8_percent_encode(&value, NON_ALPHANUMERIC).to_string(),
            )
        })
}

pub fn format_url(
    base_url: &str,
    path_template: &str,
    query_params: Option<impl Serialize>,
    substitutes: Option<SubstitutePairs>,
) -> Result<String, serde_urlencoded::ser::Error> {
    let formatted_path = substitutes.map_or_else(
        || path_template.to_string(),
        |substitutes| format_path(path_template, &substitutes),
    );

    let formatted_querystring = query_params.map_or_else(
        || Ok(String::new()),
        |query_params| {
            let query_string = serde_urlencoded::to_string(query_params)?;
            Ok(String::from("?") + (&query_string))
        },
    )?;

    let safe_formatted_route = strip_double_slash(base_url, &formatted_path);

    Ok(format!(
        "{}{}{}",
        base_url, safe_formatted_route, formatted_querystring
    ))
}

pub struct FormatUrlV2<'a, T: Serialize> {
    base: &'a str,
    path_template: Option<&'a str>,
    query_params: Option<T>,
    substitutes: Option<SubstitutePairs<'a>>,
}

impl<'a, T: Serialize> FormatUrlV2<'a, T> {
    pub fn new(base: &'a str) -> Self {
        Self {
            base,
            path_template: None,
            query_params: None,
            substitutes: None,
        }
    }

    pub fn with_path_template(mut self, path_template: &'a str) -> Self {
        self.path_template = Some(path_template);
        self
    }

    pub fn with_query_params(mut self, params: T) -> Self {
        self.query_params = Some(params);
        self
    }

    pub fn with_substitutes(mut self, substitutes: SubstitutePairs<'a>) -> Self {
        self.substitutes = Some(substitutes);
        self
    }

    pub fn format_url(self) -> Result<String, serde_urlencoded::ser::Error> {
        let formatted_path = match (self.path_template, &self.substitutes) {
            (Some(path_template), Some(substitutes)) => format_path(path_template, &substitutes),
            (Some(path_template), _) => path_template.to_string(),
            _ => String::from(""),
        };

        let formatted_querystring = &self.query_params.map_or_else(
            || Ok(String::new()),
            |query_params| {
                let query_string = serde_urlencoded::to_string(query_params)?;
                Ok(String::from("?") + (&query_string))
            },
        )?;

        let safe_formatted_route = strip_double_slash(self.base, &formatted_path);

        Ok(format!(
            "{}{}{}",
            self.base, safe_formatted_route, formatted_querystring
        ))
    }
}

#[cfg(test)]
mod tests {
    use crate::{format_url, FormatUrlV2, SubstitutePairs};

    #[test]
    fn accepts_empty_path() {
        assert_eq!(
            format_url("https://api.example.com", "", None::<SubstitutePairs>, None),
            Ok("https://api.example.com".to_string())
        );
    }

    #[test]
    fn adds_path_to_base() {
        assert_eq!(
            format_url(
                "https://api.example.com",
                "/user",
                None::<SubstitutePairs>,
                None
            )
            .unwrap(),
            "https://api.example.com/user"
        );
    }

    #[test]
    fn strips_double_slash() {
        assert_eq!(
            format_url(
                "https://api.example.com/",
                "/user",
                None::<SubstitutePairs>,
                None
            )
            .unwrap(),
            "https://api.example.com/user"
        );
    }

    #[test]
    fn adds_path_substitutes() {
        assert_eq!(
            format_url(
                "https://api.example.com/",
                "/user/:id",
                None::<SubstitutePairs>,
                Some(vec![("id", "alextes")])
            )
            .unwrap(),
            "https://api.example.com/user/alextes"
        );
    }

    #[test]
    fn adds_querystring() {
        assert_eq!(
            format_url(
                "https://api.example.com/",
                "/user",
                Some(vec![("id", "alextes")]),
                None
            )
            .unwrap(),
            "https://api.example.com/user?id=alextes"
        );
    }

    #[test]
    fn percent_encodes_substitutes() {
        assert_eq!(
            format_url(
                "https://api.example.com/",
                "/user/:id",
                None::<SubstitutePairs>,
                Some(vec![("id", "alex tes")]),
            )
            .unwrap(),
            "https://api.example.com/user/alex%20tes"
        )
    }

    #[test]
    fn percent_encodes_query_params() {
        assert_eq!(
            format_url(
                "https://api.example.com/",
                "/user",
                Some(vec![("id", "alex+tes")]),
                None,
            )
            .unwrap(),
            "https://api.example.com/user?id=alex%2Btes"
        )
    }

    #[test]
    fn test_v2_format_url() {
        assert_eq!(
            FormatUrlV2::new("https://api.example.com/")
                .with_path_template("/user/:name")
                .with_substitutes(vec![("name", "alex")])
                .with_query_params(vec![("active", "true")])
                .format_url()
                .unwrap(),
            "https://api.example.com/user/alex?active=true"
        )
    }
}
