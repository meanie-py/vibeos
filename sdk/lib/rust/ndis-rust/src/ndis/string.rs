use crate::prelude::*;
use ntkrnl_sys::ddk::{ANSI_STRING, PANSI_STRING, PCSTR, PCSZ, PUNICODE_STRING, RtlAnsiStringToUnicodeString, RtlEqualUnicodeString, RtlInitAnsiString, RtlInitString, RtlInitUnicodeString, RtlUnicodeStringToAnsiString, RtlUpcaseUnicodeString};
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// The preamble contains only `#undef` directives, which have no Rust equivalent and define no declarations, so there is nothing to emit.

#[no_mangle]
pub unsafe extern "system" fn NdisAnsiStringToUnicodeString(
    DestinationString: PNDIS_STRING,
    SourceString: PANSI_STRING,
) -> NDIS_STATUS {
    PAGED_CODE!();
    return RtlAnsiStringToUnicodeString(
        DestinationString as PUNICODE_STRING,
        SourceString as PANSI_STRING,
        FALSE,
    ) as NDIS_STATUS;
}
#[no_mangle]
pub unsafe extern "system" fn NdisEqualString(
    String1: PNDIS_STRING,
    String2: PNDIS_STRING,
    CaseInsensitive: BOOLEAN,
) -> BOOLEAN {
    if KeGetCurrentIrql() as i32 > 1 {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0".as_ptr() as PCSTR,
            KeGetCurrentIrql() as i32,
        );
        if 0 == 0 {
            RtlAssert(
                b"0\0".as_ptr() as *const i8 as PVOID,
                b"drivers/network/ndis/ndis/string.c\0"
                    .as_ptr() as *const i8 as PVOID,
                63 as ULONG,
                core::ptr::null_mut::<CHAR>(),
            );
        } else {
        }
    }
    if String1.is_null() {
        RtlAssert(
            b"String1\0".as_ptr() as *const i8 as PVOID,
            b"drivers/network/ndis/ndis/string.c\0"
                .as_ptr() as *const i8 as PVOID,
            64 as ULONG,
            core::ptr::null_mut::<CHAR>(),
        );
    } else {
    }
    if String2.is_null() {
        RtlAssert(
            b"String2\0".as_ptr() as *const i8 as PVOID,
            b"drivers/network/ndis/ndis/string.c\0"
                .as_ptr() as *const i8 as PVOID,
            65 as ULONG,
            core::ptr::null_mut::<CHAR>(),
        );
    } else {
    }
    return RtlEqualUnicodeString(
        String1 as PUNICODE_STRING as *const _UNICODE_STRING,
        String2 as PUNICODE_STRING as *const _UNICODE_STRING,
        CaseInsensitive,
    );
}
#[no_mangle]
pub unsafe extern "system" fn NdisInitAnsiString(
    mut DestinationString: PANSI_STRING,
    mut SourceString: PCSTR,
) {
    if DestinationString.is_null() {
        RtlAssert(
            b"DestinationString\0".as_ptr() as *const core::ffi::c_char as PVOID,
            b"drivers/network/ndis/ndis/string.c\0"
                .as_ptr() as *const core::ffi::c_char as PVOID,
            90 as ULONG,
            core::ptr::null_mut::<CHAR>(),
        );
    } else {
    };
    if SourceString.is_null() {
        RtlAssert(
            b"SourceString\0".as_ptr() as *const core::ffi::c_char as PVOID,
            b"drivers/network/ndis/ndis/string.c\0"
                .as_ptr() as *const core::ffi::c_char as PVOID,
            91 as ULONG,
            core::ptr::null_mut::<CHAR>(),
        );
    } else {
    };
    RtlInitString(DestinationString as PANSI_STRING, SourceString as PCSZ);
}
#[no_mangle]
pub unsafe extern "system" fn NdisInitializeString(
    DestinationString: PNDIS_STRING,
    SourceString: PUCHAR,
) {
    let mut AnsiString: ANSI_STRING = core::mem::zeroed();

    PAGED_CODE!();

    RtlInitAnsiString(core::ptr::addr_of_mut!(AnsiString), SourceString as PCSZ);

    RtlAnsiStringToUnicodeString(
        DestinationString as PUNICODE_STRING,
        core::ptr::addr_of_mut!(AnsiString),
        TRUE,
    );
}
#[no_mangle]
pub unsafe extern "system" fn NdisInitUnicodeString(
    mut DestinationString: PNDIS_STRING,
    mut SourceString: PCWSTR,
) {
    if DestinationString.is_null() {
        RtlAssert(
            b"DestinationString\0".as_ptr() as *const i8 as PVOID,
            b"drivers/network/ndis/ndis/string.c\0"
                .as_ptr() as *const i8 as PVOID,
            142 as ULONG,
            core::ptr::null_mut::<CHAR>(),
        );
    }
    if SourceString.is_null() {
        RtlAssert(
            b"SourceString\0".as_ptr() as *const i8 as PVOID,
            b"drivers/network/ndis/ndis/string.c\0"
                .as_ptr() as *const i8 as PVOID,
            143 as ULONG,
            core::ptr::null_mut::<CHAR>(),
        );
    }
    RtlInitUnicodeString(DestinationString as PUNICODE_STRING, SourceString);
}
#[no_mangle]
pub unsafe extern "system" fn NdisUnicodeStringToAnsiString(
    mut DestinationString: PANSI_STRING,
    mut SourceString: PNDIS_STRING,
) -> NDIS_STATUS {
    if KeGetCurrentIrql() as i32 > (APC_LEVEL as i32) {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0" as *const u8 as PCSTR,
            KeGetCurrentIrql() as i32,
        );
        if 0 == 0 {
            RtlAssert(
                b"0\0" as *const u8 as *const ::core::ffi::c_char as PVOID,
                b"drivers/network/ndis/ndis/string.c\0"
                    as *const u8 as *const ::core::ffi::c_char as PVOID,
                166 as ULONG,
                ::core::ptr::null_mut::<CHAR>(),
            );
        } else {
        };
    }
    if DestinationString.is_null() {
        RtlAssert(
            b"DestinationString\0" as *const u8 as *const ::core::ffi::c_char as PVOID,
            b"drivers/network/ndis/ndis/string.c\0"
                as *const u8 as *const ::core::ffi::c_char as PVOID,
            167 as ULONG,
            ::core::ptr::null_mut::<CHAR>(),
        );
    } else {
    };
    if SourceString.is_null() {
        RtlAssert(
            b"SourceString\0" as *const u8 as *const ::core::ffi::c_char as PVOID,
            b"drivers/network/ndis/ndis/string.c\0"
                as *const u8 as *const ::core::ffi::c_char as PVOID,
            168 as ULONG,
            ::core::ptr::null_mut::<CHAR>(),
        );
    } else {
    };
    return RtlUnicodeStringToAnsiString(
        DestinationString,
        SourceString as PUNICODE_STRING,
        FALSE as BOOLEAN,
    ) as NDIS_STATUS;
}
#[no_mangle]
pub unsafe extern "system" fn NdisUpcaseUnicodeString(
    DestinationString: PUNICODE_STRING,
    SourceString: PUNICODE_STRING,
) -> NTSTATUS {
    // PAGED_CODE() expansion
    if KeGetCurrentIrql() as i32 > (APC_LEVEL as i32) {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0".as_ptr() as PCSTR,
            KeGetCurrentIrql() as i32,
        );
        if 0 == 0 {
            RtlAssert(
                b"0\0".as_ptr() as *const i8 as PVOID,
                b"drivers/network/ndis/ndis/string.c\0"
                    .as_ptr() as *const i8 as PVOID,
                195 as ULONG,
                core::ptr::null_mut::<CHAR>(),
            );
        } else {
        }
    }
    if SourceString.is_null() {
        RtlAssert(
            b"SourceString\0".as_ptr() as *const i8 as PVOID,
            b"drivers/network/ndis/ndis/string.c\0"
                .as_ptr() as *const i8 as PVOID,
            196 as ULONG,
            core::ptr::null_mut::<CHAR>(),
        );
    } else {
    }
    if DestinationString.is_null() {
        RtlAssert(
            b"DestinationString\0".as_ptr() as *const i8 as PVOID,
            b"drivers/network/ndis/ndis/string.c\0"
                .as_ptr() as *const i8 as PVOID,
            197 as ULONG,
            core::ptr::null_mut::<CHAR>(),
        );
    } else {
    }

    return RtlUpcaseUnicodeString(
        DestinationString,
        SourceString as PCUNICODE_STRING,
        FALSE as BOOLEAN,
    );
}
