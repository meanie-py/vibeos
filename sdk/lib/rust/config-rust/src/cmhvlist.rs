use ntkrnl_sys::globals::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::ex::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ddk::{CmpCompressedNameSize, CmpCopyCompressedName, HCELL_INDEX, _OBJECT_ATTRIBUTES, OBJECT_NAME_INFORMATION, ObCloseHandle, PCELL_DATA, PCMHIVE, POBJECT_NAME_INFORMATION, PUNICODE_STRING, RtlInitEmptyUnicodeString, _UNICODE_STRING, ZwCreateKey, ZwDeleteValueKey, ZwOpenKey, ZwQueryObject, ZwSetValueKey};
use core::ffi::c_void;

use ntkrnl_sys::opaque_ptr;

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

const TAG_CM: u32 = u32::from_le_bytes(*b"CM  ");
// sdk/lib/cmlib/cmdata.h. Every use below decides whether a key's name is
// stored COMPRESSED (one byte per char) or as UTF-16, and each was written as
// the bare literal 0x0001 — which is KEY_IS_VOLATILE, not KEY_COMP_NAME. So
// this file read every compressed name as UTF-16 and every UTF-16 name as
// compressed, sizing and copying the hive path wrongly either way.
const KEY_COMP_NAME: u16 = 0x0020;
const KEY_READ: u32 = 0x00020019;
const KEY_WRITE: u32 = 0x00020006;

extern "C" {
    pub static mut CmiVolatileHive: PCMHIVE;
}

static HIVELIST_NAME_BUF: [u16; 60] = [
    0x005C, 0x0052, 0x0045, 0x0047, 0x0049, 0x0053, 0x0054, 0x0052, 0x0059, 0x005C,
    0x004D, 0x0041, 0x0043, 0x0048, 0x0049, 0x004E, 0x0045, 0x005C, 0x0053, 0x0059,
    0x0053, 0x0054, 0x0045, 0x004D, 0x005C, 0x0043, 0x0075, 0x0072, 0x0072, 0x0065,
    0x006E, 0x0074, 0x0043, 0x006F, 0x006E, 0x0074, 0x0072, 0x006F, 0x006C, 0x0053,
    0x0065, 0x0074, 0x005C, 0x0043, 0x006F, 0x006E, 0x0074, 0x0072, 0x006F, 0x006C,
    0x005C, 0x0068, 0x0069, 0x0076, 0x0065, 0x006C, 0x0069, 0x0073, 0x0074, 0x0000,
];

#[no_mangle]
pub static mut HiveListValueName: _UNICODE_STRING = _UNICODE_STRING {
    Length: 118,
    MaximumLength: 120,
    Buffer: HIVELIST_NAME_BUF.as_ptr() as *mut u16,
};

extern "C" {
    fn HvGetCell(Hive: *mut core::ffi::c_void, Cell: HCELL_INDEX) -> PCELL_DATA;
    fn HvReleaseCell(Hive: *mut core::ffi::c_void, Cell: HCELL_INDEX);
}

