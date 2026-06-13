//! startup/pseudo-reloc.c — the mingw runtime pseudo-relocator (auto-import
//! fixups). Non-cygwin, __MINGW64_VERSION_MAJOR path (the section-marking
//! variant). `_alloca(the_secs)` becomes a fixed stack buffer: PE allows at
//! most 96 sections, and the whole pass runs within this one call (the
//! global `the_secs` only points into this frame for its duration).
//!
//! For our modules the `__RUNTIME_PSEUDO_RELOC_LIST__`/`_END__` markers are
//! adjacent (empty list) unless lld emits auto-import fixups, so the pass is
//! a near no-op — but the full logic is reproduced.

use crate::pesect::{__mingw_GetSectionCount, __mingw_GetSectionForAddress, _GetPEImageBase};
use crate::winapi::*;
use core::ffi::c_void;

extern "C" {
    #[link_name = "__image_base__"]
    static __ImageBase: u8;
}

// startup/pseudo-reloc-list.c — the empty list markers (lld fills the span
// between them with auto-import fixups when it emits any; otherwise they're
// adjacent and the relocator sees an empty list).
#[no_mangle]
#[used]
pub static mut __RUNTIME_PSEUDO_RELOC_LIST_END__: u8 = 0;
#[no_mangle]
#[used]
pub static mut __RUNTIME_PSEUDO_RELOC_LIST__: u8 = 0;

const MAX_SECTIONS_BUF: usize = 96;

#[derive(Clone, Copy)]
#[repr(C)]
struct SSecInfo {
    old_protect: DWORD,
    sec_start: PBYTE,
    hash: *mut IMAGE_SECTION_HEADER,
}
impl SSecInfo {
    const fn zeroed() -> Self {
        SSecInfo {
            old_protect: 0,
            sec_start: core::ptr::null_mut(),
            hash: core::ptr::null_mut(),
        }
    }
}

#[no_mangle]
pub static mut the_secs: *mut c_void = core::ptr::null_mut();
#[no_mangle]
pub static mut maxSections: i32 = 0;

#[no_mangle]
pub unsafe extern "C" fn __report_error(_msg: *const u8) -> ! {
    // The cygwin/mingw fprintf path isn't load-bearing for boot; a
    // malformed reloc list is fatal either way.
    abort()
}

unsafe fn secs() -> *mut SSecInfo {
    the_secs as *mut SSecInfo
}

unsafe fn mark_section_writable(addr: LPVOID) {
    let n = maxSections;
    let mut i = 0i32;
    while i < n {
        let s = &*secs().add(i as usize);
        if s.sec_start <= addr as PBYTE
            && (addr as PBYTE) < s.sec_start.offset((*s.hash).VirtualSize as isize)
        {
            return;
        }
        i += 1;
    }
    let i = maxSections as usize;
    let h = __mingw_GetSectionForAddress(addr);
    if h.is_null() {
        __report_error(b"no image-section\0".as_ptr());
    }
    let s = &mut *secs().add(i);
    s.hash = h;
    s.old_protect = 0;
    s.sec_start = _GetPEImageBase().offset((*h).VirtualAddress as isize);

    let mut b: MEMORY_BASIC_INFORMATION = core::mem::zeroed();
    if VirtualQuery(
        s.sec_start as *const c_void,
        &mut b,
        core::mem::size_of::<MEMORY_BASIC_INFORMATION>(),
    ) == 0
    {
        __report_error(b"VirtualQuery failed\0".as_ptr());
    }
    if b.Protect != PAGE_EXECUTE_READWRITE && b.Protect != PAGE_READWRITE {
        if VirtualProtect(
            b.BaseAddress,
            b.RegionSize,
            PAGE_EXECUTE_READWRITE,
            &mut s.old_protect,
        ) == 0
        {
            __report_error(b"VirtualProtect failed\0".as_ptr());
        }
    }
    maxSections += 1;
}

unsafe fn restore_modified_sections() {
    let n = maxSections;
    let mut i = 0i32;
    while i < n {
        let s = &*secs().add(i as usize);
        if s.old_protect == 0 {
            i += 1;
            continue;
        }
        let mut b: MEMORY_BASIC_INFORMATION = core::mem::zeroed();
        if VirtualQuery(
            s.sec_start as *const c_void,
            &mut b,
            core::mem::size_of::<MEMORY_BASIC_INFORMATION>(),
        ) == 0
        {
            __report_error(b"VirtualQuery failed\0".as_ptr());
        }
        let mut oldprot: DWORD = 0;
        VirtualProtect(b.BaseAddress, b.RegionSize, s.old_protect, &mut oldprot);
        i += 1;
    }
}

