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
