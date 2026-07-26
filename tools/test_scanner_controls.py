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

# scan_bugcheck_stubs. Seeded from fastfat 2026-07-26: three boots, three stubs
# (FatGenerateFileIdFromFcb -> FatOpenVolume -> ...), each a full build+boot to
# discover something the tree already knew. The GOOD case pins the two ways this
# scanner must NOT fire, both of which it got wrong on its first run:
#   * a faithful wrapper for a bugcheck MACRO is a real implementation, not a stub
#     (fastfat's FatBugCheck forwards A/B/C to KeBugCheckEx);
#   * the STUB_BUGCHECK #define itself contains the marker, and must not be
#     attributed to whatever function precedes it.
_STUBS_HEADER = """\
#define FooBugCheck(A,B,C) { KeBugCheckEx(0x23, __LINE__, A, B, C); }
#define FooSizeOf(X,OUT)   { *(OUT) = sizeof(*(X)); }
"""
_STUBS_BAD_C = """\
#define STUB_BUGCHECK() do { KeBugCheckEx(0x23, 0xDEAD0000, 0, 0, 0); } while(0)

/* A real wrapper for the FooBugCheck macro — forwards its args. NOT a stub. */
VOID FooBugCheck(ULONG_PTR A, ULONG_PTR B, ULONG_PTR C)
{
    KeBugCheckEx(0x23, 0x00010000 | (ULONG)(__LINE__), A, B, C);
}

/* Genuinely untranslated, and Rust declares it => reachable crash. */
NTSTATUS FooDismount(PVOID a) { (void)a; STUB_BUGCHECK(); return 0; }

/* A MACRO stubbed as a function: the extern decl can never resolve. */
ULONG FooSizeOf(PVOID a, PVOID b) { (void)a;(void)b; STUB_BUGCHECK(); return 0; }

/* Bugcheck stub that NO Rust module declares — unreachable, so not a finding. */
NTSTATUS FooNeverCalled(PVOID a) { (void)a; STUB_BUGCHECK(); return 0; }

/* Declared by Rust but never CALLED: the decl is what made this stub exist, and the
   real function is already translated as FooOrphanReal. ORPHAN_DECL, not a crash. */
NTSTATUS foo_orphan(PVOID a) { (void)a; STUB_BUGCHECK(); return 0; }
"""
_STUBS_BAD_RS = """\
extern "system" {
    fn FooDismount(a: *mut u8) -> i32;
    fn FooSizeOf(a: *mut u8, b: *mut u32);
    fn FooBugCheck(a: usize, b: usize, c: usize);
    fn foo_orphan(a: *mut u8) -> i32;
}

// FooDismount is genuinely reachable: something calls it. This is the case that
// separates a live crash from a dead declaration.
pub unsafe fn caller(a: *mut u8) -> i32 {
    // A commented-out call must NOT count as a call site — fastfat keeps the whole
    // original C body commented beside each translation:  foo_orphan(a);
    FooDismount(a)
}
"""
_STUBS_GOOD_C = """\
#define STUB_BUGCHECK() do { KeBugCheckEx(0x23, 0xDEAD0000, 0, 0, 0); } while(0)

/* Real wrapper — still must not be flagged. */
VOID FooBugCheck(ULONG_PTR A, ULONG_PTR B, ULONG_PTR C)
{
    KeBugCheckEx(0x23, 0x00010000 | (ULONG)(__LINE__), A, B, C);
}

/* Both former stubs are now real Rust; only unreachable ones remain here. */
NTSTATUS FooNeverCalled(PVOID a) { (void)a; STUB_BUGCHECK(); return 0; }
"""
_STUBS_GOOD_RS = """\
extern "system" {
    fn FooBugCheck(a: usize, b: usize, c: usize);
}
#[no_mangle]
pub unsafe extern "system" fn FooDismount(_a: *mut u8) -> i32 { 0 }
"""

# Two lanes both appending `### 9.122` at the tail on the same day. This exact
# shape has now happened six times; the merge that added this control WAS one.
_DOC_SECTIONS_BAD = """\
# Manual

### 9.121 (2026-07-25) Something earlier

Body.

### 9.122 (2026-07-25) Lane A appended here

Body.

### 9.122 (2026-07-24) Lane B independently picked the same number

Body.
"""

