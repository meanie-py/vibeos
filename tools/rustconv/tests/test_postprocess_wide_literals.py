"""A C `L"..."` hand-encoded as UTF-16LE BYTES is aligned wrong, and nothing says so.

`b"A\\0S\\0U\\0S\\0\\0\\0".as_ptr() as PCWSTR` has the right CONTENT, compiles, links, and
reads fine in review -- but `b"..."` has alignment 1, so the `*const u16` can land on an odd
address: BUGCHECK 0x1E STATUS_DATATYPE_MISALIGNMENT on amd64/arm64. mswsock shipped three;
sysdm's smbios/hardprof/userprofile/virtmem carried 110.

The rewrite must be conservative in the other direction too: a byte string behind a NARROW
pointer is CORRECT, and anything that does not decode to clean NUL-terminated ASCII is left
alone, because a wrong rewrite here is silent data corruption instead of a compile error.
"""

from __future__ import annotations

import sys
from pathlib import Path

import pytest

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

from rustconv.postprocess import fix_wide_byte_string_literals


def test_rewrites_a_wide_byte_string_and_keeps_the_cast():
    src = 'let p = b"A\\0S\\0U\\0S\\0\\0\\0".as_ptr() as PCWSTR;'
    assert fix_wide_byte_string_literals(src) == 'let p = w!("ASUS") as PCWSTR;'


@pytest.mark.parametrize(
    "ty", ["PCWSTR", "PWSTR", "LPCWSTR", "LPWSTR", "LPCTSTR", "LPTSTR", "*const u16", "*mut u16"]
)
def test_every_wide_spelling_is_covered(ty):
    src = f'b"H\\0P\\0\\0\\0".as_ptr() as {ty}'
    assert fix_wide_byte_string_literals(src) == f'w!("HP") as {ty}'


def test_narrow_byte_string_is_left_alone():
    """`b"Eee PC\\0".as_ptr() as PCSTR` is the CORRECT spelling of a C `"..."`."""
    src = 'b"Eee PC\\0".as_ptr() as PCSTR'
    assert fix_wide_byte_string_literals(src) == src


def test_narrow_ascii_behind_a_wide_pointer_is_rewritten():
    """Strictly worse than misencoding: `"Sy"` read as UTF-16 is one code unit U+7953, so
    `RegOpenKeyExW` looks up a key nobody ever created. sysdm shipped 30-odd of these."""
    src = 'RegOpenKeyExW(k, b"LogEvent\\0".as_ptr() as *const u16, 0)'
    assert fix_wide_byte_string_literals(src) == 'RegOpenKeyExW(k, w!("LogEvent") as *const u16, 0)'


def test_narrow_path_with_escaped_backslashes_round_trips():
    src = 'b"System\\\\Control\\0".as_ptr() as *const u16'
    assert fix_wide_byte_string_literals(src) == 'w!("System\\\\Control") as *const u16'


def test_unterminated_wide_string_is_left_alone():
    src = 'b"A\\0B\\0".as_ptr() as PCWSTR'
    assert fix_wide_byte_string_literals(src) == src


def test_odd_byte_count_is_not_whole_u16_units_but_may_still_be_narrow():
    """`b"A\\0B"` has no terminator either way -- neither reading applies."""
    assert (
        fix_wide_byte_string_literals('b"A\\0B".as_ptr() as PCWSTR')
        == 'b"A\\0B".as_ptr() as PCWSTR'
    )
    # ...whereas an odd length that IS NUL-terminated is an ordinary narrow string
    assert fix_wide_byte_string_literals('b"AB\\0".as_ptr() as PCWSTR') == 'w!("AB") as PCWSTR'


def test_non_ascii_code_unit_is_left_for_a_human():
    """`w!` asserts ASCII, so a rewrite here would turn a silent bug into a build failure."""
    src = 'b"\\xe9\\x00\\x00\\x00".as_ptr() as PCWSTR'
    assert fix_wide_byte_string_literals(src) == src


def test_interior_nul_is_left_alone():
    """A REG_MULTI_SZ-style double-NUL block is not a single string and must not collapse."""
    src = 'b"A\\0\\0\\0B\\0\\0\\0".as_ptr() as PCWSTR'
    assert fix_wide_byte_string_literals(src) == src


def test_hex_escapes_decode():
    src = 'b"\\x41\\x00\\x42\\x00\\x00\\x00".as_ptr() as PCWSTR'
    assert fix_wide_byte_string_literals(src) == 'w!("AB") as PCWSTR'


def test_quote_and_backslash_are_re_escaped_for_the_str_literal():
    src = 'b"\\\\\\0\\"\\0\\0\\0".as_ptr() as PCWSTR'
    assert fix_wide_byte_string_literals(src) == 'w!("\\\\\\"") as PCWSTR'


def test_the_word_as_inside_the_payload_does_not_confuse_the_cast():
    src = 'b"a\\0 \\0a\\0s\\0 \\0b\\0\\0\\0".as_ptr() as PCWSTR'
    assert fix_wide_byte_string_literals(src) == 'w!("a as b") as PCWSTR'


def test_multiple_sites_on_one_line():
    src = (
        'V { long: b"H\\0P\\0\\0\\0".as_ptr() as PCWSTR, '
        'short: b"h\\0p\\0\\0\\0".as_ptr() as PCWSTR }'
    )
    assert fix_wide_byte_string_literals(src) == (
        'V { long: w!("HP") as PCWSTR, short: w!("hp") as PCWSTR }'
    )
