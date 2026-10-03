use winsrv_sys::nt::*;
use super::GetType;
use super::popup::PPOPUP_WINDOW;
use super::popup::CreatePopupWindow;
use super::{ConsoleAllocHeap, ConsoleFreeHeap};
use winsrv_sys::winsrv::*;
use winsrv_sys::consts::*;
// Disambiguate winsrv_sys::winsrv:: (bindgen) vs sibling Rust impl.
use super::console::ConSrvGetConsole;
use super::console::ConSrvReleaseConsole;


#[repr(C)]
struct HISTORY_BUFFER {
    ListEntry: LIST_ENTRY,
    Position: ULONG,
    MaxEntries: ULONG,
    NumEntries: ULONG,
    ExeName: _UNICODE_STRING,
    Entries: PUNICODE_STRING,
}

type PHISTORY_BUFFER = *mut HISTORY_BUFFER;

// condrv flip: alias.rs defines these plain-cdecl (C: BOOLEAN, no NTAPI).
extern "C" {
    pub fn ConvertInputAnsiToUnicode( Console: PCONSRV_CONSOLE, Source: PVOID, SourceLength: USHORT, Target: *mut PWCHAR, TargetLength: PUSHORT, ) -> BOOLEAN;
    pub fn ConvertInputUnicodeToAnsi( Console: PCONSRV_CONSOLE, Source: PVOID, SourceLength: USHORT, Target: PCHAR, TargetLength: USHORT, ) -> BOOLEAN;
}

pub(crate) unsafe fn HistoryCurrentBuffer(
    Console: PCONSRV_CONSOLE,
    ExeName: PUNICODE_STRING,
) -> PHISTORY_BUFFER {
    let mut Entry = (*Console).HistoryBuffers.Flink;
    while Entry != core::ptr::addr_of_mut!((*Console).HistoryBuffers) as PLIST_ENTRY {
        let Hist = ((Entry as usize) - core::mem::offset_of!(HISTORY_BUFFER, ListEntry)) as PHISTORY_BUFFER;
        if RtlEqualUnicodeString(ExeName, core::ptr::addr_of_mut!((*Hist).ExeName), 0u8) != 0 {
            return Hist;
        }
        Entry = (*Entry).Flink;
    }

    if (*Console).NumberOfHistoryBuffers < (*Console).MaxNumberOfHistoryBuffers {
        let Hist = ConsoleAllocHeap(
            0,
            core::mem::size_of::<HISTORY_BUFFER>() + (*ExeName).Length as usize,
        ) as PHISTORY_BUFFER;
        if Hist.is_null() {
            return core::ptr::null_mut();
        }
        (*Hist).MaxEntries = (*Console).HistoryBufferSize;
        (*Hist).NumEntries = 0;
        (*Hist).Entries = ConsoleAllocHeap(
            0,
            (*Hist).MaxEntries as usize * core::mem::size_of::<_UNICODE_STRING>(),
        ) as PUNICODE_STRING;
        if (*Hist).Entries.is_null() {
            ConsoleFreeHeap(Hist as PVOID);
            return core::ptr::null_mut();
        }
        (*Hist).ExeName.Length = (*ExeName).Length;
        (*Hist).ExeName.MaximumLength = (*ExeName).Length;
        (*Hist).ExeName.Buffer = (Hist.add(1)) as PWCHAR;
        RtlCopyMemory(
            (*Hist).ExeName.Buffer as *mut core::ffi::c_void,
            (*ExeName).Buffer as *const core::ffi::c_void,
            (*ExeName).Length as usize,
        );
        InsertHeadList(
            core::ptr::addr_of_mut!((*Console).HistoryBuffers) as PLIST_ENTRY,
            core::ptr::addr_of_mut!((*Hist).ListEntry) as PLIST_ENTRY,
        );
        (*Console).NumberOfHistoryBuffers += 1;
        return Hist;
    } else {
        return core::ptr::null_mut();
    }
}

