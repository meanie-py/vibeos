use crate::prelude::*;
use core::ffi::c_void;

// ==== iphlpapi_main.c → Rust (74 fns, DeepSeek + c2rust; assembled from
//      .responses/20260718-002514-iphlpapi_main.c). Cohab-flip of dll/win32/iphlpapi/iphlpapi_main.c. ====

// ---- 00_preamble.rs ----
#[repr(C)]
pub struct _NAME_SERVER_LIST_CONTEXT {
    pub uSizeAvailable: ULONG,
    pub uSizeRequired: ULONG,
    pub pData: PIP_PER_ADAPTER_INFO,
    pub NumServers: UINT,
    pub pLastAddr: *mut IP_ADDR_STRING,
}

pub type NAME_SERVER_LIST_CONTEXT = _NAME_SERVER_LIST_CONTEXT;
pub type PNAME_SERVER_LIST_CONTEXT = *mut _NAME_SERVER_LIST_CONTEXT;
// ---- 01_DllMain.rs ----
#[no_mangle]
pub unsafe extern "system" fn DllMain(
    hinstDLL: HINSTANCE,
    fdwReason: DWORD,
    lpvReserved: LPVOID,
) -> BOOL {
    match fdwReason {
        DLL_PROCESS_ATTACH => {
            DisableThreadLibraryCalls(hinstDLL);
            interfaceMapInit();
        }
        DLL_PROCESS_DETACH => {
            interfaceMapFree();
        }
        _ => {}
    }
    return 1;
}

// ---- 02_AddIPAddress.rs ----
#[no_mangle]
pub unsafe extern "system" fn AddIPAddress(
    Address: IPAddr,
    Netmask: IPMask,
    IfIndex: DWORD,
    NteContext: PULONG,
    NteInstance: PULONG,
) -> DWORD {

    return RtlNtStatusToDosError(addIPAddress(
        Address,
        Netmask,
        IfIndex,
        NteContext,
        NteInstance,
    ));
}
// ---- 03_getInterfaceGatewayByIndex.rs ----
unsafe fn getInterfaceGatewayByIndex(index: DWORD) -> DWORD {
    let numRoutes: DWORD = getNumRoutes();
    let table: *mut RouteTable = getRouteTable();
    if table.is_null() {
        return 0;
    }

    let mut retVal: DWORD = 0;
    let mut ndx: DWORD = 0;
    while ndx < numRoutes {
        if (*(*table).routes.as_ptr().add(ndx as usize)).ifIndex == index
            && (*(*table).routes.as_ptr().add(ndx as usize)).dest == 0
        {
            retVal = (*(*table).routes.as_ptr().add(ndx as usize)).gateway;
        }
        ndx = ndx.wrapping_add(1);
    }

    HeapFree(GetProcessHeap(), 0, table as PVOID);
    retVal
}
// ---- 04_CreateIpForwardEntry.rs ----
#[no_mangle]
pub unsafe extern "system" fn CreateIpForwardEntry(pRoute: PMIB_IPFORWARDROW) -> DWORD {
    return createIpForwardEntry(pRoute);
}
// ---- 05_CreateIpNetEntry.rs ----
#[no_mangle]
pub unsafe extern "system" fn CreateIpNetEntry(pArpEntry: PMIB_IPNETROW) -> DWORD {
    TRACE!((b"pArpEntry %p\n\0".as_ptr() as *const i8), pArpEntry);
    FIXME!(":stub\n");
    return 0 as DWORD;
}
// ---- 06_CreateProxyArpEntry.rs ----
#[no_mangle]
pub unsafe extern "system" fn CreateProxyArpEntry(
    dwAddress: DWORD,
    dwMask: DWORD,
    dwIfIndex: DWORD,
) -> DWORD {
    TRACE!(
        (b"dwAddress 0x%08lx, dwMask 0x%08lx, dwIfIndex 0x%08lx\n\0".as_ptr() as *const i8),
        dwAddress,
        dwMask,
        dwIfIndex,
    );
    FIXME!(":stub\n");
    ERROR_NOT_SUPPORTED
}
// ---- 07_DeleteIPAddress.rs ----
#[no_mangle]
pub unsafe extern "system" fn DeleteIPAddress(NTEContext: ULONG) -> DWORD {

    RtlNtStatusToDosError(deleteIpAddress(NTEContext))
}
// ---- 08_DeleteIpForwardEntry.rs ----
#[no_mangle]
pub unsafe extern "system" fn DeleteIpForwardEntry(pRoute: PMIB_IPFORWARDROW) -> DWORD {
    deleteIpForwardEntry(pRoute)
}
// ---- 09_DeleteIpNetEntry.rs ----
#[no_mangle]
pub unsafe extern "system" fn DeleteIpNetEntry(pArpEntry: PMIB_IPNETROW) -> DWORD {
    TRACE!((b"pArpEntry %p\n\0".as_ptr() as *const i8), pArpEntry);
    FIXME!(":stub\n");
    return 0 as DWORD;
}
// ---- 10_DeleteProxyArpEntry.rs ----
#[no_mangle]
pub unsafe extern "system" fn DeleteProxyArpEntry(
    dwAddress: DWORD,
    dwMask: DWORD,
    dwIfIndex: DWORD,
) -> DWORD {
    TRACE!((b"dwAddress 0x%08lx, dwMask 0x%08lx, dwIfIndex 0x%08lx\n\0".as_ptr() as *const i8), dwAddress, dwMask, dwIfIndex);
    FIXME!(":stub\n");
    ERROR_NOT_SUPPORTED
}
// ---- 11_EnableRouter.rs ----
#[no_mangle]
pub unsafe extern "system" fn EnableRouter(
    pHandle: *mut HANDLE,
    pOverlapped: *mut OVERLAPPED,
) -> DWORD {
    TRACE!((b"pHandle %p, pOverlapped %p\n\0".as_ptr() as *const i8), pHandle, pOverlapped);
    FIXME!(":stub\n");
    ERROR_NOT_SUPPORTED
}
// ---- 12_FlushIpNetTable.rs ----
#[no_mangle]
pub unsafe extern "system" fn FlushIpNetTable(dwIfIndex: DWORD) -> DWORD {
    if (*__wine_dbch___default).flags as i32
        & (1i32 << __WINE_DBCL_TRACE as i32)
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"/home/curi/Projects/os-playground/vibeos-research/dll/win32/iphlpapi/iphlpapi_main.c\0"
                as *const u8 as *const i8,
            b"FlushIpNetTable\0" as *const u8 as *const i8,
            649i32,
            b"dwIfIndex 0x%08lx\n\0" as *const u8 as *const i8,
            dwIfIndex,
        );
    }
    if (*__wine_dbch___default).flags as i32
        & (1i32 << __WINE_DBCL_FIXME as i32)
        != 0
    {
        let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl_0: __wine_debug_class = __WINE_DBCL_FIXME;
        ros_dbg_log(
            __dbcl_0,
            __dbch_0,
            b"/home/curi/Projects/os-playground/vibeos-research/dll/win32/iphlpapi/iphlpapi_main.c\0"
                as *const u8 as *const i8,
            b"FlushIpNetTable\0" as *const u8 as *const i8,
            650i32,
            b":stub\n\0" as *const u8 as *const i8,
        );
    }
    ERROR_NOT_SUPPORTED
}
// ---- 13_GetAdapterIndex.rs ----
#[no_mangle]
pub unsafe extern "system" fn GetAdapterIndex(AdapterName: LPWSTR, IfIndex: PULONG) -> DWORD {
    TRACE!((b"AdapterName %p, IfIndex %p\n\0".as_ptr() as *const i8), AdapterName, IfIndex);
    FIXME!(":stub\n");
    ERROR_NOT_SUPPORTED
}
// ---- 14_TcpTableSorter.rs ----
unsafe extern "C" fn TcpTableSorter(
    a: *const core::ffi::c_void,
    b: *const core::ffi::c_void,
) -> core::ffi::c_int {
    let mut ret: core::ffi::c_int;

    if !a.is_null() && !b.is_null() {
        let rowA: PMIB_TCPROW = a as PMIB_TCPROW;
        let rowB: PMIB_TCPROW = b as PMIB_TCPROW;

        ret = (*rowA).dwLocalAddr.wrapping_sub((*rowB).dwLocalAddr) as core::ffi::c_int;
        if ret == 0 {
            ret = (*rowA).dwLocalPort.wrapping_sub((*rowB).dwLocalPort) as core::ffi::c_int;
            if ret == 0 {
                ret = (*rowA).dwRemoteAddr.wrapping_sub((*rowB).dwRemoteAddr) as core::ffi::c_int;
                if ret == 0 {
                    ret = (*rowA).dwRemotePort.wrapping_sub((*rowB).dwRemotePort) as core::ffi::c_int;
                }
            }
        }
    } else {
        ret = 0;
    }
    ret
}
// ---- 15_GetExtendedTcpTable.rs ----
#[no_mangle]
pub unsafe extern "system" fn GetExtendedTcpTable(
    mut pTcpTable: PVOID,
    mut pdwSize: PDWORD,
    mut bOrder: BOOL,
    mut ulAf: ULONG,
    mut TableClass: TCP_TABLE_CLASS,
    mut Reserved: ULONG,
) -> DWORD {
    let mut i: DWORD;
    let mut count: DWORD;
    let mut size: DWORD;
    let mut ret: DWORD = NO_ERROR;

    if pdwSize.is_null() {
        return ERROR_INVALID_PARAMETER;
    }

    if ulAf != AF_INET {
        return ERROR_INVALID_PARAMETER;
    }

    match TableClass {
        TCP_TABLE_BASIC_ALL => {
            let pOurTcpTable: PMIB_TCPTABLE = getTcpTable(ClassBasic) as PMIB_TCPTABLE;
            let pTheirTcpTable: PMIB_TCPTABLE = pTcpTable as PMIB_TCPTABLE;

            if !pOurTcpTable.is_null() {
                size = FIELD_OFFSET!(MIB_TCPTABLE, table).wrapping_add(
                    ((*pOurTcpTable).dwNumEntries as usize)
                        .wrapping_mul(core::mem::size_of::<MIB_TCPROW>()),
                ) as DWORD;
                if size > *pdwSize || pTheirTcpTable.is_null() {
                    *pdwSize = size;
                    ret = ERROR_INSUFFICIENT_BUFFER;
                } else {
                    core::ptr::copy_nonoverlapping(
                        pOurTcpTable as *const u8,
                        pTheirTcpTable as *mut u8,
                        size as usize,
                    );

                    if bOrder != 0 {
                        qsort(
                            core::ptr::addr_of_mut!((*pTheirTcpTable).table) as *mut core::ffi::c_void,
                            (*pTheirTcpTable).dwNumEntries as usize,
                            core::mem::size_of::<MIB_TCPROW>(),
                            Some(TcpTableSorter),
                        );
                    }
                }

                HeapFree(GetProcessHeap(), 0, pOurTcpTable as PVOID);
            }
        }

        TCP_TABLE_BASIC_CONNECTIONS => {
            let pOurTcpTable: PMIB_TCPTABLE = getTcpTable(ClassBasic) as PMIB_TCPTABLE;
            let pTheirTcpTable: PMIB_TCPTABLE = pTcpTable as PMIB_TCPTABLE;

            if !pOurTcpTable.is_null() {
                i = 0;
                count = 0;
                while i < (*pOurTcpTable).dwNumEntries {
                    if (*(core::ptr::addr_of_mut!((*pOurTcpTable).table) as *mut MIB_TCPROW).offset(i as isize)).__bindgen_anon_1.bindgen_union_field != MIB_TCP_STATE_LISTEN {
                        count = count.wrapping_add(1);
                    }
                    i = i.wrapping_add(1);
                }

                size = FIELD_OFFSET!(MIB_TCPTABLE, table).wrapping_add(
                    (count as usize).wrapping_mul(core::mem::size_of::<MIB_TCPROW>()),
                ) as DWORD;
                if size > *pdwSize || pTheirTcpTable.is_null() {
                    *pdwSize = size;
                    ret = ERROR_INSUFFICIENT_BUFFER;
                } else {
                    (*pTheirTcpTable).dwNumEntries = count;

                    i = 0;
                    count = 0;
                    while i < (*pOurTcpTable).dwNumEntries {
                        if (*(core::ptr::addr_of_mut!((*pOurTcpTable).table) as *mut MIB_TCPROW).offset(i as isize)).__bindgen_anon_1.bindgen_union_field != MIB_TCP_STATE_LISTEN {
                            core::ptr::copy_nonoverlapping(
                                (core::ptr::addr_of_mut!((*pOurTcpTable).table) as *mut MIB_TCPROW).offset(i as isize) as *const u8,
                                (core::ptr::addr_of_mut!((*pTheirTcpTable).table) as *mut MIB_TCPROW).offset(count as isize) as *mut u8,
                                core::mem::size_of::<MIB_TCPROW>(),
                            );
                            count = count.wrapping_add(1);
                        }
                        i = i.wrapping_add(1);
                    }

                    if bOrder != 0 {
                        qsort(
                            core::ptr::addr_of_mut!((*pTheirTcpTable).table) as *mut core::ffi::c_void,
                            (*pTheirTcpTable).dwNumEntries as usize,
                            core::mem::size_of::<MIB_TCPROW>(),
                            Some(TcpTableSorter),
                        );
                    }
                }

                HeapFree(GetProcessHeap(), 0, pOurTcpTable as PVOID);
            }
        }

        TCP_TABLE_BASIC_LISTENER => {
            let pOurTcpTable: PMIB_TCPTABLE = getTcpTable(ClassBasic) as PMIB_TCPTABLE;
            let pTheirTcpTable: PMIB_TCPTABLE = pTcpTable as PMIB_TCPTABLE;

            if !pOurTcpTable.is_null() {
                i = 0;
                count = 0;
                while i < (*pOurTcpTable).dwNumEntries {
                    if (*(core::ptr::addr_of_mut!((*pOurTcpTable).table) as *mut MIB_TCPROW).offset(i as isize)).__bindgen_anon_1.bindgen_union_field == MIB_TCP_STATE_LISTEN {
                        count = count.wrapping_add(1);
                    }
                    i = i.wrapping_add(1);
                }

                size = FIELD_OFFSET!(MIB_TCPTABLE, table).wrapping_add(
                    (count as usize).wrapping_mul(core::mem::size_of::<MIB_TCPROW>()),
                ) as DWORD;
                if size > *pdwSize || pTheirTcpTable.is_null() {
                    *pdwSize = size;
                    ret = ERROR_INSUFFICIENT_BUFFER;
                } else {
                    (*pTheirTcpTable).dwNumEntries = count;

                    i = 0;
                    count = 0;
                    while i < (*pOurTcpTable).dwNumEntries {
                        if (*(core::ptr::addr_of_mut!((*pOurTcpTable).table) as *mut MIB_TCPROW).offset(i as isize)).__bindgen_anon_1.bindgen_union_field == MIB_TCP_STATE_LISTEN {
                            core::ptr::copy_nonoverlapping(
                                (core::ptr::addr_of_mut!((*pOurTcpTable).table) as *mut MIB_TCPROW).offset(i as isize) as *const u8,
                                (core::ptr::addr_of_mut!((*pTheirTcpTable).table) as *mut MIB_TCPROW).offset(count as isize) as *mut u8,
                                core::mem::size_of::<MIB_TCPROW>(),
                            );
                            count = count.wrapping_add(1);
                        }
                        i = i.wrapping_add(1);
                    }

                    if bOrder != 0 {
                        qsort(
                            core::ptr::addr_of_mut!((*pTheirTcpTable).table) as *mut core::ffi::c_void,
                            (*pTheirTcpTable).dwNumEntries as usize,
                            core::mem::size_of::<MIB_TCPROW>(),
                            Some(TcpTableSorter),
                        );
                    }
                }

                HeapFree(GetProcessHeap(), 0, pOurTcpTable as PVOID);
            }
        }

        TCP_TABLE_OWNER_PID_ALL => {
            let pOurTcpTable: PMIB_TCPTABLE_OWNER_PID = getTcpTable(ClassModulePid) as PMIB_TCPTABLE_OWNER_PID;
            let pTheirTcpTable: PMIB_TCPTABLE_OWNER_PID = pTcpTable as PMIB_TCPTABLE_OWNER_PID;

            if !pOurTcpTable.is_null() {
                size = FIELD_OFFSET!(MIB_TCPTABLE_OWNER_PID, table).wrapping_add(
                    ((*pOurTcpTable).dwNumEntries as usize)
                        .wrapping_mul(core::mem::size_of::<MIB_TCPROW_OWNER_PID>()),
                ) as DWORD;
                if size > *pdwSize || pTheirTcpTable.is_null() {
                    *pdwSize = size;
                    ret = ERROR_INSUFFICIENT_BUFFER;
                } else {
                    core::ptr::copy_nonoverlapping(
                        pOurTcpTable as *const u8,
                        pTheirTcpTable as *mut u8,
                        size as usize,
                    );

                    if bOrder != 0 {
                        qsort(
                            core::ptr::addr_of_mut!((*pTheirTcpTable).table) as *mut core::ffi::c_void,
                            (*pTheirTcpTable).dwNumEntries as usize,
                            core::mem::size_of::<MIB_TCPROW_OWNER_PID>(),
                            Some(TcpTableSorter),
                        );
                    }
                }

                HeapFree(GetProcessHeap(), 0, pOurTcpTable as PVOID);
            }
        }

        TCP_TABLE_OWNER_PID_CONNECTIONS => {
            let pOurTcpTable: PMIB_TCPTABLE_OWNER_PID = getTcpTable(ClassModulePid) as PMIB_TCPTABLE_OWNER_PID;
            let pTheirTcpTable: PMIB_TCPTABLE_OWNER_PID = pTcpTable as PMIB_TCPTABLE_OWNER_PID;

            if !pOurTcpTable.is_null() {
                i = 0;
                count = 0;
                while i < (*pOurTcpTable).dwNumEntries {
                    if (*(core::ptr::addr_of_mut!((*pOurTcpTable).table) as *mut MIB_TCPROW_OWNER_PID).offset(i as isize)).dwState != MIB_TCP_STATE_LISTEN {
                        count = count.wrapping_add(1);
                    }
                    i = i.wrapping_add(1);
                }

                size = FIELD_OFFSET!(MIB_TCPTABLE_OWNER_PID, table).wrapping_add(
                    (count as usize).wrapping_mul(core::mem::size_of::<MIB_TCPROW_OWNER_PID>()),
                ) as DWORD;
                if size > *pdwSize || pTheirTcpTable.is_null() {
                    *pdwSize = size;
                    ret = ERROR_INSUFFICIENT_BUFFER;
                } else {
                    (*pTheirTcpTable).dwNumEntries = count;

                    i = 0;
                    count = 0;
                    while i < (*pOurTcpTable).dwNumEntries {
                        if (*(core::ptr::addr_of_mut!((*pOurTcpTable).table) as *mut MIB_TCPROW_OWNER_PID).offset(i as isize)).dwState != MIB_TCP_STATE_LISTEN {
                            core::ptr::copy_nonoverlapping(
                                (core::ptr::addr_of_mut!((*pOurTcpTable).table) as *mut MIB_TCPROW_OWNER_PID).offset(i as isize) as *const u8,
                                (core::ptr::addr_of_mut!((*pTheirTcpTable).table) as *mut MIB_TCPROW_OWNER_PID).offset(count as isize) as *mut u8,
                                core::mem::size_of::<MIB_TCPROW_OWNER_PID>(),
                            );
                            count = count.wrapping_add(1);
                        }
                        i = i.wrapping_add(1);
                    }

                    if bOrder != 0 {
                        qsort(
                            core::ptr::addr_of_mut!((*pTheirTcpTable).table) as *mut core::ffi::c_void,
                            (*pTheirTcpTable).dwNumEntries as usize,
                            core::mem::size_of::<MIB_TCPROW_OWNER_PID>(),
                            Some(TcpTableSorter),
                        );
                    }
                }

                HeapFree(GetProcessHeap(), 0, pOurTcpTable as PVOID);
            }
        }

        TCP_TABLE_OWNER_PID_LISTENER => {
            let pOurTcpTable: PMIB_TCPTABLE_OWNER_PID = getTcpTable(ClassModulePid) as PMIB_TCPTABLE_OWNER_PID;
            let pTheirTcpTable: PMIB_TCPTABLE_OWNER_PID = pTcpTable as PMIB_TCPTABLE_OWNER_PID;

            if !pOurTcpTable.is_null() {
                i = 0;
                count = 0;
                while i < (*pOurTcpTable).dwNumEntries {
                    if (*(core::ptr::addr_of_mut!((*pOurTcpTable).table) as *mut MIB_TCPROW_OWNER_PID).offset(i as isize)).dwState == MIB_TCP_STATE_LISTEN {
                        count = count.wrapping_add(1);
                    }
                    i = i.wrapping_add(1);
                }

                size = FIELD_OFFSET!(MIB_TCPTABLE_OWNER_PID, table).wrapping_add(
                    (count as usize).wrapping_mul(core::mem::size_of::<MIB_TCPROW_OWNER_PID>()),
                ) as DWORD;
                if size > *pdwSize || pTheirTcpTable.is_null() {
                    *pdwSize = size;
                    ret = ERROR_INSUFFICIENT_BUFFER;
                } else {
                    (*pTheirTcpTable).dwNumEntries = count;

                    i = 0;
                    count = 0;
                    while i < (*pOurTcpTable).dwNumEntries {
                        if (*(core::ptr::addr_of_mut!((*pOurTcpTable).table) as *mut MIB_TCPROW_OWNER_PID).offset(i as isize)).dwState == MIB_TCP_STATE_LISTEN {
                            core::ptr::copy_nonoverlapping(
                                (core::ptr::addr_of_mut!((*pOurTcpTable).table) as *mut MIB_TCPROW_OWNER_PID).offset(i as isize) as *const u8,
                                (core::ptr::addr_of_mut!((*pTheirTcpTable).table) as *mut MIB_TCPROW_OWNER_PID).offset(count as isize) as *mut u8,
                                core::mem::size_of::<MIB_TCPROW_OWNER_PID>(),
                            );
                            count = count.wrapping_add(1);
                        }
                        i = i.wrapping_add(1);
                    }

                    if bOrder != 0 {
                        qsort(
                            core::ptr::addr_of_mut!((*pTheirTcpTable).table) as *mut core::ffi::c_void,
                            (*pTheirTcpTable).dwNumEntries as usize,
                            core::mem::size_of::<MIB_TCPROW_OWNER_PID>(),
                            Some(TcpTableSorter),
                        );
                    }
                }

                HeapFree(GetProcessHeap(), 0, pOurTcpTable as PVOID);
            }
        }

        TCP_TABLE_OWNER_MODULE_ALL => {
            let pOurTcpTable: PMIB_TCPTABLE_OWNER_MODULE = getTcpTable(ClassModule) as PMIB_TCPTABLE_OWNER_MODULE;
            let pTheirTcpTable: PMIB_TCPTABLE_OWNER_MODULE = pTcpTable as PMIB_TCPTABLE_OWNER_MODULE;

            if !pOurTcpTable.is_null() {
                size = FIELD_OFFSET!(MIB_TCPTABLE_OWNER_MODULE, table).wrapping_add(
                    ((*pOurTcpTable).dwNumEntries as usize)
                        .wrapping_mul(core::mem::size_of::<MIB_TCPROW_OWNER_MODULE>()),
                ) as DWORD;
                if size > *pdwSize || pTheirTcpTable.is_null() {
                    *pdwSize = size;
                    ret = ERROR_INSUFFICIENT_BUFFER;
                } else {
                    core::ptr::copy_nonoverlapping(
                        pOurTcpTable as *const u8,
                        pTheirTcpTable as *mut u8,
                        size as usize,
                    );

                    if bOrder != 0 {
                        qsort(
                            core::ptr::addr_of_mut!((*pTheirTcpTable).table) as *mut core::ffi::c_void,
                            (*pTheirTcpTable).dwNumEntries as usize,
                            core::mem::size_of::<MIB_TCPROW_OWNER_MODULE>(),
                            Some(TcpTableSorter),
                        );
                    }
                }

                HeapFree(GetProcessHeap(), 0, pOurTcpTable as PVOID);
            }
        }

        TCP_TABLE_OWNER_MODULE_CONNECTIONS => {
            let pOurTcpTable: PMIB_TCPTABLE_OWNER_MODULE = getTcpTable(ClassModule) as PMIB_TCPTABLE_OWNER_MODULE;
            let pTheirTcpTable: PMIB_TCPTABLE_OWNER_MODULE = pTcpTable as PMIB_TCPTABLE_OWNER_MODULE;

            if !pOurTcpTable.is_null() {
                i = 0;
                count = 0;
                while i < (*pOurTcpTable).dwNumEntries {
                    if (*(core::ptr::addr_of_mut!((*pOurTcpTable).table) as *mut MIB_TCPROW_OWNER_MODULE).offset(i as isize)).dwState != MIB_TCP_STATE_LISTEN {
                        count = count.wrapping_add(1);
                    }
                    i = i.wrapping_add(1);
                }

                size = FIELD_OFFSET!(MIB_TCPTABLE_OWNER_MODULE, table).wrapping_add(
                    (count as usize).wrapping_mul(core::mem::size_of::<MIB_TCPROW_OWNER_MODULE>()),
                ) as DWORD;
                if size > *pdwSize || pTheirTcpTable.is_null() {
                    *pdwSize = size;
                    ret = ERROR_INSUFFICIENT_BUFFER;
                } else {
                    (*pTheirTcpTable).dwNumEntries = count;

                    i = 0;
                    count = 0;
                    while i < (*pOurTcpTable).dwNumEntries {
                        if (*(core::ptr::addr_of_mut!((*pOurTcpTable).table) as *mut MIB_TCPROW_OWNER_MODULE).offset(i as isize)).dwState != MIB_TCP_STATE_LISTEN {
                            core::ptr::copy_nonoverlapping(
                                (core::ptr::addr_of_mut!((*pOurTcpTable).table) as *mut MIB_TCPROW_OWNER_MODULE).offset(i as isize) as *const u8,
                                (core::ptr::addr_of_mut!((*pTheirTcpTable).table) as *mut MIB_TCPROW_OWNER_MODULE).offset(count as isize) as *mut u8,
                                core::mem::size_of::<MIB_TCPROW_OWNER_MODULE>(),
                            );
                            count = count.wrapping_add(1);
                        }
                        i = i.wrapping_add(1);
                    }

                    if bOrder != 0 {
                        qsort(
                            core::ptr::addr_of_mut!((*pTheirTcpTable).table) as *mut core::ffi::c_void,
                            (*pTheirTcpTable).dwNumEntries as usize,
                            core::mem::size_of::<MIB_TCPROW_OWNER_MODULE>(),
                            Some(TcpTableSorter),
                        );
                    }
                }

                HeapFree(GetProcessHeap(), 0, pOurTcpTable as PVOID);
            }
        }

        TCP_TABLE_OWNER_MODULE_LISTENER => {
            let pOurTcpTable: PMIB_TCPTABLE_OWNER_MODULE = getTcpTable(ClassModule) as PMIB_TCPTABLE_OWNER_MODULE;
            let pTheirTcpTable: PMIB_TCPTABLE_OWNER_MODULE = pTcpTable as PMIB_TCPTABLE_OWNER_MODULE;

            if !pOurTcpTable.is_null() {
                i = 0;
                count = 0;
                while i < (*pOurTcpTable).dwNumEntries {
                    if (*(core::ptr::addr_of_mut!((*pOurTcpTable).table) as *mut MIB_TCPROW_OWNER_MODULE).offset(i as isize)).dwState == MIB_TCP_STATE_LISTEN {
                        count = count.wrapping_add(1);
                    }
                    i = i.wrapping_add(1);
                }

                size = FIELD_OFFSET!(MIB_TCPTABLE_OWNER_MODULE, table).wrapping_add(
                    (count as usize).wrapping_mul(core::mem::size_of::<MIB_TCPROW_OWNER_MODULE>()),
                ) as DWORD;
                if size > *pdwSize || pTheirTcpTable.is_null() {
                    *pdwSize = size;
                    ret = ERROR_INSUFFICIENT_BUFFER;
                } else {
                    (*pTheirTcpTable).dwNumEntries = count;

                    i = 0;
                    count = 0;
                    while i < (*pOurTcpTable).dwNumEntries {
                        if (*(core::ptr::addr_of_mut!((*pOurTcpTable).table) as *mut MIB_TCPROW_OWNER_MODULE).offset(i as isize)).dwState == MIB_TCP_STATE_LISTEN {
                            core::ptr::copy_nonoverlapping(
                                (core::ptr::addr_of_mut!((*pOurTcpTable).table) as *mut MIB_TCPROW_OWNER_MODULE).offset(i as isize) as *const u8,
                                (core::ptr::addr_of_mut!((*pTheirTcpTable).table) as *mut MIB_TCPROW_OWNER_MODULE).offset(count as isize) as *mut u8,
                                core::mem::size_of::<MIB_TCPROW_OWNER_MODULE>(),
                            );
                            count = count.wrapping_add(1);
                        }
                        i = i.wrapping_add(1);
                    }

                    if bOrder != 0 {
                        qsort(
                            core::ptr::addr_of_mut!((*pTheirTcpTable).table) as *mut core::ffi::c_void,
                            (*pTheirTcpTable).dwNumEntries as usize,
                            core::mem::size_of::<MIB_TCPROW_OWNER_MODULE>(),
                            Some(TcpTableSorter),
                        );
                    }
                }

                HeapFree(GetProcessHeap(), 0, pOurTcpTable as PVOID);
            }
        }

        _ => {
            ret = ERROR_INVALID_PARAMETER;
        }
    }

    ret
}

