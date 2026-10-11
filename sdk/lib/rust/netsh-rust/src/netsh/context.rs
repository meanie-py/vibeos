use crate::netsh::alias::AliasCommand;
use crate::netsh::alias::ShowAliasCommand;
use crate::netsh::alias::UnaliasCommand;
// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use ntdll_sys::nt::*;
pub static RUSTCONV_WSTR_08F79C40983C: [u16; 7] = [0x0064u16, 0x0065, 0x006C, 0x0065, 0x0074, 0x0065, 0x0000];
pub static RUSTCONV_WSTR_09C488271038: [u16; 4] = [0x0061u16, 0x0064, 0x0064, 0x0000];
pub static RUSTCONV_WSTR_0EA8BD6F19AB: [u16; 5] = [0x0073u16, 0x0068, 0x006F, 0x0077, 0x0000];
pub static RUSTCONV_WSTR_12394F0C4911: [u16; 8] = [0x006Fu16, 0x0066, 0x0066, 0x006C, 0x0069, 0x006E, 0x0065, 0x0000];
pub static RUSTCONV_WSTR_1B9E0FE7B740: [u16; 2] = [0x003Fu16, 0x0000];
pub static RUSTCONV_WSTR_2867187D1E54: [u16; 6] = [0x006Eu16, 0x0065, 0x0074, 0x0073, 0x0068, 0x0000];
pub static RUSTCONV_WSTR_31DF522FC4C4: [u16; 8] = [0x0075u16, 0x006E, 0x0061, 0x006C, 0x0069, 0x0061, 0x0073, 0x0000];
pub static RUSTCONV_WSTR_433DA8124B16: [u16; 5] = [0x006Du16, 0x006F, 0x0064, 0x0065, 0x0000];
pub static RUSTCONV_WSTR_56F6C4E7F7D5: [u16; 5] = [0x0071u16, 0x0075, 0x0069, 0x0074, 0x0000];
pub static RUSTCONV_WSTR_5C9738DE6AF0: [u16; 5] = [0x0065u16, 0x0078, 0x0069, 0x0074, 0x0000];
pub static RUSTCONV_WSTR_621B80B23292: [u16; 10] = [0x006Fu16, 0x0066, 0x0066, 0x006C, 0x0069, 0x006E, 0x0065, 0x000A, 0x000A, 0x0000];
pub static RUSTCONV_WSTR_64FA0E9EA73C: [u16; 7] = [0x0063u16, 0x006F, 0x006D, 0x006D, 0x0069, 0x0074, 0x0000];
pub static RUSTCONV_WSTR_6509121F99F7: [u16; 5] = [0x0068u16, 0x0065, 0x006C, 0x0070, 0x0000];
pub static RUSTCONV_WSTR_65A415E58FC2: [u16; 8] = [0x006Du16, 0x0061, 0x0063, 0x0068, 0x0069, 0x006E, 0x0065, 0x0000];
pub static RUSTCONV_WSTR_6A44AE14AF92: [u16; 6] = [0x0061u16, 0x0062, 0x006F, 0x0072, 0x0074, 0x0000];
pub static RUSTCONV_WSTR_74778EFB9831: [u16; 9] = [0x006Fu16, 0x006E, 0x006C, 0x0069, 0x006E, 0x0065, 0x000A, 0x000A, 0x0000];
pub static RUSTCONV_WSTR_77ECA80E282A: [u16; 3] = [0x002Eu16, 0x002E, 0x0000];
pub static RUSTCONV_WSTR_7D01DF642FC6: [u16; 7] = [0x006Fu16, 0x006E, 0x006C, 0x0069, 0x006E, 0x0065, 0x0000];
pub static RUSTCONV_WSTR_991B57736ED8: [u16; 4] = [0x0062u16, 0x0079, 0x0065, 0x0000];
pub static RUSTCONV_WSTR_9BE6D558E725: [u16; 5] = [0x0070u16, 0x006F, 0x0070, 0x0064, 0x0000];
pub static RUSTCONV_WSTR_A0E70D05E916: [u16; 6] = [0x0061u16, 0x006C, 0x0069, 0x0061, 0x0073, 0x0000];
pub static RUSTCONV_WSTR_AB3681E84239: [u16; 5] = [0x0065u16, 0x0078, 0x0065, 0x0063, 0x0000];
pub static RUSTCONV_WSTR_B94677D1EF59: [u16; 7] = [0x0068u16, 0x0065, 0x006C, 0x0070, 0x0065, 0x0072, 0x0000];
pub static RUSTCONV_WSTR_E9D05A71AD4A: [u16; 5] = [0x0064u16, 0x0075, 0x006D, 0x0070, 0x0000];
pub static RUSTCONV_WSTR_EAFD7A71F5E7: [u16; 6] = [0x0070u16, 0x0075, 0x0073, 0x0068, 0x0064, 0x0000];
pub static RUSTCONV_WSTR_FCABAF3CA758: [u16; 4] = [0x0073u16, 0x0065, 0x0074, 0x0000];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[no_mangle]
pub static mut pRootContext: PCONTEXT_ENTRY =
    ::core::ptr::null::<_CONTEXT_ENTRY>() as *mut _CONTEXT_ENTRY;

#[no_mangle]
pub static mut pCurrentContext: PCONTEXT_ENTRY =
    ::core::ptr::null::<_CONTEXT_ENTRY>() as *mut _CONTEXT_ENTRY;

#[no_mangle]
pub static mut pContextStackHead: PCONTEXT_STACK_ENTRY =
    ::core::ptr::null::<_CONTEXT_STACK_ENTRY>() as *mut _CONTEXT_STACK_ENTRY;

#[no_mangle]
pub static mut pContextStackTail: PCONTEXT_STACK_ENTRY =
    ::core::ptr::null::<_CONTEXT_STACK_ENTRY>() as *mut _CONTEXT_STACK_ENTRY;

#[no_mangle]
pub static mut g_pszMachine: PWSTR = ::core::ptr::null::<WCHAR>() as *mut WCHAR;

#[no_mangle]
pub static mut g_bOnline: BOOL = TRUE;

#[repr(C)]
pub struct CONTEXT_STACK_ENTRY {
    pub pPrev: *mut CONTEXT_STACK_ENTRY,
    pub pNext: *mut CONTEXT_STACK_ENTRY,
    pub pContext: PCONTEXT_ENTRY,
}
pub type PCONTEXT_STACK_ENTRY = *mut CONTEXT_STACK_ENTRY;