# The near-miss that broke this lint's FIRST regex: `9.6c`, `9.6c.1` and `9.6c.2`
# are three DISTINCT sections, and a pattern stopping at the letter suffix reports
# them as one triplicated number. The good fixture must contain the shape the
# scanner is most likely to over-match, or "no false positives" is untested.
_DOC_SECTIONS_GOOD = """\
# Manual

### 9.6c (2026-07-01) A lettered section

Body.

#### 9.6c.1 (2026-07-01) Its first sub-section

Body.

#### 9.6c.2 (2026-07-01) Its second sub-section

Body.

### 9.122 (2026-07-25) A uniquely numbered section

Body.
"""

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

# scan_orphan_modules. The BAD case is the class itself; the GOOD case is doing
# double duty as a regression guard for the two walk defects that made the same
# tool call LIVE code dead (2026-07-25) — the direction that HIDES bugs, since
# verify_match_consts suppresses its findings in files this walk calls orphaned.
_ORPHAN_BAD_LIB = """\
pub mod live;
"""
_ORPHAN_BAD_LIVE = """\
pub fn live_fn() -> u32 { 1 }
"""
_ORPHAN_BAD_ORPHAN = """\
// Reached by no `mod` chain and carrying no banner — cargo never opens this file,
// so `cargo check` is green whatever is in it (the classpnp class_gap3.rs trap).
pub fn stranded() -> u32 { 2 }
"""
_ORPHAN_GOOD_LIB = """\
// The freeldr shape: a `/*` inside a LINE comment. Stripping block comments
// before line comments opens a phantom block comment here that swallows every
// `mod` below it — 53 of freeldr's 87, i.e. 42,750 lines of live bootloader
// reported as dead. (called from arch/*/multiboot.S asm)
pub mod live;
// `async` is a keyword since edition 2018, so ws2_32's translation of async.c
// can only be spelled with a raw identifier. Not matching `r#` orphaned 1,941
// live lines.
pub mod r#async;

/* The phantom comment above only reaches this far if something CLOSES it, and
   that closer is what makes the bug bite: an unterminated `/*` is harmless
   because the regex finds no match at all. A fixture without a later `*/`
   therefore passes under BOTH the broken and the fixed stripper — verified,
   and the reason this block is here rather than a tidier one-liner. */
"""
_ORPHAN_GOOD_ASYNC = """\
pub fn async_fn() -> u32 { 3 }
"""
_ORPHAN_GOOD_ORPHAN = """\
//! ORPHAN-BY-DESIGN: a park with its reason stated where the reader will see it.
//! This is what satisfies the gate — not the absence of orphans.
pub fn parked() -> u32 { 4 }
"""
_ORPHAN_CARGO = """\
[package]
name = "orphan-fixture"
version = "0.0.0"
edition = "2021"
"""

# scan_extern_arity. BAD is the classpnp shape that wrote to address 0x2. GOOD carries the
# two parse traps that produced wrong answers before the scanner was trusted: a comment
# INSIDE a parameter list, and an `->` inside a generic (whose `>` drove the nesting depth
# negative and hid every following top-level comma — 3,790 phantom findings).
_ARITY_CARGO = """\
[package]
name = "arity-fixture"
version = "0.0.0"
edition = "2021"
"""
_ARITY_BAD_LIB = """\
pub mod shim;
pub mod caller;
"""
_ARITY_BAD_SHIM = """\
#[no_mangle] pub unsafe extern "C" fn ScsiGetNextSenseDescriptorByType(
    _buf: *mut u8, _len: u32, _ty: u8, found: *mut u8,
) -> *mut u8 {
    if !found.is_null() { *found = 0; }
    core::ptr::null_mut()
}
"""
_ARITY_BAD_CALLER = """\
extern "C" {
    fn ScsiGetNextSenseDescriptorByType(
        start: *mut u8, start_len: u8, types: *const u8, type_count: u32,
        out_type: *mut u8, out_buf: *mut *mut u8, out_len: *mut u8,
    ) -> u8;
}
"""
_ARITY_GOOD_LIB = """\
pub mod shim;
pub mod caller;
"""
_ARITY_GOOD_SHIM = """\
#[no_mangle] pub unsafe extern "C" fn Matching(
    _a: *mut u8,
    _b: u8, // a comment INSIDE the parameter list — masked before counting
    _cb: Option<unsafe extern "system" fn(u32) -> u32>,
) -> u8 {
    0
}
"""
_ARITY_GOOD_CALLER = """\
extern "C" {
    fn Matching(
        a: *mut u8,
        b: u8, /* the other comment flavour, with an -> arrow in it */
        cb: Option<unsafe extern "system" fn(u32) -> u32>,
    ) -> u8;
}
"""

