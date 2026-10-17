"""Unit tests for scan_dropped_asserts.

The scanner's whole value is a NEGATIVE claim — "this Rust twin checks less than
its C original" — so both failure directions are expensive and both are pinned
here against a synthetic tree:

  * a false negative makes the gate decorative, which is exactly the state the
    tree was in when MI_MAKE_HARDWARE_PTE_USER shipped 0 of the C's 4 asserts;
  * a false positive on a legitimately cfg-split pair gets the gate ignored.

The comment-stripping tests are not hygiene. rustconv parks the original C in
trailing comments, so an unstripped counter scores a translated file by the C it
quotes and every dropped assert reads as present — the scanner would report a
clean tree for precisely the population it exists to police. The line-number test
pins a defect this scanner actually shipped: stripping block comments without
preserving their newlines shifted every reported line past the first one.
"""

from __future__ import annotations

import importlib.util
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent
_spec = importlib.util.spec_from_file_location(
    "scan_dropped_asserts", REPO / "tools" / "scan_dropped_asserts.py"
)
assert _spec is not None and _spec.loader is not None
sda = importlib.util.module_from_spec(_spec)
sys.modules["scan_dropped_asserts"] = sda
_spec.loader.exec_module(sda)


def build(tmp_path: Path, c_src: str, rs_src: str) -> list[tuple[Path, int, str]]:
    inc = tmp_path / "inc"
    ws = tmp_path / "ws"
    inc.mkdir()
    ws.mkdir()
    (inc / "miarm.h").write_text(c_src)
    (ws / "bridge.rs").write_text(rs_src)
    return sda.scan(header_roots=(inc,), rust_ws=ws, repo=tmp_path)


C_GUARDED = """
FORCEINLINE
VOID
MI_MAKE_HARDWARE_PTE_USER(IN PMMPTE NewPte, IN ULONG_PTR ProtectionMask)
{
    ASSERT(ProtectionMask & MM_PROTECT_ACCESS);
    ASSERT((ProtectionMask & MM_PROTECT_SPECIAL) != MM_GUARDPAGE);
    NewPte->u.Hard.Valid = TRUE;
}
"""


def test_reports_a_fully_dropped_assert(tmp_path):
    """The #39 shape: every assert gone from the Rust twin."""
    findings = build(
        tmp_path,
        C_GUARDED,
        "pub unsafe fn MI_MAKE_HARDWARE_PTE_USER(p: usize) { set_valid(p); }",
    )
    assert len(findings) == 1
    assert "MI_MAKE_HARDWARE_PTE_USER: C has 2 assert(s), Rust has 0" in findings[0][2]


def test_reports_a_PARTIAL_drop(tmp_path):
    """'Fewer than the C', not 'none' — a partial drop is the same bug class."""
    findings = build(
        tmp_path,
        C_GUARDED,
        """pub unsafe fn MI_MAKE_HARDWARE_PTE_USER(p: usize) {
               ASSERT!(mask & MM_PROTECT_ACCESS != 0);
               set_valid(p);
           }""",
    )
    assert len(findings) == 1
    assert "C has 2 assert(s), Rust has 1" in findings[0][2]


def test_parity_is_silent(tmp_path):
    findings = build(
        tmp_path,
        C_GUARDED,
        """pub unsafe fn MI_MAKE_HARDWARE_PTE_USER(p: usize) {
               ASSERT(a); ASSERT(b);
           }""",
    )
    assert findings == []


def test_rust_side_comments_do_not_count(tmp_path):
    """An assert QUOTED in a trailing C comment is not an assert that runs.

    This is the rustconv output shape, and counting it would make the scanner
    certify the population it exists to police.
    """
    findings = build(
        tmp_path,
        C_GUARDED,
        """pub unsafe fn MI_MAKE_HARDWARE_PTE_USER(p: usize) {
               // ASSERT(ProtectionMask & MM_PROTECT_ACCESS);
               /* ASSERT((ProtectionMask & MM_PROTECT_SPECIAL) != MM_GUARDPAGE); */
               set_valid(p);
           }""",
    )
    assert len(findings) == 1
    assert "Rust has 0" in findings[0][2]


