use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::mm::*;

use ntkrnl_sys::ddk::{
    ERESOURCE, ExAcquireResourceExclusiveLite, ExAllocatePool, ExFreePool,
    ExInitializeResourceLite, ExReleaseResourceLite, _IO_STATUS_BLOCK,
    KEY_VALUE_PARTIAL_INFORMATION, LARGE_INTEGER, _OBJECT_ATTRIBUTES,
    _UNICODE_STRING, PKEY_VALUE_PARTIAL_INFORMATION, PUNICODE_STRING,
    RtlInitUnicodeString, ZwClose, ZwCreateFile, ZwCreateKey,
    ZwDeleteValueKey, ZwOpenKey, ZwQueryValueKey, ZwSetValueKey, ZwWriteFile,
    _KEY_VALUE_INFORMATION_CLASS_KeyValuePartialInformation,
};
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


// Registry constants not in ntkrnl-sys
const KEY_ALL_ACCESS: ULONG = 0xF003F;
const REG_OPTION_NON_VOLATILE: ULONG = 0x00000000;

// File constants not in ntkrnl-sys
const FILE_NON_DIRECTORY_FILE: ULONG = 0x00000040;
const FILE_SYNCHRONOUS_IO_NONALERT: ULONG = 0x00000020;
const FILE_SUPERSEDE: ULONG = 0x00000000;

extern "C" {
    fn InitializeObjectAttributes( p: *mut _OBJECT_ATTRIBUTES, n: *mut _UNICODE_STRING, a: ULONG, r: HANDLE, s: PVOID, );
    fn wcslen(s: *const u16) -> usize;
}

static mut IopBootLogCreate: BOOLEAN = 0u8;
static mut IopBootLogEnabled: BOOLEAN = 0u8;
static mut IopLogFileEnabled: BOOLEAN = 0u8;
static mut IopLogEntryCount: ULONG = 0;
static mut IopBootLogResource: ERESOURCE = unsafe { core::mem::zeroed() };

// CODE_SEG("INIT")
#[no_mangle]
pub unsafe extern "system" fn IopInitBootLog(StartBootLog: BOOLEAN) {
    ExInitializeResourceLite(core::ptr::addr_of_mut!(IopBootLogResource));
    if StartBootLog != 0 {
        IopStartBootLog();
    }
}

// CODE_SEG("INIT")
#[no_mangle]
pub unsafe extern "system" fn IopStartBootLog() {
    IopBootLogCreate = 1u8;
    IopBootLogEnabled = 1u8;
}

#[no_mangle]
pub unsafe extern "system" fn IopStopBootLog() {
    IopBootLogEnabled = 0u8;
}