#[no_mangle]
pub unsafe extern "system" fn CmpGetHiveName(
    Hive: PCMHIVE,
    HiveName: PUNICODE_STRING,
) -> u8 {
    let Hive = opaque_ptr(Hive);
    let HiveName = opaque_ptr(HiveName);
    let mut RootCell: HCELL_INDEX;
    let mut LinkCell: HCELL_INDEX;
    let mut RootData: PCELL_DATA;
    let mut LinkData: PCELL_DATA;
    let mut ParentData: PCELL_DATA;
    let mut ParentNameSize: u32;
    let mut LinkNameSize: u32;
    let NameSize: usize;
    let mut p: *mut u16;
    let mut RegistryName: _UNICODE_STRING = _UNICODE_STRING {
        Length: 20,
        MaximumLength: 22,
        Buffer: w!("\\REGISTRY\\").as_ptr() as *mut u16,
    };

    RootCell = (*(*Hive).Hive.BaseBlock).RootCell;
    RootData = HvGetCell(core::ptr::addr_of_mut!((*Hive).Hive) as *mut _, RootCell);
    if RootData.is_null() {
        return 0u8;
    }

    LinkCell = (*RootData).u.KeyNode.Parent;
    HvReleaseCell(core::ptr::addr_of_mut!((*Hive).Hive) as *mut _, RootCell);

    LinkData = HvGetCell(core::ptr::addr_of_mut!((*CmiVolatileHive).Hive) as *mut _, LinkCell);
    if LinkData.is_null() {
        return 0u8;
    }
    ParentData = HvGetCell(
        core::ptr::addr_of_mut!((*CmiVolatileHive).Hive) as *mut _,
        (*LinkData).u.KeyNode.Parent,
    );
    if ParentData.is_null() {
        return 0u8;
    }

    if (*ParentData).u.KeyNode.Flags & KEY_COMP_NAME != 0 {
        ParentNameSize = CmpCompressedNameSize(
            core::ptr::addr_of!((*ParentData).u.KeyNode.Name) as *mut u16,
            (*ParentData).u.KeyNode.NameLength as u32,
        ) as u32;
    } else {
        ParentNameSize = (*ParentData).u.KeyNode.NameLength as u32;
    }

    if (*LinkData).u.KeyNode.Flags & KEY_COMP_NAME != 0 {
        LinkNameSize = CmpCompressedNameSize(
            core::ptr::addr_of!((*LinkData).u.KeyNode.Name) as *mut u16,
            (*LinkData).u.KeyNode.NameLength as u32,
        ) as u32;
    } else {
        LinkNameSize = (*LinkData).u.KeyNode.NameLength as u32;
    }

    NameSize = RegistryName.Length as usize + ParentNameSize as usize + 2 + LinkNameSize as usize;

    (*HiveName).Buffer = ExAllocatePoolWithTag(PagedPool, NameSize, 0x20436d43u32) as *mut u16;
    if (*HiveName).Buffer.is_null() {
        crate::dbg_print!(b"CmpGetHiveName: Unable to allocate memory\n\0");
        return 0u8;
    }

    (*HiveName).Length = NameSize as u16;
    (*HiveName).MaximumLength = NameSize as u16;
    p = (*HiveName).Buffer;

    core::ptr::copy_nonoverlapping(
        RegistryName.Buffer,
        p,
        RegistryName.Length as usize / 2,
    );
    p = p.add(RegistryName.Length as usize / 2);
    if (*ParentData).u.KeyNode.Flags & KEY_COMP_NAME != 0 {
        CmpCopyCompressedName(
            p,
            ParentNameSize,
            core::ptr::addr_of!((*ParentData).u.KeyNode.Name) as *mut u16,
            (*ParentData).u.KeyNode.NameLength as u32,
        );
    } else {
        core::ptr::copy_nonoverlapping(
            core::ptr::addr_of!((*ParentData).u.KeyNode.Name) as *const u16,
            p,
            ParentNameSize as usize / 2,
        );
    }

    p = p.add(ParentNameSize as usize / 2);
    *p = b'\\' as u16;
    p = p.add(1);

    if (*LinkData).u.KeyNode.Flags & KEY_COMP_NAME != 0 {
        CmpCopyCompressedName(
            p,
            LinkNameSize,
            core::ptr::addr_of!((*LinkData).u.KeyNode.Name) as *mut u16,
            (*LinkData).u.KeyNode.NameLength as u32,
        );
    } else {
        core::ptr::copy_nonoverlapping(
            core::ptr::addr_of!((*LinkData).u.KeyNode.Name) as *const u16,
            p,
            LinkNameSize as usize / 2,
        );
    }

    return 1u8;
}

