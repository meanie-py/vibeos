use winsrv_sys::nt::*;
use winsrv_sys::winsrv::*;

#[cfg_attr(feature = "winsrv_dll_only", no_mangle)]
pub static mut ConSrvDllInstance: HINSTANCE = core::ptr::null_mut();

#[cfg_attr(feature = "winsrv_dll_only", no_mangle)]
pub static mut ConSrvHeap: HANDLE = core::ptr::null_mut();

#[cfg_attr(feature = "winsrv_dll_only", no_mangle)]
pub static mut ConsoleServerApiDispatchTable: [PCSR_API_ROUTINE; ConsolepMaxApiNumber as usize - CONSRV_FIRST_API_NUMBER as usize] = [
    Some(SrvOpenConsole),
    Some(SrvGetConsoleInput),
    Some(SrvWriteConsoleInput),
    Some(SrvReadConsoleOutput),
    Some(SrvWriteConsoleOutput),
    Some(SrvReadConsoleOutputString),
    Some(SrvWriteConsoleOutputString),
    Some(SrvFillConsoleOutput),
    Some(SrvGetConsoleMode),
    Some(SrvGetConsoleNumberOfFonts),
    Some(SrvGetConsoleNumberOfInputEvents),
    Some(SrvGetConsoleScreenBufferInfo),
    Some(SrvGetConsoleCursorInfo),
    Some(SrvGetConsoleMouseInfo),
    Some(SrvGetConsoleFontInfo),
    Some(SrvGetConsoleFontSize),
    Some(SrvGetConsoleCurrentFont),
    Some(SrvSetConsoleMode),
    Some(SrvSetConsoleActiveScreenBuffer),
    Some(SrvFlushConsoleInputBuffer),
    Some(SrvGetLargestConsoleWindowSize),
    Some(SrvSetConsoleScreenBufferSize),
    Some(SrvSetConsoleCursorPosition),
    Some(SrvSetConsoleCursorInfo),
    Some(SrvSetConsoleWindowInfo),
    Some(SrvScrollConsoleScreenBuffer),
    Some(SrvSetConsoleTextAttribute),
    Some(SrvSetConsoleFont),
    Some(SrvSetConsoleIcon),
    Some(SrvReadConsole),
    Some(SrvWriteConsole),
    Some(SrvDuplicateHandle),
    Some(SrvGetHandleInformation),
    Some(SrvSetHandleInformation),
    Some(SrvCloseHandle),
    Some(SrvVerifyConsoleIoHandle),
    Some(SrvAllocConsole),
    Some(SrvFreeConsole),
    Some(SrvGetConsoleTitle),
    Some(SrvSetConsoleTitle),
    Some(SrvCreateConsoleScreenBuffer),
    Some(SrvInvalidateBitMapRect),
    Some(SrvVDMConsoleOperation),
    Some(SrvSetConsoleCursor),
    Some(SrvShowConsoleCursor),
    Some(SrvConsoleMenuControl),
    Some(SrvSetConsolePalette),
    Some(SrvSetConsoleDisplayMode),
    Some(SrvRegisterConsoleVDM),
    Some(SrvGetConsoleHardwareState),
    Some(SrvSetConsoleHardwareState),
    Some(SrvGetConsoleDisplayMode),
    Some(SrvAddConsoleAlias),
    Some(SrvGetConsoleAlias),
    Some(SrvGetConsoleAliasesLength),
    Some(SrvGetConsoleAliasExesLength),
    Some(SrvGetConsoleAliases),
    Some(SrvGetConsoleAliasExes),
    Some(SrvExpungeConsoleCommandHistory),
    Some(SrvSetConsoleNumberOfCommands),
    Some(SrvGetConsoleCommandHistoryLength),
    Some(SrvGetConsoleCommandHistory),
    Some(SrvSetConsoleCommandHistoryMode),
    Some(SrvGetConsoleCP),
    Some(SrvSetConsoleCP),
    Some(SrvSetConsoleKeyShortcuts),
    Some(SrvSetConsoleMenuClose),
    Some(SrvConsoleNotifyLastClose),
    Some(SrvGenerateConsoleCtrlEvent),
    Some(SrvGetConsoleKeyboardLayoutName),
    Some(SrvGetConsoleWindow),
    Some(SrvGetConsoleCharType),
    Some(SrvSetConsoleLocalEUDC),
    Some(SrvSetConsoleCursorMode),
    Some(SrvGetConsoleCursorMode),
    Some(SrvRegisterConsoleOS2),
    Some(SrvSetConsoleOS2OemFormat),
    Some(SrvGetConsoleNlsMode),
    Some(SrvSetConsoleNlsMode),
    Some(SrvRegisterConsoleIME),
    Some(SrvUnregisterConsoleIME),
    Some(SrvGetConsoleLangId),
    Some(SrvAttachConsole),
    Some(SrvGetConsoleSelectionInfo),
    Some(SrvGetConsoleProcessList),
    Some(SrvGetConsoleHistory),
    Some(SrvSetConsoleHistory),
];