#[no_mangle]
pub unsafe extern "system" fn IopBootLog(DriverName: PUNICODE_STRING, Success: BOOLEAN) {
    let DriverName = opaque_ptr(DriverName);
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut Buffer: [u16; 256] = [0; 256];
    let mut ValueNameBuffer: [u16; 8] = [0; 8];
    let mut KeyName: _UNICODE_STRING = core::mem::zeroed();
    let mut ValueName: _UNICODE_STRING = core::mem::zeroed();
    let mut ControlSetKey: HANDLE = core::ptr::null_mut();
    let mut BootLogKey: HANDLE = core::ptr::null_mut();
    let Status: NTSTATUS;

    if IopBootLogEnabled == 0 {
        return;
    }

    ExAcquireResourceExclusiveLite(core::ptr::addr_of_mut!(IopBootLogResource), 1u8);

    crate::swprintf!(
        Buffer.as_mut_ptr(),
        Buffer.len(),
        crate::wstr!("%ws %wZ"),
        if Success != 0 {
            crate::wstr!("Loaded driver\0").as_ptr()
        } else {
            crate::wstr!("Did not load driver\0").as_ptr()
        },
        DriverName,
    );

    crate::swprintf!(
        ValueNameBuffer.as_mut_ptr(),
        ValueNameBuffer.len(),
        crate::wstr!("%lu"),
        IopLogEntryCount,
    );

    RtlInitUnicodeString(
        core::ptr::addr_of_mut!(KeyName),
        w!("\\Registry\\Machine\\System\\CurrentControlSet").as_ptr() as *const u16,
    );
    InitializeObjectAttributes(
        core::ptr::addr_of_mut!(ObjectAttributes),
        core::ptr::addr_of_mut!(KeyName),
        OBJ_CASE_INSENSITIVE,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );
    let Status = ZwOpenKey(
        core::ptr::addr_of_mut!(ControlSetKey),
        KEY_ALL_ACCESS,
        core::ptr::addr_of_mut!(ObjectAttributes),
    );
    if Status < 0 {
        ExReleaseResourceLite(core::ptr::addr_of_mut!(IopBootLogResource));
        return;
    }

    RtlInitUnicodeString(
        core::ptr::addr_of_mut!(KeyName),
        w!("BootLog").as_ptr() as *const u16,
    );
    InitializeObjectAttributes(
        core::ptr::addr_of_mut!(ObjectAttributes),
        core::ptr::addr_of_mut!(KeyName),
        OBJ_CASE_INSENSITIVE | OBJ_OPENIF,
        ControlSetKey,
        core::ptr::null_mut(),
    );
    let Status = ZwCreateKey(
        core::ptr::addr_of_mut!(BootLogKey),
        KEY_ALL_ACCESS,
        core::ptr::addr_of_mut!(ObjectAttributes),
        0,
        core::ptr::null_mut(),
        REG_OPTION_NON_VOLATILE,
        core::ptr::null_mut(),
    );
    if Status < 0 {
        ZwClose(ControlSetKey);
        ExReleaseResourceLite(core::ptr::addr_of_mut!(IopBootLogResource));
        return;
    }

    RtlInitUnicodeString(
        core::ptr::addr_of_mut!(ValueName),
        ValueNameBuffer.as_ptr(),
    );
    let Status = ZwSetValueKey(
        BootLogKey,
        core::ptr::addr_of_mut!(ValueName),
        0,
        REG_SZ,
        Buffer.as_ptr() as PVOID,
        (wcslen(Buffer.as_ptr()) + 1) as ULONG * core::mem::size_of::<u16>() as ULONG,
    );
    ZwClose(BootLogKey);
    ZwClose(ControlSetKey);

    if Status >= 0 {
        IopLogEntryCount += 1;
    }

    ExReleaseResourceLite(core::ptr::addr_of_mut!(IopBootLogResource));
}

unsafe fn IopWriteLogFile(LogText: PWSTR) -> NTSTATUS {
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut FileName: _UNICODE_STRING = core::mem::zeroed();
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut FileHandle: HANDLE = core::ptr::null_mut();
    let CrLf: *const u16 = w!("\r\n").as_ptr() as *const u16;
    let mut Status: NTSTATUS;

    RtlInitUnicodeString(
        core::ptr::addr_of_mut!(FileName),
        w!("\\SystemRoot\\rosboot.log").as_ptr() as *const u16,
    );
    InitializeObjectAttributes(
        core::ptr::addr_of_mut!(ObjectAttributes),
        core::ptr::addr_of_mut!(FileName),
        OBJ_CASE_INSENSITIVE | OBJ_OPENIF,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );

    Status = ZwCreateFile(
        core::ptr::addr_of_mut!(FileHandle),
        FILE_APPEND_DATA | SYNCHRONIZE,
        core::ptr::addr_of_mut!(ObjectAttributes),
        core::ptr::addr_of_mut!(IoStatusBlock),
        core::ptr::null_mut(),
        0,
        0,
        FILE_OPEN,
        FILE_NON_DIRECTORY_FILE | FILE_SYNCHRONOUS_IO_NONALERT,
        core::ptr::null_mut(),
        0,
    );
    if Status < 0 {
        return Status;
    }

    if !LogText.is_null() {
        Status = ZwWriteFile(
            FileHandle,
            core::ptr::null_mut(),
            None,
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!(IoStatusBlock),
            LogText as PVOID,
            wcslen(LogText) as ULONG * core::mem::size_of::<u16>() as ULONG,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        );
        if Status < 0 {
            ZwClose(FileHandle);
            return Status;
        }
    }

    Status = ZwWriteFile(
        FileHandle,
        core::ptr::null_mut(),
        None,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(IoStatusBlock),
        CrLf as PVOID,
        2 * core::mem::size_of::<u16>() as ULONG,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );

    ZwClose(FileHandle);

    Status
}

