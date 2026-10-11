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
    _BUILTIN_FUNC_MACROS,
    _eval_int_expr,
    _parse_func_macros,
    _split_top_commas,
    build_c_catalog,
    fix_const_line,
    parse_ioctl_defines,
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
        parse_ioctl_defines(text, SCOPE, dict(_BUILTIN_FUNC_MACROS), acc)
        assert acc["IOCTL_DISK_CHECK_VERIFY"] == {0x74800}
        assert acc["FSCTL_CREATE_RAM_DISK"] == {0x240000}

    def test_unresolved_operand_skips_def(self):
        acc: dict[str, set] = {}
        parse_ioctl_defines(
            "#define IOCTL_X CTL_CODE(MYSTERY_DEVICE, 0, METHOD_BUFFERED, FILE_ANY_ACCESS)\n",
            SCOPE,
            dict(_BUILTIN_FUNC_MACROS),
            acc,
        )
        assert "IOCTL_X" not in acc  # never fabricate from an unresolved operand

    def test_plain_literal_ioctl_subsumed(self):
        # The IOCTL_/FSCTL_ namespace parser also catches plain-literal defines.
        acc: dict[str, set] = {}
        parse_ioctl_defines(
            "#define IOCTL_FOO_BAR 0x1234\n", SCOPE, dict(_BUILTIN_FUNC_MACROS), acc
        )
        assert acc["IOCTL_FOO_BAR"] == {0x1234}

    def test_ignores_non_ioctl_namespace(self):
        acc: dict[str, set] = {}
        parse_ioctl_defines("#define SOME_FLAG 0x1\n", SCOPE, dict(_BUILTIN_FUNC_MACROS), acc)
        assert acc == {}  # only IOCTL_/FSCTL_ names are evaluated


class TestFuncMacroExpander:
    """The generic function-like-macro expander — the robustness upgrade that
    lets ANY CTL_CODE-family wrapper (not just CTL_CODE) evaluate through one
    path. AFD's wrapper doesn't even use CTL_CODE; it has its own bit-packing."""

    AFD: ClassVar[dict] = {
        "_AFD_CONTROL_CODE": (
            ["Operation", "Method"],
            "((FSCTL_AFD_BASE)<<12 | (Operation<<2) | Method)",
        ),
    }

    def test_parse_func_macro(self):
        macros = _parse_func_macros(
            "#define _TDI_CONTROL_CODE(request,method)   "
            "CTL_CODE(FILE_DEVICE_TRANSPORT, request, method, FILE_ANY_ACCESS)\n"
        )
        assert macros["_TDI_CONTROL_CODE"][0] == ["request", "method"]
        assert "CTL_CODE" in macros["_TDI_CONTROL_CODE"][1]

    def test_object_macro_not_captured_as_func(self):
        # A space before '(' makes it object-like (value), not function-like.
        assert _parse_func_macros("#define FOO (1 << 2)\n") == {}

    def test_afd_wrapper_evaluates_without_ctl_code(self):
        scope = {"FSCTL_AFD_BASE": 0x12, "METHOD_NEITHER": 3, "AFD_BIND": 0}
        # IOCTL_AFD_BIND = _AFD_CONTROL_CODE(AFD_BIND, METHOD_NEITHER) = 0x12003
        assert (
            _eval_int_expr("_AFD_CONTROL_CODE(AFD_BIND, METHOD_NEITHER)", scope, self.AFD)
            == 0x12003
        )

    def test_nested_wrapper_expands_to_ctl_code(self):
        # _TDI_CONTROL_CODE → CTL_CODE → arithmetic, two levels deep.
        macros = dict(_BUILTIN_FUNC_MACROS)
        macros["_TDI_CONTROL_CODE"] = (
            ["request", "method"],
            "CTL_CODE(FILE_DEVICE_TRANSPORT, request, method, FILE_ANY_ACCESS)",
        )
        scope = {"FILE_DEVICE_TRANSPORT": 0x21, "METHOD_BUFFERED": 0, "FILE_ANY_ACCESS": 0}
        # IOCTL_TDI_CONNECT = _TDI_CONTROL_CODE(1, METHOD_BUFFERED) = 0x210004
        assert _eval_int_expr("_TDI_CONTROL_CODE(1, METHOD_BUFFERED)", scope, macros) == 0x210004

    def test_param_substitution_is_whole_word(self):
        # param `Method` must NOT match inside `METHOD_NEITHER`.
        scope = {"FSCTL_AFD_BASE": 0x12, "METHOD_NEITHER": 3, "Op": 5}
        assert _eval_int_expr("_AFD_CONTROL_CODE(Op, METHOD_NEITHER)", scope, self.AFD) == (
            (0x12 << 12) | (5 << 2) | 3
        )

    def test_recursion_guard_bails_safely(self):
        # A self-referential macro must not hang — it bails, leaving an
        # unevaluable expression → None (never a fabricated value).
        loop = {"LOOP": (["x"], "LOOP(x)")}
        assert _eval_int_expr("LOOP(1)", {}, loop) is None


