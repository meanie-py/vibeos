//! Genuine resource.cpp versus the shipping Rust owner and workers.
//! Host fixtures isolate behavior; three-target driver builds validate the NT ABI.
#![allow(
    non_snake_case,
    non_camel_case_types,
    non_upper_case_globals,
    unused_imports,
    unused_assignments,
    unused_mut
)]
extern crate self as ntkrnl_sys;
pub const STATUS_SUCCESS: i32 = 0;
pub const STATUS_INSUFFICIENT_RESOURCES: i32 = 0xc000009au32 as i32;
pub const PASSIVE_LEVEL: u8 = 0;
pub mod constants {
    pub const STATUS_INVALID_PARAMETER: i32 = 0xc000000du32 as i32;
    pub const STATUS_UNSUCCESSFUL: i32 = 0xc0000001u32 as i32;
}
pub mod mm {
    pub const NonPagedPool: u32 = 0;
}
pub mod ddk {
    pub type ULONG = u32;
    pub type POOL_TYPE = u32;
    pub type CM_RESOURCE_TYPE = i32;
    pub type NTSTATUS = i32;
    pub type PVOID = *mut core::ffi::c_void;
    pub type PCM_RESOURCE_LIST = *mut CM_RESOURCE_LIST;
    pub type PCM_PARTIAL_RESOURCE_DESCRIPTOR = *mut CM_PARTIAL_RESOURCE_DESCRIPTOR;
    pub type IID = GUID;
    #[repr(C)]
    pub struct GUID {
        pub Data1: u32,
        pub Data2: u16,
        pub Data3: u16,
        pub Data4: [u8; 8],
    }
    #[repr(C)]
    pub struct UNICODE_STRING {
        pub Length: u16,
        pub MaximumLength: u16,
        pub Buffer: *mut u16,
    }
    #[repr(C)]
    pub struct CM_PARTIAL_RESOURCE_DESCRIPTOR {
        pub Type: u8,
        pub ShareDisposition: u8,
        pub Flags: u16,
        pub Data: [u32; 3],
    }
    #[repr(C)]
    pub struct CM_PARTIAL_RESOURCE_LIST {
        pub Version: u16,
        pub Revision: u16,
        pub Count: u32,
        pub PartialDescriptors: [CM_PARTIAL_RESOURCE_DESCRIPTOR; 1],
    }
    #[repr(C)]
    pub struct CM_FULL_RESOURCE_DESCRIPTOR {
        pub InterfaceType: i32,
        pub BusNumber: u32,
        pub PartialResourceList: CM_PARTIAL_RESOURCE_LIST,
    }
    #[repr(C)]
    pub struct CM_RESOURCE_LIST {
        pub Count: u32,
        pub List: [CM_FULL_RESOURCE_DESCRIPTOR; 1],
    }
    unsafe extern "C" {
        #[link_name = "fixture_allocate"]
        pub fn ExAllocatePoolWithTag(pool: POOL_TYPE, size: usize, tag: ULONG) -> PVOID;
        pub fn fixture_free(p: PVOID, tag: ULONG, tagged: i32);
    }
    unsafe extern "system" {
        pub fn RtlAssert(exp: PVOID, file: PVOID, line: ULONG, message: *mut i8);
        pub fn KeGetCurrentIrql() -> u8;
        pub fn RtlStringFromGUID(guid: *const GUID, text: *mut UNICODE_STRING) -> NTSTATUS;
        pub fn RtlFreeUnicodeString(text: *mut UNICODE_STRING);
    }
    pub unsafe fn ExFreePool(p: PVOID) {
        fixture_free(p, 0, 0);
    }
    pub unsafe fn ExFreePoolWithTag(p: PVOID, tag: ULONG) {
        fixture_free(p, tag, 1);
    }
}
pub mod driver_ddk {
    unsafe extern "C" {
        pub fn DbgPrint(format: *const i8, ...) -> u32;
    }
}
pub mod ke {
    pub use crate::ddk::KeGetCurrentIrql;
}
pub mod rtl {
    pub unsafe fn RtlCopyMemory(
        dst: *mut core::ffi::c_void,
        src: *const core::ffi::c_void,
        n: usize,
    ) {
        core::ptr::copy_nonoverlapping(src.cast::<u8>(), dst.cast::<u8>(), n);
    }
}
pub mod inline {
    pub mod mm {
        pub unsafe fn ExAllocatePoolZero(pool: u32, n: usize, tag: u32) -> *mut core::ffi::c_void {
            crate::ddk::ExAllocatePoolWithTag(pool, n, tag)
        }
    }
}
include!(concat!(env!("OUT_DIR"), "/macros.rs"));
pub mod types {
    use crate::ddk::*;
    include!(concat!(env!("OUT_DIR"), "/interfaces.rs"));
    unsafe extern "C" {
        pub static IID_IResourceList: GUID;
    }
}
pub mod resource {
    include!(concat!(env!("OUT_DIR"), "/resource.rs"));
}
pub mod resource_owner {
    include!(concat!(env!("OUT_DIR"), "/resource_owner.rs"));
}
pub mod resource_support {
    include!(concat!(env!("OUT_DIR"), "/resource_support.rs"));
}
pub mod resource_factory {
    include!(concat!(env!("OUT_DIR"), "/resource_factory.rs"));
}
#[cfg(test)]
mod tests {
    unsafe extern "C" {
        fn fixture_exercise(reference: u32, seed: u32, failure: u32, mode: u32, result: *mut u64);
    }
    #[test]
    fn original_cpp_matches_rust_allocation_lifetime_dispatch_and_buffers() {
        for seed in 0..36 {
            for failure in 0..5 {
                for mode in 0..6 {
                    let (mut original, mut rust) = ([0u64; 7], [0u64; 7]);
                    unsafe {
                        fixture_exercise(1, seed, failure, mode, original.as_mut_ptr());
                        fixture_exercise(0, seed, failure, mode, rust.as_mut_ptr());
                    }
                    assert_eq!(original, rust, "seed={seed} fail_at={failure} mode={mode}");
                    assert_eq!(rust[3], 0, "live allocations remain");
                }
            }
        }
        // One baseline reference keeps the object alive while both implementations
        // exercise their actual atomic AddRef/Release methods concurrently.
        for reference in [true, false] {
            unsafe extern "C" {
                fn fixture_reset(failure: u32);
                fn fixture_stats(out: *mut u64);
                fn ref_PcNewResourceList(
                    out: *mut crate::types::PRESOURCELIST,
                    outer: crate::types::PUNKNOWN,
                    pool: u32,
                    t: crate::ddk::PCM_RESOURCE_LIST,
                    u: crate::ddk::PCM_RESOURCE_LIST,
                ) -> i32;
            }
            unsafe {
                fixture_reset(0);
            }
            let mut object = core::ptr::null_mut();
            let status = unsafe {
                if reference {
                    ref_PcNewResourceList(
                        &raw mut object,
                        core::ptr::null_mut(),
                        0,
                        core::ptr::null_mut(),
                        core::ptr::null_mut(),
                    )
                } else {
                    crate::resource_factory::PcNewResourceList(
                        &raw mut object,
                        core::ptr::null_mut(),
                        0,
                        core::ptr::null_mut(),
                        core::ptr::null_mut(),
                    )
                }
            };
            assert_eq!(status, 0);
            let address = object as usize;
            let threads: Vec<_> = (0..8)
                .map(|_| {
                    std::thread::spawn(move || {
                        let object = address as *mut crate::types::IResourceList;
                        for _ in 0..10000 {
                            unsafe {
                                ((*(*object).lpVtbl).AddRef.unwrap())(object);
                                ((*(*object).lpVtbl).Release.unwrap())(object);
                            }
                        }
                    })
                })
                .collect();
            for thread in threads {
                thread.join().unwrap();
            }
            unsafe {
                assert_eq!(((*(*object).lpVtbl).AddRef.unwrap())(object), 2);
                assert_eq!(((*(*object).lpVtbl).Release.unwrap())(object), 1);
                assert_eq!(((*(*object).lpVtbl).Release.unwrap())(object), 0);
                let mut stats = [0u64; 6];
                fixture_stats(stats.as_mut_ptr());
                assert_eq!(&stats[..4], &[1, 1, 0, 0]);
            }
        }
    }
}
