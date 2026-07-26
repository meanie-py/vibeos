"""C `goto` cleanup has no 1:1 Rust lowering — the tail replicates per jump site.

`SHELL32_FileOperation` (2 `goto cleanup`) and `SHFileOperationA` (3) were both
rejected for "not exactly once" on every call in the cleanup tail, while being
faithful translations. One rejected worker assembles no file, so this took the
whole TU with it.
"""

import pytest

from rustconv.call_seal import CallSealError, seal_calls, unseal_calls

C_BODY = """int f(void) {
    int err = Setup();
    if (err) goto cleanup;
    err = More();
    if (err) goto cleanup;
    Work();
cleanup:
    Release(a);
    Report(err);
    return err;
}"""

CALLS = {"Setup", "More", "Work", "Release", "Report"}


def _inlined_tail(n: int) -> str:
    """What a structured lowering emits: the tail once per jump plus fallthrough."""
    tail = "__RUSTCONV_CALL_0003__(a); __RUSTCONV_CALL_0004__(err); return err;"
    return (
        "unsafe fn f() -> i32 { let mut err = __RUSTCONV_CALL_0000__();\n"
        + f"if err != 0 {{ {tail} }}\n"
        + "err = __RUSTCONV_CALL_0001__();\n"
        + (f"if err != 0 {{ {tail} }}\n" if n > 1 else "")
        + "__RUSTCONV_CALL_0002__();\n"
        + tail
        + "}"
    )


def test_cleanup_tail_may_repeat_once_per_goto_plus_fallthrough():
    sealed = seal_calls(C_BODY, CALLS)
    restored = unseal_calls(_inlined_tail(2), sealed)
    assert restored.count("Release(a)") == 3
    assert restored.count("Setup()") == 1


def test_whole_remainder_duplication_is_accepted_too():
    """The third observed shape: a nested cascade, not just the label's tail.

    A count-based budget (`#goto + 1`, tail only) rejected this; the rule is
    positional because a lowering is not free to pick a consistent shape.
    """
    sealed = seal_calls(C_BODY, CALLS)
    cascade = _inlined_tail(2).replace(
        "__RUSTCONV_CALL_0002__();",
        "__RUSTCONV_CALL_0002__(); __RUSTCONV_CALL_0002__(); __RUSTCONV_CALL_0001__();",
    )
    assert "Work()" in unseal_calls(cascade, sealed)


def test_a_call_before_the_first_goto_stays_exactly_once():
    """Duplication can only begin where the `goto` does; the entry path is linear."""
    sealed = seal_calls(C_BODY, CALLS)
    doubled_prefix = _inlined_tail(2).replace(
        "let mut err = __RUSTCONV_CALL_0000__();",
        "let mut err = __RUSTCONV_CALL_0000__(); __RUSTCONV_CALL_0000__();",
    )
    with pytest.raises(CallSealError, match="not exactly once"):
        unseal_calls(doubled_prefix, sealed)


def test_the_callee_name_written_out_stands_in_inside_the_region():
    """A model that has inlined the tail 4× stops threading tokens and writes names.

    `HeapFree(GetProcessHeap(), 0, p)` is character-for-character what unsealing
    would have produced, so this is not a dropped call.
    """
    sealed = seal_calls(C_BODY, CALLS)
    named = _inlined_tail(2).replace("__RUSTCONV_CALL_0003__(a)", "Release(a)")
    assert unseal_calls(named, sealed).count("Release(a)") == 3


def test_a_wrong_name_does_not_stand_in():
    sealed = seal_calls(C_BODY, CALLS)
    wrong = _inlined_tail(2).replace("__RUSTCONV_CALL_0003__(a)", "RtlRelease(a)")
    with pytest.raises(CallSealError, match="missing/non-executable"):
        unseal_calls(wrong, sealed)


def test_outside_the_region_a_written_out_name_needs_an_exact_count():
    """Not a new relaxation: the exact-count recovery for a written-out name predates
    the goto rule. What the goto rule adds is standing in when replication has made
    the count ambiguous — so record the older boundary here too."""
    sealed = seal_calls(C_BODY, CALLS)
    named = _inlined_tail(2).replace("__RUSTCONV_CALL_0000__()", "Setup()", 1)
    assert "Setup()" in unseal_calls(named, sealed)

    ambiguous = named.replace("__RUSTCONV_CALL_0001__()", "Setup()", 1)
    with pytest.raises(CallSealError, match="missing/non-executable"):
        unseal_calls(ambiguous, sealed)


def test_an_invented_token_is_still_rejected_inside_the_region():
    """Relaxing duplicates must not relax `invented` — it caught 3 real ones."""
    sealed = seal_calls(C_BODY, CALLS)
    hallucinated = _inlined_tail(2).replace(
        "__RUSTCONV_CALL_0004__(err); return err;",
        "__RUSTCONV_CALL_0004__(err); __RUSTCONV_CALL_0009__(err); return err;",
    )
    with pytest.raises(CallSealError, match="invented"):
        unseal_calls(hallucinated, sealed)


def test_a_dropped_cleanup_call_is_still_caught():
    """Relaxing duplicates must not relax "missing" — that is the real signal."""
    sealed = seal_calls(C_BODY, CALLS)
    dropped = _inlined_tail(2).replace("__RUSTCONV_CALL_0003__(a); ", "")
    with pytest.raises(CallSealError, match="missing/non-executable"):
        unseal_calls(dropped, sealed)


def test_a_body_without_goto_keeps_exactly_once():
    sealed = seal_calls("int g(void) { Release(a); return 0; }", {"Release"})
    with pytest.raises(CallSealError, match="not exactly once"):
        unseal_calls(
            "unsafe fn g() -> i32 { __RUSTCONV_CALL_0000__(a); __RUSTCONV_CALL_0000__(a); 0 }",
            sealed,
        )


def test_switch_case_labels_are_not_goto_targets():
    """`case X:` must not hand out a replication budget."""
    body = """int h(int k) {
    switch (k) { case 1: Release(a); break; default: Report(k); break; }
    return 0;
}"""
    sealed = seal_calls(body, {"Release", "Report"})
    with pytest.raises(CallSealError, match="not exactly once"):
        unseal_calls(
            "unsafe fn h(k: i32) -> i32 { __RUSTCONV_CALL_0000__(a); "
            "__RUSTCONV_CALL_0000__(a); __RUSTCONV_CALL_0001__(k); 0 }",
            sealed,
        )
