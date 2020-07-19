# Changelog

The format is based on [Keep a Changelog](http://keepachangelog.com/en/1.0.0/)
and this project adheres to [Semantic Versioning](http://semver.org/spec/v2.0.0.html).

## [Unreleased]
## [0.5.5] - 2020-07-20 <a name="0.5.5"></a>
### Added
- Support for WASM [PR #26](https://github.com/amodm/webbrowser-rs/pull/26)

## [0.5.4] - 2020-06-09 <a name="0.5.4"></a>
### Fixed
- Fix README to reflect platform support for Android and Haiku

## [0.5.3] - 2020-06-09 <a name="0.5.3"></a>
### Changed
- Added support for Haiku (Untested right now!) [PR #21](https://github.com/amodm/webbrowser-rs/pull/21)
- Added support for Android [PR #19](https://github.com/amodm/webbrowser-rs/pull/19)
- Added support for kioclient and x-www-browser [PR #17](https://github.com/amodm/webbrowser-rs/pull/17)

## [0.5.2] - 2019-08-22 <a name="0.5.2"></a>
### Fixed
- Fix a COM leak bug on Windows [PR #15](https://github.com/amodm/webbrowser-rs/pull/15)

## [0.5.1] - 2019-04-01 <a name="0.5.1"></a>
### Fixed
- Fix the behaviour that open() was blocking on Linux and BSD family. [Issue #13](https://github.com/amodm/webbrowser-rs/issues/13)
- Fix tests on macos

## [0.5.0] - 2019-03-31 <a name="0.5.0"></a>
### Added
- Add BSD family to supported platforms. [PR #12](https://github.com/amodm/webbrowser-rs/pull/12)

## [0.4.0] - 2018-12-18 <a name="0.4.0"></a>
### Changed
- Use `ShellExecuteW` on Windows as the earlier approach of using cmd.exe was breaking on
  special characters. [PR #11](https://github.com/amodm/webbrowser-rs/pull/11)

### Fixed
- Fixed Apache Licensing format

## [0.3.1] - 2018-06-22 <a name="0.3.1"></a>
### Fixed
- Fix broken examples header. [PR #7](https://github.com/amodm/webbrowser-rs/pull/7)
- Fix undeclared reference to `env` that breaks Linux. [PR #8](https://github.com/amodm/webbrowser-rs/pull/8)

## [0.3.0] - 2018-06-18 <a name="0.3.0"></a>
### Changed
- Change the OS test to use conditional complication and raise a compile error if the target OS is unsupported. 
  [PR #6](https://github.com/amodm/webbrowser-rs/pull/6)
- Implement useful trait from StdLib for Browser such as `Display`, `Default` and `FromStr`.
  [PR #6](https://github.com/amodm/webbrowser-rs/pull/6)

### Fixed
- Fix the command in `open_on_windows` to use `cmd.exe` instead of `start`. [PR #5](https://github.com/amodm/webbrowser-rs/pull/5)

## [0.2.2] - 2017-01-23 <a name="0.2.2"></a>
### Fixed
- Honour the right syntax for `$BROWSER`. Closes [#3](https://github.com/amodm/webbrowser-rs/issues/3)
- Include `gvfs-open` and `gnome-open` for [#2](https://github.com/amodm/webbrowser-rs/issues/2)

## [0.2.1] - 2017-01-22 <a name="0.2.1"></a>
### Changed
- Honour `$BROWSER` env var on Linux, before choosing to fallback to `xdg-open`. [Issue #2](https://github.com/amodm/webbrowser-rs/issues/2)

## [0.1.3] - 2016-01-11 <a name="0.1.3"></a>
### Added
- Add Apache license [Issue #1](https://github.com/amodm/webbrowser-rs/issues/1)

## [0.1.2] - 2015-12-09 <a name="0.1.2"></a>
### Added
- Initial release.

[Unreleased]: https://github.com/amodm/webbrowser-rs/compare/v0.5.5...HEAD
[0.5.5]: https://github.com/amodm/webbrowser-rs/compare/v0.5.4...v0.5.5
[0.5.4]: https://github.com/amodm/webbrowser-rs/compare/v0.5.3...v0.5.4
[0.5.3]: https://github.com/amodm/webbrowser-rs/compare/v0.5.2...v0.5.3
[0.5.2]: https://github.com/amodm/webbrowser-rs/compare/v0.5.1...v0.5.2
[0.5.1]: https://github.com/amodm/webbrowser-rs/compare/v0.5.0...v0.5.1
[0.5.0]: https://github.com/amodm/webbrowser-rs/compare/v0.4.0...v0.5.0
[0.4.0]: https://github.com/amodm/webbrowser-rs/compare/v0.3.1...v0.4.0
[0.3.1]: https://github.com/amodm/webbrowser-rs/compare/v0.3.0...v0.3.1
[0.3.0]: https://github.com/amodm/webbrowser-rs/compare/v0.2.2...v0.3.0
[0.2.2]: https://github.com/amodm/webbrowser-rs/compare/v0.2.1...v0.2.2
[0.2.1]: https://github.com/amodm/webbrowser-rs/compare/v0.1.3...v0.2.1
[0.1.3]: https://github.com/amodm/webbrowser-rs/compare/v0.1.2...v0.1.3
