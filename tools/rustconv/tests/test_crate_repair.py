"""Tests for the bounded deterministic crate-repair pass.

Every fixture below mirrors the SHAPE of real rustc 1.92 JSON (captured from
`rustc --error-format=json` on minimal reproductions), because the guards in
`crate_repair` are all decisions about that shape:

  * the value span carrying `expected X, found Y` may be SECONDARY (the
    "arguments to this function are incorrect" shape puts the CALLEE in the primary span),
  * rustc's conversion `help` child span is a ZERO-LENGTH insertion point whose
    `suggested_replacement` is the panicking `.try_into().unwrap()`.

Each guard has a test that FAILS if the guard is deleted — the zero-length-span and
callee-followed-by-`(` cases produced real debris (`() as u32`, `(two) as u32`) in the
hand-rolled version of this pass.
"""

from __future__ import annotations

from pathlib import Path

import pytest

from rustconv.crate_repair import (
    RULE_ANON_UNION,
    RULE_FLEX_ARRAY,
    RULE_INDEX_USIZE,
    RULE_MACRO_CALL,
    RULE_MATCH_SCRUTINEE,
    RULE_NO_STD_PATH,
    RULE_NUMERIC_CAST,
    RULE_PACKED_FIELD_PTR,
    RULE_POINTER_CAST,
    RULE_SIZEOF_TYPE,
    CargoCheckRun,
    Edit,
    RepairError,
    apply_edits,
    count_errors,
    flex_array_decls,
    harvest_anon_unions,
    merge_plans,
    parse_cargo_messages,
    pointer_width,
    propose_edits,
    propose_flex_array_edits,
    repair_crate,
    resolve_target,
    select_non_overlapping,
)

# --- fixture helpers --------------------------------------------------------------------------


def _span(
    file_name: str,
    byte_start: int,
    byte_end: int,
    *,
    is_primary: bool = False,
    label: str | None = None,
    line: int = 1,
    column: int = 1,
    replacement: str | None = None,
    applicability: str | None = None,
) -> dict:
    return {
        "file_name": file_name,
        "byte_start": byte_start,
        "byte_end": byte_end,
        "line_start": line,
        "line_end": line,
        "column_start": column,
        "column_end": column,
        "is_primary": is_primary,
        "text": [],
        "label": label,
        "suggested_replacement": replacement,
        "suggestion_applicability": applicability,
        "expansion": None,
    }


def _diag(
    message: str,
    *,
    code: str | None = "E0308",
    level: str = "error",
    spans: list[dict] | None = None,
    children: list[dict] | None = None,
) -> dict:
    return {
        "$message_type": "diagnostic",
        "message": message,
        "code": {"code": code, "explanation": None} if code else None,
        "level": level,
        "spans": spans or [],
        "children": children or [],
        "rendered": None,
    }


def _convert_hint(file_name: str, at: int, found: str, expected: str) -> dict:
    """rustc's real help child: a ZERO-LENGTH insertion span with a panicking replacement."""
    return {
        "message": f"you can convert a `{found}` to a `{expected}` and panic if the converted "
        "value doesn't fit",
        "code": None,
        "level": "help",
        "spans": [
            _span(
                file_name,
                at,
                at,
                is_primary=True,
                replacement=".try_into().unwrap()",
                applicability="MachineApplicable",
            )
        ],
        "children": [],
        "rendered": None,
    }


def _reader(sources: dict[str, bytes]):
    return lambda name: sources.get(name)


# The real single-argument shape: primary span IS the value, plus the callee secondary span.
_ARG_SRC = b"pub fn takes_u32(_x: u32) {}\npub fn f(v: &[u8]) {\n    takes_u32(v.len());\n}\n"
_ARG_VALUE = (_ARG_SRC.index(b"v.len()", 40), _ARG_SRC.index(b"v.len()", 40) + len(b"v.len()"))
_ARG_CALLEE = (_ARG_SRC.index(b"takes_u32(v"), _ARG_SRC.index(b"takes_u32(v") + 9)


def _arg_diag(expected: str = "u32", found: str = "usize", file_name: str = "src/a.rs") -> dict:
    return _diag(
        "mismatched types",
        spans=[
            _span(
                file_name,
                *_ARG_VALUE,
                is_primary=True,
                label=f"expected `{expected}`, found `{found}`",
                line=3,
            ),
            _span(
                file_name,
                *_ARG_CALLEE,
                label="arguments to this function are incorrect",
                line=3,
            ),
        ],
        children=[_convert_hint(file_name, _ARG_VALUE[1], found, expected)],
    )


# --- rule 1: faithful truncating numeric cast -------------------------------------------------


def test_numeric_cast_is_faithful_not_panicking():
    plan = propose_edits(
        [_arg_diag()], _reader({"src/a.rs": _ARG_SRC}), target="amd64", rules=[RULE_NUMERIC_CAST]
    )
    assert [edit.replacement for edit in plan.edits] == ["(v.len()) as u32"]
    edit = plan.edits[0]
    assert edit.rule == RULE_NUMERIC_CAST
    assert "try_into" not in edit.replacement and "unwrap" not in edit.replacement
    assert "panic" in edit.authority  # the authority records WHY rustc's own fix is refused
    assert edit.original == "v.len()"


def test_zero_length_child_span_never_becomes_an_edit():
    """Guard: wrapping rustc's insertion-point span produced `() as u32` debris."""
    file_name = "src/a.rs"
    hint_at = _ARG_VALUE[1]
    diagnostic = _diag(
        "mismatched types",
        spans=[
            # The label sits on the zero-length insertion point (what a naive
            # "use the suggestion span" implementation ends up with).
            _span(
                file_name, hint_at, hint_at, is_primary=True, label="expected `u32`, found `usize`"
            )
        ],
        children=[_convert_hint(file_name, hint_at, "usize", "u32")],
    )
    plan = propose_edits(
        [diagnostic],
        _reader({file_name: _ARG_SRC}),
        target="amd64",
        rules=[RULE_NUMERIC_CAST],
    )
    assert plan.edits == ()
    assert [(item.rule, item.reason) for item in plan.skipped] == [
        (RULE_NUMERIC_CAST, "zero_length_span")
    ]


def test_callee_span_followed_by_paren_is_declined():
    """Guard: the `arguments to this function are incorrect` primary span is the CALLEE."""
    file_name = "src/a.rs"
    diagnostic = _diag(
        "arguments to this function are incorrect",
        spans=[
            _span(file_name, *_ARG_CALLEE, is_primary=True, label="expected `u32`, found `usize`"),
            _span(file_name, *_ARG_VALUE, label="expected `u32`, found `usize`"),
        ],
    )
    plan = propose_edits(
        [diagnostic], _reader({file_name: _ARG_SRC}), target="amd64", rules=[RULE_NUMERIC_CAST]
    )
    assert [edit.original for edit in plan.edits] == ["v.len()"]  # the value, never the callee
    assert ("takes_u32" in item.detail for item in plan.skipped)
    assert [item.reason for item in plan.skipped] == ["callee_span"]


