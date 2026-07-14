use crate::prelude::*;
use crate::types::*;
use core::ffi::c_void;
use crate::src::nscatent::WsNcEntryDereference;


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

pub const WsNcLock: () = ();
pub const WsNcUnlock: () = ();

#[no_mangle]
pub unsafe extern "system" fn WsNcAllocate() -> PNSCATALOG {

    let Catalog: PNSCATALOG = HeapAlloc(
        WsSockHeap,
        HEAP_ZERO_MEMORY,
        (core::mem::size_of::<NSCATALOG>() as SIZE_T),
    ) as PNSCATALOG;

    Catalog
}

#[no_mangle]
pub unsafe extern "system" fn WsNcOpen(Catalog: PNSCATALOG, ParentKey: HKEY) -> u8 {
    let mut ErrorCode: i32;
    let mut CreateDisposition: u32 = 0;
    let mut CatalogKey: HKEY = core::ptr::null_mut();
    let mut NewKey: HKEY = core::ptr::null_mut();
    let mut RegType: u32 = REG_DWORD;
    let mut RegSize: u32 = core::mem::size_of::<u32>() as u32;
    let mut UniqueId: u32 = 0;
    let mut NewData: u32 = 0;
    let mut CatalogKeyName: *mut i8;

    InitializeCriticalSection((core::ptr::addr_of_mut!((*Catalog).Lock) as *mut c_void));
    InitializeListHead(core::ptr::addr_of_mut!((*Catalog).CatalogList));

    ErrorCode = RegQueryValueEx(
        ParentKey,
        b"Current_NameSpace_Catalog\0".as_ptr() as *const i8,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(RegType),
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(RegSize),
    );

    if ErrorCode == ERROR_FILE_NOT_FOUND as i32 {
        static DefaultCatalogName: [i8; 19] = [
            78, 97, 109, 101, 83, 112, 97, 99, 101, 95, 67, 97, 116, 97, 108, 111, 103, 53, 0
        ];
        RegSize = core::mem::size_of::<[i8; 19]>() as u32;
        CatalogKeyName = HeapAlloc(WsSockHeap, 0, ((RegSize as usize as u64)) as SIZE_T) as *mut i8;
        core::ptr::copy_nonoverlapping(
            DefaultCatalogName.as_ptr(),
            CatalogKeyName,
            RegSize as usize,
        );
    } else {
        if ErrorCode != ERROR_SUCCESS as i32 {
            return 0u8;
        }

        if RegType != REG_SZ {
            return 0u8;
        }

        CatalogKeyName = HeapAlloc(WsSockHeap, 0, ((RegSize as usize as u64)) as SIZE_T) as *mut i8;

        ErrorCode = RegQueryValueEx(
            ParentKey,
            b"Current_NameSpace_Catalog\0".as_ptr() as *const i8,
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!(RegType),
            CatalogKeyName as LPBYTE,
            core::ptr::addr_of_mut!(RegSize),
        );

        ErrorCode = RegOpenKeyEx(
            ParentKey,
            CatalogKeyName as *const i8,
            0,
            MAXIMUM_ALLOWED,
            core::ptr::addr_of_mut!(CatalogKey),
        );
    }

    if ErrorCode == ERROR_SUCCESS as i32 {
        CreateDisposition = REG_OPENED_EXISTING_KEY;
    } else if ErrorCode == ERROR_FILE_NOT_FOUND as i32 {
        ErrorCode = RegCreateKeyEx(
            ParentKey,
            CatalogKeyName as *const i8,
            0,
            core::ptr::null_mut(),
            REG_OPTION_NON_VOLATILE,
            KEY_ALL_ACCESS,
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!(CatalogKey),
            core::ptr::addr_of_mut!(CreateDisposition),
        );
    }

    HeapFree(WsSockHeap, 0, CatalogKeyName as *mut core::ffi::c_void);
    RegType = REG_DWORD;
    RegSize = core::mem::size_of::<u32>() as u32;

    if ErrorCode != ERROR_SUCCESS as i32 {
        return 0u8;
    }

    if CreateDisposition == REG_CREATED_NEW_KEY {
        ErrorCode = RegSetValueEx(
            CatalogKey,
            b"Num_Catalog_Entries\0".as_ptr() as *const i8,
            0,
            REG_DWORD,
            core::ptr::addr_of!(NewData) as *const u8,
            core::mem::size_of::<u32>() as u32,
        );
        if ErrorCode != ERROR_SUCCESS as i32 {
            RegCloseKey(CatalogKey);
            return 0u8;
        }

        NewData = 1;
        ErrorCode = RegSetValueEx(
            CatalogKey,
            b"Serial_Access_Num\0".as_ptr() as *const i8,
            0,
            REG_DWORD,
            core::ptr::addr_of!(NewData) as *const u8,
            core::mem::size_of::<u32>() as u32,
        );
        if ErrorCode != ERROR_SUCCESS as i32 {
            RegCloseKey(CatalogKey);
            return 0u8;
        }

        ErrorCode = RegCreateKeyEx(
            CatalogKey,
            b"Catalog_Entries\0".as_ptr() as *const i8,
            0,
            core::ptr::null_mut(),
            REG_OPTION_NON_VOLATILE,
            KEY_ALL_ACCESS,
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!(NewKey),
            core::ptr::addr_of_mut!(CreateDisposition),
        );
        if ErrorCode != ERROR_SUCCESS as i32 {
            RegCloseKey(CatalogKey);
            return 0u8;
        }

        RegCloseKey(NewKey);
    } else {
        RegSize = core::mem::size_of::<u32>() as u32;
        ErrorCode = RegQueryValueEx(
            CatalogKey,
            b"Serial_Access_Num\0".as_ptr() as *const i8,
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!(RegType),
            core::ptr::addr_of_mut!(UniqueId) as LPBYTE,
            core::ptr::addr_of_mut!(RegSize),
        );

        if ErrorCode != ERROR_SUCCESS as i32 {
            NewData = 1;
            ErrorCode = RegSetValueEx(
                CatalogKey,
                b"Serial_Access_Num\0".as_ptr() as *const i8,
                0,
                REG_DWORD,
                core::ptr::addr_of!(NewData) as *const u8,
                core::mem::size_of::<u32>() as u32,
            );
        }
    }

    (*Catalog).CatalogKey = CatalogKey;
    1u8
}

