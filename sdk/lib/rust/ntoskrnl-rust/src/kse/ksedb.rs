//! KSE driver database (upstream ntoskrnl/kse/ksedb.c), hand-ported.
//!
//! Upstream links apphelp's sdbread.c a second time as `sdbread_km` for the
//! nine tag-walking functions this file uses. Here they are a private reader
//! over the pool copy of drvmain.sdb, with the same semantics as
//! apphelp-rust's `sdbread.rs` (tag = WORD, type in the top nibble; list and
//! string tags carry a DWORD size; every read is bounds-checked against the
//! image, so a malformed database yields TAGID_NULL / TAG_NULL, never a
//! fault). c2rust could not translate the C (its `static const UNICODE_STRING`
//! has no static spelling), and the reader had no kernel home anyway.
//!
//! https://www.geoffchappell.com/studies/windows/km/ntoskrnl/api/kshim/drvmain.htm

use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::ddk::{PCUNICODE_STRING, PKSE_SHIM, PLDR_DATA_TABLE_ENTRY, RtlEqualUnicodeString, RtlImageNtHeader, RtlInitUnicodeString};
use ntkrnl_sys::ddk::_UNICODE_STRING;
use super::defs::{STATUS_INVALID_IMAGE_FORMAT, TAG_KSE_DATABASE};
use ntkrnl_sys::ex::{ExAllocatePoolWithTag, ExFreePoolWithTag};
use ntkrnl_sys::mm::PagedPool;
use ntkrnl_sys::ddk::PCSTR;
use ntkrnl_sys::driver_ddk::DbgPrint;

use super::kse::KsepAddShimByName;

type TAG = u16;
type TAGID = u32;

const TAGID_NULL: TAGID = 0;
const TAGID_ROOT: TAGID = 0;
const _TAGID_ROOT: TAGID = 12; // first tag after the 12-byte header
const TAG_NULL: TAG = 0;

const TAG_TYPE_MASK: TAG = 0xF000;
const TAG_TYPE_STRINGREF: TAG = 0x6000;
const TAG_TYPE_LIST: TAG = 0x7000;
const TAG_TYPE_STRING: TAG = 0x8000;
const TAG_TYPE_DWORD: TAG = 0x4000;

const TAG_CHECKSUM: TAG = 0x3 | TAG_TYPE_DWORD;
const TAG_LINK_DATE: TAG = 0x1D | TAG_TYPE_DWORD;
const TAG_NAME: TAG = 0x1 | TAG_TYPE_STRINGREF;
const TAG_DATABASE: TAG = 0x1 | TAG_TYPE_LIST;
const TAG_MATCHING_FILE: TAG = 0x8 | TAG_TYPE_LIST;
const TAG_KDRIVER: TAG = 0x1C | TAG_TYPE_LIST;
const TAG_KSHIM_REF: TAG = 0x26 | TAG_TYPE_LIST;
const TAG_STRINGTABLE: TAG = 0x801 | TAG_TYPE_LIST;

const SDB_HEADER_SIZE: ULONG = 12;
const SDB_HEADER_MAGIC: ULONG = u32::from_le_bytes(*b"sdbf"); // C 'fbds'

/// The pool copy of drvmain.sdb (the C's `DB` with only the fields used).
struct Db {
    data: *const u8,
    size: u32,
    major: u32,
    minor: u32,
    stringtable: TAGID,
}

static mut KSE_DRIVER_DATABASE: Db = Db { data: core::ptr::null(), size: 0, major: 0, minor: 0, stringtable: 0 };
static mut KSE_DATABASE_ROOT: TAGID = TAGID_NULL;

impl Db {
    #[inline]
    unsafe fn read_u16(&self, off: u32) -> Option<u16> {
        let end = off.checked_add(2)?;
        if end > self.size {
            return None;
        }
        Some(core::ptr::read_unaligned(self.data.add(off as usize) as *const u16))
    }
    #[inline]
    unsafe fn read_u32(&self, off: u32) -> Option<u32> {
        let end = off.checked_add(4)?;
        if end > self.size {
            return None;
        }
        Some(core::ptr::read_unaligned(self.data.add(off as usize) as *const u32))
    }

    unsafe fn tag(&self, tagid: TAGID) -> TAG {
        self.read_u16(tagid).unwrap_or(TAG_NULL)
    }

    /// SdbGetTagDataSize.
    unsafe fn data_size(&self, tagid: TAGID) -> u32 {
        const SIZES: [u32; 6] = [0, 1, 2, 4, 8, 4]; // NULL BYTE WORD DWORD QWORD STRINGREF
        let ty = self.tag(tagid) & TAG_TYPE_MASK;
        if ty == TAG_NULL {
            return 0;
        }
        if ty <= TAG_TYPE_STRINGREF {
            return SIZES[((ty >> 12) - 1) as usize];
        }
        self.read_u32(tagid.wrapping_add(2)).unwrap_or(0)
    }

