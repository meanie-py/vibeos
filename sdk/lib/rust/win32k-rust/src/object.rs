use win32k_sys::win32k::*;
use core::ffi::{c_int, c_char};
use ntkrnl_sys::interlocked::{InterlockedIncrement, InterlockedDecrement};

// w! — NUL-terminated UTF-16 array from an ASCII literal (2-aligned 'static;
// a b"...\0" byte string cast to *const u16 is both ANSI-not-wide and
// potentially odd-aligned — see field manual / basesrv init.rs CSDVersion).
macro_rules! w {
    ($s:literal) => {{
        const __W: &[u16] = &{
            const BYTES: &[u8] = $s.as_bytes();
            let mut out = [0u16; { BYTES.len() + 1 }];
            let mut i = 0;
            while i < BYTES.len() {
                out[i] = BYTES[i] as u16;
                i += 1;
            }
            out
        };
        __W
    }};
}


// ---------------------------------------------------------------------------
// Real exported kernel symbols, re-declared against win32k-sys nominal types.
// (Same pattern as gdiobj.rs.) The symbol/ABI is unchanged; only Rust nominal
// types differ vs ntkrnl-sys's own bindgen.
//
// RtlAllocateHeap / RtlFreeHeap / ExAllocatePoolWithTag / ExFreePoolWithTag
// are NTAPI exports (ntoskrnl.spec: `@ stdcall`). NTSYSAPI/NTKERNELAPI NTAPI
// = stdcall on i386, C on amd64/arm64: extern "system" resolves correctly
// per arch. ExAllocatePoolZero is NOT an export — it is a FORCEINLINE in
// xdk/exfuncs.h (ExAllocatePoolWithTag + RtlZeroMemory), reproduced below.
extern "system" {
    fn RtlAllocateHeap(HeapHandle: PVOID, Flags: ULONG, Size: SIZE_T) -> PVOID;
    fn RtlFreeHeap(HeapHandle: PVOID, Flags: ULONG, BaseAddress: PVOID) -> BOOLEAN;
    fn ExAllocatePoolWithTag(PoolType: POOL_TYPE, NumberOfBytes: SIZE_T, Tag: ULONG) -> PVOID;
    fn ExFreePoolWithTag(P: PVOID, Tag: ULONG);
}

/// ExAllocatePoolZero — xdk/exfuncs.h FORCEINLINE:
/// `p = ExAllocatePoolWithTag(...); if (p) RtlZeroMemory(p, n); return p;`
#[inline(always)]
unsafe fn ExAllocatePoolZero(PoolType: POOL_TYPE, NumberOfBytes: SIZE_T, Tag: ULONG) -> PVOID {
    let allocation = ExAllocatePoolWithTag(PoolType, NumberOfBytes, Tag);
    if !allocation.is_null() {
        core::ptr::write_bytes(allocation as *mut u8, 0, NumberOfBytes as usize);
    }
    allocation
}

extern "C" {
    fn DbgPrint(Format: *const c_char, ...) -> u32;
}

// win32k-internal helpers NOT in the win32k-sys bindgen surface because they
// are file-local-ish (defined in other .c TUs, prototyped only in private
// headers): SetLastNtError is `VOID FASTCALL` (win32ss/gdi/ntgdi/misc.h:76);
// the cursoricon/callproc/ime allocators+destructors are plain cdecl object
// callbacks. i386 fastcall = @Name@N; everything else uniformly C.
#[cfg(target_arch = "x86")]
extern "fastcall" {
    fn SetLastNtError(Status: NTSTATUS);
}
#[cfg(not(target_arch = "x86"))]
extern "C" {
    fn SetLastNtError(Status: NTSTATUS);
}

// Object-table callbacks living in other (still-C) win32k TUs, not prototyped
// in <win32k.h> so absent from bindgen. All are plain cdecl, matching the
// `PVOID (*)(PDESKTOP,PTHREADINFO,SIZE_T,PVOID*)` / `BOOLEAN (*)(PVOID)` /
// `void (*)(PVOID)` function-pointer types of the C ObjectCallbacks table.
extern "C" {
    fn FreeCurIconObject(Object: PVOID);
    fn DestroyCallProc(Object: PVOID) -> BOOLEAN;
    fn AllocInputContextObject(
        pDesk: PDESKTOP,
        pti: PTHREADINFO,
        Size: SIZE_T,
        HandleOwner: *mut PVOID,
    ) -> PVOID;
}

// NT status / pool / tag constants not present in the win32k-sys bindgen
// constant surface.
//  - STATUS_INVALID_HANDLE: psdk/ntstatus.h
//  - PagedPool: xdk POOL_TYPE enumerator (NonPagedPool=0, PagedPool=1)
//  - USERTAG_CLIPBOARD: win32ss/user/ntuser/tags.h `'bcsU'`. C multi-char
//    constants are big-endian: 'bcsU' == 0x62637355.
const STATUS_INVALID_HANDLE: NTSTATUS = 0xC0000008u32 as NTSTATUS;
#[allow(non_upper_case_globals)]
const PagedPool: POOL_TYPE = 1;
const USERTAG_CLIPBOARD: u32 = u32::from_be_bytes([b'b', b'c', b's', b'U']);

// HEAP_NO_SERIALIZE (winnt) — RtlAllocateHeap/RtlFreeHeap flag.
const HEAP_NO_SERIALIZE: ULONG = 1;

// ---------------------------------------------------------------------------
// Header static-inline / macro reproductions (faithful, in-crate — these have
// no exported symbol so cannot be `extern`-linked).

/// LOWORD(l) — ((WORD)((DWORD_PTR)(l) & 0xffff)) (minwindef.h).
#[inline(always)]
fn LOWORD(l: usize) -> u16 {
    (l & 0xffff) as u16
}

/// HIWORD(l) — ((WORD)(((DWORD_PTR)(l) >> 16) & 0xffff)) (minwindef.h).
#[inline(always)]
fn HIWORD(l: usize) -> u16 {
    ((l >> 16) & 0xffff) as u16
}

/// RtlZeroMemory(dst, len) — memset(dst, 0, len). `len` is SIZE_T (u32 on
/// i386, u64 on amd64/arm64); write_bytes wants usize.
#[inline(always)]
unsafe fn RtlZeroMemory(dst: PVOID, len: SIZE_T) {
    core::ptr::write_bytes(dst as *mut u8, 0, len as usize);
}

/// DesktopHeapAlloc — desktop.h static __inline:
/// `RtlAllocateHeap(Desktop->pheapDesktop, HEAP_NO_SERIALIZE, Bytes)`.
#[inline(always)]
unsafe fn DesktopHeapAlloc(Desktop: PDESKTOP, Bytes: SIZE_T) -> PVOID {
    RtlAllocateHeap((*Desktop).pheapDesktop as PVOID, HEAP_NO_SERIALIZE, Bytes)
}

/// DesktopHeapFree — desktop.h static __inline:
/// `RtlFreeHeap(Desktop->pheapDesktop, HEAP_NO_SERIALIZE, lpMem)`.
#[inline(always)]
unsafe fn DesktopHeapFree(Desktop: PDESKTOP, lpMem: PVOID) -> BOOL {
    RtlFreeHeap((*Desktop).pheapDesktop as PVOID, HEAP_NO_SERIALIZE, lpMem) as BOOL
}

/// UserHeapAlloc — usrheap.h static __inline:
/// `RtlAllocateHeap(GlobalUserHeap, HEAP_NO_SERIALIZE, Bytes)`.
#[inline(always)]
unsafe fn UserHeapAlloc(Bytes: SIZE_T) -> PVOID {
    RtlAllocateHeap(GlobalUserHeap as PVOID, HEAP_NO_SERIALIZE, Bytes)
}

/// UserHeapFree — usrheap.h static __inline:
/// `RtlFreeHeap(GlobalUserHeap, HEAP_NO_SERIALIZE, lpMem)`.
#[inline(always)]
unsafe fn UserHeapFree(lpMem: PVOID) -> BOOL {
    RtlFreeHeap(GlobalUserHeap as PVOID, HEAP_NO_SERIALIZE, lpMem) as BOOL
}

/// IntReferenceThreadInfo(pti) — win32.h macro:
/// `InterlockedIncrement(&(pti)->RefCount)`. RefCount lives in the embedded
/// W32THREAD prefix (bindgen padding-collapses _THREADINFO's W32THREAD).
#[inline(always)]
unsafe fn IntReferenceThreadInfo(pti: PTHREADINFO) {
    InterlockedIncrement(core::ptr::addr_of_mut!((*(pti as *mut W32THREAD)).RefCount) as *mut i32);
}

/// IntDereferenceThreadInfo(pti) — win32.h macro: decrement RefCount; on
/// reaching 0, ASSERT(...) (no-op in DBG=0 release) then UserDeleteW32Thread.
#[inline(always)]
unsafe fn IntDereferenceThreadInfo(pti: PTHREADINFO) {
    if InterlockedDecrement(core::ptr::addr_of_mut!((*(pti as *mut W32THREAD)).RefCount) as *mut i32) == 0 {
        UserDeleteW32Thread(pti);
    }
}

/// IntReferenceProcessInfo(ppi) — win32.h macro:
/// `InterlockedIncrement((volatile LONG*)&(ppi)->RefCount)`. RefCount lives in
/// the embedded W32PROCESS prefix.
#[inline(always)]
unsafe fn IntReferenceProcessInfo(ppi: PPROCESSINFO) {
    InterlockedIncrement(core::ptr::addr_of_mut!((*(ppi as *mut W32PROCESS)).RefCount) as *mut i32);
}

/// IntDereferenceProcessInfo(ppi) — win32.h macro: decrement RefCount; on
/// reaching 0, ASSERT(...) (no-op in DBG=0 release) then UserDeleteW32Process.
#[inline(always)]
unsafe fn IntDereferenceProcessInfo(ppi: PPROCESSINFO) {
    if InterlockedDecrement(core::ptr::addr_of_mut!((*(ppi as *mut W32PROCESS)).RefCount) as *mut i32) == 0 {
        UserDeleteW32Process(ppi);
    }
}

// ERR(...) / TRACE(...) — win32kdebug.h DBG=TRUE channel macros.
//   ERR  = DBG_PRINT(..., ERR_LEVEL, ...)  → DbgPrint unconditionally
//          (the `level == ERR_LEVEL` arm of DBG_PRINT always fires).
//   TRACE= DBG_PRINT(..., TRACE_LEVEL, ...) → DbgPrint only if the UserObj
//          channel is enabled; channels are zero-initialized (disabled) and
//          nothing in the boot path enables UserObj, so TRACE is silent at
//          boot. Faithful boot behavior = no-op (no observable difference).
// First macro arg is a NUL-terminated byte-string literal (the C format);
// remaining args are the already-formed variadic values. DbgPrint is cdecl
// variadic. We pass the format verbatim (the C `"err: "`/`"(%s:%d) "`
// prefixes are cosmetic and not behaviourally observable).

