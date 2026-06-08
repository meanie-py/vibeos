//! rossym's own public types (sdk/include/reactos/rossym.h), hand-translated as
//! the type substrate for the cohab crate. rossym is a standalone lib — these
//! are NOT in ntkrnl-sys. Layout matches the C structs exactly (#[repr(C)],
//! raw field types: ULONG=u32, ULONG_PTR=usize, PCHAR=*mut i8, PVOID=*mut c_void,
//! BOOLEAN=u8). The non-DWARF (#else) ROSSYM_INFO is the one ReactOS compiles.
//! The modules import these via `use crate::types::*`.

use core::ffi::c_void;

#[repr(C)]
pub struct _ROSSYM_HEADER {
    pub SymbolsOffset: u32,
    pub SymbolsLength: u32,
    pub StringsOffset: u32,
    pub StringsLength: u32,
}
pub type ROSSYM_HEADER = _ROSSYM_HEADER;
pub type PROSSYM_HEADER = *mut _ROSSYM_HEADER;

#[repr(C)]
pub struct _ROSSYM_ENTRY {
    pub Address: usize,        // ULONG_PTR
    pub FunctionOffset: u32,   // ULONG
    pub FileOffset: u32,
    pub SourceLine: u32,
}
pub type ROSSYM_ENTRY = _ROSSYM_ENTRY;
pub type PROSSYM_ENTRY = *mut _ROSSYM_ENTRY;

#[repr(C)]
pub struct _ROSSYM_INFO {
    pub Symbols: PROSSYM_ENTRY,
    pub SymbolsCount: u32,
    pub Strings: *mut i8,      // PCHAR
    pub StringsLength: u32,
}
pub type ROSSYM_INFO = _ROSSYM_INFO;
pub type PROSSYM_INFO = *mut _ROSSYM_INFO;

#[repr(C)]
pub struct _ROSSYM_CALLBACKS {
    pub AllocMemProc: Option<unsafe extern "C" fn(usize) -> *mut c_void>,
    pub FreeMemProc: Option<unsafe extern "C" fn(*mut c_void)>,
    pub ReadFileProc: Option<unsafe extern "C" fn(*mut c_void, *mut c_void, u32) -> u8>,
    pub SeekFileProc: Option<unsafe extern "C" fn(*mut c_void, usize) -> u8>,
    pub MemGetProc: Option<unsafe extern "C" fn(*mut c_void, *mut usize, *mut c_void, u32) -> u8>,
}
pub type ROSSYM_CALLBACKS = _ROSSYM_CALLBACKS;
pub type PROSSYM_CALLBACKS = *mut _ROSSYM_CALLBACKS;
