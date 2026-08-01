# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Fixed

- Row-mode `stop_if_empty` now treats a cell holding an empty or whitespace-only
  string as empty, matching column-mode. Previously row-mode only tested for null,
  so a separator row made of empty strings did not stop extraction and unrelated
  trailing rows were pulled in. Most visible on legacy `.xls` files, where a blank
  row can be stored as empty strings rather than omitted cells.

### Added

- Test coverage for legacy `.xls` (BIFF) files, which the openpyxl-based fixtures
  could not produce. Tests require `xlwt`.

## [0.3.1] - 2026-03-07

### Changed

- Renamed package from `sheet-excavator` to `sheetio`
- MkDocs documentation site with Material theme

## [0.3.0] - 2026-03-07

### Added

- `ExtractionConfig` builder class for constructing extraction configs with method chaining
- JSON import/export for configs (`.to_json()` / `.from_json()`)
- Direct extraction via `.extract()` returning parsed Python dicts
- `.summary()` for human-readable config inspection
- Mixed Rust/Python package structure (maturin)

## [0.2.4] - 2026-01-30

### Changed

- Performance optimizations: Arc sharing for extraction config, pre-parsed config structs, HashSet/IndexSet for lookups
- Python 3.10+ requirement

## [0.2.3] - 2025-10-14

### Added

- Composite unique_id support for multirow patterns (array of column letters)
- Custom unique_id_separator option
- Optional unique_id (omit to get array output instead of dict)
- stop_if_empty support with column, row, and consecutive modes

## [0.2.0] - 2025-03-11

### Added

- Initial public release
- Three extraction functions: single_cells, multirow_patterns, dataframe
- Wildcard sheet matching with glob patterns
- Parallel file processing with configurable worker count
- break_if_null and skip_sheets support
- GitHub Actions workflow for building and publishing wheels

[Unreleased]: https://github.com/kkollsga/sheetio/compare/v0.3.1...HEAD
[0.3.1]: https://github.com/kkollsga/sheetio/compare/v0.3.0...v0.3.1
[0.3.0]: https://github.com/kkollsga/sheetio/compare/v0.2.4...v0.3.0
[0.2.4]: https://github.com/kkollsga/sheetio/compare/v0.2.3...v0.2.4
[0.2.3]: https://github.com/kkollsga/sheetio/compare/v0.2.0...v0.2.3
[0.2.0]: https://github.com/kkollsga/sheetio/releases/tag/v0.2.0
