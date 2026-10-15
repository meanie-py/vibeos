//! Rust translation of `dll/ntdll/rtl/version.c`.
//!
//! OS version/product-type RTL helpers exported from ntdll.dll:
//! `RtlGetNtProductType`, `RtlGetNtVersionNumbers`, `RtlGetVersion`, plus the
//! file-static `SetRosSpecificInfo` ReactOS hack (CORE-6611 / CORE-4620).
//!
//! Translation notes (vs. the C):
//!   * `SharedUserData` is the C macro `(KUSER_SHARED_DATA*)USER_SHARED_DATA`
//!     (fixed user VA 0x7FFE0000) — NOT an exported symbol. We rebuild the
//!     pointer from `USER_SHARED_DATA` and read `NtProductType`/`SuiteMask` by
//!     field name (the C reads the same fields; named access avoids the
//!     arch-fragile raw offsets a naive transliteration emits).
//!   * `NtCurrentPeb()` is FORCEINLINE (no exported symbol) — use the
//!     hand-written `ntdll_sys::nt::NtCurrentPeb` inline.
//!   * `InitializeObjectAttributes` / `RTL_CONSTANT_STRING` are C macros — both
//!     inlined here (the registry key/value strings are emitted as UTF-16
//!     arrays with the exact `Length`/`MaximumLength` the macro computes).
//!   * `PEB::OSBuildNumber`/`OSCSDVersion` are USHORT — widened with explicit
//!     `as` casts where the C relied on integer promotion.
//!   * All writes through caller-provided output pointers use `write_volatile`
//!     to survive LLVM `captures(none)` (see tools/lint_kernel.py).

use core::cmp::min;
use core::mem::{size_of, size_of_val, zeroed};
use core::ptr::{addr_of_mut, copy_nonoverlapping, null_mut, write_bytes, write_volatile};
use ntdll_sys::nt::*;

/// `SharedUserData` — the C `#define SharedUserData ((KUSER_SHARED_DATA*)USER_SHARED_DATA)`.
#[inline(always)]
unsafe fn shared_user_data() -> *const _KUSER_SHARED_DATA {
    USER_SHARED_DATA as usize as *const _KUSER_SHARED_DATA
}

/// `static signed char g_ReportProductType` (cached product-type override:
/// 0 = uninitialized, -1 = no override, >0 = VER_NT_* to report).
static mut g_ReportProductType: i8 = 0;

/// HACK: ReactOS specific changes, see bug-reports CORE-6611 and CORE-4620
/// (aka. #5003). C: `static VOID NTAPI SetRosSpecificInfo(PRTL_OSVERSIONINFOEXW)`.
unsafe fn SetRosSpecificInfo(VersionInformation: PRTL_OSVERSIONINFOEXW) {
    // RTL_CONSTANT_STRING(L"\\Registry\\Machine\\...\\Version") — 75 WCHARs,
    // Length = 150, MaximumLength = 152 (sizeof incl. terminating NUL).
    static KEY_PATH: [u16; 76] = [
        92, 82, 101, 103, 105, 115, 116, 114, 121, 92, 77, 97, 99, 104, 105, 110, 101, 92, 83, 89,
        83, 84, 69, 77, 92, 67, 117, 114, 114, 101, 110, 116, 67, 111, 110, 116, 114, 111, 108, 83,
        101, 116, 92, 67, 111, 110, 116, 114, 111, 108, 92, 82, 101, 97, 99, 116, 79, 83, 92, 83,
        101, 116, 116, 105, 110, 103, 115, 92, 86, 101, 114, 115, 105, 111, 110, 0,
    ];
    // RTL_CONSTANT_STRING(L"ReportAsWorkstation") — 19 WCHARs, Length = 38.
    static VAL_NAME: [u16; 20] = [
        82, 101, 112, 111, 114, 116, 65, 115, 87, 111, 114, 107, 115, 116, 97, 116, 105, 111, 110, 0,
    ];

    let mut KeyName = UNICODE_STRING {
        Length: 150,
        MaximumLength: 152,
        Buffer: KEY_PATH.as_ptr() as *mut u16,
    };
    let mut ValName = UNICODE_STRING {
        Length: 38,
        MaximumLength: 40,
        Buffer: VAL_NAME.as_ptr() as *mut u16,
    };

    // CHAR Buffer[sizeof(KEY_VALUE_PARTIAL_INFORMATION) + sizeof(ULONG)];
    let mut buffer = [0u8; size_of::<_KEY_VALUE_PARTIAL_INFORMATION>() + size_of::<ULONG>()];
    let kvp_info = buffer.as_mut_ptr() as *mut _KEY_VALUE_PARTIAL_INFORMATION;

    // InitializeObjectAttributes(&ObjectAttributes, &KeyName,
    //                            OBJ_OPENIF | OBJ_CASE_INSENSITIVE, NULL, NULL)
    // — the macro, inlined.
    let mut object_attributes: _OBJECT_ATTRIBUTES = zeroed();
    object_attributes.Length = size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
    object_attributes.RootDirectory = null_mut();
    object_attributes.ObjectName = &mut KeyName;
    object_attributes.Attributes = OBJ_OPENIF | OBJ_CASE_INSENSITIVE;
    object_attributes.SecurityDescriptor = null_mut();
    object_attributes.SecurityQualityOfService = null_mut();

    let mut h_key: HANDLE = null_mut();

    // Don't change anything if the key doesn't exist.
    let status = if g_ReportProductType == 0 {
        NtOpenKey(&mut h_key, KEY_READ, &mut object_attributes)
    } else {
        STATUS_CANCELLED
    };

    if status >= 0 {
        let mut length: ULONG = 0;
        // Get the value from the registry and make sure it's a 32-bit value.
        let status = NtQueryValueKey(
            h_key,
            &mut ValName,
            KeyValuePartialInformation,
            kvp_info as PVOID,
            size_of_val(&buffer) as ULONG,
            &mut length,
        );
        if status >= 0
            && (*kvp_info).Type == REG_DWORD
            && (*kvp_info).DataLength == size_of::<ULONG>() as ULONG
        {
            let sud = shared_user_data();
            let is_workstation = (*sud).NtProductType == NtProductWinNt;
            // *(PULONG)kvpInfo->Data — Data is a flexible array; read 4 bytes
            // from its base (Buffer is sized to hold one trailing ULONG).
            let report_as_workstation = *((*kvp_info).Data.as_ptr() as *const ULONG) != 0;
            if is_workstation != report_as_workstation {
                g_ReportProductType = if report_as_workstation {
                    VER_NT_WORKSTATION as i8
                } else {
                    VER_NT_SERVER as i8
                };
            }
        }

        NtClose(h_key);
    }

    if g_ReportProductType > 0 {
        write_volatile(
            addr_of_mut!((*VersionInformation).wProductType),
            g_ReportProductType as u8,
        );
    } else {
        // No override, caller gets the real value.
        g_ReportProductType = -1;
    }
}

