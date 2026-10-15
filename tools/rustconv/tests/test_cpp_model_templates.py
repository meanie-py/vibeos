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
