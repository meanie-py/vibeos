"""Tests for scan_ptr_highword_mask — `ptr & 0xFFFF0000 == 0` as an "at most 0xFFFF" test.

The fire cases are the two shapes the tree actually shipped (rtl-strings' integer-atom test
and rtl-misc's resource-name test). The must-not cases are the neighbours that share the mask
but not the meaning: a u32 raster op, a version DWORD, the call-proc `== 0xFFFF0000` form
(whose looseness is load-bearing), the corrected `& !0xFFFF` form, and the C quoted in a
comment.
"""

from __future__ import annotations

import subprocess
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))

from scan_ptr_highword_mask import scan_text

TOOL = Path(__file__).resolve().parent / "scan_ptr_highword_mask.py"


def test_integer_atom_shape_fires():
    # The shipped form: a bare identifier whose usize-ness is only in its declaration.
    src = "pub fn check(ptr_bits: usize) -> u16 {\n    if (ptr_bits & 0xFFFF_0000) == 0 {\n"
    assert [ln for ln, _ in scan_text(src)] == [2]


def test_bare_u32_identifier_does_not_fire():
    src = "fn f(rop4: u32) -> bool {\n    (rop4 & 0xFFFF0000) == 0\n}\n"
    assert scan_text(src) == []


def test_resource_name_shape_fires():
    src = "    if ((name as usize) & 0xFFFF0000) == 0 {\n"
    assert len(scan_text(src)) == 1


def test_not_equal_and_ulong_ptr_fire():
    src = "let x = ((p as ULONG_PTR) & 0xffff0000u64) != 0;\n"
    assert len(scan_text(src)) == 1


def test_u32_rop_does_not_fire():
    assert scan_text("    if (rop4 & 0xFFFF0000) == 0 { 1 } else { 0 }\n") == []


def test_version_dword_does_not_fire():
    src = "    if ((CmNtCSDVersion & 0xFFFF0000) == 0) && (CmNtCSDReleaseType == 1) {\n"
    assert scan_text(src) == []


def test_callproc_equality_form_does_not_fire():
    src = "    ((v as usize & 0xFFFF0000) == 0xFFFF0000) as BOOL\n"
    assert scan_text(src) == []


def test_fixed_form_does_not_fire():
    assert scan_text("    if (name as usize) & !0xFFFF == 0 {\n") == []


def test_comment_quoting_the_c_does_not_fire():
    src = "// C: if (!((ULONG_PTR)AtomName & 0xFFFF0000)) -- (x as usize & 0xFFFF0000) == 0\n"
    assert scan_text(src) == []


def test_tree_is_clean():
    proc = subprocess.run([sys.executable, str(TOOL)], capture_output=True, text=True, check=True)
    assert proc.stdout == ""
