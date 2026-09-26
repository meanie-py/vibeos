"""A class declared INSIDE a multi-purpose TU selects its support closure from its own text.

Regression: shell32.cpp's IDefClFImpl got an extern bridge for SHELL32_CoCreateInitSFWithCsidl
because render_cpp_support selected from the WHOLE file — which names that helper only in an
unrelated trampoline — and the bridge then collided (E0255) with the crate's own definition.
Also: `--fn` naming a function found in no file must not exit 0.
"""

from __future__ import annotations

from types import SimpleNamespace

from rustconv.__main__ import _class_declaration_scope, _unmatched_fn_names
from rustconv.cpp_com import render_cpp_support

TU = """\
class Other { void f(); };
extern "C" HRESULT WINAPI Trampoline() { return SHELL_GetCommandStringImpl(0, 0, 0, 0, 0); }
class Wanted : public IClassFactory
{
    LONG *pcRefDll;
public:
    STDMETHOD(LockServer)(BOOL fLock) override;
};
"""


def test_in_tu_class_scope_is_the_declaration_only(tmp_path):
    src = tmp_path / "multi.cpp"
    src.write_text(TU)
    cls = SimpleNamespace(name="Wanted", source=src, header=src)
    scope = _class_declaration_scope(cls)
    assert scope.startswith("class Wanted") and scope.rstrip().endswith("}")
    assert "SHELL_GetCommandStringImpl" not in scope and "class Other" not in scope


def test_separate_header_is_used_whole(tmp_path):
    src, hdr = tmp_path / "x.cpp", tmp_path / "x.h"
    src.write_text("")
    hdr.write_text(TU)
    cls = SimpleNamespace(name="Wanted", source=src, header=hdr)
    assert _class_declaration_scope(cls) == TU


def test_select_from_source_false_ignores_the_rest_of_the_tu(tmp_path):
    src = tmp_path / "multi.cpp"
    src.write_text(TU)
    whole = render_cpp_support(src, set(), "")
    scoped = render_cpp_support(src, set(), "class Wanted {}", select_from_source=False)
    assert "SHELL_GetCommandStringImpl" in whole, "precondition: the whole file selects it"
    assert "SHELL_GetCommandStringImpl" not in scoped


def test_unmatched_fn_names():
    # a name missing from ONE file of a multi-file run is fine; one found nowhere is not
    assert _unmatched_fn_names("A, B,C", {"A", "C"}) == {"B"}
    assert _unmatched_fn_names("A", {"A"}) == set()
    assert _unmatched_fn_names(" , A ,", set()) == {"A"}