    /// SdbpGetTagSize: the whole tag, header included.
    unsafe fn tag_size(&self, tagid: TAGID) -> u32 {
        let ty = self.tag(tagid) & TAG_TYPE_MASK;
        if ty == TAG_NULL {
            return 0;
        }
        let size = self.data_size(tagid);
        if ty <= TAG_TYPE_STRINGREF {
            size.wrapping_add(2)
        } else {
            size.wrapping_add(2 + 4)
        }
    }

    unsafe fn first_child(&self, parent: TAGID) -> TAGID {
        if parent == TAGID_ROOT {
            return if self.size <= _TAGID_ROOT { TAGID_NULL } else { _TAGID_ROOT };
        }
        if self.tag(parent) & TAG_TYPE_MASK != TAG_TYPE_LIST {
            return TAGID_NULL;
        }
        parent.wrapping_add(2 + 4)
    }

    unsafe fn next_child(&self, parent: TAGID, prev: TAGID) -> TAGID {
        let prev_size = self.tag_size(prev);
        if prev_size == 0 {
            return TAGID_NULL;
        }
        let next = prev.wrapping_add(prev_size);
        if next >= self.size || next <= prev {
            return TAGID_NULL;
        }
        if parent == TAGID_ROOT {
            return next;
        }
        let parent_size = self.tag_size(parent);
        if parent_size == 0 || next >= parent.wrapping_add(parent_size) {
            return TAGID_NULL;
        }
        next
    }

    unsafe fn find_first(&self, parent: TAGID, tag: TAG) -> TAGID {
        let mut it = self.first_child(parent);
        while it != TAGID_NULL {
            if self.tag(it) == tag {
                return it;
            }
            it = self.next_child(parent, it);
        }
        TAGID_NULL
    }

    unsafe fn find_next(&self, parent: TAGID, prev: TAGID) -> TAGID {
        let tag = self.tag(prev);
        let mut it = self.next_child(parent, prev);
        while it != TAGID_NULL {
            if self.tag(it) == tag {
                return it;
            }
            it = self.next_child(parent, it);
        }
        TAGID_NULL
    }

    unsafe fn read_dword(&self, tagid: TAGID, default: u32) -> u32 {
        if self.tag(tagid) & TAG_TYPE_MASK != TAG_TYPE_DWORD {
            return default;
        }
        self.read_u32(tagid.wrapping_add(2)).unwrap_or(default)
    }

    /// SdbGetStringTagPtr: a pointer into the image, or null. The string must
    /// be NUL-terminated inside the image (the C trusted it; a pool copy of a
    /// truncated database must not send RtlInitUnicodeString past the end).
    unsafe fn string_ptr(&self, tagid: TAGID) -> *const u16 {
        let tag = self.tag(tagid);
        if tag == TAG_NULL {
            return core::ptr::null();
        }
        let offset = match tag & TAG_TYPE_MASK {
            TAG_TYPE_STRINGREF => {
                if self.stringtable == TAGID_NULL {
                    return core::ptr::null();
                }
                let Some(r) = self.read_u32(tagid.wrapping_add(2)) else { return core::ptr::null() };
                self.stringtable.wrapping_add(r).wrapping_add(2 + 4)
            }
            TAG_TYPE_STRING => tagid.wrapping_add(2 + 4),
            _ => return core::ptr::null(),
        };
        let mut o = offset;
        loop {
            match self.read_u16(o) {
                None => return core::ptr::null(),
                Some(0) => break,
                Some(_) => o = o.wrapping_add(2),
            }
        }
        self.data.add(offset as usize) as *const u16
    }
}

#[inline]
unsafe fn db() -> &'static Db {
    &*core::ptr::addr_of!(KSE_DRIVER_DATABASE)
}

#[no_mangle]
pub unsafe extern "system" fn KseDbInitialize(DatabaseImage: PVOID, DatabaseSize: ULONG) -> NTSTATUS {
    let header = DatabaseImage as *const ULONG;
    if DatabaseImage.is_null()
        || DatabaseSize <= SDB_HEADER_SIZE
        || core::ptr::read_unaligned(header.add(2)) != SDB_HEADER_MAGIC
    {
        return STATUS_INVALID_IMAGE_FORMAT;
    }

    // The loader block is freed after boot, so keep a copy.
    let data = ExAllocatePoolWithTag(PagedPool, DatabaseSize as SIZE_T, TAG_KSE_DATABASE) as *mut u8;
    if data.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }
    core::ptr::copy_nonoverlapping(DatabaseImage as *const u8, data, DatabaseSize as usize);

    let d = &mut *core::ptr::addr_of_mut!(KSE_DRIVER_DATABASE);
    d.data = data;
    d.size = DatabaseSize;
    d.major = core::ptr::read_unaligned(header);
    d.minor = core::ptr::read_unaligned(header.add(1));
    d.stringtable = TAGID_NULL;
    d.stringtable = d.find_first(TAGID_ROOT, TAG_STRINGTABLE);
    KSE_DATABASE_ROOT = d.find_first(TAGID_ROOT, TAG_DATABASE);
    if KSE_DATABASE_ROOT == TAGID_NULL {
        ExFreePoolWithTag(data as PVOID, TAG_KSE_DATABASE);
        *d = Db { data: core::ptr::null(), size: 0, major: 0, minor: 0, stringtable: 0 };
        return STATUS_INVALID_IMAGE_FORMAT;
    }

    DbgPrint(
        b"KSE: Loaded driver database version %lu.%lu (%lu bytes)\n\0".as_ptr() as PCSTR,
        d.major,
        d.minor,
        DatabaseSize,
    );
    STATUS_SUCCESS
}

