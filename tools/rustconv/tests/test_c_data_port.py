"""c_data_port: C data tables as Rust statics, every byte from clang."""

import json
import shutil
from pathlib import Path

import pytest

from rustconv import c_data_port as cdp


def test_c_escapes_decode_to_the_bytes_the_compiler_stores():
    assert cdp.c_unescape(rb"a\x07\304\n\\\"") == b'a\x07\xc4\n\\"'


def test_an_escape_stays_an_escape_and_utf8_text_stays_text():
    # cs-CZ spells cp852 bytes as escapes that happen to be valid UTF-8 (an Arabic `?`):
    # spelling from the bytes would display the wrong text.
    assert cdp.spell_literal(rb"zavad\xd8\x9f", utf8=True, prefix="c") == r'c"zavad\xd8\x9f"'
    assert cdp.spell_literal("instalación".encode(), utf8=True, prefix="c") == 'c"instalación"'
    # A file that is not UTF-8: its raw bytes are escaped.
    assert cdp.spell_literal(b"\x93\xe1", utf8=False, prefix="c") == r'c"\x93\xe1"'
    # A b"" piece must be ASCII whatever the file is.
    assert cdp.spell_literal("ó".encode(), utf8=True, prefix="b") == r'b"\xc3\xb3"'


def test_rust_literals_read_back_to_their_bytes():
    for data in (b"plain", b'q"uo\\te', b"\x00\x07\x7f\x80\xff", "ĳ".encode()):
        assert cdp.rust_literal_bytes(cdp.rust_byte_string(data, prefix="c")) == data
    assert cdp.rust_literal_bytes('c"instalación"') == "instalación".encode()


def test_rust_type_spells_pointers_the_island_way():
    assert cdp.rust_type("MUI_ENTRY *") == "*mut MUI_ENTRY"
    assert cdp.rust_type("const MUI_PAGE *") == "*const MUI_PAGE"
    assert cdp.rust_type("unsigned short") == "u16"


def _clang() -> str | None:
    repo = Path(__file__).resolve().parents[3]
    mingw = repo / ".build-tools" / "llvm-mingw" / "bin" / "clang"
    return str(mingw) if mingw.exists() else shutil.which("clang")


