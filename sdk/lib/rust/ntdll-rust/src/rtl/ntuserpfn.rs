//! `RtlInitializeNtUserPfn` / `RtlRetrieveNtUserPfn`: user32 registers its client
//! window-procedure and worker tables with ntdll once, and gets back ntdll-resident dispatch
//! stubs that jump through the registered table. Port of upstream `sdk/lib/rtl/ntuserpfn.c` +
//! `ntuserdisp.s` (479662b3f0c), which compile into rtl_vista.
//!
//! Exported by ntdll_vista.spec unconditionally (as upstream) and by ntdll.spec at 0x600+.
//! No caller exists in the tree yet: user32 does not register.
//!
//! The registered table is read-only data that this function unprotects for the one copy, so
//! a stray write elsewhere cannot redirect the stubs. It and the stub-pointer table are
//! defined in the assembly below, because a Rust `static` that is written through is
//! writable data by construction.
//!
//! Where upstream and this port differ: upstream's `ntuserdisp.s` emits the stubs for x86 and
//! amd64 only ("TODO Implement this" for ARM), so `NtDllUserStubs` is undefined on arm64.
//! The arm64 stubs here are the same jump through the table, in four instructions.

use ntdll_sys::nt::*;

/// `RTLP_NUMBER_OF_NTUSER_WNDPROCS` below NTDDI_WIN10 (rtl_vista builds at 0x600).
pub const RTLP_NUMBER_OF_NTUSER_WNDPROCS: usize = 23;
pub const RTLP_NUMBER_OF_NTUSER_WORKERPROCS: usize = 11;
const RTL_NUMBER_OF_NTUSER_PFNS: usize =
    2 * RTLP_NUMBER_OF_NTUSER_WNDPROCS + RTLP_NUMBER_OF_NTUSER_WORKERPROCS;

#[repr(C)]
pub struct NTUSER_PFN {
    pub WndProcsA: [PVOID; RTLP_NUMBER_OF_NTUSER_WNDPROCS],
    pub WndProcsW: [PVOID; RTLP_NUMBER_OF_NTUSER_WNDPROCS],
    pub WorkerProcs: [PVOID; RTLP_NUMBER_OF_NTUSER_WORKERPROCS],
}

const _: () = assert!(
    core::mem::size_of::<NTUSER_PFN>() == RTL_NUMBER_OF_NTUSER_PFNS * core::mem::size_of::<PVOID>()
);

extern "C" {
    /// The registered procedures: zero-filled `.rdata`, written once under a temporary
    /// PAGE_READWRITE by `RtlInitializeNtUserPfn`.
    static mut RtlpNtUserPfns: NTUSER_PFN;
    /// Pointers to the dispatch stubs, in the same layout: stub `i` jumps through
    /// `RtlpNtUserPfns` slot `i`.
    static NtDllUserStubs: NTUSER_PFN;
}

static NtUserPfnsInitialized: core::sync::atomic::AtomicBool =
    core::sync::atomic::AtomicBool::new(false);