#[no_mangle]
pub unsafe extern "C" fn AddContext(
    pParentContext: PCONTEXT_ENTRY,
    pszName: PWSTR,
    pGuid: *mut GUID,
) -> PCONTEXT_ENTRY {
    let mut pEntry: PCONTEXT_ENTRY = core::ptr::null_mut();

    DPRINT!("AddContext(%S)\n", pszName);
    if !pParentContext.is_null() {
        DPRINT!("ParentContext %S\n", (*pParentContext).pszContextName);
    }

    if !pParentContext.is_null() && pszName.is_null() {
        return core::ptr::null_mut();
    }

    /* Allocate the entry */
    pEntry = HeapAlloc(
        GetProcessHeap(),
        HEAP_ZERO_MEMORY as DWORD,
        core::mem::size_of::<CONTEXT_ENTRY>() as SIZE_T,
    ) as PCONTEXT_ENTRY;
    if pEntry.is_null() {
        return core::ptr::null_mut();
    }

    /* Allocate the name buffer */
    if !pszName.is_null() {
        (*pEntry).pszContextName = HeapAlloc(
            GetProcessHeap(),
            HEAP_ZERO_MEMORY as DWORD,
            (wcslen(pszName as LPCWSTR) as SIZE_T
                + 1 as SIZE_T)
                * core::mem::size_of::<WCHAR>() as SIZE_T,
        ) as PWSTR;
        if (*pEntry).pszContextName.is_null() {
            HeapFree(
                GetProcessHeap(),
                0 as DWORD,
                pEntry as LPVOID,
            );
            return core::ptr::null_mut();
        }

        /* Fill the entry */
        wcscpy((*pEntry).pszContextName as LPWSTR, pszName as LPCWSTR);
    }

    (*pEntry).pParentContext = pParentContext;
    if !pGuid.is_null() {
        CopyMemory(
            core::ptr::addr_of_mut!((*pEntry).Guid) as *mut core::ffi::c_void,
            pGuid as *const core::ffi::c_void,
            core::mem::size_of::<GUID>(),
        );
    }

    /* Insert it */
    if !pParentContext.is_null() {
        if (*pParentContext).pSubContextHead.is_null()
            && (*pParentContext).pSubContextTail.is_null()
        {
            (*pParentContext).pSubContextHead = pEntry;
            (*pParentContext).pSubContextTail = pEntry;
        } else {
            (*pEntry).pPrev = (*pParentContext).pSubContextTail;
            (*(*pParentContext).pSubContextTail).pNext = pEntry;
            (*pParentContext).pSubContextTail = pEntry;
        }
    }

    return pEntry;
}
#[no_mangle]
pub unsafe extern "C" fn AddContextCommand(
    mut pContext: PCONTEXT_ENTRY,
    mut pwszCmdToken: LPCWSTR,
    mut pfnCmdHandler: PFN_HANDLE_CMD,
    mut dwShortCmdHelpToken: DWORD,
    mut dwCmdHlpToken: DWORD,
    mut dwFlags: DWORD,
    mut pfnOsVersionCheck: PNS_OSVERSIONCHECK,
) -> PCOMMAND_ENTRY {
    let mut pEntry: PCOMMAND_ENTRY = core::ptr::null_mut::<_COMMAND_ENTRY>();
    if pfnCmdHandler.is_none() {
        return core::ptr::null_mut::<_COMMAND_ENTRY>();
    }
    pEntry = HeapAlloc(
        GetProcessHeap(),
        HEAP_ZERO_MEMORY as DWORD,
        core::mem::size_of::<COMMAND_ENTRY>() as SIZE_T,
    ) as PCOMMAND_ENTRY;
    if pEntry.is_null() {
        return core::ptr::null_mut::<_COMMAND_ENTRY>();
    }
    (*pEntry).pwszCmdToken = HeapAlloc(
        GetProcessHeap(),
        HEAP_ZERO_MEMORY as DWORD,
        (wcslen(pwszCmdToken as *const wchar_t) as SIZE_T)
            .wrapping_add(1 as SIZE_T)
            .wrapping_mul(core::mem::size_of::<WCHAR>() as SIZE_T),
    ) as PWSTR;
    if (*pEntry).pwszCmdToken.is_null() {
        HeapFree(GetProcessHeap(), 0 as DWORD, pEntry as PVOID);
        return core::ptr::null_mut::<_COMMAND_ENTRY>();
    }
    wcscpy(
        (*pEntry).pwszCmdToken as *mut wchar_t,
        pwszCmdToken as *const wchar_t,
    );
    (*pEntry).pfnCmdHandler = pfnCmdHandler;
    (*pEntry).dwShortCmdHelpToken = dwShortCmdHelpToken;
    (*pEntry).dwCmdHlpToken = dwCmdHlpToken;
    (*pEntry).dwFlags = dwFlags;
    (*pEntry).pfnOsVersionCheck = pfnOsVersionCheck;
    if (*pContext).pCommandListHead.is_null() && (*pContext).pCommandListTail.is_null() {
        (*pContext).pCommandListHead = pEntry;
        (*pContext).pCommandListTail = pEntry;
    } else {
        (*pEntry).pPrev = (*pContext).pCommandListTail as *mut _COMMAND_ENTRY;
        (*(*pContext).pCommandListTail).pNext = pEntry as *mut _COMMAND_ENTRY;
        (*pContext).pCommandListTail = pEntry;
    }
    return pEntry;
}
#[no_mangle]
pub unsafe extern "C" fn AddCommandGroup(
    mut pContext: PCONTEXT_ENTRY,
    mut pwszCmdGroupToken: LPCWSTR,
    mut dwShortCmdHelpToken: DWORD,
    mut dwFlags: DWORD,
    mut pfnOsVersionCheck: PNS_OSVERSIONCHECK,
) -> PCOMMAND_GROUP {
    let mut pEntry: PCOMMAND_GROUP;

    DPRINT!("AddCommandGroup(%S %lu)\n", pwszCmdGroupToken, dwShortCmdHelpToken);

    /* Allocate the entry */
    pEntry = HeapAlloc(
        GetProcessHeap(),
        HEAP_ZERO_MEMORY,
        core::mem::size_of::<COMMAND_GROUP>() as SIZE_T,
    ) as PCOMMAND_GROUP;
    if pEntry.is_null() {
        return core::ptr::null_mut();
    }

    (*pEntry).pwszCmdGroupToken = HeapAlloc(
        GetProcessHeap(),
        HEAP_ZERO_MEMORY,
        (wcslen(pwszCmdGroupToken) as SIZE_T)
            .wrapping_add(1 as SIZE_T)
            .wrapping_mul(core::mem::size_of::<WCHAR>() as SIZE_T),
    ) as PWSTR;
    if (*pEntry).pwszCmdGroupToken.is_null() {
        HeapFree(GetProcessHeap(), 0, pEntry as PVOID);
        return core::ptr::null_mut();
    }

    wcscpy((*pEntry).pwszCmdGroupToken as LPWSTR, pwszCmdGroupToken);
    (*pEntry).dwShortCmdHelpToken = dwShortCmdHelpToken;
    (*pEntry).dwFlags = dwFlags;
    (*pEntry).pfnOsVersionCheck = pfnOsVersionCheck;

    if (*pContext).pGroupListHead.is_null() && (*pContext).pGroupListTail.is_null() {
        (*pContext).pGroupListHead = pEntry;
        (*pContext).pGroupListTail = pEntry;
    } else {
        (*pEntry).pPrev = (*pContext).pGroupListTail;
        (*(*pContext).pGroupListTail).pNext = pEntry;
        (*pContext).pGroupListTail = pEntry;
    }

    return pEntry;
}

