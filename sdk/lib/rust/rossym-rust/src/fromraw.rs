use crate::types::*;
use core::ffi::c_void;

extern "system" {
    fn DPRINT1(msg: *const u8);
    fn RosSymAllocMem(size: usize) -> *mut ROSSYM_INFO;
}

// ---- file-scope preamble (translated) ----
// No Rust declarations for #define NDEBUG (preprocessor symbol).
#[no_mangle]
pub unsafe extern "C" fn RosSymCreateFromRaw(
    RawData: *mut core::ffi::c_void,
    DataSize: usize,
    RosSymInfo: *mut *mut ROSSYM_INFO,
) -> i32 {
    let RosSymHeader: *mut ROSSYM_HEADER = RawData as *mut ROSSYM_HEADER;
    if ((*RosSymHeader).SymbolsOffset as usize) < (core::mem::size_of::<ROSSYM_HEADER>())
        || ((*RosSymHeader).StringsOffset as usize)
            < (((*RosSymHeader).SymbolsOffset + (*RosSymHeader).SymbolsLength) as usize)
        || DataSize
            < (((*RosSymHeader).StringsOffset + (*RosSymHeader).StringsLength) as usize)
        || ((*RosSymHeader).SymbolsLength as usize % core::mem::size_of::<ROSSYM_ENTRY>()) != 0
    {
        // DPRINT removed
        return 0;
    }
    *RosSymInfo = RosSymAllocMem(
        core::mem::size_of::<ROSSYM_INFO>()
            + (*RosSymHeader).SymbolsLength as usize
            + (*RosSymHeader).StringsLength as usize
            + 1,
    );
    if (*RosSymInfo).is_null() {
        // DPRINT removed
        return 0;
    }
    let info = *RosSymInfo;
    (*info).Symbols = (info as *mut u8).add(core::mem::size_of::<ROSSYM_INFO>())
        as *mut ROSSYM_ENTRY;
    (*info).SymbolsCount = (*RosSymHeader).SymbolsLength
        / core::mem::size_of::<ROSSYM_ENTRY>() as u32;
    (*info).Strings = (info as *mut u8)
        .add(core::mem::size_of::<ROSSYM_INFO>() + (*RosSymHeader).SymbolsLength as usize)
        as *mut i8;
    (*info).StringsLength = (*RosSymHeader).StringsLength;
    core::ptr::copy_nonoverlapping(
        (RosSymHeader as *mut u8).add((*RosSymHeader).SymbolsOffset as usize) as *const u8,
        (*info).Symbols as *mut u8,
        (*RosSymHeader).SymbolsLength as usize,
    );
    core::ptr::copy_nonoverlapping(
        (RosSymHeader as *mut u8).add((*RosSymHeader).StringsOffset as usize) as *const u8,
        (*info).Strings as *mut u8,
        (*RosSymHeader).StringsLength as usize,
    );
    (*info).Strings.add((*info).StringsLength as usize).write(0i8);
    return 1;
}