// ---- 16_UdpTableSorter.rs ----
unsafe extern "C" fn UdpTableSorter(
    a: *const core::ffi::c_void,
    b: *const core::ffi::c_void,
) -> core::ffi::c_int {
    let mut ret: core::ffi::c_int;

    if !a.is_null() && !b.is_null() {
        let rowA: PMIB_UDPROW = a as PMIB_UDPROW;
        let rowB: PMIB_UDPROW = b as PMIB_UDPROW;

        ret = (*rowA).dwLocalAddr.wrapping_sub((*rowB).dwLocalAddr) as core::ffi::c_int;
        if ret == 0 {
            ret = (*rowA).dwLocalPort.wrapping_sub((*rowB).dwLocalPort) as core::ffi::c_int;
        }
    } else {
        ret = 0;
    }
    ret
}
// ---- 17_GetExtendedUdpTable.rs ----
#[no_mangle]
pub unsafe extern "system" fn GetExtendedUdpTable(
    pUdpTable: LPVOID,
    pdwSize: PDWORD,
    bOrder: u8,
    ulAf: ULONG,
    TableClass: UDP_TABLE_CLASS,
    Reserved: ULONG,
) -> DWORD {

    let mut size: DWORD;
    let mut ret: DWORD = NO_ERROR;

    if pdwSize.is_null() {
        return ERROR_INVALID_PARAMETER;
    }

    if ulAf != AF_INET as ULONG {
        return ERROR_INVALID_PARAMETER;
    }

    match TableClass {
        UDP_TABLE_BASIC => {
            let pOurUdpTable = getUdpTable((ClassBasic as ::core::ffi::c_int as usize)) as PMIB_UDPTABLE;
            let pTheirUdpTable = pUdpTable as PMIB_UDPTABLE;

            if !pOurUdpTable.is_null() {
                size = (core::mem::offset_of!(_MIB_UDPTABLE, table) +
                    (*pOurUdpTable).dwNumEntries as usize * core::mem::size_of::<MIB_UDPROW>()) as DWORD;
                if size > *pdwSize || pTheirUdpTable.is_null() {
                    *pdwSize = size;
                    ret = ERROR_INSUFFICIENT_BUFFER;
                } else {
                    memcpy(
                        pTheirUdpTable as LPVOID,
                        pOurUdpTable as *const ::core::ffi::c_void,
                        size as usize,
                    );

                    if bOrder != 0 {
                        qsort(
                            core::ptr::addr_of_mut!((*pTheirUdpTable).table) as LPVOID,
                            (*pTheirUdpTable).dwNumEntries as usize,
                            core::mem::size_of::<MIB_UDPROW>(),
                            Some(UdpTableSorter),
                        );
                    }
                }

                HeapFree(GetProcessHeap(), 0, pOurUdpTable as LPVOID);
            }
        }

        UDP_TABLE_OWNER_PID => {
            let pOurUdpTable = getUdpTable((ClassModulePid as ::core::ffi::c_int as usize)) as PMIB_UDPTABLE_OWNER_PID;
            let pTheirUdpTable = pUdpTable as PMIB_UDPTABLE_OWNER_PID;

            if !pOurUdpTable.is_null() {
                size = (core::mem::offset_of!(_MIB_UDPTABLE_OWNER_PID, table) +
                    (*pOurUdpTable).dwNumEntries as usize * core::mem::size_of::<MIB_UDPROW_OWNER_PID>()) as DWORD;
                if size > *pdwSize || pTheirUdpTable.is_null() {
                    *pdwSize = size;
                    ret = ERROR_INSUFFICIENT_BUFFER;
                } else {
                    memcpy(
                        pTheirUdpTable as LPVOID,
                        pOurUdpTable as *const ::core::ffi::c_void,
                        size as usize,
                    );

                    if bOrder != 0 {
                        qsort(
                            core::ptr::addr_of_mut!((*pTheirUdpTable).table) as LPVOID,
                            (*pTheirUdpTable).dwNumEntries as usize,
                            core::mem::size_of::<MIB_UDPROW_OWNER_PID>(),
                            Some(UdpTableSorter),
                        );
                    }
                }

                HeapFree(GetProcessHeap(), 0, pOurUdpTable as LPVOID);
            }
        }

        UDP_TABLE_OWNER_MODULE => {
            let pOurUdpTable = getUdpTable((ClassModule as ::core::ffi::c_int as usize)) as PMIB_UDPTABLE_OWNER_MODULE;
            let pTheirUdpTable = pUdpTable as PMIB_UDPTABLE_OWNER_MODULE;

            if !pOurUdpTable.is_null() {
                size = (core::mem::offset_of!(_MIB_UDPTABLE_OWNER_MODULE, table) +
                    (*pOurUdpTable).dwNumEntries as usize * core::mem::size_of::<MIB_UDPROW_OWNER_MODULE>()) as DWORD;
                if size > *pdwSize || pTheirUdpTable.is_null() {
                    *pdwSize = size;
                    ret = ERROR_INSUFFICIENT_BUFFER;
                } else {
                    memcpy(
                        pTheirUdpTable as LPVOID,
                        pOurUdpTable as *const ::core::ffi::c_void,
                        size as usize,
                    );

                    if bOrder != 0 {
                        qsort(
                            core::ptr::addr_of_mut!((*pTheirUdpTable).table) as LPVOID,
                            (*pTheirUdpTable).dwNumEntries as usize,
                            core::mem::size_of::<MIB_UDPROW_OWNER_MODULE>(),
                            Some(UdpTableSorter),
                        );
                    }
                }

                HeapFree(GetProcessHeap(), 0, pOurUdpTable as LPVOID);
            }
        }

        _ => {
            ret = ERROR_INVALID_PARAMETER;
        }
    }

    ret
}
// ---- 18_GetFriendlyIfIndex.rs ----
#[no_mangle]
pub unsafe extern "system" fn GetFriendlyIfIndex(IfIndex: DWORD) -> DWORD {
    TRACE!((b"returning %ld\n\0".as_ptr() as *const i8), IfIndex);
    IfIndex
}
// ---- 19_GetIcmpStatistics.rs ----
#[no_mangle]
pub unsafe extern "system" fn GetIcmpStatistics(pStats: PMIB_ICMP) -> DWORD {

    let ret: DWORD;

    TRACE!((b"pStats %p\n\0".as_ptr() as *const i8), pStats);
    ret = getICMPStats(pStats);
    TRACE!((b"returning %ld\n\0".as_ptr() as *const i8), ret);
    ret
}
// ---- 20_GetIfEntry.rs ----
#[no_mangle]
pub unsafe extern "system" fn GetIfEntry(pIfRow: PMIB_IFROW) -> DWORD {

    let mut ret: DWORD;
    let name: *const core::ffi::c_char;

    TRACE!((b"pIfRow %p\n\0".as_ptr() as *const i8), pIfRow);
    if pIfRow.is_null() {
        return ERROR_INVALID_PARAMETER;
    }

    name = getInterfaceNameByIndex((*pIfRow).dwIndex);
    if !name.is_null() {
        ret = getInterfaceEntryByIndex((*pIfRow).dwIndex, pIfRow);
        if ret == NO_ERROR {
            ret = getInterfaceStatsByName((name as *const u8), pIfRow);
        }
        consumeInterfaceName((name as *const u8));
    } else {
        ret = ERROR_INVALID_DATA;
    }
    TRACE!((b"returning %ld\n\0".as_ptr() as *const i8), ret);
    ret
}
// ---- 21_IfTableSorter.rs ----
unsafe extern "C" fn IfTableSorter(
    a: *const core::ffi::c_void,
    b: *const core::ffi::c_void,
) -> i32 {
    let ret: i32;

    if !a.is_null() && !b.is_null() {
        ret = (*(a as PMIB_IFROW))
            .dwIndex
            .wrapping_sub((*(b as PMIB_IFROW)).dwIndex) as i32;
    } else {
        ret = 0;
    }
    ret
}
// ---- 22_GetInterfaceInfo.rs ----





