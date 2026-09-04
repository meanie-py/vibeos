"""cpp_core is the generic C/C++ layer; it must not learn about COM.

~70% of the tool's entire C++ capability had accumulated inside cpp_com.py, which meant
translating C++ that is not COM had almost nothing to stand on. cpp_core is the beginning
of the generic layer carved back out. The direction of the dependency is the whole point:
cpp_com imports cpp_core, never the reverse.
"""

from __future__ import annotations

import ast
import re
from pathlib import Path

import pytest

from rustconv import cpp_com, cpp_core

CORE = Path(cpp_core.__file__)
COM = Path(cpp_com.__file__)


def test_core_does_not_import_com():
    """The dependency runs one way. A cycle here would mean the split is fictional."""
    tree = ast.parse(CORE.read_text())
    imported = set()
    for node in ast.walk(tree):
        if isinstance(node, ast.ImportFrom) and node.module:
            imported.add(node.module)
        elif isinstance(node, ast.Import):
            imported |= {a.name for a in node.names}
    assert not any("cpp_com" in m for m in imported), f"cpp_core imports {imported}"


def test_core_carries_no_com_vocabulary():
    """Names an object model owns: interfaces, vtable slots, refcounting, IDL.

    HRESULT is allowed only as an entry in a type-mapping table — it is a Windows scalar
    like DWORD, and refusing it would mean the generic layer cannot map Windows C at all.
    """
    text = CORE.read_text()
    banned = (
        "IUnknown",
        "QueryInterface",
        "AddRef",
        "CComPtr",
        "OBJECT_ENTRY",
        "STDMETHOD",
        "lpVtbl",
        "widl",
        "WIDL",
    )
    found = [word for word in banned if re.search(rf"\b{re.escape(word)}\b", text)]
    assert not found, f"COM vocabulary leaked into cpp_core: {found}"


def test_com_error_identity_is_shared():
    """`except CppComError` must still catch what the generic layer raises.

    Most deterministic proofs moved to cpp_core; if this were a subclass rather than the
    same class, every existing handler would silently stop catching them.
    """
    assert cpp_com.CppComError is cpp_core.CppTranslationError


@pytest.mark.parametrize(
    ("c_type", "expected"),
    [
        ("const struct tagNMHDR *", "*const tagNMHDR"),
        ("DWORD", "DWORD"),
        ("void", "core::ffi::c_void"),
        ("unsigned long", "u32"),
        ("CShellLink::volume_info *", "*mut volume_info"),
    ],
)
def test_declarator_mapping_still_works(c_type, expected):
    """Generic declarator lowering — the piece qTox-shaped C++ needs most."""
    assert cpp_core.c_type_to_rust(c_type) == expected


def test_argument_splitting_respects_nesting():
    assert cpp_core._split_args("a, f(b,c), d") == ["a", "f(b,c)", "d"]
    assert cpp_core._split_args("Tmpl<A,B> x, y") == ["Tmpl<A,B> x", "y"]


def test_c_constexpr_evaluates_portable_bounds():
    """Array bounds must resolve without depending on an architecture."""
    assert cpp_core._eval_c_int_constexpr("('Z' - 'A' + 1)") == 26
    assert cpp_core._eval_c_int_constexpr("1 << 4") == 16
    # names/calls/floats are refused so the caller fails loud rather than guessing
    assert cpp_core._eval_c_int_constexpr("MAX_PATH") is None
    assert cpp_core._eval_c_int_constexpr("sizeof(x)") is None


def test_com_still_owns_its_own_vocabulary():
    """The split moved the generic half out — it did not gut COM."""
    text = COM.read_text()
    for word in ("QueryInterface", "lpVtbl", "CComPtr"):
        assert word in text, f"cpp_com lost {word}"
