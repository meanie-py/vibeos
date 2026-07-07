use ntkrnl_sys::globals::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::ex::*;
use ntkrnl_sys::mm::*;

use ntkrnl_sys::rtl::*;

use ntkrnl_sys::ddk::{
    _UNICODE_STRING,
    ExFreePool,
    PKEY_VALUE_FULL_INFORMATION,
    PUNICODE_STRING,
};
use ntkrnl_sys::{opaque_ptr, opaque_ptr_const};

use core::ffi::c_void;

const REG_MULTI_SZ: u32 = 7;
const STATUS_INVALID_PARAMETER: NTSTATUS = 0xC000000D_u32 as NTSTATUS;

type WCHAR = u16;
type PWCHAR = *mut WCHAR;
type PUSHORT = *mut u16;
type ULONG_PTR = usize;
type SIZE_T = usize;
const UNICODE_NULL: WCHAR = 0;

#[no_mangle]
pub unsafe extern "system" fn PnpFreeUnicodeStringList(
    UnicodeStringList: PUNICODE_STRING,
    StringCount: ULONG,
) {
    let UnicodeStringList = opaque_ptr(UnicodeStringList);
    if !UnicodeStringList.is_null() {
        for i in 0..StringCount {
            if !(*UnicodeStringList.add(i as usize)).Buffer.is_null() {
                ExFreePool((*UnicodeStringList.add(i as usize)).Buffer as *mut c_void);
            }
        }
        ExFreePool(UnicodeStringList as *mut c_void);
    }
}

