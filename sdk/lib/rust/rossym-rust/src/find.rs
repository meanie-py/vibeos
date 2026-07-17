//! [correctness-review 2026-06-07 · sonnet] Reviewed vs sdk/lib/rossym/find.c.
//! Risk: LOW. Behavior: preserved. Functions reviewed: FindEntry, RosSymGetAddressInformation (2/2).
//! Findings:
//!   - Binary search (FindEntry): CORRECT. The while-loop correctly mirrors the BSD for-loop:
//!     `Lim >>= 1` runs unconditionally at the end of each iteration; the "move right" branch
//!     additionally does `Lim -= 1` before it — semantically identical to `Lim--` inside the
//!     for-body followed by the for-increment `Lim >>= 1`. Low/Base/Mid pointer arithmetic is
//!     correct; `.add((Lim >> 1) as usize)` is safe (u32 → usize widening, no truncation).
//!   - RelativeAddress parameter type: `usize` — matches C's `ULONG_PTR`. No truncation risk.
//!   - SourceLine: C reads `RosSymEntry->SourceLine` (ULONG=u32) directly into `*LineNumber`
//!     (ULONG*). Rust casts `(*RosSymEntry).SourceLine as u32` — the field is already u32
//!     so this is a no-op cast, correct.
//!   - String pointer arithmetic: `Strings as *const i8 + FileOffset/FunctionOffset` (usize
//!     offsets) — correct; C does the same via `(PCHAR) RosSymInfo->Strings + offset`.
//!   - Empty-string fallback (`b"\0".as_ptr()`) correctly mirrors C's `Name = ""`.
//!   - BOOLEAN return: C returns BOOLEAN (u8 typedef). Rust returns `u8` (0/1). Matches.
//!   - ABI: `extern "C"` — correct for cdecl rossym API.
//!   - Minor: unused imports (ntkrnl_sys::globals, ntkrnl_sys::constants, PCSTR from ddk)
//!     — benign, no semantic impact.
//! Unverified: ASSERT(LineNumber || FileName || FunctionName) is commented out (matching C NDEBUG
//!   behaviour since #define NDEBUG is set). No functional issue.
use crate::types::*;

extern "C" {
    fn strcpy(dest: *mut i8, src: *const i8) -> *mut i8;
}

// ---- file-scope preamble (translated) ----
pub const NDEBUG: bool = true;

unsafe fn FindEntry(RosSymInfo: PROSSYM_INFO, RelativeAddress: usize) -> *mut ROSSYM_ENTRY {
    let mut Base: *mut ROSSYM_ENTRY;
    let mut Lim: u32;
    let mut Mid: *mut ROSSYM_ENTRY;
    let mut Low: *mut ROSSYM_ENTRY;

    Base = (*RosSymInfo).Symbols;
    if RelativeAddress < (*Base).Address {
        return core::ptr::null_mut();
    }

    Low = Base;

    Lim = (*RosSymInfo).SymbolsCount;
    while Lim != 0 {
        Mid = Base.add((Lim >> 1) as usize);
        if RelativeAddress == (*Mid).Address {
            return Mid;
        }
        if (*Mid).Address < RelativeAddress {
            Low = Mid;
            Base = Mid.add(1);
            Lim -= 1;
        }
        Lim >>= 1;
    }

    Low
}
#[no_mangle]
pub unsafe extern "C" fn RosSymGetAddressInformation(
    RosSymInfo: PROSSYM_INFO,
    RelativeAddress: usize,
    LineNumber: *mut u32,
    FileName: *mut i8,
    FunctionName: *mut i8,
) -> u8 {
    // DPRINT("RelativeAddress = 0x%08x\n", RelativeAddress);

    if (*RosSymInfo).Symbols.is_null()
        || (*RosSymInfo).SymbolsCount == 0
        || (*RosSymInfo).Strings.is_null()
        || (*RosSymInfo).StringsLength == 0
    {
        // DPRINT1("Uninitialized RosSymInfo\n");
        return 0; // FALSE
    }

    // ASSERT(LineNumber || FileName || FunctionName);

    let RosSymEntry: PROSSYM_ENTRY = FindEntry(RosSymInfo, RelativeAddress);

    if RosSymEntry.is_null() {
        // DPRINT("None of the requested information was found!\n");
        return 0;
    }

    if !LineNumber.is_null() {
        *LineNumber = (*RosSymEntry).SourceLine as u32;
    }
    if !FileName.is_null() {
        let Name: *const i8 = if (*RosSymEntry).FileOffset != 0 {
            ((*RosSymInfo).Strings as *const i8).add((*RosSymEntry).FileOffset as usize)
        } else {
            b"\0".as_ptr() as *const i8
        };
        strcpy(FileName, Name);
    }
    if !FunctionName.is_null() {
        let Name: *const i8 = if (*RosSymEntry).FunctionOffset != 0 {
            ((*RosSymInfo).Strings as *const i8).add((*RosSymEntry).FunctionOffset as usize)
        } else {
            b"\0".as_ptr() as *const i8
        };
        strcpy(FunctionName, Name);
    }

    1 // TRUE
}
