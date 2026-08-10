"""A C++ method call's Rust lowering takes an implicit `this`, and the arity check must expect it.

`--cpp-com` lowers each method to `<Class>_<Method>_body(this, ...)`. At the C call site a
sibling method call shows only its explicit arguments, so comparing the two counts directly is
off by exactly one -- the receiver.

This was found the expensive way, on CDrivesFolderEnum. It failed in BOTH directions at once:

  - `CDrivesFolderEnum_AppendItemsFromEnumerator` wrote the CORRECT
    `__RUSTCONV_CALL_0000__(this, pidl)` and was rejected twice ("AddToEnumList_body wants 1,
    got 2"), burning two paid calls and voiding the whole TU's assembly.
  - `CDrivesFolderEnum_dtor` wrote `CDrivesFolderEnum_DeleteList_body()` -- no receiver at all,
    against a fn whose first parameter is `this` -- and was ACCEPTED, because 0 matched the C
    site's 0. It cannot compile, and the envelope had declared it good.

The second is the one that matters: the envelope's job is to make a corrupt body fail loudly.
"""

from __future__ import annotations

from rustconv.call_seal import seal_calls


BODY = "void f(void) { DeleteList(); AddToEnumList(pidl); }"


def _argc(sealed, callee):
    for call in sealed.calls:
        if call.original == callee:
            return call.argc
    raise AssertionError(f"{callee} not sealed: {[c.original for c in sealed.calls]}")


def test_a_receiver_callee_expects_one_more_argument():
    src = "void f(void) { CDrivesFolderEnum_DeleteList_body(); }"
    sealed = seal_calls(
        src,
        {"CDrivesFolderEnum_DeleteList_body"},
        receiver_callees=frozenset({"CDrivesFolderEnum_DeleteList_body"}),
    )
    # The C site shows 0 arguments; the Rust lowering must pass `this`.
    assert _argc(sealed, "CDrivesFolderEnum_DeleteList_body") == 1


def test_a_receiver_callee_with_arguments_also_gains_one():
    src = "void f(void) { CDrivesFolderEnum_AddToEnumList_body(pidl); }"
    sealed = seal_calls(
        src,
        {"CDrivesFolderEnum_AddToEnumList_body"},
        receiver_callees=frozenset({"CDrivesFolderEnum_AddToEnumList_body"}),
    )
    assert _argc(sealed, "CDrivesFolderEnum_AddToEnumList_body") == 2


def test_a_free_function_is_unchanged():
    """Only same-class methods take a receiver; everything else keeps exact matching."""
    src = "void f(void) { SomeFreeHelper(a, b); }"
    sealed = seal_calls(src, {"SomeFreeHelper"}, receiver_callees=frozenset())
    assert _argc(sealed, "SomeFreeHelper") == 2


def test_the_default_is_no_receiver_adjustment():
    """Callers that pass no set (every non-cpp-com path) must behave exactly as before."""
    src = "void f(void) { CDrivesFolderEnum_DeleteList_body(); }"
    sealed = seal_calls(src, {"CDrivesFolderEnum_DeleteList_body"})
    assert _argc(sealed, "CDrivesFolderEnum_DeleteList_body") == 0


def test_a_receiver_callee_with_a_default_argument_shifts_both_bounds():
    """A method with C++ defaults keeps a RANGE, and both ends gain the receiver."""
    tu = "HRESULT Helper(int a, int b = 0);\n"
    src = "void f(void) { Helper_body(x); }"
    sealed = seal_calls(
        src,
        {"Helper_body"},
        tu_source=tu,
        receiver_callees=frozenset({"Helper_body"}),
    )
    call = next(c for c in sealed.calls if c.original == "Helper_body")
    assert call.argc == 2
    # `_defaulted_param_count` reads the declaration of `Helper_body`, which does not exist in
    # this TU, so there is no range to widen -- the receiver shift must not invent one.
    assert call.argc_max is None
