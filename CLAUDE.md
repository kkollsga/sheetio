# Sheet Excavator - Developer Guide

## Build / Test / Lint Commands

```bash
make dev    # maturin develop --release
make test   # pytest tests/ -x
make lint   # cargo fmt --check + clippy + ruff format --check + ruff check
make fmt    # cargo fmt + ruff format + ruff check --fix
make cov    # pytest with coverage
make clean  # cargo clean + remove build artifacts
```

Note: If both VIRTUAL_ENV and CONDA_PREFIX are set, unset CONDA_PREFIX before running maturin.

## Architecture

Single PyO3 function `excel_extract(file_paths, extraction_details, num_workers)` returns JSON string.

```
lib.rs          -> Entry point, PyO3 module definition
parallel.rs     -> Async file processing with tokio
read_excel.rs   -> Per-file extraction orchestration
utils/
  single_cells.rs       -> Extract individual cells by address
  multirow_patterns.rs  -> Extract rows with unique_id, stop conditions
  dataframe.rs          -> Extract tabular data with multi-row headers
  conversions.rs        -> Cell address parsing (e.g., "B12" -> row, col)
  helpers.rs            -> Unique key generation, sheet matching
  manipulations.rs      -> Cell value extraction and type conversion
  parsed_config.rs      -> Pre-parsed config structs (performance optimization)
```

## API Change Checklist

When modifying the `excel_extract` function or its behavior:

1. Update Rust source code
2. Update `sheet_excavator.pyi` type stub
3. Update CHANGELOG.md (under [Unreleased])
4. Update README.md if user-facing

## Commit Conventions

Format: `type: description`

Types: feat, fix, docs, refactor, test, chore, ci

## Release Process

1. Bump version in `Cargo.toml`
2. Move [Unreleased] entries to new version section in CHANGELOG.md
3. Push to main -- CI runs lint/test, then build_wheels publishes to PyPI

Note: PyPI trusted publishing requires OIDC configuration at:
https://pypi.org/manage/project/sheet-excavator/settings/publishing/
