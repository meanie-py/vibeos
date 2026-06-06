//! lock.c — the SCM's user-side start-serialization lock. Faithful translation
//! of the 4 cdecl helpers (ScmAcquire/Release/QueryServiceLockStatus{W,A}).
//! `goto done` cascades render as `loop { …; break }` onto the shared
//! ScmUnlockDatabase epilogue. Output-pointer writes use `write_volatile`
//! (the RPC marshaller reads them; prevents captures(none) elision).

use crate::types::*;
use core::mem::size_of;
use core::ptr::{addr_of_mut, null_mut, write_volatile};
use winsrv_sys::winsrv::*;

/// The unique user service start lock of the SCM.
static mut pServiceStartLock: PSTART_LOCK = null_mut();

/// NOTE: `IsServiceController` is TRUE if locked by the SCM, FALSE otherwise.
#[no_mangle]
pub unsafe extern "C" fn ScmAcquireServiceStartLock(
    IsServiceController: BOOL,
    lpLock: LPSC_RPC_LOCK,
) -> DWORD {
    let _ = IsServiceController; // only used by the (FIXME) owner-SID sizing
    let dwRequiredSize: DWORD;
    let mut dwError: DWORD = ERROR_SUCCESS;

    write_volatile(lpLock, null_mut()); // *lpLock = NULL

    /* Lock the service database exclusively */
    ScmLockDatabaseExclusive();

    loop {
        if !pServiceStartLock.is_null() {
            dwError = ERROR_SERVICE_DATABASE_LOCKED;
            break;
        }

        /* Allocate a new lock for the database */
        dwRequiredSize = size_of::<START_LOCK>() as DWORD;
        /* if !IsServiceController { dwRequiredSize += RtlLengthSid(UserSid) } — FIXME */

        pServiceStartLock = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, dwRequiredSize as _)
            as PSTART_LOCK;
        if pServiceStartLock.is_null() {
            dwError = ERROR_NOT_ENOUGH_MEMORY;
            break;
        }

        (*pServiceStartLock).Tag = LOCK_TAG;
        (*pServiceStartLock).TimeWhenLocked = time(null_mut()) as u32;
        /* FIXME: Retrieve the owner SID. Use IsServiceController. */
        (*pServiceStartLock).LockOwnerSid = null_mut();

        write_volatile(lpLock, pServiceStartLock as SC_RPC_LOCK); // *lpLock = lock
        break;
    }

    /* Unlock the service database */
    ScmUnlockDatabase();

    dwError
}

#[no_mangle]
pub unsafe extern "C" fn ScmReleaseServiceStartLock(lpLock: LPSC_RPC_LOCK) -> DWORD {
    let mut dwError: DWORD = ERROR_SUCCESS;

    if lpLock.is_null() {
        return ERROR_INVALID_SERVICE_LOCK;
    }

    let pStartLock = *lpLock as PSTART_LOCK;

    if (*pStartLock).Tag != LOCK_TAG {
        return ERROR_INVALID_SERVICE_LOCK;
    }

    /* Lock the service database exclusively */
    ScmLockDatabaseExclusive();

    /* Release the lock handle */
    if pStartLock == pServiceStartLock && !pServiceStartLock.is_null() {
        HeapFree(GetProcessHeap(), 0, pServiceStartLock as LPVOID);
        pServiceStartLock = null_mut();
        write_volatile(lpLock, null_mut()); // *lpLock = NULL
        dwError = ERROR_SUCCESS;
    } else {
        dwError = ERROR_INVALID_SERVICE_LOCK;
    }

    /* Unlock the service database */
    ScmUnlockDatabase();

    dwError
}

/// Helper for RQueryServiceLockStatusW(). Caller guarantees `lpLockStatus`
/// points at a valid, well-sized buffer.
#[no_mangle]
pub unsafe extern "C" fn ScmQueryServiceLockStatusW(lpLockStatus: LPQUERY_SERVICE_LOCK_STATUSW) {
    /* Lock the service database shared */
    ScmLockDatabaseShared();

    if !pServiceStartLock.is_null() {
        write_volatile(addr_of_mut!((*lpLockStatus).fIsLocked), 1); // TRUE
        /* FIXME: Retrieve the owner name. */
        write_volatile(addr_of_mut!((*lpLockStatus).lpLockOwner), null_mut());
        write_volatile(
            addr_of_mut!((*lpLockStatus).dwLockDuration),
            (time(null_mut()) as u32).wrapping_sub((*pServiceStartLock).TimeWhenLocked),
        );
    } else {
        write_volatile(addr_of_mut!((*lpLockStatus).fIsLocked), 0); // FALSE
        /* wcscpy((LPWSTR)(lpLockStatus + 1), L"") — empty wide string just past the struct */
        let after = (lpLockStatus as *mut u8).add(size_of::<QUERY_SERVICE_LOCK_STATUSW>()) as *mut u16;
        *after = 0;
        write_volatile(
            addr_of_mut!((*lpLockStatus).lpLockOwner),
            size_of::<QUERY_SERVICE_LOCK_STATUSW>() as LPWSTR, // RPC self-relative offset
        );
        write_volatile(addr_of_mut!((*lpLockStatus).dwLockDuration), 0);
    }

    /* Unlock the service database */
    ScmUnlockDatabase();
}

/// Helper for RQueryServiceLockStatusA(). Caller guarantees `lpLockStatus`
/// points at a valid, well-sized buffer.
#[no_mangle]
pub unsafe extern "C" fn ScmQueryServiceLockStatusA(lpLockStatus: LPQUERY_SERVICE_LOCK_STATUSA) {
    /* Lock the service database shared */
    ScmLockDatabaseShared();

    if !pServiceStartLock.is_null() {
        write_volatile(addr_of_mut!((*lpLockStatus).fIsLocked), 1); // TRUE
        /* FIXME: Retrieve the owner name. */
        write_volatile(addr_of_mut!((*lpLockStatus).lpLockOwner), null_mut());
        write_volatile(
            addr_of_mut!((*lpLockStatus).dwLockDuration),
            (time(null_mut()) as u32).wrapping_sub((*pServiceStartLock).TimeWhenLocked),
        );
    } else {
        write_volatile(addr_of_mut!((*lpLockStatus).fIsLocked), 0); // FALSE
        /* strcpy((LPSTR)(lpLockStatus + 1), "") — empty ANSI string just past the struct */
        let after = (lpLockStatus as *mut u8).add(size_of::<QUERY_SERVICE_LOCK_STATUSA>());
        *after = 0;
        write_volatile(
            addr_of_mut!((*lpLockStatus).lpLockOwner),
            size_of::<QUERY_SERVICE_LOCK_STATUSA>() as LPSTR, // RPC self-relative offset
        );
        write_volatile(addr_of_mut!((*lpLockStatus).dwLockDuration), 0);
    }

    /* Unlock the service database */
    ScmUnlockDatabase();
}