pub(crate) unsafe fn HistoryFindBuffer(
    Console: PCONSRV_CONSOLE,
    ExeName: PVOID,
    ExeLength: USHORT,
    UnicodeExe: BOOLEAN,
) -> PHISTORY_BUFFER {
    let mut ExeNameU = core::mem::zeroed::<_UNICODE_STRING>();

    if ExeName.is_null() {
        return core::ptr::null_mut();
    }

    if UnicodeExe != 0 {
        ExeNameU.Buffer = ExeName as PWCHAR;
        ExeNameU.MaximumLength = ExeLength;
    } else {
        if ConvertInputAnsiToUnicode(
            Console,
            ExeName,
            ExeLength,
            core::ptr::addr_of_mut!(ExeNameU.Buffer),
            core::ptr::addr_of_mut!(ExeNameU.MaximumLength),
        ) == 0
        {
            return core::ptr::null_mut();
        }
    }
    ExeNameU.Length = ExeNameU.MaximumLength;

    let mut Entry = (*Console).HistoryBuffers.Flink;
    while Entry != core::ptr::addr_of_mut!((*Console).HistoryBuffers) as PLIST_ENTRY {
        let Hist = ((Entry as usize) - core::mem::offset_of!(HISTORY_BUFFER, ListEntry)) as PHISTORY_BUFFER;

        if RtlPrefixUnicodeString(
            core::ptr::addr_of_mut!(ExeNameU),
            core::ptr::addr_of_mut!((*Hist).ExeName),
            1u8,
        ) != 0
        {
            if UnicodeExe == 0 {
                ConsoleFreeHeap(ExeNameU.Buffer as PVOID);
            }
            return Hist;
        }
        Entry = (*Entry).Flink;
    }

    if UnicodeExe == 0 {
        ConsoleFreeHeap(ExeNameU.Buffer as PVOID);
    }
    core::ptr::null_mut()
}

pub(crate) unsafe fn HistoryDeleteBuffer(Hist: PHISTORY_BUFFER) {
    if Hist.is_null() {
        return;
    }

    while (*Hist).NumEntries != 0 {
        (*Hist).NumEntries -= 1;
        RtlFreeUnicodeString((*Hist).Entries.add((*Hist).NumEntries as usize));
    }

    ConsoleFreeHeap((*Hist).Entries as PVOID);
    RemoveEntryList(core::ptr::addr_of_mut!((*Hist).ListEntry) as PLIST_ENTRY);
    ConsoleFreeHeap(Hist as PVOID);
}

pub(crate) unsafe fn HistoryResizeBuffer(Hist: PHISTORY_BUFFER, NumCommands: ULONG) -> NTSTATUS {
    let OldEntryList = (*Hist).Entries;

    let NewEntryList = ConsoleAllocHeap(0, NumCommands as usize * core::mem::size_of::<_UNICODE_STRING>()) as PUNICODE_STRING;
    if NewEntryList.is_null() {
        return STATUS_NO_MEMORY;
    }

    while (*Hist).NumEntries > NumCommands {
        (*Hist).NumEntries -= 1;
        RtlFreeUnicodeString((*Hist).Entries);
        (*Hist).Entries = (*Hist).Entries.add(1);
        if (*Hist).Position == 0 {
            (*Hist).Position += 1;
        }
    }

    (*Hist).MaxEntries = NumCommands;
    RtlCopyMemory(
        NewEntryList as *mut core::ffi::c_void,
        (*Hist).Entries as *const core::ffi::c_void,
        (*Hist).NumEntries as usize * core::mem::size_of::<_UNICODE_STRING>(),
    );
    (*Hist).Entries = NewEntryList;
    ConsoleFreeHeap(OldEntryList as PVOID);

    STATUS_SUCCESS
}

