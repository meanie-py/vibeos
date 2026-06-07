use core::ffi::c_void;

extern "C" {
    fn memcpy(dest: *mut core::ffi::c_void, src: *const core::ffi::c_void, n: usize) -> *mut core::ffi::c_void;
}
#[no_mangle]
pub unsafe extern "C" fn RosSymGetRawDataLength(RosSymInfo: *mut ROSSYM_INFO) -> u32 {
    (core::mem::size_of::<ROSSYM_HEADER>() as u32)
        .wrapping_add(
            ((*RosSymInfo).SymbolsCount as u32)
                .wrapping_mul(core::mem::size_of::<ROSSYM_ENTRY>() as u32)
        )
        .wrapping_add((*RosSymInfo).StringsLength)
}
#[no_mangle]
pub unsafe extern "C" fn RosSymGetRawData(RosSymInfo: *mut ROSSYM_INFO, RawData: *mut core::ffi::c_void) {
    let RosSymHeader = RawData as *mut ROSSYM_HEADER;
    (*RosSymHeader).SymbolsOffset = core::mem::size_of::<ROSSYM_HEADER>() as u32;
    (*RosSymHeader).SymbolsLength = ((*RosSymInfo).SymbolsCount as usize * core::mem::size_of::<ROSSYM_ENTRY>()) as u32;
    (*RosSymHeader).StringsOffset = (*RosSymHeader).SymbolsOffset + (*RosSymHeader).SymbolsLength;
    (*RosSymHeader).StringsLength = (*RosSymInfo).StringsLength;

    memcpy(
        (RawData as *mut u8).add((*RosSymHeader).SymbolsOffset as usize) as *mut core::ffi::c_void,
        (*RosSymInfo).Symbols as *const core::ffi::c_void,
        (*RosSymHeader).SymbolsLength as usize,
    );
    memcpy(
        (RawData as *mut u8).add((*RosSymHeader).StringsOffset as usize) as *mut core::ffi::c_void,
        (*RosSymInfo).Strings as *const core::ffi::c_void,
        (*RosSymHeader).StringsLength as usize,
    );
}
