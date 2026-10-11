use crate::netsh::context::CleanupContext;
use crate::netsh::context::CreateRootContext;
use crate::netsh::helper::CreateRootHelper;
use crate::netsh::alias::DestroyAliases;
use crate::netsh::wmi::GetWmiVersionInfo;
use crate::netsh::alias::InitAliases;
use crate::netsh::interpreter::InterpretInteractive;
use crate::netsh::interpreter::InterpretLine;
use crate::netsh::helper::LoadHelpers;
use crate::netsh::helper::UnloadHelpers;
use crate::netsh::context::g_pszMachine;
// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_1B9E0FE7B740: [u16; 2] = [0x003Fu16, 0x0000];
pub static RUSTCONV_WSTR_2876443B278F: [u16; 3] = [0x002Fu16, 0x0066, 0x0000];
pub static RUSTCONV_WSTR_4B12B7AA973F: [u16; 3] = [0x002Du16, 0x0066, 0x0000];
pub static RUSTCONV_WSTR_5462AED96D21: [u16; 3] = [0x002Fu16, 0x003F, 0x0000];
pub static RUSTCONV_WSTR_726FA0559229: [u16; 3] = [0x002Fu16, 0x0061, 0x0000];
pub static RUSTCONV_WSTR_795916488012: [u16; 3] = [0x002Du16, 0x003F, 0x0000];
pub static RUSTCONV_WSTR_7B9EDAB234EC: [u16; 3] = [0x002Du16, 0x0063, 0x0000];
pub static RUSTCONV_WSTR_8D71B3FAAB82: [u16; 2] = [0x0020u16, 0x0000];
pub static RUSTCONV_WSTR_9896841D0A6A: [u16; 5] = [0x0022u16, 0x0025, 0x0073, 0x0022, 0x0000];
pub static RUSTCONV_WSTR_A75AE3CBC09C: [u16; 3] = [0x002Fu16, 0x0063, 0x0000];
pub static RUSTCONV_WSTR_B4A9A6D1A9FB: [u16; 39] = [
    0x000Au16, 0x0054, 0x0068, 0x0065, 0x0020, 0x002D, 0x0061, 0x0020, 0x006F, 0x0070,
    0x0074, 0x0069, 0x006F, 0x006E, 0x0020, 0x0069, 0x0073, 0x0020, 0x006E, 0x006F,
    0x0074, 0x0020, 0x0069, 0x006D, 0x0070, 0x006C, 0x0065, 0x006D, 0x0065, 0x006E,
    0x0074, 0x0065, 0x0064, 0x0020, 0x0079, 0x0065, 0x0074, 0x000A, 0x0000,
];
pub static RUSTCONV_WSTR_BABF4626E77C: [u16; 3] = [0x002Fu16, 0x0072, 0x0000];
pub static RUSTCONV_WSTR_BC716484135C: [u16; 3] = [0x002Du16, 0x0072, 0x0000];
pub static RUSTCONV_WSTR_C1E8CE780EF4: [u16; 3] = [0x002Du16, 0x0061, 0x0000];
pub static RUSTCONV_WSTR_D877BF4E5023: [u16; 2] = [0x0072u16, 0x0000];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[no_mangle]
pub static mut g_hModule: HMODULE = ::core::ptr::null::<HINSTANCE__>() as *mut HINSTANCE__;
#[no_mangle]
pub unsafe extern "C" fn RunScript(filename: LPCWSTR) -> DWORD {
    let mut tmp_string: [WCHAR; (MAX_STRING_SIZE) as _] = [0; (MAX_STRING_SIZE) as _];
    let mut script: *mut FILE = core::ptr::null_mut::<FILE>();
    let mut dwError: DWORD = ERROR_SUCCESS;

    /* Open the file for processing */
    script = (_wfopen(filename as *const wchar_t, RUSTCONV_WSTR_D877BF4E5023.as_ptr() as *const wchar_t)).cast();
    if script.is_null() {
        ConResPrintf(&raw mut csStdErr, IDS_OPEN_FAILED as UINT, filename);
        return ERROR_FILE_NOT_FOUND;
    }

    /* Read and process the script */
    while !fgetws(&raw mut tmp_string as *mut wchar_t, (MAX_STRING_SIZE as i32), script).is_null() {
        dwError = InterpretLine(&raw mut tmp_string as LPWSTR);
        if dwError != ERROR_SUCCESS {
            break;
        }
    }

    /* Close the file */
    fclose(script);

    return dwError;
}
#[no_mangle]
pub unsafe extern "C" fn MergeStrings(pszStringArray: *mut LPWSTR, nCount: UINT) -> LPWSTR {
    let mut pszOutString: LPWSTR = core::ptr::null_mut();
    let mut nLength: size_t;
    let mut i: UINT;

    if pszStringArray.is_null() || nCount == 0 as UINT {
        return core::ptr::null_mut();
    }

    nLength = 0 as size_t;
    i = 0 as UINT;
    while i < nCount {
        nLength = nLength.wrapping_add(
            wcslen(*pszStringArray.offset(i as isize) as *const wchar_t) as size_t,
        );
        i = i.wrapping_add(1);
    }

    if nLength > 0 as size_t {
        nLength = nLength.wrapping_add(nCount as size_t); /* Space characters and terminating zero */
    }

    pszOutString = HeapAlloc(
        GetProcessHeap(),
        HEAP_ZERO_MEMORY as DWORD,
        (nLength as SIZE_T).wrapping_mul(core::mem::size_of::<WCHAR>() as SIZE_T),
    ) as LPWSTR;
    if pszOutString.is_null() {
        return core::ptr::null_mut();
    }

    i = 0 as UINT;
    while i < nCount {
        if i != 0 as UINT {
            wcscat(pszOutString as *mut wchar_t, RUSTCONV_WSTR_8D71B3FAAB82.as_ptr());
        }
        wcscat(
            pszOutString as *mut wchar_t,
            *pszStringArray.offset(i as isize) as *const wchar_t,
        );
        i = i.wrapping_add(1);
    }

    return pszOutString;
}

