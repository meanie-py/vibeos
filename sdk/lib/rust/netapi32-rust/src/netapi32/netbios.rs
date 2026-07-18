use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
pub const ALL_TRANSPORTS: [::core::ffi::c_char; 5] =
    unsafe { ::core::mem::transmute::<[u8; 5], [::core::ffi::c_char; 5]>(*b"M\0\0\0\0") };

static mut __wine_dbch___default: *mut __wine_debug_channel =
    unsafe { &raw const __wine_dbch_netbios as *mut __wine_debug_channel };

static mut __wine_dbch_netbios: __wine_debug_channel = unsafe {
    __wine_debug_channel {
        flags: WINE_DBG_DEFAULT_FLAGS,
        name: ::core::mem::transmute::<[u8; 15], [::core::ffi::c_char; 15]>(
            *b"netbios\0\0\0\0\0\0\0\0",
        ),
    }
};

static mut gTransports: [NetBIOSTransportTableEntry; 1] = [NetBIOSTransportTableEntry {
    id: 0,
    transport: NetBIOSTransport {
        enumerate: None,
        astat: None,
        findName: None,
        call: None,
        send: None,
        recv: None,
        hangup: None,
        cleanupAdapter: None,
        cleanup: None,
    },
}; 1];

static mut gNumTransports: UCHAR = 0 as UCHAR;

static mut gNBTable: NetBIOSAdapterTable = NetBIOSAdapterTable {
    cs: CRITICAL_SECTION {
        DebugInfo: ::core::ptr::null::<_RTL_CRITICAL_SECTION_DEBUG>()
            as *mut _RTL_CRITICAL_SECTION_DEBUG,
        LockCount: 0,
        RecursionCount: 0,
        OwningThread: ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void,
        LockSemaphore: ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void,
        SpinCount: 0,
    },
    enumerated: 0,
    enumerating: 0,
    tableSize: 0,
    table: ::core::ptr::null::<NetBIOSAdapter>() as *mut NetBIOSAdapter,
};

pub const ADAPTERS_INCR: u32 = 8;

pub const DEFAULT_NUM_SESSIONS: u32 = 16;

#[repr(C)]
pub struct _NetBIOSTransportTableEntry {
    pub id: ULONG,
    pub transport: NetBIOSTransport,
}
pub type NetBIOSTransportTableEntry = _NetBIOSTransportTableEntry;
pub type PNetBIOSTransportTableEntry = *mut _NetBIOSTransportTableEntry;

#[repr(C)]
pub struct _NetBIOSSession {
    pub inUse: BOOL,
    pub state: UCHAR,
    pub local_name: [UCHAR; NCBNAMSZ as usize],
    pub remote_name: [UCHAR; NCBNAMSZ as usize],
    pub data: *mut core::ffi::c_void,
}
pub type NetBIOSSession = _NetBIOSSession;
pub type PNetBIOSSession = *mut _NetBIOSSession;

#[repr(C)]
pub struct _NetBIOSAdapter {
    pub enabled: BOOL,
    pub shuttingDown: BOOL,
    pub resetting: LONG,
    pub transport_id: ULONG,
    pub transport: *mut NetBIOSTransport,
    pub impl_: NetBIOSAdapterImpl,
    pub cmdQueue: *mut NBCmdQueue,
    pub cs: CRITICAL_SECTION,
    pub sessionsLen: DWORD,
    pub sessions: *mut NetBIOSSession,
}
pub type NetBIOSAdapter = _NetBIOSAdapter;
pub type PNetBIOSAdapter = *mut _NetBIOSAdapter;

#[repr(C)]
pub struct _NetBIOSAdapterTable {
    pub cs: CRITICAL_SECTION,
    pub enumerated: BOOL,
    pub enumerating: BOOL,
    pub tableSize: UCHAR,
    pub table: *mut NetBIOSAdapter,
}
pub type NetBIOSAdapterTable = _NetBIOSAdapterTable;
pub type PNetBIOSAdapterTable = *mut _NetBIOSAdapterTable;
unsafe extern "C" fn nbResizeAdapterTable(newSize: u8) -> u8 {
    let ret: u8;
    if !gNBTable.table.is_null() {
        gNBTable.table = HeapReAlloc(
            GetProcessHeap(),
            HEAP_ZERO_MEMORY,
            gNBTable.table as *mut c_void,
            (newSize as usize).wrapping_mul(core::mem::size_of::<_NetBIOSAdapter>()) as SIZE_T,
        ) as *mut _NetBIOSAdapter;
    } else {
        gNBTable.table = HeapAlloc(
            GetProcessHeap(),
            HEAP_ZERO_MEMORY,
            (newSize as usize).wrapping_mul(core::mem::size_of::<_NetBIOSAdapter>()) as SIZE_T,
        ) as *mut _NetBIOSAdapter;
    }
    if !gNBTable.table.is_null() {
        gNBTable.tableSize = newSize;
        ret = NRC_GOODRET;
    } else {
        ret = NRC_OSRESNOTAV;
    }
    ret
}
#[no_mangle]
pub unsafe extern "C" fn NetBIOSInit() {
    core::ptr::write_bytes(
        core::ptr::addr_of_mut!(gNBTable) as *mut u8,
        0u8,
        core::mem::size_of::<NetBIOSAdapterTable>(),
    );
    InitializeCriticalSection(core::ptr::addr_of_mut!(gNBTable.cs));
    (*gNBTable.cs.DebugInfo).__bindgen_anon_1.Spare[0] = b"dll/win32/netapi32/netbios.c: NetBIOSAdapterTable.cs\0".as_ptr() as usize as DWORD_PTR;
}
#[no_mangle]
pub unsafe extern "C" fn NetBIOSShutdown() {
    let mut i: u8 = 0;
    EnterCriticalSection(&raw mut gNBTable.cs);
    i = 0;
    while i < gNBTable.tableSize {
        let adapter = gNBTable.table.offset(i as isize);
        if !(*adapter).transport.is_null() {
            if let Some(cleanup_adapter) = (*(*adapter).transport).cleanupAdapter {
                cleanup_adapter((*adapter).impl_.data);
            }
        }
        i = i.wrapping_add(1);
    }
    i = 0;
    while (i as usize) < gNumTransports as usize {
        if let Some(cleanup) = gTransports[i as usize].transport.cleanup {
            cleanup();
        }
        i = i.wrapping_add(1);
    }
    LeaveCriticalSection(&raw mut gNBTable.cs);
    (*gNBTable.cs.DebugInfo).__bindgen_anon_1.Spare[0] = 0 as DWORD_PTR;
    DeleteCriticalSection(&raw mut gNBTable.cs);
    HeapFree(
        GetProcessHeap(),
        0,
        gNBTable.table as *mut core::ffi::c_void,
    );
}
#[no_mangle]
pub unsafe extern "C" fn NetBIOSRegisterTransport(
    id: ULONG,
    transport: *mut NetBIOSTransport,
) -> BOOL {
    let mut ret: BOOL = 0i32;
    if (*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_TRACE as u32)) != 0 {
        ros_dbg_log(
            __WINE_DBCL_TRACE,
            __wine_dbch___default,
            b"dll/win32/netapi32/netbios.c\0"
                .as_ptr() as *const core::ffi::c_char,
            b"NetBIOSRegisterTransport\0".as_ptr() as *const core::ffi::c_char,
            135i32,
            b"transport 0x%08x, p %p\n\0".as_ptr() as *const core::ffi::c_char,
            id as core::ffi::c_uint,
            transport as *const core::ffi::c_void,
        );
    }
    if transport.is_null() {
        ret = 0i32;
    } else if (gNumTransports as usize) >= (core::mem::size_of::<[NetBIOSTransportTableEntry; 1]>()
        / core::mem::size_of::<NetBIOSTransportTableEntry>())
    {
        if (*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_FIXME as u32)) != 0 {
            ros_dbg_log(
                __WINE_DBCL_FIXME,
                __wine_dbch___default,
                b"dll/win32/netapi32/netbios.c\0"
                    .as_ptr() as *const core::ffi::c_char,
                b"NetBIOSRegisterTransport\0".as_ptr() as *const core::ffi::c_char,
                140i32,
                b"Too many transports %d\n\0".as_ptr() as *const core::ffi::c_char,
                gNumTransports as i32 + 1i32,
            );
        }
        ret = 0i32;
    } else {
        let mut i: UCHAR = 0u8;
        ret = 0i32;
        i = 0u8;
        while ret == 0 && i < gNumTransports {
            if gTransports[i as usize].id == id {
                if ((*__wine_dbch___default).flags as i32
                    & (1i32 << __WINE_DBCL_WARN as i32))
                    != 0
                {
                    ros_dbg_log(
                        __WINE_DBCL_WARN,
                        __wine_dbch___default,
                        b"dll/win32/netapi32/netbios.c\0"
                            .as_ptr() as *const core::ffi::c_char,
                        b"NetBIOSRegisterTransport\0".as_ptr() as *const core::ffi::c_char,
                        152i32,
                        b"Replacing NetBIOS transport ID %d\n\0".as_ptr()
                            as *const core::ffi::c_char,
                        id as core::ffi::c_uint,
                    );
                }
                memcpy(
                    core::ptr::addr_of_mut!(gTransports[i as usize].transport)
                        as *mut core::ffi::c_void,
                    transport as *const core::ffi::c_void,
                    core::mem::size_of::<NetBIOSTransport>() as usize,
                );
                ret = 1i32;
            }
            i = i.wrapping_add(1);
        }
        if ret == 0 {
            gTransports[gNumTransports as usize].id = id;
            memcpy(
                core::ptr::addr_of_mut!(gTransports[gNumTransports as usize].transport)
                    as *mut core::ffi::c_void,
                transport as *const core::ffi::c_void,
                core::mem::size_of::<NetBIOSTransport>() as usize,
            );
            gNumTransports = gNumTransports.wrapping_add(1);
            ret = 1i32;
        }
    }
    if (*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_TRACE as u32)) != 0 {
        ros_dbg_log(
            __WINE_DBCL_TRACE,
            __wine_dbch___default,
            b"dll/win32/netapi32/netbios.c\0"
                .as_ptr() as *const core::ffi::c_char,
            b"NetBIOSRegisterTransport\0".as_ptr() as *const core::ffi::c_char,
            167i32,
            b"returning %d\n\0".as_ptr() as *const core::ffi::c_char,
            ret,
        );
    }
    ret
}

