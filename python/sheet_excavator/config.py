"""ExtractionConfig builder for sheet_excavator extraction configurations."""

from __future__ import annotations

import json
from typing import Any


class ExtractionConfig:
    """Builder for sheet_excavator extraction configurations.

    Provides a fluent API for constructing extraction configs iteratively,
    with JSON import/export and direct execution support.

    Example:
        config = ExtractionConfig()
        config.add_sheets(["Sheet1"]) \\
            .single_cells("header", title="B2", date="D4") \\
            .multirow("items", row_range=(10, 100), unique_id="A",
                       columns={"ID": "A", "Name": "B"})

        result = config.extract(["form.xlsx"])
    """

    def __init__(self) -> None:
        self._groups: list[SheetGroup] = []

    def add_sheets(
        self,
        sheets: list[str],
        *,
        skip: list[str] | None = None,
        break_if_null: str | None = None,
    ) -> SheetGroup:
        """Add a sheet group and return it for chaining extractions.

        Args:
            sheets: Sheet names or wildcard patterns (e.g. "School_*").
            skip: Sheet names to exclude.
            break_if_null: Cell address; skip sheet if this cell is empty.
        """
        group = SheetGroup(self, sheets, skip, break_if_null)
        self._groups.append(group)
        return group

    def build(self) -> list[dict[str, Any]]:
        """Build the raw extraction_details config list.

        Returns:
            List of dicts suitable for passing to excel_extract().
        """
        return [g._build() for g in self._groups]

    def extract(self, files: list[str], *, workers: int = 5) -> dict[str, Any]:
        """Run extraction and return parsed results.

        Args:
            files: Paths to Excel files.
            workers: Number of parallel workers.

        Returns:
            Parsed dict of extraction results.
        """
        from sheet_excavator._sheet_excavator import excel_extract

        return json.loads(excel_extract(files, self.build(), workers))

    def to_json(self, path: str) -> None:
        """Save config to a JSON file."""
        with open(path, "w") as f:
            json.dump(self.build(), f, indent=2)

    @classmethod
    def from_json(cls, path: str) -> ExtractionConfig:
        """Load config from a JSON file."""
        with open(path) as f:
            data = json.load(f)
        config = cls()
        for group_dict in data:
            group = config.add_sheets(
                group_dict["sheets"],
                skip=group_dict.get("skip_sheets"),
                break_if_null=group_dict.get("break_if_null"),
            )
            for ext in group_dict.get("extractions", []):
                label = ext.get("label")
                key = label if label else group._next_unlabeled_key(ext["function"])
                group._extractions[key] = ext
        return config

    def summary(self) -> None:
        """Print a human-readable summary of the config."""
        print(f"ExtractionConfig ({len(self._groups)} sheet group{'s' if len(self._groups) != 1 else ''})")
        for i, group in enumerate(self._groups, 1):
            skip_info = f", skip={group._skip}" if group._skip else ""
            brk_info = f", break_if_null={group._break_if_null}" if group._break_if_null else ""
            print(f"\n  Group {i}: sheets={group._sheets}{skip_info}{brk_info}")
            extractions = list(group._extractions.values())
            for j, ext in enumerate(extractions):
                fn = ext["function"]
                label = ext.get("label", "")
                prefix = "  └─" if j == len(extractions) - 1 else "  ├─"
                label_str = f' "{label}"' if label else ""
                instr = ext.get("instructions", {})
                details = _format_extraction_details(fn, instr)
                print(f"  {prefix} [{fn}]{label_str} {details}")


def _format_extraction_details(fn: str, instr: dict) -> str:
    """Format extraction instructions for summary display."""
    if fn == "single_cells":
        cells = ", ".join(f"{k}->{v}" for k, v in instr.items())
        return cells
    elif fn == "multirow_patterns":
        parts = []
        if "row_range" in instr:
            parts.append(f"rows {instr['row_range'][0]}-{instr['row_range'][1]}")
        if "unique_id" in instr:
            parts.append(f"unique_id={instr['unique_id']}")
        if "columns" in instr:
            cols = ", ".join(f"{k}->{v}" for k, v in instr["columns"].items())
            parts.append(f"columns: {cols}")
        return ", ".join(parts)
    elif fn == "dataframe":
        parts = []
        if "row_range" in instr:
            parts.append(f"rows {instr['row_range'][0]}-{instr['row_range'][1]}")
        if "column_range" in instr:
            parts.append(f"cols {instr['column_range'][0]}-{instr['column_range'][1]}")
        if "header_row" in instr:
            parts.append(f"header={instr['header_row']}")
        return ", ".join(parts)
    return ""