#[no_mangle]
pub unsafe extern "system" fn GetInterfaceInfo(
    mut pIfTable: PIP_INTERFACE_INFO,
    mut dwOutBufLen: PULONG,
) -> DWORD {
    let mut ret: DWORD;

    TRACE!((b"pIfTable %p, dwOutBufLen %p\n\0".as_ptr() as *const i8), pIfTable, dwOutBufLen);
    if dwOutBufLen.is_null() {
        ret = ERROR_INVALID_PARAMETER;
    } else {
        let numNonLoopbackInterfaces: DWORD = getNumNonLoopbackInterfaces();
        let mut size: u32;
        TRACE!((b"numNonLoopbackInterfaces == 0x%x\n\0".as_ptr() as *const i8), numNonLoopbackInterfaces);
        size = (core::mem::size_of::<IP_INTERFACE_INFO>() as usize)
            .wrapping_add((numNonLoopbackInterfaces as usize)
                .wrapping_mul(core::mem::size_of::<IP_ADAPTER_INDEX_MAP>() as usize)) as u32;

        if pIfTable.is_null() || *dwOutBufLen < size {
            *dwOutBufLen = size;
            ret = ERROR_INSUFFICIENT_BUFFER;
        } else {
            let table: *mut InterfaceIndexTable = getNonLoopbackInterfaceIndexTable();

            if !table.is_null() {
                TRACE!((b"table->numIndexes == 0x%x\n\0".as_ptr() as *const i8), (*table).numIndexes);
                size = (core::mem::size_of::<IP_INTERFACE_INFO>() as usize)
                    .wrapping_add(((*table).numIndexes as usize)
                        .wrapping_mul(core::mem::size_of::<IP_ADAPTER_INDEX_MAP>() as usize)) as u32;
                if *dwOutBufLen < size {
                    *dwOutBufLen = size;
                    ret = ERROR_INSUFFICIENT_BUFFER;
                } else {
                    let mut ndx: DWORD;

                    (*pIfTable).NumAdapters = 0;
                    ndx = 0;
                    while ndx < (*table).numIndexes {
                        let mut walker: *const i8;
                        let name: *const i8;
                        let mut assigner: *mut u16;

                        (core::ptr::addr_of_mut!((*pIfTable).Adapter) as *mut IP_ADAPTER_INDEX_MAP)
                            .offset(ndx as isize)
                            .write_unaligned(IP_ADAPTER_INDEX_MAP {
                                Index: *((core::ptr::addr_of_mut!((*table).indexes) as *mut DWORD)
                                    .offset(ndx as isize)) as u32,
                                Name: [0; 128],
                            });
                        name = getInterfaceNameByIndex(
                            *((core::ptr::addr_of_mut!((*table).indexes) as *mut DWORD)
                                .offset(ndx as isize)),
                        );
                        wcscpy(
                            core::ptr::addr_of_mut!(
                                (*(core::ptr::addr_of_mut!((*pIfTable).Adapter) as *mut IP_ADAPTER_INDEX_MAP)
                                    .offset(ndx as isize))
                                .Name
                            ) as *mut u16,
                            win32macros_rust::wide_bytes!(b"\\\0D\0E\0V\0I\0C\0E\0\\\0T\0C\0P\0I\0P\0_\0\0\0") as *const u16,
                        );
                        walker = name;
                        assigner = (core::ptr::addr_of_mut!(
                            (*(core::ptr::addr_of_mut!((*pIfTable).Adapter) as *mut IP_ADAPTER_INDEX_MAP)
                                .offset(ndx as isize))
                            .Name
                        ) as *mut u16)
                            .offset(14);
                        while !walker.is_null()
                            && *walker as i32 != 0
                            && assigner.offset_from(
                                core::ptr::addr_of_mut!(
                                    (*(core::ptr::addr_of_mut!((*pIfTable).Adapter) as *mut IP_ADAPTER_INDEX_MAP)
                                        .offset(ndx as isize))
                                    .Name
                                ) as *mut u16
                            ) < (128 - 1 - 14) as isize
                        {
                            *assigner = *walker as u16;
                            walker = walker.offset(1);
                            assigner = assigner.offset(1);
                        }
                        *assigner = 0;
                        consumeInterfaceName((name as *const u8));
                        (*pIfTable).NumAdapters += 1;
                        ndx = ndx.wrapping_add(1);
                    }
                    ret = NO_ERROR;
                }
                free(table as *mut core::ffi::c_void);
            } else {
                ret = ERROR_OUTOFMEMORY;
            }
        }
    }
    TRACE!((b"returning %ld\n\0".as_ptr() as *const i8), ret);
    ret
}
// ---- 23_IpAddrTableSorter.rs ----
unsafe extern "C" fn IpAddrTableSorter(
    a: *const core::ffi::c_void,
    b: *const core::ffi::c_void,
) -> core::ffi::c_int {
    let mut ret: core::ffi::c_int;
    if !a.is_null() && !b.is_null() {
        ret = (*(a as PMIB_IPADDRROW))
            .dwAddr
            .wrapping_sub((*(b as PMIB_IPADDRROW)).dwAddr) as core::ffi::c_int;
    } else {
        ret = 0;
    }
    ret
}
// ---- 24_GetIpAddrTable.rs ----

#[no_mangle]
pub unsafe extern "system" fn GetIpAddrTable(
    pIpAddrTable: PMIB_IPADDRTABLE,
    pdwSize: PULONG,
    bOrder: BOOL,
) -> DWORD {
    let mut ret: DWORD;

    TRACE!(
        (b"pIpAddrTable %p, pdwSize %p, bOrder %ld\n\0".as_ptr() as *const i8),
        pIpAddrTable,
        pdwSize,
        bOrder as DWORD,
    );

    if pdwSize.is_null() {
        ret = ERROR_INVALID_PARAMETER;
    } else {
        let numInterfaces: DWORD = getNumInterfaces();
        let mut size: ULONG = (core::mem::size_of::<MIB_IPADDRTABLE>() as usize).wrapping_add(
            ((numInterfaces as usize).wrapping_sub(1)).wrapping_mul(core::mem::size_of::<MIB_IPADDRROW>() as usize),
        ) as ULONG;

        if pIpAddrTable.is_null() || *pdwSize < size {
            *pdwSize = size;
            ret = ERROR_INSUFFICIENT_BUFFER;
        } else {
            let table: *mut InterfaceIndexTable = getInterfaceIndexTable();

            if !table.is_null() {
                size = (core::mem::size_of::<MIB_IPADDRTABLE>() as usize).wrapping_add(
                    (((*table).numIndexes as usize).wrapping_sub(1))
                        .wrapping_mul(core::mem::size_of::<MIB_IPADDRROW>() as usize),
                ) as ULONG;

                if *pdwSize < size {
                    *pdwSize = size;
                    ret = ERROR_INSUFFICIENT_BUFFER;
                } else {
                    let mut ndx: DWORD;
                    let mut bcast: DWORD;

                    (*pIpAddrTable).dwNumEntries = 0;
                    ndx = 0;
                    while ndx < (*table).numIndexes {
                        (*(*pIpAddrTable).table.as_mut_ptr().add(ndx as usize)).dwIndex = (*(*table).indexes.as_ptr().add(ndx as usize)) as IF_INDEX;
                        (*(*pIpAddrTable).table.as_mut_ptr().add(ndx as usize)).dwAddr =
                            getInterfaceIPAddrByIndex((*(*table).indexes.as_ptr().add(ndx as usize)));
                        (*(*pIpAddrTable).table.as_mut_ptr().add(ndx as usize)).dwMask =
                            getInterfaceMaskByIndex((*(*table).indexes.as_ptr().add(ndx as usize)));
                        bcast = getInterfaceBCastAddrByIndex((*(*table).indexes.as_ptr().add(ndx as usize)));
                        (*(*pIpAddrTable).table.as_mut_ptr().add(ndx as usize)).dwBCastAddr =
                            if (bcast & (*(*pIpAddrTable).table.as_mut_ptr().add(ndx as usize)).dwMask) != 0 { 1 } else { 0 };
                        (*(*pIpAddrTable).table.as_mut_ptr().add(ndx as usize)).dwReasmSize = 65535;
                        (*(*pIpAddrTable).table.as_mut_ptr().add(ndx as usize)).unused1 = 0;
                        (*(*pIpAddrTable).table.as_mut_ptr().add(ndx as usize)).wType = 0;
                        (*pIpAddrTable).dwNumEntries = (*pIpAddrTable).dwNumEntries.wrapping_add(1);
                        ndx = ndx.wrapping_add(1);
                    }
                    if bOrder != 0 {
                        qsort(
                            core::ptr::addr_of_mut!((*pIpAddrTable).table) as *mut core::ffi::c_void,
                            (*pIpAddrTable).dwNumEntries as usize,
                            core::mem::size_of::<MIB_IPADDRROW>(),
                            Some(IpAddrTableSorter),
                        );
                    }
                    ret = NO_ERROR;
                }
                free(table as *mut core::ffi::c_void);
            } else {
                ret = ERROR_OUTOFMEMORY;
            }
        }
    }

    TRACE!((b"returning %ld\n\0".as_ptr() as *const i8), ret);
    ret
}
// ---- 25_IpForwardTableSorter.rs ----
unsafe extern "C" fn IpForwardTableSorter(a: *const core::ffi::c_void, b: *const core::ffi::c_void) -> core::ffi::c_int {
    let mut ret: core::ffi::c_int;

    if !a.is_null() && !b.is_null() {
        let rowA: PMIB_IPFORWARDROW = a as PMIB_IPFORWARDROW;
        let rowB: PMIB_IPFORWARDROW = b as PMIB_IPFORWARDROW;

        ret = (*rowA).dwForwardDest.wrapping_sub((*rowB).dwForwardDest) as core::ffi::c_int;
        if ret == 0 {
            ret = (*rowA).__bindgen_anon_2.bindgen_union_field.wrapping_sub((*rowB).__bindgen_anon_2.bindgen_union_field) as core::ffi::c_int;
            if ret == 0 {
                ret = (*rowA).dwForwardPolicy.wrapping_sub((*rowB).dwForwardPolicy) as core::ffi::c_int;
                if ret == 0 {
                    ret = (*rowA).dwForwardNextHop.wrapping_sub((*rowB).dwForwardNextHop) as core::ffi::c_int;
                }
            }
        }
    } else {
        ret = 0;
    }
    ret
}
// ---- 26_GetIpForwardTable.rs ----
#[no_mangle]
pub unsafe extern "system" fn GetIpForwardTable(
    pIpForwardTable: PMIB_IPFORWARDTABLE,
    pdwSize: PULONG,
    bOrder: BOOL,
) -> DWORD {
    let mut ret: DWORD;

    TRACE!(
        (b"pIpForwardTable %p, pdwSize %p, bOrder %ld\n\0".as_ptr() as *const i8),
        pIpForwardTable,
        pdwSize,
        bOrder as DWORD,
    );

    if pdwSize.is_null() {
        ret = ERROR_INVALID_PARAMETER;
    } else {
        let numRoutes: DWORD = getNumRoutes();
        let mut sizeNeeded: ULONG = (core::mem::size_of::<MIB_IPFORWARDTABLE>() as usize)
            .wrapping_add(
                ((numRoutes as usize).wrapping_sub(1))
                    .wrapping_mul(core::mem::size_of::<MIB_IPFORWARDROW>()),
            ) as ULONG;

        if pIpForwardTable.is_null() || *pdwSize < sizeNeeded {
            *pdwSize = sizeNeeded;
            ret = ERROR_INSUFFICIENT_BUFFER;
        } else {
            let table: *mut RouteTable = getRouteTable();
            if !table.is_null() {
                sizeNeeded = (core::mem::size_of::<MIB_IPFORWARDTABLE>() as usize).wrapping_add(
                    (((*table).numRoutes as usize).wrapping_sub(1))
                        .wrapping_mul(core::mem::size_of::<MIB_IPFORWARDROW>()),
                ) as ULONG;

                if *pdwSize < sizeNeeded {
                    *pdwSize = sizeNeeded;
                    ret = ERROR_INSUFFICIENT_BUFFER;
                } else {
                    (*pIpForwardTable).dwNumEntries = (*table).numRoutes;
                    let mut ndx: DWORD = 0;
                    while ndx < numRoutes {
                        (*(*pIpForwardTable).table.as_mut_ptr().add(ndx as usize)).dwForwardIfIndex =
                            (*(*table).routes.as_ptr().add(ndx as usize)).ifIndex as IF_INDEX;
                        (*(*pIpForwardTable).table.as_mut_ptr().add(ndx as usize)).dwForwardDest =
                            (*(*table).routes.as_ptr().add(ndx as usize)).dest;
                        (*(*pIpForwardTable).table.as_mut_ptr().add(ndx as usize)).dwForwardMask =
                            (*(*table).routes.as_ptr().add(ndx as usize)).mask;
                        (*(*pIpForwardTable).table.as_mut_ptr().add(ndx as usize)).dwForwardPolicy = 0;
                        (*(*pIpForwardTable).table.as_mut_ptr().add(ndx as usize)).dwForwardNextHop =
                            (*(*table).routes.as_ptr().add(ndx as usize)).gateway;
                        (*(*pIpForwardTable).table.as_mut_ptr().add(ndx as usize))
                            .__bindgen_anon_1
                            .bindgen_union_field = MIB_IPROUTE_TYPE_DIRECT as DWORD;
                        (*(*pIpForwardTable).table.as_mut_ptr().add(ndx as usize))
                            .__bindgen_anon_2
                            .bindgen_union_field = MIB_IPPROTO_LOCAL as DWORD;
                        (*(*pIpForwardTable).table.as_mut_ptr().add(ndx as usize)).dwForwardAge = 0;
                        (*(*pIpForwardTable).table.as_mut_ptr().add(ndx as usize)).dwForwardNextHopAS = 0;
                        (*(*pIpForwardTable).table.as_mut_ptr().add(ndx as usize)).dwForwardMetric1 =
                            (*(*table).routes.as_ptr().add(ndx as usize)).metric;
                        (*(*pIpForwardTable).table.as_mut_ptr().add(ndx as usize)).dwForwardMetric2 = 0;
                        (*(*pIpForwardTable).table.as_mut_ptr().add(ndx as usize)).dwForwardMetric3 = 0;
                        (*(*pIpForwardTable).table.as_mut_ptr().add(ndx as usize)).dwForwardMetric4 = 0;
                        (*(*pIpForwardTable).table.as_mut_ptr().add(ndx as usize)).dwForwardMetric5 = 0;
                        ndx = ndx.wrapping_add(1);
                    }
                    if bOrder != 0 {
                        qsort(
                            core::ptr::addr_of_mut!((*pIpForwardTable).table) as PVOID,
                            (*pIpForwardTable).dwNumEntries as usize,
                            core::mem::size_of::<MIB_IPFORWARDROW>(),
                            Some(IpForwardTableSorter),
                        );
                    }
                    ret = NO_ERROR;
                }
                HeapFree(GetProcessHeap(), 0, table as PVOID);
            } else {
                ret = ERROR_OUTOFMEMORY;
            }
        }
    }

    TRACE!((b"returning %ld\n\0".as_ptr() as *const i8), ret);
    return ret;
}
// ---- 27_IpNetTableSorter.rs ----
unsafe extern "C" fn IpNetTableSorter(
    a: *const core::ffi::c_void,
    b: *const core::ffi::c_void,
) -> core::ffi::c_int {
    let mut ret: core::ffi::c_int;

    if !a.is_null() && !b.is_null() {
        ret = (*(a as PMIB_IPNETROW))
            .dwAddr
            .wrapping_sub((*(b as PMIB_IPNETROW)).dwAddr) as core::ffi::c_int;
    } else {
        ret = 0;
    }
    ret
}
// ---- 28_GetIpNetTable.rs ----
#[no_mangle]
pub unsafe extern "system" fn GetIpNetTable(
    pIpNetTable: PMIB_IPNETTABLE,
    pdwSize: PULONG,
    bOrder: BOOL,
) -> DWORD {

    let mut ret: DWORD = NO_ERROR;

    TRACE!(
        (b"pIpNetTable %p, pdwSize %p, bOrder %d\n\0".as_ptr() as *const i8),
        pIpNetTable,
        pdwSize,
        bOrder as DWORD,
    );

    if pdwSize.is_null() {
        ret = ERROR_INVALID_PARAMETER;
    } else {
        let numEntries: DWORD = getNumArpEntries();
        let mut size: ULONG = core::mem::size_of::<MIB_IPNETTABLE>() as ULONG;

        if numEntries > 1 {
            size = size.wrapping_add(
                ((numEntries.wrapping_sub(1)) as usize)
                    .wrapping_mul(core::mem::size_of::<MIB_IPNETROW>()) as ULONG
            ) as ULONG;
        }

        if pIpNetTable.is_null() || *pdwSize < size {
            *pdwSize = size;
            ret = ERROR_INSUFFICIENT_BUFFER;
        } else {
            let table: PMIB_IPNETTABLE = getArpTable();
            if !table.is_null() {
                size = core::mem::size_of::<MIB_IPNETTABLE>() as ULONG;
                if (*table).dwNumEntries > 1 {
                    size = size.wrapping_add(
                        (((*table).dwNumEntries.wrapping_sub(1)) as usize)
                            .wrapping_mul(core::mem::size_of::<MIB_IPNETROW>()) as ULONG
                    ) as ULONG;
                }
                if *pdwSize < size {
                    *pdwSize = size;
                    ret = ERROR_INSUFFICIENT_BUFFER;
                } else {
                    *pdwSize = size;
                    memcpy(
                        pIpNetTable as *mut core::ffi::c_void,
                        table as *const core::ffi::c_void,
                        size as usize,
                    );
                    if bOrder != 0 {
                        qsort(
                            core::ptr::addr_of_mut!((*pIpNetTable).table) as *mut core::ffi::c_void,
                            (*pIpNetTable).dwNumEntries as usize,
                            core::mem::size_of::<MIB_IPNETROW>(),
                            Some(IpNetTableSorter),
                        );
                    }
                    ret = NO_ERROR;
                }
                HeapFree(GetProcessHeap(), 0, table as PVOID);
            }
        }
    }

    TRACE!((b"returning %d\n\0".as_ptr() as *const i8), ret);
    ret
}
// ---- 29_GetIpStatisticsEx.rs ----
#[no_mangle]
pub unsafe extern "system" fn GetIpStatisticsEx(pStats: PMIB_IPSTATS, dwFamily: DWORD) -> DWORD {
    let mut tcpFile: HANDLE = core::ptr::null_mut();
    let ret: DWORD;

    if pStats.is_null() {
        return ERROR_INVALID_PARAMETER;
    }

    if dwFamily != AF_INET && dwFamily != AF_INET6 {
        return ERROR_INVALID_PARAMETER;
    }

    if !NT_SUCCESS(openTcpFile(core::ptr::addr_of_mut!(tcpFile), FILE_READ_DATA)) {
        return ERROR_NOT_SUPPORTED;
    }

    TRACE!((b"pStats %p\n\0".as_ptr() as *const i8), pStats);
    ret = getIPStats(tcpFile, pStats);
    closeTcpFile(tcpFile);
    TRACE!((b"returning %ld\n\0".as_ptr() as *const i8), ret);
    return ret;
}

