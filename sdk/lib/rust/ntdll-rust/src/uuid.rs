//! The handful of COM interface IIDs ntdll.dll actually references — replacing
//! the reuse of the cmake `libuuid.a` (which is widl-generated from ~50 `.idl`
//! files but only contributes 3 symbols here). These are fixed, standard COM
//! GUIDs; values verified byte-for-byte against `libuuid.a`:
//!   IID_IUnknown           {00000000-0000-0000-C000-000000000046}
//!   IID_IStream            {0000000C-0000-0000-C000-000000000046}
//!   IID_ISequentialStream  {0C733A30-2A1C-11CE-ADE5-00AA0044773D}
//!
//! ntdll.dll-proper only (the vista companion doesn't link uuid). Building a
//! full widl self-host for 3 fixed GUIDs would be gross overkill.

/// `GUID` / `IID` — identical C layout (`unknwn_i.c` etc.).
#[repr(C)]
pub struct Guid {
    pub data1: u32,
    pub data2: u16,
    pub data3: u16,
    pub data4: [u8; 8],
}

unsafe impl Sync for Guid {}

/// Define an IID as a `#[no_mangle]` static named exactly like the C symbol.
/// Rust applies the platform's own decoration to no_mangle: i686-gnu prefixes
/// data with `_` (consumers reference `_IID_*`), amd64/arm64 use the bare name —
/// so one definition is correct on all arches. (`export_name` does NOT work
/// here: on i686-gnu it double-decorates → `__IID_*`.)
macro_rules! iid {
    ($name:ident, $d1:expr, $d2:expr, $d3:expr, $d4:expr) => {
        #[no_mangle]
        #[allow(non_upper_case_globals)]
        pub static $name: Guid = Guid { data1: $d1, data2: $d2, data3: $d3, data4: $d4 };
    };
}

iid!(IID_IUnknown,
     0x0000_0000, 0x0000, 0x0000, [0xc0, 0, 0, 0, 0, 0, 0, 0x46]);
iid!(IID_IStream,
     0x0000_000c, 0x0000, 0x0000, [0xc0, 0, 0, 0, 0, 0, 0, 0x46]);
iid!(IID_ISequentialStream,
     0x0c73_3a30, 0x2a1c, 0x11ce, [0xad, 0xe5, 0x00, 0xaa, 0x00, 0x44, 0x77, 0x3d]);
