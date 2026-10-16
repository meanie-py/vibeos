use crate::*;
use core::ptr::{null, null_mut};

const VHD: VirtualStorageType = VirtualStorageType {
    device_id: 2,
    vendor_id: Guid {
        data1: 0xec98_4aec, data2: 0xa0f9, data3: 0x47e9,
        data4: [0x90, 0x1f, 0x71, 0x41, 0x5a, 0x66, 0x34, 0x5b],
    },
};
const H: *mut c_void = 0x1234 as *mut c_void;
const INVALID: *mut c_void = usize::MAX as *mut c_void;

fn w(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(Some(0)).collect()
}

fn open(ty: *const VirtualStorageType, path: &str, mask: u32, flags: u32, ver: Option<u32>) -> (u32, *mut c_void) {
    let p = w(path);
    let mut h = 0xdead_beef_usize as *mut c_void;
    let par = OpenVirtualDiskParameters { version: ver.unwrap_or(0) };
    let pp = if ver.is_some() { &par as *const _ } else { null() };
    let r = unsafe { OpenVirtualDisk(ty, p.as_ptr(), mask, flags, pp, &mut h) };
    (r, h)
}

#[test]
fn layout_matches_headers() {
    assert_eq!(core::mem::size_of::<StorageDependencyInfoType1>(), 28);
    if cfg!(target_pointer_width = "64") {
        assert_eq!(core::mem::size_of::<StorageDependencyInfoType2>(), 64);
        assert_eq!(STORAGE_DEPENDENCY_INFO_SIZE, 72);
    } else {
        assert_eq!(core::mem::size_of::<StorageDependencyInfoType2>(), 48);
        assert_eq!(STORAGE_DEPENDENCY_INFO_SIZE, 56);
    }
}

#[test]
fn open_measured_cases() {
    // nonexistent + NULL parameters: 2, handle untouched
    let (r, h) = open(&VHD, "C:\\nonexistent.vhd", VIRTUAL_DISK_ACCESS_ALL, 0, None);
    assert_eq!(r, 2);
    assert_eq!(h as usize, 0xdead_beef);
    // NULL type: 87
    assert_eq!(open(null(), "C:\\nonexistent.vhd", VIRTUAL_DISK_ACCESS_ALL, 0, None).0, 87);
}

#[test]
fn open_validation_matches_wine_test_on_windows() {
    let unknown = VirtualStorageType { device_id: 0, vendor_id: Guid { data1: 0, data2: 0, data3: 0, data4: [0; 8] } };
    // NULL path
    let mut h = null_mut();
    assert_eq!(unsafe { OpenVirtualDisk(&unknown, null(), 0, 1, null(), &mut h) }, 87);
    // NULL handle out-pointer
    let p = w("test.vhd");
    assert_eq!(unsafe { OpenVirtualDisk(&unknown, p.as_ptr(), 0, 1, null(), null_mut()) }, 87);
    // versions 1..3 accepted (reach the file open -> not found), 0 and 4 rejected
    for v in 1..=3 {
        assert_eq!(open(&unknown, "test.vhd", 0, 1, Some(v)).0, 2, "version {v}");
    }
    assert_eq!(open(&unknown, "test.vhd", 0, 1, Some(0)).0, 87);
    assert_eq!(open(&unknown, "test.vhd", 0, 1, Some(4)).0, 87);
    // mask 0xffffff, bad flags
    assert_eq!(open(&unknown, "test.vhd", 0xff_ffff, 1, Some(1)).0, 87);
    assert_eq!(open(&unknown, "test.vhd", 0, 0x200, Some(1)).0, 87);
    // flags NONE with Version1 gets to the file system
    assert_eq!(open(&unknown, "test.vhd", 0, 0, Some(1)).0, 2);
}

