use winsrv_sys::nt::*;
use super::{ConsoleAllocHeap, ConsoleFreeHeap};
use winsrv_sys::winsrv::*;
use winsrv_sys::consts::*;
// Disambiguate winsrv_sys::winsrv:: (bindgen) vs sibling Rust impl.
use super::console::ConSrvGetConsole;
use super::console::ConSrvReleaseConsole;

#[repr(C)]
struct ALIAS_ENTRY {
    Next: *mut ALIAS_ENTRY,
    Source: _UNICODE_STRING,
    Target: _UNICODE_STRING,
}

type PALIAS_ENTRY = *mut ALIAS_ENTRY;

#[repr(C)]
struct ALIAS_HEADER {
    Next: *mut ALIAS_HEADER,
    ExeName: _UNICODE_STRING,
    Data: PALIAS_ENTRY,
}

type PALIAS_HEADER = *mut ALIAS_HEADER;

// condrv flip: history.rs (sibling) calls this cdecl `_ConvertInputAnsiToUnicode`.
#[no_mangle]
pub unsafe extern "C" fn ConvertInputAnsiToUnicode(
    Console: PCONSRV_CONSOLE,
    Source: PVOID,
    SourceLength: USHORT,
    Target: *mut PWCHAR,
    TargetLength: *mut USHORT,
) -> BOOLEAN {
    *TargetLength = MultiByteToWideChar(
        (*Console).InputCodePage,
        0,
        Source as LPCSTR,
        SourceLength as i32,
        core::ptr::null_mut(),
        0,
    ) as USHORT;
    *Target = ConsoleAllocHeap(0, (*TargetLength as usize) * core::mem::size_of::<WCHAR>()) as PWCHAR;
    if (*Target).is_null() {
        return 0u8;
    }

    MultiByteToWideChar(
        (*Console).InputCodePage,
        0,
        Source as LPCSTR,
        SourceLength as i32,
        *Target,
        *TargetLength as i32,
    );

    *TargetLength *= core::mem::size_of::<WCHAR>() as USHORT;

    1u8
}

#[no_mangle]
pub unsafe extern "C" fn ConvertInputUnicodeToAnsi(
    Console: PCONSRV_CONSOLE,
    Source: PVOID,
    SourceLength: USHORT,
    Target: PCHAR,
    TargetLength: USHORT,
) -> BOOLEAN {
    WideCharToMultiByte(
        (*Console).InputCodePage,
        0,
        Source as LPCWSTR,
        SourceLength as i32,
        Target as LPSTR,
        TargetLength as i32,
        core::ptr::null(),
        core::ptr::null_mut(),
    );

    1u8
}

pub(crate) unsafe fn IntFindAliasHeader(
    Console: PCONSRV_CONSOLE,
    ExeName: PVOID,
    ExeLength: USHORT,
    UnicodeExe: BOOLEAN,
) -> PALIAS_HEADER {
    let mut ExeNameU: _UNICODE_STRING = core::mem::zeroed();
    let mut RootHeader = (*Console).Aliases as PALIAS_HEADER ;
    let mut Diff: i32;

    if ExeName.is_null() {
        return core::ptr::null_mut();
    }

    if UnicodeExe != 0 {
        ExeNameU.Buffer = ExeName as PWSTR;
        ExeNameU.MaximumLength = ExeLength;
    } else {
        if ConvertInputAnsiToUnicode(
            Console,
            ExeName,
            ExeLength,
            &mut ExeNameU.Buffer,
            &mut ExeNameU.MaximumLength,
        ) == 0
        {
            return core::ptr::null_mut();
        }
    }
    ExeNameU.Length = ExeNameU.MaximumLength;

    while !RootHeader.is_null() {
        Diff = RtlCompareUnicodeString(
            core::ptr::addr_of!((*RootHeader).ExeName),
            core::ptr::addr_of!(ExeNameU),
            1u8,
        );
        if Diff == 0 {
            if UnicodeExe == 0 {
                ConsoleFreeHeap(ExeNameU.Buffer as PVOID);
            }
            return RootHeader;
        }
        if Diff > 0 {
            break;
        }

        RootHeader = (*RootHeader).Next;
    }

    if UnicodeExe == 0 {
        ConsoleFreeHeap(ExeNameU.Buffer as PVOID);
    }
    core::ptr::null_mut()
}

