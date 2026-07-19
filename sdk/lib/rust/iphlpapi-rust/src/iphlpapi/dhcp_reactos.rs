use crate::prelude::*;
use core::ffi::c_void;
#[no_mangle]
pub unsafe extern "C" fn getDhcpInfoForAdapter(AdapterIndex: u32, ptr: *mut _IP_ADAPTER_INFO) -> u32 {
    let mut ifname: *const i8 = core::ptr::null();
    let mut hKeyInterfaces: HKEY = core::ptr::null_mut();
    let mut hKeyInterface: HKEY = core::ptr::null_mut();
    let mut dwValue: u32 = 0;
    let mut dwSize: u32 = 0;
    let mut dwType: u32 = 0;
    let mut ret: u32 = ERROR_SUCCESS;

    (*ptr).DhcpEnabled = 0;
    (*ptr).LeaseObtained = 0;
    (*ptr).LeaseExpires = 0;
    strcpy(
        core::ptr::addr_of_mut!((*ptr).DhcpServer.IpAddress.String[0]),
        b"\0".as_ptr() as *const i8,
    );

    ifname = getInterfaceNameByIndex(AdapterIndex);
    if ifname.is_null() {
        return ERROR_OUTOFMEMORY;
    }

    // "SYSTEM\\CurrentControlSet\\Services\\Tcpip\\Parameters\\Interfaces" in UTF-16
    let interfaces_path: [u16; 62] = [
        0x0053,0x0059,0x0053,0x0054,0x0045,0x004D,0x005C,
        0x0043,0x0075,0x0072,0x0072,0x0065,0x006E,0x0074,
        0x0043,0x006F,0x006E,0x0074,0x0072,0x006F,0x006C,
        0x0053,0x0065,0x0074,0x005C,0x0053,0x0065,0x0072,
        0x0076,0x0069,0x0063,0x0065,0x0073,0x005C,0x0054,
        0x0063,0x0070,0x0069,0x0070,0x005C,0x0050,0x0061,
        0x0072,0x0061,0x006D,0x0065,0x0074,0x0065,0x0072,
        0x0073,0x005C,0x0049,0x006E,0x0074,0x0065,0x0072,
        0x0066,0x0061,0x0063,0x0065,0x0073,0x0000,
    ];

    'done: {
    ret = (RegOpenKeyExW(
        HKEY_LOCAL_MACHINE,
        interfaces_path.as_ptr(),
        0,
        KEY_READ,
        &raw mut hKeyInterfaces,
    ) as u32);
    if ret != ERROR_SUCCESS {
        break 'done;
    }

    ret = (RegOpenKeyExA(
        hKeyInterfaces,
        ifname,
        0,
        KEY_READ,
        &raw mut hKeyInterface,
    ) as u32);
    if ret != ERROR_SUCCESS {
        break 'done;
    }

    dwSize = core::mem::size_of_val(&(*ptr).DhcpEnabled) as u32;
    let enable_dhcp_path: [u16; 11] = [
        0x0045,0x006E,0x0061,0x0062,0x006C,0x0065,
        0x0044,0x0048,0x0043,0x0050,0x0000,
    ];
    ret = (RegQueryValueExW(
        hKeyInterface,
        enable_dhcp_path.as_ptr(),
        core::ptr::null_mut(),
        &raw mut dwType,
        &raw mut (*ptr).DhcpEnabled as *mut u8,
        &raw mut dwSize,
    ) as u32);
    if ret != ERROR_SUCCESS {
        (*ptr).DhcpEnabled = 0;
    }

    if (*ptr).DhcpEnabled != 0 {
        // LeaseObtainedTime
        dwSize = core::mem::size_of_val(&(*ptr).LeaseObtained) as u32;
        let lease_obtained_path: [u16; 18] = [
            0x004C,0x0065,0x0061,0x0073,0x0065,
            0x004F,0x0062,0x0074,0x0061,0x0069,0x006E,
            0x0065,0x0064,0x0054,0x0069,0x006D,0x0065,
            0x0000,
        ];
        ret = (RegQueryValueExW(
            hKeyInterface,
            lease_obtained_path.as_ptr(),
            core::ptr::null_mut(),
            &raw mut dwType,
            &raw mut dwValue as *mut u32 as *mut u8,
            &raw mut dwSize,
        ) as u32);
        if ret == ERROR_SUCCESS {
            (*ptr).LeaseObtained = dwValue as time_t;
        }

        // LeaseTerminatesTime
        dwSize = core::mem::size_of_val(&dwValue) as u32;
        let lease_terminates_path: [u16; 20] = [
            0x004C,0x0065,0x0061,0x0073,0x0065,
            0x0054,0x0065,0x0072,0x006D,0x0069,0x006E,
            0x0061,0x0074,0x0065,0x0073,0x0054,0x0069,
            0x006D,0x0065,0x0000,
        ];
        ret = (RegQueryValueExW(
            hKeyInterface,
            lease_terminates_path.as_ptr(),
            core::ptr::null_mut(),
            &raw mut dwType,
            &raw mut dwValue as *mut u32 as *mut u8,
            &raw mut dwSize,
        ) as u32);
        if ret == ERROR_SUCCESS {
            (*ptr).LeaseExpires = dwValue as time_t;
        }

        // DhcpServer (Ansi)
        dwSize = core::mem::size_of_val(&(*ptr).DhcpServer.IpAddress.String) as u32;
        ret = (RegQueryValueExA(
            hKeyInterface,
            b"DhcpServer\0".as_ptr() as *const i8,
            core::ptr::null_mut(),
            &raw mut dwType,
            core::ptr::addr_of_mut!((*ptr).DhcpServer.IpAddress.String[0]) as *mut u8,
            &raw mut dwSize,
        ) as u32);
        if ret != ERROR_SUCCESS {
            strcpy(
                core::ptr::addr_of_mut!((*ptr).DhcpServer.IpAddress.String[0]),
                b"\0".as_ptr() as *const i8,
            );
        }
    }
    ret = ERROR_SUCCESS;
    } // end 'done block

    // done:  (dhcp_reactos.c:90) — the shared cleanup tail every `goto done` falls into.
    if !hKeyInterface.is_null() {
        RegCloseKey(hKeyInterface);
    }
    if !hKeyInterfaces.is_null() {
        RegCloseKey(hKeyInterfaces);
    }
    if !ifname.is_null() {
        consumeInterfaceName(ifname as *const u8);
    }
    return ret;
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:     ReactOS Networking
//  * LICENSE:     GPL - See COPYING in the top level directory
//  * FILE:        lib/iphlpapi/dhcp_reactos.c
//  * PURPOSE:     DHCP helper functions for ReactOS
//  * COPYRIGHT:   Copyright 2006 Ge van Geldorp <gvg@reactos.org>
//  */
//
// #include "iphlpapi_private.h"
//
// DWORD
// getDhcpInfoForAdapter(
//     DWORD AdapterIndex,
//     PIP_ADAPTER_INFO ptr)
// {
//     const char *ifname = NULL;
//     HKEY hKeyInterfaces = NULL, hKeyInterface = NULL;
//     DWORD dwValue, dwSize, dwType;
//     DWORD ret = ERROR_SUCCESS;
//
//     ptr->DhcpEnabled = 0;
//     ptr->LeaseObtained = 0;
//     ptr->LeaseExpires = 0;
//     strcpy(ptr->DhcpServer.IpAddress.String, "");
//
//     ifname = getInterfaceNameByIndex(AdapterIndex);
//     if (!ifname)
//         return ERROR_OUTOFMEMORY;
//
//     ret = RegOpenKeyExW(HKEY_LOCAL_MACHINE,
//                         L"SYSTEM\\CurrentControlSet\\Services\\Tcpip\\Parameters\\Interfaces",
//                         0,
//                         KEY_READ,
//                         &hKeyInterfaces);
//     if (ret != ERROR_SUCCESS)
//         goto done;
//
//     ret = RegOpenKeyExA(hKeyInterfaces,
//                         ifname,
//                         0,
//                         KEY_READ,
//                         &hKeyInterface);
//     if (ret != ERROR_SUCCESS)
//         goto done;
//
//     dwSize = sizeof(ptr->DhcpEnabled);
//     ret = RegQueryValueExW(hKeyInterface,
//                            L"EnableDHCP",
//                            NULL,
//                            &dwType,
//                            (PBYTE)&ptr->DhcpEnabled,
//                            &dwSize);
//     if (ret != ERROR_SUCCESS)
//         ptr->DhcpEnabled = 0;
//
//     if (ptr->DhcpEnabled != 0)
//     {
//         dwSize = sizeof(ptr->LeaseObtained);
//         ret = RegQueryValueExW(hKeyInterface,
//                                L"LeaseObtainedTime",
//                                NULL,
//                                &dwType,
//                                (PBYTE)&dwValue,
//                                &dwSize);
//         if (ret == ERROR_SUCCESS)
//             ptr->LeaseObtained = (time_t)dwValue;
//
//         dwSize = sizeof(dwValue);
//         ret = RegQueryValueExW(hKeyInterface,
//                                L"LeaseTerminatesTime",
//                                NULL,
//                                &dwType,
//                                (PBYTE)&dwValue,
//                                &dwSize);
//         if (ret == ERROR_SUCCESS)
//             ptr->LeaseExpires = (time_t)dwValue;
//
//         dwSize = sizeof(ptr->DhcpServer.IpAddress.String);
//         ret = RegQueryValueExA(hKeyInterface,
//                                "DhcpServer",
//                                NULL,
//                                &dwType,
//                                (PBYTE)&ptr->DhcpServer.IpAddress.String,
//                                &dwSize);
//         if (ret != ERROR_SUCCESS)
//             strcpy(ptr->DhcpServer.IpAddress.String, "");
//     }
//     ret = ERROR_SUCCESS;
//
// done:
//     if (hKeyInterface)
//         RegCloseKey(hKeyInterface);
//
//     if (hKeyInterfaces)
//         RegCloseKey(hKeyInterfaces);
//
//     if (ifname)
//         consumeInterfaceName(ifname);
//
//     return ret;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
