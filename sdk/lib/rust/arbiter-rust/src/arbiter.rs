use ntkrnl_sys::globals::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::ex::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::ddk::{CM_RESOURCE_TYPE, KEVENT, PKEVENT, PRKEVENT, KeInitializeEvent, PARBITER_ALLOCATION_STATE, PARBITER_INSTANCE, ARBITER_ORDERING_LIST, PARBITER_ORDERING_LIST, PARB_TRANSLATE_ORDERING, PDEVICE_OBJECT, RTL_RANGE_LIST, PRTL_RANGE_LIST, RtlInitializeRangeList, _EVENT_TYPE_SynchronizationEvent, UINT64, PLIST_ENTRY};
use core::ffi::c_void;

extern "C" {
    fn DbgPrint(Format: *const i8, ...) -> u32;
}

extern "system" {
    fn KeGetCurrentIrql() -> u8;
    fn RtlAssert( VoidExpression: *mut core::ffi::c_void, VoidFileName: *mut core::ffi::c_void, LineNumber: u32, VoidMessage: *mut core::ffi::c_char, );
}

// Default-callback installer: the PARB_* vtable fields are NTAPI fn-pointer
// Options (extern "stdcall" on i386, "C" on 64-bit). Our Arb* defaults are
// extern "system" (= same ABI per arch) but a nominally-distinct fn-pointer type
// on i386, so bridge via the function address (usize). Mirrors the C
// `if (!Arbiter->X) Arbiter->X = ArbX;` defaulting.
macro_rules! arb_default {
    ($arb:expr, $field:ident, $f:expr) => {
        if (*$arb).$field.is_none() {
            (*$arb).$field = ::core::mem::transmute::<usize, _>($f as usize);
        }
    };
}

// Forward declarations for external symbols used in this function
const APC_LEVEL: i32 = 1;
type PCSTR = *const i8;
type CHAR = i8;
const STATUS_NOT_IMPLEMENTED: NTSTATUS = 0xC0000002u32 as i32;

// arbiter.h: 4-char tags (little-endian u32 of the C char-literal).
const ARBITER_SIGNATURE: u32 = 0x73627241; // 'sbrA'
const TAG_ARBITER: u32 = 0x4D627241; // 'MbrA'
const TAG_ARB_ALLOCATION: u32 = 0x41627241; // 'AbrA'
const TAG_ARB_RANGE: u32 = 0x52627241; // 'RbrA'

#[no_mangle]
pub unsafe extern "system" fn ArbTestAllocation(
    Arbiter: PARBITER_INSTANCE,
    ArbitrationList: PLIST_ENTRY,
) -> NTSTATUS {
    // PAGED_CODE() macro expansion
    if KeGetCurrentIrql() as i32 > APC_LEVEL as i32 {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0".as_ptr() as *const i8,
            KeGetCurrentIrql() as i32,
        );
        // The original C code's debug assertion (always true condition retained)
        if 0i32 == 0 {
            RtlAssert(
                b"0\0".as_ptr() as *mut core::ffi::c_void,
                b"sdk/lib/drivers/arbiter/arbiter.c\0".as_ptr()
                    as *mut core::ffi::c_void,
                30u32,
                core::ptr::null_mut::<i8>(),
            );
        }
    }

    // UNIMPLEMENTED macro expansion
    DbgPrint(
        b"WARNING:  %s at %s:%d is unimplemented!()\n\0".as_ptr() as *const i8,
        b"ArbTestAllocation\0".as_ptr() as *const i8,
        b"sdk/lib/drivers/arbiter/arbiter.c\0".as_ptr()
            as *const i8,
        32i32,
    );

    return STATUS_NOT_IMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn ArbRetestAllocation(
    mut Arbiter: PARBITER_INSTANCE,
    mut ArbitrationList: PLIST_ENTRY,
) -> NTSTATUS {
    const APC_LEVEL: i32 = 1;

    if (KeGetCurrentIrql() as i32) > APC_LEVEL {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0" as *const u8 as *const i8,
            KeGetCurrentIrql() as i32,
        );
        if 0i32 == 0i32 {
            RtlAssert(
                b"0\0" as *const u8 as *const i8 as *mut core::ffi::c_void,
                b"sdk/lib/drivers/arbiter/arbiter.c\0"
                    as *const u8 as *const i8 as *mut core::ffi::c_void,
                43u32,
                core::ptr::null_mut::<i8>(),
            );
        } else {
        }
    }
    DbgPrint(
        b"WARNING:  %s at %s:%d is unimplemented!()\n\0" as *const u8 as *const i8,
        b"ArbRetestAllocation\0" as *const u8 as *const i8,
        b"sdk/lib/drivers/arbiter/arbiter.c\0" as *const u8 as *const i8,
        45i32,
    );
    return STATUS_NOT_IMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn ArbCommitAllocation(mut Arbiter: PARBITER_INSTANCE) -> NTSTATUS {
    if KeGetCurrentIrql() as ::core::ffi::c_int > APC_LEVEL as ::core::ffi::c_int {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0".as_ptr() as *const i8,
            KeGetCurrentIrql() as ::core::ffi::c_int,
        );
        if 0 as ::core::ffi::c_int == 0 {
            RtlAssert(
                b"0\0".as_ptr() as *const i8 as *mut core::ffi::c_void,
                b"sdk/lib/drivers/arbiter/arbiter.c\0"
                    .as_ptr() as *const i8 as *mut core::ffi::c_void,
                55u32,
                core::ptr::null_mut::<i8>(),
            );
        } else {
        };
    }
    DbgPrint(
        b"WARNING:  %s at %s:%d is unimplemented!()\n\0".as_ptr() as *const i8,
        b"ArbCommitAllocation\0".as_ptr() as *const i8,
        b"sdk/lib/drivers/arbiter/arbiter.c\0"
            .as_ptr() as *const i8,
        57i32,
    );
    return STATUS_NOT_IMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn ArbRollbackAllocation(Arbiter: PARBITER_INSTANCE) -> NTSTATUS {
    const APC_LEVEL: i32 = 1;

    if KeGetCurrentIrql() as i32 > APC_LEVEL {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0".as_ptr() as *const i8,
            KeGetCurrentIrql() as i32,
        );
        if 0 == 0 {
            RtlAssert(
                b"0\0".as_ptr() as *const i8 as *mut core::ffi::c_void,
                b"sdk/lib/drivers/arbiter/arbiter.c\0"
                    .as_ptr() as *const i8 as *mut core::ffi::c_void,
                67u32,
                core::ptr::null_mut::<i8>(),
            );
        } else {
        };
    }
    DbgPrint(
        b"WARNING:  %s at %s:%d is unimplemented!()\n\0".as_ptr() as *const i8,
        b"ArbRollbackAllocation\0".as_ptr() as *const i8,
        b"sdk/lib/drivers/arbiter/arbiter.c\0"
            .as_ptr() as *const i8,
        69i32,
    );
    return STATUS_NOT_IMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn ArbAddReserved(_Arbiter: PARBITER_INSTANCE) -> NTSTATUS {
    if KeGetCurrentIrql() as i32 > APC_LEVEL {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0".as_ptr() as PCSTR,
            KeGetCurrentIrql() as i32,
        );
        if 0i32 == 0 {
            RtlAssert(
                b"0\0".as_ptr() as *const i8 as PVOID,
                b"sdk/lib/drivers/arbiter/arbiter.c\0".as_ptr() as *const i8 as PVOID,
                80u32,
                core::ptr::null_mut(),
            );
        } else {
        }
    }
    DbgPrint(
        b"WARNING:  %s at %s:%d is unimplemented!()\n\0".as_ptr() as PCSTR,
        b"ArbAddReserved\0".as_ptr() as *const i8,
        b"sdk/lib/drivers/arbiter/arbiter.c\0".as_ptr() as *const i8,
        82i32,
    );
    return STATUS_NOT_IMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn ArbPreprocessEntry(
    Arbiter: PARBITER_INSTANCE,
    ArbState: PARBITER_ALLOCATION_STATE,
) -> NTSTATUS {
    if (KeGetCurrentIrql() as i32) > (APC_LEVEL as i32) {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0".as_ptr() as *const i8,
            KeGetCurrentIrql() as i32,
        );
        if 0i32 == 0i32 {
            RtlAssert(
                b"0\0".as_ptr() as *mut core::ffi::c_void,
                b"sdk/lib/drivers/arbiter/arbiter.c\0".as_ptr()
                    as *mut core::ffi::c_void,
                93,
                core::ptr::null_mut::<i8>(),
            );
        }
    }
    return STATUS_SUCCESS;
}

