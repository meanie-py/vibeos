"""Tests for scan_swprintf_count: a counted swprintf linked to msvcrt's legacy export."""

from __future__ import annotations

import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))

from scan_swprintf_count import scan_text


def test_the_shipped_counted_declaration_fires():
    text = (
        'extern "C" {\n'
        "    pub fn swprintf(s: *mut u16, n: usize, fmt: *const u16, ...) -> c_int;\n"
        "}\n"
    )
    assert scan_text(text) == [(2, "swprintf", "COUNTED_UNLINKED")]


def test_a_legacy_form_linked_to_a_counted_export_fires():
    text = '#[link_name = "_snwprintf"]\nfn swprintf(buf: *mut u16, fmt: *const u16, ...) -> i32;\n'
    assert scan_text(text) == [(2, "swprintf", "LEGACY_RELINKED")]


def test_correct_declarations_and_definitions_are_silent():
    text = (
        'extern "C" {\n'
        '    #[link_name = "_snwprintf"]\n'
        "    pub fn swprintf(buf: *mut WCHAR, count: usize, fmt: *const WCHAR, ...) -> c_int;\n"
        "    fn swprintf(buffer: *mut u16, format: *const u16, ...) -> i32;\n"
        "    pub fn vswprintf(\n        buf: *mut u16,\n        fmt: *const u16,\n        args: va_list,\n    ) -> c_int;\n"
        "}\n"
        "pub unsafe fn swprintf(buf: *mut u16, n: usize, fmt: *const u16) -> i32 { 0 }\n"
    )
    assert scan_text(text) == []
