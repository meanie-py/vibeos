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
