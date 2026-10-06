use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::macros::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::ex::*;
use ntkrnl_sys::ob::*;
use ntkrnl_sys::probe::*;

use ntkrnl_sys::ddk::{
    _OBJECT_DIRECTORY, _OBJECT_DIRECTORY_ENTRY, _OBJECT_DIRECTORY_INFORMATION,
    _OBP_LOOKUP_CONTEXT,
    POBJECT_DIRECTORY, POBJECT_DIRECTORY_ENTRY, POBJECT_HEADER, POBJECT_HEADER_NAME_INFO,
    POBP_LOOKUP_CONTEXT, PDEVICE_MAP, POBJECT_DIRECTORY_INFORMATION, POBJECT_TYPE,
    POBJECT_ATTRIBUTES, PHANDLE, PUNICODE_STRING,
    ACCESS_MASK, KPROCESSOR_MODE, HANDLE,
    ObpAcquireDirectoryLockShared, ObpReleaseDirectoryLock, ObpInitializeLookupContext,
    ObpReleaseLookupContextObject, ObpReferenceNameInfo,
    RtlEqualUnicodeString, RtlUpcaseUnicodeChar,
    ExConvertPushLockSharedToExclusive, ExInitializePushLock,
    ObOpenObjectByName, ObReferenceObjectByHandle, ObCreateObject, ObInsertObject,
};
use ntkrnl_sys::{opaque_ptr, opaque_ptr_const};

/// STATUS_MORE_ENTRIES — not in ntkrnl_sys::constants, define locally.
const STATUS_MORE_ENTRIES: NTSTATUS = 0x00000105;

#[no_mangle]
pub static mut ObpDirectoryObjectType: POBJECT_TYPE = core::ptr::null_mut();

#[no_mangle]
pub unsafe extern "system" fn ObpInsertEntryDirectory(
    Parent: POBJECT_DIRECTORY,
    Context: POBP_LOOKUP_CONTEXT,
    ObjectHeader: POBJECT_HEADER,
) -> BOOLEAN {
    let ObjectHeader = opaque_ptr(ObjectHeader);
    let mut AllocatedEntry: *mut POBJECT_DIRECTORY_ENTRY;
    let NewEntry: POBJECT_DIRECTORY_ENTRY;
    let HeaderNameInfo: POBJECT_HEADER_NAME_INFO;

    if (*ObjectHeader).NameInfoOffset == 0 {
        return 0u8;
    }

    if ((*Context).Object != core::ptr::null_mut())
        || ((*Context).DirectoryLocked == 0)
        || (Parent != (*Context).Directory)
    {
        return 0u8;
    }

    NewEntry = ExAllocatePoolWithTag(
        PagedPool,
        core::mem::size_of::<_OBJECT_DIRECTORY_ENTRY>(),
        OB_DIR_TAG,
    ) as POBJECT_DIRECTORY_ENTRY;
    if NewEntry.is_null() {
        return 0u8;
    }

    (*NewEntry).HashValue = (*Context).HashValue;

    HeaderNameInfo = OBJECT_HEADER_TO_NAME_INFO(ObjectHeader);

    AllocatedEntry = core::ptr::addr_of_mut!((*Parent).HashBuckets[(*Context).HashIndex as usize]);

    (*NewEntry).ChainLink = *AllocatedEntry;
    *AllocatedEntry = NewEntry;

    (*NewEntry).Object = core::ptr::addr_of_mut!((*ObjectHeader).Body) as PVOID;

    (*HeaderNameInfo).Directory = Parent as *mut _OBJECT_DIRECTORY;
    1u8
}

#[no_mangle]
pub unsafe extern "system" fn ObpGetShadowDirectory(
    Directory: POBJECT_DIRECTORY,
) -> POBJECT_DIRECTORY {
    let DeviceMap: PDEVICE_MAP;
    let mut GlobalDosDirectory: POBJECT_DIRECTORY = core::ptr::null_mut();

    KeAcquireGuardedMutex(core::ptr::addr_of_mut!(ObpDeviceMapLock) as PKGUARDED_MUTEX);

    DeviceMap = (*Directory).DeviceMap;
    if !DeviceMap.is_null() {
        GlobalDosDirectory = (*DeviceMap).GlobalDosDevicesDirectory;
    }

    KeReleaseGuardedMutex(core::ptr::addr_of_mut!(ObpDeviceMapLock) as PKGUARDED_MUTEX);

    GlobalDosDirectory
}

