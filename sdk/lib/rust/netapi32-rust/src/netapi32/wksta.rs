use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
pub const ALL_TRANSPORTS: [::core::ffi::c_char; 5] =
    unsafe { ::core::mem::transmute::<[u8; 5], [::core::ffi::c_char; 5]>(*b"M\0\0\0\0") };

pub const TRANSPORT_NBT: [::core::ffi::c_char; 5] =
    unsafe { ::core::mem::transmute::<[u8; 5], [::core::ffi::c_char; 5]>(*b"MNBT\0") };

static mut __wine_dbch_netapi32: __wine_debug_channel = unsafe {
    __wine_debug_channel {
        flags: WINE_DBG_DEFAULT_FLAGS,
        name: ::core::mem::transmute::<[u8; 15], [::core::ffi::c_char; 15]>(
            *b"netapi32\0\0\0\0\0\0\0",
        ),
    }
};

static mut __wine_dbch___default: *mut __wine_debug_channel =
    unsafe { &raw const __wine_dbch_netapi32 as *mut __wine_debug_channel };

pub const NBT_TRANSPORT_NAME_HEADER: [::core::ffi::c_char; 21] = unsafe {
    ::core::mem::transmute::<[u8; 21], [::core::ffi::c_char; 21]>(*b"\\Device\\NetBT_Tcpip_\0")
};

pub const UNKNOWN_TRANSPORT_NAME_HEADER: [::core::ffi::c_char; 26] = unsafe {
    ::core::mem::transmute::<[u8; 26], [::core::ffi::c_char; 26]>(*b"\\Device\\UnknownTransport_\0")
};

pub const MAX_TRANSPORT_NAME: u32 = MAX_INTERFACE_NAME_LEN;
pub const MAX_TRANSPORT_ADDR: u32 = 13;

