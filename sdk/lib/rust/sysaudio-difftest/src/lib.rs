//! Buffer semantics of the genuine C and shipping Rust GetConnectRequest.
//! Small matching fixtures replace OS layouts and allocation only. Cross-architecture
//! driver compilation/linking checks ABI separately; this is a host behavior test.
#![allow(
    non_snake_case,
    non_camel_case_types,
    non_upper_case_globals,
    unused_mut
)]

#[cfg(test)]
mod tests {
    use std::{
        alloc::{alloc, dealloc, Layout},
        cell::RefCell,
        ffi::c_void,
    };
    type NTSTATUS = i32;
    type ULONG = u32;
    type SIZE_T = usize;
    type WCHAR = u16;
    type PVOID = *mut c_void;
    type PKSPIN_CONNECT = *mut KSPIN_CONNECT;
    type PIO_STACK_LOCATION = *mut IO_STACK_LOCATION;
    type PIRP = *mut IRP;
    const NonPagedPool: u32 = 0;
    const STATUS_SUCCESS: i32 = 0;
    const STATUS_UNSUCCESSFUL: i32 = 0xc0000001u32 as i32;
    const STATUS_INSUFFICIENT_RESOURCES: i32 = 0xc000009au32 as i32;
    #[repr(C)]
    struct KSPIN_CONNECT {
        bytes: [u8; 64],
    }
    #[repr(C)]
    struct UNICODE_STRING {
        Length: u16,
        MaximumLength: u16,
        Buffer: *mut u16,
    }
    #[repr(C)]
    struct FILE_OBJECT {
        FileName: UNICODE_STRING,
    }
    #[repr(C)]
    struct IO_STACK_LOCATION {
        FileObject: *mut FILE_OBJECT,
    }
    #[repr(C)]
    struct IRP {
        Stack: PIO_STACK_LOCATION,
    }
    #[derive(Default)]
    struct Allocator {
        fail: bool,
        sizes: Vec<usize>,
        allocations: Vec<(*mut u8, Layout)>,
    }
    thread_local! { static ALLOCATOR: RefCell<Allocator> = RefCell::default(); }
    #[no_mangle]
    unsafe extern "C" fn fixture_allocate(pool: u32, size: usize) -> PVOID {
        assert_eq!(pool, NonPagedPool);
        ALLOCATOR.with(|state| {
            let mut state = state.borrow_mut();
            state.sizes.push(size);
            if state.fail {
                return std::ptr::null_mut();
            }
            let layout = Layout::from_size_align(size.max(1), 16).unwrap();
            let ptr = alloc(layout);
            assert!(!ptr.is_null());
            state.allocations.push((ptr, layout));
            ptr.cast()
        })
    }
    unsafe fn AllocateItem(pool: u32, size: SIZE_T) -> PVOID {
        fixture_allocate(pool, size)
    }
    unsafe fn IoGetCurrentIrpStackLocation(irp: PIRP) -> PIO_STACK_LOCATION {
        (*irp).Stack
    }
    unsafe fn wcslen(s: *const u16) -> usize {
        let mut n = 0;
        while *s.add(n) != 0 {
            n += 1;
        }
        n
    }
    include!(concat!(env!("OUT_DIR"), "/shipping.rs"));
    extern "C" {
        fn ref_GetConnectRequest(irp: PIRP, result: *mut PKSPIN_CONNECT) -> NTSTATUS;
    }

    unsafe fn run(
        function: unsafe extern "C" fn(PIRP, *mut PKSPIN_CONNECT) -> NTSTATUS,
        maximum: u16,
        fail: bool,
    ) -> (i32, bool, Vec<usize>, Vec<u8>) {
        ALLOCATOR.with(|s| {
            let mut s = s.borrow_mut();
            assert!(s.allocations.is_empty());
            s.fail = fail;
            s.sizes.clear();
        });
        // Backing storage extends through every declared byte, including odd byte counts.
        let mut backing: Vec<u16> = (0..32768).map(|i| (i * 37 + 19) as u16).collect();
        let mut file = FILE_OBJECT {
            FileName: UNICODE_STRING {
                Length: 0,
                MaximumLength: maximum,
                Buffer: backing.as_mut_ptr(),
            },
        };
        let mut stack = IO_STACK_LOCATION {
            FileObject: &mut file,
        };
        let mut irp = IRP { Stack: &mut stack };
        let sentinel = std::ptr::NonNull::<KSPIN_CONNECT>::dangling().as_ptr();
        let mut result = sentinel;
        let status = function(&mut irp, &mut result);
        let (sizes, bytes) = ALLOCATOR.with(|s| {
            let mut s = s.borrow_mut();
            let bytes = if status == STATUS_SUCCESS {
                std::slice::from_raw_parts(result.cast::<u8>(), *s.sizes.last().unwrap()).to_vec()
            } else {
                Vec::new()
            };
            for (ptr, layout) in s.allocations.drain(..) {
                dealloc(ptr, layout);
            }
            (s.sizes.clone(), bytes)
        });
        (status, result == sentinel, sizes, bytes)
    }

    #[test]
    fn filename_boundaries_allocation_failure_and_payload_match_original_c() {
        unsafe {
            for maximum in (0..=350).chain([1023, 4096, 65534, 65535]) {
                for fail in [false, true] {
                    let c = run(ref_GetConnectRequest, maximum, fail);
                    let rust = run(GetConnectRequest, maximum, fail);
                    assert_eq!(rust, c, "maximum={maximum}, fail={fail}");
                    if maximum < 142 {
                        assert_eq!(rust.0, STATUS_UNSUCCESSFUL);
                        assert!(rust.2.is_empty());
                    } else if fail {
                        assert_eq!(rust.0, STATUS_INSUFFICIENT_RESOURCES);
                    } else {
                        assert_eq!(rust.0, STATUS_SUCCESS);
                        assert_eq!(rust.3.len(), maximum as usize - 78);
                    }
                }
            }
        }
    }
}
