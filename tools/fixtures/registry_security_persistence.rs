#![allow(non_snake_case, non_camel_case_types, dead_code)]
extern crate self as ntkrnl_sys;
use std::collections::HashMap;
use std::sync::Mutex;

pub mod base {
    pub type NTSTATUS = i32;
    pub const STATUS_SUCCESS: i32 = 0;
    pub const STATUS_INSUFFICIENT_RESOURCES: i32 = -2;
}
pub mod constants {
    pub const STATUS_INVALID_PARAMETER: i32 = -1;
    pub const STATUS_INVALID_SECURITY_DESCR: i32 = -3;
    pub const STATUS_MEDIA_WRITE_PROTECTED: i32 = -4;
}
pub mod ddk {
    #[repr(C)] pub struct HHIVE { pub ReadOnly: u8 }
    #[repr(C)] pub struct CMHIVE { pub Hive: HHIVE, pub SecurityLock: usize }
    pub type PHHIVE = *mut HHIVE;
    pub type PCMHIVE = *mut CMHIVE;
    pub struct KeyFields { pub KeyHive: PHHIVE, pub KeyCell: u32 }
    pub struct KeyUnion { pub __bindgen_anon_1: KeyFields }
    pub struct KCB { pub __bindgen_anon_2: KeyUnion }
    pub type PCM_KEY_CONTROL_BLOCK = *mut KCB;
    #[repr(C)] pub struct NODE { pub Signature: u16, pub Security: u32 }
    pub type PCM_KEY_NODE = *mut NODE;
    #[repr(C)] pub struct CM_KEY_SECURITY {
        pub Signature: u16, pub Reserved: u16, pub Flink: u32, pub Blink: u32,
        pub ReferenceCount: u32, pub DescriptorLength: u32, pub Descriptor: [u8; 20],
    }
    pub type PCM_KEY_SECURITY = *mut CM_KEY_SECURITY;
    pub unsafe fn CmpLockHiveFlusherShared(_: PCMHIVE) { super::lock_delta(1); }
    pub unsafe fn CmpUnlockHiveFlusher(_: PCMHIVE) { super::lock_delta(-1); }
    pub unsafe fn ExAcquirePushLockExclusive(_: *mut usize) { super::lock_delta(1); }
    pub unsafe fn ExReleasePushLockExclusive(_: *mut usize) { super::lock_delta(-1); }
    pub unsafe fn KeEnterCriticalRegion() { super::lock_delta(1); }
    pub unsafe fn KeLeaveCriticalRegion() { super::lock_delta(-1); }
}

#[path = __SHIPPING_MODULE__]
mod shipping;

#[derive(Default)]
struct State {
    cells: HashMap<u32, Box<[u64]>>,
    refs: HashMap<u32, i32>,
    next: u32,
    fail_allocate: bool,
    fail_map: Option<u32>,
    fail_dirty: Option<u32>,
    locks: i32,
    storage: Option<u32>,
}
static STATE: Mutex<Option<State>> = Mutex::new(None);
static TEST_LOCK: Mutex<()> = Mutex::new(());
fn with_state<T>(f: impl FnOnce(&mut State) -> T) -> T {
    f(STATE.lock().unwrap().as_mut().unwrap())
}
fn lock_delta(delta: i32) { with_state(|s| { s.locks += delta; assert!(s.locks >= 0); }); }
#[no_mangle]
unsafe extern "C" fn HvGetCell(_: ddk::PHHIVE, index: u32) -> *mut core::ffi::c_void {
    with_state(|s| {
        if s.fail_map == Some(index) { return std::ptr::null_mut(); }
        match s.cells.get_mut(&index) {
            Some(cell) => {
                *s.refs.entry(index).or_default() += 1;
                cell.as_mut_ptr().cast()
            },
            None => std::ptr::null_mut(),
        }
    })
}
#[no_mangle]
unsafe extern "C" fn HvReleaseCell(_: ddk::PHHIVE, index: u32) {
    with_state(|s| { let refs = s.refs.get_mut(&index).expect("released unmapped cell");
        *refs -= 1; assert!(*refs >= 0); });
}
#[no_mangle]
unsafe extern "system" fn HvAllocateCell(_: ddk::PHHIVE, size: u32, storage: u32, _: u32) -> u32 {
    with_state(|s| {
        s.storage = Some(storage);
        if s.fail_allocate { return u32::MAX; }
        let index = s.next | (storage << 31); s.next += 0x100;
        s.cells.insert(index, vec![0u64; (size as usize + 7) / 8].into_boxed_slice());
        index
    })
}
#[no_mangle]
unsafe extern "system" fn HvFreeCell(_: ddk::PHHIVE, index: u32) {
    with_state(|s| {
        assert_eq!(s.refs.get(&index).copied().unwrap_or(0), 0, "free while mapped");
        assert!(s.cells.remove(&index).is_some(), "free of absent cell");
    });
}
#[no_mangle]
unsafe extern "system" fn HvMarkCellDirty(_: ddk::PHHIVE, index: u32, _: u8) -> u8 {
    with_state(|s| (s.fail_dirty != Some(index)) as u8)
}
fn put<T>(index: u32, object: T) {
    let mut cell = vec![0u64; (std::mem::size_of::<T>() + 7) / 8].into_boxed_slice();
    unsafe { cell.as_mut_ptr().cast::<T>().write(object); }
    with_state(|s| { s.cells.insert(index, cell); });
}
fn node(index: u32, security: u32) { put(index, ddk::NODE { Signature: 0x6b6e, Security: security }); }
fn security(index: u32, references: u32, forward: u32, backward: u32, value: u8) {
    put(index, ddk::CM_KEY_SECURITY { Signature: 0x6b73, Reserved: 0,
        Flink: forward, Blink: backward, ReferenceCount: references,
        DescriptorLength: 20, Descriptor: [value; 20] });
}
fn read_node(index: u32) -> u32 {
    with_state(|s| unsafe { (*(s.cells[&index].as_ptr().cast::<ddk::NODE>())).Security })
}
fn read_security(index: u32) -> (u32, u32, u32, Vec<u8>) {
    with_state(|s| unsafe {
        let sk = &*s.cells[&index].as_ptr().cast::<ddk::CM_KEY_SECURITY>();
        (sk.ReferenceCount, sk.Flink, sk.Blink, sk.Descriptor.to_vec())
    })
}
fn run_test(f: impl FnOnce(&mut ddk::CMHIVE)) {
    let _guard = TEST_LOCK.lock().unwrap();
    *STATE.lock().unwrap() = Some(State { next: 0x1000, ..State::default() });
    let mut hive = ddk::CMHIVE { Hive: ddk::HHIVE { ReadOnly: 0 }, SecurityLock: 0 };
    f(&mut hive);
    with_state(|s| { assert_eq!(s.locks, 0); assert!(s.refs.values().all(|r| *r == 0)); });
}
fn persist(hive: &mut ddk::CMHIVE, index: u32, value: u8) -> i32 {
    let mut kcb = ddk::KCB { __bindgen_anon_2: ddk::KeyUnion {
        __bindgen_anon_1: ddk::KeyFields { KeyHive: &mut hive.Hive, KeyCell: index } } };
    unsafe { shipping::persist(&mut kcb, [value; 20].as_ptr(), 20) }
}

