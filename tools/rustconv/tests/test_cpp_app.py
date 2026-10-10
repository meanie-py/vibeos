import shutil

import pytest

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


def test_an_sdk_struct_base_is_embedded_by_value():
    """mspaint's BITMAPINFOEX : BITMAPINFO -- a zero-size placeholder would shift every field."""
    c = ClassModel("BITMAPINFOEX", "BITMAPINFOEX", "struct", "d.h")
    c.bases = [Base("BITMAPINFO", "public")]
    c.fields = [Field("bmiColorsExtra", "RGBQUAD[255]")]
    m = ModuleModel()
    m.classes[c.qualified] = c
    rs = cpp_app.render_classes(m)
    assert "pub __base0: BITMAPINFO,\n    pub bmiColorsExtra: [RGBQUAD; 255 as usize]," in rs


_ISLAND_VTBL = """
#[repr(C)]
pub struct IEmptyVolumeCacheCallBackVtbl {
    pub QueryInterface: ::core::option::Option<
        unsafe extern "system" fn(
            This: *mut IEmptyVolumeCacheCallBack,
            riid: REFIID,
            ppvObject: *mut *mut ::core::ffi::c_void,
        ) -> HRESULT,
    >,
    pub AddRef: ::core::option::Option<
        unsafe extern "system" fn(This: *mut IEmptyVolumeCacheCallBack) -> ULONG,
    >,
    pub Release: ::core::option::Option<
        unsafe extern "system" fn(This: *mut IEmptyVolumeCacheCallBack) -> ULONG,
    >,
    pub ScanProgress: ::core::option::Option<
        unsafe extern "system" fn(
            This: *mut IEmptyVolumeCacheCallBack,
            dwlSpaceUsed: DWORDLONG,
            dwFlags: DWORD,
            pcwszStatus: LPCWSTR,
        ) -> HRESULT,
    >,
}
"""


def test_a_class_implementing_a_com_interface_fills_the_islands_vtable():
    """cleanmgr's CEmptyVolumeCacheCallBack : IEmptyVolumeCacheCallBack."""
    cb = ClassModel("CCallBack", "CCallBack", "class", "c.hpp", polymorphic=True)
    cb.bases = [Base("IEmptyVolumeCacheCallBack", "public")]
    cb.fields = [Field("m_cRef", "LONG")]
    for name, params, ret in (
        ("QueryInterface", [("riid", "REFIID"), ("ppv", "void **")], "HRESULT"),
        ("AddRef", [], "ULONG"),
        ("Release", [], "ULONG"),
        ("ScanProgress", [("a", "DWORDLONG"), ("b", "DWORD"), ("c", "LPCWSTR")], "HRESULT"),
    ):
        cb.methods.append(
            _m(
                name,
                params=params,
                ret=ret,
                virtual=True,
                body=("c.hpp", 0, 1),
                lowered="{ return 0; }",
            )
        )
    _assign_names(cb)
    m = ModuleModel()
    m.classes[cb.qualified] = cb
    rs = cpp_app.render_classes(m, frozenset(), _ISLAND_VTBL)
    # the interface struct IS the object's vptr: no vtable of ours, no dispatchers
    assert "pub __vtbl" not in rs and "dispatch through the vtable" not in rs
    assert "pub __base0: IEmptyVolumeCacheCallBack," in rs
    assert (
        'unsafe extern "system" fn CCallBack__IEmptyVolumeCacheCallBack_ScanProgress('
        "This: *mut IEmptyVolumeCacheCallBack, dwlSpaceUsed: DWORDLONG, dwFlags: DWORD, "
        "pcwszStatus: LPCWSTR) -> HRESULT" in rs
    )
    assert "AddRef: Some(CCallBack__IEmptyVolumeCacheCallBack_AddRef)" in rs
    assert "(*this).__base0.lpVtbl = &CCallBack__IEmptyVolumeCacheCallBack_VTBL" in rs
    # the bodies are plain methods (the COM vtable is their dispatch)
    text = cpp_app.synthetic_tus(m, _ISLAND_VTBL)["c.hpp"]
    assert "HRESULT CCallBack__ScanProgress(CCallBack *this_" in text


