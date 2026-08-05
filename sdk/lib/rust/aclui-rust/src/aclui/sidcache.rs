use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
#[repr(C)]
pub struct _SIDCACHEMGR {
    pub RefCount: i32,
    pub LsaHandle: LSA_HANDLE,
    pub Lock: CRITICAL_SECTION,
    pub QueueListHead: _LIST_ENTRY,
    pub QueueLookingUp: *mut _SIDQUEUEENTRY,
    pub CacheListHead: _LIST_ENTRY,
    pub Heap: HANDLE,
    pub LookupEvent: HANDLE,
    pub LookupThread: HANDLE,
    pub SystemName: [u16; 1],
}

pub type PSIDCACHEMGR = *mut _SIDCACHEMGR;

#[repr(C)]
pub struct _SIDCACHECALLBACKINFO {
    pub CompletionProc: PSIDREQCOMPLETIONPROC,
    pub Context: PVOID,
}

pub type PSIDCACHECALLBACKINFO = *mut _SIDCACHECALLBACKINFO;

#[repr(C)]
pub struct _SIDQUEUEENTRY {
    pub ListEntry: _LIST_ENTRY,
    pub CallbackCount: u32,
    pub Callbacks: PSIDCACHECALLBACKINFO,
}

pub type PSIDQUEUEENTRY = *mut _SIDQUEUEENTRY;

#[repr(C)]
pub struct _SIDCACHEENTRY {
    pub ListEntry: _LIST_ENTRY,
    pub SidNameUse: SID_NAME_USE,
    pub AccountName: PWSTR,
    pub DomainName: PWSTR,
}

pub type PSIDCACHEENTRY = *mut _SIDCACHEENTRY;

// The un-underscored halves of sidcache.c's four `typedef struct _X {...} X, *PX;` pairs. The
// translation emitted `_X` and `PX` for each and dropped `X`, which is the spelling the other
// TUs use.
pub type SIDCACHEMGR = _SIDCACHEMGR;
pub type SIDCACHECALLBACKINFO = _SIDCACHECALLBACKINFO;
pub type SIDQUEUEENTRY = _SIDQUEUEENTRY;
pub type SIDCACHEENTRY = _SIDCACHEENTRY;