#[no_mangle]
pub unsafe extern "C" fn __write_memory(addr: *mut c_void, src: *const c_void, len: usize) {
    if len == 0 {
        return;
    }
    mark_section_writable(addr);

    let mut b: MEMORY_BASIC_INFORMATION = core::mem::zeroed();
    if VirtualQuery(
        addr,
        &mut b,
        core::mem::size_of::<MEMORY_BASIC_INFORMATION>(),
    ) == 0
    {
        __report_error(b"VirtualQuery failed\0".as_ptr());
    }
    let mut oldprot: DWORD = 0;
    let mut call_unprotect = false;
    if b.Protect != PAGE_EXECUTE_READWRITE && b.Protect != PAGE_READWRITE {
        call_unprotect = true;
        VirtualProtect(b.BaseAddress, b.RegionSize, PAGE_EXECUTE_READWRITE, &mut oldprot);
    }
    memcpy(addr, src, len);
    if call_unprotect && b.Protect != PAGE_EXECUTE_READWRITE && b.Protect != PAGE_READWRITE {
        VirtualProtect(b.BaseAddress, b.RegionSize, oldprot, &mut oldprot);
    }
}

#[repr(C)]
struct RpItemV1 {
    addend: DWORD,
    target: DWORD,
}
#[repr(C)]
struct RpItemV2 {
    sym: DWORD,
    target: DWORD,
    flags: DWORD,
}
#[repr(C)]
struct RpV2Hdr {
    magic1: DWORD,
    magic2: DWORD,
    version: DWORD,
}

const RP_VERSION_V1: DWORD = 0;
const RP_VERSION_V2: DWORD = 1;

unsafe fn do_pseudo_reloc(start: *mut c_void, end: *mut c_void, base: *mut c_void) {
    let mut reloc_target = (end as isize) - (start as isize);
    if reloc_target < 8 {
        return;
    }
    let mut v2_hdr = start as *const RpV2Hdr;
    if reloc_target >= 12
        && (*v2_hdr).magic1 == 0
        && (*v2_hdr).magic2 == 0
        && (*v2_hdr).version == RP_VERSION_V1
    {
        v2_hdr = v2_hdr.add(1);
    }

    if (*v2_hdr).magic1 != 0 || (*v2_hdr).magic2 != 0 {
        // v1 relocations.
        let mut o = v2_hdr as *const RpItemV1;
        while (o as *const c_void) < end {
            let rt = (base as isize) + (*o).target as isize;
            let p = rt as *mut DWORD;
            let newval = (*p).wrapping_add((*o).addend);
            __write_memory(
                rt as *mut c_void,
                &newval as *const DWORD as *const c_void,
                core::mem::size_of::<DWORD>(),
            );
            o = o.add(1);
        }
        return;
    }

    if (*v2_hdr).version != RP_VERSION_V2 {
        __report_error(b"unknown pseudo reloc version\0".as_ptr());
    }

    let mut r = (v2_hdr as *const RpV2Hdr).add(1) as *const RpItemV2;
    while (r as *const c_void) < end {
        reloc_target = (base as isize) + (*r).target as isize;
        let mut addr_imp = (base as isize) + (*r).sym as isize;
        addr_imp = *(addr_imp as *const isize);

        let bits = (*r).flags & 0xff;
        let mut reldata: isize;
        match bits {
            8 => {
                reldata = *(reloc_target as *const u8) as isize;
                if reldata & 0x80 != 0 {
                    reldata |= !(0xffisize);
                }
            }
            16 => {
                reldata = *(reloc_target as *const u16) as isize;
                if reldata & 0x8000 != 0 {
                    reldata |= !(0xffffisize);
                }
            }
            32 => {
                reldata = *(reloc_target as *const u32) as isize;
                #[cfg(target_pointer_width = "64")]
                if reldata & 0x8000_0000 != 0 {
                    reldata |= !(0xffff_ffffisize);
                }
            }
            #[cfg(target_pointer_width = "64")]
            64 => {
                reldata = *(reloc_target as *const u64) as isize;
            }
            _ => {
                __report_error(b"unknown pseudo reloc bit size\0".as_ptr());
            }
        }

        reldata -= (base as isize) + (*r).sym as isize;
        reldata += addr_imp;

        let n = match bits {
            8 => 1,
            16 => 2,
            32 => 4,
            #[cfg(target_pointer_width = "64")]
            64 => 8,
            _ => 0,
        };
        __write_memory(
            reloc_target as *mut c_void,
            &reldata as *const isize as *const c_void,
            n,
        );
        r = r.add(1);
    }
}

#[no_mangle]
pub unsafe extern "C" fn _pei386_runtime_relocator() {
    static mut WAS_INIT: i32 = 0;
    if WAS_INIT != 0 {
        return;
    }
    WAS_INIT += 1;

    let mut buf = [SSecInfo::zeroed(); MAX_SECTIONS_BUF];
    let m_secs = __mingw_GetSectionCount() as usize;
    // _alloca(sizeof*mSecs) → bounded stack buffer; PE caps sections at 96.
    debug_assert!(m_secs <= MAX_SECTIONS_BUF);
    the_secs = buf.as_mut_ptr() as *mut c_void;
    maxSections = 0;

    do_pseudo_reloc(
        core::ptr::addr_of_mut!(__RUNTIME_PSEUDO_RELOC_LIST__) as *mut c_void,
        core::ptr::addr_of_mut!(__RUNTIME_PSEUDO_RELOC_LIST_END__) as *mut c_void,
        core::ptr::addr_of!(__ImageBase) as *mut c_void,
    );
    restore_modified_sections();
    // keep buf alive across the calls above.
    core::hint::black_box(&mut buf);
}