def test_secondary_label_spans_are_repaired():
    """The multi-bad-argument shape: BOTH value spans are secondary, the callee is primary."""
    source = (
        b"pub fn two(_a: u32, _b: u16) {}\n"
        b"pub fn g(v: &[u8], w: &[u8]) {\n    two(v.len(), w.len());\n}\n"
    )
    first = source.index(b"v.len()", 40)
    second = source.index(b"w.len()", 40)
    callee = source.index(b"two(v")
    diagnostic = _diag(
        "arguments to this function are incorrect",
        spans=[
            _span("src/b.rs", first, first + 7, label="expected `u32`, found `usize`", line=3),
            _span("src/b.rs", second, second + 7, label="expected `u16`, found `usize`", line=3),
            _span("src/b.rs", callee, callee + 3, is_primary=True, line=3),
        ],
        children=[
            _convert_hint("src/b.rs", first + 7, "usize", "u32"),
            _convert_hint("src/b.rs", second + 7, "usize", "u16"),
        ],
    )
    plan = propose_edits(
        [diagnostic], _reader({"src/b.rs": source}), target="amd64", rules=[RULE_NUMERIC_CAST]
    )
    assert sorted(edit.replacement for edit in plan.edits) == [
        "(v.len()) as u32",
        "(w.len()) as u16",
    ]


def test_hint_that_contradicts_the_label_is_declined():
    file_name = "src/a.rs"
    diagnostic = _diag(
        "mismatched types",
        spans=[
            _span(
                file_name,
                *_ARG_VALUE,
                is_primary=True,
                label="expected `u32`, found `usize`",
                line=3,
            )
        ],
        children=[_convert_hint(file_name, _ARG_VALUE[1], "usize", "u16")],
    )
    plan = propose_edits(
        [diagnostic], _reader({file_name: _ARG_SRC}), target="amd64", rules=[RULE_NUMERIC_CAST]
    )
    assert plan.edits == ()
    assert [item.reason for item in plan.skipped] == ["hint_label_disagreement"]


def test_span_already_carrying_the_cast_is_declined():
    source = b"pub fn f(v: &[u8]) {\n    takes_u32(v.len() as u32);\n}\n"
    start = source.index(b"v.len() as u32")
    diagnostic = _diag(
        "mismatched types",
        spans=[
            _span(
                "src/a.rs",
                start,
                start + len(b"v.len() as u32"),
                is_primary=True,
                label="expected `u32`, found `usize`",
                line=2,
            )
        ],
    )
    plan = propose_edits(
        [diagnostic], _reader({"src/a.rs": source}), target="amd64", rules=[RULE_NUMERIC_CAST]
    )
    assert plan.edits == ()
    assert [item.reason for item in plan.skipped] == ["already_cast"]


def test_unbalanced_span_is_declined():
    source = b"let x = foo(a, b);\n"
    start = source.index(b"foo(a")
    diagnostic = _diag(
        "mismatched types",
        spans=[
            _span(
                "src/a.rs",
                start,
                start + len(b"foo(a"),  # deliberately unbalanced
                is_primary=True,
                label="expected `u32`, found `usize`",
            )
        ],
    )
    plan = propose_edits(
        [diagnostic], _reader({"src/a.rs": source}), target="amd64", rules=[RULE_NUMERIC_CAST]
    )
    assert plan.edits == ()
    assert [item.reason for item in plan.skipped] == ["unbalanced_span"]


def test_non_integer_mismatch_stays_red():
    diagnostic = _diag(
        "mismatched types",
        spans=[
            _span(
                "src/a.rs",
                0,
                4,
                is_primary=True,
                label="expected `MIB_ROW`, found `*mut MIB_ROW`",
            )
        ],
    )
    plan = propose_edits([diagnostic], _reader({"src/a.rs": b"abcd"}), target="amd64")
    assert plan.edits == ()
    assert len(plan.red) == 1
    assert plan.red[0].code == "E0308"


def test_byte_offsets_survive_non_ascii_source():
    """rustc reports BYTE offsets; a str round-trip would shift them past a non-ASCII comment."""
    source = "// width — the em dash makes this line longer in bytes than chars\nlet n = v.len();\n"
    data = source.encode("utf-8")
    start = data.index(b"v.len()")
    diagnostic = _diag(
        "mismatched types",
        spans=[
            _span(
                "src/a.rs",
                start,
                start + 7,
                is_primary=True,
                label="expected `u16`, found `usize`",
                line=2,
            )
        ],
    )
    plan = propose_edits(
        [diagnostic], _reader({"src/a.rs": data}), target="amd64", rules=[RULE_NUMERIC_CAST]
    )
    assert [edit.original for edit in plan.edits] == ["v.len()"]


# --- the width caveat: never bake an i386 width -----------------------------------------------


def _width_plan(target: str, expected: str):
    return propose_edits(
        [_arg_diag(expected=expected)],
        _reader({"src/a.rs": _ARG_SRC}),
        target=target,
        rules=[RULE_NUMERIC_CAST],
    )


def test_pointer_width_cast_from_one_target_is_withheld():
    """`as u32` inferred on i386 may be a SIZE_T/ULONG_PTR that is u64 elsewhere."""
    plan = _width_plan("i386", "u32")
    assert plan.edits[0].requires_target_agreement is True
    merged = merge_plans({"i386": plan})
    assert merged.edits == ()
    assert [item.reason for item in merged.withheld] == ["width_unproven"]
    assert "semantic alias" in merged.withheld[0].detail


def test_pointer_width_cast_applies_when_two_targets_agree():
    merged = merge_plans({"i386": _width_plan("i386", "u32"), "amd64": _width_plan("amd64", "u32")})
    assert [edit.replacement for edit in merged.edits] == ["(v.len()) as u32"]
    assert merged.edits[0].targets == ("amd64", "i386")
    assert "agreed by amd64,i386" in merged.edits[0].authority
    assert merged.withheld == ()


def test_a_target_that_reported_nothing_vetoes_a_width_bake():
    """desk.cpl: 54 call sites that COMPILED on i386 were broken by amd64+arm64 agreeing.

    `SendDlgItemMessage(.., CB_INSERTSTRING, PLACEMENT_CENTER, ..)` needs no cast on i386 --
    WPARAM is u32 there and so is the constant -- so i386 proposes nothing while the 64-bit pair
    both ask for `as u64`. Two targets agreeing is not agreement: silence is evidence the site is
    already right, and the honest answer is the callee's declared name (`as WPARAM`).
    """
    merged = merge_plans(
        {
            "i386": propose_edits(
                [], _reader({"src/a.rs": _ARG_SRC}), target="i386", rules=[RULE_NUMERIC_CAST]
            ),
            "amd64": _width_plan("amd64", "u64"),
            "arm64": _width_plan("arm64", "u64"),
        }
    )
    assert merged.edits == ()
    assert [item.reason for item in merged.withheld] == ["width_unproven"]
    assert "i386 reported nothing here" in merged.withheld[0].detail


