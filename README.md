# Sheet Excavator -- Fast Rust-Powered Excel Form Data Extraction

[![PyPI version](https://badge.fury.io/py/sheet-excavator.svg)](https://badge.fury.io/py/sheet-excavator)
[![CI](https://github.com/kkollsga/sheet-excavator/actions/workflows/ci.yml/badge.svg)](https://github.com/kkollsga/sheet-excavator/actions/workflows/ci.yml)
[![Python 3.10+](https://img.shields.io/badge/python-3.10+-blue.svg)](https://www.python.org/downloads/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

Extract structured data from Excel forms (.xlsx, .xlsm) into JSON. Built in Rust with Python bindings via PyO3 for fast parallel processing of hundreds of files. Designed for standardized Excel forms that don't fit the typical CSV format -- government reports, engineering forms, financial templates, and survey spreadsheets.

## Why Sheet Excavator?

- **Fast** -- Rust core with async parallel file processing. Process hundreds of Excel forms in seconds.
- **Flexible** -- Three extraction modes (single cells, row patterns, dataframes) handle any form layout.
- **Simple** -- One function, JSON config, JSON output. No complex API to learn.
- **Robust** -- Handles missing data, duplicate keys, wildcard sheet matching, and composite identifiers.

## Quick Start

```
pip install sheet-excavator
```

```python
import sheet_excavator
import json

files = ["form_001.xlsx", "form_002.xlsx", "form_003.xlsx"]

config = [
    {
        "sheets": ["Sheet1"],
        "extractions": [
            {
                "function": "single_cells",
                "label": "header",
                "instructions": {
                    "title": "b2",
                    "date": "d4",
                    "author": "b6"
                }
            },
            {
                "function": "multirow_patterns",
                "label": "items",
                "instructions": {
                    "row_range": [10, 100],
                    "unique_id": "A",
                    "stop_if_empty": "A",
                    "columns": {
                        "ID": "A",
                        "Description": "B",
                        "Value": "C"
                    }
                }
            }
        ]
    }
]

result = json.loads(sheet_excavator.excel_extract(files, config, 5))
```

## Config Builder

Use `ExtractionConfig` to build configs iteratively with method chaining, instead of writing raw dicts:

```python
from sheet_excavator import ExtractionConfig

config = ExtractionConfig()

# Add extractions with a fluent API
config.add_sheets(["Sheet1"]) \
    .single_cells("header", title="B2", date="D4", author="B6") \
    .multirow("items",
        row_range=(10, 100),
        unique_id="A",
        stop_if_empty="A",
        columns={"ID": "A", "Description": "B", "Value": "C"})

# Extract directly — returns parsed Python dicts (no json.loads needed)
result = config.extract(["form_001.xlsx", "form_002.xlsx"], workers=5)

# Save / load configs as JSON files
config.to_json("my_config.json")
config = ExtractionConfig.from_json("my_config.json")

# Inspect what you've built
config.summary()

# Or get the raw config list for manual use
raw = config.build()
```

All three extraction types are supported: `.single_cells()`, `.multirow()`, `.dataframe()`. See the [Extraction Types Reference](#extraction-types-reference) below for all available options.

## Key Features

| Feature | Description |
|---------|-------------|
| Parallel processing | Process multiple files simultaneously with configurable worker count |
| Wildcard sheets | Match sheets by pattern: `"School_*"` matches School_A, School_B, etc. |
| Composite keys | Combine multiple columns as unique identifiers: `["Project", "Year"]` |
| Gap tolerance | Continue extraction through empty rows with `stop_consecutive` |
| Duplicate handling | Automatic `_1`, `_2` suffixes for duplicate keys |
| Multi-column merge | Extract arrays from multiple columns per field: `["X", "Y", "Z"]` |
| Multi-row headers | Concatenate header rows for dataframe extraction |

## Requirements

- Python 3.10 or higher
- Supported platforms: Windows, macOS, Linux

## Extraction Types Reference

### Configuration Structure

The `extraction_details` parameter is a list of dictionaries that define the extraction rules for each Excel sheet. Each dictionary contains:
* `sheets`: A list of sheet names to extract data from. Accepts patterns with `*`. Example: `"School_*"` will loop through sheets like School_A, School_B, etc.
* `skip_sheets`: An optional list of sheet names to skip. Can be useful when using patterns in the list of sheets.
* `extractions`: A list of extraction rules that will be applied to the sheets listed.

Each extraction rule contains:
* `function`: Type of extraction function: `single_cells`, `multirow_patterns`, or `dataframe`.
* `label`: Optional key string to store results under. If not specified the extracted key value pairs will be stored directly under the sheet name.
* `break_if_null`: An optional check to skip sheet if specified cell is null.
* `instructions`: Instructions for the extraction function. See details for each function type below.

### Single Cells Extraction

Extracts individual cells from the Excel sheet.

**Instructions:**
* `instructions`: A dictionary where the keys are the reference name and the values are the cell references (e.g., `"a1"`, `"b2"`).

**Example:**
```python
{
    "sheets": ["Sheet1"],
    "extractions": [
        {
            "function": "single_cells",
            "label": "single",
            "break_if_null": "c3",
            "instructions": {
                "Value 1": "a1",
                "Value 2": "b2",
                "Value 3": "c3",
                "Date": "d4",
                "Datetime": "e5"
            }
        }
    ]
}
```

### Multirow Patterns Extraction

Extracts data from multiple rows based on a pattern.

**Instructions:**
* `row_range`: A list of two integers defining the row range to extract.
* `unique_id` (optional): The column(s) to use as a unique identifier. Can be either:
  - A single column as a string: `"B"`
  - Multiple columns as an array: `["B", "C"]` for composite keys
  - When using composite keys, if ANY column contains null/empty values, the row is skipped
  - **If omitted**: Results are returned as an array/list instead of a dictionary
* `unique_id_separator` (optional): The separator to use when joining multiple columns for composite keys. Defaults to `"_"`.
* `columns`: A dictionary where the keys are the column names and the values are the column letters (e.g., `"B"`, `"C"`).
* `stop_if_empty` (optional): Controls when to stop processing rows. Can be:
  - A column string: `"A"` - Stop when this column is empty
  - An array of columns: `["A", "B"]` - Stop when ALL specified columns are empty
  - The string `"row"` - Stop when the entire data row is empty
  - An object with detailed configuration:
    ```python
    {"column": "A", "consecutive": 2}
    ```
    or
    ```python
    {"mode": "row", "consecutive": 1}
    ```
* `stop_consecutive` (optional): Used with simple `stop_if_empty` syntax to specify how many consecutive empty rows trigger a stop. Defaults to `1`.

**Example with single unique_id:**
```python
{
    "sheets": ["Sheet 1", "Sheet 2"],
    "extractions": [
        {
            "function": "multirow_patterns",
            "label": "deposits",
            "instructions": {
                "row_range": [1, 10],
                "unique_id": "B",
                "columns": {
                    "Title": "B",
                    "Description": "C",
                    "Estimate": "D",
                    "Chance": "E",
                }
            }
        }
    ]
}
```

**Example with composite unique_id:**
```python
{
    "sheets": ["Sheet 1"],
    "extractions": [
        {
            "function": "multirow_patterns",
            "label": "projects",
            "instructions": {
                "row_range": [1, 50],
                "unique_id": ["B", "C"],
                "unique_id_separator": "-",
                "columns": {
                    "Project": "B",
                    "Year": "C",
                    "Budget": "D",
                    "Status": "E"
                }
            }
        }
    ]
}
```

**Example without unique_id (returns array/list):**
```python
{
    "sheets": ["Sheet 1"],
    "extractions": [
        {
            "function": "multirow_patterns",
            "label": "items",
            "instructions": {
                "row_range": [1, 1000],
                "stop_if_empty": "A",
                "columns": {
                    "Name": "A",
                    "Value": "B",
                    "Description": "C"
                }
            }
        }
    ]
}
# Returns: {"items": [{"Name": "...", "Value": ...}, {"Name": "...", "Value": ...}]}
```

**Example with stop_if_empty and gap tolerance:**
```python
{
    "sheets": ["Sheet 1"],
    "extractions": [
        {
            "function": "multirow_patterns",
            "label": "data",
            "instructions": {
                "row_range": [1, 100],
                "stop_if_empty": "A",
                "stop_consecutive": 3,
                "columns": {
                    "ID": "A",
                    "Value": "B"
                }
            }
        }
    ]
}
```

**Example with row-based empty detection:**
```python
{
    "sheets": ["Sheet 1"],
    "extractions": [
        {
            "function": "multirow_patterns",
            "label": "records",
            "instructions": {
                "row_range": [1, 500],
                "stop_if_empty": {
                    "mode": "row",
                    "consecutive": 2
                },
                "columns": {
                    "Field1": "A",
                    "Field2": "B",
                    "Field3": "C"
                }
            }
        }
    ]
}
```

**Example with multiple column monitoring:**
```python
{
    "sheets": ["Sheet 1"],
    "extractions": [
        {
            "function": "multirow_patterns",
            "label": "transactions",
            "instructions": {
                "row_range": [1, 1000],
                "unique_id": "A",
                "stop_if_empty": ["A", "B"],
                "columns": {
                    "ID": "A",
                    "Date": "B",
                    "Amount": "C"
                }
            }
        }
    ]
}
```

### Dataframe Extraction

Extracts tabular data with headers, returning JSON that can easily be converted to a Pandas DataFrame.

**Instructions:**
* `row_range`: A list of two integers defining the row range to extract.
* `column_range`: A list of column letters to extract.
* `header_row`: A list of row numbers to use as the header.
* `separator`: Optional separator to use when combining header cells (default `" "`).

**Example:**
```python
{
    "sheets": ["School_*"],
    "extractions": [
        {
            "function": "dataframe",
            "label": "DataFrame",
            "instructions": {
                "row_range": [5, 15],
                "column_range": ["B", "F"],
                "header_row": [2, 3, 4],
                "separator": " ",
            }
        }
    ]
}
```

## Contributing

Contributions are welcome! Please feel free to submit issues and pull requests.

```bash
# Development setup
pip install maturin pytest openpyxl ruff
maturin develop --release
make lint   # check formatting
make test   # run tests
```

## License

Sheet Excavator is released under the MIT License. See the [LICENSE](LICENSE) file for more details.