#[no_mangle]
pub unsafe extern "system" fn PnpRegMultiSzToUnicodeStrings(
    KeyValueInformation: PKEY_VALUE_FULL_INFORMATION,
    UnicodeStringList: *mut PUNICODE_STRING,
    UnicodeStringCount: *mut ULONG,
) -> NTSTATUS {
    let UnicodeStringList = opaque_ptr(UnicodeStringList);
    let UnicodeStringCount = opaque_ptr(UnicodeStringCount);
    let mut p: PWCHAR;
    let mut pp: PWCHAR;
    let mut ps: PWCHAR;
    let mut i: ULONG = 0;
    let mut n: SIZE_T;
    let mut Count: ULONG = 0;

    if (*KeyValueInformation).Type != REG_MULTI_SZ {
        return STATUS_INVALID_PARAMETER;
    }

    p = (KeyValueInformation as ULONG_PTR + (*KeyValueInformation).DataOffset as ULONG_PTR) as PWCHAR;
    pp = (p as ULONG_PTR + (*KeyValueInformation).DataLength as ULONG_PTR) as PWCHAR;

    while p != pp {
        if *p == 0 {
            Count += 1;
            if p.add(1) == pp || *p.add(1) == 0 {
                break;
            }
        }
        p = p.add(1);
    }

    if p == pp {
        Count += 1;
    }

    let unicode_string_list = ExAllocatePoolWithTag(
        PagedPool,
        core::mem::size_of::<_UNICODE_STRING>() * Count as usize,
        0x73557050, // 'sUpP'
    ) as PUNICODE_STRING;
    core::ptr::write_volatile(UnicodeStringList, unicode_string_list);
    if unicode_string_list.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    ps = (KeyValueInformation as ULONG_PTR + (*KeyValueInformation).DataOffset as ULONG_PTR) as PWCHAR;
    p = ps;

    while p != pp {
        if *p == 0 {
            n = (p as ULONG_PTR - ps as ULONG_PTR) + core::mem::size_of::<WCHAR>();

            (*unicode_string_list.add(i as usize)).Buffer = ExAllocatePoolWithTag(
                PagedPool,
                n,
                0x73557050, // 'sUpP'
            ) as PWCHAR;
            if (*unicode_string_list.add(i as usize)).Buffer.is_null() {
                PnpFreeUnicodeStringList(unicode_string_list, i);
                return STATUS_INSUFFICIENT_RESOURCES;
            }

            core::ptr::copy_nonoverlapping(
                ps as *const u8,
                (*unicode_string_list.add(i as usize)).Buffer as *mut u8,
                n,
            );

            (*unicode_string_list.add(i as usize)).MaximumLength = n as u16;
            (*unicode_string_list.add(i as usize)).Length = (n - core::mem::size_of::<WCHAR>()) as u16;

            i += 1;

            if p.add(1) == pp || *p.add(1) == 0 {
                break;
            }

            ps = p.add(1);
        }

        p = p.add(1);
    }

    if p == pp {
        n = p as ULONG_PTR - ps as ULONG_PTR;

        (*unicode_string_list.add(i as usize)).Buffer = ExAllocatePoolWithTag(
            PagedPool,
            n + core::mem::size_of::<WCHAR>(),
            0x73557050, // 'sUpP'
        ) as PWCHAR;
        if (*unicode_string_list.add(i as usize)).Buffer.is_null() {
            PnpFreeUnicodeStringList(unicode_string_list, i);
            return STATUS_INSUFFICIENT_RESOURCES;
        }

        if n != 0 {
            core::ptr::copy_nonoverlapping(
                ps as *const u8,
                (*unicode_string_list.add(i as usize)).Buffer as *mut u8,
                n,
            );
        }

        *(*unicode_string_list.add(i as usize)).Buffer.add(n / core::mem::size_of::<WCHAR>()) = UNICODE_NULL;

        (*unicode_string_list.add(i as usize)).Length = n as u16;
        (*unicode_string_list.add(i as usize)).MaximumLength = (n + core::mem::size_of::<WCHAR>()) as u16;
    }

    core::ptr::write_volatile(UnicodeStringCount, Count);
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn PnpRegSzToString(
    RegSzData: PWCHAR,
    RegSzLength: ULONG,
    StringLength: PUSHORT,
) -> BOOLEAN {
    let StringLength = opaque_ptr(StringLength);
    let pp: PWCHAR = RegSzData.add(RegSzLength as usize / core::mem::size_of::<WCHAR>());
    let mut p: PWCHAR = RegSzData;

    while p < pp {
        if *p == 0 {
            break;
        }
        p = p.add(1);
    }

    if !StringLength.is_null() {
        core::ptr::write_volatile(
            StringLength,
            ((p as usize - RegSzData as usize) / core::mem::size_of::<WCHAR>() * core::mem::size_of::<WCHAR>()) as u16,
        );
    }

    1u8
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         ReactOS Kernel
//  * LICENSE:         BSD - See COPYING.ARM in the top level directory
//  * FILE:            ntoskrnl/io/pnpmgr/pnputil.c
//  * PURPOSE:         PnP Utility Code
//  * PROGRAMMERS:     ReactOS Portable Systems Group
//  */
//
// /* INCLUDES *******************************************************************/
//
// #include <ntoskrnl.h>
// #define NDEBUG
// #include <debug.h>
//
// /* GLOBALS ********************************************************************/
//
// /* FUNCTIONS ******************************************************************/
//
// VOID
// NTAPI
// PnpFreeUnicodeStringList(IN PUNICODE_STRING UnicodeStringList,
//                          IN ULONG StringCount)
// {
//     ULONG i;
//
//     /* Go through the list */
//     if (UnicodeStringList)
//     {
//         /* Go through each string */
//         for (i = 0; i < StringCount; i++)
//         {
//             /* Check if it exists */
//             if (UnicodeStringList[i].Buffer)
//             {
//                 /* Free it */
//                 ExFreePool(UnicodeStringList[i].Buffer);
//             }
//         }
//
//         /* Free the whole list */
//         ExFreePool(UnicodeStringList);
//     }
// }
//
// NTSTATUS
// NTAPI
// PnpRegMultiSzToUnicodeStrings(IN PKEY_VALUE_FULL_INFORMATION KeyValueInformation,
//                               OUT PUNICODE_STRING *UnicodeStringList,
//                               OUT PULONG UnicodeStringCount)
// {
//     PWCHAR p, pp, ps;
//     ULONG i = 0;
//     SIZE_T n;
//     ULONG Count = 0;
//
//     /* Validate the key information */
//     if (KeyValueInformation->Type != REG_MULTI_SZ) return STATUS_INVALID_PARAMETER;
//
//     /* Set the pointers */
//     p = (PWCHAR)((ULONG_PTR)KeyValueInformation +
//                  KeyValueInformation->DataOffset);
//     pp = (PWCHAR)((ULONG_PTR)p + KeyValueInformation->DataLength);
//
//     /* Loop the data */
//     while (p != pp)
//     {
//         /* If we find a NULL, that means one string is done */
//         if (!*p)
//         {
//             /* Add to our string count */
//             Count++;
//
//             /* Check for a double-NULL, which means we're done */
//             if (((p + 1) == pp) || !(*(p + 1))) break;
//         }
//
//         /* Go to the next character */
//         p++;
//     }
//
//     /* If we looped the whole list over, we missed increment a string, do it */
//     if (p == pp) Count++;
//
//     /* Allocate the list now that we know how big it is */
//     *UnicodeStringList = ExAllocatePoolWithTag(PagedPool,
//                                                sizeof(UNICODE_STRING) * Count,
//                                                'sUpP');
//     if (!(*UnicodeStringList)) return STATUS_INSUFFICIENT_RESOURCES;
//
//     /* Set pointers for second loop */
//     ps = p = (PWCHAR)((ULONG_PTR)KeyValueInformation +
//                      KeyValueInformation->DataOffset);
//
//     /* Loop again, to do the copy this time */
//     while (p != pp)
//     {
//         /* If we find a NULL, that means one string is done */
//         if (!*p)
//         {
//             /* Check how long this string is */
//             n = (ULONG_PTR)p - (ULONG_PTR)ps + sizeof(UNICODE_NULL);
//
//             /* Allocate the buffer */
//             (*UnicodeStringList)[i].Buffer = ExAllocatePoolWithTag(PagedPool,
//                                                                    n,
//                                                                    'sUpP');
//             if (!(*UnicodeStringList)[i].Buffer)
//             {
//                 /* Back out of everything */
//                 PnpFreeUnicodeStringList(*UnicodeStringList, i);
//                 return STATUS_INSUFFICIENT_RESOURCES;
//             }
//
//             /* Copy the string into the buffer */
//             RtlCopyMemory((*UnicodeStringList)[i].Buffer, ps, n);
//
//             /* Set the lengths */
//             (*UnicodeStringList)[i].MaximumLength = (USHORT)n;
//             (*UnicodeStringList)[i].Length = (USHORT)(n - sizeof(UNICODE_NULL));
//
//             /* One more entry done */
//             i++;
//
//             /* Check for a double-NULL, which means we're done */
//             if (((p + 1) == pp) || !(*(p + 1))) break;
//
//             /* New string */
//             ps = p + 1;
//         }
//
//         /* New string */
//         p++;
//     }
//
//     /* Check if we've reached the last string */
//     if (p == pp)
//     {
//         /* Calculate the string length */
//         n = (ULONG_PTR)p - (ULONG_PTR)ps;
//
//         /* Allocate the buffer for it */
//         (*UnicodeStringList)[i].Buffer = ExAllocatePoolWithTag(PagedPool,
//                                                                n +
//                                                                sizeof(UNICODE_NULL),
//                                                                'sUpP');
//         if (!(*UnicodeStringList)[i].Buffer)
//         {
//             /* Back out of everything */
//             PnpFreeUnicodeStringList(*UnicodeStringList, i);
//             return STATUS_INSUFFICIENT_RESOURCES;
//         }
//
//         /* Make sure there's an actual string here */
//         if (n) RtlCopyMemory((*UnicodeStringList)[i].Buffer, ps, n);
//
//         /* Null-terminate the string ourselves */
//         (*UnicodeStringList)[i].Buffer[n / sizeof(WCHAR)] = UNICODE_NULL;
//
//         /* Set the lengths */
//         (*UnicodeStringList)[i].Length = (USHORT)n;
//         (*UnicodeStringList)[i].MaximumLength = (USHORT)(n + sizeof(UNICODE_NULL));
//     }
//
//     /* And we're done */
//     *UnicodeStringCount = Count;
//     return STATUS_SUCCESS;
// }
//
// BOOLEAN
// NTAPI
// PnpRegSzToString(IN PWCHAR RegSzData,
//                  IN ULONG RegSzLength,
//                  OUT PUSHORT StringLength OPTIONAL)
// {
//     PWCHAR p, pp;
//
//     /* Find the end */
//     pp = RegSzData + RegSzLength / sizeof(WCHAR);
//     for (p = RegSzData; p < pp; p++)
//     {
//         if (!*p)
//         {
//             break;
//         }
//     }
//
//     /* Return the length. Truncation can happen but is of no consequence. */
//     if (StringLength)
//     {
//         *StringLength = (USHORT)(p - RegSzData) * sizeof(WCHAR);
//     }
//     return TRUE;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
