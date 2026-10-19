"""cpp_app over an ATL in-process server: the object map, CComModule, CLSIDs, CoClass glue."""

import types

import pytest

from rustconv import cpp_app, profile_derive
from rustconv.cpp_model import (
    Base,
    ClassModel,
    Field,
    GlobalVar,
    Method,
    ModuleModel,
    MsgMapEntry,
    Param,
    _assign_names,
    parse_atl_declares,
)

_ISLAND = """
#[repr(C)]
pub struct IUnknownVtbl {
    pub QueryInterface: ::core::option::Option<
        unsafe extern "system" fn(
            This: *mut IUnknown,
            riid: REFIID,
            ppvObject: *mut *mut ::core::ffi::c_void,
        ) -> HRESULT,
    >,
    pub AddRef: ::core::option::Option<unsafe extern "system" fn(This: *mut IUnknown) -> ULONG>,
    pub Release: ::core::option::Option<unsafe extern "system" fn(This: *mut IUnknown) -> ULONG>,
}
#[repr(C)]
pub struct IFooVtbl {
    pub QueryInterface: ::core::option::Option<
        unsafe extern "system" fn(
            This: *mut IFoo,
            riid: REFIID,
            ppvObject: *mut *mut ::core::ffi::c_void,
        ) -> HRESULT,
    >,
    pub AddRef: ::core::option::Option<unsafe extern "system" fn(This: *mut IFoo) -> ULONG>,
    pub Release: ::core::option::Option<unsafe extern "system" fn(This: *mut IFoo) -> ULONG>,
    pub Bar: ::core::option::Option<unsafe extern "system" fn(This: *mut IFoo) -> HRESULT>,
}
"""


def _model(declares=None, extra_bases=()):
    """mmcshext's shape: one COM class in a BEGIN_OBJECT_MAP, a CComModule, a DEFINE_GUID."""
    c = ClassModel("CX", "CX", "class", "x.cpp", polymorphic=True)
    c.bases = [
        Base("CComCoClass<CX, &CLSID_X>", "public"),
        Base("CComObjectRootEx<CComMultiThreadModelNoCS>", "public"),
        Base("IFoo", "public"),
        *extra_bases,
    ]
    c.fields = [Field("m_p", "PWSTR")]
    c.methods = [
        Method("Bar", "method", [], "HRESULT", virtual=True, body=("x.cpp", 0, 1), lowered="{}"),
        Method("FinalConstruct", "method", [], "HRESULT", body=("x.cpp", 0, 1), lowered="{}"),
    ]
    for name, params, ret in (
        ("QueryInterface", [Param("riid", "REFIID"), Param("ppvObject", "void **")], "HRESULT"),
        ("AddRef", [], "ULONG"),
        ("Release", [], "ULONG"),
    ):
        c.methods.append(Method(name, "method", params, ret))
    c.com_map = [MsgMapEntry("COM_INTERFACE_ENTRY_IID", ["IID_IFoo", "IFoo"])]
    c.atl_declares = (
        {"DECLARE_REGISTRY_RESOURCEID": "IDR_X", "DECLARE_NOT_AGGREGATABLE": "CX"}
        if declares is None
        else declares
    )
    _assign_names(c)
    m = ModuleModel()
    m.classes[c.qualified] = c
    m.defines = {"IDR_X": "101"}
    m.globals = [
        GlobalVar(
            "CLSID_X",
            "const GUID",
            "x.cpp",
            "DEFINE_GUID(CLSID_X, 0x7A80E4A8,0x8005,0x11D2,"
            "0xBC,0xF8,0x00,0xC0,0x4F,0x72,0xC7,0x17)",
        ),
        GlobalVar("g_Module", "CComModule", "x.cpp", None),
        GlobalVar(
            "ObjectMap",
            "ATL::_ATL_OBJMAP_ENTRY[2]",
            "x.cpp",
            "BEGIN_OBJECT_MAP(ObjectMap)\n    OBJECT_ENTRY(CLSID_X, CX)\nEND_OBJECT_MAP()",
            static=True,
        ),
    ]
    return m


