use crate::prelude::*;
use crate::types::*;
use core::ffi::c_void;
use crate::src::dcatitem::WsTcEntryAllocate;
use crate::src::dcatitem::WsTcEntryInitializeFromRegistry;

extern "system" {
}

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
pub static mut HostnameGuid: GUID = _GUID {
    Data1: 0x2a800 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0 as ::core::ffi::c_ushort,
    Data3: 0 as ::core::ffi::c_ushort,
    Data4: [
        0xc0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x46 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut AddressGuid: GUID = _GUID {
    Data1: 0x2a801 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0 as ::core::ffi::c_ushort,
    Data3: 0 as ::core::ffi::c_ushort,
    Data4: [
        0xc0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x46 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut HostAddrByNameGuid: GUID = _GUID {
    Data1: 0x2a803 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0 as ::core::ffi::c_ushort,
    Data3: 0 as ::core::ffi::c_ushort,
    Data4: [
        0xc0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x46 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut IANAGuid: GUID = _GUID {
    Data1: 0x2a802 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0 as ::core::ffi::c_ushort,
    Data3: 0 as ::core::ffi::c_ushort,
    Data4: [
        0xc0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x46 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut InetHostName: GUID = _GUID {
    Data1: 0x2a803 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0 as ::core::ffi::c_ushort,
    Data3: 0 as ::core::ffi::c_ushort,
    Data4: [
        0xc0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x46 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut Ipv6Guid: GUID = _GUID {
    Data1: ((0x9 as ::core::ffi::c_int) << 16 as ::core::ffi::c_int | 53 as ::core::ffi::c_int)
        as ::core::ffi::c_uint,
    Data2: 0 as ::core::ffi::c_ushort,
    Data3: 0x1c as ::core::ffi::c_ushort,
    Data4: [
        0xc0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x46 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

// No file-scope declarations to translate.
// The C preamble contains only preprocessor macros:
// - NDEBUG is a preprocessor flag (no Rust equivalent needed)
// - WsTcLock() and WsTcUnlock() are function-like macros that will be inlined in the translated functions
#[no_mangle]
pub unsafe extern "system" fn WsTcAllocate() -> PTCATALOG {
    let mut Catalog: PTCATALOG = core::ptr::null_mut();

    Catalog = HeapAlloc(
        WsSockHeap,
        HEAP_ZERO_MEMORY,
        (core::mem::size_of::<TCATALOG>() as SIZE_T),
    ) as PTCATALOG;

    return Catalog;
}
#[no_mangle]
pub unsafe extern "system" fn WsTcOpen(Catalog: PTCATALOG, ParentKey: HKEY) -> BOOL {
    let mut ErrorCode: INT;
    let mut CreateDisposition: DWORD = unsafe { core::mem::zeroed() };
    let mut CatalogKey: HKEY = core::ptr::null_mut();
    let mut NewKey: HKEY = core::ptr::null_mut();
    let mut RegType: DWORD = REG_DWORD;
    let mut RegSize: DWORD = core::mem::size_of::<DWORD>() as DWORD;
    let mut UniqueId: DWORD = 0;
    let mut NewData: DWORD = 0;
    let mut CatalogKeyName: *mut CHAR;

    InitializeCriticalSection((core::ptr::addr_of_mut!((*Catalog).Lock) as *mut c_void));
    InitializeListHead(core::ptr::addr_of_mut!((*Catalog).ProtocolList));

    ErrorCode = RegQueryValueExA(
        ParentKey,
        b"Current_Protocol_Catalog\0".as_ptr() as LPCSTR,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(RegType),
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(RegSize),
    );

    if ErrorCode == (ERROR_FILE_NOT_FOUND as i32) {
        static DefaultCatalogName: [CHAR; 18] = unsafe { core::mem::transmute(*b"Protocol_Catalog9\0") };
        RegSize = core::mem::size_of::<[CHAR; 18]>() as DWORD;
        CatalogKeyName = HeapAlloc(WsSockHeap, 0, RegSize as SIZE_T) as *mut CHAR;
        core::ptr::copy_nonoverlapping(
            DefaultCatalogName.as_ptr(),
            CatalogKeyName,
            RegSize as usize,
        );
    } else {
        if ErrorCode != (ERROR_SUCCESS as i32) {
            return (0u8 as i32);
        }

        if RegType != REG_SZ {
            return (0u8 as i32);
        }

        CatalogKeyName = HeapAlloc(WsSockHeap, 0, RegSize as SIZE_T) as *mut CHAR;

        ErrorCode = RegQueryValueExA(
            ParentKey,
            b"Current_Protocol_Catalog\0".as_ptr() as LPCSTR,
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!(RegType),
            CatalogKeyName as LPBYTE,
            core::ptr::addr_of_mut!(RegSize),
        );

        ErrorCode = RegOpenKeyExA(
            ParentKey,
            CatalogKeyName as LPCSTR,
            0,
            MAXIMUM_ALLOWED,
            core::ptr::addr_of_mut!(CatalogKey),
        );
    }

    if ErrorCode == (ERROR_SUCCESS as i32) {
        CreateDisposition = REG_OPENED_EXISTING_KEY;
    } else if ErrorCode == (ERROR_FILE_NOT_FOUND as i32) {
        ErrorCode = RegCreateKeyExA(
            ParentKey,
            CatalogKeyName as LPCSTR,
            0,
            core::ptr::null_mut(),
            REG_OPTION_NON_VOLATILE,
            KEY_ALL_ACCESS,
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!(CatalogKey),
            core::ptr::addr_of_mut!(CreateDisposition),
        );
    }

    HeapFree(WsSockHeap, 0, CatalogKeyName as PVOID);
    RegType = REG_DWORD;
    RegSize = core::mem::size_of::<DWORD>() as DWORD;

    if ErrorCode != (ERROR_SUCCESS as i32) {
        return (0u8 as i32);
    }

    if CreateDisposition == REG_CREATED_NEW_KEY {
        ErrorCode = RegSetValueExA(
            CatalogKey,
            b"Num_Catalog_Entries\0".as_ptr() as LPCSTR,
            0,
            REG_DWORD,
            core::ptr::addr_of!(NewData) as *const BYTE,
            core::mem::size_of::<DWORD>() as DWORD,
        );
        if ErrorCode != (ERROR_SUCCESS as i32) {
            RegCloseKey(CatalogKey);
            return (0u8 as i32);
        }

        NewData = 1001;
        ErrorCode = RegSetValueExA(
            CatalogKey,
            b"Next_Catalog_Entry_ID\0".as_ptr() as LPCSTR,
            0,
            REG_DWORD,
            core::ptr::addr_of!(NewData) as *const BYTE,
            core::mem::size_of::<DWORD>() as DWORD,
        );
        if ErrorCode != (ERROR_SUCCESS as i32) {
            RegCloseKey(CatalogKey);
            return (0u8 as i32);
        }

        NewData = 1;
        ErrorCode = RegSetValueExA(
            CatalogKey,
            b"Serial_Access_Num\0".as_ptr() as LPCSTR,
            0,
            REG_DWORD,
            core::ptr::addr_of!(NewData) as *const BYTE,
            core::mem::size_of::<DWORD>() as DWORD,
        );
        if ErrorCode != (ERROR_SUCCESS as i32) {
            RegCloseKey(CatalogKey);
            return (0u8 as i32);
        }

        ErrorCode = RegCreateKeyExA(
            CatalogKey,
            b"Catalog_Entries\0".as_ptr() as LPCSTR,
            0,
            core::ptr::null_mut(),
            REG_OPTION_NON_VOLATILE,
            KEY_ALL_ACCESS,
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!(NewKey),
            core::ptr::addr_of_mut!(CreateDisposition),
        );
        if ErrorCode != (ERROR_SUCCESS as i32) {
            RegCloseKey(CatalogKey);
            return (0u8 as i32);
        }

        RegCloseKey(NewKey);
    } else {
        RegSize = core::mem::size_of::<DWORD>() as DWORD;
        ErrorCode = RegQueryValueExA(
            CatalogKey,
            b"Serial_Access_Num\0".as_ptr() as LPCSTR,
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!(RegType),
            core::ptr::addr_of_mut!(UniqueId) as LPBYTE,
            core::ptr::addr_of_mut!(RegSize),
        );

        if ErrorCode != (ERROR_SUCCESS as i32) {
            NewData = 1;
            ErrorCode = RegSetValueExA(
                CatalogKey,
                b"Serial_Access_Num\0".as_ptr() as LPCSTR,
                0,
                REG_DWORD,
                core::ptr::addr_of!(NewData) as *const BYTE,
                core::mem::size_of::<DWORD>() as DWORD,
            );
        }
    }

    (*Catalog).CatalogKey = CatalogKey;
    (1u8 as i32)
}
#[no_mangle]
pub unsafe extern "system" fn WsTcInitializeFromRegistry(
    Catalog: PTCATALOG,
    ParentKey: HKEY,
    CatalogEvent: HANDLE,
) -> DWORD {
    let mut ErrorCode: INT = WSASYSCALLFAILURE as INT;

    if WsTcOpen(Catalog, ParentKey) != 0 {
        ErrorCode = WsTcRefreshFromRegistry(Catalog, CatalogEvent) as INT;
    }

    ErrorCode as DWORD
}

#[no_mangle]
pub unsafe extern "system" fn WsTcRefreshFromRegistry(
    Catalog: PTCATALOG,
    mut CatalogEvent: *mut core::ffi::c_void,
) -> u32 {
    let mut ErrorCode: i32;
    let mut LocalEvent: u8 = 0u8;
    let mut LocalList: LIST_ENTRY = LIST_ENTRY {
        Flink: core::ptr::null_mut(),
        Blink: core::ptr::null_mut(),
    };
    let mut UniqueId: u32 = 0;
    let mut EntriesKey: HKEY = core::ptr::null_mut();
    let mut CatalogEntries: u32 = 0;
    let mut CatalogEntry: PTCATALOG_ENTRY;
    let mut NextCatalogEntry: u32 = 0;
    let mut NewChangesMade: i32;
    let mut Entry: PLIST_ENTRY;
    let mut RegType: u32 = REG_DWORD;
    let mut RegSize: u32 = core::mem::size_of::<u32>() as u32;
    let mut i: u32;

    if CatalogEvent.is_null() {
        CatalogEvent = CreateEvent(
            core::ptr::null_mut(),
            0,
            0,
            core::ptr::null(),
        );
        if CatalogEvent.is_null() {
            return WSASYSCALLFAILURE;
        }
        LocalEvent = 1u8;
    }

    EnterCriticalSection(&mut (*Catalog).Lock);

    InitializeListHead(core::ptr::addr_of_mut!(LocalList));

    loop {
        ErrorCode = WsSetupCatalogProtection(
            (*Catalog).CatalogKey,
            CatalogEvent,
            core::ptr::addr_of_mut!(UniqueId),
        );
        if ErrorCode != ERROR_SUCCESS as i32 {
            break;
        }

        if UniqueId == (*Catalog).UniqueId {
            ErrorCode = ERROR_SUCCESS as i32;
            break;
        }

        ErrorCode = RegOpenKeyEx(
            (*Catalog).CatalogKey,
            b"Catalog_Entries\0".as_ptr() as *const i8,
            0,
            MAXIMUM_ALLOWED,
            core::ptr::addr_of_mut!(EntriesKey),
        );
        if ErrorCode != ERROR_SUCCESS as i32 {
            ErrorCode = WSASYSCALLFAILURE as i32;
            break;
        }

        ErrorCode = RegQueryValueEx(
            (*Catalog).CatalogKey,
            b"Next_Catalog_Entry_ID\0".as_ptr() as *const i8,
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!(RegType),
            core::ptr::addr_of_mut!(NextCatalogEntry) as LPBYTE,
            core::ptr::addr_of_mut!(RegSize),
        );
        if ErrorCode != ERROR_SUCCESS as i32 {
            ErrorCode = WSASYSCALLFAILURE as i32;
            break;
        }

        ErrorCode = RegQueryValueEx(
            (*Catalog).CatalogKey,
            b"Num_Catalog_Entries\0".as_ptr() as *const i8,
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!(RegType),
            core::ptr::addr_of_mut!(CatalogEntries) as LPBYTE,
            core::ptr::addr_of_mut!(RegSize),
        );
        if ErrorCode != ERROR_SUCCESS as i32 {
            ErrorCode = WSASYSCALLFAILURE as i32;
            break;
        }

        i = 1;
        while i <= CatalogEntries {
            CatalogEntry = WsTcEntryAllocate();
            if CatalogEntry.is_null() {
                ErrorCode = WSA_NOT_ENOUGH_MEMORY as i32;
                break;
            }

            ErrorCode = (WsTcEntryInitializeFromRegistry(
                CatalogEntry,
                EntriesKey,
                i,
            ) as i32);
            if ErrorCode != ERROR_SUCCESS as i32 {
                WsTcEntryDereference(CatalogEntry);
                break;
            }

            InsertTailList(
                core::ptr::addr_of_mut!(LocalList),
                core::ptr::addr_of_mut!((*CatalogEntry).CatalogLink),
            );
            i = i.wrapping_add(1);
        }

        RegCloseKey(EntriesKey);

        NewChangesMade = WsCheckCatalogState(CatalogEvent);
        if NewChangesMade == 0 && ErrorCode == ERROR_SUCCESS as i32 {
            WsTcUpdateProtocolList(Catalog, core::ptr::addr_of_mut!(LocalList));
            (*Catalog).UniqueId = UniqueId;
            (*Catalog).NextId = NextCatalogEntry;
            break;
        }

        while !IsListEmpty(core::ptr::addr_of_mut!(LocalList)) {
            Entry = RemoveHeadList(core::ptr::addr_of_mut!(LocalList));
            CatalogEntry = (Entry as usize)
                .wrapping_sub(
                    core::mem::offset_of!(TCATALOG_ENTRY, CatalogLink)
                ) as PTCATALOG_ENTRY;
            WsTcEntryDereference(CatalogEntry);
        }

        if NewChangesMade == 0 {
            break;
        }
    }

    LeaveCriticalSection(&mut (*Catalog).Lock);

    if LocalEvent != 0 {
        CloseHandle(CatalogEvent);
    }

    ErrorCode as u32
}

#[no_mangle]
pub unsafe extern "system" fn WsTcGetEntryFromAf(
    Catalog: PTCATALOG,
    AddressFamily: i32,
    CatalogEntry: *mut PTCATALOG_ENTRY,
) -> u32 {
    let mut ErrorCode: i32 = (WSAEINVAL as i32);
    let mut NextEntry: PLIST_ENTRY;
    let mut Entry: PTCATALOG_ENTRY;

    *CatalogEntry = core::ptr::null_mut();

    EnterCriticalSection(core::ptr::addr_of_mut!((*Catalog).Lock));

    NextEntry = (*Catalog).ProtocolList.Flink;
    while NextEntry != core::ptr::addr_of_mut!((*Catalog).ProtocolList) {
        Entry = (NextEntry as usize - core::mem::offset_of!(TCATALOG_ENTRY, CatalogLink)) as PTCATALOG_ENTRY;
        NextEntry = (*NextEntry).Flink;

        if (*Entry).ProtocolInfo.iAddressFamily == AddressFamily {
            if (*Entry).Provider.is_null() {
                ErrorCode = WsTcLoadProvider(Catalog, Entry) as i32;

                if ErrorCode != ERROR_SUCCESS as i32 {
                    break;
                }
            }

            InterlockedIncrement(core::ptr::addr_of_mut!((*Entry).RefCount));
            *CatalogEntry = Entry;
            ErrorCode = ERROR_SUCCESS as i32;
            break;
        }
    }

    LeaveCriticalSection(core::ptr::addr_of_mut!((*Catalog).Lock));

    ErrorCode as u32
}

#[no_mangle]
pub unsafe extern "system" fn WsTcGetEntryFromCatalogEntryId(
    Catalog: PTCATALOG,
    CatalogEntryId: DWORD,
    CatalogEntry: *mut PTCATALOG_ENTRY,
) -> DWORD {
    let mut ErrorCode: INT = (WSAEINVAL as i32);
    let mut NextEntry: PLIST_ENTRY;
    let mut Entry: PTCATALOG_ENTRY;

    EnterCriticalSection(core::ptr::addr_of_mut!((*Catalog).Lock));

    NextEntry = (*Catalog).ProtocolList.Flink;
    while NextEntry != core::ptr::addr_of_mut!((*Catalog).ProtocolList) {
        Entry = (NextEntry as usize - core::mem::offset_of!(TCATALOG_ENTRY, CatalogLink)) as PTCATALOG_ENTRY;
        NextEntry = (*NextEntry).Flink;

        if (*Entry).ProtocolInfo.dwCatalogEntryId == CatalogEntryId {
            if (*Entry).Provider.is_null() {
                WsTcLoadProvider(Catalog, Entry);
            }

            InterlockedIncrement(core::ptr::addr_of_mut!((*Entry).RefCount));
            *CatalogEntry = Entry;
            ErrorCode = ERROR_SUCCESS as INT;
            break;
        }
    }

    LeaveCriticalSection(core::ptr::addr_of_mut!((*Catalog).Lock));

    return ErrorCode as DWORD;
}

#[no_mangle]
pub unsafe extern "system" fn WsTcGetEntryFromTriplet(
    Catalog: PTCATALOG,
    af: i32,
    type_: i32,
    protocol: i32,
    StartId: u32,
    CatalogEntry: *mut PTCATALOG_ENTRY,
) -> u32 {
    let mut ErrorCode: i32 = WSAEINVAL as i32;
    let mut NextEntry: PLIST_ENTRY;
    let mut Entry: PTCATALOG_ENTRY;

    *CatalogEntry = core::ptr::null_mut();

    EnterCriticalSection(&mut (*Catalog).Lock);

    NextEntry = (*Catalog).ProtocolList.Flink;

    if StartId != 0 {
        while NextEntry != core::ptr::addr_of_mut!((*Catalog).ProtocolList) {
            Entry = (NextEntry as usize).wrapping_sub(
                core::mem::offset_of!(TCATALOG_ENTRY, CatalogLink)
            ) as PTCATALOG_ENTRY;
            NextEntry = (*NextEntry).Flink;

            if (*Entry).ProtocolInfo.dwCatalogEntryId == StartId {
                break;
            }
        }
    }

    while NextEntry != core::ptr::addr_of_mut!((*Catalog).ProtocolList) {
        Entry = (NextEntry as usize).wrapping_sub(
            core::mem::offset_of!(TCATALOG_ENTRY, CatalogLink)
        ) as PTCATALOG_ENTRY;
        NextEntry = (*NextEntry).Flink;

        if (*Entry).ProtocolInfo.iAddressFamily == af || af == (AF_UNSPEC as i32) {
            if (*Entry).ProtocolInfo.iSocketType == type_ || type_ == 0 {
                if ((*Entry).ProtocolInfo.iProtocol <= protocol
                    && (*Entry).ProtocolInfo.iProtocol + (*Entry).ProtocolInfo.iProtocolMaxOffset >= protocol)
                    || protocol == 0
                {
                    if (*Entry).Provider.is_null() {
                        ErrorCode = WsTcLoadProvider(Catalog, Entry) as i32;

                        if ErrorCode != ERROR_SUCCESS as i32 {
                            break;
                        }
                    }

                    InterlockedIncrement(core::ptr::addr_of_mut!((*Entry).RefCount));
                    *CatalogEntry = Entry;
                    ErrorCode = ERROR_SUCCESS as i32;
                    break;
                } else {
                    ErrorCode = WSAEPROTONOSUPPORT as i32;
                }
            } else {
                ErrorCode = WSAESOCKTNOSUPPORT as i32;
            }
        } else {
            ErrorCode = WSAEAFNOSUPPORT as i32;
        }
    }

    LeaveCriticalSection(&mut (*Catalog).Lock);

    ErrorCode as u32
}

#[no_mangle]
pub unsafe extern "system" fn WsTcFindProvider(
    Catalog: PTCATALOG,
    ProviderId: LPGUID,
) -> PTPROVIDER {
    let mut Provider: PTPROVIDER;
    let mut Entry: PLIST_ENTRY;
    let mut CatalogEntry: PTCATALOG_ENTRY;

    Entry = (*Catalog).ProtocolList.Flink;
    while Entry != core::ptr::addr_of_mut!((*Catalog).ProtocolList) {
        CatalogEntry = (Entry as usize - core::mem::offset_of!(TCATALOG_ENTRY, CatalogLink)) as PTCATALOG_ENTRY;

        Entry = (*Entry).Flink;
        Provider = (*CatalogEntry).Provider;

        if !Provider.is_null() && !IsEqualGUID(core::ptr::addr_of_mut!((*CatalogEntry).ProtocolInfo.ProviderId), ProviderId) {
            return Provider;
        }
    }

    core::ptr::null_mut()
}

#[no_mangle]
pub unsafe extern "system" fn WsTcLoadProvider(
    Catalog: PTCATALOG,
    CatalogEntry: PTCATALOG_ENTRY,
) -> DWORD {
    let mut ErrorCode: INT = ERROR_SUCCESS as INT;
    let mut Provider: PTPROVIDER;

    EnterCriticalSection(&mut (*Catalog).Lock);

    if (*CatalogEntry).Provider.is_null() {
        Provider = WsTcFindProvider(
            Catalog,
            core::ptr::addr_of_mut!((*CatalogEntry).ProtocolInfo.ProviderId),
        );

        if !Provider.is_null() {
            WsTcEntrySetProvider(CatalogEntry, Provider);
            ErrorCode = ERROR_SUCCESS as INT;
        } else {
            Provider = WsTpAllocate();
            if !Provider.is_null() {
                ErrorCode = WsTpInitialize(
                    Provider,
                    (*CatalogEntry).DllPath.as_mut_ptr(),
                    core::ptr::addr_of_mut!((*CatalogEntry).ProtocolInfo),
                ) as INT;

                if ErrorCode == ERROR_SUCCESS as INT {
                    WsTcEntrySetProvider(CatalogEntry, Provider);
                }

                WsTpDereference(Provider);
            } else {
                ErrorCode = WSA_NOT_ENOUGH_MEMORY as INT;
            }
        }
    }

    LeaveCriticalSection(&mut (*Catalog).Lock);
    ErrorCode as DWORD
}

#[no_mangle]
pub unsafe extern "system" fn WsTcUpdateProtocolList(
    Catalog: PTCATALOG,
    List: PLIST_ENTRY,
) {
    let mut TempList: LIST_ENTRY = LIST_ENTRY {
        Flink: core::ptr::null_mut(),
        Blink: core::ptr::null_mut(),
    };
    let mut CatalogEntry: PTCATALOG_ENTRY;
    let mut OldCatalogEntry: PTCATALOG_ENTRY;
    let mut Entry: PLIST_ENTRY;

    InsertHeadList(
        core::ptr::addr_of_mut!((*Catalog).ProtocolList),
        core::ptr::addr_of_mut!(TempList),
    );
    RemoveEntryList(core::ptr::addr_of_mut!((*Catalog).ProtocolList));
    InitializeListHead(core::ptr::addr_of_mut!((*Catalog).ProtocolList));

    while !IsListEmpty(List as *const LIST_ENTRY) {
        Entry = RemoveHeadList(List);
        CatalogEntry = (Entry as usize)
            .wrapping_sub(core::mem::offset_of!(TCATALOG_ENTRY, CatalogLink))
            as PTCATALOG_ENTRY;

        Entry = TempList.Flink;
        while Entry != core::ptr::addr_of_mut!(TempList) {
            OldCatalogEntry = (Entry as usize)
                .wrapping_sub(core::mem::offset_of!(TCATALOG_ENTRY, CatalogLink))
                as PTCATALOG_ENTRY;
            Entry = (*Entry).Flink;

            if (*CatalogEntry).ProtocolInfo.dwCatalogEntryId
                == (*OldCatalogEntry).ProtocolInfo.dwCatalogEntryId
            {
                WsTcEntryDereference(CatalogEntry);
                CatalogEntry = OldCatalogEntry;
                RemoveEntryList(core::ptr::addr_of_mut!((*CatalogEntry).CatalogLink));
                (*Catalog).ItemCount = (*Catalog).ItemCount.wrapping_sub(1);
                break;
            }
        }

        InsertTailList(
            core::ptr::addr_of_mut!((*Catalog).ProtocolList),
            core::ptr::addr_of_mut!((*CatalogEntry).CatalogLink),
        );
        (*Catalog).ItemCount = (*Catalog).ItemCount.wrapping_add(1);
    }

    while !IsListEmpty(core::ptr::addr_of_mut!(TempList) as *const LIST_ENTRY) {
        Entry = RemoveHeadList(core::ptr::addr_of_mut!(TempList));
        CatalogEntry = (Entry as usize)
            .wrapping_sub(core::mem::offset_of!(TCATALOG_ENTRY, CatalogLink))
            as PTCATALOG_ENTRY;

        (*Catalog).ItemCount = (*Catalog).ItemCount.wrapping_sub(1);
        WsTcEntryDereference(CatalogEntry);
    }
}

#[no_mangle]
pub unsafe extern "system" fn WsTcEnumerateCatalogItems(
    Catalog: PTCATALOG,
    Callback: PTCATALOG_ENUMERATE_PROC,
    Context: PVOID,
) {
    let mut Entry: PLIST_ENTRY;
    let mut CatalogEntry: PTCATALOG_ENTRY;
    let mut GoOn: BOOL = (TRUE as i32);

    EnterCriticalSection(core::ptr::addr_of_mut!((*Catalog).Lock));

    Entry = (*Catalog).ProtocolList.Flink;
    while GoOn != 0 && Entry != core::ptr::addr_of_mut!((*Catalog).ProtocolList) {
        CatalogEntry = (Entry as usize -
            core::mem::offset_of!(TCATALOG_ENTRY, CatalogLink))
            as PTCATALOG_ENTRY;

        Entry = (*Entry).Flink;
        GoOn = Callback.unwrap_unchecked()(Context, CatalogEntry);
    }

    LeaveCriticalSection(core::ptr::addr_of_mut!((*Catalog).Lock));
}
#[no_mangle]
pub unsafe extern "system" fn WsTcFindIfsProviderForSocket(
    Catalog: PTCATALOG,
    Handle: SOCKET,
) -> DWORD {
    let mut Provider: PTPROVIDER;
    let mut NewHandle: SOCKET;
    let mut Error: i32 = 0;
    let mut OptionLength: i32;
    let mut Entry: PLIST_ENTRY;
    let mut ProtocolInfo: WSAPROTOCOL_INFOW = core::mem::zeroed();
    let mut UniqueId: u32;
    let mut ErrorCode: i32;
    let mut CatalogEntry: PTCATALOG_ENTRY;

    EnterCriticalSection(&mut (*Catalog).Lock);

    'CatalogChanged: loop {
        Entry = (*Catalog).ProtocolList.Flink;
        while Entry != core::ptr::addr_of_mut!((*Catalog).ProtocolList) {
            CatalogEntry = (Entry as usize - core::mem::offset_of!(TCATALOG_ENTRY, CatalogLink)) as PTCATALOG_ENTRY;

            Entry = (*Entry).Flink;

            if ((*CatalogEntry).ProtocolInfo.dwServiceFlags1 & XP1_IFS_HANDLES) != 0 {
                continue;
            }

            Provider = (*CatalogEntry).Provider;
            if Provider.is_null() {
                ErrorCode = WsTcLoadProvider(Catalog, CatalogEntry) as i32;

                if ErrorCode != ERROR_SUCCESS as i32 {
                    continue;
                }

                Provider = (*CatalogEntry).Provider;
            }

            InterlockedIncrement(core::ptr::addr_of_mut!((*CatalogEntry).RefCount));
            UniqueId = (*Catalog).UniqueId;

            LeaveCriticalSection(&mut (*Catalog).Lock);

            OptionLength = core::mem::size_of::<WSAPROTOCOL_INFOW>() as i32;
            ErrorCode = ((*Provider).Service.lpWSPGetSockOpt.unwrap_unchecked())(
                Handle,
                SOL_SOCKET as i32,
                SO_PROTOCOL_INFOW as i32,
                core::ptr::addr_of_mut!(ProtocolInfo) as *mut i8,
                core::ptr::addr_of_mut!(OptionLength),
                core::ptr::addr_of_mut!(Error),
            ) as i32;

            WsTcEntryDereference(CatalogEntry);
            if ErrorCode != ERROR_SUCCESS as i32 {
                EnterCriticalSection(&mut (*Catalog).Lock);
                if UniqueId == (*Catalog).UniqueId {
                    continue;
                }

                continue 'CatalogChanged;
            }

            NewHandle = WPUModifyIFSHandle(
                ((ProtocolInfo.dwCatalogEntryId as u64) as u32),
                Handle,
                core::ptr::addr_of_mut!(Error),
            );

            if NewHandle == INVALID_SOCKET {
                return WSAENOTSOCK as u32;
            }

            return ERROR_SUCCESS as u32;
        }

        LeaveCriticalSection(&mut (*Catalog).Lock);
        return WSAENOTSOCK as u32;
    }
}
#[no_mangle]
pub unsafe extern "system" fn WsTcRemoveCatalogItem(Catalog: PTCATALOG, Entry: PTCATALOG_ENTRY) {
    RemoveEntryList(core::ptr::addr_of_mut!((*Entry).CatalogLink));
    (*Catalog).ItemCount = (*Catalog).ItemCount.wrapping_sub(1);
}

#[no_mangle]
pub unsafe extern "system" fn WsTcDelete(Catalog: PTCATALOG) {
    let mut Entry: PLIST_ENTRY;
    let mut CatalogEntry: PTCATALOG_ENTRY;

    if (*Catalog).ProtocolList.Flink.is_null() {
        return;
    }

    EnterCriticalSection((core::ptr::addr_of_mut!((*Catalog).Lock) as *mut core::ffi::c_void as *mut _RTL_CRITICAL_SECTION));

    Entry = (*Catalog).ProtocolList.Flink;
    while Entry != core::ptr::addr_of_mut!((*Catalog).ProtocolList) {
        CatalogEntry = (Entry as usize - core::mem::offset_of!(TCATALOG_ENTRY, CatalogLink)) as PTCATALOG_ENTRY;

        WsTcRemoveCatalogItem(Catalog, CatalogEntry);
        WsTcEntryDereference(CatalogEntry);

        Entry = (*Catalog).ProtocolList.Flink;
    }

    if !(*Catalog).CatalogKey.is_null() {
        RegCloseKey((*Catalog).CatalogKey);
        (*Catalog).CatalogKey = core::ptr::null_mut();
    }

    LeaveCriticalSection((core::ptr::addr_of_mut!((*Catalog).Lock) as *mut core::ffi::c_void as *mut _RTL_CRITICAL_SECTION));
    DeleteCriticalSection((core::ptr::addr_of_mut!((*Catalog).Lock) as *mut core::ffi::c_void as *mut _RTL_CRITICAL_SECTION));

    HeapFree(WsSockHeap, 0, Catalog as *mut core::ffi::c_void);
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:   See COPYING in the top level directory
//  * PROJECT:     ReactOS WinSock 2 API
//  * FILE:        dll/win32/ws2_32/src/dcatalog.c
//  * PURPOSE:     Transport Catalog Object
//  * PROGRAMMER:  Alex Ionescu (alex@relsoft.net)
//  */
//
// /* INCLUDES ******************************************************************/
//
// #include <ws2_32.h>
//
// #define NDEBUG
// #include <debug.h>
//
// /* DATA **********************************************************************/
//
// #define WsTcLock()          EnterCriticalSection(&Catalog->Lock)
// #define WsTcUnlock()        LeaveCriticalSection(&Catalog->Lock)
//
// /* FUNCTIONS *****************************************************************/
//
// PTCATALOG
// __stdcall
// WsTcAllocate(VOID)
// {
//     PTCATALOG Catalog;
//
//     /* Allocate the object */
//     Catalog = HeapAlloc(WsSockHeap, HEAP_ZERO_MEMORY, sizeof(*Catalog));
//
//     /* Return it */
//     return Catalog;
// }
//
// BOOL
// __stdcall
// WsTcOpen(IN PTCATALOG Catalog,
//          IN HKEY ParentKey)
// {
//     INT ErrorCode;
//     DWORD CreateDisposition;
//     HKEY CatalogKey, NewKey;
//     //DWORD CatalogEntries = 0;
//     DWORD RegType = REG_DWORD;
//     DWORD RegSize = sizeof(DWORD);
//     DWORD UniqueId = 0;
//     DWORD NewData = 0;
//     CHAR* CatalogKeyName;
//
//     /* Initialize the catalog lock and namespace list */
//     InitializeCriticalSection(&Catalog->Lock);
//     InitializeListHead(&Catalog->ProtocolList);
//
//     /* Read the catalog name */
//     ErrorCode = RegQueryValueEx(ParentKey,
//                                 "Current_Protocol_Catalog",
//                                 0,
//                                 &RegType,
//                                 NULL,
//                                 &RegSize);
//     if (ErrorCode == ERROR_FILE_NOT_FOUND)
//     {
//         static const CHAR DefaultCatalogName[] = "Protocol_Catalog9";
//         RegSize = sizeof(DefaultCatalogName);
//         CatalogKeyName = HeapAlloc(WsSockHeap, 0, RegSize);
//         memcpy(CatalogKeyName, DefaultCatalogName, RegSize);
//     }
//     else
//     {
//         if (ErrorCode != ERROR_SUCCESS)
//         {
//             DPRINT1("Failed to get protocol catalog name: %d.\n", ErrorCode);
//             return FALSE;
//         }
//
//         if (RegType != REG_SZ)
//         {
//             DPRINT1("Protocol catalog name is not a string (Type %d).\n", RegType);
//             return FALSE;
//         }
//
//         CatalogKeyName = HeapAlloc(WsSockHeap, 0, RegSize);
//
//         /* Read the catalog name */
//         ErrorCode = RegQueryValueEx(ParentKey,
//                                     "Current_Protocol_Catalog",
//                                     0,
//                                     &RegType,
//                                     (LPBYTE)CatalogKeyName,
//                                     &RegSize);
//
//         /* Open the Catalog Key */
//         ErrorCode = RegOpenKeyEx(ParentKey,
//                                  CatalogKeyName,
//                                  0,
//                                  MAXIMUM_ALLOWED,
//                                  &CatalogKey);
//     }
//
//     /* If we didn't find the key, create it */
//     if (ErrorCode == ERROR_SUCCESS)
//     {
//         /* Fake that we opened an existing key */
//         CreateDisposition = REG_OPENED_EXISTING_KEY;
//     }
//     else if (ErrorCode == ERROR_FILE_NOT_FOUND)
//     {
//         /* Create the Catalog Name */
//         ErrorCode = RegCreateKeyEx(ParentKey,
//                                    CatalogKeyName,
//                                    0,
//                                    NULL,
//                                    REG_OPTION_NON_VOLATILE,
//                                    KEY_ALL_ACCESS,
//                                    NULL,
//                                    &CatalogKey,
//                                    &CreateDisposition);
//     }
//
//     HeapFree(WsSockHeap, 0, CatalogKeyName);
//     RegType = REG_DWORD;
//     RegSize = sizeof(DWORD);
//
//     /* Fail if that didn't work */
//     if (ErrorCode != ERROR_SUCCESS) return FALSE;
//
//     /* Check if we had to create the key */
//     if (CreateDisposition == REG_CREATED_NEW_KEY)
//     {
//         /* Write the count of entries (0 now) */
//         ErrorCode = RegSetValueEx(CatalogKey,
//                                   "Num_Catalog_Entries",
//                                   0,
//                                   REG_DWORD,
//                                   (LPBYTE)&NewData,
//                                   sizeof(NewData));
//         if (ErrorCode != ERROR_SUCCESS)
//         {
//             /* Close the key and fail */
//             RegCloseKey(CatalogKey);
//             return FALSE;
//         }
//
//         /* Write the first catalog entry ID */
//         NewData = 1001;
//         ErrorCode = RegSetValueEx(CatalogKey,
//                                   "Next_Catalog_Entry_ID",
//                                   0,
//                                   REG_DWORD,
//                                   (LPBYTE)&NewData,
//                                   sizeof(NewData));
//         if (ErrorCode != ERROR_SUCCESS)
//         {
//             /* Close the key and fail */
//             RegCloseKey(CatalogKey);
//             return FALSE;
//         }
//
//         /* Write the first catalog entry Uniqe ID */
//         NewData = 1;
//         ErrorCode = RegSetValueEx(CatalogKey,
//                                   "Serial_Access_Num",
//                                   0,
//                                   REG_DWORD,
//                                   (LPBYTE)&NewData,
//                                   sizeof(NewData));
//         if (ErrorCode != ERROR_SUCCESS)
//         {
//             /* Close the key and fail */
//             RegCloseKey(CatalogKey);
//             return FALSE;
//         }
//
//         /* Create a key for this entry */
//         ErrorCode = RegCreateKeyEx(CatalogKey,
//                                    "Catalog_Entries",
//                                    0,
//                                    NULL,
//                                    REG_OPTION_NON_VOLATILE,
//                                    KEY_ALL_ACCESS,
//                                    NULL,
//                                    &NewKey,
//                                    &CreateDisposition);
//         if (ErrorCode != ERROR_SUCCESS)
//         {
//             /* Close the key and fail */
//             RegCloseKey(CatalogKey);
//             return FALSE;
//         }
//
//         /* Close the key since we don't need it */
//         RegCloseKey(NewKey);
//     }
//     else
//     {
//         RegSize = sizeof(DWORD);
//         /* Read the serial number */
//         ErrorCode = RegQueryValueEx(CatalogKey,
//                                     "Serial_Access_Num",
//                                     0,
//                                     &RegType,
//                                     (LPBYTE)&UniqueId,
//                                     &RegSize);
//
//         /* Check if it's missing for some reason */
//         if (ErrorCode != ERROR_SUCCESS)
//         {
//             /* Write the first catalog entry Unique ID */
//             NewData = 1;
//             ErrorCode = RegSetValueEx(CatalogKey,
//                                       "Serial_Access_Num",
//                                       0,
//                                       REG_DWORD,
//                                       (LPBYTE)&NewData,
//                                       sizeof(NewData));
//         }
//     }
//
//     /* Set the Catalog Key */
//     Catalog->CatalogKey = CatalogKey;
//     return TRUE;
// }
//
// DWORD
// __stdcall
// WsTcInitializeFromRegistry(IN PTCATALOG Catalog,
//                            IN HKEY ParentKey,
//                            IN HANDLE CatalogEvent)
// {
//     INT ErrorCode = WSASYSCALLFAILURE;
//
//     /* Open the catalog */
//     if (WsTcOpen(Catalog, ParentKey))
//     {
//         /* Refresh it */
//         ErrorCode = WsTcRefreshFromRegistry(Catalog, CatalogEvent);
//     }
//
//     /* Return the status */
//     return ErrorCode;
// }
//
// DWORD
// __stdcall
// WsTcRefreshFromRegistry(IN PTCATALOG Catalog,
//                         IN HANDLE CatalogEvent)
// {
//     INT ErrorCode;
//     BOOLEAN LocalEvent = FALSE;
//     LIST_ENTRY LocalList;
//     DWORD UniqueId;
//     HKEY EntriesKey;
//     DWORD CatalogEntries;
//     PTCATALOG_ENTRY CatalogEntry;
//     DWORD NextCatalogEntry;
//     BOOL NewChangesMade;
//     PLIST_ENTRY Entry;
//     DWORD RegType = REG_DWORD;
//     DWORD RegSize = sizeof(DWORD);
//     DWORD i;
//
//     /* Check if we got an event */
//     if (!CatalogEvent)
//     {
//         /* Create an event ourselves */
//         CatalogEvent = CreateEvent(NULL, FALSE, FALSE, NULL);
//         if (!CatalogEvent) return WSASYSCALLFAILURE;
//         LocalEvent = TRUE;
//     }
//
//     /* Lock the catalog */
//     WsTcLock();
//
//     /* Initialize our local list for the loop */
//     InitializeListHead(&LocalList);
//
//     /* Start looping */
//     do
//     {
//         /* Setup notifications for the catalog entry */
//         ErrorCode = WsSetupCatalogProtection(Catalog->CatalogKey,
//                                              CatalogEvent,
//                                              &UniqueId);
//         if (ErrorCode != ERROR_SUCCESS) break;
//
//         /* Check if we've changed till now */
//         if (UniqueId == Catalog->UniqueId)
//         {
//             /* We haven't, so return */
//             ErrorCode = ERROR_SUCCESS;
//             break;
//         }
//
//         /* Now Open the Entries */
//         ErrorCode = RegOpenKeyEx(Catalog->CatalogKey,
//                                  "Catalog_Entries",
//                                  0,
//                                  MAXIMUM_ALLOWED,
//                                  &EntriesKey);
//         if (ErrorCode != ERROR_SUCCESS)
//         {
//             /* Critical failure */
//             ErrorCode = WSASYSCALLFAILURE;
//             break;
//         }
//
//         /* Get the next entry */
//         ErrorCode = RegQueryValueEx(Catalog->CatalogKey,
//                                     "Next_Catalog_Entry_ID",
//                                     0,
//                                     &RegType,
//                                     (LPBYTE)&NextCatalogEntry,
//                                     &RegSize);
//         if (ErrorCode != ERROR_SUCCESS)
//         {
//             /* Critical failure */
//             ErrorCode = WSASYSCALLFAILURE;
//             break;
//         }
//
//         /* Find out how many there are */
//         ErrorCode = RegQueryValueEx(Catalog->CatalogKey,
//                                     "Num_Catalog_Entries",
//                                     0,
//                                     &RegType,
//                                     (LPBYTE)&CatalogEntries,
//                                     &RegSize);
//         if (ErrorCode != ERROR_SUCCESS)
//         {
//             /* Critical failure */
//             ErrorCode = WSASYSCALLFAILURE;
//             break;
//         }
//
//         /* Initialize them all */
//         for (i = 1; i <= CatalogEntries; i++)
//         {
//             /* Allocate a Catalog Entry Structure */
//             CatalogEntry = WsTcEntryAllocate();
//             if (!CatalogEntry)
//             {
//                 /* Not enough memory, fail */
//                 ErrorCode = WSA_NOT_ENOUGH_MEMORY;
//                 break;
//             }
//
//             /* Initialize it from the Registry Key */
//             ErrorCode = WsTcEntryInitializeFromRegistry(CatalogEntry,
//                                                         EntriesKey,
//                                                         i);
//             if (ErrorCode != ERROR_SUCCESS)
//             {
//                 /* We failed to get it, dereference the entry and leave */
//                 WsTcEntryDereference(CatalogEntry);
//                 break;
//             }
//
//             /* Insert it to our List */
//             InsertTailList(&LocalList, &CatalogEntry->CatalogLink);
//         }
//
//         /* Close the catalog key */
//         RegCloseKey(EntriesKey);
//
//         /* Check if we changed during our read and if we have success */
//         NewChangesMade = WsCheckCatalogState(CatalogEvent);
//         if (!NewChangesMade && ErrorCode == ERROR_SUCCESS)
//         {
//             /* All is good, update the protocol list */
//             WsTcUpdateProtocolList(Catalog, &LocalList);
//
//             /* Update and return */
//             Catalog->UniqueId = UniqueId;
//             Catalog->NextId = NextCatalogEntry;
//             break;
//         }
//
//         /* We failed and/or catalog data changed, free what we did till now */
//         while (!IsListEmpty(&LocalList))
//         {
//             /* Get the LP Catalog Item */
//             Entry = RemoveHeadList(&LocalList);
//             CatalogEntry = CONTAINING_RECORD(Entry, TCATALOG_ENTRY, CatalogLink);
//
//             /* Dereference it */
//             WsTcEntryDereference(CatalogEntry);
//         }
//     } while (NewChangesMade);
//
//     /* Release the lock */
//     WsTcUnlock();
//
//     /* Close the event, if any was created by us */
//     if (LocalEvent) CloseHandle(CatalogEvent);
//
//     /* All Done */
//     return ErrorCode;
// }
//
// DWORD
// __stdcall
// WsTcGetEntryFromAf(IN PTCATALOG Catalog,
//                    IN INT AddressFamily,
//                    IN PTCATALOG_ENTRY *CatalogEntry)
// {
//     INT ErrorCode = WSAEINVAL;
//     PLIST_ENTRY NextEntry;
//     PTCATALOG_ENTRY Entry;
//
//     /* Assume failure */
//     *CatalogEntry = NULL;
//
//     /* Lock the catalog */
//     WsTcLock();
//
//     /* Match the Id with all the entries in the List */
//     NextEntry = Catalog->ProtocolList.Flink;
//     while (NextEntry != &Catalog->ProtocolList)
//     {
//         /* Get the Current Entry */
//         Entry = CONTAINING_RECORD(NextEntry, TCATALOG_ENTRY, CatalogLink);
//         NextEntry = NextEntry->Flink;
//
//         /* Check if this is the Catalog Entry ID we want */
//         if (Entry->ProtocolInfo.iAddressFamily == AddressFamily)
//         {
//             /* Check if it doesn't already have a provider */
//             if (!Entry->Provider)
//             {
//                 /* Match, load the Provider */
//                 ErrorCode = WsTcLoadProvider(Catalog, Entry);
//
//                 /* Make sure this didn't fail */
//                 if (ErrorCode != ERROR_SUCCESS) break;
//             }
//
//             /* Reference the entry and return it */
//             InterlockedIncrement(&Entry->RefCount);
//             *CatalogEntry = Entry;
//             ErrorCode = ERROR_SUCCESS;
//             break;
//         }
//     }
//
//     /* Release the catalog */
//     WsTcUnlock();
//
//     /* Return */
//     return ErrorCode;
// }
//
// DWORD
// __stdcall
// WsTcGetEntryFromCatalogEntryId(IN PTCATALOG Catalog,
//                                IN DWORD CatalogEntryId,
//                                IN PTCATALOG_ENTRY *CatalogEntry)
// {
//     INT ErrorCode = WSAEINVAL;
//     PLIST_ENTRY NextEntry;
//     PTCATALOG_ENTRY Entry;
//
//     /* Lock the catalog */
//     WsTcLock();
//
//     /* Match the Id with all the entries in the List */
//     NextEntry = Catalog->ProtocolList.Flink;
//     while (NextEntry != &Catalog->ProtocolList)
//     {
//         /* Get the Current Entry */
//         Entry = CONTAINING_RECORD(NextEntry, TCATALOG_ENTRY, CatalogLink);
//         NextEntry = NextEntry->Flink;
//
//         /* Check if this is the Catalog Entry ID we want */
//         if (Entry->ProtocolInfo.dwCatalogEntryId == CatalogEntryId)
//         {
//             /* Check if it doesn't already have a provider */
//             if (!Entry->Provider)
//             {
//                 /* Match, load the Provider */
//                 WsTcLoadProvider(Catalog, Entry);
//             }
//
//             /* Reference the entry and return it */
//             InterlockedIncrement(&Entry->RefCount);
//             *CatalogEntry = Entry;
//             ErrorCode = ERROR_SUCCESS;
//             break;
//         }
//     }
//
//     /* Release the catalog */
//     WsTcUnlock();
//
//     /* Return */
//     return ErrorCode;
// }
//
// DWORD
// __stdcall
// WsTcGetEntryFromTriplet(IN PTCATALOG Catalog,
//                         IN INT af,
//                         IN INT type,
//                         IN INT protocol,
//                         IN DWORD StartId,
//                         IN PTCATALOG_ENTRY *CatalogEntry)
// {
//     INT ErrorCode = WSAEINVAL;
//     PLIST_ENTRY NextEntry;
//     PTCATALOG_ENTRY Entry;
//     DPRINT("WsTcGetEntryFromTriplet: %lx, %lx, %lx, %lx\n", af, type, protocol, StartId);
//
//     /* Assume failure */
//     *CatalogEntry = NULL;
//
//     /* Lock the catalog */
//     WsTcLock();
//
//     NextEntry = Catalog->ProtocolList.Flink;
//
//     /* Check if we are starting past 0 */
//     if (StartId)
//     {
//         /* Loop the list */
//         while (NextEntry != &Catalog->ProtocolList)
//         {
//             /* Get the Current Entry */
//             Entry = CONTAINING_RECORD(NextEntry, TCATALOG_ENTRY, CatalogLink);
//             NextEntry = NextEntry->Flink;
//
//             /* Check if this is the ID where we are starting */
//             if (Entry->ProtocolInfo.dwCatalogEntryId == StartId) break;
//         }
//     }
//
//     /* Match the Id with all the entries in the List */
//     while (NextEntry != &Catalog->ProtocolList)
//     {
//         /* Get the Current Entry */
//         Entry = CONTAINING_RECORD(NextEntry, TCATALOG_ENTRY, CatalogLink);
//         NextEntry = NextEntry->Flink;
//
//         /* Check if Address Family Matches or if it's wildcard */
//         if ((Entry->ProtocolInfo.iAddressFamily == af) || (af == AF_UNSPEC))
//         {
//             /* Check if Socket Type Matches or if it's wildcard */
//             if ((Entry->ProtocolInfo.iSocketType == type) || (type == 0))
//             {
//                 /* Check if Protocol is In Range or if it's wildcard */
//                 if (((Entry->ProtocolInfo.iProtocol <= protocol) &&
//                     ((Entry->ProtocolInfo.iProtocol +
//                       Entry->ProtocolInfo.iProtocolMaxOffset) >= protocol)) ||
//                     (protocol == 0))
//                 {
//                     /* Check if it doesn't already have a provider */
//                     if (!Entry->Provider)
//                     {
//                         /* Match, load the Provider */
//                         ErrorCode = WsTcLoadProvider(Catalog, Entry);
//
//                         /* Make sure this didn't fail */
//                         if (ErrorCode != ERROR_SUCCESS) break;
//                     }
//
//                     /* Reference the entry and return it */
//                     InterlockedIncrement(&Entry->RefCount);
//                     *CatalogEntry = Entry;
//                     ErrorCode = ERROR_SUCCESS;
//                     break;
//                 }
//                 else
//                 {
//                     ErrorCode = WSAEPROTONOSUPPORT;
//                 }
//             }
//             else
//             {
//                 ErrorCode = WSAESOCKTNOSUPPORT;
//             }
//         }
//         else
//         {
//             ErrorCode = WSAEAFNOSUPPORT;
//         }
//     }
//
//     /* Release the catalog */
//     WsTcUnlock();
//
//     /* Return */
//     return ErrorCode;
// }
//
// PTPROVIDER
// __stdcall
// WsTcFindProvider(IN PTCATALOG Catalog,
//                  IN LPGUID ProviderId)
// {
//     PTPROVIDER Provider;
//     PLIST_ENTRY Entry;
//     PTCATALOG_ENTRY CatalogEntry;
//
//     /* Loop the provider list */
//     Entry = Catalog->ProtocolList.Flink;
//     while (Entry != &Catalog->ProtocolList)
//     {
//         /* Get the entry */
//         CatalogEntry = CONTAINING_RECORD(Entry, TCATALOG_ENTRY, CatalogLink);
//
//         /* Move to the next one, get the provider */
//         Entry = Entry->Flink;
//         Provider = CatalogEntry->Provider;
//
//         /* Check for a match */
//         if (Provider &&
//             IsEqualGUID(&CatalogEntry->ProtocolInfo.ProviderId, ProviderId))
//         {
//             /* Found a match */
//             return Provider;
//         }
//     }
//
//     /* No match here */
//     return NULL;
// }
//
// DWORD
// __stdcall
// WsTcLoadProvider(IN PTCATALOG Catalog,
//                  IN PTCATALOG_ENTRY CatalogEntry)
// {
//     INT ErrorCode = ERROR_SUCCESS;
//     PTPROVIDER Provider;
//     DPRINT("WsTcLoadProvider: %p, %p\n", Catalog, CatalogEntry);
//
//     /* Lock the catalog */
//     WsTcLock();
//
//     /* Check if we have a provider already */
//     if (!CatalogEntry->Provider)
//     {
//         /* Try to find another instance */
//         Provider = WsTcFindProvider(Catalog,
//                                     &CatalogEntry->ProtocolInfo.ProviderId);
//
//         /* Check if we found one now */
//         if (Provider)
//         {
//             /* Set this one as the provider */
//             WsTcEntrySetProvider(CatalogEntry, Provider);
//             ErrorCode = ERROR_SUCCESS;
//         }
//         else
//         {
//             /* Nothing found, Allocate a provider */
//             if ((Provider = WsTpAllocate()))
//             {
//                 /* Initialize it */
//                 ErrorCode = WsTpInitialize(Provider,
//                                            CatalogEntry->DllPath,
//                                            &CatalogEntry->ProtocolInfo);
//
//                 /* Ensure success */
//                 if (ErrorCode == ERROR_SUCCESS)
//                 {
//                     /* Set the provider */
//                     WsTcEntrySetProvider(CatalogEntry, Provider);
//                 }
//
//                 /* Dereference it */
//                 WsTpDereference(Provider);
//             }
//             else
//             {
//                 /* No memory */
//                 ErrorCode = WSA_NOT_ENOUGH_MEMORY;
//             }
//         }
//     }
//
//     /* Release the lock */
//     WsTcUnlock();
//     return ErrorCode;
// }
//
// VOID
// __stdcall
// WsTcUpdateProtocolList(IN PTCATALOG Catalog,
//                        IN PLIST_ENTRY List)
// {
//     LIST_ENTRY TempList;
//     PTCATALOG_ENTRY CatalogEntry, OldCatalogEntry;
//     PLIST_ENTRY Entry;
//
//     /* First move from our list to the old one */
//     InsertHeadList(&Catalog->ProtocolList, &TempList);
//     RemoveEntryList(&Catalog->ProtocolList);
//     InitializeListHead(&Catalog->ProtocolList);
//
//     /* Loop every item in the list */
//     while (!IsListEmpty(List))
//     {
//         /* Get the catalog entry */
//         Entry = RemoveHeadList(List);
//         CatalogEntry = CONTAINING_RECORD(Entry, TCATALOG_ENTRY, CatalogLink);
//
//         /* Check if this item is already in our list */
//         Entry = TempList.Flink;
//         while (Entry != &TempList)
//         {
//             /* Get the catalog entry */
//             OldCatalogEntry = CONTAINING_RECORD(Entry, TCATALOG_ENTRY, CatalogLink);
//             Entry = Entry->Flink;
//
//             /* Check if they match */
//             if (CatalogEntry->ProtocolInfo.dwCatalogEntryId ==
//                 OldCatalogEntry->ProtocolInfo.dwCatalogEntryId)
//             {
//                 /* We have a match, use the old item instead */
//                 WsTcEntryDereference(CatalogEntry);
//                 CatalogEntry = OldCatalogEntry;
//                 RemoveEntryList(&CatalogEntry->CatalogLink);
//
//                 /* Decrease the number of protocols we have */
//                 Catalog->ItemCount--;
//                 break;
//             }
//         }
//
//         /* Add this item */
//         InsertTailList(&Catalog->ProtocolList, &CatalogEntry->CatalogLink);
//         Catalog->ItemCount++;
//     }
//
//     /* If there's anything left in the temporary list */
//     while (!IsListEmpty(&TempList))
//     {
//         /* Get the entry */
//         Entry = RemoveHeadList(&TempList);
//         CatalogEntry = CONTAINING_RECORD(Entry, TCATALOG_ENTRY, CatalogLink);
//
//         /* Remove it */
//         Catalog->ItemCount--;
//         WsTcEntryDereference(CatalogEntry);
//     }
// }
//
// VOID
// __stdcall
// WsTcEnumerateCatalogItems(IN PTCATALOG Catalog,
//                           IN PTCATALOG_ENUMERATE_PROC Callback,
//                           IN PVOID Context)
// {
//     PLIST_ENTRY Entry;
//     PTCATALOG_ENTRY CatalogEntry;
//     BOOL GoOn = TRUE;
//
//     /* Lock the catalog */
//     WsTcLock();
//
//     /* Loop the entries */
//     Entry = Catalog->ProtocolList.Flink;
//     while (GoOn && (Entry != &Catalog->ProtocolList))
//     {
//         /* Get the entry */
//         CatalogEntry = CONTAINING_RECORD(Entry, TCATALOG_ENTRY, CatalogLink);
//
//         /* Move to the next one and call the callback */
//         Entry = Entry->Flink;
//         GoOn = Callback(Context, CatalogEntry);
//     }
//
//     /* Release lock */
//     WsTcUnlock();
// }
//
// DWORD
// __stdcall
// WsTcFindIfsProviderForSocket(IN PTCATALOG Catalog,
//                              IN SOCKET Handle)
// {
//     PTPROVIDER Provider;
//     IN SOCKET NewHandle;
//     INT Error;
//     INT OptionLength;
//     PLIST_ENTRY Entry;
//     WSAPROTOCOL_INFOW ProtocolInfo;
//     DWORD UniqueId;
//     INT ErrorCode;
//     PTCATALOG_ENTRY CatalogEntry;
//
//     /* Get the catalog lock */
//     WsTcLock();
//
//     /* Loop as long as the catalog changes */
// CatalogChanged:
//
//     /* Loop every provider */
//     Entry = Catalog->ProtocolList.Flink;
//     while (Entry != &Catalog->ProtocolList)
//     {
//         /* Get the catalog entry */
//         CatalogEntry = CONTAINING_RECORD(Entry, TCATALOG_ENTRY, CatalogLink);
//
//         /* Move to the next entry */
//         Entry = Entry->Flink;
//
//         /* Skip it if it doesn't support IFS */
//         if (!(CatalogEntry->ProtocolInfo.dwServiceFlags1 & XP1_IFS_HANDLES)) continue;
//
//         /* Check if we need to load it */
//         if (!(Provider = CatalogEntry->Provider))
//         {
//             /* Load it */
//             ErrorCode = WsTcLoadProvider(Catalog, CatalogEntry);
//
//             /* Skip it if we failed to load it */
//             if (ErrorCode != ERROR_SUCCESS) continue;
//
//             /* Get the provider again */
//             Provider = CatalogEntry->Provider;
//         }
//
//         /* Reference the entry and get our unique id */
//         InterlockedIncrement(&CatalogEntry->RefCount);
//         UniqueId = Catalog->UniqueId;
//
//         /* Release the lock now */
//         WsTcUnlock();
//
//         /* Get the catalog entry ID */
//         OptionLength = sizeof(ProtocolInfo);
//         ErrorCode = Provider->Service.lpWSPGetSockOpt(Handle,
//                                                       SOL_SOCKET,
//                                                       SO_PROTOCOL_INFOW,
//                                                       (PCHAR)&ProtocolInfo,
//                                                       &OptionLength,
//                                                       &Error);
//
//         /* Dereference the entry and check the result */
//         WsTcEntryDereference(CatalogEntry);
//         if (ErrorCode != ERROR_SUCCESS)
//         {
//             /* Lock and make sure this provider is still valid */
//             WsTcLock();
//             if (UniqueId == Catalog->UniqueId) continue;
//
//             /* It changed! We need to start over */
//             goto CatalogChanged;
//         }
//
//         /* Now get the IFS handle */
//         NewHandle = WPUModifyIFSHandle(ProtocolInfo.dwCatalogEntryId,
//                                        Handle,
//                                        &Error);
//
//         /* Check if the socket is invalid */
//         if (NewHandle == INVALID_SOCKET) return WSAENOTSOCK;
//
//         /* We succeeded, get out of here */
//         return ERROR_SUCCESS;
//     }
//
//     /* Unrecognized socket if we get here: note, we still have the lock */
//     WsTcUnlock();
//     return WSAENOTSOCK;
// }
//
// VOID
// __stdcall
// WsTcRemoveCatalogItem(IN PTCATALOG Catalog,
//                       IN PTCATALOG_ENTRY Entry)
// {
//     /* Remove the entry from the list */
//     RemoveEntryList(&Entry->CatalogLink);
//
//     /* Decrease our count */
//     Catalog->ItemCount--;
// }
//
// VOID
// __stdcall
// WsTcDelete(IN PTCATALOG Catalog)
// {
//     PLIST_ENTRY Entry;
//     PTCATALOG_ENTRY CatalogEntry;
//
//     /* Check if we're initialized */
//     if (!Catalog->ProtocolList.Flink) return;
//
//     /* Acquire lock */
//     WsTcLock();
//
//     /* Loop every entry */
//     Entry = Catalog->ProtocolList.Flink;
//     while (Entry != &Catalog->ProtocolList)
//     {
//         /* Get this entry */
//         CatalogEntry = CONTAINING_RECORD(Entry, TCATALOG_ENTRY, CatalogLink);
//
//         /* Remove it and dereference it */
//         WsTcRemoveCatalogItem(Catalog, CatalogEntry);
//         WsTcEntryDereference(CatalogEntry);
//
//         /* Move to the next entry */
//         Entry = Catalog->ProtocolList.Flink;
//     }
//
//     /* Check if the catalog key is opened */
//     if (Catalog->CatalogKey)
//     {
//         /* Close it */
//         RegCloseKey(Catalog->CatalogKey);
//         Catalog->CatalogKey = NULL;
//     }
//
//     /* Release and delete the lock */
//     WsTcUnlock();
//     DeleteCriticalSection(&Catalog->Lock);
//
//     /* Delete the object */
//     HeapFree(WsSockHeap, 0, Catalog);
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