unsafe extern "C" fn FreeQueueEntry(
    scm: PSIDCACHEMGR,
    QueueEntry: PSIDQUEUEENTRY,
) {
    if (*QueueEntry).ListEntry.Flink != core::ptr::null_mut() {
        RemoveEntryList(&mut (*QueueEntry).ListEntry);
    }

    HeapFree(
        (*scm).Heap,
        0,
        (*QueueEntry).Callbacks as LPVOID,
    );

    HeapFree(
        (*scm).Heap,
        0,
        QueueEntry as LPVOID,
    );
}
unsafe extern "C" fn FreeCacheEntry(
    scm: PSIDCACHEMGR,
    CacheEntry: PSIDCACHEENTRY,
) {
    RemoveEntryList(core::ptr::addr_of_mut!((*CacheEntry).ListEntry));

    HeapFree(
        (*scm).Heap,
        0,
        CacheEntry as LPVOID,
    );
}
unsafe extern "C" fn CleanupSidCacheMgr(scm: PSIDCACHEMGR) {
    LsaClose((*scm).LsaHandle);
    CloseHandle((*scm).LookupEvent);
    CloseHandle((*scm).LookupThread);

    while IsListEmpty(core::ptr::addr_of_mut!((*scm).QueueListHead)) == 0 {
        let QueueEntry: PSIDQUEUEENTRY;

        QueueEntry = (((*scm).QueueListHead.Flink as *mut u8).offset(-(core::mem::offset_of!(SIDQUEUEENTRY, ListEntry) as isize))) as PSIDQUEUEENTRY;
        FreeQueueEntry(scm, QueueEntry);
    }

    while IsListEmpty(core::ptr::addr_of_mut!((*scm).CacheListHead)) == 0 {
        let CacheEntry: PSIDCACHEENTRY;

        CacheEntry = (((*scm).CacheListHead.Flink as *mut u8).offset(-(core::mem::offset_of!(SIDCACHEENTRY, ListEntry) as isize))) as PSIDCACHEENTRY;
        FreeCacheEntry(scm, CacheEntry);
    }

    DeleteCriticalSection(core::ptr::addr_of_mut!((*scm).Lock));
}
unsafe extern "C" fn ReferenceSidCacheMgr(SidCacheMgr: HANDLE) -> PSIDCACHEMGR {
    let scm = HandleToScm!(SidCacheMgr);

    if InterlockedIncrement!(core::ptr::addr_of_mut!((*scm).RefCount)) != 1 {
        return scm;
    }

    return core::ptr::null_mut();
}
unsafe extern "C" fn DereferenceSidCacheMgr(scm: PSIDCACHEMGR) {
    if InterlockedDecrement!(core::ptr::addr_of_mut!((*scm).RefCount)) == 0 {
        SetEvent((*scm).LookupEvent);
    }
}
unsafe extern "C" fn OpenLSAPolicyHandle(
    SystemName: LPWSTR,
    DesiredAccess: ACCESS_MASK,
    PolicyHandle: *mut LSA_HANDLE,
) -> BOOL {
    let mut LsaObjectAttributes: LSA_OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut LsaSystemName: LSA_UNICODE_STRING = core::mem::zeroed();
    let psn: *mut LSA_UNICODE_STRING;
    let Status: NTSTATUS;
    let NameLength: usize;

    if !SystemName.is_null() && *SystemName.offset(0) != 0 {
        NameLength = wcslen(SystemName);
        if NameLength > (UNICODE_STRING_MAX_CHARS) as usize {
            return (FALSE) as i32;
        }

        LsaSystemName.Buffer = SystemName;
        LsaSystemName.Length = (NameLength * core::mem::size_of::<WCHAR>()) as u16;
        LsaSystemName.MaximumLength = LsaSystemName.Length + core::mem::size_of::<WCHAR>() as u16;
        psn = &mut LsaSystemName;
    } else {
        psn = core::ptr::null_mut();
    }

    Status = LsaOpenPolicy(
        psn,
        &mut LsaObjectAttributes,
        DesiredAccess,
        PolicyHandle,
    );
    if !NT_SUCCESS(Status) {
        SetLastError(LsaNtStatusToWinError(Status));
        return (FALSE) as i32;
    }

    return (TRUE) as i32;
}
unsafe extern "C" fn LookupSidInformation(
    scm: PSIDCACHEMGR,
    pSid: PSID,
    ReqResult: *mut PSIDREQRESULT,
) -> BOOL {
    let mut ReferencedDomain: PLSA_REFERENCED_DOMAIN_LIST = core::ptr::null_mut();
    let mut Names: PLSA_TRANSLATED_NAME = core::ptr::null_mut();
    let mut Domain: PLSA_TRUST_INFORMATION;
    let mut DomainName: PLSA_UNICODE_STRING;
    let mut SidNameUse: SID_NAME_USE = SidTypeUnknown;
    let mut PolicyAccountDomainInfo: PPOLICY_ACCOUNT_DOMAIN_INFO = core::ptr::null_mut();
    let Status: NTSTATUS;
    let mut AccountNameSize: DWORD;
    let mut DomainNameSize: DWORD = 0;
    let mut ReqRet: PSIDREQRESULT = core::ptr::null_mut();
    let mut Ret: BOOL = (FALSE) as i32;

    Status = LsaLookupSids(
        (*scm).LsaHandle,
        1,
        &pSid,
        &mut ReferencedDomain,
        &mut Names,
    );
    if NT_SUCCESS(Status) {
        SidNameUse = (*Names).Use;

        if !ReferencedDomain.is_null() && (*Names).DomainIndex >= 0 {
            Domain = (*ReferencedDomain).Domains.add((*Names).DomainIndex as usize);
            DomainName = &mut (*Domain).Name;
        } else {
            Domain = core::ptr::null_mut();
            DomainName = core::ptr::null_mut();
        }

        match SidNameUse {
            SidTypeAlias => {
                if !Domain.is_null() {
                    let status_query = LsaQueryInformationPolicy(
                        (*scm).LsaHandle,
                        PolicyAccountDomainInformation,
                        &mut PolicyAccountDomainInfo as *mut _ as *mut PVOID,
                    );
                    if NT_SUCCESS(status_query) {
                        DomainName = &mut (*PolicyAccountDomainInfo).DomainName;

                        SidNameUse = if !PolicyAccountDomainInfo.is_null() {
                            SidTypeGroup
                        } else {
                            SidTypeUser
                        };
                    }
                }
            }
            _ => {
                // DPRINT removed
            }
        }

        AccountNameSize = (*Names).Name.Length as DWORD;
        if !DomainName.is_null() {
            DomainNameSize = (*DomainName).Length as DWORD;
        }

        ReqRet = (HeapAlloc(
            (*scm).Heap,
            0,
            core::mem::size_of::<SIDREQRESULT>()
                + (((AccountNameSize + DomainNameSize) + 2) as usize
                    * core::mem::size_of::<WCHAR>()),
        )).cast();
        if !ReqRet.is_null() {
            (*ReqRet).RefCount = 1;
            (*ReqRet).AccountName = (ReqRet.offset(1)) as LPWSTR;
            (*ReqRet).DomainName = (*ReqRet)
                .AccountName
                .offset((AccountNameSize / core::mem::size_of::<WCHAR>() as u32) as isize)
                .offset(1);

            CopyMemory!(
                (*ReqRet).AccountName as PVOID,
                (*Names).Name.Buffer as PVOID,
                (*Names).Name.Length as usize,
            );

            if !DomainName.is_null() {
                CopyMemory!(
                    (*ReqRet).DomainName as PVOID,
                    (*DomainName).Buffer as PVOID,
                    (*DomainName).Length as usize,
                );
            }

            *(*ReqRet)
                .AccountName
                .offset((AccountNameSize / core::mem::size_of::<WCHAR>() as u32) as isize) =
                0u16;
            *(*ReqRet)
                .DomainName
                .offset((DomainNameSize / core::mem::size_of::<WCHAR>() as u32) as isize) =
                0u16;

            (*ReqRet).SidNameUse = SidNameUse;
        }

        if !PolicyAccountDomainInfo.is_null() {
            LsaFreeMemory(PolicyAccountDomainInfo as PVOID);
        }

        LsaFreeMemory(ReferencedDomain as PVOID);
        LsaFreeMemory(Names as PVOID);

        Ret = (TRUE) as i32;
    } else if Status == STATUS_NONE_MAPPED {
        Ret = (TRUE) as i32;
    }

    if Ret != 0 {
        *ReqResult = ReqRet;
    }

    Ret
}
unsafe extern "C" fn FindSidInCache(
    scm: PSIDCACHEMGR,
    pSid: PSID,
    ReqResult: *mut PSIDREQRESULT,
) -> BOOL {
    let mut CacheEntry: PSIDCACHEENTRY;
    let mut CurrentEntry: PLIST_ENTRY;
    let mut ReqRes: PSIDREQRESULT;
    let mut Ret: BOOL = (FALSE) as i32;

    CurrentEntry = (*scm).CacheListHead.Flink;
    while CurrentEntry != core::ptr::addr_of_mut!((*scm).CacheListHead) {
        CacheEntry = (CurrentEntry as *mut u8).offset(
            -(core::mem::offset_of!(SIDCACHEENTRY, ListEntry) as isize)
        ) as PSIDCACHEENTRY;

        if EqualSid(
            pSid,
            CacheEntry.offset(1) as PSID,
        ) != 0
        {
            let mut ReqResultSize: usize;
            let mut AccountNameLen: u32;
            let mut DomainNameLen: u32;

            Ret = (TRUE) as i32;

            AccountNameLen = (wcslen((*CacheEntry).AccountName)) as u32;
            DomainNameLen = (wcslen((*CacheEntry).DomainName)) as u32;

            ReqResultSize = core::mem::size_of::<SIDREQRESULT>()
                + (((AccountNameLen + 1)
                    + (DomainNameLen + 1)) as usize
                    * core::mem::size_of::<u16>());

            ReqRes = (HeapAlloc((*scm).Heap, 0, ReqResultSize as SIZE_T)).cast();
            if !ReqRes.is_null() {
                let mut Buffer: PWSTR = ReqRes.offset(1) as PWSTR;

                (*ReqRes).RefCount = 1;

                (*ReqRes).AccountName = Buffer;
                wcscpy((*ReqRes).AccountName, (*CacheEntry).AccountName);
                Buffer = Buffer.offset((AccountNameLen + 1) as isize);

                (*ReqRes).DomainName = Buffer;
                wcscpy((*ReqRes).DomainName, (*CacheEntry).DomainName);
            }

            *ReqResult = ReqRes;
            break;
        }

        CurrentEntry = (*CurrentEntry).Flink;
    }

    Ret
}
unsafe extern "C" fn CacheLookupResults(
    scm: PSIDCACHEMGR,
    pSid: PSID,
    ReqResult: PSIDREQRESULT,
) {
    let mut CacheEntry: PSIDCACHEENTRY;
    let SidLen: u32;
    let mut AccountNameLen: usize = 0;
    let mut DomainNameLen: usize = 0;
    let mut CacheEntrySize: usize = core::mem::size_of::<SIDCACHEENTRY>();

    SidLen = GetLengthSid(pSid);
    CacheEntrySize += SidLen as usize;

    AccountNameLen = wcslen((*ReqResult).AccountName);
    CacheEntrySize += (AccountNameLen + 1) * core::mem::size_of::<u16>();

    DomainNameLen = wcslen((*ReqResult).DomainName);
    CacheEntrySize += (wcslen((*ReqResult).DomainName) + 1) * core::mem::size_of::<u16>();

    CacheEntry = (HeapAlloc(
        (*scm).Heap,
        0,
        CacheEntrySize as SIZE_T,
    )).cast();
    if CacheEntry != core::ptr::null_mut() {
        let mut lpBuf: PWSTR = ((CacheEntry.offset(1) as usize) + SidLen as usize) as PWSTR;

        (*CacheEntry).SidNameUse = (*ReqResult).SidNameUse;

        CopySid(
            SidLen,
            CacheEntry.offset(1) as PSID,
            pSid,
        );

        (*CacheEntry).AccountName = lpBuf;
        wcscpy(
            lpBuf,
            (*ReqResult).AccountName,
        );
        lpBuf = lpBuf.offset((AccountNameLen + 1) as isize);

        (*CacheEntry).DomainName = lpBuf;
        wcscpy(
            lpBuf,
            (*ReqResult).DomainName,
        );
        lpBuf = lpBuf.offset((DomainNameLen + 1) as isize);

        InsertTailList(
            core::ptr::addr_of_mut!((*scm).CacheListHead),
            core::ptr::addr_of_mut!((*CacheEntry).ListEntry),
        );
    }
}
#[cfg(target_arch = "x86")]
unsafe extern "stdcall" fn LookupThreadProc(lpParameter: LPVOID) -> u32 {
    let mut hModule: HMODULE = core::ptr::null_mut();
    let scm: PSIDCACHEMGR = lpParameter as PSIDCACHEMGR;

    if GetModuleHandleExW(
        GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS,
        hDllInstance as LPCWSTR,
        &mut hModule,
    ) == 0
    {
        hModule = core::ptr::null_mut();
    }

    while (*scm).RefCount != 0 {
        let mut QueueEntry: PSIDQUEUEENTRY = core::ptr::null_mut();

        EnterCriticalSection(&mut (*scm).Lock);

        if (*scm).QueueListHead.Flink != &mut (*scm).QueueListHead as *mut _ {
            QueueEntry = ((*scm).QueueListHead.Flink as *const u8).offset(
                -(core::mem::offset_of!(SIDQUEUEENTRY, ListEntry) as isize)
            ) as PSIDQUEUEENTRY;
            RemoveEntryList(&mut (*QueueEntry).ListEntry);
            (*QueueEntry).ListEntry.Flink = core::ptr::null_mut();
        } else {
            LeaveCriticalSection(&mut (*scm).Lock);

            WaitForSingleObject((*scm).LookupEvent, INFINITE);
            continue;
        }

        (*scm).QueueLookingUp = QueueEntry;

        LeaveCriticalSection(&mut (*scm).Lock);

        if QueueEntry != core::ptr::null_mut() {
            let mut ReqResult: PSIDREQRESULT = core::ptr::null_mut();
            let mut FoundReqResult: PSIDREQRESULT = core::ptr::null_mut();
            let pSid: PSID = QueueEntry.offset(1) as PSID;

            if LookupSidInformation(scm, pSid, &mut ReqResult) == 0 {
                ReqResult = core::ptr::null_mut();
            }

            EnterCriticalSection(&mut (*scm).Lock);

            if FindSidInCache(scm, pSid, &mut FoundReqResult) == 0 {
                if ReqResult != core::ptr::null_mut() {
                    CacheLookupResults(scm, pSid, ReqResult);
                }
            } else {
                if ReqResult != core::ptr::null_mut() {
                    DereferenceSidReqResult((scm).cast(), ReqResult);
                }

                ReqResult = FoundReqResult;
            }

            if (*scm).QueueLookingUp != core::ptr::null_mut() {
                let mut i: u32 = 0;

                while (*scm).QueueLookingUp != core::ptr::null_mut()
                    && i < (*QueueEntry).CallbackCount
                {
                    let Context: PVOID = (*(*QueueEntry).Callbacks.offset(i as isize)).Context;
                    let CompletionProc: PSIDREQCOMPLETIONPROC =
                        (*(*QueueEntry).Callbacks.offset(i as isize)).CompletionProc;

                    LeaveCriticalSection(&mut (*scm).Lock);

                    CompletionProc.unwrap_unchecked()(ScmToHandle!(scm), pSid, ReqResult, Context);

                    EnterCriticalSection(&mut (*scm).Lock);

                    i += 1;
                }

                (*scm).QueueLookingUp = core::ptr::null_mut();
            }

            LeaveCriticalSection(&mut (*scm).Lock);

            FreeQueueEntry(scm, QueueEntry);
        }
    }

    CleanupSidCacheMgr(scm);

    HeapFree((*scm).Heap, 0, scm as LPVOID);

    if hModule != core::ptr::null_mut() {
        FreeLibraryAndExitThread(hModule, 0);
    }

    0
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn LookupThreadProc(lpParameter: LPVOID) -> u32 {
    let mut hModule: HMODULE = core::ptr::null_mut();
    let scm: PSIDCACHEMGR = lpParameter as PSIDCACHEMGR;

    if GetModuleHandleExW(
        (GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS) as u32,
        hDllInstance as LPCWSTR,
        &mut hModule,
    ) == 0
    {
        hModule = core::ptr::null_mut();
    }

    while (*scm).RefCount != 0 {
        let mut QueueEntry: PSIDQUEUEENTRY = core::ptr::null_mut();

        EnterCriticalSection(&mut (*scm).Lock);

        if (*scm).QueueListHead.Flink != &mut (*scm).QueueListHead as *mut _ {
            QueueEntry = ((*scm).QueueListHead.Flink as *const u8).offset(
                -(core::mem::offset_of!(SIDQUEUEENTRY, ListEntry) as isize)
            ) as PSIDQUEUEENTRY;
            RemoveEntryList(&mut (*QueueEntry).ListEntry);
            (*QueueEntry).ListEntry.Flink = core::ptr::null_mut();
        } else {
            LeaveCriticalSection(&mut (*scm).Lock);

            WaitForSingleObject((*scm).LookupEvent, (INFINITE) as u32);
            continue;
        }

        (*scm).QueueLookingUp = QueueEntry;

        LeaveCriticalSection(&mut (*scm).Lock);

        if QueueEntry != core::ptr::null_mut() {
            let mut ReqResult: PSIDREQRESULT = core::ptr::null_mut();
            let mut FoundReqResult: PSIDREQRESULT = core::ptr::null_mut();
            let pSid: PSID = QueueEntry.offset(1) as PSID;

            if LookupSidInformation(scm, pSid, &mut ReqResult) == 0 {
                ReqResult = core::ptr::null_mut();
            }

            EnterCriticalSection(&mut (*scm).Lock);

            if FindSidInCache(scm, pSid, &mut FoundReqResult) == 0 {
                if ReqResult != core::ptr::null_mut() {
                    CacheLookupResults(scm, pSid, ReqResult);
                }
            } else {
                if ReqResult != core::ptr::null_mut() {
                    DereferenceSidReqResult((scm).cast(), ReqResult);
                }

                ReqResult = FoundReqResult;
            }

            if (*scm).QueueLookingUp != core::ptr::null_mut() {
                let mut i: u32 = 0;

                while (*scm).QueueLookingUp != core::ptr::null_mut()
                    && i < (*QueueEntry).CallbackCount
                {
                    let Context: PVOID = (*(*QueueEntry).Callbacks.offset(i as isize)).Context;
                    let CompletionProc: PSIDREQCOMPLETIONPROC =
                        (*(*QueueEntry).Callbacks.offset(i as isize)).CompletionProc;

                    LeaveCriticalSection(&mut (*scm).Lock);

                    CompletionProc.unwrap_unchecked()(ScmToHandle!(scm), pSid, ReqResult, Context);

                    EnterCriticalSection(&mut (*scm).Lock);

                    i += 1;
                }

                (*scm).QueueLookingUp = core::ptr::null_mut();
            }

            LeaveCriticalSection(&mut (*scm).Lock);

            FreeQueueEntry(scm, QueueEntry);
        }
    }

    CleanupSidCacheMgr(scm);

    HeapFree((*scm).Heap, 0, scm as LPVOID);

    if hModule != core::ptr::null_mut() {
        FreeLibraryAndExitThread(hModule, 0);
    }

    0
}
#[no_mangle]
pub unsafe extern "C" fn CreateSidCacheMgr(
    Heap: HANDLE,
    SystemName: LPCWSTR,
) -> HANDLE {
    let mut scm: PSIDCACHEMGR;

    let SystemName = if SystemName.is_null() {
        w!("") as LPCWSTR
    } else {
        SystemName
    };

    scm = HeapAlloc(
        Heap,
        0,
        FIELD_OFFSET!(
            SIDCACHEMGR,
            SystemName[wcslen(SystemName) + 1]
        ),
    ) as PSIDCACHEMGR;

    if !scm.is_null() {
        ZeroMemory!(
            scm as LPVOID,
            FIELD_OFFSET!(SIDCACHEMGR, SystemName),
        );

        (*scm).RefCount = 1;
        (*scm).Heap = Heap;

        wcscpy(
            (*scm).SystemName.as_mut_ptr(),
            SystemName,
        );

        InitializeCriticalSection(core::ptr::addr_of_mut!((*scm).Lock));
        InitializeListHead(core::ptr::addr_of_mut!((*scm).QueueListHead));
        InitializeListHead(core::ptr::addr_of_mut!((*scm).CacheListHead));

        (*scm).LookupEvent = CreateEvent(
            core::ptr::null_mut(),
            (FALSE) as i32,
            (FALSE) as i32,
            core::ptr::null_mut(),
        );

        if (*scm).LookupEvent.is_null() {
            'Cleanup: loop {
                if !(*scm).LookupEvent.is_null() {
                    CloseHandle((*scm).LookupEvent);
                }

                if !(*scm).LsaHandle.is_null() {
                    LsaClose((*scm).LsaHandle);
                }

                HeapFree(
                    Heap,
                    0,
                    scm as LPVOID,
                );
                scm = core::ptr::null_mut();
                break 'Cleanup;
            }
        } else {
            if OpenLSAPolicyHandle(
                (*scm).SystemName.as_mut_ptr(),
                (POLICY_LOOKUP_NAMES | POLICY_VIEW_LOCAL_INFORMATION) as u32,
                core::ptr::addr_of_mut!((*scm).LsaHandle),
            ) == 0 {
                'Cleanup: loop {
                    if !(*scm).LookupEvent.is_null() {
                        CloseHandle((*scm).LookupEvent);
                    }

                    if !(*scm).LsaHandle.is_null() {
                        LsaClose((*scm).LsaHandle);
                    }

                    HeapFree(
                        Heap,
                        0,
                        scm as LPVOID,
                    );
                    scm = core::ptr::null_mut();
                    break 'Cleanup;
                }
            } else {
                (*scm).LookupThread = CreateThread(
                    core::ptr::null_mut(),
                    0,
                    LookupThreadProc as LPVOID,
                    scm as LPVOID,
                    0,
                    core::ptr::null_mut(),
                );

                if (*scm).LookupThread.is_null() {
                    'Cleanup: loop {
                        if !(*scm).LookupEvent.is_null() {
                            CloseHandle((*scm).LookupEvent);
                        }

                        if !(*scm).LsaHandle.is_null() {
                            LsaClose((*scm).LsaHandle);
                        }

                        HeapFree(
                            Heap,
                            0,
                            scm as LPVOID,
                        );
                        scm = core::ptr::null_mut();
                        break 'Cleanup;
                    }
                }
            }
        }
    } else {
        SetLastError(ERROR_NOT_ENOUGH_MEMORY);
    }

    scm as HANDLE
}
#[no_mangle]
pub unsafe extern "C" fn DestroySidCacheMgr(SidCacheMgr: HANDLE) {
    let scm = HandleToScm!(SidCacheMgr);

    if !scm.is_null() {
        DereferenceSidCacheMgr(scm);
    }
}
unsafe extern "C" fn QueueSidLookup(
    scm: *mut SIDCACHEMGR,
    pSid: PSID,
    CompletionProc: PSIDREQCOMPLETIONPROC,
    Context: PVOID,
) -> i32 {
    let mut CurrentEntry: PLIST_ENTRY;
    let mut QueueEntry: *mut SIDQUEUEENTRY;
    let mut FoundEntry: *mut SIDQUEUEENTRY = core::ptr::null_mut();
    let mut Ret: i32 = (FALSE) as i32;

    if (*scm).QueueLookingUp != core::ptr::null_mut()
        && EqualSid(
            pSid,
            (*scm).QueueLookingUp.offset(1) as PSID,
        ) != 0
    {
        FoundEntry = (*scm).QueueLookingUp;
    } else {
        CurrentEntry = (*scm).QueueListHead.Flink;
        while CurrentEntry != core::ptr::addr_of_mut!((*scm).QueueListHead) {
            QueueEntry = (CurrentEntry as *mut u8).offset(
                -(core::mem::offset_of!(SIDQUEUEENTRY, ListEntry) as isize)
            ) as *mut SIDQUEUEENTRY;

            if EqualSid(
                pSid,
                QueueEntry.offset(1) as PSID,
            ) != 0
            {
                FoundEntry = QueueEntry;
                break;
            }
            CurrentEntry = (*CurrentEntry).Flink;
        }
    }

    if FoundEntry == core::ptr::null_mut() {
        let SidLength: u32 = GetLengthSid(pSid);

        FoundEntry = HeapAlloc(
            (*scm).Heap,
            0,
            (core::mem::size_of::<SIDQUEUEENTRY>() as u32).wrapping_add(SidLength) as usize as SIZE_T,
        ) as *mut SIDQUEUEENTRY;
        if FoundEntry != core::ptr::null_mut() {
            CopySid(
                SidLength,
                FoundEntry.offset(1) as PSID,
                pSid,
            );

            (*FoundEntry).CallbackCount = 1;
            (*FoundEntry).Callbacks = HeapAlloc(
                (*scm).Heap,
                0,
                core::mem::size_of::<SIDCACHECALLBACKINFO>() as SIZE_T,
            ) as *mut SIDCACHECALLBACKINFO;

            if (*FoundEntry).Callbacks != core::ptr::null_mut() {
                (*(*FoundEntry).Callbacks.offset(0)).CompletionProc =
                    CompletionProc;
                (*(*FoundEntry).Callbacks.offset(0)).Context = Context;

                InsertTailList(
                    core::ptr::addr_of_mut!((*scm).QueueListHead),
                    core::ptr::addr_of_mut!((*FoundEntry).ListEntry),
                );

                SetEvent((*scm).LookupEvent);

                Ret = (TRUE) as i32;
            } else {
                HeapFree((*scm).Heap, 0, FoundEntry as LPVOID);
            }
        }
    } else {
        let Sidccb: *mut SIDCACHECALLBACKINFO = HeapReAlloc(
            (*scm).Heap,
            0,
            (*FoundEntry).Callbacks as LPVOID,
            ((*FoundEntry)
                .CallbackCount
                .wrapping_add(1) as usize)
                .wrapping_mul(core::mem::size_of::<SIDCACHECALLBACKINFO>()),
        ) as *mut SIDCACHECALLBACKINFO;
        if Sidccb != core::ptr::null_mut() {
            (*FoundEntry).Callbacks = Sidccb;
            (*(*FoundEntry)
                .Callbacks
                .offset((*FoundEntry).CallbackCount as isize))
            .CompletionProc = CompletionProc;
            let idx = (*FoundEntry).CallbackCount;
            (*FoundEntry).CallbackCount = (*FoundEntry).CallbackCount.wrapping_add(1);
            (*(*FoundEntry).Callbacks.offset(idx as isize)).Context = Context;

            Ret = (TRUE) as i32;
        }
    }

    Ret
}
#[no_mangle]
pub unsafe extern "C" fn DequeueSidLookup(
    SidCacheMgr: HANDLE,
    pSid: PSID,
) {
    let mut CurrentEntry: PLIST_ENTRY;
    let mut QueueEntry: PSIDQUEUEENTRY;
    let scm: PSIDCACHEMGR;

    scm = ReferenceSidCacheMgr(SidCacheMgr);
    if !scm.is_null() {
        EnterCriticalSection(core::ptr::addr_of_mut!((*scm).Lock));

        if !(*scm).QueueLookingUp.is_null()
            && EqualSid(
                pSid,
                (*scm).QueueLookingUp.offset(1) as PSID,
            ) != 0
        {
            (*scm).QueueLookingUp = core::ptr::null_mut();
        } else {
            CurrentEntry = (*scm).QueueListHead.Flink;
            while CurrentEntry != core::ptr::addr_of_mut!((*scm).QueueListHead) {
                QueueEntry = (CurrentEntry as *mut u8).offset(
                    -(core::mem::offset_of!(SIDQUEUEENTRY, ListEntry) as isize)
                ) as PSIDQUEUEENTRY;

                if EqualSid(
                    pSid,
                    QueueEntry.offset(1) as PSID,
                ) != 0
                {
                    FreeQueueEntry(scm, QueueEntry);
                    break;
                }
                CurrentEntry = (*CurrentEntry).Flink;
            }
        }

        LeaveCriticalSection(core::ptr::addr_of_mut!((*scm).Lock));

        DereferenceSidCacheMgr(scm);
    }
}
#[no_mangle]
pub unsafe extern "C" fn ReferenceSidReqResult(
    SidCacheMgr: HANDLE,
    ReqResult: PSIDREQRESULT,
) {
    let scm: PSIDCACHEMGR;

    scm = ReferenceSidCacheMgr(SidCacheMgr);
    if scm != core::ptr::null_mut() {
        InterlockedIncrement!(core::ptr::addr_of_mut!((*ReqResult).RefCount));

        DereferenceSidCacheMgr(scm);
    }
}
#[no_mangle]
pub unsafe extern "C" fn DereferenceSidReqResult(
    SidCacheMgr: HANDLE,
    ReqResult: PSIDREQRESULT,
) {
    let scm: PSIDCACHEMGR = ReferenceSidCacheMgr(SidCacheMgr);
    if !scm.is_null() {
        if InterlockedDecrement!(core::ptr::addr_of_mut!((*ReqResult).RefCount)) == 0 {
            HeapFree(
                (*scm).Heap,
                0,
                ReqResult as LPVOID,
            );
        }

        DereferenceSidCacheMgr(scm);
    }
}
#[no_mangle]
pub unsafe extern "C" fn LookupSidCache(
    SidCacheMgr: HANDLE,
    pSid: PSID,
    CompletionProc: PSIDREQCOMPLETIONPROC,
    Context: PVOID,
) -> BOOL {
    let mut Found: BOOL = (FALSE) as i32;
    let mut ReqResult: PSIDREQRESULT = core::ptr::null_mut();
    let scm: PSIDCACHEMGR;

    scm = ReferenceSidCacheMgr(SidCacheMgr);
    if scm != core::ptr::null_mut() {
        EnterCriticalSection(core::ptr::addr_of_mut!((*scm).Lock));

        Found = FindSidInCache(scm, pSid, core::ptr::addr_of_mut!(ReqResult));

        if Found == 0 {
            if QueueSidLookup(scm, pSid, CompletionProc, Context) == 0 {
                let mut FoundReqResult: PSIDREQRESULT = core::ptr::null_mut();

                LeaveCriticalSection(core::ptr::addr_of_mut!((*scm).Lock));

                if LookupSidInformation(scm, pSid, core::ptr::addr_of_mut!(ReqResult)) == 0 {
                    ReqResult = core::ptr::null_mut();
                }

                EnterCriticalSection(core::ptr::addr_of_mut!((*scm).Lock));

                if FindSidInCache(scm, pSid, core::ptr::addr_of_mut!(FoundReqResult)) == 0 {
                    if ReqResult != core::ptr::null_mut() {
                        CacheLookupResults(scm, pSid, ReqResult);
                    }
                } else {
                    if ReqResult != core::ptr::null_mut() {
                        DereferenceSidReqResult((scm).cast(), ReqResult);
                    }

                    ReqResult = FoundReqResult;
                }

                Found = (ReqResult != core::ptr::null_mut()) as BOOL;
            }
        }

        LeaveCriticalSection(core::ptr::addr_of_mut!((*scm).Lock));

        if Found != 0 {
            CompletionProc.unwrap_unchecked()(SidCacheMgr, pSid, ReqResult, Context);

            if ReqResult != core::ptr::null_mut() {
                HeapFree((*scm).Heap, 0, ReqResult as LPVOID);
            }
        }

        DereferenceSidCacheMgr(scm);
    }

    Found
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * ReactOS Access Control List Editor
//  * Copyright (C) 2004-2005 ReactOS Team
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
//  * Foundation, Inc., 51 Franklin Street, Fifth Floor, Boston, MA  02110-1301  USA
//  */
// /*
//  * PROJECT:         ReactOS Access Control List Editor
//  * FILE:            lib/aclui/sidcache.c
//  * PURPOSE:         Access Control List Editor
//  * PROGRAMMER:      Thomas Weidenmueller <w3seek@reactos.com>
//  *
//  * UPDATE HISTORY:
//  *      12/10/2005  Created
//  */
//
// #include "precomp.h"
//
// #include <ntsecapi.h>
//
// #define NDEBUG
// #include <debug.h>
//
// #define HandleToScm!(Handle) (PSIDCACHEMGR)(Handle)
// #define ScmToHandle!(Scm) (HANDLE)(Scm)
//
// typedef struct _SIDCACHEMGR
// {
//     volatile LONG RefCount;
//     LSA_HANDLE LsaHandle;
//     CRITICAL_SECTION Lock;
//     LIST_ENTRY QueueListHead;
//     struct _SIDQUEUEENTRY *QueueLookingUp;
//     LIST_ENTRY CacheListHead;
//     HANDLE Heap;
//     HANDLE LookupEvent;
//     HANDLE LookupThread;
//     WCHAR SystemName[1];
// } SIDCACHEMGR, *PSIDCACHEMGR;
//
//
// typedef struct _SIDCACHECALLBACKINFO
// {
//     PSIDREQCOMPLETIONPROC CompletionProc;
//     PVOID Context;
// } SIDCACHECALLBACKINFO, *PSIDCACHECALLBACKINFO;
//
//
// typedef struct _SIDQUEUEENTRY
// {
//     LIST_ENTRY ListEntry;
//     ULONG CallbackCount;
//     PSIDCACHECALLBACKINFO Callbacks;
//     /* the SID is appended to this structure */
// } SIDQUEUEENTRY, *PSIDQUEUEENTRY;
//
//
// typedef struct _SIDCACHEENTRY
// {
//     LIST_ENTRY ListEntry;
//     SID_NAME_USE SidNameUse;
//     PWSTR AccountName;
//     PWSTR DomainName;
//     /* the SID and strings are appended to this structure */
// } SIDCACHEENTRY, *PSIDCACHEENTRY;
//
//
// static VOID
// FreeQueueEntry( PSIDCACHEMGR scm,
//                 PSIDQUEUEENTRY QueueEntry)
// {
//     if (QueueEntry->ListEntry.Flink != NULL)
//     {
//         RemoveEntryList(&QueueEntry->ListEntry);
//     }
//
//     HeapFree(scm->Heap,
//              0,
//              QueueEntry->Callbacks);
//
//     HeapFree(scm->Heap,
//              0,
//              QueueEntry);
// }
//
//
// static VOID
// FreeCacheEntry( PSIDCACHEMGR scm,
//                 PSIDCACHEENTRY CacheEntry)
// {
//     RemoveEntryList(&CacheEntry->ListEntry);
//
//     HeapFree(scm->Heap,
//              0,
//              CacheEntry);
// }
//
//
// static VOID
// CleanupSidCacheMgr( PSIDCACHEMGR scm)
// {
//     LsaClose(scm->LsaHandle);
//     CloseHandle(scm->LookupEvent);
//     CloseHandle(scm->LookupThread);
//
//     /* delete the queue */
//     while (!IsListEmpty(&scm->QueueListHead))
//     {
//         PSIDQUEUEENTRY QueueEntry;
//
//         QueueEntry = CONTAINING_RECORD(scm->QueueListHead.Flink,
//                                        SIDQUEUEENTRY,
//                                        ListEntry);
//         FreeQueueEntry(scm,
//                        QueueEntry);
//     }
//
//     /* delete the cache */
//     while (!IsListEmpty(&scm->CacheListHead))
//     {
//         PSIDCACHEENTRY CacheEntry;
//
//         CacheEntry = CONTAINING_RECORD(scm->CacheListHead.Flink,
//                                        SIDCACHEENTRY,
//                                        ListEntry);
//         FreeCacheEntry(scm,
//                        CacheEntry);
//     }
//
//     DeleteCriticalSection(&scm->Lock);
// }
//
//
// static PSIDCACHEMGR
// ReferenceSidCacheMgr( HANDLE SidCacheMgr)
// {
//     PSIDCACHEMGR scm = HandleToScm!(SidCacheMgr);
//
//     if (InterlockedIncrement!(&scm->RefCount) != 1)
//     {
//         return scm;
//     }
//
//     return NULL;
// }
//
//
// static VOID
// DereferenceSidCacheMgr( PSIDCACHEMGR scm)
// {
//     if (InterlockedDecrement!(&scm->RefCount) == 0)
//     {
//         /* Signal the lookup thread so it can terminate */
//         SetEvent(scm->LookupEvent);
//     }
// }
//
//
// static BOOL
// OpenLSAPolicyHandle( LPWSTR SystemName,
//                      ACCESS_MASK DesiredAccess,
//                      PLSA_HANDLE PolicyHandle)
// {
//     LSA_OBJECT_ATTRIBUTES LsaObjectAttributes = {0};
//     LSA_UNICODE_STRING LsaSystemName, *psn;
//     NTSTATUS Status;
//     SIZE_T NameLength;
//
//     if (SystemName != NULL && SystemName[0] != '\0')
//     {
//         NameLength = wcslen(SystemName);
//         if (NameLength > UNICODE_STRING_MAX_CHARS)
//         {
//             return FALSE;
//         }
//
//         LsaSystemName.Buffer = SystemName;
//         LsaSystemName.Length = NameLength * sizeof(WCHAR);
//         LsaSystemName.MaximumLength = LsaSystemName.Length + sizeof(WCHAR);
//         psn = &LsaSystemName;
//     }
//     else
//     {
//         psn = NULL;
//     }
//
//     Status = LsaOpenPolicy(psn,
//                            &LsaObjectAttributes,
//                            DesiredAccess,
//                            PolicyHandle);
//     if (!NT_SUCCESS(Status))
//     {
//         SetLastError(LsaNtStatusToWinError(Status));
//         return FALSE;
//     }
//
//     return TRUE;
// }
//
//
// static BOOL
// LookupSidInformation( PSIDCACHEMGR scm,
//                       PSID pSid,
//                       PSIDREQRESULT *ReqResult)
// {
//     PLSA_REFERENCED_DOMAIN_LIST ReferencedDomain;
//     PLSA_TRANSLATED_NAME Names;
//     PLSA_TRUST_INFORMATION Domain;
//     PLSA_UNICODE_STRING DomainName;
//     SID_NAME_USE SidNameUse = SidTypeUnknown;
//     PPOLICY_ACCOUNT_DOMAIN_INFO PolicyAccountDomainInfo = NULL;
//     NTSTATUS Status;
//     DWORD AccountNameSize, DomainNameSize = 0;
//     PSIDREQRESULT ReqRet = NULL;
//     BOOL Ret = FALSE;
//
//     Status = LsaLookupSids(scm->LsaHandle,
//                            1,
//                            &pSid,
//                            &ReferencedDomain,
//                            &Names);
//     if (NT_SUCCESS(Status))
//     {
//         SidNameUse = Names->Use;
//
//         if (ReferencedDomain != NULL &&
//             Names->DomainIndex >= 0)
//         {
//             Domain = &ReferencedDomain->Domains[Names->DomainIndex];
//             DomainName = &Domain->Name;
//         }
//         else
//         {
//             Domain = NULL;
//             DomainName = NULL;
//         }
//
//         switch (SidNameUse)
//         {
//             case SidTypeAlias:
//             {
//                 if (Domain != NULL)
//                 {
//                     /* query the domain name for BUILTIN accounts */
//                     Status = LsaQueryInformationPolicy(scm->LsaHandle,
//                                                        PolicyAccountDomainInformation,
//                                                        (PVOID*)&PolicyAccountDomainInfo);
//                     if (NT_SUCCESS(Status))
//                     {
//                         DomainName = &PolicyAccountDomainInfo->DomainName;
//
//                         /* make the user believe this is a group */
//                         SidNameUse = (PolicyAccountDomainInfo != NULL ? SidTypeGroup : SidTypeUser);
//                     }
//                 }
//                 break;
//             }
//
//             default:
//             {
//                 DPRINT("Unhandled SID type: 0x%x\n", Names->Use);
//                 break;
//             }
//         }
//
//         AccountNameSize = Names->Name.Length;
//         if (DomainName != NULL)
//         {
//             DomainNameSize = DomainName->Length;
//         }
//
//         ReqRet = HeapAlloc(scm->Heap,
//                            0,
//                            sizeof(SIDREQRESULT) +
//                                (((AccountNameSize + DomainNameSize) + 2) * sizeof(WCHAR)));
//         if (ReqRet != NULL)
//         {
//             ReqRet->RefCount = 1;
//             ReqRet->AccountName = (LPWSTR)(ReqRet + 1);
//             ReqRet->DomainName = ReqRet->AccountName + (AccountNameSize / sizeof(WCHAR)) + 1;
//
//             CopyMemory!(ReqRet->AccountName,
//                        Names->Name.Buffer,
//                        Names->Name.Length);
//
//             if (DomainName != NULL)
//             {
//                 CopyMemory!(ReqRet->DomainName,
//                            DomainName->Buffer,
//                            DomainName->Length);
//             }
//
//             ReqRet->AccountName[AccountNameSize / sizeof(WCHAR)] = '\0';
//             ReqRet->DomainName[DomainNameSize / sizeof(WCHAR)] = '\0';
//
//             ReqRet->SidNameUse = SidNameUse;
//         }
//
//         if (PolicyAccountDomainInfo != NULL)
//         {
//             LsaFreeMemory(PolicyAccountDomainInfo);
//         }
//
//         LsaFreeMemory(ReferencedDomain);
//         LsaFreeMemory(Names);
//
//         Ret = TRUE;
//     }
//     else if (Status == STATUS_NONE_MAPPED)
//     {
//         Ret = TRUE;
//     }
//
//     if (Ret)
//     {
//         *ReqResult = ReqRet;
//     }
//
//     return Ret;
// }
//
//
// static BOOL
// FindSidInCache( PSIDCACHEMGR scm,
//                 PSID pSid,
//                 PSIDREQRESULT *ReqResult)
// {
//     PSIDCACHEENTRY CacheEntry;
//     PLIST_ENTRY CurrentEntry;
//     PSIDREQRESULT ReqRes;
//     BOOL Ret = FALSE;
//
//     /* NOTE: assumes the lists are locked! */
//
//     for (CurrentEntry = scm->CacheListHead.Flink;
//          CurrentEntry != &scm->CacheListHead;
//          CurrentEntry = CurrentEntry->Flink)
//     {
//         CacheEntry = CONTAINING_RECORD(CurrentEntry,
//                                        SIDCACHEENTRY,
//                                        ListEntry);
//
//         if (EqualSid(pSid,
//                      (PSID)(CacheEntry + 1)))
//         {
//             SIZE_T ReqResultSize;
//             ULONG AccountNameLen, DomainNameLen;
//
//             Ret = TRUE;
//
//             AccountNameLen = wcslen(CacheEntry->AccountName);
//             DomainNameLen = wcslen(CacheEntry->DomainName);
//
//             ReqResultSize = sizeof(SIDREQRESULT) +
//                                 (((AccountNameLen + 1) +
//                                   (DomainNameLen + 1)) * sizeof(WCHAR));
//
//             ReqRes = HeapAlloc(scm->Heap,
//                                0,
//                                ReqResultSize);
//             if (ReqRes != NULL)
//             {
//                 PWSTR Buffer = (PWSTR)(ReqRes + 1);
//
//                 ReqRes->RefCount = 1;
//
//                 ReqRes->AccountName = Buffer;
//                 wcscpy(ReqRes->AccountName,
//                        CacheEntry->AccountName);
//                 Buffer += AccountNameLen + 1;
//
//                 ReqRes->DomainName = Buffer;
//                 wcscpy(ReqRes->DomainName,
//                        CacheEntry->DomainName);
//             }
//
//             /* return the result, even if we weren't unable to
//                allocate enough memory! */
//             *ReqResult = ReqRes;
//             break;
//         }
//     }
//
//     return Ret;
// }
//
//
// static VOID
// CacheLookupResults( PSIDCACHEMGR scm,
//                     PSID pSid,
//                     PSIDREQRESULT ReqResult)
// {
//     PSIDCACHEENTRY CacheEntry;
//     DWORD SidLen;
//     SIZE_T AccountNameLen = 0;
//     SIZE_T DomainNameLen = 0;
//     SIZE_T CacheEntrySize = sizeof(SIDCACHEENTRY);
//
//     /* NOTE: assumes the lists are locked! */
//
//     SidLen = GetLengthSid(pSid);
//     CacheEntrySize += SidLen;
//
//     AccountNameLen = wcslen(ReqResult->AccountName);
//     CacheEntrySize += (AccountNameLen + 1) * sizeof(WCHAR);
//
//     DomainNameLen = wcslen(ReqResult->DomainName);
//     CacheEntrySize += (wcslen(ReqResult->DomainName) + 1) * sizeof(WCHAR);
//
//     CacheEntry = HeapAlloc(scm->Heap,
//                            0,
//                            CacheEntrySize);
//     if (CacheEntry != NULL)
//     {
//         PWSTR lpBuf = (PWSTR)((ULONG_PTR)(CacheEntry + 1) + SidLen);
//
//         CacheEntry->SidNameUse = ReqResult->SidNameUse;
//
//         /* append the SID */
//         CopySid(SidLen,
//                 (PSID)(CacheEntry + 1),
//                 pSid);
//
//         /* append the strings */
//         CacheEntry->AccountName = lpBuf;
//         wcscpy(lpBuf,
//                ReqResult->AccountName);
//         lpBuf += AccountNameLen + 1;
//
//         CacheEntry->DomainName = lpBuf;
//         wcscpy(lpBuf,
//                ReqResult->DomainName);
//         lpBuf += DomainNameLen + 1;
//
//         /* add the entry to the cache list */
//         InsertTailList(&scm->CacheListHead,
//                        &CacheEntry->ListEntry);
//     }
// }
//
//
// static DWORD WINAPI
// LookupThreadProc( LPVOID lpParameter)
// {
//     HMODULE hModule;
//     PSIDCACHEMGR scm = (PSIDCACHEMGR)lpParameter;
//
//     /* Reference the dll to avoid problems in case of accidental
//        FreeLibrary calls... */
//     if (!GetModuleHandleExW(GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS,
//                             (LPCWSTR)hDllInstance,
//                             &hModule))
//     {
//         hModule = NULL;
//     }
//
//     while (scm->RefCount != 0)
//     {
//         PSIDQUEUEENTRY QueueEntry = NULL;
//
//         EnterCriticalSection(&scm->Lock);
//
//         /* get the first item of the queue */
//         if (scm->QueueListHead.Flink != &scm->QueueListHead)
//         {
//             QueueEntry = CONTAINING_RECORD(scm->QueueListHead.Flink,
//                                            SIDQUEUEENTRY,
//                                            ListEntry);
//             RemoveEntryList(&QueueEntry->ListEntry);
//             QueueEntry->ListEntry.Flink = NULL;
//         }
//         else
//         {
//             LeaveCriticalSection(&scm->Lock);
//
//             /* wait for the next asynchronous lookup queued */
//             WaitForSingleObject(scm->LookupEvent,
//                                 INFINITE);
//             continue;
//         }
//
//         scm->QueueLookingUp = QueueEntry;
//
//         LeaveCriticalSection(&scm->Lock);
//
//         if (QueueEntry != NULL)
//         {
//             PSIDREQRESULT ReqResult, FoundReqResult;
//             PSID pSid = (PSID)(QueueEntry + 1);
//
//             /* lookup the SID information */
//             if (!LookupSidInformation(scm,
//                                       pSid,
//                                       &ReqResult))
//             {
//                 ReqResult = NULL;
//             }
//
//             EnterCriticalSection(&scm->Lock);
//
//             /* see if the SID was added to the cache in the meanwhile */
//             if (!FindSidInCache(scm,
//                                 pSid,
//                                 &FoundReqResult))
//             {
//                 if (ReqResult != NULL)
//                 {
//                     /* cache the results */
//                     CacheLookupResults(scm,
//                                        pSid,
//                                        ReqResult);
//                 }
//             }
//             else
//             {
//                 if (ReqResult != NULL)
//                 {
//                     /* free the information of our lookup and use the cached
//                        information*/
//                     DereferenceSidReqResult(scm,
//                                             ReqResult);
//                 }
//
//                 ReqResult = FoundReqResult;
//             }
//
//             /* notify the callers unless the lookup was cancelled */
//             if (scm->QueueLookingUp != NULL)
//             {
//                 ULONG i = 0;
//
//                 while (scm->QueueLookingUp != NULL &&
//                        i < QueueEntry->CallbackCount)
//                 {
//                     PVOID Context;
//                     PSIDREQCOMPLETIONPROC CompletionProc;
//
//                     Context = QueueEntry->Callbacks[i].Context;
//                     CompletionProc = QueueEntry->Callbacks[i].CompletionProc;
//
//                     LeaveCriticalSection(&scm->Lock);
//
//                     /* call the completion proc without holding the lock! */
//                     CompletionProc.unwrap_unchecked()(ScmToHandle!(scm),
//                                    pSid,
//                                    ReqResult,
//                                    Context);
//
//                     EnterCriticalSection(&scm->Lock);
//
//                     i++;
//                 }
//
//                 scm->QueueLookingUp = NULL;
//             }
//
//             LeaveCriticalSection(&scm->Lock);
//
//             /* free the queue item */
//             FreeQueueEntry(scm,
//                            QueueEntry);
//         }
//     }
//
//     CleanupSidCacheMgr(scm);
//
//     HeapFree(scm->Heap,
//              0,
//              scm);
//
//     if (hModule != NULL)
//     {
//         /* dereference the library and exit */
//         FreeLibraryAndExitThread(hModule,
//                                  0);
//     }
//
//     return 0;
// }
//
//
//
// HANDLE
// CreateSidCacheMgr( HANDLE Heap,
//                    LPCWSTR SystemName)
// {
//     PSIDCACHEMGR scm;
//
//     if (SystemName == NULL)
//         SystemName = L"";
//
//     scm = HeapAlloc(Heap,
//                     0,
//                     FIELD_OFFSET!(SIDCACHEMGR,
//                                  SystemName[wcslen(SystemName) + 1]));
//     if (scm != NULL)
//     {
//         /* zero the static part of the structure */
//         ZeroMemory!(scm,
//                    FIELD_OFFSET!(SIDCACHEMGR,
//                                 SystemName));
//
//         scm->RefCount = 1;
//         scm->Heap = Heap;
//
//         wcscpy(scm->SystemName,
//                SystemName);
//
//         InitializeCriticalSection(&scm->Lock);
//         InitializeListHead(&scm->QueueListHead);
//         InitializeListHead(&scm->CacheListHead);
//
//         scm->LookupEvent = CreateEvent(NULL,
//                                        FALSE,
//                                        FALSE,
//                                        NULL);
//         if (scm->LookupEvent == NULL)
//         {
//             goto Cleanup;
//         }
//
//         if (!OpenLSAPolicyHandle(scm->SystemName,
//                                  POLICY_LOOKUP_NAMES | POLICY_VIEW_LOCAL_INFORMATION,
//                                  &scm->LsaHandle))
//         {
//             goto Cleanup;
//         }
//
//         scm->LookupThread = CreateThread(NULL,
//                                          0,
//                                          LookupThreadProc,
//                                          scm,
//                                          0,
//                                          NULL);
//         if (scm->LookupThread == NULL)
//         {
// Cleanup:
//             if (scm->LookupEvent != NULL)
//             {
//                 CloseHandle(scm->LookupEvent);
//             }
//
//             if (scm->LsaHandle != NULL)
//             {
//                 LsaClose(scm->LsaHandle);
//             }
//
//             HeapFree(Heap,
//                      0,
//                      scm);
//             scm = NULL;
//         }
//     }
//     else
//     {
//         SetLastError(ERROR_NOT_ENOUGH_MEMORY);
//     }
//
//     return (HANDLE)scm;
// }
//
//
// VOID
// DestroySidCacheMgr( HANDLE SidCacheMgr)
// {
//     PSIDCACHEMGR scm = HandleToScm!(SidCacheMgr);
//
//     if (scm != NULL)
//     {
//         /* remove the keep-alive reference */
//         DereferenceSidCacheMgr(scm);
//     }
// }
//
//
// static BOOL
// QueueSidLookup( PSIDCACHEMGR scm,
//                 PSID pSid,
//                 PSIDREQCOMPLETIONPROC CompletionProc,
//                 PVOID Context)
// {
//     PLIST_ENTRY CurrentEntry;
//     PSIDQUEUEENTRY QueueEntry, FoundEntry = NULL;
//     BOOL Ret = FALSE;
//
//     /* NOTE: assumes the lists are locked! */
//
//     if (scm->QueueLookingUp != NULL &&
//         EqualSid(pSid,
//                  (PSID)(scm->QueueLookingUp + 1)))
//     {
//         FoundEntry = scm->QueueLookingUp;
//     }
//     else
//     {
//         for (CurrentEntry = scm->QueueListHead.Flink;
//              CurrentEntry != &scm->QueueListHead;
//              CurrentEntry = CurrentEntry->Flink)
//         {
//             QueueEntry = CONTAINING_RECORD(CurrentEntry,
//                                            SIDQUEUEENTRY,
//                                            ListEntry);
//
//             if (EqualSid(pSid,
//                          (PSID)(QueueEntry + 1)))
//             {
//                 FoundEntry = QueueEntry;
//                 break;
//             }
//         }
//     }
//
//     if (FoundEntry == NULL)
//     {
//         DWORD SidLength = GetLengthSid(pSid);
//
//         FoundEntry = HeapAlloc(scm->Heap,
//                                0,
//                                sizeof(SIDQUEUEENTRY) + SidLength);
//         if (FoundEntry != NULL)
//         {
//             CopySid(SidLength,
//                     (PSID)(FoundEntry + 1),
//                     pSid);
//
//             FoundEntry->CallbackCount = 1;
//             FoundEntry->Callbacks = HeapAlloc(scm->Heap,
//                                               0,
//                                               sizeof(SIDCACHECALLBACKINFO));
//
//             if (FoundEntry->Callbacks != NULL)
//             {
//                 FoundEntry->Callbacks[0].CompletionProc = CompletionProc;
//                 FoundEntry->Callbacks[0].Context = Context;
//
//                 /* append it to the queue */
//                 InsertTailList(&scm->QueueListHead,
//                                &FoundEntry->ListEntry);
//
//                 /* signal the lookup event */
//                 SetEvent(scm->LookupEvent);
//
//                 Ret = TRUE;
//             }
//             else
//             {
//                 /* unable to queue it because we couldn't allocate the callbacks
//                    array, free the memory and return */
//                 HeapFree(scm->Heap,
//                          0,
//                          FoundEntry);
//             }
//         }
//     }
//     else
//     {
//         PSIDCACHECALLBACKINFO Sidccb;
//
//         /* add the callback */
//         Sidccb = HeapReAlloc(scm->Heap,
//                              0,
//                              FoundEntry->Callbacks,
//                              (FoundEntry->CallbackCount + 1) * sizeof(SIDCACHECALLBACKINFO));
//         if (Sidccb != NULL)
//         {
//             FoundEntry->Callbacks = Sidccb;
//             FoundEntry->Callbacks[FoundEntry->CallbackCount].CompletionProc = CompletionProc;
//             FoundEntry->Callbacks[FoundEntry->CallbackCount++].Context = Context;
//
//             Ret = TRUE;
//         }
//     }
//
//     return Ret;
// }
//
//
// VOID
// DequeueSidLookup( HANDLE SidCacheMgr,
//                   PSID pSid)
// {
//     PLIST_ENTRY CurrentEntry;
//     PSIDQUEUEENTRY QueueEntry;
//     PSIDCACHEMGR scm;
//
//     scm = ReferenceSidCacheMgr(SidCacheMgr);
//     if (scm != NULL)
//     {
//         EnterCriticalSection(&scm->Lock);
//
//         if (scm->QueueLookingUp != NULL &&
//             EqualSid(pSid,
//                      (PSID)(scm->QueueLookingUp + 1)))
//         {
//             /* don't free the queue lookup item! this will be
//                done in the lookup thread */
//             scm->QueueLookingUp = NULL;
//         }
//         else
//         {
//             for (CurrentEntry = scm->QueueListHead.Flink;
//                  CurrentEntry != &scm->QueueListHead;
//                  CurrentEntry = CurrentEntry->Flink)
//             {
//                 QueueEntry = CONTAINING_RECORD(CurrentEntry,
//                                                SIDQUEUEENTRY,
//                                                ListEntry);
//
//                 if (EqualSid(pSid,
//                              (PSID)(QueueEntry + 1)))
//                 {
//                     FreeQueueEntry(scm,
//                                    QueueEntry);
//                     break;
//                 }
//             }
//         }
//
//         LeaveCriticalSection(&scm->Lock);
//
//         DereferenceSidCacheMgr(scm);
//     }
// }
//
//
// VOID
// ReferenceSidReqResult( HANDLE SidCacheMgr,
//                        PSIDREQRESULT ReqResult)
// {
//     PSIDCACHEMGR scm;
//
//     scm = ReferenceSidCacheMgr(SidCacheMgr);
//     if (scm != NULL)
//     {
//         InterlockedIncrement!(&ReqResult->RefCount);
//
//         DereferenceSidCacheMgr(scm);
//     }
// }
//
//
// VOID
// DereferenceSidReqResult( HANDLE SidCacheMgr,
//                          PSIDREQRESULT ReqResult)
// {
//     PSIDCACHEMGR scm;
//
//     scm = ReferenceSidCacheMgr(SidCacheMgr);
//     if (scm != NULL)
//     {
//         if (InterlockedDecrement!(&ReqResult->RefCount) == 0)
//         {
//             HeapFree(scm->Heap,
//                      0,
//                      ReqResult);
//         }
//
//         DereferenceSidCacheMgr(scm);
//     }
// }
//
//
// BOOL
// LookupSidCache( HANDLE SidCacheMgr,
//                 PSID pSid,
//                 PSIDREQCOMPLETIONPROC CompletionProc,
//                 PVOID Context)
// {
//     BOOL Found = FALSE;
//     PSIDREQRESULT ReqResult = NULL;
//     PSIDCACHEMGR scm;
//
//     scm = ReferenceSidCacheMgr(SidCacheMgr);
//     if (scm != NULL)
//     {
//         EnterCriticalSection(&scm->Lock);
//
//         /* search the cache */
//         Found = FindSidInCache(scm,
//                                pSid,
//                                &ReqResult);
//
//         if (!Found)
//         {
//             /* the sid is not in the cache, queue it if not already queued */
//             if (!QueueSidLookup(scm,
//                                 pSid,
//                                 CompletionProc,
//                                 Context))
//             {
//                 PSIDREQRESULT FoundReqResult = NULL;
//
//                 /* unable to queue it, look it up now */
//
//                 LeaveCriticalSection(&scm->Lock);
//
//                 /* lookup everything we need */
//                 if (!LookupSidInformation(scm,
//                                           pSid,
//                                           &ReqResult))
//                 {
//                     ReqResult = NULL;
//                 }
//
//                 EnterCriticalSection(&scm->Lock);
//
//                 /* see if the SID was added to the cache in the meanwhile */
//                 if (!FindSidInCache(scm,
//                                     pSid,
//                                     &FoundReqResult))
//                 {
//                     if (ReqResult != NULL)
//                     {
//                         /* cache the results */
//                         CacheLookupResults(scm,
//                                            pSid,
//                                            ReqResult);
//                     }
//                 }
//                 else
//                 {
//                     if (ReqResult != NULL)
//                     {
//                         /* free the information of our lookup and use the cached
//                            information*/
//                         DereferenceSidReqResult(scm,
//                                                 ReqResult);
//                     }
//
//                     ReqResult = FoundReqResult;
//                 }
//
//                 Found = (ReqResult != NULL);
//             }
//         }
//
//         LeaveCriticalSection(&scm->Lock);
//
//         /* call the completion callback */
//         if (Found)
//         {
//             CompletionProc.unwrap_unchecked()(SidCacheMgr,
//                            pSid,
//                            ReqResult,
//                            Context);
//
//             if (ReqResult != NULL)
//             {
//                 HeapFree(scm->Heap,
//                          0,
//                          ReqResult);
//             }
//         }
//
//         DereferenceSidCacheMgr(scm);
//     }
//
//     return Found;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
