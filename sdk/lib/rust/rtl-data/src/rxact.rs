use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ddk::{Commit, KEY_VALUE_BASIC_INFORMATION, KeyValueBasicInformation, KeyValueFullInformation, NtClose, NtDeleteValueKey, NtFlushKey, _OBJECT_ATTRIBUTES, PHANDLE, PKEY_VALUE_FULL_INFORMATION, PUNICODE_STRING, RtlAllocateHeap, RtlFreeHeap, RtlInitUnicodeString, ZwCreateKey, ZwDeleteKey, ZwOpenKey, ZwQueryValueKey, ZwSetValueKey};
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
pub const RXACT_DEFAULT_BUFFER_SIZE: ULONG = 4 * PAGE_SIZE;

#[repr(C)]
pub struct _RXACT_INFO {
    pub Revision: ULONG,
    pub Unknown1: ULONG,
    pub Unknown2: ULONG,
}
pub type RXACT_INFO = _RXACT_INFO;
pub type PRXACT_INFO = *mut _RXACT_INFO;

#[repr(C)]
pub struct _RXACT_DATA {
    pub ActionCount: ULONG,
    pub BufferSize: ULONG,
    pub CurrentSize: ULONG,
}
pub type RXACT_DATA = _RXACT_DATA;
pub type PRXACT_DATA = *mut _RXACT_DATA;

#[repr(C)]
pub struct _RXACT_CONTEXT {
    pub RootDirectory: HANDLE,
    pub KeyHandle: HANDLE,
    pub CanUseHandles: BOOLEAN,
    pub Data: PRXACT_DATA,
}
pub type RXACT_CONTEXT = _RXACT_CONTEXT;
pub type PRXACT_CONTEXT = *mut _RXACT_CONTEXT;

#[repr(C)]
pub struct _RXACT_ACTION {
    pub Size: ULONG,
    pub Type: ULONG,
    pub KeyName: _UNICODE_STRING,
    pub ValueName: _UNICODE_STRING,
    pub KeyHandle: HANDLE,
    pub ValueType: ULONG,
    pub ValueDataSize: ULONG,
    pub ValueData: PVOID,
}
pub type RXACT_ACTION = _RXACT_ACTION;
pub type PRXACT_ACTION = *mut _RXACT_ACTION;

