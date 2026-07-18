use winsrv_sys::nt::*;
use super::condrv::coninput::*;
use super::console::*;
use super::console::ConSrvGetConsole;
use super::console::ConSrvReleaseConsole;
use super::GetConsoleInputBufferMode;
use super::{ConsoleAllocHeap, ConsoleFreeHeap};
use winsrv_sys::winsrv::*;
use winsrv_sys::consts::*;
// Disambiguate winsrv_sys::winsrv:: (bindgen) vs sibling Rust impl.
use super::handle::ConSrvGetObject;
use super::handle::ConSrvReleaseObject;
use super::console::ConSrvConsoleProcessCtrlEvent;


macro_rules! ConSrvGetInputBuffer {
    ($ProcessData:expr, $Handle:expr, $Ptr:expr, $Access:expr, $LockConsole:expr) => {
        ConSrvGetObject(
            $ProcessData,
            $Handle,
            $Ptr as *mut _ as *mut PCONSOLE_IO_OBJECT,
            core::ptr::null_mut(),
            $Access,
            $LockConsole,
            INPUT_BUFFER,
        )
    };
}

macro_rules! ConSrvGetInputBufferAndHandleEntry {
    ($ProcessData:expr, $Handle:expr, $Ptr:expr, $Entry:expr, $Access:expr, $LockConsole:expr) => {
        ConSrvGetObject(
            $ProcessData,
            $Handle,
            $Ptr as *mut _ as *mut PCONSOLE_IO_OBJECT,
            $Entry,
            $Access,
            $LockConsole,
            INPUT_BUFFER,
        )
    };
}

macro_rules! ConSrvReleaseInputBuffer {
    ($Buff:expr, $IsConsoleLocked:expr) => {
        ConSrvReleaseObject(
            core::ptr::addr_of_mut!((*$Buff).Header),
            $IsConsoleLocked,
        )
    };
}

macro_rules! ConsoleInputUnicodeToAnsiChar {
    ($Console:expr, $dChar:expr, $sWChar:expr) => {{
        debug_assert!(($dChar as usize) != ($sWChar as usize));
        WideCharToMultiByte(
            (*$Console).InputCodePage,
            0,
            $sWChar,
            1,
            $dChar,
            1,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        );
    }};
}

macro_rules! ConsoleInputAnsiToUnicodeChar {
    ($Console:expr, $dWChar:expr, $sChar:expr) => {{
        debug_assert!(($dWChar as usize) != ($sChar as usize));
        MultiByteToWideChar(
            (*$Console).InputCodePage,
            0,
            $sChar,
            1,
            $dWChar,
            1,
        );
    }};
}

#[repr(C)]
struct GET_INPUT_INFO {
    CallingThread: PCSR_THREAD,
    HandleEntry: PVOID,
    InputBuffer: PCONSOLE_INPUT_BUFFER,
}

type PGET_INPUT_INFO = *mut GET_INPUT_INFO;

pub(crate) unsafe fn ConioInputEventToAnsi(Console: PCONSOLE, InputEvent: PINPUT_RECORD) {
    if (*InputEvent).EventType == KEY_EVENT as u16 {
        let UnicodeChar = (*InputEvent).Event.KeyEvent.uChar.UnicodeChar;
        (*InputEvent).Event.KeyEvent.uChar.UnicodeChar = 0;
        ConsoleInputUnicodeToAnsiChar!(
            Console,
            core::ptr::addr_of_mut!((*InputEvent).Event.KeyEvent.uChar.AsciiChar),
            core::ptr::addr_of!(UnicodeChar)
        );
    }
}

pub(crate) unsafe fn ConioInputEventToUnicode(Console: PCONSOLE, InputEvent: PINPUT_RECORD) {
    if (*InputEvent).EventType == KEY_EVENT as u16 {
        let AsciiChar = (*InputEvent).Event.KeyEvent.uChar.AsciiChar;
        (*InputEvent).Event.KeyEvent.uChar.AsciiChar = 0;
        ConsoleInputAnsiToUnicodeChar!(
            Console,
            core::ptr::addr_of_mut!((*InputEvent).Event.KeyEvent.uChar.UnicodeChar),
            core::ptr::addr_of!(AsciiChar)
        );
    }
}