def test_the_server_statics_are_rendered_not_translated():
    m = _model()
    rs = cpp_app.render_classes(m, frozenset(), _ISLAND)
    assert (
        "pub static CLSID_X: GUID = GUID { Data1: 0x7A80E4A8, Data2: 0x8005, Data3: 0x11D2, "
        "Data4: [0xBC, 0xF8, 0x00, 0xC0, 0x4F, 0x72, 0xC7, 0x17] };" in rs
    )
    assert (
        "pub static mut g_Module: atl_rust::dll::CComModule = atl_rust::dll::CComModule::new();"
        in rs
    )
    assert "pub static ObjectMap: [atl_rust::dll::ObjMapEntry; 2] = [" in rs
    assert (
        "atl_rust::dll::ObjMapEntry::new(core::ptr::addr_of!(CLSID_X) as *const c_void, "
        "CX__UpdateRegistry_thunk, atl_rust::dll::creator::<CX>)," in rs
    )
    assert "    atl_rust::dll::ObjMapEntry::END," in rs
    # ... so the unit the model translates no longer heads its file with them.
    head = cpp_app.synthetic_tus(m, _ISLAND)["x.cpp"].split("//")[0]
    assert "g_Module" not in head and "ObjectMap" not in head and "DEFINE_GUID" not in head


def test_an_object_map_class_gets_coclass_glue_and_its_registry_resource():
    rs = cpp_app.render_classes(_model(), frozenset(), _ISLAND)
    assert "impl atl_rust::dll::CoClass for CX {" in rs
    assert "unsafe fn destroy(p: *mut Self) { CX__com_destroy(p) }" in rs
    # IDR_X is a CMake -D define, resolved from the compile.
    assert "update_registry_from_resource((101) as u32, bRegister as i32" in rs
    assert 'unsafe extern "system" fn CX__UpdateRegistry_thunk(b: i32) -> i32' in rs
    # FinalConstruct's S_FALSE is a success: only a FAILED hr destroys the object.
    assert "if hr < 0 { CX__com_destroy(p); return hr; }" in rs


def test_no_registry_and_protect_final_construct():
    declares = {
        "DECLARE_NO_REGISTRY": "",
        "DECLARE_NOT_AGGREGATABLE": "CX",
        "DECLARE_PROTECT_FINAL_CONSTRUCT": "",
    }
    rs = cpp_app.render_classes(_model(declares), frozenset(), _ISLAND)
    assert "pub unsafe fn UpdateRegistry(_bRegister: BOOL) -> HRESULT { 0 }" in rs
    assert "atl_rust::com::internal_add_ref(core::ptr::addr_of_mut!((*p).__base1));" in rs


@pytest.mark.parametrize(
    ("declares", "match"),
    [
        ({"DECLARE_REGISTRY_RESOURCEID": "IDR_X"}, "aggregatable"),
        (
            {"DECLARE_NOT_AGGREGATABLE": "CX", "DECLARE_CLASSFACTORY_SINGLETON": "CX"},
            "DECLARE_CLASSFACTORY_SINGLETON is not ported",
        ),
        ({"DECLARE_NOT_AGGREGATABLE": "CX"}, "no UpdateRegistry"),
    ],
)
def test_what_atl_rust_dll_does_not_provide_is_refused(declares, match):
    with pytest.raises(cpp_app.CppAppError, match=match):
        cpp_app.render_classes(_model(declares), frozenset(), _ISLAND)


def test_object_entry_auto_is_refused():
    m = _model()
    m.globals.append(GlobalVar("__objMap_CX", "ATL::_ATL_OBJMAP_ENTRY", "x.cpp", "{ &CLSID_X }"))
    with pytest.raises(cpp_app.CppAppError, match="OBJECT_ENTRY_AUTO"):
        cpp_app.render_classes(m, frozenset(), _ISLAND)


