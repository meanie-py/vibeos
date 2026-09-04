"""Tests for scan_wide_literals — both directions.

The must-NOT-fire half is not politeness here, it is the whole viability of the lint. Its first
fleet run produced 20 findings of which 19 were its own gaps (prose containing `L"..."`, comments
that spell `\\0` explicitly, double NUL terminators, and UNICODE_STRING buffers that correctly have
no terminator at all). A lint that reports 19 pieces of correct code alongside 1 real bug is worse
than no lint: it gets baselined wholesale and the real bug rides along inside the baseline.
"""

from __future__ import annotations

import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))

from scan_wide_literals import scan_file


def _scan(tmp_path: Path, src: str) -> list[str]:
    f = tmp_path / "t.rs"
    f.write_text(src)
    findings, _seen, _compared = scan_file(f)
    return findings


def _coverage(tmp_path: Path, src: str) -> tuple[int, int]:
    f = tmp_path / "t.rs"
    f.write_text(src)
    _findings, seen, compared = scan_file(f)
    return seen, compared


# --- MUST CATCH -------------------------------------------------------------------------


def test_catches_the_services_key_truncation(tmp_path: Path):
    """THE motivating bug, verbatim: [u16; 33] silently dropped the final 's', so services-rust
    opened `...\\Service` and every registration failed with ERROR_FILE_NOT_FOUND."""
    out = _scan(
        tmp_path,
        '// L"System\\\\CurrentControlSet\\\\Services" — NUL-terminated.\n'
        "static SERVICES_KEY: [u16; 33] = [\n"
        + ", ".join(f"b'{c}' as u16" for c in "System")
        + ", b'\\\\' as u16, "
        + ", ".join(f"b'{c}' as u16" for c in "CurrentControlSet")
        + ", b'\\\\' as u16, "
        + ", ".join(f"b'{c}' as u16" for c in "Service")
        + ", 0,\n];\n",
    )
    assert len(out) == 1, out
    assert "TRUNCATED" in out[0]
    assert "dropped 's'" in out[0]
    assert "need 34 elements" in out[0]


def test_catches_missing_nul_when_the_comment_promises_one(tmp_path: Path):
    out = _scan(
        tmp_path,
        '// L"Services" — NUL-terminated.\n'
        "static K: [u16; 8] = [" + ", ".join(f"b'{c}' as u16" for c in "Services") + "];\n",
    )
    assert len(out) == 1 and "NO NUL terminator" in out[0], out


def test_catches_wrong_content(tmp_path: Path):
    out = _scan(
        tmp_path,
        '// L"Widget"\nstatic K: [u16; 7] = ['
        + ", ".join(f"b'{c}' as u16" for c in "Wodget")
        + ", 0];\n",
    )
    assert len(out) == 1 and "Wodget" in out[0], out


# --- MUST NOT FIRE ----------------------------------------------------------------------


def test_correct_literal_is_silent(tmp_path: Path):
    assert (
        _scan(
            tmp_path,
            '// L"Services" — NUL-terminated.\n'
            "static K: [u16; 9] = [" + ", ".join(f"b'{c}' as u16" for c in "Services") + ", 0];\n",
        )
        == []
    )


def test_unicode_string_buffer_without_nul_is_silent(tmp_path: Path):
    """iomgr's `\\??\\` prefixes are UNICODE_STRING Buffers: Length carries the size and a
    terminator is genuinely unnecessary. A blanket "must end in NUL" rule flagged five of these,
    all correct."""
    src = (
        '// L"\\\\??\\\\" — 4 wide chars, no null needed (Length == MaximumLength)\n'
        "static P: [u16; 4] = [0x5C, 0x3F, 0x3F, 0x5C];\n"
    )
    assert _scan(tmp_path, src) == []


def test_prose_mentioning_L_syntax_is_not_an_intent(tmp_path: Path):
    """hal's halpnpdd.rs really says: `Rust doesn't have L"...", so spell them out`. Reading that
    as the spec reported three correct literals as mismatches."""
    src = (
        '// UTF-16 string literals — Rust doesn\'t have `L"..."`, so spell them out.\n'
        "// All NUL-terminated.\n"
        "const ID: [u16; 3] = [b'A' as u16, b'B' as u16, 0];\n"
    )
    _seen, compared = _coverage(tmp_path, src)
    assert _scan(tmp_path, src) == []
    assert compared == 0, "an ellipsis is not a specification; it must not count as coverage"


