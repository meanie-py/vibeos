use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::ddk::{
    _OBJECT_ATTRIBUTES, _UNICODE_STRING, HANDLE,
    PKEY_VALUE_PARTIAL_INFORMATION, PNT_PRODUCT_TYPE,
    PRTL_OSVERSIONINFOEXW, PRTL_OSVERSIONINFOW, PUNICODE_STRING,
    RTL_OSVERSIONINFOEXW,
    RtlCompareUnicodeString, ZwClose, ZwOpenKey, ZwQueryValueKey,
};
use ntkrnl_sys::ex::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::{opaque_ptr, opaque_ptr_const};

// w! — NUL-terminated UTF-16 array from an ASCII literal (2-aligned 'static;
// a b"...\0" byte string cast to *const u16 is both ANSI-not-wide and
// potentially odd-aligned — see field manual / basesrv init.rs CSDVersion).
macro_rules! w {
    ($s:literal) => {{
        const __W: &[u16] = &{
            const BYTES: &[u8] = $s.as_bytes();
            let mut out = [0u16; { BYTES.len() + 1 }];
            let mut i = 0;
            while i < BYTES.len() {
                out[i] = BYTES[i] as u16;
                i += 1;
            }
            out
        };
        __W
    }};
}


const PRODUCT_TAG: ULONG = u32::from_be_bytes(*b"RtPi");

extern "system" {
    static mut NtGlobalFlag: ULONG;
    static mut NtMajorVersion: ULONG;
    static mut NtMinorVersion: ULONG;
    static mut NtOSCSDVersion: ULONG;
    static mut NtBuildNumber: ULONG;
    static mut CmNtCSDVersion: ULONG;
    static mut SharedUserData: *mut ntkrnl_sys::ddk::_KUSER_SHARED_DATA;
}

extern "C" {
    fn InitializeObjectAttributes(
        p: *mut _OBJECT_ATTRIBUTES,
        n: PUNICODE_STRING,
        a: ULONG,
        r: HANDLE,
        s: PVOID,
    );
}

// KUSER_SHARED_DATA: bindgen layout. The shadow struct here read SuiteMask at
// +0x2E0 as USHORT and NtProductType at +0x2E4 as UCHAR (real: +0x2D0 ULONG /
// +0x264 enum) — RtlGetVersion reported garbage suite/product values. Same
// landmine class as ke/amd64/kiinit.rs (2026-07-11).

#[no_mangle]
pub unsafe extern "system" fn RtlGetNtGlobalFlags() -> ULONG {
    NtGlobalFlag
}

