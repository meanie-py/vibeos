"""Mechanical LLM output defects the post-processor must fix deterministically.

All three classes below were hit for real in the ole32 translation run:

  1. `memoffset::...offset_of!`   — no memoffset crate exists in this workspace
  2. `x.QuadPart()` / `x.u1()`    — OLD bindgen's union ACCESSOR-METHOD form;
                                    the bindgen we ship emits plain fields
  3. `.ref` / `.ref_0` / `.r#ref` — a C field named `ref` is a Rust PARSE error;
                                    bindgen escapes it as `ref_`, and rustconv
                                    emitted all three spellings in ONE file
"""

from __future__ import annotations

import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

from rustconv.postprocess import (
    fix_bindgen_accessor_calls,
    fix_keyword_field_names,
    fix_memoffset_paths,
    postprocess,
)

# ---------------------------------------------------------------------------
# 1. memoffset
# ---------------------------------------------------------------------------


def test_memoffset_hybrid_path():
    code = "let o = memoffset::core::mem::offset_of!(StorageBaseImpl, ref_);"
    assert fix_memoffset_paths(code) == "let o = core::mem::offset_of!(StorageBaseImpl, ref_);"


def test_memoffset_bare_crate_path():
    code = "let o = memoffset::offset_of!(Foo, bar);"
    assert fix_memoffset_paths(code) == "let o = core::mem::offset_of!(Foo, bar);"


def test_memoffset_use_is_retargeted_at_core():
    code = "use memoffset::offset_of;\nfn f() { offset_of!(Foo, bar); }\n"
    out = fix_memoffset_paths(code)
    assert "use core::mem::offset_of;" in out
    assert "memoffset" not in out


def test_memoffset_other_use_is_dropped():
    code = "use core::ptr;\nuse memoffset::span_of;\nfn f() {}\n"
    out = fix_memoffset_paths(code)
    assert "memoffset" not in out
    assert "use core::ptr;" in out
    assert "fn f() {}" in out


def test_memoffset_is_not_a_bare_word_rewrite():
    # oleaut32's typelib really does have a FIELD named `memoffset`
    # (MSFT_TypeInfoBase.memoffset) — the pass keys on the `memoffset::` PATH.
    code = "    (*base).memoffset = compile_aux(info, file) as i32;"
    assert fix_memoffset_paths(code) == code


def test_memoffset_idempotent():
    code = "let o = core::mem::offset_of!(Foo, bar);"
    assert fix_memoffset_paths(code) == code


# ---------------------------------------------------------------------------
# 2. old-bindgen union accessor CALLS
# ---------------------------------------------------------------------------


def test_accessor_call_loses_parens():
    code = "let q = (*li).QuadPart();\nlet a = ps.u1();\nlet b = x.s2();"
    out = fix_bindgen_accessor_calls(code)
    assert "(*li).QuadPart;" in out
    assert "ps.u1;" in out
    assert "x.s2;" in out


def test_accessor_call_does_not_touch_real_methods():
    code = (
        "let p = buf.as_ptr();\nlet n = v.len();\nlet s = name.to_string();\nlet u = foo.unwrap();"
    )
    assert fix_bindgen_accessor_calls(code) == code


def test_accessor_field_form_left_alone():
    code = "let q = (*li).QuadPart;"
    assert fix_bindgen_accessor_calls(code) == code


def test_dummy_union_family_maps_to_bindgen_anon():
    code = (
        "let a = (*p).DUMMYUNIONNAME();\n"
        "let b = (*p).DUMMYUNIONNAME2.Foo;\n"
        "let c = x.DUMMYSTRUCTNAME().y;"
    )
    out = fix_bindgen_accessor_calls(code)
    assert "(*p).__bindgen_anon_1;" in out
    assert "(*p).__bindgen_anon_2.Foo;" in out
    assert "x.__bindgen_anon_1.y;" in out


def test_accessor_skips_comment_lines():
    code = "// the C code does li.QuadPart()\nlet q = li.QuadPart();"
    out = fix_bindgen_accessor_calls(code)
    assert "// the C code does li.QuadPart()" in out
    assert "let q = li.QuadPart;" in out


def test_lowpart_call_composes_with_the_anon_qualifier_pass():
    # .LowPart() → .LowPart (this pass) → .__bindgen_anon_1.LowPart (the
    # existing fix_bindgen_anon_field_access pass, which runs right after).
    out = postprocess("fn f() { let x = (*li).LowPart(); }")
    assert "(*li).__bindgen_anon_1.LowPart;" in out


# ---------------------------------------------------------------------------
# 3. Rust-keyword C field names
# ---------------------------------------------------------------------------


def test_ref_all_three_spellings_canonicalise_to_ref_():
    code = (
        "    (*This).ref = 1;\n"
        "    let n = (*This).ref_0;\n"
        "    InterlockedIncrement(addr_of_mut!((*This).r#ref));\n"
        "    let m = (*This).ref_;\n"
    )
    out = fix_keyword_field_names(code)
    assert out.count(".ref_") == 4, out
    assert ".ref " not in out and ".ref_0" not in out and ".r#ref" not in out


def test_keyword_pass_is_general_not_ref_only():
    code = "let t = (*p).type;\nlet b = (*p).box;\nlet m = (*p).match;"
    out = fix_keyword_field_names(code)
    assert "(*p).type_;" in out
    assert "(*p).box_;" in out
    assert "(*p).match_;" in out


def test_keyword_pass_does_not_eat_normal_identifiers():
    code = (
        "for i in 0..n { }\n"
        "let x = v.into();\n"
        "let y = obj.reference;\n"
        "let z = s.as_ptr();\n"
        "let w = cfg.mutex;\n"
        "let c = e.constant;\n"
    )
    assert fix_keyword_field_names(code) == code


def test_keyword_pass_idempotent():
    code = "    (*This).ref_ = 1;"
    assert fix_keyword_field_names(code) == code
    assert fix_keyword_field_names(fix_keyword_field_names(code)) == code


def test_keyword_pass_skips_comments():
    code = "// wine calls this field .ref\nlet n = (*p).ref;"
    out = fix_keyword_field_names(code)
    assert "// wine calls this field .ref" in out
    assert "(*p).ref_;" in out


# ---------------------------------------------------------------------------
# all three, wired into the real pipeline
# ---------------------------------------------------------------------------


def test_all_three_wired_into_postprocess():
    code = (
        "fn f() {\n"
        "    let o = memoffset::core::mem::offset_of!(StorageBaseImpl, ref_);\n"
        "    (*This).ref = 1;\n"
        "    let q = (*li).QuadPart();\n"
        "}\n"
    )
    out = postprocess(code)
    assert "core::mem::offset_of!(StorageBaseImpl, ref_)" in out
    assert "memoffset::" not in out
    assert "(*This).ref_ = 1;" in out
    assert "(*li).QuadPart;" in out


def test_large_integer_u_member_is_not_rerouted():
    """`x.u.HighPart` is LARGE_INTEGER's named member; only a bare `x.HighPart` is anonymous."""
    from rustconv.postprocess import fix_bindgen_anon_field_access

    assert fix_bindgen_anon_field_access("a.u.HighPart != 0") == "a.u.HighPart != 0"
    assert fix_bindgen_anon_field_access("a.HighPart") == "a.__bindgen_anon_1.HighPart"