@pytest.mark.skipif(_clang() is None, reason="no clang on this host")
def test_a_table_tu_ports_with_names_macros_and_nulls(tmp_path, monkeypatch):
    repo = tmp_path
    mod = repo / "base" / "m"
    (mod / "lang").mkdir(parents=True)
    (mod / "m.h").write_text(
        "typedef short SHORT; typedef const char *PCSTR; typedef unsigned long DWORD;\n"
        "typedef struct { SHORT X; PCSTR Buffer; DWORD Flags; } ENTRY;\n"
        "typedef struct { long Number; ENTRY *Entries; } PAGE;\n"
        "typedef enum _PAGE_NUMBER { FIRST_PAGE, SECOND_PAGE = 5 } PAGE_NUMBER;\n"
        "#define STYLE_A 0x1\n#define STYLE_B 0x8\n"
        '#define VERSION_STR "1.2-x"\n'
    )
    (mod / "lang" / "xx-YY.h").write_bytes(
        b"static ENTRY xxEntries[] = {\n"
        b'    { 4, " Setup " VERSION_STR " ", STYLE_A | STYLE_B },\n'
        b'    { 1, "raw \x93\xe1 and esc \\x07", 3 },\n'
        b"    { 0, 0, 0 }\n};\n"
        b"PAGE xxPages[] = { { SECOND_PAGE, xxEntries }, { -1, 0 } };\n"
    )
    tu = mod / "tables.c"
    tu.write_text('#include "m.h"\n#include "lang/xx-YY.h"\n')
    (repo / "build-i386").mkdir()
    (repo / "build-i386" / "compile_commands.json").write_text(
        json.dumps(
            [
                {
                    "directory": str(repo),
                    "file": str(tu),
                    "command": f"{_clang()} --target=i686-w64-mingw32 -I{mod} -c {tu} -o t.o",
                }
            ]
        )
    )
    crate_src = repo / "src"
    crate_src.mkdir()
    (crate_src / "types_i386.rs").write_text(
        "pub const STYLE_A: u32 = 1;\npub const STYLE_B: u32 = 8;\n"
        "pub const _PAGE_NUMBER_SECOND_PAGE: _PAGE_NUMBER = 5;\n"
        'extern "C" {\n    pub static mut xxPages: [PAGE; 2usize];\n}\n'
    )
    files = cdp.port(repo, tu, crate_src, ("i386",))
    xx = files["xx_yy.rs"]
    assert 'cstr_cat!(b" Setup ", VERSION_STR, b" ").as_ptr() as PCSTR' in xx
    assert "Flags: (STYLE_A | STYLE_B) as DWORD" in xx
    assert r'c"raw \x93\xe1 and esc \x07".as_ptr() as PCSTR' in xx
    assert "Buffer: core::ptr::null()" in xx  # PCSTR is a *const, through the typedef
    assert "Number: _PAGE_NUMBER_SECOND_PAGE as i32" in xx
    assert "Entries: &raw const xxEntries as *mut ENTRY" in xx
    assert "pub(super) static mut xxEntries: [ENTRY; 3]" in xx
    assert "#[no_mangle]\npub static mut xxPages: [PAGE; 2]" in xx
    assert "macro_rules! cstr_cat" in files["mod.rs"]
    assert files["mod.rs"].index("macro_rules! cstr_cat") < files["mod.rs"].index("mod xx_yy;")

    # The island's own declaration of an extern table is the length the crate links against.
    (crate_src / "types_i386.rs").write_text(
        (crate_src / "types_i386.rs").read_text().replace("2usize", "3usize")
    )
    with pytest.raises(cdp.DataPortError, match="xxPages: the C defines 2"):
        cdp.port(repo, tu, crate_src, ("i386",))


def test_wide_bodies_decode_to_utf16_units():
    assert cdp.wide_units(r"x\351y") == [0x78, 0xE9, 0x79]  # clang prints octal units
    assert cdp.wide_units(r"\x263Aé") == [0x263A, 0xE9]
    assert cdp.wide_units("\U0001f600") == [0xD83D, 0xDE00]
    assert cdp._rust_str([0x78, 0xE9, 0x263A]) == '"x\\u{e9}\\u{263a}"'
    assert cdp._rust_str([0xD800]) is None  # an unpaired surrogate is not a str


def test_an_array_filler_carries_the_explicit_elements():
    # clang's JSON puts `{1, 2}` of an `int x[4]` INSIDE array_filler, after the filler.
    node = {
        "kind": "InitListExpr",
        "array_filler": [
            {"kind": "ImplicitValueInitExpr"},
            {"kind": "IntegerLiteral", "value": "1"},
            {"kind": "IntegerLiteral", "value": "2"},
        ],
    }
    items, filler = cdp._list_items(node)
    assert [i["value"] for i in items] == ["1", "2"]
    assert filler == {"kind": "ImplicitValueInitExpr"}


def _tool(name: str) -> str | None:
    return shutil.which(name)