#[no_mangle]
pub unsafe extern "C" fn wmain(
    argc: ::core::ffi::c_int,
    argv: *const LPWSTR,
) -> ::core::ffi::c_int {
    let mut pszScriptFileName: LPCWSTR = core::ptr::null();
    let mut pszAliasFileName: LPCWSTR = core::ptr::null();
    let mut pszContext: LPCWSTR = core::ptr::null();
    let mut pszCommand: LPWSTR = core::ptr::null_mut();
    let mut index: ::core::ffi::c_int;
    let mut dwError: DWORD = ERROR_SUCCESS;

    DPRINT!(b"wmain(%S)\n\0".as_ptr() as *const i8, GetCommandLineW());

    GetWmiVersionInfo();

    g_hModule = GetModuleHandle(core::ptr::null_mut());

    /* Initialize the Console Standard Streams */
    ConInitStdStreams();

    /* FIXME: Init code goes here */
    InitAliases();
    CreateRootHelper();
    CreateRootContext();
    LoadHelpers();

    'done: {
        /* Process the command arguments */
        index = 1;
        while index < argc {
            if (_wcsicmp(*argv.offset(index as isize), RUSTCONV_WSTR_795916488012.as_ptr()) == 0) ||
                (_wcsicmp(*argv.offset(index as isize), RUSTCONV_WSTR_5462AED96D21.as_ptr()) == 0) ||
                (_wcsicmp(*argv.offset(index as isize), RUSTCONV_WSTR_1B9E0FE7B740.as_ptr()) == 0)
            {
                /* Help option */
                ConResPuts(StdOut, IDS_APP_USAGE);
                dwError = ERROR_SUCCESS;
                break 'done;
            }
            else if (_wcsicmp(*argv.offset(index as isize), RUSTCONV_WSTR_C1E8CE780EF4.as_ptr()) == 0) ||
                    (_wcsicmp(*argv.offset(index as isize), RUSTCONV_WSTR_726FA0559229.as_ptr()) == 0)
            {
                /* Aliasfile option */
                if (index + 1) < argc
                {
                    index += 1;
                    ConPuts(StdOut, RUSTCONV_WSTR_B4A9A6D1A9FB.as_ptr());
                    pszAliasFileName = *argv.offset(index as isize) as LPCWSTR;
                }
                else
                {
                    ConResPuts(StdOut, IDS_APP_USAGE);
                    dwError = ERROR_INVALID_SYNTAX;
                    break 'done;
                }
            }
            else if (_wcsicmp(*argv.offset(index as isize), RUSTCONV_WSTR_7B9EDAB234EC.as_ptr()) == 0) ||
                    (_wcsicmp(*argv.offset(index as isize), RUSTCONV_WSTR_A75AE3CBC09C.as_ptr()) == 0)
            {
                /* Context option */
                if (index + 1) < argc
                {
                    index += 1;
                    pszContext = *argv.offset(index as isize) as LPCWSTR;
                }
                else
                {
                    ConResPuts(StdOut, IDS_APP_USAGE);
                    dwError = ERROR_INVALID_SYNTAX;
                    break 'done;
                }
            }
            else if (_wcsicmp(*argv.offset(index as isize), RUSTCONV_WSTR_4B12B7AA973F.as_ptr()) == 0) ||
                    (_wcsicmp(*argv.offset(index as isize), RUSTCONV_WSTR_2876443B278F.as_ptr()) == 0)
            {
                /* File option */
                if (index + 1) < argc
                {
                    index += 1;
                    pszScriptFileName = *argv.offset(index as isize) as LPCWSTR;
                }
                else
                {
                    ConResPuts(StdOut, IDS_APP_USAGE);
                    dwError = ERROR_INVALID_SYNTAX;
                    break 'done;
                }
            }
            else if (_wcsicmp(*argv.offset(index as isize), RUSTCONV_WSTR_BC716484135C.as_ptr()) == 0) ||
                    (_wcsicmp(*argv.offset(index as isize), RUSTCONV_WSTR_BABF4626E77C.as_ptr()) == 0)
            {
                /* Remote option */
                if (index + 1) < argc
                {
                    index += 1;
                    g_pszMachine = HeapAlloc(
                        GetProcessHeap(),
                        0,
                        ((wcslen(*argv.offset(index as isize) as *const u16) + 1) as SIZE_T)
                            * (core::mem::size_of::<WCHAR>() as SIZE_T),
                    ) as PWSTR;
                    if g_pszMachine.is_null()
                    {
                        dwError = ERROR_NOT_ENOUGH_MEMORY;
                        PrintError((g_hModule).cast(), dwError);
                        break 'done;
                    }

                    wcscpy(g_pszMachine, *argv.offset(index as isize));
                }
                else
                {
                    ConResPuts(StdOut, IDS_APP_USAGE);
                    dwError = ERROR_INVALID_SYNTAX;
                    break 'done;
                }
            }
            else
            {
                if !pszScriptFileName.is_null()
                {
                    ConResPuts(StdOut, IDS_APP_USAGE);
                    dwError = ERROR_INVALID_SYNTAX;
                    break 'done;
                }
                else if pszCommand.is_null()
                {
                    pszCommand = MergeStrings(
                        argv.offset(index as isize) as *mut LPWSTR,
                        (argc - index) as UINT,
                    );
                    if pszCommand.is_null()
                    {
                        dwError = ERROR_NOT_ENOUGH_MEMORY;
                        PrintError((g_hModule).cast(), dwError);
                        break 'done;
                    }

                    break;
                }
            }
            index += 1;
        }

        /* Run the alias file */
        if !pszAliasFileName.is_null()
        {
            dwError = RunScript(pszAliasFileName);
            if dwError != ERROR_SUCCESS
            {
                break 'done;
            }
        }

        /* Set a context */
        if !pszContext.is_null()
        {
            dwError = InterpretLine(pszContext as LPWSTR);
            if dwError != ERROR_SUCCESS
            {
                break 'done;
            }
        }

        /* Run a script, the command line instruction or the interactive interpeter */
        if !pszScriptFileName.is_null()
        {
            dwError = RunScript(pszScriptFileName);
        }
        else if !pszCommand.is_null()
        {
            dwError = InterpretLine(pszCommand);
        }
        else
        {
            InterpretInteractive();
        }
    }

    /* FIXME: Cleanup code goes here */
    if !g_pszMachine.is_null()
    {
        HeapFree(GetProcessHeap(), 0, (g_pszMachine).cast());
    }

    if !pszCommand.is_null()
    {
        HeapFree(GetProcessHeap(), 0, (pszCommand).cast());
    }

    CleanupContext();
    UnloadHelpers();
    DestroyAliases();

    return if dwError == ERROR_SUCCESS {
        (EXIT_SUCCESS as i32)
    } else {
        (EXIT_FAILURE as i32)
    };
}

