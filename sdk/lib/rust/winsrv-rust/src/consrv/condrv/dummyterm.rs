use winsrv_sys::nt::*;
use winsrv_sys::winsrv::*;

unsafe extern "system" fn DummyInitTerminal(
    This: PTERMINAL,
    Console: PCONSOLE,
) -> NTSTATUS {
    STATUS_SUCCESS
}

unsafe extern "system" fn DummyDeinitTerminal(This: PTERMINAL) {
}

unsafe extern "system" fn DummyReadStream(
    This: PTERMINAL,
    Unicode: BOOLEAN,
    Buffer: PVOID,
    ReadControl: PCONSOLE_READCONSOLE_CONTROL,
    Parameter: PVOID,
    NumCharsToRead: ULONG,
    NumCharsRead: PULONG,
) -> NTSTATUS {
    STATUS_PENDING
}

unsafe extern "system" fn DummyWriteStream(
    This: PTERMINAL,
    Buff: PTEXTMODE_SCREEN_BUFFER,
    Buffer: PWCHAR,
    Length: DWORD,
    Attrib: BOOL,
) -> NTSTATUS {
    STATUS_PENDING
}

unsafe extern "system" fn DummyDrawRegion(
    This: PTERMINAL,
    Region: *mut SMALL_RECT,
) {
}

unsafe extern "system" fn DummySetCursorInfo(
    This: PTERMINAL,
    ScreenBuffer: PCONSOLE_SCREEN_BUFFER,
) -> BOOL {
    1
}

unsafe extern "system" fn DummySetScreenInfo(
    This: PTERMINAL,
    ScreenBuffer: PCONSOLE_SCREEN_BUFFER,
    OldCursorX: SHORT,
    OldCursorY: SHORT,
) -> BOOL {
    1
}

unsafe extern "system" fn DummyResizeTerminal(This: PTERMINAL) {
}

unsafe extern "system" fn DummySetActiveScreenBuffer(This: PTERMINAL) {
}

unsafe extern "system" fn DummyReleaseScreenBuffer(
    This: PTERMINAL,
    ScreenBuffer: PCONSOLE_SCREEN_BUFFER,
) {
}

unsafe extern "system" fn DummyGetLargestConsoleWindowSize(
    This: PTERMINAL,
    pSize: PCOORD,
) {
    if pSize.is_null() {
        return;
    }
    (*pSize).X = 80;
    (*pSize).Y = 25;
}

unsafe extern "system" fn DummySetPalette(
    This: PTERMINAL,
    PaletteHandle: HPALETTE,
    PaletteUsage: UINT,
) -> BOOL {
    1
}

unsafe extern "system" fn DummySetCodePage(
    This: PTERMINAL,
    CodePage: UINT,
) -> BOOL {
    1
}

unsafe extern "system" fn DummyShowMouseCursor(
    This: PTERMINAL,
    Show: BOOL,
) -> INT {
    0
}

static mut DummyVtbl: TERMINAL_VTBL = TERMINAL_VTBL {
    InitTerminal: Some(DummyInitTerminal),
    DeinitTerminal: Some(DummyDeinitTerminal),
    ReadStream: Some(DummyReadStream),
    WriteStream: Some(DummyWriteStream),
    DrawRegion: Some(DummyDrawRegion),
    SetCursorInfo: Some(DummySetCursorInfo),
    SetScreenInfo: Some(DummySetScreenInfo),
    ResizeTerminal: Some(DummyResizeTerminal),
    SetActiveScreenBuffer: Some(DummySetActiveScreenBuffer),
    ReleaseScreenBuffer: Some(DummyReleaseScreenBuffer),
    GetLargestConsoleWindowSize: Some(DummyGetLargestConsoleWindowSize),
    SetPalette: Some(DummySetPalette),
    SetCodePage: Some(DummySetCodePage),
    ShowMouseCursor: Some(DummyShowMouseCursor),
};

#[no_mangle]
pub unsafe extern "C" fn ResetTerminal(Console: PCONSOLE) {
    if Console.is_null() {
        return;
    }

    core::ptr::write_bytes(
        core::ptr::addr_of_mut!((*Console).TermIFace) as *mut u8,
        0,
        core::mem::size_of::<TERMINAL>(),
    );
    (*Console).TermIFace.Vtbl = core::ptr::addr_of_mut!(DummyVtbl);
}