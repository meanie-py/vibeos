//! eval.c → Rust (ACPI Phase 3b glue leaf). Cohab: #[no_mangle] symbols
//! replace the C file in acpi.sys at link time. ACPICA stays C (libacpica.a).
//!
//! Marshals ACPI method-evaluation IOCTLs between the WDM ACPI_EVAL_INPUT/OUTPUT
//! buffer format (acpiioct.h) and ACPICA's ACPI_OBJECT format.
//!
//! PACKED-STRUCT HAZARD: ACPI_METHOD_ARGUMENT and ACPI_EVAL_OUTPUT_BUFFER are
//! `UNALIGNED` in acpiioct.h. They are modeled `#[repr(C, packed)]` here and ALL
//! field access goes through addr_of!/addr_of_mut! + read_unaligned/write_unaligned.
//! NEVER take a reference to a packed field (UB → potential kernel hang).
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals, unused)]

use crate::shared::*;
use core::ffi::{c_char, c_int, c_void};
use core::ptr::{addr_of, addr_of_mut, read_unaligned, write_unaligned};

// Kernel types from ntkrnl-sys.
use ntkrnl_sys::ddk::{
    PIRP, PIO_STACK_LOCATION, NTSTATUS, ULONG, ULONG64, USHORT, UCHAR, PVOID, SIZE_T,
    POOL_TYPE, ExFreePoolWithTag,
};
use ntkrnl_sys::mm::NonPagedPool;
use ntkrnl_sys::base::{STATUS_SUCCESS, STATUS_INSUFFICIENT_RESOURCES, STATUS_NOT_IMPLEMENTED};
use ntkrnl_sys::constants::{
    STATUS_UNSUCCESSFUL, STATUS_INFO_LENGTH_MISMATCH, STATUS_BUFFER_TOO_SMALL,
    STATUS_BUFFER_OVERFLOW, STATUS_INVALID_PARAMETER_1, STATUS_OBJECT_NAME_NOT_FOUND,
};
use ntkrnl_sys::inline::tier1::IoGetCurrentIrpStackLocation;

// ACPICA types + functions (outbound, all extern "C").
use acpica_sys::{
    ACPI_STATUS, ACPI_HANDLE, ACPI_STRING, ACPI_SIZE, ACPI_OBJECT, ACPI_OBJECT_LIST,
    ACPI_BUFFER, ACPI_OBJECT_TYPE,
    ACPI_TYPE_INTEGER, ACPI_TYPE_STRING, ACPI_TYPE_BUFFER, ACPI_TYPE_PACKAGE,
    ACPI_TYPE_LOCAL_REFERENCE, ACPI_SINGLE_NAME,
    AcpiGetName, AcpiEvaluateObject, AcpiOsFree,
};

// AE_OK / AE_NOT_FOUND come from crate::shared (verified there). Define the rest
// from acexcep.h: EXCEP_ENV(c) = c | AE_CODE_ENVIRONMENTAL(0x0000),
//                 EXCEP_AML(c) = c | AE_CODE_AML(0x3000), AE_CODE_MASK = 0xF000.
const AE_NOT_EXIST: ACPI_STATUS = 0x0006; // EXCEP_ENV(0x0006)
const AE_NO_MEMORY: ACPI_STATUS = 0x0004; // EXCEP_ENV(0x0004)
const AE_SUPPORT: ACPI_STATUS = 0x000F; // EXCEP_ENV(0x000F)
const AE_TIME: ACPI_STATUS = 0x0011; // EXCEP_ENV(0x0011)
const AE_NO_HARDWARE_RESPONSE: ACPI_STATUS = 0x0016; // EXCEP_ENV(0x0016)
const AE_STACK_OVERFLOW: ACPI_STATUS = 0x000C; // EXCEP_ENV(0x000C)
const AE_AML_UNINITIALIZED_ARG: ACPI_STATUS = 0x3006; // EXCEP_AML(0x0006)
const AE_CODE_MASK: ACPI_STATUS = 0xF000;
const AE_CODE_ENVIRONMENTAL: ACPI_STATUS = 0x0000;
const AE_CODE_AML: ACPI_STATUS = 0x3000;

// ACPI-specific NTSTATUS codes (ntstatus.h — verified against psdk/ntstatus.h).
const STATUS_ACPI_INVALID_ARGTYPE: NTSTATUS = 0xC0140008_u32 as i32;
const STATUS_ACPI_INCORRECT_ARGUMENT_COUNT: NTSTATUS = 0xC014000B_u32 as i32;
const STATUS_ACPI_STACK_OVERFLOW: NTSTATUS = 0xC0140002_u32 as i32;
// STATUS_IO_TIMEOUT / STATUS_IO_DEVICE_ERROR are not in ntkrnl_sys (verified) —
// define from psdk/ntstatus.h.
const STATUS_IO_TIMEOUT: NTSTATUS = 0xC00000B5_u32 as i32;
const STATUS_IO_DEVICE_ERROR: NTSTATUS = 0xC0000185_u32 as i32;

// ACPI_ALLOCATE_BUFFER = (ACPI_SIZE)(-1) (actypes.h, !ACPI_NO_MEM_ALLOCATIONS).
// ACPI_SIZE is UINT64 on amd64, UINT32 on i386 and arm64 (the C build's
// ACPI_MACHINE_WIDTH); build from the field type so it is correct on every arch.
const ACPI_ALLOCATE_BUFFER: ACPI_SIZE = ACPI_SIZE::MAX; // (ACPI_SIZE)(-1)

const ANSI_NULL: c_char = 0;

// acpiioct.h driver constants.
const TAG_ACPI_PARAMETERS_LIST: ULONG = u32::from_be_bytes([b'O', b'p', b'c', b'A']); // 'OpcA'
const TAG_ACPI_PACKAGE_LIST: ULONG = u32::from_be_bytes([b'P', b'p', b'c', b'A']); // 'PpcA'

