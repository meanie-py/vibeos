//! utils.c → Rust (ACPI Phase 3b glue leaf). Cohab: #[no_mangle] symbols
//! replace the C file in acpi.sys at link time. ACPICA stays C (libacpica.a).
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals, unused)]

// NOTE: we deliberately do NOT `use crate::shared::*` here. The bindgen
// substrate (crate::types, re-exported by crate::shared) already *declares*
// `acpi_extract_package`, `acpi_evaluate_integer`, `acpi_evaluate_reference`,
// and `acpi_create_volatile_registry_tables` as `extern "C"` fns — glob-
// importing them would collide with the `#[no_mangle]` *definitions* we provide
// in this file (E0255). So we import the concrete ACPICA / device types we need
// individually instead.
use core::ffi::{c_char, c_int, c_void};
use core::ptr;

// --- ACPICA types + constants from acpica-sys (re-exported via crate::types) ---
use crate::types::{
    acpi_handle_list, ACPI_BUFFER, ACPI_HANDLE, ACPI_INTEGER, ACPI_MAX_HANDLES, ACPI_OBJECT,
    ACPI_OBJECT_LIST, ACPI_PHYSICAL_ADDRESS, ACPI_STATUS, ACPI_STRING, ACPI_TABLE_HEADER,
    ACPI_TABLE_RSDP,
};
// ACPI object-type discriminants and ACPI_FULL_PATHNAME come straight from acpica-sys.
use acpica_sys::{
    ACPI_FULL_PATHNAME, ACPI_TYPE_BUFFER, ACPI_TYPE_INTEGER, ACPI_TYPE_LOCAL_REFERENCE,
    ACPI_TYPE_PACKAGE, ACPI_TYPE_STRING,
};
// ACPICA API functions this file calls (outbound, all extern "C").
use acpica_sys::{
    AcpiEvaluateObject, AcpiGetTable, AcpiOsFree, AcpiOsGetRootPointer, AcpiOsMapMemory,
    AcpiOsUnmapMemory,
};

// --- Kernel types / functions from ntkrnl-sys ---
use ntkrnl_sys::base::{NTSTATUS, STATUS_NO_MEMORY, STATUS_SUCCESS};
use ntkrnl_sys::constants::STATUS_UNSUCCESSFUL;
use ntkrnl_sys::ddk::{
    ANSI_STRING, BOOLEAN, HANDLE, OBJECT_ATTRIBUTES, PVOID, UNICODE_STRING, ULONG, USHORT,
    RtlAnsiStringToUnicodeString, RtlFreeUnicodeString, RtlInitAnsiString, RtlInitUnicodeString,
    ZwClose, ZwCreateKey, ZwSetValueKey, PCWSTR,
};

// ---- ACPI exception codes (acexcep.h). AE_OK/BAD_PARAMETER/BUFFER_OVERFLOW are
//      in crate::shared, but we avoid that glob (see top), so define the ones we
//      use locally. EXCEP_ENV(c)=c|0x0000, EXCEP_PGM(c)=c|0x1000. Values match
//      acpi-rust/src/shared.rs (verified there). ----
const AE_OK: ACPI_STATUS = 0x0000;
const AE_BUFFER_OVERFLOW: ACPI_STATUS = 0x000B; // EXCEP_ENV(0x000B)
const AE_BAD_PARAMETER: ACPI_STATUS = 0x1001; // EXCEP_PGM(0x0001)
// FIXME(integrator): confirm against acexcep.h (acexcep.h EXCEP_ENV scheme).
const AE_BAD_DATA: ACPI_STATUS = 0x0003; // EXCEP_ENV(0x0003)
const AE_NO_MEMORY: ACPI_STATUS = 0x0004; // EXCEP_ENV(0x0004)
const AE_SUPPORT: ACPI_STATUS = 0xF; // acpica/include/acexcep.h EXCEP_ENV(0x000F), AE_CODE_ENVIRONMENTAL=0
const AE_NULL_ENTRY: ACPI_STATUS = 0x001F; // EXCEP_ENV(0x001F)

// ACPI_ALLOCATE_BUFFER = (ACPI_SIZE) -1 (acpixf.h). ACPI_SIZE follows the C build's
// ACPI_MACHINE_WIDTH: 64 on amd64, 32 on i386 and arm64 (acpica-sys's
// generate_bindings.sh). Build it from the field type so it is right on each.

