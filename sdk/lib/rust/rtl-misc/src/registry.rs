use crate::shared::RtlGetProcessHeap;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::driver_ddk::*;
use ntkrnl_sys::rtl::*;
// RtlInitEmptyUnicodeString is a C FORCEINLINE (no out-of-line symbol); use the
// ntkrnl-sys inline, not the bindgen extern (which links to a nonexistent symbol).
use ntkrnl_sys::inline::common::RtlInitEmptyUnicodeString;
use ntkrnl_sys::ddk::{ACCESS_MASK, HANDLE, _UNICODE_STRING, _KEY_VALUE_PARTIAL_INFORMATION, KEY_BASIC_INFORMATION, PKEY_VALUE_PARTIAL_INFORMATION, _KEY_INFORMATION_CLASS_KeyBasicInformation, _KEY_VALUE_INFORMATION_CLASS_KeyValueFullInformation, _KEY_VALUE_INFORMATION_CLASS_KeyValuePartialInformation, _TOKEN_INFORMATION_CLASS_TokenUser, NtClose, _OBJECT_ATTRIBUTES, PHANDLE, PKEY_BASIC_INFORMATION, PKEY_VALUE_FULL_INFORMATION, PNTSTATUS, POBJECT_ATTRIBUTES, PRTL_QUERY_REGISTRY_TABLE, PSID_AND_ATTRIBUTES, PUNICODE_STRING, RtlAllocateHeap, RtlAppendUnicodeStringToString, RtlAppendUnicodeToString, RtlConvertSidToUnicodeString, RtlExpandEnvironmentStrings_U, RtlFreeHeap, RtlFreeUnicodeString, RtlInitUnicodeString, ZwClose, ZwCreateKey, ZwDeleteKey, ZwDeleteValueKey, ZwEnumerateKey, ZwEnumerateValueKey, ZwOpenKey, ZwOpenProcessTokenEx, ZwOpenThreadTokenEx, ZwQueryInformationToken, ZwQueryValueKey, ZwSetValueKey};

// ---- local constants ----
const RTL_REGISTRY_MAXIMUM: ULONG = 0x06;
const TAG_USTR: ULONG = 0x52545355;
const MAXUSHORT: USHORT = 0xFFFF;
extern "C" { static RtlpAllocDeallocQueryBufferSize: SIZE_T; }

// Sync wrapper for raw wide-string pointers (needed for static arrays)
#[repr(transparent)]
struct SyncPcwstr(*const u16);
unsafe impl Sync for SyncPcwstr {}
unsafe impl Send for SyncPcwstr {}
impl SyncPcwstr {
    const fn new(p: *const u16) -> Self { SyncPcwstr(p) }
    fn as_pcwstr(&self) -> PCWSTR { self.0 }
}

// ---- local inline helpers (C macros) ----
#[inline]
#[allow(non_snake_case)]
unsafe fn NtCurrentProcess() -> HANDLE {
    (-1isize) as HANDLE
}

#[inline]
#[allow(non_snake_case)]
unsafe fn NtCurrentThread() -> HANDLE {
    (-2isize) as HANDLE
}

// Wide-string literal macro (same pattern as ntkrnl_sys::w!)
macro_rules! wch {
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
        __W.as_ptr()
    }};
}

// ---- internal RTL library functions (provided by other compilation units in librtl.a) ----
extern "system" {
    fn RtlpAllocateMemory(Bytes: SIZE_T, Tag: ULONG) -> PVOID;
    fn RtlpFreeMemory(Mem: PVOID, Tag: ULONG);
}
// RtlpAllocateStringMemory is a macro alias for RtlpAllocateMemory
#[inline]
unsafe fn RtlpAllocateStringMemory(Length: ULONG, Tag: ULONG) -> PVOID {
    RtlpAllocateMemory(Length as SIZE_T, Tag)
}

// ---- file-scope preamble (translated) ----
pub const TAG_RTLREGISTRY: u32 = 0x76727152;

pub static RtlpRegPaths: [SyncPcwstr; RTL_REGISTRY_MAXIMUM as usize] = [
    SyncPcwstr::new(core::ptr::null::<u16>()),
    SyncPcwstr::new(wch!("\\Registry\\Machine\\System\\CurrentControlSet\\Services")),
    SyncPcwstr::new(wch!("\\Registry\\Machine\\System\\CurrentControlSet\\Control")),
    SyncPcwstr::new(wch!("\\Registry\\Machine\\Software\\Microsoft\\Windows NT\\CurrentVersion")),
    SyncPcwstr::new(wch!("\\Registry\\Machine\\Hardware\\DeviceMap")),
    SyncPcwstr::new(wch!("\\Registry\\User\\.Default")),
];

