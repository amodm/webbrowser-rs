# webbrowser

[![Current Crates.io Version](https://img.shields.io/crates/v/webbrowser.svg)](https://crates.io/crates/webbrowser)
[![Crates.io Downloads](https://img.shields.io/crates/d/webbrowser.svg)](https://crates.io/crates/webbrowser)
[![License](https://img.shields.io/crates/l/webbrowser.svg)](LICENSE-MIT)

![Linux Build](https://github.com/amodm/webbrowser-rs/workflows/Linux/badge.svg?branch=master&x=1)
![Windows Build](https://github.com/amodm/webbrowser-rs/workflows/Windows/badge.svg?branch=master&x=1)
![MacOS Build](https://github.com/amodm/webbrowser-rs/workflows/MacOS/badge.svg?branch=master&x=1)
![Android Build](https://github.com/amodm/webbrowser-rs/workflows/Android/badge.svg?branch=master&x=1)
![WASM Build](https://github.com/amodm/webbrowser-rs/workflows/WASM/badge.svg?branch=master&x=1)

Rust library to open URLs in the web browsers available on a platform

Inspired by the [webbrowser](https://docs.python.org/2/library/webbrowser.html) python library

## Documentation

- [API Reference](http://code.rootnet.in/webbrowser-rs/webbrowser/)
- [Release Notes](CHANGELOG.md)

## Examples

```rust
use webbrowser;

if webbrowser::open("http://github.com").is_ok() {
    // ...
}
```

Currently state of platform support is:

* macos => default, as well as browsers listed under [Browser](enum.Browser.html). UTF-8 tests currently fail on Github Runner, but run fine on local, so YMMV.
* windows => default browser only
* linux/*bsd => default browser only (uses $BROWSER env var, failing back to xdg-open, gvfs-open, gnome-open, whichever works first)
* android => default browser only
* haiku => untested and experimental
* wasm -> untested and experimental
* ios => not supported right now

Important note:

* This library requires availability of browsers and a graphical environment during runtime
* `cargo test` will actually open the browser locally

## PRs invited for
* Fixing macos tests for UTF-8 URLs
* Support for other platforms, e.g. iOS

## License

Licensed under either of

* Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
* MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any
additional terms or conditions.
