//! win32ss/gdi/gdi32/objects/clientobj.c → Rust. Support functions for GDI client objects.
//!
//! Owns the global GDI client-object link hash table + its critical section and counter.
//
// NEEDS-IMPORT: HeapAlloc / HeapFree / GetProcessHeap (kernel32 WINAPI),
//   EnterCriticalSection / LeaveCriticalSection (kernel32 WINAPI, take LPCRITICAL_SECTION
//   == PRTL_CRITICAL_SECTION). Declared extern "system" below.
// NEEDS-TYPE: CRITICAL_SECTION — the win32k-sys surface has no CRITICAL_SECTION alias; it
//   is identical to RTL_CRITICAL_SECTION (winbase typedef). gcsClientObjLinks is typed
//   RTL_CRITICAL_SECTION here, matching dllmain.rs's extern declaration of it.

use win32k_sys::win32k::*;

// win32k_sys's FALSE/TRUE are bindgen u32 (#define), but BOOL is i32 — shadow with
// BOOL-typed locals so `return FALSE`/`return TRUE` type-check against the i32 BOOL returns.
const FALSE: BOOL = 0;
const TRUE: BOOL = 1;

// ── Cross-file functions ──
//    kernel32   LPVOID WINAPI HeapAlloc(HANDLE, DWORD, SIZE_T)             → extern "system"
//    kernel32   BOOL   WINAPI HeapFree(HANDLE, DWORD, LPVOID)             → extern "system"
//    kernel32   HANDLE WINAPI GetProcessHeap(VOID)                        → extern "system"
//    kernel32   VOID   WINAPI EnterCriticalSection(LPCRITICAL_SECTION)    → extern "system"
//    kernel32   VOID   WINAPI LeaveCriticalSection(LPCRITICAL_SECTION)    → extern "system"
extern "system" {
    fn HeapAlloc(hHeap: HANDLE, dwFlags: DWORD, dwBytes: SIZE_T) -> LPVOID;
    fn HeapFree(hHeap: HANDLE, dwFlags: DWORD, lpMem: LPVOID) -> BOOL;
    fn GetProcessHeap() -> HANDLE;
    fn EnterCriticalSection(lpCriticalSection: PRTL_CRITICAL_SECTION);
    fn LeaveCriticalSection(lpCriticalSection: PRTL_CRITICAL_SECTION);
}

// ── Globals DEFINED in this file (clientobj.c:10-11, 20) ──

/// `CRITICAL_SECTION gcsClientObjLinks;` — guards the client-object link hash table.
/// dllmain.rs externs and Initialize/Delete-es this; this file owns the storage.
#[no_mangle]
pub static mut gcsClientObjLinks: RTL_CRITICAL_SECTION = RTL_CRITICAL_SECTION {
    DebugInfo: core::ptr::null_mut(),
    LockCount: 0,
    RecursionCount: 0,
    OwningThread: core::ptr::null_mut(),
    LockSemaphore: core::ptr::null_mut(),
    SpinCount: 0,
};

/// `ULONG gcClientObj;` — count of live client-object links (gdiobj.rs externs this).
#[no_mangle]
pub static mut gcClientObj: ULONG = 0;

#[repr(C)]
#[derive(Copy, Clone)]
struct CLIENTOBJLINK {
    pcolNext: *mut CLIENTOBJLINK,
    hobj: HGDIOBJ,
    pvObj: PVOID,
}
type PCLIENTOBJLINK = *mut CLIENTOBJLINK;

/// `PCLIENTOBJLINK gapcolHashTable[127];` — file-scope hash table (no other file refs it).
const HASH_TABLE_SIZE: usize = 127;
static mut gapcolHashTable: [PCLIENTOBJLINK; HASH_TABLE_SIZE] =
    [core::ptr::null_mut(); HASH_TABLE_SIZE];

#[no_mangle]
pub unsafe extern "system" fn GdiCreateClientObjLink(
    hobj: HGDIOBJ,
    pvObject: PVOID,
) -> BOOL {
    let mut pcol: PCLIENTOBJLINK;
    let iHashIndex: ULONG;

    /* Allocate a link structure */
    pcol = HeapAlloc(
        GetProcessHeap(),
        0,
        core::mem::size_of::<CLIENTOBJLINK>() as SIZE_T,
    ) as PCLIENTOBJLINK;
    if pcol.is_null() {
        return FALSE;
    }

    /* Setup the link structure */
    (*pcol).hobj = hobj;
    (*pcol).pvObj = pvObject;

    /* Calculate the hash index */
    iHashIndex = ((hobj as usize) % HASH_TABLE_SIZE) as ULONG;

    /* Enter the critical section */
    EnterCriticalSection(core::ptr::addr_of_mut!(gcsClientObjLinks));

    /* Insert the link structure */
    (*pcol).pcolNext = gapcolHashTable[iHashIndex as usize];
    gapcolHashTable[iHashIndex as usize] = pcol;
    gcClientObj += 1;

    /* Leave the critical section */
    LeaveCriticalSection(core::ptr::addr_of_mut!(gcsClientObjLinks));

    TRUE
}