#[repr(C)]
pub struct WkstaTransportEnumData {
    pub n_adapt: u8,
    pub n_read: u8,
    pub prefmaxlen: u32,
    pub pbuf: *mut *mut u8,
    pub ret: u32,
}
#[no_mangle]
pub unsafe extern "C" fn NETAPI_IsLocalComputer(name: LPCWSTR) -> BOOL {
    let mut name = name; // C reassigns the parameter (name += 2); shadow as mut
    let mut buf: [u16; (MAX_COMPUTERNAME_LENGTH as usize + 1)] = [0; MAX_COMPUTERNAME_LENGTH as usize + 1];
    let mut size: DWORD = buf.len() as DWORD;

    if name.is_null() || *name == 0 {
        return 1;
    }

    let ret = GetComputerNameW(buf.as_mut_ptr(), &mut size as *mut DWORD);
    if ret != 0 && *name == '\\' as u16 && *name.add(1) == '\\' as u16 {
        name = name.add(2);
    }

    if ret != 0 && strcmpiW(name, buf.as_ptr()) == 0 {
        1
    } else {
        0
    }
}
unsafe extern "C" fn wprint_mac(buffer: *mut u16, len: i32, ifRow: *const MIB_IFROW) {
    let mut i: i32 = 0;
    let mut val: u8;

    if buffer.is_null() {
        return;
    }
    if len < 1 {
        return;
    }
    if ifRow.is_null() {
        *buffer = 0u16;
        return;
    }

    i = 0;
    while (i as u32) < (*ifRow).dwPhysAddrLen && 2 * i < len {
        val = (*ifRow).bPhysAddr[i as usize];
        if (val >> 4) > 9 {
            *buffer.offset((2 * i) as isize) = ((val as i32 >> 4) + 'A' as i32 - 10) as u16;
        } else {
            *buffer.offset((2 * i) as isize) = ((val as i32 >> 4) + '0' as i32) as u16;
        }
        if (val & 0xf) > 9 {
            *buffer.offset((2 * i + 1) as isize) = ((val as i32 & 0xf) + 'A' as i32 - 10) as u16;
        } else {
            *buffer.offset((2 * i + 1) as isize) = ((val as i32 & 0xf) + '0' as i32) as u16;
        }
        i += 1;
    }
    *buffer.offset((2 * i) as isize) = 0u16;
}
unsafe extern "C" fn wprint_name(
    buffer: *mut u16,
    len: i32,
    transport: u32,
    ifRow: *mut MIB_IFROW,
) {
    if buffer.is_null() {
        return;
    }
    if ifRow.is_null() {
        *buffer = 0;
        return;
    }

    let mut name: *const i8 = if memcmp(
        (&transport as *const u32).cast::<core::ffi::c_void>(),
        TRANSPORT_NBT.as_ptr().cast::<core::ffi::c_void>(),
        core::mem::size_of::<u32>() as usize,
    ) == 0
    {
        NBT_TRANSPORT_NAME_HEADER.as_ptr()
    } else {
        UNKNOWN_TRANSPORT_NAME_HEADER.as_ptr()
    };

    let mut ptr1 = buffer;
    while *name != 0 && ptr1 < buffer.add(len as usize) {
        *ptr1 = *name as u16;
        ptr1 = ptr1.add(1);
        name = name.add(1);
    }

    let mut ptr2 = (*ifRow).wszName.as_mut_ptr();
    while *ptr2 != 0 && ptr1 < buffer.add(len as usize) {
        *ptr1 = *ptr2;
        ptr1 = ptr1.add(1);
        ptr2 = ptr2.add(1);
    }

    *ptr1 = 0;
}
unsafe extern "system" fn WkstaEnumAdaptersCallback(
    totalLANAs: u8,
    lanaIndex: u8,
    transport: u32,
    data: *const NetBIOSAdapterImpl,
    closure: *mut core::ffi::c_void,
) -> i32 {
    let mut ret: i32 = 0;
    let enumData: *mut WkstaTransportEnumData = closure as *mut WkstaTransportEnumData;
    if !enumData.is_null() && !(*enumData).pbuf.is_null() {
        if lanaIndex as i32 == 0 {
            let mut toAllocate: u32;
            (*enumData).n_adapt = totalLANAs;
            (*enumData).n_read = 0;
            toAllocate = (totalLANAs as usize)
                .wrapping_mul(
                    core::mem::size_of::<WKSTA_TRANSPORT_INFO_0>()
                        .wrapping_add(MAX_TRANSPORT_NAME as usize * core::mem::size_of::<u16>())
                        .wrapping_add(MAX_TRANSPORT_ADDR as usize * core::mem::size_of::<u16>()),
                ) as u32;
            if (*enumData).prefmaxlen != MAX_PREFERRED_LENGTH {
                toAllocate = (*enumData).prefmaxlen;
            }
            NetApiBufferAllocate(toAllocate, (*enumData).pbuf as *mut LPVOID);
        }
        if !(*(*enumData).pbuf).is_null() {
            let spaceFor: u8;
            if (*enumData).prefmaxlen == MAX_PREFERRED_LENGTH {
                spaceFor = totalLANAs;
            } else {
                spaceFor = ((*enumData).prefmaxlen as usize)
                    .wrapping_div(
                        core::mem::size_of::<WKSTA_TRANSPORT_INFO_0>()
                            .wrapping_add(
                                (MAX_TRANSPORT_NAME + MAX_TRANSPORT_ADDR) as usize
                                    * core::mem::size_of::<u16>(),
                            ),
                    ) as u8;
            }
            if (*enumData).n_read < spaceFor {
                let ti: *mut _WKSTA_TRANSPORT_INFO_0;
                let transport_name: *mut u16;
                let transport_addr: *mut u16;
                let mut ifRow: MIB_IFROW = {
                    let mut row: MIB_IFROW = core::mem::zeroed();
                    row.dwOperStatus = IF_OPER_STATUS_NON_OPERATIONAL;
                    row
                };

                ti = (*(*enumData).pbuf).offset(
                    ((*enumData).n_read as usize)
                        .wrapping_mul(core::mem::size_of::<WKSTA_TRANSPORT_INFO_0>()) as isize,
                ) as *mut _WKSTA_TRANSPORT_INFO_0;
                transport_name = (*(*enumData).pbuf)
                    .offset(
                        (totalLANAs as usize)
                            .wrapping_mul(core::mem::size_of::<WKSTA_TRANSPORT_INFO_0>())
                            as isize,
                    )
                    .offset(
                        (((*enumData).n_read as i32 * MAX_TRANSPORT_NAME as i32) as usize)
                            .wrapping_mul(core::mem::size_of::<u16>()) as isize,
                    ) as *mut u16;
                transport_addr = (*(*enumData).pbuf)
                    .offset(
                        (totalLANAs as usize).wrapping_mul(
                            core::mem::size_of::<WKSTA_TRANSPORT_INFO_0>()
                                .wrapping_add(
                                    MAX_TRANSPORT_NAME as usize * core::mem::size_of::<u16>(),
                                ),
                        ) as isize,
                    )
                    .offset(
                        (((*enumData).n_read as i32 * MAX_TRANSPORT_ADDR as i32) as usize)
                            .wrapping_mul(core::mem::size_of::<u16>()) as isize,
                    ) as *mut u16;

                ifRow.dwIndex = (*data).ifIndex;
                GetIfEntry(&mut ifRow);
                (*ti).wkti0_quality_of_service = 0;
                (*ti).wkti0_number_of_vcs = 0;
                (*ti).wkti0_transport_name = transport_name;
                wprint_name(
                    (*ti).wkti0_transport_name,
                    MAX_TRANSPORT_NAME as i32,
                    transport,
                    &mut ifRow,
                );
                (*ti).wkti0_transport_address = transport_addr;
                wprint_mac(
                    (*ti).wkti0_transport_address,
                    MAX_TRANSPORT_ADDR as i32,
                    &mut ifRow,
                );
                if memcmp(
                    &transport as *const _ as *const core::ffi::c_void,
                    &TRANSPORT_NBT as *const _ as *const core::ffi::c_void,
                    core::mem::size_of::<u32>() as usize,
                ) == 0
                {
                    (*ti).wkti0_wan_ish = 1;
                } else {
                    (*ti).wkti0_wan_ish = 0;
                }

                if (*__wine_dbch___default).flags as i32
                    & ((1i32) << __WINE_DBCL_TRACE as i32) != 0
                {
                    ros_dbg_log(
                        __WINE_DBCL_TRACE,
                        __wine_dbch___default,
                        b"dll/win32/netapi32/wksta.c\0"
                            .as_ptr() as *const i8,
                        b"WkstaEnumAdaptersCallback\0".as_ptr() as *const i8,
                        190i32,
                        b"%d of %d:ti at %p\n\0".as_ptr() as *const i8,
                        lanaIndex as i32,
                        totalLANAs as i32,
                        ti,
                    );
                }
                if (*__wine_dbch___default).flags as i32
                    & ((1i32) << __WINE_DBCL_TRACE as i32) != 0
                {
                    ros_dbg_log(
                        __WINE_DBCL_TRACE,
                        __wine_dbch___default,
                        b"dll/win32/netapi32/wksta.c\0"
                            .as_ptr() as *const i8,
                        b"WkstaEnumAdaptersCallback\0".as_ptr() as *const i8,
                        193i32,
                        b"transport_name at %p %s\n\0".as_ptr() as *const i8,
                        (*ti).wkti0_transport_name,
                        debugstr_w((*ti).wkti0_transport_name as *const u16),
                    );
                }
                if (*__wine_dbch___default).flags as i32
                    & ((1i32) << __WINE_DBCL_TRACE as i32) != 0
                {
                    ros_dbg_log(
                        __WINE_DBCL_TRACE,
                        __wine_dbch___default,
                        b"dll/win32/netapi32/wksta.c\0"
                            .as_ptr() as *const i8,
                        b"WkstaEnumAdaptersCallback\0".as_ptr() as *const i8,
                        196i32,
                        b"transport_address at %p %s\n\0".as_ptr() as *const i8,
                        (*ti).wkti0_transport_address,
                        debugstr_w((*ti).wkti0_transport_address as *const u16),
                    );
                }

                (*enumData).n_read = (*enumData).n_read.wrapping_add(1);
                (*enumData).ret = NERR_Success as u32;
                ret = 1;
            } else {
                (*enumData).ret = ERROR_MORE_DATA as u32;
                ret = 0;
            }
        } else {
            (*enumData).ret = ERROR_OUTOFMEMORY as u32;
            ret = 0;
        }
    } else {
        ret = 0;
    }
    ret
}