#[cfg_attr(feature = "winsrv_dll_only", no_mangle)]
pub static mut ConsoleServerApiServerValidTable: [BOOLEAN; ConsolepMaxApiNumber as usize - CONSRV_FIRST_API_NUMBER as usize] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 1, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0,
];

#[cfg(feature = "CSR_DBG")]
#[cfg_attr(feature = "winsrv_dll_only", no_mangle)]
pub static mut ConsoleServerApiNameTable: [PCHAR; ConsolepMaxApiNumber as usize - CONSRV_FIRST_API_NUMBER as usize] = [
    b"OpenConsole\0".as_ptr() as PCHAR,
    b"GetConsoleInput\0".as_ptr() as PCHAR,
    b"WriteConsoleInput\0".as_ptr() as PCHAR,
    b"ReadConsoleOutput\0".as_ptr() as PCHAR,
    b"WriteConsoleOutput\0".as_ptr() as PCHAR,
    b"ReadConsoleOutputString\0".as_ptr() as PCHAR,
    b"WriteConsoleOutputString\0".as_ptr() as PCHAR,
    b"FillConsoleOutput\0".as_ptr() as PCHAR,
    b"GetConsoleMode\0".as_ptr() as PCHAR,
    b"GetConsoleNumberOfFonts\0".as_ptr() as PCHAR,
    b"GetConsoleNumberOfInputEvents\0".as_ptr() as PCHAR,
    b"GetConsoleScreenBufferInfo\0".as_ptr() as PCHAR,
    b"GetConsoleCursorInfo\0".as_ptr() as PCHAR,
    b"GetConsoleMouseInfo\0".as_ptr() as PCHAR,
    b"GetConsoleFontInfo\0".as_ptr() as PCHAR,
    b"GetConsoleFontSize\0".as_ptr() as PCHAR,
    b"GetConsoleCurrentFont\0".as_ptr() as PCHAR,
    b"SetConsoleMode\0".as_ptr() as PCHAR,
    b"SetConsoleActiveScreenBuffer\0".as_ptr() as PCHAR,
    b"FlushConsoleInputBuffer\0".as_ptr() as PCHAR,
    b"GetLargestConsoleWindowSize\0".as_ptr() as PCHAR,
    b"SetConsoleScreenBufferSize\0".as_ptr() as PCHAR,
    b"SetConsoleCursorPosition\0".as_ptr() as PCHAR,
    b"SetConsoleCursorInfo\0".as_ptr() as PCHAR,
    b"SetConsoleWindowInfo\0".as_ptr() as PCHAR,
    b"ScrollConsoleScreenBuffer\0".as_ptr() as PCHAR,
    b"SetConsoleTextAttribute\0".as_ptr() as PCHAR,
    b"SetConsoleFont\0".as_ptr() as PCHAR,
    b"SetConsoleIcon\0".as_ptr() as PCHAR,
    b"ReadConsole\0".as_ptr() as PCHAR,
    b"WriteConsole\0".as_ptr() as PCHAR,
    b"DuplicateHandle\0".as_ptr() as PCHAR,
    b"GetHandleInformation\0".as_ptr() as PCHAR,
    b"SetHandleInformation\0".as_ptr() as PCHAR,
    b"CloseHandle\0".as_ptr() as PCHAR,
    b"VerifyConsoleIoHandle\0".as_ptr() as PCHAR,
    b"AllocConsole\0".as_ptr() as PCHAR,
    b"FreeConsole\0".as_ptr() as PCHAR,
    b"GetConsoleTitle\0".as_ptr() as PCHAR,
    b"SetConsoleTitle\0".as_ptr() as PCHAR,
    b"CreateConsoleScreenBuffer\0".as_ptr() as PCHAR,
    b"InvalidateBitMapRect\0".as_ptr() as PCHAR,
    b"VDMConsoleOperation\0".as_ptr() as PCHAR,
    b"SetConsoleCursor\0".as_ptr() as PCHAR,
    b"ShowConsoleCursor\0".as_ptr() as PCHAR,
    b"ConsoleMenuControl\0".as_ptr() as PCHAR,
    b"SetConsolePalette\0".as_ptr() as PCHAR,
    b"SetConsoleDisplayMode\0".as_ptr() as PCHAR,
    b"RegisterConsoleVDM\0".as_ptr() as PCHAR,
    b"GetConsoleHardwareState\0".as_ptr() as PCHAR,
    b"SetConsoleHardwareState\0".as_ptr() as PCHAR,
    b"GetConsoleDisplayMode\0".as_ptr() as PCHAR,
    b"AddConsoleAlias\0".as_ptr() as PCHAR,
    b"GetConsoleAlias\0".as_ptr() as PCHAR,
    b"GetConsoleAliasesLength\0".as_ptr() as PCHAR,
    b"GetConsoleAliasExesLength\0".as_ptr() as PCHAR,
    b"GetConsoleAliases\0".as_ptr() as PCHAR,
    b"GetConsoleAliasExes\0".as_ptr() as PCHAR,
    b"ExpungeConsoleCommandHistory\0".as_ptr() as PCHAR,
    b"SetConsoleNumberOfCommands\0".as_ptr() as PCHAR,
    b"GetConsoleCommandHistoryLength\0".as_ptr() as PCHAR,
    b"GetConsoleCommandHistory\0".as_ptr() as PCHAR,
    b"SetConsoleCommandHistoryMode\0".as_ptr() as PCHAR,
    b"GetConsoleCP\0".as_ptr() as PCHAR,
    b"SetConsoleCP\0".as_ptr() as PCHAR,
    b"SetConsoleKeyShortcuts\0".as_ptr() as PCHAR,
    b"SetConsoleMenuClose\0".as_ptr() as PCHAR,
    b"ConsoleNotifyLastClose\0".as_ptr() as PCHAR,
    b"GenerateConsoleCtrlEvent\0".as_ptr() as PCHAR,
    b"GetConsoleKeyboardLayoutName\0".as_ptr() as PCHAR,
    b"GetConsoleWindow\0".as_ptr() as PCHAR,
    b"GetConsoleCharType\0".as_ptr() as PCHAR,
    b"SetConsoleLocalEUDC\0".as_ptr() as PCHAR,
    b"SetConsoleCursorMode\0".as_ptr() as PCHAR,
    b"GetConsoleCursorMode\0".as_ptr() as PCHAR,
    b"RegisterConsoleOS2\0".as_ptr() as PCHAR,
    b"SetConsoleOS2OemFormat\0".as_ptr() as PCHAR,
    b"GetConsoleNlsMode\0".as_ptr() as PCHAR,
    b"SetConsoleNlsMode\0".as_ptr() as PCHAR,
    b"RegisterConsoleIME\0".as_ptr() as PCHAR,
    b"UnregisterConsoleIME\0".as_ptr() as PCHAR,
    b"GetConsoleLangId\0".as_ptr() as PCHAR,
    b"AttachConsole\0".as_ptr() as PCHAR,
    b"GetConsoleSelectionInfo\0".as_ptr() as PCHAR,
    b"GetConsoleProcessList\0".as_ptr() as PCHAR,
    b"GetConsoleHistory\0".as_ptr() as PCHAR,
    b"SetConsoleHistory\0".as_ptr() as PCHAR,
];

