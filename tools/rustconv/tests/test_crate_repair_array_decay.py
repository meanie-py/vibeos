"""RULE_ARRAY_DECAY: ``[T; N]`` where a ``*T`` is wanted -> ``.as_ptr()`` / ``.as_mut_ptr()``.

C decays an array argument to a pointer for free; Rust does not, so every translated module
arrives with a pile of ``expected `*mut u16`, found `[u16; 80]``` and has historically paid for
them by hand. intl alone carried ~40.

The load-bearing test here is ``element_type_mismatch_stays_red``. Decay is only decay when the
element type is IDENTICAL to the pointee — ``[u16; N]`` behind a ``*mut u8`` is a reinterpret,
the same shape as the wide-literal bug class, and a rule that smoothed it over would bless
exactly the bug we spent five modules chasing. Every other guard likewise has a test that fails
if the guard is deleted.
"""

from __future__ import annotations

from rustconv.crate_repair import (
    RULE_ARRAY_DECAY,
    RULE_POINTER_CAST,
    merge_plans,
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


def _plan(src: bytes, diags: list[dict], *, target: str = "i386", name: str = "src/a.rs"):
    return propose_edits(diags, lambda _n: src if _n == name else None, target=target)


def test_mut_pointer_wants_as_mut_ptr() -> None:
    src = b"unsafe fn f() { GetWindowTextW(hwnd, szBuffer, 80); }"
    plan = _plan(src, [_site(src, b"szBuffer", "*mut u16", "[u16; 80]")])
    assert [edit.replacement for edit in plan.edits] == ["(szBuffer).as_mut_ptr()"]
    assert plan.edits[0].rule == RULE_ARRAY_DECAY


def test_const_pointer_wants_as_ptr() -> None:
    src = b"unsafe fn f() { lstrcmpW(szBuffer, other); }"
    plan = _plan(src, [_site(src, b"szBuffer", "*const u16", "[u16; 80]")])
    assert [edit.replacement for edit in plan.edits] == ["(szBuffer).as_ptr()"]


def test_element_type_mismatch_stays_red() -> None:
    """THE guard. `[u16; N]` -> `*mut u8` is a reinterpret, not a decay."""
    src = b"unsafe fn f() { g(szBuffer); }"
    plan = _plan(src, [_site(src, b"szBuffer", "*mut u8", "[u16; 80]")])
    assert plan.edits == ()
    assert [item.reason for item in plan.skipped] == ["element_type_mismatch"]


def test_void_pointer_is_not_decayed_here() -> None:
    """`*mut c_void` is decay PLUS a conversion; this rule refuses to guess the second step."""
    src = b"unsafe fn f() { memcpy_like(buf, n); }"
    plan = _plan(src, [_site(src, b"buf", "*mut c_void", "[u16; 8]")])
    assert plan.edits == ()
    assert [item.reason for item in plan.skipped] == ["element_type_mismatch"]


def test_field_access_array_decays() -> None:
    """The array need not be a bare identifier — anchor on the span, not an ident pattern."""
    src = b"unsafe fn f() { g((*pGlobalData).szBuffer); }"
    plan = _plan(src, [_site(src, b"(*pGlobalData).szBuffer", "*mut u16", "[u16; 32]")])
    assert [edit.replacement for edit in plan.edits] == ["((*pGlobalData).szBuffer).as_mut_ptr()"]


def test_assignment_target_stays_red() -> None:
    src = b"unsafe fn f() { szBuffer = other; }"
    plan = _plan(src, [_site(src, b"szBuffer", "*mut u16", "[u16; 80]")])
    assert plan.edits == ()
    assert [item.reason for item in plan.skipped] == ["assignment_target"]


def test_already_decayed_stays_red() -> None:
    src = b"unsafe fn f() { g(szBuffer.as_mut_ptr()); }"
    plan = _plan(src, [_site(src, b"szBuffer.as_mut_ptr()", "*mut u16", "[u16; 80]")])
    assert plan.edits == ()
    assert [item.reason for item in plan.skipped] == ["already_decayed"]


def test_match_pattern_stays_red() -> None:
    src = b"unsafe fn f() { match x { szBuffer => 1, _ => 0 }; }"
    plan = _plan(src, [_site(src, b"szBuffer", "*mut u16", "[u16; 80]")])
    assert plan.edits == ()
    assert [item.reason for item in plan.skipped] == ["match_pattern"]


def test_callee_span_stays_red() -> None:
    src = b"unsafe fn f() { szBuffer(1); }"
    plan = _plan(src, [_site(src, b"szBuffer", "*mut u16", "[u16; 80]")])
    assert plan.edits == ()
    assert [item.reason for item in plan.skipped] == ["callee_span"]


def test_pointer_cast_rule_does_not_claim_an_array() -> None:
    """The two rules answer disjoint shapes; only the decay rule may fire on `found [T; N]`."""
    src = b"unsafe fn f() { g(szBuffer); }"
    plan = _plan(src, [_site(src, b"szBuffer", "*mut u16", "[u16; 80]")])
    assert {edit.rule for edit in plan.edits} == {RULE_ARRAY_DECAY}
    assert RULE_POINTER_CAST not in {edit.rule for edit in plan.edits}


def test_decay_survives_single_target_merge() -> None:
    """Decay is width-INDEPENDENT, so one target proposing it is sufficient authority.

    This is the deliberate exception to the module's cross-target discipline: `as_ptr` names no
    type and bakes no width, so there is nothing for a second target to corroborate. If this
    ever starts being withheld as `width_unproven`, the flag was set wrong.
    """
    src = b"unsafe fn f() { g(szBuffer); }"
    plan = _plan(src, [_site(src, b"szBuffer", "*mut u16", "[u16; 80]")])
    merged = merge_plans({"i386": plan})
    assert [edit.replacement for edit in merged.edits] == ["(szBuffer).as_mut_ptr()"]
    assert merged.withheld == ()


def test_decay_is_not_reported_as_red() -> None:
    """A claimed diagnostic must not also land in `red` — that would double-count the site."""
    src = b"unsafe fn f() { g(szBuffer); }"
    plan = _plan(src, [_site(src, b"szBuffer", "*mut u16", "[u16; 80]")])
    assert plan.red == ()


# --- E0605: the same decay wearing C's explicit cast, `(LPARAM)szBuffer` ----------------------


def _cast_diag(src: bytes, needle: bytes, found: str, target: str, name: str = "src/a.rs") -> dict:
    start = src.index(needle)
    return {
        "$message_type": "diagnostic",
        "message": f"non-primitive cast: `{found}` as `{target}`",
        "code": {"code": "E0605", "explanation": None},
        "level": "error",
        "spans": [_span(name, start, start + len(needle), primary=True)],
        "children": [],
        "rendered": None,
    }


def test_cast_to_pointer_wide_int_decays_and_keeps_source_spelling() -> None:
    """`i32` is what rustc RESOLVED; `LPARAM` is what the source says and must survive."""
    src = b"unsafe fn f() { SendMessageW(h, m, 0, lang as LPARAM); }"
    plan = _plan(src, [_cast_diag(src, b"lang as LPARAM", "[u16; 255]", "i32")])
    assert [edit.replacement for edit in plan.edits] == ["(lang).as_ptr() as LPARAM"]
    assert plan.edits[0].requires_target_agreement is True


def test_cast_to_matching_pointer_decays_without_agreement() -> None:
    src = b"unsafe fn f() { g(buf as *mut u16); }"
    plan = _plan(src, [_cast_diag(src, b"buf as *mut u16", "[u16; 8]", "*mut u16")])
    assert [edit.replacement for edit in plan.edits] == ["(buf).as_mut_ptr() as *mut u16"]
    assert plan.edits[0].requires_target_agreement is False


def test_cast_to_mismatched_pointer_is_honoured_not_refused() -> None:
    """THE ASYMMETRY. A written cast states the reinterpret; an argument conversion does not.

    `(BYTE *)szBuffer` is C's own spelling and reproducing it is faithfulness. Compare
    `test_element_type_mismatch_stays_red`: the identical type pair in ARGUMENT position stays
    red, because there nothing in the source claims a reinterpret and the mismatch is instead
    evidence that a declaration is wrong.
    """
    src = b"unsafe fn f() { g(buf as *mut u8); }"
    plan = _plan(src, [_cast_diag(src, b"buf as *mut u8", "[u16; 8]", "*mut u8")])
    assert [edit.replacement for edit in plan.edits] == ["(buf).as_mut_ptr() as *mut u8"]
    assert "SOURCE CAST states" in plan.edits[0].authority


def test_cast_to_void_pointer_decays() -> None:
    """`ZeroMemory!(buf as *mut c_void, ..)` — the commonest shape of the above in this tree."""
    src = b"unsafe fn f() { ZeroMemory(buf as *mut c_void, n); }"
    plan = _plan(src, [_cast_diag(src, b"buf as *mut c_void", "[u16; 8]", "*mut c_void")])
    assert [edit.replacement for edit in plan.edits] == ["(buf).as_mut_ptr() as *mut c_void"]


def test_byte_reinterpret_cast_is_withheld_by_target_disagreement() -> None:
    """`[u8; 4] as u32` is pointer-wide on i386 ONLY. Agreement is what unmasks it.

    This is the guard that stops the cast branch turning a byte-reinterpret into an ADDRESS.
    i386 proposes; amd64 (8-byte pointers) declines; `merge_plans` withholds rather than
    applying a 32-bit-only inference.
    """
    src = b"unsafe fn f() { g(raw as u32); }"
    diag = [_cast_diag(src, b"raw as u32", "[u8; 4]", "u32")]
    i386 = _plan(src, diag, target="i386")
    amd64 = _plan(src, diag, target="amd64")
    assert [edit.replacement for edit in i386.edits] == ["(raw).as_ptr() as u32"]
    assert amd64.edits == ()
    assert [item.reason for item in amd64.skipped] == ["cast_target_not_pointer_wide"]
    merged = merge_plans({"i386": i386, "amd64": amd64})
    assert merged.edits == ()
    assert [item.reason for item in merged.withheld] == ["width_unproven"]


def test_lparam_cast_survives_agreement_across_widths() -> None:
    """The counterpart: a real LPARAM resolves to i32/i64 per arch but ONE source replacement."""
    src = b"unsafe fn f() { SendMessageW(h, m, 0, lang as LPARAM); }"
    i386 = _plan(src, [_cast_diag(src, b"lang as LPARAM", "[u16; 255]", "i32")], target="i386")
    amd64 = _plan(src, [_cast_diag(src, b"lang as LPARAM", "[u16; 255]", "i64")], target="amd64")
    arm64 = _plan(src, [_cast_diag(src, b"lang as LPARAM", "[u16; 255]", "i64")], target="arm64")
    merged = merge_plans({"i386": i386, "amd64": amd64, "arm64": arm64})
    assert [edit.replacement for edit in merged.edits] == ["(lang).as_ptr() as LPARAM"]
    assert merged.withheld == ()


def test_nested_cast_splits_at_the_outer_operator() -> None:
    src = b"unsafe fn f() { g(h(x as u32) as LPARAM); }"
    plan = _plan(src, [_cast_diag(src, b"h(x as u32) as LPARAM", "[u16; 4]", "i32")])
    assert [edit.replacement for edit in plan.edits] == ["(h(x as u32)).as_ptr() as LPARAM"]


# --- rustc's multi-argument rendering ---------------------------------------------------------


def test_multi_argument_mismatch_is_seen_through_child_notes() -> None:
    """2+ bad arguments => rustc drops the labels and moves the types into child notes.

    Every rule in crate_repair reads `_mismatch_sites`, so before this was handled a plain
    `wcscpy(dst, src)` with both arguments undecayed was invisible to ALL of them. This is a
    verbatim reduction of what rustc 1.92 emitted for intl-rust/src/intl/currency.rs:317.
    """
    src = b"unsafe fn f() { wcscpy(\n        dstBuffer,\n        srcBuffer,\n    ); }"
    dst_at = src.index(b"dstBuffer")
    src_at = src.index(b"srcBuffer")
    diag = {
        "$message_type": "diagnostic",
        "message": "arguments to this function are incorrect",
        "code": {"code": "E0308", "explanation": None},
        "level": "error",
        "spans": [_span("src/a.rs", src.index(b"wcscpy"), src.index(b"wcscpy") + 6, primary=True)],
        "children": [
            {
                "level": "note",
                "message": "expected `*mut u16`, found `[u16; 13]`",
                "code": None,
                "spans": [_span("src/a.rs", dst_at, dst_at + 9)],
                "children": [],
                "rendered": None,
            },
            {
                "level": "note",
                "message": "expected raw pointer `*mut u16`\n         found array `[u16; 13]`",
                "code": None,
                "spans": [],
                "children": [],
                "rendered": None,
            },
            {
                "level": "note",
                "message": "expected `*const u16`, found `[u16; 13]`",
                "code": None,
                "spans": [_span("src/a.rs", src_at, src_at + 9)],
                "children": [],
                "rendered": None,
            },
        ],
        "rendered": None,
    }
    plan = _plan(src, [diag])
    assert [edit.replacement for edit in plan.edits] == [
        "(dstBuffer).as_mut_ptr()",
        "(srcBuffer).as_ptr()",
    ]
    assert plan.red == ()


def test_labelled_span_and_child_note_do_not_double_count() -> None:
    """The single-argument shape carries BOTH; the same span must yield one site, not two."""
    src = b"unsafe fn f() { g(szBuffer); }"
    at = src.index(b"szBuffer")
    diag = _site(src, b"szBuffer", "*mut u16", "[u16; 80]")
    diag["children"] = [
        {
            "level": "note",
            "message": "expected `*mut u16`, found `[u16; 80]`",
            "code": None,
            "spans": [_span("src/a.rs", at, at + 8)],
            "children": [],
            "rendered": None,
        }
    ]
    plan = _plan(src, [diag])
    assert [edit.replacement for edit in plan.edits] == ["(szBuffer).as_mut_ptr()"]