// C: PUSER_HANDLE_TABLE gHandleTable = NULL;  — object.c OWNS this exported
// global. win32k-sys bindgen also declares it `extern static mut`; the glob
// `use win32k_sys::win32k::*` therefore brings in an extern decl. A local
// `#[no_mangle]` definition both shadows the glob within this module and
// provides the real symbol for all other (still-C) win32k TUs.
#[no_mangle]
pub static mut gHandleTable: PUSER_HANDLE_TABLE = core::ptr::null_mut();

// The C ObjectCallbacks table uses plain (cdecl) function pointers:
//   PVOID   (*ObjectAlloc)(PDESKTOP, PTHREADINFO, SIZE_T, PVOID*);
//   BOOLEAN (*ObjectDestroy)(PVOID);
//   void    (*ObjectFree)(PVOID);
// All entries (bindgen co_UserDestroyWindow/IntRemoveHook/... and the
// in-crate Alloc*/Free* helpers) are therefore `extern "C"`.
type ObjectAllocFn = unsafe extern "C" fn(PDESKTOP, PTHREADINFO, SIZE_T, *mut PVOID) -> PVOID;
type ObjectDestroyFn = unsafe extern "C" fn(PVOID) -> BOOLEAN;
type ObjectFreeFn = unsafe extern "C" fn(PVOID);

#[repr(C)]
struct ObjectCallbackEntry {
    ObjectAlloc: Option<ObjectAllocFn>,
    ObjectDestroy: Option<ObjectDestroyFn>,
    ObjectFree: Option<ObjectFreeFn>,
}

static ObjectCallbacks: [ObjectCallbackEntry; TYPE_CTYPES as usize] = [
    ObjectCallbackEntry { ObjectAlloc: None, ObjectDestroy: None, ObjectFree: None },
    ObjectCallbackEntry { ObjectAlloc: Some(AllocDeskThreadObject), ObjectDestroy: Some(co_UserDestroyWindow), ObjectFree: Some(FreeDeskThreadObject) },
    ObjectCallbackEntry { ObjectAlloc: Some(AllocDeskProcObject), ObjectDestroy: Some(UserDestroyMenuObject), ObjectFree: Some(FreeDeskProcObject) },
    ObjectCallbackEntry { ObjectAlloc: Some(AllocProcMarkObject), ObjectDestroy: Some(IntDestroyCurIconObject), ObjectFree: Some(FreeCurIconObject) },
    ObjectCallbackEntry { ObjectAlloc: Some(AllocSysObject), ObjectDestroy: None, ObjectFree: Some(FreeSysObject) },
    ObjectCallbackEntry { ObjectAlloc: Some(AllocDeskThreadObject), ObjectDestroy: Some(IntRemoveHook), ObjectFree: Some(FreeDeskThreadObject) },
    ObjectCallbackEntry { ObjectAlloc: Some(AllocSysObjectCB), ObjectDestroy: None, ObjectFree: Some(FreeSysObjectCB) },
    ObjectCallbackEntry { ObjectAlloc: Some(AllocDeskProcObject), ObjectDestroy: Some(DestroyCallProc), ObjectFree: Some(FreeDeskProcObject) },
    ObjectCallbackEntry { ObjectAlloc: Some(AllocProcMarkObject), ObjectDestroy: Some(UserDestroyAccelTable), ObjectFree: Some(FreeProcMarkObject) },
    ObjectCallbackEntry { ObjectAlloc: None, ObjectDestroy: None, ObjectFree: None },
    ObjectCallbackEntry { ObjectAlloc: None, ObjectDestroy: None, ObjectFree: None },
    ObjectCallbackEntry { ObjectAlloc: None, ObjectDestroy: None, ObjectFree: None },
    ObjectCallbackEntry { ObjectAlloc: Some(AllocSysObject), ObjectDestroy: None, ObjectFree: Some(FreeSysObject) },
    ObjectCallbackEntry { ObjectAlloc: Some(AllocSysObject), ObjectDestroy: None, ObjectFree: Some(FreeSysObject) },
    ObjectCallbackEntry { ObjectAlloc: Some(AllocSysObject), ObjectDestroy: None, ObjectFree: Some(FreeSysObject) },
    ObjectCallbackEntry { ObjectAlloc: Some(AllocThreadObject), ObjectDestroy: Some(IntRemoveEvent), ObjectFree: Some(FreeThreadObject) },
    ObjectCallbackEntry { ObjectAlloc: Some(AllocSysObject), ObjectDestroy: None, ObjectFree: Some(FreeSysObject) },
    ObjectCallbackEntry { ObjectAlloc: Some(AllocInputContextObject), ObjectDestroy: Some(UserDestroyInputContext), ObjectFree: Some(UserFreeInputContext) },
    ObjectCallbackEntry { ObjectAlloc: None, ObjectDestroy: None, ObjectFree: None },
    ObjectCallbackEntry { ObjectAlloc: None, ObjectDestroy: None, ObjectFree: None },
    ObjectCallbackEntry { ObjectAlloc: None, ObjectDestroy: None, ObjectFree: None },
    ObjectCallbackEntry { ObjectAlloc: None, ObjectDestroy: None, ObjectFree: None },
];

// Helpers that read `UserHandleCount` through the W32PROCESS prefix embedded
// in PROCESSINFO (bindgen padding-collapses the anonymous W32PROCESS member;
// the C reaches it via name injection as `ppi->UserHandleCount`).
#[inline(always)]
unsafe fn ppi_user_handle_count(ppi: PPROCESSINFO) -> *mut LONG {
    core::ptr::addr_of_mut!((*(ppi as *mut W32PROCESS)).UserHandleCount)
}

unsafe extern "C" fn AllocThreadObject(
    _pDesk: PDESKTOP,
    pti: PTHREADINFO,
    Size: SIZE_T,
    HandleOwner: *mut PVOID,
) -> PVOID {
    // UNREFERENCED_PARAMETER(pDesk);  // ((void)(pDesk)) — no-op
    let ObjHead: PTHROBJHEAD;

    ObjHead = UserHeapAlloc(Size) as PTHROBJHEAD;
    if ObjHead.is_null() {
        return core::ptr::null_mut();
    }

    RtlZeroMemory(ObjHead as PVOID, Size);

    (*ObjHead).pti = pti as *mut _THREADINFO;
    IntReferenceThreadInfo(pti);
    *HandleOwner = pti as PVOID;
    /* It's a thread object, but it still count as one for the process */
    let c = ppi_user_handle_count((*pti).ppi);
    *c = (*c).wrapping_add(1);

    return ObjHead as PVOID;
}

unsafe extern "C" fn FreeThreadObject(Object: PVOID) {
    let ObjHead = Object as PTHROBJHEAD;
    let pti = (*ObjHead).pti as PTHREADINFO;

    UserHeapFree(ObjHead as PVOID);

    let c = ppi_user_handle_count((*pti).ppi);
    *c = (*c).wrapping_sub(1);
    IntDereferenceThreadInfo(pti);
}

unsafe extern "C" fn AllocDeskThreadObject(
    pDesk: PDESKTOP,
    pti: PTHREADINFO,
    Size: SIZE_T,
    HandleOwner: *mut PVOID,
) -> PVOID {

    let mut pDesk = pDesk;
    let ObjHead: PTHRDESKHEAD;

    if pDesk.is_null() {
        pDesk = (*pti).rpdesk;
    }

    ObjHead = DesktopHeapAlloc(pDesk, Size) as PTHRDESKHEAD;
    if ObjHead.is_null() {
        return core::ptr::null_mut();
    }

    RtlZeroMemory(ObjHead as PVOID, Size);

    (*ObjHead).pSelf = ObjHead as PVOID;
    (*ObjHead).rpdesk = pDesk as *mut _DESKTOP;
    // ObjHead->pti — from the embedded THROBJHEAD (offset 8), reached via
    // the THROBJHEAD view since bindgen collapses the anon member.
    (*(ObjHead as PTHROBJHEAD)).pti = pti as *mut _THREADINFO;
    IntReferenceThreadInfo(pti);
    *HandleOwner = pti as PVOID;
    let c = ppi_user_handle_count((*pti).ppi);
    *c = (*c).wrapping_add(1);

    ObjHead as PVOID
}

unsafe extern "C" fn FreeDeskThreadObject(Object: PVOID) {
    let ObjHead = Object as PTHRDESKHEAD;
    let pDesk = (*ObjHead).rpdesk as PDESKTOP;
    let pti = (*(ObjHead as PTHROBJHEAD)).pti as PTHREADINFO;

    DesktopHeapFree(pDesk, Object);

    let c = ppi_user_handle_count((*pti).ppi);
    *c = (*c).wrapping_sub(1);
    IntDereferenceThreadInfo(pti);
}

unsafe extern "C" fn AllocDeskProcObject(
    pDesk: PDESKTOP,
    pti: PTHREADINFO,
    Size: SIZE_T,
    HandleOwner: *mut PVOID,
) -> PVOID {

    let ObjHead: PPROCDESKHEAD = DesktopHeapAlloc(pDesk, Size) as PPROCDESKHEAD;
    if ObjHead.is_null() {
        return core::ptr::null_mut();
    }

    RtlZeroMemory(ObjHead as PVOID, Size);

    let ppi: PPROCESSINFO = (*pti).ppi;

    (*ObjHead).pSelf = ObjHead as PVOID;
    (*ObjHead).rpdesk = pDesk as *mut _DESKTOP;
    (*ObjHead).hTaskWow = ppi as DWORD_PTR;
    let c = ppi_user_handle_count(ppi);
    *c = (*c).wrapping_add(1);
    IntReferenceProcessInfo(ppi);
    *HandleOwner = ppi as PVOID;

    ObjHead as PVOID
}

unsafe extern "C" fn FreeDeskProcObject(Object: PVOID) {
    let ObjHead = Object as PPROCDESKHEAD;
    let pDesk = (*ObjHead).rpdesk as PDESKTOP;
    let ppi = (*ObjHead).hTaskWow as PPROCESSINFO;

    let c = ppi_user_handle_count(ppi);
    *c = (*c).wrapping_sub(1);
    IntDereferenceProcessInfo(ppi);

    DesktopHeapFree(pDesk, Object);
}

unsafe extern "C" fn AllocProcMarkObject(
    _pDesk: PDESKTOP,
    pti: PTHREADINFO,
    Size: SIZE_T,
    HandleOwner: *mut PVOID,
) -> PVOID {
    let ppi: PPROCESSINFO = (*pti).ppi;

    // UNREFERENCED_PARAMETER(pDesk);  // ((void)(pDesk)) — no-op

    let ObjHead: PPROCMARKHEAD = UserHeapAlloc(Size) as PPROCMARKHEAD;
    if ObjHead.is_null() {
        return core::ptr::null_mut();
    }

    RtlZeroMemory(ObjHead as PVOID, Size);

    (*ObjHead).ppi = ppi as *mut _PROCESSINFO;
    IntReferenceProcessInfo(ppi);
    *HandleOwner = ppi as PVOID;
    let c = ppi_user_handle_count(ppi);
    *c = (*c).wrapping_add(1);

    ObjHead as PVOID
}