// condrv flip: lineinput.rs calls this as `_HistoryAddEntry@12` (stdcall),
// matching its History* sibling family (Rust-only callers, self-consistent).
#[no_mangle]
pub unsafe extern "system" fn HistoryAddEntry(
    Console: PCONSRV_CONSOLE,
    ExeName: PUNICODE_STRING,
    Entry: PUNICODE_STRING,
) {
    let Hist = HistoryCurrentBuffer(Console, ExeName);

    if Hist.is_null() {
        return;
    }

    if (*Entry).Length == 0 || (*Hist).MaxEntries == 0 ||
        ((*Hist).NumEntries > 0 &&
         RtlEqualUnicodeString(
             (*Hist).Entries.add(((*Hist).NumEntries - 1) as usize),
             Entry,
             0u8,
         ) != 0)
    {
        return;
    }

    if (*Console).HistoryNoDup != 0 {
        let mut i = (*Hist).NumEntries as i32 - 1;

        while i >= 0 {
            if RtlEqualUnicodeString(
                (*Hist).Entries.add(i as usize),
                Entry,
                0u8,
            ) != 0
            {
                let NewEntry = *(*Hist).Entries.add(i as usize);

                RtlMoveMemory(
                    (*Hist).Entries.add(i as usize) as *mut core::ffi::c_void,
                    (*Hist).Entries.add((i + 1) as usize) as *const core::ffi::c_void,
                    ((*Hist).NumEntries - (i as u32 + 1)) as usize * core::mem::size_of::<_UNICODE_STRING>(),
                );
                *(*Hist).Entries.add(((*Hist).NumEntries - 1) as usize) = NewEntry;
                (*Hist).Position = (*Hist).NumEntries - 1;
                return;
            }
            i -= 1;
        }
    }

    if (*Hist).NumEntries == (*Hist).MaxEntries {
        RtlFreeUnicodeString((*Hist).Entries);
        RtlMoveMemory(
            (*Hist).Entries as *mut core::ffi::c_void,
            (*Hist).Entries.add(1) as *const core::ffi::c_void,
            {
                (*Hist).NumEntries -= 1;
                (*Hist).NumEntries as usize * core::mem::size_of::<_UNICODE_STRING>()
            },
        );
    }

    if RtlDuplicateUnicodeString(
        0,
        Entry,
        (*Hist).Entries.add((*Hist).NumEntries as usize),
    ) >= 0
    {
        (*Hist).NumEntries += 1;
    }
    (*Hist).Position = (*Hist).NumEntries - 1;
}

#[no_mangle]
pub unsafe extern "C" fn HistoryGetCurrentEntry(
    Console: PCONSRV_CONSOLE,
    ExeName: PUNICODE_STRING,
    Entry: PUNICODE_STRING,
) {
    let Hist = HistoryCurrentBuffer(Console, ExeName);

    if Hist.is_null() || (*Hist).NumEntries == 0 {
        (*Entry).Length = 0;
    } else {
        *Entry = *(*Hist).Entries.add((*Hist).Position as usize);
    }
}

#[no_mangle]
pub unsafe extern "C" fn HistoryRecallHistory(
    Console: PCONSRV_CONSOLE,
    ExeName: PUNICODE_STRING,
    Offset: i32,
    Entry: PUNICODE_STRING,
) -> BOOL {
    let Hist = HistoryCurrentBuffer(Console, ExeName);

    if Hist.is_null() || (*Hist).NumEntries == 0 {
        return 0;
    }

    let mut Position = ((*Hist).Position as i32 + Offset) as ULONG;
    if Position > (*Hist).NumEntries - 1 {
        Position = (*Hist).NumEntries - 1;
    }
    (*Hist).Position = Position;

    *Entry = *(*Hist).Entries.add((*Hist).Position as usize);
    1
}

#[no_mangle]
pub unsafe extern "C" fn HistoryFindEntryByPrefix(
    Console: PCONSRV_CONSOLE,
    ExeName: PUNICODE_STRING,
    Prefix: PUNICODE_STRING,
    Entry: PUNICODE_STRING,
) -> BOOL {
    let Hist = HistoryCurrentBuffer(Console, ExeName);
    if Hist.is_null() || (*Hist).NumEntries == 0 {
        return 0;
    }

    if (*Console).LineUpPressed != 0 {
        (*Hist).Position = if (*Hist).Position != 0 {
            (*Hist).Position
        } else {
            (*Hist).NumEntries
        } - 1;
    }
    (*Console).LineUpPressed = 1u8;

    let mut HistPos = (*Hist).Position as i32;
    loop {
        if RtlPrefixUnicodeString(
            Prefix,
            (*Hist).Entries.add(HistPos as usize),
            0u8,
        ) != 0
        {
            (*Hist).Position = HistPos as ULONG;
            *Entry = *(*Hist).Entries.add(HistPos as usize);
            return 1;
        }
        HistPos -= 1;
        if HistPos < 0 {
            HistPos += (*Hist).NumEntries as i32;
        }
        if HistPos == (*Hist).Position as i32 {
            break;
        }
    }

    0
}