#[no_mangle]
pub unsafe extern "system" fn CmpAddToHiveFileList(Hive: PCMHIVE) -> i32 {
    let Hive = opaque_ptr(Hive);
    let mut Status: i32;
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut KeyHandle: *mut core::ffi::c_void = core::ptr::null_mut();
    let mut HivePath: _UNICODE_STRING = core::mem::zeroed();
    let mut FilePath: *mut u16;
    let mut Length: u32 = 0;
    let mut DummyNameInfo: OBJECT_NAME_INFORMATION = core::mem::zeroed();
    let mut FileNameInfo: POBJECT_NAME_INFORMATION = core::ptr::null_mut();

    HivePath.Buffer = core::ptr::null_mut();

    ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as u32;
    ObjectAttributes.RootDirectory = core::ptr::null_mut();
    ObjectAttributes.Attributes = 0x00000040u32 | 0x00000200u32;
    ObjectAttributes.ObjectName = core::ptr::addr_of_mut!(HiveListValueName);
    ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
    ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

    Status = ZwCreateKey(
        core::ptr::addr_of_mut!(KeyHandle),
        // Upstream 396019984c9: we only ever set a value here, so ask for exactly that.
        KEY_SET_VALUE,
        core::ptr::addr_of_mut!(ObjectAttributes),
        0,
        core::ptr::null_mut(),
        0x00000001u32,
        core::ptr::null_mut(),
    );
    if !NT_SUCCESS(Status) {
        crate::dbg_print!(
            b"CmpAddToHiveFileList: Creation or opening of the hive list failed, status = 0x%08lx\n\0",
            Status
        );
        return Status;
    }

    if CmpGetHiveName(Hive, core::ptr::addr_of_mut!(HivePath)) == 0 {
        crate::dbg_print!(b"CmpAddToHiveFileList: Unable to retrieve the hive name\n\0");
        Status = STATUS_NO_MEMORY;
        'Quickie: loop {
            if !HivePath.Buffer.is_null() {
                ExFreePoolWithTag(HivePath.Buffer as *mut core::ffi::c_void, TAG_CM);
            }
            if !FileNameInfo.is_null() {
                ExFreePoolWithTag(FileNameInfo as *mut core::ffi::c_void, TAG_CM);
            }
            ObCloseHandle(KeyHandle, 0i8);
            return Status;
        }
    }

    if ((*Hive).Hive.HiveFlags & 0x00000001u32) == 0 {
        Status = ZwQueryObject(
            (*Hive).FileHandles[0],
            1u32,
            core::ptr::addr_of_mut!(DummyNameInfo) as *mut core::ffi::c_void,
            core::mem::size_of_val(&DummyNameInfo) as u32,
            core::ptr::addr_of_mut!(Length),
        );
        if Status != 0x80000005u32 as i32 {
            crate::dbg_print!(
                b"CmpAddToHiveFileList: Hive file name size query failed, status = 0x%08lx\n\0",
                Status
            );
            'Quickie: loop {
                if !HivePath.Buffer.is_null() {
                    ExFreePoolWithTag(HivePath.Buffer as *mut core::ffi::c_void, TAG_CM);
                }
                if !FileNameInfo.is_null() {
                    ExFreePoolWithTag(FileNameInfo as *mut core::ffi::c_void, TAG_CM);
                }
                ObCloseHandle(KeyHandle, 0i8);
                return Status;
            }
        }

        FileNameInfo = ExAllocatePoolWithTag(
            PagedPool,
            (Length as usize) + core::mem::size_of::<u16>(),
            TAG_CM,
        ) as POBJECT_NAME_INFORMATION;
        if FileNameInfo.is_null() {
            Status = STATUS_INSUFFICIENT_RESOURCES;
            'Quickie: loop {
                if !HivePath.Buffer.is_null() {
                    ExFreePoolWithTag(HivePath.Buffer as *mut core::ffi::c_void, TAG_CM);
                }
                if !FileNameInfo.is_null() {
                    ExFreePoolWithTag(FileNameInfo as *mut core::ffi::c_void, TAG_CM);
                }
                ObCloseHandle(KeyHandle, 0i8);
                return Status;
            }
        }

        Status = ZwQueryObject(
            (*Hive).FileHandles[0],
            1u32,
            FileNameInfo as *mut core::ffi::c_void,
            Length,
            core::ptr::addr_of_mut!(Length),
        );
        if NT_SUCCESS(Status) {
            Length -= core::mem::size_of::<OBJECT_NAME_INFORMATION>() as u32;
            FilePath = (*FileNameInfo).Name.Buffer;
            *FilePath.offset((Length / core::mem::size_of::<u16>() as u32) as isize) = 0u16;
            Length += core::mem::size_of::<u16>() as u32;
        } else {
            crate::dbg_print!(
                b"CmpAddToHiveFileList: Hive file name query failed, status = 0x%08lx\n\0",
                Status
            );
            'Quickie: loop {
                if !HivePath.Buffer.is_null() {
                    ExFreePoolWithTag(HivePath.Buffer as *mut core::ffi::c_void, TAG_CM);
                }
                if !FileNameInfo.is_null() {
                    ExFreePoolWithTag(FileNameInfo as *mut core::ffi::c_void, TAG_CM);
                }
                ObCloseHandle(KeyHandle, 0i8);
                return Status;
            }
        }
    } else {
        FilePath = w!("").as_ptr() as *mut u16;
        Length = core::mem::size_of::<u16>() as u32;
    }

    Status = ZwSetValueKey(
        KeyHandle,
        core::ptr::addr_of_mut!(HivePath),
        0,
        1u32,
        FilePath as *mut core::ffi::c_void,
        Length,
    );
    if !NT_SUCCESS(Status) {
        crate::dbg_print!(
            b"CmpAddToHiveFileList: Setting of entry in the hive list failed, status = 0x%08lx\n\0",
            Status
        );
    }

    // Capture the hive path and reset the local buffer so the cleanup below does not
    // free it. Upstream 396019984c9: CmpRemoveFromHiveFileList() used to RE-DERIVE this
    // path from the hive at unload time, by which point CmpUnlinkFromMasterHive() had
    // already invalidated the LinkData node it reads — so the name came back wrong (or
    // with a bogus KEY_COMP_NAME flag) and the wrong value name was looked up. Capturing
    // it here, while the node is still valid, is the fix.
    //
    // Only the success path reaches this point: every failure above returns out of a
    // 'Quickie block, which is this translation's spelling of the C's `goto Quit` — and
    // the C's capture likewise sits after the last goto, so a failed add captures nothing
    // and CmpRemoveFromHiveFileList() then sees a NULL Buffer and does nothing.
    (*Hive).HiveRootPath = HivePath;

    // The C nulls HivePath.Buffer here so its shared `Quit:` cleanup skips the free. This
    // translation has no shared cleanup — each failure path returns its own copy — so the
    // buffer simply is not freed on the one path that reaches here, and the null-and-test
    // dance would be dead code. The hive owns the allocation now; CmpRemoveFromHiveFileList
    // frees it.
    if !FileNameInfo.is_null() {
        ExFreePoolWithTag(FileNameInfo as *mut core::ffi::c_void, TAG_CM);
    }
    ObCloseHandle(KeyHandle, 0i8);
    Status
}