#[no_mangle]
pub unsafe extern "system" fn NetWkstaTransportEnum(
    ServerName: LPWSTR,
    level: u32,
    pbuf: *mut PBYTE,
    prefmaxlen: u32,
    read_entries: LPDWORD,
    total_entries: PDWORD,
    hresume: PDWORD,
) -> u32 {
    let mut ret: u32 = 0;
    if (*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_TRACE as u32)) != 0 {
        let dbch = __wine_dbch___default;
        ros_dbg_log(
            __WINE_DBCL_TRACE,
            dbch,
            b"dll/win32/netapi32/wksta.c\0".as_ptr() as *const i8,
            b"NetWkstaTransportEnum\0".as_ptr() as *const i8,
            228i32,
            b":%s, 0x%08x, %p, 0x%08x, %p, %p, %p\n\0".as_ptr() as *const i8,
            debugstr_w(ServerName as *const u16),
            level,
            pbuf,
            prefmaxlen,
            read_entries,
            total_entries,
            hresume,
        );
    }
    if NETAPI_IsLocalComputer(ServerName as LPCWSTR) == 0 {
        if (*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_FIXME as u32)) != 0 {
            let dbch = __wine_dbch___default;
            ros_dbg_log(
                __WINE_DBCL_FIXME,
                dbch,
                b"dll/win32/netapi32/wksta.c\0".as_ptr() as *const i8,
                b"NetWkstaTransportEnum\0".as_ptr() as *const i8,
                231i32,
                b":not implemented for non-local computers\n\0".as_ptr() as *const i8,
            );
        }
        ret = ERROR_INVALID_LEVEL;
    } else {
        if !hresume.is_null() && *hresume != 0 {
            if (*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_FIXME as u32)) != 0 {
                let dbch = __wine_dbch___default;
                ros_dbg_log(
                    __WINE_DBCL_FIXME,
                    dbch,
                    b"dll/win32/netapi32/wksta.c\0".as_ptr() as *const i8,
                    b"NetWkstaTransportEnum\0".as_ptr() as *const i8,
                    238i32,
                    b":resume handle not implemented\n\0".as_ptr() as *const i8,
                );
            }
            return ERROR_INVALID_LEVEL;
        }
        match level {
            0 => {
                let mut allTransports: u32 = 0;
                let mut enumData = WkstaTransportEnumData {
                    n_adapt: 0,
                    n_read: 0,
                    prefmaxlen: 0,
                    pbuf: core::ptr::null_mut(),
                    ret: 0,
                };
                if NetBIOSNumAdapters() == 0 {
                    return ERROR_NETWORK_UNREACHABLE;
                }
                if read_entries.is_null() {
                    return STATUS_ACCESS_VIOLATION as u32;
                }
                if total_entries.is_null() || pbuf.is_null() {
                    return RPC_X_NULL_REF_POINTER;
                }
                enumData.prefmaxlen = prefmaxlen;
                enumData.pbuf = pbuf;
                core::ptr::copy_nonoverlapping(
                    ALL_TRANSPORTS.as_ptr() as *const u8,
                    &raw mut allTransports as *mut u32 as *mut u8,
                    core::mem::size_of::<u32>(),
                );
                NetBIOSEnumAdapters(
                    allTransports,
                    Some(WkstaEnumAdaptersCallback as unsafe extern "system" fn(u8, u8, u32, *const NetBIOSAdapterImpl, *mut core::ffi::c_void) -> i32),
                    &raw mut enumData as *mut core::ffi::c_void,
                );
                *read_entries = enumData.n_read as u32;
                *total_entries = enumData.n_adapt as u32;
                if !hresume.is_null() {
                    *hresume = 0;
                }
                ret = enumData.ret;
            }
            _ => {
                if (*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_TRACE as u32)) != 0 {
                    let dbch = __wine_dbch___default;
                    ros_dbg_log(
                        __WINE_DBCL_TRACE,
                        dbch,
                        b"dll/win32/netapi32/wksta.c\0".as_ptr() as *const i8,
                        b"NetWkstaTransportEnum\0".as_ptr() as *const i8,
                        268i32,
                        b"Invalid level %d is specified\n\0".as_ptr() as *const i8,
                        level,
                    );
                }
                ret = ERROR_INVALID_LEVEL;
            }
        }
    }
    ret
}