def test_c_side_comments_do_not_count(tmp_path):
    """Symmetric: a commented-out C assert is not a check the Rust owes."""
    findings = build(
        tmp_path,
        """
FORCEINLINE VOID MiFoo(VOID)
{
    /* ASSERT(retired); */
    // ASSERT(also_retired);
    Work();
}
""",
        "pub unsafe fn MiFoo() { work(); }",
    )
    assert findings == []


def test_string_literal_is_not_an_assert(tmp_path):
    findings = build(
        tmp_path,
        C_GUARDED,
        """pub unsafe fn MI_MAKE_HARDWARE_PTE_USER(p: usize) {
               ASSERT(a);
               DbgPrint(b"ASSERT(x) failed\\0".as_ptr());
               ASSERT(b);
           }""",
    )
    assert findings == []


def test_line_number_survives_a_block_comment(tmp_path):
    """Stripping a comment must keep its newlines or every line drifts."""
    c = "/*\n a\n multi\n line\n banner\n */\n" + C_GUARDED
    findings = build(tmp_path, c, "pub unsafe fn MI_MAKE_HARDWARE_PTE_USER(p: usize) {}")
    assert len(findings) == 1
    reported = findings[0][1]
    wanted = c.splitlines().index("FORCEINLINE") + 1  # 1-based, past the banner
    assert reported == wanted, f"reported line {reported}, FORCEINLINE is at {wanted}"


def test_a_c_prototype_is_not_a_definition(tmp_path):
    """A declaration has no body to drop asserts from."""
    findings = build(
        tmp_path,
        "FORCEINLINE VOID MiFoo(VOID);\n",
        "pub unsafe fn MiFoo() { work(); }",
    )
    assert findings == []


def test_unpaired_c_function_is_not_reported(tmp_path):
    """No Rust twin means nothing was translated, so nothing was dropped."""
    findings = build(tmp_path, C_GUARDED, "pub fn unrelated() {}")
    assert findings == []


def test_cfg_split_twins_score_by_the_best_of_them(tmp_path):
    """An arch variant that legitimately checks less must not report while its
    sibling still carries the assert — otherwise every cfg pair floods."""
    findings = build(
        tmp_path,
        C_GUARDED,
        """
        #[cfg(target_arch = "aarch64")]
        pub unsafe fn MI_MAKE_HARDWARE_PTE_USER(p: usize) { set_valid(p); }

        #[cfg(not(target_arch = "aarch64"))]
        pub unsafe fn MI_MAKE_HARDWARE_PTE_USER(p: usize) { ASSERT(a); ASSERT(b); }
        """,
    )
    assert findings == []


def test_cargo_target_dir_is_pruned(tmp_path):
    """sdk/lib/rust/target is build output, and cargo deletes directories under
    a walk in progress. A stale copy there must not satisfy the pairing."""
    inc = tmp_path / "inc"
    ws = tmp_path / "ws"
    (ws / "target").mkdir(parents=True)
    inc.mkdir()
    (inc / "miarm.h").write_text(C_GUARDED)
    (ws / "target" / "stale.rs").write_text(
        "pub unsafe fn MI_MAKE_HARDWARE_PTE_USER(p: usize) { ASSERT(a); ASSERT(b); }"
    )
    (ws / "bridge.rs").write_text(
        "pub unsafe fn MI_MAKE_HARDWARE_PTE_USER(p: usize) { set_valid(p); }"
    )
    findings = sda.scan(header_roots=(inc,), rust_ws=ws, repo=tmp_path)
    assert len(findings) == 1, "the pruned target copy must not mask the real drop"


def test_a_wrapper_inherits_its_sited_body(tmp_path):
    """`ExAcquirePushLockExclusive` forwards to `ExAcquirePushLockExclusiveSited`, which
    the export shims call directly; the C's assert lives in the Sited body."""
    findings = build(
        tmp_path,
        """
FORCEINLINE VOID MiFoo(VOID)
{
    ASSERT(locked);
}
""",
        """pub unsafe fn MiFoo() { MiFooSited(0) }
           pub unsafe fn MiFooSited(site: usize) { crate::ASSERT!(locked()); }""",
    )
    assert findings == []