pub(crate) unsafe fn PreprocessInput(
    Console: PCONSRV_CONSOLE,
    InputEvent: PINPUT_RECORD,
    mut NumEventsToWrite: ULONG,
) -> ULONG {
    let mut NumEvents = NumEventsToWrite;
    let mut CurrentEvent = InputEvent;

    while NumEvents > 0 {
        if (*CurrentEvent).EventType == KEY_EVENT as u16 && (*CurrentEvent).Event.KeyEvent.bKeyDown != 0 {
            let vk = (*CurrentEvent).Event.KeyEvent.wVirtualKeyCode;
            if ((*Console).PauseFlags & PAUSED_FROM_KEYBOARD as u8) == 0 {
                let cks = (*CurrentEvent).Event.KeyEvent.dwControlKeyState;
                if ((*Console).InputBuffer.Mode & ENABLE_LINE_INPUT as u16) != 0
                    && (vk == VK_PAUSE as u16
                        || (vk == b'S' as u16
                            && (cks & (LEFT_CTRL_PRESSED | RIGHT_CTRL_PRESSED)) != 0
                            && (cks & (LEFT_ALT_PRESSED | RIGHT_ALT_PRESSED)) == 0))
                {
                    ConioPause(Console, PAUSED_FROM_KEYBOARD as u8);

                    RtlMoveMemory(
                        CurrentEvent as PVOID,
                        CurrentEvent.add(1) as PVOID,
                        ((NumEvents - 1) * core::mem::size_of::<INPUT_RECORD>() as u32) as usize,
                    );
                    NumEventsToWrite = NumEventsToWrite.wrapping_sub(1);
                    NumEvents = NumEvents.wrapping_sub(1);
                    continue;
                }
            } else {
                if (vk < VK_SHIFT as u16 || vk > VK_CAPITAL as u16)
                    && vk != VK_LWIN as u16
                    && vk != VK_RWIN as u16
                    && vk != VK_NUMLOCK as u16
                    && vk != VK_SCROLL as u16
                {
                    ConioUnpause(Console, PAUSED_FROM_KEYBOARD as u8);

                    RtlMoveMemory(
                        CurrentEvent as PVOID,
                        CurrentEvent.add(1) as PVOID,
                        ((NumEvents - 1) * core::mem::size_of::<INPUT_RECORD>() as u32) as usize,
                    );
                    NumEventsToWrite = NumEventsToWrite.wrapping_sub(1);
                    NumEvents = NumEvents.wrapping_sub(1);
                    continue;
                }
            }
        }

        CurrentEvent = CurrentEvent.add(1);
        NumEvents = NumEvents.wrapping_sub(1);
    }

    NumEventsToWrite
}

pub(crate) unsafe fn PostprocessInput(Console: PCONSRV_CONSOLE) {
    CsrNotifyWait(
        core::ptr::addr_of_mut!((*Console).ReadWaitQueue),
        0u8,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );
    if IsListEmpty(core::ptr::addr_of_mut!((*Console).ReadWaitQueue)) == 0 {
        CsrDereferenceWait(core::ptr::addr_of_mut!((*Console).ReadWaitQueue));
    }
}

pub(crate) unsafe fn ConioAddInputEvents(
    Console: PCONSRV_CONSOLE,
    InputRecords: PINPUT_RECORD,
    mut NumEventsToWrite: ULONG,
    NumEventsWritten: *mut ULONG,
    AppendToEnd: BOOLEAN,
) -> NTSTATUS {
    if !NumEventsWritten.is_null() {
        *NumEventsWritten = 0;
    }

    NumEventsToWrite = PreprocessInput(Console, InputRecords, NumEventsToWrite);
    if NumEventsToWrite == 0 {
        return STATUS_SUCCESS;
    }

    let Status = ConDrvWriteConsoleInput(
        Console as PCONSOLE,
        core::ptr::addr_of_mut!((*Console).InputBuffer),
        AppendToEnd,
        InputRecords,
        NumEventsToWrite,
        NumEventsWritten,
    );

    if Status == STATUS_SUCCESS {
        PostprocessInput(Console);
    }

    Status
}

