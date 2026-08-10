"""Controls for the string-literal multiset oracle.

The scanner exists because a translated literal can be REAL, correctly spelled, and in the
WRONG SLOT — the shape no compiler, sealed-constant envelope or boot can see. So the first
thing it must do is catch exactly that, and the second is stay quiet about the five ways a
faithful translation legitimately SPELLS a literal differently. A checker that fires on
those is worse than none: it trains you to ignore it.

Positive control is the real defect (field manual, the sealing/RECALL entry): undname.c's
`case 'C': "pascal"` shipped as `"__stdcall"`, a genuine member of the same switch.
"""

from __future__ import annotations

import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))

from verify_translation_literals import _dewiden, check_pair, strip_untaken_arms


def _pair(tmp_path: Path, c_src: str, rs_src: str) -> list[str]:
    c = tmp_path / "mod.c"
    rs = tmp_path / "mod.rs"
    c.write_text(c_src)
    rs.write_text(rs_src)
    return check_pair(rs, c)


# ── the positive control: a permutation, which leaves the SET identical ────────────────
PERMUTED_C = """
static void f(char ch) {
    switch (ch) {
    case 'A': conv = "cdecl";    break;
    case 'C': conv = "pascal";   break;
    case 'G': conv = "stdcall";  break;
    }
}
"""
PERMUTED_RS = """
unsafe fn f(ch: u8) {
    match ch {
        b'A' => conv = b"cdecl\\0".as_ptr(),
        b'C' => conv = b"stdcall\\0".as_ptr(),
        b'G' => conv = b"stdcall\\0".as_ptr(),
        _ => {}
    }
}
"""


def test_a_permuted_literal_is_caught(tmp_path):
    f = _pair(tmp_path, PERMUTED_C, PERMUTED_RS)
    assert any("MISSING" in x and "pascal" in x for x in f), f
    assert any("EXTRA" in x and "stdcall" in x for x in f), f


def test_the_same_literals_are_clean(tmp_path):
    good = PERMUTED_RS.replace(
        r'b"stdcall\0".as_ptr(),' + "\n        b'G'", r'b"pascal\0".as_ptr(),' + "\n        b'G'"
    )
    assert _pair(tmp_path, PERMUTED_C, good) == []


def test_a_count_change_is_caught_even_when_the_set_is_unchanged(tmp_path):
    # The reason this is a MULTISET: both sides use {"x"}, but the C uses it twice.
    f = _pair(tmp_path, 'char *a = "x"; char *b = "x";', r'let a = b"x\0"; let b2 = b"y\0";')
    assert any("MISSING" in x and "'x'" in x for x in f), f


# ── negative controls: the five legitimate spelling differences ────────────────────────
def test_debug_channel_formats_are_not_findings(tmp_path):
    # Every translated msvcrt module lowers TRACE/WARN/ERR to no-op macro_rules!, which
    # drops the format string. Uniform convention, not a per-file accident.
    assert _pair(tmp_path, 'ERR("Unknown type %c\\n", dt);', "ERR!(dt);") == []


def test_a_wide_literal_matches_its_interleaved_rust(tmp_path):
    # L"COMSPEC" vs b"C\0O\0M\0S\0P\0E\0C\0\0\0" — the same string.
    assert _pair(tmp_path, 'x = L"COMSPEC";', r'x = b"C\0O\0M\0S\0P\0E\0C\0\0\0";') == []


def test_nul_spelling_does_not_matter(tmp_path):
    assert _pair(tmp_path, 'x = "hi";', r'x = b"hi\x00";') == []


def test_an_untaken_c_arm_is_not_a_finding(tmp_path):
    # asctime's "%s %s %2d ..." is the _MSVCR_VER>=140 arm; the shipping one is %02d.
    # A checker blind to #if sends you hunting a bug that is not there.
    c = """
#if _MSVCR_VER>=140
    _snprintf(buf, 26, "%2d");
#else
    _snprintf(buf, 26, "%02d");
#endif
"""
    assert _pair(tmp_path, c, r'_snprintf(buf, 26, b"%02d\0");') == []