#[no_mangle]
pub unsafe extern "system" fn NetpGetComputerName(Buffer: *mut LPWSTR) -> NET_API_STATUS {
    let mut dwSize: u32 = (MAX_COMPUTERNAME_LENGTH + 1) as u32;
    if (*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_TRACE as u32)) != 0 {
        ros_dbg_log(
            __WINE_DBCL_TRACE,
            __wine_dbch___default,
            b"dll/win32/netapi32/wksta.c\0".as_ptr() as *const i8,
            b"NetpGetComputerName\0".as_ptr() as *const i8,
            283,
            b"(%p)\n\0".as_ptr() as *const i8,
            Buffer,
        );
    }
    NetApiBufferAllocate(
        (dwSize as usize).wrapping_mul(core::mem::size_of::<WCHAR>()) as u32,
        Buffer as *mut PVOID,
    );
    if GetComputerNameW(*Buffer, &raw mut dwSize) != 0 {
        return NetApiBufferReallocate(
            *Buffer as PVOID,
            ((dwSize + 1) as usize).wrapping_mul(core::mem::size_of::<WCHAR>()) as u32,
            Buffer as *mut PVOID,
        );
    } else {
        NetApiBufferFree(*Buffer as PVOID);
        return ERROR_NOT_ENOUGH_MEMORY as u32;
    }
}