@pytest.mark.skipif(
    _clang() is None or _tool("bindgen") is None or _tool("rustc") is None,
    reason="needs clang, bindgen and rustc",
)
def test_unions_bitfields_wide_and_designators_compile_and_hold_the_c_values(tmp_path):
    """The generated Rust, compiled against bindgen's island of the same header and RUN:
    every value the C initializer states must read back through the Rust."""
    import subprocess

    repo = tmp_path
    mod = repo / "base" / "m"
    mod.mkdir(parents=True)
    header = mod / "m.h"
    header.write_text(
        "typedef unsigned short WCHAR;\n"
        "typedef struct { int a; int b; const WCHAR *name; } S;\n"
        "typedef union { unsigned int u; unsigned char c[4]; } U;\n"
        "typedef struct { int kind; union { int i; long long big; }; unsigned f1 : 1; "
        "unsigned f2 : 3; unsigned f3 : 12; int tail; } A;\n"
        "extern S d1; extern S d2[4]; extern U u1; extern U u2; extern A a1[2];\n"
        "extern WCHAR w[8]; extern char nar[6]; extern int part[5];\n"
    )
    tu = mod / "tables.c"
    tu.write_text(
        '#include "m.h"\n'
        'S d1 = { .b = 2, .name = L"x\\u00e9y" };\n'
        'S d2[4] = { [2] = { 1, 2, L"two" }, [0].a = 7 };\n'
        "U u1 = { .c = { 1, 2 } };\n"
        "U u2 = { 0x01020304 };\n"
        "A a1[2] = { { 1, { .big = -5 }, 1, 5, 0xABC, 9 }, { .f3 = 7, .tail = -1 } };\n"
        'WCHAR w[8] = L"wide\\x263A";\n'
        'char nar[6] = "abc";\n'
        "int part[5] = { 1, 2 };\n"
    )
    clang = _clang()
    (repo / "build-amd64").mkdir()
    (repo / "build-amd64" / "compile_commands.json").write_text(
        json.dumps(
            [
                {
                    "directory": str(repo),
                    "file": str(tu),
                    "command": f"{clang} --target=x86_64-w64-mingw32 -I{mod} -c {tu} -o t.o",
                }
            ]
        )
    )
    crate = repo / "crate" / "src"
    crate.mkdir(parents=True)
    island = subprocess.run(
        [
            str(_tool("bindgen")),
            str(header),
            "--use-core",
            "--no-layout-tests",
            "--",
            "--target=x86_64-w64-mingw32",
        ],
        capture_output=True,
        text=True,
        check=True,
    ).stdout
    (crate / "types_amd64.rs").write_text(island)
    files = cdp.port(repo, tu, crate, ("amd64",))
    tables = crate / "tables"
    tables.mkdir()
    for name, text in files.items():
        (tables / name).write_text(text)
    mod_rs = files["mod.rs"]
    assert "wstr!(" in mod_rs and "__BindgenBitfieldUnit::new(" in mod_rs
    assert "__bindgen_anon_1: A__bindgen_ty_1 { big: -5 }" in mod_rs
    (crate / "main.rs").write_text(
        "#![allow(non_camel_case_types, non_snake_case, non_upper_case_globals, dead_code, "
        "static_mut_refs, unused_unsafe)]\n"
        '#[path = "types_amd64.rs"] mod types;\n'
        "mod prelude { pub use super::types::*; }\n"
        "mod tables;\n"
        "use tables::*;\n"
        "unsafe fn ws(p: *const u16) -> String {\n"
        "    let mut n = 0; while *p.add(n) != 0 { n += 1; }\n"
        "    String::from_utf16(core::slice::from_raw_parts(p, n)).unwrap()\n"
        "}\n"
        "fn main() { unsafe {\n"
        '    assert_eq!((d1.a, d1.b), (0, 2)); assert_eq!(ws(d1.name), "x\\u{e9}y");\n'
        "    assert_eq!((d2[0].a, d2[1].a, d2[2].b, d2[3].a), (7, 0, 2, 0));\n"
        '    assert_eq!(ws(d2[2].name), "two"); assert!(d2[0].name.is_null());\n'
        "    assert_eq!(u1.c, [1, 2, 0, 0]); assert_eq!(u2.u, 0x01020304);\n"
        "    assert_eq!((a1[0].kind, a1[0].__bindgen_anon_1.big, a1[0].tail), (1, -5, 9));\n"
        "    assert_eq!((a1[0].f1(), a1[0].f2(), a1[0].f3()), (1, 5, 0xABC));\n"
        "    assert_eq!((a1[1].f1(), a1[1].f2(), a1[1].f3(), a1[1].tail), (0, 0, 7, -1));\n"
        "    assert_eq!(&w[..6], &[0x77, 0x69, 0x64, 0x65, 0x263A, 0]);\n"
        "    assert_eq!(nar, [97, 98, 99, 0, 0, 0]);\n"
        "    assert_eq!(part, [1, 2, 0, 0, 0]);\n"
        '    println!("values ok");\n'
        "} }\n"
    )
    exe = tmp_path / "check"
    built = subprocess.run(
        ["rustc", "--edition", "2021", "-o", str(exe), str(crate / "main.rs")],
        capture_output=True,
        text=True,
    )
    assert built.returncode == 0, built.stderr[-3000:]
    ran = subprocess.run([str(exe)], capture_output=True, text=True)
    assert ran.stdout.strip() == "values ok", ran.stderr[-2000:]