// ---- 30_GetNetworkParams.rs ----
#[no_mangle]
pub unsafe extern "system" fn GetNetworkParams(
    pFixedInfo: PFIXED_INFO,
    pOutBufLen: PULONG,
) -> DWORD {
    let mut ret: DWORD;
    let mut size: DWORD;
    let mut type_0: DWORD;
    let mut regReturn: LONG;
    let mut hKey: HKEY = core::ptr::null_mut();
    let mut resInfo: PIPHLP_RES_INFO;

    TRACE!((b"pFixedInfo %p, pOutBufLen %p\n\0".as_ptr() as *const i8), pFixedInfo, pOutBufLen);

    if pOutBufLen.is_null() {
        return ERROR_INVALID_PARAMETER;
    }

    resInfo = getResInfo();
    if resInfo.is_null() {
        return ERROR_OUTOFMEMORY;
    }

    size = (core::mem::size_of::<FIXED_INFO>()).wrapping_add(
        if (*resInfo).riCount > 1 {
            ((*resInfo).riCount.wrapping_sub(1) as usize)
                .wrapping_mul(core::mem::size_of::<IP_ADDR_STRING>())
        } else {
            0
        }
    ) as DWORD;

    if pFixedInfo.is_null() || *pOutBufLen < size {
        *pOutBufLen = size;
        disposeResInfo(resInfo);
        return ERROR_BUFFER_OVERFLOW;
    }

    memset(pFixedInfo as *mut core::ffi::c_void, 0, size as usize);

    regReturn = RegOpenKeyExA(
        HKEY_LOCAL_MACHINE,
        b"SYSTEM\\CurrentControlSet\\Services\\Tcpip\\Parameters\0".as_ptr() as LPCSTR,
        0,
        KEY_READ,
        core::ptr::addr_of_mut!(hKey),
    );

    if regReturn == ERROR_SUCCESS as LONG {
        type_0 = REG_SZ;
        size = core::mem::size_of::<[i8; 132]>() as DWORD;
        regReturn = RegQueryValueExA(
            hKey,
            b"Hostname\0".as_ptr() as LPCSTR,
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!(type_0),
            core::ptr::addr_of_mut!((*pFixedInfo).HostName) as LPBYTE,
            core::ptr::addr_of_mut!(size),
        );

        type_0 = REG_SZ;
        size = core::mem::size_of::<[i8; 132]>() as DWORD;
        regReturn = RegQueryValueExA(
            hKey,
            b"DhcpDomain\0".as_ptr() as LPCSTR,
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!(type_0),
            core::ptr::addr_of_mut!((*pFixedInfo).DomainName) as LPBYTE,
            core::ptr::addr_of_mut!(size),
        );

        if regReturn == ERROR_FILE_NOT_FOUND as LONG
            || (regReturn == ERROR_SUCCESS as LONG && size < 1)
        {
            type_0 = REG_SZ;
            size = core::mem::size_of::<[i8; 132]>() as DWORD;
            regReturn = RegQueryValueExA(
                hKey,
                b"Domain\0".as_ptr() as LPCSTR,
                core::ptr::null_mut(),
                core::ptr::addr_of_mut!(type_0),
                core::ptr::addr_of_mut!((*pFixedInfo).DomainName) as LPBYTE,
                core::ptr::addr_of_mut!(size),
            );
        }

        RegCloseKey(hKey);
    }

    TRACE!((b"GetComputerNameExA: %s\n\0".as_ptr() as *const i8), core::ptr::addr_of_mut!((*pFixedInfo).DomainName));

    if (*resInfo).riCount > 0 {
        CopyMemory(
            core::ptr::addr_of_mut!((*pFixedInfo).DnsServerList) as *mut core::ffi::c_void,
            (*resInfo).DnsList as *const core::ffi::c_void,
            core::mem::size_of::<IP_ADDR_STRING>(),
        );

        if (*resInfo).riCount > 1 {
            let mut pSrc: *mut IP_ADDR_STRING = (*(*resInfo).DnsList).Next as *mut IP_ADDR_STRING;
            let mut pTarget: *mut IP_ADDR_STRING = (pFixedInfo as *mut u8)
                .offset(core::mem::size_of::<FIXED_INFO>() as isize)
                as *mut IP_ADDR_STRING;

            (*pFixedInfo).DnsServerList.Next = pTarget as *mut _IP_ADDR_STRING;

            loop {
                CopyMemory(
                    pTarget as *mut core::ffi::c_void,
                    pSrc as *const core::ffi::c_void,
                    core::mem::size_of::<IP_ADDR_STRING>(),
                );
                (*resInfo).riCount = (*resInfo).riCount.wrapping_sub(1);

                if (*resInfo).riCount > 1 {
                    (*pTarget).Next = (pTarget as *mut u8)
                        .offset(core::mem::size_of::<IP_ADDR_STRING>() as isize)
                        as *mut _IP_ADDR_STRING;
                    pTarget = (*pTarget).Next as *mut IP_ADDR_STRING;
                    pSrc = (*pSrc).Next as *mut IP_ADDR_STRING;
                } else {
                    (*pTarget).Next = core::ptr::null_mut();
                    break;
                }
            }
        } else {
            (*pFixedInfo).DnsServerList.Next = core::ptr::null_mut();
        }
    }

    (*pFixedInfo).NodeType = HYBRID_NODETYPE;

    regReturn = RegOpenKeyExA(
        HKEY_LOCAL_MACHINE,
        b"SYSTEM\\CurrentControlSet\\Services\\VxD\\MSTCP\0".as_ptr() as LPCSTR,
        0,
        KEY_READ,
        core::ptr::addr_of_mut!(hKey),
    );

    if regReturn != ERROR_SUCCESS as LONG {
        regReturn = RegOpenKeyExA(
            HKEY_LOCAL_MACHINE,
            b"SYSTEM\\CurrentControlSet\\Services\\NetBT\\Parameters\0".as_ptr() as LPCSTR,
            0,
            KEY_READ,
            core::ptr::addr_of_mut!(hKey),
        );
    }

    if regReturn == ERROR_SUCCESS as LONG {
        let mut size_0: DWORD = core::mem::size_of::<[i8; 260]>() as DWORD;
        RegQueryValueExA(
            hKey,
            b"ScopeID\0".as_ptr() as LPCSTR,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!((*pFixedInfo).ScopeId) as LPBYTE,
            core::ptr::addr_of_mut!(size_0),
        );
        RegCloseKey(hKey);
    }

    disposeResInfo(resInfo);

    ret = NO_ERROR;
    TRACE!((b"returning %ld\n\0".as_ptr() as *const i8), ret);

    return ret;
}
// ---- 31_GetNumberOfInterfaces.rs ----
#[no_mangle]
pub unsafe extern "system" fn GetNumberOfInterfaces(pdwNumIf: PDWORD) -> DWORD {

    TRACE!((b"pdwNumIf %p\n\0".as_ptr() as *const i8), pdwNumIf);

    let ret: DWORD;
    if pdwNumIf.is_null() {
        ret = ERROR_INVALID_PARAMETER;
    } else {
        *pdwNumIf = getNumInterfaces();
        ret = NO_ERROR;
    }

    TRACE!((b"returning %ld\n\0".as_ptr() as *const i8), ret);
    ret
}
// ---- 32_GetOwnerModuleFromPidEntry.rs ----
unsafe fn GetOwnerModuleFromPidEntry(
    OwningPid: DWORD,
    Class: TCPIP_OWNER_MODULE_INFO_CLASS,
    Buffer: PVOID,
    pdwSize: PDWORD,
) -> DWORD {
    let mut Process: HANDLE;
    let mut FileLen: DWORD;
    let mut PathLen: DWORD;
    let mut Error: DWORD;
    let mut File: [WCHAR; MAX_PATH as usize] = [0; MAX_PATH as usize];
    let mut Path: [WCHAR; MAX_PATH as usize] = [0; MAX_PATH as usize];
    let mut BasicInfo: PTCPIP_OWNER_MODULE_BASIC_INFO;

    if IsBadWritePtr(pdwSize as PVOID, core::mem::size_of::<DWORD>()) != 0
        || IsBadWritePtr(Buffer, *pdwSize as usize) != 0
    {
        return ERROR_INVALID_PARAMETER;
    }

    if OwningPid == 0 {
        return ERROR_NOT_FOUND;
    }

    Process = OpenProcess(PROCESS_QUERY_INFORMATION | PROCESS_VM_READ, FALSE, OwningPid);
    if Process.is_null() {
        return GetLastError();
    }

    FileLen = GetModuleBaseNameW(
        Process,
        core::ptr::null_mut(),
        File.as_mut_ptr(),
        MAX_PATH,
    );
    if FileLen != 0 {
        PathLen = GetModuleFileNameExW(
            Process,
            core::ptr::null_mut(),
            Path.as_mut_ptr(),
            MAX_PATH,
        );
        if PathLen == 0 {
            CloseHandle(Process);
            return GetLastError();
        }

        FileLen = FileLen.wrapping_add(1);
        PathLen = PathLen.wrapping_add(1);
        PathLen = PathLen.wrapping_mul(core::mem::size_of::<WCHAR>() as DWORD);
        FileLen = FileLen.wrapping_mul(core::mem::size_of::<WCHAR>() as DWORD);
    } else {
        Error = GetLastError();

        if Error == ERROR_PARTIAL_COPY {
            wcscpy(
                File.as_mut_ptr(),
                [b'S' as u16, b'y' as u16, b's' as u16, b't' as u16, b'e' as u16, b'm' as u16, 0u16].as_ptr(),
            );
            wcscpy(
                Path.as_mut_ptr(),
                [b'S' as u16, b'y' as u16, b's' as u16, b't' as u16, b'e' as u16, b'm' as u16, 0u16].as_ptr(),
            );

            PathLen = core::mem::size_of::<[u16; 7]>() as DWORD;
            FileLen = core::mem::size_of::<[u16; 7]>() as DWORD;
        } else {
            CloseHandle(Process);
            return Error;
        }
    }

    CloseHandle(Process);

    if (*pdwSize as usize)
        < core::mem::size_of::<TCPIP_OWNER_MODULE_BASIC_INFO>()
            .wrapping_add(PathLen as usize)
            .wrapping_add(FileLen as usize)
    {
        *pdwSize = (core::mem::size_of::<TCPIP_OWNER_MODULE_BASIC_INFO>()
            .wrapping_add(PathLen as usize)
            .wrapping_add(FileLen as usize)) as DWORD;
        return ERROR_INSUFFICIENT_BUFFER;
    }

    BasicInfo = Buffer as PTCPIP_OWNER_MODULE_BASIC_INFO;
    (*BasicInfo).pModuleName = ((BasicInfo as usize)
        .wrapping_add(core::mem::size_of::<TCPIP_OWNER_MODULE_BASIC_INFO>())) as PWSTR;
    (*BasicInfo).pModulePath = (((*BasicInfo).pModuleName as usize)
        .wrapping_add(FileLen as usize)) as PWSTR;
    wcscpy((*BasicInfo).pModuleName, File.as_mut_ptr());
    wcscpy((*BasicInfo).pModulePath, Path.as_mut_ptr());
    *pdwSize = (core::mem::size_of::<TCPIP_OWNER_MODULE_BASIC_INFO>()
        .wrapping_add(PathLen as usize)
        .wrapping_add(FileLen as usize)) as DWORD;

    return NO_ERROR;
}
// ---- 33_GetOwnerModuleFromTagEntry.rs ----
unsafe fn GetOwnerModuleFromTagEntry(
    OwningPid: DWORD,
    OwningTag: DWORD,
    Class: TCPIP_OWNER_MODULE_INFO_CLASS,
    Buffer: PVOID,
    pdwSize: PDWORD,
) -> DWORD {
    let mut Size: u32;
    let mut Res: HRESULT;
    let mut hAdvapi32: HANDLE;
    let mut SysDir: [u16; 260] = [0; 260];
    let mut BasicInfo: PTCPIP_OWNER_MODULE_BASIC_INFO;
    let mut _I_QueryTagInformation: Option<unsafe extern "C" fn(PVOID, DWORD, PVOID) -> u32>;
    let mut ServiceQuery = ServiceQueryStruct {
        ProcessId: 0,
        ServiceTag: 0,
        TagType: 0,
        Buffer: core::ptr::null_mut::<u16>(),
    };

    if IsBadWritePtr(pdwSize as PVOID, core::mem::size_of::<DWORD>() as usize) != 0
        || IsBadWritePtr(Buffer, *pdwSize as usize) != 0
    {
        return ERROR_INVALID_PARAMETER;
    }

    Size = GetSystemDirectoryW(SysDir.as_mut_ptr(), MAX_PATH as u32);
    if Size == 0 {
        return GetLastError();
    }

    // L"\advapi32.dll"
    static ADVAPI32_DLL_BUF: [u16; 14] = [
        b'\\' as u16, b'a' as u16, b'd' as u16, b'v' as u16, b'a' as u16, b'p' as u16, b'i' as u16, b'3' as u16, b'2' as u16, b'.' as u16,
        b'd' as u16, b'l' as u16, b'l' as u16, 0,
    ];
    Res = StringCchCatW(
        SysDir.as_mut_ptr().offset(Size as isize),
        (MAX_PATH as u32 - Size) as usize,
        ADVAPI32_DLL_BUF.as_ptr() as *const u16,
    );
    if FAILED(Res) {
        return Res as DWORD;
    }

    hAdvapi32 = GetModuleHandleW(SysDir.as_mut_ptr()) as HANDLE;
    if hAdvapi32.is_null() {
        return GetLastError();
    }

    _I_QueryTagInformation = core::mem::transmute::<PVOID, Option<unsafe extern "C" fn(PVOID, DWORD, PVOID) -> u32>>(
        GetProcAddress(hAdvapi32 as HINSTANCE, b"I_QueryTagInformation\0".as_ptr() as LPCSTR) as PVOID
    );
    if _I_QueryTagInformation.is_none() {
        return GetLastError();
    }

    ServiceQuery.ProcessId = OwningPid;
    ServiceQuery.ServiceTag = OwningTag;
    ServiceQuery.TagType = 0;
    ServiceQuery.Buffer = core::ptr::null_mut::<u16>();

    // SAFETY: guarded by the `_I_QueryTagInformation.is_none() { return }` check above, so this
    // is always Some — unwrap_unchecked avoids a panic path (no .expect/.unwrap in no_std code).
    Res = _I_QueryTagInformation.unwrap_unchecked()(
        core::ptr::null_mut(),
        1,
        &mut ServiceQuery as *mut ServiceQueryStruct as PVOID,
    ) as HRESULT;
    if Res != ERROR_SUCCESS as i32 {
        return Res as DWORD;
    }

    Size = (wcslen(ServiceQuery.Buffer as *const u16) * core::mem::size_of::<u16>()
        + core::mem::size_of::<u16>()) as u32;

    if *pdwSize < (core::mem::size_of::<TCPIP_OWNER_MODULE_BASIC_INFO>() + (2 * Size) as usize) as u32 {
        *pdwSize = (core::mem::size_of::<TCPIP_OWNER_MODULE_BASIC_INFO>() + (2 * Size) as usize) as DWORD;
        LocalFree(ServiceQuery.Buffer as HLOCAL);
        return ERROR_INSUFFICIENT_BUFFER;
    }

    BasicInfo = Buffer as PTCPIP_OWNER_MODULE_BASIC_INFO;
    (*BasicInfo).pModuleName = ((BasicInfo as usize)
        + core::mem::size_of::<TCPIP_OWNER_MODULE_BASIC_INFO>()) as PVOID as *mut u16;
    (*BasicInfo).pModulePath = (((*BasicInfo).pModuleName as usize)
        + Size as usize) as PVOID as *mut u16;
    wcscpy((*BasicInfo).pModuleName as *mut u16, ServiceQuery.Buffer as *const u16);
    wcscpy((*BasicInfo).pModulePath as *mut u16, ServiceQuery.Buffer as *const u16);
    *pdwSize = (core::mem::size_of::<TCPIP_OWNER_MODULE_BASIC_INFO>() + (2 * Size) as usize) as DWORD;
    LocalFree(ServiceQuery.Buffer as HLOCAL);

    return NO_ERROR;
}

#[repr(C)]
struct ServiceQueryStruct {
    ProcessId: DWORD,
    ServiceTag: DWORD,
    TagType: DWORD,
    Buffer: *mut u16,
}


type HLOCAL = *mut core::ffi::c_void;
type TCPIP_OWNER_MODULE_INFO_CLASS = u32;
type PTCPIP_OWNER_MODULE_BASIC_INFO = *mut TCPIP_OWNER_MODULE_BASIC_INFO;

