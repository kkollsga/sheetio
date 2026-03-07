# Config Builder Guide

The `ExtractionConfig` class provides a fluent Python API for building extraction configurations. Instead of writing raw nested dicts, you use methods with named parameters, IDE autocomplete, and built-in validation.

## Getting Started

```python
from sheet_excavator import ExtractionConfig

config = ExtractionConfig()
```

### Step 1: Add a Sheet Group

A **sheet group** defines which sheets to process and what extractions to run on them.

```python
group = config.add_sheets(["Sheet1"])
```

You can use wildcards and skip specific sheets:

```python
group = config.add_sheets(["School_*"], skip=["School_Summary"])
```

### Step 2: Add Extractions

Chain extraction methods on the sheet group. There are three types:

```python
group.single_cells("header", title="B2", date="D4")
group.multirow("items", row_range=(10, 100), unique_id="A",
               columns={"ID": "A", "Name": "B"})
group.dataframe("table", header_row=[2, 3], row_range=(5, 50),
                column_range=["B", "F"])
```

### Step 3: Extract

```python
result = config.extract(["form_001.xlsx", "form_002.xlsx"], workers=5)
```

This returns a parsed Python dict -- no `json.loads()` needed.

---

## Extraction Methods

### `.single_cells()` -- Extract Individual Cells

Pull values from specific cell addresses.

```python
# Simple names as keyword arguments
group.single_cells("header", title="B2", date="D4", author="B6")

# Names with spaces or special characters use cells= dict
group.single_cells("header", cells={"Report Title": "B2", "Date (ISO)": "D4"})

# Mix both -- kwargs override cells on key collision
group.single_cells("header", cells={"title": "A1"}, title="B2")
# Result: title -> B2 (kwargs wins)
```

**Array references** -- extract multiple cells into a list:

```python
group.single_cells("meta", cells={"history": ["H7", "H8", "H9"]})
```

**Parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| `label` | `str \| None` | Key to nest results under. If omitted, values go directly under sheet name. |
| `cells` | `dict` | Maps names to cell addresses (`"B2"`) or cell address arrays (`["H7","H8"]`). |
| `break_if_null` | `str` | Cell address; skip this extraction if cell is empty. |
| `**kwargs` | `str` | Additional name=address mappings (simple names only). |

---

### `.multirow()` -- Extract Row Patterns

Extract data from multiple rows in a structured pattern. This is the most powerful extraction type.

#### Basic usage with unique ID (returns dict):

```python
group.multirow("deposits",
    row_range=(1, 100),
    unique_id="B",
    columns={
        "Title": "B",
        "Description": "C",
        "Estimate": "D",
        "Chance": "E"
    })
```

#### Without unique ID (returns list):

```python
group.multirow("items",
    row_range=(1, 1000),
    stop_if_empty="A",
    columns={"Name": "A", "Value": "B"})
# Result: [{"Name": "...", "Value": ...}, ...]
```

#### Composite keys:

```python
group.multirow("projects",
    row_range=(1, 50),
    unique_id=["B", "C"],           # combine two columns
    unique_id_separator="-",         # "ProjectA-2024"
    columns={"Project": "B", "Year": "C", "Budget": "D"})
```

#### Stop conditions:

```python
# Stop when column A is empty
group.multirow("data", row_range=(1, 1000), stop_if_empty="A",
               columns={"ID": "A", "Value": "B"})

# Stop when ALL of these columns are empty
group.multirow("data", row_range=(1, 1000), stop_if_empty=["A", "B"],
               columns={"ID": "A", "Date": "B", "Amount": "C"})

# Stop when entire row is empty
group.multirow("data", row_range=(1, 500),
               stop_if_empty={"mode": "row", "consecutive": 2},
               columns={"F1": "A", "F2": "B"})

# Gap tolerance -- tolerate up to 2 consecutive empty rows
group.multirow("data", row_range=(1, 1000),
               stop_if_empty="A", stop_consecutive=3,
               columns={"ID": "A", "Value": "B"})
```

#### Multi-column merge:

```python
group.multirow("data", row_range=(1, 50), unique_id="A",
               columns={"ID": "A", "Values": ["B", "C", "D"]})
# Result: {"ID001": {"ID": "ID001", "Values": ["valB", "valC", "valD"]}}
```

**Parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| `label` | `str \| None` | Key to nest results under. |
| `row_range` | `tuple \| list` | `(start_row, end_row)`, 1-based. |
| `columns` | `dict` | Maps output names to column letters. Use list for merged columns. |
| `unique_id` | `str \| list \| None` | Column(s) for dict keys. Omit for list output. |
| `unique_id_separator` | `str` | Separator for composite keys (default `"_"`). |
| `stop_if_empty` | `str \| list \| dict` | When to stop processing rows. |
| `stop_consecutive` | `int` | How many empty rows before stopping (default 1). |
| `break_if_null` | `str` | Cell address; skip extraction if cell is empty. |

---

### `.dataframe()` -- Extract Tabular Data

Extract tabular data with headers, returning column-oriented data suitable for Pandas DataFrames.