pub(crate) unsafe fn IntCreateAliasHeader(
    Console: PCONSRV_CONSOLE,
    ExeName: PVOID,
    ExeLength: USHORT,
    UnicodeExe: BOOLEAN,
) -> PALIAS_HEADER {
    let mut ExeNameU: _UNICODE_STRING = core::mem::zeroed();
    let Entry: PALIAS_HEADER;

    if ExeName.is_null() {
        return core::ptr::null_mut();
    }

    if UnicodeExe != 0 {
        ExeNameU.Buffer = ExeName as PWSTR;
        ExeNameU.MaximumLength = ExeLength;
    } else {
        if ConvertInputAnsiToUnicode(
            Console,
            ExeName,
            ExeLength,
            &mut ExeNameU.Buffer,
            &mut ExeNameU.MaximumLength,
        ) == 0
        {
            return core::ptr::null_mut();
        }
    }
    ExeNameU.Length = ExeNameU.MaximumLength;

    Entry = ConsoleAllocHeap(
        0,
        core::mem::size_of::<ALIAS_HEADER>() + ExeNameU.Length as usize,
    ) as PALIAS_HEADER;
    if Entry.is_null() {
        if UnicodeExe == 0 {
            ConsoleFreeHeap(ExeNameU.Buffer as PVOID);
        }
        return Entry;
    }

    (*Entry).ExeName.Buffer = (Entry as usize + core::mem::size_of::<ALIAS_HEADER>()) as PWSTR;
    (*Entry).ExeName.Length = 0;
    (*Entry).ExeName.MaximumLength = ExeNameU.Length;
    RtlCopyUnicodeString(
        core::ptr::addr_of_mut!((*Entry).ExeName),
        core::ptr::addr_of!(ExeNameU),
    );

    (*Entry).Data = core::ptr::null_mut();
    (*Entry).Next = core::ptr::null_mut();

    if UnicodeExe == 0 {
        ConsoleFreeHeap(ExeNameU.Buffer as PVOID);
    }
    Entry
}

pub(crate) unsafe fn IntInsertAliasHeader(RootHeader: *mut PALIAS_HEADER, NewHeader: PALIAS_HEADER) {
    let mut CurrentHeader: PALIAS_HEADER;
    let mut LastLink = RootHeader;
    let mut Diff: i32;

    loop {
        CurrentHeader = *LastLink;
        if CurrentHeader.is_null() {
            break;
        }

        Diff = RtlCompareUnicodeString(
            core::ptr::addr_of!((*NewHeader).ExeName),
            core::ptr::addr_of!((*CurrentHeader).ExeName),
            1u8,
        );
        if Diff < 0 {
            break;
        }

        LastLink = core::ptr::addr_of_mut!((*CurrentHeader).Next);
    }

    *LastLink = NewHeader;
    (*NewHeader).Next = CurrentHeader;
}

pub(crate) unsafe fn IntGetAliasEntry(
    Console: PCONSRV_CONSOLE,
    Header: PALIAS_HEADER,
    Source: PVOID,
    SourceLength: USHORT,
    Unicode: BOOLEAN,
) -> PALIAS_ENTRY {
    let mut SourceU: _UNICODE_STRING = core::mem::zeroed();
    let mut Entry: PALIAS_ENTRY;
    let mut Diff: i32;

    if Header.is_null() || Source.is_null() {
        return core::ptr::null_mut();
    }

    if Unicode != 0 {
        SourceU.Buffer = Source as PWSTR;
        SourceU.MaximumLength = SourceLength;
    } else {
        if ConvertInputAnsiToUnicode(
            Console,
            Source,
            SourceLength,
            &mut SourceU.Buffer,
            &mut SourceU.MaximumLength,
        ) == 0
        {
            return core::ptr::null_mut();
        }
    }
    SourceU.Length = SourceU.MaximumLength;

    Entry = (*Header).Data;
    while !Entry.is_null() {
        Diff = RtlCompareUnicodeString(
            core::ptr::addr_of!((*Entry).Source),
            core::ptr::addr_of!(SourceU),
            1u8,
        );
        if Diff == 0 {
            if Unicode == 0 {
                ConsoleFreeHeap(SourceU.Buffer as PVOID);
            }
            return Entry;
        }
        if Diff > 0 {
            break;
        }

        Entry = (*Entry).Next;
    }

    if Unicode == 0 {
        ConsoleFreeHeap(SourceU.Buffer as PVOID);
    }
    core::ptr::null_mut()
}