// Registry / object-attribute constants — NOT present in ntkrnl_sys::ddk for the
// driver build (only in hal_bindgen). Values from the brief and verified against
// hal_bindgen (KEY_WRITE=131078=0x20006, REG_OPTION_VOLATILE=1, REG_BINARY=3,
// OBJ_CASE_INSENSITIVE=0x40, OBJ_KERNEL_HANDLE=0x200).
// FIXME(integrator): if these land in ntkrnl_sys::ddk/constants, import instead.
const KEY_WRITE: ULONG = 0x20006;
const REG_OPTION_VOLATILE: ULONG = 1;
const REG_BINARY: ULONG = 3;
const OBJ_CASE_INSENSITIVE: ULONG = 0x40;
const OBJ_KERNEL_HANDLE: ULONG = 0x200;

// ACPICA root-table entry sizes (actbl.h): XSDT entries are 8 bytes (64-bit
// physical addresses), RSDT entries are 4 bytes (32-bit).
// FIXME(integrator): confirm ACPI_XSDT_ENTRY_SIZE/ACPI_RSDT_ENTRY_SIZE.
const ACPI_XSDT_ENTRY_SIZE: u32 = 8;
const ACPI_RSDT_ENTRY_SIZE: u32 = 4;

// AcpiGbl_DoNotUseXsdt — ACPICA exported global (BOOLEAN).
extern "C" {
    static AcpiGbl_DoNotUseXsdt: BOOLEAN;
}

// RtlStringCbPrintfW is a FORCEINLINE in ntstrsafe.h (no export). The sole use
// formats a bounded "%08X" into a 9-WCHAR buffer → use exported C-variadic swprintf.
extern "C" {
    fn swprintf(buffer: *mut u16, format: *const u16, ...) -> i32;
}

// ---- inline helpers (ACPI_SUCCESS/FAILURE/NT_SUCCESS) ----
#[inline(always)]
unsafe fn ACPI_FAILURE(s: ACPI_STATUS) -> bool {
    s != AE_OK
}
#[inline(always)]
unsafe fn NT_SUCCESS(s: NTSTATUS) -> bool {
    s >= 0
}

// ===========================================================================
//  acpi_util_eval_error  (static → private)
//  Under ACPI_DEBUG_OUTPUT it formats + prints a name; that macro is NOT
//  defined in the driver's NDEBUG build, so the C body is just `return;`.
// ===========================================================================
unsafe fn acpi_util_eval_error(_h: ACPI_HANDLE, _p: ACPI_STRING, _s: ACPI_STATUS) {
    // #ifndef ACPI_DEBUG_OUTPUT → return;
}