const ACPI_OBJECT_NAME_LENGTH: ULONG = 4 + 1;
const ACPI_MAX_PACKAGE_DEPTH: ULONG = 5;

const ACPI_METHOD_ARGUMENT_INTEGER: USHORT = 0x0;
const ACPI_METHOD_ARGUMENT_STRING: USHORT = 0x1;
const ACPI_METHOD_ARGUMENT_BUFFER: USHORT = 0x2;
const ACPI_METHOD_ARGUMENT_PACKAGE: USHORT = 0x3;

// 4-char C multichar 'WXYZ' == from_be_bytes([W,X,Y,Z]).
const ACPI_EVAL_INPUT_BUFFER_SIGNATURE: ULONG = u32::from_be_bytes([b'B', b'i', b'e', b'A']);
const ACPI_EVAL_INPUT_BUFFER_SIMPLE_INTEGER_SIGNATURE: ULONG =
    u32::from_be_bytes([b'I', b'i', b'e', b'A']);
const ACPI_EVAL_INPUT_BUFFER_SIMPLE_STRING_SIGNATURE: ULONG =
    u32::from_be_bytes([b'S', b'i', b'e', b'A']);
const ACPI_EVAL_INPUT_BUFFER_COMPLEX_SIGNATURE: ULONG =
    u32::from_be_bytes([b'C', b'i', b'e', b'A']);
const ACPI_EVAL_OUTPUT_BUFFER_SIGNATURE: ULONG = u32::from_be_bytes([b'B', b'o', b'e', b'A']);

// ===========================================================================
//  acpiioct.h structs (NOT in any -sys crate — defined locally per the brief).
//  Layouts taken verbatim from sdk/include/ddk/acpiioct.h.
// ===========================================================================

/// _ACPI_METHOD_ARGUMENT — UNALIGNED in C → #[repr(C, packed)].
/// Type(u16) + DataLength(u16) + union { Argument:u32, Data:[u8;1] }.
/// `Argument` overlaps `Data[0..4]`; Data starts at offset 4 (== offset of Argument).
#[repr(C, packed)]
pub struct ACPI_METHOD_ARGUMENT {
    pub Type: USHORT,
    pub DataLength: USHORT,
    pub Argument: ULONG, // union member; Data[] begins at this same offset (4)
}
type PACPI_METHOD_ARGUMENT = *mut ACPI_METHOD_ARGUMENT;

#[repr(C)]
pub struct ACPI_EVAL_INPUT_BUFFER {
    pub Signature: ULONG,
    pub MethodName: [UCHAR; 4], // union with MethodNameAsUlong:ULONG
}
type PACPI_EVAL_INPUT_BUFFER = *mut ACPI_EVAL_INPUT_BUFFER;

#[repr(C)]
pub struct ACPI_EVAL_INPUT_BUFFER_SIMPLE_INTEGER {
    pub Signature: ULONG,
    pub MethodName: [UCHAR; 4],
    pub IntegerArgument: ULONG,
}

#[repr(C)]
pub struct ACPI_EVAL_INPUT_BUFFER_SIMPLE_STRING {
    pub Signature: ULONG,
    pub MethodName: [UCHAR; 4],
    pub StringLength: ULONG,
    pub String: [UCHAR; 1], // ANYSIZE_ARRAY
}

#[repr(C)]
pub struct ACPI_EVAL_INPUT_BUFFER_COMPLEX {
    pub Signature: ULONG,
    pub MethodName: [UCHAR; 4],
    pub Size: ULONG,
    pub ArgumentCount: ULONG,
    pub Argument: [ACPI_METHOD_ARGUMENT; 1], // ANYSIZE_ARRAY
}

/// _ACPI_EVAL_OUTPUT_BUFFER — UNALIGNED in C → #[repr(C, packed)].
#[repr(C, packed)]
pub struct ACPI_EVAL_OUTPUT_BUFFER {
    pub Signature: ULONG,
    pub Length: ULONG,
    pub Count: ULONG,
    pub Argument: [ACPI_METHOD_ARGUMENT; 1], // ANYSIZE_ARRAY
}

// ===========================================================================
//  acpiioct.h macros → inline helpers.
// ===========================================================================

/// FIELD_OFFSET(ACPI_METHOD_ARGUMENT, Data) == 4 (Data overlaps Argument at off 4).
const ACPI_METHOD_ARGUMENT_DATA_OFFSET: ULONG = 4;

/// ACPI_METHOD_ARGUMENT_LENGTH(dl) = FIELD_OFFSET(.,Data) + max(sizeof(ULONG), dl).
#[inline(always)]
fn ACPI_METHOD_ARGUMENT_LENGTH(data_length: ULONG) -> ULONG {
    ACPI_METHOD_ARGUMENT_DATA_OFFSET + core::cmp::max(core::mem::size_of::<ULONG>() as ULONG, data_length)
}

/// ACPI_METHOD_ARGUMENT_LENGTH_FROM_ARGUMENT(arg) — packed read of DataLength.
#[inline(always)]
unsafe fn ACPI_METHOD_ARGUMENT_LENGTH_FROM_ARGUMENT(arg: PACPI_METHOD_ARGUMENT) -> ULONG {
    let dl = read_unaligned(addr_of!((*arg).DataLength)) as ULONG;
    ACPI_METHOD_ARGUMENT_LENGTH(dl)
}

/// ACPI_METHOD_NEXT_ARGUMENT(arg) = (PUCHAR)arg + LENGTH_FROM_ARGUMENT(arg).
#[inline(always)]
unsafe fn ACPI_METHOD_NEXT_ARGUMENT(arg: PACPI_METHOD_ARGUMENT) -> PACPI_METHOD_ARGUMENT {
    let len = ACPI_METHOD_ARGUMENT_LENGTH_FROM_ARGUMENT(arg);
    (arg as *mut u8).add(len as usize) as PACPI_METHOD_ARGUMENT
}