def test_abbreviated_path_intent_is_skipped(tmp_path: Path):
    """`L"\\\\Registry\\\\Machine\\\\...\\\\Version"` states nothing checkable."""
    src = (
        "// L\"\\\\Registry\\\\Machine\\\\...\\\\Version\"\nconst K: [u16; 2] = [b'A' as u16, 0];\n"
    )
    assert _scan(tmp_path, src) == []


def test_comment_spelling_the_nul_explicitly_is_silent(tmp_path: Path):
    """rtl-rxact writes `// L"RXACT\\0" / L"Log\\0"`. The \\0 is the same terminator the array
    already has; not stripping it on the intent side reported both as TRUNCATED."""
    src = (
        '// L"RXACT\\0" / L"Log\\0" (RtlInitUnicodeString scans for the NUL).\n'
        "static RXACT_NAME: [u16; 6] = [0x0052, 0x0058, 0x0041, 0x0043, 0x0054, 0x0000];\n"
    )
    assert _scan(tmp_path, src) == []


def test_double_nul_is_silent(tmp_path: Path):
    """hal usage.rs really ends `0x0000, 0x0000`. Harmless — RtlInitUnicodeString stops at the
    first — but a one-NUL strip reported it as a content mismatch."""
    src = "// L\"AB\"\nconst K: [u16; 4] = [b'A' as u16, b'B' as u16, 0x0000, 0x0000];\n"
    assert _scan(tmp_path, src) == []


def test_alternatives_in_one_comment_match_either(tmp_path: Path):
    """`// L"DependOnService" / L"DependOnGroup"` documents a family; demanding the first would
    report the second as wrong."""
    src = (
        '// L"DependOnService" / L"DependOnGroup"\n'
        "static V: [u16; 14] = [" + ", ".join(f"b'{c}' as u16" for c in "DependOnGroup") + ", 0];\n"
    )
    assert _scan(tmp_path, src) == []


def test_unrelated_comment_above_a_blank_line_is_not_the_intent(tmp_path: Path):
    """Only a CONTIGUOUS `//` block is this literal's spec."""
    src = "// L\"Something\"\n\nstatic K: [u16; 3] = [b'A' as u16, b'B' as u16, 0];\n"
    _seen, compared = _coverage(tmp_path, src)
    assert compared == 0 and _scan(tmp_path, src) == []


def test_coverage_is_reported(tmp_path: Path):
    """A literal with no intent comment is a blind spot, not a pass — it must not count as
    'compared'. "0 findings" over 0 comparisons is the green that means nothing."""
    src = "static K: [u16; 3] = [b'A' as u16, b'B' as u16, 0];\n"
    seen, compared = _coverage(tmp_path, src)
    assert seen == 1 and compared == 0


# --- the byte-string-cast half ------------------------------------------------------------
#
# The array rule above needs an intent comment to check against. This one does not: a byte
# string cast to a UTF-16 pointer is a self-contained claim about its own bytes, so all three
# checks are decidable. Both shipped bugs below are real — msvcrt's `_wsystem` and
# `msvcrt_search_executable`, the latter silently breaking PATH search for every spawn.


def test_catches_ascii_bytes_cast_to_a_wide_pointer(tmp_path: Path):
    """msvcrt process.c's `_wsystem`: `L" /c "` written as plain ASCII. `wcscat` reads
    `' '`+`'/'` as ONE code unit 0x2F20 and then walks past the end of a 5-byte static."""
    out = _scan(tmp_path, 'wcscat(cmd, b" /c \\0".as_ptr() as *const wchar_t);\n')
    assert len(out) == 1, out
    assert "ODD length" in out[0] or "8-bit text" in out[0], out


def test_catches_single_byte_terminator(tmp_path: Path):
    """msvcrt process.c's `msvcrt_search_executable`: `L"PATH"` interleaved correctly but with
    ONE trailing zero byte, so the final code unit's high half comes from the next object."""
    out = _scan(tmp_path, 'env = _wgetenv(b"P\\0A\\0T\\0H\\0\\0".as_ptr() as *const wchar_t);\n')
    assert len(out) == 1, out
    assert "ODD length" in out[0], out


def test_catches_odd_length(tmp_path: Path):
    out = _scan(tmp_path, 'f(b"M\\0M\\0\\0".as_ptr() as *const u16);\n')
    assert len(out) == 1, out
    assert "ODD length" in out[0]


