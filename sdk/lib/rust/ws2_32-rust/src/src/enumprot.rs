use crate::prelude::*;
use crate::types::*;
use core::ffi::c_void;
use crate::src::dcatalog::WsTcAllocate;
use crate::src::dcatalog::WsTcInitializeFromRegistry;
use crate::src::dprocess::WsProcGetNsCatalog;
use crate::src::wsautil::WsOpenRegistryRoot;


// ---- file-scope preamble (translated) ----
// No file-scope declarations to translate from this preamble.
// The #define NDEBUG is a preprocessor directive that affects compilation
// but does not produce runtime declarations.
#[no_mangle]
pub unsafe extern "system" fn CheckProtocolMatch(
    ProtocolSet: LPINT,
    ProtocolInfo: LPWSAPROTOCOL_INFOW
) -> i32 {
    let mut Return: i32 = 0;
    let mut i: u32 = 0;
    let mut ProtocolId: i32;

    if !ProtocolSet.is_null() {
        ProtocolId = *ProtocolSet.offset(i as isize);

        while ProtocolId != 0 && (*ProtocolInfo).iProtocol != 0 {
            if (ProtocolId >= (*ProtocolInfo).iProtocol) &&
               (ProtocolId <= ((*ProtocolInfo).iProtocol +
                               (*ProtocolInfo).iProtocolMaxOffset)) {
                Return = 1;
                break;
            }

            i += 1;
            ProtocolId = *ProtocolSet.offset(i as isize);
        }
    } else {
        Return = 1;
    }

    Return
}
#[no_mangle]
pub unsafe extern "system" fn ProtocolInfoFromContext(
    ProtocolInfo: LPWSAPROTOCOL_INFOW,
    Context: PPROTOCOL_ENUM_CONTEXT,
) {
    if ((*Context).BufferUsed + (core::mem::size_of::<WSAPROTOCOL_INFOW>() as u32))
        <= (*Context).BufferLength
    {
        core::ptr::copy_nonoverlapping(
            ProtocolInfo,
            (((*Context).ProtocolBuffer as usize) + ((*Context).BufferUsed as usize)) as LPWSAPROTOCOL_INFOW,
            1,
        );

        (*Context).Count += 1;
    }
}
#[no_mangle]
pub unsafe extern "system" fn ProtocolEnumerationProc(
    EnumContext: PVOID,
    Entry: PTCATALOG_ENTRY
) -> BOOL {
    let Context = EnumContext as PPROTOCOL_ENUM_CONTEXT;
    let ProtocolInfo = core::ptr::addr_of_mut!((*Entry).ProtocolInfo);

    if CheckProtocolMatch((*Context).Protocols, ProtocolInfo) != 0 {
        ProtocolInfoFromContext(ProtocolInfo, Context);
        (*Context).BufferUsed += core::mem::size_of::<WSAPROTOCOL_INFOW>() as u32;
    }

    1i32
}
#[no_mangle]
pub unsafe extern "system" fn ProviderEnumerationProc(
    EnumContext: PVOID,
    Entry: PNSCATALOG_ENTRY,
) -> BOOL {
    let mut PathLen: INT;
    let Context = EnumContext as PPROVIDER_ENUM_CONTEXT;

    if IsEqualGUID(
        core::ptr::addr_of_mut!((*Entry).ProviderId),
        core::ptr::addr_of_mut!((*Context).ProviderId),
    )
    {
        PathLen = wcslen((*Entry).DllPath.as_ptr()) as INT + 1;
        (*Context).FoundPathLen = (PathLen as u32);
        (*Context).Found = 1;

        if PathLen <= (*Context).ProviderDllPathLen {
            wcscpy((*Context).ProviderDllPath, (*Entry).DllPath.as_ptr());
        }

        return 0;
    } else {
        return 1;
    }
}

#[no_mangle]
pub unsafe extern "system" fn OpenInitializedCatalog() -> PTCATALOG {
    let mut Catalog: PTCATALOG;
    let mut WsKey: HKEY;

    Catalog = WsTcAllocate();
    if !Catalog.is_null() {
        WsKey = WsOpenRegistryRoot();
        WsTcInitializeFromRegistry(Catalog, WsKey, core::ptr::null_mut());
        RegCloseKey(WsKey);
    }

    return Catalog;
}