#[cfg_attr(feature = "winsrv_dll_only", no_mangle)]
pub unsafe extern "system" fn ConSrvNewProcess(
    SourceProcess: PCSR_PROCESS,
    TargetProcess: PCSR_PROCESS,
) -> NTSTATUS {
    let mut Status = STATUS_SUCCESS;

    if TargetProcess.is_null() {
        return STATUS_INVALID_PARAMETER;
    }

    let TargetProcessData = (((*TargetProcess).ServerData[CONSRV_SERVERDLL_INDEX as usize]) as PCONSOLE_PROCESS_DATA);

    core::ptr::write_bytes(TargetProcessData as *mut u8, 0, core::mem::size_of::<CONSOLE_PROCESS_DATA>());
    (*TargetProcessData).Process = TargetProcess;
    (*TargetProcessData).ConsoleHandle = core::ptr::null_mut();
    (*TargetProcessData).ConsoleApp = 0u8;

    (*TargetProcessData).HandleTableSize = 0;
    (*TargetProcessData).HandleTable = core::ptr::null_mut();

    RtlInitializeCriticalSection(core::ptr::addr_of_mut!((*TargetProcessData).HandleTableLock));

    if SourceProcess.is_null() {
        return STATUS_SUCCESS;
    }

    if ((*TargetProcess).Flags & CsrProcessIsConsoleApp) != 0 {
        let SourceProcessData = (((*SourceProcess).ServerData[CONSRV_SERVERDLL_INDEX as usize]) as PCONSOLE_PROCESS_DATA);
        let mut SourceConsole: PCONSRV_CONSOLE = core::ptr::null_mut();

        if ConSrvValidateConsole(
            &mut SourceConsole as *mut PCONSRV_CONSOLE,
            (*SourceProcessData).ConsoleHandle,
            CONSOLE_RUNNING,
            1u8,
        ) != 0
        {
            Status = ConSrvInheritHandlesTable(SourceProcessData, TargetProcessData);
            if Status < 0 {
                // DPRINT1
            }

            LeaveCriticalSection(core::ptr::addr_of_mut!((*SourceConsole).Lock));
        }
    }

    Status
}

