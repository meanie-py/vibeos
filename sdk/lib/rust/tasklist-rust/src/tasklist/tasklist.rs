// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_075DE2B906DB: [u16; 2] = [0x000Au16, 0x0000];
pub static RUSTCONV_WSTR_0AE0FBC4D35C: [u16; 6] = [0x0025u16, 0x002A, 0x006C, 0x006C, 0x0064, 0x0000];
pub static RUSTCONV_WSTR_19697AB0AF60: [u16; 20] = [
    0x0053u16, 0x0079, 0x0073, 0x0074, 0x0065, 0x006D, 0x0020, 0x0049, 0x0064, 0x006C,
    0x0065, 0x0020, 0x0050, 0x0072, 0x006F, 0x0063, 0x0065, 0x0073, 0x0073, 0x0000,
];
pub static RUSTCONV_WSTR_4B4C91EDBEEC: [u16; 2] = [0x003Du16, 0x0000];
pub static RUSTCONV_WSTR_559A3F24DB94: [u16; 7] = [0x0025u16, 0x002A, 0x002E, 0x002A, 0x006C, 0x0073, 0x0000];
pub static RUSTCONV_WSTR_96A296D224F2: [u16; 1] = [0x0000u16];
pub static RUSTCONV_WSTR_B9557C9818DD: [u16; 5] = [0x0025u16, 0x002A, 0x006C, 0x0073, 0x0000];
pub static RUSTCONV_WSTR_DF73BCF53ABC: [u16; 8] = [0x0025u16, 0x002D, 0x002A, 0x002E, 0x002A, 0x006C, 0x0073, 0x0000];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
pub static RUSTCONV_C2R_WSTR_1B9E0FE7B740: [u16; 2] = [0x003Fu16, 0x0000];
pub static RUSTCONV_C2R_WSTR_CA4ED608831C: [u16; 3] = [0x006Eu16, 0x0068, 0x0000];

static mut OptionList: [PCWSTR; 2] = unsafe {
    [
        RUSTCONV_C2R_WSTR_1B9E0FE7B740.as_ptr(),
        RUSTCONV_C2R_WSTR_CA4ED608831C.as_ptr(),
    ]
};