# msvcrt's printf `...` entry points are hand-transcribed from wcs.c into their own TU
# (stable Rust cannot va_start; left in wcs.c clang re-inlines the C engine into them).
# Nothing can grade them by execution — msvcrt's printf plumbing is only coherent where a
# va_list IS a pointer, which no SysV host is — so this scanner IS the check, and a blind
# one would be silent about a real printf bug. The BAD fixture is the exact class:
# `_snprintf_s_l` with its `len` and `count` arguments swapped, which compiles, links,
# boots, and truncates only for small counts.
_SHIM_WCS = """\
int WINAPIV _snprintf(char *str, size_t len, const char *format, ...)
{
    int retval;
    va_list valist;
    va_start(valist, format);
    retval = vsnprintf(str, len, format, valist);
    va_end(valist);
    return retval;
}

int WINAPIV _snprintf_s_l(char *str, size_t len, size_t count,
        const char *format, _locale_t locale, ...)
{
    int retval;
    va_list valist;
    va_start(valist, locale);
    retval = _vsnprintf_s_l(str, len, count, format, locale, valist);
    va_end(valist);
    return retval;
}
"""

_SHIM_GOOD = """\
int WINAPIV _snprintf(char *str, size_t len, const char *format, ...)
{
    int retval;
    va_list valist;
    va_start(valist, format);
    retval = _vsnprintf(str, len, format, valist);
    va_end(valist);
    return retval;
}

int WINAPIV _snprintf_s_l(char *str, size_t len, size_t count,
        const char *format, _locale_t locale, ...)
{
    int retval;
    va_list valist;
    va_start(valist, locale);
    retval = _vsnprintf_s_l(str, len, count, format, locale, valist);
    va_end(valist);
    return retval;
}
"""

_SHIM_BAD = _SHIM_GOOD.replace("_vsnprintf_s_l(str, len, count,", "_vsnprintf_s_l(str, count, len,")

# The macro the GOOD fixture relies on: wcs.c writes `vsnprintf`, the shim spells out
# `_vsnprintf`, and that is only an identity while stdio.h says so.
_SHIM_STDIO = "#define vsnprintf _vsnprintf\n"