def test_correct_wide_byte_string_is_silent(tmp_path: Path):
    assert (
        _scan(tmp_path, 'wcscat(cmd, b" \\0/\\0c\\0 \\0\\0\\0".as_ptr() as *const wchar_t);\n')
        == []
    )


def test_catches_the_alias_spelling(tmp_path: Path):
    """`as LPCWSTR` is the same bug as `as *const u16`. The lint was blind to it for one pass,
    reported clean, and 111 further sites were sitting behind the alias."""
    out = _scan(tmp_path, 'g(b"\\\\Device\\\\CdRom\\0".as_ptr() as LPCWSTR);\n')
    assert len(out) == 1, out


def test_catches_the_transmute_spelling(tmp_path: Path):
    """`transmute(b"...".as_ptr())` reaches the same wrong pointer with no `as` in sight."""
    out = _scan(tmp_path, 'let p = core::mem::transmute(b"All Users\\0".as_ptr());\n')
    assert len(out) == 1, out


def test_a_comment_describing_the_bug_is_not_the_bug(tmp_path: Path):
    """Several crates carry a comment quoting the bad form verbatim while FIXING it. Matching
    inside those turns a fix into a finding — which is how this lint would get baselined."""
    src = (
        '// (replacing incorrect b"...".as_ptr() as *mut u16 casts)\n'
        "static OBJECT_NAME_STR: [u16; 2] = [b'O' as u16, 0];\n"
    )
    assert _scan(tmp_path, src) == []


def test_byte_string_not_cast_to_a_wide_pointer_is_silent(tmp_path: Path):
    """An 8-bit string used as an 8-bit string is correct and extremely common."""
    assert _scan(tmp_path, 'puts(b"hello\\0".as_ptr() as *const i8);\n') == []


# --- the C source spelling swallowed whole, `L"..."` and all ---------------------------------


def test_l_prefixed_source_text_is_a_finding(tmp_path: Path):
    """netcfgx translated `swprintf(szBuffer, L"%u", Gw.Metric)` as `b"L\\"%u\\"\\0"`, so the
    format string at runtime is the six characters `L"%u"`. No cast to key off — the site is
    `as *const _` — and the bytes are consistently 8-bit, just 8-bit of the wrong text, so
    every encoding rule reads it as fine."""
    out = _scan(tmp_path, 'swprintf(b.as_mut_ptr(), b"L\\"%u\\"\\0".as_ptr() as *const _, n);\n')
    assert len(out) == 1, out
    assert "SOURCE TEXT" in out[0]


def test_l_prefix_rule_needs_both_ends(tmp_path: Path):
    """Anchored negatives: a string that merely STARTS with `L"`, or one that merely ends with
    a quote, is ordinary data. Only the matched pair is the translator having copied the C."""
    for src in (
        'f(b"L\\"unclosed\\0".as_ptr() as LPCWSTR);\n',
        'f(b"trailing\\"\\0".as_ptr() as LPCWSTR);\n',
    ):
        assert not [x for x in _scan(tmp_path, src) if "SOURCE TEXT" in x], src


def test_l_prefix_rule_ignores_commented_out_c(tmp_path: Path):
    """Translated files keep the original C alongside as `//` reference blocks, which are full
    of genuine `L"..."`. Flagging those would bury the real finding in its own source."""
    assert _scan(tmp_path, '// swprintf(szBuffer, b"L\\"%u\\"\\0", n);\n') == []


# --- an interior u16 NUL: even, terminated, interleaved — and still wrong --------------------


def test_double_interleaved_literal_is_a_finding(tmp_path: Path):
    """The shape a repair script produces when it treats an already-UTF-16 literal as 8-bit
    text and interleaves it a second time. It passes every other check in this rule — even
    length, proper `\\0\\0` terminator, every second byte zero — and decodes to `I`. This is
    not hypothetical: that script wrote raw NUL bytes into netcfgx's netinstall.rs and the
    scanner reported the file clean."""
    out = _scan(tmp_path, 'f(b"I\\0\\0\\0n\\0\\0\\0f\\0\\0\\0\\0\\0".as_ptr() as LPCWSTR);\n')
    assert len(out) == 1, out
    assert "INTERIOR" in out[0]