def test_an_interface_base_the_island_lacks_fails_loud():
    """mmcshext's IPersistFile was embedded with no vtable: the first call jumped to 0."""
    m = _model(extra_bases=(Base("IPersistFile", "public"),))
    with pytest.raises(cpp_app.CppAppError, match="no IPersistFileVtbl"):
        cpp_app.render_classes(m, frozenset(), _ISLAND)


def test_parse_atl_declares_reads_only_the_class_level():
    text = (
        "class X : public A { public: X() { DECLARE_INNER(1); } "
        "DECLARE_REGISTRY_RESOURCEID(IDR_X) DECLARE_NOT_AGGREGATABLE(X) "
        "DECLARE_PROTECT_FINAL_CONSTRUCT() };"
    )
    assert parse_atl_declares(text) == {
        "DECLARE_REGISTRY_RESOURCEID": "IDR_X",
        "DECLARE_NOT_AGGREGATABLE": "X",
        "DECLARE_PROTECT_FINAL_CONSTRUCT": "",
    }


def test_an_external_class_nothing_names_is_pruned():
    m = _model()
    reg = ClassModel("CRegKey", "ATL::CRegKey", "class", "atlbase.h", external=True)
    reg.methods = [Method("CRegKey", "ctor", [], "", lowered="{}")]
    win = ClassModel("CWindow", "ATL::CWindow", "class", "atlwin.h", external=True)
    m.classes["ATL::CRegKey"] = reg
    m.classes["ATL::CWindow"] = win
    m.classes["CX"].fields.append(Field("m_w", "CWindow"))
    assert cpp_app.prune_unused_external(m) == ["CRegKey"]
    assert "ATL::CWindow" in m.classes and "ATL::CRegKey" not in m.classes


def test_a_window_template_base_keeps_cwindow():
    # acppage: `class CLayerUIPropPage : public CPropertyPageImpl<CLayerUIPropPage>, ..` never
    # spells CWindow, but the template IS it (cpp_app.short), and its layout is CWindow's.
    m = _model(extra_bases=(Base("CPropertyPageImpl<CX>", "public"),))
    win = ClassModel("CWindow", "ATL::CWindow", "class", "atlwin.h", external=True)
    m.classes["ATL::CWindow"] = win
    assert cpp_app.prune_unused_external(m) == []
    assert "ATL::CWindow" in m.classes


def test_the_island_allowlists_the_classes_interface_bases(tmp_path):
    tu = tmp_path / "x.cpp"
    tu.write_text(
        "class CX :\n    public CComCoClass<CX, &CLSID_X>,\n"
        "    public CComObjectRootEx<CComMultiThreadModelNoCS>,\n"
        "    public IExtractIconW, // icon\n    public IPersistFile\n{ };\n"
        "struct S : ATL::IDispatchImpl<IFoo> { };\n"
    )

    module = types.SimpleNamespace(compiled_tus=[tu])
    assert profile_derive.cpp_interface_bases(module) == ["IExtractIconW", "IPersistFile"]
    lines = profile_derive._allow_files_lines(["a"], ["IExtractIconW", "IPersistFile"])
    assert lines[-1].endswith("--allowlist-type '(IExtractIconW|IPersistFile)(Vtbl)?' )")


_IBASE = """
#[repr(C)]
pub struct IBaseVtbl {
    pub QueryInterface: ::core::option::Option<
        unsafe extern "system" fn(
            This: *mut IBase,
            riid: REFIID,
            ppvObject: *mut *mut ::core::ffi::c_void,
        ) -> HRESULT,
    >,
    pub AddRef: ::core::option::Option<unsafe extern "system" fn(This: *mut IBase) -> ULONG>,
    pub Release: ::core::option::Option<unsafe extern "system" fn(This: *mut IBase) -> ULONG>,
}
"""


