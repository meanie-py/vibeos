"""Argument typing for C++ sibling-overload binding.

Every gap here has the same failure shape: the binder cannot type an argument, so two
same-arity candidates stay tied and the run hard-fails before payment. That is the SAFE
half of the shape. The dangerous half is a wrong answer, which compiles and dispatches to
the wrong method -- so each test below pins both that a type IS resolved and that an
unprovable one stays unresolved rather than guessed.
"""

from pathlib import Path

import pytest

from rustconv.cpp_com import (
    island_pointer_aliases,
    island_struct_field_types,
    module_is_unicode,
    sdk_type_macro_aliases,
)
from rustconv.cpp_core import _abi_c_type_key, _argument_c_type, integer_cannot_bind_pointer

REPO = Path(__file__).resolve().parents[3]
ISLANDS = [
    REPO / f"sdk/lib/rust/shell32-rust/src/island_{a}.rs" for a in ("i386", "amd64", "arm64")
]
pytestmark = pytest.mark.skipif(
    not all(path.is_file() for path in ISLANDS), reason="shell32 islands not present"
)


def test_a_member_access_is_typed_from_the_struct():
    """`hittest.iItem` is an int because LVHITTESTINFO says so -- CDefView's blocker."""
    fields = island_struct_field_types(ISLANDS)
    assert fields["LVHITTESTINFO"]["iItem"] == "int"
    assert _argument_c_type("hittest.iItem", {"hittest": "LVHITTESTINFO"}, None, fields) == "int"


def test_an_arrow_needs_a_pointer_and_a_dot_needs_a_value():
    """A symbol table disagreeing with the source is a reason to decline, not to guess."""
    fields = island_struct_field_types(ISLANDS)
    assert _argument_c_type("h->iItem", {"h": "LVHITTESTINFO"}, None, fields) is None
    assert _argument_c_type("h.iItem", {"h": "LVHITTESTINFO *"}, None, fields) is None
    assert _argument_c_type("h->iItem", {"h": "LVHITTESTINFO *"}, None, fields) == "int"


def test_a_pointer_typedef_still_reaches_the_struct():
    """Win32 hides the star (`LPNMLISTVIEW`); without this `params->iItem` is untypable."""
    aliases = island_pointer_aliases(ISLANDS)
    assert aliases["LPNMLISTVIEW"] == "tagNMLISTVIEW *"
    fields = island_struct_field_types(ISLANDS)
    assert (
        _argument_c_type("params->iItem", {"params": "LPNMLISTVIEW"}, None, fields, aliases)
        == "int"
    )


def test_an_unknown_field_or_base_is_unresolved_not_invented():
    fields = island_struct_field_types(ISLANDS)
    assert _argument_c_type("hittest.nope", {"hittest": "LVHITTESTINFO"}, None, fields) is None
    assert _argument_c_type("unknown.iItem", {}, None, fields) is None
    # No field map at all must not silently fall through to the bare-identifier path.
    assert _argument_c_type("hittest.iItem", {"hittest": "LVHITTESTINFO"}, None, None) is None


def test_a_preprocessor_type_alias_is_followed():
    """`#define LV_HITTESTINFO LVHITTESTINFO` -- bindgen can never emit this."""
    aliases = sdk_type_macro_aliases(REPO, unicode=True)
    assert aliases["LV_HITTESTINFO"] == "LVHITTESTINFO"
    # ...and chains onto the A/W macro: LV_ITEM -> LVITEM -> LVITEMW.
    assert aliases["LV_ITEM"] == "LVITEMW"
    assert sdk_type_macro_aliases(REPO, unicode=False)["LV_ITEM"] == "LVITEMA"


def test_the_aw_half_is_left_unresolved_when_the_module_did_not_settle_it():
    """A WRONG half binds the wrong overload; an absent one is merely a loud ambiguity."""
    assert "LVITEM" not in sdk_type_macro_aliases(REPO, unicode=None)


def test_shell32_is_measured_unicode_not_assumed():
    assert module_is_unicode(REPO / "dll/win32/shell32/CDefView.cpp") is True


def test_a_cpp_reference_binds_the_referent():
    """`LVITEM&` takes an lvalue of LVITEM; the reference is not part of the bound type."""
    assert _abi_c_type_key("LVITEM&") == _abi_c_type_key("LVITEM")
    assert _abi_c_type_key("POINT *") != _abi_c_type_key("POINT")


def test_an_integer_never_binds_a_pointer_parameter():
    """The rule that separates f(int) from f(PCUITEMID_CHILD) when NEITHER matches exactly."""
    assert integer_cannot_bind_pointer("UINT", "POINT *")
    assert not integer_cannot_bind_pointer("UINT", "int")
    # It must never reject the pair it cannot judge -- that would discard a real candidate.
    assert not integer_cannot_bind_pointer("SomeUnknownType", "POINT *")
    assert not integer_cannot_bind_pointer("POINT *", "POINT *")


def test_a_field_the_arches_disagree_on_is_dropped():
    """Cross-arch agreement is the whole reason these fields are usable as evidence."""
    single = island_struct_field_types([ISLANDS[0]])
    every = island_struct_field_types(ISLANDS)
    assert every["LVHITTESTINFO"]["iItem"] == "int"
    # Nothing survives the intersection that one arch alone did not also prove.
    for name, members in every.items():
        for member, c_type in members.items():
            assert single.get(name, {}).get(member) == c_type