// ===========================================================================
//  acpi_extract_package  (external, plain C → extern "C")
// ===========================================================================
#[no_mangle]
pub unsafe extern "C" fn acpi_extract_package(
    package: *mut ACPI_OBJECT,
    format: *mut ACPI_BUFFER,
    buffer: *mut ACPI_BUFFER,
) -> ACPI_STATUS {
    let mut size_required: u32 = 0;
    let mut tail_offset: u32 = 0;
    let mut format_string: *mut c_char = ptr::null_mut();
    let mut format_count: u32 = 0;
    let mut i: u32;
    let mut head: *mut u8;
    let mut tail: *mut u8;

    // sizeof(ACPI_INTEGER) == 8; sizeof(char*)/sizeof(UINT8*) == pointer width.
    let sizeof_acpi_integer = core::mem::size_of::<ACPI_INTEGER>() as u32;
    let sizeof_ptr = core::mem::size_of::<*mut u8>() as u32;
    let sizeof_char = core::mem::size_of::<c_char>() as u32;
    let sizeof_uint8 = 1u32;

    if package.is_null()
        || (*package).Type != ACPI_TYPE_PACKAGE
        || (*package).Package.Count < 1
    {
        return AE_BAD_PARAMETER;
    }

    if format.is_null() || (*format).Pointer.is_null() || (*format).Length < 1 {
        return AE_BAD_PARAMETER;
    }

    if buffer.is_null() {
        return AE_BAD_PARAMETER;
    }

    // format_count = (format->Length / sizeof(char)) - 1;
    format_count = ((*format).Length as u32 / sizeof_char).wrapping_sub(1);
    if format_count > (*package).Package.Count {
        return AE_BAD_DATA;
    }

    format_string = (*format).Pointer as *mut c_char;

    // --- Calculate size_required. ---
    i = 0;
    while i < format_count {
        // ACPI_OBJECT *element = &(package->Package.Elements[i]);
        // flexible-array → pointer arithmetic, NOT [i] indexing.
        let element: *mut ACPI_OBJECT = (*package).Package.Elements.add(i as usize);

        // element is never null here (it's &arr[i]); C checks anyway.
        if element.is_null() {
            return AE_BAD_DATA;
        }

        let fmt = *format_string.add(i as usize) as u8 as char;

        match (*element).Type {
            t if t == ACPI_TYPE_INTEGER => match fmt {
                'N' => {
                    size_required = size_required.wrapping_add(sizeof_acpi_integer);
                    tail_offset = tail_offset.wrapping_add(sizeof_acpi_integer);
                }
                'S' => {
                    size_required = size_required
                        .wrapping_add(sizeof_ptr)
                        .wrapping_add(sizeof_acpi_integer)
                        .wrapping_add(sizeof_char);
                    tail_offset = tail_offset.wrapping_add(sizeof_ptr);
                }
                _ => {
                    return AE_BAD_DATA;
                }
            },
            t if t == ACPI_TYPE_STRING || t == ACPI_TYPE_BUFFER => match fmt {
                'S' => {
                    // sizeof(char*) + (element->String.Length * sizeof(char)) + sizeof(char)
                    size_required = size_required
                        .wrapping_add(sizeof_ptr)
                        .wrapping_add((*element).String.Length.wrapping_mul(sizeof_char))
                        .wrapping_add(sizeof_char);
                    tail_offset = tail_offset.wrapping_add(sizeof_ptr);
                }
                'B' => {
                    // sizeof(UINT8*) + (element->Buffer.Length * sizeof(UINT8))
                    size_required = size_required
                        .wrapping_add(sizeof_ptr)
                        .wrapping_add((*element).Buffer.Length.wrapping_mul(sizeof_uint8));
                    tail_offset = tail_offset.wrapping_add(sizeof_ptr);
                }
                _ => {
                    return AE_BAD_DATA;
                }
            },
            // ACPI_TYPE_PACKAGE and default → unsupported.
            _ => {
                return AE_SUPPORT;
            }
        }

        i = i.wrapping_add(1);
    }

    // --- Validate output buffer. ---
    // ACPI_BUFFER.Length is ACPI_SIZE (u64 on amd64, u32 on i386/arm64). size_required
    // is u32; compare/store with the field's native width.
    let size_required_sz = size_required as u64;
    let buf_len = (*buffer).Length as u64;
    if buf_len < size_required_sz {
        (*buffer).Length = size_required as _;
        return AE_BUFFER_OVERFLOW;
    } else if buf_len != size_required_sz || (*buffer).Pointer.is_null() {
        return AE_BAD_PARAMETER;
    }

    head = (*buffer).Pointer as *mut u8;
    tail = ((*buffer).Pointer as *mut u8).add(tail_offset as usize);

    // --- Extract package data. ---
    i = 0;
    while i < format_count {
        let element: *mut ACPI_OBJECT = (*package).Package.Elements.add(i as usize);

        if element.is_null() {
            return AE_BAD_DATA;
        }

        let fmt = *format_string.add(i as usize) as u8 as char;

        match (*element).Type {
            t if t == ACPI_TYPE_INTEGER => match fmt {
                'N' => {
                    // *((ACPI_INTEGER*)head) = element->Integer.Value;
                    ptr::write_unaligned(head as *mut ACPI_INTEGER, (*element).Integer.Value);
                    head = head.add(sizeof_acpi_integer as usize);
                }
                'S' => {
                    // pointer = (UINT8**)head; *pointer = tail;
                    ptr::write_unaligned(head as *mut *mut u8, tail);
                    // *((ACPI_INTEGER*)tail) = element->Integer.Value;
                    ptr::write_unaligned(tail as *mut ACPI_INTEGER, (*element).Integer.Value);
                    // head += sizeof(ACPI_INTEGER*);  (NOTE: C uses pointer size here)
                    head = head.add(sizeof_ptr as usize);
                    // tail += sizeof(ACPI_INTEGER);
                    tail = tail.add(sizeof_acpi_integer as usize);
                    // NULL terminate string: *tail = (char)0; tail += sizeof(char);
                    ptr::write_unaligned(tail, 0u8);
                    tail = tail.add(sizeof_char as usize);
                }
                _ => { /* Should never get here */ }
            },
            t if t == ACPI_TYPE_STRING || t == ACPI_TYPE_BUFFER => match fmt {
                'S' => {
                    ptr::write_unaligned(head as *mut *mut u8, tail);
                    // memcpy(tail, element->String.Pointer, element->String.Length);
                    ptr::copy_nonoverlapping(
                        (*element).String.Pointer as *const u8,
                        tail,
                        (*element).String.Length as usize,
                    );
                    head = head.add(sizeof_ptr as usize);
                    tail = tail.add(((*element).String.Length.wrapping_mul(sizeof_char)) as usize);
                    // NULL terminate string
                    ptr::write_unaligned(tail, 0u8);
                    tail = tail.add(sizeof_char as usize);
                }
                'B' => {
                    ptr::write_unaligned(head as *mut *mut u8, tail);
                    // memcpy(tail, element->Buffer.Pointer, element->Buffer.Length);
                    ptr::copy_nonoverlapping(
                        (*element).Buffer.Pointer as *const u8,
                        tail,
                        (*element).Buffer.Length as usize,
                    );
                    head = head.add(sizeof_ptr as usize);
                    tail = tail.add(((*element).Buffer.Length.wrapping_mul(sizeof_uint8)) as usize);
                }
                _ => { /* Should never get here */ }
            },
            // ACPI_TYPE_PACKAGE / default → nothing.
            _ => { /* Should never get here */ }
        }

        i = i.wrapping_add(1);
    }

    AE_OK
}

