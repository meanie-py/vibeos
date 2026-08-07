"""Deterministic C++/ATL COM-class pipeline."""

from pathlib import Path

import pytest

from rustconv.__main__ import _cost_cap_reached, _extend_cpp_class_scan, _trivial_cpp_body
from rustconv.call_seal import seal_calls, unseal_calls
from rustconv.callgraph import FunctionDep, extract_call_graph
from rustconv.config import REACTOS_ROOT, SHELL32_ISLAND_FILES, TARGETS, detect_target
from rustconv.cpp_com import (
    CppComError,
    InterfaceMethod,
    InterfaceSpec,
    _parse_plain_state_field,
    _state_field_name,
    analyze_class,
    apply_interface_method_signatures,
    audit_worker_body,
    c_type_to_rust,
    discover_widl_header,
    island_integer_aliases,
    island_struct_types,
    lower_trivial_worker,
    normalize_cpp_idioms,
    normalize_worker_body,
    normalize_worker_semantics,
    parse_widl_interface,
    pods_with_drop,
    prepare_helper_closure,
    render_class_scaffold,
    render_cpp_support,
    render_interface,
    render_object_map_adapter,
    render_wide_literal_statics,
    require_supported_worker_raii,
    rewrite_wide_literals,
    strip_debug_statements,
    worker_signature,
)
from rustconv.function_prompt import _format_callee_context, load_function_system_prompt
from rustconv.resolver import ResolveResult
from rustconv.scanner import scan_file
from rustconv.stitcher import _generate_imports_from_resolve

SHELL32 = REACTOS_ROOT / "dll" / "win32" / "shell32"

# widl-GENERATED psdk headers only exist after a configured build (build-<arch>/sdk/include/
# psdk); the reactos-private IDLs are committed and always resolvable. On a worktree with no
# i386 build tree the "no flattened vtable definition found" failure is a MISSING BUILD, not
# a code defect — so gate those cases explicitly instead of leaving `check-tooling` red for
# everyone who hasn't built i386 here yet.
GENERATED_PSDK_I386 = REACTOS_ROOT / "build-i386" / "sdk" / "include" / "psdk"
needs_generated_psdk_i386 = pytest.mark.skipif(
    not GENERATED_PSDK_I386.is_dir(),
    reason=f"needs widl-generated headers in {GENERATED_PSDK_I386} (build i386 first)",
)

WIDL_HEADER = r"""
typedef struct IDragSourceHelperVtbl {
    BEGIN_INTERFACE
    HRESULT (STDMETHODCALLTYPE *QueryInterface)(
        IDragSourceHelper *This, REFIID riid, void **ppvObject);
    ULONG (STDMETHODCALLTYPE *AddRef)(IDragSourceHelper *This);
    ULONG (STDMETHODCALLTYPE *Release)(IDragSourceHelper *This);
    HRESULT (STDMETHODCALLTYPE *InitializeFromBitmap)(
        IDragSourceHelper *This, LPSHDRAGIMAGE pshdi, IDataObject *pDataObject);
    HRESULT (STDMETHODCALLTYPE *InitializeFromWindow)(
        IDragSourceHelper *This, HWND hwnd, POINT *ppt, IDataObject *pDataObject);
    END_INTERFACE
} IDragSourceHelperVtbl;

typedef struct IDropTargetHelperVtbl {
    BEGIN_INTERFACE
    HRESULT (STDMETHODCALLTYPE *QueryInterface)(
        IDropTargetHelper *This, REFIID riid, void **ppvObject);
    ULONG (STDMETHODCALLTYPE *AddRef)(IDropTargetHelper *This);
    ULONG (STDMETHODCALLTYPE *Release)(IDropTargetHelper *This);
    HRESULT (STDMETHODCALLTYPE *DragEnter)(
        IDropTargetHelper *This, HWND hwndTarget, IDataObject *pDataObject,
        POINT *ppt, DWORD dwEffect);
    HRESULT (STDMETHODCALLTYPE *DragLeave)(IDropTargetHelper *This);
    HRESULT (STDMETHODCALLTYPE *DragOver)(
        IDropTargetHelper *This, POINT *ppt, DWORD dwEffect);
    HRESULT (STDMETHODCALLTYPE *Drop)(
        IDropTargetHelper *This, IDataObject *pDataObject, POINT *ppt, DWORD dwEffect);
    HRESULT (STDMETHODCALLTYPE *Show)(IDropTargetHelper *This, BOOL fShow);
    END_INTERFACE
} IDropTargetHelperVtbl;
"""


def _widl(tmp_path: Path) -> Path:
    path = tmp_path / "shobjidl.h"
    path.write_text(WIDL_HEADER)
    return path


def test_drop_target_helper_class_ir_is_exact():
    source = SHELL32 / "CDropTargetHelper.cpp"
    cls = analyze_class(
        source,
        "CDropTargetHelper",
        object_map_source=SHELL32 / "shell32.cpp",
    )
    assert cls.header == SHELL32 / "CDropTargetHelper.h"
    assert [(x.iid, x.interface) for x in cls.com_map] == [
        ("IID_IDragSourceHelper", "IDragSourceHelper"),
        ("IID_IDropTargetHelper", "IDropTargetHelper"),
    ]
    assert cls.registry_resource == "IDR_DRAGDROPHELPER"
    assert cls.not_aggregatable
    assert cls.object_map_clsid == "CLSID_DragDropHelper"
    assert cls.state_fields == []
    cls.require_supported()


def test_class_definition_discovery_ignores_forward_declarations():
    menu = analyze_class(
        SHELL32 / "shellmenu" / "CMenuBand.cpp",
        "CMenuBand",
        object_map_source=SHELL32 / "shell32.cpp",
    )
    assert menu.header == SHELL32 / "shellmenu" / "CMenuBand.h"

    in_source = analyze_class(
        SHELL32 / "CDefView.cpp",
        "CDefView",
        object_map_source=SHELL32 / "shell32.cpp",
    )
    assert in_source.header == SHELL32 / "CDefView.cpp"


def test_com_map_ignores_commented_out_interfaces():
    cls = analyze_class(
        SHELL32 / "CActiveDesktop.cpp",
        "CActiveDesktop",
        object_map_source=SHELL32 / "shell32.cpp",
    )
    assert [entry.interface for entry in cls.com_map] == ["IActiveDesktop", "IPropertyBag"]
    assert cls.base_classes == [
        "public CComCoClass<CActiveDesktop, &CLSID_ActiveDesktop>",
        "public CComObjectRootEx<CComMultiThreadModelNoCS>",
        "public IActiveDesktop",
        "public IPropertyBag",
    ]
    cls.require_supported()


def test_shell32_has_a_dedicated_body_only_target_and_prompt():
    target = detect_target(SHELL32 / "CDropTargetHelper.cpp")
    assert target is TARGETS["shell32"]
    assert target.output_crate.name == "shell32-rust"
    prompt = load_function_system_prompt(target, mode="mechanical")
    assert "method BODY" in prompt
    assert "deterministically" in prompt
    assert "ntoskrnl" not in prompt


def test_widl_vtable_is_flattened_ground_truth(tmp_path: Path):
    spec = parse_widl_interface(_widl(tmp_path), "IDragSourceHelper")
    assert [method.name for method in spec.methods] == [
        "QueryInterface",
        "AddRef",
        "Release",
        "InitializeFromBitmap",
        "InitializeFromWindow",
    ]
    assert spec.methods[0].params == (
        ("IDragSourceHelper *", "This"),
        ("REFIID", "riid"),
        ("void **", "ppvObject"),
    )


def test_widl_function_array_parameter_decays_to_pointer(tmp_path: Path):
    header = tmp_path / "array-param.h"
    header.write_text(
        """typedef struct IArrayVtbl {
        HRESULT (STDMETHODCALLTYPE *Query)(
            IArray *This, ULONG count, OLECMD commands[]);
        } IArrayVtbl;"""
    )
    spec = parse_widl_interface(header, "IArray")
    assert spec.methods[0].params[-1] == ("OLECMD *", "commands")
    assert "commands: *mut OLECMD" in render_interface(spec)


@pytest.mark.parametrize("declarator", ["commands[]", "commands []"])
def test_cpp_scanner_function_array_parameter_decays_to_pointer(tmp_path: Path, declarator: str):
    source = tmp_path / "array-param.cpp"
    source.write_text(f"HRESULT C::Query(ULONG count, OLECMD {declarator}) {{ return S_OK; }}")
    method = scan_file(source).functions[0]
    assert method.params == [("ULONG", "count"), ("OLECMD *", "commands")]


def test_widl_header_discovery_is_per_interface(tmp_path: Path):
    psdk = tmp_path / "build-i386" / "sdk" / "include" / "psdk"
    psdk.mkdir(parents=True)
    (psdk / "first.h").write_text("typedef struct IFirstVtbl { int slot; } IFirstVtbl;")
    (psdk / "second.h").write_text("typedef struct ISecondVtbl { int slot; } ISecondVtbl;")
    assert discover_widl_header(tmp_path, "i386", interface="IFirst").name == "first.h"
    assert discover_widl_header(tmp_path, "i386", interface="ISecond").name == "second.h"


def test_legacy_declare_interface_is_flattened_deterministically(tmp_path: Path):
    header = tmp_path / "legacy.h"
    header.write_text(
        """DECLARE_INTERFACE_(ILegacy, IUnknown)
{
    STDMETHOD(QueryInterface) (THIS_ _In_ REFIID riid, _Outptr_ void **ppv) PURE;
    STDMETHOD_(ULONG,AddRef) (THIS) PURE;
    STDMETHOD_(ULONG,Release) (THIS) PURE;
    STDMETHOD(DoThing)(THIS_ DWORD flags, _Out_ int *value) PURE;
};"""
    )
    spec = parse_widl_interface(header, "ILegacy")
    assert [method.name for method in spec.methods] == [
        "QueryInterface",
        "AddRef",
        "Release",
        "DoThing",
    ]
    assert spec.methods[3].params == (
        ("ILegacy *", "This"),
        ("DWORD", "flags"),
        ("int *", "value"),
    )


def test_interface_renderer_owns_system_abi(tmp_path: Path):
    rust = render_interface(parse_widl_interface(_widl(tmp_path), "IDropTargetHelper"))
    assert 'pub DragEnter: unsafe extern "system" fn(' in rust
    assert "This: *mut IDropTargetHelper" in rust
    assert "ppvObject: *mut *mut core::ffi::c_void" in rust
    assert 'extern "C" {\n    pub static IID_IDropTargetHelper: IID;\n}' in rust


def test_class_scaffold_owns_offsets_and_vtable_order(tmp_path: Path):
    cls = analyze_class(
        SHELL32 / "CDropTargetHelper.cpp",
        "CDropTargetHelper",
        object_map_source=SHELL32 / "shell32.cpp",
    )
    interfaces = [parse_widl_interface(_widl(tmp_path), entry.interface) for entry in cls.com_map]
    rust = render_class_scaffold(cls, interfaces)
    assert "pub IDragSourceHelper_iface: IDragSourceHelper" in rust
    assert "pub IDropTargetHelper_iface: IDropTargetHelper" in rust
    assert "offset_of!(CDropTargetHelper, IDropTargetHelper_iface)" in rust
    assert "DragEnter: CDropTargetHelper_IDropTargetHelper_DragEnter" in rust
    assert 'extern "system" fn CDropTargetHelper_IDropTargetHelper_DragEnter' in rust
    assert "CDropTargetHelper_DragEnter_body(this" in rust
    assert "CLSID_DragDropHelper currently maps to CDropTargetHelper" in rust
    assert "cpp_com_factory_unimplemented" not in rust
    assert "CDropTargetHelper_QueryInterface(this, riid, ppv)" in rust
    assert "Shell32RustObjectLock();" in rust
    assert "Shell32RustObjectUnlock();" in rust
    assert "HEAP_ZERO_MEMORY" in rust


@needs_generated_psdk_i386
def test_class_scaffold_uses_one_slot_for_com_map_iid_aliases(tmp_path: Path):
    menu = analyze_class(
        SHELL32 / "shellmenu" / "CMenuBand.cpp",
        "CMenuBand",
        object_map_source=SHELL32 / "shell32.cpp",
    )
    desk = parse_widl_interface(
        discover_widl_header(REACTOS_ROOT, "i386", interface="IDeskBand"),
        "IDeskBand",
    )
    # Exercise the alias entries in isolation; unrelated CMenuBand interfaces
    # need not be rendered for this layout invariant.
    menu.com_map = [entry for entry in menu.com_map if entry.interface == "IDeskBand"]
    rust = render_class_scaffold(menu, [desk])
    assert rust.count("pub IDeskBand_iface: IDeskBand,") == 1
    assert rust.count("pub static CMENUBAND_IDESKBAND_VTBL") == 1
    assert "IID_IOleWindow" in rust
    assert "IID_IDeskBand" in rust


@needs_generated_psdk_i386
def test_class_scaffold_uses_derived_slot_for_mapped_base_interface():
    menu = analyze_class(
        SHELL32 / "COpenWithMenu.cpp",
        "COpenWithMenu",
        object_map_source=SHELL32 / "shell32.cpp",
    )
    interfaces = [
        parse_widl_interface(
            discover_widl_header(REACTOS_ROOT, "i386", interface=entry.interface),
            entry.interface,
        )
        for entry in menu.com_map
    ]
    context_menu_2 = next(item for item in interfaces if item.name == "IContextMenu2")
    assert context_menu_2.base == "IContextMenu"

    rust = render_class_scaffold(menu, interfaces)
    assert "pub IContextMenu2_iface: IContextMenu2," in rust
    assert "pub IContextMenu_iface: IContextMenu," not in rust
    assert "pub static COPENWITHMENU_ICONTEXTMENU_VTBL" not in rust
    assert rust.count("(*this).IContextMenu2_iface") >= 2


def test_object_map_adapter_redirects_atl_creation():
    cls = analyze_class(
        SHELL32 / "CDropTargetHelper.cpp",
        "CDropTargetHelper",
        object_map_source=SHELL32 / "shell32.cpp",
    )
    cpp = render_object_map_adapter(cls)
    assert "class CDropTargetHelperRustEntry : public CDropTargetHelper" in cpp
    assert "return CDropTargetHelper_CreateInstance(" in cpp
    assert "OBJECT_ENTRY(CLSID_DragDropHelper, CDropTargetHelperRustEntry)" in cpp
    assert "#ifndef SHELL32_RUST_OBJECT_LOCK_BRIDGE_DEFINED" in cpp
    assert "Shell32RustObjectLock() { gModule.Lock(); }" in cpp


def test_internal_shell_object_creator_proves_nonaggregated_private_activation():
    source = SHELL32 / "CEnumIDListBase.cpp"
    cls = analyze_class(
        source,
        "CEnumIDListBase",
        object_map_source=SHELL32 / "shell32.cpp",
    )
    assert cls.internal_shell_object_creator
    assert cls.object_map_clsid is None
    cls.require_supported(allow_internal=True)
    with pytest.raises(CppComError, match="no OBJECT_ENTRY activation"):
        cls.require_supported()


def test_internal_scaffold_has_private_rust_factory_and_no_export_adapter():
    source = SHELL32 / "CEnumIDListBase.cpp"
    cls = analyze_class(source, "CEnumIDListBase", object_map_source=SHELL32 / "shell32.cpp")
    interface = parse_widl_interface(
        discover_widl_header(REACTOS_ROOT, "amd64", interface="IEnumIDList"),
        "IEnumIDList",
    )
    rust = render_class_scaffold(cls, [interface], allow_internal=True)
    assert "Internal activation: shellutils.h ShellObjectCreator<T>" in rust
    assert "pub unsafe fn CEnumIDListBase_CreateInstance(" in rust
    assert '#[no_mangle] pub unsafe extern "system" fn CEnumIDListBase_CreateInstance' not in rust