// C: void FreeProcMarkObject(PVOID Object)  — NOT static in object.c; called
// cross-TU (cursoricon.c FreeCurIconObject). cdecl.
#[no_mangle]
pub unsafe extern "C" fn FreeProcMarkObject(Object: PVOID) {

    let ppi = (*(Object as PPROCMARKHEAD)).ppi as PPROCESSINFO;

    UserHeapFree(Object);

    let c = ppi_user_handle_count(ppi);
    *c = (*c).wrapping_sub(1);
    IntDereferenceProcessInfo(ppi);
}

unsafe extern "C" fn AllocSysObject(
    _pDesk: PDESKTOP,
    _pti: PTHREADINFO,
    Size: SIZE_T,
    ObjectOwner: *mut PVOID,
) -> PVOID {

    // UNREFERENCED_PARAMETER(pDesk); UNREFERENCED_PARAMETER(pti);  // no-ops
    let Object: PVOID;

    Object = UserHeapAlloc(Size);
    if Object.is_null() {
        return core::ptr::null_mut();
    }

    *ObjectOwner = core::ptr::null_mut();

    RtlZeroMemory(Object, Size);
    return Object;
}

unsafe extern "C" fn AllocSysObjectCB(
    _pDesk: PDESKTOP,
    _pti: PTHREADINFO,
    Size: SIZE_T,
    ObjectOwner: *mut PVOID,
) -> PVOID {
    // UNREFERENCED_PARAMETER(pDesk); UNREFERENCED_PARAMETER(pti);  // no-ops
    let Object: PVOID;

    /* Allocate the clipboard data */
    Object = ExAllocatePoolZero(PagedPool, Size, USERTAG_CLIPBOARD);
    if Object.is_null() {
        ERR!("ExAllocatePoolZero failed. No object created.\n");
        return core::ptr::null_mut();
    }

    *ObjectOwner = core::ptr::null_mut();
    Object
}

unsafe extern "C" fn FreeSysObject(Object: PVOID) {
    UserHeapFree(Object);
}

unsafe extern "C" fn FreeSysObjectCB(Object: PVOID) {
    ExFreePoolWithTag(Object, USERTAG_CLIPBOARD);
}

// void DbgUserDumpHandleTable(VOID)  — object.c, #if DBG (this is a
// DBG:BOOL=TRUE build), exported (non-static), called from main.c
// (UserDeleteW32Process / UserProcessDestroy) and alloc_user_entry. cdecl.
// peProcess / UserHandleCount live in the W32PROCESS prefix embedded in
// PROCESSINFO (bindgen padding-collapses the anon member); ppiNext /
// DbgHandleCount are direct PROCESSINFO fields.
#[no_mangle]
pub unsafe extern "C" fn DbgUserDumpHandleTable() {

    let mut HandleCounts: [i32; TYPE_CTYPES as usize] = [0; TYPE_CTYPES as usize];
    let mut ppiList: PPROCESSINFO;
    let mut i: i32;
    let TypeNames: [*const u16; 22] = [
        w!("Free").as_ptr() as *const u16,
        w!("Window").as_ptr() as *const u16,
        w!("Menu").as_ptr() as *const u16,
        w!("CursorIcon").as_ptr() as *const u16,
        w!("SMWP").as_ptr() as *const u16,
        w!("Hook").as_ptr() as *const u16,
        w!("ClipBoardData").as_ptr() as *const u16,
        w!("CallProc").as_ptr() as *const u16,
        w!("Accel").as_ptr() as *const u16,
        w!("DDEaccess").as_ptr() as *const u16,
        w!("DDEconv").as_ptr() as *const u16,
        w!("DDExact").as_ptr() as *const u16,
        w!("Monitor").as_ptr() as *const u16,
        w!("KBDlayout").as_ptr() as *const u16,
        w!("KBDfile").as_ptr() as *const u16,
        w!("Event").as_ptr() as *const u16,
        w!("Timer").as_ptr() as *const u16,
        w!("InputContext").as_ptr() as *const u16,
        w!("HidData").as_ptr() as *const u16,
        w!("DeviceInfo").as_ptr() as *const u16,
        w!("TouchInput").as_ptr() as *const u16,
        w!("GestureInfo").as_ptr() as *const u16,
    ];

    ERR!("Total handles count: %lu\n", (*gpsi).cHandleEntries);

    core::ptr::write_bytes(HandleCounts.as_mut_ptr() as *mut u8, 0, core::mem::size_of_val(&HandleCounts));

    ppiList = gppiList;
    while !ppiList.is_null() {
        let peProcess = (*(ppiList as *mut W32PROCESS)).peProcess as *mut _EPROCESS;
        ERR!(
            "Process %s (%p) handles count: %d\n\t",
            (*peProcess).ImageFileName.as_ptr(),
            (*peProcess).UniqueProcessId,
            *ppi_user_handle_count(ppiList)
        );

        i = 1;
        while i < TYPE_CTYPES as i32 {
            HandleCounts[i as usize] += (*ppiList).DbgHandleCount[i as usize] as i32;

            DbgPrint(
                b"%S: %lu, \0".as_ptr() as *const c_char,
                TypeNames[i as usize],
                (*ppiList).DbgHandleCount[i as usize]
            );
            if i % 6 == 0 {
                DbgPrint(b"\n\t\0".as_ptr() as *const c_char);
            }
            i += 1;
        }
        DbgPrint(b"\n\0".as_ptr() as *const c_char);

        ppiList = (*ppiList).ppiNext;
    }

    ERR!("Total handles of the running processes: \n\t");
    i = 1;
    while i < TYPE_CTYPES as i32 {
        DbgPrint(
            b"%S: %d, \0".as_ptr() as *const c_char,
            TypeNames[i as usize],
            HandleCounts[i as usize]
        );
        if i % 6 == 0 {
            DbgPrint(b"\n\t\0".as_ptr() as *const c_char);
        }
        i += 1;
    }
    DbgPrint(b"\n\0".as_ptr() as *const c_char);

    core::ptr::write_bytes(HandleCounts.as_mut_ptr() as *mut u8, 0, core::mem::size_of_val(&HandleCounts));
    i = 0;
    while i < (*gHandleTable).nb_handles {
        let e = (*gHandleTable).handles.offset(i as isize);
        HandleCounts[entry_type(e) as usize] += 1;
        i += 1;
    }

    ERR!("Total handles count allocated: \n\t");
    i = 1;
    while i < TYPE_CTYPES as i32 {
        DbgPrint(
            b"%S: %d, \0".as_ptr() as *const c_char,
            TypeNames[i as usize],
            HandleCounts[i as usize]
        );
        if i % 6 == 0 {
            DbgPrint(b"\n\t\0".as_ptr() as *const c_char);
        }
        i += 1;
    }
    DbgPrint(b"\n\0".as_ptr() as *const c_char);
}

// ---------------------------------------------------------------------------
// captures(none) hardening (project canonical fix — feedback_volatile_static /
// captures(none) class; proven necessary by gdiobj.rs Phase 1a). The USER
// handle table entries (`type_`, `generation`, `flags`, `ptr`, `pi`) are read
// here and in the entry-validation/refcount path while *sibling* functions
// (alloc_user_entry / free_user_entry / UserAllocHandle / UserMarkObjectDestroy
// / UserFreeHandle) write them. Across the foreign-ABI callback boundary LLVM
// (esp. i386) would otherwise hoist these loads, so each entry-field read that
// a sibling can mutate uses read_volatile, and the paired mutations use
// write_volatile — faithful to C's per-access memory semantics, no logic
// change.
#[inline(always)]
unsafe fn entry_type(e: PUSER_HANDLE_ENTRY) -> u8 {
    core::ptr::read_volatile(core::ptr::addr_of!((*e).type_))
}
#[inline(always)]
unsafe fn entry_generation(e: PUSER_HANDLE_ENTRY) -> u16 {
    core::ptr::read_volatile(core::ptr::addr_of!((*e).generation))
}
#[inline(always)]
unsafe fn entry_flags(e: PUSER_HANDLE_ENTRY) -> u8 {
    core::ptr::read_volatile(core::ptr::addr_of!((*e).flags))
}
#[inline(always)]
unsafe fn entry_ptr(e: PUSER_HANDLE_ENTRY) -> PVOID {
    core::ptr::read_volatile(core::ptr::addr_of!((*e).ptr))
}
#[inline(always)]
unsafe fn entry_pi(e: PUSER_HANDLE_ENTRY) -> PVOID {
    // The `pi`/`pti`/`ppi` overlap is a C anonymous union; bindgen lowers it
    // to `__bindgen_anon_1.pi` (all three alias the same PVOID slot).
    core::ptr::read_volatile(core::ptr::addr_of!((*e).__bindgen_anon_1.pi))
}

unsafe fn handle_to_entry(ht: PUSER_HANDLE_TABLE, handle: HANDLE) -> PUSER_HANDLE_ENTRY {

    let generation: u16;
    // C: int index = (LOWORD(handle) - FIRST_USER_HANDLE) >> 1;
    // LOWORD is WORD, promoted to int before the subtraction.
    let index: i32 = ((LOWORD(handle as usize) as i32) - FIRST_USER_HANDLE as i32) >> 1;
    if index < 0 || index >= (*ht).nb_handles {
        return core::ptr::null_mut();
    }
    let entry = (*ht).handles.offset(index as isize);
    if entry_type(entry) == 0 {
        return core::ptr::null_mut();
    }
    generation = HIWORD(handle as usize);
    if generation == entry_generation(entry) || generation == 0 || generation == 0xffff {
        return entry;
    }
    core::ptr::null_mut()
}

unsafe fn entry_to_handle(ht: PUSER_HANDLE_TABLE, ptr: PUSER_HANDLE_ENTRY) -> HANDLE {
    // C: int index = ptr - ht->handles;  (element-count pointer subtraction)
    let index = ptr.offset_from((*ht).handles) as INT_PTR;
    ((index << 1) + FIRST_USER_HANDLE as INT_PTR
        + ((entry_generation(ptr) as INT_PTR) << 16)) as HANDLE
}