// C decl (coninput.c) is plain cdecl (no NTAPI); still-C condrv/frontend
// callers reference `_ConioProcessInputEvent` → extern "C".
#[no_mangle]
pub unsafe extern "C" fn ConioProcessInputEvent(
    Console: PCONSRV_CONSOLE,
    InputEvent: PINPUT_RECORD,
) -> NTSTATUS {
    let mut NumEventsWritten: ULONG = 0;

    if (*InputEvent).EventType == KEY_EVENT as u16 {
        let Down = (*InputEvent).Event.KeyEvent.bKeyDown;
        let VirtualKeyCode = (*InputEvent).Event.KeyEvent.wVirtualKeyCode as u32;
        let ShiftState = (*InputEvent).Event.KeyEvent.dwControlKeyState;

        if (GetConsoleInputBufferMode(Console) & ENABLE_PROCESSED_INPUT as u16) != 0
            && Down != 0
            && (VirtualKeyCode == VK_PAUSE as u32 || VirtualKeyCode == b'C' as u32)
            && (ShiftState & (LEFT_CTRL_PRESSED | RIGHT_CTRL_PRESSED)) != 0
        {
            ConSrvConsoleProcessCtrlEvent(Console, 0, CTRL_C_EVENT);

            if !(*Console).LineBuffer.is_null() && (*Console).LineComplete == 0 {
                (*Console).LinePos = 0;
                (*Console).LineSize = 0;
                (*Console).LineComplete = 1u8;
            }
            return STATUS_SUCCESS;
        }
    }

    ConioAddInputEvents(Console, InputEvent, 1, &mut NumEventsWritten, 1u8)
}

pub(crate) unsafe fn WaitBeforeReading(
    InputInfo: PGET_INPUT_INFO,
    ApiMessage: PCSR_API_MESSAGE,
    WaitFunction: CSR_WAIT_FUNCTION,
    CreateWaitBlock: BOOLEAN,
) -> NTSTATUS {
    if CreateWaitBlock != 0 {
        let Console = (*(*InputInfo).InputBuffer).Header.Console as PCONSRV_CONSOLE;

        let CapturedInputInfo = ConsoleAllocHeap(0, core::mem::size_of::<GET_INPUT_INFO>()) as PGET_INPUT_INFO;
        if CapturedInputInfo.is_null() {
            return STATUS_NO_MEMORY;
        }

        RtlMoveMemory(
            CapturedInputInfo as PVOID,
            InputInfo as PVOID,
            core::mem::size_of::<GET_INPUT_INFO>(),
        );

        if CsrCreateWait(
            core::ptr::addr_of_mut!((*Console).ReadWaitQueue),
            WaitFunction,
            (*InputInfo).CallingThread,
            ApiMessage,
            CapturedInputInfo as PVOID,
        ) == 0
        {
            ConsoleFreeHeap(CapturedInputInfo as PVOID);
            return STATUS_NO_MEMORY;
        }
    }

    STATUS_PENDING
}

unsafe extern "system" fn ReadCharsThread(
    _WaitList: PLIST_ENTRY,
    _WaitThread: PCSR_THREAD,
    WaitApiMessage: PCSR_API_MESSAGE,
    WaitContext: PVOID,
    _WaitArgument1: PVOID,
    WaitArgument2: PVOID,
    WaitFlags: ULONG,
) -> BOOLEAN {
    let InputInfo = WaitContext as PGET_INPUT_INFO;
    let InputHandle = WaitArgument2;

    let Status: NTSTATUS;

    if (WaitFlags & CsrProcessTerminating) != 0 {
        Status = STATUS_THREAD_IS_TERMINATING;
    } else if !InputHandle.is_null() {
        Status = if InputHandle == (*InputInfo).HandleEntry {
            STATUS_ALERTED
        } else {
            STATUS_PENDING
        };
    } else {
        Status = ReadChars(InputInfo, WaitApiMessage, 0u8);
    }

    if Status != STATUS_PENDING {
        (*WaitApiMessage).__bindgen_anon_1.__bindgen_anon_1.Status = Status;
        ConsoleFreeHeap(InputInfo as PVOID);
    }

    if Status == STATUS_PENDING { 0u8 } else { 1u8 }
}