def test_shell_item_array_internal_init_factory_sequence_and_stgmedium_lifecycle():
    source = SHELL32 / "CShellItem.cpp"
    cls = analyze_class(source, "CShellItemArray", object_map_source=SHELL32 / "shell32.cpp")
    assert cls.internal_shell_object_creator
    assert cls.internal_initializer_params == (("IDataObject *", "pdo"),)
    assert cls.stgmedium_lifecycle_fields == ("m_Medium",)
    assert {field.name: field.rust_type for field in cls.plain_state_fields()} == {
        "m_pCIDA": "*mut CIDA",
        "m_Medium": "STGMEDIUM",
    }
    cls.require_supported(allow_internal=True)
    interface = parse_widl_interface(
        discover_widl_header(REACTOS_ROOT, "amd64", interface="IShellItemArray"),
        "IShellItemArray",
    )
    rust = render_class_scaffold(cls, [interface], allow_internal=True)
    assert "pub unsafe fn CShellItemArray_CreateInstance(" in rust
    assert "pdo: *mut IDataObject" in rust
    assert "outer: *mut IUnknown" not in rust
    initialize = rust.index("CShellItemArray_Initialize_body(this, pdo)")
    query = rust.index("CShellItemArray_QueryInterface(this, riid, ppv)", initialize)
    release = rust.index("CShellItemArray_Release(this)", query)
    assert initialize < query < release
    assert "if hr >= 0" in rust[initialize:query]


def test_internal_init_accepts_repeated_call_sites_and_winapi_initializer():
    source = SHELL32 / "folders" / "CPrinterFolder.cpp"
    cls = analyze_class(source, "CPrintersEnum", object_map_source=SHELL32 / "shell32.cpp")
    assert cls.internal_shell_object_creator
    assert cls.internal_initializer_params == (("HWND", "hwndOwner"), ("DWORD", "dwFlags"))
    assert not cls.unsupported
    cls.require_supported(allow_internal=True)
    scan = _extend_cpp_class_scan(
        scan_file(source), source, "CPrintersEnum", tuple(cls.local_base_classes)
    )
    inherited = {"Next", "Skip", "Reset", "Clone"}
    assert inherited <= {item.name for item in scan.functions}
    assert all(
        item.cpp_class == "CPrintersEnum" for item in scan.functions if item.name in inherited
    )


def test_stgmedium_state_without_exact_lifecycle_fails_loud(tmp_path: Path):
    source = tmp_path / "CBadMedium.cpp"
    source.write_text(
        """
class CBadMedium : public CComCoClass<CBadMedium>,
    public CComObjectRootEx<CComMultiThreadModelNoCS>, public IFoo {
    STGMEDIUM m_medium;
public:
    CBadMedium() { m_medium.tymed = TYMED_NULL; }
    ~CBadMedium() { }
    DECLARE_NOT_AGGREGATABLE(CBadMedium)
    BEGIN_COM_MAP(CBadMedium)
        COM_INTERFACE_ENTRY_IID(IID_IFoo, IFoo)
    END_COM_MAP()
};
"""
    )
    object_map = tmp_path / "shell32.cpp"
    object_map.write_text("OBJECT_ENTRY(CLSID_BadMedium, CBadMedium)\n")
    cls = analyze_class(source, "CBadMedium", object_map_source=object_map)
    with pytest.raises(CppComError, match="lacks source-proven constructor/destructor"):
        cls.require_supported()


@pytest.mark.parametrize(
    ("c_type", "rust_type"),
    [
        ("void **", "*mut *mut core::ffi::c_void"),
        ("const IID *", "*const IID"),
        ("QCMINFO_IDMAP const *", "*const QCMINFO_IDMAP"),
        ("IDataObject *", "*mut IDataObject"),
        ("CStringW &", "&mut CStringW"),
        ("const CStringW &", "&CStringW"),
        ("struct tagNMHDR *", "*mut tagNMHDR"),
        ("const struct DEFFOLDERSETTINGS *", "*const DEFFOLDERSETTINGS"),
        ("DWORD", "DWORD"),
    ],
)
def test_c_declarator_mapping(c_type: str, rust_type: str):
    assert c_type_to_rust(c_type) == rust_type


def test_worker_signature_moves_reference_punctuation_from_scanner_name():
    signature = worker_signature("AppendToPathList", None, [("CStringW", "&paths")], "void")
    assert signature == "pub unsafe fn AppendToPathList_body(paths: &mut CStringW) -> ()"


def test_plain_c_state_has_deterministic_layout_and_zero_initialization():
    cls = analyze_class(
        SHELL32 / "droptargets" / "CexeDropHandler.cpp",
        "CExeDropHandler",
        object_map_source=SHELL32 / "shell32.cpp",
    )
    assert cls.state_fields == ["CLSID *pclsid", "LPWSTR sPathTarget"]
    cls.require_supported()
    fields = cls.plain_state_fields()
    assert [(field.c_type, field.name) for field in fields] == [
        ("CLSID *", "pclsid"),
        ("LPWSTR", "sPathTarget"),
    ]
    interfaces = [
        InterfaceSpec(
            entry.interface,
            (
                InterfaceMethod("QueryInterface", "HRESULT", ((f"{entry.interface} *", "This"),)),
                InterfaceMethod("AddRef", "ULONG", ((f"{entry.interface} *", "This"),)),
                InterfaceMethod("Release", "ULONG", ((f"{entry.interface} *", "This"),)),
            ),
            "fixture.h",
        )
        for entry in cls.com_map
    ]
    rust = render_class_scaffold(cls, interfaces)
    assert "pub pclsid: *mut CLSID," in rust
    assert "pub sPathTarget: LPWSTR," in rust
    assert "pclsid: core::mem::zeroed()," in rust
    assert "sPathTarget: core::mem::zeroed()," in rust


def test_ccomptr_state_has_deterministic_layout_assignment_and_release(tmp_path: Path):
    source = tmp_path / "CState.cpp"
    source.write_text(
        """
class CState :
    public CComCoClass<CState>,
    public CComObjectRootEx<CComMultiThreadModelNoCS>,
    public IFoo
{
    CComPtr<IUnknown> m_site;
    DECLARE_NOT_AGGREGATABLE(CState)
    BEGIN_COM_MAP(CState)
        COM_INTERFACE_ENTRY_IID(IID_IFoo, IFoo)
    END_COM_MAP()
};
"""
    )
    object_map = tmp_path / "shell32.cpp"
    object_map.write_text("OBJECT_ENTRY(CLSID_State, CState)\n")
    cls = analyze_class(source, "CState", object_map_source=object_map)
    cls.require_supported()
    field = cls.plain_state_fields()[0]
    assert field.rust_type == "*mut IUnknown"
    assert field.assignment_helper == "cpp_com_assign_m_site"
    interfaces = [
        InterfaceSpec(
            "IFoo",
            (
                InterfaceMethod("QueryInterface", "HRESULT", (("IFoo *", "This"),)),
                InterfaceMethod("AddRef", "ULONG", (("IFoo *", "This"),)),
                InterfaceMethod("Release", "ULONG", (("IFoo *", "This"),)),
            ),
            "fixture.h",
        )
    ]
    rust = render_class_scaffold(cls, interfaces)
    assert "pub m_site: *mut IUnknown" in rust
    assert "let unknown = value as *mut IUnknown" in rust
    assert "core::mem::transmute((*(*unknown).lpVtbl).AddRef)" in rust
    assert "add_ref(unknown)" in rust
    assert "let unknown = old as *mut IUnknown" in rust
    assert "core::mem::transmute((*(*unknown).lpVtbl).Release)" in rust
    assert "release(unknown)" in rust
    assert "CState_cpp_com_assign_m_site(this, core::ptr::null_mut())" in rust


def test_cstringw_object_state_has_deterministic_init_assignment_and_drop(tmp_path: Path):
    source = tmp_path / "CState.cpp"
    source.write_text(
        """
class CState :
    public CComCoClass<CState>,
    public CComObjectRootEx<CComMultiThreadModelNoCS>,
    public IFoo
{
    CStringW m_text;
    DECLARE_NOT_AGGREGATABLE(CState)
    BEGIN_COM_MAP(CState)
        COM_INTERFACE_ENTRY_IID(IID_IFoo, IFoo)
    END_COM_MAP()
};
"""
    )
    object_map = tmp_path / "shell32.cpp"
    object_map.write_text("OBJECT_ENTRY(CLSID_State, CState)\n")
    cls = analyze_class(source, "CState", object_map_source=object_map)
    cls.require_supported()
    interfaces = [
        InterfaceSpec(
            "IFoo",
            (
                InterfaceMethod("QueryInterface", "HRESULT", (("IFoo *", "This"),)),
                InterfaceMethod("AddRef", "ULONG", (("IFoo *", "This"),)),
                InterfaceMethod("Release", "ULONG", (("IFoo *", "This"),)),
            ),
            "fixture.h",
        )
    ]
    rust = render_class_scaffold(cls, interfaces)
    assert "pub m_text: CStringW" in rust
    assert "CState_cpp_com_assign_m_text" in rust
    assert "(*this).m_text.assign(value)" in rust
    assert "m_text: CStringW::new()" in rust
    assert "core::ptr::drop_in_place(&mut (*this).m_text)" in rust


def test_unknown_by_value_state_is_not_mislabeled_plain_c(tmp_path: Path):
    source = tmp_path / "CState.cpp"
    source.write_text(
        """
class CState :
    public CComCoClass<CState>,
    public CComObjectRootEx<CComMultiThreadModelNoCS>,
    public IFoo
{
    Widget m_widget;
    DECLARE_NOT_AGGREGATABLE(CState)
    BEGIN_COM_MAP(CState)
        COM_INTERFACE_ENTRY_IID(IID_IFoo, IFoo)
    END_COM_MAP()
};
"""
    )
    object_map = tmp_path / "shell32.cpp"
    object_map.write_text("OBJECT_ENTRY(CLSID_State, CState)\n")
    cls = analyze_class(source, "CState", object_map_source=object_map)
    with pytest.raises(CppComError, match="not proven C scalar/pointer state"):
        cls.require_supported()


def test_inline_static_method_and_trailing_comment_are_not_object_state():
    cls = analyze_class(
        SHELL32 / "folders" / "CControlPanelFolder.cpp",
        "COpenControlPanel",
        object_map_source=SHELL32 / "shell32.cpp",
    )
    assert cls.state_fields == []
    cls.require_supported()


def test_shell_link_monster_preflight_owns_packed_pods_and_fixed_arrays():
    source = SHELL32 / "CShellLink.cpp"
    cls = analyze_class(source, "CShellLink", object_map_source=SHELL32 / "shell32.cpp")
    cls.require_supported()
    assert len(cls.com_map) == 13
    assert cls.object_map_clsid == "CLSID_ShellLink"
    pods = {item.name: item for item in cls.pod_state_types}
    assert set(pods) == {"SHELL_LINK_HEADER", "volume_info"}
    assert all(item.packed for item in pods.values())
    volume_fields = {field.name: field.rust_type for field in pods["volume_info"].fields}
    assert volume_fields == {"type": "DWORD", "serial": "DWORD", "label": "[WCHAR; 12]"}
    state = {field.name: field.rust_type for field in cls.plain_state_fields()}
    assert state["m_Header"] == "SHELL_LINK_HEADER"
    assert state["volume"] == "volume_info"
    assert state["m_site"] == "*mut IUnknown"
    assert state["m_DropTarget"] == "*mut IDropTarget"


def test_helper_closure_binds_overloads_and_excludes_unrelated_functions():
    scan = scan_file(SHELL32 / "folders" / "CControlPanelFolder.cpp")
    plan = prepare_helper_closure(extract_call_graph(scan), "COpenControlPanel")
    by_name = {dep.name: dep for dep in plan}
    assert set(by_name) == {
        "GetParsingName",
        "CreateCplAbsoluteParsingPath",
        "FindExeCplClass",
        "FindExeCplClass_2",
        "COpenControlPanel_Open",
        "COpenControlPanel_GetPath",
        "COpenControlPanel_GetCurrentView",
        "COpenControlPanel_ctor",
        "COpenControlPanel_dtor",
    }
    assert "FindExeCplClass_2_body(pszName, clsid)" in by_name["COpenControlPanel_Open"].body_text
    assert "FindExeCplClass_2_body" in by_name["COpenControlPanel_Open"].callees
    assert (
        "FindExeCplClass_body(Canonical, HKEY_CURRENT_USER, FALSE, clsid)"
        in by_name["FindExeCplClass_2"].body_text
    )
    assert "CControlPanelFolder" not in " ".join(by_name)
    assert 'L"CLSID\\\\%s"' not in by_name["FindExeCplClass"].body_text
    assert "CPP_COM_WSTR_" in by_name["FindExeCplClass"].body_text
    statics = render_wide_literal_statics(extract_call_graph(scan), plan)
    assert "pub static CPP_COM_WSTR_" in statics
    assert "[WCHAR;" in statics


def test_helper_closure_resolves_a_free_helper_against_a_method_of_the_same_name():
    """CDefaultContextMenu has BOTH a static `MapVerbToCmdId/5` and a method
    `MapVerbToCmdId/3`, and C++ tells them apart by scope: the method body calls the
    member unqualified, and reaches the free one as `::MapVerbToCmdId`.

    The reachability walk only knew the free helper, so the 3-argument member call read
    as an unbindable arity and hard-failed the whole 2,024-line TU before payment — even
    though the rewrite pass right below it already skipped these to let the method pass
    take them. The asymmetry was the bug; both spellings must survive.
    """
    source = SHELL32 / "CDefaultContextMenu.cpp"
    cls = analyze_class(source, "CDefaultContextMenu")
    plan = prepare_helper_closure(
        extract_call_graph(scan_file(source)),
        "CDefaultContextMenu",
        cls.plain_state_fields(),
    )
    by_name = {dep.name: dep for dep in plan}
    assert "MapVerbToCmdId" in by_name, "the free static helper"
    assert "CDefaultContextMenu_MapVerbToCmdId" in by_name, "the member of the same name"
    assert len(by_name["MapVerbToCmdId"].params) == 5
    assert len(by_name["CDefaultContextMenu_MapVerbToCmdId"].params) == 3
    # The `::`-qualified call keeps the free helper; the unqualified one takes the member.
    dynamic = by_name["CDefaultContextMenu_GetDynamicEntry"]
    invoke = by_name["CDefaultContextMenu_InvokeCommand"]
    assert "MapVerbToCmdId_body" in " ".join(dep.body_text for dep in (dynamic, invoke)), (
        "at least one call site is bound"
    )
    assert "CDefaultContextMenu_MapVerbToCmdId_body" in invoke.body_text


def test_helper_closure_makes_ccomptr_assignment_and_access_explicit():
    source = SHELL32 / "CCopyAsPathMenu.cpp"
    cls = analyze_class(source, "CCopyAsPathMenu", object_map_source=SHELL32 / "shell32.cpp")
    cls.require_supported()
    plan = prepare_helper_closure(
        extract_call_graph(scan_file(source)), "CCopyAsPathMenu", cls.plain_state_fields()
    )
    by_name = {dep.name: dep for dep in plan}
    initialize = by_name["CCopyAsPathMenu_Initialize"]
    assert "CCopyAsPathMenu_cpp_com_assign_m_pDataObject(this, pdtobj);" in initialize.body_text
    assert "CCopyAsPathMenu_cpp_com_assign_m_pDataObject" in initialize.callees
    assert (
        "CCopyAsPathMenu_DoCopyAsPath_body(this->m_pDataObject)"
        in by_name["CCopyAsPathMenu_InvokeCommand"].body_text
    )
    assert "CCopyAsPathMenu_DoCopyAsPath_body" in by_name["CCopyAsPathMenu_InvokeCommand"].callees
    assert "DoCopyAsPath" not in by_name["CCopyAsPathMenu_InvokeCommand"].callees


def test_rewritten_local_worker_has_deterministic_rust_callee_context():
    caller = FunctionDep(
        name="C_Call",
        cpp_class="C",
        cpp_method="Call",
        callees={"Helper_body"},
    )
    helper = FunctionDep(
        name="Helper",
        params=[("DWORD", "value")],
        return_type="HRESULT",
    )
    context = _format_callee_context(caller, {}, [caller, helper], set())
    assert "generated local Rust worker" in context
    assert "pub unsafe fn Helper_body(value: DWORD) -> HRESULT;" in context


def test_wide_character_literals_are_numeric_before_prompting():
    roots = [
        FunctionDep(
            name="C_f",
            cpp_class="C",
            cpp_method="f",
            body_text="HRESULT C::f() { return Find(L' '); }",
        )
    ]
    plan = prepare_helper_closure(roots, "C")
    assert "Find(0x0020u16)" in next(dep for dep in plan if dep.name == "C_f").body_text


def test_narrow_char_literal_ending_in_L_does_not_open_a_wide_literal():
    """`case 'L':` must stay narrow — the regex this replaced ran to the next `'`.

    shlexec.cpp has `case 'L':` at line 400 and a comment reading "PIDL's" at
    line 2482; `L'(?:\\.|[^'\\])+'` matched across all 1,800 lines between them
    and the whole TU failed to translate.
    """
    assert rewrite_wide_literals("case 'L':") == "case 'L':"
    assert rewrite_wide_literals("case 'L': if (p == L':') x;") == (
        "case 'L': if (p == 0x003Au16) x;"
    )