#[repr(C)]
struct TCPIP_OWNER_MODULE_BASIC_INFO {
    pModuleName: *mut u16,
    pModulePath: *mut u16,
}
// ---- 34_CreateNameServerListEnumNamesFunc.rs ----
unsafe extern "C" fn CreateNameServerListEnumNamesFunc(
    Interface: PWSTR,
    Server: PWSTR,
    Data: PVOID,
) {

    let pNext: *mut IP_ADDR_STRING;
    let Context: PNAME_SERVER_LIST_CONTEXT = Data as PNAME_SERVER_LIST_CONTEXT;

    if (*Context).NumServers == 0 {
        if (*Context).uSizeAvailable >= (*Context).uSizeRequired {
            WideCharToMultiByte(
                CP_ACP,
                0,
                Server as LPCWSTR,
                -1,
                core::ptr::addr_of_mut!((*(*Context).pData).DnsServerList.IpAddress.String) as LPSTR,
                16,
                core::ptr::null(),
                core::ptr::null_mut(),
            );
            (*(*Context).pData).DnsServerList.IpAddress.String[15] = 0;
            (*Context).pLastAddr = core::ptr::addr_of_mut!((*(*Context).pData).DnsServerList);
        }
    } else {
        (*Context).uSizeRequired = ((*Context).uSizeRequired as usize)
            .wrapping_add(core::mem::size_of::<IP_ADDR_STRING>()) as ULONG;
        if (*Context).uSizeAvailable >= (*Context).uSizeRequired {
            pNext = ((*Context).pLastAddr as *mut u8)
                .offset(core::mem::size_of::<IP_ADDR_STRING>() as isize)
                as *mut IP_ADDR_STRING;
            WideCharToMultiByte(
                CP_ACP,
                0,
                Server as LPCWSTR,
                -1,
                core::ptr::addr_of_mut!((*pNext).IpAddress.String) as LPSTR,
                16,
                core::ptr::null(),
                core::ptr::null_mut(),
            );
            (*pNext).IpAddress.String[15] = 0;
            (*(*Context).pLastAddr).Next = pNext as *mut _IP_ADDR_STRING;
            (*Context).pLastAddr = pNext;
            (*pNext).Next = core::ptr::null_mut();
        }
    }
    (*Context).NumServers = (*Context).NumServers.wrapping_add(1);
}
// ---- 35_GetPerAdapterInfo.rs ----
#[no_mangle]
pub unsafe extern "system" fn GetPerAdapterInfo(
    IfIndex: ULONG,
    pPerAdapterInfo: PIP_PER_ADAPTER_INFO,
    pOutBufLen: PULONG,
) -> DWORD {

    const CP_ACP: u32 = 0;
    const HKEY_LOCAL_MACHINE: HKEY = 0x80000002 as HKEY;
    const NOERROR: DWORD = 0;

    let mut hkey: HKEY = core::ptr::null_mut();
    let mut dwSize: DWORD = 0;
    let ifName: *const i8;
    let mut Context: NAME_SERVER_LIST_CONTEXT = NAME_SERVER_LIST_CONTEXT {
        uSizeAvailable: 0,
        uSizeRequired: 0,
        pData: core::ptr::null_mut(),
        NumServers: 0,
        pLastAddr: core::ptr::null_mut(),
    };
    let mut keyname: [u16; 200] = [0u16; 200];
    {
        // C: WCHAR keyname[200] = L"SYSTEM\\CurrentControlSet\\Services\\Tcpip\\Parameters\\Interfaces\\";
        let __p = "SYSTEM\\CurrentControlSet\\Services\\Tcpip\\Parameters\\Interfaces\\";
        let mut __i = 0usize;
        for __c in __p.chars() { keyname[__i] = __c as u16; __i += 1; }
    }

    if pOutBufLen.is_null() {
        return ERROR_INVALID_PARAMETER;
    }

    if pPerAdapterInfo.is_null()
        || (*pOutBufLen as usize) < core::mem::size_of::<IP_PER_ADAPTER_INFO>()
    {
        *pOutBufLen = core::mem::size_of::<IP_PER_ADAPTER_INFO>() as ULONG;
        return ERROR_BUFFER_OVERFLOW;
    }

    ifName = getInterfaceNameByIndex(IfIndex);
    if ifName.is_null() {
        return ERROR_INVALID_PARAMETER;
    }

    MultiByteToWideChar(
        CP_ACP,
        0,
        ifName,
        -1,
        keyname.as_mut_ptr().offset(62),
        (core::mem::size_of::<[u16; 200]>() / core::mem::size_of::<u16>() - 63) as i32,
    );
    HeapFree(GetProcessHeap(), 0, ifName as LPVOID);

    if RegOpenKeyExW(
        HKEY_LOCAL_MACHINE,
        keyname.as_ptr(),
        0,
        KEY_READ,
        core::ptr::addr_of_mut!(hkey),
    ) != ERROR_SUCCESS as i32
    {
        return ERROR_NOT_SUPPORTED;
    }

    Context.NumServers = 0;
    Context.uSizeAvailable = *pOutBufLen;
    Context.uSizeRequired = core::mem::size_of::<IP_PER_ADAPTER_INFO>() as ULONG;
    Context.pData = pPerAdapterInfo;

    if *pOutBufLen as usize >= core::mem::size_of::<IP_PER_ADAPTER_INFO>() {
        ZeroMemory(
            pPerAdapterInfo as LPVOID,
            core::mem::size_of::<IP_PER_ADAPTER_INFO>(),
        );
    }

    EnumNameServers(
        hkey,
        keyname.as_mut_ptr().offset(62),
        core::ptr::addr_of_mut!(Context) as LPVOID,
        Some(CreateNameServerListEnumNamesFunc),
    );

    if Context.uSizeRequired > Context.uSizeAvailable {
        *pOutBufLen = Context.uSizeRequired;
        RegCloseKey(hkey);
        return ERROR_BUFFER_OVERFLOW;
    }

    if RegQueryValueExW(
        hkey,
        win32macros_rust::wide_bytes!(b"N\0a\0m\0e\0S\0e\0r\0v\0e\0r\0\0\0") as *const u16,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(dwSize),
    ) == ERROR_SUCCESS as i32
    {
        (*pPerAdapterInfo).AutoconfigActive = 0;
    } else {
        (*pPerAdapterInfo).AutoconfigActive = 1;
    }

    RegCloseKey(hkey);
    NOERROR
}
// ---- 36_GetRTTAndHopCount.rs ----
#[no_mangle]
pub unsafe extern "system" fn GetRTTAndHopCount(
    DestIpAddress: IPAddr,
    HopCount: PULONG,
    MaxHops: ULONG,
    RTT: PULONG,
) -> BOOL {
    if (*__wine_dbch___default).flags as i32
        & (1i32) << __WINE_DBCL_TRACE as i32
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"/home/curi/Projects/os-playground/vibeos-research/dll/win32/iphlpapi/iphlpapi_main.c\0"
                as *const u8 as *const i8,
            b"GetRTTAndHopCount\0" as *const u8 as *const i8,
            2485i32,
            b"DestIpAddress 0x%08lx, HopCount %p, MaxHops %ld, RTT %p\n\0" as *const u8
                as *const i8,
            DestIpAddress,
            HopCount,
            MaxHops,
            RTT,
        );
    }
    if (*__wine_dbch___default).flags as i32
        & (1i32) << __WINE_DBCL_FIXME as i32
        != 0
    {
        let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl_0: __wine_debug_class = __WINE_DBCL_FIXME;
        ros_dbg_log(
            __dbcl_0,
            __dbch_0,
            b"/home/curi/Projects/os-playground/vibeos-research/dll/win32/iphlpapi/iphlpapi_main.c\0"
                as *const u8 as *const i8,
            b"GetRTTAndHopCount\0" as *const u8 as *const i8,
            2486i32,
            b":stub\n\0" as *const u8 as *const i8,
        );
    }
    return 0i32;
}
// ---- 37_GetTcpStatisticsEx.rs ----
#[no_mangle]
pub unsafe extern "system" fn GetTcpStatisticsEx(
    pStats: PMIB_TCPSTATS,
    dwFamily: DWORD,
) -> DWORD {
    let mut tcpFile: HANDLE = core::ptr::null_mut();
    let ret: DWORD;

    if pStats.is_null() {
        return ERROR_INVALID_PARAMETER;
    }

    if dwFamily != AF_INET && dwFamily != AF_INET6 {
        return ERROR_INVALID_PARAMETER;
    }

    if !NT_SUCCESS(openTcpFile(core::ptr::addr_of_mut!(tcpFile), FILE_READ_DATA)) {
        return ERROR_NOT_SUPPORTED;
    }

    TRACE!((b"pStats %p\n\0".as_ptr() as *const i8), pStats);
    ret = getTCPStats(tcpFile, pStats);
    closeTcpFile(tcpFile);
    TRACE!((b"returning %ld\n\0".as_ptr() as *const i8), ret);
    ret
}
// ---- 38_GetUdpStatisticsEx.rs ----
#[no_mangle]
pub unsafe extern "system" fn GetUdpStatisticsEx(
    pStats: PMIB_UDPSTATS,
    dwFamily: DWORD,
) -> DWORD {
    let mut tcpFile: HANDLE = core::ptr::null_mut();
    let mut ret: DWORD;

    if pStats.is_null() {
        return ERROR_INVALID_PARAMETER;
    }

    if dwFamily != AF_INET && dwFamily != AF_INET6 {
        return ERROR_INVALID_PARAMETER;
    }

    if !NT_SUCCESS(openTcpFile(core::ptr::addr_of_mut!(tcpFile), FILE_READ_DATA)) {
        return ERROR_NOT_SUPPORTED;
    }

    TRACE!((b"pStats %p\n\0".as_ptr() as *const i8), pStats);
    ret = getUDPStats(tcpFile, pStats);
    closeTcpFile(tcpFile);
    TRACE!((b"returning %ld\n\0".as_ptr() as *const i8), ret);
    ret
}
// ---- 39_GetUniDirectionalAdapterInfo.rs ----
#[no_mangle]
pub unsafe extern "system" fn GetUniDirectionalAdapterInfo(
    pIPIfInfo: PIP_UNIDIRECTIONAL_ADAPTER_ADDRESS,
    dwOutBufLen: PULONG,
) -> DWORD {
    TRACE!(
        (b"pIPIfInfo %p, dwOutBufLen %p\n\0".as_ptr() as *const i8),
        pIPIfInfo,
        dwOutBufLen,
    );
    ERROR_NOT_SUPPORTED
}
// ---- 40_IpReleaseAddress.rs ----
#[no_mangle]
pub unsafe extern "system" fn IpReleaseAddress(AdapterInfo: PIP_ADAPTER_INDEX_MAP) -> DWORD {

    let mut Status: DWORD;
    let mut Version: DWORD = 0;

    if AdapterInfo.is_null() {
        return ERROR_INVALID_PARAMETER;
    }

    if DhcpCApiInitialize(core::ptr::addr_of_mut!(Version)) != ERROR_SUCCESS {
        return ERROR_PROC_NOT_FOUND;
    }

    Status = DhcpReleaseParameters(core::ptr::addr_of_mut!((*AdapterInfo).Name) as PWSTR);

    DhcpCApiCleanup();

    Status
}
// ---- 41_IpRenewAddress.rs ----
#[no_mangle]
pub unsafe extern "system" fn IpRenewAddress(AdapterInfo: PIP_ADAPTER_INDEX_MAP) -> DWORD {

    let mut Status: DWORD;
    let mut Version: DWORD = 0;

    if AdapterInfo.is_null() {
        return ERROR_INVALID_PARAMETER;
    }

    if DhcpCApiInitialize(core::ptr::addr_of_mut!(Version)) != ERROR_SUCCESS {
        return ERROR_PROC_NOT_FOUND;
    }

    Status = DhcpAcquireParameters(core::ptr::addr_of_mut!((*AdapterInfo).Name) as PWSTR);

    DhcpCApiCleanup();

    return Status;
}
// ---- 42_NotifyAddrChange.rs ----
#[no_mangle]
pub unsafe extern "system" fn NotifyAddrChange(
    Handle: PHANDLE,
    overlapped: LPOVERLAPPED,
) -> DWORD {

    FIXME!(
        "(Handle %p, overlapped %p): stub\n",
        Handle,
        overlapped,
    );

    if !Handle.is_null() {
        *Handle = INVALID_HANDLE_VALUE as *mut core::ffi::c_void;
    }

    if !overlapped.is_null() {
        (*(overlapped as *mut IO_STATUS_BLOCK)).__bindgen_anon_1.Status = STATUS_PENDING;
    }

    ERROR_IO_PENDING
}
// ---- 43_NotifyRouteChange.rs ----
#[no_mangle]
pub unsafe extern "system" fn NotifyRouteChange(
    Handle: PHANDLE,
    overlapped: LPOVERLAPPED,
) -> DWORD {
    FIXME!(
        "(Handle %p, overlapped %p): stub\n",
        Handle,
        overlapped,
    );
    ERROR_NOT_SUPPORTED
}
// ---- 44_SendARP.rs ----
#[no_mangle]
pub unsafe extern "system" fn SendARP(
    DestIP: IPAddr,
    SrcIP: IPAddr,
    pMacAddr: PULONG,
    PhyAddrLen: PULONG,
) -> DWORD {
    let mut IPs: [IPAddr; 2] = [0; 2];
    let mut Size: ULONG;

    if IsBadWritePtr(pMacAddr as PVOID, core::mem::size_of::<ULONG>()) != 0
        || IsBadWritePtr(PhyAddrLen as PVOID, core::mem::size_of::<ULONG>()) != 0
    {
        return ERROR_INVALID_PARAMETER;
    }

    IPs[0] = DestIP;
    IPs[1] = SrcIP;
    Size = core::mem::size_of::<[IPAddr; 2]>() as ULONG;

    TCPSendIoctl(
        INVALID_HANDLE_VALUE,
        IOCTL_QUERY_IP_HW_ADDRESS,
        IPs.as_mut_ptr() as LPVOID,
        core::ptr::addr_of_mut!(Size),
        pMacAddr as LPVOID,
        PhyAddrLen,
    )
}
// ---- 45_SetIfEntry.rs ----
#[no_mangle]
pub unsafe extern "system" fn SetIfEntry(pIfRow: PMIB_IFROW) -> DWORD {
    FIXME!("(pIfRow %p): stub\n", pIfRow);
    ERROR_NOT_SUPPORTED
}
// ---- 46_SetIpForwardEntry.rs ----
#[no_mangle]
pub unsafe extern "system" fn SetIpForwardEntry(pRoute: PMIB_IPFORWARDROW) -> DWORD {

    setIpForwardEntry(pRoute)
}
// ---- 47_SetIpNetEntry.rs ----
#[no_mangle]
pub unsafe extern "system" fn SetIpNetEntry(pArpEntry: PMIB_IPNETROW) -> DWORD {
    let mut tcpFile: HANDLE = core::ptr::null_mut();
    let mut status: NTSTATUS;
    let mut req: TCP_REQUEST_SET_INFORMATION_EX_ARP_ENTRY = core::mem::zeroed();
    let mut id: TDIEntityID = TDIEntityID {
        tei_entity: 0,
        tei_instance: 0,
    };
    let mut returnSize: DWORD = 0;
    let arpBuff: PMIB_IPNETROW;

    if pArpEntry.is_null() {
        return ERROR_INVALID_PARAMETER;
    }

    if !NT_SUCCESS(openTcpFile(
        core::ptr::addr_of_mut!(tcpFile),
        FILE_READ_DATA | FILE_WRITE_DATA,
    )) {
        return ERROR_NOT_SUPPORTED;
    }

    if !NT_SUCCESS(getNthIpEntity(tcpFile, (*pArpEntry).dwIndex, core::ptr::addr_of_mut!(id))) {
        closeTcpFile(tcpFile);
        return ERROR_INVALID_PARAMETER;
    }

    req.Req.as_mut().ID.toi_class = INFO_CLASS_PROTOCOL;
    req.Req.as_mut().ID.toi_type = INFO_TYPE_PROVIDER;
    req.Req.as_mut().ID.toi_id = IP_MIB_ARPTABLE_ENTRY_ID;
    req.Req.as_mut().ID.toi_entity.tei_instance = id.tei_instance;
    req.Req.as_mut().ID.toi_entity.tei_entity = AT_ENTITY;
    req.Req.as_mut().BufferSize = core::mem::size_of::<MIB_IPNETROW>() as u32;
    arpBuff = req.Req.as_mut().Buffer.as_mut_ptr() as PMIB_IPNETROW;

    RtlCopyMemory(
        arpBuff as LPVOID,
        pArpEntry as LPVOID,
        core::mem::size_of::<MIB_IPNETROW>(),
    );

    status = DeviceIoControl(
        tcpFile,
        IOCTL_TCP_SET_INFORMATION_EX,
        core::ptr::addr_of_mut!(req) as LPVOID,
        core::mem::size_of::<TCP_REQUEST_SET_INFORMATION_EX_ARP_ENTRY>() as DWORD,
        core::ptr::null_mut(),
        0,
        core::ptr::addr_of_mut!(returnSize),
        core::ptr::null_mut(),
    ) as NTSTATUS;

    closeTcpFile(tcpFile);

    if status != 0 {
        NO_ERROR
    } else {
        ERROR_INVALID_PARAMETER
    }
}

