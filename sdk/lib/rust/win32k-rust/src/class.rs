use win32k_sys::win32k::*;
use ntkrnl_sys::ddk::{ExAllocatePoolWithTag, ExFreePoolWithTag, PsGetCurrentProcessWin32Process, PsGetCurrentThreadWin32Thread, RtlAddAtomToAtomTable, RtlAllocateHeap, RtlDeleteAtomFromAtomTable, RtlFreeHeap, RtlInitUnicodeString, RtlLookupAtomInAtomTable, RtlQueryAtomInAtomTable, RtlUnicodeStringToAnsiString};
use ntkrnl_sys::rtl::{RtlCopyMemory, RtlZeroMemory};
use ntkrnl_sys::interlocked::InterlockedExchangePointer;
use ntkrnl_sys::base::{NT_SUCCESS, STATUS_INSUFFICIENT_RESOURCES, STATUS_SUCCESS};
use ntkrnl_sys::constants::{STATUS_ACCESS_VIOLATION, STATUS_BUFFER_TOO_SMALL, STATUS_INVALID_PARAMETER, STATUS_OBJECT_NAME_NOT_FOUND};
use core::ffi::c_void;

// SetLastNtError is FASTCALL (`@SetLastNtError@4`).
#[cfg(target_arch = "x86")]
extern "fastcall" {
    fn SetLastNtError(Status: NTSTATUS);
}
#[cfg(not(target_arch = "x86"))]
extern "C" {
    fn SetLastNtError(Status: NTSTATUS);
}

// SafeProbeForRead/Write are NTAPI (`_SafeProbeForRead@12`) returning NTSTATUS.
extern "system" {
    fn SafeProbeForRead(Address: PVOID, Length: SIZE_T, Alignment: u32) -> NTSTATUS;
    fn SafeProbeForWrite(Address: PVOID, Length: SIZE_T, Alignment: u32) -> NTSTATUS;
}

// pool tags (C multichar big-endian -> reversed bytes for from_le_bytes)
const TAG_STRING: u32 = u32::from_le_bytes(*b"STR "); // nolint: const-contradiction (' RTS' -- win32ss/user/ntuser/tags.h:3; freeldr's TAG_STRING is ' rtS')
const TAG_USTR: u32 = u32::from_le_bytes(*b"USTR");     // 'RTSU'
const USERTAG_CLASS: u32 = u32::from_le_bytes(*b"Uscl"); // 'lcsU'
const UNICODE_NULL: u16 = 0;

// --- macro reimplementations ---
// IS_ATOM!(x): (ULONG_PTR(x) > 0) && (ULONG_PTR(x) < 0x10000). Yields BOOL (i32)
// so call sites can use both `if IS_ATOM!(x) != 0` and bool contexts via `!= 0`.
macro_rules! IS_ATOM {
    ($x:expr) => {{ let v = ($x) as usize; ((v > 0) && (v < 0x10000)) as BOOL }};
}
// IS_INTRESOURCE!(i): (ULONG_PTR(i) >> 16) == 0. Yields BOOL (i32).
macro_rules! IS_INTRESOURCE {
    ($i:expr) => {{ ((($i) as usize >> 16) == 0) as BOOL }};
}
#[inline(always)]
fn ICLASS_TO_MASK(iCls: u32) -> u32 {
    1u32 << iCls
}
#[inline(always)]
fn HandleToUlong<T>(h: *mut T) -> ULONG {
    h as usize as ULONG
}

// `static __inline` heap allocs (usrheap.h / desktop.h).
#[inline(always)]
unsafe fn UserHeapAlloc(Bytes: usize) -> PVOID {
    RtlAllocateHeap(GlobalUserHeap, HEAP_NO_SERIALIZE, Bytes as SIZE_T)
}
#[inline(always)]
unsafe fn DesktopHeapAlloc(Desktop: PDESKTOP, Bytes: usize) -> PVOID {
    RtlAllocateHeap((*Desktop).pheapDesktop as PVOID, HEAP_NO_SERIALIZE, Bytes as SIZE_T)
}

// w_str! — ASCII string literal -> static NUL-terminated [u16]; yields PWSTR.
macro_rules! w_str {
    ($s:literal) => {{
        const N: usize = $s.len() + 1;
        const fn to_u16<const M: usize>(s: &str) -> [u16; M] {
            let b = s.as_bytes();
            let mut out = [0u16; M];
            let mut i = 0;
            while i < b.len() {
                out[i] = b[i] as u16;
                i += 1;
            }
            out
        }
        static W: [u16; N] = to_u16::<N>($s);
        W.as_ptr() as PWSTR
    }};
}

// Real cross-TU symbols: DestroyCallProc (callproc.rs, cdecl), GetW32ProcessInfo (cdecl).
extern "C" {
    fn DestroyCallProc(CallProc: PCALLPROCDATA) -> u8;
    fn GetW32ProcessInfo() -> PPROCESSINFO;
}
// DceFreeClassDCE is FASTCALL (`@DceFreeClassDCE@4`).
#[cfg(target_arch = "x86")]
extern "fastcall" {
    fn DceFreeClassDCE(pdce: *mut c_void);
}
#[cfg(not(target_arch = "x86"))]
extern "C" {
    fn DceFreeClassDCE(pdce: *mut c_void);
}

// `static __inline` heap frees (desktop.h / usrheap.h) — RtlFreeHeap wrappers.
#[inline(always)]
unsafe fn DesktopHeapFree(pDesk: PDESKTOP, lpMem: *mut c_void) -> u8 {
    RtlFreeHeap((*pDesk).pheapDesktop as PVOID, HEAP_NO_SERIALIZE, lpMem)
}
#[inline(always)]
unsafe fn UserHeapFree(lpMem: *mut c_void) -> u8 {
    RtlFreeHeap(GlobalUserHeap, HEAP_NO_SERIALIZE, lpMem)
}
// IsCallProcHandle (class.h): high 16 bits == 0xFFFF.
#[inline(always)]
unsafe fn IsCallProcHandle(lpWndProc: WNDPROC) -> BOOL {
    let v = lpWndProc.map(|f| f as usize).unwrap_or(0);
    ((v & 0xFFFF0000) == 0xFFFF0000) as BOOL
}
// UserHMGetHandle(obj) = ((obj)->head.h); head.h at object offset 0.
#[inline(always)]
unsafe fn UserHMGetHandle<T>(obj: *mut T) -> HANDLE {
    *(obj as *const HANDLE)
}
// GETPFNCLIENT{A,W}(fnid) = (WNDPROC)*((ULONG_PTR*)&gpsi->apfnClient{A,W} + (fnid-FNID_FIRST))
#[inline(always)]
unsafe fn GETPFNCLIENTA(fnid: INT) -> WNDPROC {
    // Per process: an x86-64 guest's user32 has its own procs (pfnclient.rs).
    core::mem::transmute(crate::pfnclient::client_proc(true, (fnid - FNID_FIRST as INT) as usize))
}
#[inline(always)]
unsafe fn GETPFNCLIENTW(fnid: INT) -> WNDPROC {
    core::mem::transmute(crate::pfnclient::client_proc(false, (fnid - FNID_FIRST as INT) as usize))
}
// GETPFNSERVER(fnid) = gpsi->aStoCidPfn[fnid - FNID_FIRST]
#[inline(always)]
unsafe fn GETPFNSERVER(fnid: INT) -> WNDPROC {
    // aStoCidPfn is filled from the W table, so a private W table answers it too.
    let i = (fnid - FNID_FIRST as INT) as usize;
    match crate::pfnclient::private_w(i) {
        Some(p) => core::mem::transmute(p),
        None => (*gpsi).aStoCidPfn[i],
    }
}

// GetW32ThreadInfo is a real exported cdecl symbol (`_GetW32ThreadInfo`).
extern "C" {
    fn GetW32ThreadInfo() -> PTHREADINFO;
}

// co_IntCopyImage returns a HANDLE; thin HICON-typed wrapper for class.c call sites.
#[inline(always)]
unsafe fn co_IntCopyImage_h(h: PVOID, ty: UINT, cx: INT, cy: INT, flags: UINT) -> HICON {
    co_IntCopyImage(h, ty, cx, cy, flags) as HICON
}

// MmUserProbeAddress — highest user-mode address + 1 (DATA export from ntoskrnl,
// ntoskrnl.spec `@ extern`). A plain extern reference is unsatisfiable for a data
// import (the import lib provides only __imp_, no thunk). Per-arch like console.rs's
// ExDesktopObjectType: i386/arm64 raw-dylib +verbatim (symbol IS the value); amd64
// binds __imp_ (ptr-to-value) + deref.
#[cfg(any(target_arch = "x86", target_arch = "aarch64"))]
#[link(name = "ntoskrnl.exe", kind = "raw-dylib", modifiers = "+verbatim")]
extern "C" {
    static MmUserProbeAddress: ULONG_PTR;
}
#[cfg(target_arch = "x86_64")]
extern "C" {
    #[link_name = "__imp_MmUserProbeAddress"]
    static MmUserProbeAddress_imp: *const ULONG_PTR;
}
#[cfg(any(target_arch = "x86", target_arch = "aarch64"))]
#[inline(always)]
unsafe fn mm_user_probe_address() -> usize {
    MmUserProbeAddress as usize
}
#[cfg(target_arch = "x86_64")]
#[inline(always)]
unsafe fn mm_user_probe_address() -> usize {
    *MmUserProbeAddress_imp as usize
}

// probe_read_checked(Ptr): range-check then fault-protected read, returning
// the caught NTSTATUS on failure instead of a default.
//
// RENAMED from the old `probe_read_generic` (and deliberately NOT reusing
// that name, which collides with `ntkrnl_sys::probe::probe_read_generic` --
// a DIFFERENT function with different semantics; the name clash is what hid
// this file's local shadow from an audit of the real one). This local
// version keeps the same MmUserProbeAddress-based range check the old one
// had (`mm_user_probe_address()`, declared above per-arch specifically
// because win32k.sys cannot link the DATA import `MmUserProbeAddress`
// directly -- see the comment on that helper), but is wrong in BOTH
// directions if left as-is:
//   - out-of-range used to silently return `default` where the C macro
//     RAISES (caught by the enclosing SEH, which then calls
//     SetLastNtError) -- so a bad pointer yielded a zeroed UNICODE_STRING
//     the caller then trusted as real data;
//   - in-range-but-unmapped used to `read_volatile` unconditionally, which
//     still faults (bugchecks under DUMMY_PSEH) on an unmapped page.
// Fixed by returning `Err(STATUS_ACCESS_VIOLATION)` on out-of-range instead
// of a default, and doing the actual read through
// `ntkrnl_sys::probe::safe_copy_raw` (hardware-fault-tolerant, catches the
// unmapped-page case the bare `read_volatile` could not).
#[inline(always)]
unsafe fn probe_read_checked<T: Copy>(ptr: *const T) -> Result<T, NTSTATUS> {
    let addr = ptr as usize;
    let end = addr.wrapping_add(core::mem::size_of::<T>().wrapping_sub(1));
    if end < addr || end >= mm_user_probe_address() {
        return Err(STATUS_ACCESS_VIOLATION);
    }
    let mut out: core::mem::MaybeUninit<T> = core::mem::MaybeUninit::uninit();
    ntkrnl_sys::probe::safe_copy_raw(
        out.as_mut_ptr() as *mut c_void,
        ptr as *const c_void,
        core::mem::size_of::<T>(),
    )?;
    // A fault may have copied a prefix; only a complete copy (Ok above)
    // initializes T, so this is sound.
    Ok(out.assume_init())
}

// ProbeForReadUnicodeString(Ptr) — captures a UNICODE_STRING by value.
#[inline(always)]
unsafe fn ProbeForReadUnicodeString(ptr: PUNICODE_STRING) -> Result<_UNICODE_STRING, NTSTATUS> {
    probe_read_checked(ptr as *const _UNICODE_STRING)
}

// ProbeForReadUint(Ptr) — captures a UINT by value.
#[inline(always)]
unsafe fn ProbeForReadUint(ptr: *const UINT) -> Result<UINT, NTSTATUS> {
    probe_read_checked(ptr)
}

// ProbeForWriteUnicodeString(Ptr) — write-probe a UNICODE_STRING.
//
// Uses SafeProbeForWrite (already imported at the top of this file — the
// real kernel routine, returns NTSTATUS) rather than the raw raising
// `ProbeForWrite` extern: this is the one caller of `ProbeForWrite` in the
// file, and inside a guarded region an uncaught raise still bugchecks
// under DUMMY_PSEH just like `ProbeForRead` would. A write probe is not a
// read probe, so this does NOT reuse `probe_read_checked` — SafeProbeForWrite
// is the write-shaped equivalent.
#[inline(always)]
unsafe fn ProbeForWriteUnicodeString(ptr: PUNICODE_STRING) -> Result<(), NTSTATUS> {
    let st = SafeProbeForWrite(
        ptr as *mut c_void,
        core::mem::size_of::<_UNICODE_STRING>() as SIZE_T,
        // probe.h ProbeForWriteGenericType checks range only, no alignment;
        // align_of (8 on 64-bit) would reject addresses the C accepts.
        1,
    );
    if st < 0 {
        Err(st)
    } else {
        Ok(())
    }
}

// RtlInitEmptyUnicodeString(Dst, Buffer, MaxLen).
#[inline(always)]
unsafe fn RtlInitEmptyUnicodeString(dst: PUNICODE_STRING, buffer: PWSTR, max_len: USHORT) {
    (*dst).Length = 0;
    (*dst).MaximumLength = max_len;
    (*dst).Buffer = buffer;
}

// RtlUnicodeStringToAnsiSize(String): single-byte-codepage branch of the C macro,
// (String->Length + sizeof(UNICODE_NULL)) / sizeof(WCHAR).
#[inline(always)]
unsafe fn RtlUnicodeStringToAnsiSize(string: PUNICODE_STRING) -> ULONG {
    (((*string).Length as ULONG) + core::mem::size_of::<u16>() as ULONG)
        / core::mem::size_of::<u16>() as ULONG
}

// ReadUnaligned/WriteUnaligned helpers (unaligned.h). On i386 ULONG_PTR is 32-bit.
#[inline(always)]
unsafe fn ReadUnalignedU32(p: PVOID) -> ULONG {
    core::ptr::read_unaligned(p as *const ULONG)
}
#[inline(always)]
unsafe fn WriteUnalignedU32(p: PVOID, val: ULONG) {
    core::ptr::write_unaligned(p as *mut ULONG, val);
}
#[inline(always)]
unsafe fn ReadUnalignedUlongPtr(p: PVOID) -> ULONG_PTR {
    core::ptr::read_unaligned(p as *const ULONG_PTR)
}
#[inline(always)]
unsafe fn WriteUnalignedUlongPtr(p: PVOID, val: ULONG_PTR) {
    core::ptr::write_unaligned(p as *mut ULONG_PTR, val);
}

// UserHeapAddressToUser (usrheap.h): translate kernel heap addr to its user mapping.
#[inline(always)]
unsafe fn UserHeapAddressToUser(lpMem: PVOID) -> PVOID {
    let w32 = PsGetCurrentProcessWin32Process() as PPROCESSINFO;
    ((lpMem as usize)
        .wrapping_sub(GlobalUserHeap as usize)
        .wrapping_add((*w32).HeapMappings.UserMapping as usize)) as PVOID
}

// W32PF_flags lives in the flattened W32PROCESS base of PROCESSINFO.
// Layout: peProcess(ptr) + RefCount(u32) => W32PF_flags at byte offset
// (size_of::<PVOID>() + 4), i.e. __bindgen_padding_0[1] on i386, [2] otherwise.
#[inline(always)]
unsafe fn ppi_w32pf_flags(ppi: PPROCESSINFO) -> *mut ULONG {
    let off = core::mem::size_of::<PVOID>() + core::mem::size_of::<DWORD>();
    (ppi as *mut u8).add(off) as *mut ULONG
}

// THROBJHEAD.pti: bindgen flattens HEAD{h, cLockObj} + pti into __bindgen_padding_0.
// pti is a pointer following HEAD{HANDLE, DWORD}; on 64-bit there are 4 bytes of
// tail padding so pti is at offset 16 (amd64/arm64) / 8 (i386). HANDLE+DWORD (=12 on
// 64-bit) MISSES the padding → garbage pti → AV. Use 2*usize = 16/8 correctly per
// arch (matches event.rs/timer.rs). [latent amd64/arm64 offset bug fix]
#[inline(always)]
unsafe fn head_pti(head: *const _THRDESKHEAD) -> PTHREADINFO {
    let off = 2 * core::mem::size_of::<usize>();
    *((head as *const u8).add(off) as *const PTHREADINFO)
}

