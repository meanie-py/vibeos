use winsrv_sys::nt::*;
use super::super::super::{_stricmp, strchr};
use super::super::super::condrv::text::*;
use super::super::super::{GetType, wcslen};
use super::super::terminal::ConioDrawConsole;
use crate::consrv::{ConsoleAllocHeap, ConsoleFreeHeap};
use winsrv_sys::winsrv::*;
use winsrv_sys::consts::*;
// Disambiguate winsrv_sys::winsrv:: (bindgen) vs sibling Rust impl.
use super::super::super::init::ConSrvDllInstance;
use super::super::super::condrv::console::ConDrvValidateConsoleUnsafe;


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


// IOCTL_CONSOLE_* — from sdk/include/reactos/drivers/blue/ntddblue.h.
// Hand-evaluated CTL_CODE values to avoid pulling that header into bindgen.
// CTL_CODE(FILE_DEVICE_SCREEN=0x1C, fn, method, access)
//   = (DeviceType << 16) | (Access << 14) | (Function << 2) | Method
// FILE_DEVICE_SCREEN=0x1C, FILE_READ_ACCESS=1, FILE_WRITE_ACCESS=2,
// METHOD_BUFFERED=0, METHOD_IN_DIRECT=1, METHOD_OUT_DIRECT=2.
pub const IOCTL_CONSOLE_WRITE_OUTPUT_CHARACTER: u32 = (0x1C << 16) | (2 << 14) | (0x822 << 2) | 1;
pub const IOCTL_CONSOLE_DRAW:                    u32 = (0x1C << 16) | (2 << 14) | (0x830 << 2) | 1;
pub const IOCTL_CONSOLE_LOADFONT:                u32 = (0x1C << 16) | (2 << 14) | (0x840 << 2) | 1;
pub const IOCTL_CONSOLE_SET_CURSOR_INFO:         u32 = (0x1C << 16) | (2 << 14) | (0x806 << 2) | 0;
pub const IOCTL_CONSOLE_SET_SCREEN_BUFFER_INFO:  u32 = (0x1C << 16) | (2 << 14) | (0x807 << 2) | 0;
pub const STATUS_INVALID_BUFFER_SIZE: NTSTATUS = 0xC0000206u32 as NTSTATUS;

extern "C" {
    pub fn wcscpy(dest: *mut u16, src: *const u16) -> *mut u16;
    pub fn wcscat(dest: *mut u16, src: *const u16) -> *mut u16;
    pub fn RtlStringCbPrintfA(pszDest: *mut i8, cbDest: usize, pszFormat: *const i8, ...) -> NTSTATUS;
}
unsafe extern "system" {
    pub fn DeviceIoControl( hDevice: HANDLE, dwIoControlCode: u32, lpInBuffer: *const core::ffi::c_void, nInBufferSize: u32, lpOutBuffer: *mut core::ffi::c_void, nOutBufferSize: u32, lpBytesReturned: *mut u32, lpOverlapped: *mut core::ffi::c_void, ) -> BOOL;
}
extern "system" {
    pub fn Beep(dwFreq: u32, dwDuration: u32) -> BOOL;
    pub fn Sleep(dwMilliseconds: u32);
}

// tuiterm.c-internal helpers (file-static in C). Phase 5.6 will
// promote these from the gui/ translation, or land as Rust stubs.
extern "system" {
    pub fn ConioProcessKey(Console: PCONSRV_CONSOLE, msg: *mut tagMSG);
    pub fn ConioEffectiveCursorSize(Console: PCONSRV_CONSOLE, n: u32) -> u32;
    pub fn ConsoleOutputUnicodeToAnsiChar(Console: PCONSRV_CONSOLE, dChar: *mut i8, sWChar: *mut u16) -> u8;
    pub fn RtlUnicodeStringToAnsiSize(SourceString: *const _UNICODE_STRING) -> u32;
}

// Win32 user32 helpers tuiterm.c relies on.
extern "system" {
    pub fn CreateWindowW(
        lpClassName: LPCWSTR, lpWindowName: LPCWSTR,
        dwStyle: u32, X: i32, Y: i32, nWidth: i32, nHeight: i32,
        hWndParent: HWND, hMenu: HMENU, hInstance: HINSTANCE,
        lpParam: *mut core::ffi::c_void,
    ) -> HWND;
}
#[inline(always)]
#[allow(non_snake_case)]
fn LOWORD(x: u32) -> u16 { (x & 0xFFFF) as u16 }

// ---- file-scope preamble (translated) ----
#[repr(C)]
pub struct _CFHEADER {
    pub Signature: ULONG,
    pub Reserved1: ULONG,
    pub CabinetSize: ULONG,
    pub Reserved2: ULONG,
    pub FileTableOffset: ULONG,
    pub Reserved3: ULONG,
    pub Version: USHORT,
    pub FolderCount: USHORT,
    pub FileCount: USHORT,
    pub Flags: USHORT,
    pub SetID: USHORT,
    pub CabinetNumber: USHORT,
}

pub type PCFHEADER = *mut _CFHEADER;
pub type CFHEADER = _CFHEADER;

#[repr(C)]
pub struct _CFFILE {
    pub FileSize: ULONG,
    pub FileOffset: ULONG,
    pub FileControlID: USHORT,
    pub FileDate: USHORT,
    pub FileTime: USHORT,
    pub Attributes: USHORT,
}

pub type PCFFILE = *mut _CFFILE;

pub const CAB_SIGNATURE: ULONG = 0x4643534D;
pub const CAB_VERSION: USHORT = 0x0103;

#[repr(C)]
pub struct _TUI_CONSOLE_DATA {
    pub Lock: CRITICAL_SECTION,
    pub Entry: LIST_ENTRY,
    pub hWindow: HWND,
    pub Console: PCONSRV_CONSOLE,
    pub ActiveBuffer: PCONSOLE_SCREEN_BUFFER,
}

pub type PTUI_CONSOLE_DATA = *mut _TUI_CONSOLE_DATA;
pub type TUI_CONSOLE_DATA = _TUI_CONSOLE_DATA;