#[no_mangle]
pub unsafe extern "system" fn FreeQuotedString(pszQuotedString: LPWSTR) {
    DPRINT!("FreeQuotedString(%S)\n", pszQuotedString);
    HeapFree(GetProcessHeap(), 0, pszQuotedString as PVOID);
}

#[no_mangle]
pub unsafe extern "system" fn FreeString(pszString: LPWSTR) {
    // DPRINT("FreeString(%S)\n", pszString);
    LocalFree((pszString).cast());
}

#[no_mangle]
pub unsafe extern "system" fn MakeQuotedString(pszString: LPWSTR) -> LPWSTR {
    let mut pszQuotedString: LPWSTR;

    DPRINT!("MakeQuotedString(%S)\n", pszString);

    pszQuotedString = HeapAlloc(GetProcessHeap(), 0, ((wcslen(pszString) + 3) * core::mem::size_of::<WCHAR>()) as SIZE_T) as LPWSTR;
    if pszQuotedString.is_null() {
        return core::ptr::null_mut();
    }

    _swprintf(pszQuotedString, RUSTCONV_WSTR_9896841D0A6A.as_ptr(), pszString);

    return pszQuotedString;
}

#[no_mangle]
pub unsafe extern "C" fn MakeString_v(mut hModule: HANDLE, mut dwMsgId: DWORD, ap_list: *mut ::core::ffi::c_void) -> LPWSTR {
    let mut pszStr: LPCWSTR = core::ptr::null();
    let mut pszInBuffer: LPWSTR = core::ptr::null_mut();
    let mut pszOutBuffer: LPWSTR = core::ptr::null_mut();
    let mut dwLength: DWORD;
    let mut ap: va_list;

    DPRINT!("MakeString(%p %lu ...)\n", hModule, dwMsgId);

    dwLength = LoadStringW(
        (hModule as HMODULE).cast(),
        dwMsgId as UINT,
        core::ptr::addr_of_mut!(pszStr) as LPWSTR,
        0,
    ) as DWORD;
    if dwLength == 0 {
        return core::ptr::null_mut();
    }

    /* Allocate and copy the resource string, NUL-terminated */
    pszInBuffer = HeapAlloc(
        GetProcessHeap(),
        0,
        (dwLength.wrapping_add(1) as SIZE_T)
            .wrapping_mul(core::mem::size_of::<WCHAR>() as SIZE_T),
    ) as LPWSTR;
    if pszInBuffer.is_null() {
        return core::ptr::null_mut();
    }
    CopyMemory!(
        pszInBuffer as *mut core::ffi::c_void,
        pszStr as *const core::ffi::c_void,
        (dwLength as usize).wrapping_mul(core::mem::size_of::<WCHAR>()),
    );
    *pszInBuffer.offset(dwLength as isize) = UNICODE_NULL;

    ap = *(ap_list as *mut va_list);
    FormatMessageW(
        FORMAT_MESSAGE_ALLOCATE_BUFFER | FORMAT_MESSAGE_FROM_STRING,
        pszInBuffer as LPCVOID,
        0,
        0,
        core::ptr::addr_of_mut!(pszOutBuffer) as LPWSTR,
        0,
        core::ptr::addr_of_mut!(ap),
    );

    HeapFree(GetProcessHeap(), 0, pszInBuffer as PVOID);

    return pszOutBuffer;
}



