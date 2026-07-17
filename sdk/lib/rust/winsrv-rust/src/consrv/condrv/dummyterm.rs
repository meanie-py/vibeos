use winsrv_sys::nt::*;
use winsrv_sys::winsrv::*;

unsafe extern "system" fn DummyInitTerminal(
    _This: PTERMINAL,
    _Console: PCONSOLE,
) -> NTSTATUS {
    STATUS_SUCCESS
}

unsafe extern "system" fn DummyDeinitTerminal(_This: PTERMINAL) {
}

unsafe extern "system" fn DummyReadStream(
    _This: PTERMINAL,
    _Unicode: BOOLEAN,
    _Buffer: PVOID,
    _ReadControl: PCONSOLE_READCONSOLE_CONTROL,
    _Parameter: PVOID,
    _NumCharsToRead: ULONG,
    _NumCharsRead: PULONG,
) -> NTSTATUS {
    STATUS_PENDING
}

unsafe extern "system" fn DummyWriteStream(
    _This: PTERMINAL,
    _Buff: PTEXTMODE_SCREEN_BUFFER,
    _Buffer: PWCHAR,
    _Length: DWORD,
    _Attrib: BOOL,
) -> NTSTATUS {
    STATUS_PENDING
}

unsafe extern "system" fn DummyDrawRegion(
    _This: PTERMINAL,
    _Region: *mut SMALL_RECT,
) {
}

unsafe extern "system" fn DummySetCursorInfo(
    _This: PTERMINAL,
    _ScreenBuffer: PCONSOLE_SCREEN_BUFFER,
) -> BOOL {
    1
}

unsafe extern "system" fn DummySetScreenInfo(
    _This: PTERMINAL,
    _ScreenBuffer: PCONSOLE_SCREEN_BUFFER,
    _OldCursorX: SHORT,
    _OldCursorY: SHORT,
) -> BOOL {
    1
}

unsafe extern "system" fn DummyResizeTerminal(_This: PTERMINAL) {
}

unsafe extern "system" fn DummySetActiveScreenBuffer(_This: PTERMINAL) {
}

unsafe extern "system" fn DummyReleaseScreenBuffer(
    _This: PTERMINAL,
    _ScreenBuffer: PCONSOLE_SCREEN_BUFFER,
) {
}

unsafe extern "system" fn DummyGetLargestConsoleWindowSize(
    _This: PTERMINAL,
    pSize: PCOORD,
) {
    if pSize.is_null() {
        return;
    }
    (*pSize).X = 80;
    (*pSize).Y = 25;
}

unsafe extern "system" fn DummySetPalette(
    _This: PTERMINAL,
    _PaletteHandle: HPALETTE,
    _PaletteUsage: UINT,
) -> BOOL {
    1
}

unsafe extern "system" fn DummySetCodePage(
    _This: PTERMINAL,
    _CodePage: UINT,
) -> BOOL {
    1
}

unsafe extern "system" fn DummyShowMouseCursor(
    _This: PTERMINAL,
    _Show: BOOL,
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