// tuiterm.c file-static macros for the doubly-linked TUI_CONSOLE_DATA list.
#[inline(always)]
#[allow(non_snake_case)]
unsafe fn GetNextConsole(Console: PTUI_CONSOLE_DATA) -> PTUI_CONSOLE_DATA {
    (((*Console).Entry.Flink as usize) - core::mem::offset_of!(TUI_CONSOLE_DATA, Entry)) as PTUI_CONSOLE_DATA
}
#[inline(always)]
#[allow(non_snake_case)]
unsafe fn GetPrevConsole(Console: PTUI_CONSOLE_DATA) -> PTUI_CONSOLE_DATA {
    (((*Console).Entry.Blink as usize) - core::mem::offset_of!(TUI_CONSOLE_DATA, Entry)) as PTUI_CONSOLE_DATA
}

static mut VirtConsList: LIST_ENTRY = LIST_ENTRY { Flink: core::ptr::null_mut(), Blink: core::ptr::null_mut() };
static mut ActiveConsole: PTUI_CONSOLE_DATA = core::ptr::null_mut();
static mut ActiveVirtConsLock: CRITICAL_SECTION = unsafe { core::mem::zeroed() };
static mut PhysicalConsoleSize: COORD = COORD { X: 0, Y: 0 };
static mut ConsoleDeviceHandle: HANDLE = core::ptr::null_mut();
static mut ConsInitialized: BOOL = 0;

// `FRONTEND_VTBL` field types use `extern "stdcall"` on i386 and
// `extern "C"` on amd64/arm64 (per bindgen translating __stdcall).
// Our Tui* impls are `extern "system"` (same ABI per arch, distinct
// Rust type). Lazy-init via `*const ()` runtime-transmute (mirrors
// DummyVtbl in consrv/condrv/dummyterm.rs).
static mut TuiVtbl: FRONTEND_VTBL = FRONTEND_VTBL {
    InitFrontEnd: None,
    DeinitFrontEnd: None,
    DrawRegion: None,
    WriteStream: None,
    RingBell: None,
    SetCursorInfo: None,
    SetScreenInfo: None,
    ResizeTerminal: None,
    SetActiveScreenBuffer: None,
    ReleaseScreenBuffer: None,
    RefreshInternalInfo: None,
    ChangeTitle: None,
    ChangeIcon: None,
    GetThreadConsoleDesktop: None,
    GetConsoleWindowHandle: None,
    GetLargestConsoleWindowSize: None,
    GetSelectionInfo: None,
    SetPalette: None,
    SetCodePage: None,
    GetDisplayMode: None,
    SetDisplayMode: None,
    ShowMouseCursor: None,
    SetMouseCursor: None,
    MenuControl: None,
    SetMenuClose: None,
};

pub(crate) unsafe fn InitTuiVtblIfNeeded() {
    if TuiVtbl.InitFrontEnd.is_some() {
        return;
    }
    TuiVtbl = FRONTEND_VTBL {
        InitFrontEnd: Some(core::mem::transmute(TuiInitFrontEnd as *const ())),
        DeinitFrontEnd: Some(core::mem::transmute(TuiDeinitFrontEnd as *const ())),
        DrawRegion: Some(core::mem::transmute(TuiDrawRegion as *const ())),
        WriteStream: Some(core::mem::transmute(TuiWriteStream as *const ())),
        RingBell: Some(core::mem::transmute(TuiRingBell as *const ())),
        SetCursorInfo: Some(core::mem::transmute(TuiSetCursorInfo as *const ())),
        SetScreenInfo: Some(core::mem::transmute(TuiSetScreenInfo as *const ())),
        ResizeTerminal: Some(core::mem::transmute(TuiResizeTerminal as *const ())),
        SetActiveScreenBuffer: Some(core::mem::transmute(TuiSetActiveScreenBuffer as *const ())),
        ReleaseScreenBuffer: Some(core::mem::transmute(TuiReleaseScreenBuffer as *const ())),
        RefreshInternalInfo: Some(core::mem::transmute(TuiRefreshInternalInfo as *const ())),
        ChangeTitle: Some(core::mem::transmute(TuiChangeTitle as *const ())),
        ChangeIcon: Some(core::mem::transmute(TuiChangeIcon as *const ())),
        GetThreadConsoleDesktop: Some(core::mem::transmute(TuiGetThreadConsoleDesktop as *const ())),
        GetConsoleWindowHandle: Some(core::mem::transmute(TuiGetConsoleWindowHandle as *const ())),
        GetLargestConsoleWindowSize: Some(core::mem::transmute(TuiGetLargestConsoleWindowSize as *const ())),
        GetSelectionInfo: Some(core::mem::transmute(TuiGetSelectionInfo as *const ())),
        SetPalette: Some(core::mem::transmute(TuiSetPalette as *const ())),
        SetCodePage: Some(core::mem::transmute(TuiSetCodePage as *const ())),
        GetDisplayMode: Some(core::mem::transmute(TuiGetDisplayMode as *const ())),
        SetDisplayMode: Some(core::mem::transmute(TuiSetDisplayMode as *const ())),
        ShowMouseCursor: Some(core::mem::transmute(TuiShowMouseCursor as *const ())),
        SetMouseCursor: Some(core::mem::transmute(TuiSetMouseCursor as *const ())),
        MenuControl: Some(core::mem::transmute(TuiMenuControl as *const ())),
        SetMenuClose: Some(core::mem::transmute(TuiSetMenuClose as *const ())),
    };
}

pub(crate) unsafe fn ScmLoadDriver(lpServiceName: LPCWSTR) -> u32 {
    let mut Status: i32 = STATUS_SUCCESS;
    let mut WasPrivilegeEnabled: u8 = 0u8;
    let pszDriverPath: PWSTR;
    let mut DriverPath: _UNICODE_STRING = core::mem::zeroed();

    pszDriverPath = ConsoleAllocHeap(
        HEAP_ZERO_MEMORY,
        (52 + wcslen(lpServiceName) + 1) * core::mem::size_of::<u16>(),
    ) as PWSTR;
    if pszDriverPath == core::ptr::null_mut() {
        return ERROR_NOT_ENOUGH_MEMORY;
    }

    wcscpy(
        pszDriverPath,
        w!("\\Registry\\Machine\\System\\CurrentControlSet\\Services\\").as_ptr() as *const u16,
    );
    wcscat(pszDriverPath, lpServiceName);

    RtlInitUnicodeString(&mut DriverPath, pszDriverPath);

    Status = RtlAdjustPrivilege(
        SE_LOAD_DRIVER_PRIVILEGE,
        1u8,
        0u8,
        &mut WasPrivilegeEnabled,
    );
    if !NT_SUCCESS(Status) {
        ConsoleFreeHeap(pszDriverPath as *mut core::ffi::c_void);
        return RtlNtStatusToDosError(Status);
    }

    Status = NtLoadDriver(&mut DriverPath);

    RtlAdjustPrivilege(
        SE_LOAD_DRIVER_PRIVILEGE,
        WasPrivilegeEnabled,
        0u8,
        &mut WasPrivilegeEnabled,
    );

    ConsoleFreeHeap(pszDriverPath as *mut core::ffi::c_void);
    RtlNtStatusToDosError(Status)
}

