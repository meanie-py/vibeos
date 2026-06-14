// Translated from win32ss/user/rtl/image.c — RtlGetExpWinVer.
// Returns the expected OS version (MAKEWORD(minor, major)) from an app module's
// PE OptionalHeader subsystem version. Defaults to Windows 3.10 (WINVER_WIN31).
use win32k_sys::win32k::*;

extern "system" {
    fn RtlImageNtHeader(BaseAddress: PVOID) -> PIMAGE_NT_HEADERS;
}

#[no_mangle]
pub unsafe extern "C" fn RtlGetExpWinVer(BaseAddress: PVOID) -> ULONG {
    // Default to Windows 3.10 (WINVER_WIN31).
    let mut dw_major_version: ULONG = 3;
    let mut dw_minor_version: ULONG = 10;
    let mut aligned_address: ULONG_PTR = BaseAddress as ULONG_PTR;

    /* Remove the magic flag for non-mapped images */
    if aligned_address & 1 != 0 {
        aligned_address &= !1;
    }

    /* Non-zero and 64K-aligned (LOWORD == 0) */
    if aligned_address != 0 && (aligned_address & 0xFFFF) == 0 {
        let p_nt_header = RtlImageNtHeader(aligned_address as PVOID);
        if !p_nt_header.is_null() {
            dw_major_version = (*p_nt_header).OptionalHeader.MajorSubsystemVersion as ULONG;
            if dw_major_version == 1 {
                dw_major_version = 3;
            } else {
                dw_minor_version = (*p_nt_header).OptionalHeader.MinorSubsystemVersion as ULONG;
            }
        }
    }

    /* MAKEWORD(low = minor, high = major) */
    ((dw_minor_version & 0xff) | ((dw_major_version & 0xff) << 8)) as ULONG
}