def _dep(name, params):
    from rustconv.callgraph import FunctionDep

    return FunctionDep(name=name, params=params, cpp_class="C", body_text="")


def _bind(body, candidates, *, fields=None, aliases=None):
    from rustconv.callgraph import FunctionDep
    from rustconv.cpp_com import _bind_sibling_overload

    caller = FunctionDep(name="C_caller", cpp_class="C", body_text=body, params=[])
    start = body.index("f(")
    return _bind_sibling_overload(
        body, start, start + 1, candidates, caller, [], "C", "f", None, fields, aliases
    ).name


def test_a_star_attached_to_the_name_is_still_a_declaration():
    """`TYPE *name;` is the dominant Win32 spelling and was silently unmatched.

    The pattern demanded whitespace AFTER the star, so it read only `TYPE* name` -- and a
    symbol it fails to declare is not an error anywhere, just an argument that contributes
    no evidence. That is how `lpdi->item` left two candidates tied.
    """
    fields = island_struct_field_types(ISLANDS)
    aliases = sdk_type_macro_aliases(REPO, unicode=True)
    candidates = [_dep("C_f_int", [("int", "i")]), _dep("C_f_item", [("LVITEM&", "it")])]
    body = "{ NMLVDISPINFOW    *lpdi; f(lpdi->item); }"
    assert _bind(body, candidates, fields=fields, aliases=aliases) == "C_f_item"


def test_an_integer_argument_discards_the_pointer_candidate():
    """Neither candidate matches UINT exactly; only viability separates them.

    `PCUITEMID_CHILD` only READS as a pointer once the island alias is applied, so this
    also pins that the viability rule runs on resolved spellings rather than raw ones.
    """
    aliases = island_pointer_aliases(ISLANDS)
    assert aliases["PCUITEMID_CHILD"] == "ITEMID_CHILD *"
    candidates = [_dep("C_f_pidl", [("PCUITEMID_CHILD", "p")]), _dep("C_f_int", [("int", "i")])]
    assert _bind("{ UINT i = 0; f(i); }", candidates, aliases=aliases) == "C_f_int"


def test_a_genuinely_ambiguous_call_still_hard_fails():
    """The safety property: no evidence must stay an error, never a coin flip."""
    from rustconv.cpp_com import CppComError

    candidates = [_dep("C_f_a", [("int", "i")]), _dep("C_f_b", [("DWORD", "d")])]
    with pytest.raises(CppComError, match="cannot bind overloaded sibling call"):
        _bind("{ f(SOME_UNKNOWN_MACRO); }", candidates)


def test_a_nonzero_integer_literal_is_an_int_but_zero_stays_untyped():
    """`0` is a null pointer constant; typing it `int` would discard a real candidate."""
    from rustconv.cpp_core import _argument_c_type as arg

    assert arg("-2", {}) == "int"
    assert arg("42", {}) == "int"
    assert arg("0x10", {}) == "int"
    assert arg("0", {}) is None
    assert arg("0L", {}) is None


def test_an_integer_does_not_bind_a_struct_parameter():
    """A plain C aggregate has no converting constructor, by value or by reference."""
    from rustconv.cpp_com import _no_viable_conversion

    fields = island_struct_field_types(ISLANDS)
    assert _no_viable_conversion("LRESULT", "LVITEMW&", fields)
    assert _no_viable_conversion("int", "LVITEMW", fields)
    # A pointer to that struct is rejected too, but by the POINTER rule -- an integer
    # binds neither the aggregate nor a pointer to it.
    assert _no_viable_conversion("int", "LVITEMW *", fields)
    # A pointer argument against that pointer parameter is the case that must survive.
    assert not _no_viable_conversion("LVITEMW *", "LVITEMW *", fields)
    # Without a field map nothing is provable, so nothing is rejected.
    assert not _no_viable_conversion("LRESULT", "LVITEMW&", None)


def test_the_vtable_arity_fallback_will_not_cross_an_impossible_conversion():
    """A private helper must not inherit a vtable signature it does not implement.

    The fallback binds on arity when the exact shape misses -- which is correct, because
    the scanner and the vtable often spell one parameter differently. Unconstrained it
    also captured CDefView's `SelectAndPositionItem(int, UINT, POINT *)` and OVERWROTE
    its parameters with the slot's `(PCUITEMID_CHILD, UINT, POINT *)`. Nothing downstream
    could see that swap.
    """
    from rustconv.cpp_com import _shapes_provably_disagree

    aliases = island_pointer_aliases(ISLANDS)
    vtable = [("PCUITEMID_CHILD", "pidl"), ("UINT", "f"), ("POINT *", "pt")]
    helper = [("int", "Idx"), ("UINT", "fSVSI"), ("POINT *", "ppt")]
    assert _shapes_provably_disagree(helper, vtable, aliases)
    # The real implementor differs only in SPELLING and must still bind.
    method = [("LPCITEMIDLIST", "item"), ("UINT", "flags"), ("POINT *", "point")]
    assert not _shapes_provably_disagree(method, vtable, aliases)
