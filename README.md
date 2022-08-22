# format-url

Format URLs for fetch requests using templates and substitution values.

This library is currently considering a pure function based design where the data required is essentially constructed in the function call, and a builder pattern design.

## Wishlist
* Accept any struct as a source for path template substitutes as long as all values can be converted to string.
* Accept any struct as a source for query params as long as all values can be converted to string.
* Support for lists and nested values.
