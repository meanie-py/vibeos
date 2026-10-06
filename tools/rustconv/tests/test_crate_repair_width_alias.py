"""RULE_WIDTH_ALIAS: answer an int-vs-int E0308 with the callee's declared TYPE NAME.

The rule exists because ``RULE_NUMERIC_CAST``'s answer — the width rustc resolved on the
target it happened to run on — is unprovable for this class. On syssetup, of the 17 integer
sites erroring on all three targets exactly ONE had three-way agreement: the parameter types
genuinely differ per arch (SIZE_T, WPARAM, UINT_PTR), so cross-target voting can never settle
them. The NAME is what is portable.

Each guard below has a test that fails if the guard is deleted. ``not_whole_argument`` is the
one that caught a real mis-fire: rustc labels the narrowest disagreeing OPERAND, so in
``SetAutoDaylight(SendDlgItemMessage(..) != (BST_UNCHECKED) as i64)`` the span sits inside
argument 0 rather than at it, and casting it to ``SetAutoDaylight``'s ``BOOL`` is a well-formed
edit to the wrong expression.
"""

from __future__ import annotations

from rustconv.crate_repair import (
    RULE_NUMERIC_CAST,
    RULE_WIDTH_ALIAS,
    _local_authorities,
    harvest_signatures,
    propose_edits,
)


def _span(file_name: str, start: int, end: int, *, label: str | None = None, primary=False) -> dict:
    return {
        "file_name": file_name,
        "byte_start": start,
        "byte_end": end,
        "line_start": 1,
        "line_end": 1,
        "column_start": 1,
        "column_end": 1,
        "is_primary": primary,
        "text": [],
        "label": label,
        "suggested_replacement": None,
        "suggestion_applicability": None,
        "expansion": None,
    }


def _diag(spans: list[dict]) -> dict:
    return {
        "$message_type": "diagnostic",
        "message": "mismatched types",
        "code": {"code": "E0308", "explanation": None},
        "level": "error",
        "spans": spans,
        "children": [],
        "rendered": None,
    }


def _site(src: bytes, needle: bytes, expected: str, found: str, name: str = "src/a.rs"):
    start = src.index(needle)
    return _diag(
        [
            _span(
                name,
                start,
                start + len(needle),
                label=f"expected `{expected}`, found `{found}`",
                primary=True,
            )
        ]
    )


SIGS = {"HeapAlloc": ["HANDLE", "DWORD", "SIZE_T"]}
ALIASES = frozenset({"SIZE_T", "HANDLE", "DWORD"})


def _plan(src: bytes, diags, **kw):
    return propose_edits(
        diags,
        lambda n: src if n == "src/a.rs" else None,
        target="i686-pc-windows-gnu",
        signatures=kw.pop("signatures", SIGS),
        local_aliases=kw.pop("local_aliases", ALIASES),
        **kw,
    )


def test_casts_to_the_callees_declared_alias():
    src = b"unsafe { HeapAlloc(h, 0, (n) as u64); }\n"
    plan = _plan(src, [_site(src, b"(n) as u64", "u32", "u64")])
    edits = [e for e in plan.edits if e.rule == RULE_WIDTH_ALIAS]
    assert len(edits) == 1
    assert edits[0].replacement == "(n) as SIZE_T"
    # Portable by construction: no width was inferred, so no cross-target vote is needed.
    assert edits[0].requires_target_agreement is False


def test_baked_width_is_replaced_not_stacked():
    src = b"unsafe { HeapAlloc(h, 0, (n) as u64); }\n"
    plan = _plan(src, [_site(src, b"(n) as u64", "u32", "u64")])
    assert "u64" not in next(e for e in plan.edits if e.rule == RULE_WIDTH_ALIAS).replacement


def test_existing_alias_cast_that_still_errors_requires_scope_repair():
    src = b"unsafe { HeapAlloc(h, 0, (n) as SIZE_T); }\n"
    plan = _plan(src, [_site(src, b"(n) as SIZE_T", "u32", "usize")])
    assert not plan.edits
    assert any(s.reason == "alias_scope_mismatch" for s in plan.skipped)


def test_argument_without_a_cast_gets_one():
    src = b"unsafe { HeapAlloc(h, 0, size_of::<T>()); }\n"
    plan = _plan(src, [_site(src, b"size_of::<T>()", "u32", "usize")])
    edit = next(e for e in plan.edits if e.rule == RULE_WIDTH_ALIAS)
    assert edit.replacement == "(size_of::<T>()) as SIZE_T"


def test_numeric_cast_rule_stands_down_on_a_claimed_span():
    src = b"unsafe { HeapAlloc(h, 0, (n) as u64); }\n"
    plan = _plan(src, [_site(src, b"(n) as u64", "u32", "u64")])
    assert not [e for e in plan.edits if e.rule == RULE_NUMERIC_CAST]
    assert any(s.reason == "claimed_by_width_alias" for s in plan.skipped)


def test_sub_expression_of_an_argument_is_refused():
    # The mis-fire this guard exists for: the span is an OPERAND inside argument 0.
    src = b"unsafe { HeapAlloc(h, 0, m != (n) as u64); }\n"
    plan = _plan(src, [_site(src, b"(n) as u64", "u32", "u64")])
    assert not [e for e in plan.edits if e.rule == RULE_WIDTH_ALIAS]
    assert any(s.reason == "not_whole_argument" for s in plan.skipped)


