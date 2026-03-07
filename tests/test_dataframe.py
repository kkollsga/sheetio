"""Tests for dataframe extraction function."""

from conftest import run_extract


def test_basic_extraction(tmp_excel_dataframe):
    """Multi-row header + data extraction."""
    config = [
        {
            "sheets": ["School_A"],
            "extractions": [
                {
                    "function": "dataframe",
                    "label": "DataFrame",
                    "instructions": {
                        "row_range": [5, 15],
                        "column_range": ["B", "F"],
                        "header_row": [2, 3, 4],
                        "separator": " ",
                    },
                }
            ],
        }
    ]
    result = run_extract([tmp_excel_dataframe], config)
    file_key = list(result.keys())[0]
    df_data = result[file_key]["School_A"]["DataFrame"]

    assert isinstance(df_data, dict)
    keys = list(df_data.keys())
    assert len(keys) == 5  # 5 columns (B through F)


def test_header_concatenation(tmp_excel_dataframe):
    """Verify header join with separator."""
    config = [
        {
            "sheets": ["School_A"],
            "extractions": [
                {
                    "function": "dataframe",
                    "label": "DataFrame",
                    "instructions": {
                        "row_range": [5, 15],
                        "column_range": ["B", "F"],
                        "header_row": [2, 3, 4],
                        "separator": " ",
                    },
                }
            ],
        }
    ]
    result = run_extract([tmp_excel_dataframe], config)
    file_key = list(result.keys())[0]
    df_data = result[file_key]["School_A"]["DataFrame"]

    keys = list(df_data.keys())
    # Headers should be concatenated from 3 rows
    assert any("Student" in k for k in keys)
    assert any("Math" in k for k in keys)


def test_column_range(tmp_excel_dataframe):
    """Correct range ["C","E"] extracts only 3 columns."""
    config = [
        {
            "sheets": ["School_A"],
            "extractions": [
                {
                    "function": "dataframe",
                    "label": "DataFrame",
                    "instructions": {
                        "row_range": [5, 15],
                        "column_range": ["C", "E"],
                        "header_row": [2, 3, 4],
                        "separator": " ",
                    },
                }
            ],
        }
    ]
    result = run_extract([tmp_excel_dataframe], config)
    file_key = list(result.keys())[0]
    df_data = result[file_key]["School_A"]["DataFrame"]

    keys = list(df_data.keys())
    assert len(keys) == 3  # C, D, E only
