use win32k_sys::win32k::*;
use ntkrnl_sys::ddk::{KeEnterCriticalRegion, KeLeaveCriticalRegion};
use ntkrnl_sys::interlocked::{InterlockedIncrement, InterlockedDecrement, InterlockedExchangeAdd};

// win32k's own non-raising user-pointer validator (safe_user_io.rs). Under
// DUMMY_PSEH the DDK ProbeFor* macros are unusable here: they RAISE, and nothing
// would catch it.
extern "system" {
    fn SafeProbeForWrite(a: *mut core::ffi::c_void, l: usize, al: u32) -> i32;
}

// C debug macros — silenced via macro_rules! (variadic no-op).
#[allow(unused_macros)] macro_rules! NT_ASSERT { ($($t:tt)*) => { () }; }

// These are real exported kernel symbols. ntkrnl-sys declares them too, but
// nominally typed against ntkrnl-sys's own bindgen structs (distinct nominal
// types from win32k-sys's, even though byte-identical — both bindgen the same C
// headers). To avoid casting at every call site we re-declare them here against
// win32k-sys types; the symbol/ABI is unchanged (extern "system" == stdcall on
// i386, C on amd64/arm64), only the Rust nominal types differ.
extern "system" {
    fn PsGetCurrentThreadWin32Thread() -> PTHREADINFO;
    fn PsGetCurrentProcessWin32Process() -> PPROCESSINFO;
    fn PsGetProcessWin32Process(Process: PEPROCESS) -> PPROCESSINFO;
    fn PsGetCurrentProcessId() -> HANDLE;
    fn PsGetCurrentThreadId() -> HANDLE;
    fn PsLookupProcessByProcessId(ProcessId: HANDLE, Process: *mut PEPROCESS) -> NTSTATUS;
    fn MmCreateSection(
        SectionObject: *mut PVOID,
        DesiredAccess: ACCESS_MASK,
        ObjectAttributes: POBJECT_ATTRIBUTES,
        MaximumSize: PLARGE_INTEGER,
        SectionPageProtection: ULONG,
        AllocationAttributes: ULONG,
        FileHandle: HANDLE,
        File: PFILE_OBJECT,
    ) -> NTSTATUS;
    fn MmMapViewInSessionSpace(Section: PVOID, MappedBase: *mut PVOID, ViewSize: PSIZE_T)
        -> NTSTATUS;
    fn MmMapViewOfSection(
        SectionObject: PVOID,
        Process: PEPROCESS,
        BaseAddress: *mut PVOID,
        ZeroBits: ULONG_PTR,
        CommitSize: SIZE_T,
        SectionOffset: PLARGE_INTEGER,
        ViewSize: PSIZE_T,
        InheritDisposition: SECTION_INHERIT,
        AllocationType: ULONG,
        Protect: ULONG,
    ) -> NTSTATUS;
    fn ExInitializePagedLookasideList(
        Lookaside: PPAGED_LOOKASIDE_LIST,
        Allocate: PALLOCATE_FUNCTION,
        Free: PFREE_FUNCTION,
        Flags: ULONG,
        Size: SIZE_T,
        Tag: ULONG,
        Depth: USHORT,
    );
    fn ExAllocatePoolWithTag(PoolType: POOL_TYPE, NumberOfBytes: SIZE_T, Tag: ULONG) -> PVOID;
    fn ExFreePoolWithTag(P: PVOID, Tag: ULONG);
}
use core::ffi::c_void;

// FORCEINLINE/macro/intrinsic equivalents from the C headers (faithful translations).
// These are NOT exported kernel symbols; the C source reaches them via header macros
// or static-inline definitions, so they are reproduced here in-crate.

// ObfDereferenceObject is FASTCALL (the `Obf` prefix = NT fastcall;
// ntkrnl-sys declares it `extern "fastcall"`). i386 fastcall = @N
// decoration; amd64/arm64 uniformly C.
#[cfg(target_arch = "x86")]
extern "fastcall" {
    // ObDereferenceObject(x) is the macro for ObfDereferenceObject(x).
    fn ObfDereferenceObject(Object: PVOID) -> isize;
}
#[cfg(not(target_arch = "x86"))]
extern "C" {
    fn ObfDereferenceObject(Object: PVOID) -> isize;
}
extern "C" {
    fn DbgPrint(Format: *const i8, ...) -> u32;
}

/// ObDereferenceObject — macro alias for ObfDereferenceObject.
#[inline(always)]
unsafe fn ObDereferenceObject(object: PVOID) {
    ObfDereferenceObject(object);
}

// NT status / pool / section / tag constants (from psdk/ntstatus.h, xdk/winnt_old.h,
// win32ss/user/ntuser/tags.h, win32ss/include/ntgdihdl.h). C multi-char literals are
// big-endian: 'bthG' == [b'G', b't', b'h', b'b'] little-endian = 0x6874_6247... but
// these are taken from the header values directly.
const STATUS_SUCCESS: NTSTATUS = 0x0000_0000u32 as NTSTATUS;
const STATUS_NO_MEMORY: NTSTATUS = 0xC000_0017u32 as NTSTATUS;
const STATUS_INSUFFICIENT_RESOURCES: NTSTATUS = 0xC000_009Au32 as NTSTATUS;
const SECTION_ALL_ACCESS: u32 = 0xf001f;
// 'bthG' (GDI handle table) and '0alG' (lookaside start) — multi-char C char
// constants are big-endian on the host: value = (c0<<24)|(c1<<16)|(c2<<8)|c3.
const TAG_GDIHNDTBLE: u32 = u32::from_be_bytes([b'b', b't', b'h', b'G']);
const GDITAG_HMGR_LOOKASIDE_START: u32 = u32::from_be_bytes([b'0', b'a', b'l', b'G']);

// PagedPool / NonPagedPool POOL_TYPE values (xdk: NonPagedPool=0, PagedPool=1).
#[allow(non_upper_case_globals)]
const PagedPool: POOL_TYPE = 1;
#[allow(non_upper_case_globals)]
const NonPagedPool: POOL_TYPE = 0;

/// InterlockedReadUlong — FORCEINLINE in gdiobj.c: `return *Source;` (volatile read).
#[inline(always)]
unsafe fn InterlockedReadUlong(source: *mut u32) -> u32 {
    core::ptr::read_volatile(source)
}

/// UlongToPtr(ul) — ((PVOID)(ULONG_PTR)(ul)) (basetsd.h).
#[inline(always)]
unsafe fn UlongToPtr(ul: u32) -> *mut c_void {
    ul as usize as *mut c_void
}

/// PtrToUlong(p) — ((ULONG)(ULONG_PTR)(p)) (basetsd.h).
#[inline(always)]
fn PtrToUlong<T>(p: *mut T) -> u32 {
    p as usize as u32
}

/// HandleToUlong(h) — ((ULONG)(ULONG_PTR)(h)) (basetsd.h).
#[inline(always)]
fn HandleToUlong(h: HANDLE) -> u32 {
    h as usize as u32
}

/// UlongToHandle(ul) / ULongToHandle(ul) — ((HANDLE)(ULONG_PTR)(ul)) (basetsd.h).
#[inline(always)]
fn UlongToHandle(ul: u32) -> HANDLE {
    ul as usize as HANDLE
}
#[inline(always)]
#[allow(non_snake_case)]
fn ULongToHandle(ul: u32) -> HANDLE {
    ul as usize as HANDLE
}

/// NT_SUCCESS(Status) — ((NTSTATUS)(Status) >= 0).
#[inline(always)]
fn NT_SUCCESS(status: NTSTATUS) -> bool {
    (status as i32) >= 0
}

/// RtlZeroMemory(dst, len) — memset(dst, 0, len).
#[inline(always)]
unsafe fn RtlZeroMemory(dst: PVOID, len: usize) {
    core::ptr::write_bytes(dst as *mut u8, 0, len);
}

/// GDI_HANDLE_GET_INDEX(h) — ((ULONG_PTR)(h)) & GDI_HANDLE_INDEX_MASK (ntgdihdl.h).
#[inline(always)]
fn GDI_HANDLE_GET_INDEX(h: HANDLE) -> u32 {
    ((h as usize as u32) & GDI_HANDLE_INDEX_MASK) as u32
}

/// GDI_HANDLE_GET_TYPE(h) — ((ULONG_PTR)(h)) & GDI_HANDLE_TYPE_MASK (ntgdihdl.h).
#[inline(always)]
fn GDI_HANDLE_GET_TYPE(h: HGDIOBJ) -> u32 {
    (h as usize as u32) & GDI_HANDLE_TYPE_MASK
}

/// GDI_HANDLE_IS_STOCKOBJ(h) — 0 != (((ULONG_PTR)(h)) & GDI_HANDLE_STOCK_MASK).
#[inline(always)]
fn GDI_HANDLE_IS_STOCKOBJ(h: HGDIOBJ) -> u32 {
    (((h as usize as u32) & GDI_HANDLE_STOCK_MASK) != 0) as u32
}

/// GDIOBJ_POOL_TAG(type) — (0x47'0044' + ((type & 0x1f) << 24)) from gdiobj.c.
#[inline(always)]
fn GDIOBJ_POOL_TAG_FN(objt: u8) -> u32 {
    0x00684700u32 + ((((objt as u32) & 0x1f)) << 24)
}

/// min(a,b) — minmax.h macro.
#[inline(always)]
fn min(a: u32, b: u32) -> u32 {
    if a < b { a } else { b }
}

/// InterlockedOr — atomic OR, returns OLD value.
#[inline(always)]
unsafe fn InterlockedOr(destination: *mut LONG, value: LONG) -> LONG {
    use core::sync::atomic::{AtomicI32, Ordering};
    (*(destination as *const AtomicI32)).fetch_or(value, Ordering::SeqCst)
}

/// InterlockedAnd — atomic AND, returns OLD value.
#[inline(always)]
unsafe fn InterlockedAnd(destination: *mut i32, value: i32) -> i32 {
    use core::sync::atomic::{AtomicI32, Ordering};
    (*(destination as *const AtomicI32)).fetch_and(value, Ordering::SeqCst)
}

/// _InterlockedOr16 — atomic 16-bit OR, returns OLD value.
#[inline(always)]
unsafe fn InterlockedOr16(destination: *mut i16, value: i16) -> i16 {
    use core::sync::atomic::{AtomicI16, Ordering};
    (*(destination as *const AtomicI16)).fetch_or(value, Ordering::SeqCst)
}

/// InterlockedCompareExchange — CAS, returns the ORIGINAL value at Destination
/// (matching the x86 `lock cmpxchg` / WDK semantics).
#[inline(always)]
unsafe fn InterlockedCompareExchange(destination: *mut i32, exchange: i32, comparand: i32) -> i32 {
    use core::sync::atomic::{AtomicI32, Ordering};
    let atomic = &*(destination as *const AtomicI32);
    match atomic.compare_exchange(comparand, exchange, Ordering::SeqCst, Ordering::SeqCst) {
        Ok(prev) => prev,
        Err(prev) => prev,
    }
}

/// ExTryAcquirePushLockExclusive — misc.h FORCEINLINE:
/// `return !InterlockedBitTestAndSet((PLONG)PushLock, EX_PUSH_LOCK_LOCK_V);`
/// EX_PUSH_LOCK_LOCK_V == bit 0. Returns TRUE iff the lock bit was previously clear.
#[inline(always)]
unsafe fn ExTryAcquirePushLockExclusive(push_lock: *mut c_void) -> u8 {
    use core::sync::atomic::{AtomicUsize, Ordering};
    let atomic = &*(push_lock as *const AtomicUsize);
    let old = atomic.fetch_or(1usize, Ordering::SeqCst);
    ((old & 1) == 0) as u8
}

// ExfAcquirePushLockExclusive / ExfReleasePushLockExclusive / ExfTryToWakePushLock
// are the FASTCALL slow-path exports (NT `Exf` prefix = fastcall; exfuncs.h:
// `VOID FASTCALL ExfAcquirePushLockExclusive(...)` etc.). i386 fastcall = @N
// decoration; amd64/arm64 uniformly C. The Ex{Initialize,Acquire,Release}
// PushLockExclusive entry points are FORCEINLINE macros (ntoskrnl
// internal/ex.h), NOT exported symbols — reproduced in-crate below as faithful
// translations of those FORCEINLINE bodies.
#[cfg(target_arch = "x86")]
extern "fastcall" {
    fn ExfAcquirePushLockExclusive(PushLock: PEX_PUSH_LOCK);
    fn ExfReleasePushLockExclusive(PushLock: PEX_PUSH_LOCK);
    fn ExfTryToWakePushLock(PushLock: PEX_PUSH_LOCK);
}
#[cfg(not(target_arch = "x86"))]
extern "C" {
    fn ExfAcquirePushLockExclusive(PushLock: PEX_PUSH_LOCK);
    fn ExfReleasePushLockExclusive(PushLock: PEX_PUSH_LOCK);
    fn ExfTryToWakePushLock(PushLock: PEX_PUSH_LOCK);
}

/// ExInitializePushLock — internal/ex.h FORCEINLINE `_ExInitializePushLock`:
/// `PushLock->Ptr = 0;` (#define ExInitializePushLock _ExInitializePushLock).
#[inline(always)]
unsafe fn ExInitializePushLock(push_lock: PEX_PUSH_LOCK) {
    (*push_lock).__bindgen_anon_1.Ptr = core::ptr::null_mut();
}

/// ExAcquirePushLockExclusive — internal/ex.h FORCEINLINE:
/// `if (InterlockedBitTestAndSet((PLONG)PushLock, EX_PUSH_LOCK_LOCK_V))`
/// `    ExfAcquirePushLockExclusive(PushLock);` (EX_PUSH_LOCK_LOCK_V == bit 0).
#[inline(always)]
unsafe fn ExAcquirePushLockExclusive(push_lock: PEX_PUSH_LOCK) {
    use core::sync::atomic::{AtomicUsize, Ordering};
    let atomic = &*(push_lock as *const AtomicUsize);
    let old = atomic.fetch_or(1usize, Ordering::SeqCst);
    if (old & 1) != 0 {
        // Someone changed it, use the slow path.
        ExfAcquirePushLockExclusive(push_lock);
    }
}

/// ExReleasePushLockExclusive — internal/ex.h FORCEINLINE:
/// `OldValue.Value = InterlockedExchangeAddSizeT((PSIZE_T)PushLock,`
/// `                     -(SSIZE_T)EX_PUSH_LOCK_LOCK);`
/// `if ((OldValue.Waiting) && !(OldValue.Waking)) ExfTryToWakePushLock(PushLock);`
/// EX_PUSH_LOCK_LOCK == 0x1; bitfield layout (extypes.h): bit0 Locked,
/// bit1 Waiting, bit2 Waking, bit3 MultipleShared, bits4.. Shared.
#[inline(always)]
unsafe fn ExReleasePushLockExclusive(push_lock: PEX_PUSH_LOCK) {
    use core::sync::atomic::{AtomicUsize, Ordering};
    let atomic = &*(push_lock as *const AtomicUsize);
    let old_value = atomic.fetch_sub(1usize, Ordering::SeqCst);
    let waiting = (old_value & 0x2) != 0;
    let waking = (old_value & 0x4) != 0;
    if waiting && !waking {
        ExfTryToWakePushLock(push_lock);
    }
}

// ExAllocate/FreeToPagedLookasideList are real NTAPI ntoskrnl exports
// (ntoskrnl.spec: `@ stdcall ExAllocateFromPagedLookasideList(ptr)` /
// `@ stdcall ExFreeToPagedLookasideList(ptr ptr)`). win32k links the
// ntoskrnl import lib, so call the export directly — behaviour is
// identical to the exfuncs.h inline (the inline IS this code) and it
// avoids the i386 SLIST-primitive plumbing (RtlInterlockedPushEntrySList
// is not an i386 export). extern "system" = stdcall on i386, C elsewhere.
extern "system" {
    fn ExAllocateFromPagedLookasideList(Lookaside: PPAGED_LOOKASIDE_LIST) -> PVOID;
    fn ExFreeToPagedLookasideList(Lookaside: PPAGED_LOOKASIDE_LIST, Entry: PVOID);
}

// FontGetObject is FASTCALL (text.h: `ULONG FASTCALL FontGetObject`);
// PEN_GetObject is NTAPI (pen.h: `INT NTAPI PEN_GetObject`) → stdcall
// on i386. Different ABIs — must be declared separately.
#[cfg(target_arch = "x86")]
extern "fastcall" {
    fn FontGetObject(pvObj: *mut c_void, cbCount: i32, pvBuffer: *mut c_void) -> i32;
}
#[cfg(not(target_arch = "x86"))]
extern "C" {
    fn FontGetObject(pvObj: *mut c_void, cbCount: i32, pvBuffer: *mut c_void) -> i32;
}
extern "system" {
    fn PEN_GetObject(pvObj: *mut c_void, cbCount: i32, pvBuffer: *mut c_void) -> i32;
}