/// Pointer to Data[0] of a (packed) ACPI_METHOD_ARGUMENT.
#[inline(always)]
unsafe fn ACPI_METHOD_ARGUMENT_DATA(arg: PACPI_METHOD_ARGUMENT) -> *mut u8 {
    // Data starts at the same offset as Argument (4).
    addr_of_mut!((*arg).Argument) as *mut u8
}

#[inline(always)]
unsafe fn ACPI_METHOD_SET_ARGUMENT_INTEGER(arg: PACPI_METHOD_ARGUMENT, int_data: ULONG) {
    write_unaligned(addr_of_mut!((*arg).Type), ACPI_METHOD_ARGUMENT_INTEGER);
    write_unaligned(addr_of_mut!((*arg).DataLength), core::mem::size_of::<ULONG>() as USHORT);
    write_unaligned(addr_of_mut!((*arg).Argument), int_data);
}

#[inline(always)]
unsafe fn ACPI_METHOD_SET_ARGUMENT_STRING(arg: PACPI_METHOD_ARGUMENT, str_data: *const c_char) {
    let len = (strlen(str_data) as USHORT).wrapping_add(core::mem::size_of::<UCHAR>() as USHORT);
    write_unaligned(addr_of_mut!((*arg).Type), ACPI_METHOD_ARGUMENT_STRING);
    write_unaligned(addr_of_mut!((*arg).DataLength), len);
    RtlCopyMemory(
        ACPI_METHOD_ARGUMENT_DATA(arg) as *mut c_void,
        str_data as *const c_void,
        len as usize,
    );
}

#[inline(always)]
unsafe fn ACPI_METHOD_SET_ARGUMENT_BUFFER(
    arg: PACPI_METHOD_ARGUMENT,
    buff_data: *const u8,
    buff_length: USHORT,
) {
    write_unaligned(addr_of_mut!((*arg).Type), ACPI_METHOD_ARGUMENT_BUFFER);
    write_unaligned(addr_of_mut!((*arg).DataLength), buff_length);
    RtlCopyMemory(
        ACPI_METHOD_ARGUMENT_DATA(arg) as *mut c_void,
        buff_data as *const c_void,
        buff_length as usize,
    );
}

// AcpiVerifyInBuffer(Stack, Length) = InputBufferLength >= Length.
#[inline(always)]
unsafe fn AcpiVerifyInBuffer(stack: PIO_STACK_LOCATION, length: ULONG) -> bool {
    (*stack).Parameters.DeviceIoControl.InputBufferLength >= length
}

// AcpiVerifyOutBuffer(Stack, Length) = OutputBufferLength >= Length.
#[inline(always)]
unsafe fn AcpiVerifyOutBuffer(stack: PIO_STACK_LOCATION, length: ULONG) -> bool {
    (*stack).Parameters.DeviceIoControl.OutputBufferLength >= length
}

// acexcep.h ACPI_ENV_EXCEPTION / ACPI_AML_EXCEPTION.
#[inline(always)]
fn ACPI_ENV_EXCEPTION(s: ACPI_STATUS) -> bool {
    (s & AE_CODE_MASK) == AE_CODE_ENVIRONMENTAL
}
#[inline(always)]
fn ACPI_AML_EXCEPTION(s: ACPI_STATUS) -> bool {
    (s & AE_CODE_MASK) == AE_CODE_AML
}

// ACPI_SUCCESS / NT_SUCCESS helpers.
#[inline(always)]
fn ACPI_SUCCESS(s: ACPI_STATUS) -> bool {
    s == AE_OK
}
#[inline(always)]
fn NT_SUCCESS(s: NTSTATUS) -> bool {
    s >= 0
}

// ===========================================================================
//  External C functions (CRT / intsafe / pool).
// ===========================================================================
extern "C" {
    fn strlen(s: *const c_char) -> usize;
}

// RtlCopyMemory is a C macro (→ memcpy), not an export → inline (ntkrnl_sys::rtl form).
#[inline]
unsafe fn RtlCopyMemory(dst: *mut c_void, src: *const c_void, len: usize) {
    core::ptr::copy_nonoverlapping(src as *const u8, dst as *mut u8, len);
}
// RtlULongMult is a FORCEINLINE in ntintsafe.h (no export) → inline checked multiply.
#[inline]
unsafe fn RtlULongMult(a: ULONG, b: ULONG, out: *mut ULONG) -> NTSTATUS {
    match a.checked_mul(b) {
        Some(v) => {
            *out = v;
            STATUS_SUCCESS
        }
        None => 0xC0000095_u32 as i32, // STATUS_INTEGER_OVERFLOW
    }
}
// ExAllocatePoolUninitialized is a FORCEINLINE (exfuncs.h) → ExAllocatePoolWithTag.
use ntkrnl_sys::ex::ExAllocatePoolWithTag;
#[inline]
unsafe fn ExAllocatePoolUninitialized(
    PoolType: POOL_TYPE,
    NumberOfBytes: SIZE_T,
    Tag: ULONG,
) -> PVOID {
    ExAllocatePoolWithTag(PoolType, NumberOfBytes as _, Tag) as PVOID
}

// ===========================================================================
//  EvalConvertObjectReference (static → private)
// ===========================================================================
unsafe fn EvalConvertObjectReference(
    Argument: PACPI_METHOD_ARGUMENT,
    Reference: *mut ACPI_OBJECT,
) -> NTSTATUS {
    write_unaligned(addr_of_mut!((*Argument).Type), ACPI_METHOD_ARGUMENT_STRING);
    write_unaligned(addr_of_mut!((*Argument).DataLength), ACPI_OBJECT_NAME_LENGTH as USHORT);

    // Convert the object handle to an ACPI name.
    let mut OutName: ACPI_BUFFER = core::mem::zeroed();
    OutName.Length = ACPI_OBJECT_NAME_LENGTH as ACPI_SIZE;
    OutName.Pointer = ACPI_METHOD_ARGUMENT_DATA(Argument) as *mut c_void;

    let handle = read_unaligned(addr_of!((*Reference).Reference.Handle));
    let AcpiStatus = AcpiGetName(handle, ACPI_SINGLE_NAME, &mut OutName);
    if !ACPI_SUCCESS(AcpiStatus) {
        DPRINT1!("AcpiGetName() failed on %p with status 0x%04x\n", handle, AcpiStatus);
        return STATUS_UNSUCCESSFUL;
    }

    STATUS_SUCCESS
}