// ===========================================================================
//  acpi_evaluate_integer  (external, plain C → extern "C")
// ===========================================================================
#[no_mangle]
pub unsafe extern "C" fn acpi_evaluate_integer(
    handle: ACPI_HANDLE,
    pathname: ACPI_STRING,
    arguments: *mut ACPI_OBJECT_LIST,
    data: *mut u64,
) -> ACPI_STATUS {
    let mut status: ACPI_STATUS;
    let mut element: ACPI_OBJECT = core::mem::zeroed();
    // ACPI_BUFFER buffer = { sizeof(ACPI_OBJECT), &element };
    let mut buffer = ACPI_BUFFER {
        Length: core::mem::size_of::<ACPI_OBJECT>() as _,
        Pointer: &mut element as *mut ACPI_OBJECT as *mut c_void,
    };

    if data.is_null() {
        return AE_BAD_PARAMETER;
    }

    status = AcpiEvaluateObject(handle, pathname, arguments, &mut buffer);
    if ACPI_FAILURE(status) {
        acpi_util_eval_error(handle, pathname, status);
        return status;
    }

    if element.Type != ACPI_TYPE_INTEGER {
        acpi_util_eval_error(handle, pathname, AE_BAD_DATA);
        return AE_BAD_DATA;
    }

    // *data = element.Integer.Value;  (final out-param store → write_volatile)
    ptr::write_volatile(data, element.Integer.Value);

    AE_OK
}