def test_a_cfg_any_rust_item_is_not_a_finding(tmp_path):
    # The repo's spelling for "this C block is not compiled here" — inert on both sides.
    rs = """
match x {
    #[cfg(any())] // `#ifdef _UCRT`; note the semicolon in this comment
    b'G' => { f(b"&\\0"); }
    _ => {}
}
"""
    assert _pair(tmp_path, "int x;", rs) == []


def test_rust_vocabulary_is_not_translated_content(tmp_path):
    rs = """
#[cfg(target_arch = "x86")]
extern "C" fn f() { let _ = cfg!(target_arch = "arm"); }
"""
    assert _pair(tmp_path, "void f(void) {}", rs) == []


# ── the two traps that silently UN-skip content, so they get their own controls ────────
def test_a_loop_label_does_not_desynchronise_the_comment_scanner(tmp_path):
    # `'fail:` looks like the start of a char literal. A naive scanner runs to the next
    # quote and every `//` after it stops being recognised — which leaked a fragment of a
    # commented-out line as a finding.
    rs = """
'fail: loop {
    break 'fail;
}
// let s = "ghost";
"""
    assert _pair(tmp_path, "void f(void) {}", rs) == []


def test_strip_untaken_arms_keeps_undecidable_conditions(tmp_path):
    # Over-report rather than hide: an unrecognised condition keeps BOTH arms.
    src = '#if SOMETHING_UNKNOWN\n"a";\n#else\n"b";\n#endif\n'
    kept = strip_untaken_arms(src)
    assert '"a"' in kept and '"b"' in kept


def test_dewiden_leaves_a_narrow_literal_alone():
    assert _dewiden(r"hello\0") == "hello"
    assert _dewiden(r"h\0e\0l\0l\0o\0\0") == "hello"
    # Not an interleave — must not be mangled.
    assert _dewiden(r"a\0b") == r"a\0b"


# ── the four ways this tree spells a wide literal as an ARRAY ──────────────────────────
# Keying on the DECLARATION meant a new blind spot per spelling, and each one presented as
# a wall of confident MISSING findings against code that was present and correct. Detection
# is by shape now, so all four have to read the same.
def test_every_wide_array_spelling_is_seen(tmp_path):
    c = 'void f(void) { g(L"State"); }'
    for rs in (
        "static NAME: [u16; 6] = [b'S' as u16, b't' as u16, b'a' as u16,"
        " b't' as u16, b'e' as u16, 0];\nunsafe fn f() { g(NAME.as_ptr()); }",
        "const NAME: &[u16] = &[0x53, 0x74, 0x61, 0x74, 0x65, 0x0000];\n"
        "unsafe fn f() { g(NAME.as_ptr()); }",
        "unsafe fn f() { let n: [u16; 6] = [0x53, 0x74, 0x61, 0x74, 0x65, 0]; g(n.as_ptr()); }",
        "unsafe fn f() { let n = &[0x53u16, 0x74, 0x61, 0x74, 0x65, 0]; g(n.as_ptr()); }",
        "unsafe fn f() { g(['S' as u16, 't' as u16, 'a' as u16,"
        " 't' as u16, 'e' as u16, 0].as_ptr()); }",
    ):
        assert _pair(tmp_path, c, rs) == [], rs


def test_a_zero_padded_array_is_not_read_as_truncated(tmp_path):
    """`[u16; 8]` holding 5 characters and three NULs is the whole string, not the first one."""
    rs = "static NAME: [u16; 8] = [0x53, 0x74, 0x61, 0x74, 0x65, 0, 0, 0];\nunsafe fn f() {}"
    assert _pair(tmp_path, 'void f(void) { g(L"State"); }', rs) == []


def test_a_single_character_wide_array_is_seen(tmp_path):
    """`[c, 0]` is a one-character string; userenv spells the path separator that way."""
    rs = "static SEP: [u16; 2] = [b'\\\\' as u16, 0];\nunsafe fn f() {}"
    assert _pair(tmp_path, 'void f(void) { g(L"\\\\"); }', rs) == []