#[no_mangle]
pub unsafe extern "system" fn RtlpQueryRegistryDirect(
    ValueType: ULONG,
    ValueData: PVOID,
    ValueLength: ULONG,
    Buffer: PVOID,
) -> NTSTATUS {
    let ActualLength: USHORT;
    let ReturnString: PUNICODE_STRING = Buffer as PUNICODE_STRING;
    let mut Length: PULONG = Buffer as PULONG;
    let RealLength: ULONG;

    if (ValueType == REG_SZ) || (ValueType == REG_EXPAND_SZ) || (ValueType == REG_MULTI_SZ) {
        if ValueLength > MAXUSHORT as ULONG {
            ActualLength = MAXUSHORT;
        } else {
            ActualLength = ValueLength as USHORT;
        }

        if (*ReturnString).Buffer.is_null() {
            (*ReturnString).Buffer =
                RtlpAllocateStringMemory(ActualLength as ULONG, TAG_RTLREGISTRY) as PWSTR;
            if (*ReturnString).Buffer.is_null() {
                return STATUS_NO_MEMORY;
            }
            (*ReturnString).MaximumLength = ActualLength;
        } else if ActualLength > (*ReturnString).MaximumLength {
            return STATUS_BUFFER_TOO_SMALL;
        }

        RtlCopyMemory(
            (*ReturnString).Buffer as PVOID,
            ValueData,
            ActualLength as usize,
        );
        (*ReturnString).Length =
            ActualLength - core::mem::size_of::<u16>() as USHORT;
    } else if ValueLength <= core::mem::size_of::<ULONG>() as ULONG {
        if (Buffer != ValueData) && (ValueLength != 0) {
            RtlCopyMemory(Buffer, ValueData, ValueLength as usize);
        }
    } else {
        if (*Length as LONG) < 0 {
            RealLength = (-(*Length as LONG)) as ULONG;
            if RealLength < ValueLength {
                return STATUS_BUFFER_TOO_SMALL;
            }
            RtlCopyMemory(Buffer, ValueData, ValueLength as usize);
        } else {
            if ValueType != REG_BINARY {
                if *Length < (2 * core::mem::size_of::<ULONG>() as ULONG + ValueLength) {
                    return STATUS_BUFFER_TOO_SMALL;
                }

                *Length = ValueLength;
                Length = Length.add(1);
                *Length = ValueType;
                Length = Length.add(1);
                RtlCopyMemory(Length as PVOID, ValueData, ValueLength as usize);
            } else {
                RtlCopyMemory(Length as PVOID, ValueData, ValueLength as usize);
            }
        }
    }

    STATUS_SUCCESS
}
#[no_mangle]
pub unsafe extern "system" fn RtlpCallQueryRegistryRoutine(
    QueryTable: PRTL_QUERY_REGISTRY_TABLE,
    KeyValueInfo: PKEY_VALUE_FULL_INFORMATION,
    InfoSize: PULONG,
    Context: PVOID,
    Environment: PVOID,
) -> NTSTATUS {
    let InfoLength: ULONG;
    let mut Length: SIZE_T;
    let mut SpareLength: SIZE_T;
    let mut c: SIZE_T;
    let mut RequiredLength: ULONG = 0;
    let mut SpareData: PCHAR;
    let DataEnd: PCHAR;
    let mut Type: ULONG;
    let Name: PWCHAR;
    let mut p: PWCHAR;
    let ValueEnd: PWCHAR;
    let mut Data: PVOID;
    let mut Status: NTSTATUS;
    let mut FoundExpander: BOOLEAN = 0u8;
    let mut Source: _UNICODE_STRING = core::mem::zeroed();
    let mut Destination: _UNICODE_STRING = core::mem::zeroed();

    /* Setup defaults */
    InfoLength = *InfoSize;
    core::ptr::write_volatile(InfoSize, 0);

    /* Check if there's no data */
    if (*KeyValueInfo).DataOffset == MAXULONG {
        return if (*QueryTable).Flags & RTL_QUERY_REGISTRY_REQUIRED != 0 {
            STATUS_OBJECT_NAME_NOT_FOUND
        } else {
            STATUS_SUCCESS
        };
    }

    /* Setup spare data pointers */
    SpareData = KeyValueInfo as PCHAR;
    SpareLength = InfoLength as SIZE_T;
    DataEnd = SpareData.add(SpareLength as usize);

    /* Check if there's no value or data */
    if ((*KeyValueInfo).Type == REG_NONE)
        || ((*KeyValueInfo).DataLength == 0 && (*KeyValueInfo).Type == (*QueryTable).DefaultType)
    {
        /* Check if there's no value */
        if (*QueryTable).DefaultType == REG_NONE {
            return if (*QueryTable).Flags & RTL_QUERY_REGISTRY_REQUIRED != 0 {
                STATUS_OBJECT_NAME_NOT_FOUND
            } else {
                STATUS_SUCCESS
            };
        }

        /* We can setup a default value... capture the defaults */
        Name = (*QueryTable).Name as PWCHAR;
        Type = (*QueryTable).DefaultType;
        Data = (*QueryTable).DefaultData;
        Length = (*QueryTable).DefaultLength as SIZE_T;
        if Length == 0 {
            /* No default length given, try to calculate it */
            p = Data as PWCHAR;
            if Type == REG_SZ || Type == REG_EXPAND_SZ {
                /* This is a string, count the characters */
                while *p != 0 {
                    p = p.add(1);
                }
                p = p.add(1);
                Length = (p as ULONG_PTR - Data as ULONG_PTR) as SIZE_T;
            } else if Type == REG_MULTI_SZ {
                /* This is a multi-string, calculate all characters */
                while *p != 0 {
                    while *p != 0 {
                        p = p.add(1);
                    }
                    p = p.add(1);
                }
                Length = (p as ULONG_PTR - Data as ULONG_PTR
                    + core::mem::size_of::<u16>() as ULONG_PTR) as SIZE_T;
            }
        }
    } else {
        /* Check if we have length */
        if (*KeyValueInfo).DataLength != 0 {
            SpareData = SpareData
                .add((*KeyValueInfo).DataOffset as usize + (*KeyValueInfo).DataLength as usize);
        } else {
            let name_offset = {
                let base = KeyValueInfo as ULONG_PTR;
                core::ptr::addr_of!((*KeyValueInfo).Name) as ULONG_PTR - base
            };
            SpareData = SpareData.add(name_offset as usize + (*KeyValueInfo).NameLength as usize);
        }

        /* Align the pointer and get new size of spare data */
        SpareData = (((SpareData as ULONG_PTR) + 7) & !7) as PCHAR;
        SpareLength = (DataEnd as ULONG_PTR - SpareData as ULONG_PTR) as SIZE_T;

        /* Check if we have space to copy the data */
        RequiredLength = (*KeyValueInfo).NameLength + core::mem::size_of::<u16>() as ULONG;
        if (SpareData as ULONG_PTR > DataEnd as ULONG_PTR)
            || (SpareLength < RequiredLength as SIZE_T)
        {
            core::ptr::write_volatile(InfoSize, (SpareData as ULONG_PTR - KeyValueInfo as ULONG_PTR) as ULONG + RequiredLength);
            return STATUS_BUFFER_TOO_SMALL;
        }

        /* Check if this isn't a direct return */
        if (*QueryTable).Flags & RTL_QUERY_REGISTRY_DIRECT == 0 {
            /* Copy the data and null-terminate it */
            Name = SpareData as PWCHAR;
            RtlCopyMemory(
                Name as PVOID,
                core::ptr::addr_of!((*KeyValueInfo).Name) as PVOID,
                (*KeyValueInfo).NameLength as SIZE_T,
            );
            *Name.add((*KeyValueInfo).NameLength as usize / core::mem::size_of::<u16>()) = 0;

            /* Update the spare data information */
            SpareData = SpareData.add(RequiredLength as usize);
            SpareData = (((SpareData as ULONG_PTR) + 7) & !7) as PCHAR;
            SpareLength = (DataEnd as ULONG_PTR - SpareData as ULONG_PTR) as SIZE_T;
        } else {
            /* Just return the name */
            Name = (*QueryTable).Name as PWCHAR;
        }

        /* Capture key data */
        Type = (*KeyValueInfo).Type;
        Data = (KeyValueInfo as ULONG_PTR + (*KeyValueInfo).DataOffset as ULONG_PTR) as PVOID;
        Length = (*KeyValueInfo).DataLength as SIZE_T;
    }

    /* Check if we're expanding */
    if (*QueryTable).Flags & RTL_QUERY_REGISTRY_NOEXPAND == 0 {
        /* Check if it's a multi-string */
        if Type == REG_MULTI_SZ {
            Status = STATUS_SUCCESS;
            ValueEnd = (Data as ULONG_PTR + Length as ULONG_PTR
                - 2 * core::mem::size_of::<u16>() as ULONG_PTR) as PWSTR;
            p = Data as PWCHAR;

            while (p as ULONG_PTR) < ValueEnd as ULONG_PTR {
                while *p != 0 {
                    p = p.add(1);
                }
                p = p.add(1);

                Length = (p as ULONG_PTR - Data as ULONG_PTR) as SIZE_T;
                if (*QueryTable).Flags & RTL_QUERY_REGISTRY_DIRECT != 0 {
                    Status = RtlpQueryRegistryDirect(
                        REG_SZ,
                        Data,
                        Length as ULONG,
                        (*QueryTable).EntryContext,
                    );
                    (*QueryTable).EntryContext =
                        ((*QueryTable).EntryContext as ULONG_PTR
                            + core::mem::size_of::<_UNICODE_STRING>() as ULONG_PTR)
                            as PVOID;
                } else {
                    Status = ((*QueryTable).QueryRoutine.unwrap())(
                        Name,
                        REG_SZ,
                        Data,
                        Length as ULONG,
                        Context,
                        (*QueryTable).EntryContext,
                    );
                }

                if Status == STATUS_BUFFER_TOO_SMALL {
                    Status = STATUS_SUCCESS;
                }
                if !NT_SUCCESS(Status) {
                    break;
                }

                Data = p as PVOID;
            }

            return Status;
        }

        /* Check if this is an expand string */
        if Type == REG_EXPAND_SZ && Length >= core::mem::size_of::<u16>() as SIZE_T {
            c = Length - core::mem::size_of::<u16>() as SIZE_T;
            p = Data as PWCHAR;
            while c != 0 {
                if *p == b'%' as u16 {
                    FoundExpander = 1u8;
                    break;
                }
                p = p.add(1);
                c -= core::mem::size_of::<u16>() as SIZE_T;
            }

            if FoundExpander != 0 {
                /* Setup the source string */
                RtlInitEmptyUnicodeString(&mut Source, Data as PWSTR, Length as USHORT);
                Source.Length = Source.MaximumLength - core::mem::size_of::<u16>() as USHORT;

                /* Setup the destination string */
                RtlInitEmptyUnicodeString(&mut Destination, SpareData as PWCHAR, 0);

                if SpareLength == 0 {
                    Destination.MaximumLength = 0;
                } else if SpareLength <= MAXUSHORT as SIZE_T {
                    Destination.MaximumLength = SpareLength as USHORT;
                    *Destination
                        .Buffer
                        .add(SpareLength as usize / core::mem::size_of::<u16>() - 1) = 0;
                } else {
                    Destination.MaximumLength = MAXUSHORT as USHORT;
                    *Destination
                        .Buffer
                        .add(MAXUSHORT as usize / core::mem::size_of::<u16>() - 1) = 0;
                }

                Status = RtlExpandEnvironmentStrings_U(
                    Environment as PWSTR,
                    &mut Source,
                    &mut Destination,
                    &mut RequiredLength,
                );
                Type = REG_SZ;

                if NT_SUCCESS(Status) {
                    Data = Destination.Buffer as PVOID;
                    Length = (Destination.Length + core::mem::size_of::<u16>() as USHORT) as SIZE_T;
                } else {
                    if Status == STATUS_BUFFER_TOO_SMALL {
                        core::ptr::write_volatile(InfoSize, (SpareData as ULONG_PTR - KeyValueInfo as ULONG_PTR) as ULONG + RequiredLength);
                    }
                    return Status;
                }
            }
        }
    }

    /* Check if this is a direct query */
    if (*QueryTable).Flags & RTL_QUERY_REGISTRY_DIRECT != 0 {
        Status = RtlpQueryRegistryDirect(Type, Data, Length as ULONG, (*QueryTable).EntryContext);
    } else {
        Status = ((*QueryTable).QueryRoutine.unwrap())(
            Name,
            Type,
            Data,
            Length as ULONG,
            Context,
            (*QueryTable).EntryContext,
        );
    }

    if Status == STATUS_BUFFER_TOO_SMALL {
        STATUS_SUCCESS
    } else {
        Status
    }
}

