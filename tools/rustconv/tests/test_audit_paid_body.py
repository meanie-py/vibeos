"""The plain-C paid path must unseal EXACTLY ONCE.

It used to unseal, then unseal the already-unsealed text again — `_unseal_worker`
followed by a second `_unseal_plain_c` over its output. The second pass sees no tokens
at all, so every one reads as missing; it survived only because `unseal_calls`'s by-name
recovery re-sealed them from the names the first pass had just restored. Wherever that
round trip was not exact, a body the first pass ACCEPTED came back rejected, and a
rejected worker assembles nothing for the whole TU. Four netcfgx bodies died that way.
"""

from types import SimpleNamespace

import pytest

from rustconv.__main__ import CppComError, _audit_paid_body
from rustconv.call_seal import seal_calls

# C switch fallthrough becomes duplicated syntax in structured Rust, and `unseal_calls`
# accepts one token spelled twice in mutually exclusive `else if` arms. Its own output
# then has `Tail(7)` twice for a single occurrence in the C — so the second pass could
# not tell which name belonged to the token, found none missing that it could re-derive,
# and rejected the body the first pass had just accepted.
FALLTHROUGH_C = "int f(int x) { switch (x) { case 1: First(); case 2: Tail(7); break; } }"
FALLTHROUGH_RUST = (
    "unsafe fn f(x: i32) { if x == 1 { __RUSTCONV_CALL_0000__(); "
    "__RUSTCONV_CALL_0001__(7); } else if x == 2 { __RUSTCONV_CALL_0001__(7); } }"
)


def _audit(rust, c_source, callees):
    dep = SimpleNamespace(
        name="f", body_text=c_source, cpp_class=None, params=[], return_type="int"
    )
    result = SimpleNamespace(rust_code=rust)
    body = _audit_paid_body(
        dep,
        result,
        cpp_com_mode=False,
        is_cpp_source=False,
        fact_seals={},
        call_seals={"f": seal_calls(c_source, callees)},
        target_name="netcfgx",
    )
    return body, result


def test_a_body_the_envelope_accepts_is_not_rejected_by_a_second_pass():
    body, result = _audit(FALLTHROUGH_RUST, FALLTHROUGH_C, {"First", "Tail"})
    assert body.count("Tail(7)") == 2
    assert "First()" in body
    # What the caller banks must be the ACCEPTED text, and it must carry no token.
    assert result.rust_code == body
    assert "__RUSTCONV_" not in body


def test_a_genuinely_damaged_envelope_is_still_rejected():
    with pytest.raises(CppComError, match="missing/non-executable"):
        _audit(
            "unsafe fn f(x: i32) { __RUSTCONV_CALL_0000__(); }",
            FALLTHROUGH_C,
            {"First", "Tail"},
        )


def test_the_retry_prompt_carries_the_finding_and_the_faithfulness_clause():
    """A retry that re-sends the identical prompt is a re-roll, not a repair, and the
    rejects that reach one are systematic: the model is IMPROVING on the C (writing the
    loop the C forgot to advance, hoisting the `goto` tail) and will do it again."""
    from rustconv.__main__ import _reject_correction

    assert _reject_correction(None) == ""
    correction = _reject_correction(
        CppComError(
            "ScanForInfFile: model damaged deterministic call envelope "
            "(not exactly once: __RUSTCONV_CALL_0023__)"
        )
    )
    assert "not exactly once: __RUSTCONV_CALL_0023__" in correction
    assert "loops forever" in correction
    assert "Do not repair the C" in correction