def test_multi_sz_shape_is_deliberately_not_exempt(tmp_path: Path):
    """REG_MULTI_SZ legitimately holds interior NULs and ends with two u16 NULs, so exempting
    that shape looks obviously right. It is not, and this test pins the measurement that
    settled it: the tree has ZERO multi-sz byte-cast literals and THREE double-interleaved
    ones, two of which end in four zero bytes. The exemption would have hidden every real
    instance to protect a case that does not occur."""
    out = _scan(tmp_path, 'f(b"a\\0\\0\\0b\\0\\0\\0\\0\\0".as_ptr() as LPCWSTR);\n')
    assert len(out) == 1 and "INTERIOR" in out[0], out


def test_correct_wide_literal_has_no_interior_nul(tmp_path: Path):
    """Anchor: the CORRECT spelling of the same string must stay silent, so the finding above
    is about the doubling and not about the rule firing on everything."""
    assert _scan(tmp_path, 'f(b"I\\0n\\0f\\0\\0\\0".as_ptr() as LPCWSTR);\n') == []


# --- element-type spellings and suffixed literals ---------------------------------------------


def test_wchar_element_type_is_examined(tmp_path: Path):
    """`[WCHAR; N]` is by far the commonest spelling in the tree and was invisible for a long time.

    The rule matched `[u16; N]` only, so it reported clean over ~2x more declarations than it
    actually looked at (946 examined against 1975 present).
    """
    src = (
        '// L"System"\n'
        "static K: [WCHAR; 7] = [b'S' as WCHAR, b'y' as WCHAR, b's' as WCHAR, b't' as WCHAR,\n"
        "    b'e' as WCHAR, b'm' as WCHAR, 0];\n"
    )
    assert _scan(tmp_path, src) == []
    _seen, compared = _coverage(tmp_path, src)
    assert compared == 1, "a WCHAR array must be compared against its intent, not skipped"


def test_wchar_truncation_is_caught(tmp_path: Path):
    """The must-FIRE direction for the alias spelling: same bug, same verdict."""
    src = (
        '// L"System"\n'
        "static K: [WCHAR; 6] = [b'S' as WCHAR, b'y' as WCHAR, b's' as WCHAR, b't' as WCHAR,\n"
        "    b'e' as WCHAR, 0];\n"
    )
    findings = _scan(tmp_path, src)
    assert len(findings) == 1, findings
    # The message must name the REAL element type; it hardcoded `u16` for every array.
    assert "[WCHAR; 6]" in findings[0] and "TRUNCATED" in findings[0], findings[0]


def test_tchar_element_type_is_examined(tmp_path: Path):
    """TCHAR is included even though it is NOT always 16-bit (c_char in ntdll-sys).

    Both checks are width-agnostic — decoded ordinals vs the comment, element count vs the
    declared bound — so an 8-bit TCHAR array is checked just as correctly.
    """
    src = "// L\"ab\"\nstatic K: [TCHAR; 3] = [b'a' as TCHAR, b'b' as TCHAR, 0];\n"
    assert _scan(tmp_path, src) == []
    _seen, compared = _coverage(tmp_path, src)
    assert compared == 1


def test_suffixed_literals_do_not_invent_elements(tmp_path: Path):
    """`0x0069u16` is ONE element, not `0x0069` followed by `16`.

    Without consuming the suffix the hex alternative stops at `0x0069`, the scan resumes at
    `u16`, and the bare-decimal alternative matches the `16` — inventing an element with value
    16 and inflating the count this lint exists to check. dnsrslvr's IP6_ARPA decoded as
    'i\\x10p6.arpa.\\x10' for exactly this reason.
    """
    src = '// L"ip"\nstatic K: [u16; 3] = [0x0069u16, 0x0070, 0u16];\n'
    assert _scan(tmp_path, src) == []


def test_suffixed_literals_still_catch_a_real_truncation(tmp_path: Path):
    """Consuming the suffix must not blunt the rule — the same array one short still fires."""
    src = '// L"ip6"\nstatic K: [u16; 3] = [0x0069u16, 0x0070, 0u16];\n'
    findings = _scan(tmp_path, src)
    assert len(findings) == 1 and '"ip"' in findings[0]


# --- use-site-gated unterminated rule ----------------------------------------------------------


def test_unterminated_passed_to_a_nul_scanner_is_a_finding(tmp_path: Path):
    """THE must-fire case. RtlInitUnicodeString computes Length BY scanning for the NUL.

    smss-csrss shipped eight of these (`\\NLS\\NlsSectionUnicode` and friends): the C wrote
    `RtlInitUnicodeString(&n, L"\\NLS\\NlsSectionUnicode")`, whose literal carries a terminator for
    free, and the hand-encoding dropped it.
    """
    src = (
        "static NLS_X: [u16; 3] = [b'a' as u16, b'b' as u16, b'c' as u16];\n"
        "fn f() { RtlInitUnicodeString(&mut n, NLS_X.as_ptr()); }\n"
    )
    findings = _scan(tmp_path, src)
    assert len(findings) == 1, findings
    assert "NO NUL terminator" in findings[0] and '"abc"' in findings[0]