#[no_mangle]
pub unsafe extern "system" fn WSCEnumProtocols(
    lpiProtocols: LPINT,
    lpProtocolBuffer: LPWSAPROTOCOL_INFOW,
    lpdwBufferLength: LPDWORD,
    lpErrno: LPINT,
) -> INT {
    let mut Status: INT;
    let Catalog: PTCATALOG;
    let mut Context: PROTOCOL_ENUM_CONTEXT = core::mem::zeroed();

    Catalog = OpenInitializedCatalog();
    if Catalog.is_null() {
        *lpErrno = (WSAENOBUFS as i32);
        return SOCKET_ERROR;
    }

    Context.Protocols = lpiProtocols;
    Context.ProtocolBuffer = lpProtocolBuffer;
    Context.BufferLength = if !lpProtocolBuffer.is_null() {
        *lpdwBufferLength
    } else {
        0
    };
    Context.BufferUsed = 0;
    Context.Count = 0;
    Context.ErrorCode = (ERROR_SUCCESS as i32);

    WsTcEnumerateCatalogItems(
        Catalog,
        Some(ProtocolEnumerationProc),
        &mut Context as *mut _ as *mut core::ffi::c_void,
    );

    Status = (Context.Count as i32);

    if Context.ErrorCode == (ERROR_SUCCESS as i32) {
        if Context.BufferLength < Context.BufferUsed {
            *lpdwBufferLength = Context.BufferUsed;
            *lpErrno = (WSAENOBUFS as i32);
            Status = SOCKET_ERROR;
        }
    } else {
        Status = SOCKET_ERROR;
        *lpErrno = Context.ErrorCode;
    }

    WsTcDelete(Catalog);

    Status
}

#[no_mangle]
pub unsafe extern "system" fn WSAEnumProtocolsA(
    lpiProtocols: LPINT,
    lpProtocolBuffer: LPWSAPROTOCOL_INFOA,
    lpdwBufferLength: LPDWORD,
) -> INT {

    let mut error: INT = unsafe { core::mem::zeroed() };
    let mut i: INT;
    let mut count: INT;
    let mut protocolInfoW: LPWSAPROTOCOL_INFOW;
    let mut size: DWORD = 0;

    if lpdwBufferLength.is_null() {
        SetLastError(WSAENOBUFS as u32);
        return SOCKET_ERROR;
    }

    count = WSCEnumProtocols(lpiProtocols, core::ptr::null_mut(), &mut size, &mut error);

    if lpProtocolBuffer.is_null() || *lpdwBufferLength < (size / core::mem::size_of::<WSAPROTOCOL_INFOW>() as u32) * core::mem::size_of::<WSAPROTOCOL_INFOA>() as u32 {
        *lpdwBufferLength = (size / core::mem::size_of::<WSAPROTOCOL_INFOW>() as u32) * core::mem::size_of::<WSAPROTOCOL_INFOA>() as u32;
        SetLastError(WSAENOBUFS as u32);
        return SOCKET_ERROR;
    }

    protocolInfoW = HeapAlloc(WsSockHeap, 0, ((size as usize as u64)) as SIZE_T) as LPWSAPROTOCOL_INFOW;
    count = WSCEnumProtocols(lpiProtocols, protocolInfoW, &mut size, &mut error);

    if SOCKET_ERROR == count {
        HeapFree(WsSockHeap, 0, protocolInfoW as LPVOID);
        SetLastError(error as u32);
        return SOCKET_ERROR;
    }

    *lpdwBufferLength = 0;
    i = 0;
    while i < count {
        let dest_offset = core::mem::size_of::<WSAPROTOCOL_INFOA>() - core::mem::size_of_val(&(*lpProtocolBuffer.offset(i as isize)).szProtocol);
        core::ptr::copy_nonoverlapping(
            protocolInfoW.offset(i as isize) as *const u8,
            lpProtocolBuffer.offset(i as isize) as *mut u8,
            dest_offset,
        );
        wcstombs(
            (*lpProtocolBuffer.offset(i as isize)).szProtocol.as_mut_ptr(),
            (*protocolInfoW.offset(i as isize)).szProtocol.as_ptr(),
            core::mem::size_of_val(&(*lpProtocolBuffer.offset(i as isize)).szProtocol),
        );
        *lpdwBufferLength += core::mem::size_of::<WSAPROTOCOL_INFOA>() as u32;
        i += 1;
    }

    HeapFree(WsSockHeap, 0, protocolInfoW as LPVOID);
    i
}