def test_divergent_widths_are_withheld_not_guessed():
    """The wkssvc mistake: i386 wants u32, amd64 wants u64 — the real type is SIZE_T."""
    merged = merge_plans({"i386": _width_plan("i386", "u32"), "amd64": _width_plan("amd64", "u64")})
    assert merged.edits == ()
    assert [item.reason for item in merged.withheld] == ["target_divergent"]
    detail = merged.withheld[0].detail
    assert "i386=>(v.len()) as u32" in detail and "amd64=>(v.len()) as u64" in detail


def test_cast_to_usize_is_portable_from_a_single_target():
    """`as usize` IS the width-varying alias, so no cross-target proof is needed."""
    plan = _width_plan("i386", "usize")
    assert plan.edits[0].requires_target_agreement is False
    merged = merge_plans({"i386": plan})
    assert [edit.replacement for edit in merged.edits] == ["(v.len()) as usize"]


def test_non_pointer_width_cast_is_portable_from_a_single_target():
    merged = merge_plans({"i386": _width_plan("i386", "u16")})
    assert [edit.replacement for edit in merged.edits] == ["(v.len()) as u16"]


def test_single_target_width_override_is_recorded_in_the_authority():
    merged = merge_plans({"i386": _width_plan("i386", "u32")}, allow_single_target_width=True)
    assert len(merged.edits) == 1
    assert "single_target_width_override(i386)" in merged.edits[0].authority


def test_pointer_width_is_loud_on_an_unknown_target():
    with pytest.raises(RepairError, match="unknown target"):
        pointer_width("riscv64-unknown-none")
    assert pointer_width("i686-pc-windows-gnu") == 4
    assert pointer_width("x86_64-pc-windows-gnu") == 8
    assert resolve_target("arm64") == "aarch64-pc-windows-gnullvm"


# --- rule 3: std -> core ----------------------------------------------------------------------


def test_std_import_is_rewritten_to_core():
    source = b"#![no_std]\nuse std::mem;\n"
    start = source.index(b"std::")
    diagnostic = _diag(
        "unresolved import `std`",
        code="E0432",
        spans=[
            _span(
                "src/lib.rs",
                start,
                start + 3,
                is_primary=True,
                label="use of unresolved module or unlinked crate `std`",
                line=2,
            )
        ],
    )
    plan = propose_edits(
        [diagnostic], _reader({"src/lib.rs": source}), target="amd64", rules=[RULE_NO_STD_PATH]
    )
    assert [(edit.original, edit.replacement) for edit in plan.edits] == [("std", "core")]
    assert "core" in plan.edits[0].authority


def test_std_path_outside_core_is_declined():
    source = b"#![no_std]\nlet m = std::collections::BTreeMap::new();\n"
    start = source.index(b"std::")
    diagnostic = _diag(
        "failed to resolve: use of unresolved module or unlinked crate `std`",
        code="E0433",
        spans=[
            _span(
                "src/lib.rs",
                start,
                start + 3,
                is_primary=True,
                label="use of unresolved module or unlinked crate `std`",
                line=2,
            )
        ],
    )
    plan = propose_edits(
        [diagnostic], _reader({"src/lib.rs": source}), target="amd64", rules=[RULE_NO_STD_PATH]
    )
    assert plan.edits == ()
    assert [(item.reason, item.detail) for item in plan.skipped] == [
        ("std_path_not_in_core", "std::collections")
    ]


# --- rule 4: raw-pointer .cast() --------------------------------------------------------------


def _pointer_diag(expected: str, found: str, source: bytes, needle: bytes) -> dict:
    start = source.index(needle)
    return _diag(
        "mismatched types",
        spans=[
            _span(
                "src/c.rs",
                start,
                start + len(needle),
                is_primary=True,
                label=f"expected `{expected}`, found `{found}`",
            )
        ],
    )


def test_same_mutability_pointer_mismatch_becomes_cast():
    source = b"unsafe { Foo((*pkt).Srb); }\n"
    plan = propose_edits(
        [_pointer_diag("*mut c_void", "*mut SRB_HEADER", source, b"(*pkt).Srb")],
        _reader({"src/c.rs": source}),
        target="amd64",
        rules=[RULE_POINTER_CAST],
    )
    assert [edit.replacement for edit in plan.edits] == ["((*pkt).Srb).cast()"]
    assert "INFERS" in plan.edits[0].authority


def test_pointer_mutability_flip_stays_red():
    source = b"unsafe { Foo((*pkt).Srb); }\n"
    plan = propose_edits(
        [_pointer_diag("*const i8", "*mut i8", source, b"(*pkt).Srb")],
        _reader({"src/c.rs": source}),
        target="amd64",
        rules=[RULE_POINTER_CAST],
    )
    assert plan.edits == ()
    assert [item.reason for item in plan.skipped] == ["pointer_mutability_flip"]


# --- rule 2: bindgen flexible-array members ---------------------------------------------------

_TYPES = """
#[repr(C)]
pub struct MIB_IPFORWARDTABLE {
    pub dwNumEntries: DWORD,
    pub table: [MIB_IPFORWARDROW; 1],
}
#[repr(C)]
pub struct IP_ADAPTER_INDEX_MAP {
    pub Name: [WCHAR; 128],
    pub routes: [MIB_ROW; 1],
}
#[repr(C)]
pub struct FIXED {
    pub table: [OTHER; 8],
}
"""


def test_flex_array_decls_refuse_ambiguous_field_names():
    decls = flex_array_decls(_TYPES)
    assert decls.fields == {"routes": "MIB_ROW"}
    assert decls.ambiguous == frozenset({"table"})  # `table` is [_;1] here and [_;8] there
    assert "Name" not in decls.fields


def test_flex_array_decls_do_not_depend_on_rustfmt_layout():
    """A layout-sensitive pattern found ZERO fields in a hand-written types module — a silent
    no-op is the failure mode this rule must never have."""
    one_line = "#[repr(C)]\npub struct T { pub n: u32, pub routes: [MIB_ROW; 1] }\n"
    assert flex_array_decls(one_line).fields == {"routes": "MIB_ROW"}
    spaced = "pub struct T {\n    pub routes : [ MIB_ROW ; 1 ] ,\n}\n"
    assert flex_array_decls(spaced).fields == {"routes": "MIB_ROW"}


