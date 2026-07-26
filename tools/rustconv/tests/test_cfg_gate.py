"""Carrying a C `#ifdef` build-configuration guard into the translated Rust.

Every case is drawn from wldap32 (2026-07-26), where a build with `/* #undef HAVE_LDAP */`
had the whole libldap arm translated and 1087 rustc errors to show for it. The two
regression tests at the bottom are for bugs this module SHIPPED and that failed silently:
byte-vs-char offsets, and a substring guard match.
"""

import pytest

from rustconv.cfg_gate import (
    BODY_GATED,
    COMPLEX,
    UNGATED,
    WHOLLY_GATED,
    CfgGateError,
    _count_ifdefs,
    _rust_fns,
    apply_file,
    classify_c,
)

GUARDS = frozenset({"HAVE_LDAP", "HAVE_LDAP_H", "HAVE_LDAP_PARSE_REFERENCE"})


def _c(tmp_path, text, name="t.c"):
    p = tmp_path / name
    p.write_text(text)
    return p


def _verdicts(tmp_path, text):
    return {v.name: v for v in classify_c(_c(tmp_path, text), GUARDS)}


# --------------------------------------------------------------- classification


def test_body_gated_is_the_common_shape(tmp_path):
    """150 of wldap32's 153 guarded bodies are `<decl> #ifdef ... #endif return <var>;`."""
    v = _verdicts(
        tmp_path,
        """
ULONG ldap_addA( void *ld )
{
    ULONG ret = WLDAP32_LDAP_NOT_SUPPORTED;
#ifdef HAVE_LDAP
    ret = ldap_add( ld );
#endif
    return ret;
}
""",
    )["ldap_addA"]
    assert v.kind == BODY_GATED
    assert v.ret_var == "ret"
    assert v.guard == "HAVE_LDAP"
    assert v.feature == "have_ldap"


def test_untouched_when_the_guard_is_absent(tmp_path):
    v = _verdicts(tmp_path, "ULONG f( void ) { return 1; }")["f"]
    assert v.kind == UNGATED


def test_file_scope_ifdef_makes_the_whole_definition_gated(tmp_path):
    """wldap32.h:244 puts 43 of its 62 marshalling helpers inside one `#ifdef`."""
    v = _verdicts(
        tmp_path,
        """
#ifdef HAVE_LDAP
static LDAPMod *modAtoW( LDAPModA *mod ) { return 0; }
#endif
""",
    )["modAtoW"]
    assert v.kind == WHOLLY_GATED
    assert v.guard == "HAVE_LDAP"


def test_else_arm_is_refused_because_the_else_is_what_ships(tmp_path):
    """`#else` means the OTHER arm is the one this build compiles. Gating the translated
    arm would ship the wrong half, so refuse rather than guess."""
    v = _verdicts(
        tmp_path,
        """
ULONG ber_peek_tag( void *b )
{
#ifdef HAVE_LDAP
    return ber_peek_tag_real( b );
#else
    return LBER_ERROR;
#endif
}
""",
    )["ber_peek_tag"]
    assert v.kind == COMPLEX
    assert "else" in v.reason


def test_guard_nested_in_a_switch_is_refused(tmp_path):
    """error.c's map_error. Refusing it is what surfaced the §9.112 catch-all binding:
    the labels are ldap.h constants, and an unresolved bare identifier in PATTERN
    position matches everything."""
    v = _verdicts(
        tmp_path,
        """
ULONG map_error( int error )
{
    switch (error)
    {
#ifdef HAVE_LDAP_H
    case LDAP_TIMEOUT: return WLDAP32_LDAP_TIMEOUT;
#endif
    default: return error;
    }
}
""",
    )["map_error"]
    assert v.kind == COMPLEX


def test_two_guards_in_one_module_each_keep_their_own_feature(tmp_path):
    """wldap32 branches on EIGHT config probes; gating them all on the obvious one
    would have been silently wrong for seven."""
    vs = _verdicts(
        tmp_path,
        """
ULONG a( void ) { ULONG ret = 1;
#ifdef HAVE_LDAP
    ret = 2;
#endif
    return ret; }
ULONG b( void ) { ULONG ret = 1;
#ifdef HAVE_LDAP_H
    ret = 3;
#endif
    return ret; }
""",
    )
    assert vs["a"].feature == "have_ldap"
    assert vs["b"].feature == "have_ldap_h"


def test_a_name_classified_two_ways_is_an_error_not_a_coin_flip(tmp_path):
    from rustconv.cfg_gate import run

    (tmp_path / "one.c").write_text(
        "ULONG f( void ) { ULONG ret = 1;\n#ifdef HAVE_LDAP\nret = 2;\n#endif\nreturn ret; }"
    )
    (tmp_path / "two.c").write_text("ULONG f( void ) { return 1; }")
    rust = tmp_path / "rs"
    rust.mkdir()
    with pytest.raises(CfgGateError):
        run(tmp_path, rust, GUARDS, apply=False)


