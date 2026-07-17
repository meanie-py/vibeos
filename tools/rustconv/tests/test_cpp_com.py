"""Deterministic C++/ATL COM-class pipeline."""

from pathlib import Path

import pytest

from rustconv.call_seal import seal_calls, unseal_calls
from rustconv.callgraph import FunctionDep, extract_call_graph
from rustconv.config import REACTOS_ROOT, TARGETS, detect_target
from rustconv.cpp_com import (
    CppComError,
    InterfaceMethod,
    InterfaceSpec,
    analyze_class,
    apply_interface_method_signatures,
    audit_worker_body,
    c_type_to_rust,
    discover_widl_header,
    lower_trivial_worker,
    normalize_worker_body,
    normalize_worker_semantics,
    parse_widl_interface,
    prepare_helper_closure,
    render_class_scaffold,
    render_cpp_support,
    render_interface,
    render_object_map_adapter,
    render_wide_literal_statics,
    require_supported_worker_raii,
    strip_debug_statements,
    worker_signature,
)
from rustconv.function_prompt import _format_callee_context, load_function_system_prompt
from rustconv.resolver import ResolveResult
from rustconv.scanner import scan_file
from rustconv.stitcher import _generate_imports_from_resolve

SHELL32 = REACTOS_ROOT / "dll" / "win32" / "shell32"

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


@pytest.mark.parametrize(
    ("c_type", "rust_type"),
    [
        ("void **", "*mut *mut core::ffi::c_void"),
        ("const IID *", "*const IID"),
        ("IDataObject *", "*mut IDataObject"),
        ("CStringW &", "&mut CStringW"),
        ("const CStringW &", "&CStringW"),
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


def test_unmapped_cpp_raii_state_still_fails_before_scaffold_generation(tmp_path: Path):
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
    with pytest.raises(CppComError, match="not proven C scalar/pointer state"):
        cls.require_supported()


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


def test_helper_closure_keeps_same_file_external_functions_at_abi_boundary():
    scan = scan_file(SHELL32 / "CShellItem.cpp")
    plan = prepare_helper_closure(extract_call_graph(scan), "CShellItem")
    assert "CShellItem_GetParent" in {dep.name for dep in plan}
    assert "SHCreateShellItem" not in {dep.name for dep in plan}


def test_helper_worker_signature_has_no_invented_this_parameter():
    assert worker_signature(
        "GetParsingName", None, [("PCIDLIST_ABSOLUTE", "pidl"), ("PWSTR *", "Name")], "HRESULT"
    ) == ("pub unsafe fn GetParsingName_body(pidl: PCIDLIST_ABSOLUTE, Name: *mut PWSTR) -> HRESULT")


def test_interface_vtable_repairs_scanner_lost_cpp_return_types():
    scan = scan_file(SHELL32 / "folders" / "CControlPanelFolder.cpp")
    deps = extract_call_graph(scan)
    assert next(d for d in deps if d.name == "COpenControlPanel_Open").return_type == ""
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


def test_same_arity_helper_overload_fails_before_payment():
    roots = [
        FunctionDep(
            name="C_f",
            cpp_class="C",
            cpp_method="f",
            body_text="HRESULT C::f() { return H(1); }",
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
    with pytest.raises(CppComError, match="local overload H/1 is ambiguous"):
        prepare_helper_closure(roots, "C")


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


def test_custom_base_class_fails_before_inherited_state_is_lost():
    cls = analyze_class(
        SHELL32 / "CCopyMoveToMenu.cpp",
        "CMoveToMenu",
        object_map_source=SHELL32 / "shell32.cpp",
    )
    with pytest.raises(CppComError, match="custom/inherited base class CCopyMoveToMenu"):
        cls.require_supported()


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
    assert "folder = core::ptr::null_mut();" in normalized


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


def test_folder_options_support_owns_full_sdk_sfv_create_layout():
    source = SHELL32 / "CFolderOptions.cpp"
    support = render_cpp_support(source)
    assert "pub struct SFV_CREATE" in support
    assert "pub pshf: *mut IShellFolder" in support
    assert "pub psvOuter: *mut core::ffi::c_void" in support
    assert "pub psfvcb: *mut core::ffi::c_void" in support
    assert "fn SHCreateShellFolderView(create: *const SFV_CREATE" in support


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


def test_worker_audit_rejects_model_owned_symbolic_constant_values():
    c_body = "{ return HRESULT_FROM_WIN32(ERROR_NOT_FOUND); }"
    rust = (
        "pub unsafe fn C_f_body() -> HRESULT { "
        "const ERROR_NOT_FOUND: DWORD = 2; HRESULT_FROM_WIN32(ERROR_NOT_FOUND as LONG) }"
    )
    findings = audit_worker_body(c_body, rust, "C_f_body")
    assert any("redefined source symbolic constant" in finding for finding in findings)


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