def test_flex_array_read_becomes_pointer_arithmetic():
    decls = flex_array_decls(_TYPES)
    source = b"pub unsafe fn f(t: *mut T, i: usize) -> u32 {\n    (*t).routes[i].dest\n}\n"
    edits, skipped = propose_flex_array_edits("src/f.rs", source, decls)
    assert [edit.replacement for edit in edits] == ["(*(*t).routes.as_ptr().add(i))"]
    assert edits[0].original == "(*t).routes[i]"
    assert edits[0].rule == RULE_FLEX_ARRAY
    assert "[MIB_ROW; 1]" in edits[0].authority
    assert skipped and skipped[0].reason == "ambiguous_field_length"


def test_flex_array_write_uses_as_mut_ptr_through_a_field_hop():
    decls = flex_array_decls(_TYPES)
    source = b"pub unsafe fn f(t: *mut T, i: usize) {\n    (*t).routes[i].dest = 0;\n}\n"
    edits, _ = propose_flex_array_edits("src/f.rs", source, decls)
    assert [edit.replacement for edit in edits] == ["(*(*t).routes.as_mut_ptr().add(i))"]


def test_flex_array_borrowed_mut_uses_as_mut_ptr():
    decls = flex_array_decls(_TYPES)
    source = b"pub unsafe fn f(t: *mut T, i: usize) {\n    g(&mut (*t).routes[i]);\n}\n"
    edits, _ = propose_flex_array_edits("src/f.rs", source, decls)
    assert [edit.replacement for edit in edits] == ["(*(*t).routes.as_mut_ptr().add(i))"]


def test_flex_array_index_zero_is_left_alone():
    decls = flex_array_decls(_TYPES)
    source = b"pub unsafe fn f(t: *mut T) -> u32 {\n    (*t).routes[0].dest\n}\n"
    edits, skipped = propose_flex_array_edits("src/f.rs", source, decls)
    assert edits == []
    assert [item.reason for item in skipped] == ["ambiguous_field_length"]


def test_flex_array_outside_unsafe_is_declined():
    decls = flex_array_decls(_TYPES)
    source = b"pub fn f(t: &T, i: usize) -> u32 {\n    t.routes[i].dest\n}\n"
    edits, skipped = propose_flex_array_edits("src/f.rs", source, decls)
    assert edits == []
    assert "not_unsafe_context" in {item.reason for item in skipped}


def test_flex_array_refuses_a_base_it_cannot_close():
    """A method-call/paren chain means our lexical base is a SUFFIX — splicing `*` would corrupt."""
    decls = flex_array_decls(_TYPES)
    source = b"pub unsafe fn f(t: *mut T, i: usize) -> u32 {\n    (*head(t)).q.routes[i].dest\n}\n"
    edits, skipped = propose_flex_array_edits("src/f.rs", source, decls)
    assert edits == []
    assert "base_not_closed" in {item.reason for item in skipped}


def test_flex_array_ignores_comments_and_strings():
    decls = flex_array_decls(_TYPES)
    source = b'pub unsafe fn f() {\n    // (*t).routes[i]\n    let s = "(*t).routes[i]";\n}\n'
    edits, _ = propose_flex_array_edits("src/f.rs", source, decls)
    assert edits == []


# --- span selection + application --------------------------------------------------------------


def test_overlapping_spans_keep_the_outer_one():
    outer = Edit("src/a.rs", 10, 30, "X", RULE_NUMERIC_CAST, "a", original="o")
    inner = Edit("src/a.rs", 15, 20, "Y", RULE_NUMERIC_CAST, "a", original="i")
    later = Edit("src/a.rs", 40, 45, "Z", RULE_NUMERIC_CAST, "a", original="l")
    chosen, dropped = select_non_overlapping([inner, outer, later])
    assert [edit.replacement for edit in chosen] == ["X", "Z"]
    assert [item.reason for item in dropped] == ["overlapping_span"]


def test_apply_edits_splices_back_to_front(tmp_path: Path):
    path = tmp_path / "src" / "a.rs"
    path.parent.mkdir(parents=True)
    original = b"let x = one(a) + two(b);\n"
    path.write_bytes(original)
    first = original.index(b"a)")
    second = original.index(b"b)")
    edits = [
        Edit("src/a.rs", first, first + 1, "(a) as u32", RULE_NUMERIC_CAST, "auth", original="a"),
        Edit("src/a.rs", second, second + 1, "(b) as u16", RULE_NUMERIC_CAST, "auth", original="b"),
    ]
    applied, snapshot, failed = apply_edits(edits, tmp_path)
    assert len(applied) == 2 and failed == []
    assert path.read_bytes() == b"let x = one((a) as u32) + two((b) as u16);\n"
    assert snapshot["src/a.rs"] == original


def test_apply_edits_refuses_a_stale_span(tmp_path: Path):
    path = tmp_path / "a.rs"
    path.write_bytes(b"let x = 1;\n")
    edits = [Edit("a.rs", 0, 3, "REPL", RULE_NUMERIC_CAST, "auth", original="not-there")]
    applied, _, failed = apply_edits(edits, tmp_path)
    assert applied == []
    assert [item.reason for item in failed] == ["stale_span"]
    assert path.read_bytes() == b"let x = 1;\n"


def test_apply_edits_refuses_to_write_a_panicking_conversion(tmp_path: Path):
    """Backstop: no rule may ever write rustc's `.try_into().unwrap()` into a live service."""
    path = tmp_path / "a.rs"
    path.write_bytes(b"let x = v.len();\n")
    edit = Edit(
        "a.rs",
        8,
        15,
        "v.len().try_into().unwrap()",
        RULE_NUMERIC_CAST,
        "auth",
        original="v.len()",
    )
    with pytest.raises(RepairError, match="panicking conversion"):
        apply_edits([edit], tmp_path)
    assert path.read_bytes() == b"let x = v.len();\n"


def test_apply_edits_raises_on_overlap(tmp_path: Path):
    path = tmp_path / "a.rs"
    path.write_bytes(b"0123456789\n")
    edits = [
        Edit("a.rs", 0, 5, "A", RULE_NUMERIC_CAST, "auth", original="01234"),
        Edit("a.rs", 3, 8, "B", RULE_NUMERIC_CAST, "auth", original="34567"),
    ]
    with pytest.raises(RepairError, match="overlapping edits"):
        apply_edits(edits, tmp_path)


# --- cargo message parsing --------------------------------------------------------------------


def test_parse_cargo_messages_unwraps_and_dedupes():
    import json as _json

    diagnostic = _arg_diag()
    lines = [
        _json.dumps({"reason": "compiler-artifact", "target": {}}),
        _json.dumps({"reason": "compiler-message", "message": diagnostic}),
        _json.dumps({"reason": "compiler-message", "message": diagnostic}),  # lib + test target
        _json.dumps(_diag("aborting due to 1 previous error", code=None)),
        "not json at all",
    ]
    parsed = parse_cargo_messages("\n".join(lines))
    assert len(parsed) == 2
    assert count_errors(parsed) == 1  # the summary line is not an error


# --- driver safety invariant -------------------------------------------------------------------