/// C: `BOOLEAN NTAPI RtlGetNtProductType(PNT_PRODUCT_TYPE ProductType)`.
/// Retrieves the OS product type; always returns TRUE. @implemented.
#[no_mangle]
pub unsafe extern "system" fn RtlGetNtProductType(ProductType: PNT_PRODUCT_TYPE) -> BOOLEAN {
    let product_type = (*shared_user_data()).NtProductType;
    write_volatile(ProductType, product_type);

    if g_ReportProductType == 0 {
        // Initialize cached value.
        let mut ovi: RTL_OSVERSIONINFOEXW = zeroed();
        ovi.dwOSVersionInfoSize = size_of::<RTL_OSVERSIONINFOEXW>() as DWORD;
        ovi.wProductType = product_type as u8;
        SetRosSpecificInfo(&mut ovi);
    }

    if g_ReportProductType > 0 {
        write_volatile(ProductType, g_ReportProductType as _NT_PRODUCT_TYPE);
    }
    1 // TRUE
}

/// C: `VOID NTAPI RtlGetNtVersionNumbers(PULONG pMajorVersion,
/// PULONG pMinorVersion, PULONG pBuildNumber)`. Reports at least 5.1 (XP) —
/// pre-XP this call didn't exist. @implemented.
#[no_mangle]
pub unsafe extern "system" fn RtlGetNtVersionNumbers(
    pMajorVersion: *mut ULONG,
    pMinorVersion: *mut ULONG,
    pBuildNumber: *mut ULONG,
) {
    let peb = NtCurrentPeb();
    let major = (*peb).OSMajorVersion;
    let minor = (*peb).OSMinorVersion;

    if !pMajorVersion.is_null() {
        write_volatile(pMajorVersion, if major < 5 { 5 } else { major });
    }

    if !pMinorVersion.is_null() {
        let value = if (major < 5) || (major == 5 && minor < 1) {
            1
        } else {
            minor
        };
        write_volatile(pMinorVersion, value);
    }

    if !pBuildNumber.is_null() {
        // Windows really does this! (OSBuildNumber is USHORT.)
        write_volatile(pBuildNumber, 0xF000_0000u32 | (*peb).OSBuildNumber as ULONG);
    }
}