def test_wide_literals_inside_comments_and_narrow_strings_are_left_alone():
    assert rewrite_wide_literals("/* PIDL's and 'My Computer' */") == (
        "/* PIDL's and 'My Computer' */"
    )
    assert rewrite_wide_literals('// a L"x" in a comment\n') == '// a L"x" in a comment\n'
    assert rewrite_wide_literals('s = "has L\' inside";') == 's = "has L\' inside";'


def test_escaped_quotes_inside_wide_char_literals():
    assert rewrite_wide_literals(r"c == L'\''") == "c == 0x0027u16"
    assert rewrite_wide_literals(r"c == L'\\'") == "c == 0x005Cu16"


def test_helper_closure_keeps_same_file_external_functions_at_abi_boundary():
    scan = scan_file(SHELL32 / "CShellItem.cpp")
    plan = prepare_helper_closure(extract_call_graph(scan), "CShellItem")
    assert "CShellItem_GetParent" in {dep.name for dep in plan}
    assert "SHCreateShellItem" not in {dep.name for dep in plan}


def test_helper_worker_signature_has_no_invented_this_parameter():
    assert worker_signature(
        "GetParsingName", None, [("PCIDLIST_ABSOLUTE", "pidl"), ("PWSTR *", "Name")], "HRESULT"
    ) == ("pub unsafe fn GetParsingName_body(pidl: PCIDLIST_ABSOLUTE, Name: *mut PWSTR) -> HRESULT")


@needs_generated_psdk_i386
def test_interface_vtable_confirms_scanner_recovered_cpp_return_types():
    scan = scan_file(SHELL32 / "folders" / "CControlPanelFolder.cpp")
    deps = extract_call_graph(scan)
    assert next(d for d in deps if d.name == "COpenControlPanel_Open").return_type == "HRESULT"
    header = discover_widl_header(REACTOS_ROOT, "i386", interface="IOpenControlPanel")
    interface = parse_widl_interface(header, "IOpenControlPanel")
    repaired = apply_interface_method_signatures(deps, "COpenControlPanel", [interface])
    method = next(d for d in repaired if d.name == "COpenControlPanel_Open")
    assert method.return_type == "HRESULT"
    assert method.params == [
        ("LPCWSTR", "pszName"),
        ("LPCWSTR", "pszPage"),
        ("IUnknown *", "punkSite"),
    ]


def test_legacy_reactos_declared_interface_is_compiler_authority():
    header = discover_widl_header(REACTOS_ROOT, "amd64", interface="IShellMenu2")
    interface = parse_widl_interface(header, "IShellMenu2")
    assert header.name == "shlobj_undoc.h"
    assert [method.name for method in interface.methods[-5:]] == [
        "GetSubMenu",
        "SetToolbar",
        "SetMinWidth",
        "SetNoBorder",
        "SetTheme",
    ]


def test_scanner_treats_cpp_this_macro_as_void_parameter_list():
    scan = scan_file(SHELL32 / "shellmenu" / "CMenuBand.cpp")
    method = next(
        function
        for function in scan.functions
        if function.cpp_class == "CMenuBand" and function.name == "GetSubMenu"
    )
    assert method.params == []


def test_inherited_interface_method_ignores_receiver_spelling_conflict():
    dep = FunctionDep(
        name="C_GetClassID",
        cpp_class="C",
        cpp_method="GetClassID",
        params=[("CLSID *", "pClassID")],
    )
    parent = InterfaceSpec(
        "IPersist",
        (
            InterfaceMethod(
                "GetClassID", "HRESULT", (("IPersist *", "This"), ("CLSID *", "pClassID"))
            ),
        ),
        "fixture.h",
    )
    child = InterfaceSpec(
        "IPersistFile",
        (
            InterfaceMethod(
                "GetClassID",
                "HRESULT",
                (("IPersistFile *", "This"), ("CLSID *", "pClassID")),
            ),
        ),
        "fixture.h",
    )
    repaired = apply_interface_method_signatures([dep], "C", [parent, child])
    assert repaired[0].return_type == "HRESULT"
    assert repaired[0].params == [("CLSID *", "pClassID")]


def test_interface_binding_treats_lpvoid_pointer_as_void_double_pointer():
    dep = FunctionDep(
        name="C_BindToObject",
        cpp_class="C",
        cpp_method="BindToObject",
        params=[
            ("PCUIDLIST_RELATIVE", "pidl"),
            ("LPBC", "pbc"),
            ("REFIID", "riid"),
            ("LPVOID *", "ppvOut"),
        ],
    )
    interface = InterfaceSpec(
        "IShellFolder",
        (
            InterfaceMethod(
                "BindToObject",
                "HRESULT",
                (
                    ("IShellFolder *", "This"),
                    ("PCUIDLIST_RELATIVE", "pidl"),
                    ("LPBC", "pbc"),
                    ("REFIID", "riid"),
                    ("void **", "ppvOut"),
                ),
            ),
        ),
        "fixture.h",
    )
    repaired = apply_interface_method_signatures([dep], "C", [interface])
    assert repaired[0].params[-1] == ("void **", "ppvOut")


def test_same_arity_helper_overload_binds_from_declared_argument_type():
    roots = [
        FunctionDep(
            name="C_f",
            cpp_class="C",
            cpp_method="f",
            params=[("long", "value")],
            body_text="HRESULT C::f(long value) { return H(value); }",
        ),
        FunctionDep(
            name="H", params=[("int", "x")], body_text="int H(int x) { return x; }", is_static=True
        ),
        FunctionDep(
            name="H_2",
            params=[("long", "x")],
            body_text="int H(long x) { return x + 1; }",
            is_static=True,
        ),
    ]
    plan = prepare_helper_closure(roots, "C")
    method = next(dep for dep in plan if dep.name == "C_f")
    assert "H_2_body(value)" in method.body_text
    assert "H_body" not in method.body_text


def test_same_arity_helper_overload_ambiguity_fails_before_payment():
    roots = [
        FunctionDep(
            name="C_f",
            cpp_class="C",
            cpp_method="f",
            body_text="HRESULT C::f() { return H(UnknownExpression()); }",
        ),
        FunctionDep(
            name="H", params=[("int", "x")], body_text="int H(int x) { return x; }", is_static=True
        ),
        FunctionDep(
            name="H_2",
            params=[("long", "x")],
            body_text="int H(long x) { return x + 1; }",
            is_static=True,
        ),
    ]
    with pytest.raises(CppComError, match="cannot bind overloaded sibling call H"):
        prepare_helper_closure(roots, "C")


def test_same_arity_helper_overload_binds_from_nested_helper_return_type():
    roots = [
        FunctionDep(
            name="C_f",
            cpp_class="C",
            cpp_method="f",
            params=[("PCWSTR", "path")],
            body_text="UINT C::f(PCWSTR path) { return Kind(Number(path)); }",
        ),
        FunctionDep(
            name="Number",
            return_type="INT8",
            params=[("PCWSTR", "path")],
            body_text="INT8 Number(PCWSTR path) { return 0; }",
            is_static=True,
        ),
        FunctionDep(
            name="Kind",
            return_type="UINT",
            params=[("INT8", "number")],
            body_text="UINT Kind(INT8 number) { return number; }",
            is_static=True,
        ),
        FunctionDep(
            name="Kind_2",
            return_type="UINT",
            params=[("PCWSTR", "path")],
            body_text="UINT Kind(PCWSTR path) { return 1; }",
            is_static=True,
        ),
    ]
    plan = prepare_helper_closure(roots, "C")
    method = next(dep for dep in plan if dep.name == "C_f")
    assert "Kind_body(Number_body(path))" in method.body_text


def test_local_helper_default_argument_is_materialized_before_prompting():
    roots = [
        FunctionDep(
            name="C_f",
            cpp_class="C",
            cpp_method="f",
            body_text="BOOL C::f() { return Helper(7); }",
        ),
        FunctionDep(
            name="Helper",
            return_type="BOOL",
            params=[("INT", "value"), ("BOOL", "enabled")],
            body_text=("BOOL Helper(INT value, BOOL enabled = FALSE) { return enabled && value; }"),
            is_static=True,
        ),
    ]
    plan = prepare_helper_closure(roots, "C")
    method = next(dep for dep in plan if dep.name == "C_f")
    assert "Helper_body(7, FALSE)" in method.body_text


def test_scanner_keeps_cpp_optional_parameter_in_worker_signature():
    scan = scan_file(SHELL32 / "folders" / "CDrivesFolder.cpp")
    helper = next(
        function for function in scan.functions if function.name == "SHELL_GetAutoRunInfPath"
    )
    assert helper.params[-1] == ("BOOL", "ForInvoke")


def test_helper_arity_treats_comparison_as_expression_not_template_depth():
    roots = [
        FunctionDep(
            name="C_f",
            cpp_class="C",
            cpp_method="f",
            body_text="int C::f(int x, int y) { return H(x < y, y); }",
        ),
        FunctionDep(
            name="H",
            params=[("BOOL", "less"), ("int", "value")],
            body_text="int H(BOOL less, int value) { return less ? value : 0; }",
            is_static=True,
        ),
    ]
    plan = prepare_helper_closure(roots, "C")
    assert {dep.name for dep in plan} >= {"H", "C_f"}


def test_cpp_scanner_recovers_unadorned_out_of_line_method(tmp_path: Path):
    source = tmp_path / "method.cpp"
    source.write_text("BOOL CThing::Method(DWORD flags, void *context) { return flags != 0; }\n")
    scan = scan_file(source)
    assert scan.unextracted == []
    assert len(scan.functions) == 1
    method = scan.functions[0]
    assert (method.cpp_class, method.name, method.return_type) == ("CThing", "Method", "BOOL")
    assert method.params == [("DWORD", "flags"), ("void *", "context")]


def test_cpp_scanner_preserves_qualified_pointer_return_type(tmp_path: Path):
    source = tmp_path / "nested_return.cpp"
    source.write_text("CThing::ITEM *CThing::Find(DWORD id) { return nullptr; }\n")
    method = scan_file(source).functions[0]
    assert (method.cpp_class, method.name, method.return_type) == (
        "CThing",
        "Find",
        "CThing::ITEM *",
    )


def test_nested_pointer_pod_and_enum_are_deterministic_source_types(tmp_path: Path):
    source = tmp_path / "nested.cpp"
    source.write_text(
        """
class CThing : public CComCoClass<CThing>,
               public CComObjectRootEx<CComMultiThreadModelNoCS>, public IFoo {
  enum KIND { KIND_BAD = -1, KIND_ONE = 1, KIND_TWO };
  struct ITEM { KIND kind; DWORD size; ITEM *next; };
  ITEM *items;
  DECLARE_NOT_AGGREGATABLE(CThing)
  BEGIN_COM_MAP(CThing) COM_INTERFACE_ENTRY_IID(IID_IFoo, IFoo) END_COM_MAP()
};
"""
    )
    object_map = tmp_path / "shell32.cpp"
    object_map.write_text("OBJECT_ENTRY(CLSID_Thing, CThing)\n")
    cls = analyze_class(source, "CThing", object_map_source=object_map)
    cls.require_supported()
    assert cls.enum_state_types[0].variants == (
        ("KIND_BAD", -1),
        ("KIND_ONE", 1),
        ("KIND_TWO", 2),
    )
    assert [pod.name for pod in cls.pod_state_types] == ["ITEM"]
    rendered = render_class_scaffold(cls, [InterfaceSpec("IFoo", (), "test.h")])
    assert "pub type KIND = INT;" in rendered
    assert "pub struct ITEM" in rendered


def test_worker_audit_rejects_reordered_nested_calls():
    source = "void C::f() { HeapFree(GetProcessHeap(), 0, p); }"
    rust = "pub unsafe fn C_f_body(this: *mut C) { GetProcessHeap(HeapFree(), 0, p); }"
    findings = audit_worker_body(source, rust, "C_f_body", "pub unsafe fn C_f_body(this: *mut C)")
    assert any("runtime call nesting reversed" in finding for finding in findings)


def test_worker_audit_allows_pure_nested_call_hoisting():
    source = "void C::f() { HeapAlloc(GetProcessHeap(), 0, size); }"
    rust = (
        "pub unsafe fn C_f_body(this: *mut C) { "
        "let heap = GetProcessHeap(); HeapAlloc(heap, 0, size); }"
    )
    assert audit_worker_body(source, rust, "C_f_body", "pub unsafe fn C_f_body(this: *mut C)") == []


def test_cpp_scanner_expands_inline_atl_stDMETHOD_definition(tmp_path: Path):
    source = tmp_path / "inline.cpp"
    source.write_text(
        """
class CInline
{
    STDMETHOD(QueryService)(REFGUID svc, REFIID riid, void **ppv) override
    {
        return Forward(svc, riid, ppv);
    }
};
"""
    )
    scan = scan_file(source)
    assert scan.unextracted == []
    assert len(scan.functions) == 1
    method = scan.functions[0]
    assert (method.cpp_class, method.name, method.return_type) == (
        "CInline",
        "QueryService",
        "HRESULT",
    )
    assert method.params == [
        ("REFGUID", "svc"),
        ("REFIID", "riid"),
        ("void * *", "ppv"),
    ]


def test_internal_com_object_fails_until_construction_is_mapped():
    cls = analyze_class(
        SHELL32 / "folders" / "CControlPanelFolder.cpp",
        "CControlPanelEnum",
        object_map_source=SHELL32 / "shell32.cpp",
    )
    with pytest.raises(CppComError, match="internal COM-object construction is not mapped"):
        cls.require_supported()


def test_struct_class_object_entry_auto_and_fixed_scalar_array_are_supported():
    scan = scan_file(SHELL32 / "CRecycleBinCleaner.cpp")
    assert {item.cpp_class for item in scan.functions if item.name != "CreateInstance"} == {
        "CRecycleBinCleaner"
    }
    cls = analyze_class(
        SHELL32 / "CRecycleBinCleaner.cpp",
        "CRecycleBinCleaner",
        object_map_source=SHELL32 / "shell32.cpp",
    )
    cls.require_supported()
    assert cls.object_map_clsid == "CLSID_RecycleBinCleaner"
    assert cls.state_fields == ["WCHAR m_wszVolume[4]"]
    state = cls.plain_state_fields()[0]
    assert (state.name, state.rust_type) == ("m_wszVolume", "[WCHAR; 4]")


def test_concrete_local_base_is_flattened_with_inherited_state():
    # CMoveToMenu : CCopyMoveToMenu — the concrete same-header base is flattened
    # into the translation: its state precedes the derived class's own, and the
    # base is recorded so scan-extend re-owns its method bodies.
    cls = analyze_class(
        SHELL32 / "CCopyMoveToMenu.cpp",
        "CMoveToMenu",
        object_map_source=SHELL32 / "shell32.cpp",
    )
    cls.require_supported()
    assert cls.local_base_classes == ["CCopyMoveToMenu"]
    names = [_state_field_name(decl) for decl in cls.state_fields]
    assert "m_pidlFolder" in names and "m_fnOldWndProc" in names
    # The one mechanically-known ATL state template is flattened too.
    dispatch = analyze_class(
        SHELL32 / "CShellDispatch.cpp",
        "CShellDispatch",
        object_map_source=SHELL32 / "shell32.cpp",
    )
    dispatch.require_supported()
    assert dispatch.local_base_classes == ["IDispatchImpl"]
    assert dispatch.state_fields == ["CComPtr<ITypeInfo> m_pTypeInfo"]


def test_out_of_header_concrete_base_and_plain_member_class_layout_are_proven():
    cls = analyze_class(
        SHELL32 / "dialogs" / "filedefext.cpp",
        "CFileDefExt",
        object_map_source=SHELL32 / "shell32.cpp",
    )
    cls.require_supported()
    assert cls.local_base_classes == ["CObjectWithSiteBase"]
    assert cls.state_fields[0] == "IUnknown* m_pUnkSite"
    version = next(item for item in cls.pod_state_types if item.name == "CFileVersionInfo")
    assert [field.name for field in version.fields] == [
        "m_pInfo",
        "m_wLang",
        "m_wCode",
        "m_wszLang",
    ]

    scan = _extend_cpp_class_scan(
        scan_file(SHELL32 / "dialogs" / "filedefext.cpp"),
        SHELL32 / "dialogs" / "filedefext.cpp",
        "CFileDefExt",
        tuple(cls.local_base_classes),
    )
    base_dtor = next(item for item in scan.functions if item.name.endswith("_base_dtor"))
    assert (base_dtor.cpp_class, base_dtor.name, base_dtor.return_type, base_dtor.is_dtor) == (
        "CFileDefExt",
        "CObjectWithSiteBase_base_dtor",
        "void",
        False,
    )


