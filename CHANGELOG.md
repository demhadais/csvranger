# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## 0.3.0 (2026-06-02)
### Breaking Changes
- The function `parse_legacy_csv_value_as_f64` previously did a naive calculation that ends up with imprecise floating point values. It has now been updated such that strings like `"98.2%"` are correctly parsed as `0.982`.

## 0.2.0 (2026-05-29)
### Breaking Changes
- Previously, deserializing a `TenxCsvValue` with the `legacy` feature enabled would fail for anything non-string. Now, it tries to deserialize as an integer, then a float, finally falling back to the complex CSV parsing if necessary. This is a bug fix but also a breaking change.

## 0.1.2 (2026-05-27)
### Changed
- Added another optimistic check when parsing legacy CSVs that prevents using the regular expression, which improves performance. This is the main path, so checking the regex is avoided substantially more often now.

## 0.1.1 (2026-05-27)
### Fixes
- fix a small typo in the crate documentation.

## 0.1.0 (2026-05-27)
This is the first release of `csvranger`, which includes the enum `TenxCsvValue` and the two functions `parse_legacy_csv_value_as_f64` and `parse_legacy_csv_value_as_i64` (`legacy` feature only).