def test_an_entry_for_an_interface_ancestor_lands_on_the_derived_base():
    # sendmail/mydocs: COM_INTERFACE_ENTRY_IID(IID_IPersist, IPersist) on a class that
    # derives only IPersistFile -- static_cast<IPersist*> is the IPersistFile subobject.
    m = _model()
    m.classes["CX"].com_map.append(MsgMapEntry("COM_INTERFACE_ENTRY_IID", ["IID_IBase", "IBase"]))
    rs = cpp_app.render_classes(m, frozenset(), _ISLAND + _IBASE)
    assert (
        "iid: core::ptr::addr_of!(IID_IBase) as *const c_void, offset: core::mem::offset_of!(CX, __base2)"
        in rs
    )


def test_an_ambiguous_interface_ancestor_is_refused():
    m = _model(extra_bases=(Base("IFoo2", "public"),))
    m.classes["CX"].com_map.append(MsgMapEntry("COM_INTERFACE_ENTRY_IID", ["IID_IBase", "IBase"]))
    ifoo2 = "#[repr(C)]\npub struct IFooVtbl" + _ISLAND.split("pub struct IFooVtbl")[1]
    island = _ISLAND + _IBASE + ifoo2.replace("IFoo", "IFoo2")
    with pytest.raises(cpp_app.CppAppError, match="IBase is an ambiguous base"):
        cpp_app.render_classes(m, frozenset(), island)


def test_a_brace_list_static_renders_against_the_substrate_struct():
    # ATL's `RECT CWindow::rcDefault = { CW_USEDEFAULT, CW_USEDEFAULT, 0, 0 };`. RECT is the
    # substrate's (`pub type RECT = tagRECT;` in ntdll-sys), not the crate island's.
    substrate = (
        "pub type RECT = tagRECT;\n#[repr(C)]\npub struct tagRECT {\n    pub left: LONG,\n"
        "    pub top: LONG,\n    pub right: LONG,\n    pub bottom: LONG,\n}\n"
    )
    fields = cpp_app.island_struct_fields(substrate)
    assert fields["RECT"] == ["left", "top", "right", "bottom"]
    f = Field("rcDefault", "RECT", static=True)
    f.default_init = "{ CW_USEDEFAULT, CW_USEDEFAULT, 0, 0 }"
    cls = ClassModel("CWindow", "ATL::CWindow", "class", "atlwin.h", external=True)
    out = cpp_app._static_member_init(cls, f, "RECT", ModuleModel(), fields)
    assert out == (
        "RECT { left: CW_USEDEFAULT as _, top: CW_USEDEFAULT as _, right: 0 as _, bottom: 0 as _ }"
    )


def test_a_brace_list_that_does_not_fit_the_struct_is_refused():
    fields = {"RECT": ["left", "top", "right", "bottom"]}
    f = Field("rc", "RECT", static=True)
    f.default_init = "{ 1, 2, 3 }"
    cls = ClassModel("CX", "CX", "class", "x.h")
    with pytest.raises(cpp_app.CppAppError, match="cannot render"):
        cpp_app._static_member_init(cls, f, "RECT", ModuleModel(), fields)
    f.default_init = "{ f(), 2, 3, 4 }"
    with pytest.raises(cpp_app.CppAppError, match="cannot render"):
        cpp_app._static_member_init(cls, f, "RECT", ModuleModel(), fields)


def test_an_empty_ccommodule_subclass_is_the_module():
    # acppage: `class CLayerUIPropPageModule : public CComModule {}; CLayerUIPropPageModule gModule;`
    m = _model()
    sub = ClassModel("CMyModule", "CMyModule", "class", "x.cpp", polymorphic=True)
    sub.bases = [Base("CComModule", "public")]
    m.classes["CMyModule"] = sub
    m.globals[1].c_type = "CMyModule"
    assert cpp_app.fold_module_subclasses(m) == ["CMyModule"]
    assert "CMyModule" not in m.classes and m.globals[1].c_type == "CComModule"
    rs = cpp_app.render_classes(m, frozenset(), _ISLAND)
    assert "pub static mut g_Module: atl_rust::dll::CComModule" in rs


