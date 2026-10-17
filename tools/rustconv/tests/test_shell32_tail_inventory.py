from pathlib import Path

from rustconv.shell32_inventory import check_inventory, extract_live_sources


def _write(path: Path, text: str = "// source\n") -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(text)


def _mini_repo(tmp_path: Path) -> tuple[Path, Path]:
    root = tmp_path
    shell = root / "dll/win32/shell32"
    _write(shell / "one.cpp")
    _write(shell / "sub/two.cpp")
    _write(shell / "shim.c")
    _write(
        shell / "CMakeLists.txt",
        """
list(APPEND SOURCE
    one.cpp
    # dead.cpp
    ${CMAKE_CURRENT_BINARY_DIR}/generated.c)
add_subdirectory(sub)
list(APPEND PCH_SKIP_SOURCE shim.c)
""",
    )
    _write(shell / "sub/CMakeLists.txt", "list(APPEND SOURCE two.cpp)\n")
    ledger = root / "ledger.json"
    ledger.write_text(
        """{
  "unit": [
    {"path": "dll/win32/shell32/one.cpp", "language": "cpp",
     "cohort": "pilot", "status": "active"},
    {"path": "dll/win32/shell32/sub/two.cpp", "language": "cpp",
     "cohort": "pilot", "status": "active"},
    {"path": "dll/win32/shell32/shim.c", "language": "c",
     "cohort": "shim", "status": "sanctioned"}
  ]
}\n"""
    )
    return root, ledger


def test_extract_live_sources_ignores_comments_variables_and_generated(tmp_path: Path):
    root, _ = _mini_repo(tmp_path)
    live = extract_live_sources(
        root,
        (
            "dll/win32/shell32/CMakeLists.txt",
            "dll/win32/shell32/sub/CMakeLists.txt",
        ),
    )
    assert live == {
        "dll/win32/shell32/one.cpp",
        "dll/win32/shell32/sub/two.cpp",
        "dll/win32/shell32/shim.c",
    }


def test_inventory_matches_live_cmake(tmp_path: Path):
    root, ledger = _mini_repo(tmp_path)
    report = check_inventory(
        root,
        ledger,
        (
            "dll/win32/shell32/CMakeLists.txt",
            "dll/win32/shell32/sub/CMakeLists.txt",
        ),
    )
    assert report.ok
    assert (report.cpp_files, report.cpp_lines) == (2, 2)
    assert (report.c_files, report.c_lines) == (1, 1)


def test_inventory_fails_loud_on_unclassified_and_stale_status(tmp_path: Path):
    root, ledger = _mini_repo(tmp_path)
    import json

    data = json.loads(ledger.read_text())
    data["unit"][0]["status"] = "removed"
    data["unit"] = [unit for unit in data["unit"] if not unit["path"].endswith("two.cpp")]
    ledger.write_text(json.dumps(data))
    report = check_inventory(
        root,
        ledger,
        (
            "dll/win32/shell32/CMakeLists.txt",
            "dll/win32/shell32/sub/CMakeLists.txt",
        ),
    )
    assert not report.ok
    assert report.unexpectedly_live == ("dll/win32/shell32/one.cpp",)
    assert report.unclassified == ("dll/win32/shell32/sub/two.cpp",)