#[no_mangle]
pub unsafe extern "system" fn CmpRemoveFromHiveFileList(Hive: PCMHIVE) {
    let Hive = opaque_ptr(Hive);
    let Status: i32;
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut KeyHandle: *mut core::ffi::c_void = core::ptr::null_mut();

    // Do nothing if no hive path was captured, e.g. CmpAddToHiveFileList() failed.
    if (*Hive).HiveRootPath.Buffer.is_null() {
        return;
    }

    ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as u32;
    ObjectAttributes.RootDirectory = core::ptr::null_mut();
    ObjectAttributes.Attributes = 0x00000040 | 0x00000200;
    ObjectAttributes.ObjectName = core::ptr::addr_of_mut!(HiveListValueName);
    ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
    ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

    Status = ZwOpenKey(
        core::ptr::addr_of_mut!(KeyHandle),
        // Upstream 396019984c9: only a value is deleted here.
        KEY_SET_VALUE,
        core::ptr::addr_of_mut!(ObjectAttributes),
    );
    if !NT_SUCCESS(Status) {
        crate::dbg_print!(
            b"CmpRemoveFromHiveFileList: Opening of the hive list failed, status = 0x%08lx\n\0",
            Status
        );
        return;
    }

    // Delete the hive path from the list. This is the path CAPTURED at add time, not one
    // re-derived now: CmpUnlinkFromMasterHive() has already run by the time CmUnloadKey()
    // gets here, so re-deriving read an invalidated LinkData node. That is why ReactOS
    // left \Registry\Machine\USetup_Software listed in the hivelist after 1st-stage
    // setup had cleanly unloaded it.
    ZwDeleteValueKey(KeyHandle, core::ptr::addr_of_mut!((*Hive).HiveRootPath));

    // Free the captured hive path.
    ExFreePoolWithTag(
        (*Hive).HiveRootPath.Buffer as *mut core::ffi::c_void,
        TAG_CM,
    );
    RtlInitEmptyUnicodeString(
        core::ptr::addr_of_mut!((*Hive).HiveRootPath),
        core::ptr::null_mut(),
        0,
    );

    ObCloseHandle(KeyHandle, 0);
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         ReactOS Kernel
//  * LICENSE:         GPL - See COPYING in the top level directory
//  * FILE:            ntoskrnl/config/cmhvlist.c
//  * PURPOSE:         Configuration Manager - Hives file list management
//  * PROGRAMMERS:     Hermes BELUSCA - MAITO
//  */
//
// /* INCLUDES *******************************************************************/
//
// #include "ntoskrnl.h"
// #define NDEBUG
// #include "debug.h"
//
// /* GLOBALS ********************************************************************/
//
// UNICODE_STRING HiveListValueName = RTL_CONSTANT_STRING(L"\\REGISTRY\\MACHINE\\SYSTEM\\CurrentControlSet\\Control\\hivelist");
//
// /* FUNCTIONS ******************************************************************/
//
// /* Note: the caller is expected to free the HiveName string buffer */
// BOOLEAN
// NTAPI
// CmpGetHiveName(IN PCMHIVE Hive,
//                OUT PUNICODE_STRING HiveName)
// {
//     HCELL_INDEX RootCell, LinkCell;
//     PCELL_DATA RootData, LinkData, ParentData;
//     ULONG ParentNameSize, LinkNameSize;
//     SIZE_T NameSize;
//     PWCHAR p;
//     UNICODE_STRING RegistryName = RTL_CONSTANT_STRING(L"\\REGISTRY\\");
//
//     /* Get the root cell of this hive */
//     RootCell = Hive->Hive.BaseBlock->RootCell;
//     RootData = HvGetCell(&Hive->Hive, RootCell);
//     if (!RootData) return FALSE;
//
//     /* Get the cell index at which this hive is linked to, and its parent */
//     LinkCell = RootData->u.KeyNode.Parent;
//     HvReleaseCell(&Hive->Hive, RootCell);
//
//     /* Sanity check */
//     ASSERT((&CmiVolatileHive->Hive)->ReleaseCellRoutine == NULL);
//
//     /* Get the cell data for link and parent */
//     LinkData = HvGetCell(&CmiVolatileHive->Hive, LinkCell);
//     if (!LinkData) return FALSE;
//     ParentData = HvGetCell(&CmiVolatileHive->Hive, LinkData->u.KeyNode.Parent);
//     if (!ParentData) return FALSE;
//
//     /* Get the size of the parent name */
//     if (ParentData->u.KeyNode.Flags & KEY_COMP_NAME)
//     {
//         ParentNameSize = CmpCompressedNameSize(ParentData->u.KeyNode.Name,
//                                                ParentData->u.KeyNode.NameLength);
//     }
//     else
//     {
//         ParentNameSize = ParentData->u.KeyNode.NameLength;
//     }
//
//     /* Get the size of the link name */
//     if (LinkData->u.KeyNode.Flags & KEY_COMP_NAME)
//     {
//         LinkNameSize = CmpCompressedNameSize(LinkData->u.KeyNode.Name,
//                                              LinkData->u.KeyNode.NameLength);
//     }
//     else
//     {
//         LinkNameSize = LinkData->u.KeyNode.NameLength;
//     }
//
//     /* No need to account for terminal NULL character since we deal with counted UNICODE strings */
//     NameSize = RegistryName.Length + ParentNameSize + sizeof(WCHAR) + LinkNameSize;
//
//     /* Allocate the memory */
//     HiveName->Buffer = ExAllocatePoolWithTag(PagedPool, NameSize, TAG_CM);
//     if (!HiveName->Buffer)
//     {
//         /* Fail */
//         DPRINT1("CmpGetHiveName: Unable to allocate memory\n");
//         return FALSE;
//     }
//
//     /* Build the string for it */
//     HiveName->Length = HiveName->MaximumLength = (USHORT)NameSize;
//     p = HiveName->Buffer;
//
//     /* Copy the parent name */
//     RtlCopyMemory(p, RegistryName.Buffer, RegistryName.Length);
//     p += RegistryName.Length / sizeof(WCHAR);
//     if (ParentData->u.KeyNode.Flags & KEY_COMP_NAME)
//     {
//         CmpCopyCompressedName(p,
//                               ParentNameSize,
//                               ParentData->u.KeyNode.Name,
//                               ParentData->u.KeyNode.NameLength);
//     }
//     else
//     {
//         RtlCopyMemory(p, ParentData->u.KeyNode.Name, ParentNameSize);
//     }
//
//     /* Add a path separator between parent and link */
//     p += ParentNameSize / sizeof(WCHAR);
//     *p = OBJ_NAME_PATH_SEPARATOR;
//     ++p;
//
//     /* Now copy the link name */
//     if (LinkData->u.KeyNode.Flags & KEY_COMP_NAME)
//     {
//         CmpCopyCompressedName(p,
//                               LinkNameSize,
//                               LinkData->u.KeyNode.Name,
//                               LinkData->u.KeyNode.NameLength);
//
//     }
//     else
//     {
//         RtlCopyMemory(p, LinkData->u.KeyNode.Name, LinkNameSize);
//     }
//
//     /* All done */
//     return TRUE;
// }
//
// NTSTATUS
// NTAPI
// CmpAddToHiveFileList(IN PCMHIVE Hive)
// {
//     NTSTATUS Status;
//     OBJECT_ATTRIBUTES ObjectAttributes;
//     HANDLE KeyHandle;
//     UNICODE_STRING HivePath;
//     PWCHAR FilePath;
//     ULONG Length;
//     OBJECT_NAME_INFORMATION DummyNameInfo;
//     POBJECT_NAME_INFORMATION FileNameInfo;
//
//     HivePath.Buffer = NULL;
//     FileNameInfo = NULL;
//
//     /* Create or open the hive list key */
//     InitializeObjectAttributes(&ObjectAttributes,
//                                &HiveListValueName,
//                                OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE,
//                                NULL,
//                                NULL);
//     Status = ZwCreateKey(&KeyHandle,
//                          KEY_READ | KEY_WRITE,
//                          &ObjectAttributes,
//                          0,
//                          NULL,
//                          REG_OPTION_VOLATILE,
//                          NULL);
//     if (!NT_SUCCESS(Status))
//     {
//         /* Fail */
//         DPRINT1("CmpAddToHiveFileList: Creation or opening of the hive list failed, status = 0x%08lx\n", Status);
//         return Status;
//     }
//
//     /* Retrieve the name of the hive */
//     if (!CmpGetHiveName(Hive, &HivePath))
//     {
//         /* Fail */
//         DPRINT1("CmpAddToHiveFileList: Unable to retrieve the hive name\n");
//         Status = STATUS_NO_MEMORY;
//         goto Quickie;
//     }
//
//     /* Get the name of the corresponding file */
//     if (!(Hive->Hive.HiveFlags & HIVE_VOLATILE))
//     {
//         /* Determine the right buffer size and allocate */
//         Status = ZwQueryObject(Hive->FileHandles[HFILE_TYPE_PRIMARY],
//                                ObjectNameInformation,
//                                &DummyNameInfo,
//                                sizeof(DummyNameInfo),
//                                &Length);
//         if (Status != STATUS_BUFFER_OVERFLOW)
//         {
//             DPRINT1("CmpAddToHiveFileList: Hive file name size query failed, status = 0x%08lx\n", Status);
//             goto Quickie;
//         }
//
//         FileNameInfo = ExAllocatePoolWithTag(PagedPool,
//                                              Length + sizeof(UNICODE_NULL),
//                                              TAG_CM);
//         if (FileNameInfo == NULL)
//         {
//             Status = STATUS_INSUFFICIENT_RESOURCES;
//             goto Quickie;
//         }
//
//         /* Try to get the value */
//         Status = ZwQueryObject(Hive->FileHandles[HFILE_TYPE_PRIMARY],
//                                ObjectNameInformation,
//                                FileNameInfo,
//                                Length,
//                                &Length);
//         if (NT_SUCCESS(Status))
//         {
//             /* Null-terminate and add the length of the terminator */
//             Length -= sizeof(OBJECT_NAME_INFORMATION);
//             FilePath = FileNameInfo->Name.Buffer;
//             FilePath[Length / sizeof(WCHAR)] = UNICODE_NULL;
//             Length += sizeof(UNICODE_NULL);
//         }
//         else
//         {
//             /* Fail */
//             DPRINT1("CmpAddToHiveFileList: Hive file name query failed, status = 0x%08lx\n", Status);
//             goto Quickie;
//         }
//     }
//     else
//     {
//         /* No name */
//         FilePath = L"";
//         Length = sizeof(UNICODE_NULL);
//     }
//
//     /* Set the entry in the hive list */
//     Status = ZwSetValueKey(KeyHandle,
//                            &HivePath,
//                            0,
//                            REG_SZ,
//                            FilePath,
//                            Length);
//     if (!NT_SUCCESS(Status))
//     {
//         /* Fail */
//         DPRINT1("CmpAddToHiveFileList: Setting of entry in the hive list failed, status = 0x%08lx\n", Status);
//     }
//
// Quickie:
//     /* Cleanup and return status */
//     if (HivePath.Buffer)
//     {
//         ExFreePoolWithTag(HivePath.Buffer, TAG_CM);
//     }
//     if (FileNameInfo)
//     {
//         ExFreePoolWithTag(FileNameInfo, TAG_CM);
//     }
//     ObCloseHandle(KeyHandle, KernelMode);
//     return Status;
// }
//
// VOID
// NTAPI
// CmpRemoveFromHiveFileList(IN PCMHIVE Hive)
// {
//     NTSTATUS Status;
//     OBJECT_ATTRIBUTES ObjectAttributes;
//     HANDLE KeyHandle;
//     UNICODE_STRING HivePath;
//
//     /* Open the hive list key */
//     InitializeObjectAttributes(&ObjectAttributes,
//                                &HiveListValueName,
//                                OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE,
//                                NULL,
//                                NULL);
//     Status = ZwOpenKey(&KeyHandle,
//                        KEY_READ | KEY_WRITE,
//                        &ObjectAttributes);
//     if (!NT_SUCCESS(Status))
//     {
//         /* Fail */
//         DPRINT1("CmpRemoveFromHiveFileList: Opening of the hive list failed, status = 0x%08lx\n", Status);
//         return;
//     }
//
//     /* Get the hive path name */
//     CmpGetHiveName(Hive, &HivePath);
//
//     /* Delete the hive path name from the list */
//     ZwDeleteValueKey(KeyHandle, &HivePath);
//
//     /* Cleanup allocation and handle */
//     ExFreePoolWithTag(HivePath.Buffer, TAG_CM);
//     ObCloseHandle(KeyHandle, KernelMode);
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
