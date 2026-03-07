#!/usr/bin/env python3
import json
import os
import tempfile

import sheet_excavator
from openpyxl import Workbook

temp_dir = tempfile.mkdtemp()
filepath = os.path.join(temp_dir, "test.xlsx")

wb = Workbook()
ws1 = wb.active
ws1.title = "Sheet1"
ws1["A1"] = "Value A"
ws1["C3"] = "Not Null"

ws2 = wb.create_sheet("Sheet2")
ws2["A1"] = "Value B"
ws2["C3"] = None  # Explicitly set to None

wb.save(filepath)

config = [
    {
        "sheets": ["Sheet1", "Sheet2"],
        "extractions": [{"function": "single_cells", "break_if_null": "c3", "instructions": {"test": "a1"}}],
    }
]

result = sheet_excavator.excel_extract([filepath], config, 1)
parsed = json.loads(result)

print("Result:")
print(json.dumps(parsed, indent=2))

file_key = list(parsed.keys())[0]
print(f"\nSheets in result: {[k for k in parsed[file_key].keys() if k != 'filepath']}")

os.remove(filepath)
os.rmdir(temp_dir)
