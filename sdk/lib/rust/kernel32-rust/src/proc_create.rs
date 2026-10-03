//! proc_create.rs — CreateProcessInternalW and its wrappers, hand-translated
//! from dll/win32/kernel32/client/proc.c lines 2240-4839.
//!
//! Faithful translation. SXS-guarded code (`#if _SXS_SUPPORT_ENABLED_`) is
//! OMITTED (macro is false). PSEH lowered per DUMMY_PSEH. The big function's
//! control-flow (AppNameRetry / StartScan / VdmShortCircuit / Quickie labels)
//! is modeled with labeled blocks/loops — see comments inline.

use crate::substrate::*;

use crate::proc_helpers::{
    BasepCheckWebBladeHashes, BasepIsProcessAllowed, BasepIsImageVersionOk,
    BasepReplaceProcessThreadTokens, BuildSubSysCommandLine, StuffStdHandle,
};
use crate::proc_params::BasePushProcessParameters;

/* ---------------------------------------------------------------------------
 * Local helpers / constants not present in the substrate.
 * ------------------------------------------------------------------------- */

#[inline]
fn NT_SUCCESS(s: NTSTATUS) -> bool {
    s >= 0
}

/* The substrate exports TRUE/FALSE as `u32`; proc.c uses them as BOOL (c_int)
 * for return values and `Result`, and as BOOLEAN (u8) via `as BOOLEAN` casts.
 * Shadow them as BOOL here so `return FALSE;` / `Result = TRUE;` type-check, and
 * `FALSE as BOOLEAN` still works (i32 -> u8 cast). */
const TRUE: BOOL = 1;
const FALSE: BOOL = 0;

/* NTSTATUS codes used in the big `switch (Status)` that ntdll-sys doesn't
 * export (ntstatus.h cast-macros aren't bindgen-evaluable). Values verified
 * against sdk/include/ndk/ntstatus.h. */
const STATUS_INVALID_IMAGE_NE_FORMAT: NTSTATUS = 0xC000011B_u32 as NTSTATUS;
const STATUS_INVALID_IMAGE_NOT_MZ: NTSTATUS = 0xC000012F_u32 as NTSTATUS;
const STATUS_INVALID_IMAGE_PROTECT: NTSTATUS = 0xC0000130_u32 as NTSTATUS;
const STATUS_INVALID_IMAGE_WIN_16: NTSTATUS = 0xC0000131_u32 as NTSTATUS;
const STATUS_FILE_IS_OFFLINE: NTSTATUS = 0xC0000267_u32 as NTSTATUS;
const STATUS_INVALID_IMAGE_WIN_64: NTSTATUS = 0xC000035A_u32 as NTSTATUS;
const STATUS_VDM_DISALLOWED: NTSTATUS = 0xC0000414_u32 as NTSTATUS;
const STATUS_IMAGE_MACHINE_TYPE_MISMATCH_EXE: NTSTATUS = 0x40000023_u32 as NTSTATUS;
const STATUS_NAME_TOO_LONG: NTSTATUS = 0xC0000106_u32 as NTSTATUS;

/* Win32 flag/error/file constants missing from the substrate. */
const STARTF_SHELLPRIVATE: DWORD = 0x0000_0400; // == STARTF_HASSHELLDATA
const INVALID_FILE_ATTRIBUTES: DWORD = 0xFFFF_FFFF; // (DWORD)-1
const INVALID_HANDLE_VALUE: HANDLE = -1isize as HANDLE;
const UNICODE_NULL: WCHAR = 0;
const ANSI_NULL_SIZE: SIZE_T = 1; // sizeof(ANSI_NULL) == sizeof(CHAR) == 1
const UNICODE_STRING_MAX_BYTES_LOCAL: SIZE_T = 65534;

/* PROCESS_ALL_ACCESS / THREAD_ALL_ACCESS (NTDDI_LONGHORN values):
 *   STANDARD_RIGHTS_REQUIRED(0xF0000) | SYNCHRONIZE(0x100000) | 0xFFFF */
const PROCESS_ALL_ACCESS_LOCAL: ACCESS_MASK = 0x000F_0000 | 0x0010_0000 | 0xFFFF;
const THREAD_ALL_ACCESS_LOCAL: ACCESS_MASK = 0x000F_0000 | 0x0010_0000 | 0xFFFF;

/* PROCESS_PRIORITY_CLASS_REALTIME is the value that REALTIME maps to (HIGH+1).
 * The C uses the named constant PROCESS_PRIORITY_CLASS_REALTIME in a compare;
 * it is present in the substrate. */

/* ---------------------------------------------------------------------------
 * Tiny wide-string helpers (wcslen/wcscpy/wcscat not in substrate).
 * ------------------------------------------------------------------------- */

#[inline]
unsafe fn wcslen(mut s: *const u16) -> usize {
    let mut n = 0usize;
    while *s != 0 {
        s = s.add(1);
        n += 1;
    }
    n
}

#[inline]
unsafe fn wcscpy(dst: *mut u16, mut src: *const u16) -> *mut u16 {
    let mut d = dst;
    while *src != 0 {
        *d = *src;
        d = d.add(1);
        src = src.add(1);
    }
    *d = 0;
    dst
}

#[inline]
unsafe fn wcscat(dst: *mut u16, src: *const u16) -> *mut u16 {
    let mut d = dst;
    while *d != 0 {
        d = d.add(1);
    }
    wcscpy(d, src);
    dst
}

#[inline]
unsafe fn _wcsnicmp(a: *const u16, b: *const u16, count: usize) -> i32 {
    for i in 0..count {
        let ca = *a.add(i);
        let cb = *b.add(i);
        // case-fold ASCII letters
        let la = if (0x41..=0x5A).contains(&ca) { ca + 0x20 } else { ca };
        let lb = if (0x41..=0x5A).contains(&cb) { cb + 0x20 } else { cb };
        if la != lb {
            return if la < lb { -1 } else { 1 };
        }
        if ca == 0 {
            return 0;
        }
    }
    0
}

/* L"cmd /c " */
static CMD_STRING: [u16; 8] = [
    0x0063, 0x006D, 0x0064, 0x0020, 0x002F, 0x0063, 0x0020, 0x0000,
];

/* ---------------------------------------------------------------------------
 * CreateProcessInternalW
 * ------------------------------------------------------------------------- */