#[no_mangle]
pub unsafe extern "system" fn WSAEnumProtocolsW(
    lpiProtocols: LPINT,
    lpProtocolBuffer: LPWSAPROTOCOL_INFOW,
    lpdwBufferLength: LPDWORD,
) -> INT {
    let mut error: INT = 0;
    let count: INT = WSCEnumProtocols(
        lpiProtocols,
        lpProtocolBuffer,
        lpdwBufferLength,
        &mut error as LPINT,
    );
    if SOCKET_ERROR == count {
        SetLastError(error as u32);
        return SOCKET_ERROR;
    }
    return count;
}

#[no_mangle]
pub unsafe extern "system" fn WPUGetProviderPath(
    lpProviderId: LPGUID,
    lpszProviderDllPath: LPWSTR,
    lpProviderDllPathLen: LPINT,
    lpErrno: LPINT,
) -> INT {
    WSCGetProviderPath(lpProviderId, lpszProviderDllPath, lpProviderDllPathLen, lpErrno)
}

#[no_mangle]
pub unsafe extern "system" fn WSAProviderConfigChange(
    lpNotificationHandle: LPHANDLE,
    lpOverlapped: LPWSAOVERLAPPED,
    lpCompletionRoutine: LPWSAOVERLAPPED_COMPLETION_ROUTINE,
) -> INT {

    SetLastError(WSAEINVAL);
    SOCKET_ERROR
}