#[no_mangle]
pub unsafe extern "C" fn HistoryDisplayCurrentHistory(
    Console: PCONSRV_CONSOLE,
    ExeName: PUNICODE_STRING,
) -> PPOPUP_WINDOW {
    let Hist = HistoryCurrentBuffer(Console, ExeName);

    if Hist.is_null() {
        return core::ptr::null_mut();
    }
    if (*Hist).NumEntries == 0 {
        return core::ptr::null_mut();
    }

    if GetType((*Console).ActiveBuffer) != TEXTMODE_BUFFER as ULONG {
        return core::ptr::null_mut();
    }
    let ActiveBuffer = (*Console).ActiveBuffer;

    let Width: i16 = 40;
    let Height: i16 = 10;

    let xLeft = (*ActiveBuffer).ViewOrigin.X + ((*ActiveBuffer).ViewSize.X - Width) / 2;
    let yTop = (*ActiveBuffer).ViewOrigin.Y + ((*ActiveBuffer).ViewSize.Y - Height) / 2;

    let Popup = CreatePopupWindow(Console, ActiveBuffer, xLeft, yTop, Width, Height);
    if Popup.is_null() {
        return core::ptr::null_mut();
    }

    (*Popup).PopupInputRoutine = None;

    Popup
}

#[no_mangle]
pub unsafe extern "C" fn HistoryDeleteCurrentBuffer(
    Console: PCONSRV_CONSOLE,
    ExeName: PUNICODE_STRING,
) {
    HistoryDeleteBuffer(HistoryCurrentBuffer(Console, ExeName));
}

#[no_mangle]
pub unsafe extern "C" fn HistoryDeleteBuffers(Console: PCONSRV_CONSOLE) {
    while IsListEmpty(core::ptr::addr_of_mut!((*Console).HistoryBuffers) as PLIST_ENTRY) == 0 {
        let CurrentEntry = RemoveHeadList(core::ptr::addr_of_mut!((*Console).HistoryBuffers) as PLIST_ENTRY);
        let HistoryBuffer = ((CurrentEntry as usize) - core::mem::offset_of!(HISTORY_BUFFER, ListEntry)) as PHISTORY_BUFFER;
        HistoryDeleteBuffer(HistoryBuffer);
    }
}

#[no_mangle]
pub unsafe extern "C" fn HistoryReshapeAllBuffers(
    Console: PCONSRV_CONSOLE,
    HistoryBufferSize: ULONG,
    MaxNumberOfHistoryBuffers: ULONG,
    HistoryNoDup: BOOLEAN,
) {
    if MaxNumberOfHistoryBuffers < (*Console).NumberOfHistoryBuffers {
        while IsListEmpty(core::ptr::addr_of_mut!((*Console).HistoryBuffers) as PLIST_ENTRY) == 0 &&
              (*Console).NumberOfHistoryBuffers > MaxNumberOfHistoryBuffers
        {
            let Entry = RemoveTailList(core::ptr::addr_of_mut!((*Console).HistoryBuffers) as PLIST_ENTRY);
            let Hist = ((Entry as usize) - core::mem::offset_of!(HISTORY_BUFFER, ListEntry)) as PHISTORY_BUFFER;
            HistoryDeleteBuffer(Hist);
            (*Console).NumberOfHistoryBuffers -= 1;
        }
    }
    (*Console).MaxNumberOfHistoryBuffers = MaxNumberOfHistoryBuffers;

    let mut Entry = (*Console).HistoryBuffers.Flink;
    while Entry != core::ptr::addr_of_mut!((*Console).HistoryBuffers) as PLIST_ENTRY {
        let Hist = ((Entry as usize) - core::mem::offset_of!(HISTORY_BUFFER, ListEntry)) as PHISTORY_BUFFER;
        let _Status = HistoryResizeBuffer(Hist, HistoryBufferSize);
        Entry = (*Entry).Flink;
    }
    (*Console).HistoryBufferSize = HistoryBufferSize;

    (*Console).HistoryNoDup = if HistoryNoDup != 0 { 1 } else { 0 };
}

