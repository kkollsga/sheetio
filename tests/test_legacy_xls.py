"""Tests for legacy .xls (BIFF) files.

The rest of the suite builds fixtures with openpyxl, which only writes .xlsx. That
left the .xls reader — a separate calamine code path reached through
``open_workbook_auto`` — entirely uncovered, so a calamine upgrade could change how
legacy files parse without any test noticing.

These tests pin down the parts that differ between the two formats: how a blank cell
is stored, and what that means for the ``stop_if_empty`` stop conditions.
"""

from conftest import run_extract


def single_cells_config(instructions, sheet="Sheet1"):
    return [
        {
            "sheets": [sheet],
            "extractions": [
                {
                    "function": "single_cells",
                    "label": "cells",
                    "instructions": instructions,
                }
            ],
        }
    ]


def multirow_config(stop_if_empty, sheet="Sheet1", row_range=(1, 4)):
    return [
        {
            "sheets": [sheet],
            "extractions": [
                {
                    "function": "multirow_patterns",
                    "label": "rows",
                    "instructions": {
                        "row_range": list(row_range),
                        "unique_id": "A",
                        "stop_if_empty": stop_if_empty,
                        "columns": {"ID": "A", "Name": "B"},
                    },
                }
            ],
        }
    ]


def sheet_data(result, sheet="Sheet1", label="cells"):
    file_key = list(result.keys())[0]
    return result[file_key][sheet][label]


def test_single_cells_reads_xls(tmp_xls_simple):
    """single_cells pulls typed values out of a legacy .xls file."""
    config = single_cells_config({"Value 1": "a1", "Value 2": "b2", "Value 3": "c3", "Date": "d4", "Datetime": "e5"})
    data = sheet_data(run_extract([tmp_xls_simple], config))

    assert data["Value 1"] == "Title Value"
    assert data["Value 2"] == "Description Text"
    assert data["Value 3"] == 12345.0
    assert "Date" in data
    assert "Datetime" in data


def test_multirow_reads_xls(tmp_xls_multirow):
    """multirow_patterns keys rows by unique_id from a legacy .xls file."""
    config = [
        {
            "sheets": ["TestSheet"],
            "extractions": [
                {
                    "function": "multirow_patterns",
                    "label": "rows",
                    "instructions": {
                        "row_range": [2, 4],
                        "unique_id": "A",
                        "columns": {"ID": "A", "Name": "B", "Value": "C"},
                    },
                }
            ],
        }
    ]
    rows = sheet_data(run_extract([tmp_xls_multirow], config), "TestSheet", "rows")

    assert list(rows.keys()) == ["001", "002", "003"]
    assert rows["002"]["Name"] == "Bob"
    assert rows["003"]["Value"] == 300.0


def test_blank_cell_is_null_but_empty_string_is_preserved(tmp_xls_blank_kinds):
    """A never-written cell reads as null; a stored empty string stays a string.

    calamine 0.36 started preserving empty strings in .xls rather than discarding
    them, so these two cases are genuinely distinct on the wire and this test is
    what would catch that semantic flipping back.
    """
    config = single_cells_config({"real": "A1", "empty_string": "A2", "whitespace": "A3", "never_written": "A4"})
    data = sheet_data(run_extract([tmp_xls_blank_kinds], config))

    assert data["real"] == "A1val"
    assert data["empty_string"] == ""
    assert data["whitespace"] == ""
    assert data["never_written"] is None


def test_row_mode_stop_halts_on_empty_string_row(tmp_xls_empty_string_row):
    """Row-mode stop_if_empty treats an all-empty-string row as the end of the data.

    Regression test: the row-mode check previously only tested for null, so on .xls
    -- where a blank row can be stored as empty strings -- it ran straight past the
    separator row and picked up unrelated trailing data.
    """
    rows = sheet_data(run_extract([tmp_xls_empty_string_row], multirow_config({"mode": "row"})), label="rows")

    assert list(rows.keys()) == ["001", "002"]


def test_column_mode_stop_halts_on_empty_string_row(tmp_xls_empty_string_row):
    """Column-mode stop_if_empty stops on the same row as row-mode."""
    rows = sheet_data(run_extract([tmp_xls_empty_string_row], multirow_config("A")), label="rows")

    assert list(rows.keys()) == ["001", "002"]


def test_stop_modes_agree_on_whitespace_row_in_xlsx(tmp_excel_whitespace_row):
    """The same agreement holds for whitespace-only rows in .xlsx.

    Row-mode and column-mode are both spelled ``stop_if_empty``, so they must agree
    on what counts as empty regardless of file format.
    """
    row_mode = sheet_data(run_extract([tmp_excel_whitespace_row], multirow_config({"mode": "row"})), label="rows")
    column_mode = sheet_data(run_extract([tmp_excel_whitespace_row], multirow_config("A")), label="rows")

    assert list(row_mode.keys()) == ["001", "002"]
    assert list(column_mode.keys()) == ["001", "002"]


def test_xls_and_xlsx_extract_alike(tmp_xls_simple, tmp_excel_simple):
    """The same config over equivalent .xls and .xlsx files yields the same values."""
    config = single_cells_config({"Value 1": "a1", "Value 2": "b2", "Value 3": "c3"})

    xls = sheet_data(run_extract([tmp_xls_simple], config))
    xlsx = sheet_data(run_extract([tmp_excel_simple], config))

    assert xls == xlsx