#[no_mangle]
pub unsafe extern "C" fn NetBIOSRegisterAdapter(
    transport: u32,
    ifIndex: u32,
    data: *mut core::ffi::c_void,
) -> i32 {
    let mut ret: i32;
    let mut i: u8 = 0;

    // Debug log: entry
    if unsafe { (*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_TRACE as u32)) != 0 } {
        unsafe {
            ros_dbg_log(
                __WINE_DBCL_TRACE,
                __wine_dbch___default,
                b"dll/win32/netapi32/netbios.c\0"
                    as *const u8 as *const core::ffi::c_char,
                b"NetBIOSRegisterAdapter\0" as *const u8 as *const core::ffi::c_char,
                181i32,
                b": transport 0x%08x, ifIndex 0x%08x, data %p\n\0" as *const u8
                    as *const core::ffi::c_char,
                transport,
                ifIndex,
                data,
            );
        }
    }

    i = 0;
    while (i as i32) < (gNumTransports as i32)
        && unsafe { (*gTransports.as_ptr().add(i as usize)).id } != transport
    {
        i = i.wrapping_add(1);
    }

    if (i as i32) < (gNumTransports as i32)
        && unsafe { (*gTransports.as_ptr().add(i as usize)).id } == transport
    {
        let transportPtr = unsafe {
            &raw mut gTransports[i as usize].transport
                as *mut NetBIOSTransport
        };

        // Debug log: found transport
        if unsafe { (*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_TRACE as u32)) != 0 } {
            unsafe {
                ros_dbg_log(
                    __WINE_DBCL_TRACE,
                    __wine_dbch___default,
                    b"dll/win32/netapi32/netbios.c\0"
                        as *const u8 as *const core::ffi::c_char,
                    b"NetBIOSRegisterAdapter\0" as *const u8 as *const core::ffi::c_char,
                    188i32,
                    b": found transport %p for id 0x%08x\n\0" as *const u8
                        as *const core::ffi::c_char,
                    transportPtr,
                    transport,
                );
            }
        }

        unsafe { EnterCriticalSection(&raw mut gNBTable.cs) };
        ret = 0i32; // FALSE
        i = 0;
        while (i as i32) < (gNBTable.tableSize as i32)
            && !unsafe { (*gNBTable.table.add(i as usize)).transport.is_null() }
        {
            i = i.wrapping_add(1);
        }

        if (i as i32) == (gNBTable.tableSize as i32)
            && (gNBTable.tableSize as i32) < (MAX_LANA as i32 + 1i32)
        {
            let mut newSize: u8;
            if (gNBTable.tableSize as i32) < (MAX_LANA as i32 + 1i32) - (ADAPTERS_INCR as i32) {
                newSize = (gNBTable.tableSize as i32 + ADAPTERS_INCR as i32) as u8;
            } else {
                newSize = (MAX_LANA as i32 + 1i32) as u8;
            }
            nbResizeAdapterTable(newSize);
        }

        if (i as i32) < (gNBTable.tableSize as i32)
            && unsafe { (*gNBTable.table.add(i as usize)).transport.is_null() }
        {
            // Debug log: registering as LANA
            if unsafe { (*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_TRACE as u32)) != 0 } {
                unsafe {
                    ros_dbg_log(
                        __WINE_DBCL_TRACE,
                        __wine_dbch___default,
                        b"dll/win32/netapi32/netbios.c\0"
                            as *const u8 as *const core::ffi::c_char,
                        b"NetBIOSRegisterAdapter\0" as *const u8 as *const core::ffi::c_char,
                        207i32,
                        b": registering as LANA %d\n\0" as *const u8
                            as *const core::ffi::c_char,
                        i as i32,
                    );
                }
            }

            unsafe {
                let entry = &mut *gNBTable.table.add(i as usize);
                entry.transport_id = transport;
                entry.transport = transportPtr;
                entry.impl_.lana = i;
                entry.impl_.ifIndex = ifIndex;
                entry.impl_.data = data;
                entry.cmdQueue = NBCmdQueueCreate(GetProcessHeap());
                InitializeCriticalSection(&raw mut entry.cs);
                (*entry.cs.DebugInfo).__bindgen_anon_1.Spare[0usize] =
                    b"dll/win32/netapi32/netbios.c: NetBIOSAdapterTable.NetBIOSAdapter.cs\0"
                        as *const u8 as *const core::ffi::c_char as usize as DWORD_PTR;
                entry.enabled = 1i32; // TRUE
            }
            ret = 1i32; // TRUE
        }

        unsafe { LeaveCriticalSection(&raw mut gNBTable.cs) };
    } else {
        ret = 0i32; // FALSE
    }

    // Debug log: return
    if unsafe { (*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_TRACE as u32)) != 0 } {
        unsafe {
            ros_dbg_log(
                __WINE_DBCL_TRACE,
                __wine_dbch___default,
                b"dll/win32/netapi32/netbios.c\0"
                    as *const u8 as *const core::ffi::c_char,
                b"NetBIOSRegisterAdapter\0" as *const u8 as *const core::ffi::c_char,
                223i32,
                b"returning %d\n\0" as *const u8 as *const core::ffi::c_char,
                ret,
            );
        }
    }

    ret
}
#[no_mangle]
pub unsafe extern "C" fn NetBIOSEnableAdapter(lana: UCHAR) {
    if (unsafe { (*__wine_dbch___default).flags } as i32
        & (1 << __WINE_DBCL_TRACE as i32)
        != 0)
    {
        let dbch: *mut __wine_debug_channel = unsafe { __wine_dbch___default };
        ros_dbg_log(
            __WINE_DBCL_TRACE,
            dbch,
            b"dll/win32/netapi32/netbios.c\0"
                .as_ptr() as *const i8,
            b"NetBIOSEnableAdapter\0".as_ptr() as *const i8,
            233,
            b": %d\n\0".as_ptr() as *const i8,
            lana as i32,
        );
    }
    if (lana as usize) < unsafe { gNBTable.tableSize } as usize {
        unsafe { EnterCriticalSection(&raw mut gNBTable.cs) };
        if !unsafe { (*gNBTable.table.offset(lana as isize)).transport.is_null() } {
            unsafe { (*gNBTable.table.offset(lana as isize)).enabled = TRUE as BOOL };
        }
        unsafe { LeaveCriticalSection(&raw mut gNBTable.cs) };
    }
}
unsafe extern "C" fn nbShutdownAdapter(adapter: *mut _NetBIOSAdapter) {
    if !adapter.is_null() {
        (*adapter).shuttingDown = TRUE;
        NBCmdQueueCancelAll((*adapter).cmdQueue);
        if let Some(cleanup_adapter) = (*(*adapter).transport).cleanupAdapter {
            cleanup_adapter((*adapter).impl_.data);
        }
        NBCmdQueueDestroy((*adapter).cmdQueue);
        (*(*adapter).cs.DebugInfo).__bindgen_anon_1.Spare[0] = 0;
        DeleteCriticalSection(&raw mut (*adapter).cs);
        memset(
            adapter as *mut core::ffi::c_void,
            0,
            core::mem::size_of::<_NetBIOSAdapter>(),
        );
    }
}
unsafe extern "C" fn nbInternalEnum() {
    let mut i: u8;
    EnterCriticalSection(&raw mut gNBTable.cs);

    // mark:
    i = 0;
    while i < gNBTable.tableSize {
        let ptr = gNBTable.table.offset(i as isize);
        if (*ptr).enabled != 0 && !(*ptr).transport.is_null() {
            (*ptr).enabled = 0;
        }
        i = i.wrapping_add(1);
    }

    // store adapters:
    i = 0;
    while i < gNumTransports {
        if let Some(enumerate) = gTransports[i as usize].transport.enumerate {
            enumerate();
        }
        i = i.wrapping_add(1);
    }

    // sweep:
    i = 0;
    while i < gNBTable.tableSize {
        let ptr = gNBTable.table.offset(i as isize);
        if (*ptr).enabled == 0 && !(*ptr).transport.is_null() {
            nbShutdownAdapter(ptr);
        }
        i = i.wrapping_add(1);
    }

    gNBTable.enumerated = 1; // TRUE
    LeaveCriticalSection(&raw mut gNBTable.cs);
}
#[no_mangle]
pub unsafe extern "C" fn NetBIOSNumAdapters() -> UCHAR {
    if gNBTable.enumerated == 0 {
        nbInternalEnum();
    }
    let mut ret: UCHAR = 0;
    let table_size = gNBTable.tableSize;
    for i in 0..table_size {
        if !(*gNBTable.table.offset(i as isize)).transport.is_null() {
            ret = ret.wrapping_add(1);
        }
    }
    ret
}
#[no_mangle]
pub unsafe extern "C" fn NetBIOSEnumAdapters(
    transport: ULONG,
    cb: NetBIOSEnumAdaptersCallback,
    closure: *mut core::ffi::c_void,
) {
    if (*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_TRACE as u32)) != 0 {
        ros_dbg_log(
            __WINE_DBCL_TRACE,
            __wine_dbch___default,
            b"dll/win32/netapi32/netbios.c\0"
                .as_ptr() as *const i8,
            b"NetBIOSEnumAdapters\0".as_ptr() as *const i8,
            300i32,
            b"transport 0x%08x, callback %p, closure %p\n\0".as_ptr() as *const i8,
            transport,
            cb,
            closure,
        );
    }

    if let Some(cb) = cb {
        let enumAll: i32 = (memcmp(
            &transport as *const ULONG as *const core::ffi::c_void,
            ALL_TRANSPORTS.as_ptr() as *const core::ffi::c_void,
            core::mem::size_of::<ULONG>(),
        ) == 0) as i32;

        let mut numLANAs: u8 = 0;
        EnterCriticalSection(&raw mut gNBTable.cs);
        if gNBTable.enumerating == 0 {
            gNBTable.enumerating = TRUE;
            nbInternalEnum();
            gNBTable.enumerating = FALSE;
        }

        for i in 0..gNBTable.tableSize {
            let adapter_ptr = gNBTable.table.add(i as usize);
            if enumAll != 0 || (*adapter_ptr).transport_id == transport {
                numLANAs = numLANAs.wrapping_add(1);
            }
        }

        if numLANAs > 0 {
            let mut lanaIndex: u8 = 0;
            for i in 0..gNBTable.tableSize {
                let adapter_ptr = gNBTable.table.add(i as usize);
                if (*adapter_ptr).transport_id != 0
                    && (enumAll != 0 || (*adapter_ptr).transport_id == transport)
                {
                    let index = lanaIndex;
                    lanaIndex = lanaIndex.wrapping_add(1);
                    cb(
                        numLANAs,
                        index,
                        (*adapter_ptr).transport_id,
                        &raw mut (*adapter_ptr).impl_,
                        closure,
                    );
                }
            }
        }
        LeaveCriticalSection(&raw mut gNBTable.cs);
    }
}
unsafe extern "C" fn nbGetAdapter(lana: u8) -> *mut NetBIOSAdapter {
    let mut ret: *mut NetBIOSAdapter = core::ptr::null_mut();
    if (*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_TRACE as u32)) != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/netapi32/netbios.c\0"
                .as_ptr() as *const i8,
            b"nbGetAdapter\0".as_ptr() as *const i8,
            334i32,
            b": lana %d, num allocated adapters %d\n\0".as_ptr() as *const i8,
            lana as i32,
            gNBTable.tableSize as i32,
        );
    }
    if (lana as i32) < gNBTable.tableSize as i32
        && (*gNBTable.table.offset(lana as isize)).transport_id != 0u32
        && !(*gNBTable.table.offset(lana as isize)).transport.is_null()
    {
        ret = gNBTable.table.offset(lana as isize);
    }
    if (*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_TRACE as u32)) != 0 {
        let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl_0: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl_0,
            __dbch_0,
            b"dll/win32/netapi32/netbios.c\0"
                .as_ptr() as *const i8,
            b"nbGetAdapter\0".as_ptr() as *const i8,
            338i32,
            b"returning %p\n\0".as_ptr() as *const i8,
            ret,
        );
    }
    ret
}
unsafe extern "C" fn nbEnum(ncb: PNCB) -> UCHAR {
    let lanas: PLANA_ENUM = (*ncb).ncb_buffer as PLANA_ENUM;
    let mut i: UCHAR = 0;
    let mut ret: UCHAR = 0;

    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        ros_dbg_log(
            __WINE_DBCL_TRACE,
            __wine_dbch___default,
            b"dll/win32/netapi32/netbios.c\0"
                .as_ptr() as *const ::core::ffi::c_char,
            b"nbEnum\0".as_ptr() as *const ::core::ffi::c_char,
            347 as ::core::ffi::c_int,
            b": ncb %p\n\0".as_ptr() as *const ::core::ffi::c_char,
            ncb,
        );
    }

    if lanas.is_null() {
        ret = NRC_BUFLEN as UCHAR;
    } else if ((*ncb).ncb_length as usize) < ::core::mem::size_of::<LANA_ENUM>() as usize {
        ret = NRC_BUFLEN as UCHAR;
    } else {
        nbInternalEnum();
        (*lanas).length = 0 as UCHAR;
        i = 0 as UCHAR;
        while (i as ::core::ffi::c_int) < gNBTable.tableSize as ::core::ffi::c_int {
            if !(*gNBTable.table.offset(i as isize)).transport.is_null() {
                (*lanas).length = (*lanas).length.wrapping_add(1);
                (*lanas).lana[i as usize] = i;
            }
            i = i.wrapping_add(1);
        }
        ret = NRC_GOODRET as UCHAR;
    }

    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        ros_dbg_log(
            __WINE_DBCL_TRACE,
            __wine_dbch___default,
            b"dll/win32/netapi32/netbios.c\0"
                .as_ptr() as *const ::core::ffi::c_char,
            b"nbEnum\0".as_ptr() as *const ::core::ffi::c_char,
            365 as ::core::ffi::c_int,
            b"returning 0x%02x\n\0".as_ptr() as *const ::core::ffi::c_char,
            ret as ::core::ffi::c_int,
        );
    }

    ret
}
unsafe extern "C" fn nbCancel(adapter: *mut NetBIOSAdapter, ncb: PNCB) -> UCHAR {
    let mut ret: UCHAR = 0;
    if (*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_TRACE as u32)) != 0 {
        ros_dbg_log(
            __WINE_DBCL_TRACE,
            __wine_dbch___default,
            b"dll/win32/netapi32/netbios.c\0".as_ptr() as *const i8,
            b"nbCancel\0".as_ptr() as *const i8,
            375i32,
            b": adapter %p, ncb %p\n\0".as_ptr() as *const i8,
            adapter,
            ncb,
        );
    }
    if adapter.is_null() {
        return NRC_BRIDGE as UCHAR;
    }
    if ncb.is_null() {
        return NRC_INVADDRESS as UCHAR;
    }
    match (*ncb).ncb_command as u32 & 0x7f {
        NCBCANCEL | NCBADDNAME | NCBADDGRNAME | NCBDELNAME | NCBRESET | NCBSSTAT => {
            ret = NRC_CANCEL as UCHAR;
        }
        NCBCALL | NCBSEND | NCBCHAINSEND | NCBSENDNA | NCBCHAINSENDNA | NCBHANGUP => {
            if (*ncb).ncb_lsn as u32 >= (*adapter).sessionsLen {
                ret = NRC_SNUMOUT as UCHAR;
            } else if (*((*adapter).sessions).offset((*ncb).ncb_lsn as isize)).inUse == 0 {
                ret = NRC_SNUMOUT as UCHAR;
            } else {
                ret = NBCmdQueueCancel((*adapter).cmdQueue, ncb);
                if ret as i32 == NRC_CMDCAN as i32 || ret as i32 == NRC_CANOCCR as i32 {
                    nbInternalHangup(
                        adapter,
                        ((*adapter).sessions).offset((*ncb).ncb_lsn as isize) as *mut NetBIOSSession,
                    );
                }
            }
        }
        _ => {
            ret = NBCmdQueueCancel((*adapter).cmdQueue, ncb);
        }
    }
    if (*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_TRACE as u32)) != 0 {
        ros_dbg_log(
            __WINE_DBCL_TRACE,
            __wine_dbch___default,
            b"dll/win32/netapi32/netbios.c\0".as_ptr() as *const i8,
            b"nbCancel\0".as_ptr() as *const i8,
            416i32,
            b"returning 0x%02x\n\0".as_ptr() as *const i8,
            ret as i32,
        );
    }
    return ret;
}
unsafe extern "C" fn nbResizeAdapter(adapter: *mut NetBIOSAdapter, sessionsLen: UCHAR) -> UCHAR {
    let mut ret: UCHAR = NRC_GOODRET as UCHAR;
    if !adapter.is_null() && (*adapter).sessionsLen < (sessionsLen as u32) {
        let newSessions: *mut NetBIOSSession;
        if !(*adapter).sessions.is_null() {
            newSessions = HeapReAlloc(
                GetProcessHeap(),
                HEAP_ZERO_MEMORY as DWORD,
                (*adapter).sessions as PVOID,
                (sessionsLen as SIZE_T)
                    .wrapping_mul(core::mem::size_of::<NetBIOSSession>() as SIZE_T),
            ) as *mut NetBIOSSession;
        } else {
            newSessions = HeapAlloc(
                GetProcessHeap(),
                HEAP_ZERO_MEMORY as DWORD,
                (sessionsLen as SIZE_T)
                    .wrapping_mul(core::mem::size_of::<NetBIOSSession>() as SIZE_T),
            ) as *mut NetBIOSSession;
        }
        if !newSessions.is_null() {
            (*adapter).sessions = newSessions;
            (*adapter).sessionsLen = sessionsLen as u32;
        } else {
            ret = NRC_OSRESNOTAV as UCHAR;
        }
    }
    ret
}
unsafe extern "C" fn nbReset(adapter: *mut NetBIOSAdapter, ncb: PNCB) -> UCHAR {
    if (*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_TRACE as u32)) != 0 {
        ros_dbg_log(
            __WINE_DBCL_TRACE,
            __wine_dbch___default as *mut __wine_debug_channel,
            b"dll/win32/netapi32/netbios.c\0".as_ptr() as *const i8,
            b"nbReset\0".as_ptr() as *const i8,
            457i32,
            b": adapter %p, ncb %p\n\0".as_ptr() as *const i8,
            adapter as *mut core::ffi::c_void,
            ncb as *mut core::ffi::c_void,
        );
    }
    if adapter.is_null() {
        return NRC_BRIDGE as UCHAR;
    }
    if ncb.is_null() {
        return NRC_INVADDRESS as UCHAR;
    }

    let mut ret: UCHAR;
    if InterlockedIncrement(&raw mut (*adapter).resetting) == 1i32 as core::ffi::c_long {
        let resizeTo: UCHAR;

        NBCmdQueueCancelAll((*adapter).cmdQueue);

        EnterCriticalSection(&raw mut (*adapter).cs);

        let sessions_ptr = (*adapter).sessions;
        let sessions_len = (*adapter).sessionsLen;
        let mut i: UCHAR = 0;
        while (i as u32) < sessions_len {
            if (*sessions_ptr.offset(i as isize)).inUse != 0 {
                nbInternalHangup(
                    adapter,
                    sessions_ptr.offset(i as isize) as *mut NetBIOSSession,
                );
            }
            i = i.wrapping_add(1);
        }

        if (*ncb).ncb_lsn == 0 {
            resizeTo = if (*ncb).ncb_callname[0] as i32 == 0 {
                DEFAULT_NUM_SESSIONS as UCHAR
            } else {
                (*ncb).ncb_callname[0] as UCHAR
            };
        } else if (*adapter).sessionsLen == 0u32 {
            resizeTo = DEFAULT_NUM_SESSIONS as UCHAR;
        } else {
            resizeTo = 0u8;
        }

        if resizeTo > 0u8 {
            ret = nbResizeAdapter(adapter, resizeTo);
        } else {
            ret = NRC_GOODRET as UCHAR;
        }

        LeaveCriticalSection(&raw mut (*adapter).cs);
    } else {
        ret = NRC_TOOMANY as UCHAR;
    }

    InterlockedDecrement(&raw mut (*adapter).resetting);

    if (*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_TRACE as u32)) != 0 {
        ros_dbg_log(
            __WINE_DBCL_TRACE,
            __wine_dbch___default as *mut __wine_debug_channel,
            b"dll/win32/netapi32/netbios.c\0".as_ptr() as *const i8,
            b"nbReset\0".as_ptr() as *const i8,
            488i32,
            b"returning 0x%02x\n\0".as_ptr() as *const i8,
            ret as i32,
        );
    }

    ret
}
unsafe extern "C" fn nbSStat(adapter: *mut NetBIOSAdapter, ncb: PNCB) -> u8 {
    if (*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_TRACE as u32)) != 0 {
        ros_dbg_log(
            __WINE_DBCL_TRACE,
            __wine_dbch___default,
            b"dll/win32/netapi32/netbios.c\0"
                .as_ptr() as *const i8,
            b"nbSStat\0".as_ptr() as *const i8,
            497i32,
            b": adapter %p, NCB %p\n\0".as_ptr() as *const i8,
            adapter,
            ncb,
        );
    }
    if adapter.is_null() {
        return NRC_BADDR as u8;
    }
    if (*adapter).sessionsLen == 0u32 {
        return NRC_ENVNOTDEF as u8;
    }
    if ncb.is_null() {
        return NRC_INVADDRESS as u8;
    }
    if (*ncb).ncb_buffer.is_null() {
        return NRC_BADDR as u8;
    }
    if ((*ncb).ncb_length as usize) < core::mem::size_of::<SESSION_HEADER>() as usize {
        return NRC_BUFLEN as u8;
    }
    let sstat: PSESSION_HEADER = (*ncb).ncb_buffer as PSESSION_HEADER;
    let mut ret: u8 = NRC_GOODRET as u8;
    core::ptr::write_bytes(sstat as *mut u8, 0u8, core::mem::size_of::<SESSION_HEADER>() as usize);
    let spaceFor: u8 = ((*ncb).ncb_length as usize)
        .wrapping_sub(core::mem::size_of::<SESSION_HEADER>() as usize)
        .wrapping_div(core::mem::size_of::<SESSION_BUFFER>() as usize) as u8;
    EnterCriticalSection(core::ptr::addr_of_mut!((*adapter).cs));
    let mut i: u8 = 0u8;
    while ret as i32 == NRC_GOODRET as i32 && (i as u32) < (*adapter).sessionsLen {
        if (*(*adapter).sessions.offset(i as isize)).inUse != 0
            && ((*ncb).ncb_name[0usize] as i32 == '*' as i32
                || memcmp(
                    core::ptr::addr_of_mut!((*ncb).ncb_name) as *const u8 as *const core::ffi::c_void,
                    core::ptr::addr_of_mut!((*(*adapter).sessions.offset(i as isize)).local_name)
                        as *const u8 as *const core::ffi::c_void,
                    NCBNAMSZ as usize,
                ) == 0)
        {
            if ((*sstat).num_sess as i32) < spaceFor as i32 {
                let buf: PSESSION_BUFFER = (sstat as *mut u8)
                    .offset(core::mem::size_of::<SESSION_HEADER>() as isize)
                    .offset(
                        ((*sstat).num_sess as usize)
                            .wrapping_mul(core::mem::size_of::<SESSION_BUFFER>() as usize)
                            as isize,
                    ) as PSESSION_BUFFER;
                (*buf).lsn = i;
                (*buf).state = (*(*adapter).sessions.offset(i as isize)).state;
                core::ptr::copy_nonoverlapping(
                    core::ptr::addr_of_mut!((*(*adapter).sessions.offset(i as isize)).local_name)
                        as *const u8 as *const core::ffi::c_void,
                    core::ptr::addr_of_mut!((*buf).local_name) as *mut u8 as *mut core::ffi::c_void,
                    NCBNAMSZ as usize,
                );
                core::ptr::copy_nonoverlapping(
                    core::ptr::addr_of_mut!((*(*adapter).sessions.offset(i as isize)).remote_name)
                        as *const u8 as *const core::ffi::c_void,
                    core::ptr::addr_of_mut!((*buf).remote_name) as *mut u8 as *mut core::ffi::c_void,
                    NCBNAMSZ as usize,
                );
                (*buf).sends_outstanding = 0u8;
                (*buf).rcvs_outstanding = (*buf).sends_outstanding;
                (*sstat).num_sess = (*sstat).num_sess.wrapping_add(1u8);
            } else {
                ret = NRC_BUFLEN as u8;
            }
        }
        i = i.wrapping_add(1u8);
    }
    LeaveCriticalSection(core::ptr::addr_of_mut!((*adapter).cs));
    if (*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_TRACE as u32)) != 0 {
        ros_dbg_log(
            __WINE_DBCL_TRACE,
            __wine_dbch___default,
            b"dll/win32/netapi32/netbios.c\0"
                .as_ptr() as *const i8,
            b"nbSStat\0".as_ptr() as *const i8,
            537i32,
            b"returning 0x%02x\n\0".as_ptr() as *const i8,
            ret as i32,
        );
    }
    ret
}
unsafe extern "C" fn nbCall(adapter: *mut NetBIOSAdapter, ncb: PNCB) -> UCHAR {
    let mut ret: UCHAR;
    let mut i: UCHAR;

    if adapter.is_null() {
        return NRC_BRIDGE as UCHAR;
    }
    if (*adapter).sessionsLen == 0 {
        return NRC_ENVNOTDEF as UCHAR;
    }
    if (*(*adapter).transport).call.is_none() {
        return NRC_ILLCMD as UCHAR;
    }
    if ncb.is_null() {
        return NRC_INVADDRESS as UCHAR;
    }

    EnterCriticalSection(core::ptr::addr_of_mut!((*adapter).cs));

    i = 0u8;
    while (i as u32) < (*adapter).sessionsLen && (*(*adapter).sessions.offset(i as isize)).inUse != 0 {
        i = i.wrapping_add(1);
    }
    if (i as u32) < (*adapter).sessionsLen {
        let session = (*adapter).sessions.offset(i as isize);
        (*session).inUse = TRUE;
        (*session).state = CALL_PENDING as UCHAR;
        memcpy(
            (*session).local_name.as_mut_ptr() as *mut ::core::ffi::c_void,
            (*ncb).ncb_name.as_ptr() as *const ::core::ffi::c_void,
            NCBNAMSZ as usize,
        );
        memcpy(
            (*session).remote_name.as_mut_ptr() as *mut ::core::ffi::c_void,
            (*ncb).ncb_callname.as_ptr() as *const ::core::ffi::c_void,
            NCBNAMSZ as usize,
        );
        ret = NRC_GOODRET as UCHAR;
    } else {
        ret = NRC_LOCTFUL as UCHAR;
    }

    LeaveCriticalSection(core::ptr::addr_of_mut!((*adapter).cs));

    if ret as i32 == NRC_GOODRET as i32 {
        if let Some(call_fn) = (*(*adapter).transport).call {
            ret = call_fn(
                (*adapter).impl_.data,
                ncb,
                core::ptr::addr_of_mut!((*(*adapter).sessions.offset(i as isize)).data),
            );
        } else {
            ret = NRC_ILLCMD as UCHAR;
        }
        if ret as i32 == NRC_GOODRET as i32 {
            (*ncb).ncb_lsn = i;
            (*(*adapter).sessions.offset(i as isize)).state = SESSION_ESTABLISHED as UCHAR;
        } else {
            let session = (*adapter).sessions.offset(i as isize);
            (*session).inUse = FALSE;
            (*session).state = 0u8;
        }
    }

    if (*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_TRACE as u32)) != 0 {
        ros_dbg_log(
            __WINE_DBCL_TRACE,
            __wine_dbch___default,
            b"dll/win32/netapi32/netbios.c\0" as *const u8 as *const ::core::ffi::c_char,
            b"nbCall\0" as *const u8 as *const ::core::ffi::c_char,
            582i32,
            b"returning 0x%02x\n\0" as *const u8 as *const ::core::ffi::c_char,
            ret as i32,
        );
    }

    ret
}
unsafe extern "C" fn nbSend(adapter: *mut NetBIOSAdapter, ncb: PNCB) -> UCHAR {
    if adapter.is_null() {
        return NRC_BRIDGE as UCHAR;
    }
    if (*(*adapter).transport).send.is_none() {
        return NRC_ILLCMD as UCHAR;
    }
    if ncb.is_null() {
        return NRC_INVADDRESS as UCHAR;
    }
    if (*ncb).ncb_lsn as usize >= (*adapter).sessionsLen as usize {
        return NRC_SNUMOUT as UCHAR;
    }
    let session = (*adapter).sessions.add((*ncb).ncb_lsn as usize);
    if (*session).inUse == 0 {
        return NRC_SNUMOUT as UCHAR;
    }
    if (*ncb).ncb_buffer.is_null() {
        return NRC_BADDR as UCHAR;
    }

    let ret;
    if (*session).state as core::ffi::c_int != SESSION_ESTABLISHED as i32 {
        ret = NRC_SNUMOUT as UCHAR;
    } else if let Some(send_fn) = (*(*adapter).transport).send {
        ret = send_fn(
            (*adapter).impl_.data,
            (*session).data,
            ncb,
        );
    } else {
        ret = NRC_ILLCMD as UCHAR;
    }
    ret
}
unsafe extern "C" fn nbRecv(adapter: *mut NetBIOSAdapter, ncb: PNCB) -> UCHAR {
    let mut ret: UCHAR = 0;
    let session: *mut NetBIOSSession;

    if adapter.is_null() {
        return NRC_BRIDGE;
    }
    if (*(*adapter).transport).recv.is_none() {
        return NRC_ILLCMD;
    }
    if ncb.is_null() {
        return NRC_INVADDRESS;
    }
    if (*ncb).ncb_lsn as u32 >= (*adapter).sessionsLen {
        return NRC_SNUMOUT;
    }
    if (*(*adapter).sessions.offset((*ncb).ncb_lsn as isize)).inUse == 0 {
        return NRC_SNUMOUT;
    }
    if (*ncb).ncb_buffer.is_null() {
        return NRC_BADDR;
    }

    session = (*adapter).sessions.offset((*ncb).ncb_lsn as isize) as *mut NetBIOSSession;
    if (*session).state as i32 != SESSION_ESTABLISHED as i32 {
        ret = NRC_SNUMOUT;
    } else if let Some(recv_fn) = (*(*adapter).transport).recv {
        ret = recv_fn(
            (*adapter).impl_.data,
            (*session).data,
            ncb,
        );
    } else {
        ret = NRC_ILLCMD as UCHAR;
    }
    return ret;
}
unsafe extern "C" fn nbInternalHangup(adapter: *mut _NetBIOSAdapter, session: *mut _NetBIOSSession) -> u8 {
    if adapter.is_null() {
        return NRC_BRIDGE as u8;
    }
    if session.is_null() {
        return NRC_SNUMOUT as u8;
    }

    let ret: u8;
    if let Some(hangup_fn) = (*(*adapter).transport).hangup {
        ret = hangup_fn(
            (*adapter).impl_.data,
            (*session).data,
        );
    } else {
        ret = NRC_ILLCMD as u8;
    }

    EnterCriticalSection(&raw mut (*adapter).cs);
    memset(
        session as *mut core::ffi::c_void,
        0i32,
        core::mem::size_of::<_NetBIOSSession>() as usize,
    );
    LeaveCriticalSection(&raw mut (*adapter).cs);

    ret
}
unsafe extern "C" fn nbHangup(adapter: *mut _NetBIOSAdapter, ncb: *const NCB) -> u8 {
    if adapter.is_null() {
        return NRC_BRIDGE as u8;
    }
    if ncb.is_null() {
        return NRC_INVADDRESS as u8;
    }
    if (*ncb).ncb_lsn as u32 >= (*adapter).sessionsLen {
        return NRC_SNUMOUT as u8;
    }
    if (*(*adapter).sessions.offset((*ncb).ncb_lsn as isize)).inUse == 0 {
        return NRC_SNUMOUT as u8;
    }
    let session = (*adapter).sessions.offset((*ncb).ncb_lsn as isize) as *mut _NetBIOSSession;
    let ret;
    if (*session).state != SESSION_ESTABLISHED as u8 {
        ret = NRC_SNUMOUT as u8;
    } else {
        (*session).state = HANGUP_PENDING as u8;
        ret = nbInternalHangup(adapter, session);
    }
    ret
}
#[no_mangle]
pub unsafe extern "C" fn NetBIOSHangupSession(ncb: *const NCB) {
    if ncb.is_null() {
        return;
    }

    let adapter: *mut NetBIOSAdapter = nbGetAdapter((*ncb).ncb_lana_num);
    if !adapter.is_null() {
        if ((*ncb).ncb_lsn as u32) < (*adapter).sessionsLen
            && (*(*adapter).sessions.add((*ncb).ncb_lsn as usize)).inUse != 0
        {
            nbHangup(adapter, ncb);
        }
    }
}
unsafe extern "C" fn nbAStat(adapter: *mut NetBIOSAdapter, ncb: PNCB) -> u8 {
    if adapter.is_null() {
        return NRC_BRIDGE as u8;
    }
    if (*(*adapter).transport).astat.is_none() {
        return NRC_ILLCMD as u8;
    }
    if ncb.is_null() {
        return NRC_INVADDRESS as u8;
    }
    if (*ncb).ncb_buffer.is_null() {
        return NRC_BADDR as u8;
    }
    if ((*ncb).ncb_length as usize) < core::mem::size_of::<ADAPTER_STATUS>() {
        return NRC_BUFLEN as u8;
    }

    let astat_fn = (*(*adapter).transport).astat.unwrap_unchecked();
    let ret = astat_fn((*adapter).impl_.data, ncb);

    if (*ncb).ncb_callname[0] == b'*' {
        let astat: *mut ADAPTER_STATUS = (*ncb).ncb_buffer as *mut ADAPTER_STATUS;
        (*astat).max_cfg_sess = (*adapter).sessionsLen as u16;
        (*astat).max_sess = (*astat).max_cfg_sess;
    }

    ret
}
unsafe extern "C" fn nbDispatch(adapter: *mut NetBIOSAdapter, ncb: PNCB) -> UCHAR {
    let mut ret: UCHAR;
    let cmd: UCHAR;

    if (*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_TRACE as u32)) != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/netapi32/netbios.c\0"
                .as_ptr() as *const i8,
            b"nbDispatch\0".as_ptr() as *const i8,
            703i32,
            b": adapter %p, ncb %p\n\0".as_ptr() as *const i8,
            adapter,
            ncb,
        );
    }
    if adapter.is_null() {
        return NRC_BRIDGE as UCHAR;
    }
    if ncb.is_null() {
        return NRC_INVADDRESS as UCHAR;
    }
    cmd = ((*ncb).ncb_command as i32 & 0x7fi32) as UCHAR;
    if cmd as u32 == NCBRESET {
        ret = nbReset(adapter, ncb);
    } else {
        ret = NBCmdQueueAdd((*adapter).cmdQueue, ncb);
        if ret as i32 == NRC_GOODRET as i32 {
            match cmd as u32 {
                NCBCALL => {
                    ret = nbCall(adapter, ncb);
                }
                NCBSEND | NCBSENDNA | NCBCHAINSEND | NCBCHAINSENDNA => {
                    ret = nbSend(adapter, ncb);
                }
                NCBRECV => {
                    ret = nbRecv(adapter, ncb);
                }
                NCBHANGUP => {
                    ret = nbHangup(adapter, ncb as *const NCB);
                }
                NCBASTAT => {
                    ret = nbAStat(adapter, ncb);
                }
                NCBFINDNAME => {
                    if let Some(find_name) = (*(*adapter).transport).findName {
                        ret = find_name(
                            (*adapter).impl_.data,
                            ncb,
                        );
                    } else {
                        ret = NRC_ILLCMD as UCHAR;
                    }
                }
                _ => {
                    if ((*__wine_dbch___default).flags as i32)
                        & (1i32 << (__WINE_DBCL_FIXME as i32))
                        != 0
                    {
                        let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
                        let __dbcl_0: __wine_debug_class = __WINE_DBCL_FIXME;
                        ros_dbg_log(
                            __dbcl_0,
                            __dbch_0,
                            b"dll/win32/netapi32/netbios.c\0"
                                .as_ptr() as *const i8,
                            b"nbDispatch\0".as_ptr() as *const i8,
                            753i32,
                            b"(%p): command code 0x%02x\n\0".as_ptr() as *const i8,
                            ncb,
                            (*ncb).ncb_command as i32,
                        );
                    }
                    ret = NRC_ILLCMD as UCHAR;
                }
            }
            NBCmdQueueComplete((*adapter).cmdQueue, ncb, ret);
        }
    }
    if (*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_TRACE as u32)) != 0 {
        let __dbch_1: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl_1: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl_1,
            __dbch_1,
            b"dll/win32/netapi32/netbios.c\0"
                .as_ptr() as *const i8,
            b"nbDispatch\0".as_ptr() as *const i8,
            759i32,
            b"returning 0x%02x\n\0".as_ptr() as *const i8,
            ret as i32,
        );
    }
    return ret;
}
#[cfg(target_arch = "x86")]
unsafe extern "system" fn nbCmdThread(lpVoid: LPVOID) -> DWORD {
    let ncb: PNCB = lpVoid as PNCB;
    if !ncb.is_null() {
        let mut ret: UCHAR = 0;
        let adapter: *mut NetBIOSAdapter = nbGetAdapter((*ncb).ncb_lana_num);
        if !adapter.is_null() {
            ret = nbDispatch(adapter, ncb);
        } else {
            ret = NRC_BRIDGE as UCHAR;
        }
        (*ncb).ncb_cmd_cplt = ret;
        (*ncb).ncb_retcode = (*ncb).ncb_cmd_cplt;
        if let Some(post_fn) = (*ncb).ncb_post {
            post_fn(ncb);
        } else if !(*ncb).ncb_event.is_null() {
            SetEvent((*ncb).ncb_event);
        }
    }
    0u32
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "system" fn nbCmdThread(lpVoid: LPVOID) -> DWORD {
    let ncb: PNCB = lpVoid as PNCB;
    if !ncb.is_null() {
        let mut ret: UCHAR = 0;
        let adapter: *mut NetBIOSAdapter = nbGetAdapter((*ncb).ncb_lana_num);
        if !adapter.is_null() {
            ret = nbDispatch(adapter, ncb);
        } else {
            ret = NRC_BRIDGE as UCHAR;
        }
        (*ncb).ncb_cmd_cplt = ret;
        (*ncb).ncb_retcode = (*ncb).ncb_cmd_cplt;
        if let Some(post_fn) = (*ncb).ncb_post {
            post_fn(ncb);
        } else if !(*ncb).ncb_event.is_null() {
            SetEvent((*ncb).ncb_event);
        }
    }
    0u32
}

#[no_mangle]
pub unsafe extern "system" fn Netbios(ncb: PNCB) -> UCHAR {
    let mut ret: UCHAR;
    let cmd: UCHAR;

    if (*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_TRACE as u32)) != 0 {
        ros_dbg_log(
            __WINE_DBCL_TRACE,
            __wine_dbch___default,
            b"dll/win32/netapi32/netbios.c\0"
                .as_ptr() as *const i8,
            b"Netbios\0".as_ptr() as *const i8,
            789,
            b"ncb = %p\n\0".as_ptr() as *const i8,
            ncb,
        );
    }

    if ncb.is_null() {
        return NRC_INVADDRESS as UCHAR;
    }

    if (*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_TRACE as u32)) != 0 {
        ros_dbg_log(
            __WINE_DBCL_TRACE,
            __wine_dbch___default,
            b"dll/win32/netapi32/netbios.c\0"
                .as_ptr() as *const i8,
            b"Netbios\0".as_ptr() as *const i8,
            794,
            b"ncb_command 0x%02x, ncb_lana_num %d, ncb_buffer %p, ncb_length %d\n\0"
                .as_ptr() as *const i8,
            (*ncb).ncb_command as i32,
            (*ncb).ncb_lana_num as i32,
            (*ncb).ncb_buffer,
            (*ncb).ncb_length as i32,
        );
    }

    cmd = ((*ncb).ncb_command as i32 & 0x7f) as UCHAR;

    if cmd as u32 == NCBENUM {
        ret = nbEnum(ncb);
        (*ncb).ncb_cmd_cplt = ret;
        (*ncb).ncb_retcode = ret;
    } else if cmd as u32 == NCBADDNAME {
        if (*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_FIXME as u32)) != 0 {
            ros_dbg_log(
                __WINE_DBCL_FIXME,
                __wine_dbch___default,
                b"dll/win32/netapi32/netbios.c\0"
                    .as_ptr() as *const i8,
                b"Netbios\0".as_ptr() as *const i8,
                801,
                b"NCBADDNAME: stub, returning success\n\0".as_ptr() as *const i8,
            );
        }
        ret = NRC_GOODRET as UCHAR;
        (*ncb).ncb_cmd_cplt = ret;
        (*ncb).ncb_retcode = ret;
    } else {
        let adapter: *mut NetBIOSAdapter;

        if gNBTable.enumerated == 0 {
            nbInternalEnum();
        }

        adapter = nbGetAdapter((*ncb).ncb_lana_num);
        if adapter.is_null() {
            ret = NRC_BRIDGE as UCHAR;
        } else if (*adapter).shuttingDown != 0 {
            ret = NRC_IFBUSY as UCHAR;
        } else if (*adapter).resetting != 0 {
            ret = NRC_TOOMANY as UCHAR;
        } else if cmd as u32 == NCBCANCEL {
            ret = nbCancel(adapter, ncb);
            (*ncb).ncb_cmd_cplt = ret;
            (*ncb).ncb_retcode = ret;
        } else if cmd as u32 == NCBSSTAT {
            ret = nbSStat(adapter, ncb);
            (*ncb).ncb_cmd_cplt = ret;
            (*ncb).ncb_retcode = ret;
        } else if ((*ncb).ncb_command as u32 & ASYNCH) != 0 {
            let thread: HANDLE = CreateThread(
                core::ptr::null_mut::<core::ffi::c_void>(),
                0 as SIZE_T,
                Some(nbCmdThread),
                ncb as LPVOID,
                CREATE_SUSPENDED as DWORD,
                core::ptr::null_mut::<DWORD>(),
            );
            if !thread.is_null() {
                (*ncb).ncb_cmd_cplt = NRC_PENDING as UCHAR;
                (*ncb).ncb_retcode = (*ncb).ncb_cmd_cplt;
                if !(*ncb).ncb_event.is_null() {
                    ResetEvent((*ncb).ncb_event);
                }
                ResumeThread(thread);
                CloseHandle(thread);
                ret = NRC_GOODRET as UCHAR;
            } else {
                ret = NRC_OSRESNOTAV as UCHAR;
                (*ncb).ncb_cmd_cplt = ret;
                (*ncb).ncb_retcode = ret;
            }
        } else {
            ret = nbDispatch(adapter, ncb);
            (*ncb).ncb_cmd_cplt = ret;
            (*ncb).ncb_retcode = ret;
        }
    }

    if (*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_TRACE as u32)) != 0 {
        ros_dbg_log(
            __WINE_DBCL_TRACE,
            __wine_dbch___default,
            b"dll/win32/netapi32/netbios.c\0"
                .as_ptr() as *const i8,
            b"Netbios\0".as_ptr() as *const i8,
            858,
            b"returning 0x%02x\n\0".as_ptr() as *const i8,
            ret as i32,
        );
    }

    ret
}