class SheetGroup:
    """A group of sheets with shared extractions. Supports method chaining."""

    def __init__(
        self,
        parent: ExtractionConfig,
        sheets: list[str],
        skip: list[str] | None,
        break_if_null: str | None,
    ) -> None:
        self._parent = parent
        self._sheets = sheets
        self._skip = skip
        self._break_if_null = break_if_null
        self._extractions: dict[str, dict[str, Any]] = {}
        self._unlabeled_counter = 0

    def _next_unlabeled_key(self, function: str) -> str:
        self._unlabeled_counter += 1
        return f"_unlabeled_{function}_{self._unlabeled_counter}"

    def single_cells(
        self,
        label: str | None = None,
        cells: dict[str, str | list[str]] | None = None,
        *,
        break_if_null: str | None = None,
        **kwargs: str | list[str],
    ) -> SheetGroup:
        """Add a single_cells extraction.

        Args:
            label: Optional label for nesting results.
            cells: Dict mapping names to cell addresses.
            break_if_null: Cell address; skip extraction if cell is empty.
            **kwargs: Additional name=cell_address mappings.

        Example:
            group.single_cells("header", title="B2", date="D4")
            group.single_cells("header", cells={"Report Title": "B2"})
        """
        instructions: dict[str, Any] = {}
        if cells:
            instructions.update(cells)
        instructions.update(kwargs)

        ext: dict[str, Any] = {"function": "single_cells", "instructions": instructions}
        if label:
            ext["label"] = label
        if break_if_null:
            ext["break_if_null"] = break_if_null

        key = label if label else self._next_unlabeled_key("single_cells")
        self._extractions[key] = ext
        return self

    def multirow(
        self,
        label: str | None = None,
        *,
        row_range: tuple[int, int] | list[int],
        columns: dict[str, str | list[str]],
        unique_id: str | list[str] | None = None,
        unique_id_separator: str | None = None,
        stop_if_empty: str | list[str] | dict[str, Any] | None = None,
        stop_consecutive: int | None = None,
        break_if_null: str | None = None,
    ) -> SheetGroup:
        """Add a multirow_patterns extraction.

        Args:
            label: Optional label for nesting results.
            row_range: (start_row, end_row), 1-based.
            columns: Dict mapping names to column letters.
            unique_id: Column(s) for dict keys. Omit for array result.
            unique_id_separator: Separator for composite keys (default "_").
            stop_if_empty: Column(s) or dict config for stop condition.
            stop_consecutive: Consecutive empty rows before stopping.
            break_if_null: Cell address; skip extraction if cell is empty.
        """
        instructions: dict[str, Any] = {
            "row_range": list(row_range),
            "columns": columns,
        }
        if unique_id is not None:
            instructions["unique_id"] = unique_id
        if unique_id_separator is not None:
            instructions["unique_id_separator"] = unique_id_separator
        if stop_if_empty is not None:
            instructions["stop_if_empty"] = stop_if_empty
        if stop_consecutive is not None:
            instructions["stop_consecutive"] = stop_consecutive

        ext: dict[str, Any] = {"function": "multirow_patterns", "instructions": instructions}
        if label:
            ext["label"] = label
        if break_if_null:
            ext["break_if_null"] = break_if_null

        key = label if label else self._next_unlabeled_key("multirow_patterns")
        self._extractions[key] = ext
        return self

    def dataframe(
        self,
        label: str | None = None,
        *,
        row_range: tuple[int, int] | list[int],
        column_range: list[str],
        header_row: int | list[int],
        separator: str = " ",
        break_if_null: str | None = None,
    ) -> SheetGroup:
        """Add a dataframe extraction.

        Args:
            label: Optional label for nesting results.
            row_range: (start_row, end_row), 1-based.
            column_range: [start_col, end_col] as column letters.
            header_row: Row number(s) for column headers.
            separator: Separator for joining multi-row headers.
            break_if_null: Cell address; skip extraction if cell is empty.
        """
        header = header_row if isinstance(header_row, list) else [header_row]
        instructions: dict[str, Any] = {
            "row_range": list(row_range),
            "column_range": column_range,
            "header_row": header,
        }
        if separator != " ":
            instructions["separator"] = separator

        ext: dict[str, Any] = {"function": "dataframe", "instructions": instructions}
        if label:
            ext["label"] = label
        if break_if_null:
            ext["break_if_null"] = break_if_null

        key = label if label else self._next_unlabeled_key("dataframe")
        self._extractions[key] = ext
        return self

    def remove(self, label: str) -> SheetGroup:
        """Remove an extraction by label.

        Raises:
            KeyError: If label not found.
        """
        if label not in self._extractions:
            raise KeyError(f"No extraction with label '{label}'")
        del self._extractions[label]
        return self

    def add_sheets(
        self,
        sheets: list[str],
        *,
        skip: list[str] | None = None,
        break_if_null: str | None = None,
    ) -> SheetGroup:
        """Create a new sheet group (delegates to parent)."""
        return self._parent.add_sheets(sheets, skip=skip, break_if_null=break_if_null)

    def build(self) -> list[dict[str, Any]]:
        """Delegate to parent's build()."""
        return self._parent.build()

    def extract(self, files: list[str], *, workers: int = 5) -> dict[str, Any]:
        """Delegate to parent's extract()."""
        return self._parent.extract(files, workers=workers)

    def _build(self) -> dict[str, Any]:
        """Build this group's config dict."""
        result: dict[str, Any] = {
            "sheets": self._sheets,
            "extractions": list(self._extractions.values()),
        }
        if self._skip:
            result["skip_sheets"] = self._skip
        if self._break_if_null:
            result["break_if_null"] = self._break_if_null
        return result