@pytest.mark.parametrize(
    ("source", "name"),
    [
        (SHELL32 / "shellmenu" / "CMenuSite.cpp", "CMenuSite"),
        (SHELL32 / "shellmenu" / "CMenuDeskBar.cpp", "CMenuDeskBar"),
    ],
)
def test_shellmenu_window_impl_uses_rust_owned_hwnd_state(source: Path, name: str):
    cls = analyze_class(source, name, object_map_source=SHELL32 / "shell32.cpp")
    cls.require_supported()
    assert cls.state_fields[0] == "HWND m_hWnd"


def test_legacy_shell_interfaces_keep_unnamed_params_and_comment_spelling():
    desk = parse_widl_interface(REACTOS_ROOT / "sdk/include/psdk/shlobj.h", "IDeskBarClient")
    assert len(next(method for method in desk.methods if method.name == "GetSize").params) == 3
    banner = parse_widl_interface(
        REACTOS_ROOT / "sdk/include/reactos/shlobj_undoc.h", "IBanneredBar"
    )
    set_icon_size = next(method for method in banner.methods if method.name == "SetIconSize")
    assert len(set_icon_size.params) == 2


def test_zero_cost_cap_blocks_before_the_first_paid_call():
    assert _cost_cap_reached(0.0, 0.0)
    assert not _cost_cap_reached(None, 0.0)
    assert not _cost_cap_reached(1.0, 0.99)


def test_zero_only_cpp_initializer_list_is_deterministic():
    assert _trivial_cpp_body("Thing::Thing() : ptr(NULL), count(0), ready(FALSE) {}")
    assert not _trivial_cpp_body("Thing::Thing() : ptr(CreateThing()) {}")


def test_final_construct_is_wired_before_query_interface():
    cls = analyze_class(
        SHELL32 / "folders" / "CMyDocsFolder.cpp",
        "CMyDocsFolder",
        object_map_source=SHELL32 / "shell32.cpp",
    )
    cls.require_supported()
    assert cls.final_construct
    interfaces = [
        InterfaceSpec(
            entry.interface,
            (
                InterfaceMethod(
                    "QueryInterface",
                    "HRESULT",
                    ((f"{entry.interface} *", "This"), ("REFIID", "riid"), ("void **", "ppv")),
                ),
                InterfaceMethod("AddRef", "ULONG", ((f"{entry.interface} *", "This"),)),
                InterfaceMethod("Release", "ULONG", ((f"{entry.interface} *", "This"),)),
            ),
            "fixture.h",
        )
        for entry in cls.com_map
    ]
    rust = render_class_scaffold(cls, interfaces)
    construct = rust.index("CMyDocsFolder_FinalConstruct_body(this)")
    query = rust.index("CMyDocsFolder_QueryInterface(this, riid, ppv)", construct)
    assert construct < query
    assert "if construct_hr < 0" in rust
    assert "CMyDocsFolder_Release(this);" in rust


def test_static_methods_and_destructor_debris_are_not_object_state():
    folder = analyze_class(
        SHELL32 / "CFolderOptions.cpp",
        "CFolderOptions",
        object_map_source=SHELL32 / "shell32.cpp",
    )
    assert folder.state_fields == ["CComPtr<IUnknown> m_pSite"]
    notification = analyze_class(
        SHELL32 / "CUserNotification.cpp",
        "CUserNotification",
        object_map_source=SHELL32 / "shell32.cpp",
    )
    assert all(not field.startswith(("static ", "~", "#")) for field in notification.state_fields)


def test_user_notification_support_uses_32_bit_window_long_exports():
    rust = render_cpp_support(SHELL32 / "CUserNotification.cpp")
    assert '#[cfg_attr(target_pointer_width = "32", link_name = "GetWindowLongW")]' in rust
    assert '#[cfg_attr(target_pointer_width = "32", link_name = "SetWindowLongW")]' in rust


def test_cpp_scanner_strips_legacy_sal_direction_tokens(tmp_path: Path):
    source = tmp_path / "annotated.cpp"
    source.write_text(
        "VOID CThing::Delay(IN HRESULT hRes, OUT DWORD *value OPTIONAL) { (void)hRes; }"
    )
    functions = scan_file(source, force_cpp=True).functions
    assert len(functions) == 1
    assert functions[0].params == [("HRESULT", "hRes"), ("DWORD *", "value")]


def test_cpp_scanner_recovers_return_type_before_winapi_qualified_method():
    functions = scan_file(SHELL32 / "folders" / "CDrivesFolder.cpp", force_cpp=True).functions
    final_construct = next(item for item in functions if item.name == "FinalConstruct")
    assert final_construct.return_type == "HRESULT"


def test_cpp_header_scan_extracts_inline_class_methods():
    scan = scan_file(SHELL32 / "folders" / "CMyDocsFolder.h", force_cpp=True)
    methods = {item.name for item in scan.functions if item.cpp_class == "CMyDocsFolder"}
    assert {"GetMaxLength", "GetValidCharacters"} <= methods


def test_worker_audit_preserves_constants_and_rejects_abi_surface():
    c_body = '{ FIXME("x"); if (flags & 0x20) return 7; return E_NOTIMPL; }'
    good = (
        "pub unsafe fn C_f_body(this: *mut C) -> HRESULT "
        '{ FIXME!("x"); if flags & 0x20 != 0 { return 7; } E_NOTIMPL }'
    )
    assert audit_worker_body(c_body, good, "C_f_body") == []

    bad = '#[no_mangle] pub extern "system" fn C_f_body() -> HRESULT { 0 }'
    findings = audit_worker_body(c_body, bad, "C_f_body")
    assert any("export attribute" in finding for finding in findings)
    assert any("ABI declaration" in finding for finding in findings)
    assert any("E_NOTIMPL" in finding for finding in findings)
    assert any("numeric C literal" in finding for finding in findings)


def test_worker_audit_allows_vtable_calls_but_rejects_option_guessing():
    source = "{ return object->GetData(out); }"
    good = "pub unsafe fn C_f_body() -> HRESULT { ((*(*object).lpVtbl).GetData)(object, out) }"
    assert audit_worker_body(source, good, "C_f_body") == []
    bad = (
        "pub unsafe fn C_f_body() -> HRESULT { "
        "(*object).lpVtbl.as_ref().unwrap().GetData.unwrap()(object, out) }"
    )
    findings = audit_worker_body(source, bad, "C_f_body")
    assert any("Option/panic ABI guess" in item for item in findings)


def test_worker_audit_allows_function_pointer_types_but_rejects_extern_declarations():
    source = "{ return register_callback(Worker); }"
    good = (
        "pub unsafe fn C_f_body() -> HRESULT { register_callback(Worker as unsafe extern "
        '"system" fn(HWND, UINT, WPARAM, LPARAM) -> LRESULT) }'
    )
    assert audit_worker_body(source, good, "C_f_body") == []
    bad = 'pub unsafe fn C_f_body() { extern "system" { fn invented(); } }'
    assert any("ABI declaration" in item for item in audit_worker_body(source, bad, "C_f_body"))


def test_unused_ccomptr_model_debris_is_removed():
    source = "{ CComPtr<IShellView> view; return E_NOTIMPL; }"
    rust = (
        "pub unsafe fn C_f_body() -> HRESULT {\n"
        "    let view: *mut IShellView = core::ptr::null_mut();\n"
        "    if !view.is_null() { ((*(*view).lpVtbl).Release)(view); }\n"
        "    E_NOTIMPL\n"
        "}"
    )
    normalized = normalize_worker_semantics(source, rust)
    assert "let view" not in normalized
    assert "Release" not in normalized


def test_worker_audit_allows_query_interface_calls_but_rejects_implementation():
    source = "{ return object->QueryInterface(riid, out); }"
    call = (
        "pub unsafe fn C_f_body() -> HRESULT { "
        "((*(*object).lpVtbl).QueryInterface)(object, riid, out) }"
    )
    assert audit_worker_body(source, call, "C_f_body") == []

    implementation = "pub unsafe fn C_QueryInterface() -> HRESULT { S_OK }"
    findings = audit_worker_body(source, implementation, "C_QueryInterface")
    assert any("QueryInterface implementation" in item for item in findings)


def test_worker_audit_rejects_raw_c_control_flow_but_ignores_comments():
    signature = "pub unsafe fn C_Work_body(this: *mut C) -> HRESULT"
    rust = signature + " { // goto harmless;\n goto cleanup;\ncleanup:\n S_OK }"
    findings = audit_worker_body(
        "HRESULT C::Work() { return S_OK; }", rust, "C_Work_body", signature
    )
    assert "worker emitted raw C goto/label syntax" in findings


def test_worker_audit_allows_create_instance_calls_but_rejects_implementations():
    source = "{ return CoCreateInstance(clsid, NULL, flags, iid, out); }"
    call = "pub unsafe fn C_f_body() -> HRESULT { CoCreateInstance(clsid, NULL, flags, iid, out) }"
    assert audit_worker_body(source, call, "C_f_body") == []
    implementation = "pub unsafe fn C_CreateInstance() -> HRESULT { E_NOTIMPL }"
    findings = audit_worker_body(source, implementation, "C_CreateInstance")
    assert any("activation factory implementation" in item for item in findings)


def test_worker_semantics_restores_cpp_com_ptr_scope_exit_ownership():
    source = """HRESULT C::f() {
        CComPtr<IShellFolder> folder;
        CComPtr<IShellFolder> desktop;
        return SHGetDesktopFolder(&folder);
    }"""
    rust = """pub unsafe fn C_f_body() -> HRESULT {
        let mut folder: *mut IShellFolder = core::ptr::null_mut();
        let mut desktop: *mut IShellFolder = core::ptr::null_mut();
        let hr = SHGetDesktopFolder(&mut folder);
        if hr < 0 { return hr; }
        ((*(*folder).lpVtbl).Release)(folder);
        hr
    }"""
    normalized = normalize_worker_semantics(source, rust)
    assert "let _folder_cpp_com_guard = CppComPtrGuard::new(" in normalized
    assert "let _desktop_cpp_com_guard = CppComPtrGuard::new(" in normalized

    replayed = normalize_worker_semantics(source, normalized)
    assert replayed == normalized
    assert "folder = core::ptr::null_mut();" in normalized


def test_worker_semantics_lowers_model_atl_com_ptr_wrapper_and_iid_pair():
    source = """HRESULT C::f() {
        CComPtr<IShellView> view;
        return IUnknown_QueryService(m_pSite, SID_SFolderView,
                                     IID_PPV_ARG(IShellView, &view));
    }"""
    rust = """pub unsafe fn C_f_body(this: *mut C) -> HRESULT {
        let mut view: CComPtr<IShellView> = CComPtr::new();
        if (*this).m_pSite.0.is_null() { return S_OK; }
        let (iid, ppv) = IID_PPV_ARG(IShellView, &mut view);
        IUnknown_QueryService((*this).m_pSite.0, &SID_SFolderView, iid, ppv)
    }"""
    normalized = normalize_worker_semantics(source, rust)
    assert "CComPtr<IShellView>" not in normalized
    assert "let mut view: *mut IShellView = core::ptr::null_mut();" in normalized
    assert "let _view_cpp_com_guard = CppComPtrGuard::new(" in normalized
    assert "(*this).m_pSite.0" not in normalized
    assert "core::ptr::addr_of!(IID_IShellView)" in normalized
    assert "core::ptr::addr_of_mut!(view).cast()" in normalized


def test_sibling_receiver_insertion_preserves_explicit_other_object_receiver():
    source = """HRESULT C::f(ARG a) {
        C_g_body(a);
        other->g(a);
    }"""
    rust = """pub unsafe fn C_f_body(this: *mut C, a: ARG) {
        C_g_body(a);
        C_g_body(other, a);
    }"""
    normalized = normalize_worker_semantics(source, rust)
    assert "C_g_body(this, a)" in normalized
    assert "C_g_body(other, a)" in normalized
    assert "C_g_body(this, other, a)" not in normalized


def test_inline_struct_receiver_resource_string_and_callback_are_lowered():
    source = """__stdcall Initialize() {
        CStringW Tmp(MAKEINTRESOURCEW(IDS_TEXT));
        C_Output_body(IDS_TEXT, out);
        *pdwlSpaceUsed = bin.i64Size;
        picb->ScanProgress(bin.i64Size, flags, NULL);
    }"""
    rust = """pub unsafe fn C_Initialize_body(this: *mut C) {
        let Tmp = CStringW::new((IDS_TEXT as u16) as *mut u16);
        SHStrDupW(Tmp, out);
        C_Output_body(IDS_TEXT, out);
        *pdwlSpaceUsed = bin.i64Size;
        ((*(*picb)).lpVtbl).ScanProgress(picb, bin.i64Size, flags, core::ptr::null());
    }"""
    normalized = normalize_worker_semantics(source, rust)
    assert "CStringW::from_resource(IDS_TEXT as UINT)" in normalized
    assert "SHStrDupW(Tmp.as_ptr(), out)" in normalized
    assert "C_Output_body(this, IDS_TEXT, out)" in normalized
    assert "*pdwlSpaceUsed = bin.i64Size as DWORDLONG" in normalized
    assert "((*(*picb).lpVtbl).ScanProgress)(" in normalized


def test_worker_semantics_lowers_static_guid_predicate_without_model_pointer_cast():
    normalized = normalize_worker_semantics(
        "HRESULT f() { return IsEqualGUID(rbhid, BHID_SFObject); }",
        "pub unsafe fn f() -> bool { IsEqualGUID(rbhid, BHID_SFObject) }",
    )
    assert "cpp_com_iid_equal(rbhid, core::ptr::addr_of!(BHID_SFObject))" in normalized


def test_custom_body_local_raii_fails_before_payment():
    dep = FunctionDep(name="C_AddPages", body_text="HRESULT f() { CDrop drop(data); }")
    with pytest.raises(CppComError, match="CDrop local"):
        require_supported_worker_raii([dep])


def test_worker_semantics_rejects_unanchored_cpp_com_ptr_local():
    source = "CComPtr<IShellFolder> folder; return use(folder);"
    rust = "pub unsafe fn C_f_body() -> HRESULT { use_folder() }"
    with pytest.raises(CppComError, match="CComPtr local folder"):
        normalize_worker_semantics(source, rust)


def test_worker_semantics_owns_ccom_ptr_parameter_copy():
    source = "CComPtr<IContextMenu> ctxMenu = contextMenu; return use(ctxMenu);"
    rust = """pub unsafe fn C_f_body(contextMenu: *mut IContextMenu) -> HRESULT {
        let ctxMenu = contextMenu;
        use(ctxMenu)
    }"""
    normalized = normalize_worker_semantics(source, rust)
    assert "let mut ctxMenu: *mut IContextMenu = contextMenu;" in normalized
    assert "((*(*ctxMenu).lpVtbl).AddRef)(ctxMenu)" in normalized
    assert "CppComPtrGuard::new(core::ptr::addr_of_mut!(ctxMenu))" in normalized


def test_worker_semantics_repairs_split_iid_ppv_arg_binding():
    source = "return object->QueryInterface(IID_PPV_ARG(IMenuPopup, &m_child));"
    rust = """pub unsafe fn C_f_body(this: *mut C, object: *mut IUnknown) -> HRESULT {
        let riid = IID_PPV_ARG(IMenuPopup; let ppv = &mut (*this).m_child);
        ((*(*object).lpVtbl).QueryInterface)(object, riid, ppv)
    }"""
    normalized = normalize_worker_semantics(source, rust)
    assert "IID_PPV_ARG" not in normalized
    assert "let riid = core::ptr::addr_of!(IID_IMenuPopup);" in normalized
    assert "let ppv = core::ptr::addr_of_mut!((*this).m_child).cast();" in normalized


def test_worker_audit_rejects_model_owned_scope_guards():
    source = "{ return S_OK; }"
    rust = "pub unsafe fn C_f_body() -> HRESULT { let _g = scopeguard::guard(p, drop); S_OK }"
    findings = audit_worker_body(source, rust, "C_f_body")
    assert any("model-owned ownership guard" in item for item in findings)


