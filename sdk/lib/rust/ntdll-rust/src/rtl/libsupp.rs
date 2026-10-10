//! `rtl/libsupp.c` — the rtl↔ntdll glue boundary (RTL Support Routines).
//!
//! User-mode rtl callbacks the shared `librtl` walks for *process* state:
//! PEB/TEB accessors, heap/atom critical-section locks, stack-limit + frame
//! chain capture, PE-header parsing, and SxS/apiset file-name redirection.
//!
//! rustconv hand-fixes (integration notes):
//!   * `RtlGetProcessHeap` is FORCEINLINE and `MAKELANGID`/`PRIMARYLANGID`/
//!     `SUBLANGID`/`LANGIDFROMLCID`/`UInt32x32To64`/`RtlpAllocateStringMemory`
//!     are C macros — rustconv emitted them as `extern`s (won't link). All
//!     reconstructed as `#[inline(always)]` helpers below.
//!   * `RtlAllocateHeap`/`RtlFreeHeap`/`RtlInitUnicodeString`/… come from the
//!     `ntdll_sys::nt::*` glob — removed from the extern block (dup decl).
//!   * `PUNICORE_STRING` typo → `PUNICODE_STRING`.
//!   * ABI per the C qualifiers: NTAPI Rtl*/Ldr* → `extern "system"`; the
//!     un-qualified atom-table callbacks + Wine resource helpers + get_buffer/
//!     find_actctx_dll → `extern "C"` (cdecl). #1 i386 crash cause if wrong.
//!   * SEH (`_SEH2_*`) is dead under DUMMY_PSEH — handlers dropped.

#![allow(static_mut_refs)]

use ntdll_sys::nt::*;
use core::ffi::c_void;

// Resource-directory walkers (Wine-style, cdecl) — defined in ldr resource C.
// Plus the libcntpr CRT string helpers (cdecl) the SxS path uses.
extern "C" {
    fn find_entry_by_name(dir: *mut IMAGE_RESOURCE_DIRECTORY, name: LPCWSTR, root: *mut c_void, want_dir: i32) -> *mut IMAGE_RESOURCE_DIRECTORY;
    fn find_entry_by_id(dir: *mut IMAGE_RESOURCE_DIRECTORY, id: WORD, root: *mut c_void, want_dir: i32) -> *mut IMAGE_RESOURCE_DIRECTORY;
    fn find_first_entry(dir: *mut IMAGE_RESOURCE_DIRECTORY, root: *mut c_void, want_dir: i32) -> *mut IMAGE_RESOURCE_DIRECTORY;
    fn push_language(list: *mut USHORT, pos: ULONG, lang: WORD) -> i32;
    // ApiSetResolveToHost is plain `NTSTATUS ...(...)` in apisets.h (no NTAPI)
    // → cdecl. Declaring it stdcall earlier produced `_ApiSetResolveToHost@16`
    // (undefined); the real symbol is `_ApiSetResolveToHost`.
    fn ApiSetResolveToHost(ApisetVersion: u32, OriginalName: PCUNICODE_STRING, RedirectedDll: PBOOLEAN, ApisetName: PUNICODE_STRING) -> NTSTATUS;
    fn memcpy(dest: *mut u8, src: *const u8, n: usize) -> *mut u8;
    fn wcslen(s: *const u16) -> usize;
    fn wcsrchr(s: *const u16, c: u16) -> *mut u16;
    fn wcscpy(dest: *mut u16, src: *const u16) -> *mut u16;
    fn _wcsnicmp(s1: *const u16, s2: *const u16, n: usize) -> i32;
    fn _wcsicmp(s1: *const u16, s2: *const u16) -> i32;
}

// STATUS / APISET constants not in the ntdll-sys constants surface.
const STATUS_SXS_KEY_NOT_FOUND: NTSTATUS = 0xC0150008u32 as i32;
const STATUS_RESOURCE_DATA_NOT_FOUND: NTSTATUS = 0xC0000089u32 as i32;
const STATUS_RESOURCE_TYPE_NOT_FOUND: NTSTATUS = 0xC000008Au32 as i32;
const STATUS_RESOURCE_NAME_NOT_FOUND: NTSTATUS = 0xC000008Bu32 as i32;
const STATUS_RESOURCE_LANG_NOT_FOUND: NTSTATUS = 0xC0000204u32 as i32;
// apisets.h bit flags: (1 << n).
const APISET_WIN7: DWORD = 1 << 0;
const APISET_WIN8: DWORD = 1 << 1;
const APISET_WIN81: DWORD = 1 << 2;
const APISET_WIN10: DWORD = 1 << 3;
// NTAPI externals (defined in shared rtl). `RtlDoesFileExists_UStr` is
// `BOOLEAN NTAPI` in path.c → stdcall (`_RtlDoesFileExists_UStr@4`); declaring
// it cdecl earlier left it undefined.
extern "system" {
    fn RtlpImageNtHeaderEx(Flags: ULONG, Base: PVOID, Size: ULONG64, OutHeaders: *mut PIMAGE_NT_HEADERS) -> NTSTATUS;
    fn RtlDoesFileExists_UStr(Name: PUNICODE_STRING) -> u8;
}

/// `RosGetProcessCompatVersion(VOID)` — `static inline` in compat_undoc.h, so
/// it has no out-of-line symbol; inlined here. Reads the shim data hung off
/// the PEB (`pShimData`) and returns the recorded process compat version, or 0.
#[inline(always)]
unsafe fn RosGetProcessCompatVersion() -> DWORD {
    const REACTOS_SHIMDATA_MAGIC: DWORD = 0xAC0DEDAB;
    const REACTOS_COMPATVERSION_UNINITIALIZED: DWORD = 0xfffffffe;
    // struct _ReactOS_ShimData: dwReserved1[130], dwSize, dwMagic,
    //   dwReserved2[242], dwRosProcessCompatVersion.
    let p_shim_data = (*NtCurrentPeb()).pShimData as *const DWORD;
    if !p_shim_data.is_null() {
        let dw_size = *p_shim_data.add(130);
        let dw_magic = *p_shim_data.add(131);
        // sizeof(ReactOS_ShimData) = (130 + 1 + 1 + 242 + 1) * 4 = 1500.
        const SHIMDATA_SIZE: DWORD = (130 + 1 + 1 + 242 + 1) * 4;
        if dw_magic == REACTOS_SHIMDATA_MAGIC && dw_size == SHIMDATA_SIZE {
            let compat = *p_shim_data.add(130 + 1 + 1 + 242);
            return if compat < REACTOS_COMPATVERSION_UNINITIALIZED { compat } else { 0 };
        }
    }
    0
}

/// `RtlInitEmptyUnicodeString` — FORCEINLINE in rtlfuncs.h. On amd64/arm64 the workspace firewall
/// (lto=false, cgu=256) can prevent the cross-cgu `#[inline(always)]` from firing, leaving an
/// out-of-line reference to the unmangled symbol. `#[no_mangle]` emits it so the link resolves
/// (i386 inlines fine; the symbol is dead-stripped there).
#[no_mangle]
pub unsafe extern "C" fn RtlInitEmptyUnicodeString(s: *mut UNICODE_STRING, buffer: PWSTR, buffer_size: USHORT) {
    (*s).Length = 0;
    (*s).MaximumLength = buffer_size;
    (*s).Buffer = buffer;
}

/// `rtl_get_process_heap()` (FORCEINLINE macro): `NtCurrentPeb()->ProcessHeap`.
#[inline(always)]
unsafe fn rtl_get_process_heap() -> PVOID {
    (*NtCurrentPeb()).ProcessHeap
}

/// `SharedUserData` macro: `(KUSER_SHARED_DATA*)USER_SHARED_DATA`.
#[inline(always)]
unsafe fn shared_user_data() -> *const _KUSER_SHARED_DATA {
    USER_SHARED_DATA as usize as *const _KUSER_SHARED_DATA
}

/// `RtlpAllocateStringMemory(Bytes, Tag)` macro → `RtlpAllocateMemory`.
#[inline(always)]
unsafe fn RtlpAllocateStringMemory(Bytes: SIZE_T, Tag: ULONG) -> PVOID {
    RtlpAllocateMemory(Bytes as UINT, Tag)
}

