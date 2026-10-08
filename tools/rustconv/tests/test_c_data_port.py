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