#[no_mangle]
pub unsafe extern "system" fn ObpLookupEntryDirectory(
    mut Directory: POBJECT_DIRECTORY,
    Name: PUNICODE_STRING,
    Attributes: ULONG,
    SearchShadow: BOOLEAN,
    Context: POBP_LOOKUP_CONTEXT,
) -> PVOID {
    let Name = opaque_ptr(Name);
    let mut CaseInsensitive: BOOLEAN = 0u8;
    let mut HeaderNameInfo: POBJECT_HEADER_NAME_INFO;
    let mut ObjectHeader: POBJECT_HEADER;
    let mut HashValue: ULONG;
    let mut HashIndex: ULONG;
    let mut TotalChars: LONG;
    let mut CurrentChar: WCHAR;
    let mut AllocatedEntry: *mut POBJECT_DIRECTORY_ENTRY;
    let LookupBucket: *mut POBJECT_DIRECTORY_ENTRY;
    let mut CurrentEntry: POBJECT_DIRECTORY_ENTRY;
    let mut FoundObject: PVOID = core::ptr::null_mut();
    let mut Buffer: PWSTR;
    let ShadowDirectory: POBJECT_DIRECTORY;
    let SearchShadow = if ObpLUIDDeviceMapsEnabled == 0 {
        0u8
    } else {
        SearchShadow
    };

    if Directory.is_null() || Name.is_null() {
        FoundObject = core::ptr::null_mut();
    } else {
        TotalChars = ((*Name).Length / core::mem::size_of::<WCHAR>() as u16) as LONG;
        Buffer = (*Name).Buffer;

        if (Attributes & OBJ_CASE_INSENSITIVE) != 0 {
            CaseInsensitive = 1u8;
        }

        if Buffer.is_null() || TotalChars == 0 {
            FoundObject = core::ptr::null_mut();
        } else {
            HashValue = 0;
            while TotalChars > 0 {
                CurrentChar = *Buffer;
                Buffer = Buffer.add(1);

                HashValue = HashValue.wrapping_add(HashValue << 1).wrapping_add(HashValue >> 1);

                if CurrentChar < b'a' as u16 {
                    HashValue = HashValue.wrapping_add(CurrentChar as ULONG);
                } else if CurrentChar > b'z' as u16 {
                    HashValue = HashValue.wrapping_add(RtlUpcaseUnicodeChar(CurrentChar) as ULONG);
                } else {
                    HashValue = HashValue.wrapping_add((CurrentChar - (b'a' as u16 - b'A' as u16)) as ULONG);
                }

                TotalChars -= 1;
            }

            HashIndex = HashValue % 37;

            (*Context).HashValue = HashValue;
            (*Context).HashIndex = HashIndex as USHORT;

            'DoItAgain: loop {
                AllocatedEntry = core::ptr::addr_of_mut!((*Directory).HashBuckets[HashIndex as usize]);
                let LookupBucket = AllocatedEntry;

                if (*Context).DirectoryLocked == 0 {
                    ObpAcquireDirectoryLockShared(Directory, Context);
                }

                loop {
                    CurrentEntry = *AllocatedEntry;
                    if CurrentEntry.is_null() {
                        break;
                    }

                    if (*CurrentEntry).HashValue == HashValue {
                        ObjectHeader = OBJECT_TO_OBJECT_HEADER((*CurrentEntry).Object);

                        if (*ObjectHeader).NameInfoOffset != 0 {
                            HeaderNameInfo = OBJECT_HEADER_TO_NAME_INFO(ObjectHeader);

                            if ((*Name).Length == (*HeaderNameInfo).Name.Length)
                                && (RtlEqualUnicodeString(Name, core::ptr::addr_of!((*HeaderNameInfo).Name), CaseInsensitive) != 0)
                            {
                                break;
                            }
                        }
                    }

                    AllocatedEntry = core::ptr::addr_of_mut!((*CurrentEntry).ChainLink);
                }

                if !CurrentEntry.is_null() {
                    if AllocatedEntry != LookupBucket {
                        if ((*Context).DirectoryLocked != 0)
                            || (ExConvertPushLockSharedToExclusive(core::ptr::addr_of_mut!((*Directory).Lock)) != 0)
                        {
                            *AllocatedEntry = (*CurrentEntry).ChainLink;
                            (*CurrentEntry).ChainLink = *LookupBucket;
                            *LookupBucket = CurrentEntry;
                        }
                    }

                    FoundObject = (*CurrentEntry).Object;
                    break 'DoItAgain;
                } else {
                    if (*Context).DirectoryLocked == 0 {
                        ObpReleaseDirectoryLock(Directory, Context);
                    }

                    if (SearchShadow != 0) && !(*Directory).DeviceMap.is_null() {
                        let ShadowDirectory = ObpGetShadowDirectory(Directory);
                        if !ShadowDirectory.is_null() {
                            Directory = ShadowDirectory;
                            continue 'DoItAgain;
                        }
                    }

                    break 'DoItAgain;
                }
            }
        }
    }

    if !FoundObject.is_null() {
        ObjectHeader = OBJECT_TO_OBJECT_HEADER(FoundObject);
        ObpReferenceNameInfo(ObjectHeader);

        ObReferenceObject(FoundObject);

        if (*Context).DirectoryLocked == 0 {
            ObpReleaseDirectoryLock(Directory, Context);
        }
    }

    ObpReleaseLookupContextObject(Context);
    (*Context).Object = FoundObject;

    FoundObject
}

#[no_mangle]
pub unsafe extern "system" fn ObpDeleteEntryDirectory(Context: POBP_LOOKUP_CONTEXT) -> BOOLEAN {
    let Directory: POBJECT_DIRECTORY;
    let AllocatedEntry: *mut POBJECT_DIRECTORY_ENTRY;
    let CurrentEntry: POBJECT_DIRECTORY_ENTRY;

    Directory = (*Context).Directory;
    if Directory.is_null() {
        return 0u8;
    }

    AllocatedEntry = core::ptr::addr_of_mut!((*Directory).HashBuckets[(*Context).HashIndex as usize]);
    CurrentEntry = *AllocatedEntry;

    *AllocatedEntry = (*CurrentEntry).ChainLink;
    (*CurrentEntry).ChainLink = core::ptr::null_mut();

    ExFreePoolWithTag(CurrentEntry as PVOID, OB_DIR_TAG);

    1u8
}

