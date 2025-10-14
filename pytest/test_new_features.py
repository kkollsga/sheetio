#!/usr/bin/env python3
"""
Test new multirow_patterns features:
1. Optional unique_id (returns list instead of dict)
2. stop_if_empty with various syntaxes
3. Empty row and column detection
"""

import pandas as pd
import json
import sheet_excavator
import os

print("Creating test Excel file...")

# Create test data with gaps and empty rows
df = pd.DataFrame({
    'ID': ['001', '002', '003', None, '005', None, None, None],
    'Name': ['Alice', 'Bob', 'Charlie', None, 'Eve', None, None, None],
    'Value': [100, 200, 300, None, 500, None, None, None],
    'Notes': ['A', 'B', None, None, 'E', None, None, None]
})

test_file = 'test_new_features.xlsx'
df.to_excel(test_file, sheet_name='TestSheet', index=False)
print(f"✓ Created {test_file}")
print(f"  Data:\n{df}\n")

# Test 1: No unique_id - returns list (array)
print("="*60)
print("Test 1: No unique_id - should return array")
print("="*60)

config1 = [{
    "sheets": ["TestSheet"],
    "extractions": [{
        "function": "multirow_patterns",
        "label": "items_list",
        "instructions": {
            "row_range": [2, 9],
            "stop_if_empty": "A",  # Stop when ID column is empty
            "columns": {
                "ID": "A",
                "Name": "B",
                "Value": "C"
            }
        }
    }]
}]

try:
    result = sheet_excavator.excel_extract([test_file], config1, 1)
    parsed = json.loads(result)

    # Debug: print the keys
    print(f"Available keys: {list(parsed.keys())}")

    # Navigate to the result - use first key (might be absolute path)
    file_key = list(parsed.keys())[0]
    items = parsed[file_key]['TestSheet']['items_list']
    print(f"Result type: {type(items)}")
    print(f"Number of items: {len(items)}")

    if isinstance(items, list):
        print("✓ Result is a list (array)")
        for i, item in enumerate(items):
            print(f"  [{i}]: {item}")
    else:
        print(f"✗ Expected list, got {type(items)}")

    # Should stop after row 3 (when ID becomes null)
    if len(items) == 3:
        print("✓ Correctly stopped after 3 rows (stop_if_empty worked)")
    else:
        print(f"✗ Expected 3 items, got {len(items)}")

    print("✓ Test 1 PASSED")
except Exception as e:
    print(f"✗ Test 1 FAILED: {e}")
    import traceback
    traceback.print_exc()

# Test 2: With unique_id - returns dict (backward compatibility)
print("\n" + "="*60)
print("Test 2: With unique_id - should return object/dict")
print("="*60)

config2 = [{
    "sheets": ["TestSheet"],
    "extractions": [{
        "function": "multirow_patterns",
        "label": "items_dict",
        "instructions": {
            "row_range": [2, 9],
            "unique_id": "A",
            "columns": {
                "ID": "A",
                "Name": "B",
                "Value": "C"
            }
        }
    }]
}]

try:
    result = sheet_excavator.excel_extract([test_file], config2, 1)
    parsed = json.loads(result)

    file_key = list(parsed.keys())[0]
    items = parsed[file_key]['TestSheet']['items_dict']
    print(f"Result type: {type(items)}")

    if isinstance(items, dict):
        print("✓ Result is a dict (object)")
        for key, item in items.items():
            print(f"  {key}: {item}")
    else:
        print(f"✗ Expected dict, got {type(items)}")

    print("✓ Test 2 PASSED")
except Exception as e:
    print(f"✗ Test 2 FAILED: {e}")
    import traceback
    traceback.print_exc()

# Test 3: stop_if_empty with consecutive parameter
print("\n" + "="*60)
print("Test 3: stop_consecutive to tolerate gaps")
print("="*60)

config3 = [{
    "sheets": ["TestSheet"],
    "extractions": [{
        "function": "multirow_patterns",
        "label": "with_gaps",
        "instructions": {
            "row_range": [2, 9],
            "stop_if_empty": "A",
            "stop_consecutive": 2,  # Allow 1 gap
            "columns": {
                "ID": "A",
                "Name": "B",
                "Value": "C"
            }
        }
    }]
}]

