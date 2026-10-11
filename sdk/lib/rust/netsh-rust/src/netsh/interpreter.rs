use crate::netsh::help::PrintCommandHelp;
use crate::netsh::help::PrintContextHelp;
use crate::netsh::help::PrintGroupHelp;
use crate::netsh::context::g_bOnline;
use crate::netsh::context::g_pszMachine;
use crate::netsh::context::pCurrentContext;
use crate::netsh::context::pRootContext;
// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use ntdll_sys::nt::*;
pub static RUSTCONV_WSTR_075DE2B906DB: [u16; 2] = [0x000Au16, 0x0000];
pub static RUSTCONV_WSTR_1ABD7BB80B8B: [u16; 6] = [0x005Bu16, 0x0025, 0x0073, 0x005D, 0x0020, 0x0000];
pub static RUSTCONV_WSTR_1B9E0FE7B740: [u16; 2] = [0x003Fu16, 0x0000];
pub static RUSTCONV_WSTR_6509121F99F7: [u16; 5] = [0x0068u16, 0x0065, 0x006C, 0x0070, 0x0000];
pub static RUSTCONV_WSTR_8D71B3FAAB82: [u16; 2] = [0x0020u16, 0x0000];
pub static RUSTCONV_WSTR_EAFD7A71F5E7: [u16; 6] = [0x0070u16, 0x0075, 0x0073, 0x0068, 0x0064, 0x0000];
pub static RUSTCONV_WSTR_FA802ED35611: [u16; 2] = [0x003Eu16, 0x0000];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// NDEBUG is a build-time define with no runtime constant to emit.

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum INTERPRETER_STATE {
    STATE_ANALYZE = 0,
    STATE_COMMAND = 1,
    STATE_GROUP = 2,
    STATE_CONTEXT = 3,
    STATE_PARENT_CONTEXT = 4,
    STATE_DONE = 5,
}

pub const STATE_ANALYZE: i32 = 0;
pub const STATE_COMMAND: i32 = 1;
pub const STATE_GROUP: i32 = 2;
pub const STATE_CONTEXT: i32 = 3;
pub const STATE_PARENT_CONTEXT: i32 = 4;
pub const STATE_DONE: i32 = 5;

unsafe extern "C" fn GetGroupCommand(
    mut pszCommand: PWSTR,
    mut pGroup: PCOMMAND_GROUP,
) -> PCOMMAND_ENTRY {
    let mut pCommand: PCOMMAND_ENTRY = core::ptr::null_mut();
    DPRINT!("GetGroupCommand(%S %p)\n", pszCommand, pGroup);
    if pszCommand.is_null() {
        return core::ptr::null_mut();
    }
    pCommand = (*pGroup).pCommandListHead;
    while !pCommand.is_null() {
        if MatchToken(pszCommand, (*pCommand).pwszCmdToken) != 0 {
            return pCommand;
        }
        pCommand = (*pCommand).pNext;
    }
    return core::ptr::null_mut();
}

unsafe extern "C" fn GetContextCommand(
    mut pszCommand: PWSTR,
    mut pContext: PCONTEXT_ENTRY,
) -> PCOMMAND_ENTRY {
    let mut pCommand: PCOMMAND_ENTRY = core::ptr::null_mut();

    DPRINT!("GetContextCommand(%S %p)\n", pszCommand, pContext);

    if pszCommand.is_null() {
        return core::ptr::null_mut();
    }

    pCommand = (*pContext).pCommandListHead;
    while !pCommand.is_null() {
        if MatchToken(pszCommand as LPCWSTR, (*pCommand).pwszCmdToken as LPCWSTR) != 0 {
            return pCommand;
        }

        pCommand = (*pCommand).pNext;
    }

    return core::ptr::null_mut();
}