// DRIVEROBJ_vCleanup — exported NTAPI from win32ss/gdi/eng/driverobj.c. The
// declared ABI must match the bindgen GDICLEANUPPROC fn-pointer type per arch
// (stdcall on i386, C on amd64/arm64) so it can be stored in apfnCleanup.
extern "system" {
    fn DRIVEROBJ_vCleanup(pObject: PVOID);
}

// DbgGdiHTIntegrityCheck — only referenced inside #if DBG blocks (DBG=0 in the
// release smoke build). The C calls are dead; provide a no-op so any surviving
// reference compiles identically (it is never invoked at runtime in this build).
#[inline(always)]
unsafe fn DbgGdiHTIntegrityCheck() {}

// NT_ASSERT / ASSERT_* — all expand to `((void)0)` in the non-DBG build
// (rtlfuncs.h NT_ASSERT_NOASSUME, gdiobj.c ASSERT_*_OBJECT_TYPE). No-ops.
// The expression argument is still evaluated below as in C `(void)(expr)`-style
// macros would NOT be — NT_ASSERT_NOASSUME is `((void)0)` and does not evaluate
// its argument, so we accept and discard it without evaluating side effects here
// is impossible in Rust call syntax; the arg is a value that was already formed.
// All call sites pass pure predicate expressions (no side effects), matching C.
#[inline(always)]
fn NT_ASSERT<T>(_condition: T) {}

#[no_mangle]
pub static mut gpvGdiHdlTblSection: PVOID = core::ptr::null_mut();

#[no_mangle]
pub static mut gpentHmgr: PENTRY = core::ptr::null_mut();

#[no_mangle]
pub static mut gpaulRefCount: PULONG = core::ptr::null_mut();

#[no_mangle]
pub static mut gulFirstFree: ULONG = 0;

#[no_mangle]
pub static mut gulFirstUnused: ULONG = 0;

#[no_mangle]
pub static mut gpaLookasideList: PPAGED_LOOKASIDE_LIST = core::ptr::null_mut();

#[no_mangle]
pub static mut GdiHandleTable: PGDI_HANDLE_TABLE = core::ptr::null_mut();

pub const REF_MASK_REUSE: u32 = 0xff000000;
pub const REF_INC_REUSE: u32 = 0x01000000;
pub const REF_MASK_VALID: u32 = 0x00800000;
pub const REF_MASK_COUNT: u32 = 0x007fffff;
pub const REF_MASK_INUSE: u32 = 0x00ffffff;

/// GDIOBJ_POOL_TAG(type) — gdiobj.c macro:
/// (0x47'0044' + ((type & 0x1f) << 24)) — i.e. 0x00684700 + ((type & 0x1f) << 24).
#[inline(always)]
fn GDIOBJ_POOL_TAG(objt: u8) -> u32 {
    GDIOBJ_POOL_TAG_FN(objt)
}

// ASSERT_*_OBJECT_TYPE / ASSERT_LOCK_ORDER — defined empty in the non-DBG build
// (gdiobj.c #else branch). No-ops; argument carries no side effects in C either.
#[inline(always)]
fn ASSERT_SHARED_OBJECT_TYPE(_objt: u8) {}
#[inline(always)]
fn ASSERT_EXCLUSIVE_OBJECT_TYPE(_objt: u8) {}
#[inline(always)]
fn ASSERT_TRYLOCK_OBJECT_TYPE(_objt: u8) {}

static apfnCleanup: [GDICLEANUPPROC; 32] = [
    None,
    Some(DC_vCleanup),
    None,
    None,
    Some(REGION_vCleanup),
    Some(SURFACE_vCleanup),
    Some(GDIOBJ_vCleanup),
    Some(GDIOBJ_vCleanup),
    Some(PALETTE_vCleanup),
    Some(GDIOBJ_vCleanup),
    Some(GDIOBJ_vCleanup),
    None,
    None,
    None,
    Some(GDIOBJ_vCleanup),
    None,
    None,
    None,
    None,
    None,
    None,
    Some(GDIOBJ_vCleanup),
    None,
    None,
    None,
    None,
    None,
    None,
    Some(DRIVEROBJ_vCleanup),
    None,
    None,
    None,
];

static apfnDelete: [GDIOBJDELETEPROC; 32] = [
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    Some(BRUSH_vDeleteObject),
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
];

#[inline(always)]

unsafe fn INCREASE_THREAD_LOCK_COUNT(hobj: HANDLE) {
    let pti: PTHREADINFO = PsGetCurrentThreadWin32Thread();
    let _ = hobj;
    if !pti.is_null() {
        #[cfg(debug_assertions)]
        {
            let index = ((hobj as usize) >> 16) & 0x1f;
            (*pti).acExclusiveLockCount[index] = (*pti).acExclusiveLockCount[index].wrapping_add(1);
        }
        (*pti).cExclusiveLocks = (*pti).cExclusiveLocks.wrapping_add(1);
    }
}

unsafe fn DECREASE_THREAD_LOCK_COUNT(hobj: HANDLE) {
    let pti: PTHREADINFO = PsGetCurrentThreadWin32Thread();
    let _ = hobj;
    if !pti.is_null() {
        #[cfg(debug_assertions)]
        {
            (*pti).acExclusiveLockCount[((hobj as ULONG_PTR >> 16) & 0x1f) as usize] -= 1;
        }
        (*pti).cExclusiveLocks -= 1;
    }
}

// ASSERT_LOCK_ORDER — the function-bodied form exists only under #if DBG in
// gdiobj.c; in the non-DBG release build the macro `#define ASSERT_LOCK_ORDER(hobj)`
// expands to nothing. Faithful release behavior is a no-op.
#[inline(always)]
unsafe fn ASSERT_LOCK_ORDER(_objt: u8) {}

// static VOID NTAPI GDIOBJ_vCleanup(PVOID) — ABI must match the bindgen
// GDICLEANUPPROC fn-pointer type per arch so it can be stored in apfnCleanup.
unsafe extern "system" fn GDIOBJ_vCleanup(_ObjectBody: PVOID) {
    /* Nothing to do */
}

unsafe fn InitLookasideList(objt: UCHAR, cjSize: ULONG) {
    ExInitializePagedLookasideList(
        gpaLookasideList.add(objt as usize),
        None,
        None,
        0,
        cjSize as SIZE_T,
        GDITAG_HMGR_LOOKASIDE_START + ((objt as ULONG) << 24),
        0,
    );
}