#[no_mangle]
pub unsafe extern "system" fn NtOpenDirectoryObject(
    DirectoryHandle: PHANDLE,
    DesiredAccess: ACCESS_MASK,
    ObjectAttributes: POBJECT_ATTRIBUTES,
) -> NTSTATUS {
    let DirectoryHandle = opaque_ptr(DirectoryHandle);
    let ObjectAttributes = opaque_ptr(ObjectAttributes);
    let mut Directory: HANDLE = core::ptr::null_mut();
    let PreviousMode: KPROCESSOR_MODE = ExGetPreviousMode();
    let mut Status: NTSTATUS;

    if PreviousMode != 0 {
        if let Err(s) = probe_user_buffer(DirectoryHandle as _, core::mem::size_of::<HANDLE>(), core::mem::align_of::<HANDLE>(), PreviousMode) {
            return s;
        }
    }

    Status = ObOpenObjectByName(
        ObjectAttributes,
        ObpDirectoryObjectType,
        PreviousMode,
        core::ptr::null_mut(),
        DesiredAccess,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(Directory),
    );
    if Status >= 0 {
        // C: _SEH2_TRY { *DirectoryHandle = Directory; } _SEH2_EXCEPT { Status = code; }
        if let Err(s) = write_back(DirectoryHandle, Directory) {
            Status = s;
        }
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn NtQueryDirectoryObject(
    DirectoryHandle: HANDLE,
    Buffer: PVOID,
    BufferLength: ULONG,
    ReturnSingleEntry: BOOLEAN,
    RestartScan: BOOLEAN,
    Context: *mut ULONG,
    ReturnLength: *mut ULONG,
) -> NTSTATUS {
    let DirectoryHandle = opaque_ptr(DirectoryHandle);
    let Buffer = opaque_ptr(Buffer);
    let Context = opaque_ptr(Context);
    let ReturnLength = opaque_ptr(ReturnLength);
    let mut Directory: POBJECT_DIRECTORY = core::ptr::null_mut();
    let PreviousMode: KPROCESSOR_MODE = ExGetPreviousMode();
    let mut SkipEntries: ULONG = 0;
    let mut Status: NTSTATUS;
    let LocalBuffer: PVOID;
    let mut DirectoryInfo: POBJECT_DIRECTORY_INFORMATION;
    let mut Length: ULONG;
    let mut TotalLength: ULONG;
    let mut Count: ULONG;
    let mut CurrentEntry: ULONG;
    let mut Hash: ULONG;
    let mut Entry: POBJECT_DIRECTORY_ENTRY;
    let mut ObjectHeader: POBJECT_HEADER;
    let mut ObjectNameInfo: POBJECT_HEADER_NAME_INFO;
    let mut Name: UNICODE_STRING;
    let mut p: PWSTR;
    let mut LookupContext: _OBP_LOOKUP_CONTEXT = core::mem::zeroed();

    ObpInitializeLookupContext(core::ptr::addr_of_mut!(LookupContext));

    if PreviousMode != 0 {
        if let Err(s) = probe_user_buffer(Buffer, BufferLength as usize, core::mem::size_of::<WCHAR>(), PreviousMode) {
            return s;
        }
        if let Err(s) = probe_user_buffer(Context as _, core::mem::size_of::<ULONG>(), core::mem::align_of::<ULONG>(), PreviousMode) {
            return s;
        }
        if RestartScan == 0 {
            SkipEntries = *Context;
        }
        if !ReturnLength.is_null() {
            if let Err(s) = probe_user_buffer(ReturnLength as _, core::mem::size_of::<ULONG>(), core::mem::align_of::<ULONG>(), PreviousMode) {
                return s;
            }
        }
    } else if RestartScan == 0 {
        SkipEntries = *Context;
    }

    LocalBuffer = ExAllocatePoolWithTag(
        PagedPool,
        core::mem::size_of::<_OBJECT_DIRECTORY_INFORMATION>() + BufferLength as usize,
        OB_NAME_TAG,
    );
    if LocalBuffer.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }
    core::ptr::write_bytes(LocalBuffer as *mut u8, 0, BufferLength as usize);

    Status = ObReferenceObjectByHandle(
        DirectoryHandle,
        DIRECTORY_QUERY,
        ObpDirectoryObjectType,
        PreviousMode,
        core::ptr::addr_of_mut!(Directory) as *mut PVOID,
        core::ptr::null_mut(),
    );
    if Status < 0 {
        ExFreePoolWithTag(LocalBuffer, OB_NAME_TAG);
        return Status;
    }

    ObpAcquireDirectoryLockShared(Directory, core::ptr::addr_of_mut!(LookupContext));

    DirectoryInfo = LocalBuffer as POBJECT_DIRECTORY_INFORMATION;
    TotalLength = core::mem::size_of::<_OBJECT_DIRECTORY_INFORMATION>() as ULONG;

    Count = 0;
    CurrentEntry = 0;

    Status = STATUS_NO_MORE_ENTRIES;
    Hash = 0;
    'outer: while Hash < 37 {
        Entry = (*Directory).HashBuckets[Hash as usize];
        while !Entry.is_null() {
            if SkipEntries == CurrentEntry {
                CurrentEntry += 1;

                ObjectHeader = OBJECT_TO_OBJECT_HEADER((*Entry).Object);
                ObjectNameInfo = OBJECT_HEADER_TO_NAME_INFO(ObjectHeader);

                if !ObjectNameInfo.is_null() {
                    Name = UNICODE_STRING {
                        Length: (*ObjectNameInfo).Name.Length,
                        MaximumLength: (*ObjectNameInfo).Name.MaximumLength,
                        Buffer: (*ObjectNameInfo).Name.Buffer,
                    };
                } else {
                    Name = UNICODE_STRING {
                        Length: 0,
                        MaximumLength: 0,
                        Buffer: core::ptr::null_mut(),
                    };
                }

                Length = core::mem::size_of::<_OBJECT_DIRECTORY_INFORMATION>() as ULONG
                    + Name.Length as ULONG
                    + core::mem::size_of::<WCHAR>() as ULONG
                    + (*(*ObjectHeader).Type).Name.Length as ULONG
                    + core::mem::size_of::<WCHAR>() as ULONG;

                if (TotalLength + Length) > BufferLength {
                    if ReturnSingleEntry != 0 {
                        TotalLength += Length;
                        Status = STATUS_BUFFER_TOO_SMALL;
                    } else {
                        Status = STATUS_MORE_ENTRIES;
                    }

                    CurrentEntry -= 1;
                    break 'outer;
                }

                (*DirectoryInfo).Name.Length = Name.Length;
                (*DirectoryInfo).Name.MaximumLength = Name.Length + core::mem::size_of::<WCHAR>() as u16;
                (*DirectoryInfo).Name.Buffer = Name.Buffer;
                (*DirectoryInfo).TypeName.Length = (*(*ObjectHeader).Type).Name.Length;
                (*DirectoryInfo).TypeName.MaximumLength = (*(*ObjectHeader).Type).Name.Length + core::mem::size_of::<WCHAR>() as u16;
                (*DirectoryInfo).TypeName.Buffer = (*(*ObjectHeader).Type).Name.Buffer;

                Status = STATUS_SUCCESS;

                TotalLength += Length;
                DirectoryInfo = DirectoryInfo.add(1);
                Count += 1;

                if ReturnSingleEntry != 0 {
                    break 'outer;
                }

                SkipEntries += 1;
            } else {
                CurrentEntry += 1;
            }

            Entry = (*Entry).ChainLink;
        }
        Hash += 1;
    }

    if Status >= 0 {
        core::ptr::write_bytes(DirectoryInfo as *mut u8, 0, core::mem::size_of::<_OBJECT_DIRECTORY_INFORMATION>());
        DirectoryInfo = DirectoryInfo.add(1);

        p = DirectoryInfo as PWSTR;
        DirectoryInfo = LocalBuffer as POBJECT_DIRECTORY_INFORMATION;
        let mut i = Count;
        while i > 0 {
            core::ptr::copy_nonoverlapping(
                (*DirectoryInfo).Name.Buffer as *const u8,
                p as *mut u8,
                (*DirectoryInfo).Name.Length as usize,
            );

            (*DirectoryInfo).Name.Buffer = ((Buffer as ULONG_PTR) + ((p as ULONG_PTR) - (LocalBuffer as ULONG_PTR))) as PWSTR;

            p = ((p as ULONG_PTR) + (*DirectoryInfo).Name.Length as ULONG_PTR) as PWSTR;
            *p = 0;
            p = p.add(1);

            core::ptr::copy_nonoverlapping(
                (*DirectoryInfo).TypeName.Buffer as *const u8,
                p as *mut u8,
                (*DirectoryInfo).TypeName.Length as usize,
            );

            (*DirectoryInfo).TypeName.Buffer = ((Buffer as ULONG_PTR) + ((p as ULONG_PTR) - (LocalBuffer as ULONG_PTR))) as PWSTR;

            p = ((p as ULONG_PTR) + (*DirectoryInfo).TypeName.Length as ULONG_PTR) as PWSTR;
            *p = 0;
            p = p.add(1);

            DirectoryInfo = DirectoryInfo.add(1);
            i -= 1;
        }

        // The C stores this outside its _SEH2_TRY; Context is a probed user
        // pointer all the same.
        if let Err(s) = write_back(Context, CurrentEntry) {
            Status = s;
        }
    }

    // C: _SEH2_TRY { copy the buffer; write ReturnLength } _SEH2_EXCEPT { Status = code; }
    let copied = safe_copy_raw(
        Buffer,
        LocalBuffer as *const core::ffi::c_void,
        if TotalLength <= BufferLength { TotalLength } else { BufferLength } as usize,
    )
    .and_then(|()| if ReturnLength.is_null() { Ok(()) } else { write_back(ReturnLength, TotalLength) });
    if let Err(s) = copied {
        Status = s;
    }

    ObpReleaseDirectoryLock(Directory, core::ptr::addr_of_mut!(LookupContext));

    ObDereferenceObject(Directory as PVOID);
    ExFreePoolWithTag(LocalBuffer, OB_NAME_TAG);

    Status
}

