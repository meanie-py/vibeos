//! guid.c — the four recycle-bin interface IIDs.
//!
//! Hand-written rather than bought: the C is four `DEFINE_GUID` macro invocations with no
//! function bodies at all, so there is nothing for `rustconv translate` to pay per-function
//! for. The values are transcribed digit-for-digit from
//! dll/win32/shell32/shellrecyclebin/guid.c and are contiguous — the four interfaces share
//! 392ec73a-45e9-43de-a841-4f321aeaa580 and differ only in the final byte (0x80..0x83).
//!
//! `#[no_mangle]` on purpose: the island carries the matching
//! `extern "C" { pub static IID_IRecycleBin: IID; }` declarations that recyclebin.h emits, and
//! every consumer — inside this crate and in shell32 across the C ABI — resolves to THESE
//! definitions at link time. guid.c was the only definition site in the C build too.

use crate::prelude::*;

/// `392ec73a-45e9-43de-a841-4f321aeaa580`
#[no_mangle]
pub static IID_IRecycleBin: IID = IID {
    Data1: 0x392ec73a,
    Data2: 0x45e9,
    Data3: 0x43de,
    Data4: [0xa8, 0x41, 0x4f, 0x32, 0x1a, 0xea, 0xa5, 0x80],
};

/// `392ec73a-45e9-43de-a841-4f321aeaa581`
#[no_mangle]
pub static IID_IRecycleBinEnumList: IID = IID {
    Data1: 0x392ec73a,
    Data2: 0x45e9,
    Data3: 0x43de,
    Data4: [0xa8, 0x41, 0x4f, 0x32, 0x1a, 0xea, 0xa5, 0x81],
};

/// `392ec73a-45e9-43de-a841-4f321aeaa582`
#[no_mangle]
pub static IID_IRecycleBinFile: IID = IID {
    Data1: 0x392ec73a,
    Data2: 0x45e9,
    Data3: 0x43de,
    Data4: [0xa8, 0x41, 0x4f, 0x32, 0x1a, 0xea, 0xa5, 0x82],
};

/// `392ec73a-45e9-43de-a841-4f321aeaa583`
#[no_mangle]
pub static IID_IRecycleBin5: IID = IID {
    Data1: 0x392ec73a,
    Data2: 0x45e9,
    Data3: 0x43de,
    Data4: [0xa8, 0x41, 0x4f, 0x32, 0x1a, 0xea, 0xa5, 0x83],
};
