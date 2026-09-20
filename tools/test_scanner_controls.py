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

_BITWISE_NOT_BAD = """\
pub unsafe fn adjust_fua(flags: u32, special: u32) -> bool {
    // C: DEV_WRITE_CACHE && !DEV_POWER_PROTECTED && !CLASS_SPECIAL_FUA_NOT_SUPPORTED.
    // Both negated guards are BITWISE not -> constant true -> both guards dead.
    (flags & 0x1) != 0 && !(flags & 0x10) != 0 && !(special & 0x80) != 0
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


CONTROLS: list[Control] = [
    Control(
        name="scan_dropped_asserts",
        script="tools/scan_dropped_asserts.py",
        argv=["--root"],
        bad={"miarm.h": _DROPPED_ASSERT_H, "bridge.rs": _DROPPED_ASSERT_BAD},
        good={"miarm.h": _DROPPED_ASSERT_H, "bridge.rs": _DROPPED_ASSERT_GOOD},
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
        name="scan_bitwise_not_bool",
        script="tools/scan_bitwise_not_bool.py",
        argv=["--lint"],
        bad={"bad.rs": _BITWISE_NOT_BAD},
        good={"good.rs": _BITWISE_NOT_GOOD},
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
    "scan_duplicate_symbols": "operates on link artifacts, not source",
    "scan_const_contradictions": "needs the whole tree (finds one NAME with two VALUES)",
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
