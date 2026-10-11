"""Tests for `crate_integrate` — landing a rustconv translation onto a crate prelude.

The two safety properties under test, in order of how much they hurt when broken:

  1. An extern decl the prelude ALREADY provides must be DROPPED. rustconv emits
     `extern "C" { fn HeapAlloc(..); }`; WINAPI is `__stdcall` on i386, so that decl asks
     for a symbol (`HeapAlloc`) that does not exist there (`_HeapAlloc@12` does) — and
     because a local item SHADOWS a glob import, it silently wins over the prelude's
     correct `extern "system"` decl. Links on amd64/arm64, breaks i386.
  2. An extern decl the prelude does NOT provide must be KEPT and REPORTED — never
     silently dropped into an undefined-at-link surprise.

Plus the two regression bugs inherited from `tools/integrate_ole32_ring.py`, which are the
reason the classifiers look the way they do (see the last two tests).
"""

from __future__ import annotations

import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

from rustconv.crate_integrate import (
    _PROVIDED_CACHE,
    extern_block_spans,
    extern_declared,
    integrate,
    promote_pub,
    provided_names,
)

PRELUDE = """\
//! Shared import surface.
pub use com_sys::*;
pub use crate::winemacros::*;
pub use crate::{TRACE, WARN};

pub type LPUNKNOWN = *mut IUnknown;

// kernel32 (WINAPI = extern "system" — the ABI the translations get WRONG).
extern "system" {
    pub fn GetProcessHeap() -> HANDLE;
    pub fn HeapAlloc(hHeap: HANDLE, dwFlags: DWORD, dwBytes: SIZE_T) -> *mut c_void;
    pub fn HeapFree(hHeap: HANDLE, dwFlags: DWORD, lpMem: *mut c_void) -> BOOL;
}
"""

WINEMACROS = """\
pub unsafe extern "C" fn IUnknown_Release(punk: *mut IUnknown) -> ULONG { 0 }
unsafe fn not_pub_so_not_reachable(x: u32) -> u32 { x }
pub const S_OK: HRESULT = 0;
"""

COM_SYS_LIB = """\
#![no_std]
#[cfg(target_arch = "x86")]
include!("bindgen_i386.rs");
#[cfg(target_arch = "x86_64")]
include!("bindgen_amd64.rs");
"""

# Deliberately different per arch: a name provided on ANY arch counts as provided.
BINDGEN_I386 = """\
pub type HRESULT = i32;
pub struct IUnknown { pub lpVtbl: *const c_void }
pub type DWORD = u32;
pub type I386_ONLY_T = u32;
"""
BINDGEN_AMD64 = """\
pub type HRESULT = i32;
pub struct IUnknown { pub lpVtbl: *const c_void }
pub type DWORD = u32;
pub type AMD64_ONLY_T = u64;
"""


def make_crate(tmp_path: Path, *, prelude: bool = True) -> Path:
    """A miniature workspace: <ws>/mycrate + <ws>/com-sys (a sibling, hyphen-named)."""
    ws = tmp_path / "rust"
    crate = ws / "mycrate"
    (crate / "src").mkdir(parents=True)
    if prelude:
        (crate / "src" / "prelude.rs").write_text(PRELUDE)
    (crate / "src" / "winemacros.rs").write_text(WINEMACROS)

    com = ws / "com-sys" / "src"
    com.mkdir(parents=True)
    (com / "lib.rs").write_text(COM_SYS_LIB)
    (com / "bindgen_i386.rs").write_text(BINDGEN_I386)
    (com / "bindgen_amd64.rs").write_text(BINDGEN_AMD64)

    _PROVIDED_CACHE.clear()  # per-crate cache is keyed by path; tmp_path is fresh anyway
    return crate


# --------------------------------------------------------------------------------------
# provided_names — computed from the real prelude, not a hardcoded list
# --------------------------------------------------------------------------------------


