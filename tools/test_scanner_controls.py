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

import importlib.util
import subprocess
import sys
from contextlib import redirect_stderr, redirect_stdout
from dataclasses import dataclass, field
from io import StringIO
from pathlib import Path

import pytest
import vendored

REPO = Path(__file__).resolve().parent.parent


# Both historical spellings of the ks/pcix/ramdisk import bug. The first is what
# shipped for months (`ntoskrnl.dll`, a file that does not exist); the second is
# what the FIRST repair shipped, because rustc appends `.dll` even to a name that
# already has an extension (`ntoskrnl.exe.dll`).
# pcix PciBuildHackTable: ASSERT(!NT_SUCCESS(Status)) lowered with the condition as guard.
_ASSERT_POLARITY_BAD = """
unsafe fn f(Status: NTSTATUS) {
    if !NT_SUCCESS(Status) {
        RtlAssert(
            b"!NT_SUCCESS(Status)\\0".as_ptr() as PVOID,
            b"init.c\\0".as_ptr() as PVOID,
            670,
            null_mut(),
        );
    }
}
"""
_ASSERT_POLARITY_GOOD = _ASSERT_POLARITY_BAD.replace("if !NT_SUCCESS", "if NT_SUCCESS")

_RAWDYLIB_BAD = """\
#[link(name = "ntoskrnl", kind = "raw-dylib")]
extern "C" {
    fn IoAllocateIrp(n: i8, charge: i32) -> *mut u8;
}

#[link(name = "ntoskrnl.exe", kind = "raw-dylib")]
extern "C" {
    fn IoFreeIrp(irp: *mut u8);
}
"""

# The corrected form, plus the two bare names that are CORRECT and must not be
# flagged: ntdll really is ntdll.dll and hal really is hal.dll. A scanner that
# just demanded +verbatim everywhere would flag these and teach the next agent to
# write `ntdll.dll.dll`.
_RAWDYLIB_GOOD = """\
#[link(name = "ntoskrnl.exe", kind = "raw-dylib", modifiers = "+verbatim")]
extern "C" {
    fn IoAllocateIrp(n: i8, charge: i32) -> *mut u8;
}

#[link(name = "ntdll", kind = "raw-dylib")]
extern "C" {
    fn RtlInitUnicodeString(s: *mut u8, p: *const u16);
}

#[link(name = "hal", kind = "raw-dylib")]
extern "C" {
    fn KeStallExecutionProcessor(us: u32);
}

#[link(name = "ntoskrnl", kind = "static")]
extern "C" {
    fn NotRawDylibSoNotOurProblem();
}
"""


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
    # Fixtures with a complete, self-contained call graph can inject that
    # workspace through the scanner's API, without changing CLI/gate defaults.
    summary_workspace: bool = False
    # Local optional filtering only; the gate runs all controls.
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

# scan_ptr_byte_diff. The version.dll bug verbatim: C's `lpNextSlash - lpSubBlock` on LPCWSTR
# counts WCHARs, the translation subtracted addresses. The oracle is the REAL
# dll/win32/version/version.c (the scanner pairs by function name against tracked C), so
# the fixture keeps the real function name. GOOD holds the fix and the byte-offset shape
# that must stay silent: both operands byte pointers, which is address arithmetic.
_PTR_BYTE_DIFF_BAD = """\
unsafe fn VersionInfo32_QueryValue(info: *const u8, mut lpSubBlock: LPCWSTR) -> i32 {
    let mut lpNextSlash: LPCWSTR = lpSubBlock;
    let len = (lpNextSlash as isize - lpSubBlock as isize) as UINT;
    len as i32
}
"""
_PTR_BYTE_DIFF_GOOD = """\
unsafe fn VersionInfo32_QueryValue(info: *const u8, mut lpSubBlock: LPCWSTR) -> i32 {
    let mut lpNextSlash: LPCWSTR = lpSubBlock;
    let len = lpNextSlash.offset_from(lpSubBlock) as UINT;
    let base: *const u8 = info;
    let end: *const u8 = info;
    let bytes = (end as usize - base as usize) as UINT;
    (len + bytes) as i32
}
"""

# scan_misaligned_cast. cryptlib A_SHAFinal before the fix: the length words stored through
# `Buffer + Pad` cast to *mut u32 over a [u8; 72]. GOOD is the fix, plus a cast that is only
# PASSED on (never dereferenced here), which must stay silent.
_MISALIGNED_CAST_BAD = """\
pub unsafe fn A_SHAFinal(Pad: u32, LengthHi: u32) {
    let mut Buffer: [u8; 72] = [0; 72];
    let Count: *mut u32;
    Count = Buffer.as_mut_ptr().add(Pad as usize) as *mut u32;
    *Count.offset(0) = LengthHi;
}
"""
_MISALIGNED_CAST_GOOD = """\
pub unsafe fn A_SHAFinal(Pad: u32, LengthHi: u32) {
    let mut Buffer: [u8; 72] = [0; 72];
    let Count: *mut u32;
    Count = Buffer.as_mut_ptr().add(Pad as usize) as *mut u32;
    Count.offset(0).write_unaligned(LengthHi);
    consume(Buffer.as_mut_ptr() as *mut u32);
}
"""

# scan_constant_placement. A real C twin (drivers/usb/usbuhci/usbuhci.c) whose two
# isochronous limits came back in each other's places: the TD count holds the transfer
# size and vice versa. Every constant is present, so the fact seal's SET check passes;
# GOOD puts them back.
_PLACEMENT_BAD = """\
pub unsafe extern "system" fn UhciQueryEndpointRequirements(
    uhciExtension: PVOID,
    EndpointProperties: PUSBPORT_ENDPOINT_PROPERTIES,
    EndpointRequirements: PUSBPORT_ENDPOINT_REQUIREMENTS,
) {
    let TdCount: ULONG = (2 * UHCI_MAX_ISO_TRANSFER_SIZE) as ULONG;
    (*EndpointRequirements).MaxTransferSize = UHCI_MAX_ISO_TD_COUNT as ULONG;
}
"""
_PLACEMENT_GOOD = (
    _PLACEMENT_BAD.replace("UHCI_MAX_ISO_TRANSFER_SIZE) as", "@@) as")
    .replace("= UHCI_MAX_ISO_TD_COUNT as", "= UHCI_MAX_ISO_TRANSFER_SIZE as")
    .replace("@@", "UHCI_MAX_ISO_TD_COUNT")
)

# scan_dropped_null_guard. compress before the fix: compress.c forwards final_size to
# lznt1_decompress, whose `if (final_size)` is the guard — so this also pins the one-level
# call-through. The fixture path shares `compress` with sdk/lib/rtl/compress.c, which is
# what makes that the twin; GOOD is the guarded store.
_NULL_GUARD_BAD = """\
pub unsafe extern "system" fn RtlDecompressFragment(
    format: u16, uncompressed: *mut u8, uncompressed_size: u32, compressed: *const u8,
    compressed_size: u32, offset: u32, final_size: *mut u32, workspace: *mut u8,
) -> i32 {
    *final_size = 0;
    0
}
"""
_NULL_GUARD_GOOD = """\
pub unsafe extern "system" fn RtlDecompressFragment(
    format: u16, uncompressed: *mut u8, uncompressed_size: u32, compressed: *const u8,
    compressed_size: u32, offset: u32, final_size: *mut u32, workspace: *mut u8,
) -> i32 {
    if !final_size.is_null() {
        *final_size = 0;
    }
    0
}
"""

# scan_store_on_failure. compress before the fix: lznt1_decompress writes *final_size only
# at its `out:` label, after every `return STATUS_BAD_COMPRESSION_BUFFER`; the Rust wrote it
# and then returned whatever status came back. The pairing needs a `rust/<crate>/` path.
_STORE_ON_FAILURE_BAD = """\
pub unsafe extern "system" fn RtlDecompressFragment(
    format: u16, uncompressed: *mut u8, uncompressed_size: u32, compressed: *const u8,
    compressed_size: u32, offset: u32, final_size: *mut u32, workspace: *mut u8,
) -> u32 {
    let result = decompress(uncompressed, compressed, offset);
    if !final_size.is_null() {
        *final_size = result.bytes_written as u32;
    }
    result.status
}
"""
# The GOOD fixture also carries the idioms that must stay quiet: a store dominated by a
# success test, and COM's `*pp = NULL` before a failure return.
_STORE_ON_FAILURE_GOOD = """\
pub unsafe extern "system" fn RtlDecompressFragment(
    format: u16, uncompressed: *mut u8, uncompressed_size: u32, compressed: *const u8,
    compressed_size: u32, offset: u32, final_size: *mut u32, workspace: *mut u8,
) -> u32 {
    let result = decompress(uncompressed, compressed, offset);
    if result.status == STATUS_SUCCESS && !final_size.is_null() {
        *final_size = result.bytes_written as u32;
    }
    result.status
}
pub unsafe extern "system" fn RtlCompressBuffer(
    format: u16, src: *const u8, src_size: u32, dst: *mut u8, dst_size: u32,
    chunk: u32, final_size: *mut u32, workspace: *mut u8,
) -> u32 {
    if format == 0 {
        return STATUS_INVALID_PARAMETER;
    }
    match compress(dst, src) {
        Ok(n) => {
            *final_size = n as u32;
            STATUS_SUCCESS
        }
        Err(st) => st,
    }
}
"""

# scan_bool_width, narrow-definition half. userenv before the fix: RefreshPolicy exported
# `-> u8` where gpolicy.c returns BOOL, so every C caller tested all of EAX against a
# callee that set AL. The DECL half: a Rust extern declaring the u8 definition `-> BOOL`.
_BOOL_WIDTH_BAD = {
    "rust/userenv-rust/src/gpolicy.rs": """\
#[no_mangle]
pub unsafe extern "system" fn RefreshPolicy(bMachine: BOOL) -> u8 {
    1u8
}
#[no_mangle]
pub unsafe extern "system" fn MyNarrowHelper() -> u8 {
    1u8
}
""",
    "rust/caller-rust/src/lib.rs": """\
extern "system" {
    fn MyNarrowHelper() -> i32;
}
""",
    # Per-arch cfg twins of one export, only one of them widened.
    "rust/win32k-rust/src/region.rs": """\
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn MyRegionTest(p: PREGION) -> BOOL { 1 }
#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn MyRegionTest(p: PREGION) -> u8 { 1 }
""",
}
_BOOL_WIDTH_GOOD = {
    "rust/userenv-rust/src/gpolicy.rs": """\
#[no_mangle]
pub unsafe extern "system" fn RefreshPolicy(bMachine: BOOL) -> BOOL {
    1
}
#[no_mangle]
pub unsafe extern "system" fn MyNarrowHelper() -> u8 {
    1u8
}
""",
    "rust/caller-rust/src/lib.rs": """\
extern "system" {
    fn MyNarrowHelper() -> u8;
}
""",
    "rust/win32k-rust/src/region.rs": """\
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn MyRegionTest(p: PREGION) -> BOOL { 1 }
#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn MyRegionTest(p: PREGION) -> BOOL { 1 }
""",
}

# scan_cfg_vs_c. user32 before the fix: class.c's `#ifdef _WIN64` index check was
# translated as amd64-only, so arm64 returned a truncated pointer for GCLP_* indices.
_WIN64_CFG_BAD = """\
pub unsafe extern "system" fn GetClassLongW(hWnd: HWND, nIndex: i32) -> u32 {
    let mut Ret: usize = 0;
    #[cfg(target_arch = "x86_64")]
    {
        Ret = 1;
    }
    #[cfg(not(target_arch = "x86_64"))]
    {
        Ret = 2;
    }
    Ret as u32
}
"""
_WIN64_CFG_GOOD = _WIN64_CFG_BAD.replace('target_arch = "x86_64"', 'target_pointer_width = "64"')

# Its UP/SMP axis. bug.c freezes the other CPUs under `#ifdef CONFIG_SMP`; the Rust gated
# that on an ARCH, so it froze in UP builds and never on arm64 SMP. `feature = "smp"` is
# how Rust spells CONFIG_SMP (ntkrnlmp).
_SMP_CFG_BAD = """\
pub unsafe extern "system" fn KeBugCheckWithTf(BugCheckCode: u32, TrapFrame: PVOID) {
    #[cfg(target_arch = "x86")]
    {
        KxFreezeExecution();
    }
    KiDisplayBlueScreen(BugCheckCode);
}
"""
_SMP_CFG_GOOD = _SMP_CFG_BAD.replace('target_arch = "x86"', 'feature = "smp"')

# The same scanner's no-C half: an orphaned cfg (its fastcall twin deleted) stacked onto
# the next item ANDs to "never", compiling RtlPrefetchMemoryNonTemporal out on every arch.
_DEAD_CFG_BAD = """\
#[cfg(not(target_arch = "x86"))]
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn RtlPrefetchMemoryNonTemporal(_Source: PVOID, _Length: SIZE_T) {}
"""
_DEAD_CFG_GOOD = """\
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn RtlPrefetchMemoryNonTemporal(_Source: PVOID, _Length: SIZE_T) {}
#[cfg(all(target_arch = "x86_64", target_os = "windows"))]
pub fn allowed() {}
"""

_BITWISE_NOT_BAD = """\
pub unsafe fn adjust_fua(flags: u32, special: u32) -> bool {
    // C: DEV_WRITE_CACHE && !DEV_POWER_PROTECTED && !CLASS_SPECIAL_FUA_NOT_SUPPORTED.
    // Both negated guards are BITWISE not -> constant true -> both guards dead.
    (flags & 0x1) != 0 && !(flags & 0x10) != 0 && !(special & 0x80) != 0
}
pub unsafe fn prepare_copy(Success: u8) -> u32 {
    // C: if (!Success) -- usetup quit every install. The `if` before the `!` once read as
    // a macro name, so the bare `!x != 0` form was invisible.
    if !Success != 0 {
        return 29;
    }
    24
}
"""

# A KEYWORD before the `!` once read as a macro bang (`if!`), hiding every `if !x != 0`:
# usetup's PrepareCopyPage always quit and powercfg's Pos_InitData always failed.
_BITWISE_NOT_KEYWORD_BAD = """\
pub unsafe fn prepare(success: u8) -> u32 {
    if !success != 0 {
        return 1;
    }
    0
}
"""

# The GOOD fixture carries the fix AND both idioms the scanner must never flag. Without
# them a scanner that simply matched every `!` would still pass its negative control.
_BITWISE_NOT_GOOD = """\
pub unsafe fn adjust_fua(flags: u32, special: u32) -> bool {
    (flags & 0x1) != 0 && (flags & 0x10) == 0 && (special & 0x80) == 0
}
pub fn unknown_flag_test(opts: u32) -> bool {
    // `x & !MASK` — genuinely bitwise, genuinely wanted: "any bit outside MASK?"
    (opts & !(0x1u32 | 0x2u32)) != 0
}
pub fn layout_check() {
    // A MACRO bang, not the not-operator.
    assert!(core::mem::offset_of!(SomeStruct, first_field) == 0);
}
pub fn deliberate(x: u32) -> bool {
    !x != 0 // nolint: bitwise-not-bool (complement really is wanted)
}
"""

_ASSERT_UB_BAD = """\
pub unsafe fn paged_thing() {
    // PAGED_CODE() — a DBG-only diagnostic, lowered to UB (§9.120).
    if KeGetCurrentIrql() > APC_LEVEL {
        core::hint::unreachable_unchecked();
    }
}
pub unsafe fn asserted_thing(p: *const u8) {
    // The second lowering of the same class: a panic!, which in kernel mode is a
    // SILENT HANG (the panic handler loops forever). Worse than the bugcheck variant.
    if p.is_null() {
        panic!("NT_ASSERT failed");
    }
}
"""
_ASSERT_UB_GOOD = """\
pub unsafe fn paged_thing() {
    if KeGetCurrentIrql() > APC_LEVEL {
        DbgPrint(b"NTDDK: Pageable code called at IRQL > APC_LEVEL\\n\\0".as_ptr() as *const i8);
    }
}
pub unsafe fn asserted_thing(p: *const u8) {
    // The faithful lowering: what C's ASSERT calls under DBG. Prints and RETURNS.
    if p.is_null() {
        RtlAssert(
            b"p != NULL\\0".as_ptr() as *mut core::ffi::c_void,
            b"good.rs\\0".as_ptr() as *mut core::ffi::c_void,
            line!(),
            core::ptr::null_mut::<i8>(),
        );
    }
}
pub unsafe fn reports_unimplemented() {
    // ReactOS's UNIMPLEMENTED macro: the macro name is TEXT inside a format string.
    // This neither panics nor hangs, and flagging it is the false positive that made
    // the scanner report pcix/fdo.rs twice. Literals must be blanked before matching.
    DbgPrint(b"WARNING:  %s at %s:%d is unimplemented!()\\n\\0".as_ptr() as *const i8);
}
#[cfg(target_arch = "aarch64")]
pub unsafe fn port_io() -> u8 {
    // A hang macro with no assert origin is legitimate — this branch genuinely cannot
    // run. Only an assert-DERIVED one is a defect, so the origin test is what gates.
    unreachable!("x86 port I/O is not used on the arm64 UEFI loader")
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

# scan_opaque_ptr_arith. Seeded from consrv's handle table (2026-09-23): the
# field is typed by bindgen as a pointer to an opaque 1-byte stub, so .add(i)
# strides one byte. The fixture ships its own bindgen stand-in, because the
# scanner reads field types from *-sys bindgen files.
_OPAQUE_BINDGEN = """\
pub struct _IO_HANDLE {
    pub _address: u8,
}
pub struct _PROCESS_DATA {
    pub HandleTable: *mut _IO_HANDLE,
}
"""
_OPAQUE_BAD = """\
pub unsafe fn get(pd: *mut _PROCESS_DATA, i: usize) -> *mut RealHandle {
    (*pd).HandleTable.add(i) as *mut RealHandle
}
"""
_OPAQUE_GOOD = """\
pub unsafe fn get(pd: *mut _PROCESS_DATA, i: usize) -> *mut RealHandle {
    ((*pd).HandleTable as *mut RealHandle).add(i)
}
"""

# scan_cdecl_alias. Seeded from fastfat's rust_cohab_forceinline.c (2026-09-26): the
# bare cdecl name aliased onto the stdcall body double-pops the arguments. The GOOD side
# keeps an @0 alias (sound: both conventions are the same code) and a real thunk.
_CDECL_ALIAS_BAD = r"""
__asm__(".globl _KeInitializeSpinLock\n"
        ".set _KeInitializeSpinLock, _KeInitializeSpinLock@4\n");
