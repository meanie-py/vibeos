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