#[test]
fn survives_kcb_destruction_and_uses_key_storage_class() {
    run_test(|hive| {
        for index in [0x100, 0x8000_0100] {
            node(index, u32::MAX);
            assert_eq!(persist(hive, index, 42), 0); // temporary KCB is now gone
            let sk = read_node(index);
            assert_eq!(read_security(sk), (1, sk, sk, vec![42; 20]));
            assert_eq!(with_state(|s| s.storage), Some(index >> 31));
        }
    });
}
#[test]
fn allocation_mapping_and_dirty_failures_preserve_previous_descriptor() {
    run_test(|hive| {
        node(0x100, 0x200); security(0x200, 1, 0x200, 0x200, 7);
        for failure in 0..6 {
            with_state(|s| {
                s.fail_allocate = failure == 0;
                s.fail_map = match failure { 1 => Some(s.next), 2 => Some(0x100), 3 => Some(0x200), _ => None };
                s.fail_dirty = match failure { 4 => Some(0x100), 5 => Some(0x200), _ => None };
            });
            assert_ne!(persist(hive, 0x100, 9), 0);
            assert_eq!(read_node(0x100), 0x200);
            assert_eq!(read_security(0x200), (1, 0x200, 0x200, vec![7; 20]));
            assert_eq!(with_state(|s| s.cells.len()), 2);
        }
    });
}
#[test]
fn shared_security_is_copy_on_write() {
    run_test(|hive| {
        node(0x100, 0x300); node(0x200, 0x300); security(0x300, 2, 0x300, 0x300, 3);
        assert_eq!(persist(hive, 0x100, 8), 0);
        assert_eq!(read_node(0x200), 0x300);
        assert_eq!(read_security(0x300), (1, 0x300, 0x300, vec![3; 20]));
        assert_eq!(read_security(read_node(0x100)).3, vec![8; 20]);
    });
}
#[test]
fn replacement_unlinks_singleton_and_multi_entry_lists() {
    run_test(|hive| {
        node(0x100, 0x200); security(0x200, 1, 0x200, 0x200, 1);
        assert_eq!(persist(hive, 0x100, 2), 0);
        assert!(!with_state(|s| s.cells.contains_key(&0x200)));
        node(0x100, 0x200);
        security(0x200, 1, 0x300, 0x400, 1);
        security(0x300, 1, 0x400, 0x200, 3);
        security(0x400, 1, 0x200, 0x300, 4);
        assert_eq!(persist(hive, 0x100, 9), 0);
        assert_eq!(read_security(0x300), (1, 0x400, 0x400, vec![3; 20]));
        assert_eq!(read_security(0x400), (1, 0x300, 0x300, vec![4; 20]));
        assert!(!with_state(|s| s.cells.contains_key(&0x200)));
    });
}
#[test]
fn neighbor_failures_and_invalid_links_do_not_publish() {
    run_test(|hive| {
        node(0x100, 0x200); security(0x200, 1, 0x300, 0x300, 1);
        security(0x300, 1, 0x200, 0x200, 3);
        for dirty in [false, true] {
            with_state(|s| { s.fail_map = (!dirty).then_some(0x300); s.fail_dirty = dirty.then_some(0x300); });
            assert_ne!(persist(hive, 0x100, 9), 0);
            assert_eq!(read_node(0x100), 0x200);
            assert_eq!(read_security(0x200), (1, 0x300, 0x300, vec![1; 20]));
        }
        with_state(|s| { s.fail_map = None; s.fail_dirty = None; });
        security(0x300, 1, 0x300, 0x300, 3);
        assert_ne!(persist(hive, 0x100, 9), 0);
        assert_eq!(read_node(0x100), 0x200);
    });
}