"""
_CDECL_ALIAS_GOOD = r"""
__asm__(".globl _KeGetCurrentProcessorNumber\n"
        ".set _KeGetCurrentProcessorNumber, _KeGetCurrentProcessorNumber@0");
__asm__(".globl _KeInitializeSpinLock\n_KeInitializeSpinLock:\n"
        " pushl 4(%esp)\n call _KeInitializeSpinLock@4\n ret\n");
"""


# scan_fnptr_abi_transmute. Seeded from consrv's GuiVtbl (2026-09-23): a plain
# Rust fn put in an NTAPI slot through `as *const ()`, which is fine on 64-bit
# and a csrss crash on i386. GOOD is the fixed shape: extern "system", passed
# directly, plus a foreign decl (body-less, inside an extern block) that must
# not be mistaken for a Rust-ABI fn.
_FNABI_BAD = """\
pub unsafe fn InitFrontEnd(this: *mut u8) -> i32 { 0 }
pub static SLOT: Option<unsafe extern "system" fn(*mut u8) -> i32> =
    unsafe { Some(core::mem::transmute(InitFrontEnd as *const ())) };
"""
# Second shape (classpnp 2026-09-26): the source type SAYS extern "C", so rustc
# agrees with the fn, and the inferred target re-types it for an NTAPI slot.
_FNABI_BAD_TYPED = """\
pub unsafe extern "C" fn Done(d: *mut u8, i: *mut u8, c: *mut u8) -> i32 { 0 }
pub unsafe fn f(irp: *mut u8) {
    IoSetCompletionRoutine(
        irp,
        Some(core::mem::transmute::<unsafe extern "C" fn(*mut u8, *mut u8, P) -> i32, _>(Done)),
        core::ptr::null_mut(), 1, 1, 1,
    );
}
"""
_FNABI_GOOD = """\
pub unsafe extern "system" fn InitFrontEnd(this: *mut u8) -> i32 { 0 }
pub static SLOT: Option<unsafe extern "system" fn(*mut u8) -> i32> = Some(InitFrontEnd);
extern "system" {
    fn Foreign(x: *mut u8) -> i32;
}
pub static SLOT2: Option<unsafe extern "system" fn(*mut u8) -> i32> =
    unsafe { Some(core::mem::transmute(Foreign as *const ())) };
pub unsafe extern "C" fn cexit(code: i32) -> ! { loop {} }
pub static EXIT: unsafe extern "C" fn(i32) =
    unsafe { core::mem::transmute::<unsafe extern "C" fn(i32) -> !, _>(cexit) };