extern "system" {
    pub fn ConDrvReadConsole( Console: PCONSOLE, InputBuffer: PCONSOLE_INPUT_BUFFER, Unicode: BOOLEAN, Buffer: PVOID, ReadControl: PCONSOLE_READCONSOLE_CONTROL, Parameter: PVOID, NumCharsToRead: ULONG, NumCharsRead: *mut ULONG, ) -> NTSTATUS;
}

pub(crate) unsafe fn ReadChars(
    InputInfo: PGET_INPUT_INFO,
    ApiMessage: PCSR_API_MESSAGE,
    CreateWaitBlock: BOOLEAN,
) -> NTSTATUS {
    let ReadConsoleRequest = core::ptr::addr_of_mut!(
        (*(ApiMessage as PCONSOLE_API_MESSAGE)).Data.ReadConsoleRequest
    );
    let InputBuffer = (*InputInfo).InputBuffer;

    let mut ReadControl: CONSOLE_READCONSOLE_CONTROL = core::mem::zeroed();
    let ExeName: _UNICODE_STRING;

    let Buffer: PVOID;
    let mut NrCharactersRead: ULONG = 0;
    let CharSize = if (*ReadConsoleRequest).Unicode != 0 {
        core::mem::size_of::<u16>() as ULONG
    } else {
        core::mem::size_of::<i8>() as ULONG
    };

    if (*ReadConsoleRequest).InitialNumBytes == 0
        && (*ReadConsoleRequest).ExeLength <= core::mem::size_of_val(&(*ReadConsoleRequest).StaticBuffer) as u16
    {
        ExeName = _UNICODE_STRING {
            Length: (*ReadConsoleRequest).ExeLength,
            MaximumLength: (*ReadConsoleRequest).ExeLength,
            Buffer: (*ReadConsoleRequest).StaticBuffer.as_mut_ptr() as *mut u16,
        };
    } else {
        ExeName = _UNICODE_STRING {
            Length: 0,
            MaximumLength: 0,
            Buffer: core::ptr::null_mut(),
        };
    }

    ReadControl.nLength = core::mem::size_of::<CONSOLE_READCONSOLE_CONTROL>() as ULONG;
    ReadControl.nInitialChars = (*ReadConsoleRequest).InitialNumBytes / CharSize;
    ReadControl.dwCtrlWakeupMask = (*ReadConsoleRequest).CtrlWakeupMask;
    ReadControl.dwControlKeyState = (*ReadConsoleRequest).ControlKeyState;

    if (*ReadConsoleRequest).CaptureBufferSize <= core::mem::size_of_val(&(*ReadConsoleRequest).StaticBuffer) as ULONG {
        Buffer = (*ReadConsoleRequest).StaticBuffer.as_mut_ptr() as PVOID;
    } else {
        Buffer = (*ReadConsoleRequest).Buffer;
    }

    let Status = ConDrvReadConsole(
        (*InputBuffer).Header.Console,
        InputBuffer,
        (*ReadConsoleRequest).Unicode,
        Buffer,
        &mut ReadControl as *mut CONSOLE_READCONSOLE_CONTROL,
        core::ptr::addr_of!(ExeName) as PVOID,
        (*ReadConsoleRequest).NumBytes / CharSize,
        &mut NrCharactersRead,
    );

    if Status == STATUS_PENDING {
        WaitBeforeReading(InputInfo, ApiMessage, Some(core::mem::transmute(ReadCharsThread as *const ())), CreateWaitBlock)
    } else {
        (*ReadConsoleRequest).NumBytes = NrCharactersRead * CharSize;
        (*ReadConsoleRequest).ControlKeyState = ReadControl.dwControlKeyState;
        Status
    }
}

