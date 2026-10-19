"""The native generator derives ntdll's committed table from the C source."""

import subprocess
from pathlib import Path

REPO = Path(__file__).resolve().parents[1]


def test_committed_table_matches_the_c_table():
    result = subprocess.run(
        [str(REPO / "tools/codegen"), "apisets-table", "--check"],
        capture_output=True,
        text=True,
    )
    assert result.returncode == 0, result.stdout + result.stderr


def test_commented_entries_are_skipped_and_flags_combine(tmp_path):
    source = tmp_path / "apisets.c"
    source.write_text(
        '{ RTL_CONSTANT_STRING(L"api-a"), RTL_CONSTANT_STRING(L"k.dll"), '
        "APISET_WIN7 | APISET_WIN10 },\n"
        '// { RTL_CONSTANT_STRING(L"api-b"), RTL_CONSTANT_STRING(L"x.dll"), APISET_WIN8 },\n'
    )
    result = subprocess.run(
        [str(REPO / "tools/codegen"), "apisets-table", "--input", str(source), "--stdout"],
        capture_output=True,
        text=True,
    )
    assert result.returncode == 0, result.stderr
    assert "dw_os_versions: 0x9" in result.stdout
    assert "static N1" not in result.stdout
