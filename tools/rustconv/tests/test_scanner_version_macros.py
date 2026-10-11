"""scanner: `#if (NTDDI_VERSION >= NTDDI_VISTA)` is decided at the module's profile.

Arbiter library, 2026-10-11.

The arbiter library splits signatures and whole functions on NTDDI_VERSION. With a macro on
both sides the condition was undecidable, the Vista-only definition read as "lost", and the
fallback scan parsed the Vista arms: 22 hand fixes after the paid run.
"""

from pathlib import Path

from rustconv.dead_regions import eval_int_guard
from rustconv.scanner import build_int_defines, scan_file

SDKDDKVER = (
    "#define _WIN32_WINNT_WS03 0x0502\n"
    "#define NTDDI_WS03SP1 0x05020100\n"
    "#define NTDDI_WIN6 0x06000000\n"
    "#define NTDDI_VISTA NTDDI_WIN6\n"
)

ARBITER = (
    "NTSTATUS\nNTAPI\n"
    "#if (NTDDI_VERSION >= NTDDI_VISTA)\n"
    "QueryArbitrate(PARBITER_INSTANCE Arbiter, PPARAMETERS Parameters)\n"
    "{\n    return 0;\n}\n"
    "#else\n"
    "QueryArbitrate(PARBITER_INSTANCE Arbiter, PLIST_ENTRY ArbitrationList)\n"
    "{\n    return 0;\n}\n"
    "#endif\n\n"
    "#if (NTDDI_VERSION >= NTDDI_VISTA)\n"
    "NTSTATUS NTAPI InitializeRangeList(PARBITER_INSTANCE Arbiter)\n{\n    return 0;\n}\n"
    "#endif\n\n"
    "VOID NTAPI Other(VOID)\n{\n}\n"
)


def _module(tmp_path: Path, cmake: str = "") -> Path:
    (tmp_path / "sdk" / "include" / "psdk").mkdir(parents=True)
    (tmp_path / "sdk" / "include" / "psdk" / "sdkddkver.h").write_text(SDKDDKVER)
    mod = tmp_path / "sdk" / "lib" / "arbiter"
    mod.mkdir(parents=True)
    (mod / "CMakeLists.txt").write_text(cmake)
    c = mod / "handler.c"
    c.write_text(ARBITER)
    return c


def test_the_pre_vista_arm_is_the_one_scanned(tmp_path: Path):
    c = _module(tmp_path)
    fns = {f.name: f for f in scan_file(c).functions}
    assert set(fns) == {"QueryArbitrate", "Other"}
    assert fns["QueryArbitrate"].params == [
        ("PARBITER_INSTANCE", "Arbiter"),
        ("PLIST_ENTRY", "ArbitrationList"),
    ]


def test_a_module_override_moves_the_profile(tmp_path: Path):
    c = _module(tmp_path, "add_definitions(-D_WIN32_WINNT=0x600)\n")
    assert int(build_int_defines(c)["NTDDI_VERSION"], 0) == 0x06000000
    fns = {f.name: f for f in scan_file(c).functions}
    assert {"QueryArbitrate", "InitializeRangeList", "Other"} == set(fns)
    assert fns["QueryArbitrate"].params[1] == ("PPARAMETERS", "Parameters")


def test_macro_on_both_sides_decides_only_when_both_are_pinned():
    pins = {"NTDDI_VERSION": 0x05020100, "NTDDI_VISTA": 0x06000000}
    assert eval_int_guard("(NTDDI_VERSION >= NTDDI_VISTA)", pins) is False
    assert eval_int_guard("(NTDDI_VERSION < NTDDI_VISTA)", pins) is True
    assert eval_int_guard("(NTDDI_VERSION >= NTDDI_WIN10)", pins) is None
    assert eval_int_guard("_MSVCR_VER >= 80L", {"_MSVCR_VER": 0}) is False