// ---- 48_SetIpStatistics.rs ----
#[no_mangle]
pub unsafe extern "system" fn SetIpStatistics(pIpStats: PMIB_IPSTATS) -> DWORD {
    FIXME!("(pIpStats %p): stub\n", pIpStats);
    0
}
// ---- 49_SetIpTTL.rs ----
#[no_mangle]
pub unsafe extern "system" fn SetIpTTL(nTTL: UINT) -> DWORD {
    FIXME!("(nTTL %d): stub\n", nTTL);
    0
}
// ---- 50_SetTcpEntry.rs ----
#[no_mangle]
pub unsafe extern "system" fn SetTcpEntry(pTcpRow: PMIB_TCPROW) -> DWORD {
    FIXME!("(pTcpRow %p): stub\n", pTcpRow);
    0
}
// ---- 51_UnenableRouter.rs ----
#[no_mangle]
pub unsafe extern "system" fn UnenableRouter(
    pOverlapped: *mut OVERLAPPED,
    lpdwEnableCount: LPDWORD,
) -> DWORD {
    FIXME!(
        "(pOverlapped %p, lpdwEnableCount %p): stub\n",
        pOverlapped,
        lpdwEnableCount,
    );
    ERROR_NOT_SUPPORTED
}
// ---- 52_GetIpErrorString.rs ----
#[no_mangle]
pub unsafe extern "system" fn GetIpErrorString(
    ErrorCode: IP_STATUS,
    Buffer: PWSTR,
    Size: PDWORD,
) -> DWORD {
    FIXME!(":stub\n");
    0u32
}
// ---- 53_GetAdapterOrderMap.rs ----
#[no_mangle]
pub unsafe extern "system" fn GetAdapterOrderMap() -> PIP_ADAPTER_ORDER_MAP {
    FIXME!(":stub\n");
    core::ptr::null_mut()
}
// ---- 54_GetAdaptersAddresses.rs ----
// #ifdef GetAdaptersAddressesV1 in iphlpapi_main.c — the build defines GetAdaptersAddressesV2
// (NOT V1), so this V1 body is preprocessed OUT of the C build; address.c provides the active
// GetAdaptersAddresses. cfg(any()) mirrors the false #ifdef: source preserved (never-delete),
// excluded from compile so it can't shadow address.c's symbol.
#[cfg(any())]
#[no_mangle]
pub unsafe extern "system" fn GetAdaptersAddresses(
    Family: ULONG,
    Flags: ULONG,
    Reserved: PVOID,
    pAdapterAddresses: PIP_ADAPTER_ADDRESSES,
    pOutBufLen: PULONG,
) -> DWORD {

    if pOutBufLen.is_null() {
        return ERROR_INVALID_PARAMETER;
    }
    if !Reserved.is_null() {
        return ERROR_INVALID_PARAMETER;
    }

    let indexTable = getInterfaceIndexTable();
    if indexTable.is_null() {
        return ERROR_NOT_ENOUGH_MEMORY;
    }

    let mut tcpFile: HANDLE = core::ptr::null_mut();
    let ret = openTcpFile(&mut tcpFile, FILE_READ_DATA);
    if !NT_SUCCESS(ret) {
        free(indexTable as PVOID);
        return ERROR_NO_DATA;
    }

    let mut requiredSize: ULONG = 0;
    let mut ifInfo: IFInfo = core::mem::zeroed();
    let mut i = (*indexTable).numIndexes as i32;

    while i >= 0 {
        if NT_SUCCESS(getIPAddrEntryForIf(
            tcpFile,
            core::ptr::null_mut(),
            (*(*indexTable).indexes.as_ptr().add(i as usize)),
            &mut ifInfo,
        )) {
            requiredSize += core::mem::size_of::<IP_ADAPTER_ADDRESSES>() as ULONG;

            if (Flags & GAA_FLAG_SKIP_FRIENDLY_NAME) == 0 {
                requiredSize += ifInfo.if_info.ent.if_descrlen + 1;
            }

            requiredSize += ifInfo.if_info.ent.if_descrlen + 1;

            if (Flags & GAA_FLAG_SKIP_UNICAST) == 0 {
                requiredSize += core::mem::size_of::<IP_ADAPTER_UNICAST_ADDRESS>() as ULONG;
            }

            requiredSize += 2 * core::mem::size_of::<WCHAR>() as ULONG;
        }
        i -= 1;
    }

    TRACE!(b"size: %d, requiredSize: %d\n\0".as_ptr(), *pOutBufLen, requiredSize);

    if pAdapterAddresses.is_null() || *pOutBufLen < requiredSize {
        *pOutBufLen = requiredSize;
        closeTcpFile(tcpFile);
        free(indexTable as PVOID);
        return ERROR_BUFFER_OVERFLOW;
    }

    RtlZeroMemory(pAdapterAddresses as PVOID, requiredSize as usize);

    let mut currentAddress = pAdapterAddresses;
    i = (*indexTable).numIndexes as i32;

    while i >= 0 {
        if NT_SUCCESS(getIPAddrEntryForIf(
            tcpFile,
            core::ptr::null_mut(),
            (*(*indexTable).indexes.as_ptr().add(i as usize)),
            &mut ifInfo,
        )) {
            let mut currentLocation = (currentAddress as PUCHAR)
                .offset(core::mem::size_of::<IP_ADAPTER_ADDRESSES>() as isize);

            if (Flags & GAA_FLAG_SKIP_FRIENDLY_NAME) == 0 {
                (*currentAddress).FriendlyName = currentLocation as *mut _;
                currentLocation = currentLocation.offset(core::mem::size_of::<WCHAR>() as isize);
            }

            (*currentAddress).AdapterName = currentLocation as *mut _;
            currentLocation = currentLocation.offset((ifInfo.if_info.ent.if_descrlen + 1) as isize);

            if (Flags & GAA_FLAG_SKIP_UNICAST) == 0 {
                (*currentAddress).FirstUnicastAddress = currentLocation as *mut _;
                currentLocation = currentLocation.offset(core::mem::size_of::<IP_ADAPTER_UNICAST_ADDRESS>() as isize);
                (*(*currentAddress).FirstUnicastAddress).Address.lpSockaddr = currentLocation as *mut _;
                currentLocation = currentLocation.offset(core::mem::size_of::<sockaddr>() as isize);
            }

            (*currentAddress).DnsSuffix = currentLocation as *mut _;
            currentLocation = currentLocation.offset(core::mem::size_of::<WCHAR>() as isize);

            (*currentAddress).Description = currentLocation as *mut _;
            currentLocation = currentLocation.offset(core::mem::size_of::<WCHAR>() as isize);

            (*currentAddress).Next = currentLocation as *mut _;
            if i == 0 {
                (*currentAddress).Next = core::ptr::null_mut();
            }

            currentAddress = (*currentAddress).Next;
        }
        i -= 1;
    }

    currentAddress = pAdapterAddresses;
    i = (*indexTable).numIndexes as i32;

    while i >= 0 {
        if NT_SUCCESS(getIPAddrEntryForIf(
            tcpFile,
            core::ptr::null_mut(),
            (*(*indexTable).indexes.as_ptr().add(i as usize)),
            &mut ifInfo,
        )) {
            (*currentAddress).__bindgen_anon_1.__bindgen_anon_1.as_mut().Length =
                core::mem::size_of::<IP_ADAPTER_ADDRESSES>() as ULONG;
            (*currentAddress).__bindgen_anon_1.__bindgen_anon_1.as_mut().IfIndex =
                *(*indexTable).indexes.as_ptr().add(i as usize);

            memcpy(
                (*currentAddress).AdapterName as *mut core::ffi::c_void,
                ifInfo.if_info.ent.if_descr.as_ptr() as *const core::ffi::c_void,
                ifInfo.if_info.ent.if_descrlen as usize,
            );
            *((*currentAddress).AdapterName as *mut u8).offset(ifInfo.if_info.ent.if_descrlen as isize) = b'\0';

            if (Flags & GAA_FLAG_SKIP_UNICAST) == 0 {
                (*(*currentAddress).FirstUnicastAddress).__bindgen_anon_1.__bindgen_anon_1.as_mut().Length =
                    core::mem::size_of::<IP_ADAPTER_UNICAST_ADDRESS>() as ULONG;
                (*(*currentAddress).FirstUnicastAddress).__bindgen_anon_1.__bindgen_anon_1.as_mut().Flags = 0;
                (*(*currentAddress).FirstUnicastAddress).Next = core::ptr::null_mut();
                (*(*(*currentAddress).FirstUnicastAddress).Address.lpSockaddr).sa_family = AF_INET as u16;
                memcpy(
                    (*(*(*currentAddress).FirstUnicastAddress).Address.lpSockaddr).sa_data.as_mut_ptr() as PVOID,
                    &ifInfo.ip_addr.iae_addr as *const _ as *const core::ffi::c_void,
                    core::mem::size_of_val(&ifInfo.ip_addr.iae_addr),
                );
                (*(*currentAddress).FirstUnicastAddress).Address.iSockaddrLength =
                    (core::mem::size_of_val(&ifInfo.ip_addr.iae_addr) + core::mem::size_of::<USHORT>()) as i32;
                (*(*currentAddress).FirstUnicastAddress).PrefixOrigin = IpPrefixOriginOther;
                (*(*currentAddress).FirstUnicastAddress).SuffixOrigin = IpPrefixOriginOther;
                (*(*currentAddress).FirstUnicastAddress).DadState = IpDadStatePreferred;
                (*(*currentAddress).FirstUnicastAddress).ValidLifetime = 0xFFFFFFFF;
                (*(*currentAddress).FirstUnicastAddress).PreferredLifetime = 0xFFFFFFFF;
                (*(*currentAddress).FirstUnicastAddress).LeaseLifetime = 0xFFFFFFFF;
            }

            (*currentAddress).FirstAnycastAddress = core::ptr::null_mut();
            (*currentAddress).FirstMulticastAddress = core::ptr::null_mut();
            (*currentAddress).FirstDnsServerAddress = core::ptr::null_mut();

            *((*currentAddress).DnsSuffix as *mut WCHAR) = UNICODE_NULL;
            *((*currentAddress).Description as *mut WCHAR) = UNICODE_NULL;
            *((*currentAddress).FriendlyName as *mut WCHAR) = UNICODE_NULL;

            memcpy(
                (*currentAddress).PhysicalAddress.as_mut_ptr() as PVOID,
                ifInfo.if_info.ent.if_physaddr.as_ptr() as *const core::ffi::c_void,
                ifInfo.if_info.ent.if_physaddrlen as usize,
            );
            (*currentAddress).PhysicalAddressLength = ifInfo.if_info.ent.if_physaddrlen;

            (*currentAddress).Flags = 0;

            (*currentAddress).Mtu = ifInfo.if_info.ent.if_mtu;

            (*currentAddress).IfType = ifInfo.if_info.ent.if_type;

            if ifInfo.if_info.ent.if_operstatus >= IF_OPER_STATUS_CONNECTING {
                (*currentAddress).OperStatus = IfOperStatusUp;
            } else {
                (*currentAddress).OperStatus = IfOperStatusDown;
            }

            currentAddress = (*currentAddress).Next;
        }
        i -= 1;
    }

    closeTcpFile(tcpFile);
    free(indexTable as PVOID);

    NO_ERROR
}
// ---- 55_CancelIPChangeNotify.rs ----
#[no_mangle]
pub unsafe extern "system" fn CancelIPChangeNotify(notifyOverlapped: LPOVERLAPPED) -> BOOL {
    FIXME!(":stub\n");
    0
}
// ---- 56_GetBestInterfaceEx.rs ----
#[no_mangle]
pub unsafe extern "system" fn GetBestInterfaceEx(
    pDestAddr: *mut sockaddr,
    pdwBestIfIndex: PDWORD,
) -> DWORD {
    FIXME!(":stub\n");
    return 0;
}
// ---- 57_NhpAllocateAndGetInterfaceInfoFromStack.rs ----
// #ifdef GetAdaptersAddressesV1 (see GetAdaptersAddresses above) — excluded from the V2 build;
// address.c provides the active NhpAllocateAndGetInterfaceInfoFromStack. cfg(any()) mirrors it.
#[cfg(any())]
#[no_mangle]
pub unsafe extern "system" fn NhpAllocateAndGetInterfaceInfoFromStack(
    ppTable: *mut *mut IP_INTERFACE_NAME_INFO,
    pdwCount: PDWORD,
    bOrder: BOOL,
    hHeap: HANDLE,
    dwFlags: DWORD,
) -> DWORD {
    FIXME!(":stub\n");
    0
}
// ---- 58_GetIcmpStatisticsEx.rs ----
#[no_mangle]
pub unsafe extern "system" fn GetIcmpStatisticsEx(
    pStats: PMIB_ICMP_EX,
    dwFamily: DWORD,
) -> DWORD {
    if (*__wine_dbch___default).flags as i32
        & (1i32) << __WINE_DBCL_FIXME as i32
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_FIXME;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"/home/curi/Projects/os-playground/vibeos-research/dll/win32/iphlpapi/iphlpapi_main.c\0"
                as *const u8 as *const i8,
            b"GetIcmpStatisticsEx\0" as *const u8 as *const i8,
            3240i32,
            b":stub\n\0" as *const u8 as *const i8,
        );
    }

    if pStats.is_null() {
        return ERROR_INVALID_PARAMETER;
    }

    if dwFamily != AF_INET as u32 && dwFamily != AF_INET6 as u32 {
        return ERROR_INVALID_PARAMETER;
    }

    0u32
}
// ---- 59_SetIpForwardEntryToStack.rs ----
#[no_mangle]
pub unsafe extern "system" fn SetIpForwardEntryToStack(pRoute: PMIB_IPFORWARDROW) -> DWORD {
    FIXME!("SetIpForwardEntryToStack() stub\n");
    0
}
// ---- 60_GetInterfaceNameInternal.rs ----
unsafe fn GetInterfaceNameInternal(
    pInterfaceGUID: *const GUID,
    pInterfaceName: PWSTR,
    pOutBufLen: *mut u32,
) -> u32 {

    const HKEY_LOCAL_MACHINE: HKEY = 0x80000002 as HKEY;
    const MAX_INTERFACE_NAME_LEN: u32 = 256;

    let mut GuidString: UNICODE_STRING = UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut result: u32;
    let mut type_0: u32 = 0;
    let mut szKeyName: [u16; 520] = [0; 520];
    let mut hr: i32;
    let mut hKey: HKEY = core::ptr::null_mut();

    if pInterfaceGUID.is_null() || pOutBufLen.is_null() {
        return ERROR_INVALID_PARAMETER;
    }

    result = RtlStringFromGUID(pInterfaceGUID, core::ptr::addr_of_mut!(GuidString)) as u32;

    if !NT_SUCCESS(result as i32) {
        return RtlNtStatusToDosError(result as i32) as u32;
    }

    hr = StringCbPrintfW_1(
        szKeyName.as_mut_ptr(),
        core::mem::size_of::<[u16; 520]>(),
        win32macros_rust::wide_bytes!(b"S\0Y\0S\0T\0E\0M\0\\\0C\0u\0r\0r\0e\0n\0t\0C\0o\0n\0t\0r\0o\0l\0S\0e\0t\0\\\0C\0o\0n\0t\0r\0o\0l\0\\\0N\0e\0t\0w\0o\0r\0k\0\\\0{\x004\0D\x003\x006\0E\09\x007\x002\0-\0E\x003\x002\x005\0-\x001\x001\0C\0E\0-\0B\0F\0C\x001\0-\x000\08\x000\x000\x002\0B\0E\x001\x000\x003\x001\08\0}\0\\\0%\0s\0\\\0C\0o\0n\0n\0e\0c\0t\0i\0o\0n\0\0\0") as *const u16,
        GuidString.Buffer,
    );
    RtlFreeUnicodeString(core::ptr::addr_of_mut!(GuidString));

    if FAILED(hr) {
        return ERROR_BUFFER_OVERFLOW;
    }

    result = RegOpenKeyExW(
        HKEY_LOCAL_MACHINE,
        szKeyName.as_mut_ptr(),
        0,
        KEY_READ,
        core::ptr::addr_of_mut!(hKey),
    ) as u32;

    if result != ERROR_SUCCESS {
        return ERROR_NOT_FOUND;
    }

    result = RegQueryValueExW(
        hKey,
        win32macros_rust::wide_bytes!(b"N\0a\0m\0e\0\0\0") as *const u16,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(type_0),
        pInterfaceName as *mut u8,
        pOutBufLen,
    ) as u32;

    RegCloseKey(hKey);

    if result == ERROR_MORE_DATA {
        *pOutBufLen = MAX_INTERFACE_NAME_LEN * 2;
        return ERROR_INSUFFICIENT_BUFFER;
    }

    if result != ERROR_SUCCESS || type_0 != REG_SZ {
        return ERROR_NO_DATA;
    }

    ERROR_SUCCESS
}
// ---- 61_GetAdaptersInfo.rs ----






const MAX_ADAPTER_NAME_LENGTH: usize = 256;
const HKEY_CURRENT_USER: HKEY = 0x80000001 as HKEY;
const INADDR_NONE: u32 = 0xffffffff;
const INADDR_ANY: u32 = 0x00000000;
const TRUE: i32 = 1;
const FALSE: i32 = 0;
type BOOL = i32;

