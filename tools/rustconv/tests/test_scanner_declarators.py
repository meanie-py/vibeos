"""Declarator shapes that silently cost the scanner a whole function.

Both bugs here share a failure mode: the function does not come back WRONG, it does not come
back at all. Downstream that reads as "unextractable body" or a call the binder cannot bind, so
the loud failure lands far from the declaration that caused it.
"""

from pathlib import Path

import pytest

from rustconv.scanner import scan_file

CLASS = "class C {\npublic:\n    %s\n    void Sentinel() { int x = 1; }\n};\n"


def _names(tmp_path: Path, text: str, suffix: str = ".cpp") -> dict[str, object]:
    path = tmp_path / f"probe{suffix}"
    path.write_text(text)
    return {fn.name: fn for fn in scan_file(path, force_cpp=suffix == ".cpp").functions}


@pytest.mark.parametrize(
    "declarator",
    [
        "CWndClassInfo& GetIt()",
        "CWndClassInfo & GetIt()",
        "CWndClassInfo &GetIt()",
        "static ATL::CWndClassInfo& GetIt()",
        "int& GetIt()",
    ],
)
def test_a_reference_return_still_extracts(tmp_path, declarator):
    """`T& f() {...}` had no case in the declarator descent, so EVERY one was dropped.

    Not a rare shape — it is how C++ spells a singleton accessor. CDefView's
    `static ATL::CWndClassInfo& GetWndClassInfo()` is the instance that held a 4,869-line TU
    out of paid translation, and the return type is irrelevant: a plain `int&` dropped too.
    """
    body = "\n    {\n        static int wc = 0;\n        return wc;\n    }"
    found = _names(tmp_path, CLASS % (declarator + body))
    assert "GetIt" in found, f"{declarator!r} was dropped"
    assert "&" in found["GetIt"].return_type, "the & must survive for c_type_to_rust to lower it"


def test_pointer_returns_did_not_regress(tmp_path):
    """The pointer descent this shares code with — `char **f()` was its own past incident."""
    found = _names(tmp_path, CLASS % "CWndClassInfo* GetIt()\n    { return 0; }")
    assert "GetIt" in found
    assert found["GetIt"].return_type.endswith("*")


def test_c_empty_parameter_list_is_zero_parameters(tmp_path):
    """`f(void)` declares NOTHING, but tree-sitter still reports one parameter_declaration.

    Recording it gave the function arity 1, so the binder read `SelectExtOnRename()` as arity 0
    against "available arities: [1]" and refused to bind. The uppercase `VOID` was already
    special-cased on the other two parameter paths in scanner.py and never on this one.
    """
    found = _names(tmp_path, "int f(void) { return 1; }\n", suffix=".c")
    assert found["f"].params == []


def test_a_void_pointer_is_still_a_real_parameter(tmp_path):
    """The negative control: only the BARE spelling is the empty list."""
    found = _names(
        tmp_path, "int g(void *p) { return 2; }\nint h(void *p, int n) { return 3; }\n", ".c"
    )
    assert found["g"].params == [("void *", "p")]
    assert found["h"].params == [("void *", "p"), ("int", "n")]