unsafe extern "system" fn ReadInputBufferThread(
    _WaitList: PLIST_ENTRY,
    _WaitThread: PCSR_THREAD,
    WaitApiMessage: PCSR_API_MESSAGE,
    WaitContext: PVOID,
    _WaitArgument1: PVOID,
    WaitArgument2: PVOID,
    WaitFlags: ULONG,
) -> BOOLEAN {
    let InputInfo = WaitContext as PGET_INPUT_INFO;
    let InputHandle = WaitArgument2;

    let Status: NTSTATUS;

    if (WaitFlags & CsrProcessTerminating) != 0 {
        Status = STATUS_THREAD_IS_TERMINATING;
    } else if !InputHandle.is_null() {
        Status = if InputHandle == (*InputInfo).HandleEntry {
            STATUS_ALERTED
        } else {
            STATUS_PENDING
        };
    } else {
        Status = ReadInputBuffer(InputInfo, WaitApiMessage, 0u8);
    }

    if Status != STATUS_PENDING {
        (*WaitApiMessage).__bindgen_anon_1.__bindgen_anon_1.Status = Status;
        ConsoleFreeHeap(InputInfo as PVOID);
    }

    if Status == STATUS_PENDING { 0u8 } else { 1u8 }
}

extern "system" {
    pub fn ConDrvGetConsoleInput( Console: PCONSOLE, InputBuffer: PCONSOLE_INPUT_BUFFER, KeepEvents: BOOLEAN, WaitForMoreEvents: BOOLEAN, InputRecord: PINPUT_RECORD, NumEventsToRead: ULONG, NumEventsRead: *mut ULONG, ) -> NTSTATUS;
}

pub(crate) unsafe fn ReadInputBuffer(
    InputInfo: PGET_INPUT_INFO,
    ApiMessage: PCSR_API_MESSAGE,
    CreateWaitBlock: BOOLEAN,
) -> NTSTATUS {
    let GetInputRequest = core::ptr::addr_of_mut!(
        (*(ApiMessage as PCONSOLE_API_MESSAGE)).Data.GetInputRequest
    );
    let InputBuffer = (*InputInfo).InputBuffer;
    let mut NumEventsRead: ULONG = 0;

    let InputRecord: PINPUT_RECORD;

    if (*GetInputRequest).NumRecords <= (core::mem::size_of_val(&(*GetInputRequest).RecordStaticBuffer) / core::mem::size_of::<INPUT_RECORD>()) as ULONG {
        InputRecord = (*GetInputRequest).RecordStaticBuffer.as_mut_ptr();
    } else {
        InputRecord = (*GetInputRequest).RecordBufPtr;
    }

    let Status = ConDrvGetConsoleInput(
        (*InputBuffer).Header.Console,
        InputBuffer,
        if ((*GetInputRequest).Flags as u32 & CONSOLE_READ_NOREMOVE) != 0 { 1u8 } else { 0u8 },
        if ((*GetInputRequest).Flags as u32 & CONSOLE_READ_NOWAIT) == 0 { 1u8 } else { 0u8 },
        InputRecord,
        (*GetInputRequest).NumRecords,
        &mut NumEventsRead,
    );

    if Status == STATUS_PENDING {
        WaitBeforeReading(InputInfo, ApiMessage, Some(core::mem::transmute(ReadInputBufferThread as *const ())), CreateWaitBlock)
    } else {
        (*GetInputRequest).NumRecords = NumEventsRead;

        if Status >= 0 {
            if (*GetInputRequest).Unicode == 0 {
                for i in 0..NumEventsRead {
                    ConioInputEventToAnsi((*InputBuffer).Header.Console, InputRecord.add(i as usize));
                }
            }
        }

        Status
    }
}

#[no_mangle]
pub unsafe extern "system" fn SrvReadConsole(
    ApiMessage: PCSR_API_MESSAGE,
    ReplyCode: PCSR_REPLY_CODE,
) -> NTSTATUS {
    let ProcessData = ConsoleGetPerProcessData((*CsrGetClientThread()).Process);
    let mut Console: PCONSRV_CONSOLE = core::ptr::null_mut();
    let Status = ConSrvGetConsole(ProcessData, core::ptr::addr_of_mut!(Console), 1u8);
    if !NT_SUCCESS(Status) {
        return Status;
    }

    let Status = SrvReadConsoleImpl(ProcessData, Console, ApiMessage, ReplyCode);

    ConSrvReleaseConsole(Console, 1u8);
    Status
}

