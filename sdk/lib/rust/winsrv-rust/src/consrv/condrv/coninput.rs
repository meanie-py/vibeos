use super::super::term::*;
use winsrv_sys::nt::*;
use winsrv_sys::winsrv::*;
use winsrv_sys::consts::*;
use crate::consrv::{ConsoleAllocHeap, ConsoleFreeHeap};

extern "system" {
    pub fn CONTAINING_RECORD(address: PVOID, struct_type: PVOID, field_offset: usize) -> PVOID;
    pub fn InitializeObjectAttributes( p: *mut _OBJECT_ATTRIBUTES, n: PVOID, a: ULONG, r: PVOID, s: PVOID, );
    pub fn _InterlockedIncrement(addend: *mut LONG) -> LONG;
    pub fn _InterlockedDecrement(addend: *mut LONG) -> LONG;
    pub fn _InterlockedExchange(target: *mut LONG, value: LONG) -> LONG;
}
// TermReadStream supplied by super::super::term::*

pub(crate) unsafe fn AddInputEvents(
    Console: PCONSOLE,
    mut InputRecords: PINPUT_RECORD,
    NumEventsToWrite: ULONG,
    NumEventsWritten: *mut ULONG,
    AppendToEnd: BOOLEAN,
) -> NTSTATUS {
    let mut Status = STATUS_SUCCESS;
    let mut i: ULONG = 0;
    let mut SetWaitEvent = 0u8;

    if !NumEventsWritten.is_null() {
        *NumEventsWritten = 0;
    }

    if NumEventsToWrite == 1 && IsListEmpty(core::ptr::addr_of_mut!((*Console).InputBuffer.InputEvents)) == 0 {
        let InputRecord = InputRecords;
        let LastInputRecord: PINPUT_RECORD;
        let ConInRec: *mut ConsoleInput;

        if AppendToEnd != 0 {
            let offset = core::mem::offset_of!(ConsoleInput, ListEntry);
            ConInRec = CONTAINING_RECORD(
                (*Console).InputBuffer.InputEvents.Blink as PVOID,
                core::ptr::null_mut(),
                offset,
            ) as *mut ConsoleInput;
        } else {
            let offset = core::mem::offset_of!(ConsoleInput, ListEntry);
            ConInRec = CONTAINING_RECORD(
                (*Console).InputBuffer.InputEvents.Flink as PVOID,
                core::ptr::null_mut(),
                offset,
            ) as *mut ConsoleInput;
        }
        LastInputRecord = core::ptr::addr_of_mut!((*ConInRec).InputEvent);

        if (*InputRecord).EventType == (KEY_EVENT as u16)
            && (*InputRecord).Event.KeyEvent.bKeyDown != 0
        {
            if (*LastInputRecord).EventType == (KEY_EVENT as u16)
                && (*LastInputRecord).Event.KeyEvent.bKeyDown != 0
                && (*LastInputRecord).Event.KeyEvent.wVirtualScanCode == (*InputRecord).Event.KeyEvent.wVirtualScanCode
                && (*LastInputRecord).Event.KeyEvent.uChar.UnicodeChar == (*InputRecord).Event.KeyEvent.uChar.UnicodeChar
                && (*LastInputRecord).Event.KeyEvent.dwControlKeyState == (*InputRecord).Event.KeyEvent.dwControlKeyState
            {
                (*LastInputRecord).Event.KeyEvent.wRepeatCount =
                    (*LastInputRecord).Event.KeyEvent.wRepeatCount.wrapping_add(
                        (*InputRecord).Event.KeyEvent.wRepeatCount,
                    );

                i = 1;
                Status = STATUS_SUCCESS;
            }
        } else if (*InputRecord).EventType == (MOUSE_EVENT as u16)
            && (*InputRecord).Event.MouseEvent.dwEventFlags == MOUSE_MOVED
        {
            if (*LastInputRecord).EventType == (MOUSE_EVENT as u16)
                && (*LastInputRecord).Event.MouseEvent.dwEventFlags == MOUSE_MOVED
            {
                (*LastInputRecord).Event.MouseEvent.dwMousePosition.X =
                    (*InputRecord).Event.MouseEvent.dwMousePosition.X;
                (*LastInputRecord).Event.MouseEvent.dwMousePosition.Y =
                    (*InputRecord).Event.MouseEvent.dwMousePosition.Y;

                i = 1;
                Status = STATUS_SUCCESS;
            }
        }
    }

    if i == 1 && Status == STATUS_SUCCESS {
        if !NumEventsWritten.is_null() {
            *NumEventsWritten = i;
        }
        return Status;
    }

    if AppendToEnd == 0 {
        InputRecords = InputRecords.add((NumEventsToWrite - 1) as usize);
    }

    if IsListEmpty(core::ptr::addr_of_mut!((*Console).InputBuffer.InputEvents)) != 0 {
        SetWaitEvent = 1;
    }

    i = 0;
    while i < NumEventsToWrite && NT_SUCCESS(Status) {
        let InputRecord: PINPUT_RECORD;
        let ConInRec: *mut ConsoleInput;

        if AppendToEnd != 0 {
            InputRecord = InputRecords;
            InputRecords = InputRecords.add(1);
        } else {
            InputRecord = InputRecords;
            InputRecords = InputRecords.sub(1);
        }

        ConInRec = ConsoleAllocHeap(0, core::mem::size_of::<ConsoleInput>()) as *mut ConsoleInput;
        if ConInRec.is_null() {
            Status = STATUS_INSUFFICIENT_RESOURCES;
            i += 1;
            continue;
        }

        (*ConInRec).InputEvent = *InputRecord;

        if AppendToEnd != 0 {
            InsertTailList(
                core::ptr::addr_of_mut!((*Console).InputBuffer.InputEvents),
                core::ptr::addr_of_mut!((*ConInRec).ListEntry),
            );
        } else {
            InsertHeadList(
                core::ptr::addr_of_mut!((*Console).InputBuffer.InputEvents),
                core::ptr::addr_of_mut!((*ConInRec).ListEntry),
            );
        }
        _InterlockedIncrement(core::ptr::addr_of_mut!((*Console).InputBuffer.NumberOfEvents) as *mut LONG);

        Status = STATUS_SUCCESS;
        i += 1;
    }

    if SetWaitEvent != 0 {
        NtSetEvent((*Console).InputBuffer.ActiveEvent, core::ptr::null_mut());
    }

    if !NumEventsWritten.is_null() {
        *NumEventsWritten = i;
    }

    Status
}