#[cfg_attr(feature = "winsrv_dll_only", no_mangle)]
pub unsafe extern "system" fn ConSrvConnect(
    CsrProcess: PCSR_PROCESS,
    ConnectionInfo: PVOID,
    ConnectionInfoLength: PULONG,
) -> NTSTATUS {
    let mut Status = STATUS_SUCCESS;
    let ConnectInfo = ConnectionInfo as PCONSRV_API_CONNECTINFO;
    let ProcessData = (((*CsrProcess).ServerData[CONSRV_SERVERDLL_INDEX as usize]) as PCONSOLE_PROCESS_DATA);

    if ConnectionInfo.is_null() || ConnectionInfoLength.is_null() || *ConnectionInfoLength != core::mem::size_of::<CONSRV_API_CONNECTINFO>() as ULONG {
        return STATUS_UNSUCCESSFUL;
    }

    (*ProcessData).CtrlRoutine = (*ConnectInfo).CtrlRoutine;

    if (*ConnectInfo).IsConsoleApp == 0 {
        return STATUS_SUCCESS;
    }

    if (*ConnectInfo).ConsoleStartInfo.ConsoleHandle.is_null() {
        let mut ConsoleInitInfo: CONSOLE_INIT_INFO = core::mem::zeroed();

        ConsoleInitInfo.ConsoleStartInfo = core::ptr::addr_of_mut!((*ConnectInfo).ConsoleStartInfo);
        ConsoleInitInfo.IsWindowVisible = (*ConnectInfo).IsWindowVisible;
        ConsoleInitInfo.TitleLength = (*ConnectInfo).TitleLength;
        ConsoleInitInfo.ConsoleTitle = (*ConnectInfo).ConsoleTitle;
        ConsoleInitInfo.DesktopLength = 0;
        ConsoleInitInfo.Desktop = core::ptr::null_mut();
        ConsoleInitInfo.AppNameLength = (*ConnectInfo).AppNameLength;
        ConsoleInitInfo.AppName = (*ConnectInfo).AppName;
        ConsoleInitInfo.CurDirLength = (*ConnectInfo).CurDirLength;
        ConsoleInitInfo.CurDir = (*ConnectInfo).CurDir;

        if (*ConnectInfo).DesktopLength != 0 {
            ConsoleInitInfo.DesktopLength = (*ConnectInfo).DesktopLength;

            ConsoleInitInfo.Desktop = ConsoleAllocHeap(HEAP_ZERO_MEMORY, ConsoleInitInfo.DesktopLength as usize);
            if ConsoleInitInfo.Desktop.is_null() {
                return STATUS_NO_MEMORY;
            }

            Status = NtReadVirtualMemory(
                (*(*ProcessData).Process).ProcessHandle,
                (*ConnectInfo).Desktop as PVOID,
                ConsoleInitInfo.Desktop,
                ConsoleInitInfo.DesktopLength as usize,
                core::ptr::null_mut(),
            );
            if Status < 0 {
                ConsoleFreeHeap(ConsoleInitInfo.Desktop);
                return Status;
            }
        }

        Status = ConSrvAllocateConsole(
            ProcessData,
            core::ptr::addr_of_mut!((*ConnectInfo).ConsoleStartInfo.InputHandle),
            core::ptr::addr_of_mut!((*ConnectInfo).ConsoleStartInfo.OutputHandle),
            core::ptr::addr_of_mut!((*ConnectInfo).ConsoleStartInfo.ErrorHandle),
            &mut ConsoleInitInfo as *mut CONSOLE_INIT_INFO,
        );

        if ConsoleInitInfo.DesktopLength != 0 {
            ConsoleFreeHeap(ConsoleInitInfo.Desktop);
        }

        if Status < 0 {
            return Status;
        }
    } else {
        Status = ConSrvInheritConsole(
            ProcessData,
            (*ConnectInfo).ConsoleStartInfo.ConsoleHandle,
            0u8,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!((*ConnectInfo).ConsoleStartInfo),
        );
        if Status < 0 {
            return Status;
        }
    }

    (*ProcessData).PropRoutine = (*ConnectInfo).PropRoutine;

    STATUS_SUCCESS
}

