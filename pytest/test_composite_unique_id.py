#!/usr/bin/env python3
"""Test composite unique_id functionality for multirow_patterns"""

import json

import sheet_excavator

# Test file paths (you'll need to update with actual test Excel files)
files = ["pytest/data/ExcelFormaterTest.xlsm"]

# Test case 1: Backwards compatibility - single unique_id as string
test_single_unique_id = [
    {
        "sheets": ["Generell info og kommentarer"],
        "extractions": [
            {
                "function": "multirow_patterns",
                "label": "test_single",
                "instructions": {
                    "row_range": [28, 44],
                    "unique_id": "B",  # Single column as string (backwards compatible)
                    "columns": {"Deposit": "B", "Discovery_well": "C", "Description": "D"},
                },
            }
        ],
    }
]

# Test case 2: Composite unique_id with default separator
test_composite_default = [
    {
        "sheets": ["Generell info og kommentarer"],
        "extractions": [
            {
                "function": "multirow_patterns",
                "label": "test_composite",
                "instructions": {
                    "row_range": [28, 44],
                    "unique_id": ["B", "C"],  # Composite key with default "_" separator
                    "columns": {"Deposit": "B", "Discovery_well": "C", "Description": "D", "Oil_base": "F"},
                },
            }
        ],
    }
]

# Test case 3: Composite unique_id with custom separator
test_composite_custom = [
    {
        "sheets": ["Generell info og kommentarer"],
        "extractions": [
            {
                "function": "multirow_patterns",
                "label": "test_custom_sep",
                "instructions": {
                    "row_range": [28, 44],
                    "unique_id": ["B", "C"],  # Composite key
                    "unique_id_separator": "-",  # Custom separator
                    "columns": {"Deposit": "B", "Discovery_well": "C", "Description": "D"},
                },
            }
        ],
    }
]

# Test case 4: Triple composite key
test_triple_composite = [
    {
        "sheets": ["Generell info og kommentarer"],
        "extractions": [
            {
                "function": "multirow_patterns",
                "label": "test_triple",
                "instructions": {
                    "row_range": [28, 44],
                    "unique_id": ["B", "C", "D"],  # Three columns
                    "unique_id_separator": "|",
                    "columns": {"Deposit": "B", "Discovery_well": "C", "Description": "D", "Oil_base": "F"},
                },
            }
        ],
    }
]


def run_test(test_name, extraction_details):
    """Run a test and print results"""
    print(f"\n{'=' * 60}")
    print(f"Test: {test_name}")
    print(f"{'=' * 60}")

    try:
        results = sheet_excavator.excel_extract(files, extraction_details, 5)
        parsed = json.loads(results)

        # Print the unique keys generated
        for file_path, file_data in parsed.items():
            for sheet_name, sheet_data in file_data.items():
                for label, label_data in sheet_data.items():
                    if isinstance(label_data, dict):
                        print(f"\nLabel: {label}")
                        print(f"Generated keys: {list(label_data.keys())[:5]}...")  # Show first 5 keys
                        if len(label_data) > 5:
                            print(f"  (and {len(label_data) - 5} more)")

        print(f"\n✓ Test passed: {test_name}")
        return True

    except Exception as e:
        print(f"\n✗ Test failed: {test_name}")
        print(f"Error: {str(e)}")
        return False


def test_error_cases():
    """Test error handling for invalid configurations"""
    print(f"\n{'=' * 60}")
    print("Test: Error Cases")
    print(f"{'=' * 60}")

    # Test case: Empty array
    test_empty_array = [
        {
            "sheets": ["Generell info og kommentarer"],
            "extractions": [
                {
                    "function": "multirow_patterns",
                    "instructions": {
                        "row_range": [28, 44],
                        "unique_id": [],  # Empty array - should error
                        "columns": {"Deposit": "B"},
                    },
                }
            ],
        }
    ]

    # Test case: Invalid column in array
    test_invalid_column = [
        {
            "sheets": ["Generell info og kommentarer"],
            "extractions": [
                {
                    "function": "multirow_patterns",
                    "instructions": {
                        "row_range": [28, 44],
                        "unique_id": ["B", "ZZZ"],  # Invalid column
                        "columns": {"Deposit": "B"},
                    },
                }
            ],
        }
    ]

    # Test case: Non-string in array
    test_non_string = [
        {
            "sheets": ["Generell info og kommentarer"],
            "extractions": [
                {
                    "function": "multirow_patterns",
                    "instructions": {
                        "row_range": [28, 44],
                        "unique_id": ["B", 123],  # Non-string value
                        "columns": {"Deposit": "B"},
                    },
                }
            ],
        }
    ]

    error_tests = [
        ("Empty unique_id array", test_empty_array),
        ("Invalid column in unique_id", test_invalid_column),
        ("Non-string in unique_id array", test_non_string),
    ]

    for test_name, test_data in error_tests:
        print(f"\nTesting: {test_name}")
        try:
            sheet_excavator.excel_extract(files, test_data, 5)
            print("  ✗ Should have raised an error but didn't")
        except Exception as e:
            error_msg = str(e)
            print(f"  ✓ Correctly raised error: {error_msg[:100]}...")


def main():
    """Run all tests"""
    print("Testing Composite unique_id Feature")
    print("=" * 60)

    # Run positive tests
    tests = [
        ("Single unique_id (backwards compatibility)", test_single_unique_id),
        ("Composite unique_id with default separator", test_composite_default),
        ("Composite unique_id with custom separator", test_composite_custom),
        ("Triple composite unique_id", test_triple_composite),
    ]

    passed = 0
    failed = 0

    for test_name, test_data in tests:
        if run_test(test_name, test_data):
            passed += 1
        else:
            failed += 1

    # Run error case tests
    test_error_cases()

    # Summary
    print(f"\n{'=' * 60}")
    print("Test Summary")
    print(f"{'=' * 60}")
    print(f"Passed: {passed}/{len(tests)}")
    print(f"Failed: {failed}/{len(tests)}")

    if failed == 0:
        print("\n✓ All tests passed!")
    else:
        print(f"\n✗ {failed} test(s) failed")


if __name__ == "__main__":
    main()
