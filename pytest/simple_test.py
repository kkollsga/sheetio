#!/usr/bin/env python3
"""
Simple standalone test for composite unique_id feature
"""

import json
import os

import pandas as pd
import sheet_excavator

print("Creating test Excel file...")

# Create a simple test DataFrame
df = pd.DataFrame(
    {
        "ID": ["001", "002", "003", "001"],
        "Name": ["Alpha", "Beta", "Gamma", "Alpha"],
        "Year": [2024, 2024, 2025, 2025],
        "Value": [100, 200, 300, 150],
    }
)

# Save to Excel
test_file = "simple_test.xlsx"
df.to_excel(test_file, sheet_name="TestSheet", index=False)
print(f"✓ Created {test_file}")
print(f"  Data:\n{df}\n")

# Test 1: Single column unique_id (backwards compatible)
print("=" * 60)
print("Test 1: Single column unique_id (backwards compatible)")
print("=" * 60)

config1 = [
    {
        "sheets": ["TestSheet"],
        "extractions": [
            {
                "function": "multirow_patterns",
                "label": "test_single",
                "instructions": {
                    "row_range": [2, 5],  # Excel rows (1-indexed, row 1 is header)
                    "unique_id": "A",  # Just ID column
                    "columns": {"ID": "A", "Name": "B", "Value": "D"},
                },
            }
        ],
    }
]

try:
    result = sheet_excavator.excel_extract([test_file], config1, 5)
    print(f"Raw result type: {type(result)}")
    print(f"Raw result: {result[:200]}..." if len(result) > 200 else f"Raw result: {result}")

    parsed = json.loads(result)
    print(f"Parsed type: {type(parsed)}")

    # Check if parsed is a dict before iterating
    if isinstance(parsed, dict):
        for file_path, file_data in parsed.items():
            if isinstance(file_data, dict):
                for sheet_name, sheet_data in file_data.items():
                    if isinstance(sheet_data, dict):
                        for label, label_data in sheet_data.items():
                            if isinstance(label_data, dict):
                                print(f"Generated keys: {list(label_data.keys())}")
                                for key, value in label_data.items():
                                    print(f"  {key}: {value}")
                            else:
                                print(f"Label data is not a dict: {type(label_data)}")
                    else:
                        print(f"Sheet data is not a dict: {type(sheet_data)}")
            else:
                print(f"File data is not a dict: {type(file_data)}")
    else:
        print(f"Parsed result is not a dict: {parsed}")

    print("✓ Test 1 PASSED")
except Exception as e:
    print(f"✗ Test 1 FAILED: {e}")
    import traceback

    traceback.print_exc()

# Test 2: Composite unique_id
print("\n" + "=" * 60)
print("Test 2: Composite unique_id with array")
print("=" * 60)

config2 = [
    {
        "sheets": ["TestSheet"],
        "extractions": [
            {
                "function": "multirow_patterns",
                "label": "test_composite",
                "instructions": {
                    "row_range": [2, 5],
                    "unique_id": ["A", "C"],  # ID + Year composite key
                    "unique_id_separator": "-",
                    "columns": {"ID": "A", "Name": "B", "Year": "C", "Value": "D"},
                },
            }
        ],
    }
]

try:
    result = sheet_excavator.excel_extract([test_file], config2, 5)
    print(f"Raw result type: {type(result)}")
    print(f"Raw result: {result[:200]}..." if len(result) > 200 else f"Raw result: {result}")

    parsed = json.loads(result)
    print(f"Parsed type: {type(parsed)}")

    # Check if parsed is a dict before iterating
    if isinstance(parsed, dict):
        for file_path, file_data in parsed.items():
            if isinstance(file_data, dict):
                for sheet_name, sheet_data in file_data.items():
                    if isinstance(sheet_data, dict):
                        for label, label_data in sheet_data.items():
                            if isinstance(label_data, dict):
                                print(f"Generated keys: {list(label_data.keys())}")
                                for key, value in label_data.items():
                                    print(f"  {key}: {value}")
                            else:
                                print(f"Label data is not a dict: {type(label_data)}")
                    else:
                        print(f"Sheet data is not a dict: {type(sheet_data)}")
            else:
                print(f"File data is not a dict: {type(file_data)}")
    else:
        print(f"Parsed result is not a dict: {parsed}")

    print("✓ Test 2 PASSED")
except Exception as e:
    print(f"✗ Test 2 FAILED: {e}")
    import traceback

    traceback.print_exc()

# Cleanup
if os.path.exists(test_file):
    os.remove(test_file)
    print(f"\n✓ Cleaned up {test_file}")