pub(crate) unsafe fn IntCreateAliasEntry(
    Console: PCONSRV_CONSOLE,
    Source: PVOID,
    SourceLength: USHORT,
    Target: PVOID,
    TargetLength: USHORT,
    Unicode: BOOLEAN,
) -> PALIAS_ENTRY {
    let mut SourceU: _UNICODE_STRING = core::mem::zeroed();
    let mut TargetU: _UNICODE_STRING = core::mem::zeroed();
    let Entry: PALIAS_ENTRY;

    if Unicode != 0 {
        SourceU.Buffer = Source as PWSTR;
        TargetU.Buffer = Target as PWSTR;
        SourceU.MaximumLength = SourceLength;
        TargetU.MaximumLength = TargetLength;
    } else {
        if ConvertInputAnsiToUnicode(
            Console,
            Source,
            SourceLength,
            &mut SourceU.Buffer,
            &mut SourceU.MaximumLength,
        ) == 0
        {
            return core::ptr::null_mut();
        }

        if ConvertInputAnsiToUnicode(
            Console,
            Target,
            TargetLength,
            &mut TargetU.Buffer,
            &mut TargetU.MaximumLength,
        ) == 0
        {
            ConsoleFreeHeap(SourceU.Buffer as PVOID);
            return core::ptr::null_mut();
        }
    }
    SourceU.Length = SourceU.MaximumLength;
    TargetU.Length = TargetU.MaximumLength;

    Entry = ConsoleAllocHeap(
        0,
        core::mem::size_of::<ALIAS_ENTRY>() + SourceU.Length as usize + TargetU.Length as usize,
    ) as PALIAS_ENTRY;
    if Entry.is_null() {
        if Unicode == 0 {
            ConsoleFreeHeap(TargetU.Buffer as PVOID);
            ConsoleFreeHeap(SourceU.Buffer as PVOID);
        }
        return Entry;
    }

    (*Entry).Source.Buffer = (Entry as usize + core::mem::size_of::<ALIAS_ENTRY>()) as PWSTR;
    (*Entry).Source.Length = 0;
    (*Entry).Source.MaximumLength = SourceU.Length;
    RtlCopyUnicodeString(
        core::ptr::addr_of_mut!((*Entry).Source),
        core::ptr::addr_of!(SourceU),
    );

    (*Entry).Target.Buffer = ((*Entry).Source.Buffer as usize + (*Entry).Source.MaximumLength as usize) as PWSTR;
    (*Entry).Target.Length = 0;
    (*Entry).Target.MaximumLength = TargetU.Length;
    RtlCopyUnicodeString(
        core::ptr::addr_of_mut!((*Entry).Target),
        core::ptr::addr_of!(TargetU),
    );

    (*Entry).Next = core::ptr::null_mut();

    if Unicode == 0 {
        ConsoleFreeHeap(TargetU.Buffer as PVOID);
        ConsoleFreeHeap(SourceU.Buffer as PVOID);
    }
    Entry
}

pub(crate) unsafe fn IntInsertAliasEntry(Header: PALIAS_HEADER, NewEntry: PALIAS_ENTRY) {
    let mut CurrentEntry: PALIAS_ENTRY;
    let mut LastLink = core::ptr::addr_of_mut!((*Header).Data);
    let mut Diff: i32;

    loop {
        CurrentEntry = *LastLink;
        if CurrentEntry.is_null() {
            break;
        }

        Diff = RtlCompareUnicodeString(
            core::ptr::addr_of!((*NewEntry).Source),
            core::ptr::addr_of!((*CurrentEntry).Source),
            1u8,
        );
        if Diff < 0 {
            break;
        }

        LastLink = core::ptr::addr_of_mut!((*CurrentEntry).Next);
    }

    *LastLink = NewEntry;
    (*NewEntry).Next = CurrentEntry;
}

pub(crate) unsafe fn IntDeleteAliasEntry(Header: PALIAS_HEADER, Entry: PALIAS_ENTRY) {
    let mut LastLink = core::ptr::addr_of_mut!((*Header).Data);
    let mut CurEntry: PALIAS_ENTRY;

    loop {
        CurEntry = *LastLink;
        if CurEntry.is_null() {
            break;
        }

        if CurEntry == Entry {
            *LastLink = (*Entry).Next;
            ConsoleFreeHeap(Entry as PVOID);
            return;
        }
        LastLink = core::ptr::addr_of_mut!((*CurEntry).Next);
    }
}

pub(crate) unsafe fn IntGetConsoleAliasesExesLength(mut RootHeader: PALIAS_HEADER, IsUnicode: BOOLEAN) -> UINT {
    let mut Length: UINT = 0;

    while !RootHeader.is_null() {
        Length += (*RootHeader).ExeName.Length as UINT + core::mem::size_of::<WCHAR>() as UINT;
        RootHeader = (*RootHeader).Next;
    }

    if IsUnicode == 0 {
        Length /= core::mem::size_of::<WCHAR>() as UINT;
    }

    Length
}

pub(crate) unsafe fn IntGetAllConsoleAliasesLength(Header: PALIAS_HEADER, IsUnicode: BOOLEAN) -> UINT {
    let mut Length: UINT = 0;
    let mut CurEntry = (*Header).Data;

    while !CurEntry.is_null() {
        Length += (*CurEntry).Source.Length as UINT;
        Length += (*CurEntry).Target.Length as UINT;
        Length += 2 * core::mem::size_of::<WCHAR>() as UINT;
        CurEntry = (*CurEntry).Next;
    }

    if IsUnicode == 0 {
        Length /= core::mem::size_of::<WCHAR>() as UINT;
    }

    Length
}

