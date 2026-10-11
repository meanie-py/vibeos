use crate::netsh::wmi::CheckOsVersion;
use crate::netsh::context::g_bOnline;
use crate::netsh::context::g_pszMachine;
use crate::netsh::context::pCurrentContext;
use crate::netsh::context::pRootContext;
// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_075DE2B906DB: [u16; 2] = [0x000Au16, 0x0000];
pub static RUSTCONV_WSTR_447438009695: [u16; 6] = [0x0025u16, 0x0073, 0x0020, 0x0025, 0x0073, 0x0000];
pub static RUSTCONV_WSTR_62F97C5A1C97: [u16; 38] = [
    0x0025u16, 0x002D, 0x0031, 0x0035, 0x0073, 0x0020, 0x002D, 0x0020, 0x0043, 0x0068,
    0x0061, 0x006E, 0x0067, 0x0065, 0x0073, 0x0020, 0x0074, 0x006F, 0x0020, 0x0074,
    0x0068, 0x0065, 0x0020, 0x0022, 0x0025, 0x0073, 0x0022, 0x0020, 0x0063, 0x006F,
    0x006E, 0x0074, 0x0065, 0x0078, 0x0074, 0x002E, 0x000A, 0x0000,
];
pub static RUSTCONV_WSTR_69D435449EDE: [u16; 7] = [0x0025u16, 0x0073, 0x0025, 0x0073, 0x0025, 0x0073, 0x0000];
pub static RUSTCONV_WSTR_8D71B3FAAB82: [u16; 2] = [0x0020u16, 0x0000];
pub static RUSTCONV_WSTR_93729641F56A: [u16; 11] = [
    0x0025u16, 0x002D, 0x0031, 0x0035, 0x0073, 0x0020, 0x002D, 0x0020, 0x0025, 0x0073,
    0x0000,
];
pub static RUSTCONV_WSTR_96A296D224F2: [u16; 1] = [0x0000u16];
pub static RUSTCONV_WSTR_DEEFE3BC3DD3: [u16; 4] = [0x0020u16, 0x0025, 0x0073, 0x0000];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// #define NDEBUG (empty object-like macro, no value to emit)

pub const HUGE_HELP_BUFFER_SIZE: usize = 2048;

pub const SMALL_HELP_BUFFER_SIZE: usize = 160;

pub const TINY_HELP_BUFFER_SIZE: usize = 80;

// typedef enum { Command, Group, SubContext } HELP_TYPE, *PHELP_TYPE;
pub type HELP_TYPE = i32;
pub type PHELP_TYPE = *mut HELP_TYPE;
pub const Command: HELP_TYPE = 0;
pub const Group: HELP_TYPE = 1;
pub const SubContext: HELP_TYPE = 2;

// Anonymous union member of HELP_ENTRY, given a name so it can be referenced.
#[repr(C)]
#[derive(Clone, Copy)]
pub union HELP_ENTRY_Pointer {
    pub pCommand: PCOMMAND_ENTRY,
    pub pGroup: PCOMMAND_GROUP,
    pub pSubContext: PCONTEXT_ENTRY,
}

// typedef struct { ... } HELP_ENTRY, *PHELP_ENTRY;
#[repr(C)]
#[derive(Clone, Copy)]
pub struct HELP_ENTRY {
    pub Type: HELP_TYPE,
    pub pszCommand: PWSTR,
    pub dwHelpId: DWORD,
    pub Pointer: HELP_ENTRY_Pointer,
}
pub type PHELP_ENTRY = *mut HELP_ENTRY;

unsafe extern "C" fn GetContextFullName(
    pContext: PCONTEXT_ENTRY,
    pszBuffer: LPWSTR,
    cchLength: DWORD,
) {
    if pContext != pRootContext {
        GetContextFullName((*pContext).pParentContext, pszBuffer, cchLength);
        wcscat(pszBuffer, RUSTCONV_WSTR_8D71B3FAAB82.as_ptr());
        wcscat(pszBuffer, (*pContext).pszContextName);
    } else {
        wcscpy(pszBuffer, (*pContext).pszContextName);
    }
}

