"""Unit tests for scan_formatless_printf.

Pins the three shapes that ship: the verbatim-copy stub (freeldr's RtlStringCbPrintfW),
the empty-string stub (classpnp's pair), and the two that are RIGHT and must stay quiet:
a v-form taking a va_list, and a fixed-slot replay (KdIoPrintf).
"""

from __future__ import annotations

import importlib.util
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent
_spec = importlib.util.spec_from_file_location(
    "scan_formatless_printf", REPO / "tools" / "scan_formatless_printf.py"
)
assert _spec is not None and _spec.loader is not None
sfp = importlib.util.module_from_spec(_spec)
sys.modules["scan_formatless_printf"] = sfp
_spec.loader.exec_module(sfp)


def _scan(tmp_path: Path, body: str) -> list[str]:
    f = tmp_path / "shim.rs"
    f.write_text(body)
    return [name for _, _, name in sfp.scan([f])]


def test_flags_a_def_that_ends_at_its_format(tmp_path: Path) -> None:
    body = """
#[no_mangle]
pub unsafe extern "C" fn RtlStringCbPrintfW(
    dest: *mut u16,
    cb_dest: usize,
    fmt: *const u16,
) -> NTSTATUS { 0 }
#[no_mangle] pub unsafe extern "C" fn RtlStringCchPrintfA(d: *mut i8, n: usize, fmt: *const i8) {}
"""
    assert _scan(tmp_path, body) == ["RtlStringCbPrintfW", "RtlStringCchPrintfA"]


def test_line_number_is_the_attribute(tmp_path: Path) -> None:
    body = '\n\n#[no_mangle]\npub unsafe extern "C" fn sprintf(b: *mut i8, format: *const i8) {}\n'
    f = tmp_path / "shim.rs"
    f.write_text(body)
    assert [(line, name) for _, line, name in sfp.scan([f])] == [(3, "sprintf")]


def test_quiet_on_slots_va_list_and_declarations(tmp_path: Path) -> None:
    body = """
#[no_mangle]
pub unsafe extern "C" fn KdIoPrintf(Format: PCSTR, a0: usize, a1: usize) {}
#[no_mangle]
pub unsafe extern "C" fn _vsnwprintf(b: *mut u16, n: usize, fmt: *const u16, ap: va_list) {}
#[no_mangle]
pub unsafe extern "C" fn StringCchVPrintfW(d: *mut u16, n: usize, fmt: *const u16, ap: *mut u8) {}
extern "C" {
    fn RtlStringCbPrintfW(dest: *mut u16, cb: usize, fmt: *const u16, ...) -> NTSTATUS;
}
// #[no_mangle] pub unsafe extern "C" fn RtlStringCbPrintfA(d: *mut i8, n: usize, fmt: *const i8) {}
"""
    assert _scan(tmp_path, body) == []