def test_provided_names_harvests_prelude_externs_globs_and_reexports(tmp_path):
    crate = make_crate(tmp_path)
    p = provided_names(crate)

    # the prelude's own extern "system" surface (the correctly-ABI'd Win32 decls)
    assert {"GetProcessHeap", "HeapAlloc", "HeapFree"} <= p
    # the prelude's own items
    assert "LPUNKNOWN" in p
    # `pub use crate::winemacros::*;` — pub items only
    assert "IUnknown_Release" in p and "S_OK" in p
    assert "not_pub_so_not_reachable" not in p  # private: a glob does NOT re-export it
    # `pub use com_sys::*;` — sibling workspace crate, underscore→hyphen dir, via include!
    assert {"HRESULT", "IUnknown", "DWORD"} <= p
    # explicit re-export list
    assert {"TRACE", "WARN"} <= p


def test_provided_names_unions_the_per_arch_bindgen_files(tmp_path):
    # A name provided on ANY arch counts as provided: dropping a decl the prelude supplies
    # is the whole point, and the prelude supplies it on the arch that has it.
    p = provided_names(make_crate(tmp_path))
    assert "I386_ONLY_T" in p
    assert "AMD64_ONLY_T" in p


def test_no_prelude_means_no_provided_names_and_no_rewrite(tmp_path):
    crate = make_crate(tmp_path, prelude=False)
    assert provided_names(crate) == set()

    src = 'use ntdll_sys::nt::*;\n\nextern "C" {\n    fn HeapAlloc(h: HANDLE) -> LPVOID;\n}\n'
    out, report = integrate(src, crate, module="x")
    # Fail-safe: never invent an import surface the crate doesn't have.
    assert out == src
    assert not report.changed


# --------------------------------------------------------------------------------------
# integrate
# --------------------------------------------------------------------------------------

TRANSLATED = """\
use ntdll_sys::nt::*;
use core::ffi::c_void;

extern "C" {
    fn TRACE(msg: *const u8, ...);
    fn IUnknown_Release(punk: *mut IUnknown) -> ULONG;
}
extern "C" {
    fn GetProcessHeap() -> HANDLE;
    fn HeapAlloc(hHeap: HANDLE, dwFlags: DWORD, dwBytes: SIZE_T) -> LPVOID;
    fn HeapFree(hHeap: HANDLE, dwFlags: DWORD, lpMem: LPVOID) -> BOOL;
    fn SomeInventedSymbol(x: DWORD) -> HRESULT;
}
extern "C" {
    static IID_IFoo: GUID;
}

unsafe extern "C" fn local_helper(x: DWORD) -> HRESULT { 0 }
"""


def test_substrate_glob_is_swapped_for_the_prelude(tmp_path):
    out, report = integrate(TRANSLATED, make_crate(tmp_path), module="m")
    assert "use ntdll_sys::nt::*;" not in out
    assert out.startswith("use crate::prelude::*;\n")
    assert report.substrate_swapped
    # unrelated `use` lines survive
    assert "use core::ffi::c_void;" in out


def test_provided_extern_with_wrong_abi_is_dropped_not_kept(tmp_path):
    """THE test: the `extern "C" fn HeapAlloc` decl (stdcall on i386!) must be gone."""
    out, report = integrate(TRANSLATED, make_crate(tmp_path), module="m")
    for gone in ("HeapAlloc", "HeapFree", "GetProcessHeap", "IUnknown_Release", "TRACE"):
        assert f"fn {gone}(" not in out, f"{gone} decl survived — it shadows the prelude"
        assert gone in report.dropped_provided


def test_unprovided_extern_is_kept_and_reported(tmp_path):
    out, report = integrate(TRANSLATED, make_crate(tmp_path), module="m")
    assert "fn SomeInventedSymbol(x: DWORD) -> HRESULT;" in out
    assert report.kept_unprovided == ["SomeInventedSymbol"]
    # A data static the prelude lacks is a genuine link symbol — keep, but flag it.
    assert "static IID_IFoo: GUID;" in out
    assert report.kept_statics == ["IID_IFoo"]


def test_fully_provided_block_is_deleted(tmp_path):
    out, report = integrate(TRANSLATED, make_crate(tmp_path), module="m")
    # 3 blocks in → the all-provided one is gone; the other two keep a decl each.
    assert len(extern_block_spans(out)) == 2
    assert report.blocks_removed == 1
    assert 'extern "C" {\n}' not in out and 'extern "C" {\n\n}' not in out


