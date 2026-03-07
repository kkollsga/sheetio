# Sheet Excavator

**Fast Rust-powered Excel form data extraction to JSON.**

[![PyPI version](https://badge.fury.io/py/sheet-excavator.svg)](https://badge.fury.io/py/sheet-excavator)
[![Python 3.10+](https://img.shields.io/badge/python-3.10+-blue.svg)](https://www.python.org/downloads/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

Sheet Excavator extracts structured data from Excel forms (.xlsx, .xlsm) into JSON. Built in Rust with Python bindings via PyO3 for fast parallel processing of hundreds of files.

Designed for standardized Excel forms that don't fit the typical CSV format -- government reports, engineering forms, financial templates, and survey spreadsheets.

## Installation

```
pip install sheet-excavator
```

## Two Ways to Use It

### Config Builder (recommended)

Build extraction configs with Python methods, get results as dicts:

```python
from sheet_excavator import ExtractionConfig

config = ExtractionConfig()
config.add_sheets(["Sheet1"]) \
    .single_cells("header", title="B2", date="D4") \
    .multirow("items",
        row_range=(10, 100),
        unique_id="A",
        columns={"ID": "A", "Name": "B", "Value": "C"})

result = config.extract(["form_001.xlsx", "form_002.xlsx"], workers=5)
```

See the [Config Builder Guide](config-builder.md) for the full walkthrough.

### Raw API

Pass config as dicts, get JSON string back:

```python
import sheet_excavator
import json

config = [
    {
        "sheets": ["Sheet1"],
        "extractions": [
            {
                "function": "single_cells",
                "label": "header",
                "instructions": {"title": "B2", "date": "D4"}
            }
        ]
    }
]

result = json.loads(sheet_excavator.excel_extract(files, config, 5))
```

See the [Extraction Types Reference](extraction-types.md) for the full config schema.

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
