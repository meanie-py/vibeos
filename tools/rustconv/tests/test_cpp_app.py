from rustconv import cpp_app, cpp_lower
from rustconv.cpp_model import (
    Base,
    ClassModel,
    Field,
    GlobalVar,
    Method,
    ModuleModel,
    Param,
    _assign_names,
    _ctor_args,
    parse_msg_map,
)


def _m(name, kind="method", params=(), ret="void", **kw):
    return Method(name, kind, [Param(n, t) for n, t in params], ret, **kw)


def _model():
    """sndrec32's shape: a class with two pure-virtual bases, the second at an offset."""
    recv = ClassModel("audio_receiver", "snd::audio_receiver", "class", "r.hpp", polymorphic=True)
    recv.fields = [Field("bytes_received", "unsigned int")]
    recv.methods = [
        _m("audio_receiver", "ctor", body=("r.hpp", 0, 1), lowered="{}"),
        _m(
            "audio_receive",
            params=[("d", "unsigned char *"), ("n", "unsigned int")],
            virtual=True,
            pure=True,
        ),
    ]
    prod = ClassModel("audio_producer", "snd::audio_producer", "class", "p.hpp", polymorphic=True)
    prod.fields = [Field("bytes_played_", "unsigned int")]
    prod.methods = [
        _m(
            "read",
            params=[("b", "BYTE *"), ("n", "unsigned int")],
            ret="unsigned int",
            virtual=True,
            pure=True,
        )
    ]
    buf = ClassModel("audio_membuffer", "snd::audio_membuffer", "class", "m.hpp", polymorphic=True)
    buf.bases = [Base("audio_receiver", "public"), Base("audio_producer", "public")]
    buf.fields = [Field("audio_data", "BYTE *"), Field("cb", "void (*)(unsigned int)")]
    buf.methods = [
        _m(
            "~audio_membuffer",
            "dtor",
            virtual=True,
            body=("m.hpp", 0, 1),
            lowered="{ this_->free_mem_(); }",
        ),
        _m(
            "read",
            params=[("b", "BYTE *"), ("n", "unsigned int")],
            ret="unsigned int",
            body=("m.cpp", 0, 1),
            lowered="{ return 0; }",
        ),
        _m("alloc", params=[("s", "unsigned int")], body=("m.cpp", 0, 1), lowered="{}"),
        _m("alloc", params=[("s", "float")], body=("m.cpp", 0, 1), lowered="{}"),
    ]
    for c in (recv, prod, buf):
        _assign_names(c)
    m = ModuleModel()
    m.classes = {c.qualified: c for c in (recv, prod, buf)}
    return m


def test_overloads_constructors_and_operators_get_unique_names():
    c = _model().classes["snd::audio_membuffer"]
    assert [x.rust_name for x in c.methods] == ["__dtor", "read", "alloc__0", "alloc__1"]
    o = ClassModel("f", "f", "class", "f.h", methods=[_m("operator==", "operator")])
    _assign_names(o)
    assert o.methods[0].rust_name == "op_eq"


def test_layout_vtables_and_thunks():
    rs = cpp_app.render_classes(_model())
    # bases are embedded in declaration order; a root carries the vtable pointer
    assert "pub __base0: audio_receiver,\n    pub __base1: audio_producer," in rs
    assert "pub struct audio_producer {\n    pub __vtbl: *const audio_producer__vtbl," in rs
    # the secondary base's thunk walks back by the field offset, to the BODY
    assert "offset_of!(audio_membuffer, __base1)" in rs
    assert ".read__body(a0, a1)" in rs
    # a pure virtual with no overrider anywhere traps, never jumps through null
    assert 'cpp_pure_virtual_call("snd::audio_receiver::audio_receive")' in rs
    # a real destructor makes Drop (and so no Copy); a destructor-free class is Copy
    assert "impl Drop for audio_membuffer" in rs
    assert "#[derive(Clone, Copy)]\n#[repr(C)]\npub struct audio_producer" in rs
    assert 'Option<unsafe extern "C" fn(u32)>' in rs


def test_virtual_method_is_a_dispatcher_and_its_body_is_separate():
    tus = cpp_app.synthetic_tus(_model())
    text = tus["m.cpp"]
    assert "unsigned int audio_membuffer__read__body(audio_membuffer *this_, BYTE * b" in text
    assert "void audio_membuffer__alloc__1(audio_membuffer *this_, float s)" in text
    rs = cpp_app.render_classes(_model())
    assert "pub unsafe fn read(&mut self, b: *mut BYTE, n: u32) -> u32 {" in rs


def test_constructor_prologue_runs_bases_then_vtables_then_members_in_declaration_order():
    m = _model()
    c = m.classes["snd::audio_membuffer"]
    ctor = _m("audio_membuffer", "ctor", body=("m.hpp", 0, 1), lowered="{ go(); }")
    ctor.lowered_inits = [("cb", "(0)"), ("audio_data", "0")]
    c.methods.insert(0, ctor)
    _assign_names(c)
    text = cpp_app.synthetic_tus(m)["m.hpp"]
    body = text[text.index("audio_membuffer____ctor_0") :]
    order = [
        body.index(s)
        for s in (
            "this_->__base0.__ctor_0();",
            "this_->__base1.__ctor_0();",
            "this_->__init_vtables();",
            "this_->audio_data = 0;",
            "this_->cb = 0;",
            "go();",
        )
    ]
    assert order == sorted(order)


