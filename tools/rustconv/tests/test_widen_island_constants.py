"""widen_island_constants: bindgen's narrowed `L"..."` macro constants, checked against the C."""

import importlib.util
import pathlib
import sys

_MOD = pathlib.Path(__file__).resolve().parents[1] / "widen_island_constants.py"
_spec = importlib.util.spec_from_file_location("widen_island_constants", _MOD)
assert _spec and _spec.loader
wic = importlib.util.module_from_spec(_spec)
sys.modules["widen_island_constants"] = wic
_spec.loader.exec_module(wic)

_ISLAND = (
    'pub const DD_ATA_REG_MAX_TARGET_ID: &[u8; 12] = b"MaxTargetId\\0";\n'
    'pub const NARROW_FOR_REAL: &[u8; 4] = b"abc\\0";\n'
    'pub const TWO_WAYS: &[u8; 2] = b"x\\0";\n'
    'pub const DIFFERENT: &[u8; 4] = b"abc\\0";\n'
)


def test_a_constant_the_c_defines_wide_becomes_a_utf16_array():
    wide = {"DD_ATA_REG_MAX_TARGET_ID": "MaxTargetId", "TWO_WAYS": None, "DIFFERENT": "abd"}
    out, widened, refused = wic.widen(_ISLAND, wide)
    assert widened == ["DD_ATA_REG_MAX_TARGET_ID"]
    assert (
        "pub const DD_ATA_REG_MAX_TARGET_ID: &[u16; 12] = &[0x004D, 0x0061, 0x0078, 0x0054, "
        '0x0061, 0x0072, 0x0067, 0x0065, 0x0074, 0x0049, 0x0064, 0x0000]; // L"MaxTargetId"'
    ) in out
    # A narrow C string is untouched; two definitions or a different text are refused, loudly.
    assert 'pub const NARROW_FOR_REAL: &[u8; 4] = b"abc\\0";' in out
    assert sorted(refused) == ["DIFFERENT", "TWO_WAYS"]


def test_the_oracle_reads_one_text_per_name(tmp_path):
    import subprocess

    subprocess.run(["git", "init", "-q"], cwd=tmp_path, check=True)
    (tmp_path / "a.h").write_text('#define A L"x"\n#define B L"y"\n#define N "narrow"\n')
    (tmp_path / "b.h").write_text('#define A L"x"\n#define B L"z"\n#define P L"C:\\\\dir"\n')
    subprocess.run(["git", "add", "-A"], cwd=tmp_path, check=True)
    assert wic.wide_defines(tmp_path) == {"A": "x", "B": None, "P": "C:\\dir"}


def test_a_unicode_islands_aw_switch_is_widened_through_its_w_target():
    """`#define FINDMSGSTRING FINDMSGSTRINGW` (UNICODE) / `...A` (not): bindgen evaluates the
    chain to `L"commdlg_FindReplace"` and drops the L; only the W text is the oracle."""
    island = 'pub type TCHAR = WCHAR;\npub const FINDMSGSTRING: &[u8; 4] = b"abc\\0";\n'
    wide = {"FINDMSGSTRINGW": "abc"}
    aliases = {"FINDMSGSTRING": {"FINDMSGSTRINGW", "FINDMSGSTRINGA"}}
    out, widened, _ = wic.widen(island, wide, aliases)
    assert widened == ["FINDMSGSTRINGW"]
    assert "pub const FINDMSGSTRING: &[u16; 4] = &[0x0061, 0x0062, 0x0063, 0x0000];" in out
    # An ANSI island, or one that does not say, keeps the narrow bytes.
    for header in ("pub type TCHAR = CHAR;\n", ""):
        out, widened, _ = wic.widen(
            island.replace("pub type TCHAR = WCHAR;\n", header), wide, aliases
        )
        assert widened == [] and 'b"abc\\0"' in out


def test_an_island_without_tchar_takes_the_charset_from_the_compile():
    # kbswitch: no TCHAR in the island, -DUNICODE on the compile; INDICATOR_CLASS stayed narrow.
    text = 'pub const INDICATOR_CLASS: &[u8; 10] = b"Indicator\\0";\n'
    wide = {"INDICATOR_CLASSW": "Indicator"}
    aliases = {"INDICATOR_CLASS": {"INDICATOR_CLASSW", "INDICATOR_CLASSA"}}
    same, widened, _ = wic.widen(text, wide, aliases)
    assert widened == [] and same == text
    new, widened, _ = wic.widen(text, wide, aliases, "W")
    assert widened == ["INDICATOR_CLASSW"]
    assert "pub const INDICATOR_CLASS: &[u16; 10] = &[0x0049," in new
