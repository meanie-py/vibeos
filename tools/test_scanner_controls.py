"""
Positive/negative controls for the gate scanners — "an oracle needs an oracle".

WHY THIS EXISTS. A scanner that reports `0` is telling you one of two things and
does not know which: the tree is clean, or the matcher is dead. For the twelve
lints whose baseline is EMPTY those two outputs are byte-identical, so a broken
scanner is indistinguishable from a healthy one — and it fails silently, in the
direction that looks like success.

That is not hypothetical. Three real instances:

  * scan_bugcheck_codes reported a clean 0 for six weeks while
    `KeBugCheckEx(0x35u32, 0,0,0,0)` sat in classpnp-rust. Its regex required the
    digits to be followed directly by `,`/`)`, and the Rust type suffix broke the
    match. It survived the entire §9.113 sweep that existed to find exactly that
    (field manual §9.120).
  * verify_constants was blind to `CTL_CODE()` macros, so a garbage-translated
    IOCTL went unchecked — the class that cost the arm64 ramdisk 0x64/0x23 grind
    (tools/rustconv/tests/test_verify_constants_ctlcode.py).
  * marshalgen's report iterated the generator's own flagged set and so inherited
    its blind spot (§9.115); its audit read a stale build artifact (§9.115a).

WHAT A CONTROL PROVES, AND WHAT IT DOES NOT. The positive control proves the
matcher is ALIVE — it still fires on a shape we know is bad. The negative control
proves it is not merely trigger-happy (a scanner that flags everything also
"catches" every bug, and is equally useless). Neither proves COMPLETENESS: this
harness would have caught `0x35u32` only because a suffixed literal is in the
fixture below. So fixtures are seeded from REAL incidents in the field manual —
the one corpus guaranteed to contain shapes we actually got wrong — rather than
from what the regex obviously handles.

Adding a scanner? Add a Control here. If a scanner genuinely cannot be driven by
a fixture directory, add it to UNCOVERED with a reason: test_every_lint_is_
accounted_for fails on anything that is in neither list, so coverage gaps are
LOUD rather than silently absent. (A coverage table that quietly omits entries
would be the same bug this file exists to prevent.)
"""

from __future__ import annotations

import subprocess
import sys
from dataclasses import dataclass, field
from pathlib import Path

import pytest

REPO = Path(__file__).resolve().parent.parent


@dataclass
class Control:
    """One scanner + a fixture it MUST flag and one it must NOT."""

    name: str  # lint-spec name (must match source_lint_specs)
    script: str  # repo-relative path to the scanner
    argv: list[str] = field(default_factory=list)  # extra flags before the path
    bad: dict[str, str] = field(default_factory=dict)  # relpath -> known-BAD content
    good: dict[str, str] = field(default_factory=dict)  # relpath -> known-GOOD content
    # Some scanners take explicit FILES (scan_silent_failures reads each arg with
    # read_bytes and skips directories); others take a root to walk.
    pass_files: bool = False
    # Only files matching this suffix are handed over when pass_files is set —
    # a crate fixture ships a Cargo.toml the scanner must not be asked to parse.
    file_suffix: str = ".rs"
    # scan_irql_balance / scan_ob_refcount ALWAYS build a whole-workspace
    # call-graph summary (their own comment: "call-graph facts are global"),
    # so each invocation costs ~50s no matter how small the fixture. Marked
    # so a local run can `-m "not slow"`; the gate still runs everything —
    # skipping a control by default would recreate the blind spot.
    slow: bool = False


# ── fixtures, each seeded from a real incident ───────────────────────────────

_BUGCHECK_BAD = """\
#[no_mangle]
pub unsafe extern "system" fn Boom() {
    // The literal that hid from this very scanner for six weeks (§9.120).
    KeBugCheckEx(0x35u32, 0, 0, 0, 0);
}
"""
_BUGCHECK_GOOD = """\
#[no_mangle]
pub unsafe extern "system" fn Boom() {
    KeBugCheckEx(ntkrnl_sys::bugcodes::INVALID_AFFINITY_SET, 0, 0, 0, 0);
}
"""

