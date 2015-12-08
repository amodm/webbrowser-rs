# webbrowser

Rust library to open URLs in the web browsers available on a platform

Inspired by the [webbrowser](https://docs.python.org/2/library/webbrowser.html) python library

##Examples

```rust
use webbrowser;

if webbrowser::open("http://github.com").is_ok() {
    // ...
}
```

Currently state of platform support is:

* macos => default, as well as browsers listed under [Browser](enum.Browser.html)
* windows => default browser only
* linux => default browser only
* android => not supported right now
* ios => not supported right now

Important note:

* This library requires availability of browsers and a graphical environment during runtime
* `cargo test` will actually open the browser locally

License: MIT