def test_worker_audit_rejects_body_local_abi_types():
    source = "{ return Call(); }"
    rust = """pub unsafe fn C_f_body() -> HRESULT {
        #[repr(C)] struct ABI_INFO { size: DWORD }
        Call()
    }"""
    findings = audit_worker_body(source, rust, "C_f_body")
    assert any("ABI type declaration" in item for item in findings)


def test_worker_audit_allows_exact_source_local_pod_layout():
    source = """HRESULT f() {
        struct sized_chunk { DWORD size; unsigned char data[1]; } *chunk;
        return Call(chunk);
    }"""
    rust = """pub unsafe fn C_f_body() -> HRESULT {
        #[repr(C)] struct sized_chunk { size: DWORD, data: [u8; 1] }
        Call(core::ptr::null_mut::<sized_chunk>())
    }"""
    assert audit_worker_body(source, rust, "C_f_body") == []


def test_worker_audit_rejects_changed_source_local_pod_layout():
    source = """HRESULT f() {
        struct sized_chunk { DWORD size; unsigned char data[1]; } *chunk;
        return Call(chunk);
    }"""
    rust = """pub unsafe fn C_f_body() -> HRESULT {
        #[repr(C)] struct sized_chunk { size: DWORD, invented: DWORD }
        Call(core::ptr::null_mut::<sized_chunk>())
    }"""
    findings = audit_worker_body(source, rust, "C_f_body")
    assert any("ABI type declaration" in item for item in findings)


def test_worker_semantics_discards_model_shadow_of_source_named_sdk_type():
    source = "HRESULT f() { RECT rc = { 0, 0, 1, 1 }; return Call(&rc); }"
    rust = """pub unsafe fn C_f_body() -> HRESULT {
        #[repr(C)] struct RECT { left: i32, top: i32, right: i32, bottom: i32 }
        let rc = RECT { left: 0, top: 0, right: 1, bottom: 1 };
        Call(&rc)
    }"""
    normalized = normalize_worker_semantics(source, rust)
    assert "struct RECT" not in normalized
    assert "let rc = RECT" in normalized
    assert audit_worker_body(source, normalized, "C_f_body") == []


def test_worker_semantics_keeps_source_local_pod_for_layout_audit():
    source = "HRESULT f() { struct local { DWORD size; } value; return Call(&value); }"
    rust = """pub unsafe fn C_f_body() -> HRESULT {
        #[repr(C)] struct local { size: DWORD }
        let value: local = core::mem::zeroed();
        Call(&value)
    }"""
    normalized = normalize_worker_semantics(source, rust)
    assert "struct local" in normalized
    assert audit_worker_body(source, normalized, "C_f_body") == []


def test_worker_semantics_raw_escapes_cpp_final_local_only_in_code():
    source = 'HRESULT f() { LPWSTR final = NULL; TRACE("final"); return Use(final); }'
    rust = """pub unsafe fn C_f_body() -> HRESULT {
        let mut final: LPWSTR = core::ptr::null_mut();
        TRACE!("final");
        Use(final)
    }"""
    normalized = normalize_worker_semantics(source, rust)
    assert "let mut r#final" in normalized
    assert "Use(r#final)" in normalized
    assert 'TRACE!("final")' in normalized


def test_worker_semantics_lowers_balanced_countof_member_forms():
    source = "HRESULT f(INFO *p) { return Use(_countof(p->data)); }"
    rust = """pub unsafe fn C_f_body(p: *mut INFO) -> HRESULT {
        Use(_countof((*p).data) as UINT)
            + Use(_countof!((*p).other))
            + Use(_countof(&(*p).third))
    }"""
    normalized = normalize_worker_semantics(source, rust)
    assert "_countof" not in normalized
    assert "((*p).data).len() as UINT" in normalized
    assert "((*p).other).len()" in normalized
    assert "((*p).third).len()" in normalized


def test_cpp_support_generates_used_source_local_pods_and_honors_exclusion(tmp_path: Path):
    source = tmp_path / "local.cpp"
    source.write_text(
        '#include "pshpack1.h"\n'
        "struct LOCAL_INFO { DWORD size; WCHAR label[12]; };\n"
        '#include "poppack.h"\n'
        "struct UNUSED_INFO { DWORD ignored; };\n"
        "HRESULT f(LOCAL_INFO *info) { return info->size; }\n"
    )
    support = render_cpp_support(source)
    assert "#[repr(C, packed)]\npub struct LOCAL_INFO" in support
    assert "pub size: DWORD" in support
    assert "pub label: [WCHAR; 12]" in support
    assert "UNUSED_INFO" not in support
    assert "struct LOCAL_INFO" not in render_cpp_support(source, exclude_types={"LOCAL_INFO"})


def test_class_state_expands_plain_multi_declarators(tmp_path: Path):
    source = tmp_path / "CMulti.cpp"
    source.write_text(
        "class CMulti : public CComCoClass<CMulti, &CLSID_Multi>, "
        "public CComObjectRootEx<CComMultiThreadModelNoCS>, public IUnknown {\n"
        "UINT first, second;\nHICON icon, fallback;\n"
        "DECLARE_NOT_AGGREGATABLE(CMulti)\n"
        "BEGIN_COM_MAP(CMulti) COM_INTERFACE_ENTRY(IUnknown) END_COM_MAP()\n};\n"
        "OBJECT_ENTRY_AUTO(CLSID_Multi, CMulti)\n"
    )
    cls = analyze_class(source, "CMulti")
    assert cls.state_fields == ["UINT first", "UINT second", "HICON icon", "HICON fallback"]
    assert [field.name for field in cls.plain_state_fields()] == [
        "first",
        "second",
        "icon",
        "fallback",
    ]


def test_central_instance_macro_is_proven_non_aggregatable(tmp_path: Path):
    source = tmp_path / "CCentral.cpp"
    source.write_text(
        "class CCentral : public CComCoClass<CCentral, &CLSID_Central>, "
        "public CComObjectRootEx<CComMultiThreadModelNoCS>, public IUnknown {\n"
        "DECLARE_CENTRAL_INSTANCE_NOT_AGGREGATABLE(CCentral)\n"
        "BEGIN_COM_MAP(CCentral) COM_INTERFACE_ENTRY(IUnknown) END_COM_MAP()\n};\n"
        "OBJECT_ENTRY_AUTO(CLSID_Central, CCentral)\n"
    )
    cls = analyze_class(source, "CCentral")
    assert cls.not_aggregatable


def test_cpp_support_derives_unique_numeric_sdk_facts(tmp_path: Path):
    repo = tmp_path / "repo"
    source = repo / "dll/win32/example/worker.cpp"
    header = repo / "sdk/include/psdk/example.h"
    source.parent.mkdir(parents=True)
    header.parent.mkdir(parents=True)
    source.write_text("DWORD f() { return SDK_FLAG | CONFLICT_FLAG | EXPRESSION_FLAG; }")
    header.write_text(
        "#define SDK_FLAG 0x40U\n"
        "#define CONFLICT_FLAG 1\n"
        "#define EXPRESSION_FLAG (1 << 2)\n"
        "enum E {\n    ENUM_FLAG = 7,\n};\n"
    )
    second = repo / "sdk/include/psdk/conflict.h"
    second.write_text("#define CONFLICT_FLAG 2\n")
    support = render_cpp_support(source, additional_source="ENUM_FLAG")
    assert "pub const SDK_FLAG: DWORD = 0x40;" in support
    assert "pub const ENUM_FLAG: DWORD = 7;" in support
    assert "CONFLICT_FLAG: DWORD" not in support
    assert "EXPRESSION_FLAG: DWORD" not in support


def test_cpp_support_derives_referenced_sdk_pod_closure(tmp_path: Path):
    repo = tmp_path / "repo"
    source = repo / "dll/win32/example/worker.cpp"
    header = repo / "sdk/include/psdk/example.h"
    source.parent.mkdir(parents=True)
    header.parent.mkdir(parents=True)
    source.write_text("HRESULT f(LPOUTER value) { return value->inner.value; }")
    header.write_text(
        "#define LABEL_COUNT 4\n"
        "typedef struct tagINNER { DWORD value; } INNER, *LPINNER;\n"
        "typedef struct { INNER inner; _Field_z_ WCHAR label[LABEL_COUNT]; } OUTER, *LPOUTER;\n"
        "typedef struct { DWORD ignored; } UNUSED, *LPUNUSED;\n"
    )
    support = render_cpp_support(source)
    assert "pub struct INNER" in support
    assert "pub value: DWORD" in support
    assert "pub struct OUTER" in support
    assert "pub inner: INNER" in support
    assert "pub label: [WCHAR; 4]" in support
    assert "pub type LPOUTER = *mut OUTER;" in support
    assert "UNUSED" not in support


def test_cpp_support_rejects_conflicting_sdk_pod_layouts(tmp_path: Path):
    repo = tmp_path / "repo"
    source = repo / "dll/win32/example/worker.cpp"
    include = repo / "sdk/include/psdk"
    source.parent.mkdir(parents=True)
    include.mkdir(parents=True)
    source.write_text("HRESULT f(LPCONFLICT value);")
    (include / "first.h").write_text("typedef struct { DWORD first; } CONFLICT, *LPCONFLICT;\n")
    (include / "second.h").write_text("typedef struct { DWORD second; } CONFLICT, *LPCONFLICT;\n")
    support = render_cpp_support(source)
    assert "pub struct CONFLICT" not in support
    assert "pub type LPCONFLICT" not in support


def test_cpp_support_derives_shell_property_sheet_callback_alias(tmp_path: Path):
    repo = tmp_path / "repo"
    source = repo / "dll/win32/example/worker.cpp"
    (repo / "sdk/include").mkdir(parents=True)
    source.parent.mkdir(parents=True)
    source.write_text("HRESULT f(EXPPS page, LPFNSVADDPROPSHEETPAGE callback);")
    support = render_cpp_support(source)
    assert "pub type EXPPS = UINT;" in support
    assert "pub type LPFNSVADDPROPSHEETPAGE" in support
    assert 'extern "system" fn(HPROPSHEETPAGE, LPARAM) -> BOOL' in support


def test_worker_audit_ignores_abi_words_in_comments():
    source = "{ return S_OK; }"
    rust = "pub unsafe fn C_f_body() -> HRESULT { // define struct elsewhere\n S_OK }"
    assert audit_worker_body(source, rust, "C_f_body") == []


def test_worker_normalizer_discards_all_model_owned_module_surface():
    signature = "pub unsafe fn C_f_body(this: *mut C) -> HRESULT"
    rust = """#![allow(dead_code)]
use crate::invented::*;
#[repr(C)] struct RECT { left: i32 }
pub unsafe fn C_f_body(this: *mut C) -> HRESULT { Call(this) }
fn trailing_helper() {}
"""
    assert normalize_worker_body(rust, signature) == signature + " { Call(this) }"


def test_proven_unimplemented_stub_is_lowered_without_a_model():
    signature = "pub unsafe fn C_f_body(this: *mut C) -> HRESULT"
    assert lower_trivial_worker("C::f() { UNIMPLEMENTED; return E_NOTIMPL; }", signature) == (
        signature + " { E_NOTIMPL }"
    )
    assert lower_trivial_worker("C::f() { UNIMPLEMENTED; mutate_state(); }", signature) is None
    assert lower_trivial_worker('C::f() { FIXME("%p", this); return E_NOTIMPL; }', signature) == (
        signature + " { E_NOTIMPL }"
    )
    assert lower_trivial_worker('C::f() { TRACE("%p", this); mutate_state(); }', signature) is None


def test_model_view_strips_only_complete_debug_statements():
    source = 'C::f() { TRACE("x"); value = Call(); if (WARN("y")) return 1; ERR("z"); }'
    stripped = strip_debug_statements(source)
    assert 'TRACE("x")' not in stripped
    assert 'ERR("z")' not in stripped
    assert 'WARN("y")' in stripped
    assert "value = Call()" in stripped


def test_empty_constructor_only_lowers_proven_zero_initializers():
    signature = "pub unsafe fn C_ctor_body(this: *mut C)"
    assert (
        lower_trivial_worker("C::C() : m_pointer(NULL), m_count(0) {}", signature)
        == signature + " {}"
    )
    assert lower_trivial_worker("C::C() : m_count(7) {}", signature) is None
    assert lower_trivial_worker("C::C() : m_owner(CreateOwner()) {}", signature) is None


def test_proven_out_assignment_is_lowered_without_a_model():
    signature = "pub unsafe fn C_Get_body(this: *mut C, pView: *mut CPVIEW) -> HRESULT"
    assert (
        lower_trivial_worker(
            "HRESULT C::Get(CPVIEW *pView) { *pView = CPVIEW_CLASSIC; return S_OK; }",
            signature,
        )
        == signature + " { *pView = CPVIEW_CLASSIC; S_OK }"
    )


def test_shell32_stitcher_never_imports_kernel_substrate():
    imports = _generate_imports_from_resolve(ResolveResult(), target=TARGETS["shell32"])
    assert "use crate::shellcom::*;" in imports
    assert "use crate::types::*;" in imports
    assert not any("ntkrnl_sys" in item for item in imports)


def test_worker_normalizer_removes_invented_debug_extern_only():
    raw = """pub unsafe fn C_f_body() -> HRESULT {
    extern "C" { fn FIXME(format: *const i8, ...); }
    FIXME(b"x\\0".as_ptr() as *const i8);
    E_NOTIMPL
}"""
    normalized = normalize_worker_body(raw)
    assert 'extern "C"' not in normalized
    assert "FIXME!(" in normalized
    assert audit_worker_body('{ FIXME("x"); return E_NOTIMPL; }', normalized, "C_f_body") == []
    assert normalize_worker_body("fn f() -> HRESULT { HRESULT(E_NOTIMPL) }").endswith(
        "{ E_NOTIMPL }"
    )


def test_worker_normalizer_removes_declaration_only_ffi_blocks():
    raw = """pub unsafe fn C_f_body() -> HRESULT {
    extern "system" {
        fn GetLastError() -> DWORD;
        fn Call(x: DWORD) -> HRESULT;
    }
    Call(GetLastError())
}"""
    normalized = normalize_worker_body(raw)
    assert 'extern "system"' not in normalized
    assert "Call(GetLastError())" in normalized


def test_worker_normalizer_discards_token_declaration_before_call_seal_audit():
    sealed = seal_calls("HRESULT f() { return Call(1); }", {"Call"})
    raw = """pub unsafe fn f() -> HRESULT {
        extern "system" { fn __RUSTCONV_CALL_0000__(value: DWORD) -> HRESULT; }
        __RUSTCONV_CALL_0000__(1)
    }"""
    normalized = normalize_worker_body(raw)
    assert 'extern "system"' not in normalized
    restored = unseal_calls(normalized, sealed)
    assert "Call(1)" in restored


def test_worker_normalizer_restores_generated_parameter_names():
    raw = "pub unsafe fn Worker(_this: *mut C, _value: DWORD) { use_it(_value) }"
    signature = "pub unsafe fn Worker(this: *mut C, value: DWORD)"
    normalized = normalize_worker_body(raw, signature)
    assert "fn Worker(this: *mut C, value: DWORD)" in normalized
    assert "use_it(value)" in normalized


def test_worker_normalizer_replaces_model_type_spellings_with_generated_signature():
    raw = "pub unsafe fn Worker(this: *mut C, value: u32) -> i32 { value as i32 }"
    signature = "pub unsafe fn Worker(this: *mut C, value: DWORD) -> HRESULT"
    normalized = normalize_worker_body(raw, signature)
    assert normalized == signature + " { value as i32 }"


def test_worker_normalizer_promotes_proven_constant_array_bound_only():
    raw = """pub unsafe fn Worker(n: usize) {
    let fixed: usize = (38 + 1) as usize;
    let runtime: usize = n + 1;
    let a: [u16; fixed] = [0; fixed];
    use_it(a, runtime);
}"""
    normalized = normalize_worker_body(raw)
    assert "const fixed: usize = (38 + 1) as usize;" in normalized
    assert "let runtime: usize = n + 1;" in normalized


def test_cpp_support_closure_is_source_selected_and_recovers_file_scope_table():
    rust = render_cpp_support(SHELL32 / "CCopyAsPathMenu.cpp")
    assert "pub const IDS_COPYASPATHMENU: UINT = 30328;" in rust
    assert 'CMVERBMAP { Verb: b"copyaspath\\0".as_ptr()' in rust
    assert "fn SHCreateShellItemArrayFromDataObject" in rust
    assert "fn CoTaskMemFree" in rust  # selected by CComHeapPtr ownership
    assert "pub struct CStringW" in rust
    assert "fn HeapReAlloc" in rust
    assert rust.count("fn InsertMenuItemW") == 1
    assert 'link_name = "Shell32RustMapContextMenuVerbToCmdId"' in rust