#[no_mangle]
pub unsafe extern "system" fn GetAdaptersInfo(
    pAdapterInfo: PIP_ADAPTER_INFO,
    pOutBufLen: PULONG,
) -> DWORD {
    let mut ret: DWORD;

    TRACE!((b"pAdapterInfo %p, pOutBufLen %p\n\0".as_ptr() as *const i8), pAdapterInfo, pOutBufLen);

    if pOutBufLen.is_null() {
        ret = ERROR_INVALID_PARAMETER;
    } else {
        let numNonLoopbackInterfaces: DWORD = getNumNonLoopbackInterfaces();

        if numNonLoopbackInterfaces > 0 {
            let mut size: u32 = (core::mem::size_of::<IP_ADAPTER_INFO>() as u32)
                .wrapping_mul(numNonLoopbackInterfaces);

            if pAdapterInfo.is_null() || *pOutBufLen < size {
                *pOutBufLen = size;
                ret = ERROR_BUFFER_OVERFLOW;
            } else {
                let table: *mut InterfaceIndexTable = getNonLoopbackInterfaceIndexTable();

                if !table.is_null() {
                    size = (core::mem::size_of::<IP_ADAPTER_INFO>() as u32)
                        .wrapping_mul((*table).numIndexes);
                    if *pOutBufLen < size {
                        *pOutBufLen = size;
                        ret = ERROR_INSUFFICIENT_BUFFER;
                    } else {
                        let mut ndx: DWORD;
                        let mut hKey: HKEY = core::ptr::null_mut();
                        let mut winsEnabled: BOOL = FALSE;
                        let mut primaryWINS: IP_ADDRESS_STRING = core::mem::zeroed();
                        let mut secondaryWINS: IP_ADDRESS_STRING = core::mem::zeroed();

                        memset(pAdapterInfo as *mut core::ffi::c_void, 0, size as usize);

                        if RegOpenKeyA(
                            HKEY_CURRENT_USER,
                            b"Software\\Wine\\Network\0".as_ptr() as *const i8,
                            &mut hKey,
                        ) == ERROR_SUCCESS as i32
                        {
                            let mut size_reg: DWORD = core::mem::size_of::<[u8; 16]>() as DWORD;
                            let mut addr: u32;

                            RegQueryValueExA(
                                hKey,
                                b"WinsServer\0".as_ptr() as *const i8,
                                core::ptr::null_mut(),
                                core::ptr::null_mut(),
                                primaryWINS.String.as_mut_ptr() as *mut u8,
                                &mut size_reg,
                            );
                            addr = inet_addr(primaryWINS.String.as_ptr());
                            if addr != INADDR_NONE && addr != INADDR_ANY {
                                winsEnabled = TRUE;
                            }
                            size_reg = core::mem::size_of::<[u8; 16]>() as DWORD;
                            RegQueryValueExA(
                                hKey,
                                b"BackupWinsServer\0".as_ptr() as *const i8,
                                core::ptr::null_mut(),
                                core::ptr::null_mut(),
                                secondaryWINS.String.as_mut_ptr() as *mut u8,
                                &mut size_reg,
                            );
                            addr = inet_addr(secondaryWINS.String.as_ptr());
                            if addr != INADDR_NONE && addr != INADDR_ANY {
                                winsEnabled = TRUE;
                            }
                            RegCloseKey(hKey);
                        }
                        TRACE!((b"num of index is %lu\n\0".as_ptr() as *const i8), (*table).numIndexes);
                        ndx = 0;
                        while ndx < (*table).numIndexes {
                            let ptr: PIP_ADAPTER_INFO = pAdapterInfo.offset(ndx as isize);
                            let mut addrLen: DWORD = core::mem::size_of::<[u8; 8]>() as DWORD;
                            let mut type_val: DWORD = 0;
                            let ifname: *const i8 = getInterfaceNameByIndex(
                                *(core::ptr::addr_of_mut!((*table).indexes) as *mut DWORD).offset(ndx as isize),
                            );
                            if ifname.is_null() {
                                ret = ERROR_OUTOFMEMORY;
                                break;
                            }

                            strncpy(
                                (*ptr).AdapterName.as_mut_ptr(),
                                ifname,
                                core::mem::size_of::<[u8; 260]>(),
                            );
                            consumeInterfaceName((ifname as *const u8));
                            (*ptr).AdapterName[MAX_ADAPTER_NAME_LENGTH] = 0;
                            getInterfacePhysicalByIndex(
                                *(core::ptr::addr_of_mut!((*table).indexes) as *mut DWORD).offset(ndx as isize),
                                &mut addrLen,
                                (*ptr).Address.as_mut_ptr(),
                                &mut type_val,
                            );
                            (*ptr).AddressLength = addrLen;
                            (*ptr).Type = type_val;
                            (*ptr).Index = *(core::ptr::addr_of_mut!((*table).indexes) as *mut DWORD).offset(ndx as isize);
                            toIPAddressString(
                                getInterfaceIPAddrByIndex(
                                    *(core::ptr::addr_of_mut!((*table).indexes) as *mut DWORD).offset(ndx as isize),
                                ),
                                (*ptr).IpAddressList.IpAddress.String.as_mut_ptr(),
                            );
                            toIPAddressString(
                                getInterfaceMaskByIndex(
                                    *(core::ptr::addr_of_mut!((*table).indexes) as *mut DWORD).offset(ndx as isize),
                                ),
                                (*ptr).IpAddressList.IpMask.String.as_mut_ptr(),
                            );
                            (*ptr).IpAddressList.Context = (*ptr).Index;
                            toIPAddressString(
                                getInterfaceGatewayByIndex(
                                    *(core::ptr::addr_of_mut!((*table).indexes) as *mut DWORD).offset(ndx as isize),
                                ),
                                (*ptr).GatewayList.IpAddress.String.as_mut_ptr(),
                            );
                            getDhcpInfoForAdapter(
                                *(core::ptr::addr_of_mut!((*table).indexes) as *mut DWORD).offset(ndx as isize),
                                ptr,
                            );
                            if winsEnabled != 0 {
                                (*ptr).HaveWins = TRUE;
                                memcpy(
                                    (*ptr).PrimaryWinsServer.IpAddress.String.as_mut_ptr() as *mut core::ffi::c_void,
                                    primaryWINS.String.as_ptr() as *const core::ffi::c_void,
                                    core::mem::size_of::<[u8; 16]>(),
                                );
                                memcpy(
                                    (*ptr).SecondaryWinsServer.IpAddress.String.as_mut_ptr() as *mut core::ffi::c_void,
                                    secondaryWINS.String.as_ptr() as *const core::ffi::c_void,
                                    core::mem::size_of::<[u8; 16]>(),
                                );
                            }
                            if ndx < (*table).numIndexes.wrapping_sub(1) {
                                (*ptr).Next = pAdapterInfo.offset(ndx.wrapping_add(1) as isize);
                            } else {
                                (*ptr).Next = core::ptr::null_mut();
                            }
                            ndx = ndx.wrapping_add(1);
                        }
                        ret = NO_ERROR;
                    }
                    free(table as *mut core::ffi::c_void);
                } else {
                    ret = ERROR_OUTOFMEMORY;
                }
            }
        } else {
            ret = ERROR_NO_DATA;
        }
    }
    TRACE!((b"returning %ld\n\0".as_ptr() as *const i8), ret);
    ret
}
// ---- 62_AllocateAndGetTcpExTableFromStack.rs ----
#[no_mangle]
pub unsafe extern "system" fn AllocateAndGetTcpExTableFromStack(
    ppTcpTable: *mut PVOID,
    bOrder: BOOL,
    heap: HANDLE,
    flags: DWORD,
    family: DWORD,
) -> DWORD {
    let mut ret: DWORD;

    TRACE!(
        (b"ppTcpTable %p, bOrder %ld, heap 0x%p, flags 0x%08lx, family 0x%08lx\n\0".as_ptr() as *const i8),
        ppTcpTable,
        bOrder as DWORD,
        heap,
        flags,
        family,
    );

    if ppTcpTable.is_null() {
        ret = ERROR_INVALID_PARAMETER;
    } else {
        let mut dwSize: DWORD = 0;

        *ppTcpTable = core::ptr::null_mut();
        ret = GetExtendedTcpTable(
            *ppTcpTable,
            core::ptr::addr_of_mut!(dwSize),
            bOrder,
            family as ULONG,
            TCP_TABLE_OWNER_PID_ALL,
            0,
        );
        if ret == ERROR_INSUFFICIENT_BUFFER {
            *ppTcpTable = HeapAlloc(heap, flags, dwSize as SIZE_T) as PVOID;
            ret = GetExtendedTcpTable(
                *ppTcpTable,
                core::ptr::addr_of_mut!(dwSize),
                bOrder,
                family as ULONG,
                TCP_TABLE_OWNER_PID_ALL,
                0,
            );
            if ret != NO_ERROR {
                HeapFree(heap, flags, *ppTcpTable);
                *ppTcpTable = core::ptr::null_mut();
            }
        }
    }

    TRACE!((b"returning %ld\n\0".as_ptr() as *const i8), ret);
    ret
}
// ---- 63_AllocateAndGetTcpExTable2FromStack.rs ----
#[no_mangle]
pub unsafe extern "system" fn AllocateAndGetTcpExTable2FromStack(
    ppTcpTable: *mut PVOID,
    bOrder: BOOL,
    heap: HANDLE,
    flags: DWORD,
    family: DWORD,
    class: TCP_TABLE_CLASS,
) -> DWORD {
    let mut ret: DWORD;

    if ppTcpTable.is_null() {
        ret = ERROR_INVALID_PARAMETER;
    } else {
        let mut dwSize: DWORD = 0;
        *ppTcpTable = core::ptr::null_mut();
        ret = GetExtendedTcpTable(
            *ppTcpTable,
            core::ptr::addr_of_mut!(dwSize),
            bOrder,
            family,
            class,
            0,
        );
        if ret == ERROR_INSUFFICIENT_BUFFER {
            *ppTcpTable = HeapAlloc(heap, flags, dwSize as SIZE_T);
            ret = GetExtendedTcpTable(
                *ppTcpTable,
                core::ptr::addr_of_mut!(dwSize),
                bOrder,
                family,
                class,
                0,
            );
            if ret != NO_ERROR {
                HeapFree(heap, flags, *ppTcpTable);
                *ppTcpTable = core::ptr::null_mut();
            }
        }
    }
    ret
}
// ---- 64_GetTcpTable.rs ----
#[no_mangle]
pub unsafe extern "system" fn GetTcpTable(
    pTcpTable: PMIB_TCPTABLE,
    pdwSize: PDWORD,
    bOrder: BOOL,
) -> DWORD {
    return GetExtendedTcpTable(
        pTcpTable as PVOID,
        pdwSize,
        bOrder,
        AF_INET as ULONG,
        TCP_TABLE_BASIC_ALL,
        0 as ULONG,
    );
}
// ---- 65_AllocateAndGetUdpExTableFromStack.rs ----
#[no_mangle]
pub unsafe extern "system" fn AllocateAndGetUdpExTableFromStack(
    ppUdpTable: *mut PVOID,
    bOrder: u8,
    heap: HANDLE,
    flags: DWORD,
    family: DWORD,
) -> DWORD {
    let mut ret: DWORD;

    if ppUdpTable.is_null() {
        ret = ERROR_INVALID_PARAMETER;
    } else {
        let mut dwSize: DWORD = 0;
        *ppUdpTable = core::ptr::null_mut();
        ret = GetExtendedUdpTable(
            *ppUdpTable,
            core::ptr::addr_of_mut!(dwSize),
            bOrder,
            family,
            UDP_TABLE_OWNER_PID,
            0,
        );
        if ret == ERROR_INSUFFICIENT_BUFFER {
            *ppUdpTable = HeapAlloc(heap, flags, dwSize as SIZE_T);
            ret = GetExtendedUdpTable(
                *ppUdpTable,
                core::ptr::addr_of_mut!(dwSize),
                bOrder,
                family,
                UDP_TABLE_OWNER_PID,
                0,
            );
            if ret != NO_ERROR {
                HeapFree(heap, flags, *ppUdpTable);
                *ppUdpTable = core::ptr::null_mut();
            }
        }
    }
    ret
}
// ---- 66_AllocateAndGetUdpExTable2FromStack.rs ----
#[no_mangle]
pub unsafe extern "system" fn AllocateAndGetUdpExTable2FromStack(
    ppUdpTable: *mut PVOID,
    bOrder: u8,
    heap: HANDLE,
    flags: DWORD,
    family: DWORD,
    class: UDP_TABLE_CLASS,
) -> DWORD {

    let mut ret: DWORD;

    if ppUdpTable.is_null() {
        ret = ERROR_INVALID_PARAMETER;
    } else {
        let mut dwSize: DWORD = 0;
        *ppUdpTable = core::ptr::null_mut();
        ret = GetExtendedUdpTable(*ppUdpTable, &mut dwSize, bOrder, family, class, 0);
        if ret == ERROR_INSUFFICIENT_BUFFER {
            *ppUdpTable = HeapAlloc(heap, flags, dwSize as SIZE_T);
            ret = GetExtendedUdpTable(*ppUdpTable, &mut dwSize, bOrder, family, class, 0);
            if ret != NO_ERROR {
                HeapFree(heap, flags, *ppUdpTable);
                *ppUdpTable = core::ptr::null_mut();
            }
        }
    }
    ret
}
// ---- 67_GetUdpTable.rs ----
#[no_mangle]
pub unsafe extern "system" fn GetUdpTable(
    pUdpTable: PMIB_UDPTABLE,
    pdwSize: PDWORD,
    bOrder: u8,
) -> DWORD {
    return GetExtendedUdpTable(
        pUdpTable as LPVOID,
        pdwSize,
        bOrder,
        AF_INET as ULONG,
        UDP_TABLE_BASIC,
        0 as ULONG,
    );
}
// ---- 68_GetIfTable.rs ----

#[no_mangle]
pub unsafe extern "system" fn GetIfTable(
    pIfTable: PMIB_IFTABLE,
    pdwSize: PULONG,
    bOrder: BOOL,
) -> DWORD {
    let mut ret: DWORD;

    TRACE!(
        (b"pIfTable %p, pdwSize %p, bOrder %ld\n\0".as_ptr() as *const i8),
        pdwSize,
        pdwSize,
        bOrder as DWORD,
    );

    if pdwSize.is_null() {
        ret = ERROR_INVALID_PARAMETER;
    } else {
        let numInterfaces: DWORD = getNumInterfaces();
        let mut size: ULONG;

        TRACE!(
            (b"GetIfTable: numInterfaces = %d\n\0".as_ptr() as *const i8),
            numInterfaces as i32,
        );

        size = (core::mem::size_of::<MIB_IFTABLE>()).wrapping_add(
            ((numInterfaces as usize).wrapping_sub(1))
                .wrapping_mul(core::mem::size_of::<MIB_IFROW>()),
        ) as ULONG;

        if pIfTable.is_null() || *pdwSize < size {
            *pdwSize = size;
            ret = ERROR_INSUFFICIENT_BUFFER;
        } else {
            let table: *mut InterfaceIndexTable = getInterfaceIndexTable();

            if !table.is_null() {
                size = (core::mem::size_of::<MIB_IFTABLE>()).wrapping_add(
                    (((*table).numIndexes as usize).wrapping_sub(1))
                        .wrapping_mul(core::mem::size_of::<MIB_IFROW>()),
                ) as ULONG;

                if *pdwSize < size {
                    *pdwSize = size;
                    ret = ERROR_INSUFFICIENT_BUFFER;
                } else {
                    let mut ndx: DWORD;

                    (*pIfTable).dwNumEntries = 0;
                    ndx = 0;
                    while ndx < (*table).numIndexes {
                        (*(core::ptr::addr_of_mut!((*pIfTable).table) as *mut MIB_IFROW)
                            .offset(ndx as isize))
                        .dwIndex = *(core::ptr::addr_of_mut!((*table).indexes) as *mut DWORD)
                            .offset(ndx as isize);
                        GetIfEntry(
                            core::ptr::addr_of_mut!((*pIfTable).table).offset(ndx as isize)
                                as PMIB_IFROW,
                        );
                        (*pIfTable).dwNumEntries = (*pIfTable).dwNumEntries.wrapping_add(1);
                        ndx = ndx.wrapping_add(1);
                    }

                    if bOrder != 0 {
                        qsort(
                            core::ptr::addr_of_mut!((*pIfTable).table) as *mut core::ffi::c_void,
                            (*pIfTable).dwNumEntries as usize,
                            core::mem::size_of::<MIB_IFROW>(),
                            Some(IfTableSorter),
                        );
                    }
                    ret = NO_ERROR;
                }
                free(table as *mut core::ffi::c_void);
            } else {
                ret = ERROR_OUTOFMEMORY;
            }
        }
    }

    TRACE!((b"returning %ld\n\0".as_ptr() as *const i8), ret);
    return ret;
}
// ---- 69_AllocateAndGetIpAddrTableFromStack.rs ----
#[no_mangle]
pub unsafe extern "system" fn AllocateAndGetIpAddrTableFromStack(
    ppIpAddrTable: *mut PMIB_IPADDRTABLE,
    bOrder: BOOL,
    heap: HANDLE,
    flags: DWORD,
) -> DWORD {

    let mut ret: DWORD;

    TRACE!(
        (b"ppIpAddrTable %p, bOrder %ld, heap 0x%p, flags 0x%08lx\n\0".as_ptr() as *const i8),
        ppIpAddrTable,
        bOrder as DWORD,
        heap,
        flags,
    );

    if ppIpAddrTable.is_null() {
        ret = ERROR_INVALID_PARAMETER;
    } else {
        let mut dwSize: DWORD = 0;

        *ppIpAddrTable = core::ptr::null_mut();
        ret = GetIpAddrTable(*ppIpAddrTable, core::ptr::addr_of_mut!(dwSize), bOrder);
        if ret == ERROR_INSUFFICIENT_BUFFER {
            *ppIpAddrTable = HeapAlloc(heap, flags, dwSize as SIZE_T) as PMIB_IPADDRTABLE;
            ret = GetIpAddrTable(*ppIpAddrTable, core::ptr::addr_of_mut!(dwSize), bOrder);
            if ret != NO_ERROR {
                HeapFree(heap, flags, *ppIpAddrTable as LPVOID);
                *ppIpAddrTable = core::ptr::null_mut();
            }
        }
    }

    TRACE!((b"returning %ld\n\0".as_ptr() as *const i8), ret);
    ret
}
// ---- 70_AllocateAndGetIpForwardTableFromStack.rs ----
#[no_mangle]
pub unsafe extern "system" fn AllocateAndGetIpForwardTableFromStack(
    ppIpForwardTable: *mut PMIB_IPFORWARDTABLE,
    bOrder: BOOL,
    heap: HANDLE,
    flags: DWORD,
) -> DWORD {
    let mut ret: DWORD;

    TRACE!(
        (b"ppIpForwardTable %p, bOrder %ld, heap 0x%p, flags 0x%08lx\n\0".as_ptr() as *const i8),
        ppIpForwardTable,
        bOrder as DWORD,
        heap,
        flags,
    );

    if ppIpForwardTable.is_null() {
        ret = ERROR_INVALID_PARAMETER;
    } else {
        let mut dwSize: DWORD = 0;

        *ppIpForwardTable = core::ptr::null_mut();
        ret = GetIpForwardTable(*ppIpForwardTable, core::ptr::addr_of_mut!(dwSize), bOrder);
        if ret == ERROR_INSUFFICIENT_BUFFER {
            *ppIpForwardTable = HeapAlloc(heap, flags, dwSize as SIZE_T) as PMIB_IPFORWARDTABLE;
            ret = GetIpForwardTable(*ppIpForwardTable, core::ptr::addr_of_mut!(dwSize), bOrder);
            if ret != NO_ERROR {
                HeapFree(heap, flags, *ppIpForwardTable as LPVOID);
                *ppIpForwardTable = core::ptr::null_mut();
            }
        }
    }

    TRACE!((b"returning %ld\n\0".as_ptr() as *const i8), ret);
    return ret;
}
// ---- 71_AllocateAndGetIpNetTableFromStack.rs ----
#[no_mangle]
pub unsafe extern "system" fn AllocateAndGetIpNetTableFromStack(
    ppIpNetTable: *mut PMIB_IPNETTABLE,
    bOrder: BOOL,
    heap: HANDLE,
    flags: DWORD,
) -> DWORD {

    let ret: DWORD;

    if ppIpNetTable.is_null() {
        ret = ERROR_INVALID_PARAMETER;
    } else {
        let mut dwSize: DWORD = 0;

        *ppIpNetTable = core::ptr::null_mut();
        ret = GetIpNetTable(*ppIpNetTable, &mut dwSize, bOrder);
        if ret == ERROR_INSUFFICIENT_BUFFER {
            *ppIpNetTable = HeapAlloc(heap, flags, dwSize as SIZE_T) as PMIB_IPNETTABLE;
            let ret2 = GetIpNetTable(*ppIpNetTable, &mut dwSize, bOrder);
            if ret2 != NO_ERROR {
                HeapFree(heap, flags, *ppIpNetTable as LPVOID);
                *ppIpNetTable = core::ptr::null_mut();
            }
            return ret2;
        }
    }

    ret
}
// ---- 72_GetIpStatistics.rs ----
#[no_mangle]
pub unsafe extern "system" fn GetIpStatistics(pStats: PMIB_IPSTATS) -> DWORD {
    return GetIpStatisticsEx(pStats, PF_INET as DWORD);
}
// ---- 73_GetOwnerModuleFromTcpEntry.rs ----
#[no_mangle]
pub unsafe extern "system" fn GetOwnerModuleFromTcpEntry(
    pTcpEntry: PMIB_TCPROW_OWNER_MODULE,
    Class: TCPIP_OWNER_MODULE_INFO_CLASS,
    Buffer: PVOID,
    pdwSize: PDWORD,
) -> DWORD {
    if (*pTcpEntry).OwningModuleInfo[0] != 0 {
        return GetOwnerModuleFromTagEntry(
            (*pTcpEntry).dwOwningPid,
            (*pTcpEntry).OwningModuleInfo[0] as DWORD,
            Class,
            Buffer,
            pdwSize,
        );
    } else {
        return GetOwnerModuleFromPidEntry((*pTcpEntry).dwOwningPid, Class, Buffer, pdwSize);
    }
}
// ---- 74_GetOwnerModuleFromUdpEntry.rs ----
#[no_mangle]
pub unsafe extern "system" fn GetOwnerModuleFromUdpEntry(
    pUdpEntry: PMIB_UDPROW_OWNER_MODULE,
    Class: TCPIP_OWNER_MODULE_INFO_CLASS,
    Buffer: PVOID,
    pdwSize: PDWORD,
) -> DWORD {
    if (*pUdpEntry).OwningModuleInfo[0] != 0 {
        return GetOwnerModuleFromTagEntry(
            (*pUdpEntry).dwOwningPid,
            (*pUdpEntry).OwningModuleInfo[0] as DWORD,
            Class,
            Buffer,
            pdwSize,
        );
    } else {
        return GetOwnerModuleFromPidEntry((*pUdpEntry).dwOwningPid, Class, Buffer, pdwSize);
    }
}
// ---- 75_GetTcpStatistics.rs ----
#[no_mangle]
pub unsafe extern "system" fn GetTcpStatistics(pStats: PMIB_TCPSTATS) -> DWORD {
    return GetTcpStatisticsEx(pStats, PF_INET as DWORD);
}
// ---- 76_GetUdpStatistics.rs ----
#[no_mangle]
pub unsafe extern "system" fn GetUdpStatistics(pStats: PMIB_UDPSTATS) -> DWORD {
    return GetUdpStatisticsEx(pStats, PF_INET as DWORD);
}
// ---- 77_NhGetInterfaceNameFromDeviceGuid.rs ----
#[no_mangle]
pub unsafe extern "system" fn NhGetInterfaceNameFromDeviceGuid(
    pInterfaceGUID: *const GUID,
    pInterfaceName: PWSTR,
    pOutBufLen: *mut u32,
    dwUnknown4: u32,
    dwUnknown5: u32,
) -> u32 {

    SetLastError(ERROR_SUCCESS);

    if pInterfaceName.is_null() {
        return ERROR_INVALID_PARAMETER;
    }

    GetInterfaceNameInternal(pInterfaceGUID, pInterfaceName, pOutBufLen)
}
// ---- 78_NhGetInterfaceNameFromGuid.rs ----
#[no_mangle]
pub unsafe extern "system" fn NhGetInterfaceNameFromGuid(
    pInterfaceGUID: *const GUID,
    pInterfaceName: PWSTR,
    pOutBufLen: *mut u32,
    dwUnknown4: u32,
    dwUnknown5: u32,
) -> u32 {

    let result = GetInterfaceNameInternal(pInterfaceGUID, pInterfaceName, pOutBufLen);

    if result == ERROR_NOT_FOUND {
        SetLastError(ERROR_PATH_NOT_FOUND);
    }

    result
}
// ---- NhGetGuidFromInterfaceName (added by the 2026-09-20 upstream sync) ----
//
// Upstream implemented this after our fork (iphlpapi_main.c) and exported it from
// iphlpapi.spec; their new `netsh interface ip` build-out (ifmon/ip.c, +2409 lines) and
// netsh.exe both call it, so on i386 the merge landed two modules that could not link.
// It is the inverse of NhGetInterfaceNameFromGuid above: walk the network-adapter class
// key, read each adapter's Connection\Name, and return the adapter GUID whose friendly
// name matches.
//
// Deliberately hand-parses the `{8-4-4-4-12}` subkey name instead of calling
// RtlGUIDFromString: that would pull UNICODE_STRING plumbing into this crate for one
// call site, and the subkey format here is fixed by the registry layout.
const NETWORK_CLASS_KEY: &[u16] = &[
    b'S' as u16, b'Y' as u16, b'S' as u16, b'T' as u16, b'E' as u16, b'M' as u16, b'\\' as u16,
    b'C' as u16, b'u' as u16, b'r' as u16, b'r' as u16, b'e' as u16, b'n' as u16, b't' as u16,
    b'C' as u16, b'o' as u16, b'n' as u16, b't' as u16, b'r' as u16, b'o' as u16, b'l' as u16,
    b'S' as u16, b'e' as u16, b't' as u16, b'\\' as u16,
    b'C' as u16, b'o' as u16, b'n' as u16, b't' as u16, b'r' as u16, b'o' as u16, b'l' as u16,
    b'\\' as u16,
    b'N' as u16, b'e' as u16, b't' as u16, b'w' as u16, b'o' as u16, b'r' as u16, b'k' as u16,
    b'\\' as u16,
    b'{' as u16, b'4' as u16, b'D' as u16, b'3' as u16, b'6' as u16, b'E' as u16, b'9' as u16,
    b'7' as u16, b'2' as u16, b'-' as u16, b'E' as u16, b'3' as u16, b'2' as u16, b'5' as u16,
    b'-' as u16, b'1' as u16, b'1' as u16, b'C' as u16, b'E' as u16, b'-' as u16,
    b'B' as u16, b'F' as u16, b'C' as u16, b'1' as u16, b'-' as u16,
    b'0' as u16, b'8' as u16, b'0' as u16, b'0' as u16, b'2' as u16, b'B' as u16,
    b'E' as u16, b'1' as u16, b'0' as u16, b'3' as u16, b'1' as u16, b'8' as u16,
    b'}' as u16, 0,
];
const CONNECTION_SUBKEY: &[u16] = &[
    b'C' as u16, b'o' as u16, b'n' as u16, b'n' as u16, b'e' as u16, b'c' as u16,
    b't' as u16, b'i' as u16, b'o' as u16, b'n' as u16, 0,
];
const NAME_VALUE: &[u16] = &[b'N' as u16, b'a' as u16, b'm' as u16, b'e' as u16, 0];