def test_repair_crate_reverts_a_pass_that_raises_the_error_count(tmp_path: Path):
    """The 'never leave it worse' abort: a regressing pass is REVERTED, not left behind."""
    crate_dir = tmp_path / "demo-rust"
    (crate_dir / "src").mkdir(parents=True)
    source = _ARG_SRC
    (crate_dir / "src" / "a.rs").write_bytes(source)
    file_name = "demo-rust/src/a.rs"
    start = source.index(b"v.len()", 40)
    diagnostic = _diag(
        "mismatched types",
        spans=[
            _span(
                file_name,
                start,
                start + 7,
                is_primary=True,
                label="expected `u16`, found `usize`",
                line=3,
            )
        ],
    )
    calls = {"n": 0}

    def fake_check(target: str) -> CargoCheckRun:
        calls["n"] += 1
        if calls["n"] == 1:
            return CargoCheckRun(target, (diagnostic,), 1, 101)
        # Pretend our edit made things worse (this is what the guard exists for).
        return CargoCheckRun(target, (diagnostic, diagnostic), 9, 101)

    session = repair_crate(
        "demo-rust",
        root=tmp_path,
        targets=["amd64"],
        rules=[RULE_NUMERIC_CAST],
        check_runner=fake_check,
    )
    assert session.status == "regressed"
    assert "REVERTED" in session.detail
    assert (crate_dir / "src" / "a.rs").read_bytes() == source


def test_repair_crate_reaches_a_fixpoint_and_ledgers_every_edit(tmp_path: Path):
    crate_dir = tmp_path / "demo-rust"
    (crate_dir / "src").mkdir(parents=True)
    source = _ARG_SRC
    (crate_dir / "src" / "a.rs").write_bytes(source)
    file_name = "demo-rust/src/a.rs"
    start = source.index(b"v.len()", 40)
    diagnostic = _diag(
        "mismatched types",
        spans=[
            _span(
                file_name,
                start,
                start + 7,
                is_primary=True,
                label="expected `u16`, found `usize`",
                line=3,
            )
        ],
    )
    state = {"fixed": False}

    def fake_check(target: str) -> CargoCheckRun:
        if state["fixed"]:
            return CargoCheckRun(target, (), 0, 0)
        state["fixed"] = True
        return CargoCheckRun(target, (diagnostic,), 1, 101)

    session = repair_crate(
        "demo-rust",
        root=tmp_path,
        targets=["amd64"],
        rules=[RULE_NUMERIC_CAST],
        check_runner=fake_check,
    )
    assert session.status == "fixpoint"
    assert (crate_dir / "src" / "a.rs").read_bytes().count(b"(v.len()) as u16") == 1
    ledger = session.to_dict()
    assert ledger["applied_total"] == 1
    entry = ledger["passes"][0]["applied"][0]
    assert entry["rule"] == RULE_NUMERIC_CAST
    assert entry["authority"] and entry["before"] == "v.len()"
    assert entry["targets"] == ["amd64"]


def test_repair_crate_dry_run_writes_nothing(tmp_path: Path):
    crate_dir = tmp_path / "demo-rust"
    (crate_dir / "src").mkdir(parents=True)
    (crate_dir / "src" / "a.rs").write_bytes(_ARG_SRC)
    file_name = "demo-rust/src/a.rs"
    start = _ARG_SRC.index(b"v.len()", 40)
    diagnostic = _diag(
        "mismatched types",
        spans=[
            _span(
                file_name,
                start,
                start + 7,
                is_primary=True,
                label="expected `u16`, found `usize`",
                line=3,
            )
        ],
    )
    session = repair_crate(
        "demo-rust",
        root=tmp_path,
        targets=["amd64"],
        rules=[RULE_NUMERIC_CAST],
        dry_run=True,
        check_runner=lambda target: CargoCheckRun(target, (diagnostic,), 1, 101),
    )
    assert session.status == "dry_run"
    assert len(session.applied) == 1
    assert (crate_dir / "src" / "a.rs").read_bytes() == _ARG_SRC


def test_repair_crate_reports_a_missing_types_module_instead_of_guessing(tmp_path: Path):
    crate_dir = tmp_path / "demo-rust"
    (crate_dir / "src").mkdir(parents=True)
    (crate_dir / "src" / "a.rs").write_bytes(b"pub fn f() {}\n")
    session = repair_crate(
        "demo-rust",
        root=tmp_path,
        targets=["amd64"],
        check_runner=lambda target: CargoCheckRun(target, (), 0, 0),
    )
    reasons = {item.reason for item in session.passes[0].skipped}
    assert "no_bindgen_types_module" in reasons


def test_repair_crate_says_so_when_the_types_module_has_no_flex_members(tmp_path: Path):
    crate_dir = tmp_path / "demo-rust"
    (crate_dir / "src").mkdir(parents=True)
    (crate_dir / "src" / "types.rs").write_text("pub struct T { pub n: u32 }\n")
    (crate_dir / "src" / "a.rs").write_bytes(b"pub fn f() {}\n")
    session = repair_crate(
        "demo-rust",
        root=tmp_path,
        targets=["amd64"],
        check_runner=lambda target: CargoCheckRun(target, (), 0, 0),
    )
    reasons = {item.reason for item in session.passes[0].skipped}
    assert "no_flex_array_members" in reasons


def test_flex_array_decls_parses_real_bindgen_suffix():
    """bindgen writes `[T; 1usize]`, not `[T; 1]`.

    Regression: the original count pattern was `(?P<count>\\d+)\\s*\\]`, so `\\d+` matched the
    `1` and the closing `\\]` then hit the `u`. Result: ZERO fields discovered on every
    generated types module in the tree (iphlpapi_rust declares 32), i.e. the rule was a
    silent no-op. Found only by running the pass against a real crate.
    """
    source = """
#[repr(C)]
pub struct _IP_INTERFACE_INFO {
    pub NumAdapters: ::core::ffi::c_int,
    pub Adapter: [IP_ADAPTER_INDEX_MAP; 1usize],
}
#[repr(C)]
pub struct _MIB_IFTABLE {
    pub table: [MIB_IFROW; 1usize],
}
"""
    decls = flex_array_decls(source)
    assert decls.fields["Adapter"] == "IP_ADAPTER_INDEX_MAP"
    assert decls.fields["table"] == "MIB_IFROW"

    # The unsuffixed spelling must keep working too.
    assert flex_array_decls("pub struct S { pub f: [u8; 1], }").fields["f"] == "u8"
    # A non-1 length is still not a flexible array.
    assert "f" not in flex_array_decls("pub struct S { pub f: [u8; 8usize], }").fields


# --- rule: anonymous-union field routing ------------------------------------------------------

