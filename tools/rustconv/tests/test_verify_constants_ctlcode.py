"""CTL_CODE (IOCTL_*/FSCTL_*) evaluation in build_c_catalog.

Device-control codes are #defined via the CTL_CODE() macro, not as plain
literals, so before this they never entered the catalog and a garbage-translated
Rust copy went unchecked — the class that cost the arm64 ramdisk 0x64/0x23 grind
(FSCTL_CREATE_RAM_DISK = 0x900c4 vs real 0x240000; IOCTL_DISK_CHECK_VERIFY =
0x70000 colliding with GET_DRIVE_GEOMETRY vs real 0x74800). These pin the macro
math and the parser pieces so neither silently regresses.
"""

from pathlib import Path
from typing import ClassVar

from rustconv.config import REACTOS_ROOT
from rustconv.verify_constants import (
    _eval_int_expr,
    _split_top_commas,
    build_c_catalog,
    fix_const_line,
    parse_ctl_code_defines,
    verify_file,
)

# CTL_CODE(DeviceType, Function, Method, Access)
#   = (DeviceType << 16) | (Access << 14) | (Function << 2) | Method
SCOPE = {
    "FILE_DEVICE_DISK": 0x07,
    "FILE_DEVICE_CD_ROM": 0x02,
    "FILE_DEVICE_VIRTUAL_DISK": 0x24,
    "FILE_DEVICE_FILE_SYSTEM": 0x09,
    "IOCTL_DISK_BASE": 0x07,
    "METHOD_BUFFERED": 0,
    "FILE_ANY_ACCESS": 0,
    "FILE_READ_ACCESS": 1,
    "FILE_WRITE_ACCESS": 2,
}


class TestEvalIntExpr:
    def test_literals_and_names(self):
        assert _eval_int_expr("0x0200", SCOPE) == 0x200
        assert _eval_int_expr("FILE_DEVICE_DISK", SCOPE) == 7
        assert _eval_int_expr("0", SCOPE) == 0

    def test_operators_and_parens(self):
        assert _eval_int_expr("FILE_READ_ACCESS | FILE_WRITE_ACCESS", SCOPE) == 3
        assert _eval_int_expr("(7 << 16)", SCOPE) == 0x70000

    def test_unresolved_name_bails(self):
        assert _eval_int_expr("NOT_A_REAL_NAME", SCOPE) is None

    def test_unrecognized_token_bails(self):
        # `*` is not in the token set — must not silently drop it and fabricate.
        assert _eval_int_expr("7 * 2", SCOPE) is None


class TestSplitTopCommas:
    def test_respects_paren_depth(self):
        assert _split_top_commas("a, b, (c, d), e") == ["a", "b", "(c, d)", "e"]


class TestCtlCodeParser:
    def test_evaluates_macro(self):
        acc: dict[str, set] = {}
        text = (
            "#define IOCTL_DISK_CHECK_VERIFY "
            "CTL_CODE(IOCTL_DISK_BASE, 0x0200, METHOD_BUFFERED, FILE_READ_ACCESS)\n"
            "#define FSCTL_CREATE_RAM_DISK "
            "CTL_CODE(FILE_DEVICE_VIRTUAL_DISK, 0x0000, METHOD_BUFFERED, FILE_ANY_ACCESS)\n"
        )
        parse_ctl_code_defines(text, SCOPE, acc)
        assert acc["IOCTL_DISK_CHECK_VERIFY"] == {0x74800}
        assert acc["FSCTL_CREATE_RAM_DISK"] == {0x240000}

    def test_unresolved_operand_skips_def(self):
        acc: dict[str, set] = {}
        parse_ctl_code_defines(
            "#define IOCTL_X CTL_CODE(MYSTERY_DEVICE, 0, METHOD_BUFFERED, FILE_ANY_ACCESS)\n",
            SCOPE,
            acc,
        )
        assert "IOCTL_X" not in acc  # never fabricate from an unresolved operand


class TestRealCatalog:
    """Integration: the real ReactOS headers must yield the known-good values.
    These are exactly the constants the ramdisk fix corrected — a regression in
    the evaluator (or a header move) trips here, not at a runtime BUGCHECK."""

    EXPECT: ClassVar[dict[str, int]] = {
        "FSCTL_CREATE_RAM_DISK": 0x240000,
        "IOCTL_DISK_CHECK_VERIFY": 0x74800,
        "IOCTL_DISK_GET_DRIVE_GEOMETRY": 0x70000,
        "IOCTL_DISK_GET_DRIVE_LAYOUT": 0x7400C,
        "IOCTL_CDROM_CHECK_VERIFY": 0x24800,
        "FSCTL_OPLOCK_BREAK_ACKNOWLEDGE": 0x9000C,
        "FSCTL_OPLOCK_BREAK_NOTIFY": 0x90014,
    }

    def test_known_values(self):
        cat = build_c_catalog(REACTOS_ROOT)
        for name, val in self.EXPECT.items():
            assert cat.get(name) == val, f"{name}: {cat.get(name)!r} != {val:#x}"

    def test_at_least_a_hundred_codes(self):
        cat = build_c_catalog(REACTOS_ROOT)
        codes = [k for k in cat if k.startswith(("IOCTL_", "FSCTL_"))]
        assert len(codes) > 100  # ~326 today; guards against the pass silently no-op'ing


class TestFixConstLine:
    def test_fixes_module_level(self):
        out = fix_const_line("const IOCTL_X: u32 = 0x00000000;", "IOCTL_X", 0x24064)
        assert out == "const IOCTL_X: u32 = 0x00024064;"

    def test_fixes_indented_function_local(self):
        # parse_rust_const strips the line so it DETECTS local consts; fix_const_line
        # must preserve the indentation and actually rewrite (was a silent no-op).
        out = fix_const_line("    const IOCTL_X: u32 = 0x0004D054;", "IOCTL_X", 0x4D048)
        assert out == "    const IOCTL_X: u32 = 0x0004D048;"

    def test_fix_via_verify_file(self, tmp_path: Path):
        cat = build_c_catalog(REACTOS_ROOT)
        rs = tmp_path / "drv.rs"
        rs.write_text("fn f() {\n    const FSCTL_CREATE_RAM_DISK: u32 = 0x000900C4;\n}\n")
        assert verify_file(rs, cat, fix=True)  # one mismatch, fixed
        assert "0x00240000" in rs.read_text()
        assert verify_file(rs, cat) == []  # clean on re-scan


class TestVerifyFileFlagsGarbageIoctl:
    def test_flags_wrong_and_passes_right(self, tmp_path: Path):
        cat = build_c_catalog(REACTOS_ROOT)
        rs = tmp_path / "drv.rs"
        rs.write_text(
            "const FSCTL_CREATE_RAM_DISK: u32 = 0x000900C4;\n"  # garbage (real 0x240000)
            "const IOCTL_DISK_CHECK_VERIFY: u32 = 0x00074800;\n"  # correct
        )
        ms = verify_file(rs, cat)
        names = {m["name"] for m in ms}
        assert "FSCTL_CREATE_RAM_DISK" in names
        assert "IOCTL_DISK_CHECK_VERIFY" not in names