unsafe extern "C" fn PrintCurrentContextHeader(mut pContext: PCONTEXT_ENTRY) {
    if pContext == pCurrentContext {
        ConResPrintf(&raw mut csStdOut, IDS_THIS_COMMANDS as UINT);
    } else {
        let mut szBuffer: [WCHAR; SMALL_HELP_BUFFER_SIZE] = [0; SMALL_HELP_BUFFER_SIZE];
        GetContextFullName(
            pContext,
            szBuffer.as_mut_ptr() as LPWSTR,
            _countof!(szBuffer) as DWORD,
        );
        ConResPrintf(
            &raw mut csStdOut,
            IDS_CONTEXT_COMMANDS as UINT,
            szBuffer.as_mut_ptr() as *mut WCHAR,
        );
    }
}

unsafe extern "C" fn PrintShortGroupCommands(
    pContext: PCONTEXT_ENTRY,
    pGroup: PCOMMAND_GROUP,
) {
    let mut pCommand: PCOMMAND_ENTRY;
    let mut szBuffer1: [WCHAR; TINY_HELP_BUFFER_SIZE] = [0; TINY_HELP_BUFFER_SIZE];
    let mut szBuffer2: [WCHAR; SMALL_HELP_BUFFER_SIZE] = [0; SMALL_HELP_BUFFER_SIZE];

    pCommand = (*pGroup).pCommandListHead;
    while !pCommand.is_null() {
        'next: {
            DPRINT!("CheckVersion (Command) %S %S", (*pGroup).pwszCmdGroupToken, (*pCommand).pwszCmdToken);
            if CheckOsVersion((*pCommand).pfnOsVersionCheck) == 0 {
                break 'next;
            }

            if ((((*pCommand).dwFlags & CMD_FLAG_LOCAL) != 0 && !g_pszMachine.is_null())
                || (((*pCommand).dwFlags & CMD_FLAG_ONLINE) != 0 && g_bOnline == (FALSE as i32))
                || ((*pCommand).dwFlags & CMD_FLAG_HIDDEN) != 0)
            {
                break 'next;
            }

            _swprintf(
                core::ptr::addr_of_mut!(szBuffer1) as *mut WCHAR,
                RUSTCONV_WSTR_447438009695.as_ptr(),
                (*pGroup).pwszCmdGroupToken,
                (*pCommand).pwszCmdToken,
            );
            LoadStringW(
                ((*pContext).hModule).cast(),
                (*pCommand).dwShortCmdHelpToken,
                core::ptr::addr_of_mut!(szBuffer2) as LPWSTR,
                _countof!(szBuffer2) as INT,
            );

            ConPrintf(
                StdOut,
                RUSTCONV_WSTR_93729641F56A.as_ptr(),
                core::ptr::addr_of_mut!(szBuffer1) as *mut WCHAR,
                core::ptr::addr_of_mut!(szBuffer2) as *mut WCHAR,
            );
        }
        pCommand = (*pCommand).pNext;
    }
}

unsafe extern "C" fn HelpCompare(p1: *const ::core::ffi::c_void, p2: *const ::core::ffi::c_void) -> ::core::ffi::c_int {
    return _wcsicmp(
        (*(p1 as PHELP_ENTRY)).pszCommand as *const wchar_t,
        (*(p2 as PHELP_ENTRY)).pszCommand as *const wchar_t,
    );
}