pub const RXactDeleteKey: u32 = 1;
pub const RXactSetValueKey: u32 = 2;
unsafe fn RXactInitializeContext(
    Context: PRXACT_CONTEXT,
    RootDirectory: HANDLE,
    KeyHandle: HANDLE,
) {
    (*Context).Data = core::ptr::null_mut();
    (*Context).RootDirectory = RootDirectory;
    (*Context).CanUseHandles = 1u8;
    (*Context).KeyHandle = KeyHandle;
}
unsafe fn RXactpOpenTargetKey(
    RootDirectory: HANDLE,
    ActionType: ULONG,
    KeyName: PUNICODE_STRING,
    KeyHandle: PHANDLE,
) -> NTSTATUS {
    let Status: NTSTATUS;
    let mut Disposition: ULONG = 0;
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();

    if ActionType == RXactDeleteKey as ULONG {
        ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
        ObjectAttributes.RootDirectory = RootDirectory;
        ObjectAttributes.ObjectName = KeyName;
        ObjectAttributes.Attributes = OBJ_CASE_INSENSITIVE;
        ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
        ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

        Status = ZwOpenKey(KeyHandle, DELETE, core::ptr::addr_of_mut!(ObjectAttributes));
    } else if ActionType == RXactSetValueKey as ULONG {
        ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
        ObjectAttributes.RootDirectory = RootDirectory;
        ObjectAttributes.ObjectName = KeyName;
        ObjectAttributes.Attributes = OBJ_CASE_INSENSITIVE | OBJ_OPENIF;
        ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
        ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

        Status = ZwCreateKey(
            KeyHandle,
            KEY_WRITE,
            core::ptr::addr_of_mut!(ObjectAttributes),
            0,
            core::ptr::null_mut(),
            0,
            core::ptr::addr_of_mut!(Disposition),
        );
    } else {
        return STATUS_INVALID_PARAMETER;
    }

    Status
}
unsafe fn RXactpCommit(Context: PRXACT_CONTEXT) -> NTSTATUS {
    let Data: PRXACT_DATA = (*Context).Data;

    let mut Action: PRXACT_ACTION = Data.add(1) as PRXACT_ACTION;

    let mut i: ULONG = 0;
    while i < (*Data).ActionCount {
        (*Action).KeyName.Buffer =
            (Data as *mut u8).add((*Action).KeyName.Buffer as usize) as PWSTR;
        (*Action).ValueName.Buffer =
            (Data as *mut u8).add((*Action).ValueName.Buffer as usize) as PWSTR;
        (*Action).ValueData =
            (Data as *mut u8).add((*Action).ValueData as usize) as *mut core::ffi::c_void;

        if (*Action).Type == RXactDeleteKey as ULONG {
            if (*Action).KeyHandle != INVALID_HANDLE_VALUE as HANDLE
                && (*Context).CanUseHandles != 0
            {
                let Status = ZwDeleteKey((*Action).KeyHandle);
                if !NT_SUCCESS(Status) {
                    return Status;
                }
            } else {
                let mut KeyHandle: HANDLE = core::ptr::null_mut();
                let Status = RXactpOpenTargetKey(
                    (*Context).RootDirectory,
                    RXactDeleteKey as ULONG,
                    core::ptr::addr_of_mut!((*Action).KeyName),
                    &mut KeyHandle,
                );
                if NT_SUCCESS(Status) {
                    let Status = ZwDeleteKey(KeyHandle);
                    let _TmpStatus = NtClose(KeyHandle);
                    if !NT_SUCCESS(Status) {
                        return Status;
                    }
                } else {
                    if Status != STATUS_OBJECT_NAME_NOT_FOUND {
                        return Status;
                    }
                }
            }
        } else if (*Action).Type == RXactSetValueKey as ULONG {
            if (*Action).KeyHandle != INVALID_HANDLE_VALUE as HANDLE
                && (*Context).CanUseHandles != 0
            {
                let Status = ZwSetValueKey(
                    (*Action).KeyHandle,
                    core::ptr::addr_of_mut!((*Action).ValueName),
                    0,
                    (*Action).ValueType,
                    (*Action).ValueData,
                    (*Action).ValueDataSize,
                );
                if !NT_SUCCESS(Status) {
                    return Status;
                }
            } else {
                let mut KeyHandle: HANDLE = core::ptr::null_mut();
                let Status = RXactpOpenTargetKey(
                    (*Context).RootDirectory,
                    RXactSetValueKey as ULONG,
                    core::ptr::addr_of_mut!((*Action).KeyName),
                    &mut KeyHandle,
                );
                if !NT_SUCCESS(Status) {
                    return Status;
                }

                let Status = ZwSetValueKey(
                    KeyHandle,
                    core::ptr::addr_of_mut!((*Action).ValueName),
                    0,
                    (*Action).ValueType,
                    (*Action).ValueData,
                    (*Action).ValueDataSize,
                );

                let _TmpStatus = NtClose(KeyHandle);

                if !NT_SUCCESS(Status) {
                    return Status;
                }
            }
        } else {
            return STATUS_INVALID_PARAMETER;
        }

        Action = (Action as *mut u8).add((*Action).Size as usize) as PRXACT_ACTION;
        i += 1;
    }

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn RtlStartRXact(Context: PRXACT_CONTEXT) -> NTSTATUS {
    if !(*Context).Data.is_null() {
        return STATUS_RXACT_INVALID_STATE;
    }

    let Buffer = RtlAllocateHeap(RtlGetProcessHeap(), 0, (4 * 4096) as usize) as PRXACT_DATA;
    if Buffer.is_null() {
        return STATUS_NO_MEMORY;
    }

    (*Buffer).ActionCount = 0;
    (*Buffer).BufferSize = 4 * 4096;
    (*Buffer).CurrentSize = core::mem::size_of::<RXACT_DATA>() as u32;
    (*Context).Data = Buffer;

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn RtlAbortRXact(Context: PRXACT_CONTEXT) -> NTSTATUS {
    if (*Context).Data.is_null() {
        return STATUS_RXACT_INVALID_STATE;
    }

    RtlFreeHeap(RtlGetProcessHeap(), 0, (*Context).Data as PVOID);

    RXactInitializeContext(Context, (*Context).RootDirectory, (*Context).KeyHandle);

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn RtlInitializeRXact(
    RootDirectory: HANDLE,
    Commit: BOOLEAN,
    OutContext: *mut PRXACT_CONTEXT,
) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let _TmpStatus: NTSTATUS;
    let Context: PRXACT_CONTEXT;
    let KeyValueInformation: PKEY_VALUE_FULL_INFORMATION;
    let mut KeyValueBasicInfo: KEY_VALUE_BASIC_INFORMATION = core::mem::zeroed();
    let mut ValueName: _UNICODE_STRING = core::mem::zeroed();
    let mut KeyName: _UNICODE_STRING = core::mem::zeroed();
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut TransactionInfo: RXACT_INFO = core::mem::zeroed();
    let mut Disposition: ULONG = 0;
    let mut ValueType: ULONG = 0;
    let mut ValueDataLength: ULONG;
    let mut Length: ULONG = 0;
    let mut KeyHandle: HANDLE = core::ptr::null_mut();

    RtlInitUnicodeString(&mut KeyName, "RXACT\0".encode_utf16().collect::<Vec<u16>>().as_ptr());

    ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
    ObjectAttributes.RootDirectory = RootDirectory;
    ObjectAttributes.ObjectName = &mut KeyName;
    ObjectAttributes.Attributes = OBJ_CASE_INSENSITIVE | OBJ_OPENIF;
    ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
    ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

    Status = ZwCreateKey(
        &mut KeyHandle,
        KEY_READ | KEY_WRITE | DELETE,
        &mut ObjectAttributes,
        0,
        core::ptr::null_mut(),
        0,
        &mut Disposition,
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }

    Context = RtlAllocateHeap(RtlGetProcessHeap(), 0, core::mem::size_of::<RXACT_CONTEXT>()) as PRXACT_CONTEXT;
    *OutContext = Context;
    if Context.is_null() {
        let _ = ZwDeleteKey(KeyHandle);
        let _ = NtClose(KeyHandle);
        return STATUS_NO_MEMORY;
    }

    RXactInitializeContext(Context, RootDirectory, KeyHandle);

    if Disposition == REG_CREATED_NEW_KEY {
        TransactionInfo.Revision = 1;
        RtlInitUnicodeString(&mut ValueName, core::ptr::null_mut());
        Status = ZwSetValueKey(
            KeyHandle,
            &mut ValueName,
            0,
            REG_NONE,
            &mut TransactionInfo as *mut _ as PVOID,
            core::mem::size_of::<RXACT_INFO>() as ULONG,
        );
        if !NT_SUCCESS(Status) {
            let _ = ZwDeleteKey(KeyHandle);
            let _ = NtClose(KeyHandle);
            RtlFreeHeap(RtlGetProcessHeap(), 0, *OutContext as PVOID);
            return Status;
        }

        return STATUS_RXACT_STATE_CREATED;
    } else {
        ValueDataLength = core::mem::size_of::<RXACT_INFO>() as ULONG;
        Status = RtlpNtQueryValueKey(
            KeyHandle,
            &mut ValueType,
            &mut TransactionInfo as *mut _ as PVOID,
            &mut ValueDataLength,
            0,
        );
        if !NT_SUCCESS(Status) {
            let _ = NtClose(KeyHandle);
            RtlFreeHeap(RtlGetProcessHeap(), 0, Context as PVOID);
            return Status;
        }

        if (ValueDataLength != core::mem::size_of::<RXACT_INFO>() as ULONG)
            || (TransactionInfo.Revision != 1)
        {
            let _ = NtClose(KeyHandle);
            RtlFreeHeap(RtlGetProcessHeap(), 0, Context as PVOID);
            return STATUS_UNKNOWN_REVISION;
        }

        RtlInitUnicodeString(&mut ValueName, "Log\0".encode_utf16().collect::<Vec<u16>>().as_ptr());
        Status = ZwQueryValueKey(
            KeyHandle,
            &mut ValueName,
            KeyValueBasicInformation,
            &mut KeyValueBasicInfo as *mut _ as PVOID,
            core::mem::size_of::<KEY_VALUE_BASIC_INFORMATION>() as ULONG,
            &mut Length,
        );
        if !NT_SUCCESS(Status) {
            return STATUS_SUCCESS;
        }

        if Commit == 0 {
            return STATUS_RXACT_COMMIT_NECESSARY;
        }

        Status = ZwQueryValueKey(
            KeyHandle,
            &mut ValueName,
            KeyValueFullInformation,
            core::ptr::null_mut(),
            0,
            &mut Length,
        );
        if Status != STATUS_BUFFER_TOO_SMALL {
            return Status;
        }

        KeyValueInformation = RtlAllocateHeap(RtlGetProcessHeap(), 0, Length as usize) as PKEY_VALUE_FULL_INFORMATION;
        if KeyValueInformation.is_null() {
            return STATUS_NO_MEMORY;
        }

        Status = ZwQueryValueKey(
            KeyHandle,
            &mut ValueName,
            KeyValueFullInformation,
            KeyValueInformation as PVOID,
            Length,
            &mut Length,
        );
        if !NT_SUCCESS(Status) {
            RtlFreeHeap(RtlGetProcessHeap(), 0, KeyValueInformation as PVOID);
            RtlFreeHeap(RtlGetProcessHeap(), 0, Context as PVOID);
            return Status;
        }

        (*Context).Data = ((KeyValueInformation as PUCHAR)
            .add((*KeyValueInformation).DataOffset as usize)) as PRXACT_DATA;

        (*Context).CanUseHandles = 0u8;

        Status = RXactpCommit(Context);
        if !NT_SUCCESS(Status) {
            RtlFreeHeap(RtlGetProcessHeap(), 0, KeyValueInformation as PVOID);
            RtlFreeHeap(RtlGetProcessHeap(), 0, Context as PVOID);
            return Status;
        }

        Status = NtDeleteValueKey(KeyHandle, &mut ValueName);

        (*Context).Data = KeyValueInformation as PRXACT_DATA;

        Status = RtlAbortRXact(Context);

        return Status;
    }
}

#[no_mangle]
pub unsafe extern "system" fn RtlAddAttributeActionToRXact(
    Context: PRXACT_CONTEXT,
    ActionType: ULONG,
    KeyName: PUNICODE_STRING,
    KeyHandle: HANDLE,
    ValueName: PUNICODE_STRING,
    ValueType: ULONG,
    ValueData: PVOID,
    ValueDataSize: ULONG,
) -> NTSTATUS {
    let ActionSize: ULONG;
    let RequiredSize: ULONG;
    let mut BufferSize: ULONG;
    let mut CurrentOffset: ULONG;
    let NewData: PRXACT_DATA;
    let Action: PRXACT_ACTION;

    /* Validate ActionType parameter */
    if (ActionType != RXactDeleteKey as ULONG) && (ActionType != RXactSetValueKey as ULONG) {
        return STATUS_INVALID_PARAMETER;
    }

    /* Calculate the size of the new action record */
    ActionSize = ALIGN_UP_BY((*ValueName).Length as ULONG, core::mem::size_of::<ULONG>() as ULONG)
        + ALIGN_UP_BY(ValueDataSize, core::mem::size_of::<ULONG>() as ULONG)
        + ALIGN_UP_BY((*KeyName).Length as ULONG, core::mem::size_of::<ULONG>() as ULONG)
        + ALIGN_UP_BY(
            core::mem::size_of::<RXACT_ACTION>() as ULONG,
            core::mem::size_of::<ULONG>() as ULONG,
        );

    /* Calculate the new buffer size we need */
    RequiredSize = ActionSize + (*(*Context).Data).CurrentSize;

    /* Check for integer overflow */
    if RequiredSize < ActionSize {
        return STATUS_NO_MEMORY;
    }

    /* Check if the buffer is large enough */
    BufferSize = (*(*Context).Data).BufferSize;
    if RequiredSize > BufferSize {
        /* Increase by a factor of 2, until it is large enough */
        while BufferSize < RequiredSize {
            BufferSize *= 2;
        }

        /* Allocate a new buffer from the heap */
        NewData = RtlAllocateHeap(RtlGetProcessHeap(), 0, BufferSize as usize) as PRXACT_DATA;
        if NewData.is_null() {
            return STATUS_NO_MEMORY;
        }

        /* Copy the old buffer to the new one */
        RtlCopyMemory(
            NewData as PVOID,
            (*Context).Data as PVOID,
            (*(*Context).Data).CurrentSize as usize,
        );

        /* Free the old buffer and use the new one */
        RtlFreeHeap(RtlGetProcessHeap(), 0, (*Context).Data as PVOID);
        (*Context).Data = NewData;
        (*NewData).BufferSize = BufferSize;
    }

    /* Get the next action record */
    Action = ((*Context).Data as PUCHAR).add((*(*Context).Data).CurrentSize as usize)
        as PRXACT_ACTION;

    /* Fill in the fields */
    (*Action).Size = ActionSize;
    (*Action).Type = ActionType;
    (*Action).KeyName = *KeyName;
    (*Action).ValueName = *ValueName;
    (*Action).ValueType = ValueType;
    (*Action).ValueDataSize = ValueDataSize;
    (*Action).KeyHandle = KeyHandle;

    /* Copy the key name (and convert the pointer to a buffer offset) */
    CurrentOffset = (*(*Context).Data).CurrentSize + core::mem::size_of::<RXACT_ACTION>() as ULONG;
    (*Action).KeyName.Buffer = UlongToPtr(CurrentOffset) as PWSTR;
    RtlCopyMemory(
        ((*Context).Data as PUCHAR).add(CurrentOffset as usize) as PVOID,
        (*KeyName).Buffer as PVOID,
        (*KeyName).Length as usize,
    );

    /* Copy the value name (and convert the pointer to a buffer offset) */
    CurrentOffset += ALIGN_UP_BY((*KeyName).Length as ULONG, core::mem::size_of::<ULONG>() as ULONG);
    (*Action).ValueName.Buffer = UlongToPtr(CurrentOffset) as PWSTR;
    RtlCopyMemory(
        ((*Context).Data as PUCHAR).add(CurrentOffset as usize) as PVOID,
        (*ValueName).Buffer as PVOID,
        (*ValueName).Length as usize,
    );

    /* Update the offset */
    CurrentOffset += ALIGN_UP_BY((*ValueName).Length as ULONG, core::mem::size_of::<ULONG>() as ULONG);

    /* Is this a set action? */
    if ActionType == RXactSetValueKey as ULONG {
        /* Copy the key value data as well */
        (*Action).ValueData = UlongToPtr(CurrentOffset) as PVOID;
        RtlCopyMemory(
            ((*Context).Data as PUCHAR).add(CurrentOffset as usize) as PVOID,
            ValueData,
            ValueDataSize as usize,
        );
        CurrentOffset += ALIGN_UP_BY(ValueDataSize, core::mem::size_of::<ULONG>() as ULONG);
    }

    /* Update data site and action count */
    (*(*Context).Data).CurrentSize = CurrentOffset;
    (*(*Context).Data).ActionCount += 1;

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn RtlAddActionToRXact(
    Context: PRXACT_CONTEXT,
    ActionType: ULONG,
    KeyName: PUNICODE_STRING,
    ValueType: ULONG,
    ValueData: PVOID,
    ValueDataSize: ULONG,
) -> NTSTATUS {
    let mut ValueName: _UNICODE_STRING = core::mem::zeroed();

    RtlInitUnicodeString(&mut ValueName, core::ptr::null_mut());
    RtlAddAttributeActionToRXact(
        Context,
        ActionType,
        KeyName,
        INVALID_HANDLE_VALUE as HANDLE,
        &mut ValueName,
        ValueType,
        ValueData,
        ValueDataSize,
    )
}

#[no_mangle]
pub unsafe extern "system" fn RtlApplyRXactNoFlush(
    Context: PRXACT_CONTEXT,
) -> NTSTATUS {
    let mut Status: NTSTATUS;

    Status = RXactpCommit(Context);
    if !NT_SUCCESS(Status) {
        return Status;
    }

    Status = RtlAbortRXact(Context);

    Status
}

#[no_mangle]
pub unsafe extern "system" fn RtlApplyRXact(Context: PRXACT_CONTEXT) -> NTSTATUS {
    let mut ValueName: _UNICODE_STRING = core::mem::zeroed();
    let mut Status: NTSTATUS;

    RtlInitUnicodeString(&mut ValueName, "L\0o\0g\0\0\0".as_ptr() as PWSTR);
    Status = ZwSetValueKey(
        (*Context).KeyHandle,
        &mut ValueName,
        0,
        REG_BINARY,
        (*Context).Data as PVOID,
        (*(*Context).Data).CurrentSize,
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }

    Status = NtFlushKey((*Context).KeyHandle);
    if !NT_SUCCESS(Status) {
        NtDeleteValueKey((*Context).KeyHandle, &mut ValueName);
        return Status;
    }

    Status = RXactpCommit(Context);
    if !NT_SUCCESS(Status) {
        NtDeleteValueKey((*Context).KeyHandle, &mut ValueName);
        return Status;
    }

    Status = NtDeleteValueKey((*Context).KeyHandle, &mut ValueName);

    Status = RtlAbortRXact(Context);

    let _ = Status;

    STATUS_SUCCESS
}