pub(crate) unsafe fn ScmUnloadDriver(lpServiceName: LPCWSTR) -> u32 {
    let mut Status: i32 = STATUS_SUCCESS;
    let mut WasPrivilegeEnabled: u8 = 0u8;
    let pszDriverPath: PWSTR;
    let mut DriverPath: _UNICODE_STRING = core::mem::zeroed();

    pszDriverPath = ConsoleAllocHeap(
        HEAP_ZERO_MEMORY,
        (52 + wcslen(lpServiceName) + 1) * core::mem::size_of::<u16>(),
    ) as PWSTR;
    if pszDriverPath.is_null() {
        return ERROR_NOT_ENOUGH_MEMORY;
    }

    wcscpy(
        pszDriverPath,
        b"\\Registry\\Machine\\System\\CurrentControlSet\\Services\\\0"
            .as_ptr()
            .cast::<u16>(),
    );
    wcscat(pszDriverPath, lpServiceName);

    RtlInitUnicodeString(&mut DriverPath, pszDriverPath);

    Status = RtlAdjustPrivilege(
        SE_LOAD_DRIVER_PRIVILEGE,
        1u8,
        0u8,
        &mut WasPrivilegeEnabled,
    );
    if !NT_SUCCESS(Status) {
        ConsoleFreeHeap(pszDriverPath as *mut core::ffi::c_void);
        return RtlNtStatusToDosError(Status);
    }

    Status = NtUnloadDriver(&mut DriverPath);

    RtlAdjustPrivilege(
        SE_LOAD_DRIVER_PRIVILEGE,
        WasPrivilegeEnabled,
        0u8,
        &mut WasPrivilegeEnabled,
    );

    ConsoleFreeHeap(pszDriverPath as *mut core::ffi::c_void);
    RtlNtStatusToDosError(Status)
}

pub(crate) unsafe fn TuiSwapConsole(Next: i32) -> u8 {
    static mut SwapConsole: *mut _TUI_CONSOLE_DATA = core::ptr::null_mut();
    let mut BytesReturned: u32 = 0;
    let mut Title: ANSI_STRING = core::mem::zeroed();
    let Buffer: *mut core::ffi::c_void;
    let pos: PCOORD;

    if 0 != Next {
        EnterCriticalSection(core::ptr::addr_of_mut!(ActiveVirtConsLock));
        if SwapConsole.is_null() {
            SwapConsole = ActiveConsole;
        }

        SwapConsole = if 0 < Next {
            GetNextConsole(SwapConsole)
        } else {
            GetPrevConsole(SwapConsole)
        };
        Title.MaximumLength = RtlUnicodeStringToAnsiSize(core::ptr::addr_of_mut!(
            (*(*SwapConsole).Console).Title
        )) as u16;
        Title.Length = 0;
        Buffer = ConsoleAllocHeap(
            0,
            core::mem::size_of::<COORD>() + Title.MaximumLength as usize,
        );
        pos = Buffer as PCOORD;
        Title.Buffer = (Buffer as usize + core::mem::size_of::<COORD>()) as *mut i8;

        RtlUnicodeStringToAnsiString(
            core::ptr::addr_of_mut!(Title),
            core::ptr::addr_of_mut!((*(*SwapConsole).Console).Title),
            0u8,
        );
        (*pos).X = (PhysicalConsoleSize.X - Title.Length as i16) / 2;
        (*pos).Y = PhysicalConsoleSize.Y / 2;
        ConioDrawConsole((*ActiveConsole).Console);
        if DeviceIoControl(
            ConsoleDeviceHandle,
            IOCTL_CONSOLE_WRITE_OUTPUT_CHARACTER,
            core::ptr::null_mut(),
            0,
            Buffer,
            core::mem::size_of::<COORD>() as u32 + Title.Length as u32,
            core::ptr::addr_of_mut!(BytesReturned),
            core::ptr::null_mut(),
        ) == 0
        {
        }
        ConsoleFreeHeap(Buffer);
        LeaveCriticalSection(core::ptr::addr_of_mut!(ActiveVirtConsLock));

        return 1u8;
    } else if !SwapConsole.is_null() {
        EnterCriticalSection(core::ptr::addr_of_mut!(ActiveVirtConsLock));
        if SwapConsole != ActiveConsole {
            (*(*SwapConsole).Entry.Blink).Flink = (*SwapConsole).Entry.Flink;
            (*(*SwapConsole).Entry.Flink).Blink = (*SwapConsole).Entry.Blink;
            (*SwapConsole).Entry.Flink = core::ptr::addr_of_mut!((*ActiveConsole).Entry);
            (*SwapConsole).Entry.Blink = (*ActiveConsole).Entry.Blink;
            (*(*ActiveConsole).Entry.Blink).Flink = core::ptr::addr_of_mut!((*SwapConsole).Entry);
            (*ActiveConsole).Entry.Blink = core::ptr::addr_of_mut!((*SwapConsole).Entry);
        }
        ActiveConsole = SwapConsole;
        SwapConsole = core::ptr::null_mut();
        ConioDrawConsole((*ActiveConsole).Console);
        LeaveCriticalSection(core::ptr::addr_of_mut!(ActiveVirtConsLock));
        return 1u8;
    } else {
        return 0u8;
    }
}

