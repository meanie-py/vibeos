use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
pub static RUSTCONV_C2R_WSTR_0DFCEF8A9C67: [u16; 23] = [
    0x0043u16, 0x0075, 0x0072, 0x0072, 0x0065, 0x006E, 0x0074, 0x0055, 0x0073, 0x0065,
    0x0072, 0x0047, 0x0072, 0x006F, 0x0075, 0x0070, 0x0050, 0x006F, 0x006C, 0x0069,
    0x0063, 0x0079, 0x0000,
];
pub static RUSTCONV_C2R_WSTR_3744D4E2CD83: [u16; 23] = [
    0x004Cu16, 0x006F, 0x0063, 0x0061, 0x006C, 0x004D, 0x0061, 0x0063, 0x0068, 0x0069,
    0x006E, 0x0065, 0x0045, 0x006E, 0x0074, 0x0065, 0x0072, 0x0070, 0x0072, 0x0069,
    0x0073, 0x0065, 0x0000,
];
pub static RUSTCONV_C2R_WSTR_51AE552436FD: [u16; 9] = [0x0053u16, 0x0065, 0x0072, 0x0076, 0x0069, 0x0063, 0x0065, 0x0073, 0x0000];
pub static RUSTCONV_C2R_WSTR_7A8E4722B332: [u16; 12] = [
    0x0043u16, 0x0075, 0x0072, 0x0072, 0x0065, 0x006E, 0x0074, 0x0055, 0x0073, 0x0065,
    0x0072, 0x0000,
];
pub static RUSTCONV_C2R_WSTR_8274397E0662: [u16; 6] = [0x0055u16, 0x0073, 0x0065, 0x0072, 0x0073, 0x0000];
pub static RUSTCONV_C2R_WSTR_877512FD7C20: [u16; 24] = [
    0x004Cu16, 0x006F, 0x0063, 0x0061, 0x006C, 0x004D, 0x0061, 0x0063, 0x0068, 0x0069,
    0x006E, 0x0065, 0x0047, 0x0072, 0x006F, 0x0075, 0x0070, 0x0050, 0x006F, 0x006C,
    0x0069, 0x0063, 0x0079, 0x0000,
];
pub static RUSTCONV_C2R_WSTR_D3391816B6AF: [u16; 15] = [
    0x0043u16, 0x0075, 0x0072, 0x0072, 0x0065, 0x006E, 0x0074, 0x0053, 0x0065, 0x0072,
    0x0076, 0x0069, 0x0063, 0x0065, 0x0000,
];
pub static RUSTCONV_C2R_WSTR_DCEA9A5751F0: [u16; 13] = [
    0x004Cu16, 0x006F, 0x0063, 0x0061, 0x006C, 0x004D, 0x0061, 0x0063, 0x0068, 0x0069,
    0x006E, 0x0065, 0x0000,
];

static mut gSystemStoreLocations: [CERT_SYSTEM_STORE_LOCATION; 8] = unsafe {
    [
        _CERT_SYSTEM_STORE_LOCATION {
            dwFlags: CERT_SYSTEM_STORE_CURRENT_USER as DWORD,
            pwszStoreLocation: RUSTCONV_C2R_WSTR_7A8E4722B332.as_ptr() as PCWSTR,
        },
        _CERT_SYSTEM_STORE_LOCATION {
            dwFlags: CERT_SYSTEM_STORE_LOCAL_MACHINE as DWORD,
            pwszStoreLocation: RUSTCONV_C2R_WSTR_DCEA9A5751F0.as_ptr() as PCWSTR,
        },
        _CERT_SYSTEM_STORE_LOCATION {
            dwFlags: CERT_SYSTEM_STORE_CURRENT_SERVICE as DWORD,
            pwszStoreLocation: RUSTCONV_C2R_WSTR_D3391816B6AF.as_ptr() as PCWSTR,
        },
        _CERT_SYSTEM_STORE_LOCATION {
            dwFlags: CERT_SYSTEM_STORE_SERVICES as DWORD,
            pwszStoreLocation: RUSTCONV_C2R_WSTR_51AE552436FD.as_ptr() as PCWSTR,
        },
        _CERT_SYSTEM_STORE_LOCATION {
            dwFlags: CERT_SYSTEM_STORE_USERS as DWORD,
            pwszStoreLocation: RUSTCONV_C2R_WSTR_8274397E0662.as_ptr() as PCWSTR,
        },
        _CERT_SYSTEM_STORE_LOCATION {
            dwFlags: CERT_SYSTEM_STORE_CURRENT_USER_GROUP_POLICY as DWORD,
            pwszStoreLocation: RUSTCONV_C2R_WSTR_0DFCEF8A9C67.as_ptr() as PCWSTR,
        },
        _CERT_SYSTEM_STORE_LOCATION {
            dwFlags: CERT_SYSTEM_STORE_LOCAL_MACHINE_GROUP_POLICY as DWORD,
            pwszStoreLocation: RUSTCONV_C2R_WSTR_877512FD7C20.as_ptr() as PCWSTR,
        },
        _CERT_SYSTEM_STORE_LOCATION {
            dwFlags: CERT_SYSTEM_STORE_LOCAL_MACHINE_ENTERPRISE as DWORD,
            pwszStoreLocation: RUSTCONV_C2R_WSTR_3744D4E2CD83.as_ptr() as PCWSTR,
        },
    ]
};


// WINE_DEFAULT_DEBUG_CHANNEL(crypt): declares the "crypt" debug channel.
// Handled by the DPRINT*/TRACE/WARN/ERR macros in the crate; no Rust item emitted.

pub type PCERT_SYSTEM_STORE_LOCATION = *mut CERT_SYSTEM_STORE_LOCATION;

// Raw pointers are not Sync; the table is read-only and holds only static UTF-16 literals.
unsafe impl Sync for CERT_SYSTEM_STORE_LOCATION {}

#[no_mangle]
pub unsafe extern "system" fn CertEnumSystemStoreLocation(
    dwFlags: DWORD,
    pvArg: *mut core::ffi::c_void,
    pfnEnum: PFN_CERT_ENUM_SYSTEM_STORE_LOCATION,
) -> BOOL {
    let mut i: DWORD;

    /* Check input flags */
    if dwFlags != 0 {
        SetLastError((E_INVALIDARG) as DWORD);
        return (FALSE) as BOOL;
    }

    /* Return fixed system stores */
    i = 0;
    while i < ARRAYSIZE!(gSystemStoreLocations) {
        if (pfnEnum.unwrap_unchecked())(
            gSystemStoreLocations[i as usize].pwszStoreLocation,
            gSystemStoreLocations[i as usize].dwFlags,
            core::ptr::null_mut(),
            pvArg,
        ) == 0
        {
            return (FALSE) as BOOL;
        }
        i = i.wrapping_add(1);
    }

    /* FIXME: Return registered OID system stores by calling CryptEnumOIDFunction */

    return (TRUE) as BOOL;
}