unsafe fn IopCreateLogFile() -> NTSTATUS {
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut FileName: _UNICODE_STRING = core::mem::zeroed();
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut FileHandle: HANDLE = core::ptr::null_mut();
    let mut ByteOffset: LARGE_INTEGER = core::mem::zeroed();
    let mut Signature: u16 = 0xFEFF;
    let mut Status: NTSTATUS;

    ExAcquireResourceExclusiveLite(core::ptr::addr_of_mut!(IopBootLogResource), 1u8);

    RtlInitUnicodeString(
        core::ptr::addr_of_mut!(FileName),
        w!("\\SystemRoot\\rosboot.log").as_ptr() as *const u16,
    );
    InitializeObjectAttributes(
        core::ptr::addr_of_mut!(ObjectAttributes),
        core::ptr::addr_of_mut!(FileName),
        OBJ_CASE_INSENSITIVE | OBJ_OPENIF,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );

    Status = ZwCreateFile(
        core::ptr::addr_of_mut!(FileHandle),
        FILE_ALL_ACCESS,
        core::ptr::addr_of_mut!(ObjectAttributes),
        core::ptr::addr_of_mut!(IoStatusBlock),
        core::ptr::null_mut(),
        0,
        0,
        FILE_SUPERSEDE,
        FILE_NON_DIRECTORY_FILE | FILE_SYNCHRONOUS_IO_NONALERT,
        core::ptr::null_mut(),
        0,
    );
    if Status < 0 {
        return Status;
    }

    ByteOffset.QuadPart = 0i64;

    Status = ZwWriteFile(
        FileHandle,
        core::ptr::null_mut(),
        None,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(IoStatusBlock),
        core::ptr::addr_of_mut!(Signature) as PVOID,
        core::mem::size_of::<u16>() as ULONG,
        core::ptr::addr_of_mut!(ByteOffset),
        core::ptr::null_mut(),
    );

    ZwClose(FileHandle);

    Status
}