#[inline]
fn ascii_upper_u16(c: u16) -> u16 {
    if (b'a' as u16..=b'z' as u16).contains(&c) { c - 32 } else { c }
}

/// Case-insensitive compare of the first `n` code units. ASCII folding only, which is what
/// an adapter friendly name comparison needs and what keeps this free of NLS dependencies.
unsafe fn wcsnicmp_ascii(a: *const u16, b: *const u16, n: usize) -> bool {
    for i in 0..n {
        let (ca, cb) = (*a.add(i), *b.add(i));
        if ca == 0 || cb == 0 {
            return ca == cb;
        }
        if ascii_upper_u16(ca) != ascii_upper_u16(cb) {
            return false;
        }
    }
    true
}

/// Parse one hex digit; None on anything else.
#[inline]
fn hex_val(c: u16) -> Option<u32> {
    match c {
        c if (b'0' as u16..=b'9' as u16).contains(&c) => Some((c - b'0' as u16) as u32),
        c if (b'a' as u16..=b'f' as u16).contains(&c) => Some((c - b'a' as u16 + 10) as u32),
        c if (b'A' as u16..=b'F' as u16).contains(&c) => Some((c - b'A' as u16 + 10) as u32),
        _ => None,
    }
}

/// `{XXXXXXXX-XXXX-XXXX-XXXX-XXXXXXXXXXXX}` -> GUID. The braces are required, as they are
/// how the subkey names are actually written.
unsafe fn guid_from_braced(s: *const u16, out: *mut GUID) -> bool {
    let mut hex = [0u32; 32];
    let layout: [usize; 5] = [8, 4, 4, 4, 12];
    let mut i = 0usize; // index into s
    let mut h = 0usize; // index into hex
    if *s.add(i) != b'{' as u16 {
        return false;
    }
    i += 1;
    for (g, &len) in layout.iter().enumerate() {
        for _ in 0..len {
            match hex_val(*s.add(i)) {
                Some(v) => { hex[h] = v; h += 1; i += 1; }
                None => return false,
            }
        }
        if g < 4 {
            if *s.add(i) != b'-' as u16 { return false; }
            i += 1;
        }
    }
    if *s.add(i) != b'}' as u16 || *s.add(i + 1) != 0 {
        return false;
    }
    let nib = |from: usize, count: usize| -> u32 {
        let mut v = 0u32;
        for k in 0..count { v = (v << 4) | hex[from + k]; }
        v
    };
    (*out).Data1 = nib(0, 8);
    (*out).Data2 = nib(8, 4) as u16;
    (*out).Data3 = nib(12, 4) as u16;
    for b in 0..8usize {
        (*out).Data4[b] = nib(16 + b * 2, 2) as u8;
    }
    true
}

#[no_mangle]
pub unsafe extern "system" fn NhGetGuidFromInterfaceName(
    pInterfaceName: PWSTR,
    pInterfaceGUID: *mut GUID,
    _dwUnknown3: u32,
    _dwUnknown4: u32,
) -> u32 {
    const HKEY_LOCAL_MACHINE_L: HKEY = 0x80000002u32 as HKEY;

    if pInterfaceName.is_null() || pInterfaceGUID.is_null() {
        return ERROR_INVALID_PARAMETER;
    }

    let mut h_adapters: HKEY = core::ptr::null_mut();
    if RegOpenKeyExW(
        HKEY_LOCAL_MACHINE_L,
        NETWORK_CLASS_KEY.as_ptr(),
        0,
        KEY_READ,
        core::ptr::addr_of_mut!(h_adapters),
    ) as u32
        != ERROR_SUCCESS
    {
        return ERROR_NOT_FOUND;
    }

    let mut found = false;
    let mut index: u32 = 0;
    let mut guid_buf: [u16; 45] = [0; 45];

    loop {
        let mut name_size: u32 = guid_buf.len() as u32;
        if RegEnumKeyExW(
            h_adapters,
            index,
            guid_buf.as_mut_ptr(),
            core::ptr::addr_of_mut!(name_size),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        ) as u32
            != ERROR_SUCCESS
        {
            break;
        }
        index += 1;

        let mut h_adapter: HKEY = core::ptr::null_mut();
        if RegOpenKeyExW(
            h_adapters,
            guid_buf.as_ptr(),
            0,
            KEY_READ,
            core::ptr::addr_of_mut!(h_adapter),
        ) as u32
            != ERROR_SUCCESS
        {
            continue;
        }

        let mut h_conn: HKEY = core::ptr::null_mut();
        if RegOpenKeyExW(
            h_adapter,
            CONNECTION_SUBKEY.as_ptr(),
            0,
            KEY_READ,
            core::ptr::addr_of_mut!(h_conn),
        ) as u32
            == ERROR_SUCCESS
        {
            let mut value_size: u32 = 0;
            if RegQueryValueExW(
                h_conn,
                NAME_VALUE.as_ptr(),
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                core::ptr::addr_of_mut!(value_size),
            ) as u32
                == ERROR_SUCCESS
                && value_size >= core::mem::size_of::<u16>() as u32
            {
                let buf = HeapAlloc(GetProcessHeap(), 0, value_size as _) as *mut u16;
                if !buf.is_null() {
                    if RegQueryValueExW(
                        h_conn,
                        NAME_VALUE.as_ptr(),
                        core::ptr::null_mut(),
                        core::ptr::null_mut(),
                        buf as *mut u8,
                        core::ptr::addr_of_mut!(value_size),
                    ) as u32
                        == ERROR_SUCCESS
                    {
                        // NUL-terminate defensively: REG_SZ is not guaranteed to be.
                        let last = (value_size as usize / core::mem::size_of::<u16>())
                            .saturating_sub(1);
                        *buf.add(last) = 0;
                        let n = wcslen(buf);
                        if n != 0 && wcsnicmp_ascii(pInterfaceName, buf, n) {
                            found = guid_from_braced(guid_buf.as_ptr(), pInterfaceGUID);
                        }
                    }
                    HeapFree(GetProcessHeap(), 0, buf as *mut c_void);
                }
            }
            RegCloseKey(h_conn);
        }
        RegCloseKey(h_adapter);

        if found {
            break;
        }
    }

    RegCloseKey(h_adapters);

    if found { ERROR_SUCCESS } else { ERROR_NOT_FOUND }
}

// ---- 79_AllocateAndGetTcpTableFromStack.rs ----
#[no_mangle]
pub unsafe extern "system" fn AllocateAndGetTcpTableFromStack(
    ppTcpTable: *mut PMIB_TCPTABLE,
    bOrder: BOOL,
    heap: HANDLE,
    flags: DWORD,
) -> DWORD {

    let ret: DWORD;

    if ppTcpTable.is_null() {
        ret = ERROR_INVALID_PARAMETER;
    } else {
        let mut dwSize: DWORD = 0;

        *ppTcpTable = core::ptr::null_mut();
        ret = GetTcpTable(*ppTcpTable, &mut dwSize, bOrder);
        if ret == ERROR_INSUFFICIENT_BUFFER {
            *ppTcpTable = HeapAlloc(heap, flags, dwSize as SIZE_T) as PMIB_TCPTABLE;
            let ret2 = GetTcpTable(*ppTcpTable, &mut dwSize, bOrder);
            if ret2 != NO_ERROR {
                HeapFree(heap, flags, *ppTcpTable as LPVOID);
                *ppTcpTable = core::ptr::null_mut();
            }
            return ret2;
        }
    }

    ret
}
// ---- 80_AllocateAndGetUdpTableFromStack.rs ----

#[no_mangle]
pub unsafe extern "system" fn AllocateAndGetUdpTableFromStack(
    ppUdpTable: *mut PMIB_UDPTABLE,
    bOrder: u8,
    heap: HANDLE,
    flags: DWORD,
) -> DWORD {
    let mut ret: DWORD;

    TRACE!(
        (b"ppUdpTable %p, bOrder %ld, heap 0x%p, flags 0x%08lx\n\0".as_ptr() as *const i8),
        ppUdpTable,
        bOrder as DWORD,
        heap,
        flags,
    );

    if ppUdpTable.is_null() {
        ret = ERROR_INVALID_PARAMETER;
    } else {
        let mut dwSize: DWORD = 0;

        *ppUdpTable = core::ptr::null_mut();
        ret = GetUdpTable(*ppUdpTable, core::ptr::addr_of_mut!(dwSize), bOrder);
        if ret == ERROR_INSUFFICIENT_BUFFER {
            *ppUdpTable = HeapAlloc(heap, flags, dwSize as SIZE_T) as PMIB_UDPTABLE;
            ret = GetUdpTable(*ppUdpTable, core::ptr::addr_of_mut!(dwSize), bOrder);
            if ret != NO_ERROR {
                HeapFree(heap, flags, *ppUdpTable as LPVOID);
                *ppUdpTable = core::ptr::null_mut();
            }
        }
    }

    TRACE!((b"returning %ld\n\0".as_ptr() as *const i8), ret);
    ret
}
// ---- 81_AllocateAndGetIfTableFromStack.rs ----
#[no_mangle]
pub unsafe extern "system" fn AllocateAndGetIfTableFromStack(
    ppIfTable: *mut PMIB_IFTABLE,
    bOrder: BOOL,
    heap: HANDLE,
    flags: DWORD,
) -> DWORD {
    let mut ret: DWORD;

    TRACE!(
        (b"ppIfTable %p, bOrder %ld, heap 0x%p, flags 0x%08lx\n\0".as_ptr() as *const i8),
        ppIfTable,
        bOrder as DWORD,
        heap,
        flags,
    );

    if ppIfTable.is_null() {
        ret = ERROR_INVALID_PARAMETER;
    } else {
        let mut dwSize: DWORD = 0;

        *ppIfTable = core::ptr::null_mut();
        ret = GetIfTable(*ppIfTable, core::ptr::addr_of_mut!(dwSize), bOrder);
        if ret == ERROR_INSUFFICIENT_BUFFER {
            *ppIfTable = HeapAlloc(heap, flags, dwSize as SIZE_T) as PMIB_IFTABLE;
            ret = GetIfTable(*ppIfTable, core::ptr::addr_of_mut!(dwSize), bOrder);
            if ret != NO_ERROR {
                HeapFree(heap, flags, *ppIfTable as LPVOID);
                *ppIfTable = core::ptr::null_mut();
            }
        }
    }

    TRACE!((b"returning %ld\n\0".as_ptr() as *const i8), ret);
    ret
}
// ---- 82_GetBestRoute.rs ----
#[no_mangle]
pub unsafe extern "system" fn GetBestRoute(
    dwDestAddr: DWORD,
    dwSourceAddr: DWORD,
    pBestRoute: PMIB_IPFORWARDROW,
) -> DWORD {

    let mut table: PMIB_IPFORWARDTABLE = core::ptr::null_mut();
    let ret: DWORD;

    if pBestRoute.is_null() {
        return ERROR_INVALID_PARAMETER;
    }

    AllocateAndGetIpForwardTableFromStack(
        core::ptr::addr_of_mut!(table),
        FALSE,
        GetProcessHeap(),
        0,
    );

    if !table.is_null() {
        let mut ndx: DWORD = 0;
        let mut minMaskSize: DWORD = 255;
        let mut matchedNdx: DWORD = 0;

        while ndx < (*table).dwNumEntries {
            let table_entry = (core::ptr::addr_of_mut!((*table).table) as *mut MIB_IPFORWARDROW).offset(ndx as isize);

            if (dwDestAddr & (*table_entry).dwForwardMask)
                == ((*table_entry).dwForwardDest & (*table_entry).dwForwardMask)
            {
                let mut hostMaskSize: DWORD = 0;

                if _BitScanForward(
                    core::ptr::addr_of_mut!(hostMaskSize) as *mut ::core::ffi::c_ulong,
                    ntohl((*table_entry).dwForwardMask as ::core::ffi::c_ulong),
                ) == 0
                {
                    hostMaskSize = 32;
                }

                if hostMaskSize < minMaskSize {
                    minMaskSize = hostMaskSize;
                    matchedNdx = ndx;
                }
            }
            ndx = ndx.wrapping_add(1);
        }

        let matched_entry = (core::ptr::addr_of_mut!((*table).table) as *mut MIB_IPFORWARDROW).offset(matchedNdx as isize);
        core::ptr::copy_nonoverlapping(
            matched_entry,
            pBestRoute,
            1,
        );

        HeapFree(GetProcessHeap(), 0, table as LPVOID);
        ret = ERROR_SUCCESS;
    } else {
        ret = ERROR_OUTOFMEMORY;
    }

    ret
}
// ---- 83_GetBestInterface.rs ----
#[no_mangle]
pub unsafe extern "system" fn GetBestInterface(
    dwDestAddr: IPAddr,
    pdwBestIfIndex: PDWORD,
) -> DWORD {
    let mut ret: DWORD;

    TRACE!((b"dwDestAddr 0x%08lx, pdwBestIfIndex %p\n\0".as_ptr() as *const i8), dwDestAddr, pdwBestIfIndex);

    if pdwBestIfIndex.is_null() {
        ret = ERROR_INVALID_PARAMETER;
    } else {
        let mut ipRow: MIB_IPFORWARDROW = core::mem::zeroed();

        ret = GetBestRoute(dwDestAddr, 0, &mut ipRow);
        if ret == ERROR_SUCCESS {
            *pdwBestIfIndex = ipRow.dwForwardIfIndex;
        }
    }

    TRACE!((b"returning %ld\n\0".as_ptr() as *const i8), ret);
    ret
}
