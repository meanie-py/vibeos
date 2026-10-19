use crate::twain_32::dsm_ctrl::TWAIN_CloseDS;
use crate::twain_32::dsm_ctrl::TWAIN_CloseDSM;
use crate::twain_32::dsm_ctrl::TWAIN_ControlNull;
use crate::twain_32::dsm_ctrl::TWAIN_GetDSMStatus;
use crate::twain_32::dsm_ctrl::TWAIN_IdentityGetDefault;
use crate::twain_32::dsm_ctrl::TWAIN_IdentityGetFirst;
use crate::twain_32::dsm_ctrl::TWAIN_IdentityGetNext;
use crate::twain_32::dsm_ctrl::TWAIN_OpenDS;
use crate::twain_32::dsm_ctrl::TWAIN_OpenDSM;
use crate::twain_32::dsm_ctrl::TWAIN_ProcessEvent;
use crate::twain_32::dsm_ctrl::TWAIN_UserSelect;
use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[no_mangle]
pub static mut DSM_twCC: TW_UINT16 = 0;

#[no_mangle]
pub static mut activeSources: *mut activeDS = ::core::ptr::null::<activeDS>() as *mut activeDS;

#[no_mangle]
pub static mut DSM_hinstance: HINSTANCE = ::core::ptr::null::<HINSTANCE__>() as *mut HINSTANCE__;

// WINE_DEFAULT_DEBUG_CHANNEL(twain): debug channel name for this translation unit.

#[no_mangle]
pub unsafe extern "system" fn DllMain(hinstance: HINSTANCE, reason: DWORD, reserved: LPVOID) -> BOOL {
    match reason {
        DLL_PROCESS_ATTACH => {
            DisableThreadLibraryCalls(hinstance as HMODULE);
            DSM_hinstance = hinstance;
        }
        DLL_PROCESS_DETACH => {}
        _ => {}
    }
    return (TRUE) as BOOL;
}

unsafe extern "C" fn TWAIN_LookupSource(pDest: *const TW_IDENTITY) -> *mut activeDS {
    let mut pSource: *mut activeDS = core::ptr::null_mut();
    pSource = activeSources;
    while !pSource.is_null() {
        if (*pSource).identity.Id == (*pDest).Id {
            break;
        }
        pSource = (*pSource).next as *mut activeDS;
    }
    return pSource;
}

unsafe extern "C" fn TWAIN_SourceManagerHandler(
    pOrigin: pTW_IDENTITY,
    DAT: TW_UINT16,
    MSG: TW_UINT16,
    pData: TW_MEMREF,
) -> TW_UINT16 {
    let mut twRC: TW_UINT16 = (TWRC_SUCCESS as u16);

    match (DAT) as u32 {
        DAT_IDENTITY => {
            match (MSG) as u32 {
                MSG_CLOSEDS => {
                    twRC = TWAIN_CloseDS(pOrigin, pData);
                }
                MSG_GETDEFAULT => {
                    twRC = TWAIN_IdentityGetDefault(pOrigin, pData);
                }
                MSG_GETFIRST => {
                    twRC = TWAIN_IdentityGetFirst(pOrigin, pData);
                }
                MSG_GETNEXT => {
                    twRC = TWAIN_IdentityGetNext(pOrigin, pData);
                }
                MSG_OPENDS => {
                    twRC = TWAIN_OpenDS(pOrigin, pData);
                }
                MSG_USERSELECT => {
                    twRC = TWAIN_UserSelect(pOrigin, pData);
                }
                _ => {
                    /* Unrecognized operation triplet */
                    twRC = (TWRC_FAILURE as u16);
                    DSM_twCC = (TWCC_BADPROTOCOL as u16);
                    WARN!("unrecognized operation triplet\n");
                }
            }
        }

        DAT_PARENT => {
            match (MSG) as u32 {
                MSG_CLOSEDSM => {
                    twRC = TWAIN_CloseDSM(pOrigin, pData);
                }
                MSG_OPENDSM => {
                    twRC = TWAIN_OpenDSM(pOrigin, pData);
                }
                _ => {
                    /* Unrecognized operation triplet */
                    twRC = (TWRC_FAILURE as u16);
                    DSM_twCC = (TWCC_BADPROTOCOL as u16);
                    WARN!("unrecognized operation triplet\n");
                }
            }
        }

        DAT_STATUS => {
            if MSG == (MSG_GET as TW_UINT16) {
                twRC = TWAIN_GetDSMStatus(pOrigin, pData);
            } else {
                twRC = (TWRC_FAILURE as u16);
                DSM_twCC = (TWCC_BADPROTOCOL as u16);
                WARN!("unrecognized operation triplet\n");
            }
        }

        _ => {
            twRC = (TWRC_FAILURE as u16);
            DSM_twCC = (TWCC_BADPROTOCOL as u16);
            WARN!("unrecognized operation triplet\n");
        }
    }

    return twRC;
}

