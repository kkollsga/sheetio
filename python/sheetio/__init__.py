"""Fast Rust-powered Excel form data extraction to JSON."""

from sheetio._sheetio import excel_extract
from sheetio.config import ExtractionConfig

__all__ = ["excel_extract", "ExtractionConfig"]