/// `NT_SUCCESS(x)` macro: `(NTSTATUS)(x) >= 0`.
#[inline(always)]
fn NT_SUCCESS(status: NTSTATUS) -> bool { status >= 0 }

// LANGID/locale + 64-bit-multiply macros (ntbasedef.h).
#[inline(always)]
fn MAKELANGID(p: u16, s: u16) -> u16 { ((s as u16) << 10) | (p as u16) }
#[inline(always)]
fn PRIMARYLANGID(lgid: u16) -> u16 { lgid & 0x3ff }
#[inline(always)]
fn SUBLANGID(lgid: u16) -> u16 { lgid >> 10 }
#[inline(always)]
fn LANGIDFROMLCID(lcid: LCID) -> u16 { lcid as u16 }
#[inline(always)]
fn UInt32x32To64(a: u32, b: u32) -> u64 { (a as u64) * (b as u64) }

// ---- file-scope preamble (translated) ----
// These three are referenced cross-module (rtl/image.c, rtl/registry.c,
// ldr/ldrinit.c) so they must keep their C symbol names.
#[no_mangle]
pub static mut RtlpAllocDeallocQueryBufferSize: SIZE_T = PAGE_SIZE as SIZE_T;
#[no_mangle]
pub static mut LdrpTopLevelDllBeingLoadedTeb: PTEB = core::ptr::null_mut();
#[no_mangle]
pub static mut MmHighestUserAddress: PVOID = 0x7FFEFFFF as PVOID;

#[repr(C)]
pub struct _RTL_ATOM_HANDLE {
    pub Handle: RTL_HANDLE_TABLE_ENTRY,
    pub AtomEntry: PRTL_ATOM_TABLE_ENTRY,
}

pub type RTL_ATOM_HANDLE = _RTL_ATOM_HANDLE;
pub type PRTL_ATOM_HANDLE = *mut _RTL_ATOM_HANDLE;

pub const TAG_USTR: u32 = 0x52545355;

static mut CachedApisetVersion: DWORD = !0u32;

#[no_mangle]
pub unsafe extern "system" fn RtlpCheckForActiveDebugger() -> u8 {
    (*NtCurrentPeb()).BeingDebugged
}

#[no_mangle]
pub unsafe extern "system" fn RtlpSetInDbgPrint() -> u8 {
    if (*NtCurrentTeb()).InDbgPrint != 0 {
        return 1u8;
    }
    (*NtCurrentTeb()).InDbgPrint = 1u8;
    0u8
}
#[no_mangle]
pub unsafe extern "system" fn RtlpClearInDbgPrint() {
    (*NtCurrentTeb()).InDbgPrint = 0u8;
}

#[no_mangle]
pub unsafe extern "system" fn RtlpGetMode() -> KPROCESSOR_MODE {
    1 // UserMode
}

#[no_mangle]
pub unsafe extern "system" fn RtlGetCurrentPeb() -> PPEB {
    NtCurrentPeb()
}

#[no_mangle]
pub unsafe extern "system" fn RtlAcquirePebLock() {
    let Peb = NtCurrentPeb();
    RtlEnterCriticalSection((*Peb).FastPebLock);
}

#[no_mangle]
pub unsafe extern "system" fn RtlReleasePebLock() {
    let Peb = NtCurrentPeb();
    RtlLeaveCriticalSection((*Peb).FastPebLock);
}

#[no_mangle]
pub unsafe extern "system" fn RtlGetNtGlobalFlags() -> ULONG {
    let pPeb = NtCurrentPeb();
    (*pPeb).NtGlobalFlag
}

// ---- PEB/TEB/PE-header accessor shims for the rtl-strings Rust waves ----
// These mirror the C helpers in dll/ntdll/rtl/libsupp.c. They let the shared
// rtl-strings ENV/PRC/THR/PATH waves read process/thread state without
// mirroring per-arch PEB/TEB/IMAGE_NT_HEADERS layouts. The C `libsupp.c` flip
// dropped them; restored here so the path.c flip links in the ntdll user link.

/// `RtlpGetPebEnvironment` — `Peb->ProcessParameters->Environment`.
#[no_mangle]
pub unsafe extern "system" fn RtlpGetPebEnvironment() -> PWSTR {
    let Peb = NtCurrentPeb();
    if Peb.is_null() || (*Peb).ProcessParameters.is_null() {
        return core::ptr::null_mut();
    }
    (*(*Peb).ProcessParameters).Environment
}

/// `RtlpSetPebEnvironment` — set `Peb->ProcessParameters->Environment`.
#[no_mangle]
pub unsafe extern "system" fn RtlpSetPebEnvironment(NewEnvironment: PWSTR) {
    let Peb = NtCurrentPeb();
    if !Peb.is_null() && !(*Peb).ProcessParameters.is_null() {
        (*(*Peb).ProcessParameters).Environment = NewEnvironment;
    }
}

/// `RtlpThrGetPebImageBaseAddress` — `Peb->ImageBaseAddress`.
#[no_mangle]
pub unsafe extern "system" fn RtlpThrGetPebImageBaseAddress() -> PVOID {
    let Peb = NtCurrentPeb();
    if Peb.is_null() { core::ptr::null_mut() } else { (*Peb).ImageBaseAddress }
}

/// `RtlpThrGetPebMinimumStackCommit` — `Peb->MinimumStackCommit`.
#[no_mangle]
pub unsafe extern "system" fn RtlpThrGetPebMinimumStackCommit() -> SIZE_T {
    let Peb = NtCurrentPeb();
    if Peb.is_null() { 0 } else { (*Peb).MinimumStackCommit as SIZE_T }
}

/// `RtlpThrGetPebNtGlobalFlag` — `Peb->NtGlobalFlag`.
#[no_mangle]
pub unsafe extern "system" fn RtlpThrGetPebNtGlobalFlag() -> ULONG {
    let Peb = NtCurrentPeb();
    if Peb.is_null() { 0 } else { (*Peb).NtGlobalFlag }
}

/// `RtlpThrSetFreeStackOnTermination` — `NtCurrentTeb()->FreeStackOnTermination = TRUE`.
#[no_mangle]
pub unsafe extern "system" fn RtlpThrSetFreeStackOnTermination() {
    (*NtCurrentTeb()).FreeStackOnTermination = 1;
}

/// `RtlpThrTebDeallocStackAddr` — `&((PTEB)TebBase)->DeallocationStack`.
#[no_mangle]
pub unsafe extern "system" fn RtlpThrTebDeallocStackAddr(TebBase: PVOID) -> PVOID {
    core::ptr::addr_of_mut!((*(TebBase as PTEB)).DeallocationStack) as PVOID
}

/// True when `Headers` is a PE32 optional header rather than this build's
/// native one.
///
/// A WoW64 process's image is a PE32 while this code is compiled 64-bit, so
/// `IMAGE_OPTIONAL_HEADER32` packs SizeOfStackReserve/SizeOfStackCommit as
/// adjacent 4-byte fields exactly where the 64-bit layout has a single 8-byte
/// SizeOfStackReserve. Reading it at native width therefore returns
/// `(SizeOfStackCommit << 32) | SizeOfStackReserve` — for a stock 1MB/4KB i386
/// image that is a 16TB reserve, and `ZwAllocateVirtualMemory` rejects it with
/// STATUS_INVALID_PARAMETER_4, failing every `RtlCreateUserThread` in the
/// process. Only a WoW64 process reaches this, which is why it stayed hidden
/// until 32-bit guests could create threads (task #64).
///
/// `Magic` sits at offset 0 of the optional header in both layouts, so reading
/// it before choosing a layout is always safe.
#[inline(always)]
unsafe fn RtlpThrImageIsPe32(Headers: PVOID) -> bool {
    (*(Headers as PIMAGE_NT_HEADERS32)).OptionalHeader.Magic as u32
        == IMAGE_NT_OPTIONAL_HDR32_MAGIC
}