def test_a_struct_table_is_zipped_with_its_declared_fields():
    """mspaint's CommandToToolMapping: the model guessed `nCommand`/`nTool` for `id`/`tool`."""
    from rustconv.cpp_model import GlobalVar

    c = ClassModel("COMMAND_TO_TOOL", "COMMAND_TO_TOOL", "struct", "t.cpp")
    c.fields = [Field("id", "UINT"), Field("tool", "TOOLTYPE")]
    m = ModuleModel()
    m.classes[c.qualified] = c
    m.globals = [
        GlobalVar(
            "CommandToToolMapping",
            "const COMMAND_TO_TOOL[2]",
            "t.cpp",
            "{\n    { ID_FREESEL, TOOL_FREESEL },\n    { ID_RECTSEL, TOOL_RECTSEL },\n}",
        )
    ]
    rs = cpp_app.render_classes(m)
    assert "pub static CommandToToolMapping: [COMMAND_TO_TOOL; 2] = [" in rs
    assert "COMMAND_TO_TOOL { id: (ID_FREESEL) as _, tool: (TOOL_FREESEL) as _ }" in rs
    assert "CommandToToolMapping" not in "".join(cpp_app.synthetic_tus(m).values())


def test_a_translated_copy_of_a_generated_static_is_dropped():
    rs = "pub static T: [X; 2] = [\n    X { a: 1 },\n    X { a: 2 },\n];\nfn f() { T[0]; }\n"
    assert cpp_app.drop_generated_statics(rs, {"T"}) == "fn f() { T[0]; }\n"
    assert cpp_app.drop_generated_statics(rs, {"U"}) == rs


@pytest.mark.skipif(shutil.which("clang") is None, reason="needs clang for a JSON AST")
def test_an_anonymous_union_member_keeps_its_slot(tmp_path):
    """msctfime's CTextEventSink: `union { UINT m_uFlags; FN m_fn; };` between two fields.
    Skipping it would shift every later field; its members are reached through `__anon0`."""
    import json
    import subprocess

    from rustconv import c_data_port as cdp
    from rustconv import cpp_model

    src = tmp_path / "t.cpp"
    src.write_text(
        "typedef unsigned int UINT; typedef int (*FN)(int);\n"
        "class Sink { long m_cRefs; union { UINT m_uFlags; FN m_fn; }; void *m_after;\n"
        "public: void Set(UINT f); };\n"
        "void Sink::Set(UINT f) { m_uFlags = f; Sink *o = this; o->m_fn = 0; }\n"
    )
    ast = json.loads(
        subprocess.run(
            ["clang", "-x", "c++", "-fsyntax-only", "-Xclang", "-ast-dump=json", str(src)],
            capture_output=True,
            text=True,
            check=True,
        ).stdout
    )
    cdp._attach_files(ast)
    ex = cpp_model._Extractor(tmp_path, tmp_path)
    ex.walk(ast["inner"])
    ex.lower_pending()
    sink = ex.model.classes["Sink"]
    assert [f.name for f in sink.fields] == ["m_cRefs", "__anon0", "m_after"]
    assert ex.model.classes["Sink__anon0"].tag == "union"
    (body,) = [m.lowered for m in sink.methods if m.name == "Set"]
    assert "this_->__anon0.m_uFlags = f" in body and "o->__anon0.m_fn = 0" in body
    assert "pub union Sink__anon0 {" in cpp_app.render_classes(ex.model)


@pytest.mark.skipif(shutil.which("clang") is None, reason="needs clang for a JSON AST")
def test_a_nothrow_tag_new_lowers_like_new_and_a_buffer_placement_does_not(tmp_path):
    """CTF's `new(cicNoThrow) T(args)` (~130 audit rejects across msutb/cicuif/msctf)."""
    import json
    import subprocess

    from rustconv import c_data_port as cdp
    from rustconv import cpp_model

    src = tmp_path / "t.cpp"
    src.write_text(
        "typedef __SIZE_TYPE__ size_t; struct CicNoThrow { };\n"
        "#define cicNoThrow CicNoThrow{}\n"
        "void* operator new(size_t size, const CicNoThrow&) noexcept;\n"
        "void* operator new(size_t, void* p) noexcept { return p; }\n"
        "class Item { public: int v; Item(int x) : v(x) {} };\n"
        "Item* make(int a) { Item *p = new(cicNoThrow) Item(a); return p; }\n"
        "Item* inplace(void *buf) { return new(buf) Item(1); }\n"
    )
    ast = json.loads(
        subprocess.run(
            [
                "clang",
                "--target=i686-w64-mingw32",
                "-x",
                "c++",
                "-fsyntax-only",
                "-Xclang",
                "-ast-dump=json",
                str(src),
            ],
            capture_output=True,
            text=True,
            check=True,
        ).stdout
    )
    cdp._attach_files(ast)
    ex = cpp_model._Extractor(tmp_path, tmp_path)
    ex.walk(ast["inner"])
    ex.lower_pending()
    by = {f.name: f.lowered for f in ex.model.functions}
    # The tag's allocator, by its mangled name, so a C++ half and a Rust half share a heap.
    assert "Item *p = Item__new_in_0(cpp_alloc_CicNoThrow::<Item>(), a);" in by["make"]
    assert "new(buf) Item(1)" in by["inplace"]
    assert ex.model.allocators["new:CicNoThrow"] == "_ZnwjRK10CicNoThrow"
    rs = cpp_app.render_classes(ex.model)
    assert "if __p.is_null() { return __p; } (*__p).__ctor_0(x)" in rs
    assert 'link_name = "_ZnwyRK10CicNoThrow"' in rs
    assert "pub unsafe fn cpp_alloc_CicNoThrow<T>()" in rs