def test_string_cb_printf_support_is_rust_fixed_arity_not_variadic_block(tmp_path):
    source = tmp_path / "format.cpp"
    source.write_text('void f() { StringCbPrintfW(out, bytes, L"%s", value); }')
    support = render_cpp_support(source)
    assert "unsafe fn cpp_com_string_cb_printf_w" in support
    assert "macro_rules! StringCbPrintfW" in support
    assert "fn StringCbPrintfW(" not in support
    normalized = normalize_worker_semantics(
        source.read_text(),
        "unsafe fn f() { StringCbPrintfW(out, bytes, format, value); }",
    )
    assert "StringCbPrintfW!(out, bytes, format, value)" in normalized


def test_copy_as_path_adapter_bridges_internal_cpp_linkage():
    cls = analyze_class(
        SHELL32 / "CCopyAsPathMenu.cpp",
        "CCopyAsPathMenu",
        object_map_source=SHELL32 / "shell32.cpp",
    )
    adapter = render_object_map_adapter(cls)
    assert "class CCopyAsPathMenuRustEntry" in adapter
    assert "Shell32RustMapContextMenuVerbToCmdId" in adapter
    assert "Shell32RustGetCommandStringImpl" in adapter


def test_cpp_semantic_lowering_runs_after_cache_and_owns_header_macros():
    source = """HRESULT C::f() {
        SIZE_T cch = lstrlenW(str) + 1;
        CStringW text(MAKEINTRESOURCEW(IDS_TEXT));
        CComPtr<IShellItemArray> array;
        return Call(IID_PPV_ARG(IShellItemArray, &array));
    }"""
    rust = """pub unsafe fn C_f_body() -> HRESULT {
        let cch = lstrlenW(str) + 1;
        let str_text: *const u16 = IDS_TEXT as usize as *const u16;
        mii.dwTypeData = str_text as *mut u16;
        let mut array: *mut IShellItemArray = core::ptr::null_mut();
        Call(IID_PPV_ARG(IShellItemArray, &mut array))
    }"""
    lowered = normalize_worker_semantics(source, rust)
    assert "let cch: SIZE_T = lstrlenW(str) as SIZE_T + 1;" in lowered
    assert "CStringW::from_resource(IDS_TEXT as UINT)" in lowered
    assert "str_text.as_mut_ptr()" in lowered
    assert "core::ptr::addr_of!(IID_IShellItemArray)" in lowered
    assert "core::ptr::addr_of_mut!(array).cast()" in lowered


def test_cshellitem_support_and_semantics_are_deterministic():
    source_path = SHELL32 / "CShellItem.cpp"
    support = render_cpp_support(source_path)
    assert "fn ILClone" in support
    assert "fn SHGetDesktopFolder" in support
    assert 'extern "C" {\n    fn _ILIsDesktop' in support
    assert "static BHID_SFObject: GUID" in support
    assert "static IID_IShellFolder: GUID" in support
    source = """HRESULT CShellItem::f() {
        return CShellItem_get_shellfolder_body(this, pbc, riid, out);
    }"""
    rust = """pub unsafe fn C_f_body(this: *mut C) -> HRESULT {
        let mut out: *mut IShellFolder = std::ptr::null_mut();
        if FAILED_UNEXPECTEDLY(hr) { return hr; }
        CShellItem_get_shellfolder_body(
            pbc, IID_PPV_ARG(IShellFolder, &mut out as *mut *mut IShellFolder)
        )
    }"""
    normalized = normalize_worker_semantics(source, rust)
    assert "core::ptr::null_mut" in normalized
    assert "if ((hr) < 0)" in normalized
    assert "CShellItem_get_shellfolder_body(this, pbc," in normalized
    assert "core::ptr::addr_of!(IID_IShellFolder)" in normalized
    recursive = normalize_worker_semantics(
        "HRESULT CShellItem::BindToHandler() { return CShellItem_BindToHandler_body(pbc); }",
        "pub unsafe fn CShellItem_BindToHandler_body(this: *mut CShellItem, pbc: *mut IBindCtx) "
        "-> HRESULT { CShellItem_BindToHandler_body(pbc) }",
    )
    assert "fn CShellItem_BindToHandler_body(this: *mut CShellItem" in recursive
    assert "CShellItem_BindToHandler_body(this, pbc)" in recursive
    tuple_lowered = normalize_worker_semantics(
        "HRESULT CShellItem::f() { return IID_PPV_ARG(IShellFolder, ppsf); }",
        "pub unsafe fn f() { let (riid, ppv) = IID_PPV_ARG(ppsf); }",
    )
    assert "let riid = core::ptr::addr_of!(IID_IShellFolder);" in tuple_lowered
    assert "let ppv = ppsf.cast();" in tuple_lowered


def test_helper_closure_does_not_rebind_other_object_com_method():
    source = SHELL32 / "CShellItem.cpp"
    cls = analyze_class(source, "CShellItem", object_map_source=SHELL32 / "shell32.cpp")
    deps = prepare_helper_closure(
        extract_call_graph(scan_file(source)), "CShellItem", cls.plain_state_fields()
    )
    compare = next(dep for dep in deps if dep.name == "CShellItem_Compare")
    assert "pIDList->GetIDList" in compare.body_text
    assert "pIDList->CShellItem_GetIDList_body" not in compare.body_text


def test_folder_pair_support_closes_internal_factory_and_header_macros():
    source = SHELL32 / "folders" / "CFontsFolder.cpp"
    support = render_cpp_support(source)
    assert 'link_name = "Shell32RustCoCreateInitSF"' in support
    assert "fn _ILIsSpecialFolder" in support
    assert "static CLSID_ShellFSFolder: GUID" in support
    assert "static IID_IShellFolder2: GUID" in support
    assert "static IID_IShellFolder2: GUID" not in render_cpp_support(source, {"IID_IShellFolder2"})
    cls = analyze_class(source, "CFontsFolder", object_map_source=SHELL32 / "shell32.cpp")
    adapter = render_object_map_adapter(cls)
    assert "Shell32RustCoCreateInitSF" in adapter


def test_mydocs_support_includes_header_inline_dependencies_and_sdk_facts():
    source = SHELL32 / "folders" / "CMyDocsFolder.cpp"
    header = (SHELL32 / "folders" / "CMyDocsFolder.h").read_text()
    support = render_cpp_support(source, additional_source=header)
    assert "fn _ILCreateMyDocuments() -> LPITEMIDLIST" in support
    assert "fn SHStrDupW(source: LPCWSTR" in support
    assert "pub const CSIDL_PERSONAL: INT = 0x0005" in support
    assert "pub const SFGAO_HASPROPSHEET: SFGAOF = 0x0000_0040" in support
    assert "pub static INVALID_FILETITLE_CHARACTERSW: [WCHAR; 10]" in support
    assert "static CLSID_MyDocuments: GUID" in support


def test_mydocs_semantics_lower_assert_iid_alias_and_bool_helper():
    source = "HRESULT C::f() { ATLASSERT(x); return S_OK; }"
    rust = """pub unsafe fn C_f_body(this: *mut C, pidl: LPCITEMIDLIST) -> HRESULT {
        ATLASSERT((*this).x);
        let ppv = &mut (*this).inner;
        let hr = Make(IID_PPV_ARG(IShellFolder2, ppv));
        if !_ILIsSpecialFolder(pidl) { return hr; }
        S_OK
    }"""
    normalized = normalize_worker_semantics(source, rust)
    assert "ATLASSERT" not in normalized
    assert "IID_PPV_ARG" not in normalized
    assert "addr_of!(IID_IShellFolder2)" in normalized
    assert "addr_of_mut!((*this).inner).cast()" in normalized
    assert "_ILIsSpecialFolder(pidl) == 0" in normalized


def test_invalid_filename_static_is_passed_as_a_pointer():
    source = "{ return SHStrDupW(INVALID_FILETITLE_CHARACTERSW, out); }"
    rust = "pub unsafe fn f(out: *mut LPWSTR) { SHStrDupW(INVALID_FILETITLE_CHARACTERSW, out); }"
    normalized = normalize_worker_semantics(source, rust)
    assert "SHStrDupW(INVALID_FILETITLE_CHARACTERSW.as_ptr(), out)" in normalized

    lowered = normalize_worker_semantics(
        "HRESULT CFontsFolder::Initialize() { "
        "return SHELL32_CoCreateInitSF(IID_PPV_ARG(IShellFolder2, &m_pisfInner)); }",
        "pub unsafe fn f(this: *mut CFontsFolder) { "
        "SHELL32_CoCreateInitSF(IID_PPV_ARG(IShellFolder2, &mut (*this).m_pisfInner)) }",
    )
    assert "core::ptr::addr_of!(IID_IShellFolder2)" in lowered
    assert "core::ptr::addr_of_mut!((*this).m_pisfInner).cast()" in lowered


def test_worker_normalizes_resolver_constant_qualification():
    normalized = normalize_worker_semantics(
        "HRESULT C::f() { return S_OK; }",
        "pub unsafe fn C_f_body() -> HRESULT { let x = [0u16; crate::driver_ddk::MAX_PATH]; S_OK }",
    )
    assert "[0u16; MAX_PATH]" in normalized


def test_worker_normalizes_packed_pidl_union_cast_and_debug_dump():
    source = "{ pdump(pidl); return (PIDLPrinterStruct *)&pdata->u.cfont; }"
    rust = """pub unsafe fn f(pidl: LPCITEMIDLIST, pdata: *mut PIDLDATA) {
        pdump(pidl);
        if (*pdata).type_ == 0 { let p = &mut (*pdata).u.cfont as *mut PIDLPrinterStruct; }
    }"""
    normalized = normalize_worker_semantics(source, rust)
    assert "pdump" not in normalized
    assert "(*pdata).type_" in normalized
    assert "core::ptr::addr_of_mut!((*pdata).u.cfont).cast::<PIDLPrinterStruct>()" in normalized


def test_worker_normalizes_shell_folder_cpp_implicit_conversions():
    source = "{ *ppidl = 0; return SHELL32_CompareDetails(this, l, a, b); }"
    rust = """pub unsafe fn f(this: *mut CPrinterFolder, ppidl: *mut LPITEMIDLIST) {
        *ppidl = 0;
        if IsEqualIID(riid, &IID_IShellView) { }
        let x = SFV_CREATE { pshf: this, psvOuter: core::ptr::null_mut(),
            psfvcb: core::ptr::null_mut(), cbSize: 0 };
        *pcsFlags = PrinterSFHeader[i].colstate;
        (*psd).fmt = PrinterSFHeader[i].fmt;
        SHSetStrRet(&raw mut (*psd).str, PrinterSFHeader[i].colnameid);
        SHSetStrRet(strRet, (*printer).szName);
        (*psd).str.cStr[0] = 0;
        SHELL32_CompareDetails(this, l, a, b);
    }"""
    normalized = normalize_worker_semantics(source, rust)
    assert "*ppidl = core::ptr::null_mut();" in normalized
    assert "cpp_com_iid_equal(riid" in normalized
    assert "pshf: core::ptr::addr_of_mut!((*this).IShellFolder_iface)" in normalized
    assert ".colstate as SHCOLSTATEF" in normalized
    assert ".fmt as INT" in normalized
    assert "cpp_com_SHSetStrRetResource(&raw mut (*psd).str_" in normalized
    assert "cpp_com_SHSetStrRetW(strRet" in normalized
    assert "(*psd).str_.u.cStr" in normalized
    assert "(*psd).str_.u.cStr[0] = 0i8" in normalized
    assert (
        "SHELL32_CompareDetails(core::ptr::addr_of_mut!((*this).IShellFolder2_iface)" in normalized
    )
    assert audit_worker_body(source, normalized, "f") == []


def test_folder_options_support_owns_full_sdk_sfv_create_layout():
    source = SHELL32 / "CFolderOptions.cpp"
    support = render_cpp_support(source)
    assert "pub struct SFV_CREATE" in support
    assert "pub pshf: *mut IShellFolder" in support
    assert "pub psvOuter: *mut core::ffi::c_void" in support
    assert "pub psfvcb: *mut core::ffi::c_void" in support
    assert "fn SHCreateShellFolderView(create: *const SFV_CREATE" in support


def test_printer_support_closes_sdk_pod_interface_dependencies_without_duplicates():
    source = SHELL32 / "folders" / "CPrinterFolder.cpp"
    support = render_cpp_support(source, {"IID_IUnknown"})
    assert support.count("pub struct SFV_CREATE") == 1
    assert "pub psfvcb: *mut core::ffi::c_void" in support
    assert "static IID_IUnknown" not in support
    assert "pub static PrinterSFHeader: [shvheader; 6]" in support
    assert "colstate: (SHCOLSTATE_TYPE_STR | SHCOLSTATE_ONBYDEFAULT) as WORD" in support
    assert "pub const PrinterSHELLVIEWCOLUMNS: UINT = 6;" in support
    assert "pub const IDS_SHV_COLUMN_NAME: UINT" in support


def test_net_folder_support_closes_string_copy_and_shell_attribute_dependencies():
    support = render_cpp_support(SHELL32 / "folders" / "CNetFolder.cpp")
    assert "fn lstrlenW(text: LPCWSTR) -> INT" in support
    assert 'link_name = "memcpy"' in support
    assert "pub const E_UNEXPECTED: HRESULT" in support
    assert "pub const SFGAO_CANLINK: SFGAOF" in support


def test_drives_support_does_not_redefine_cpp_boolean_literals():
    support = render_cpp_support(SHELL32 / "folders" / "CDrivesFolder.cpp")
    assert "pub const false" not in support
    assert "pub const true" not in support


def test_printer_support_owns_inline_strret_overloads_and_cpp_icon_bridge():
    support = render_cpp_support(SHELL32 / "folders" / "CPrinterFolder.cpp")
    assert "unsafe fn cpp_com_SHSetStrRetW" in support
    assert "unsafe fn cpp_com_SHSetStrRetResource" in support
    assert 'link_name = "CPrintersExtractIconW_RustBridge"' in support
    assert 'link_name = "CPrinterFolder_CompareDetails_RustBridge"' in support
    assert "static mut shell32_hInstance" in support


def test_sfv_create_model_shadow_is_replaced_from_source_authority():
    source = """HRESULT f() {
        CComPtr<IShellFolder> pSF;
        SFV_CREATE create = { sizeof(SFV_CREATE), pSF };
        return SHCreateShellFolderView(&create, out);
    }"""
    rust = """pub unsafe fn f() -> HRESULT {
        #[repr(C)] struct SFV_CREATE { cbSize: u32, pFolder: *mut IShellFolder }
        let mut pSF: *mut IShellFolder = core::ptr::null_mut();
        let create = SFV_CREATE { cbSize: 8, pFolder: pSF };
        SHCreateShellFolderView(&create, out)
    }"""
    normalized = normalize_worker_semantics(source, rust)
    assert "struct SFV_CREATE" not in normalized
    assert "size_of::<SFV_CREATE>() as UINT" in normalized
    assert "pshf: pSF" in normalized
    assert "psvOuter: core::ptr::null_mut()" in normalized
    assert "psfvcb: core::ptr::null_mut()" in normalized


def test_ccomptr_member_copy_gets_scope_guard_and_explicit_release_is_idempotent():
    source = """HRESULT C::SetClient() {
        CComPtr<IMenuPopup> child = m_subMenuChild;
        if (child) child.Release();
        return S_OK;
    }"""
    rust = """pub unsafe fn C_SetClient_body(this: *mut C) -> HRESULT {
        let child: *mut IMenuPopup = (*this).m_subMenuChild;
        if !child.is_null() {
            ((*(*child).lpVtbl).Release)(child as *mut IUnknown);
        }
        S_OK
    }"""
    normalized = normalize_worker_semantics(source, rust)
    assert "let mut child: *mut IMenuPopup = (*this).m_subMenuChild;" in normalized
    assert "((*(*child).lpVtbl).AddRef)(child)" in normalized
    assert "let _child_cpp_com_guard = CppComPtrGuard::new" in normalized
    assert "child = core::ptr::null_mut();" in normalized