// ===========================================================================
//  EvalGetElementSize (static → private, recursive)
// ===========================================================================
unsafe fn EvalGetElementSize(
    Obj: *mut ACPI_OBJECT,
    Depth: ULONG,
    Count: *mut ULONG, // _Out_opt_ (may be null)
    Size: *mut ULONG,
) -> NTSTATUS {
    let TotalCount: ULONG;
    let TotalLength: ULONG;

    if Depth >= ACPI_MAX_PACKAGE_DEPTH {
        return STATUS_UNSUCCESSFUL;
    }

    let obj_type = read_unaligned(addr_of!((*Obj).Type));
    match obj_type {
        x if x == ACPI_TYPE_INTEGER => {
            TotalLength = ACPI_METHOD_ARGUMENT_LENGTH(core::mem::size_of::<ULONG>() as ULONG);
            TotalCount = 1;
        }
        x if x == ACPI_TYPE_STRING => {
            let slen = read_unaligned(addr_of!((*Obj).String.Length));
            TotalLength =
                ACPI_METHOD_ARGUMENT_LENGTH(slen + core::mem::size_of::<UCHAR>() as ULONG);
            TotalCount = 1;
        }
        x if x == ACPI_TYPE_BUFFER => {
            let blen = read_unaligned(addr_of!((*Obj).Buffer.Length));
            TotalLength = ACPI_METHOD_ARGUMENT_LENGTH(blen);
            TotalCount = 1;
        }
        x if x == ACPI_TYPE_PACKAGE => {
            let pkg_count = read_unaligned(addr_of!((*Obj).Package.Count));
            let elements = read_unaligned(addr_of!((*Obj).Package.Elements));

            // Get the size of the current packet.
            let mut TotalPackageLength: ULONG = 0;
            let mut i: ULONG = 0;
            while i < pkg_count {
                let mut ElementSize: ULONG = 0;
                let Status = EvalGetElementSize(
                    elements.add(i as usize),
                    Depth + 1,
                    core::ptr::null_mut(),
                    &mut ElementSize,
                );
                if !NT_SUCCESS(Status) {
                    return Status;
                }
                TotalPackageLength = TotalPackageLength.wrapping_add(ElementSize);
                i += 1;
            }

            // Check if we need to wrap the list of elements into a package.
            if Depth > 0 {
                TotalPackageLength = ACPI_METHOD_ARGUMENT_LENGTH(TotalPackageLength);
            }

            TotalLength = TotalPackageLength;
            TotalCount = pkg_count;
        }
        x if x == ACPI_TYPE_LOCAL_REFERENCE => {
            TotalLength = ACPI_METHOD_ARGUMENT_LENGTH(ACPI_OBJECT_NAME_LENGTH);
            TotalCount = 1;
        }
        _ => {
            DPRINT1!("Unsupported element type %u\n", obj_type);
            return STATUS_UNSUCCESSFUL;
        }
    }

    if !Count.is_null() {
        *Count = TotalCount;
    }
    *Size = TotalLength;

    STATUS_SUCCESS
}

// ===========================================================================
//  EvalConvertEvaluationResults (static → private, recursive)
// ===========================================================================
unsafe fn EvalConvertEvaluationResults(
    Argument: PACPI_METHOD_ARGUMENT,
    Depth: ULONG,
    Obj: *mut ACPI_OBJECT,
) -> NTSTATUS {
    if Depth >= ACPI_MAX_PACKAGE_DEPTH {
        return STATUS_UNSUCCESSFUL;
    }

    let mut Ptr: PACPI_METHOD_ARGUMENT = Argument;
    let obj_type = read_unaligned(addr_of!((*Obj).Type));
    match obj_type {
        x if x == ACPI_TYPE_INTEGER => {
            let value = read_unaligned(addr_of!((*Obj).Integer.Value));
            ACPI_METHOD_SET_ARGUMENT_INTEGER(Ptr, value as ULONG);
        }
        x if x == ACPI_TYPE_STRING => {
            let pointer = read_unaligned(addr_of!((*Obj).String.Pointer));
            ACPI_METHOD_SET_ARGUMENT_STRING(Ptr, pointer);
        }
        x if x == ACPI_TYPE_BUFFER => {
            let pointer = read_unaligned(addr_of!((*Obj).Buffer.Pointer));
            let length = read_unaligned(addr_of!((*Obj).Buffer.Length));
            ACPI_METHOD_SET_ARGUMENT_BUFFER(Ptr, pointer, length as USHORT);
        }
        x if x == ACPI_TYPE_PACKAGE => {
            let pkg_count = read_unaligned(addr_of!((*Obj).Package.Count));
            let elements = read_unaligned(addr_of!((*Obj).Package.Elements));

            // Check if we need to wrap the list of elements into a package.
            if Depth > 0 {
                // Get the size of the current packet.
                let mut TotalPackageLength: ULONG = 0;
                let mut i: ULONG = 0;
                while i < pkg_count {
                    let mut ElementSize: ULONG = 0;
                    let Status = EvalGetElementSize(
                        elements.add(i as usize),
                        Depth + 1,
                        core::ptr::null_mut(),
                        &mut ElementSize,
                    );
                    if !NT_SUCCESS(Status) {
                        return Status;
                    }
                    TotalPackageLength = TotalPackageLength.wrapping_add(ElementSize);
                    i += 1;
                }

                // Start a new package.
                write_unaligned(addr_of_mut!((*Argument).Type), ACPI_METHOD_ARGUMENT_PACKAGE);
                write_unaligned(addr_of_mut!((*Argument).DataLength), TotalPackageLength as USHORT);

                Ptr = ACPI_METHOD_ARGUMENT_DATA(Ptr) as PACPI_METHOD_ARGUMENT;
            }

            let mut i: ULONG = 0;
            while i < pkg_count {
                let Status =
                    EvalConvertEvaluationResults(Ptr, Depth + 1, elements.add(i as usize));
                if !NT_SUCCESS(Status) {
                    return Status;
                }
                Ptr = ACPI_METHOD_NEXT_ARGUMENT(Ptr);
                i += 1;
            }
        }
        x if x == ACPI_TYPE_LOCAL_REFERENCE => {
            let Status = EvalConvertObjectReference(Ptr, Obj);
            if !NT_SUCCESS(Status) {
                return Status;
            }
        }
        _ => {
            DPRINT1!("Unsupported element type %u\n", obj_type);
            return STATUS_UNSUCCESSFUL;
        }
    }

    STATUS_SUCCESS
}