// ===========================================================================
//  acpi_evaluate_reference  (external, plain C → extern "C")
// ===========================================================================
#[no_mangle]
pub unsafe extern "C" fn acpi_evaluate_reference(
    handle: ACPI_HANDLE,
    pathname: ACPI_STRING,
    arguments: *mut ACPI_OBJECT_LIST,
    list: *mut acpi_handle_list,
) -> ACPI_STATUS {
    let mut status: ACPI_STATUS = AE_OK;
    let mut package: *mut ACPI_OBJECT = ptr::null_mut();
    let mut element: *mut ACPI_OBJECT;
    // ACPI_BUFFER buffer = { ACPI_ALLOCATE_BUFFER, NULL };
    // ACPI_ALLOCATE_BUFFER == (ACPI_SIZE)-1 — build at field width via `!0`.
    let mut buffer = ACPI_BUFFER {
        Length: !0 as _, // (ACPI_SIZE)-1 == ACPI_ALLOCATE_BUFFER
        Pointer: ptr::null_mut(),
    };
    let mut i: u32;

    if list.is_null() {
        return AE_BAD_PARAMETER;
    }

    // Evaluate object.
    status = AcpiEvaluateObject(handle, pathname, arguments, &mut buffer);
    if ACPI_FAILURE(status) {
        // goto end;
        return finish_reference(list, &mut buffer, status);
    }

    package = buffer.Pointer as *mut ACPI_OBJECT;

    if buffer.Length == 0 || package.is_null() {
        status = AE_BAD_DATA;
        acpi_util_eval_error(handle, pathname, status);
        return finish_reference(list, &mut buffer, status);
    }
    if (*package).Type != ACPI_TYPE_PACKAGE {
        status = AE_BAD_DATA;
        acpi_util_eval_error(handle, pathname, status);
        return finish_reference(list, &mut buffer, status);
    }
    if (*package).Package.Count == 0 {
        status = AE_BAD_DATA;
        acpi_util_eval_error(handle, pathname, status);
        return finish_reference(list, &mut buffer, status);
    }

    if (*package).Package.Count > ACPI_MAX_HANDLES {
        // NOTE: C returns AE_NO_MEMORY directly here, WITHOUT going through the
        // `end:` cleanup (no AcpiOsFree, no list->count=0). Faithful: but buffer
        // was allocated by ACPICA → leaks per the original C. Preserve behavior.
        return AE_NO_MEMORY;
    }
    (*list).count = (*package).Package.Count;

    // Extract package data.
    i = 0;
    while i < (*list).count {
        // element = &(package->Package.Elements[i]); flexible array → ptr math.
        element = (*package).Package.Elements.add(i as usize);

        if (*element).Type != ACPI_TYPE_LOCAL_REFERENCE {
            status = AE_BAD_DATA;
            acpi_util_eval_error(handle, pathname, status);
            break;
        }

        if (*element).Reference.Handle.is_null() {
            status = AE_NULL_ENTRY;
            break;
        }
        // Get the ACPI_HANDLE.
        // list->handles[i] = element->Reference.Handle;
        // handles is [ACPI_HANDLE; 10]; i < count <= ACPI_MAX_HANDLES(10) → in bounds.
        let handles_base = (*list).handles.as_mut_ptr();
        ptr::write(handles_base.add(i as usize), (*element).Reference.Handle);

        i = i.wrapping_add(1);
    }

    finish_reference(list, &mut buffer, status)
}

// `end:` label of acpi_evaluate_reference, factored out so each `goto end`
// becomes a tail return. Frees the ACPICA buffer and zeroes list->count on
// failure, then returns the status.
#[inline(always)]
unsafe fn finish_reference(
    list: *mut acpi_handle_list,
    buffer: *mut ACPI_BUFFER,
    status: ACPI_STATUS,
) -> ACPI_STATUS {
    if ACPI_FAILURE(status) {
        if !list.is_null() {
            (*list).count = 0;
        }
        // C comment: //ExFreePool(list->handles);  (left out in C)
    }

    if !(*buffer).Pointer.is_null() {
        AcpiOsFree((*buffer).Pointer);
    }

    status
}

