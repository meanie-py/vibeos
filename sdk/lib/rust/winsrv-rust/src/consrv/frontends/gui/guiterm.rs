use winsrv_sys::nt::*;
use winsrv_sys::winsrv::*;
use winsrv_sys::consts::*;
use super::helpers::*;
use super::conwnd::*;
use super::fullscreen::*;

// w! — NUL-terminated UTF-16 array from an ASCII literal (2-aligned 'static;
// a b"...\0" byte string cast to *const u16 is both ANSI-not-wide and
// potentially odd-aligned — see field manual / basesrv init.rs CSDVersion).
macro_rules! w {
    ($s:literal) => {{
        const __W: &[u16] = &{
            const BYTES: &[u8] = $s.as_bytes();
            let mut out = [0u16; { BYTES.len() + 1 }];
            let mut i = 0;
            while i < BYTES.len() {
                out[i] = BYTES[i] as u16;
                i += 1;
            }
            out
        };
        __W
    }};
}


extern "C" {
    /// `InitTTFontCache`/`RefreshTTFontCache` — concfg-side helpers (cdecl;
    /// concfg exports the undecorated `_InitTTFontCache`).
    fn InitTTFontCache();
}
extern "system" {
    /// `GetType` — consrv-internal helper (def consrv/mod.rs, #[no_mangle]
    /// extern "system"). `MonitorFromWindow`/`ScrollWindowEx` — user32 stdcall.
    /// (`GetScreenBufferSizeUnits`/`SmallRectToRect` are now plain Rust helpers
    /// in super::helpers, resolved via `use super::helpers::*`.)
    fn MonitorFromWindow(hWnd: HWND, dwFlags: DWORD) -> HMONITOR;
    fn GetType(Buff: PCONSOLE_SCREEN_BUFFER) -> u32;
    fn ScrollWindowEx( hWnd: HWND, dx: i32, dy: i32, prcScroll: *const RECT, prcClip: *const RECT, hrgnUpdate: HRGN, prcUpdate: *mut RECT, flags: UINT, ) -> i32;
}
extern "system" {
    fn SendNotifyMessageW(hWnd: HWND, Msg: UINT, wParam: WPARAM, lParam: LPARAM) -> BOOL;
    fn CopyIcon(hIcon: HICON) -> HICON;
    fn SelectPalette(hdc: HDC, hpal: HPALETTE, bForceBkgd: BOOL) -> HPALETTE;
    fn RealizePalette(hdc: HDC) -> UINT;
    // InitFonts comes from super::conwnd::* with correct sig.
    fn EnableMenuItem(hMenu: HMENU, uIDEnableItem: UINT, uEnable: UINT) -> BOOL;
    fn ConsoleAllocHeap(Flags: ULONG, Size: SIZE_T) -> PVOID;
    fn ConsoleFreeHeap(HeapBase: winsrv_sys::winsrv::PVOID) -> winsrv_sys::winsrv::BOOLEAN; // def returns BOOLEAN — ret-void decl is a §9.99 fold
    fn GuiConsoleGetDefaultSettings(TermInfo: *mut GUI_CONSOLE_INFO);
    fn GuiConsoleReadUserSettings(TermInfo: *mut GUI_CONSOLE_INFO);
    // StringCchCopyNW is a plain Rust helper in super::helpers (via glob import).
}

// ---- file-scope preamble (translated) ----
pub const CONGUI_UPDATE_TIME: u32 = 0;

pub const CONGUI_UPDATE_TIMER: u32 = 1;

pub const PM_CREATE_CONSOLE: u32 = WM_APP + 1;

pub const PM_DESTROY_CONSOLE: u32 = WM_APP + 2;

#[repr(C)]
pub struct GUI_INIT_INFO {
    pub GuiThreadStartupEvent: HANDLE,
    pub InputThreadId: ULONG_PTR,
    pub WinSta: HWINSTA,
    pub Desktop: HDESK,
    pub hIcon: HICON,
    pub hIconSm: HICON,
    pub IsWindowVisible: BOOLEAN,
    pub TermInfo: GUI_CONSOLE_INFO,
}

pub type PGUI_INIT_INFO = *mut GUI_INIT_INFO;

pub static mut ConsInitialized: BOOL = 0;

// FRONTEND_VTBL slots are NTAPI: stdcall on i386, the C ABI elsewhere. Every
// implementation must be `extern "system"`, and the slots take them directly so
// rustc checks the ABI and signature. This table used to fill every slot through
// `transmute(f as *const ())` under a comment claiming the impls were already
// `extern "system"`. 22 of 25 were plain Rust-ABI fns and five returned u8 for
// C's BOOL (six, counting GuiSetScreenInfo). That call-compatibility happened to
// hold on amd64/arm64 and broke i386: csrss died (BUGCHECK 0xF4) creating its
// first GUI console.
pub static mut GuiVtbl: FRONTEND_VTBL = unsafe { FRONTEND_VTBL {
    InitFrontEnd: Some(GuiInitFrontEnd),
    DeinitFrontEnd: Some(GuiDeinitFrontEnd),
    DrawRegion: Some(GuiDrawRegion),
    WriteStream: Some(GuiWriteStream),
    RingBell: Some(GuiRingBell),
    SetCursorInfo: Some(GuiSetCursorInfo),
    SetScreenInfo: Some(GuiSetScreenInfo),
    ResizeTerminal: Some(GuiResizeTerminal),
    SetActiveScreenBuffer: Some(GuiSetActiveScreenBuffer),
    ReleaseScreenBuffer: Some(GuiReleaseScreenBuffer),
    RefreshInternalInfo: Some(GuiRefreshInternalInfo),
    ChangeTitle: Some(GuiChangeTitle),
    ChangeIcon: Some(GuiChangeIcon),
    GetThreadConsoleDesktop: Some(GuiGetThreadConsoleDesktop),
    GetConsoleWindowHandle: Some(GuiGetConsoleWindowHandle),
    GetLargestConsoleWindowSize: Some(GuiGetLargestConsoleWindowSize),
    GetSelectionInfo: Some(GuiGetSelectionInfo),
    SetPalette: Some(GuiSetPalette),
    SetCodePage: Some(GuiSetCodePage),
    GetDisplayMode: Some(GuiGetDisplayMode),
    SetDisplayMode: Some(GuiSetDisplayMode),
    ShowMouseCursor: Some(GuiShowMouseCursor),
    SetMouseCursor: Some(GuiSetMouseCursor),
    MenuControl: Some(GuiMenuControl),
    SetMenuClose: Some(GuiSetMenuClose),
    GetFontSize: Some(GuiGetFontSize),
} };

pub unsafe fn GuiConsoleMoveWindow(GuiData: PGUI_CONSOLE_DATA) {
    if (*GuiData).GuiInfo.AutoPosition == 0 {
        SetWindowPos(
            (*GuiData).hWindow,
            core::ptr::null_mut(),
            (*GuiData).GuiInfo.WindowOrigin.x,
            (*GuiData).GuiInfo.WindowOrigin.y,
            0,
            0,
            SWP_NOZORDER | SWP_NOSIZE | SWP_NOACTIVATE,
        );
    }
}

pub unsafe fn DrawRegion(GuiData: PGUI_CONSOLE_DATA, Region: *mut SMALL_RECT) {
    let mut RegionRect: RECT = core::mem::zeroed();

    SmallRectToRect(GuiData, &mut RegionRect, Region);
    InvalidateRect((*GuiData).hWindow, &RegionRect, 0);
}

pub unsafe fn InvalidateCell(GuiData: PGUI_CONSOLE_DATA, x: i16, y: i16) {
    let mut CellRect: SMALL_RECT = SMALL_RECT {
        Left: x,
        Top: y,
        Right: x,
        Bottom: y,
    };
    DrawRegion(GuiData, &mut CellRect);
}

pub unsafe extern "system" fn GuiConsoleInputThread(Param: PVOID) -> ULONG {
    let mut Status: NTSTATUS;
    let mut pcsrt: PCSR_THREAD = core::ptr::null_mut();
    let GuiInitInfo: PGUI_INIT_INFO = Param as PGUI_INIT_INFO;
    let mut DesktopConsoleThreadInfo: DESKTOP_CONSOLE_THREAD = core::mem::zeroed();
    let InputThreadId: ULONG_PTR = HandleToUlong((*winsrv_sys::nt::NtCurrentTeb()).ClientId.UniqueThread) as ULONG_PTR;
    let mut hThread: HANDLE = core::ptr::null_mut();

    let mut WindowCount: LONG = 0;
    let mut msg: MSG = core::mem::zeroed();

    DesktopConsoleThreadInfo.DesktopHandle = (*GuiInitInfo).Desktop;
    DesktopConsoleThreadInfo.ThreadId = InputThreadId;
    Status = NtUserConsoleControl(
        ConsoleCtrlDesktopConsoleThread,
        &mut DesktopConsoleThreadInfo as *mut _ as PVOID,
        core::mem::size_of::<DESKTOP_CONSOLE_THREAD>() as ULONG,
    );
    if !NT_SUCCESS(Status) {
        'Quit: {
            NtUserConsoleControl(
                ConsoleCtrlDesktopConsoleThread,
                &mut DesktopConsoleThreadInfo as *mut _ as PVOID,
                core::mem::size_of::<DESKTOP_CONSOLE_THREAD>() as ULONG,
            );

            CloseDesktop(DesktopConsoleThreadInfo.DesktopHandle);

            if !pcsrt.is_null() {
                CsrDereferenceThread(pcsrt);
            }

            RtlExitUserThread(Status);
            return 0;
        }
    }

    pcsrt = CsrConnectToUser();
    if pcsrt.is_null() {
        'Quit: {
            DesktopConsoleThreadInfo.ThreadId = 0;
            NtUserConsoleControl(
                ConsoleCtrlDesktopConsoleThread,
                &mut DesktopConsoleThreadInfo as *mut _ as PVOID,
                core::mem::size_of::<DESKTOP_CONSOLE_THREAD>() as ULONG,
            );

            CloseDesktop(DesktopConsoleThreadInfo.DesktopHandle);

            if !pcsrt.is_null() {
                CsrDereferenceThread(pcsrt);
            }

            RtlExitUserThread(Status);
            return 0;
        }
    }
    hThread = (*pcsrt).ThreadHandle;

    if SetThreadDesktop(DesktopConsoleThreadInfo.DesktopHandle) == 0 {
        'Quit: {
            DesktopConsoleThreadInfo.ThreadId = 0;
            NtUserConsoleControl(
                ConsoleCtrlDesktopConsoleThread,
                &mut DesktopConsoleThreadInfo as *mut _ as PVOID,
                core::mem::size_of::<DESKTOP_CONSOLE_THREAD>() as ULONG,
            );

            CloseDesktop(DesktopConsoleThreadInfo.DesktopHandle);

            if !pcsrt.is_null() {
                CsrDereferenceThread(pcsrt);
            }

            RtlExitUserThread(Status);
            return 0;
        }
    }

    NtSetEvent((*GuiInitInfo).GuiThreadStartupEvent, core::ptr::null_mut());
    Status = STATUS_SUCCESS;

    while GetMessageW(&mut msg, core::ptr::null_mut(), 0, 0) != 0 {
        match msg.message {
            PM_CREATE_CONSOLE => {
                let GuiData: PGUI_CONSOLE_DATA = msg.lParam as PGUI_CONSOLE_DATA;
                let Console: PCONSRV_CONSOLE = (*GuiData).Console;
                let NewWindow: HWND;
                let mut rcWnd: RECT = core::mem::zeroed();

                NewWindow = CreateWindowExW(
                    WS_EX_CLIENTEDGE,
                    GUI_CONWND_CLASS_W.as_ptr(),
                    (*Console).Title.Buffer,
                    WS_OVERLAPPEDWINDOW,
                    CW_USEDEFAULT,
                    CW_USEDEFAULT,
                    CW_USEDEFAULT,
                    CW_USEDEFAULT,
                    if (*GuiData).IsWindowVisible != 0 { HWND_DESKTOP } else { HWND_MESSAGE },
                    core::ptr::null_mut(),
                    ConSrvDllInstance,
                    GuiData as PVOID,
                );
                if NewWindow.is_null() {
                    continue;
                }

                _InterlockedIncrement(&mut WindowCount);

                GetWindowRect((*GuiData).hWindow, &mut rcWnd);
                (*GuiData).GuiInfo.WindowOrigin.x = rcWnd.left;
                (*GuiData).GuiInfo.WindowOrigin.y = rcWnd.top;

                if (*GuiData).IsWindowVisible != 0 {
                    GuiConsoleMoveWindow(GuiData);
                    SendMessageW((*GuiData).hWindow, PM_RESIZE_TERMINAL, 0, 0);
                }

                CreateSysMenu((*GuiData).hWindow);

                if (*GuiData).IsWindowVisible != 0 {
                    if (*GuiData).GuiInfo.FullScreen != 0 {
                        SwitchFullScreen(GuiData, 1);
                    }

                    ShowWindowAsync(NewWindow, (*GuiData).GuiInfo.ShowWindow as i32);
                } else {
                    ShowWindowAsync(NewWindow, SW_HIDE as i32);
                }

                continue;
            }

            PM_DESTROY_CONSOLE => {
                let GuiData: PGUI_CONSOLE_DATA = msg.lParam as PGUI_CONSOLE_DATA;
                let mut TempMsg: MSG = core::mem::zeroed();

                while PeekMessageW(&mut TempMsg, core::ptr::null_mut(), 0, 0, PM_REMOVE) != 0 {
                    DispatchMessageW(&TempMsg);
                }

                if (*GuiData).hWindow.is_null() {
                    continue;
                }

                DestroyWindow((*GuiData).hWindow);

                NtSetEvent((*GuiData).hGuiTermEvent, core::ptr::null_mut());

                if _InterlockedDecrement(&mut WindowCount) == 0 {
                    'Quit: {
                        DesktopConsoleThreadInfo.ThreadId = 0;
                        NtUserConsoleControl(
                            ConsoleCtrlDesktopConsoleThread,
                            &mut DesktopConsoleThreadInfo as *mut _ as PVOID,
                            core::mem::size_of::<DESKTOP_CONSOLE_THREAD>() as ULONG,
                        );

                        CloseDesktop(DesktopConsoleThreadInfo.DesktopHandle);

                        if !pcsrt.is_null() {
                            CsrDereferenceThread(pcsrt);
                        }

                        RtlExitUserThread(Status);
                        return 0;
                    }
                }

                continue;
            }

            _ => {}
        }

        TranslateMessage(&msg);
        DispatchMessageW(&msg);
    }

    DesktopConsoleThreadInfo.ThreadId = 0;
    NtUserConsoleControl(
        ConsoleCtrlDesktopConsoleThread,
        &mut DesktopConsoleThreadInfo as *mut _ as PVOID,
        core::mem::size_of::<DESKTOP_CONSOLE_THREAD>() as ULONG,
    );

    CloseDesktop(DesktopConsoleThreadInfo.DesktopHandle);

    if !pcsrt.is_null() {
        CsrDereferenceThread(pcsrt);
    }

    RtlExitUserThread(Status);
    0
}