pub(crate) unsafe fn TuiCopyRect(mut Dest: PCHAR, Buff: PTEXTMODE_SCREEN_BUFFER, Region: *mut SMALL_RECT) {
    let SrcDelta: u32;
    let DestDelta: u32;
    let mut i: i32;
    let mut Src: PCHAR_INFO;
    let SrcEnd: PCHAR_INFO;

    Src = ConioCoordToPointer(Buff, (*Region).Left as u32, (*Region).Top as u32);
    SrcDelta = ((*Buff).ScreenBufferSize.X as u32) * core::mem::size_of::<CHAR_INFO>() as u32;
    SrcEnd = ((*Buff).Buffer as usize +
              ((*Buff).ScreenBufferSize.Y as usize) *
              ((*Buff).ScreenBufferSize.X as usize) *
              core::mem::size_of::<CHAR_INFO>()) as PCHAR_INFO;
    DestDelta = (ConioRectWidth(Region) as u32) * 2;

    i = (*Region).Top as i32;
    while i <= (*Region).Bottom as i32 {
        ConsoleOutputUnicodeToAnsiChar(
            (*Buff).Header.Console as PCONSRV_CONSOLE,
            Dest,
            core::ptr::addr_of_mut!((*Src).Char.UnicodeChar)
        );
        *((Dest as usize + 1) as PBYTE) = (*Src).Attributes as u8;

        Src = (Src as usize + SrcDelta as usize) as PCHAR_INFO;
        if SrcEnd as usize <= Src as usize {
            Src = (Src as usize - 
                   ((*Buff).ScreenBufferSize.Y as usize) * 
                   ((*Buff).ScreenBufferSize.X as usize) * 
                   core::mem::size_of::<CHAR_INFO>()) as PCHAR_INFO;
        }
        Dest = (Dest as usize + DestDelta as usize) as PCHAR;
        i += 1;
    }
}

pub(crate) unsafe extern "system" fn TuiConsoleWndProc(hWnd: HWND, msg: u32, wParam: WPARAM, lParam: LPARAM) -> LRESULT {
    match msg {
        WM_CHAR | WM_SYSCHAR | WM_KEYDOWN | WM_SYSKEYDOWN | WM_KEYUP | WM_SYSKEYUP => {
            if ConDrvValidateConsoleUnsafe(
                (*ActiveConsole).Console as PCONSOLE,
                CONSOLE_RUNNING,
                1u8,
            ) != 0
            {
                let mut Message: MSG = core::mem::zeroed();
                Message.hwnd = hWnd;
                Message.message = msg;
                Message.wParam = wParam;
                Message.lParam = lParam;

                ConioProcessKey((*ActiveConsole).Console, core::ptr::addr_of_mut!(Message));
                LeaveCriticalSection(core::ptr::addr_of_mut!((*(*ActiveConsole).Console).Lock));
            }
        }

        WM_ACTIVATE => {
            if ConDrvValidateConsoleUnsafe(
                (*ActiveConsole).Console as PCONSOLE,
                CONSOLE_RUNNING,
                1u8,
            ) != 0
            {
                if LOWORD(wParam as u32) as u32 != WA_INACTIVE {
                    SetFocus(hWnd);
                    ConioDrawConsole((*ActiveConsole).Console);
                }
                LeaveCriticalSection(core::ptr::addr_of_mut!((*(*ActiveConsole).Console).Lock));
            }
        }

        _ => {}
    }

    DefWindowProcW(hWnd, msg, wParam, lParam)
}

unsafe extern "system" fn TuiConsoleThread(Param: PVOID) -> DWORD {
    let TuiData = Param as PTUI_CONSOLE_DATA;
    let Console = (*TuiData).Console;
    let NewWindow: HWND;
    let mut msg: MSG = core::mem::zeroed();

    NewWindow = CreateWindowW(
        w!("TuiConsoleWindowClass").as_ptr() as *const u16,
        (*Console).Title.Buffer,
        0,
        -32000,
        -32000,
        0,
        0,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        ConSrvDllInstance,
        Console as PVOID,
    );
    if NewWindow == core::ptr::null_mut() {
        return 1;
    }
    (*TuiData).hWindow = NewWindow;

    SetForegroundWindow((*TuiData).hWindow);
    NtUserConsoleControl(ConsoleAcquireDisplayOwnership, core::ptr::null_mut(), 0);

    while GetMessageW(
        core::ptr::addr_of_mut!(msg),
        core::ptr::null_mut(),
        0,
        0,
    ) != 0
    {
        TranslateMessage(core::ptr::addr_of_mut!(msg));
        DispatchMessageW(core::ptr::addr_of_mut!(msg));
    }

    0
}

