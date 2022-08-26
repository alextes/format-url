Format URLs for fetch requests using templates and substitution values.

## Usage
```
use format_url::FormatUrl;

let url = FormatUrl::new("https://api.example.com/")
    .with_path_template("/user/:name")
    .with_substitutes(vec![("name", "alex")])
    .with_query_params(vec![("active", "true")])
    .format_url();

assert_eq!(url, "https://api.example.com/user/alex?active=true");
```

## Wishlist
* Support for lists and nested values. (serde_urlencoded -> serde_qs)
* Support receiving query params as any value serde_urlencoded or serde_qs can serialize.
* Support receiving path template substitutes as a (Hash)Map, perhaps even a struct with
matching fields.