#[no_mangle]
pub unsafe extern "system" fn I_NetNameCompare(
    p1: *mut core::ffi::c_void,
    wkgrp: *mut u16,
    comp: *mut u16,
    p4: *mut core::ffi::c_void,
    p5: *mut core::ffi::c_void,
) -> u32 {
    if (*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_FIXME as u32)) != 0 {
        ros_dbg_log(
            __WINE_DBCL_FIXME,
            __wine_dbch___default,
            c"dll/win32/netapi32/wksta.c".as_ptr(),
            c"I_NetNameCompare".as_ptr(),
            302i32,
            c"(%p %s %s %p %p): stub\n".as_ptr(),
            p1,
            debugstr_w(wkgrp as *const u16),
            debugstr_w(comp as *const u16),
            p4,
            p5,
        );
    }
    ERROR_INVALID_PARAMETER as u32
}

#[no_mangle]
pub unsafe extern "system" fn I_NetNameValidate(
    p1: LPVOID,
    wkgrp: LPWSTR,
    p3: LPVOID,
    p4: LPVOID,
) -> NET_API_STATUS {
    if (*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_FIXME as u32)) != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_FIXME;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/netapi32/wksta.c\0"
                .as_ptr() as *const core::ffi::c_char,
            b"I_NetNameValidate\0".as_ptr() as *const core::ffi::c_char,
            309i32,
            b"(%p %s %p %p): stub\n\0".as_ptr() as *const core::ffi::c_char,
            p1,
            debugstr_w(wkgrp as *const WCHAR),
            p3,
            p4,
        );
    }
    ERROR_INVALID_PARAMETER
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /* Copyright 2002 Andriy Palamarchuk
//  * Copyright (c) 2003 Juan Lang
//  *
//  * netapi32 user functions
//  *
//  * This library is free software; you can redistribute it and/or
//  * modify it under the terms of the GNU Lesser General Public
//  * License as published by the Free Software Foundation; either
//  * version 2.1 of the License, or (at your option) any later version.
//  *
//  * This library is distributed in the hope that it will be useful,
//  * but WITHOUT ANY WARRANTY; without even the implied warranty of
//  * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU
//  * Lesser General Public License for more details.
//  *
//  * You should have received a copy of the GNU Lesser General Public
//  * License along with this library; if not, write to the Free Software
//  * Foundation, Inc., 51 Franklin St, Fifth Floor, Boston, MA 02110-1301, USA
//  */
//
// #include "netapi32.h"
//
// #include <lmwksta.h>
// #include <lmjoin.h>
//
// WINE_DEFAULT_DEBUG_CHANNEL(netapi32);
//
// /************************************************************
//  *                NETAPI_IsLocalComputer
//  *
//  * Checks whether the server name indicates local machine.
//  */
// DECLSPEC_HIDDEN BOOL NETAPI_IsLocalComputer( LMCSTR name )
// {
//     WCHAR buf[MAX_COMPUTERNAME_LENGTH + 1];
//     DWORD size = sizeof(buf) / sizeof(buf[0]);
//     BOOL ret;
//
//     if (!name || !name[0]) return TRUE;
//
//     ret = GetComputerNameW( buf,  &size );
//     if (ret && name[0] == '\\' && name[1] == '\\') name += 2;
//     return ret && !strcmpiW( name, buf );
// }
//
// static void wprint_mac(WCHAR* buffer, int len, const MIB_IFROW *ifRow)
// {
//     int i;
//     unsigned char val;
//
//     if (!buffer)
//         return;
//     if (len < 1)
//         return;
//     if (!ifRow)
//     {
//         *buffer = '\0';
//         return;
//     }
//
//     for (i = 0; i < ifRow->dwPhysAddrLen && 2 * i < len; i++)
//     {
//         val = ifRow->bPhysAddr[i];
//         if ((val >>4) >9)
//             buffer[2*i] = (WCHAR)((val >>4) + 'A' - 10);
//         else
//             buffer[2*i] = (WCHAR)((val >>4) + '0');
//         if ((val & 0xf ) >9)
//             buffer[2*i+1] = (WCHAR)((val & 0xf) + 'A' - 10);
//         else
//             buffer[2*i+1] = (WCHAR)((val & 0xf) + '0');
//     }
//     buffer[2*i]=0;
// }
//
// /* Theoretically this could be too short, except that MS defines
//  * MAX_ADAPTER_NAME as 128, and MAX_INTERFACE_NAME_LEN as 256, and both
//  * represent a count of WCHARs, so even with an extraordinarily long header
//  * this will be plenty
//  */
// #define MAX_TRANSPORT_NAME MAX_INTERFACE_NAME_LEN
// #define MAX_TRANSPORT_ADDR 13
//
// #define NBT_TRANSPORT_NAME_HEADER "\\Device\\NetBT_Tcpip_"
// #define UNKNOWN_TRANSPORT_NAME_HEADER "\\Device\\UnknownTransport_"
//
// static void wprint_name(WCHAR *buffer, int len, ULONG transport,
//  PMIB_IFROW ifRow)
// {
//     WCHAR *ptr1, *ptr2;
//     const char *name;
//
//     if (!buffer)
//         return;
//     if (!ifRow)
//     {
//         *buffer = '\0';
//         return;
//     }
//
//     if (!memcmp(&transport, TRANSPORT_NBT, sizeof(ULONG)))
//         name = NBT_TRANSPORT_NAME_HEADER;
//     else
//         name = UNKNOWN_TRANSPORT_NAME_HEADER;
//
//     for (ptr1 = buffer; *name && ptr1 < buffer + len; ptr1++, name++)
//         *ptr1 = *name;
//     for (ptr2 = ifRow->wszName; *ptr2 && ptr1 < buffer + len; ptr1++, ptr2++)
//         *ptr1 = *ptr2;
//     *ptr1 = '\0';
// }
//
// /***********************************************************************
//  *                NetWkstaTransportEnum  (NETAPI32.@)
//  */
//  
// struct WkstaTransportEnumData
// {
//     UCHAR          n_adapt;
//     UCHAR          n_read;
//     DWORD          prefmaxlen;
//     LPBYTE        *pbuf;
//     NET_API_STATUS ret;
// };
//
// /**********************************************************************/
//
// static BOOL WkstaEnumAdaptersCallback(UCHAR totalLANAs, UCHAR lanaIndex,
//  ULONG transport, const NetBIOSAdapterImpl *data, void *closure)
// {
//     BOOL ret;
//     struct WkstaTransportEnumData *enumData = closure;
//
//     if (enumData && enumData->pbuf)
//     {
//         if (lanaIndex == 0)
//         {
//             DWORD toAllocate;
//
//             enumData->n_adapt = totalLANAs;
//             enumData->n_read = 0;
//
//             toAllocate = totalLANAs * (sizeof(WKSTA_TRANSPORT_INFO_0)
//              + MAX_TRANSPORT_NAME * sizeof(WCHAR) +
//              MAX_TRANSPORT_ADDR * sizeof(WCHAR));
//             if (enumData->prefmaxlen != MAX_PREFERRED_LENGTH)
//                 toAllocate = enumData->prefmaxlen;
//             NetApiBufferAllocate(toAllocate, (LPVOID *)enumData->pbuf);
//         }
//         if (*(enumData->pbuf))
//         {
//             UCHAR spaceFor;
//
//             if (enumData->prefmaxlen == MAX_PREFERRED_LENGTH)
//                 spaceFor = totalLANAs;
//             else
//                 spaceFor = enumData->prefmaxlen /
//                  (sizeof(WKSTA_TRANSPORT_INFO_0) + (MAX_TRANSPORT_NAME +
//                  MAX_TRANSPORT_ADDR) * sizeof(WCHAR));
//             if (enumData->n_read < spaceFor)
//             {
//                 PWKSTA_TRANSPORT_INFO_0 ti;
//                 LMSTR transport_name, transport_addr;
//                 MIB_IFROW ifRow;
//
//                 ti = (PWKSTA_TRANSPORT_INFO_0)(*(enumData->pbuf) +
//                  enumData->n_read * sizeof(WKSTA_TRANSPORT_INFO_0));
//                 transport_name = (LMSTR)(*(enumData->pbuf) +
//                  totalLANAs * sizeof(WKSTA_TRANSPORT_INFO_0) +
//                  enumData->n_read * MAX_TRANSPORT_NAME * sizeof(WCHAR));
//                 transport_addr = (LMSTR)(*(enumData->pbuf) +
//                  totalLANAs * (sizeof(WKSTA_TRANSPORT_INFO_0) +
//                  MAX_TRANSPORT_NAME * sizeof(WCHAR)) +
//                  enumData->n_read * MAX_TRANSPORT_ADDR * sizeof(WCHAR));
//
//                 ifRow.dwIndex = data->ifIndex;
//                 GetIfEntry(&ifRow);
//                 ti->wkti0_quality_of_service = 0;
//                 ti->wkti0_number_of_vcs = 0;
//                 ti->wkti0_transport_name = transport_name;
//                 wprint_name(ti->wkti0_transport_name, MAX_TRANSPORT_NAME,
//                  transport, &ifRow);
//                 ti->wkti0_transport_address = transport_addr;
//                 wprint_mac(ti->wkti0_transport_address, MAX_TRANSPORT_ADDR,
//                  &ifRow);
//                 if (!memcmp(&transport, TRANSPORT_NBT, sizeof(ULONG)))
//                     ti->wkti0_wan_ish = TRUE;
//                 else
//                     ti->wkti0_wan_ish = FALSE;
//                 TRACE("%d of %d:ti at %p\n", lanaIndex, totalLANAs, ti);
//                 TRACE("transport_name at %p %s\n",
//                  ti->wkti0_transport_name,
//                  debugstr_w(ti->wkti0_transport_name));
//                 TRACE("transport_address at %p %s\n",
//                  ti->wkti0_transport_address,
//                  debugstr_w(ti->wkti0_transport_address));
//                 enumData->n_read++;
//                 enumData->ret = NERR_Success;
//                 ret = TRUE;
//             }
//             else
//             {
//                 enumData->ret = ERROR_MORE_DATA;
//                 ret = FALSE;
//             }
//         }
//         else
//         {
//             enumData->ret = ERROR_OUTOFMEMORY;
//             ret = FALSE;
//         }
//     }
//     else
//         ret = FALSE;
//     return ret;
// }
//
// /**********************************************************************/
//
// NET_API_STATUS WINAPI 
// NetWkstaTransportEnum(LMSTR ServerName, DWORD level, PBYTE* pbuf,
//       DWORD prefmaxlen, LPDWORD read_entries,
//       PDWORD total_entries, PDWORD hresume)
// {
//     NET_API_STATUS ret;
//
//     TRACE(":%s, 0x%08x, %p, 0x%08x, %p, %p, %p\n", debugstr_w(ServerName), 
//      level, pbuf, prefmaxlen, read_entries, total_entries,hresume);
//     if (!NETAPI_IsLocalComputer(ServerName))
//     {
//         FIXME(":not implemented for non-local computers\n");
//         ret = ERROR_INVALID_LEVEL;
//     }
//     else
//     {
//         if (hresume && *hresume)
//         {
//           FIXME(":resume handle not implemented\n");
//           return ERROR_INVALID_LEVEL;
//         }
//
//         switch (level)
//         {
//             case 0: /* transport info */
//             {
//                 ULONG allTransports;
//                 struct WkstaTransportEnumData enumData;
//
//                 if (NetBIOSNumAdapters() == 0)
//                   return ERROR_NETWORK_UNREACHABLE;
//                 if (!read_entries)
//                   return STATUS_ACCESS_VIOLATION as u32;
//                 if (!total_entries || !pbuf)
//                   return RPC_X_NULL_REF_POINTER;
//
//                 enumData.prefmaxlen = prefmaxlen;
//                 enumData.pbuf = pbuf;
//                 memcpy(&allTransports, ALL_TRANSPORTS, sizeof(ULONG));
//                 NetBIOSEnumAdapters(allTransports, WkstaEnumAdaptersCallback,
//                  &enumData);
//                 *read_entries = enumData.n_read;
//                 *total_entries = enumData.n_adapt;
//                 if (hresume) *hresume= 0;
//                 ret = enumData.ret;
//                 break;
//             }
//             default:
//                 TRACE("Invalid level %d is specified\n", level);
//                 ret = ERROR_INVALID_LEVEL;
//         }
//     }
//     return ret;
// }
//
//
// /************************************************************
//  *                NetpGetComputerName  (NETAPI32.@)
//  */
// NET_API_STATUS WINAPI NetpGetComputerName(LPWSTR *Buffer)
// {
//     DWORD dwSize = MAX_COMPUTERNAME_LENGTH + 1;
//
//     TRACE("(%p)\n", Buffer);
//     NetApiBufferAllocate(dwSize * sizeof(WCHAR), (LPVOID *) Buffer);
//     if (GetComputerNameW(*Buffer,  &dwSize))
//     {
//         return NetApiBufferReallocate(
//             *Buffer, (dwSize + 1) * sizeof(WCHAR),
//             (LPVOID *) Buffer);
//     }
//     else
//     {
//         NetApiBufferFree(*Buffer);
//         return ERROR_NOT_ENOUGH_MEMORY;
//     }
// }
//
// NET_API_STATUS WINAPI I_NetNameCompare(LPVOID p1, LPWSTR wkgrp, LPWSTR comp,
//  LPVOID p4, LPVOID p5)
// {
//     FIXME("(%p %s %s %p %p): stub\n", p1, debugstr_w(wkgrp), debugstr_w(comp),
//      p4, p5);
//     return ERROR_INVALID_PARAMETER;
// }
//
// NET_API_STATUS WINAPI I_NetNameValidate(LPVOID p1, LPWSTR wkgrp, LPVOID p3,
//  LPVOID p4)
// {
//     FIXME("(%p %s %p %p): stub\n", p1, debugstr_w(wkgrp), p3, p4);
//     return ERROR_INVALID_PARAMETER;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