#[no_mangle]
pub unsafe extern "system" fn RtlInitializeNtUserPfn(
    WndProcsA: *const core::ffi::c_void,
    SizeOfWndProcsA: SIZE_T,
    WndProcsW: *const core::ffi::c_void,
    SizeOfWndProcsW: SIZE_T,
    WorkerProcs: *const core::ffi::c_void,
    SizeOfWorkerProcs: SIZE_T,
) -> NTSTATUS {
    use core::sync::atomic::Ordering;
    let ptr = core::mem::size_of::<PVOID>();

    if SizeOfWndProcsA as usize != RTLP_NUMBER_OF_NTUSER_WNDPROCS * ptr
        || SizeOfWndProcsW as usize != RTLP_NUMBER_OF_NTUSER_WNDPROCS * ptr
        || SizeOfWorkerProcs as usize != RTLP_NUMBER_OF_NTUSER_WORKERPROCS * ptr
    {
        return STATUS_INVALID_PARAMETER;
    }

    if NtUserPfnsInitialized.load(Ordering::Acquire) {
        return STATUS_INVALID_PARAMETER;
    }

    let table = core::ptr::addr_of_mut!(RtlpNtUserPfns);
    let mut BaseAddress: PVOID = table as PVOID;
    let mut RegionSize: SIZE_T = core::mem::size_of::<NTUSER_PFN>() as SIZE_T;
    let mut OldProtect: ULONG = 0;
    let mut Status = NtProtectVirtualMemory(
        NtCurrentProcess(),
        &mut BaseAddress,
        &mut RegionSize,
        PAGE_READWRITE,
        &mut OldProtect,
    );
    if Status < 0 {
        return Status;
    }

    core::ptr::copy_nonoverlapping(
        WndProcsA as *const u8,
        core::ptr::addr_of_mut!((*table).WndProcsA) as *mut u8,
        SizeOfWndProcsA as usize,
    );
    core::ptr::copy_nonoverlapping(
        WndProcsW as *const u8,
        core::ptr::addr_of_mut!((*table).WndProcsW) as *mut u8,
        SizeOfWndProcsW as usize,
    );
    core::ptr::copy_nonoverlapping(
        WorkerProcs as *const u8,
        core::ptr::addr_of_mut!((*table).WorkerProcs) as *mut u8,
        SizeOfWorkerProcs as usize,
    );

    // BaseAddress/RegionSize now hold the page-rounded region the first call reported.
    Status = NtProtectVirtualMemory(
        NtCurrentProcess(),
        &mut BaseAddress,
        &mut RegionSize,
        OldProtect,
        &mut OldProtect,
    );
    if Status < 0 {
        return Status;
    }

    NtUserPfnsInitialized.store(true, Ordering::Release);
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn RtlRetrieveNtUserPfn(
    NtdllWndProcsA: *mut *const core::ffi::c_void,
    NtdllWndProcsW: *mut *const core::ffi::c_void,
    NtdllWorkerProcs: *mut *const core::ffi::c_void,
) -> NTSTATUS {
    if !NtUserPfnsInitialized.load(core::sync::atomic::Ordering::Acquire) {
        return STATUS_INVALID_PARAMETER;
    }

    let stubs = core::ptr::addr_of!(NtDllUserStubs);
    *NtdllWndProcsA = core::ptr::addr_of!((*stubs).WndProcsA) as *const _;
    *NtdllWndProcsW = core::ptr::addr_of!((*stubs).WndProcsW) as *const _;
    *NtdllWorkerProcs = core::ptr::addr_of!((*stubs).WorkerProcs) as *const _;
    STATUS_SUCCESS
}

// The dispatch stubs, one per slot, each 16-byte aligned, then the table of their addresses.
// RTL_NUMBER_OF_NTUSER_PFNS = 57 is spelled out in the .irp lists (checked above).

#[cfg(target_arch = "x86")]
core::arch::global_asm!(
    ".section .rdata,\"dr\"",
    ".p2align 2",
    ".globl _RtlpNtUserPfns",
    "_RtlpNtUserPfns:",
    ".fill 57, 4, 0",
    ".text",
    ".irp i, 0,1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25,26,27,28,29,30,31,32,33,34,35,36,37,38,39,40,41,42,43,44,45,46,47,48,49,50,51,52,53,54,55,56",
    ".p2align 4",
    "RtlpNtUserPfnDispatch\\i:",
    "jmp dword ptr [_RtlpNtUserPfns + \\i * 4]",
    ".endr",
    ".section .rdata,\"dr\"",
    ".p2align 2",
    ".globl _NtDllUserStubs",
    "_NtDllUserStubs:",
    ".irp i, 0,1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25,26,27,28,29,30,31,32,33,34,35,36,37,38,39,40,41,42,43,44,45,46,47,48,49,50,51,52,53,54,55,56",
    ".long RtlpNtUserPfnDispatch\\i",
    ".endr",
    ".text",
);

#[cfg(target_arch = "x86_64")]
core::arch::global_asm!(
    ".section .rdata,\"dr\"",
    ".p2align 3",
    ".globl RtlpNtUserPfns",
    "RtlpNtUserPfns:",
    ".fill 57, 8, 0",
    ".text",
    ".irp i, 0,1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25,26,27,28,29,30,31,32,33,34,35,36,37,38,39,40,41,42,43,44,45,46,47,48,49,50,51,52,53,54,55,56",
    ".p2align 4",
    "RtlpNtUserPfnDispatch\\i:",
    "jmp qword ptr [rip + RtlpNtUserPfns + \\i * 8]",
    ".endr",
    ".section .rdata,\"dr\"",
    ".p2align 3",
    ".globl NtDllUserStubs",
    "NtDllUserStubs:",
    ".irp i, 0,1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25,26,27,28,29,30,31,32,33,34,35,36,37,38,39,40,41,42,43,44,45,46,47,48,49,50,51,52,53,54,55,56",
    ".quad RtlpNtUserPfnDispatch\\i",
    ".endr",
    ".text",
);

#[cfg(target_arch = "aarch64")]
core::arch::global_asm!(
    ".section .rdata,\"dr\"",
    ".p2align 3",
    ".globl RtlpNtUserPfns",
    "RtlpNtUserPfns:",
    ".fill 57, 8, 0",
    ".text",
    ".irp i, 0,1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25,26,27,28,29,30,31,32,33,34,35,36,37,38,39,40,41,42,43,44,45,46,47,48,49,50,51,52,53,54,55,56",
    ".p2align 4",
    "RtlpNtUserPfnDispatch\\i:",
    "adrp x16, RtlpNtUserPfns",
    "add x16, x16, :lo12:RtlpNtUserPfns",
    "ldr x16, [x16, #(\\i * 8)]",
    "br x16",
    ".endr",
    ".section .rdata,\"dr\"",
    ".p2align 3",
    ".globl NtDllUserStubs",
    "NtDllUserStubs:",
    ".irp i, 0,1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25,26,27,28,29,30,31,32,33,34,35,36,37,38,39,40,41,42,43,44,45,46,47,48,49,50,51,52,53,54,55,56",
    ".quad RtlpNtUserPfnDispatch\\i",
    ".endr",
    ".text",
);