// SYSTEMCUR(ARROW) = gasyscur[ROCR_ARROW].handle.
#[inline(always)]
unsafe fn SYSTEMCUR_ARROW() -> PCURICON_OBJECT {
    let base = core::ptr::addr_of!(gasyscur) as *const SYSTEMCURICO;
    (*base.add(ROCR_ARROW as usize)).handle
}

// ARRAYSIZE for the DefaultServerClasses table (6 active entries).
const DEFAULT_SERVER_CLASSES_COUNT: u32 = 6;

// REGISTER_SYSCLASS table of system classes registered by win32k (class.c).
// 6 active entries (the Tooltips entry is #if 0'd out in the C source).
#[repr(transparent)]
struct SysClassTable([REGISTER_SYSCLASS; DEFAULT_SERVER_CLASSES_COUNT as usize]);
unsafe impl Sync for SysClassTable {}
impl core::ops::Index<usize> for SysClassTable {
    type Output = REGISTER_SYSCLASS;
    #[inline(always)]
    fn index(&self, i: usize) -> &REGISTER_SYSCLASS {
        &self.0[i]
    }
}

#[allow(non_upper_case_globals)]
static DefaultServerClasses: SysClassTable = SysClassTable([
    // WC_DESKTOP (MAKEINTATOM(0x8001))
    REGISTER_SYSCLASS {
        ClassName: 0x8001 as PWSTR,
        Style: CS_GLOBALCLASS | CS_DBLCLKS,
        ProcW: None,
        ExtraBytes: (core::mem::size_of::<ULONG>() * 2) as UINT,
        hCursor: OCR_NORMAL as HICON,
        hBrush: COLOR_BACKGROUND as HBRUSH,
        fiId: FNID_DESKTOP as WORD,
        iCls: ICLS_DESKTOP as WORD,
    },
    // WC_SWITCH (MAKEINTATOM(0x8003))
    REGISTER_SYSCLASS {
        ClassName: 0x8003 as PWSTR,
        Style: CS_VREDRAW | CS_HREDRAW | CS_SAVEBITS,
        ProcW: None,
        ExtraBytes: core::mem::size_of::<LONG_PTR>() as UINT,
        hCursor: OCR_NORMAL as HICON,
        hBrush: core::ptr::null_mut(),
        fiId: FNID_SWITCH as WORD,
        iCls: ICLS_SWITCH as WORD,
    },
    // WC_MENU (MAKEINTATOM(0x8000))
    REGISTER_SYSCLASS {
        ClassName: 0x8000 as PWSTR,
        Style: CS_DBLCLKS | CS_SAVEBITS | CS_DROPSHADOW,
        ProcW: None,
        ExtraBytes: 16,
        hCursor: OCR_NORMAL as HICON,
        hBrush: (COLOR_MENU + 1) as HBRUSH,
        fiId: FNID_MENU as WORD,
        iCls: ICLS_MENU as WORD,
    },
    // "ScrollBar"
    REGISTER_SYSCLASS {
        ClassName: w_str!("ScrollBar"),
        Style: CS_DBLCLKS | CS_VREDRAW | CS_HREDRAW | CS_PARENTDC,
        ProcW: None,
        ExtraBytes: (core::mem::size_of::<SBWND>() - core::mem::size_of::<WND>()) as UINT,
        hCursor: OCR_NORMAL as HICON,
        hBrush: core::ptr::null_mut(),
        fiId: FNID_SCROLLBAR as WORD,
        iCls: ICLS_SCROLLBAR as WORD,
    },
    // WC_ICONTITLE (MAKEINTATOM(0x8004))
    REGISTER_SYSCLASS {
        ClassName: 0x8004 as PWSTR,
        Style: 0,
        ProcW: None,
        ExtraBytes: 0,
        hCursor: OCR_NORMAL as HICON,
        hBrush: core::ptr::null_mut(),
        fiId: FNID_ICONTITLE as WORD,
        iCls: ICLS_ICONTITLE as WORD,
    },
    // "Message"
    REGISTER_SYSCLASS {
        ClassName: w_str!("Message"),
        Style: CS_GLOBALCLASS,
        ProcW: None,
        ExtraBytes: 0,
        hCursor: OCR_NORMAL as HICON,
        hBrush: core::ptr::null_mut(),
        fiId: FNID_MESSAGEWND as WORD,
        iCls: ICLS_HWNDMESSAGE as WORD,
    },
]);

// ProbeAndCaptureUnicodeStringOrAtom (class.c) — NTAPI; probes a user UNICODE_STRING
// or recognizes an atom, capturing into a PagedPool buffer (TAG_STRING).
#[no_mangle]
pub unsafe extern "system" fn ProbeAndCaptureUnicodeStringOrAtom(
    pustrOut: PUNICODE_STRING,
    pustrUnsafe: PUNICODE_STRING,
) -> NTSTATUS {
    let mut status: NTSTATUS = STATUS_SUCCESS;
    let ustr_copy: _UNICODE_STRING;

    /* Default to NULL */
    RtlInitEmptyUnicodeString(pustrOut, core::ptr::null_mut(), 0);

    // ---- dropped _SEH2_TRY/_SEH2_EXCEPT recovered here (class.c:204) ----
    // Shape: "free pustrOut->Buffer if already allocated, then cleanup" --
    // the handler only sets `status` to the caught code if a buffer was
    // already allocated at fault time (faithfully replicated below: if the
    // FIRST probe/capture faults, pustrOut->Buffer is still NULL, so the C
    // handler's `if (pustrOut->Buffer)` is false and `status` stays
    // STATUS_SUCCESS despite the exception -- an existing C oddity, kept
    // as-is rather than "fixed").
    'seh: {
        // ProbeForRead(pustrUnsafe, sizeof(UNICODE_STRING), 1) + `ustr_copy = *pustrUnsafe`
        // -- exactly probe_read_checked's shape (probe then capture a T by value).
        ustr_copy = match probe_read_checked(pustrUnsafe as *const _UNICODE_STRING) {
            Ok(v) => v,
            Err(st) => {
                if !(*pustrOut).Buffer.is_null() {
                    ExFreePoolWithTag((*pustrOut).Buffer as *mut c_void, TAG_STRING);
                    status = st;
                }
                break 'seh;
            }
        };

        /* Validate the string */
        if (ustr_copy.Length & 1) != 0 || ustr_copy.Buffer.is_null() {
            return STATUS_INVALID_PARAMETER;
        }

        /* Check if this is an atom */
        if IS_ATOM!(ustr_copy.Buffer) != 0 {
            /* Copy the atom, length is 0 */
            (*pustrOut).MaximumLength = 0;
            (*pustrOut).Length = 0;
            (*pustrOut).Buffer = ustr_copy.Buffer;
        } else {
            /* Get the length, maximum length includes zero termination */
            (*pustrOut).Length = ustr_copy.Length;
            (*pustrOut).MaximumLength = ustr_copy.Length + core::mem::size_of::<u16>() as USHORT;

            /* Allocate a buffer */
            (*pustrOut).Buffer = ExAllocatePoolWithTag(
                PagedPool,
                (*pustrOut).MaximumLength as SIZE_T,
                TAG_STRING,
            ) as PWSTR;
            if (*pustrOut).Buffer.is_null() {
                return STATUS_INSUFFICIENT_RESOURCES;
            }

            /* Copy the string and zero terminate it */
            // ProbeForRead(ustr_copy.Buffer, pustrOut->Length, 1) + RtlCopyMemory.
            // The range check is NOT optional: safe_copy_raw alone would happily
            // copy from a kernel address the user named.
            if let Err(st) = crate::safe_user_io::copy_from_user(
                (*pustrOut).Buffer as *mut c_void,
                ustr_copy.Buffer as *const c_void,
                (*pustrOut).Length as usize,
                1,
            ) {
                // pustrOut->Buffer is allocated at this point, so (unlike the
                // first probe above) the handler's `if (pustrOut->Buffer)`
                // is true here: free it and record the status.
                ExFreePoolWithTag((*pustrOut).Buffer as *mut c_void, TAG_STRING);
                status = st;
                break 'seh;
            }
            *(*pustrOut).Buffer.add((*pustrOut).Length as usize / core::mem::size_of::<u16>()) =
                UNICODE_NULL;
        }

        break 'seh;
    }

    status
}

// IntFreeClassMenuName (class.c) — free a changed/allocated menu name.
unsafe fn IntFreeClassMenuName(Class: PCLS) {
    if !(*Class).lpszClientUnicodeMenuName.is_null() && (*Class).MenuNameIsString() != 0 {
        UserHeapFree((*Class).lpszClientUnicodeMenuName as *mut c_void);
        (*Class).lpszClientUnicodeMenuName = core::ptr::null_mut();
        (*Class).lpszClientAnsiMenuName = core::ptr::null_mut();
    }
}

// ---- file-scope preamble (translated) ----
// PWSTR (*mut u16) is not Sync; wrap the static array in a Sync newtype.
struct ControlsListTable([PWSTR; 17]);
unsafe impl Sync for ControlsListTable {}
impl core::ops::Index<usize> for ControlsListTable {
    type Output = PWSTR;
    #[inline(always)]
    fn index(&self, i: usize) -> &PWSTR {
        &self.0[i]
    }
}
#[allow(non_upper_case_globals)]
pub static ControlsList: ControlsListTable = ControlsListTable([
    w_str!("Button"),
    w_str!("Edit"),
    w_str!("Static"),
    w_str!("ListBox"),
    w_str!("ScrollBar"),
    w_str!("ComboBox"),
    w_str!("MDIClient"),
    w_str!("ComboLBox"),
    w_str!("DDEMLEvent"),
    w_str!("DDEMLMom"),
    w_str!("DMGClass"),
    w_str!("DDEMLAnsiClient"),
    w_str!("DDEMLUnicodeClient"),
    w_str!("DDEMLAnsiServer"),
    w_str!("DDEMLUnicodeServer"),
    w_str!("IME"),
    w_str!("Ghost"),
]);

#[repr(C)]
pub struct FnidToiClsEntry {
    pub FnId: core::ffi::c_int,
    pub ClsId: core::ffi::c_int,
}