_ASSERT_UB_BAD = """\
pub unsafe fn paged_thing() {
    // PAGED_CODE() — a DBG-only diagnostic, lowered to UB (§9.120).
    if KeGetCurrentIrql() > APC_LEVEL {
        core::hint::unreachable_unchecked();
    }
}
"""
_ASSERT_UB_GOOD = """\
pub unsafe fn paged_thing() {
    if KeGetCurrentIrql() > APC_LEVEL {
        DbgPrint(b"NTDDK: Pageable code called at IRQL > APC_LEVEL\\n\\0".as_ptr() as *const i8);
    }
}
"""

_MAGIC_OFFSET_BAD = """\
pub unsafe fn read_field(p: *const u8) -> u64 {
    // Suffixed literal — the shape that evaded this scanner's \\b anchor.
    *(p.byte_add(0x18usize) as *const u64)
}
"""
_MAGIC_OFFSET_GOOD = """\
pub unsafe fn read_field(p: *const Thing) -> u64 {
    *((p as *const u8).byte_add(core::mem::offset_of!(Thing, field)) as *const u64)
}
"""

_LINT_KERNEL_BAD = """\
#[no_mangle]
pub unsafe extern "system" fn GetThing(OutValue: *mut u32) {
    // Unprotected output write — the captures(none) -> ud2 class.
    *OutValue = 42;
}
"""
_LINT_KERNEL_GOOD = """\
#[no_mangle]
pub unsafe extern "system" fn GetThing(OutValue: *mut u32) {
    core::ptr::write_volatile(OutValue, 42);
}
"""

_SILENT_FAIL_BAD = """\
pub unsafe fn spin() {
    let v: Result<u32, u32> = Err(1);
    let _x = v.unwrap();
    loop {}
}
"""
_SILENT_FAIL_GOOD = """\
pub unsafe fn spin() -> Result<u32, u32> {
    let v: Result<u32, u32> = Err(1);
    v
}
"""

_DANGLING_BAD = """\
pub unsafe fn name_ptr() -> *const u16 {
    // C's `WCHAR foo[] = L"x"` has real storage; the transmute temporary does not.
    let p = core::mem::transmute::<[u8; 4], [u16; 2]>(*b"a\\0b\\0").as_ptr();
    p
}
"""
_DANGLING_GOOD = """\
pub unsafe fn name_ptr() -> *const u16 {
    static NAME: [u16; 2] = [0x61, 0x00];
    NAME.as_ptr()
}
"""

_IRQL_BAD = """\
pub unsafe fn leaky(cond: bool) -> u32 {
    let mut old: u8 = 0;
    KeRaiseIrql(DISPATCH_LEVEL, &mut old);
    if cond {
        return 1;  // early return LEAKS the raise
    }
    KeLowerIrql(old);
    0
}
"""
_IRQL_GOOD = """\
pub unsafe fn balanced(cond: bool) -> u32 {
    let mut old: u8 = 0;
    KeRaiseIrql(DISPATCH_LEVEL, &mut old);
    let rc = if cond { 1 } else { 0 };
    KeLowerIrql(old);
    rc
}
"""

_OBREF_BAD = """\
pub unsafe fn leaky(cond: bool) -> u32 {
    ObReferenceObject(obj);
    if cond {
        return 1;  // early return LEAKS the reference
    }
    ObDereferenceObject(obj);
    0
}
"""
_OBREF_GOOD = """\
pub unsafe fn balanced(cond: bool) -> u32 {
    ObReferenceObject(obj);
    let rc = if cond { 1 } else { 0 };
    ObDereferenceObject(obj);
    rc
}
"""