"""

# scan_unaligned_wide. Seeded from ws2_32's L"LibraryPath" (2026-10-04): a UTF-16 literal
# spelled as a byte string and cast to a wide pointer, alignment 1, which landed on an odd
# address and failed the kernel's ProbeForRead. GOOD has the aligned macro form and a narrow
# C string that must not be mistaken for a wide one.
_UNALIGNED_WIDE_BAD = """\
pub unsafe fn f() {
    RegQueryValueExW(key, b"L\\0i\\0b\\0\\0\\0".as_ptr() as LPCWSTR, 0 as _);
}
"""
_UNALIGNED_WIDE_GOOD = """\
pub unsafe fn f() {
    RegQueryValueExW(key, win32macros_rust::wide_bytes!(b"L\\0i\\0b\\0\\0\\0") as LPCWSTR, 0 as _);
    OutputDebugStringA(b"text\\0".as_ptr() as PCSTR);
}
"""

# scan_wide_byte_copy. Seeded from fastfat's FatQueryDirectory (2026-09-23): the
# C's RtlCopyMemory byte count kept as the element count of a u16 copy. GOOD has
# the fixed byte copy, a count already in WCHARs (/ size_of), and an ANSI copy
# whose uncast .Buffer must not be mistaken for a wide one.
_WIDEBYTE_BAD = """\
pub unsafe fn f(name: *const UNICODE_STRING, dst: *mut u8, BytesConverted: u32) {
    core::ptr::copy_nonoverlapping(
        (*name).Buffer,
        dst as *mut u16,
        BytesConverted as usize,
    );
}
"""
_WIDEBYTE_GOOD = """\
pub unsafe fn f(name: *const UNICODE_STRING, dst: *mut u8, BytesConverted: u32) {
    core::ptr::copy_nonoverlapping((*name).Buffer as *const u8, dst, BytesConverted as usize);
    core::ptr::copy_nonoverlapping(
        (*name).Buffer,
        dst as *mut u16,
        (*name).Length as usize / core::mem::size_of::<u16>(),
    );
}
pub unsafe fn g(a: *const ANSI_STRING, out: *mut i8) {
    core::ptr::copy_nonoverlapping((*a).Buffer, out, (*a).Length as usize);
}
"""

# scan_typed_byte_copy. Seeded from partmgr's FdoIoctlDiskGetDriveLayoutEx (2026-10-07):
# `as *const _` inferred the struct from the source, so the byte count became a struct
# count. GOOD: the fixed byte copy, a double cast that ends in u8, an element count, and an
# uncast struct copy whose `...Size` names an entry count.
_TYPEDBYTE_BAD = """\
pub unsafe fn f(Irp: PIRP, size: usize) {
    let mut layoutEx: *mut _DRIVE_LAYOUT_INFORMATION_EX = core::ptr::null_mut();
    core::ptr::copy_nonoverlapping(
        layoutEx as *const _,
        (*Irp).AssociatedIrp.SystemBuffer as *mut _,
        size,
    );
}
pub unsafe fn g(a: *const ENTRY, b: *mut ENTRY, n: usize) {
    core::ptr::copy_nonoverlapping(a, b, n * core::mem::size_of::<ENTRY>());
}
"""
_TYPEDBYTE_GOOD = """\
pub unsafe fn f(Irp: PIRP, size: usize, h: *const HDR, out: *mut u8) {
    let mut layoutEx: *mut _DRIVE_LAYOUT_INFORMATION_EX = core::ptr::null_mut();
    core::ptr::copy_nonoverlapping(
        layoutEx as *const u8,
        (*Irp).AssociatedIrp.SystemBuffer as *mut u8,
        size,
    );
    core::ptr::copy_nonoverlapping(h as *const HDR as *const u8, out, (*h).HeaderSize as usize);
}
pub unsafe fn g(a: *const ENTRY, b: *mut ENTRY, OldListSize: u32) {
    core::ptr::copy_nonoverlapping(a, b, OldListSize as usize);
}
"""

# scan_noop_goto. Seeded from setuplib's ParseArcName (2026-10-07): `goto Quit;` became a
# labeled block holding only its own break. GOOD: the label spans the skipped code.
_NOOPGOTO_BAD = """\
pub unsafe fn f(ramdisk: bool) -> i32 {
    if ramdisk {
        'quit: {
            break 'quit;
        }
    }
    parse_controller()
}
"""
_NOOPGOTO_GOOD = """\
// The fix replaced `'quit: { break 'quit; }` with a block that spans the parse.
pub unsafe fn f(ramdisk: bool) -> i32 {
    'quit: {
        if ramdisk {
            break 'quit;
        }
        parse_controller();
    }
    0
}
"""

# The inline comments are load-bearing. tree-sitter makes a comment a CHILD of
# the argument list, so counting it as an argument shifts every positional index
# — the level check reads the comment (not DISPATCH_LEVEL) and the last-arg
# OldIrql lookup lands one slot early. Here that makes the scanner BLIND, so the
# positive control is the only thing that can catch it (an FP fixture cannot).
# Sibling incident, same root, opposite direction: scan_ob_refcount's fastfat FP.
_IRQL_BAD = """\
pub unsafe fn leaky(cond: bool) -> u32 {
    let mut old: u8 = 0;
    KeRaiseIrql(/* NewIrql */ DISPATCH_LEVEL, &mut old /* OldIrql */);
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

# scan_extern_decl_drift. BAD carries both halves of what the tool compares:
# DEPTH (a `*` the def does not have) and WIDTH (the same indirection depth, a
# wider pointee). The width case IS fastfat's FatGetNextMcbEntry — an 8-byte
# store through a 4-byte out-param, which the depth-only tool could not see.
#
# The GOOD tree pins every shape that made this tool cry wolf. The first four
# took it from 2,682 findings to 10; adding width resolution added four more,
# and each of those is a rule whose removal would bury the real findings again:
# an alias spelled differently per island, a path-qualified alias, a use-rename,
# a #[link_name] decl naming a different symbol, `Option<Callback>` against the
# callback, a pointer passed as a pointer-width integer, a by-value parameter
# the callee reads NARROWER than it is passed, and a `type X = path::X`
# re-export — which normalises to `X -> X`, a cycle, and silently made
# `*mut c_void` and therefore PVOID undecidable everywhere.
_DECL_DRIFT_BAD_A = """\
pub type PIRP_CONTEXT = *mut IRP_CONTEXT;
pub type VBO = u32;
extern "system" {
    // One `*` too many: volinfo declared cachesup's FatUnpinRepinnedBcbs this way,
    // so the callee read a pointer-to-pointer as the struct itself.
    fn FatUnpinRepinnedBcbs(IrpContext: *mut PIRP_CONTEXT);
    // Same depth, WIDER pointee: the callee stores 8 bytes into the caller's
    // 4-byte VBO. Nothing about either signature looks wrong.
    fn FatGetNextMcbEntry(Vcb: *mut VCB, Vbo: *mut u64) -> u8;
    // By value, the callee reads 64 bits where 32 were passed — the top half is
    // whatever was in the register.
    fn MmAllocateMemoryWithType(Size: u32) -> *mut u8;
}
pub unsafe fn caller(mut ctx: PIRP_CONTEXT, v: *mut VCB) {
    let mut vbo: u64 = 0;
    FatUnpinRepinnedBcbs(&mut ctx);
    FatGetNextMcbEntry(v, &mut vbo);
    MmAllocateMemoryWithType(4);
}
"""
_DECL_DRIFT_BAD_B = """\
pub type VBO = u32;
#[no_mangle]
pub unsafe extern "system" fn FatUnpinRepinnedBcbs(_IrpContext: *mut IRP_CONTEXT) {}
#[no_mangle]
pub unsafe extern "system" fn FatGetNextMcbEntry(_Vcb: *mut VCB, Vbo: *mut VBO) -> u8 {
    *Vbo = 0;
    1
}
#[no_mangle]
pub unsafe extern "system" fn MmAllocateMemoryWithType(_Size: usize) -> *mut u8 {
    core::ptr::null_mut()
}
"""
_DECL_DRIFT_GOOD_A = """\
pub type VBO = u32;
pub type PKSPIN_LOCK = *mut u64;
pub type c_void = core::ffi::c_void;
pub type PVOID = *mut c_void;
pub type HWND = PVOID;
pub type DLGPROC = Option<unsafe extern "system" fn(a: u32) -> u32>;
pub type DRAWSTATEPROC = Option<unsafe extern "system" fn(b: u32) -> u32>;
use ntkrnl_sys::ddk::PKSPIN_LOCK as DdkPKSPIN_LOCK;
extern "system" {
    // Agrees with the def: same depth, spelled through an alias.
    fn FatGetNextMcbEntry(Vcb: *mut VCB, Vbo: *mut VBO) -> u8;
    // A use-rename is an alias too.
    fn ExfInsert(Lock: DdkPKSPIN_LOCK) -> u8;
    // A path-qualified alias is the same alias.
    fn MmDeleteVirtualMapping(Process: ntkrnl_sys::ddk::PEPROCESS) -> u8;
    // #[link_name] declares SOMETHING ELSE — never pair it with this def by ident.
    #[link_name = "Shell32RustCoCreateInitSF"]
    fn SHELL32_CoCreateInitSF(a: u32, b: u32) -> u32;
    // `Option<DLGPROC>` and `DLGPROC` are one pointer either way.
    fn DialogBoxIndirectParamA(Proc: Option<DLGPROC>) -> isize;
    // A handle passed as a pointer-width integer: same register, same bits.
    fn co_UserSetCapture(Wnd: isize) -> u8;
    // A callback parameter declared as a bare pointer: the typedef IS a pointer.
    fn DrawStateW(Cb: *mut c_void) -> u8;
    // The callee reads a subset of the bits the caller definitely set.
    fn FatDeleteDirent(Flag: BOOL) -> u8;
    // `*mut c_void` against PVOID — only resolvable if the c_void re-export
    // above is NOT recorded as an alias of itself.
    fn ExFreePool(P: *mut c_void);
}
pub unsafe fn caller(v: *mut VCB, l: DdkPKSPIN_LOCK, p: ntkrnl_sys::ddk::PEPROCESS) {
    let mut vbo: VBO = 0;
    FatGetNextMcbEntry(v, &mut vbo);
    ExfInsert(l);
    MmDeleteVirtualMapping(p);
    SHELL32_CoCreateInitSF(1, 2);
    DialogBoxIndirectParamA(None);
    co_UserSetCapture(0);
    DrawStateW(core::ptr::null_mut());
    FatDeleteDirent(0);
    ExFreePool(core::ptr::null_mut());
}
"""
_DECL_DRIFT_GOOD_B = """\
pub type PVOID = *mut core::ffi::c_void;
pub type HWND = PVOID;
pub type DLGPROC = Option<unsafe extern "system" fn(a: u32) -> u32>;
pub type DRAWSTATEPROC = Option<unsafe extern "system" fn(b: u32) -> u32>;
#[no_mangle]
pub unsafe extern "system" fn FatGetNextMcbEntry(_Vcb: *mut VCB, Vbo: *mut VBO) -> u8 {
    *Vbo = 0;
    1
}
#[no_mangle]
pub unsafe extern "system" fn ExfInsert(_Lock: PKSPIN_LOCK) -> u8 {
    0
}
#[no_mangle]
pub unsafe extern "system" fn MmDeleteVirtualMapping(_Process: PEPROCESS) -> u8 {
    0
}
#[no_mangle]
pub unsafe extern "system" fn SHELL32_CoCreateInitSF(_a: u32, _b: u32, _c: u32) -> u32 {
    0
}
#[no_mangle]
pub unsafe extern "system" fn DialogBoxIndirectParamA(_Proc: DLGPROC) -> isize {
    0
}
#[no_mangle]
pub unsafe extern "system" fn co_UserSetCapture(_Wnd: HWND) -> u8 {
    0
}
#[no_mangle]
pub unsafe extern "system" fn DrawStateW(_Cb: DRAWSTATEPROC) -> u8 {
    0
}
#[no_mangle]
pub unsafe extern "system" fn FatDeleteDirent(_Flag: BOOLEAN) -> u8 {
    0
}
#[no_mangle]
pub unsafe extern "system" fn ExFreePool(_P: PVOID) {}
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

// fastfat's FatMarkHandle, 2026-07-29. The out-param of ObReferenceObjectByHandle
// is at a FIXED index, and tree-sitter puts an inline comment in the argument
// list — so `/* UserMode */` shifted the index by one and the scanner tracked
// the comment's own text, reporting a leak of a variable named `UserMode` on
// two paths that both dereference correctly.
pub unsafe fn balanced_with_inline_comment(h: HANDLE, cond: bool) -> u32 {
    let mut fo: *mut FILE_OBJECT = null_mut();
    let status = ObReferenceObjectByHandle(
        h,
        0,
        IoFileObjectType as *mut _,
        1, /* UserMode */
        &mut fo as *mut *mut FILE_OBJECT as *mut PVOID,
        null_mut(),
    );
    if !NT_SUCCESS(status) {
        return 1;
    }
    if cond {
        ObDereferenceObject(fo as *mut _);
        return 2;
    }
    ObDereferenceObject(fo as *mut _);
    0
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

# scan_type_alias_contradictions. Modelled on fastfat's VBO (task #46): the
# generated islands said ULONG32 (4) while dirsup.rs said LARGE_INTEGER (8), so an
# 8-byte store went through a 4-byte out-param on every file open. Both fixtures
# also carry the two FALSE-POSITIVE shapes, because those are what decide whether
# anyone trusts the lint: a `#[cfg]`-gated pointer-width pair (only one is ever
# compiled) and an associated type in two impls (differing per impl is the POINT).
# Both must stay silent in the BAD fixture too — a scanner that flags them would
# bury its one real finding in noise.
_ALIAS_FP = """\
#[cfg(target_pointer_width = "64")]
pub type UINTN = u64;
#[cfg(target_pointer_width = "32")]
pub type UINTN = u32;

trait Width { type Ch; }
impl Width for Narrow { type Ch = i8; }
impl Width for Wide { type Ch = u16; }
"""

_ALIAS_BAD_A = "pub type VBO = ULONG32;\n" + _ALIAS_FP
_ALIAS_BAD_B = "type VBO = LARGE_INTEGER;\n"
_ALIAS_GOOD_A = "pub type VBO = ULONG32;\n" + _ALIAS_FP
_ALIAS_GOOD_B = "use crate::types::VBO;\n"

# scan_mask_drift. Seeded from E21 (2026-08-01): NtOpenKey masked a bare 0x0100
# where the C masks ~KEY_WOW64_RES (0x0300), so KEY_WOW64_32KEY reached the object
# access check and every RegOpenKeyEx asking for it got ACCESS_DENIED — while the
# identical RegCreateKeyEx, which masks correctly, succeeded. The BAD fixture also
# carries the second shape from the same hunt: i386 KeFlushCurrentTb dropped its
# whole CR4.PGE branch, so there is no live mask of that target at all.
#
# The GOOD case pins the false positives an earlier version of this scanner
# reported, every one of which cost a hand-audit:
#   * the live spelling is usually `&= !(NAME as ULONG)`, not a bare `!NAME`;
#   * translation re-cases locals, so the C's `Cr4` is the Rust's `cr4`;
#   * a literal that MATCHES its named constant is correct, merely unnamed;
#   * `~` over a bare literal names no constant to check.
# It also pins the trap that made counting occurrences unusable: a fix's own
# explanatory comment QUOTES the C, so the C statement appears twice while the
# live code masks once. That must stay silent.
# scan_macro_fn_shadow. Seeded from shell32-rust 2026-08-08: the crate had the correct
# `#[macro_export] macro_rules! UNREFERENCED_PARAMETER` in win32.rs — its own comment citing
# the ud2/BUGCHECK class — and two modules carried an empty `fn` shadow anyway, because both
# are declared BEFORE win32 in lib.rs and macro_rules! textual scoping only runs forward.
# The GOOD case pins the two ways this scanner must NOT fire, because a false positive here
# is a lint you cannot satisfy by doing the right thing:
#   * a same-named `fn` with a REAL body is an ordinary inline helper, not a shadow
#     (shell32-rust's `pub fn LOWORD(l: DWORD) -> WORD { (l & 0xFFFF) as WORD }`);
#   * importing the macro and calling it with `!` is the FIX, and must come back clean.
_MACROSHADOW_BAD = """\
#[macro_export]
macro_rules! UNREFERENCED_PARAMETER {
    ($($arg:tt)*) => {{}};
}

#[inline] fn UNREFERENCED_PARAMETER<T>(_: T) {}

pub unsafe fn CRecycleBin_Parse(pbc: *mut u8) {
    UNREFERENCED_PARAMETER(pbc);
}
"""

_MACROSHADOW_GOOD = """\
#[macro_export]
macro_rules! UNREFERENCED_PARAMETER {
    ($($arg:tt)*) => {{}};
}

// A real body is a helper, not a barrier — this must stay silent.
#[inline]
pub fn LOWORD(l: u32) -> u16 {
    (l & 0xFFFF) as u16
}

pub unsafe fn CRecycleBin_Parse(pbc: *mut u8) {
    UNREFERENCED_PARAMETER!(pbc);
}
"""

# The DDK case: UNREFERENCED_PARAMETER is a macro in every DDK header, so an extern
# declaration of it is a finding with no in-crate macro_rules! (cdfs, fastfat and cdrom
# declared it in 30 files, resolved by whichever crate exported a no-op, 2026-10-09).
_MACROSHADOW_DDK_BAD = """\
extern "C" {
    fn UNREFERENCED_PARAMETER(param: *mut core::ffi::c_void);
}

pub unsafe fn CdCommonClose(IrpContext: *mut u8) {
    UNREFERENCED_PARAMETER(IrpContext as *mut core::ffi::c_void);
}
"""

_MACROSHADOW_DDK_GOOD = """\
macro_rules! UNREFERENCED_PARAMETER {
    ($($p:expr),* $(,)?) => {{ $( let _ = $p; )* }};
}

pub unsafe fn CdCommonClose(IrpContext: *mut u8) {
    UNREFERENCED_PARAMETER!(IrpContext);
}
"""

_MASKDRIFT_BAD = """\
const KEY_WOW64_RES: ACCESS_MASK = 0x00000300;
const CR4_PGE: ULONG = 0x00000080;

pub unsafe fn NtOpenKey(DesiredAccess: u32) -> u32 {
    let mut DesiredAccess = DesiredAccess & !0x0100u32;
    DesiredAccess
}

pub unsafe fn KeFlushCurrentTb() {
    let cr3 = readcr3();
    writecr3(cr3);
}

//     DesiredAccess &= ~KEY_WOW64_RES;
//     Cr4 = __readcr4();
//     __writecr4(Cr4 & ~CR4_PGE);
"""
_MASKDRIFT_GOOD = """\
const KEY_WOW64_RES: ACCESS_MASK = 0x00000300;
const CR4_PGE: ULONG = 0x00000080;
const MEM_EXECUTE_OPTION_VALID_FLAGS: u32 = 0x3F;

pub unsafe fn NtOpenKey(DesiredAccess: u32) -> u32 {
    // C: `DesiredAccess &= ~KEY_WOW64_RES;` — quoted here on purpose. An
    // occurrence-counting rule reads this as a second C mask and reports a drop.
    let mut DesiredAccess = DesiredAccess & !KEY_WOW64_RES;
    DesiredAccess
}

pub unsafe fn KeFlushCurrentTb() {
    let cr4: ULONG = readcr4();
    writecr4(cr4 & !CR4_PGE);
}

pub unsafe fn DC_vPrepareDCsForBlit(dc: &mut DC) {
    dc.flags &= !(DC_DIRTY_RAO as ULONG);
}

pub unsafe fn MiValidateExecuteOptions(ExecuteOptions: u32) -> bool {
    (ExecuteOptions & !MEM_EXECUTE_OPTION_VALID_FLAGS) != 0
}

pub unsafe fn MiAlignDown(Address: usize) -> usize {
    Address & !0xFFF
}

//     DesiredAccess &= ~KEY_WOW64_RES;
//     Cr4 = __readcr4();
//     __writecr4(Cr4 & ~CR4_PGE);
//     pdc->flags &= ~DC_DIRTY_RAO;
//     if (ExecuteOptions & ~MEM_EXECUTE_OPTION_VALID_FLAGS)
//     Address &= ~0xFFF;
"""

# scan_dropped_prints. Seeded from CmpLinkKeyToHive (task #76, 2026-08-02): three
# failure returns, upstream prints on each, all three dropped — so when
# \\REGISTRY\\USER\\S-1-5-18 did not appear there was no signal anywhere, and the
# #73 hunt spent two build+boot cycles establishing what one grep would have
# given free. The BAD fixture also carries the NOOP shape from the same sweep
# (cmlib's `unsafe fn DPRINT1(_fmt) {}`), which is worse than a missing print
# because it reads as covered.
#
# The GOOD case pins every false positive that cost this scanner precision, each
# found by shipping it wrong first:
#   * the wrapper/_impl split — 1490 functions tree-wide keep the prints in the
#     body, so exact-name pairing calls every wrapper silent (143 false hits);
#   * `rtl_string_cb_printf_w!` formats into a BUFFER and matched a "contains
#     print" heuristic, hiding three real drops in CmpCreateControlSet;
#   * DPRINT/TRACE/WARN are compiled out or off by default upstream, so dropping
#     them loses nothing and flagging them would triple the harvest with nothing
#     behind it;
#   * a bare `FIXME(...)`/`ERR(...)` call IS the wine debug channel, not a stub;
#   * a C function with no live Rust counterpart is untranslated, not silent.
_DROPPEDPRINT_BAD = """\
unsafe fn DPRINT1(_format: *const i8) {}

pub unsafe fn CmpLinkKeyToHive(LinkKeyName: PCWSTR) -> u8 {
    if !NT_SUCCESS(Status) { return 0u8; }
    if Disposition != REG_CREATED_NEW_KEY { return 0u8; }
    1u8
}

// BOOLEAN
// CmpLinkKeyToHive(PCWSTR LinkKeyName)
// {
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("couldn't create %S, Status = 0x%lx\\n", LinkKeyName, Status);
//         return FALSE;
//     }
//     if (Disposition != REG_CREATED_NEW_KEY)
//     {
//         DPRINT1("%S already exists!\\n", LinkKeyName);
//         return FALSE;
//     }
//     return TRUE;
// }
"""
_DROPPEDPRINT_GOOD = """\
unsafe extern "C" { fn FIXME(fmt: *const i8, ...); }

pub unsafe fn CmpLinkKeyToHive(LinkKeyName: PCWSTR) -> u8 {
    if !NT_SUCCESS(Status) {
        crate::dbg_print!(b"couldn't create %S, Status = 0x%lx\\n\\0", LinkKeyName, Status);
        return 0u8;
    }
    if Disposition != REG_CREATED_NEW_KEY {
        crate::dbg_print!(b"%S already exists!\\n\\0", LinkKeyName);
        return 0u8;
    }
    1u8
}

pub unsafe extern "C" fn GdiSelectVisRgn(a: u32) { GdiSelectVisRgnImpl(a) }
unsafe fn GdiSelectVisRgnImpl(a: u32) {
    if a == 0 { crate::dbg_print!(b"no region\\n\\0"); }
}

pub unsafe fn CmpCreateControlSet(a: u32) -> u8 {
    if a == 0 {
        crate::dbg_print!(b"Failed to create ControlSet001\\n\\0");
        crate::rtl_string_cb_printf_w!(buf, 8, w!("CONTROLSET%03ld"));
        return 0u8;
    }
    1u8
}

pub unsafe fn RegisterCMMA(id: u32) -> i32 {
    FIXME(b"stub\\0".as_ptr() as *const i8);
    0
}

pub unsafe fn CmpQuiet(a: u32) -> u8 { if a == 0 { return 0u8; } 1u8 }

// BOOLEAN
// CmpLinkKeyToHive(PCWSTR LinkKeyName)
// {
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("couldn't create %S, Status = 0x%lx\\n", LinkKeyName, Status);
//         return FALSE;
//     }
//     if (Disposition != REG_CREATED_NEW_KEY)
//     {
//         DPRINT1("%S already exists!\\n", LinkKeyName);
//         return FALSE;
//     }
//     return TRUE;
// }
//
// VOID GdiSelectVisRgn(ULONG a)
// {
//     if (!a) { DPRINT1("no region\\n"); }
// }
//
// BOOLEAN CmpCreateControlSet(ULONG a)
// {
//     if (!a) { DPRINT1("Failed to create ControlSet001\\n"); return FALSE; }
//     return TRUE;
// }
//
// LONG RegisterCMMA(ULONG id)
// {
//     FIXME("stub\\n");
//     return 0;
// }
//
// BOOLEAN CmpQuiet(ULONG a)
// {
//     if (!a) { DPRINT("quiet\\n"); TRACE("quiet\\n"); WARN("quiet\\n"); return FALSE; }
//     return TRUE;
// }
//
// BOOLEAN CmpNotTranslatedHere(ULONG a)
// {
//     if (!a) { DPRINT1("no live counterpart\\n"); return FALSE; }
//     return TRUE;
// }
"""

# vendored_registry (tools/vendored.py). The bug it exists to catch is a vendored library
# arriving in dll/3rdparty or sdk/lib/3rdparty with no vendor.yaml entry — it then silently
# joins the "C we owe a translation" denominator, and nothing anywhere says so. BAD is a
# container holding one registered tree and one that is not; GOOD registers both.
#
# The stamp text is FORMATTED FROM THE TOOL'S OWN TEMPLATE rather than pasted, because a
# hand-copied stamp would make this control fail the day the template gains a line — a
# false alarm about the fixture rather than about the scanner.
_VENDOR_STAMP = vendored._STAMP_TEMPLATE.format(
    name="Known", upstream="http://example.invalid/", license="MIT"
)
_VENDOR_STAMP_2 = vendored._STAMP_TEMPLATE.format(
    name="Surprise", upstream="http://example.invalid/", license="MIT"
)
# `license_note` is present because a tree that CLAIMS a licence must be able to point
# at the grant (vendored.check). It is not what this control tests — it just keeps the
# fixture a valid registry, so a failure here means the scanner, not the fixture.
_VENDOR_TREE = """\
  - path: libs/known
    name: Known
    upstream: http://example.invalid/
    license: MIT
    license_note: fixture tree; no grant shipped
    reason: a registered vendored tree
"""
_VENDOR_BAD_YAML = "containers: [libs]\ntrees:\n" + _VENDOR_TREE
_VENDOR_GOOD_YAML = (
    _VENDOR_BAD_YAML
    + """\
  - path: libs/surprise
    name: Surprise
    upstream: http://example.invalid/
    license: MIT
    license_note: fixture tree; no grant shipped
    reason: the library that used to arrive unnoticed
"""
)

# scan_raising_probes. Seeded from ex-rust NtAllocateLocallyUniqueId (2026-10-04): the
# probe was kept, but it was the raising export, so a bad pointer from user mode was
# an unhandled exception. GOOD is the Result probe, plus the two spellings that must
# NOT fire: a qualified call (the raising C-ABI re-export) and an extern declaration.
_RAISINGPROBE_BAD = """
extern "system" {
    fn ProbeForWrite(Address: PVOID, Length: usize, Alignment: usize);
}
pub unsafe extern "system" fn NtAllocateLocallyUniqueId(LocallyUniqueId: *mut LUID) -> NTSTATUS {
    if ExGetPreviousMode() != 0 {
        ProbeForWrite(LocallyUniqueId as PVOID, core::mem::size_of::<LUID>(), 4);
    }
    STATUS_SUCCESS
}
"""
_RAISINGPROBE_GOOD = """
extern "system" {
    fn ProbeForWrite(Address: PVOID, Length: usize, Alignment: usize);
}
pub unsafe extern "C" fn ProbeForWriteUlong(address: *mut ULONG) {
    ntkrnl_sys::inline::common::ProbeForWriteUlong(address)
}
pub unsafe extern "system" fn NtAllocateLocallyUniqueId(LocallyUniqueId: *mut LUID) -> NTSTATUS {
    if ExGetPreviousMode() != 0 {
        // ProbeForWrite(LocallyUniqueId, sizeof(LUID), sizeof(ULONG));
        let probed = ntkrnl_sys::probe::probe_user_buffer(LocallyUniqueId as PVOID, 8, 4, 1);
        if let Err(status) = probed {
            return status;
        }
    }
    STATUS_SUCCESS
}
"""

# The MDL half of the same class: MmProbeAndLockPages raises, and a path-qualified
# call is still a call (win32k shipped `ntkrnl_sys::ddk::MmProbeAndLockPages(..)`).
# GOOD pins both ways out: the status form, and a call inside a real SEH frame.
_RAISINGMDL_BAD = """
pub unsafe extern "C" fn HackSecureVirtualMemory(pmdl: PMDL) -> NTSTATUS {
    ntkrnl_sys::ddk::MmProbeAndLockPages(pmdl, 1i8, 0);
    STATUS_SUCCESS
}
"""
_RAISINGMDL_GOOD = """
extern "system" {
    fn MmProbeAndLockPages(Mdl: PMDL, AccessMode: i8, Operation: u32);
}
pub unsafe extern "C" fn HackSecureVirtualMemory(pmdl: PMDL) -> NTSTATUS {
    let status = ntkrnl_sys::probe::MmTryProbeAndLockPages(pmdl, 1i8, 0);
    if status < 0 {
        return status;
    }
    match ntkrnl_sys::seh::try_except(
        &mut (),
        |_| {
            MmProbeAndLockPages(pmdl, 0i8, 1);
        },
        |_, _| 1,
    ) {
        Ok(()) => STATUS_SUCCESS,
        Err(code) => code,
    }
}
"""

# scan_raw_user_access: the store after the probe. BAD writes the caller's handle with
# a plain deref; GOOD does the same through write_back, and computing a field's
# address (addr_of_mut!) is not an access.
_RAWUSER_BAD = """
pub unsafe extern "system" fn NtOpenKeyedEvent(OutHandle: PHANDLE, Access: ULONG) -> NTSTATUS {
    let OutHandle = opaque_ptr(OutHandle);
    probe_write_generic(OutHandle, 1, 1)?;
    *OutHandle = core::ptr::null_mut();
    STATUS_SUCCESS
}
"""
_RAWUSER_GOOD = """
pub unsafe extern "system" fn NtOpenKeyedEvent(OutHandle: PHANDLE, Info: PIO_STATUS) -> NTSTATUS {
    let OutHandle = opaque_ptr(OutHandle);
    probe_write_generic(OutHandle, 1, 1)?;
    let _ = write_back(OutHandle, core::ptr::null_mut());
    let _ = write_back(core::ptr::addr_of_mut!((*Info).Information), 0);
    STATUS_SUCCESS
}
"""


# scan_double_fetch: the SAME user address captured twice on one path. BAD validates
# a length on the first read_back and sizes the copy from a second — the TOCTOU a
# thread on another CPU races. GOOD captures once and uses the kernel copy for both.
_DBLFETCH_BAD = """
pub unsafe extern "system" fn NtThing(P: *mut _UNICODE_STRING, Mode: KPROCESSOR_MODE) -> NTSTATUS {
    let hdr = match read_back(P) { Ok(v) => v, Err(s) => return s };
    if hdr.Length > limit { return STATUS_INVALID_PARAMETER; }
    let again = match read_back(P) { Ok(v) => v, Err(s) => return s };
    let _ = safe_copy_raw(dst, again.Buffer as *const _, again.Length as usize);
    STATUS_SUCCESS
}
"""
_DBLFETCH_GOOD = """
pub unsafe extern "system" fn NtThing(P: *mut _UNICODE_STRING, Mode: KPROCESSOR_MODE) -> NTSTATUS {
    let hdr = match read_back(P) { Ok(v) => v, Err(s) => return s };
    if hdr.Length > limit { return STATUS_INVALID_PARAMETER; }
    let _ = safe_copy_raw(dst, hdr.Buffer as *const _, hdr.Length as usize);
    STATUS_SUCCESS
}
"""


# scan_safe_probe_deref: a user pointer range-probed and then touched RAW. Seeded from
# ks.sys's KsFastPropertyHandler (2026-10). BAD exercises the Ks* suffix anchor and the
# trusted-kernel-arm skip: the Mode==0 copy must stay quiet, while the UserMode arm's
# raw copy_nonoverlapping after the range-only probe is the any-process BUGCHECK. GOOD
# keeps the probe for its range check and routes the copy through the fault-tolerant
# safe_copy_raw.
_SAFEPROBE_BAD = """
pub unsafe fn KsFastPropertyHandler(Property: *const KSPROPERTY, Mode: i8) -> BOOLEAN {
    let mut PropRequest: KSPROPERTY = core::mem::zeroed();
    let dst = &mut PropRequest as *mut KSPROPERTY as *mut u8;
    if Mode == 0i8 {
        core::ptr::copy_nonoverlapping(Property as *const u8, dst, 24);
    } else {
        if !NT_SUCCESS(KsSafeProbeForRead(Property as *const c_void, 24, 1)) { return FALSE; }
        core::ptr::copy_nonoverlapping(Property as *const u8, dst, 24);
    }
    TRUE
}
"""
_SAFEPROBE_GOOD = """
pub unsafe fn KsFastPropertyHandler(Property: *const KSPROPERTY, Mode: i8) -> BOOLEAN {
    let mut PropRequest: KSPROPERTY = core::mem::zeroed();
    let dst = &mut PropRequest as *mut KSPROPERTY;
    if Mode == 0i8 {
        core::ptr::copy_nonoverlapping(Property as *const u8, dst as *mut u8, 24);
    } else {
        if !NT_SUCCESS(KsSafeProbeForRead(Property as *const c_void, 24, 1)) { return FALSE; }
        if ntkrnl_sys::probe::safe_copy_raw(
            dst as *mut c_void, Property as *const c_void, 24,
        ).is_err() { return FALSE; }
    }
    TRUE
}
"""


# scan_dropped_seh_pairs. The LPC shape: the C is cohab-flipped OUT of the build
# (commented in the CMake source list) and a separate .rs took over under
# #[no_mangle], so the two files are never in the same place and the inline
# scanner above is structurally blind to it. BAD keeps the probe and loses the
# handler; GOOD routes the same probe through the validate-first substrate. The
# GOOD half also pins the filter that matters most: it is the presence of a
# CATCH idiom that clears a pair, not the absence of a probe.
_SEHPAIR_CMAKE = """
list(APPEND SOURCE
    #${REACTOS_SOURCE_DIR}/ntoskrnl/lpc/reply.c
)
"""
_SEHPAIR_C = """
NTSTATUS NtReplyPort(HANDLE PortHandle, PPORT_MESSAGE ReplyMessage)
{
    if (PreviousMode != KernelMode)
    {
        _SEH2_TRY
        {
            ProbeForRead(ReplyMessage, sizeof(*ReplyMessage), sizeof(ULONG));
            Captured = *(volatile PORT_MESSAGE*)ReplyMessage;
        }
        _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
        {
            _SEH2_YIELD(return _SEH2_GetExceptionCode());
        }
        _SEH2_END;
    }
    return STATUS_SUCCESS;
}
"""
_SEHPAIR_BAD_RS = """
#[no_mangle]
pub unsafe extern "system" fn NtReplyPort(ReplyMessage: PPORT_MESSAGE) -> NTSTATUS {
    if previous_mode != 0i8 {
        ProbeForRead(ReplyMessage as PVOID, core::mem::size_of::<PORT_MESSAGE>(), 4);
        let captured = *(ReplyMessage as *const PORT_MESSAGE);
    }
    STATUS_SUCCESS
}
"""
_SEHPAIR_GOOD_RS = """
#[no_mangle]
pub unsafe extern "system" fn NtReplyPort(ReplyMessage: PPORT_MESSAGE) -> NTSTATUS {
    if previous_mode != 0i8 {
        let captured = match ntkrnl_sys::probe::probe_read_generic::<PORT_MESSAGE>(
            ReplyMessage as *const PORT_MESSAGE, 4, previous_mode,
        ) {
            Ok(v) => v,
            Err(st) => return st,
        };
    }
    STATUS_SUCCESS
}
"""

# scan_dropped_probes. The NtSetValueKey shape (#79): the C probes Data inside one
# _SEH2_TRY and copies it inside a second, both regions dropped, leaving a bare
# `Status = STATUS_SUCCESS` where the bounds check belongs. The GOOD half proves the
# scanner clears once validation is restored through the substrate — including the
# turbofish spelling, which an earlier revision of the call regex could not see.
_DROPPEDPROBE_BAD = """
pub unsafe fn NtSetValueKey(Data: PVOID, DataSize: ULONG) -> NTSTATUS {
    let mut Status = STATUS_SUCCESS;
    core::ptr::copy_nonoverlapping(Data as *const u8, DataCopy as *mut u8, DataSize as usize);
    Status
}
// NTSTATUS NtSetValueKey(PVOID Data, ULONG DataSize)
// {
//     _SEH2_TRY { ProbeForRead(Data, DataSize, 1); } _SEH2_END;
//     RtlCopyMemory(DataCopy, Data, DataSize);
// }
"""

_DROPPEDPROBE_GOOD = """
pub unsafe fn NtSetValueKey(Data: PVOID, DataSize: ULONG) -> NTSTATUS {
    probe_user_buffer(Data as *const c_void, DataSize as usize, 1, PreviousMode)?;
    let _hdr = probe_read_user::<u32>(Data as *const u32)?;
    core::ptr::copy_nonoverlapping(Data as *const u8, DataCopy as *mut u8, DataSize as usize);
    STATUS_SUCCESS
}
// NTSTATUS NtSetValueKey(PVOID Data, ULONG DataSize)
// {
//     _SEH2_TRY { ProbeForRead(Data, DataSize, 1); } _SEH2_END;
//     RtlCopyMemory(DataCopy, Data, DataSize);
// }
// The write-side substrate counts too: IopCreateFile's ProbeForWriteHandle became
// probe_write_generic (2026-10-04), and the scanner first reported it as dropped.
pub unsafe fn NtOpenSection(SectionHandle: *mut HANDLE) -> NTSTATUS {
    if let Err(status) = ntkrnl_sys::probe::probe_write_generic(SectionHandle, 1, PreviousMode) {
        return status;
    }
    *SectionHandle = Handle;
    STATUS_SUCCESS
}
// NTSTATUS NtOpenSection(PHANDLE SectionHandle)
// {
//     _SEH2_TRY { ProbeForWriteHandle(SectionHandle); } _SEH2_END;
//     *SectionHandle = Handle;
// }
"""

# scan_dropped_asserts. Seeded from task #39, 2026-09-06: MI_MAKE_HARDWARE_PTE_USER
# is a FORCEINLINE in miarm.h whose first four statements are asserts ("check that
# we are not setting valid a page that should not be"), and BOTH hand-written Rust
# bridges shipped zero of them. On x86 the missing precondition is invisible; on
# arm64 MmProtectToPteMask is the only source of the L3 descriptor-type bit, so a
# no-access mask yields Valid=1 on a descriptor the MMU rejects and the machine
# stops with no bugcheck. The GOOD case pins the two ways this scanner could go
# noisy: an assert QUOTED in a trailing C comment is not one that runs (rustconv
# parks the original C there, so counting it would certify the whole translated
# population), and a cfg-split twin that legitimately checks less must not report
# while its sibling still carries the assert.
_DROPPED_ASSERT_H = """
FORCEINLINE
VOID
MI_MAKE_HARDWARE_PTE_USER(IN PMMPTE NewPte, IN ULONG_PTR ProtectionMask)
{
    /* Check that we are not setting valid a page that should not be */
    ASSERT(ProtectionMask & MM_PROTECT_ACCESS);
    ASSERT((ProtectionMask & MM_PROTECT_SPECIAL) != MM_GUARDPAGE);
    NewPte->u.Hard.Valid = TRUE;
}
"""

_DROPPED_ASSERT_BAD = """
pub unsafe fn MI_MAKE_HARDWARE_PTE_USER(new_pte: *mut MMPTE, mask: usize) {
    // ASSERT(ProtectionMask & MM_PROTECT_ACCESS);
    (*new_pte).set_valid(true);
}
"""

_DROPPED_ASSERT_GOOD = """
#[cfg(target_arch = "aarch64")]
pub unsafe fn MI_MAKE_HARDWARE_PTE_USER(new_pte: *mut MMPTE, mask: usize) {
    ASSERT!(mask & MM_PROTECT_ACCESS != 0);
    ASSERT!(mask & MM_PROTECT_SPECIAL != MM_GUARDPAGE);
    (*new_pte).set_valid(true);
}

#[cfg(not(target_arch = "aarch64"))]
pub unsafe fn MI_MAKE_HARDWARE_PTE_USER(new_pte: *mut MMPTE, mask: usize) {
    (*new_pte).set_valid(true);
}
"""


# scan_unpruned_walk. Seeded from the `source-early` move (2026-09-06): six audited
# scanners enumerated Rust sources with `rglob("*.rs")` and dropped build output with a
# post-filter, which is too late by construction — rglob has already descended into
# sdk/lib/rust/target. Run beside the builds instead of after them, cargo deleting a
# directory underneath raised FileNotFoundError out of the generator and the gate
# reported `lint:source-early: NEW lint findings (exit 1)` with no findings in it.
# The GOOD case pins the ways this scanner must NOT fire, all of which a first draft got
# wrong: the source_files() spelling that is the fix, a recursive walk that crosses no
# mutable tree, a line carrying the explicit opt-out marker that the discriminating
# control test needs, and the pruned build-dir walk.
# The BUILD-dir half was added 2026-09-13 after three tools were found walking a build
# tree with a post-filter (scan_ud2, scan_self_recursive, scan_flex_array_panic) — none
# of them a Rust walk, so the original rule could not see any of them. Note the two
# halves of the GOOD non-firing case: the pattern alone is not the finding, the pattern
# rooted at a mutable tree is.
_UNPRUNED_BAD = """\
from pathlib import Path


def collect(root: Path):
    for p in root.rglob("*.rs"):
        if "/target/" in str(p):
            continue
        yield p


def collect_other(root: Path):
    for p in root.glob("**/*.rs"):
        yield p


def find_lib(build: Path, want: str):
    # The BUILD-dir half of the class: not a Rust walk at all, and the post-filter
    # names the very tree the descent must not enter.
    for p in build.rglob(want):
        if "-target/" not in str(p):
            return p
    return None
"""
_UNPRUNED_GOOD = """\
import os
from pathlib import Path

from source_walk import source_files


def collect(root: Path):
    yield from source_files(root)


def collect_pyfiles(root: Path):
    # Not a Rust walk and not rooted at a build dir, so it crosses no mutable tree
    # and is not this lint's business. Both halves of that matter: the same spelling
    # rooted at a BUILD dir is a finding (see find_lib in the bad fixture).
    yield from root.rglob("*.py")


def find_lib(build: Path, want: str):
    # Pruned during the descent, so the churning tree is never entered.
    for dirpath, dirnames, filenames in os.walk(build):
        dirnames[:] = [d for d in dirnames if not d.endswith("-target")]
        if want in filenames:
            return Path(dirpath) / want
    return None


def deliberate_control(root: Path):
    return root.rglob("*.rs")  # unpruned-walk: intentional
"""

# scan_com_double_release. Seeded from CDefView 2026-09-19: `CComPtr<IEnumIDList>`
# lowered to a raw pointer plus a CppComPtrGuard, with the explicit Release() calls
# ALSO kept, so every FillList released twice and the guard then called through a
# dangling vtable. arm64 died on it at once; amd64 and i386 ran the same code and
# passed the 91-test conformance sweep three times. The GOOD case pins the three
# spellings that are correct and must not be reported: release-then-null,
# release-then-reassign (a hand-over-hand walk), and a release with no guard at all.
_COMREL_BAD = """
pub unsafe fn FillList_body(this: *mut T) -> HRESULT {
    let mut pEnum: *mut IEnumIDList = core::ptr::null_mut();
    let _pEnum_cpp_com_guard = CppComPtrGuard::new(core::ptr::addr_of_mut!(pEnum));
    if !pEnum.is_null() {
        ((*(*pEnum).lpVtbl).Release)(pEnum);
    }
    return S_OK;
}
"""

_COMREL_GOOD = """
pub unsafe fn Exec_body(this: *mut T) -> HRESULT {
    let mut pStream: *mut IStream = core::ptr::null_mut();
    let _pStream_cpp_com_guard = CppComPtrGuard::new(core::ptr::addr_of_mut!(pStream));
    if !pStream.is_null() {
        ((*(*pStream).lpVtbl).Release)(pStream);
        pStream = core::ptr::null_mut();
    }
    return S_OK;
}

pub unsafe fn IsSubMenuParent_body(hwnd: HWND) -> BOOL {
    let _popup_cpp_com_guard = CppComPtrGuard::new(core::ptr::addr_of_mut!(popup));
    while !popup.is_null() {
        ((*(*popup).lpVtbl).Release)(popup);
        popup = next;
    }
    FALSE
}

pub unsafe fn SaveViewState_body(this: *mut T) -> HRESULT {
    let mut pPB: *mut IPropertyBag = core::ptr::null_mut();
    ((*(*pPB).lpVtbl).Release)(pPB);
    return S_OK;
}
"""


# scan_duplicate_symbols. 88b04bf7f86: reviving an orphan-cfg'd empty Rust stub put a
# second i386 `@RtlPrefetchMemoryNonTemporal@8` in the crate whose global_asm! already
# defines it. rustc can't see into the assembler and --allow-multiple-definition picked
# the stub: BUGCHECK 0x1E in kmtest RtlMemoryKM. The GOOD tree carries the three shapes
# that must stay quiet: the stub gated off x86, smp/not-smp feature twins, and a cdecl
# asm label beside a stdcall Rust fn of the same name (two different i386 symbols).
_DUP_CARGO = """\
[package]
name = "rtlk-rust"
version = "0.1.0"

[lib]
crate-type = ["staticlib"]
path = "src/lib.rs"

[features]
smp = []
"""
_DUP_ASM = """\
#[cfg(target_arch = "x86")]
core::arch::global_asm!(
    ".globl @RtlPrefetchMemoryNonTemporal@8",
    "@RtlPrefetchMemoryNonTemporal@8:",
    "    ret",
    ".globl _KeGetPcr",
    "_KeGetPcr:",
    "    ret",
);
"""
_DUP_BAD = {
    "rtlk-rust/Cargo.toml": _DUP_CARGO,
    "rtlk-rust/src/lib.rs": "mod prefetch_asm;\nmod misc;\n",
    "rtlk-rust/src/prefetch_asm.rs": _DUP_ASM,
    "rtlk-rust/src/misc.rs": """\
#[no_mangle]
pub unsafe extern "fastcall" fn RtlPrefetchMemoryNonTemporal(_Source: PVOID, _Length: SIZE_T) {}
""",
}
_DUP_GOOD = {
    "rtlk-rust/Cargo.toml": _DUP_CARGO,
    "rtlk-rust/src/lib.rs": (
        'mod prefetch_asm;\n#[cfg(not(target_arch = "x86"))]\nmod misc;\nmod lock;\n'
    ),
    "rtlk-rust/src/prefetch_asm.rs": _DUP_ASM,
    "rtlk-rust/src/misc.rs": """\
#[no_mangle]
pub unsafe extern "C" fn RtlPrefetchMemoryNonTemporal(_Source: PVOID, _Length: SIZE_T) {}
""",
    "rtlk-rust/src/lock.rs": """\
#[cfg(feature = "smp")]
#[no_mangle]
pub unsafe extern "fastcall" fn KxAcquireSpinLock(l: *mut usize) { spin(l) }
#[cfg(not(feature = "smp"))]
#[no_mangle]
pub unsafe extern "fastcall" fn KxAcquireSpinLock(_l: *mut usize) {}
#[no_mangle]
pub unsafe extern "system" fn KeGetPcr() -> usize { 0 }
""",
}


# scan_export_shadows. classpnp, 2026-10-07: `#[no_mangle] RtlInterlockedPopEntrySList`
# returning NULL beat the importlib, so every transfer-packet free list dropped its
# entries. The name is the FORWARD TARGET of `ExpInterlockedPopEntrySList`, not the
# export itself — the column the scanner must read. The GOOD tree keeps a marked
# header inline, a name the kernel does not export, and a crate without ntkrnl-sys.
_EXPORT_SHADOW_SPECS = {
    "ntoskrnl/ntoskrnl.spec": (
        "@ stdcall -arch=x86_64,arm64 ExpInterlockedPopEntrySList(ptr ptr)"
        " RtlInterlockedPopEntrySList\n"
        "@ stdcall KeInitializeSpinLock(ptr)\n"
    ),
    "hal/hal.spec": "@ extern KdComPortInUse\n",
    "ntkrnl-sys/Cargo.toml": '[package]\nname = "ntkrnl-sys"\nversion = "0.1.0"\n',
    "ntkrnl-sys/src/lib.rs": "",
}
_EXPORT_SHADOW_DRV_CARGO = """\
[package]
name = "drv-rust"
version = "0.1.0"

[lib]
crate-type = ["staticlib"]
path = "src/lib.rs"

[dependencies]
ntkrnl-sys = { path = "../ntkrnl-sys" }
"""
_EXPORT_SHADOW_BAD = {
    **_EXPORT_SHADOW_SPECS,
    "drv-rust/Cargo.toml": _EXPORT_SHADOW_DRV_CARGO,
    "drv-rust/src/lib.rs": """\
#[no_mangle]
pub unsafe extern "system" fn RtlInterlockedPopEntrySList(_h: PVOID) -> PVOID { null_mut() }
""",
}
_EXPORT_SHADOW_GOOD = {
    **_EXPORT_SHADOW_SPECS,
    "drv-rust/Cargo.toml": _EXPORT_SHADOW_DRV_CARGO,
    "drv-rust/src/lib.rs": """\
// nolint: export-shadow (wdm.h FORCEINLINE; the driver is expected to carry it)
#[no_mangle]
pub unsafe extern "C" fn KeInitializeSpinLock(l: *mut usize) { *l = 0; }
#[no_mangle]
pub unsafe extern "system" fn DrvPopEntry(_h: PVOID) -> PVOID { null_mut() }
""",
    "user-rust/Cargo.toml": _EXPORT_SHADOW_DRV_CARGO.replace("drv-rust", "user-rust").replace(
        '[dependencies]\nntkrnl-sys = { path = "../ntkrnl-sys" }\n', ""
    ),
    "user-rust/src/lib.rs": """\
#[no_mangle]
pub unsafe extern "system" fn RtlInterlockedPopEntrySList(_h: PVOID) -> PVOID { null_mut() }
""",
}


# dhcpcsvc, 2026-10-04: `(*ip).name` is a `[c_char; 256]`, passed by value to a variadic
# logger. i386 pushes the bytes, so `%s` dereferenced "nets". The good side keeps the
# `.as_ptr()` fix and a same-named POINTER field, which a name-only match would flag.
_VARIADIC_ARRAY_BAD = """\
pub struct interface_info { pub next: *mut interface_info, pub name: [c_char; 256] }
extern "C" { pub fn note(format: *const c_char, ...) -> c_int; }
unsafe fn send_discover(ipp: *mut c_void) {
    let ip = ipp as *mut interface_info;
    note(b"DHCPDISCOVER on %s\\0".as_ptr() as *const i8, (*ip).name);
}
"""
_VARIADIC_ARRAY_GOOD = """\
pub struct interface_info { pub next: *mut interface_info, pub name: [c_char; 256] }
pub struct universe { pub name: *const c_char }
extern "C" { pub fn note(format: *const c_char, ...) -> c_int; }
unsafe fn send_discover(ipp: *mut c_void, u: *mut universe) {
    let ip = ipp as *mut interface_info;
    note(b"DHCPDISCOVER on %s\\0".as_ptr() as *const i8, (*ip).name.as_ptr());
    note(b"%s\\0".as_ptr() as *const i8, (*u).name);
}
"""

# The bare-variable half (2026-10-10): sndrec32 passed `static mut str_mono: [u16; 10]` to
# `gprintf`, its alias of the variadic `_snwprintf`; i386 read "st" as the %s pointer.
_VARIADIC_ARRAY_BARE_BAD = """\
extern "C" { pub fn _snwprintf(b: *mut u16, n: usize, f: *const u16, ...) -> i32; }
pub use crate::prelude::_snwprintf as gprintf;
pub static mut str_mono: [u16; 10] = [0; 10];
pub static mut str_stereo: [u16; 10] = [0; 10];
unsafe fn paint(t: *mut u16, f: *const u16, two: bool) {
    gprintf(t, 100, f, if two { str_stereo } else { str_mono });
}
"""
_VARIADIC_ARRAY_BARE_GOOD = """\
extern "C" { pub fn _snwprintf(b: *mut u16, n: usize, f: *const u16, ...) -> i32; }
pub use crate::prelude::_snwprintf as gprintf;
pub static mut str_mono: [u16; 10] = [0; 10];
pub static mut scroll: [u8; 4] = [0; 4];
unsafe fn paint(t: *mut u16, f: *const u16, scroll: i32) {
    gprintf(t, 100, f, str_mono.as_ptr());
    gprintf(t, 100, f, scroll);
    /* gprintf(t, 100, f, str_mono) */
}
"""

# scan_byteptr_stride. Seeded from oleaut32 IRecordInfoImpl_RecordCopy (2026-10-04): the C's
# `((BYTE*)src_rec) + offset` translated through `*mut PBYTE`, an 8x stride.
_BYTEPTR_BAD = """\
unsafe fn copy(src_rec: *mut c_void, off: isize) -> *mut c_void {
    (src_rec as *mut PBYTE)
        .offset(off) as *mut c_void
}
"""
_BYTEPTR_GOOD = """\
unsafe fn copy(src_rec: *mut c_void, off: isize) -> *mut c_void {
    (src_rec as *mut BYTE).offset(off) as *mut c_void
}
"""

# scan_gmem_flags. Seeded from oleaut32 create_stream (2026-10-04): the C's GMEM_FIXED came
# out as HEAP_ZERO_MEMORY, which GlobalAlloc rejects.
_GMEM_BAD = """\
unsafe fn load(n: u32) -> HGLOBAL {
    GlobalAlloc(HEAP_ZERO_MEMORY, n as SIZE_T)
}
"""
_GMEM_GOOD = """\
unsafe fn load(n: u32) -> HGLOBAL {
    GlobalAlloc(GMEM_FIXED, n as SIZE_T)
}
"""

# scan_raw_bitfield. Seeded from atapi's AtaReqInformationalExceptionsControlModePage
# (2026-10-08): the page code passed as the WIDTH of a 1-byte bitfield unit, a panic.
_RAWBF_BAD = """\
unsafe fn page(p: *mut MODE_INFO_EXCEPTIONS) {
    (*p)._bitfield_1.set(0, 0x1C, 0x1C);
}
"""
_RAWBF_GOOD = """\
unsafe fn page(p: *mut MODE_INFO_EXCEPTIONS) {
    (*p).set_PageCode(0x1C);
}
"""

# scan_let_shadow. Seeded from freeldr EnumerateHarddisks (2026-10-04): C `DiskCount++`
# became a nested `let _DiskCount`, so the BIOS disk count stayed 0.
_SHADOW_BAD = """\
unsafe fn enumerate() -> u8 {
    let mut DiskCount: u8 = 0;
    while probe() {
        let _DiskCount = DiskCount + 1;
    }
    DiskCount
}
"""
_SHADOW_GOOD = """\
unsafe fn enumerate() -> u8 {
    let mut DiskCount: u8 = 0;
    while probe() {
        DiskCount += 1;
    }
    DiskCount
}
"""

# scan_winerror_consts. Seeded from oleaut32 olesys.rs (2026-10-05): the typelib error
# table held neighbours' values, so LoadTypeLib reported TYPE_E_AMBIGUOUSNAME.
_WINERR_BAD = """\
pub const TYPE_E_CANTLOADLIBRARY: HRESULT = 0x8002_802Cu32 as HRESULT;
"""
_WINERR_GOOD = """\
pub const TYPE_E_CANTLOADLIBRARY: HRESULT = 0x8002_9C4Au32 as HRESULT;
"""

# scan_list_walk_from_head. Seeded from rpcss IrotIsRunning (2026-10-11): the cursor
# started at the list head it stops at, so the walk ran zero times.
_LISTWALK_BAD = """\
unsafe fn find(data: u32) -> i32 {
    e = (core::ptr::addr_of_mut!(Table) as *mut u8).sub(core::mem::offset_of!(item, entry)) as *mut item;
    while core::ptr::addr_of_mut!((*e).entry) != core::ptr::addr_of_mut!(Table) {
        e = ((*e).entry.next as *mut u8).sub(core::mem::offset_of!(item, entry)) as *mut item;
    }
    0
}
"""
_LISTWALK_GOOD = _LISTWALK_BAD.replace(
    "(core::ptr::addr_of_mut!(Table) as *mut u8)", "(Table.next as *mut u8)"
)

# scan_dowhile_continue. Seeded from syssetup RegisterTypeLibraries (2026-10-04): a C
# do-while's `continue` re-ran the same INF line forever once translated to `loop`.
# scan_case_bitor_pattern. Seeded from comctl32 TREEVIEW_SelectItem (2026-10-09): C's
# `case TVGN_CARET|TVSI_NOSINGLEEXPAND:` is ONE value, the OR; the same text as a Rust
# arm is an or-pattern matching EITHER constant. The GOOD case is the faithful guard.
_CASE_BITOR_C = """\
switch (action) {
case TVGN_CARET|TVSI_NOSINGLEEXPAND:
    bSingleExpand = FALSE;
    break;
}
"""

_CASE_BITOR_BAD = """\
unsafe fn select(action: u32) {
    match action {
        TVGN_CARET | TVSI_NOSINGLEEXPAND => { single = 0; }
        _ => {}
    }
}
"""

_CASE_BITOR_GOOD = """\
unsafe fn select(action: u32) {
    match action {
        x if x == TVGN_CARET | TVSI_NOSINGLEEXPAND => { single = 0; }
        _ => {}
    }
}
"""

_DOWHILE_BAD = """\
unsafe fn register(ctx: *mut Ctx) {
    loop {
        if read_line(ctx) == 0 {
            continue;
        }
        if next_line(ctx) == 0 {
            break;
        }
    }
}
"""
_DOWHILE_GOOD = """\
unsafe fn register(ctx: *mut Ctx) {
    loop {
        'next: {
            if read_line(ctx) == 0 {
                break 'next;
            }
        }
        if next_line(ctx) == 0 {
            break;
        }
    }
}
"""


_FORMATLESS_PRINTF_BAD = """\
#[no_mangle]
pub unsafe extern "C" fn RtlStringCbPrintfW(d: *mut u16, cb: usize, fmt: *const u16) -> i32 {
    0
}
"""

# The fixed shapes: fixed slots replayed as a va_list, and a v-form taking one.
_FORMATLESS_PRINTF_GOOD = """\
#[no_mangle]
pub unsafe extern "C" fn KdIoPrintf(Format: *const i8, a0: usize, a1: usize) {}
#[no_mangle]
pub unsafe extern "C" fn _vsnwprintf(b: *mut u16, n: usize, fmt: *const u16, ap: *mut u8) {}
"""

_BYTE_STEP_ONE_BAD = """\
pub unsafe fn folders(h: *mut CFHEADER) -> *mut u8 {
    // C: (PUCHAR)(PCABHeader + 1) -- past the whole header, not one byte.
    (h as *mut u8).add(1)
}
"""

_BYTE_STEP_ONE_GOOD = """\
pub unsafe fn folders(h: *mut CFHEADER) -> *mut u8 {
    h.add(1) as *mut u8
}
pub unsafe fn len_plus_nul(p: *const i8) -> usize {
    strlen(p as *const i8).wrapping_add(1)
}
"""

_PTR_TYPEDEF_CAST_READ_BAD = """\
pub unsafe fn locale(e: PVOID) -> PCWSTR {
    // C: ((PGENENTRY)e)->Id
    (*(e as *mut PGENENTRY).read()).Id
}
"""

_PTR_TYPEDEF_CAST_READ_GOOD = """\
pub unsafe fn locale(e: PVOID) -> PCWSTR {
    (*(e as PGENENTRY)).Id
}
pub unsafe fn slot(h: *mut PVOID) -> PVOID {
    (h as *mut PVOID).read()
}
"""

_INFO_CLASS_BAD = """\
pub unsafe fn extend(fo: PFILE_OBJECT, size: *mut LARGE_INTEGER) -> NTSTATUS {
    // C: FileEndOfFileInformation (20). 2 is FileFullDirectoryInformation.
    IoSetInformation(fo, 2u32, 8, size as PVOID)
}
"""

_INFO_CLASS_GOOD = """\
pub unsafe fn extend(fo: PFILE_OBJECT, size: *mut LARGE_INTEGER) -> NTSTATUS {
    IoSetInformation(fo, _FILE_INFORMATION_CLASS_FileEndOfFileInformation as u32, 8, size as PVOID)
}
extern "system" {
    fn IoSetInformation(fo: PFILE_OBJECT, class: u32, len: u32, buf: PVOID) -> NTSTATUS;
}
"""

_CONST_NARROWING_BAD = """\
pub const SRB_CLASS_FLAGS_PERSISTANT: u32 = 0x20000000;
pub unsafe fn flush(srb: *mut SRB) {
    (*srb).SrbFlags |= (SRB_CLASS_FLAGS_PERSISTANT as USHORT) as u32;
}
"""

# The fix, plus the all-ones idiom that must stay quiet (WEOF, a 0xFF fill byte).
_CONST_NARROWING_GOOD = """\
pub const SRB_CLASS_FLAGS_PERSISTANT: u32 = 0x20000000;
pub const EOF: i32 = -1;
pub const BLOCK_UNUSED: u32 = 0xffffffff;
pub unsafe fn flush(srb: *mut SRB) {
    (*srb).SrbFlags |= SRB_CLASS_FLAGS_PERSISTANT;
    let weof = EOF as u16;
    let fill = BLOCK_UNUSED as u8;
}
"""

_NARROW_WIDE_BAD = """\
pub unsafe fn pattern(FilePattern: UNICODE_STRING) -> bool {
    RtlCompareMemory(FilePattern.Buffer as PVOID, b"*.*\\0".as_ptr() as PVOID, 6) == 6
}
// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//     if (RtlCompareMemory(FilePattern.Buffer, L"*.*", 6) == 6)
"""

# The fix, a macro that builds UTF-16 from a byte literal, and a narrow literal the C also
# passes narrow.
_NARROW_WIDE_GOOD = """\
pub unsafe fn pattern(FilePattern: UNICODE_STRING) -> bool {
    RtlCompareMemory(FilePattern.Buffer as PVOID, STAR_DOT_STAR_W.as_ptr() as PVOID, 6) == 6
        && RtlInitUnicodeString(&mut s, wstr!(b"*.*\\0")) == 0
        && DbgPrint(b"hi\\0".as_ptr()) == 0
}
// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//     if (RtlCompareMemory(FilePattern.Buffer, L"*.*", 6) == 6)
//     RtlInitUnicodeString(&s, L"*.*");
//     DbgPrint("hi");
"""

_CALL_ARG_SWAP_BAD = """\
pub unsafe extern "system" fn InitializeProfiles() -> BOOL {
    let mut szProfilesPath: [WCHAR; 260] = [0; 260];
    let mut szBuffer: [WCHAR; 260] = [0; 260];
    LoadStringW(hInstance, IDS_PROFILEPATH, szProfilesPath.as_mut_ptr(), 260);
    ExpandEnvironmentStringsW(szBuffer.as_mut_ptr(), szProfilesPath.as_mut_ptr(), 260);
    1
}
// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
// BOOL
// InitializeProfiles(VOID)
// {
//     WCHAR szProfilesPath[MAX_PATH];
//     WCHAR szBuffer[MAX_PATH];
//     LoadStringW(hInstance, IDS_PROFILEPATH, szBuffer, ARRAYSIZE(szBuffer));
//     ExpandEnvironmentStringsW(szBuffer, szProfilesPath, ARRAYSIZE(szProfilesPath));
//     return TRUE;
// }
"""

# The fix, plus a Rust alias of the C's variable (`let key = InputBuffer`), which is the same.
_CALL_ARG_SWAP_GOOD = """\
pub unsafe extern "system" fn InitializeProfiles() -> BOOL {
    let mut szProfilesPath: [WCHAR; 260] = [0; 260];
    let mut szBuffer: [WCHAR; 260] = [0; 260];
    LoadStringW(hInstance, IDS_PROFILEPATH, szBuffer.as_mut_ptr(), 260);
    ExpandEnvironmentStringsW(szBuffer.as_mut_ptr(), szProfilesPath.as_mut_ptr(), 260);
    1
}
pub unsafe fn ReadKey(InputBuffer: PVOID) {
    let keyParameters: PVOID = InputBuffer;
    ReadDvd(keyParameters);
}
// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
// BOOL
// InitializeProfiles(VOID)
// {
//     WCHAR szProfilesPath[MAX_PATH];
//     WCHAR szBuffer[MAX_PATH];
//     LoadStringW(hInstance, IDS_PROFILEPATH, szBuffer, ARRAYSIZE(szBuffer));
//     ExpandEnvironmentStringsW(szBuffer, szProfilesPath, ARRAYSIZE(szProfilesPath));
//     return TRUE;
// }
// VOID
// ReadKey(PVOID InputBuffer)
// {
//     PVOID keyParameters;
//     PVOID InputBuffer;
//     ReadDvd(InputBuffer);
// }
"""

_SSE_HELPERS_BAD = """\
#[inline]
pub fn IS_DIRTY_SSE(entry: ULONG_PTR) -> bool {
    (entry & 0x00000008) != 0
}
"""

# mm.h's encoding, including a cfg'd 64-bit mask and a helper built from its siblings.
_SSE_HELPERS_GOOD = """\
#[inline]
fn IS_DIRTY_SSE(sse: ULONG_PTR) -> bool {
    (sse & 0x2) != 0
}
#[inline]
fn PAGE_FROM_SSE(sse: ULONG_PTR) -> ULONG_PTR {
    #[cfg(target_pointer_width = "64")]
    { sse & 0xFFFFFFFFF000 }
    #[cfg(target_pointer_width = "32")]
    { sse & 0xFFFFF000 }
}
#[inline]
fn BUMPREF_SSE(sse: ULONG_PTR) -> ULONG_PTR {
    PAGE_FROM_SSE(sse) | ((SHARE_COUNT_FROM_SSE(sse) + 1) << 3) | (sse & 0x7)
}
"""

_READ_AFTER_CLEAR_BAD = """\
pub unsafe fn delete(PointerPte: PMMPTE, WasDirty: *mut u8) {
    let mut OldPte = MMPTE { Long: 0 };
    OldPte.Long = InterlockedExchangePte(PointerPte, 0);
    if OldPte.Long != 0 {
        *WasDirty = (((*PointerPte).Long >> 6) & 1) as u8;
    }
}
"""

# The fix; and an exchange with a non-zero value, whose location is still meaningful after.
_READ_AFTER_CLEAR_GOOD = """\
pub unsafe fn delete(PointerPte: PMMPTE, WasDirty: *mut u8) {
    let mut OldPte = MMPTE { Long: 0 };
    OldPte.Long = InterlockedExchangePte(PointerPte, 0);
    if OldPte.Long != 0 {
        *WasDirty = ((OldPte.Long >> 6) & 1) as u8;
    }
}
pub unsafe fn protect(PointerPte: PMMPTE, TempPte: MMPTE) -> usize {
    let _ = InterlockedExchangePte(PointerPte, TempPte.Long);
    (*PointerPte).Long
}
"""

_INIT_THEN_MOVE_BAD = """\
pub unsafe fn shutdown() {
    let mut Event: core::mem::MaybeUninit<_> = core::mem::MaybeUninit::uninit();
    KeInitializeEvent(Event.as_mut_ptr(), _EVENT_TYPE_NotificationEvent, 0u8);
    let Event = Event.assume_init();
    send(core::ptr::addr_of!(Event) as *mut _);
}
"""

# The fix; and a MaybeUninit filled by a non-initializer and then read, which is fine.
_INIT_THEN_MOVE_GOOD = """\
pub unsafe fn shutdown() {
    let mut Event: KEVENT = core::mem::zeroed();
    KeInitializeEvent(&mut Event, _EVENT_TYPE_NotificationEvent, 0u8);
    send(&mut Event);
    let mut tmp = core::mem::MaybeUninit::<u32>::uninit();
    REVERSE_BYTES(tmp.as_mut_ptr(), &x);
    let v = tmp.assume_init();
}
"""

_CONTAINING_RECORD_NULL_BAD = """\
pub unsafe fn walk(e: PLIST_ENTRY) -> PDOPE {
    CONTAINING_RECORD(
        e as PVOID,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    ) as PDOPE
}
"""

# The fix; and a declaration, which has the name but no call.
_CONTAINING_RECORD_NULL_GOOD = """\
extern "C" {
    fn CONTAINING_RECORD(address: PVOID, type_size: usize, field_offset: usize) -> PVOID;
}
pub unsafe fn walk(e: PLIST_ENTRY) -> PDOPE {
    CONTAINING_RECORD(
        e as PVOID,
        core::mem::size_of::<DOPE>(),
        core::mem::offset_of!(DOPE, Volume),
    ) as PDOPE
}
"""

_COMMENT_SWALLOWED_BAD = """\
pub unsafe fn load(r: u32) -> i32 {
    match r { 1 => { /* Fail */ // DPRINT removed return STATUS_NOT_REGISTRY_FILE;
    } _ => {} }
    0
}
"""

# The fix, plus prose comments that mention code without being code.
_COMMENT_SWALLOWED_GOOD = """\
pub unsafe fn load(r: u32) -> i32 {
    match r {
        1 => {
            return STATUS_NOT_REGISTRY_FILE; // DPRINT removed: the C printed here
        }
        _ => {}
    }
    // DPRINT removed return X; (a whole-line comment is prose, never code)
    0
}
"""

_CALL_LITERAL_BAD = """\
const CM_CHECK_REGISTRY_PURGE_VOLATILES: u32 = 0x00000002;
pub unsafe fn load() {
    CmpCmdHiveOpen(file, ctx, &mut a, &mut h, 0x00000001u32);
}
// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//     Status = CmpCmdHiveOpen(SourceFile, &Ctx, &Allocate, &CmHive,
//                             CM_CHECK_REGISTRY_PURGE_VOLATILES);
"""

# The value the C names (fine), a literal where the C also passes a literal (fine), and a
# call after `!` in the C that must still count as a call.
_CALL_LITERAL_GOOD = """\
const CM_CHECK_REGISTRY_PURGE_VOLATILES: u32 = 0x00000002;
pub unsafe fn load() {
    CmpCmdHiveOpen(file, ctx, &mut a, &mut h, 2u32);
    SystemParametersInfoW(SPI_GETSCREENSAVETIMEOUT, 0, &mut t, 0);
}
// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//     Status = CmpCmdHiveOpen(SourceFile, &Ctx, &Allocate, &CmHive,
//                             CM_CHECK_REGISTRY_PURGE_VOLATILES);
//     if (!SystemParametersInfoW(SPI_GETSCREENSAVETIMEOUT, 0, &Timeout, 0))
//         SystemParametersInfoW(SPI_SETSCREENSAVETIMEOUT, 600, 0, SPIF_UPDATEINIFILE);
"""

CONTROLS: list[Control] = [
    Control(
        name="scan_byteptr_stride",
        script="tools/scan_byteptr_stride.py",
        argv=["--lint"],
        bad={"recinfo.rs": _BYTEPTR_BAD},
        good={"recinfo.rs": _BYTEPTR_GOOD},
    ),
    Control(
        name="scan_gmem_flags",
        script="tools/scan_gmem_flags.py",
        argv=["--lint"],
        bad={"olepicture.rs": _GMEM_BAD},
        good={"olepicture.rs": _GMEM_GOOD},
    ),
    Control(
        name="scan_raw_bitfield",
        script="tools/scan_raw_bitfield.py",
        argv=["--lint"],
        bad={"satl.rs": _RAWBF_BAD},
        good={"satl.rs": _RAWBF_GOOD},
    ),
    Control(
        name="scan_let_shadow",
        script="tools/scan_let_shadow.py",
        argv=["--lint"],
        bad={"hwdisk.rs": _SHADOW_BAD},
        good={"hwdisk.rs": _SHADOW_GOOD},
    ),
    Control(
        name="scan_winerror_consts",
        script="tools/scan_winerror_consts.py",
        argv=["--lint"],
        bad={"olesys.rs": _WINERR_BAD},
        good={"olesys.rs": _WINERR_GOOD},
    ),
    Control(
        name="scan_case_bitor_pattern",
        script="tools/scan_case_bitor_pattern.py",
        argv=["--lint"],
        bad={"treeview.c": _CASE_BITOR_C, "treeview.rs": _CASE_BITOR_BAD},
        good={"treeview.c": _CASE_BITOR_C, "treeview.rs": _CASE_BITOR_GOOD},
    ),
    Control(
        name="scan_list_walk_from_head",
        script="tools/scan_list_walk_from_head.py",
        argv=["--lint"],
        bad={"irotp.rs": _LISTWALK_BAD},
        good={"irotp.rs": _LISTWALK_GOOD},
    ),
    Control(
        name="scan_dowhile_continue",
        script="tools/scan_dowhile_continue.py",
        argv=["--lint"],
        bad={"install.rs": _DOWHILE_BAD},
        good={"install.rs": _DOWHILE_GOOD},
    ),
    Control(
        name="scan_dropped_asserts",
        script="tools/scan_dropped_asserts.py",
        argv=["--root"],
        bad={"miarm.h": _DROPPED_ASSERT_H, "bridge.rs": _DROPPED_ASSERT_BAD},
        good={"miarm.h": _DROPPED_ASSERT_H, "bridge.rs": _DROPPED_ASSERT_GOOD},
    ),
    Control(
        name="scan_assert_polarity",
        script="tools/scan_assert_polarity.py",
        argv=["--root"],
        bad={"src/init.rs": _ASSERT_POLARITY_BAD},
        good={"src/init.rs": _ASSERT_POLARITY_GOOD},
    ),
    Control(
        name="scan_rawdylib_linkname",
        script="tools/scan_rawdylib_linkname.py",
        argv=["--root"],
        bad={"src/glue.rs": _RAWDYLIB_BAD},
        good={"src/glue.rs": _RAWDYLIB_GOOD},
    ),
    Control(
        name="vendored_registry",
        script="tools/vendored.py",
        argv=["--repo"],
        bad={
            "vendor.yaml": _VENDOR_BAD_YAML,
            "libs/known/VENDORED": _VENDOR_STAMP,
            "libs/known/a.c": "int a;\n",
            "libs/surprise/b.c": "int b;\n",
        },
        good={
            "vendor.yaml": _VENDOR_GOOD_YAML,
            "libs/known/VENDORED": _VENDOR_STAMP,
            "libs/known/a.c": "int a;\n",
            "libs/surprise/VENDORED": _VENDOR_STAMP_2,
            "libs/surprise/b.c": "int b;\n",
        },
    ),
    Control(
        name="scan_dropped_prints",
        script="tools/scan_dropped_prints.py",
        argv=["--lint"],
        bad={"cmsysini.rs": _DROPPEDPRINT_BAD},
        good={"cmsysini.rs": _DROPPEDPRINT_GOOD},
        pass_files=True,
    ),
    Control(
        name="scan_dropped_seh_pairs",
        script="tools/scan_dropped_seh_pairs.py",
        argv=["--lint"],
        bad={
            "ntos.cmake": _SEHPAIR_CMAKE,
            "ntoskrnl/lpc/reply.c": _SEHPAIR_C,
            "sdk/lib/rust/lpc-rust/src/reply.rs": _SEHPAIR_BAD_RS,
        },
        good={
            "ntos.cmake": _SEHPAIR_CMAKE,
            "ntoskrnl/lpc/reply.c": _SEHPAIR_C,
            "sdk/lib/rust/lpc-rust/src/reply.rs": _SEHPAIR_GOOD_RS,
        },
    ),
    Control(
        name="scan_raising_probes",
        script="tools/scan_raising_probes.py",
        argv=["--lint"],
        bad={"sdk/lib/rust/ex-rust/src/uuid.rs": _RAISINGPROBE_BAD},
        good={"sdk/lib/rust/ex-rust/src/uuid.rs": _RAISINGPROBE_GOOD},
    ),
    Control(
        name="scan_raising_probes",
        script="tools/scan_raising_probes.py",
        argv=["--lint"],
        bad={"sdk/lib/rust/win32k-rust/src/mem.rs": _RAISINGMDL_BAD},
        good={"sdk/lib/rust/win32k-rust/src/mem.rs": _RAISINGMDL_GOOD},
    ),
    Control(
        name="scan_raw_user_access",
        script="tools/scan_raw_user_access.py",
        argv=[],
        bad={"sdk/lib/rust/ex-rust/src/keyedevt.rs": _RAWUSER_BAD},
        good={"sdk/lib/rust/ex-rust/src/keyedevt.rs": _RAWUSER_GOOD},
    ),
    Control(
        name="scan_double_fetch",
        script="tools/scan_double_fetch.py",
        argv=["--lint"],
        bad={"sdk/lib/rust/ex-rust/src/dblfetch.rs": _DBLFETCH_BAD},
        good={"sdk/lib/rust/ex-rust/src/dblfetch.rs": _DBLFETCH_GOOD},
    ),
    Control(
        name="scan_safe_probe_deref",
        script="tools/scan_safe_probe_deref.py",
        argv=["--lint"],
        bad={"sdk/lib/rust/ks-rust/src/property.rs": _SAFEPROBE_BAD},
        good={"sdk/lib/rust/ks-rust/src/property.rs": _SAFEPROBE_GOOD},
    ),
    Control(
        name="strip_msvc_cmake",
        script="tools/strip_msvc_cmake.py",
        argv=["--check"],
        bad={"CMakeLists.txt": "if(MSVC OR X)\n    a()\nendif()\n"},
        good={"CMakeLists.txt": 'if(CMAKE_C_COMPILER_ID STREQUAL "Clang")\n    a()\nendif()\n'},
        pass_files=True,
        file_suffix="CMakeLists.txt",
    ),
    Control(
        name="scan_dropped_probes",
        script="tools/scan_dropped_probes.py",
        argv=["--lint"],
        bad={"ntapi.rs": _DROPPEDPROBE_BAD},
        good={"ntapi.rs": _DROPPEDPROBE_GOOD},
        pass_files=True,
    ),
    Control(
        name="scan_unpruned_walk",
        script="tools/scan_unpruned_walk.py",
        argv=["--lint"],
        bad={"walker.py": _UNPRUNED_BAD},
        good={"walker.py": _UNPRUNED_GOOD},
    ),
    Control(
        name="scan_macro_fn_shadow",
        script="tools/scan_macro_fn_shadow.py",
        argv=["--lint"],
        bad={"recycle_bin.rs": _MACROSHADOW_BAD},
        good={"recycle_bin.rs": _MACROSHADOW_GOOD},
        pass_files=True,
    ),
    Control(
        name="scan_macro_fn_shadow",
        script="tools/scan_macro_fn_shadow.py",
        argv=["--lint"],
        bad={"close.rs": _MACROSHADOW_DDK_BAD},
        good={"close.rs": _MACROSHADOW_DDK_GOOD},
        pass_files=True,
    ),
    Control(
        name="scan_mask_drift",
        script="tools/scan_mask_drift.py",
        argv=["--lint"],
        bad={"ntapi.rs": _MASKDRIFT_BAD},
        good={"ntapi.rs": _MASKDRIFT_GOOD},
        pass_files=True,
    ),
    Control(
        name="scan_type_alias_contradictions",
        script="tools/scan_type_alias_contradictions.py",
        argv=["--lint"],
        bad={"types_i386.rs": _ALIAS_BAD_A, "dirsup.rs": _ALIAS_BAD_B},
        good={"types_i386.rs": _ALIAS_GOOD_A, "dirsup.rs": _ALIAS_GOOD_B},
    ),
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
        name="scan_ptr_byte_diff",
        script="tools/scan_ptr_byte_diff.py",
        argv=["--lint"],
        bad={"version-rust/src/lib.rs": _PTR_BYTE_DIFF_BAD},
        good={"version-rust/src/lib.rs": _PTR_BYTE_DIFF_GOOD},
    ),
    Control(
        name="scan_misaligned_cast",
        script="tools/scan_misaligned_cast.py",
        argv=["--lint"],
        bad={"sha1.rs": _MISALIGNED_CAST_BAD},
        good={"sha1.rs": _MISALIGNED_CAST_GOOD},
    ),
    Control(
        name="scan_dropped_null_guard",
        script="tools/scan_dropped_null_guard.py",
        argv=["--lint"],
        bad={"compress/src/lib.rs": _NULL_GUARD_BAD},
        good={"compress/src/lib.rs": _NULL_GUARD_GOOD},
    ),
    Control(
        name="scan_constant_placement",
        script="tools/scan_constant_placement.py",
        argv=["--lint"],
        bad={"usbuhci/src/usbuhci.rs": _PLACEMENT_BAD},
        good={"usbuhci/src/usbuhci.rs": _PLACEMENT_GOOD},
    ),
    Control(
        name="scan_store_on_failure",
        script="tools/scan_store_on_failure.py",
        argv=["--lint"],
        bad={"rust/compress/src/lib.rs": _STORE_ON_FAILURE_BAD},
        good={"rust/compress/src/lib.rs": _STORE_ON_FAILURE_GOOD},
    ),
    Control(
        name="scan_bool_width",
        script="tools/scan_bool_width.py",
        argv=["--lint"],
        bad=_BOOL_WIDTH_BAD,
        good=_BOOL_WIDTH_GOOD,
    ),
    Control(
        name="scan_duplicate_symbols",
        script="tools/scan_duplicate_symbols.py",
        argv=["--lint"],
        bad=_DUP_BAD,
        good=_DUP_GOOD,
    ),
    Control(
        name="scan_export_shadows",
        script="tools/scan_export_shadows.py",
        argv=["--lint"],
        bad=_EXPORT_SHADOW_BAD,
        good=_EXPORT_SHADOW_GOOD,
    ),
    Control(
        name="scan_cfg_vs_c",
        script="tools/scan_cfg_vs_c.py",
        argv=["--lint"],
        bad={
            "rust/ntoskrnl-rust/src/ke/bug.rs": _SMP_CFG_BAD,
            "rust/user32-rust/src/windows/class.rs": _WIN64_CFG_BAD,
            "rust/rtlk-rust/src/misc.rs": _DEAD_CFG_BAD,
        },
        good={
            "rust/ntoskrnl-rust/src/ke/bug.rs": _SMP_CFG_GOOD,
            "rust/user32-rust/src/windows/class.rs": _WIN64_CFG_GOOD,
            "rust/rtlk-rust/src/misc.rs": _DEAD_CFG_GOOD,
        },
    ),
    Control(
        name="scan_bitwise_not_bool",
        script="tools/scan_bitwise_not_bool.py",
        argv=["--lint"],
        bad={"bad.rs": _BITWISE_NOT_BAD},
        good={"good.rs": _BITWISE_NOT_GOOD},
    ),
    Control(
        name="scan_info_class_literals",
        script="tools/scan_info_class_literals.py",
        argv=["--lint"],
        bad={"bad.rs": _INFO_CLASS_BAD},
        good={"good.rs": _INFO_CLASS_GOOD},
    ),
    Control(
        name="scan_const_narrowing_cast",
        script="tools/scan_const_narrowing_cast.py",
        argv=["--lint"],
        bad={"bad.rs": _CONST_NARROWING_BAD},
        good={"good.rs": _CONST_NARROWING_GOOD},
    ),
    Control(
        name="scan_narrow_vs_wide_literal",
        script="tools/scan_narrow_vs_wide_literal.py",
        argv=["--lint"],
        bad={"bad.rs": _NARROW_WIDE_BAD},
        good={"good.rs": _NARROW_WIDE_GOOD},
    ),
    Control(
        name="scan_call_arg_swap",
        script="tools/scan_call_arg_swap.py",
        argv=["--lint"],
        bad={"bad.rs": _CALL_ARG_SWAP_BAD},
        good={"good.rs": _CALL_ARG_SWAP_GOOD},
    ),
    Control(
        name="scan_sse_helpers",
        script="tools/scan_sse_helpers.py",
        argv=["--lint"],
        bad={"bad.rs": _SSE_HELPERS_BAD},
        good={"good.rs": _SSE_HELPERS_GOOD},
    ),
    Control(
        name="scan_read_after_clear",
        script="tools/scan_read_after_clear.py",
        argv=["--lint"],
        bad={"bad.rs": _READ_AFTER_CLEAR_BAD},
        good={"good.rs": _READ_AFTER_CLEAR_GOOD},
    ),
    Control(
        name="scan_init_then_move",
        script="tools/scan_init_then_move.py",
        argv=["--lint"],
        bad={"bad.rs": _INIT_THEN_MOVE_BAD},
        good={"good.rs": _INIT_THEN_MOVE_GOOD},
    ),
    Control(
        name="scan_containing_record_null",
        script="tools/scan_containing_record_null.py",
        argv=["--lint"],
        bad={"bad.rs": _CONTAINING_RECORD_NULL_BAD},
        good={"good.rs": _CONTAINING_RECORD_NULL_GOOD},
    ),
    Control(
        name="scan_comment_swallowed_code",
        script="tools/scan_comment_swallowed_code.py",
        argv=["--lint"],
        bad={"bad.rs": _COMMENT_SWALLOWED_BAD},
        good={"good.rs": _COMMENT_SWALLOWED_GOOD},
    ),
    Control(
        name="scan_call_literal_vs_c",
        script="tools/scan_call_literal_vs_c.py",
        argv=["--lint"],
        bad={"bad.rs": _CALL_LITERAL_BAD},
        good={"good.rs": _CALL_LITERAL_GOOD},
    ),
    Control(
        name="scan_formatless_printf",
        script="tools/scan_formatless_printf.py",
        argv=["--lint"],
        bad={"bad.rs": _FORMATLESS_PRINTF_BAD},
        good={"good.rs": _FORMATLESS_PRINTF_GOOD},
    ),
    Control(
        name="scan_byte_step_one",
        script="tools/scan_byte_step_one.py",
        argv=["--lint"],
        bad={"bad.rs": _BYTE_STEP_ONE_BAD},
        good={"good.rs": _BYTE_STEP_ONE_GOOD},
    ),
    Control(
        name="scan_ptr_typedef_cast_read",
        script="tools/scan_ptr_typedef_cast_read.py",
        argv=["--lint"],
        bad={"bad.rs": _PTR_TYPEDEF_CAST_READ_BAD},
        good={"good.rs": _PTR_TYPEDEF_CAST_READ_GOOD},
    ),
    Control(
        name="scan_variadic_array_arg",
        script="tools/scan_variadic_array_arg.py",
        argv=["--lint"],
        bad={"bad.rs": _VARIADIC_ARRAY_BAD},
        good={"good.rs": _VARIADIC_ARRAY_GOOD},
    ),
    Control(
        name="scan_variadic_array_arg",
        script="tools/scan_variadic_array_arg.py",
        argv=["--lint"],
        bad={"bad.rs": _VARIADIC_ARRAY_BARE_BAD},
        good={"good.rs": _VARIADIC_ARRAY_BARE_GOOD},
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
        name="scan_opaque_ptr_arith",
        script="tools/scan_opaque_ptr_arith.py",
        argv=["--lint"],
        bad={"x-sys/src/opaque_fixture_bindgen.rs": _OPAQUE_BINDGEN, "src/bad.rs": _OPAQUE_BAD},
        good={"x-sys/src/opaque_fixture_bindgen.rs": _OPAQUE_BINDGEN, "src/good.rs": _OPAQUE_GOOD},
    ),
    Control(
        name="scan_fnptr_abi_transmute",
        script="tools/scan_fnptr_abi_transmute.py",
        argv=["--lint"],
        bad={"bad.rs": _FNABI_BAD, "bad_typed.rs": _FNABI_BAD_TYPED},
        good={"good.rs": _FNABI_GOOD},
    ),
    Control(
        name="scan_cdecl_alias",
        script="tools/scan_cdecl_alias.py",
        argv=["--lint"],
        bad={"bridge.c": _CDECL_ALIAS_BAD},
        good={"bridge.c": _CDECL_ALIAS_GOOD},
    ),
    Control(
        name="scan_unaligned_wide",
        script="tools/scan_unaligned_wide.py",
        argv=["--lint"],
        bad={"bad.rs": _UNALIGNED_WIDE_BAD},
        good={"good.rs": _UNALIGNED_WIDE_GOOD},
    ),
    Control(
        name="scan_wide_byte_copy",
        script="tools/scan_wide_byte_copy.py",
        argv=["--lint"],
        bad={"bad.rs": _WIDEBYTE_BAD},
        good={"good.rs": _WIDEBYTE_GOOD},
    ),
    Control(
        name="scan_typed_byte_copy",
        script="tools/scan_typed_byte_copy.py",
        argv=["--lint"],
        bad={"bad.rs": _TYPEDBYTE_BAD},
        good={"good.rs": _TYPEDBYTE_GOOD},
    ),
    Control(
        name="scan_noop_goto",
        script="tools/scan_noop_goto.py",
        argv=["--lint"],
        bad={"bad.rs": _NOOPGOTO_BAD},
        good={"good.rs": _NOOPGOTO_GOOD},
    ),
    Control(
        name="scan_irql_balance",
        script="tools/scan_irql_balance.py",
        argv=["--lint"],
        summary_workspace=True,
        bad={"bad.rs": _IRQL_BAD},
        good={"good.rs": _IRQL_GOOD},
    ),
    Control(
        name="scan_extern_decl_drift",
        script="tools/scan_extern_decl_drift.py",
        argv=["--lint"],
        bad={"src/a.rs": _DECL_DRIFT_BAD_A, "src/b.rs": _DECL_DRIFT_BAD_B},
        good={"src/a.rs": _DECL_DRIFT_GOOD_A, "src/b.rs": _DECL_DRIFT_GOOD_B},
    ),
    Control(
        name="scan_com_double_release",
        script="tools/scan_com_double_release.py",
        argv=["--lint"],
        bad={"src/bad.rs": _COMREL_BAD},
        good={"src/good.rs": _COMREL_GOOD},
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
    "scan_pack_drift": (
        "it maps every flipped module to its crate through the tree's own CMakeLists flip "
        "comments, so a fixture directory has no module to resolve. Driven directly in "
        "tools/test_scan_pack_drift.py against a temporary repo: it must flag diskpart's "
        "unpacked PARTITION_SECTOR and must not flag a packed one or a struct outside a "
        "pshpack region."
    ),
    "scan_ptr_typedef_indirection": (
        "its pairing is body_audit's -- every module whose CMakeLists links a Rust crate, its "
        "flipped-out TUs (from the build's compiled-TU set) against its crates' sources -- so a "
        "fixture directory has no module to resolve. The decision is findings_for(c_body, "
        "rust), driven directly in tools/test_scan_ptr_typedef_indirection.py: it must flag "
        "usetup's `*mut PFILEPATHS_W` FileCopyCallback shape at its line, and must not flag a "
        "genuine `PFOO *x` or a nolint line."
    ),
    "scan_hresult_literals": (
        "its oracle is the C module scan_invented_constants.c_dirs_for() finds for a crate in "
        "the real tree, so a fixture directory has no crate to map. The decision is "
        "scan_files(rs_files, c_dirs, names), driven directly in "
        "tools/test_scan_hresult_literals.py with an explicit C dir: it must flag ole32's "
        "STG_E_TOOMANYOPENFILES-for-READFAULT shape and must not flag a value the C function "
        "returns, a named const, a nolint line, an extern declaration, or an NTSTATUS function."
    ),
    # Already have dedicated suites (the prototypes this harness generalizes).
    "scan_wide_literals": "covered by tools/test_scan_wide_literals.py (20 tests)",
    "verify_c_signatures": "covered by tools/test_verify_c_signatures.py (9 tests)",
    "marshalgen_audit": "covered by tools/wow64gen/test_wow64gen.py",
    "verify_svc_numbering": "covered by tools/wow64gen/test_wow64gen.py",
    # Both compare a translated Rust module against the C it came FROM, so neither walks a
    # tree the fixture-dir driver above can substitute — they need a .c/.rs PAIR (or, for
    # the ctype tables, ctype.c plus the compiled object as the oracle). Same shape as the
    # --cmake/--ramdisk case below. Dedicated suites instead, each with a real positive
    # control taken from the defect that motivated the tool:
    "verify_translation_literals": (
        "covered by tools/test_verify_translation_literals.py (12 tests) — positive control "
        'is undname.c\'s `case \'C\': "pascal"` shipping as "__stdcall", plus negatives for '
        "each way a faithful translation legitimately SPELLS a literal differently"
    ),
    "verify_ucrtbase_cohab": (
        "covered by tools/test_verify_ucrtbase_cohab.py (8 tests) — positive control is the "
        "one-line accident the crate split exists to prevent, `crt-shared-cargo` depending on "
        "`msvcrt-crt-rust`, plus the transitive form; negatives are the correct wiring and a "
        "`[features]` entry that NAMES a crate without depending on it. Not a fixture-dir "
        "control because the lint reads a dependency GRAPH and an nm'd archive, neither of "
        "which the tree-walking driver above can express"
    ),
    "gen_ctype_tables": (
        "covered by tools/test_gen_ctype_tables.py (9 tests) — positive control is prove() "
        "exiting on ONE wrong byte in 257, which is the claim the whole generator rests on"
    ),
    # Audits a PAIR of specific named files (--cmake / --ramdisk), not a tree, so the
    # fixture-dir driver above cannot express it. Controls are the same shape and live
    # in tests/qemu/cli/test_cli_units.py::test_boot_media_drift_* (5 tests): positive
    # = the real #71 bug, a hive on the livecd and absent from the ramdisk; negatives =
    # both media agreeing, and the BCD hive staged to efi/boot which it must NOT demand.
    # It also self-checks for the dead-matcher failure this file exists to prevent: an
    # empty reference set is NO_REFERENCE and a hard failure, never a silent green.
    "scan_boot_media_drift": "covered by tests/qemu/cli/test_cli_units.py (5 tests)",
    # Diffs the WHOLE tree against a checked-in snapshot (tools/cohab-census.txt); it
    # takes no path argument and has no notion of a root, so the fixture-dir driver
    # above cannot express it. The control it needs is the same one in spirit, and it
    # lives in its own suite: remove the single `#` from ntos.cmake's
    # `#${REACTOS_SOURCE_DIR}/ntoskrnl/lpc/reply.c` -- the exact edit a bad merge
    # resolution makes -- and assert the scanner reports CENSUS-REMOVED and exits 1,
    # then restores the file and asserts clean. That is the 2026-09-19 partial-flip
    # shape, which every boot test passed while the guest was dead.
    "cohab_census": (
        "covered by tools/test_cohab_census.py (3 tests) -- positive control is "
        "uncommenting a real flipped source line in ntoskrnl/ntos.cmake"
    ),
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
    "scan_import_cc": (
        "main() resolves its oracle from the WHOLE repo — every .spec plus every "
        "add_importlibs() — so a fixture directory has nothing to be judged against. "
        "scan_crate()/read_spec()/crate_link_set()/c_defines() all take their inputs as "
        "arguments and are driven with synthetic specs in tools/test_scan_import_cc.py "
        "(15 tests), which pins each of the six blind spots its first whole-tree run had."
    ),
    "scan_raw_uart_writes": (
        "main() walks the real tree, but the decision is scan_text(), a pure function "
        "over a string, driven directly in tools/test_scan_raw_uart_writes.py. Its fire "
        "cases are the three shapes the 87 real stores took when it landed; its must-not "
        "cases pin the nearest-binding rule that keeps an unrelated `u` from being "
        "reported, plus comments, strings and comparisons."
    ),
    "scan_ptr_highword_mask": (
        "main() walks the real tree, but the decision is scan_text(), a pure function over a "
        "string, driven directly in tools/test_scan_ptr_highword_mask.py. Its fire cases are "
        "the two shapes that shipped (a declared-usize identifier, an `as usize` cast); its "
        "must-not cases pin the u32 rop, the version DWORD, the call-proc `== 0xFFFF0000` "
        "form, the fixed `& !0xFFFF` form and C quoted in a comment."
    ),
    "scan_octal_literals": (
        "main() walks the real tree, but the matcher and the comment/literal tokenizer are "
        "pure functions over a string — scan_text()/strip_noise() — and are driven directly "
        "in tools/test_scan_octal_literals.py (17 tests). A fixture DIRECTORY would prove "
        "less than those do: every one of this lint's failure modes is a lexical shape "
        "(float exponent, lifetime tick, nested block comment, raw string), and each is "
        "pinned by the shape that actually produced a false positive."
    ),
    "scan_arg_side_effects": (
        "main() walks the real tree and compares against a baseline, but everything that "
        "decides a finding — the comment/string tokenizer, the logical-line joiner, the "
        "enclosing-call walk and the for-clause exclusion — is a pure function over a "
        "string, driven directly in tools/test_scan_arg_side_effects.py (9 tests). Its "
        "positive control is the REAL msvcrt demangler bug spelled across four lines, and "
        "each negative control is a shape that made an earlier draft useless (45 findings "
        "from `for` steps, 53 from prose, 80 from one unbalanced macro)."
    ),
    "scan_const_contradictions": "needs the whole tree (finds one NAME with two VALUES)",
    "scan_const_within_crate": (
        "needs the whole tree's const declarations. The decision, within_crate(), is a pure "
        "function over one dict, driven in tools/test_scan_const_vs_island.py with userenv's "
        "IDS_PROFILEPATH (0 and 1 in one crate) as the positive control."
    ),
    "scan_const_vs_island": (
        "needs the whole tree's bindgen islands as its oracle. The decision, "
        "island_disagreements(), is a pure function over two dicts, driven in "
        "tools/test_scan_const_vs_island.py with the fsrtl swap as the positive control."
    ),
    "scan_const_vs_own_island": (
        "needs the whole tree's per-crate islands as its oracle. The decision, "
        "own_island_shadows(), is a pure function over two dicts, driven in "
        "tools/test_scan_const_vs_island.py with atapi's MAX_SLOTS (16 copied from usbxhci, "
        "its island says 32) as the positive control."
    ),
    "scan_bindgen_padding_index": (
        "a regex over single source lines with no tree model, so a fixture dir would only "
        "restate its unit tests. tools/test_scan_bindgen_padding_index.py drives scan_text "
        "with the user32 wnd_pti read as the positive control and address-of/offset_of "
        "uses (layout-stable) as negatives, and asserts the real tree is clean."
    ),
    "scan_extern_static_width": (
        "pairs the whole tree's C __asm__ definitions with its Rust extern statics, so a "
        "fixture dir would only restate its unit tests. tools/test_scan_extern_static_width.py "
        "drives c_definitions/scan_rust with fastfat's MAX_LFN_CHARACTERS over-read as the "
        "positive control, equal/narrower/unknown widths as negatives, and asserts the real "
        "tree is clean."
    ),
    "scan_res_named_types": (
        "reads COMMITTED binary resource objects (git ls-files '*.res'), not Rust source, so "
        "a source fixture dir has nothing to offer it. The parser is driven directly in "
        "tools/test_scan_res_named_types.py with synthetic COFF objects: a string-typed "
        "MESSAGETABLE fires, the integer type 11 and a genuinely custom string type do not."
    ),
    "scan_invented_constants": (
        "needs the real tree on BOTH sides: the self-labelled Rust const and the C module "
        "that crate translates. A synthetic fixture would only prove the regex fires. Its "
        "risky half -- the C expression evaluator -- is unit-tested directly in "
        "tools/test_scan_invented_constants.py, which pins the two bugs it has actually had "
        "(a name regex matching inside 0x0501, and C's implicit enum numbering after an "
        "explicit initialiser)."
    ),
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
    if ctl.summary_workspace:
        spec = importlib.util.spec_from_file_location(ctl.name, REPO / ctl.script)
        assert spec is not None and spec.loader is not None
        module = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(module)
        out = StringIO()
        with redirect_stdout(out), redirect_stderr(out):
            rc = module.main([*ctl.argv, *targets], summary_root=root)
        return rc, out.getvalue()
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


def test_irql_fixture_summary_resolves_cross_file_wrapper(tmp_path):
    ctl = next(c for c in CONTROLS if c.name == "scan_irql_balance")
    root = _materialize(
        tmp_path,
        {
            "wrapper.rs": "unsafe fn raise_wrapper() -> u8 {\n"
            "let saved = KeRaiseIrqlToDpcLevel();\nreturn saved;\n}\n",
            "caller.rs": "unsafe fn caller(cond: bool) {\nlet saved = raise_wrapper();\n"
            "if cond { return; }\nKeLowerIrql(saved);\n}\n",
        },
    )
    rc, out = _run(ctl, root)
    assert rc == 1 and "IRQL_LEAK_RETURN" in out and "caller.rs" in out
    (root / "caller.rs").write_text(
        "unsafe fn caller() {\nlet saved = raise_wrapper();\nKeLowerIrql(saved);\nreturn;\n}\n"
    )
    rc, out = _run(ctl, root)
    assert rc == 0 and out == ""


def test_irql_cli_default_still_builds_whole_workspace_summaries(tmp_path, monkeypatch):
    spec = importlib.util.spec_from_file_location(
        "irql_default_control", REPO / "tools/scan_irql_balance.py"
    )
    assert spec is not None and spec.loader is not None
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    seen = []

    def files(paths):
        seen.append(paths)
        return iter(())

    monkeypatch.setattr(module, "_iter_files", files)
    monkeypatch.setattr(module, "build_summaries", lambda *a: (set(), set(), {}))
    target = tmp_path / "selected.rs"
    assert module.main(["--lint", str(target)]) == 0
    assert seen == [[module.RUST_WS], [target]]


def test_irql_a_passive_only_wait_does_not_make_its_callers_sleep(tmp_path):
    """KeSynchronizeExecution waits only for a passive-level interrupt, which may be
    synchronized only at PASSIVE_LEVEL; every caller holding a spin lock around it
    (scsiport's) would otherwise be an IRQL_SLEEP_WHILE_HELD."""
    spec = importlib.util.spec_from_file_location(
        "irql_conditional_control", REPO / "tools/scan_irql_balance.py"
    )
    assert spec is not None and spec.loader is not None
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    src = tmp_path / "ke.rs"
    src.write_text(
        "fn KeSynchronizeExecution() { IopSync(); }\n"
        "fn IopSync() { KeWaitForSingleObject(); }\n"
        "fn Caller() { KeSynchronizeExecution(); }\n"
    )
    may_sleep, _, _ = module.build_summaries([src], {})
    assert "IopSync" in may_sleep
    assert "KeSynchronizeExecution" not in may_sleep
    assert "Caller" not in may_sleep


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


def test_cc_mismatch_flags_one_undefined_symbol_declared_two_ways(tmp_path: Path):
    """[DECL_CC_SPLIT]: no in-crate definition, two i386 conventions across the crate.

    The fastfat shape, spelled as it shipped: fileinfo.rs said "system" for the cdecl
    IoGetCurrentIrpStackLocation thunk, on a line that also opens and closes another block,
    while the other files said "C". The one-view-per-crate fold left esp 4 bytes off after
    every fileinfo call (i386 BUGCHECK 0x50 saving a file). Both sides must be reported."""
    cc = _cc_mod()
    files = {
        "src/cachesup.rs": (
            'extern "C" {\n    fn IoGetCurrentIrpStackLocation(irp: *mut u8) -> *mut u8;\n}\n'
        ),
        "src/fileinfo.rs": (
            'extern "system" {\n    fn FatX(a: u32);\n'
            '    } extern "C" { fn KdPrintEx(c: u32, ...); } extern "system" {\n'
            "    fn IoGetCurrentIrpStackLocation(Irp: *mut u8) -> *mut u8;\n}\n"
        ),
    }
    hits = cc.scan_crate(str(_materialize(tmp_path, files) / "src"))
    split = [h for h in hits if "DECL_CC_SPLIT" in h and "IoGetCurrentIrpStackLocation" in h]
    assert any("cachesup.rs" in h for h in split) and any("fileinfo.rs" in h for h in split), hits


def test_cc_mismatch_counts_a_sys_crate_import_as_a_declaration(tmp_path: Path, monkeypatch):
    """[DECL_CC_SPLIT] where one side is not an extern block at all: pnp.rs imported the cdecl
    IoCopyCurrentIrpStackLocationToNext from ntkrnl_sys::ddk, flush.rs declared it "system".
    rustc merged them; FatCommonFlushBuffers' finally returned into its own stack on i386.
    The import must count, and a crate that only imports must stay quiet."""
    cc = _cc_mod()
    lib = _materialize(
        tmp_path,
        {
            "demo-sys/src/bindgen_i386.rs": (
                'extern "C" {\n    pub fn IoCopyCurrentIrpStackLocationToNext(Irp: *mut u8);\n}\n'
            ),
            "fat-rust/src/pnp.rs": (
                "use demo_sys::ddk::{KeX, IoCopyCurrentIrpStackLocationToNext};\n"
            ),
            "fat-rust/src/flush.rs": (
                'extern "system" {\n    fn IoCopyCurrentIrpStackLocationToNext(Irp: *mut u8);\n}\n'
            ),
            "clean-rust/src/pnp.rs": "use demo_sys::ddk::IoCopyCurrentIrpStackLocationToNext;\n",
        },
    )
    monkeypatch.setattr(cc, "BASE", str(lib))
    monkeypatch.setattr(cc, "_SYS_CACHE", {})
    hits = cc.scan_crate(str(lib / "fat-rust/src"))
    split = [h for h in hits if "DECL_CC_SPLIT" in h]
    assert any("pnp.rs" in h for h in split) and any("flush.rs" in h for h in split), hits
    assert not cc.scan_crate(str(lib / "clean-rust/src"))


def test_cc_mismatch_ignores_a_commented_out_declaration(tmp_path: Path):
    """section.rs keeps `// DUPLICATE: fn KeRaiseIrql(..)` inside an extern "system" block
    next to a live extern "C" one. A comment is not a declaration: no split, no mismatch."""
    cc = _cc_mod()
    files = {
        "src/section.rs": (
            'extern "C" {\n    fn KeRaiseIrql(n: u8, o: *mut u8);\n}\n'
            'extern "system" {\n    // DUPLICATE: fn KeRaiseIrql(n: u8, o: *mut u8);\n}\n'
        ),
    }
    hits = cc.scan_crate(str(_materialize(tmp_path, files) / "src"))
    assert not hits, hits


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


# --- scan_bugcheck_stubs: nested checkouts are ANOTHER lane's tree ----------


def test_bugcheck_stubs_skips_a_nested_git_checkout(tmp_path: Path):
    """The scanner globs `**/rust_cohab_stub.c`, so without pruning it reads any
    worktree nested under the repo (agents create them at
    `.claude/worktrees/<lane>`) and reports that lane's stubs, at that lane's
    commit, as findings here.

    Real incident 2026-07-26: a `.claude/worktrees/fex-m4` checkout added 18
    phantom fastfat findings to an unrelated lane's `check`, failing the gate on
    all three arches with duplicates of already-baselined findings. That is the
    READ side of the fleet-isolation law -- a verdict that depends on which
    nested checkouts happen to exist on this machine is not a gate.
    """
    root = _materialize(
        tmp_path,
        {
            "drivers/foo/rust_cohab_stub.c": _STUBS_BAD_C,
            "drivers/foo/foo.h": _STUBS_HEADER,
            "sdk/lib/rust/foo-rust/src/lib.rs": _STUBS_BAD_RS,
        },
    )
    cmd = [sys.executable, str(REPO / "tools/scan_bugcheck_stubs.py"), str(root)]
    before = subprocess.run(cmd, cwd=REPO, capture_output=True, text=True).stdout
    assert before.strip(), "fixture must produce findings, else this proves nothing"

    # Same content again, but inside a nested checkout: must add NOTHING.
    nested = root / ".claude" / "worktrees" / "sibling"
    for rel in (
        "drivers/foo/rust_cohab_stub.c",
        "drivers/foo/foo.h",
        "sdk/lib/rust/foo-rust/src/lib.rs",
    ):
        dst = nested / rel
        dst.parent.mkdir(parents=True, exist_ok=True)
        dst.write_text((root / rel).read_text())
    (nested / ".git").write_text("gitdir: /elsewhere/.git/worktrees/sibling\n")

    after = subprocess.run(cmd, cwd=REPO, capture_output=True, text=True).stdout
    assert after == before, f"nested checkout leaked findings:\n{after}"
    assert ".claude" not in after


# --- scan_extern_decl_drift: a re-export is not an alias of itself -----------


def test_decl_drift_sees_through_a_c_void_re_export(tmp_path: Path):
    """`pub type c_void = core::ffi::c_void;` must not read as the cycle `c_void -> c_void`.

    This one cannot be caught by the negative control, because its failure mode
    is SILENCE: a cycle resolves to "undecidable", the tool skips the parameter,
    and the report is clean for the wrong reason. It cost real coverage — every
    type spelled through PVOID, which in this tree is most of them.

    So assert the finding is actually PRODUCED. The drift below is only visible
    if `*mut c_void` resolves: decl `*mut PVOID` is two levels, def `PVOID` is
    one, and PVOID is `*mut c_void`.
    """
    root = _materialize(
        tmp_path,
        {
            "src/a.rs": (
                "pub type c_void = core::ffi::c_void;\n"
                "pub type PVOID = *mut c_void;\n"
                'extern "system" {\n'
                "    fn ExFreePool(P: *mut PVOID);\n"
                "}\n"
                "pub unsafe fn caller(p: *mut PVOID) { ExFreePool(p); }\n"
            ),
            "src/b.rs": (
                "pub type PVOID = *mut core::ffi::c_void;\n"
                "#[no_mangle]\n"
                'pub unsafe extern "system" fn ExFreePool(_P: PVOID) {}\n'
            ),
        },
    )
    cmd = [sys.executable, str(REPO / "tools/scan_extern_decl_drift.py"), str(root)]
    out = subprocess.run(cmd, cwd=REPO, capture_output=True, text=True).stdout
    assert "ExFreePool" in out, (
        "the c_void re-export made PVOID undecidable — coverage lost silently\n" + out
    )


# ── scan_orphan_modules: the PHANTOM class (a `mod` with no file behind it) ──
# Its own control for the same reason ROOTLESS needed one: the table's positive control is
# satisfied by ANY output line naming the fixture, so folding a phantom into the orphan
# fixture would be "covered" by the already-passing orphan hit and prove nothing.
#
# The class comes from desk (2026-08-05): lib.rs carried `pub mod guid;` against a file that
# never existed, and separately declared 20 TUs when 17 had been translated. Both survived a
# full green `check` AND `check-tooling`, because a crate parked out of `workspace.members`
# is never compiled and no source scanner opens a path that is absent.
_PHANTOM_CARGO = """\
[package]
name = "phantom-fixture"
version = "0.0.0"
edition = "2021"
"""
_PHANTOM_BAD_LIB = """\
pub mod real;
pub mod ghost;
"""
_PHANTOM_GOOD_LIB = """\
pub mod real;
// A `#[path]` selector names its own file and must NOT read as a phantom — this is the
// per-arch island shape (`types_i386.rs`) every translated crate carries.
#[cfg_attr(target_arch = "x86", path = "real.rs")]
pub mod selected;
// A directory module resolves through `<name>/mod.rs`, not `<name>.rs`.
pub mod dir_mod;
"""


def test_phantom_module_is_reported(tmp_path: Path):
    """`mod ghost;` with no ghost.rs cannot compile — and nothing else can see it."""
    _root, out = _orphan_out(
        tmp_path,
        {
            "Cargo.toml": _PHANTOM_CARGO,
            "src/lib.rs": _PHANTOM_BAD_LIB,
            "src/real.rs": "pub fn ok() -> u32 { 1 }\n",
        },
    )
    hits = [ln for ln in out.splitlines() if "[PHANTOM]" in ln and "ghost" in ln]
    assert hits, f"phantom `mod ghost;` was not reported.\n--- output ---\n{out}"


def test_path_attr_and_dir_modules_are_not_phantoms(tmp_path: Path):
    """The must-NOT-fire half. A gate that flags every per-arch `#[path]` island would fire
    on essentially every translated crate in the tree and be turned off within the hour."""
    _root, out = _orphan_out(
        tmp_path,
        {
            "Cargo.toml": _PHANTOM_CARGO,
            "src/lib.rs": _PHANTOM_GOOD_LIB,
            "src/real.rs": "pub fn ok() -> u32 { 1 }\n",
            "src/dir_mod/mod.rs": "pub fn ok() -> u32 { 2 }\n",
        },
    )
    assert "[PHANTOM]" not in out, f"false positive on a legitimate module shape:\n{out}"


def test_bitwise_not_after_a_keyword_is_flagged():
    """`if !x != 0` once read as the macro `if!` and was never flagged (usetup, powercfg)."""
    from tools.scan_bitwise_not_bool import scan_line

    for line in _BITWISE_NOT_KEYWORD_BAD.splitlines():
        if "!success" in line:
            assert scan_line(line)
    assert not scan_line("    if success == 0 {")
    assert not scan_line("    assert!(x != 0);")


def test_type_alias_tchar_family_is_compared_within_a_crate_only(tmp_path):
    """BCHAR is WCHAR in a -DUNICODE module's island and BYTE in an ANSI one's (the
    appcompat shims): two right answers to two compiles, not a contradiction. Inside ONE
    crate the two widths are still a bug (aclui's TCHAR once came from the ANSI glob)."""
    spec = importlib.util.spec_from_file_location(
        "alias_tchar_control", REPO / "tools/scan_type_alias_contradictions.py"
    )
    assert spec is not None and spec.loader is not None
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    rust = tmp_path / "sdk" / "lib" / "rust"
    for crate, rhs in (("wide-rust", "WCHAR"), ("ansi-rust", "BYTE")):
        (rust / crate / "src").mkdir(parents=True)
        (rust / crate / "src" / "types.rs").write_text(
            f"pub type WCHAR = u16;\npub type BYTE = u8;\npub type BCHAR = {rhs};\n"
        )
    decls, aliases = module.collect(tmp_path)
    assert not [h for h in module.contradictions(decls, aliases) if h[0] == "BCHAR"]
    (rust / "wide-rust" / "src" / "prelude.rs").write_text("pub type BCHAR = u8;\n")
    decls, aliases = module.collect(tmp_path)
    hits = [h for h in module.contradictions(decls, aliases) if h[0] == "BCHAR"]
    assert hits and all("wide-rust" in h[1] for h in hits)


def test_inline_flag_literal_is_paired_within_its_own_field(tmp_path):
    """c_data_port's keyboard tables: `VSC_VK { Vsc: 0x10 as BYTE, Vk: 433 as USHORT
    /* VK_MEDIA_PREV_TRACK | KBDEXT */ }`. The comment annotates 433 alone (0xB1 | 0x100);
    pairing the scan code 0x10 with the VK, and an |-list whose KBDEXT the catalog does not
    know, were 4108 false findings."""
    spec = importlib.util.spec_from_file_location(
        "inline_flag_field_control", REPO / "tools/scan_inline_flag_literals.py"
    )
    assert spec is not None and spec.loader is not None
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    rs = tmp_path / "t.rs"
    rs.write_text(
        "static T: [VSC_VK; 2] = [\n"
        "    VSC_VK { Vsc: 0x10 as BYTE, Vk: 433 as USHORT /* VK_MEDIA_PREV_TRACK | KBDEXT */ },\n"
        "    VSC_VK { Vsc: 0x47 as BYTE, Vk: 3108 as USHORT /* VK_HOME | KNUMS */ },\n"
        "];\n"
        "const BAD: u32 = 0x25; // VK_HOME\n"
    )
    catalog = {"VK_MEDIA_PREV_TRACK": 0xB1, "VK_HOME": 0x24}
    hits = module.scan_file(rs, catalog)
    assert [(h[1], h[2]) for h in hits] == [(0x25, "VK_HOME")]


def test_formatless_printf_name_needs_a_real_printf(tmp_path):
    """apphelp's `SeiPrintFunctionName(lpProcName, szOrdProcFmt)` ends in a `...Fmt`
    parameter and its name contains Print+F(unctionName); it is not a printf, and the old
    case-insensitive `printf` match flagged it. A real one must still flag."""
    spec = importlib.util.spec_from_file_location(
        "formatless_name_control", REPO / "tools/scan_formatless_printf.py"
    )
    assert spec is not None and spec.loader is not None
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    rs = tmp_path / "x.rs"
    rs.write_text(
        "#[no_mangle]\n"
        'pub unsafe extern "C" fn SeiPrintFunctionName(p: LPCSTR, szOrdProcFmt: *mut i8)'
        " -> LPCSTR { p }\n"
        "#[no_mangle]\n"
        'pub unsafe extern "C" fn DbgPrintfEx(Format: PCSTR) -> u32 { 0 }\n'
    )
    assert [f[2] for f in module.scan([rs])] == ["DbgPrintfEx"]
