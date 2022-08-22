# format-url

Format URLs for fetch requests using templates and substitution values.

This library is currently considering a pure function based design where the data required is essentially constructed in the function call, and a builder pattern design.

## Usage - fn pattern
```rs
let url = format_url(
    "https://api.example.com/",
    "/user",
    Some(HashMap::from([("id".to_string(), "alex+tes".to_string())])),
    None,
);
// Ok("https://api.example.com/user?id=alex%2Btes".to_string())
```

## Usage - builder pattern
```rs
let url = FormatUrlV2::new("https://api.example.com/")
    .with_path_template("/user/:name")
    .with_substitutes(HashMap::from([(
        String::from("name"),
        String::from("alex")
    )]))
    .with_query_params(HashMap::from([(
        String::from("active"),
        String::from("true")
    )]))
    .format_url()
    .unwrap();
// "https://api.example.com/user/alex?active=true"
```

## Wishlist
* Accept any struct as a source for path template substitutes as long as all values can be converted to string.
* Accept any struct as a source for query params as long as all values can be converted to string.
* Support for lists and nested values.
