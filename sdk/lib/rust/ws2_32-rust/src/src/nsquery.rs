use crate::prelude::*;
use crate::types::*;
use core::ffi::c_void;
use crate::src::nscatalo::WsNcGetCatalogFromProviderId;
use crate::src::nscatalo::WsNcGetServiceClassInfo;
use crate::src::nsprovid::WsNqProvDelete;
use crate::src::nsprovid::WsNqProvLookupServiceBegin;
use crate::src::nsprovid::WsNqProvLookupServiceEnd;
use crate::src::nsprovid::WsNqProvLookupServiceNext;
use crate::src::qshelpr::CopyQuerySetW;
use crate::src::rasdial::WSAttemptAutodialName;


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

pub const WsNqLock: () = ();
pub const WsNqUnlock: () = ();

#[no_mangle]
pub unsafe extern "system" fn WsNqAllocate() -> PNSQUERY {

    let NsQuery = HeapAlloc(
        WsSockHeap,
        HEAP_ZERO_MEMORY,
        (core::mem::size_of::<NSQUERY>() as SIZE_T),
    ) as PNSQUERY;

    if !NsQuery.is_null() {
        (*NsQuery).Signature = !0xBEADFACE;
        InitializeListHead(core::ptr::addr_of_mut!((*NsQuery).ProviderList));
        (*NsQuery).TryAgain = (1u8 as u32);
    }

    NsQuery
}

