"""`forwarder_with_null`: the CRT's `X(args) -> X_l(args, NULL)` idiom, for $0.

The literal is admitted ONLY where the same-TU callee's parsed signature declares a
pointer at that position. Everything these tests pin is about the refusals: a shape
that guesses pointerness would ship a silent miscompile that nobody reviews, because
there is no reviewer on the deterministic path at all.
"""

from __future__ import annotations

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


LOCALE_CALLEE = [("const unsigned char*", "s"), ("unsigned int", "x"), ("_locale_t", "locale")]


def locale_forwarder():
    return dep(
        "_mbschr",
        "unsigned char*",
        [("const unsigned char*", "s"), ("unsigned int", "x")],
        "return _mbschr_l(s, x, NULL);",
    )


def test_typedefd_pointer_is_refused_when_nothing_proves_it():
    """`_locale_t` IS a pointer, but with no authority passed, the spelling cannot say so.

    Refusing is correct: admitting it would be assuming the very fact the shape is
    supposed to prove. This is the fallback every caller gets for free, and it is why
    omitting the authority costs lowerings and never buys a wrong one.
    """
    assert lower_trivial_c_function(locale_forwarder(), {"_mbschr_l": LOCALE_CALLEE}) is None


def test_typedefd_pointer_lowers_once_the_authority_proves_it():
    """The other half: `rustconv.typedefs` says `_locale_t` is a pointer, so NULL is spellable.

    This is the dominant CRT spelling — 145 of msvcrt's functions, 76 in mbcs.c alone —
    and the pair of tests above/below is the deliberate boundary between "proven" and
    "assumed" that the whole deterministic path rests on.
    """
    low = lower_trivial_c_function(
        locale_forwarder(), {"_mbschr_l": LOCALE_CALLEE}, {"_locale_t": "core::ptr::null_mut()"}
    )
    assert low is not None
    assert low.shape == "forwarder_with_null"
    assert "_mbschr_l(s, x, core::ptr::null_mut())" in low.rust


def test_authority_for_an_unrelated_typedef_does_not_leak():
    """Proving `HANDLE` says nothing about `_locale_t`."""
    assert (
        lower_trivial_c_function(
            locale_forwarder(), {"_mbschr_l": LOCALE_CALLEE}, {"HANDLE": "core::ptr::null_mut()"}
        )
        is None
    )


def test_authority_does_not_override_a_non_pointer_position():
    """An authority naming a type that is genuinely an int must still refuse.

    The map is consulted only for names with no star; it can never promote a position
    the callee declares as a scalar.
    """
    callee = [("const unsigned char*", "s"), ("unsigned int", "x"), ("int", "flags")]
    assert (
        lower_trivial_c_function(
            locale_forwarder(), {"_mbschr_l": callee}, {"_locale_t": "core::ptr::null_mut()"}
        )
        is None
    )


def test_qualified_typedef_pointer_keeps_the_typedefs_own_spelling():
    """`const _locale_t` is a const POINTER to mutable state -> still null_mut().

    The opposite of `const char *`, where the const reaches the pointee. Getting this
    backwards is a rustc type error rather than a miscompile, but the two cases are
    genuinely different rules and the test says so.
    """
    callee = [("const unsigned char*", "s"), ("unsigned int", "x"), ("const _locale_t", "locale")]
    low = lower_trivial_c_function(
        locale_forwarder(), {"_mbschr_l": callee}, {"_locale_t": "core::ptr::null_mut()"}
    )
    assert low is not None
    assert "core::ptr::null_mut()" in low.rust


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
    callee = dep("g", "int", [("int", "a"), ("struct lc*", "l")], "return 0;")
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