pub(crate) unsafe fn TuiSetConsoleOutputCP(
    hNtConddHandle: HANDLE,
    CodePage: UINT,
) -> BOOLEAN
{
    static mut LastLoadedCodepage: UINT = 0;
    let mut FontFile: _UNICODE_STRING = core::mem::zeroed();
    FontFile.Length = 48;
    FontFile.MaximumLength = 50;
    FontFile.Buffer = w!("\\SystemRoot\\vgafonts.cab").as_ptr() as *mut u16;
    let mut FontName: [CHAR; 20] = [0; 20];

    let mut Status: NTSTATUS;
    let mut FileHandle: HANDLE = core::ptr::null_mut();
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut FontBitField: PUCHAR = core::ptr::null_mut();

    let mut FileSectionHandle: HANDLE = core::ptr::null_mut();
    let mut FileBuffer: PUCHAR = core::ptr::null_mut();
    let mut FileSize: SIZE_T = 0;
    let CabFileHeader: PCFHEADER;
    let mut Data: _DataUnion = core::mem::zeroed();
    let mut FoundFile: PCFFILE = core::ptr::null_mut();
    let mut FileName: PSTR;
    let mut Index: USHORT;

    #[repr(C)]
    union _DataUnion {
        CabFile: PCFFILE,
        Buffer: PVOID,
    }

    if CodePage == LastLoadedCodepage {
        return 1u8;
    }

    ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as u32;
    ObjectAttributes.RootDirectory = core::ptr::null_mut();
    ObjectAttributes.ObjectName = core::ptr::addr_of_mut!(FontFile);
    ObjectAttributes.Attributes = OBJ_CASE_INSENSITIVE;
    ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
    ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

    Status = NtOpenFile(
        core::ptr::addr_of_mut!(FileHandle),
        GENERIC_READ | SYNCHRONIZE,
        core::ptr::addr_of_mut!(ObjectAttributes),
        core::ptr::addr_of_mut!(IoStatusBlock),
        FILE_SHARE_READ,
        FILE_SYNCHRONOUS_IO_NONALERT,
    );
    if !NT_SUCCESS(Status) {
        return 0u8;
    }

    Status = NtCreateSection(
        core::ptr::addr_of_mut!(FileSectionHandle),
        SECTION_ALL_ACCESS,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        PAGE_READONLY,
        SEC_COMMIT,
        FileHandle,
    );
    if !NT_SUCCESS(Status) {
        'Exit: {
            if !FileBuffer.is_null() {
                NtUnmapViewOfSection(NtCurrentProcess(), FileBuffer as PVOID);
            }

            if !FileSectionHandle.is_null() {
                NtClose(FileSectionHandle);
            }

            NtClose(FileHandle);

            if NT_SUCCESS(Status) {
                LastLoadedCodepage = CodePage;
            }

            return if NT_SUCCESS(Status) { 1u8 } else { 0u8 };
        }
    }

    Status = NtMapViewOfSection(
        FileSectionHandle,
        NtCurrentProcess(),
        core::ptr::addr_of_mut!(FileBuffer) as *mut PVOID,
        0,
        0,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(FileSize),
        ViewUnmap,
        0,
        PAGE_READONLY,
    );
    if !NT_SUCCESS(Status) {
        'Exit: {
            if !FileBuffer.is_null() {
                NtUnmapViewOfSection(NtCurrentProcess(), FileBuffer as PVOID);
            }

            if !FileSectionHandle.is_null() {
                NtClose(FileSectionHandle);
            }

            NtClose(FileHandle);

            if NT_SUCCESS(Status) {
                LastLoadedCodepage = CodePage;
            }

            return if NT_SUCCESS(Status) { 1u8 } else { 0u8 };
        }
    }

    {
        CabFileHeader = FileBuffer as PCFHEADER;

        if FileSize <= core::mem::size_of::<CFHEADER>() as SIZE_T ||
            (*CabFileHeader).Signature != CAB_SIGNATURE ||
            (*CabFileHeader).Version != CAB_VERSION ||
            (*CabFileHeader).FolderCount == 0 ||
            (*CabFileHeader).FileCount == 0 ||
            (*CabFileHeader).FileTableOffset < core::mem::size_of::<CFHEADER>() as u32 {
            Status = STATUS_UNSUCCESSFUL;
            'Exit: {
                if !FileBuffer.is_null() {
                    NtUnmapViewOfSection(NtCurrentProcess(), FileBuffer as PVOID);
                }

                if !FileSectionHandle.is_null() {
                    NtClose(FileSectionHandle);
                }

                NtClose(FileHandle);

                if NT_SUCCESS(Status) {
                    LastLoadedCodepage = CodePage;
                }

                return if NT_SUCCESS(Status) { 1u8 } else { 0u8 };
            }
        }

        RtlStringCbPrintfA(
            FontName.as_mut_ptr(),
            core::mem::size_of_val(&FontName),
            b"%u-8x8.bin\0".as_ptr() as PSTR,
            CodePage,
        );

        Data.CabFile = (FileBuffer.offset((*CabFileHeader).FileTableOffset as isize)) as PCFFILE;
        Index = 0;
        while Index < (*CabFileHeader).FileCount {
            FileName = (Data.CabFile.offset(1)) as PSTR;

            if FoundFile.is_null() {
                if _stricmp(FontName.as_ptr(), FileName) == 0 {
                    FoundFile = Data.CabFile;
                }
            }

            Data.CabFile = (strchr(FileName, 0).offset(1)) as PCFFILE;
            Index += 1;
        }

        if FoundFile.is_null() {
            Status = STATUS_OBJECT_NAME_NOT_FOUND;
            'Exit: {
                if !FileBuffer.is_null() {
                    NtUnmapViewOfSection(NtCurrentProcess(), FileBuffer as PVOID);
                }

                if !FileSectionHandle.is_null() {
                    NtClose(FileSectionHandle);
                }

                NtClose(FileHandle);

                if NT_SUCCESS(Status) {
                    LastLoadedCodepage = CodePage;
                }

                return if NT_SUCCESS(Status) { 1u8 } else { 0u8 };
            }
        }

        if (*FoundFile).FileSize != 256 * 8 {
            Status = STATUS_INVALID_BUFFER_SIZE;
            'Exit: {
                if !FileBuffer.is_null() {
                    NtUnmapViewOfSection(NtCurrentProcess(), FileBuffer as PVOID);
                }

                if !FileSectionHandle.is_null() {
                    NtClose(FileSectionHandle);
                }

                NtClose(FileHandle);

                if NT_SUCCESS(Status) {
                    LastLoadedCodepage = CodePage;
                }

                return if NT_SUCCESS(Status) { 1u8 } else { 0u8 };
            }
        }

        FontBitField = RtlAllocateHeap(RtlGetProcessHeap(), 0, (*FoundFile).FileSize as SIZE_T) as PUCHAR;
        if FontBitField.is_null() {
            Status = STATUS_NO_MEMORY;
            'Exit: {
                if !FileBuffer.is_null() {
                    NtUnmapViewOfSection(NtCurrentProcess(), FileBuffer as PVOID);
                }

                if !FileSectionHandle.is_null() {
                    NtClose(FileSectionHandle);
                }

                NtClose(FileHandle);

                if NT_SUCCESS(Status) {
                    LastLoadedCodepage = CodePage;
                }

                return if NT_SUCCESS(Status) { 1u8 } else { 0u8 };
            }
        }

        Data.Buffer = (Data.Buffer as ULONG_PTR + 8) as PVOID;
        Data.Buffer = (Data.Buffer as ULONG_PTR + (*FoundFile).FileOffset as ULONG_PTR) as PVOID;

        core::ptr::copy_nonoverlapping(Data.Buffer as *const u8, FontBitField, (*FoundFile).FileSize as usize);
        Status = STATUS_SUCCESS;
    }

    if NT_SUCCESS(Status) {
        Status = NtDeviceIoControlFile(
            hNtConddHandle,
            core::ptr::null_mut(),
            None,
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!(IoStatusBlock),
            IOCTL_CONSOLE_LOADFONT,
            FontBitField as PVOID,
            (*FoundFile).FileSize,
            core::ptr::null_mut(),
            0,
        );
    }

    if !FontBitField.is_null() {
        RtlFreeHeap(RtlGetProcessHeap(), 0, FontBitField as PVOID);
    }

    if !FileBuffer.is_null() {
        NtUnmapViewOfSection(NtCurrentProcess(), FileBuffer as PVOID);
    }

    if !FileSectionHandle.is_null() {
        NtClose(FileSectionHandle);
    }

    NtClose(FileHandle);

    if NT_SUCCESS(Status) {
        LastLoadedCodepage = CodePage;
    }

    if NT_SUCCESS(Status) { 1u8 } else { 0u8 }
}

