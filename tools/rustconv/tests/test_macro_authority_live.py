"""live_defines' parse: every file the preprocessor entered is live, defines or not."""

from pathlib import Path

from rustconv.macro_authority import _collect_defines


def test_an_entered_header_with_no_defines_is_still_live(tmp_path: Path) -> None:
    tu, header, other = tmp_path / "main.c", tmp_path / "private.h", tmp_path / "defs.h"
    out = (
        f'# 1 "{tu}"\n'
        '# 1 "<built-in>" 1\n'
        "#define __STDC__ 1\n"
        f'# 1 "{header}" 1\n'
        "static inline int helper(void) { return 0; }\n"
        f'# 1 "{other}" 1\n'
        "#define ONE 1\n"
        f'# 3 "{tu}" 2\n'
        "#define LOCAL 2\n"
    )
    live: dict[Path, set[str]] = {}
    _collect_defines(out, live)
    assert live[header.resolve()] == set()
    assert live[other.resolve()] == {"ONE"}
    assert live[tu.resolve()] == {"LOCAL"}
    assert not any(str(k).startswith("<") for k in live)
