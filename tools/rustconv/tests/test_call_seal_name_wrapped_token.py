"""A callee written BOTH by name and by token doubles the call, and "exactly once" misses it.

The call envelope's duplicate rule counts TOKENS. If the model wraps the sealed token in a
hand-written copy of the callee's own name -- `NAME(<token>(args))` -- the token still appears
exactly once, the audit passes, and substitution turns it into `NAME(NAME(args))`: a second
call to the same function taking the first call's result as its only argument.

Seen once, in `CCPLItemMenu::InvokeCommand` (folders/CControlPanelFolder.cpp), where eight
callees were doubled in one ACCEPTED body -- including a zero-argument one,
`_ILCreateControlPanel(_ILCreateControlPanel())`. It does not compile, so it fails loudly rather
than shipping; but it is a paid artifact the envelope declared good, and that is the property
worth defending.
"""

from __future__ import annotations

import pytest

from rustconv.call_seal import CallSealError, SealedCall, SealedCalls, unseal_calls


def _sealed(*names: str) -> SealedCalls:
    calls = tuple(
        SealedCall(f"__RUSTCONV_CALL_{i:04d}__", name, False, 1) for i, name in enumerate(names)
    )
    return SealedCalls("", calls)


def _audit(rust: str, sealed: SealedCalls) -> str:
    return unseal_calls(rust, sealed)


def test_name_wrapped_token_is_rejected():
    sealed = _sealed("wsprintfW")
    rust = "fn f() { wsprintfW(__RUSTCONV_CALL_0000__(buf)); }"
    with pytest.raises(CallSealError) as exc:
        _audit(rust, sealed)
    assert "wsprintfW" in str(exc.value)
    assert "NAME(NAME" in str(exc.value)


def test_zero_argument_form_is_rejected():
    """`_ILCreateControlPanel(_ILCreateControlPanel())` -- no args to disambiguate on."""
    sealed = _sealed("_ILCreateControlPanel")
    rust = "fn f() { let p = _ILCreateControlPanel(__RUSTCONV_CALL_0000__()); }"
    with pytest.raises(CallSealError) as exc:
        _audit(rust, sealed)
    assert "_ILCreateControlPanel" in str(exc.value)


def test_an_ordinary_single_call_passes():
    sealed = _sealed("wsprintfW")
    rust = "fn f() { __RUSTCONV_CALL_0000__(buf); }"
    assert "wsprintfW(buf)" in _audit(rust, sealed)


def test_genuine_nesting_with_a_second_argument_is_left_alone():
    """`f(f(x), y)` is real code; only the inner-call-is-the-WHOLE-argument shape is corruption."""
    sealed = _sealed("wsprintfW")
    rust = "fn f() { wsprintfW(__RUSTCONV_CALL_0000__(x), y); }"
    _audit(rust, sealed)