```python
group.dataframe("grades",
    header_row=[2, 3, 4],         # multi-row header, joined with separator
    row_range=(5, 50),
    column_range=["B", "F"],
    separator=" ")                 # join multi-row headers with space
```

**Parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| `label` | `str \| None` | Key to nest results under. |
| `row_range` | `tuple \| list` | `(start_row, end_row)`, 1-based. |
| `column_range` | `list` | `[start_col, end_col]` as column letters (e.g. `["B", "F"]`). |
| `header_row` | `int \| list` | Row number(s) for column headers. |
| `separator` | `str` | Separator for joining multi-row headers (default `" "`). |
| `break_if_null` | `str` | Cell address; skip extraction if cell is empty. |

---

## Sheet Group Options

### `break_if_null` at group level

Skip an entire sheet if a specific cell is empty. Useful for forms where some sheets are unused.

```python
config.add_sheets(["Sheet1"], break_if_null="C3") \
    .single_cells("header", title="B2")
```

### Wildcards and skip

Process sheets matching a pattern, excluding specific ones:

```python
config.add_sheets(["Report_*"], skip=["Report_Summary", "Report_Template"]) \
    .multirow("data", row_range=(5, 500), columns={"ID": "A", "Value": "B"})
```

---

## Iterative Workflow

The builder is designed for iterative development. Test your config against a sample file, adjust, and repeat.

```python
config = ExtractionConfig()
group = config.add_sheets(["Sheet1"])

# Start with header cells
group.single_cells("header", title="B2", date="D4")
result = config.extract(["sample.xlsx"])
print(result)  # check output

# Add row extraction based on what you see
group.multirow("items",
    row_range=(10, 100),
    unique_id="A",
    stop_if_empty="A",
    columns={"ID": "A", "Name": "B", "Value": "C"})
result = config.extract(["sample.xlsx"])
print(result)  # check again

# Overwrite an extraction (same label replaces previous)
group.single_cells("header", title="B2", date="D4", version="F1")
```

### Modifying Configs

```python
# Remove an extraction by label
group.remove("header")

# Overwrite by calling with the same label
group.single_cells("header", title="B2", date="D4", version="F1")
```

---

## Multiple Sheet Groups

Different sheets can have different extraction rules:

```python
config = ExtractionConfig()

# Group 1: summary sheet
config.add_sheets(["Summary"]) \
    .single_cells("totals", total="B20", average="B21")

# Group 2: data sheets (wildcard)
config.add_sheets(["Data_*"], skip=["Data_Template"]) \
    .multirow("records", row_range=(2, 5000), unique_id="A",
              stop_if_empty="A",
              columns={"ID": "A", "Date": "B", "Amount": "C"})

# Group 3: grade sheets with dataframe
config.add_sheets(["School_*"]) \
    .dataframe("grades", header_row=[2, 3], row_range=(4, 50),
               column_range=["B", "G"])

result = config.extract(glob.glob("forms/*.xlsx"), workers=8)
```

---

## Saving and Loading Configs

### Save to JSON

```python
config.to_json("extraction_config.json")
```

This writes the raw config list (same format as `config.build()`):

```json
[
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
```

### Load from JSON

```python
config = ExtractionConfig.from_json("extraction_config.json")
result = config.extract(files)
```

You can also load configs written by hand -- the JSON format is the same as the raw API.

---

## Inspecting Configs

### `.summary()`

Print a tree view of what's configured:

```python
config.summary()
```

```
ExtractionConfig (2 sheet groups)

  Group 1: sheets=['Sheet1']
  ├─ [single_cells] "header" title->B2, date->D4
  └─ [multirow_patterns] "items" rows 10-100, unique_id=A, columns: ID->A, Name->B

  Group 2: sheets=['School_*'], skip=['School_X']
  └─ [dataframe] "grades" rows 5-50, cols B-F, header=[2, 3, 4]
```

### `.build()`

Get the raw config list for manual inspection or passing to `excel_extract()`:

```python
raw = config.build()
print(json.dumps(raw, indent=2))
```

---

## Complete Example

Processing government inspection forms with a header section, a checklist, and a data table:

```python
from sheet_excavator import ExtractionConfig
import glob

config = ExtractionConfig()

group = config.add_sheets(["Inspection"], break_if_null="B2")

# Header metadata
group.single_cells("metadata",
    cells={"Inspector Name": "B2", "Inspection Date": "D2"},
    facility="B4", region="D4")

# Checklist items (rows 8-30, stop at first empty)
group.multirow("checklist",
    row_range=(8, 30),
    stop_if_empty="A",
    columns={
        "Item": "A",
        "Status": "B",
        "Notes": "C"
    })

# Measurement table (multi-row header)
group.dataframe("measurements",
    header_row=[35, 36],
    row_range=(37, 100),
    column_range=["A", "H"])

# Save for reuse
config.to_json("inspection_config.json")

# Process all forms
files = glob.glob("/reports/2024/*.xlsx")
result = config.extract(files, workers=8)

# result is a dict keyed by filename
for filename, file_data in result.items():
    inspection = file_data["Inspection"]
    print(f"{inspection['metadata']['Inspector Name']}: "
          f"{len(inspection['checklist'])} items checked")
```