#[no_mangle]
pub unsafe extern "system" fn ArbAllocateEntry(
    Arbiter: PARBITER_INSTANCE,
    ArbState: PARBITER_ALLOCATION_STATE,
) -> NTSTATUS {
    if (KeGetCurrentIrql() as i32) > APC_LEVEL {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0"
                .as_ptr() as *const i8 as *const i8,
            KeGetCurrentIrql() as i32,
        );
        if (0 as i32) == 0 {
            RtlAssert(
                b"0\0".as_ptr() as *const i8 as *const i8 as *mut core::ffi::c_void,
                b"sdk/lib/drivers/arbiter/arbiter.c\0"
                    .as_ptr() as *const i8 as *const i8 as *mut core::ffi::c_void,
                105u32,
                core::ptr::null_mut::<i8>(),
            );
        }
    }
    DbgPrint(
        b"WARNING:  %s at %s:%d is unimplemented!()\n\0"
            .as_ptr() as *const i8 as *const i8,
        b"ArbAllocateEntry\0".as_ptr() as *const i8 as *const i8,
        b"sdk/lib/drivers/arbiter/arbiter.c\0"
            .as_ptr() as *const i8 as *const i8,
        107i32,
    );
    return STATUS_NOT_IMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn ArbGetNextAllocationRange(
    Arbiter: PARBITER_INSTANCE,
    ArbState: PARBITER_ALLOCATION_STATE,
) -> i32 {
    unsafe {
        if KeGetCurrentIrql() as i32 > APC_LEVEL as i32 {
            DbgPrint(
                b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0" as *const u8 as *const i8,
                KeGetCurrentIrql() as i32,
            );
            if 0 as i32 == 0 {
                RtlAssert(
                    b"0\0" as *const u8 as *mut ::core::ffi::c_void,
                    b"sdk/lib/drivers/arbiter/arbiter.c\0"
                        as *const u8 as *mut ::core::ffi::c_void,
                    118,
                    core::ptr::null_mut::<i8>(),
                );
            } else {
            }
        }
        DbgPrint(
            b"WARNING:  %s at %s:%d is unimplemented!()\n\0" as *const u8 as *const i8,
            b"ArbGetNextAllocationRange\0" as *const u8 as *const i8,
            b"sdk/lib/drivers/arbiter/arbiter.c\0"
                as *const u8 as *const i8,
            120 as i32,
        );
        return 0;
    }
}

#[no_mangle]
pub unsafe extern "system" fn ArbFindSuitableRange(
    mut Arbiter: PARBITER_INSTANCE,
    mut ArbState: PARBITER_ALLOCATION_STATE,
) -> BOOLEAN {
    if KeGetCurrentIrql() as i32 > APC_LEVEL {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0".as_ptr() as *const i8,
            KeGetCurrentIrql() as i32,
        );
        if 0 as i32 == 0 {
            RtlAssert(
                b"0\0".as_ptr() as *const i8 as *mut core::ffi::c_void,
                b"sdk/lib/drivers/arbiter/arbiter.c\0".as_ptr() as *const i8 as *mut core::ffi::c_void,
                131 as u32,
                core::ptr::null_mut::<i8>(),
            );
        } else {
            // empty else
        }
    }
    DbgPrint(
        b"WARNING:  %s at %s:%d is unimplemented!()\n\0".as_ptr() as *const i8,
        b"ArbFindSuitableRange\0".as_ptr() as *const i8,
        b"sdk/lib/drivers/arbiter/arbiter.c\0".as_ptr() as *const i8,
        133 as i32,
    );
    return 0 as BOOLEAN;
}