// ===========================================================================
//  EvalGetPackageCount (static → private)
// ===========================================================================
unsafe fn EvalGetPackageCount(
    Package: PACPI_METHOD_ARGUMENT,
    PackageArgument: PACPI_METHOD_ARGUMENT,
    DataLength: ULONG,
) -> ULONG {
    // Empty package.
    let pkg_argument = read_unaligned(addr_of!((*Package).Argument));
    if DataLength < ACPI_METHOD_ARGUMENT_LENGTH(0) || pkg_argument == 0 {
        return 0;
    }

    let mut Ptr: PACPI_METHOD_ARGUMENT = PackageArgument;
    let mut TotalLength: ULONG = 0;
    let mut TotalCount: ULONG = 0;
    while TotalLength < DataLength {
        TotalLength =
            TotalLength.wrapping_add(ACPI_METHOD_ARGUMENT_LENGTH_FROM_ARGUMENT(Ptr));
        TotalCount += 1;

        Ptr = ACPI_METHOD_NEXT_ARGUMENT(Ptr);
    }

    TotalCount
}

// ===========================================================================
//  EvalConvertParameterObjects (static → private, recursive)
// ===========================================================================
unsafe fn EvalConvertParameterObjects(
    Arg: *mut ACPI_OBJECT,
    Depth: ULONG,
    Argument: PACPI_METHOD_ARGUMENT,
    IoStack: PIO_STACK_LOCATION,
    mut Offset: ULONG,
) -> NTSTATUS {
    if Depth >= ACPI_MAX_PACKAGE_DEPTH {
        return STATUS_UNSUCCESSFUL;
    }

    // Validate that the method argument fits into the buffer.
    if Depth > 0 {
        Offset = Offset.wrapping_add(ACPI_METHOD_ARGUMENT_LENGTH_FROM_ARGUMENT(Argument));

        if !AcpiVerifyInBuffer(IoStack, Offset) {
            DPRINT1!("Argument buffer outside of argument bounds\n");
            return STATUS_ACPI_INVALID_ARGTYPE;
        }
    }

    let arg_type = read_unaligned(addr_of!((*Argument).Type));
    match arg_type {
        x if x == ACPI_METHOD_ARGUMENT_INTEGER => {
            let argument_val = read_unaligned(addr_of!((*Argument).Argument));
            write_unaligned(addr_of_mut!((*Arg).Type), ACPI_TYPE_INTEGER);
            write_unaligned(addr_of_mut!((*Arg).Integer.Value), argument_val as ULONG64);
        }
        x if x == ACPI_METHOD_ARGUMENT_STRING => {
            let data_length = read_unaligned(addr_of!((*Argument).DataLength)) as ULONG;
            // FIXME: ReactOS FIXME — should either default to empty string or reject.
            if data_length < core::mem::size_of::<UCHAR>() as ULONG {
                return STATUS_NOT_IMPLEMENTED;
            }

            write_unaligned(addr_of_mut!((*Arg).Type), ACPI_TYPE_STRING);
            write_unaligned(
                addr_of_mut!((*Arg).String.Pointer),
                ACPI_METHOD_ARGUMENT_DATA(Argument) as *mut c_char,
            );
            write_unaligned(
                addr_of_mut!((*Arg).String.Length),
                data_length - core::mem::size_of::<UCHAR>() as ULONG,
            );
        }
        x if x == ACPI_METHOD_ARGUMENT_BUFFER => {
            let data_length = read_unaligned(addr_of!((*Argument).DataLength)) as ULONG;
            write_unaligned(addr_of_mut!((*Arg).Type), ACPI_TYPE_BUFFER);
            write_unaligned(
                addr_of_mut!((*Arg).Buffer.Pointer),
                ACPI_METHOD_ARGUMENT_DATA(Argument),
            );
            write_unaligned(addr_of_mut!((*Arg).Buffer.Length), data_length);
        }
        x if x == ACPI_METHOD_ARGUMENT_PACKAGE => {
            write_unaligned(addr_of_mut!((*Arg).Type), ACPI_TYPE_PACKAGE);

            let data = ACPI_METHOD_ARGUMENT_DATA(Argument) as PACPI_METHOD_ARGUMENT;
            let data_length = read_unaligned(addr_of!((*Argument).DataLength)) as ULONG;
            let pkg_count = EvalGetPackageCount(Argument, data, data_length);
            write_unaligned(addr_of_mut!((*Arg).Package.Count), pkg_count);

            // Empty package, nothing more to convert.
            if pkg_count == 0 {
                write_unaligned(addr_of_mut!((*Arg).Package.Elements), core::ptr::null_mut());
                return STATUS_SUCCESS;
            }

            let mut PackageSize: ULONG = 0;
            let Status =
                RtlULongMult(pkg_count, core::mem::size_of::<ACPI_OBJECT>() as ULONG, &mut PackageSize);
            if !NT_SUCCESS(Status) {
                DPRINT1!("Invalid package count 0x%x\n", pkg_count);
                return STATUS_ACPI_INCORRECT_ARGUMENT_COUNT;
            }

            let elements = ExAllocatePoolUninitialized(
                NonPagedPool,
                PackageSize as SIZE_T,
                TAG_ACPI_PACKAGE_LIST,
            ) as *mut ACPI_OBJECT;
            write_unaligned(addr_of_mut!((*Arg).Package.Elements), elements);
            if elements.is_null() {
                return STATUS_INSUFFICIENT_RESOURCES;
            }

            let mut PackageArgument: PACPI_METHOD_ARGUMENT = data;
            let mut i: ULONG = 0;
            while i < pkg_count {
                let Status = EvalConvertParameterObjects(
                    elements.add(i as usize),
                    Depth + 1,
                    PackageArgument,
                    IoStack,
                    Offset,
                );
                if !NT_SUCCESS(Status) {
                    ExFreePoolWithTag(elements as PVOID, TAG_ACPI_PACKAGE_LIST);
                    return Status;
                }

                PackageArgument = ACPI_METHOD_NEXT_ARGUMENT(PackageArgument);
                i += 1;
            }
        }
        _ => {
            DPRINT1!("Unknown argument type %u\n", arg_type as core::ffi::c_uint);
            return STATUS_UNSUCCESSFUL;
        }
    }

    STATUS_SUCCESS
}