unsafe extern "C" fn GetContextGroup(
    mut pszGroup: PWSTR,
    mut pContext: PCONTEXT_ENTRY,
) -> PCOMMAND_GROUP {
    let mut pGroup: PCOMMAND_GROUP = core::ptr::null_mut();
    DPRINT!("GetContextGroup(%S %p)\n", pszGroup, pContext);

    if pszGroup.is_null() {
        return core::ptr::null_mut();
    }

    pGroup = (*pContext).pGroupListHead;
    while !pGroup.is_null() {
        if MatchToken(pszGroup, (*pGroup).pwszCmdGroupToken) != 0 {
            return pGroup;
        }

        pGroup = (*pGroup).pNext;
    }

    return core::ptr::null_mut();
}

unsafe extern "C" fn GetContextSubContext(
    mut pszContext: PWSTR,
    mut pContext: PCONTEXT_ENTRY,
) -> PCONTEXT_ENTRY {
    let mut pSubContext: PCONTEXT_ENTRY = core::ptr::null_mut::<_CONTEXT_ENTRY>();

    DPRINT!("GetContextSubContext(%S %p)\n", pszContext, pContext);

    if pszContext.is_null() {
        return core::ptr::null_mut::<_CONTEXT_ENTRY>();
    }

    pSubContext = (*pContext).pSubContextHead as PCONTEXT_ENTRY;
    while !pSubContext.is_null() {
        if MatchToken(
            pszContext as LPCWSTR,
            (*pSubContext).pszContextName as LPCWSTR,
        ) != 0
        {
            return pSubContext;
        }

        pSubContext = (*pSubContext).pNext as PCONTEXT_ENTRY;
    }

    return core::ptr::null_mut::<_CONTEXT_ENTRY>();
}

