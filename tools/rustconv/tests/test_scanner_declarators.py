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


def test_winapi_recovered_as_the_name_gives_the_name_back(tmp_path):
    """tree-sitter sometimes makes WINAPI the declarator and puts the real name in an ERROR.

    comctl32undoc.c's `Alloc` and `Free` were planned as two functions named `WINAPI`, while
    the identically shaped `ReAlloc` between them parsed the other way round.
    """
    fns = _names(
        tmp_path,
        "LPVOID WINAPI Alloc (DWORD dwSize)\n{\n    return 0;\n}\n"
        "LPVOID WINAPI ReAlloc (LPVOID p, DWORD dwSize)\n{\n    return p;\n}\n",
        suffix=".c",
    )
    assert {"Alloc", "ReAlloc"} <= fns.keys()
    assert "WINAPI" not in fns
    assert fns["Alloc"].calling_convention == "WINAPI"


SPLIT_BRACE = """
static int Before(int a)
{
    return a;
}

static int Split(int msg, int a)
{
    switch (msg)
    {
    case 1:
#ifdef __REACTOS__
        if (a) {
#else
        if (a && msg) {
#endif
            a++;
        }
        return a;
    }
    return 0;
}

static int After(int a)
{
    return a;
}
"""


def test_braces_split_across_reactos_arms_still_extract(tmp_path):
    """combo.c's WM_SYSKEYDOWN case opens a `{` in one `__REACTOS__` arm, closes it in another."""
    fns = _names(tmp_path, SPLIT_BRACE, suffix=".c")
    assert {"Before", "Split", "After"} <= fns.keys()
    assert "#ifdef" not in fns["Split"].body_text


def test_a_file_that_scans_completely_keeps_its_reactos_lines(tmp_path):
    """The resolution is a fallback: banked bodies from a clean file must not change text."""
    text = "static int F(int a)\n{\n#ifdef __REACTOS__\n    a++;\n#endif\n    return a;\n}\n"
    fns = _names(tmp_path, text, suffix=".c")
    assert "#ifdef __REACTOS__" in fns["F"].body_text


SCOPED_RENAME = """
#define Widget_Add Internal_Add
#ifdef USE_OTHER
#define Widget_Del Other_Del
#endif

int WINAPI Widget_Add(int a)
{
    TRACE("Widget_Add\\n");
    return a;
}

int WINAPI Widget_Del(int a)
{
    return Widget_Add(a);
}

#undef Widget_Add

int WINAPI Widget_Add(int a)
{
    return Internal_Add(a) + 1;
}
"""


def test_a_scoped_rename_names_the_definition_as_the_compiler_does(tmp_path):
    """imagelist.c defines its internal bodies under the public ImageList_* spelling."""
    fns = _names(tmp_path, SCOPED_RENAME, suffix=".c")
    assert {"Internal_Add", "Widget_Add", "Widget_Del"} <= fns.keys()
    assert "Internal_Add(a)" in fns["Widget_Del"].body_text  # calls inside the window too
    assert 'TRACE("Widget_Add' in fns["Internal_Add"].body_text  # literals stay text
    assert "+ 1" in fns["Widget_Add"].body_text  # the public one is the wrapper


_ONE_SPLIT = """
#ifdef __REACTOS__
static void {name}(INFO *infoPtr, LPPOINT lpPos, INT nItem)
#else
static void {name}(INFO *infoPtr, LPPOINT lpPos)
#endif
{{
    *lpPos = infoPtr->pos;
}}
"""
# Two in a row, as listview.c has (NextIconPosTop, NextIconPosLeft): every function is still
# extracted, but tree-sitter wraps what follows in one ERROR node that the preamble kept.
SPLIT_SIGNATURE = (
    _ONE_SPLIT.format(name="Top")
    + _ONE_SPLIT.format(name="Left")
    + "static int After(int a)\n{\n    return a + 1;\n}\n"
)


def test_a_split_signature_does_not_dump_bodies_into_the_preamble(tmp_path):
    """listview.c: the rest of the file became one ERROR node, carried in every prompt."""
    path = tmp_path / "probe.c"
    path.write_text(SPLIT_SIGNATURE)
    scan = scan_file(path)
    assert {"Top", "Left", "After"} <= {f.name for f in scan.functions}
    assert "return a + 1" not in scan.preamble_text
    assert scan.preamble_swallowed_functions == 0


def test_a_template_argument_keeps_its_own_pointer():
    """cleanmgr: `CAtlList<CCleanupHandler *>` is a list of pointers; counting every `*` in the
    string made it `*mut CAtlList<CCleanupHandler >`, a pointer to a list of objects."""
    from rustconv.cpp_core import c_type_to_rust

    assert c_type_to_rust("CAtlList<CCleanupHandler *>") == "CAtlList<*mut CCleanupHandler>"
    assert c_type_to_rust("CSimpleArray<int *> *") == "*mut CSimpleArray<*mut i32>"
    assert c_type_to_rust("const CComPtr<IFoo> &") == "&CComPtr<IFoo>"
    assert c_type_to_rust("CSimpleMap<UINT, const WCHAR *>") == "CSimpleMap<UINT, *const WCHAR>"
    assert c_type_to_rust("ATL::CAtlList<ns::X*>") == "CAtlList<*mut X>"
    assert c_type_to_rust("CFoo<3>") == "CFoo<3>"