#[no_mangle]
pub unsafe extern "system" fn InitGdiHandleTable() -> NTSTATUS {
    let mut liSize = LARGE_INTEGER { QuadPart: 0 };
    let mut pvSection: PVOID = core::ptr::null_mut();
    let mut cjViewSize: SIZE_T = 0;

    liSize.QuadPart = core::mem::size_of::<GDI_HANDLE_TABLE>() as i64;
    let status = MmCreateSection(
        &raw mut gpvGdiHdlTblSection,
        SECTION_ALL_ACCESS,
        core::ptr::null_mut(),
        &mut liSize,
        PAGE_READWRITE,
        SEC_COMMIT | 0x1,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );
    if !NT_SUCCESS(status) {
        return status;
    }

    let status = MmMapViewInSessionSpace(
        gpvGdiHdlTblSection,
        &raw mut gpentHmgr as *mut _ as *mut PVOID,
        &mut cjViewSize,
    );
    if !NT_SUCCESS(status) {
        ObDereferenceObject(gpvGdiHdlTblSection);
        return status;
    }

    gpaulRefCount = EngAllocSectionMem(
        &mut pvSection,
        FL_ZERO_MEMORY,
        (GDI_HANDLE_COUNT as SIZE_T) * (core::mem::size_of::<ULONG>() as SIZE_T),
        u32::from_ne_bytes([b'f', b'r', b'H', b'G']),
    ) as *mut ULONG;
    if gpaulRefCount.is_null() {
        ObDereferenceObject(gpvGdiHdlTblSection);
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    gulFirstFree = 0;
    gulFirstUnused = RESERVE_ENTRIES_COUNT;

    GdiHandleTable = gpentHmgr as PGDI_HANDLE_TABLE;

    gpaLookasideList = ExAllocatePoolWithTag(
        NonPagedPool,
        (GDIObjTypeTotal as SIZE_T) * (core::mem::size_of::<PAGED_LOOKASIDE_LIST>() as SIZE_T),
        TAG_GDIHNDTBLE,
    ) as *mut PAGED_LOOKASIDE_LIST;
    if gpaLookasideList.is_null() {
        return STATUS_NO_MEMORY;
    }

    InitLookasideList(GDIObjType_DC_TYPE as u8, core::mem::size_of::<DC>() as ULONG);
    InitLookasideList(GDIObjType_RGN_TYPE as u8, core::mem::size_of::<REGION>() as ULONG);
    InitLookasideList(GDIObjType_SURF_TYPE as u8, core::mem::size_of::<SURFACE>() as ULONG);
    InitLookasideList(GDIObjType_CLIENTOBJ_TYPE as u8, core::mem::size_of::<CLIENTOBJ>() as ULONG);
    InitLookasideList(GDIObjType_PATH_TYPE as u8, core::mem::size_of::<PATH>() as ULONG);
    InitLookasideList(GDIObjType_PAL_TYPE as u8, core::mem::size_of::<PALETTE>() as ULONG);
    InitLookasideList(GDIObjType_ICMLCS_TYPE as u8, core::mem::size_of::<COLORSPACE>() as ULONG);
    InitLookasideList(GDIObjType_LFONT_TYPE as u8, core::mem::size_of::<TEXTOBJ>() as ULONG);
    InitLookasideList(GDIObjType_BRUSH_TYPE as u8, core::mem::size_of::<BRUSH>() as ULONG);

    STATUS_SUCCESS
}

unsafe fn IncrementCurrentProcessGdiHandleCount() {
    let ppi: PPROCESSINFO = PsGetCurrentProcessWin32Process();
    if !ppi.is_null() {
        InterlockedIncrement(core::ptr::addr_of_mut!((*(ppi as *mut W32PROCESS)).GDIHandleCount) as *mut i32);
    }
}

unsafe fn DecrementCurrentProcessGdiHandleCount() {
    let ppi: PPROCESSINFO = PsGetCurrentProcessWin32Process();
    if !ppi.is_null() {
        InterlockedDecrement(core::ptr::addr_of_mut!((*(ppi as *mut W32PROCESS)).GDIHandleCount) as *mut i32);
    }
}

unsafe fn IncrementGdiHandleCount(ulProcessId: u32) {
    let mut pep: PEPROCESS = core::ptr::null_mut();
    let ppi: PPROCESSINFO;
    let Status: i32;

    Status = PsLookupProcessByProcessId(ULongToHandle(ulProcessId), &mut pep);

    ppi = PsGetProcessWin32Process(pep);
    if !ppi.is_null() {
        InterlockedIncrement(core::ptr::addr_of_mut!((*(ppi as *mut W32PROCESS)).GDIHandleCount) as *mut i32);
    }
    if NT_SUCCESS(Status) {
        ObDereferenceObject(pep as *mut core::ffi::c_void);
    }
}

unsafe fn DecrementGdiHandleCount(ulProcessId: u32) {
    let mut pep: PEPROCESS = core::ptr::null_mut();
    let ppi: PPROCESSINFO;
    let Status: i32;

    Status = PsLookupProcessByProcessId(ULongToHandle(ulProcessId), &mut pep);

    ppi = PsGetProcessWin32Process(pep);
    if !ppi.is_null() {
        InterlockedDecrement(core::ptr::addr_of_mut!((*(ppi as *mut W32PROCESS)).GDIHandleCount) as *mut i32);
    }
    if NT_SUCCESS(Status) {
        ObDereferenceObject(pep as *mut core::ffi::c_void);
    }
}

unsafe fn ENTRY_pentPopFreeEntry() -> PENTRY {
    let mut iFirst: u32;
    let mut iNext: u32;
    let iPrev: u32;
    let mut pentFree: PENTRY;

    loop {
        iFirst = InterlockedReadUlong(core::ptr::addr_of_mut!(gulFirstFree));

        if (iFirst & GDI_HANDLE_INDEX_MASK) == 0 {
            iFirst = InterlockedIncrement(core::ptr::addr_of_mut!(gulFirstUnused) as *mut i32) as u32 - 1;

            if iFirst >= GDI_HANDLE_COUNT {
                InterlockedDecrement(core::ptr::addr_of_mut!(gulFirstUnused) as *mut i32);
                return core::ptr::null_mut();
            }

            return gpentHmgr.offset(iFirst as isize);
        }

        pentFree = gpentHmgr.offset((iFirst & GDI_HANDLE_INDEX_MASK) as isize);

        iNext = GDI_HANDLE_GET_INDEX((*pentFree).einfo.hFree);
        iNext |= (iFirst & !GDI_HANDLE_INDEX_MASK).wrapping_add(0x10000);

        let iPrev = InterlockedCompareExchange(
            core::ptr::addr_of_mut!(gulFirstFree) as *mut i32,
            iNext as i32,
            iFirst as i32,
        ) as u32;

        if iPrev == iFirst {
            break;
        }
    }

    pentFree
}

unsafe fn ENTRY_vPushFreeEntry(pentFree: PENTRY) {
    let mut iToFree: u32;
    let mut iFirst: u32;
    let mut iPrev: u32;
    let idxToFree: u32;

    idxToFree = pentFree.offset_from(gpentHmgr) as u32;

    (*pentFree).Objt = GDIObjType_DEF_TYPE as u8;
    (*pentFree).ObjectOwner.ulObj = 0;
    (*pentFree).pUser = core::ptr::null_mut();

    // C: REF_INC_REUSE (0x01000000). 0x80000000 added 0x80 to the reuse byte, so the
    // counter that makes a stale handle detectable wrapped every second free.
    InterlockedExchangeAdd(
        gpaulRefCount.add(idxToFree as usize) as *mut i32,
        REF_INC_REUSE as i32
    );
    (*pentFree).FullUnique = (*pentFree).FullUnique.wrapping_add(0x0100);

    loop {
        iFirst = InterlockedReadUlong(core::ptr::addr_of_mut!(gulFirstFree));

        (*pentFree).einfo.pobj = UlongToPtr(iFirst & GDI_HANDLE_INDEX_MASK) as POBJ;

        iToFree = idxToFree | ((iFirst & !GDI_HANDLE_INDEX_MASK).wrapping_add(0x10000));

        iPrev = InterlockedCompareExchange(
            core::ptr::addr_of_mut!(gulFirstFree) as *mut i32,
            iToFree as i32,
            iFirst as i32,
        ) as u32;

        if iPrev == iFirst {
            break;
        }
    }
}

unsafe fn ENTRY_ReferenceEntryByHandle(hobj: HGDIOBJ, fl: FLONG) -> PENTRY {
    let mut ulIndex: u32;
    let mut cNewRefs: u32;
    let mut cOldRefs: u32;
    let pentry: PENTRY;

    // The TIF_MOVESIZETRACKING hack that used to force GDIOBJFLAG_IGNOREPID here
    // was REVERTED upstream (640e39c72e, reverting #8402) because it crashes the
    // kernel: DceUpdateVisRgn locks a region not owned by the current process,
    // REGION_LockRgn wrongly succeeds, and REGION_vSyncRegion then reads a user-mode
    // region attribute living in a DIFFERENT process. CORE-20720 / CORE-5601. We
    // translated the C between the hack and its revert, so we inherited the crash.
    ulIndex = GDI_HANDLE_GET_INDEX(hobj);

    pentry = gpentHmgr.offset(ulIndex as isize);

    cOldRefs = core::ptr::read_volatile(gpaulRefCount.offset(ulIndex as isize));

    loop {
        if (cOldRefs & REF_MASK_VALID) == 0 {
            return core::ptr::null_mut();
        }

        // pentry->FullUnique / ObjectOwner.ulObj are shared mutable handle-table
        // memory (e.g. GDIOBJ_ConvertToStockObj rewrites them for stock objects).
        // C reads them as plain loads here, but the compiler must not cache/hoist
        // the slot across the foreign call below; volatile loads match the C
        // memory semantics exactly without changing logic.
        if core::ptr::read_volatile(core::ptr::addr_of!((*pentry).FullUnique))
            != ((hobj as usize >> 16) as u16) {
            return core::ptr::null_mut();
        }

        if (fl & GDIOBJFLAG_IGNOREPID) == 0 &&
            core::ptr::read_volatile(core::ptr::addr_of!((*pentry).ObjectOwner.ulObj)) != GDI_OBJ_HMGR_PUBLIC &&
            core::ptr::read_volatile(core::ptr::addr_of!((*pentry).ObjectOwner.ulObj)) != PtrToUlong(PsGetCurrentProcessId() as *mut _) {
            return core::ptr::null_mut();
        }

        cNewRefs = cOldRefs + 1;
        cOldRefs = InterlockedCompareExchange(
            gpaulRefCount.offset(ulIndex as isize) as *mut i32,
            cNewRefs as i32,
            cOldRefs as i32,
        ) as u32;

        if cNewRefs == cOldRefs + 1 {
            break;
        }
    }

    pentry
}

unsafe fn ENTRY_hInsertObject(pentry: PENTRY, pobj: POBJ, objt: UCHAR, ulOwner: ULONG) -> HGDIOBJ {

    let ulIndex = pentry.offset_from(gpentHmgr) as ULONG;

    (*pentry).einfo.pobj = pobj;
    (*pentry).Objt = objt & 0x1f;
    (*pentry).FullUnique = ((*pentry).FullUnique & 0xff00) | objt as USHORT;
    (*pentry).ObjectOwner.ulObj = ulOwner;

    InterlockedOr(
        gpaulRefCount.offset(ulIndex as isize) as *mut LONG,
        (REF_MASK_VALID | 1) as LONG
    );

    (((*pentry).FullUnique as ULONG_PTR) << 16 | ulIndex as ULONG_PTR) as HGDIOBJ
}
#[no_mangle]
pub unsafe extern "system" fn GDIOBJ_AllocateObject(
    objt: u8,
    cjSize: u32,
    fl: FLONG,
) -> POBJ {

    const EVENT_ALLOCATE: u32 = 0;

    let pobj: POBJ;

    if (fl & BASEFLAG_LOOKASIDE) != 0 {
        pobj = ExAllocateFromPagedLookasideList(
            gpaLookasideList.add((objt & 0x1f) as usize)
        ) as POBJ;
    } else {
        pobj = ExAllocatePoolWithTag(PagedPool, cjSize as SIZE_T, GDIOBJ_POOL_TAG(objt)) as POBJ;
    }

    if pobj.is_null() {
        return core::ptr::null_mut();
    }

    RtlZeroMemory(pobj as PVOID, cjSize as usize);
    (*pobj).hHmgr = ((objt as usize) << 16) as HGDIOBJ;
    (*pobj).cExclusiveLock = 0;
    (*pobj).__bindgen_anon_1.ulShareCount = 1;
    (*pobj).BaseFlags = (fl & 0xffff) as u16;
    let _ = 0;

    pobj
}
#[no_mangle]
pub unsafe extern "system" fn GDIOBJ_vFreeObject(pobj: POBJ) {

    let objt: UCHAR;

    objt = (((*pobj).hHmgr as ULONG_PTR >> 16) & 0x1f) as UCHAR;

    if apfnDelete[objt as usize].is_some() {
        apfnDelete[objt as usize].unwrap()(pobj as PVOID);
    } else {
        NT_ASSERT!(apfnCleanup[objt as usize].is_some());
        apfnCleanup[objt as usize].unwrap()(pobj as PVOID);

        if (*pobj).BaseFlags & (BASEFLAG_LOOKASIDE as u16) != 0 {
            ExFreeToPagedLookasideList(
                gpaLookasideList.add(objt as usize),
                pobj as PVOID,
            );
        } else {
            ExFreePoolWithTag(pobj as PVOID, GDIOBJ_POOL_TAG(objt));
        }
    }
}
#[no_mangle]
pub unsafe extern "system" fn GDIOBJ_vDereferenceObject(pobj: POBJ) {

    const EVENT_DEREFERENCE: u32 = 2;

    let cRefs: u32;
    let ulIndex: u32;

    ulIndex = GDI_HANDLE_GET_INDEX((*pobj).hHmgr);

    if ulIndex != 0 {
        if (*gpaulRefCount.offset(ulIndex as isize) & REF_MASK_COUNT) == 0 {
        }
        cRefs = InterlockedDecrement(gpaulRefCount.offset(ulIndex as isize) as *mut i32) as u32;
        let _ = cRefs;

        if (cRefs & REF_MASK_INUSE) == 0 {
            if (*gpentHmgr.offset(ulIndex as isize)).ObjectOwner.ulObj != GDI_OBJ_HMGR_PUBLIC &&
               (*gpentHmgr.offset(ulIndex as isize)).ObjectOwner.ulObj != GDI_OBJ_HMGR_NONE {
                DecrementCurrentProcessGdiHandleCount();
            }

            ENTRY_vPushFreeEntry(gpentHmgr.offset(ulIndex as isize));

            GDIOBJ_vFreeObject(pobj);
        }
    } else {
        cRefs = InterlockedDecrement(core::ptr::addr_of_mut!((*pobj).__bindgen_anon_1.ulShareCount) as *mut i32) as u32;
        let _ = cRefs;

        if cRefs == 0 {
            GDIOBJ_vFreeObject(pobj);
        }
    }
}
#[no_mangle]
pub unsafe extern "system" fn GDIOBJ_ReferenceObjectByHandle(
    hobj: HGDIOBJ,
    objt: u8,
) -> POBJ {

    const EVENT_REFERENCE: u32 = 1;

    let pentry: PENTRY;
    let pobj: POBJ;

    ASSERT_SHARED_OBJECT_TYPE(objt);
    if ((((hobj as usize) >> 16) & 0x1f) as u8) != objt {
        return core::ptr::null_mut();
    }

    pentry = ENTRY_ReferenceEntryByHandle(hobj, 0);
    if pentry.is_null() {
        return core::ptr::null_mut();
    }

    pobj = (*pentry).einfo.pobj;

    if (*pobj).cExclusiveLock != 0 {
        GDIOBJ_vDereferenceObject(pobj);
        return core::ptr::null_mut();
    }

    let _ = *gpaulRefCount.offset(pentry.offset_from(gpentHmgr));

    pobj
}
#[no_mangle]
pub unsafe extern "system" fn GDIOBJ_vReferenceObjectByPointer(pobj: POBJ) {
    let cRefs: ULONG;

    if GDI_HANDLE_GET_INDEX((*pobj).hHmgr) != 0 {
        let ulIndex = GDI_HANDLE_GET_INDEX((*pobj).hHmgr);
        cRefs = InterlockedIncrement(gpaulRefCount.add(ulIndex as usize) as *mut LONG) as ULONG;
    } else {
        cRefs = InterlockedIncrement(core::ptr::addr_of_mut!((*pobj).__bindgen_anon_1.ulShareCount) as *mut LONG) as ULONG;
    }

    let _ = cRefs;
}

#[no_mangle]
pub unsafe extern "system" fn GDIOBJ_TryLockObject(
    hobj: HGDIOBJ,
    objt: u8,
) -> PGDIOBJ {

    let pentry: PENTRY;
    let pobj: POBJ;
    let dwThreadId: u32;

    ASSERT_TRYLOCK_OBJECT_TYPE(objt);
    if ((((hobj as usize) >> 16) & 0x1f) as u8) != objt {
        return core::ptr::null_mut();
    }

    ASSERT_LOCK_ORDER(objt);

    pentry = ENTRY_ReferenceEntryByHandle(hobj, 0);
    if pentry.is_null() {
        return core::ptr::null_mut();
    }

    pobj = (*pentry).einfo.pobj;

    dwThreadId = PtrToUlong(PsGetCurrentThreadId() as *mut c_void);
    if (*pobj).__bindgen_anon_1.dwThreadId != dwThreadId {
        KeEnterCriticalRegion();
        if ExTryAcquirePushLockExclusive(core::ptr::addr_of_mut!((*pobj).pushlock) as *mut c_void) == 0 {
            let cRefs: u32;
            let ulIndex: u32;

            KeLeaveCriticalRegion();

            ulIndex = GDI_HANDLE_GET_INDEX((*pobj).hHmgr);

            cRefs = InterlockedDecrement(gpaulRefCount.offset(ulIndex as isize) as *mut i32) as u32;

            return core::ptr::null_mut();
        }

        (*pobj).__bindgen_anon_1.dwThreadId = dwThreadId;
    }

    (*pobj).cExclusiveLock = (*pobj).cExclusiveLock.wrapping_add(1);
    INCREASE_THREAD_LOCK_COUNT(hobj as HANDLE);
    let _ = 0;

    pobj as PGDIOBJ
}
#[no_mangle]
pub unsafe extern "system" fn GDIOBJ_LockObject(
    hobj: HGDIOBJ,
    objt: u8,
) -> PGDIOBJ {
    
    const EVENT_LOCK: u32 = 1;
    
    let pentry: PENTRY;
    let pobj: POBJ;
    let dwThreadId: u32;

    ASSERT_EXCLUSIVE_OBJECT_TYPE(objt);
    if ((((hobj as usize) >> 16) & 0x1f) as u8) != objt {
        return core::ptr::null_mut();
    }

    ASSERT_LOCK_ORDER(objt);

    pentry = ENTRY_ReferenceEntryByHandle(hobj, 0);
    if pentry.is_null() {
        return core::ptr::null_mut();
    }

    pobj = (*pentry).einfo.pobj;

    dwThreadId = PtrToUlong(PsGetCurrentThreadId());
    if (*pobj).__bindgen_anon_1.dwThreadId != dwThreadId {
        KeEnterCriticalRegion();
        ExAcquirePushLockExclusive(core::ptr::addr_of_mut!((*pobj).pushlock));

        (*pobj).__bindgen_anon_1.dwThreadId = dwThreadId;
    }

    (*pobj).cExclusiveLock = (*pobj).cExclusiveLock.wrapping_add(1);
    INCREASE_THREAD_LOCK_COUNT(hobj);
    let _ = 0;

    pobj as PGDIOBJ
}
#[no_mangle]
pub unsafe extern "system" fn GDIOBJ_vUnlockObject(pobj: POBJ) {
    let mut cRefs: ULONG;
    let ulIndex: ULONG;

    (*pobj).cExclusiveLock -= 1;
    DECREASE_THREAD_LOCK_COUNT((*pobj).hHmgr);
    let _ = 0;

    if (*pobj).cExclusiveLock == 0 {
        (*pobj).__bindgen_anon_1.dwThreadId = 0;

        ExReleasePushLockExclusive(core::ptr::addr_of_mut!((*pobj).pushlock));
        KeLeaveCriticalRegion();
    }

    ulIndex = GDI_HANDLE_GET_INDEX((*pobj).hHmgr);

    cRefs = InterlockedDecrement(gpaulRefCount.add(ulIndex as usize) as *mut LONG) as ULONG;
}

// DC_LockDc / DC_UnlockDc — FORCEINLINE in dc.h:219/238 (plain cdecl per bindgen).
// Provide as #[no_mangle] cdecl so consumers (pdevobj.rs, mouse.rs) resolve.
#[no_mangle]
pub unsafe extern "C" fn DC_LockDc(hdc: HDC) -> PDC {
    // PDC pdc = (PDC)GDIOBJ_LockObject(hdc, GDIObjType_DC_TYPE);
    GDIOBJ_LockObject(hdc as HGDIOBJ, GDIObjType_GDIObjType_DC_TYPE as u8) as PDC
}

#[no_mangle]
pub unsafe extern "C" fn DC_UnlockDc(pdc: PDC) {
    // GDIOBJ_vUnlockObject(&pdc->BaseObject);
    GDIOBJ_vUnlockObject(core::ptr::addr_of_mut!((*pdc).BaseObject) as POBJ);
}

/// Who filled the handle table: live entries by type, by owning process, and the top owner's
/// types. A full table is always a leak, and the object that finally fails to insert is rarely
/// the one leaking.
unsafe fn dbg_census_full_table() {
    let mut by_type = [0u32; 32];
    let mut owners = [(0u32, 0u32); 16];
    let mut top_types = [0u32; 32];
    let end = core::cmp::min(gulFirstUnused, GDI_HANDLE_COUNT);
    for i in RESERVE_ENTRIES_COUNT..end {
        let pentry = gpentHmgr.offset(i as isize);
        let objt = ((*pentry).Objt & 0x1f) as usize;
        if objt == GDIObjType_DEF_TYPE as usize {
            continue;
        }
        by_type[objt] += 1;
        let owner = (*pentry).ObjectOwner.ulObj & !1;
        if let Some(slot) = owners.iter_mut().find(|s| s.1 == 0 || s.0 == owner) {
            *slot = (owner, slot.1 + 1);
        }
    }
    let top = owners.iter().max_by_key(|s| s.1).map(|s| s.0).unwrap_or(0);
    for i in RESERVE_ENTRIES_COUNT..end {
        let pentry = gpentHmgr.offset(i as isize);
        if (*pentry).ObjectOwner.ulObj & !1 == top {
            top_types[((*pentry).Objt & 0x1f) as usize] += 1;
        }
    }
    DbgPrint(b"[GDI-CENSUS] handle table full, firstUnused=%lu\n\0".as_ptr() as *const i8, gulFirstUnused);
    for (t, c) in by_type.iter().enumerate() {
        if *c != 0 {
            DbgPrint(b"[GDI-CENSUS]   type %02x: %lu\n\0".as_ptr() as *const i8, t as u32, *c);
        }
    }
    for (pid, c) in owners.iter().filter(|s| s.1 != 0) {
        DbgPrint(b"[GDI-CENSUS]   owner pid %lx: %lu\n\0".as_ptr() as *const i8, *pid, *c);
    }
    for (t, c) in top_types.iter().enumerate() {
        if *c != 0 {
            DbgPrint(b"[GDI-CENSUS]   top owner %lx type %02x: %lu\n\0".as_ptr() as *const i8, top, t as u32, *c);
        }
    }
}

#[no_mangle]
pub unsafe extern "system" fn GDIOBJ_hInsertObject(
    pobj: POBJ,
    ulOwner: ULONG,
) -> HGDIOBJ {

    let pentry: PENTRY;
    let objt: UCHAR;

    pentry = ENTRY_pentPopFreeEntry();
    if pentry.is_null() {
        /* The C's DPRINT1. A full handle table makes every Create* return NULL, and callers like
         * co_WinPosSetWindowPos use the result unchecked; say so before they fault. */
        static FULL: core::sync::atomic::AtomicU32 = core::sync::atomic::AtomicU32::new(0);
        let n = FULL.fetch_add(1, core::sync::atomic::Ordering::Relaxed);
        if n < 3 {
            DbgPrint(b"GDIOBJ: Could not get a free entry.\n\0".as_ptr() as *const i8);
        }
        if n == 0 {
            dbg_census_full_table();
        }
        return core::ptr::null_mut();
    }

    ExInitializePushLock(core::ptr::addr_of_mut!((*pobj).pushlock));
    KeEnterCriticalRegion();
    ExAcquirePushLockExclusive(core::ptr::addr_of_mut!((*pobj).pushlock));
    (*pobj).cExclusiveLock = 1;
    (*pobj).__bindgen_anon_1.dwThreadId = PtrToUlong(PsGetCurrentThreadId() as *mut core::ffi::c_void);
    INCREASE_THREAD_LOCK_COUNT((*pobj).hHmgr);

    objt = (((*pobj).hHmgr as ULONG_PTR >> 16) & 0xff) as UCHAR;

    let ulOwner = if ulOwner == GDI_OBJ_HMGR_POWNED {
        IncrementCurrentProcessGdiHandleCount();
        HandleToUlong(PsGetCurrentProcessId())
    } else {
        ulOwner
    };

    (*pobj).hHmgr = ENTRY_hInsertObject(pentry, pobj, objt, ulOwner);

    let _ = 0;
    (*pobj).hHmgr
}
#[no_mangle]
pub unsafe extern "system" fn GDIOBJ_vSetObjectOwner(
    pobj: POBJ,
    ulNewOwner: u32,
) {
    let pentry: PENTRY;
    let ulOldOwner: u32;

    if GDI_HANDLE_IS_STOCKOBJ((*pobj).hHmgr) != 0 {
        return;
    }

    NT_ASSERT!(GDI_HANDLE_GET_INDEX((*pobj).hHmgr) != 0);
    pentry = &mut (*gpentHmgr.offset(GDI_HANDLE_GET_INDEX((*pobj).hHmgr) as isize));

    ulOldOwner = (*pentry).ObjectOwner.ulObj;
    if ulOldOwner == ulNewOwner {
        return;
    }

    let mut ulNewOwner = ulNewOwner;
    if ulNewOwner == GDI_OBJ_HMGR_POWNED {
        ulNewOwner = HandleToUlong(PsGetCurrentProcessId());
    }

    if ulNewOwner == GDI_OBJ_HMGR_NONE {
        ulNewOwner = GDI_OBJ_HMGR_PUBLIC;
    }

    if (ulOldOwner != GDI_OBJ_HMGR_PUBLIC) &&
       (ulOldOwner != GDI_OBJ_HMGR_NONE) {
        DecrementGdiHandleCount(ulOldOwner);
    }

    if (ulNewOwner != GDI_OBJ_HMGR_PUBLIC) &&
       (ulNewOwner != GDI_OBJ_HMGR_NONE) {
        IncrementGdiHandleCount(ulNewOwner);
    } else {
        NT_ASSERT!((*pentry).pUser.is_null());
    }

    (*pentry).ObjectOwner.ulObj = ulNewOwner;
    let _ = 0;
}
#[no_mangle]
pub unsafe extern "system" fn GDIOBJ_bLockMultipleObjects(
    ulCount: ULONG,
    ahObj: *mut HGDIOBJ,
    apObj: *mut PGDIOBJ,
    objt: UCHAR,
) -> BOOL {
    let mut auiIndices: [UINT; 3] = [0, 1, 2];
    let mut i: UINT;
    let mut j: UINT;
    let mut tmp: UINT;

    i = 0;
    while i < ulCount - 1 {
        j = i + 1;
        while j < ulCount {
            if (*ahObj.add(auiIndices[i as usize] as usize) as ULONG_PTR) <
               (*ahObj.add(auiIndices[j as usize] as usize) as ULONG_PTR)
            {
                tmp = auiIndices[i as usize];
                auiIndices[i as usize] = auiIndices[j as usize];
                auiIndices[j as usize] = tmp;
            }
            j += 1;
        }
        i += 1;
    }

    i = 0;
    while i < ulCount {
        if *ahObj.add(auiIndices[i as usize] as usize) == core::ptr::null_mut() {
            *apObj.add(auiIndices[i as usize] as usize) = core::ptr::null_mut();
            i += 1;
            continue;
        }

        *apObj.add(auiIndices[i as usize] as usize) = 
            GDIOBJ_LockObject(*ahObj.add(auiIndices[i as usize] as usize), objt);

        if *apObj.add(auiIndices[i as usize] as usize) == core::ptr::null_mut() {
            while i > 0 {
                i -= 1;
                if !(*apObj.add(auiIndices[i as usize] as usize)).is_null() {
                    GDIOBJ_vUnlockObject(*apObj.add(auiIndices[i as usize] as usize) as POBJ);
                }
            }
            return 0;
        }
        i += 1;
    }

    return 1;
}
#[no_mangle]
pub unsafe extern "system" fn GDIOBJ_pvGetObjectAttr(pobj: POBJ) -> PVOID {
    let ulIndex: ULONG = GDI_HANDLE_GET_INDEX((*pobj).hHmgr);
    (*gpentHmgr.offset(ulIndex as isize)).pUser
}
#[no_mangle]
pub unsafe extern "system" fn GDIOBJ_vSetObjectAttr(pobj: POBJ, pvObjAttr: PVOID) {

    let ulIndex: ULONG = GDI_HANDLE_GET_INDEX((*pobj).hHmgr);
    (*gpentHmgr.offset(ulIndex as isize)).pUser = pvObjAttr;
}
#[no_mangle]
pub unsafe extern "system" fn GDIOBJ_vDeleteObject(pobj: POBJ) {

    const EVENT_DELETE: u32 = 1;

    let ulIndex: u32;

    InterlockedOr16(
        core::ptr::addr_of_mut!((*pobj).BaseFlags) as *mut i16,
        BASEFLAG_READY_TO_DIE as i16,
    );
    let _ = 0;

    ulIndex = GDI_HANDLE_GET_INDEX((*pobj).hHmgr);
    if ulIndex != 0 {
        InterlockedAnd(
            gpaulRefCount.add(ulIndex as usize) as *mut i32,
            (!REF_MASK_VALID) as i32,
        );

        if (*pobj).cExclusiveLock != 0 {
            (*pobj).__bindgen_anon_1.dwThreadId = 0;
            (*pobj).cExclusiveLock = 0;

            ExReleasePushLockExclusive(core::ptr::addr_of_mut!((*pobj).pushlock));
            KeLeaveCriticalRegion();
            DECREASE_THREAD_LOCK_COUNT((*pobj).hHmgr);
        }
    }

    GDIOBJ_vDereferenceObject(pobj);
}
#[no_mangle]
pub unsafe extern "system" fn GreIsHandleValid(hobj: HGDIOBJ) -> BOOL {
    let pentry: PENTRY;

    pentry = ENTRY_ReferenceEntryByHandle(hobj, 0);
    if pentry.is_null() {
        return 0;
    }
    GDIOBJ_vDereferenceObject((*pentry).einfo.pobj);
    return 1;
}
#[no_mangle]
pub unsafe extern "system" fn GreDeleteObject(hobj: HGDIOBJ) -> BOOL {

    let pentry: PENTRY;

    if GDI_HANDLE_IS_STOCKOBJ(hobj) != 0 {
        return 0;
    }

    pentry = ENTRY_ReferenceEntryByHandle(hobj, 0);
    if pentry.is_null() {
        return 0;
    }

    if (*pentry).ObjectOwner.ulObj == GDI_OBJ_HMGR_PUBLIC {
        GDIOBJ_vDereferenceObject((*pentry).einfo.pobj);
        return 0;
    }

    GDIOBJ_vDeleteObject((*pentry).einfo.pobj);
    return 1;
}
#[no_mangle]
pub unsafe extern "system" fn GreGetObjectOwner(hobj: HGDIOBJ) -> ULONG {
    let ulIndex: ULONG;
    let ulOwner: ULONG;

    if hobj.is_null() {
        return GDI_OBJ_HMGR_RESTRICTED;
    }

    ulIndex = GDI_HANDLE_GET_INDEX(hobj);

    if ulIndex >= GDI_HANDLE_COUNT ||
        (*gpentHmgr.offset(ulIndex as isize)).Objt == GDIObjType_DEF_TYPE as u8 ||
        ((hobj as ULONG_PTR) >> 16) != (*gpentHmgr.offset(ulIndex as isize)).FullUnique as ULONG_PTR
    {
        return GDI_OBJ_HMGR_RESTRICTED;
    }

    ulOwner = (*gpentHmgr.offset(ulIndex as isize)).ObjectOwner.ulObj;

    if ulOwner == HandleToUlong(PsGetCurrentProcessId()) {
        return GDI_OBJ_HMGR_POWNED;
    }

    if ulOwner == GDI_OBJ_HMGR_PUBLIC {
        return GDI_OBJ_HMGR_PUBLIC;
    }

    return GDI_OBJ_HMGR_RESTRICTED;
}
#[no_mangle]
pub unsafe extern "system" fn GreSetObjectOwnerEx(
    hobj: HGDIOBJ,
    ulOwner: u32,
    Flags: u32,
) -> i32 {

    let pentry: PENTRY;

    if GDI_HANDLE_IS_STOCKOBJ(hobj) != 0 {
        return 0;
    }

    pentry = ENTRY_ReferenceEntryByHandle(hobj, Flags);
    if pentry.is_null() {
        return 0;
    }

    GDIOBJ_vSetObjectOwner((*pentry).einfo.pobj, ulOwner);

    GDIOBJ_vDereferenceObject((*pentry).einfo.pobj);

    return 1;
}
#[no_mangle]
pub unsafe extern "system" fn GreSetObjectOwner(
    hobj: HGDIOBJ,
    ulOwner: u32,
) -> i32 {
    GreSetObjectOwnerEx(hobj, ulOwner, 0)
}
#[no_mangle]
pub unsafe extern "system" fn GreGetObject(
    hobj: HGDIOBJ,
    cbCount: i32,
    pvBuffer: *mut core::ffi::c_void,
) -> i32 {

    let mut iResult: i32 = 0;

    let objt: u8 = (((hobj as usize) >> 16) & 0x1f) as u8;
    if objt != GDIObjType_BRUSH_TYPE as u8 &&
       objt != GDIObjType_SURF_TYPE as u8 &&
       objt != GDIObjType_LFONT_TYPE as u8 &&
       objt != GDIObjType_PAL_TYPE as u8
    {
        return 0;
    }

    let pvObj = GDIOBJ_ReferenceObjectByHandle(hobj, objt);
    if pvObj.is_null()
    {
        return 0;
    }

    match GDI_HANDLE_GET_TYPE(hobj) {
        GDILoObjType_LO_PEN_TYPE | GDILoObjType_LO_EXTPEN_TYPE => {
            iResult = PEN_GetObject(pvObj as *mut c_void, cbCount, pvBuffer);
        }

        GDILoObjType_LO_BRUSH_TYPE => {
            iResult = BRUSH_GetObject(pvObj as PBRUSH, cbCount, pvBuffer as LPLOGBRUSH);
        }

        GDILoObjType_LO_BITMAP_TYPE => {
            iResult = BITMAP_GetObject(pvObj as *mut SURFACE, cbCount, pvBuffer as LPVOID);
        }

        GDILoObjType_LO_FONT_TYPE => {
            iResult = FontGetObject(pvObj as *mut c_void, cbCount, pvBuffer);
        }

        GDILoObjType_LO_PALETTE_TYPE => {
            iResult = PALETTE_GetObject(pvObj as PPALETTE, cbCount, pvBuffer as LPLOGBRUSH);
        }

        _ => {}
    }

    GDIOBJ_vDereferenceObject(pvObj);
    iResult
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiExtGetObjectW(
    hobj: HANDLE,
    cjBufferSize: i32,
    lpBuffer: LPVOID,
) -> i32 {

    let mut iResult: u32;
    let mut cjMaxSize: u32;
    
    #[repr(C)]
    union ObjectUnion {
        bitmap: BITMAP,
        dibsection: DIBSECTION,
        logpen: LOGPEN,
        logbrush: LOGBRUSH,
        logfontw: LOGFONTW,
        extlogfontw: EXTLOGFONTW,
        enumlogfontexdvw: ENUMLOGFONTEXDVW,
    }
    
    let mut object: ObjectUnion = core::mem::zeroed();

    cjMaxSize = min(cjBufferSize as u32, core::mem::size_of::<ObjectUnion>() as u32);

    iResult = GreGetObject(
        hobj,
        cjMaxSize as i32,
        if !lpBuffer.is_null() {
            core::ptr::addr_of_mut!(object) as *mut core::ffi::c_void
        } else {
            core::ptr::null_mut()
        },
    ) as u32;

    if !lpBuffer.is_null() && iResult != 0 {
        cjMaxSize = min(cjMaxSize, iResult);

        // The C probes lpBuffer for WRITE at cjMaxSize with sizeof(WORD) alignment
        // inside a _SEH2_TRY, returning 0 from the handler. The probe was dropped, so
        // this copied up to sizeof(object) bytes of KERNEL STACK to whatever address
        // the caller passed — a kernel-stack disclosure and an arbitrary kernel write
        // in the same statement, from an unprivileged GDI call.
        if !NT_SUCCESS(SafeProbeForWrite(
            lpBuffer as *mut core::ffi::c_void,
            cjMaxSize as usize,
            core::mem::size_of::<WORD>() as u32,
        )) {
            return 0;
        }

        // The probe above VALIDATES only. Given what the comment says this statement
        // can do -- copy kernel stack to a caller-chosen address -- the publish must be
        // the protected one, and a fault takes the probe's own exit.
        if !crate::copy::publish_to_user(
            lpBuffer as *mut core::ffi::c_void,
            core::ptr::addr_of!(object) as *const core::ffi::c_void,
            cjMaxSize as usize,
        ) {
            return 0;
        }
    }

    iResult as i32
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiCreateClientObj(
    ulType: ULONG,
) -> HANDLE {

    let pObject: POBJ;
    let handle: HANDLE;

    if (ulType != GDILoObjType_LO_METAFILE16_TYPE) &&
        (ulType != GDILoObjType_LO_METAFILE_TYPE) &&
        (ulType != GDILoObjType_LO_METADC16_TYPE)
    {
        return core::ptr::null_mut();
    }

    pObject = GDIOBJ_AllocateObject(GDIObjType_CLIENTOBJ_TYPE as u8,
                                    core::mem::size_of::<CLIENTOBJ>() as u32,
                                    BASEFLAG_LOOKASIDE);
    if pObject.is_null()
    {
        return core::ptr::null_mut();
    }

    (*pObject).hHmgr = UlongToHandle(ulType | GDILoObjType_LO_CLIENTOBJ_TYPE);

    handle = GDIOBJ_hInsertObject(pObject, GDI_OBJ_HMGR_POWNED);
    if handle.is_null()
    {
        GDIOBJ_vFreeObject(pObject);
        return core::ptr::null_mut();
    }

    GDIOBJ_vUnlockObject(pObject);

    return handle;
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiDeleteClientObj(hobj: HANDLE) -> BOOL {

    let ulType: ULONG = GDI_HANDLE_GET_TYPE(hobj);

    if (ulType & GDI_HANDLE_BASETYPE_MASK) != GDILoObjType_LO_CLIENTOBJ_TYPE {
        return 0;
    }

    return GreDeleteObject(hobj);
}
#[no_mangle]
// #[inline(never)] restores the C's CODEGEN SHAPE, and is load-bearing (§9.99, middle
// row). This returns NULL on a bad handle and engbrush.rs's EBRUSHOBJ_bRealizeBrush
// derefs the result unchecked -- which is exactly what the C does: engbrush.c has only
// `ASSERT(psurfPattern)`, and ASSERT compiles to nothing in retail. Inlined, LLVM sees
// NULL reach a deref, calls it UB, and folds the tail to `unreachable` -> ud2. Kept
// out of line, the caller gets an opaque pointer, emits the ordinary load, and faults
// where C faults. Do NOT "fix" this with a null check here: that is a behaviour
// divergence on the path the C would fault on.
#[inline(never)]
pub unsafe extern "system" fn GDIOBJ_ShareLockObj(
    hObj: HGDIOBJ,
    ExpectedType: u32,
) -> PGDIOBJ {

    let mut ExpectedType = ExpectedType;
    if ExpectedType == GDI_OBJECT_TYPE_DONTCARE {
        ExpectedType = GDI_HANDLE_GET_TYPE(hObj);
    }
    GDIOBJ_ReferenceObjectByHandle(hObj, ((ExpectedType >> 16) & 0x1f) as u8) as PGDIOBJ
}
#[no_mangle]
pub unsafe extern "system" fn GDIOBJ_ConvertToStockObj(phObj: *mut HGDIOBJ) -> BOOL {
    let pentry: PENTRY;
    let pobj: POBJ;

    pentry = ENTRY_ReferenceEntryByHandle(*phObj, 0);
    if pentry.is_null() {
        return 0;
    }

    (*pentry).FullUnique |= GDI_ENTRY_STOCK_MASK as u16;
    (*pentry).ObjectOwner.ulObj = 0;

    pobj = (*pentry).einfo.pobj;

    (*pobj).hHmgr = ((*pobj).hHmgr as usize | GDI_HANDLE_STOCK_MASK as usize) as HGDIOBJ;

    *phObj = (*pobj).hHmgr;

    GDIOBJ_vDereferenceObject(pobj);

    return 1;
}
#[no_mangle]
pub unsafe extern "system" fn GDIOBJ_ConvertFromStockObj(phObj: *mut HGDIOBJ) -> BOOL {
    let pentry: PENTRY;
    let pobj: POBJ;

    pentry = ENTRY_ReferenceEntryByHandle(*phObj, 0);
    if pentry.is_null() {
        return 0;
    }

    (*pentry).FullUnique &= !(GDI_ENTRY_STOCK_MASK as u16);
    (*pentry).ObjectOwner.ulObj = PtrToUlong(PsGetCurrentProcessId() as *mut core::ffi::c_void);

    pobj = (*pentry).einfo.pobj;

    (*pobj).hHmgr = ((*pobj).hHmgr as usize & !(GDI_HANDLE_STOCK_MASK as usize)) as HGDIOBJ;

    *phObj = (*pobj).hHmgr;

    GDIOBJ_vDereferenceObject(pobj);

    return 1;
}
#[no_mangle]
pub unsafe extern "system" fn GDIOBJ_AllocObjWithHandle(
    ObjectType: ULONG,
    cjSize: ULONG,
) -> POBJ {
    let pobj: POBJ;
    let mut fl: FLONG = 0;
    let objt: UCHAR = ((ObjectType >> 16) & 0xFF) as UCHAR;

    if (objt == GDIObjType_DC_TYPE as u8 && cjSize == core::mem::size_of::<DC>() as u32) ||
       (objt == GDIObjType_PAL_TYPE as u8 && cjSize == core::mem::size_of::<PALETTE>() as u32) ||
       (objt == GDIObjType_RGN_TYPE as u8 && cjSize == core::mem::size_of::<REGION>() as u32) ||
       (objt == GDIObjType_SURF_TYPE as u8 && cjSize == core::mem::size_of::<SURFACE>() as u32) ||
       (objt == GDIObjType_PATH_TYPE as u8 && cjSize == core::mem::size_of::<PATH>() as u32)
    {
        fl |= BASEFLAG_LOOKASIDE;
    }

    pobj = GDIOBJ_AllocateObject(objt, cjSize, fl);
    if pobj.is_null() {
        return core::ptr::null_mut();
    }

    if GDIOBJ_hInsertObject(pobj, GDI_OBJ_HMGR_POWNED).is_null() {
        GDIOBJ_vFreeObject(pobj);
        return core::ptr::null_mut();
    }
    return pobj;
}

#[no_mangle]
pub unsafe extern "system" fn GDI_MapHandleTable(pProcess: PEPROCESS) -> PVOID {
    let mut pvMappedView: PVOID = core::ptr::null_mut();
    let Status: NTSTATUS;
    let mut liOffset: LARGE_INTEGER = core::mem::zeroed();
    let mut cjViewSize: SIZE_T = core::mem::size_of::<GDI_HANDLE_TABLE>() as SIZE_T;

    liOffset.QuadPart = 0;

    Status = MmMapViewOfSection(
        gpvGdiHdlTblSection,
        pProcess,
        &mut pvMappedView,
        crate::init::W32Wow64ZeroBits(pProcess) as _,
        0,
        &mut liOffset,
        &mut cjViewSize,
        ViewUnmap,
        SEC_NO_CHANGE,
        PAGE_READONLY,
    );

    if NT_SUCCESS(Status) == false {
        return core::ptr::null_mut();
    }

    return pvMappedView;
}

#[no_mangle]
pub unsafe extern "system" fn GDI_CleanupForProcess(Process: *mut _EPROCESS) -> u8 {

    let mut pentry: PENTRY;
    let mut ulIndex: u32;
    let dwProcessId: u32;
    let ppi: PPROCESSINFO;

    dwProcessId = PtrToUlong(PsGetCurrentProcessId() as *mut core::ffi::c_void);

    ulIndex = RESERVE_ENTRIES_COUNT;
    while ulIndex < gulFirstUnused {
        pentry = gpentHmgr.add(ulIndex as usize);

        if (*pentry).ObjectOwner.ulObj == dwProcessId {
            InterlockedIncrement(gpaulRefCount.add(ulIndex as usize) as *mut i32);
            GDIOBJ_vDeleteObject((*pentry).einfo.pobj);
        }

        ulIndex = ulIndex + 1;
    }

    // #if DBG only — DbgGdiHTIntegrityCheck is a no-op in the release build.
    DbgGdiHTIntegrityCheck();

    ppi = PsGetCurrentProcessWin32Process();

    // NOTE: the GDIHandleCount leak check and the leak-reporting loop below are
    // NOT #if DBG in the C — they execute unconditionally. Only their bodies
    // (DPRINT1 / ASSERT(FALSE) / DBG_DUMP_EVENT_LIST) are no-ops in release.
    if (*(ppi as *mut W32PROCESS)).GDIHandleCount != 0 {
        // DPRINT1 / ASSERT(FALSE) — no-ops in the release build.
    }

    ulIndex = RESERVE_ENTRIES_COUNT;
    while ulIndex < gulFirstUnused {
        pentry = gpentHmgr.add(ulIndex as usize);

        if (*pentry).ObjectOwner.ulObj == dwProcessId {
            // DPRINT1 / DBG_DUMP_EVENT_LIST / ASSERT(FALSE) — no-ops in release.
        }

        ulIndex = ulIndex + 1;
    }

    1u8
}

#[no_mangle]
pub unsafe extern "C" fn GetBrushAttrPool() -> PGDI_POOL {
    let ppi: PPROCESSINFO = PsGetCurrentProcessWin32Process();
    
    (*ppi).pPoolBrushAttr
}

// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         ReactOS win32 kernel mode subsystem
//  * LICENSE:         GPL - See COPYING in the top level directory
//  * FILE:            win32ss/gdi/ntgdi/gdiobj.c
//  * PURPOSE:         General GDI object manipulation routines
//  * PROGRAMMERS:     Timo Kreuzer
//  */
//
// /*
//  * If you want to understand this code, you need to start thinking in portals.
//  * - gpaulRefCount is a global pointer to an allocated array of ULONG values,
//  * one for each handle. Bits 0 - 22 contain a reference count for the handle.
//  * It gets increased for each handle lock / reference. Bit 23 contains a valid
//  * bit. If this bit is 0, the handle got deleted and will be pushed to the free
//  * list, once all references are gone. Bits 24 - 31 contain the reuse value of
//  * the handle, which allows to check if the entry was changed before atomically
//  * exchanging the reference count.
//  * - Objects can exist with or without a handle
//  *   - Objects with a handle can be locked either exclusively or shared.
//  *     Both locks increase the handle reference count in gpaulRefCount.
//  *     Exclusive locks also increase the BASEOBJECT's cExclusiveLock field
//  *     and the first lock (can be acquired recursively) acquires a pushlock
//  *     that is also stored in the BASEOBJECT.
//  *   - Objects without a handle cannot have exclusive locks. Their reference
//  *     count is tracked in the BASEOBJECT's ulShareCount field.
//  * - An object that is inserted in the handle table automatically has an
//  *   exclusive lock. For objects that are "shared objects" (BRUSH, PALETTE, ...)
//  *   this is the only way it can ever be exclusively locked. It prevents the
//  *   object from being locked by another thread. A shared lock will simply fail,
//  *   while an exclusive lock will succeed after the object was unlocked.
//  *
//  * Ownership:
//  *
//  * Owner:               POWNED  PUBLIC  NONE    spec
//  * ---------------------------------------------------
//  * LockForRead          +       +       -       PUBLIC
//  * LockForWrite         +       -       -       POWNED
//  * LockAny              +       +       +       NONE
//  * NtGdiDeleteObjectApp +       -       -       PUBLIC
//  * GreDeleteObject      +       +       +       NONE
//  * GreSetOwner(POWNED)  -       -       +       -
//  * GreSetOwner(PUBLIC)  +       -       +       -
//  * GreSetOwner(NONE)    +       -       -       -
//  *
//  */
//
// /* INCLUDES ******************************************************************/
//
// #include <win32k.h>
// #define NDEBUG
// #include <debug.h>
//
// FORCEINLINE
// ULONG
// InterlockedReadUlong(
//     _In_ _Interlocked_operand_ ULONG volatile *Source)
// {
//     return *Source;
// }
//
// FORCEINLINE
// void
// INCREASE_THREAD_LOCK_COUNT(
//     _In_ HANDLE hobj)
// {
//     PTHREADINFO pti = PsGetCurrentThreadWin32Thread();
//     DBG_UNREFERENCED_PARAMETER(hobj);
//     if (pti)
//     {
// #if DBG
//         pti->acExclusiveLockCount[((ULONG_PTR)hobj >> 16) & 0x1f]++;
// #endif
//         pti->cExclusiveLocks++;
//     }
// }
//
// FORCEINLINE
// void
// DECREASE_THREAD_LOCK_COUNT(
//     _In_ HANDLE hobj)
// {
//     PTHREADINFO pti = PsGetCurrentThreadWin32Thread();
//     DBG_UNREFERENCED_PARAMETER(hobj);
//     if (pti)
//     {
// #if DBG
//         pti->acExclusiveLockCount[((ULONG_PTR)hobj >> 16) & 0x1f]--;
// #endif
//         pti->cExclusiveLocks--;
//     }
// }
//
// #if DBG
// VOID
// ASSERT_LOCK_ORDER(
//     _In_ UCHAR objt)
// {
//     PTHREADINFO pti = PsGetCurrentThreadWin32Thread();
//     ULONG i;
//
//     if (pti)
//     {
//         /* Ensure correct locking order! */
//         for (i = objt + 1; i < GDIObjTypeTotal; i++)
//         {
//             NT_ASSERT(pti->acExclusiveLockCount[i] == 0);
//         }
//     }
// }
// #define ASSERT_SHARED_OBJECT_TYPE(objt) \
//     ASSERT((objt) == GDIObjType_SURF_TYPE || \
//            (objt) == GDIObjType_PAL_TYPE || \
//            (objt) == GDIObjType_LFONT_TYPE || \
//            (objt) == GDIObjType_PATH_TYPE || \
//            (objt) == GDIObjType_BRUSH_TYPE)
// #define ASSERT_EXCLUSIVE_OBJECT_TYPE(objt) \
//     ASSERT((objt) == GDIObjType_DC_TYPE || \
//            (objt) == GDIObjType_RGN_TYPE || \
//            (objt) == GDIObjType_UMPD_TYPE || \
//            (objt) == GDIObjType_META_TYPE)
// #define ASSERT_TRYLOCK_OBJECT_TYPE(objt) \
//     ASSERT((objt) == GDIObjType_DRVOBJ_TYPE)
// #else
// #define ASSERT_LOCK_ORDER(hobj)
// #define ASSERT_SHARED_OBJECT_TYPE(objt)
// #define ASSERT_EXCLUSIVE_OBJECT_TYPE(objt)
// #define ASSERT_TRYLOCK_OBJECT_TYPE(objt)
// #endif
//
// #if defined(_M_IX86) || defined(_M_AMD64)
// #define InterlockedOr16 _InterlockedOr16
// #endif
//
// #define GDIOBJ_POOL_TAG(type) ('00hG' + (((type) & 0x1f) << 24))
//
// enum
// {
//     REF_MASK_REUSE = 0xff000000,
//     REF_INC_REUSE  = 0x01000000,
//     REF_MASK_VALID = 0x00800000,
//     REF_MASK_COUNT = 0x007fffff,
//     REF_MASK_INUSE = 0x00ffffff,
// };
//
// /* GLOBALS *******************************************************************/
//
// /* Per session handle table globals */
// static PVOID gpvGdiHdlTblSection = NULL;
// PENTRY gpentHmgr;
// PULONG gpaulRefCount;
// volatile ULONG gulFirstFree;
// volatile ULONG gulFirstUnused;
// static PPAGED_LOOKASIDE_LIST gpaLookasideList;
//
// static VOID NTAPI GDIOBJ_vCleanup(PVOID ObjectBody);
//
// static const
// GDICLEANUPPROC
// apfnCleanup[] =
// {
//     NULL,              /* 00 GDIObjType_DEF_TYPE */
//     DC_vCleanup,       /* 01 GDIObjType_DC_TYPE */
//     NULL,              /* 02 GDIObjType_UNUSED1_TYPE */
//     NULL,              /* 03 GDIObjType_UNUSED2_TYPE */
//     REGION_vCleanup,   /* 04 GDIObjType_RGN_TYPE */
//     SURFACE_vCleanup,  /* 05 GDIObjType_SURF_TYPE */
//     GDIOBJ_vCleanup,   /* 06 GDIObjType_CLIENTOBJ_TYPE */
//     GDIOBJ_vCleanup,   /* 07 GDIObjType_PATH_TYPE */
//     PALETTE_vCleanup,  /* 08 GDIObjType_PAL_TYPE */
//     GDIOBJ_vCleanup,   /* 09 GDIObjType_ICMLCS_TYPE */
//     GDIOBJ_vCleanup,   /* 0a GDIObjType_LFONT_TYPE */
//     NULL,              /* 0b GDIObjType_RFONT_TYPE, unused */
//     NULL,              /* 0c GDIObjType_PFE_TYPE, unused */
//     NULL,              /* 0d GDIObjType_PFT_TYPE, unused */
//     GDIOBJ_vCleanup,   /* 0e GDIObjType_ICMCXF_TYPE */
//     NULL,              /* 0f GDIObjType_SPRITE_TYPE, unused */
//     NULL,              /* 10 GDIObjType_BRUSH_TYPE, BRUSH, PEN, EXTPEN */
//     NULL,              /* 11 GDIObjType_UMPD_TYPE, unused */
//     NULL,              /* 12 GDIObjType_UNUSED4_TYPE */
//     NULL,              /* 13 GDIObjType_SPACE_TYPE, unused */
//     NULL,              /* 14 GDIObjType_UNUSED5_TYPE */
//     GDIOBJ_vCleanup,   /* 15 GDIObjType_META_TYPE */
//     NULL,              /* 16 GDIObjType_EFSTATE_TYPE, unused */
//     NULL,              /* 17 GDIObjType_BMFD_TYPE, unused */
//     NULL,              /* 18 GDIObjType_VTFD_TYPE, unused */
//     NULL,              /* 19 GDIObjType_TTFD_TYPE, unused */
//     NULL,              /* 1a GDIObjType_RC_TYPE, unused */
//     NULL,              /* 1b GDIObjType_TEMP_TYPE, unused */
//     DRIVEROBJ_vCleanup,/* 1c GDIObjType_DRVOBJ_TYPE */
//     NULL,              /* 1d GDIObjType_DCIOBJ_TYPE, unused */
//     NULL,              /* 1e GDIObjType_SPOOL_TYPE, unused */
//     NULL,              /* 1f reserved entry */
// };
//
// static const
// GDIOBJDELETEPROC
// apfnDelete[] =
// {
//     NULL,              /* 00 GDIObjType_DEF_TYPE */
//     NULL,              /* 01 GDIObjType_DC_TYPE */
//     NULL,              /* 02 GDIObjType_UNUSED1_TYPE */
//     NULL,              /* 03 GDIObjType_UNUSED2_TYPE */
//     NULL,              /* 04 GDIObjType_RGN_TYPE */
//     NULL,              /* 05 GDIObjType_SURF_TYPE */
//     NULL,              /* 06 GDIObjType_CLIENTOBJ_TYPE */
//     NULL,              /* 07 GDIObjType_PATH_TYPE */
//     NULL,              /* 08 GDIObjType_PAL_TYPE */
//     NULL,              /* 09 GDIObjType_ICMLCS_TYPE */
//     NULL,              /* 0a GDIObjType_LFONT_TYPE */
//     NULL,              /* 0b GDIObjType_RFONT_TYPE, unused */
//     NULL,              /* 0c GDIObjType_PFE_TYPE, unused */
//     NULL,              /* 0d GDIObjType_PFT_TYPE, unused */
//     NULL,              /* 0e GDIObjType_ICMCXF_TYPE */
//     NULL,                 /* 0f GDIObjType_SPRITE_TYPE, unused */
//     BRUSH_vDeleteObject,  /* 10 GDIObjType_BRUSH_TYPE, BRUSH, PEN, EXTPEN */
//     NULL,              /* 11 GDIObjType_UMPD_TYPE, unused */
//     NULL,              /* 12 GDIObjType_UNUSED4_TYPE */
//     NULL,              /* 13 GDIObjType_SPACE_TYPE, unused */
//     NULL,              /* 14 GDIObjType_UNUSED5_TYPE */
//     NULL,              /* 15 GDIObjType_META_TYPE, unused */
//     NULL,              /* 16 GDIObjType_EFSTATE_TYPE, unused */
//     NULL,              /* 17 GDIObjType_BMFD_TYPE, unused */
//     NULL,              /* 18 GDIObjType_VTFD_TYPE, unused */
//     NULL,              /* 19 GDIObjType_TTFD_TYPE, unused */
//     NULL,              /* 1a GDIObjType_RC_TYPE, unused */
//     NULL,              /* 1b GDIObjType_TEMP_TYPE, unused */
//     NULL,              /* 1c GDIObjType_DRVOBJ_TYPE */
//     NULL,              /* 1d GDIObjType_DCIOBJ_TYPE, unused */
//     NULL,              /* 1e GDIObjType_SPOOL_TYPE, unused */
//     NULL,              /* 1f reserved entry */
// };
//
// /* INTERNAL FUNCTIONS ********************************************************/
//
// static
// VOID
// NTAPI
// GDIOBJ_vCleanup(PVOID ObjectBody)
// {
//     /* Nothing to do */
// }
//
// static
// VOID
// InitLookasideList(UCHAR objt, ULONG cjSize)
// {
//     ExInitializePagedLookasideList(&gpaLookasideList[objt],
//                                    NULL,
//                                    NULL,
//                                    0,
//                                    cjSize,
//                                    GDITAG_HMGR_LOOKASIDE_START + (objt << 24),
//                                    0);
// }
//
// CODE_SEG("INIT")
// NTSTATUS
// NTAPI
// InitGdiHandleTable(void)
// {
//     NTSTATUS status;
//     LARGE_INTEGER liSize;
//     PVOID pvSection;
//     SIZE_T cjViewSize = 0;
//
//     /* Create a section for the shared handle table */
//     liSize.QuadPart = sizeof(GDI_HANDLE_TABLE); // GDI_HANDLE_COUNT * sizeof(ENTRY);
//     status = MmCreateSection(&gpvGdiHdlTblSection,
//                              SECTION_ALL_ACCESS,
//                              NULL,
//                              &liSize,
//                              PAGE_READWRITE,
//                              SEC_COMMIT | 0x1,
//                              NULL,
//                              NULL);
//     if (!NT_SUCCESS(status))
//     {
//         DPRINT1("INITGDI: Could not allocate a GDI handle table.\n");
//         return status;
//     }
//
//     /* Map the section in session space */
//     status = MmMapViewInSessionSpace(gpvGdiHdlTblSection,
//                                      (PVOID*)&gpentHmgr,
//                                      &cjViewSize);
//     if (!NT_SUCCESS(status))
//     {
//         DPRINT1("INITGDI: Failed to map handle table section\n");
//         ObDereferenceObject(gpvGdiHdlTblSection);
//         return status;
//     }
//
//     /* Allocate memory for the reference counter table */
//     gpaulRefCount = EngAllocSectionMem(&pvSection,
//                                      FL_ZERO_MEMORY,
//                                      GDI_HANDLE_COUNT * sizeof(ULONG),
//                                      'frHG');
//     if (!gpaulRefCount)
//     {
//         DPRINT1("INITGDI: Failed to allocate reference table.\n");
//         ObDereferenceObject(gpvGdiHdlTblSection);
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     gulFirstFree = 0;
//     gulFirstUnused = RESERVE_ENTRIES_COUNT;
//
//     GdiHandleTable = (PVOID)gpentHmgr;
//
//     /* Initialize the lookaside lists */
//     gpaLookasideList = ExAllocatePoolWithTag(NonPagedPool,
//                            GDIObjTypeTotal * sizeof(PAGED_LOOKASIDE_LIST),
//                            TAG_GDIHNDTBLE);
//     if(!gpaLookasideList)
//         return STATUS_NO_MEMORY;
//
//     InitLookasideList(GDIObjType_DC_TYPE, sizeof(DC));
//     InitLookasideList(GDIObjType_RGN_TYPE, sizeof(REGION));
//     InitLookasideList(GDIObjType_SURF_TYPE, sizeof(SURFACE));
//     InitLookasideList(GDIObjType_CLIENTOBJ_TYPE, sizeof(CLIENTOBJ));
//     InitLookasideList(GDIObjType_PATH_TYPE, sizeof(PATH));
//     InitLookasideList(GDIObjType_PAL_TYPE, sizeof(PALETTE));
//     InitLookasideList(GDIObjType_ICMLCS_TYPE, sizeof(COLORSPACE));
//     InitLookasideList(GDIObjType_LFONT_TYPE, sizeof(TEXTOBJ));
//     InitLookasideList(GDIObjType_BRUSH_TYPE, sizeof(BRUSH));
//
//     return STATUS_SUCCESS;
// }
//
// FORCEINLINE
// VOID
// IncrementCurrentProcessGdiHandleCount(void)
// {
//     PPROCESSINFO ppi = PsGetCurrentProcessWin32Process();
//     if (ppi) InterlockedIncrement((LONG*)&ppi->GDIHandleCount);
// }
//
// FORCEINLINE
// VOID
// DecrementCurrentProcessGdiHandleCount(void)
// {
//     PPROCESSINFO ppi = PsGetCurrentProcessWin32Process();
//     if (ppi) InterlockedDecrement((LONG*)&ppi->GDIHandleCount);
// }
//
// static inline
// VOID
// IncrementGdiHandleCount(ULONG ulProcessId)
// {
//     PEPROCESS pep;
//     PPROCESSINFO ppi;
//     NTSTATUS Status;
//
//     Status = PsLookupProcessByProcessId(ULongToHandle(ulProcessId), &pep);
//     NT_ASSERT(NT_SUCCESS(Status));
//     __analysis_assume(NT_SUCCESS(Status));
//
//     ppi = PsGetProcessWin32Process(pep);
//     if (ppi) InterlockedIncrement((LONG*)&ppi->GDIHandleCount);
//     if (NT_SUCCESS(Status)) ObDereferenceObject(pep);
// }
//
// static inline
// VOID
// DecrementGdiHandleCount(ULONG ulProcessId)
// {
//     PEPROCESS pep;
//     PPROCESSINFO ppi;
//     NTSTATUS Status;
//
//     Status = PsLookupProcessByProcessId(ULongToHandle(ulProcessId), &pep);
//     NT_ASSERT(NT_SUCCESS(Status));
//     __analysis_assume(NT_SUCCESS(Status));
//
//     ppi = PsGetProcessWin32Process(pep);
//     if (ppi) InterlockedDecrement((LONG*)&ppi->GDIHandleCount);
//     if (NT_SUCCESS(Status)) ObDereferenceObject(pep);
// }
//
// static
// PENTRY
// ENTRY_pentPopFreeEntry(VOID)
// {
//     ULONG iFirst, iNext, iPrev;
//     PENTRY pentFree;
//
//     DPRINT("Enter InterLockedPopFreeEntry\n");
//
//     do
//     {
//         /* Get the index and sequence number of the first free entry */
//         iFirst = InterlockedReadUlong(&gulFirstFree);
//
//         /* Check if we have a free entry */
//         if (!(iFirst & GDI_HANDLE_INDEX_MASK))
//         {
//             /* Increment FirstUnused and get the new index */
//             iFirst = InterlockedIncrement((LONG*)&gulFirstUnused) - 1;
//
//             /* Check if we have unused entries left */
//             if (iFirst >= GDI_HANDLE_COUNT)
//             {
//                 DPRINT1("No more GDI handles left!\n");
//                 InterlockedDecrement((LONG*)&gulFirstUnused);
//                 return 0;
//             }
//
//             /* Return the old entry */
//             return &gpentHmgr[iFirst];
//         }
//
//         /* Get a pointer to the first free entry */
//         pentFree = &gpentHmgr[iFirst & GDI_HANDLE_INDEX_MASK];
//
//         /* Create a new value with an increased sequence number */
//         iNext = GDI_HANDLE_GET_INDEX(pentFree->einfo.hFree);
//         iNext |= (iFirst & ~GDI_HANDLE_INDEX_MASK) + 0x10000;
//
//         /* Try to exchange the FirstFree value */
//         iPrev = InterlockedCompareExchange((LONG*)&gulFirstFree,
//                                            iNext,
//                                            iFirst);
//     }
//     while (iPrev != iFirst);
//
//     /* Sanity check: is entry really free? */
//     ASSERT(((ULONG_PTR)pentFree->einfo.pobj & ~GDI_HANDLE_INDEX_MASK) == 0);
//
//     return pentFree;
// }
//
// /* Pushes an entry of the handle table to the free list,
//    The entry must not have any references left */
// static
// VOID
// ENTRY_vPushFreeEntry(PENTRY pentFree)
// {
//     ULONG iToFree, iFirst, iPrev, idxToFree;
//
//     DPRINT("Enter ENTRY_vPushFreeEntry\n");
//
//     idxToFree = pentFree - gpentHmgr;
//     ASSERT((gpaulRefCount[idxToFree] & REF_MASK_INUSE) == 0);
//
//     /* Initialize entry */
//     pentFree->Objt = GDIObjType_DEF_TYPE;
//     pentFree->ObjectOwner.ulObj = 0;
//     pentFree->pUser = NULL;
//
//     /* Increase reuse counter in entry and reference counter */
//     InterlockedExchangeAdd((LONG*)&gpaulRefCount[idxToFree], REF_INC_REUSE);
//     pentFree->FullUnique += 0x0100;
//
//     do
//     {
//         /* Get the current first free index and sequence number */
//         iFirst = InterlockedReadUlong(&gulFirstFree);
//
//         /* Set the einfo.pobj member to the index of the first free entry */
//         pentFree->einfo.pobj = UlongToPtr(iFirst & GDI_HANDLE_INDEX_MASK);
//
//         /* Combine new index and increased sequence number in iToFree */
//         iToFree = idxToFree | ((iFirst & ~GDI_HANDLE_INDEX_MASK) + 0x10000);
//
//         /* Try to atomically update the first free entry */
//         iPrev = InterlockedCompareExchange((LONG*)&gulFirstFree,
//                                            iToFree,
//                                            iFirst);
//     }
//     while (iPrev != iFirst);
// }
//
// static
// PENTRY
// ENTRY_ReferenceEntryByHandle(HGDIOBJ hobj, FLONG fl)
// {
//     ULONG ulIndex, cNewRefs, cOldRefs;
//     PENTRY pentry;
//     PTHREADINFO pti = PsGetCurrentThreadWin32Thread();
//
//     /* HACK: This may be a hack but it fixes CORE-5601.
//      * Allow a window that is moving or resizing to have access to all of its child
//      * windows dc's even if the dc belongs to another process i.e. 3D Screensaver */
//     if (pti && pti->TIF_flags & TIF_MOVESIZETRACKING)
//         fl = GDIOBJFLAG_IGNOREPID;
//
//     /* Get the handle index and check if its too big */
//     ulIndex = GDI_HANDLE_GET_INDEX(hobj);
//
//     /* Get pointer to the entry */
//     pentry = &gpentHmgr[ulIndex];
//
//     /* Get the current reference count */
//     cOldRefs = gpaulRefCount[ulIndex];
//
//     do
//     {
//         /* Check if the slot is deleted */
//         if ((cOldRefs & REF_MASK_VALID) == 0)
//         {
//             DPRINT("GDIOBJ: Slot is not valid: 0x%lx, hobh=%p\n", cOldRefs, hobj);
//             return NULL;
//         }
//
//         /* Check if the unique value matches */
//         if (pentry->FullUnique != (USHORT)((ULONG_PTR)hobj >> 16))
//         {
//             DPRINT("GDIOBJ: Wrong unique value. Handle: 0x%4x, entry: 0x%4x\n",
//                    (USHORT)((ULONG_PTR)hobj >> 16), pentry->FullUnique);
//             return NULL;
//         }
//
//         /* Check if the object owner is this process or public */
//         if (!(fl & GDIOBJFLAG_IGNOREPID) &&
//             pentry->ObjectOwner.ulObj != GDI_OBJ_HMGR_PUBLIC &&
//             pentry->ObjectOwner.ulObj != PtrToUlong(PsGetCurrentProcessId()))
//         {
//             DPRINT("GDIOBJ: Cannot reference foreign handle %p, pentry=%p:%lx.\n",
//                     hobj, pentry, pentry->ObjectOwner.ulObj);
//             return NULL;
//         }
//
//         /* Try to atomically increment the reference count */
//         cNewRefs = cOldRefs + 1;
//         cOldRefs = InterlockedCompareExchange((PLONG)&gpaulRefCount[ulIndex],
//                                               cNewRefs,
//                                               cOldRefs);
//     }
//     while (cNewRefs != cOldRefs + 1);
//
//     /* Integrity checks */
//     ASSERT((pentry->FullUnique & 0x1f) == pentry->Objt);
//     ASSERT(pentry->einfo.pobj != NULL);
//
//     /* Check if lower 32 bits match, the upper 32 bits are ignored */
//     ASSERT(pentry->einfo.pobj->hHmgr == UlongToPtr(PtrToUlong(hobj)));
//
//     return pentry;
// }
//
// static
// HGDIOBJ
// ENTRY_hInsertObject(PENTRY pentry, POBJ pobj, UCHAR objt, ULONG ulOwner)
// {
//     ULONG ulIndex;
//
//     /* Calculate the handle index */
//     ulIndex = pentry - gpentHmgr;
//
//     /* Update the fields in the ENTRY */
//     pentry->einfo.pobj = pobj;
//     pentry->Objt = objt & 0x1f;
//     pentry->FullUnique = (pentry->FullUnique & 0xff00) | objt;
//     pentry->ObjectOwner.ulObj = ulOwner;
//
//     /* Make the handle valid with 1 reference */
//     ASSERT((gpaulRefCount[ulIndex] & REF_MASK_INUSE) == 0);
//     InterlockedOr((LONG*)&gpaulRefCount[ulIndex], REF_MASK_VALID | 1);
//
//     /* Return the handle */
//     return (HGDIOBJ)(((ULONG_PTR)pentry->FullUnique << 16) | ulIndex);
// }
//
// POBJ
// NTAPI
// GDIOBJ_AllocateObject(UCHAR objt, ULONG cjSize, FLONG fl)
// {
//     POBJ pobj;
//
//     if (fl & BASEFLAG_LOOKASIDE)
//     {
//         /* Allocate the object from a lookaside list */
//         pobj = ExAllocateFromPagedLookasideList(&gpaLookasideList[objt & 0x1f]);
//     }
//     else
//     {
//         /* Allocate the object from paged pool */
//         pobj = ExAllocatePoolWithTag(PagedPool, cjSize, GDIOBJ_POOL_TAG(objt));
//     }
//
//     if (!pobj) return NULL;
//
//     /* Initialize the object */
//     RtlZeroMemory(pobj, cjSize);
//     pobj->hHmgr = (HGDIOBJ)((ULONG_PTR)objt << 16);
//     pobj->cExclusiveLock = 0;
//     pobj->ulShareCount = 1;
//     pobj->BaseFlags = fl & 0xffff;
//     DBG_INITLOG(&pobj->slhLog);
//     DBG_LOGEVENT(&pobj->slhLog, EVENT_ALLOCATE, 0);
//
//     return pobj;
// }
//
// VOID
// NTAPI
// GDIOBJ_vFreeObject(POBJ pobj)
// {
//     UCHAR objt;
//
//     DBG_CLEANUP_EVENT_LIST(&pobj->slhLog);
//
//     /* Get the object type */
//     objt = ((ULONG_PTR)pobj->hHmgr >> 16) & 0x1f;
//
//     /* Check if we have a delete procedure (for C++ based objects) */
//     if (apfnDelete[objt] != NULL)
//     {
//         /* Invoke the delete procedure */
//         apfnDelete[objt](pobj);
//     }
//     else
//     {
//         /* Call the cleanup procedure */
//         NT_ASSERT(apfnCleanup[objt]);
//         apfnCleanup[objt](pobj);
//
//         /* Check if the object is allocated from a lookaside list */
//         if (pobj->BaseFlags & BASEFLAG_LOOKASIDE)
//         {
//             ExFreeToPagedLookasideList(&gpaLookasideList[objt], pobj);
//         }
//         else
//         {
//             ExFreePoolWithTag(pobj, GDIOBJ_POOL_TAG(objt));
//         }
//     }
// }
//
// VOID
// NTAPI
// GDIOBJ_vDereferenceObject(POBJ pobj)
// {
//     ULONG cRefs, ulIndex;
//
//     /* Calculate the index */
//     ulIndex = GDI_HANDLE_GET_INDEX(pobj->hHmgr);
//
//     /* Check if the object has a handle */
//     if (ulIndex)
//     {
//         /* Decrement reference count */
//         if ((gpaulRefCount[ulIndex] & REF_MASK_COUNT) == 0)
//         {
//             DBG_DUMP_EVENT_LIST(&pobj->slhLog);
//         }
//         ASSERT((gpaulRefCount[ulIndex] & REF_MASK_COUNT) > 0);
//         cRefs = InterlockedDecrement((LONG*)&gpaulRefCount[ulIndex]);
//         DBG_LOGEVENT(&pobj->slhLog, EVENT_DEREFERENCE, cRefs);
//
//         /* Check if we reached 0 and handle bit is not set */
//         if ((cRefs & REF_MASK_INUSE) == 0)
//         {
//             /* Make sure it's ok to delete the object */
//             ASSERT(pobj->BaseFlags & BASEFLAG_READY_TO_DIE);
//
//             /* Check if the handle was process owned */
//             if (gpentHmgr[ulIndex].ObjectOwner.ulObj != GDI_OBJ_HMGR_PUBLIC &&
//                 gpentHmgr[ulIndex].ObjectOwner.ulObj != GDI_OBJ_HMGR_NONE)
//             {
//                 /* Decrement the process handle count */
//                 ASSERT(gpentHmgr[ulIndex].ObjectOwner.ulObj ==
//                        HandleToUlong(PsGetCurrentProcessId()));
//                 DecrementCurrentProcessGdiHandleCount();
//             }
//
//             /* Push entry to the free list */
//             ENTRY_vPushFreeEntry(&gpentHmgr[ulIndex]);
//
//             /* Free the object */
//             GDIOBJ_vFreeObject(pobj);
//         }
//     }
//     else
//     {
//         /* Decrement the objects reference count */
//         ASSERT(pobj->ulShareCount > 0);
//         cRefs = InterlockedDecrement((LONG*)&pobj->ulShareCount);
//         DBG_LOGEVENT(&pobj->slhLog, EVENT_DEREFERENCE, cRefs);
//
//         /* Check if we reached 0 */
//         if (cRefs == 0)
//         {
//             /* Free the object */
//             GDIOBJ_vFreeObject(pobj);
//         }
//     }
// }
//
// POBJ
// NTAPI
// GDIOBJ_ReferenceObjectByHandle(
//     HGDIOBJ hobj,
//     UCHAR objt)
// {
//     PENTRY pentry;
//     POBJ pobj;
//
//     /* Check if the handle type matches */
//     ASSERT_SHARED_OBJECT_TYPE(objt);
//     if ((((ULONG_PTR)hobj >> 16) & 0x1f) != objt)
//     {
//         DPRINT("GDIOBJ: Wrong type. handle=%p, type=%x\n", hobj, objt);
//         return NULL;
//     }
//
//     /* Reference the handle entry */
//     pentry = ENTRY_ReferenceEntryByHandle(hobj, 0);
//     if (!pentry)
//     {
//         DPRINT("GDIOBJ: Requested handle 0x%p is not valid.\n", hobj);
//         return NULL;
//     }
//
//     /* Get the pointer to the BASEOBJECT */
//     pobj = pentry->einfo.pobj;
//
//     /* Check if the object is exclusively locked */
//     if (pobj->cExclusiveLock != 0)
//     {
//         DPRINT1("GDIOBJ: Cannot reference object %p with exclusive lock.\n", hobj);
//         GDIOBJ_vDereferenceObject(pobj);
//         DBG_DUMP_EVENT_LIST(&pobj->slhLog);
//         return NULL;
//     }
//
//     DBG_LOGEVENT(&pobj->slhLog, EVENT_REFERENCE, gpaulRefCount[pentry - gpentHmgr]);
//
//     /* All is well, return the object */
//     return pobj;
// }
//
// VOID
// NTAPI
// GDIOBJ_vReferenceObjectByPointer(POBJ pobj)
// {
//     ULONG cRefs;
//
//     /* Check if the object has a handle */
//     if (GDI_HANDLE_GET_INDEX(pobj->hHmgr))
//     {
//         /* Increase the handle's reference count */
//         ULONG ulIndex = GDI_HANDLE_GET_INDEX(pobj->hHmgr);
//         ASSERT((gpaulRefCount[ulIndex] & REF_MASK_COUNT) > 0);
//         cRefs = InterlockedIncrement((LONG*)&gpaulRefCount[ulIndex]);
//     }
//     else
//     {
//         /* Increase the object's reference count */
//         cRefs = InterlockedIncrement((LONG*)&pobj->ulShareCount);
//     }
//
//     DBG_LOGEVENT(&pobj->slhLog, EVENT_REFERENCE, cRefs);
// }
//
// PGDIOBJ
// NTAPI
// GDIOBJ_TryLockObject(
//     HGDIOBJ hobj,
//     UCHAR objt)
// {
//     PENTRY pentry;
//     POBJ pobj;
//     DWORD dwThreadId;
//
//     /* Check if the handle type matches */
//     ASSERT_TRYLOCK_OBJECT_TYPE(objt);
//     if ((((ULONG_PTR)hobj >> 16) & 0x1f) != objt)
//     {
//         DPRINT("Wrong object type: hobj=0x%p, objt=0x%x\n", hobj, objt);
//         return NULL;
//     }
//
//     /* Make sure lock order is correct */
//     ASSERT_LOCK_ORDER(objt);
//
//     /* Reference the handle entry */
//     pentry = ENTRY_ReferenceEntryByHandle(hobj, 0);
//     if (!pentry)
//     {
//         DPRINT("GDIOBJ: Requested handle 0x%p is not valid.\n", hobj);
//         return NULL;
//     }
//
//     /* Get the pointer to the BASEOBJECT */
//     pobj = pentry->einfo.pobj;
//
//     /* Check if we already own the lock */
//     dwThreadId = PtrToUlong(PsGetCurrentThreadId());
//     if (pobj->dwThreadId != dwThreadId)
//     {
//         /* Disable APCs and try acquiring the push lock */
//         KeEnterCriticalRegion();
//         if(!ExTryAcquirePushLockExclusive(&pobj->pushlock))
//         {
//             ULONG cRefs, ulIndex;
//             /* Already owned. Clean up and leave. */
//             KeLeaveCriticalRegion();
//
//             /* Calculate the index */
//             ulIndex = GDI_HANDLE_GET_INDEX(pobj->hHmgr);
//
//             /* Decrement reference count */
//             ASSERT((gpaulRefCount[ulIndex] & REF_MASK_COUNT) > 0);
//             cRefs = InterlockedDecrement((LONG*)&gpaulRefCount[ulIndex]);
//             ASSERT(cRefs & REF_MASK_VALID);
//
//             return NULL;
//         }
//
//         /* Set us as lock owner */
//         ASSERT(pobj->dwThreadId == 0);
//         pobj->dwThreadId = dwThreadId;
//     }
//
//     /* Increase lock count */
//     pobj->cExclusiveLock++;
//     INCREASE_THREAD_LOCK_COUNT(hobj);
//     DBG_LOGEVENT(&pobj->slhLog, EVENT_LOCK, 0);
//
//     /* Return the object */
//     return pobj;
// }
//
// PGDIOBJ
// NTAPI
// GDIOBJ_LockObject(
//     HGDIOBJ hobj,
//     UCHAR objt)
// {
//     PENTRY pentry;
//     POBJ pobj;
//     DWORD dwThreadId;
//
//     /* Check if the handle type matches */
//     ASSERT_EXCLUSIVE_OBJECT_TYPE(objt);
//     if ((((ULONG_PTR)hobj >> 16) & 0x1f) != objt)
//     {
//         DPRINT("Wrong object type: hobj=0x%p, objt=0x%x\n", hobj, objt);
//         return NULL;
//     }
//
//     /* Make sure lock order is correct */
//     ASSERT_LOCK_ORDER(objt);
//
//     /* Reference the handle entry */
//     pentry = ENTRY_ReferenceEntryByHandle(hobj, 0);
//     if (!pentry)
//     {
//         DPRINT("GDIOBJ: Requested handle 0x%p is not valid.\n", hobj);
//         return NULL;
//     }
//
//     /* Get the pointer to the BASEOBJECT */
//     pobj = pentry->einfo.pobj;
//
//     /* Check if we already own the lock */
//     dwThreadId = PtrToUlong(PsGetCurrentThreadId());
//     if (pobj->dwThreadId != dwThreadId)
//     {
//         /* Disable APCs and acquire the push lock */
//         KeEnterCriticalRegion();
//         ExAcquirePushLockExclusive(&pobj->pushlock);
//
//         /* Set us as lock owner */
//         ASSERT(pobj->dwThreadId == 0);
//         pobj->dwThreadId = dwThreadId;
//     }
//
//     /* Increase lock count */
//     pobj->cExclusiveLock++;
//     INCREASE_THREAD_LOCK_COUNT(hobj);
//     DBG_LOGEVENT(&pobj->slhLog, EVENT_LOCK, 0);
//
//     /* Return the object */
//     return pobj;
// }
//
// VOID
// NTAPI
// GDIOBJ_vUnlockObject(POBJ pobj)
// {
//     ULONG cRefs, ulIndex;
//     ASSERT(pobj->cExclusiveLock > 0);
//
//     /* Decrease lock count */
//     pobj->cExclusiveLock--;
//     DECREASE_THREAD_LOCK_COUNT(pobj->hHmgr);
//     DBG_LOGEVENT(&pobj->slhLog, EVENT_UNLOCK, 0);
//
//     /* Check if this was the last lock */
//     if (pobj->cExclusiveLock == 0)
//     {
//         /* Reset lock owner */
//         pobj->dwThreadId = 0;
//
//         /* Release the pushlock and reenable APCs */
//         ExReleasePushLockExclusive(&pobj->pushlock);
//         KeLeaveCriticalRegion();
//     }
//
//     /* Calculate the index */
//     ulIndex = GDI_HANDLE_GET_INDEX(pobj->hHmgr);
//
//     /* Decrement reference count */
//     ASSERT((gpaulRefCount[ulIndex] & REF_MASK_COUNT) > 0);
//     cRefs = InterlockedDecrement((LONG*)&gpaulRefCount[ulIndex]);
//     ASSERT(cRefs & REF_MASK_VALID);
// }
//
// HGDIOBJ
// NTAPI
// GDIOBJ_hInsertObject(
//     POBJ pobj,
//     ULONG ulOwner)
// {
//     PENTRY pentry;
//     UCHAR objt;
//
//     /* Must have no handle and only one reference */
//     ASSERT(GDI_HANDLE_GET_INDEX(pobj->hHmgr) == 0);
//     ASSERT(pobj->cExclusiveLock == 0);
//     ASSERT(pobj->ulShareCount == 1);
//
//     /* Get a free handle entry */
//     pentry = ENTRY_pentPopFreeEntry();
//     if (!pentry)
//     {
//         DPRINT1("GDIOBJ: Could not get a free entry.\n");
//         return NULL;
//     }
//
//     /* Make the object exclusively locked */
//     ExInitializePushLock(&pobj->pushlock);
//     KeEnterCriticalRegion();
//     ExAcquirePushLockExclusive(&pobj->pushlock);
//     pobj->cExclusiveLock = 1;
//     pobj->dwThreadId = PtrToUlong(PsGetCurrentThreadId());
//     INCREASE_THREAD_LOCK_COUNT(pobj->hHmgr);
//
//     /* Get object type from the hHmgr field */
//     objt = ((ULONG_PTR)pobj->hHmgr >> 16) & 0xff;
//     ASSERT(objt != GDIObjType_DEF_TYPE);
//
//     /* Check if current process is requested owner */
//     if (ulOwner == GDI_OBJ_HMGR_POWNED)
//     {
//         /* Increment the process handle count */
//         IncrementCurrentProcessGdiHandleCount();
//
//         /* Use Process id */
//         ulOwner = HandleToUlong(PsGetCurrentProcessId());
//     }
//
//     /* Insert the object into the handle table */
//     pobj->hHmgr = ENTRY_hInsertObject(pentry, pobj, objt, ulOwner);
//
//     /* Return the handle */
//     DPRINT("GDIOBJ: Created handle: %p\n", pobj->hHmgr);
//     DBG_LOGEVENT(&pobj->slhLog, EVENT_CREATE_HANDLE, 0);
//     return pobj->hHmgr;
// }
//
// VOID
// NTAPI
// GDIOBJ_vSetObjectOwner(
//     POBJ pobj,
//     ULONG ulNewOwner)
// {
//     PENTRY pentry;
//     ULONG ulOldOwner;
//
//     /* This is a ugly HACK, needed to fix IntGdiSetDCOwnerEx */
//     if (GDI_HANDLE_IS_STOCKOBJ(pobj->hHmgr))
//     {
//         DPRINT("Trying to set ownership of stock object %p to %lx\n", pobj->hHmgr, ulNewOwner);
//         return;
//     }
//
//     /* Get the handle entry */
//     NT_ASSERT(GDI_HANDLE_GET_INDEX(pobj->hHmgr));
//     pentry = &gpentHmgr[GDI_HANDLE_GET_INDEX(pobj->hHmgr)];
//
//     /* Check if the new owner is the same as the old one */
//     ulOldOwner = pentry->ObjectOwner.ulObj;
//     if (ulOldOwner == ulNewOwner)
//     {
//         /* Nothing to do */
//         return;
//     }
//
//     /* Is the current process requested? */
//     if (ulNewOwner == GDI_OBJ_HMGR_POWNED)
//     {
//         /* Use process id */
//         ulNewOwner = HandleToUlong(PsGetCurrentProcessId());
//     }
//
//     // HACK
//     if (ulNewOwner == GDI_OBJ_HMGR_NONE)
//         ulNewOwner = GDI_OBJ_HMGR_PUBLIC;
//
//     /* Was the object process owned? */
//     if ((ulOldOwner != GDI_OBJ_HMGR_PUBLIC) &&
//         (ulOldOwner != GDI_OBJ_HMGR_NONE))
//     {
//         /* Decrement the previous owners handle count */
//         DecrementGdiHandleCount(ulOldOwner);
//     }
//
//     /* Is the new owner a process? */
//     if ((ulNewOwner != GDI_OBJ_HMGR_PUBLIC) &&
//         (ulNewOwner != GDI_OBJ_HMGR_NONE))
//     {
//         /* Increment the new owners handle count */
//         IncrementGdiHandleCount(ulNewOwner);
//     }
//     else
//     {
//         /* Make sure we don't leak user mode memory */
//         NT_ASSERT(pentry->pUser == NULL);
//     }
//
//     /* Set new owner */
//     pentry->ObjectOwner.ulObj = ulNewOwner;
//     DBG_LOGEVENT(&pobj->slhLog, EVENT_SET_OWNER, 0);
// }
//
// /* Locks 2 or 3 objects at a time */
// BOOL
// NTAPI
// GDIOBJ_bLockMultipleObjects(
//     IN ULONG ulCount,
//     IN HGDIOBJ* ahObj,
//     OUT PGDIOBJ* apObj,
//     IN UCHAR objt)
// {
//     UINT auiIndices[3] = {0, 1, 2};
//     UINT i, j, tmp;
//
//     ASSERT(ulCount <= 3);
//
//     /* Sort the handles */
//     for (i = 0; i < ulCount - 1; i++)
//     {
//         for (j = i + 1; j < ulCount; j++)
//         {
//             if ((ULONG_PTR)ahObj[auiIndices[i]] <
//                 (ULONG_PTR)ahObj[auiIndices[j]])
//             {
//                 tmp = auiIndices[i];
//                 auiIndices[i] = auiIndices[j];
//                 auiIndices[j] = tmp;
//             }
//         }
//     }
//
//     /* Lock the objects in safe order */
//     for (i = 0; i < ulCount; i++)
//     {
//         /* Skip NULL handles */
//         if (ahObj[auiIndices[i]] == NULL)
//         {
//             apObj[auiIndices[i]] = NULL;
//             continue;
//         }
//
//         /* Lock the object */
//         apObj[auiIndices[i]] = GDIOBJ_LockObject(ahObj[auiIndices[i]], objt);
//
//         /* Check for failure */
//         if (apObj[auiIndices[i]] == NULL)
//         {
//             /* Cleanup */
//             while (i--)
//             {
//                 if (apObj[auiIndices[i]])
//                     GDIOBJ_vUnlockObject(apObj[auiIndices[i]]);
//             }
//             return FALSE;
//         }
//     }
//
//     return TRUE;
// }
//
// PVOID
// NTAPI
// GDIOBJ_pvGetObjectAttr(POBJ pobj)
// {
//     ULONG ulIndex = GDI_HANDLE_GET_INDEX(pobj->hHmgr);
//     return gpentHmgr[ulIndex].pUser;
// }
//
// VOID
// NTAPI
// GDIOBJ_vSetObjectAttr(POBJ pobj, PVOID pvObjAttr)
// {
//     ULONG ulIndex;
//
//     ASSERT(pobj->hHmgr);
//
//     /* Get the handle index */
//     ulIndex = GDI_HANDLE_GET_INDEX(pobj->hHmgr);
//
//     /* Set pointer to the usermode attribute */
//     gpentHmgr[ulIndex].pUser = pvObjAttr;
// }
//
// VOID
// NTAPI
// GDIOBJ_vDeleteObject(POBJ pobj)
// {
//     ULONG ulIndex;
//
//     /* Set the object's delete flag */
//     InterlockedOr16((SHORT*)&pobj->BaseFlags, BASEFLAG_READY_TO_DIE);
//     DBG_LOGEVENT(&pobj->slhLog, EVENT_DELETE, 0);
//
//     /* Get the handle index */
//     ulIndex = GDI_HANDLE_GET_INDEX(pobj->hHmgr);
//     if (ulIndex)
//     {
//         /* Reset the handle valid bit */
//         InterlockedAnd((LONG*)&gpaulRefCount[ulIndex], ~REF_MASK_VALID);
//
//         /* Check if the object is exclusively locked */
//         if (pobj->cExclusiveLock != 0)
//         {
//             /* Reset lock owner and lock count */
//             pobj->dwThreadId = 0;
//             pobj->cExclusiveLock = 0;
//
//             /* Release the pushlock and reenable APCs */
//             ExReleasePushLockExclusive(&pobj->pushlock);
//             KeLeaveCriticalRegion();
//             DECREASE_THREAD_LOCK_COUNT(pobj->hHmgr);
//         }
//     }
//
//     /* Dereference the object (will take care of deletion) */
//     GDIOBJ_vDereferenceObject(pobj);
// }
//
// BOOL
// NTAPI
// GreIsHandleValid(HGDIOBJ hobj)
// {
//     PENTRY pentry;
//
//     pentry = ENTRY_ReferenceEntryByHandle(hobj, 0);
//     if (!pentry) return FALSE;
//     GDIOBJ_vDereferenceObject(pentry->einfo.pobj);
//     return TRUE;
// }
//
// BOOL
// NTAPI
// GreDeleteObject(HGDIOBJ hobj)
// {
//     PENTRY pentry;
//
//     /* Check for stock objects */
//     if (GDI_HANDLE_IS_STOCKOBJ(hobj))
//     {
//         DPRINT1("GreDeleteObject: Cannot delete stock object %p.\n", hobj);
//         return FALSE;
//     }
//
//     /* Reference the handle entry */
//     pentry = ENTRY_ReferenceEntryByHandle(hobj, 0);
//     if (!pentry)
//     {
//         DPRINT1("GreDeleteObject: Trying to delete invalid object %p\n", hobj);
//         return FALSE;
//     }
//
//     /* Check for public owner */
//     if (pentry->ObjectOwner.ulObj == GDI_OBJ_HMGR_PUBLIC)
//     {
//         DPRINT1("GreDeleteObject: Trying to delete global object %p\n", hobj);
//         GDIOBJ_vDereferenceObject(pentry->einfo.pobj);
//         return FALSE;
//     }
//
//     /* Delete the object */
//     GDIOBJ_vDeleteObject(pentry->einfo.pobj);
//     return TRUE;
// }
//
// ULONG
// NTAPI
// GreGetObjectOwner(HGDIOBJ hobj)
// {
//     ULONG ulIndex, ulOwner;
//
//     if (hobj == NULL)
//     {
//         DPRINT("GreGetObjectOwner: invalid NULL handle\n");
//         return GDI_OBJ_HMGR_RESTRICTED;
//     }
//
//     /* Get the handle index */
//     ulIndex = GDI_HANDLE_GET_INDEX(hobj);
//
//     /* Check if the handle is valid */
//     if (ulIndex >= GDI_HANDLE_COUNT ||
//         gpentHmgr[ulIndex].Objt == GDIObjType_DEF_TYPE ||
//         ((ULONG_PTR)hobj >> 16) != gpentHmgr[ulIndex].FullUnique)
//     {
//         DPRINT1("GreGetObjectOwner: invalid handle 0x%p.\n", hobj);
//         return GDI_OBJ_HMGR_RESTRICTED;
//     }
//
//     /* Get the object owner */
//     ulOwner = gpentHmgr[ulIndex].ObjectOwner.ulObj;
//
//     if (ulOwner == HandleToUlong(PsGetCurrentProcessId()))
//         return GDI_OBJ_HMGR_POWNED;
//
//     if (ulOwner == GDI_OBJ_HMGR_PUBLIC)
//         return GDI_OBJ_HMGR_PUBLIC;
//
//     return GDI_OBJ_HMGR_RESTRICTED;
// }
//
// BOOL
// NTAPI
// GreSetObjectOwnerEx(
//     HGDIOBJ hobj,
//     ULONG ulOwner,
//     ULONG Flags)
// {
//     PENTRY pentry;
//
//     /* Check for stock objects */
//     if (GDI_HANDLE_IS_STOCKOBJ(hobj))
//     {
//         DPRINT("GreSetObjectOwner: Got stock object %p\n", hobj);
//         return FALSE;
//     }
//
//     /* Reference the handle entry */
//     pentry = ENTRY_ReferenceEntryByHandle(hobj, Flags);
//     if (!pentry)
//     {
//         DPRINT("GreSetObjectOwner: Invalid handle 0x%p.\n", hobj);
//         return FALSE;
//     }
//
//     /* Call internal function */
//     GDIOBJ_vSetObjectOwner(pentry->einfo.pobj, ulOwner);
//
//     /* Dereference the object */
//     GDIOBJ_vDereferenceObject(pentry->einfo.pobj);
//
//     return TRUE;
// }
//
// BOOL
// NTAPI
// GreSetObjectOwner(
//     HGDIOBJ hobj,
//     ULONG ulOwner)
// {
//     return GreSetObjectOwnerEx(hobj, ulOwner, 0);
// }
//
// INT
// NTAPI
// GreGetObject(
//     IN HGDIOBJ hobj,
//     IN INT cbCount,
//     OUT PVOID pvBuffer)
// {
//     PVOID pvObj;
//     UCHAR objt;
//     INT iResult = 0;
//
//     /* Verify object type */
//     objt = ((ULONG_PTR)hobj >> 16) & 0x1f;
//     if (objt != GDIObjType_BRUSH_TYPE &&
//         objt != GDIObjType_SURF_TYPE &&
//         objt != GDIObjType_LFONT_TYPE &&
//         objt != GDIObjType_PAL_TYPE)
//     {
//         DPRINT1("GreGetObject: Invalid object type\n");
//         return 0;
//     }
//
//     pvObj = GDIOBJ_ReferenceObjectByHandle(hobj, objt);
//     if (!pvObj)
//     {
//         DPRINT("GreGetObject: Could not lock object\n");
//         return 0;
//     }
//
//     switch (GDI_HANDLE_GET_TYPE(hobj))
//     {
//         case GDILoObjType_LO_PEN_TYPE:
//         case GDILoObjType_LO_EXTPEN_TYPE:
//             iResult = PEN_GetObject(pvObj, cbCount, pvBuffer);
//             break;
//
//         case GDILoObjType_LO_BRUSH_TYPE:
//             iResult = BRUSH_GetObject(pvObj, cbCount, pvBuffer);
//             break;
//
//         case GDILoObjType_LO_BITMAP_TYPE:
//             iResult = BITMAP_GetObject(pvObj, cbCount, pvBuffer);
//             break;
//
//         case GDILoObjType_LO_FONT_TYPE:
//             iResult = FontGetObject(pvObj, cbCount, pvBuffer);
//             break;
//
//         case GDILoObjType_LO_PALETTE_TYPE:
//             iResult = PALETTE_GetObject(pvObj, cbCount, pvBuffer);
//             break;
//
//         default:
//             DPRINT1("GDI object type of 0x%p not implemented\n", hobj);
//             break;
//     }
//
//     GDIOBJ_vDereferenceObject(pvObj);
//     return iResult;
// }
//
// W32KAPI
// INT
// APIENTRY
// NtGdiExtGetObjectW(
//     IN HANDLE hobj,
//     IN INT cjBufferSize,
//     OUT LPVOID lpBuffer)
// {
//     UINT iResult, cjMaxSize;
//     union
//     {
//         BITMAP bitmap;
//         DIBSECTION dibsection;
//         LOGPEN logpen;
//         LOGBRUSH logbrush;
//         LOGFONTW logfontw;
//         EXTLOGFONTW extlogfontw;
//         ENUMLOGFONTEXDVW enumlogfontexdvw;
//     } object;
//
//     /* Normalize to the largest supported object size */
//     cjMaxSize = min((UINT)cjBufferSize, sizeof(object));
//
//     /* Now do the actual call */
//     iResult = GreGetObject(hobj, cjMaxSize, lpBuffer ? &object : NULL);
//
//     /* Check if we have a buffer and data */
//     if ((lpBuffer != NULL) && (iResult != 0))
//     {
//         /* Enter SEH for buffer transfer */
//         _SEH2_TRY
//         {
//             /* Probe the buffer and copy it */
//             cjMaxSize = min(cjMaxSize, iResult);
//             ProbeForWrite(lpBuffer, cjMaxSize, sizeof(WORD));
//             RtlCopyMemory(lpBuffer, &object, cjMaxSize);
//         }
//         _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//         {
//             /* Clear the return value.
//              * Do *NOT* set last error here! */
//             iResult = 0;
//         }
//         _SEH2_END;
//     }
//
//     /* Return the count */
//     return iResult;
// }
//
// W32KAPI
// HANDLE
// APIENTRY
// NtGdiCreateClientObj(
//     IN ULONG ulType)
// {
//     POBJ pObject;
//     HANDLE handle;
//
//     /* Check if ulType is valid */
//     if ((ulType != GDILoObjType_LO_METAFILE16_TYPE) &&
//         (ulType != GDILoObjType_LO_METAFILE_TYPE) &&
//         (ulType != GDILoObjType_LO_METADC16_TYPE))
//     {
//         DPRINT1("NtGdiCreateClientObj: Invalid object type 0x%lx.\n", ulType);
//         return NULL;
//     }
//
//     /* Allocate a new object */
//     pObject = GDIOBJ_AllocateObject(GDIObjType_CLIENTOBJ_TYPE,
//                                     sizeof(CLIENTOBJ),
//                                     BASEFLAG_LOOKASIDE);
//     if (!pObject)
//     {
//         DPRINT1("NtGdiCreateClientObj: Could not allocate a clientobj.\n");
//         return NULL;
//     }
//
//     /* Set the real object type */
//     pObject->hHmgr = UlongToHandle(ulType | GDILoObjType_LO_CLIENTOBJ_TYPE);
//
//     /* Create a handle */
//     handle = GDIOBJ_hInsertObject(pObject, GDI_OBJ_HMGR_POWNED);
//     if (!handle)
//     {
//         DPRINT1("NtGdiCreateClientObj: Could not create a handle.\n");
//         GDIOBJ_vFreeObject(pObject);
//         return NULL;
//     }
//
//     /* Unlock it */
//     GDIOBJ_vUnlockObject(pObject);
//
//     return handle;
// }
//
// W32KAPI
// BOOL
// APIENTRY
// NtGdiDeleteClientObj(
//     IN HANDLE hobj)
// {
//     /* We first need to get the real type from the handle */
//     ULONG ulType = GDI_HANDLE_GET_TYPE(hobj);
//
//     /* Check if it's really a CLIENTOBJ */
//     if ((ulType & GDI_HANDLE_BASETYPE_MASK) != GDILoObjType_LO_CLIENTOBJ_TYPE)
//     {
//         /* FIXME: SetLastError? */
//         return FALSE;
//     }
//
//     return GreDeleteObject(hobj);
// }
//
//
//
// PGDI_HANDLE_TABLE GdiHandleTable = NULL;
//
// PGDIOBJ NTAPI
// GDIOBJ_ShareLockObj(HGDIOBJ hObj, DWORD ExpectedType)
// {
//     if (ExpectedType == GDI_OBJECT_TYPE_DONTCARE)
//         ExpectedType = GDI_HANDLE_GET_TYPE(hObj);
//     return GDIOBJ_ReferenceObjectByHandle(hObj, (ExpectedType >> 16) & 0x1f);
// }
//
// // This function is not safe to use with concurrent deleting attempts
// // That shouldn't be a problem, since we don't have any processes yet,
// // that could delete the handle
// BOOL
// NTAPI
// GDIOBJ_ConvertToStockObj(HGDIOBJ *phObj)
// {
//     PENTRY pentry;
//     POBJ pobj;
//
//     /* Reference the handle entry */
//     pentry = ENTRY_ReferenceEntryByHandle(*phObj, 0);
//     if (!pentry)
//     {
//         DPRINT1("GDIOBJ: Requested handle 0x%p is not valid.\n", *phObj);
//         return FALSE;
//     }
//
//     /* Update the entry */
//     pentry->FullUnique |= GDI_ENTRY_STOCK_MASK;
//     pentry->ObjectOwner.ulObj = 0;
//
//     /* Get the pointer to the BASEOBJECT */
//     pobj = pentry->einfo.pobj;
//
//     /* Calculate the new handle */
//     pobj->hHmgr = (HGDIOBJ)((ULONG_PTR)pobj->hHmgr | GDI_HANDLE_STOCK_MASK);
//
//     /* Return the new handle */
//     *phObj = pobj->hHmgr;
//
//     /* Dereference the handle */
//     GDIOBJ_vDereferenceObject(pobj);
//
//     return TRUE;
// }
//
// BOOL
// NTAPI
// GDIOBJ_ConvertFromStockObj(HGDIOBJ *phObj)
// {
//     PENTRY pentry;
//     POBJ pobj;
//
//     /* Reference the handle entry */
//     pentry = ENTRY_ReferenceEntryByHandle(*phObj, 0);
//     if (!pentry)
//     {
//         DPRINT1("GDIOBJ: Requested handle 0x%p is not valid.\n", *phObj);
//         return FALSE;
//     }
//
//     /* Update the entry */
//     pentry->FullUnique &= ~GDI_ENTRY_STOCK_MASK;
//     pentry->ObjectOwner.ulObj = PtrToUlong(PsGetCurrentProcessId());
//
//     /* Get the pointer to the BASEOBJECT */
//     pobj = pentry->einfo.pobj;
//
//     /* Calculate the new handle */
//     pobj->hHmgr = (HGDIOBJ)((ULONG_PTR)pobj->hHmgr & ~GDI_HANDLE_STOCK_MASK);
//
//     /* Return the new handle */
//     *phObj = pobj->hHmgr;
//
//     /* Dereference the handle */
//     GDIOBJ_vDereferenceObject(pobj);
//
//     return TRUE;
// }
//
// POBJ NTAPI
// GDIOBJ_AllocObjWithHandle(ULONG ObjectType, ULONG cjSize)
// {
//     POBJ pobj;
//     FLONG fl = 0;
//     UCHAR objt = (ObjectType >> 16) & 0xFF;
//
//     if ((objt == GDIObjType_DC_TYPE && cjSize == sizeof(DC)) ||
//         (objt == GDIObjType_PAL_TYPE && cjSize == sizeof(PALETTE)) ||
//         (objt == GDIObjType_RGN_TYPE && cjSize == sizeof(REGION)) ||
//         (objt == GDIObjType_SURF_TYPE && cjSize == sizeof(SURFACE)) ||
//         (objt == GDIObjType_PATH_TYPE && cjSize == sizeof(PATH)))
//     {
//         fl |= BASEFLAG_LOOKASIDE;
//     }
//
//     pobj = GDIOBJ_AllocateObject(objt, cjSize, fl);
//     if (!pobj)
//     {
//         return NULL;
//     }
//
//     if (!GDIOBJ_hInsertObject(pobj, GDI_OBJ_HMGR_POWNED))
//     {
//         GDIOBJ_vFreeObject(pobj);
//         return NULL;
//     }
//     return pobj;
// }
//
// PVOID NTAPI
// GDI_MapHandleTable(PEPROCESS pProcess)
// {
//     PVOID pvMappedView = NULL;
//     NTSTATUS Status;
//     LARGE_INTEGER liOffset;
//     SIZE_T cjViewSize = sizeof(GDI_HANDLE_TABLE);
//
//     liOffset.QuadPart = 0;
//
//     ASSERT(gpvGdiHdlTblSection != NULL);
//     ASSERT(pProcess != NULL);
//
//     Status = MmMapViewOfSection(gpvGdiHdlTblSection,
//                                 pProcess,
//                                 &pvMappedView,
//                                 0,
//                                 0,
//                                 &liOffset,
//                                 &cjViewSize,
//                                 ViewUnmap,
//                                 SEC_NO_CHANGE,
//                                 PAGE_READONLY);
//
//     if (!NT_SUCCESS(Status))
//         return NULL;
//
//     return pvMappedView;
// }
// /* --- pretranslation split: the remaining function groups live in gdiobj.split2..2.c and are #included below so this stays ONE translation unit (zero CMake/linkage change). Reassembled verbatim by cpp. --- */
// /* gdiobj.split2.c — #include fragment of gdiobj.c, carved for pretranslation so rustconv gets a small file. NOT a standalone TU; NOT in CMake; do not compile directly. The preprocessor reassembles the original gdiobj.c TU verbatim. See docs/engineering/win32k-pretranslation-audit.md */
//
//
// BOOL NTAPI
// GDI_CleanupForProcess(struct _EPROCESS *Process)
// {
//     PENTRY pentry;
//     ULONG ulIndex;
//     DWORD dwProcessId;
//     PPROCESSINFO ppi;
//
//     DPRINT("CleanupForProcess prochandle %p Pid %p\n",
//            Process, Process->UniqueProcessId);
//
//     ASSERT(Process == PsGetCurrentProcess());
//
//     /* Get the current process Id */
//     dwProcessId = PtrToUlong(PsGetCurrentProcessId());
//
//     /* Loop all handles in the handle table */
//     for (ulIndex = RESERVE_ENTRIES_COUNT; ulIndex < gulFirstUnused; ulIndex++)
//     {
//         pentry = &gpentHmgr[ulIndex];
//
//         /* Check if the object is owned by the process */
//         if (pentry->ObjectOwner.ulObj == dwProcessId)
//         {
//             ASSERT(pentry->einfo.pobj->cExclusiveLock == 0);
//
//             /* Reference the object and delete it */
//             InterlockedIncrement((LONG*)&gpaulRefCount[ulIndex]);
//             GDIOBJ_vDeleteObject(pentry->einfo.pobj);
//         }
//     }
//
// #if DBG
//     DbgGdiHTIntegrityCheck();
// #endif
//
//     ppi = PsGetCurrentProcessWin32Process();
//     DPRINT("Completed cleanup for process %p\n", Process->UniqueProcessId);
//     if (ppi->GDIHandleCount != 0)
//     {
//         DPRINT1("Leaking %d handles!\n", ppi->GDIHandleCount);
//         ASSERT(FALSE);
//     }
//
//     /* Loop all handles in the handle table */
//     for (ulIndex = RESERVE_ENTRIES_COUNT; ulIndex < gulFirstUnused; ulIndex++)
//     {
//         pentry = &gpentHmgr[ulIndex];
//
//         /* Check if the object is owned by the process */
//         if (pentry->ObjectOwner.ulObj == dwProcessId)
//         {
//             DPRINT1("Leaking object. Index=%lx, type=0x%x, refcount=%lx\n",
//                     ulIndex, pentry->Objt, gpaulRefCount[ulIndex]);
//             DBG_DUMP_EVENT_LIST(&pentry->einfo.pobj->slhLog);
//             //DBG_CLEANUP_EVENT_LIST(&pentry->einfo.pobj->slhLog);
//             ASSERT(FALSE);
//         }
//     }
//
//     return TRUE;
// }
//
// /// HACK!
// PGDI_POOL
// GetBrushAttrPool(VOID)
// {
//     PPROCESSINFO ppi;
//
//     ppi = PsGetCurrentProcessWin32Process();
//     NT_ASSERT(ppi != NULL);
//
//     return ppi->pPoolBrushAttr;
// }
//
// /* EOF */
//
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
