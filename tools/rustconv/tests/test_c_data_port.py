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