pub(crate) unsafe fn TuiInit(OemCP: UINT) -> BOOL {
    let mut Success: BOOL;
    let mut ScrInfo: CONSOLE_SCREEN_BUFFER_INFO = core::mem::zeroed();
    let mut BytesReturned: u32 = 0;
    let mut wc: WNDCLASSEXW = core::mem::zeroed();
    let ConsoleClassAtom: ATOM;
    let TextAttribute: u16 = FOREGROUND_BLUE | FOREGROUND_GREEN | FOREGROUND_RED;

    const INVALID_HANDLE_VALUE: HANDLE = -1isize as HANDLE;
    const FILE_ALL_ACCESS: u32 = 0x1F01FF;
    const IOCTL_CONSOLE_RESET_SCREEN: u32 = 0x00020000;
    const IOCTL_CONSOLE_SET_TEXT_ATTRIBUTE: u32 = 0x00020004;
    const IOCTL_CONSOLE_GET_SCREEN_BUFFER_INFO: u32 = 0x00020008;

    if ConsInitialized != 0 {
        return 1;
    }

    ScmLoadDriver(w!("Blue").as_ptr() as *const u16);

    ConsoleDeviceHandle = CreateFileW(
        w!("\\\\.\\BlueScreen").as_ptr() as *const u16,
        FILE_ALL_ACCESS,
        0,
        core::ptr::null_mut(),
        OPEN_EXISTING,
        0,
        core::ptr::null_mut(),
    );

    if ConsoleDeviceHandle == INVALID_HANDLE_VALUE {
        return 0;
    }

    Success = 1;
    if DeviceIoControl(
        ConsoleDeviceHandle,
        IOCTL_CONSOLE_RESET_SCREEN,
        &Success as *const BOOL as *const core::ffi::c_void,
        core::mem::size_of::<BOOL>() as u32,
        core::ptr::null_mut(),
        0,
        &mut BytesReturned,
        core::ptr::null_mut(),
    ) == 0
    {
        CloseHandle(ConsoleDeviceHandle);
        return 0;
    }

    if TuiSetConsoleOutputCP(ConsoleDeviceHandle, OemCP) == 0 {
    }

    if DeviceIoControl(
        ConsoleDeviceHandle,
        IOCTL_CONSOLE_SET_TEXT_ATTRIBUTE,
        &TextAttribute as *const u16 as *const core::ffi::c_void,
        core::mem::size_of::<u16>() as u32,
        core::ptr::null_mut(),
        0,
        &mut BytesReturned,
        core::ptr::null_mut(),
    ) == 0
    {
    }

    ActiveConsole = core::ptr::null_mut();
    InitializeListHead(&raw mut VirtConsList);
    InitializeCriticalSection(&raw mut ActiveVirtConsLock);

    if DeviceIoControl(
        ConsoleDeviceHandle,
        IOCTL_CONSOLE_GET_SCREEN_BUFFER_INFO,
        core::ptr::null(),
        0,
        &mut ScrInfo as *mut CONSOLE_SCREEN_BUFFER_INFO as *mut core::ffi::c_void,
        core::mem::size_of::<CONSOLE_SCREEN_BUFFER_INFO>() as u32,
        &mut BytesReturned,
        core::ptr::null_mut(),
    ) == 0
    {
        Success = 0;
        'Quit: loop {
            if Success == 0 {
                DeleteCriticalSection(&raw mut ActiveVirtConsLock);
                CloseHandle(ConsoleDeviceHandle);
            }

            ConsInitialized = Success;
            return Success;
        }
    }
    PhysicalConsoleSize = ScrInfo.dwSize;

    core::ptr::write_bytes(&mut wc as *mut WNDCLASSEXW, 0, 1);
    wc.cbSize = core::mem::size_of::<WNDCLASSEXW>() as u32;
    wc.lpszClassName = w!("TuiConsoleWindowClass").as_ptr() as *const u16;
    wc.lpfnWndProc = Some(TuiConsoleWndProc);
    wc.cbWndExtra = 0;
    wc.hInstance = ConSrvDllInstance;

    ConsoleClassAtom = RegisterClassExW(&wc);
    if ConsoleClassAtom == 0 {
        Success = 0;
    } else {
        Success = 1;
    }

    if Success == 0 {
        DeleteCriticalSection(&raw mut ActiveVirtConsLock);
        CloseHandle(ConsoleDeviceHandle);
    }

    ConsInitialized = Success;
    return Success;
}

pub(crate) unsafe extern "system" fn TuiInitFrontEnd(
    This: PFRONTEND,
    Console: PCONSRV_CONSOLE,
) -> NTSTATUS {
    let TuiData: PTUI_CONSOLE_DATA;
    let ThreadHandle: HANDLE;

    if This.is_null() || Console.is_null() {
        return STATUS_INVALID_PARAMETER;
    }

    if GetType((*Console).ActiveBuffer) != TEXTMODE_BUFFER {
        return STATUS_INVALID_PARAMETER;
    }

    TuiData = ConsoleAllocHeap(
        HEAP_ZERO_MEMORY,
        core::mem::size_of::<TUI_CONSOLE_DATA>(),
    ) as PTUI_CONSOLE_DATA;
    if TuiData.is_null() {
        return STATUS_UNSUCCESSFUL;
    }

    (*TuiData).Console = Console;
    (*TuiData).ActiveBuffer = (*Console).ActiveBuffer;
    (*TuiData).hWindow = core::ptr::null_mut();

    InitializeCriticalSection(core::ptr::addr_of_mut!((*TuiData).Lock));

    ThreadHandle = CreateThread(
        core::ptr::null_mut(),
        0,
        Some(core::mem::transmute(TuiConsoleThread as *const ())),
        TuiData as PVOID,
        0,
        core::ptr::null_mut(),
    );
    if ThreadHandle.is_null() {
        TuiDeinitFrontEnd(This);
        return STATUS_UNSUCCESSFUL;
    }
    CloseHandle(ThreadHandle);

    EnterCriticalSection(core::ptr::addr_of_mut!(ActiveVirtConsLock));
    InsertTailList(
        core::ptr::addr_of_mut!(VirtConsList),
        core::ptr::addr_of_mut!((*TuiData).Entry),
    );
    ActiveConsole = TuiData;
    LeaveCriticalSection(core::ptr::addr_of_mut!(ActiveVirtConsLock));

    (*This).Context = TuiData as *mut core::ffi::c_void;
    (*This).Context2 = core::ptr::null_mut();

    STATUS_SUCCESS
}