def test_an_override_two_levels_below_its_virtual_is_still_a_body():
    """RubberTool : SmoothDrawTool : ToolBase, OnSpecialTweak declared only in ToolBase."""
    from rustconv.cpp_model import Base, Method

    def cls(name, base=None, methods=()):
        c = ClassModel(name, name, "struct", "t.cpp")
        c.bases = [Base(base, "public", False)] if base else []
        c.methods = list(methods)
        c.polymorphic = True
        return c

    tweak = Method("OnSpecialTweak", "method", [], "void", virtual=True)
    over = Method("OnSpecialTweak", "method", [], "void")
    m = ModuleModel()
    for c in (
        cls("ToolBase", methods=[tweak]),
        cls("SmoothDrawTool", "ToolBase"),
        cls("RubberTool", "SmoothDrawTool", [over]),
    ):
        m.classes[c.qualified] = c
    h = cpp_app.Hierarchy(m)
    assert h.is_virtual(m.classes["RubberTool"], over)


def test_an_invented_this_member_for_a_global_is_dropped():
    from rustconv.cpp_model import GlobalVar

    c = ClassModel("CMainWindow", "CMainWindow", "class", "t.cpp")
    c.fields = [Field("m_hMenu", "HMENU")]
    m = ModuleModel()
    m.classes[c.qualified] = c
    m.globals = [GlobalVar("selectionModel", "SelectionModel", "t.cpp", None)]
    rs = "(*this_).selectionModel.moveSelection(-1, 0); (*this_).m_hMenu = x;"
    assert (
        cpp_app.unthis_globals(rs, m)
        == "selectionModel.moveSelection(-1, 0); (*this_).m_hMenu = x;"
    )


def test_a_bare_call_to_a_unique_static_method_is_qualified():
    c = ClassModel("CImageDx", "CImageDx", "class", "t.cpp")
    c.methods = [_m("_shared", ret="SHARED *", static=True), _m("Attach")]
    m = ModuleModel()
    m.classes[c.qualified] = c
    rs = "get((*_shared()).x); CImageDx::_shared(); o._shared(); fn _shared() {}"
    out = cpp_app.qualify_static_calls(rs, m)
    assert out.startswith("get((*CImageDx::_shared()).x); CImageDx::_shared(); o._shared();")


@pytest.mark.skipif(shutil.which("clang") is None, reason="needs clang for a JSON AST")
def test_an_implicit_conversion_operator_is_an_explicit_call(tmp_path):
    """mspaint's `IsWindowVisible(textEditWindow)` through CWindow::operator HWND()."""
    import json
    import subprocess

    from rustconv import c_data_port as cdp
    from rustconv import cpp_model

    src = tmp_path / "t.cpp"
    src.write_text(
        "typedef struct HWND__ *HWND; int IsWindowVisible(HWND);\n"
        "class CWindow { public: HWND m_hWnd; operator HWND() const { return m_hWnd; } };\n"
        "class CEdit : public CWindow { public: void Go(); };\n"
        "CEdit textEditWindow;\n"
        "void CEdit::Go() { IsWindowVisible(textEditWindow); IsWindowVisible(*this); }\n"
    )
    ast = json.loads(
        subprocess.run(
            ["clang", "-x", "c++", "-fsyntax-only", "-Xclang", "-ast-dump=json", str(src)],
            capture_output=True,
            text=True,
            check=True,
        ).stdout
    )
    cdp._attach_files(ast)
    ex = cpp_model._Extractor(tmp_path, tmp_path)
    ex.walk(ast["inner"])
    ex.lower_pending()
    (go,) = [m.lowered for m in ex.model.classes["CEdit"].methods if m.name == "Go"]
    assert go == (
        "{ IsWindowVisible((textEditWindow).op_to_HWND()); "
        "IsWindowVisible((*this_).op_to_HWND()); }"
    )