def test_kept_block_keeps_its_own_abi(tmp_path):
    # Rebuilding every block as `extern "C"` would silently downgrade an `extern "system"`
    # one — the exact i386-decoration bug this module exists to prevent.
    src = 'use ntdll_sys::nt::*;\nextern "system" {\n    fn NotInPrelude(x: DWORD) -> BOOL;\n}\n'
    out, _ = integrate(src, make_crate(tmp_path), module="m")
    assert 'extern "system" {' in out
    assert 'extern "C" {' not in out


def test_self_declared_function_decl_is_dropped(tmp_path):
    # rustconv translates function-by-function, so a caller can get an extern decl for a
    # callee defined in the very same file (E0428 duplicate definition).
    src = (
        "use ntdll_sys::nt::*;\n"
        'extern "C" {\n    fn local_helper(x: DWORD) -> HRESULT;\n}\n'
        'unsafe extern "C" fn local_helper(x: DWORD) -> HRESULT { 0 }\n'
    )
    out, report = integrate(src, make_crate(tmp_path), module="m")
    assert report.dropped_self == ["local_helper"]
    assert "fn local_helper(x: DWORD) -> HRESULT;" not in out
    assert 'unsafe extern "C" fn local_helper' in out  # the DEFINITION survives


def test_integrate_is_idempotent(tmp_path):
    crate = make_crate(tmp_path)
    once, _ = integrate(TRANSLATED, crate, module="m")
    twice, report2 = integrate(once, crate, module="m")
    assert once == twice
    assert not report2.changed


# --------------------------------------------------------------------------------------
# promote_pub — the DECLSPEC_HIDDEN cross-TU helpers
# --------------------------------------------------------------------------------------


def test_promote_pub_promotes_definitions_only(tmp_path):
    src = 'extern "C" {\n    fn FAILED(hr: HRESULT) -> BOOL;\n}\nunsafe fn Shared(x: u32) {}\n'
    out, promoted = promote_pub(src, {"Shared", "FAILED"})
    assert promoted == ["Shared"]
    assert "pub unsafe fn Shared(x: u32) {}" in out
    # A decl INSIDE an extern block is not a definition: promoting it would make the report
    # lie about what this file defines (the ole32 script's first cut did exactly this, and
    # promoted `FAILED`).
    assert "pub fn FAILED" not in out


def test_promote_pub_runs_for_names_siblings_declare(tmp_path):
    crate = make_crate(tmp_path)
    # A sibling module declares `Shared` extern → the definition here must become `pub`, or
    # the prelude's glob can't re-export it to that caller.
    (crate / "src" / "sibling.rs").write_text('extern "C" {\n    fn Shared(x: u32) -> u32;\n}\n')
    src = 'use ntdll_sys::nt::*;\nunsafe extern "C" fn Shared(x: u32) -> u32 { x }\n'
    out, report = integrate(src, crate, module="m")
    assert report.promoted_pub == ["Shared"]
    assert 'pub unsafe extern "C" fn Shared' in out


# --------------------------------------------------------------------------------------
# The two regression bugs preserved from tools/integrate_ole32_ring.py.
# --------------------------------------------------------------------------------------


def test_regression_comment_preceded_decl_is_still_classified(tmp_path):
    """split_decls glues a decl's preceding comment lines onto it. With a `^`-anchored but
    NON-multiline classifier, every comment-preceded decl failed to match — and an
    unclassified decl is KEPT, i.e. a stale `extern "C" fn HeapAlloc` survives and breaks
    i386. The classifiers are re.M and strip comment lines before matching."""
    src = (
        "use ntdll_sys::nt::*;\n"
        'extern "C" {\n'
        "    // kernel32 heap (WINAPI)\n"
        "    fn HeapAlloc(hHeap: HANDLE, dwFlags: DWORD, dwBytes: SIZE_T) -> LPVOID;\n"
        "}\n"
    )
    out, report = integrate(src, make_crate(tmp_path), module="m")
    assert report.dropped_provided == ["HeapAlloc"]
    assert "fn HeapAlloc" not in out