unsafe fn alloc_user_entry(ht: PUSER_HANDLE_TABLE) -> PUSER_HANDLE_ENTRY {

    let entry: PUSER_HANDLE_ENTRY;
    TRACE!(b"handles used %lu\n\0", (*gpsi).cHandleEntries);

    if !(*ht).freelist.is_null() {
        entry = (*ht).freelist;
        // entry->ptr is the freelist `next` link (free_user_entry stored it
        // there); read it volatile to mirror C's reload after sibling writes.
        (*ht).freelist = entry_ptr(entry) as PUSER_HANDLE_ENTRY;

        (*gpsi).cHandleEntries += 1;
        return entry;
    }

    if (*ht).nb_handles >= (*ht).allocated_handles {
        ERR!(
            "Out of user handles! Used -> %lu, NM_Handle -> %d\n",
            (*gpsi).cHandleEntries,
            (*ht).nb_handles
        );

        // #if DBG DbgUserDumpHandleTable(); #endif  — DBG=TRUE build: live.
        DbgUserDumpHandleTable();

        return core::ptr::null_mut();
    }

    entry = (*ht).handles.offset((*ht).nb_handles as isize);
    (*ht).nb_handles += 1;

    core::ptr::write_volatile(core::ptr::addr_of_mut!((*entry).generation), 1);

    (*gpsi).cHandleEntries += 1;

    return entry;
}

unsafe fn UserInitHandleTable(ht: PUSER_HANDLE_TABLE, mem: PVOID, bytes: ULONG) {
    (*ht).freelist = core::ptr::null_mut();
    (*ht).handles = mem as PUSER_HANDLE_ENTRY;

    (*ht).nb_handles = 0;
    (*ht).allocated_handles =
        (bytes / core::mem::size_of::<USER_HANDLE_ENTRY>() as ULONG) as c_int;
}

unsafe fn free_user_entry(ht: PUSER_HANDLE_TABLE, entry: PUSER_HANDLE_ENTRY) -> PVOID {
    let ret: PVOID;

    // #if DBG { ... DbgHandleCount-- ... } — DBG=0 release: entire block is
    // dead (PROCESSINFO::DbgHandleCount is DBG-only). Faithful no-op.

    // The slot is about to be reused, so no job may keep a grant on it (upstream b272427baa3).
    if entry_flags(entry) & HANDLEENTRY_GRANTED as u8 != 0 {
        crate::job::IntCleanupGrantedHandle(entry_to_handle(ht, entry));
    }

    ret = entry_ptr(entry);
    core::ptr::write_volatile(core::ptr::addr_of_mut!((*entry).ptr), (*ht).freelist as PVOID);
    core::ptr::write_volatile(core::ptr::addr_of_mut!((*entry).type_), 0);
    core::ptr::write_volatile(core::ptr::addr_of_mut!((*entry).flags), 0);
    core::ptr::write_volatile(core::ptr::addr_of_mut!((*entry).__bindgen_anon_1.pi), core::ptr::null_mut());
    (*ht).freelist = entry;

    (*gpsi).cHandleEntries -= 1;

    ret
}

unsafe fn UserAllocHandle(
    ht: PUSER_HANDLE_TABLE,
    object: PVOID,
    type_: HANDLE_TYPE,
    HandleOwner: PVOID,
) -> HANDLE {
    let entry = alloc_user_entry(ht);
    if entry.is_null() {
        return 0 as HANDLE;
    }
    core::ptr::write_volatile(core::ptr::addr_of_mut!((*entry).ptr), object);
    core::ptr::write_volatile(core::ptr::addr_of_mut!((*entry).type_), type_ as u8);
    core::ptr::write_volatile(core::ptr::addr_of_mut!((*entry).flags), 0);
    core::ptr::write_volatile(core::ptr::addr_of_mut!((*entry).__bindgen_anon_1.pi), HandleOwner);
    // C: if (++entry->generation >= 0xffff) entry->generation = 1;
    let g = entry_generation(entry).wrapping_add(1);
    core::ptr::write_volatile(core::ptr::addr_of_mut!((*entry).generation), g);
    if g >= 0xffff {
        core::ptr::write_volatile(core::ptr::addr_of_mut!((*entry).generation), 1);
    }

    /* We have created a handle, which is a reference! */
    UserReferenceObject(object);

    entry_to_handle(ht, entry)
}

// PVOID UserGetObjectNoErr(PUSER_HANDLE_TABLE, HANDLE, HANDLE_TYPE);
// object.h — plain cdecl, called cross-TU.
#[no_mangle]
pub unsafe extern "C" fn UserGetObjectNoErr(ht: PUSER_HANDLE_TABLE, handle: HANDLE, type_: HANDLE_TYPE) -> PVOID {
    let entry: PUSER_HANDLE_ENTRY;

    // ASSERT(ht); — no-op in DBG=0 release.
    entry = handle_to_entry(ht, handle);
    if entry.is_null() || entry_type(entry) as HANDLE_TYPE != type_ {
        return core::ptr::null_mut();
    }
    return entry_ptr(entry);
}

// PVOID UserGetObject(PUSER_HANDLE_TABLE ht, HANDLE handle, HANDLE_TYPE type);
// object.h — plain cdecl, called cross-TU.
#[no_mangle]
pub unsafe extern "C" fn UserGetObject(ht: PUSER_HANDLE_TABLE, handle: HANDLE, type_: HANDLE_TYPE) -> PVOID {
    // ASSERT(ht); — no-op in DBG=0 release.
    let entry: PUSER_HANDLE_ENTRY = handle_to_entry(ht, handle);
    if entry.is_null() || entry_type(entry) as HANDLE_TYPE != type_ {
        EngSetLastError(ERROR_INVALID_HANDLE);
        return core::ptr::null_mut();
    }
    entry_ptr(entry)
}

#[allow(dead_code)]
unsafe fn get_user_full_handle(ht: PUSER_HANDLE_TABLE, handle: HANDLE) -> HANDLE {
    let entry: PUSER_HANDLE_ENTRY;

    if (handle as ULONG_PTR) >> 16 != 0 {
        return handle;
    }
    entry = handle_to_entry(ht, handle);
    if entry.is_null() {
        return handle;
    }
    return entry_to_handle(ht, entry);
}

