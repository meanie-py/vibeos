use ntdll_sys::nt::*;

type PUSER_SOUND_SENTRY = Option<unsafe extern "system" fn() -> BOOLEAN>;

static mut _UserSoundSentry: PUSER_SOUND_SENTRY = Some(FirstSoundSentry);

unsafe extern "system" fn FailSoundSentry() -> BOOLEAN {
    0u8
}

unsafe extern "system" fn FirstSoundSentry() -> BOOLEAN {
    static DLL_NAME: [u16; 7] = [
        b'w' as u16,
        b'i' as u16,
        b'n' as u16,
        b's' as u16,
        b'r' as u16,
        b'v' as u16,
        0,
    ];
    let mut DllString = _UNICODE_STRING {
        Length: 12,
        MaximumLength: 14,
        Buffer: DLL_NAME.as_ptr() as *mut u16,
    };

    static FUNC_NAME: [u8; 17] = [
        b'_', b'U', b's', b'e', b'r', b'S', b'o', b'u', b'n', b'd', b'S', b'e', b'n', b't', b'r',
        b'y', 0,
    ];
    let mut FuncString = STRING {
        Length: 16,
        MaximumLength: 17,
        Buffer: FUNC_NAME.as_ptr() as *mut i8,
    };

    let mut DllHandle: PVOID = core::ptr::null_mut();
    let mut NewSoundSentry: PUSER_SOUND_SENTRY = Some(FailSoundSentry);

    let Status = LdrGetDllHandle(
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        &mut DllString,
        &mut DllHandle,
    );

    if Status >= 0 {
        let mut ProcAddr: PVOID = core::ptr::null_mut();
        let Status = LdrGetProcedureAddress(DllHandle, &mut FuncString, 0, &mut ProcAddr);
        if Status >= 0 {
            NewSoundSentry = Some(core::mem::transmute(ProcAddr));
        }
    }

    _UserSoundSentry = NewSoundSentry;
    (_UserSoundSentry.unwrap())()
}

#[no_mangle]
pub unsafe extern "system" fn BaseSrvSoundSentryNotification(
    _ApiMessage: *mut core::ffi::c_void,
    _ReplyCode: *mut ULONG,
) -> NTSTATUS {
    if (_UserSoundSentry.unwrap())() != 0 {
        STATUS_SUCCESS
    } else {
        STATUS_ACCESS_DENIED
    }
}

// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:       See COPYING in the top level directory
//  * PROJECT:         ReactOS Base API Server DLL
//  * FILE:            subsystems/win/basesrv/sndsntry.c
//  * PURPOSE:         Sound Sentry Notifications
//  * PROGRAMMERS:     Alex Ionescu (alex.ionescu@reactos.org)
//  */
//
// /* INCLUDES *******************************************************************/
//
// #include "basesrv.h"
//
// #define NDEBUG
// #include <debug.h>
//
// /* GLOBALS ********************************************************************/
//
// typedef BOOL (NTAPI *PUSER_SOUND_SENTRY)(VOID);
// BOOL NTAPI FirstSoundSentry(VOID);
//
// static PUSER_SOUND_SENTRY _UserSoundSentry = FirstSoundSentry;
//
// /* PRIVATE FUNCTIONS **********************************************************/
//
// BOOL
// NTAPI
// FailSoundSentry(VOID)
// {
//     /* In case the function can't be found/is unimplemented */
//     return FALSE;
// }
//
// BOOL
// NTAPI
// FirstSoundSentry(VOID)
// {
//     UNICODE_STRING DllString = RTL_CONSTANT_STRING(L"winsrv");
//     STRING FuncString = RTL_CONSTANT_STRING("_UserSoundSentry");
//     HANDLE DllHandle;
//     NTSTATUS Status;
//     PUSER_SOUND_SENTRY NewSoundSentry = FailSoundSentry;
//
//     /* Load winsrv manually */
//     Status = LdrGetDllHandle(NULL, NULL, &DllString, &DllHandle);
//     if (NT_SUCCESS(Status))
//     {
//         /* If it was found, get SoundSentry export */
//         Status = LdrGetProcedureAddress(DllHandle,
//                                         &FuncString,
//                                         0,
//                                         (PVOID*)&NewSoundSentry);
//     }
//
//     /* Set it as the callback for the future, and call it */
//     _UserSoundSentry = NewSoundSentry;
//     return _UserSoundSentry();
// }
//
// /* PUBLIC SERVER APIS *********************************************************/
//
// CSR_API(BaseSrvSoundSentryNotification)
// {
//     /* Call the API and see if it succeeds */
//     return (_UserSoundSentry() ? STATUS_SUCCESS : STATUS_ACCESS_DENIED);
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
