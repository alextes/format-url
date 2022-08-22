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
//!     .format_url();
//!
//! assert_eq!(url, "https://api.example.com/user/alex?active=true");
//! ```
//!
//! ## Wishlist
//! * Support for lists and nested values. (serde_urlencoded -> serde_qs)
//! * Support receiving query params as any value serde_urlencoded or serde_qs can serialize.
//! * Support receiving path template substitutes as a (Hash)Map, perhaps even a struct with
//! matching fields.

use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};

type SubstitutePairs<'a> = Vec<(&'a str, &'a str)>;
type QueryParams<'a> = Vec<(&'a str, &'a str)>;

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
                &format!(":{key}"),
                &utf8_percent_encode(&value, NON_ALPHANUMERIC).to_string(),
            )
        })
}

fn naive_encode_query_string<'a>(query_params: &QueryParams<'a>) -> String {
    let query_string = query_params
        .iter()
        .map(|(key, value)| {
            format!(
                "{}={}",
                utf8_percent_encode(key, NON_ALPHANUMERIC),
                utf8_percent_encode(value, NON_ALPHANUMERIC)
            )
        })
        .collect::<Vec<String>>()
        .join("&");

    "?".to_string() + (&query_string)
}

/// A collection of all the components and configuration that together serialize into a URL.
pub struct FormatUrl<'a> {
    base: &'a str,
    disable_encoding: bool,
    path_template: Option<&'a str>,
    query_params: Option<QueryParams<'a>>,
    substitutes: Option<SubstitutePairs<'a>>,
}

impl<'a> FormatUrl<'a> {
    /// In rare cases you may need the query parameter key/value pairs not to be encoded.
    pub fn disable_encoding(mut self) -> Self {
        self.disable_encoding = true;
        self
    }

    /// Takes all of the provided arguments and turns them into a single URL to fetch.
    pub fn format_url(self) -> String {
        let formatted_path = match (self.path_template, &self.substitutes) {
            (Some(path_template), Some(substitutes)) => format_path(path_template, &substitutes),
            (Some(path_template), _) => path_template.to_string(),
            _ => String::from(""),
        };

        let formatted_querystring = &self.query_params.map_or_else(
            || String::new(),
            |query_params| match self.disable_encoding {
                false => naive_encode_query_string(&query_params),
                true => {
                    let query_string = query_params
                        .iter()
                        .map(|(key, value)| format!("{key}={value}"))
                        .collect::<Vec<String>>()
                        .join("&");
                    "?".to_string() + &query_string
                }
            },
        );

        let safe_formatted_route = strip_double_slash(self.base, &formatted_path);

        format!(
            "{}{}{}",
            self.base, safe_formatted_route, formatted_querystring
        )
    }

    /// Start building a URL. The minimum required is some hostname.
    pub fn new(base: &'a str) -> Self {
        Self {
            base,
            disable_encoding: false,
            path_template: None,
            query_params: None,
            substitutes: None,
        }
    }

    /// Add a path, optionally marking sections for substitution using `:key`.
    pub fn with_path_template(mut self, path_template: &'a str) -> Self {
        self.path_template = Some(path_template);
        self
    }

    /// Add some query parameters.
    pub fn with_query_params(mut self, params: QueryParams<'a>) -> Self {
        self.query_params = Some(params);
        self
    }

    /// Add substitutes to substitute matching `:key` sequences in the path template.
    pub fn with_substitutes(mut self, substitutes: SubstitutePairs<'a>) -> Self {
        self.substitutes = Some(substitutes);
        self
    }
}

#[cfg(test)]
mod tests {
    use crate::FormatUrl;

    #[test]
    fn no_formatting_test() {
        assert_eq!(
            FormatUrl::new("https://api.example.com").format_url(),
            "https://api.example.com".to_string()
        );
    }

    #[test]
    fn path_test() {
        assert_eq!(
            FormatUrl::new("https://api.example.com",)
                .with_path_template("/user")
                .format_url(),
            "https://api.example.com/user"
        );
    }

    #[test]
    fn strip_double_slash_test() {
        assert_eq!(
            FormatUrl::new("https://api.example.com/")
                .with_path_template("/user")
                .format_url(),
            "https://api.example.com/user"
        );
    }

    #[test]
    fn path_substitutes_test() {
        assert_eq!(
            FormatUrl::new("https://api.example.com/",)
                .with_path_template("/user/:id",)
                .with_substitutes(vec![("id", "alextes")])
                .format_url(),
            "https://api.example.com/user/alextes"
        );
    }

    #[test]
    fn querystring_test() {
        assert_eq!(
            FormatUrl::new("https://api.example.com/user",)
                .with_query_params(vec![("id", "alextes")],)
                .format_url(),
            "https://api.example.com/user?id=alextes"
        );
    }

    #[test]
    fn percent_encode_substitutes_test() {
        assert_eq!(
            FormatUrl::new("https://api.example.com/",)
                .with_path_template("/user/:id",)
                .with_substitutes(vec![("id", "alex tes")])
                .format_url(),
            "https://api.example.com/user/alex%20tes"
        )
    }

    #[test]
    fn percent_encode_query_params_test() {
        assert_eq!(
            FormatUrl::new("https://api.example.com/user",)
                .with_query_params(vec![("id", "alex+tes")],)
                .format_url(),
            "https://api.example.com/user?id=alex%2Btes"
        )
    }

    #[test]
    fn disable_encoding_test() {
        assert_eq!(
            FormatUrl::new("https://api.example.com/user",)
                .with_query_params(vec![("id", "alex+tes")],)
                .disable_encoding()
                .format_url(),
            "https://api.example.com/user?id=alex+tes"
        )
    }
}