#[no_mangle]
pub unsafe extern "system" fn NetpNetBiosStatusToApiStatus(nrc: DWORD) -> DWORD {
    match nrc {
        0 => NO_ERROR,
        9 => NERR_NoNetworkResource,
        13 => NERR_AlreadyExists,
        14 => NERR_TooManyNames,
        15 => NERR_DeleteLater,
        18 => ERROR_REM_NOT_LIST,
        20 => NERR_NameNotFound,
        21 => ERROR_INVALID_PARAMETER,
        22 => NERR_DuplicateName,
        23 => ERROR_INVALID_PARAMETER,
        25 => NERR_DuplicateName,
        _ => NERR_NetworkError,
    }
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /* Copyright (c) 2003 Juan Lang
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
// #include "nbcmdqueue.h"
//
// WINE_DEFAULT_DEBUG_CHANNEL(netbios);
//
// /* This file provides a NetBIOS emulator that implements the NetBIOS interface,
//  * including thread safety and asynchronous call support.  The protocol
//  * implementation is separate, with blocking (synchronous) functions.
//  */
//
// #define ADAPTERS_INCR 8
// #define DEFAULT_NUM_SESSIONS 16
//
// typedef struct _NetBIOSTransportTableEntry
// {
//     ULONG            id;
//     NetBIOSTransport transport;
// } NetBIOSTransportTableEntry;
//
// typedef struct _NetBIOSSession
// {
//     BOOL  inUse;
//     UCHAR state;
//     UCHAR local_name[NCBNAMSZ];
//     UCHAR remote_name[NCBNAMSZ];
//     void *data;
// } NetBIOSSession;
//
// /* This struct needs a little explanation, unfortunately.  enabled is only
//  * used by nbInternalEnum (see).  If transport_id is not 0 and transport
//  * is not NULL, the adapter is considered valid.  (transport is a pointer to
//  * an entry in a NetBIOSTransportTableEntry.)  data has data for the callers of
//  * NetBIOSEnumAdapters to be able to see.  The lana is repeated there, even
//  * though I don't use it internally--it's for transports to use re-enabling
//  * adapters using NetBIOSEnableAdapter.
//  */
// typedef struct _NetBIOSAdapter
// {
//     BOOL               enabled;
//     BOOL               shuttingDown;
//     LONG               resetting;
//     ULONG              transport_id;
//     NetBIOSTransport  *transport;
//     NetBIOSAdapterImpl impl;
//     struct NBCmdQueue *cmdQueue;
//     CRITICAL_SECTION   cs;
//     DWORD              sessionsLen;
//     NetBIOSSession    *sessions;
// } NetBIOSAdapter;
//
// typedef struct _NetBIOSAdapterTable {
//     CRITICAL_SECTION cs;
//     BOOL             enumerated;
//     BOOL             enumerating;
//     UCHAR            tableSize;
//     NetBIOSAdapter  *table;
// } NetBIOSAdapterTable;
//
// /* Just enough space for NBT right now */
// static NetBIOSTransportTableEntry gTransports[1];
// static UCHAR gNumTransports = 0;
// static NetBIOSAdapterTable gNBTable;
//
// static UCHAR nbResizeAdapterTable(UCHAR newSize)
// {
//     UCHAR ret;
//
//     if (gNBTable.table)
//         gNBTable.table = HeapReAlloc(GetProcessHeap(),
//          HEAP_ZERO_MEMORY, gNBTable.table,
//          newSize * sizeof(NetBIOSAdapter));
//     else
//         gNBTable.table = HeapAlloc(GetProcessHeap(),
//          HEAP_ZERO_MEMORY, newSize * sizeof(NetBIOSAdapter));
//     if (gNBTable.table)
//     {
//         gNBTable.tableSize = newSize;
//         ret = NRC_GOODRET;
//     }
//     else
//         ret = NRC_OSRESNOTAV;
//     return ret;
// }
//
// void NetBIOSInit(void)
// {
//     memset(&gNBTable, 0, sizeof(gNBTable));
//     InitializeCriticalSection(&gNBTable.cs);
//     gNBTable.cs.DebugInfo->Spare[0] = (DWORD_PTR)(__FILE__ ": NetBIOSAdapterTable.cs");
// }
//
// void NetBIOSShutdown(void)
// {
//     UCHAR i;
//
//     EnterCriticalSection(&gNBTable.cs);
//     for (i = 0; i < gNBTable.tableSize; i++)
//     {
//         if (gNBTable.table[i].transport &&
//          gNBTable.table[i].transport->cleanupAdapter)
//             gNBTable.table[i].transport->cleanupAdapter(
//              gNBTable.table[i].impl.data);
//     }
//     for (i = 0; i < gNumTransports; i++)
//         if (gTransports[i].transport.cleanup)
//             gTransports[i].transport.cleanup();
//     LeaveCriticalSection(&gNBTable.cs);
//     gNBTable.cs.DebugInfo->Spare[0] = 0;
//     DeleteCriticalSection(&gNBTable.cs);
//     HeapFree(GetProcessHeap(), 0, gNBTable.table);
// }
//
// BOOL NetBIOSRegisterTransport(ULONG id, NetBIOSTransport *transport)
// {
//     BOOL ret;
//
//     TRACE(": transport 0x%08x, p %p\n", id, transport);
//     if (!transport)
//         ret = FALSE;
//     else if (gNumTransports >= sizeof(gTransports) / sizeof(gTransports[0]))
//     {
//         FIXME("Too many transports %d\n", gNumTransports + 1);
//         ret = FALSE;
//     }
//     else
//     {
//         UCHAR i;
//
//         ret = FALSE;
//         for (i = 0; !ret && i < gNumTransports; i++)
//         {
//             if (gTransports[i].id == id)
//             {
//                 WARN("Replacing NetBIOS transport ID %d\n", id);
//                 memcpy(&gTransports[i].transport, transport,
//                  sizeof(NetBIOSTransport));
//                 ret = TRUE;
//             }
//         }
//         if (!ret)
//         {
//             gTransports[gNumTransports].id = id;
//             memcpy(&gTransports[gNumTransports].transport, transport,
//              sizeof(NetBIOSTransport));
//             gNumTransports++;
//             ret = TRUE;
//         }
//     }
//     TRACE("returning %d\n", ret);
//     return ret;
// }
//
// /* In this, I acquire the table lock to make sure no one else is modifying it.
//  * This is _probably_ overkill since it should only be called during the
//  * context of a NetBIOSEnum call, but just to be safe..
//  */
// BOOL NetBIOSRegisterAdapter(ULONG transport, DWORD ifIndex, void *data)
// {
//     BOOL ret;
//     UCHAR i;
//
//     TRACE(": transport 0x%08x, ifIndex 0x%08x, data %p\n", transport, ifIndex,
//      data);
//     for (i = 0; i < gNumTransports && gTransports[i].id != transport; i++)
//         ;
//     if ((i < gNumTransports) && gTransports[i].id == transport)
//     {
//         NetBIOSTransport *transportPtr = &gTransports[i].transport;
//
//         TRACE(": found transport %p for id 0x%08x\n", transportPtr, transport);
//
//         EnterCriticalSection(&gNBTable.cs);
//         ret = FALSE;
//         for (i = 0; i < gNBTable.tableSize &&
//          gNBTable.table[i].transport != 0; i++)
//             ;
//         if (i == gNBTable.tableSize && gNBTable.tableSize < MAX_LANA + 1)
//         {
//             UCHAR newSize;
//
//             if (gNBTable.tableSize < (MAX_LANA + 1) - ADAPTERS_INCR)
//                 newSize = gNBTable.tableSize + ADAPTERS_INCR;
//             else
//                 newSize = MAX_LANA + 1;
//             nbResizeAdapterTable(newSize);
//         }
//         if (i < gNBTable.tableSize && gNBTable.table[i].transport == 0)
//         {
//             TRACE(": registering as LANA %d\n", i);
//             gNBTable.table[i].transport_id = transport;
//             gNBTable.table[i].transport = transportPtr;
//             gNBTable.table[i].impl.lana = i;
//             gNBTable.table[i].impl.ifIndex = ifIndex;
//             gNBTable.table[i].impl.data = data;
//             gNBTable.table[i].cmdQueue = NBCmdQueueCreate(GetProcessHeap());
//             InitializeCriticalSection(&gNBTable.table[i].cs);
//             gNBTable.table[i].cs.DebugInfo->Spare[0] = (DWORD_PTR)(__FILE__ ": NetBIOSAdapterTable.NetBIOSAdapter.cs");
//             gNBTable.table[i].enabled = TRUE;
//             ret = TRUE;
//         }
//         LeaveCriticalSection(&gNBTable.cs);
//     }
//     else
//         ret = FALSE;
//     TRACE("returning %d\n", ret);
//     return ret;
// }
//
// /* In this, I acquire the table lock to make sure no one else is modifying it.
//  * This is _probably_ overkill since it should only be called during the
//  * context of a NetBIOSEnum call, but just to be safe..
//  */
// void NetBIOSEnableAdapter(UCHAR lana)
// {
//     TRACE(": %d\n", lana);
//     if (lana < gNBTable.tableSize)
//     {
//         EnterCriticalSection(&gNBTable.cs);
//         if (gNBTable.table[lana].transport != 0)
//             gNBTable.table[lana].enabled = TRUE;
//         LeaveCriticalSection(&gNBTable.cs);
//     }
// }
//
// static void nbShutdownAdapter(NetBIOSAdapter *adapter)
// {
//     if (adapter)
//     {
//         adapter->shuttingDown = TRUE;
//         NBCmdQueueCancelAll(adapter->cmdQueue);
//         if (adapter->transport->cleanupAdapter)
//             adapter->transport->cleanupAdapter(adapter->impl.data);
//         NBCmdQueueDestroy(adapter->cmdQueue);
//         adapter->cs.DebugInfo->Spare[0] = 0;
//         DeleteCriticalSection(&adapter->cs);
//         memset(adapter, 0, sizeof(NetBIOSAdapter));
//     }
// }
//
// static void nbInternalEnum(void)
// {
//     UCHAR i;
//
//     EnterCriticalSection(&gNBTable.cs);
//     TRACE("before mark\n");
//     /* mark: */
//     for (i = 0; i < gNBTable.tableSize; i++)
//         if (gNBTable.table[i].enabled && gNBTable.table[i].transport != 0)
//             gNBTable.table[i].enabled = FALSE;
//
//     TRACE("marked, before store, %d transports\n", gNumTransports);
//     /* store adapters: */
//     for (i = 0; i < gNumTransports; i++)
//         if (gTransports[i].transport.enumerate)
//             gTransports[i].transport.enumerate();
//
//     TRACE("before sweep\n");
//     /* sweep: */
//     for (i = 0; i < gNBTable.tableSize; i++)
//         if (!gNBTable.table[i].enabled && gNBTable.table[i].transport != 0)
//             nbShutdownAdapter(&gNBTable.table[i]);
//     gNBTable.enumerated = TRUE;
//     LeaveCriticalSection(&gNBTable.cs);
// }
//
// UCHAR NetBIOSNumAdapters(void)
// {
//     UCHAR ret, i;
//
//     if (!gNBTable.enumerated)
//         nbInternalEnum();
//     for (i = 0, ret = 0; i < gNBTable.tableSize; i++)
//         if (gNBTable.table[i].transport != 0)
//             ret++;
//     return ret;
// }
//
// void NetBIOSEnumAdapters(ULONG transport, NetBIOSEnumAdaptersCallback cb,
//  void *closure)
// {
//     TRACE("transport 0x%08x, callback %p, closure %p\n", transport, cb,
//      closure);
//     if (cb)
//     {
//         BOOL enumAll = memcmp(&transport, ALL_TRANSPORTS, sizeof(ULONG)) == 0;
//         UCHAR i, numLANAs = 0;
//
//         EnterCriticalSection(&gNBTable.cs);
//         if (!gNBTable.enumerating)
//         {
//             gNBTable.enumerating = TRUE;
//             nbInternalEnum();
//             gNBTable.enumerating = FALSE;
//         }
//         for (i = 0; i < gNBTable.tableSize; i++)
//             if (enumAll || gNBTable.table[i].transport_id == transport)
//                 numLANAs++;
//         if (numLANAs > 0)
//         {
//             UCHAR lanaIndex = 0;
//
//             for (i = 0; i < gNBTable.tableSize; i++)
//                 if (gNBTable.table[i].transport_id != 0 &&
//                  (enumAll || gNBTable.table[i].transport_id == transport))
//                     cb(numLANAs, lanaIndex++, gNBTable.table[i].transport_id,
//                      &gNBTable.table[i].impl, closure);
//         }
//         LeaveCriticalSection(&gNBTable.cs);
//     }
// }
//
// static NetBIOSAdapter *nbGetAdapter(UCHAR lana)
// {
//     NetBIOSAdapter *ret = NULL;
//
//     TRACE(": lana %d, num allocated adapters %d\n", lana, gNBTable.tableSize);
//     if (lana < gNBTable.tableSize && gNBTable.table[lana].transport_id != 0
//      && gNBTable.table[lana].transport)
//         ret = &gNBTable.table[lana];
//     TRACE("returning %p\n", ret);
//     return ret;
// }
//
// static UCHAR nbEnum(PNCB ncb)
// {
//     PLANA_ENUM lanas = (PLANA_ENUM)ncb->ncb_buffer;
//     UCHAR i, ret;
//
//     TRACE(": ncb %p\n", ncb);
//
//     if (!lanas)
//         ret = NRC_BUFLEN;
//     else if (ncb->ncb_length < sizeof(LANA_ENUM))
//         ret = NRC_BUFLEN;
//     else
//     {
//         nbInternalEnum();
//         lanas->length = 0;
//         for (i = 0; i < gNBTable.tableSize; i++)
//             if (gNBTable.table[i].transport)
//             {
//                 lanas->length++;
//                 lanas->lana[i] = i;
//             }
//         ret = NRC_GOODRET;
//     }
//     TRACE("returning 0x%02x\n", ret);
//     return ret;
// }
//
// static UCHAR nbInternalHangup(NetBIOSAdapter *adapter, NetBIOSSession *session);
//
// static UCHAR nbCancel(NetBIOSAdapter *adapter, PNCB ncb)
// {
//     UCHAR ret;
//
//     TRACE(": adapter %p, ncb %p\n", adapter, ncb);
//
//     if (!adapter) return NRC_BRIDGE;
//     if (!ncb) return NRC_INVADDRESS;
//
//     switch (ncb->ncb_command & 0x7f)
//     {
//         case NCBCANCEL:
//         case NCBADDNAME:
//         case NCBADDGRNAME:
//         case NCBDELNAME:
//         case NCBRESET:
//         case NCBSSTAT:
//             ret = NRC_CANCEL;
//             break;
//
//         /* NCBCALL, NCBCHAINSEND/NCBSEND, NCBHANGUP all close the associated
//          * session if cancelled */
//         case NCBCALL:
//         case NCBSEND:
//         case NCBCHAINSEND:
//         case NCBSENDNA:
//         case NCBCHAINSENDNA:
//         case NCBHANGUP:
//         {
//             if (ncb->ncb_lsn >= adapter->sessionsLen)
//                 ret = NRC_SNUMOUT;
//             else if (!adapter->sessions[ncb->ncb_lsn].inUse)
//                 ret = NRC_SNUMOUT;
//             else
//             {
//                 ret = NBCmdQueueCancel(adapter->cmdQueue, ncb);
//                 if (ret == NRC_CMDCAN || ret == NRC_CANOCCR)
//                     nbInternalHangup(adapter, &adapter->sessions[ncb->ncb_lsn]);
//             }
//             break;
//         }
//
//         default:
//             ret = NBCmdQueueCancel(adapter->cmdQueue, ncb);
//     }
//     TRACE("returning 0x%02x\n", ret);
//     return ret;
// }
//
// /* Resizes adapter to contain space for at least sessionsLen sessions.
//  * If allocating more space for sessions, sets the adapter's sessionsLen to
//  * sessionsLen.  If the adapter's sessionsLen was already at least sessionsLen,
//  * does nothing.  Does not modify existing sessions.  Assumes the adapter is
//  * locked.
//  * Returns NRC_GOODRET on success, and something else on failure.
//  */
// static UCHAR nbResizeAdapter(NetBIOSAdapter *adapter, UCHAR sessionsLen)
// {
//     UCHAR ret = NRC_GOODRET;
//
//     if (adapter && adapter->sessionsLen < sessionsLen)
//     {
//         NetBIOSSession *newSessions;
//
//         if (adapter->sessions)
//             newSessions = HeapReAlloc(GetProcessHeap(),
//              HEAP_ZERO_MEMORY, adapter->sessions, sessionsLen *
//              sizeof(NetBIOSSession));
//         else
//             newSessions = HeapAlloc(GetProcessHeap(),
//              HEAP_ZERO_MEMORY, sessionsLen * sizeof(NetBIOSSession));
//         if (newSessions)
//         {
//             adapter->sessions = newSessions;
//             adapter->sessionsLen = sessionsLen;
//         }
//         else
//             ret = NRC_OSRESNOTAV;
//     }
//     return ret;
// }
//
// static UCHAR nbReset(NetBIOSAdapter *adapter, PNCB ncb)
// {
//     UCHAR ret;
//
//     TRACE(": adapter %p, ncb %p\n", adapter, ncb);
//
//     if (!adapter) return NRC_BRIDGE;
//     if (!ncb) return NRC_INVADDRESS;
//
//     if (InterlockedIncrement(&adapter->resetting) == 1)
//     {
//         UCHAR i, resizeTo;
//
//         NBCmdQueueCancelAll(adapter->cmdQueue);
//
//         EnterCriticalSection(&adapter->cs);
//         for (i = 0; i < adapter->sessionsLen; i++)
//             if (adapter->sessions[i].inUse)
//                 nbInternalHangup(adapter, &adapter->sessions[i]);
//         if (!ncb->ncb_lsn)
//             resizeTo = ncb->ncb_callname[0] == 0 ? DEFAULT_NUM_SESSIONS :
//              ncb->ncb_callname[0];
//         else if (adapter->sessionsLen == 0)
//             resizeTo = DEFAULT_NUM_SESSIONS;
//         else
//             resizeTo = 0;
//         if (resizeTo > 0)
//             ret = nbResizeAdapter(adapter, resizeTo);
//         else
//             ret = NRC_GOODRET;
//         LeaveCriticalSection(&adapter->cs);
//     }
//     else
//         ret = NRC_TOOMANY;
//     InterlockedDecrement(&adapter->resetting);
//     TRACE("returning 0x%02x\n", ret);
//     return ret;
// }
//
// static UCHAR nbSStat(NetBIOSAdapter *adapter, PNCB ncb)
// {
//     UCHAR ret, i, spaceFor;
//     PSESSION_HEADER sstat;
//
//     TRACE(": adapter %p, NCB %p\n", adapter, ncb);
//
//     if (!adapter) return NRC_BADDR;
//     if (adapter->sessionsLen == 0) return NRC_ENVNOTDEF;
//     if (!ncb) return NRC_INVADDRESS;
//     if (!ncb->ncb_buffer) return NRC_BADDR;
//     if (ncb->ncb_length < sizeof(SESSION_HEADER)) return NRC_BUFLEN;
//
//     sstat = (PSESSION_HEADER)ncb->ncb_buffer;
//     ret = NRC_GOODRET;
//     memset(sstat, 0, sizeof(SESSION_HEADER));
//     spaceFor = (ncb->ncb_length - sizeof(SESSION_HEADER)) /
//      sizeof(SESSION_BUFFER);
//     EnterCriticalSection(&adapter->cs);
//     for (i = 0; ret == NRC_GOODRET && i < adapter->sessionsLen; i++)
//     {
//         if (adapter->sessions[i].inUse && (ncb->ncb_name[0] == '*' ||
//          !memcmp(ncb->ncb_name, adapter->sessions[i].local_name, NCBNAMSZ)))
//         {
//             if (sstat->num_sess < spaceFor)
//             {
//                 PSESSION_BUFFER buf;
//                
//                 buf = (PSESSION_BUFFER)((PUCHAR)sstat + sizeof(SESSION_HEADER)
//                  + sstat->num_sess * sizeof(SESSION_BUFFER));
//                 buf->lsn = i;
//                 buf->state = adapter->sessions[i].state;
//                 memcpy(buf->local_name, adapter->sessions[i].local_name,
//                  NCBNAMSZ);
//                 memcpy(buf->remote_name, adapter->sessions[i].remote_name,
//                  NCBNAMSZ);
//                 buf->rcvs_outstanding = buf->sends_outstanding = 0;
//                 sstat->num_sess++;
//             }
//             else
//                 ret = NRC_BUFLEN;
//         }
//     }
//     LeaveCriticalSection(&adapter->cs);
//
//     TRACE("returning 0x%02x\n", ret);
//     return ret;
// }
//
// static UCHAR nbCall(NetBIOSAdapter *adapter, PNCB ncb)
// {
//     UCHAR ret, i;
//
//     TRACE(": adapter %p, NCB %p\n", adapter, ncb);
//
//     if (!adapter) return NRC_BRIDGE;
//     if (adapter->sessionsLen == 0) return NRC_ENVNOTDEF;
//     if (!adapter->transport->call) return NRC_ILLCMD;
//     if (!ncb) return NRC_INVADDRESS;
//
//     EnterCriticalSection(&adapter->cs);
//     for (i = 0; i < adapter->sessionsLen && adapter->sessions[i].inUse; i++)
//         ;
//     if (i < adapter->sessionsLen)
//     {
//         adapter->sessions[i].inUse = TRUE;
//         adapter->sessions[i].state = CALL_PENDING;
//         memcpy(adapter->sessions[i].local_name, ncb->ncb_name, NCBNAMSZ);
//         memcpy(adapter->sessions[i].remote_name, ncb->ncb_callname, NCBNAMSZ);
//         ret = NRC_GOODRET;
//     }
//     else
//         ret = NRC_LOCTFUL;
//     LeaveCriticalSection(&adapter->cs);
//
//     if (ret == NRC_GOODRET)
//     {
//         ret = adapter->transport->call(adapter->impl.data, ncb,
//          &adapter->sessions[i].data);
//         if (ret == NRC_GOODRET)
//         {
//             ncb->ncb_lsn = i;
//             adapter->sessions[i].state = SESSION_ESTABLISHED;
//         }
//         else
//         {
//             adapter->sessions[i].inUse = FALSE;
//             adapter->sessions[i].state = 0;
//         }
//     }
//     TRACE("returning 0x%02x\n", ret);
//     return ret;
// }
//
// static UCHAR nbSend(NetBIOSAdapter *adapter, PNCB ncb)
// {
//     UCHAR ret;
//     NetBIOSSession *session;
//
//     if (!adapter) return NRC_BRIDGE;
//     if (!adapter->transport->send) return NRC_ILLCMD;
//     if (!ncb) return NRC_INVADDRESS;
//     if (ncb->ncb_lsn >= adapter->sessionsLen) return NRC_SNUMOUT;
//     if (!adapter->sessions[ncb->ncb_lsn].inUse) return NRC_SNUMOUT;
//     if (!ncb->ncb_buffer) return NRC_BADDR;
//
//     session = &adapter->sessions[ncb->ncb_lsn];
//     if (session->state != SESSION_ESTABLISHED)
//         ret = NRC_SNUMOUT;
//     else
//         ret = adapter->transport->send(adapter->impl.data, session->data, ncb);
//     return ret;
// }
//
// static UCHAR nbRecv(NetBIOSAdapter *adapter, PNCB ncb)
// {
//     UCHAR ret;
//     NetBIOSSession *session;
//
//     if (!adapter) return NRC_BRIDGE;
//     if (!adapter->transport->recv) return NRC_ILLCMD;
//     if (!ncb) return NRC_INVADDRESS;
//     if (ncb->ncb_lsn >= adapter->sessionsLen) return NRC_SNUMOUT;
//     if (!adapter->sessions[ncb->ncb_lsn].inUse) return NRC_SNUMOUT;
//     if (!ncb->ncb_buffer) return NRC_BADDR;
//
//     session = &adapter->sessions[ncb->ncb_lsn];
//     if (session->state != SESSION_ESTABLISHED)
//         ret = NRC_SNUMOUT;
//     else
//         ret = adapter->transport->recv(adapter->impl.data, session->data, ncb);
//     return ret;
// }
//
// static UCHAR nbInternalHangup(NetBIOSAdapter *adapter, NetBIOSSession *session)
// {
//     UCHAR ret;
//
//     if (!adapter) return NRC_BRIDGE;
//     if (!session) return NRC_SNUMOUT;
//
//     if (adapter->transport->hangup)
//         ret = adapter->transport->hangup(adapter->impl.data, session->data);
//     else
//         ret = NRC_ILLCMD;
//     EnterCriticalSection(&adapter->cs);
//     memset(session, 0, sizeof(NetBIOSSession));
//     LeaveCriticalSection(&adapter->cs);
//     return ret;
// }
//
// static UCHAR nbHangup(NetBIOSAdapter *adapter, const NCB *ncb)
// {
//     UCHAR ret;
//     NetBIOSSession *session;
//
//     if (!adapter) return NRC_BRIDGE;
//     if (!ncb) return NRC_INVADDRESS;
//     if (ncb->ncb_lsn >= adapter->sessionsLen) return NRC_SNUMOUT;
//     if (!adapter->sessions[ncb->ncb_lsn].inUse) return NRC_SNUMOUT;
//
//     session = &adapter->sessions[ncb->ncb_lsn];
//     if (session->state != SESSION_ESTABLISHED)
//         ret = NRC_SNUMOUT;
//     else
//     {
//         session->state = HANGUP_PENDING;
//         ret = nbInternalHangup(adapter, session);
//     }
//     return ret;
// }
//
// void NetBIOSHangupSession(const NCB *ncb)
// {
//     NetBIOSAdapter *adapter;
//
//     if (!ncb) return;
//
//     adapter = nbGetAdapter(ncb->ncb_lana_num);
//     if (adapter)
//     {
//         if (ncb->ncb_lsn < adapter->sessionsLen &&
//          adapter->sessions[ncb->ncb_lsn].inUse)
//             nbHangup(adapter, ncb);
//     }
// }
//
// static UCHAR nbAStat(NetBIOSAdapter *adapter, PNCB ncb)
// {
//     UCHAR ret;
//
//     if (!adapter) return NRC_BRIDGE;
//     if (!adapter->transport->astat) return NRC_ILLCMD;
//     if (!ncb) return NRC_INVADDRESS;
//     if (!ncb->ncb_buffer) return NRC_BADDR;
//     if (ncb->ncb_length < sizeof(ADAPTER_STATUS)) return NRC_BUFLEN;
//
//     ret = adapter->transport->astat(adapter->impl.data, ncb);
//     if (ncb->ncb_callname[0] == '*')
//     {
//         PADAPTER_STATUS astat = (PADAPTER_STATUS)ncb->ncb_buffer;
//
//         astat->max_sess = astat->max_cfg_sess = adapter->sessionsLen;
//     }
//     return ret;
// }
//
// static UCHAR nbDispatch(NetBIOSAdapter *adapter, PNCB ncb)
// {
//     UCHAR ret, cmd;
//
//     TRACE(": adapter %p, ncb %p\n", adapter, ncb);
//
//     if (!adapter) return NRC_BRIDGE;
//     if (!ncb) return NRC_INVADDRESS;
//
//     cmd = ncb->ncb_command & 0x7f;
//     if (cmd == NCBRESET)
//         ret = nbReset(adapter, ncb);
//     else
//     {
//         ret = NBCmdQueueAdd(adapter->cmdQueue, ncb);
//         if (ret == NRC_GOODRET)
//         {
//             switch (cmd)
//             {
//                 case NCBCALL:
//                     ret = nbCall(adapter, ncb);
//                     break;
//
//                 /* WinNT doesn't chain sends, it always sends immediately.
//                  * Doubt there's any real significance to the NA variants.
//                  */
//                 case NCBSEND:
//                 case NCBSENDNA:
//                 case NCBCHAINSEND:
//                 case NCBCHAINSENDNA:
//                     ret = nbSend(adapter, ncb);
//                     break;
//
//                 case NCBRECV:
//                     ret = nbRecv(adapter, ncb);
//                     break;
//
//                 case NCBHANGUP:
//                     ret = nbHangup(adapter, ncb);
//                     break;
//
//                 case NCBASTAT:
//                     ret = nbAStat(adapter, ncb);
//                     break;
//
//                 case NCBFINDNAME:
//                     if (adapter->transport->findName)
//                         ret = adapter->transport->findName(adapter->impl.data,
//                          ncb);
//                     else
//                         ret = NRC_ILLCMD;
//                     break;
//
//                 default:
//                     FIXME("(%p): command code 0x%02x\n", ncb, ncb->ncb_command);
//                     ret = NRC_ILLCMD;
//             }
//             NBCmdQueueComplete(adapter->cmdQueue, ncb, ret);
//         }
//     }
//     TRACE("returning 0x%02x\n", ret);
//     return ret;
// }
//
// static DWORD WINAPI nbCmdThread(LPVOID lpVoid)
// {
//     PNCB ncb = lpVoid;
//
//     if (ncb)
//     {
//         UCHAR ret;
//         NetBIOSAdapter *adapter = nbGetAdapter(ncb->ncb_lana_num);
//
//         if (adapter)
//             ret = nbDispatch(adapter, ncb);
//         else
//             ret = NRC_BRIDGE;
//         ncb->ncb_retcode = ncb->ncb_cmd_cplt = ret;
//         if (ncb->ncb_post)
//             ncb->ncb_post(ncb);
//         else if (ncb->ncb_event)
//             SetEvent(ncb->ncb_event);
//     }
//     return 0;
// }
//
// UCHAR WINAPI Netbios(PNCB ncb)
// {
//     UCHAR ret, cmd;
//
//     TRACE("ncb = %p\n", ncb);
//
//     if (!ncb) return NRC_INVADDRESS;
//
//     TRACE("ncb_command 0x%02x, ncb_lana_num %d, ncb_buffer %p, ncb_length %d\n",
//      ncb->ncb_command, ncb->ncb_lana_num, ncb->ncb_buffer, ncb->ncb_length);
//     cmd = ncb->ncb_command & 0x7f;
//
//     if (cmd == NCBENUM)
//         ncb->ncb_retcode = ncb->ncb_cmd_cplt = ret = nbEnum(ncb);
//     else if (cmd == NCBADDNAME)
//     {
//         FIXME("NCBADDNAME: stub, returning success\n");
//         ncb->ncb_retcode = ncb->ncb_cmd_cplt = ret = NRC_GOODRET;
//     }
//     else
//     {
//         NetBIOSAdapter *adapter;
//
//         /* Apps not specifically written for WinNT won't do an NCBENUM first,
//          * so make sure the table has been enumerated at least once
//          */
//         if (!gNBTable.enumerated)
//             nbInternalEnum();
//         adapter = nbGetAdapter(ncb->ncb_lana_num);
//         if (!adapter)
//             ret = NRC_BRIDGE;
//         else
//         {
//             if (adapter->shuttingDown)
//                 ret = NRC_IFBUSY;
//             else if (adapter->resetting)
//                 ret = NRC_TOOMANY;
//             else
//             {
//                 /* non-asynch commands first */
//                 if (cmd == NCBCANCEL)
//                     ncb->ncb_retcode = ncb->ncb_cmd_cplt = ret =
//                      nbCancel(adapter, ncb);
//                 else if (cmd == NCBSSTAT)
//                     ncb->ncb_retcode = ncb->ncb_cmd_cplt = ret =
//                      nbSStat(adapter, ncb);
//                 else
//                 {
//                     if (ncb->ncb_command & ASYNCH)
//                     {
//                         HANDLE thread = CreateThread(NULL, 0, nbCmdThread, ncb,
//                          CREATE_SUSPENDED, NULL);
//
//                         if (thread != NULL)
//                         {
//                             ncb->ncb_retcode = ncb->ncb_cmd_cplt = NRC_PENDING;
//                             if (ncb->ncb_event)
//                                 ResetEvent(ncb->ncb_event);
//                             ResumeThread(thread);
//                             CloseHandle(thread);
//                             ret = NRC_GOODRET;
//                         }
//                         else
//                         ncb->ncb_retcode = ncb->ncb_cmd_cplt = ret =
//                          NRC_OSRESNOTAV;
//                     }
//                     else
//                         ncb->ncb_retcode = ncb->ncb_cmd_cplt = ret =
//                          nbDispatch(adapter, ncb);
//                 }
//             }
//         }
//     }
//     TRACE("returning 0x%02x\n", ret);
//     return ret;
// }
//
// DWORD WINAPI NetpNetBiosStatusToApiStatus(DWORD nrc)
// {
//     DWORD ret;
//
//     switch (nrc)
//     {
//         case NRC_GOODRET:
//             ret = NO_ERROR;
//             break;
//         case NRC_NORES:
//             ret = NERR_NoNetworkResource;
//             break;
//         case NRC_DUPNAME:
//             ret = NERR_AlreadyExists;
//             break;
//         case NRC_NAMTFUL:
//             ret = NERR_TooManyNames;
//             break;
//         case NRC_ACTSES:
//             ret = NERR_DeleteLater;
//             break;
//         case NRC_REMTFUL:
//             ret = ERROR_REM_NOT_LIST;
//             break;
//         case NRC_NOCALL:
//             ret = NERR_NameNotFound;
//             break;
//         case NRC_NOWILD:
//             ret = ERROR_INVALID_PARAMETER;
//             break;
//         case NRC_INUSE:
//             ret = NERR_DuplicateName;
//             break;
//         case NRC_NAMERR:
//             ret = ERROR_INVALID_PARAMETER;
//             break;
//         case NRC_NAMCONF:
//             ret = NERR_DuplicateName;
//             break;
//         default:
//             ret = NERR_NetworkError;
//     }
//     return ret;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
