"""Unit tests for tools/rpc_try_wrap.py's dead except-arm cleanup."""

from __future__ import annotations

import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import rpc_try_wrap as w

WRAPPED = """fn f() -> u32 {
    let mut ret: u32 = 0;
    let mut _SEH2_Volatile0: i32 = 0;
    let mut _SEH2_VolatileExceptionCode: u32 = 0;
    ret = rpc_try(|| Stub(a)).unwrap_or_else(|code| Map(code as _) as _);
    if _SEH2_Volatile0 != 0 || 0 != 0 && 1 != 0 {
        ret = Map(RpcExceptionCode());
    }
    let mut __seh_raised: i32 = 0;
    ret = rpc_try(|| Stub(b)).unwrap_or_else(|code| code as _);
    if __seh_raised != 0 {
        ret = Map(_SEH2_VolatileExceptionCode);
    }
    ret
}
"""


def test_drops_arms_and_then_the_unused_flags():
    new, dropped = w.drop_dead_arms(WRAPPED)
    assert dropped == 2
    assert "RpcExceptionCode" not in new and "_SEH2_Volatile" not in new
    assert "__seh_raised" not in new
    assert new.count("rpc_try(") == 2


def test_keeps_an_arm_that_does_not_read_the_exception_code():
    text = "fn g() {\n    let x = 0;\n    if _SEH2_Volatile0 != 0 {\n        cleanup();\n    }\n}\n"
    new, dropped = w.drop_dead_arms(text)
    assert dropped == 0 and new == text  # a live flag read keeps the locals too


def test_a_path_qualified_or_wrapped_line_counts_as_guarded():
    handlers = {"ElfrReportEventW": ("assign", "I_RpcMapWin32Status", None)}
    text = (
        "fn h() {\n    let s = rpc_try(||\n        advapi32_sys::ElfrReportEventW(a, b)\n"
        "    ).unwrap_or_else(|code| code as _);\n}\n"
    )
    _, done, leftovers = w.rewrite(text, handlers, set())
    assert done == 0 and leftovers == []


def test_a_block_closure_counts_as_guarded():
    handlers = {"ElfrReportEventW": ("assign", "I_RpcMapWin32Status", None)}
    text = "fn h() {\n    s = rpc_try(|| {\n        advapi32_sys::ElfrReportEventW(a)\n    });\n}\n"
    _, done, leftovers = w.rewrite(text, handlers, set())
    assert done == 0 and leftovers == []


def test_any_flag_name_that_is_never_raised():
    text = (
        "fn j() {\n    let rpc_faulted: bool = false;\n    if rpc_faulted {\n"
        "        e = RpcExceptionCode!() as DWORD;\n    }\n"
        "    if x\n    {\n        y();\n    }\n}\n"
    )
    new, dropped = w.drop_dead_arms(text)
    assert dropped == 1 and "rpc_faulted" not in new and "if x" in new


def test_a_flag_that_can_be_raised_is_left_alone():
    text = (
        "fn k() {\n    let mut hit = false;\n    hit = probe();\n    if hit {\n"
        "        e = RpcExceptionCode!();\n    }\n}\n"
    )
    assert w.drop_dead_arms(text) == (text, 0)
