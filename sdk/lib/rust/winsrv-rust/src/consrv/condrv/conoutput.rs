use winsrv_sys::nt::*;
use winsrv_sys::winsrv::*;

extern "system" {
    fn CONTAINING_RECORD(addr: *const u8, Type: usize, field_offset: usize) -> *mut u8;
}

extern "system" {
    fn TEXTMODE_BUFFER_Initialize( Buffer: *mut PCONSOLE_SCREEN_BUFFER, Console: PCONSOLE, ProcessHandle: HANDLE, TextModeInfo: PTEXTMODE_BUFFER_INFO, ) -> NTSTATUS;
    fn GRAPHICS_BUFFER_Initialize( Buffer: *mut PCONSOLE_SCREEN_BUFFER, Console: PCONSOLE, ProcessHandle: HANDLE, GraphicsInfo: PGRAPHICS_BUFFER_INFO, ) -> NTSTATUS;
    fn TEXTMODE_BUFFER_Destroy(Buffer: PCONSOLE_SCREEN_BUFFER);
    fn GRAPHICS_BUFFER_Destroy(Buffer: PCONSOLE_SCREEN_BUFFER);
    fn ConsoleAllocHeap(flags: ULONG, size: usize) -> PVOID;
    fn ConsoleFreeHeap(ptr: PVOID);
    fn TermReleaseScreenBuffer(Console: PCONSOLE, Buffer: PCONSOLE_SCREEN_BUFFER);
    fn TermSetActiveScreenBuffer(Console: PCONSOLE);
    fn TermDrawRegion(Console: PCONSOLE, Region: PSMALL_RECT);
    fn TermSetPalette(Console: PCONSOLE, PaletteHandle: HPALETTE, PaletteUsage: UINT) -> BOOL;
    fn TermSetCursorInfo(Console: PCONSOLE, Buffer: PCONSOLE_SCREEN_BUFFER) -> BOOL;
    fn TermSetScreenInfo(Console: PCONSOLE, Buffer: PCONSOLE_SCREEN_BUFFER, OldCursorX: SHORT, OldCursorY: SHORT) -> BOOL;
}

unsafe fn CONSOLE_SCREEN_BUFFER_Initialize(
    Buffer: *mut PCONSOLE_SCREEN_BUFFER,
    Console: PCONSOLE,
    Type: CONSOLE_IO_OBJECT_TYPE,
    Size: usize,
) -> NTSTATUS {
    if Buffer.is_null() || Console.is_null() {
        return STATUS_INVALID_PARAMETER;
    }

    let alloc_size = if Size > core::mem::size_of::<CONSOLE_SCREEN_BUFFER>() {
        Size
    } else {
        core::mem::size_of::<CONSOLE_SCREEN_BUFFER>()
    };

    *Buffer = ConsoleAllocHeap(HEAP_ZERO_MEMORY, alloc_size) as PCONSOLE_SCREEN_BUFFER;
    if (*Buffer).is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    ConSrvInitObject(
        core::ptr::addr_of_mut!((**Buffer).Header),
        Type,
        Console,
    );
    STATUS_SUCCESS
}

unsafe fn CONSOLE_SCREEN_BUFFER_Destroy(Buffer: PCONSOLE_SCREEN_BUFFER) {
    match (*Buffer).Header.Type {
        TEXTMODE_BUFFER => {
            TEXTMODE_BUFFER_Destroy(Buffer);
        }
        GRAPHICS_BUFFER => {
            GRAPHICS_BUFFER_Destroy(Buffer);
        }
        SCREEN_BUFFER => {
            if !(*Buffer).PaletteHandle.is_null() {
                DeleteObject((*Buffer).PaletteHandle);
            }
            ConsoleFreeHeap(Buffer as PVOID);
        }
        _ => {}
    }
}

