use ntdll_sys::nt::*;
use core::ffi::c_void;
use crate::dnslib::types::*;

#[no_mangle]
pub unsafe extern "system" fn Hostent_Init(
    Buffer: *mut *mut c_void,
    AddressFamily: u16,
    AddressSize: u32,
    AddressCount: u32,
    AliasCount: u32,
) -> *mut HOSTENT {
    let mut buffer_position = (*Buffer) as usize;

    // Align the hostent on the buffer's 4 byte boundary
    buffer_position = (buffer_position + 3) & !3;

    // Set up the basic data
    let Hostent = buffer_position as *mut HOSTENT;
    (*Hostent).h_length = AddressSize as i16;
    (*Hostent).h_addrtype = AddressFamily as i16;

    // Put aliases after Hostent
    (*Hostent).h_aliases = (((Hostent.add(1)) as usize) & !3usize) as *mut *mut i8;

    // Zero it out
    core::ptr::write_bytes(
        (*Hostent).h_aliases as *mut u8,
        0,
        (AliasCount as usize) * core::mem::size_of::<*mut i8>(),
    );

    // Put addresses after aliases
    (*Hostent).h_addr_list = (((*Hostent).h_aliases as usize)
        + (AliasCount as usize) * core::mem::size_of::<*mut i8>()
        + core::mem::size_of::<*mut i8>()) as *mut *mut i8;

    // Update the location
    buffer_position = ((*Hostent).h_addr_list as usize)
        + ((AddressCount as usize) * core::mem::size_of::<*mut i8>()
            + core::mem::size_of::<*mut i8>());

    // Send it back
    core::ptr::write_volatile(Buffer, buffer_position as *mut c_void);

    Hostent
}

#[no_mangle]
pub unsafe extern "system" fn Dns_PtrArrayToOffsetArray(mut List: *mut *mut i8, Base: usize) {
    // C: do { *List = (PCHAR)((ULONG_PTR)*List - Base); } while(*List++);
    // Terminate on the converted value being zero, then advance.
    loop {
        let v = *List;
        *List = ((v as usize).wrapping_sub(Base)) as *mut i8;
        let converted = *List;
        List = List.add(1);
        if converted.is_null() {
            break;
        }
    }
}

#[no_mangle]
pub unsafe extern "system" fn Hostent_ConvertToOffsets(Hostent: *mut HOSTENT) {
    if !(*Hostent).h_name.is_null() {
        (*Hostent).h_name =
            ((*Hostent).h_name as usize).wrapping_sub(Hostent as usize) as *mut i8;
    }

    if !(*Hostent).h_aliases.is_null() {
        (*Hostent).h_aliases =
            (((*Hostent).h_aliases as usize).wrapping_sub(Hostent as usize)) as *mut *mut i8;
        Dns_PtrArrayToOffsetArray((*Hostent).h_aliases as *mut *mut i8, Hostent as usize);
    }

    if !(*Hostent).h_addr_list.is_null() {
        Dns_PtrArrayToOffsetArray((*Hostent).h_addr_list as *mut *mut i8, Hostent as usize);
    }
}