#[no_mangle]
pub unsafe extern "system" fn MatchEnumTag(
    hModule: HANDLE,
    pwcArg: LPCWSTR,
    dwNumArg: DWORD,
    pEnumTable: *const TOKEN_VALUE,
    pdwValue: PDWORD,
) -> DWORD {
    let mut i: DWORD;

    DPRINT!("MatchEnumTag(%p %p %lu %p %p)\n", hModule, pwcArg, dwNumArg, pEnumTable, pdwValue);

    if pEnumTable.is_null() || pdwValue.is_null() {
        return ERROR_INVALID_PARAMETER;
    }

    i = 0;
    while i < dwNumArg {
        DPRINT!("%S -- %S\n", pwcArg, (*pEnumTable.add(i as usize)).pwszToken);
        if MatchToken(pwcArg, (*pEnumTable.add(i as usize)).pwszToken) != 0 {
            *pdwValue = (*pEnumTable.add(i as usize)).dwValue;
            return ERROR_SUCCESS;
        }
        i += 1;
    }

    return ERROR_NOT_FOUND;
}

#[no_mangle]
pub unsafe extern "system" fn MatchTagsInCmdLine(
         hModule: HANDLE,
            ppwcArguments: *mut LPWSTR,
         dwCurrentIndex: DWORD,
         dwArgCount: DWORD,
            pttTags: *mut TAG_TYPE,
         dwTagCount: DWORD,
          pdwTagType: *mut DWORD,
) -> DWORD {
    let mut pszEqual: PWSTR = core::ptr::null_mut();
    let mut i: DWORD = 0;
    let mut j: DWORD = 0;
    let mut dwTagLength: DWORD = 0;

    DPRINT1!("MatchTagsInCmdLine(%p %p %lu %lu %p %lu %p)\n",
            hModule, ppwcArguments, dwCurrentIndex, dwArgCount,
            pttTags, dwTagCount, pdwTagType);

    /* Identify tagged arguments (tag=value) */
    i = dwCurrentIndex;
    while i < dwArgCount {
        DPRINT!("Argument %lu: %S\n", i, *ppwcArguments.offset(i as isize));
        *pdwTagType.offset(i.wrapping_sub(dwCurrentIndex) as isize) =
            (1 as DWORD).wrapping_neg();

        /* Skip arguments that do not have a tag */
        pszEqual = wcschr(*ppwcArguments.offset(i as isize) as *const u16, 0x003Du16) as PWSTR;
        if !pszEqual.is_null() {
            dwTagLength = pszEqual.offset_from(*ppwcArguments.offset(i as isize)) as DWORD;
            DPRINT!("Tag length %lu\n", dwTagLength);
            DPRINT!("Value length %lu\n", wcslen(pszEqual.offset(1 as isize) as *const u16));

            *pdwTagType.offset(i.wrapping_sub(dwCurrentIndex) as isize) =
                (1 as DWORD).wrapping_neg();
            j = 0;
            while j < dwTagCount {
                DPRINT!("Test tag %S -- %S\n", (*pttTags.offset(j as isize)).pwszTag, *ppwcArguments.offset(i as isize));
                if (wcslen((*pttTags.offset(j as isize)).pwszTag as *const u16) as usize == dwTagLength as usize)
                    && (_wcsnicmp(
                            *ppwcArguments.offset(i as isize) as *const u16,
                            (*pttTags.offset(j as isize)).pwszTag as *const u16,
                            dwTagLength as usize) == 0)
                {
                    DPRINT!("Found tag %S\n", (*pttTags.offset(j as isize)).pwszTag);
                    (*pttTags.offset(j as isize)).bPresent = (TRUE as i32);
                    *pdwTagType.offset(i.wrapping_sub(dwCurrentIndex) as isize) = j;

                    /* Remove the tag name from the argument */
                    wcscpy(*ppwcArguments.offset(i as isize), pszEqual.offset(1 as isize) as *const u16);
                    break;
                } else {
                    j = j.wrapping_add(1);
                }
            }
        }
        i = i.wrapping_add(1);
    }

    /* Identify un-tagged arguments (value) */
    i = dwCurrentIndex;
    while i < dwArgCount {
        if *pdwTagType.offset(i.wrapping_sub(dwCurrentIndex) as isize)
            == (1 as DWORD).wrapping_neg()
        {
            j = 0;
            while j < dwTagCount {
                DPRINT!("Test tag %S\n", (*pttTags.offset(j as isize)).pwszTag);
                if (*pttTags.offset(j as isize)).bPresent == (FALSE as i32) {
                    DPRINT!("Found tag %S\n", (*pttTags.offset(j as isize)).pwszTag);
                    (*pttTags.offset(j as isize)).bPresent = (TRUE as i32);
                    *pdwTagType.offset(i.wrapping_sub(dwCurrentIndex) as isize) = j;
                    break;
                } else {
                    j = j.wrapping_add(1);
                }
            }
        }
        i = i.wrapping_add(1);
    }

    return ERROR_SUCCESS;
}

