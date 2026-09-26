"""sdk/tools/gen-delaylib.py: a NONAME export must be delay-bound by ORDINAL.

The DLL exports no name for `Name @127 NONAME`, so a by-name INT entry can never resolve and
every call raises 0xc06d007f. That was shell32 -> browseui!SHOnCWMCommandLine, i.e. every
folder window opened from another process.
"""

import importlib.util
from pathlib import Path

import pytest

REPO = Path(__file__).resolve().parents[1]
_spec = importlib.util.spec_from_file_location(
    "gen_delaylib", REPO / "sdk" / "tools" / "gen-delaylib.py"
)
assert _spec is not None and _spec.loader is not None
gen = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(gen)

DEF = """LIBRARY browseui.dll
EXPORTS
 ByName @101
 SHOnCWMCommandLine @127 NONAME
 Data @130 DATA
"""


@pytest.fixture
def exports(tmp_path):
    p = tmp_path / "x.def"
    p.write_text(DEF)
    dll, exports = gen.parse_def(str(p))
    assert dll == "browseui.dll"
    return exports


def test_noname_export_carries_its_ordinal(exports):
    assert ("SHOnCWMCommandLine", False, 127) in exports
    assert ("ByName", False, None) in exports


@pytest.mark.parametrize(
    ("arch", "entry"),
    [("i386", "\t.long 0x8000007f"), ("amd64", "\t.long 0x7f\n\t.long 0x80000000")],
)
def test_noname_export_is_bound_by_ordinal(exports, arch, entry):
    asm = gen.emit("browseui.dll", exports, arch)
    assert entry in asm
    # ...and never by name: no hint/name string for it.
    assert '.asciz "SHOnCWMCommandLine"' not in asm
    assert '.asciz "ByName"' in asm


def test_noname_without_ordinal_fails_loud(tmp_path):
    p = tmp_path / "x.def"
    p.write_text("LIBRARY a.dll\nEXPORTS\n Broken NONAME\n")
    with pytest.raises(SystemExit, match="no @ordinal"):
        gen.parse_def(str(p))
