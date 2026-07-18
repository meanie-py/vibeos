use win32k_sys::win32k::*;
use ntkrnl_sys::ddk::{RtlAnsiStringToUnicodeString, RtlCreateUnicodeString, RtlInitAnsiString, RtlInitUnicodeString};
use ntkrnl_sys::base::NT_SUCCESS;

extern "C" {
    fn wcslen(s: PCWSTR) -> usize;
    fn strlen(s: *const i8) -> usize;
}

#[no_mangle]
pub unsafe extern "system" fn RtlInitLargeAnsiString(
    DestinationString: PLARGE_ANSI_STRING,
    SourceString: PCSZ,
    Unknown: INT,
) {
    let DestSize: USHORT;

    if !SourceString.is_null() {
        DestSize = strlen(SourceString as *const _) as USHORT;
        (*DestinationString).Length = DestSize as u32;
        // MaximumLength is a :31 bitfield → bindgen set_ accessor.
        (*DestinationString).set_MaximumLength(DestSize as u32 + core::mem::size_of::<CHAR>() as u32);
    } else {
        (*DestinationString).Length = 0;
        (*DestinationString).set_MaximumLength(0);
    }

    (*DestinationString).Buffer = SourceString as PCHAR;
    (*DestinationString).set_bAnsi(1);
}

#[no_mangle]
pub unsafe extern "system" fn RtlInitLargeUnicodeString(
    DestinationString: PLARGE_UNICODE_STRING,
    SourceString: PCWSTR,
    Unknown: INT,
) {

    let DestSize: USHORT;

    if !SourceString.is_null() {
        DestSize = (wcslen(SourceString) as USHORT) * core::mem::size_of::<u16>() as USHORT;
        (*DestinationString).Length = DestSize as u32;
        (*DestinationString).set_MaximumLength(DestSize as u32 + core::mem::size_of::<u16>() as u32);
    } else {
        (*DestinationString).Length = 0;
        (*DestinationString).set_MaximumLength(0);
    }

    (*DestinationString).Buffer = SourceString as PWSTR;
    (*DestinationString).set_bAnsi(0);
}

#[no_mangle]
pub unsafe extern "system" fn RtlLargeStringToUnicodeString(
    DestinationString: PUNICODE_STRING,
    SourceString: PLARGE_STRING,
) -> BOOL {
    let mut AnsiString: ANSI_STRING = core::mem::zeroed();

    if DestinationString.is_null() || SourceString.is_null() {
        return 0;
    }

    // Cross-crate bridges: the Rtl* fns are ntkrnl_sys and take ntkrnl_sys
    // _UNICODE_STRING/_ANSI_STRING; our locals are the win32k_sys twins (`as _`).
    RtlInitUnicodeString(DestinationString as _, core::ptr::null_mut());

    if (*SourceString).bAnsi() != 0 {
        RtlInitAnsiString(
            core::ptr::addr_of_mut!(AnsiString) as _,
            (*SourceString).Buffer as LPSTR,
        );
        NT_SUCCESS(RtlAnsiStringToUnicodeString(
            DestinationString as _,
            core::ptr::addr_of_mut!(AnsiString) as _,
            1u8,
        )) as BOOL
    } else {
        RtlCreateUnicodeString(DestinationString as _, (*SourceString).Buffer as _) as BOOL
    }
}