class TestRealCatalog:
    """Integration: the real ReactOS headers must yield the known-good values.
    These are exactly the constants the ramdisk fix corrected — a regression in
    the evaluator (or a header move) trips here, not at a runtime BUGCHECK."""

    EXPECT: ClassVar[dict[str, int]] = {
        # CTL_CODE-direct (the original ramdisk-fix constants — regression guard):
        "FSCTL_CREATE_RAM_DISK": 0x240000,
        "IOCTL_DISK_CHECK_VERIFY": 0x74800,
        "IOCTL_DISK_GET_DRIVE_GEOMETRY": 0x70000,
        "IOCTL_DISK_GET_DRIVE_LAYOUT": 0x7400C,
        "IOCTL_CDROM_CHECK_VERIFY": 0x24800,
        "FSCTL_OPLOCK_BREAK_ACKNOWLEDGE": 0x9000C,
        "FSCTL_OPLOCK_BREAK_NOTIFY": 0x90014,
        # Wrapper-macro families (the robustness upgrade). AFD uses its own
        # bit-packing (no CTL_CODE); TDI/NDIS wrap CTL_CODE. IOCTL_AFD_SEND is
        # the value netstack cited as the one their afd bug #3 had scrambled.
        "IOCTL_AFD_BIND": 0x12003,
        "IOCTL_AFD_SEND": 0x1201F,
        "IOCTL_AFD_RECV": 0x12017,
        "IOCTL_TDI_CONNECT": 0x210004,
        "IOCTL_TDI_SEND": 0x21001D,
        "IOCTL_NDIS_QUERY_GLOBAL_STATS": 0x170002,
        "IOCTL_VOLSNAP_FLUSH_AND_HOLD_WRITES": 0x53C000,  # ntifs.template.h
    }

    def test_known_values(self):
        cat = build_c_catalog(REACTOS_ROOT)
        for name, val in self.EXPECT.items():
            assert cat.get(name) == val, f"{name}: {cat.get(name)!r} != {val:#x}"

    def test_at_least_a_hundred_codes(self):
        cat = build_c_catalog(REACTOS_ROOT)
        codes = [k for k in cat if k.startswith(("IOCTL_", "FSCTL_"))]
        assert len(codes) > 100  # ~390 today (incl. AFD/TDI/NDIS/USB families)


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


class TestReactosDowngrade:
    def test_nx_names_take_the_value_the_tree_compiles(self, tmp_path: Path):
        # iotypes.h says 0x200, but every in-tree consumer zeroes it under __REACTOS__.
        cat = build_c_catalog(REACTOS_ROOT)
        assert cat["POOL_NX_ALLOCATION"] == 0
        assert cat["MdlMappingNoExecute"] == 0
        rs = tmp_path / "drv.rs"
        rs.write_text("pub const POOL_NX_ALLOCATION: u32 = 0x0200;\n")
        assert {m["name"] for m in verify_file(rs, cat)} == {"POOL_NX_ALLOCATION"}