/// `RtlpThrImageStackReserve` — `((PIMAGE_NT_HEADERS)Headers)->OptionalHeader.SizeOfStackReserve`,
/// read at the width the header itself declares (see [`RtlpThrImageIsPe32`]).
#[no_mangle]
pub unsafe extern "system" fn RtlpThrImageStackReserve(Headers: PVOID) -> SIZE_T {
    if RtlpThrImageIsPe32(Headers) {
        return (*(Headers as PIMAGE_NT_HEADERS32)).OptionalHeader.SizeOfStackReserve as SIZE_T;
    }
    (*(Headers as PIMAGE_NT_HEADERS)).OptionalHeader.SizeOfStackReserve as SIZE_T
}

/// `RtlpThrImageStackCommit` — `((PIMAGE_NT_HEADERS)Headers)->OptionalHeader.SizeOfStackCommit`,
/// read at the width the header itself declares (see [`RtlpThrImageIsPe32`]).
#[no_mangle]
pub unsafe extern "system" fn RtlpThrImageStackCommit(Headers: PVOID) -> SIZE_T {
    if RtlpThrImageIsPe32(Headers) {
        return (*(Headers as PIMAGE_NT_HEADERS32)).OptionalHeader.SizeOfStackCommit as SIZE_T;
    }
    (*(Headers as PIMAGE_NT_HEADERS)).OptionalHeader.SizeOfStackCommit as SIZE_T
}

/// `RtlpThrCurrentTeb` — `(PVOID)NtCurrentTeb()`.
#[no_mangle]
pub unsafe extern "system" fn RtlpThrCurrentTeb() -> PVOID {
    NtCurrentTeb() as PVOID
}

/// `RtlpPrcGetPebOSMajorVersion` — `Peb->OSMajorVersion`.
#[no_mangle]
pub unsafe extern "system" fn RtlpPrcGetPebOSMajorVersion() -> ULONG {
    let Peb = NtCurrentPeb();
    if Peb.is_null() { 0 } else { (*Peb).OSMajorVersion }
}

/// `RtlpPrcGetPebOSMinorVersion` — `Peb->OSMinorVersion`.
#[no_mangle]
pub unsafe extern "system" fn RtlpPrcGetPebOSMinorVersion() -> ULONG {
    let Peb = NtCurrentPeb();
    if Peb.is_null() { 0 } else { (*Peb).OSMinorVersion }
}

/// `RtlpPrcPebProcessParametersAddr` — `&((PPEB)PebBase)->ProcessParameters`.
#[no_mangle]
pub unsafe extern "system" fn RtlpPrcPebProcessParametersAddr(PebBase: PVOID) -> PVOID {
    core::ptr::addr_of_mut!((*(PebBase as PPEB)).ProcessParameters) as PVOID
}

/// `RtlGetProcessHeap` — FORCEINLINE macro in C; real out-of-line symbol for
/// the Rust callers (`_RtlGetProcessHeap@0` on i386). `Peb->ProcessHeap`.
#[no_mangle]
pub unsafe extern "system" fn RtlGetProcessHeap() -> PVOID {
    rtl_get_process_heap()
}

#[no_mangle]
pub unsafe extern "system" fn RtlDeleteHeapLock(Lock: PHEAP_LOCK) -> NTSTATUS {
    RtlDeleteCriticalSection(core::ptr::addr_of_mut!((*Lock).__bindgen_anon_1.CriticalSection))
}

#[no_mangle]
pub unsafe extern "system" fn RtlEnterHeapLock(
    Lock: PHEAP_LOCK,
    Exclusive: u8,
) -> i32 {
    let _ = Exclusive;
    RtlEnterCriticalSection(core::ptr::addr_of_mut!((*Lock).__bindgen_anon_1.CriticalSection))
}

#[no_mangle]
pub unsafe extern "system" fn RtlTryEnterHeapLock(
    Lock: PHEAP_LOCK,
    Exclusive: u8,
) -> u8 {
    let _ = Exclusive;

    RtlTryEnterCriticalSection(core::ptr::addr_of_mut!((*Lock).__bindgen_anon_1.CriticalSection)) as u8
}

#[no_mangle]
pub unsafe extern "system" fn RtlInitializeHeapLock(Lock: *mut PHEAP_LOCK) -> NTSTATUS {
    RtlInitializeCriticalSection(core::ptr::addr_of_mut!((*(*Lock)).__bindgen_anon_1.CriticalSection))
}