def test_callee_outside_this_crate_is_refused():
    src = b"unsafe { RtlAllocateHeap(h, 0, (n) as u64); }\n"
    plan = _plan(src, [_site(src, b"(n) as u64", "u32", "u64")])
    assert not [e for e in plan.edits if e.rule == RULE_WIDTH_ALIAS]
    assert any(s.reason == "callee_not_local" for s in plan.skipped)


def test_alias_this_crate_never_mentions_is_refused():
    src = b"unsafe { HeapAlloc(h, 0, (n) as u64); }\n"
    plan = _plan(src, [_site(src, b"(n) as u64", "u32", "u64")], local_aliases=frozenset())
    assert not [e for e in plan.edits if e.rule == RULE_WIDTH_ALIAS]
    assert any(s.reason == "alias_not_local" for s in plan.skipped)


def test_primitive_parameter_is_left_to_the_numeric_rule():
    src = b"unsafe { Takes(h, 0, (n) as u64); }\n"
    plan = _plan(
        src,
        [_site(src, b"(n) as u64", "u32", "u64")],
        signatures={"Takes": ["HANDLE", "DWORD", "u32"]},
    )
    assert not [e for e in plan.edits if e.rule == RULE_WIDTH_ALIAS]
    assert [e for e in plan.edits if e.rule == RULE_NUMERIC_CAST]


def test_rule_is_disabled_rather_than_guessing_without_signatures():
    src = b"unsafe { HeapAlloc(h, 0, (n) as u64); }\n"
    plan = _plan(src, [_site(src, b"(n) as u64", "u32", "u64")], signatures={})
    assert not [e for e in plan.edits if e.rule == RULE_WIDTH_ALIAS]


def test_harvest_keeps_the_last_parameter_of_a_one_line_declaration():
    # `_match_forward` returns the index PAST the `)`, so slicing to `close` used to leave the
    # final parameter spelled `SIZE_T)` — which failed every consumer's identifier check.
    sigs = harvest_signatures(
        {"a.rs": b'unsafe extern "C" { pub fn F(a: HANDLE, b: SIZE_T) -> u32; }'}
    )
    assert sigs["F"] == ["HANDLE", "SIZE_T"]


def test_local_authorities_sees_glob_reachable_names(tmp_path):
    # The aliases are not DECLARED by these crates — the island opens with a glob — so a
    # declarations-only sweep finds none and silently disables the rule.
    src = tmp_path / "src"
    src.mkdir()
    (src / "types_i386.rs").write_text(
        "pub use ntdll_sys::nt::*;\n"
        'unsafe extern "C" { pub fn HeapAlloc(h: HANDLE, f: DWORD, n: SIZE_T) -> LPVOID; }\n'
    )
    sigs, names = _local_authorities(tmp_path)
    assert sigs["HeapAlloc"] == ["HANDLE", "DWORD", "SIZE_T"]
    assert {"HANDLE", "DWORD", "SIZE_T"} <= names


def test_comparison_promotes_descriptor_byte_instead_of_truncating_resource_type():
    from rustconv.crate_repair import RULE_COMPARISON

    src = b"fn f() { if descriptor.Type == Type { found(); } }"
    # Locate the RHS, not the same-spelled field on the LHS.
    start = src.index(b"== Type") + 3
    diag = _diag(
        [_span("src/a.rs", start, start + 4, label="expected `u8`, found `i32`", primary=True)]
    )
    plan = _plan(src, [diag])
    assert len(plan.edits) == 1
    assert plan.edits[0].rule == RULE_COMPARISON
    assert plan.edits[0].original == "descriptor.Type"
    assert plan.edits[0].replacement == "(descriptor.Type) as i32"


def test_unknown_comparison_width_never_uses_assignment_truncation():
    src = b"fn f() { if a == b { found(); } }"
    plan = _plan(src, [_site(src, b"b", "u8", "usize")])
    assert not plan.edits
    assert any(s.reason == "promotion_not_proven" for s in plan.skipped)


def test_a_public_ntdll_glob_lends_ntdll_sys_signatures(tmp_path):
    # setuplib: `RtlAllocateHeap(.., Size: SIZE_T)` lives in ntdll-sys, reached through the
    # island's `pub use ntdll_sys::nt::*;` -- the same chain that carries SIZE_T.
    crate, sys = tmp_path / "x-rust", tmp_path / "ntdll-sys"
    (crate / "src").mkdir(parents=True)
    (sys / "src").mkdir(parents=True)
    (sys / "src" / "bindgen_amd64.rs").write_text(
        "pub type SIZE_T = u64;\n"
        'extern "C" { pub fn RtlAllocateHeap(HeapHandle: PVOID, Flags: ULONG, Size: SIZE_T)'
        " -> PVOID; pub fn Local(a: ULONG); }\n"
    )
    (crate / "src" / "types_amd64.rs").write_text(
        'pub use ntdll_sys::nt::*;\nunsafe extern "C" { pub fn Local(a: DWORD); }\n'
    )
    sigs, names = _local_authorities(crate)
    assert sigs["RtlAllocateHeap"] == ["PVOID", "ULONG", "SIZE_T"]
    assert sigs["Local"] == ["DWORD"]  # the crate's own declaration wins
    assert "SIZE_T" in names
    # A private (kernel-style) glob is not followed.
    (crate / "src" / "types_amd64.rs").write_text("use ntdll_sys::nt::*;\n")
    assert "RtlAllocateHeap" not in _local_authorities(crate)[0]