pub unsafe fn GuiInit(
    ConsoleInitInfo: PCONSOLE_INIT_INFO,
    ConsoleLeaderProcessHandle: HANDLE,
    GuiInitInfo: PGUI_INIT_INFO,
) -> BOOL {
    let mut Success: BOOL = 1;
    let mut DesktopPath: _UNICODE_STRING = core::mem::zeroed();
    let mut DesktopConsoleThreadInfo: DESKTOP_CONSOLE_THREAD = core::mem::zeroed();
    let mut hWinSta: HWINSTA = core::ptr::null_mut();
    let hDesk: HDESK;

    let mut Status: NTSTATUS;
    let mut hInputThread: HANDLE = core::ptr::null_mut();
    let mut ClientId: _CLIENT_ID = core::mem::zeroed();

    if ConsInitialized == 0 {
        if RegisterConWndClass(ConSrvDllInstance) == 0 {
            return 0;
        }

        InitTTFontCache();

        ConsInitialized = 1;
    }

    if CsrImpersonateClient(core::ptr::null_mut()) == 0 {
        return 0;
    }

    if (*ConsoleInitInfo).DesktopLength != 0 {
        DesktopPath.MaximumLength = (*ConsoleInitInfo).DesktopLength as u16;
        DesktopPath.Length = DesktopPath.MaximumLength - core::mem::size_of::<WCHAR>() as u16;
        DesktopPath.Buffer = (*ConsoleInitInfo).Desktop;
    } else {
        RtlInitUnicodeString(&mut DesktopPath, w!("Default").as_ptr() as *const u16);
    }

    hDesk = NtUserResolveDesktop(
        ConsoleLeaderProcessHandle,
        &mut DesktopPath,
        0,
        &mut hWinSta,
    );

    CsrRevertToSelf();

    if hDesk.is_null() {
        return 0;
    }

    DesktopConsoleThreadInfo.DesktopHandle = hDesk;
    DesktopConsoleThreadInfo.ThreadId = INVALID_HANDLE_VALUE as ULONG_PTR;
    NtUserConsoleControl(
        ConsoleCtrlDesktopConsoleThread,
        &mut DesktopConsoleThreadInfo as *mut _ as PVOID,
        core::mem::size_of::<DESKTOP_CONSOLE_THREAD>() as ULONG,
    );

    (*GuiInitInfo).WinSta = hWinSta;
    (*GuiInitInfo).Desktop = hDesk;

    if DesktopConsoleThreadInfo.ThreadId != 0 {
        (*GuiInitInfo).InputThreadId = DesktopConsoleThreadInfo.ThreadId;
        'Quit: loop {
            if Success == 0 {
                CloseDesktop(hDesk);
                CloseWindowStation(hWinSta);
            }
            return Success;
        }
    }

    Status = NtCreateEvent(
        &mut (*GuiInitInfo).GuiThreadStartupEvent,
        EVENT_ALL_ACCESS,
        core::ptr::null_mut(),
        _EVENT_TYPE_SynchronizationEvent,
        0,
    );
    if !NT_SUCCESS(Status) {
        Success = 0;
        'Quit: loop {
            if Success == 0 {
                CloseDesktop(hDesk);
                CloseWindowStation(hWinSta);
            }
            return Success;
        }
    }

    Status = NtDuplicateObject(
        (-1isize) as HANDLE,
        hDesk as HANDLE,
        (-1isize) as HANDLE,
        &mut (*GuiInitInfo).Desktop as *mut _ as PHANDLE,
        0,
        0,
        DUPLICATE_SAME_ACCESS,
    );
    if !NT_SUCCESS(Status) {
        Success = 0;
        'Quit: loop {
            if Success == 0 {
                CloseDesktop(hDesk);
                CloseWindowStation(hWinSta);
            }
            return Success;
        }
    }

    Status = RtlCreateUserThread(
        (-1isize) as HANDLE,
        core::ptr::null_mut(),
        1,
        0,
        0,
        0,
        Some(GuiConsoleInputThread),
        GuiInitInfo as PVOID,
        &mut hInputThread,
        &mut ClientId,
    );
    if NT_SUCCESS(Status) {
        CsrAddStaticServerThread(hInputThread, &mut ClientId, 0);
        Status = NtResumeThread(hInputThread, core::ptr::null_mut());
    }

    if !NT_SUCCESS(Status) || hInputThread.is_null() {
        if !hInputThread.is_null() {
            NtClose(hInputThread);
        }

        CloseDesktop((*GuiInitInfo).Desktop);

        NtClose((*GuiInitInfo).GuiThreadStartupEvent);

        Success = 0;
        'Quit: loop {
            if Success == 0 {
                CloseDesktop(hDesk);
                CloseWindowStation(hWinSta);
            }
            return Success;
        }
    }

    NtWaitForSingleObject((*GuiInitInfo).GuiThreadStartupEvent, 0, core::ptr::null_mut());
    NtClose((*GuiInitInfo).GuiThreadStartupEvent);

    (*GuiInitInfo).InputThreadId = ClientId.UniqueThread as ULONG_PTR;
    (*GuiInitInfo).WinSta = hWinSta;
    (*GuiInitInfo).Desktop = hDesk;

    if Success == 0 {
        CloseDesktop(hDesk);
        CloseWindowStation(hWinSta);
    }

    Success
}

pub unsafe extern "system" fn GuiInitFrontEnd(This: PFRONTEND, Console: PCONSRV_CONSOLE) -> NTSTATUS {
    let GuiInitInfo: PGUI_INIT_INFO;
    let GuiData: PGUI_CONSOLE_DATA;

    if This.is_null() || Console.is_null() || (*This).Context2.is_null() {
        return STATUS_INVALID_PARAMETER;
    }

    GuiInitInfo = (*This).Context2 as PGUI_INIT_INFO;

    GuiData = ConsoleAllocHeap(HEAP_ZERO_MEMORY, core::mem::size_of::<GUI_CONSOLE_DATA>() as SIZE_T) as PGUI_CONSOLE_DATA;
    if GuiData.is_null() {
        return STATUS_UNSUCCESSFUL;
    }

    (*GuiData).Console = Console;
    (*GuiData).ActiveBuffer = (*Console).ActiveBuffer;
    (*GuiData).hWindow = core::ptr::null_mut();
    (*GuiData).IsWindowVisible = (*GuiInitInfo).IsWindowVisible;

    (*Console).FixedSize = 0u8;

    InitializeCriticalSection(core::ptr::addr_of_mut!((*GuiData).Lock));

    core::ptr::copy_nonoverlapping(
        core::ptr::addr_of!((*GuiInitInfo).TermInfo) as *const u8,
        core::ptr::addr_of_mut!((*GuiData).GuiInfo) as *mut u8,
        core::mem::size_of::<GUI_CONSOLE_INFO>()
    );

    if !(*GuiInitInfo).hIcon.is_null() {
        (*GuiData).hIcon = (*GuiInitInfo).hIcon;
    } else {
        (*GuiData).hIcon = ghDefaultIcon;
    }

    if !(*GuiInitInfo).hIconSm.is_null() {
        (*GuiData).hIconSm = (*GuiInitInfo).hIconSm;
    } else {
        (*GuiData).hIconSm = ghDefaultIconSm;
    }

    (*GuiData).hCursor = ghDefaultCursor;
    (*GuiData).MouseCursorRefCount = 0;

    (*GuiData).IgnoreNextMouseEvent = 0;
    (*GuiData).HackCORE8394IgnoreNextMove = 0;

    (*GuiData).IsCloseButtonEnabled = 1;

    (*GuiData).CmdIdLow = 0;
    (*GuiData).CmdIdHigh = 0;

    core::ptr::write_bytes(
        core::ptr::addr_of_mut!((*GuiData).Selection) as *mut u8,
        0,
        core::mem::size_of_val(&(*GuiData).Selection)
    );
    (*GuiData).Selection.dwFlags = CONSOLE_NO_SELECTION;
    core::ptr::write_bytes(
        core::ptr::addr_of_mut!((*GuiData).dwSelectionCursor) as *mut u8,
        0,
        core::mem::size_of_val(&(*GuiData).dwSelectionCursor)
    );
    (*GuiData).LineSelection = 0;

    (*GuiData).InputThreadId = (*GuiInitInfo).InputThreadId;
    (*GuiData).WinSta = (*GuiInitInfo).WinSta;
    (*GuiData).Desktop = (*GuiInitInfo).Desktop;

    (*This).Context = GuiData as *mut core::ffi::c_void;
    ConsoleFreeHeap((*This).Context2);
    (*This).Context2 = core::ptr::null_mut();

    NtCreateEvent(
        core::ptr::addr_of_mut!((*GuiData).hGuiInitEvent),
        EVENT_ALL_ACCESS,
        core::ptr::null_mut(),
        _EVENT_TYPE_SynchronizationEvent,
        0u8
    );
    NtCreateEvent(
        core::ptr::addr_of_mut!((*GuiData).hGuiTermEvent),
        EVENT_ALL_ACCESS,
        core::ptr::null_mut(),
        _EVENT_TYPE_SynchronizationEvent,
        0u8
    );

    PostThreadMessageW((*GuiData).InputThreadId as u32, PM_CREATE_CONSOLE, 0, GuiData as LPARAM);

    NtWaitForSingleObject((*GuiData).hGuiInitEvent, 0u8, core::ptr::null_mut());
    NtClose((*GuiData).hGuiInitEvent);
    (*GuiData).hGuiInitEvent = core::ptr::null_mut();

    if (*GuiData).hWindow.is_null() {
        GuiDeinitFrontEnd(This);
        return STATUS_UNSUCCESSFUL;
    }

    STATUS_SUCCESS
}

pub unsafe extern "system" fn GuiDeinitFrontEnd(This: PFRONTEND) {
    let GuiData = (*This).Context as PGUI_CONSOLE_DATA;

    PostThreadMessageW((*GuiData).InputThreadId as u32, PM_DESTROY_CONSOLE, 0, GuiData as LPARAM);
    NtWaitForSingleObject((*GuiData).hGuiTermEvent, 0u8, core::ptr::null_mut());
    NtClose((*GuiData).hGuiTermEvent);
    (*GuiData).hGuiTermEvent = core::ptr::null_mut();

    CloseDesktop((*GuiData).Desktop);
    CloseWindowStation((*GuiData).WinSta);

    if !(*GuiData).hIcon.is_null() && (*GuiData).hIcon != ghDefaultIcon {
        DestroyIcon((*GuiData).hIcon);
    }
    if !(*GuiData).hIconSm.is_null() && (*GuiData).hIconSm != ghDefaultIconSm {
        DestroyIcon((*GuiData).hIconSm);
    }

    (*This).Context = core::ptr::null_mut();
    DeleteCriticalSection(core::ptr::addr_of_mut!((*GuiData).Lock));
    ConsoleFreeHeap(GuiData as *mut _);
}

