use win32k_sys::win32k::*;
use ntkrnl_sys::base::NT_SUCCESS;

// SafeProbeForRead/Write: NTAPI (extern "system"), Length = SIZE_T (canonical — icm.rs).
extern "system" {
    fn SafeProbeForRead(Address: LPVOID, Length: SIZE_T, Alignment: u32) -> NTSTATUS;
    fn SafeProbeForWrite(Address: LPVOID, Length: SIZE_T, Alignment: u32) -> NTSTATUS;
}

// The fault-tolerant kernel copy, imported from ntoskrnl (cdecl; ntoskrnl.spec).
// Returns 1 if the copy completed, 0 if it faulted. amd64/arm64: the native-SEH
// shim in ntoskrnl/kmsafecopy.c. i386: the exception-table fault-fixup primitive
// in ntoskrnl-rust ke/i386/safecopy.rs (field manual §9.122). All three arches
// carry the per-boot `[KMSAFECOPY] … (PASS)` oracle proving it actually catches.
extern "C" {
    fn KmProtectedCopyMemory(Dst: PVOID, Src: PVOID, Len: usize) -> i32;
}

const STATUS_ACCESS_VIOLATION: NTSTATUS = 0xC0000005_u32 as NTSTATUS;

// Callers reach these through the MmCopyFromCaller/MmCopyToCaller macros (= the
// `_`-prefixed names, cdecl). PAGED_CODE()/ASSERT(ExGetPreviousMode()==UserMode)
// are retail no-ops, dropped. extern "C" + #[no_mangle].
//
// WHY THE COPY IS FAULT-TOLERANT AND THE PROBE IS NOT (task #68):
// `SafeProbeFor*` is deliberately validate-ONLY — it range/alignment-checks the
// buffer against MmUserProbeAddress and does not touch the page, because a touch
// fault has no handler under USE_DUMMY_PSEH (see safe_user_io.rs). That leaves a
// hole a plain `RtlCopyMemory` walked straight into: a *partially* mapped user
// buffer passes the range check and then faults mid-copy, so ANY user process,
// 32- or 64-bit, could kill the box by passing one. The probe stays validate-only
// (a touch-test would still be uncaught, and it cannot tell "invalid" from "paged
// out" anyway); the COPY is what becomes fault-tolerant. MmAccessFault still runs
// first, so a valid-but-nonresident page pages in and the copy proceeds — only a
// genuinely bad address turns into STATUS_ACCESS_VIOLATION.
#[no_mangle]
pub unsafe extern "C" fn _MmCopyFromCaller(
    Target: PVOID,
    Source: PVOID,
    Bytes: UINT,
) -> NTSTATUS {
    let Status = SafeProbeForRead(Source as LPVOID, Bytes as SIZE_T, 1);
    if NT_SUCCESS(Status) {
        if Bytes != 0 && KmProtectedCopyMemory(Target, Source, Bytes as usize) == 0 {
            return STATUS_ACCESS_VIOLATION;
        }
    }
    Status
}

#[no_mangle]
pub unsafe extern "C" fn _MmCopyToCaller(
    Target: PVOID,
    Source: PVOID,
    Bytes: UINT,
) -> NTSTATUS {
    let Status = SafeProbeForWrite(Target as LPVOID, Bytes as SIZE_T, 1);
    if NT_SUCCESS(Status) {
        if Bytes != 0 && KmProtectedCopyMemory(Target, Source, Bytes as usize) == 0 {
            return STATUS_ACCESS_VIOLATION;
        }
    }
    Status
}