#[no_mangle]
pub unsafe extern "system" fn RtlLeaveHeapLock(Lock: PHEAP_LOCK) -> NTSTATUS {
    RtlLeaveCriticalSection(core::ptr::addr_of_mut!((*Lock).__bindgen_anon_1.CriticalSection))
}
#[no_mangle]
pub unsafe extern "system" fn RtlpAllocateMemory(Bytes: UINT, Tag: ULONG) -> PVOID {
    let _ = Tag;
    
    RtlAllocateHeap(rtl_get_process_heap(), 0, Bytes as SIZE_T)
}
#[no_mangle]
pub unsafe extern "system" fn RtlpFreeMemory(Mem: PVOID, _Tag: ULONG) {

    RtlFreeHeap(rtl_get_process_heap(), 0, Mem);
}
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn CHECK_PAGED_CODE_RTL(_file: *mut i8, _line: i32) {
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn CHECK_PAGED_CODE_RTL(_file: *mut i8, _line: i32) {
}


#[no_mangle]
pub unsafe extern "system" fn RtlpSetHeapParameters(Parameters: PRTL_HEAP_PARAMETERS) {
    let Peb: PPEB;

    Peb = RtlGetCurrentPeb();

    if (*Parameters).SegmentCommit == 0 {
        (*Parameters).SegmentCommit = (*Peb).HeapSegmentCommit;
    }
    if (*Parameters).SegmentReserve == 0 {
        (*Parameters).SegmentReserve = (*Peb).HeapSegmentReserve;
    }
    if (*Parameters).DeCommitFreeBlockThreshold == 0 {
        (*Parameters).DeCommitFreeBlockThreshold = (*Peb).HeapDeCommitFreeBlockThreshold;
    }
    if (*Parameters).DeCommitTotalFreeThreshold == 0 {
        (*Parameters).DeCommitTotalFreeThreshold = (*Peb).HeapDeCommitTotalFreeThreshold;
    }
}

#[no_mangle]
pub unsafe extern "system" fn RtlpHandleDpcStackException(
    _RegistrationFrame: PEXCEPTION_REGISTRATION_RECORD,
    _RegistrationFrameEnd: ULONG_PTR,
    _StackLow: PULONG_PTR,
    _StackHigh: PULONG_PTR,
) -> BOOLEAN {
    0u8
}
#[no_mangle]
pub unsafe extern "system" fn RtlpCheckLogException(
    _ExceptionRecord: PEXCEPTION_RECORD,
    _ContextRecord: PCONTEXT,
    _ContextData: PVOID,
    _Size: ULONG,
) {
}
#[no_mangle]
pub unsafe extern "system" fn RtlpCaptureStackLimits(
    _Ebp: ULONG_PTR,
    StackBegin: *mut ULONG_PTR,
    StackEnd: *mut ULONG_PTR,
) -> BOOLEAN {
    *StackBegin = (*NtCurrentTeb()).NtTib.StackLimit as ULONG_PTR;
    *StackEnd = (*NtCurrentTeb()).NtTib.StackBase as ULONG_PTR;
    1u8
}
#[cfg(not(target_arch = "x86_64"))]
#[no_mangle]
pub unsafe extern "system" fn RtlWalkFrameChain(
    Callers: *mut PVOID,
    Count: ULONG,
    Flags: ULONG,
) -> ULONG {
    // RtlWalkFrameChain(NULL, ...) must return 0. The C reference caught the NULL
    // output-store fault with _SEH2_EXCEPT{ i = 0 }; that __except is dead here, so a NULL
    // Callers would AV on the `*Callers.offset(i) = ...` store below. In user mode native
    // SEH works, but this fn is called WITHOUT a caller __try (kmtest RtlStack.c:32), so the
    // AV self-crashes the process (exit 128, the RtlStack UM quarantine). Guard it explicitly.
    if Callers.is_null() {
        return 0;
    }
    let mut Stack: ULONG_PTR;
    let mut NewStack: ULONG_PTR;
    let mut StackBegin: ULONG_PTR;
    let mut StackEnd: ULONG_PTR = 0;
    let mut Eip: ULONG_PTR; // the return address, pointer-wide (a ULONG truncated it on 64-bit)
    let Result: BOOLEAN;
    let mut StopSearch: BOOLEAN = 0u8;
    let mut i: ULONG = 0;

    #[cfg(target_arch = "x86")]
    {
        core::arch::asm!("mov {}, ebp", out(reg) Stack);
    }

    #[cfg(target_arch = "powerpc")]
    {
        core::arch::asm!("mr {},1", out(reg) Stack);
    }

    #[cfg(target_arch = "arm")]
    {
        #[cfg(target_env = "gnu")]
        {
            core::arch::asm!("mov {}, sp", out(reg) Stack);
        }
        #[cfg(not(target_env = "gnu"))]
        {
            Stack = core::ptr::addr_of!(Stack) as ULONG_PTR;
        }
    }

    // The walk below reads [frame] = caller's frame, [frame + ptr] = return address: the
    // x86 EBP chain, and on AArch64 exactly the x29 frame record (fp, lr). It must start at
    // x29 -- starting at sp read locals as frame records and returned 0 frames (RtlStack UM).
    #[cfg(target_arch = "aarch64")]
    {
        core::arch::asm!("mov {}, x29", out(reg) Stack);
    }

    StackBegin = Stack;

    if Flags == 0 {
        Result = RtlpCaptureStackLimits(
            Stack,
            core::ptr::addr_of_mut!(StackBegin),
            core::ptr::addr_of_mut!(StackEnd),
        );
        if Result == 0 {
            return 0;
        }
    }

    {
        i = 0;
        while i < Count {
            if (Stack >= StackEnd)
                || (if i == 0 { Stack < StackBegin } else { Stack <= StackBegin })
                || ((StackEnd - Stack) < (2 * core::mem::size_of::<ULONG_PTR>() as ULONG_PTR))
            {
                break;
            }

            NewStack = *(Stack as PULONG_PTR);
            Eip = *((Stack + core::mem::size_of::<ULONG_PTR>() as ULONG_PTR) as PULONG_PTR);

            if !((Stack < NewStack) && (NewStack < StackEnd)) {
                StopSearch = 1u8;
            }

            if (StackBegin < Eip as ULONG_PTR) && ((Eip as ULONG_PTR) < StackEnd) {
                break;
            }

            *Callers.offset(i as isize) = Eip as PVOID;

            if StopSearch != 0 {
                i += 1;
                break;
            }

            Stack = NewStack;
            i += 1;
        }
    }

    i
}

#[no_mangle]
pub unsafe extern "system" fn RtlpGetStackLimits(
    LowLimit: PULONG_PTR,
    HighLimit: PULONG_PTR,
) {
    *LowLimit = (*NtCurrentTeb()).NtTib.StackLimit as ULONG_PTR;
    *HighLimit = (*NtCurrentTeb()).NtTib.StackBase as ULONG_PTR;
}

#[no_mangle]
pub unsafe extern "system" fn RtlIsThreadWithinLoaderCallout() -> u8 {
    if LdrpTopLevelDllBeingLoadedTeb == NtCurrentTeb() { 1 } else { 0 }
}
#[no_mangle]
pub unsafe extern "C" fn RtlpInitAtomTableLock(AtomTable: PRTL_ATOM_TABLE) -> NTSTATUS {
    RtlInitializeCriticalSection(core::ptr::addr_of_mut!((*AtomTable).__bindgen_anon_1.CriticalSection));
    STATUS_SUCCESS
}
#[no_mangle]
pub unsafe extern "C" fn RtlpDestroyAtomTableLock(AtomTable: PRTL_ATOM_TABLE) {
    RtlDeleteCriticalSection(core::ptr::addr_of_mut!((*AtomTable).__bindgen_anon_1.CriticalSection));
}
#[no_mangle]
pub unsafe extern "C" fn RtlpLockAtomTable(AtomTable: PRTL_ATOM_TABLE) -> u8 {
    RtlEnterCriticalSection(core::ptr::addr_of_mut!((*AtomTable).__bindgen_anon_1.CriticalSection));
    1u8
}
#[no_mangle]
pub unsafe extern "C" fn RtlpUnlockAtomTable(AtomTable: PRTL_ATOM_TABLE) {
    RtlLeaveCriticalSection(core::ptr::addr_of_mut!((*AtomTable).__bindgen_anon_1.CriticalSection));
}
#[no_mangle]
pub unsafe extern "C" fn RtlpCreateAtomHandleTable(AtomTable: PRTL_ATOM_TABLE) -> u8 {
    RtlInitializeHandleTable(0xCFFF,
                            core::mem::size_of::<RTL_ATOM_HANDLE>() as u32,
                            core::ptr::addr_of_mut!((*AtomTable).__bindgen_anon_2.RtlHandleTable));
    
    1u8
}
#[no_mangle]
pub unsafe extern "C" fn RtlpDestroyAtomHandleTable(AtomTable: PRTL_ATOM_TABLE) {
    RtlDestroyHandleTable(core::ptr::addr_of_mut!((*AtomTable).__bindgen_anon_2.RtlHandleTable));
}
#[no_mangle]
pub unsafe extern "C" fn RtlpAllocAtomTable(Size: u32) -> PRTL_ATOM_TABLE {
    
    RtlAllocateHeap(rtl_get_process_heap(), HEAP_ZERO_MEMORY, Size as SIZE_T) as PRTL_ATOM_TABLE
}
#[no_mangle]
pub unsafe extern "C" fn RtlpFreeAtomTable(AtomTable: PRTL_ATOM_TABLE) {
    
    RtlFreeHeap(rtl_get_process_heap(), 0, AtomTable as PVOID);
}
#[no_mangle]
pub unsafe extern "C" fn RtlpAllocAtomTableEntry(Size: u32) -> PRTL_ATOM_TABLE_ENTRY {
    
    RtlAllocateHeap(rtl_get_process_heap(), HEAP_ZERO_MEMORY, Size as SIZE_T) as PRTL_ATOM_TABLE_ENTRY
}
#[no_mangle]
pub unsafe extern "C" fn RtlpFreeAtomTableEntry(Entry: PRTL_ATOM_TABLE_ENTRY) {
    RtlFreeHeap(rtl_get_process_heap(), 0, Entry as PVOID);
}
#[no_mangle]
pub unsafe extern "C" fn RtlpFreeAtomHandle(AtomTable: PRTL_ATOM_TABLE, Entry: PRTL_ATOM_TABLE_ENTRY) {
    let mut RtlHandleEntry: PRTL_HANDLE_TABLE_ENTRY = core::ptr::null_mut();

    if RtlIsValidIndexHandle(
        core::ptr::addr_of_mut!((*AtomTable).__bindgen_anon_2.RtlHandleTable),
        (*Entry).HandleIndex as u32,
        core::ptr::addr_of_mut!(RtlHandleEntry),
    ) != 0
    {
        RtlFreeHandle(
            core::ptr::addr_of_mut!((*AtomTable).__bindgen_anon_2.RtlHandleTable),
            RtlHandleEntry,
        );
    }
}
#[no_mangle]
pub unsafe extern "C" fn RtlpCreateAtomHandle(AtomTable: PRTL_ATOM_TABLE, Entry: PRTL_ATOM_TABLE_ENTRY) -> u8 {
    let mut HandleIndex: u32 = 0;
    let RtlHandle: PRTL_HANDLE_TABLE_ENTRY = RtlAllocateHandle(
        core::ptr::addr_of_mut!((*AtomTable).__bindgen_anon_2.RtlHandleTable),
        &mut HandleIndex
    );
    
    if !RtlHandle.is_null() {
        let AtomHandle: *mut _RTL_ATOM_HANDLE = RtlHandle as *mut _RTL_ATOM_HANDLE;
        
        // Exceeds the C (bound 0xC000): the atom is 0xC000 + index in a
        // USHORT, so an index from 0x4000 up wrapped below 0xC000.
        if HandleIndex < 0x4000 {
            (*Entry).HandleIndex = HandleIndex as u16;
            (*Entry).Atom = 0xC000 + HandleIndex as u16;
            
            (*AtomHandle).AtomEntry = Entry;
            (*AtomHandle).Handle.__bindgen_anon_1.Flags = RTL_HANDLE_VALID;
            
            return 1u8;
        } else {
            (*AtomHandle).Handle.__bindgen_anon_1.Flags = RTL_HANDLE_VALID;
            
            RtlFreeHandle(
                core::ptr::addr_of_mut!((*AtomTable).__bindgen_anon_2.RtlHandleTable),
                RtlHandle
            );
        }
    }
    
    0u8
}
#[no_mangle]
pub unsafe extern "C" fn RtlpGetAtomEntry(AtomTable: PRTL_ATOM_TABLE, Index: u32) -> PRTL_ATOM_TABLE_ENTRY {
    let mut RtlHandle: PRTL_HANDLE_TABLE_ENTRY = core::ptr::null_mut();
    
    if RtlIsValidIndexHandle(
        core::ptr::addr_of_mut!((*AtomTable).__bindgen_anon_2.RtlHandleTable),
        Index,
        core::ptr::addr_of_mut!(RtlHandle)
    ) != 0
    {
        let AtomHandle = RtlHandle as *mut _RTL_ATOM_HANDLE;
        return (*AtomHandle).AtomEntry;
    }
    
    core::ptr::null_mut()
}

// vista build: loader-support machinery whose externs (RtlpImageNtHeaderEx,
// push_language, find_entry_by_*, actctx) resolve only in the full ntdll link.
#[cfg(not(feature = "vista"))]
#[no_mangle]
pub unsafe extern "system" fn RtlImageNtHeaderEx(
    Flags: ULONG,
    Base: PVOID,
    Size: ULONG64,
    OutHeaders: *mut PIMAGE_NT_HEADERS,
) -> NTSTATUS {
    let Status: NTSTATUS;

    if !OutHeaders.is_null() {
        *OutHeaders = core::ptr::null_mut();
    }

    Status = RtlpImageNtHeaderEx(Flags, Base, Size, OutHeaders);

    return Status;
}
// vista build: loader-support machinery whose externs (RtlpImageNtHeaderEx,
// push_language, find_entry_by_*, actctx) resolve only in the full ntdll link.
#[cfg(not(feature = "vista"))]
#[no_mangle]
pub unsafe extern "C" fn find_entry(
    BaseAddress: PVOID,
    info: *mut LDR_RESOURCE_INFO,
    mut level: ULONG,
    ret: *mut *mut core::ffi::c_void,
    want_dir: i32,
) -> NTSTATUS {
    let mut size: ULONG = 0;
    let root: *mut core::ffi::c_void;
    let mut resdirptr: *mut IMAGE_RESOURCE_DIRECTORY;
    let mut list: [USHORT; 9] = [0; 9];
    let mut i: i32;
    let mut pos: i32 = 0;
    let mut user_lcid: LCID = 0;
    let mut system_lcid: LCID = 0;

    root = RtlImageDirectoryEntryToData(
        BaseAddress,
        1u8,
        IMAGE_DIRECTORY_ENTRY_RESOURCE as USHORT,
        &mut size,
    );
    if root.is_null() {
        return STATUS_RESOURCE_DATA_NOT_FOUND;
    }
    if size < core::mem::size_of::<IMAGE_RESOURCE_DIRECTORY>() as ULONG {
        return STATUS_RESOURCE_DATA_NOT_FOUND;
    }
    resdirptr = root as *mut IMAGE_RESOURCE_DIRECTORY;

    level = level.wrapping_sub(1);
    if level == 0xFFFFFFFF {
        *ret = resdirptr as *mut core::ffi::c_void;
        return STATUS_SUCCESS;
    }

    *ret = find_entry_by_name(
        resdirptr,
        (*info).Type as LPCWSTR,
        root,
        if want_dir != 0 || level != 0 { 1 } else { 0 },
    ) as *mut core::ffi::c_void;
    if (*ret).is_null() {
        return STATUS_RESOURCE_TYPE_NOT_FOUND;
    }
    level = level.wrapping_sub(1);
    if level == 0xFFFFFFFF {
        return STATUS_SUCCESS;
    }

    resdirptr = *ret as *mut IMAGE_RESOURCE_DIRECTORY;
    *ret = find_entry_by_name(
        resdirptr,
        (*info).Name as LPCWSTR,
        root,
        if want_dir != 0 || level != 0 { 1 } else { 0 },
    ) as *mut core::ffi::c_void;
    if (*ret).is_null() {
        return STATUS_RESOURCE_NAME_NOT_FOUND;
    }
    level = level.wrapping_sub(1);
    if level == 0xFFFFFFFF {
        return STATUS_SUCCESS;
    }
    if level != 0 {
        return STATUS_INVALID_PARAMETER;
    }

    pos = push_language(list.as_mut_ptr(), pos as ULONG, (*info).Language as WORD);

    pos = push_language(
        list.as_mut_ptr(),
        pos as ULONG,
        MAKELANGID(PRIMARYLANGID((*info).Language as u16), SUBLANG_NEUTRAL as u16),
    );

    pos = push_language(
        list.as_mut_ptr(),
        pos as ULONG,
        MAKELANGID(LANG_NEUTRAL as u16, SUBLANG_NEUTRAL as u16),
    );

    if PRIMARYLANGID((*info).Language as u16) == LANG_NEUTRAL as u16 {
        if SUBLANGID((*info).Language as u16) != SUBLANG_SYS_DEFAULT as u16 {
            pos = push_language(
                list.as_mut_ptr(),
                pos as ULONG,
                LANGIDFROMLCID((*NtCurrentTeb()).CurrentLocale) as WORD,
            );

            if NT_SUCCESS(NtQueryDefaultLocale(1u8, &mut user_lcid)) {
                pos = push_language(list.as_mut_ptr(), pos as ULONG, LANGIDFROMLCID(user_lcid) as WORD);

                pos = push_language(
                    list.as_mut_ptr(),
                    pos as ULONG,
                    MAKELANGID(PRIMARYLANGID(user_lcid as u16), SUBLANG_NEUTRAL as u16),
                );
            }
        }

        if NT_SUCCESS(NtQueryDefaultLocale(0u8, &mut system_lcid)) {
            pos = push_language(
                list.as_mut_ptr(),
                pos as ULONG,
                LANGIDFROMLCID(system_lcid) as WORD,
            );

            pos = push_language(
                list.as_mut_ptr(),
                pos as ULONG,
                MAKELANGID(PRIMARYLANGID(system_lcid as u16), SUBLANG_NEUTRAL as u16),
            );
        }

        pos = push_language(
            list.as_mut_ptr(),
            pos as ULONG,
            MAKELANGID(LANG_ENGLISH as u16, SUBLANG_DEFAULT as u16),
        );
    }

    resdirptr = *ret as *mut IMAGE_RESOURCE_DIRECTORY;
    i = 0;
    while i < pos {
        *ret = find_entry_by_id(resdirptr, list[i as usize], root, want_dir) as *mut core::ffi::c_void;
        if !(*ret).is_null() {
            return STATUS_SUCCESS;
        }
        i += 1;
    }

    if PRIMARYLANGID((*info).Language as u16) == LANG_NEUTRAL as u16 {
        *ret = find_first_entry(resdirptr, root, want_dir) as *mut core::ffi::c_void;
        if !(*ret).is_null() {
            return STATUS_SUCCESS;
        }
    }
    STATUS_RESOURCE_LANG_NOT_FOUND
}

// vista build: loader-support machinery whose externs (RtlpImageNtHeaderEx,
// push_language, find_entry_by_*, actctx) resolve only in the full ntdll link.
#[cfg(not(feature = "vista"))]
#[no_mangle]
pub unsafe extern "system" fn RtlPcToFileHeader(
    PcValue: PVOID,
    BaseOfImage: *mut PVOID,
) -> PVOID {
    let ModuleListHead: PLIST_ENTRY;
    let mut Entry: PLIST_ENTRY;
    let mut Module: PLDR_DATA_TABLE_ENTRY;
    let mut ImageBase: PVOID = core::ptr::null_mut();

    if crate::ldr::ldrinit::LdrpLdrData().is_null() {
        *BaseOfImage = core::ptr::null_mut();
        return core::ptr::null_mut();
    }

    RtlEnterCriticalSection(core::ptr::addr_of_mut!(crate::ldr::ldrinit::LdrpLoaderLock));
    ModuleListHead = core::ptr::addr_of_mut!((*crate::ldr::ldrinit::LdrpLdrData()).InLoadOrderModuleList);
    Entry = (*ModuleListHead).Flink;
    while Entry != ModuleListHead {
        Module = (Entry as *mut u8).offset(-(core::mem::offset_of!(LDR_DATA_TABLE_ENTRY, InLoadOrderLinks) as isize)) as PLDR_DATA_TABLE_ENTRY;

        if (PcValue as ULONG_PTR) >= ((*Module).DllBase as ULONG_PTR) &&
            (PcValue as ULONG_PTR) < ((*Module).DllBase as ULONG_PTR) + (*Module).SizeOfImage as ULONG_PTR {
            ImageBase = (*Module).DllBase;
            break;
        }
        Entry = (*Entry).Flink;
    }
    RtlLeaveCriticalSection(core::ptr::addr_of_mut!(crate::ldr::ldrinit::LdrpLoaderLock));

    *BaseOfImage = ImageBase;
    ImageBase
}
// vista build: loader-support machinery whose externs (RtlpImageNtHeaderEx,
// push_language, find_entry_by_*, actctx) resolve only in the full ntdll link.
#[cfg(not(feature = "vista"))]
#[no_mangle]
pub unsafe extern "C" fn get_buffer(
    buffer: *mut LPWSTR,
    needed: usize,
    CallerBuffer: PUNICODE_STRING,
    bAllocateBuffer: u8,
) -> i32 {
    let p: LPWSTR;

    if !CallerBuffer.is_null() && (*CallerBuffer).MaximumLength as usize > needed {
        p = (*CallerBuffer).Buffer;
        (*CallerBuffer).Length = (needed - core::mem::size_of::<u16>()) as u16;
    } else {
        if bAllocateBuffer == 0 {
            return STATUS_BUFFER_TOO_SMALL;
        }

        if !CallerBuffer.is_null() {
            *((*CallerBuffer).Buffer) = 0;
        }

        p = RtlAllocateHeap(rtl_get_process_heap(), 0, needed as SIZE_T) as LPWSTR;
        if p.is_null() {
            return STATUS_NO_MEMORY;
        }
    }
    *buffer = p;

    return STATUS_SUCCESS;
}
// vista build: loader-support machinery whose externs (RtlpImageNtHeaderEx,
// push_language, find_entry_by_*, actctx) resolve only in the full ntdll link.
#[cfg(not(feature = "vista"))]
#[no_mangle]
pub unsafe extern "C" fn find_actctx_dll(
    pnameW: PUNICODE_STRING,
    fullname: *mut LPWSTR,
    CallerBuffer: PUNICODE_STRING,
    bAllocateBuffer: u8,
) -> i32 {

    static winsxsW: [u16; 8] = [92, 119, 105, 110, 115, 120, 115, 92];
    static dotManifestW: [u16; 10] = [46, 109, 97, 110, 105, 102, 101, 115, 116, 0];

    let mut info: *mut ACTIVATION_CONTEXT_ASSEMBLY_DETAILED_INFORMATION;
    let mut data: ACTCTX_SECTION_KEYED_DATA = core::mem::zeroed();
    let mut status: i32;
    let mut needed: usize = 0;
    let mut size: usize = 1024;
    let mut p: *mut u16;

    data.cbSize = core::mem::size_of::<ACTCTX_SECTION_KEYED_DATA>() as u32;
    status = RtlFindActivationContextSectionString(
        FIND_ACTCTX_SECTION_KEY_RETURN_HACTCTX,
        core::ptr::null_mut(),
        ACTIVATION_CONTEXT_SECTION_DLL_REDIRECTION,
        pnameW,
        &mut data as *mut _ as *mut c_void,
    );
    if status != STATUS_SUCCESS {
        return status;
    }

    'done: loop {
        loop {
            info = RtlAllocateHeap(rtl_get_process_heap(), 0, size as SIZE_T) as *mut ACTIVATION_CONTEXT_ASSEMBLY_DETAILED_INFORMATION;
            if info.is_null() {
                status = STATUS_NO_MEMORY;
                break 'done;
            }
            status = RtlQueryInformationActivationContext(
                0,
                data.hActCtx,
                core::ptr::addr_of_mut!(data.ulAssemblyRosterIndex) as *mut _,
                AssemblyDetailedInformationInActivationContext,
                info as *mut _,
                size as SIZE_T,
                &mut needed as *mut usize as *mut SIZE_T,
            );
            if status == STATUS_SUCCESS {
                break;
            }
            if status != STATUS_BUFFER_TOO_SMALL {
                break 'done;
            }
            RtlFreeHeap(rtl_get_process_heap(), 0, info as *mut _);
            size = needed;
        }

        if (*info).lpAssemblyManifestPath.is_null() {
            status = STATUS_SXS_KEY_NOT_FOUND;
            break 'done;
        }

        p = wcsrchr((*info).lpAssemblyManifestPath, 92);
        if !p.is_null() {
            let dirlen: u32 = (*info).ulAssemblyDirectoryNameLength / core::mem::size_of::<u16>() as u32;

            p = p.offset(1);
            if (*info).lpAssemblyDirectoryName.is_null()
                || _wcsnicmp(p, (*info).lpAssemblyDirectoryName, dirlen as usize) != 0
                || _wcsicmp(p.offset(dirlen as isize), dotManifestW.as_ptr()) != 0
            {
                let dirlen_manifest: u32 = ((p as usize - (*info).lpAssemblyManifestPath as usize) / core::mem::size_of::<u16>()) as u32;
                needed = (dirlen_manifest as usize + 1) * core::mem::size_of::<u16>() + (*pnameW).Length as usize;

                status = get_buffer(fullname, needed, CallerBuffer, bAllocateBuffer);
                if !NT_SUCCESS(status) {
                    break 'done;
                }

                p = *fullname;

                memcpy(
                    p as *mut u8,
                    (*info).lpAssemblyManifestPath as *const u8,
                    dirlen_manifest as usize * core::mem::size_of::<u16>(),
                );
                p = p.offset(dirlen_manifest as isize);
                memcpy(p as *mut u8, (*pnameW).Buffer as *const u8, (*pnameW).Length as usize);
                p = p.offset(((*pnameW).Length / core::mem::size_of::<u16>() as u16) as isize);
                *p = 0;

                break 'done;
            }
        }

        needed = wcslen((*shared_user_data()).NtSystemRoot.as_ptr()) * core::mem::size_of::<u16>()
            + core::mem::size_of_val(&winsxsW)
            + (*info).ulAssemblyDirectoryNameLength as usize
            + (*pnameW).Length as usize
            + 2 * core::mem::size_of::<u16>();

        status = get_buffer(fullname, needed, CallerBuffer, bAllocateBuffer);
        if !NT_SUCCESS(status) {
            break 'done;
        }

        p = *fullname;

        wcscpy(p, (*shared_user_data()).NtSystemRoot.as_ptr());
        p = p.offset(wcslen(p) as isize);
        memcpy(p as *mut u8, winsxsW.as_ptr() as *const u8, core::mem::size_of_val(&winsxsW));
        p = p.offset((core::mem::size_of_val(&winsxsW) / core::mem::size_of::<u16>()) as isize);
        memcpy(
            p as *mut u8,
            (*info).lpAssemblyDirectoryName as *const u8,
            (*info).ulAssemblyDirectoryNameLength as usize,
        );
        p = p.offset(((*info).ulAssemblyDirectoryNameLength / core::mem::size_of::<u16>() as u32) as isize);
        *p = 92;
        p = p.offset(1);
        memcpy(p as *mut u8, (*pnameW).Buffer as *const u8, (*pnameW).Length as usize);
        p = p.offset(((*pnameW).Length / core::mem::size_of::<u16>() as u16) as isize);
        *p = 0;

        break 'done;
    }

    RtlFreeHeap(rtl_get_process_heap(), 0, info as *mut _);
    RtlReleaseActivationContext(data.hActCtx);
    status
}

