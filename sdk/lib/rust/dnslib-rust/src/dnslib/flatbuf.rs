use ntdll_sys::nt::*;
use core::ffi::c_void;
use crate::dnslib::types::*;

extern "C" {
    fn strlen(s: *const i8) -> usize;
    fn wcslen(s: *const u16) -> usize;
}

#[no_mangle]
pub unsafe extern "system" fn FlatBuf_Init(
    FlatBuffer: *mut FLATBUFF,
    Buffer: *mut c_void,
    Size: usize,
) {
    (*FlatBuffer).Buffer = Buffer;
    (*FlatBuffer).BufferPos = Buffer as usize;
    (*FlatBuffer).BufferEnd = ((*FlatBuffer).BufferPos + Size) as *mut c_void;
    (*FlatBuffer).BufferFreeSize = Size;
    (*FlatBuffer).BufferSize = Size;
}

#[no_mangle]
pub unsafe extern "system" fn FlatBuf_Arg_Reserve(
    Position: *mut usize,
    FreeSize: *mut usize,
    Size: usize,
    Align: usize,
) -> *mut c_void {
    let mut OldPosition: usize = *Position;
    let mut NewFreeSize: usize = *FreeSize;

    if Align != 0 {
        OldPosition = OldPosition.wrapping_add(
            (Align.wrapping_sub(1)) & (!Align)
        );
    }

    let NewPosition: usize = OldPosition.wrapping_add(Size);

    NewFreeSize = NewFreeSize.wrapping_add(OldPosition.wrapping_sub(NewPosition));

    core::ptr::write_volatile(Position, NewPosition);
    core::ptr::write_volatile(FreeSize, NewFreeSize);

    if NewFreeSize > 0 {
        OldPosition as *mut c_void
    } else {
        core::ptr::null_mut()
    }
}

#[no_mangle]
pub unsafe extern "system" fn FlatBuf_Arg_CopyMemory(
    Position: *mut usize,
    FreeSize: *mut usize,
    Buffer: *const c_void,
    Size: usize,
    Align: usize,
) -> *mut c_void {
    let Destination = FlatBuf_Arg_Reserve(Position, FreeSize, Size, Align);
    if !Destination.is_null() {
        core::ptr::copy_nonoverlapping(Buffer as *const u8, Destination as *mut u8, Size);
    }
    Destination
}

#[no_mangle]
pub unsafe extern "system" fn FlatBuf_Arg_WriteString(
    Position: *mut usize,
    FreeSize: *mut usize,
    String: *mut c_void,
    IsUnicode: u8,
) -> *mut c_void {
    let StringLength: usize;
    let Align: usize;

    if IsUnicode != 0 {
        StringLength = (wcslen(String as *const u16) + 1) * core::mem::size_of::<u16>();
        Align = core::mem::size_of::<u16>();
    } else {
        StringLength = strlen(String as *const i8) + 1;
        Align = core::mem::size_of::<u8>();
    }

    let Destination = FlatBuf_Arg_Reserve(Position, FreeSize, StringLength, Align);
    if !Destination.is_null() {
        core::ptr::copy_nonoverlapping(
            String as *const u8,
            Destination as *mut u8,
            StringLength,
        );
    }
    Destination
}