#[no_mangle]
pub unsafe extern "C" fn IntDeleteAllAliases(Console: PCONSRV_CONSOLE) {
    let mut Header = (*Console).Aliases as PALIAS_HEADER ;
    let mut NextHeader: PALIAS_HEADER;
    let mut Entry: PALIAS_ENTRY;
    let mut NextEntry: PALIAS_ENTRY;

    while !Header.is_null() {
        NextHeader = (*Header).Next;
        Entry = (*Header).Data;
        while !Entry.is_null() {
            NextEntry = (*Entry).Next;
            ConsoleFreeHeap(Entry as PVOID);
            Entry = NextEntry;
        }
        ConsoleFreeHeap(Header as PVOID);
        Header = NextHeader;
    }
}

#[no_mangle]
pub unsafe extern "system" fn SrvAddConsoleAlias(
    ApiMessage: PCSR_API_MESSAGE,
    ReplyCode: PCSR_REPLY_CODE,
) -> NTSTATUS {
    let _ = ReplyCode;
    let ProcessData = ConsoleGetPerProcessData((*CsrGetClientThread()).Process);
    let mut Console: PCONSRV_CONSOLE = core::ptr::null_mut();
    let Status = ConSrvGetConsole(ProcessData, core::ptr::addr_of_mut!(Console), 1u8);
    if !NT_SUCCESS(Status) { return Status; }
    let Request_w = core::ptr::addr_of_mut!((*(ApiMessage as PCONSOLE_API_MESSAGE)).Data.ConsoleAliasRequest) as PCONSOLE_ADDGETALIAS;
    let result = SrvAddConsoleAliasImpl(Console, ApiMessage, Request_w);
    ConSrvReleaseConsole(Console, 1u8);
    result
}