#[no_mangle]
pub unsafe extern "system" fn ConDrvCreateScreenBuffer(
    Buffer: *mut PCONSOLE_SCREEN_BUFFER,
    Console: PCONSOLE,
    ProcessHandle: HANDLE,
    BufferType: ULONG,
    ScreenBufferInfo: PVOID,
) -> NTSTATUS {
    let mut Status = STATUS_UNSUCCESSFUL;

    if Console.is_null()
        || Buffer.is_null()
        || (BufferType != CONSOLE_TEXTMODE_BUFFER && BufferType != CONSOLE_GRAPHICS_BUFFER)
    {
        return STATUS_INVALID_PARAMETER;
    }

    let ProcessHandle = if ProcessHandle.is_null() {
        NtCurrentProcess()
    } else {
        ProcessHandle
    };

    match BufferType {
        CONSOLE_TEXTMODE_BUFFER => {
            Status = TEXTMODE_BUFFER_Initialize(
                Buffer,
                Console,
                ProcessHandle,
                ScreenBufferInfo as PTEXTMODE_BUFFER_INFO,
            );
        }
        CONSOLE_GRAPHICS_BUFFER => {
            Status = GRAPHICS_BUFFER_Initialize(
                Buffer,
                Console,
                ProcessHandle,
                ScreenBufferInfo as PGRAPHICS_BUFFER_INFO,
            );
        }
        _ => {
            debug_assert!(false);
        }
    }

    if NT_SUCCESS(Status) {
        InsertHeadList(
            core::ptr::addr_of_mut!((*Console).BufferList) as PLIST_ENTRY,
            core::ptr::addr_of_mut!((**Buffer).ListEntry) as PLIST_ENTRY,
        );
    }

    Status
}

unsafe fn ConioSetActiveScreenBuffer(Buffer: PCONSOLE_SCREEN_BUFFER) {
    let Console = (*Buffer).Header.Console;
    (*Console).ActiveBuffer = Buffer;
    TermSetActiveScreenBuffer(Console);
}

#[no_mangle]
pub unsafe extern "system" fn ConDrvDeleteScreenBuffer(Buffer: PCONSOLE_SCREEN_BUFFER) {
    let Console = (*Buffer).Header.Console;

    TermReleaseScreenBuffer(Console, Buffer);

    RemoveEntryList(core::ptr::addr_of_mut!((*Buffer).ListEntry) as PLIST_ENTRY);

    if Buffer == (*Console).ActiveBuffer {
        if !IsListEmpty(core::ptr::addr_of_mut!((*Console).BufferList) as PLIST_ENTRY) {
            let NewBuffer = (((*Console).BufferList.Flink as usize)
                - core::mem::offset_of!(CONSOLE_SCREEN_BUFFER, ListEntry))
                as PCONSOLE_SCREEN_BUFFER;

            ConioSetActiveScreenBuffer(NewBuffer);
        } else {
            (*Console).ActiveBuffer = core::ptr::null_mut();
        }
    }

    CONSOLE_SCREEN_BUFFER_Destroy(Buffer);
}

