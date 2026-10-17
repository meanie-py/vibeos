"""cpp_model: a module class template becomes one concrete class per instantiation.

Runs the host clang on a small module (the shape of rapps' rosui.h), not the tree's TUs."""

from __future__ import annotations

import json
import shutil
import subprocess
from pathlib import Path

import pytest

from rustconv import c_data_port as cdp
from rustconv import cpp_model

SRC = """
struct W { int m_hWnd; int IsWindow() { return m_hWnd; } };
struct P { virtual ~P() {} };
template <class T = W> class U : public P, public T {
public:
  virtual int Get() { return T::m_hWnd + 1; }
  int Unused() { return T::IsWindow(); }
  U *Self() { U *me = this; return me; }
};
template <class T, int G = 10> class A {
protected: T *m_p;
public: A() { m_p = 0; } int Count() const { return G; }
};
class C : public A<P> { public: int N() { return Count(); } };
class D : public U<W> {};
typedef P PP;
A<PP> *g_ap;
#define FNX(fn) int fn(int a)
FNX(fnClose) { return a; }
int Commented(int b) // a comment between the declarator and the brace
{ return b; }
template <class T> static int Bf(const T &base, int a) { return a + (int)sizeof(T); }
int UseBf() { char buf[4]; return Bf(*g_ap, 1) + Bf(buf, 2) + Bf(*g_ap, 3); }
int main() { D d; U<W> *q = new U<W>(); C c; return d.Get() + c.N() + (q->Self() != 0); }
"""


@pytest.fixture(scope="module")
def model(tmp_path_factory) -> cpp_model.ModuleModel:
    if shutil.which("clang++") is None:
        pytest.skip("no host clang++")
    d = tmp_path_factory.mktemp("mod")
    tu = d / "t.cpp"
    tu.write_text(SRC)
    out = subprocess.run(
        ["clang++", "-fsyntax-only", "-Xclang", "-ast-dump=json", str(tu)],
        capture_output=True,
        check=True,
    ).stdout
    ast = json.loads(out)
    cdp._attach_files(ast)
    ex = cpp_model._Extractor(d, d)
    ex.scan_overloads(ast["inner"])
    ex.walk(ast["inner"])
    ex.instance_sugar(ast["inner"])
    ex.apply_instances()
    ex.lower_pending()
    ex.apply_instances()
    return ex.model


def test_each_instantiation_is_a_class_and_the_pattern_is_not(model):
    assert "U_W" not in model.classes  # U<W> is U<> (W is the default): the bare name
    assert "U" in model.classes and model.classes["U"].template == "U"
    assert "A_P" in model.classes
    assert [b.name for b in model.classes["U"].bases] == ["P", "W"]


def test_arguments_are_substituted_in_fields(model):
    assert [(f.name, f.c_type) for f in model.classes["A_P"].fields] == [("m_p", "P *")]


def test_every_spelling_reaches_the_class(model):
    assert model.instances["U<W>"] == "U" and model.instances["U<>"] == "U"
    assert model.instances["A<P>"] == "A_P" and model.instances["A<P,10>"] == "A_P"
    assert [b.name for b in model.classes["C"].bases] == ["A_P"]
    assert [b.name for b in model.classes["D"].bases] == ["U"]


def test_a_typedef_spelling_reaches_the_class(model):
    # clang names the specialization A<P>; the module wrote A<PP>.
    assert next(g for g in model.globals if g.name == "g_ap").c_type == "A_P *"


def test_only_used_members_have_bodies(model):
    u = model.classes["U"]
    assert u.method("Get")[0].body is not None
    assert u.method("Unused")[0].body is None


def test_the_injected_class_name_is_the_instance(model):
    a_p = model.classes["A_P"]
    assert a_p.method("Count")[0].lowered == "{ return 10; }"
    low = model.classes["U"].method("Self")[0].lowered or ""
    assert "U *me" in low
    assert model.classes["U"].method("Get")[0].lowered == "{ return this_->__base1.m_hWnd + 1; }"


def test_the_model_round_trips_its_instances(model):
    again = cpp_model.ModuleModel.from_json(model.to_json())
    assert again.instances == model.instances
    assert again.classes["A_P"].template == "A"


def test_a_macro_declared_function_is_spelled_from_its_type(model):
    # fdi.h's FNCLOSE(fnFileClose): the text names no function.
    f = next(f for f in model.functions if f.name == "fnClose")
    assert f.signature == "int fnClose(int a)"


def test_a_comment_in_a_signature_cannot_swallow_the_body(model):
    f = next(f for f in model.functions if f.name == "Commented")
    assert f.signature == "int Commented(int b)"