CONTROLS: list[Control] = [
    Control(
        name="scan_bugcheck_codes",
        script="tools/scan_bugcheck_codes.py",
        argv=["--lint"],
        bad={"bad.rs": _BUGCHECK_BAD},
        good={"good.rs": _BUGCHECK_GOOD},
    ),
    Control(
        name="scan_assert_to_ub",
        script="tools/scan_assert_to_ub.py",
        argv=["--lint"],
        bad={"bad.rs": _ASSERT_UB_BAD},
        good={"good.rs": _ASSERT_UB_GOOD},
    ),
    Control(
        name="scan_magic_offsets",
        script="tools/scan_magic_offsets.py",
        argv=["--lint"],
        bad={"bad.rs": _MAGIC_OFFSET_BAD},
        good={"good.rs": _MAGIC_OFFSET_GOOD},
    ),
    Control(
        name="lint_kernel",
        script="tools/lint_kernel.py",
        bad={"bad.rs": _LINT_KERNEL_BAD},
        good={"good.rs": _LINT_KERNEL_GOOD},
        # Takes explicit files; handed a directory it silently scans nothing and
        # prints "No unprotected output pointer writes found." — a clean bill of
        # health for a tree it never looked at.
        pass_files=True,
    ),
    Control(
        name="scan_silent_failures",
        script="tools/scan_silent_failures.py",
        # --no-cache is REQUIRED, not tidiness: this scanner memoises results by
        # file hash, so without it a control could "pass" off a cache entry while
        # the matcher never ran — precisely the kind of false green this file
        # exists to prevent.
        argv=["--lint", "--no-cache"],
        bad={"bad.rs": _SILENT_FAIL_BAD},
        good={"good.rs": _SILENT_FAIL_GOOD},
        pass_files=True,
    ),
    Control(
        name="scan_dangling_transmute",
        script="tools/scan_dangling_transmute.py",
        argv=["--lint"],
        bad={"bad.rs": _DANGLING_BAD},
        good={"good.rs": _DANGLING_GOOD},
    ),
    Control(
        name="scan_irql_balance",
        script="tools/scan_irql_balance.py",
        argv=["--lint"],
        slow=True,
        bad={"bad.rs": _IRQL_BAD},
        good={"good.rs": _IRQL_GOOD},
    ),
    Control(
        name="scan_ob_refcount",
        script="tools/scan_ob_refcount.py",
        argv=["--lint"],
        slow=True,
        bad={"bad.rs": _OBREF_BAD},
        good={"good.rs": _OBREF_GOOD},
    ),
]

# Lints deliberately NOT fixture-driven, each with the reason. Anything absent
# from both this set and CONTROLS fails test_every_lint_is_accounted_for.
UNCOVERED: dict[str, str] = {
    # Already have dedicated suites (the prototypes this harness generalizes).
    "scan_wide_literals": "covered by tools/test_scan_wide_literals.py (12 tests)",
    "verify_c_signatures": "covered by tools/test_verify_c_signatures.py (9 tests)",
    "marshalgen_audit": "covered by tools/wow64gen/test_wow64gen.py",
    "verify_svc_numbering": "covered by tools/wow64gen/test_wow64gen.py",
    # Need a real build tree / toolchain, not a fixture dir.
    "shellcheck": "third-party binary; upstream-tested",
    "py-syntax": "stdlib compileall; liveness is not in question",
    "scan_nightly_surface": "no positional path arg; scans a fixed workspace root",
    "verify_conv_macros": "needs a populated sdk/include header tree",
    "verify_lost_exports": "needs matching .spec + crate layout",
    "verify_ffi": "needs the whole workspace to resolve cross-crate ABI",
    "verify_constants": "needs the C header catalog; see test_verify_constants_ctlcode.py",
    "verify_constants_enums": "same catalog dependency as verify_constants",
    "verify_tables": "needs generated tables from a build",
    "verify_match_consts": "needs the C header catalog",
    "verify_signatures": "needs build-<arch> generated headers",
    "scan_cc_mismatch": "needs cross-crate view of the workspace",
    "scan_duplicate_symbols": "operates on link artifacts, not source",
    "scan_const_contradictions": "needs the whole tree (finds one NAME with two VALUES)",
    "scan_inline_flag_literals": "needs the sdk/include header catalog to know the right value",
}