def test_a_numeric_table_is_not_mistaken_for_a_string(tmp_path):
    """No NUL terminator and not text — an ordinary table must contribute no literals."""
    rs = "static T: [u16; 4] = [1000, 2000, 3000, 4000];\nunsafe fn f() {}"
    assert _pair(tmp_path, "void f(void) {}", rs) == []


def test_the_double_interleaved_shape_is_refused(tmp_path):
    """wkssvc's real defect: `&[0x43,0x00,0x68,0x00,...]` as u16 elements reads as just 'C'.

    The array must NOT be credited with the string it looks like in hex, or the checker
    would certify the exact bug it exists to find — every registry value name in
    SaveWorkstationInfo truncated to its first character.
    """
    rs = (
        "unsafe fn f() { let n: &[u16] = &[0x43,0x00,0x68,0x00,0x61,0x00,"
        "0x72,0x00,0x00,0x00]; g(n.as_ptr()); }"
    )
    out = _pair(tmp_path, 'void f(void) { g(L"Char"); }', rs)
    assert any("MISSING" in f and "Char" in f for f in out), out


# ── the spellings a faithful translation legitimately differs on ───────────────────────
def test_adjacent_c_literals_are_concatenated(tmp_path):
    """C joins adjacent literals; the `L` prefix of the next one sits in the gap."""
    c = 'void f(void) { g(L"abc " \n L"def" \n L"\\n"); }'
    assert _pair(tmp_path, c, 'unsafe fn f() { g(b"abc def\\n\\0".as_ptr()); }') == []


def test_an_escaped_backslash_before_zero_is_not_a_terminator(tmp_path):
    r"""`...CentralProcessor\\0` ends in backslash + the DIGIT zero, not a NUL escape."""
    c = 'void f(void) { g(L"HARDWARE\\\\CentralProcessor\\\\0"); }'
    rs = 'unsafe fn f() { g(w!("HARDWARE\\\\CentralProcessor\\\\0")); }'
    assert _pair(tmp_path, c, rs) == []


def test_a_raw_control_character_matches_its_escape(tmp_path):
    """sysdm embedded a literal TAB where the C spells `\\t`; identical at runtime."""
    assert _pair(tmp_path, 'void f(void) { g(L"a\\tb"); }', 'unsafe fn f() { g("a\tb"); }') == []


def test_an_embedded_nul_is_stripped_on_both_sides(tmp_path):
    """netcfgx's `L"0.0.0.0\\0"` against a Rust `w!("0.0.0.0\\0")` meaning the same thing."""
    c = 'void f(void) { g(L"0.0.0.0\\0"); }'
    assert _pair(tmp_path, c, 'unsafe fn f() { g(w!("0.0.0.0\\0")); }') == []


def test_an_if_zero_arm_is_not_a_finding(tmp_path):
    """`#if 0` is how C parks a block; umpnpmgr parks an RPC endpoint that way."""
    c = 'void f(void) {\n#if 0\n  g(L"parked");\n#endif\n  g(L"live");\n}'
    assert _pair(tmp_path, c, 'unsafe fn f() { g(w!("live")); }') == []


def test_a_hoisted_constant_is_compared_by_presence_not_count(tmp_path):
    """The Rust names a literal once however many times the C spells it."""
    c = 'void f(void) { g(L"Path"); h(L"Path"); i(L"Path"); }'
    rs = (
        "static P: [u16; 5] = [b'P' as u16, b'a' as u16, b't' as u16, b'h' as u16, 0];\n"
        "unsafe fn f() { g(P.as_ptr()); h(P.as_ptr()); i(P.as_ptr()); }"
    )
    assert _pair(tmp_path, c, rs) == []


def test_reactos_dprint_formats_are_not_findings(tmp_path):
    """DPRINT is ReactOS's channel; scan_dropped_prints owns the dropped-diagnostic class."""
    c = 'void f(void) { DPRINT1("failed %lu\\n", e); g(L"real"); }'
    assert _pair(tmp_path, c, 'unsafe fn f() { g(w!("real")); }') == []