#[no_mangle]
pub unsafe extern "system" fn IopSaveBootLogToFile() {
    let mut KeyInfo: PKEY_VALUE_PARTIAL_INFORMATION;
    let mut ValueNameBuffer: [u16; 8] = [0; 8];
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut KeyName: _UNICODE_STRING = core::mem::zeroed();
    let mut ValueName: _UNICODE_STRING = core::mem::zeroed();
    let mut KeyHandle: HANDLE = core::ptr::null_mut();
    let BufferSize: ULONG;
    let mut ResultLength: ULONG = 0;
    let mut i: ULONG;
    let mut Status: NTSTATUS;

    if IopBootLogCreate == 0 {
        return;
    }

    ExAcquireResourceExclusiveLite(core::ptr::addr_of_mut!(IopBootLogResource), 1u8);

    Status = IopCreateLogFile();
    if Status < 0 {
        ExReleaseResourceLite(core::ptr::addr_of_mut!(IopBootLogResource));
        return;
    }

    Status = IopWriteLogFile(core::ptr::null_mut());
    if Status < 0 {
        ExReleaseResourceLite(core::ptr::addr_of_mut!(IopBootLogResource));
        return;
    }

    BufferSize = core::mem::size_of::<KEY_VALUE_PARTIAL_INFORMATION>() as ULONG
        + 256 * core::mem::size_of::<u16>() as ULONG;
    KeyInfo = ExAllocatePool(PagedPool, BufferSize as ntkrnl_sys::ddk::SIZE_T) as PKEY_VALUE_PARTIAL_INFORMATION;
    if KeyInfo.is_null() {
        ExReleaseResourceLite(core::ptr::addr_of_mut!(IopBootLogResource));
        return;
    }

    RtlInitUnicodeString(
        core::ptr::addr_of_mut!(KeyName),
        w!("\\Registry\\Machine\\System\\CurrentControlSet\\BootLog").as_ptr() as *const u16,
    );
    InitializeObjectAttributes(
        core::ptr::addr_of_mut!(ObjectAttributes),
        core::ptr::addr_of_mut!(KeyName),
        OBJ_CASE_INSENSITIVE,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );
    Status = ZwOpenKey(
        core::ptr::addr_of_mut!(KeyHandle),
        KEY_ALL_ACCESS,
        core::ptr::addr_of_mut!(ObjectAttributes),
    );
    if Status < 0 {
        ExFreePool(KeyInfo as PVOID);
        ExReleaseResourceLite(core::ptr::addr_of_mut!(IopBootLogResource));
        return;
    }

    i = 0;
    loop {
        crate::swprintf!(
            ValueNameBuffer.as_mut_ptr(),
            ValueNameBuffer.len(),
            crate::wstr!("%lu"),
            i,
        );

        RtlInitUnicodeString(
            core::ptr::addr_of_mut!(ValueName),
            ValueNameBuffer.as_ptr(),
        );

        Status = ZwQueryValueKey(
            KeyHandle,
            core::ptr::addr_of_mut!(ValueName),
            _KEY_VALUE_INFORMATION_CLASS_KeyValuePartialInformation,
            KeyInfo as PVOID,
            BufferSize,
            core::ptr::addr_of_mut!(ResultLength),
        );
        if Status == STATUS_OBJECT_NAME_NOT_FOUND {
            break;
        }

        if Status < 0 {
            ZwClose(KeyHandle);
            ExFreePool(KeyInfo as PVOID);
            ExReleaseResourceLite(core::ptr::addr_of_mut!(IopBootLogResource));
            return;
        }

        Status = IopWriteLogFile(core::ptr::addr_of!((*KeyInfo).Data) as PWSTR);
        if Status < 0 {
            ZwClose(KeyHandle);
            ExFreePool(KeyInfo as PVOID);
            ExReleaseResourceLite(core::ptr::addr_of_mut!(IopBootLogResource));
            return;
        }

        ZwDeleteValueKey(KeyHandle, core::ptr::addr_of_mut!(ValueName));

        i += 1;
    }

    ZwClose(KeyHandle);

    ExFreePool(KeyInfo as PVOID);

    IopLogFileEnabled = 1u8;
    ExReleaseResourceLite(core::ptr::addr_of_mut!(IopBootLogResource));
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:       See COPYING in the top level directory
//  * PROJECT:         ReactOS kernel
//  * FILE:            ntoskrnl/io/iomgr/bootlog.c
//  * PURPOSE:         Boot log file support
//  *
//  * PROGRAMMERS:     Eric Kohl
//  */
//
// /* INCLUDES *****************************************************************/
//
// #include <ntoskrnl.h>
// #define NDEBUG
// #include <debug.h>
//
// /* GLOBALS ******************************************************************/
//
// static BOOLEAN IopBootLogCreate = FALSE;
// static BOOLEAN IopBootLogEnabled = FALSE;
// static BOOLEAN IopLogFileEnabled = FALSE;
// static ULONG IopLogEntryCount = 0;
// static ERESOURCE IopBootLogResource;
//
//
// /* FUNCTIONS ****************************************************************/
//
// CODE_SEG("INIT")
// VOID
// IopInitBootLog(BOOLEAN StartBootLog)
// {
//     ExInitializeResourceLite(&IopBootLogResource);
//     if (StartBootLog) IopStartBootLog();
// }
//
//
// CODE_SEG("INIT")
// VOID
// IopStartBootLog(VOID)
// {
//     IopBootLogCreate = TRUE;
//     IopBootLogEnabled = TRUE;
// }
//
//
// VOID
// IopStopBootLog(VOID)
// {
//     IopBootLogEnabled = FALSE;
// }
//
//
// VOID
// IopBootLog(PUNICODE_STRING DriverName,
//            BOOLEAN Success)
// {
//     OBJECT_ATTRIBUTES ObjectAttributes;
//     WCHAR Buffer[256];
//     WCHAR ValueNameBuffer[8];
//     UNICODE_STRING KeyName;
//     UNICODE_STRING ValueName;
//     HANDLE ControlSetKey;
//     HANDLE BootLogKey;
//     NTSTATUS Status;
//
//     if (IopBootLogEnabled == FALSE)
//         return;
//
//     ExAcquireResourceExclusiveLite(&IopBootLogResource, TRUE);
//
//     DPRINT("Boot log: %wS %wZ\n",
//            Success ? L"Loaded driver" : L"Did not load driver",
//            DriverName);
//
//     swprintf(Buffer,
//              L"%ws %wZ",
//              Success ? L"Loaded driver" : L"Did not load driver",
//              DriverName);
//
//     swprintf(ValueNameBuffer,
//              L"%lu",
//              IopLogEntryCount);
//
//     RtlInitUnicodeString(&KeyName,
//                          L"\\Registry\\Machine\\System\\CurrentControlSet");
//     InitializeObjectAttributes(&ObjectAttributes,
//                                &KeyName,
//                                OBJ_CASE_INSENSITIVE,
//                                NULL,
//                                NULL);
//     Status = ZwOpenKey(&ControlSetKey,
//                        KEY_ALL_ACCESS,
//                        &ObjectAttributes);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("ZwOpenKey() failed (Status %lx)\n", Status);
//         ExReleaseResourceLite(&IopBootLogResource);
//         return;
//     }
//
//     RtlInitUnicodeString(&KeyName, L"BootLog");
//     InitializeObjectAttributes(&ObjectAttributes,
//                                &KeyName,
//                                OBJ_CASE_INSENSITIVE | OBJ_OPENIF,
//                                ControlSetKey,
//                                NULL);
//     Status = ZwCreateKey(&BootLogKey,
//                          KEY_ALL_ACCESS,
//                          &ObjectAttributes,
//                          0,
//                          NULL,
//                          REG_OPTION_NON_VOLATILE,
//                          NULL);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("ZwCreateKey() failed (Status %lx)\n", Status);
//         ZwClose(ControlSetKey);
//         ExReleaseResourceLite(&IopBootLogResource);
//         return;
//     }
//
//     RtlInitUnicodeString(&ValueName, ValueNameBuffer);
//     Status = ZwSetValueKey(BootLogKey,
//                            &ValueName,
//                            0,
//                            REG_SZ,
//                            (PVOID)Buffer,
//                            (ULONG)(wcslen(Buffer) + 1) * sizeof(WCHAR));
//     ZwClose(BootLogKey);
//     ZwClose(ControlSetKey);
//
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("NtSetValueKey() failed (Status %lx)\n", Status);
//     }
//     else
//     {
//         IopLogEntryCount++;
//     }
//
//     ExReleaseResourceLite(&IopBootLogResource);
// }
//
//
// static
// NTSTATUS
// IopWriteLogFile(PWSTR LogText)
// {
//     OBJECT_ATTRIBUTES ObjectAttributes;
//     UNICODE_STRING FileName;
//     IO_STATUS_BLOCK IoStatusBlock;
//     HANDLE FileHandle;
//     PWSTR CrLf = L"\r\n";
//     NTSTATUS Status;
//
//     DPRINT("IopWriteLogFile() called\n");
//
//     RtlInitUnicodeString(&FileName,
//                          L"\\SystemRoot\\rosboot.log");
//     InitializeObjectAttributes(&ObjectAttributes,
//                                &FileName,
//                                OBJ_CASE_INSENSITIVE | OBJ_OPENIF,
//                                NULL,
//                                NULL);
//
//     Status = ZwCreateFile(&FileHandle,
//                           FILE_APPEND_DATA | SYNCHRONIZE,
//                           &ObjectAttributes,
//                           &IoStatusBlock,
//                           NULL,
//                           0,
//                           0,
//                           FILE_OPEN,
//                           FILE_NON_DIRECTORY_FILE | FILE_SYNCHRONOUS_IO_NONALERT,
//                           NULL,
//                           0);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("ZwCreateFile() failed (Status %lx)\n", Status);
//         return Status;
//     }
//
//     if (LogText != NULL)
//     {
//         Status = ZwWriteFile(FileHandle,
//                              NULL,
//                              NULL,
//                              NULL,
//                              &IoStatusBlock,
//                              LogText,
//                              (ULONG)wcslen(LogText) * sizeof(WCHAR),
//                              NULL,
//                              NULL);
//         if (!NT_SUCCESS(Status))
//         {
//             DPRINT1("ZwWriteFile() failed (Status %lx)\n", Status);
//             ZwClose(FileHandle);
//             return Status;
//         }
//     }
//
//     /* L"\r\n" */
//     Status = ZwWriteFile(FileHandle,
//                          NULL,
//                          NULL,
//                          NULL,
//                          &IoStatusBlock,
//                          (PVOID)CrLf,
//                          2 * sizeof(WCHAR),
//                          NULL,
//                          NULL);
//
//     ZwClose(FileHandle);
//
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("ZwWriteFile() failed (Status %lx)\n", Status);
//     }
//
//     return Status;
// }
//
//
// static
// NTSTATUS
// IopCreateLogFile(VOID)
// {
//     OBJECT_ATTRIBUTES ObjectAttributes;
//     UNICODE_STRING FileName;
//     IO_STATUS_BLOCK IoStatusBlock;
//     HANDLE FileHandle;
//     LARGE_INTEGER ByteOffset;
//     WCHAR Signature;
//     NTSTATUS Status;
//
//     DPRINT("IopSaveBootLogToFile() called\n");
//
//     ExAcquireResourceExclusiveLite(&IopBootLogResource, TRUE);
//
//     RtlInitUnicodeString(&FileName,
//                          L"\\SystemRoot\\rosboot.log");
//     InitializeObjectAttributes(&ObjectAttributes,
//                                &FileName,
//                                OBJ_CASE_INSENSITIVE | OBJ_OPENIF,
//                                NULL,
//                                NULL);
//
//     Status = ZwCreateFile(&FileHandle,
//                           FILE_ALL_ACCESS,
//                           &ObjectAttributes,
//                           &IoStatusBlock,
//                           NULL,
//                           0,
//                           0,
//                           FILE_SUPERSEDE,
//                           FILE_NON_DIRECTORY_FILE | FILE_SYNCHRONOUS_IO_NONALERT,
//                           NULL,
//                           0);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("ZwCreateFile() failed (Status %lx)\n", Status);
//         return Status;
//     }
//
//     ByteOffset.QuadPart = (LONGLONG)0;
//
//     Signature = 0xFEFF;
//     Status = ZwWriteFile(FileHandle,
//                          NULL,
//                          NULL,
//                          NULL,
//                          &IoStatusBlock,
//                          (PVOID)&Signature,
//                          sizeof(WCHAR),
//                          &ByteOffset,
//                          NULL);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("ZwWriteKey() failed (Status %lx)\n", Status);
//     }
//
//     ZwClose(FileHandle);
//
//     return Status;
// }
//
//
// VOID
// IopSaveBootLogToFile(VOID)
// {
//     PKEY_VALUE_PARTIAL_INFORMATION KeyInfo;
//     WCHAR ValueNameBuffer[8];
//     OBJECT_ATTRIBUTES ObjectAttributes;
//     UNICODE_STRING KeyName;
//     UNICODE_STRING ValueName;
//     HANDLE KeyHandle;
//     ULONG BufferSize;
//     ULONG ResultLength;
//     ULONG i;
//     NTSTATUS Status;
//
//     if (IopBootLogCreate == FALSE)
//         return;
//
//     DPRINT("IopSaveBootLogToFile() called\n");
//
//     ExAcquireResourceExclusiveLite(&IopBootLogResource, TRUE);
//
//     Status = IopCreateLogFile();
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("IopCreateLogFile() failed (Status %lx)\n", Status);
//         ExReleaseResourceLite(&IopBootLogResource);
//         return;
//     }
//
//     //Status = IopWriteLogFile(L"ReactOS "KERNEL_VERSION_STR);
//
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("IopWriteLogFile() failed (Status %lx)\n", Status);
//         ExReleaseResourceLite(&IopBootLogResource);
//         return;
//     }
//
//     Status = IopWriteLogFile(NULL);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("IopWriteLogFile() failed (Status %lx)\n", Status);
//         ExReleaseResourceLite(&IopBootLogResource);
//         return;
//     }
//
//
//     BufferSize = sizeof(KEY_VALUE_PARTIAL_INFORMATION) + 256 * sizeof(WCHAR);
//     KeyInfo = ExAllocatePool(PagedPool,
//                              BufferSize);
//     if (KeyInfo == NULL)
//     {
//         ExReleaseResourceLite(&IopBootLogResource);
//         return;
//     }
//
//     RtlInitUnicodeString(&KeyName,
//                          L"\\Registry\\Machine\\System\\CurrentControlSet\\BootLog");
//     InitializeObjectAttributes(&ObjectAttributes,
//                                &KeyName,
//                                OBJ_CASE_INSENSITIVE,
//                                NULL,
//                                NULL);
//     Status = ZwOpenKey(&KeyHandle,
//                        KEY_ALL_ACCESS,
//                        &ObjectAttributes);
//     if (!NT_SUCCESS(Status))
//     {
//         ExFreePool(KeyInfo);
//         ExReleaseResourceLite(&IopBootLogResource);
//         return;
//     }
//
//     for (i = 0; ; i++)
//     {
//         swprintf(ValueNameBuffer,
//                  L"%lu", i);
//
//         RtlInitUnicodeString(&ValueName,
//                              ValueNameBuffer);
//
//         Status = ZwQueryValueKey(KeyHandle,
//                                  &ValueName,
//                                  KeyValuePartialInformation,
//                                  KeyInfo,
//                                  BufferSize,
//                                  &ResultLength);
//         if (Status == STATUS_OBJECT_NAME_NOT_FOUND)
//         {
//             break;
//         }
//
//         if (!NT_SUCCESS(Status))
//         {
//             ZwClose(KeyHandle);
//             ExFreePool(KeyInfo);
//             ExReleaseResourceLite(&IopBootLogResource);
//             return;
//         }
//
//         Status = IopWriteLogFile((PWSTR)&KeyInfo->Data);
//         if (!NT_SUCCESS(Status))
//         {
//             ZwClose(KeyHandle);
//             ExFreePool(KeyInfo);
//             ExReleaseResourceLite(&IopBootLogResource);
//             return;
//         }
//
//         /* Delete keys */
//         ZwDeleteValueKey(KeyHandle,
//                          &ValueName);
//     }
//
//     ZwClose(KeyHandle);
//
//     ExFreePool(KeyInfo);
//
//     IopLogFileEnabled = TRUE;
//     ExReleaseResourceLite(&IopBootLogResource);
//
//     DPRINT("IopSaveBootLogToFile() done\n");
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
