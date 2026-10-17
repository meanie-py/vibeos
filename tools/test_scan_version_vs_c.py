"""Runs the built-in self-tests of the two scanners the 2026-10-16 version sweep built on:
scan_version_vs_c (the NTDDI/__REACTOS__ evaluator and the Rust gate masker) and
scan_macro_fn_shadow (its `{NOTHING;}` harvest). Neither self-test was run by any gate."""

from __future__ import annotations

import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent.parent))
from tools import scan_macro_fn_shadow, scan_version_vs_c


def test_scan_version_vs_c_self_test():
    assert scan_version_vs_c.self_test() == 0


def test_scan_macro_fn_shadow_self_test():
    assert scan_macro_fn_shadow.self_test() == 0


def test_module_version_reads_the_build_before_cmake():
    # classpnp's CMakeLists lost its 0x602 in the cargo cutover; history recovers it.
    env, src = scan_version_vs_c.module_version(
        scan_version_vs_c.REPO / "drivers/storage/class/classpnp/utils.c"
    )
    assert env is not None
    assert env["_WIN32_WINNT"] == 0x602 and env["NTDDI_VERSION"] == 0x06020000, src
