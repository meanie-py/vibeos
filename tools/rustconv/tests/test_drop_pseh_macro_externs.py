"""Regression tests for `drop_pseh_macro_externs`.

Under USE_DUMMY_PSEH the `_SEH2_*` handlers are dead code, and the system
prompt tells the LLM to drop them — but rustconv still occasionally emits an
`extern "system" { fn _SEH2_TRY(); … }` block. Those are PSEH *macros*, not
symbols: declaring them extern is a guaranteed link failure if referenced and
dead noise otherwise. `strip_user_inline_redeclarations` only drops a block
when EVERY fn is a known PEB/TEB inline, so a mixed SEH2 + real-import block
leaks the SEH2 lines through. This pass strips just those decls.
"""

from __future__ import annotations

import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

from rustconv.postprocess import drop_pseh_macro_externs


def test_drops_pure_seh2_block_entirely():
    code = (
        'extern "system" {\n'
        "    fn _SEH2_TRY();\n"
        "    fn _SEH2_EXCEPT(filter: i32) -> i32;\n"
        "    fn _SEH2_GetExceptionCode() -> u32;\n"
        "    fn _SEH2_END();\n"
        "}\n"
    )
    out = drop_pseh_macro_externs(code)
    # No real decls survive (the comment breadcrumb names them, but there is
    # no `fn _SEH2_` decl and no extern block left).
    assert "fn _SEH2_" not in out
    assert 'extern "system"' not in out
    assert "drop_pseh_macro_externs" in out  # left an audit breadcrumb


def test_mixed_block_keeps_real_fn_drops_seh2():
    code = (
        'extern "system" {\n'
        "    fn _SEH2_TRY();\n"
        "    fn WlStrDup(s: PCWSTR) -> PWSTR;\n"
        "    fn _SEH2_GetExceptionInformation() -> *mut u8;\n"
        "}\n"
    )
    out = drop_pseh_macro_externs(code)
    assert "WlStrDup" in out
    assert "fn _SEH2_TRY" not in out
    assert "_SEH2_GetExceptionInformation" not in out.split("// drop")[0]
    # the surviving block keeps the original ABI
    assert 'extern "system" {' in out


def test_drops_rtlgetprocessheap_macro():
    code = 'extern "system" {\n    fn RtlGetProcessHeap() -> PVOID;\n}\n'
    out = drop_pseh_macro_externs(code)
    assert "fn RtlGetProcessHeap" not in out
    assert 'extern "system"' not in out


def test_preserves_unrelated_extern_block():
    code = (
        'extern "C" {\n'
        "    fn GetProcessHeap() -> PVOID;\n"
        "    fn HeapAlloc(h: PVOID, f: u32, n: usize) -> PVOID;\n"
        "}\n"
    )
    out = drop_pseh_macro_externs(code)
    assert out == code  # no SEH2/macro names → untouched, byte-identical


def test_pub_fn_seh2_also_dropped():
    # rustconv sometimes marks them `pub fn`.
    code = (
        'extern "system" {\n'
        "    pub fn _SEH2_Volatile0() -> i32;\n"
        "    pub fn wcschr(s: PCWSTR, c: u16) -> PWSTR;\n"
        "}\n"
    )
    out = drop_pseh_macro_externs(code)
    assert "_SEH2_Volatile0" not in out.split("// drop")[0]
    assert "wcschr" in out