#[no_mangle]
pub unsafe extern "system" fn ArbAddAllocation(
    mut Arbiter: PARBITER_INSTANCE,
    mut ArbState: PARBITER_ALLOCATION_STATE,
) {
    if KeGetCurrentIrql() as i32 > APC_LEVEL {
        DbgPrint(
            (b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0".as_ptr() as *const i8),
            KeGetCurrentIrql() as i32,
        );
        if 0i32 == 0 {
            RtlAssert(
                (b"0\0".as_ptr() as *const i8) as *mut core::ffi::c_void, // PVOID from string
                b"sdk/lib/drivers/arbiter/arbiter.c\0".as_ptr() as *mut core::ffi::c_void,
                144u32,
                core::ptr::null_mut::<i8>(),
            );
        }
    }
    DbgPrint(
        (b"WARNING:  %s at %s:%d is unimplemented!()\n\0".as_ptr() as *const i8),
        (b"ArbAddAllocation\0".as_ptr() as *const i8),
        (b"sdk/lib/drivers/arbiter/arbiter.c\0".as_ptr() as *const i8),
        146i32,
    );
}

#[no_mangle]
pub unsafe extern "system" fn ArbBacktrackAllocation(
    Arbiter: PARBITER_INSTANCE,
    ArbState: PARBITER_ALLOCATION_STATE,
) {
    let irql = KeGetCurrentIrql() as i32;
    if irql > APC_LEVEL as i32 {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0".as_ptr() as *const i8,
            irql,
        );
        if 0 as i32 == 0 {
            RtlAssert(
                b"0\0".as_ptr() as *const i8 as *mut core::ffi::c_void,
                b"sdk/lib/drivers/arbiter/arbiter.c\0".as_ptr() as *const i8 as *mut core::ffi::c_void,
                156 as u32,
                core::ptr::null_mut::<i8>(),
            );
        }
    }
    DbgPrint(
        b"WARNING:  %s at %s:%d is unimplemented!()\n\0".as_ptr() as *const i8,
        b"ArbBacktrackAllocation\0".as_ptr() as *const i8,
        b"sdk/lib/drivers/arbiter/arbiter.c\0".as_ptr() as *const i8,
        158 as i32,
    );
}
#[no_mangle] #[no_mangle]
#[no_mangle] pub unsafe extern "system" fn ArbOverrideConflict(Arbiter: PARBITER_INSTANCE) -> NTSTATUS {
    const APC_LEVEL: i32 = 1;
    if KeGetCurrentIrql() as i32 > APC_LEVEL {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0" as *const u8 as *const i8,
            KeGetCurrentIrql() as i32,
        );
        if 0 as i32 == 0 {
            RtlAssert(
                b"0\0".as_ptr() as *const i8 as *mut core::ffi::c_void,
                b"sdk/lib/drivers/arbiter/arbiter.c\0".as_ptr() as *const i8 as *mut core::ffi::c_void,
                168 as u32,
                core::ptr::null_mut::<i8>(),
            );
        }
    }
    DbgPrint(
        b"WARNING:  %s at %s:%d is unimplemented!()\n\0" as *const u8 as *const i8,
        b"ArbOverrideConflict\0".as_ptr() as *const i8,
        b"sdk/lib/drivers/arbiter/arbiter.c\0".as_ptr() as *const i8,
        170 as i32,
    );
    return STATUS_NOT_IMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn ArbBootAllocation(
    Arbiter: PARBITER_INSTANCE,
    ArbitrationList: PLIST_ENTRY,
) -> NTSTATUS {
    if (KeGetCurrentIrql() as i32) > (APC_LEVEL as i32) {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0".as_ptr() as *const i8,
            KeGetCurrentIrql() as i32,
        );
        if 0i32 == 0 {
            RtlAssert(
                (b"0\0".as_ptr() as *const i8) as *mut core::ffi::c_void,
                (b"sdk/lib/drivers/arbiter/arbiter.c\0".as_ptr() as *const i8) as *mut core::ffi::c_void,
                181u32,
                core::ptr::null_mut::<i8>(),
            );
        } else {
        }
    }
    DbgPrint(
        b"WARNING:  %s at %s:%d is unimplemented!()\n\0".as_ptr() as *const i8,
        b"ArbBootAllocation\0".as_ptr() as *const i8,
        b"sdk/lib/drivers/arbiter/arbiter.c\0".as_ptr() as *const i8,
        183i32,
    );
    return STATUS_NOT_IMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn ArbQueryConflict(mut Arbiter: PARBITER_INSTANCE) -> NTSTATUS {
    const APC_LEVEL: u8 = 1;
    if KeGetCurrentIrql() > APC_LEVEL {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0" as *const u8 as *const i8,
            KeGetCurrentIrql() as i32,
        );
        if 0 as i32 == 0 {
            RtlAssert(
                b"0\0" as *const u8 as *const core::ffi::c_char as *mut core::ffi::c_void,
                b"sdk/lib/drivers/arbiter/arbiter.c\0"
                    as *const u8 as *const core::ffi::c_char as *mut core::ffi::c_void,
                194 as u32,
                core::ptr::null_mut::<core::ffi::c_char>(),
            );
        } else {
        }
    }
    DbgPrint(
        b"WARNING:  %s at %s:%d is unimplemented!()\n\0" as *const u8 as *const i8,
        b"ArbQueryConflict\0" as *const u8 as *const core::ffi::c_char,
        b"sdk/lib/drivers/arbiter/arbiter.c\0"
            as *const u8 as *const core::ffi::c_char,
        196 as i32,
    );
    return ntkrnl_sys::base::STATUS_NOT_IMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn ArbStartArbiter(Arbiter: PARBITER_INSTANCE) -> NTSTATUS {
    if KeGetCurrentIrql() as core::ffi::c_int > APC_LEVEL as core::ffi::c_int {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0".as_ptr() as *const i8,
            KeGetCurrentIrql() as core::ffi::c_int,
        );
        if 0 as core::ffi::c_int == 0 {
            RtlAssert(
                b"0\0".as_ptr() as *const i8 as *mut core::ffi::c_void,
                b"sdk/lib/drivers/arbiter/arbiter.c\0"
                    .as_ptr() as *const i8 as *mut core::ffi::c_void,
                207 as u32,
                core::ptr::null_mut::<i8>(),
            );
        }
    }
    DbgPrint(
        b"WARNING:  %s at %s:%d is unimplemented!()\n\0".as_ptr() as *const i8,
        b"ArbStartArbiter\0".as_ptr() as *const i8,
        b"sdk/lib/drivers/arbiter/arbiter.c\0"
            .as_ptr() as *const i8,
        209 as core::ffi::c_int,
    );
    return STATUS_NOT_IMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn ArbAddOrdering(
    mut OrderList: PARBITER_ORDERING_LIST,
    mut MinimumAddress: u64,
    mut MaximumAddress: u64,
) -> i32 {
    const APC_LEVEL: i32 = 1;

    if KeGetCurrentIrql() as i32 > APC_LEVEL {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0".as_ptr() as *const i8,
            KeGetCurrentIrql() as i32,
        );
        if 0i32 == 0 {
            RtlAssert(
                b"0\0".as_ptr() as *const i8 as *mut core::ffi::c_void,
                b"sdk/lib/drivers/arbiter/arbiter.c\0".as_ptr()
                    as *const i8 as *mut core::ffi::c_void,
                221u32,
                core::ptr::null_mut::<core::ffi::c_char>(),
            );
        } else {
        };
    }
    DbgPrint(
        b"WARNING:  %s at %s:%d is unimplemented!()\n\0".as_ptr() as *const i8,
        b"ArbAddOrdering\0".as_ptr() as *const i8,
        b"sdk/lib/drivers/arbiter/arbiter.c\0".as_ptr()
            as *const i8,
        223i32,
    );
    return STATUS_NOT_IMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn ArbPruneOrdering(
    OrderingList: PARBITER_ORDERING_LIST,
    MinimumAddress: UINT64,
    MaximumAddress: UINT64,
) -> NTSTATUS {
    if KeGetCurrentIrql() as i32 > APC_LEVEL {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0".as_ptr() as *const i8,
            KeGetCurrentIrql() as i32,
        );
        if 0i32 == 0 {
            RtlAssert(
                b"0\0".as_ptr() as *const i8 as *mut core::ffi::c_void,
                b"sdk/lib/drivers/arbiter/arbiter.c\0"
                    .as_ptr() as *const i8 as *mut core::ffi::c_void,
                235u32,
                core::ptr::null_mut::<i8>(),
            );
        } else {
        }
    }
    DbgPrint(
        b"WARNING:  %s at %s:%d is unimplemented!()\n\0".as_ptr() as *const i8,
        b"ArbPruneOrdering\0".as_ptr() as *const i8,
        b"sdk/lib/drivers/arbiter/arbiter.c\0"
            .as_ptr() as *const i8,
        237i32,
    );
    return STATUS_NOT_IMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn ArbInitializeOrderingList(
    OrderList: *mut ARBITER_ORDERING_LIST,
) -> NTSTATUS {
    // PAGED_CODE() macro expansion
    if (KeGetCurrentIrql() as i32) > APC_LEVEL {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0".as_ptr() as *const i8,
            KeGetCurrentIrql() as i32,
        );
        if 0 == 0 {
            RtlAssert(
                b"0\0".as_ptr() as *const i8 as *mut core::ffi::c_void,
                b"sdk/lib/drivers/arbiter/arbiter.c\0"
                    .as_ptr() as *const i8 as *mut core::ffi::c_void,
                247,
                core::ptr::null_mut::<i8>(),
            );
        }
    }

    // UNIMPLEMENTED macro expansion
    DbgPrint(
        b"WARNING:  %s at %s:%d is unimplemented!()\n\0".as_ptr() as *const i8,
        b"ArbInitializeOrderingList\0".as_ptr() as *const i8,
        b"sdk/lib/drivers/arbiter/arbiter.c\0".as_ptr() as *const i8,
        249i32,
    );

    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn ArbFreeOrderingList(
    OrderList: *mut core::ffi::c_void,
) {
    if KeGetCurrentIrql() as i32 > APC_LEVEL as i32 {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0".as_ptr() as *const i8,
            KeGetCurrentIrql() as i32,
        );
        if 0i32 == 0 {
            RtlAssert(
                b"0\0".as_ptr() as *mut core::ffi::c_void,
                b"sdk/lib/drivers/arbiter/arbiter.c\0"
                    .as_ptr()
                    as *mut core::ffi::c_void,
                259u32,
                core::ptr::null_mut::<i8>(),
            );
        } else {
        };
    }
    DbgPrint(
        b"WARNING:  %s at %s:%d is unimplemented!()\n\0".as_ptr() as *const i8,
        b"ArbFreeOrderingList\0".as_ptr() as *const i8,
        b"sdk/lib/drivers/arbiter/arbiter.c\0"
            .as_ptr()
            as *const i8,
        261i32,
    );
}

