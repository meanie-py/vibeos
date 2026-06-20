//! win32ss/reactx/dxg/ddhmg.c — Dd handle-manager.

use win32k_sys::win32k::*;
use ntkrnl_sys::ddk::PsGetCurrentProcessId;
use ntkrnl_sys::interlocked::InterlockedIncrement;
// Exported ntoskrnl KeGetCurrentThread (real symbol on all 3 arches), not the
// mm.rs inline helper — that inlines via KeGetCurrentPrcb (a C FORCEINLINE with
// no exported symbol) on arm64, undefined at dxg.sys link.
use ntkrnl_sys::ddk::PsGetCurrentThread;
use crate::types::*;
use crate::tags::*;
use crate::driver_main::gpLockShortDelay;

// ---- file-scope globals owned by ddhmg.c (dxg_int.h:175-180) ----
pub static mut gcSizeDdHmgr: ULONG = 1024;
pub static mut gpentDdHmgr: PDD_ENTRY = core::ptr::null_mut();
pub static mut gcMaxDdHmgr: ULONG = 0;
pub static mut gpentDdHmgrLast: PDD_ENTRY = core::ptr::null_mut();
pub static mut ghFreeDdHmgr: ULONG = 0;
pub static mut ghsemHmgr: HSEMAPHORE = core::ptr::null_mut();

pub unsafe extern "C" fn VerifyObjectOwner(pEntry: PDD_ENTRY) -> BOOL {
    let Pid = (PsGetCurrentProcessId() as DWORD_PTR as DWORD) & 0xFFFFFFFC;
    let check = ((*pEntry).Pid as DWORD_PTR as DWORD) & 0xFFFFFFFE;
    if (check == Pid) || (check == 0) { 1 } else { 0 }
}

#[no_mangle]
pub unsafe extern "C" fn DdHmgCreate() -> BOOL {
    gpentDdHmgr = EngAllocMem(
        FL_ZERO_MEMORY,
        (gcSizeDdHmgr as usize * core::mem::size_of::<DD_ENTRY>()) as ULONG,
        TAG_THDD,
    ) as PDD_ENTRY;
    ghFreeDdHmgr = 0;
    gcMaxDdHmgr = 1;

    if !gpentDdHmgr.is_null() {
        ghsemHmgr = EngCreateSemaphore();

        if !ghsemHmgr.is_null() {
            gpLockShortDelay = EngAllocMem(
                FL_ZERO_MEMORY | FL_NONPAGED_MEMORY,
                core::mem::size_of::<LARGE_INTEGER>() as ULONG,
                TAG_GINI,
            ) as PLARGE_INTEGER;

            if !gpLockShortDelay.is_null() {
                (*gpLockShortDelay).__bindgen_anon_1.HighPart = -1;
                return 1;
            }

            EngDeleteSemaphore(ghsemHmgr);
            ghsemHmgr = core::ptr::null_mut();
        }

        EngFreeMem(gpentDdHmgr as PVOID);
        gpentDdHmgr = core::ptr::null_mut();
    }

    0
}

#[no_mangle]
pub unsafe extern "C" fn DdHmgDestroy() -> BOOL {
    gcMaxDdHmgr = 0;
    gcSizeDdHmgr = 0;
    ghFreeDdHmgr = 0;
    gpentDdHmgrLast = core::ptr::null_mut();

    if !gpentDdHmgr.is_null() {
        EngFreeMem(gpentDdHmgr as PVOID);
        gpentDdHmgr = core::ptr::null_mut();
    }

    if !ghsemHmgr.is_null() {
        EngDeleteSemaphore(ghsemHmgr);
        ghsemHmgr = core::ptr::null_mut();
    }

    1
}

pub unsafe extern "C" fn DdHmgLock(
    DdHandle: HANDLE,
    ObjectType: UCHAR,
    LockOwned: BOOLEAN,
) -> PVOID {
    let Index: DWORD = DDHMG_HTOI(DdHandle) as DWORD;

    let mut pEntry: PDD_ENTRY;
    let mut Object: PVOID = core::ptr::null_mut();

    if LockOwned == 0 {
        EngAcquireSemaphore(ghsemHmgr);
    }

    if Index < gcMaxDdHmgr {
        pEntry = (gpentDdHmgr as PBYTE)
            .add(core::mem::size_of::<DD_ENTRY>() * Index as usize)
            as PDD_ENTRY;

        if VerifyObjectOwner(pEntry) != 0 {
            if (*pEntry).Objt == ObjectType
                && (*pEntry).FullUnique == (((DdHandle as ULONG_PTR) >> 21) & 0x7FF) as USHORT
                && (*(*pEntry).u.pobj).cExclusiveLock == 0
            {
                InterlockedIncrement(
                    core::ptr::addr_of_mut!((*(*pEntry).u.pobj).cExclusiveLock) as *mut LONG,
                );
                (*(*pEntry).u.pobj).Tid = PsGetCurrentThread() as PVOID;
                Object = (*pEntry).u.pobj as PVOID;
            }
        }
    }

    if LockOwned == 0 {
        EngReleaseSemaphore(ghsemHmgr);
    }

    Object
}

pub unsafe extern "C" fn DdAllocateObject(
    objSize: ULONG,
    objType: UCHAR,
    objZeroMemory: BOOLEAN,
) -> PVOID {
    let pObject: PVOID = if objZeroMemory != 0 {
        EngAllocMem(FL_ZERO_MEMORY, objSize, ((objType as ULONG) << 24) + TAG_DH_0)
    } else {
        EngAllocMem(0, objSize, ((objType as ULONG) << 24) + TAG_DH_0)
    };

    if pObject.is_null() {
        EngSetLastError(ERROR_NOT_ENOUGH_MEMORY);
    }

    pObject
}

