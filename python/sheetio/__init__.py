"""Fast Rust-powered Excel form data extraction to JSON."""

try:
    from sheetio._sheetio import excel_extract
except ImportError:
    excel_extract = None  # Native module unavailable (e.g. docs build)

from sheetio.config import ExtractionConfig

__all__ = ["excel_extract", "ExtractionConfig"]
