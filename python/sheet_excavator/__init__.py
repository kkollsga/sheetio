"""Fast Rust-powered Excel form data extraction to JSON."""

from sheet_excavator._sheet_excavator import excel_extract
from sheet_excavator.config import ExtractionConfig

__all__ = ["excel_extract", "ExtractionConfig"]