#[no_mangle]
pub unsafe extern "system" fn CreateProcessInternalW(
    hUserToken: HANDLE,
    mut lpApplicationName: LPCWSTR,
    mut lpCommandLine: LPWSTR,
    lpProcessAttributes: LPSECURITY_ATTRIBUTES,
    lpThreadAttributes: LPSECURITY_ATTRIBUTES,
    mut bInheritHandles: BOOL,
    mut dwCreationFlags: DWORD,
    mut lpEnvironment: LPVOID,
    lpCurrentDirectory: LPCWSTR,
    lpStartupInfo: LPSTARTUPINFOW,
    lpProcessInformation: LPPROCESS_INFORMATION,
    hNewToken: PHANDLE,
) -> BOOL {
    //
    // Core variables used for creating the initial process and thread
    //
    let mut LocalThreadAttributes: SECURITY_ATTRIBUTES = core::mem::zeroed();
    let mut LocalProcessAttributes: SECURITY_ATTRIBUTES = core::mem::zeroed();
    let mut LocalObjectAttributes: OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut ObjectAttributes: POBJECT_ATTRIBUTES;
    let mut ImageInformation: SECTION_IMAGE_INFORMATION = core::mem::zeroed();
    let mut IoStatusBlock: IO_STATUS_BLOCK = core::mem::zeroed();
    let mut ClientId: CLIENT_ID = core::mem::zeroed();
    let NoWindow: ULONG;
    let mut StackSize: ULONG;
    let mut ErrorCode: ULONG;
    let mut Flags: ULONG;
    let mut RegionSize: SIZE_T;
    let mut ImageMachine: USHORT;
    let mut PrivilegeValue: ULONG;
    let mut ErrorResponse: ULONG;
    let mut ErrorParameters: [ULONG_PTR; 2] = [0; 2];
    let mut InJob: BOOLEAN;
    let mut SaferNeeded: BOOLEAN;
    let mut UseLargePages: BOOLEAN = 0;
    let mut HavePrivilege: BOOLEAN;
    let mut QuerySection: BOOLEAN;
    let mut SkipSaferAndAppCompat: BOOLEAN;
    let mut Context: CONTEXT = core::mem::zeroed();
    let mut CsrMsg: [BASE_API_MESSAGE; 2] = core::mem::zeroed();
    let CreateProcessMsg: *mut BASE_CREATE_PROCESS;
    let mut CaptureBuffer: PCSR_CAPTURE_BUFFER;
    let mut BaseAddress: PVOID;
    let mut PrivilegeState: PVOID;
    let mut RealTimePrivilegeState: PVOID;
    let mut DebugHandle: HANDLE;
    let mut TokenHandle: HANDLE;
    let mut JobHandle: HANDLE;
    let mut KeyHandle: HANDLE = core::ptr::null_mut();
    let mut ThreadHandle: HANDLE;
    let mut FileHandle: HANDLE;
    let mut SectionHandle: HANDLE;
    let mut ProcessHandle: HANDLE;
    let mut ResumeCount: ULONG = 0;
    let mut PriorityClass: PROCESS_PRIORITY_CLASS = core::mem::zeroed();
    let mut Status: NTSTATUS;
    let mut AppCompatStatus: NTSTATUS;
    let mut SaferStatus: NTSTATUS;
    let mut IFEOStatus: NTSTATUS;
    let mut ImageDbgStatus: NTSTATUS;
    let Peb: PPEB;
    let mut RemotePeb: PPEB;
    let Teb: PTEB;
    let mut InitialTeb: INITIAL_TEB = core::mem::zeroed();
    let TibValue: PVOID;
    let NtHeaders: PIMAGE_NT_HEADERS;
    let mut StartupInfo: STARTUPINFOW;
    let mut ProcessParameters: PRTL_USER_PROCESS_PARAMETERS = core::ptr::null_mut();
    let mut DebuggerString: UNICODE_STRING = core::mem::zeroed();
    /* C leaves Result uninitialized; every path assigns before the Quickie
     * read. Init to FALSE for Rust's definite-assignment checker (neutral). */
    let mut Result: BOOL = FALSE;
    //
    // Variables used for command-line and argument parsing
    //
    let mut pcScan: PCHAR;
    let mut n: SIZE_T;
    let mut SaveChar: WCHAR = 0;
    let mut Length: ULONG;
    let mut FileAttribs: ULONG;
    let mut CmdQuoteLength: ULONG;
    let mut ResultSize: ULONG = 0;
    let mut EnvironmentLength: SIZE_T;
    let mut CmdLineLength: SIZE_T;
    let mut QuotedCmdLine: PWCHAR;
    let mut AnsiCmdCommand: PWCHAR;
    let mut ExtBuffer: PWCHAR;
    let mut CurrentDirectory: PWCHAR;
    let mut NullBuffer: PWCHAR;
    let mut ScanString: PWCHAR;
    let mut NameBuffer: PWCHAR;
    let mut SearchPath: PWCHAR;
    let mut DebuggerCmdLine: PWCHAR;
    let mut AnsiEnv: ANSI_STRING = core::mem::zeroed();
    let mut UnicodeEnv: UNICODE_STRING = core::mem::zeroed();
    let mut PathName: UNICODE_STRING = core::mem::zeroed();
    let mut SearchRetry: BOOLEAN;
    let mut QuotesNeeded: BOOLEAN;
    let mut CmdLineIsAppName: BOOLEAN;
    let mut HasQuotes: BOOLEAN;

    //
    // Variables used for Fusion/SxS (Side-by-Side Assemblies)
    //
    let mut SxsPathType: RTL_PATH_TYPE = core::mem::zeroed();
    let mut PathType: RTL_PATH_TYPE;
    let mut FusionFlags: ULONG;

    //
    // Variables used for path conversion (and partially Fusion/SxS)
    //
    let mut FilePart: PWCHAR;
    let mut PathBuffer: PWCHAR;
    let mut FreeBuffer: PWCHAR;
    let mut TranslationStatus: BOOLEAN;
    let mut SxsWin32RelativePath: RTL_RELATIVE_NAME_U = core::mem::zeroed();
    let mut PathBufferString: UNICODE_STRING = core::mem::zeroed();
    let mut SxsWin32ExePath: UNICODE_STRING = core::mem::zeroed();

    //
    // Variables used by Application Compatibility (and partially Fusion/SxS)
    //
    let mut AppCompatSxsData: PVOID;
    let mut AppCompatData: PVOID;
    let mut AppCompatSxsDataSize: ULONG;
    let mut AppCompatDataSize: ULONG;
    //
    // Variables used by VDM (Virtual Dos Machine) and WOW32 (16-bit Support)
    //
    let mut BinarySubType: ULONG;
    let mut VdmBinaryType: ULONG;
    let mut VdmTask: ULONG;
    let mut VdmReserve: ULONG;
    let mut VdmUndoLevel: ULONG;
    let mut UseVdmReserve: BOOLEAN;
    let mut VdmWaitObject: HANDLE;
    let mut VdmAnsiEnv: ANSI_STRING = core::mem::zeroed();
    let mut VdmString: UNICODE_STRING = core::mem::zeroed();
    let mut VdmUnicodeEnv: UNICODE_STRING = core::mem::zeroed();
    let mut IsWowApp: BOOLEAN;
    let CheckVdmMsg: *mut BASE_CHECK_VDM;

    /* Zero out the initial core variables and handles */
    QuerySection = FALSE as BOOLEAN;
    InJob = FALSE as BOOLEAN;
    SkipSaferAndAppCompat = FALSE as BOOLEAN;
    let mut ParameterFlags: ULONG = 0;
    Flags = 0;
    DebugHandle = core::ptr::null_mut();
    JobHandle = core::ptr::null_mut();
    TokenHandle = core::ptr::null_mut();
    FileHandle = core::ptr::null_mut();
    SectionHandle = core::ptr::null_mut();
    ProcessHandle = core::ptr::null_mut();
    ThreadHandle = core::ptr::null_mut();
    ClientId.UniqueProcess = core::ptr::null_mut();
    ClientId.UniqueThread = core::ptr::null_mut();
    BaseAddress = 1usize as PVOID;

    /* Zero out initial SxS and Application Compatibility state */
    AppCompatData = core::ptr::null_mut();
    AppCompatDataSize = 0;
    AppCompatSxsData = core::ptr::null_mut();
    AppCompatSxsDataSize = 0;
    CaptureBuffer = core::ptr::null_mut();
    FusionFlags = 0;

    /* Zero out initial parsing variables -- others are initialized later */
    DebuggerCmdLine = core::ptr::null_mut();
    PathBuffer = core::ptr::null_mut();
    SearchPath = core::ptr::null_mut();
    NullBuffer = core::ptr::null_mut();
    FreeBuffer = core::ptr::null_mut();
    NameBuffer = core::ptr::null_mut();
    CurrentDirectory = core::ptr::null_mut();
    FilePart = core::ptr::null_mut();
    DebuggerString.Buffer = core::ptr::null_mut();
    HasQuotes = FALSE as BOOLEAN;
    QuotedCmdLine = core::ptr::null_mut();

    /* Zero out initial VDM state */
    VdmAnsiEnv.Buffer = core::ptr::null_mut();
    VdmUnicodeEnv.Buffer = core::ptr::null_mut();
    VdmString.Buffer = core::ptr::null_mut();
    VdmTask = 0;
    VdmUndoLevel = 0;
    VdmBinaryType = 0;
    VdmReserve = 0;
    VdmWaitObject = core::ptr::null_mut();
    UseVdmReserve = FALSE as BOOLEAN;
    IsWowApp = FALSE as BOOLEAN;

    /* Set message structures */
    CreateProcessMsg = &mut CsrMsg[0].Data.CreateProcessRequest;
    CheckVdmMsg = &mut CsrMsg[1].Data.CheckVDMRequest;

    /* Clear the more complex structures by zeroing out their entire memory */
    core::ptr::write_bytes(&mut Context as *mut CONTEXT as *mut u8, 0, core::mem::size_of::<CONTEXT>());
    core::ptr::write_bytes(
        &mut (*CreateProcessMsg).Sxs as *mut _ as *mut u8,
        0,
        core::mem::size_of_val(&(*CreateProcessMsg).Sxs),
    );
    core::ptr::write_bytes(
        &mut LocalProcessAttributes as *mut SECURITY_ATTRIBUTES as *mut u8,
        0,
        core::mem::size_of::<SECURITY_ATTRIBUTES>(),
    );
    core::ptr::write_bytes(
        &mut LocalThreadAttributes as *mut SECURITY_ATTRIBUTES as *mut u8,
        0,
        core::mem::size_of::<SECURITY_ATTRIBUTES>(),
    );

    /* Zero out output arguments as well */
    core::ptr::write_bytes(
        lpProcessInformation as *mut u8,
        0,
        core::mem::size_of::<PROCESS_INFORMATION>(),
    );
    if !hNewToken.is_null() {
        *hNewToken = core::ptr::null_mut();
    }

    /* Capture the special window flag */
    NoWindow = dwCreationFlags & CREATE_NO_WINDOW;
    dwCreationFlags &= !CREATE_NO_WINDOW;

    /* Finally, set our TEB and PEB */
    Teb = NtCurrentTeb();
    Peb = NtCurrentPeb();

    /* This combination is illegal (see MSDN) */
    if (dwCreationFlags & (DETACHED_PROCESS | CREATE_NEW_CONSOLE))
        == (DETACHED_PROCESS | CREATE_NEW_CONSOLE)
    {
        SetLastError(ERROR_INVALID_PARAMETER);
        return FALSE;
    }

    /* Convert the priority class */
    if dwCreationFlags & IDLE_PRIORITY_CLASS != 0 {
        PriorityClass.PriorityClass = PROCESS_PRIORITY_CLASS_IDLE as UCHAR;
    } else if dwCreationFlags & BELOW_NORMAL_PRIORITY_CLASS != 0 {
        PriorityClass.PriorityClass = PROCESS_PRIORITY_CLASS_BELOW_NORMAL as UCHAR;
    } else if dwCreationFlags & NORMAL_PRIORITY_CLASS != 0 {
        PriorityClass.PriorityClass = PROCESS_PRIORITY_CLASS_NORMAL as UCHAR;
    } else if dwCreationFlags & ABOVE_NORMAL_PRIORITY_CLASS != 0 {
        PriorityClass.PriorityClass = PROCESS_PRIORITY_CLASS_ABOVE_NORMAL as UCHAR;
    } else if dwCreationFlags & HIGH_PRIORITY_CLASS != 0 {
        PriorityClass.PriorityClass = PROCESS_PRIORITY_CLASS_HIGH as UCHAR;
    } else if dwCreationFlags & REALTIME_PRIORITY_CLASS != 0 {
        PriorityClass.PriorityClass = PROCESS_PRIORITY_CLASS_HIGH as UCHAR;
        PriorityClass.PriorityClass += (BasepIsRealtimeAllowed(FALSE as BOOLEAN) != core::ptr::null_mut()) as UCHAR;
    } else {
        PriorityClass.PriorityClass = PROCESS_PRIORITY_CLASS_INVALID as UCHAR;
    }

    /* Done with the priority masks, so get rid of them */
    PriorityClass.Foreground = FALSE as BOOLEAN;
    dwCreationFlags &= !(NORMAL_PRIORITY_CLASS
        | IDLE_PRIORITY_CLASS
        | HIGH_PRIORITY_CLASS
        | REALTIME_PRIORITY_CLASS
        | BELOW_NORMAL_PRIORITY_CLASS
        | ABOVE_NORMAL_PRIORITY_CLASS);

    /* You cannot request both a shared and a separate WoW VDM */
    if (dwCreationFlags & CREATE_SEPARATE_WOW_VDM != 0)
        && (dwCreationFlags & CREATE_SHARED_WOW_VDM != 0)
    {
        /* Fail such nonsensical attempts */
        SetLastError(ERROR_INVALID_PARAMETER);
        return FALSE;
    } else if (dwCreationFlags & CREATE_SHARED_WOW_VDM == 0)
        && ((*BaseStaticServerData).DefaultSeparateVDM != 0)
    {
        /* A shared WoW VDM was not requested but system enforces separation */
        dwCreationFlags |= CREATE_SEPARATE_WOW_VDM;
    }

    /* If a shared WoW VDM is used, make sure the process isn't in a job.
     * C: `if (!flags && NtIsProcessInJob(...))` — the NTSTATUS is used as a
     * boolean, i.e. non-zero (incl. STATUS_PROCESS_IN_JOB == 1) is true. */
    if (dwCreationFlags & CREATE_SEPARATE_WOW_VDM == 0)
        && (NtIsProcessInJob(NtCurrentProcess(), core::ptr::null_mut()) != 0)
    {
        /* Remove the shared flag and add the separate flag */
        dwCreationFlags = (dwCreationFlags & !CREATE_SHARED_WOW_VDM) | CREATE_SEPARATE_WOW_VDM;
    }

    /* Convert the environment */
    if (!lpEnvironment.is_null()) && (dwCreationFlags & CREATE_UNICODE_ENVIRONMENT == 0) {
        /* Scan the environment to calculate its Unicode size */
        pcScan = lpEnvironment as PCHAR;
        AnsiEnv.Buffer = pcScan;
        while (*pcScan != 0) || (*(pcScan.add(1)) != 0) {
            pcScan = pcScan.add(1);
        }

        /* Make sure the environment is not too large */
        EnvironmentLength =
            (pcScan as usize + ANSI_NULL_SIZE as usize - lpEnvironment as usize) as SIZE_T;
        if EnvironmentLength > 0xFFFF {
            /* Fail */
            SetLastError(ERROR_INVALID_PARAMETER);
            return FALSE;
        }

        /* Create our ANSI String */
        AnsiEnv.Length = EnvironmentLength as USHORT;
        AnsiEnv.MaximumLength = (AnsiEnv.Length as u32 + ANSI_NULL_SIZE as u32) as USHORT;

        /* Allocate memory for the Unicode Environment */
        UnicodeEnv.Buffer = core::ptr::null_mut();
        RegionSize = (AnsiEnv.MaximumLength as SIZE_T) * core::mem::size_of::<WCHAR>() as SIZE_T;
        Status = NtAllocateVirtualMemory(
            NtCurrentProcess(),
            &mut UnicodeEnv.Buffer as *mut PWSTR as *mut PVOID,
            0,
            &mut RegionSize,
            MEM_COMMIT,
            PAGE_READWRITE,
        );
        if !NT_SUCCESS(Status) {
            /* Fail */
            BaseSetLastNTError(Status);
            return FALSE;
        }

        /* Use the allocated size and convert */
        UnicodeEnv.MaximumLength = RegionSize as USHORT;
        Status = RtlAnsiStringToUnicodeString(&mut UnicodeEnv, &mut AnsiEnv, FALSE as BOOLEAN);
        if !NT_SUCCESS(Status) {
            /* Fail */
            NtFreeVirtualMemory(
                NtCurrentProcess(),
                &mut UnicodeEnv.Buffer as *mut PWSTR as *mut PVOID,
                &mut RegionSize,
                MEM_RELEASE,
            );
            BaseSetLastNTError(Status);
            return FALSE;
        }

        /* Now set the Unicode environment as the environment string pointer */
        lpEnvironment = UnicodeEnv.Buffer as LPVOID;
    }

    /* Make a copy of the caller's startup info since we'll modify it */
    StartupInfo = *lpStartupInfo;

    /* Check if private data is being sent on the same channel as std handles */
    if (StartupInfo.dwFlags & STARTF_USESTDHANDLES != 0)
        && (StartupInfo.dwFlags & (STARTF_USEHOTKEY | STARTF_SHELLPRIVATE) != 0)
    {
        /* Cannot use the std handles since we have monitor/hotkey values */
        StartupInfo.dwFlags &= !STARTF_USESTDHANDLES;
    }

    /* ----------------------------------------------------------------------
     * The whole post-init body lives inside 'quickie. `goto Quickie` becomes
     * `break 'quickie`; the Quickie cleanup runs after the block.
     * -------------------------------------------------------------------- */
    'quickie: {
        /* If there's a debugger, or we have to launch cmd.exe, we go back here.
         * `goto AppNameRetry` => `continue 'app_name_retry`. The success tail
         * (VdmShortCircuit) is reached by falling off the loop (break) or by
         * `goto VdmShortCircuit` (=> break 'app_name_retry). */
        'app_name_retry: loop {
            /* New iteration -- free any existing name buffer */
            if !NameBuffer.is_null() {
                RtlFreeHeap(RtlGetProcessHeap(), 0, NameBuffer as PVOID);
                NameBuffer = core::ptr::null_mut();
            }

            /* New iteration -- free any existing free buffer */
            if !FreeBuffer.is_null() {
                RtlFreeHeap(RtlGetProcessHeap(), 0, FreeBuffer as PVOID);
                FreeBuffer = core::ptr::null_mut();
            }

            /* New iteration -- close any existing file handle */
            if !FileHandle.is_null() {
                NtClose(FileHandle);
                FileHandle = core::ptr::null_mut();
            }

            /* Set the initial parsing state. This code can loop -- don't move this! */
            ErrorCode = 0;
            SearchRetry = TRUE as BOOLEAN;
            QuotesNeeded = FALSE as BOOLEAN;
            CmdLineIsAppName = FALSE as BOOLEAN;

            /* First check if we don't have an application name */
            if lpApplicationName.is_null() {
                /* This should be the first time we attempt creating one */
                debug_assert!(NameBuffer.is_null());

                /* Allocate a buffer to hold it */
                NameBuffer = RtlAllocateHeap(
                    RtlGetProcessHeap(),
                    0,
                    MAX_PATH as SIZE_T * core::mem::size_of::<WCHAR>() as SIZE_T,
                ) as PWCHAR;
                if NameBuffer.is_null() {
                    SetLastError(ERROR_NOT_ENOUGH_MEMORY);
                    Result = FALSE;
                    break 'quickie;
                }

                /* Initialize the application name and our parsing parameters */
                ScanString = lpCommandLine;
                NullBuffer = lpCommandLine;
                lpApplicationName = lpCommandLine;

                /* Check for an initial quote. `skip_space_scan` models the C
                 * control flow: the quote `if` and the no-quote `else`(StartScan)
                 * both fall through to the SHARED terminate+search code (C lines
                 * 2679-2796). `goto StartScan` (line 2795) jumps back INTO the
                 * else branch (re-running the space-scan), so we wrap the shared
                 * code in 'start_scan and gate the space-scan with this flag. */
                let mut skip_space_scan: bool;
                if *lpCommandLine == '"' as u16 {
                    /* We found a quote, keep searching for another one */
                    SearchRetry = FALSE as BOOLEAN;
                    ScanString = ScanString.add(1);
                    lpApplicationName = ScanString;
                    while *ScanString != 0 {
                        /* Have we found the terminating quote? */
                        if *ScanString == '"' as u16 {
                            /* We're done, get out of here */
                            NullBuffer = ScanString;
                            HasQuotes = TRUE as BOOLEAN;
                            break;
                        }

                        /* Keep searching for the quote */
                        ScanString = ScanString.add(1);
                        NullBuffer = ScanString;
                    }

                    /* Quote case: skip the space-scan on first entry, run the
                     * shared terminate+search directly. */
                    skip_space_scan = true;
                } else {
                    /* No-quote: first entry runs the StartScan space-scan. */
                    skip_space_scan = false;
                }

                /* StartScan label target + shared terminate/compute/search.
                 * `goto StartScan` => `continue 'start_scan` (which re-runs the
                 * space-scan, since skip_space_scan is cleared below). */
                'start_scan: loop {
                    if !skip_space_scan {
                        /* We simply make the application name be the command line */
                        lpApplicationName = lpCommandLine;
                        while *ScanString != 0 {
                            /* Check if it starts with a space or tab */
                            if (*ScanString == ' ' as u16) || (*ScanString == '\t' as u16) {
                                /* Break out of the search loop */
                                NullBuffer = ScanString;
                                break;
                            }

                            /* Keep searching for a space or tab */
                            ScanString = ScanString.add(1);
                            NullBuffer = ScanString;
                        }
                    }
                    /* After the first iteration, any re-entry is a `goto StartScan`
                     * which always runs the space-scan. */
                    skip_space_scan = false;

                    /* We have found the end of the application name, terminate it */
                    SaveChar = *NullBuffer;
                    *NullBuffer = UNICODE_NULL;

                    /* New iteration -- free any existing saved path */
                    if !SearchPath.is_null() {
                        RtlFreeHeap(RtlGetProcessHeap(), 0, SearchPath as PVOID);
                        SearchPath = core::ptr::null_mut();
                    }

                    /* Now compute the final EXE path based on the name */
                    SearchPath = BaseComputeProcessExePath(lpApplicationName as LPWSTR);
                    if SearchPath.is_null() {
                        SetLastError(ERROR_NOT_ENOUGH_MEMORY);
                        Result = FALSE;
                        break 'quickie;
                    }

                    /* And search for the executable in the search path */
                    Length = SearchPathW(
                        SearchPath,
                        lpApplicationName,
                        (['.' as u16, 'e' as u16, 'x' as u16, 'e' as u16, 0]).as_ptr(),
                        MAX_PATH,
                        NameBuffer,
                        core::ptr::null_mut(),
                    );

                    /* Did we find it? */
                    if (Length != 0) && (Length < MAX_PATH) {
                        /* Get file attributes */
                        FileAttribs = GetFileAttributesW(NameBuffer);
                        if (FileAttribs != INVALID_FILE_ATTRIBUTES)
                            && (FileAttribs & FILE_ATTRIBUTE_DIRECTORY != 0)
                        {
                            /* This was a directory, fail later on */
                            Length = 0;
                        } else {
                            /* It's a file! */
                            Length += 1;
                        }
                    }

                    /* Check if there was a failure in SearchPathW */
                    if (Length != 0) && (Length < MAX_PATH) {
                        /* Everything looks good, restore the name */
                        *NullBuffer = SaveChar;
                        lpApplicationName = NameBuffer;
                        break 'start_scan;
                    } else {
                        /* Check if this was a relative path, which would explain it */
                        PathType = RtlDetermineDosPathNameType_U(lpApplicationName);
                        if PathType != RtlPathTypeRelative {
                            /* This should fail, and give us a detailed LastError */
                            FileHandle = CreateFileW(
                                lpApplicationName,
                                GENERIC_READ,
                                FILE_SHARE_READ | FILE_SHARE_WRITE,
                                core::ptr::null_mut(),
                                OPEN_EXISTING,
                                FILE_ATTRIBUTE_NORMAL,
                                core::ptr::null_mut(),
                            );
                            if FileHandle != INVALID_HANDLE_VALUE {
                                /* It worked? Return a generic error */
                                CloseHandle(FileHandle);
                                FileHandle = core::ptr::null_mut();
                                BaseSetLastNTError(STATUS_OBJECT_NAME_NOT_FOUND);
                            }
                        } else {
                            /* Path was absolute, which means it doesn't exist */
                            BaseSetLastNTError(STATUS_OBJECT_NAME_NOT_FOUND);
                        }

                        /* Did we already fail once? */
                        if ErrorCode != 0 {
                            /* Set the error code */
                            SetLastError(ErrorCode);
                        } else {
                            /* Not yet, cache it */
                            ErrorCode = GetLastError();
                        }

                        /* Put back the command line */
                        *NullBuffer = SaveChar;
                        lpApplicationName = NameBuffer;

                        /* It's possible there's whitespace in the directory name */
                        if (*ScanString == 0) || (SearchRetry == 0) {
                            /* Not the case, give up completely */
                            Result = FALSE;
                            break 'quickie;
                        }

                        /* There are spaces, so keep trying the next possibility */
                        ScanString = ScanString.add(1);
                        NullBuffer = ScanString;

                        /* We will have to add a quote, since there is a space */
                        QuotesNeeded = TRUE as BOOLEAN;
                        HasQuotes = TRUE as BOOLEAN;
                        continue 'start_scan; /* goto StartScan */
                    }
                }
            } else if lpCommandLine.is_null() || (*lpCommandLine == 0) {
                /* We don't have a command line, so just use the application name */
                CmdLineIsAppName = TRUE as BOOLEAN;
                lpCommandLine = lpApplicationName as LPWSTR;
            }

            /* Convert the application name to its NT path */
            TranslationStatus = RtlDosPathNameToRelativeNtPathName_U(
                lpApplicationName,
                &mut PathName,
                core::ptr::null_mut(),
                &mut SxsWin32RelativePath,
            );
            if TranslationStatus == 0 {
                /* Path must be invalid somehow, bail out */
                SetLastError(ERROR_PATH_NOT_FOUND);
                Result = FALSE;
                break 'quickie;
            }

            /* Setup the buffer that needs to be freed at the end */
            debug_assert!(FreeBuffer.is_null());
            FreeBuffer = PathName.Buffer;

            /* Check what kind of path the application is, for SxS (Fusion) purposes */
            RtlInitUnicodeString(&mut SxsWin32ExePath, lpApplicationName);
            SxsPathType = RtlDetermineDosPathNameType_U(lpApplicationName);
            if (SxsPathType != RtlPathTypeDriveAbsolute)
                && (SxsPathType != RtlPathTypeLocalDevice)
                && (SxsPathType != RtlPathTypeRootLocalDevice)
                && (SxsPathType != RtlPathTypeUncAbsolute)
            {
                /* Relative-type path, get the full path */
                RtlInitEmptyUnicodeString(&mut PathBufferString, core::ptr::null_mut(), 0);
                Status = RtlGetFullPathName_UstrEx(
                    &mut SxsWin32ExePath,
                    core::ptr::null_mut(),
                    &mut PathBufferString,
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                    &mut SxsPathType,
                    core::ptr::null_mut(),
                );
                if !NT_SUCCESS(Status) {
                    /* Fail the rest of the create */
                    RtlReleaseRelativeName(&mut SxsWin32RelativePath);
                    BaseSetLastNTError(Status);
                    Result = FALSE;
                    break 'quickie;
                }

                /* Use this full path as the SxS path */
                SxsWin32ExePath = PathBufferString;
                PathBuffer = PathBufferString.Buffer;
                PathBufferString.Buffer = core::ptr::null_mut();
            }

            /* Also set the .EXE path based on the path name */
            if SxsWin32RelativePath.RelativeName.Length != 0 {
                /* If it's relative, capture the relative name */
                PathName = SxsWin32RelativePath.RelativeName;
            } else {
                /* Otherwise, it's absolute, make sure no relative dir is used */
                SxsWin32RelativePath.ContainingDirectory = core::ptr::null_mut();
            }

            /* Now use the path name, and the root path, to try opening the app */
            /* InitializeObjectAttributes(&LocalObjectAttributes, &PathName,
             *   OBJ_CASE_INSENSITIVE, ContainingDirectory, NULL) */
            LocalObjectAttributes.Length = core::mem::size_of::<OBJECT_ATTRIBUTES>() as ULONG;
            LocalObjectAttributes.RootDirectory = SxsWin32RelativePath.ContainingDirectory;
            LocalObjectAttributes.ObjectName = &mut PathName;
            LocalObjectAttributes.Attributes = OBJ_CASE_INSENSITIVE;
            LocalObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
            LocalObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

            Status = NtOpenFile(
                &mut FileHandle,
                SYNCHRONIZE | FILE_READ_ATTRIBUTES | FILE_READ_DATA | FILE_EXECUTE,
                &mut LocalObjectAttributes,
                &mut IoStatusBlock,
                FILE_SHARE_READ | FILE_SHARE_DELETE,
                FILE_SYNCHRONOUS_IO_NONALERT | FILE_NON_DIRECTORY_FILE,
            );
            if !NT_SUCCESS(Status) {
                /* Try to open the app just for execute purposes instead */
                Status = NtOpenFile(
                    &mut FileHandle,
                    SYNCHRONIZE | FILE_EXECUTE,
                    &mut LocalObjectAttributes,
                    &mut IoStatusBlock,
                    FILE_SHARE_READ | FILE_SHARE_DELETE,
                    FILE_SYNCHRONOUS_IO_NONALERT | FILE_NON_DIRECTORY_FILE,
                );
            }

            /* Cleanup in preparation for failure or success */
            RtlReleaseRelativeName(&mut SxsWin32RelativePath);

            if !NT_SUCCESS(Status) {
                /* Failure path, try to understand why */
                if RtlIsDosDeviceName_U(lpApplicationName) != 0 {
                    /* If a device is being executed, return this special error code */
                    SetLastError(ERROR_BAD_DEVICE);
                    Result = FALSE;
                    break 'quickie;
                } else {
                    /* Otherwise return the converted NT error code */
                    BaseSetLastNTError(Status);
                    Result = FALSE;
                    break 'quickie;
                }
            }

            /* Did the caller specify a desktop? */
            if StartupInfo.lpDesktop.is_null() {
                /* Use the one from the current process */
                StartupInfo.lpDesktop = (*(*Peb).ProcessParameters).DesktopInfo.Buffer;
            }

            /* Create a section for this file */
            Status = NtCreateSection(
                &mut SectionHandle,
                SECTION_ALL_ACCESS,
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                PAGE_EXECUTE,
                SEC_IMAGE,
                FileHandle,
            );
            if NT_SUCCESS(Status) {
                /* Are we running on Windows Embedded, Datacenter, Blade or Starter? */
                if SharedUserData_SuiteMask()
                    & (VER_SUITE_EMBEDDEDNT | VER_SUITE_DATACENTER | VER_SUITE_PERSONAL | VER_SUITE_BLADE)
                    != 0
                {
                    /* These SKUs do not allow running certain applications */
                    Status = BasepCheckWebBladeHashes(FileHandle);
                    if Status == STATUS_ACCESS_DENIED {
                        /* And this is one of them! */
                        SetLastError(ERROR_ACCESS_DISABLED_WEBBLADE);
                        Result = FALSE;
                        break 'quickie;
                    }

                    /* Did we get some other failure? */
                    if !NT_SUCCESS(Status) {
                        /* If we couldn't check the hashes, assume nefariousness */
                        SetLastError(ERROR_ACCESS_DISABLED_WEBBLADE_TAMPER);
                        Result = FALSE;
                        break 'quickie;
                    }
                }

                /* Now do Winsafer, etc, checks */
                Status = BasepIsProcessAllowed(lpApplicationName as LPWSTR);
                if !NT_SUCCESS(Status) {
                    /* Fail if we're not allowed to launch the process */
                    BaseSetLastNTError(Status);
                    if !SectionHandle.is_null() {
                        NtClose(SectionHandle);
                        SectionHandle = core::ptr::null_mut();
                    }
                    Result = FALSE;
                    break 'quickie;
                }

                /* Is a DOS VDM being forced, but we already have a WOW32 instance ready? */
                if (dwCreationFlags & CREATE_FORCEDOS != 0)
                    && ((*BaseStaticServerData).IsWowTaskReady != 0)
                {
                    /* This request can't be satisfied, instead, a separate VDM is needed */
                    dwCreationFlags &= !(CREATE_FORCEDOS | CREATE_SHARED_WOW_VDM);
                    dwCreationFlags |= CREATE_SEPARATE_WOW_VDM;

                    /* Set a failure code, ask for VDM reservation */
                    Status = STATUS_INVALID_IMAGE_WIN_16;
                    UseVdmReserve = TRUE as BOOLEAN;

                    /* Close the current handle */
                    NtClose(SectionHandle);
                    SectionHandle = core::ptr::null_mut();

                    /* Don't query the section later */
                    QuerySection = FALSE as BOOLEAN;
                }
            }

            /* Did we already do these checks? */
            if SkipSaferAndAppCompat == 0 {
                /* Is everything OK so far, OR do we have an non-MZ, non-DOS app? */
                if NT_SUCCESS(Status)
                    || ((Status == STATUS_INVALID_IMAGE_NOT_MZ)
                        && (BaseIsDosApplication(&mut PathName, Status) == 0))
                {
                    /* Clear the machine type in case of failure */
                    ImageMachine = 0;

                    /* Clean any app compat data that may have accumulated */
                    BasepFreeAppCompatData(AppCompatData, AppCompatSxsData);
                    AppCompatData = core::ptr::null_mut();
                    AppCompatSxsData = core::ptr::null_mut();

                    /* Do we have a section? */
                    if !SectionHandle.is_null() {
                        /* Have we already queried it? */
                        if QuerySection != 0 {
                            /* Nothing to do */
                            AppCompatStatus = STATUS_SUCCESS;
                        } else {
                            /* Get some information about the executable */
                            AppCompatStatus = NtQuerySection(
                                SectionHandle,
                                SectionImageInformation,
                                &mut ImageInformation as *mut _ as PVOID,
                                core::mem::size_of::<SECTION_IMAGE_INFORMATION>() as SIZE_T,
                                core::ptr::null_mut(),
                            );
                        }

                        /* Do we have section information now? */
                        if NT_SUCCESS(AppCompatStatus) {
                            /* Don't ask for it again, save the machine type */
                            QuerySection = TRUE as BOOLEAN;
                            ImageMachine = ImageInformation.Machine;
                        }
                    }

                    /* Is there a reason/Shim we shouldn't run this application? */
                    AppCompatStatus = BasepCheckBadapp(
                        FileHandle,
                        FreeBuffer,
                        lpEnvironment as PWCHAR,
                        ImageMachine,
                        &mut AppCompatData,
                        &mut AppCompatDataSize,
                        &mut AppCompatSxsData,
                        &mut AppCompatSxsDataSize,
                        &mut FusionFlags,
                    );
                    if !NT_SUCCESS(AppCompatStatus) {
                        /* This is usually the status we get back */
                        if AppCompatStatus == STATUS_ACCESS_DENIED {
                            /* Convert it to something more Win32-specific */
                            SetLastError(ERROR_CANCELLED);
                        } else {
                            /* Some other error */
                            BaseSetLastNTError(AppCompatStatus);
                        }

                        /* Did we have a section? */
                        if !SectionHandle.is_null() {
                            /* Clean it up */
                            NtClose(SectionHandle);
                            SectionHandle = core::ptr::null_mut();
                        }

                        /* Fail the call */
                        Result = FALSE;
                        break 'quickie;
                    }
                }
            }

            /* Have we already done, and do we need to do, SRP (WinSafer) checks?
             * NOTE: the C uses BITWISE `~(dwCreationFlags & FLAG)` (not logical
             * `!`), so this is non-zero unless the masked value is 0xFFFFFFFF —
             * an upstream ReactOS quirk preserved faithfully. */
            if (SkipSaferAndAppCompat == 0)
                && ((!(dwCreationFlags & CREATE_PRESERVE_CODE_AUTHZ_LEVEL)) != 0)
            {
                /* Assume yes */
                SaferNeeded = TRUE as BOOLEAN;
                match Status {
                    STATUS_INVALID_IMAGE_NE_FORMAT
                    | STATUS_INVALID_IMAGE_PROTECT
                    | STATUS_INVALID_IMAGE_WIN_16
                    | STATUS_FILE_IS_OFFLINE => {
                        /* For all DOS, 16-bit, OS/2 images, we do */
                    }
                    STATUS_INVALID_IMAGE_NOT_MZ => {
                        /* For invalid files, we don't, unless it's a .BAT file */
                        if BaseIsDosApplication(&mut PathName, Status) != 0 {
                            /* break: do safer */
                        } else {
                            /* fall through to default */
                            if !NT_SUCCESS(Status) {
                                SaferNeeded = FALSE as BOOLEAN;
                            }
                        }
                    }
                    _ => {
                        /* Any other error codes we also don't */
                        if !NT_SUCCESS(Status) {
                            SaferNeeded = FALSE as BOOLEAN;
                        }
                        /* But for success, we do */
                    }
                }

                /* Okay, so what did the checks above result in? */
                if SaferNeeded != 0 {
                    /* We have to call into the WinSafer library and actually check */
                    SaferStatus = BasepCheckWinSaferRestrictions(
                        hUserToken,
                        lpApplicationName as LPWSTR,
                        FileHandle,
                        &mut InJob,
                        &mut TokenHandle,
                        &mut JobHandle,
                    );
                    if SaferStatus as u32 == 0xFFFFFFFF {
                        /* Back in 2003, they didn't have an NTSTATUS for this... */
                        SetLastError(ERROR_ACCESS_DISABLED_BY_POLICY);
                        Result = FALSE;
                        break 'quickie;
                    }

                    /* Other status codes are not-Safer related, just convert them */
                    if !NT_SUCCESS(SaferStatus) {
                        BaseSetLastNTError(SaferStatus);
                        Result = FALSE;
                        break 'quickie;
                    }
                }
            }

            /* The last step is to figure out why the section object was not created.
             * The C `switch (Status)` has fallthrough from WIN_16 (FORCEDOS case)
             * into the {PROTECT,NOT_MZ,NE_FORMAT} block. We model that with a
             * `match` whose WIN_16 arm, when it does NOT take the WOW path,
             * `continue`s into the shared executable block via a flag. */
            {
                let mut fall_into_exe_block = false;

                if Status == STATUS_INVALID_IMAGE_WIN_16 {
                    /* 16-bit binary. Should we use WOW or does the caller force VDM? */
                    if dwCreationFlags & CREATE_FORCEDOS == 0 {
                        /* Remember that we're launching WOW */
                        IsWowApp = TRUE as BOOLEAN;

                        /* Create the VDM environment, it's valid for WOW too */
                        Result = BaseCreateVDMEnvironment(
                            lpEnvironment as PWCHAR,
                            &mut VdmAnsiEnv,
                            &mut VdmUnicodeEnv,
                        );
                        if Result == 0 {
                            break 'quickie;
                        }

                        /* We're going to try this twice, so do a loop */
                        loop {
                            /* Pick which kind of WOW mode we want to run in */
                            VdmBinaryType = if dwCreationFlags & CREATE_SEPARATE_WOW_VDM != 0 {
                                BINARY_TYPE_SEPARATE_WOW
                            } else {
                                BINARY_TYPE_WOW
                            };

                            /* Get all the VDM settings and current status */
                            Status = BaseCheckVDM(
                                VdmBinaryType,
                                lpApplicationName,
                                lpCommandLine,
                                lpCurrentDirectory,
                                &mut VdmAnsiEnv,
                                &mut CsrMsg[1],
                                &mut VdmTask,
                                dwCreationFlags,
                                &mut StartupInfo,
                                hUserToken,
                            );

                            /* If it worked, no need to try again */
                            if NT_SUCCESS(Status) {
                                break;
                            }

                            /* Check if it's disallowed or if it's our second time */
                            BaseSetLastNTError(Status);
                            if (Status == STATUS_VDM_DISALLOWED)
                                || (VdmBinaryType == BINARY_TYPE_SEPARATE_WOW)
                                || (GetLastError() == ERROR_ACCESS_DENIED)
                            {
                                /* Fail the call -- we won't try again */
                                Result = FALSE;
                                break 'quickie;
                            }

                            /* Try one more time, but with a separate WOW instance */
                            dwCreationFlags |= CREATE_SEPARATE_WOW_VDM;
                        }

                        /* Check which VDM state we're currently in */
                        match (*CheckVdmMsg).VDMState as ULONG
                            & (VDM_NOT_LOADED | VDM_NOT_READY | VDM_READY)
                        {
                            VDM_NOT_LOADED => {
                                /* VDM is not fully loaded, so not that much to undo */
                                VdmUndoLevel = VDM_UNDO_PARTIAL;

                                /* Reset VDM reserve if needed */
                                if UseVdmReserve != 0 {
                                    VdmReserve = 1;
                                }

                                /* Get the required parameters and names for launch */
                                Result = BaseGetVdmConfigInfo(
                                    lpCommandLine,
                                    VdmTask,
                                    VdmBinaryType,
                                    &mut VdmString,
                                    &mut VdmReserve,
                                );
                                if Result == 0 {
                                    BaseSetLastNTError(Status);
                                    break 'quickie;
                                }

                                /* Update the command-line with the VDM one instead */
                                lpCommandLine = VdmString.Buffer;
                                lpApplicationName = core::ptr::null_mut();

                                /* We don't want a console, detachment, nor a window */
                                dwCreationFlags |= CREATE_NO_WINDOW;
                                dwCreationFlags &= !(CREATE_NEW_CONSOLE | DETACHED_PROCESS);

                                /* Force feedback on */
                                StartupInfo.dwFlags |= STARTF_FORCEONFEEDBACK;
                            }
                            VDM_READY => {
                                /* VDM is ready, so we have to undo everything */
                                VdmUndoLevel = VDM_UNDO_REUSE;

                                /* Check if CSRSS wants us to wait on VDM */
                                VdmWaitObject = (*CheckVdmMsg).WaitObjectForParent;
                            }
                            VDM_NOT_READY => {
                                /* Something is wrong with VDM, we'll fail the call */
                                SetLastError(ERROR_NOT_READY);
                                Result = FALSE;
                                break 'quickie;
                            }
                            _ => {}
                        }

                        /* Since to get NULL, we allocate from 0x1, account for this */
                        VdmReserve = VdmReserve.wrapping_sub(1);

                        /* This implies VDM is ready, so skip everything else */
                        if !VdmWaitObject.is_null() {
                            break 'app_name_retry; /* goto VdmShortCircuit */
                        }

                        /* Don't inherit handles since we're doing VDM now */
                        bInheritHandles = FALSE;

                        /* Had the user passed in environment? If so, destroy it */
                        if (!lpEnvironment.is_null())
                            && (dwCreationFlags & CREATE_UNICODE_ENVIRONMENT == 0)
                        {
                            RtlDestroyEnvironment(lpEnvironment as *mut u16);
                        }

                        /* We've already done all these checks, don't do them again */
                        SkipSaferAndAppCompat = TRUE as BOOLEAN;
                        continue 'app_name_retry; /* goto AppNameRetry */
                    }

                    /* No break here on purpose, so FORCEDOS drops down! */
                    fall_into_exe_block = true;
                }

                if fall_into_exe_block
                    || Status == STATUS_INVALID_IMAGE_PROTECT
                    || Status == STATUS_INVALID_IMAGE_NOT_MZ
                    || Status == STATUS_INVALID_IMAGE_NE_FORMAT
                {
                    /* We're launching an executable application */
                    BinarySubType = BINARY_TYPE_EXE;

                    /* We can drop here from other "cases" above too, so check.
                     * NOTE: the C assigns BinarySubType inside the condition. */
                    let is_dos_app = if (Status == STATUS_INVALID_IMAGE_PROTECT)
                        || (Status == STATUS_INVALID_IMAGE_NE_FORMAT)
                    {
                        true
                    } else {
                        BinarySubType = BaseIsDosApplication(&mut PathName, Status);
                        BinarySubType != 0
                    };

                    if is_dos_app {
                        /* We're launching a DOS application */
                        VdmBinaryType = BINARY_TYPE_DOS;

                        /* Based on the caller environment, create a VDM one */
                        Result = BaseCreateVDMEnvironment(
                            lpEnvironment as PWCHAR,
                            &mut VdmAnsiEnv,
                            &mut VdmUnicodeEnv,
                        );
                        if Result == 0 {
                            break 'quickie;
                        }

                        /* Check the current state of the VDM subsystem */
                        Status = BaseCheckVDM(
                            VdmBinaryType | BinarySubType,
                            lpApplicationName,
                            lpCommandLine,
                            lpCurrentDirectory,
                            &mut VdmAnsiEnv,
                            &mut CsrMsg[1],
                            &mut VdmTask,
                            dwCreationFlags,
                            &mut StartupInfo,
                            core::ptr::null_mut(),
                        );
                        if !NT_SUCCESS(Status) {
                            /* Failed to inquire about VDM, fail the call */
                            BaseSetLastNTError(Status);
                            Result = FALSE;
                            break 'quickie;
                        }

                        /* Handle possible VDM states */
                        match (*CheckVdmMsg).VDMState as ULONG
                            & (VDM_NOT_LOADED | VDM_NOT_READY | VDM_READY)
                        {
                            VDM_NOT_LOADED => {
                                /* If VDM is not loaded, we'll do a partial undo */
                                VdmUndoLevel = VDM_UNDO_PARTIAL;

                                /* A VDM process can't also be detached, so fail */
                                if dwCreationFlags & DETACHED_PROCESS != 0 {
                                    SetLastError(ERROR_ACCESS_DENIED);
                                    return FALSE;
                                }

                                /* Get the required parameters and names for launch */
                                Result = BaseGetVdmConfigInfo(
                                    lpCommandLine,
                                    VdmTask,
                                    VdmBinaryType,
                                    &mut VdmString,
                                    &mut VdmReserve,
                                );
                                if Result == 0 {
                                    BaseSetLastNTError(Status);
                                    break 'quickie;
                                }

                                /* Update the command-line to launch VDM instead */
                                lpCommandLine = VdmString.Buffer;
                                lpApplicationName = core::ptr::null_mut();
                            }
                            VDM_READY => {
                                /* VDM is ready, so we have to undo everything */
                                VdmUndoLevel = VDM_UNDO_REUSE;

                                /* Check if CSRSS wants us to wait on VDM */
                                VdmWaitObject = (*CheckVdmMsg).WaitObjectForParent;
                            }
                            VDM_NOT_READY => {
                                /* Something is wrong with VDM, we'll fail the call */
                                SetLastError(ERROR_NOT_READY);
                                Result = FALSE;
                                break 'quickie;
                            }
                            _ => {}
                        }

                        /* Since to get NULL, we allocate from 0x1, account for this */
                        VdmReserve = VdmReserve.wrapping_sub(1);

                        /* This implies VDM is ready, so skip everything else */
                        if !VdmWaitObject.is_null() {
                            break 'app_name_retry; /* goto VdmShortCircuit */
                        }

                        /* Don't inherit handles since we're doing VDM now */
                        bInheritHandles = FALSE;

                        /* Had the user passed in environment? If so, destroy it */
                        if (!lpEnvironment.is_null())
                            && (dwCreationFlags & CREATE_UNICODE_ENVIRONMENT == 0)
                        {
                            RtlDestroyEnvironment(lpEnvironment as *mut u16);
                        }

                        /* Use our VDM Unicode environment instead */
                        lpEnvironment = VdmUnicodeEnv.Buffer as LPVOID;
                    } else {
                        /* It's a batch file, get the extension */
                        ExtBuffer = PathName
                            .Buffer
                            .add(PathName.Length as usize / core::mem::size_of::<WCHAR>() - 4);

                        /* Make sure the extensions are correct */
                        let bat: [u16; 5] = ['.' as u16, 'b' as u16, 'a' as u16, 't' as u16, 0];
                        let cmd: [u16; 5] = ['.' as u16, 'c' as u16, 'm' as u16, 'd' as u16, 0];
                        if (PathName.Length < (4 * core::mem::size_of::<WCHAR>()) as USHORT)
                            || ((_wcsnicmp(ExtBuffer, bat.as_ptr(), 4) != 0)
                                && (_wcsnicmp(ExtBuffer, cmd.as_ptr(), 4) != 0))
                        {
                            SetLastError(ERROR_BAD_EXE_FORMAT);
                            Result = FALSE;
                            break 'quickie;
                        }

                        /* Check if we need to account for quotes around the path */
                        CmdQuoteLength = (CmdLineIsAppName != 0 || HasQuotes != 0) as ULONG;
                        if CmdLineIsAppName == 0 {
                            if HasQuotes != 0 {
                                CmdQuoteLength += 1;
                            }
                        } else {
                            CmdQuoteLength += 1;
                        }

                        /* Calculate the length of the command line */
                        CmdLineLength = wcslen(lpCommandLine) as SIZE_T;
                        CmdLineLength += wcslen(CMD_STRING.as_ptr()) as SIZE_T;
                        CmdLineLength += CmdQuoteLength as SIZE_T + ANSI_NULL_SIZE;
                        CmdLineLength *= core::mem::size_of::<WCHAR>() as SIZE_T;

                        /* Allocate space for the new command line */
                        AnsiCmdCommand =
                            RtlAllocateHeap(RtlGetProcessHeap(), 0, CmdLineLength) as PWCHAR;
                        if AnsiCmdCommand.is_null() {
                            BaseSetLastNTError(STATUS_NO_MEMORY);
                            Result = FALSE;
                            break 'quickie;
                        }

                        /* Build it */
                        wcscpy(AnsiCmdCommand, CMD_STRING.as_ptr());
                        let quote: [u16; 2] = ['"' as u16, 0];
                        if (CmdLineIsAppName != 0) || (HasQuotes != 0) {
                            wcscat(AnsiCmdCommand, quote.as_ptr());
                        }
                        wcscat(AnsiCmdCommand, lpCommandLine);
                        if (CmdLineIsAppName != 0) || (HasQuotes != 0) {
                            wcscat(AnsiCmdCommand, quote.as_ptr());
                        }

                        /* Create it as a Unicode String */
                        RtlInitUnicodeString(&mut DebuggerString, AnsiCmdCommand);

                        /* Set the command line to this */
                        lpCommandLine = DebuggerString.Buffer;
                        lpApplicationName = core::ptr::null_mut();
                    }

                    /* We've already done all these checks, don't do them again */
                    SkipSaferAndAppCompat = TRUE as BOOLEAN;
                    continue 'app_name_retry; /* goto AppNameRetry */
                } else if Status == STATUS_INVALID_IMAGE_WIN_64 {
                    /* 64-bit binaries are not allowed to run on 32-bit ReactOS */
                    SetLastError(ERROR_EXE_MACHINE_TYPE_MISMATCH);
                    Result = FALSE;
                    break 'quickie;
                } else if Status == STATUS_FILE_IS_OFFLINE {
                    /* Set the correct last error for this */
                    SetLastError(ERROR_FILE_OFFLINE);
                    /* break (fall through to rest of function) */
                } else {
                    /* Any other error, convert it to a generic Win32 error */
                    if !NT_SUCCESS(Status) {
                        SetLastError(ERROR_BAD_EXE_FORMAT);
                        Result = FALSE;
                        break 'quickie;
                    }

                    /* Otherwise, this must be success */
                    debug_assert!(Status == STATUS_SUCCESS);
                }
            }

            /* Is this not a WOW application, but a WOW32 VDM was requested for it? */
            if (IsWowApp == 0) && (dwCreationFlags & CREATE_SEPARATE_WOW_VDM != 0) {
                /* Ignore the nonsensical request */
                dwCreationFlags &= !CREATE_SEPARATE_WOW_VDM;
            }

            /* Did we already check information for the section? */
            if QuerySection == 0 {
                /* Get some information about the executable */
                Status = NtQuerySection(
                    SectionHandle,
                    SectionImageInformation,
                    &mut ImageInformation as *mut _ as PVOID,
                    core::mem::size_of::<SECTION_IMAGE_INFORMATION>() as SIZE_T,
                    core::ptr::null_mut(),
                );
                if !NT_SUCCESS(Status) {
                    /* We failed, bail out */
                    BaseSetLastNTError(Status);
                    Result = FALSE;
                    break 'quickie;
                }

                /* Don't check this later */
                QuerySection = TRUE as BOOLEAN;
            }

            /* Check if this was linked as a DLL */
            if ImageInformation.ImageCharacteristics & IMAGE_FILE_DLL as USHORT != 0 {
                /* These aren't valid images to try to execute! */
                SetLastError(ERROR_BAD_EXE_FORMAT);
                Result = FALSE;
                break 'quickie;
            }

            /* Don't let callers pass in this flag -- we'll only get it from IFEO */
            Flags &= !PROCESS_CREATE_FLAGS_LARGE_PAGES;

            /* Clear the IFEO-missing flag, before we know for sure... */
            ParameterFlags &= !2;

            /* If the process is being debugged, only read IFEO if the PEB says so */
            if (dwCreationFlags & (DEBUG_PROCESS | DEBUG_ONLY_THIS_PROCESS) == 0)
                || ((*Peb).ReadImageFileExecOptions != 0)
            {
                /* Let's do this! Attempt to open IFEO */
                IFEOStatus = LdrOpenImageFileOptionsKey(&mut PathName, 0, &mut KeyHandle);
                if !NT_SUCCESS(IFEOStatus) {
                    /* We failed, set the flag so we store this in the parameters */
                    if IFEOStatus == STATUS_OBJECT_NAME_NOT_FOUND {
                        ParameterFlags |= 2;
                    }
                } else {
                    /* Was this our first time going through this path? */
                    if DebuggerCmdLine.is_null() {
                        /* Allocate a buffer for the debugger path */
                        DebuggerCmdLine = RtlAllocateHeap(
                            RtlGetProcessHeap(),
                            0,
                            MAX_PATH as SIZE_T * core::mem::size_of::<WCHAR>() as SIZE_T,
                        ) as PWCHAR;
                        if DebuggerCmdLine.is_null() {
                            /* Close IFEO on failure */
                            IFEOStatus = NtClose(KeyHandle);
                            debug_assert!(NT_SUCCESS(IFEOStatus));

                            /* Fail the call */
                            SetLastError(ERROR_NOT_ENOUGH_MEMORY);
                            Result = FALSE;
                            break 'quickie;
                        }
                    }

                    /* Now query for the debugger */
                    let dbg_name: [u16; 9] = [
                        'D' as u16, 'e' as u16, 'b' as u16, 'u' as u16, 'g' as u16, 'g' as u16,
                        'e' as u16, 'r' as u16, 0,
                    ];
                    IFEOStatus = LdrQueryImageFileKeyOption(
                        KeyHandle,
                        dbg_name.as_ptr(),
                        REG_SZ,
                        DebuggerCmdLine as PVOID,
                        MAX_PATH as ULONG * core::mem::size_of::<WCHAR>() as ULONG,
                        &mut ResultSize,
                    );
                    if (!NT_SUCCESS(IFEOStatus))
                        || (ResultSize < core::mem::size_of::<WCHAR>() as ULONG)
                        || (*DebuggerCmdLine == UNICODE_NULL)
                    {
                        /* If it's not there, or too small, or invalid, ignore it */
                        RtlFreeHeap(RtlGetProcessHeap(), 0, DebuggerCmdLine as PVOID);
                        DebuggerCmdLine = core::ptr::null_mut();
                    }

                    /* Also query if we should map with large pages */
                    let ulp_name: [u16; 14] = [
                        'U' as u16, 's' as u16, 'e' as u16, 'L' as u16, 'a' as u16, 'r' as u16,
                        'g' as u16, 'e' as u16, 'P' as u16, 'a' as u16, 'g' as u16, 'e' as u16,
                        's' as u16, 0,
                    ];
                    IFEOStatus = LdrQueryImageFileKeyOption(
                        KeyHandle,
                        ulp_name.as_ptr(),
                        REG_DWORD,
                        &mut UseLargePages as *mut BOOLEAN as PVOID,
                        core::mem::size_of_val(&UseLargePages) as ULONG,
                        core::ptr::null_mut(),
                    );
                    if NT_SUCCESS(IFEOStatus) && (UseLargePages != 0) {
                        /* Do it! This is the only way this flag can be set */
                        Flags |= PROCESS_CREATE_FLAGS_LARGE_PAGES;
                    }

                    /* We're done with IFEO, can close it now */
                    IFEOStatus = NtClose(KeyHandle);
                    debug_assert!(NT_SUCCESS(IFEOStatus));
                }
            }

            /* Make sure the image was compiled for this processor */
            if (ImageInformation.Machine < SharedUserData_ImageNumberLow())
                || (ImageInformation.Machine > SharedUserData_ImageNumberHigh())
            {
                /* It was not -- raise a hard error */
                ErrorResponse = ResponseOk as ULONG;
                ErrorParameters[0] = &mut PathName as *mut UNICODE_STRING as ULONG_PTR;
                NtRaiseHardError(
                    STATUS_IMAGE_MACHINE_TYPE_MISMATCH_EXE,
                    1,
                    1,
                    ErrorParameters.as_mut_ptr(),
                    OptionOk as ULONG,
                    &mut ErrorResponse,
                );
                if (*Peb).ImageSubsystemMajorVersion <= 3 {
                    /* If it's really old, return this error */
                    SetLastError(ERROR_BAD_EXE_FORMAT);
                } else {
                    /* Otherwise, return a more modern error */
                    SetLastError(ERROR_EXE_MACHINE_TYPE_MISMATCH);
                }

                /* Go to the failure path */
                Result = FALSE;
                break 'quickie;
            }

            /* Check if this isn't a Windows image */
            if (ImageInformation.SubSystemType != IMAGE_SUBSYSTEM_WINDOWS_GUI)
                && (ImageInformation.SubSystemType != IMAGE_SUBSYSTEM_WINDOWS_CUI)
            {
                /* Get rid of section-related information since we'll retry */
                NtClose(SectionHandle);
                SectionHandle = core::ptr::null_mut();
                QuerySection = FALSE as BOOLEAN;

                /* The only other non-Windows image type we support here is POSIX */
                if ImageInformation.SubSystemType != IMAGE_SUBSYSTEM_POSIX_CUI {
                    /* Bail out if it's something else */
                    SetLastError(ERROR_CHILD_NOT_COMPLETE);
                    Result = FALSE;
                    break 'quickie;
                }

                /* Now build the command-line to have posix launch this image */
                let posix: [u16; 10] = [
                    'P' as u16, 'O' as u16, 'S' as u16, 'I' as u16, 'X' as u16, ' ' as u16,
                    '/' as u16, 'P' as u16, ' ' as u16, 0,
                ];
                Result = BuildSubSysCommandLine(
                    posix.as_ptr(),
                    lpApplicationName,
                    lpCommandLine,
                    &mut DebuggerString,
                ) as BOOL;
                if Result == 0 {
                    /* Bail out if that failed */
                    break 'quickie;
                }

                /* And re-try launching the process, with the new command-line now */
                lpCommandLine = DebuggerString.Buffer;
                lpApplicationName = core::ptr::null_mut();

                /* We've already done all these checks, don't do them again */
                SkipSaferAndAppCompat = TRUE as BOOLEAN;
                continue 'app_name_retry; /* goto AppNameRetry */
            }

            /* Was this image built for a version of Windows whose images we can run? */
            Result = BasepIsImageVersionOk(
                ImageInformation.__bindgen_anon_1.__bindgen_anon_1.SubSystemMajorVersion as ULONG,
                ImageInformation.__bindgen_anon_1.__bindgen_anon_1.SubSystemMinorVersion as ULONG,
            ) as BOOL;
            if Result == 0 {
                /* It was not, bail out */
                SetLastError(ERROR_BAD_EXE_FORMAT);
                break 'quickie;
            }

            /* Check if there is a debugger associated with the application */
            if !DebuggerCmdLine.is_null() {
                /* Get the length of the command line */
                n = wcslen(lpCommandLine) as SIZE_T;
                if n == 0 {
                    /* There's no command line, use the application name instead */
                    lpCommandLine = lpApplicationName as LPWSTR;
                    n = wcslen(lpCommandLine) as SIZE_T;
                }

                /* Protect against overflow */
                if n > UNICODE_STRING_MAX_CHARS as SIZE_T {
                    BaseSetLastNTError(STATUS_NAME_TOO_LONG);
                    Result = FALSE;
                    break 'quickie;
                }

                /* Now add the length of the debugger command-line */
                n += wcslen(DebuggerCmdLine) as SIZE_T;

                /* Again make sure we don't overflow */
                if n > UNICODE_STRING_MAX_CHARS as SIZE_T {
                    BaseSetLastNTError(STATUS_NAME_TOO_LONG);
                    Result = FALSE;
                    break 'quickie;
                }

                /* Account for the quotes and space between the two:
                 * sizeof("\" \"") - sizeof(ANSI_NULL) == 4 - 1 == 3 */
                n += 3;

                /* Convert to bytes, and make sure we don't overflow */
                n *= core::mem::size_of::<WCHAR>() as SIZE_T;
                if n > UNICODE_STRING_MAX_BYTES_LOCAL {
                    BaseSetLastNTError(STATUS_NAME_TOO_LONG);
                    Result = FALSE;
                    break 'quickie;
                }

                /* Allocate space for the string */
                DebuggerString.Buffer = RtlAllocateHeap(RtlGetProcessHeap(), 0, n) as PWSTR;
                if DebuggerString.Buffer.is_null() {
                    SetLastError(ERROR_NOT_ENOUGH_MEMORY);
                    Result = FALSE;
                    break 'quickie;
                }

                /* Set the length */
                RtlInitEmptyUnicodeString(&mut DebuggerString, DebuggerString.Buffer, n as USHORT);

                /* Now perform the command line creation */
                ImageDbgStatus = RtlAppendUnicodeToString(&mut DebuggerString, DebuggerCmdLine);
                debug_assert!(NT_SUCCESS(ImageDbgStatus));
                let space: [u16; 2] = [' ' as u16, 0];
                ImageDbgStatus = RtlAppendUnicodeToString(&mut DebuggerString, space.as_ptr());
                debug_assert!(NT_SUCCESS(ImageDbgStatus));
                ImageDbgStatus = RtlAppendUnicodeToString(&mut DebuggerString, lpCommandLine);
                debug_assert!(NT_SUCCESS(ImageDbgStatus));

                /* DbgPrint: "BASE: Calling debugger with '%wZ'" — dropped */

                /* Update the command line and application name */
                lpCommandLine = DebuggerString.Buffer;
                lpApplicationName = core::ptr::null_mut();

                /* Close all temporary state */
                NtClose(SectionHandle);
                SectionHandle = core::ptr::null_mut();
                QuerySection = FALSE as BOOLEAN;

                /* Free all temporary memory */
                RtlFreeHeap(RtlGetProcessHeap(), 0, NameBuffer as PVOID);
                NameBuffer = core::ptr::null_mut();
                RtlFreeHeap(RtlGetProcessHeap(), 0, FreeBuffer as PVOID);
                FreeBuffer = core::ptr::null_mut();
                RtlFreeHeap(RtlGetProcessHeap(), 0, DebuggerCmdLine as PVOID);
                DebuggerCmdLine = core::ptr::null_mut();
                continue 'app_name_retry; /* goto AppNameRetry */
            }

            /* Initialize the process object attributes */
            ObjectAttributes = BaseFormatObjectAttributes(
                &mut LocalObjectAttributes,
                lpProcessAttributes,
                core::ptr::null_mut(),
            );
            if (!hUserToken.is_null()) && (!lpProcessAttributes.is_null()) {
                /* Augment them with information from the user */
                LocalProcessAttributes = *lpProcessAttributes;
                LocalProcessAttributes.lpSecurityDescriptor = core::ptr::null_mut();
                ObjectAttributes = BaseFormatObjectAttributes(
                    &mut LocalObjectAttributes,
                    &mut LocalProcessAttributes,
                    core::ptr::null_mut(),
                );
            }

            /* CREATE_BREAKAWAY_FROM_JOB allows the child process to break
             * away from the job associated with the calling process (upstream
             * 55563444377; the old C keyed this off DEBUG_PROCESS) */
            if dwCreationFlags & CREATE_BREAKAWAY_FROM_JOB != 0 {
                Flags |= PROCESS_CREATE_FLAGS_BREAKAWAY;
            }

            /* Check if we're going to be debugged */
            if dwCreationFlags & (DEBUG_PROCESS | DEBUG_ONLY_THIS_PROCESS) != 0 {
                /* Connect to DbgUi */
                Status = DbgUiConnectToDbg();
                if !NT_SUCCESS(Status) {
                    BaseSetLastNTError(Status);
                    Result = FALSE;
                    break 'quickie;
                }

                /* Get the debug object */
                DebugHandle = DbgUiGetThreadDebugObject();

                /* Check if only this process will be debugged */
                if dwCreationFlags & DEBUG_ONLY_THIS_PROCESS != 0 {
                    /* Set process flag */
                    Flags |= PROCESS_CREATE_FLAGS_NO_DEBUG_INHERIT;
                }
            }

            /* Set inherit flag */
            if bInheritHandles != 0 {
                Flags |= PROCESS_CREATE_FLAGS_INHERIT_HANDLES;
            }

            /* Check if the process should be created with large pages */
            HavePrivilege = FALSE as BOOLEAN;
            PrivilegeState = core::ptr::null_mut();
            if Flags & PROCESS_CREATE_FLAGS_LARGE_PAGES != 0 {
                /* Acquire the required privilege so that the kernel won't fail the call */
                PrivilegeValue = SE_LOCK_MEMORY_PRIVILEGE;
                Status = RtlAcquirePrivilege(&mut PrivilegeValue, 1, 0, &mut PrivilegeState);
                if NT_SUCCESS(Status) {
                    /* Remember to release it later */
                    HavePrivilege = TRUE as BOOLEAN;
                }
            }

            /* Save the current TIB value since kernel overwrites it to store PEB */
            TibValue = (*Teb).NtTib.ArbitraryUserPointer;

            /* Tell the kernel to create the process */
            Status = NtCreateProcessEx(
                &mut ProcessHandle,
                PROCESS_ALL_ACCESS_LOCAL,
                ObjectAttributes,
                NtCurrentProcess(),
                Flags,
                SectionHandle,
                DebugHandle,
                core::ptr::null_mut(),
                InJob,
            );

            /* Load the PEB address from the hacky location where kernel stores it */
            RemotePeb = (*Teb).NtTib.ArbitraryUserPointer as PPEB;

            /* And restore the old TIB value */
            (*Teb).NtTib.ArbitraryUserPointer = TibValue;

            /* Release the large page privilege if we had acquired it */
            if HavePrivilege != 0 {
                RtlReleasePrivilege(PrivilegeState);
            }

            /* And now check if the kernel failed to create the process */
            if !NT_SUCCESS(Status) {
                /* Go to failure path */
                BaseSetLastNTError(Status);
                Result = FALSE;
                break 'quickie;
            }

            /* Check if there is a priority class to set */
            if PriorityClass.PriorityClass != 0 {
                /* Reset current privilege state */
                RealTimePrivilegeState = core::ptr::null_mut();

                /* Is realtime priority being requested? */
                if PriorityClass.PriorityClass == PROCESS_PRIORITY_CLASS_REALTIME as UCHAR {
                    /* Check if the caller has real-time access, and enable it if so */
                    RealTimePrivilegeState = BasepIsRealtimeAllowed(TRUE as BOOLEAN);
                }

                /* Set the new priority class and release the privilege */
                Status = NtSetInformationProcess(
                    ProcessHandle,
                    ProcessPriorityClass,
                    &mut PriorityClass as *mut _ as PVOID,
                    core::mem::size_of::<PROCESS_PRIORITY_CLASS>() as ULONG,
                );
                if !RealTimePrivilegeState.is_null() {
                    RtlReleasePrivilege(RealTimePrivilegeState);
                }

                /* Check if we failed to set the priority class */
                if !NT_SUCCESS(Status) {
                    /* Bail out on failure */
                    BaseSetLastNTError(Status);
                    Result = FALSE;
                    break 'quickie;
                }
            }

            /* Check if the caller wants the default error mode */
            if dwCreationFlags & CREATE_DEFAULT_ERROR_MODE != 0 {
                /* Set Error Mode to only fail on critical errors */
                let mut HardErrorMode: ULONG = SEM_FAILCRITICALERRORS;
                NtSetInformationProcess(
                    ProcessHandle,
                    ProcessDefaultHardErrorMode,
                    &mut HardErrorMode as *mut ULONG as PVOID,
                    core::mem::size_of::<ULONG>() as ULONG,
                );
            }

            /* Check if this was a VDM binary */
            if VdmBinaryType != 0 {
                /* Update VDM by telling it the process has now been created */
                VdmWaitObject = ProcessHandle;
                Result = BaseUpdateVDMEntry(
                    VdmEntryUpdateProcess,
                    &mut VdmWaitObject,
                    VdmTask,
                    VdmBinaryType,
                );

                if Result == 0 {
                    /* Bail out on failure */
                    VdmWaitObject = core::ptr::null_mut();
                    break 'quickie;
                }

                /* At this point, a failure means VDM has to undo all the state */
                VdmUndoLevel |= VDM_UNDO_FULL;
            }

            /* Check if VDM needed reserved low-memory */
            if VdmReserve != 0 {
                /* Reserve the requested allocation */
                RegionSize = VdmReserve as SIZE_T;
                Status = NtAllocateVirtualMemory(
                    ProcessHandle,
                    &mut BaseAddress,
                    0,
                    &mut RegionSize,
                    MEM_RESERVE,
                    PAGE_EXECUTE_READWRITE,
                );
                if !NT_SUCCESS(Status) {
                    /* Bail out on failure */
                    BaseSetLastNTError(Status);
                    Result = FALSE;
                    break 'quickie;
                }

                VdmReserve = RegionSize as ULONG;
            }

            /* Check if we've already queried information on the section */
            if QuerySection == 0 {
                /* We haven't, so get some information about the executable */
                Status = NtQuerySection(
                    SectionHandle,
                    SectionImageInformation,
                    &mut ImageInformation as *mut _ as PVOID,
                    core::mem::size_of::<SECTION_IMAGE_INFORMATION>() as SIZE_T,
                    core::ptr::null_mut(),
                );
                if !NT_SUCCESS(Status) {
                    /* Bail out on failure */
                    BaseSetLastNTError(Status);
                    Result = FALSE;
                    break 'quickie;
                }

                /* If we encounter a restart, don't re-query this information again */
                QuerySection = TRUE as BOOLEAN;
            }

            /* Do we need to apply SxS to this image? */
            if ImageInformation.DllCharacteristics & IMAGE_DLLCHARACTERISTICS_NO_ISOLATION as USHORT
                == 0
            {
                /* Too bad, we don't support this yet (DPRINT dropped) */
            }

            /* There's some SxS flag that we need to set if fusion flags have 1 set */
            if FusionFlags & 1 != 0 {
                (*CreateProcessMsg).Sxs.Flags |= 0x10;
            }

            /* Check if we have a current directory */
            if !lpCurrentDirectory.is_null() {
                /* Allocate a buffer so we can keep a Unicode copy */
                CurrentDirectory = RtlAllocateHeap(
                    RtlGetProcessHeap(),
                    0,
                    (MAX_PATH as SIZE_T * core::mem::size_of::<WCHAR>() as SIZE_T)
                        + core::mem::size_of::<WCHAR>() as SIZE_T, /* sizeof(UNICODE_NULL) */
                ) as PWCHAR;
                if CurrentDirectory.is_null() {
                    /* Bail out if this failed */
                    BaseSetLastNTError(STATUS_NO_MEMORY);
                    Result = FALSE;
                    break 'quickie;
                }

                /* Get the length in Unicode */
                Length = GetFullPathNameW(
                    lpCurrentDirectory,
                    MAX_PATH,
                    CurrentDirectory,
                    &mut FilePart,
                );
                if Length > MAX_PATH {
                    /* The directory is too long, so bail out */
                    SetLastError(ERROR_DIRECTORY);
                    Result = FALSE;
                    break 'quickie;
                }

                /* Make sure the directory is actually valid */
                FileAttribs = GetFileAttributesW(CurrentDirectory);
                if (FileAttribs == INVALID_FILE_ATTRIBUTES)
                    || (FileAttribs & FILE_ATTRIBUTE_DIRECTORY == 0)
                {
                    /* It isn't, so bail out */
                    SetLastError(ERROR_DIRECTORY);
                    Result = FALSE;
                    break 'quickie;
                }
            }

            /* Insert quotes if needed */
            if (QuotesNeeded != 0) || (CmdLineIsAppName != 0) {
                /* Allocate our buffer, plus enough space for quotes and a NULL.
                 * 2*sizeof(L'"') + sizeof(UNICODE_NULL) = 2*2 + 2 = 6 bytes. */
                QuotedCmdLine = RtlAllocateHeap(
                    RtlGetProcessHeap(),
                    0,
                    (wcslen(lpCommandLine) * core::mem::size_of::<WCHAR>()) as SIZE_T
                        + (2 * core::mem::size_of::<WCHAR>() + core::mem::size_of::<WCHAR>())
                            as SIZE_T,
                ) as PWCHAR;
                if !QuotedCmdLine.is_null() {
                    /* Copy the first quote */
                    let quote: [u16; 2] = ['"' as u16, 0];
                    wcscpy(QuotedCmdLine, quote.as_ptr());

                    /* Save the current null-character */
                    if QuotesNeeded != 0 {
                        SaveChar = *NullBuffer;
                        *NullBuffer = UNICODE_NULL;
                    }

                    /* Copy the command line and the final quote */
                    wcscat(QuotedCmdLine, lpCommandLine);
                    wcscat(QuotedCmdLine, quote.as_ptr());

                    /* Copy the null-char back */
                    if QuotesNeeded != 0 {
                        *NullBuffer = SaveChar;
                        wcscat(QuotedCmdLine, NullBuffer);
                    }
                } else {
                    /* We can't put quotes around the thing, so try it anyway */
                    if QuotesNeeded != 0 {
                        QuotesNeeded = FALSE as BOOLEAN;
                    }
                    if CmdLineIsAppName != 0 {
                        CmdLineIsAppName = FALSE as BOOLEAN;
                    }
                }
            }

            /* Use isolation if needed */
            if (*CreateProcessMsg).Sxs.Flags & 1 != 0 {
                ParameterFlags |= 1;
            }

            /* Set the new command-line if needed */
            if (QuotesNeeded != 0) || (CmdLineIsAppName != 0) {
                lpCommandLine = QuotedCmdLine;
            }

            /* Call the helper function in charge of RTL_USER_PROCESS_PARAMETERS */
            Result = BasePushProcessParameters(
                ParameterFlags,
                ProcessHandle,
                RemotePeb,
                lpApplicationName,
                CurrentDirectory,
                lpCommandLine,
                lpEnvironment,
                &mut StartupInfo,
                dwCreationFlags | NoWindow,
                bInheritHandles,
                if IsWowApp != 0 {
                    IMAGE_SUBSYSTEM_WINDOWS_GUI
                } else {
                    0
                },
                AppCompatData,
                AppCompatDataSize,
            ) as BOOL;
            if Result == 0 {
                /* The remote process would have an undefined state, so fail the call */
                break 'quickie;
            }

            /* Free the VDM command line string as it's no longer needed */
            RtlFreeUnicodeString(&mut VdmString);
            VdmString.Buffer = core::ptr::null_mut();

            /* Non-VDM console applications usually inherit handles unless specified */
            if (VdmBinaryType == 0)
                && (bInheritHandles == 0)
                && (StartupInfo.dwFlags & STARTF_USESTDHANDLES == 0)
                && (dwCreationFlags & (CREATE_NO_WINDOW | CREATE_NEW_CONSOLE | DETACHED_PROCESS)
                    == 0)
                && (ImageInformation.SubSystemType == IMAGE_SUBSYSTEM_WINDOWS_CUI)
            {
                /* Get the remote parameters */
                Status = NtReadVirtualMemory(
                    ProcessHandle,
                    &mut (*RemotePeb).ProcessParameters as *mut _ as PVOID,
                    &mut ProcessParameters as *mut _ as PVOID,
                    core::mem::size_of::<PRTL_USER_PROCESS_PARAMETERS>() as SIZE_T,
                    core::ptr::null_mut(),
                );
                if NT_SUCCESS(Status) {
                    /* Duplicate standard input unless it's a console handle */
                    if !IsConsoleHandle((*(*Peb).ProcessParameters).StandardInput) {
                        StuffStdHandle(
                            ProcessHandle,
                            (*(*Peb).ProcessParameters).StandardInput,
                            &mut (*ProcessParameters).StandardInput,
                        );
                    }

                    /* Duplicate standard output unless it's a console handle */
                    if !IsConsoleHandle((*(*Peb).ProcessParameters).StandardOutput) {
                        StuffStdHandle(
                            ProcessHandle,
                            (*(*Peb).ProcessParameters).StandardOutput,
                            &mut (*ProcessParameters).StandardOutput,
                        );
                    }

                    /* Duplicate standard error unless it's a console handle */
                    if !IsConsoleHandle((*(*Peb).ProcessParameters).StandardError) {
                        StuffStdHandle(
                            ProcessHandle,
                            (*(*Peb).ProcessParameters).StandardError,
                            &mut (*ProcessParameters).StandardError,
                        );
                    }
                }
            }

            /* Create the Thread's Stack */
            StackSize = core::cmp::max(256u32 * 1024, ImageInformation.MaximumStackSize as ULONG);
            Status = BaseCreateStack(
                ProcessHandle,
                ImageInformation.CommittedStackSize,
                StackSize as SIZE_T,
                &mut InitialTeb,
            );
            if !NT_SUCCESS(Status) {
                BaseSetLastNTError(Status);
                Result = FALSE;
                break 'quickie;
            }

            /* Create the Thread's Context */
            BaseInitializeContext(
                &mut Context,
                RemotePeb as PVOID,
                ImageInformation.TransferAddress,
                InitialTeb.StackBase,
                0,
            );

            /* Convert the thread attributes */
            ObjectAttributes = BaseFormatObjectAttributes(
                &mut LocalObjectAttributes,
                lpThreadAttributes,
                core::ptr::null_mut(),
            );
            if (!hUserToken.is_null()) && (!lpThreadAttributes.is_null()) {
                /* If the caller specified a user token, zero the security descriptor */
                LocalThreadAttributes = *lpThreadAttributes;
                LocalThreadAttributes.lpSecurityDescriptor = core::ptr::null_mut();
                ObjectAttributes = BaseFormatObjectAttributes(
                    &mut LocalObjectAttributes,
                    &mut LocalThreadAttributes,
                    core::ptr::null_mut(),
                );
            }

            /* Create the Kernel Thread Object */
            Status = NtCreateThread(
                &mut ThreadHandle,
                THREAD_ALL_ACCESS_LOCAL,
                ObjectAttributes,
                ProcessHandle,
                &mut ClientId,
                &mut Context,
                &mut InitialTeb,
                TRUE as BOOLEAN,
            );
            if !NT_SUCCESS(Status) {
                /* A process is not allowed to exist without a main thread, so fail */
                BaseSetLastNTError(Status);
                Result = FALSE;
                break 'quickie;
            }

            /* Begin filling out the CSRSS message, first with our IDs and handles */
            (*CreateProcessMsg).ProcessHandle = ProcessHandle;
            (*CreateProcessMsg).ThreadHandle = ThreadHandle;
            (*CreateProcessMsg).ClientId = ClientId;

            /* Write the remote PEB address and clear it locally */
            (*CreateProcessMsg).PebAddressNative = RemotePeb as PVOID;
            #[cfg(target_pointer_width = "64")]
            {
                /* WoW64 (Phase C): an i386 image on a 64-bit host is a WoW64
                 * process. Report its PEB32 (EPROCESS.Wow64Process, via
                 * ProcessWow64Information) so csrss/basesrv take the wow64 path
                 * with a VALID PEB32. Leaving this 0 while ProcessorArchitecture
                 * is set to INTEL (below) makes basesrv walk a wow64 path with a
                 * null PEB32 and fault (raise 0x6BA → csrss dies → BUGCHECK 0xF4;
                 * field manual §9.46). */
                let mut Peb32: ULONG_PTR = 0;
                if ImageInformation.Machine as u32 == IMAGE_FILE_MACHINE_I386 {
                    extern "system" {
                        fn NtQueryInformationProcess(
                            ProcessHandle: HANDLE,
                            ProcessInformationClass: u32,
                            ProcessInformation: PVOID,
                            ProcessInformationLength: ULONG,
                            ReturnLength: PULONG,
                        ) -> NTSTATUS;
                    }
                    const PROCESS_WOW64_INFORMATION: u32 = 26;
                    NtQueryInformationProcess(
                        ProcessHandle,
                        PROCESS_WOW64_INFORMATION,
                        core::ptr::addr_of_mut!(Peb32) as PVOID,
                        core::mem::size_of::<ULONG_PTR>() as ULONG,
                        core::ptr::null_mut(),
                    );
                }
                (*CreateProcessMsg).PebAddressWow64 = Peb32 as ULONG;
            }
            #[cfg(target_pointer_width = "32")]
            {
                (*CreateProcessMsg).PebAddressWow64 = RemotePeb as ULONG;
            }
            RemotePeb = core::ptr::null_mut();

            /* Now check what kind of architecture this image was made for */
            match ImageInformation.Machine as u32 {
                IMAGE_FILE_MACHINE_I386 => {
                    (*CreateProcessMsg).ProcessorArchitecture =
                        PROCESSOR_ARCHITECTURE_INTEL as USHORT;
                }
                IMAGE_FILE_MACHINE_IA64 => {
                    (*CreateProcessMsg).ProcessorArchitecture =
                        PROCESSOR_ARCHITECTURE_IA64 as USHORT;
                }
                IMAGE_FILE_MACHINE_AMD64 => {
                    (*CreateProcessMsg).ProcessorArchitecture =
                        PROCESSOR_ARCHITECTURE_AMD64 as USHORT;
                }
                IMAGE_FILE_MACHINE_ARM64 => {
                    (*CreateProcessMsg).ProcessorArchitecture =
                        PROCESSOR_ARCHITECTURE_ARM64 as USHORT;
                }
                _ => {
                    /* DbgPrint "No mapping for ImageInformation.Machine" — dropped */
                    (*CreateProcessMsg).ProcessorArchitecture =
                        PROCESSOR_ARCHITECTURE_UNKNOWN as USHORT;
                }
            }

            /* Write the input creation flags except any debugger-related flags */
            (*CreateProcessMsg).CreationFlags =
                dwCreationFlags & !(DEBUG_PROCESS | DEBUG_ONLY_THIS_PROCESS);

            /* CSRSS needs to know if this is a GUI app or not */
            if (ImageInformation.SubSystemType == IMAGE_SUBSYSTEM_WINDOWS_GUI) || (IsWowApp != 0) {
                /* For GUI apps we turn on the 2nd bit. */
                (*CreateProcessMsg).ProcessHandle =
                    AddToHandle((*CreateProcessMsg).ProcessHandle, 2);

                /* Also check if the parent is also a GUI process */
                NtHeaders = RtlImageNtHeader(GetModuleHandleW(core::ptr::null()) as PVOID);
                if (!NtHeaders.is_null())
                    && ((*NtHeaders).OptionalHeader.Subsystem == IMAGE_SUBSYSTEM_WINDOWS_GUI as WORD)
                {
                    /* Let it know that it should display the hourglass mouse cursor */
                    (*CreateProcessMsg).ProcessHandle =
                        AddToHandle((*CreateProcessMsg).ProcessHandle, 1);
                }
            }

            /* hourglass cursor on/off feedback */
            if StartupInfo.dwFlags & STARTF_FORCEONFEEDBACK != 0 {
                (*CreateProcessMsg).ProcessHandle =
                    AddToHandle((*CreateProcessMsg).ProcessHandle, 1);
            }
            if StartupInfo.dwFlags & STARTF_FORCEOFFFEEDBACK != 0 {
                (*CreateProcessMsg).ProcessHandle =
                    RemoveFromHandle((*CreateProcessMsg).ProcessHandle, 1);
            }

            /* Also store which kind of VDM app (if any) this is */
            (*CreateProcessMsg).VdmBinaryType = VdmBinaryType;

            /* And if it really is a VDM app... */
            if VdmBinaryType != 0 {
                /* Store the VDM console handle and the task ID */
                (*CreateProcessMsg).hVDM = if VdmTask != 0 {
                    core::ptr::null_mut()
                } else {
                    (*(*Peb).ProcessParameters).ConsoleHandle
                };
                (*CreateProcessMsg).VdmTask = VdmTask;
            } else if VdmReserve != 0 {
                /* Extended VDM, set a flag */
                (*CreateProcessMsg).VdmBinaryType |= BINARY_TYPE_WOW_EX;
            }

            /* Check if there's side-by-side assembly data associated with the process */
            if (*CreateProcessMsg).Sxs.Flags != 0 {
                /* This should not happen in ReactOS yet */
                BaseSetLastNTError(STATUS_NOT_IMPLEMENTED);
                NtTerminateProcess(ProcessHandle, STATUS_NOT_IMPLEMENTED);
                Result = FALSE;
                break 'quickie;
            }

            /* We are finally ready to call CSRSS to tell it about our new process! */
            #[cfg(target_arch = "aarch64")]
            {
                /* ARM64: Skip CSRSS notification if not connected (native process) */
                if BaseCsrConnectionEstablished != 0 {
                    CsrClientCallServer(
                        &mut CsrMsg[0] as *mut BASE_API_MESSAGE as PCSR_API_MESSAGE,
                        CaptureBuffer,
                        ((BASESRV_SERVERDLL_INDEX << 16) | (BasepCreateProcess as ULONG))
                            as CSR_API_NUMBER,
                        core::mem::size_of::<BASE_CREATE_PROCESS>() as ULONG,
                    );

                    /* CSRSS has returned, free the capture buffer now if we had one */
                    if !CaptureBuffer.is_null() {
                        CsrFreeCaptureBuffer(CaptureBuffer);
                        CaptureBuffer = core::ptr::null_mut();
                    }

                    /* Check if CSRSS failed to accept ownership of the new process */
                    if !NT_SUCCESS(CsrMsg[0].Status) {
                        BaseSetLastNTError(CsrMsg[0].Status);
                        NtTerminateProcess(ProcessHandle, CsrMsg[0].Status);
                        Result = FALSE;
                        break 'quickie;
                    }
                } else {
                    /* No CSRSS connection — free capture buffer and continue */
                    if !CaptureBuffer.is_null() {
                        CsrFreeCaptureBuffer(CaptureBuffer);
                        CaptureBuffer = core::ptr::null_mut();
                    }
                }
            }
            #[cfg(not(target_arch = "aarch64"))]
            {
                CsrClientCallServer(
                    &mut CsrMsg[0] as *mut BASE_API_MESSAGE as PCSR_API_MESSAGE,
                    CaptureBuffer,
                    ((BASESRV_SERVERDLL_INDEX << 16) | (BasepCreateProcess as ULONG))
                        as CSR_API_NUMBER,
                    core::mem::size_of::<BASE_CREATE_PROCESS>() as ULONG,
                );

                /* CSRSS has returned, free the capture buffer now if we had one */
                if !CaptureBuffer.is_null() {
                    CsrFreeCaptureBuffer(CaptureBuffer);
                    CaptureBuffer = core::ptr::null_mut();
                }

                /* Check if CSRSS failed to accept ownership of the new process */
                if !NT_SUCCESS(CsrMsg[0].Status) {
                    BaseSetLastNTError(CsrMsg[0].Status);
                    NtTerminateProcess(ProcessHandle, CsrMsg[0].Status);
                    Result = FALSE;
                    break 'quickie;
                }
            }

            /* Check if we have a token due to Authz/Safer, not passed by the user */
            if (!TokenHandle.is_null()) && (hUserToken.is_null()) {
                /* Replace the process and/or thread token with the one from Safer */
                Status =
                    BasepReplaceProcessThreadTokens(TokenHandle, ProcessHandle, ThreadHandle);
                if !NT_SUCCESS(Status) {
                    /* If this failed, kill the process and enter the failure path */
                    NtTerminateProcess(ProcessHandle, Status);
                    BaseSetLastNTError(Status);
                    Result = FALSE;
                    break 'quickie;
                }
            }

            /* Check if a job was associated with this process */
            if !JobHandle.is_null() {
                /* Bind the process and job together now */
                Status = NtAssignProcessToJobObject(JobHandle, ProcessHandle);
                if !NT_SUCCESS(Status) {
                    /* Kill the process and enter the failure path if binding failed */
                    NtTerminateProcess(ProcessHandle, STATUS_ACCESS_DENIED);
                    BaseSetLastNTError(Status);
                    Result = FALSE;
                    break 'quickie;
                }
            }

            /* Finally, resume the thread to actually get the process started */
            if dwCreationFlags & CREATE_SUSPENDED == 0 {
                NtResumeThread(ThreadHandle, &mut ResumeCount);
            }

            /* Fall through to the success tail (VdmShortCircuit). */
            break 'app_name_retry;
        } // 'app_name_retry

        /* ----- VdmShortCircuit: success tail ----- */

        /* We made it this far, meaning we have a fully created process and thread */
        Result = TRUE;

        /* Anyone doing a VDM undo should now undo everything, since we are done */
        if VdmUndoLevel != 0 {
            VdmUndoLevel |= VDM_UNDO_COMPLETED;
        }

        /* Having a VDM wait object implies this must be a VDM process */
        if !VdmWaitObject.is_null() {
            /* Check if it's a 16-bit separate WOW process */
            if VdmBinaryType == BINARY_TYPE_SEPARATE_WOW {
                /* OR-in the special flag to indicate this, and return to caller */
                VdmWaitObject = AddToHandle(VdmWaitObject, 2);
                (*lpProcessInformation).hProcess = VdmWaitObject;

                /* Check if this was a re-used VDM */
                if VdmUndoLevel & VDM_UNDO_REUSE != 0 {
                    /* No Client ID should be returned in this case */
                    ClientId.UniqueProcess = core::ptr::null_mut();
                    ClientId.UniqueThread = core::ptr::null_mut();
                }
            } else {
                /* OR-in the special flag to indicate this is not a separate VDM */
                VdmWaitObject = AddToHandle(VdmWaitObject, 1);
                (*lpProcessInformation).hProcess = VdmWaitObject;
            }

            /* Close the original process handle, since it's not needed for VDM */
            if !ProcessHandle.is_null() {
                NtClose(ProcessHandle);
            }
        } else {
            /* This is a regular process, so return the real process handle */
            (*lpProcessInformation).hProcess = ProcessHandle;
        }

        /* Return the rest of the process information based on what we have so far */
        (*lpProcessInformation).hThread = ThreadHandle;
        (*lpProcessInformation).dwProcessId = HandleToUlong(ClientId.UniqueProcess);
        (*lpProcessInformation).dwThreadId = HandleToUlong(ClientId.UniqueThread);

        /* NULL these out here so we know to treat this as a success scenario */
        ProcessHandle = core::ptr::null_mut();
        ThreadHandle = core::ptr::null_mut();
    } // 'quickie

    /* ----- Quickie: cleanup tail ----- */

    /* Free the debugger command line if one was allocated */
    if !DebuggerCmdLine.is_null() {
        RtlFreeHeap(RtlGetProcessHeap(), 0, DebuggerCmdLine as PVOID);
    }

    /* Check if an SxS full path was queried */
    if !PathBuffer.is_null() {
        /* Reinitialize the executable path */
        RtlInitEmptyUnicodeString(&mut SxsWin32ExePath, core::ptr::null_mut(), 0);
        SxsWin32ExePath.Length = 0;

        /* Free the path buffer */
        RtlFreeHeap(RtlGetProcessHeap(), 0, PathBuffer as PVOID);
    }

    /* Check if an environment was passed in */
    if (!lpEnvironment.is_null()) && (dwCreationFlags & CREATE_UNICODE_ENVIRONMENT == 0) {
        /* Destroy it */
        RtlDestroyEnvironment(lpEnvironment as *mut u16);

        /* If this was the VDM environment too, clear that as well */
        if VdmUnicodeEnv.Buffer as LPVOID == lpEnvironment {
            VdmUnicodeEnv.Buffer = core::ptr::null_mut();
        }
        lpEnvironment = core::ptr::null_mut();
    }

    /* Unconditionally free all the name parsing buffers we always allocate */
    RtlFreeHeap(RtlGetProcessHeap(), 0, QuotedCmdLine as PVOID);
    RtlFreeHeap(RtlGetProcessHeap(), 0, NameBuffer as PVOID);
    RtlFreeHeap(RtlGetProcessHeap(), 0, CurrentDirectory as PVOID);
    RtlFreeHeap(RtlGetProcessHeap(), 0, FreeBuffer as PVOID);

    /* Close open file/section handles */
    if !FileHandle.is_null() {
        NtClose(FileHandle);
    }
    if !SectionHandle.is_null() {
        NtClose(SectionHandle);
    }

    /* If we have a thread handle, this was a failure path */
    if !ThreadHandle.is_null() {
        /* So kill the process and close the thread handle */
        NtTerminateProcess(ProcessHandle, STATUS_SUCCESS);
        NtClose(ThreadHandle);
    }

    /* If we have a process handle, this was a failure path, so close it */
    if !ProcessHandle.is_null() {
        NtClose(ProcessHandle);
    }

    /* Thread/process handles, if any, are now processed. Now close this one. */
    if !JobHandle.is_null() {
        NtClose(JobHandle);
    }

    /* Check if we had created a token */
    if !TokenHandle.is_null() {
        /* And if the user asked for one */
        if !hUserToken.is_null() {
            /* Then return it */
            *hNewToken = TokenHandle;
        } else {
            /* User didn't want it, so we used it temporarily -- close it */
            NtClose(TokenHandle);
        }
    }

    /* Free any temporary app compatibility data, it's no longer needed */
    BasepFreeAppCompatData(AppCompatData, AppCompatSxsData);

    /* Free a few strings. The API takes care of these possibly being NULL */
    RtlFreeUnicodeString(&mut VdmString);
    RtlFreeUnicodeString(&mut DebuggerString);

    /* Check if we had built any sort of VDM environment */
    if (!VdmAnsiEnv.Buffer.is_null()) || (!VdmUnicodeEnv.Buffer.is_null()) {
        /* Free it */
        BaseDestroyVDMEnvironment(&mut VdmAnsiEnv, &mut VdmUnicodeEnv);
    }

    /* Check if this was any kind of VDM application that we ended up creating */
    if (VdmUndoLevel != 0) && (VdmUndoLevel & VDM_UNDO_COMPLETED == 0) {
        /* Send an undo */
        BaseUpdateVDMEntry(
            VdmEntryUndo,
            &mut VdmTask as *mut ULONG as PHANDLE,
            VdmUndoLevel,
            VdmBinaryType,
        );

        /* And close whatever VDM handle we were using for notifications */
        if !VdmWaitObject.is_null() {
            NtClose(VdmWaitObject);
        }
    }

    /* Check if we ended up here with an allocated search path, and free it */
    if !SearchPath.is_null() {
        RtlFreeHeap(RtlGetProcessHeap(), 0, SearchPath as PVOID);
    }

    /* Finally, return the API's result */
    Result
}