pub(crate) unsafe extern "system" fn TuiDeinitFrontEnd(This: PFRONTEND) {
    let TuiData = (*This).Context as PTUI_CONSOLE_DATA;

    DestroyWindow((*TuiData).hWindow);

    EnterCriticalSection(core::ptr::addr_of_mut!(ActiveVirtConsLock));
    ActiveConsole = GetNextConsole(TuiData);
    RemoveEntryList(core::ptr::addr_of_mut!((*TuiData).Entry));

    LeaveCriticalSection(core::ptr::addr_of_mut!(ActiveVirtConsLock));

    if !ActiveConsole.is_null() {
        ConioDrawConsole((*ActiveConsole).Console);
    }

    (*This).Context = core::ptr::null_mut();
    DeleteCriticalSection(core::ptr::addr_of_mut!((*TuiData).Lock));
    ConsoleFreeHeap(TuiData as *mut core::ffi::c_void);
}

unsafe extern "system" fn TuiDrawRegion(This: PFRONTEND, Region: *mut SMALL_RECT) {
    let TuiData = (*This).Context as PTUI_CONSOLE_DATA;
    let Buff = (*(*TuiData).Console).ActiveBuffer;
    let ConsoleDraw: PCONSOLE_DRAW;
    let mut BytesReturned: u32 = 0;
    let ConsoleDrawSize: u32;

    if TuiData != ActiveConsole {
        return;
    }
    if GetType(Buff) != TEXTMODE_BUFFER {
        return;
    }

    ConsoleDrawSize = core::mem::size_of::<CONSOLE_DRAW>() as u32 +
                      ((ConioRectWidth(Region) as u32) * (ConioRectHeight(Region) as u32)) * 2;
    ConsoleDraw = ConsoleAllocHeap(0, ConsoleDrawSize as usize) as PCONSOLE_DRAW;
    if ConsoleDraw.is_null() {
        return;
    }
    (*ConsoleDraw).X = (*Region).Left as u16;
    (*ConsoleDraw).Y = (*Region).Top as u16;
    (*ConsoleDraw).SizeX = (ConioRectWidth(Region)) as u16;
    (*ConsoleDraw).SizeY = (ConioRectHeight(Region)) as u16;
    (*ConsoleDraw).CursorX = (*Buff).CursorPosition.X as u16;
    (*ConsoleDraw).CursorY = (*Buff).CursorPosition.Y as u16;

    TuiCopyRect(ConsoleDraw.offset(1) as *mut i8 , Buff as PTEXTMODE_SCREEN_BUFFER, Region);

    if DeviceIoControl(ConsoleDeviceHandle, IOCTL_CONSOLE_DRAW,
                       core::ptr::null_mut(), 0, ConsoleDraw as *mut core::ffi::c_void, ConsoleDrawSize, &mut BytesReturned, core::ptr::null_mut()) == 0 {
        ConsoleFreeHeap(ConsoleDraw as *mut core::ffi::c_void);
        return;
    }

    ConsoleFreeHeap(ConsoleDraw as *mut core::ffi::c_void);
}

pub(crate) unsafe extern "system" fn TuiWriteStream(
    This: PFRONTEND,
    _Region: *mut SMALL_RECT,
    _CursorStartX: i16,
    _CursorStartY: i16,
    _ScrolledLines: u32,
    Buffer: *mut u16,
    Length: u32,
) {
    let TuiData = (*This).Context as *mut TUI_CONSOLE_DATA;
    let Buff = (*(*TuiData).Console).ActiveBuffer;
    let NewBuffer: *mut i8;
    let NewLength: u32;
    let mut BytesWritten: u32 = 0;

    if TuiData != ActiveConsole {
        return;
    }
    if GetType(Buff) != TEXTMODE_BUFFER {
        return;
    }

    NewLength = WideCharToMultiByte(
        (*(*TuiData).Console).OutputCodePage,
        0,
        Buffer,
        Length as i32,
        core::ptr::null_mut(),
        0,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    ) as u32;
    NewBuffer = RtlAllocateHeap(
        RtlGetProcessHeap(),
        0,
        ((NewLength as usize) * core::mem::size_of::<i8>()) as SIZE_T,
    ) as *mut i8;
    if NewBuffer.is_null() {
        return;
    }

    WideCharToMultiByte(
        (*(*TuiData).Console).OutputCodePage,
        0,
        Buffer,
        Length as i32,
        NewBuffer,
        NewLength as i32,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );

    if WriteFile(
        ConsoleDeviceHandle,
        NewBuffer as *const core::ffi::c_void,
        ((NewLength as usize) * core::mem::size_of::<i8>()) as u32,
        &mut BytesWritten,
        core::ptr::null_mut(),
    ) == 0
    {
    }

    RtlFreeHeap(RtlGetProcessHeap(), 0, NewBuffer as *mut core::ffi::c_void);
}

pub(crate) unsafe extern "system" fn TuiRingBell(_This: PFRONTEND) {
    Beep(800, 200);
}

pub(crate) unsafe extern "system" fn TuiSetCursorInfo(
    This: PFRONTEND,
    Buff: PCONSOLE_SCREEN_BUFFER,
) -> BOOL {
    let TuiData = (*This).Context as PTUI_CONSOLE_DATA;
    let mut Info: CONSOLE_CURSOR_INFO = core::mem::zeroed();
    let mut BytesReturned: DWORD = 0;

    if TuiData != ActiveConsole {
        return 1;
    }
    if (*TuiData).Console as *mut _ as *mut core::ffi::c_void
        != (*Buff).Header.Console as *mut _ as *mut core::ffi::c_void
    {
        return 1;
    }
    if GetType(Buff) != TEXTMODE_BUFFER {
        return 0;
    }

    Info.dwSize = ConioEffectiveCursorSize((*TuiData).Console, 100);
    Info.bVisible = (*Buff).CursorInfo.bVisible;

    if DeviceIoControl(
        ConsoleDeviceHandle,
        IOCTL_CONSOLE_SET_CURSOR_INFO,
        core::ptr::addr_of!(Info) as *mut core::ffi::c_void,
        core::mem::size_of::<CONSOLE_CURSOR_INFO>() as DWORD,
        core::ptr::null_mut(),
        0,
        core::ptr::addr_of_mut!(BytesReturned),
        core::ptr::null_mut(),
    ) == 0
    {
        return 0;
    }

    1
}

