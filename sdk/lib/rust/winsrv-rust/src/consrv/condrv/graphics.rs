use winsrv_sys::nt::*;
use winsrv_sys::winsrv::*;
use winsrv_sys::consts::*;
use crate::consrv::{ConsoleAllocHeap, ConsoleFreeHeap};



// Plain-cdecl (no NTAPI) in the still-C condrv (text.c) → extern "C".
extern "C" {
    pub fn CONSOLE_SCREEN_BUFFER_Initialize( Buffer: *mut PCONSOLE_SCREEN_BUFFER, Console: PCONSOLE, Type: CONSOLE_IO_OBJECT_TYPE, Size: usize, ) -> NTSTATUS;
    pub fn CONSOLE_SCREEN_BUFFER_Destroy(Buffer: PCONSOLE_SCREEN_BUFFER);
}

// condrv flip: defined plain-cdecl (NTSTATUS, no NTAPI) in C graphics.c;
// conoutput.rs declares it extern "C" → match (was wrongly extern "system").
#[no_mangle]
pub unsafe extern "C" fn GRAPHICS_BUFFER_Initialize(
    Buffer: *mut PCONSOLE_SCREEN_BUFFER,
    Console: PCONSOLE,
    ProcessHandle: HANDLE,
    GraphicsInfo: PGRAPHICS_BUFFER_INFO,
) -> NTSTATUS {
    let mut Status: NTSTATUS = STATUS_SUCCESS;
    let mut NewBuffer: PGRAPHICS_SCREEN_BUFFER = core::ptr::null_mut();

    let mut SectionSize: LARGE_INTEGER = core::mem::zeroed();
    let mut ViewSize: SIZE_T = 0;

    if Buffer.is_null() || Console.is_null() || GraphicsInfo.is_null() {
        return STATUS_INVALID_PARAMETER;
    }

    *Buffer = core::ptr::null_mut();

    Status = CONSOLE_SCREEN_BUFFER_Initialize(
        &mut NewBuffer as *mut PGRAPHICS_SCREEN_BUFFER as *mut PCONSOLE_SCREEN_BUFFER,
        Console,
        GRAPHICS_BUFFER,
        core::mem::size_of::<GRAPHICS_SCREEN_BUFFER>(),
    );
    if Status < 0 {
        return Status;
    }

    (*NewBuffer).ClientProcess = ProcessHandle;

    (*NewBuffer).BitMapInfoLength = (*GraphicsInfo).Info.dwBitMapInfoLength;

    (*NewBuffer).BitMapInfo = ConsoleAllocHeap(HEAP_ZERO_MEMORY, (*NewBuffer).BitMapInfoLength as usize) as *mut BITMAPINFO;
    if (*NewBuffer).BitMapInfo.is_null() {
        CONSOLE_SCREEN_BUFFER_Destroy(NewBuffer as PCONSOLE_SCREEN_BUFFER);
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    if (*(*GraphicsInfo).Info.lpBitMapInfo).bmiHeader.biHeight > 0 {
        (*(*GraphicsInfo).Info.lpBitMapInfo).bmiHeader.biHeight = -(*(*GraphicsInfo).Info.lpBitMapInfo).bmiHeader.biHeight;
    }

    if (*(*GraphicsInfo).Info.lpBitMapInfo).bmiHeader.biCompression != BI_RGB {
        (*(*GraphicsInfo).Info.lpBitMapInfo).bmiHeader.biCompression = BI_RGB;
    }

    RtlCopyMemory(
        (*NewBuffer).BitMapInfo as PVOID,
        (*GraphicsInfo).Info.lpBitMapInfo as PVOID,
        (*GraphicsInfo).Info.dwBitMapInfoLength as usize,
    );

    (*NewBuffer).BitMapUsage = (*GraphicsInfo).Info.dwUsage;

    if (*(*GraphicsInfo).Info.lpBitMapInfo).bmiHeader.biWidth <= 0xFFFF
        && -(*(*GraphicsInfo).Info.lpBitMapInfo).bmiHeader.biHeight <= 0xFFFF
    {
        (*NewBuffer).ScreenBufferSize.X = (*(*GraphicsInfo).Info.lpBitMapInfo).bmiHeader.biWidth as i16;
        (*NewBuffer).ScreenBufferSize.Y = (-(*(*GraphicsInfo).Info.lpBitMapInfo).bmiHeader.biHeight) as i16;

        (*NewBuffer).OldViewSize = (*NewBuffer).ViewSize;
        (*NewBuffer).ViewSize = (*NewBuffer).ScreenBufferSize;
        (*NewBuffer).OldScreenBufferSize = (*NewBuffer).ScreenBufferSize;
    } else {
        Status = STATUS_INSUFFICIENT_RESOURCES;
        ConsoleFreeHeap((*NewBuffer).BitMapInfo as PVOID);
        CONSOLE_SCREEN_BUFFER_Destroy(NewBuffer as PCONSOLE_SCREEN_BUFFER);
        return Status;
    }

    Status = NtCreateMutant(
        &mut (*NewBuffer).Mutex,
        MUTANT_ALL_ACCESS,
        core::ptr::null_mut(),
        0u8,
    );
    if Status < 0 {
        ConsoleFreeHeap((*NewBuffer).BitMapInfo as PVOID);
        CONSOLE_SCREEN_BUFFER_Destroy(NewBuffer as PCONSOLE_SCREEN_BUFFER);
        return Status;
    }

    Status = NtDuplicateObject(
        NtCurrentProcess(),
        (*NewBuffer).Mutex,
        ProcessHandle,
        &mut (*NewBuffer).ClientMutex,
        0,
        0,
        DUPLICATE_SAME_ACCESS,
    );
    if Status < 0 {
        NtClose((*NewBuffer).Mutex);
        ConsoleFreeHeap((*NewBuffer).BitMapInfo as PVOID);
        CONSOLE_SCREEN_BUFFER_Destroy(NewBuffer as PCONSOLE_SCREEN_BUFFER);
        return Status;
    }

    SectionSize.QuadPart = (*(*NewBuffer).BitMapInfo).bmiHeader.biSizeImage as i64;
    Status = NtCreateSection(
        &mut (*NewBuffer).hSection,
        SECTION_ALL_ACCESS,
        core::ptr::null_mut(),
        &mut SectionSize,
        PAGE_READWRITE,
        SEC_COMMIT,
        core::ptr::null_mut(),
    );
    if Status < 0 {
        NtDuplicateObject(
            ProcessHandle,
            (*NewBuffer).ClientMutex,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            0,
            0,
            DUPLICATE_CLOSE_SOURCE,
        );
        NtClose((*NewBuffer).Mutex);
        ConsoleFreeHeap((*NewBuffer).BitMapInfo as PVOID);
        CONSOLE_SCREEN_BUFFER_Destroy(NewBuffer as PCONSOLE_SCREEN_BUFFER);
        return Status;
    }

    ViewSize = 0;
    (*NewBuffer).BitMap = core::ptr::null_mut();
    Status = NtMapViewOfSection(
        (*NewBuffer).hSection,
        NtCurrentProcess(),
        &mut (*NewBuffer).BitMap as *mut PVOID,
        0,
        0,
        core::ptr::null_mut(),
        &mut ViewSize,
        ViewUnmap,
        0,
        PAGE_READWRITE,
    );
    if Status < 0 {
        NtClose((*NewBuffer).hSection);
        NtDuplicateObject(
            ProcessHandle,
            (*NewBuffer).ClientMutex,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            0,
            0,
            DUPLICATE_CLOSE_SOURCE,
        );
        NtClose((*NewBuffer).Mutex);
        ConsoleFreeHeap((*NewBuffer).BitMapInfo as PVOID);
        CONSOLE_SCREEN_BUFFER_Destroy(NewBuffer as PCONSOLE_SCREEN_BUFFER);
        return Status;
    }

    ViewSize = 0;
    (*NewBuffer).ClientBitMap = core::ptr::null_mut();
    Status = NtMapViewOfSection(
        (*NewBuffer).hSection,
        ProcessHandle,
        &mut (*NewBuffer).ClientBitMap as *mut PVOID,
        0,
        0,
        core::ptr::null_mut(),
        &mut ViewSize,
        ViewUnmap,
        0,
        PAGE_READWRITE,
    );
    if Status < 0 {
        NtUnmapViewOfSection(NtCurrentProcess(), (*NewBuffer).BitMap);
        NtClose((*NewBuffer).hSection);
        NtDuplicateObject(
            ProcessHandle,
            (*NewBuffer).ClientMutex,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            0,
            0,
            DUPLICATE_CLOSE_SOURCE,
        );
        NtClose((*NewBuffer).Mutex);
        ConsoleFreeHeap((*NewBuffer).BitMapInfo as PVOID);
        CONSOLE_SCREEN_BUFFER_Destroy(NewBuffer as PCONSOLE_SCREEN_BUFFER);
        return Status;
    }

    (*NewBuffer).ViewOrigin.X = 0;
    (*NewBuffer).ViewOrigin.Y = 0;

    (*NewBuffer).CursorBlinkOn = 0u8;
    (*NewBuffer).ForceCursorOff = 1u8;
    (*NewBuffer).CursorInfo.bVisible = 0i32;
    (*NewBuffer).CursorInfo.dwSize = 0;
    (*NewBuffer).CursorPosition.X = 0;
    (*NewBuffer).CursorPosition.Y = 0;

    (*NewBuffer).Mode = 0;

    *Buffer = NewBuffer as PCONSOLE_SCREEN_BUFFER;
    Status = STATUS_SUCCESS;

    Status
}

#[no_mangle]
pub unsafe extern "C" fn GRAPHICS_BUFFER_Destroy(Buffer: PCONSOLE_SCREEN_BUFFER) {
    let Buff = Buffer as PGRAPHICS_SCREEN_BUFFER;

    (*Buffer).Header.Type = SCREEN_BUFFER;

    NtUnmapViewOfSection((*Buff).ClientProcess, (*Buff).ClientBitMap);
    NtUnmapViewOfSection(NtCurrentProcess(), (*Buff).BitMap);
    NtClose((*Buff).hSection);
    NtDuplicateObject(
        (*Buff).ClientProcess,
        (*Buff).ClientMutex,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        0,
        0,
        DUPLICATE_CLOSE_SOURCE,
    );
    NtClose((*Buff).Mutex);
    ConsoleFreeHeap((*Buff).BitMapInfo as PVOID);

    CONSOLE_SCREEN_BUFFER_Destroy(Buffer);
}