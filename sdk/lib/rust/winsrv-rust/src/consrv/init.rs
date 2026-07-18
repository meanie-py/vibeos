use winsrv_sys::nt::*;
use super::console::*;
use super::handle::*;
use super::history::*;
// Phase 5.4 sibling Srv* definitions
use super::alias::*;
use super::coninput::*;
use super::frontendctl::*;
use super::subsysreg::*;
use winsrv_sys::winsrv::*;
use winsrv_sys::consts::*;
// Disambiguate winsrv_sys::winsrv:: (bindgen) vs sibling Rust impl.
use super::handle::ConSrvInheritHandlesTable;
use super::console::ConSrvValidateConsole;
use super::console::ConSrvInitConsoleSupport;

use super::{ConsoleAllocHeap, ConsoleFreeHeap};


// Srv* dispatch fns translated in other C files (conoutput.c/console.c etc.)
// — declared here so the dispatch table compiles. Real symbols come from C link.
extern "system" {
    pub fn SrvAllocConsole(ApiMessage: PCSR_API_MESSAGE, ReplyCode: PCSR_REPLY_CODE) -> NTSTATUS;
    pub fn SrvAttachConsole(ApiMessage: PCSR_API_MESSAGE, ReplyCode: PCSR_REPLY_CODE) -> NTSTATUS;
    pub fn SrvConsoleNotifyLastClose(ApiMessage: PCSR_API_MESSAGE, ReplyCode: PCSR_REPLY_CODE) -> NTSTATUS;
    pub fn SrvCreateConsoleScreenBuffer(ApiMessage: PCSR_API_MESSAGE, ReplyCode: PCSR_REPLY_CODE) -> NTSTATUS;
    pub fn SrvFillConsoleOutput(ApiMessage: PCSR_API_MESSAGE, ReplyCode: PCSR_REPLY_CODE) -> NTSTATUS;
    pub fn SrvFreeConsole(ApiMessage: PCSR_API_MESSAGE, ReplyCode: PCSR_REPLY_CODE) -> NTSTATUS;
    pub fn SrvGenerateConsoleCtrlEvent(ApiMessage: PCSR_API_MESSAGE, ReplyCode: PCSR_REPLY_CODE) -> NTSTATUS;
    pub fn SrvGetConsoleCP(ApiMessage: PCSR_API_MESSAGE, ReplyCode: PCSR_REPLY_CODE) -> NTSTATUS;
    pub fn SrvGetConsoleCharType(ApiMessage: PCSR_API_MESSAGE, ReplyCode: PCSR_REPLY_CODE) -> NTSTATUS;
    pub fn SrvGetConsoleCursorInfo(ApiMessage: PCSR_API_MESSAGE, ReplyCode: PCSR_REPLY_CODE) -> NTSTATUS;
    pub fn SrvGetConsoleCursorMode(ApiMessage: PCSR_API_MESSAGE, ReplyCode: PCSR_REPLY_CODE) -> NTSTATUS;
    pub fn SrvGetConsoleKeyboardLayoutName(ApiMessage: PCSR_API_MESSAGE, ReplyCode: PCSR_REPLY_CODE) -> NTSTATUS;
    pub fn SrvGetConsoleLangId(ApiMessage: PCSR_API_MESSAGE, ReplyCode: PCSR_REPLY_CODE) -> NTSTATUS;
    pub fn SrvGetConsoleMode(ApiMessage: PCSR_API_MESSAGE, ReplyCode: PCSR_REPLY_CODE) -> NTSTATUS;
    pub fn SrvGetConsoleMouseInfo(ApiMessage: PCSR_API_MESSAGE, ReplyCode: PCSR_REPLY_CODE) -> NTSTATUS;
    pub fn SrvGetConsoleNlsMode(ApiMessage: PCSR_API_MESSAGE, ReplyCode: PCSR_REPLY_CODE) -> NTSTATUS;
    pub fn SrvGetConsoleProcessList(ApiMessage: PCSR_API_MESSAGE, ReplyCode: PCSR_REPLY_CODE) -> NTSTATUS;
    pub fn SrvGetConsoleScreenBufferInfo(ApiMessage: PCSR_API_MESSAGE, ReplyCode: PCSR_REPLY_CODE) -> NTSTATUS;
    pub fn SrvGetConsoleTitle(ApiMessage: PCSR_API_MESSAGE, ReplyCode: PCSR_REPLY_CODE) -> NTSTATUS;
    pub fn SrvInvalidateBitMapRect(ApiMessage: PCSR_API_MESSAGE, ReplyCode: PCSR_REPLY_CODE) -> NTSTATUS;
    pub fn SrvReadConsoleOutput(ApiMessage: PCSR_API_MESSAGE, ReplyCode: PCSR_REPLY_CODE) -> NTSTATUS;
    pub fn SrvReadConsoleOutputString(ApiMessage: PCSR_API_MESSAGE, ReplyCode: PCSR_REPLY_CODE) -> NTSTATUS;
    pub fn SrvScrollConsoleScreenBuffer(ApiMessage: PCSR_API_MESSAGE, ReplyCode: PCSR_REPLY_CODE) -> NTSTATUS;
    pub fn SrvSetConsoleActiveScreenBuffer(ApiMessage: PCSR_API_MESSAGE, ReplyCode: PCSR_REPLY_CODE) -> NTSTATUS;
    pub fn SrvSetConsoleCP(ApiMessage: PCSR_API_MESSAGE, ReplyCode: PCSR_REPLY_CODE) -> NTSTATUS;
    pub fn SrvSetConsoleCursorInfo(ApiMessage: PCSR_API_MESSAGE, ReplyCode: PCSR_REPLY_CODE) -> NTSTATUS;
    pub fn SrvSetConsoleCursorMode(ApiMessage: PCSR_API_MESSAGE, ReplyCode: PCSR_REPLY_CODE) -> NTSTATUS;
    pub fn SrvSetConsoleCursorPosition(ApiMessage: PCSR_API_MESSAGE, ReplyCode: PCSR_REPLY_CODE) -> NTSTATUS;
    pub fn SrvSetConsoleKeyShortcuts(ApiMessage: PCSR_API_MESSAGE, ReplyCode: PCSR_REPLY_CODE) -> NTSTATUS;
    pub fn SrvSetConsoleLocalEUDC(ApiMessage: PCSR_API_MESSAGE, ReplyCode: PCSR_REPLY_CODE) -> NTSTATUS;
    pub fn SrvSetConsoleMode(ApiMessage: PCSR_API_MESSAGE, ReplyCode: PCSR_REPLY_CODE) -> NTSTATUS;
    pub fn SrvSetConsoleNlsMode(ApiMessage: PCSR_API_MESSAGE, ReplyCode: PCSR_REPLY_CODE) -> NTSTATUS;
    pub fn SrvSetConsolePalette(ApiMessage: PCSR_API_MESSAGE, ReplyCode: PCSR_REPLY_CODE) -> NTSTATUS;
    pub fn SrvSetConsoleScreenBufferSize(ApiMessage: PCSR_API_MESSAGE, ReplyCode: PCSR_REPLY_CODE) -> NTSTATUS;
    pub fn SrvSetConsoleTextAttribute(ApiMessage: PCSR_API_MESSAGE, ReplyCode: PCSR_REPLY_CODE) -> NTSTATUS;
    pub fn SrvSetConsoleTitle(ApiMessage: PCSR_API_MESSAGE, ReplyCode: PCSR_REPLY_CODE) -> NTSTATUS;
    pub fn SrvSetConsoleWindowInfo(ApiMessage: PCSR_API_MESSAGE, ReplyCode: PCSR_REPLY_CODE) -> NTSTATUS;
    pub fn SrvWriteConsole(ApiMessage: PCSR_API_MESSAGE, ReplyCode: PCSR_REPLY_CODE) -> NTSTATUS;
    pub fn SrvWriteConsoleOutput(ApiMessage: PCSR_API_MESSAGE, ReplyCode: PCSR_REPLY_CODE) -> NTSTATUS;
    pub fn SrvWriteConsoleOutputString(ApiMessage: PCSR_API_MESSAGE, ReplyCode: PCSR_REPLY_CODE) -> NTSTATUS;
}

