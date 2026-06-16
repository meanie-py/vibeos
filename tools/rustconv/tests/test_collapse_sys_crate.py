"""Regression tests for `collapse_sys_crate`'s sibling-substrate retarget.

The user-mode translation prompt names ntdll-sys, so the LLM frequently
imports `ntdll_sys::nt::*` even when the configured target is a *different*
user-mode substrate (winlogon-sys, winsrv-sys, …). Before the fix,
`collapse_sys_crate` only rewrote the kernel crate (`ntkrnl_sys::`), so the
wrong sibling import survived postprocess and failed to compile (E0432:
unresolved import — winlogon-rust doesn't depend on ntdll-sys).

These pin the deterministic retarget onto the configured `sys_crate`.
"""

from __future__ import annotations

import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

from rustconv.postprocess import collapse_sys_crate


def test_retargets_wrong_sibling_sys_crate_glob():
    code = "use ntdll_sys::nt::*;\nuse crate::preamble::*;\n\nfn f() {}\n"
    out = collapse_sys_crate(code, sys_crate="winlogon_sys", sys_umbrella="nt")
    assert "ntdll_sys" not in out
    assert "use winlogon_sys::nt::*;" in out
    # The unrelated import is untouched.
    assert "use crate::preamble::*;" in out


def test_retargets_sibling_module_path_to_umbrella():
    # A nested-module path collapses to the umbrella, like ntkrnl_sys does.
    code = "let x = ntdll_sys::base::PVOID::default();\n"
    out = collapse_sys_crate(code, sys_crate="winlogon_sys", sys_umbrella="nt")
    assert "ntdll_sys" not in out
    assert "winlogon_sys::nt::PVOID" in out


def test_leaves_the_targets_own_crate_alone():
    # If the LLM already used the right crate, no spurious rewrite/dup.
    code = "use winlogon_sys::nt::*;\n\nfn f() {}\n"
    out = collapse_sys_crate(code, sys_crate="winlogon_sys", sys_umbrella="nt")
    assert out.count("use winlogon_sys::nt::*;") == 1


def test_kernel_crate_still_collapses():
    # The original ntkrnl_sys behaviour is preserved.
    code = "use ntkrnl_sys::ddk::FOO;\nlet y = ntkrnl_sys::base::BAR;\n"
    out = collapse_sys_crate(code, sys_crate="ntdll_sys", sys_umbrella="nt")
    assert "ntkrnl_sys" not in out
    assert "ntdll_sys::nt::BAR" in out
    assert "use ntdll_sys::nt::*;" in out