#[no_mangle]
pub unsafe extern "system" fn SrvGetConsoleCommandHistory(
    ApiMessage: *mut _CSR_API_MESSAGE,
    ReplyCode: PCSR_REPLY_CODE,
) -> NTSTATUS {
    let ProcessData = ConsoleGetPerProcessData((*CsrGetClientThread()).Process);
    let mut Console: PCONSRV_CONSOLE = core::ptr::null_mut();
    let _gs = ConSrvGetConsole(ProcessData, core::ptr::addr_of_mut!(Console), 1u8);
    if !NT_SUCCESS(_gs) { let _ = ReplyCode; return _gs; }
    let GetCommandHistoryRequest = core::ptr::addr_of_mut!((*(ApiMessage as *mut _CONSOLE_API_MESSAGE)).Data.GetCommandHistoryRequest);

    let mut Status = STATUS_SUCCESS;
    let mut BytesWritten: ULONG = 0;

    if CsrValidateMessageBuffer(
        ApiMessage,
        core::ptr::addr_of_mut!((*GetCommandHistoryRequest).History) as *mut PVOID,
        (*GetCommandHistoryRequest).HistoryLength as ULONG,
        core::mem::size_of::<u8>() as ULONG,
    ) == 0 ||
       CsrValidateMessageBuffer(
           ApiMessage,
           core::ptr::addr_of_mut!((*GetCommandHistoryRequest).ExeName) as *mut PVOID,
           (*GetCommandHistoryRequest).ExeLength as ULONG,
           core::mem::size_of::<u8>() as ULONG,
       ) == 0
    {
        ConSrvReleaseConsole(Console, 1u8);
        return STATUS_INVALID_PARAMETER;
    }

    let Hist = HistoryFindBuffer(
        Console,
        (*GetCommandHistoryRequest).ExeName as PVOID,
        (*GetCommandHistoryRequest).ExeLength,
        (*GetCommandHistoryRequest).Unicode2,
    );
    if !Hist.is_null() {
        let mut TargetBufferA: LPSTR = core::ptr::null_mut();
        let mut TargetBufferW: LPWSTR = core::ptr::null_mut();
        let mut BufferSize = (*GetCommandHistoryRequest).HistoryLength;

        let mut Offset: ULONG = 0;

        if (*GetCommandHistoryRequest).Unicode != 0 {
            TargetBufferW = (*GetCommandHistoryRequest).History as LPWSTR;
            BufferSize /= core::mem::size_of::<u16>() as u32;
        } else {
            TargetBufferA = (*GetCommandHistoryRequest).History as LPSTR;
        }

        let mut i = 0;
        while i < (*Hist).NumEntries {
            let SourceLength = (*(*Hist).Entries.add(i as usize)).Length / core::mem::size_of::<u16>() as u16;
            if Offset + SourceLength as u32 + 1 > BufferSize {
                Status = STATUS_BUFFER_OVERFLOW;
                break;
            }

            if (*GetCommandHistoryRequest).Unicode != 0 {
                RtlCopyMemory(
                    TargetBufferW.add(Offset as usize) as *mut core::ffi::c_void,
                    (*(*Hist).Entries.add(i as usize)).Buffer as *const core::ffi::c_void,
                    SourceLength as usize * core::mem::size_of::<u16>(),
                );
                Offset += SourceLength as u32;
                *TargetBufferW.add(Offset as usize) = 0;
                Offset += 1;
            } else {
                ConvertInputUnicodeToAnsi(
                    Console,
                    (*(*Hist).Entries.add(i as usize)).Buffer as PVOID,
                    SourceLength * core::mem::size_of::<u16>() as u16,
                    TargetBufferA.add(Offset as usize),
                    SourceLength,
                );
                Offset += SourceLength as u32;
                *TargetBufferA.add(Offset as usize) = 0;
                Offset += 1;
            }
            i += 1;
        }

        if (*GetCommandHistoryRequest).Unicode != 0 {
            BytesWritten = Offset * core::mem::size_of::<u16>() as u32;
        } else {
            BytesWritten = Offset;
        }
    }

    (*GetCommandHistoryRequest).HistoryLength = BytesWritten;

    ConSrvReleaseConsole(Console, 1u8);
    Status
}