#[no_mangle]
pub unsafe extern "system" fn DSM_Entry(
    pOrigin: pTW_IDENTITY,
    pDest: pTW_IDENTITY,
    DG: TW_UINT32,
    DAT: TW_UINT16,
    MSG: TW_UINT16,
    pData: TW_MEMREF,
) -> TW_UINT16 {
    let mut twRC: TW_UINT16 = (TWRC_SUCCESS as u16); /* Return Code */

    if DG == DG_CONTROL && DAT as ::core::ffi::c_int == (DAT_NULL as i32) {
        let mut pSource: *mut activeDS = TWAIN_LookupSource(pOrigin as *const TW_IDENTITY);
        if pSource.is_null() {
            ERR!("No source associated with pSource %p\n", pDest);
            DSM_twCC = (TWCC_BADPROTOCOL as u16);
            return (TWRC_FAILURE) as TW_UINT16;
        }

        return TWAIN_ControlNull(pOrigin, pDest, pSource, MSG, pData);
    }

    if !pDest.is_null() {
        let mut pSource_0: *mut activeDS = TWAIN_LookupSource(pDest as *const TW_IDENTITY);
        /* This operation's destination is a source */

        if pSource_0.is_null() {
            ERR!("No source associated with pDest %p\n", pDest);
            DSM_twCC = (TWCC_BADDEST as u16);
            return (TWRC_FAILURE) as TW_UINT16;
        }

        if DG == DG_CONTROL && DAT as ::core::ffi::c_int == (DAT_EVENT as i32)
            && MSG as ::core::ffi::c_int == (MSG_PROCESSEVENT as i32)
        {
            twRC = TWAIN_ProcessEvent(pOrigin, pSource_0, pData);
            if twRC as ::core::ffi::c_int == (TWRC_DSEVENT as i32) {
                return twRC;
            }
        }

        if DG == DG_CONTROL && DAT as ::core::ffi::c_int == (DAT_USERINTERFACE as i32)
            && (MSG as ::core::ffi::c_int == (MSG_ENABLEDS as i32)
                || MSG as ::core::ffi::c_int == (MSG_ENABLEDSUIONLY as i32))
            && !pData.is_null()
        {
            (*pSource_0).ui_window = (*(pData as *mut TW_USERINTERFACE)).hParent as HWND;
        }

        DSM_twCC = (TWCC_SUCCESS as u16);
        twRC = (*pSource_0).dsEntry.unwrap_unchecked()(
            pOrigin,
            DG,
            DAT,
            MSG,
            pData,
        );
        return twRC;
    }

    match DG {
        DG_CONTROL => {
            twRC = TWAIN_SourceManagerHandler(pOrigin, DAT, MSG, pData);
        }
        _ => {
            FIXME!("The DSM does not handle DG %ld\n", DG);
            DSM_twCC = (TWCC_BADPROTOCOL as u16);
            twRC = (TWRC_FAILURE as u16);
        }
    }
    return twRC;
}