// vista build (ntdll_vista.dll) excludes the apisets/loader machinery these
// depend on (ApiSetResolveToHost, RtlDoesFileExists_UStr, RtlpImageNtHeaderEx
// resolve only in the full ntdll link); nothing in the vista PE calls them.
#[cfg(not(feature = "vista"))]
#[no_mangle]
pub unsafe extern "system" fn RtlDosApplyFileIsolationRedirection_Ustr(
    Flags: u32,
    OriginalName: PUNICODE_STRING,
    Extension: PUNICODE_STRING,
    StaticString: PUNICODE_STRING,
    DynamicString: PUNICODE_STRING,
    NewName: *mut PUNICODE_STRING,
    _NewFlags: *mut u32,
    _FileNameSize: PSIZE_T,
    _RequiredLength: PSIZE_T,
) -> i32 {

    let mut Status: i32;
    let mut fullname: LPWSTR = core::ptr::null_mut();
    let mut buffer: [u16; MAX_PATH as usize] = [0; MAX_PATH as usize];
    let mut localStr: _UNICODE_STRING = core::mem::zeroed();
    let mut localStr2: _UNICODE_STRING = core::mem::zeroed();
    let mut pstrParam: PUNICODE_STRING;
    let mut p: *mut u16;
    let mut GotExtension: u8;
    let mut c: u16;

    if OriginalName.is_null() {
        return STATUS_INVALID_PARAMETER;
    }

    if DynamicString.is_null() && StaticString.is_null() {
        return STATUS_INVALID_PARAMETER;
    }

    if !DynamicString.is_null() && !StaticString.is_null() && NewName.is_null() {
        return STATUS_INVALID_PARAMETER;
    }

    if (*OriginalName).Buffer.is_null() || (*OriginalName).Length == 0 {
        return STATUS_SXS_KEY_NOT_FOUND;
    }

    if !StaticString.is_null() && (OriginalName == StaticString || (*OriginalName).Buffer == (*StaticString).Buffer) {
        return STATUS_SXS_KEY_NOT_FOUND;
    }

    if (Flags & RTL_DOS_APPLY_FILE_REDIRECTION_USTR_FLAG_RESPECT_DOT_LOCAL) != 0 &&
        !(*NtCurrentPeb()).ProcessParameters.is_null() &&
        ((*(*NtCurrentPeb()).ProcessParameters).Flags & RTL_USER_PROCESS_PARAMETERS_PRIVATE_DLL_PATH) != 0
    {
        let mut RealName: _UNICODE_STRING = core::mem::zeroed();
        let mut LocalName: _UNICODE_STRING = core::mem::zeroed();
        let mut RealNameBuf: [u16; MAX_PATH as usize] = [0; MAX_PATH as usize];
        let mut LocalNameBuf: [u16; MAX_PATH as usize] = [0; MAX_PATH as usize];

        RtlInitEmptyUnicodeString(&mut RealName, RealNameBuf.as_mut_ptr(), core::mem::size_of_val(&RealNameBuf) as u16);
        RtlInitEmptyUnicodeString(&mut LocalName, LocalNameBuf.as_mut_ptr(), core::mem::size_of_val(&LocalNameBuf) as u16);

        Status = RtlComputePrivatizedDllName_U(OriginalName, &mut RealName, &mut LocalName);
        if !NT_SUCCESS(Status) {
            return Status;
        }

        if RtlDoesFileExists_UStr(&mut LocalName) != 0 {
            Status = get_buffer(&mut fullname, LocalName.Length as usize + core::mem::size_of::<u16>(), StaticString, if !DynamicString.is_null() { 1u8 } else { 0u8 });
            if NT_SUCCESS(Status) {
                core::ptr::copy_nonoverlapping(LocalName.Buffer as *const u8, fullname as *mut u8, LocalName.Length as usize + core::mem::size_of::<u16>());
            }
        } else if RtlDoesFileExists_UStr(&mut RealName) != 0 {
            Status = get_buffer(&mut fullname, RealName.Length as usize + core::mem::size_of::<u16>(), StaticString, if !DynamicString.is_null() { 1u8 } else { 0u8 });
            if NT_SUCCESS(Status) {
                core::ptr::copy_nonoverlapping(RealName.Buffer as *const u8, fullname as *mut u8, RealName.Length as usize + core::mem::size_of::<u16>());
            }
        } else {
            Status = STATUS_NOT_FOUND;
        }

        if RealName.Buffer != RealNameBuf.as_mut_ptr() {
            RtlFreeUnicodeString(&mut RealName);
        }
        if LocalName.Buffer != LocalNameBuf.as_mut_ptr() {
            RtlFreeUnicodeString(&mut LocalName);
        }

        if NT_SUCCESS(Status) {
            if StaticString.is_null() || (*StaticString).Buffer != fullname {
                RtlInitUnicodeString(DynamicString, fullname);
                *NewName = DynamicString;
            } else {
                *NewName = StaticString;
            }
            return Status;
        }
    }

    pstrParam = OriginalName;

    p = (*OriginalName).Buffer.offset(((*OriginalName).Length / core::mem::size_of::<u16>() as u16 - 1) as isize);
    GotExtension = 0u8;
    while p >= (*OriginalName).Buffer {
        c = *p;
        p = p.offset(-1);
        if c == 0x002E {
            GotExtension = 1u8;
        } else if c == 0x005C {
            localStr.Buffer = p.offset(2);
            localStr.Length = (*OriginalName).Length - ((localStr.Buffer as usize - (*OriginalName).Buffer as usize) as u16);
            localStr.MaximumLength = (*OriginalName).MaximumLength - ((localStr.Buffer as usize - (*OriginalName).Buffer as usize) as u16);
            pstrParam = &mut localStr;
            break;
        }
    }

    if GotExtension == 0 {
        if Extension.is_null() {
            return STATUS_SXS_KEY_NOT_FOUND;
        }

        if (*pstrParam).Length + (*Extension).Length > core::mem::size_of_val(&buffer) as u16 {
            return STATUS_NO_MEMORY;
        }

        RtlInitEmptyUnicodeString(&mut localStr2, buffer.as_mut_ptr(), core::mem::size_of_val(&buffer) as u16);
        RtlAppendUnicodeStringToString(&mut localStr2, pstrParam);
        RtlAppendUnicodeStringToString(&mut localStr2, Extension);
        pstrParam = &mut localStr2;
    }

    Status = find_actctx_dll(pstrParam, &mut fullname, StaticString, if !DynamicString.is_null() { 1u8 } else { 0u8 });
    if !NT_SUCCESS(Status) {
        return Status;
    }

    if StaticString.is_null() || (*StaticString).Buffer != fullname {
        RtlInitUnicodeString(DynamicString, fullname);
        *NewName = DynamicString;
    } else {
        *NewName = StaticString;
    }

    Status
}