def test_a_ccommodule_subclass_with_members_is_refused():
    m = _model()
    sub = ClassModel("CMyModule", "CMyModule", "class", "x.cpp")
    sub.bases = [Base("CComModule", "public")]
    sub.fields = [Field("m_extra", "int")]
    m.classes["CMyModule"] = sub
    with pytest.raises(cpp_app.CppAppError, match="CComModule subclass with fields"):
        cpp_app.fold_module_subclasses(m)


def test_a_ccommodule_subclass_method_stays_a_module_function_and_its_calls_follow():
    # fontext: CFontExtModule::Init creates the font cache, then calls CComModule::Init.
    from rustconv.cpp_model import FreeFunc

    m = _model()
    sub = ClassModel("CFontExtModule", "CFontExtModule", "class", "fontext.cpp")
    sub.bases = [Base("CComModule", "public")]
    init = Method("Init", "method", [Param("p", "_ATL_OBJMAP_ENTRY *")], "void", rust_name="Init")
    init.lowered = (
        "{\n    g_FontCache = CFontCache__new_0();\n    CComModule::this_->__base0.Init(p);\n}"
    )
    term = Method("Term", "method", [], "void", rust_name="Term")
    term.lowered = "{\n    CComModule::this_->__base0.Term();\n}"
    sub.methods = [init, term]
    m.classes["CFontExtModule"] = sub
    m.globals[1].c_type = "CFontExtModule"
    g = m.globals[1].name
    m.functions.append(
        FreeFunc(
            "DllMain",
            "BOOL WINAPI DllMain()",
            "fontext.cpp",
            f"{{ {g}.Init(ObjectMap); {g}.Term(); {g}.__base0.DllCanUnloadNow(); }}",
        )
    )
    assert cpp_app.fold_module_subclasses(m) == ["CFontExtModule"]
    fns = {f.name: f for f in m.functions}
    assert "this_->Init(p);" in fns["CFontExtModule_Init"].lowered
    assert fns["CFontExtModule_Init"].signature.startswith(
        "void CFontExtModule_Init(CComModule *this_, _ATL_OBJMAP_ENTRY * p)"
    )
    body = fns["DllMain"].lowered
    assert f"CFontExtModule_Init(&{g}, ObjectMap);" in body
    assert f"CFontExtModule_Term(&{g});" in body
    assert f"{g}.DllCanUnloadNow()" in body


def _with_virtual_dtor(m):
    c = m.classes["CX"]
    c.methods.append(
        Method("~CX", "dtor", [], "", virtual=True, body=("x.cpp", 0, 1), lowered="{ f(); }")
    )
    _assign_names(c)
    return m


def test_a_com_class_virtual_dtor_without_a_vtable_of_ours_deletes_statically():
    # cabview's CEnumIDList: every other virtual is an IEnumIDList method, so the interface's
    # lpVtbl is the only vptr, and nothing in the module derives from it.
    rs = cpp_app.render_classes(_with_virtual_dtor(_model()), frozenset(), _ISLAND)
    delete = next(ln for ln in rs.splitlines() if ln.startswith("pub unsafe fn CX__delete"))
    assert "drop_in_place(p)" in delete and "__vtbl" not in delete


def test_the_same_dtor_with_a_module_subclass_still_fails_loud():
    m = _with_virtual_dtor(_model())
    d = ClassModel("CY", "CY", "class", "x.cpp", polymorphic=True)
    d.bases = [Base("CX", "public")]
    _assign_names(d)
    m.classes[d.qualified] = d
    with pytest.raises(cpp_app.CppAppError, match="virtual destructor but no vtable"):
        cpp_app.render_classes(m, frozenset(), _ISLAND)