#[no_mangle]
pub unsafe extern "system" fn SrvGetConsoleCommandHistoryLength(
    ApiMessage: *mut _CSR_API_MESSAGE,
    ReplyCode: PCSR_REPLY_CODE,
) -> NTSTATUS {
    let ProcessData = ConsoleGetPerProcessData((*CsrGetClientThread()).Process);
    let mut Console: PCONSRV_CONSOLE = core::ptr::null_mut();
    let _gs = ConSrvGetConsole(ProcessData, core::ptr::addr_of_mut!(Console), 1u8);
    if !NT_SUCCESS(_gs) { let _ = ReplyCode; return _gs; }
    let GetCommandHistoryLengthRequest = core::ptr::addr_of_mut!((*(ApiMessage as *mut _CONSOLE_API_MESSAGE)).Data.GetCommandHistoryLengthRequest);

    let mut Length: ULONG = 0;

    if CsrValidateMessageBuffer(
        ApiMessage,
        core::ptr::addr_of_mut!((*GetCommandHistoryLengthRequest).ExeName) as *mut PVOID,
        (*GetCommandHistoryLengthRequest).ExeLength as ULONG,
        core::mem::size_of::<u8>() as ULONG,
    ) == 0
    {
        ConSrvReleaseConsole(Console, 1u8);
        return STATUS_INVALID_PARAMETER;
    }

    let Hist = HistoryFindBuffer(
        Console,
        (*GetCommandHistoryLengthRequest).ExeName as PVOID,
        (*GetCommandHistoryLengthRequest).ExeLength,
        (*GetCommandHistoryLengthRequest).Unicode2,
    );
    if !Hist.is_null() {
        let mut i = 0;
        while i < (*Hist).NumEntries {
            Length += (*(*Hist).Entries.add(i as usize)).Length as u32 + core::mem::size_of::<u16>() as u32;
            i += 1;
        }
    }

    if (*GetCommandHistoryLengthRequest).Unicode == 0 {
        Length /= core::mem::size_of::<u16>() as u32;
    }

    (*GetCommandHistoryLengthRequest).HistoryLength = Length;

    ConSrvReleaseConsole(Console, 1u8);
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn SrvExpungeConsoleCommandHistory(
    ApiMessage: *mut _CSR_API_MESSAGE,
    ReplyCode: PCSR_REPLY_CODE,
) -> NTSTATUS {
    let ProcessData = ConsoleGetPerProcessData((*CsrGetClientThread()).Process);
    let mut Console: PCONSRV_CONSOLE = core::ptr::null_mut();
    let _gs = ConSrvGetConsole(ProcessData, core::ptr::addr_of_mut!(Console), 1u8);
    if !NT_SUCCESS(_gs) { let _ = ReplyCode; return _gs; }
    let ExpungeCommandHistoryRequest = core::ptr::addr_of_mut!((*(ApiMessage as *mut _CONSOLE_API_MESSAGE)).Data.ExpungeCommandHistoryRequest);

    if CsrValidateMessageBuffer(
        ApiMessage,
        core::ptr::addr_of_mut!((*ExpungeCommandHistoryRequest).ExeName) as *mut PVOID,
        (*ExpungeCommandHistoryRequest).ExeLength as ULONG,
        core::mem::size_of::<u8>() as ULONG,
    ) == 0
    {
        ConSrvReleaseConsole(Console, 1u8);
        return STATUS_INVALID_PARAMETER;
    }

    let Hist = HistoryFindBuffer(
        Console,
        (*ExpungeCommandHistoryRequest).ExeName as PVOID,
        (*ExpungeCommandHistoryRequest).ExeLength,
        (*ExpungeCommandHistoryRequest).Unicode2,
    );
    HistoryDeleteBuffer(Hist);

    ConSrvReleaseConsole(Console, 1u8);
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn SrvSetConsoleNumberOfCommands(
    ApiMessage: *mut _CSR_API_MESSAGE,
    ReplyCode: PCSR_REPLY_CODE,
) -> NTSTATUS {
    let ProcessData = ConsoleGetPerProcessData((*CsrGetClientThread()).Process);
    let mut Console: PCONSRV_CONSOLE = core::ptr::null_mut();
    let _gs = ConSrvGetConsole(ProcessData, core::ptr::addr_of_mut!(Console), 1u8);
    if !NT_SUCCESS(_gs) { let _ = ReplyCode; return _gs; }
    let SetHistoryNumberCommandsRequest = core::ptr::addr_of_mut!((*(ApiMessage as *mut _CONSOLE_API_MESSAGE)).Data.SetHistoryNumberCommandsRequest);

    let mut Status = STATUS_SUCCESS;

    if CsrValidateMessageBuffer(
        ApiMessage,
        core::ptr::addr_of_mut!((*SetHistoryNumberCommandsRequest).ExeName) as *mut PVOID,
        (*SetHistoryNumberCommandsRequest).ExeLength as ULONG,
        core::mem::size_of::<u8>() as ULONG,
    ) == 0
    {
        ConSrvReleaseConsole(Console, 1u8);
        return STATUS_INVALID_PARAMETER;
    }

    let Hist = HistoryFindBuffer(
        Console,
        (*SetHistoryNumberCommandsRequest).ExeName as PVOID,
        (*SetHistoryNumberCommandsRequest).ExeLength,
        (*SetHistoryNumberCommandsRequest).Unicode2,
    );
    if !Hist.is_null() {
        Status = HistoryResizeBuffer(Hist, (*SetHistoryNumberCommandsRequest).NumCommands);
    }

    ConSrvReleaseConsole(Console, 1u8);
    Status
}

#[no_mangle]
pub unsafe extern "system" fn SrvGetConsoleHistory(
    ApiMessage: *mut _CSR_API_MESSAGE,
    _ReplyCode: PCSR_REPLY_CODE,
) -> NTSTATUS {
    let ProcessData = ConsoleGetPerProcessData((*CsrGetClientThread()).Process);
    let HistoryInfoRequest = core::ptr::addr_of_mut!((*(ApiMessage as *mut _CONSOLE_API_MESSAGE)).Data.HistoryInfoRequest);

    let mut Console: PCONSRV_CONSOLE = core::ptr::null_mut();

    let Status = ConSrvGetConsole(
        ProcessData,
        core::ptr::addr_of_mut!(Console),
        1u8,
    );
    if Status < 0 {
        return Status;
    }

    (*HistoryInfoRequest).HistoryBufferSize = (*Console).HistoryBufferSize;
    (*HistoryInfoRequest).NumberOfHistoryBuffers = (*Console).MaxNumberOfHistoryBuffers;
    (*HistoryInfoRequest).dwFlags = if (*Console).HistoryNoDup != 0 { 1 } else { 0 };

    ConSrvReleaseConsole(Console, 1u8);
    Status
}

#[no_mangle]
pub unsafe extern "system" fn SrvSetConsoleHistory(
    ApiMessage: *mut _CSR_API_MESSAGE,
    _ReplyCode: PCSR_REPLY_CODE,
) -> NTSTATUS {
    let ProcessData = ConsoleGetPerProcessData((*CsrGetClientThread()).Process);
    let HistoryInfoRequest = core::ptr::addr_of_mut!((*(ApiMessage as *mut _CONSOLE_API_MESSAGE)).Data.HistoryInfoRequest);

    let mut Console: PCONSRV_CONSOLE = core::ptr::null_mut();

    let Status = ConSrvGetConsole(
        ProcessData,
        core::ptr::addr_of_mut!(Console),
        1u8,
    );
    if Status < 0 {
        return Status;
    }

    HistoryReshapeAllBuffers(
        Console,
        (*HistoryInfoRequest).HistoryBufferSize,
        (*HistoryInfoRequest).NumberOfHistoryBuffers,
        if (*HistoryInfoRequest).dwFlags & 1 != 0 { 1u8 } else { 0u8 },
    );

    ConSrvReleaseConsole(Console, 1u8);
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn SrvSetConsoleCommandHistoryMode(
    ApiMessage: *mut _CSR_API_MESSAGE,
    ReplyCode: PCSR_REPLY_CODE,
) -> NTSTATUS {
    let ProcessData = ConsoleGetPerProcessData((*CsrGetClientThread()).Process);
    let mut Console: PCONSRV_CONSOLE = core::ptr::null_mut();
    let _gs = ConSrvGetConsole(ProcessData, core::ptr::addr_of_mut!(Console), 1u8);
    if !NT_SUCCESS(_gs) { let _ = ReplyCode; return _gs; }
    let SetHistoryModeRequest = core::ptr::addr_of_mut!((*(ApiMessage as *mut _CONSOLE_API_MESSAGE)).Data.SetHistoryModeRequest);

    (*Console).InsertMode = if (*SetHistoryModeRequest).Mode & 1 != 0 { 1u8 } else { 0u8 };

    ConSrvReleaseConsole(Console, 1u8);
    STATUS_SUCCESS
}