def test_ccomptr_self_copy_uses_deterministic_iunknown_lifetime_guard():
    source = "{ CComPtr<CMenuBand> safeThis = this; return S_OK; }"
    rust = """pub unsafe fn CMenuBand_f_body(this: *mut CMenuBand) -> HRESULT {
        let unknown_ptr = this as *mut IUnknown;
        ((*(*unknown_ptr).lpVtbl).AddRef)(unknown_ptr);
        struct SafeThisGuard(*mut IUnknown);
        impl Drop for SafeThisGuard { fn drop(&mut self) { unsafe {
            ((*(*self.0).lpVtbl).Release)(self.0); } } }
        let _guard = SafeThisGuard(unknown_ptr);
        S_OK
    }"""
    normalized = normalize_worker_semantics(source, rust)
    assert "struct SafeThisGuard" not in normalized
    assert "let mut safeThis: *mut IUnknown = this.cast();" in normalized
    assert "let _safeThis_cpp_com_guard = CppComPtrGuard::new" in normalized
    assert audit_worker_body(source, normalized, "CMenuBand_f_body") == []
    assert normalize_worker_semantics(source, normalized) == normalized


def test_model_comdrop_wrapper_is_lowered_to_scaffold_owned_guards():
    source = "{ CComPtr<IBandSite> pBandSite; return S_OK; }"
    rust = """pub unsafe fn C_f_body() -> HRESULT {
        struct ComDrop<T> { ptr: *mut T }
        impl<T> Drop for ComDrop<T> { fn drop(&mut self) { unsafe {
            if !self.ptr.is_null() { Release(self.ptr); }
        } } }
        let mut pBandSite = ComDrop::<IBandSite> { ptr: core::ptr::null_mut() };
        Use(&mut pBandSite.ptr); S_OK
    }"""
    normalized = normalize_worker_semantics(source, rust)
    assert "struct ComDrop" not in normalized
    assert "let mut pBandSite: *mut IBandSite = core::ptr::null_mut();" in normalized
    assert "let _pBandSite_cpp_com_guard = CppComPtrGuard::new" in normalized
    assert "Use(&mut pBandSite)" in normalized


def test_worker_audit_does_not_treat_macros_or_cast_types_as_constants():
    c_body = (
        "{ if (FAILED(hr)) return (HRESULT)(UINT)(SIZE_T)value; "
        "pointer = (CLSID *)&CLSID_Value; return S_OK; }"
    )
    rust = (
        "pub unsafe fn C_f_body() -> HRESULT { if hr < 0 { return value as HRESULT; } "
        "pointer = &CLSID_Value as *const CLSID as *mut CLSID; S_OK }"
    )
    assert audit_worker_body(c_body, rust, "C_f_body") == []

    c_body = "{ if (const CLSID* pClsid = IsRegItem(pidl)) return S_OK; return S_FALSE; }"
    rust = (
        "pub unsafe fn C_f_body() -> HRESULT { "
        "if !IsRegItem(pidl).is_null() { S_OK } else { S_FALSE } }"
    )
    assert audit_worker_body(c_body, rust, "C_f_body") == []


def test_worker_audit_ignores_constants_used_only_by_debug_macros():
    c_body = '{ TRACE("%d", value == STRRET_CSTR); return S_OK; }'
    rust = "pub unsafe fn C_f_body() -> HRESULT { S_OK }"
    assert audit_worker_body(c_body, rust, "C_f_body") == []


def test_printer_enum_template_is_lowered_to_explicit_cpp_bridge():
    source = """HRESULT CPrinterFolder::EnumObjects(
        HWND hwndOwner, DWORD dwFlags, IEnumIDList **ppEnumIDList) {
        return ShellObjectCreatorInit<CPrintersEnum>(
            hwndOwner, dwFlags, IID_PPV_ARG(IEnumIDList, ppEnumIDList));
    }"""
    signature = (
        "pub unsafe fn CPrinterFolder_EnumObjects_body(this: *mut CPrinterFolder, "
        "hwndOwner: HWND, dwFlags: DWORD, ppEnumIDList: *mut *mut IEnumIDList) -> HRESULT"
    )
    lowered = lower_trivial_worker(source, signature)
    assert lowered is not None
    assert "CPrintersEnum_CreateInstance(hwndOwner, dwFlags, ppEnumIDList)" in lowered


def test_worker_audit_rejects_model_owned_symbolic_constant_values():
    c_body = "{ return HRESULT_FROM_WIN32(ERROR_NOT_FOUND); }"
    rust = (
        "pub unsafe fn C_f_body() -> HRESULT { "
        "const ERROR_NOT_FOUND: DWORD = 2; HRESULT_FROM_WIN32(ERROR_NOT_FOUND as LONG) }"
    )
    findings = audit_worker_body(c_body, rust, "C_f_body")
    assert any("redefined source symbolic constant" in finding for finding in findings)


def test_worker_audit_does_not_treat_const_pointer_type_as_constant_definition():
    source = "HRESULT f() { CHAR *value = GetValue(); return value ? S_OK : E_FAIL; }"
    rust = (
        "pub unsafe fn C_f_body() -> HRESULT { let value: *const CHAR = GetValue(); "
        "if value.is_null() { E_FAIL } else { S_OK } }"
    )
    findings = audit_worker_body(source, rust, "C_f_body")
    assert not any("redefined source symbolic constant" in finding for finding in findings)


def test_worker_audit_accepts_named_rust_form_of_source_anonymous_local_pod():
    source = """HRESULT f() {
        struct { LPCWSTR name; DWORD flags; BOOL enabled; } Rows[] = {
            { L"one", 1, TRUE }, { NULL, 0, FALSE }
        };
        return Use(Rows);
    }"""
    rust = """pub unsafe fn C_f_body() -> HRESULT {
        struct Row { name: LPCWSTR, flags: DWORD, enabled: BOOL }
        let Rows = [
            Row { name: ONE.as_ptr(), flags: 1, enabled: TRUE },
            Row { name: core::ptr::null(), flags: 0, enabled: FALSE },
        ];
        Use(Rows)
    }"""
    assert audit_worker_body(source, rust, "C_f_body") == []


def test_worker_audit_rejects_wrong_named_form_of_source_anonymous_local_pod():
    source = """HRESULT f() {
        struct { LPCWSTR name; DWORD flags; BOOL enabled; } Rows[] = {};
        return Use(Rows);
    }"""
    rust = """pub unsafe fn C_f_body() -> HRESULT {
        #[repr(C)] struct Row { name: LPCWSTR, enabled: BOOL }
        Use(core::ptr::null_mut())
    }"""
    findings = audit_worker_body(source, rust, "C_f_body")
    assert any("forbidden ABI type declaration: Row" in finding for finding in findings)


def test_worker_audit_checks_the_deterministic_signature():
    expected = worker_signature("C_f", "C", [("POINT *", "ppt"), ("DWORD", "effect")], "HRESULT")
    good = (
        "pub unsafe fn C_f_body(this: *mut C, ppt: *mut POINT, effect: DWORD) "
        "-> HRESULT { E_NOTIMPL }"
    )
    assert audit_worker_body("{ return E_NOTIMPL; }", good, "C_f_body", expected) == []
    bad = "pub unsafe fn C_f_body(this: *mut C, ppt: POINT, effect: DWORD) -> HRESULT { E_NOTIMPL }"
    assert any(
        "signature differs" in finding
        for finding in audit_worker_body("{ return E_NOTIMPL; }", bad, "C_f_body", expected)
    )

    mutable = (
        "pub unsafe fn C_f_body(this: *mut C, ppt: *mut POINT, mut effect: DWORD) "
        "-> HRESULT { effect += 1; E_NOTIMPL }"
    )
    normalized = normalize_worker_body(mutable, expected)
    assert "mut effect: DWORD" in normalized
    assert audit_worker_body("{ return E_NOTIMPL; }", normalized, "C_f_body", expected) == []


def test_worker_normalizer_does_not_parse_rust_path_colons_as_parameters():
    expected = (
        "pub unsafe fn C_f_body(this: *mut C, out: *mut core::ffi::c_void, "
        "size: *mut DWORD) -> HRESULT"
    )
    model = (
        "pub unsafe fn C_f_body(this: *mut C, out: *mut core::ffi::c_void, "
        "size: *mut u32) -> i32 { S_OK }"
    )
    assert normalize_worker_body(model, expected).startswith(expected + " {")


def test_worker_normalizer_discards_model_comments_inside_generated_signature():
    expected = "pub unsafe fn C_f_body(this: *mut C, value: DWORD, flags: DWORD) -> HRESULT"
    model = """pub unsafe fn C_f_body(
        this: *mut C,
        value: DWORD, // retained for ABI
        flags: DWORD, /* generated slot */
    ) -> HRESULT { S_OK }"""
    normalized = normalize_worker_body(model, expected)
    assert normalized.startswith(expected + " {")
    assert "retained for ABI" not in normalized


def test_island_integer_aliases_requires_every_arch_to_agree(tmp_path):
    """An SDK enum is int-sized only if EVERY arch island says so, identically."""

    def island(text: str, name: str) -> Path:
        path = tmp_path / name
        path.write_text(text, encoding="utf-8")
        return path

    agree = [
        island(
            "pub type __WIDL_generated_00000001 = ::core::ffi::c_int;\n"
            "pub use self::__WIDL_generated_00000001 as FOLDERVIEWMODE;\n"
            "pub type SIZE_T = u32;\n",
            f"island_{arch}.rs",
        )
        for arch in ("i386", "amd64")
    ]
    # Second arch widens SIZE_T, exactly as a real 64-bit island does.
    agree[1].write_text(
        "pub type __WIDL_generated_00000001 = ::core::ffi::c_int;\n"
        "pub use self::__WIDL_generated_00000001 as FOLDERVIEWMODE;\n"
        "pub type SIZE_T = u64;\n",
        encoding="utf-8",
    )
    proven = island_integer_aliases(agree)
    assert "FOLDERVIEWMODE" in proven, "re-export chained onto an int alias is int-sized"
    assert "SIZE_T" not in proven, "a width that changes per arch must never be assumed"


def test_island_integer_aliases_is_empty_when_an_island_is_missing(tmp_path):
    """No islands, no proof -- the caller must stay a loud preflight failure."""
    assert island_integer_aliases([tmp_path / "island_nope.rs"]) == frozenset()
    assert island_integer_aliases([]) == frozenset()


def test_sdk_enum_state_field_is_accepted_only_with_island_proof():
    """FOLDERVIEWMODE m_FVM: unprovable from the header alone, provable from the islands."""
    source = SHELL32 / "CDefViewUtil.cpp"
    if not source.is_file():
        pytest.skip("CDefViewUtil.cpp has been retired from the tree")
    with pytest.raises(CppComError, match="not proven C scalar/pointer state"):
        analyze_class(
            source, "CShellFolderViewCBWrapper", object_map_source=SHELL32 / "shell32.cpp"
        ).require_supported(allow_internal=True)
    proved = analyze_class(
        source,
        "CShellFolderViewCBWrapper",
        object_map_source=SHELL32 / "shell32.cpp",
        island_files=[SHELL32_ISLAND_FILES[a] for a in ("i386", "amd64", "arm64")],
    )
    proved.require_supported(allow_internal=True)
    fvm = next(f for f in proved.plain_state_fields() if f.name == "m_FVM")
    assert fvm.rust_type == "FOLDERVIEWMODE", "must reference the island alias, not a local copy"


def test_island_integer_aliases_follows_a_chain_into_the_sys_crate(tmp_path):
    """`SHCONTF = DWORD` is only provable if the chain can leave the island.

    bindgen emits a module's OWN aliases into the island and leaves the base types
    (`DWORD`, `UINT`, ...) to the `-sys` glob at its head. A resolver that stops at the
    island edge proves neither, so an ordinary DWORD state field is rejected and a class
    that is perfectly translatable stays UNSUPPORTED.
    """

    def workspace(arch: str, dword_rhs: str, size_t_rhs: str) -> Path:
        island_dir = tmp_path / "shell32-rust" / "src"
        island_dir.mkdir(parents=True, exist_ok=True)
        sys_dir = tmp_path / "ntdll-sys" / "src"
        sys_dir.mkdir(parents=True, exist_ok=True)
        (sys_dir / f"bindgen_{arch}.rs").write_text(
            f"pub type DWORD = {dword_rhs};\npub type SIZE_T = {size_t_rhs};\n",
            encoding="utf-8",
        )
        island = island_dir / f"island_{arch}.rs"
        island.write_text(
            "pub use ntdll_sys::nt::*;\npub type SHCONTF = DWORD;\npub type PSIZE_T = SIZE_T;\n",
            encoding="utf-8",
        )
        return island

    islands = [
        workspace("i386", "::core::ffi::c_ulong", "u32"),
        workspace("amd64", "::core::ffi::c_ulong", "u64"),
    ]
    proven = island_integer_aliases(islands)
    assert "SHCONTF" in proven, "a two-hop chain through the -sys crate must resolve"
    assert "DWORD" in proven, "the -sys base type itself is proven too"
    # The agreement check still does the safety work ACROSS the crate boundary: a chain that
    # ends in a different primitive per arch is rejected however many hops it took to get there.
    assert "PSIZE_T" not in proven
    assert "SIZE_T" not in proven


def test_island_integer_aliases_terminates_on_a_cyclic_alias(tmp_path):
    """A malformed island that aliases in a loop must return, not hang."""
    islands = []
    for arch in ("i386", "amd64"):
        path = tmp_path / f"island_{arch}.rs"
        path.write_text("pub type A = B;\npub type B = A;\npub type OK = u32;\n", encoding="utf-8")
        islands.append(path)
    proven = island_integer_aliases(islands)
    assert proven == frozenset({"OK"})


def test_island_struct_types_follows_an_alias_and_needs_every_arch(tmp_path):
    """`FORMATETC` is `pub type FORMATETC = tagFORMATETC;` over `pub struct tagFORMATETC`.

    Matching only `pub struct` therefore proves almost nothing: the C spelling a state
    field uses is nearly always the alias, not the tag. And the proof must hold on every
    arch for the same reason widths must — a name one island happens to carry is not a
    fact about the type.
    """
    islands = []
    for arch, extra in (("i386", ""), ("amd64", "pub struct OnlyHere {}\n")):
        path = tmp_path / f"island_{arch}.rs"
        path.write_text(
            "pub struct tagFORMATETC {}\npub type FORMATETC = tagFORMATETC;\n"
            "pub type NotAStruct = u32;\n" + extra,
            encoding="utf-8",
        )
        islands.append(path)
    proven = island_struct_types(islands)
    assert "tagFORMATETC" in proven
    assert "FORMATETC" in proven, "the alias must resolve onto the struct it names"
    assert "NotAStruct" not in proven
    assert "OnlyHere" not in proven, "one arch is not a proof"


def test_atl_container_state_field_keeps_its_type_and_its_drop():
    """`CSimpleArray<FORMATETC> m_Formats` — the field that had CIDLDataObj UNSUPPORTED.

    A container cannot be flattened to a pointer the way `CComPtr` is, because its
    destructor walks storage the scaffold does not own. So the field keeps the container
    type and `atl.rs`'s `Drop` restores the C++ member destructor.
    """
    field = _parse_plain_state_field(
        "CSimpleArray<FORMATETC> m_Formats",
        island_struct_types={"FORMATETC"},
    )
    assert field.raii_container == "CSimpleArray<FORMATETC>"
    assert field.rust_type == "CSimpleArray<FORMATETC>"
    assert field.init_expression == "CSimpleArray::new()", "zeroed() is not a valid container"


def test_atl_container_over_an_unproven_element_is_still_refused():
    """An element nobody proved a layout for stays a refusal.

    The container's `Drop` runs one `drop_in_place` per element, so the element's own
    destructor semantics have to be known before a single byte is generated. An element
    that is neither an island struct nor a source-discovered POD is exactly the case where
    they are not, and guessing would ship either a leak or a wrong layout.
    """
    with pytest.raises(CppComError, match="container element type is not a proven layout"):
        _parse_plain_state_field(
            "CAtlList<StaticShellEntry> m_StaticEntries",
            island_struct_types={"FORMATETC"},
        )