// ===========================================================================
//  acpi_create_registry_table  (static → private). NTSTATUS, plain C.
// ===========================================================================
unsafe fn acpi_create_registry_table(
    ParentKeyHandle: HANDLE,
    OutTable: *mut ACPI_TABLE_HEADER,
    KeyName: PCWSTR,
) -> NTSTATUS {
    let mut ObjectAttributes: OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut HardwareKeyName: UNICODE_STRING = core::mem::zeroed();
    let mut ValueName: UNICODE_STRING = core::mem::zeroed();
    let mut HardwareKeyNameA: ANSI_STRING = core::mem::zeroed();
    let mut KeyHandle: HANDLE = ptr::null_mut();
    let mut SubKeyHandle: HANDLE = ptr::null_mut();
    let mut Status: NTSTATUS;

    // char OemId[7] = {0}; char OemTableId[9] = {0}; WCHAR OemRevision[9] = {0};
    let mut OemId: [c_char; 7] = [0; 7];
    let mut OemTableId: [c_char; 9] = [0; 9];
    let mut OemRevision: [u16; 9] = [0; 9];

    // C_ASSERT(...) → no-op.

    // RtlCopyMemory(OemId, OutTable->OemId, sizeof(OutTable->OemId)); (6 bytes)
    ptr::copy_nonoverlapping( // nolint: silent-failure (byte-array pointee — count==bytes)
        (*OutTable).OemId.as_ptr(),
        OemId.as_mut_ptr(),
        core::mem::size_of_val(&(*OutTable).OemId),
    );
    // RtlCopyMemory(OemTableId, OutTable->OemTableId, sizeof(OutTable->OemTableId)); (8 bytes)
    ptr::copy_nonoverlapping( // nolint: silent-failure (byte-array pointee — count==bytes)
        (*OutTable).OemTableId.as_ptr(),
        OemTableId.as_mut_ptr(),
        core::mem::size_of_val(&(*OutTable).OemTableId),
    );

    // Create table subkey
    RtlInitUnicodeString(&mut HardwareKeyName, KeyName);
    initialize_object_attributes(
        &mut ObjectAttributes,
        &mut HardwareKeyName,
        OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE,
        ParentKeyHandle,
        ptr::null_mut(),
    );
    Status = ZwCreateKey(
        &mut KeyHandle,
        KEY_WRITE,
        &mut ObjectAttributes,
        0,
        ptr::null_mut(),
        REG_OPTION_VOLATILE,
        ptr::null_mut(),
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }

    // if (OutTable->OemRevision != 0)  — OemRevision is UINT32 in a non-packed
    // header; plain read is fine, but use read_unaligned to be safe.
    let oem_revision = ptr::read_unaligned(ptr::addr_of!((*OutTable).OemRevision));
    if oem_revision != 0 {
        // OEM subkey 1: OemId
        RtlInitAnsiString(&mut HardwareKeyNameA, OemId.as_ptr());
        Status = RtlAnsiStringToUnicodeString(&mut HardwareKeyName, &mut HardwareKeyNameA, 1 /*TRUE*/);
        if !NT_SUCCESS(Status) {
            ZwClose(KeyHandle);
            return Status;
        }

        initialize_object_attributes(
            &mut ObjectAttributes,
            &mut HardwareKeyName,
            OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE,
            KeyHandle,
            ptr::null_mut(),
        );
        Status = ZwCreateKey(
            &mut SubKeyHandle,
            KEY_WRITE,
            &mut ObjectAttributes,
            0,
            ptr::null_mut(),
            REG_OPTION_VOLATILE,
            ptr::null_mut(),
        );
        RtlFreeUnicodeString(&mut HardwareKeyName);
        ZwClose(KeyHandle);
        if !NT_SUCCESS(Status) {
            return Status;
        }
        KeyHandle = SubKeyHandle;

        // OEM subkey 2: OemTableId
        RtlInitAnsiString(&mut HardwareKeyNameA, OemTableId.as_ptr());
        Status = RtlAnsiStringToUnicodeString(&mut HardwareKeyName, &mut HardwareKeyNameA, 1 /*TRUE*/);
        if !NT_SUCCESS(Status) {
            ZwClose(KeyHandle);
            return Status;
        }

        initialize_object_attributes(
            &mut ObjectAttributes,
            &mut HardwareKeyName,
            OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE,
            KeyHandle,
            ptr::null_mut(),
        );
        Status = ZwCreateKey(
            &mut SubKeyHandle,
            KEY_WRITE,
            &mut ObjectAttributes,
            0,
            ptr::null_mut(),
            REG_OPTION_VOLATILE,
            ptr::null_mut(),
        );
        RtlFreeUnicodeString(&mut HardwareKeyName);
        ZwClose(KeyHandle);
        if !NT_SUCCESS(Status) {
            return Status;
        }
        KeyHandle = SubKeyHandle;

        // OEM subkey 3: OemRevision as "%08X" (fits the 9-WCHAR buffer).
        swprintf(
            OemRevision.as_mut_ptr(),
            ntkrnl_sys::w!("%08X").as_ptr(),
            oem_revision,
        );
        Status = STATUS_SUCCESS;
        if !NT_SUCCESS(Status) {
            ZwClose(KeyHandle);
            return Status;
        }
        RtlInitUnicodeString(&mut HardwareKeyName, OemRevision.as_ptr());

        initialize_object_attributes(
            &mut ObjectAttributes,
            &mut HardwareKeyName,
            OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE,
            KeyHandle,
            ptr::null_mut(),
        );
        Status = ZwCreateKey(
            &mut SubKeyHandle,
            KEY_WRITE,
            &mut ObjectAttributes,
            0,
            ptr::null_mut(),
            REG_OPTION_VOLATILE,
            ptr::null_mut(),
        );
        ZwClose(KeyHandle);
        if !NT_SUCCESS(Status) {
            return Status;
        }
        KeyHandle = SubKeyHandle;
    }

    // Table reg value name is always '00000000'
    RtlInitUnicodeString(&mut ValueName, ntkrnl_sys::w!("00000000").as_ptr());
    let table_len = ptr::read_unaligned(ptr::addr_of!((*OutTable).Length));
    Status = ZwSetValueKey(
        KeyHandle,
        &mut ValueName,
        0,
        REG_BINARY,
        OutTable as PVOID,
        table_len,
    );
    ZwClose(KeyHandle);
    if !NT_SUCCESS(Status) {
        return Status;
    }

    STATUS_SUCCESS
}