def test_pragma_pack_follows_pshpack_and_pragma_stacks():
    from rustconv.cpp_model import pragma_pack_at

    assert pragma_pack_at("struct A {};") is None
    assert pragma_pack_at("#include <pshpack1.h>\nstruct CABITEM") == 1
    assert pragma_pack_at("#include <pshpack1.h>\n#include <poppack.h>\nstruct B") is None
    assert pragma_pack_at("#pragma pack(push, 2)\n#include <pshpack4.h>\n#include <poppack.h>") == 2
    assert pragma_pack_at("#pragma pack(push, 8)\n#pragma pack(pop)\n") is None


def test_a_packed_struct_and_its_class_constants_render():
    # cabview's CABITEM: under pshpack1.h, with `enum { FSATTS = .. };` in its scope.
    c = ClassModel("CABITEM", "CABITEM", "struct", "x.cpp", pack=1, constants=[("FSATTS", 55)])
    c.fields = [Field("cb", "WORD"), Field("Size", "UINT")]
    m = ModuleModel()
    m.classes[c.qualified] = c
    rs = cpp_app.render_classes(m, frozenset(), _ISLAND)
    assert "#[repr(C, packed)]\npub struct CABITEM" in rs
    assert "pub const FSATTS: i32 = 55;" in rs


def _handler_table_model():
    # stobject's g_IconHandlers: `{ VOLUME_SERVICE_FLAG, Volume_Init, .., Volume_Message }`.
    from rustconv.cpp_model import FreeFunc

    c = ClassModel("SysTrayIconHandlers_t", "SysTrayIconHandlers_t", "struct", "precomp.h")
    c.fields = [
        Field("dwServiceFlag", "DWORD"),
        Field("pfnInit", "PFNSTINIT"),
        Field("pfnMessage", "PFNSTMESSAGE"),
    ]
    m = ModuleModel()
    m.classes[c.qualified] = c
    m.functions = [
        FreeFunc("Volume_Init", "HRESULT STDMETHODCALLTYPE Volume_Init(CSysTray * p)", "v.cpp", ""),
        FreeFunc(
            "Volume_Message",
            "HRESULT STDMETHODCALLTYPE Volume_Message(CSysTray * p, UINT u, WPARAM w, "
            "LPARAM l, LRESULT &r)",
            "v.cpp",
            "",
        ),
    ]
    g = GlobalVar(
        "g_IconHandlers",
        "SysTrayIconHandlers_t []",
        "c.cpp",
        "{ { 4, Volume_Init, Volume_Message } }",
    )
    return m, g


def test_a_function_in_a_handler_table_is_some_and_a_reference_param_transmutes():
    m, g = _handler_table_model()
    out = cpp_app.aggregate_static(g, cpp_app.Hierarchy(m), m) or ""
    assert "pfnInit: Some(Volume_Init)" in out
    assert (
        'pfnMessage: Some(unsafe { core::mem::transmute(Volume_Message as unsafe extern "system" '
        "fn(_, _, _, _, _) -> _) })" in out
    )


def test_a_module_cwintraits_typedef_gives_the_window_its_styles():
    from rustconv.cpp_model import _split_top

    assert [" ".join(a.split()) for a in _split_top("WS_POPUP | WS_DLGFRAME,\n WS_EX_LEFT")] == [
        "WS_POPUP | WS_DLGFRAME",
        "WS_EX_LEFT",
    ]
    m = ModuleModel(win_traits={"CMessageWndClass": ["WS_POPUP", "WS_EX_LEFT"]})
    assert ModuleModel.from_json(m.to_json()).win_traits == m.win_traits


def test_the_atl_contract_sees_a_global_cstring_the_body_does_not_spell():
    from rustconv import atl_contract

    m = ModuleModel()
    m.globals = [GlobalVar("g_strTooltip", "CString", "power.cpp", None)]
    body = "g_strTooltip.Format(IDS_PWR_CHARGING, p);"
    assert atl_contract.contract(body) == ""
    declared = cpp_app.declared_types(m, "Power_Update", body)
    assert "atl_format!" in atl_contract.contract(body, declared=declared)
