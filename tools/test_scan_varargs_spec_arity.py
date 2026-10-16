"""Tests for scan_varargs_spec_arity: a count-form swprintf bound to msvcrt's legacy export."""

from __future__ import annotations

import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))

from scan_varargs_spec_arity import scan_text, spec_arities

SPEC = spec_arities(
    [
        "@ varargs _snwprintf(ptr long wstr)\n"
        "@ varargs swprintf(ptr wstr) _swprintf # Non-conforming swprintf\n"
        "@ cdecl vswprintf(ptr wstr ptr) _vswprintf\n"
    ]
)


def test_the_wscript_declaration_fires():
    text = (
        'extern "C" {\n'
        "    pub fn swprintf(s: *mut u16, n: usize, fmt: *const u16, ...) -> c_int; // x.rs\n"
        "}\n"
    )
    assert scan_text(text, SPEC) == [(2, "swprintf", 3, [2])]


def test_the_fixed_bindings_are_silent():
    text = (
        '    #[link_name = "_snwprintf"]\n'
        "    pub fn swprintf(s: *mut u16, n: usize, fmt: *const u16, ...) -> c_int;\n"
        "    fn swprintf(buffer: *mut WCHAR, format: *const WCHAR, ...) -> INT;\n"
        "    pub fn _snwprintf(b: *mut u16, n: usize, f: *const u16, ...) -> i32;\n"
    )
    assert scan_text(text, SPEC) == []


def test_a_link_name_is_what_is_checked_and_generics_do_not_split_params():
    text = (
        '    #[link_name = "swprintf"]\n'
        "    fn my_fmt(b: *mut u16, n: Option<fn(u32, u32)>, f: *const u16, ...) -> i32;\n"
    )
    assert scan_text(text, SPEC) == [(2, "swprintf", 3, [2])]


def test_non_variadic_decls_unknown_names_and_comments_are_not_candidates():
    text = (
        "    pub fn vswprintf(b: *mut u16, n: usize, f: *const u16, a: va_list) -> i32;\n"
        "    pub fn my_printf(a: i32, b: i32, c: i32, ...) -> i32;\n"
        "    // pub fn swprintf(s: *mut u16, n: usize, fmt: *const u16, ...) -> c_int;\n"
    )
    assert scan_text(text, SPEC) == []
