use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
use serde::Serialize;
use std::collections::HashMap;

#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum SubstituteValue {
    String(String),
    Int(i32),
    Float(f32),
}

fn string_from_substitute_value(value: &SubstituteValue) -> String {
    match value {
        SubstituteValue::String(string) => string.to_string(),
        SubstituteValue::Int(int) => int.to_string(),
        SubstituteValue::Float(float) => float.to_string(),
    }
}

fn encoded_string_from_substitute_value(value: &SubstituteValue) -> String {
    utf8_percent_encode(&string_from_substitute_value(value), NON_ALPHANUMERIC).to_string()
}

fn strip_double_slash<'a>(base_url: &str, route_template: &'a str) -> &'a str {
    if base_url.ends_with("/") && route_template.starts_with("/") {
        &route_template[1..]
    } else {
        route_template
    }
}

fn format_path(route_template: &str, substitutes: HashMap<String, SubstituteValue>) -> String {
    substitutes
        .into_iter()
        .fold(route_template.to_owned(), |route, (key, value)| {
            route.replace(
                &format!(":{}", key),
                &encoded_string_from_substitute_value(&value),
            )
        })
}

pub fn format_url(
    base_url: &str,
    path_template: &str,
    query_params: Option<HashMap<String, SubstituteValue>>,
    substitutes: Option<HashMap<String, SubstituteValue>>,
) -> Result<String, serde_urlencoded::ser::Error> {
    let formatted_route = substitutes.map_or_else(
        || path_template.to_string(),
        |substitutes| format_path(path_template, substitutes),
    );

    let formatted_querystring = query_params.map_or_else(
        || Ok(String::new()),
        |query_params| {
            let query_string = serde_urlencoded::to_string(query_params)?;
            Ok(String::from("?") + (&query_string))
        },
    )?;

    let safe_formatted_route = strip_double_slash(base_url, &formatted_route);

    Ok(format!(
        "{}{}{}",
        base_url, safe_formatted_route, formatted_querystring
    ))
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::{format_url, SubstituteValue};

    #[test]
    fn accepts_empty_path() {
        assert_eq!(
            format_url("https://api.example.com", "", None, None),
            Ok("https://api.example.com".to_string())
        );
    }

    #[test]
    fn adds_path_to_base() {
        assert_eq!(
            format_url("https://api.example.com", "/user", None, None),
            Ok("https://api.example.com/user".to_string())
        );
    }

    #[test]
    fn strips_double_slash() {
        assert_eq!(
            format_url("https://api.example.com/", "/user", None, None),
            Ok("https://api.example.com/user".to_string())
        );
    }

    #[test]
    fn adds_path_substitutes() {
        assert_eq!(
            format_url(
                "https://api.example.com/",
                "/user/:id",
                None,
                Some(HashMap::from([(
                    "id".to_string(),
                    SubstituteValue::String("alextes".to_string())
                )]),)
            ),
            Ok("https://api.example.com/user/alextes".to_string())
        );
    }

    #[test]
    fn adds_querystring() {
        assert_eq!(
            format_url(
                "https://api.example.com/",
                "/user",
                Some(HashMap::from([(
                    "id".to_string(),
                    SubstituteValue::String(String::from("alextes"))
                )])),
                None
            ),
            Ok("https://api.example.com/user?id=alextes".to_string())
        );
    }

    #[test]
    fn percent_encodes_substitutes() {
        assert_eq!(
            format_url(
                "https://api.example.com/",
                "/user/:id",
                None,
                Some(HashMap::from([(
                    "id".to_string(),
                    SubstituteValue::String("alex tes".to_string())
                )])),
            ),
            Ok("https://api.example.com/user/alex%20tes".to_string())
        )
    }

    #[test]
    fn percent_encodes_query_params() {
        assert_eq!(
            format_url(
                "https://api.example.com/",
                "/user",
                Some(HashMap::from([(
                    "id".to_string(),
                    SubstituteValue::String("alex+tes".to_string())
                )])),
                None,
            ),
            Ok("https://api.example.com/user?id=alex%2Btes".to_string())
        )
    }
}