unsafe extern "C" fn PrintContext(mut pContext: PCONTEXT_ENTRY) {
    let mut pCommand: PCOMMAND_ENTRY = ::core::ptr::null_mut::<_COMMAND_ENTRY>();
    let mut pGroup: PCOMMAND_GROUP = ::core::ptr::null_mut::<_COMMAND_GROUP>();
    let mut pSubContext: PCONTEXT_ENTRY = ::core::ptr::null_mut::<_CONTEXT_ENTRY>();
    let mut pHelpArray: PHELP_ENTRY = ::core::ptr::null_mut::<HELP_ENTRY>();
    let mut dwCount: DWORD = 0 as DWORD;
    let mut dwIndex: DWORD = 0;
    let mut szBuffer: [WCHAR; SMALL_HELP_BUFFER_SIZE as usize] = [0; SMALL_HELP_BUFFER_SIZE as usize];
    DPRINT!(b"PrintContext()\n\0".as_ptr() as *const i8);
    if pContext != pRootContext {
        PrintContext((*pContext).pParentContext as PCONTEXT_ENTRY);
    }
    PrintCurrentContextHeader(pContext);
    /* Count short commands */
    pCommand = (*pContext).pCommandListHead;
    while !pCommand.is_null() {
        if CheckOsVersion((*pCommand).pfnOsVersionCheck) != 0 {
            if !((((*pCommand).dwFlags & (CMD_FLAG_LOCAL as DWORD)) != 0 && !g_pszMachine.is_null())
                || (((*pCommand).dwFlags & (CMD_FLAG_ONLINE as DWORD)) != 0 && g_bOnline == (FALSE as BOOL))
                || ((*pCommand).dwFlags & (CMD_FLAG_HIDDEN as DWORD)) != 0)
            {
                dwCount = dwCount.wrapping_add(1);
            }
        }
        pCommand = (*pCommand).pNext as PCOMMAND_ENTRY;
    }
    /* Count short groups */
    pGroup = (*pContext).pGroupListHead;
    while !pGroup.is_null() {
        if CheckOsVersion((*pGroup).pfnOsVersionCheck) != 0 {
            if !((((*pGroup).dwFlags & (CMD_FLAG_LOCAL as DWORD)) != 0 && !g_pszMachine.is_null())
                || (((*pGroup).dwFlags & (CMD_FLAG_ONLINE as DWORD)) != 0 && g_bOnline == (FALSE as BOOL))
                || ((*pGroup).dwFlags & (CMD_FLAG_HIDDEN as DWORD)) != 0)
            {
                dwCount = dwCount.wrapping_add(1);
            }
        }
        pGroup = (*pGroup).pNext as PCOMMAND_GROUP;
    }
    /* Count short subcontexts */
    pSubContext = (*pContext).pSubContextHead as PCONTEXT_ENTRY;
    while !pSubContext.is_null() {
        if CheckOsVersion((*pSubContext).pfnOsVersionCheck) != 0 {
            if !((((*pSubContext).dwFlags & (CMD_FLAG_LOCAL as DWORD)) != 0 && !g_pszMachine.is_null())
                || (((*pSubContext).dwFlags & (CMD_FLAG_ONLINE as DWORD)) != 0 && g_bOnline == (FALSE as BOOL))
                || ((*pSubContext).dwFlags & (CMD_FLAG_HIDDEN as DWORD)) != 0)
            {
                dwCount = dwCount.wrapping_add(1);
            }
        }
        pSubContext = (*pSubContext).pNext as PCONTEXT_ENTRY;
    }
    pHelpArray = HeapAlloc(
        GetProcessHeap(),
        HEAP_ZERO_MEMORY as DWORD,
        (dwCount as SIZE_T).wrapping_mul(core::mem::size_of::<HELP_ENTRY>() as SIZE_T),
    ) as PHELP_ENTRY;
    if pHelpArray.is_null() {
        return;
    }
    dwIndex = 0 as DWORD;
    /* Add short commands */
    pCommand = (*pContext).pCommandListHead;
    while !pCommand.is_null() {
        if CheckOsVersion((*pCommand).pfnOsVersionCheck) != 0 {
            if !((((*pCommand).dwFlags & (CMD_FLAG_LOCAL as DWORD)) != 0 && !g_pszMachine.is_null())
                || (((*pCommand).dwFlags & (CMD_FLAG_ONLINE as DWORD)) != 0 && g_bOnline == (FALSE as BOOL))
                || ((*pCommand).dwFlags & (CMD_FLAG_HIDDEN as DWORD)) != 0)
            {
                (*pHelpArray.offset(dwIndex as isize)).Type = Command;
                (*pHelpArray.offset(dwIndex as isize)).pszCommand = (*pCommand).pwszCmdToken;
                (*pHelpArray.offset(dwIndex as isize)).dwHelpId = (*pCommand).dwShortCmdHelpToken;
                (*pHelpArray.offset(dwIndex as isize)).Pointer.pCommand = pCommand;
                dwIndex = dwIndex.wrapping_add(1);
            }
        }
        pCommand = (*pCommand).pNext as PCOMMAND_ENTRY;
    }
    /* Add short groups */
    pGroup = (*pContext).pGroupListHead;
    while !pGroup.is_null() {
        if CheckOsVersion((*pGroup).pfnOsVersionCheck) != 0 {
            if !((((*pGroup).dwFlags & (CMD_FLAG_LOCAL as DWORD)) != 0 && !g_pszMachine.is_null())
                || (((*pGroup).dwFlags & (CMD_FLAG_ONLINE as DWORD)) != 0 && g_bOnline == (FALSE as BOOL))
                || ((*pGroup).dwFlags & (CMD_FLAG_HIDDEN as DWORD)) != 0)
            {
                (*pHelpArray.offset(dwIndex as isize)).Type = Group;
                (*pHelpArray.offset(dwIndex as isize)).pszCommand = (*pGroup).pwszCmdGroupToken;
                (*pHelpArray.offset(dwIndex as isize)).dwHelpId = (*pGroup).dwShortCmdHelpToken;
                (*pHelpArray.offset(dwIndex as isize)).Pointer.pGroup = pGroup;
                dwIndex = dwIndex.wrapping_add(1);
            }
        }
        pGroup = (*pGroup).pNext as PCOMMAND_GROUP;
    }
    /* Count short subcontexts */
    pSubContext = (*pContext).pSubContextHead as PCONTEXT_ENTRY;
    while !pSubContext.is_null() {
        if CheckOsVersion((*pSubContext).pfnOsVersionCheck) != 0 {
            if !((((*pSubContext).dwFlags & (CMD_FLAG_LOCAL as DWORD)) != 0 && !g_pszMachine.is_null())
                || (((*pSubContext).dwFlags & (CMD_FLAG_ONLINE as DWORD)) != 0 && g_bOnline == (FALSE as BOOL))
                || ((*pSubContext).dwFlags & (CMD_FLAG_HIDDEN as DWORD)) != 0)
            {
                (*pHelpArray.offset(dwIndex as isize)).Type = SubContext;
                (*pHelpArray.offset(dwIndex as isize)).pszCommand = (*pSubContext).pszContextName;
                (*pHelpArray.offset(dwIndex as isize)).Pointer.pSubContext = pSubContext;
                dwIndex = dwIndex.wrapping_add(1);
            }
        }
        pSubContext = (*pSubContext).pNext as PCONTEXT_ENTRY;
    }
    qsort(
        pHelpArray as *mut ::core::ffi::c_void,
        dwCount as size_t,
        core::mem::size_of::<HELP_ENTRY>() as size_t,
        Some(
            HelpCompare
                as unsafe extern "C" fn(
                    *const ::core::ffi::c_void,
                    *const ::core::ffi::c_void,
                ) -> ::core::ffi::c_int,
        ),
    );
    dwIndex = 0 as DWORD;
    while dwIndex < dwCount {
        match (*pHelpArray.offset(dwIndex as isize)).Type as ::core::ffi::c_uint {
            0 => {
                if LoadStringW(
                    ((*pContext).hModule as HINSTANCE).cast(),
                    (*pHelpArray.offset(dwIndex as isize)).dwHelpId as UINT,
                    &raw mut szBuffer as LPWSTR,
                    _countof!(szBuffer) as INT,
                ) == 0 as INT
                {
                    szBuffer[0 as usize] = UNICODE_NULL as WCHAR;
                }
                ConPrintf(
                    &raw mut csStdOut,
                    RUSTCONV_WSTR_93729641F56A.as_ptr(),
                    (*pHelpArray.offset(dwIndex as isize)).pszCommand,
                    &raw mut szBuffer as *mut WCHAR,
                );
            }
            1 => {
                if LoadStringW(
                    ((*pContext).hModule as HINSTANCE).cast(),
                    (*pHelpArray.offset(dwIndex as isize)).dwHelpId as UINT,
                    &raw mut szBuffer as LPWSTR,
                    _countof!(szBuffer) as INT,
                ) == 0 as INT
                {
                    szBuffer[0 as usize] = UNICODE_NULL as WCHAR;
                }
                ConPrintf(
                    &raw mut csStdOut,
                    RUSTCONV_WSTR_93729641F56A.as_ptr(),
                    (*pHelpArray.offset(dwIndex as isize)).pszCommand,
                    &raw mut szBuffer as *mut WCHAR,
                );
            }
            2 => {
                GetContextFullName(
                    (*pHelpArray.offset(dwIndex as isize)).Pointer.pSubContext,
                    &raw mut szBuffer as LPWSTR,
                    _countof!(szBuffer) as DWORD,
                );
                ConPrintf(
                    &raw mut csStdOut,
                    RUSTCONV_WSTR_62F97C5A1C97.as_ptr(),
                    (*pHelpArray.offset(dwIndex as isize)).pszCommand,
                    &raw mut szBuffer as *mut WCHAR,
                );
            }
            _ => {}
        }
        dwIndex = dwIndex.wrapping_add(1);
    }
    if !pHelpArray.is_null() {
        HeapFree(
            GetProcessHeap(),
            0 as DWORD,
            pHelpArray as PVOID,
        );
    }
}

