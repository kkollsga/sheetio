# New Features - Multirow Patterns Enhancement

## Summary

Enhanced the `multirow_patterns` extraction function with optional unique_id and configurable stop mechanisms.

## New Features

### 1. Optional `unique_id`

**Previous behavior:** `unique_id` was required, results returned as dictionary with unique_id as keys.

**New behavior:**
- When `unique_id` is provided → returns dictionary (backward compatible)
- When `unique_id` is omitted → returns array/list

**Example:**
```python
# Without unique_id - returns array
{
    "function": "multirow_patterns",
    "instructions": {
        "row_range": [1, 100],
        "stop_if_empty": "A",
        "columns": {"Name": "A", "Value": "B"}
    }
}
# Result: [{"Name": "Alice", "Value": 100}, {"Name": "Bob", "Value": 200}]
```

### 2. `stop_if_empty` Parameter

Configurable mechanism to detect when to stop processing rows.

**Syntax options:**

#### Simple column string
```python
"stop_if_empty": "A"  # Stop when column A is empty
```

#### Multiple columns (array)
```python
"stop_if_empty": ["A", "B"]  # Stop when ALL columns are empty
```

#### Row mode
```python
"stop_if_empty": "row"  # Stop when entire data row is empty
```

#### Object syntax (detailed control)
```python
"stop_if_empty": {
    "column": "A",       # or ["A", "B"]
    "consecutive": 2     # Stop after 2 consecutive empty rows
}
```

```python
"stop_if_empty": {
    "mode": "row",       # "row" or "column"
    "consecutive": 1
}
```

### 3. `stop_consecutive` Parameter

Used with simple `stop_if_empty` syntax to control gap tolerance:

```python
"stop_if_empty": "A",
"stop_consecutive": 3  # Tolerate up to 2 empty rows (gaps in data)
```

## Implementation Details

### Files Modified

1. **src/utils/multirow_patterns.rs**
   - Changed return type from `Result<IndexMap<String, Value>, Error>` to `Result<Value, Error>`
   - Added `parse_stop_config()` function to parse stop configuration
   - Added `is_row_empty()` function to check if entire row is empty
   - Added `are_columns_empty()` function to check specific columns
   - Made `unique_id` optional
   - Implemented branching logic for dict vs array results
   - Added consecutive empty counter with configurable threshold

2. **src/read_excel.rs**
   - Updated to handle `Value` return type from `multirow_patterns`
   - Added logic to handle both Object and Array results
   - Maintains backward compatibility with other extraction functions

### Test Coverage

New test file: `pytest/test_new_features.py`

Tests cover:
- ✅ Array results without unique_id
- ✅ Dict results with unique_id (backward compatibility)
- ✅ Gap tolerance with stop_consecutive
- ✅ Row-based empty detection
- ✅ Multiple column monitoring
- ✅ Object syntax for stop configuration

All tests pass successfully.

## Backward Compatibility

✅ **Fully backward compatible**
- Existing configurations with `unique_id` work unchanged
- Returns same dictionary structure as before
- No breaking changes to API

## Use Cases

### Use Case 1: Unknown Data Length
```python
# Large row_range with auto-stop when data ends
"row_range": [1, 10000],
"stop_if_empty": "A"
```

### Use Case 2: Data with Gaps
```python
# Tolerate occasional empty rows
"stop_if_empty": "A",
"stop_consecutive": 5  # Continue through up to 4 empty rows
```

### Use Case 3: Simple Lists
```python
# Extract simple list without unique identifiers
# No unique_id needed - returns array
"stop_if_empty": "A",
"columns": {"Item": "A", "Quantity": "B"}
```

### Use Case 4: Sparse Data
```python
# Only stop when multiple key columns are all empty
"stop_if_empty": ["A", "B", "C"]
```

## Performance

No significant performance impact. Stop mechanism allows early exit from large row ranges, potentially improving performance for sparse data.

## Migration Guide

No migration needed! Existing code continues to work without changes.

To use new features:
1. Omit `unique_id` for list results
2. Add `stop_if_empty` for auto-detection of data end
3. Use `stop_consecutive` to handle gaps in data