/* ---------------------------------------------------------------------------
 * CreateProcessW — thin wrapper.
 * ------------------------------------------------------------------------- */

#[no_mangle]
pub unsafe extern "system" fn CreateProcessW(
    lpApplicationName: LPCWSTR,
    lpCommandLine: LPWSTR,
    lpProcessAttributes: LPSECURITY_ATTRIBUTES,
    lpThreadAttributes: LPSECURITY_ATTRIBUTES,
    bInheritHandles: BOOL,
    dwCreationFlags: DWORD,
    lpEnvironment: LPVOID,
    lpCurrentDirectory: LPCWSTR,
    lpStartupInfo: LPSTARTUPINFOW,
    lpProcessInformation: LPPROCESS_INFORMATION,
) -> BOOL {
    /* Call the internal (but exported) version */
    CreateProcessInternalW(
        core::ptr::null_mut(),
        lpApplicationName,
        lpCommandLine,
        lpProcessAttributes,
        lpThreadAttributes,
        bInheritHandles,
        dwCreationFlags,
        lpEnvironment,
        lpCurrentDirectory,
        lpStartupInfo,
        lpProcessInformation,
        core::ptr::null_mut(),
    )
}

/* ---------------------------------------------------------------------------
 * CreateProcessInternalA — ANSI wrapper.
 * ------------------------------------------------------------------------- */