pub(crate) unsafe extern "system" fn TuiSetScreenInfo(
    This: PFRONTEND,
    Buff: PCONSOLE_SCREEN_BUFFER,
    _OldCursorX: SHORT,
    _OldCursorY: SHORT,
) -> BOOL {
    let TuiData = (*This).Context as PTUI_CONSOLE_DATA;
    let mut Info: CONSOLE_SCREEN_BUFFER_INFO = core::mem::zeroed();
    let mut BytesReturned: DWORD = 0;

    if TuiData != ActiveConsole {
        return 1;
    }
    if (*(*TuiData).Console).ActiveBuffer != Buff {
        return 1;
    }
    if GetType(Buff) != TEXTMODE_BUFFER {
        return 0;
    }

    Info.dwCursorPosition = (*Buff).CursorPosition;
    Info.wAttributes = (*(Buff as PTEXTMODE_SCREEN_BUFFER)).ScreenDefaultAttrib;

    if DeviceIoControl(
        ConsoleDeviceHandle,
        IOCTL_CONSOLE_SET_SCREEN_BUFFER_INFO,
        core::ptr::addr_of!(Info) as LPVOID,
        core::mem::size_of::<CONSOLE_SCREEN_BUFFER_INFO>() as DWORD,
        core::ptr::null_mut(),
        0,
        core::ptr::addr_of_mut!(BytesReturned),
        core::ptr::null_mut(),
    ) == 0
    {
        return 0;
    }

    1
}

pub(crate) unsafe extern "system" fn TuiResizeTerminal(_This: PFRONTEND) {
}

unsafe extern "system" fn TuiSetActiveScreenBuffer(_This: PFRONTEND) {
}

unsafe extern "system" fn TuiReleaseScreenBuffer(
    _This: PFRONTEND,
    _ScreenBuffer: PCONSOLE_SCREEN_BUFFER,
) {
}

unsafe extern "system" fn TuiRefreshInternalInfo(_This: PFRONTEND) {
}

pub(crate) unsafe extern "system" fn TuiChangeTitle(_This: PFRONTEND) {
}

pub(crate) unsafe extern "system" fn TuiChangeIcon(
    _This: PFRONTEND,
    _IconHandle: HICON,
) -> BOOL {
    1
}

unsafe extern "system" fn TuiGetThreadConsoleDesktop(_This: PFRONTEND) -> HDESK {
    core::ptr::null_mut()
}

pub(crate) unsafe extern "system" fn TuiGetConsoleWindowHandle(This: PFRONTEND) -> HWND {
    let TuiData = (*This).Context as *mut _TUI_CONSOLE_DATA;
    (*TuiData).hWindow
}

pub(crate) unsafe extern "system" fn TuiGetLargestConsoleWindowSize(
    _This: PFRONTEND,
    pSize: PCOORD,
) {
    if pSize.is_null() {
        return;
    }
    *pSize = PhysicalConsoleSize;
}

pub(crate) unsafe extern "system" fn TuiGetSelectionInfo(
    _This: PFRONTEND,
    _pSelectionInfo: PCONSOLE_SELECTION_INFO,
) -> BOOL {
    1
}

pub(crate) unsafe extern "system" fn TuiSetPalette(
    _This: PFRONTEND,
    _PaletteHandle: HPALETTE,
    _PaletteUsage: u32,
) -> u8 {
    1u8
}

unsafe extern "system" fn TuiSetCodePage(
    _This: PFRONTEND,
    CodePage: UINT,
) -> BOOL {
    if TuiSetConsoleOutputCP(ConsoleDeviceHandle, CodePage) == 0 {
        return 0;
    }

    return 1;
}

unsafe extern "system" fn TuiGetDisplayMode(_This: PFRONTEND) -> ULONG {
    CONSOLE_FULLSCREEN_HARDWARE
}

pub(crate) unsafe extern "system" fn TuiSetDisplayMode(
    _This: PFRONTEND,
    _NewMode: ULONG,
) -> BOOL {
    1
}

pub(crate) unsafe extern "system" fn TuiShowMouseCursor(
    _This: PFRONTEND,
    _Show: BOOL,
) -> INT {
    return 0;
}

pub(crate) unsafe extern "system" fn TuiSetMouseCursor(
    _This: PFRONTEND,
    _CursorHandle: HCURSOR,
) -> BOOL {
    1
}

unsafe extern "system" fn TuiMenuControl(
    _This: PFRONTEND,
    _CmdIdLow: UINT,
    _CmdIdHigh: UINT,
) -> HMENU {
    core::ptr::null_mut()
}

pub(crate) unsafe extern "system" fn TuiSetMenuClose(
    _This: PFRONTEND,
    _Enable: BOOL,
) -> BOOL {
    1
}

pub(crate) unsafe fn IsConsoleMode() -> BOOLEAN {
    NtUserCallNoParam(NOPARAM_ROUTINE_ISCONSOLEMODE) as BOOLEAN
}

#[no_mangle]
pub unsafe extern "system" fn TuiLoadFrontEnd(
    FrontEnd: PFRONTEND,
    ConsoleInfo: PCONSOLE_STATE_INFO,
    _ConsoleInitInfo: PCONSOLE_INIT_INFO,
    _ConsoleLeaderProcessHandle: HANDLE,
) -> NTSTATUS {
    if FrontEnd.is_null() || ConsoleInfo.is_null() {
        return STATUS_INVALID_PARAMETER;
    }

    if IsConsoleMode() == 0 {
        return STATUS_UNSUCCESSFUL;
    }

    if TuiInit((*ConsoleInfo).CodePage) == 0 {
        return STATUS_UNSUCCESSFUL;
    }

    InitTuiVtblIfNeeded();
    (*FrontEnd).Vtbl = core::ptr::addr_of_mut!(TuiVtbl);
    (*FrontEnd).Context = core::ptr::null_mut();
    (*FrontEnd).Context2 = core::ptr::null_mut();

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn TuiUnloadFrontEnd(FrontEnd: PFRONTEND) -> NTSTATUS {
    if FrontEnd.is_null() {
        return STATUS_INVALID_PARAMETER;
    }
    if !(*FrontEnd).Context.is_null() {
        TuiDeinitFrontEnd(FrontEnd);
    }

    STATUS_SUCCESS
}