#[no_mangle]
pub unsafe extern "system" fn WsNqInitialize(Query: PNSQUERY) -> DWORD {

    InitializeCriticalSection(core::ptr::addr_of_mut!((*Query).Lock) as *mut core::ffi::c_void);
    (*Query).RefCount = 1;
    (*Query).Signature = 0xBEADFACE;
    ERROR_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn WsNqValidateAndReference(Query: PNSQUERY) -> BOOL {
    if (*Query).Signature != 0xBEADFACE {
        return (FALSE as i32);
    }
    if (*Query).RefCount == 0 {
        return (FALSE as i32);
    }
    InterlockedIncrement(core::ptr::addr_of_mut!((*Query).RefCount));
    return (TRUE as i32);
}

#[no_mangle]
pub unsafe extern "system" fn WsNqDelete(NsQuery: PNSQUERY) {
    let mut Provider: PNSQUERY_PROVIDER;
    let mut Entry: PLIST_ENTRY;

    if (*NsQuery).ProviderList.Flink.is_null() {
        return;
    }

    while !IsListEmpty(core::ptr::addr_of_mut!((*NsQuery).ProviderList)) {
        Entry = RemoveHeadList(core::ptr::addr_of_mut!((*NsQuery).ProviderList));

        Provider = (Entry as usize).wrapping_sub(
            core::ptr::addr_of_mut!((*core::ptr::null_mut::<NSQUERY_PROVIDER>()).QueryLink) as usize
        ) as PNSQUERY_PROVIDER;

        WsNqProvDelete(Provider);
    }

    (*NsQuery).Signature = !(0xBEADFACE as u32);
    DeleteCriticalSection(core::ptr::addr_of_mut!((*NsQuery).Lock));
    HeapFree(WsSockHeap, 0, NsQuery as PVOID);
}

#[no_mangle]
pub unsafe extern "system" fn WsNqDereference(Query: PNSQUERY) {
    if InterlockedDecrement(&mut (*Query).RefCount) == 0 {
        WsNqDelete(Query);
    }
}
#[no_mangle]
pub unsafe extern "system" fn WsNqBeginEnumerationProc(
    Context: PVOID,
    Entry: PNSCATALOG_ENTRY,
) -> BOOL {
    let mut Provider: PNS_PROVIDER;
    let mut GoOn: BOOLEAN = 1u8;
    let EnumContext: PENUM_CONTEXT = Context as PENUM_CONTEXT;
    let NsQuery: PNSQUERY = (*EnumContext).NsQuery;
    let NamespaceId: DWORD = (*Entry).NamespaceId;

    if (((*(*EnumContext).lpqsRestrictions).dwNameSpace == NamespaceId ||
         (*(*EnumContext).lpqsRestrictions).dwNameSpace == NS_ALL) &&
        ((*(*EnumContext).lpqsRestrictions).dwNumberOfProtocols == 0 ||
         WsNcMatchProtocols(
             NamespaceId,
             (*Entry).AddressFamily,
             (*EnumContext).lpqsRestrictions,
         ) != 0) &&
        (*Entry).Enabled != 0)
    {
        Provider = (*Entry).Provider;
        if Provider.is_null() {
            if WsNcLoadProvider((*EnumContext).Catalog, Entry) != (ERROR_SUCCESS as i32) {
                return 1;
            }
            Provider = (*Entry).Provider;
        }

        if WsNqAddProvider(NsQuery, Provider) == 0 {
            (*EnumContext).ErrorCode = (WSASYSCALLFAILURE as i32);
            GoOn = 0u8;
        }
    }

    GoOn as BOOL
}

#[no_mangle]
pub unsafe extern "system" fn WsNqLookupServiceEnd(NsQuery: PNSQUERY) -> DWORD {
    let mut Provider: PNSQUERY_PROVIDER;
    let mut Entry: PLIST_ENTRY;

    EnterCriticalSection(core::ptr::addr_of_mut!((*NsQuery).Lock));
    (*NsQuery).ShuttingDown = 1u8;

    Entry = (*NsQuery).ProviderList.Flink;
    while Entry != core::ptr::addr_of_mut!((*NsQuery).ProviderList) {
        Provider = (Entry as usize -
            core::ptr::addr_of_mut!((*core::ptr::null_mut::<NSQUERY_PROVIDER>()).QueryLink) as usize)
            as PNSQUERY_PROVIDER;

        WsNqProvLookupServiceEnd(Provider);

        Entry = (*Entry).Flink;
    }

    LeaveCriticalSection(core::ptr::addr_of_mut!((*NsQuery).Lock));
    ERROR_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn WsNqLookupServiceNext(
    NsQuery: PNSQUERY,
    ControlFlags: DWORD,
    BufferLength: PDWORD,
    Results: LPWSAQUERYSETW,
) -> DWORD {
    let mut ErrorCode: i32 = SOCKET_ERROR;
    let mut OldErrorCode: i32;
    let mut Provider: PNSQUERY_PROVIDER;
    let mut NextProvider: PNSQUERY_PROVIDER;
    let mut Entry: PLIST_ENTRY;

    if (*NsQuery).ShuttingDown != 0 {
        SetLastError(WSAECANCELLED);
        return ErrorCode as DWORD;
    }

    EnterCriticalSection(core::ptr::addr_of_mut!((*NsQuery).Lock));

    NextProvider = (*NsQuery).ActiveProvider;
    if NextProvider.is_null() {
        if (*NsQuery).CurrentProvider.is_null() {
            LeaveCriticalSection(core::ptr::addr_of_mut!((*NsQuery).Lock));
            SetLastError(WSA_E_NO_MORE);
            return SOCKET_ERROR as DWORD;
        }

        Entry = (*NsQuery).ProviderList.Blink;
        NextProvider = (Entry as usize - core::mem::offset_of!(_NSQUERY_PROVIDER, QueryLink)) as PNSQUERY_PROVIDER;

        while !NextProvider.is_null() {
            if (*(*NextProvider).Provider).Service.NSPIoctl.is_some() {
                RemoveEntryList(core::ptr::addr_of_mut!((*NextProvider).QueryLink));
                InsertHeadList(
                    core::ptr::addr_of_mut!((*NsQuery).ProviderList),
                    core::ptr::addr_of_mut!((*NextProvider).QueryLink),
                );

                (*NsQuery).ActiveProvider = NextProvider;
                break;
            }

            NextProvider = WsNqPreviousProvider(NsQuery, NextProvider);
        }
    }

    LeaveCriticalSection(core::ptr::addr_of_mut!((*NsQuery).Lock));

    while !NextProvider.is_null() {
        ErrorCode = WsNqProvLookupServiceNext(NextProvider, ControlFlags, BufferLength, Results) as i32;

        if ErrorCode == ERROR_SUCCESS as i32
            || GetLastError() == WSAEFAULT
            || (*NsQuery).ShuttingDown != 0
        {
            break;
        }

        EnterCriticalSection(core::ptr::addr_of_mut!((*NsQuery).Lock));

        if !(*NsQuery).ActiveProvider.is_null() {
            Provider = NextProvider;
            NextProvider = WsNqNextProvider(NsQuery, (*NsQuery).ActiveProvider);

            if Provider == (*NsQuery).ActiveProvider {
                (*NsQuery).ActiveProvider = NextProvider;
            }
        } else {
            NextProvider = core::ptr::null_mut();
        }

        if NextProvider.is_null() && ErrorCode == SOCKET_ERROR && (*NsQuery).TryAgain != 0 {
            OldErrorCode = GetLastError() as i32;

            (*NsQuery).TryAgain = 0;

            if WSAttemptAutodialName((*NsQuery).QuerySet) != 0 {
                while !IsListEmpty(core::ptr::addr_of_mut!((*NsQuery).ProviderList)) {
                    Entry = RemoveHeadList(core::ptr::addr_of_mut!((*NsQuery).ProviderList));
                    Provider = (Entry as usize - core::mem::offset_of!(_NSQUERY_PROVIDER, QueryLink)) as PNSQUERY_PROVIDER;

                    WsNqProvLookupServiceEnd(Provider);
                    WsNqProvDelete(Provider);
                }

                if WsNqLookupServiceBegin(
                    NsQuery,
                    (*NsQuery).QuerySet,
                    (*NsQuery).ControlFlags,
                    (*NsQuery).Catalog,
                ) == ERROR_SUCCESS
                {
                    (*NsQuery).ActiveProvider = WsNqNextProvider(NsQuery, (*NsQuery).ActiveProvider);
                }
            } else {
                SetLastError(OldErrorCode as DWORD);
            }
        }

        LeaveCriticalSection(core::ptr::addr_of_mut!((*NsQuery).Lock));
    }

    ErrorCode as DWORD
}

#[no_mangle]
pub unsafe extern "system" fn WsNqLookupServiceBegin(
    NsQuery: PNSQUERY,
    Restrictions: LPWSAQUERYSETW,
    ControlFlags: u32,
    Catalog: PNSCATALOG,
) -> u32 {
    let mut ClassInfo: WSASERVICECLASSINFOW = WSASERVICECLASSINFOW {
        lpServiceClassId: core::ptr::null_mut(),
        lpszServiceClassName: core::ptr::null_mut(),
        dwCount: 0,
        lpClassInfos: core::ptr::null_mut(),
    };
    let pClassInfo: LPWSASERVICECLASSINFOW = &mut ClassInfo;
    let mut Provider: PNSQUERY_PROVIDER;
    let mut NextProvider: PNSQUERY_PROVIDER;
    let mut Entry: PLIST_ENTRY;
    let mut ErrorCode: i32;
    let mut ClassInfoSize: u32 = 0;
    let mut CatalogEntry: PNSCATALOG_ENTRY = core::ptr::null_mut();
    let mut EnumContext: ENUM_CONTEXT = ENUM_CONTEXT {
        lpqsRestrictions: core::ptr::null_mut(),
        ErrorCode: 0,
        NsQuery: core::ptr::null_mut(),
        Catalog: core::ptr::null_mut(),
    };
    let TryAgain: u8;

    'exit: {
        if (*NsQuery).TryAgain != 0 {
            ErrorCode = CopyQuerySetW(Restrictions, &mut (*NsQuery).QuerySet);
            TryAgain = if ErrorCode == ERROR_SUCCESS as i32 { 1u8 } else { 0u8 };

            if TryAgain == 0 {
                SetLastError(ErrorCode as u32);
                ErrorCode = SOCKET_ERROR;
                (*NsQuery).TryAgain = 0;
                break 'exit;
            }

            (*NsQuery).ControlFlags = ControlFlags;
            (*NsQuery).Catalog = Catalog;
        }

        if !(*Restrictions).lpNSProviderId.is_null() {
            ErrorCode = WsNcGetCatalogFromProviderId(
                Catalog,
                (((*Restrictions).lpNSProviderId as *mut c_void) as *mut _GUID),
                &mut CatalogEntry,
            ) as i32;

            if ErrorCode != ERROR_SUCCESS as i32 {
                SetLastError(WSAEINVAL);
                ErrorCode = SOCKET_ERROR;
                break 'exit;
            } else if WsNqAddProvider(NsQuery, (*CatalogEntry).Provider) == 0 {
                SetLastError(WSA_NOT_ENOUGH_MEMORY);
                ErrorCode = SOCKET_ERROR;
                break 'exit;
            }
        } else {
            EnumContext.lpqsRestrictions = Restrictions;
            EnumContext.ErrorCode = ERROR_SUCCESS as i32;
            EnumContext.NsQuery = NsQuery;
            EnumContext.Catalog = Catalog;

            WsNcEnumerateCatalogItems(
                Catalog,
                Some(WsNqBeginEnumerationProc),
                &mut EnumContext as *mut ENUM_CONTEXT as *mut core::ffi::c_void,
            );
            ErrorCode = EnumContext.ErrorCode;

            if ErrorCode != ERROR_SUCCESS as i32 {
                SetLastError(WSAEINVAL);
                ErrorCode = SOCKET_ERROR;
                break 'exit;
            }
        }

        ClassInfo.lpServiceClassId = (*Restrictions).lpServiceClassId;
        ErrorCode = WsNcGetServiceClassInfo(Catalog, &mut ClassInfoSize, pClassInfo);

        if !(ErrorCode == SOCKET_ERROR && GetLastError() == WSAEFAULT) {
            ErrorCode = ERROR_SUCCESS as i32;
        }

        if IsListEmpty(&mut (*NsQuery).ProviderList) {
            ErrorCode = SOCKET_ERROR;
            SetLastError(WSASERVICE_NOT_FOUND);
            break 'exit;
        }

        Entry = (*NsQuery).ProviderList.Flink;
        NextProvider = (Entry as usize).wrapping_sub(
            core::ptr::addr_of_mut!((*core::ptr::null_mut::<_NSQUERY_PROVIDER>()).QueryLink) as usize
        ) as PNSQUERY_PROVIDER;

        while !NextProvider.is_null() {
            ErrorCode = (WsNqProvLookupServiceBegin(
                NextProvider,
                Restrictions,
                pClassInfo,
                ControlFlags,
            ) as i32);

            if ErrorCode == SOCKET_ERROR {
                Provider = NextProvider;
                NextProvider = WsNqNextProvider(NsQuery, NextProvider);
                RemoveEntryList(&mut (*Provider).QueryLink);
                WsNqProvDelete(Provider);
            } else {
                NextProvider = WsNqNextProvider(NsQuery, NextProvider);
            }
        }
    }

    if ErrorCode == SOCKET_ERROR {
        while !IsListEmpty(&mut (*NsQuery).ProviderList) {
            Entry = RemoveHeadList(&mut (*NsQuery).ProviderList);

            Provider = (Entry as usize).wrapping_sub(
                core::ptr::addr_of_mut!((*core::ptr::null_mut::<_NSQUERY_PROVIDER>()).QueryLink) as usize
            ) as PNSQUERY_PROVIDER;
            WsNqProvDelete(Provider);
        }
    } else {
        Entry = (*NsQuery).ProviderList.Flink;
        (*NsQuery).ActiveProvider = (Entry as usize).wrapping_sub(
            core::ptr::addr_of_mut!((*core::ptr::null_mut::<_NSQUERY_PROVIDER>()).QueryLink) as usize
        ) as PNSQUERY_PROVIDER;
    }

    ErrorCode as u32
}
#[no_mangle]
pub unsafe extern "system" fn WsNqNextProvider(
    Query: PNSQUERY,
    Provider: PNSQUERY_PROVIDER,
) -> PNSQUERY_PROVIDER {
    let mut NextProvider: PNSQUERY_PROVIDER = core::ptr::null_mut();
    let mut Entry: PLIST_ENTRY;

    Entry = (*Provider).QueryLink.Flink;
    if Entry != core::ptr::addr_of_mut!((*Query).ProviderList) {
        NextProvider = (Entry as usize).wrapping_sub(
            core::ptr::addr_of_mut!((*core::ptr::null_mut::<NSQUERY_PROVIDER>()).QueryLink) as usize
        ) as PNSQUERY_PROVIDER;
    }

    return NextProvider;
}
#[no_mangle]
pub unsafe extern "system" fn WsNqPreviousProvider(
    Query: PNSQUERY,
    Provider: PNSQUERY_PROVIDER,
) -> PNSQUERY_PROVIDER {
    let mut PrevProvider: PNSQUERY_PROVIDER = core::ptr::null_mut();
    let mut Entry: PLIST_ENTRY;

    Entry = (*Provider).QueryLink.Blink;
    if Entry != core::ptr::addr_of_mut!((*Query).ProviderList) {
        PrevProvider = (Entry as usize).wrapping_sub(
            core::ptr::addr_of_mut!((*core::ptr::null_mut::<_NSQUERY_PROVIDER>()).QueryLink) as usize
        ) as PNSQUERY_PROVIDER;
    }

    return PrevProvider;
}
#[no_mangle]
pub unsafe extern "system" fn WsNqAddProvider(Query: PNSQUERY, Provider: PNS_PROVIDER) -> BOOL {
    let mut Success: BOOL = (1u8 as i32);
    let QueryProvider: PNSQUERY_PROVIDER;

    QueryProvider = WsNqProvAllocate();
    if !QueryProvider.is_null() {
        WsNqProvInitialize(QueryProvider, Provider);
        InsertTailList(
            core::ptr::addr_of_mut!((*Query).ProviderList),
            core::ptr::addr_of_mut!((*QueryProvider).QueryLink),
        );
    } else {
        SetLastError(WSASYSCALLFAILURE as DWORD);
        Success = (0u8 as i32);
    }

    Success
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:   See COPYING in the top level directory
//  * PROJECT:     ReactOS WinSock 2 API
//  * FILE:        dll/win32/ws2_32/src/nsquery.c
//  * PURPOSE:     Namespace Query Object
//  * PROGRAMMER:  Alex Ionescu (alex@relsoft.net)
//  */
//
// /* INCLUDES ******************************************************************/
//
// #include <ws2_32.h>
//
// /* DATA **********************************************************************/
//
// #define WsNqLock()          EnterCriticalSection(&NsQuery->Lock)
// #define WsNqUnlock()        LeaveCriticalSection(&NsQuery->Lock)
//
// /* FUNCTIONS *****************************************************************/
//
// PNSQUERY
// __stdcall
// WsNqAllocate(VOID)
// {
//     PNSQUERY NsQuery;
//
//     /* Allocate the object */
//     NsQuery = HeapAlloc(WsSockHeap, HEAP_ZERO_MEMORY, sizeof(*NsQuery));
//     if (NsQuery)
//     {
//         /* Set non-zero fields */
//         NsQuery->Signature = ~0xBEADFACE;
//         InitializeListHead(&NsQuery->ProviderList);
//         NsQuery->TryAgain = TRUE;
//     }
//
//     /* Return it */
//     return NsQuery;
// }
//
// DWORD
// __stdcall
// WsNqInitialize(IN PNSQUERY Query)
// {
//     /* Initialize the lock */
//     InitializeCriticalSection(&Query->Lock);
//
//     /* Set initial reference count and signature */
//     Query->RefCount = 1;
//     Query->Signature = 0xBEADFACE;
//
//     /* Return success */
//     return ERROR_SUCCESS;
// }
//
// BOOL
// __stdcall
// WsNqValidateAndReference(IN PNSQUERY Query)
// {
//     /* Check the signature first */
//     if (Query->Signature != 0xBEADFACE) return FALSE;
//
//     /* Validate the reference count */
//     if (!Query->RefCount) return FALSE;
//
//     /* Increase reference count */
//     InterlockedIncrement(&Query->RefCount);
//
//     /* Return success */
//     return TRUE;
// }
//
// VOID
// __stdcall
// WsNqDelete(IN PNSQUERY NsQuery)
// {
//     PNSQUERY_PROVIDER Provider;
//     PLIST_ENTRY Entry;
//
//     /* Make sure that we got initialized */
//     if (!NsQuery->ProviderList.Flink) return;
//
//     /* Loop the provider list */
//     while (!IsListEmpty(&NsQuery->ProviderList))
//     {
//         /* Remove the entry */
//         Entry = RemoveHeadList(&NsQuery->ProviderList);
//
//         /* Get the provider */
//         Provider = CONTAINING_RECORD(Entry, NSQUERY_PROVIDER, QueryLink);
//
//         /* Delete it */
//         WsNqProvDelete(Provider);
//     }
//
//     /* Remove the signature and delete the lock */
//     NsQuery->Signature = ~0xBEADFACE;
//     DeleteCriticalSection(&NsQuery->Lock);
//
//     /* Free us */
//     HeapFree(WsSockHeap, 0, NsQuery);
// }
//
// VOID
// __stdcall
// WsNqDereference(IN PNSQUERY Query)
// {
//     /* Decrease the reference count and check if it's zero */
//     if (!InterlockedDecrement(&Query->RefCount))
//     {
//         /* Delete us*/
//         WsNqDelete(Query);
//     }
// }
//
// BOOL
// __stdcall
// WsNqBeginEnumerationProc(PVOID Context,
//                          PNSCATALOG_ENTRY Entry)
// {
//     PNS_PROVIDER Provider;
//     BOOLEAN GoOn = TRUE;
//     PENUM_CONTEXT EnumContext = (PENUM_CONTEXT)Context;
//     PNSQUERY NsQuery = EnumContext->NsQuery;
//     DWORD NamespaceId = Entry->NamespaceId;
//
//     /* Match the namespace ID, protocols and make sure it's enabled */
//     if ((((EnumContext->lpqsRestrictions->dwNameSpace == NamespaceId) ||
//           (EnumContext->lpqsRestrictions->dwNameSpace == NS_ALL)) &&
//          (!(EnumContext->lpqsRestrictions->dwNumberOfProtocols) ||
//            (WsNcMatchProtocols(NamespaceId,
//                                Entry->AddressFamily,
//                                EnumContext->lpqsRestrictions)))) &&
//         (Entry->Enabled))
//     {
//         /* Get the provider */
//         if (!(Provider = Entry->Provider))
//         {
//             /* None was loaded, load it */
//             if (WsNcLoadProvider(EnumContext->Catalog, Entry) != ERROR_SUCCESS)
//             {
//                 /* Return TRUE to continue enumerating */
//                 return TRUE;
//             }
//
//             /* Set the provider */
//             Provider = Entry->Provider;
//         }
//
//         /* Add it to the query */
//         if (!WsNqAddProvider(NsQuery, Provider))
//         {
//             /* We failed */
//             EnumContext->ErrorCode = WSASYSCALLFAILURE;
//             GoOn = FALSE;
//         }
//     }
//
//     /* Return to caller */
//     return GoOn;
// }
//
// DWORD
// __stdcall
// WsNqLookupServiceEnd(IN PNSQUERY NsQuery)
// {
//     PNSQUERY_PROVIDER Provider;
//     PLIST_ENTRY Entry;
//
//     /* Protect us from closure */
//     WsNqLock();
//     NsQuery->ShuttingDown = TRUE;
//
//     /* Get the list and loop */
//     Entry = NsQuery->ProviderList.Flink;
//     while (Entry != &NsQuery->ProviderList)
//     {
//         /* Get the provider */
//         Provider = CONTAINING_RECORD(Entry, NSQUERY_PROVIDER, QueryLink);
//
//         /* Call its routine */
//         WsNqProvLookupServiceEnd(Provider);
//
//         /* Move to the next one */
//         Entry = Entry->Flink;
//     }
//
//     /* Release lock and return success */
//     WsNqUnlock();
//     return ERROR_SUCCESS;
// }
//
// DWORD
// __stdcall
// WsNqLookupServiceNext(IN PNSQUERY NsQuery,
//                       IN DWORD ControlFlags,
//                       OUT PDWORD BufferLength,
//                       OUT LPWSAQUERYSETW Results)
// {
//     INT ErrorCode = SOCKET_ERROR, OldErrorCode;
//     PNSQUERY_PROVIDER Provider, NextProvider;
//     PLIST_ENTRY Entry;
//
//     /* Make sure we're not shutting down */
//     if (NsQuery->ShuttingDown)
//     {
//         /* We are shutting down, fail */
//         SetLastError(WSAECANCELLED);
//         return ErrorCode;
//     }
//
//     /* Acquire query lock */
//     WsNqLock();
//
//     /* Check if we already have an active provider */
//     NextProvider = NsQuery->ActiveProvider;
//     if (!NextProvider)
//     {
//         /* Make sure we have a current provider */
//         if (!NsQuery->CurrentProvider)
//         {
//             /* We don't; fail */
//             WsNqUnlock();
//             SetLastError(WSA_E_NO_MORE);
//             return SOCKET_ERROR;
//         }
//
//         /* Get the last provider in the list and start looping */
//         Entry = NsQuery->ProviderList.Blink;
//         NextProvider = CONTAINING_RECORD(Entry, NSQUERY_PROVIDER, QueryLink);
//         while (NextProvider)
//         {
//             /* Check if this is a new-style provider */
//             if (NextProvider->Provider->Service.NSPIoctl)
//             {
//                 /* Remove it and re-add it on top */
//                 RemoveEntryList(&NextProvider->QueryLink);
//                 InsertHeadList(&NsQuery->ProviderList, &NextProvider->QueryLink);
//
//                 /* Set it as the active provider and exit the loop */
//                 NsQuery->ActiveProvider = NextProvider;
//                 break;
//             }
//
//             /* Get the previous provider */
//             NextProvider = WsNqPreviousProvider(NsQuery, NextProvider);
//         }
//     }
//
//     /* Release the lock */
//     WsNqUnlock();
//
//     /* Restart and keep looping as long as there is an active provider */
//     while (NextProvider)
//     {
//         /* Call its routine */
//         ErrorCode = WsNqProvLookupServiceNext(NextProvider,
//                                               ControlFlags,
//                                               BufferLength,
//                                               Results);
//         /* Check for error or shutdown */
//         if ((ErrorCode == ERROR_SUCCESS) ||
//             (GetLastError() == WSAEFAULT) || (NsQuery->ShuttingDown))
//         {
//             /* Get out */
//             break;
//         }
//
//         /* Acquire Query Lock */
//         WsNqLock();
//
//         /* Check if we have an active provider */
//         if (NsQuery->ActiveProvider)
//         {
//             /* Save the old provider and get the next one */
//             Provider = NextProvider;
//             NextProvider = WsNqNextProvider(NsQuery, NsQuery->ActiveProvider);
//
//             /* Was the old provider our active? */
//             if (Provider == NsQuery->ActiveProvider)
//             {
//                 /* Change our active provider to the new one */
//                 NsQuery->ActiveProvider = NextProvider;
//             }
//         }
//         else
//         {
//             /* No next provider */
//             NextProvider = NULL;
//         }
//
//         /* Check if we failed and if we can try again */
//         if (!(NextProvider) &&
//             (ErrorCode == SOCKET_ERROR) &&
//             (NsQuery->TryAgain))
//         {
//             /* Save the error code so RAS doesn't overwrite it */
//             OldErrorCode = GetLastError();
//
//             /* Make sure we won't try for a 3rd time */
//             NsQuery->TryAgain = FALSE;
//
//             /* Call the helper to auto-dial */
//             if (WSAttemptAutodialName(NsQuery->QuerySet))
//             {
//                 /* It succeeded, so we'll delete the current state. */
//                 while (!IsListEmpty(&NsQuery->ProviderList))
//                 {
//                     /* Remove the entry and get its provider */
//                     Entry = RemoveHeadList(&NsQuery->ProviderList);
//                     Provider = CONTAINING_RECORD(Entry,
//                                                  NSQUERY_PROVIDER,
//                                                  QueryLink);
//
//                     /* Reset it */
//                     WsNqProvLookupServiceEnd(Provider);
//                     WsNqProvDelete(Provider);
//                 }
//
//                 /* Start a new query */
//                 if (WsNqLookupServiceBegin(NsQuery,
//                                            NsQuery->QuerySet,
//                                            NsQuery->ControlFlags,
//                                            NsQuery->Catalog) == ERROR_SUCCESS)
//                 {
//                     /* New query succeeded, set active provider now */
//                     NsQuery->ActiveProvider =
//                         WsNqNextProvider(NsQuery, NsQuery->ActiveProvider);
//                 }
//             }
//             else
//             {
//                 /* Reset the error code */
//                 SetLastError(OldErrorCode);
//             }
//         }
//
//         /* Release lock */
//         WsNqUnlock();
//     }
//
//     /* Return */
//     return ErrorCode;
// }
//
// DWORD
// __stdcall
// WsNqLookupServiceBegin(IN PNSQUERY NsQuery,
//                        IN LPWSAQUERYSETW Restrictions,
//                        IN DWORD ControlFlags,
//                        IN PNSCATALOG Catalog)
// {
//     WSASERVICECLASSINFOW ClassInfo;
//     LPWSASERVICECLASSINFOW pClassInfo = &ClassInfo;
//     PNSQUERY_PROVIDER Provider, NextProvider;
//     PLIST_ENTRY Entry;
//     INT ErrorCode;
//     DWORD ClassInfoSize;
//     PNSCATALOG_ENTRY CatalogEntry = NULL;
//     ENUM_CONTEXT EnumContext;
//     BOOLEAN TryAgain;
//
//     /* Check for RAS Auto-dial attempt */
//     if (NsQuery->TryAgain)
//     {
//         /* Make a copy of the query set */
//         ErrorCode = CopyQuerySetW(Restrictions, &NsQuery->QuerySet);
//         TryAgain = (ErrorCode == ERROR_SUCCESS);
//
//         /* Check if we'll try again */
//         if (!TryAgain)
//         {
//             /* We won't, fail */
//             SetLastError(ErrorCode);
//             ErrorCode = SOCKET_ERROR;
//             NsQuery->TryAgain = FALSE;
//             goto Exit;
//         }
//
//         /* Cache the information for a restart */
//         NsQuery->ControlFlags = ControlFlags;
//         NsQuery->Catalog = Catalog;
//     }
//
//     /* Check if we have a specific ID */
//     if (Restrictions->lpNSProviderId)
//     {
//         /* Get the catalog entry */
//         ErrorCode = WsNcGetCatalogFromProviderId(Catalog,
//                                                  Restrictions->lpNSProviderId,
//                                                  &CatalogEntry);
//         /* Check for failure */
//         if (ErrorCode != ERROR_SUCCESS)
//         {
//             /* Fail */
//             SetLastError(WSAEINVAL);
//             ErrorCode = SOCKET_ERROR;
//             goto Exit;
//         }
//         /* We succeeded, add this provider */
//         else if (!WsNqAddProvider(NsQuery, CatalogEntry->Provider))
//         {
//             /* Fail */
//             SetLastError(WSA_NOT_ENOUGH_MEMORY);
//             ErrorCode = SOCKET_ERROR;
//             goto Exit;
//         }
//     }
//     else
//     {
//         /* Setup the lookup context */
//         EnumContext.lpqsRestrictions = Restrictions;
//         EnumContext.ErrorCode = ERROR_SUCCESS;
//         EnumContext.NsQuery = NsQuery;
//         EnumContext.Catalog = Catalog;
//
//         /* Do a lookup for every entry */
//         WsNcEnumerateCatalogItems(Catalog,
//                                   WsNqBeginEnumerationProc,
//                                   &EnumContext);
//         ErrorCode = EnumContext.ErrorCode;
//
//         /* Check for success */
//         if (ErrorCode != ERROR_SUCCESS)
//         {
//             /* Fail */
//             SetLastError(WSAEINVAL);
//             ErrorCode = SOCKET_ERROR;
//             goto Exit;
//         }
//     }
//
//     /* Get the class information */
//     ClassInfo.lpServiceClassId = Restrictions->lpServiceClassId;
//     ErrorCode = WsNcGetServiceClassInfo(Catalog, &ClassInfoSize, pClassInfo);
//
//     /* Check if more buffer space is needed */
//     if ((ErrorCode == SOCKET_ERROR) && (GetLastError() == WSAEFAULT))
//     {
//         /* FIXME: The WS 2.2 spec hasn't been finalized yet on this... */
//     }
//     else
//     {
//         /* Assume success */
//         ErrorCode = ERROR_SUCCESS;
//     }
//
//     /* Check if the provider list is empty */
//     if (IsListEmpty(&NsQuery->ProviderList))
//     {
//         /* We don't have any providers to handle this! */
//         ErrorCode = SOCKET_ERROR;
//         SetLastError(WSASERVICE_NOT_FOUND);
//         goto Exit;
//     }
//
//     /* Get the first provider and loop */
//     Entry = NsQuery->ProviderList.Flink;
//     NextProvider = CONTAINING_RECORD(Entry, NSQUERY_PROVIDER, QueryLink);
//     while (NextProvider)
//     {
//         /* Call it */
//         ErrorCode = WsNqProvLookupServiceBegin(NextProvider,
//                                                Restrictions,
//                                                pClassInfo,
//                                                ControlFlags);
//         /* Check for error */
//         if (ErrorCode == SOCKET_ERROR)
//         {
//             /* Remove this provider, get the next one, delete the old one */
//             Provider = NextProvider;
//             NextProvider = WsNqNextProvider(NsQuery, NextProvider);
//             RemoveEntryList(&Provider->QueryLink);
//             WsNqProvDelete(Provider);
//         }
//         else
//         {
//             /* Get the next provider */
//             NextProvider = WsNqNextProvider(NsQuery, NextProvider);
//         }
//     }
//
// Exit:
//     /* Check if we had an error somewhere */
//     if (ErrorCode == SOCKET_ERROR)
//     {
//         /* Loop the list */
//         while (!IsListEmpty(&NsQuery->ProviderList))
//         {
//             /* Remove this provider */
//             Entry = RemoveHeadList(&NsQuery->ProviderList);
//
//             /* Get the failed provider and delete it */
//             Provider = CONTAINING_RECORD(Entry, NSQUERY_PROVIDER, QueryLink);
//             WsNqProvDelete(Provider);
//         }
//     }
//     else
//     {
//         /* Set the active provider */
//         Entry = NsQuery->ProviderList.Flink;
//         NsQuery->ActiveProvider = CONTAINING_RECORD(Entry,
//                                                     NSQUERY_PROVIDER,
//                                                     QueryLink);
//     }
//
//     /* Return */
//     return ErrorCode;
// }
//
// PNSQUERY_PROVIDER
// __stdcall
// WsNqNextProvider(IN PNSQUERY Query,
//                  IN PNSQUERY_PROVIDER Provider)
// {
//     PNSQUERY_PROVIDER NextProvider = NULL;
//     PLIST_ENTRY Entry;
//
//     /* Get the first entry and get its provider */
//     Entry = Provider->QueryLink.Flink;
//     if (Entry != &Query->ProviderList)
//     {
//         /* Get the current provider */
//         NextProvider = CONTAINING_RECORD(Entry, NSQUERY_PROVIDER, QueryLink);
//     }
//
//     /* Return it */
//     return NextProvider;
// }
//
// PNSQUERY_PROVIDER
// __stdcall
// WsNqPreviousProvider(IN PNSQUERY Query,
//                      IN PNSQUERY_PROVIDER Provider)
// {
//     PNSQUERY_PROVIDER PrevProvider = NULL;
//     PLIST_ENTRY Entry;
//
//     /* Get the last entry and get its provider */
//     Entry = Provider->QueryLink.Blink;
//     if (Entry != &Query->ProviderList)
//     {
//         /* Get the current provider */
//         PrevProvider = CONTAINING_RECORD(Entry, NSQUERY_PROVIDER, QueryLink);
//     }
//
//     /* Return it */
//     return PrevProvider;
// }
//
// BOOL
// __stdcall
// WsNqAddProvider(IN PNSQUERY Query,
//                 IN PNS_PROVIDER Provider)
// {
//     BOOL Success = TRUE;
//     PNSQUERY_PROVIDER QueryProvider;
//
//     /* Allocate a new Query Provider */
//     if ((QueryProvider = WsNqProvAllocate()))
//     {
//         /* Initialize it */
//         WsNqProvInitialize(QueryProvider, Provider);
//
//         /* Insert it into the provider list */
//         InsertTailList(&Query->ProviderList, &QueryProvider->QueryLink);
//     }
//     else
//     {
//         /* We failed */
//         SetLastError(WSASYSCALLFAILURE);
//         Success = FALSE;
//     }
//
//     /* Return */
//     return Success;
// }
//
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