// ===========================================================================
//  EvalCreateParametersList (static → private)
// ===========================================================================
unsafe fn EvalCreateParametersList(
    Irp: PIRP,
    IoStack: PIO_STACK_LOCATION,
    EvalInputBuffer: PACPI_EVAL_INPUT_BUFFER,
    ParamList: *mut ACPI_OBJECT_LIST,
) -> NTSTATUS {
    // RTL_SIZEOF_THROUGH_FIELD(ACPI_EVAL_INPUT_BUFFER, Signature)
    //   = offsetof(Signature) + sizeof(Signature) = 0 + 4 = 4.
    let through_signature = core::mem::size_of::<ULONG>() as ULONG;
    if !AcpiVerifyInBuffer(IoStack, through_signature) {
        DPRINT1!("Buffer too small\n");
        return STATUS_INFO_LENGTH_MISMATCH;
    }

    let signature = (*EvalInputBuffer).Signature;
    match signature {
        s if s == ACPI_EVAL_INPUT_BUFFER_SIGNATURE => {
            if !AcpiVerifyInBuffer(IoStack, core::mem::size_of::<ACPI_EVAL_INPUT_BUFFER>() as ULONG) {
                DPRINT1!("Buffer too small\n");
                return STATUS_INFO_LENGTH_MISMATCH;
            }
            (*ParamList).Count = 0;
        }
        s if s == ACPI_EVAL_INPUT_BUFFER_SIMPLE_INTEGER_SIGNATURE => {
            if !AcpiVerifyInBuffer(
                IoStack,
                core::mem::size_of::<ACPI_EVAL_INPUT_BUFFER_SIMPLE_INTEGER>() as ULONG,
            ) {
                DPRINT1!("Buffer too small\n");
                return STATUS_INFO_LENGTH_MISMATCH;
            }

            let Arg = ExAllocatePoolUninitialized(
                NonPagedPool,
                core::mem::size_of::<ACPI_OBJECT>() as SIZE_T,
                TAG_ACPI_PARAMETERS_LIST,
            ) as *mut ACPI_OBJECT;
            if Arg.is_null() {
                return STATUS_INSUFFICIENT_RESOURCES;
            }

            (*ParamList).Count = 1;
            (*ParamList).Pointer = Arg;

            let SimpleInt =
                (*Irp).AssociatedIrp.SystemBuffer as *mut ACPI_EVAL_INPUT_BUFFER_SIMPLE_INTEGER;

            write_unaligned(addr_of_mut!((*Arg).Type), ACPI_TYPE_INTEGER);
            let int_arg = (*SimpleInt).IntegerArgument;
            write_unaligned(addr_of_mut!((*Arg).Integer.Value), int_arg as ULONG64);
        }
        s if s == ACPI_EVAL_INPUT_BUFFER_SIMPLE_STRING_SIGNATURE => {
            if !AcpiVerifyInBuffer(
                IoStack,
                core::mem::size_of::<ACPI_EVAL_INPUT_BUFFER_SIMPLE_STRING>() as ULONG,
            ) {
                DPRINT1!("Buffer too small\n");
                return STATUS_INFO_LENGTH_MISMATCH;
            }

            let Arg = ExAllocatePoolUninitialized(
                NonPagedPool,
                core::mem::size_of::<ACPI_OBJECT>() as SIZE_T,
                TAG_ACPI_PARAMETERS_LIST,
            ) as *mut ACPI_OBJECT;
            if Arg.is_null() {
                return STATUS_INSUFFICIENT_RESOURCES;
            }

            (*ParamList).Count = 1;
            (*ParamList).Pointer = Arg;

            let SimpleStr =
                (*Irp).AssociatedIrp.SystemBuffer as *mut ACPI_EVAL_INPUT_BUFFER_SIMPLE_STRING;

            write_unaligned(addr_of_mut!((*Arg).Type), ACPI_TYPE_STRING);
            write_unaligned(
                addr_of_mut!((*Arg).String.Pointer),
                addr_of_mut!((*SimpleStr).String) as *mut c_char,
            );
            let str_len = (*SimpleStr).StringLength;
            write_unaligned(addr_of_mut!((*Arg).String.Length), str_len);
        }
        s if s == ACPI_EVAL_INPUT_BUFFER_COMPLEX_SIGNATURE => {
            if !AcpiVerifyInBuffer(
                IoStack,
                core::mem::size_of::<ACPI_EVAL_INPUT_BUFFER_COMPLEX>() as ULONG,
            ) {
                DPRINT1!("Buffer too small\n");
                return STATUS_INFO_LENGTH_MISMATCH;
            }

            let ComplexBuffer =
                (*Irp).AssociatedIrp.SystemBuffer as *mut ACPI_EVAL_INPUT_BUFFER_COMPLEX;

            let count = (*ComplexBuffer).ArgumentCount;
            (*ParamList).Count = count;
            if count == 0 {
                DPRINT1!("No arguments\n");
                return STATUS_ACPI_INCORRECT_ARGUMENT_COUNT;
            }

            let mut ArgumentsSize: ULONG = 0;
            let Status =
                RtlULongMult(count, core::mem::size_of::<ACPI_OBJECT>() as ULONG, &mut ArgumentsSize);
            if !NT_SUCCESS(Status) {
                DPRINT1!("Invalid argument count 0x%x\n", count);
                return STATUS_ACPI_INCORRECT_ARGUMENT_COUNT;
            }

            let mut Arg = ExAllocatePoolUninitialized(
                NonPagedPool,
                ArgumentsSize as SIZE_T,
                TAG_ACPI_PARAMETERS_LIST,
            ) as *mut ACPI_OBJECT;
            if Arg.is_null() {
                return STATUS_INSUFFICIENT_RESOURCES;
            }

            (*ParamList).Pointer = Arg;

            let mut Argument: PACPI_METHOD_ARGUMENT =
                addr_of_mut!((*ComplexBuffer).Argument) as PACPI_METHOD_ARGUMENT;
            // FIELD_OFFSET(ACPI_EVAL_INPUT_BUFFER_COMPLEX, Argument) = 4+4+4+4 = 16.
            let mut Length: ULONG = field_offset_complex_argument();

            let mut i: ULONG = 0;
            while i < count {
                let Offset = Length;
                Length =
                    Length.wrapping_add(ACPI_METHOD_ARGUMENT_LENGTH_FROM_ARGUMENT(Argument));

                if !AcpiVerifyInBuffer(IoStack, Length) {
                    DPRINT1!("Argument buffer outside of argument bounds\n");
                    ExFreePoolWithTag((*ParamList).Pointer as PVOID, TAG_ACPI_PARAMETERS_LIST);
                    return STATUS_ACPI_INVALID_ARGTYPE;
                }

                let Status = EvalConvertParameterObjects(Arg, 0, Argument, IoStack, Offset);
                if !NT_SUCCESS(Status) {
                    ExFreePoolWithTag((*ParamList).Pointer as PVOID, TAG_ACPI_PARAMETERS_LIST);
                    return Status;
                }

                Arg = Arg.add(1);
                Argument = ACPI_METHOD_NEXT_ARGUMENT(Argument);
                i += 1;
            }
        }
        _ => {
            DPRINT1!("Unsupported input buffer signature: 0x%lx\n", signature);
            return STATUS_INVALID_PARAMETER_1;
        }
    }

    STATUS_SUCCESS
}