pub(crate) unsafe fn SrvAddConsoleAliasImpl(
    Console: PCONSRV_CONSOLE,
    ApiMessage: PCSR_API_MESSAGE,
    ConsoleAliasRequest: PCONSOLE_ADDGETALIAS,
) -> NTSTATUS {
    let mut Header: PALIAS_HEADER;
    let Entry: PALIAS_ENTRY;
    let lpTarget: PVOID;

    if CsrValidateMessageBuffer(
        ApiMessage,
        core::ptr::addr_of_mut!((*ConsoleAliasRequest).Source) as *mut PVOID,
        (*ConsoleAliasRequest).SourceLength as ULONG,
        core::mem::size_of::<BYTE>() as ULONG,
    ) == 0 || CsrValidateMessageBuffer(
        ApiMessage,
        core::ptr::addr_of_mut!((*ConsoleAliasRequest).Target) as *mut PVOID,
        (*ConsoleAliasRequest).TargetLength as ULONG,
        core::mem::size_of::<BYTE>() as ULONG,
    ) == 0 || CsrValidateMessageBuffer(
        ApiMessage,
        core::ptr::addr_of_mut!((*ConsoleAliasRequest).ExeName) as *mut PVOID,
        (*ConsoleAliasRequest).ExeLength as ULONG,
        core::mem::size_of::<BYTE>() as ULONG,
    ) == 0 {
        return STATUS_INVALID_PARAMETER;
    }

    lpTarget = if (*ConsoleAliasRequest).TargetLength != 0 {
        (*ConsoleAliasRequest).Target as PVOID
    } else {
        core::ptr::null_mut()
    };

    Header = IntFindAliasHeader(
        Console,
        (*ConsoleAliasRequest).ExeName as PVOID,
        (*ConsoleAliasRequest).ExeLength,
        (*ConsoleAliasRequest).Unicode2,
    );
    if Header.is_null() && !lpTarget.is_null() {
        Header = IntCreateAliasHeader(
            Console,
            (*ConsoleAliasRequest).ExeName as PVOID,
            (*ConsoleAliasRequest).ExeLength,
            (*ConsoleAliasRequest).Unicode2,
        );
        if Header.is_null() {
            return STATUS_NO_MEMORY;
        }

        IntInsertAliasHeader(core::ptr::addr_of_mut!((*Console).Aliases) as *mut PALIAS_HEADER, Header);
    }

    if lpTarget.is_null() {
        Entry = IntGetAliasEntry(
            Console,
            Header,
            (*ConsoleAliasRequest).Source as PVOID,
            (*ConsoleAliasRequest).SourceLength,
            (*ConsoleAliasRequest).Unicode,
        );
        if Entry.is_null() {
            return STATUS_UNSUCCESSFUL;
        }

        IntDeleteAliasEntry(Header, Entry);
    } else {
        Entry = IntCreateAliasEntry(
            Console,
            (*ConsoleAliasRequest).Source as PVOID,
            (*ConsoleAliasRequest).SourceLength,
            (*ConsoleAliasRequest).Target as PVOID,
            (*ConsoleAliasRequest).TargetLength,
            (*ConsoleAliasRequest).Unicode,
        );
        if Entry.is_null() {
            return STATUS_NO_MEMORY;
        }

        IntInsertAliasEntry(Header, Entry);
    }

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn SrvGetConsoleAlias(
    ApiMessage: PCSR_API_MESSAGE,
    ReplyCode: PCSR_REPLY_CODE,
) -> NTSTATUS {
    let _ = ReplyCode;
    let ProcessData = ConsoleGetPerProcessData((*CsrGetClientThread()).Process);
    let mut Console: PCONSRV_CONSOLE = core::ptr::null_mut();
    let Status = ConSrvGetConsole(ProcessData, core::ptr::addr_of_mut!(Console), 1u8);
    if !NT_SUCCESS(Status) { return Status; }
    let Request_w = core::ptr::addr_of_mut!((*(ApiMessage as PCONSOLE_API_MESSAGE)).Data.ConsoleAliasRequest) as PCONSOLE_ADDGETALIAS;
    let result = SrvGetConsoleAliasImpl(Console, ApiMessage, Request_w);
    ConSrvReleaseConsole(Console, 1u8);
    result
}

pub(crate) unsafe fn SrvGetConsoleAliasImpl(
    Console: PCONSRV_CONSOLE,
    ApiMessage: PCSR_API_MESSAGE,
    ConsoleAliasRequest: PCONSOLE_ADDGETALIAS,
) -> NTSTATUS {
    let Header: PALIAS_HEADER;
    let Entry: PALIAS_ENTRY;
    let Length: UINT;
    let lpTarget: PVOID;

    if CsrValidateMessageBuffer(
        ApiMessage,
        core::ptr::addr_of_mut!((*ConsoleAliasRequest).Source) as *mut PVOID,
        (*ConsoleAliasRequest).SourceLength as ULONG,
        core::mem::size_of::<BYTE>() as ULONG,
    ) == 0 || CsrValidateMessageBuffer(
        ApiMessage,
        core::ptr::addr_of_mut!((*ConsoleAliasRequest).Target) as *mut PVOID,
        (*ConsoleAliasRequest).TargetLength as ULONG,
        core::mem::size_of::<BYTE>() as ULONG,
    ) == 0 || CsrValidateMessageBuffer(
        ApiMessage,
        core::ptr::addr_of_mut!((*ConsoleAliasRequest).ExeName) as *mut PVOID,
        (*ConsoleAliasRequest).ExeLength as ULONG,
        core::mem::size_of::<BYTE>() as ULONG,
    ) == 0 {
        return STATUS_INVALID_PARAMETER;
    }

    lpTarget = (*ConsoleAliasRequest).Target as PVOID;

    if (*ConsoleAliasRequest).ExeLength == 0
        || lpTarget.is_null()
        || (*ConsoleAliasRequest).TargetLength == 0
        || (*ConsoleAliasRequest).SourceLength == 0
    {
        return STATUS_INVALID_PARAMETER;
    }

    Header = IntFindAliasHeader(
        Console,
        (*ConsoleAliasRequest).ExeName as PVOID,
        (*ConsoleAliasRequest).ExeLength,
        (*ConsoleAliasRequest).Unicode2,
    );
    if Header.is_null() {
        return STATUS_UNSUCCESSFUL;
    }

    Entry = IntGetAliasEntry(
        Console,
        Header,
        (*ConsoleAliasRequest).Source as PVOID,
        (*ConsoleAliasRequest).SourceLength,
        (*ConsoleAliasRequest).Unicode,
    );
    if Entry.is_null() {
        return STATUS_UNSUCCESSFUL;
    }

    if (*ConsoleAliasRequest).Unicode != 0 {
        Length = (*Entry).Target.Length as UINT + core::mem::size_of::<WCHAR>() as UINT;
        if Length > (*ConsoleAliasRequest).TargetLength as UINT {
            return STATUS_BUFFER_TOO_SMALL;
        }

        core::ptr::copy_nonoverlapping(
            (*Entry).Target.Buffer as *const u8,
            lpTarget as *mut u8,
            (*Entry).Target.Length as usize,
        );
        (*ConsoleAliasRequest).TargetLength = Length as USHORT;
    } else {
        Length = ((*Entry).Target.Length as UINT + core::mem::size_of::<WCHAR>() as UINT)
            / core::mem::size_of::<WCHAR>() as UINT;
        if Length > (*ConsoleAliasRequest).TargetLength as UINT {
            return STATUS_BUFFER_TOO_SMALL;
        }

        ConvertInputUnicodeToAnsi(
            Console,
            (*Entry).Target.Buffer as PVOID,
            (*Entry).Target.Length,
            lpTarget as PCHAR,
            ((*Entry).Target.Length / core::mem::size_of::<WCHAR>() as USHORT) as USHORT,
        );
        (*ConsoleAliasRequest).TargetLength = Length as USHORT;
    }

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn SrvGetConsoleAliases(
    ApiMessage: PCSR_API_MESSAGE,
    ReplyCode: PCSR_REPLY_CODE,
) -> NTSTATUS {
    let _ = ReplyCode;
    let ProcessData = ConsoleGetPerProcessData((*CsrGetClientThread()).Process);
    let mut Console: PCONSRV_CONSOLE = core::ptr::null_mut();
    let Status = ConSrvGetConsole(ProcessData, core::ptr::addr_of_mut!(Console), 1u8);
    if !NT_SUCCESS(Status) { return Status; }
    let Request_w = core::ptr::addr_of_mut!((*(ApiMessage as PCONSOLE_API_MESSAGE)).Data.GetAllAliasesRequest) as PCONSOLE_GETALLALIASES;
    let result = SrvGetConsoleAliasesImpl(Console, ApiMessage, Request_w);
    ConSrvReleaseConsole(Console, 1u8);
    result
}

pub(crate) unsafe fn SrvGetConsoleAliasesImpl(
    Console: PCONSRV_CONSOLE,
    ApiMessage: PCSR_API_MESSAGE,
    GetAllAliasesRequest: PCONSOLE_GETALLALIASES,
) -> NTSTATUS {
    let mut Status = STATUS_SUCCESS;
    let mut BytesWritten: ULONG = 0;
    let Header: PALIAS_HEADER;

    if CsrValidateMessageBuffer(
        ApiMessage,
        core::ptr::addr_of!((*GetAllAliasesRequest).ExeName) as *mut PVOID,
        (*GetAllAliasesRequest).ExeLength as ULONG,
        core::mem::size_of::<BYTE>() as ULONG,
    ) == 0 || CsrValidateMessageBuffer(
        ApiMessage,
        core::ptr::addr_of!((*GetAllAliasesRequest).AliasesBuffer) as *mut PVOID,
        (*GetAllAliasesRequest).AliasesBufferLength,
        core::mem::size_of::<BYTE>() as ULONG,
    ) == 0 {
        return STATUS_INVALID_PARAMETER;
    }

    Header = IntFindAliasHeader(
        Console,
        (*GetAllAliasesRequest).ExeName as PVOID,
        (*GetAllAliasesRequest).ExeLength,
        (*GetAllAliasesRequest).Unicode2,
    );
    if Header.is_null() {
        Status = STATUS_UNSUCCESSFUL;
    } else if IntGetAllConsoleAliasesLength(Header, (*GetAllAliasesRequest).Unicode)
        > (*GetAllAliasesRequest).AliasesBufferLength
    {
        Status = STATUS_BUFFER_OVERFLOW;
    } else {
        let mut TargetBufferA: LPSTR = core::ptr::null_mut();
        let mut TargetBufferW: LPWSTR = core::ptr::null_mut();
        let mut TargetBufferLength = (*GetAllAliasesRequest).AliasesBufferLength;

        let mut CurEntry = (*Header).Data;
        let mut Offset: UINT = 0;
        let mut SourceLength: UINT;
        let mut TargetLength: UINT;

        if (*GetAllAliasesRequest).Unicode != 0 {
            TargetBufferW = (*GetAllAliasesRequest).AliasesBuffer as LPWSTR;
            TargetBufferLength /= core::mem::size_of::<WCHAR>() as ULONG;
        } else {
            TargetBufferA = (*GetAllAliasesRequest).AliasesBuffer as LPSTR;
        }

        while !CurEntry.is_null() {
            SourceLength = (*CurEntry).Source.Length as UINT / core::mem::size_of::<WCHAR>() as UINT;
            TargetLength = (*CurEntry).Target.Length as UINT / core::mem::size_of::<WCHAR>() as UINT;
            if Offset + TargetLength + SourceLength + 2 > TargetBufferLength {
                Status = STATUS_BUFFER_OVERFLOW;
                break;
            }

            if (*GetAllAliasesRequest).Unicode != 0 {
                core::ptr::copy_nonoverlapping(
                    (*CurEntry).Source.Buffer,
                    TargetBufferW.add(Offset as usize),
                    SourceLength as usize,
                );
                Offset += SourceLength;
                *TargetBufferW.add(Offset as usize) = b'=' as u16;
                Offset += 1;
                core::ptr::copy_nonoverlapping(
                    (*CurEntry).Target.Buffer,
                    TargetBufferW.add(Offset as usize),
                    TargetLength as usize,
                );
                Offset += TargetLength;
                *TargetBufferW.add(Offset as usize) = 0;
                Offset += 1;
            } else {
                ConvertInputUnicodeToAnsi(
                    Console,
                    (*CurEntry).Source.Buffer as PVOID,
                    (SourceLength * core::mem::size_of::<WCHAR>() as UINT) as USHORT,
                    TargetBufferA.add(Offset as usize),
                    SourceLength as USHORT,
                );
                Offset += SourceLength;
                *TargetBufferA.add(Offset as usize) = b'=' as i8;
                Offset += 1;
                ConvertInputUnicodeToAnsi(
                    Console,
                    (*CurEntry).Target.Buffer as PVOID,
                    (TargetLength * core::mem::size_of::<WCHAR>() as UINT) as USHORT,
                    TargetBufferA.add(Offset as usize),
                    TargetLength as USHORT,
                );
                Offset += TargetLength;
                *TargetBufferA.add(Offset as usize) = 0;
                Offset += 1;
            }

            CurEntry = (*CurEntry).Next;
        }

        if (*GetAllAliasesRequest).Unicode != 0 {
            BytesWritten = Offset * core::mem::size_of::<WCHAR>() as UINT;
        } else {
            BytesWritten = Offset;
        }
    }

    (*GetAllAliasesRequest).AliasesBufferLength = BytesWritten;

    Status
}

#[no_mangle]
pub unsafe extern "system" fn SrvGetConsoleAliasesLength(
    ApiMessage: PCSR_API_MESSAGE,
    ReplyCode: PCSR_REPLY_CODE,
) -> NTSTATUS {
    let _ = ReplyCode;
    let ProcessData = ConsoleGetPerProcessData((*CsrGetClientThread()).Process);
    let mut Console: PCONSRV_CONSOLE = core::ptr::null_mut();
    let Status = ConSrvGetConsole(ProcessData, core::ptr::addr_of_mut!(Console), 1u8);
    if !NT_SUCCESS(Status) { return Status; }
    let Request_w = core::ptr::addr_of_mut!((*(ApiMessage as PCONSOLE_API_MESSAGE)).Data.GetAllAliasesLengthRequest) as PCONSOLE_GETALLALIASESLENGTH;
    let result = SrvGetConsoleAliasesLengthImpl(Console, ApiMessage, Request_w);
    ConSrvReleaseConsole(Console, 1u8);
    result
}

pub(crate) unsafe fn SrvGetConsoleAliasesLengthImpl(
    Console: PCONSRV_CONSOLE,
    ApiMessage: PCSR_API_MESSAGE,
    GetAllAliasesLengthRequest: PCONSOLE_GETALLALIASESLENGTH,
) -> NTSTATUS {
    let mut Status = STATUS_SUCCESS;
    let Header: PALIAS_HEADER;

    if CsrValidateMessageBuffer(
        ApiMessage,
        core::ptr::addr_of!((*GetAllAliasesLengthRequest).ExeName) as *mut PVOID,
        (*GetAllAliasesLengthRequest).ExeLength as ULONG,
        core::mem::size_of::<BYTE>() as ULONG,
    ) == 0 {
        return STATUS_INVALID_PARAMETER;
    }

    Header = IntFindAliasHeader(
        Console,
        (*GetAllAliasesLengthRequest).ExeName as PVOID,
        (*GetAllAliasesLengthRequest).ExeLength,
        (*GetAllAliasesLengthRequest).Unicode2,
    );
    if !Header.is_null() {
        (*GetAllAliasesLengthRequest).Length =
            IntGetAllConsoleAliasesLength(Header, (*GetAllAliasesLengthRequest).Unicode);
        Status = STATUS_SUCCESS;
    } else {
        (*GetAllAliasesLengthRequest).Length = 0;
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn SrvGetConsoleAliasExes(
    ApiMessage: PCSR_API_MESSAGE,
    ReplyCode: PCSR_REPLY_CODE,
) -> NTSTATUS {
    let _ = ReplyCode;
    let ProcessData = ConsoleGetPerProcessData((*CsrGetClientThread()).Process);
    let mut Console: PCONSRV_CONSOLE = core::ptr::null_mut();
    let Status = ConSrvGetConsole(ProcessData, core::ptr::addr_of_mut!(Console), 1u8);
    if !NT_SUCCESS(Status) { return Status; }
    let Request_w = core::ptr::addr_of_mut!((*(ApiMessage as PCONSOLE_API_MESSAGE)).Data.GetAliasesExesRequest) as PCONSOLE_GETALIASESEXES;
    let result = SrvGetConsoleAliasExesImpl(Console, ApiMessage, Request_w);
    ConSrvReleaseConsole(Console, 1u8);
    result
}

pub(crate) unsafe fn SrvGetConsoleAliasExesImpl(
    Console: PCONSRV_CONSOLE,
    ApiMessage: PCSR_API_MESSAGE,
    GetAliasesExesRequest: PCONSOLE_GETALIASESEXES,
) -> NTSTATUS {
    let mut Status = STATUS_SUCCESS;
    let mut BytesWritten: UINT = 0;

    if CsrValidateMessageBuffer(
        ApiMessage,
        core::ptr::addr_of_mut!((*GetAliasesExesRequest).ExeNames) as *mut PVOID,
        (*GetAliasesExesRequest).Length,
        core::mem::size_of::<BYTE>() as ULONG,
    ) == 0 {
        return STATUS_INVALID_PARAMETER;
    }

    if IntGetConsoleAliasesExesLength((*Console).Aliases as PALIAS_HEADER , (*GetAliasesExesRequest).Unicode)
        > (*GetAliasesExesRequest).Length
    {
        Status = STATUS_BUFFER_OVERFLOW;
    } else {
        let mut RootHeader = (*Console).Aliases as PALIAS_HEADER ;

        let mut TargetBufferA: LPSTR = core::ptr::null_mut();
        let mut TargetBufferW: LPWSTR = core::ptr::null_mut();
        let mut TargetBufferSize = (*GetAliasesExesRequest).Length;

        let mut Offset: UINT = 0;
        let mut Length: UINT;

        if (*GetAliasesExesRequest).Unicode != 0 {
            TargetBufferW = (*GetAliasesExesRequest).ExeNames as LPWSTR;
            TargetBufferSize /= core::mem::size_of::<WCHAR>() as ULONG;
        } else {
            TargetBufferA = (*GetAliasesExesRequest).ExeNames as LPSTR;
        }

        while !RootHeader.is_null() {
            Length = (*RootHeader).ExeName.Length as UINT / core::mem::size_of::<WCHAR>() as UINT;
            if Offset + Length + 1 > TargetBufferSize {
                Status = STATUS_BUFFER_OVERFLOW;
                break;
            }

            if (*GetAliasesExesRequest).Unicode != 0 {
                core::ptr::copy_nonoverlapping(
                    (*RootHeader).ExeName.Buffer,
                    TargetBufferW.add(Offset as usize),
                    Length as usize,
                );
                Offset += Length;
                *TargetBufferW.add(Offset as usize) = 0;
                Offset += 1;
            } else {
                ConvertInputUnicodeToAnsi(
                    Console,
                    (*RootHeader).ExeName.Buffer as PVOID,
                    (Length * core::mem::size_of::<WCHAR>() as UINT) as USHORT,
                    TargetBufferA.add(Offset as usize),
                    Length as USHORT,
                );
                Offset += Length;
                *TargetBufferA.add(Offset as usize) = 0;
                Offset += 1;
            }

            RootHeader = (*RootHeader).Next;
        }

        if (*GetAliasesExesRequest).Unicode != 0 {
            BytesWritten = Offset * core::mem::size_of::<WCHAR>() as UINT;
        } else {
            BytesWritten = Offset;
        }
    }

    (*GetAliasesExesRequest).Length = BytesWritten;

    Status
}

#[no_mangle]
pub unsafe extern "system" fn SrvGetConsoleAliasExesLength(
    ApiMessage: PCSR_API_MESSAGE,
    ReplyCode: PCSR_REPLY_CODE,
) -> NTSTATUS {
    let _ = ReplyCode;
    let ProcessData = ConsoleGetPerProcessData((*CsrGetClientThread()).Process);
    let mut Console: PCONSRV_CONSOLE = core::ptr::null_mut();
    let Status = ConSrvGetConsole(ProcessData, core::ptr::addr_of_mut!(Console), 1u8);
    if !NT_SUCCESS(Status) { return Status; }
    let Request_w = core::ptr::addr_of_mut!((*(ApiMessage as PCONSOLE_API_MESSAGE)).Data.GetAliasesExesLengthRequest) as PCONSOLE_GETALIASESEXESLENGTH;
    let result = SrvGetConsoleAliasExesLengthImpl(Console, ApiMessage, Request_w);
    ConSrvReleaseConsole(Console, 1u8);
    result
}

pub(crate) unsafe fn SrvGetConsoleAliasExesLengthImpl(
    Console: PCONSRV_CONSOLE,
    _ApiMessage: PCSR_API_MESSAGE,
    GetAliasesExesLengthRequest: PCONSOLE_GETALIASESEXESLENGTH,
) -> NTSTATUS {

    (*GetAliasesExesLengthRequest).Length = IntGetConsoleAliasesExesLength(
        (*Console).Aliases as PALIAS_HEADER ,
        (*GetAliasesExesLengthRequest).Unicode,
    );

    STATUS_SUCCESS
}