unsafe fn db_string_equals(parent: TAGID, tag: TAG, string: PCUNICODE_STRING) -> bool {
    let d = db();
    let ptr = d.string_ptr(d.find_first(parent, tag));
    if ptr.is_null() {
        return false;
    }
    let mut value: _UNICODE_STRING = core::mem::zeroed();
    RtlInitUnicodeString(&mut value as *mut _ as *mut _, ptr as *mut u16);
    RtlEqualUnicodeString(&value as *const _ as PCUNICODE_STRING, string, 1) != 0
}

unsafe fn matching_file_matches(matching_file: TAGID, ldr: PLDR_DATA_TABLE_ENTRY) -> bool {
    static SELF_NAME: [u16; 2] = [b'*' as u16, 0];
    let mut self_str: _UNICODE_STRING = core::mem::zeroed();
    RtlInitUnicodeString(&mut self_str as *mut _ as *mut _, SELF_NAME.as_ptr() as *mut u16);

    // Only the driver itself can be checked here.
    if !db_string_equals(matching_file, TAG_NAME, &self_str as *const _ as PCUNICODE_STRING)
        && !db_string_equals(matching_file, TAG_NAME, core::ptr::addr_of!((*ldr).BaseDllName) as PCUNICODE_STRING)
    {
        return false;
    }

    let nt = RtlImageNtHeader((*ldr).DllBase);
    if nt.is_null() {
        return false;
    }

    let d = db();
    let mut attr = d.first_child(matching_file);
    while attr != TAGID_NULL {
        match d.tag(attr) {
            TAG_NAME => {}
            TAG_LINK_DATE => {
                if d.read_dword(attr, 0) != (*nt).FileHeader.TimeDateStamp {
                    return false;
                }
            }
            TAG_CHECKSUM => {
                if d.read_dword(attr, 0) != (*nt).OptionalHeader.CheckSum {
                    return false;
                }
            }
            other => {
                // Do not shim a driver based on attributes that were not checked.
                DbgPrint(
                    b"KSE: Unsupported matching attribute 0x%x for %wZ\n\0".as_ptr() as PCSTR,
                    other as ULONG,
                    core::ptr::addr_of!((*ldr).BaseDllName),
                );
                return false;
            }
        }
        attr = d.next_child(matching_file, attr);
    }
    true
}

#[no_mangle]
pub unsafe extern "system" fn KseDbGetShimsForDriver(
    LdrEntry: PLDR_DATA_TABLE_ENTRY,
    Shims: *mut PKSE_SHIM,
    NumberOfShims: PULONG,
) {
    let root = KSE_DATABASE_ROOT;
    if root == TAGID_NULL {
        return;
    }
    let d = db();
    let base_name = core::ptr::addr_of!((*LdrEntry).BaseDllName) as PCUNICODE_STRING;

    let mut driver = d.find_first(root, TAG_KDRIVER);
    while driver != TAGID_NULL {
        if db_string_equals(driver, TAG_NAME, base_name) {
            let mut matched = true;
            let mut mf = d.find_first(driver, TAG_MATCHING_FILE);
            while mf != TAGID_NULL && matched {
                matched = matching_file_matches(mf, LdrEntry);
                mf = d.find_next(driver, mf);
            }

            if !matched {
                DbgPrint(
                    b"KSE: %wZ is in the driver database, but does not match\n\0".as_ptr() as PCSTR,
                    base_name,
                );
            } else {
                let mut shim_ref = d.find_first(driver, TAG_KSHIM_REF);
                while shim_ref != TAGID_NULL {
                    let name = d.string_ptr(d.find_first(shim_ref, TAG_NAME));
                    if !name.is_null() {
                        let mut shim_name: _UNICODE_STRING = core::mem::zeroed();
                        RtlInitUnicodeString(&mut shim_name as *mut _ as *mut _, name as *mut u16);
                        KsepAddShimByName(&shim_name as *const _ as PCUNICODE_STRING, Shims, NumberOfShims);
                    }
                    shim_ref = d.find_next(driver, shim_ref);
                }
            }
        }
        driver = d.find_next(root, driver);
    }
}
