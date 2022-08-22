//! Format URLs for fetch requests using templates and substitution values.
//!
//! ## Usage
//! ```
//! use format_url::FormatUrl;
//!
//! let url = FormatUrl::new("https://api.example.com/")
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
//! * No need to annotate generic T for FormatUrl

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

pub struct FormatUrl<'a, T: Serialize> {
    base: &'a str,
    path_template: Option<&'a str>,
    query_params: Option<T>,
    substitutes: Option<SubstitutePairs<'a>>,
}

impl<'a, T: Serialize> FormatUrl<'a, T> {
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
    use crate::{FormatUrl, SubstitutePairs};

    #[test]
    fn no_formatting_test() {
        assert_eq!(
            FormatUrl::<SubstitutePairs>::new("https://api.example.com").format_url(),
            Ok("https://api.example.com".to_string())
        );
    }

    #[test]
    fn path_test() {
        assert_eq!(
            FormatUrl::<SubstitutePairs>::new("https://api.example.com",)
                .with_path_template("/user")
                .format_url()
                .unwrap(),
            "https://api.example.com/user"
        );
    }

    #[test]
    fn strip_double_slash_test() {
        assert_eq!(
            FormatUrl::<SubstitutePairs>::new("https://api.example.com/")
                .with_path_template("/user")
                .format_url()
                .unwrap(),
            "https://api.example.com/user"
        );
    }

    #[test]
    fn path_substitutes_test() {
        assert_eq!(
            FormatUrl::<SubstitutePairs>::new("https://api.example.com/",)
                .with_path_template("/user/:id",)
                .with_substitutes(vec![("id", "alextes")])
                .format_url()
                .unwrap(),
            "https://api.example.com/user/alextes"
        );
    }

    #[test]
    fn querystring_test() {
        assert_eq!(
            FormatUrl::new("https://api.example.com/user",)
                .with_query_params(vec![("id", "alextes")],)
                .format_url()
                .unwrap(),
            "https://api.example.com/user?id=alextes"
        );
    }

    #[test]
    fn percent_encode_substitutes_test() {
        assert_eq!(
            FormatUrl::<SubstitutePairs>::new("https://api.example.com/",)
                .with_path_template("/user/:id",)
                .with_substitutes(vec![("id", "alex tes")])
                .format_url()
                .unwrap(),
            "https://api.example.com/user/alex%20tes"
        )
    }

    #[test]
    fn percent_encode_query_params_test() {
        assert_eq!(
            FormatUrl::<SubstitutePairs>::new("https://api.example.com/user",)
                .with_query_params(vec![("id", "alex+tes")],)
                .format_url()
                .unwrap(),
            "https://api.example.com/user?id=alex%2Btes"
        )
    }
}
