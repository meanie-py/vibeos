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
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// No file-scope declarations needed for this translation unit.
//
// All functions reference types already imported from ntkrnl_sys (PCONSOLE, PTEXTMODE_SCREEN_BUFFER, etc.)
// and contain no static data, typedefs, enums, or constant definitions at file scope.

pub(crate) unsafe fn WriteConsoleThread(
    WaitList: PLIST_ENTRY,
    WaitThread: PCSR_THREAD,
    WaitApiMessage: PCSR_API_MESSAGE,
    WaitContext: *mut core::ffi::c_void,
    WaitArgument1: *mut core::ffi::c_void,
    WaitArgument2: *mut core::ffi::c_void,
    WaitFlags: u32,
) -> u8 {
    let mut Status: i32;

    if (WaitFlags & CsrProcessTerminating) != 0 {
        Status = STATUS_THREAD_IS_TERMINATING;
        'Quit: {
            if Status != STATUS_PENDING {
                (*WaitApiMessage).__bindgen_anon_1.__bindgen_anon_1.Status = Status;
            }
            return if Status == STATUS_PENDING { 0u8 } else { 1u8 };
        }
    }

    Status = DoWriteConsole(WaitApiMessage, WaitThread, 0u8);

    if Status != STATUS_PENDING {
        (*WaitApiMessage).__bindgen_anon_1.__bindgen_anon_1.Status = Status;
    }

    if Status == STATUS_PENDING { 0u8 } else { 1u8 }
}

pub(crate) unsafe fn DoWriteConsole(
    ApiMessage: PCSR_API_MESSAGE,
    ClientThread: PCSR_THREAD,
    CreateWaitBlock: u8,
) -> i32 {
    let mut Status: i32;
    let WriteConsoleRequest: PCONSOLE_WRITECONSOLE = 
        core::ptr::addr_of_mut!((*((ApiMessage as PCONSOLE_API_MESSAGE))).Data.WriteConsoleRequest);
    let mut ScreenBuffer: PTEXTMODE_SCREEN_BUFFER = core::ptr::null_mut();

    let Buffer: *mut core::ffi::c_void;
    let mut NrCharactersWritten: u32 = 0;
    let CharSize: u32 = if (*WriteConsoleRequest).Unicode != 0 {
        core::mem::size_of::<u16>() as u32
    } else {
        core::mem::size_of::<u8>() as u32
    };

    Status = ConSrvGetTextModeBuffer(
        ConsoleGetPerProcessData((*ClientThread).Process),
        (*WriteConsoleRequest).OutputHandle,
        &mut ScreenBuffer as *mut PTEXTMODE_SCREEN_BUFFER as *mut PCONSOLE_IO_OBJECT,
        GENERIC_WRITE,
        0u8,
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }

    if (*WriteConsoleRequest).UsingStaticBuffer != 0
        && (*WriteConsoleRequest).NumBytes <= core::mem::size_of_val(&(*WriteConsoleRequest).StaticBuffer) as u32
    {
        Buffer = (*WriteConsoleRequest).StaticBuffer.as_mut_ptr() as *mut core::ffi::c_void;
    } else {
        Buffer = (*WriteConsoleRequest).Buffer;
    }

    Status = ConDrvWriteConsole(
        (*ScreenBuffer).Header.Console,
        ScreenBuffer,
        (*WriteConsoleRequest).Unicode,
        Buffer,
        (*WriteConsoleRequest).NumBytes / CharSize,
        &mut NrCharactersWritten,
    );

    if Status == STATUS_PENDING {
        if CreateWaitBlock != 0 {
            let Console: PCONSRV_CONSOLE = (*ScreenBuffer).Header.Console as PCONSRV_CONSOLE;

            if CsrCreateWait(
                core::ptr::addr_of_mut!((*Console).WriteWaitQueue),
                Some(core::mem::transmute(WriteConsoleThread as *const ())),
                ClientThread,
                ApiMessage,
                core::ptr::null_mut(),
            ) == 0
            {
                Status = STATUS_NO_MEMORY;
                ConSrvReleaseScreenBuffer(ScreenBuffer, 0u8);
                return Status;
            }
        }
    } else {
        (*WriteConsoleRequest).NumBytes = NrCharactersWritten * CharSize;
    }

    ConSrvReleaseScreenBuffer(ScreenBuffer, 0u8);
    Status
}
