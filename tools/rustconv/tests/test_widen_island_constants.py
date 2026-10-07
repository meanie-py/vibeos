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