# The real prsht.h shape: one struct with two nested anonymous unions, one of whose members
# (`pszIcon`) is ALSO spelled directly on the outer struct in the C header's other arm.
_PSH_ISLAND = b"""
#[repr(C)]
pub struct _PROPSHEETHEADERW {
    pub dwSize: DWORD,
    pub dwFlags: DWORD,
    pub __bindgen_anon_1: _PROPSHEETHEADERW__bindgen_ty_1,
    pub pszCaption: LPCWSTR,
    pub nPages: UINT,
    pub __bindgen_anon_2: _PROPSHEETHEADERW__bindgen_ty_2,
}
#[repr(C)]
pub union _PROPSHEETHEADERW__bindgen_ty_1 {
    pub hIcon: HICON,
    pub pszIcon: LPCWSTR,
}
#[repr(C)]
pub union _PROPSHEETHEADERW__bindgen_ty_2 {
    pub nStartPage: UINT,
    pub pStartPage: LPCWSTR,
}
"""


def _no_field_diag(field: str, ty: str, file_name: str, at: tuple[int, int], line: int = 1) -> dict:
    return _diag(
        f"no field `{field}` on type `{ty}`",
        code="E0609",
        spans=[_span(file_name, at[0], at[1], is_primary=True, line=line)],
    )


def test_anon_union_field_is_routed_through_the_bindgen_union():
    src = b"pub unsafe fn f(psh: &mut _PROPSHEETHEADERW) {\n    psh.nStartPage = 0;\n}\n"
    at = (src.index(b"nStartPage"), src.index(b"nStartPage") + len(b"nStartPage"))
    plan = propose_edits(
        [_no_field_diag("nStartPage", "_PROPSHEETHEADERW", "src/a.rs", at, line=2)],
        _reader({"src/a.rs": src}),
        target="i386",
        rules=[RULE_ANON_UNION],
        anon_unions=harvest_anon_unions({"types_i386.rs": _PSH_ISLAND}),
    )
    assert [e.replacement for e in plan.edits] == ["__bindgen_anon_2.nStartPage"]
    assert plan.edits[0].original == "nStartPage"
    assert not plan.red


def test_anon_union_rule_reads_through_a_raw_pointer_type():
    """rustc spells the type `*mut S` when the access is through a pointer — same struct."""
    src = b"pub unsafe fn f(p: *mut _PROPSHEETHEADERW) {\n    (*p).hIcon = h;\n}\n"
    at = (src.index(b"hIcon"), src.index(b"hIcon") + len(b"hIcon"))
    plan = propose_edits(
        [_no_field_diag("hIcon", "*mut _PROPSHEETHEADERW", "src/a.rs", at, line=2)],
        _reader({"src/a.rs": src}),
        target="i386",
        rules=[RULE_ANON_UNION],
        anon_unions=harvest_anon_unions({"types_i386.rs": _PSH_ISLAND}),
    )
    assert [e.replacement for e in plan.edits] == ["__bindgen_anon_1.hIcon"]


def test_anon_union_rule_declines_a_struct_literal_field():
    """E0560 looks like the same mistake but `S { a.b: v }` is not legal Rust.

    Regression guard: the hand-rolled version of this pass rewrote E0560 sites and every one
    came back a parse error on the next pass.
    """
    src = b"pub fn f() -> _PROPSHEETHEADERW {\n    _PROPSHEETHEADERW { nStartPage: 0 }\n}\n"
    at = (src.index(b"nStartPage"), src.index(b"nStartPage") + len(b"nStartPage"))
    plan = propose_edits(
        [
            _diag(
                "struct `_PROPSHEETHEADERW` has no field named `nStartPage`",
                code="E0560",
                spans=[_span("src/a.rs", *at, is_primary=True, line=2)],
            )
        ],
        _reader({"src/a.rs": src}),
        target="i386",
        rules=[RULE_ANON_UNION],
        anon_unions=harvest_anon_unions({"types_i386.rs": _PSH_ISLAND}),
    )
    assert not plan.edits
    assert [item.reason for item in plan.skipped] == ["struct_literal_field"]
    # Declining is not claiming: the diagnostic still has to show up as RED for a human.
    assert [item.code for item in plan.red] == ["E0560"]


def test_anon_union_map_never_shadows_an_outer_field():
    """C's anonymous-member injection loses to an outer member of the same name."""
    island = b"""
pub struct S {
    pub x: DWORD,
    pub __bindgen_anon_1: S__bindgen_ty_1,
}
pub union S__bindgen_ty_1 {
    pub x: UINT,
    pub y: UINT,
}
"""
    amap = harvest_anon_unions({"types_i386.rs": island})
    assert ("S", "x") not in amap
    assert amap[("S", "y")] == "__bindgen_anon_1.y"


def test_anon_union_map_drops_a_field_the_arches_route_differently():
    """Three islands, three bindgen runs: numbering need not agree, and a guess is not allowed."""
    i386 = b"""
pub struct S {
    pub __bindgen_anon_1: A,
}
pub union A {
    pub y: UINT,
}
"""
    amd64 = b"""
pub struct S {
    pub __bindgen_anon_2: A,
}
pub union A {
    pub y: UINT,
}
"""
    assert harvest_anon_unions({"i.rs": i386})[("S", "y")] == "__bindgen_anon_1.y"
    assert ("S", "y") not in harvest_anon_unions({"i.rs": i386, "a.rs": amd64})


def test_anon_union_map_drops_a_field_that_is_direct_on_another_arch():
    """Direct on i386, unionised on amd64: routing it would break the arch that compiles."""
    i386 = b"pub struct S {\n    pub y: UINT,\n}\n"
    amd64 = b"""
pub struct S {
    pub __bindgen_anon_1: A,
}
pub union A {
    pub y: UINT,
}
"""
    assert harvest_anon_unions({"a.rs": amd64})[("S", "y")] == "__bindgen_anon_1.y"
    assert ("S", "y") not in harvest_anon_unions({"i.rs": i386, "a.rs": amd64})


def test_anon_union_rule_declines_a_span_that_is_not_the_field():
    src = b"pub unsafe fn f(psh: &mut _PROPSHEETHEADERW) {\n    psh.nStartPage = 0;\n}\n"
    plan = propose_edits(
        [_no_field_diag("nStartPage", "_PROPSHEETHEADERW", "src/a.rs", (0, 3), line=1)],
        _reader({"src/a.rs": src}),
        target="i386",
        rules=[RULE_ANON_UNION],
        anon_unions=harvest_anon_unions({"types_i386.rs": _PSH_ISLAND}),
    )
    assert not plan.edits
    assert [item.reason for item in plan.skipped] == ["span_is_not_the_field"]


def test_anon_union_rule_without_a_map_contributes_nothing():
    """No island parsed -> the rule stands down; it never invents `__bindgen_anon_1`."""
    src = b"pub unsafe fn f(psh: &mut _PROPSHEETHEADERW) {\n    psh.nStartPage = 0;\n}\n"
    at = (src.index(b"nStartPage"), src.index(b"nStartPage") + len(b"nStartPage"))
    plan = propose_edits(
        [_no_field_diag("nStartPage", "_PROPSHEETHEADERW", "src/a.rs", at, line=2)],
        _reader({"src/a.rs": src}),
        target="i386",
        rules=[RULE_ANON_UNION],
    )
    assert not plan.edits
    assert [item.code for item in plan.red] == ["E0609"]