def test_a_function_template_is_one_function_per_instantiation(model):
    names = sorted(f.name for f in model.functions if f.name.startswith("Bf"))
    assert names == ["Bf__T0", "Bf__T1"]
    use = next(f for f in model.functions if f.name == "UseBf").lowered
    assert use.count("Bf__T0(") == 2 and use.count("Bf__T1(") == 1
    arr = next(f for f in model.functions if f.name == "Bf__T1")
    assert arr.signature == "static int Bf__T1(const char (&base)[4], int a)"


def test_relative_path_is_inside_the_module(model):
    assert Path(model.classes["A_P"].file).name == "t.cpp"


BASE_SRC = """
#include "../inc/r.h"
class CUiRect : public RECT { public: CUiRect() { left = 0; } };
class CUiMargin : public CUiRect { public: CUiMargin() {} };
class Box { public: CUiMargin m_Margin; int F(RECT &r) { r.left += m_Margin.left; return 0; } };
class Pan { public: Box *b; void G() { b->m_Margin.top = 4; } };
int main() { Box x; Pan p; p.b = &x; p.G(); RECT r; return x.F(r); }
"""


def test_a_field_of_a_two_hop_external_base_is_reached_by_path(tmp_path):
    # rapps' CUiMargin -> CUiRect -> RECT: clang casts to `tagRECT`, the base list says
    # `RECT`, and RECT is two hops away. Without the path, Rust went through the Deref.
    if shutil.which("clang++") is None:
        pytest.skip("no host clang++")
    (tmp_path / "inc").mkdir()
    (tmp_path / "inc" / "r.h").write_text("typedef struct tagRECT { int left; int top; } RECT;\n")
    d = tmp_path / "mod"
    d.mkdir()
    (d / "t.cpp").write_text(BASE_SRC)
    out = subprocess.run(
        ["clang++", "-fsyntax-only", "-Xclang", "-ast-dump=json", str(d / "t.cpp")],
        capture_output=True,
        check=True,
    ).stdout
    ast = json.loads(out)
    cdp._attach_files(ast)
    ex = cpp_model._Extractor(d, d)
    ex.scan_overloads(ast["inner"])
    ex.walk(ast["inner"])
    ex.lower_pending()
    box = ex.model.classes["Box"].method("F")[0].lowered
    assert "this_->m_Margin.__base0.__base0.left" in box
    pan = ex.model.classes["Pan"].method("G")[0].lowered
    assert pan == "{ this_->b->m_Margin.__base0.__base0.top = 4; }"


DELETE_SRC = """
class Rich { public: virtual ~Rich() {} };
class Disp { public: Rich *RichEdit; int *buf; ~Disp() { delete RichEdit; delete[] buf; } };
int main() { Disp d; d.RichEdit = new Rich(); d.buf = new int[2]; return 0; }
"""


def test_delete_of_an_implicit_this_member_keeps_the_member_path(tmp_path):
    # rapps: `delete RichEdit;` lowered to `CAppRichEdit__delete(RichEdit)ichEdit;` -- the
    # whole expression was replaced by the operand's SOURCE text, so its `this_->` edit
    # landed inside the replaced span and left the tail behind.
    if shutil.which("clang++") is None:
        pytest.skip("no host clang++")
    (tmp_path / "t.cpp").write_text(DELETE_SRC)
    out = subprocess.run(
        ["clang++", "-fsyntax-only", "-Xclang", "-ast-dump=json", str(tmp_path / "t.cpp")],
        capture_output=True,
        check=True,
    ).stdout
    ast = json.loads(out)
    cdp._attach_files(ast)
    ex = cpp_model._Extractor(tmp_path, tmp_path)
    ex.scan_overloads(ast["inner"])
    ex.walk(ast["inner"])
    ex.lower_pending()
    dtor = next(m for m in ex.model.classes["Disp"].methods if m.kind == "dtor").lowered
    assert "Rich__delete(this_->RichEdit);" in dtor
    assert "cpp_delete_array(this_->buf);" in dtor


UPCAST_SRC = """
struct Prim { virtual ~Prim() {} virtual struct Box *AsBox() { return 0; } };
struct Box { virtual ~Box() {} int margin; };
struct Panel : public Prim, public Box { Box *AsBox() { return this; } int m() { return margin; } };
int main() { Panel p; return p.AsBox() != 0 && p.m() == 0; }
"""


