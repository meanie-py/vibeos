import shutil

import pytest

from rustconv import cpp_app, cpp_lower
from rustconv.cpp_model import (
    Base,
    ClassModel,
    Field,
    FreeFunc,
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
    # ...and a polymorphic destructor runs with the class's OWN vtables, as in C++ (rapps'
    # CPointerArray destructor reached the derived OnRemoveItem and double-deleted)
    assert "unsafe { self.__init_vtables(); self.__dtor__body() }" in rs
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
    # REFIID is a reference in the C++ method and a pointer in the slot: reborrowed.
    assert ".QueryInterface(&*riid, ppvObject) }" in rs
    assert "(*this).__base0.lpVtbl = &CCallBack__IEmptyVolumeCacheCallBack_VTBL" in rs
    # the bodies are plain methods (the COM vtable is their dispatch)
    text = cpp_app.synthetic_tus(m, _ISLAND_VTBL)["c.hpp"]
    assert "HRESULT CCallBack__ScanProgress(CCallBack *this_" in text


def test_a_com_slot_takes_the_overload_with_its_arity():
    """mmc_new's CConsoleWnd implements IConsole2::Expand(item, bExpand) AND
    IConsoleNameSpace2::Expand(item): each vtable gets the overload whose arity matches."""
    island = (
        "#[repr(C)]\npub struct IAVtbl {\n    pub Expand: ::core::option::Option<\n"
        '        unsafe extern "system" fn(This: *mut IA, h: HSCOPEITEM, b: BOOL) -> HRESULT,\n'
        "    >,\n}\n"
        "#[repr(C)]\npub struct IBVtbl {\n    pub Expand: ::core::option::Option<\n"
        '        unsafe extern "system" fn(This: *mut IB, h: HSCOPEITEM) -> HRESULT,\n'
        "    >,\n}\n"
    )
    c = ClassModel("W", "W", "class", "w.h", polymorphic=True)
    c.bases = [Base("IA", "public"), Base("IB", "public")]
    for params in ([("h", "HSCOPEITEM"), ("b", "BOOL")], [("h", "HSCOPEITEM")]):
        c.methods.append(
            _m(
                "Expand",
                params=params,
                ret="HRESULT",
                virtual=True,
                body=("w.h", 0, 1),
                lowered="{ return 0; }",
            )
        )
    _assign_names(c)
    m = ModuleModel()
    m.classes[c.qualified] = c
    rs = cpp_app.render_classes(m, frozenset(), island)
    one, two = c.methods[0].rust_name, c.methods[1].rust_name
    assert one != two
    assert f".{one}(h, b) }}" in rs
    assert f".{two}(h) }}" in rs


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


@pytest.mark.skipif(shutil.which("clang") is None, reason="needs clang for a JSON AST")
def test_a_static_method_called_through_an_object_is_an_associated_call(tmp_path):
    """mspaint's `toolsModel.CreateBrush(..)`, a static method: Rust needs `ToolsModel::`."""
    import json
    import subprocess

    from rustconv import c_data_port as cdp
    from rustconv import cpp_model

    src = tmp_path / "t.cpp"
    src.write_text(
        "class TM { public: static int CreateBrush(int a); int x; };\nTM toolsModel;\n"
        "class P { public: TM m; void Go(); };\n"
        "void P::Go() { toolsModel.CreateBrush(1); m.CreateBrush(toolsModel.x); }\n"
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
    (go,) = [m.lowered for m in ex.model.classes["P"].methods if m.name == "Go"]
    assert go == "{ TM::CreateBrush(1); TM::CreateBrush(toolsModel.x); }"


def test_a_cpp_global_qualifier_is_not_a_crate_path():
    rs = "if ::IsWindow(h) != 0 { ::core::ptr::null::<u8>(); x.f(::GetDC(h)); }"
    assert cpp_app.drop_global_qualifier(rs) == (
        "if IsWindow(h) != 0 { ::core::ptr::null::<u8>(); x.f(GetDC(h)); }"
    )


def test_a_function_scope_using_namespace_maps_bare_names_to_the_island():
    from rustconv.cpp_app import _apply_using

    chunk = "void f(){\n    using namespace Gdiplus;\n    GpBitmap *p = NULL; x.GpBitmap;\n}"
    out = _apply_using(chunk, frozenset({"Gdiplus_GpBitmap"}))
    assert out == "void f(){\n    Gdiplus_GpBitmap *p = NULL; x.GpBitmap;\n}"


@pytest.mark.skipif(shutil.which("clang") is None, reason="needs clang for a JSON AST")
def test_overloaded_free_functions_keep_both_and_calls_pick_the_resolved_one(tmp_path):
    """mspaint's drawing.h: Fill(.., COLORREF) and Fill(.., HBRUSH). One name dropped one."""
    import json
    import subprocess

    from rustconv import c_data_port as cdp
    from rustconv import cpp_model

    src = tmp_path / "t.cpp"
    src.write_text(
        "typedef unsigned long COLORREF; typedef struct HBRUSH__ *HBRUSH; typedef void* HDC;\n"
        "void Fill(HDC hdc, long x, COLORREF color);\nvoid Fill(HDC hdc, long x, HBRUSH b);\n"
        "void Fill(HDC hdc, long x, COLORREF color) { }\n"
        "void Fill(HDC hdc, long x, HBRUSH b) { Fill(hdc, x, (COLORREF)0); }\n"
        "void Use(HDC h, HBRUSH b) { Fill(h, 1, b); }\n"
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
    ex.scan_overloads(ast["inner"])
    ex.walk(ast["inner"])
    ex.lower_pending()
    by = {f.name: f for f in ex.model.functions}
    assert set(by) == {"Fill__0", "Fill__1", "Use"}
    assert by["Fill__1"].signature.startswith("void Fill__1(")
    assert by["Fill__1"].lowered == "{ Fill__0(hdc, x, (COLORREF)0); }"
    assert by["Use"].lowered == "{ Fill__1(h, 1, b); }"


def test_assemble_leaves_extern_declarations_alone():
    """atlimage: a worker's `extern "C" { fn CImage__s_x(..) -> i32; }` was moved into impl."""
    c = ClassModel("CImage", "CImage", "class", "t.cpp")
    m = ModuleModel()
    m.classes[c.qualified] = c
    rs = (
        'extern "C" {\n    fn CImage__s_x(n: u32) -> i32;\n}\n'
        "pub unsafe fn CImage__Go(this_: *mut CImage) {\n    CImage__s_x(1);\n}\n"
    )
    out = cpp_app.assemble(rs, m)
    assert 'extern "C" {\n    fn CImage__s_x(n: u32) -> i32;\n}\n' in out
    assert "impl CImage {" in out and "pub unsafe fn Go(&mut self)" in out


def test_external_reach_follows_resolved_calls_and_the_dtor():
    """mspaint: a static `CImage::GetImporterFilterString` reaches the helper it calls, not
    the CImage methods the module never calls; and CImage's dtor (always translated, Drop
    calls it) reaches `Destroy`."""
    img = ClassModel("CImage", "ATL::CImage", "class", "atlimage.h", external=True)
    img.methods = [
        _m("CImage", "ctor", rust_name="__ctor_0"),
        _m("~CImage", "dtor", rust_name="__dtor", calls=[("CImage", "Destroy")]),
        _m("Destroy", rust_name="Destroy"),
        _m(
            "GetImporterFilterString",
            static=True,
            rust_name="GetImporterFilterString",
            calls=[("CImage", "BuildCodecFilterString")],
        ),
        _m("BuildCodecFilterString", static=True, rust_name="BuildCodecFilterString"),
        _m("Attach", rust_name="Attach"),
    ]
    model = ModuleModel(classes={"ATL::CImage": img})
    model.functions = [
        FreeFunc("f", "void f()", "m.cpp", "{}", calls=[("CImage", "GetImporterFilterString")])
    ]
    reached = cpp_app._reached_by_calls(model, [img])
    assert reached == {
        ("CImage", "GetImporterFilterString"),
        ("CImage", "BuildCodecFilterString"),
        ("CImage", "Destroy"),
    }


@pytest.mark.skipif(shutil.which("clang") is None, reason="needs clang for a JSON AST")
def test_a_functional_cast_to_a_scalar_is_a_c_cast(tmp_path):
    """dib.cpp's `INT(cx * eTan)`: left as written, the model called `INT(..)` like a fn."""
    import json
    import subprocess

    from rustconv import c_data_port as cdp
    from rustconv import cpp_model

    src = tmp_path / "t.cpp"
    src.write_text(
        "typedef int INT;\nclass P { public: void Go(double e); int d; };\n"
        "void P::Go(double e) { d = INT(3 * e) + 1; }\n"
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
    (go,) = [m.lowered for m in ex.model.classes["P"].methods if m.name == "Go"]
    assert go == "{ this_->d = ((INT)(3 * e)) + 1; }"


def test_text_macros_are_wide_literals_in_a_unicode_tu():
    out = cpp_lower.unicode_text_macros(
        's += TEXT(\';\'); s += _T(" (" ); f(TEXT("a\\"b"), TEXTX(1));'
    )
    assert out == 's += L\';\'; s += L" ("; f(L"a\\"b", TEXTX(1));'


@pytest.mark.skipif(shutil.which("clang") is None, reason="needs clang for a JSON AST")
def test_an_object_macro_over_a_callers_local_is_inlined(tmp_path):
    """palette.cpp's `#define Y_MARGIN ((rcClient.bottom / 2) - CXY_COLORBOX)`: it names
    the caller's local, which no Rust item can, so it is inlined where it was expanded."""
    import json
    import subprocess

    from rustconv import c_data_port as cdp
    from rustconv import cpp_model

    src = tmp_path / "t.cpp"
    src.write_text(
        "#define CXY 5\n#define Y_MARGIN ((rc.bottom / 2) - CXY) /* centre */\n"
        "struct R { int bottom; };\nclass P { public: int Go(R rc); };\n"
        "int P::Go(R rc) { int dy = 3 - Y_MARGIN; return CXY + dy; }\n"
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
    (go,) = [m.lowered for m in ex.model.classes["P"].methods if m.name == "Go"]
    assert go == "{ int dy = 3 - (((rc.bottom / 2) - CXY)); return CXY + dy; }"


@pytest.mark.skipif(shutil.which("clang") is None, reason="needs clang for a JSON AST")
def test_a_token_pasted_local_is_left_to_the_macro(tmp_path):
    """cleanmgr: a `##`-pasted name is spelled in clang's `<scratch space>`, which is no
    file and no `#define` line; the inliner must leave it alone, not open it."""
    import json
    import subprocess

    from rustconv import c_data_port as cdp
    from rustconv import cpp_model

    src = tmp_path / "t.cpp"
    src.write_text(
        "#define V(n) v##n\nclass P { public: int Go(int v1); };\n"
        "int P::Go(int v1) { return V(1); }\n"
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
    (go,) = [m.lowered for m in ex.model.classes["P"].methods if m.name == "Go"]
    assert go == "{ return V(1); }"


@pytest.mark.skipif(shutil.which("clang") is None, reason="needs clang for a JSON AST")
def test_a_ctor_init_through_a_macro_call_keeps_its_arguments(tmp_path):
    """selectionmodel.cpp's `m_rgbBack(RGB(255, 255, 255))` was lowered as `RGB`."""
    import json
    import subprocess

    from rustconv import c_data_port as cdp
    from rustconv import cpp_model

    src = tmp_path / "t.cpp"
    src.write_text(
        "#define RGB(r, g, b) ((r) | ((g) << 8) | ((b) << 16))\n"
        "class S { public: S(); int m_rgb; };\nS::S() : m_rgb(RGB(255, 255, 255)) {}\n"
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
    (ctor,) = [m for m in ex.model.classes["S"].methods if m.kind == "ctor"]
    assert ctor.lowered_inits == [("m_rgb", "RGB(255, 255, 255)")]


def test_a_header_static_function_is_visible_to_the_crate():
    rs = (
        'unsafe extern "C" fn Zoomed__0(x: i32) -> i32 {\n    fn inner() {}\n    x\n}\n'
        "pub fn Keep() {}\nfn Plain() {}\n"
    )
    out = cpp_app.publish_header_statics(rs)
    assert out.startswith('pub unsafe extern "C" fn Zoomed__0')
    assert "    fn inner() {}" in out and "pub fn Keep()" in out and "pub fn Plain()" in out
    assert cpp_app.publish_header_statics(out) == out


def test_a_nested_module_class_field_is_the_rendered_class_not_bindgens():
    tm = ClassModel("ToolsModel", "ToolsModel", "class", "t.h")
    island = frozenset({"ToolsModel_VirtualBrush", "ToolsModel_FN_X"})
    out = cpp_app._nested_names("VirtualBrush", tm, island, frozenset({"VirtualBrush"}))
    assert out == "VirtualBrush"
    assert (
        cpp_app._nested_names("FN_X", tm, island, frozenset({"VirtualBrush"})) == "ToolsModel_FN_X"
    )


def test_a_nested_type_in_a_unit_signature_and_body_is_the_islands():
    """registry.cpp's `SetWallpaper(LPCWSTR, WallpaperStyle style)` and atlimagedx's
    `FN_Startup Startup = ..` reached the model bare; the island spells them Outer_Inner."""
    rs = ClassModel("RegistrySettings", "RegistrySettings", "class", "r.h")
    rs.methods = [
        _m(
            "SetWallpaper",
            params=[
                ("szFileName", "LPCWSTR"),
                ("style", "WallpaperStyle"),
                ("s", "CSimpleString &"),
            ],
            static=True,
            rust_name="SetWallpaper",
            body=("r.cpp", 0, 1),
            lowered="{ FN_Startup f; }",
        )
    ]
    model = ModuleModel(classes={"RegistrySettings": rs})
    island = (
        "pub type RegistrySettings_WallpaperStyle = u32;\n"
        "pub type RegistrySettings_FN_Startup = u32;\n"
        "pub type RegistrySettings_CSimpleString = u32;\n"
    )
    unit = cpp_app.synthetic_tus(model, island)["r.cpp"]
    # An atl_rust type keeps its bare name even where the island has an Outer_ spelling.
    assert "LPCWSTR szFileName, RegistrySettings_WallpaperStyle style, CSimpleString & s)" in unit
    assert "{ RegistrySettings_FN_Startup f; }" in unit


@pytest.mark.skipif(shutil.which("clang") is None, reason="needs clang for a JSON AST")
def test_a_field_of_a_plain_struct_base_goes_through_the_base(tmp_path):
    """CRect derives from RECT: `m_rcOld.left` is the base subobject's field. Without the
    path Rust reached it through the generated Deref (dangerous_implicit_autorefs)."""
    import json
    import subprocess

    from rustconv import c_data_port as cdp
    from rustconv import cpp_model

    src = tmp_path / "t.cpp"
    src.write_text(
        "struct tagRECT { int left; int top; };\n"
        "class CRect : public tagRECT { public: int Width(); };\n"
        "class S { public: CRect m_rc; int Go(); };\n"
        "int S::Go() { return m_rc.left + m_rc.Width(); }\n"
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
    (go,) = [m.lowered for m in ex.model.classes["S"].methods if m.name == "Go"]
    assert go == "{ return this_->m_rc.__base0.left + this_->m_rc.Width(); }"


def _atl_model(*classes):
    cwnd = ClassModel("CWindow", "ATL::CWindow", "class", "atlwin.h", external=True)
    cwnd.fields = [Field("m_hWnd", "HWND")]
    for c in (cwnd, *classes):
        _assign_names(c)
    m = ModuleModel()
    m.classes = {c.qualified: c for c in (cwnd, *classes)}
    return m


def test_a_property_page_gets_rosdlgs_glue_and_its_overrides():
    """cleanmgr's CCleanMgrProperties : CPropertyPageImpl<T>: Create builds the page, the
    class's OnApply overrides the trait default, and CHAIN_MSG_MAP reaches the base's map."""
    page = ClassModel("P", "P", "struct", "p.cpp")
    page.bases = [Base("CPropertyPageImpl<P>", "public")]
    page.constants = [("IDD", 101)]
    page.methods = [_m("OnApply", ret="int", body=("p.cpp", 0, 1), lowered="{ return 0; }")]
    page.msg_map = parse_msg_map(
        "BEGIN_MSG_MAP(P)\n CHAIN_MSG_MAP(CPropertyPageImpl<P>)\nEND_MSG_MAP()"
    )
    rs = cpp_app.render_classes(_atl_model(page))
    assert "pub __base0: CWindow," in rs
    assert "atl_rust::win::create_property_page(self as *mut Self, 101 as u16" in rs
    assert "impl atl_rust::win::PropertyPage for P {" in rs
    assert "unsafe fn on_apply(&mut self) -> i32 { self.OnApply() as i32 }" in rs
    # A hook the class does not declare keeps the C++ base's default (the trait's).
    assert "on_set_active" not in rs
    assert "atl_rust::win::property_page_map(self as *mut Self, hwnd, msg, l, result)" in rs


def test_a_message_map_entry_can_name_a_member_objects_handler():
    """certmgr's MESSAGE_HANDLER(WM_SIZE, m_Splitter.OnSize): the handler is a method of a
    module class held by value, so the dispatch calls through the field."""
    hp = [("u", "UINT"), ("w", "WPARAM"), ("l", "LPARAM"), ("b", "BOOL &")]
    spl = ClassModel("CSplitter", "CSplitter", "struct", "s.hpp")
    spl.methods = [_m("OnSize", params=hp, ret="LRESULT", body=("s.hpp", 0, 1), lowered="{}")]
    dlg = ClassModel("D", "D", "class", "d.cpp")
    dlg.bases = [Base("CDialogImpl<D>", "public")]
    dlg.constants = [("IDD", 101)]
    dlg.fields = [Field("m_Splitter", "CSplitter")]
    dlg.msg_map = parse_msg_map(
        "BEGIN_MSG_MAP(D)\n MESSAGE_HANDLER(WM_SIZE, m_Splitter.OnSize)\nEND_MSG_MAP()"
    )
    rs = cpp_app.render_classes(_atl_model(spl, dlg))
    assert "*result = self.m_Splitter.OnSize(msg as _, w as _, l as _, &mut handled)" in rs


def test_an_exe_module_gets_win_main_and_its_overrides(tmp_path, monkeypatch):
    """cleanmgr's CCleanMgrModule : CAtlExeModuleT<T> overrides ParseCommandLine and Run;
    the rest (InitializeCom, the message loop) stay ATL's."""
    monkeypatch.setattr(cpp_app, "_REPO", tmp_path)
    (tmp_path / "m.cpp").write_text("class M : public CAtlExeModuleT<M> {};\n")
    mod = ClassModel("M", "M", "class", "m.cpp")
    mod.bases = [Base("ATL::CAtlExeModuleT<M>", "public")]
    mod.methods = [
        _m("ParseCommandLine", params=[("l", "LPCTSTR"), ("hr", "HRESULT *")], ret="bool"),
        _m("Run", params=[("n", "int")], ret="HRESULT"),
    ]
    rs = cpp_app.render_classes(_atl_model(mod))
    assert "pub __base0: atl_rust::exe::CAtlExeModule," in rs
    assert "atl_rust::exe::win_main(self as *mut Self, show as i32)" in rs
    assert "impl atl_rust::exe::ExeModule for M {" in rs
    assert (
        "unsafe fn parse_command_line(&mut self, cmd_line: *const u16, ret_code: *mut i32) "
        "-> bool { (self.ParseCommandLine(cmd_line as _, ret_code as _)) as i32 != 0 }"
    ) in rs
    assert "unsafe fn run(&mut self, show: i32) -> i32 { self.Run(show as _) as i32 }" in rs
    assert "initialize_com" not in rs


def test_an_exe_module_with_a_com_object_map_is_refused(tmp_path, monkeypatch):
    """atl_rust registers no class objects: a module that declares some must not get glue
    that silently skips them."""
    monkeypatch.setattr(cpp_app, "_REPO", tmp_path)
    (tmp_path / "m.cpp").write_text("OBJECT_ENTRY_AUTO(CLSID_X, CX)\n")
    mod = ClassModel("M", "M", "class", "m.cpp")
    mod.bases = [Base("ATL::CAtlExeModuleT<M>", "public")]
    with pytest.raises(cpp_app.CppAppError, match="COM object map"):
        cpp_app.render_classes(_atl_model(mod))


def test_a_callback_static_member_keeps_the_system_abi():
    """cleanmgr: `static BOOL CALLBACK EnumSingleInstanceCallback(HWND, LPARAM)` is handed to
    EnumWindows; assembling it as a plain Rust fn made the call site a type error."""
    c = ClassModel("M", "M", "class", "m.cpp")
    c.methods = [
        _m("Cb", params=[("h", "HWND"), ("l", "LPARAM")], ret="BOOL", static=True, abi="system"),
        _m("Plain", static=True),
    ]
    _assign_names(c)
    m = ModuleModel()
    m.classes[c.qualified] = c
    out = cpp_app.assemble(
        "unsafe fn M__Cb(h: HWND, l: LPARAM) -> BOOL { 1 }\nunsafe fn M__Plain() {}\n", m
    )
    assert 'pub unsafe extern "system" fn Cb(h: HWND, l: LPARAM) -> BOOL' in out
    assert "pub unsafe fn Plain()" in out


def test_assemble_files_a_nested_class_method_under_the_nested_class():
    """msctfime's `CTextEventSink__anon0____ctor_0` belongs to the union, not CTextEventSink."""
    m = ModuleModel()
    for n in ("CTextEventSink", "CTextEventSink__anon0"):
        m.classes[n] = ClassModel(n, n, "struct", "s.h")
    rs = cpp_app.assemble(
        "pub unsafe fn CTextEventSink__anon0____ctor_0(this_: *mut CTextEventSink__anon0) {}\n", m
    )
    assert "impl CTextEventSink__anon0 {" in rs
    assert "pub unsafe fn __ctor_0(&mut self" in rs


def test_a_type_nested_in_a_class_is_reachable_by_its_bare_name():
    """msctfime's sinks.h: `typedef INT (CALLBACK *FN_COMPARE)(..)` inside a class."""
    m = ModuleModel()
    for n in ("CActiveLanguageProfileNotifySink", "CicProfile"):
        m.classes[n] = ClassModel(n, n, "class", "s.h")
    island = (
        "pub type CActiveLanguageProfileNotifySink_FN_COMPARE = Option<fn()>;\n"
        "pub struct CicProfile_tagLANG_PROF_ENUM_ARG {\n    pub a: u32,\n}\n"
        "pub type CicProfile_LANG_PROF_ENUM_ARG = CicProfile_tagLANG_PROF_ENUM_ARG;\n"
        "pub type LANG_PROF_ENUM_ARG = u32;\n"
    )
    glue = cpp_app._render_island_glue(m, island)
    assert "pub type FN_COMPARE = CActiveLanguageProfileNotifySink_FN_COMPARE;" in glue
    assert "pub type tagLANG_PROF_ENUM_ARG = CicProfile_tagLANG_PROF_ENUM_ARG;" in glue
    assert "pub type LANG_PROF_ENUM_ARG =" not in glue  # the island already has one
    m.classes["tagLANG_PROF_ENUM_ARG"] = ClassModel(
        "tagLANG_PROF_ENUM_ARG", "tagLANG_PROF_ENUM_ARG", "struct", "p.h"
    )
    glue = cpp_app._render_island_glue(m, island)
    assert "pub type tagLANG_PROF_ENUM_ARG =" not in glue  # cppapp renders that class


def test_island_glue_gives_a_packed_interface_no_deref():
    """shlobj.h's `#pragma pack`: `&self._base` of a packed field is E0793."""
    island = (
        "#[repr(C, packed)]\n#[derive(Debug, Copy, Clone)]\n"
        "pub struct IShellFolder {\n    pub _base: IUnknown,\n}\n"
        "#[repr(C)]\npub struct ITfLangBarMgr_P {\n    pub _base: ITfLangBarMgr,\n}\n"
    )
    glue = cpp_app._render_island_glue(ModuleModel(), island)
    assert "Deref for IShellFolder" not in glue
    assert "Deref for ITfLangBarMgr_P" in glue


def test_a_module_operator_is_named_translatably_and_exported_under_its_symbol():
    from types import SimpleNamespace

    from rustconv.cpp_model import _Extractor

    ex = SimpleNamespace(_overloads={})
    assert _Extractor.fn_rust_name(ex, {"name": "operator new"}) == "cxx_operator_new"  # type: ignore[arg-type]
    assert _Extractor.fn_rust_name(ex, {"name": "operator delete[]"}) == (  # type: ignore[arg-type]
        "cxx_operator_delete_array"
    )
    m = ModuleModel()
    m.allocators = {"new:CicNoThrow": "_ZnwjRK10CicNoThrow", "delete": "_ZdlPv"}
    m.allocator_fns = {"new:CicNoThrow": "cxx_operator_new__0", "delete": "cxx_operator_delete"}
    rs = cpp_app._allocator_exports(m)
    assert 'export_name = "_ZnwjRK10CicNoThrow"' in rs
    assert 'export_name = "_ZnwyRK10CicNoThrow"' in rs  # size_t is `y` on 64-bit
    assert "cxx_operator_new__0(n as _, core::mem::transmute(tag))" in rs
    assert 'export_name = "_ZdlPv")]' in rs and "cxx_operator_delete(p as _)" in rs
    assert ModuleModel.from_json(m.to_json()).allocator_fns == m.allocator_fns
    assert cpp_app._allocator_exports(ModuleModel()) == ""


def _pair():
    """msctfime's shape: CCompFrameWindow : CUIFWindow, both the module's."""
    m = ModuleModel()
    base = ClassModel("CUIFWindow", "CUIFWindow", "class", "w.h")
    base.fields = [Field("m_hWnd", "HWND")]
    derived = ClassModel("CCompFrameWindow", "CCompFrameWindow", "class", "c.h")
    derived.bases = [Base("CUIFWindow", "public")]
    for c in (base, derived):
        m.classes[c.qualified] = c
    return m


def test_a_class_derefs_to_its_own_modules_primary_base():
    rs = cpp_app.render_classes(_pair())
    assert "impl core::ops::Deref for CCompFrameWindow { type Target = CUIFWindow;" in rs
    assert "impl core::ops::DerefMut for CCompFrameWindow {" in rs


def test_a_class_over_a_dependencys_class_does_not_derive_copy():
    m = ModuleModel()
    grip = ClassModel("CDefCompFrameGripper", "CDefCompFrameGripper", "class", "g.h")
    grip.bases = [Base("CUIFGripper", "public")]
    held = ClassModel("CHolder", "CHolder", "class", "h.h")
    held.fields = [Field("m_grip", "const CUIFGripper")]
    plain = ClassModel("CPlain", "CPlain", "class", "p.h")
    plain.fields = [Field("x", "int")]
    for c in (grip, held, plain):
        m.classes[c.qualified] = c
    island = "pub use cicero_rust::cicero::cpp_classes::CUIFGripper; // cicero-rust owns it\n"
    rs = cpp_app.render_classes(m, frozenset(), island)
    for name in ("CDefCompFrameGripper", "CHolder"):
        assert f"#[repr(C)]\npub struct {name} {{" in rs
        assert f"Copy)]\n#[repr(C)]\npub struct {name} {{" not in rs
    assert "#[derive(Clone, Copy)]\n#[repr(C)]\npub struct CPlain {" in rs


def test_a_reference_member_of_an_aggregate_static_takes_the_address():
    assert cpp_app._is_reference("REFGUID") and cpp_app._is_reference("const GUID &")
    assert not cpp_app._is_reference("GUID") and not cpp_app._is_reference("REFERENCE_TIME_x")
    m = ModuleModel()
    mb = ClassModel("MODEBIAS", "MODEBIAS", "struct", "m.cpp")
    mb.fields = [Field("m_guid", "REFGUID"), Field("m_dwBias", "DWORD")]
    m.classes[mb.qualified] = mb
    g = GlobalVar("g_ModeBias", "const MODEBIAS [1]", "m.cpp", "{ { GUID_MODEBIAS_NONE, 0 } }")
    rs = cpp_app.aggregate_static(g, cpp_app.Hierarchy(m), m)
    assert rs is not None
    assert "m_guid: core::ptr::addr_of!(GUID_MODEBIAS_NONE) as _" in rs
    assert "m_dwBias: (0) as _" in rs


def test_a_constructor_parameter_named_like_a_rust_keyword_is_escaped():
    """certmgr's CStoreList(StoreType type): the __make/__new wrappers took `type` bare."""
    c = ClassModel("CStoreList", "CStoreList", "class", "s.hpp")
    c.methods = [
        _m("CStoreList", "ctor", params=[("type", "StoreType")], body=("s.hpp", 0, 1), lowered="{}")
    ]
    m = ModuleModel()
    m.classes[c.qualified] = c
    rs = cpp_app.render_classes(m)
    assert "CStoreList__make_0(r#type: StoreType)" in rs
    assert "(type:" not in rs


def test_a_global_with_a_no_argument_constructor_is_initialized_by_cpp_classes():
    """certmgr's `CCertMgrModule _AtlModule;`: the model turned the C stub's
    `_AtlModule.__ctor_0();` into an invented extern twice, so it is rendered, not bought."""
    from rustconv.cpp_model import GlobalVar

    mod = ClassModel("M", "M", "class", "m.cpp")
    mod.methods = [_m("M", "ctor", body=("m.cpp", 0, 1), lowered="{}")]
    m = ModuleModel()
    m.classes[mod.qualified] = mod
    m.globals = [GlobalVar("_AtlModule", "M", "m.cpp", None)]
    rs = cpp_app.render_classes(m)
    assert 'pub unsafe extern "C" fn __cxx_global_init() {' in rs
    assert "(*core::ptr::addr_of_mut!(_AtlModule)).__ctor_0();" in rs
    h = cpp_app.Hierarchy(m)
    # no synthetic C at all: a new preamble line is a unit the response cache never bought
    assert cpp_app._global_init(m, h) == ""


def test_a_class_holding_an_stl_container_is_not_copy():
    """drwtsn32's ModuleData holds a std::string: its destructor frees, so no Copy."""
    c = ClassModel("ModuleData", "ModuleData", "struct", "d.h")
    c.fields = [Field("ModuleName", "std::string"), Field("Size", "DWORD")]
    m = ModuleModel()
    m.classes[c.qualified] = c
    rs = cpp_app.render_classes(m)
    assert "pub ModuleName: StdString," in rs
    assert "#[derive(Clone, Copy)]\n#[repr(C)]\npub struct ModuleData" not in rs


def test_a_string_macro_beside_a_literal_is_concatenated_before_the_model_sees_it():
    """drwtsn32's `xfprintf(output, "%5d: %ls" NEWLINE, ...)` came back as
    `xfprintf(output, NEWLINE, ...)`: the literal was dropped. Fold it like phase 6 does."""
    from rustconv import cpp_lower

    macros = cpp_lower.string_macros(['#define NEWLINE "\\r\\n"\n#define K L"k"\n'])
    fold = cpp_lower.concat_string_macros
    assert fold('f(o, "%5d: %ls" NEWLINE, p);', macros) == 'f(o, "%5d: %ls\\r\\n", p);'
    assert fold('f(o, NEWLINE "a" NEWLINE NEWLINE);', macros) == 'f(o, "\\r\\na\\r\\n\\r\\n");'
    assert fold('g("x" K);', macros) == 'g(L"xk");'
    # standalone uses stay names: their bodies keep the response-cache key they were bought under
    assert fold("f(o, NEWLINE);", macros) == "f(o, NEWLINE);"
    # a hex escape would swallow the next literal's first chars: left as written
    assert fold('f("\\x4" NEWLINE);', macros) == 'f("\\x4" NEWLINE);'


def test_a_method_with_a_lifetime_parameter_is_still_moved_into_its_impl():
    """mmc_new's `CRecentFileEntry__FileName<'a>(this_: *mut ..) -> &'a CAtlString`."""
    c = ClassModel("CRecentFileEntry", "CRecentFileEntry", "class", "r.h")
    m = ModuleModel()
    m.classes[c.qualified] = c
    rust = (
        "pub unsafe fn CRecentFileEntry__FileName<'a>(this_: *mut CRecentFileEntry)"
        " -> &'a CAtlString {\n    &(*this_).m_FileName\n}\n"
    )
    out = cpp_app.assemble(rust, m)
    assert "impl CRecentFileEntry {" in out
    assert "pub unsafe fn FileName<'a>(&mut self) -> &'a CAtlString {" in out
    assert "CRecentFileEntry__FileName" not in out


def test_a_wide_string_global_is_a_static_of_its_units():
    from rustconv.cpp_app import wide_string_static
    from rustconv.cpp_model import GlobalVar

    g = GlobalVar("UsageString", "const WCHAR[6]", "x.h", 'L"a\\\\\\n"')
    assert wide_string_static(g) == "pub static UsageString: [u16; 6] = [97, 92, 10, 0, 0, 0];"
    g = GlobalVar("s", "WCHAR []", "x.h", 'L"hi"')
    assert wide_string_static(g) == "pub static mut s: [u16; 3] = [104, 105, 0];"
    # Not a lone wide literal (a macro the model must resolve): the model gets it.
    assert wide_string_static(GlobalVar("t", "const WCHAR[4]", "x.h", 'L"a" X')) is None
    assert wide_string_static(GlobalVar("u", "const WCHAR[1]", "x.h", 'L"ab"')) is None


def test_a_class_member_with_no_initializer_is_default_constructed():
    # rapps: CUiContainer has no constructor of its own; its `CUiCollection m_Children`
    # (a CPointerArray whose constructor calls DPA_Create) was left zeroed, and the first
    # GetCount read address 0. A plain member, a pointer member and a member whose class
    # has nothing to construct get no call.
    arr = ClassModel(
        "CPointerArray",
        "CPointerArray",
        "class",
        "r.h",
        polymorphic=True,
        methods=[_m("CPointerArray", "ctor", [("GrowthRate", "INT")])],
    )
    arr.methods[0].params[0].default = "10"
    coll = ClassModel(
        "CUiCollection", "CUiCollection", "class", "r.h", [Base("CPointerArray", "public")]
    )
    plain = ClassModel("CPlain", "CPlain", "struct", "r.h", fields=[Field("x", "INT")])
    cont = ClassModel(
        "CUiContainer",
        "CUiContainer",
        "class",
        "r.h",
        fields=[
            Field("m_Children", "CUiCollection"),
            Field("m_pOther", "CUiCollection *"),
            Field("m_Plain", "CPlain"),
            Field("n", "INT"),
        ],
    )
    model = ModuleModel(classes={c.name: c for c in (arr, coll, plain, cont)})
    h = cpp_app.Hierarchy(model)
    for c in model.classes.values():
        _assign_names(c)
    lines = cpp_app._ctor_prologue(cont, None, h, model)
    assert lines == ["    this_->m_Children.__ctor_0();"]


def test_a_qualified_base_call_is_non_virtual():
    # rapps: CUiWindow<CToolbar>::ComputeMinimalSize's `return CUiBox::ComputeMinimalSize(size);`
    # reached the virtual dispatcher -- its own override -- and recursed forever.
    box = ClassModel(
        "CUiBox",
        "CUiBox",
        "class",
        "r.h",
        polymorphic=True,
        methods=[
            _m("ComputeMinimalSize", params=[("size", "SIZE *")], virtual=True, body=("r.h", 0, 1)),
            _m("Plain", body=("r.h", 0, 1)),
        ],
    )
    win = ClassModel("CWin", "CWin", "class", "r.h", [Base("CUiBox", "public")], polymorphic=True)
    model = ModuleModel(classes={c.name: c for c in (box, win)})
    for c in model.classes.values():
        _assign_names(c)
    h = cpp_app.Hierarchy(model)
    body = (
        "{ return CUiBox::this_->__base0.ComputeMinimalSize(size); "
        "CUiBox::this_->__base0.Plain(); CListView::this_->Create(p); }"
    )
    out = cpp_app._qualified_base_calls(body, h)
    assert "this_->__base0.ComputeMinimalSize__body(size)" in out
    assert "this_->__base0.Plain()" in out and "CUiBox::" not in out
    assert "CListView::this_->Create(p)" in out  # no base path: left alone