#[no_mangle]
pub unsafe extern "system" fn WsNcInitializeFromRegistry(
    Catalog: PNSCATALOG,
    ParentKey: HKEY,
    CatalogEvent: HANDLE,
) -> INT {
    let mut ErrorCode: INT = (WSASYSCALLFAILURE as i32);

    if WsNcOpen(Catalog, ParentKey) != 0 {
        ErrorCode = WsNcRefreshFromRegistry(Catalog, CatalogEvent);
    }

    return ErrorCode;
}
#[no_mangle]
pub unsafe extern "system" fn WsNcRefreshFromRegistry(
    Catalog: PNSCATALOG,
    mut CatalogEvent: HANDLE,
) -> INT {
    let mut ErrorCode: INT;
    let mut LocalEvent: BOOLEAN = 0u8;
    let mut LocalList: LIST_ENTRY = LIST_ENTRY {
        Flink: core::ptr::null_mut(),
        Blink: core::ptr::null_mut(),
    };
    let mut UniqueId: DWORD = 0;
    let mut EntriesKey: HKEY = core::ptr::null_mut();
    let mut CatalogEntries: DWORD = 0;
    let mut CatalogEntry: PNSCATALOG_ENTRY;
    let mut NewChangesMade: BOOL;
    let mut Entry: PLIST_ENTRY;
    let mut RegType: DWORD = REG_DWORD;
    let mut RegSize: DWORD = core::mem::size_of::<DWORD>() as DWORD;
    let mut i: DWORD;

    if CatalogEvent.is_null() {
        CatalogEvent = CreateEvent(core::ptr::null_mut(), 0, 0, core::ptr::null_mut());
        if CatalogEvent.is_null() {
            return (WSASYSCALLFAILURE as i32);
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
        if ErrorCode != (ERROR_SUCCESS as i32) {
            break;
        }

        if UniqueId == (*Catalog).UniqueId {
            ErrorCode = ERROR_SUCCESS as INT;
            break;
        }

        ErrorCode = RegOpenKeyEx(
            (*Catalog).CatalogKey,
            b"Catalog_Entries\0".as_ptr() as LPCSTR,
            0,
            MAXIMUM_ALLOWED,
            core::ptr::addr_of_mut!(EntriesKey),
        ) as INT;
        if ErrorCode != (ERROR_SUCCESS as i32) {
            ErrorCode = (WSASYSCALLFAILURE as i32);
            break;
        }

        ErrorCode = RegQueryValueEx(
            (*Catalog).CatalogKey,
            b"Num_Catalog_Entries\0".as_ptr() as LPCSTR,
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!(RegType),
            core::ptr::addr_of_mut!(CatalogEntries) as LPBYTE,
            core::ptr::addr_of_mut!(RegSize),
        ) as INT;
        if ErrorCode != (ERROR_SUCCESS as i32) {
            ErrorCode = (WSASYSCALLFAILURE as i32);
            break;
        }

        i = 1;
        while i <= CatalogEntries {
            CatalogEntry = WsNcEntryAllocate();
            if CatalogEntry.is_null() {
                ErrorCode = (WSA_NOT_ENOUGH_MEMORY as i32);
                break;
            }

            ErrorCode = WsNcEntryInitializeFromRegistry(CatalogEntry, EntriesKey, i as ULONG);
            if ErrorCode != (ERROR_SUCCESS as i32) {
                WsNcEntryDereference(CatalogEntry);
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
        if NewChangesMade == 0 && ErrorCode == (ERROR_SUCCESS as i32) {
            WsNcUpdateNamespaceList(Catalog, core::ptr::addr_of_mut!(LocalList));
            (*Catalog).UniqueId = UniqueId;
            break;
        }

        while !IsListEmpty(core::ptr::addr_of_mut!(LocalList)) {
            Entry = RemoveHeadList(core::ptr::addr_of_mut!(LocalList));
            CatalogEntry = ((Entry as ULONG_PTR).wrapping_sub(
                core::ptr::addr_of_mut!(
                    (*core::ptr::null_mut::<NSCATALOG_ENTRY>()).CatalogLink
                ) as ULONG_PTR,
            )) as PNSCATALOG_ENTRY;
            WsNcEntryDereference(CatalogEntry);
        }

        if NewChangesMade == 0 {
            break;
        }
    }

    LeaveCriticalSection(&mut (*Catalog).Lock);

    if LocalEvent != 0 {
        CloseHandle(CatalogEvent);
    }

    ErrorCode
}

#[no_mangle]
pub unsafe extern "system" fn WsNcEnumerateCatalogItems(
    Catalog: PNSCATALOG,
    Callback: PNSCATALOG_ENUMERATE_PROC,
    Context: PVOID,
) {
    let mut Entry: PLIST_ENTRY;
    let mut CatalogEntry: PNSCATALOG_ENTRY;
    let mut GoOn: BOOL = (TRUE as i32);

    EnterCriticalSection(&mut (*Catalog).Lock);

    Entry = (*Catalog).CatalogList.Flink;
    while GoOn != 0 && Entry != core::ptr::addr_of_mut!((*Catalog).CatalogList) {
        CatalogEntry = (Entry as usize -
            core::mem::offset_of!(NSCATALOG_ENTRY, CatalogLink)) as PNSCATALOG_ENTRY;

        Entry = (*Entry).Flink;
        GoOn = Callback.unwrap_unchecked()(Context, CatalogEntry);
    }

    LeaveCriticalSection(&mut (*Catalog).Lock);
}
#[no_mangle]
pub unsafe extern "system" fn WsNcLoadProvider(
    Catalog: PNSCATALOG,
    CatalogEntry: PNSCATALOG_ENTRY,
) -> INT {
    let mut ErrorCode: INT = (ERROR_SUCCESS as i32);
    let mut Provider: PNS_PROVIDER;

    EnterCriticalSection(&mut (*Catalog).Lock);

    if (*CatalogEntry).Provider.is_null() {
        Provider = WsNpAllocate();
        if !Provider.is_null() {
            ErrorCode = (WsNpInitialize(
                Provider,
                (*CatalogEntry).DllPath.as_mut_ptr(),
                core::ptr::addr_of_mut!((*CatalogEntry).ProviderId),
            ) as i32);

            if ErrorCode == (ERROR_SUCCESS as i32) {
                WsNcEntrySetProvider(CatalogEntry, Provider);
            }

            WsNpDereference(Provider);
        } else {
            ErrorCode = (WSA_NOT_ENOUGH_MEMORY as i32);
        }
    }

    LeaveCriticalSection(&mut (*Catalog).Lock);
    return ErrorCode;
}

#[no_mangle]
pub unsafe extern "system" fn WsNcGetServiceClassInfo(
    Catalog: PNSCATALOG,
    BugSize: LPDWORD,
    lpServiceClassInfo: LPWSASERVICECLASSINFOW,
) -> INT {
    SetLastError(ERROR_SUCCESS);
    return SOCKET_ERROR;
}

#[no_mangle]
pub unsafe extern "system" fn WsNcUpdateNamespaceList(
    Catalog: PNSCATALOG,
    List: PLIST_ENTRY,
) {
    let mut TempList: LIST_ENTRY = LIST_ENTRY {
        Flink: core::ptr::null_mut(),
        Blink: core::ptr::null_mut(),
    };
    let mut CatalogEntry: PNSCATALOG_ENTRY;
    let mut OldCatalogEntry: PNSCATALOG_ENTRY;
    let mut Entry: PLIST_ENTRY;

    InsertHeadList(
        core::ptr::addr_of_mut!((*Catalog).CatalogList),
        core::ptr::addr_of_mut!(TempList),
    );
    RemoveEntryList(core::ptr::addr_of_mut!((*Catalog).CatalogList));
    InitializeListHead(core::ptr::addr_of_mut!((*Catalog).CatalogList));

    while !IsListEmpty(List) {
        Entry = RemoveHeadList(List);
        CatalogEntry = (Entry as usize)
            .wrapping_sub(core::ptr::addr_of_mut!((*core::ptr::null_mut::<NSCATALOG_ENTRY>()).CatalogLink) as usize)
            as PNSCATALOG_ENTRY;

        Entry = TempList.Flink;
        while Entry != core::ptr::addr_of_mut!(TempList) {
            OldCatalogEntry = (Entry as usize)
                .wrapping_sub(core::ptr::addr_of_mut!((*core::ptr::null_mut::<NSCATALOG_ENTRY>()).CatalogLink) as usize)
                as PNSCATALOG_ENTRY;
            Entry = (*Entry).Flink;

            if IsEqualGUID(
                core::ptr::addr_of_mut!((*CatalogEntry).ProviderId),
                core::ptr::addr_of_mut!((*OldCatalogEntry).ProviderId),
            )
            {
                WsNcEntryDereference(CatalogEntry);
                CatalogEntry = OldCatalogEntry;
                RemoveEntryList(core::ptr::addr_of_mut!((*CatalogEntry).CatalogLink));
                (*Catalog).ItemCount = (*Catalog).ItemCount.wrapping_sub(1);
                break;
            }
        }

        InsertTailList(
            core::ptr::addr_of_mut!((*Catalog).CatalogList),
            core::ptr::addr_of_mut!((*CatalogEntry).CatalogLink),
        );
        (*Catalog).ItemCount = (*Catalog).ItemCount.wrapping_add(1);
    }

    while !IsListEmpty(core::ptr::addr_of_mut!(TempList)) {
        Entry = RemoveHeadList(core::ptr::addr_of_mut!(TempList));
        CatalogEntry = (Entry as usize)
            .wrapping_sub(core::ptr::addr_of_mut!((*core::ptr::null_mut::<NSCATALOG_ENTRY>()).CatalogLink) as usize)
            as PNSCATALOG_ENTRY;

        (*Catalog).ItemCount = (*Catalog).ItemCount.wrapping_sub(1);
        WsNcEntryDereference(CatalogEntry);
    }
}
#[no_mangle]
pub unsafe extern "system" fn WsNcGetCatalogFromProviderId(
    Catalog: PNSCATALOG,
    ProviderId: LPGUID,
    CatalogEntry: *mut PNSCATALOG_ENTRY,
) -> INT {
    let mut ErrorCode: INT = (WSAEINVAL as i32);
    let mut NextEntry: PLIST_ENTRY;
    let mut Entry: PNSCATALOG_ENTRY;

    EnterCriticalSection(&mut (*Catalog).Lock);

    NextEntry = (*Catalog).CatalogList.Flink;
    while NextEntry != core::ptr::addr_of_mut!((*Catalog).CatalogList) {
        Entry = (NextEntry as *mut u8).offset(
            -(core::mem::offset_of!(NSCATALOG_ENTRY, CatalogLink) as isize)
        ) as PNSCATALOG_ENTRY;
        NextEntry = (*NextEntry).Flink;

        if IsEqualGUID(core::ptr::addr_of_mut!((*Entry).ProviderId), ProviderId) {
            if (*Entry).Provider.is_null() {
                ErrorCode = WsNcLoadProvider(Catalog, Entry);
            }

            if !(*Entry).Provider.is_null() {
                InterlockedIncrement(core::ptr::addr_of_mut!((*Entry).RefCount));
                *CatalogEntry = Entry;
                ErrorCode = (ERROR_SUCCESS as i32);
                break;
            }
        }
    }

    LeaveCriticalSection(&mut (*Catalog).Lock);
    ErrorCode
}

#[no_mangle]
pub unsafe extern "system" fn WsNcMatchProtocols(
    NameSpace: DWORD,
    AddressFamily: LONG,
    QuerySet: LPWSAQUERYSETW,
) -> BOOL {
    let mut ProtocolCount: DWORD = (*QuerySet).dwNumberOfProtocols;
    let mut AfpProtocols: LPAFPROTOCOLS = (*QuerySet).lpafpProtocols;
    let Family: LONG;

    if AddressFamily != -1 {
        if AddressFamily == AF_UNSPEC as LONG {
            return 1;
        }
        Family = AddressFamily;
    } else {
        if NameSpace == NS_SAP {
            Family = AF_IPX as LONG;
        } else {
            return 1;
        }
    }

    loop {
        let fresh = ProtocolCount;
        ProtocolCount = ProtocolCount.wrapping_sub(1);
        if fresh == 0 {
            break;
        }

        if (*AfpProtocols).iAddressFamily == (AF_UNSPEC as i32)
            || (*AfpProtocols).iAddressFamily as LONG == Family
        {
            return 1;
        }

        AfpProtocols = AfpProtocols.offset(1);
    }

    return 0;
}
#[no_mangle]
pub unsafe extern "system" fn WsNcRemoveCatalogItem(
    Catalog: PNSCATALOG,
    Entry: PNSCATALOG_ENTRY,
) {
    RemoveEntryList(core::ptr::addr_of_mut!((*Entry).CatalogLink));
    (*Catalog).ItemCount = (*Catalog).ItemCount.wrapping_sub(1);
}
#[no_mangle]
pub unsafe extern "system" fn WsNcDelete(Catalog: PNSCATALOG) {
    let mut Entry: PLIST_ENTRY;
    let mut CatalogEntry: PNSCATALOG_ENTRY;

    if (*Catalog).CatalogList.Flink.is_null() {
        return;
    }

    EnterCriticalSection((core::ptr::addr_of_mut!((*Catalog).Lock) as *mut core::ffi::c_void as *mut _RTL_CRITICAL_SECTION));

    Entry = (*Catalog).CatalogList.Flink;
    while Entry != core::ptr::addr_of_mut!((*Catalog).CatalogList) {
        CatalogEntry = (Entry as usize - core::mem::offset_of!(NSCATALOG_ENTRY, CatalogLink)) as PNSCATALOG_ENTRY;

        WsNcRemoveCatalogItem(Catalog, CatalogEntry);
        WsNcEntryDereference(CatalogEntry);

        Entry = (*Catalog).CatalogList.Flink;
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
//  * FILE:        dll/win32/ws2_32/src/nscatalo.c
//  * PURPOSE:     Namespace Catalog Object
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
// #define WsNcLock()          EnterCriticalSection(&Catalog->Lock)
// #define WsNcUnlock()        LeaveCriticalSection(&Catalog->Lock)
//
// /* FUNCTIONS *****************************************************************/
//
// PNSCATALOG
// __stdcall
// WsNcAllocate(VOID)
// {
//     PNSCATALOG Catalog;
//
//     /* Allocate the catalog */
//     Catalog = HeapAlloc(WsSockHeap, HEAP_ZERO_MEMORY, sizeof(*Catalog));
//
//     /* Return it */
//     return Catalog;
// }
//
// BOOLEAN
// __stdcall
// WsNcOpen(IN PNSCATALOG Catalog,
//          IN HKEY ParentKey)
// {
//     LONG ErrorCode;
//     DWORD CreateDisposition;
//     HKEY CatalogKey, NewKey;
//     DWORD RegType = REG_DWORD;
//     DWORD RegSize = sizeof(DWORD);
//     DWORD UniqueId = 0;
//     DWORD NewData = 0;
//     CHAR* CatalogKeyName;
//
//     /* Initialize the catalog lock and namespace list */
//     InitializeCriticalSection(&Catalog->Lock);
//     InitializeListHead(&Catalog->CatalogList);
//
//     /* Read the catalog name */
//     ErrorCode = RegQueryValueEx(ParentKey,
//                                 "Current_NameSpace_Catalog",
//                                 0,
//                                 &RegType,
//                                 NULL,
//                                 &RegSize);
//     if (ErrorCode == ERROR_FILE_NOT_FOUND)
//     {
//         static const CHAR DefaultCatalogName[] = "NameSpace_Catalog5";
//         RegSize = sizeof(DefaultCatalogName);
//         CatalogKeyName = HeapAlloc(WsSockHeap, 0, RegSize);
//         memcpy(CatalogKeyName, DefaultCatalogName, RegSize);
//     }
//     else
//     {
//         if (ErrorCode != ERROR_SUCCESS)
//         {
//             DPRINT1("Failed to get namespace catalog name: %d.\n", ErrorCode);
//             return FALSE;
//         }
//
//         if (RegType != REG_SZ)
//         {
//             DPRINT1("Namespace catalog name is not a string (Type %d).\n", RegType);
//             return FALSE;
//         }
//
//         CatalogKeyName = HeapAlloc(WsSockHeap, 0, RegSize);
//
//         /* Read the catalog name */
//         ErrorCode = RegQueryValueEx(ParentKey,
//                                     "Current_NameSpace_Catalog",
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
//         RegSize = sizeof(UniqueId);
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
// INT
// __stdcall
// WsNcInitializeFromRegistry(IN PNSCATALOG Catalog,
//                            IN HKEY ParentKey,
//                            IN HANDLE CatalogEvent)
// {
//     INT ErrorCode = WSASYSCALLFAILURE;
//
//     /* Open the catalog */
//     if (WsNcOpen(Catalog, ParentKey))
//     {
//         /* Refresh it */
//         ErrorCode = WsNcRefreshFromRegistry(Catalog, CatalogEvent);
//     }
//
//     /* Return the status */
//     return ErrorCode;
// }
//
// INT
// __stdcall
// WsNcRefreshFromRegistry(IN PNSCATALOG Catalog,
//                         IN HANDLE CatalogEvent)
// {
//     INT ErrorCode;
//     BOOLEAN LocalEvent = FALSE;
//     LIST_ENTRY LocalList;
//     DWORD UniqueId;
//     HKEY EntriesKey;
//     DWORD CatalogEntries;
//     PNSCATALOG_ENTRY CatalogEntry;
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
//     WsNcLock();
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
//             CatalogEntry = WsNcEntryAllocate();
//             if (!CatalogEntry)
//             {
//                 /* Not enough memory, fail */
//                 ErrorCode = WSA_NOT_ENOUGH_MEMORY;
//                 break;
//             }
//
//             /* Initialize it from the Registry Key */
//             ErrorCode = WsNcEntryInitializeFromRegistry(CatalogEntry,
//                                                         EntriesKey,
//                                                         i);
//             if (ErrorCode != ERROR_SUCCESS)
//             {
//                 /* We failed to get it, dereference the entry and leave */
//                 WsNcEntryDereference(CatalogEntry);
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
//             WsNcUpdateNamespaceList(Catalog, &LocalList);
//
//             /* Update and return */
//             Catalog->UniqueId = UniqueId;
//             break;
//         }
//
//         /* We failed and/or catalog data changed, free what we did till now */
//         while (!IsListEmpty(&LocalList))
//         {
//             /* Get the LP Catalog Item */
//             Entry = RemoveHeadList(&LocalList);
//             CatalogEntry = CONTAINING_RECORD(Entry, NSCATALOG_ENTRY, CatalogLink);
//
//             /* Dereference it */
//             WsNcEntryDereference(CatalogEntry);
//         }
//     } while (NewChangesMade);
//
//     /* Release the lock */
//     WsNcUnlock();
//
//     /* Close the event, if any was created by us */
//     if (LocalEvent) CloseHandle(CatalogEvent);
//
//     /* All Done */
//     return ErrorCode;
// }
//
// VOID
// __stdcall
// WsNcEnumerateCatalogItems(IN PNSCATALOG Catalog,
//                           IN PNSCATALOG_ENUMERATE_PROC Callback,
//                           IN PVOID Context)
// {
//     PLIST_ENTRY Entry;
//     PNSCATALOG_ENTRY CatalogEntry;
//     BOOL GoOn = TRUE;
//
//     /* Lock the catalog */
//     WsNcLock();
//
//     /* Loop the entries */
//     Entry = Catalog->CatalogList.Flink;
//     while (GoOn && (Entry != &Catalog->CatalogList))
//     {
//         /* Get the entry */
//         CatalogEntry = CONTAINING_RECORD(Entry, NSCATALOG_ENTRY, CatalogLink);
//
//         /* Move to the next one and call the callback */
//         Entry = Entry->Flink;
//         GoOn = Callback(Context, CatalogEntry);
//     }
//
//     /* Release the lock */
//     WsNcUnlock();
// }
//
// INT
// __stdcall
// WsNcLoadProvider(IN PNSCATALOG Catalog,
//                  IN PNSCATALOG_ENTRY CatalogEntry)
// {
//     INT ErrorCode = ERROR_SUCCESS;
//     PNS_PROVIDER Provider;
//
//     /* Lock the catalog */
//     WsNcLock();
//
//     /* Check if we have a provider already */
//     if (!CatalogEntry->Provider)
//     {
//         /* Allocate a provider */
//         if ((Provider = WsNpAllocate()))
//         {
//             /* Initialize it */
//             ErrorCode = WsNpInitialize(Provider,
//                                        CatalogEntry->DllPath,
//                                        &CatalogEntry->ProviderId);
//
//             /* Ensure success */
//             if (ErrorCode == ERROR_SUCCESS)
//             {
//                 /* Set the provider */
//                 WsNcEntrySetProvider(CatalogEntry, Provider);
//             }
//
//             /* Dereference it */
//             WsNpDereference(Provider);
//         }
//         else
//         {
//             /* No memory */
//             ErrorCode = WSA_NOT_ENOUGH_MEMORY;
//         }
//     }
//
//     /* Release the lock and return */
//     WsNcUnlock();
//     return ErrorCode;
// }
//
// INT
// __stdcall
// WsNcGetServiceClassInfo(IN PNSCATALOG Catalog,
//                         IN OUT LPDWORD BugSize,
//                         IN OUT LPWSASERVICECLASSINFOW lpServiceClassInfo)
// {
//     /* Not yet implemented in the spec? */
//     SetLastError(ERROR_SUCCESS);
//     return SOCKET_ERROR;
// }
//
// VOID
// __stdcall
// WsNcUpdateNamespaceList(IN PNSCATALOG Catalog,
//                         IN PLIST_ENTRY List)
// {
//     LIST_ENTRY TempList;
//     PNSCATALOG_ENTRY CatalogEntry, OldCatalogEntry;
//     PLIST_ENTRY Entry;
//
//     /* First move from our list to the old one */
//     InsertHeadList(&Catalog->CatalogList, &TempList);
//     RemoveEntryList(&Catalog->CatalogList);
//     InitializeListHead(&Catalog->CatalogList);
//
//     /* Loop every item on the list */
//     while (!IsListEmpty(List))
//     {
//         /* Get the catalog entry */
//         Entry = RemoveHeadList(List);
//         CatalogEntry = CONTAINING_RECORD(Entry, NSCATALOG_ENTRY, CatalogLink);
//
//         /* Check if this item is already on our list */
//         Entry = TempList.Flink;
//         while (Entry != &TempList)
//         {
//             /* Get the catalog entry */
//             OldCatalogEntry = CONTAINING_RECORD(Entry, NSCATALOG_ENTRY, CatalogLink);
//             Entry = Entry->Flink;
//
//             /* Check if they match */
//             if (IsEqualGUID(&CatalogEntry->ProviderId,
//                             &OldCatalogEntry->ProviderId))
//             {
//                 /* We have a match, use the old item instead */
//                 WsNcEntryDereference(CatalogEntry);
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
//         InsertTailList(&Catalog->CatalogList, &CatalogEntry->CatalogLink);
//         Catalog->ItemCount++;
//     }
//
//     /* If there's anything left on the temporary list */
//     while (!IsListEmpty(&TempList))
//     {
//         /* Get the entry */
//         Entry = RemoveHeadList(&TempList);
//         CatalogEntry = CONTAINING_RECORD(Entry, NSCATALOG_ENTRY, CatalogLink);
//
//         /* Remove it */
//         Catalog->ItemCount--;
//         WsNcEntryDereference(CatalogEntry);
//     }
// }
//
// INT
// __stdcall
// WsNcGetCatalogFromProviderId(IN PNSCATALOG Catalog,
//                              IN LPGUID ProviderId,
//                              OUT PNSCATALOG_ENTRY *CatalogEntry)
// {
//     INT ErrorCode = WSAEINVAL;
//     PLIST_ENTRY NextEntry;
//     PNSCATALOG_ENTRY Entry;
//
//     /* Lock the catalog */
//     WsNcLock();
//
//     /* Match the Id with all the entries in the List */
//     NextEntry = Catalog->CatalogList.Flink;
//     while (NextEntry != &Catalog->CatalogList)
//     {
//         /* Get the Current Entry */
//         Entry = CONTAINING_RECORD(NextEntry, NSCATALOG_ENTRY, CatalogLink);
//         NextEntry = NextEntry->Flink;
//
//         /* Check if this is the Catalog Entry ID we want */
//         if (IsEqualGUID(&Entry->ProviderId, ProviderId))
//         {
//             /* If it doesn't already have a provider, load the provider */
//             if (!Entry->Provider)
//                 ErrorCode = WsNcLoadProvider(Catalog, Entry);
//
//             /* If we succeeded, reference the entry and return it */
//             if (Entry->Provider /* || ErrorCode == ERROR_SUCCESS */)
//             {
//                 InterlockedIncrement(&Entry->RefCount);
//                 *CatalogEntry = Entry;
//                 ErrorCode = ERROR_SUCCESS;
//                 break;
//             }
//         }
//     }
//
//     /* Release the lock and return */
//     WsNcUnlock();
//     return ErrorCode;
// }
//
// BOOL
// __stdcall
// WsNcMatchProtocols(IN DWORD NameSpace,
//                    IN LONG AddressFamily,
//                    IN LPWSAQUERYSETW QuerySet)
// {
//     DWORD ProtocolCount = QuerySet->dwNumberOfProtocols;
//     LPAFPROTOCOLS AfpProtocols = QuerySet->lpafpProtocols;
//     LONG Family;
//
//     /* Check for valid family */
//     if (AddressFamily != -1)
//     {
//         /* Check if it's the magic */
//         if (AddressFamily == AF_UNSPEC) return TRUE;
//         Family = AddressFamily;
//     }
//     else
//     {
//         /* No family given, check for namespace */
//         if (NameSpace == NS_SAP)
//         {
//             /* Use IPX family */
//             Family = AF_IPX;
//         }
//         else
//         {
//             /* Other namespace, it's valid */
//             return TRUE;
//         }
//     }
//
//     /* Now try to get a match */
//     while (ProtocolCount--)
//     {
//         /* Check this protocol entry */
//         if ((AfpProtocols->iAddressFamily == AF_UNSPEC) ||
//             (AfpProtocols->iAddressFamily == Family))
//         {
//             /* Match found */
//             return TRUE;
//         }
//
//         /* Move to the next one */
//         AfpProtocols++;
//     }
//
//     /* No match */
//     return FALSE;
// }
//
// VOID
// __stdcall
// WsNcRemoveCatalogItem(IN PNSCATALOG Catalog,
//                       IN PNSCATALOG_ENTRY Entry)
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
// WsNcDelete(IN PNSCATALOG Catalog)
// {
//     PLIST_ENTRY Entry;
//     PNSCATALOG_ENTRY CatalogEntry;
//
//     /* Check if we're initialized */
//     if (!Catalog->CatalogList.Flink) return;
//
//     /* Acquire lock */
//     WsNcLock();
//
//     /* Loop every entry */
//     Entry = Catalog->CatalogList.Flink;
//     while (Entry != &Catalog->CatalogList)
//     {
//         /* Get this entry */
//         CatalogEntry = CONTAINING_RECORD(Entry, NSCATALOG_ENTRY, CatalogLink);
//
//         /* Remove it and dereference it */
//         WsNcRemoveCatalogItem(Catalog, CatalogEntry);
//         WsNcEntryDereference(CatalogEntry);
//
//         /* Move to the next entry */
//         Entry = Catalog->CatalogList.Flink;
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
//     WsNcUnlock();
//     DeleteCriticalSection(&Catalog->Lock);
//
//     /* Delete the object */
//     HeapFree(WsSockHeap, 0, Catalog);
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
