use crate::types::*;
use ntkrnl_sys::globals::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::ddk::{PCSTR};
use core::ffi::c_void;

extern "C" {
    fn strcpy(dest: *mut i8, src: *const i8) -> *mut i8;
}

// ---- file-scope preamble (translated) ----
pub const NDEBUG: bool = true;

unsafe fn FindEntry(RosSymInfo: PROSSYM_INFO, RelativeAddress: usize) -> *mut ROSSYM_ENTRY {
    let mut Base: *mut ROSSYM_ENTRY;
    let mut Lim: u32;
    let mut Mid: *mut ROSSYM_ENTRY;
    let mut Low: *mut ROSSYM_ENTRY;

    Base = (*RosSymInfo).Symbols;
    if RelativeAddress < (*Base).Address {
        return core::ptr::null_mut();
    }

    Low = Base;

    Lim = (*RosSymInfo).SymbolsCount;
    while Lim != 0 {
        Mid = Base.add((Lim >> 1) as usize);
        if RelativeAddress == (*Mid).Address {
            return Mid;
        }
        if (*Mid).Address < RelativeAddress {
            Low = Mid;
            Base = Mid.add(1);
            Lim -= 1;
        }
        Lim >>= 1;
    }

    Low
}
#[no_mangle]
pub unsafe extern "C" fn RosSymGetAddressInformation(
    RosSymInfo: PROSSYM_INFO,
    RelativeAddress: usize,
    LineNumber: *mut u32,
    FileName: *mut i8,
    FunctionName: *mut i8,
) -> u8 {
    // DPRINT("RelativeAddress = 0x%08x\n", RelativeAddress);

    if (*RosSymInfo).Symbols.is_null()
        || (*RosSymInfo).SymbolsCount == 0
        || (*RosSymInfo).Strings.is_null()
        || (*RosSymInfo).StringsLength == 0
    {
        // DPRINT1("Uninitialized RosSymInfo\n");
        return 0; // FALSE
    }

    // ASSERT(LineNumber || FileName || FunctionName);

    let RosSymEntry: PROSSYM_ENTRY = FindEntry(RosSymInfo, RelativeAddress);

    if RosSymEntry.is_null() {
        // DPRINT("None of the requested information was found!\n");
        return 0;
    }

    if !LineNumber.is_null() {
        *LineNumber = (*RosSymEntry).SourceLine as u32;
    }
    if !FileName.is_null() {
        let Name: *const i8 = if (*RosSymEntry).FileOffset != 0 {
            ((*RosSymInfo).Strings as *const i8).add((*RosSymEntry).FileOffset as usize)
        } else {
            b"\0".as_ptr() as *const i8
        };
        strcpy(FileName, Name);
    }
    if !FunctionName.is_null() {
        let Name: *const i8 = if (*RosSymEntry).FunctionOffset != 0 {
            ((*RosSymInfo).Strings as *const i8).add((*RosSymEntry).FunctionOffset as usize)
        } else {
            b"\0".as_ptr() as *const i8
        };
        strcpy(FunctionName, Name);
    }

    1 // TRUE
}