def test_returning_this_as_a_secondary_base_is_the_subobject(tmp_path):
    # rapps' `CUiBox *AsBox() { return this; }`: CUiBox is the SECOND base, so the pointer
    # moves. Lowered as bare `this_`, the model cast it without the offset.
    if shutil.which("clang++") is None:
        pytest.skip("no host clang++")
    (tmp_path / "t.cpp").write_text(UPCAST_SRC)
    out = subprocess.run(
        ["clang++", "-fsyntax-only", "-Xclang", "-ast-dump=json", str(tmp_path / "t.cpp")],
        capture_output=True,
        check=True,
    ).stdout
    ast = json.loads(out)
    cdp._attach_files(ast)
    ex = cpp_model._Extractor(tmp_path, tmp_path)
    ex.scan_overloads(ast["inner"])
    ex.walk(ast["inner"])
    ex.lower_pending()
    panel = ex.model.classes["Panel"]
    as_box = next(m for m in panel.methods if m.name == "AsBox").lowered
    assert "(&(this_)->__base1)" in as_box
    # an implicit-this member of the same base keeps its member path, not a wrapped `this`
    assert "this_->__base1.margin" in next(m for m in panel.methods if m.name == "m").lowered


CONVERT_SRC = """
#include "str.h"
struct Name { Name(int v) : v(v) {} int v; };
static int use(const CStringA &s) { return s.s != 0; }
static int named(const Name &n) { return n.v; }
struct Cab {
    char *m_file;
    int open(char *name) {
        return use(name) + use("lit") + use(CStringA(name)) + use(m_file) + named(3);
    }
};
int main() { Cab c; c.m_file = 0; return c.open(0); }
"""


def test_an_implicit_converting_constructor_is_written_out(tmp_path):
    # rapps' fnFileOpen passes a `char *` to `const CStringA &`: with no `CStringA(` in the
    # text the model cast the pointer instead of constructing the string.
    if shutil.which("clang++") is None:
        pytest.skip("no host clang++")
    (tmp_path / "inc").mkdir()
    (tmp_path / "inc" / "str.h").write_text(
        "struct CStringA { CStringA(const char *p) : s(p) {} const char *s; };\n"
    )
    d = tmp_path / "mod"
    d.mkdir()
    (d / "t.cpp").write_text(CONVERT_SRC)
    out = subprocess.run(
        [
            "clang++",
            "-fsyntax-only",
            "-I",
            str(tmp_path / "inc"),
            "-Xclang",
            "-ast-dump=json",
            str(d / "t.cpp"),
        ],
        capture_output=True,
        check=True,
    ).stdout
    ast = json.loads(out)
    cdp._attach_files(ast)
    ex = cpp_model._Extractor(d, d)
    ex.scan_overloads(ast["inner"])
    ex.walk(ast["inner"])
    ex.lower_pending()
    body = next(m for m in ex.model.classes["Cab"].methods if m.name == "open").lowered
    assert "use(CStringA(name))" in body
    assert 'use("lit")' in body  # a literal stays as written
    assert "use(CStringA(name)) + use(CStringA(this_->m_file))" in body  # written: unchanged
    assert "named(Name__make_0(3))" in body


STATIC_SRC = """
typedef unsigned long DWORD;
typedef unsigned char BYTE;
struct DC {
    static DC *s_dst;
    static DWORD s_idx;
    static DWORD s_low;
    static int s_n;
    int x;
    static void init();
};
DC *DC::s_dst = 0;
DWORD DC::s_idx = (DWORD)-1;
DWORD DC::s_low = (BYTE)-1;
void DC::init() { if (!DC::s_dst) DC::s_dst = new DC; s_n = 1; s_idx = 0; }
int main() { DC::init(); return DC::s_n; }
"""


def test_a_static_member_defined_out_of_line_is_its_static_and_keeps_its_value(tmp_path):
    # cicero's `CUIFBitmapDC *CUIFBitmapDC::s_phdcDst = NULL;`: every use after the
    # definition refers to the DEFINITION's decl, which was not registered, so the body kept
    # `CUIFBitmapDC::s_phdcDst`. And msctfime's `DWORD TLS::s_dwTlsIndex = (DWORD)-1;`
    # shipped as a static of 0.
    if shutil.which("clang++") is None:
        pytest.skip("no host clang++")
    (tmp_path / "t.cpp").write_text(STATIC_SRC)
    out = subprocess.run(
        ["clang++", "-fsyntax-only", "-Xclang", "-ast-dump=json", str(tmp_path / "t.cpp")],
        capture_output=True,
        check=True,
    ).stdout
    ast = json.loads(out)
    cdp._attach_files(ast)
    ex = cpp_model._Extractor(tmp_path, tmp_path)
    ex.scan_overloads(ast["inner"])
    ex.walk(ast["inner"])
    ex.lower_pending()
    init = next(m for m in ex.model.classes["DC"].methods if m.name == "init").lowered
    assert "if (!DC__s_dst) DC__s_dst = " in init and "DC__s_idx = 0" in init
    assert not any(g.name.startswith("s_") for g in ex.model.globals)  # no stray globals

    from rustconv import cpp_app

    classes = cpp_app.render_classes(ex.model)
    assert "pub static mut DC__s_dst: *mut DC = unsafe { core::mem::zeroed() };" in classes
    assert "DC__s_idx: DWORD = (-1i64) as DWORD;" in classes
    assert "DC__s_low: DWORD = ((-1i64) as BYTE) as DWORD;" in classes