// vista build (ntdll_vista.dll) excludes the apisets/loader machinery these
// depend on (ApiSetResolveToHost, RtlDoesFileExists_UStr, RtlpImageNtHeaderEx
// resolve only in the full ntdll link); nothing in the vista PE calls them.
#[cfg(not(feature = "vista"))]
unsafe fn LdrpApisetVersion() -> DWORD {
    if CachedApisetVersion == !0u32 {

        let CompatVersion = RosGetProcessCompatVersion();

        match CompatVersion {
            0 => {}
            _WIN32_WINNT_VISTA => {
                CachedApisetVersion = 0;
            }
            _WIN32_WINNT_WIN7 => {
                CachedApisetVersion = APISET_WIN7;
            }
            _WIN32_WINNT_WIN8 => {
                CachedApisetVersion = APISET_WIN8;
            }
            _WIN32_WINNT_WINBLUE => {
                CachedApisetVersion = APISET_WIN81;
            }
            _WIN32_WINNT_WIN10 => {
                CachedApisetVersion = APISET_WIN10;
            }
            _ => {
                CachedApisetVersion = 0;
            }
        }
    }

    CachedApisetVersion
}
/// Off-by-default toggle for the [APISETDBG] api-set redirection trace in
/// LdrpApplyFileNameRedirection. See fex-bringup-plan.md rung 2.
const APISET_TRACE: bool = false;

