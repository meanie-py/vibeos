use winsrv_sys::nt::*;
use winsrv_sys::winsrv::*;
use winsrv_sys::consts::*;

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

// Compute the vtable lazily via a const-fn-style block. The bindgen
// `TERMINAL_VTBL` field types use `extern "stdcall"` on i386 and
// `extern "C"` on amd64/arm64 (per __stdcall macro), but our Dummy*
// functions are declared `extern "system"` (which resolves to the
// same ABI per-arch). Rust's type system sees them as distinct fn
// types, so we transmute each fn pointer into the bindgen-expected
// fn pointer type. transmute is not const, so DummyVtbl is built at
// runtime via a OnceCell-shape; for static-mut here we use a small
// init helper invoked lazily from ResetTerminal.
static mut DummyVtbl: TERMINAL_VTBL = TERMINAL_VTBL {
    InitTerminal: None,
    DeinitTerminal: None,
    ReadStream: None,
    WriteStream: None,
    DrawRegion: None,
    SetCursorInfo: None,
    SetScreenInfo: None,
    ResizeTerminal: None,
    SetActiveScreenBuffer: None,
    ReleaseScreenBuffer: None,
    GetLargestConsoleWindowSize: None,
    SetPalette: None,
    SetCodePage: None,
    ShowMouseCursor: None,
};

pub(crate) unsafe fn InitDummyVtblIfNeeded() {
    if DummyVtbl.InitTerminal.is_some() {
        return;
    }
    DummyVtbl = TERMINAL_VTBL {
        InitTerminal: Some(core::mem::transmute(DummyInitTerminal as *const ())),
        DeinitTerminal: Some(core::mem::transmute(DummyDeinitTerminal as *const ())),
        ReadStream: Some(core::mem::transmute(DummyReadStream as *const ())),
        WriteStream: Some(core::mem::transmute(DummyWriteStream as *const ())),
        DrawRegion: Some(core::mem::transmute(DummyDrawRegion as *const ())),
        SetCursorInfo: Some(core::mem::transmute(DummySetCursorInfo as *const ())),
        SetScreenInfo: Some(core::mem::transmute(DummySetScreenInfo as *const ())),
        ResizeTerminal: Some(core::mem::transmute(DummyResizeTerminal as *const ())),
        SetActiveScreenBuffer: Some(core::mem::transmute(DummySetActiveScreenBuffer as *const ())),
        ReleaseScreenBuffer: Some(core::mem::transmute(DummyReleaseScreenBuffer as *const ())),
        GetLargestConsoleWindowSize: Some(core::mem::transmute(DummyGetLargestConsoleWindowSize as *const ())),
        SetPalette: Some(core::mem::transmute(DummySetPalette as *const ())),
        SetCodePage: Some(core::mem::transmute(DummySetCodePage as *const ())),
        ShowMouseCursor: Some(core::mem::transmute(DummyShowMouseCursor as *const ())),
    };
}

#[no_mangle]
pub unsafe extern "C" fn ResetTerminal(Console: PCONSOLE) {
    if Console.is_null() {
        return;
    }

    InitDummyVtblIfNeeded();
    core::ptr::write_bytes(
        core::ptr::addr_of_mut!((*Console).TermIFace) as *mut u8,
        0,
        core::mem::size_of::<TERMINAL>(),
    );
    (*Console).TermIFace.Vtbl = core::ptr::addr_of_mut!(DummyVtbl);
}