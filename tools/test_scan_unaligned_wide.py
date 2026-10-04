"""tools/scan_unaligned_wide.py: what it flags and what it rewrites."""

import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent.parent))
from tools import scan_unaligned_wide as suw

BAD = 'f(b"L\\0i\\0b\\0\\0\\0".as_ptr() as LPCWSTR);'


def test_flags_a_wide_byte_literal_cast_to_a_wide_pointer():
    assert suw.SITE.search(BAD)
    assert suw.SITE.search('x = b"a\\0\\0\\0".as_ptr() as *const u16;')
    assert suw.SITE.search('x = b"a\\0\\0\\0".as_ptr() as *mut WCHAR;')


def test_leaves_narrow_strings_and_the_fixed_form_alone():
    assert not suw.SITE.search('f(b"text\\0".as_ptr() as PCSTR);')
    assert not suw.SITE.search('f(win32macros_rust::wide_bytes!(b"L\\0\\0\\0") as LPCWSTR);')


def test_the_rewrite_keeps_the_bytes_and_the_cast():
    out = suw.SITE.sub(lambda m: f"{suw.MACRO}({m.group('lit')}){m.group('cast')}", BAD)
    assert out == 'f(win32macros_rust::wide_bytes!(b"L\\0i\\0b\\0\\0\\0") as LPCWSTR);'
    assert not suw.SITE.search(out)
