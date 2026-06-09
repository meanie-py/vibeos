use ntdll_sys::nt::*;

#[repr(C)]
struct BASE_APP_COMPAT_EXTRA1 {
    String1: [WCHAR; 32],
    tdwFlags: ULONG,
    SizeOfStruct: ULONG,
    tdwMagic: ULONG,
    ttrExe: ULONG,
    dword50: ULONG,
    dword54: ULONG,
    dword58: ULONG,
    ttrLayer: PVOID,
    String2: [i8; 28],
    dword7C: ULONG,
    dword80: ULONG,
    field_84: ULONG,
    String3: [i8; 256],
}

type PBASE_APP_COMPAT_EXTRA1 = *mut BASE_APP_COMPAT_EXTRA1;

#[repr(C)]
struct BASE_APP_COMPAT_DATA {
    FileName: _UNICODE_STRING,
    ProcessHandle: HANDLE,
    Flags: ULONG,
    Code: USHORT,
    Unknown_012: USHORT,
    Environment: PVOID,
    EnvironmentSize: ULONG,
    AppCompExtra1: PBASE_APP_COMPAT_EXTRA1,
    AppCompExtra1Size: ULONG,
    AppCompExtra2: PVOID,
    AppCompExtra2Size: ULONG,
    CheckRunAppResult: BOOLEAN,
    Flags2: ULONG,
}

type PBASE_APP_COMPAT_DATA = *mut BASE_APP_COMPAT_DATA;

#[no_mangle]
pub unsafe extern "system" fn BaseSrvCheckApplicationCompatibility(ApiMessage: PVOID) -> NTSTATUS {
    let AppCompatData = core::ptr::addr_of_mut!(
        (*(ApiMessage as *mut CSR_API_MESSAGE))
            .__bindgen_anon_1
            .__bindgen_anon_1
            .Data
    ) as PBASE_APP_COMPAT_DATA;

    (*AppCompatData).CheckRunAppResult = 0u8;

    STATUS_SUCCESS
}

// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:       See COPYING in the top level directory
//  * PROJECT:         ReactOS Base API Server DLL
//  * FILE:            subsystems/win/basesrv/appcompat.c
//  * PURPOSE:         Initialization
//  * PROGRAMMERS:     Timo Kreuzer (timo.kreuzer@reactos.org)
//  */
//
// /* INCLUDES *******************************************************************/
//
// #include "basesrv.h"
// #include "api.h"
//
// #define NDEBUG
// #include <debug.h>
//
// /* GLOBALS ********************************************************************/
//
// typedef struct _BASE_APP_COMPAT_EXTRA1
// {
//     WCHAR String1[32];
//     ULONG tdwFlags;
//     ULONG SizeOfStruct;
//     ULONG tdwMagic;
//     ULONG ttrExe;
//     ULONG dword50;
//     ULONG dword54;
//     ULONG dword58;
//     PVOID ttrLayer;
//     CHAR String2[28];
//     ULONG dword7C;
//     ULONG dword80;
//     ULONG field_84;
//     CHAR String3[256];
// } BASE_APP_COMPAT_EXTRA1, *PBASE_APP_COMPAT_EXTRA1;
//
// typedef struct _BASE_APP_COMPAT_DATA
// {
//     UNICODE_STRING FileName;
//     HANDLE ProcessHandle;
//     ULONG Flags;
//     USHORT Code;
//     USHORT Unknown_012;
//     PVOID Environment;
//     ULONG EnvironmentSize;
//     PBASE_APP_COMPAT_EXTRA1 AppCompExtra1;
//     ULONG AppCompExtra1Size;
//     PVOID AppCompExtra2;
//     ULONG AppCompExtra2Size;
//     BOOL CheckRunAppResult;
//     ULONG Flags2;
// } BASE_APP_COMPAT_DATA, *PBASE_APP_COMPAT_DATA;
//
// /* PUBLIC SERVER APIS *********************************************************/
//
// CSR_API(BaseSrvCheckApplicationCompatibility)
// {
//     PBASE_APP_COMPAT_DATA AppCompatData = (PBASE_APP_COMPAT_DATA)&ApiMessage->Data;
//     DPRINT1("BASESRV: BaseSrvCheckApplicationCompatibility(%wZ)\n",
//            AppCompatData->FileName);
//
//     AppCompatData->CheckRunAppResult = FALSE;
//
//     return STATUS_SUCCESS;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