unsafe fn SrvReadConsoleImpl(
    ProcessData: PCONSOLE_PROCESS_DATA,
    Console: PCONSRV_CONSOLE,
    ApiMessage: PCSR_API_MESSAGE,
    ReplyCode: PCSR_REPLY_CODE,
) -> NTSTATUS {
    let _ = Console;
    let ReadConsoleRequest = core::ptr::addr_of_mut!(
        (*(ApiMessage as PCONSOLE_API_MESSAGE)).Data.ReadConsoleRequest
    );

    if (*ReadConsoleRequest).CaptureBufferSize > core::mem::size_of_val(&(*ReadConsoleRequest).StaticBuffer) as ULONG {
        if CsrValidateMessageBuffer(
            ApiMessage,
            core::ptr::addr_of_mut!((*ReadConsoleRequest).Buffer) as *mut PVOID,
            (*ReadConsoleRequest).CaptureBufferSize,
            core::mem::size_of::<u8>() as ULONG,
        ) == 0
        {
            return STATUS_INVALID_PARAMETER;
        }
    }

    if (*ReadConsoleRequest).InitialNumBytes > (*ReadConsoleRequest).NumBytes {
        return STATUS_INVALID_PARAMETER;
    }

    let mut HandleEntry: PVOID = core::ptr::null_mut();
    let mut InputBuffer: PCONSOLE_INPUT_BUFFER = core::ptr::null_mut();
    let Status = ConSrvGetInputBufferAndHandleEntry!(
        ProcessData,
        (*ReadConsoleRequest).InputHandle,
        &mut InputBuffer,
        &mut HandleEntry,
        GENERIC_READ,
        1u8
    );
    if Status < 0 {
        return Status;
    }

    let mut InputInfo: GET_INPUT_INFO = core::mem::zeroed();
    InputInfo.CallingThread = CsrGetClientThread();
    InputInfo.HandleEntry = HandleEntry;
    InputInfo.InputBuffer = InputBuffer;

    let Status = ReadChars(&mut InputInfo, ApiMessage, 1u8);

    ConSrvReleaseInputBuffer!(InputBuffer, 1u8);

    if Status == STATUS_PENDING {
        *ReplyCode = CsrReplyPending;
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn SrvGetConsoleInput(
    ApiMessage: PCSR_API_MESSAGE,
    ReplyCode: PCSR_REPLY_CODE,
) -> NTSTATUS {
    let ProcessData = ConsoleGetPerProcessData((*CsrGetClientThread()).Process);
    let mut Console: PCONSRV_CONSOLE = core::ptr::null_mut();
    let Status = ConSrvGetConsole(ProcessData, core::ptr::addr_of_mut!(Console), 1u8);
    if !NT_SUCCESS(Status) {
        return Status;
    }

    let Status = SrvGetConsoleInputImpl(ProcessData, Console, ApiMessage, ReplyCode);

    ConSrvReleaseConsole(Console, 1u8);
    Status
}

unsafe fn SrvGetConsoleInputImpl(
    ProcessData: PCONSOLE_PROCESS_DATA,
    Console: PCONSRV_CONSOLE,
    ApiMessage: PCSR_API_MESSAGE,
    ReplyCode: PCSR_REPLY_CODE,
) -> NTSTATUS {
    let _ = Console;
    let GetInputRequest = core::ptr::addr_of_mut!(
        (*(ApiMessage as PCONSOLE_API_MESSAGE)).Data.GetInputRequest
    );

    if ((*GetInputRequest).Flags & !(CONSOLE_READ_NOREMOVE | CONSOLE_READ_NOWAIT) as u16) != 0 {
        return STATUS_INVALID_PARAMETER;
    }

    if (*GetInputRequest).NumRecords > (core::mem::size_of_val(&(*GetInputRequest).RecordStaticBuffer) / core::mem::size_of::<INPUT_RECORD>()) as ULONG {
        if CsrValidateMessageBuffer(
            ApiMessage,
            core::ptr::addr_of_mut!((*GetInputRequest).RecordBufPtr) as *mut PVOID,
            (*GetInputRequest).NumRecords,
            core::mem::size_of::<INPUT_RECORD>() as ULONG,
        ) == 0
        {
            return STATUS_INVALID_PARAMETER;
        }
    }

    let mut HandleEntry: PVOID = core::ptr::null_mut();
    let mut InputBuffer: PCONSOLE_INPUT_BUFFER = core::ptr::null_mut();
    let Status = ConSrvGetInputBufferAndHandleEntry!(
        ProcessData,
        (*GetInputRequest).InputHandle,
        &mut InputBuffer,
        &mut HandleEntry,
        GENERIC_READ,
        1u8
    );
    if Status < 0 {
        return Status;
    }

    let mut InputInfo: GET_INPUT_INFO = core::mem::zeroed();
    InputInfo.CallingThread = CsrGetClientThread();
    InputInfo.HandleEntry = HandleEntry;
    InputInfo.InputBuffer = InputBuffer;

    let Status = ReadInputBuffer(&mut InputInfo, ApiMessage, 1u8);

    ConSrvReleaseInputBuffer!(InputBuffer, 1u8);

    if Status == STATUS_PENDING {
        *ReplyCode = CsrReplyPending;
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn SrvWriteConsoleInput(
    ApiMessage: PCSR_API_MESSAGE,
    ReplyCode: PCSR_REPLY_CODE,
) -> NTSTATUS {
    let ProcessData = ConsoleGetPerProcessData((*CsrGetClientThread()).Process);
    let mut Console: PCONSRV_CONSOLE = core::ptr::null_mut();
    let Status = ConSrvGetConsole(ProcessData, core::ptr::addr_of_mut!(Console), 1u8);
    if !NT_SUCCESS(Status) {
        return Status;
    }

    let Status = SrvWriteConsoleInputImpl(ProcessData, Console, ApiMessage, ReplyCode);

    ConSrvReleaseConsole(Console, 1u8);
    Status
}

unsafe fn SrvWriteConsoleInputImpl(
    ProcessData: PCONSOLE_PROCESS_DATA,
    Console: PCONSRV_CONSOLE,
    ApiMessage: PCSR_API_MESSAGE,
    _ReplyCode: PCSR_REPLY_CODE,
) -> NTSTATUS {
    let WriteInputRequest = core::ptr::addr_of_mut!(
        (*(ApiMessage as PCONSOLE_API_MESSAGE)).Data.WriteInputRequest
    );

    let InputRecord: PINPUT_RECORD;

    if (*WriteInputRequest).NumRecords <= (core::mem::size_of_val(&(*WriteInputRequest).RecordStaticBuffer) / core::mem::size_of::<INPUT_RECORD>()) as ULONG {
        InputRecord = (*WriteInputRequest).RecordStaticBuffer.as_mut_ptr();
    } else {
        if CsrValidateMessageBuffer(
            ApiMessage,
            core::ptr::addr_of_mut!((*WriteInputRequest).RecordBufPtr) as *mut PVOID,
            (*WriteInputRequest).NumRecords,
            core::mem::size_of::<INPUT_RECORD>() as ULONG,
        ) == 0
        {
            return STATUS_INVALID_PARAMETER;
        }

        InputRecord = (*WriteInputRequest).RecordBufPtr;
    }

    let mut InputBuffer: PCONSOLE_INPUT_BUFFER = core::ptr::null_mut();
    let mut Status = ConSrvGetInputBuffer!(
        ProcessData,
        (*WriteInputRequest).InputHandle,
        &mut InputBuffer,
        GENERIC_WRITE,
        1u8
    );
    if Status < 0 {
        (*WriteInputRequest).NumRecords = 0;
        return Status;
    }

    if (*WriteInputRequest).Unicode == 0 {
        for i in 0..(*WriteInputRequest).NumRecords {
            ConioInputEventToUnicode(Console as PCONSOLE, InputRecord.add(i as usize));
        }
    }

    let mut NumEventsWritten: ULONG = 0;
    Status = ConioAddInputEvents(
        Console,
        InputRecord,
        (*WriteInputRequest).NumRecords,
        &mut NumEventsWritten,
        (*WriteInputRequest).AppendToEnd,
    );

    (*WriteInputRequest).NumRecords = NumEventsWritten;

    ConSrvReleaseInputBuffer!(InputBuffer, 1u8);
    Status
}

extern "system" {
    pub fn ConDrvFlushConsoleInputBuffer( Console: PCONSOLE, InputBuffer: PCONSOLE_INPUT_BUFFER, ) -> NTSTATUS;
}

#[no_mangle]
pub unsafe extern "system" fn SrvFlushConsoleInputBuffer(
    ApiMessage: PCSR_API_MESSAGE,
    ReplyCode: PCSR_REPLY_CODE,
) -> NTSTATUS {
    let ProcessData = ConsoleGetPerProcessData((*CsrGetClientThread()).Process);
    let mut Console: PCONSRV_CONSOLE = core::ptr::null_mut();
    let Status = ConSrvGetConsole(ProcessData, core::ptr::addr_of_mut!(Console), 1u8);
    if !NT_SUCCESS(Status) {
        return Status;
    }

    let Status = SrvFlushConsoleInputBufferImpl(ProcessData, Console, ApiMessage, ReplyCode);

    ConSrvReleaseConsole(Console, 1u8);
    Status
}

unsafe fn SrvFlushConsoleInputBufferImpl(
    ProcessData: PCONSOLE_PROCESS_DATA,
    Console: PCONSRV_CONSOLE,
    ApiMessage: PCSR_API_MESSAGE,
    _ReplyCode: PCSR_REPLY_CODE,
) -> NTSTATUS {
    let FlushInputBufferRequest = core::ptr::addr_of_mut!(
        (*(ApiMessage as PCONSOLE_API_MESSAGE)).Data.FlushInputBufferRequest
    );

    let mut InputBuffer: PCONSOLE_INPUT_BUFFER = core::ptr::null_mut();
    let Status = ConSrvGetInputBuffer!(
        ProcessData,
        (*FlushInputBufferRequest).InputHandle,
        &mut InputBuffer,
        GENERIC_WRITE,
        1u8
    );
    if Status < 0 {
        return Status;
    }

    let Status = ConDrvFlushConsoleInputBuffer(Console as PCONSOLE, InputBuffer);

    ConSrvReleaseInputBuffer!(InputBuffer, 1u8);
    Status
}

extern "system" {
    pub fn ConDrvGetConsoleNumberOfInputEvents( Console: PCONSOLE, InputBuffer: PCONSOLE_INPUT_BUFFER, NumberOfEvents: *mut ULONG, ) -> NTSTATUS;
}

#[no_mangle]
pub unsafe extern "system" fn SrvGetConsoleNumberOfInputEvents(
    ApiMessage: PCSR_API_MESSAGE,
    ReplyCode: PCSR_REPLY_CODE,
) -> NTSTATUS {
    let ProcessData = ConsoleGetPerProcessData((*CsrGetClientThread()).Process);
    let mut Console: PCONSRV_CONSOLE = core::ptr::null_mut();
    let Status = ConSrvGetConsole(ProcessData, core::ptr::addr_of_mut!(Console), 1u8);
    if !NT_SUCCESS(Status) {
        return Status;
    }

    let Status = SrvGetConsoleNumberOfInputEventsImpl(ProcessData, Console, ApiMessage, ReplyCode);

    ConSrvReleaseConsole(Console, 1u8);
    Status
}

unsafe fn SrvGetConsoleNumberOfInputEventsImpl(
    ProcessData: PCONSOLE_PROCESS_DATA,
    Console: PCONSRV_CONSOLE,
    ApiMessage: PCSR_API_MESSAGE,
    _ReplyCode: PCSR_REPLY_CODE,
) -> NTSTATUS {
    let GetNumInputEventsRequest = core::ptr::addr_of_mut!(
        (*(ApiMessage as PCONSOLE_API_MESSAGE)).Data.GetNumInputEventsRequest
    );

    let mut InputBuffer: PCONSOLE_INPUT_BUFFER = core::ptr::null_mut();
    let Status = ConSrvGetInputBuffer!(
        ProcessData,
        (*GetNumInputEventsRequest).InputHandle,
        &mut InputBuffer,
        GENERIC_READ,
        1u8
    );
    if Status < 0 {
        return Status;
    }

    let Status = ConDrvGetConsoleNumberOfInputEvents(
        Console as PCONSOLE,
        InputBuffer,
        core::ptr::addr_of_mut!((*GetNumInputEventsRequest).NumberOfEvents),
    );

    ConSrvReleaseInputBuffer!(InputBuffer, 1u8);
    Status
}