#[no_mangle]
pub unsafe extern "system" fn MatchToken(pwszUserToken: LPCWSTR, pwszCmdToken: LPCWSTR) -> BOOL {
    DPRINT!("MatchToken(%S %S)\n", pwszUserToken, pwszCmdToken);

    if pwszUserToken.is_null() || pwszCmdToken.is_null() {
        return (FALSE) as BOOL;
    }

    return if _wcsnicmp(pwszUserToken, pwszCmdToken, wcslen(pwszUserToken)) == 0 {
        (TRUE as i32)
    } else {
        (FALSE as i32)
    };
}

#[no_mangle]
pub unsafe extern "system" fn NsGetFriendlyNameFromIfName(
    mut dwUnknown1: DWORD,
    mut pszIfName: PWSTR,
    mut pszFriendlyName: PWSTR,
    mut pdwFriendlyName: PDWORD,
) -> DWORD {
    let mut UnicodeIfName: _UNICODE_STRING = core::mem::zeroed();
    let mut InterfaceGuid: GUID = core::mem::zeroed();
    let mut Status: NTSTATUS = 0;
    let mut ret: DWORD = 0;

    DPRINT!(
        "NsGetFriendlyNameFromIfName(%lx %S %p %p)\n",
        dwUnknown1,
        pszIfName,
        pszFriendlyName,
        pdwFriendlyName
    );

    RtlInitUnicodeString(core::ptr::addr_of_mut!(UnicodeIfName), pszIfName);
    Status = RtlGUIDFromString(
        core::ptr::addr_of_mut!(UnicodeIfName),
        core::ptr::addr_of_mut!(InterfaceGuid),
    );
    if !NT_SUCCESS!(Status) {
        DPRINT1!("RtlGUIDFromString failed 0x%08lx\n", Status);
        return RtlNtStatusToDosError(Status) as DWORD;
    }

    ret = NhGetInterfaceNameFromDeviceGuid(
        core::ptr::addr_of_mut!(InterfaceGuid),
        pszFriendlyName,
        pdwFriendlyName,
        0,
        0,
    );
    if ret != ERROR_SUCCESS {
        DPRINT1!("NhGetInterfaceNameFromDeviceGuid() failed %lu\n", ret);
    }

    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn NsGetIfNameFromFriendlyName(
    dwUnknown1: DWORD,
    pszFriendlyName: PWSTR,
    pszIfName: PWSTR,
    pdwIfName: PDWORD,
) -> DWORD {
    let mut UnicodeIfName: _UNICODE_STRING = core::mem::zeroed();
    let mut InterfaceGuid: GUID = core::mem::zeroed();
    let mut Status: NTSTATUS;
    let mut ret: DWORD;

    DPRINT!(
        "NsGetIfNameFromFriendlyName(%lx %S %p %p)\n",
        dwUnknown1,
        pszFriendlyName,
        pszIfName,
        pdwIfName
    );

    ret = NhGetGuidFromInterfaceName(
        pszFriendlyName,
        core::ptr::addr_of_mut!(InterfaceGuid),
        0,
        0,
    );
    if ret != ERROR_SUCCESS {
        DPRINT1!("NhGetGuidFromInterfaceName failed %lu\n", ret);
        return ret;
    }

    RtlInitUnicodeString(core::ptr::addr_of_mut!(UnicodeIfName), core::ptr::null_mut());
    Status = RtlStringFromGUID(
        core::ptr::addr_of_mut!(InterfaceGuid),
        core::ptr::addr_of_mut!(UnicodeIfName),
    );
    if !NT_SUCCESS!(Status) {
        DPRINT1!("RtlStringFromGUID failed 0x%08lx\n", Status);
        ret = RtlNtStatusToDosError(Status) as DWORD;
    }

    if *pdwIfName >= UnicodeIfName.MaximumLength as DWORD {
        CopyMemory!(
            pszIfName as *mut core::ffi::c_void,
            UnicodeIfName.Buffer as *const core::ffi::c_void,
            UnicodeIfName.MaximumLength as usize,
        );
        *pdwIfName = UnicodeIfName.MaximumLength as DWORD;
    }

    RtlFreeUnicodeString(core::ptr::addr_of_mut!(UnicodeIfName));

    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn PreprocessCommand(
    hModule: HANDLE,
    ppwcArguments: *mut LPWSTR,
    dwCurrentIndex: DWORD,
    dwArgCount: DWORD,
    pttTags: *mut TAG_TYPE,
    dwTagCount: DWORD,
    dwMinArgs: DWORD,
    dwMaxArgs: DWORD,
    pdwTagType: *mut DWORD,
) -> DWORD {
    let mut i: DWORD;
    let mut dwError: DWORD = ERROR_SUCCESS as DWORD;

    DPRINT!("PreprocessCommand()\n");

    if (ppwcArguments.is_null()) || (pttTags.is_null()) || (pdwTagType.is_null()) {
        return ERROR_INVALID_PARAMETER as DWORD;
    }

    if (dwArgCount.wrapping_sub(dwCurrentIndex) < dwMinArgs)
        || (dwArgCount.wrapping_sub(dwCurrentIndex) > dwMaxArgs)
    {
        return ERROR_INVALID_SYNTAX as DWORD;
    }

    i = 0 as DWORD;
    while i < dwTagCount {
        (*pttTags.offset(i as isize)).bPresent = FALSE as BOOL;
        i = i.wrapping_add(1);
    }

    if dwArgCount.wrapping_sub(dwCurrentIndex) > 0 as DWORD {
        dwError = MatchTagsInCmdLine(
            hModule,
            ppwcArguments,
            dwCurrentIndex,
            dwArgCount,
            pttTags,
            dwTagCount,
            pdwTagType,
        );
        if dwError != ERROR_SUCCESS as DWORD {
            return dwError;
        }
    }

    /* Fail, if a required tag is missing */
    i = 0 as DWORD;
    while i < dwTagCount {
        if ((*pttTags.offset(i as isize)).dwRequired & NS_REQ_PRESENT as DWORD) != 0
            && ((*pttTags.offset(i as isize)).bPresent == FALSE as BOOL)
        {
            return ERROR_INVALID_SYNTAX as DWORD;
        }
        i = i.wrapping_add(1);
    }

    return 0 as DWORD;
}

#[no_mangle]
pub unsafe extern "C" fn PrintError_v(mut hModule: HANDLE, mut dwErrId: DWORD, ap_list: *mut ::core::ffi::c_void) -> DWORD {
    let mut Length: INT = 0;
    let mut ap: va_list = *(ap_list as *mut va_list);

    if !hModule.is_null() {
        Length = ConResMsgPrintfExV(
            &raw mut csStdErr,
            hModule as HINSTANCE,
            0 as DWORD,
            dwErrId as UINT,
            LANG_USER_DEFAULT as LANGID,
            &raw mut ap,
        );
    } else {
        if dwErrId > NETSH_ERROR_BASE as DWORD && dwErrId < NETSH_ERROR_END as DWORD {
            Length = ConResMsgPrintfExV(
                &raw mut csStdErr,
                g_hModule as HINSTANCE,
                0 as DWORD,
                dwErrId as UINT,
                LANG_USER_DEFAULT as LANGID,
                &raw mut ap,
            );
        } else {
            Length = ConMsgPrintfV(
                &raw mut csStdErr,
                FORMAT_MESSAGE_FROM_SYSTEM as DWORD,
                ::core::ptr::null::<::core::ffi::c_void>(),
                dwErrId,
                LANG_USER_DEFAULT as DWORD,
                &raw mut ap,
            );
        }
    }
    return Length as DWORD;
}



#[no_mangle]
pub unsafe extern "C" fn PrintMessageFromModule_v(
    hModule: HANDLE,
    dwMsgId: DWORD,
    ap_list: *mut ::core::ffi::c_void,
) -> DWORD {
    let mut Length: INT;
    let mut ap: va_list = *(ap_list as *mut va_list);
    Length = ConResMsgPrintfExV(StdOut, hModule, 0, dwMsgId,
                                LANG_USER_DEFAULT, core::ptr::addr_of_mut!(ap));

    return Length as DWORD;
}



#[no_mangle]
pub unsafe extern "C" fn PrintMessage_v(pwszFormat: LPCWSTR, ap_list: *mut ::core::ffi::c_void) -> DWORD {
    let mut Length: INT;
    let mut ap: va_list;

    ap = va_start!(varargs_rust::VaArgs::from_ptr(ap_list), pwszFormat);
    Length = ConPrintfV(StdOut, pwszFormat, ap);
    va_end!(ap);

    return Length as DWORD;
}