pub unsafe extern "system" fn GuiDrawRegion(This: PFRONTEND, Region: *mut SMALL_RECT) {
    let GuiData = (*This).Context as PGUI_CONSOLE_DATA;

    if (*GuiData).IsWindowVisible == 0 {
        return;
    }

    DrawRegion(GuiData, Region);
}

pub unsafe extern "system" fn GuiWriteStream(
    This: PFRONTEND,
    Region: *mut SMALL_RECT,
    CursorStartX: i16,
    CursorStartY: i16,
    ScrolledLines: u32,
    _Buffer: *mut u16,
    _Length: u32,
) {
    let GuiData = (*This).Context as PGUI_CONSOLE_DATA;
    let CursorEndX: i16;
    let CursorEndY: i16;
    let ScrollRect: RECT;

    if GuiData.is_null() || (*GuiData).hWindow.is_null() {
        return;
    }

    if (*GuiData).IsWindowVisible == 0 {
        return;
    }

    let Buff = (*GuiData).ActiveBuffer;
    if GetType(Buff) != TEXTMODE_BUFFER {
        return;
    }

    if ScrolledLines != 0 {
        ScrollRect = RECT {
            left: 0,
            top: 0,
            right: (*Buff).ViewSize.X as i32 * (*GuiData).CharWidth as i32,
            bottom: (*Region).Top as i32 * (*GuiData).CharHeight as i32,
        };

        ScrollWindowEx(
            (*GuiData).hWindow,
            0,
            -((ScrolledLines * (*GuiData).CharHeight) as i32),
            &ScrollRect,
            core::ptr::null(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            SW_INVALIDATE,
        );
    }

    DrawRegion(GuiData, Region);

    if CursorStartX < (*Region).Left || (*Region).Right < CursorStartX
        || CursorStartY < (*Region).Top || (*Region).Bottom < CursorStartY
    {
        InvalidateCell(GuiData, CursorStartX, CursorStartY);
    }

    CursorEndX = (*Buff).CursorPosition.X;
    CursorEndY = (*Buff).CursorPosition.Y;
    if (CursorEndX < (*Region).Left || (*Region).Right < CursorEndX
        || CursorEndY < (*Region).Top || (*Region).Bottom < CursorEndY)
        && (CursorEndX != CursorStartX || CursorEndY != CursorStartY)
    {
        InvalidateCell(GuiData, CursorEndX, CursorEndY);
    }

    (*Buff).CursorBlinkOn = 1;
    SetTimer((*GuiData).hWindow, CONGUI_UPDATE_TIMER as UINT_PTR, CONGUI_UPDATE_TIME, None);
}

pub unsafe extern "system" fn GuiRingBell(This: PFRONTEND) {
    let GuiData = (*This).Context as *mut GUI_CONSOLE_DATA;
    
    
    SendNotifyMessageW((*GuiData).hWindow, PM_CONSOLE_BEEP, 0, 0);
}

pub unsafe extern "system" fn GuiSetCursorInfo(
    This: PFRONTEND,
    Buff: PCONSOLE_SCREEN_BUFFER,
) -> BOOL {
    let GuiData: PGUI_CONSOLE_DATA = (*This).Context as PGUI_CONSOLE_DATA;

    if (*GuiData).IsWindowVisible == 0u8 {
        return TRUE as BOOL;
    }

    if (*GuiData).ActiveBuffer == Buff {
        InvalidateCell(
            GuiData,
            (*Buff).CursorPosition.X,
            (*Buff).CursorPosition.Y,
        );
    }

    TRUE as BOOL
}

pub unsafe extern "system" fn GuiSetScreenInfo(
    This: PFRONTEND,
    Buff: PCONSOLE_SCREEN_BUFFER,
    OldCursorX: i16,
    OldCursorY: i16,
) -> BOOL {
    let GuiData = (*This).Context as PGUI_CONSOLE_DATA;

    if (*GuiData).IsWindowVisible == 0 {
        return TRUE as BOOL;
    }

    if (*GuiData).ActiveBuffer == Buff {
        InvalidateCell(GuiData, OldCursorX, OldCursorY);
        InvalidateCell(GuiData, (*Buff).CursorPosition.X, (*Buff).CursorPosition.Y);
    }

    TRUE as BOOL
}

pub unsafe extern "system" fn GuiResizeTerminal(This: PFRONTEND) {
    let GuiData = (*This).Context as *mut GUI_CONSOLE_DATA;

    PostMessageW((*GuiData).hWindow, PM_RESIZE_TERMINAL, 0, 0);
}

pub unsafe extern "system" fn GuiSetActiveScreenBuffer(This: PFRONTEND) {
    let GuiData = (*This).Context as *mut GUI_CONSOLE_DATA;
    let ActiveBuffer: PCONSOLE_SCREEN_BUFFER;
    let hPalette: HPALETTE;

    EnterCriticalSection(core::ptr::addr_of_mut!((*GuiData).Lock));
    (*GuiData).WindowSizeLock = 1;

    InterlockedExchangePointer(
        core::ptr::addr_of_mut!((*GuiData).ActiveBuffer) as *mut *mut core::ffi::c_void,
        ConDrvGetActiveScreenBuffer((*GuiData).Console as PCONSOLE) as *mut core::ffi::c_void
    );

    (*GuiData).WindowSizeLock = 0;
    LeaveCriticalSection(core::ptr::addr_of_mut!((*GuiData).Lock));

    ActiveBuffer = (*GuiData).ActiveBuffer;

    if (*ActiveBuffer).PaletteHandle.is_null() {
        hPalette = (*GuiData).hSysPalette;
    } else {
        hPalette = (*ActiveBuffer).PaletteHandle;
    }

    SelectPalette((*GuiData).hMemDC, hPalette, 0);

    SetSystemPaletteUse((*GuiData).hMemDC, (*ActiveBuffer).PaletteUsage);

    RealizePalette((*GuiData).hMemDC);

    GuiResizeTerminal(This);
}

pub unsafe extern "system" fn GuiReleaseScreenBuffer(This: PFRONTEND, ScreenBuffer: PCONSOLE_SCREEN_BUFFER) {
    let GuiData = (*This).Context as *mut GUI_CONSOLE_DATA;

    if ScreenBuffer != (*GuiData).ActiveBuffer {
        return;
    }

    if GetCurrentObject((*GuiData).hMemDC, OBJ_PAL) == (*ScreenBuffer).PaletteHandle as *mut core::ffi::c_void {
        SelectPalette((*GuiData).hMemDC, (*GuiData).hSysPalette, 0);
    }

    if ScreenBuffer != (*(*GuiData).Console).ActiveBuffer {
        GuiSetActiveScreenBuffer(This);
    } else {
        EnterCriticalSection(core::ptr::addr_of_mut!((*GuiData).Lock));
        (*GuiData).WindowSizeLock = 1;

        InterlockedExchangePointer(
            core::ptr::addr_of_mut!((*GuiData).ActiveBuffer) as *mut *mut core::ffi::c_void,
            core::ptr::null_mut()
        );

        (*GuiData).WindowSizeLock = 0;
        LeaveCriticalSection(core::ptr::addr_of_mut!((*GuiData).Lock));
    }
}

pub unsafe extern "system" fn GuiRefreshInternalInfo(This: PFRONTEND) {
    let GuiData = (*This).Context as *mut _GUI_CONSOLE_DATA;

    SetConWndConsoleLeaderCID(GuiData);

    (*GuiData).MouseCursorRefCount = 0;
    GuiSetMouseCursor(This, core::ptr::null_mut());
}

pub unsafe extern "system" fn GuiChangeTitle(This: PFRONTEND) {
    let GuiData = (*This).Context as *mut GUI_CONSOLE_DATA;
    PostMessageW((*GuiData).hWindow, PM_CONSOLE_SET_TITLE, 0, 0);
}

pub unsafe extern "system" fn GuiChangeIcon(
    This: PFRONTEND,
    IconHandle: HICON
) -> BOOL {

    let GuiData = (*This).Context as *mut GUI_CONSOLE_DATA;
    let hIcon: HICON;
    let hIconSm: HICON;

    if IconHandle.is_null() {
        hIcon = ghDefaultIcon;
        hIconSm = ghDefaultIconSm;
    } else {
        hIcon = CopyIcon(IconHandle);
        hIconSm = CopyIcon(IconHandle);
    }

    if hIcon.is_null() {
        return FALSE as BOOL;
    }

    if hIcon != (*GuiData).hIcon {
        if !(*GuiData).hIcon.is_null() && (*GuiData).hIcon != ghDefaultIcon {
            DestroyIcon((*GuiData).hIcon);
        }
        if !(*GuiData).hIconSm.is_null() && (*GuiData).hIconSm != ghDefaultIconSm {
            DestroyIcon((*GuiData).hIconSm);
        }

        (*GuiData).hIcon = hIcon;
        (*GuiData).hIconSm = hIconSm;

        PostMessageW((*GuiData).hWindow, WM_SETICON, ICON_BIG as WPARAM, (*GuiData).hIcon as LPARAM);
        PostMessageW((*GuiData).hWindow, WM_SETICON, ICON_SMALL as WPARAM, (*GuiData).hIconSm as LPARAM);
    }

    TRUE as BOOL
}

pub unsafe extern "system" fn GuiGetThreadConsoleDesktop(This: PFRONTEND) -> HDESK {
    let GuiData = (*This).Context as PGUI_CONSOLE_DATA;
    (*GuiData).Desktop
}

pub unsafe extern "system" fn GuiGetConsoleWindowHandle(This: PFRONTEND) -> HWND {
    let GuiData = (*This).Context as PGUI_CONSOLE_DATA;
    (*GuiData).hWindow
}

pub unsafe extern "system" fn GuiGetLargestConsoleWindowSize(
    This: PFRONTEND,
    pSize: PCOORD,
) {
    let GuiData = (*This).Context as *mut GUI_CONSOLE_DATA;
    let ActiveBuffer: PCONSOLE_SCREEN_BUFFER;
    let hMonitor: HMONITOR;
    let mut MonitorInfo: MONITORINFO = core::mem::zeroed();
    let mut Width: LONG;
    let mut Height: LONG;
    let mut WidthUnit: UINT = 0;
    let mut HeightUnit: UINT = 0;

    if pSize.is_null() {
        return;
    }

    MonitorInfo.cbSize = core::mem::size_of::<MONITORINFO>() as DWORD;
    hMonitor = MonitorFromWindow((*GuiData).hWindow, MONITOR_DEFAULTTOPRIMARY);
    if !hMonitor.is_null() && GetMonitorInfoW(hMonitor, &mut MonitorInfo) != 0 {
        Width = MonitorInfo.rcWork.right - MonitorInfo.rcWork.left;
        Height = MonitorInfo.rcWork.bottom - MonitorInfo.rcWork.top;
    } else {
        Width = GetSystemMetrics(SM_CXFULLSCREEN as i32);
        Height = GetSystemMetrics(SM_CYFULLSCREEN as i32);
    }

    ActiveBuffer = (*GuiData).ActiveBuffer;
    GetScreenBufferSizeUnits(ActiveBuffer, GuiData, &mut WidthUnit, &mut HeightUnit);

    Width -= 2 * (GetSystemMetrics(SM_CXFRAME as i32) + GetSystemMetrics(SM_CXEDGE as i32));
    Height -= 2 * (GetSystemMetrics(SM_CYFRAME as i32) + GetSystemMetrics(SM_CYEDGE as i32)) + GetSystemMetrics(SM_CYCAPTION as i32);

    if Width < 0 {
        Width = 0;
    }
    if Height < 0 {
        Height = 0;
    }

    (*pSize).X = (Width / WidthUnit as i32) as SHORT + 2;
    (*pSize).Y = (Height / HeightUnit as i32) as SHORT + 1;
}

pub unsafe extern "system" fn GuiGetSelectionInfo(
    This: PFRONTEND,
    pSelectionInfo: PCONSOLE_SELECTION_INFO,
) -> BOOL {
    let GuiData = (*This).Context as *mut GUI_CONSOLE_DATA;

    if pSelectionInfo.is_null() {
        return FALSE as BOOL;
    }

    core::ptr::write_bytes(pSelectionInfo, 0, 1);
    if (*GuiData).Selection.dwFlags != CONSOLE_NO_SELECTION {
        RtlCopyMemory(
            pSelectionInfo as *mut core::ffi::c_void,
            core::ptr::addr_of!((*GuiData).Selection) as *const core::ffi::c_void,
            core::mem::size_of::<CONSOLE_SELECTION_INFO>(),
        );
    }

    TRUE as BOOL
}

pub unsafe extern "system" fn GuiSetPalette(
    This: PFRONTEND,
    PaletteHandle: HPALETTE,
    PaletteUsage: UINT,
) -> BOOL {

    let GuiData = (*This).Context as PGUI_CONSOLE_DATA;
    let OldPalette: HPALETTE;

    if PaletteHandle.is_null() {
        return 0;
    }

    OldPalette = SelectPalette((*GuiData).hMemDC, PaletteHandle, 0);
    if OldPalette.is_null() {
        return 0;
    }

    SetSystemPaletteUse((*GuiData).hMemDC, PaletteUsage);

    RealizePalette((*GuiData).hMemDC);

    if (*GuiData).hSysPalette.is_null() {
        (*GuiData).hSysPalette = OldPalette;
    }

    1
}

pub unsafe extern "system" fn GuiSetCodePage(
    This: PFRONTEND,
    CodePage: u32,
) -> BOOL {
    let GuiData = (*This).Context as *mut GUI_CONSOLE_DATA;

    if InitFonts(
        GuiData,
        (*GuiData).GuiInfo.FaceName.as_ptr(),
        (*GuiData).GuiInfo.FontWeight,
        (*GuiData).GuiInfo.FontFamily,
        (*GuiData).GuiInfo.FontSize,
        CodePage,
        0,
    ) == 0
    {
        return FALSE as BOOL;
    }

    TRUE as BOOL
}

pub unsafe extern "system" fn GuiGetDisplayMode(This: PFRONTEND) -> ULONG {
    let GuiData = (*This).Context as PGUI_CONSOLE_DATA;
    let mut DisplayMode: ULONG = 0;

    if (*GuiData).GuiInfo.FullScreen != 0 {
        DisplayMode |= CONSOLE_FULLSCREEN_HARDWARE;
    } else {
        DisplayMode |= CONSOLE_WINDOWED;
    }

    DisplayMode
}

pub unsafe extern "system" fn GuiSetDisplayMode(
    This: PFRONTEND,
    NewMode: ULONG,
) -> BOOL {
    let GuiData = (*This).Context as PGUI_CONSOLE_DATA;
    let FullScreen: BOOL;

    if (NewMode & !(CONSOLE_FULLSCREEN_MODE | CONSOLE_WINDOWED_MODE)) != 0 {
        return 0;
    }

    if (*GuiData).IsWindowVisible == 0 {
        return 1;
    }

    FullScreen = if (NewMode & CONSOLE_FULLSCREEN_MODE) != 0 { 1 } else { 0 };

    if (FullScreen != 0) != ((*GuiData).GuiInfo.FullScreen != 0) {
        SwitchFullScreen(GuiData, FullScreen);
    }

    1
}

pub unsafe extern "system" fn GuiShowMouseCursor(
    This: PFRONTEND,
    Show: BOOL,
) -> INT {
    let GuiData = (*This).Context as PGUI_CONSOLE_DATA;

    if (*GuiData).IsWindowVisible != 0 {
        if Show != 0 {
            (*GuiData).MouseCursorRefCount += 1;
        } else {
            (*GuiData).MouseCursorRefCount -= 1;
        }

        PostMessageW((*GuiData).hWindow, WM_SETCURSOR, -1i32 as WPARAM, -1i32 as LPARAM);
    }

    (*GuiData).MouseCursorRefCount
}

pub unsafe extern "system" fn GuiSetMouseCursor(
    This: PFRONTEND,
    CursorHandle: HCURSOR,
) -> BOOL {
    let GuiData = (*This).Context as *mut _GUI_CONSOLE_DATA;

    if (*GuiData).IsWindowVisible == 0 {
        return TRUE as BOOL;
    }

    (*GuiData).hCursor = if !CursorHandle.is_null() {
        CursorHandle
    } else {
        ghDefaultCursor
    };

    PostMessageW((*GuiData).hWindow, WM_SETCURSOR, -1i32 as WPARAM, -1i32 as LPARAM);

    TRUE as BOOL
}

pub unsafe extern "system" fn GuiMenuControl(
    This: PFRONTEND,
    CmdIdLow: UINT,
    CmdIdHigh: UINT,
) -> HMENU {
    let GuiData = (*This).Context as PGUI_CONSOLE_DATA;

    (*GuiData).CmdIdLow = CmdIdLow;
    (*GuiData).CmdIdHigh = CmdIdHigh;

    (*GuiData).hSysMenu
}

pub unsafe extern "system" fn GuiSetMenuClose(
    This: PFRONTEND,
    Enable: BOOL,
) -> BOOL {

    let GuiData = (*This).Context as PGUI_CONSOLE_DATA;

    if (*GuiData).hSysMenu.is_null() {
        return 0;
    }

    (*GuiData).IsCloseButtonEnabled = if Enable != 0 { 1 } else { 0 };
    EnableMenuItem(
        (*GuiData).hSysMenu,
        SC_CLOSE,
        MF_BYCOMMAND | if Enable != 0 { MF_ENABLED } else { MF_GRAYED },
    );

    1
}

/// The font's size as created, not the measured cell: for a TrueType face that
/// is {0, height}, and Windows 11 reports exactly that (Consolas 16 reads 0x16).
pub unsafe extern "system" fn GuiGetFontSize(
    This: PFRONTEND,
    pSize: PCOORD,
) {
    let GuiData = (*This).Context as PGUI_CONSOLE_DATA;
    if pSize.is_null() {
        return;
    }
    *pSize = (*GuiData).GuiInfo.FontSize;
}

#[no_mangle]
pub unsafe extern "system" fn GuiLoadFrontEnd(
    FrontEnd: PFRONTEND,
    ConsoleInfo: PCONSOLE_STATE_INFO,
    ConsoleInitInfo: PCONSOLE_INIT_INFO,
    ConsoleLeaderProcessHandle: HANDLE,
) -> NTSTATUS {

    if FrontEnd.is_null() || ConsoleInfo.is_null() || ConsoleInitInfo.is_null() {
        return STATUS_INVALID_PARAMETER;
    }

    let ConsoleStartInfo: PCONSOLE_START_INFO = (*ConsoleInitInfo).ConsoleStartInfo;

    let GuiInitInfo: PGUI_INIT_INFO = ConsoleAllocHeap(
        HEAP_ZERO_MEMORY,
        core::mem::size_of::<GUI_INIT_INFO>() as SIZE_T,
    ) as PGUI_INIT_INFO;
    if GuiInitInfo.is_null() {
        return STATUS_NO_MEMORY;
    }

    if GuiInit(ConsoleInitInfo, ConsoleLeaderProcessHandle, GuiInitInfo) == 0 {
        ConsoleFreeHeap(GuiInitInfo as PVOID);
        return STATUS_UNSUCCESSFUL;
    }

    (*GuiInitInfo).IsWindowVisible = (*ConsoleInitInfo).IsWindowVisible;
    if (*GuiInitInfo).IsWindowVisible != 0 {
        let mut UserObjectFlags: USEROBJECTFLAGS = core::mem::zeroed();
        if GetUserObjectInformationW(
            (*GuiInitInfo).WinSta as HANDLE,
            UOI_FLAGS as i32,
            &mut UserObjectFlags as *mut _ as PVOID,
            core::mem::size_of::<USEROBJECTFLAGS>() as DWORD,
            core::ptr::null_mut(),
        ) != 0
        {
            if (UserObjectFlags.dwFlags & WSF_VISIBLE) == 0 {
                (*GuiInitInfo).IsWindowVisible = 0u8;
            }
        }
    }

    (*GuiInitInfo).TermInfo.ShowWindow = SW_SHOWNORMAL as WORD;

    if (*GuiInitInfo).IsWindowVisible != 0 {
        if ((*ConsoleStartInfo).dwStartupFlags & STARTF_TITLEISLINKNAME) == 0 {
            if (*ConsoleStartInfo).dwStartupFlags & STARTF_USESHOWWINDOW != 0 {
                (*GuiInitInfo).TermInfo.ShowWindow = (*ConsoleStartInfo).wShowWindow as WORD;
            }
            if (*ConsoleStartInfo).dwStartupFlags & STARTF_USEPOSITION != 0 {
                (*ConsoleInfo).AutoPosition = 0;
                (*ConsoleInfo).WindowPosition.x = (*ConsoleStartInfo).dwWindowOrigin.X as LONG;
                (*ConsoleInfo).WindowPosition.y = (*ConsoleStartInfo).dwWindowOrigin.Y as LONG;
            }
            if (*ConsoleStartInfo).dwStartupFlags & STARTF_RUNFULLSCREEN != 0 {
                (*ConsoleInfo).FullScreen = 1;
            }
        }
    }

    StringCchCopyNW(
        (*GuiInitInfo).TermInfo.FaceName.as_mut_ptr(),
        (*GuiInitInfo).TermInfo.FaceName.len(),
        (*ConsoleInfo).FaceName.as_ptr(),
        (*ConsoleInfo).FaceName.len(),
    );
    (*GuiInitInfo).TermInfo.FontFamily = (*ConsoleInfo).FontFamily;
    (*GuiInitInfo).TermInfo.FontSize = (*ConsoleInfo).FontSize;
    (*GuiInitInfo).TermInfo.FontWeight = (*ConsoleInfo).FontWeight;

    (*GuiInitInfo).TermInfo.FullScreen = (*ConsoleInfo).FullScreen;
    (*GuiInitInfo).TermInfo.AutoPosition = (*ConsoleInfo).AutoPosition;
    (*GuiInitInfo).TermInfo.WindowOrigin = (*ConsoleInfo).WindowPosition;

    (*GuiInitInfo).hIcon = (*ConsoleStartInfo).hIcon;
    (*GuiInitInfo).hIconSm = (*ConsoleStartInfo).hIconSm;

    (*FrontEnd).Vtbl = &raw const GuiVtbl as *const _ as *mut _;
    (*FrontEnd).Context = core::ptr::null_mut();
    (*FrontEnd).Context2 = GuiInitInfo as PVOID;

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn GuiUnloadFrontEnd(FrontEnd: PFRONTEND) -> NTSTATUS {
    if FrontEnd.is_null() {
        return STATUS_INVALID_PARAMETER;
    }

    if !(*FrontEnd).Context.is_null() {
        GuiDeinitFrontEnd(FrontEnd);
    }
    if !(*FrontEnd).Context2.is_null() {
        ConsoleFreeHeap((*FrontEnd).Context2);
    }

    STATUS_SUCCESS
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:       See COPYING in the top level directory
//  * PROJECT:         ReactOS Console Server DLL
//  * FILE:            win32ss/user/winsrv/consrv/frontends/gui/guiterm.c
//  * PURPOSE:         GUI Terminal Front-End
//  * PROGRAMMERS:     Gé van Geldorp
//  *                  Johannes Anderwald
//  *                  Jeffrey Morlan
//  *                  Hermes Belusca-Maito (hermes.belusca@sfr.fr)
//  */
//
// /* INCLUDES *******************************************************************/
//
// #include <consrv.h>
//
// #define NDEBUG
// #include <debug.h>
//
// #include "concfg/font.h"
// #include "guiterm.h"
// #include "resource.h"
//
// // HACK!! Remove it when the hack in GuiWriteStream is fixed
// #define CONGUI_UPDATE_TIME    0
// #define CONGUI_UPDATE_TIMER   1
//
// #define PM_CREATE_CONSOLE     (WM_APP + 1)
// #define PM_DESTROY_CONSOLE    (WM_APP + 2)
//
//
// /* GLOBALS ********************************************************************/
//
// typedef struct _GUI_INIT_INFO
// {
//     HANDLE GuiThreadStartupEvent;
//     ULONG_PTR InputThreadId;
//     HWINSTA WinSta;
//     HDESK Desktop;
//     HICON hIcon;
//     HICON hIconSm;
//     BOOLEAN IsWindowVisible;
//     GUI_CONSOLE_INFO TermInfo;
// } GUI_INIT_INFO, *PGUI_INIT_INFO;
//
// static BOOL ConsInitialized = FALSE;
//
// extern HICON   ghDefaultIcon;
// extern HICON   ghDefaultIconSm;
// extern HCURSOR ghDefaultCursor;
//
// VOID
// SetConWndConsoleLeaderCID(IN PGUI_CONSOLE_DATA GuiData);
// BOOLEAN
// RegisterConWndClass(IN HINSTANCE hInstance);
// BOOLEAN
// UnRegisterConWndClass(HINSTANCE hInstance);
//
// /* FUNCTIONS ******************************************************************/
//
// VOID
// GuiConsoleMoveWindow(PGUI_CONSOLE_DATA GuiData)
// {
//     /* Move the window if needed (not positioned by the system) */
//     if (!GuiData->GuiInfo.AutoPosition)
//     {
//         SetWindowPos(GuiData->hWindow,
//                      NULL,
//                      GuiData->GuiInfo.WindowOrigin.x,
//                      GuiData->GuiInfo.WindowOrigin.y,
//                      0, 0,
//                      SWP_NOZORDER | SWP_NOSIZE | SWP_NOACTIVATE);
//     }
// }
//
// static VOID
// DrawRegion(PGUI_CONSOLE_DATA GuiData,
//            SMALL_RECT* Region)
// {
//     RECT RegionRect;
//
//     SmallRectToRect(GuiData, &RegionRect, Region);
//     /* Do not erase the background: it speeds up redrawing and reduce flickering */
//     InvalidateRect(GuiData->hWindow, &RegionRect, FALSE);
//     /**UpdateWindow(GuiData->hWindow);**/
// }
//
// VOID
// InvalidateCell(PGUI_CONSOLE_DATA GuiData,
//                SHORT x, SHORT y)
// {
//     SMALL_RECT CellRect = { x, y, x, y };
//     DrawRegion(GuiData, &CellRect);
// }
//
//
// /******************************************************************************
//  *                        GUI Terminal Initialization                         *
//  ******************************************************************************/
//
// // FIXME: HACK: Potential HACK for CORE-8129; see revision 63595.
// VOID
// CreateSysMenu(HWND hWnd);
//
// static ULONG NTAPI
// GuiConsoleInputThread(PVOID Param)
// {
//     NTSTATUS Status;
//     PCSR_THREAD pcsrt = NULL;
//     PGUI_INIT_INFO GuiInitInfo = (PGUI_INIT_INFO)Param;
//     DESKTOP_CONSOLE_THREAD DesktopConsoleThreadInfo;
//     ULONG_PTR InputThreadId = HandleToUlong(NtCurrentTeb()->ClientId.UniqueThread);
//     HANDLE hThread = NULL;
//
//     LONG WindowCount = 0;
//     MSG msg;
//
//     /*
//      * This thread dispatches all the console notifications to the
//      * notification window. It is common for all the console windows
//      * in a given desktop in a window station.
//      */
//
//     /* Assign this console input thread to this desktop */
//     DesktopConsoleThreadInfo.DesktopHandle = GuiInitInfo->Desktop; // Duplicated desktop handle
//     DesktopConsoleThreadInfo.ThreadId = InputThreadId;
//     Status = NtUserConsoleControl(ConsoleCtrlDesktopConsoleThread,
//                                   &DesktopConsoleThreadInfo,
//                                   sizeof(DesktopConsoleThreadInfo));
//     if (!NT_SUCCESS(Status)) goto Quit;
//
//     /* Connect this CSR thread to the USER subsystem */
//     pcsrt = CsrConnectToUser();
//     if (pcsrt == NULL) goto Quit;
//     hThread = pcsrt->ThreadHandle;
//
//     /* Assign the desktop to this thread */
//     if (!SetThreadDesktop(DesktopConsoleThreadInfo.DesktopHandle)) goto Quit;
//
//     /* The thread has been initialized, set the event */
//     NtSetEvent(GuiInitInfo->GuiThreadStartupEvent, NULL);
//     Status = STATUS_SUCCESS;
//
//     while (GetMessageW(&msg, NULL, 0, 0))
//     {
//         switch (msg.message)
//         {
//             case PM_CREATE_CONSOLE:
//             {
//                 PGUI_CONSOLE_DATA GuiData = (PGUI_CONSOLE_DATA)msg.lParam;
//                 PCONSRV_CONSOLE Console = GuiData->Console;
//                 HWND NewWindow;
//                 RECT rcWnd;
//
//                 DPRINT("PM_CREATE_CONSOLE -- creating window\n");
//
//                 NewWindow = CreateWindowExW(WS_EX_CLIENTEDGE,
//                                             GUI_CONWND_CLASS_W.as_ptr(),
//                                             Console->Title.Buffer,
//                                             WS_OVERLAPPEDWINDOW,
//                                             CW_USEDEFAULT,
//                                             CW_USEDEFAULT,
//                                             CW_USEDEFAULT,
//                                             CW_USEDEFAULT,
//                                             GuiData->IsWindowVisible ? HWND_DESKTOP : HWND_MESSAGE,
//                                             NULL,
//                                             ConSrvDllInstance,
//                                             (PVOID)GuiData);
//                 if (NewWindow == NULL)
//                 {
//                     DPRINT1("Failed to create a new console window\n");
//                     continue;
//                 }
//
//                 ASSERT(NewWindow == GuiData->hWindow);
//
//                 _InterlockedIncrement(&WindowCount);
//
//                 //
//                 // FIXME: TODO: Move everything there into conwnd.c!OnNcCreate()
//                 //
//
//                 /* Retrieve our real position */
//                 // See conwnd.c!OnMove()
//                 GetWindowRect(GuiData->hWindow, &rcWnd);
//                 GuiData->GuiInfo.WindowOrigin.x = rcWnd.left;
//                 GuiData->GuiInfo.WindowOrigin.y = rcWnd.top;
//
//                 if (GuiData->IsWindowVisible)
//                 {
//                     /* Move and resize the window to the user's values */
//                     /* CAN WE DEADLOCK ?? */
//                     GuiConsoleMoveWindow(GuiData); // FIXME: This MUST be done via the CreateWindowExW call.
//                     SendMessageW(GuiData->hWindow, PM_RESIZE_TERMINAL, 0, 0);
//                 }
//
//                 // FIXME: HACK: Potential HACK for CORE-8129; see revision 63595.
//                 CreateSysMenu(GuiData->hWindow);
//
//                 if (GuiData->IsWindowVisible)
//                 {
//                     /* Switch to full-screen mode if necessary */
//                     // FIXME: Move elsewhere, it cause misdrawings of the window.
//                     if (GuiData->GuiInfo.FullScreen) SwitchFullScreen(GuiData, TRUE);
//
//                     DPRINT("PM_CREATE_CONSOLE -- showing window\n");
//                     ShowWindowAsync(NewWindow, (int)GuiData->GuiInfo.ShowWindow);
//                 }
//                 else
//                 {
//                     DPRINT("PM_CREATE_CONSOLE -- hidden window\n");
//                     ShowWindowAsync(NewWindow, SW_HIDE as i32);
//                 }
//
//                 continue;
//             }
//
//             case PM_DESTROY_CONSOLE:
//             {
//                 PGUI_CONSOLE_DATA GuiData = (PGUI_CONSOLE_DATA)msg.lParam;
//                 MSG TempMsg;
//
//                 /* Exit the full screen mode if it was already set */
//                 // LeaveFullScreen(GuiData);
//
//                 /*
//                  * Window creation is done using a PostMessage(), so it's possible
//                  * that the window that we want to destroy doesn't exist yet.
//                  * So first empty the message queue.
//                  */
//                 while (PeekMessageW(&TempMsg, NULL, 0, 0, PM_REMOVE))
//                 {
//                     DispatchMessageW(&TempMsg);
//                 }
//
//                 if (GuiData->hWindow == NULL) continue;
//
//                 DestroyWindow(GuiData->hWindow);
//
//                 NtSetEvent(GuiData->hGuiTermEvent, NULL);
//
//                 if (_InterlockedDecrement(&WindowCount) == 0)
//                 {
//                     DPRINT("CONSRV: Going to quit the Input Thread 0x%p\n", InputThreadId);
//                     goto Quit;
//                 }
//
//                 continue;
//             }
//         }
//
//         TranslateMessage(&msg);
//         DispatchMessageW(&msg);
//     }
//
// Quit:
//     DPRINT("CONSRV: Quit the Input Thread 0x%p, Status = 0x%08lx\n", InputThreadId, Status);
//
//     /* Remove this console input thread from this desktop */
//     // DesktopConsoleThreadInfo.DesktopHandle;
//     DesktopConsoleThreadInfo.ThreadId = 0;
//     NtUserConsoleControl(ConsoleCtrlDesktopConsoleThread,
//                          &DesktopConsoleThreadInfo,
//                          sizeof(DesktopConsoleThreadInfo));
//
//     /* Close the duplicated desktop handle */
//     CloseDesktop(DesktopConsoleThreadInfo.DesktopHandle); // NtUserCloseDesktop
//
//     /* Cleanup CSR thread */
//     if (pcsrt)
//     {
//         if (hThread != pcsrt->ThreadHandle)
//             DPRINT1("WARNING!! hThread (0x%p) != pcsrt->ThreadHandle (0x%p), you may expect crashes soon!!\n", hThread, pcsrt->ThreadHandle);
//
//         CsrDereferenceThread(pcsrt);
//     }
//
//     /* Exit the thread */
//     RtlExitUserThread(Status);
//     return 0;
// }
//
// // FIXME: Maybe return a NTSTATUS
// static BOOL
// GuiInit(IN PCONSOLE_INIT_INFO ConsoleInitInfo,
//         IN HANDLE ConsoleLeaderProcessHandle,
//         IN OUT PGUI_INIT_INFO GuiInitInfo)
// {
//     BOOL Success = TRUE;
//     UNICODE_STRING DesktopPath;
//     DESKTOP_CONSOLE_THREAD DesktopConsoleThreadInfo;
//     HWINSTA hWinSta;
//     HDESK hDesk;
//
//     NTSTATUS Status;
//     HANDLE hInputThread;
//     CLIENT_ID ClientId;
//
//     /* Perform one-time initialization */
//     if (!ConsInitialized)
//     {
//         /* Initialize and register the console window class */
//         if (!RegisterConWndClass(ConSrvDllInstance)) return FALSE;
//
//         /* Initialize the font support -- additional TrueType font cache */
//         InitTTFontCache();
//
//         ConsInitialized = TRUE;
//     }
//
//     /*
//      * Set-up the console input thread. We have
//      * one console input thread per desktop.
//      */
//
//     if (!CsrImpersonateClient(NULL))
//         // return STATUS_BAD_IMPERSONATION_LEVEL;
//         return FALSE;
//
//     if (ConsoleInitInfo->DesktopLength)
//     {
//         DesktopPath.MaximumLength = ConsoleInitInfo->DesktopLength;
//         DesktopPath.Length = DesktopPath.MaximumLength - sizeof(UNICODE_NULL);
//         DesktopPath.Buffer = ConsoleInitInfo->Desktop;
//     }
//     else
//     {
//         RtlInitUnicodeString(&DesktopPath, L"Default");
//     }
//
//     hDesk = NtUserResolveDesktop(ConsoleLeaderProcessHandle,
//                                  &DesktopPath,
//                                  FALSE,
//                                  &hWinSta);
//     DPRINT("NtUserResolveDesktop(DesktopPath = '%wZ') returned hDesk = 0x%p; hWinSta = 0x%p\n",
//            &DesktopPath, hDesk, hWinSta);
//
//     CsrRevertToSelf();
//
//     if (hDesk == NULL) return FALSE;
//
//     /*
//      * We need to see whether we need to create a
//      * new console input thread for this desktop.
//      */
//     DesktopConsoleThreadInfo.DesktopHandle = hDesk;
//     DesktopConsoleThreadInfo.ThreadId = (ULONG_PTR)INVALID_HANDLE_VALUE; // Special value to say we just want to retrieve the thread ID.
//     NtUserConsoleControl(ConsoleCtrlDesktopConsoleThread,
//                          &DesktopConsoleThreadInfo,
//                          sizeof(DesktopConsoleThreadInfo));
//     DPRINT("NtUserConsoleControl returned ThreadId = 0x%p\n", DesktopConsoleThreadInfo.ThreadId);
//
//     /*
//      * Save the opened window station and desktop handles in the initialization
//      * structure. They will be used later on, and released, by the GUI frontend.
//      */
//     GuiInitInfo->WinSta  = hWinSta;
//     GuiInitInfo->Desktop = hDesk;
//
//     /* Here GuiInitInfo contains original handles */
//
//     /* If we already have a console input thread on this desktop... */
//     if (DesktopConsoleThreadInfo.ThreadId != 0)
//     {
//         /* ... just use it... */
//         DPRINT("Using input thread InputThreadId = 0x%p\n", DesktopConsoleThreadInfo.ThreadId);
//         GuiInitInfo->InputThreadId = DesktopConsoleThreadInfo.ThreadId;
//         goto Quit;
//     }
//
//     /* ... otherwise create a new one. */
//
//     /* Initialize a startup event for the thread to signal it */
//     Status = NtCreateEvent(&GuiInitInfo->GuiThreadStartupEvent, EVENT_ALL_ACCESS,
//                            NULL, SynchronizationEvent, FALSE);
//     if (!NT_SUCCESS(Status))
//     {
//         Success = FALSE;
//         goto Quit;
//     }
//
//     /*
//      * Duplicate the desktop handle for the console input thread internal needs.
//      * If it happens to need also a window station handle in the future, then
//      * it is there that you also need to duplicate the window station handle!
//      *
//      * Note also that we are going to temporarily overwrite the stored handles
//      * in GuiInitInfo because it happens that we use also this structure to give
//      * the duplicated handles to the input thread that is going to initialize.
//      * After the input thread finishes its initialization, we restore the handles
//      * in GuiInitInfo to their old values.
//      */
//     Status = NtDuplicateObject(NtCurrentProcess(),
//                                hDesk,
//                                NtCurrentProcess(),
//                                (PHANDLE)&GuiInitInfo->Desktop,
//                                0, 0, DUPLICATE_SAME_ACCESS);
//     if (!NT_SUCCESS(Status))
//     {
//         Success = FALSE;
//         goto Quit;
//     }
//
//     /* Here GuiInitInfo contains duplicated handles */
//
//     Status = RtlCreateUserThread(NtCurrentProcess(),
//                                  NULL,
//                                  TRUE, // Start the thread in suspended state
//                                  0,
//                                  0,
//                                  0,
//                                  (PVOID)GuiConsoleInputThread,
//                                  (PVOID)GuiInitInfo,
//                                  &hInputThread,
//                                  &ClientId);
//     if (NT_SUCCESS(Status))
//     {
//         /* Add it as a static server thread and resume it */
//         CsrAddStaticServerThread(hInputThread, &ClientId, 0);
//         Status = NtResumeThread(hInputThread, NULL);
//     }
//     DPRINT("Thread creation hInputThread = 0x%p, InputThreadId = 0x%p, Status = 0x%08lx\n",
//            hInputThread, ClientId.UniqueThread, Status);
//
//     if (!NT_SUCCESS(Status) || hInputThread == NULL)
//     {
//         /* Close the thread's handle */
//         if (hInputThread) NtClose(hInputThread);
//
//         /* We need to close here the duplicated desktop handle */
//         CloseDesktop(GuiInitInfo->Desktop); // NtUserCloseDesktop
//
//         /* Close the startup event and bail out */
//         NtClose(GuiInitInfo->GuiThreadStartupEvent);
//
//         DPRINT1("CONSRV: Failed to create graphics console thread.\n");
//         Success = FALSE;
//         goto Quit;
//     }
//
//     /* No need to close hInputThread, this is done by CSR automatically */
//
//     /* Wait for the thread to finish its initialization, and close the startup event */
//     NtWaitForSingleObject(GuiInitInfo->GuiThreadStartupEvent, FALSE, NULL);
//     NtClose(GuiInitInfo->GuiThreadStartupEvent);
//
//     /*
//      * Save the input thread ID for later use, and restore the original handles.
//      * The copies are held by the console input thread.
//      */
//     GuiInitInfo->InputThreadId = (ULONG_PTR)ClientId.UniqueThread;
//     GuiInitInfo->WinSta  = hWinSta;
//     GuiInitInfo->Desktop = hDesk;
//
//     /* Here GuiInitInfo contains again original handles */
//
// Quit:
//     if (!Success)
//     {
//         /*
//          * Close the original handles. Do not use the copies in GuiInitInfo
//          * because we may have failed in the middle of the duplicate operation
//          * and the handles stored in GuiInitInfo may have changed.
//          */
//         CloseDesktop(hDesk); // NtUserCloseDesktop
//         CloseWindowStation(hWinSta); // NtUserCloseWindowStation
//     }
//
//     return Success;
// }
//
//
// /******************************************************************************
//  *                             GUI Console Driver                             *
//  ******************************************************************************/
//
// static VOID NTAPI
// GuiDeinitFrontEnd(IN OUT PFRONTEND This);
//
// static NTSTATUS NTAPI
// GuiInitFrontEnd(IN OUT PFRONTEND This,
//                 IN PCONSRV_CONSOLE Console)
// {
//     PGUI_INIT_INFO GuiInitInfo;
//     PGUI_CONSOLE_DATA GuiData;
//
//     if (This == NULL || Console == NULL || This->Context2 == NULL)
//         return STATUS_INVALID_PARAMETER;
//
//     ASSERT(This->Console == Console);
//
//     GuiInitInfo = This->Context2;
//
//     /* Terminal data allocation */
//     GuiData = ConsoleAllocHeap(HEAP_ZERO_MEMORY, sizeof(*GuiData));
//     if (!GuiData)
//     {
//         DPRINT1("CONSRV: Failed to create GUI_CONSOLE_DATA\n");
//         return STATUS_UNSUCCESSFUL;
//     }
//     /// /* HACK */ Console->FrontEndIFace.Context = (PVOID)GuiData; /* HACK */
//     GuiData->Console      = Console;
//     GuiData->ActiveBuffer = Console->ActiveBuffer;
//     GuiData->hWindow = NULL;
//     GuiData->IsWindowVisible = GuiInitInfo->IsWindowVisible;
//
//     /* The console can be resized */
//     Console->FixedSize = FALSE;
//
//     InitializeCriticalSection(&GuiData->Lock);
//
//     /*
//      * Set up GUI data
//      */
//     RtlCopyMemory(&GuiData->GuiInfo, &GuiInitInfo->TermInfo, sizeof(GuiInitInfo->TermInfo));
//
//     /* Initialize the icon handles */
//     if (GuiInitInfo->hIcon != NULL)
//         GuiData->hIcon = GuiInitInfo->hIcon;
//     else
//         GuiData->hIcon = ghDefaultIcon;
//
//     if (GuiInitInfo->hIconSm != NULL)
//         GuiData->hIconSm = GuiInitInfo->hIconSm;
//     else
//         GuiData->hIconSm = ghDefaultIconSm;
//
//     ASSERT(GuiData->hIcon && GuiData->hIconSm);
//
//     /* Mouse is shown by default with its default cursor shape */
//     GuiData->hCursor = ghDefaultCursor;
//     GuiData->MouseCursorRefCount = 0;
//
//     /* A priori don't ignore mouse events */
//     GuiData->IgnoreNextMouseEvent = FALSE;
//     /* Initialize HACK FOR CORE-8394. See conwnd.c!OnMouse for more details. */
//     GuiData->HackCORE8394IgnoreNextMove = FALSE;
//
//     /* Close button and the corresponding system menu item are enabled by default */
//     GuiData->IsCloseButtonEnabled = TRUE;
//
//     /* There is no user-reserved menu id range by default */
//     GuiData->CmdIdLow = GuiData->CmdIdHigh = 0;
//
//     /* Initialize the selection */
//     RtlZeroMemory(&GuiData->Selection, sizeof(GuiData->Selection));
//     GuiData->Selection.dwFlags = CONSOLE_NO_SELECTION;
//     RtlZeroMemory(&GuiData->dwSelectionCursor, sizeof(GuiData->dwSelectionCursor));
//     GuiData->LineSelection = FALSE; // Default to block selection
//     // TODO: Retrieve the selection mode via the registry.
//
//     GuiData->InputThreadId = GuiInitInfo->InputThreadId;
//     GuiData->WinSta  = GuiInitInfo->WinSta;
//     GuiData->Desktop = GuiInitInfo->Desktop;
//
//     /* Finally, finish to initialize the frontend structure */
//     This->Context  = GuiData;
//     ConsoleFreeHeap(This->Context2);
//     This->Context2 = NULL;
//
//     /*
//      * We need to wait until the GUI has been fully initialized
//      * to retrieve custom settings i.e. WindowSize etc...
//      * Ideally we could use SendNotifyMessage for this but its not
//      * yet implemented.
//      */
//     NtCreateEvent(&GuiData->hGuiInitEvent, EVENT_ALL_ACCESS,
//                   NULL, SynchronizationEvent, FALSE);
//     NtCreateEvent(&GuiData->hGuiTermEvent, EVENT_ALL_ACCESS,
//                   NULL, SynchronizationEvent, FALSE);
//
//     DPRINT("GUI - Checkpoint\n");
//
//     /* Create the terminal window */
//     PostThreadMessageW(GuiData->InputThreadId, PM_CREATE_CONSOLE, 0, (LPARAM)GuiData);
//
//     /* Wait until initialization has finished */
//     NtWaitForSingleObject(GuiData->hGuiInitEvent, FALSE, NULL);
//     DPRINT("OK we created the console window\n");
//     NtClose(GuiData->hGuiInitEvent);
//     GuiData->hGuiInitEvent = NULL;
//
//     /* Check whether we really succeeded in initializing the terminal window */
//     if (GuiData->hWindow == NULL)
//     {
//         DPRINT("GuiInitConsole - We failed at creating a new terminal window\n");
//         GuiDeinitFrontEnd(This);
//         return STATUS_UNSUCCESSFUL;
//     }
//
//     return STATUS_SUCCESS;
// }
//
// static VOID NTAPI
// GuiDeinitFrontEnd(IN OUT PFRONTEND This)
// {
//     PGUI_CONSOLE_DATA GuiData = This->Context;
//
//     DPRINT("Send PM_DESTROY_CONSOLE message and wait on hGuiTermEvent...\n");
//     PostThreadMessageW(GuiData->InputThreadId, PM_DESTROY_CONSOLE, 0, (LPARAM)GuiData);
//     NtWaitForSingleObject(GuiData->hGuiTermEvent, FALSE, NULL);
//     DPRINT("hGuiTermEvent set\n");
//     NtClose(GuiData->hGuiTermEvent);
//     GuiData->hGuiTermEvent = NULL;
//
//     CloseDesktop(GuiData->Desktop); // NtUserCloseDesktop
//     CloseWindowStation(GuiData->WinSta); // NtUserCloseWindowStation
//
//     DPRINT("Destroying icons !! - GuiData->hIcon = 0x%p ; ghDefaultIcon = 0x%p ; GuiData->hIconSm = 0x%p ; ghDefaultIconSm = 0x%p\n",
//             GuiData->hIcon, ghDefaultIcon, GuiData->hIconSm, ghDefaultIconSm);
//     if (GuiData->hIcon != NULL && GuiData->hIcon != ghDefaultIcon)
//     {
//         DPRINT("Destroy hIcon\n");
//         DestroyIcon(GuiData->hIcon);
//     }
//     if (GuiData->hIconSm != NULL && GuiData->hIconSm != ghDefaultIconSm)
//     {
//         DPRINT("Destroy hIconSm\n");
//         DestroyIcon(GuiData->hIconSm);
//     }
//
//     This->Context = NULL;
//     DeleteCriticalSection(&GuiData->Lock);
//     ConsoleFreeHeap(GuiData);
//
//     DPRINT("Quit GuiDeinitFrontEnd\n");
// }
//
// static VOID NTAPI
// GuiDrawRegion(IN OUT PFRONTEND This,
//               SMALL_RECT* Region)
// {
//     PGUI_CONSOLE_DATA GuiData = This->Context;
//
//     /* Do nothing if the window is hidden */
//     if (!GuiData->IsWindowVisible) return;
//
//     DrawRegion(GuiData, Region);
// }
//
// static VOID NTAPI
// GuiWriteStream(IN OUT PFRONTEND This,
//                SMALL_RECT* Region,
//                SHORT CursorStartX,
//                SHORT CursorStartY,
//                UINT ScrolledLines,
//                PWCHAR Buffer,
//                UINT Length)
// {
//     PGUI_CONSOLE_DATA GuiData = This->Context;
//     PCONSOLE_SCREEN_BUFFER Buff;
//     SHORT CursorEndX, CursorEndY;
//     RECT ScrollRect;
//
//     if (NULL == GuiData || NULL == GuiData->hWindow) return;
//
//     /* Do nothing if the window is hidden */
//     if (!GuiData->IsWindowVisible) return;
//
//     Buff = GuiData->ActiveBuffer;
//     if (GetType(Buff) != TEXTMODE_BUFFER) return;
//
//     if (0 != ScrolledLines)
//     {
//         ScrollRect.left = 0;
//         ScrollRect.top = 0;
//         ScrollRect.right = Buff->ViewSize.X * GuiData->CharWidth;
//         ScrollRect.bottom = Region->Top * GuiData->CharHeight;
//
//         ScrollWindowEx(GuiData->hWindow,
//                        0,
//                        -(int)(ScrolledLines * GuiData->CharHeight),
//                        &ScrollRect,
//                        NULL,
//                        NULL,
//                        NULL,
//                        SW_INVALIDATE);
//     }
//
//     DrawRegion(GuiData, Region);
//
//     if (CursorStartX < Region->Left || Region->Right < CursorStartX
//             || CursorStartY < Region->Top || Region->Bottom < CursorStartY)
//     {
//         InvalidateCell(GuiData, CursorStartX, CursorStartY);
//     }
//
//     CursorEndX = Buff->CursorPosition.X;
//     CursorEndY = Buff->CursorPosition.Y;
//     if ((CursorEndX < Region->Left || Region->Right < CursorEndX
//             || CursorEndY < Region->Top || Region->Bottom < CursorEndY)
//             && (CursorEndX != CursorStartX || CursorEndY != CursorStartY))
//     {
//         InvalidateCell(GuiData, CursorEndX, CursorEndY);
//     }
//
//     // HACK!!
//     // Set up the update timer (very short interval) - this is a "hack" for getting the OS to
//     // repaint the window without having it just freeze up and stay on the screen permanently.
//     Buff->CursorBlinkOn = TRUE;
//     SetTimer(GuiData->hWindow, CONGUI_UPDATE_TIMER, CONGUI_UPDATE_TIME, NULL);
// }
//
// /* static */ VOID NTAPI
// GuiRingBell(IN OUT PFRONTEND This)
// {
//     PGUI_CONSOLE_DATA GuiData = This->Context;
//
//     /* Emit an error beep sound */
//     SendNotifyMessage(GuiData->hWindow, PM_CONSOLE_BEEP, 0, 0);
// }
//
// static BOOL NTAPI
// GuiSetCursorInfo(IN OUT PFRONTEND This,
//                  PCONSOLE_SCREEN_BUFFER Buff)
// {
//     PGUI_CONSOLE_DATA GuiData = This->Context;
//
//     /* Do nothing if the window is hidden */
//     if (!GuiData->IsWindowVisible) return TRUE;
//
//     if (GuiData->ActiveBuffer == Buff)
//     {
//         InvalidateCell(GuiData, Buff->CursorPosition.X, Buff->CursorPosition.Y);
//     }
//
//     return TRUE;
// }
//
// static BOOL NTAPI
// GuiSetScreenInfo(IN OUT PFRONTEND This,
//                  PCONSOLE_SCREEN_BUFFER Buff,
//                  SHORT OldCursorX,
//                  SHORT OldCursorY)
// {
//     PGUI_CONSOLE_DATA GuiData = This->Context;
//
//     /* Do nothing if the window is hidden */
//     if (!GuiData->IsWindowVisible) return TRUE;
//
//     if (GuiData->ActiveBuffer == Buff)
//     {
//         /* Redraw char at old position (remove cursor) */
//         InvalidateCell(GuiData, OldCursorX, OldCursorY);
//         /* Redraw char at new position (show cursor) */
//         InvalidateCell(GuiData, Buff->CursorPosition.X, Buff->CursorPosition.Y);
//     }
//
//     return TRUE;
// }
//
// static VOID NTAPI
// GuiResizeTerminal(IN OUT PFRONTEND This)
// {
//     PGUI_CONSOLE_DATA GuiData = This->Context;
//
//     /* Resize the window to the user's values */
//     PostMessageW(GuiData->hWindow, PM_RESIZE_TERMINAL, 0, 0);
// }
//
// static VOID NTAPI
// GuiSetActiveScreenBuffer(IN OUT PFRONTEND This)
// {
//     PGUI_CONSOLE_DATA GuiData = This->Context;
//     PCONSOLE_SCREEN_BUFFER ActiveBuffer;
//     HPALETTE hPalette;
//
//     EnterCriticalSection(&GuiData->Lock);
//     GuiData->WindowSizeLock = TRUE;
//
//     InterlockedExchangePointer((PVOID*)&GuiData->ActiveBuffer,
//                                ConDrvGetActiveScreenBuffer((PCONSOLE)GuiData->Console));
//
//     GuiData->WindowSizeLock = FALSE;
//     LeaveCriticalSection(&GuiData->Lock);
//
//     ActiveBuffer = GuiData->ActiveBuffer;
//
//     /* Change the current palette */
//     if (ActiveBuffer->PaletteHandle == NULL)
//         hPalette = GuiData->hSysPalette;
//     else
//         hPalette = ActiveBuffer->PaletteHandle;
//
//     DPRINT("GuiSetActiveScreenBuffer using palette 0x%p\n", hPalette);
//
//     /* Set the new palette for the framebuffer */
//     SelectPalette(GuiData->hMemDC, hPalette, FALSE);
//
//     /* Specify the use of the system palette for the framebuffer */
//     SetSystemPaletteUse(GuiData->hMemDC, ActiveBuffer->PaletteUsage);
//
//     /* Realize the (logical) palette */
//     RealizePalette(GuiData->hMemDC);
//
//     GuiResizeTerminal(This);
//     // ConioDrawConsole(Console);
// }
//
// static VOID NTAPI
// GuiReleaseScreenBuffer(IN OUT PFRONTEND This,
//                        IN PCONSOLE_SCREEN_BUFFER ScreenBuffer)
// {
//     PGUI_CONSOLE_DATA GuiData = This->Context;
//
//     /*
//      * If we were notified to release a screen buffer that is not actually
//      * ours, then just ignore the notification...
//      */
//     if (ScreenBuffer != GuiData->ActiveBuffer) return;
//
//     /*
//      * ... else, we must release our active buffer. Two cases are present:
//      * - If ScreenBuffer (== GuiData->ActiveBuffer) IS NOT the console
//      *   active screen buffer, then we can safely switch to it.
//      * - If ScreenBuffer IS the console active screen buffer, we must release
//      *   it ONLY.
//      */
//
//     /* Release the old active palette and set the default one */
//     if (GetCurrentObject(GuiData->hMemDC, OBJ_PAL) == ScreenBuffer->PaletteHandle)
//     {
//         /* Set the new palette */
//         SelectPalette(GuiData->hMemDC, GuiData->hSysPalette, FALSE);
//     }
//
//     /* Set the adequate active screen buffer */
//     if (ScreenBuffer != GuiData->Console->ActiveBuffer)
//     {
//         GuiSetActiveScreenBuffer(This);
//     }
//     else
//     {
//         EnterCriticalSection(&GuiData->Lock);
//         GuiData->WindowSizeLock = TRUE;
//
//         InterlockedExchangePointer((PVOID*)&GuiData->ActiveBuffer, NULL);
//
//         GuiData->WindowSizeLock = FALSE;
//         LeaveCriticalSection(&GuiData->Lock);
//     }
// }
//
// static BOOL NTAPI
// GuiSetMouseCursor(IN OUT PFRONTEND This,
//                   HCURSOR CursorHandle);
//
// static VOID NTAPI
// GuiRefreshInternalInfo(IN OUT PFRONTEND This)
// {
//     PGUI_CONSOLE_DATA GuiData = This->Context;
//
//     /* Update the console leader information held by the window */
//     SetConWndConsoleLeaderCID(GuiData);
//
//     /*
//      * HACK:
//      * We reset the cursor here so that, when a console app quits, we reset
//      * the cursor to the default one. It's quite a hack since it doesn't proceed
//      * per - console process... This must be fixed.
//      *
//      * See GuiInitConsole(...) for more information.
//      */
//
//     /* Mouse is shown by default with its default cursor shape */
//     GuiData->MouseCursorRefCount = 0; // Reinitialize the reference counter
//     GuiSetMouseCursor(This, NULL);
// }
//
// static VOID NTAPI
// GuiChangeTitle(IN OUT PFRONTEND This)
// {
//     PGUI_CONSOLE_DATA GuiData = This->Context;
//     PostMessageW(GuiData->hWindow, PM_CONSOLE_SET_TITLE, 0, 0);
// }
//
// static BOOL NTAPI
// GuiChangeIcon(IN OUT PFRONTEND This,
//               HICON IconHandle)
// {
//     PGUI_CONSOLE_DATA GuiData = This->Context;
//     HICON hIcon, hIconSm;
//
//     if (IconHandle == NULL)
//     {
//         hIcon   = ghDefaultIcon;
//         hIconSm = ghDefaultIconSm;
//     }
//     else
//     {
//         hIcon   = CopyIcon(IconHandle);
//         hIconSm = CopyIcon(IconHandle);
//     }
//
//     if (hIcon == NULL)
//         return FALSE;
//
//     if (hIcon != GuiData->hIcon)
//     {
//         if (GuiData->hIcon != NULL && GuiData->hIcon != ghDefaultIcon)
//         {
//             DestroyIcon(GuiData->hIcon);
//         }
//         if (GuiData->hIconSm != NULL && GuiData->hIconSm != ghDefaultIconSm)
//         {
//             DestroyIcon(GuiData->hIconSm);
//         }
//
//         GuiData->hIcon   = hIcon;
//         GuiData->hIconSm = hIconSm;
//
//         DPRINT("Set icons in GuiChangeIcon\n");
//         PostMessageW(GuiData->hWindow, WM_SETICON, ICON_BIG  , (LPARAM)GuiData->hIcon  );
//         PostMessageW(GuiData->hWindow, WM_SETICON, ICON_SMALL as WPARAM, (LPARAM)GuiData->hIconSm);
//     }
//
//     return TRUE;
// }
//
// static HDESK NTAPI
// GuiGetThreadConsoleDesktop(IN OUT PFRONTEND This)
// {
//     PGUI_CONSOLE_DATA GuiData = This->Context;
//     return GuiData->Desktop;
// }
//
// static HWND NTAPI
// GuiGetConsoleWindowHandle(IN OUT PFRONTEND This)
// {
//     PGUI_CONSOLE_DATA GuiData = This->Context;
//     return GuiData->hWindow;
// }
//
// static VOID NTAPI
// GuiGetLargestConsoleWindowSize(IN OUT PFRONTEND This,
//                                PCOORD pSize)
// {
//     PGUI_CONSOLE_DATA GuiData = This->Context;
//     PCONSOLE_SCREEN_BUFFER ActiveBuffer;
//     HMONITOR hMonitor;
//     MONITORINFO MonitorInfo;
//     LONG Width, Height;
//     UINT WidthUnit, HeightUnit;
//
//     if (!pSize) return;
//
//     /*
//      * Retrieve the monitor that is mostly covered by the current console window;
//      * default to primary monitor otherwise.
//      */
//     MonitorInfo.cbSize = sizeof(MonitorInfo);
//     hMonitor = MonitorFromWindow(GuiData->hWindow, MONITOR_DEFAULTTOPRIMARY);
//     if (hMonitor && GetMonitorInfoW(hMonitor, &MonitorInfo))
//     {
//         /* Retrieve the width and height of the client area of this monitor */
//         Width  = MonitorInfo.rcWork.right - MonitorInfo.rcWork.left;
//         Height = MonitorInfo.rcWork.bottom - MonitorInfo.rcWork.top;
//     }
//     else
//     {
//         /*
//          * Retrieve the width and height of the client area for a full-screen
//          * window on the primary display monitor.
//          */
//         Width  = GetSystemMetrics(SM_CXFULLSCREEN as i32);
//         Height = GetSystemMetrics(SM_CYFULLSCREEN as i32);
//
//         // RECT WorkArea;
//         // SystemParametersInfoW(SPI_GETWORKAREA, 0, &WorkArea, 0);
//         // Width  = WorkArea.right;
//         // Height = WorkArea.bottom;
//     }
//
//     ActiveBuffer = GuiData->ActiveBuffer;
// #if 0
//     // NOTE: This would be surprising if we wouldn't have an associated buffer...
//     if (ActiveBuffer)
// #endif
//         GetScreenBufferSizeUnits(ActiveBuffer, GuiData, &WidthUnit, &HeightUnit);
// #if 0
//     else
//         /* Default: graphics mode */
//         WidthUnit = HeightUnit = 1;
// #endif
//
//     Width  -= (2 * (GetSystemMetrics(SM_CXFRAME as i32) + GetSystemMetrics(SM_CXEDGE as i32)));
//     Height -= (2 * (GetSystemMetrics(SM_CYFRAME as i32) + GetSystemMetrics(SM_CYEDGE as i32)) + GetSystemMetrics(SM_CYCAPTION as i32));
//
//     if (Width  < 0) Width  = 0;
//     if (Height < 0) Height = 0;
//
//     pSize->X = (SHORT)(Width  / (int)WidthUnit ) /* HACK */ + 2;
//     pSize->Y = (SHORT)(Height / (int)HeightUnit) /* HACK */ + 1;
// }
//
// static BOOL NTAPI
// GuiGetSelectionInfo(IN OUT PFRONTEND This,
//                     PCONSOLE_SELECTION_INFO pSelectionInfo)
// {
//     PGUI_CONSOLE_DATA GuiData = This->Context;
//
//     if (pSelectionInfo == NULL) return FALSE;
//
//     ZeroMemory(pSelectionInfo, sizeof(*pSelectionInfo));
//     if (GuiData->Selection.dwFlags != CONSOLE_NO_SELECTION)
//         RtlCopyMemory(pSelectionInfo, &GuiData->Selection, sizeof(*pSelectionInfo));
//
//     return TRUE;
// }
//
// static BOOL NTAPI
// GuiSetPalette(IN OUT PFRONTEND This,
//               HPALETTE PaletteHandle,
//               UINT PaletteUsage)
// {
//     PGUI_CONSOLE_DATA GuiData = This->Context;
//     HPALETTE OldPalette;
//
//     // if (GetType(GuiData->ActiveBuffer) != GRAPHICS_BUFFER) return FALSE;
//     if (PaletteHandle == NULL) return FALSE;
//
//     /* Set the new palette for the framebuffer */
//     OldPalette = SelectPalette(GuiData->hMemDC, PaletteHandle, FALSE);
//     if (OldPalette == NULL) return FALSE;
//
//     /* Specify the use of the system palette for the framebuffer */
//     SetSystemPaletteUse(GuiData->hMemDC, PaletteUsage);
//
//     /* Realize the (logical) palette */
//     RealizePalette(GuiData->hMemDC);
//
//     /* Save the original system palette handle */
//     if (GuiData->hSysPalette == NULL) GuiData->hSysPalette = OldPalette;
//
//     return TRUE;
// }
//
// static BOOL NTAPI
// GuiSetCodePage(IN OUT PFRONTEND This,
//                UINT CodePage)
// {
//     PGUI_CONSOLE_DATA GuiData = This->Context;
//
//     /*
//      * Attempt to reinitialize the current font for the new code page,
//      * trying to keep the current font with the same characteristics.
//      * If the current font does not support the new code page, switch
//      * to a different font supporting the code page but having similar
//      * characteristics.
//      * If no font can be found for this code page, stay using the
//      * original font and refuse changing the code page.
//      */
//     if (!InitFonts(GuiData,
//                    GuiData->GuiInfo.FaceName,
//                    GuiData->GuiInfo.FontWeight,
//                    GuiData->GuiInfo.FontFamily,
//                    GuiData->GuiInfo.FontSize,
//                    CodePage, FALSE))
//     {
//         DPRINT1("Failed to initialize font '%S' for code page %d - Refuse CP change\n",
//                 GuiData->GuiInfo.FaceName, CodePage);
//         return FALSE;
//     }
//
//     return TRUE;
// }
//
// static ULONG NTAPI
// GuiGetDisplayMode(IN OUT PFRONTEND This)
// {
//     PGUI_CONSOLE_DATA GuiData = This->Context;
//     ULONG DisplayMode = 0;
//
//     if (GuiData->GuiInfo.FullScreen)
//         DisplayMode |= CONSOLE_FULLSCREEN_HARDWARE; // CONSOLE_FULLSCREEN
//     else
//         DisplayMode |= CONSOLE_WINDOWED;
//
//     return DisplayMode;
// }
//
// static BOOL NTAPI
// GuiSetDisplayMode(IN OUT PFRONTEND This,
//                   ULONG NewMode)
// {
//     PGUI_CONSOLE_DATA GuiData = This->Context;
//     BOOL FullScreen;
//
//     if (NewMode & ~(CONSOLE_FULLSCREEN_MODE | CONSOLE_WINDOWED_MODE))
//         return FALSE;
//
//     /* Do nothing if the window is hidden */
//     if (!GuiData->IsWindowVisible) return TRUE;
//
//     FullScreen = ((NewMode & CONSOLE_FULLSCREEN_MODE) != 0);
//
//     if (FullScreen != GuiData->GuiInfo.FullScreen)
//     {
//         SwitchFullScreen(GuiData, FullScreen);
//     }
//
//     return TRUE;
// }
//
// static INT NTAPI
// GuiShowMouseCursor(IN OUT PFRONTEND This,
//                    BOOL Show)
// {
//     PGUI_CONSOLE_DATA GuiData = This->Context;
//
//     if (GuiData->IsWindowVisible)
//     {
//         /* Set the reference count */
//         if (Show) ++GuiData->MouseCursorRefCount;
//         else      --GuiData->MouseCursorRefCount;
//
//         /* Effectively show (or hide) the cursor (use special values for (w|l)Param) */
//         PostMessageW(GuiData->hWindow, WM_SETCURSOR, -1, -1);
//     }
//
//     return GuiData->MouseCursorRefCount;
// }
//
// static BOOL NTAPI
// GuiSetMouseCursor(IN OUT PFRONTEND This,
//                   HCURSOR CursorHandle)
// {
//     PGUI_CONSOLE_DATA GuiData = This->Context;
//
//     /* Do nothing if the window is hidden */
//     if (!GuiData->IsWindowVisible) return TRUE;
//
//     /*
//      * Set the cursor's handle. If the given handle is NULL,
//      * then restore the default cursor.
//      */
//     GuiData->hCursor = (CursorHandle ? CursorHandle : ghDefaultCursor);
//
//     /* Effectively modify the shape of the cursor (use special values for (w|l)Param) */
//     PostMessageW(GuiData->hWindow, WM_SETCURSOR, -1, -1);
//
//     return TRUE;
// }
//
// static HMENU NTAPI
// GuiMenuControl(IN OUT PFRONTEND This,
//                UINT CmdIdLow,
//                UINT CmdIdHigh)
// {
//     PGUI_CONSOLE_DATA GuiData = This->Context;
//
//     GuiData->CmdIdLow  = CmdIdLow ;
//     GuiData->CmdIdHigh = CmdIdHigh;
//
//     return GuiData->hSysMenu;
// }
//
// static BOOL NTAPI
// GuiSetMenuClose(IN OUT PFRONTEND This,
//                 BOOL Enable)
// {
//     /*
//      * NOTE: See http://www.mail-archive.com/harbour@harbour-project.org/msg27509.html
//      * or http://harbour-devel.1590103.n2.nabble.com/Question-about-hb-gt-win-CtrlHandler-usage-td4670862i20.html (DEAD_LINK)
//      * for more information.
//      */
//
//     PGUI_CONSOLE_DATA GuiData = This->Context;
//
//     if (GuiData->hSysMenu == NULL) return FALSE;
//
//     GuiData->IsCloseButtonEnabled = Enable;
//     EnableMenuItem(GuiData->hSysMenu, SC_CLOSE, MF_BYCOMMAND | (Enable ? MF_ENABLED : MF_GRAYED));
//
//     return TRUE;
// }
//
// static FRONTEND_VTBL GuiVtbl =
// {
//     GuiInitFrontEnd,
//     GuiDeinitFrontEnd,
//     GuiDrawRegion,
//     GuiWriteStream,
//     GuiRingBell,
//     GuiSetCursorInfo,
//     GuiSetScreenInfo,
//     GuiResizeTerminal,
//     GuiSetActiveScreenBuffer,
//     GuiReleaseScreenBuffer,
//     GuiRefreshInternalInfo,
//     GuiChangeTitle,
//     GuiChangeIcon,
//     GuiGetThreadConsoleDesktop,
//     GuiGetConsoleWindowHandle,
//     GuiGetLargestConsoleWindowSize,
//     GuiGetSelectionInfo,
//     GuiSetPalette,
//     GuiSetCodePage,
//     GuiGetDisplayMode,
//     GuiSetDisplayMode,
//     GuiShowMouseCursor,
//     GuiSetMouseCursor,
//     GuiMenuControl,
//     GuiSetMenuClose,
// };
//
//
// NTSTATUS NTAPI
// GuiLoadFrontEnd(IN OUT PFRONTEND FrontEnd,
//                 IN OUT PCONSOLE_STATE_INFO ConsoleInfo,
//                 IN OUT PCONSOLE_INIT_INFO ConsoleInitInfo,
//                 IN HANDLE ConsoleLeaderProcessHandle)
// {
//     PCONSOLE_START_INFO ConsoleStartInfo;
//     PGUI_INIT_INFO GuiInitInfo;
//     USEROBJECTFLAGS UserObjectFlags;
//
//     if (FrontEnd == NULL || ConsoleInfo == NULL || ConsoleInitInfo == NULL)
//         return STATUS_INVALID_PARAMETER;
//
//     ConsoleStartInfo = ConsoleInitInfo->ConsoleStartInfo;
//
//     /*
//      * Initialize a private initialization info structure for later use.
//      * It must be freed by a call to GuiUnloadFrontEnd or GuiInitFrontEnd.
//      */
//     GuiInitInfo = ConsoleAllocHeap(HEAP_ZERO_MEMORY, sizeof(*GuiInitInfo));
//     if (GuiInitInfo == NULL) return STATUS_NO_MEMORY;
//
//     /* Initialize GUI terminal emulator common functionalities */
//     if (!GuiInit(ConsoleInitInfo, ConsoleLeaderProcessHandle, GuiInitInfo))
//     {
//         ConsoleFreeHeap(GuiInitInfo);
//         return STATUS_UNSUCCESSFUL;
//     }
//
//     GuiInitInfo->IsWindowVisible = ConsoleInitInfo->IsWindowVisible;
//     if (GuiInitInfo->IsWindowVisible)
//     {
//         /* Don't show the console if the window station is not interactive */
//         if (GetUserObjectInformationW(GuiInitInfo->WinSta,
//                                       UOI_FLAGS,
//                                       &UserObjectFlags,
//                                       sizeof(UserObjectFlags),
//                                       NULL))
//         {
//             if (!(UserObjectFlags.dwFlags & WSF_VISIBLE))
//                 GuiInitInfo->IsWindowVisible = FALSE;
//         }
//     }
//
//     /*
//      * Load terminal settings
//      */
// #if 0
//     /* Impersonate the caller in order to retrieve settings in its context */
//     // if (!CsrImpersonateClient(NULL))
//         // return STATUS_UNSUCCESSFUL;
//     CsrImpersonateClient(NULL);
//
//     /* 1. Load the default settings */
//     GuiConsoleGetDefaultSettings(&GuiInitInfo->TermInfo);
// #endif
//
//     GuiInitInfo->TermInfo.ShowWindow = SW_SHOWNORMAL;
//
//     if (GuiInitInfo->IsWindowVisible)
//     {
//         /* 2. Load the remaining console settings via the registry */
//         if ((ConsoleStartInfo->dwStartupFlags & STARTF_TITLEISLINKNAME) == 0)
//         {
// #if 0
//             /* Load the terminal information from the registry */
//             GuiConsoleReadUserSettings(&GuiInitInfo->TermInfo);
// #endif
//
//             /*
//              * Now, update them with the properties the user might gave to us
//              * via the STARTUPINFO structure before calling CreateProcess
//              * (and which was transmitted via the ConsoleStartInfo structure).
//              * We therefore overwrite the values read in the registry.
//              */
//             if (ConsoleStartInfo->dwStartupFlags & STARTF_USESHOWWINDOW)
//             {
//                 GuiInitInfo->TermInfo.ShowWindow = ConsoleStartInfo->wShowWindow;
//             }
//             if (ConsoleStartInfo->dwStartupFlags & STARTF_USEPOSITION)
//             {
//                 ConsoleInfo->AutoPosition = FALSE;
//                 ConsoleInfo->WindowPosition.x = ConsoleStartInfo->dwWindowOrigin.X;
//                 ConsoleInfo->WindowPosition.y = ConsoleStartInfo->dwWindowOrigin.Y;
//             }
//             if (ConsoleStartInfo->dwStartupFlags & STARTF_RUNFULLSCREEN)
//             {
//                 ConsoleInfo->FullScreen = TRUE;
//             }
//         }
//     }
//
// #if 0
//     /* Revert impersonation */
//     CsrRevertToSelf();
// #endif
//
//     // Font data
//     StringCchCopyNW(GuiInitInfo->TermInfo.FaceName, ARRAYSIZE(GuiInitInfo->TermInfo.FaceName),
//                     ConsoleInfo->FaceName, ARRAYSIZE(ConsoleInfo->FaceName));
//     GuiInitInfo->TermInfo.FontFamily = ConsoleInfo->FontFamily;
//     GuiInitInfo->TermInfo.FontSize   = ConsoleInfo->FontSize;
//     GuiInitInfo->TermInfo.FontWeight = ConsoleInfo->FontWeight;
//
//     // Display
//     GuiInitInfo->TermInfo.FullScreen   = ConsoleInfo->FullScreen;
//     GuiInitInfo->TermInfo.AutoPosition = ConsoleInfo->AutoPosition;
//     GuiInitInfo->TermInfo.WindowOrigin = ConsoleInfo->WindowPosition;
//
//     /* Initialize the icon handles */
//     // if (ConsoleStartInfo->hIcon != NULL)
//         GuiInitInfo->hIcon = ConsoleStartInfo->hIcon;
//     // else
//         // GuiInitInfo->hIcon = ghDefaultIcon;
//
//     // if (ConsoleStartInfo->hIconSm != NULL)
//         GuiInitInfo->hIconSm = ConsoleStartInfo->hIconSm;
//     // else
//         // GuiInitInfo->hIconSm = ghDefaultIconSm;
//
//     // ASSERT(GuiInitInfo->hIcon && GuiInitInfo->hIconSm);
//
//     /* Finally, initialize the frontend structure */
//     FrontEnd->Vtbl     = &GuiVtbl;
//     FrontEnd->Context  = NULL;
//     FrontEnd->Context2 = GuiInitInfo;
//
//     return STATUS_SUCCESS;
// }
//
// NTSTATUS NTAPI
// GuiUnloadFrontEnd(IN OUT PFRONTEND FrontEnd)
// {
//     if (FrontEnd == NULL) return STATUS_INVALID_PARAMETER;
//
//     if (FrontEnd->Context ) GuiDeinitFrontEnd(FrontEnd);
//     if (FrontEnd->Context2) ConsoleFreeHeap(FrontEnd->Context2);
//
//     return STATUS_SUCCESS;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