# --- rule: match-scrutinee cast ---------------------------------------------------------------

_MATCH_SRC = b"""pub unsafe fn f(code: UINT) -> i32 {
    match code {
        PSN_APPLY => 1,
        PSN_RESET => 2,
        _ => 0,
    }
}
"""


def _arm_diag(arm: bytes, expected: str, found: str, src: bytes = _MATCH_SRC) -> dict:
    at = src.index(arm)
    return _diag(
        "mismatched types",
        spans=[
            _span(
                "src/a.rs",
                at,
                at + len(arm),
                is_primary=True,
                label=f"expected `{expected}`, found `{found}`",
                line=3,
            )
        ],
    )


def test_match_arm_mismatch_casts_the_scrutinee_not_the_pattern():
    plan = propose_edits(
        [_arm_diag(b"PSN_APPLY", "u32", "i32")],
        _reader({"src/a.rs": _MATCH_SRC}),
        target="i386",
        rules=[RULE_MATCH_SCRUTINEE],
    )
    assert len(plan.edits) == 1
    edit = plan.edits[0]
    assert edit.original == "code"
    assert edit.replacement == "(code) as i32"
    # The ARM bytes must be untouched: rewriting a constant in a pattern is the §9.112 shape.
    assert _MATCH_SRC[edit.byte_start : edit.byte_end] == b"code"


def test_match_scrutinee_cast_uses_found_not_expected():
    """`expected` is the SCRUTINEE's type; casting to it writes a no-op that burns a pass."""
    plan = propose_edits(
        [_arm_diag(b"PSN_APPLY", "u32", "i32")],
        _reader({"src/a.rs": _MATCH_SRC}),
        target="i386",
        rules=[RULE_MATCH_SCRUTINEE],
    )
    assert plan.edits[0].replacement.endswith("as i32")


def test_match_with_arms_that_disagree_is_left_alone():
    plan = propose_edits(
        [_arm_diag(b"PSN_APPLY", "u32", "i32"), _arm_diag(b"PSN_RESET", "u32", "u16")],
        _reader({"src/a.rs": _MATCH_SRC}),
        target="i386",
        rules=[RULE_MATCH_SCRUTINEE],
    )
    assert not plan.edits
    assert [item.reason for item in plan.skipped] == ["arms_disagree"]
    assert "i32" in plan.skipped[0].detail and "u16" in plan.skipped[0].detail


def test_match_arms_that_agree_produce_exactly_one_edit():
    plan = propose_edits(
        [_arm_diag(b"PSN_APPLY", "u32", "i32"), _arm_diag(b"PSN_RESET", "u32", "i32")],
        _reader({"src/a.rs": _MATCH_SRC}),
        target="i386",
        rules=[RULE_MATCH_SCRUTINEE],
    )
    assert len(plan.edits) == 1


def test_match_scrutinee_already_cast_is_not_cast_again():
    """Otherwise the cast stack grows once per pass and the loop never reaches a fixpoint."""
    src = _MATCH_SRC.replace(b"match code {", b"match (code) as i32 {")
    plan = propose_edits(
        [_arm_diag(b"PSN_APPLY", "u32", "i32", src=src)],
        _reader({"src/a.rs": src}),
        target="i386",
        rules=[RULE_MATCH_SCRUTINEE],
    )
    assert not plan.edits


def test_match_scrutinee_edit_needs_cross_target_agreement():
    """`found` is a resolved primitive, so one target's answer can bake that arch's width."""
    plan = propose_edits(
        [_arm_diag(b"PSN_APPLY", "u32", "i32")],
        _reader({"src/a.rs": _MATCH_SRC}),
        target="i386",
        rules=[RULE_MATCH_SCRUTINEE],
    )
    assert plan.edits[0].requires_target_agreement
    merged = merge_plans({"i386": plan})
    assert not merged.edits
    assert [item.reason for item in merged.withheld] == ["width_unproven"]


def test_non_arm_mismatch_is_not_treated_as_a_scrutinee():
    """A plain argument mismatch has no enclosing match; the rule must not invent one."""
    plan = propose_edits(
        [_arg_diag()], _reader({"src/a.rs": _ARG_SRC}), target="amd64", rules=[RULE_MATCH_SCRUTINEE]
    )
    assert not plan.edits
    assert [item.code for item in plan.red] == ["E0308"]


# --- rule: packed-field pointer ---------------------------------------------------------------

_PACKED_SRC = b"""pub unsafe fn f(d: *mut SP_DRVINFO_DETAIL_DATA_W) -> *const WCHAR {
    (*d).HardwareID.as_ptr()
}
"""


def _packed_diag(place: bytes, src: bytes = _PACKED_SRC) -> dict:
    at = src.index(place)
    return _diag(
        "reference to packed field is unaligned",
        code="E0793",
        spans=[_span("src/a.rs", at, at + len(place), is_primary=True, line=2)],
    )


def test_packed_field_as_ptr_becomes_addr_of():
    plan = propose_edits(
        [_packed_diag(b"(*d).HardwareID")],
        _reader({"src/a.rs": _PACKED_SRC}),
        target="i386",
        rules=[RULE_PACKED_FIELD_PTR],
    )
    assert [e.replacement for e in plan.edits] == ["core::ptr::addr_of!((*d).HardwareID).cast()"]
    edit = plan.edits[0]
    # The `.as_ptr()` call must be consumed, not left dangling after the macro.
    assert _PACKED_SRC[edit.byte_start : edit.byte_end] == b"(*d).HardwareID.as_ptr()"
    assert not edit.requires_target_agreement  # no width is baked


def test_packed_field_as_mut_ptr_uses_addr_of_mut():
    src = _PACKED_SRC.replace(b".as_ptr()", b".as_mut_ptr()")
    plan = propose_edits(
        [_packed_diag(b"(*d).HardwareID", src=src)],
        _reader({"src/a.rs": src}),
        target="i386",
        rules=[RULE_PACKED_FIELD_PTR],
    )
    assert [e.replacement for e in plan.edits] == [
        "core::ptr::addr_of_mut!((*d).HardwareID).cast()"
    ]


def test_packed_field_that_is_not_an_as_ptr_call_is_declined():
    """E0793 also fires on a plain `&packed.field`, which this rewrite would corrupt."""
    src = b"pub unsafe fn f(d: *mut S) {\n    g(&(*d).field);\n}\n"
    at = src.index(b"(*d).field")
    plan = propose_edits(
        [
            _diag(
                "reference to packed field is unaligned",
                code="E0793",
                spans=[_span("src/a.rs", at, at + len(b"(*d).field"), is_primary=True, line=2)],
            )
        ],
        _reader({"src/a.rs": src}),
        target="i386",
        rules=[RULE_PACKED_FIELD_PTR],
    )
    assert not plan.edits
    assert [item.reason for item in plan.skipped] == ["not_an_as_ptr_call"]