unsafe extern "C" fn SubContextCompare(p1: *const ::core::ffi::c_void, p2: *const ::core::ffi::c_void) -> ::core::ffi::c_int {
    return _wcsicmp(
        (*(p1 as *mut PCONTEXT_ENTRY)).read().pszContextName as *const u16,
        (*(p2 as *mut PCONTEXT_ENTRY)).read().pszContextName as *const u16,
    );
}

unsafe extern "C" fn PrintSubcontexts(pContext: PCONTEXT_ENTRY) {
    let mut pSubContext: PCONTEXT_ENTRY = core::ptr::null_mut();
    let mut pSubContextArray: *mut PCONTEXT_ENTRY = core::ptr::null_mut();
    let mut dwCount: DWORD;
    let mut dwIndex: DWORD;

    if (*pContext).pSubContextHead.is_null() {
        return;
    }

    dwCount = 0;
    pSubContext = (*pContext).pSubContextHead;
    while !pSubContext.is_null() {
        if CheckOsVersion((*pSubContext).pfnOsVersionCheck) != 0 {
            if !(((*pSubContext).dwFlags & CMD_FLAG_LOCAL) != 0 && !g_pszMachine.is_null()
                || ((*pSubContext).dwFlags & CMD_FLAG_ONLINE) != 0 && g_bOnline == (FALSE as i32)
                || ((*pSubContext).dwFlags & CMD_FLAG_HIDDEN) != 0)
            {
                dwCount = dwCount.wrapping_add(1);
            }
        }
        pSubContext = (*pSubContext).pNext;
    }

    pSubContextArray = HeapAlloc(
        GetProcessHeap(),
        0,
        (dwCount as SIZE_T).wrapping_mul(core::mem::size_of::<PCONTEXT_ENTRY>() as SIZE_T),
    ) as *mut PCONTEXT_ENTRY;
    if pSubContextArray.is_null() {
        return;
    }

    dwIndex = 0;
    pSubContext = (*pContext).pSubContextHead;
    while !pSubContext.is_null() {
        if CheckOsVersion((*pSubContext).pfnOsVersionCheck) != 0 {
            if !(((*pSubContext).dwFlags & CMD_FLAG_LOCAL) != 0 && !g_pszMachine.is_null()
                || ((*pSubContext).dwFlags & CMD_FLAG_ONLINE) != 0 && g_bOnline == (FALSE as i32)
                || ((*pSubContext).dwFlags & CMD_FLAG_HIDDEN) != 0)
            {
                *pSubContextArray.offset(dwIndex as isize) = pSubContext;
                dwIndex = dwIndex.wrapping_add(1);
            }
        }
        pSubContext = (*pSubContext).pNext;
    }

    qsort(
        pSubContextArray as *mut core::ffi::c_void,
        dwCount as size_t,
        core::mem::size_of::<PCONTEXT_ENTRY>() as size_t,
        Some(
            SubContextCompare
                as unsafe extern "C" fn(
                    *const core::ffi::c_void,
                    *const core::ffi::c_void,
                ) -> core::ffi::c_int,
        ),
    );

    ConResPrintf(StdOut, IDS_SUBCONTEXT_HEADER);
    dwIndex = 0;
    while dwIndex < dwCount {
        ConPrintf(
            StdOut,
            RUSTCONV_WSTR_DEEFE3BC3DD3.as_ptr(),
            (*(*pSubContextArray.offset(dwIndex as isize))).pszContextName,
        );
        dwIndex = dwIndex.wrapping_add(1);
    }
    ConPuts(StdOut, RUSTCONV_WSTR_075DE2B906DB.as_ptr());

    HeapFree(
        GetProcessHeap(),
        0,
        pSubContextArray as PVOID,
    );
}
#[no_mangle]
pub unsafe extern "C" fn PrintCommandHelp(pContext: PCONTEXT_ENTRY, pGroup: PCOMMAND_GROUP, pCommand: PCOMMAND_ENTRY) {
    let mut pszCommandBuffer: LPWSTR = core::ptr::null_mut();
    let mut Args: [DWORD_PTR; 2] = [0; 2];
    let mut dwLength: DWORD = 1;

    DPRINT!("PrintCommandHelp(%p %p %p)\n", pContext, pGroup, pCommand);

    if CheckOsVersion((*pCommand).pfnOsVersionCheck) == 0 {
        return;
    }

    if (((*pCommand).dwFlags & CMD_FLAG_LOCAL) != 0 && !g_pszMachine.is_null())
        || (((*pCommand).dwFlags & CMD_FLAG_ONLINE) != 0 && g_bOnline == (FALSE as i32))
        || ((*pCommand).dwFlags & CMD_FLAG_HIDDEN) != 0
    {
        return;
    }

    dwLength = dwLength.wrapping_add(
        wcslen((*pCommand).pwszCmdToken as *const u16) as DWORD,
    );
    if !pGroup.is_null() {
        dwLength = dwLength.wrapping_add(
            (wcslen((*pGroup).pwszCmdGroupToken as *const u16) as DWORD)
                .wrapping_add(1 as DWORD),
        );
    }

    pszCommandBuffer = HeapAlloc(
        GetProcessHeap(),
        0,
        (dwLength as SIZE_T).wrapping_mul(core::mem::size_of::<WCHAR>() as SIZE_T),
    ) as LPWSTR;
    if pszCommandBuffer.is_null() {
        return;
    }

    _swprintf(
        pszCommandBuffer as *mut u16,
        RUSTCONV_WSTR_69D435449EDE.as_ptr(),
        if !pGroup.is_null() {
            (*pGroup).pwszCmdGroupToken as *const u16
        } else {
            RUSTCONV_WSTR_96A296D224F2.as_ptr()
        },
        if !pGroup.is_null() {
            RUSTCONV_WSTR_8D71B3FAAB82.as_ptr()
        } else {
            RUSTCONV_WSTR_96A296D224F2.as_ptr()
        },
        (*pCommand).pwszCmdToken,
    );

    Args[0] = pszCommandBuffer as DWORD_PTR;
    Args[1] = core::ptr::null_mut::<u8>() as DWORD_PTR;

    ConResMsgPrintfExV(
        core::ptr::addr_of_mut!(csStdOut),
        (*pContext).hModule as HINSTANCE,
        FORMAT_MESSAGE_ARGUMENT_ARRAY,
        (*pCommand).dwCmdHlpToken,
        LANG_USER_DEFAULT,
        core::ptr::addr_of_mut!(Args) as *mut va_list,
    );
    ConPuts(core::ptr::addr_of_mut!(csStdOut), RUSTCONV_WSTR_075DE2B906DB.as_ptr());

    HeapFree(
        GetProcessHeap(),
        0,
        pszCommandBuffer as PVOID,
    );
}
#[no_mangle]
pub unsafe extern "C" fn PrintGroupHelp(pContext: PCONTEXT_ENTRY, pszGroupName: LPWSTR, bRecurse: BOOL) {
    let mut pGroup: PCOMMAND_GROUP = core::ptr::null_mut();

    if bRecurse != 0 {
        if pContext != pRootContext {
            PrintGroupHelp((*pContext).pParentContext as PCONTEXT_ENTRY, pszGroupName, bRecurse);
        }
    }

    pGroup = (*pContext).pGroupListHead;
    while !pGroup.is_null() {
        if _wcsicmp(
            pszGroupName as *const u16,
            (*pGroup).pwszCmdGroupToken as *const u16,
        ) == 0
        {
            PrintCurrentContextHeader(pContext);
            PrintShortGroupCommands(pContext, pGroup);
        }
        pGroup = (*pGroup).pNext as PCOMMAND_GROUP;
    }
}
#[no_mangle]
pub unsafe extern "C" fn PrintContextHelp(pContext: PCONTEXT_ENTRY) {
    ConResPrintf(core::ptr::addr_of_mut!(csStdOut), IDS_HELP_HEADER as UINT);
    PrintContext(pContext);
    PrintSubcontexts(pContext);
    ConResPrintf(core::ptr::addr_of_mut!(csStdOut), IDS_HELP_FOOTER as UINT);
}