#[no_mangle]
pub unsafe extern "system" fn RtlpAllocDeallocQueryBuffer(
    BufferSize: PSIZE_T,
    OldBuffer: PVOID,
    _OldBufferSize: SIZE_T,
    Status: PNTSTATUS,
) -> PVOID {
    let mut Buffer: PVOID = core::ptr::null_mut();

    if !Status.is_null() {
        core::ptr::write_volatile(Status, STATUS_SUCCESS as NTSTATUS);
    }

    if !OldBuffer.is_null() {
        RtlpFreeMemory(OldBuffer, TAG_RTLREGISTRY as ULONG);
    }

    if !BufferSize.is_null() {
        Buffer = RtlpAllocateMemory(*BufferSize as SIZE_T, TAG_RTLREGISTRY as ULONG);
        if Buffer.is_null() && !Status.is_null() {
            core::ptr::write_volatile(Status, STATUS_NO_MEMORY as NTSTATUS);
        }
    }

    Buffer
}

#[no_mangle]
pub unsafe extern "system" fn RtlpGetRegistryHandle(
    RelativeTo: ULONG,
    Path: PCWSTR,
    Create: BOOLEAN,
    KeyHandle: PHANDLE,
) -> NTSTATUS {
    let mut KeyPath: _UNICODE_STRING = core::mem::zeroed();
    let mut KeyName: _UNICODE_STRING = core::mem::zeroed();
    let mut KeyBuffer: [WCHAR; MAX_PATH as usize] = [0; MAX_PATH as usize];
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut Status: NTSTATUS;
    let mut RelativeTo = RelativeTo;
    let mut Path = Path;

    /* Check if we just want the handle */
    if RelativeTo & RTL_REGISTRY_HANDLE != 0 {
        core::ptr::write_volatile(KeyHandle, Path as HANDLE);
        return STATUS_SUCCESS;
    }

    /* Check for optional flag */
    if RelativeTo & RTL_REGISTRY_OPTIONAL != 0 {
        /* Mask it out */
        RelativeTo &= !RTL_REGISTRY_OPTIONAL;
    }

    /* Fail on invalid parameter */
    if RelativeTo >= RTL_REGISTRY_MAXIMUM {
        return STATUS_INVALID_PARAMETER;
    }

    /* Initialize the key name */
    RtlInitEmptyUnicodeString(
        &mut KeyName,
        KeyBuffer.as_mut_ptr(),
        core::mem::size_of_val(&KeyBuffer) as USHORT,
    );

    /* Check if we have to lookup a path to prefix */
    if RelativeTo != RTL_REGISTRY_ABSOLUTE {
        /* Check if we need the current user key */
        if RelativeTo == RTL_REGISTRY_USER {
            /* Get the user key path */
            Status = RtlFormatCurrentUserKeyPath(&mut KeyPath);

            /* Check if it worked */
            if NT_SUCCESS(Status) {
                /* Append the user key path */
                Status = RtlAppendUnicodeStringToString(&mut KeyName, &mut KeyPath);

                /* Free the user key path */
                RtlFreeUnicodeString(&mut KeyPath);
            } else {
                /* It didn't work so fall back to the default user key */
                Status = RtlAppendUnicodeToString(
                    &mut KeyName,
                    RtlpRegPaths[RTL_REGISTRY_USER as usize].as_pcwstr(),
                );
            }
        } else {
            /* Get one of the prefixes */
            Status = RtlAppendUnicodeToString(
                &mut KeyName,
                RtlpRegPaths[RelativeTo as usize].as_pcwstr(),
            );
        }

        /* Check for failure, otherwise, append the path separator */
        if !NT_SUCCESS(Status) {
            return Status;
        }
        Status = RtlAppendUnicodeToString(&mut KeyName, wch!("\\"));
        if !NT_SUCCESS(Status) {
            return Status;
        }
    }

    if !Path.is_null() {
        /* And now append the path */
        if *Path == '\\' as WCHAR && RelativeTo != RTL_REGISTRY_ABSOLUTE {
            Path = Path.add(1);
        }
        Status = RtlAppendUnicodeToString(&mut KeyName, Path);
        if !NT_SUCCESS(Status) {
            return Status;
        }
    }

    /* Initialize the object attributes */
    ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
    ObjectAttributes.RootDirectory = core::ptr::null_mut();
    ObjectAttributes.Attributes = OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE;
    ObjectAttributes.ObjectName = &mut KeyName;
    ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
    ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

    /* Check if we want to create it */
    if Create != 0 {
        /* Create the key with write privileges */
        Status = ZwCreateKey(
            KeyHandle,
            GENERIC_WRITE,
            &mut ObjectAttributes,
            0,
            core::ptr::null_mut(),
            0,
            core::ptr::null_mut(),
        );
    } else {
        /* Otherwise, just open it with read access */
        Status = ZwOpenKey(
            KeyHandle,
            MAXIMUM_ALLOWED | GENERIC_READ,
            &mut ObjectAttributes,
        );
    }

    /* Return status */
    Status
}
#[no_mangle]
pub unsafe extern "C" fn RtlpCloseRegistryHandle(RelativeTo: ULONG, KeyHandle: HANDLE) {
    if (RelativeTo & RTL_REGISTRY_HANDLE) == 0 {
        ZwClose(KeyHandle);
    }
}