def test_a_static_member_initializer_it_cannot_render_is_refused():
    from rustconv import cpp_app
    from rustconv.cpp_model import ClassModel, Field, ModuleModel

    cls = ClassModel("K", "K", "class", "k.h")
    f = Field("s_p", "LPCWSTR", "public", True, 'L"x"')
    with pytest.raises(cpp_app.CppAppError, match="K::s_p"):
        cpp_app._static_member_init(cls, f, "LPCWSTR", ModuleModel())


TEMPLATE_INIT_SRC = """
struct ArrBase { ArrBase(unsigned n) : n(n) {} unsigned n; };
template <typename T_ITEM> struct Arr : ArrBase { Arr() : ArrBase(sizeof(T_ITEM)) {} };
template <typename T_DATA> struct Lock { Lock(int h) : h(h) {} int h; };
template <typename T_DATA> struct CLock : Lock<T_DATA> { CLock(int h) : Lock<T_DATA>(h) {} };
struct Item { int a, b; };
int main() { Arr<Item> a; CLock<Item> l(3); return (int)a.n + l.h; }
"""


def test_a_template_instance_constructs_its_bases_with_its_own_arguments(tmp_path):
    # cicero's CicArray<T_ITEM> (`: CicArrayBase(sizeof(T_ITEM))`) and CicIMCCLock<T_DATA>
    # (`: CIC_IMCC_LOCK<T_DATA>(hIMCC)`): the instance's init list was the pattern's text.
    if shutil.which("clang++") is None:
        pytest.skip("no host clang++")
    (tmp_path / "t.cpp").write_text(TEMPLATE_INIT_SRC)
    out = subprocess.run(
        ["clang++", "-fsyntax-only", "-Xclang", "-ast-dump=json", str(tmp_path / "t.cpp")],
        capture_output=True,
        check=True,
    ).stdout
    ast = json.loads(out)
    cdp._attach_files(ast)
    ex = cpp_model._Extractor(tmp_path, tmp_path)
    ex.scan_overloads(ast["inner"])
    ex.walk(ast["inner"])
    ex.lower_pending()
    ex.apply_instances()
    inits = {
        c.name: dict(m.lowered_inits)
        for c in ex.model.classes.values()
        for m in c.methods
        if m.kind == "ctor" and c.template in ("Arr", "CLock")
    }
    (arr,) = [v for k, v in inits.items() if k.startswith("Arr")]
    (lock,) = [v for k, v in inits.items() if k.startswith("CLock")]
    assert list(arr.values()) == ["(sizeof(Item))"]
    assert list(lock.values()) == ["(h)"]


def test_a_registered_template_is_left_to_its_hand_written_generic(tmp_path):
    # cicero-rust's `CicArray<T>` is the one struct every CTF crate builds against.
    if shutil.which("clang++") is None:
        pytest.skip("no host clang++")
    (tmp_path / "t.cpp").write_text(
        "struct Base { Base(unsigned n) : n(n) {} unsigned n; };\n"
        "template <typename T_ITEM> struct CicArray : Base {\n"
        "    CicArray() : Base(sizeof(T_ITEM)) {}\n"
        "    T_ITEM *at(unsigned i) { return 0; }\n"
        "};\n"
        "struct Obj { int x; };\n"
        "int main() { CicArray<Obj> a; return a.at(0) != 0; }\n"
    )
    out = subprocess.run(
        ["clang++", "-fsyntax-only", "-Xclang", "-ast-dump=json", str(tmp_path / "t.cpp")],
        capture_output=True,
        check=True,
    ).stdout
    ast = json.loads(out)
    cdp._attach_files(ast)
    ex = cpp_model._Extractor(tmp_path, tmp_path)
    ex.scan_overloads(ast["inner"])
    ex.walk(ast["inner"])
    assert not [c for c in ex.model.classes.values() if c.template == "CicArray"]
