use winsrv_sys::nt::*;
use super::super::condrv::text::*;
use crate::consrv::{ConsoleAllocHeap, ConsoleFreeHeap};
use winsrv_sys::winsrv::*;
use winsrv_sys::consts::*;

extern "C" {
    pub fn LineInputKeyDown(Console: PCONSRV_CONSOLE, ExeName: PUNICODE_STRING, KeyEvent: *mut KEY_EVENT_RECORD);
    pub fn memcpy(dest: PVOID, src: PVOID, count: SIZE_T) -> PVOID;
}
// _InterlockedDecrement is the stdcall MSVC intrinsic — consrv/shim.rs provides
// a single `_InterlockedDecrement` (i386 `__InterlockedDecrement@4`); declare it
// `extern "system"` to match (coninput.rs declares it the same way), rather than
// the cdecl twin the old C shim carried just for this call site.
extern "system" {
    pub fn _InterlockedDecrement(Target: *mut LONG) -> LONG;
}

// GUI frontend (Phase 5.6 — not yet translated). Stub extern so the
// FrontEndLoadingMethods static and ConSrvLoadFrontEnd dispatch can
// reference these by name; consrv-gui-rust (or the C consrv-gui.dll)
// will define them.
extern "system" {
    pub fn GuiLoadFrontEnd(
        FrontEnd: PFRONTEND,
        ConsoleInfo: PCONSOLE_STATE_INFO,
        ConsoleInitInfo: PCONSOLE_INIT_INFO,
        ConsoleLeaderProcessHandle: HANDLE,
    ) -> NTSTATUS;
    pub fn GuiUnloadFrontEnd(FrontEnd: PFRONTEND) -> NTSTATUS;
}
// TermDrawRegion is a TERMINAL_VTBL dispatch helper (term.h #define in C);
// use the sibling-Rust inline implementation in consrv::term rather than an
// extern symbol (there is no exported `_TermDrawRegion@N`).
use super::super::term::TermDrawRegion;

// ---- file-scope preamble (translated) ----
pub type FRONTEND_LOAD = Option<
    unsafe extern "system" fn(
        FrontEnd: PFRONTEND,
        ConsoleInfo: PCONSOLE_STATE_INFO,
        ConsoleInitInfo: PCONSOLE_INIT_INFO,
        ConsoleLeaderProcessHandle: HANDLE,
    ) -> NTSTATUS,
>;

pub type FRONTEND_UNLOAD = Option<
    unsafe extern "system" fn(
        FrontEnd: PFRONTEND,
    ) -> NTSTATUS,
>;

#[repr(C)]
pub struct FrontEndLoadingMethod {
    pub FrontEndName: [CHAR; 80],
    pub FrontEndLoad: FRONTEND_LOAD,
    pub FrontEndUnload: FRONTEND_UNLOAD,
}

pub static mut FrontEndLoadingMethods: [FrontEndLoadingMethod; 1] = [
    FrontEndLoadingMethod {
        FrontEndName: [
            b'G' as CHAR, b'U' as CHAR, b'I' as CHAR, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0,
        ],
        FrontEndLoad: Some(GuiLoadFrontEnd),
        FrontEndUnload: Some(GuiUnloadFrontEnd),
    },
];