def test_terminated_array_is_silent(tmp_path: Path):
    src = (
        "static NLS_X: [u16; 4] = [b'a' as u16, b'b' as u16, b'c' as u16, 0];\n"
        "fn f() { RtlInitUnicodeString(&mut n, NLS_X.as_ptr()); }\n"
    )
    assert _scan(tmp_path, src) == []


def test_bounded_callee_is_silent(tmp_path: Path):
    """`wcsncmp(p, x.as_ptr(), 3)` reads three units and stops — no terminator is needed."""
    src = (
        "static PROBE: [u16; 3] = [b'a' as u16, b'b' as u16, b'c' as u16];\n"
        "fn f() { wcsncmp(p, PROBE.as_ptr(), 3); }\n"
    )
    assert _scan(tmp_path, src) == []


def test_bounded_callee_is_found_through_a_nested_argument(tmp_path: Path):
    """The bounded callee must be located depth-aware, not by a paren-free lookbehind.

    npfs writes `RtlCopyMemory(addr_of_mut!((*Info).FileSystemName) as PVOID, X.as_ptr(), n)`.
    A gap pattern that forbids parens never reaches back to `RtlCopyMemory`, so a correct,
    deliberately-unterminated literal reads as unbounded and gets reported.
    """
    src = (
        "static NPFS_FS_NAME: [u16; 4] = [b'N' as u16, b'P' as u16, b'F' as u16, b'S' as u16];\n"
        "fn f() { RtlCopyMemory(addr_of_mut!((*Info).FileSystemName) as PVOID,\n"
        "    NPFS_FS_NAME.as_ptr(), n); }\n"
    )
    assert _scan(tmp_path, src) == []


def test_unicode_string_buffer_field_is_silent(tmp_path: Path):
    """`Buffer:` beside an explicit Length is correct by construction — wmi declares four."""
    src = (
        "static NAME_BUF: [u16; 3] = [b'a' as u16, b'b' as u16, b'c' as u16];\n"
        "pub static mut NAME: _UNICODE_STRING = _UNICODE_STRING {\n"
        "    Length: 6, MaximumLength: 6, Buffer: NAME_BUF.as_ptr() as *mut u16,\n"
        "};\n"
    )
    assert _scan(tmp_path, src) == []


def test_numeric_table_is_silent(tmp_path: Path):
    """Most unterminated arrays were never strings. Flagging tables is how a lint gets ignored."""
    src = (
        "static ZIPMASK: [u16; 5] = [0x0001, 0x0003, 0x0007, 0x000f, 0x001f];\n"
        "fn f() { g(ZIPMASK.as_ptr()); }\n"
    )
    assert _scan(tmp_path, src) == []


def test_length_dependent_use_vetoes_the_finding(tmp_path: Path):
    """Appending a terminator here would CHANGE the length the code computes, not fix anything.

    rtl-actctx's wine_trailerW is compared using `size_of_val()` AND `.len()`.
    """
    src = (
        "static TRAILER: [u16; 3] = [b'a' as u16, b'b' as u16, b'c' as u16];\n"
        "fn f() { if n == core::mem::size_of_val(&TRAILER) { g(TRAILER.as_ptr()); } }\n"
    )
    assert _scan(tmp_path, src) == []


def test_name_declared_twice_in_a_file_is_not_attributed(tmp_path: Path):
    """`fmtW` is declared twice inside rtl-actctx/src/actctx.rs alone.

    Matching uses by name across a file would credit one array's uses to the other, so the
    verdict is meaningless — skip rather than guess.
    """
    src = (
        "fn a() { static fmtW: [u16; 3] = [b'a' as u16, b'b' as u16, b'c' as u16];\n"
        "    let _ = fmtW; }\n"
        "fn b() { static fmtW: [u16; 4] = [b'a' as u16, b'b' as u16, b'c' as u16, 0];\n"
        "    RtlInitUnicodeString(&mut n, fmtW.as_ptr()); }\n"
    )
    assert _scan(tmp_path, src) == []