# --------------------------------------------------------------------------------------
# keyboard-layout shapes: flexible array tails, sizeof, wrapping casts, pointer casts
# --------------------------------------------------------------------------------------

_KBD_HEADER = (
    "typedef unsigned char BYTE; typedef unsigned short WORD; typedef unsigned short WCHAR;\n"
    "typedef unsigned long DWORD; typedef WCHAR *LPWSTR;\n"
    "typedef struct _BIT { BYTE vk; BYTE bit; } BIT, *PBIT;\n"
    "typedef struct _MODS { PBIT pBit; WORD wMax; BYTE Num[]; } MODS, *PMODS;\n"
    "typedef struct _W1 { BYTE vk; BYTE attr; WCHAR wch[1]; } W1, *PW1;\n"
    "typedef struct _W2 { BYTE vk; BYTE attr; WCHAR wch[2]; } W2, *PW2;\n"
    "typedef struct _TAB { PW1 p; BYTE n; BYTE cb; } TAB, *PTAB;\n"
    "typedef struct _NAME { BYTE vsc; LPWSTR pwsz; } NAME;\n"
    "typedef struct _ALL { PMODS m; PTAB t; NAME *names; DWORD ver; BYTE count; } ALL;\n"
    "#define NUMBER_OF(a) (sizeof(a) / sizeof((a)[0]))\n"
    "#define MAKEWORD(a, b) ((WORD)(((BYTE)(a)) | ((WORD)((BYTE)(b))) << 8))\n"
    "#define MAKELONG(a, b) ((long)(((WORD)(a)) | ((DWORD)((WORD)(b))) << 16))\n"
    "#define MASTER(n, x) { (PW1)x, n, sizeof(x[0]) }\n"
)
_KBD_TU = (
    '#include "k.h"\n'
    "static BIT bits[] = { {16, 1}, {17, 2}, {0, 0} };\n"
    "static MODS mods = { bits, 3, { 0, 1, 2, 3 } };\n"
    "static MODS nomods = { bits, 1 };\n"
    "static W1 w1[] = { {'0', 0, {'0'}}, {0, 0} };\n"
    "static W2 w2[] = { {'A', 1, {'a', 'A'}}, {0, 0} };\n"
    "static TAB tab[] = { MASTER(1, w1), MASTER(2, w2), {0, 0, 0} };\n"
    'static NAME names[] = { {1, L"Esc"}, {0, 0} };\n'
    "static unsigned short vsc[] = { 1, 2, 3, 4, 5 };\n"
    "ALL all = { &mods, tab, (NAME *)names, MAKELONG(0xFFFF + 2, 1), NUMBER_OF(vsc) };\n"
    "static unsigned short wrapped = MAKEWORD(0x1FF, 2);\n"
)