unsafe extern "C" fn InterpretCommand(
    argv: *mut LPWSTR,
    dwArgCount: DWORD,
    bDone: PBOOL,
) -> DWORD {
    let mut pTempContext: PCONTEXT_ENTRY = core::ptr::null_mut();
    let mut pTempSubContext: PCONTEXT_ENTRY = core::ptr::null_mut();
    let mut pGroup: PCOMMAND_GROUP = core::ptr::null_mut();
    let mut pCommand: PCOMMAND_ENTRY = core::ptr::null_mut();
    let mut State: INTERPRETER_STATE = STATE_ANALYZE;
    let mut dwArgIndex: DWORD = 0;
    let mut dwError: DWORD = ERROR_SUCCESS;

    /* If no args provided */
    if dwArgCount == 0 {
        return ERROR_SUCCESS;
    }

    if pCurrentContext.is_null() {
        pCurrentContext = pRootContext;
    }

    /* Check for help keywords */
    if (dwArgCount == 1)
        && ((_wcsicmp(
            *argv.offset(0 as isize) as *const u16,
            RUSTCONV_WSTR_1B9E0FE7B740.as_ptr(),
        ) == 0)
            || (_wcsicmp(
                *argv.offset(0 as isize) as *const u16,
                RUSTCONV_WSTR_6509121F99F7.as_ptr(),
            ) == 0))
    {
        PrintContextHelp(pCurrentContext);
        return ERROR_SUCCESS;
    }

    pTempContext = pCurrentContext;
    while dwArgIndex < dwArgCount {
        match State {
            STATE_ANALYZE => {
                'arm: {
                    DPRINT!("STATE_ANALYZE\n");

                    pCommand = GetContextCommand(
                        *argv.offset(dwArgIndex as isize) as PWSTR,
                        pTempContext,
                    );
                    if !pCommand.is_null() {
                        State = STATE_COMMAND;
                        break 'arm;
                    }

                    pGroup = GetContextGroup(
                        *argv.offset(dwArgIndex as isize) as PWSTR,
                        pTempContext,
                    );
                    if !pGroup.is_null() {
                        State = STATE_GROUP;
                        break 'arm;
                    }

                    pTempSubContext = GetContextSubContext(
                        *argv.offset(dwArgIndex as isize) as PWSTR,
                        pTempContext,
                    );
                    if !pTempSubContext.is_null() {
                        State = STATE_CONTEXT;
                        break 'arm;
                    }

                    State = STATE_PARENT_CONTEXT;
                }
            }

            STATE_COMMAND => {
                'arm: {
                    DPRINT!("STATE_COMMAND\n");

                    if CheckOsVersion((*pCommand).pfnOsVersionCheck) == 0 {
                        DPRINT!("Command: Version check failed!\n");
                        dwError = ERROR_CMD_NOT_FOUND;
                        State = STATE_DONE;
                        break 'arm;
                    }

                    if (((*pCommand).dwFlags & CMD_FLAG_LOCAL) != 0
                        && !g_pszMachine.is_null())
                        || (((*pCommand).dwFlags & CMD_FLAG_ONLINE) != 0
                            && g_bOnline == FALSE)
                    {
                        dwError = ERROR_CMD_NOT_FOUND;
                        State = STATE_DONE;
                        break 'arm;
                    }

                    /* Check for help keywords */
                    if (dwArgIndex.wrapping_add(1)
                        == dwArgCount.wrapping_sub(1))
                        && ((_wcsicmp(
                            *argv.offset(dwArgIndex.wrapping_add(1) as isize)
                                as *const u16,
                            RUSTCONV_WSTR_1B9E0FE7B740.as_ptr(),
                        ) == 0)
                            || (_wcsicmp(
                                *argv.offset(
                                    dwArgIndex.wrapping_add(1) as isize,
                                ) as *const u16,
                                RUSTCONV_WSTR_6509121F99F7.as_ptr(),
                            ) == 0))
                    {
                        PrintCommandHelp(pTempContext, pGroup, pCommand);
                        State = STATE_DONE;
                        break 'arm;
                    }

                    if (*pCommand).pfnCmdHandler.is_some() {
                        dwArgIndex = dwArgIndex.wrapping_add(1);
                        dwError = (*pCommand)
                            .pfnCmdHandler
                            .expect("non-null function pointer")(
                            g_pszMachine as LPCWSTR,
                            argv,
                            dwArgIndex,
                            dwArgCount,
                            0,
                            core::ptr::null_mut(),
                            bDone,
                        );
                        if dwError != ERROR_SUCCESS {
                            if dwError == ERROR_SHOW_USAGE {
                                PrintCommandHelp(pTempContext, pGroup, pCommand);
                                dwError = ERROR_SUPPRESS_OUTPUT;
                            }
                        } else {
                            /* Execute the commands following a pushd command */
                            if (_wcsicmp(
                                (*pCommand).pwszCmdToken as *const u16,
                                RUSTCONV_WSTR_EAFD7A71F5E7.as_ptr(),
                            ) == 0)
                                && (dwArgIndex < dwArgCount)
                            {
                                State = STATE_ANALYZE;
                                break 'arm;
                            }
                        }
                    }

                    State = STATE_DONE;
                }
            }

            STATE_GROUP => {
                'arm: {
                    DPRINT!("STATE_GROUP\n");

                    if CheckOsVersion((*pGroup).pfnOsVersionCheck) == 0 {
                        DPRINT!("Group: Version check failed!\n");
                        dwError = ERROR_CMD_NOT_FOUND;
                        State = STATE_DONE;
                        break 'arm;
                    }

                    if (((*pGroup).dwFlags & CMD_FLAG_LOCAL) != 0
                        && !g_pszMachine.is_null())
                        || (((*pGroup).dwFlags & CMD_FLAG_ONLINE) != 0
                            && g_bOnline == FALSE)
                    {
                        dwError = ERROR_CMD_NOT_FOUND;
                        State = STATE_DONE;
                        break 'arm;
                    }

                    /* Check for group without command */
                    if dwArgIndex == dwArgCount.wrapping_sub(1) {
                        PrintGroupHelp(
                            pTempContext,
                            (*pGroup).pwszCmdGroupToken as LPWSTR,
                            TRUE,
                        );
                        State = STATE_DONE;
                        break 'arm;
                    }

                    /* Check for help keywords */
                    if (dwArgIndex.wrapping_add(1)
                        <= dwArgCount.wrapping_sub(1))
                        && ((_wcsicmp(
                            *argv.offset(dwArgIndex.wrapping_add(1) as isize)
                                as *const u16,
                            RUSTCONV_WSTR_1B9E0FE7B740.as_ptr(),
                        ) == 0)
                            || (_wcsicmp(
                                *argv.offset(
                                    dwArgIndex.wrapping_add(1) as isize,
                                ) as *const u16,
                                RUSTCONV_WSTR_6509121F99F7.as_ptr(),
                            ) == 0))
                    {
                        PrintGroupHelp(
                            pTempContext,
                            (*pGroup).pwszCmdGroupToken as LPWSTR,
                            TRUE,
                        );
                        State = STATE_DONE;
                        break 'arm;
                    }

                    dwArgIndex = dwArgIndex.wrapping_add(1);
                    pCommand = GetGroupCommand(
                        *argv.offset(dwArgIndex as isize) as PWSTR,
                        pGroup,
                    );
                    if !pCommand.is_null() {
                        State = STATE_COMMAND;
                        break 'arm;
                    }

                    dwError = ERROR_CMD_NOT_FOUND;
                    State = STATE_DONE;
                }
            }

            STATE_CONTEXT => {
                'arm: {
                    DPRINT!("STATE_CONTEXT\n");

                    if CheckOsVersion((*pTempSubContext).pfnOsVersionCheck) == 0 {
                        DPRINT!("Context: Version check failed!\n");
                        dwError = ERROR_CMD_NOT_FOUND;
                        State = STATE_DONE;
                        break 'arm;
                    }

                    if (((*pTempSubContext).dwFlags & CMD_FLAG_LOCAL) != 0
                        && !g_pszMachine.is_null())
                        || (((*pTempSubContext).dwFlags & CMD_FLAG_ONLINE) != 0
                            && g_bOnline == FALSE)
                    {
                        dwError = ERROR_CMD_NOT_FOUND;
                        State = STATE_DONE;
                        break 'arm;
                    }

                    if pTempSubContext == pCurrentContext {
                        if dwArgIndex != dwArgCount.wrapping_sub(1) {
                            dwError = ERROR_CMD_NOT_FOUND;
                        }
                        State = STATE_DONE;
                        break 'arm;
                    }

                    if dwArgIndex == dwArgCount.wrapping_sub(1) {
                        DPRINT!("Set current context\n");
                        pCurrentContext = pTempSubContext;
                        if (*pCurrentContext).pfnConnectFn.is_some() {
                            dwError = (*pCurrentContext)
                                .pfnConnectFn
                                .expect("non-null function pointer")(
                                g_pszMachine as LPCWSTR
                            );
                        }
                        State = STATE_DONE;
                        break 'arm;
                    }

                    /* Check for help keywords */
                    if (dwArgIndex.wrapping_add(1)
                        <= dwArgCount.wrapping_sub(1))
                        && ((_wcsicmp(
                            *argv.offset(dwArgIndex.wrapping_add(1) as isize)
                                as *const u16,
                            RUSTCONV_WSTR_1B9E0FE7B740.as_ptr(),
                        ) == 0)
                            || (_wcsicmp(
                                *argv.offset(
                                    dwArgIndex.wrapping_add(1) as isize,
                                ) as *const u16,
                                RUSTCONV_WSTR_6509121F99F7.as_ptr(),
                            ) == 0))
                    {
                        PrintContextHelp(pTempSubContext);
                        State = STATE_DONE;
                        break 'arm;
                    }

                    DPRINT!("Change temorary context\n");
                    pTempContext = pTempSubContext;
                    State = STATE_ANALYZE;
                    dwArgIndex = dwArgIndex.wrapping_add(1);
                }
            }

            STATE_PARENT_CONTEXT => {
                'arm: {
                    DPRINT!("STATE_PARENT_CONTEXT\n");

                    if (*pTempContext).pParentContext.is_null() {
                        dwError = ERROR_CMD_NOT_FOUND;
                        State = STATE_DONE;
                        break 'arm;
                    }

                    DPRINT!("Change temorary context\n");
                    pTempContext = (*pTempContext).pParentContext as PCONTEXT_ENTRY;
                    State = STATE_ANALYZE;
                    dwArgIndex = 0;
                }
            }

            STATE_DONE => {
                DPRINT!("STATE_DONE dwError %ld\n", dwError);
                return dwError;
            }

            _ => {}
        }
    }

    /* Done */
    return ERROR_SUCCESS;
}
#[no_mangle]
pub unsafe extern "C" fn InterpretLine(mut pszInputLine: LPWSTR) -> DWORD {
    let mut args_vector: [LPWSTR; MAX_ARGS_COUNT] = [core::ptr::null_mut::<WCHAR>(); MAX_ARGS_COUNT];
    let mut dwArgCount: DWORD = 0;
    let mut i: DWORD = 0;
    let mut len: DWORD = 0;
    let mut bWhiteSpace: BOOL = TRUE;
    let mut bDone: BOOL = FALSE;
    let mut bInQuotes: BOOL = FALSE;
    let mut ptr: LPWSTR = core::ptr::null_mut::<WCHAR>();
    let mut pStartQuote: LPWSTR = core::ptr::null_mut::<WCHAR>();
    let mut pEndQuote: LPWSTR = core::ptr::null_mut::<WCHAR>();

    memset(
        &raw mut args_vector as *mut LPWSTR as *mut ::core::ffi::c_void,
        0,
        core::mem::size_of::<[LPWSTR; MAX_ARGS_COUNT]>(),
    );

    ptr = pszInputLine;
    while *ptr != 0 {
        if *ptr == 0x0022u16 {
            bInQuotes = if bInQuotes != 0 { FALSE } else { TRUE };
        }

        if (iswspace(*ptr) != 0 && bInQuotes == FALSE) || *ptr == 0x000Au16 {
            *ptr = 0;
            bWhiteSpace = TRUE;
        } else {
            if bWhiteSpace != FALSE && dwArgCount < MAX_ARGS_COUNT {
                args_vector[dwArgCount as usize] = ptr;
                dwArgCount = dwArgCount.wrapping_add(1);
            }

            bWhiteSpace = FALSE;
        }

        ptr = ptr.offset(1);
    }

    /* Remove quotation marks */
    i = 0;
    while i < dwArgCount {
        pStartQuote = wcschr(args_vector[i as usize] as *const WCHAR, 0x0022u16) as LPWSTR;
        if !pStartQuote.is_null() {
            pEndQuote = wcschr(
                pStartQuote.offset(1 as isize) as *const WCHAR,
                0x0022u16,
            ) as LPWSTR;
            if !pEndQuote.is_null() {
                len = pEndQuote.offset_from(pStartQuote) as DWORD;
                memmove(
                    pStartQuote as *mut ::core::ffi::c_void,
                    pStartQuote.offset(1 as isize) as *const ::core::ffi::c_void,
                    (len.wrapping_sub(1) as usize)
                        .wrapping_mul(core::mem::size_of::<WCHAR>()),
                );
                *pEndQuote.offset(-(1 as isize)) = UNICODE_NULL;
            }
        }
        i = i.wrapping_add(1);
    }

    return InterpretCommand(
        &raw mut args_vector as *mut LPWSTR,
        dwArgCount,
        &raw mut bDone,
    );
}
#[no_mangle]
pub unsafe extern "C" fn PrintPrompt(pContext: PCONTEXT_ENTRY) {
    if pContext != pRootContext {
        PrintPrompt((*pContext).pParentContext);
        ConPuts(StdOut, RUSTCONV_WSTR_8D71B3FAAB82.as_ptr());
    }

    ConPuts(StdOut, (*pContext).pszContextName);
}
#[no_mangle]
pub unsafe extern "C" fn InterpretInteractive() {
    let mut input_line: [WCHAR; MAX_STRING_SIZE] = [0; MAX_STRING_SIZE];
    let mut args_vector: [LPWSTR; MAX_ARGS_COUNT] = [core::ptr::null_mut(); MAX_ARGS_COUNT];
    let mut dwArgCount: DWORD = 0;
    let mut i: DWORD = 0;
    let mut len: DWORD = 0;
    let mut bWhiteSpace: BOOL = TRUE;
    let mut bDone: BOOL = FALSE;
    let mut bInQuotes: BOOL = FALSE;
    let mut ptr: LPWSTR = core::ptr::null_mut();
    let mut pStartQuote: LPWSTR = core::ptr::null_mut();
    let mut pEndQuote: LPWSTR = core::ptr::null_mut();
    let mut dwError: DWORD = ERROR_SUCCESS;

    loop {
        dwArgCount = 0;
        memset(
            core::ptr::addr_of_mut!(args_vector) as *mut ::core::ffi::c_void,
            0,
            core::mem::size_of::<[LPWSTR; MAX_ARGS_COUNT]>(),
        );

        /* Shown just before the input where the user places commands */
        if !g_pszMachine.is_null() {
            ConPrintf(StdOut, RUSTCONV_WSTR_1ABD7BB80B8B.as_ptr(), g_pszMachine);
        }
        PrintPrompt(pCurrentContext);
        ConPuts(StdOut, RUSTCONV_WSTR_FA802ED35611.as_ptr());

        /* Get input from the user. */
        fgetws(
            core::ptr::addr_of_mut!(input_line) as *mut WCHAR,
            MAX_STRING_SIZE,
            stdin,
        );

        ptr = core::ptr::addr_of_mut!(input_line) as *mut WCHAR as LPWSTR;
        while *ptr != 0 {
            if *ptr == 0x0022u16 {
                bInQuotes = if bInQuotes != 0 {
                    FALSE
                } else {
                    TRUE
                };
            }

            if (iswspace(*ptr as u32) != 0 && bInQuotes == FALSE)
                || *ptr == 0x000Au16
            {
                *ptr = UNICODE_NULL;
                bWhiteSpace = TRUE;
            } else {
                if (bWhiteSpace != FALSE) && (dwArgCount < MAX_ARGS_COUNT) {
                    args_vector[dwArgCount as usize] = ptr;
                    dwArgCount = dwArgCount.wrapping_add(1);
                }
                bWhiteSpace = FALSE;
            }
            ptr = ptr.offset(1);
        }

        /* Remove quotation marks */
        i = 0;
        while i < dwArgCount {
            pStartQuote = wcschr(
                args_vector[i as usize] as *const u16,
                0x0022u16,
            ) as LPWSTR;
            if !pStartQuote.is_null() {
                pEndQuote = wcschr(
                    pStartQuote.offset(1 as isize) as *const u16,
                    0x0022u16,
                ) as LPWSTR;
                if !pEndQuote.is_null() {
                    len = pEndQuote.offset_from(pStartQuote) as ::core::ffi::c_longlong as DWORD;
                    memmove(
                        pStartQuote as *mut ::core::ffi::c_void,
                        pStartQuote.offset(1 as isize) as *const ::core::ffi::c_void,
                        (len.wrapping_sub(1 as DWORD) as usize)
                            .wrapping_mul(core::mem::size_of::<WCHAR>()),
                    );
                    *pEndQuote.offset(-(1 as isize)) = UNICODE_NULL;
                }
            }
            i = i.wrapping_add(1);
        }

        dwError = InterpretCommand(
            core::ptr::addr_of_mut!(args_vector) as *mut LPWSTR,
            dwArgCount,
            core::ptr::addr_of_mut!(bDone),
        );
        if (dwError != ERROR_SUCCESS) && (dwError != ERROR_SUPPRESS_OUTPUT) {
            let mut pszCommandString: PWSTR = MergeStrings(
                core::ptr::addr_of_mut!(args_vector) as *mut LPWSTR,
                dwArgCount as UINT,
            ) as PWSTR;
            PrintError(core::ptr::null_mut(), dwError, pszCommandString);
            HeapFree(
                GetProcessHeap(),
                0,
                pszCommandString as PVOID,
            );
        }
        if dwError != ERROR_SUPPRESS_OUTPUT {
            ConPuts(StdOut, RUSTCONV_WSTR_075DE2B906DB.as_ptr());
        }

        if bDone != 0 {
            break;
        }
    }
}
