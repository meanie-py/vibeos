"""ntdll-rust's committed api-set table is exactly what the C table generates."""

import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))

import gen_apisets_table as gen


def test_committed_table_matches_the_c_table():
    assert gen.OUTPUT.read_text() == gen.render(gen.SOURCE.read_text()), (
        "stale: run python3 tools/gen_apisets_table.py"
    )


def test_commented_entries_are_skipped_and_flags_combine():
    source = (
        '{ RTL_CONSTANT_STRING(L"api-a"), RTL_CONSTANT_STRING(L"k.dll"), '
        "APISET_WIN7 | APISET_WIN10 },\n"
        '// { RTL_CONSTANT_STRING(L"api-b"), RTL_CONSTANT_STRING(L"x.dll"), APISET_WIN8 },\n'
    )
    assert gen.entries(source) == [("api-a", "k.dll", 9)]
