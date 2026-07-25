"""`_locale_t` is already a pointer — the model gave it a second star 73 times in one file.

The interesting half of this suite is what must NOT fire. mbcs.c carries 46 parameters where
the model's spelling differs from a mechanical mapping and the model is RIGHT (`size_t` vs
`usize`, `pthreadmbcinfo` vs `*mut threadmbcinfo`), sitting in the same signatures as the 73
real defects. Any rule loose enough to "fix" those is a regression.
"""

from __future__ import annotations

from pathlib import Path

import pytest

from rustconv.scanner import scan_file
from rustconv.signature_fidelity import repair_signatures
from rustconv.typedefs import pointer_typedefs_for_source

REPO_ROOT = Path(__file__).resolve().parents[3]
MBCS = REPO_ROOT / "dll/win32/msvcrt/mbcs.c"

PTRS = {"_locale_t": "core::ptr::null_mut()", "HANDLE": "core::ptr::null_mut()"}


def _scan(tmp_path: Path, c_text: str):
    src = tmp_path / "m.c"
    src.write_text(c_text)
    return scan_file(src)


# --------------------------------------------------------------------------- #
# the defect
# --------------------------------------------------------------------------- #


def test_invented_star_on_a_pointer_typedef_is_dropped(tmp_path):
    scan = _scan(tmp_path, "int f(unsigned int c, _locale_t locale) { return 0; }\n")
    rust = 'pub unsafe extern "C" fn f(c: u32, locale: *mut _locale_t) -> i32 { 0 }'
    out, findings = repair_signatures(rust, scan, PTRS)
    assert "locale: _locale_t" in out
    assert "*mut _locale_t" not in out
    assert len(findings) == 1 and findings[0].function == "f"


def test_const_star_is_dropped_too(tmp_path):
    scan = _scan(tmp_path, "int f(_locale_t locale) { return 0; }\n")
    rust = 'pub unsafe extern "C" fn f(locale: *const _locale_t) -> i32 { 0 }'
    out, _ = repair_signatures(rust, scan, PTRS)
    assert "locale: _locale_t" in out


def test_other_parameters_are_untouched(tmp_path):
    scan = _scan(tmp_path, "int f(char *s, _locale_t l) { return 0; }\n")
    rust = 'pub unsafe extern "C" fn f(s: *mut u8, l: *mut _locale_t) -> i32 { 0 }'
    out, findings = repair_signatures(rust, scan, PTRS)
    assert "s: *mut u8" in out and "l: _locale_t" in out
    assert len(findings) == 1


def test_a_mut_parameter_pattern_survives(tmp_path):
    scan = _scan(tmp_path, "int f(_locale_t l) { return 0; }\n")
    rust = 'pub unsafe extern "C" fn f(mut l: *mut _locale_t) -> i32 { 0 }'
    out, _ = repair_signatures(rust, scan, PTRS)
    assert "mut l: _locale_t" in out


# --------------------------------------------------------------------------- #
# what must NOT fire
# --------------------------------------------------------------------------- #


def test_the_expanded_spelling_is_left_alone(tmp_path):
    """`*mut localeinfo_struct` IS `_locale_t`. Different name, so out of scope — and right."""
    scan = _scan(tmp_path, "int f(_locale_t l) { return 0; }\n")
    rust = 'pub unsafe extern "C" fn f(l: *mut localeinfo_struct) -> i32 { 0 }'
    out, findings = repair_signatures(rust, scan, PTRS)
    assert out == rust and not findings


def test_a_c_pointer_to_the_typedef_keeps_its_star(tmp_path):
    """`_locale_t *` genuinely IS a double pointer. The C's own star is not invention."""
    scan = _scan(tmp_path, "int f(_locale_t *l) { return 0; }\n")
    rust = 'pub unsafe extern "C" fn f(l: *mut _locale_t) -> i32 { 0 }'
    out, findings = repair_signatures(rust, scan, PTRS)
    assert out == rust and not findings


def test_an_unproven_typedef_is_left_alone(tmp_path):
    """Not in the authority => not proven a pointer => not ours to touch."""
    scan = _scan(tmp_path, "int f(mystery_t m) { return 0; }\n")
    rust = 'pub unsafe extern "C" fn f(m: *mut mystery_t) -> i32 { 0 }'
    out, findings = repair_signatures(rust, scan, PTRS)
    assert out == rust and not findings


def test_benign_respellings_are_not_touched(tmp_path):
    """size_t/usize and threadmbcinfo*/pthreadmbcinfo are CORRECT and must survive."""
    scan = _scan(tmp_path, "int f(size_t n, threadmbcinfo *mbc) { return 0; }\n")
    rust = 'pub unsafe extern "C" fn f(n: usize, mbc: pthreadmbcinfo) -> i32 { 0 }'
    out, findings = repair_signatures(rust, scan, PTRS)
    assert out == rust and not findings


def test_a_function_with_no_c_counterpart_is_left_alone(tmp_path):
    scan = _scan(tmp_path, "int g(void) { return 0; }\n")
    rust = 'pub unsafe extern "C" fn helper(l: *mut _locale_t) -> i32 { 0 }'
    out, findings = repair_signatures(rust, scan, PTRS)
    assert out == rust and not findings


def test_param_count_mismatch_refuses(tmp_path):
    """`(void)` scans as one parameter; positions no longer line up, so nothing is provable."""
    scan = _scan(tmp_path, "int f(void) { return 0; }\n")
    rust = 'pub unsafe extern "C" fn f(l: *mut _locale_t) -> i32 { 0 }'
    out, findings = repair_signatures(rust, scan, PTRS)
    assert out == rust and not findings


def test_no_typedef_authority_is_a_no_op(tmp_path):
    scan = _scan(tmp_path, "int f(_locale_t l) { return 0; }\n")
    rust = 'pub unsafe extern "C" fn f(l: *mut _locale_t) -> i32 { 0 }'
    out, findings = repair_signatures(rust, scan, {})
    assert out == rust and not findings


def test_body_text_is_never_rewritten(tmp_path):
    """Only the parameter list. A cast inside a body is not a signature."""
    scan = _scan(tmp_path, "int f(_locale_t l) { return 0; }\n")
    rust = (
        'pub unsafe extern "C" fn f(l: *mut _locale_t) -> i32 {\n'
        "    let p = q as *mut _locale_t;\n"
        "    0\n}"
    )
    out, _ = repair_signatures(rust, scan, PTRS)
    assert "l: _locale_t" in out
    assert "q as *mut _locale_t" in out


# --------------------------------------------------------------------------- #
# against the real file
# --------------------------------------------------------------------------- #


@pytest.mark.skipif(not MBCS.is_file(), reason="msvcrt sources not present")
def test_mbcs_locale_t_is_proven_a_pointer():
    assert "_locale_t" in pointer_typedefs_for_source(MBCS)


@pytest.mark.skipif(not MBCS.is_file(), reason="msvcrt sources not present")
def test_idempotent_on_the_real_scan():
    """A second pass must find nothing — otherwise the rewrite is not a fixpoint."""
    scan = scan_file(MBCS)
    ptrs = pointer_typedefs_for_source(MBCS)
    rust = 'pub unsafe extern "C" fn _ismbcl2_l(c: u32, locale: *mut _locale_t) -> i32 { 0 }'
    once, first = repair_signatures(rust, scan, ptrs)
    twice, second = repair_signatures(once, scan, ptrs)
    assert first and not second
    assert once == twice