pub unsafe extern "C" fn DdFreeObject(pObject: PVOID) {
    EngFreeMem(pObject);
}

pub unsafe extern "C" fn DdGetFreeHandle(objType: UCHAR) -> HANDLE {
    let mut mAllocMem: PVOID;
    let mut mAllocEntries: ULONG;
    let mut pEntry: PDD_ENTRY;
    let retVal: ULONG_PTR;
    let index: ULONG;

    if gpentDdHmgr.is_null() {
        return core::ptr::null_mut();
    }

    if gcMaxDdHmgr == DDHMG_HANDLE_LIMIT as ULONG {
        return core::ptr::null_mut();
    }

    if ghFreeDdHmgr != 0 {
        index = ghFreeDdHmgr;
        pEntry = (gpentDdHmgr as PBYTE)
            .add(core::mem::size_of::<DD_ENTRY>() * index as usize)
            as PDD_ENTRY;

        ghFreeDdHmgr = (*pEntry).u.NextFree;

        (*pEntry).FullUnique = (objType | 8) as USHORT;
        retVal = (((*pEntry).FullUnique as ULONG_PTR) << 21) | index as ULONG_PTR;
        return retVal as HANDLE;
    }

    if gcSizeDdHmgr == gcMaxDdHmgr {
        mAllocEntries = gcSizeDdHmgr + 1024;
        mAllocMem = EngAllocMem(
            FL_ZERO_MEMORY,
            (core::mem::size_of::<DD_ENTRY>() * mAllocEntries as usize) as ULONG,
            TAG_THDD,
        );
        if mAllocMem.is_null() {
            return core::ptr::null_mut();
        }

        core::ptr::copy_nonoverlapping(
            gpentDdHmgr as *const u8,
            mAllocMem as *mut u8,
            core::mem::size_of::<DD_ENTRY>() * gcSizeDdHmgr as usize,
        );
        gcSizeDdHmgr = mAllocEntries;
        gpentDdHmgrLast = gpentDdHmgr;
        EngFreeMem(gpentDdHmgr as PVOID);
        gpentDdHmgr = mAllocMem as PDD_ENTRY;
    }

    pEntry = (gpentDdHmgr as PBYTE)
        .add(core::mem::size_of::<DD_ENTRY>() * gcMaxDdHmgr as usize)
        as PDD_ENTRY;

    (*pEntry).FullUnique = (objType | 8) as USHORT;
    retVal = (((*pEntry).FullUnique as ULONG_PTR) << 21) | gcMaxDdHmgr as ULONG_PTR;
    gcMaxDdHmgr += 1;

    retVal as HANDLE
}

#[no_mangle]
pub unsafe extern "C" fn DdHmgAlloc(
    objSize: ULONG,
    objType: CHAR,
    objLock: BOOLEAN,
) -> HANDLE {
    let pObject: PVOID;
    let DdHandle: HANDLE;
    let mut pEntry: PDD_ENTRY;
    let Index: DWORD;

    pObject = DdAllocateObject(objSize, objType as UCHAR, 1);
    if pObject.is_null() {
        return core::ptr::null_mut();
    }

    EngAcquireSemaphore(ghsemHmgr);

    DdHandle = DdGetFreeHandle(objType as UCHAR);

    if !DdHandle.is_null() {
        Index = DDHMG_HTOI(DdHandle) as DWORD;

        pEntry = (gpentDdHmgr as PBYTE)
            .add(core::mem::size_of::<DD_ENTRY>() * Index as usize)
            as PDD_ENTRY;

        (*pEntry).u.pobj = pObject as PDD_BASEOBJECT;
        (*pEntry).Objt = objType as UCHAR;

        (*pEntry).Pid = (((PsGetCurrentProcessId() as ULONG_PTR) & 0xFFFFFFFC)
            | ((*pEntry).Pid as ULONG_PTR & 1)) as HANDLE;

        if objLock != 0 {
            InterlockedIncrement(
                core::ptr::addr_of_mut!((*(*pEntry).u.pobj).cExclusiveLock) as *mut LONG,
            );
            (*(*pEntry).u.pobj).Tid = PsGetCurrentThread() as PVOID;
        }
        (*(*pEntry).u.pobj).hHmgr = DdHandle;

        EngReleaseSemaphore(ghsemHmgr);

        if objLock == 0 {
            return DdHandle;
        }

        return pEntry as HANDLE;
    }

    EngReleaseSemaphore(ghsemHmgr);
    DdFreeObject(pObject);
    core::ptr::null_mut()
}

pub unsafe extern "C" fn DdHmgFree(DdHandle: HANDLE) {
    let pEntry: PDD_ENTRY;

    let Index: DWORD = DDHMG_HTOI(DdHandle) as DWORD;

    EngAcquireSemaphore(ghsemHmgr);

    pEntry = (gpentDdHmgr as PBYTE)
        .add(core::mem::size_of::<DD_ENTRY>() * Index as usize)
        as PDD_ENTRY;

    if !(*pEntry).u.pobj.is_null() {
        DdFreeObject((*pEntry).u.pobj as PVOID);
    }

    (*pEntry).u.NextFree = ghFreeDdHmgr;

    (*pEntry).Pid = ((*pEntry).Pid as DWORD_PTR & 1) as HANDLE;
    ghFreeDdHmgr = Index;

    EngReleaseSemaphore(ghsemHmgr);
}