#[no_mangle]
pub unsafe extern "system" fn ConDrvSetConsoleActiveScreenBuffer(
    Console: PCONSOLE,
    Buffer: PCONSOLE_SCREEN_BUFFER,
) -> NTSTATUS {
    if Console.is_null() || Buffer.is_null() {
        return STATUS_INVALID_PARAMETER;
    }

    debug_assert!(Console == (*Buffer).Header.Console);

    if Buffer == (*Console).ActiveBuffer {
        return STATUS_SUCCESS;
    }

    if (*(*Console).ActiveBuffer).Header.ReferenceCount == 0 {
        ConDrvDeleteScreenBuffer((*Console).ActiveBuffer);
    }

    ConioSetActiveScreenBuffer(Buffer);

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn ConDrvGetActiveScreenBuffer(Console: PCONSOLE) -> PCONSOLE_SCREEN_BUFFER {
    if !Console.is_null() {
        (*Console).ActiveBuffer
    } else {
        core::ptr::null_mut()
    }
}

#[no_mangle]
pub unsafe extern "system" fn ConDrvInvalidateBitMapRect(
    Console: PCONSOLE,
    Buffer: PCONSOLE_SCREEN_BUFFER,
    Region: PSMALL_RECT,
) -> NTSTATUS {
    if Console.is_null() || Buffer.is_null() || Region.is_null() {
        return STATUS_INVALID_PARAMETER;
    }

    debug_assert!(Console == (*Buffer).Header.Console);

    if Buffer == (*Console).ActiveBuffer {
        TermDrawRegion(Console, Region);
    }

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn ConDrvSetConsolePalette(
    Console: PCONSOLE,
    Buffer: PCONSOLE_SCREEN_BUFFER,
    PaletteHandle: HPALETTE,
    PaletteUsage: UINT,
) -> NTSTATUS {
    if Console.is_null() || Buffer.is_null() {
        return STATUS_INVALID_PARAMETER;
    }

    if PaletteUsage != SYSPAL_STATIC
        && PaletteUsage != SYSPAL_NOSTATIC
        && PaletteUsage != SYSPAL_NOSTATIC256
    {
        return STATUS_INVALID_PARAMETER;
    }

    debug_assert!(Console == (*Buffer).Header.Console);

    let Success = TermSetPalette(Console, PaletteHandle, PaletteUsage);
    if Success != 0 {
        if !(*Buffer).PaletteHandle.is_null() && (*Buffer).PaletteHandle != PaletteHandle {
            DeleteObject((*Buffer).PaletteHandle);
        }

        (*Buffer).PaletteHandle = PaletteHandle;
        (*Buffer).PaletteUsage = PaletteUsage;
    }

    if Success != 0 {
        STATUS_SUCCESS
    } else {
        STATUS_UNSUCCESSFUL
    }
}

#[no_mangle]
pub unsafe extern "system" fn ConDrvGetConsoleCursorInfo(
    Console: PCONSOLE,
    Buffer: PTEXTMODE_SCREEN_BUFFER,
    CursorInfo: PCONSOLE_CURSOR_INFO,
) -> NTSTATUS {
    if Console.is_null() || Buffer.is_null() || CursorInfo.is_null() {
        return STATUS_INVALID_PARAMETER;
    }

    debug_assert!(Console == (*Buffer).Header.Console);

    *CursorInfo = (*Buffer).CursorInfo;

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn ConDrvSetConsoleCursorInfo(
    Console: PCONSOLE,
    Buffer: PTEXTMODE_SCREEN_BUFFER,
    CursorInfo: PCONSOLE_CURSOR_INFO,
) -> NTSTATUS {
    if Console.is_null() || Buffer.is_null() || CursorInfo.is_null() {
        return STATUS_INVALID_PARAMETER;
    }

    debug_assert!(Console == (*Buffer).Header.Console);

    let Size = (*CursorInfo).dwSize.max(1).min(100);
    let Visible = (*CursorInfo).bVisible;

    let mut Success = 1u8;

    if (Size != (*Buffer).CursorInfo.dwSize)
        || (Visible != 0 && (*Buffer).CursorInfo.bVisible == 0)
        || (Visible == 0 && (*Buffer).CursorInfo.bVisible != 0)
    {
        (*Buffer).CursorInfo.dwSize = Size;
        (*Buffer).CursorInfo.bVisible = Visible;

        Success = TermSetCursorInfo(Console, Buffer as PCONSOLE_SCREEN_BUFFER) as u8;
    }

    if Success != 0 {
        STATUS_SUCCESS
    } else {
        STATUS_UNSUCCESSFUL
    }
}

#[no_mangle]
pub unsafe extern "system" fn ConDrvSetConsoleCursorPosition(
    Console: PCONSOLE,
    Buffer: PTEXTMODE_SCREEN_BUFFER,
    Position: PCOORD,
) -> NTSTATUS {
    if Console.is_null() || Buffer.is_null() || Position.is_null() {
        return STATUS_INVALID_PARAMETER;
    }

    debug_assert!(Console == (*Buffer).Header.Console);

    if (*Position).X < 0
        || (*Position).X >= (*Buffer).ScreenBufferSize.X
        || (*Position).Y < 0
        || (*Position).Y >= (*Buffer).ScreenBufferSize.Y
    {
        return STATUS_INVALID_PARAMETER;
    }

    let OldCursorX = (*Buffer).CursorPosition.X;
    let OldCursorY = (*Buffer).CursorPosition.Y;
    (*Buffer).CursorPosition = *Position;

    if (Buffer as PCONSOLE_SCREEN_BUFFER) == (*Console).ActiveBuffer
        && TermSetScreenInfo(Console, Buffer as PCONSOLE_SCREEN_BUFFER, OldCursorX, OldCursorY) == 0
    {
        return STATUS_UNSUCCESSFUL;
    }

    STATUS_SUCCESS
}