#[no_mangle]
pub unsafe extern "system" fn RtlCheckRegistryKey(
    RelativeTo: ULONG,
    Path: PWSTR,
) -> NTSTATUS {
    let mut KeyHandle: HANDLE = core::ptr::null_mut();

    let Status = RtlpGetRegistryHandle(RelativeTo, Path, 0u8, &mut KeyHandle);
    if !NT_SUCCESS(Status) {
        return Status;
    }

    ZwClose(KeyHandle);
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn RtlCreateRegistryKey(
    RelativeTo: ULONG,
    Path: PWSTR,
) -> NTSTATUS {
    let mut KeyHandle: HANDLE = core::ptr::null_mut();
    let Status: NTSTATUS;

    Status = RtlpGetRegistryHandle(RelativeTo, Path, 1u8, &mut KeyHandle);
    if !NT_SUCCESS(Status) {
        return Status;
    }

    RtlpCloseRegistryHandle(RelativeTo, KeyHandle);
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn RtlDeleteRegistryValue(
    RelativeTo: ULONG,
    Path: PCWSTR,
    ValueName: PCWSTR,
) -> NTSTATUS {
    let mut KeyHandle: HANDLE = core::ptr::null_mut();
    let mut Status: NTSTATUS;
    let mut Name: _UNICODE_STRING = core::mem::zeroed();

    Status = RtlpGetRegistryHandle(RelativeTo, Path, 1u8, &mut KeyHandle);
    if !NT_SUCCESS(Status) {
        return Status;
    }

    RtlInitUnicodeString(&mut Name, ValueName);
    Status = ZwDeleteValueKey(KeyHandle, &mut Name);

    RtlpCloseRegistryHandle(RelativeTo, KeyHandle);
    Status
}

#[no_mangle]
pub unsafe extern "system" fn RtlWriteRegistryValue(
    RelativeTo: ULONG,
    Path: PCWSTR,
    ValueName: PCWSTR,
    ValueType: ULONG,
    ValueData: PVOID,
    ValueLength: ULONG,
) -> NTSTATUS {
    let mut KeyHandle: HANDLE = core::ptr::null_mut();
    let mut Status: NTSTATUS;
    let mut Name: _UNICODE_STRING = core::mem::zeroed();

    Status = RtlpGetRegistryHandle(RelativeTo, Path, 1u8, &mut KeyHandle);
    if !NT_SUCCESS(Status) {
        return Status;
    }

    RtlInitUnicodeString(&mut Name, ValueName);
    Status = ZwSetValueKey(KeyHandle, &mut Name, 0, ValueType, ValueData, ValueLength);

    RtlpCloseRegistryHandle(RelativeTo, KeyHandle);
    Status
}

#[no_mangle]
pub unsafe extern "system" fn RtlOpenCurrentUser(
    DesiredAccess: ACCESS_MASK,
    KeyHandle: PHANDLE,
) -> NTSTATUS {
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut KeyPath: _UNICODE_STRING = core::mem::zeroed();
    let mut Status: NTSTATUS;

    Status = RtlFormatCurrentUserKeyPath(&mut KeyPath);
    if NT_SUCCESS(Status) {
        ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as u32;
        ObjectAttributes.RootDirectory = core::ptr::null_mut();
        ObjectAttributes.Attributes = (OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE) as u32;
        ObjectAttributes.ObjectName = &mut KeyPath;
        ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
        ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

        Status = ZwOpenKey(KeyHandle, DesiredAccess, &mut ObjectAttributes);

        RtlFreeUnicodeString(&mut KeyPath);
        if NT_SUCCESS(Status) {
            return STATUS_SUCCESS;
        }
    }

    RtlInitUnicodeString(&mut KeyPath, RtlpRegPaths[RTL_REGISTRY_USER as usize].as_pcwstr());
    ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as u32;
    ObjectAttributes.RootDirectory = core::ptr::null_mut();
    ObjectAttributes.Attributes = (OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE) as u32;
    ObjectAttributes.ObjectName = &mut KeyPath;
    ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
    ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

    Status = ZwOpenKey(KeyHandle, DesiredAccess, &mut ObjectAttributes);

    Status
}

#[no_mangle]
pub unsafe extern "system" fn RtlFormatCurrentUserKeyPath(KeyPath: PUNICODE_STRING) -> NTSTATUS {
    let mut TokenHandle: HANDLE = core::ptr::null_mut();
    let mut Buffer: [UCHAR; 256] = [0; 256];
    let SidBuffer: PSID_AND_ATTRIBUTES;
    let mut Length: ULONG = 0;
    let mut SidString: _UNICODE_STRING = core::mem::zeroed();
    let mut Status: NTSTATUS;

    /* Open the thread token */
    Status = ZwOpenThreadTokenEx(
        NtCurrentThread(),
        TOKEN_QUERY as ACCESS_MASK,
        1u8,
        OBJ_KERNEL_HANDLE,
        &mut TokenHandle,
    );
    if !NT_SUCCESS(Status) {
        /* We failed, is it because we don't have a thread token? */
        if Status != STATUS_NO_TOKEN {
            return Status;
        }

        /* It is, so use the process token */
        Status = ZwOpenProcessTokenEx(
            NtCurrentProcess(),
            TOKEN_QUERY as ACCESS_MASK,
            OBJ_KERNEL_HANDLE,
            &mut TokenHandle,
        );
        if !NT_SUCCESS(Status) {
            return Status;
        }
    }

    /* Now query the token information */
    SidBuffer = Buffer.as_mut_ptr() as PSID_AND_ATTRIBUTES;
    Status = ZwQueryInformationToken(
        TokenHandle,
        _TOKEN_INFORMATION_CLASS_TokenUser,
        SidBuffer as PVOID,
        core::mem::size_of::<[UCHAR; 256]>() as ULONG,
        &mut Length,
    );

    /* Close the handle and handle failure */
    ZwClose(TokenHandle);
    if !NT_SUCCESS(Status) {
        return Status;
    }

    /* Convert the SID */
    Status = RtlConvertSidToUnicodeString(&mut SidString, (*SidBuffer).Sid, 1u8);
    if !NT_SUCCESS(Status) {
        return Status;
    }

    /* Add the length of the prefix */
    Length = SidString.Length as ULONG
        + (core::mem::size_of::<[u16; 16]>() as ULONG);

    /* Initialize a string */
    let buf = RtlpAllocateStringMemory(Length, TAG_USTR) as PWSTR;
    RtlInitEmptyUnicodeString(KeyPath, buf, Length as USHORT);
    if (*KeyPath).Buffer.is_null() {
        /* Free the string and fail */
        RtlFreeUnicodeString(&mut SidString);
        return STATUS_NO_MEMORY;
    }

    /* Append the prefix and SID */
    let prefix: [u16; 16] = [
        0x5c, b'R' as u16, b'E' as u16, b'G' as u16, b'I' as u16, b'S' as u16,
        b'T' as u16, b'R' as u16, b'Y' as u16, 0x5c, b'U' as u16, b'S' as u16,
        b'E' as u16, b'R' as u16, 0x5c, 0,
    ];
    RtlAppendUnicodeToString(KeyPath, prefix.as_ptr());
    RtlAppendUnicodeStringToString(KeyPath, &mut SidString);

    /* Free the temporary string and return success */
    RtlFreeUnicodeString(&mut SidString);
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn RtlpNtCreateKey(
    KeyHandle: HANDLE,
    DesiredAccess: ACCESS_MASK,
    ObjectAttributes: POBJECT_ATTRIBUTES,
    _TitleIndex: ULONG,
    _Class: PUNICODE_STRING,
    Disposition: PULONG,
) -> NTSTATUS {
    if !ObjectAttributes.is_null() {
        (*ObjectAttributes).Attributes &= !(OBJ_PERMANENT | OBJ_EXCLUSIVE);
    }

    ZwCreateKey(
        KeyHandle as PHANDLE,
        DesiredAccess,
        ObjectAttributes,
        0,
        core::ptr::null_mut(),
        0,
        Disposition,
    )
}

#[no_mangle]
pub unsafe extern "system" fn RtlpNtEnumerateSubKey(
    KeyHandle: HANDLE,
    SubKeyName: PUNICODE_STRING,
    Index: ULONG,
    _Unused: ULONG,
) -> NTSTATUS {
    let mut KeyInfo: PKEY_BASIC_INFORMATION = core::ptr::null_mut();
    let mut BufferLength: ULONG = 0;
    let mut ReturnedLength: ULONG = 0;
    let mut Status: NTSTATUS;

    if (*SubKeyName).MaximumLength != 0 {
        BufferLength = (*SubKeyName).MaximumLength as ULONG
            + core::mem::size_of::<KEY_BASIC_INFORMATION>() as ULONG;
        KeyInfo = RtlAllocateHeap(RtlGetProcessHeap(), 0, (BufferLength as SIZE_T).try_into().unwrap())
            as PKEY_BASIC_INFORMATION;
        if KeyInfo.is_null() {
            return STATUS_NO_MEMORY;
        }
    }

    Status = ZwEnumerateKey(
        KeyHandle,
        Index,
        _KEY_INFORMATION_CLASS_KeyBasicInformation,
        KeyInfo as PVOID,
        BufferLength,
        &mut ReturnedLength,
    );

    if NT_SUCCESS(Status) && !KeyInfo.is_null() {
        if (*KeyInfo).NameLength <= (*SubKeyName).MaximumLength as ULONG {
            (*SubKeyName).Length = (*KeyInfo).NameLength as USHORT;

            core::ptr::copy(
                core::ptr::addr_of!((*KeyInfo).Name) as *const u8,
                (*SubKeyName).Buffer as *mut u8,
                (*SubKeyName).Length as usize,
            );
        } else {
            Status = STATUS_BUFFER_OVERFLOW;
        }
    }

    if !KeyInfo.is_null() {
        RtlFreeHeap(RtlGetProcessHeap(), 0, KeyInfo as PVOID);
    }
    Status
}

#[no_mangle]
pub unsafe extern "system" fn RtlpNtMakeTemporaryKey(KeyHandle: HANDLE) -> NTSTATUS {
    ZwDeleteKey(KeyHandle)
}

#[no_mangle]
pub unsafe extern "system" fn RtlpNtOpenKey(
    KeyHandle: HANDLE,
    DesiredAccess: ACCESS_MASK,
    ObjectAttributes: POBJECT_ATTRIBUTES,
    _Unused: ULONG,
) -> NTSTATUS {
    if !ObjectAttributes.is_null() {
        (*ObjectAttributes).Attributes &= !(OBJ_PERMANENT | OBJ_EXCLUSIVE);
    }

    ZwOpenKey(KeyHandle as PHANDLE, DesiredAccess, ObjectAttributes)
}

#[no_mangle]
pub unsafe extern "system" fn RtlpNtQueryValueKey(
    KeyHandle: HANDLE,
    Type: PULONG,
    Data: PVOID,
    DataLength: PULONG,
    _Unused: ULONG,
) -> NTSTATUS {
    let mut ValueName: _UNICODE_STRING = core::mem::zeroed();
    let mut BufferLength: ULONG = 0;
    let Status: NTSTATUS;

    /* Clear the value name */
    RtlInitEmptyUnicodeString(&mut ValueName, core::ptr::null_mut(), 0);

    /* Check if we were already given a length */
    if !DataLength.is_null() {
        BufferLength = *DataLength;
    }

    /* Add the size of the structure */
    BufferLength += core::mem::offset_of!(_KEY_VALUE_PARTIAL_INFORMATION, Data) as ULONG;

    /* Allocate memory for the value */
    let ValueInfo = RtlAllocateHeap(RtlGetProcessHeap(), 0, BufferLength as _)
        as PKEY_VALUE_PARTIAL_INFORMATION;
    if ValueInfo.is_null() {
        return STATUS_NO_MEMORY;
    }

    /* Query the value */
    Status = ZwQueryValueKey(
        KeyHandle,
        &mut ValueName,
        _KEY_VALUE_INFORMATION_CLASS_KeyValuePartialInformation,
        ValueInfo as PVOID,
        BufferLength,
        &mut BufferLength,
    );
    if NT_SUCCESS(Status) || Status == STATUS_BUFFER_OVERFLOW {
        /* Return the length and type */
        if !DataLength.is_null() {
            core::ptr::write_volatile(DataLength, (*ValueInfo).DataLength);
        }
        if !Type.is_null() {
            core::ptr::write_volatile(Type, (*ValueInfo).Type);
        }
    }

    /* Check if the caller wanted data back, and we got it */
    if NT_SUCCESS(Status) && !Data.is_null() {
        /* Copy it */
        core::ptr::copy(
            (*ValueInfo).Data.as_ptr(),
            Data as *mut u8,
            (*ValueInfo).DataLength as usize,
        );
    }

    /* Free the memory and return status */
    RtlFreeHeap(RtlGetProcessHeap(), 0, ValueInfo as PVOID);
    Status
}

#[no_mangle]
pub unsafe extern "system" fn RtlpNtSetValueKey(
    KeyHandle: HANDLE,
    Type: ULONG,
    Data: PVOID,
    DataLength: ULONG,
) -> NTSTATUS {
    let mut ValueName: _UNICODE_STRING = core::mem::zeroed();

    RtlInitEmptyUnicodeString(&mut ValueName, core::ptr::null_mut(), 0);
    ZwSetValueKey(
        KeyHandle,
        &mut ValueName,
        0,
        Type,
        Data,
        DataLength,
    )
}

#[no_mangle]
pub unsafe extern "system" fn RtlQueryRegistryValues(
    RelativeTo: ULONG,
    Path: PCWSTR,
    QueryTable: PRTL_QUERY_REGISTRY_TABLE,
    Context: PVOID,
    Environment: PVOID,
) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let mut KeyValueInfo: PKEY_VALUE_FULL_INFORMATION;
    let mut KeyHandle: HANDLE = core::ptr::null_mut();
    let mut CurrentKey: HANDLE;
    let mut BufferSize: SIZE_T;
    let mut InfoSize: SIZE_T;
    let mut KeyPath: _UNICODE_STRING = core::mem::zeroed();
    let mut KeyValueName: _UNICODE_STRING = core::mem::zeroed();
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut i: ULONG;
    let mut Value: ULONG;
    let mut ResultLength: ULONG = 0;

    let mut QueryTable = QueryTable;

    /* Get the registry handle */
    Status = RtlpGetRegistryHandle(RelativeTo, Path, 0u8, &mut KeyHandle);
    if !NT_SUCCESS(Status) {
        return Status;
    }

    /* Initialize the path */
    RtlInitUnicodeString(
        &mut KeyPath,
        if (RelativeTo & RTL_REGISTRY_HANDLE) != 0 {
            core::ptr::null()
        } else {
            Path
        },
    );

    /* Allocate a query buffer */
    BufferSize = RtlpAllocDeallocQueryBufferSize;
    KeyValueInfo = RtlpAllocDeallocQueryBuffer(
        &mut BufferSize,
        core::ptr::null_mut(),
        0,
        &mut Status,
    ) as PKEY_VALUE_FULL_INFORMATION;
    if KeyValueInfo.is_null() {
        RtlpCloseRegistryHandle(RelativeTo, KeyHandle);
        return Status;
    }

    /* Set defaults */
    (*KeyValueInfo).DataOffset = 0;
    InfoSize = BufferSize - core::mem::size_of::<u16>() as SIZE_T;
    CurrentKey = KeyHandle;

    /* Loop the query table */
    'mainloop: while (*QueryTable).QueryRoutine.is_some()
        || ((*QueryTable).Flags & (RTL_QUERY_REGISTRY_SUBKEY | RTL_QUERY_REGISTRY_DIRECT)) != 0
    {
        let mut process_values = false;

        /* Check if the request is invalid */
        if ((*QueryTable).Flags & RTL_QUERY_REGISTRY_DIRECT) != 0
            && ((*QueryTable).Name.is_null()
                || ((*QueryTable).Flags & RTL_QUERY_REGISTRY_SUBKEY) != 0
                || (*QueryTable).QueryRoutine.is_some())
        {
            Status = STATUS_INVALID_PARAMETER;
            break;
        }

        /* Check if we want a specific key */
        if ((*QueryTable).Flags & (RTL_QUERY_REGISTRY_TOPKEY | RTL_QUERY_REGISTRY_SUBKEY)) != 0 {
            if CurrentKey != KeyHandle {
                NtClose(CurrentKey);
                CurrentKey = KeyHandle;
            }
        }

        /* Check if we're querying the subkey */
        if ((*QueryTable).Flags & RTL_QUERY_REGISTRY_SUBKEY) != 0 {
            if (*QueryTable).Name.is_null() {
                Status = STATUS_INVALID_PARAMETER;
            } else {
                RtlInitUnicodeString(&mut KeyPath, (*QueryTable).Name);

                ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
                ObjectAttributes.RootDirectory = KeyHandle;
                ObjectAttributes.Attributes = OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE;
                ObjectAttributes.ObjectName = &mut KeyPath;
                ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
                ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

                Status = ZwOpenKey(&mut CurrentKey, MAXIMUM_ALLOWED, &mut ObjectAttributes);
                if NT_SUCCESS(Status) {
                    if (*QueryTable).QueryRoutine.is_some() {
                        process_values = true;
                    }
                }
            }
        } else if !(*QueryTable).Name.is_null() && !process_values {
            RtlInitUnicodeString(&mut KeyValueName, (*QueryTable).Name);

            i = 0;
            loop {
                if {
                    let old = i;
                    i += 1;
                    old
                } > 4
                {
                    break;
                }

                Status = ZwQueryValueKey(
                    CurrentKey,
                    &mut KeyValueName,
                    _KEY_VALUE_INFORMATION_CLASS_KeyValueFullInformation,
                    KeyValueInfo as PVOID,
                    InfoSize as ULONG,
                    &mut ResultLength,
                );
                if Status == STATUS_BUFFER_OVERFLOW {
                    Status = STATUS_BUFFER_TOO_SMALL;
                }

                if !NT_SUCCESS(Status) {
                    if Status == STATUS_OBJECT_NAME_NOT_FOUND {
                        (*KeyValueInfo).Type = REG_NONE;
                        (*KeyValueInfo).DataLength = 0;
                        ResultLength = InfoSize as ULONG;

                        Status = RtlpCallQueryRegistryRoutine(
                            QueryTable,
                            KeyValueInfo,
                            &mut ResultLength,
                            Context,
                            Environment,
                        );
                    }

                    if Status == STATUS_BUFFER_TOO_SMALL {
                        BufferSize = ResultLength as SIZE_T
                            + core::mem::size_of::<usize>() as SIZE_T
                            + core::mem::size_of::<u16>() as SIZE_T;
                        KeyValueInfo = RtlpAllocDeallocQueryBuffer(
                            &mut BufferSize,
                            KeyValueInfo as PVOID,
                            BufferSize,
                            &mut Status,
                        ) as PKEY_VALUE_FULL_INFORMATION;
                        if KeyValueInfo.is_null() {
                            break;
                        }

                        (*KeyValueInfo).DataOffset = 0;
                        InfoSize = BufferSize - core::mem::size_of::<u16>() as SIZE_T;
                        continue;
                    }
                } else {
                    if (*KeyValueInfo).Type == REG_MULTI_SZ {
                        *(KeyValueInfo as *mut u16)
                            .add(ResultLength as usize / core::mem::size_of::<u16>()) = 0;
                        (*KeyValueInfo).DataLength += core::mem::size_of::<u16>() as ULONG;
                    }

                    ResultLength = InfoSize as ULONG;
                    Status = RtlpCallQueryRegistryRoutine(
                        QueryTable,
                        KeyValueInfo,
                        &mut ResultLength,
                        Context,
                        Environment,
                    );

                    if Status == STATUS_BUFFER_TOO_SMALL {
                        BufferSize = ResultLength as SIZE_T
                            + core::mem::size_of::<usize>() as SIZE_T
                            + core::mem::size_of::<u16>() as SIZE_T;
                        KeyValueInfo = RtlpAllocDeallocQueryBuffer(
                            &mut BufferSize,
                            KeyValueInfo as PVOID,
                            BufferSize,
                            &mut Status,
                        ) as PKEY_VALUE_FULL_INFORMATION;
                        if KeyValueInfo.is_null() {
                            break;
                        }

                        (*KeyValueInfo).DataOffset = 0;
                        InfoSize = BufferSize - core::mem::size_of::<u16>() as SIZE_T;
                        continue;
                    }

                    if NT_SUCCESS(Status)
                        && ((*QueryTable).Flags & RTL_QUERY_REGISTRY_DELETE) != 0
                    {
                        ZwDeleteValueKey(CurrentKey, &mut KeyValueName);
                    }
                }

                break;
            }
        } else if ((*QueryTable).Flags & RTL_QUERY_REGISTRY_NOVALUE) != 0 && !process_values {
            Status = ((*QueryTable).QueryRoutine.unwrap())(
                core::ptr::null_mut(),
                REG_NONE,
                core::ptr::null_mut(),
                0,
                Context,
                (*QueryTable).EntryContext,
            );
        } else {
            process_values = true;
        }

        if process_values {
            i = 0;
            Value = 0;
            loop {
                Status = ZwEnumerateValueKey(
                    CurrentKey,
                    Value,
                    _KEY_VALUE_INFORMATION_CLASS_KeyValueFullInformation,
                    KeyValueInfo as PVOID,
                    InfoSize as ULONG,
                    &mut ResultLength,
                );
                if Status == STATUS_BUFFER_OVERFLOW {
                    Status = STATUS_BUFFER_TOO_SMALL;
                }

                if Status == STATUS_NO_MORE_ENTRIES {
                    if Value == 0 && ((*QueryTable).Flags & RTL_QUERY_REGISTRY_REQUIRED) != 0 {
                        Status = STATUS_OBJECT_NAME_NOT_FOUND;
                    } else {
                        Status = STATUS_SUCCESS;
                    }
                    break;
                }

                if NT_SUCCESS(Status) {
                    ResultLength = InfoSize as ULONG;
                    Status = RtlpCallQueryRegistryRoutine(
                        QueryTable,
                        KeyValueInfo,
                        &mut ResultLength,
                        Context,
                        Environment,
                    );
                }

                if Status == STATUS_BUFFER_TOO_SMALL {
                    BufferSize = ResultLength as SIZE_T
                        + core::mem::size_of::<usize>() as SIZE_T
                        + core::mem::size_of::<u16>() as SIZE_T;
                    KeyValueInfo = RtlpAllocDeallocQueryBuffer(
                        &mut BufferSize,
                        KeyValueInfo as PVOID,
                        BufferSize,
                        &mut Status,
                    ) as PKEY_VALUE_FULL_INFORMATION;
                    if KeyValueInfo.is_null() {
                        break;
                    }

                    (*KeyValueInfo).DataOffset = 0;
                    InfoSize = BufferSize - core::mem::size_of::<u16>() as SIZE_T;

                    if {
                        let old = i;
                        i += 1;
                        old
                    } <= 4
                    {
                        continue;
                    }
                    break;
                }

                if !NT_SUCCESS(Status) {
                    break;
                }

                i = 0;
                if ((*QueryTable).Flags & RTL_QUERY_REGISTRY_DELETE) != 0 {
                    RtlInitEmptyUnicodeString(
                        &mut KeyValueName,
                        (*KeyValueInfo).Name.as_mut_ptr(),
                        (*KeyValueInfo).NameLength as u16,
                    );
                    KeyValueName.Length = KeyValueName.MaximumLength;

                    Status = ZwDeleteValueKey(CurrentKey, &mut KeyValueName);
                    if NT_SUCCESS(Status) {
                        Value = Value.wrapping_sub(1);
                    }
                }

                Value = Value.wrapping_add(1);
            }
        }

        if !NT_SUCCESS(Status) {
            break 'mainloop;
        }

        QueryTable = QueryTable.add(1);
    }

    /* Check if we need to close our handle */
    if !KeyHandle.is_null() {
        RtlpCloseRegistryHandle(RelativeTo, KeyHandle);
    }
    if !CurrentKey.is_null() && CurrentKey != KeyHandle {
        ZwClose(CurrentKey);
    }

    /* Free our buffer and return status */
    RtlpAllocDeallocQueryBuffer(
        core::ptr::null_mut(),
        KeyValueInfo as PVOID,
        BufferSize,
        core::ptr::null_mut(),
    );
    Status
}