def test_container_element_pod_keeps_its_raii_members_owning(tmp_path: Path):
    """`CAtlList<DynamicShellEntry>` — CDefaultContextMenu's shape, and the leak it hides.

    Discovering the element is only half of it. The element POD holds a
    `CComPtr<IContextMenu>`, and the COM *owner's* field spelling for that is a raw
    `*mut IContextMenu` — correct there, because the owner's generated `Release` does the
    releasing explicitly. A POD has no generated code at all, so the same spelling inside
    it means the container's per-element `drop_in_place` drops a raw pointer: one leaked
    interface reference per entry, with nothing red anywhere. The POD therefore has to
    keep the owning types, and Rust's field-drop order is then the C++ implicit dtor.
    """
    source = tmp_path / "menu.cpp"
    source.write_text(
        """
typedef struct _DynamicShellEntry_
{
    UINT iIdCmdFirst;
    CComPtr<IContextMenu> pCM;
} DynamicShellEntry, *PDynamicShellEntry;

typedef struct _StaticShellEntry_
{
    CStringW Verb;
    HKEY hkClass;
} StaticShellEntry, *PStaticShellEntry;

class CMenu : public CComCoClass<CMenu>,
              public CComObjectRootEx<CComMultiThreadModelNoCS>, public IFoo {
  CAtlList<DynamicShellEntry> m_DynamicEntries;
  CAtlList<StaticShellEntry> m_StaticEntries;
  DECLARE_NOT_AGGREGATABLE(CMenu)
  BEGIN_COM_MAP(CMenu) COM_INTERFACE_ENTRY_IID(IID_IFoo, IFoo) END_COM_MAP()
};
"""
    )
    object_map = tmp_path / "shell32.cpp"
    object_map.write_text("OBJECT_ENTRY(CLSID_Menu, CMenu)\n")
    cls = analyze_class(source, "CMenu", object_map_source=object_map)
    cls.require_supported()

    # The element hides in the template argument; the plain `Type name` scan cannot see it.
    assert sorted(pod.name for pod in cls.pod_state_types) == [
        "DynamicShellEntry",
        "StaticShellEntry",
    ]
    assert pods_with_drop(cls.pod_state_types) == {"DynamicShellEntry", "StaticShellEntry"}

    rendered = render_class_scaffold(cls, [InterfaceSpec("IFoo", (), "test.h")])
    assert "pub pCM: CComPtr<IContextMenu>," in rendered
    assert "pub pCM: *mut IContextMenu," not in rendered, "a raw pointer here leaks per entry"
    assert "pub Verb: CStringW," in rendered
    # The containers themselves still drop in Release, after the explicit dtor body.
    assert "core::ptr::drop_in_place(&mut (*this).m_DynamicEntries);" in rendered


def test_by_value_pod_with_raii_members_is_dropped_by_release(tmp_path: Path):
    """The same leak one level up: a POD held BY VALUE by the COM owner.

    Release only ever dropped fields it could see were RAII from their own declaration.
    A `DynamicShellEntry m_entry;` declaration looks like any other POD, so the member's
    destructor went missing exactly as it did inside the container.
    """
    source = tmp_path / "owner.cpp"
    source.write_text(
        """
typedef struct _Entry_ { UINT id; CComPtr<IContextMenu> pCM; } Entry, *PEntry;
typedef struct _Wrapper_ { Entry inner; DWORD flags; } Wrapper, *PWrapper;

class COwner : public CComCoClass<COwner>,
               public CComObjectRootEx<CComMultiThreadModelNoCS>, public IFoo {
  Wrapper m_wrapper;
  Entry *m_borrowed;
  DECLARE_NOT_AGGREGATABLE(COwner)
  BEGIN_COM_MAP(COwner) COM_INTERFACE_ENTRY_IID(IID_IFoo, IFoo) END_COM_MAP()
};
"""
    )
    object_map = tmp_path / "shell32.cpp"
    object_map.write_text("OBJECT_ENTRY(CLSID_Owner, COwner)\n")
    cls = analyze_class(source, "COwner", object_map_source=object_map)
    cls.require_supported()

    # Transitive: Wrapper has no RAII field of its own, it just contains one that does.
    assert pods_with_drop(cls.pod_state_types) == {"Entry", "Wrapper"}

    rendered = render_class_scaffold(cls, [InterfaceSpec("IFoo", (), "test.h")])
    assert "core::ptr::drop_in_place(&mut (*this).m_wrapper);" in rendered
    assert "drop_in_place(&mut (*this).m_borrowed)" not in rendered, "a POINTER owns nothing"


def test_free_function_support_pod_keeps_its_raii_members_owning(tmp_path: Path):
    """The same leak on the free-function path, which is most of shell32.

    `BrItemData` (brfolder.cpp) is the precedent, and it is how the bug was found: the
    SHIPPED brfolder.rs spells these fields `CComPtr` / `CComHeapPtr`, while the generator
    emitted raw pointers — right next to the `CComHeapPtr` support block it was not using.
    A regeneration would have silently downgraded a correct file into a leaking one.
    """
    source = tmp_path / "brlike.cpp"
    source.write_text(
        """
struct BrItemData
{
    CComPtr<IShellFolder> lpsfParent;
    PCIDLIST_RELATIVE pidlChild;
    CComHeapPtr<ITEMIDLIST_ABSOLUTE> pidlFull;
};
static void use(BrItemData *p) { (void)p; }
"""
    )
    rendered = render_cpp_support(source, {"BrItemData", "lpsfParent", "pidlChild", "pidlFull"})
    assert "pub lpsfParent: CComPtr<IShellFolder>," in rendered
    assert "pub pidlFull: CComHeapPtr<ITEMIDLIST_ABSOLUTE>," in rendered
    assert "pub lpsfParent: *mut IShellFolder," not in rendered
    assert "pub pidlChild: PCIDLIST_RELATIVE," in rendered, "a plain C field is unchanged"


def test_packed_pod_with_a_destructor_member_fails_before_payment(tmp_path: Path):
    """`#[repr(C, packed)]` plus drop glue is not expressible: rustc cannot reference a
    misaligned place to drop it. Catch it in preflight, naming the C++ construct, rather
    than paying for a translation and then reading E0793."""
    source = tmp_path / "packed.cpp"
    source.write_text(
        """
#include <pshpack1.h>
typedef struct _Tight_ { BYTE tag; CStringW name; } Tight, *PTight;
#include <poppack.h>

class CPacked : public CComCoClass<CPacked>,
                public CComObjectRootEx<CComMultiThreadModelNoCS>, public IFoo {
  Tight m_tight;
  DECLARE_NOT_AGGREGATABLE(CPacked)
  BEGIN_COM_MAP(CPacked) COM_INTERFACE_ENTRY_IID(IID_IFoo, IFoo) END_COM_MAP()
};
"""
    )
    object_map = tmp_path / "shell32.cpp"
    object_map.write_text("OBJECT_ENTRY(CLSID_Packed, CPacked)\n")
    cls = analyze_class(source, "CPacked", object_map_source=object_map)
    with pytest.raises(CppComError, match="Rust cannot drop a packed field"):
        cls.require_supported()


def test_unlisted_cxx_container_names_the_missing_lowering():
    """A refusal that names the mechanism, not just the declaration.

    `CAtlMap<K, V> m_x` is not a mystery — it is a container `atl.rs` has not lowered
    yet, and saying so is what stops the next reader concluding the whole TU is
    untranslatable.
    """
    with pytest.raises(CppComError, match="no Rust lowering for `CAtlArray`"):
        _parse_plain_state_field("CAtlArray<UINT> m_ids")


def test_array_of_owned_heap_pointers_frees_every_element():
    """`CComHeapPtr<ITEMIDLIST> m_pidls[2]` — CRecycleBinFolderViewCB's shape.

    C++ destroys each element, so the extent has to survive into the Rust type for the
    generated Release to have something to iterate.
    """
    field = _parse_plain_state_field("CComHeapPtr<ITEMIDLIST> m_pidls[2]")
    assert field.owned_heap_ptr
    assert field.array_len == 2
    assert field.rust_type == "[*mut ITEMIDLIST; 2]"


def test_iid_null_ppv_arg_expands_to_three_arguments():
    """`IID_NULL_PPV_ARG` is a *different* macro, not a spelling of `IID_PPV_ARG`.

    It does not contain the substring `IID_PPV_ARG`, so every rule keyed on
    `\\bIID_PPV_ARG` was structurally incapable of matching it: the macro reached
    CDefaultContextMenu as an unexpanded call. The middle `NULL` is
    `GetUIObjectOf`'s reserved `prgfInOut`, and dropping it does not fail to
    compile — it shifts `ppv` into the `prgfInOut` slot, which is far worse.
    """
    source = (
        "m_psf->GetUIObjectOf(pdcm->hwnd, m_cidl, m_apidl, "
        "IID_NULL_PPV_ARG(IDataObject, &m_pDataObj));"
    )
    for shape in (
        "IID_NULL_PPV_ARG(IDataObject, core::ptr::addr_of_mut!((*this).m_pDataObj))",
        "IID_NULL_PPV_ARG(IDataObject, &mut pdo)",
        "IID_NULL_PPV_ARG(IDataObject, pdo as *mut *mut IDataObject)",
    ):
        expanded = normalize_cpp_idioms(source, shape)
        assert "IID_NULL_PPV_ARG(" not in expanded, "a macro is not a symbol; it cannot link"
        assert "core::ptr::addr_of!(IID_IDataObject), core::ptr::null_mut(), " in expanded
        assert expanded.rstrip().endswith(".cast()")


def test_the_plain_ppv_macro_gains_no_null_argument():
    """The negative control: generalizing the family must not give `IID_PPV_ARG` a
    third argument, which would shift every argument after it in ~200 call sites."""
    expanded = normalize_cpp_idioms("", "IID_PPV_ARG(IShellFolder, &mut sp)")
    assert expanded == "core::ptr::addr_of!(IID_IShellFolder), core::ptr::addr_of_mut!(sp).cast()"
    assert "null_mut" not in expanded


def test_a_ppv_macro_never_becomes_a_phantom_guid_extern(tmp_path: Path):
    """A GUID constant is referenced, never CALLED.

    The ppv macros match the `IID_*` spelling, so the extern harvest declared
    `static IID_NULL_PPV_ARG: GUID;` — a symbol that exists in no library. It was
    excluded by a hand-maintained denylist listing `IID_PPV_ARG` alone, so the
    first new family member shipped the phantom. Deciding it positionally is what
    retires the denylist.
    """
    source = tmp_path / "phantom.cpp"
    source.write_text(
        """
static HRESULT get(IShellFolder *psf, HWND hwnd, PCUITEMID_CHILD pidl)
{
    CComPtr<IDataObject> pdo;
    return psf->GetUIObjectOf(hwnd, 1, &pidl, IID_NULL_PPV_ARG(IDataObject, &pdo));
}
"""
    )
    rendered = render_cpp_support(source)
    assert "IID_NULL_PPV_ARG: GUID" not in rendered
    assert "IID_PPV_ARG: GUID" not in rendered
    assert "static IID_IDataObject: GUID;" in rendered, (
        "the interface IID the expansion references still needs its declaration"
    )


def test_a_ppv_macro_name_in_the_riid_slot_is_recovered_from_the_source():
    """The fourth shape: right arity, wrong name.

    CDefaultContextMenu::DoCreateLink came back with `&IID_PPV_ARG as *const _ as
    REFIID` — the model split `IID_PPV_ARG(IDropTarget, &pDT)` into the correct two
    arguments but put the MACRO's name in the riid slot. The ppv slot was already
    right, so no expansion rule fired; the name simply exists in no library.
    """
    source = "hr = m_psf->CreateViewObject(NULL, IID_PPV_ARG(IDropTarget, &pDT));"
    rust = (
        "&IID_PPV_ARG as *const _ as REFIID, "
        "&mut pDT as *mut *mut IDropTarget as *mut *mut core::ffi::c_void"
    )
    assert normalize_cpp_idioms(source, rust).startswith("&IID_IDropTarget as *const _ as REFIID")


def test_two_candidate_interfaces_leave_the_riid_slot_alone():
    """Recovery is only sound when the body names ONE interface. Two is a guess, and a
    guessed IID is a QueryInterface for the wrong thing — so leave it uncompilable."""
    source = "a(IID_PPV_ARG(IDropTarget, &x)); b(IID_PPV_ARG(IShellFolder, &y));"
    assert normalize_cpp_idioms(source, "&IID_PPV_ARG as REFIID") == "&IID_PPV_ARG as REFIID"


def test_plain_helper_class_passes_preflight_without_com_surface():
    """A file-local C++ helper is SIMPLER than the supported case, not less proven.

    `--free-functions` skips every class method and names `--cpp-com --class` as the route,
    but the COM preflight then refuses a class with no BEGIN_COM_MAP / aggregation policy /
    OBJECT_ENTRY — so COpenWithList and COpenWithDialog had no path through the pipeline at
    all, and COpenWithMenu.cpp could not be flipped. The three requirements are vacuous for a
    class with no vtable, no QI and no activation.
    """
    source = SHELL32 / "COpenWithMenu.cpp"
    cls = analyze_class(source, "COpenWithList", object_map_source=SHELL32 / "shell32.cpp")
    assert cls.com_map == []
    assert cls.object_map_clsid is None
    assert cls.interface_names() == []
    cls.require_supported(allow_plain=True)
    with pytest.raises(CppComError, match="no BEGIN_COM_MAP"):
        cls.require_supported()


def test_plain_refuses_a_class_that_is_actually_com():
    """Plainness is asserted by the caller, so a wrong assertion must be loud.

    Silently honouring --plain on a COM class would emit it with no vtable and no activation
    — an object that compiles, links, and fails the moment anything QIs it.
    """
    source = SHELL32 / "COpenWithMenu.cpp"
    cls = analyze_class(source, "COpenWithMenu", object_map_source=SHELL32 / "shell32.cpp")
    assert cls.com_map
    with pytest.raises(CppComError, match="it is a COM class"):
        cls.require_supported(allow_plain=True)


def test_hicon_truthiness_negation_wraps_the_whole_place_expression():
    """The `!` goes in front of the receiver chain, not the field.

    `hIcon != 0` is pointer truthiness, but the rewrite matched the bare NAME, so a correct
    `(*p).hIcon != 0` from the model came back as `(*p).!hIcon.is_null()` — a parse error.
    It failed loud rather than shipping, but it is a deterministic pass corrupting good
    input: any model that lowers the test this way hits it.
    """
    from rustconv.cpp_com import normalize_worker_semantics

    for rust, expected in (
        ("if hIcon != 0 { }", "if !hIcon.is_null() { }"),
        ("if (*p).hIcon != 0 { }", "if !(*p).hIcon.is_null() { }"),
        ("if fi.hIcon != 0 { }", "if !fi.hIcon.is_null() { }"),
        ("if (*this).m_pApp.hIcon != 0 { }", "if !(*this).m_pApp.hIcon.is_null() { }"),
    ):
        assert normalize_worker_semantics("", rust) == expected

    # `m_hIcon` has no word boundary before `hIcon`, so the bare-name rule must not reach
    # inside it — its own receiver-qualified rule runs first.
    assert "m_!hIcon" not in normalize_worker_semantics("", "if (*this).m_hIcon != 0 { }")


def test_declaration_only_destructor_does_not_eat_the_first_data_member():
    """`~COpenWithList();` used to strand its `~` onto the member below it.

    The statement became `~ SApp *m_pApp`, and the "starts with ~" filter dropped the whole
    thing — so every class with an OUT-OF-LINE destructor silently lost its first field. ATL
    classes write `~CFoo() {}` inline, where the block remover leaves a clean `;`, which is
    why this survived until the first plain helper class.
    """
    source = SHELL32 / "COpenWithMenu.cpp"
    cls = analyze_class(source, "COpenWithList", object_map_source=SHELL32 / "shell32.cpp")
    # Exact C++ declaration order, first member included.
    assert cls.state_fields == [
        "SApp *m_pApp",
        "UINT m_cApp",
        "UINT m_cRecommended",
        "BOOL m_bNoOpen",
    ]


def test_calling_convention_and_qualified_return_types_leave_no_debris():
    """Two more strandings that each swallowed the member behind them.

    `static INT_PTR CALLBACK DialogProc(...)` matched only from `CALLBACK`, leaving
    `static INT_PTR`; `COpenWithList::SApp *GetCurrentApp()` matched only from `SApp`,
    leaving `COpenWithList::`. Both glued onto `const OPENASINFO *m_pInfo`.
    """
    source = SHELL32 / "COpenWithMenu.cpp"
    cls = analyze_class(source, "COpenWithDialog", object_map_source=SHELL32 / "shell32.cpp")
    assert cls.state_fields[0] == "const OPENASINFO *m_pInfo"
    assert len(cls.state_fields) == 10
    assert not any("static" in f or "::" in f for f in cls.state_fields)


def test_debris_in_a_field_is_a_hard_failure_not_a_guess():
    """A mis-parsed member silently changes the object layout, so it must never ship."""
    from rustconv.cpp_com import _state_fields

    with pytest.raises(CppComError, match="static"):
        _state_fields("class X { static INT_PTR static INT_PTR const FOO *m_pInfo; };")