#[cfg_attr(feature = "winsrv_dll_only", no_mangle)]
pub static mut ConSrvDllInstance: HINSTANCE = core::ptr::null_mut();

#[cfg_attr(feature = "winsrv_dll_only", no_mangle)]
pub static mut ConSrvHeap: HANDLE = core::ptr::null_mut();

// Shadow PCSR_API_ROUTINE locally with extern "system" — bindgen
// emits arch-specific ABI literals (extern "stdcall" on i386, extern
// "C" elsewhere) and Rust's type system treats those as distinct from
// extern "system" even when codegen is identical. `LoadedServerDll.
// DispatchTable` is `*mut PCSR_API_ROUTINE` so the pointer is cast at
// the assignment site. Mirrors basesrv/init.rs in smss-csrss-rust and
// the 5.2 usersrv init.rs CsrApiRoutineSystem pattern.
type ConsoleApiRoutineSystem = Option<unsafe extern "system" fn(
    ApiMessage: PCSR_API_MESSAGE,
    ReplyCode: PCSR_REPLY_CODE,
) -> NTSTATUS>;

#[cfg_attr(feature = "winsrv_dll_only", no_mangle)]
pub static mut ConsoleServerApiDispatchTable: [ConsoleApiRoutineSystem; ConsolepMaxApiNumber as usize - CONSRV_FIRST_API_NUMBER as usize] = [
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
    0, 0, 0, 0, 0, 0, 0,
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

    let TargetProcessData = (*(*TargetProcess).ServerData.as_ptr().add(CONSRV_SERVERDLL_INDEX as usize)) as PCONSOLE_PROCESS_DATA ;

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
        let SourceProcessData = (*(*SourceProcess).ServerData.as_ptr().add(CONSRV_SERVERDLL_INDEX as usize)) as PCONSOLE_PROCESS_DATA ;
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
    let ProcessData = (*(*CsrProcess).ServerData.as_ptr().add(CONSRV_SERVERDLL_INDEX as usize)) as PCONSOLE_PROCESS_DATA ;

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
        ConsoleInitInfo.ConsoleTitle = (*ConnectInfo).ConsoleTitle.as_mut_ptr();
        ConsoleInitInfo.DesktopLength = 0;
        ConsoleInitInfo.Desktop = core::ptr::null_mut();
        ConsoleInitInfo.AppNameLength = (*ConnectInfo).AppNameLength;
        ConsoleInitInfo.AppName = (*ConnectInfo).AppName.as_mut_ptr();
        ConsoleInitInfo.CurDirLength = (*ConnectInfo).CurDirLength;
        ConsoleInitInfo.CurDir = (*ConnectInfo).CurDir.as_mut_ptr();

        if (*ConnectInfo).DesktopLength != 0 {
            ConsoleInitInfo.DesktopLength = (*ConnectInfo).DesktopLength;

            ConsoleInitInfo.Desktop = ConsoleAllocHeap(HEAP_ZERO_MEMORY, ConsoleInitInfo.DesktopLength as usize) as *mut u16;
            if ConsoleInitInfo.Desktop.is_null() {
                return STATUS_NO_MEMORY;
            }

            Status = NtReadVirtualMemory(
                (*(*ProcessData).Process).ProcessHandle,
                (*ConnectInfo).Desktop as PVOID,
                ConsoleInitInfo.Desktop as PVOID,
                ConsoleInitInfo.DesktopLength as SIZE_T,
                core::ptr::null_mut(),
            );
            if Status < 0 {
                ConsoleFreeHeap(ConsoleInitInfo.Desktop as *mut core::ffi::c_void);
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
            ConsoleFreeHeap(ConsoleInitInfo.Desktop as *mut core::ffi::c_void);
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
    let ProcessData = (*(*CsrProcess).ServerData.as_ptr().add(CONSRV_SERVERDLL_INDEX as usize)) as PCONSOLE_PROCESS_DATA ;

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
    (*LoadedServerDll).DispatchTable = ConsoleServerApiDispatchTable.as_mut_ptr() as *mut PCSR_API_ROUTINE;
    (*LoadedServerDll).ValidTable = ConsoleServerApiServerValidTable.as_mut_ptr();
    #[cfg(feature = "CSR_DBG")]
    {
        (*LoadedServerDll).NameTable = ConsoleServerApiNameTable.as_mut_ptr();
    }
    (*LoadedServerDll).SizeOfProcessData = core::mem::size_of::<CONSOLE_PROCESS_DATA>() as ULONG;
    // CSR_SERVER_DLL callback field types use extern "stdcall" on i386 /
    // extern "C" on amd64+arm64 (bindgen __stdcall); our Rust impls are
    // extern "system". Codegen is identical per arch; transmute bridges
    // the Rust-type-system difference.
    (*LoadedServerDll).ConnectCallback = Some(core::mem::transmute(ConSrvConnect as *const ()));
    (*LoadedServerDll).DisconnectCallback = Some(core::mem::transmute(ConSrvDisconnect as *const ()));
    (*LoadedServerDll).NewProcessCallback = Some(core::mem::transmute(ConSrvNewProcess as *const ()));
    // `ConsoleClientShutdown` exists both in winsrv-sys (bindgen-emitted
    // extern decl) and as a sibling Rust impl. Resolve to the local one.
    (*LoadedServerDll).ShutdownProcessCallback = Some(core::mem::transmute(crate::consrv::shutdown::ConsoleClientShutdown as *const ()));

    ConSrvDllInstance = (*LoadedServerDll).ServerHandle as HINSTANCE;

    STATUS_SUCCESS
}