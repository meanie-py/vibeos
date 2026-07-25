"""`forwarder_with_null`: the CRT's `X(args) -> X_l(args, NULL)` idiom, for $0.

The literal is admitted ONLY where the same-TU callee's parsed signature declares a
pointer at that position. Everything these tests pin is about the refusals: a shape
that guesses pointerness would ship a silent miscompile that nobody reviews, because
there is no reviewer on the deterministic path at all.
"""

from __future__ import annotations

import pytest

from rustconv.callgraph import FunctionDep
from rustconv.trivial_c import lower_trivial_c_function, lower_trivial_c_functions


def dep(name, ret, params, body, callees=()):
    sig_params = ", ".join(f"{t} {n}" for t, n in params)
    return FunctionDep(
        name=name,
        params=list(params),
        return_type=ret,
        body_text=f"{ret} CDECL {name}({sig_params})\n{{\n    {body}\n}}",
        callees=list(callees),
    )


CALLEE_PTR = [("const unsigned char*", "s"), ("unsigned int", "x"), ("struct lc*", "locale")]


def test_null_at_a_proven_pointer_position_lowers():
    d = dep(
        "_mbschr",
        "unsigned char*",
        [("const unsigned char*", "s"), ("unsigned int", "x")],
        "return _mbschr_l(s, x, NULL);",
    )
    low = lower_trivial_c_function(d, {"_mbschr_l": CALLEE_PTR})
    assert low is not None
    assert low.shape == "forwarder_with_null"
    assert "_mbschr_l(s, x, core::ptr::null_mut())" in low.rust


def test_const_pointee_picks_null_not_null_mut():
    callee = [("int", "a"), ("const char*", "p")]
    d = dep("f", "int", [("int", "a")], "return g(a, NULL);")
    low = lower_trivial_c_function(d, {"g": callee})
    assert low is not None
    assert "g(a, core::ptr::null())" in low.rust


def test_null_at_a_non_pointer_position_is_refused():
    """`NULL` where the callee declares an int is not this shape — pay the model."""
    callee = [("const unsigned char*", "s"), ("unsigned int", "x"), ("int", "flags")]
    d = dep(
        "_mbschr",
        "unsigned char*",
        [("const unsigned char*", "s"), ("unsigned int", "x")],
        "return _mbschr_l(s, x, NULL);",
    )
    assert lower_trivial_c_function(d, {"_mbschr_l": callee}) is None


def test_typedefd_pointer_is_refused_because_nothing_proves_it():
    """`_locale_t` IS a pointer, but neither the spelling nor the type mapper says so.

    This is the dominant CRT spelling and the reason this shape's real msvcrt reach is
    11 functions rather than the ~146 a text-only match suggests. Refusing is correct:
    admitting it would be assuming the very fact the shape is supposed to prove. Pinned
    so that if a typedef authority is ever wired in, this test is the deliberate
    decision point rather than a silent behaviour change.
    """
    callee = [("const unsigned char*", "s"), ("unsigned int", "x"), ("_locale_t", "locale")]
    d = dep(
        "_mbschr",
        "unsigned char*",
        [("const unsigned char*", "s"), ("unsigned int", "x")],
        "return _mbschr_l(s, x, NULL);",
    )
    assert lower_trivial_c_function(d, {"_mbschr_l": callee}) is None


def test_unknown_callee_is_refused():
    """No sibling signature -> nothing to prove the NULL against."""
    d = dep(
        "_mbschr",
        "unsigned char*",
        [("const unsigned char*", "s"), ("unsigned int", "x")],
        "return _mbschr_l(s, x, NULL);",
    )
    assert lower_trivial_c_function(d, {}) is None
    assert lower_trivial_c_function(d) is None  # no map at all: pre-existing behaviour


def test_arity_mismatch_is_refused():
    """A variadic or mis-parsed callee must not be forwarded to blind."""
    short = [("const unsigned char*", "s"), ("unsigned int", "x")]
    d = dep(
        "_mbschr",
        "unsigned char*",
        [("const unsigned char*", "s"), ("unsigned int", "x")],
        "return _mbschr_l(s, x, NULL);",
    )
    assert lower_trivial_c_function(d, {"_mbschr_l": short}) is None


def test_reordered_arguments_are_refused():
    d = dep(
        "_mbschr",
        "unsigned char*",
        [("const unsigned char*", "s"), ("unsigned int", "x")],
        "return _mbschr_l(x, s, NULL);",
    )
    assert lower_trivial_c_function(d, {"_mbschr_l": CALLEE_PTR}) is None


def test_plain_forwarder_without_null_still_works():
    """The pre-existing shape must be untouched by the extension."""
    d = dep("a", "int", [("int", "x")], "return b(x);")
    low = lower_trivial_c_function(d, {"b": [("int", "x")]})
    assert low is not None and low.shape == "forwarder"


def test_batch_builds_the_sibling_map_itself():
    """`lower_trivial_c_functions` must supply signatures without the caller helping."""
    callee = dep(
        "g", "int", [("int", "a"), ("struct lc*", "l")], "return 0;"
    )
    caller = dep("f", "int", [("int", "a")], "return g(a, NULL);", callees=["g"])
    out = lower_trivial_c_functions([callee, caller])
    assert out["f"].shape == "forwarder_with_null"


def test_null_moved_to_the_wrong_slot_is_caught():
    """C forwards NULL last; a lowering that put it first must not verify."""
    callee = [("struct lc*", "l"), ("int", "a")]
    d = dep("f", "int", [("int", "a")], "return g(NULL, a);")
    low = lower_trivial_c_function(d, {"g": callee})
    assert low is not None  # this IS the shape — NULL first is legitimate here
    assert "g(core::ptr::null_mut(), a)" in low.rust