#[no_mangle]
pub unsafe extern "C" fn AddGroupCommand(
    mut pGroup: PCOMMAND_GROUP,
    mut pwszCmdToken: LPCWSTR,
    mut pfnCmdHandler: PFN_HANDLE_CMD,
    mut dwShortCmdHelpToken: DWORD,
    mut dwCmdHlpToken: DWORD,
    mut dwFlags: DWORD,
    mut pfnOsVersionCheck: PNS_OSVERSIONCHECK,
) -> PCOMMAND_ENTRY {
    let mut pEntry: PCOMMAND_ENTRY = core::ptr::null_mut();

    if pfnCmdHandler.is_none() {
        return core::ptr::null_mut();
    }

    /* Allocate the entry */
    pEntry = HeapAlloc(
        GetProcessHeap(),
        HEAP_ZERO_MEMORY as DWORD,
        core::mem::size_of::<COMMAND_ENTRY>() as SIZE_T,
    ) as PCOMMAND_ENTRY;
    if pEntry.is_null() {
        return core::ptr::null_mut();
    }

    (*pEntry).pwszCmdToken = HeapAlloc(
        GetProcessHeap(),
        HEAP_ZERO_MEMORY as DWORD,
        (wcslen(pwszCmdToken as *const wchar_t) as SIZE_T
            + 1 as SIZE_T)
            .wrapping_mul(core::mem::size_of::<WCHAR>() as SIZE_T),
    ) as PWSTR;
    if (*pEntry).pwszCmdToken.is_null() {
        HeapFree(
            GetProcessHeap(),
            0 as DWORD,
            pEntry as PVOID,
        );
        return core::ptr::null_mut();
    }

    wcscpy(
        (*pEntry).pwszCmdToken as *mut wchar_t,
        pwszCmdToken as *const wchar_t,
    );

    (*pEntry).pfnCmdHandler = pfnCmdHandler;
    (*pEntry).dwShortCmdHelpToken = dwShortCmdHelpToken;
    (*pEntry).dwCmdHlpToken = dwCmdHlpToken;
    (*pEntry).dwFlags = dwFlags;
    (*pEntry).pfnOsVersionCheck = pfnOsVersionCheck;

    if (*pGroup).pCommandListHead.is_null() && (*pGroup).pCommandListTail.is_null() {
        (*pGroup).pCommandListHead = pEntry;
        (*pGroup).pCommandListTail = pEntry;
    } else {
        (*pEntry).pPrev = (*pGroup).pCommandListTail;
        (*(*pGroup).pCommandListTail).pNext = pEntry;
        (*pGroup).pCommandListTail = pEntry;
    }

    return pEntry;
}
#[no_mangle]
pub unsafe extern "C" fn RemoveContextFromStack(mut pContextEntry: PCONTEXT_ENTRY) {
    let mut pStackEntry: PCONTEXT_STACK_ENTRY = core::ptr::null_mut();
    let mut pNextEntry: PCONTEXT_STACK_ENTRY = core::ptr::null_mut();
    if pContextStackHead.is_null() {
        return;
    }
    pStackEntry = pContextStackHead;
    while (1) != 0 {
        if (*pStackEntry).pContext == pContextEntry {
            if pStackEntry == pContextStackHead && pStackEntry == pContextStackTail {
                pContextStackHead = core::ptr::null_mut();
                pContextStackTail = core::ptr::null_mut();
                HeapFree(GetProcessHeap(), 0 as DWORD, pStackEntry as PVOID);
                return;
            } else if pStackEntry == pContextStackHead {
                (*(*pStackEntry).pNext).pPrev = core::ptr::null_mut();
                pContextStackHead = (*pStackEntry).pNext as PCONTEXT_STACK_ENTRY;
                HeapFree(GetProcessHeap(), 0 as DWORD, pStackEntry as PVOID);
                pStackEntry = pContextStackHead;
            } else if pStackEntry == pContextStackTail {
                (*(*pStackEntry).pPrev).pNext = core::ptr::null_mut();
                pContextStackTail = (*pStackEntry).pPrev as PCONTEXT_STACK_ENTRY;
                HeapFree(GetProcessHeap(), 0 as DWORD, pStackEntry as PVOID);
                return;
            } else {
                pNextEntry = (*pStackEntry).pNext as PCONTEXT_STACK_ENTRY;
                (*(*pStackEntry).pPrev).pNext = (*pStackEntry).pNext;
                (*(*pStackEntry).pNext).pPrev = (*pStackEntry).pPrev;
                HeapFree(GetProcessHeap(), 0 as DWORD, pStackEntry as PVOID);
                pStackEntry = pNextEntry;
            }
        } else {
            if pStackEntry == pContextStackTail {
                return;
            }
            pStackEntry = (*pStackEntry).pNext as PCONTEXT_STACK_ENTRY;
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn DeleteContext(pszName: PWSTR) {}

unsafe extern "C" fn ContextCompare(
    p1: *const core::ffi::c_void,
    p2: *const core::ffi::c_void,
) -> core::ffi::c_int {
    return (*(p1 as PCONTEXT_ENTRY)).ulPriority.wrapping_sub((*(p2 as PCONTEXT_ENTRY)).ulPriority) as core::ffi::c_int;
}

unsafe extern "C" fn DumpContext(
    pContext: PCONTEXT_ENTRY,
    pwszMachine: LPCWSTR,
    ppwcArguments: *mut LPWSTR,
    dwArgCount: DWORD,
    pvData: LPCVOID,
) -> DWORD {
    let mut pSubContext: PCONTEXT_ENTRY = core::ptr::null_mut();
    let mut pSortArray: *mut PCONTEXT_ENTRY = core::ptr::null_mut();
    let mut dwCount: DWORD;
    let mut dwIndex: DWORD;
    let mut dwError: DWORD = ERROR_SUCCESS;

    DPRINT!("DumpContext()\n");

    if (*pContext).pfnDumpFn.is_some() {
        dwError = (*pContext).pfnDumpFn.expect("non-null function pointer")(
            pwszMachine,
            ppwcArguments,
            dwArgCount,
            pvData,
        );
        if dwError != ERROR_SUCCESS {
            DPRINT1!("Dump function failed (Error %lu)\n", dwError);
            return dwError;
        }
    }

    if (*pContext).pSubContextHead.is_null() {
        return dwError;
    }

    /* Count the sub-contexts */
    dwCount = 0;
    pSubContext = (*pContext).pSubContextHead as PCONTEXT_ENTRY;
    while !pSubContext.is_null() {
        dwCount = dwCount.wrapping_add(1);
        pSubContext = (*pSubContext).pNext as PCONTEXT_ENTRY;
    }

    /* Allocate the sort array */
    pSortArray = HeapAlloc(
        GetProcessHeap(),
        0,
        (dwCount as SIZE_T).wrapping_mul(core::mem::size_of::<PCONTEXT_ENTRY>() as SIZE_T),
    ) as *mut PCONTEXT_ENTRY;
    if pSortArray.is_null() {
        return ERROR_NOT_ENOUGH_MEMORY;
    }

    /* Fill the sort array */
    dwIndex = 0;
    pSubContext = (*pContext).pSubContextHead as PCONTEXT_ENTRY;
    while !pSubContext.is_null() {
        let fresh16 = &mut *pSortArray.offset(dwIndex as isize);
        *fresh16 = pSubContext;
        dwIndex = dwIndex.wrapping_add(1);
        pSubContext = (*pSubContext).pNext as PCONTEXT_ENTRY;
    }

    /* Sort the array */
    qsort(
        pSortArray as *mut core::ffi::c_void,
        dwCount as usize,
        core::mem::size_of::<PCONTEXT_ENTRY>() as usize,
        Some(
            ContextCompare
                as unsafe extern "C" fn(
                    *const core::ffi::c_void,
                    *const core::ffi::c_void,
                ) -> core::ffi::c_int,
        ),
    );

    /* Dump the sub-contexts */
    dwIndex = 0;
    while dwIndex < dwCount {
        dwError = DumpContext(
            *pSortArray.offset(dwIndex as isize),
            pwszMachine,
            ppwcArguments,
            dwArgCount,
            pvData,
        );
        if dwError != ERROR_SUCCESS {
            DPRINT1!("Dump function failed (Error %lu)\n", dwError);
            break;
        }
        dwIndex = dwIndex.wrapping_add(1);
    }

    /* Free the sort array */
    HeapFree(GetProcessHeap(), 0, pSortArray as *mut core::ffi::c_void);

    return dwError;
}

unsafe extern "C" fn CommitContext(mut pContext: PCONTEXT_ENTRY, mut dwAction: DWORD) -> DWORD {
    let mut pSubContext: PCONTEXT_ENTRY = core::ptr::null_mut();
    let mut pSortArray: *mut PCONTEXT_ENTRY = core::ptr::null_mut();
    let mut dwCount: DWORD;
    let mut dwIndex: DWORD;
    let mut dwError: DWORD = ERROR_SUCCESS;

    DPRINT1!("CommitContext(%p %lu)\n", pContext, dwAction);

    if (*pContext).pfnCommitFn.is_some() {
        dwError = (*pContext).pfnCommitFn.expect("non-null function pointer")(dwAction);
        if dwError != ERROR_SUCCESS {
            DPRINT1!("Commit function failed (Error %lu)\n", dwError);
            return dwError;
        }
    }

    if (*pContext).pSubContextHead.is_null() {
        return dwError;
    }

    /* Count the sub-contexts */
    dwCount = 0;
    pSubContext = (*pContext).pSubContextHead as PCONTEXT_ENTRY;
    while !pSubContext.is_null() {
        dwCount = dwCount.wrapping_add(1);
        pSubContext = (*pSubContext).pNext as PCONTEXT_ENTRY;
    }

    /* Allocate the sort array */
    pSortArray = HeapAlloc(
        GetProcessHeap(),
        0,
        (dwCount as SIZE_T).wrapping_mul(core::mem::size_of::<PCONTEXT_ENTRY>() as SIZE_T),
    ) as *mut PCONTEXT_ENTRY;
    if pSortArray.is_null() {
        return ERROR_NOT_ENOUGH_MEMORY;
    }

    /* Fill the sort array */
    dwIndex = 0;
    pSubContext = (*pContext).pSubContextHead as PCONTEXT_ENTRY;
    while !pSubContext.is_null() {
        *pSortArray.add(dwIndex as usize) = pSubContext;
        dwIndex = dwIndex.wrapping_add(1);
        pSubContext = (*pSubContext).pNext as PCONTEXT_ENTRY;
    }

    /* Sort the array */
    qsort(
        pSortArray as *mut ::core::ffi::c_void,
        dwCount as size_t,
        core::mem::size_of::<PCONTEXT_ENTRY>() as size_t,
        Some(
            ContextCompare
                as unsafe extern "C" fn(
                    *const ::core::ffi::c_void,
                    *const ::core::ffi::c_void,
                ) -> ::core::ffi::c_int,
        ),
    );

    /* Commit the sub-contexts */
    dwIndex = 0;
    while dwIndex < dwCount {
        dwError = CommitContext(*pSortArray.add(dwIndex as usize), dwAction);
        if dwError != ERROR_SUCCESS {
            DPRINT1!("Commit function failed (Error %lu)\n", dwError);
            break;
        }
        dwIndex = dwIndex.wrapping_add(1);
    }

    /* Free the sort array */
    HeapFree(
        GetProcessHeap(),
        0,
        pSortArray as PVOID,
    );

    return dwError;
}
#[no_mangle]
pub unsafe extern "system" fn UpCommand(
    pwszMachine: LPCWSTR,
    argv: *mut LPWSTR,
    dwCurrentIndex: DWORD,
    dwArgCount: DWORD,
    dwFlags: DWORD,
    pvData: LPCVOID,
    pbDone: *mut BOOL,
) -> DWORD {
    if pCurrentContext != pRootContext {
        pCurrentContext = (*pCurrentContext).pParentContext as PCONTEXT_ENTRY;
    }

    return ERROR_SUCCESS as DWORD;
}

#[no_mangle]
pub unsafe extern "system" fn AbortCommand(
    pwszMachine: LPCWSTR,
    argv: *mut LPWSTR,
    dwCurrentIndex: DWORD,
    dwArgCount: DWORD,
    dwFlags: DWORD,
    pvData: LPCVOID,
    pbDone: *mut BOOL,
) -> DWORD {
    DPRINT!("AbortCommand()\n");
    CommitContext(pRootContext, NETSH_FLUSH);
    return ERROR_SUCCESS;
}

#[no_mangle]
pub unsafe extern "system" fn CommitCommand(
    pwszMachine: LPCWSTR,
    argv: *mut LPWSTR,
    dwCurrentIndex: DWORD,
    dwArgCount: DWORD,
    dwFlags: DWORD,
    pvData: LPCVOID,
    pbDone: *mut BOOL,
) -> DWORD {
    DPRINT!("CommitCommand()\n");
    CommitContext(pRootContext, NETSH_SAVE as ::core::ffi::c_int as DWORD);
    return ERROR_SUCCESS as DWORD;
}

#[no_mangle]
pub unsafe extern "system" fn DumpCommand(
    pwszMachine: LPCWSTR,
    ppwcArguments: *mut LPWSTR,
    dwCurrentIndex: DWORD,
    dwArgCount: DWORD,
    dwFlags: DWORD,
    pvData: LPCVOID,
    pbDone: *mut BOOL,
) -> DWORD {
    DPRINT!("DumpCommand()\n");

    return DumpContext(
        pCurrentContext,
        pwszMachine,
        ppwcArguments,
        dwArgCount,
        pvData,
    );
}

#[no_mangle]
pub unsafe extern "system" fn ExecCommand(
    pwszMachine: LPCWSTR,
    argv: *mut LPWSTR,
    dwCurrentIndex: DWORD,
    dwArgCount: DWORD,
    dwFlags: DWORD,
    pvData: LPCVOID,
    pbDone: *mut BOOL,
) -> DWORD {
    DPRINT!("ExecCommand()\n");

    if dwArgCount.wrapping_sub(dwCurrentIndex) != 1 {
        return ERROR_SHOW_USAGE;
    }

    return RunScript(*argv.offset(dwCurrentIndex as isize));
}

#[no_mangle]
pub unsafe extern "system" fn ExitCommand(
    pwszMachine: LPCWSTR,
    argv: *mut LPWSTR,
    dwCurrentIndex: DWORD,
    dwArgCount: DWORD,
    dwFlags: DWORD,
    pvData: LPCVOID,
    pbDone: *mut BOOL,
) -> DWORD {
    if g_bOnline == FALSE {
        CommitContext(pRootContext, NETSH_FLUSH);
    }

    *pbDone = TRUE;
    return ERROR_SUCCESS;
}

#[no_mangle]
pub unsafe extern "system" fn RemCommand(
    pwszMachine: LPCWSTR,
    argv: *mut LPWSTR,
    dwCurrentIndex: DWORD,
    dwArgCount: DWORD,
    dwFlags: DWORD,
    pvData: LPCVOID,
    pbDone: *mut BOOL,
) -> DWORD {
    ERROR_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn OfflineCommand(
    pwszMachine: LPCWSTR,
    argv: *mut LPWSTR,
    dwCurrentIndex: DWORD,
    dwArgCount: DWORD,
    dwFlags: DWORD,
    pvData: LPCVOID,
    pbDone: *mut BOOL,
) -> DWORD {
    DPRINT!("OfflineCommand()\n");
    CommitContext(pRootContext, NETSH_UNCOMMIT);
    g_bOnline = FALSE;
    return ERROR_SUCCESS;
}

#[no_mangle]
pub unsafe extern "system" fn OnlineCommand(
    pwszMachine: LPCWSTR,
    argv: *mut LPWSTR,
    dwCurrentIndex: DWORD,
    dwArgCount: DWORD,
    dwFlags: DWORD,
    pvData: LPCVOID,
    pbDone: *mut BOOL,
) -> DWORD {
    DPRINT!("OnlineCommand()\n");
    CommitContext(pRootContext, NETSH_COMMIT as DWORD);
    g_bOnline = TRUE as BOOL;
    return ERROR_SUCCESS as DWORD;
}

#[no_mangle]
pub unsafe extern "system" fn PopdCommand(
    pwszMachine: LPCWSTR,
    argv: *mut LPWSTR,
    dwCurrentIndex: DWORD,
    dwArgCount: DWORD,
    dwFlags: DWORD,
    pvData: LPCVOID,
    pbDone: *mut BOOL,
) -> DWORD {
    let mut pEntry: PCONTEXT_STACK_ENTRY = core::ptr::null_mut();

    DPRINT!("PopdCommand()\n");

    if pContextStackHead.is_null() {
        return ERROR_SUCCESS;
    }

    pEntry = pContextStackHead;

    pCurrentContext = (*pEntry).pContext;

    if pContextStackTail == pEntry {
        pContextStackHead = core::ptr::null_mut();
        pContextStackTail = core::ptr::null_mut();
    } else {
        pContextStackHead = (*pEntry).pNext;
        (*pContextStackHead).pPrev = core::ptr::null_mut();
    }

    HeapFree(GetProcessHeap(), 0, pEntry as PVOID);

    return ERROR_SUCCESS;
}

#[no_mangle]
pub unsafe extern "system" fn PushdCommand(
    pwszMachine: LPCWSTR,
    argv: *mut LPWSTR,
    dwCurrentIndex: DWORD,
    dwArgCount: DWORD,
    dwFlags: DWORD,
    pvData: LPCVOID,
    pbDone: *mut BOOL,
) -> DWORD {
    let mut pEntry: PCONTEXT_STACK_ENTRY = core::ptr::null_mut();

    DPRINT!("PushdCommand()\n");

    pEntry = HeapAlloc(
        GetProcessHeap(),
        HEAP_ZERO_MEMORY as DWORD,
        core::mem::size_of::<CONTEXT_STACK_ENTRY>() as SIZE_T,
    ) as PCONTEXT_STACK_ENTRY;
    if pEntry.is_null() {
        return ERROR_NOT_ENOUGH_MEMORY as DWORD;
    }

    (*pEntry).pContext = pCurrentContext;
    if pContextStackHead.is_null() {
        pContextStackHead = pEntry;
        pContextStackTail = pEntry;
    } else {
        (*pEntry).pNext = pContextStackHead;
        (*pContextStackHead).pPrev = pEntry;
        pContextStackHead = pEntry;
    }

    return ERROR_SUCCESS as DWORD;
}
#[no_mangle]
pub unsafe extern "system" fn SetMachineCommand(
    pwszMachine: LPCWSTR,
    argv: *mut LPWSTR,
    dwCurrentIndex: DWORD,
    dwArgCount: DWORD,
    dwFlags: DWORD,
    pvData: LPCVOID,
    pbDone: *mut BOOL,
) -> DWORD {
    let mut dwError: DWORD = ERROR_SUCCESS as DWORD;

    DPRINT!(
        "SetMachineCommand(pwszMachine %S  dwCurrentIndex %lu  dwArgCount %lu)\n",
        pwszMachine,
        dwCurrentIndex,
        dwArgCount
    );

    if dwArgCount.wrapping_sub(dwCurrentIndex) > 1 as DWORD {
        return ERROR_SHOW_USAGE as DWORD;
    }

    if !g_pszMachine.is_null() {
        HeapFree(
            GetProcessHeap(),
            0 as DWORD,
            g_pszMachine as PVOID,
        );
        g_pszMachine = core::ptr::null_mut();
    }

    if dwArgCount.wrapping_sub(dwCurrentIndex) == 1 as DWORD {
        g_pszMachine = HeapAlloc(
            GetProcessHeap(),
            0 as DWORD,
            (core::mem::size_of::<LPWSTR>() + 1 as usize)
                * core::mem::size_of::<WCHAR>(),
        ) as PWSTR;
        if g_pszMachine.is_null() {
            return ERROR_NOT_ENOUGH_MEMORY as DWORD;
        }
        wcscpy(
            g_pszMachine,
            *argv.offset(dwCurrentIndex as isize),
        );
    }

    return dwError;
}

#[no_mangle]
pub unsafe extern "system" fn SetModeCommand(
    pwszMachine: LPCWSTR,
    argv: *mut LPWSTR,
    dwCurrentIndex: DWORD,
    dwArgCount: DWORD,
    dwFlags: DWORD,
    pvData: LPCVOID,
    pbDone: *mut BOOL,
) -> DWORD {
    let mut dwError: DWORD = ERROR_SUCCESS;

    DPRINT!("SetModeCommand(pwszMachine %S  dwCurrentIndex %lu  dwArgCount %lu)\n",
           pwszMachine, dwCurrentIndex, dwArgCount);

    if (dwArgCount.wrapping_sub(dwCurrentIndex)) != 1 {
        return ERROR_SHOW_USAGE;
    }

    if _wcsicmp(*argv.add(dwCurrentIndex as usize) as *const u16, RUSTCONV_WSTR_12394F0C4911.as_ptr()) == 0
    {
        CommitContext(pRootContext, NETSH_UNCOMMIT);
        g_bOnline = FALSE;
    }
    else if _wcsicmp(*argv.add(dwCurrentIndex as usize) as *const u16, RUSTCONV_WSTR_7D01DF642FC6.as_ptr()) == 0
    {
        CommitContext(pRootContext, NETSH_COMMIT);
        g_bOnline = TRUE;
    }
    else
    {
        dwError = ERROR_INVALID_SYNTAX;
    }

    return dwError;
}
#[no_mangle]
pub unsafe extern "system" fn ShowModeCommand(
    pwszMachine: LPCWSTR,
    argv: *mut LPWSTR,
    dwCurrentIndex: DWORD,
    dwArgCount: DWORD,
    dwFlags: DWORD,
    pvData: LPCVOID,
    pbDone: *mut BOOL,
) -> DWORD {
    DPRINT!("ShowModeCommand()\n");
    ConPuts(
        core::ptr::addr_of_mut!(StdOut),
        if g_bOnline != 0 {
            RUSTCONV_WSTR_74778EFB9831.as_ptr()
        } else {
            RUSTCONV_WSTR_621B80B23292.as_ptr()
        },
    );
    return ERROR_SUCCESS;
}
#[no_mangle]
pub unsafe extern "C" fn CreateRootContext() -> BOOL {
    let mut pGroup: PCOMMAND_GROUP = core::ptr::null_mut();

    pRootContext = AddContext(
        core::ptr::null_mut(),
        RUSTCONV_WSTR_2867187D1E54.as_ptr(),
        core::ptr::null_mut(),
    );
    DPRINT!("pRootContext: %p\n", pRootContext);
    if pRootContext.is_null() {
        return FALSE;
    }

    (*pRootContext).hModule = g_hModule;

    AddContextCommand(
        pRootContext,
        RUSTCONV_WSTR_77ECA80E282A.as_ptr(),
        Some(UpCommand as unsafe extern "C" fn(LPCWSTR, *mut LPWSTR, DWORD, DWORD, DWORD, LPCVOID, *mut BOOL) -> DWORD),
        IDS_HLP_UP,
        IDS_HLP_UP_EX,
        0,
        None,
    );
    AddContextCommand(
        pRootContext,
        RUSTCONV_WSTR_1B9E0FE7B740.as_ptr(),
        None,
        IDS_HLP_HELP,
        IDS_HLP_HELP_EX,
        0,
        None,
    );
    AddContextCommand(
        pRootContext,
        RUSTCONV_WSTR_6A44AE14AF92.as_ptr(),
        Some(AbortCommand as unsafe extern "C" fn(LPCWSTR, *mut LPWSTR, DWORD, DWORD, DWORD, LPCVOID, *mut BOOL) -> DWORD),
        IDS_HLP_ABORT,
        IDS_HLP_ABORT_EX,
        0,
        None,
    );
    AddContextCommand(
        pRootContext,
        RUSTCONV_WSTR_A0E70D05E916.as_ptr(),
        Some(AliasCommand as unsafe extern "C" fn(LPCWSTR, *mut LPWSTR, DWORD, DWORD, DWORD, LPCVOID, *mut BOOL) -> DWORD),
        IDS_HLP_ALIAS,
        IDS_HLP_ALIAS_EX,
        0,
        None,
    );
    AddContextCommand(
        pRootContext,
        RUSTCONV_WSTR_991B57736ED8.as_ptr(),
        Some(ExitCommand as unsafe extern "C" fn(LPCWSTR, *mut LPWSTR, DWORD, DWORD, DWORD, LPCVOID, *mut BOOL) -> DWORD),
        IDS_HLP_EXIT,
        IDS_HLP_EXIT_EX,
        0,
        None,
    );
    AddContextCommand(
        pRootContext,
        RUSTCONV_WSTR_64FA0E9EA73C.as_ptr(),
        Some(CommitCommand as unsafe extern "C" fn(LPCWSTR, *mut LPWSTR, DWORD, DWORD, DWORD, LPCVOID, *mut BOOL) -> DWORD),
        IDS_HLP_COMMIT,
        IDS_HLP_COMMIT_EX,
        0,
        None,
    );
    AddContextCommand(
        pRootContext,
        RUSTCONV_WSTR_E9D05A71AD4A.as_ptr(),
        Some(DumpCommand as unsafe extern "C" fn(LPCWSTR, *mut LPWSTR, DWORD, DWORD, DWORD, LPCVOID, *mut BOOL) -> DWORD),
        IDS_HLP_DUMP,
        IDS_HLP_DUMP_EX,
        0,
        None,
    );
    AddContextCommand(
        pRootContext,
        RUSTCONV_WSTR_AB3681E84239.as_ptr(),
        Some(ExecCommand as unsafe extern "C" fn(LPCWSTR, *mut LPWSTR, DWORD, DWORD, DWORD, LPCVOID, *mut BOOL) -> DWORD),
        IDS_HLP_EXEC,
        IDS_HLP_EXEC_EX,
        0,
        None,
    );
    AddContextCommand(
        pRootContext,
        RUSTCONV_WSTR_5C9738DE6AF0.as_ptr(),
        Some(ExitCommand as unsafe extern "C" fn(LPCWSTR, *mut LPWSTR, DWORD, DWORD, DWORD, LPCVOID, *mut BOOL) -> DWORD),
        IDS_HLP_EXIT,
        IDS_HLP_EXIT_EX,
        0,
        None,
    );
    AddContextCommand(
        pRootContext,
        RUSTCONV_WSTR_6509121F99F7.as_ptr(),
        None,
        IDS_HLP_HELP,
        IDS_HLP_HELP_EX,
        0,
        None,
    );
    AddContextCommand(
        pRootContext,
        RUSTCONV_WSTR_12394F0C4911.as_ptr(),
        Some(OfflineCommand as unsafe extern "C" fn(LPCWSTR, *mut LPWSTR, DWORD, DWORD, DWORD, LPCVOID, *mut BOOL) -> DWORD),
        IDS_HLP_OFFLINE,
        IDS_HLP_OFFLINE_EX,
        0,
        None,
    );
    AddContextCommand(
        pRootContext,
        RUSTCONV_WSTR_7D01DF642FC6.as_ptr(),
        Some(OnlineCommand as unsafe extern "C" fn(LPCWSTR, *mut LPWSTR, DWORD, DWORD, DWORD, LPCVOID, *mut BOOL) -> DWORD),
        IDS_HLP_ONLINE,
        IDS_HLP_ONLINE_EX,
        0,
        None,
    );
    AddContextCommand(
        pRootContext,
        RUSTCONV_WSTR_9BE6D558E725.as_ptr(),
        Some(PopdCommand as unsafe extern "C" fn(LPCWSTR, *mut LPWSTR, DWORD, DWORD, DWORD, LPCVOID, *mut BOOL) -> DWORD),
        IDS_HLP_POPD,
        IDS_HLP_POPD_EX,
        0,
        None,
    );
    AddContextCommand(
        pRootContext,
        RUSTCONV_WSTR_EAFD7A71F5E7.as_ptr(),
        Some(PushdCommand as unsafe extern "C" fn(LPCWSTR, *mut LPWSTR, DWORD, DWORD, DWORD, LPCVOID, *mut BOOL) -> DWORD),
        IDS_HLP_PUSHD,
        IDS_HLP_PUSHD_EX,
        0,
        None,
    );
    AddContextCommand(
        pRootContext,
        RUSTCONV_WSTR_56F6C4E7F7D5.as_ptr(),
        Some(ExitCommand as unsafe extern "C" fn(LPCWSTR, *mut LPWSTR, DWORD, DWORD, DWORD, LPCVOID, *mut BOOL) -> DWORD),
        IDS_HLP_EXIT,
        IDS_HLP_EXIT_EX,
        0,
        None,
    );
    AddContextCommand(
        pRootContext,
        RUSTCONV_WSTR_31DF522FC4C4.as_ptr(),
        Some(UnaliasCommand as unsafe extern "C" fn(LPCWSTR, *mut LPWSTR, DWORD, DWORD, DWORD, LPCVOID, *mut BOOL) -> DWORD),
        IDS_HLP_UNALIAS,
        IDS_HLP_UNALIAS_EX,
        0,
        None,
    );

    pGroup = AddCommandGroup(
        pRootContext,
        RUSTCONV_WSTR_09C488271038.as_ptr(),
        IDS_HLP_GROUP_ADD,
        0,
        None,
    );
    if !pGroup.is_null() {
        AddGroupCommand(
            pGroup,
            RUSTCONV_WSTR_B94677D1EF59.as_ptr(),
            Some(AddHelperCommand as unsafe extern "C" fn(LPCWSTR, *mut LPWSTR, DWORD, DWORD, DWORD, LPCVOID, *mut BOOL) -> DWORD),
            IDS_HLP_ADD_HELPER,
            IDS_HLP_ADD_HELPER_EX,
            CMD_FLAG_LOCAL,
            None,
        );
    }

    pGroup = AddCommandGroup(
        pRootContext,
        RUSTCONV_WSTR_08F79C40983C.as_ptr(),
        IDS_HLP_GROUP_DELETE,
        0,
        None,
    );
    if !pGroup.is_null() {
        AddGroupCommand(
            pGroup,
            RUSTCONV_WSTR_B94677D1EF59.as_ptr(),
            Some(DeleteHelperCommand as unsafe extern "C" fn(LPCWSTR, *mut LPWSTR, DWORD, DWORD, DWORD, LPCVOID, *mut BOOL) -> DWORD),
            IDS_HLP_DEL_HELPER,
            IDS_HLP_DEL_HELPER_EX,
            CMD_FLAG_LOCAL,
            None,
        );
    }

    pGroup = AddCommandGroup(
        pRootContext,
        RUSTCONV_WSTR_FCABAF3CA758.as_ptr(),
        IDS_HLP_GROUP_SET,
        0,
        None,
    );
    if !pGroup.is_null() {
        AddGroupCommand(
            pGroup,
            RUSTCONV_WSTR_65A415E58FC2.as_ptr(),
            Some(SetMachineCommand as unsafe extern "C" fn(LPCWSTR, *mut LPWSTR, DWORD, DWORD, DWORD, LPCVOID, *mut BOOL) -> DWORD),
            IDS_HLP_SET_MACHINE,
            IDS_HLP_SET_MACHINE_EX,
            CMD_FLAG_ONLINE,
            None,
        );
        AddGroupCommand(
            pGroup,
            RUSTCONV_WSTR_433DA8124B16.as_ptr(),
            Some(SetModeCommand as unsafe extern "C" fn(LPCWSTR, *mut LPWSTR, DWORD, DWORD, DWORD, LPCVOID, *mut BOOL) -> DWORD),
            IDS_HLP_SET_MODE,
            IDS_HLP_SET_MODE_EX,
            0,
            None,
        );
    }

    pGroup = AddCommandGroup(
        pRootContext,
        RUSTCONV_WSTR_0EA8BD6F19AB.as_ptr(),
        IDS_HLP_GROUP_SHOW,
        0,
        None,
    );
    if !pGroup.is_null() {
        AddGroupCommand(
            pGroup,
            RUSTCONV_WSTR_A0E70D05E916.as_ptr(),
            Some(ShowAliasCommand as unsafe extern "C" fn(LPCWSTR, *mut LPWSTR, DWORD, DWORD, DWORD, LPCVOID, *mut BOOL) -> DWORD),
            IDS_HLP_SHOW_ALIAS,
            IDS_HLP_SHOW_ALIAS_EX,
            0,
            None,
        );
        AddGroupCommand(
            pGroup,
            RUSTCONV_WSTR_B94677D1EF59.as_ptr(),
            Some(ShowHelperCommand as unsafe extern "C" fn(LPCWSTR, *mut PWSTR, DWORD, DWORD, DWORD, LPCVOID, *mut BOOL) -> DWORD),
            IDS_HLP_SHOW_HELPER,
            IDS_HLP_SHOW_HELPER_EX,
            0,
            None,
        );
        AddGroupCommand(
            pGroup,
            RUSTCONV_WSTR_433DA8124B16.as_ptr(),
            Some(ShowModeCommand as unsafe extern "C" fn(LPCWSTR, *mut LPWSTR, DWORD, DWORD, DWORD, LPCVOID, *mut BOOL) -> DWORD),
            IDS_HLP_SHOW_MODE,
            IDS_HLP_SHOW_MODE_EX,
            0,
            None,
        );
    }

    pCurrentContext = pRootContext;

    return TRUE;
}

unsafe extern "C" fn FindSubContextByGuid(
    mut pContext: PCONTEXT_ENTRY,
    mut pGuid: *const GUID,
) -> PCONTEXT_ENTRY {
    let mut pResultContext: PCONTEXT_ENTRY = core::ptr::null_mut();
    let mut pSubContext: PCONTEXT_ENTRY = core::ptr::null_mut();
    DPRINT!("FindSubContextByGuid(%p)\n", pContext);
    DPRINT!("%lx <--> %lx\n", (*pContext).Guid.Data1, (*pGuid).Data1);
    if IsEqualGUID(&raw mut (*pContext).Guid, pGuid) != 0 {
        DPRINT!("Found!\n");
        return pContext;
    }
    pSubContext = (*pContext).pSubContextHead;
    while !pSubContext.is_null() {
        pResultContext = FindSubContextByGuid(pSubContext, pGuid);
        if !pResultContext.is_null() {
            return pResultContext;
        }
        pSubContext = (*pSubContext).pNext;
    }
    return core::ptr::null_mut();
}
#[no_mangle]
pub unsafe extern "C" fn FindContextByGuid(pGuid: *const GUID) -> PCONTEXT_ENTRY {
    if pRootContext.is_null() {
        return core::ptr::null_mut();
    }
    return FindSubContextByGuid(pRootContext, pGuid);
}

#[no_mangle]
pub unsafe extern "system" fn RegisterContext(pChildContext: *const NS_CONTEXT_ATTRIBUTES) -> DWORD {
    let mut pHelper: PHELPER_ENTRY;
    let mut pContext: PCONTEXT_ENTRY;
    let mut pParentContext: PCONTEXT_ENTRY;
    let mut pGroup: PCOMMAND_GROUP;
    let mut i: DWORD;
    let mut j: DWORD;
    let mut dwError: DWORD = ERROR_SUCCESS;

    DPRINT!("RegisterContext(%p)\n", pChildContext);
    if pChildContext.is_null() {
        DPRINT1!("Invalid child context!\n");
        return ERROR_INVALID_PARAMETER;
    }

    if ((*pChildContext).pwszContext.is_null())
        || (wcslen((*pChildContext).pwszContext) == 0)
        || ((wcschr((*pChildContext).pwszContext, 0x0020u16) as usize) != 0 as usize)
        || ((wcschr((*pChildContext).pwszContext, 0x003Du16) as usize) != 0 as usize)
    {
        DPRINT1!("Invalid context name!\n");
        return ERROR_INVALID_PARAMETER;
    }

    DPRINT!("Name: %S\n", (*pChildContext).pwszContext);
    DPRINT!("Groups: %lu\n", (*pChildContext).ulNumGroups);
    DPRINT!("Top commands: %lu\n", (*pChildContext).ulNumTopCmds);

    pHelper = FindHelper(&raw const (*pChildContext).guidHelper, pHelperListHead);
    DPRINT!("Helper %p\n", pHelper);
    pParentContext = pRootContext;
    if !pHelper.is_null() {
        pParentContext = FindContextByGuid(&raw mut (*pHelper).ParentHelperGuid);
        DPRINT!("pParentContext %p\n", pParentContext);
        if pParentContext.is_null() {
            pParentContext = pRootContext;
        }
    }

    pContext = AddContext(
        pParentContext,
        (*pChildContext).pwszContext as PWSTR,
        &raw const (*pChildContext).guidHelper as *mut GUID,
    );
    if !pContext.is_null() {
        (*pContext).pfnCommitFn = (*pChildContext).pfnCommitFn;
        (*pContext).pfnDumpFn = (*pChildContext).pfnDumpFn;
        (*pContext).pfnConnectFn = (*pChildContext).pfnConnectFn;
        (*pContext).pfnOsVersionCheck = (*pChildContext).pfnOsVersionCheck;
        (*pContext).dwFlags = (*pChildContext).dwFlags;
        (*pContext).ulPriority = if ((*pChildContext).dwFlags & CMD_FLAG_PRIORITY) != 0 {
            (*pChildContext).ulPriority
        } else {
            DEFAULT_CONTEXT_PRIORITY
        };

        if (!pHelper.is_null()) && (!(*pHelper).pDllEntry.is_null()) {
            (*pContext).hModule = (*(*pHelper).pDllEntry).hModule;
        }

        i = 0;
        while i < (*pChildContext).ulNumTopCmds {
            AddContextCommand(
                pContext,
                (*(*pChildContext).pTopCmds.offset(i as isize)).pwszCmdToken,
                (*(*pChildContext).pTopCmds.offset(i as isize)).pfnCmdHandler,
                (*(*pChildContext).pTopCmds.offset(i as isize)).dwShortCmdHelpToken,
                (*(*pChildContext).pTopCmds.offset(i as isize)).dwCmdHlpToken,
                (*(*pChildContext).pTopCmds.offset(i as isize)).dwFlags,
                (*(*pChildContext).pTopCmds.offset(i as isize)).pOsVersionCheck,
            );
            i = i.wrapping_add(1);
        }

        /* Add command groups */
        i = 0;
        while i < (*pChildContext).ulNumGroups {
            pGroup = AddCommandGroup(
                pContext,
                (*(*pChildContext).pCmdGroups.offset(i as isize)).pwszCmdGroupToken,
                (*(*pChildContext).pCmdGroups.offset(i as isize)).dwShortCmdHelpToken,
                (*(*pChildContext).pCmdGroups.offset(i as isize)).dwFlags,
                (*(*pChildContext).pCmdGroups.offset(i as isize)).pOsVersionCheck,
            );
            if !pGroup.is_null() {
                j = 0;
                while j < (*(*pChildContext).pCmdGroups.offset(i as isize)).ulCmdGroupSize {
                    AddGroupCommand(
                        pGroup,
                        (*(*(*pChildContext).pCmdGroups.offset(i as isize)).pCmdGroup.offset(j as isize)).pwszCmdToken,
                        (*(*(*pChildContext).pCmdGroups.offset(i as isize)).pCmdGroup.offset(j as isize)).pfnCmdHandler,
                        (*(*(*pChildContext).pCmdGroups.offset(i as isize)).pCmdGroup.offset(j as isize)).dwShortCmdHelpToken,
                        (*(*(*pChildContext).pCmdGroups.offset(i as isize)).pCmdGroup.offset(j as isize)).dwCmdHlpToken,
                        (*(*(*pChildContext).pCmdGroups.offset(i as isize)).pCmdGroup.offset(j as isize)).dwFlags,
                        (*(*(*pChildContext).pCmdGroups.offset(i as isize)).pCmdGroup.offset(j as isize)).pOsVersionCheck,
                    );
                    j = j.wrapping_add(1);
                }
            }
            i = i.wrapping_add(1);
        }

        if let Some(pfnConnectFn) = (*pContext).pfnConnectFn {
            dwError = pfnConnectFn(g_pszMachine as LPCWSTR);
            DPRINT!("pfnConnectFn(%S) returned %lu\n", g_pszMachine, dwError);
        }
    }

    return dwError;
}

#[no_mangle]
pub unsafe extern "C" fn CleanupContext() {}
