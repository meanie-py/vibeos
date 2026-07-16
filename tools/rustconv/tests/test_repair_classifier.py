from rustconv.repair_classifier import (
    classify_diagnostic,
    repair_worker,
    source_findings,
)


def test_classifies_known_rustc_failures():
    assert classify_diagnostic({"code": "E0435", "message": "not constant"}) == "const_context"
    assert classify_diagnostic({"code": "E0659", "message": "ambiguous"}) == "ambiguous_import"
    assert classify_diagnostic({"code": None, "message": "expected expression"}) == "raw_c_syntax"


def test_source_failure_backlog_detectors_are_mechanical():
    rust = """use crate::types::*;
fn f() { goto out; x = cond ? 1 : 2; }
out:
"""
    assert source_findings(rust) == ["glob_import", "raw_c_goto", "raw_c_ternary"]


def test_repair_uses_base_signature_and_promotes_constant_array_bound():
    base = "pub unsafe fn body(x: DWORD) -> HRESULT { x as HRESULT }"
    candidate = """pub unsafe fn body(x: u32) -> i32 {
        let width: usize = (38 + 1) as usize;
        let a: [u16; width] = [0; width];
        a[0] as i32 + x as i32
    }"""
    result = repair_worker(base, candidate, "body")
    assert result.code.startswith("pub unsafe fn body(x: DWORD) -> HRESULT")
    assert "const width: usize" in result.code
    assert result.applied == ("const_context", "generated_signature")


def test_repair_restores_only_base_authorized_call_argument_casts():
    base = """pub unsafe fn body(cch: UINT, p: PWSTR) -> HRESULT {
        let _ = Copy(p, cch as usize, p);
        Free(p as *mut core::ffi::c_void);
        Other(p);
        0
    }"""
    candidate = """pub unsafe fn body(cch: UINT, p: PWSTR) -> HRESULT {
        let _ = Copy(p, cch, p);
        Free(p);
        Other(core::ptr::null_mut());
        0
    }"""
    result = repair_worker(base, candidate, "body")
    assert "Copy(p, cch as usize, p)" in result.code
    assert "Free(p as *mut core::ffi::c_void)" in result.code
    assert "Other(core::ptr::null_mut())" in result.code
    assert result.applied == ("base_argument_casts:2",)


def test_repair_restores_base_initializer_and_marks_reassigned_local_mutable():
    base = """pub unsafe fn body() -> HRESULT {
        let out: PWSTR = core::ptr::null_mut();
        let hr: HRESULT;
        hr = First(&mut out);
        hr
    }"""
    candidate = """pub unsafe fn body() -> HRESULT {
        let out: PWSTR;
        let hr: HRESULT;
        hr = First(&mut out);
        hr = Second(out);
        hr
    }"""
    result = repair_worker(base, candidate, "body")
    assert "let out: PWSTR = core::ptr::null_mut();" in result.code
    assert "let mut hr: HRESULT;" in result.code
    assert result.applied == ("base_initializers:1", "mutable_reassignments:1")


def test_repair_restores_base_mutability_bool_values_and_integer_predicate():
    base = """pub unsafe fn body() -> HRESULT {
        let mut out: PWSTR = core::ptr::null_mut();
        if IS_INTRESOURCE(out) != 0 { Call(TRUE, &mut out); }
        0
    }"""
    candidate = """pub unsafe fn body() -> HRESULT {
        let out: PWSTR = core::ptr::null_mut();
        if IS_INTRESOURCE(out) { Call(true, &mut out); }
        0
    }"""
    result = repair_worker(base, candidate, "body")
    assert "let mut out" in result.code
    assert "IS_INTRESOURCE(out) != 0" in result.code
    assert "Call(TRUE, &mut out)" in result.code
    assert result.applied == (
        "base_mutability:1",
        "base_argument_values:1",
        "base_integer_predicates:1",
    )