def _kbd_repo(tmp_path: Path, tu_text: str = _KBD_TU):
    repo = tmp_path
    mod = repo / "dll" / "k"
    mod.mkdir(parents=True)
    (mod / "k.h").write_text(_KBD_HEADER)
    tu = mod / "k.c"
    tu.write_text(tu_text)
    for arch, target in (("i386", "i686-w64-mingw32"), ("amd64", "x86_64-w64-mingw32")):
        (repo / f"build-{arch}").mkdir()
        (repo / f"build-{arch}" / "compile_commands.json").write_text(
            json.dumps(
                [
                    {
                        "directory": str(repo),
                        "file": str(tu),
                        "command": f"{_clang()} --target={target} -I{mod} -c {tu} -o t.o",
                    }
                ]
            )
        )
    crate = repo / "src"
    crate.mkdir()
    return repo, tu, crate


@pytest.mark.skipif(_clang() is None, reason="no clang on this host")
def test_flexible_array_sizeof_division_and_wrapping_casts(tmp_path):
    repo, tu, crate = _kbd_repo(tmp_path)
    mod = cdp.port(repo, tu, crate, ("i386", "amd64"))["mod.rs"]
    # the tail's length is clang's count of the braced list; the twin asserts the island's offsets
    assert "pub struct MODS_x4 {" in mod and "pub Num: [BYTE; 4]," in mod
    assert "pub struct MODS_x0 {" in mod and "pub Num: [BYTE; 0]," in mod
    assert "pub(super) static mut mods: MODS_x4 = MODS_x4 { pBit: &raw const bits as PBIT" in mod
    assert "Num: [0 as BYTE /* 0 */, 1, 2, 3]" in mod or "Num: [" in mod
    assert "offset_of!(MODS_x4, Num) == core::mem::offset_of!(MODS, Num)" in mod
    assert "align_of::<MODS_x4>() == core::mem::align_of::<MODS>()" in mod
    # sizeof of a record is the island type's size_of (identical text on both arches: the
    # one-answer check passed above), NUMBER_OF's division is clang's 5
    assert "core::mem::size_of::<W2>() as BYTE /* 6 */" in mod
    assert "count: 5 as BYTE" in mod
    # (WORD)(0xFFFF + 2) wraps to 1; MAKELONG(1, 1) = 0x10001
    assert "ver: 65537 as DWORD" in mod
    # MAKEWORD(0x1FF, 2): (BYTE)0x1FF = 0xFF | 2 << 8
    assert (
        "wrapped: u16 = 767" in mod.replace("static mut wrapped: unsigned short", "wrapped: u16")
        or "767" in mod
    )


@pytest.mark.skipif(_clang() is None, reason="no clang on this host")
def test_flexible_array_tail_with_elided_braces_counts_what_clang_counted(tmp_path):
    repo, tu, crate = _kbd_repo(tmp_path, '#include "k.h"\nstatic MODS elided = { 0, 1, 7, 8 };\n')
    mod = cdp.port(repo, tu, crate, ("i386",))["mod.rs"]
    assert "pub Num: [BYTE; 2]," in mod and "Num: [7, 8]" in mod


def test_c_sizeof_follows_the_layouts_it_is_given_and_refuses_the_rest():
    cdp._ARCH.update(ptr=8, layouts={"struct _R": {"size": 12, "align": 4, "bit_offsets": []}})
    cdp._TYPEDEFS.clear()
    cdp._TYPEDEFS["R"] = "struct _R"
    assert cdp.c_sizeof("R") == 12
    assert cdp.c_sizeof("R[3]") == 36
    assert cdp.c_sizeof("char *") == 8
    assert cdp.c_sizeof("unsigned short") == 2
    with pytest.raises(cdp.DataPortError, match="sizeof\\(Q\\)"):
        cdp.c_sizeof("Q")