#[allow(dead_code)]
unsafe fn get_user_object_handle(ht: PUSER_HANDLE_TABLE, handle: *mut HANDLE, type_: HANDLE_TYPE) -> PVOID {
    let entry: PUSER_HANDLE_ENTRY = handle_to_entry(ht, *handle);
    if entry.is_null() || entry_type(entry) as HANDLE_TYPE != type_ {
        return core::ptr::null_mut();
    }
    *handle = entry_to_handle(ht, entry);
    entry_ptr(entry)
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn UserCreateHandleTable() -> BOOL {
    let mut mem: PVOID;
    let HandleCount: INT = 1024 * 4;

    mem = UserHeapAlloc((core::mem::size_of::<USER_HANDLE_ENTRY>() * HandleCount as usize) as SIZE_T);
    if mem.is_null() {
        ERR!("Failed creating handle table\n");
        return 0;
    }

    gHandleTable = UserHeapAlloc(core::mem::size_of::<USER_HANDLE_TABLE>() as SIZE_T) as PUSER_HANDLE_TABLE;
    if gHandleTable.is_null() {
        UserHeapFree(mem);
        ERR!("Failed creating handle table\n");
        return 0;
    }

    UserInitHandleTable(gHandleTable, mem, (core::mem::size_of::<USER_HANDLE_ENTRY>() * HandleCount as usize) as ULONG);

    return 1;
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn UserCreateHandleTable() -> BOOL {
    let mut mem: PVOID;
    let HandleCount: INT = 1024 * 4;

    mem = UserHeapAlloc((core::mem::size_of::<USER_HANDLE_ENTRY>() * HandleCount as usize) as SIZE_T);
    if mem.is_null() {
        ERR!("Failed creating handle table\n");
        return 0;
    }

    gHandleTable = UserHeapAlloc(core::mem::size_of::<USER_HANDLE_TABLE>() as SIZE_T) as PUSER_HANDLE_TABLE;
    if gHandleTable.is_null() {
        UserHeapFree(mem);
        ERR!("Failed creating handle table\n");
        return 0;
    }

    UserInitHandleTable(gHandleTable, mem, (core::mem::size_of::<USER_HANDLE_ENTRY>() * HandleCount as usize) as ULONG);

    return 1;
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn UserCreateObject(
    ht: PUSER_HANDLE_TABLE,
    pDesktop: PDESKTOP,
    pti: PTHREADINFO,
    h: *mut HANDLE,
    type_: HANDLE_TYPE,
    size: ULONG,
) -> PVOID {
    let mut ObjectOwner: PVOID = core::ptr::null_mut();

    let Object = ObjectCallbacks[type_ as usize].ObjectAlloc.unwrap()(
        pDesktop,
        pti,
        size as SIZE_T,
        &mut ObjectOwner as *mut PVOID,
    );
    if Object.is_null() {
        ERR!("User object allocation failed. Out of memory!\n");
        return core::ptr::null_mut();
    }

    let hi = UserAllocHandle(ht, Object, type_, ObjectOwner);
    if hi == 0 as HANDLE {
        ERR!("Out of user handles!\n");
        ObjectCallbacks[type_ as usize].ObjectFree.unwrap()(Object);
        return core::ptr::null_mut();
    }

    (*(Object as PHEAD)).h = hi;

    UserReferenceObject(Object);

    if !h.is_null() {
        *h = hi;
    }
    Object
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn UserCreateObject(
    ht: PUSER_HANDLE_TABLE,
    pDesktop: PDESKTOP,
    pti: PTHREADINFO,
    h: *mut HANDLE,
    type_: HANDLE_TYPE,
    size: ULONG,
) -> PVOID {
    let mut ObjectOwner: PVOID = core::ptr::null_mut();

    let Object = ObjectCallbacks[type_ as usize].ObjectAlloc.unwrap()(
        pDesktop,
        pti,
        size as SIZE_T,
        &mut ObjectOwner as *mut PVOID,
    );
    if Object.is_null() {
        ERR!("User object allocation failed. Out of memory!\n");
        return core::ptr::null_mut();
    }

    let hi = UserAllocHandle(ht, Object, type_, ObjectOwner);
    if hi == 0 as HANDLE {
        ERR!("Out of user handles!\n");
        ObjectCallbacks[type_ as usize].ObjectFree.unwrap()(Object);
        return core::ptr::null_mut();
    }

    (*(Object as PHEAD)).h = hi;

    UserReferenceObject(Object);

    if !h.is_null() {
        *h = hi;
    }
    Object
}

#[no_mangle]
#[cfg(target_arch = "x86")]
pub unsafe extern "fastcall" fn UserMarkObjectDestroy(Object: PVOID) -> BOOL {
    let entry: PUSER_HANDLE_ENTRY;
    let ObjHead: PHEAD = Object as PHEAD;

    entry = handle_to_entry(gHandleTable, (*ObjHead).h);

    // ASSERT(entry != NULL); — no-op in DBG=0 release.

    // entry->flags |= HANDLEENTRY_DESTROY;  (flags is BYTE; sibling-written)
    let f = entry_flags(entry) | HANDLEENTRY_DESTROY as u8;
    core::ptr::write_volatile(core::ptr::addr_of_mut!((*entry).flags), f);

    if (*ObjHead).cLockObj > 1 {
        let f = entry_flags(entry) & !(HANDLEENTRY_INDESTROY as u8);
        core::ptr::write_volatile(core::ptr::addr_of_mut!((*entry).flags), f);
        TRACE!(b"Count %d\n\0", (*ObjHead).cLockObj);
        return 0;
    }

    1
}

#[no_mangle]
#[cfg(not(target_arch = "x86"))]
pub unsafe extern "C" fn UserMarkObjectDestroy(Object: PVOID) -> BOOL {
    let entry: PUSER_HANDLE_ENTRY;
    let ObjHead: PHEAD = Object as PHEAD;

    entry = handle_to_entry(gHandleTable, (*ObjHead).h);

    // ASSERT(entry != NULL); — no-op in DBG=0 release.

    // entry->flags |= HANDLEENTRY_DESTROY;  (flags is BYTE; sibling-written)
    let f = entry_flags(entry) | HANDLEENTRY_DESTROY as u8;
    core::ptr::write_volatile(core::ptr::addr_of_mut!((*entry).flags), f);

    if (*ObjHead).cLockObj > 1 {
        let f = entry_flags(entry) & !(HANDLEENTRY_INDESTROY as u8);
        core::ptr::write_volatile(core::ptr::addr_of_mut!((*entry).flags), f);
        TRACE!(b"Count %d\n\0", (*ObjHead).cLockObj);
        return 0;
    }

    1
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn UserDereferenceObject(Object: PVOID) -> BOOL {
    let ObjHead: PHEAD = Object as PHEAD;

    // ASSERT(cLockObj >= 1 && < 0x10000); — no-op in DBG=0 release.
    (*ObjHead).cLockObj -= 1;
    if (*ObjHead).cLockObj == 0 {
        let entry: PUSER_HANDLE_ENTRY;
        let type_: HANDLE_TYPE;

        entry = handle_to_entry(gHandleTable, (*ObjHead).h);

        // ASSERT(entry != NULL && (flags & HANDLEENTRY_INDESTROY) &&
        //        type != TYPE_FREE && type < TYPE_CTYPES); — DBG=0 no-ops.
        type_ = entry_type(entry) as HANDLE_TYPE;

        free_user_entry(gHandleTable, entry);

        // ASSERT(ObjectCallbacks[type].ObjectFree != NULL); — DBG=0 no-op.
        ObjectCallbacks[type_ as usize].ObjectFree.unwrap()(Object);

        return 1;
    }
    0
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn UserDereferenceObject(Object: PVOID) -> BOOL {
    let ObjHead: PHEAD = Object as PHEAD;

    // ASSERT(cLockObj >= 1 && < 0x10000); — no-op in DBG=0 release.
    (*ObjHead).cLockObj -= 1;
    if (*ObjHead).cLockObj == 0 {
        let entry: PUSER_HANDLE_ENTRY;
        let type_: HANDLE_TYPE;

        entry = handle_to_entry(gHandleTable, (*ObjHead).h);

        // ASSERT(entry != NULL && (flags & HANDLEENTRY_INDESTROY) &&
        //        type != TYPE_FREE && type < TYPE_CTYPES); — DBG=0 no-ops.
        type_ = entry_type(entry) as HANDLE_TYPE;

        free_user_entry(gHandleTable, entry);

        // ASSERT(ObjectCallbacks[type].ObjectFree != NULL); — DBG=0 no-op.
        ObjectCallbacks[type_ as usize].ObjectFree.unwrap()(Object);

        return 1;
    }
    0
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn UserFreeHandle(ht: PUSER_HANDLE_TABLE, handle: HANDLE) -> BOOL {

    let entry: PUSER_HANDLE_ENTRY;

    entry = handle_to_entry(ht, handle);
    if entry.is_null() {
        SetLastNtError(STATUS_INVALID_HANDLE);
        return 0;
    }

    // HANDLEENTRY_GRANTED has to survive for free_user_entry.
    core::ptr::write_volatile(
        core::ptr::addr_of_mut!((*entry).flags),
        (entry_flags(entry) & HANDLEENTRY_GRANTED as u8) | HANDLEENTRY_INDESTROY as u8,
    );

    UserDereferenceObject(entry_ptr(entry))
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn UserFreeHandle(ht: PUSER_HANDLE_TABLE, handle: HANDLE) -> BOOL {

    let entry: PUSER_HANDLE_ENTRY;

    entry = handle_to_entry(ht, handle);
    if entry.is_null() {
        SetLastNtError(STATUS_INVALID_HANDLE);
        return 0;
    }

    // HANDLEENTRY_GRANTED has to survive for free_user_entry.
    core::ptr::write_volatile(
        core::ptr::addr_of_mut!((*entry).flags),
        (entry_flags(entry) & HANDLEENTRY_GRANTED as u8) | HANDLEENTRY_INDESTROY as u8,
    );

    UserDereferenceObject(entry_ptr(entry))
}
/// Validate a USER handle about to be granted to a job, and mark it so that freeing it
/// withdraws the grant. The mark is not cleared on revoke; it only says the granted lists are
/// worth sweeping (upstream b272427baa3, FASTCALL in the C but only win32k's job.rs calls it).
pub(crate) unsafe fn UserMarkHandleGranted(h: HANDLE) -> BOOL {
    let entry = handle_to_entry(gHandleTable, h);
    if entry.is_null() {
        return 0;
    }
    core::ptr::write_volatile(
        core::ptr::addr_of_mut!((*entry).flags),
        entry_flags(entry) | HANDLEENTRY_GRANTED as u8,
    );
    1
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn UserObjectInDestroy(h: HANDLE) -> BOOL {
    let entry: PUSER_HANDLE_ENTRY = handle_to_entry(gHandleTable, h);
    if entry.is_null() {
        SetLastNtError(STATUS_INVALID_HANDLE);
        return 1;
    }
    // C: return (entry->flags & HANDLEENTRY_INDESTROY);
    (entry_flags(entry) & HANDLEENTRY_INDESTROY as u8) as BOOL
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn UserObjectInDestroy(h: HANDLE) -> BOOL {
    let entry: PUSER_HANDLE_ENTRY = handle_to_entry(gHandleTable, h);
    if entry.is_null() {
        SetLastNtError(STATUS_INVALID_HANDLE);
        return 1;
    }
    // C: return (entry->flags & HANDLEENTRY_INDESTROY);
    (entry_flags(entry) & HANDLEENTRY_INDESTROY as u8) as BOOL
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn UserDeleteObject(h: HANDLE, type_: HANDLE_TYPE) -> BOOL {
    let body: PVOID = UserGetObject(gHandleTable, h, type_);

    if body.is_null() {
        return 0;
    }

    ntkrnl_sys::ASSERT!((*((body as PHEAD))).cLockObj >= 1);
    ntkrnl_sys::ASSERT!((*((body as PHEAD))).cLockObj < 0x10000);

    UserFreeHandle(gHandleTable, h)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn UserDeleteObject(h: HANDLE, type_: HANDLE_TYPE) -> BOOL {
    let body: PVOID = UserGetObject(gHandleTable, h, type_);

    if body.is_null() {
        return 0;
    }

    ntkrnl_sys::ASSERT!((*((body as PHEAD))).cLockObj >= 1);
    ntkrnl_sys::ASSERT!((*((body as PHEAD))).cLockObj < 0x10000);

    UserFreeHandle(gHandleTable, h)
}

// VOID FASTCALL UserReferenceObject(PVOID obj);  — object.h, called cross-TU.
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn UserReferenceObject(obj: PVOID) {
    let ObjHead = obj as PHEAD;
    // ASSERT(cLockObj < 0x10000); — no-op in DBG=0 release.
    (*ObjHead).cLockObj += 1;
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn UserReferenceObject(obj: PVOID) {
    let ObjHead = obj as PHEAD;
    (*ObjHead).cLockObj += 1;
}

// PVOID FASTCALL UserReferenceObjectByHandle(HANDLE handle, HANDLE_TYPE type);
// object.h, called cross-TU.
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn UserReferenceObjectByHandle(handle: HANDLE, type_: HANDLE_TYPE) -> PVOID {
    let object: PVOID = UserGetObject(gHandleTable, handle, type_);
    if !object.is_null() {
        UserReferenceObject(object);
    }
    object
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn UserReferenceObjectByHandle(handle: HANDLE, type_: HANDLE_TYPE) -> PVOID {
    let object: PVOID = UserGetObject(gHandleTable, handle, type_);
    if !object.is_null() {
        UserReferenceObject(object);
    }
    object
}

// BOOLEAN UserDestroyObjectsForOwner(PUSER_HANDLE_TABLE Table, PVOID Owner);
// object.h — plain cdecl (no FASTCALL/NTAPI), called cross-TU.
#[no_mangle]
pub unsafe extern "C" fn UserDestroyObjectsForOwner(
    Table: PUSER_HANDLE_TABLE,
    Owner: PVOID,
) -> BOOLEAN {
    let mut i: c_int;
    let mut Entry: PUSER_HANDLE_ENTRY;
    let mut Ret: BOOLEAN = 1u8;

    /* Sweep the whole handle table */
    i = 0;
    while i < (*Table).allocated_handles {
        Entry = (*Table).handles.offset(i as isize);

        if entry_pi(Entry) != Owner {
            i += 1;
            continue;
        }

        /* Do not destroy if it's already been done */
        if (entry_flags(Entry) & HANDLEENTRY_INDESTROY as u8) != 0 {
            i += 1;
            continue;
        }

        /* Call destructor */
        if (ObjectCallbacks[entry_type(Entry) as usize].ObjectDestroy.unwrap())(entry_ptr(Entry)) == 0 {
            // C varargs default-promote BYTE -> int for "%u".
            ERR!(
                "Failed destructing object %p, type %u.\n",
                entry_ptr(Entry),
                entry_type(Entry) as c_int
            );
            /* Don't return immediately, we must continue destroying the other objects */
            Ret = 0u8;
        }

        i += 1;
    }

    Ret
}
#[no_mangle]
pub unsafe extern "system" fn NtUserValidateHandleSecure(
    handle: HANDLE
) -> BOOL {
    let uType: UINT;
    let ppi: PPROCESSINFO;
    let entry: PUSER_HANDLE_ENTRY;
    let mut Ret: BOOL = 0;

    UserEnterExclusive();

    entry = handle_to_entry(gHandleTable, handle);
    if entry.is_null() {
        EngSetLastError(ERROR_INVALID_HANDLE);
        // goto Exit; // Return FALSE
        UserLeave();
        return Ret;
    }
    uType = entry_type(entry) as UINT;
    ppi = match uType {
        x if x == TYPE_WINDOW as UINT || x == TYPE_INPUTCONTEXT as UINT => {
            (*(entry_pi(entry) as PTHREADINFO)).ppi
        }
        x if x == TYPE_MENU as UINT
            || x == TYPE_ACCELTABLE as UINT
            || x == TYPE_CURSOR as UINT
            || x == TYPE_HOOK as UINT
            || x == TYPE_CALLPROC as UINT
            || x == TYPE_SETWINDOWPOS as UINT =>
        {
            entry_pi(entry) as PPROCESSINFO
        }
        _ => core::ptr::null_mut(),
    };

    if ppi.is_null() {
        // goto Exit; // Return FALSE
        UserLeave();
        return Ret;
    }

    // Same process job returns TRUE.
    if (*(*gptiCurrent).ppi).pW32Job == (*ppi).pW32Job {
        Ret = 1;
    }

    UserLeave();
    Ret
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn UserAssignmentLock(ppvObj: *mut PVOID, pvNew: PVOID) -> PVOID {
    let pvOld: PVOID = *ppvObj;
    *ppvObj = pvNew;

    if !pvOld.is_null() && pvOld == pvNew {
        return pvOld;
    }

    if !pvNew.is_null() {
        UserReferenceObject(pvNew);
    }

    if !pvOld.is_null() {
        if UserDereferenceObject(pvOld) != 0 {
            return core::ptr::null_mut();
        }
    }

    pvOld
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn UserAssignmentLock(ppvObj: *mut PVOID, pvNew: PVOID) -> PVOID {
    let pvOld: PVOID = *ppvObj;
    *ppvObj = pvNew;

    if !pvOld.is_null() && pvOld == pvNew {
        return pvOld;
    }

    if !pvNew.is_null() {
        UserReferenceObject(pvNew);
    }

    if !pvOld.is_null() {
        if UserDereferenceObject(pvOld) != 0 {
            return core::ptr::null_mut();
        }
    }

    pvOld
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn UserAssignmentUnlock(ppvObj: *mut PVOID) -> PVOID {
    let pvOld: PVOID = *ppvObj;
    *ppvObj = core::ptr::null_mut();

    if !pvOld.is_null() {
        if UserDereferenceObject(pvOld) != 0 {
            return core::ptr::null_mut();
        }
    }

    pvOld
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn UserAssignmentUnlock(ppvObj: *mut PVOID) -> PVOID {
    let pvOld: PVOID = *ppvObj;
    *ppvObj = core::ptr::null_mut();

    if !pvOld.is_null() {
        if UserDereferenceObject(pvOld) != 0 {
            return core::ptr::null_mut();
        }
    }

    pvOld
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:        See COPYING in the top level directory
//  * PROJECT:          ReactOS kernel
//  * PURPOSE:          User handle manager
//  * FILE:             win32ss/user/ntuser/object.c
//  * PROGRAMER:        Copyright (C) 2001 Alexandre Julliard
//  */
//
// #include <win32k.h>
// DBG_DEFAULT_CHANNEL(UserObj);
//
// //int usedHandles=0;
// PUSER_HANDLE_TABLE gHandleTable = NULL;
//
// /* Forward declarations */
// _Success_(return!=NULL)
// static PVOID AllocThreadObject(
//     _In_ PDESKTOP pDesk,
//     _In_ PTHREADINFO pti,
//     _In_ SIZE_T Size,
//     _Out_ PVOID* HandleOwner)
// {
//     PTHROBJHEAD ObjHead;
//
//     UNREFERENCED_PARAMETER(pDesk);
//
//     ASSERT(Size > sizeof(*ObjHead));
//     ASSERT(pti != NULL);
//
//     ObjHead = UserHeapAlloc(Size);
//     if (!ObjHead)
//         return NULL;
//
//     RtlZeroMemory(ObjHead, Size);
//
//     ObjHead->pti = pti;
//     IntReferenceThreadInfo(pti);
//     *HandleOwner = pti;
//     /* It's a thread object, but it still count as one for the process */
//     pti->ppi->UserHandleCount++;
//
//     return ObjHead;
// }
//
// static void FreeThreadObject(
//     _In_ PVOID Object)
// {
//     PTHROBJHEAD ObjHead = (PTHROBJHEAD)Object;
//     PTHREADINFO pti = ObjHead->pti;
//
//     UserHeapFree(ObjHead);
//
//     pti->ppi->UserHandleCount--;
//     IntDereferenceThreadInfo(pti);
// }
//
// _Success_(return!=NULL)
// static PVOID AllocDeskThreadObject(
//     _In_ PDESKTOP pDesk,
//     _In_ PTHREADINFO pti,
//     _In_ SIZE_T Size,
//     _Out_ PVOID* HandleOwner)
// {
//     PTHRDESKHEAD ObjHead;
//
//     ASSERT(Size > sizeof(*ObjHead));
//     ASSERT(pti != NULL);
//
//     if (!pDesk)
//         pDesk = pti->rpdesk;
//
//     ObjHead = DesktopHeapAlloc(pDesk, Size);
//     if (!ObjHead)
//         return NULL;
//
//     RtlZeroMemory(ObjHead, Size);
//
//     ObjHead->pSelf = ObjHead;
//     ObjHead->rpdesk = pDesk;
//     ObjHead->pti = pti;
//     IntReferenceThreadInfo(pti);
//     *HandleOwner = pti;
//     /* It's a thread object, but it still count as one for the process */
//     pti->ppi->UserHandleCount++;
//
//     return ObjHead;
// }
//
// static void FreeDeskThreadObject(
//     _In_ PVOID Object)
// {
//     PTHRDESKHEAD ObjHead = (PTHRDESKHEAD)Object;
//     PDESKTOP pDesk = ObjHead->rpdesk;
//     PTHREADINFO pti = ObjHead->pti;
//
//     DesktopHeapFree(pDesk, Object);
//
//     pti->ppi->UserHandleCount--;
//     IntDereferenceThreadInfo(pti);
// }
//
// _Success_(return!=NULL)
// static PVOID AllocDeskProcObject(
//     _In_ PDESKTOP pDesk,
//     _In_ PTHREADINFO pti,
//     _In_ SIZE_T Size,
//     _Out_ PVOID* HandleOwner)
// {
//     PPROCDESKHEAD ObjHead;
//     PPROCESSINFO ppi;
//
//     ASSERT(Size > sizeof(*ObjHead));
//     ASSERT(pDesk != NULL);
//     ASSERT(pti != NULL);
//
//     ObjHead = DesktopHeapAlloc(pDesk, Size);
//     if (!ObjHead)
//         return NULL;
//
//     RtlZeroMemory(ObjHead, Size);
//
//     ppi = pti->ppi;
//
//     ObjHead->pSelf = ObjHead;
//     ObjHead->rpdesk = pDesk;
//     ObjHead->hTaskWow = (DWORD_PTR)ppi;
//     ppi->UserHandleCount++;
//     IntReferenceProcessInfo(ppi);
//     *HandleOwner = ppi;
//
//     return ObjHead;
// }
//
// static void FreeDeskProcObject(
//     _In_ PVOID Object)
// {
//     PPROCDESKHEAD ObjHead = (PPROCDESKHEAD)Object;
//     PDESKTOP pDesk = ObjHead->rpdesk;
//     PPROCESSINFO ppi = (PPROCESSINFO)ObjHead->hTaskWow;
//
//     ppi->UserHandleCount--;
//     IntDereferenceProcessInfo(ppi);
//
//     DesktopHeapFree(pDesk, Object);
// }
//
// _Success_(return!=NULL)
// static PVOID AllocProcMarkObject(
//     _In_ PDESKTOP pDesk,
//     _In_ PTHREADINFO pti,
//     _In_ SIZE_T Size,
//     _Out_ PVOID* HandleOwner)
// {
//     PPROCMARKHEAD ObjHead;
//     PPROCESSINFO ppi = pti->ppi;
//
//     UNREFERENCED_PARAMETER(pDesk);
//
//     ASSERT(Size > sizeof(*ObjHead));
//
//     ObjHead = UserHeapAlloc(Size);
//     if (!ObjHead)
//         return NULL;
//
//     RtlZeroMemory(ObjHead, Size);
//
//     ObjHead->ppi = ppi;
//     IntReferenceProcessInfo(ppi);
//     *HandleOwner = ppi;
//     ppi->UserHandleCount++;
//
//     return ObjHead;
// }
//
// void FreeProcMarkObject(
//     _In_ PVOID Object)
// {
//     PPROCESSINFO ppi = ((PPROCMARKHEAD)Object)->ppi;
//
//     UserHeapFree(Object);
//
//     ppi->UserHandleCount--;
//     IntDereferenceProcessInfo(ppi);
// }
//
// _Success_(return!=NULL)
// static PVOID AllocSysObject(
//     _In_ PDESKTOP pDesk,
//     _In_ PTHREADINFO pti,
//     _In_ SIZE_T Size,
//     _Out_ PVOID* ObjectOwner)
// {
//     PVOID Object;
//
//     UNREFERENCED_PARAMETER(pDesk);
//     UNREFERENCED_PARAMETER(pti);
//
//     ASSERT(Size > sizeof(HEAD));
//
//     Object = UserHeapAlloc(Size);
//     if (!Object)
//         return NULL;
//
//     *ObjectOwner = NULL;
//
//     RtlZeroMemory(Object, Size);
//     return Object;
// }
//
// _Success_(return!=NULL)
// static PVOID AllocSysObjectCB(
//     _In_ PDESKTOP pDesk,
//     _In_ PTHREADINFO pti,
//     _In_ SIZE_T Size,
//     _Out_ PVOID* ObjectOwner)
// {
//     PVOID Object;
//
//     UNREFERENCED_PARAMETER(pDesk);
//     UNREFERENCED_PARAMETER(pti);
//     ASSERT(Size > sizeof(HEAD));
//
//     /* Allocate the clipboard data */
//     // FIXME: This allocation should be done on the current session pool;
//     // however ReactOS' MM doesn't support session pool yet.
//     Object = ExAllocatePoolZero(/* SESSION_POOL_MASK | */ PagedPool, Size, USERTAG_CLIPBOARD);
//     if (!Object)
//     {
//         ERR("ExAllocatePoolZero failed. No object created.\n");
//         return NULL;
//     }
//
//     *ObjectOwner = NULL;
//     return Object;
// }
//
// static void FreeSysObject(
//     _In_ PVOID Object)
// {
//     UserHeapFree(Object);
// }
//
// static void FreeSysObjectCB(
//     _In_ PVOID Object)
// {
//     ExFreePoolWithTag(Object, USERTAG_CLIPBOARD);
// }
//
// static const struct
// {
//     PVOID   (*ObjectAlloc)(PDESKTOP, PTHREADINFO, SIZE_T, PVOID*);
//     BOOLEAN (*ObjectDestroy)(PVOID);
//     void    (*ObjectFree)(PVOID);
// } ObjectCallbacks[TYPE_CTYPES] =
// {
//     { NULL,                     NULL,                       NULL },                 /* TYPE_FREE */
//     { AllocDeskThreadObject,    co_UserDestroyWindow,       FreeDeskThreadObject }, /* TYPE_WINDOW */
//     { AllocDeskProcObject,      UserDestroyMenuObject,      FreeDeskProcObject },   /* TYPE_MENU */
//     { AllocProcMarkObject,      IntDestroyCurIconObject,    FreeCurIconObject },    /* TYPE_CURSOR */
//     { AllocSysObject,           /*UserSetWindowPosCleanup*/NULL, FreeSysObject },   /* TYPE_SETWINDOWPOS */
//     { AllocDeskThreadObject,    IntRemoveHook,              FreeDeskThreadObject }, /* TYPE_HOOK */
//     { AllocSysObjectCB,         /*UserClipDataCleanup*/NULL,FreeSysObjectCB },      /* TYPE_CLIPDATA */
//     { AllocDeskProcObject,      DestroyCallProc,            FreeDeskProcObject },   /* TYPE_CALLPROC */
//     { AllocProcMarkObject,      UserDestroyAccelTable,      FreeProcMarkObject },   /* TYPE_ACCELTABLE */
//     { NULL,                     NULL,                       NULL },                 /* TYPE_DDEACCESS */
//     { NULL,                     NULL,                       NULL },                 /* TYPE_DDECONV */
//     { NULL,                     NULL,                       NULL },                 /* TYPE_DDEXACT */
//     { AllocSysObject,           /*UserMonitorCleanup*/NULL, FreeSysObject },        /* TYPE_MONITOR */
//     { AllocSysObject,           /*UserKbdLayoutCleanup*/NULL,FreeSysObject },       /* TYPE_KBDLAYOUT */
//     { AllocSysObject,           /*UserKbdFileCleanup*/NULL, FreeSysObject },        /* TYPE_KBDFILE */
//     { AllocThreadObject,        IntRemoveEvent,             FreeThreadObject },     /* TYPE_WINEVENTHOOK */
//     { AllocSysObject,           /*UserTimerCleanup*/NULL,   FreeSysObject },        /* TYPE_TIMER */
//     { AllocInputContextObject,  UserDestroyInputContext,    UserFreeInputContext }, /* TYPE_INPUTCONTEXT */
//     { NULL,                     NULL,                       NULL },                 /* TYPE_HIDDATA */
//     { NULL,                     NULL,                       NULL },                 /* TYPE_DEVICEINFO */
//     { NULL,                     NULL,                       NULL },                 /* TYPE_TOUCHINPUTINFO */
//     { NULL,                     NULL,                       NULL },                 /* TYPE_GESTUREINFOOBJ */
// };
//
// #if DBG
//
// void DbgUserDumpHandleTable(VOID)
// {
//     int HandleCounts[TYPE_CTYPES];
//     PPROCESSINFO ppiList;
//     int i;
//     PWCHAR TypeNames[] = {L"Free",L"Window",L"Menu", L"CursorIcon", L"SMWP", L"Hook", L"ClipBoardData", L"CallProc",
//                           L"Accel", L"DDEaccess", L"DDEconv", L"DDExact", L"Monitor", L"KBDlayout", L"KBDfile",
//                           L"Event", L"Timer", L"InputContext", L"HidData", L"DeviceInfo", L"TouchInput",L"GestureInfo"};
//
//     ERR("Total handles count: %lu\n", gpsi->cHandleEntries);
//
//     memset(HandleCounts, 0, sizeof(HandleCounts));
//
//     /* First of all count the number of handles per type */
//     ppiList = gppiList;
//     while (ppiList)
//     {
//         ERR("Process %s (%p) handles count: %d\n\t", ppiList->peProcess->ImageFileName, ppiList->peProcess->UniqueProcessId, ppiList->UserHandleCount);
//
//         for (i = 1 ;i < TYPE_CTYPES; i++)
//         {
//             HandleCounts[i] += ppiList->DbgHandleCount[i];
//
//             DbgPrint("%S: %lu, ", TypeNames[i], ppiList->DbgHandleCount[i]);
//             if (i % 6 == 0)
//                 DbgPrint("\n\t");
//         }
//         DbgPrint("\n");
//
//         ppiList = ppiList->ppiNext;
//     }
//
//     /* Print total type counts */
//     ERR("Total handles of the running processes: \n\t");
//     for (i = 1 ;i < TYPE_CTYPES; i++)
//     {
//         DbgPrint("%S: %d, ", TypeNames[i], HandleCounts[i]);
//         if (i % 6 == 0)
//             DbgPrint("\n\t");
//     }
//     DbgPrint("\n");
//
//     /* Now count the handle counts that are allocated from the handle table */
//     memset(HandleCounts, 0, sizeof(HandleCounts));
//     for (i = 0; i < gHandleTable->nb_handles; i++)
//          HandleCounts[gHandleTable->handles[i].type]++;
//
//     ERR("Total handles count allocated: \n\t");
//     for (i = 1 ;i < TYPE_CTYPES; i++)
//     {
//         DbgPrint("%S: %d, ", TypeNames[i], HandleCounts[i]);
//         if (i % 6 == 0)
//             DbgPrint("\n\t");
//     }
//     DbgPrint("\n");
// }
//
// #endif
//
// PUSER_HANDLE_ENTRY handle_to_entry(PUSER_HANDLE_TABLE ht, HANDLE handle )
// {
//    unsigned short generation;
//    int index = (LOWORD(handle) - FIRST_USER_HANDLE) >> 1;
//    if (index < 0 || index >= ht->nb_handles)
//       return NULL;
//    if (!ht->handles[index].type)
//       return NULL;
//    generation = HIWORD(handle);
//    if (generation == ht->handles[index].generation || !generation || generation == 0xffff)
//       return &ht->handles[index];
//    return NULL;
// }
//
// __inline static HANDLE entry_to_handle(PUSER_HANDLE_TABLE ht, PUSER_HANDLE_ENTRY ptr )
// {
//    int index = ptr - ht->handles;
//    return (HANDLE)((((INT_PTR)index << 1) + FIRST_USER_HANDLE) + (ptr->generation << 16));
// }
//
// __inline static PUSER_HANDLE_ENTRY alloc_user_entry(PUSER_HANDLE_TABLE ht)
// {
//    PUSER_HANDLE_ENTRY entry;
//    TRACE("handles used %lu\n", gpsi->cHandleEntries);
//
//    if (ht->freelist)
//    {
//       entry = ht->freelist;
//       ht->freelist = entry->ptr;
//
//       gpsi->cHandleEntries++;
//       return entry;
//    }
//
//    if (ht->nb_handles >= ht->allocated_handles)  /* Need to grow the array */
//    {
//        ERR("Out of user handles! Used -> %lu, NM_Handle -> %d\n", gpsi->cHandleEntries, ht->nb_handles);
//
// #if DBG
//        DbgUserDumpHandleTable();
// #endif
//
//       return NULL;
// #if 0
//       PUSER_HANDLE_ENTRY new_handles;
//       /* Grow array by 50% (but at minimum 32 entries) */
//       int growth = max( 32, ht->allocated_handles / 2 );
//       int new_size = min( ht->allocated_handles + growth, (LAST_USER_HANDLE-FIRST_USER_HANDLE+1) >> 1 );
//       if (new_size <= ht->allocated_handles)
//          return NULL;
//       if (!(new_handles = UserHeapReAlloc( ht->handles, new_size * sizeof(*ht->handles) )))
//          return NULL;
//       ht->handles = new_handles;
//       ht->allocated_handles = new_size;
// #endif
//    }
//
//    entry = &ht->handles[ht->nb_handles++];
//
//    entry->generation = 1;
//
//    gpsi->cHandleEntries++;
//
//    return entry;
// }
//
// VOID UserInitHandleTable(PUSER_HANDLE_TABLE ht, PVOID mem, ULONG bytes)
// {
//    ht->freelist = NULL;
//    ht->handles = mem;
//
//    ht->nb_handles = 0;
//    ht->allocated_handles = bytes / sizeof(USER_HANDLE_ENTRY);
// }
//
//
// __inline static void *free_user_entry(PUSER_HANDLE_TABLE ht, PUSER_HANDLE_ENTRY entry)
// {
//    void *ret;
//
// #if DBG
//    {
//        PPROCESSINFO ppi;
//        switch (entry->type)
//        {
//            case TYPE_WINDOW:
//            case TYPE_HOOK:
//            case TYPE_WINEVENTHOOK:
//                ppi = ((PTHREADINFO)entry->pi)->ppi;
//                break;
//            case TYPE_MENU:
//            case TYPE_CURSOR:
//            case TYPE_CALLPROC:
//            case TYPE_ACCELTABLE:
//                ppi = entry->pi;
//                break;
//            default:
//                ppi = NULL;
//        }
//        if (ppi)
//            ppi->DbgHandleCount[entry->type]--;
//    }
// #endif
//
//    ret = entry->ptr;
//    entry->ptr  = ht->freelist;
//    entry->type = 0;
//    entry->flags = 0;
//    entry->pi = NULL;
//    ht->freelist  = entry;
//
//    gpsi->cHandleEntries--;
//
//    return ret;
// }
//
// /* allocate a user handle for a given object */
// HANDLE UserAllocHandle(
//     _Inout_ PUSER_HANDLE_TABLE ht,
//     _In_ PVOID object,
//     _In_ HANDLE_TYPE type,
//     _In_ PVOID HandleOwner)
// {
//    PUSER_HANDLE_ENTRY entry = alloc_user_entry(ht);
//    if (!entry)
//       return 0;
//    entry->ptr  = object;
//    entry->type = type;
//    entry->flags = 0;
//    entry->pi = HandleOwner;
//    if (++entry->generation >= 0xffff)
//       entry->generation = 1;
//
//    /* We have created a handle, which is a reference! */
//    UserReferenceObject(object);
//
//    return entry_to_handle(ht, entry );
// }
//
// /* return a pointer to a user object from its handle without setting an error */
// PVOID UserGetObjectNoErr(PUSER_HANDLE_TABLE ht, HANDLE handle, HANDLE_TYPE type )
// {
//    PUSER_HANDLE_ENTRY entry;
//
//    ASSERT(ht);
//
//    if (!(entry = handle_to_entry(ht, handle )) || entry->type != type)
//    {
//       return NULL;
//    }
//    return entry->ptr;
// }
//
// /* return a pointer to a user object from its handle */
// PVOID UserGetObject(PUSER_HANDLE_TABLE ht, HANDLE handle, HANDLE_TYPE type )
// {
//    PUSER_HANDLE_ENTRY entry;
//
//    ASSERT(ht);
//
//    if (!(entry = handle_to_entry(ht, handle )) || entry->type != type)
//    {
//       EngSetLastError(ERROR_INVALID_HANDLE);
//       return NULL;
//    }
//    return entry->ptr;
// }
//
//
// /* Get the full handle (32bit) for a possibly truncated (16bit) handle */
// HANDLE get_user_full_handle(PUSER_HANDLE_TABLE ht,  HANDLE handle )
// {
//    PUSER_HANDLE_ENTRY entry;
//
//    if ((ULONG_PTR)handle >> 16)
//       return handle;
//    if (!(entry = handle_to_entry(ht, handle )))
//       return handle;
//    return entry_to_handle( ht, entry );
// }
//
//
// /* Same as get_user_object plus set the handle to the full 32-bit value */
// void *get_user_object_handle(PUSER_HANDLE_TABLE ht,  HANDLE* handle, HANDLE_TYPE type )
// {
//    PUSER_HANDLE_ENTRY entry;
//
//    if (!(entry = handle_to_entry(ht, *handle )) || entry->type != type)
//       return NULL;
//    *handle = entry_to_handle( ht, entry );
//    return entry->ptr;
// }
//
//
//
// BOOL FASTCALL UserCreateHandleTable(VOID)
// {
//    PVOID mem;
//    INT HandleCount = 1024 * 4;
//
//    // FIXME: Don't alloc all at once! Must be mapped into umode also...
//    mem = UserHeapAlloc(sizeof(USER_HANDLE_ENTRY) * HandleCount);
//    if (!mem)
//    {
//       ERR("Failed creating handle table\n");
//       return FALSE;
//    }
//
//    gHandleTable = UserHeapAlloc(sizeof(USER_HANDLE_TABLE));
//    if (gHandleTable == NULL)
//    {
//        UserHeapFree(mem);
//        ERR("Failed creating handle table\n");
//        return FALSE;
//    }
//
//    // FIXME: Make auto growable
//    UserInitHandleTable(gHandleTable, mem, sizeof(USER_HANDLE_ENTRY) * HandleCount);
//
//    return TRUE;
// }
//
// //
// // New
// //
// PVOID
// FASTCALL
// UserCreateObject( PUSER_HANDLE_TABLE ht,
//                   PDESKTOP pDesktop,
//                   PTHREADINFO pti,
//                   HANDLE* h,
//                   HANDLE_TYPE type,
//                   ULONG size)
// {
//    HANDLE hi;
//    PVOID Object;
//    PVOID ObjectOwner;
//
//    /* Some sanity checks. Other checks will be made in the allocator */
//    ASSERT(type < TYPE_CTYPES);
//    ASSERT(type != TYPE_FREE);
//    ASSERT(ht != NULL);
//
//    /* Allocate the object */
//    ASSERT(ObjectCallbacks[type].ObjectAlloc != NULL);
//    Object = ObjectCallbacks[type].ObjectAlloc(pDesktop, pti, size, &ObjectOwner);
//    if (!Object)
//    {
//        ERR("User object allocation failed. Out of memory!\n");
//        return NULL;
//    }
//
//    hi = UserAllocHandle(ht, Object, type, ObjectOwner);
//    if (hi == NULL)
//    {
//        ERR("Out of user handles!\n");
//        ObjectCallbacks[type].ObjectFree(Object);
//        return NULL;
//    }
//
// #if DBG
//    if (pti)
//        pti->ppi->DbgHandleCount[type]++;
// #endif
//
//    /* Give this object its identity. */
//    ((PHEAD)Object)->h = hi;
//
//    /* The caller will get a locked object.
//     * Note: with the reference from the handle, that makes two */
//    UserReferenceObject(Object);
//
//    if (h)
//       *h = hi;
//    return Object;
// }
//
// // Win: HMMarkObjectDestroy
// BOOL
// FASTCALL
// UserMarkObjectDestroy(PVOID Object)
// {
//     PUSER_HANDLE_ENTRY entry;
//     PHEAD ObjHead = Object;
//
//     entry = handle_to_entry(gHandleTable, ObjHead->h);
//
//     ASSERT(entry != NULL);
//
//     entry->flags |= HANDLEENTRY_DESTROY;
//
//     if (ObjHead->cLockObj > 1)
//     {
//         entry->flags &= ~HANDLEENTRY_INDESTROY;
//         TRACE("Count %d\n",ObjHead->cLockObj);
//         return FALSE;
//     }
//
//     return TRUE;
// }
//
// BOOL
// FASTCALL
// UserDereferenceObject(PVOID Object)
// {
//     PHEAD ObjHead = Object;
//
//     ASSERT(ObjHead->cLockObj >= 1);
//     ASSERT(ObjHead->cLockObj < 0x10000);
//
//     if (--ObjHead->cLockObj == 0)
//     {
//         PUSER_HANDLE_ENTRY entry;
//         HANDLE_TYPE type;
//
//         entry = handle_to_entry(gHandleTable, ObjHead->h);
//
//         ASSERT(entry != NULL);
//         /* The entry should be marked as in deletion */
//         ASSERT(entry->flags & HANDLEENTRY_INDESTROY);
//
//         type = entry->type;
//         ASSERT(type != TYPE_FREE);
//         ASSERT(type < TYPE_CTYPES);
//
//         /* We can now get rid of everything */
//         free_user_entry(gHandleTable, entry );
//
// #if 0
//         /* Call the object destructor */
//         ASSERT(ObjectCallbacks[type].ObjectCleanup != NULL);
//         ObjectCallbacks[type].ObjectCleanup(Object);
// #endif
//
//         /* And free it */
//         ASSERT(ObjectCallbacks[type].ObjectFree != NULL);
//         ObjectCallbacks[type].ObjectFree(Object);
//
//         return TRUE;
//     }
//     return FALSE;
// }
//
// BOOL
// FASTCALL
// UserFreeHandle(PUSER_HANDLE_TABLE ht,  HANDLE handle )
// {
//   PUSER_HANDLE_ENTRY entry;
//
//   if (!(entry = handle_to_entry( ht, handle )))
//   {
//      SetLastNtError( STATUS_INVALID_HANDLE );
//      return FALSE;
//   }
//
//   entry->flags = HANDLEENTRY_INDESTROY;
//
//   return UserDereferenceObject(entry->ptr);
// }
//
// BOOL
// FASTCALL
// UserObjectInDestroy(HANDLE h)
// {
//   PUSER_HANDLE_ENTRY entry;
//
//   if (!(entry = handle_to_entry( gHandleTable, h )))
//   {
//      SetLastNtError( STATUS_INVALID_HANDLE );
//      return TRUE;
//   }
//   return (entry->flags & HANDLEENTRY_INDESTROY);
// }
//
// BOOL
// FASTCALL
// UserDeleteObject(HANDLE h, HANDLE_TYPE type )
// {
//    PVOID body = UserGetObject(gHandleTable, h, type);
//
//    if (!body) return FALSE;
//
//    ASSERT( ((PHEAD)body)->cLockObj >= 1);
//    ASSERT( ((PHEAD)body)->cLockObj < 0x10000);
//
//    return UserFreeHandle(gHandleTable, h);
// }
//
// VOID
// FASTCALL
// UserReferenceObject(PVOID obj)
// {
//    PHEAD ObjHead = obj;
//    ASSERT(ObjHead->cLockObj < 0x10000);
//
//    ObjHead->cLockObj++;
// }
//
// PVOID
// FASTCALL
// UserReferenceObjectByHandle(HANDLE handle, HANDLE_TYPE type)
// {
//     PVOID object;
//
//     object = UserGetObject(gHandleTable, handle, type);
//     if (object)
//     {
//        UserReferenceObject(object);
//     }
//     return object;
// }
//
// BOOLEAN
// UserDestroyObjectsForOwner(PUSER_HANDLE_TABLE Table, PVOID Owner)
// {
//     int i;
//     PUSER_HANDLE_ENTRY Entry;
//     BOOLEAN Ret = TRUE;
//
//     /* Sweep the whole handle table */
//     for (i = 0; i < Table->allocated_handles; i++)
//     {
//         Entry = &Table->handles[i];
//
//         if (Entry->pi != Owner)
//             continue;
//
//         /* Do not destroy if it's already been done */
//         if (Entry->flags & HANDLEENTRY_INDESTROY)
//             continue;
//
//         /* Call destructor */
//         if (!ObjectCallbacks[Entry->type].ObjectDestroy(Entry->ptr))
//         {
//             ERR("Failed destructing object %p, type %u.\n", Entry->ptr, Entry->type);
//             /* Don't return immediately, we must continue destroying the other objects */
//             Ret = FALSE;
//         }
//     }
//
//     return Ret;
// }
//
// /*
//  *
//  * Status
//  *    @implemented
//  */
//
// BOOL
// APIENTRY
// NtUserValidateHandleSecure(
//    HANDLE handle)
// {
//    UINT uType;
//    PPROCESSINFO ppi;
//    PUSER_HANDLE_ENTRY entry;
//    BOOL Ret = FALSE;
//
//    UserEnterExclusive();
//
//    if (!(entry = handle_to_entry(gHandleTable, handle )))
//    {
//       EngSetLastError(ERROR_INVALID_HANDLE);
//       goto Exit; // Return FALSE
//    }
//    uType = entry->type;
//    switch (uType)
//    {
//        case TYPE_WINDOW:
//        case TYPE_INPUTCONTEXT:
//           ppi = ((PTHREADINFO)entry->pi)->ppi;
//           break;
//        case TYPE_MENU:
//        case TYPE_ACCELTABLE:
//        case TYPE_CURSOR:
//        case TYPE_HOOK:
//        case TYPE_CALLPROC:
//        case TYPE_SETWINDOWPOS:
//           ppi = entry->pi;
//           break;
//        default:
//           ppi = NULL;
//           break;
//    }
//
//    if (!ppi)
//        goto Exit; // Return FALSE
//
//    // Same process job returns TRUE.
//    if (gptiCurrent->ppi->pW32Job == ppi->pW32Job) Ret = TRUE;
//
// Exit:
//    UserLeave();
//    return Ret;
// }
//
// // Win: HMAssignmentLock
// PVOID FASTCALL UserAssignmentLock(PVOID *ppvObj, PVOID pvNew)
// {
//     PVOID pvOld = *ppvObj;
//     *ppvObj = pvNew;
//
//     if (pvOld && pvOld == pvNew)
//         return pvOld;
//
//     if (pvNew)
//         UserReferenceObject(pvNew);
//
//     if (pvOld)
//     {
//         if (UserDereferenceObject(pvOld))
//             pvOld = NULL;
//     }
//
//     return pvOld;
// }
//
// // Win: HMAssignmentUnlock
// PVOID FASTCALL UserAssignmentUnlock(PVOID *ppvObj)
// {
//     PVOID pvOld = *ppvObj;
//     *ppvObj = NULL;
//
//     if (pvOld)
//     {
//         if (UserDereferenceObject(pvOld))
//             pvOld = NULL;
//     }
//
//     return pvOld;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