def _run(ctl: Control, root: Path) -> tuple[int, str]:
    if ctl.pass_files:
        targets = [str(p) for p in sorted(root.rglob(f"*{ctl.file_suffix}"))]
        assert targets, f"{ctl.name}: fixture produced no {ctl.file_suffix} files"
    else:
        targets = [str(root)]
    cmd = [sys.executable, str(REPO / ctl.script), *ctl.argv, *targets]
    p = subprocess.run(cmd, cwd=REPO, capture_output=True, text=True)
    return p.returncode, (p.stdout or "") + (p.stderr or "")


def _materialize(tmp_path: Path, files: dict[str, str]) -> Path:
    d = tmp_path / "fixture"
    d.mkdir(parents=True, exist_ok=True)
    for name, content in files.items():
        target = d / name
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_text(content)
    return d


@pytest.mark.parametrize(
    "ctl",
    [pytest.param(c, marks=pytest.mark.slow) if c.slow else c for c in CONTROLS],
    ids=lambda c: c.name,
)
def test_positive_control_scanner_is_not_blind(ctl: Control, tmp_path: Path):
    """The scanner MUST flag a shape we know is bad.

    A failure here means the scanner has gone blind — it will report a clean 0
    over a tree that still contains this exact bug.
    """
    root = _materialize(tmp_path, ctl.bad)
    rc, out = _run(ctl, root)
    # The finding must NAME the fixture. An earlier version of this assertion
    # accepted any non-empty output, and lint_kernel duly "passed" on its own
    # banner line ("No unprotected output pointer writes found.") — the harness
    # had the exact disease it exists to detect. Require a located hit.
    hits = [ln for ln in out.splitlines() if str(root) in ln]
    assert hits, (
        f"{ctl.name} did NOT flag its known-bad fixture — the matcher is dead, "
        f"and a clean report from it means nothing.\n"
        f"(exit={rc}; no output line referenced {root})\n--- output ---\n{out}"
    )


@pytest.mark.parametrize(
    "ctl",
    [pytest.param(c, marks=pytest.mark.slow) if c.slow else c for c in CONTROLS],
    ids=lambda c: c.name,
)
def test_negative_control_scanner_is_not_trigger_happy(ctl: Control, tmp_path: Path):
    """The scanner must NOT flag the corrected form of the same code.

    Without this, 'catches the bug' is satisfiable by flagging everything.
    """
    root = _materialize(tmp_path, ctl.good)
    _rc, out = _run(ctl, root)
    findings = [ln for ln in out.splitlines() if str(root) in ln or "fixture/" in ln]
    assert not findings, (
        f"{ctl.name} flagged its known-GOOD fixture — false positive.\n"
        f"--- findings ---\n" + "\n".join(findings)
    )


def test_every_lint_is_accounted_for():
    """Every gate lint has a control or a written reason it does not.

    This is the property that keeps the harness itself honest: a coverage table
    that silently omits a scanner would be the same failure mode the scanners
    have — looking complete while being blind.
    """
    sys.path.insert(0, str(REPO / "tests" / "qemu"))
    from cli.lints import source_lint_specs

    gate = {name for name, _cmd, _ff in source_lint_specs(REPO)}
    covered = {c.name for c in CONTROLS}
    accounted = covered | set(UNCOVERED)
    missing = sorted(gate - accounted)
    assert not missing, (
        "these gate lints have neither a control nor a documented reason: "
        f"{missing}\nAdd a Control, or add them to UNCOVERED with a reason."
    )


def test_control_names_match_real_lints():
    """Guard against a control that silently tests nothing because the lint was
    renamed — a control pointed at a dead name is itself a liar."""
    sys.path.insert(0, str(REPO / "tests" / "qemu"))
    from cli.lints import source_lint_specs

    gate = {name for name, _cmd, _ff in source_lint_specs(REPO)}
    stale = sorted({c.name for c in CONTROLS} - gate) + sorted(set(UNCOVERED) - gate)
    assert not stale, f"control/UNCOVERED entries name lints that no longer exist: {stale}"
