use percent_encoding::{utf8_percent_encode, CONTROLS};
use serde::Serialize;
use std::collections::HashMap;

#[derive(Debug, Serialize)]
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

pub fn format_url(
    base_url: &str,
    route_template: &str,
    substitutes: HashMap<String, SubstituteValue>,
) -> String {
    let safe_route = if base_url.ends_with("/") && route_template.starts_with("/") {
        &route_template[1..]
    } else {
        route_template
    };

    let mut query_params = HashMap::new();

    let mut formatted_route = safe_route.to_owned();

    for (key, value) in substitutes {
        if formatted_route.contains(&format!(":{}", key)) {
            let encoded = match value {
                SubstituteValue::String(string) => {
                    utf8_percent_encode(&string, CONTROLS).to_string()
                }
                SubstituteValue::Int(int) => int.to_string(),
                SubstituteValue::Float(float) => float.to_string(),
            };

            formatted_route = formatted_route.replace(&format!(":{}", key), &encoded);
        } else {
            let value_str = string_from_substitute_value(&value);

            query_params.insert(key, value_str);
        };
    }

    let formatted_querystring = serde_qs::to_string(&query_params).unwrap();

    let safe_query_string = if formatted_querystring.is_empty() {
        "".to_owned()
    } else {
        format!("?{}", formatted_querystring)
    };

    format!("{}{}{}", base_url, formatted_route, safe_query_string)
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::{format_url, SubstituteValue};

    #[test]
    fn accepts_empty_path() {
        assert_eq!(
            format_url("https://api.example.com", "", HashMap::new()),
            "https://api.example.com"
        );
    }

    #[test]
    fn adds_path_to_base() {
        assert_eq!(
            format_url("https://api.example.com", "/user", HashMap::new()),
            "https://api.example.com/user"
        );
    }

    #[test]
    fn strips_double_slash() {
        assert_eq!(
            format_url("https://api.example.com/", "/user", HashMap::new()),
            "https://api.example.com/user"
        );
    }

    #[test]
    fn adds_path_substitutes() {
        assert_eq!(
            format_url(
                "https://api.example.com/",
                "/user/:id",
                HashMap::from([(
                    "id".to_string(),
                    SubstituteValue::String("alextes".to_string())
                )]),
            ),
            "https://api.example.com/user/alextes"
        );
    }

    #[test]
    fn adds_querystring() {
        assert_eq!(
            format_url(
                "https://api.example.com/",
                "/user",
                HashMap::from([(
                    "id".to_string(),
                    SubstituteValue::String("alextes".to_string())
                )]),
            ),
            "https://api.example.com/user?id=alextes"
        );
    }
}