def test_global_objects_are_constructed_before_the_entry_point():
    m = _model()
    fmt = ClassModel("audio_format", "snd::audio_format", "class", "f.hpp")
    fmt.methods = [
        _m(
            "audio_format",
            "ctor",
            params=[("a", "int"), ("b", "int")],
            body=("f.hpp", 0, 1),
            lowered="{}",
        )
    ]
    _assign_names(fmt)
    m.classes[fmt.qualified] = fmt
    m.globals = [GlobalVar("FMT", "snd::audio_format", "main.cpp", "(1, 2)")]
    from rustconv.cpp_model import FreeFunc

    m.functions = [FreeFunc("wWinMain", "int wWinMain(void)", "main.cpp", "{\n    return 0;\n}")]
    text = cpp_app.synthetic_tus(m)["main.cpp"]
    assert text.index("__cxx_global_init();") < text.index("return 0;")
    assert "    FMT.__ctor_0(1, 2);" in text
    assert "pub static mut FMT: audio_format" in cpp_app.render_classes(m)


def test_assemble_moves_translated_bodies_into_their_class():
    rust = (
        "use crate::prelude::*;\n"
        '#[no_mangle]\npub unsafe extern "C" fn audio_membuffer__alloc__1('
        "this_: *mut audio_membuffer, s: f32) {\n"
        "    (*this_).audio_data = core::ptr::null_mut();\n}\n"
        "unsafe fn helper() {}\n"
    )
    out = cpp_app.assemble(rust, _model())
    assert "impl audio_membuffer {" in out
    assert "pub unsafe fn alloc__1(&mut self, s: f32) {" in out
    assert "let this_: *mut Self = self;" in out
    assert "unsafe fn helper() {}" in out and "audio_membuffer__alloc__1" not in out
    assert cpp_app.assemble(out, _model()) == out  # idempotent: replay re-runs it


def test_msg_map_order_is_kept_and_comments_dropped():
    text = """BEGIN_MSG_MAP(CCanvasWindow)
        MESSAGE_HANDLER(WM_SIZE, OnSize) // resize
        COMMAND_ID_HANDLER(IDOK, OnOK)
        CHAIN_MSG_MAP(CBase<CCanvasWindow, x>)
    END_MSG_MAP()"""
    entries = parse_msg_map(text)
    assert [(e.macro, e.args) for e in entries] == [
        ("MESSAGE_HANDLER", ["WM_SIZE", "OnSize"]),
        ("COMMAND_ID_HANDLER", ["IDOK", "OnOK"]),
        ("CHAIN_MSG_MAP", ["CBase<CCanvasWindow, x>"]),
    ]


def test_ctor_args_normalizes_clangs_declarator_ranges():
    assert _ctor_args("UNKNOWN_FORMAT(0, 0, 0)", ("UNKNOWN_FORMAT",)) == "(0, 0, 0)"
    assert _ctor_args("ofn", ("ofn",)) is None
    assert _ctor_args("bytes_received", ("x",)) == "bytes_received"


def test_lowering_edits_apply_right_to_left_keeping_insertion_order():
    assert cpp_lower._apply("a b", [(2, 0, "X"), (2, 0, "Y"), (0, 1, "Z")]) == "Z XYb"


def test_a_derived_class_extends_its_primary_bases_vtable():
    """mspaint: ToolBase owns the vtable, SmoothDrawTool adds OnDraw, PenTool overrides it."""
    tool = ClassModel("ToolBase", "ToolBase", "struct", "t.h", polymorphic=True)
    tool.methods = [
        _m("OnButtonDown", params=[("x", "int")], virtual=True, body=("t.cpp", 0, 1), lowered="{}")
    ]
    smooth = ClassModel("SmoothDrawTool", "SmoothDrawTool", "struct", "t.h", polymorphic=True)
    smooth.bases = [Base("ToolBase", "public")]
    smooth.methods = [_m("OnDraw", params=[("x", "int")], virtual=True, pure=True)]
    pen = ClassModel("PenTool", "PenTool", "struct", "t.h", polymorphic=True)
    pen.bases = [Base("SmoothDrawTool", "public")]
    pen.methods = [
        _m("OnDraw", params=[("x", "int")], virtual=True, body=("t.cpp", 0, 1), lowered="{}")
    ]
    m = ModuleModel()
    for c in (tool, smooth, pen):
        _assign_names(c)
        m.classes[c.qualified] = c
    rs = cpp_app.render_classes(m)
    # the extension begins with the base table, so a ToolBase view of it stays valid
    assert "pub struct SmoothDrawTool__vtbl {\n    pub __base: ToolBase__vtbl," in rs
    assert "pub OnDraw: unsafe fn(*mut ToolBase, i32)," in rs
    # PenTool's table is the extended type, nested, and reaches its OnDraw body
    assert (
        "pub static PenTool__VTBL_b0b0: SmoothDrawTool__vtbl = SmoothDrawTool__vtbl { __base:" in rs
    )
    assert ".OnDraw__body(a0)" in rs
    # dispatch reads the table as the type that declared the slot
    assert "let vt = (*sub).__vtbl as *const SmoothDrawTool__vtbl;" in rs
    assert "offset_of!(PenTool, __base0.__base0)" in rs
