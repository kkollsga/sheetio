import glob

import sheet_excavator

files = glob.glob(r"D:\temp\*")
print(files)
# files = ['D:\\temp\\File3.xlsm']
extraction_details = [
    {
        "sheets": ["Sheet1"],
        "extractions": [
            {
                "function": "single_cells",
                "label": "single",
                "instructions": {"1": "a1", "2": "b2", "3": "c3", "dato": "d4", "datotid": "e5"},
            }
        ],
    },
    {
        "sheets": ["Innledning"],
        "extractions": [
            {
                "function": "single_cells",
                "instructions": {"navn": "b34", "dato": "d34", "person": "d29", "telefon": "f29"},
            }
        ],
    },
    {
        "sheets": ["Generell info og kommentarer"],
        "extractions": [
            {"function": "single_cells", "label": "single", "instructions": {"field": "d7", "od-id": "m8"}},
            {
                "function": "multirow_patterns",
                "label": "deposits",
                "instructions": {
                    "row_range": [28, 44],
                    "unique_id": "B",
                    "columns": {
                        "Deposit": "B",
                        "Discovery_well": "C",
                        "Description": "D",
                        "Oil_low": "E",
                        "Oil_base": "F",
                        "Oil_high": "G",
                        "Cond_low": "H",
                        "Cond_base": "I",
                        "Cond_high": "J",
                        "AssGass_low": "K",
                        "AssGass_base": "L",
                        "AssGass_high": "M",
                        "FriGass_low": "N",
                        "FriGass_base": "O",
                        "FriGass_high": "P",
                    },
                },
            },
        ],
    },
    {
        "sheets": ["Profil_*"],
        "extractions": [
            {
                "function": "single_cells",
                "label": "single",
                "instructions": {"project_name": ["h7", "h8", "h9", "h10", "h11"]},
            },
            {
                "function": "multirow_patterns",
                "label": "projects",
                "instructions": {
                    "row_range": [7, 25],
                    "unique_id": "H",
                    "columns": {
                        "Deposit": "H",
                        "Oil": "M",
                        "NGL": "P",
                        "Gas": "S",
                        "Cond": "V",
                        "Deposits": ["X", "Y", "Z", "AA"],
                    },
                },
            },
            {
                "function": "dataframe",
                "label": "tabelTest",
                "instructions": {"row_range": [39, 64], "column_range": ["H", "AA"], "header_row": [34, 35]},
            },
        ],
        "skip_sheets": ["Profil_Total"],
        "break_if_null": "h7",
    },
]

results = sheet_excavator.excel_extract(files, extraction_details, 10)
# print(json.dumps(json.loads(results), indent=3))
