use winsrv_sys::nt::*;
use super::condrv::text::*;
use super::condrv::conoutput::*;
use super::condrv::coninput::*;
use super::condrv::console::*;
use super::console::*;
use super::handle::*;
use super::lineinput::*;
use super::history::*;
use super::init::*;
use winsrv_sys::winsrv::*;
use winsrv_sys::consts::*;
// Disambiguate winsrv_sys::winsrv:: (bindgen) vs sibling Rust impl.
use super::console::ConSrvConsoleCtrlEventTimeout;


pub(crate) unsafe fn NotifyConsoleProcessForShutdown(
    CsrProcess: PCSR_PROCESS,
    ProcessData: PCONSOLE_PROCESS_DATA,
    Flags: ULONG,
) {
    ConSrvConsoleCtrlEventTimeout(
        CTRL_LOGOFF_EVENT,
        ProcessData,
        ShutdownSettings.WaitToKillAppTimeout,
    );
}

pub(crate) unsafe fn NotifyGenericProcessForShutdown(
    CsrProcess: PCSR_PROCESS,
    Flags: ULONG,
) -> BOOL {
    1
}

#[no_mangle]
pub unsafe extern "system" fn NonConsoleProcessShutdown(
    Process: PCSR_PROCESS,
    Flags: ULONG,
) -> ULONG {
    if NotifyGenericProcessForShutdown(Process, Flags) != 0 {
        NtTerminateProcess((*Process).ProcessHandle, 0);
        WaitForSingleObject((*Process).ProcessHandle, ShutdownSettings.ProcessTerminateTimeout);
    }

    CsrDereferenceProcess(Process);
    CsrShutdownCsrProcess
}

#[no_mangle]
pub unsafe extern "system" fn ConsoleClientShutdown(
    CsrProcess: PCSR_PROCESS,
    Flags: ULONG,
    FirstPhase: BOOLEAN,
) -> ULONG {
    let ProcessData = ((*CsrProcess).ServerData[CONSRV_SERVERDLL_INDEX as usize]) as PCONSOLE_PROCESS_DATA;

    if (Flags & EWX_SHUTDOWN) == EWX_LOGOFF &&
       ((*CsrProcess).ShutdownFlags & (CsrShutdownSystem | CsrShutdownOther)) != 0
    {
        return CsrShutdownNonCsrProcess;
    }

    if !(*ProcessData).ConsoleHandle.is_null() ||
       !(*ProcessData).HandleTable.is_null()
    {
        NotifyConsoleProcessForShutdown(CsrProcess, ProcessData, Flags);
        CsrDereferenceProcess(CsrProcess);
        return CsrShutdownCsrProcess;
    } else {
        if FirstPhase != 0 {
            return CsrShutdownNonCsrProcess;
        }

        return NonConsoleProcessShutdown(CsrProcess, Flags);
    }
}