// ===========================================================================
//  acpi_create_volatile_registry_tables  (external, plain C → extern "C")
// ===========================================================================
#[no_mangle]
pub unsafe extern "C" fn acpi_create_volatile_registry_tables() -> NTSTATUS {
    let mut ObjectAttributes: OBJECT_ATTRIBUTES = core::mem::zeroed();
    // UNICODE_STRING HardwareKeyName = RTL_CONSTANT_STRING(L"\\Registry\\Machine\\HARDWARE\\ACPI");
    let hw_path: &[u16] = ntkrnl_sys::w!("\\Registry\\Machine\\HARDWARE\\ACPI");
    // RTL_CONSTANT_STRING: Length = byte-length excluding NUL; MaximumLength = +NUL.
    let hw_chars = hw_path.len() - 1; // exclude trailing NUL
    let mut HardwareKeyName = UNICODE_STRING {
        Length: (hw_chars * 2) as USHORT,
        MaximumLength: (hw_path.len() * 2) as USHORT,
        Buffer: hw_path.as_ptr() as *mut u16,
    };
    let mut KeyHandle: HANDLE = ptr::null_mut();
    let mut Status: NTSTATUS;
    let mut AcpiStatus: ACPI_STATUS;
    let mut OutTable: *mut ACPI_TABLE_HEADER = ptr::null_mut();
    let mut RsdpAddress: ACPI_PHYSICAL_ADDRESS;
    let mut Rsdp: *mut ACPI_TABLE_RSDP;
    let mut Address: ACPI_PHYSICAL_ADDRESS;
    let mut TableEntrySize: u32;

    // Create Main Hardware ACPI key
    initialize_object_attributes(
        &mut ObjectAttributes,
        &mut HardwareKeyName,
        OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE,
        ptr::null_mut(),
        ptr::null_mut(),
    );
    Status = ZwCreateKey(
        &mut KeyHandle,
        KEY_WRITE,
        &mut ObjectAttributes,
        0,
        ptr::null_mut(),
        REG_OPTION_VOLATILE,
        ptr::null_mut(),
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }

    // AcpiGetTable signature is ACPI_STRING (*mut c_char). ACPI_SIG_* are
    // byte-string consts (b"DSDT\0"); pass an explicit c-string pointer.
    // Read DSDT
    AcpiStatus = AcpiGetTable(
        c"DSDT".as_ptr() as ACPI_STRING,
        0,
        &mut OutTable,
    );
    if ACPI_FAILURE(AcpiStatus) {
        Status = STATUS_UNSUCCESSFUL;
        ZwClose(KeyHandle);
        return Status;
    }
    Status = acpi_create_registry_table(KeyHandle, OutTable, ntkrnl_sys::w!("DSDT").as_ptr());
    if !NT_SUCCESS(Status) {
        ZwClose(KeyHandle);
        return Status;
    }

    // Read FACS
    AcpiStatus = AcpiGetTable(
        c"FACS".as_ptr() as ACPI_STRING,
        0,
        &mut OutTable,
    );
    if ACPI_FAILURE(AcpiStatus) {
        Status = STATUS_UNSUCCESSFUL;
        ZwClose(KeyHandle);
        return Status;
    }
    Status = acpi_create_registry_table(KeyHandle, OutTable, ntkrnl_sys::w!("FACS").as_ptr());
    if !NT_SUCCESS(Status) {
        ZwClose(KeyHandle);
        return Status;
    }

    // Read FADT  (ACPI_SIG_FADT == "FACP")
    AcpiStatus = AcpiGetTable(
        c"FACP".as_ptr() as ACPI_STRING,
        0,
        &mut OutTable,
    );
    if ACPI_FAILURE(AcpiStatus) {
        Status = STATUS_UNSUCCESSFUL;
        ZwClose(KeyHandle);
        return Status;
    }
    Status = acpi_create_registry_table(KeyHandle, OutTable, ntkrnl_sys::w!("FADT").as_ptr());
    if !NT_SUCCESS(Status) {
        ZwClose(KeyHandle);
        return Status;
    }

    // Rough copy from ACPICA reading of RSDT/XSDT.
    RsdpAddress = AcpiOsGetRootPointer();
    Rsdp = AcpiOsMapMemory(RsdpAddress, core::mem::size_of::<ACPI_TABLE_RSDP>() as _)
        as *mut ACPI_TABLE_RSDP;
    if Rsdp.is_null() {
        Status = STATUS_NO_MEMORY;
        ZwClose(KeyHandle);
        return Status;
    }

    // ACPI_TABLE_RSDP is #[repr(C, packed)] → read fields unaligned.
    let rsdp_revision = ptr::read_unaligned(ptr::addr_of!((*Rsdp).Revision));
    let rsdp_xsdt = ptr::read_unaligned(ptr::addr_of!((*Rsdp).XsdtPhysicalAddress));
    let rsdp_rsdt = ptr::read_unaligned(ptr::addr_of!((*Rsdp).RsdtPhysicalAddress));

    if rsdp_revision > 1 && rsdp_xsdt != 0 && AcpiGbl_DoNotUseXsdt == 0 {
        Address = rsdp_xsdt as ACPI_PHYSICAL_ADDRESS;
        TableEntrySize = ACPI_XSDT_ENTRY_SIZE;
    } else {
        Address = rsdp_rsdt as ACPI_PHYSICAL_ADDRESS;
        TableEntrySize = ACPI_RSDT_ENTRY_SIZE;
    }

    // Unmap the RSDP before mapping other tables.
    AcpiOsUnmapMemory(Rsdp as PVOID, core::mem::size_of::<ACPI_TABLE_RSDP>() as _);
    OutTable = AcpiOsMapMemory(Address, TableEntrySize as _) as *mut ACPI_TABLE_HEADER;
    if OutTable.is_null() {
        Status = STATUS_NO_MEMORY;
        ZwClose(KeyHandle);
        return Status;
    }

    // Dump RSDT
    Status = acpi_create_registry_table(KeyHandle, OutTable, ntkrnl_sys::w!("RSDT").as_ptr());
    AcpiOsUnmapMemory(OutTable as PVOID, TableEntrySize as _);
    // (no goto on failure here — falls through to the `done:` cleanup below.)

    // done:  (every C `goto done` is rendered as an early return that closes
    // KeyHandle first; this is the natural fall-through.)
    ZwClose(KeyHandle);
    Status
}

// InitializeObjectAttributes() macro → inline struct fill.
#[inline(always)]
unsafe fn initialize_object_attributes(
    p: *mut OBJECT_ATTRIBUTES,
    name: *mut UNICODE_STRING,
    attributes: ULONG,
    root_directory: HANDLE,
    security_descriptor: PVOID,
) {
    (*p).Length = core::mem::size_of::<OBJECT_ATTRIBUTES>() as ULONG;
    (*p).RootDirectory = root_directory;
    (*p).ObjectName = name;
    (*p).Attributes = attributes;
    (*p).SecurityDescriptor = security_descriptor;
    (*p).SecurityQualityOfService = ptr::null_mut();
}