#[no_mangle]
pub unsafe extern "system" fn CreateProcessInternalA(
    hToken: HANDLE,
    lpApplicationName: LPCSTR,
    lpCommandLine: LPSTR,
    lpProcessAttributes: LPSECURITY_ATTRIBUTES,
    lpThreadAttributes: LPSECURITY_ATTRIBUTES,
    bInheritHandles: BOOL,
    dwCreationFlags: DWORD,
    lpEnvironment: LPVOID,
    lpCurrentDirectory: LPCSTR,
    lpStartupInfo: LPSTARTUPINFOA,
    lpProcessInformation: LPPROCESS_INFORMATION,
    hNewToken: PHANDLE,
) -> BOOL {
    let mut CommandLine: UNICODE_STRING = core::mem::zeroed();
    let mut ApplicationName: UNICODE_STRING = core::mem::zeroed();
    let mut CurrentDirectory: UNICODE_STRING = core::mem::zeroed();
    let bRetVal: BOOL;
    let mut StartupInfo: STARTUPINFOW = core::mem::zeroed();

    /* Copy Startup Info */
    core::ptr::copy_nonoverlapping(
        lpStartupInfo as *const u8,
        &mut StartupInfo as *mut STARTUPINFOW as *mut u8,
        core::mem::size_of::<STARTUPINFOA>(),
    );

    /* Initialize all strings to nothing */
    CommandLine.Buffer = core::ptr::null_mut();
    ApplicationName.Buffer = core::ptr::null_mut();
    CurrentDirectory.Buffer = core::ptr::null_mut();
    StartupInfo.lpDesktop = core::ptr::null_mut();
    StartupInfo.lpReserved = core::ptr::null_mut();
    StartupInfo.lpTitle = core::ptr::null_mut();

    /* Convert the Command line */
    if !lpCommandLine.is_null() {
        Basep8BitStringToDynamicUnicodeString(&mut CommandLine, lpCommandLine);
    }

    /* Convert the Name and Directory */
    if !lpApplicationName.is_null() {
        Basep8BitStringToDynamicUnicodeString(&mut ApplicationName, lpApplicationName);
    }
    if !lpCurrentDirectory.is_null() {
        Basep8BitStringToDynamicUnicodeString(&mut CurrentDirectory, lpCurrentDirectory);
    }

    /* Now convert Startup Strings */
    if !(*lpStartupInfo).lpReserved.is_null() {
        BasepAnsiStringToHeapUnicodeString(
            (*lpStartupInfo).lpReserved,
            &mut StartupInfo.lpReserved,
        );
    }
    if !(*lpStartupInfo).lpDesktop.is_null() {
        BasepAnsiStringToHeapUnicodeString((*lpStartupInfo).lpDesktop, &mut StartupInfo.lpDesktop);
    }
    if !(*lpStartupInfo).lpTitle.is_null() {
        BasepAnsiStringToHeapUnicodeString((*lpStartupInfo).lpTitle, &mut StartupInfo.lpTitle);
    }

    /* Call the Unicode function */
    bRetVal = CreateProcessInternalW(
        hToken,
        ApplicationName.Buffer,
        CommandLine.Buffer,
        lpProcessAttributes,
        lpThreadAttributes,
        bInheritHandles,
        dwCreationFlags,
        lpEnvironment,
        CurrentDirectory.Buffer,
        &mut StartupInfo,
        lpProcessInformation,
        hNewToken,
    );

    /* Clean up */
    RtlFreeUnicodeString(&mut ApplicationName);
    RtlFreeUnicodeString(&mut CommandLine);
    RtlFreeUnicodeString(&mut CurrentDirectory);
    RtlFreeHeap(RtlGetProcessHeap(), 0, StartupInfo.lpDesktop as PVOID);
    RtlFreeHeap(RtlGetProcessHeap(), 0, StartupInfo.lpReserved as PVOID);
    RtlFreeHeap(RtlGetProcessHeap(), 0, StartupInfo.lpTitle as PVOID);

    /* Return what Unicode did */
    bRetVal
}

