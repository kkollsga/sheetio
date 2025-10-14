# Sheet Excavator
A fast Rust-powered tool for extracting data from Excel forms into JSON.

## Overview
Sheet Excavator is a Rust-based tool designed to facilitate the efficient extraction of data from standardized Excel forms. Traditional reporting often relies on Excel forms that do not conform to the typical CSV data storage format, making data extraction challenging. Existing Python-based workflows may also suffer from performance issues when handling large databases of forms stored in .xlsx files.

Leveraging Rust's high performance and robust multithreading capabilities, Sheet Excavator provides a powerful API tailored for extracting data from unstructured Excel layouts. It supports various functionalities including single cell extraction, row-based patterns, and multi-column arrays, returning results in an easy-to-use JSON format.

## Key features
- High Performance: Utilizes Rust’s efficiency and multithreading to handle large datasets.
- Flexible Data Extraction: Supports various extraction methods for complex Excel form layouts.
- JSON Output: Seamlessly integrates with modern data pipelines by outputting data in JSON format.

### Install with pip
*To install Sheet Excavator, run the following command in your terminal:*
```
pip install sheet-excavator
```
*To upgrade an already installed version of Sheet Excavator, use:*
```
pip install --upgrade sheet-excavator
```

## Sheet Excavator Usage Guide

### Overview
`sheet_excavator` is a Python library designed to assist in extracting data from Excel sheets. This guide provides an overview of how to use the library and its various features.

### Basic Usage
To get started with `sheet_excavator`, you can follow these steps:

```python
import sheet_excavator
import glob
import json

files = glob.glob(r"D:\temp\*") # List of files to process
extraction_details = [...]  # define list extraction details to apply to each file (see below)
workers = 10 # Number of parallell workers, should reflect number of cpu cores on the system.
results = sheet_excavator.excel_extract(files, extraction_details, workers) # excel_extractor returns a json formated string
dict_results = json.loads(results) # convert the json string to a python dict
print(json.dumps(dict_results, indent=3))
```

### Extraction Details
The `extraction_details` parameter is a list of dictionaries that define the extraction rules for each Excel sheet. Each dictionary contains the following keys:
* `sheets`: A list of sheet names to extract data from. Accepts patterns with *. Example School_* will loop through sheets like School_A, School_B, etc.
* `skip_sheets`: An optional list of sheet names to skip. Can be useful when using patterns in the list of sheets.
* `extractions`: A list of extraction rules (see below), that will be applied to the sheets listed.


### Extraction Rules
The `extractions` key in the `extraction_details` dictionary contains a list of extraction rules.
* `function`: Type of extraction function (see details below). There are three types `single_cells`, `multirow_patterns`, and `dataframe`.
* `label`: Optional key string to store results under. If not specified the extracted key value pairs will be stored directly under the sheet name.
* `break_if_null`: An optional check to skip sheet if specified cell is null.
* `instructions`: Instructions for the extraction function. See details for each function type below. 

#### Single Cells Extraction
The `single_cells` extraction rule extracts individual cells from the Excel sheet.

**Instructions:**
* `instructions`: A dictionary where the keys are the reference name (e.g. "Title", "Description", etc) and the values are the cell references (e.g., "a1", "b2", etc.).

**Example:**
```python
{
    "sheets": ["Sheet1"], # List of sheets to loop through
    "extractions": [ # List of extractions to attempt
        {
            "function": "single_cells", # Function type
            "label": "single", # Optional label that defines a parent key
            "break_if_null": "c3", # Before attempting to extract values from sheet, checks if this cell is null
            "instructions": { # Instructions for selected function
                "Value 1": "a1", # Title, cell address pairs.
                "Value 2": "b2",
                "Value 3": "c3",
                "Date": "d4",
                "Datetime": "e5"
            }
        }
    ]
}
```

#### Multirow Patterns Extraction
The `multirow_patterns` extraction rule extracts data from multiple rows in the Excel sheet based on a pattern.

**Instructions:**
* `row_range`: A list of two integers defining the row range to extract. The function will iterate through the rows within this range.
* `unique_id` (optional): The column(s) to use as a unique identifier. Can be either:
  - A single column as a string: `"B"`
  - Multiple columns as an array: `["B", "C"]` for composite keys
  - When using composite keys, if ANY column contains null/empty values, the row is skipped
  - **If omitted**: Results are returned as an array/list instead of a dictionary
* `unique_id_separator` (optional): The separator to use when joining multiple columns for composite keys. Defaults to `"_"`.
* `columns`: A dictionary where the keys are the column names and the values are the column letters (e.g., "B", "C", etc.).
* `stop_if_empty` (optional): Controls when to stop processing rows. Can be:
  - A column string: `"A"` - Stop when this column is empty
  - An array of columns: `["A", "B"]` - Stop when ALL specified columns are empty
  - The string `"row"` - Stop when the entire data row is empty
  - An object with detailed configuration:
    ```python
    {
        "column": "A",        # or ["A", "B"] for multiple columns
        "consecutive": 2      # Number of consecutive empty rows/columns to trigger stop
    }
    ```
    or
    ```python
    {
        "mode": "row",        # "row" or "column"
        "consecutive": 1      # Default is 1
    }
    ```
* `stop_consecutive` (optional): Used with simple `stop_if_empty` syntax to specify how many consecutive empty rows trigger a stop. Defaults to `1`.

**Example with single unique_id:**
```python
{
    "sheets": ["Sheet 1", "Sheet 2"], # List of sheets to loop through
    "extractions": [ # List of extractions to attempt
        {
            "function": "multirow_patterns", # Function type
            "label": "deposits", # Optional label that defines a parent key
            "instructions": { # Instructions for selected function
                "row_range": [1, 10], # Range of rows to iterate through
                "unique_id": "B", # Single column as unique identifier
                "columns": { # Columns to extract data from, keys are used as value title.
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
                "unique_id": ["B", "C"],  # Composite key from columns B and C
                "unique_id_separator": "-",  # Optional: use "-" instead of default "_"
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
                "stop_if_empty": "A",  # Stop when column A is empty
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
                "stop_consecutive": 3,  # Tolerate up to 2 empty rows
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
                "stop_if_empty": ["A", "B"],  # Stop when both ID and Date are empty
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

#### Dataframe Extraction
The dataframe extraction rule extracts data into a Pandas DataFrame.

**Instructions:**

* `row_range`: A list of two integers defining the row range to extract.
* `column_range`: A list of column letters to extract.
* `header_row`: A list of row numbers to use as the header.
* `separator`: Optional separator to use when combining header cells (default " ").

**Example:**
```python
{
    "sheets": ["School_*"],  # List of sheets to loop through
    "extractions": [ # List of extractions to attempt
        {
            "function": "dataframe", # Function type
            "label": "DataFrame", # Optional label that defines a parent key
            "instructions": { # Instructions for selected function
                "row_range": [5, 15], # Range of rows where data is extracted from
                "column_range": ["B", "F"], # Range of columns to extract headers and data
                "header_row": [2, 3, 4], # List of rows that contain header data (will be concatenated to a string)
                "separator": " ", # Optional separator specifier (defaults to " ")
            }
        }
    ]
}
```

By following this guide, you should be able to use the `sheet_excavator` library to extract data from your Excel sheets. The data is returned as json_formatted string.

## License
Sheet Excavator is released under the MIT License. See the LICENSE file for more details.