def test_repair_lowers_no_std_wide_alloc_and_typed_memcpy_idioms():
    base = """unsafe fn wide_len(_: LPCWSTR) -> usize { 0 }
    unsafe fn body(this: *mut Object, file: LPCWSTR) -> HRESULT {
        let len = wide_len(file);
        let target = crate::wine::shellole::SHAlloc((len + 1) * core::mem::size_of::<WCHAR>()) as LPWSTR;
        core::ptr::copy_nonoverlapping(file, target, len + 1);
        0
    }"""
    candidate = """unsafe fn body(this: *mut Object, file: LPCWSTR) -> HRESULT {
        let len = strlenW(file) as usize;
        let target = SHAlloc((len + 1) * std::mem::size_of::<WCHAR>()) as LPWSTR;
        memcpy(target as *mut core::ffi::c_void, file as *const core::ffi::c_void,
               (len + 1) * core::mem::size_of::<WCHAR>());
        0
    }"""
    result = repair_worker(base, candidate, "body")
    assert "let len = wide_len(file) as usize" in result.code
    assert "crate::wine::shellole::SHAlloc" in result.code
    assert "core::ptr::copy_nonoverlapping(" in result.code
    assert "(file) as *const u8, (target) as *mut u8" in result.code
    assert "(len + 1) * core::mem::size_of::<WCHAR>()" in result.code
    assert result.applied == (
        "no_std_paths:1",
        "callee_alias:strlenW->wide_len:1",
        "callee_path:SHAlloc:1",
        "typed_memcpy:1",
    )


def test_repair_lowers_com_union_out_pointers_and_sealed_strsafe_append():
    base = """unsafe fn init_format_etc(_: *mut FORMATETC, _: u16, _: u32) {}
    unsafe fn append_argument(_: &mut LPWSTR, _: &mut usize, _: LPCWSTR, _: bool) {}
    unsafe fn body(data: *mut IDataObject) -> HRESULT {
        let mut fmt: FORMATETC = core::mem::zeroed();
        let mut medium: STGMEDIUM = core::mem::zeroed();
        init_format_etc(&mut fmt, CF_HDROP, TYMED_HGLOBAL);
        let _ = medium.u.hGlobal;
        0
    }"""
    candidate = """pub static CPP_COM_WSTR_0123456789AB: [WCHAR; 7] = [0x22, 0x25, 0x6c, 0x73, 0x22, 0x20, 0];
    unsafe fn body(data: *mut IDataObject) -> HRESULT {
        let mut fmt: FORMATETC = std::mem::zeroed();
        let mut medium: STGMEDIUM = std::mem::zeroed();
        InitFormatEtc(&mut fmt, CF_HDROP, TYMED_HGLOBAL);
        ((*(*data).lpVtbl).GetData)(data, &mut fmt, &mut medium);
        let _ = medium.hGlobal;
        let mut end = core::ptr::null_mut();
        let mut remaining = 10;
        StringCchPrintfExW(end, remaining, &mut end, &mut remaining, 0,
            CPP_COM_WSTR_0123456789AB.as_ptr(), core::ptr::null());
        0
    }"""
    result = repair_worker(base, candidate, "body")
    assert "init_format_etc(&mut fmt" in result.code
    assert "(&mut fmt as *mut FORMATETC).cast()" in result.code
    assert "(&mut medium as *mut STGMEDIUM).cast()" in result.code
    assert "medium.u.hGlobal" in result.code
    assert "append_argument(&mut end, &mut remaining, core::ptr::null(), true)" in result.code


def test_repair_restores_abi_struct_field_names_pointer_nulls_and_signed_offsets():
    base = """unsafe fn body() -> HRESULT {
        let mut info = INFO { hInstApp: core::ptr::null_mut(), hIconOrMonitor: core::ptr::null_mut() };
        let p = path.offset(wsprintfW(path, FORMAT.as_ptr()) as isize);
        let error = ERROR_SUCCESS as DWORD;
        0
    }"""
    candidate = """unsafe fn body() -> HRESULT {
        let mut info = INFO { hInstApp: 0, hIcon: core::ptr::null_mut() };
        let offset = wsprintfW(path, FORMAT.as_ptr());
        let p = path.add(offset);
        let error: u32 = if ok { ERROR_SUCCESS } else { GetLastError() };
        0
    }"""
    result = repair_worker(base, candidate, "body")
    assert "hInstApp: core::ptr::null_mut()" in result.code
    assert "hIconOrMonitor: core::ptr::null_mut()" in result.code
    assert "path.offset(offset as isize)" in result.code
    assert "ERROR_SUCCESS as u32" in result.code
