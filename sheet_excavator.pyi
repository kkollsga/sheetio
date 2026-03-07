"""Fast Rust-powered Excel form data extraction to JSON."""

def excel_extract(
    file_paths: list[str],
    extraction_details: list[dict],
    num_workers: int | None = None,
) -> str:
    """Extract data from Excel files based on extraction rules.

    Args:
        file_paths: List of paths to Excel files (.xlsx, .xlsm).
        extraction_details: List of extraction configuration dicts.
        num_workers: Number of parallel workers. Defaults to 5.

    Returns:
        JSON string containing extracted data keyed by filename and sheet.
    """
    ...
