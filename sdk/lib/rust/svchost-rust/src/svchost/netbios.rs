use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[no_mangle]
pub static mut GlobalNetBiosUseCount: LONG = 0;

#[no_mangle]
pub static mut LanaFlags: [DWORD; 8] = [0; 8];

#[no_mangle]
pub static mut SvcNetBiosCritSec: CRITICAL_SECTION = _RTL_CRITICAL_SECTION {
    DebugInfo: ::core::ptr::null::<_RTL_CRITICAL_SECTION_DEBUG>()
        as *mut _RTL_CRITICAL_SECTION_DEBUG,
    LockCount: 0,
    RecursionCount: 0,
    OwningThread: ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void,
    LockSemaphore: ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void,
    SpinCount: 0,
};
#[no_mangle]
pub unsafe extern "system" fn SvcNetBiosStatusToApiStatus(NetBiosError: DWORD) -> DWORD {
    /* Convert from one status to another */
    match NetBiosError {
        NRC_GOODRET => return NERR_Success as DWORD,
        NRC_INUSE | NRC_NAMCONF => return NERR_DuplicateName as DWORD,
        NRC_NOWILD | NRC_NAMERR => return ERROR_INVALID_PARAMETER as DWORD,
        NRC_NOCALL => return NERR_NameNotFound as DWORD,
        NRC_NORES => return NERR_NoNetworkResource as DWORD,
        NRC_DUPNAME => return NERR_AlreadyExists as DWORD,
        NRC_NAMTFUL => return NERR_TooManyNames as DWORD,
        NRC_ACTSES => return NERR_DeleteLater as DWORD,
        NRC_REMTFUL => return ERROR_REM_NOT_LIST as DWORD,
        _ => return NERR_NetworkError as DWORD,
    }
}
#[no_mangle]
pub unsafe extern "system" fn LanaFlagIsSet(Lana: UCHAR) -> BOOL {
    let mut i: DWORD = (Lana as DWORD) / 32;

    /* Clear the flag for this LANA */
    return (if i <= 7 {
        LanaFlags[i as usize] & (1 << ((Lana as DWORD) - 32 * i))
    } else {
        FALSE
    }) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn SetLanaFlag(Lana: UCHAR) {
    let mut i: DWORD = Lana as DWORD / 32;

    /* Set the flag for this LANA */
    if i <= 7 {
        LanaFlags[i as usize] |= (1 as DWORD)
            << (Lana as DWORD).wrapping_sub((32 as DWORD).wrapping_mul(i));
    }
}

#[no_mangle]
pub unsafe extern "system" fn SvcNetBiosInit() {
    /* Initialize NetBIOS-related structures and variables */
    InitializeCriticalSection(core::ptr::addr_of_mut!(SvcNetBiosCritSec));
    GlobalNetBiosUseCount = 0;
    ZeroMemory!(
        core::ptr::addr_of_mut!(LanaFlags) as *mut core::ffi::c_void,
        core::mem::size_of::<[DWORD; 8]>(),
    );
}

#[no_mangle]
pub unsafe extern "system" fn SvcNetBiosClose() {
    /* While holding the lock, drop a reference*/
    EnterCriticalSection(core::ptr::addr_of_mut!(SvcNetBiosCritSec));
    if (GlobalNetBiosUseCount != 0) && ({
        GlobalNetBiosUseCount -= 1;
        GlobalNetBiosUseCount == 0
    }) {
        /* All references are gone, clear all LANA's */
        ZeroMemory!(
            core::ptr::addr_of_mut!(LanaFlags) as *mut core::ffi::c_void,
            core::mem::size_of::<[DWORD; 8]>(),
        );
    }
    LeaveCriticalSection(core::ptr::addr_of_mut!(SvcNetBiosCritSec));
}
#[no_mangle]
pub unsafe extern "system" fn SvcNetBiosOpen() {
    EnterCriticalSection(core::ptr::addr_of_mut!(SvcNetBiosCritSec));
    GlobalNetBiosUseCount += 1;
    LeaveCriticalSection(core::ptr::addr_of_mut!(SvcNetBiosCritSec));
}

#[no_mangle]
pub unsafe extern "system" fn SvcNetBiosReset(LanaNum: UCHAR) -> DWORD {
    let mut dwError: DWORD = ERROR_SUCCESS as DWORD;
    let mut nbtError: UCHAR;
    let mut ncb: NCB = core::mem::zeroed();

    /* Block all other NetBIOS operations */
    EnterCriticalSection(core::ptr::addr_of_mut!(SvcNetBiosCritSec));

    /* Is this LANA enabled? */
    if !(LanaFlagIsSet(LanaNum) != 0) {
        /* Yep, build a reset packet */
        ZeroMemory!(
            core::ptr::addr_of_mut!(ncb) as *mut core::ffi::c_void,
            core::mem::size_of::<NCB>(),
        );
        ncb.ncb_lsn = 0 as UCHAR;
        ncb.ncb_command = NCBRESET as UCHAR;
        ncb.ncb_callname[0 as usize] = 0xFE as UCHAR; // Max Sessions
        ncb.ncb_callname[1 as usize] = 0 as UCHAR;
        ncb.ncb_callname[2 as usize] = 0xFD as UCHAR; // Max Names
        ncb.ncb_callname[3 as usize] = 0 as UCHAR;
        ncb.ncb_lana_num = LanaNum;

        /* Send it */
        nbtError = Netbios(core::ptr::addr_of_mut!(ncb));

        /* Convert the status to Win32 format */
        dwError = SvcNetBiosStatusToApiStatus(nbtError as DWORD);

        /* Enable the LANA if the reset worked */
        if dwError == ERROR_SUCCESS as DWORD {
            SetLanaFlag(LanaNum);
        }
    }

    /* Drop the lock and return */
    LeaveCriticalSection(core::ptr::addr_of_mut!(SvcNetBiosCritSec));
    return dwError;
}
