"""Tests for scan_calloc_elem_type: a BSTR array sized in OLECHARs (wscript's Host_Echo)."""

from __future__ import annotations

import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))

from scan_calloc_elem_type import scan_text


def test_the_wscript_echo_array_fires():
    text = (
        "    argc = (ubound - lbound + 1) as u32;\n"
        "    strs = calloc((argc as usize) as _,\n"
        "        (core::mem::size_of::<OLECHAR>()) as _) as *mut BSTR;\n"
    )
    assert scan_text(text) == [(2, "OLECHAR", "BSTR")]


def test_a_byte_array_of_pointers_fires_in_any_spelling():
    text = "let p = heap_calloc(n, ::core::mem::size_of::<u8>()) as *mut LPWSTR;\n"
    assert scan_text(text) == [(1, "u8", "LPWSTR")]


def test_matching_and_pointer_sized_elements_are_silent():
    text = (
        "strs = calloc(argc as _, size_of::<BSTR>() as _) as *mut BSTR;\n"
        "buf = calloc(n, core::mem::size_of::<WCHAR>()) as *mut WCHAR;\n"
        "t = calloc(32, core::mem::size_of::<*mut core::ffi::c_void>() as _) as *mut _PVFV;\n"
        "v = calloc(1, size_of::<u8>()) as *mut c_void;\n"
    )
    assert scan_text(text) == []


def test_a_flexible_array_tail_and_a_comment_are_not_candidates():
    text = (
        "cur = malloc(offset_of!(T, data) + n * size_of::<c_char>()) as *mut T;\n"
        "// strs = calloc(argc, size_of::<OLECHAR>()) as *mut BSTR;\n"
    )
    assert scan_text(text) == []