#[no_mangle]
pub unsafe extern "system" fn ArbBuildAssignmentOrdering(
    ArbInstance: PARBITER_INSTANCE,
    OrderName: PCWSTR,
    ReservedOrderName: PCWSTR,
    TranslateOrderingFunction: PARB_TRANSLATE_ORDERING,
) -> NTSTATUS {
    if KeGetCurrentIrql() as i32 > APC_LEVEL {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0".as_ptr() as *const i8,
            KeGetCurrentIrql() as i32,
        );
        if 0i32 == 0 {
            RtlAssert(
                b"0\0".as_ptr() as *const i8 as *mut core::ffi::c_void,
                b"sdk/lib/drivers/arbiter/arbiter.c\0".as_ptr() as *const i8 as *mut core::ffi::c_void,
                273u32,
                core::ptr::null_mut::<i8>(),
            );
        }
    }
    DbgPrint(
        b"WARNING:  %s at %s:%d is unimplemented!()\n\0".as_ptr() as *const i8,
        b"ArbBuildAssignmentOrdering\0".as_ptr() as *const i8,
        b"sdk/lib/drivers/arbiter/arbiter.c\0".as_ptr() as *const i8,
        275i32,
    );
    return STATUS_SUCCESS;
}

#[no_mangle]
pub unsafe extern "system" fn ArbInitializeArbiterInstance(
    mut Arbiter: PARBITER_INSTANCE,
    mut BusDeviceObject: PDEVICE_OBJECT,
    mut ResourceType: CM_RESOURCE_TYPE,
    mut ArbiterName: PCWSTR,
    mut OrderName: PCWSTR,
    mut TranslateOrderingFunction: PARB_TRANSLATE_ORDERING,
) -> NTSTATUS {
    let mut Status: NTSTATUS;

    unsafe {
        // PAGED_CODE
        if KeGetCurrentIrql() as i32 > 1 /* APC_LEVEL */ {
            DbgPrint(
                b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0" as *const u8
                    as *const i8,
                KeGetCurrentIrql() as i32,
            );
            if 0 == 0 {
                RtlAssert(
                    b"0\0" as *const u8 as *const i8 as *mut core::ffi::c_void,
                    b"sdk/lib/drivers/arbiter/arbiter.c\0"
                        as *const u8 as *const i8 as *mut core::ffi::c_void,
                    292,
                    core::ptr::null_mut::<i8>(),
                );
            }
        }

        // ASSERTs
        if (*Arbiter).UnpackRequirement.is_none() {
            RtlAssert(
                b"Arbiter->UnpackRequirement != NULL\0" as *const u8 as *const i8
                    as *mut core::ffi::c_void,
                b"sdk/lib/drivers/arbiter/arbiter.c\0"
                    as *const u8 as *const i8 as *mut core::ffi::c_void,
                296,
                core::ptr::null_mut::<i8>(),
            );
        }
        if (*Arbiter).PackResource.is_none() {
            RtlAssert(
                b"Arbiter->PackResource != NULL\0" as *const u8 as *const i8
                    as *mut core::ffi::c_void,
                b"sdk/lib/drivers/arbiter/arbiter.c\0"
                    as *const u8 as *const i8 as *mut core::ffi::c_void,
                297,
                core::ptr::null_mut::<i8>(),
            );
        }
        if (*Arbiter).UnpackResource.is_none() {
            RtlAssert(
                b"Arbiter->UnpackResource != NULL\0" as *const u8 as *const i8
                    as *mut core::ffi::c_void,
                b"sdk/lib/drivers/arbiter/arbiter.c\0"
                    as *const u8 as *const i8 as *mut core::ffi::c_void,
                298,
                core::ptr::null_mut::<i8>(),
            );
        }
        if !(*Arbiter).MutexEvent.is_null() {
            RtlAssert(
                b"Arbiter->MutexEvent == NULL\0" as *const u8 as *const i8
                    as *mut core::ffi::c_void,
                b"sdk/lib/drivers/arbiter/arbiter.c\0"
                    as *const u8 as *const i8 as *mut core::ffi::c_void,
                299,
                core::ptr::null_mut::<i8>(),
            );
        }
        if !(*Arbiter).Allocation.is_null() {
            RtlAssert(
                b"Arbiter->Allocation == NULL\0" as *const u8 as *const i8
                    as *mut core::ffi::c_void,
                b"sdk/lib/drivers/arbiter/arbiter.c\0"
                    as *const u8 as *const i8 as *mut core::ffi::c_void,
                300,
                core::ptr::null_mut::<i8>(),
            );
        }
        if !(*Arbiter).PossibleAllocation.is_null() {
            RtlAssert(
                b"Arbiter->PossibleAllocation == NULL\0" as *const u8 as *const i8
                    as *mut core::ffi::c_void,
                b"sdk/lib/drivers/arbiter/arbiter.c\0"
                    as *const u8 as *const i8 as *mut core::ffi::c_void,
                301,
                core::ptr::null_mut::<i8>(),
            );
        }
        if !(*Arbiter).AllocationStack.is_null() {
            RtlAssert(
                b"Arbiter->AllocationStack == NULL\0" as *const u8 as *const i8
                    as *mut core::ffi::c_void,
                b"sdk/lib/drivers/arbiter/arbiter.c\0"
                    as *const u8 as *const i8 as *mut core::ffi::c_void,
                302,
                core::ptr::null_mut::<i8>(),
            );
        }

        (*Arbiter).Signature = ARBITER_SIGNATURE;
        (*Arbiter).BusDeviceObject = BusDeviceObject;

        (*Arbiter).MutexEvent = ExAllocatePoolWithTag(
            NonPagedPool,
            core::mem::size_of::<KEVENT>() as SIZE_T,
            TAG_ARBITER as u32,
        ) as PKEVENT;
        if (*Arbiter).MutexEvent.is_null() {
            if DbgPrint(
                b"(%s:%d) ArbInitializeArbiterInstance: STATUS_INSUFFICIENT_RESOURCES\n\0"
                    as *const u8 as *const i8,
                b"sdk/lib/drivers/arbiter/arbiter.c\0"
                    as *const u8 as *const i8,
                310,
            ) != 0
            {
                DbgPrint(
                    b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as *const i8,
                    b"sdk/lib/drivers/arbiter/arbiter.c\0"
                        as *const u8 as *const i8,
                    310,
                );
            }
            return STATUS_INSUFFICIENT_RESOURCES;
        }

        KeInitializeEvent(
            (*Arbiter).MutexEvent as PRKEVENT,
            _EVENT_TYPE_SynchronizationEvent,
            1, // TRUE
        );

        (*Arbiter).AllocationStack = ExAllocatePoolWithTag(
            PagedPool,
            PAGE_SIZE as SIZE_T,
            TAG_ARB_ALLOCATION as u32,
        ) as PARBITER_ALLOCATION_STATE;
        if (*Arbiter).AllocationStack.is_null() {
            if DbgPrint(
                b"(%s:%d) ArbInitializeArbiterInstance: STATUS_INSUFFICIENT_RESOURCES\n\0"
                    as *const u8 as *const i8,
                b"sdk/lib/drivers/arbiter/arbiter.c\0"
                    as *const u8 as *const i8,
                319,
            ) != 0
            {
                DbgPrint(
                    b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as *const i8,
                    b"sdk/lib/drivers/arbiter/arbiter.c\0"
                        as *const u8 as *const i8,
                    319,
                );
            }
            ExFreePoolWithTag((*Arbiter).MutexEvent as *mut core::ffi::c_void, TAG_ARBITER as u32);
            return STATUS_INSUFFICIENT_RESOURCES;
        }

        (*Arbiter).AllocationStackMaxSize = PAGE_SIZE as u32;

        (*Arbiter).Allocation = ExAllocatePoolWithTag(
            PagedPool,
            core::mem::size_of::<RTL_RANGE_LIST>() as SIZE_T,
            TAG_ARB_RANGE as u32,
        ) as PRTL_RANGE_LIST;
        if (*Arbiter).Allocation.is_null() {
            if DbgPrint(
                b"(%s:%d) ArbInitializeArbiterInstance: STATUS_INSUFFICIENT_RESOURCES\n\0"
                    as *const u8 as *const i8,
                b"sdk/lib/drivers/arbiter/arbiter.c\0"
                    as *const u8 as *const i8,
                329,
            ) != 0
            {
                DbgPrint(
                    b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as *const i8,
                    b"sdk/lib/drivers/arbiter/arbiter.c\0"
                        as *const u8 as *const i8,
                    329,
                );
            }
            ExFreePoolWithTag(
                (*Arbiter).AllocationStack as *mut core::ffi::c_void,
                TAG_ARB_ALLOCATION as u32,
            );
            ExFreePoolWithTag((*Arbiter).MutexEvent as *mut core::ffi::c_void, TAG_ARBITER as u32);
            return STATUS_INSUFFICIENT_RESOURCES;
        }

        (*Arbiter).PossibleAllocation = ExAllocatePoolWithTag(
            PagedPool,
            core::mem::size_of::<RTL_RANGE_LIST>() as SIZE_T,
            TAG_ARB_RANGE as u32,
        ) as PRTL_RANGE_LIST;
        if (*Arbiter).PossibleAllocation.is_null() {
            if DbgPrint(
                b"(%s:%d) ArbInitializeArbiterInstance: STATUS_INSUFFICIENT_RESOURCES\n\0"
                    as *const u8 as *const i8,
                b"sdk/lib/drivers/arbiter/arbiter.c\0"
                    as *const u8 as *const i8,
                338,
            ) != 0
            {
                DbgPrint(
                    b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as *const i8,
                    b"sdk/lib/drivers/arbiter/arbiter.c\0"
                        as *const u8 as *const i8,
                    338,
                );
            }
            ExFreePoolWithTag(
                (*Arbiter).Allocation as *mut core::ffi::c_void,
                TAG_ARB_RANGE as u32,
            );
            ExFreePoolWithTag(
                (*Arbiter).AllocationStack as *mut core::ffi::c_void,
                TAG_ARB_ALLOCATION as u32,
            );
            ExFreePoolWithTag((*Arbiter).MutexEvent as *mut core::ffi::c_void, TAG_ARBITER as u32);
            return STATUS_INSUFFICIENT_RESOURCES;
        }

        RtlInitializeRangeList((*Arbiter).Allocation);
        RtlInitializeRangeList((*Arbiter).PossibleAllocation);

        (*Arbiter).Name = ArbiterName;
        (*Arbiter).ResourceType = ResourceType;
        (*Arbiter).TransactionInProgress = 0; // FALSE as BOOLEAN

        arb_default!(Arbiter, TestAllocation, ArbTestAllocation);
        arb_default!(Arbiter, RetestAllocation, ArbRetestAllocation);
        arb_default!(Arbiter, CommitAllocation, ArbCommitAllocation);
        arb_default!(Arbiter, RollbackAllocation, ArbRollbackAllocation);
        arb_default!(Arbiter, AddReserved, ArbAddReserved);
        arb_default!(Arbiter, PreprocessEntry, ArbPreprocessEntry);
        arb_default!(Arbiter, AllocateEntry, ArbAllocateEntry);
        arb_default!(Arbiter, GetNextAllocationRange, ArbGetNextAllocationRange);
        arb_default!(Arbiter, FindSuitableRange, ArbFindSuitableRange);
        arb_default!(Arbiter, AddAllocation, ArbAddAllocation);
        arb_default!(Arbiter, BacktrackAllocation, ArbBacktrackAllocation);
        arb_default!(Arbiter, OverrideConflict, ArbOverrideConflict);
        arb_default!(Arbiter, BootAllocation, ArbBootAllocation);
        arb_default!(Arbiter, QueryConflict, ArbQueryConflict);
        arb_default!(Arbiter, StartArbiter, ArbStartArbiter);

        Status = ArbBuildAssignmentOrdering(
            Arbiter,
            OrderName,
            OrderName,
            TranslateOrderingFunction,
        );
        if NT_SUCCESS(Status) {
            return STATUS_SUCCESS;
        }

        if DbgPrint(
            b"(%s:%d) ArbInitializeArbiterInstance: Status %X\n\0" as *const u8 as *const i8,
            b"sdk/lib/drivers/arbiter/arbiter.c\0"
                as *const u8 as *const i8,
            403,
            Status,
        ) != 0
        {
            DbgPrint(
                b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as *const i8,
                b"sdk/lib/drivers/arbiter/arbiter.c\0"
                    as *const u8 as *const i8,
                403,
            );
        }
        Status
    }
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:     ReactOS Kernel&Driver SDK
//  * LICENSE:     GPL-2.0-or-later (https://spdx.org/licenses/GPL-2.0-or-later)
//  * PURPOSE:     Hardware Resources Arbiter Library
//  * COPYRIGHT:   Copyright 2020 Vadim Galyant <vgal@rambler.ru>
//  */
//
// /* INCLUDES *******************************************************************/
//
// #include <ntifs.h>
// #include <ndk/rtlfuncs.h>
// #include "arbiter.h"
//
// #define NDEBUG
// #include <debug.h>
//
// /* GLOBALS ********************************************************************/
//
// /* DATA **********************************************************************/
//
// /* FUNCTIONS ******************************************************************/
//
// CODE_SEG("PAGE")
// NTSTATUS
// NTAPI
// ArbTestAllocation(
//      PARBITER_INSTANCE Arbiter,
//      PLIST_ENTRY ArbitrationList)
// {
//     PAGED_CODE();
//
//     UNIMPLEMENTED;
//     return STATUS_NOT_IMPLEMENTED;
// }
//
// CODE_SEG("PAGE")
// NTSTATUS
// NTAPI
// ArbRetestAllocation(
//      PARBITER_INSTANCE Arbiter,
//      PLIST_ENTRY ArbitrationList)
// {
//     PAGED_CODE();
//
//     UNIMPLEMENTED;
//     return STATUS_NOT_IMPLEMENTED;
// }
//
// CODE_SEG("PAGE")
// NTSTATUS
// NTAPI
// ArbCommitAllocation(
//      PARBITER_INSTANCE Arbiter)
// {
//     PAGED_CODE();
//
//     UNIMPLEMENTED;
//     return STATUS_NOT_IMPLEMENTED;
// }
//
// CODE_SEG("PAGE")
// NTSTATUS
// NTAPI
// ArbRollbackAllocation(
//      PARBITER_INSTANCE Arbiter)
// {
//     PAGED_CODE();
//
//     UNIMPLEMENTED;
//     return STATUS_NOT_IMPLEMENTED;
// }
//
// /* FIXME: the prototype is not correct yet. */
// CODE_SEG("PAGE")
// NTSTATUS
// NTAPI
// ArbAddReserved(
//      PARBITER_INSTANCE Arbiter)
// {
//     PAGED_CODE();
//
//     UNIMPLEMENTED;
//     return STATUS_NOT_IMPLEMENTED;
// }
//
// CODE_SEG("PAGE")
// NTSTATUS
// NTAPI
// ArbPreprocessEntry(
//      PARBITER_INSTANCE Arbiter,
//      PARBITER_ALLOCATION_STATE ArbState)
// {
//     PAGED_CODE();
//
//     return STATUS_SUCCESS;
// }
//
// CODE_SEG("PAGE")
// NTSTATUS
// NTAPI
// ArbAllocateEntry(
//      PARBITER_INSTANCE Arbiter,
//      PARBITER_ALLOCATION_STATE ArbState)
// {
//     PAGED_CODE();
//
//     UNIMPLEMENTED;
//     return STATUS_NOT_IMPLEMENTED;
// }
//
// CODE_SEG("PAGE")
// BOOLEAN
// NTAPI
// ArbGetNextAllocationRange(
//      PARBITER_INSTANCE Arbiter,
//      PARBITER_ALLOCATION_STATE ArbState)
// {
//     PAGED_CODE();
//
//     UNIMPLEMENTED;
//     return FALSE;
// }
//
// CODE_SEG("PAGE")
// BOOLEAN
// NTAPI
// ArbFindSuitableRange(
//      PARBITER_INSTANCE Arbiter,
//      PARBITER_ALLOCATION_STATE ArbState)
// {
//     PAGED_CODE();
//
//     UNIMPLEMENTED;
//     return FALSE;
// }
//
// CODE_SEG("PAGE")
// VOID
// NTAPI
// ArbAddAllocation(
//      PARBITER_INSTANCE Arbiter,
//      PARBITER_ALLOCATION_STATE ArbState)
// {
//     PAGED_CODE();
//
//     UNIMPLEMENTED;
// }
//
// CODE_SEG("PAGE")
// VOID
// NTAPI
// ArbBacktrackAllocation(
//      PARBITER_INSTANCE Arbiter,
//      PARBITER_ALLOCATION_STATE ArbState)
// {
//     PAGED_CODE();
//
//     UNIMPLEMENTED;
// }
//
// /* FIXME: the prototype is not correct yet. */
// CODE_SEG("PAGE")
// NTSTATUS
// NTAPI
// ArbOverrideConflict(
//      PARBITER_INSTANCE Arbiter)
// {
//     PAGED_CODE();
//
//     UNIMPLEMENTED;
//     return STATUS_NOT_IMPLEMENTED;
// }
//
// CODE_SEG("PAGE")
// NTSTATUS
// NTAPI
// ArbBootAllocation(
//      PARBITER_INSTANCE Arbiter,
//      PLIST_ENTRY ArbitrationList)
// {
//     PAGED_CODE();
//
//     UNIMPLEMENTED;
//     return STATUS_NOT_IMPLEMENTED;
// }
//
// /* FIXME: the prototype is not correct yet. */
// CODE_SEG("PAGE")
// NTSTATUS
// NTAPI
// ArbQueryConflict(
//      PARBITER_INSTANCE Arbiter)
// {
//     PAGED_CODE();
//
//     UNIMPLEMENTED;
//     return STATUS_NOT_IMPLEMENTED;
// }
//
// /* FIXME: the prototype is not correct yet. */
// CODE_SEG("PAGE")
// NTSTATUS
// NTAPI
// ArbStartArbiter(
//      PARBITER_INSTANCE Arbiter)
// {
//     PAGED_CODE();
//
//     UNIMPLEMENTED;
//     return STATUS_NOT_IMPLEMENTED;
// }
//
// CODE_SEG("PAGE")
// NTSTATUS
// NTAPI
// ArbAddOrdering(
//      PARBITER_ORDERING_LIST OrderList,
//      UINT64 MinimumAddress,
//      UINT64 MaximumAddress)
// {
//     PAGED_CODE();
//
//     UNIMPLEMENTED;
//     return STATUS_NOT_IMPLEMENTED;
// }
//
// CODE_SEG("PAGE")
// NTSTATUS
// NTAPI
// ArbPruneOrdering(
//      PARBITER_ORDERING_LIST OrderingList,
//      UINT64 MinimumAddress,
//      UINT64 MaximumAddress)
// {
//     PAGED_CODE();
//
//     UNIMPLEMENTED;
//     return STATUS_NOT_IMPLEMENTED;
// }
//
// CODE_SEG("PAGE")
// NTSTATUS
// NTAPI
// ArbInitializeOrderingList(
//      PARBITER_ORDERING_LIST OrderList)
// {
//     PAGED_CODE();
//
//     UNIMPLEMENTED;
//     return STATUS_NOT_IMPLEMENTED;
// }
//
// CODE_SEG("PAGE")
// VOID
// NTAPI
// ArbFreeOrderingList(
//      PARBITER_ORDERING_LIST OrderList)
// {
//     PAGED_CODE();
//
//     UNIMPLEMENTED;
// }
//
// CODE_SEG("PAGE")
// NTSTATUS
// NTAPI
// ArbBuildAssignmentOrdering(
//      PARBITER_INSTANCE ArbInstance,
//      PCWSTR OrderName,
//      PCWSTR ReservedOrderName,
//      PARB_TRANSLATE_ORDERING TranslateOrderingFunction)
// {
//     PAGED_CODE();
//
//     UNIMPLEMENTED;
//     return STATUS_SUCCESS;
// }
//
// CODE_SEG("PAGE")
// NTSTATUS
// NTAPI
// ArbInitializeArbiterInstance(
//      PARBITER_INSTANCE Arbiter,
//      PDEVICE_OBJECT BusDeviceObject,
//      CM_RESOURCE_TYPE ResourceType,
//      PCWSTR ArbiterName,
//      PCWSTR OrderName,
//      PARB_TRANSLATE_ORDERING TranslateOrderingFunction)
// {
//     NTSTATUS Status;
//
//     PAGED_CODE();
//
//     DPRINT("ArbInitializeArbiterInstance: '%S'\n", ArbiterName);
//
//     ASSERT(Arbiter->UnpackRequirement != NULL);
//     ASSERT(Arbiter->PackResource != NULL);
//     ASSERT(Arbiter->UnpackResource != NULL);
//     ASSERT(Arbiter->MutexEvent == NULL);
//     ASSERT(Arbiter->Allocation == NULL);
//     ASSERT(Arbiter->PossibleAllocation == NULL);
//     ASSERT(Arbiter->AllocationStack == NULL);
//
//     Arbiter->Signature = ARBITER_SIGNATURE;
//     Arbiter->BusDeviceObject = BusDeviceObject;
//
//     Arbiter->MutexEvent = ExAllocatePoolWithTag(NonPagedPool, sizeof(KEVENT), TAG_ARBITER);
//     if (!Arbiter->MutexEvent)
//     {
//         DPRINT1("ArbInitializeArbiterInstance: STATUS_INSUFFICIENT_RESOURCES\n");
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     KeInitializeEvent(Arbiter->MutexEvent, SynchronizationEvent, TRUE);
//
//     Arbiter->AllocationStack = ExAllocatePoolWithTag(PagedPool, PAGE_SIZE, TAG_ARB_ALLOCATION);
//     if (!Arbiter->AllocationStack)
//     {
//         DPRINT1("ArbInitializeArbiterInstance: STATUS_INSUFFICIENT_RESOURCES\n");
//         ExFreePoolWithTag(Arbiter->MutexEvent, TAG_ARBITER);
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     Arbiter->AllocationStackMaxSize = PAGE_SIZE;
//
//     Arbiter->Allocation = ExAllocatePoolWithTag(PagedPool, sizeof(RTL_RANGE_LIST), TAG_ARB_RANGE);
//     if (!Arbiter->Allocation)
//     {
//         DPRINT1("ArbInitializeArbiterInstance: STATUS_INSUFFICIENT_RESOURCES\n");
//         ExFreePoolWithTag(Arbiter->AllocationStack, TAG_ARB_ALLOCATION);
//         ExFreePoolWithTag(Arbiter->MutexEvent, TAG_ARBITER);
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     Arbiter->PossibleAllocation = ExAllocatePoolWithTag(PagedPool, sizeof(RTL_RANGE_LIST), TAG_ARB_RANGE);
//     if (!Arbiter->PossibleAllocation)
//     {
//         DPRINT1("ArbInitializeArbiterInstance: STATUS_INSUFFICIENT_RESOURCES\n");
//         ExFreePoolWithTag(Arbiter->Allocation, TAG_ARB_RANGE);
//         ExFreePoolWithTag(Arbiter->AllocationStack, TAG_ARB_ALLOCATION);
//         ExFreePoolWithTag(Arbiter->MutexEvent, TAG_ARBITER);
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     RtlInitializeRangeList(Arbiter->Allocation);
//     RtlInitializeRangeList(Arbiter->PossibleAllocation);
//
//     Arbiter->Name = ArbiterName;
//     Arbiter->ResourceType = ResourceType;
//     Arbiter->TransactionInProgress = FALSE;
//
//     if (!Arbiter->TestAllocation)
//         Arbiter->TestAllocation = ArbTestAllocation;
//
//     if (!Arbiter->RetestAllocation)
//         Arbiter->RetestAllocation = ArbRetestAllocation;
//
//     if (!Arbiter->CommitAllocation)
//         Arbiter->CommitAllocation = ArbCommitAllocation;
//
//     if (!Arbiter->RollbackAllocation)
//         Arbiter->RollbackAllocation = ArbRollbackAllocation;
//
//     if (!Arbiter->AddReserved)
//         Arbiter->AddReserved = ArbAddReserved;
//
//     if (!Arbiter->PreprocessEntry)
//         Arbiter->PreprocessEntry = ArbPreprocessEntry;
//
//     if (!Arbiter->AllocateEntry)
//         Arbiter->AllocateEntry = ArbAllocateEntry;
//
//     if (!Arbiter->GetNextAllocationRange)
//         Arbiter->GetNextAllocationRange = ArbGetNextAllocationRange;
//
//     if (!Arbiter->FindSuitableRange)
//         Arbiter->FindSuitableRange = ArbFindSuitableRange;
//
//     if (!Arbiter->AddAllocation)
//         Arbiter->AddAllocation = ArbAddAllocation;
//
//     if (!Arbiter->BacktrackAllocation)
//         Arbiter->BacktrackAllocation = ArbBacktrackAllocation;
//
//     if (!Arbiter->OverrideConflict)
//         Arbiter->OverrideConflict = ArbOverrideConflict;
//
//     if (!Arbiter->BootAllocation)
//         Arbiter->BootAllocation = ArbBootAllocation;
//
//     if (!Arbiter->QueryConflict)
//         Arbiter->QueryConflict = ArbQueryConflict;
//
//     if (!Arbiter->StartArbiter)
//         Arbiter->StartArbiter = ArbStartArbiter;
//
//     Status = ArbBuildAssignmentOrdering(Arbiter, OrderName, OrderName, TranslateOrderingFunction);
//     if (NT_SUCCESS(Status))
//     {
//         return STATUS_SUCCESS;
//     }
//
//     DPRINT1("ArbInitializeArbiterInstance: Status %X\n", Status);
//
//     return Status;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
