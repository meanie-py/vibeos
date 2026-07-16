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


def test_internal_com_object_fails_until_construction_is_mapped():
    cls = analyze_class(
        SHELL32 / "folders" / "CControlPanelFolder.cpp",
        "CControlPanelEnum",
        object_map_source=SHELL32 / "shell32.cpp",
    )
    with pytest.raises(CppComError, match="internal COM-object construction is not mapped"):
        cls.require_supported()


def test_custom_base_class_fails_before_inherited_state_is_lost():
    cls = analyze_class(
        SHELL32 / "CCopyMoveToMenu.cpp",
        "CMoveToMenu",
        object_map_source=SHELL32 / "shell32.cpp",
    )
    with pytest.raises(CppComError, match="custom/inherited base class CCopyMoveToMenu"):
        cls.require_supported()


def test_custom_atl_lifecycle_fails_before_factory_generation():
    cls = analyze_class(
        SHELL32 / "folders" / "CMyDocsFolder.cpp",
        "CMyDocsFolder",
        object_map_source=SHELL32 / "shell32.cpp",
    )
    with pytest.raises(CppComError, match="custom ATL lifecycle method FinalConstruct"):
        cls.require_supported()


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


def test_worker_audit_rejects_body_local_abi_types():
    source = "{ return Call(); }"
    rust = """pub unsafe fn C_f_body() -> HRESULT {
        #[repr(C)] struct ABI_INFO { size: DWORD }
        Call()
    }"""
    findings = audit_worker_body(source, rust, "C_f_body")
    assert any("ABI type declaration" in item for item in findings)


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
        Call(IID_PPV_ARG(IShellItemArray, &mut array))
    }"""
    lowered = normalize_worker_semantics(source, rust)
    assert "let cch: SIZE_T = lstrlenW(str) as SIZE_T + 1;" in lowered
    assert "CStringW::from_resource(IDS_TEXT as UINT)" in lowered
    assert "str_text.as_mut_ptr()" in lowered
    assert "core::ptr::addr_of!(IID_IShellItemArray)" in lowered
    assert "core::ptr::addr_of_mut!(array).cast()" in lowered


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