#[no_mangle]
pub unsafe extern "system" fn NtCreateDirectoryObject(
    DirectoryHandle: PHANDLE,
    DesiredAccess: ACCESS_MASK,
    ObjectAttributes: POBJECT_ATTRIBUTES,
) -> NTSTATUS {
    let DirectoryHandle = opaque_ptr(DirectoryHandle);
    let ObjectAttributes = opaque_ptr(ObjectAttributes);
    let mut Directory: POBJECT_DIRECTORY = core::ptr::null_mut();
    let mut NewHandle: HANDLE = core::ptr::null_mut();
    let PreviousMode: KPROCESSOR_MODE = ExGetPreviousMode();
    let mut Status: NTSTATUS;

    if PreviousMode != 0 {
        if let Err(s) = probe_user_buffer(DirectoryHandle as _, core::mem::size_of::<HANDLE>(), core::mem::align_of::<HANDLE>(), PreviousMode) {
            return s;
        }
    }

    Status = ObCreateObject(
        PreviousMode,
        ObpDirectoryObjectType,
        ObjectAttributes,
        PreviousMode,
        core::ptr::null_mut(),
        core::mem::size_of::<_OBJECT_DIRECTORY>() as ULONG,
        0,
        0,
        core::ptr::addr_of_mut!(Directory) as *mut PVOID,
    );
    if Status < 0 {
        return Status;
    }

    core::ptr::write_bytes(Directory as *mut u8, 0, core::mem::size_of::<_OBJECT_DIRECTORY>());
    ExInitializePushLock(core::ptr::addr_of_mut!((*Directory).Lock) as ntkrnl_sys::ddk::PULONG_PTR);
    (*Directory).SessionId = -1i32 as ULONG;

    Status = ObInsertObject(
        Directory as PVOID,
        core::ptr::null_mut(),
        DesiredAccess,
        0,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(NewHandle),
    );

    // C: _SEH2_TRY { *DirectoryHandle = NewHandle; } _SEH2_EXCEPT { Status = code; }
    if let Err(s) = write_back(DirectoryHandle, NewHandle) {
        Status = s;
    }

    Status
}

// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         ReactOS Kernel
//  * LICENSE:         GPL - See COPYING in the top level directory
//  * FILE:            ntoskrnl/ob/obdir.c
//  * PURPOSE:         Manages the Object Manager's Directory Implementation,
//  *                  such as functions for addition, deletion and lookup into
//  *                  the Object Manager's namespace. These routines are separate
//  *                  from the Namespace Implementation because they are largely
//  *                  independent and could be used for other namespaces.
//  * PROGRAMMERS:     Alex Ionescu (alex.ionescu@reactos.org)
//  *                  Thomas Weidenmueller (w3seek@reactos.org)
//  */
//
// /* INCLUDES ***************************************************************/
//
// #include <ntoskrnl.h>
// #define NDEBUG
// #include <debug.h>
//
// POBJECT_TYPE ObpDirectoryObjectType = NULL;
//
// /* PRIVATE FUNCTIONS ******************************************************/
//
// /*++
// * @name ObpInsertEntryDirectory
// *
// *     The ObpInsertEntryDirectory routine <FILLMEIN>.
// *
// * @param Parent
// *        <FILLMEIN>.
// *
// * @param Context
// *        <FILLMEIN>.
// *
// * @param ObjectHeader
// *        <FILLMEIN>.
// *
// * @return TRUE if the object was inserted, FALSE otherwise.
// *
// * @remarks None.
// *
// *--*/
// BOOLEAN
// NTAPI
// ObpInsertEntryDirectory(IN POBJECT_DIRECTORY Parent,
//                         IN POBP_LOOKUP_CONTEXT Context,
//                         IN POBJECT_HEADER ObjectHeader)
// {
//     POBJECT_DIRECTORY_ENTRY *AllocatedEntry;
//     POBJECT_DIRECTORY_ENTRY NewEntry;
//     POBJECT_HEADER_NAME_INFO HeaderNameInfo;
//
//     /* Make sure we have a name */
//     ASSERT(ObjectHeader->NameInfoOffset != 0);
//
//     /* Validate the context */
//     if ((Context->Object) ||
//         !(Context->DirectoryLocked) ||
//         (Parent != Context->Directory))
//     {
//         /* Invalid context */
//         DPRINT1("OB: ObpInsertEntryDirectory - invalid context %p %u\n",
//                 Context, Context->DirectoryLocked);
//         ASSERT(FALSE);
//         return FALSE;
//     }
//
//     /* Allocate a new Directory Entry */
//     NewEntry = ExAllocatePoolWithTag(PagedPool,
//                                      sizeof(OBJECT_DIRECTORY_ENTRY),
//                                      OB_DIR_TAG);
//     if (!NewEntry) return FALSE;
//
//     /* Save the hash */
//     NewEntry->HashValue = Context->HashValue;
//
//     /* Get the Object Name Information */
//     HeaderNameInfo = OBJECT_HEADER_TO_NAME_INFO(ObjectHeader);
//
//     /* Get the Allocated entry */
//     AllocatedEntry = &Parent->HashBuckets[Context->HashIndex];
//
//     /* Set it */
//     NewEntry->ChainLink = *AllocatedEntry;
//     *AllocatedEntry = NewEntry;
//
//     /* Associate the Object */
//     NewEntry->Object = &ObjectHeader->Body;
//
//     /* Associate the Directory */
//     HeaderNameInfo->Directory = Parent;
//     return TRUE;
// }
//
// /*++
// * @name ObpGetShadowDirectory
// *
// *     The ObpGetShadowDirectory routine <FILLMEIN>.
// *
// * @param Directory
// *        <FILLMEIN>.
// *
// * @return Pointer to the global DOS directory if any, or NULL otherwise.
// *
// * @remarks None.
// *
// *--*/
// POBJECT_DIRECTORY
// NTAPI
// ObpGetShadowDirectory(IN POBJECT_DIRECTORY Directory)
// {
//     PDEVICE_MAP DeviceMap;
//     POBJECT_DIRECTORY GlobalDosDirectory = NULL;
//
//     /* Acquire the device map lock */
//     KeAcquireGuardedMutex(&ObpDeviceMapLock);
//
//     /* Get the global DOS directory if any */
//     DeviceMap = Directory->DeviceMap;
//     if (DeviceMap != NULL)
//     {
//         GlobalDosDirectory = DeviceMap->GlobalDosDevicesDirectory;
//     }
//
//     /* Release the devicemap lock */
//     KeReleaseGuardedMutex(&ObpDeviceMapLock);
//
//     return GlobalDosDirectory;
// }
//
// /*++
// * @name ObpLookupEntryDirectory
// *
// *     The ObpLookupEntryDirectory routine <FILLMEIN>.
// *
// * @param Directory
// *        <FILLMEIN>.
// *
// * @param Name
// *        <FILLMEIN>.
// *
// * @param Attributes
// *        <FILLMEIN>.
// *
// * @param SearchShadow
// *        <FILLMEIN>.
// *
// * @param Context
// *        <FILLMEIN>.
// *
// * @return Pointer to the object which was found, or NULL otherwise.
// *
// * @remarks None.
// *
// *--*/
// PVOID
// NTAPI
// ObpLookupEntryDirectory(IN POBJECT_DIRECTORY Directory,
//                         IN PUNICODE_STRING Name,
//                         IN ULONG Attributes,
//                         IN BOOLEAN SearchShadow,
//                         IN POBP_LOOKUP_CONTEXT Context)
// {
//     BOOLEAN CaseInsensitive = FALSE;
//     POBJECT_HEADER_NAME_INFO HeaderNameInfo;
//     POBJECT_HEADER ObjectHeader;
//     ULONG HashValue;
//     ULONG HashIndex;
//     LONG TotalChars;
//     WCHAR CurrentChar;
//     POBJECT_DIRECTORY_ENTRY *AllocatedEntry;
//     POBJECT_DIRECTORY_ENTRY *LookupBucket;
//     POBJECT_DIRECTORY_ENTRY CurrentEntry;
//     PVOID FoundObject = NULL;
//     PWSTR Buffer;
//     POBJECT_DIRECTORY ShadowDirectory;
//
//     PAGED_CODE();
//
//     /* Check if we should search the shadow directory */
//     if (ObpLUIDDeviceMapsEnabled == 0) SearchShadow = FALSE;
//
//     /* Fail if we don't have a directory or name */
//     if (!(Directory) || !(Name)) goto Quickie;
//
//     /* Get name information */
//     TotalChars = Name->Length / sizeof(WCHAR);
//     Buffer = Name->Buffer;
//
//     /* Set up case-sensitivity */
//     if (Attributes & OBJ_CASE_INSENSITIVE) CaseInsensitive = TRUE;
//
//     /* Fail if the name is empty */
//     if (!(Buffer) || !(TotalChars)) goto Quickie;
//
//     /* Create the Hash */
//     for (HashValue = 0; TotalChars; TotalChars--)
//     {
//         /* Go to the next Character */
//         CurrentChar = *Buffer++;
//
//         /* Prepare the Hash */
//         HashValue += (HashValue << 1) + (HashValue >> 1);
//
//         /* Create the rest based on the name */
//         if (CurrentChar < 'a') HashValue += CurrentChar;
//         else if (CurrentChar > 'z') HashValue += RtlUpcaseUnicodeChar(CurrentChar);
//         else HashValue += (CurrentChar - ('a'-'A'));
//     }
//
//     /* Merge it with our number of hash buckets */
//     HashIndex = HashValue % 37;
//
//     /* Save the result */
//     Context->HashValue = HashValue;
//     Context->HashIndex = (USHORT)HashIndex;
//
// DoItAgain:
//     /* Get the root entry and set it as our lookup bucket */
//     AllocatedEntry = &Directory->HashBuckets[HashIndex];
//     LookupBucket = AllocatedEntry;
//
//     /* Check if the directory is already locked */
//     if (!Context->DirectoryLocked)
//     {
//         /* Lock it */
//         ObpAcquireDirectoryLockShared(Directory, Context);
//     }
//
//     /* Start looping */
//     while ((CurrentEntry = *AllocatedEntry))
//     {
//         /* Do the hashes match? */
//         if (CurrentEntry->HashValue == HashValue)
//         {
//             /* Make sure that it has a name */
//             ObjectHeader = OBJECT_TO_OBJECT_HEADER(CurrentEntry->Object);
//
//             /* Get the name information */
//             ASSERT(ObjectHeader->NameInfoOffset != 0);
//             HeaderNameInfo = OBJECT_HEADER_TO_NAME_INFO(ObjectHeader);
//
//             /* Do the names match? */
//             if ((Name->Length == HeaderNameInfo->Name.Length) &&
//                 (RtlEqualUnicodeString(Name, &HeaderNameInfo->Name, CaseInsensitive)))
//             {
//                 break;
//             }
//         }
//
//         /* Move to the next entry */
//         AllocatedEntry = &CurrentEntry->ChainLink;
//     }
//
//     /* Check if we still have an entry */
//     if (CurrentEntry)
//     {
//         /* Set this entry as the first, to speed up incoming insertion */
//         if (AllocatedEntry != LookupBucket)
//         {
//             /* Check if the directory was locked or convert the lock */
//             if ((Context->DirectoryLocked) ||
//                 (ExConvertPushLockSharedToExclusive(&Directory->Lock)))
//             {
//                 /* Set the Current Entry */
//                 *AllocatedEntry = CurrentEntry->ChainLink;
//
//                 /* Link to the old Hash Entry */
//                 CurrentEntry->ChainLink = *LookupBucket;
//
//                 /* Set the new Hash Entry */
//                 *LookupBucket = CurrentEntry;
//             }
//         }
//
//         /* Save the found object */
//         FoundObject = CurrentEntry->Object;
//         goto Quickie;
//     }
//     else
//     {
//         /* Check if the directory was locked */
//         if (!Context->DirectoryLocked)
//         {
//             /* Release the lock */
//             ObpReleaseDirectoryLock(Directory, Context);
//         }
//
//         /* Check if we should scan the shadow directory */
//         if ((SearchShadow) && (Directory->DeviceMap))
//         {
//             ShadowDirectory = ObpGetShadowDirectory(Directory);
//             /* A global DOS directory was found, loop it again */
//             if (ShadowDirectory != NULL)
//             {
//                 Directory = ShadowDirectory;
//                 goto DoItAgain;
//             }
//         }
//     }
//
// Quickie:
//     /* Check if we inserted an object */
//     if (FoundObject)
//     {
//         /* Get the object name information */
//         ObjectHeader = OBJECT_TO_OBJECT_HEADER(FoundObject);
//         ObpReferenceNameInfo(ObjectHeader);
//
//         /* Reference the object being looked up */
//         ObReferenceObject(FoundObject);
//
//         /* Check if the directory was locked */
//         if (!Context->DirectoryLocked)
//         {
//             /* Release the lock */
//             ObpReleaseDirectoryLock(Directory, Context);
//         }
//     }
//
//     /* Release any object previously looked up and replace it with the new one */
//     ObpReleaseLookupContextObject(Context);
//     Context->Object = FoundObject;
//
//     /* Return the object we found */
//     return FoundObject;
// }
//
// /*++
// * @name ObpDeleteEntryDirectory
// *
// *     The ObpDeleteEntryDirectory routine <FILLMEIN>.
// *
// * @param Context
// *        <FILLMEIN>.
// *
// * @return TRUE if the object was deleted, FALSE otherwise.
// *
// * @remarks None.
// *
// *--*/
// BOOLEAN
// NTAPI
// ObpDeleteEntryDirectory(POBP_LOOKUP_CONTEXT Context)
// {
//     POBJECT_DIRECTORY Directory;
//     POBJECT_DIRECTORY_ENTRY *AllocatedEntry;
//     POBJECT_DIRECTORY_ENTRY CurrentEntry;
//
//     /* Get the Directory */
//     Directory = Context->Directory;
//     if (!Directory) return FALSE;
//
//     /* Get the Entry */
//     AllocatedEntry = &Directory->HashBuckets[Context->HashIndex];
//     CurrentEntry = *AllocatedEntry;
//
//     /* Unlink the Entry */
//     *AllocatedEntry = CurrentEntry->ChainLink;
//     CurrentEntry->ChainLink = NULL;
//
//     /* Free it */
//     ExFreePoolWithTag(CurrentEntry, OB_DIR_TAG);
//
//     /* Return */
//     return TRUE;
// }
//
// /* FUNCTIONS **************************************************************/
//
// /*++
// * @name NtOpenDirectoryObject
// * @implemented NT4
// *
// *     The NtOpenDirectoryObject routine opens a namespace directory object.
// *
// * @param DirectoryHandle
// *        Variable which receives the directory handle.
// *
// * @param DesiredAccess
// *        Desired access to the directory.
// *
// * @param ObjectAttributes
// *        Structure describing the directory.
// *
// * @return STATUS_SUCCESS or appropriate error value.
// *
// * @remarks None.
// *
// *--*/
// NTSTATUS
// NTAPI
// NtOpenDirectoryObject(OUT PHANDLE DirectoryHandle,
//                       IN ACCESS_MASK DesiredAccess,
//                       IN POBJECT_ATTRIBUTES ObjectAttributes)
// {
//     HANDLE Directory;
//     KPROCESSOR_MODE PreviousMode = ExGetPreviousMode();
//     NTSTATUS Status;
//     PAGED_CODE();
//
//     /* Check if we need to do any probing */
//     if (PreviousMode != KernelMode)
//     {
//         _SEH2_TRY
//         {
//             /* Probe the return handle */
//             ProbeForWriteHandle(DirectoryHandle);
//         }
//         _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//         {
//             /* Return the exception code */
//             _SEH2_YIELD(return _SEH2_GetExceptionCode());
//         }
//         _SEH2_END;
//     }
//
//     /* Open the directory object */
//     Status = ObOpenObjectByName(ObjectAttributes,
//                                 ObpDirectoryObjectType,
//                                 PreviousMode,
//                                 NULL,
//                                 DesiredAccess,
//                                 NULL,
//                                 &Directory);
//     if (NT_SUCCESS(Status))
//     {
//         _SEH2_TRY
//         {
//             /* Write back the handle to the caller */
//             *DirectoryHandle = Directory;
//         }
//         _SEH2_EXCEPT(ExSystemExceptionFilter())
//         {
//             /* Get the exception code */
//             Status = _SEH2_GetExceptionCode();
//         }
//         _SEH2_END;
//     }
//
//     /* Return the status to the caller */
//     return Status;
// }
//
// /*++
// * @name NtQueryDirectoryObject
// * @implemented NT4
// *
// *     The NtQueryDirectoryObject routine reads information from a directory in
// *     the system namespace.
// *
// * @param DirectoryHandle
// *        Handle obtained with NtOpenDirectoryObject which
// *        must grant DIRECTORY_QUERY access to the directory object.
// *
// * @param Buffer
// *        Buffer to hold the data read.
// *
// * @param BufferLength
// *        Size of the buffer in bytes.
// *
// * @param ReturnSingleEntry
// *        When TRUE, only 1 entry is written in DirObjInformation;
// *        otherwise as many as will fit in the buffer.
// *
// * @param RestartScan
// *        If TRUE start reading at index 0.
// *        If FALSE start reading at the index specified by *ObjectIndex.
// *
// * @param Context
// *        Zero based index into the directory, interpretation
// *        depends on RestartScan.
// *
// * @param ReturnLength
// *        Caller supplied storage for the number of bytes
// *        written (or NULL).
// *
// * @return STATUS_SUCCESS or appropriate error value.
// *
// * @remarks Although you can iterate over the directory by calling this
// *          function multiple times, the directory is unlocked between
// *          calls. This means that another thread can change the directory
// *          and so iterating doesn't guarantee a consistent picture of the
// *          directory. Best thing is to retrieve all directory entries in
// *          one call.
// *
// *--*/
// NTSTATUS
// NTAPI
// NtQueryDirectoryObject(IN HANDLE DirectoryHandle,
//                        OUT PVOID Buffer,
//                        IN ULONG BufferLength,
//                        IN BOOLEAN ReturnSingleEntry,
//                        IN BOOLEAN RestartScan,
//                        IN OUT PULONG Context,
//                        OUT PULONG ReturnLength OPTIONAL)
// {
//     POBJECT_DIRECTORY Directory;
//     KPROCESSOR_MODE PreviousMode = ExGetPreviousMode();
//     ULONG SkipEntries = 0;
//     NTSTATUS Status;
//     PVOID LocalBuffer;
//     POBJECT_DIRECTORY_INFORMATION DirectoryInfo;
//     ULONG Length, TotalLength;
//     ULONG Count, CurrentEntry;
//     ULONG Hash;
//     POBJECT_DIRECTORY_ENTRY Entry;
//     POBJECT_HEADER ObjectHeader;
//     POBJECT_HEADER_NAME_INFO ObjectNameInfo;
//     UNICODE_STRING Name;
//     PWSTR p;
//     OBP_LOOKUP_CONTEXT LookupContext;
//     PAGED_CODE();
//
//     /* Initialize lookup */
//     ObpInitializeLookupContext(&LookupContext);
//
//     /* Check if we need to do any probing */
//     if (PreviousMode != KernelMode)
//     {
//         _SEH2_TRY
//         {
//             /* Probe the buffer (assuming it will hold Unicode characters) */
//             ProbeForWrite(Buffer, BufferLength, sizeof(WCHAR));
//
//             /* Probe the context and copy it unless scan-restart was requested */
//             ProbeForWriteUlong(Context);
//             if (!RestartScan) SkipEntries = *Context;
//
//             /* Probe the return length if the caller specified one */
//             if (ReturnLength) ProbeForWriteUlong(ReturnLength);
//         }
//         _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//         {
//             /* Return the exception code */
//             _SEH2_YIELD(return _SEH2_GetExceptionCode());
//         }
//         _SEH2_END;
//     }
//     else if (!RestartScan)
//     {
//         /* This is kernel mode, save the context without probing, if needed */
//         SkipEntries = *Context;
//     }
//
//     /* Allocate a buffer */
//     LocalBuffer = ExAllocatePoolWithTag(PagedPool,
//                                         sizeof(OBJECT_DIRECTORY_INFORMATION) +
//                                         BufferLength,
//                                         OB_NAME_TAG);
//     if (!LocalBuffer) return STATUS_INSUFFICIENT_RESOURCES;
//     RtlZeroMemory(LocalBuffer, BufferLength);
//
//     /* Get a reference to directory */
//     Status = ObReferenceObjectByHandle(DirectoryHandle,
//                                        DIRECTORY_QUERY,
//                                        ObpDirectoryObjectType,
//                                        PreviousMode,
//                                        (PVOID*)&Directory,
//                                        NULL);
//     if (!NT_SUCCESS(Status))
//     {
//         /* Free the buffer and fail */
//         ExFreePoolWithTag(LocalBuffer, OB_NAME_TAG);
//         return Status;
//     }
//
//     /* Lock the directory in shared mode */
//     ObpAcquireDirectoryLockShared(Directory, &LookupContext);
//
//     /* Start at position 0 */
//     DirectoryInfo = (POBJECT_DIRECTORY_INFORMATION)LocalBuffer;
//     TotalLength = sizeof(OBJECT_DIRECTORY_INFORMATION);
//
//     /* Start with 0 entries */
//     Count = 0;
//     CurrentEntry = 0;
//
//     /* Set default status and start looping */
//     Status = STATUS_NO_MORE_ENTRIES;
//     for (Hash = 0; Hash < 37; Hash++)
//     {
//         /* Get this entry and loop all of them */
//         Entry = Directory->HashBuckets[Hash];
//         while (Entry)
//         {
//             /* Check if we should process this entry */
//             if (SkipEntries == CurrentEntry++)
//             {
//                 /* Get the header data */
//                 ObjectHeader = OBJECT_TO_OBJECT_HEADER(Entry->Object);
//                 ObjectNameInfo = OBJECT_HEADER_TO_NAME_INFO(ObjectHeader);
//
//                 /* Get the object name */
//                 if (ObjectNameInfo)
//                 {
//                     /* Use the one we have */
//                     Name = ObjectNameInfo->Name;
//                 }
//                 else
//                 {
//                     /* Otherwise, use an empty one */
//                     RtlInitEmptyUnicodeString(&Name, NULL, 0);
//                 }
//
//                 /* Calculate the length for this entry */
//                 Length = sizeof(OBJECT_DIRECTORY_INFORMATION) +
//                          Name.Length + sizeof(UNICODE_NULL) +
//                          ObjectHeader->Type->Name.Length + sizeof(UNICODE_NULL);
//
//                 /* Make sure this entry won't overflow */
//                 if ((TotalLength + Length) > BufferLength)
//                 {
//                     /* Check if the caller wanted only an entry */
//                     if (ReturnSingleEntry)
//                     {
//                         /* Then we'll fail and ask for more buffer */
//                         TotalLength += Length;
//                         Status = STATUS_BUFFER_TOO_SMALL;
//                     }
//                     else
//                     {
//                         /* Otherwise, we'll say we're done for now */
//                         Status = STATUS_MORE_ENTRIES;
//                     }
//
//                     /* Decrease the entry since we didn't process */
//                     CurrentEntry--;
//                     goto Quickie;
//                 }
//
//                 /* Now fill in the buffer */
//                 DirectoryInfo->Name.Length = Name.Length;
//                 DirectoryInfo->Name.MaximumLength = Name.Length +
//                                                     sizeof(UNICODE_NULL);
//                 DirectoryInfo->Name.Buffer = Name.Buffer;
//                 DirectoryInfo->TypeName.Length = ObjectHeader->
//                                                  Type->Name.Length;
//                 DirectoryInfo->TypeName.MaximumLength = ObjectHeader->
//                                                         Type->Name.Length +
//                                                         sizeof(UNICODE_NULL);
//                 DirectoryInfo->TypeName.Buffer = ObjectHeader->
//                                                  Type->Name.Buffer;
//
//                 /* Set success */
//                 Status = STATUS_SUCCESS;
//
//                 /* Increase statistics */
//                 TotalLength += Length;
//                 DirectoryInfo++;
//                 Count++;
//
//                 /* If the caller only wanted an entry, bail out */
//                 if (ReturnSingleEntry) goto Quickie;
//
//                 /* Increase the key by one */
//                 SkipEntries++;
//             }
//
//             /* Move to the next directory */
//             Entry = Entry->ChainLink;
//         }
//     }
//
// Quickie:
//     /* Make sure we got success */
//     if (NT_SUCCESS(Status))
//     {
//         /* Clear the current pointer and set it */
//         RtlZeroMemory(DirectoryInfo, sizeof(OBJECT_DIRECTORY_INFORMATION));
//         DirectoryInfo++;
//
//         /* Set the buffer here now and loop entries */
//         p = (PWSTR)DirectoryInfo;
//         DirectoryInfo = LocalBuffer;
//         while (Count--)
//         {
//             /* Copy the name buffer */
//             RtlCopyMemory(p,
//                           DirectoryInfo->Name.Buffer,
//                           DirectoryInfo->Name.Length);
//
//             /* Now fixup the pointers */
//             DirectoryInfo->Name.Buffer = (PVOID)((ULONG_PTR)Buffer +
//                                                  ((ULONG_PTR)p -
//                                                   (ULONG_PTR)LocalBuffer));
//
//             /* Advance in buffer and NULL-terminate */
//             p = (PVOID)((ULONG_PTR)p + DirectoryInfo->Name.Length);
//             *p++ = UNICODE_NULL;
//
//             /* Now copy the type name buffer */
//             RtlCopyMemory(p,
//                           DirectoryInfo->TypeName.Buffer,
//                           DirectoryInfo->TypeName.Length);
//
//             /* Now fixup the pointers */
//             DirectoryInfo->TypeName.Buffer = (PVOID)((ULONG_PTR)Buffer +
//                                                      ((ULONG_PTR)p -
//                                                      (ULONG_PTR)LocalBuffer));
//
//             /* Advance in buffer and NULL-terminate */
//             p = (PVOID)((ULONG_PTR)p + DirectoryInfo->TypeName.Length);
//             *p++ = UNICODE_NULL;
//
//             /* Move to the next entry */
//             DirectoryInfo++;
//         }
//
//         /* Set the key */
//         *Context = CurrentEntry;
//     }
//
//     _SEH2_TRY
//     {
//         /* Copy the buffer */
//         RtlCopyMemory(Buffer,
//                       LocalBuffer,
//                       (TotalLength <= BufferLength) ?
//                       TotalLength : BufferLength);
//
//         /* Check if the caller requested the return length and return it*/
//         if (ReturnLength) *ReturnLength = TotalLength;
//     }
//     _SEH2_EXCEPT(ExSystemExceptionFilter())
//     {
//         /* Get the exception code */
//         Status = _SEH2_GetExceptionCode();
//     }
//     _SEH2_END;
//
//     /* Unlock the directory */
//     ObpReleaseDirectoryLock(Directory, &LookupContext);
//
//     /* Dereference the directory and free our buffer */
//     ObDereferenceObject(Directory);
//     ExFreePoolWithTag(LocalBuffer, OB_NAME_TAG);
//
//     /* Return status to caller */
//     return Status;
// }
//
// /*++
// * @name NtCreateDirectoryObject
// * @implemented NT4
// *
// *     The NtOpenDirectoryObject routine creates or opens a directory object.
// *
// * @param DirectoryHandle
// *        Variable which receives the directory handle.
// *
// * @param DesiredAccess
// *        Desired access to the directory.
// *
// * @param ObjectAttributes
// *        Structure describing the directory.
// *
// * @return STATUS_SUCCESS or appropriate error value.
// *
// * @remarks None.
// *
// *--*/
// NTSTATUS
// NTAPI
// NtCreateDirectoryObject(OUT PHANDLE DirectoryHandle,
//                         IN ACCESS_MASK DesiredAccess,
//                         IN POBJECT_ATTRIBUTES ObjectAttributes)
// {
//     POBJECT_DIRECTORY Directory;
//     HANDLE NewHandle;
//     KPROCESSOR_MODE PreviousMode = ExGetPreviousMode();
//     NTSTATUS Status;
//     PAGED_CODE();
//
//     /* Check if we need to do any probing */
//     if (PreviousMode != KernelMode)
//     {
//         _SEH2_TRY
//         {
//             /* Probe the return handle */
//             ProbeForWriteHandle(DirectoryHandle);
//         }
//         _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//         {
//             /* Return the exception code */
//             _SEH2_YIELD(return _SEH2_GetExceptionCode());
//         }
//         _SEH2_END;
//     }
//
//     /* Create the object */
//     Status = ObCreateObject(PreviousMode,
//                             ObpDirectoryObjectType,
//                             ObjectAttributes,
//                             PreviousMode,
//                             NULL,
//                             sizeof(OBJECT_DIRECTORY),
//                             0,
//                             0,
//                             (PVOID*)&Directory);
//     if (!NT_SUCCESS(Status)) return Status;
//
//     /* Setup the object */
//     RtlZeroMemory(Directory, sizeof(OBJECT_DIRECTORY));
//     ExInitializePushLock(&Directory->Lock);
//     Directory->SessionId = -1;
//
//     /* Insert it into the handle table */
//     Status = ObInsertObject((PVOID)Directory,
//                             NULL,
//                             DesiredAccess,
//                             0,
//                             NULL,
//                             &NewHandle);
//
//     /* Enter SEH to protect write */
//     _SEH2_TRY
//     {
//         /* Return the handle back to the caller */
//         *DirectoryHandle = NewHandle;
//     }
//     _SEH2_EXCEPT(ExSystemExceptionFilter())
//     {
//         /* Get the exception code */
//         Status = _SEH2_GetExceptionCode();
//     }
//     _SEH2_END;
//
//     /* Return status to caller */
//     return Status;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