pub const OPTION_INVALID: i32 = -1;
pub const OPTION_HELP: i32 = 0;
pub const OPTION_NOHEADER: i32 = 1;
pub const RES_STR_MAXLEN: i32 = 64;
#[no_mangle]
pub unsafe extern "C" fn PrintSplitLine(mut Length: UINT) {
    while Length != 0 {
        ConPuts(StdOut, RUSTCONV_WSTR_4B4C91EDBEEC.as_ptr());
        Length = Length.wrapping_sub(1);
    }
}
#[no_mangle]
pub unsafe extern "C" fn PrintSpace(Length: UINT) {
    ConPrintf(StdOut, RUSTCONV_WSTR_B9557C9818DD.as_ptr() as PCWSTR, Length as INT, RUSTCONV_WSTR_96A296D224F2.as_ptr() as PCWSTR);
}
#[no_mangle]
pub unsafe extern "C" fn PrintString(String: LPCWSTR, MaxWidth: UINT, bAlignLeft: BOOL) {
    ConPrintf(
        StdOut,
        if bAlignLeft != 0 { RUSTCONV_WSTR_DF73BCF53ABC.as_ptr() } else { RUSTCONV_WSTR_559A3F24DB94.as_ptr() },
        MaxWidth,
        MaxWidth,
        String,
    );
}
#[no_mangle]
pub unsafe extern "C" fn PrintResString(uID: UINT, MaxWidth: UINT, bAlignLeft: BOOL) {
    let mut StringBuffer: [WCHAR; (RES_STR_MAXLEN as usize)] = [0; (RES_STR_MAXLEN as usize)];
    LoadStringW(
        core::ptr::null_mut(),
        uID,
        core::ptr::addr_of_mut!(StringBuffer) as *mut WCHAR as LPWSTR,
        (_countof!(StringBuffer)) as i32,
    );
    PrintString(
        core::ptr::addr_of_mut!(StringBuffer) as *mut WCHAR as LPCWSTR,
        MaxWidth,
        bAlignLeft,
    );
}
#[no_mangle]
pub unsafe extern "C" fn PrintNum(Number: LONGLONG, MaxWidth: UINT) {
    ConPrintf(
        StdOut,
        RUSTCONV_WSTR_0AE0FBC4D35C.as_ptr() as *const u16,
        MaxWidth,
        Number,
    );
}
#[no_mangle]
pub unsafe extern "C" fn PrintMemory(MemorySizeByte: SIZE_T, MaxWidth: UINT) -> BOOL {
    let mut MemorySize: SIZE_T = MemorySizeByte >> 10;

    let mut NumberString: [WCHAR; 27] = [0; 27]; // length 26 is enough to display ULLONG_MAX in decimal with comma, one more for zero-terminated.
    C_ASSERT!(core::mem::size_of::<SIZE_T>() <= 8);

    let mut pNumberStr: PWCHAR = core::ptr::addr_of_mut!(NumberString) as PWCHAR;

    // calculate the length
    let mut PrintLength: UINT = 0 as UINT;
    let mut Tmp: SIZE_T = MemorySize;
    let mut Mod: UINT = 1 as UINT;

    loop {
        Tmp = Tmp.wrapping_div(10 as SIZE_T);
        PrintLength = PrintLength.wrapping_add(1);
        Mod = Mod.wrapping_mul(10 as UINT);
        if !(Tmp != 0) {
            break;
        }
    }

    let mut i: UINT = PrintLength;
    while i != 0 {
        Mod = Mod.wrapping_div(10 as UINT);
        *pNumberStr = ((0x0030u16 as SIZE_T).wrapping_add(MemorySize / (Mod as SIZE_T))) as WCHAR;
        MemorySize = MemorySize % (Mod as SIZE_T);
        pNumberStr = pNumberStr.add(1);

        if i != (1 as UINT) && i.wrapping_rem(3 as UINT) == (1 as UINT) {
            *pNumberStr = 0x002Cu16;
            pNumberStr = pNumberStr.add(1);
        }

        i = i.wrapping_sub(1);
    }

    let mut FormatStr: [WCHAR; (RES_STR_MAXLEN as usize)] = [0; (RES_STR_MAXLEN as usize)];
    LoadStringW(core::ptr::null_mut(), IDS_MEMORY_STR as UINT, core::ptr::addr_of_mut!(FormatStr) as LPWSTR, (_countof!(FormatStr)) as i32);

    let mut String: [WCHAR; (RES_STR_MAXLEN + 27) as usize] = [0; (RES_STR_MAXLEN + 27) as usize];
    StringCchPrintfW!(core::ptr::addr_of_mut!(String) as *mut WCHAR, _countof!(String), core::ptr::addr_of_mut!(FormatStr) as *mut WCHAR, core::ptr::addr_of_mut!(NumberString) as *mut WCHAR);
    PrintString(core::ptr::addr_of_mut!(String) as *mut WCHAR as LPCWSTR, MaxWidth, FALSE as BOOL);

    return TRUE as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn PrintHeader() {
    PrintResString(IDS_HEADER_IMAGENAME, COLUMNWIDTH_IMAGENAME, (TRUE) as BOOL);
    PrintSpace(1);
    PrintResString(IDS_HEADER_PID, COLUMNWIDTH_PID, (FALSE) as BOOL);
    PrintSpace(1);
    PrintResString(IDS_HEADER_SESSION, COLUMNWIDTH_SESSION, (FALSE) as BOOL);
    PrintSpace(1);
    PrintResString(IDS_HEADER_MEMUSAGE, COLUMNWIDTH_MEMUSAGE, (FALSE) as BOOL);

    ConPuts(StdOut, RUSTCONV_WSTR_075DE2B906DB.as_ptr());

    PrintSplitLine(COLUMNWIDTH_IMAGENAME);
    PrintSpace(1);
    PrintSplitLine(COLUMNWIDTH_PID);
    PrintSpace(1);
    PrintSplitLine(COLUMNWIDTH_SESSION);
    PrintSpace(1);
    PrintSplitLine(COLUMNWIDTH_MEMUSAGE);

    ConPuts(StdOut, RUSTCONV_WSTR_075DE2B906DB.as_ptr());
}
#[no_mangle]
pub unsafe extern "C" fn EnumProcessAndPrint(bNoHeader: BOOL) -> BOOL {
    // Call NtQuerySystemInformation for the process information
    let mut ProcessInfoBufferLength: ULONG = 0;
    let mut ResultLength: ULONG = 0;
    let mut ProcessInfoBuffer: PBYTE = core::ptr::null_mut::<BYTE>();

    // Get the buffer size we need
    let mut Status: NTSTATUS = NtQuerySystemInformation(
        SystemProcessInformation,
        0 as PVOID,
        0 as ULONG,
        &raw mut ResultLength,
    );

    // New process/thread might appear before we call for the actual data.
    // Try to avoid this by retrying several times.
    let mut Retry: UINT = 0 as UINT;
    while Retry < NT_SYSTEM_QUERY_MAX_RETRY as UINT {
        // (Re)allocate buffer
        ProcessInfoBufferLength = ResultLength;
        ResultLength = 0;
        if !ProcessInfoBuffer.is_null() {
            let mut NewProcessInfoBuffer: PBYTE = HeapReAlloc(
                GetProcessHeap(),
                0 as DWORD,
                ProcessInfoBuffer as PVOID,
                ProcessInfoBufferLength as SIZE_T,
            ) as PBYTE;
            if !NewProcessInfoBuffer.is_null() {
                ProcessInfoBuffer = NewProcessInfoBuffer;
            } else {
                // out of memory
                ConResMsgPrintf(
                    &raw mut csStdErr,
                    0 as DWORD,
                    IDS_OUT_OF_MEMORY as UINT,
                );
                HeapFree(
                    GetProcessHeap(),
                    0 as DWORD,
                    ProcessInfoBuffer as PVOID,
                );
                return (FALSE) as BOOL;
            }
        } else {
            ProcessInfoBuffer = HeapAlloc(
                GetProcessHeap(),
                0 as DWORD,
                ProcessInfoBufferLength as SIZE_T,
            ) as PBYTE;
            if ProcessInfoBuffer.is_null() {
                // out of memory
                ConResMsgPrintf(
                    &raw mut csStdErr,
                    0 as DWORD,
                    IDS_OUT_OF_MEMORY as UINT,
                );
                return (FALSE) as BOOL;
            }
        }

        // Query information
        Status = NtQuerySystemInformation(
            SystemProcessInformation,
            ProcessInfoBuffer as PVOID,
            ProcessInfoBufferLength,
            &raw mut ResultLength,
        );
        if Status != STATUS_INFO_LENGTH_MISMATCH {
            break;
        }
        Retry = Retry.wrapping_add(1);
    }

    if !NT_SUCCESS!(Status) {
        // tried NT_SYSTEM_QUERY_MAX_RETRY times, or failed with some other reason
        ConResMsgPrintf(
            &raw mut csStdErr,
            0 as DWORD,
            IDS_ENUM_FAILED as UINT,
        );
        HeapFree(
            GetProcessHeap(),
            0 as DWORD,
            ProcessInfoBuffer as PVOID,
        );
        return (FALSE) as BOOL;
    }

    ConPuts(&raw mut csStdOut, RUSTCONV_WSTR_075DE2B906DB.as_ptr());

    if bNoHeader == 0 {
        PrintHeader();
    }

    let mut pSPI: PSYSTEM_PROCESS_INFORMATION =
        core::ptr::null_mut::<_SYSTEM_PROCESS_INFORMATION>();
    pSPI = ProcessInfoBuffer as PSYSTEM_PROCESS_INFORMATION;
    while !pSPI.is_null() {
        PrintString(
            if !(*pSPI).UniqueProcessId.is_null() {
                (*pSPI).ImageName.Buffer as LPCWSTR
            } else {
                RUSTCONV_WSTR_19697AB0AF60.as_ptr()
            },
            COLUMNWIDTH_IMAGENAME as UINT,
            TRUE as BOOL,
        );
        PrintSpace(1 as UINT);
        PrintNum(
            (*pSPI).UniqueProcessId as INT_PTR as LONGLONG,
            COLUMNWIDTH_PID as UINT,
        );
        PrintSpace(1 as UINT);
        PrintNum(
            (*pSPI).SessionId as ULONGLONG as LONGLONG,
            COLUMNWIDTH_SESSION as UINT,
        );
        PrintSpace(1 as UINT);
        PrintMemory(
            (*pSPI).WorkingSetSize,
            COLUMNWIDTH_MEMUSAGE as UINT,
        );

        ConPuts(&raw mut csStdOut, RUSTCONV_WSTR_075DE2B906DB.as_ptr());

        if (*pSPI).NextEntryOffset == 0 as ULONG {
            break;
        }
        pSPI = (pSPI as LPBYTE).offset((*pSPI).NextEntryOffset as isize)
            as PSYSTEM_PROCESS_INFORMATION;
    }

    HeapFree(
        GetProcessHeap(),
        0 as DWORD,
        ProcessInfoBuffer as PVOID,
    );
    return (TRUE) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn GetOptionType(mut szOption: LPCWSTR) -> INT {
    if *szOption.offset(0 as isize) != 0x002Fu16
        && *szOption.offset(0 as isize) != 0x002Du16
    {
        return OPTION_INVALID as INT;
    }
    szOption = szOption.offset(1);

    let mut i: UINT = 0 as UINT;
    while i < _countof!(OptionList) {
        if _wcsicmp(OptionList[i as usize], szOption) == 0 {
            return i as INT;
        }
        i += 1;
    }
    return OPTION_INVALID as INT;
}
#[no_mangle]
pub unsafe extern "C" fn ProcessArguments(mut argc: INT, mut argv: *mut *mut WCHAR) -> BOOL {
    let mut bHasHelp: BOOL = (FALSE) as i32;
    let mut bHasNoHeader: BOOL = (FALSE) as i32;
    let mut i: INT = 1;
    while i < argc {
        let mut Option: INT = GetOptionType(*argv.offset(i as isize) as LPCWSTR);
        match Option {
            OPTION_HELP => {
                if bHasHelp != 0 {
                    // -? already specified
                    ConResMsgPrintf(
                        &raw mut csStdErr,
                        0,
                        IDS_OPTION_TOO_MUCH,
                        *argv.offset(i as isize),
                        1,
                    );
                    ConResMsgPrintf(&raw mut csStdErr, 0, IDS_USAGE);
                    return (FALSE) as BOOL;
                }
                bHasHelp = (TRUE) as i32;
            }
            OPTION_NOHEADER => {
                if bHasNoHeader != 0 {
                    // -nh already specified
                    ConResMsgPrintf(
                        &raw mut csStdErr,
                        0,
                        IDS_OPTION_TOO_MUCH,
                        *argv.offset(i as isize),
                        1,
                    );
                    ConResMsgPrintf(&raw mut csStdErr, 0, IDS_USAGE);
                    return (FALSE) as BOOL;
                }
                bHasNoHeader = (TRUE) as i32;
            }
            OPTION_INVALID | _ => {
                ConResMsgPrintf(&raw mut csStdErr, 0, IDS_INVALID_OPTION);
                ConResMsgPrintf(&raw mut csStdErr, 0, IDS_USAGE);
                return (FALSE) as BOOL;
            }
        }
        i += 1;
    }

    if bHasHelp != 0 {
        if argc > 2 {
            // any arguments other than -? exists
            ConResMsgPrintf(&raw mut csStdErr, 0, IDS_INVALID_SYNTAX);
            ConResMsgPrintf(&raw mut csStdErr, 0, IDS_USAGE);
            return (FALSE) as BOOL;
        } else {
            ConResMsgPrintf(&raw mut csStdOut, 0, IDS_USAGE);
            ConResMsgPrintf(&raw mut csStdOut, 0, IDS_DESCRIPTION);
            return (FALSE) as BOOL;
        }
    } else {
        EnumProcessAndPrint(bHasNoHeader);
    }
    return (TRUE) as BOOL;
}

#[no_mangle]
pub unsafe extern "C" fn wmain(argc: i32, argv: *mut *mut u16) -> i32 {
    /* Initialize the Console Standard Streams */
    ConInitStdStreams();

    if ProcessArguments(argc, argv) == 0 {
        return 1;
    }
    return 0;
}
