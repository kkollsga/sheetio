# Extraction Types Reference

This is the reference for the raw JSON config format used by `excel_extract()`. If you're using the [Config Builder](config-builder.md), these are the underlying structures that `.build()` produces.

## Configuration Structure

The `extraction_details` parameter is a list of dictionaries. Each dictionary defines a **sheet group**:

```python
[
    {
        "sheets": ["Sheet1", "Report_*"],    # required
        "skip_sheets": ["Report_Draft"],     # optional
        "break_if_null": "C3",               # optional
        "extractions": [...]                  # required
    }
]
```

| Key | Type | Description |
|-----|------|-------------|
| `sheets` | `list[str]` | Sheet names or wildcard patterns. `"School_*"` matches School_A, School_B, etc. |
| `skip_sheets` | `list[str]` | Sheet names to exclude from wildcard matches. |
| `break_if_null` | `str` | Cell address; skip sheet if this cell is empty. |
| `extractions` | `list[dict]` | Extraction rules to apply to matched sheets. |

Each extraction rule:

```python
{
    "function": "single_cells",      # required: single_cells, multirow_patterns, or dataframe
    "label": "header",               # optional: key to nest results under
    "break_if_null": "C3",           # optional: skip if cell is empty
    "instructions": {...}            # required: function-specific config
}
```

---

## Single Cells

Extracts individual cells by address.

**Instructions:** A dictionary mapping output names to cell references.

```python
{
    "function": "single_cells",
    "label": "header",
    "instructions": {
        "title": "A1",
        "date": "D4",
        "values": ["H7", "H8", "H9"]   # array of cells -> array of values
    }
}
```

Cell addresses are case-insensitive (`"a1"` and `"A1"` both work).

---

## Multirow Patterns

Extracts data from rows following a repeating pattern.

### Instructions

| Key | Type | Required | Description |
|-----|------|----------|-------------|
| `row_range` | `[start, end]` | Yes | Row range (1-based, inclusive). |
| `columns` | `dict` | Yes | Maps output names to column letters. |
| `unique_id` | `str \| list` | No | Column(s) for dict keys. Omit for array output. |
| `unique_id_separator` | `str` | No | Separator for composite keys (default `"_"`). |
| `stop_if_empty` | `str \| list \| dict` | No | Stop condition configuration. |
| `stop_consecutive` | `int` | No | Consecutive empty rows to trigger stop (default 1). |

### Examples

**With single unique_id** (returns dict keyed by column values):

```python
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
            "Chance": "E"
        }
    }
}
```

**With composite unique_id:**

```python
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
```

**Without unique_id** (returns array):

```python
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
# Returns: {"items": [{"Name": "...", "Value": ...}, ...]}
```

### Stop Conditions

**Single column:**
```python
"stop_if_empty": "A"
```

**Multiple columns** (stop when ALL are empty):
```python
"stop_if_empty": ["A", "B"]
```

**Entire row:**
```python
"stop_if_empty": {"mode": "row", "consecutive": 2}
```

**Column with object syntax:**
```python
"stop_if_empty": {"column": "A", "consecutive": 3}
```

**Gap tolerance** (with simple syntax):
```python
"stop_if_empty": "A",
"stop_consecutive": 3    // tolerate up to 2 consecutive empty rows
```

### Multi-column Merge

Map a single output field to multiple columns:

```python
"columns": {
    "ID": "A",
    "Combined": ["B", "C", "D"]    // returns array of values
}
```

---

## Dataframe

Extracts tabular data with headers, returning column-oriented JSON suitable for Pandas DataFrames.

### Instructions

| Key | Type | Required | Description |
|-----|------|----------|-------------|
| `row_range` | `[start, end]` | Yes | Data row range (1-based, inclusive). |
| `column_range` | `[start, end]` | Yes | Column range as letters (e.g. `["B", "F"]`). |
| `header_row` | `int \| list` | Yes | Row number(s) for column headers. |
| `separator` | `str` | No | Separator for joining multi-row headers (default `" "`). |

### Example

```python
{
    "function": "dataframe",
    "label": "grades",
    "instructions": {
        "row_range": [5, 15],
        "column_range": ["B", "F"],
        "header_row": [2, 3, 4],
        "separator": " "
    }
}
```

With multi-row headers `[2, 3, 4]`, header cells are concatenated with the separator. For example, if row 2 has "Math", row 3 has "Score", and row 4 has "(100)", the column header becomes `"Math Score (100)"`.

---

## Output Structure

The result is a JSON string (use `json.loads()` to parse) with this structure:

```json
{
    "filename_without_extension": {
        "filepath": "/full/path/to/file.xlsx",
        "Sheet1": {
            "label_name": {
                // extraction results
            }
        }
    }
}
```

When processing multiple files, each file gets its own top-level key. Duplicate filenames get `_1`, `_2` suffixes automatically.