def test_regression_uppercase_extern_C_abi_matches(tmp_path):
    """The ABI character class must accept uppercase: `extern "C"` is the common spelling.
    A `[a-z]+` class silently saw no definitions at all, which made the self-decl /
    provided-by-sibling checks quietly useless."""
    crate = make_crate(tmp_path)
    # (a) blocks: an uppercase-"C" block must be found at all
    assert len(extern_block_spans('extern "C" {\n    fn Foo();\n}\n')) == 1
    assert extern_declared('extern "C" {\n    fn Foo();\n}\n') == {"Foo"}
    # (b) definitions: `pub unsafe extern "C" fn X` in a module must count as provided
    (crate / "src" / "winemacros.rs").write_text(
        'pub unsafe extern "C" fn UpperCaseAbiFn(x: u32) -> u32 { x }\n'
    )
    _PROVIDED_CACHE.clear()
    assert "UpperCaseAbiFn" in provided_names(crate)
    src = 'use ntdll_sys::nt::*;\nextern "C" {\n    fn UpperCaseAbiFn(x: u32) -> u32;\n}\n'
    out, report = integrate(src, crate, module="m")
    assert report.dropped_provided == ["UpperCaseAbiFn"]
    assert "fn UpperCaseAbiFn" not in out


def test_extern_block_in_trailing_c_comments_is_ignored(tmp_path):
    """assemble_file appends the original C source as `//` comments — a C++-guarded header
    quoted there really does contain `extern "C" {`."""
    src = (
        "use ntdll_sys::nt::*;\n"
        'unsafe extern "C" fn f() {}\n'
        "// ===== ORIGINAL C SOURCE =====\n"
        '// extern "C" {\n'
        "//     void HeapAlloc(void);\n"
        "// }\n"
    )
    out, report = integrate(src, make_crate(tmp_path), module="m")
    assert report.blocks_removed == 0
    assert '// extern "C" {' in out  # the comment block is untouched


def test_bound_fn_pointer_expect_becomes_unchecked():
    from rustconv.crate_integrate import unchecked_fn_pointer_calls

    src = (
        "let f = (*d).Chars\n    .QueryInformationHandler\n"
        '    .expect("non-null function pointer");\nStatus = f(a, b);\n'
        'let g = (*d).Other.expect("non-null function pointer");\nstore(g);\n'
    )
    out, n = unchecked_fn_pointer_calls(src)
    assert n == 1
    assert ".QueryInformationHandler\n    .unwrap_unchecked();" in out
    assert '(*d).Other.expect("non-null function pointer");' in out  # stored, not called


def test_rewrapped_fn_pointer_expect_becomes_unchecked():
    from rustconv.crate_integrate import unchecked_fn_pointer_calls

    src = 'Some(\n  (*d).ISRHandler\n    .expect("non-null function pointer"),\n)\n.unwrap_unchecked()(a);'
    out, n = unchecked_fn_pointer_calls(src)
    assert n == 1 and "expect" not in out


def test_a_type_the_prelude_reexports_from_this_module_is_kept(tmp_path):
    """resolve wires `pub use crate::m::unit::LPFNCONSTRUCTOR;`; integrating `unit` must not
    then drop LPFNCONSTRUCTOR as prelude-provided -- that deleted msctf's two typedefs."""
    crate = make_crate(tmp_path)
    prelude = crate / "src" / "prelude.rs"
    prelude.write_text(
        prelude.read_text() + "pub use crate::m::unit::LPFNCONSTRUCTOR;\n"
        "pub use crate::m::other::Elsewhere;\n"
    )
    _PROVIDED_CACHE.clear()
    src = (
        "use crate::prelude::*;\n"
        'pub type LPFNCONSTRUCTOR = unsafe extern "C" fn() -> i32;\n'
        "pub type Elsewhere = u32;\n"
    )
    out, _ = integrate(src, crate, module="unit")
    assert "pub type LPFNCONSTRUCTOR" in out
    assert "pub type Elsewhere" not in out  # another module's: still the prelude's to supply