/* ---------------------------------------------------------------------------
 * CreateProcessA — thin wrapper.
 * ------------------------------------------------------------------------- */

#[no_mangle]
pub unsafe extern "system" fn CreateProcessA(
    lpApplicationName: LPCSTR,
    lpCommandLine: LPSTR,
    lpProcessAttributes: LPSECURITY_ATTRIBUTES,
    lpThreadAttributes: LPSECURITY_ATTRIBUTES,
    bInheritHandles: BOOL,
    dwCreationFlags: DWORD,
    lpEnvironment: LPVOID,
    lpCurrentDirectory: LPCSTR,
    lpStartupInfo: LPSTARTUPINFOA,
    lpProcessInformation: LPPROCESS_INFORMATION,
) -> BOOL {
    /* Call the internal (but exported) version */
    CreateProcessInternalA(
        core::ptr::null_mut(),
        lpApplicationName,
        lpCommandLine,
        lpProcessAttributes,
        lpThreadAttributes,
        bInheritHandles,
        dwCreationFlags,
        lpEnvironment,
        lpCurrentDirectory,
        lpStartupInfo,
        lpProcessInformation,
        core::ptr::null_mut(),
    )
}

/* ---------------------------------------------------------------------------
 * WinExec
 * ------------------------------------------------------------------------- */