/// C: `NTSTATUS NTAPI RtlGetVersion(PRTL_OSVERSIONINFOW lpVersionInformation)`.
/// User-mode counterpart of ntoskrnl's RtlGetVersion. @implemented.
#[no_mangle]
pub unsafe extern "system" fn RtlGetVersion(lpVersionInformation: PRTL_OSVERSIONINFOW) -> NTSTATUS {
    let peb = NtCurrentPeb();

    let info_size = (*lpVersionInformation).dwOSVersionInfoSize;
    if info_size != size_of::<RTL_OSVERSIONINFOW>() as DWORD
        && info_size != size_of::<RTL_OSVERSIONINFOEXW>() as DWORD
    {
        return STATUS_INVALID_PARAMETER;
    }

    write_volatile(
        addr_of_mut!((*lpVersionInformation).dwMajorVersion),
        (*peb).OSMajorVersion,
    );
    write_volatile(
        addr_of_mut!((*lpVersionInformation).dwMinorVersion),
        (*peb).OSMinorVersion,
    );
    write_volatile(
        addr_of_mut!((*lpVersionInformation).dwBuildNumber),
        (*peb).OSBuildNumber as DWORD,
    );
    write_volatile(
        addr_of_mut!((*lpVersionInformation).dwPlatformId),
        (*peb).OSPlatformId,
    );

    // RtlZeroMemory(szCSDVersion).
    let csd_dst = addr_of_mut!((*lpVersionInformation).szCSDVersion) as *mut u16;
    write_bytes(
        csd_dst as *mut u8,
        0,
        size_of_val(&(*lpVersionInformation).szCSDVersion),
    );

    // If we have a CSD version string, initialized by Application Compatibility...
    let csd = (*peb).CSDVersion;
    let length: usize = if csd.Length != 0 && !csd.Buffer.is_null() && *csd.Buffer != 0 {
        // ... copy it (min(wcslen, ARRAYSIZE - 1))...
        let mut src_len = 0usize;
        while *csd.Buffer.add(src_len) != 0 {
            src_len += 1;
        }
        let n = min(src_len, 128 - 1);
        copy_nonoverlapping(csd.Buffer, csd_dst, n);
        n
    } else {
        // ... otherwise we just null-terminate it.
        0
    };

    // Always null-terminate the user CSD version string.
    write_volatile(csd_dst.add(length), 0u16);

    if info_size == size_of::<RTL_OSVERSIONINFOEXW>() as DWORD {
        let info_ex = lpVersionInformation as PRTL_OSVERSIONINFOEXW;
        let sud = shared_user_data();
        write_volatile(
            addr_of_mut!((*info_ex).wServicePackMajor),
            (((*peb).OSCSDVersion >> 8) & 0xFF) as WORD,
        );
        write_volatile(
            addr_of_mut!((*info_ex).wServicePackMinor),
            ((*peb).OSCSDVersion & 0xFF) as WORD,
        );
        write_volatile(
            addr_of_mut!((*info_ex).wSuiteMask),
            ((*sud).SuiteMask & 0xFFFF) as WORD,
        );
        write_volatile(
            addr_of_mut!((*info_ex).wProductType),
            (*sud).NtProductType as u8,
        );
        write_volatile(addr_of_mut!((*info_ex).wReserved), 0u8);

        // HACK: ReactOS specific changes, see CORE-6611 and CORE-4620 (#5003).
        SetRosSpecificInfo(info_ex);
    }

    STATUS_SUCCESS
}

// winnt.h PRODUCT_* values RtlGetProductInfo reports.
const PRODUCT_UNDEFINED: ULONG = 0x0000_0000;
const PRODUCT_BUSINESS: ULONG = 0x0000_0006;
const PRODUCT_STANDARD_SERVER: ULONG = 0x0000_0007;
const PRODUCT_ENTERPRISE_SERVER: ULONG = 0x0000_000A;
const PRODUCT_PROFESSIONAL: ULONG = 0x0000_0030;

/// C: `BOOLEAN WINAPI RtlGetProductInfo(ULONG OSMajorVersion, ULONG OSMinorVersion,
/// ULONG SpMajorVersion, ULONG SpMinorVersion, PULONG ReturnedProductType)`
/// (sdk/lib/rtl/product.c, which no target compiles: kernel32's GetProductInfo
/// forwards here and was a dead forward). Pre-Vista versions have no product SKU and
/// report PRODUCT_UNDEFINED with FALSE, as Windows does.
///
/// Divergence from product.c: the type comes from RtlGetNtProductType rather than a
/// raw SharedUserData read, so the ReactOS product-type override that GetVersionEx
/// reports (SetRosSpecificInfo) is reported here too and the two never disagree.
#[no_mangle]
pub unsafe extern "system" fn RtlGetProductInfo(
    OSMajorVersion: ULONG,
    OSMinorVersion: ULONG,
    _SpMajorVersion: ULONG,
    _SpMinorVersion: ULONG,
    ReturnedProductType: PULONG,
) -> BOOLEAN {
    if ReturnedProductType.is_null() {
        return 0;
    }
    if OSMajorVersion < 6 {
        write_volatile(ReturnedProductType, PRODUCT_UNDEFINED);
        return 0;
    }
    let mut product_type: _NT_PRODUCT_TYPE = zeroed();
    RtlGetNtProductType(&mut product_type);
    let sku = if product_type == NtProductWinNt {
        if OSMajorVersion == 6 && OSMinorVersion == 0 {
            PRODUCT_BUSINESS
        } else {
            PRODUCT_PROFESSIONAL
        }
    } else if product_type == NtProductLanManNt {
        PRODUCT_ENTERPRISE_SERVER
    } else if product_type == NtProductServer {
        PRODUCT_STANDARD_SERVER
    } else {
        write_volatile(ReturnedProductType, PRODUCT_UNDEFINED);
        return 0;
    };
    write_volatile(ReturnedProductType, sku);
    1
}
