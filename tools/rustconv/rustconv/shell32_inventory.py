"""Live-CMake inventory ratchet for the shell32 C/C++ translation tail.

The ledger is intentionally separate from CMake: CMake answers what is compiled now;
the ledger answers which translation cohort owns it and whether it is expected to remain.
Any source added silently, removed without updating the ledger, or marked removed while it
is still live is a hard failure.
"""

from __future__ import annotations

import json
from dataclasses import dataclass
from pathlib import Path

DEFAULT_CMAKE_FILES = (
    "dll/win32/shell32/CMakeLists.txt",
    "dll/win32/shell32/shelldesktop/CMakeLists.txt",
    "dll/win32/shell32/shellmenu/CMakeLists.txt",
    "dll/win32/shell32/shellrecyclebin/CMakeLists.txt",
)
DEFAULT_LEDGER = "tools/rustconv/data/shell32_cpp_tail.json"
VALID_STATUS = {"active", "removed", "sanctioned"}
VALID_LANGUAGE = {"c", "cpp"}


@dataclass(frozen=True)
class LedgerUnit:
    path: str
    language: str
    cohort: str
    status: str


@dataclass(frozen=True)
class InventoryReport:
    ok: bool
    live: tuple[str, ...]
    expected_live: tuple[str, ...]
    unclassified: tuple[str, ...]
    unexpectedly_live: tuple[str, ...]
    expected_but_absent: tuple[str, ...]
    missing_files: tuple[str, ...]
    duplicate_ledger_paths: tuple[str, ...]
    cpp_files: int
    cpp_lines: int
    c_files: int
    c_lines: int

    def to_dict(self) -> dict[str, object]:
        return {
            "ok": self.ok,
            "live": list(self.live),
            "expected_live": list(self.expected_live),
            "unclassified": list(self.unclassified),
            "unexpectedly_live": list(self.unexpectedly_live),
            "expected_but_absent": list(self.expected_but_absent),
            "missing_files": list(self.missing_files),
            "duplicate_ledger_paths": list(self.duplicate_ledger_paths),
            "cpp_files": self.cpp_files,
            "cpp_lines": self.cpp_lines,
            "c_files": self.c_files,
            "c_lines": self.c_lines,
        }


def _source_tokens(cmake_text: str) -> list[str]:
    """Return literal C/C++ path tokens, ignoring variables and comments.

    This deliberately recognizes only files that exist in the source checkout. Generated
    `${CMAKE_CURRENT_BINARY_DIR}/shell32_stubs.c` therefore cannot masquerade as a source TU.
    Parentheses are separators so `CFolderItemVerbs.cpp)` is still inventoried.
    """

    found: list[str] = []
    for raw_line in cmake_text.splitlines():
        code = raw_line.split("#", 1)[0]
        for token in code.replace("(", " ").replace(")", " ").split():
            if "$" in token or not token.endswith((".c", ".cpp")):
                continue
            found.append(token)
    return found


def extract_live_sources(
    repo: Path, cmake_files: tuple[str, ...] = DEFAULT_CMAKE_FILES
) -> set[str]:
    live: set[str] = set()
    for cmake_rel in cmake_files:
        cmake = repo / cmake_rel
        for token in _source_tokens(cmake.read_text()):
            source = (cmake.parent / token).resolve()
            if not source.is_file():
                continue
            try:
                rel = source.relative_to(repo.resolve()).as_posix()
            except ValueError:
                continue
            live.add(rel)
    return live


def load_ledger(path: Path) -> tuple[list[LedgerUnit], tuple[str, ...]]:
    data = json.loads(path.read_text())
    units: list[LedgerUnit] = []
    for status in VALID_STATUS:
        for cohort, paths in data.get(status, {}).items():
            for unit_path in paths:
                units.append(
                    LedgerUnit(
                        path=str(unit_path),
                        language="cpp" if str(unit_path).endswith(".cpp") else "c",
                        cohort=str(cohort),
                        status=status,
                    )
                )
    seen: set[str] = set()
    duplicates: set[str] = set()
    for raw in data.get("unit", []):
        unit = LedgerUnit(
            path=str(raw["path"]),
            language=str(raw["language"]),
            cohort=str(raw["cohort"]),
            status=str(raw["status"]),
        )
        if unit.language not in VALID_LANGUAGE:
            raise ValueError(f"{unit.path}: invalid language {unit.language!r}")
        if unit.status not in VALID_STATUS:
            raise ValueError(f"{unit.path}: invalid status {unit.status!r}")
        expected_suffix = ".cpp" if unit.language == "cpp" else ".c"
        if not unit.path.endswith(expected_suffix):
            raise ValueError(f"{unit.path}: language {unit.language!r} disagrees with suffix")
        if unit.path in seen:
            duplicates.add(unit.path)
        seen.add(unit.path)
        units.append(unit)
    return units, tuple(sorted(duplicates))


def check_inventory(
    repo: Path,
    ledger_path: Path | None = None,
    cmake_files: tuple[str, ...] = DEFAULT_CMAKE_FILES,
) -> InventoryReport:
    ledger_path = ledger_path or repo / DEFAULT_LEDGER
    units, duplicates = load_ledger(ledger_path)
    live = extract_live_sources(repo, cmake_files)
    by_path = {unit.path: unit for unit in units}
    expected_live = {unit.path for unit in units if unit.status in {"active", "sanctioned"}}
    removed = {unit.path for unit in units if unit.status == "removed"}
    unclassified = live - set(by_path)
    unexpectedly_live = live & removed
    expected_but_absent = expected_live - live
    missing_files = {unit.path for unit in units if not (repo / unit.path).is_file()}

    cpp = sorted(path for path in live if path.endswith(".cpp"))
    c = sorted(path for path in live if path.endswith(".c"))
    cpp_lines = sum(len((repo / path).read_text(errors="replace").splitlines()) for path in cpp)
    c_lines = sum(len((repo / path).read_text(errors="replace").splitlines()) for path in c)
    ok = not (
        unclassified or unexpectedly_live or expected_but_absent or missing_files or duplicates
    )
    return InventoryReport(
        ok=ok,
        live=tuple(sorted(live)),
        expected_live=tuple(sorted(expected_live)),
        unclassified=tuple(sorted(unclassified)),
        unexpectedly_live=tuple(sorted(unexpectedly_live)),
        expected_but_absent=tuple(sorted(expected_but_absent)),
        missing_files=tuple(sorted(missing_files)),
        duplicate_ledger_paths=duplicates,
        cpp_files=len(cpp),
        cpp_lines=cpp_lines,
        c_files=len(c),
        c_lines=c_lines,
    )


def print_report(report: InventoryReport, *, as_json: bool = False) -> None:
    if as_json:
        print(json.dumps(report.to_dict(), indent=2, sort_keys=True))
        return
    verdict = "OK" if report.ok else "FAIL"
    print(
        f"shell32-tail-inventory: {verdict} — "
        f"{report.cpp_files} cpp/{report.cpp_lines} lines, "
        f"{report.c_files} c/{report.c_lines} lines"
    )
    for label, paths in (
        ("unclassified live source", report.unclassified),
        ("marked removed but still live", report.unexpectedly_live),
        ("expected live but absent", report.expected_but_absent),
        ("ledger path missing on disk", report.missing_files),
        ("duplicate ledger path", report.duplicate_ledger_paths),
    ):
        for path in paths:
            print(f"  {label}: {path}")