#[cfg_attr(feature = "winsrv_dll_only", no_mangle)]
pub unsafe extern "system" fn ConSrvDisconnect(CsrProcess: PCSR_PROCESS) {
    let ProcessData = (((*CsrProcess).ServerData[CONSRV_SERVERDLL_INDEX as usize]) as PCONSOLE_PROCESS_DATA);

    if !(*ProcessData).ConsoleHandle.is_null() || !(*ProcessData).HandleTable.is_null() {
        ConSrvRemoveConsole(ProcessData);
    }

    RtlDeleteCriticalSection(core::ptr::addr_of_mut!((*ProcessData).HandleTableLock));
}

#[cfg_attr(feature = "winsrv_dll_only", no_mangle)]
pub unsafe extern "system" fn ConServerDllInitialization(LoadedServerDll: PCSR_SERVER_DLL) -> NTSTATUS {
    ConSrvHeap = RtlGetProcessHeap();

    ConSrvInitConsoleSupport();

    (*LoadedServerDll).ApiBase = CONSRV_FIRST_API_NUMBER;
    (*LoadedServerDll).HighestApiSupported = ConsolepMaxApiNumber;
    (*LoadedServerDll).DispatchTable = ConsoleServerApiDispatchTable.as_mut_ptr();
    (*LoadedServerDll).ValidTable = ConsoleServerApiServerValidTable.as_mut_ptr();
    #[cfg(feature = "CSR_DBG")]
    {
        (*LoadedServerDll).NameTable = ConsoleServerApiNameTable.as_mut_ptr();
    }
    (*LoadedServerDll).SizeOfProcessData = core::mem::size_of::<CONSOLE_PROCESS_DATA>() as ULONG;
    (*LoadedServerDll).ConnectCallback = Some(ConSrvConnect);
    (*LoadedServerDll).DisconnectCallback = Some(ConSrvDisconnect);
    (*LoadedServerDll).NewProcessCallback = Some(ConSrvNewProcess);
    (*LoadedServerDll).ShutdownProcessCallback = Some(ConsoleClientShutdown);

    ConSrvDllInstance = (*LoadedServerDll).ServerHandle;

    STATUS_SUCCESS
}