# --- rule: index cast to usize ----------------------------------------------------------------

_INDEX_SRC = b"pub fn f(t: &[u8], i: UINT) -> u8 {\n    t[i]\n}\n"


def _index_diag(index_type: str = "u32", place: bytes = b"i", src: bytes = _INDEX_SRC) -> dict:
    at = src.index(place, src.index(b"t["))
    return _diag(
        f"the type `[u8]` cannot be indexed by `{index_type}`",
        code="E0277",
        spans=[_span("src/a.rs", at, at + len(place), is_primary=True, line=2)],
    )


def test_non_usize_index_is_cast_to_usize():
    plan = propose_edits(
        [_index_diag()], _reader({"src/a.rs": _INDEX_SRC}), target="i386", rules=[RULE_INDEX_USIZE]
    )
    assert [e.replacement for e in plan.edits] == ["(i) as usize"]
    # `usize` is the width-varying alias itself, so one target's answer is portable.
    assert not plan.edits[0].requires_target_agreement


def test_index_by_a_non_integer_is_left_alone():
    """E0277 also covers `map[key]`; only an integer index has an `as usize` answer."""
    plan = propose_edits(
        [_index_diag(index_type="String")],
        _reader({"src/a.rs": _INDEX_SRC}),
        target="i386",
        rules=[RULE_INDEX_USIZE],
    )
    assert not plan.edits
    assert [item.code for item in plan.red] == ["E0277"]


def test_index_already_cast_is_not_cast_again():
    src = _INDEX_SRC.replace(b"t[i]", b"t[(i) as usize]")
    plan = propose_edits(
        [_index_diag(place=b"(i) as usize", src=src)],
        _reader({"src/a.rs": src}),
        target="i386",
        rules=[RULE_INDEX_USIZE],
    )
    assert not plan.edits


# --- rule: a header macro called as a function -------------------------------------------------

_MACRO_SRC = b"pub unsafe fn f(a: &[u8]) -> u32 {\n    _countof(a)\n}\n"


def _found_macro_diag(name: bytes, src: bytes, line: int = 2) -> dict:
    at = src.index(name)
    return _diag(
        f"expected function, found macro `{name.decode()}`",
        code="E0423",
        spans=[_span("src/a.rs", at, at + len(name), is_primary=True, line=line)],
    )


def test_a_macro_called_as_a_function_gets_its_bang():
    plan = propose_edits(
        [_found_macro_diag(b"_countof", _MACRO_SRC)],
        _reader({"src/a.rs": _MACRO_SRC}),
        target="i386",
        rules=[RULE_MACRO_CALL],
    )
    assert [e.replacement for e in plan.edits] == ["_countof!"]
    assert not plan.edits[0].requires_target_agreement


def test_the_long_rustc_wording_is_matched_too():
    """rustc says "expected function, tuple struct or tuple variant, found macro" at most sites.

    Regression: matching only the short wording caught 1 of 61 sites on sysdm.cpl, and a rule
    that fires once looks like a rule with little to do rather than a broken pattern.
    """
    at = _MACRO_SRC.index(b"_countof")
    diagnostic = _diag(
        "expected function, tuple struct or tuple variant, found macro `_countof`",
        code="E0423",
        spans=[_span("src/a.rs", at, at + 8, is_primary=True, line=2)],
    )
    plan = propose_edits(
        [diagnostic], _reader({"src/a.rs": _MACRO_SRC}), target="i386", rules=[RULE_MACRO_CALL]
    )
    assert [e.replacement for e in plan.edits] == ["_countof!"]


def test_a_macro_not_in_call_position_is_declined():
    """Appending `!` where no `(` follows does not even parse."""
    src = b"pub fn f() -> u32 {\n    let x = _countof;\n    0\n}\n"
    plan = propose_edits(
        [_found_macro_diag(b"_countof", src)],
        _reader({"src/a.rs": src}),
        target="i386",
        rules=[RULE_MACRO_CALL],
    )
    assert not plan.edits
    assert [item.reason for item in plan.skipped] == ["not_a_call"]


# --- rule: C `sizeof(TYPE)` that lost its operator ---------------------------------------------


def _found_type_diag(name: bytes, src: bytes, line: int = 2) -> dict:
    at = src.index(name)
    return _diag(
        f"expected value, found type alias `{name.decode()}`",
        code="E0423",
        spans=[_span("src/a.rs", at, at + len(name), is_primary=True, line=line)],
    )


def test_bare_type_name_in_value_position_becomes_size_of():
    src = b"pub unsafe fn f(b: &mut B) {\n    b.biSize = BITMAPINFO;\n}\n"
    plan = propose_edits(
        [_found_type_diag(b"BITMAPINFO", src)],
        _reader({"src/a.rs": src}),
        target="i386",
        rules=[RULE_SIZEOF_TYPE],
    )
    assert [e.replacement for e in plan.edits] == ["core::mem::size_of::<BITMAPINFO>()"]


def test_size_of_val_wrapper_is_swallowed_whole():
    """Rewriting only the operand leaves `size_of_val(&size_of::<T>())` — the size of a usize.

    That compiles, and is wrong on every arch, which is the worst of both worlds.
    """
    src = b"pub unsafe fn f(n: &mut N) {\n    n.cbSize = core::mem::size_of_val(&NONCLIENTMETRICS) as u32;\n}\n"
    plan = propose_edits(
        [_found_type_diag(b"NONCLIENTMETRICS", src)],
        _reader({"src/a.rs": src}),
        target="i386",
        rules=[RULE_SIZEOF_TYPE],
    )
    assert [e.replacement for e in plan.edits] == ["core::mem::size_of::<NONCLIENTMETRICS>()"]
    edit = plan.edits[0]
    assert src[edit.byte_start : edit.byte_end] == b"core::mem::size_of_val(&NONCLIENTMETRICS)"


def test_a_macro_type_argument_is_not_a_lost_sizeof():
    """setuplib: `RTL_NUMBER_OF_FIELD(VOLINFO, DeviceName)` became `size_of::<VOLINFO>()`."""
    for call in (
        b"RTL_NUMBER_OF_FIELD(VOLINFO, DeviceName)",
        b"RTL_NUMBER_OF_FIELD!(VOLINFO, DeviceName)",
    ):
        src = b"pub fn f() -> usize {\n    " + call + b" + 1\n}\n"
        plan = propose_edits(
            [_found_type_diag(b"VOLINFO", src)],
            _reader({"src/a.rs": src}),
            target="i386",
            rules=[RULE_SIZEOF_TYPE],
        )
        assert not plan.edits
        assert [s.reason for s in plan.skipped] == ["macro_type_argument"]