CONTROLS: list[Control] = [
    Control(
        name="scan_varargs_shim_parity",
        script="tools/scan_varargs_shim_parity.py",
        argv=["--lint"],
        bad={"wcs.c": _SHIM_WCS, "rust_varargs_shim.c": _SHIM_BAD, "stdio.h": _SHIM_STDIO},
        good={"wcs.c": _SHIM_WCS, "rust_varargs_shim.c": _SHIM_GOOD, "stdio.h": _SHIM_STDIO},
    ),
    Control(
        name="scan_doc_sections",
        script="tools/scan_doc_sections.py",
        argv=["--lint"],
        bad={"bad.md": _DOC_SECTIONS_BAD},
        good={"good.md": _DOC_SECTIONS_GOOD},
        pass_files=True,
        file_suffix=".md",
    ),
    Control(
        name="scan_extern_arity",
        script="tools/scan_extern_arity.py",
        bad={
            "Cargo.toml": _ARITY_CARGO,
            "src/lib.rs": _ARITY_BAD_LIB,
            "src/shim.rs": _ARITY_BAD_SHIM,
            "src/caller.rs": _ARITY_BAD_CALLER,
        },
        good={
            "Cargo.toml": _ARITY_CARGO,
            "src/lib.rs": _ARITY_GOOD_LIB,
            "src/shim.rs": _ARITY_GOOD_SHIM,
            "src/caller.rs": _ARITY_GOOD_CALLER,
        },
    ),
    Control(
        name="scan_orphan_modules",
        script="tools/scan_orphan_modules.py",
        bad={
            "Cargo.toml": _ORPHAN_CARGO,
            "src/lib.rs": _ORPHAN_BAD_LIB,
            "src/live.rs": _ORPHAN_BAD_LIVE,
            "src/orphan.rs": _ORPHAN_BAD_ORPHAN,
        },
        good={
            "Cargo.toml": _ORPHAN_CARGO,
            "src/lib.rs": _ORPHAN_GOOD_LIB,
            "src/live.rs": _ORPHAN_BAD_LIVE,
            "src/async.rs": _ORPHAN_GOOD_ASYNC,
            "src/orphan.rs": _ORPHAN_GOOD_ORPHAN,
        },
    ),
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
    Control(
        name="scan_bugcheck_stubs",
        script="tools/scan_bugcheck_stubs.py",
        bad={
            "drivers/foo/rust_cohab_stub.c": _STUBS_BAD_C,
            "drivers/foo/foo.h": _STUBS_HEADER,
            "sdk/lib/rust/foo-rust/src/lib.rs": _STUBS_BAD_RS,
        },
        good={
            "drivers/foo/rust_cohab_stub.c": _STUBS_GOOD_C,
            "drivers/foo/foo.h": _STUBS_HEADER,
            "sdk/lib/rust/foo-rust/src/lib.rs": _STUBS_GOOD_RS,
        },
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
    "scan_cc_mismatch": (
        "main() scans a fixed workspace root, so the subprocess harness cannot drive it — "
        "but scan_crate()/fix_crate() take a crate dir, and are covered by direct import at "
        "the bottom of this file. That gap let a WRITE-CAPABLE --fix ship untested."
    ),
    "scan_duplicate_symbols": "operates on link artifacts, not source",
    "scan_const_contradictions": "needs the whole tree (finds one NAME with two VALUES)",
    "scan_inline_flag_literals": "needs the sdk/include header catalog to know the right value",
    "shell32_tail_inventory": (
        "not a pattern scanner: reconciles the four LIVE CMake source lists against the "
        "ledger in tools/rustconv/data/shell32_cpp_tail.json. Its input is repo state, not "
        "file content, so a fixture tree cannot exercise it — the real ratchet is that every "
        "shell32 TU removal must update the ledger in the same commit or the gate fails."
    ),
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


# ── scan_orphan_modules: the ROOTLESS class needs its own control ────────────
# The table's positive control only requires SOME output line naming the fixture, so
# folding a rootless file into the existing orphan fixture would have been satisfied by
# the already-passing orphan hit — the harness would have "covered" the new class while
# proving nothing about it. Hence a targeted pair.
_ROOTLESS_WORKSPACE_CARGO = """\
[workspace]
members = ["real-rust"]
"""
_ROOTLESS_REAL_CARGO = """\
[package]
name = "real-rust"
version = "0.0.0"
edition = "2021"
"""
_ROOTLESS_DUMP = """\
// mscms-rust's exact shape: a translation dumped under sdk/lib/rust/<name>/src with no
// Cargo.toml of its own. `crate_root_of` walks up to the WORKSPACE manifest, whose
// crate_live_files() is empty, and the "unparseable crate -> fail open" branch then
// swallowed the whole directory. 4,782 lines reported clean for as long as the gate existed.
pub fn never_compiled() -> u32 { 7 }
"""
_ROOTLESS_DUMP_BANNERED = "//! ORPHAN-BY-DESIGN: parked, unintegrated.\n" + _ROOTLESS_DUMP


def _orphan_out(tmp_path: Path, files: dict) -> tuple[Path, str]:
    root = _materialize(tmp_path, files)
    ctl = next(c for c in CONTROLS if c.name == "scan_orphan_modules")
    _rc, out = _run(ctl, root)
    return root, out


def test_rootless_crate_is_not_invisible(tmp_path: Path):
    """A directory of .rs with NO crate root above it must be reported, not failed open."""
    root, out = _orphan_out(
        tmp_path,
        {
            "Cargo.toml": _ROOTLESS_WORKSPACE_CARGO,
            "real-rust/Cargo.toml": _ROOTLESS_REAL_CARGO,
            "real-rust/src/lib.rs": "pub fn ok() -> u32 { 1 }\n",
            "dump-rust/src/dumped.rs": _ROOTLESS_DUMP,
        },
    )
    hits = [ln for ln in out.splitlines() if "[ROOTLESS]" in ln and "dumped.rs" in ln]
    assert hits, f"rootless dump was not reported.\n(root={root})\n--- output ---\n{out}"
    # ...and the real crate beside it must NOT be swept up by the same rule. Match on the
    # crate's PATH, not on "lib.rs" — the finding's own text says "no crate root
    # (lib.rs/main.rs) above this file", so a substring test hits the message, not a hit.
    assert not [ln for ln in out.splitlines() if "real-rust" in ln], out


def test_rootless_crate_accepts_the_same_banner_escape(tmp_path: Path):
    """Declaring it is the sanctioned move — the gate is 'no UNDECLARED orphans'."""
    _root, out = _orphan_out(
        tmp_path,
        {
            "Cargo.toml": _ROOTLESS_WORKSPACE_CARGO,
            "real-rust/Cargo.toml": _ROOTLESS_REAL_CARGO,
            "real-rust/src/lib.rs": "pub fn ok() -> u32 { 1 }\n",
            "dump-rust/src/dumped.rs": _ROOTLESS_DUMP_BANNERED,
        },
    )
    assert "[ROOTLESS]" not in out, out


# --------------------------------------------------------------------------------------
# scan_cc_mismatch — driven by direct import, because main() has no path argument.
#
# These exist because of a live near-miss. A mainline lane taught the scanner about
# #[link_name]/#[export_name] (the right call: the LINKER pairs on the symbol, not on the
# Rust identifier), which added a capture group to DEF_RE and DECL_FN_RE. Every *scanner*
# read was updated. The *fixer* read the same regexes positionally and was not:
#
#   * `cfg, cc, name = m.groups()` in crate_def_conventions() -> ValueError, --fix dead.
#   * `d.group(1)` in fix_file() became the #[link_name] group, which is None for an
#     ordinary decl -> every decl treated as unnamed -> --fix relocates NOTHING and exits
#     reporting success. A silent no-op is the failure mode this whole file exists to catch.
#
# So the load-bearing assertion below is not "the fixer is correct", it is "the fixer is
# ALIVE": it must actually move a decl it is supposed to move.
# --------------------------------------------------------------------------------------


def _cc_mod():
    sys.path.insert(0, str(REPO / "tools"))
    import scan_cc_mismatch

    return scan_cc_mismatch


_CC_MISMATCHED = """\
extern "C" {
    pub fn BT958HwStartIO(dev: *mut u8) -> u8;
}

#[no_mangle]
pub unsafe extern "stdcall" fn BT958HwStartIO(dev: *mut u8) -> u8 { 0 }
"""


def test_cc_mismatch_scanner_fires_on_a_stdcall_def_behind_a_c_decl(tmp_path: Path):
    """The buslogic shape: decl says cdecl, definition says stdcall. i386 UB-fold risk."""
    cc = _cc_mod()
    src = _materialize(tmp_path, {"src/hw.rs": _CC_MISMATCHED}) / "src"
    hits = cc.scan_crate(str(src))
    assert any("BT958HwStartIO" in h for h in hits), hits


def test_cc_mismatch_fixer_is_alive(tmp_path: Path):
    """--fix must MOVE the decl. `moved == []` is the silent-no-op regression, not a pass."""
    cc = _cc_mod()
    root = _materialize(tmp_path, {"src/hw.rs": _CC_MISMATCHED})
    src = root / "src"
    moved, refused = cc.fix_crate(str(src))
    assert moved, f"--fix relocated nothing; it is a no-op again. refused={refused}"
    text = (src / "hw.rs").read_text()
    assert 'extern "stdcall" {' in text, text
    # ...and the fix must actually settle the lint it was written to settle.
    assert not cc.scan_crate(str(src)), cc.scan_crate(str(src))


def test_cc_mismatch_pairs_on_the_symbol_not_the_rust_identifier(tmp_path: Path):
    """#[link_name]/#[export_name] retarget the symbol; scanner AND fixer must follow it.

    Both halves matter. `decl_a` must pair with the export it NAMES (a real mismatch that
    identifier-keying never compares), and must NOT pair with the same-named local
    definition beside it (a false positive that identifier-keying invents)."""
    cc = _cc_mod()
    files = {
        "src/hw.rs": """\
extern "C" {
    #[link_name = "RealTarget"]
    pub fn decl_a(x: u32) -> u32;
}

#[export_name = "RealTarget"]
pub unsafe extern "stdcall" fn some_other_name(x: u32) -> u32 { x }

#[no_mangle]
pub unsafe extern "C" fn decl_a(x: u32) -> u32 { x }
"""
    }
    src = _materialize(tmp_path, files) / "src"
    hits = cc.scan_crate(str(src))
    assert any("RealTarget" in h for h in hits), f"retargeted pair never compared: {hits}"
    assert not any("decl_a declared" in h for h in hits), f"paired on identifier: {hits}"


def test_cc_mismatch_fixer_refuses_to_edit_a_generated_island(tmp_path: Path):
    """An island decl is bindgen's transcript of the C prototype — EVIDENCE, not opinion.

    Rewriting it is how e33a810c9ad silenced 8 real callback-ABI bugs and read as progress
    (305 -> 16 findings). The DEFINITION is the suspect side, so --fix must refuse."""
    cc = _cc_mod()
    root = _materialize(tmp_path, {"src/types_i386.rs": _CC_MISMATCHED})
    src = root / "src"
    before = (src / "types_i386.rs").read_text()
    moved, refused = cc.fix_crate(str(src))
    assert not moved, f"--fix edited a generated island: {moved}"
    assert any("BT958HwStartIO" in r for r in refused), refused
    assert (src / "types_i386.rs").read_text() == before