#[no_mangle]
pub unsafe extern "system" fn GdiGetClientObjLink(hobj: HGDIOBJ) -> PVOID {
    let iHashIndex: ULONG;
    let mut pcol: PCLIENTOBJLINK;
    let mut pvObject: PVOID = core::ptr::null_mut();

    /* Calculate the hash index */
    iHashIndex = ((hobj as usize) % HASH_TABLE_SIZE) as ULONG;

    /* Enter the critical section */
    EnterCriticalSection(core::ptr::addr_of_mut!(gcsClientObjLinks));

    /* Loop the link entries in this hash bucket */
    pcol = gapcolHashTable[iHashIndex as usize];
    while !pcol.is_null() {
        /* Check if this is the object we want */
        if (*pcol).hobj == hobj {
            /* Get the object pointer and bail out */
            pvObject = (*pcol).pvObj;
            break;
        }

        /* Go to the next entry */
        pcol = (*pcol).pcolNext;
    }

    /* Leave the critical section */
    LeaveCriticalSection(core::ptr::addr_of_mut!(gcsClientObjLinks));

    pvObject
}

#[no_mangle]
pub unsafe extern "system" fn GdiRemoveClientObjLink(hobj: HGDIOBJ) -> PVOID {
    let mut pcol: PCLIENTOBJLINK;
    let mut ppcol: *mut PCLIENTOBJLINK;
    let iHashIndex: ULONG;
    let mut pvObject: PVOID = core::ptr::null_mut();

    /* Calculate the hash index */
    iHashIndex = ((hobj as usize) % HASH_TABLE_SIZE) as ULONG;

    /* Enter the critical section */
    EnterCriticalSection(core::ptr::addr_of_mut!(gcsClientObjLinks));

    /* Loop the link entries in this hash bucket */
    ppcol = core::ptr::addr_of_mut!(gapcolHashTable[iHashIndex as usize]);
    while !(*ppcol).is_null() {
        /* Get the current client object link */
        pcol = *ppcol;

        /* Check if this is the one we want */
        if (*pcol).hobj == hobj {
            /* Update the link pointer, removing this link */
            *ppcol = (*pcol).pcolNext;
            gcClientObj -= 1;

            /* Get the object pointer */
            pvObject = (*pcol).pvObj;

            /* Free the link structure */
            HeapFree(GetProcessHeap(), 0, pcol as LPVOID);

            /* We're done */
            break;
        }

        /* Go to the next link pointer */
        ppcol = core::ptr::addr_of_mut!((*pcol).pcolNext);
    }

    /* Leave the critical section */
    LeaveCriticalSection(core::ptr::addr_of_mut!(gcsClientObjLinks));

    /* Return the object pointer, or NULL if we did not find it */
    pvObject
}

#[no_mangle]
pub unsafe extern "system" fn GdiCreateClientObj(
    pvObject: PVOID,
    eObjType: GDILOOBJTYPE,
) -> HGDIOBJ {
    let hobj: HGDIOBJ;

    /* Call win32k to create a client object handle */
    hobj = NtGdiCreateClientObj(eObjType as ULONG);
    if hobj.is_null() {
        return core::ptr::null_mut();
    }

    /* Create the client object link */
    if GdiCreateClientObjLink(hobj, pvObject) == FALSE {
        NtGdiDeleteClientObj(hobj);
        return core::ptr::null_mut();
    }

    hobj
}

#[no_mangle]
pub unsafe extern "system" fn GdiDeleteClientObj(hobj: HGDIOBJ) -> PVOID {
    let pvObject: PVOID;

    /* Remove the client object link */
    pvObject = GdiRemoveClientObjLink(hobj);
    if pvObject.is_null() {
        return core::ptr::null_mut();
    }

    /* Call win32k to delete the handle */
    if NtGdiDeleteClientObj(hobj) == FALSE {
        // ASSERT(FALSE);
    }

    pvObject
}