#[no_mangle]
pub unsafe extern "system" fn WinExec(lpCmdLine: LPCSTR, uCmdShow: UINT) -> UINT {
    let mut StartupInfo: STARTUPINFOA = core::mem::zeroed();
    let mut ProcessInformation: PROCESS_INFORMATION = core::mem::zeroed();
    let dosErr: DWORD;

    core::ptr::write_bytes(
        &mut StartupInfo as *mut STARTUPINFOA as *mut u8,
        0,
        core::mem::size_of::<STARTUPINFOA>(),
    );
    StartupInfo.cb = core::mem::size_of::<STARTUPINFOA>() as DWORD;
    StartupInfo.wShowWindow = uCmdShow as WORD;
    StartupInfo.dwFlags = 0;

    if CreateProcessA(
        core::ptr::null(),
        lpCmdLine as LPSTR,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        FALSE,
        0,
        core::ptr::null_mut(),
        core::ptr::null(),
        &mut StartupInfo,
        &mut ProcessInformation,
    ) == 0
    {
        dosErr = GetLastError();
        return if dosErr < 32 { dosErr } else { ERROR_BAD_FORMAT };
    }

    if let Some(WaitForInputIdle) = UserWaitForInputIdleRoutine {
        WaitForInputIdle(ProcessInformation.hProcess, 10000);
    }

    NtClose(ProcessInformation.hProcess);
    NtClose(ProcessInformation.hThread);

    33 /* Something bigger than 31 means success. */
}