pub(crate) unsafe fn PurgeInputBuffer(InputBuffer: PCONSOLE_INPUT_BUFFER) {
    let mut CurrentEntry: PLIST_ENTRY;
    let mut Event: *mut ConsoleInput;

    _InterlockedExchange(core::ptr::addr_of_mut!((*InputBuffer).NumberOfEvents) as *mut LONG, 0);
    while IsListEmpty(core::ptr::addr_of_mut!((*InputBuffer).InputEvents)) == 0 {
        CurrentEntry = RemoveHeadList(core::ptr::addr_of_mut!((*InputBuffer).InputEvents));
        let offset = core::mem::offset_of!(ConsoleInput, ListEntry);
        Event = CONTAINING_RECORD(CurrentEntry as PVOID, core::ptr::null_mut(), offset) as *mut ConsoleInput;
        ConsoleFreeHeap(Event as PVOID);
    }
}

#[no_mangle]
pub unsafe extern "system" fn ConDrvInitInputBuffer(
    Console: PCONSOLE,
    InputBufferSize: ULONG,
) -> NTSTATUS {
    let Status: NTSTATUS;
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();

    ConSrvInitObject(
        core::ptr::addr_of_mut!((*Console).InputBuffer.Header),
        INPUT_BUFFER,
        Console,
    );

    InitializeObjectAttributes(
        &mut ObjectAttributes,
        core::ptr::null_mut(),
        OBJ_INHERIT,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );

    Status = NtCreateEvent(
        core::ptr::addr_of_mut!((*Console).InputBuffer.ActiveEvent),
        EVENT_ALL_ACCESS,
        &mut ObjectAttributes,
        _EVENT_TYPE_NotificationEvent,
        0u8,
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }

    (*Console).InputBuffer.InputBufferSize = InputBufferSize;
    (*Console).InputBuffer.NumberOfEvents = 0;
    InitializeListHead(core::ptr::addr_of_mut!((*Console).InputBuffer.InputEvents));
    (*Console).InputBuffer.Mode = (ENABLE_PROCESSED_INPUT | ENABLE_LINE_INPUT | ENABLE_ECHO_INPUT | ENABLE_MOUSE_INPUT) as u16;

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn ConDrvDeinitInputBuffer(Console: PCONSOLE) {
    PurgeInputBuffer(core::ptr::addr_of_mut!((*Console).InputBuffer));
    NtClose((*Console).InputBuffer.ActiveEvent);
}

#[no_mangle]
pub unsafe extern "system" fn ConDrvReadConsole(
    Console: PCONSOLE,
    InputBuffer: PCONSOLE_INPUT_BUFFER,
    Unicode: BOOLEAN,
    Buffer: PVOID,
    ReadControl: PCONSOLE_READCONSOLE_CONTROL,
    Parameter: PVOID,
    NumCharsToRead: ULONG,
    NumCharsRead: *mut ULONG,
) -> NTSTATUS {
    if Console.is_null() || InputBuffer.is_null() || ReadControl.is_null()
        || (*ReadControl).nLength != core::mem::size_of::<CONSOLE_READCONSOLE_CONTROL>() as ULONG
    {
        return STATUS_INVALID_PARAMETER;
    }

    TermReadStream(
        Console as PCONSRV_CONSOLE,
        Unicode,
        Buffer,
        ReadControl,
        Parameter,
        NumCharsToRead,
        NumCharsRead,
    )
}

#[no_mangle]
pub unsafe extern "system" fn ConDrvGetConsoleInput(
    Console: PCONSOLE,
    InputBuffer: PCONSOLE_INPUT_BUFFER,
    KeepEvents: BOOLEAN,
    WaitForMoreEvents: BOOLEAN,
    mut InputRecord: PINPUT_RECORD,
    NumEventsToRead: ULONG,
    NumEventsRead: *mut ULONG,
) -> NTSTATUS {
    let mut CurrentInput: PLIST_ENTRY;
    let mut Input: *mut ConsoleInput;
    let mut i: ULONG = 0;

    if Console.is_null() || InputBuffer.is_null() {
        return STATUS_INVALID_PARAMETER;
    }

    if !NumEventsRead.is_null() {
        *NumEventsRead = 0;
    }

    if IsListEmpty(core::ptr::addr_of_mut!((*InputBuffer).InputEvents)) != 0 {
        return if WaitForMoreEvents != 0 { STATUS_PENDING } else { STATUS_SUCCESS };
    }

    CurrentInput = (*InputBuffer).InputEvents.Flink;
    i = 0;
    while CurrentInput != core::ptr::addr_of_mut!((*InputBuffer).InputEvents) as PLIST_ENTRY
        && i < NumEventsToRead
    {
        let offset = core::mem::offset_of!(ConsoleInput, ListEntry);
        Input = CONTAINING_RECORD(CurrentInput as PVOID, core::ptr::null_mut(), offset) as *mut ConsoleInput;

        *InputRecord = (*Input).InputEvent;

        InputRecord = InputRecord.add(1);
        i += 1;
        CurrentInput = (*CurrentInput).Flink;

        if KeepEvents == 0 {
            _InterlockedDecrement(core::ptr::addr_of_mut!((*InputBuffer).NumberOfEvents) as *mut LONG);
            RemoveEntryList(core::ptr::addr_of_mut!((*Input).ListEntry));
            ConsoleFreeHeap(Input as PVOID);
        }
    }

    if !NumEventsRead.is_null() {
        *NumEventsRead = i;
    }

    if IsListEmpty(core::ptr::addr_of_mut!((*InputBuffer).InputEvents)) != 0 {
        NtClearEvent((*InputBuffer).ActiveEvent);
    }

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn ConDrvWriteConsoleInput(
    Console: PCONSOLE,
    InputBuffer: PCONSOLE_INPUT_BUFFER,
    AppendToEnd: BOOLEAN,
    InputRecord: PINPUT_RECORD,
    NumEventsToWrite: ULONG,
    NumEventsWritten: *mut ULONG,
) -> NTSTATUS {
    if Console.is_null() || InputBuffer.is_null() {
        return STATUS_INVALID_PARAMETER;
    }

    if !NumEventsWritten.is_null() {
        *NumEventsWritten = 0;
    }

    AddInputEvents(
        Console,
        InputRecord,
        NumEventsToWrite,
        NumEventsWritten,
        AppendToEnd,
    )
}

#[no_mangle]
pub unsafe extern "system" fn ConDrvFlushConsoleInputBuffer(
    Console: PCONSOLE,
    InputBuffer: PCONSOLE_INPUT_BUFFER,
) -> NTSTATUS {
    if Console.is_null() || InputBuffer.is_null() {
        return STATUS_INVALID_PARAMETER;
    }

    PurgeInputBuffer(InputBuffer);
    NtClearEvent((*InputBuffer).ActiveEvent);

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn ConDrvGetConsoleNumberOfInputEvents(
    Console: PCONSOLE,
    InputBuffer: PCONSOLE_INPUT_BUFFER,
    NumberOfEvents: *mut ULONG,
) -> NTSTATUS {
    if Console.is_null() || InputBuffer.is_null() || NumberOfEvents.is_null() {
        return STATUS_INVALID_PARAMETER;
    }

    *NumberOfEvents = (*InputBuffer).NumberOfEvents;
    STATUS_SUCCESS
}