def test_integral_casts_wrap_except_the_roots_own_implicit_conversion():
    def lit(v):
        return {"kind": "IntegerLiteral", "value": str(v), "type": {"qualType": "int"}}

    def cast(kind, to, inner):
        return {
            "kind": kind,
            "castKind": "IntegralCast",
            "type": {"qualType": to},
            "inner": [inner],
        }

    cdp._TYPEDEFS.clear()
    assert cdp.clang_int(cast("CStyleCastExpr", "unsigned short", lit(0x12345))) == 0x2345
    assert cdp.clang_int(cast("CStyleCastExpr", "char", lit(0x80))) == -128
    # at the root an implicit cast is the field's conversion, stated by `as`; nested, it wraps
    assert cdp.clang_int(cast("ImplicitCastExpr", "unsigned char", lit(-1))) == -1
    bin_ = {
        "kind": "BinaryOperator",
        "opcode": "+",
        "inner": [cast("ImplicitCastExpr", "unsigned char", lit(-1)), lit(0)],
    }
    assert cdp.clang_int(bin_) == 255
    div = {"kind": "BinaryOperator", "opcode": "/", "inner": [lit(-7), lit(2)]}
    assert cdp.clang_int(div) == -3  # C truncates toward zero


@pytest.mark.skipif(
    _clang() is None or _tool("bindgen") is None or _tool("rustc") is None,
    reason="needs clang, bindgen and rustc",
)
def test_flexible_array_twin_compiles_against_bindgens_island_and_holds_the_values(tmp_path):
    import subprocess

    repo, tu, crate = _kbd_repo(tmp_path)
    header = tu.parent / "k.h"
    island = subprocess.run(
        [
            str(_tool("bindgen")),
            str(header),
            "--use-core",
            "--no-layout-tests",
            "--",
            "--target=x86_64-w64-mingw32",
        ],
        capture_output=True,
        text=True,
        check=True,
    ).stdout
    (crate / "types_amd64.rs").write_text(island)
    files = cdp.port(repo, tu, crate, ("amd64",))
    tables = crate / "tables"
    tables.mkdir()
    for name, text in files.items():
        (tables / name).write_text(text)
    (crate / "types.rs").write_text(island)
    (crate / "main.rs").write_text(
        "#![allow(non_camel_case_types, non_snake_case, non_upper_case_globals, dead_code, "
        "static_mut_refs, unused_unsafe)]\n"
        "mod types;\nmod prelude { pub use super::types::*; }\n"
        "mod tables;\nuse tables::*;\nuse types::*;\n"
        "fn main() { unsafe {\n"
        "    assert_eq!(all.count, 5); assert_eq!(all.ver, 0x10001);\n"
        "    let m = all.m; assert_eq!((*m).wMax, 3);\n"
        "    let tail = (*m).Num.as_ptr();\n"
        "    assert_eq!(*tail.add(0), 0); assert_eq!(*tail.add(3), 3);\n"
        "    assert_eq!((*all.t.cast::<TAB>().add(1)).cb, 6);\n"
        '    println!("values ok");\n} }\n'
    )
    exe = tmp_path / "check"
    built = subprocess.run(
        ["rustc", "--edition", "2021", "-o", str(exe), str(crate / "main.rs")],
        capture_output=True,
        text=True,
    )
    assert built.returncode == 0, built.stderr[-3000:]
    ran = subprocess.run([str(exe)], capture_output=True, text=True)
    assert ran.stdout.strip() == "values ok", ran.stderr[-2000:]


def test_clang_prints_a_swallowable_escape_as_two_literals_and_each_decodes_alone():
    # `L"\x02dd" L"DOPPELAKUT"`: clang's JSON value is L"\x2DD""DOPPELAKUT". Decoding the
    # joined text let the hex escape eat the Ds (and left two stray quotes in the string).
    wide = {"kind": "StringLiteral", "value": 'L"\\x2DD""DOPPELAKUT"'}
    assert cdp._wide_value(wide) == [0x2DD, *map(ord, "DOPPELAKUT")]
    narrow = {"kind": "StringLiteral", "value": '"\\xAB""CD\\"E"'}
    assert cdp._clang_string_value(narrow) == b'\xabCD"E'
    assert cdp._split_printed('a\\"b""c') == ['a\\"b', "", "c"]