# --------------------------------------------------------------- the transform


RUST = """use crate::prelude::*;

#[no_mangle]
pub unsafe extern "C" fn ldap_addA(ld: *mut WLDAP32_LDAP) -> ULONG {
    let mut ret: ULONG = WLDAP32_LDAP_NOT_SUPPORTED;
    ret = ldap_add(ld);
    ret
}
"""


def test_twin_carries_the_attributes_and_the_lifted_initializer(tmp_path):
    """`#[no_mangle]` must survive on the off-arm or the export vanishes, and the return
    value is LIFTED from the translated `let`, never re-derived from the C constant."""
    v = _verdicts(
        tmp_path,
        """
ULONG ldap_addA( void *ld )
{
    ULONG ret = WLDAP32_LDAP_NOT_SUPPORTED;
#ifdef HAVE_LDAP
    ret = ldap_add( ld );
#endif
    return ret;
}
""",
    )
    rs = tmp_path / "add.rs"
    rs.write_text(RUST)
    res = apply_file(rs, v, write=True)
    out = rs.read_text()

    assert res.twinned == ["ldap_addA"]
    assert '#[cfg(not(feature = "have_ldap"))]' in out
    assert '#[cfg(feature = "have_ldap")]' in out
    assert out.count("#[no_mangle]") == 2, "both arms must keep the export attribute"
    assert "WLDAP32_LDAP_NOT_SUPPORTED\n}" in out
    # the paid translation itself is untouched
    assert "ret = ldap_add(ld);" in out


def test_applying_twice_is_a_no_op(tmp_path):
    v = _verdicts(
        tmp_path,
        """
ULONG ldap_addA( void *ld )
{
    ULONG ret = WLDAP32_LDAP_NOT_SUPPORTED;
#ifdef HAVE_LDAP
    ret = ldap_add( ld );
#endif
    return ret;
}
""",
    )
    rs = tmp_path / "add.rs"
    rs.write_text(RUST)
    apply_file(rs, v, write=True)
    once = rs.read_text()
    apply_file(rs, v, write=True)
    assert rs.read_text() == once


def test_complex_is_reported_and_never_written(tmp_path):
    v = _verdicts(
        tmp_path,
        """
ULONG ldap_addA( void *ld )
{
#ifdef HAVE_LDAP
    return 1;
#else
    return 2;
#endif
}
""",
    )
    rs = tmp_path / "add.rs"
    rs.write_text(RUST)
    res = apply_file(rs, v, write=True)
    assert [n for n, _ in res.skipped] == ["ldap_addA"]
    assert rs.read_text() == RUST


# --------------------------------------------------------------- regressions


def test_non_ascii_does_not_shift_extracted_names():
    """SHIPPED BUG: names were sliced out of a `str` with tree-sitter's BYTE offsets. One
    em-dash in helpers.rs's doc comment shifted every later name by two, so `bvarraydup`
    came out `arraydup(b`, matched no C verdict, and 43 functions were silently skipped.
    The identifier check now turns that into a loud failure; correct offsets keep it from
    arising at all."""
    src = '//! wldap32 — the em-dash that did it\n\npub unsafe extern "C" fn bvarraydup() {}\n'
    assert [f.name for f in _rust_fns(src.encode())] == ["bvarraydup"]


def test_guard_counting_is_word_anchored_not_substring():
    """SHIPPED BUG: `"ifdef HAVE_LDAP"` is a SUBSTRING of `"ifdef HAVE_LDAP_PARSE_REFERENCE"`,
    so a plain `str.count` saw two regions where there is one and refused two perfectly
    ordinary bodies as COMPLEX -- leaving them silently ungated."""
    text = "#ifdef HAVE_LDAP_PARSE_REFERENCE\nx;\n#endif\n"
    assert _count_ifdefs(text, GUARDS) == 1
    assert _count_ifdefs("#ifdef HAVE_LDAP\ny;\n#endif\n" + text, GUARDS) == 2


def test_the_reference_probe_body_classifies_as_body_gated(tmp_path):
    """The end-to-end shape the substring bug got wrong."""
    v = _verdicts(
        tmp_path,
        """
ULONG ldap_parse_referenceW( void *ld )
{
    ULONG ret = WLDAP32_LDAP_NOT_SUPPORTED;
#ifdef HAVE_LDAP_PARSE_REFERENCE
    ret = ldap_parse_reference( ld );
#endif
    return ret;
}
""",
    )["ldap_parse_referenceW"]
    assert v.kind == BODY_GATED
    assert v.feature == "have_ldap_parse_reference"