#[no_mangle]
pub unsafe extern "system" fn WSCGetProviderPath(
    lpProviderId: LPGUID,
    lpszProviderDllPath: LPWSTR,
    lpProviderDllPathLen: LPINT,
    lpErrno: LPINT,
) -> INT {
    let mut Thread: PWSTHREAD = core::ptr::null_mut();
    let mut Process: PWSPROCESS = core::ptr::null_mut();
    let Catalog: PNSCATALOG;
    let mut ErrorCode: INT;
    let PathLen: INT;
    let mut Context: PROVIDER_ENUM_CONTEXT = core::mem::zeroed();

    ErrorCode = WsApiProlog(&mut Process, &mut Thread);
    if ErrorCode != ERROR_SUCCESS as INT {
        *lpErrno = ErrorCode;
        return SOCKET_ERROR;
    }

    Catalog = WsProcGetNsCatalog(Process);

    Context.ProviderId = *lpProviderId;
    Context.ProviderDllPath = lpszProviderDllPath;
    Context.ProviderDllPathLen = *lpProviderDllPathLen;
    Context.FoundPathLen = 0;
    Context.Found = 0;
    Context.ErrorCode = ERROR_SUCCESS as INT;

    ErrorCode = ERROR_SUCCESS as INT;

    WsNcEnumerateCatalogItems(Catalog, Some(ProviderEnumerationProc), &mut Context as *mut _ as *mut core::ffi::c_void);

    if Context.ErrorCode == ERROR_SUCCESS as INT {
        if Context.Found != 0 {
            PathLen = (Context.FoundPathLen as i32);

            if Context.FoundPathLen <= (*lpProviderDllPathLen as u32) {
                *lpProviderDllPathLen = ((Context.FoundPathLen - (1 as i32 as u32)) as i32);
            } else {
                ErrorCode = (WSAEFAULT as i32);
                *lpProviderDllPathLen = PathLen;
            }
        } else {
            ErrorCode = (WSAEINVAL as i32);
        }
    } else {
        ErrorCode = Context.ErrorCode;
    }

    if ErrorCode != ERROR_SUCCESS as INT {
        *lpErrno = ErrorCode;
        return SOCKET_ERROR;
    }

    0
}
// enforce_import_abi: moved 3 import decl(s) into the extern block their
// exporting DLL's .spec requires: wcscpy->C, wcslen->C, wcstombs->C.
// The CRT is cdecl, Win32 is stdcall, and the C source does not say which is which —
// a wrong guess is a silent i386 stack walk on every call. The .spec is the truth.


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:   See COPYING in the top level directory
//  * PROJECT:     ReactOS WinSock 2 API
//  * FILE:        dll/win32/ws2_32/src/enumprot.c
//  * PURPOSE:     Protocol Enumeration
//  * PROGRAMMER:  Alex Ionescu (alex@relsoft.net)
//  *              Pierre Schweitzer
//  */
//
// /* INCLUDES ******************************************************************/
//
// #include <ws2_32.h>
//
// #define NDEBUG
// #include <debug.h>
//
// /* FUNCTIONS *****************************************************************/
//
// BOOL
// __stdcall
// CheckProtocolMatch(IN LPINT ProtocolSet,
//                    IN LPWSAPROTOCOL_INFOW ProtocolInfo)
// {
//     BOOL Return = FALSE;
//     DWORD i = 0;
//     INT ProtocolId;
//
//     /* Make sure we have a set */
//     if (ProtocolSet)
//     {
//         /* Get the first ID */
//         ProtocolId = ProtocolSet[i];
//
//         /* Loop the list */
//         while (ProtocolId != 0 && ProtocolInfo->iProtocol != 0)
//         {
//             /* Check if it's within ranges */
//             if ((ProtocolId >= ProtocolInfo->iProtocol) &&
//                 (ProtocolId <= (ProtocolInfo->iProtocol +
//                                 ProtocolInfo->iProtocolMaxOffset)))
//             {
//                 /* Found it */
//                 Return = TRUE;
//                 break;
//             }
//
//             /* Move on */
//             i++;
//             ProtocolId = ProtocolSet[i];
//         }
//     }
//     else
//     {
//         /* Assume match */
//         Return = TRUE;
//     }
//
//     /* Return result */
//     return Return;
// }
//
// VOID
// __stdcall
// ProtocolInfoFromContext(IN LPWSAPROTOCOL_INFOW ProtocolInfo,
//                         IN PPROTOCOL_ENUM_CONTEXT Context)
// {
//     /* Check if we'll have space */
//     if ((Context->BufferUsed + sizeof(*ProtocolInfo)) <=
//         (Context->BufferLength))
//     {
//         /* Copy the data */
//         RtlMoveMemory((PVOID)((ULONG_PTR)Context->ProtocolBuffer +
//                               Context->BufferUsed),
//                       ProtocolInfo,
//                       sizeof(*ProtocolInfo));
//
//         /* Increase the count */
//         Context->Count++;
//     }
// }
//
// BOOL
// __stdcall
// ProtocolEnumerationProc(PVOID EnumContext,
//                         PTCATALOG_ENTRY Entry)
// {
//     PPROTOCOL_ENUM_CONTEXT Context = (PPROTOCOL_ENUM_CONTEXT)EnumContext;
//     LPWSAPROTOCOL_INFOW ProtocolInfo = &Entry->ProtocolInfo;
//
//     /* Check if this protocol matches */
//     if (CheckProtocolMatch(Context->Protocols, ProtocolInfo))
//     {
//         /* Copy the information */
//         ProtocolInfoFromContext(ProtocolInfo, Context);
//         Context->BufferUsed += sizeof(*ProtocolInfo);
//     }
//
//     /* Continue enumeration */
//     return TRUE;
// }
//
// BOOL
// __stdcall
// ProviderEnumerationProc(PVOID EnumContext,
//                         PNSCATALOG_ENTRY Entry)
// {
//     INT PathLen;
//     PPROVIDER_ENUM_CONTEXT Context = (PPROVIDER_ENUM_CONTEXT)EnumContext;
//
//     /* Check if this provider matches */
//     if (IsEqualGUID(&Entry->ProviderId, &Context->ProviderId))
//     {
//         /* Get the information about the provider */
//         PathLen = wcslen(Entry->DllPath) + 1;
//         Context->FoundPathLen = PathLen;
//         Context->Found = 1;
//
//         /* If we have enough room, copy path */
//         if (PathLen <= Context->ProviderDllPathLen)
//         {
//             wcscpy(Context->ProviderDllPath, Entry->DllPath);
//         }
//
//         /* Stop enumeration */
//         return FALSE;
//     }
//     else
//     {
//         /* Continue enumeration */
//         return TRUE;
//     }
//
// }
//
// PTCATALOG
// __stdcall
// OpenInitializedCatalog(VOID)
// {
//     PTCATALOG Catalog;
//     HKEY WsKey;
//
//     /* Allocate the catalog */
//     Catalog = WsTcAllocate();
//     if (Catalog)
//     {
//         /* Open the WS Key */
//         WsKey = WsOpenRegistryRoot();
//
//         /* Initialize the catalog */
//         WsTcInitializeFromRegistry(Catalog, WsKey, NULL);
//
//         /* Close the key */
//         RegCloseKey(WsKey);
//     }
//
//     /* Return it */
//     return Catalog;
// }
//
// /*
//  * @implemented
//  */
// INT
// __stdcall
// WSCEnumProtocols(IN LPINT lpiProtocols,
//                  OUT LPWSAPROTOCOL_INFOW lpProtocolBuffer,
//                  IN OUT LPDWORD lpdwBufferLength,
//                  OUT LPINT lpErrno)
// {
//     INT Status;
//     PTCATALOG Catalog;
//     PROTOCOL_ENUM_CONTEXT Context;
//     DPRINT("WSCEnumProtocols: %p\n", lpiProtocols);
//
//     /* Create a catalog object from the current one */
//     Catalog = OpenInitializedCatalog();
//     if (!Catalog)
//     {
//         /* Fail if we couldn't */
//         *lpErrno = WSAENOBUFS;
//         return SOCKET_ERROR;
//     }
//
//     /* Setup the context */
//     Context.Protocols = lpiProtocols;
//     Context.ProtocolBuffer = lpProtocolBuffer;
//     Context.BufferLength = lpProtocolBuffer ? *lpdwBufferLength : 0;
//     Context.BufferUsed = 0;
//     Context.Count = 0;
//     Context.ErrorCode = ERROR_SUCCESS;
//
//     /* Enumerate the catalog */
//     WsTcEnumerateCatalogItems(Catalog, ProtocolEnumerationProc, &Context);
//
//     /* Get status */
//     Status = Context.Count;
//
//     /* Check the error code */
//     if (Context.ErrorCode == ERROR_SUCCESS)
//     {
//         /* Check if enough space was available */
//         if (Context.BufferLength < Context.BufferUsed)
//         {
//             /* Fail and tell them how much we need */
//             *lpdwBufferLength = Context.BufferUsed;
//             *lpErrno = WSAENOBUFS;
//             Status = SOCKET_ERROR;
//         }
//     }
//     else
//     {
//         /* Failure, normalize error */
//         Status = SOCKET_ERROR;
//         *lpErrno = Context.ErrorCode;
//     }
//
//     /* Delete the catalog object */
//     WsTcDelete(Catalog);
//
//     /* Return */
//     return Status;
// }
//
// /*
//  * @implemented
//  */
// INT
// __stdcall
// WSAEnumProtocolsA(IN LPINT lpiProtocols,
//                   OUT LPWSAPROTOCOL_INFOA lpProtocolBuffer,
//                   IN OUT LPDWORD lpdwBufferLength)
// {
//     INT error, i, count;
//     LPWSAPROTOCOL_INFOW protocolInfoW;
//     DWORD size;
//     DPRINT("WSAEnumProtocolsA: %p %p %p\n", lpiProtocols, lpProtocolBuffer, lpdwBufferLength);
//     if (!lpdwBufferLength)
//     {
//         SetLastError(WSAENOBUFS);
//         return SOCKET_ERROR;
//     }
//     count = WSCEnumProtocols(lpiProtocols, NULL, &size, &error);
//     if (!lpProtocolBuffer || *lpdwBufferLength < (size/sizeof(WSAPROTOCOL_INFOW))*sizeof(WSAPROTOCOL_INFOA))
//     {
//         *lpdwBufferLength = (size/sizeof(WSAPROTOCOL_INFOW))*sizeof(WSAPROTOCOL_INFOA);
//         SetLastError(WSAENOBUFS);
//         return SOCKET_ERROR;
//     }
//     protocolInfoW = HeapAlloc(WsSockHeap, 0, size);
//     count = WSCEnumProtocols(lpiProtocols, protocolInfoW, &size, &error);
//     if (SOCKET_ERROR == count)
//     {
//         HeapFree(WsSockHeap, 0, protocolInfoW);
//         SetLastError(error);
//         return SOCKET_ERROR;
//     }
//     *lpdwBufferLength = 0;
//     for (i = 0; i < count; i++)
//     {
//         /* Copy the data */
//         RtlMoveMemory(&lpProtocolBuffer[i],
//                       &protocolInfoW[i],
//                       sizeof(lpProtocolBuffer[0])-sizeof(lpProtocolBuffer[0].szProtocol));
//         wcstombs(lpProtocolBuffer[i].szProtocol, protocolInfoW[i].szProtocol, sizeof(lpProtocolBuffer[0].szProtocol));
//         *lpdwBufferLength += sizeof(WSAPROTOCOL_INFOA);
//     }
//     HeapFree(WsSockHeap, 0, protocolInfoW);
//     return i;
// }
//
// /*
//  * @implemented
//  */
// INT
// __stdcall
// WSAEnumProtocolsW(IN LPINT lpiProtocols,
//                   OUT LPWSAPROTOCOL_INFOW lpProtocolBuffer,
//                   IN OUT  LPDWORD lpdwBufferLength)
// {
//     INT error, count;
//     DPRINT("WSAEnumProtocolsW: %p %p %p\n", lpiProtocols, lpProtocolBuffer, lpdwBufferLength);
//     count = WSCEnumProtocols(lpiProtocols, lpProtocolBuffer, lpdwBufferLength, &error);
//     if (SOCKET_ERROR == count)
//     {
//         SetLastError(error);
//         return SOCKET_ERROR;
//     }
//     return count;
// }
//
//
// /*
//  * @implemented
//  */
// INT
// __stdcall
// WPUGetProviderPath(IN LPGUID lpProviderId,
//                    OUT LPWSTR lpszProviderDllPath,
//                    IN OUT LPINT lpProviderDllPathLen,
//                    OUT LPINT lpErrno)
// {
//     return WSCGetProviderPath(lpProviderId, lpszProviderDllPath, lpProviderDllPathLen, lpErrno);
// }
//
// /*
//  * @implemented
//  */
// INT
// __stdcall
// WSAProviderConfigChange(IN OUT LPHANDLE lpNotificationHandle,
//                         IN LPWSAOVERLAPPED lpOverlapped,
//                         IN LPWSAOVERLAPPED_COMPLETION_ROUTINE lpCompletionRoutine)
// {
//     DPRINT("WSAProviderConfigChange: %p\n", lpNotificationHandle);
//     UNIMPLEMENTED;
//     SetLastError(WSAEINVAL);
//     return SOCKET_ERROR;
// }
//
// /*
//  * @implemented
//  */
// INT
// __stdcall
// WSCGetProviderPath(IN LPGUID lpProviderId,
//                    OUT LPWSTR lpszProviderDllPath,
//                    IN OUT LPINT lpProviderDllPathLen,
//                    OUT LPINT lpErrno)
// {
//     PWSTHREAD Thread;
//     PWSPROCESS Process;
//     PNSCATALOG Catalog;
//     INT ErrorCode, PathLen;
//     PROVIDER_ENUM_CONTEXT Context;
//
//     DPRINT("WSCGetProviderPath: %p %p %p %p\n", lpProviderId, lpszProviderDllPath, lpProviderDllPathLen, lpErrno);
//
//     /* Enter prolog */
//     if ((ErrorCode = WsApiProlog(&Process, &Thread)) != ERROR_SUCCESS)
//     {
//         /* FIXME: if WSANOTINITIALISED, we should init
//          * and perform the search!
//          */
//
//         /* Leave now */
//         *lpErrno = ErrorCode;
//         return SOCKET_ERROR;
//     }
//
//     /* Get the catalog */
//     Catalog = WsProcGetNsCatalog(Process);
//
//     _SEH2_TRY
//     {
//         /* Setup the context */
//         Context.ProviderId = *lpProviderId;
//         Context.ProviderDllPath = lpszProviderDllPath;
//         Context.ProviderDllPathLen = *lpProviderDllPathLen;
//         Context.FoundPathLen = 0;
//         Context.Found = 0;
//         Context.ErrorCode = ERROR_SUCCESS;
//
//         ErrorCode = ERROR_SUCCESS;
//
//         /* Enumerate the catalog */
//         WsNcEnumerateCatalogItems(Catalog, Some(ProviderEnumerationProc), &Context);
//
//         /* Check the error code */
//         if (Context.ErrorCode == ERROR_SUCCESS)
//         {
//             /* Check if provider was found */
//             if (Context.Found)
//             {
//                 PathLen = Context.FoundPathLen;
//
//                 /* Check whether buffer is too small
//                  * If it isn't, return length without null char
//                  * (see ProviderEnumerationProc)
//                  */
//                 if (Context.FoundPathLen <= *lpProviderDllPathLen)
//                 {
//                     PathLen = Context.FoundPathLen - 1;
//                 }
//                 else
//                 {
//                     ErrorCode = WSAEFAULT;
//                 }
//
//                 /* Set returned/required length */
//                 *lpProviderDllPathLen = PathLen;
//             }
//             else
//             {
//                 ErrorCode = WSAEINVAL;
//             }
//         }
//         else
//         {
//             ErrorCode = Context.ErrorCode;
//         }
//     }
//     _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//     {
//         ErrorCode = WSAEFAULT;
//     }
//     _SEH2_END;
//
//     /* Do we have to return failure? */
//     if (ErrorCode != ERROR_SUCCESS)
//     {
//         *lpErrno = ErrorCode;
//         return SOCKET_ERROR;
//     }
//
//     return 0;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