/// FIELD_OFFSET(ACPI_EVAL_INPUT_BUFFER_COMPLEX, Argument) = 16
/// (Signature 4 + MethodName 4 + Size 4 + ArgumentCount 4).
#[inline(always)]
fn field_offset_complex_argument() -> ULONG {
    core::mem::offset_of!(ACPI_EVAL_INPUT_BUFFER_COMPLEX, Argument) as ULONG
}

/// FIELD_OFFSET(ACPI_EVAL_OUTPUT_BUFFER, Argument) = 12
/// (Signature 4 + Length 4 + Count 4).
#[inline(always)]
fn field_offset_output_argument() -> ULONG {
    core::mem::offset_of!(ACPI_EVAL_OUTPUT_BUFFER, Argument) as ULONG
}

// ===========================================================================
//  EvalFreeParameterArgument (static → private, recursive)
// ===========================================================================
unsafe fn EvalFreeParameterArgument(Arg: *mut ACPI_OBJECT, Depth: ULONG) {
    if Depth >= ACPI_MAX_PACKAGE_DEPTH {
        return;
    }

    let arg_type = read_unaligned(addr_of!((*Arg).Type));
    if arg_type == ACPI_TYPE_PACKAGE {
        let pkg_count = read_unaligned(addr_of!((*Arg).Package.Count));
        let elements = read_unaligned(addr_of!((*Arg).Package.Elements));

        let mut i: ULONG = 0;
        while i < pkg_count {
            EvalFreeParameterArgument(elements.add(i as usize), Depth + 1);
            i += 1;
        }

        // Check if the package isn't empty, and free it.
        if !elements.is_null() {
            ExFreePoolWithTag(elements as PVOID, TAG_ACPI_PACKAGE_LIST);
        }
    }
}

// ===========================================================================
//  EvalFreeParametersList (static → private)
// ===========================================================================
unsafe fn EvalFreeParametersList(ParamList: *mut ACPI_OBJECT_LIST) {
    let mut Arg = (*ParamList).Pointer;
    let count = (*ParamList).Count;
    let mut i: ULONG = 0;
    while i < count {
        EvalFreeParameterArgument(Arg, 0);
        Arg = Arg.add(1);
        i += 1;
    }

    ExFreePoolWithTag((*ParamList).Pointer as PVOID, TAG_ACPI_PARAMETERS_LIST);
}

// ===========================================================================
//  EvalAcpiStatusToNtStatus (static → private)
// ===========================================================================
unsafe fn EvalAcpiStatusToNtStatus(AcpiStatus: ACPI_STATUS) -> NTSTATUS {
    if ACPI_ENV_EXCEPTION(AcpiStatus) {
        match AcpiStatus {
            x if x == AE_NOT_FOUND || x == AE_NOT_EXIST => return STATUS_OBJECT_NAME_NOT_FOUND,
            x if x == AE_NO_MEMORY => return STATUS_INSUFFICIENT_RESOURCES,
            x if x == AE_SUPPORT => return STATUS_ACPI_INCORRECT_ARGUMENT_COUNT,
            x if x == AE_TIME => return STATUS_IO_TIMEOUT,
            x if x == AE_NO_HARDWARE_RESPONSE => return STATUS_IO_DEVICE_ERROR,
            x if x == AE_STACK_OVERFLOW => return STATUS_ACPI_STACK_OVERFLOW,
            _ => return STATUS_UNSUCCESSFUL,
        }
    }

    if ACPI_AML_EXCEPTION(AcpiStatus) {
        match AcpiStatus {
            x if x == AE_AML_UNINITIALIZED_ARG => return STATUS_ACPI_INCORRECT_ARGUMENT_COUNT,
            _ => return STATUS_UNSUCCESSFUL,
        }
    }

    STATUS_UNSUCCESSFUL
}