pub static FnidToiCls: [FnidToiClsEntry; 17] =
[
    FnidToiClsEntry { FnId: FNID_SCROLLBAR as core::ffi::c_int,  ClsId: ICLS_SCROLLBAR as core::ffi::c_int },
    FnidToiClsEntry { FnId: FNID_ICONTITLE as core::ffi::c_int,  ClsId: ICLS_ICONTITLE as core::ffi::c_int },
    FnidToiClsEntry { FnId: FNID_MENU as core::ffi::c_int,       ClsId: ICLS_MENU as core::ffi::c_int },
    FnidToiClsEntry { FnId: FNID_DESKTOP as core::ffi::c_int,    ClsId: ICLS_DESKTOP as core::ffi::c_int },
    FnidToiClsEntry { FnId: FNID_SWITCH as core::ffi::c_int,     ClsId: ICLS_SWITCH as core::ffi::c_int },
    FnidToiClsEntry { FnId: FNID_MESSAGEWND as core::ffi::c_int, ClsId: ICLS_HWNDMESSAGE as core::ffi::c_int },
    FnidToiClsEntry { FnId: FNID_BUTTON as core::ffi::c_int,     ClsId: ICLS_BUTTON as core::ffi::c_int },
    FnidToiClsEntry { FnId: FNID_COMBOBOX as core::ffi::c_int,   ClsId: ICLS_COMBOBOX as core::ffi::c_int },
    FnidToiClsEntry { FnId: FNID_COMBOLBOX as core::ffi::c_int,  ClsId: ICLS_COMBOLBOX as core::ffi::c_int },
    FnidToiClsEntry { FnId: FNID_DIALOG as core::ffi::c_int,     ClsId: ICLS_DIALOG as core::ffi::c_int },
    FnidToiClsEntry { FnId: FNID_EDIT as core::ffi::c_int,       ClsId: ICLS_EDIT as core::ffi::c_int },
    FnidToiClsEntry { FnId: FNID_LISTBOX as core::ffi::c_int,    ClsId: ICLS_LISTBOX as core::ffi::c_int },
    FnidToiClsEntry { FnId: FNID_MDICLIENT as core::ffi::c_int,  ClsId: ICLS_MDICLIENT as core::ffi::c_int },
    FnidToiClsEntry { FnId: FNID_STATIC as core::ffi::c_int,     ClsId: ICLS_STATIC as core::ffi::c_int },
    FnidToiClsEntry { FnId: FNID_IME as core::ffi::c_int,        ClsId: ICLS_IME as core::ffi::c_int },
    FnidToiClsEntry { FnId: FNID_GHOST as core::ffi::c_int,      ClsId: ICLS_GHOST as core::ffi::c_int },
    FnidToiClsEntry { FnId: FNID_TOOLTIPS as core::ffi::c_int,   ClsId: ICLS_TOOLTIPS as core::ffi::c_int },
];

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn LookupFnIdToiCls(FnId: i32, iCls: *mut i32) -> BOOL {
    LookupFnIdToiCls_impl(FnId, iCls)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn LookupFnIdToiCls(FnId: i32, iCls: *mut i32) -> BOOL {
    LookupFnIdToiCls_impl(FnId, iCls)
}

unsafe fn LookupFnIdToiCls_impl(FnId: i32, iCls: *mut i32) -> BOOL {
    let count = FnidToiCls.len();
    let mut i = 0;
    while i < count {
        if FnidToiCls[i].FnId == FnId {
            if !iCls.is_null() {
                *iCls = FnidToiCls[i].ClsId;
            }
            return 1;
        }
        i += 1;
    }
    if !iCls.is_null() {
        *iCls = 0;
    }
    0
}

unsafe fn IntDestroyClass(Class: PCLS) {
    let pDesk: PDESKTOP;

    if (*Class).pclsBase == Class {
        let mut CallProc: PCALLPROCDATA;
        let mut NextCallProc: PCALLPROCDATA;

        CallProc = (*Class).spcpdFirst;
        while CallProc != core::ptr::null_mut() {
            NextCallProc = (*CallProc).spcpdNext;

            (*CallProc).spcpdNext = core::ptr::null_mut();
            DestroyCallProc(CallProc);

            CallProc = NextCallProc;
        }

        if (*Class).atomClassName != 0 {
            IntDeregisterClassAtom((*Class).atomClassName);
        }
        if (*Class).atomNVClassName != 0 {
            IntDeregisterClassAtom((*Class).atomNVClassName);
        }

        if (*Class).pdce != core::ptr::null_mut() {
            DceFreeClassDCE((*Class).pdce);
            (*Class).pdce = core::ptr::null_mut();
        }

        IntFreeClassMenuName(Class);
    }

    if (*Class).spicn != core::ptr::null_mut() {
        UserDereferenceObject((*Class).spicn as *mut core::ffi::c_void);
    }
    if (*Class).spcur != core::ptr::null_mut() {
        UserDereferenceObject((*Class).spcur as *mut core::ffi::c_void);
    }
    if (*Class).spicnSm != core::ptr::null_mut() {
        UserDereferenceObject((*Class).spicnSm as *mut core::ffi::c_void);
        if ((*Class).CSF_flags as u32 & CSF_CACHEDSMICON) != 0
            && UserObjectInDestroy(UserHMGetHandle((*Class).spicnSm)) == 0
        {
            IntDestroyCurIconObject((*Class).spicnSm as _);
        }
    }

    pDesk = (*Class).rpdeskParent;
    (*Class).rpdeskParent = core::ptr::null_mut();

    if pDesk != core::ptr::null_mut() {
        DesktopHeapFree(pDesk, Class as *mut core::ffi::c_void);
    } else {
        UserHeapFree(Class as *mut core::ffi::c_void);
    }
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn DestroyProcessClasses(Process: PPROCESSINFO) {
    DestroyProcessClasses_impl(Process)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn DestroyProcessClasses(Process: PPROCESSINFO) {
    DestroyProcessClasses_impl(Process)
}

unsafe fn DestroyProcessClasses_impl(Process: PPROCESSINFO) {
    let mut Class: PCLS;
    let pi: PPROCESSINFO = Process;

    if pi != core::ptr::null_mut() {
        Class = (*pi).pclsPrivateList;
        while Class != core::ptr::null_mut() {
            (*pi).pclsPrivateList = (*Class).pclsNext;

            IntDestroyClass(Class);

            Class = (*pi).pclsPrivateList;
        }

        Class = (*pi).pclsPublicList;
        while Class != core::ptr::null_mut() {
            (*pi).pclsPublicList = (*Class).pclsNext;

            IntDestroyClass(Class);

            Class = (*pi).pclsPublicList;
        }
    }
}

unsafe fn IntRegisterClassAtom(ClassName: PUNICODE_STRING, pAtom: *mut RTL_ATOM) -> BOOL {
    let mut szBuf: [u16; 65] = [0; 65];
    let mut AtomName: PWSTR = szBuf.as_mut_ptr();
    let Status: NTSTATUS;

    if (*ClassName).Length != 0 {
        if (*ClassName).Length as usize + core::mem::size_of::<u16>() > core::mem::size_of_val(&szBuf) {
            AtomName = ExAllocatePoolWithTag(PagedPool,
                                             ((*ClassName).Length as usize + core::mem::size_of::<u16>()) as SIZE_T,
                                             TAG_USTR) as PWSTR;

            if AtomName.is_null() {
                EngSetLastError(ERROR_OUTOFMEMORY);
                return 0;
            }
        }

        // ---- dropped _SEH2_TRY/_SEH2_EXCEPT recovered here (class.c:368) ----
        // No ProbeForRead in the C at this site (rule: don't add one where
        // the C has none) -- RtlCopyMemory alone was the guarded region, so
        // safe_copy_raw (fault-tolerant copy, no probing) is the direct match.
        // Handler shape: free AtomName if not the stack buffer, SetLastNtError,
        // then an immediate `_SEH2_YIELD(return FALSE)` -- not a fall-through.
        if let Err(st) = ntkrnl_sys::probe::safe_copy_raw(
            AtomName as *mut core::ffi::c_void,
            (*ClassName).Buffer as *const core::ffi::c_void,
            (*ClassName).Length as usize,
        ) {
            if AtomName != szBuf.as_mut_ptr() {
                ExFreePoolWithTag(AtomName as *mut core::ffi::c_void, TAG_USTR);
            }
            SetLastNtError(st);
            return 0;
        }

        *AtomName.add((*ClassName).Length as usize / core::mem::size_of::<u16>()) = 0;
    } else {
        AtomName = (*ClassName).Buffer;
    }

    Status = RtlAddAtomToAtomTable(gAtomTable as _, AtomName, pAtom);

    if AtomName != (*ClassName).Buffer && AtomName != szBuf.as_mut_ptr() {
        ExFreePoolWithTag(AtomName as *mut core::ffi::c_void, TAG_USTR);
    }

    if !NT_SUCCESS(Status) {
        SetLastNtError(Status);
        return 0;
    }

    1
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn RegisterControlAtoms() -> BOOL {
    RegisterControlAtoms_impl()
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn RegisterControlAtoms() -> BOOL {
    RegisterControlAtoms_impl()
}

unsafe fn RegisterControlAtoms_impl() -> BOOL {
    let mut Atom: RTL_ATOM = 0;
    let mut ClassName: _UNICODE_STRING = core::mem::zeroed();
    let mut i: INT = 0;

    while i < ICLS_DESKTOP as INT {
        RtlInitUnicodeString(core::ptr::addr_of_mut!(ClassName) as _, ControlsList[i as usize]);
        if IntRegisterClassAtom(&mut ClassName, &mut Atom) != 0 {
            (*gpsi).atomSysClass[i as usize] = Atom;
            TRACE!(b"Reg Control Atom %ls: 0x%x\n\0".as_ptr() as *const i8,
                  ControlsList[i as usize], Atom);
        }
        i += 1;
    }
    1
}

unsafe fn IntDeregisterClassAtom(Atom: RTL_ATOM) -> NTSTATUS {
    RtlDeleteAtomFromAtomTable(gAtomTable as _, Atom)
}

#[no_mangle]
pub unsafe extern "C" fn UserAddCallProcToClass(mut Class: PCLS, CallProc: PCALLPROCDATA) {
    let BaseClass: PCLS = (*Class).pclsBase;
    (*CallProc).spcpdNext = (*BaseClass).spcpdFirst;
    (*BaseClass).spcpdFirst = CallProc;

    Class = (*Class).pclsClone;
    while !Class.is_null() {
        (*Class).spcpdFirst = (*BaseClass).spcpdFirst;
        Class = (*Class).pclsNext;
    }
}

unsafe fn IntSetClassAtom(mut Class: PCLS, ClassName: PUNICODE_STRING) -> BOOL {
    let mut Atom: RTL_ATOM = 0;

    Class = (*Class).pclsBase;
    if (*ClassName).Length > 0 {
        if IntRegisterClassAtom(ClassName, &mut Atom) == 0 {
            return 0;
        }
    } else {
        if IS_ATOM!((*ClassName).Buffer) != 0 {
            Atom = ((*ClassName).Buffer as ULONG_PTR & 0xffff) as RTL_ATOM;
        } else {
            EngSetLastError(ERROR_INVALID_PARAMETER);
            return 0;
        }
    }

    IntDeregisterClassAtom((*Class).atomNVClassName);

    (*Class).atomNVClassName = Atom;

    Class = (*Class).pclsClone;
    while !Class.is_null() {
        (*Class).atomNVClassName = Atom;

        Class = (*Class).pclsNext;
    }

    1
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn IntGetClassWndProc(Class: PCLS, Ansi: BOOL) -> WNDPROC {
    IntGetClassWndProc_impl(Class, Ansi)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn IntGetClassWndProc(Class: PCLS, Ansi: BOOL) -> WNDPROC {
    IntGetClassWndProc_impl(Class, Ansi)
}

#[allow(unpredictable_function_pointer_comparisons)] // FNPTR-CMP(audited): intentional identity compare mirroring the C source; revisit -> core::ptr::fn_addr_eq (see docs/engineering/active-lanes/warnings-burndown-plan.md)
unsafe fn IntGetClassWndProc_impl(Class: PCLS, Ansi: BOOL) -> WNDPROC {
    let mut i: INT;
    let mut gcpd: WNDPROC = None;
    let mut Ret: WNDPROC = None;

    if ((*Class).CSF_flags as u32 & CSF_SERVERSIDEPROC) != 0 {
        i = FNID_FIRST as INT;
        while i <= FNID_SWITCH as INT {
            if GETPFNSERVER(i) == (*Class).lpfnWndProc {
                if Ansi != 0 {
                    Ret = GETPFNCLIENTA(i);
                } else {
                    Ret = GETPFNCLIENTW(i);
                }
            }
            i += 1;
        }
        return Ret;
    }
    Ret = (*Class).lpfnWndProc;

    if (*Class).fnid <= FNID_GHOST as _ && (*Class).fnid >= FNID_BUTTON as _ {
        if Ansi != 0 {
            if GETPFNCLIENTW((*Class).fnid as INT) == (*Class).lpfnWndProc {
                Ret = GETPFNCLIENTA((*Class).fnid as INT);
            }
        } else {
            if GETPFNCLIENTA((*Class).fnid as INT) == (*Class).lpfnWndProc {
                Ret = GETPFNCLIENTW((*Class).fnid as INT);
            }
        }
    }

    if Ret != (*Class).lpfnWndProc
        || (Ansi != 0) == (((*Class).CSF_flags as u32 & CSF_ANSIPROC) != 0)
    {
        return Ret;
    }

    gcpd = core::mem::transmute(UserGetCPD(
        Class as _,
        (if Ansi != 0 { UserGetCPDA2U } else { UserGetCPDU2A }) | UserGetCPDClass,
        core::mem::transmute::<_, ULONG_PTR>(Ret),
    ));

    if gcpd.is_some() { gcpd } else { Ret }
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn IntSetClassWndProc(Class: PCLS, WndProc: WNDPROC, Ansi: BOOL) -> WNDPROC {
    IntSetClassWndProc_impl(Class, WndProc, Ansi)
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn IntSetClassWndProc(Class: PCLS, WndProc: WNDPROC, Ansi: BOOL) -> WNDPROC {
    IntSetClassWndProc_impl(Class, WndProc, Ansi)
}

#[allow(unpredictable_function_pointer_comparisons)] // FNPTR-CMP(audited): intentional identity compare mirroring the C source; revisit -> core::ptr::fn_addr_eq (see docs/engineering/active-lanes/warnings-burndown-plan.md)
unsafe fn IntSetClassWndProc_impl(mut Class: PCLS, mut WndProc: WNDPROC, Ansi: BOOL) -> WNDPROC {
    let mut i: INT;
    let pcpd: PCALLPROCDATA;
    let Ret: WNDPROC;
    let mut chWndProc: WNDPROC;

    Ret = IntGetClassWndProc(Class, Ansi);

    if (*Class).CSF_flags & (CSF_SERVERSIDEPROC as u32) != 0 {
        if Ansi != 0 {
            (*Class).CSF_flags |= CSF_ANSIPROC as u32;
        }
        (*Class).CSF_flags &= !(CSF_SERVERSIDEPROC as u32);
        (*Class).set_Unicode(((Ansi == 0) as u8) as UINT);
    }

    if WndProc.is_none() {
        WndProc = (*Class).lpfnWndProc;
    }

    chWndProc = WndProc;

    if IsCallProcHandle(WndProc) != 0 {
        pcpd = UserGetObject(gHandleTable, core::mem::transmute(WndProc), TYPE_CALLPROC as u32) as PCALLPROCDATA;
        if !pcpd.is_null() {
            chWndProc = (*pcpd).pfnClientPrevious;
        }
    }

    (*Class).lpfnWndProc = chWndProc;

    chWndProc = None;

    i = FNID_FIRST as i32;
    while i <= FNID_SWITCH as i32 {
        if GETPFNCLIENTW(i) == (*Class).lpfnWndProc {
            chWndProc = GETPFNSERVER(i);
            break;
        }
        if GETPFNCLIENTA(i) == (*Class).lpfnWndProc {
            chWndProc = GETPFNSERVER(i);
            break;
        }
        i += 1;
    }

    if chWndProc.is_some() {
        (*Class).lpfnWndProc = chWndProc;
        (*Class).set_Unicode(1);
        (*Class).CSF_flags &= !(CSF_ANSIPROC as u32);
        (*Class).CSF_flags |= CSF_SERVERSIDEPROC as u32;
    } else {
        (*Class).set_Unicode(((Ansi == 0) as u8) as UINT);

        if Ansi != 0 {
            (*Class).CSF_flags |= CSF_ANSIPROC as u32;
        } else {
            (*Class).CSF_flags &= !(CSF_ANSIPROC as u32);
        }
    }

    chWndProc = (*Class).lpfnWndProc;

    Class = (*Class).pclsClone;
    while !Class.is_null() {
        (*Class).set_Unicode(((Ansi == 0) as u8) as UINT);
        (*Class).lpfnWndProc = chWndProc;

        Class = (*Class).pclsNext;
    }

    Ret
}

unsafe fn IntGetClassForDesktop(BaseClass: PCLS, ClassLink: *mut PCLS, Desktop: PDESKTOP) -> PCLS {
    let ClassSize: SIZE_T;
    let mut Class: PCLS;

    if (*BaseClass).rpdeskParent == Desktop {
        return BaseClass;
    }

    if (*BaseClass).rpdeskParent == core::ptr::null_mut() {
        Class = core::ptr::null_mut();
    } else {
        Class = (*BaseClass).pclsClone;
        while Class != core::ptr::null_mut() {
            if (*Class).rpdeskParent == Desktop {
                break;
            }

            Class = (*Class).pclsNext;
        }
    }

    if Class == core::ptr::null_mut() {
        ClassSize = (core::mem::size_of::<_CLS>() as SIZE_T) + ((*BaseClass).cbclsExtra as SIZE_T);

        Class = DesktopHeapAlloc(Desktop, ClassSize as usize) as PCLS;

        if Class != core::ptr::null_mut() {
            RtlCopyMemory(Class as PVOID, BaseClass as PVOID, ClassSize as usize);

            if (*Class).spcur != core::ptr::null_mut() {
                UserReferenceObject((*Class).spcur as PVOID);
            }
            if (*Class).spicn != core::ptr::null_mut() {
                UserReferenceObject((*Class).spicn as PVOID);
            }
            if (*Class).spicnSm != core::ptr::null_mut() {
                UserReferenceObject((*Class).spicnSm as PVOID);
            }

            TRACE!(b"Clone Class 0x%p hM 0x%p\n %S\n\0".as_ptr() as PCHAR, Class, (*Class).hModule, (*Class).lpszClientUnicodeMenuName);

            if (*Class).hModule != hModClient &&
               (*Class).fnid <= FNID_GHOST as u32 &&
               (*Class).fnid >= FNID_BUTTON as u32 {
                (*Class).hModule = hModClient;
                TRACE!(b"Clone Class 0x%p Reset hM 0x%p\n\0".as_ptr() as PCHAR, Class, (*Class).hModule);
            }

            (*Class).rpdeskParent = Desktop;
            (*Class).cWndReferenceCount = 0;

            if (*BaseClass).rpdeskParent == core::ptr::null_mut() {
                (*Class).pclsBase = Class;
                (*Class).pclsNext = (*BaseClass).pclsNext;

                InterlockedExchangePointer(ClassLink as *mut PVOID, Class as PVOID);

                (*BaseClass).pclsBase = core::ptr::null_mut();
                (*BaseClass).pclsClone = core::ptr::null_mut();
                IntDestroyClass(BaseClass);
            } else {
                (*Class).pclsClone = core::ptr::null_mut();
                (*Class).pclsBase = BaseClass;
                (*Class).pclsNext = (*BaseClass).pclsClone;
                InterlockedExchangePointer(core::ptr::addr_of_mut!((*BaseClass).pclsClone) as *mut PVOID, Class as PVOID);
            }
        } else {
            EngSetLastError(ERROR_NOT_ENOUGH_MEMORY);
        }
    }
    Class
}

#[no_mangle]
pub unsafe extern "C" fn IntReferenceClass(BaseClass: PCLS, ClassLink: *mut PCLS, Desktop: PDESKTOP) -> PCLS {
    let Class: PCLS;

    if Desktop != core::ptr::null_mut() {
        Class = IntGetClassForDesktop(BaseClass, ClassLink, Desktop);
    } else {
        Class = BaseClass;
    }

    if Class != core::ptr::null_mut() {
        (*Class).cWndReferenceCount += 1;
    }

    Class
}

unsafe fn IntMakeCloneBaseClass(Class: PCLS, BaseClassLink: *mut PCLS, CloneLink: *mut PCLS)
{
    let mut Clone: PCLS;

    /* Unlink the clone */
    *CloneLink = (*Class).pclsNext;
    (*Class).pclsClone = (*(*Class).pclsBase).pclsClone;

    /* Update the class information to make it a base class */
    (*Class).pclsBase = Class;
    (*Class).pclsNext = (**BaseClassLink).pclsNext;

    /* Update all clones */
    Clone = (*Class).pclsClone;
    while !Clone.is_null()
    {
        (*Clone).pclsBase = Class;

        Clone = (*Clone).pclsNext;
    }

    /* Link in the new base class */
    let _ = InterlockedExchangePointer(BaseClassLink as *mut *mut core::ffi::c_void,
                                       Class as *mut core::ffi::c_void);
}

#[no_mangle]
pub unsafe extern "C" fn IntDereferenceClass(Class: PCLS, Desktop: PDESKTOPINFO, pi: PPROCESSINFO) {
    let mut PrevLink: *mut PCLS;
    let BaseClass: PCLS;
    let mut CurrentClass: PCLS;

    BaseClass = (*Class).pclsBase;

    (*Class).cWndReferenceCount -= 1;
    if (*Class).cWndReferenceCount == 0 {
        if BaseClass == Class {
            if (*BaseClass).pclsClone != core::ptr::null_mut() {
                if (*BaseClass).Global() != 0 {
                    PrevLink = core::ptr::addr_of_mut!((*pi).pclsPublicList);
                } else {
                    PrevLink = core::ptr::addr_of_mut!((*pi).pclsPrivateList);
                }

                CurrentClass = *PrevLink;
                while CurrentClass != BaseClass {
                    PrevLink = core::ptr::addr_of_mut!((*CurrentClass).pclsNext);
                    CurrentClass = (*CurrentClass).pclsNext;
                }

                IntMakeCloneBaseClass((*BaseClass).pclsClone,
                                      PrevLink,
                                      core::ptr::addr_of_mut!((*BaseClass).pclsClone));

                (*BaseClass).pclsBase = core::ptr::null_mut();
                (*BaseClass).pclsClone = core::ptr::null_mut();
                IntDestroyClass(BaseClass);
            }
        } else {
            PrevLink = core::ptr::addr_of_mut!((*BaseClass).pclsClone);
            CurrentClass = (*BaseClass).pclsClone;
            while CurrentClass != Class {
                PrevLink = core::ptr::addr_of_mut!((*CurrentClass).pclsNext);
                CurrentClass = (*CurrentClass).pclsNext;
            }

            InterlockedExchangePointer(PrevLink as *mut PVOID,
                                       (*Class).pclsNext as PVOID);

            IntDestroyClass(Class);
        }
    }
}

unsafe fn IntMoveClassToSharedHeap(Class: PCLS, ClassLinkPtr: *mut *mut PCLS) -> BOOL {
    let NewClass: PCLS;
    let ClassSize: SIZE_T;

    ClassSize = core::mem::size_of::<_CLS>() as SIZE_T + (*Class).cbclsExtra as SIZE_T;

    NewClass = UserHeapAlloc(ClassSize as usize) as PCLS;
    if NewClass != core::ptr::null_mut() {
        RtlCopyMemory(NewClass as PVOID, Class as PVOID, ClassSize as usize);

        (*NewClass).rpdeskParent = core::ptr::null_mut();
        (*NewClass).pclsBase = NewClass;

        if (*NewClass).spcur != core::ptr::null_mut() {
            UserReferenceObject((*NewClass).spcur as PVOID);
        }
        if (*NewClass).spicn != core::ptr::null_mut() {
            UserReferenceObject((*NewClass).spicn as PVOID);
        }
        if (*NewClass).spicnSm != core::ptr::null_mut() {
            UserReferenceObject((*NewClass).spicnSm as PVOID);
        }

        InterlockedExchangePointer(*ClassLinkPtr as *mut PVOID, NewClass as PVOID);
        *ClassLinkPtr = core::ptr::addr_of_mut!((*NewClass).pclsNext);

        (*Class).pclsBase = core::ptr::null_mut();
        IntDestroyClass(Class);
        return 1;
    }

    0
}

unsafe fn IntCheckDesktopClasses(
    Desktop: PDESKTOP,
    ClassList: *mut PCLS,
    FreeOnFailure: BOOL,
    Ret: *mut BOOL,
) {
    let mut Class: PCLS;
    let mut NextClass: PCLS;
    let mut Link: *mut PCLS;

    Link = ClassList;
    Class = *Link;
    while Class != core::ptr::null_mut() {
        NextClass = (*Class).pclsNext;

        if (*Class).rpdeskParent == Desktop && (*Class).cWndReferenceCount == 0 {
            if IntMoveClassToSharedHeap(Class, &mut Link) != 0 {
            } else {
                if FreeOnFailure != 0 {
                    InterlockedExchangePointer(
                        Link as *mut PVOID,
                        (*Class).pclsNext as PVOID,
                    );

                    (*Class).pclsBase = core::ptr::null_mut();
                    IntDestroyClass(Class);
                } else {
                    Link = core::ptr::addr_of_mut!((*Class).pclsNext);
                    *Ret = 0;
                }
            }
        } else {
            Link = core::ptr::addr_of_mut!((*Class).pclsNext);
        }

        Class = NextClass;
    }
}

#[no_mangle]
pub unsafe extern "C" fn IntCheckProcessDesktopClasses(
    Desktop: PDESKTOP,
    FreeOnFailure: BOOL,
) -> BOOL {
    let pi: PPROCESSINFO;
    let mut Ret: BOOL = 1;

    pi = GetW32ProcessInfo();

    IntCheckDesktopClasses(
        Desktop,
        core::ptr::addr_of_mut!((*pi).pclsPrivateList),
        FreeOnFailure,
        core::ptr::addr_of_mut!(Ret),
    );

    IntCheckDesktopClasses(
        Desktop,
        core::ptr::addr_of_mut!((*pi).pclsPublicList),
        FreeOnFailure,
        core::ptr::addr_of_mut!(Ret),
    );

    if Ret == 0 {
        EngSetLastError(ERROR_NOT_ENOUGH_MEMORY);
    }

    Ret
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn IntCreateClass(
    lpwcx: *const WNDCLASSEXW,
    ClassName: PUNICODE_STRING,
    ClassVersion: PUNICODE_STRING,
    MenuName: PUNICODE_STRING,
    fnID: u32,
    dwFlags: u32,
    Desktop: PDESKTOP,
    pi: PPROCESSINFO,
) -> PCLS {
    IntCreateClass_impl(lpwcx, ClassName, ClassVersion, MenuName, fnID, dwFlags, Desktop, pi)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn IntCreateClass(
    lpwcx: *const WNDCLASSEXW,
    ClassName: PUNICODE_STRING,
    ClassVersion: PUNICODE_STRING,
    MenuName: PUNICODE_STRING,
    fnID: u32,
    dwFlags: u32,
    Desktop: PDESKTOP,
    pi: PPROCESSINFO,
) -> PCLS {
    IntCreateClass_impl(lpwcx, ClassName, ClassVersion, MenuName, fnID, dwFlags, Desktop, pi)
}

#[allow(unpredictable_function_pointer_comparisons)] // FNPTR-CMP(audited): intentional identity compare mirroring the C source; revisit -> core::ptr::fn_addr_eq (see docs/engineering/active-lanes/warnings-burndown-plan.md)
unsafe fn IntCreateClass_impl(
    lpwcx: *const WNDCLASSEXW,
    ClassName: PUNICODE_STRING,
    ClassVersion: PUNICODE_STRING,
    MenuName: PUNICODE_STRING,
    fnID: u32,
    dwFlags: u32,
    Desktop: PDESKTOP,
    pi: PPROCESSINFO,
) -> PCLS {
    let ClassSize: usize;
    let mut Class: PCLS = core::ptr::null_mut();
    let mut Atom: RTL_ATOM = 0;
    let mut verAtom: RTL_ATOM = 0;
    let mut WndProc: WNDPROC;
    let mut pszMenuName: PWSTR = core::ptr::null_mut();
    let mut Status: NTSTATUS = STATUS_SUCCESS;

    let _ = pi;

    if IntRegisterClassAtom(ClassName, &mut Atom) == 0 {
        return core::ptr::null_mut();
    }

    if IntRegisterClassAtom(ClassVersion, &mut verAtom) == 0 {
        IntDeregisterClassAtom(Atom);
        return core::ptr::null_mut();
    }

    ClassSize = core::mem::size_of::<_CLS>() + (*lpwcx).cbClsExtra as usize;

    'nomem: {
        if (*MenuName).Length != 0 {
            pszMenuName = UserHeapAlloc(
                (*MenuName).Length as usize
                    + core::mem::size_of::<u16>()
                    + RtlUnicodeStringToAnsiSize(MenuName) as usize,
            ) as PWSTR;
            if pszMenuName.is_null() {
                break 'nomem;
            }
        }

        if !Desktop.is_null() {
            Class = DesktopHeapAlloc(Desktop, ClassSize as usize) as PCLS;
        } else {
            Class = UserHeapAlloc(ClassSize as usize) as PCLS;
        }

        if !Class.is_null() {
            let mut iCls: i32 = 0;

            core::ptr::write_bytes(Class as *mut u8, 0, ClassSize);

            (*Class).rpdeskParent = Desktop;
            (*Class).pclsBase = Class;
            (*Class).atomClassName = verAtom;
            (*Class).atomNVClassName = Atom;
            (*Class).fnid = fnID;
            (*Class).CSF_flags = dwFlags;

            if LookupFnIdToiCls((*Class).fnid as i32, &mut iCls) != 0
                && (*gpsi).atomSysClass[iCls as usize] == 0
            {
                (*gpsi).atomSysClass[iCls as usize] = (*Class).atomClassName;
            }

            // ---- dropped _SEH2_TRY/_SEH2_EXCEPT recovered here (class.c:1222) ----
            // Handler shape: plain fall-through `Status = _SEH2_GetExceptionCode();`
            // -- the `if !NT_SUCCESS(Status)` cleanup right after this block
            // already exists and is unchanged. `lpwcx` itself is always a
            // kernel-owned copy at both call sites (UserRegisterClass's
            // captured struct, or the local `wc` built by
            // UserRegisterSystemClasses), so its field reads can't fault and
            // get no new protection (rule: don't add a probe/guard the C
            // didn't need). `(*MenuName).Buffer`, however, is the raw
            // CALLER pointer (NtUserRegisterClassExWOW only captures the
            // UNICODE_STRING header, not the string bytes) -- that's the
            // literal RtlCopyMemory the C's SEH existed to catch.
            'seh_try: {
                let mut pszMenuNameBuffer: PWSTR = pszMenuName;

                (*Class).lpfnWndProc = (*lpwcx).lpfnWndProc;
                (*Class).style = (*lpwcx).style;
                (*Class).cbclsExtra = (*lpwcx).cbClsExtra;
                (*Class).cbwndExtra = (*lpwcx).cbWndExtra;
                (*Class).hModule = (*lpwcx).hInstance;
                (*Class).spicn = if !(*lpwcx).hIcon.is_null() {
                    UserGetCurIconObject((*lpwcx).hIcon)
                } else {
                    core::ptr::null_mut()
                };
                (*Class).spcur = if !(*lpwcx).hCursor.is_null() {
                    UserGetCurIconObject((*lpwcx).hCursor)
                } else {
                    core::ptr::null_mut()
                };
                (*Class).spicnSm = if !(*lpwcx).hIconSm.is_null() {
                    UserGetCurIconObject((*lpwcx).hIconSm)
                } else {
                    core::ptr::null_mut()
                };
                (*Class).hbrBackground = (*lpwcx).hbrBackground;

                if !pszMenuNameBuffer.is_null() {
                    (*Class).set_MenuNameIsString(1);

                    (*Class).lpszClientUnicodeMenuName = pszMenuNameBuffer;
                    if let Err(st) = ntkrnl_sys::probe::safe_copy_raw(
                        (*Class).lpszClientUnicodeMenuName as *mut core::ffi::c_void,
                        (*MenuName).Buffer as *const core::ffi::c_void,
                        (*MenuName).Length as usize,
                    ) {
                        Status = st;
                        break 'seh_try;
                    }
                    *(*Class)
                        .lpszClientUnicodeMenuName
                        .add((*MenuName).Length as usize / core::mem::size_of::<u16>()) = 0;

                    pszMenuNameBuffer =
                        pszMenuNameBuffer.add((*MenuName).Length as usize / core::mem::size_of::<u16>() + 1);
                } else {
                    (*Class).lpszClientUnicodeMenuName = (*MenuName).Buffer;
                }

                'leave: {
                    if !pszMenuNameBuffer.is_null() {
                        let mut AnsiString: ANSI_STRING = core::mem::zeroed();

                        (*Class).lpszClientAnsiMenuName = pszMenuNameBuffer as PSTR;
                        AnsiString.MaximumLength = RtlUnicodeStringToAnsiSize(MenuName) as u16;
                        AnsiString.Buffer = (*Class).lpszClientAnsiMenuName;
                        Status = RtlUnicodeStringToAnsiString(core::ptr::addr_of_mut!(AnsiString) as _, MenuName as _, 0);
                        if !NT_SUCCESS(Status) {
                            break 'leave;
                        }
                    } else {
                        (*Class).lpszClientAnsiMenuName = (*MenuName).Buffer as PSTR;
                    }

                    (*Class).lpszMenuName = (*Class).lpszClientUnicodeMenuName;

                    if ((*Class).CSF_flags & CSF_SERVERSIDEPROC) == 0 {
                        WndProc = None;

                        let mut i = FNID_FIRST as i32;
                        while i <= FNID_SWITCH as i32 {
                            if GETPFNCLIENTW(i) == (*Class).lpfnWndProc {
                                WndProc = GETPFNSERVER(i);
                                break;
                            }
                            if GETPFNCLIENTA(i) == (*Class).lpfnWndProc {
                                WndProc = GETPFNSERVER(i);
                                break;
                            }
                            i += 1;
                        }
                        if WndProc.is_some() {
                            (*Class).CSF_flags |= CSF_SERVERSIDEPROC;
                            (*Class).CSF_flags &= !CSF_ANSIPROC;
                            (*Class).lpfnWndProc = WndProc;
                        }
                    }

                    if ((*Class).CSF_flags & CSF_ANSIPROC) == 0 {
                        (*Class).set_Unicode(1);
                    }

                    if ((*Class).style & CS_GLOBALCLASS) != 0 {
                        (*Class).set_Global(1);
                    }
                }
            }

            if !NT_SUCCESS(Status) {
                SetLastNtError(Status);

                if !pszMenuName.is_null() {
                    UserHeapFree(pszMenuName as *mut core::ffi::c_void);
                }

                DesktopHeapFree(Desktop, Class as *mut core::ffi::c_void);
                Class = core::ptr::null_mut();

                IntDeregisterClassAtom(verAtom);
                IntDeregisterClassAtom(Atom);
            }

            return Class;
        }
    }

    if !pszMenuName.is_null() {
        UserHeapFree(pszMenuName as *mut core::ffi::c_void);
    }

    IntDeregisterClassAtom(Atom);
    IntDeregisterClassAtom(verAtom);

    EngSetLastError(ERROR_NOT_ENOUGH_MEMORY);

    Class
}

unsafe fn IntFindClass(
    Atom: RTL_ATOM,
    hInstance: HINSTANCE,
    ClassList: *mut PCLS,
    Link: *mut *mut PCLS,
) -> PCLS {
    let mut PrevLink: *mut PCLS = ClassList;
    let mut Class: PCLS = *PrevLink;

    while !Class.is_null() {
        if (*Class).atomClassName == Atom
            && (hInstance.is_null() || (*Class).hModule == hInstance)
            && ((*Class).CSF_flags as u32 & CSF_WOWDEFERDESTROY) == 0
        {
            if !Link.is_null() {
                *Link = PrevLink;
            }
            break;
        }

        PrevLink = core::ptr::addr_of_mut!((*Class).pclsNext);
        Class = (*Class).pclsNext;
    }

    Class
}
#[no_mangle]
pub unsafe extern "system" fn IntGetAtomFromStringOrAtom(
    ClassName: PUNICODE_STRING,
    Atom: *mut RTL_ATOM,
) -> i32 {
    let mut Ret: i32 = 0;

    if (*ClassName).Length != 0 {
        let mut szBuf: [u16; 65] = [0; 65];
        let mut AtomName: PWSTR = szBuf.as_mut_ptr();
        let mut Status = STATUS_INVALID_PARAMETER;

        *Atom = 0;

        if ((*ClassName).Length as usize + core::mem::size_of::<u16>())
            > core::mem::size_of::<[u16; 65]>()
        {
            AtomName = ExAllocatePoolWithTag(
                PagedPool,
                ((*ClassName).Length as usize + core::mem::size_of::<u16>()) as SIZE_T,
                0x52545355,
            ) as PWSTR;
            if AtomName.is_null() {
                EngSetLastError(ERROR_OUTOFMEMORY);
                return 0;
            }
        }

        // ---- dropped _SEH2_TRY/_SEH2_EXCEPT recovered here (class.c:1331) ----
        // Same shape as IntRegisterClassAtom above (site class.c:368): no
        // ProbeForRead in the C, RtlCopyMemory alone was guarded, handler
        // frees AtomName if heap-allocated, SetLastNtError, immediate
        // `_SEH2_YIELD(return FALSE)`.
        if let Err(st) = ntkrnl_sys::probe::safe_copy_raw(
            AtomName as *mut core::ffi::c_void,
            (*ClassName).Buffer as *const core::ffi::c_void,
            (*ClassName).Length as usize,
        ) {
            if AtomName != szBuf.as_mut_ptr() {
                ExFreePoolWithTag(AtomName as *mut core::ffi::c_void, 0x52545355);
            }
            SetLastNtError(st);
            return 0;
        }

        *AtomName.add((*ClassName).Length as usize / core::mem::size_of::<u16>()) = 0;

        Status = RtlLookupAtomInAtomTable(gAtomTable as _, AtomName, Atom);

        if AtomName != szBuf.as_mut_ptr() {
            ExFreePoolWithTag(AtomName as *mut core::ffi::c_void, 0x52545355);
        }

        if NT_SUCCESS(Status) {
            Ret = 1;
        } else {
            if Status != STATUS_OBJECT_NAME_NOT_FOUND {
                SetLastNtError(Status);
            }
        }
    } else {
        if !(*ClassName).Buffer.is_null() {
            *Atom = (*ClassName).Buffer as usize as RTL_ATOM;
            Ret = 1;
        } else {
            *Atom = 0;
            EngSetLastError(ERROR_CLASS_DOES_NOT_EXIST);
            Ret = 0;
        }
    }

    Ret
}

#[no_mangle]
pub unsafe extern "C" fn IntGetClassAtom(
    ClassName: PUNICODE_STRING,
    hInstance: HINSTANCE,
    pi: PPROCESSINFO,
    BaseClass: *mut PCLS,
    Link: *mut *mut PCLS,
) -> RTL_ATOM {
    let mut Atom: RTL_ATOM = 0;

    if IntGetAtomFromStringOrAtom(ClassName, &mut Atom) != 0 && Atom != 0 {
        let mut Class: PCLS;

        'found: {
            /* Step 1: Try to find an exact match of locally registered classes */
            Class = IntFindClass(
                Atom,
                hInstance,
                core::ptr::addr_of_mut!((*pi).pclsPrivateList),
                Link,
            );
            if !Class.is_null() {
                TRACE!(b"Step 1: 0x%p\n\0".as_ptr() as *const i8, Class);
                break 'found;
            }

            /* Step 2: Try to find any globally registered class. The hInstance
                       is not relevant for global classes */
            Class = IntFindClass(
                Atom,
                core::ptr::null_mut(),
                core::ptr::addr_of_mut!((*pi).pclsPublicList),
                Link,
            );
            if !Class.is_null() {
                TRACE!(
                    b"Step 2: 0x%p 0x%p\n\0".as_ptr() as *const i8,
                    Class,
                    (*Class).hModule,
                );
                break 'found;
            }

            /* Step 3: Try to find any local class registered by user32 */
            Class = IntFindClass(
                Atom,
                hModClient,
                core::ptr::addr_of_mut!((*pi).pclsPrivateList),
                Link,
            );
            if !Class.is_null() {
                TRACE!(b"Step 3: 0x%p\n\0".as_ptr() as *const i8, Class);
                break 'found;
            }

            /* Step 4: Try to find any global class registered by user32 */
            Class = IntFindClass(
                Atom,
                hModClient,
                core::ptr::addr_of_mut!((*pi).pclsPublicList),
                Link,
            );
            if Class.is_null() {
                return 0;
            } else {
                TRACE!(b"Step 4: 0x%p\n\0".as_ptr() as *const i8, Class);
            }
        }

        *BaseClass = Class;
    } else {
        Atom = 0;
    }

    Atom
}

#[no_mangle]
pub unsafe extern "C" fn IntGetAndReferenceClass(
    ClassName: PUNICODE_STRING,
    hInstance: HINSTANCE,
    bDesktopThread: BOOL,
) -> PCLS {
    let mut ClassLink: *mut PCLS = core::ptr::null_mut();
    let mut Class: PCLS = core::ptr::null_mut();
    let ClassAtom: RTL_ATOM;
    let pti: PTHREADINFO;

    if bDesktopThread != 0 {
        pti = gptiDesktopThread;
    } else {
        pti = PsGetCurrentThreadWin32Thread() as PTHREADINFO;
    }

    if ((*ppi_w32pf_flags((*pti).ppi)) & W32PF_CLASSESREGISTERED) == 0 {
        UserRegisterSystemClasses();
    }

    ClassAtom = IntGetClassAtom(
        ClassName,
        hInstance,
        (*pti).ppi,
        &mut Class,
        &mut ClassLink,
    );

    if ClassAtom == 0 as RTL_ATOM {
        return core::ptr::null_mut();
    }

    Class = IntReferenceClass(Class, ClassLink, (*pti).rpdesk);
    if Class == core::ptr::null_mut() {
        return core::ptr::null_mut();
    }

    Class
}

#[no_mangle]
pub unsafe extern "C" fn UserRegisterClass(
    lpwcx: *const WNDCLASSEXW,
    ClassName: PUNICODE_STRING,
    ClassVersion: PUNICODE_STRING,
    MenuName: PUNICODE_STRING,
    fnID: u32,
    dwFlags: u32,
) -> RTL_ATOM {
    let pti: PTHREADINFO = GetW32ThreadInfo();

    let pi: PPROCESSINFO = (*pti).ppi;

    let mut ClassAtom: RTL_ATOM = 0;
    let mut Ret: RTL_ATOM = 0;

    if IntGetAtomFromStringOrAtom(ClassVersion, &mut ClassAtom) != 0
        && ClassAtom != 0
        && (dwFlags & CSF_SERVERSIDEPROC) == 0
    {
        let mut Class: PCLS = IntFindClass(
            ClassAtom,
            (*lpwcx).hInstance,
            core::ptr::addr_of_mut!((*pi).pclsPrivateList),
            core::ptr::null_mut(),
        );

        if !Class.is_null() && (*Class).Global() == 0 {
            EngSetLastError(ERROR_CLASS_ALREADY_EXISTS);
            return 0;
        }

        if ((*lpwcx).style & CS_GLOBALCLASS) != 0 {
            Class = IntFindClass(
                ClassAtom,
                core::ptr::null_mut(),
                core::ptr::addr_of_mut!((*pi).pclsPublicList),
                core::ptr::null_mut(),
            );

            if !Class.is_null() && (*Class).Global() != 0 {
                EngSetLastError(ERROR_CLASS_ALREADY_EXISTS);
                return 0;
            }
        }
    }

    let Class: PCLS = IntCreateClass(
        lpwcx,
        ClassName,
        ClassVersion,
        MenuName,
        fnID,
        dwFlags,
        (*pti).rpdesk,
        pi,
    );

    if !Class.is_null() {
        let List: *mut PCLS = if (*Class).Global() != 0 {
            core::ptr::addr_of_mut!((*pi).pclsPublicList)
        } else {
            core::ptr::addr_of_mut!((*pi).pclsPrivateList)
        };

        (*Class).pclsNext = *List;
        InterlockedExchangePointer(List as *mut *mut core::ffi::c_void, Class as *mut core::ffi::c_void);

        Ret = (*Class).atomNVClassName;
    }

    Ret
}

#[no_mangle]
pub unsafe extern "C" fn UserUnregisterClass(
    ClassName: PUNICODE_STRING,
    hInstance: HINSTANCE,
    pClassMenuName: PCLSMENUNAME,
) -> BOOL {
    let mut Link: *mut PCLS = core::ptr::null_mut();
    let pi: PPROCESSINFO;
    let ClassAtom: RTL_ATOM;
    let mut Class: PCLS = core::ptr::null_mut();

    pi = GetW32ProcessInfo();

    ClassAtom = IntGetClassAtom(
        ClassName,
        hInstance,
        pi,
        &mut Class,
        &mut Link,
    );
    if ClassAtom == 0 as RTL_ATOM {
        EngSetLastError(ERROR_CLASS_DOES_NOT_EXIST);
        return 0;
    }

    if (*Class).cWndReferenceCount != 0 || (*Class).pclsClone != core::ptr::null_mut() {
        EngSetLastError(ERROR_CLASS_HAS_WINDOWS);
        return 0;
    }

    /* Unlink the class */
    *Link = (*Class).pclsNext;

    if NT_SUCCESS(IntDeregisterClassAtom((*Class).atomClassName)) {
        (*Class).atomClassName = 0;
        IntDestroyClass(Class);
        return 1;
    }
    0
}

#[no_mangle]
pub unsafe extern "C" fn UserGetClassName(
    Class: PCLS,
    ClassName: PUNICODE_STRING,
    Atom: RTL_ATOM,
    Ansi: BOOL,
) -> INT {
    let mut Status: NTSTATUS = STATUS_SUCCESS;
    let mut szStaticTemp: [WCHAR; 32] = [0; 32];
    let mut szTemp: PWSTR = core::ptr::null_mut();
    let mut BufLen: ULONG = core::mem::size_of::<[WCHAR; 32]>() as ULONG;
    let mut Ret: INT = 0;

    'leave: {
        if Ansi != 0 {
            let AnsiClassName: PANSI_STRING = ClassName as PANSI_STRING;
            let mut UnicodeClassName: _UNICODE_STRING = core::mem::zeroed();

            if BufLen / core::mem::size_of::<WCHAR>() as ULONG
                > (*AnsiClassName).MaximumLength as ULONG
            {
                BufLen = (*AnsiClassName).MaximumLength as ULONG
                    * core::mem::size_of::<WCHAR>() as ULONG;
            }

            Status = RtlQueryAtomInAtomTable(
                gAtomTable as _,
                (*Class).atomClassName,
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                szStaticTemp.as_mut_ptr() as PWSTR,
                &mut BufLen,
            );
            if Status == STATUS_BUFFER_TOO_SMALL {
                if BufLen / core::mem::size_of::<WCHAR>() as ULONG
                    > (*AnsiClassName).MaximumLength as ULONG
                {
                    BufLen = (*AnsiClassName).MaximumLength as ULONG
                        * core::mem::size_of::<WCHAR>() as ULONG;
                }

                szTemp = ExAllocatePoolWithTag(PagedPool, BufLen as SIZE_T, USERTAG_CLASS) as PWSTR;
                if szTemp.is_null() {
                    EngSetLastError(ERROR_NOT_ENOUGH_MEMORY);
                    break 'leave;
                }

                Status = RtlQueryAtomInAtomTable(
                    gAtomTable as _,
                    if Atom != 0 { Atom } else { (*Class).atomNVClassName },
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                    szTemp as PWSTR,
                    &mut BufLen,
                );
            } else {
                szTemp = szStaticTemp.as_mut_ptr();
            }

            if NT_SUCCESS(Status) {
                RtlInitUnicodeString(core::ptr::addr_of_mut!(UnicodeClassName) as _, szTemp);

                // ---- dropped _SEH2_TRY/_SEH2_EXCEPT recovered here (class.c:1753) ----
                // The C calls RtlUnicodeStringToAnsiString(AnsiClassName, ...)
                // directly, writing ANSI bytes straight into AnsiClassName->Buffer,
                // which IS the CALLER's raw pointer (this fn's `ClassName` param is
                // `&CapturedClassName` from NtUserGetClassName, but `.Buffer`'s
                // VALUE was only shallow-copied from the original user pointer).
                // RtlUnicodeStringToAnsiString is an opaque ntoskrnl routine --
                // we cannot make its internal store fault-tolerant directly. So
                // the conversion is redirected into a KERNEL-owned temporary
                // ANSI_STRING, and `safe_copy_raw` does the one store that
                // actually reaches the caller's buffer: same bytes end up in the
                // same place on success, but the only risky store now goes
                // through hardware fault protection instead of a bare write.
                let mut TempAnsi: ANSI_STRING = core::mem::zeroed();
                TempAnsi.MaximumLength = (*AnsiClassName).MaximumLength;
                TempAnsi.Buffer = ExAllocatePoolWithTag(
                    PagedPool,
                    TempAnsi.MaximumLength as SIZE_T,
                    USERTAG_CLASS,
                ) as PSTR;
                if TempAnsi.Buffer.is_null() {
                    EngSetLastError(ERROR_NOT_ENOUGH_MEMORY);
                    break 'leave;
                }

                Status = RtlUnicodeStringToAnsiString(
                    core::ptr::addr_of_mut!(TempAnsi) as _,
                    core::ptr::addr_of_mut!(UnicodeClassName) as _,
                    0u8,
                );
                if !NT_SUCCESS(Status) {
                    ExFreePoolWithTag(TempAnsi.Buffer as *mut c_void, USERTAG_CLASS);
                    SetLastNtError(Status);
                    break 'leave;
                }

                // RtlUnicodeStringToAnsiString terminates when MaximumLength has room, and the
                // C ran it on the caller's buffer; copy that terminator out too (see the
                // Unicode branch below).
                let mut CopyLen = TempAnsi.Length as usize;
                if CopyLen < TempAnsi.MaximumLength as usize {
                    *TempAnsi.Buffer.add(CopyLen) = 0;
                    CopyLen += 1;
                }
                if let Err(_st) = ntkrnl_sys::probe::safe_copy_raw(
                    (*AnsiClassName).Buffer as *mut c_void,
                    TempAnsi.Buffer as *const c_void,
                    CopyLen,
                ) {
                    ExFreePoolWithTag(TempAnsi.Buffer as *mut c_void, USERTAG_CLASS);
                    SetLastNtError(STATUS_ACCESS_VIOLATION);
                    break 'leave;
                }
                (*AnsiClassName).Length = TempAnsi.Length;
                ExFreePoolWithTag(TempAnsi.Buffer as *mut c_void, USERTAG_CLASS);
            }

            Ret = (BufLen / core::mem::size_of::<WCHAR>() as ULONG) as INT;
        } else {
            BufLen = (*ClassName).MaximumLength as ULONG;

            // ---- dropped _SEH2_TRY/_SEH2_EXCEPT recovered here (class.c:1753) ----
            // Same reasoning as the ansi branch above: RtlQueryAtomInAtomTable
            // writes straight into `ClassName->Buffer`, the caller's raw
            // pointer, and is an opaque call we cannot wrap directly. Query
            // into a KERNEL temp buffer of the same capacity, then
            // `safe_copy_raw` the result out to the caller's buffer.
            let TempBufCap = BufLen.max(1);
            let TempBuf: PWSTR =
                ExAllocatePoolWithTag(PagedPool, TempBufCap as SIZE_T, USERTAG_CLASS) as PWSTR;
            if TempBuf.is_null() {
                EngSetLastError(ERROR_NOT_ENOUGH_MEMORY);
                break 'leave;
            }

            Status = RtlQueryAtomInAtomTable(
                gAtomTable as _,
                if Atom != 0 { Atom } else { (*Class).atomNVClassName },
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                TempBuf,
                &mut BufLen,
            );

            if !NT_SUCCESS(Status) {
                ExFreePoolWithTag(TempBuf as *mut c_void, USERTAG_CLASS);
                SetLastNtError(Status);
                break 'leave;
            }

            // The C lets RtlQueryAtomInAtomTable write straight into the caller's buffer,
            // name AND terminator when there is room. BufLen counts the name only, so copying
            // BufLen bytes out of the temp dropped the NUL: GetClassNameW then handed back
            // whatever the caller's buffer held past the name (a zeroed buffer hid it).
            let mut CopyLen = BufLen.min(TempBufCap) as usize;
            if CopyLen + core::mem::size_of::<WCHAR>() <= TempBufCap as usize {
                *TempBuf.add(CopyLen / core::mem::size_of::<WCHAR>()) = 0;
                CopyLen += core::mem::size_of::<WCHAR>();
            }
            if let Err(_st) = ntkrnl_sys::probe::safe_copy_raw(
                (*ClassName).Buffer as *mut c_void,
                TempBuf as *const c_void,
                CopyLen,
            ) {
                ExFreePoolWithTag(TempBuf as *mut c_void, USERTAG_CLASS);
                SetLastNtError(STATUS_ACCESS_VIOLATION);
                break 'leave;
            }
            ExFreePoolWithTag(TempBuf as *mut c_void, USERTAG_CLASS);

            Ret = (BufLen / core::mem::size_of::<WCHAR>() as ULONG) as INT;
        }
    }

    if Ansi != 0 && !szTemp.is_null() && szTemp != szStaticTemp.as_mut_ptr() {
        ExFreePoolWithTag(szTemp as *mut core::ffi::c_void, USERTAG_CLASS);
    }

    Ret
}

unsafe fn IntSetClassMenuName(Class: PCLS, MenuName: PUNICODE_STRING) -> BOOL {
    let mut Ret: BOOL = 0;

    /* Change the base class first */
    let mut Class = (*Class).pclsBase;

    if (*MenuName).Length != 0 {
        let mut AnsiString: ANSI_STRING = core::mem::zeroed();

        AnsiString.MaximumLength = RtlUnicodeStringToAnsiSize(MenuName) as USHORT;

        let strBufW: PWSTR = UserHeapAlloc(
            (*MenuName).Length as usize + core::mem::size_of::<u16>()
                + AnsiString.MaximumLength as usize,
        ) as PWSTR;

        if strBufW != core::ptr::null_mut() {
            // ---- dropped _SEH2_TRY/_SEH2_EXCEPT recovered here (class.c:1810) ----
            // Handler shape: `SetLastNtError(code);` only, no early return
            // (Ret stays 0, matching the C falling through to _SEH2_END with
            // Ret untouched). `(*MenuName).Buffer` is a raw caller pointer,
            // so the RtlCopyMemory read from it is the real fault risk --
            // replaced with safe_copy_raw.
            'seh: {
                /* Copy the unicode string */
                if let Err(st) = ntkrnl_sys::probe::safe_copy_raw(
                    strBufW as *mut core::ffi::c_void,
                    (*MenuName).Buffer as *const core::ffi::c_void,
                    (*MenuName).Length as usize,
                ) {
                    SetLastNtError(st);
                    break 'seh;
                }
                *strBufW.add((*MenuName).Length as usize / core::mem::size_of::<u16>()) = 0;

                /* Create an ANSI copy of the string */
                AnsiString.Buffer = strBufW.add((*MenuName).Length as usize / core::mem::size_of::<u16>() + 1) as PSTR;
                // Source is now the already fault-protected kernel copy
                // (strBufW), not the raw `MenuName` pointer -- avoids a
                // second raw read of caller memory for a value we already
                // hold safely, and needs no separate guard.
                let mut SafeMenuName: _UNICODE_STRING = core::mem::zeroed();
                SafeMenuName.Length = (*MenuName).Length;
                SafeMenuName.MaximumLength = (*MenuName).Length;
                SafeMenuName.Buffer = strBufW;
                let Status = RtlUnicodeStringToAnsiString(core::ptr::addr_of_mut!(AnsiString) as _, core::ptr::addr_of_mut!(SafeMenuName) as _, 0u8);
                if !NT_SUCCESS(Status) {
                    SetLastNtError(Status);
                } else {
                    Ret = 1;
                }
            }

            if Ret != 0 {
                /* Update the base class */
                IntFreeClassMenuName(Class);
                (*Class).lpszClientUnicodeMenuName = strBufW;
                (*Class).lpszClientAnsiMenuName = AnsiString.Buffer;
                (*Class).set_MenuNameIsString(1);

                /* Update the clones */
                Class = (*Class).pclsClone;
                while Class != core::ptr::null_mut() {
                    (*Class).lpszClientUnicodeMenuName = strBufW;
                    (*Class).lpszClientAnsiMenuName = AnsiString.Buffer;
                    (*Class).set_MenuNameIsString(1);

                    Class = (*Class).pclsNext;
                }
            } else {
                ERR!("Failed to copy class menu name!\n");
                UserHeapFree(strBufW as *mut core::ffi::c_void);
            }
        } else {
            EngSetLastError(ERROR_NOT_ENOUGH_MEMORY);
        }
    } else {
        /* Update the base class */
        IntFreeClassMenuName(Class);
        (*Class).lpszClientUnicodeMenuName = (*MenuName).Buffer;
        (*Class).lpszClientAnsiMenuName = (*MenuName).Buffer as PSTR;
        (*Class).set_MenuNameIsString(0);

        /* Update the clones */
        Class = (*Class).pclsClone;
        while Class != core::ptr::null_mut() {
            (*Class).lpszClientUnicodeMenuName = (*MenuName).Buffer;
            (*Class).lpszClientAnsiMenuName = (*MenuName).Buffer as PSTR;
            (*Class).set_MenuNameIsString(0);

            Class = (*Class).pclsNext;
        }

        Ret = 1;
    }

    Ret
}

unsafe fn IntGetSetClassLongPtr(Class: PCLS, Index: ULONG, NewValue: ULONG_PTR, Size: ULONG) -> ULONG_PTR {
    let Address = (Class.add(1) as PUCHAR).add(Index as usize) as PVOID;
    let OldValue: ULONG_PTR;

    #[cfg(target_pointer_width = "64")]
    {
        if Size as usize == core::mem::size_of::<LONG>() {
            OldValue = ReadUnalignedU32(Address) as ULONG_PTR;
            WriteUnalignedU32(Address, NewValue as u32);
            return OldValue;
        }
    }

    OldValue = ReadUnalignedUlongPtr(Address);
    WriteUnalignedUlongPtr(Address, NewValue);

    OldValue
}

#[no_mangle]
pub unsafe extern "C" fn UserSetClassLongPtr(mut Class: PCLS, Index: INT, NewLong: ULONG_PTR, Ansi: BOOL, Size: ULONG) -> ULONG_PTR {
    let mut Ret: ULONG_PTR = 0;

    /* Change the information in the base class first, then update the clones */
    Class = (*Class).pclsBase;

    if Index >= 0 {
        if ((Index as ULONG).wrapping_add(Size)) < Index as ULONG
            || ((Index as ULONG).wrapping_add(Size)) > (*Class).cbclsExtra as ULONG
        {
            EngSetLastError(ERROR_INVALID_PARAMETER);
            return 0;
        }

        Ret = IntGetSetClassLongPtr(Class, Index as ULONG, NewLong, Size);

        /* Update the clones */
        Class = (*Class).pclsClone;
        while !Class.is_null() {
            IntGetSetClassLongPtr(Class, Index as ULONG, NewLong, Size);
            Class = (*Class).pclsNext;
        }

        return Ret;
    }

    match Index {
        x if x == GCL_CBWNDEXTRA => {
            Ret = (*Class).cbwndExtra as ULONG_PTR;
            (*Class).cbwndExtra = NewLong as INT;

            Class = (*Class).pclsClone;
            while !Class.is_null() {
                (*Class).cbwndExtra = NewLong as INT;
                Class = (*Class).pclsNext;
            }
        }

        x if x == GCL_CBCLSEXTRA => {
            EngSetLastError(ERROR_INVALID_PARAMETER);
        }

        x if x == GCLP_HBRBACKGROUND => {
            Ret = (*Class).hbrBackground as ULONG_PTR;
            (*Class).hbrBackground = NewLong as HBRUSH;

            Class = (*Class).pclsClone;
            while !Class.is_null() {
                (*Class).hbrBackground = NewLong as HBRUSH;
                Class = (*Class).pclsNext;
            }
        }

        x if x == GCLP_HCURSOR => {
            let mut NewCursor: PCURICON_OBJECT = core::ptr::null_mut();

            if NewLong != 0 {
                NewCursor = UserGetCurIconObject(NewLong as HCURSOR);
                if NewCursor.is_null() {
                    EngSetLastError(ERROR_INVALID_CURSOR_HANDLE);
                    return 0;
                }
            }

            if !(*Class).spcur.is_null() {
                Ret = UserHMGetHandle((*Class).spcur) as ULONG_PTR;
                UserDereferenceObject((*Class).spcur as PVOID);
            } else {
                Ret = 0;
            }

            if Ret == NewLong {
                return Ret;
            }

            (*Class).spcur = NewCursor;

            Class = (*Class).pclsClone;
            while !Class.is_null() {
                if !(*Class).spcur.is_null() {
                    UserDereferenceObject((*Class).spcur as PVOID);
                }
                if !NewCursor.is_null() {
                    UserReferenceObject(NewCursor as PVOID);
                }
                (*Class).spcur = NewCursor;
                Class = (*Class).pclsNext;
            }
        }

        x if x == GCLP_HICON => {
            let mut NewIcon: PCURICON_OBJECT = core::ptr::null_mut();
            let mut NewSmallIcon: PCURICON_OBJECT = core::ptr::null_mut();

            if NewLong != 0 {
                NewIcon = UserGetCurIconObject(NewLong as HCURSOR);
                if NewIcon.is_null() {
                    EngSetLastError(ERROR_INVALID_ICON_HANDLE);
                    return 0;
                }
            }

            if !(*Class).spicn.is_null() {
                Ret = UserHMGetHandle((*Class).spicn) as ULONG_PTR;
                UserDereferenceObject((*Class).spicn as PVOID);
            } else {
                Ret = 0;
            }

            if Ret == NewLong {
                return Ret;
            }

            if Ret != 0 && ((*Class).CSF_flags as ULONG & CSF_CACHEDSMICON) != 0 {
                UserDereferenceObject((*Class).spicnSm as PVOID);
                IntDestroyCurIconObject((*Class).spicnSm as _);
                (*Class).spicnSm = core::ptr::null_mut();
                (*Class).CSF_flags &= !(CSF_CACHEDSMICON as u32);
            }

            if NewLong != 0 && (*Class).spicnSm.is_null() {
                let mut SmallIconHandle: HICON = core::ptr::null_mut();
                if ((*NewIcon).CURSORF_flags as ULONG & (CURSORF_LRSHARED | CURSORF_FROMRESOURCE))
                    == (CURSORF_LRSHARED | CURSORF_FROMRESOURCE)
                {
                    SmallIconHandle = co_IntCopyImage_h(
                        NewLong as PVOID,
                        IMAGE_ICON,
                        UserGetSystemMetrics(SM_CXSMICON),
                        UserGetSystemMetrics(SM_CYSMICON),
                        LR_COPYFROMRESOURCE,
                    );
                }
                if SmallIconHandle.is_null() {
                    SmallIconHandle = co_IntCopyImage_h(
                        NewLong as PVOID,
                        IMAGE_ICON,
                        UserGetSystemMetrics(SM_CXSMICON),
                        UserGetSystemMetrics(SM_CYSMICON),
                        0,
                    );
                }
                if !SmallIconHandle.is_null() {
                    (*Class).spicnSm = UserGetCurIconObject(SmallIconHandle);
                    NewSmallIcon = (*Class).spicnSm;
                    (*Class).CSF_flags |= CSF_CACHEDSMICON as u32;
                }
            }

            (*Class).spicn = NewIcon;

            Class = (*Class).pclsClone;
            while !Class.is_null() {
                if !(*Class).spicn.is_null() {
                    UserDereferenceObject((*Class).spicn as PVOID);
                }
                if !NewIcon.is_null() {
                    UserReferenceObject(NewIcon as PVOID);
                }
                (*Class).spicn = NewIcon;
                if !NewSmallIcon.is_null() {
                    if !(*Class).spicnSm.is_null() {
                        UserDereferenceObject((*Class).spicnSm as PVOID);
                    }
                    UserReferenceObject(NewSmallIcon as PVOID);
                    (*Class).spicnSm = NewSmallIcon;
                    (*Class).CSF_flags |= CSF_CACHEDSMICON as u32;
                }
                Class = (*Class).pclsNext;
            }
        }

        x if x == GCLP_HICONSM => {
            let mut NewSmallIcon: PCURICON_OBJECT = core::ptr::null_mut();
            let mut NewIconFromCache: BOOLEAN = 0u8;

            if NewLong != 0 {
                NewSmallIcon = UserGetCurIconObject(NewLong as HCURSOR);
                if NewSmallIcon.is_null() {
                    EngSetLastError(ERROR_INVALID_ICON_HANDLE);
                    return 0;
                }
            } else {
                let mut SmallIconHandle: HICON = core::ptr::null_mut();
                if ((*(*Class).spicn).CURSORF_flags as ULONG
                    & (CURSORF_LRSHARED | CURSORF_FROMRESOURCE))
                    == (CURSORF_LRSHARED | CURSORF_FROMRESOURCE)
                {
                    SmallIconHandle = co_IntCopyImage_h(
                        UserHMGetHandle((*Class).spicn) as PVOID,
                        IMAGE_ICON,
                        UserGetSystemMetrics(SM_CXSMICON),
                        UserGetSystemMetrics(SM_CYSMICON),
                        LR_COPYFROMRESOURCE,
                    );
                }
                if SmallIconHandle.is_null() {
                    SmallIconHandle = co_IntCopyImage_h(
                        UserHMGetHandle((*Class).spicn) as PVOID,
                        IMAGE_ICON,
                        UserGetSystemMetrics(SM_CXSMICON),
                        UserGetSystemMetrics(SM_CYSMICON),
                        0,
                    );
                }
                if !SmallIconHandle.is_null() {
                    NewSmallIcon = UserGetCurIconObject(SmallIconHandle);
                    NewIconFromCache = 1u8;
                } else {
                    ERR!("Failed getting a small icon for the class.\n");
                }
            }

            if !(*Class).spicnSm.is_null() {
                if ((*Class).CSF_flags as ULONG & CSF_CACHEDSMICON) != 0 {
                    IntDestroyCurIconObject((*Class).spicnSm as _);
                    Ret = 0;
                } else {
                    Ret = UserHMGetHandle((*Class).spicnSm) as ULONG_PTR;
                }
                UserDereferenceObject((*Class).spicnSm as PVOID);
            } else {
                Ret = 0;
            }

            if NewIconFromCache != 0 {
                (*Class).CSF_flags |= CSF_CACHEDSMICON as u32;
            } else {
                (*Class).CSF_flags &= !(CSF_CACHEDSMICON as u32);
            }
            (*Class).spicnSm = NewSmallIcon;

            Class = (*Class).pclsClone;
            while !Class.is_null() {
                if !(*Class).spicnSm.is_null() {
                    UserDereferenceObject((*Class).spicnSm as PVOID);
                }
                if !NewSmallIcon.is_null() {
                    UserReferenceObject(NewSmallIcon as PVOID);
                }
                if NewIconFromCache != 0 {
                    (*Class).CSF_flags |= CSF_CACHEDSMICON as u32;
                } else {
                    (*Class).CSF_flags &= !(CSF_CACHEDSMICON as u32);
                }
                (*Class).spicnSm = NewSmallIcon;
                Class = (*Class).pclsNext;
            }
        }

        x if x == GCLP_HMODULE => {
            Ret = (*Class).hModule as ULONG_PTR;
            (*Class).hModule = NewLong as HINSTANCE;

            Class = (*Class).pclsClone;
            while !Class.is_null() {
                (*Class).hModule = NewLong as HINSTANCE;
                Class = (*Class).pclsNext;
            }
        }

        x if x == GCLP_MENUNAME => {
            let Value = NewLong as PUNICODE_STRING;

            if IntSetClassMenuName(Class, Value) == 0 {
                ERR!("Setting the class menu name failed!\n");
            }
        }

        x if x == GCL_STYLE => {
            Ret = (*Class).style as ULONG_PTR;
            (*Class).style = NewLong as UINT;

            Class = (*Class).pclsClone;
            while !Class.is_null() {
                (*Class).style = NewLong as UINT;
                Class = (*Class).pclsNext;
            }
        }

        x if x == GCLP_WNDPROC => {
            Ret = core::mem::transmute::<WNDPROC, ULONG_PTR>(IntSetClassWndProc(
                Class,
                core::mem::transmute::<ULONG_PTR, WNDPROC>(NewLong),
                Ansi,
            ));
        }

        x if x == GCW_ATOM => {
            let Value = NewLong as PUNICODE_STRING;

            Ret = (*Class).atomNVClassName as ULONG_PTR;
            if IntSetClassAtom(Class, Value) == 0 {
                Ret = 0;
            }
        }

        _ => {
            EngSetLastError(ERROR_INVALID_INDEX);
        }
    }

    Ret
}

unsafe fn UserGetClassInfo(
    Class: PCLS,
    lpwcx: PWNDCLASSEXW,
    Ansi: BOOL,
    hInstance: HINSTANCE,
) -> BOOL {
    if Class.is_null() {
        return 0;
    }

    (*lpwcx).style = (*Class).style;

    if (*Class).fnid != 0 {
        (*lpwcx).style &= !CS_GLOBALCLASS;
    }

    (*lpwcx).lpfnWndProc = IntGetClassWndProc(Class, Ansi);

    (*lpwcx).cbClsExtra = (*Class).cbclsExtra;
    (*lpwcx).cbWndExtra = (*Class).cbwndExtra;
    (*lpwcx).hIcon = if !(*Class).spicn.is_null() {
        UserHMGetHandle((*Class).spicn) as HICON
    } else {
        core::ptr::null_mut()
    };
    (*lpwcx).hCursor = if !(*Class).spcur.is_null() {
        UserHMGetHandle((*Class).spcur) as HCURSOR
    } else {
        core::ptr::null_mut()
    };
    (*lpwcx).hIconSm = if !(*Class).spicnSm.is_null() {
        UserHMGetHandle((*Class).spicnSm) as HICON
    } else {
        core::ptr::null_mut()
    };
    (*lpwcx).hbrBackground = (*Class).hbrBackground;

    if Ansi != 0 {
        (*(lpwcx as PWNDCLASSEXA)).lpszMenuName =
            (*Class).lpszClientAnsiMenuName as LPCSTR;
    } else {
        (*lpwcx).lpszMenuName = (*Class).lpszClientUnicodeMenuName as LPCWSTR;
    }

    if !(*Class).lpszClientUnicodeMenuName.is_null() && (*Class).MenuNameIsString() != 0 {
        (*lpwcx).lpszMenuName = UserHeapAddressToUser(if Ansi != 0 {
            (*Class).lpszClientAnsiMenuName as PVOID
        } else {
            (*Class).lpszClientUnicodeMenuName as PVOID
        }) as LPCWSTR;
    }

    if hInstance == hModClient {
        (*lpwcx).hInstance = core::ptr::null_mut();
    } else {
        (*lpwcx).hInstance = hInstance;
    }

    1
}
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn UserRegisterSystemClasses() -> BOOL {
    let mut ClassName: _UNICODE_STRING = core::mem::zeroed();
    let mut MenuName: _UNICODE_STRING = core::mem::zeroed();
    let ppi: PPROCESSINFO = GetW32ProcessInfo();
    let mut wc: WNDCLASSEXW = core::mem::zeroed();
    let mut Ret: BOOL = 1;
    let mut Flags: u32 = 0;

    if ((*ppi_w32pf_flags(ppi)) & W32PF_CLASSESREGISTERED) != 0 {
        return 1;
    }


    if hModClient.is_null() {
        return 0;
    }

    RtlZeroMemory(
        core::ptr::addr_of_mut!(ClassName) as *mut core::ffi::c_void,
        core::mem::size_of::<_UNICODE_STRING>(),
    );
    RtlZeroMemory(
        core::ptr::addr_of_mut!(MenuName) as *mut core::ffi::c_void,
        core::mem::size_of::<_UNICODE_STRING>(),
    );

    let count = DEFAULT_SERVER_CLASSES_COUNT;
    let mut i: u32 = 0;
    while i != count {
        let entry = core::ptr::addr_of!(DefaultServerClasses[i as usize]);

        if IS_ATOM!((*entry).ClassName) == 0 {
            RtlInitUnicodeString(core::ptr::addr_of_mut!(ClassName) as _, (*entry).ClassName);
        } else {
            ClassName.Buffer = (*entry).ClassName;
            ClassName.Length = 0;
            ClassName.MaximumLength = 0;
        }

        wc.cbSize = core::mem::size_of::<WNDCLASSEXW>() as u32;
        wc.style = (*entry).Style;

        Flags |= CSF_SERVERSIDEPROC;

        if (*entry).ProcW.is_some() || !core::mem::transmute::<_, *mut core::ffi::c_void>((*entry).ProcW).is_null() {
            wc.lpfnWndProc = (*entry).ProcW;
            wc.hInstance = hModuleWin as _;
        } else {
            wc.lpfnWndProc = GETPFNSERVER((*entry).fiId as INT);
            wc.hInstance = hModClient;
        }

        wc.cbClsExtra = 0;
        wc.cbWndExtra = (*entry).ExtraBytes as INT;
        wc.hIcon = core::ptr::null_mut();

        wc.hCursor = core::ptr::null_mut();
        if (*entry).hCursor == OCR_NORMAL as HICON {
            if SYSTEMCUR_ARROW().is_null() {
                // Only a defect once the cursors are actually loaded. Before that
                // it is the expected state and NOT worth an ERR: system cursors
                // load via a user-mode callback, which cannot be made from the
                // system desktop thread (its process has no KernelCallbackTable),
                // so nothing can populate gasyscur until the first real GUI process
                // exists. Every process registering before that legitimately gets
                // NULL. Measured on an i386 boot: 7 such processes (csrss + six
                // background services) x 6 server-side classes = 42 lines of
                // "should not happen!!" for a state that must happen. Every process
                // that draws visible UI -- winlogon, msgina, explorer -- registers
                // after the load and takes the real cursor.
                if crate::winsta::gbDesktopCursorsAndIconsLoaded != 0 {
                    ERR!("SYSTEMCUR(ARROW) == NULL after the deferred load!!\n");
                }
            } else {
                wc.hCursor = UserHMGetHandle(SYSTEMCUR_ARROW()) as _;
            }
        }

        let mut hBrush: HBRUSH = (*entry).hBrush;
        if (hBrush as usize) <= (COLOR_MENUBAR as usize) {
            hBrush = IntGetSysColorBrush(HandleToUlong(hBrush as *mut core::ffi::c_void) as INT) as HBRUSH;
        }
        wc.hbrBackground = hBrush;
        wc.lpszMenuName = core::ptr::null_mut();
        wc.lpszClassName = ClassName.Buffer;
        wc.hIconSm = core::ptr::null_mut();

        let Class: PCLS = IntCreateClass(
            core::ptr::addr_of!(wc),
            core::ptr::addr_of_mut!(ClassName),
            core::ptr::addr_of_mut!(ClassName),
            core::ptr::addr_of_mut!(MenuName),
            (*entry).fiId as u32,
            Flags,
            core::ptr::null_mut(),
            ppi,
        );

        if !Class.is_null() {
            (*Class).pclsNext = (*ppi).pclsPublicList;
            InterlockedExchangePointer(
                core::ptr::addr_of_mut!((*ppi).pclsPublicList) as *mut *mut core::ffi::c_void,
                Class as *mut core::ffi::c_void,
            );

            (*ppi).dwRegisteredClasses |= ICLASS_TO_MASK((*entry).iCls as u32);
        } else {
            ERR!("!!! Registering system class failed!\n");
            Ret = 0;
        }

        i += 1;
    }

    if Ret != 0 {
        *ppi_w32pf_flags(ppi) |= W32PF_CLASSESREGISTERED;
    }
    Ret
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn UserRegisterSystemClasses() -> BOOL {
    let mut ClassName: _UNICODE_STRING = core::mem::zeroed();
    let mut MenuName: _UNICODE_STRING = core::mem::zeroed();
    let ppi: PPROCESSINFO = GetW32ProcessInfo();
    let mut wc: WNDCLASSEXW = core::mem::zeroed();
    let mut Ret: BOOL = 1;
    let mut Flags: u32 = 0;

    if ((*ppi_w32pf_flags(ppi)) & W32PF_CLASSESREGISTERED) != 0 {
        return 1;
    }


    if hModClient.is_null() {
        return 0;
    }

    RtlZeroMemory(
        core::ptr::addr_of_mut!(ClassName) as *mut core::ffi::c_void,
        core::mem::size_of::<_UNICODE_STRING>(),
    );
    RtlZeroMemory(
        core::ptr::addr_of_mut!(MenuName) as *mut core::ffi::c_void,
        core::mem::size_of::<_UNICODE_STRING>(),
    );

    let count = DEFAULT_SERVER_CLASSES_COUNT;
    let mut i: u32 = 0;
    while i != count {
        let entry = core::ptr::addr_of!(DefaultServerClasses[i as usize]);

        if IS_ATOM!((*entry).ClassName) == 0 {
            RtlInitUnicodeString(core::ptr::addr_of_mut!(ClassName) as _, (*entry).ClassName);
        } else {
            ClassName.Buffer = (*entry).ClassName;
            ClassName.Length = 0;
            ClassName.MaximumLength = 0;
        }

        wc.cbSize = core::mem::size_of::<WNDCLASSEXW>() as u32;
        wc.style = (*entry).Style;

        Flags |= CSF_SERVERSIDEPROC;

        if (*entry).ProcW.is_some() || !core::mem::transmute::<_, *mut core::ffi::c_void>((*entry).ProcW).is_null() {
            wc.lpfnWndProc = (*entry).ProcW;
            wc.hInstance = hModuleWin as _;
        } else {
            wc.lpfnWndProc = GETPFNSERVER((*entry).fiId as INT);
            wc.hInstance = hModClient;
        }

        wc.cbClsExtra = 0;
        wc.cbWndExtra = (*entry).ExtraBytes as INT;
        wc.hIcon = core::ptr::null_mut();

        wc.hCursor = core::ptr::null_mut();
        if (*entry).hCursor == OCR_NORMAL as HICON {
            if SYSTEMCUR_ARROW().is_null() {
                // Only a defect once the cursors are actually loaded. Before that
                // it is the expected state and NOT worth an ERR: system cursors
                // load via a user-mode callback, which cannot be made from the
                // system desktop thread (its process has no KernelCallbackTable),
                // so nothing can populate gasyscur until the first real GUI process
                // exists. Every process registering before that legitimately gets
                // NULL. Measured on an i386 boot: 7 such processes (csrss + six
                // background services) x 6 server-side classes = 42 lines of
                // "should not happen!!" for a state that must happen. Every process
                // that draws visible UI -- winlogon, msgina, explorer -- registers
                // after the load and takes the real cursor.
                if crate::winsta::gbDesktopCursorsAndIconsLoaded != 0 {
                    ERR!("SYSTEMCUR(ARROW) == NULL after the deferred load!!\n");
                }
            } else {
                wc.hCursor = UserHMGetHandle(SYSTEMCUR_ARROW()) as _;
            }
        }

        let mut hBrush: HBRUSH = (*entry).hBrush;
        if (hBrush as usize) <= (COLOR_MENUBAR as usize) {
            hBrush = IntGetSysColorBrush(HandleToUlong(hBrush as *mut core::ffi::c_void) as INT) as HBRUSH;
        }
        wc.hbrBackground = hBrush;
        wc.lpszMenuName = core::ptr::null_mut();
        wc.lpszClassName = ClassName.Buffer;
        wc.hIconSm = core::ptr::null_mut();

        let Class: PCLS = IntCreateClass(
            core::ptr::addr_of!(wc),
            core::ptr::addr_of_mut!(ClassName),
            core::ptr::addr_of_mut!(ClassName),
            core::ptr::addr_of_mut!(MenuName),
            (*entry).fiId as u32,
            Flags,
            core::ptr::null_mut(),
            ppi,
        );

        if !Class.is_null() {
            (*Class).pclsNext = (*ppi).pclsPublicList;
            InterlockedExchangePointer(
                core::ptr::addr_of_mut!((*ppi).pclsPublicList) as *mut *mut core::ffi::c_void,
                Class as *mut core::ffi::c_void,
            );

            (*ppi).dwRegisteredClasses |= ICLASS_TO_MASK((*entry).iCls as u32);
        } else {
            ERR!("!!! Registering system class failed!\n");
            Ret = 0;
        }

        i += 1;
    }

    if Ret != 0 {
        *ppi_w32pf_flags(ppi) |= W32PF_CLASSESREGISTERED;
    }
    Ret
}
#[no_mangle]
pub unsafe extern "system" fn NtUserRegisterClassExWOW(
    lpwcx: *mut WNDCLASSEXW,
    ClassName: PUNICODE_STRING,
    ClsVersion: PUNICODE_STRING,
    pClassMenuName: PCLSMENUNAME,
    fnID: u32,
    Flags: u32,
    _pWow: LPDWORD,
) -> RTL_ATOM {
    let mut CapturedClassInfo: WNDCLASSEXW = core::mem::zeroed();
    let mut CapturedName: _UNICODE_STRING = core::mem::zeroed();
    let mut CapturedMenuName: _UNICODE_STRING = core::mem::zeroed();
    let mut CapturedVersion: _UNICODE_STRING = core::mem::zeroed();
    let mut Ret: RTL_ATOM = 0;
    let ppi: PPROCESSINFO = GetW32ProcessInfo();
    let mut Exception: u8 = 0;
    // Pool copies of the three string buffers (see capture_string_buffer below);
    // they must outlive UserRegisterClass.
    let mut name_guard: Option<ntkrnl_sys::probe::PoolAllocGuard> = None;
    let mut version_guard: Option<ntkrnl_sys::probe::PoolAllocGuard> = None;
    let mut menu_guard: Option<ntkrnl_sys::probe::PoolAllocGuard> = None;

    if (Flags & !(CSF_ANSIPROC)) != 0 {
        EngSetLastError(ERROR_INVALID_FLAGS);
        return Ret;
    }

    UserEnterExclusive();

    if ((*ppi_w32pf_flags(ppi)) & W32PF_CLASSESREGISTERED) == 0 {
        UserRegisterSystemClasses();
    }

    // ---- dropped _SEH2_TRY/_SEH2_EXCEPT recovered here (class.c:2577) ----
    // Handler shape: ERR(...) + SetLastNtError(code) + `Exception = TRUE`,
    // a FLAG the code below tests (`if (!Exception) { Ret = UserRegisterClass(...) }`).
    // Note this differs from the `EngSetLastError(ERROR_INVALID_PARAMETER); break 'seh;`
    // paths already in this block (the C's `goto InvalidParameter` sites): those
    // are deliberate validation failures, NOT exceptions -- they do NOT set
    // Exception, so UserRegisterClass still runs on that path in the C too
    // (an existing oddity, left exactly as the current code already has it).
    'seh: {
        /* Probe the parameters and basic parameter checks */
        let cbSize = match ProbeForReadUint(core::ptr::addr_of_mut!((*lpwcx).cbSize)) {
            Ok(v) => v,
            Err(st) => {
                ERR!("NtUserRegisterClassExWOW Exception Error!\n");
                SetLastNtError(st);
                Exception = 1;
                break 'seh;
            }
        };
        if cbSize != core::mem::size_of::<WNDCLASSEXW>() as u32 {
            EngSetLastError(ERROR_INVALID_PARAMETER);
            break 'seh;
        }

        // ProbeForRead(lpwcx, sizeof(WNDCLASSEXW), sizeof(ULONG)); RtlCopyMemory.
        // The probe above covered only cbSize; range-check the whole struct.
        if let Err(st) = crate::safe_user_io::copy_from_user(
            core::ptr::addr_of_mut!(CapturedClassInfo) as *mut c_void,
            lpwcx as *const c_void,
            core::mem::size_of::<WNDCLASSEXW>(),
            core::mem::size_of::<ULONG>() as u32,
        ) {
            ERR!("NtUserRegisterClassExWOW Exception Error!\n");
            SetLastNtError(st);
            Exception = 1;
            break 'seh;
        }

        CapturedName = match ProbeForReadUnicodeString(ClassName) {
            Ok(v) => v,
            Err(st) => {
                ERR!("NtUserRegisterClassExWOW Exception Error!\n");
                SetLastNtError(st);
                Exception = 1;
                break 'seh;
            }
        };
        CapturedVersion = match ProbeForReadUnicodeString(ClsVersion) {
            Ok(v) => v,
            Err(st) => {
                ERR!("NtUserRegisterClassExWOW Exception Error!\n");
                SetLastNtError(st);
                Exception = 1;
                break 'seh;
            }
        };

        // ProbeForRead(pClassMenuName, sizeof(CLSMENUNAME), 1), then
        // ProbeForReadUnicodeString(pClassMenuName->pusMenuName): capture the
        // CLSMENUNAME once instead of dereferencing the user pointer.
        let ClassMenuName: CLSMENUNAME = match crate::safe_user_io::capture_user(pClassMenuName as *const CLSMENUNAME) {
            Ok(v) => v,
            Err(st) => {
                ERR!("NtUserRegisterClassExWOW Exception Error!\n");
                SetLastNtError(st);
                Exception = 1;
                break 'seh;
            }
        };
        CapturedMenuName = match ProbeForReadUnicodeString(ClassMenuName.pusMenuName) {
            Ok(v) => v,
            Err(st) => {
                ERR!("NtUserRegisterClassExWOW Exception Error!\n");
                SetLastNtError(st);
                Exception = 1;
                break 'seh;
            }
        };

        if (CapturedName.Length & 1) != 0
            || (CapturedMenuName.Length & 1) != 0
            || CapturedClassInfo.cbClsExtra < 0
            || ((CapturedClassInfo.cbClsExtra as u32
                + CapturedName.Length as u32
                + CapturedMenuName.Length as u32
                + core::mem::size_of::<CLS>() as u32)
                < CapturedClassInfo.cbClsExtra as u32)
            || CapturedClassInfo.cbWndExtra < 0
            || CapturedClassInfo.hInstance.is_null()
        {
            EngSetLastError(ERROR_INVALID_PARAMETER);
            break 'seh;
        }

        if CapturedName.Length != 0 {
            // The C ProbeForReads the buffer here and UserRegisterClass then reads
            // it IN PLACE after the _SEH2_TRY ends (a page freed meanwhile faults
            // there). Probe and hand it a pool copy instead.
            match crate::safe_user_io::capture_string_buffer(
                &mut CapturedName,
                u32::from_le_bytes(*b"Ucrn"),
            ) {
                Ok(guard) => name_guard = guard,
                Err(st) => {
                    ERR!("NtUserRegisterClassExWOW Exception Error!\n");
                    SetLastNtError(st);
                    Exception = 1;
                    break 'seh;
                }
            }
        } else {
            if IS_ATOM!(CapturedName.Buffer) == 0 {
                EngSetLastError(ERROR_INVALID_PARAMETER);
                break 'seh;
            }
        }

        if CapturedVersion.Length != 0 {
            // The C ProbeForReads the buffer here and UserRegisterClass then reads
            // it IN PLACE after the _SEH2_TRY ends (a page freed meanwhile faults
            // there). Probe and hand it a pool copy instead.
            match crate::safe_user_io::capture_string_buffer(
                &mut CapturedVersion,
                u32::from_le_bytes(*b"Ucrv"),
            ) {
                Ok(guard) => version_guard = guard,
                Err(st) => {
                    ERR!("NtUserRegisterClassExWOW Exception Error!\n");
                    SetLastNtError(st);
                    Exception = 1;
                    break 'seh;
                }
            }
        } else {
            if IS_ATOM!(CapturedVersion.Buffer) == 0 {
                EngSetLastError(ERROR_INVALID_PARAMETER);
                break 'seh;
            }
        }

        if CapturedMenuName.Length != 0 {
            // The C ProbeForReads the buffer here and UserRegisterClass then reads
            // it IN PLACE after the _SEH2_TRY ends (a page freed meanwhile faults
            // there). Probe and hand it a pool copy instead.
            match crate::safe_user_io::capture_string_buffer(
                &mut CapturedMenuName,
                u32::from_le_bytes(*b"Ucrm"),
            ) {
                Ok(guard) => menu_guard = guard,
                Err(st) => {
                    ERR!("NtUserRegisterClassExWOW Exception Error!\n");
                    SetLastNtError(st);
                    Exception = 1;
                    break 'seh;
                }
            }
        } else if !CapturedMenuName.Buffer.is_null()
            && IS_INTRESOURCE!(CapturedMenuName.Buffer) == 0
        {
            EngSetLastError(ERROR_INVALID_PARAMETER);
            break 'seh;
        }

        if IsCallProcHandle(CapturedClassInfo.lpfnWndProc) != 0 {
            // Never seen this yet
        }
    }

    if Exception == 0 {
        Ret = UserRegisterClass(
            core::ptr::addr_of_mut!(CapturedClassInfo),
            core::ptr::addr_of_mut!(CapturedName),
            core::ptr::addr_of_mut!(CapturedVersion),
            core::ptr::addr_of_mut!(CapturedMenuName),
            fnID,
            Flags,
        );
    }

    UserLeave();

    Ret
}

#[no_mangle]
pub unsafe extern "system" fn IntNtUserSetClassLongPtr(
    hWnd: HWND,
    Offset: INT,
    mut dwNewLong: ULONG_PTR,
    Ansi: BOOL,
    Size: ULONG,
) -> ULONG_PTR {
    let pi: PPROCESSINFO;
    let Window: PWND;
    let mut Ret: ULONG_PTR = 0;

    UserEnterExclusive();

    pi = GetW32ProcessInfo();

    Window = UserGetWindowObject(hWnd);
    if !Window.is_null() {
        'cleanup: {
            if (*head_pti(core::ptr::addr_of!((*Window).head))).ppi != pi {
                EngSetLastError(ERROR_ACCESS_DENIED);
                break 'cleanup;
            }

            // ---- dropped _SEH2_TRY/_SEH2_EXCEPT recovered here (class.c:2692) ----
            // Handler shape: `SetLastNtError(code);` only, falling through to
            // `Cleanup: UserLeave(); return Ret;` (Ret stays whatever it was,
            // 0 here since this is before UserSetClassLongPtr runs).
            'try_block: {
                let mut Value: _UNICODE_STRING = core::mem::zeroed();

                /* Probe the parameters */
                if Offset == GCW_ATOM as INT || Offset == GCLP_MENUNAME as INT {
                    if IS_ATOM!(dwNewLong) != 0 {
                        Value.MaximumLength = 0;
                        Value.Length = 0;
                        Value.Buffer = dwNewLong as PWSTR;
                    } else {
                        Value = match ProbeForReadUnicodeString(dwNewLong as PUNICODE_STRING) {
                            Ok(v) => v,
                            Err(st) => {
                                SetLastNtError(st);
                                break 'try_block;
                            }
                        };
                    }

                    if Value.Length & 1 != 0 {
                        EngSetLastError(ERROR_INVALID_PARAMETER);
                        break 'try_block;
                    }

                    if Value.Length != 0 {
                        // ProbeForRead(Value.Buffer, Value.Length, sizeof(WCHAR)) --
                        // probe-only (no capture), so SafeProbeForRead (real,
                        // non-raising kernel routine) instead of the raw
                        // raising `ProbeForRead` extern.
                        if SafeProbeForRead(
                            Value.Buffer as PVOID,
                            Value.Length as SIZE_T,
                            core::mem::size_of::<u16>() as u32,
                        ) < 0
                        {
                            SetLastNtError(STATUS_ACCESS_VIOLATION);
                            break 'try_block;
                        }
                    } else {
                        if Offset == GCW_ATOM as INT && IS_ATOM!(Value.Buffer as ULONG_PTR) == 0 {
                            EngSetLastError(ERROR_INVALID_PARAMETER);
                            break 'try_block;
                        } else if Offset == GCLP_MENUNAME as INT
                            && IS_INTRESOURCE!(Value.Buffer as ULONG_PTR) == 0
                        {
                            EngSetLastError(ERROR_INVALID_PARAMETER);
                            break 'try_block;
                        }
                    }

                    dwNewLong = core::ptr::addr_of_mut!(Value) as ULONG_PTR;
                }

                Ret = UserSetClassLongPtr((*Window).pcls, Offset, dwNewLong, Ansi, Size);
                match Offset {
                    GCLP_HICONSM | GCLP_HICON => {
                        if Ret != 0 && Ret != dwNewLong {
                            UserPaintCaption(Window, DC_ICON as INT);
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    UserLeave();

    Ret
}
#[no_mangle]
pub unsafe extern "system" fn NtUserSetClassLong(
    hWnd: HWND,
    Offset: INT,
    dwNewLong: ULONG,
    Ansi: BOOL,
) -> ULONG_PTR {
    IntNtUserSetClassLongPtr(
        hWnd,
        Offset,
        dwNewLong as ULONG_PTR,
        Ansi,
        core::mem::size_of::<LONG>() as ULONG,
    )
}

#[no_mangle]
pub unsafe extern "system" fn NtUserSetClassLongPtr(
    hWnd: HWND,
    Offset: INT,
    dwNewLong: ULONG_PTR,
    Ansi: BOOL,
) -> ULONG_PTR {
    IntNtUserSetClassLongPtr(hWnd, Offset, dwNewLong, Ansi, core::mem::size_of::<LONG_PTR>() as ULONG)
}
#[no_mangle]
pub unsafe extern "system" fn NtUserSetClassWord(
    hWnd: HWND,
    nIndex: INT,
    wNewWord: WORD,
) -> WORD {
    0
}
#[no_mangle]
pub unsafe extern "system" fn NtUserUnregisterClass(
    ClassNameOrAtom: PUNICODE_STRING,
    hInstance: HINSTANCE,
    pClassMenuName: PCLSMENUNAME,
) -> BOOL {
    let mut SafeClassName: _UNICODE_STRING = core::mem::zeroed();
    let Status: NTSTATUS;
    let Ret: BOOL;

    Status = ProbeAndCaptureUnicodeStringOrAtom(
        core::ptr::addr_of_mut!(SafeClassName),
        ClassNameOrAtom,
    );
    if !NT_SUCCESS(Status) {
        ERR!("Error capturing the class name\n");
        SetLastNtError(Status);
        return 0;
    }

    UserEnterExclusive();

    Ret = UserUnregisterClass(
        core::ptr::addr_of_mut!(SafeClassName),
        hInstance,
        core::ptr::null_mut(),
    );

    UserLeave();

    if !SafeClassName.Buffer.is_null() && IS_ATOM!(SafeClassName.Buffer) == 0 {
        ExFreePoolWithTag(SafeClassName.Buffer as *mut core::ffi::c_void, TAG_STRING);
    }

    Ret
}
#[no_mangle]
pub unsafe extern "system" fn NtUserGetClassInfo(
    mut hInstance: HINSTANCE,
    ClassName: PUNICODE_STRING,
    lpWndClassEx: LPWNDCLASSEXW,
    ppszMenuName: *mut LPWSTR,
    bAnsi: BOOL,
) -> BOOL {
    let mut SafeClassName: _UNICODE_STRING = core::mem::zeroed();
    let mut Safewcexw: WNDCLASSEXW = core::mem::zeroed();
    let mut Class: PCLS = core::ptr::null_mut();
    let mut ClassAtom: RTL_ATOM = 0;
    let ppi: PPROCESSINFO;
    let mut Ret: BOOL = 1;
    let mut Status: NTSTATUS;

    Status = SafeProbeForWrite(
        lpWndClassEx as *mut _,
        core::mem::size_of::<WNDCLASSEXW>() as _,
        core::mem::size_of::<u32>() as _,
    );
    if NT_SUCCESS(Status) {
        Status = SafeProbeForWrite(
            ppszMenuName as *mut _,
            core::mem::size_of::<LPWSTR>() as _,
            core::mem::size_of::<*mut core::ffi::c_void>() as _,
        );
    }
    if !NT_SUCCESS(Status) {
        SetLastNtError(Status);
        return 0;
    }
    // The probe VALIDATES only; this copy is the access. A fault lands in Status,
    // which is checked just below -- one _SEH2_TRY's worth of behaviour.
    if NT_SUCCESS(Status)
        && !crate::copy::publish_to_user(
            &mut Safewcexw as *mut _ as *mut _,
            lpWndClassEx as *const _,
            core::mem::size_of::<WNDCLASSEXW>() as _,
        )
    {
        Status = STATUS_ACCESS_VIOLATION;
    }

    Status = ProbeAndCaptureUnicodeStringOrAtom(&mut SafeClassName, ClassName);
    if !NT_SUCCESS(Status) {
        ERR!("Error capturing the class name\n");
        SetLastNtError(Status);
        return 0;
    }

    if hInstance.is_null() {
        hInstance = hModClient;
    }

    TRACE!(b"GetClassInfo(%wZ, %p)\n\0".as_ptr() as *const _, &SafeClassName, hInstance);

    UserEnterExclusive();

    ppi = GetW32ProcessInfo();
    if ((*ppi_w32pf_flags(ppi)) & W32PF_CLASSESREGISTERED) == 0 {
        UserRegisterSystemClasses();
    }

    ClassAtom = IntGetClassAtom(
        &mut SafeClassName,
        hInstance,
        ppi,
        &mut Class,
        core::ptr::null_mut(),
    );
    if ClassAtom != 0 {
        ClassAtom = (*Class).atomNVClassName;
        Ret = UserGetClassInfo(Class, &mut Safewcexw, bAnsi, hInstance);
    } else {
        EngSetLastError(ERROR_CLASS_DOES_NOT_EXIST);
        Ret = 0;
    }

    UserLeave();

    // ---- dropped _SEH2_TRY/_SEH2_EXCEPT recovered here (class.c:2867) ----
    // Handler shape: `EngSetLastError(ERROR_CLASS_DOES_NOT_EXIST); Ret = FALSE;`
    // -- a FIXED error, NOT the caught exception code (unlike every other
    // site in this file). `lpWndClassEx`/`ppszMenuName` were only
    // range/alignment-checked by SafeProbeForWrite at function entry, which
    // doesn't guarantee the actual store won't fault (page since revoked,
    // etc.) -- that's exactly what this handler existed to catch.
    if Ret != 0 {
        'seh_ret: {
            if !ppszMenuName.is_null() {
                let menu_name_val = Safewcexw.lpszMenuName as LPWSTR;
                if ntkrnl_sys::probe::safe_copy_raw(
                    ppszMenuName as *mut c_void,
                    core::ptr::addr_of!(menu_name_val) as *const c_void,
                    core::mem::size_of::<LPWSTR>(),
                )
                .is_err()
                {
                    EngSetLastError(ERROR_CLASS_DOES_NOT_EXIST);
                    Ret = 0;
                    break 'seh_ret;
                }
            }

            if ntkrnl_sys::probe::safe_copy_raw(
                lpWndClassEx as *mut c_void,
                core::ptr::addr_of!(Safewcexw) as *const c_void,
                core::mem::size_of::<WNDCLASSEXW>(),
            )
            .is_err()
            {
                EngSetLastError(ERROR_CLASS_DOES_NOT_EXIST);
                Ret = 0;
                break 'seh_ret;
            }

            Ret = ClassAtom as BOOL;
        }
    }

    if IS_ATOM!(SafeClassName.Buffer) == 0 {
        ExFreePoolWithTag(SafeClassName.Buffer as *mut _, TAG_STRING);
    }

    Ret
}
#[no_mangle]
pub unsafe extern "system" fn NtUserGetClassName(
    hWnd: HWND,
    Real: BOOL,
    ClassName: PUNICODE_STRING,
) -> INT {
    let Window: PWND;
    let mut CapturedClassName: _UNICODE_STRING = core::mem::zeroed();
    let mut iCls: INT = 0;
    let mut Ret: INT = 0;
    let mut Atom: RTL_ATOM = 0;

    UserEnterShared();

    Window = UserGetWindowObject(hWnd);
    if !Window.is_null() {
        if Real != 0 && (*Window).fnid != 0 && ((*Window).fnid & FNID_DESTROY as u32) == 0 {
            if LookupFnIdToiCls((*Window).fnid as i32, core::ptr::addr_of_mut!(iCls)) != 0 {
                Atom = (*gpsi).atomSysClass[iCls as usize];
            }
        }

        // ---- dropped _SEH2_TRY/_SEH2_EXCEPT recovered here (class.c:2928) ----
        // Handler shape: `SetLastNtError(code);` only, falling through to
        // `UserLeave(); return Ret;` with Ret left as whatever it already was.
        'seh: {
            if let Err(st) = ProbeForWriteUnicodeString(ClassName) {
                SetLastNtError(st);
                break 'seh;
            }
            // `CapturedClassName = *ClassName` in the C -- `ClassName` here is
            // the raw syscall argument, so this bare dereference is exactly
            // probe_read_checked's shape (probe then capture a T by value).
            CapturedClassName = match probe_read_checked(ClassName as *const _UNICODE_STRING) {
                Ok(v) => v,
                Err(st) => {
                    SetLastNtError(st);
                    break 'seh;
                }
            };
            if CapturedClassName.Length != 0 {
                // ProbeForRead(CapturedClassName.Buffer, CapturedClassName.Length,
                // sizeof(WCHAR)) -- probe-only, so SafeProbeForRead.
                if SafeProbeForRead(
                    CapturedClassName.Buffer as PVOID,
                    CapturedClassName.Length as SIZE_T,
                    core::mem::size_of::<u16>() as u32,
                ) < 0
                {
                    SetLastNtError(STATUS_ACCESS_VIOLATION);
                    break 'seh;
                }
            }

            Ret = UserGetClassName(
                (*Window).pcls,
                core::ptr::addr_of_mut!(CapturedClassName),
                Atom,
                0,
            );

            if Ret != 0 {
                // `ClassName->Length = CapturedClassName.Length` in the C --
                // a write to the caller's raw pointer, still inside the same
                // guarded region. On failure here the C's handler only calls
                // SetLastNtError; Ret keeps whatever UserGetClassName
                // returned (not reset), matching that exactly.
                if ntkrnl_sys::probe::safe_copy_raw(
                    core::ptr::addr_of_mut!((*ClassName).Length) as *mut c_void,
                    core::ptr::addr_of!(CapturedClassName.Length) as *const c_void,
                    core::mem::size_of::<USHORT>(),
                )
                .is_err()
                {
                    SetLastNtError(STATUS_ACCESS_VIOLATION);
                    break 'seh;
                }
            }
        }
    }

    UserLeave();

    Ret
}
#[no_mangle]
pub unsafe extern "system" fn NtUserGetWOWClass(
    hInstance: HINSTANCE,
    ClassName: PUNICODE_STRING,
) -> PCLS {
    let mut SafeClassName: _UNICODE_STRING = core::mem::zeroed();
    let pi: PPROCESSINFO;
    let mut Class: PCLS = core::ptr::null_mut();
    let mut ClassAtom: RTL_ATOM;
    let Status: NTSTATUS;

    Status = ProbeAndCaptureUnicodeStringOrAtom(&mut SafeClassName, ClassName);
    if !NT_SUCCESS(Status) {
        SetLastNtError(Status);
        return core::ptr::null_mut();
    }

    UserEnterExclusive();

    pi = GetW32ProcessInfo();

    ClassAtom = IntGetClassAtom(
        &mut SafeClassName,
        hInstance,
        pi,
        &mut Class,
        core::ptr::null_mut(),
    );
    if ClassAtom == 0 {
        EngSetLastError(ERROR_CLASS_DOES_NOT_EXIST);
    }

    if !SafeClassName.Buffer.is_null() && IS_ATOM!(SafeClassName.Buffer) == 0 {
        ExFreePoolWithTag(SafeClassName.Buffer as *mut core::ffi::c_void, TAG_STRING);
    }

    UserLeave();

    Class
}
