use ntdll_sys::nt::*;
use crate::prelude::*;
use crate::console::cffi::*;
use core::ffi::c_void;
const FALSE: BOOL = 0;
const TRUE: BOOL = 1;

// kernel32 exports referenced by the console init path (resolve to sibling
// #[no_mangle] definitions at link).
extern "system" {
    fn GetSystemDirectoryW(lpBuffer: LPWSTR, uSize: UINT) -> UINT;
    fn LoadLibraryW(lpLibFileName: LPCWSTR) -> HMODULE;
    fn FreeLibrary(hLibModule: HMODULE) -> BOOL;
    fn GetProcAddress(hModule: HMODULE, lpProcName: LPCSTR) -> FARPROC;
    fn GetOEMCP() -> UINT;
}

// winnt.h HandleToUlong: truncate a handle to a ULONG.
#[inline]
unsafe fn HandleToUlong(h: HANDLE) -> ULONG {
    (h as usize) as ULONG
}

// ---- file-scope preamble (translated) ----
pub static mut ConsoleLock: RTL_CRITICAL_SECTION = RTL_CRITICAL_SECTION {
    DebugInfo: core::ptr::null_mut(),
    LockCount: -1,
    RecursionCount: 0,
    OwningThread: core::ptr::null_mut(),
    LockSemaphore: core::ptr::null_mut(),
    SpinCount: 0,
};
pub static mut ConsoleInitialized: BOOLEAN = 0;
pub static mut DefaultConsoleTitle: PWSTR = {
    const S: &[u16] = &[
        'R' as u16, 'e' as u16, 'a' as u16, 'c' as u16, 't' as u16,
        'O' as u16, 'S' as u16, ' ' as u16, 'C' as u16, 'o' as u16,
        'n' as u16, 's' as u16, 'o' as u16, 'l' as u16, 'e' as u16,
        0u16,
    ];
    S.as_ptr() as PWSTR
};