#[no_mangle]
pub unsafe extern "system" fn RtlGetVersion(
    lpVersionInformation: PRTL_OSVERSIONINFOW,
) -> NTSTATUS {
    (*lpVersionInformation).dwMajorVersion = NtMajorVersion;
    (*lpVersionInformation).dwMinorVersion = NtMinorVersion;
    (*lpVersionInformation).dwBuildNumber = NtBuildNumber & 0x3FFF;
    (*lpVersionInformation).dwPlatformId = 2;

    if (*lpVersionInformation).dwOSVersionInfoSize
        == core::mem::size_of::<RTL_OSVERSIONINFOEXW>() as ULONG
    {
        let info_ex = lpVersionInformation as PRTL_OSVERSIONINFOEXW;
        (*info_ex).wServicePackMajor = ((CmNtCSDVersion >> 8) & 0xFF) as USHORT;
        (*info_ex).wServicePackMinor = (CmNtCSDVersion & 0xFF) as USHORT;
        (*info_ex).wSuiteMask = ((*SharedUserData).SuiteMask & 0xFFFF) as USHORT;
        (*info_ex).wProductType = (*SharedUserData).NtProductType as UCHAR;
        (*info_ex).wReserved = 0;
    }

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn RtlGetNtProductType(ProductType: PNT_PRODUCT_TYPE) -> BOOLEAN {
    let mut key: HANDLE = core::ptr::null_mut();
    let mut success: BOOLEAN;
    let mut returned_length: ULONG = 0;
    let mut object_attributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let buffer_key_length: ULONG =
        core::mem::size_of::<KEY_VALUE_PARTIAL_INFORMATION>() as ULONG + (256 * 2);
    #[no_mangle]
    pub static mut WORKSTATION_PRODUCT: _UNICODE_STRING = _UNICODE_STRING {
        Length: 10,
        MaximumLength: 12,
        Buffer: w!("WinNT").as_ptr() as *mut u16,
    };
    #[no_mangle]
    pub static mut LANMAN_PRODUCT: _UNICODE_STRING = _UNICODE_STRING {
        Length: 16,
        MaximumLength: 18,
        Buffer: w!("LanmanNT").as_ptr() as *mut u16,
    };
    #[no_mangle]
    pub static mut SERVER_PRODUCT: _UNICODE_STRING = _UNICODE_STRING {
        Length: 16,
        MaximumLength: 18,
        Buffer: w!("ServerNT").as_ptr() as *mut u16,
    };
    #[no_mangle]
    pub static mut KEY_PRODUCT: _UNICODE_STRING = _UNICODE_STRING {
        Length: 160,
        MaximumLength: 162,
        Buffer: w!("\\Registry\\Machine\\SYSTEM\\CurrentControlSet\\Control\\ProductOptions").as_ptr() as *mut u16,
    };
    #[no_mangle]
    pub static mut VALUE_NT_PRODUCT: _UNICODE_STRING = _UNICODE_STRING {
        Length: 22,
        MaximumLength: 24,
        Buffer: w!("ProductType").as_ptr() as *mut u16,
    };

    let buffer_key = ExAllocatePoolWithTag(PagedPool, buffer_key_length as SIZE_T, PRODUCT_TAG);
    if buffer_key.is_null() {
        success = 0u8;
        *ProductType = 1;
        return success;
    }

    InitializeObjectAttributes(
        &mut object_attributes,
        &raw mut KEY_PRODUCT,
        OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );

    let status = ZwOpenKey(&mut key, KEY_QUERY_VALUE, &mut object_attributes);
    if status < 0 {
        success = 0u8;
        ExFreePoolWithTag(buffer_key, PRODUCT_TAG);
        *ProductType = 1;
        return success;
    }

    let status = ZwQueryValueKey(
        key,
        &raw mut VALUE_NT_PRODUCT,
        2,
        buffer_key,
        buffer_key_length,
        &mut returned_length,
    );

    ZwClose(key);

    if status < 0 {
        success = 0u8;
        ExFreePoolWithTag(buffer_key, PRODUCT_TAG);
        *ProductType = 1;
        return success;
    }

    let buffer_key_info = buffer_key as PKEY_VALUE_PARTIAL_INFORMATION;
    if (*buffer_key_info).Type != 1 {
        success = 0u8;
        ExFreePoolWithTag(buffer_key, PRODUCT_TAG);
        *ProductType = 1;
        return success;
    }

    let mut nt_product_type = _UNICODE_STRING {
        Length: (*buffer_key_info).DataLength as USHORT,
        MaximumLength: (*buffer_key_info).DataLength as USHORT,
        Buffer: (*buffer_key_info).Data.as_ptr() as *mut u16,
    };

    if nt_product_type.Length > 2
        && *nt_product_type
            .Buffer
            .offset((nt_product_type.Length / 2 - 1) as isize)
            == 0
    {
        nt_product_type.Length -= 2;
    }

    if RtlCompareUnicodeString(&mut nt_product_type, &raw mut WORKSTATION_PRODUCT, 1) == 0 {
        *ProductType = 1;
        success = 1u8;
    } else if RtlCompareUnicodeString(&mut nt_product_type, &raw mut LANMAN_PRODUCT, 1) == 0 {
        *ProductType = 2;
        success = 1u8;
    } else if RtlCompareUnicodeString(&mut nt_product_type, &raw mut SERVER_PRODUCT, 1) == 0 {
        *ProductType = 3;
        success = 1u8;
    } else {
        success = 0u8;
        *ProductType = 1;
    }

    ExFreePoolWithTag(buffer_key, PRODUCT_TAG);
    success
}

#[repr(C)]
struct KEY_VALUE_PARTIAL_INFORMATION {
    TitleIndex: ULONG,
    Type: ULONG,
    DataLength: ULONG,
    Data: [UCHAR; 1],
}

// i386's `@RtlPrefetchMemoryNonTemporal@8` is prefetch_asm.rs (the prefetch.S port):
// it starts with `ret`, which ke/i386/kiinit.rs patches to a NOP at boot to enable the
// loop. An empty Rust stub here binds the same name and the patch then runs it off its
// end (i386 BUGCHECK 0x1E, EIP=0, in kmtest RtlMemoryKM). Non-x86 only.
#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn RtlPrefetchMemoryNonTemporal(_Source: PVOID, _Length: SIZE_T) {}
#[no_mangle]
pub unsafe extern "system" fn AVrfInternalHeapFreeNotification(
    _AllocationBase: PVOID,
    _AllocationSize: SIZE_T,
) {
    let _AllocationBase = opaque_ptr(_AllocationBase);
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:       See COPYING in the top level directory
//  * PROJECT:         ReactOS kernel
//  * FILE:            ntoskrnl/rtl/misc.c
//  * PURPOSE:         Various functions
//  *
//  * PROGRAMMERS:
//  */
//
// /* INCLUDES *****************************************************************/
//
// #include <ntoskrnl.h>
// #define NDEBUG
// #include <debug.h>
//
// /* GLOBALS *******************************************************************/
//
// extern ULONG NtGlobalFlag;
// extern ULONG NtMajorVersion;
// extern ULONG NtMinorVersion;
// extern ULONG NtOSCSDVersion;
// #define PRODUCT_TAG 'iPtR'
//
// /* FUNCTIONS *****************************************************************/
//
// /*
// * @implemented
// */
// ULONG
// NTAPI
// RtlGetNtGlobalFlags(VOID)
// {
//     return NtGlobalFlag;
// }
//
// /*
//  * @implemented
//  */
// NTSTATUS
// NTAPI
// RtlGetVersion(IN OUT PRTL_OSVERSIONINFOW lpVersionInformation)
// {
//     PAGED_CODE();
//
//     /* Return the basics */
//     lpVersionInformation->dwMajorVersion = NtMajorVersion;
//     lpVersionInformation->dwMinorVersion = NtMinorVersion;
//     lpVersionInformation->dwBuildNumber = NtBuildNumber & 0x3FFF;
//     lpVersionInformation->dwPlatformId = VER_PLATFORM_WIN32_NT;
//
//     /* Check if this is the extended version */
//     if (lpVersionInformation->dwOSVersionInfoSize == sizeof(RTL_OSVERSIONINFOEXW))
//     {
//         PRTL_OSVERSIONINFOEXW InfoEx = (PRTL_OSVERSIONINFOEXW)lpVersionInformation;
//         InfoEx->wServicePackMajor = (USHORT)(CmNtCSDVersion >> 8) & 0xFF;
//         InfoEx->wServicePackMinor = (USHORT)(CmNtCSDVersion & 0xFF);
//         InfoEx->wSuiteMask = (USHORT)(SharedUserData->SuiteMask & 0xFFFF);
//         InfoEx->wProductType = SharedUserData->NtProductType;
//         InfoEx->wReserved = 0;
//     }
//
//     /* Always succeed */
//     return STATUS_SUCCESS;
// }
//
// /**
//  * @brief
//  * Retrieves the NT type product of the operating system. This is the kernel-mode variant
//  * of this function.
//  *
//  * @param[out]  ProductType
//  *      The NT type product enumeration value returned by the call.
//  *
//  * @return
//  *      The function returns TRUE when the call successfully returned the type product of the system.
//  *      It'll return FALSE on failure otherwise. In the latter case the function will return WinNT
//  *      as the default product type.
//  *
//  * @remarks
//  *      The call expects to be called at PASSIVE_LEVEL. The function firstly checks if the product type is
//  *      actually valid by checking the "ProductTypeIsValid" member of _KUSER_SHARED_DATA structure.
//  *      Currently we do not implement code that is responsible for the management of this member, yet.
//  *
//  */
// BOOLEAN
// NTAPI
// RtlGetNtProductType(OUT PNT_PRODUCT_TYPE ProductType)
// {
//     HANDLE Key;
//     BOOLEAN Success;
//     ULONG ReturnedLength;
//     OBJECT_ATTRIBUTES ObjectAttributes;
//     NTSTATUS Status;
//     PKEY_VALUE_PARTIAL_INFORMATION BufferKey;
//     ULONG BufferKeyLength = sizeof(PKEY_VALUE_PARTIAL_INFORMATION) + (256 * sizeof(WCHAR));
//     UNICODE_STRING NtProductType;
//     static UNICODE_STRING WorkstationProduct = RTL_CONSTANT_STRING(L"WinNT");
//     static UNICODE_STRING LanManProduct = RTL_CONSTANT_STRING(L"LanmanNT");
//     static UNICODE_STRING ServerProduct = RTL_CONSTANT_STRING(L"ServerNT");
//     static UNICODE_STRING KeyProduct = RTL_CONSTANT_STRING(L"\\Registry\\Machine\\SYSTEM\\CurrentControlSet\\Control\\ProductOptions");
//     static UNICODE_STRING ValueNtProduct = RTL_CONSTANT_STRING(L"ProductType");
//
//     PAGED_CODE();
//
//     /* Before doing anything else we must allocate some buffer space in the pool */
//     BufferKey = ExAllocatePoolWithTag(PagedPool, BufferKeyLength, PRODUCT_TAG);
//     if (!BufferKey)
//     {
//         /* We failed to allocate pool memory, bail out */
//         DPRINT1("RtlGetNtProductType(): Memory pool allocation has failed!\n");
//         Success = FALSE;
//         goto Exit;
//     }
//
//     /* Initialize the object attributes to open our key */
//     InitializeObjectAttributes(&ObjectAttributes,
//                                &KeyProduct,
//                                OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE,
//                                NULL,
//                                NULL);
//
//     /* Open the key */
//     Status = ZwOpenKey(&Key, KEY_QUERY_VALUE, &ObjectAttributes);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("RtlGetNtProductType(): The ZwOpenKey() function has failed! (Status: 0x%lx)\n", Status);
//         Success = FALSE;
//         goto Exit;
//     }
//
//     /* Now it's time to query the value from the key to check what kind of product the OS is */
//     Status = ZwQueryValueKey(Key,
//                              &ValueNtProduct,
//                              KeyValuePartialInformation,
//                              BufferKey,
//                              BufferKeyLength,
//                              &ReturnedLength);
//
//     /* Free the key from the memory */
//     ZwClose(Key);
//
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("RtlGetNtProductType(): The ZwQueryValueKey() function has failed! (Status: 0x%lx)\n", Status);
//         Success = FALSE;
//         goto Exit;
//     }
//
//     /*
//      * Do a sanity check before we get the product type of the operating system
//      * so that we're sure the type of the value we've got is actually correct.
//      */
//     if (BufferKey->Type != REG_SZ)
//     {
//         /* We've got something else, so bail out */
//         DPRINT1("RtlGetNtProductType(): An invalid value type has been found!\n");
//         Success = FALSE;
//         goto Exit;
//     }
//
//     /* Initialise the Unicode string with the data from the registry value */
//     NtProductType.Length = NtProductType.MaximumLength = BufferKey->DataLength;
//     NtProductType.Buffer = (PWCHAR)BufferKey->Data;
//
//     if (NtProductType.Length > sizeof(WCHAR) && NtProductType.Buffer[NtProductType.Length /
//         sizeof(WCHAR) - 1] == UNICODE_NULL)
//     {
//         NtProductType.Length -= sizeof(WCHAR);
//     }
//
//     /* Now it's time to get the product based on the value data */
//     if (RtlCompareUnicodeString(&NtProductType, &WorkstationProduct, TRUE) == 0)
//     {
//         /* The following product is an OS Workstation */
//         *ProductType = NtProductWinNt;
//         Success = TRUE;
//     }
//     else if (RtlCompareUnicodeString(&NtProductType, &LanManProduct, TRUE) == 0)
//     {
//         /* The following product is an OS Advanced Server */
//         *ProductType = NtProductLanManNt;
//         Success = TRUE;
//     }
//     else if (RtlCompareUnicodeString(&NtProductType, &ServerProduct, TRUE) == 0)
//     {
//         /* The following product is an OS Server */
//         *ProductType = NtProductServer;
//         Success = TRUE;
//     }
//     else
//     {
//         /* None of the product types match so bail out */
//         DPRINT1("RtlGetNtProductType(): Couldn't find a valid product type! Defaulting to WinNT...\n");
//         Success = FALSE;
//         goto Exit;
//     }
//
// Exit:
//     if (!Success)
//     {
//         *ProductType = NtProductWinNt;
//     }
//
//     ExFreePoolWithTag(BufferKey, PRODUCT_TAG);
//     return Success;
// }
//
// #if !defined(_M_IX86)
// //
// // Stub for architectures which don't have this implemented
// //
// VOID
// FASTCALL
// RtlPrefetchMemoryNonTemporal(IN PVOID Source,
//                              IN SIZE_T Length)
// {
//     //
//     // Do nothing
//     //
//     UNREFERENCED_PARAMETER(Source);
//     UNREFERENCED_PARAMETER(Length);
// }
// #endif
//
// VOID NTAPI
// AVrfInternalHeapFreeNotification(PVOID AllocationBase, SIZE_T AllocationSize)
// {
//     /* Stub for linking against rtl */
// }
//
//
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
