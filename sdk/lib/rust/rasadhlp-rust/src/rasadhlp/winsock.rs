use crate::prelude::*;
use crate::rasadhlp::autodial::{AcsHlpAttemptConnection, AcsHlpNoteNewConnection};
use core::ffi::c_void;

#[no_mangle]
pub unsafe extern "system" fn WSAttemptAutodialAddr(
    Name: *const SOCKADDR,
    NameLength: INT,
) -> BOOL {
    let Ip = Name as PSOCKADDR_IN;
    let NetBios = Name as PSOCKADDR_NB;
    let mut AutodialAddress: AUTODIAL_ADDR = core::mem::zeroed();

    match (*Name).sa_family as u32 {
        AF_INET => {
            AutodialAddress.Family = AutoDialIp;
            AutodialAddress.__bindgen_anon_1.Ip4Address = (*Ip).sin_addr;
        }
        AF_NETBIOS => {
            AutodialAddress.Family = AutoDialNetBios;
            RtlCopyMemory!(
                core::ptr::addr_of_mut!(AutodialAddress.__bindgen_anon_1.NetBiosAddress) as *mut _,
                (*NetBios).snb_name.as_ptr() as *const _,
                NETBIOS_NAME_LENGTH,
            );
        }
        _ => {
            return (FALSE) as i32;
        }
    }

    return AcsHlpAttemptConnection(core::ptr::addr_of_mut!(AutodialAddress)) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn WSAttemptAutodialName(Restrictions: LPWSAQUERYSETW) -> BOOL {
    let mut AutodialAddress: AUTODIAL_ADDR = core::mem::zeroed();
    let mut AnsiIp: [CHAR; 17] = core::mem::zeroed();
    let Guid: LPGUID = (*Restrictions).lpServiceClassId;

    if (*Restrictions).lpszServiceInstanceName.is_null() {
        return (FALSE) as i32;
    }

    if memcmp(Guid as *const core::ffi::c_void, &HostnameGuid as *const GUID as *const core::ffi::c_void, core::mem::size_of::<GUID>()) == 0 {
        AutodialAddress.Family = AutoDialIpHost;
        WideCharToMultiByte(
            CP_ACP,
            0,
            (*Restrictions).lpszServiceInstanceName,
            -1,
            AutodialAddress.__bindgen_anon_1.HostName.as_mut_ptr(),
            (INTERNET_MAX_PATH_LENGTH - 1) as c_int,
            core::ptr::null(),
            core::ptr::null_mut(),
        );

        return AcsHlpAttemptConnection(&mut AutodialAddress) as BOOL;
    } else if memcmp(Guid as *const core::ffi::c_void, &AddressGuid as *const GUID as *const core::ffi::c_void, core::mem::size_of::<GUID>()) == 0 {
        AutodialAddress.Family = AutoDialIp;

        WideCharToMultiByte(
            CP_ACP,
            0,
            (*Restrictions).lpszServiceInstanceName,
            -1,
            AnsiIp.as_mut_ptr(),
            (core::mem::size_of_val(&AnsiIp) - 1) as c_int,
            core::ptr::null(),
            core::ptr::null_mut(),
        );
        _strlwr(AnsiIp.as_mut_ptr());
        AutodialAddress.__bindgen_anon_1.Ip4Address.S_un.S_addr = inet_addr(AnsiIp.as_ptr());

        if AutodialAddress.__bindgen_anon_1.Ip4Address.S_un.S_addr == INADDR_NONE {
            return FALSE as BOOL;
        }

        return AcsHlpAttemptConnection(&mut AutodialAddress) as BOOL;
    } else {
        return FALSE as BOOL;
    }
}

#[no_mangle]
pub unsafe extern "system" fn WSNoteSuccessfulHostentLookup(
    Name: *const i8,
    Address: u32,
) {
    let mut AutodialAddress: AUTODIAL_ADDR = core::mem::zeroed();
    let mut AutodialConnection: AUTODIAL_CONN = core::mem::zeroed();

    if Name.is_null() || strlen(Name) == 0 {
        return;
    }

    AutodialAddress.Family = AutoDialIpHost;
    strcpy(AutodialAddress.__bindgen_anon_1.HostName.as_mut_ptr(), Name);

    AutodialConnection.Family = ConnectionIp;
    AutodialConnection.__bindgen_anon_1.Ip4Address = Address;
    AcsHlpNoteNewConnection(&mut AutodialAddress, &mut AutodialConnection);
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:   See COPYING in the top level directory
//  * PROJECT:     ReactOS Winsock 2 SPI
//  * FILE:        lib/mswsock/lib/init.c
//  * PURPOSE:     DLL Initialization
//  */
//
// #include "precomp.h"
//
// #include <winnls.h>
// #include <nsp_dns.h>
//
// /* FUNCTIONS *****************************************************************/
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// WSAttemptAutodialAddr( const SOCKADDR  *Name,
//                        INT NameLength)
// {
//     PSOCKADDR_IN Ip = (PSOCKADDR_IN)Name;
//     PSOCKADDR_NB NetBios = (PSOCKADDR_NB)Name;
//     AUTODIAL_ADDR AutodialAddress;
//
//     /* Check the family type */
//     switch (Name->sa_family)
//     {
//         case AF_INET:
//             /* Normal IPv4, set the Autodial Address Data */
//             AutodialAddress.Family = AutoDialIp;
//             AutodialAddress.Ip4Address = Ip->sin_addr;
//             break;
//
//         case AF_NETBIOS:
//             /* NetBIOS, set the Autodial Address Data*/
//             AutodialAddress.Family = AutoDialNetBios;
//             RtlCopyMemory(&AutodialAddress.NetBiosAddress,
//                           NetBios->snb_name,
//                           NETBIOS_NAME_LENGTH);
//             break;
//
//         default:
//             /* Unsupported family type */
//             return FALSE;
//     }
//
//     /* Call the public routine */
//     return AcsHlpAttemptConnection(&AutodialAddress);
// }
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// WSAttemptAutodialName( const LPWSAQUERYSETW Restrictions)
// {
//     AUTODIAL_ADDR AutodialAddress;
//     CHAR AnsiIp[17];
//     LPGUID Guid = Restrictions->lpServiceClassId;
//
//     /* Make sure we actually have a name */
//     if (!Restrictions->lpszServiceInstanceName) return FALSE;
//
//     /* Check if this is the Hostname GUID */
//     if (!memcmp(Guid, &HostnameGuid, sizeof(GUID)))
//     {
//         /* It is. Set up the Autodial Address Data */
//         AutodialAddress.Family = AutoDialIpHost;
//         WideCharToMultiByte(CP_ACP,
//                             0,
//                             Restrictions->lpszServiceInstanceName,
//                             -1,
//                             AutodialAddress.HostName,
//                             INTERNET_MAX_PATH_LENGTH - 1,
//                             0,
//                             0);
//
//         /* Call the public routine */
//         return AcsHlpAttemptConnection(&AutodialAddress);
//     }
//     else if (!memcmp(Guid, &AddressGuid, sizeof(GUID)))
//     {
//         /* It's actually the IP String GUID */
//         AutodialAddress.Family = AutoDialIp;
//
//         /* Convert the IP String to ANSI and then convert it to IP */
//         WideCharToMultiByte(CP_ACP,
//                             0,
//                             Restrictions->lpszServiceInstanceName,
//                             -1,
//                             AnsiIp,
//                             sizeof(AnsiIp) - 1,
//                             0,
//                             0);
//         _strlwr(AnsiIp);
//         AutodialAddress.Ip4Address.S_un.S_addr = inet_addr(AnsiIp);
//
//         /* Make sure the IP is valid */
//         if (AutodialAddress.Ip4Address.S_un.S_addr == -1) return FALSE;
//
//         /* Call the public routine */
//         return AcsHlpAttemptConnection(&AutodialAddress);
//     }
//     else
//     {
//         /* Unknown GUID type */
//         return FALSE;
//     }
// }
//
// /*
//  * @implemented
//  */
// VOID
// WINAPI
// WSNoteSuccessfulHostentLookup( const CHAR  *Name,
//                                const ULONG Address)
// {
//     AUTODIAL_ADDR AutodialAddress;
//     AUTODIAL_CONN AutodialConnection;
//
//     /* Make sure there actually is a name */
//     if (!(Name) || !strlen(Name)) return;
//
//     /* Setup the Address */
//     AutodialAddress.Family = AutoDialIpHost;
//     strcpy(AutodialAddress.HostName, Name);
//
//     /* Setup the new connection */
//     AutodialConnection.Family = ConnectionIp;
//     AutodialConnection.Ip4Address = Address;
//     AcsHlpNoteNewConnection(&AutodialAddress, &AutodialConnection);
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