#[no_mangle]
pub unsafe extern "system" fn PropDialogHandler(lpThreadParameter: LPVOID) -> DWORD {
    let mut Status: NTSTATUS = STATUS_SUCCESS;
    let mut hConsoleApplet: HMODULE = core::ptr::null_mut();
    let mut CPlApplet: APPLET_PROC;
    static mut AlreadyDisplayingProps: BOOL = 0;
    let mut szBuffer: [u16; MAX_PATH as usize] = [0u16; MAX_PATH as usize];

    if ConsoleInitialized == 0 || AlreadyDisplayingProps != 0 {
        if !lpThreadParameter.is_null() {
            CloseHandle(lpThreadParameter as HANDLE);
        }
        return STATUS_UNSUCCESSFUL as DWORD;
    }

    AlreadyDisplayingProps = 1;

    'quit: {
        GetSystemDirectoryW(szBuffer.as_mut_ptr(), MAX_PATH);
        wcscat(szBuffer.as_mut_ptr(), w!("\\console.dll").as_ptr() as *const u16);
        hConsoleApplet = LoadLibraryW(szBuffer.as_mut_ptr());
        if hConsoleApplet.is_null() {
            Status = STATUS_UNSUCCESSFUL;
            break 'quit;
        }

        CPlApplet = core::mem::transmute::<FARPROC, APPLET_PROC>(
            GetProcAddress(hConsoleApplet, "CPlApplet\0".as_ptr() as *const i8)
        );
        if CPlApplet.is_none() {
            Status = STATUS_UNSUCCESSFUL;
            break 'quit;
        }

        let CPlApplet = CPlApplet.unwrap();

        if CPlApplet(core::ptr::null_mut(), CPL_INIT, 0, 0) == 0 {
            Status = STATUS_UNSUCCESSFUL;
            break 'quit;
        }

        if CPlApplet(core::ptr::null_mut(), CPL_GETCOUNT, 0, 0) != 1 {
            Status = STATUS_UNSUCCESSFUL;
            break 'quit;
        }

        CPlApplet(lpThreadParameter as HWND, CPL_DBLCLK, 0, 0);
        CPlApplet(core::ptr::null_mut(), CPL_EXIT, 0, 0);
    }

    if !hConsoleApplet.is_null() {
        FreeLibrary(hConsoleApplet);
    }
    AlreadyDisplayingProps = 0;
    return Status as DWORD;
}
unsafe fn ParseShellInfo(
    lpszShellInfo: LPCWSTR,
    lpszKeyword: LPCWSTR,
) -> INT {
    // DPRINT removed
    return 0;
}
#[no_mangle]
pub unsafe extern "C" fn SetUpConsoleInfo(
    CaptureTitle: BOOLEAN,
    pTitleLength: LPDWORD,
    lpTitle: *mut LPWSTR,
    pDesktopLength: LPDWORD,
    lpDesktop: *mut LPWSTR,
    ConsoleStartInfo: *mut CONSOLE_START_INFO,
) {
    let Parameters: PRTL_USER_PROCESS_PARAMETERS = (*NtCurrentPeb()).ProcessParameters;
    let mut Length: DWORD;

    /* Initialize the fields */
    (*ConsoleStartInfo).IconIndex = 0;
    (*ConsoleStartInfo).hIcon = core::ptr::null_mut();
    (*ConsoleStartInfo).hIconSm = core::ptr::null_mut();
    (*ConsoleStartInfo).dwStartupFlags = (*Parameters).WindowFlags;
    (*ConsoleStartInfo).nFont = 0;
    (*ConsoleStartInfo).nInputBufferSize = 0;
    (*ConsoleStartInfo).uCodePage = GetOEMCP();

    if !lpTitle.is_null() {
        let Title: LPWSTR;

        /* If we don't have any title, use the default one */
        if (*Parameters).WindowTitle.Buffer.is_null() {
            Title = DefaultConsoleTitle as LPWSTR;
            Length = (lstrlenW(DefaultConsoleTitle) as DWORD).wrapping_mul(core::mem::size_of::<WCHAR>() as DWORD);
        } else {
            Title = (*Parameters).WindowTitle.Buffer;
            Length = (*Parameters).WindowTitle.Length as DWORD;
        }

        /* Retrieve the needed buffer size */
        Length = Length.wrapping_add(core::mem::size_of::<WCHAR>() as DWORD);
        if *pTitleLength > 0 {
            Length = min(Length, *pTitleLength);
        }
        *pTitleLength = Length;

        /* Capture the data if needed, or, return a pointer to it */
        if CaptureTitle != 0 {
            /*
             * Length is always >= sizeof(WCHAR). Copy everything but the
             * possible trailing NULL character, and then NULL-terminate.
             */
            Length = Length.wrapping_sub(core::mem::size_of::<WCHAR>() as DWORD);
            RtlCopyMemory(*lpTitle as *mut core::ffi::c_void, Title as *const core::ffi::c_void, Length as SIZE_T);
            *((*lpTitle).offset((Length / core::mem::size_of::<WCHAR>() as DWORD) as isize)) = UNICODE_NULL;
        } else {
            *lpTitle = Title;
        }
    } else {
        *pTitleLength = 0;
    }

    if !lpDesktop.is_null() && !(*Parameters).DesktopInfo.Buffer.is_null() && *(*Parameters).DesktopInfo.Buffer != 0 {
        /* Retrieve the needed buffer size */
        Length = (*Parameters).DesktopInfo.Length as DWORD + (core::mem::size_of::<WCHAR>() as DWORD);
        if *pDesktopLength > 0 {
            Length = min(Length, *pDesktopLength);
        }
        *pDesktopLength = Length;

        /* Return a pointer to the data */
        *lpDesktop = (*Parameters).DesktopInfo.Buffer;
    } else {
        *pDesktopLength = 0;
        if !lpDesktop.is_null() {
            *lpDesktop = core::ptr::null_mut();
        }
    }

    if (*Parameters).WindowFlags & STARTF_USEFILLATTRIBUTE != 0 {
        (*ConsoleStartInfo).wFillAttribute = (*Parameters).FillAttribute as WORD;
    }
    if (*Parameters).WindowFlags & STARTF_USECOUNTCHARS != 0 {
        (*ConsoleStartInfo).dwScreenBufferSize.X = (*Parameters).CountCharsX as SHORT;
        (*ConsoleStartInfo).dwScreenBufferSize.Y = (*Parameters).CountCharsY as SHORT;
    }
    if (*Parameters).WindowFlags & STARTF_USESHOWWINDOW != 0 {
        (*ConsoleStartInfo).wShowWindow = (*Parameters).ShowWindowFlags as WORD;
    }
    if (*Parameters).WindowFlags & STARTF_USEPOSITION != 0 {
        (*ConsoleStartInfo).dwWindowOrigin.X = (*Parameters).StartingX as SHORT;
        (*ConsoleStartInfo).dwWindowOrigin.Y = (*Parameters).StartingY as SHORT;
    }
    if (*Parameters).WindowFlags & STARTF_USESIZE != 0 {
        (*ConsoleStartInfo).dwWindowSize.X = (*Parameters).CountX as SHORT;
        (*ConsoleStartInfo).dwWindowSize.Y = (*Parameters).CountY as SHORT;
    }

    /* Get shell information (ShellInfo.Buffer is NULL-terminated) */
    if !(*Parameters).ShellInfo.Buffer.is_null() {
        (*ConsoleStartInfo).IconIndex = ParseShellInfo((*Parameters).ShellInfo.Buffer, w!("dde.").as_ptr() as LPCWSTR);

        if (*Parameters).WindowFlags & STARTF_USEHOTKEY == 0 {
            (*ConsoleStartInfo).dwHotKey = ParseShellInfo((*Parameters).ShellInfo.Buffer, w!("hotkey.").as_ptr() as LPCWSTR) as DWORD;
        } else {
            (*ConsoleStartInfo).dwHotKey = HandleToUlong((*Parameters).StandardInput);
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn SetUpHandles(ConsoleStartInfo: *const CONSOLE_START_INFO) {
    let Parameters: *mut RTL_USER_PROCESS_PARAMETERS = (*NtCurrentPeb()).ProcessParameters;

    if (*ConsoleStartInfo).dwStartupFlags & STARTF_USEHOTKEY != 0 {
        (*Parameters).WindowFlags &= !STARTF_USEHOTKEY;
    }
    if (*ConsoleStartInfo).dwStartupFlags & STARTF_SHELLPRIVATE != 0 {
        (*Parameters).WindowFlags &= !STARTF_SHELLPRIVATE;
    }

    /* We got the handles, let's set them */
    (*Parameters).ConsoleHandle = (*ConsoleStartInfo).ConsoleHandle;

    if (*ConsoleStartInfo).dwStartupFlags & STARTF_USESTDHANDLES == 0 {
        (*Parameters).StandardInput = (*ConsoleStartInfo).InputHandle;
        (*Parameters).StandardOutput = (*ConsoleStartInfo).OutputHandle;
        (*Parameters).StandardError = (*ConsoleStartInfo).ErrorHandle;
    }
}

unsafe fn IsConsoleApp() -> BOOLEAN {
    let ImageNtHeader: PIMAGE_NT_HEADERS = RtlImageNtHeader((*NtCurrentPeb()).ImageBaseAddress);
    return if !ImageNtHeader.is_null() && ((*ImageNtHeader).OptionalHeader.Subsystem == IMAGE_SUBSYSTEM_WINDOWS_CUI as u16) { 1 } else { 0 };
}
unsafe fn ConnectConsole(
    SessionDir: PWSTR,
    ConnectInfo: PCONSRV_API_CONNECTINFO,
    InServerProcess: PBOOLEAN,
) -> BOOLEAN {
    // C: `ULONG ConnectInfoSize = sizeof(*ConnectInfo);` then `&ConnectInfoSize`.
    // CsrClientConnectToServer takes this as an IN/OUT PULONG and WRITES the actual
    // size back. It MUST be a mutable local passed as a real `&mut` — the previous
    // `&ConnectInfoSize as *const ULONG as *mut ULONG` wrote through a *const-derived
    // pointer to an immutable `let`, which is UB. arm64 gnullvm LLVM exploited that UB
    // and miscompiled the post-connect reads (stale ConsoleStartInfo.InitEvents → wait
    // on null handles → deadlock at NtWaitForMultipleObjects); a debug print perturbed
    // codegen and masked it (the Heisenbug). i386/amd64 codegen happened not to bite.
    let mut ConnectInfoSize: ULONG = core::mem::size_of::<CONSRV_API_CONNECTINFO>() as ULONG;

    // ASSERT(SessionDir);

    /* Connect to the Console Server */
    // DPRINT("Connecting to the Console Server...\n");
    let Status: NTSTATUS = CsrClientConnectToServer(
        SessionDir,
        CONSRV_SERVERDLL_INDEX,
        ConnectInfo as *mut _,
        &mut ConnectInfoSize as *mut ULONG,
        InServerProcess,
    );
    if !NT_SUCCESS(Status) {
        // DPRINT1("Failed to connect to the Console Server (Status %lx)\n", Status);
        return 0;
    }

    /* Nothing to do for server-to-server */
    if *InServerProcess != 0 {
        return 1;
    }

    /* Nothing to do if this is not a console app */
    if (*ConnectInfo).IsConsoleApp == 0 {
        return 1;
    }

    /* Wait for the connection to finish */
    // Is ConnectInfo->ConsoleStartInfo.InitEvents aligned on handle boundary ????
    let Status: NTSTATUS = NtWaitForMultipleObjects(
        MAX_INIT_EVENTS,
        (*ConnectInfo).ConsoleStartInfo.InitEvents.as_ptr() as *mut _,
        WaitAny,
        0,
        core::ptr::null_mut(),
    );
    if !NT_SUCCESS(Status) {
        BaseSetLastNTError(Status);
        return 0;
    }

    NtClose((*ConnectInfo).ConsoleStartInfo.InitEvents[INIT_SUCCESS as usize]);
    NtClose((*ConnectInfo).ConsoleStartInfo.InitEvents[INIT_FAILURE as usize]);
    if Status != INIT_SUCCESS as NTSTATUS {
        (*(*NtCurrentPeb()).ProcessParameters).ConsoleHandle = core::ptr::null_mut();
        return 0;
    }

    return 1;
}

#[no_mangle]
pub unsafe extern "system" fn ConDllInitialize(
    Reason: ULONG,
    SessionDir: PWSTR,
) -> BOOLEAN {
    let mut Status: NTSTATUS;
    let Parameters: PRTL_USER_PROCESS_PARAMETERS = (*NtCurrentPeb()).ProcessParameters;
    let mut InServerProcess: BOOLEAN = 0;
    let mut ConnectInfo: CONSRV_API_CONNECTINFO = core::mem::zeroed();

    if Reason != DLL_PROCESS_ATTACH {
        if (Reason == DLL_THREAD_ATTACH) && (IsConsoleApp() != 0) {
            /* Sync the new thread's LangId with the console's one */
            SetTEBLangID();
        } else if Reason == DLL_PROCESS_DETACH {
            /* Free our resources */
            if ConsoleInitialized != 0 {
                ConsoleInitialized = 0;
                RtlDeleteCriticalSection(&raw mut ConsoleLock);
            }
        }

        return 1;
    }

    /* Initialize our global console DLL lock */
    Status = RtlInitializeCriticalSection(&raw mut ConsoleLock);
    if !NT_SUCCESS(Status) {
        return 0;
    }
    ConsoleInitialized = 1;

    /* Show by default the console window when applicable */
    ConnectInfo.IsWindowVisible = 1;
    /* If this is a console app, a console will be created/opened */
    ConnectInfo.IsConsoleApp = IsConsoleApp();

    /* Do nothing if this is not a console app... */
    if ConnectInfo.IsConsoleApp == 0 {
    }

    /*
     * Handle the special flags given to us by BasePushProcessParameters.
     */
    if (*Parameters).ConsoleHandle == HANDLE_DETACHED_PROCESS {
        /*
         * The new process does not inherit its parent's console and cannot
         * attach to the console of its parent. The new process can call the
         * AllocConsole function at a later time to create a console.
         */
        (*Parameters).ConsoleHandle = core::ptr::null_mut(); // Do not inherit the parent's console.
        ConnectInfo.IsConsoleApp = 0; // Do not create any console.
    } else if (*Parameters).ConsoleHandle == HANDLE_CREATE_NEW_CONSOLE {
        /*
         * The new process has a new console, instead of inheriting
         * its parent's console.
         */
        (*Parameters).ConsoleHandle = core::ptr::null_mut(); // Do not inherit the parent's console.
    } else if (*Parameters).ConsoleHandle == HANDLE_CREATE_NO_WINDOW {
        /*
         * The process is a console application that is being run
         * without a console window. Therefore, the console handle
         * for the application is not set.
         */
        (*Parameters).ConsoleHandle = core::ptr::null_mut(); // Do not inherit the parent's console.
        ConnectInfo.IsWindowVisible = 0; // A console is created but is not shown to the user.
    } else {
    }

    /* Do nothing if this is not a console app... */
    if ConnectInfo.IsConsoleApp == 0 {
        /* Do not inherit the parent's console if we are not a console app */
        (*Parameters).ConsoleHandle = core::ptr::null_mut();
    }

    /* Now use the proper console handle */
    ConnectInfo.ConsoleStartInfo.ConsoleHandle = (*Parameters).ConsoleHandle;

    /* Initialize the console dispatchers */
    ConnectInfo.CtrlRoutine = core::mem::transmute::<unsafe extern "system" fn(LPVOID) -> DWORD, LPTHREAD_START_ROUTINE>(ConsoleControlDispatcher);
    ConnectInfo.PropRoutine = core::mem::transmute::<unsafe extern "system" fn(LPVOID) -> DWORD, LPTHREAD_START_ROUTINE>(PropDialogHandler);
    // ConnectInfo.ImeRoutine  = ImeRoutine;

    /* Set up the console properties */
    if (ConnectInfo.IsConsoleApp != 0) && ((*Parameters).ConsoleHandle).is_null() {
        /*
         * We can set up the console properties only if we create a new one
         * (we do not inherit it from our parent).
         */

        let mut ConsoleTitle: LPWSTR = ConnectInfo.ConsoleTitle.as_mut_ptr();

        ConnectInfo.TitleLength = core::mem::size_of::<[u16; 260]>() as u32;
        ConnectInfo.DesktopLength = 0; // SetUpConsoleInfo will give us the real length.

        SetUpConsoleInfo(
            1,
            &mut ConnectInfo.TitleLength,
            &mut ConsoleTitle,
            &mut ConnectInfo.DesktopLength,
            &mut ConnectInfo.Desktop,
            &mut ConnectInfo.ConsoleStartInfo,
        );
    } else {
        ConnectInfo.TitleLength = 0;
        ConnectInfo.DesktopLength = 0;
    }

    /* Initialize the Input EXE name */
    if ConnectInfo.IsConsoleApp != 0 {
        let mut CurDir: LPWSTR = ConnectInfo.CurDir.as_mut_ptr();
        let mut AppName: LPWSTR = ConnectInfo.AppName.as_mut_ptr();

        InitExeName();

        ConnectInfo.CurDirLength = core::mem::size_of::<[u16; 260]>() as u32;
        ConnectInfo.AppNameLength = core::mem::size_of::<[u16; 260]>() as u32;

        SetUpAppName(
            1,
            &mut ConnectInfo.CurDirLength,
            &mut CurDir,
            &mut ConnectInfo.AppNameLength,
            &mut AppName,
        );
    } else {
        ConnectInfo.CurDirLength = 0;
        ConnectInfo.AppNameLength = 0;
    }

    /*
     * Initialize Console Ctrl Handling, that needs to be supported by
     * all applications, especially because it is used at shutdown.
     */
    InitializeCtrlHandling();

    /* Connect to the Console Server */
    if ConnectConsole(SessionDir, &mut ConnectInfo, &mut InServerProcess) == 0 {
        /* CONSRV may not be loaded (e.g., minimal boot without winsrv).
         * Don't fail -- console APIs will be unavailable but the process can run. */
    }

    /* If we are not doing server-to-server init and if this is a console app... */
    if (InServerProcess == 0) && (ConnectInfo.IsConsoleApp != 0) {
        /* ... set the handles that we got */
        if (*Parameters).ConsoleHandle.is_null() {
            SetUpHandles(&ConnectInfo.ConsoleStartInfo);
        }

        InputWaitHandle = ConnectInfo.ConsoleStartInfo.InputWaitHandle;

        /* Sync the current thread's LangId with the console's one */
        SetTEBLangID();
    }

    return 1;
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:       See COPYING in the top level directory
//  * PROJECT:         ReactOS system libraries
//  * FILE:            dll/win32/kernel32/client/console/init.c
//  * PURPOSE:         Console API Client Initialization
//  * PROGRAMMERS:     Alex Ionescu (alex.ionescu@reactos.org)
//  *                  Aleksey Bragin (aleksey@reactos.org)
//  *                  Hermes Belusca-Maito (hermes.belusca@sfr.fr)
//  */
//
// /* INCLUDES *******************************************************************/
//
// #include <k32.h>
//
// // For Control Panel Applet
// #include <cpl.h>
//
// #define NDEBUG
// #include <debug.h>
//
//
// /* GLOBALS ********************************************************************/
//
// RTL_CRITICAL_SECTION ConsoleLock;
// BOOLEAN ConsoleInitialized = FALSE;
// extern HANDLE InputWaitHandle;
//
// static const PWSTR DefaultConsoleTitle = L"ReactOS Console";
//
// /* FUNCTIONS ******************************************************************/
//
// DWORD
// WINAPI
// PropDialogHandler(IN LPVOID lpThreadParameter)
// {
//     // NOTE: lpThreadParameter corresponds to the client shared section handle.
//
//     NTSTATUS Status = STATUS_SUCCESS;
//     HMODULE hConsoleApplet = NULL;
//     APPLET_PROC CPlApplet;
//     static BOOL AlreadyDisplayingProps = FALSE;
//     WCHAR szBuffer[MAX_PATH];
//
//     /*
//      * Do not launch more than once the console property dialog applet,
//      * or (albeit less probable), if we are not initialized.
//      */
//     if (!ConsoleInitialized || AlreadyDisplayingProps)
//     {
//         /* Close the associated client shared section handle if needed */
//         if (lpThreadParameter)
//             CloseHandle((HANDLE)lpThreadParameter);
//
//         return STATUS_UNSUCCESSFUL as DWORD;
//     }
//
//     AlreadyDisplayingProps = TRUE;
//
//     /* Load the control applet */
//     GetSystemDirectoryW(szBuffer, MAX_PATH);
//     wcscat(szBuffer, L"\\console.dll");
//     hConsoleApplet = LoadLibraryW(szBuffer);
//     if (hConsoleApplet == NULL)
//     {
//         DPRINT1("Failed to load console.dll\n");
//         Status = STATUS_UNSUCCESSFUL;
//         goto Quit;
//     }
//
//     /* Load its main function */
//     CPlApplet = (APPLET_PROC)GetProcAddress(hConsoleApplet, "CPlApplet");
//     if (CPlApplet == NULL)
//     {
//         DPRINT1("Error: console.dll misses CPlApplet export\n");
//         Status = STATUS_UNSUCCESSFUL;
//         goto Quit;
//     }
//
//     /* Initialize the applet */
//     if (CPlApplet(NULL, CPL_INIT, 0, 0) == FALSE)
//     {
//         DPRINT1("Error: failed to initialize console.dll\n");
//         Status = STATUS_UNSUCCESSFUL;
//         goto Quit;
//     }
//
//     /* Check the count */
//     if (CPlApplet(NULL, CPL_GETCOUNT, 0, 0) != 1)
//     {
//         DPRINT1("Error: console.dll returned unexpected CPL count\n");
//         Status = STATUS_UNSUCCESSFUL;
//         goto Quit;
//     }
//
//     /*
//      * Start the applet. For Windows compatibility purposes we need
//      * to pass the client shared section handle (lpThreadParameter)
//      * via the hWnd parameter of the CPlApplet function.
//      */
//     CPlApplet((HWND)lpThreadParameter, CPL_DBLCLK, 0, 0);
//
//     /* We have finished */
//     CPlApplet(NULL, CPL_EXIT, 0, 0);
//
// Quit:
//     if (hConsoleApplet) FreeLibrary(hConsoleApplet);
//     AlreadyDisplayingProps = FALSE;
//     return Status as DWORD;
// }
//
//
// static INT
// ParseShellInfo(LPCWSTR lpszShellInfo,
//                LPCWSTR lpszKeyword)
// {
//     DPRINT("ParseShellInfo is UNIMPLEMENTED\n");
//     return 0;
// }
//
//
// /*
//  * NOTE:
//  * The "LPDWORD Length" parameters point on input to the maximum size of
//  * the buffers that can hold data (if != 0), and on output they hold the
//  * real size of the data. If "Length" are == 0 on input, then on output
//  * they receive the full size of the data.
//  * The "LPWSTR* lpTitle" parameter has a double meaning:
//  * - when "CaptureTitle" is TRUE, data is copied to the buffer pointed
//  *   by the pointer (*lpTitle).
//  * - when "CaptureTitle" is FALSE, "*lpTitle" is set to the address of
//  *   the source data.
//  */
// VOID
// SetUpConsoleInfo(IN BOOLEAN CaptureTitle,
//                  IN OUT LPDWORD pTitleLength,
//                  IN OUT LPWSTR* lpTitle OPTIONAL,
//                  IN OUT LPDWORD pDesktopLength,
//                  IN OUT LPWSTR* lpDesktop OPTIONAL,
//                  IN OUT PCONSOLE_START_INFO ConsoleStartInfo)
// {
//     PRTL_USER_PROCESS_PARAMETERS Parameters = NtCurrentPeb()->ProcessParameters;
//     DWORD Length;
//
//     /* Initialize the fields */
//
//     ConsoleStartInfo->IconIndex = 0;
//     ConsoleStartInfo->hIcon   = NULL;
//     ConsoleStartInfo->hIconSm = NULL;
//     ConsoleStartInfo->dwStartupFlags = Parameters->WindowFlags;
//     ConsoleStartInfo->nFont = 0;
//     ConsoleStartInfo->nInputBufferSize = 0;
//     ConsoleStartInfo->uCodePage = GetOEMCP();
//
//     if (lpTitle)
//     {
//         LPWSTR Title;
//
//         /* If we don't have any title, use the default one */
//         if (Parameters->WindowTitle.Buffer == NULL)
//         {
//             Title  = DefaultConsoleTitle;
//             Length = lstrlenW(DefaultConsoleTitle) * sizeof(WCHAR); // sizeof(DefaultConsoleTitle);
//         }
//         else
//         {
//             Title  = Parameters->WindowTitle.Buffer;
//             Length = Parameters->WindowTitle.Length;
//         }
//
//         /* Retrieve the needed buffer size */
//         Length += sizeof(WCHAR);
//         if (*pTitleLength > 0) Length = min(Length, *pTitleLength);
//         *pTitleLength = Length;
//
//         /* Capture the data if needed, or, return a pointer to it */
//         if (CaptureTitle)
//         {
//             /*
//              * Length is always >= sizeof(WCHAR). Copy everything but the
//              * possible trailing NULL character, and then NULL-terminate.
//              */
//             Length -= sizeof(WCHAR);
//             RtlCopyMemory(*lpTitle, Title, Length);
//             (*lpTitle)[Length / sizeof(WCHAR)] = UNICODE_NULL;
//         }
//         else
//         {
//             *lpTitle = Title;
//         }
//     }
//     else
//     {
//         *pTitleLength = 0;
//     }
//
//     if (lpDesktop && Parameters->DesktopInfo.Buffer && *Parameters->DesktopInfo.Buffer)
//     {
//         /* Retrieve the needed buffer size */
//         Length = Parameters->DesktopInfo.Length + sizeof(WCHAR);
//         if (*pDesktopLength > 0) Length = min(Length, *pDesktopLength);
//         *pDesktopLength = Length;
//
//         /* Return a pointer to the data */
//         *lpDesktop = Parameters->DesktopInfo.Buffer;
//     }
//     else
//     {
//         *pDesktopLength = 0;
//         if (lpDesktop) *lpDesktop = NULL;
//     }
//
//     if (Parameters->WindowFlags & STARTF_USEFILLATTRIBUTE)
//     {
//         ConsoleStartInfo->wFillAttribute = (WORD)Parameters->FillAttribute;
//     }
//     if (Parameters->WindowFlags & STARTF_USECOUNTCHARS)
//     {
//         ConsoleStartInfo->dwScreenBufferSize.X = (SHORT)Parameters->CountCharsX;
//         ConsoleStartInfo->dwScreenBufferSize.Y = (SHORT)Parameters->CountCharsY;
//     }
//     if (Parameters->WindowFlags & STARTF_USESHOWWINDOW)
//     {
//         ConsoleStartInfo->wShowWindow = (WORD)Parameters->ShowWindowFlags;
//     }
//     if (Parameters->WindowFlags & STARTF_USEPOSITION)
//     {
//         ConsoleStartInfo->dwWindowOrigin.X = (SHORT)Parameters->StartingX;
//         ConsoleStartInfo->dwWindowOrigin.Y = (SHORT)Parameters->StartingY;
//     }
//     if (Parameters->WindowFlags & STARTF_USESIZE)
//     {
//         ConsoleStartInfo->dwWindowSize.X = (SHORT)Parameters->CountX;
//         ConsoleStartInfo->dwWindowSize.Y = (SHORT)Parameters->CountY;
//     }
//
//     /* Get shell information (ShellInfo.Buffer is NULL-terminated) */
//     if (Parameters->ShellInfo.Buffer != NULL)
//     {
//         ConsoleStartInfo->IconIndex = ParseShellInfo(Parameters->ShellInfo.Buffer, L"dde.");
//
//         if ((Parameters->WindowFlags & STARTF_USEHOTKEY) == 0)
//             ConsoleStartInfo->dwHotKey = ParseShellInfo(Parameters->ShellInfo.Buffer, L"hotkey.");
//         else
//             ConsoleStartInfo->dwHotKey = HandleToUlong(Parameters->StandardInput);
//     }
// }
//
//
// VOID
// SetUpHandles(IN PCONSOLE_START_INFO ConsoleStartInfo)
// {
//     PRTL_USER_PROCESS_PARAMETERS Parameters = NtCurrentPeb()->ProcessParameters;
//
//     if (ConsoleStartInfo->dwStartupFlags & STARTF_USEHOTKEY)
//     {
//         Parameters->WindowFlags &= ~STARTF_USEHOTKEY;
//     }
//     if (ConsoleStartInfo->dwStartupFlags & STARTF_SHELLPRIVATE)
//     {
//         Parameters->WindowFlags &= ~STARTF_SHELLPRIVATE;
//     }
//
//     /* We got the handles, let's set them */
//     Parameters->ConsoleHandle = ConsoleStartInfo->ConsoleHandle;
//
//     if ((ConsoleStartInfo->dwStartupFlags & STARTF_USESTDHANDLES) == 0)
//     {
//         Parameters->StandardInput  = ConsoleStartInfo->InputHandle;
//         Parameters->StandardOutput = ConsoleStartInfo->OutputHandle;
//         Parameters->StandardError  = ConsoleStartInfo->ErrorHandle;
//     }
// }
//
//
// static BOOLEAN
// IsConsoleApp(VOID)
// {
//     PIMAGE_NT_HEADERS ImageNtHeader = RtlImageNtHeader(NtCurrentPeb()->ImageBaseAddress);
//     return (ImageNtHeader && (ImageNtHeader->OptionalHeader.Subsystem ==
//                               IMAGE_SUBSYSTEM_WINDOWS_CUI));
// }
//
//
// static BOOLEAN
// ConnectConsole(IN PWSTR SessionDir,
//                IN PCONSRV_API_CONNECTINFO ConnectInfo,
//                OUT PBOOLEAN InServerProcess)
// {
//     NTSTATUS Status;
//     ULONG ConnectInfoSize = sizeof(*ConnectInfo);
//
//     ASSERT(SessionDir);
//
//     /* Connect to the Console Server */
//     DPRINT("Connecting to the Console Server...\n");
//     Status = CsrClientConnectToServer(SessionDir,
//                                       CONSRV_SERVERDLL_INDEX,
//                                       ConnectInfo,
//                                       &ConnectInfoSize,
//                                       InServerProcess);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("Failed to connect to the Console Server (Status %lx)\n", Status);
//         return FALSE;
//     }
//
//     /* Nothing to do for server-to-server */
//     if (*InServerProcess) return TRUE;
//
//     /* Nothing to do if this is not a console app */
//     if (!ConnectInfo->IsConsoleApp) return TRUE;
//
//     /* Wait for the connection to finish */
//     // Is ConnectInfo->ConsoleStartInfo.InitEvents aligned on handle boundary ????
//     Status = NtWaitForMultipleObjects(MAX_INIT_EVENTS,
//                                       ConnectInfo->ConsoleStartInfo.InitEvents,
//                                       WaitAny, FALSE, NULL);
//     if (!NT_SUCCESS(Status))
//     {
//         BaseSetLastNTError(Status);
//         return FALSE;
//     }
//
//     NtClose(ConnectInfo->ConsoleStartInfo.InitEvents[INIT_SUCCESS]);
//     NtClose(ConnectInfo->ConsoleStartInfo.InitEvents[INIT_FAILURE]);
//     if (Status != INIT_SUCCESS)
//     {
//         NtCurrentPeb()->ProcessParameters->ConsoleHandle = NULL;
//         return FALSE;
//     }
//
//     return TRUE;
// }
//
//
// BOOLEAN
// WINAPI
// ConDllInitialize(IN ULONG Reason,
//                  IN PWSTR SessionDir)
// {
//     NTSTATUS Status;
//     PRTL_USER_PROCESS_PARAMETERS Parameters = NtCurrentPeb()->ProcessParameters;
//     BOOLEAN InServerProcess = FALSE;
//     CONSRV_API_CONNECTINFO ConnectInfo;
//
//     if (Reason != DLL_PROCESS_ATTACH)
//     {
//         if ((Reason == DLL_THREAD_ATTACH) && IsConsoleApp())
//         {
//             /* Sync the new thread's LangId with the console's one */
//             SetTEBLangID();
//         }
//         else if (Reason == DLL_PROCESS_DETACH)
//         {
//             /* Free our resources */
//             if (ConsoleInitialized != FALSE)
//             {
//                 ConsoleInitialized = FALSE;
//                 RtlDeleteCriticalSection(&ConsoleLock);
//             }
//         }
//
//         return TRUE;
//     }
//
//     DPRINT("ConDllInitialize for: %wZ\n"
//            "Our current console handles are: 0x%p, 0x%p, 0x%p 0x%p\n",
//            &Parameters->ImagePathName,
//            Parameters->ConsoleHandle,
//            Parameters->StandardInput,
//            Parameters->StandardOutput,
//            Parameters->StandardError);
//
//     /* Initialize our global console DLL lock */
//     Status = RtlInitializeCriticalSection(&ConsoleLock);
//     if (!NT_SUCCESS(Status)) return FALSE;
//     ConsoleInitialized = TRUE;
//
//     /* Show by default the console window when applicable */
//     ConnectInfo.IsWindowVisible = TRUE;
//     /* If this is a console app, a console will be created/opened */
//     ConnectInfo.IsConsoleApp = IsConsoleApp();
//
//     /* Do nothing if this is not a console app... */
//     if (!ConnectInfo.IsConsoleApp)
//     {
//         DPRINT("Image is not a console application\n");
//     }
//
//     /*
//      * Handle the special flags given to us by BasePushProcessParameters.
//      */
//     if (Parameters->ConsoleHandle == HANDLE_DETACHED_PROCESS)
//     {
//         /* No console to create */
//         DPRINT("No console to create\n");
//         /*
//          * The new process does not inherit its parent's console and cannot
//          * attach to the console of its parent. The new process can call the
//          * AllocConsole function at a later time to create a console.
//          */
//         Parameters->ConsoleHandle = NULL;  // Do not inherit the parent's console.
//         ConnectInfo.IsConsoleApp  = FALSE; // Do not create any console.
//     }
//     else if (Parameters->ConsoleHandle == HANDLE_CREATE_NEW_CONSOLE)
//     {
//         /* We'll get the real one soon */
//         DPRINT("Creating a new separate console\n");
//         /*
//          * The new process has a new console, instead of inheriting
//          * its parent's console.
//          */
//         Parameters->ConsoleHandle = NULL; // Do not inherit the parent's console.
//     }
//     else if (Parameters->ConsoleHandle == HANDLE_CREATE_NO_WINDOW)
//     {
//         /* We'll get the real one soon */
//         DPRINT("Creating a new invisible console\n");
//         /*
//          * The process is a console application that is being run
//          * without a console window. Therefore, the console handle
//          * for the application is not set.
//          */
//         Parameters->ConsoleHandle   = NULL;  // Do not inherit the parent's console.
//         ConnectInfo.IsWindowVisible = FALSE; // A console is created but is not shown to the user.
//     }
//     else
//     {
//         DPRINT("Using existing console: 0x%p\n", Parameters->ConsoleHandle);
//     }
//
//     /* Do nothing if this is not a console app... */
//     if (!ConnectInfo.IsConsoleApp)
//     {
//         /* Do not inherit the parent's console if we are not a console app */
//         Parameters->ConsoleHandle = NULL;
//     }
//
//     /* Now use the proper console handle */
//     ConnectInfo.ConsoleStartInfo.ConsoleHandle = Parameters->ConsoleHandle;
//
//     /* Initialize the console dispatchers */
//     ConnectInfo.CtrlRoutine = ConsoleControlDispatcher;
//     ConnectInfo.PropRoutine = PropDialogHandler;
//     // ConnectInfo.ImeRoutine  = ImeRoutine;
//
//     /* Set up the console properties */
//     if (ConnectInfo.IsConsoleApp && Parameters->ConsoleHandle == NULL)
//     {
//         /*
//          * We can set up the console properties only if we create a new one
//          * (we do not inherit it from our parent).
//          */
//
//         LPWSTR ConsoleTitle = ConnectInfo.ConsoleTitle;
//
//         ConnectInfo.TitleLength   = sizeof(ConnectInfo.ConsoleTitle);
//         ConnectInfo.DesktopLength = 0; // SetUpConsoleInfo will give us the real length.
//
//         SetUpConsoleInfo(TRUE,
//                          &ConnectInfo.TitleLength,
//                          &ConsoleTitle,
//                          &ConnectInfo.DesktopLength,
//                          &ConnectInfo.Desktop,
//                          &ConnectInfo.ConsoleStartInfo);
//         DPRINT("ConsoleTitle = '%S' - Desktop = '%S'\n",
//                ConsoleTitle, ConnectInfo.Desktop);
//     }
//     else
//     {
//         ConnectInfo.TitleLength   = 0;
//         ConnectInfo.DesktopLength = 0;
//     }
//
//     /* Initialize the Input EXE name */
//     if (ConnectInfo.IsConsoleApp)
//     {
//         LPWSTR CurDir  = ConnectInfo.CurDir;
//         LPWSTR AppName = ConnectInfo.AppName;
//
//         InitExeName();
//
//         ConnectInfo.CurDirLength  = sizeof(ConnectInfo.CurDir);
//         ConnectInfo.AppNameLength = sizeof(ConnectInfo.AppName);
//
//         SetUpAppName(TRUE,
//                      &ConnectInfo.CurDirLength,
//                      &CurDir,
//                      &ConnectInfo.AppNameLength,
//                      &AppName);
//         DPRINT("CurDir = '%S' - AppName = '%S'\n",
//                CurDir, AppName);
//     }
//     else
//     {
//         ConnectInfo.CurDirLength  = 0;
//         ConnectInfo.AppNameLength = 0;
//     }
//
//     /*
//      * Initialize Console Ctrl Handling, that needs to be supported by
//      * all applications, especially because it is used at shutdown.
//      */
//     InitializeCtrlHandling();
//
//     /* Connect to the Console Server */
//     if (!ConnectConsole(SessionDir,
//                         &ConnectInfo,
//                         &InServerProcess))
//     {
//         DPRINT1("Console server not available, continuing without console support\n");
//         /* CONSRV may not be loaded (e.g., minimal boot without winsrv).
//          * Don't fail -- console APIs will be unavailable but the process can run. */
//     }
//
//     /* If we are not doing server-to-server init and if this is a console app... */
//     if (!InServerProcess && ConnectInfo.IsConsoleApp)
//     {
//         /* ... set the handles that we got */
//         if (Parameters->ConsoleHandle == NULL)
//             SetUpHandles(&ConnectInfo.ConsoleStartInfo);
//
//         InputWaitHandle = ConnectInfo.ConsoleStartInfo.InputWaitHandle;
//
//         /* Sync the current thread's LangId with the console's one */
//         SetTEBLangID();
//     }
//
//     DPRINT("Console setup: 0x%p, 0x%p, 0x%p, 0x%p\n",
//            Parameters->ConsoleHandle,
//            Parameters->StandardInput,
//            Parameters->StandardOutput,
//            Parameters->StandardError);
//
//     return TRUE;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
