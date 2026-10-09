"""Unit tests for scan_call_literal_vs_c.

Pins both oracles -- an ALL_CAPS macro resolved from the crate's consts, and an enum member
resolved from the bindgen islands -- and that the island outranks a crate's own copy, which
can be invented (fastfat's `MmFlushForWrite = 2`; mmtypes.h says 1).
"""

from __future__ import annotations

import importlib.util
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent
_spec = importlib.util.spec_from_file_location(
    "scan_call_literal_vs_c", REPO / "tools" / "scan_call_literal_vs_c.py"
)
assert _spec is not None and _spec.loader is not None
scl = importlib.util.module_from_spec(_spec)
sys.modules["scan_call_literal_vs_c"] = scl
_spec.loader.exec_module(scl)

_C = """
// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
"""


def _scan(tmp_path: Path, rust: str, c: str) -> list[str]:
    (tmp_path / "Cargo.toml").write_text("[package]\nname = 'x'\n")
    f = tmp_path / "x.rs"
    f.write_text(rust + _C + "\n".join("// " + ln for ln in c.splitlines()) + "\n")
    return [what for _, _, what in scl.scan([f])]


def test_all_caps_macro_from_crate_const(tmp_path: Path) -> None:
    hits = _scan(
        tmp_path,
        "const CM_CHECK_REGISTRY_PURGE_VOLATILES: u32 = 2;\n"
        "pub unsafe fn f() { CmpCmdHiveOpen(a, b, 1u32); }\n",
        "CmpCmdHiveOpen(a, b, CM_CHECK_REGISTRY_PURGE_VOLATILES);",
    )
    assert hits and hits[0].startswith("MISMATCH CmpCmdHiveOpen arg2")


def test_enum_member_from_island(tmp_path: Path) -> None:
    # PopShutdownSystem passed PowerDown (1) where the C says HalRebootRoutine (3).
    hits = _scan(
        tmp_path,
        "pub unsafe fn f() { HalReturnToFirmware(1); }\n",
        "HalReturnToFirmware(HalRebootRoutine);",
    )
    assert hits == ["MISMATCH HalReturnToFirmware arg0: 1 where C passes HalRebootRoutine=[3]"]


def test_enum_member_right_value_is_quiet(tmp_path: Path) -> None:
    assert not _scan(
        tmp_path,
        "pub unsafe fn f() { HalReturnToFirmware(3); }\n",
        "HalReturnToFirmware(HalRebootRoutine);",
    )


def test_island_outranks_an_invented_crate_const(tmp_path: Path) -> None:
    # The crate's own copy says 2; the header (island) says 1, and 1 is what was passed.
    assert not _scan(
        tmp_path,
        "const MmFlushForWrite: u32 = 2;\npub unsafe fn f() { MmFlushImageSection(s, 1); }\n",
        "MmFlushImageSection(Section, MmFlushForWrite);",
    )


def test_camel_case_variable_is_not_a_constant(tmp_path: Path) -> None:
    assert not _scan(
        tmp_path,
        "pub unsafe fn f() { Frob(0); }\n",
        "Frob(SomeLocalVariable);",
    )