#[test]
fn open_existing_file_is_provider_not_found_and_open_errors_pass_through() {
    let (r, h) = open(&VHD, "C:\\exists.vhd", VIRTUAL_DISK_ACCESS_ALL, 0, None);
    assert_eq!(r, 0xC03A_0014);
    assert_eq!(h as usize, 0xdead_beef, "handle must stay untouched on failure");
    assert_eq!(open(&VHD, "C:\\nodir\\x.vhd", 0, 0, None).0, 3);
    assert_eq!(open(&VHD, "C:\\locked.vhd", 0, 0, None).0, 32);
}

#[test]
fn detach() {
    unsafe {
        assert_eq!(DetachVirtualDisk(INVALID, 0, 0), 6);
        assert_eq!(DetachVirtualDisk(null_mut(), 0, 0), 6);
        assert_eq!(DetachVirtualDisk(H, 0, 0), 0xC03A_0015);
        assert_eq!(DetachVirtualDisk(H, 1, 0), 87);
    }
}

fn sdi(h: *mut c_void, flags: u32, size: u32, version: u32) -> (u32, Option<u32>, u32) {
    // 4096-byte zeroed, 8-aligned buffer; NumberEntries pre-poisoned
    let mut buf = vec![0u64; 512];
    let info = buf.as_mut_ptr() as *mut StorageDependencyInfo;
    let mut used = 0xaaaa_aaaau32;
    unsafe {
        (*info).version = version;
        (*info).number_entries = 0x77;
        let r = GetStorageDependencyInformation(h, flags, size, info, &mut used);
        (r, if used == 0xaaaa_aaaa { None } else { Some(used) }, (*info).number_entries)
    }
}

#[test]
fn storage_dependency_measured() {
    // HOST_VOLUMES on a non-virtual volume
    assert_eq!(sdi(H, 1, 4096, 2).0, 0xC03A_0015);
    // NONE: success, 0 entries, SizeUsed = sizeof(STORAGE_DEPENDENCY_INFO)
    let (r, used, n) = sdi(H, 0, 4096, 2);
    assert_eq!((r, used, n), (0, Some(STORAGE_DEPENDENCY_INFO_SIZE), 0));
    if cfg!(target_pointer_width = "64") {
        assert_eq!(used, Some(72));
    }
    // DISK_HANDLE: 87 regardless of the handle kind
    assert_eq!(sdi(H, 2, 4096, 2).0, 87);
    // bad handles: 6
    assert_eq!(sdi(INVALID, 0, 4096, 2).0, 6);
    assert_eq!(sdi(null_mut(), 1, 4096, 2).0, 6);
    // NULL info: 87
    let mut used = 0;
    assert_eq!(unsafe { GetStorageDependencyInformation(H, 0, 4096, null_mut(), &mut used) }, 87);
}

#[test]
fn storage_dependency_edges() {
    // size 0 (Wine's test): 87, SizeUsed untouched
    assert_eq!(sdi(H, 2, 0, 2), (87, None, 0x77));
    assert_eq!(sdi(H, 0, 0, 2).0, 87);
    // unknown flag bit
    assert_eq!(sdi(H, 4, 4096, 2).0, 87);
    // too small: 122 and the required size reported, entry count untouched
    assert_eq!(sdi(H, 0, STORAGE_DEPENDENCY_INFO_SIZE - 1, 2), (122, Some(STORAGE_DEPENDENCY_INFO_SIZE), 0x77));
    // exact size ok, version 1 ok, version 0/3 rejected
    assert_eq!(sdi(H, 0, STORAGE_DEPENDENCY_INFO_SIZE, 2).0, 0);
    assert_eq!(sdi(H, 0, 4096, 1).0, 0);
    assert_eq!(sdi(H, 0, 4096, 0).0, 87);
    assert_eq!(sdi(H, 0, 4096, 3).0, 87);
    // SizeUsed is optional
    let mut buf = vec![0u64; 16];
    let info = buf.as_mut_ptr() as *mut StorageDependencyInfo;
    unsafe {
        (*info).version = 2;
        assert_eq!(GetStorageDependencyInformation(H, 0, 128, info, null_mut()), 0);
    }
}