// ===========================================================================
//  EvalEvaluateObject (static → private)
// ===========================================================================
unsafe fn EvalEvaluateObject(
    DeviceData: PPDO_DEVICE_DATA,
    EvalInputBuffer: PACPI_EVAL_INPUT_BUFFER,
    ParamList: *mut ACPI_OBJECT_LIST,
    ReturnBuffer: *mut ACPI_BUFFER,
) -> ACPI_STATUS {
    let mut MethodName: [c_char; ACPI_OBJECT_NAME_LENGTH as usize] =
        [0; ACPI_OBJECT_NAME_LENGTH as usize];

    // RtlCopyMemory(MethodName, EvalInputBuffer->MethodName, ACPI_OBJECT_NAME_LENGTH - 1);
    RtlCopyMemory(
        MethodName.as_mut_ptr() as *mut c_void,
        addr_of!((*EvalInputBuffer).MethodName) as *const c_void,
        (ACPI_OBJECT_NAME_LENGTH - 1) as usize,
    );
    MethodName[(ACPI_OBJECT_NAME_LENGTH - 1) as usize] = ANSI_NULL;

    AcpiEvaluateObject(
        (*DeviceData).AcpiHandle,
        MethodName.as_mut_ptr() as ACPI_STRING,
        ParamList,
        ReturnBuffer,
    )
}

// ===========================================================================
//  EvalCreateOutputArguments (static → private)
// ===========================================================================
unsafe fn EvalCreateOutputArguments(
    Irp: PIRP,
    IoStack: PIO_STACK_LOCATION,
    ReturnBuffer: *mut ACPI_BUFFER,
) -> NTSTATUS {
    // If we didn't get anything back then we're done.
    if (*ReturnBuffer).Pointer.is_null() || (*ReturnBuffer).Length == 0 {
        return STATUS_SUCCESS;
    }

    // No output buffer is provided, we're done.
    if (*IoStack).Parameters.DeviceIoControl.OutputBufferLength == 0 {
        return STATUS_SUCCESS;
    }

    if !AcpiVerifyOutBuffer(IoStack, core::mem::size_of::<ACPI_EVAL_OUTPUT_BUFFER>() as ULONG) {
        DPRINT1!("Buffer too small\n");
        return STATUS_BUFFER_TOO_SMALL;
    }

    let Obj = (*ReturnBuffer).Pointer as *mut ACPI_OBJECT;

    let mut Count: ULONG = 0;
    let mut ExtraParamLength: ULONG = 0;
    let Status = EvalGetElementSize(Obj, 0, &mut Count, &mut ExtraParamLength);
    if !NT_SUCCESS(Status) {
        return Status;
    }

    let OutputBufSize = field_offset_output_argument().wrapping_add(ExtraParamLength);

    // #else (drop UNIT_TEST path): OutputBuffer = Irp->AssociatedIrp.SystemBuffer.
    let OutputBuffer = (*Irp).AssociatedIrp.SystemBuffer as *mut ACPI_EVAL_OUTPUT_BUFFER;
    write_unaligned(addr_of_mut!((*OutputBuffer).Signature), ACPI_EVAL_OUTPUT_BUFFER_SIGNATURE);
    write_unaligned(addr_of_mut!((*OutputBuffer).Length), OutputBufSize);
    write_unaligned(addr_of_mut!((*OutputBuffer).Count), Count);

    if !AcpiVerifyOutBuffer(IoStack, OutputBufSize) {
        DPRINT!();
        (*Irp).IoStatus.Information = OutputBufSize as _;
        return STATUS_BUFFER_OVERFLOW;
    }

    let Status = EvalConvertEvaluationResults(
        addr_of_mut!((*OutputBuffer).Argument) as PACPI_METHOD_ARGUMENT,
        0,
        Obj,
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }

    (*Irp).IoStatus.Information = OutputBufSize as _;
    STATUS_SUCCESS
}

// ===========================================================================
//  Bus_PDO_EvalMethod (external linkage, NTAPI → extern "system")
// ===========================================================================
#[no_mangle]
pub unsafe extern "system" fn Bus_PDO_EvalMethod(
    DeviceData: PPDO_DEVICE_DATA,
    Irp: PIRP,
) -> NTSTATUS {
    let mut ParamList: ACPI_OBJECT_LIST = core::mem::zeroed();
    let mut ReturnBuffer: ACPI_BUFFER = ACPI_BUFFER {
        Length: ACPI_ALLOCATE_BUFFER,
        Pointer: core::ptr::null_mut(),
    };

    let IoStack = IoGetCurrentIrpStackLocation(Irp);
    let EvalInputBuffer = (*Irp).AssociatedIrp.SystemBuffer as PACPI_EVAL_INPUT_BUFFER;

    let Status =
        EvalCreateParametersList(Irp, IoStack, EvalInputBuffer, &mut ParamList);
    if !NT_SUCCESS(Status) {
        return Status;
    }

    let AcpiStatus = EvalEvaluateObject(DeviceData, EvalInputBuffer, &mut ParamList, &mut ReturnBuffer);

    if ParamList.Count != 0 {
        EvalFreeParametersList(&mut ParamList);
    }

    if !ACPI_SUCCESS(AcpiStatus) {
        DPRINT!();
        return EvalAcpiStatusToNtStatus(AcpiStatus);
    }

    let Status = EvalCreateOutputArguments(Irp, IoStack, &mut ReturnBuffer);

    if !ReturnBuffer.Pointer.is_null() {
        AcpiOsFree(ReturnBuffer.Pointer);
    }

    Status
}
