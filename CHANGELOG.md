# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog], and this project adheres to
[Semantic Versioning].

[Keep a Changelog]: https://keepachangelog.com/en/1.1.0/
[Semantic Versioning]: https://semver.org/spec/v2.0.0.html

## [Unreleased]

### Changed

- BREAKING CHANGE: `Daterange::start_date` return type has been changed
  from `DateTime` to `Option<DateTime>`. This is based on the update in
  draft 18 where it was clarified that all daterange attributes, other
  than `ID`, are optional on an instance of the tag because the values
  are inherited from previous instances of the tag. Given that parsing
  in this library occurs line-by-line, we must consider this optional
  now.

## [0.6.0] - 2025-08-19

### Added

- `TagValue::try_as_ordered_attribute_list` which provides a `Vec` over
  the tag attributes (instead of the `HashMap` that the previous
  attribute list offers). This can be used if order of attributes is
  important for the user.

### Changed

- Internally all "attribute list" tags now use the ordered attribute
  list method for finding attributes (non-breaking change).

## [0.5.0] - 2025-08-16

### Changed
- Reorganized public module structure in hope of being easier to follow
  for outside users. More information can be found here: [#3].

### Fixed
- Several type signatures were made more clear by being explicit about
  elided lifetimes (based on new suggestions from `cargo clippy` as of
  `rustc` 1.89.0).
- Documentation fix for custom tag parsing in `Reader` example.

[#3]: https://github.com/theRealRobG/m3u8/pull/3

## [0.4.0] - 2025-08-15

### Added

- Documentation across the whole library.
- `Display` implementation for `DateTime`.
- Input validation on `date_time!` macro.
- `GetKnown` trait and impl on `Option<EnumeratedString<T>>` to make
  getting optional known values from tags easier.
- `AllowedCpc` abstraction.
- `FromIterator` for `EnumeratedStringList`
- Several new error types to match the new tag value approach.

### Changed

- Made `HlsLine::Uri` and `HlsLine::Comment` hold `Cow` for easier
  mutability and user constructed values.
- Less generic constraints on struct definition for several types,
  including `CustomTag`, `EnumeratedString`, and `EnumeratedStringList`.
- Completely changed how `CustomTag` is implemented by the user (and
  documented the updates). The changes improve write performance where
  there is no mutation of the tag and also improves the ergonomics of
  how the user provides the tag value in cases of mutation.
- Builder syntax for all tags updated so that `finish` method is not
  available until all required properties are set, and required
  properties have been removed from builder constructor. This allows all
  properties to be named (including the required ones) and seems like a
  better design.
- Changed `SemiParsedTagValue` (and associated types) to `TagValue` (and
  associated types). This moves context of what value should be
  attempted to be parsed from the input data to wherever is asking for
  the value, which for the HLS tags, means that the desired value type
  will be known and so parsing can be optimized.
- Changed lib name from `m3u8` to `quick-m3u8` (due to name clash on
  Crates.io).
- `StreamInf::allowed_cpc` returns `Option<AllowedCpc>` instead of
  `Option<&str>`.

### Removed

- `Writer::write_hls_tag` method (prefer combination of `HlsLine::from`
  and `Writer::write_hls_line`).
- Constructor methods for all tags (prefer builder pattern).
- Constructor method for `ParsingOptions` (prefer builder pattern).

### Fixed

- `DateTime` parsing allows for a space separator between date and time
  (not just `T` or `t`).
- Made `EnumeratedStringList::is_empty` behavior match
  `EnumeratedStringListIter::count` is zero behvior.

[unreleased]: https://github.com/theRealRobG/m3u8/compare/0.6.0...HEAD
[0.6.0]: https://github.com/theRealRobG/m3u8/compare/0.5.0...0.6.0
[0.5.0]: https://github.com/theRealRobG/m3u8/compare/0.4.0...0.5.0
[0.4.0]: https://github.com/theRealRobG/m3u8/releases/tag/0.4.0