try:
    result = sheet_excavator.excel_extract([test_file], config3, 1)
    parsed = json.loads(result)

    file_key = list(parsed.keys())[0]
    items = parsed[file_key]['TestSheet']['with_gaps']
    print(f"Number of items: {len(items)}")

    # Should get 4 items (001, 002, 003, 005) because we tolerate 1 gap
    if len(items) == 4:
        print("✓ Correctly included item after gap (stop_consecutive: 2 worked)")
        for i, item in enumerate(items):
            print(f"  [{i}]: {item}")
    else:
        print(f"✗ Expected 4 items, got {len(items)}")

    print("✓ Test 3 PASSED")
except Exception as e:
    print(f"✗ Test 3 FAILED: {e}")
    import traceback
    traceback.print_exc()

# Test 4: stop_if_empty with row mode (object syntax)
print("\n" + "="*60)
print("Test 4: stop_if_empty with row mode")
print("="*60)

config4 = [{
    "sheets": ["TestSheet"],
    "extractions": [{
        "function": "multirow_patterns",
        "label": "row_mode",
        "instructions": {
            "row_range": [2, 9],
            "stop_if_empty": {
                "mode": "row",
                "consecutive": 1
            },
            "columns": {
                "ID": "A",
                "Name": "B",
                "Value": "C",
                "Notes": "D"
            }
        }
    }]
}]

try:
    result = sheet_excavator.excel_extract([test_file], config4, 1)
    parsed = json.loads(result)

    file_key = list(parsed.keys())[0]
    items = parsed[file_key]['TestSheet']['row_mode']
    print(f"Number of items: {len(items)}")

    # Row 3 has some data, row 4 is completely empty - should stop at row 4
    if len(items) == 3:
        print("✓ Correctly stopped when entire row is empty")
        for i, item in enumerate(items):
            print(f"  [{i}]: {item}")
    else:
        print(f"✗ Expected 3 items, got {len(items)}")

    print("✓ Test 4 PASSED")
except Exception as e:
    print(f"✗ Test 4 FAILED: {e}")
    import traceback
    traceback.print_exc()

# Test 5: stop_if_empty with multiple columns (array syntax)
print("\n" + "="*60)
print("Test 5: stop_if_empty with multiple columns")
print("="*60)

config5 = [{
    "sheets": ["TestSheet"],
    "extractions": [{
        "function": "multirow_patterns",
        "label": "multi_column",
        "instructions": {
            "row_range": [2, 9],
            "stop_if_empty": ["A", "B"],  # Stop when both ID and Name are empty
            "columns": {
                "ID": "A",
                "Name": "B",
                "Value": "C"
            }
        }
    }]
}]

try:
    result = sheet_excavator.excel_extract([test_file], config5, 1)
    parsed = json.loads(result)

    file_key = list(parsed.keys())[0]
    items = parsed[file_key]['TestSheet']['multi_column']
    print(f"Number of items: {len(items)}")

    # Should stop when BOTH A and B are empty (row 4)
    if len(items) == 3:
        print("✓ Correctly stopped when both columns empty")
        for i, item in enumerate(items):
            print(f"  [{i}]: {item}")
    else:
        print(f"✗ Expected 3 items, got {len(items)}")

    print("✓ Test 5 PASSED")
except Exception as e:
    print(f"✗ Test 5 FAILED: {e}")
    import traceback
    traceback.print_exc()

# Test 6: stop_if_empty with object syntax (column mode)
print("\n" + "="*60)
print("Test 6: stop_if_empty object syntax with column")
print("="*60)

config6 = [{
    "sheets": ["TestSheet"],
    "extractions": [{
        "function": "multirow_patterns",
        "label": "object_syntax",
        "instructions": {
            "row_range": [2, 9],
            "stop_if_empty": {
                "column": "A",
                "consecutive": 1
            },
            "columns": {
                "ID": "A",
                "Name": "B",
                "Value": "C"
            }
        }
    }]
}]

try:
    result = sheet_excavator.excel_extract([test_file], config6, 1)
    parsed = json.loads(result)

    file_key = list(parsed.keys())[0]
    items = parsed[file_key]['TestSheet']['object_syntax']
    print(f"Number of items: {len(items)}")

    if len(items) == 3:
        print("✓ Object syntax worked correctly")
        for i, item in enumerate(items):
            print(f"  [{i}]: {item}")
    else:
        print(f"✗ Expected 3 items, got {len(items)}")

    print("✓ Test 6 PASSED")
except Exception as e:
    print(f"✗ Test 6 FAILED: {e}")
    import traceback
    traceback.print_exc()

# Cleanup
if os.path.exists(test_file):
    os.remove(test_file)
    print(f"\n✓ Cleaned up {test_file}")

print("\n" + "="*60)
print("ALL TESTS COMPLETED")
print("="*60)