// `TERMINAL_VTBL` field types use `extern "stdcall"` on i386 and
// `extern "C"` on amd64/arm64 (per bindgen translating __stdcall),
// while our ConSrvTerm* fns are `extern "system"` (same ABI per arch,
// distinct Rust type). transmute is not const-stable for these
// zero-sized fn items, so we lazy-init the vtable at first load via
// the `*const ()` runtime-transmute pattern (mirrors DummyVtbl in
// consrv/condrv/dummyterm.rs).
pub static mut ConSrvTermVtbl: TERMINAL_VTBL = TERMINAL_VTBL {
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

pub(crate) unsafe fn InitConSrvTermVtblIfNeeded() {
    if ConSrvTermVtbl.InitTerminal.is_some() {
        return;
    }
    ConSrvTermVtbl = TERMINAL_VTBL {
        InitTerminal: Some(core::mem::transmute(ConSrvTermInitTerminal as *const ())),
        DeinitTerminal: Some(core::mem::transmute(ConSrvTermDeinitTerminal as *const ())),
        ReadStream: Some(core::mem::transmute(ConSrvTermReadStream as *const ())),
        WriteStream: Some(core::mem::transmute(ConSrvTermWriteStream as *const ())),
        DrawRegion: Some(core::mem::transmute(ConSrvTermDrawRegion as *const ())),
        SetCursorInfo: Some(core::mem::transmute(ConSrvTermSetCursorInfo as *const ())),
        SetScreenInfo: Some(core::mem::transmute(ConSrvTermSetScreenInfo as *const ())),
        ResizeTerminal: Some(core::mem::transmute(ConSrvTermResizeTerminal as *const ())),
        SetActiveScreenBuffer: Some(core::mem::transmute(ConSrvTermSetActiveScreenBuffer as *const ())),
        ReleaseScreenBuffer: Some(core::mem::transmute(ConSrvTermReleaseScreenBuffer as *const ())),
        GetLargestConsoleWindowSize: Some(core::mem::transmute(ConSrvTermGetLargestConsoleWindowSize as *const ())),
        SetPalette: Some(core::mem::transmute(ConSrvTermSetPalette as *const ())),
        SetCodePage: Some(core::mem::transmute(ConSrvTermSetCodePage as *const ())),
        ShowMouseCursor: Some(core::mem::transmute(ConSrvTermShowMouseCursor as *const ())),
    };
}

pub const TAB_WIDTH: i32 = 8;

pub(crate) unsafe fn ConioInputEventToAnsi(Console: PCONSOLE, InputEvent: PINPUT_RECORD) {
    if (*InputEvent).EventType == KEY_EVENT as u16 {
        let UnicodeChar: u16 = (*InputEvent).Event.KeyEvent.uChar.UnicodeChar;
        (*InputEvent).Event.KeyEvent.uChar.UnicodeChar = 0;
        
        let mut AnsiChar: u8 = 0;
        WideCharToMultiByte(
            (*Console).InputCodePage,
            0,
            &UnicodeChar as *const u16,
            1,
            &mut AnsiChar as *mut u8 as *mut i8,
            1,
            core::ptr::null(),
            core::ptr::null_mut()
        );
        (*InputEvent).Event.KeyEvent.uChar.AsciiChar = AnsiChar as i8;
    }
}

pub(crate) unsafe fn ConioInputEventToUnicode(Console: PCONSOLE, InputEvent: PINPUT_RECORD) {
    if (*InputEvent).EventType == KEY_EVENT as u16 {
        let AsciiChar: CHAR = (*InputEvent).Event.KeyEvent.uChar.AsciiChar;
        (*InputEvent).Event.KeyEvent.uChar.AsciiChar = 0;
        MultiByteToWideChar(
            (*Console).InputCodePage,
            0,
            &AsciiChar,
            1,
            &mut (*InputEvent).Event.KeyEvent.uChar.UnicodeChar,
            1,
        );
    }
}

pub(crate) unsafe fn ConSrvLoadFrontEnd(
    FrontEnd: PFRONTEND,
    ConsoleInfo: PCONSOLE_STATE_INFO,
    ConsoleInitInfo: PCONSOLE_INIT_INFO,
    ConsoleLeaderProcessHandle: HANDLE,
) -> NTSTATUS {
    let mut Status: NTSTATUS = STATUS_SUCCESS;
    let mut i: ULONG;

    i = 0;
    while i < (core::mem::size_of_val(&FrontEndLoadingMethods) / core::mem::size_of::<(
        [CHAR; 80],
        FRONTEND_LOAD,
        FRONTEND_UNLOAD,
    )>()) as ULONG
    {
        Status = (FrontEndLoadingMethods[i as usize].FrontEndLoad.unwrap())(
            FrontEnd,
            ConsoleInfo,
            ConsoleInitInfo,
            ConsoleLeaderProcessHandle,
        );
        if NT_SUCCESS(Status) {
            (*FrontEnd).UnloadFrontEnd = core::mem::transmute(FrontEndLoadingMethods[i as usize].FrontEndUnload);
            break;
        }
        i += 1;
    }

    Status
}

pub(crate) unsafe fn ConSrvUnloadFrontEnd(FrontEnd: PFRONTEND) -> i32 {
    if FrontEnd.is_null() {
        return 0xC000000Du32 as i32;
    }
    ((*FrontEnd).UnloadFrontEnd.unwrap())(FrontEnd)
}

#[no_mangle]
pub unsafe extern "system" fn ConSrvInitTerminal(
    Terminal: PTERMINAL,
    ConsoleInfo: PCONSOLE_STATE_INFO,
    ConsoleInitInfo: PCONSOLE_INIT_INFO,
    ConsoleLeaderProcessHandle: HANDLE,
) -> NTSTATUS {
    let Status: NTSTATUS;
    let FrontEnd: PFRONTEND;

    FrontEnd = ConsoleAllocHeap(HEAP_ZERO_MEMORY, core::mem::size_of::<_FRONTEND>()) as PFRONTEND;
    if FrontEnd.is_null() {
        return STATUS_NO_MEMORY;
    }

    Status = ConSrvLoadFrontEnd(
        FrontEnd,
        ConsoleInfo,
        ConsoleInitInfo,
        ConsoleLeaderProcessHandle,
    );
    if !NT_SUCCESS(Status) {
        ConsoleFreeHeap(FrontEnd as PVOID);
        return Status;
    }

    InitConSrvTermVtblIfNeeded();
    (*Terminal).Vtbl = core::ptr::addr_of_mut!(ConSrvTermVtbl);
    (*Terminal).Context = FrontEnd as PVOID;

    STATUS_SUCCESS
}

pub unsafe extern "system" fn ConSrvDeinitTerminal(Terminal: PTERMINAL) -> i32 {
    let mut Status: i32 = STATUS_SUCCESS;
    let FrontEnd: PFRONTEND = (*Terminal).Context as PFRONTEND;

    (*Terminal).Context = core::ptr::null_mut();
    (*Terminal).Vtbl = core::ptr::null_mut();

    if !FrontEnd.is_null() {
        Status = ConSrvUnloadFrontEnd(FrontEnd);
        ConsoleFreeHeap(FrontEnd as *mut core::ffi::c_void);
    }

    Status
}

pub(crate) unsafe extern "system" fn ConSrvTermInitTerminal(
    This: PTERMINAL,
    Console: PCONSOLE,
) -> NTSTATUS {
    let FrontEnd: PFRONTEND = (*This).Context as PFRONTEND;
    let ConSrvConsole: PCONSRV_CONSOLE = Console as PCONSRV_CONSOLE;

    (*FrontEnd).Console = ConSrvConsole;

    core::ptr::copy_nonoverlapping(
        FrontEnd as *const u8,
        core::ptr::addr_of_mut!((*ConSrvConsole).FrontEndIFace) as *mut u8,
        core::mem::size_of::<_FRONTEND>(),
    );

    let Status = ((*(*FrontEnd).Vtbl).InitFrontEnd.unwrap())(FrontEnd, ConSrvConsole);
    if !NT_SUCCESS(Status) {
        // DPRINT1 omitted
    }

    core::ptr::copy_nonoverlapping(
        FrontEnd as *const u8,
        core::ptr::addr_of_mut!((*ConSrvConsole).FrontEndIFace) as *mut u8,
        core::mem::size_of::<_FRONTEND>(),
    );

    Status
}

pub(crate) unsafe extern "system" fn ConSrvTermDeinitTerminal(This: PTERMINAL) {
    let FrontEnd: PFRONTEND = (*This).Context as PFRONTEND;
    ((*(*FrontEnd).Vtbl).DeinitFrontEnd.unwrap())(FrontEnd);
}

pub(crate) unsafe extern "system" fn ConSrvTermReadStream(
    This: PTERMINAL,
    Unicode: BOOLEAN,
    Buffer: PVOID,
    ReadControl: PCONSOLE_READCONSOLE_CONTROL,
    Parameter: PVOID,
    NumCharsToRead: ULONG,
    NumCharsRead: PULONG,
) -> NTSTATUS {

    let FrontEnd = (*This).Context as PFRONTEND;
    let Console = (*FrontEnd).Console;
    let InputBuffer = core::ptr::addr_of_mut!((*Console).InputBuffer);
    let ExeName = Parameter as PUNICODE_STRING;

    let mut Status = STATUS_PENDING;

    let mut CurrentEntry: PLIST_ENTRY;
    let mut Input: *mut ConsoleInput;
    let mut i: ULONG = 0;

    if (*InputBuffer).Mode & ENABLE_LINE_INPUT as u16 != 0 {
        if (*Console).LineBuffer.is_null() {
            (*Console).LineMaxSize = if 256 > NumCharsToRead { 256 } else { NumCharsToRead };

            (*ReadControl).nInitialChars = if (*ReadControl).nInitialChars < NumCharsToRead {
                (*ReadControl).nInitialChars
            } else {
                NumCharsToRead
            };

            (*Console).LineBuffer = ConsoleAllocHeap(0, ((*Console).LineMaxSize as usize) * core::mem::size_of::<WCHAR>()) as *mut WCHAR;
            if (*Console).LineBuffer.is_null() {
                return STATUS_NO_MEMORY;
            }

            (*Console).LinePos = (*ReadControl).nInitialChars;
            (*Console).LineSize = (*ReadControl).nInitialChars;
            (*Console).LineComplete = 0u8;
            (*Console).LineUpPressed = 0u8;
            (*Console).LineInsertToggle = (*Console).InsertMode;
            (*Console).LineWakeupMask = (*ReadControl).dwCtrlWakeupMask;

            memcpy(
                (*Console).LineBuffer as PVOID,
                Buffer,
                (((*Console).LineSize as usize) * core::mem::size_of::<WCHAR>()) as SIZE_T,
            );
            if (*Console).LineSize >= (*Console).LineMaxSize {
                (*Console).LineComplete = 1u8;
                (*Console).LinePos = 0;
            }
        }

        while (*Console).LineComplete == 0 && IsListEmpty(core::ptr::addr_of_mut!((*InputBuffer).InputEvents)) == 0 {
            _InterlockedDecrement(core::ptr::addr_of_mut!((*InputBuffer).NumberOfEvents) as *mut LONG);
            CurrentEntry = RemoveHeadList(core::ptr::addr_of_mut!((*InputBuffer).InputEvents));
            if IsListEmpty(core::ptr::addr_of_mut!((*InputBuffer).InputEvents)) != 0 {
                NtClearEvent((*InputBuffer).ActiveEvent);
            }
            Input = (CurrentEntry as usize - core::mem::offset_of!(ConsoleInput, ListEntry)) as *mut ConsoleInput;

            if (*Input).InputEvent.EventType == KEY_EVENT as u16 && (*Input).InputEvent.Event.KeyEvent.bKeyDown != 0
            {
                LineInputKeyDown(Console, ExeName, core::ptr::addr_of_mut!((*Input).InputEvent.Event.KeyEvent));
                (*ReadControl).dwControlKeyState = (*Input).InputEvent.Event.KeyEvent.dwControlKeyState;
            }
            ConsoleFreeHeap(Input as PVOID);
        }

        if (*Console).LineComplete != 0 {
            while i < NumCharsToRead && (*Console).LinePos < (*Console).LineSize {
                let Char = *(*Console).LineBuffer.add((*Console).LinePos as usize);
                (*Console).LinePos += 1;

                if Unicode != 0 {
                    *(Buffer as *mut WCHAR).add(i as usize) = Char;
                } else {
                    WideCharToMultiByte(
                        (*Console).InputCodePage,
                        0,
                        core::ptr::addr_of!(Char),
                        1,
                        (Buffer as *mut CHAR).add(i as usize),
                        1,
                        core::ptr::null(),
                        core::ptr::null_mut(),
                    );
                }
                i += 1;
            }

            if (*Console).LinePos >= (*Console).LineSize {
                ConsoleFreeHeap((*Console).LineBuffer as PVOID);
                (*Console).LineBuffer = core::ptr::null_mut();
                (*Console).LinePos = 0;
                (*Console).LineMaxSize = 0;
                (*Console).LineSize = 0;
                (*Console).LineComplete = 0u8;
            }

            Status = STATUS_SUCCESS;
        }
    } else {
        while i < NumCharsToRead && IsListEmpty(core::ptr::addr_of_mut!((*InputBuffer).InputEvents)) == 0 {
            _InterlockedDecrement(core::ptr::addr_of_mut!((*InputBuffer).NumberOfEvents) as *mut LONG);
            CurrentEntry = RemoveHeadList(core::ptr::addr_of_mut!((*InputBuffer).InputEvents));
            if IsListEmpty(core::ptr::addr_of_mut!((*InputBuffer).InputEvents)) != 0 {
                NtClearEvent((*InputBuffer).ActiveEvent);
            }
            Input = (CurrentEntry as usize - core::mem::offset_of!(ConsoleInput, ListEntry)) as *mut ConsoleInput;

            if (*Input).InputEvent.EventType == KEY_EVENT as u16 && (*Input).InputEvent.Event.KeyEvent.bKeyDown != 0
                && (*Input).InputEvent.Event.KeyEvent.uChar.UnicodeChar != 0
            {
                let Char = (*Input).InputEvent.Event.KeyEvent.uChar.UnicodeChar;

                if Unicode != 0 {
                    *(Buffer as *mut WCHAR).add(i as usize) = Char;
                } else {
                    WideCharToMultiByte(
                        (*Console).InputCodePage,
                        0,
                        core::ptr::addr_of!(Char),
                        1,
                        (Buffer as *mut CHAR).add(i as usize),
                        1,
                        core::ptr::null(),
                        core::ptr::null_mut(),
                    );
                }
                i += 1;

                Status = STATUS_SUCCESS;
            }
            ConsoleFreeHeap(Input as PVOID);
        }
    }

    if !NumCharsRead.is_null() {
        *NumCharsRead = i;
    }

    Status
}

pub(crate) unsafe fn ConioNextLine(
    Buff: PTEXTMODE_SCREEN_BUFFER,
    UpdateRect: PSMALL_RECT,
    ScrolledLines: PUINT,
) {
    (*Buff).CursorPosition.Y += 1;
    if (*Buff).CursorPosition.Y == (*Buff).ScreenBufferSize.Y {
        (*Buff).CursorPosition.Y -= 1;
        (*Buff).VirtualY += 1;
        if (*Buff).VirtualY == (*Buff).ScreenBufferSize.Y as u16 {
            (*Buff).VirtualY = 0;
        }
        *ScrolledLines += 1;
        ClearLineBuffer(Buff);
        if (*UpdateRect).Top != 0 {
            (*UpdateRect).Top -= 1;
        }
    }
    (*UpdateRect).Left = 0;
    (*UpdateRect).Right = (*Buff).ScreenBufferSize.X - 1;
    (*UpdateRect).Bottom = (*Buff).CursorPosition.Y;
}

pub(crate) unsafe fn ConioWriteConsole(
    FrontEnd: PFRONTEND,
    Buff: PTEXTMODE_SCREEN_BUFFER,
    Buffer: *mut u16,
    Length: u32,
    Attrib: u8,
) -> i32 {
    let Console: PCONSRV_CONSOLE = (*FrontEnd).Console;

    let mut i: u32;
    let mut Ptr: PCHAR_INFO;
    let mut UpdateRect: SMALL_RECT = core::mem::zeroed();
    let mut CursorStartX: i16;
    let CursorStartY: i16;
    let mut ScrolledLines: u32;
    let mut bFullwidth: u8;
    let bCJK: u8 = (*Console).IsCJK;

    if Length == 0 {
        return STATUS_SUCCESS;
    }

    CursorStartX = (*Buff).CursorPosition.X;
    CursorStartY = (*Buff).CursorPosition.Y;
    UpdateRect.Left = (*Buff).ScreenBufferSize.X;
    UpdateRect.Top = (*Buff).CursorPosition.Y;
    UpdateRect.Right = -1;
    UpdateRect.Bottom = (*Buff).CursorPosition.Y;
    ScrolledLines = 0;

    i = 0;
    while i < Length {
        if (*Buff).Mode & ENABLE_PROCESSED_OUTPUT as u16 != 0 {
            if *Buffer.add(i as usize) == 0x000D {
                (*Buff).CursorPosition.X = 0;
                CursorStartX = (*Buff).CursorPosition.X;
                UpdateRect.Left = min(UpdateRect.Left, (*Buff).CursorPosition.X);
                UpdateRect.Right = max(UpdateRect.Right, (*Buff).CursorPosition.X);
                i += 1;
                continue;
            } else if *Buffer.add(i as usize) == 0x000A {
                (*Buff).CursorPosition.X = 0;
                CursorStartX = (*Buff).CursorPosition.X;
                ConioNextLine(Buff, &mut UpdateRect, &mut ScrolledLines);
                i += 1;
                continue;
            } else if *Buffer.add(i as usize) == 0x0008 {
                if (*Buff).CursorPosition.X == 0 && (*Buff).CursorPosition.Y == 0 {
                    i += 1;
                    continue;
                }

                if (*Buff).CursorPosition.X == 0 {
                    (*Buff).CursorPosition.X = (*Buff).ScreenBufferSize.X - 1;
                    (*Buff).CursorPosition.Y -= 1;
                    UpdateRect.Top = min(UpdateRect.Top, (*Buff).CursorPosition.Y);
                } else {
                    (*Buff).CursorPosition.X -= 1;
                }
                Ptr = ConioCoordToPointer(Buff, (*Buff).CursorPosition.X as u32, (*Buff).CursorPosition.Y as u32);

                if (*Ptr).Attributes & COMMON_LVB_LEADING_BYTE != 0 {
                    if (*Buff).CursorPosition.X == 0 && (*Buff).CursorPosition.Y == 0 {
                        i += 1;
                        continue;
                    }

                    if (*Buff).CursorPosition.X == 0 {
                        (*Buff).CursorPosition.X = (*Buff).ScreenBufferSize.X - 1;
                        (*Buff).CursorPosition.Y -= 1;
                        UpdateRect.Top = min(UpdateRect.Top, (*Buff).CursorPosition.Y);
                    } else {
                        (*Buff).CursorPosition.X -= 1;
                    }
                    Ptr = ConioCoordToPointer(Buff, (*Buff).CursorPosition.X as u32, (*Buff).CursorPosition.Y as u32);
                }

                if (*Ptr).Attributes & COMMON_LVB_TRAILING_BYTE != 0 {
                    (*Ptr).Char.UnicodeChar = 0x0020;
                    if Attrib != 0 {
                        (*Ptr).Attributes = (*Buff).ScreenDefaultAttrib;
                    }
                    (*Ptr).Attributes &= !COMMON_LVB_SBCSDBCS;

                    if (*Buff).CursorPosition.X > 0 {
                        (*Buff).CursorPosition.X -= 1;
                    }
                    Ptr = ConioCoordToPointer(Buff, (*Buff).CursorPosition.X as u32, (*Buff).CursorPosition.Y as u32);
                }

                (*Ptr).Char.UnicodeChar = 0x0020;
                if Attrib != 0 {
                    (*Ptr).Attributes = (*Buff).ScreenDefaultAttrib;
                }
                (*Ptr).Attributes &= !COMMON_LVB_SBCSDBCS;

                UpdateRect.Left = min(UpdateRect.Left, (*Buff).CursorPosition.X);
                UpdateRect.Right = max(UpdateRect.Right, (*Buff).CursorPosition.X);
                i += 1;
                continue;
            } else if *Buffer.add(i as usize) == 0x0009 {
                let mut EndX: u32;

                Ptr = ConioCoordToPointer(Buff, (*Buff).CursorPosition.X as u32, (*Buff).CursorPosition.Y as u32);

                if (*Ptr).Attributes & COMMON_LVB_TRAILING_BYTE != 0 {
                    if (*Buff).CursorPosition.X > 0 {
                        (*Buff).CursorPosition.X -= 1;
                    }
                    Ptr = ConioCoordToPointer(Buff, (*Buff).CursorPosition.X as u32, (*Buff).CursorPosition.Y as u32);
                }

                UpdateRect.Left = min(UpdateRect.Left, (*Buff).CursorPosition.X);

                EndX = ((*Buff).CursorPosition.X as u32 + 8) & !(8 - 1);
                EndX = min(EndX, (*Buff).ScreenBufferSize.X as u32);

                while ((*Buff).CursorPosition.X as u32) < EndX {
                    (*Ptr).Char.UnicodeChar = 0x0020;
                    if Attrib != 0 {
                        (*Ptr).Attributes = (*Buff).ScreenDefaultAttrib;
                    }
                    (*Ptr).Attributes &= !COMMON_LVB_SBCSDBCS;

                    Ptr = Ptr.add(1);
                    (*Buff).CursorPosition.X += 1;
                }
                if (*Buff).CursorPosition.X < (*Buff).ScreenBufferSize.X {
                    if (*Ptr).Attributes & COMMON_LVB_TRAILING_BYTE != 0 {
                        (*Ptr).Char.UnicodeChar = 0x0020;
                        if Attrib != 0 {
                            (*Ptr).Attributes = (*Buff).ScreenDefaultAttrib;
                        }
                        (*Ptr).Attributes &= !COMMON_LVB_SBCSDBCS;
                    }
                }
                UpdateRect.Right = max(UpdateRect.Right, (*Buff).CursorPosition.X);

                if (*Buff).CursorPosition.X >= (*Buff).ScreenBufferSize.X {
                    if (*Buff).Mode & ENABLE_WRAP_AT_EOL_OUTPUT as u16 != 0 {
                        (*Buff).CursorPosition.X = 0;
                        CursorStartX = (*Buff).CursorPosition.X;
                        ConioNextLine(Buff, &mut UpdateRect, &mut ScrolledLines);
                    } else {
                        (*Buff).CursorPosition.X = CursorStartX;
                    }
                }
                i += 1;
                continue;
            } else if *Buffer.add(i as usize) == 0x0007 {
                ((*(*FrontEnd).Vtbl).RingBell.unwrap())(FrontEnd);
                i += 1;
                continue;
            }
        }
        UpdateRect.Left = min(UpdateRect.Left, (*Buff).CursorPosition.X);
        UpdateRect.Right = max(UpdateRect.Right, (*Buff).CursorPosition.X);

        bFullwidth = if bCJK != 0 && IS_FULL_WIDTH(*Buffer.add(i as usize)) != 0 { 1 } else { 0 };

        if bFullwidth != 0 {
            if (*Buff).CursorPosition.X >= (*Buff).ScreenBufferSize.X - 1 {
                if (*Buff).Mode & ENABLE_WRAP_AT_EOL_OUTPUT as u16 != 0 {
                    (*Buff).CursorPosition.X = 0;
                    CursorStartX = (*Buff).CursorPosition.X;
                    ConioNextLine(Buff, &mut UpdateRect, &mut ScrolledLines);
                } else {
                    (*Buff).CursorPosition.X = CursorStartX;
                }
            }

            if (*Buff).CursorPosition.X >= (*Buff).ScreenBufferSize.X - 1 {
                i += 1;
                continue;
            }
        }

        Ptr = ConioCoordToPointer(Buff, (*Buff).CursorPosition.X as u32, (*Buff).CursorPosition.Y as u32);

        if (*Ptr).Attributes & COMMON_LVB_TRAILING_BYTE != 0 {
            if (*Buff).CursorPosition.X > 0 {
                Ptr = ConioCoordToPointer(Buff, ((*Buff).CursorPosition.X - 1) as u32, (*Buff).CursorPosition.Y as u32);
                (*Ptr).Char.UnicodeChar = 0x0020;
                if Attrib != 0 {
                    (*Ptr).Attributes = (*Buff).ScreenDefaultAttrib;
                }
                (*Ptr).Attributes &= !COMMON_LVB_SBCSDBCS;
            }
            Ptr = ConioCoordToPointer(Buff, (*Buff).CursorPosition.X as u32, (*Buff).CursorPosition.Y as u32);
        }

        if bFullwidth != 0 {
            (*Ptr).Char.UnicodeChar = *Buffer.add(i as usize);
            if Attrib != 0 {
                (*Ptr).Attributes = (*Buff).ScreenDefaultAttrib;
            }
            (*Ptr).Attributes &= !COMMON_LVB_SBCSDBCS;
            (*Ptr).Attributes |= COMMON_LVB_LEADING_BYTE;

            (*Buff).CursorPosition.X += 1;
            Ptr = ConioCoordToPointer(Buff, (*Buff).CursorPosition.X as u32, (*Buff).CursorPosition.Y as u32);
            if Attrib != 0 {
                (*Ptr).Attributes = (*Buff).ScreenDefaultAttrib;
            }
            (*Ptr).Attributes &= !COMMON_LVB_SBCSDBCS;
            (*Ptr).Attributes |= COMMON_LVB_TRAILING_BYTE;
        } else {
            (*Ptr).Char.UnicodeChar = *Buffer.add(i as usize);
            if Attrib != 0 {
                (*Ptr).Attributes = (*Buff).ScreenDefaultAttrib;
            }
            (*Ptr).Attributes &= !COMMON_LVB_SBCSDBCS;
        }

        Ptr = Ptr.add(1);
        (*Buff).CursorPosition.X += 1;

        if (*Buff).CursorPosition.X < (*Buff).ScreenBufferSize.X {
            if (*Ptr).Attributes & COMMON_LVB_TRAILING_BYTE != 0 {
                (*Ptr).Char.UnicodeChar = 0x0020;
                if Attrib != 0 {
                    (*Ptr).Attributes = (*Buff).ScreenDefaultAttrib;
                }
                (*Ptr).Attributes &= !COMMON_LVB_SBCSDBCS;
            }
        }

        if (*Buff).CursorPosition.X >= (*Buff).ScreenBufferSize.X {
            if (*Buff).Mode & ENABLE_WRAP_AT_EOL_OUTPUT as u16 != 0 {
                (*Buff).CursorPosition.X = 0;
                CursorStartX = (*Buff).CursorPosition.X;
                ConioNextLine(Buff, &mut UpdateRect, &mut ScrolledLines);
            } else {
                (*Buff).CursorPosition.X = CursorStartX;
            }
        }

        i += 1;
    }

    if ConioIsRectEmpty(&mut UpdateRect) == 0 && Buff as PCONSOLE_SCREEN_BUFFER == (*Console).ActiveBuffer {
        ((*(*FrontEnd).Vtbl).WriteStream.unwrap())(
            FrontEnd,
            &mut UpdateRect,
            CursorStartX,
            CursorStartY,
            ScrolledLines,
            Buffer,
            Length,
        );
    }

    STATUS_SUCCESS
}

pub(crate) unsafe extern "system" fn ConSrvTermWriteStream(
    This: PTERMINAL,
    Buff: PTEXTMODE_SCREEN_BUFFER,
    Buffer: *mut u16,
    Length: u32,
    Attrib: BOOL,
) -> i32 {
    let FrontEnd: PFRONTEND = (*This).Context as PFRONTEND;
    ConioWriteConsole(FrontEnd, Buff, Buffer, Length, Attrib as u8)
}

pub(crate) unsafe fn ConioDrawConsole(Console: PCONSRV_CONSOLE) {
    let mut Region: SMALL_RECT = core::mem::zeroed();
    let ActiveBuffer: PCONSOLE_SCREEN_BUFFER = (*Console).ActiveBuffer;

    if ActiveBuffer.is_null() {
        return;
    }

    ConioInitRect(
        &mut Region,
        0,
        0,
        (*ActiveBuffer).ViewSize.Y - 1,
        (*ActiveBuffer).ViewSize.X - 1,
    );
    TermDrawRegion(Console, &mut Region);
}

pub(crate) unsafe extern "system" fn ConSrvTermDrawRegion(
    This: PTERMINAL,
    Region: *mut SMALL_RECT,
) {
    let FrontEnd: PFRONTEND = (*This).Context as PFRONTEND;
    ((*(*FrontEnd).Vtbl).DrawRegion.unwrap())(FrontEnd, Region);
}

pub(crate) unsafe extern "system" fn ConSrvTermSetCursorInfo(
    This: PTERMINAL,
    ScreenBuffer: PCONSOLE_SCREEN_BUFFER,
) -> BOOL {
    let FrontEnd: PFRONTEND = (*This).Context as PFRONTEND;
    ((*(*FrontEnd).Vtbl).SetCursorInfo.unwrap())(FrontEnd, ScreenBuffer)
}

pub(crate) unsafe extern "system" fn ConSrvTermSetScreenInfo(
    This: PTERMINAL,
    ScreenBuffer: PCONSOLE_SCREEN_BUFFER,
    OldCursorX: SHORT,
    OldCursorY: SHORT,
) -> BOOL {
    let FrontEnd = (*This).Context as PFRONTEND;
    ((*(*FrontEnd).Vtbl).SetScreenInfo.unwrap())(
        FrontEnd,
        ScreenBuffer,
        OldCursorX,
        OldCursorY,
    )
}

pub(crate) unsafe extern "system" fn ConSrvTermResizeTerminal(This: PTERMINAL) {
    let FrontEnd: PFRONTEND = (*This).Context as PFRONTEND;
    ((*(*FrontEnd).Vtbl).ResizeTerminal.unwrap())(FrontEnd);
}

pub(crate) unsafe extern "system" fn ConSrvTermSetActiveScreenBuffer(This: PTERMINAL) {
    let FrontEnd: PFRONTEND = (*This).Context as PFRONTEND;
    ((*(*FrontEnd).Vtbl).SetActiveScreenBuffer.unwrap())(FrontEnd);
}

pub(crate) unsafe extern "system" fn ConSrvTermReleaseScreenBuffer(
    This: PTERMINAL,
    ScreenBuffer: PCONSOLE_SCREEN_BUFFER,
) {
    let FrontEnd: PFRONTEND = (*This).Context as PFRONTEND;
    let vtbl = (*FrontEnd).Vtbl;
    ((*vtbl).ReleaseScreenBuffer.unwrap())(FrontEnd, ScreenBuffer);
}

pub(crate) unsafe extern "system" fn ConSrvTermGetLargestConsoleWindowSize(
    This: PTERMINAL,
    pSize: PCOORD,
) {
    let FrontEnd: PFRONTEND = (*This).Context as PFRONTEND;
    ((*(*FrontEnd).Vtbl).GetLargestConsoleWindowSize.unwrap())(FrontEnd, pSize);
}

pub(crate) unsafe extern "system" fn ConSrvTermSetPalette(
    This: PTERMINAL,
    PaletteHandle: HPALETTE,
    PaletteUsage: UINT,
) -> BOOL {
    let FrontEnd: PFRONTEND = (*This).Context as PFRONTEND;
    ((*(*FrontEnd).Vtbl).SetPalette.unwrap())(FrontEnd, PaletteHandle, PaletteUsage)
}

pub(crate) unsafe extern "system" fn ConSrvTermSetCodePage(
    This: PTERMINAL,
    CodePage: UINT,
) -> BOOL {
    let FrontEnd = (*This).Context as PFRONTEND;
    ((*(*FrontEnd).Vtbl).SetCodePage.unwrap())(FrontEnd, CodePage)
}

pub(crate) unsafe extern "system" fn ConSrvTermShowMouseCursor(
    This: PTERMINAL,
    Show: BOOL,
) -> INT {
    let FrontEnd: PFRONTEND = (*This).Context as PFRONTEND;
    ((*(*FrontEnd).Vtbl).ShowMouseCursor.unwrap())(FrontEnd, Show)
}

pub(crate) unsafe fn ResetFrontEnd(Console: PCONSOLE) {
    let ConSrvConsole: PCONSRV_CONSOLE = Console as PCONSRV_CONSOLE;
    if Console.is_null() {
        return;
    }

    RtlZeroMemory(
        core::ptr::addr_of_mut!((*ConSrvConsole).FrontEndIFace) as *mut core::ffi::c_void,
        core::mem::size_of_val(&(*ConSrvConsole).FrontEndIFace),
    );
    InitConSrvTermVtblIfNeeded();
    // C source: `ConSrvConsole->FrontEndIFace.Vtbl = &ConSrvTermVtbl;` —
    // FrontEndIFace.Vtbl is PFRONTEND_VTBL but ConSrvTermVtbl is
    // TERMINAL_VTBL. C's permissive pointer cast is benign here; bridge
    // explicitly in Rust.
    (*ConSrvConsole).FrontEndIFace.Vtbl = core::ptr::addr_of_mut!(ConSrvTermVtbl) as PFRONTEND_VTBL;
}