// vista build (ntdll_vista.dll) excludes the apisets/loader machinery these
// depend on (ApiSetResolveToHost, RtlDoesFileExists_UStr, RtlpImageNtHeaderEx
// resolve only in the full ntdll link); nothing in the vista PE calls them.
#[cfg(not(feature = "vista"))]
#[no_mangle]
pub unsafe extern "system" fn LdrpApplyFileNameRedirection(
    OriginalName: PUNICODE_STRING,
    Extension: PUNICODE_STRING,
    StaticString: PUNICODE_STRING,
    DynamicString: PUNICODE_STRING,
    NewName: *mut PUNICODE_STRING,
    RedirectedDll: PBOOLEAN,
) -> i32 {

    const TAG_USTR: u32 = 0x52545355;
    const STATUS_INVALID_PARAMETER: i32 = 0xC000000D_u32 as i32;
    const STATUS_SXS_KEY_NOT_FOUND: i32 = 0xC0150008_u32 as i32;

    if OriginalName.is_null() {
        return STATUS_INVALID_PARAMETER;
    }

    if DynamicString.is_null() && StaticString.is_null() {
        return STATUS_INVALID_PARAMETER;
    }

    if NewName.is_null() {
        return STATUS_INVALID_PARAMETER;
    }

    *RedirectedDll = 0u8;

    let _PrevNewName = *NewName;
    let mut ApisetName: _UNICODE_STRING = core::mem::zeroed();
    let mut Status: i32 = STATUS_SUCCESS;

    let ApisetVersion = LdrpApisetVersion();
    if ApisetVersion != 0 {
        Status = ApiSetResolveToHost(ApisetVersion, OriginalName, RedirectedDll, &mut ApisetName);
        // [APISETDBG] api-set redirection trace, off by default. Flip
        // APISET_TRACE to debug api-ms-win-* resolution (the tool that found the
        // fexprobe UCRT-import failure — resolution worked but the built path
        // was wrong; fex-bringup-plan.md rung 2). Gated to api-* names only.
        if APISET_TRACE
            && (*OriginalName).Length >= 8
            && (*(*OriginalName).Buffer.add(0) | 0x20) == b'a' as u16
            && (*(*OriginalName).Buffer.add(1) | 0x20) == b'p' as u16
            && (*(*OriginalName).Buffer.add(2) | 0x20) == b'i' as u16
        {
            DbgPrint(
                b"[APISETDBG] name=%wZ ver=0x%x status=0x%x redir=%d target=%wZ\n\0".as_ptr()
                    as *const i8,
                OriginalName,
                ApisetVersion,
                Status,
                *RedirectedDll as u32,
                &mut ApisetName,
            );
        }
        if !NT_SUCCESS(Status) {
            return Status;
        }
    }

    if *RedirectedDll != 0 {
        // Emit the BARE host-DLL name (e.g. "ucrtbase.dll") and let the normal
        // DLL search path resolve it out of System32 — the same path every other
        // system import takes. (The stock ReactOS code prefixed
        // NtSystemRoot\System32\ to force an absolute load; on this arm64 port
        // that absolute path fails to map with STATUS_OBJECT_NAME_INVALID —
        // fexprobe was the first binary to import api-ms-win-crt-* api-sets and
        // surfaced it. The bare name + search path is robust and matches how the
        // initial kernel32/ntdll imports already resolve.)
        let Needed = ApisetName.Length as usize + 2;
        let ResultPath: PUNICODE_STRING;

        if !StaticString.is_null() && (*StaticString).MaximumLength >= Needed as u16 {
            (*StaticString).Length = 0;
            ResultPath = StaticString;
        } else if !DynamicString.is_null() {
            (*DynamicString).Buffer = RtlpAllocateStringMemory(Needed as SIZE_T, TAG_USTR) as *mut u16;
            if (*DynamicString).Buffer.is_null() {
                return STATUS_NO_MEMORY;
            }
            (*DynamicString).MaximumLength = Needed as u16;
            (*DynamicString).Length = 0;

            ResultPath = DynamicString;
        } else {
            return STATUS_INVALID_PARAMETER;
        }

        RtlAppendUnicodeStringToString(ResultPath, &ApisetName);
        *NewName = ResultPath;
        if APISET_TRACE {
            DbgPrint(
                b"[APISETDBG] redirect -> %wZ (search-path)\n\0".as_ptr() as *const i8,
                ResultPath,
            );
        }
    } else {
        Status = RtlDosApplyFileIsolationRedirection_Ustr(
            RTL_DOS_APPLY_FILE_REDIRECTION_USTR_FLAG_RESPECT_DOT_LOCAL,
            OriginalName,
            Extension,
            StaticString,
            DynamicString,
            NewName,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        );

        if NT_SUCCESS(Status) {
            *RedirectedDll = 1u8;
        } else if Status == STATUS_SXS_KEY_NOT_FOUND {
            Status = STATUS_SUCCESS;
        } else {
            if !DynamicString.is_null() && !(*DynamicString).Buffer.is_null() {
                RtlFreeUnicodeString(DynamicString);
            }
            return Status;
        }
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn RtlWow64EnableFsRedirection(
    _Wow64FsEnableRedirection: BOOLEAN,
) -> NTSTATUS {
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn RtlWow64EnableFsRedirectionEx(
    _Wow64FsEnableRedirection: PVOID,
    _OldFsRedirectionLevel: *mut PVOID,
) -> NTSTATUS {
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn RtlComputeImportTableHash(
    _FileHandle: HANDLE,
    _Hash: PCHAR,
    _ImportTableHashSize: ULONG,
) -> NTSTATUS {
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn RtlGetTickCount() -> ULONG {
    // TickCount lives in KUSER_SHARED_DATA's second anon union (KSYSTEM_TIME).
    let tc = core::ptr::addr_of!((*shared_user_data()).__bindgen_anon_2.TickCount);
    let low: ULONG;
    let high: ULONG;

    #[cfg(target_pointer_width = "64")]
    {
        // 64-bit: a single volatile read of the whole QuadPart.
        let q = core::ptr::read_volatile(tc as *const ULONG64);
        low = (q & 0xFFFF_FFFF) as ULONG;
        high = (q >> 32) as ULONG;
    }

    #[cfg(not(target_pointer_width = "64"))]
    {
        // 32-bit: spin until High1Time == High2Time (torn-read guard).
        let mut l: ULONG;
        let mut h: ULONG;
        loop {
            h = core::ptr::read_volatile(core::ptr::addr_of!((*tc).High1Time)) as ULONG;
            l = core::ptr::read_volatile(core::ptr::addr_of!((*tc).LowPart));
            if h == core::ptr::read_volatile(core::ptr::addr_of!((*tc).High2Time)) as ULONG {
                break;
            }
            core::hint::spin_loop();
        }
        low = l;
        high = h;
    }

    let mult = (*shared_user_data()).TickCountMultiplier;
    ((UInt32x32To64(low, mult) >> 24)
        + UInt32x32To64((high << 8) & 0xFFFF_FFFF, mult)) as ULONG
}

/// `RtlpSafeCopyMemory` — SEH-guarded copy. Under DUMMY_PSEH the `_SEH2_EXCEPT`
/// handler is dead code, so this reduces to a copy that always succeeds.
/// `NTSTATUS NTAPI` in the C → `extern "system"`.
#[no_mangle]
pub unsafe extern "system" fn RtlpSafeCopyMemory(
    Destination: PVOID,
    Source: *const c_void,
    Length: SIZE_T,
) -> NTSTATUS {
    memcpy(Destination as *mut u8, Source as *const u8, Length as usize);
    STATUS_SUCCESS
}
