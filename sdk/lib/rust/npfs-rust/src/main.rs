//! main.rs — main.c: Named Pipe FileSystem Driver Initialization.
//! DriverEntry, the alias-registry bootstrap (NpReadAlias/NpInitializeAliases/NpCompareAliasNames),
//! and the stub NpFsdDirectoryControl.
#![allow(non_snake_case, non_camel_case_types, dead_code, unused_imports, unused_variables, unused_mut, unused_parens, unused_assignments)]

use core::mem::{size_of, transmute, zeroed};
use core::ptr::{addr_of, addr_of_mut, null_mut};

use ntkrnl_sys::base::{NT_SUCCESS, InitializeListHead, InsertHeadList, InsertTailList, RemoveEntryList, RemoveHeadList, RemoveTailList, IsListEmpty};
use ntkrnl_sys::ddk::*;
use ntkrnl_sys::ex::{ExAllocatePoolWithTag, ExFreePoolWithTag};
use ntkrnl_sys::driver_ddk::{wcslen, IoCompleteRequest};

// Both `base::*` and `ddk::*` (bindgen) export these primitive type aliases (to the same
// underlying machine type, but as distinct `pub type` items) → a bare reference would be an
// E0659 glob ambiguity. Pin them explicitly to base so the bare names resolve unambiguously.
use ntkrnl_sys::base::{
    BOOLEAN, LONG, NTSTATUS, PVOID, PWCHAR, PWSTR, SIZE_T, UCHAR, ULONG, ULONG_PTR, USHORT, WCHAR,
};

use crate::glue::*;
use crate::globals::*;
use crate::types::*;
use crate::helpers::*;
use crate::CONTAINING_RECORD;

// Cross-module dispatch routines wired into DriverObject->MajorFunction[].
use crate::create::{NpFsdCreate, NpFsdCreateNamedPipe};
use crate::close::NpFsdClose;
use crate::read::{NpFastRead, NpFsdRead};
use crate::write::{NpFastWrite, NpFsdWrite};
use crate::fileinfo::{NpFsdQueryInformation, NpFsdSetInformation};
use crate::volinfo::NpFsdQueryVolumeInformation;
use crate::cleanup::NpFsdCleanup;
use crate::flushbuf::NpFsdFlushBuffers;
use crate::fsctrl::NpFsdFileSystemControl;
use crate::seinfo::{NpFsdQuerySecurityInfo, NpFsdSetSecurityInfo};
use crate::strucsup::{NpInitializeVcb, NpCreateRootDcb};

// File ID number for NPFS bugchecking support: NPFS_BUGCHECK_MAIN (glue.rs).

// ---- constants not in glue.rs (header-harvested; flag in summary) ----------
const NPFS_BUGCHECK_FILE_ID: ULONG = NPFS_BUGCHECK_MAIN; // glue.rs: NPFS_BUGCHECK_MAIN (0x0009)

const REG_MULTI_SZ: ULONG = 7; // sdk/include/xdk/cmtypes.h: REG_MULTI_SZ
const REG_NONE: ULONG = 0; // sdk/include/xdk/cmtypes.h: REG_NONE
const RTL_QUERY_REGISTRY_NOEXPAND: ULONG = 0x0000_0010; // sdk/include/xdk/rtltypes.h: RTL_QUERY_REGISTRY_NOEXPAND
const RTL_REGISTRY_SERVICES: ULONG = 1; // sdk/include/xdk/rtltypes.h: RTL_REGISTRY_SERVICES
const RTL_REGISTRY_OPTIONAL: ULONG = 0x8000_0000; // sdk/include/xdk/rtltypes.h: RTL_REGISTRY_OPTIONAL
const OBJ_NAME_PATH_SEPARATOR: u16 = b'\\' as u16; // sdk/include/xdk/obtypes.h: OBJ_NAME_PATH_SEPARATOR (L'\\')
const UNICODE_NULL: u16 = 0; // sdk/include/xdk/ntbasedef.h: UNICODE_NULL ((WCHAR)0)
const FILE_DEVICE_NAMED_PIPE: ULONG = 0x0000_0011; // sdk/include/xdk/iotypes.h: FILE_DEVICE_NAMED_PIPE
const DO_LONG_TERM_REQUESTS: ULONG = 0x0000_0200; // sdk/include/xdk/iotypes.h: DO_LONG_TERM_REQUESTS
const IO_NO_INCREMENT: i8 = 0; // sdk/include/xdk/iotypes.h: IO_NO_INCREMENT

/// wsz — const ASCII &[u8] → NUL-terminated UTF-16 `[u16; N]`. N = char count + 1.
const fn wsz<const N: usize>(s: &[u8]) -> [u16; N] {
    let mut a = [0u16; N];
    let mut i = 0;
    while i < s.len() {
        a[i] = s[i] as u16;
        i += 1;
    }
    a
}

// L"\\Device\\NamedPipe" — 17 wide chars + NUL.
static NAMEDPIPE_DEVICE_NAME: [u16; 18] = wsz(b"\\Device\\NamedPipe");

/* GLOBALS ********************************************************************/

pub static mut NpfsDeviceObject: PDEVICE_OBJECT = null_mut();
pub static mut NpAliases: PVOID = null_mut();
// NpAliasList / NpAliasListByLength[] are centralized in crate::globals.

/// FAST_IO_DISPATCH = { sizeof, NULL (FastIoCheckIfPossible), NpFastRead, NpFastWrite }.
/// Built at runtime in DriverEntry (statics can't hold transmuted fn pointers cleanly across
/// arch ABIs) — see DriverEntry. The C had it as a file-scope initialized struct.
pub static mut NpFastIoDispatch: FAST_IO_DISPATCH = unsafe { zeroed() };

/* FUNCTIONS ******************************************************************/

#[no_mangle]
pub unsafe extern "system" fn NpReadAlias(
    ValueName: PWSTR,
    ValueType: ULONG,
    ValueData: PVOID,
    ValueLength: ULONG,
    Context: PVOID,
    EntryContext: PVOID,
) -> NTSTATUS {
    let QueryContext = Context as PNPFS_QUERY_VALUE_CONTEXT;
    let mut CurrentString: PWSTR;
    let mut Length: SIZE_T;
    let mut CurrentAlias: PNPFS_ALIAS;
    let mut TempString: UNICODE_STRING = zeroed();
    let mut CurrentTargetName: *mut UNICODE_STRING;

    /* Check if we have the expected type */
    if ValueType != REG_MULTI_SZ {
        return STATUS_INVALID_PARAMETER;
    }

    /* Check if only the size is requested */
    if (*QueryContext).SizeOnly != FALSE {
        /* Count this entry */
        (*QueryContext).NumberOfEntries += 1;

        /* Get the length of the value name (i.e. the target name). */
        Length = (wcslen(ValueName) * size_of::<u16>()) as SIZE_T;

        /* Add the size of the name plus a '\' and a UNICODE_STRING structure */
        (*QueryContext).FullSize += Length
            + size_of::<u16>() as SIZE_T            // sizeof(UNICODE_NULL)
            + size_of::<u16>() as SIZE_T            // sizeof(OBJ_NAME_PATH_SEPARATOR)
            + size_of::<UNICODE_STRING>() as SIZE_T;

        /* Loop while we have alias names */
        CurrentString = ValueData as PWSTR;
        while *CurrentString != UNICODE_NULL {
            /* Count this alias */
            (*QueryContext).NumberOfAliases += 1;

            /* Get the length of the current string (i.e. the alias name) */
            Length = (wcslen(CurrentString) * size_of::<u16>()) as SIZE_T;

            /* Count the length plus the size of an NPFS_ALIAS structure */
            (*QueryContext).FullSize +=
                Length + size_of::<u16>() as SIZE_T + size_of::<NPFS_ALIAS>() as SIZE_T;

            /* Go to the next string */
            CurrentString = CurrentString
                .add((Length as usize + size_of::<u16>()) / size_of::<u16>());
        }
    } else {
        /* Get the next name string pointer */
        CurrentTargetName = (*QueryContext).CurrentTargetName;
        (*QueryContext).CurrentTargetName = (*QueryContext).CurrentTargetName.add(1);

        /* Get the length of the value name (i.e. the target name). */
        Length = (wcslen(ValueName) * size_of::<u16>()) as SIZE_T;

        /* Initialize the current name string (one char more than the name) */
        (*CurrentTargetName).Buffer = (*QueryContext).CurrentStringPointer;
        (*CurrentTargetName).Length = (Length + size_of::<u16>() as SIZE_T) as USHORT; // + sizeof(OBJ_NAME_PATH_SEPARATOR)
        (*CurrentTargetName).MaximumLength =
            ((*CurrentTargetName).Length as SIZE_T + size_of::<u16>() as SIZE_T) as USHORT; // + sizeof(UNICODE_NULL)

        /* Update the current string pointer */
        (*QueryContext).CurrentStringPointer = (*QueryContext)
            .CurrentStringPointer
            .add((*CurrentTargetName).MaximumLength as usize / size_of::<u16>());

        /* Prepend a '\' before the name */
        *(*CurrentTargetName).Buffer.add(0) = OBJ_NAME_PATH_SEPARATOR;

        /* Append the value name (including the NULL termination) */
        core::ptr::copy_nonoverlapping(
            ValueName,
            (*CurrentTargetName).Buffer.add(1),
            (Length as usize + size_of::<u16>()) / size_of::<u16>(),
        );

        /* Upcase the target name */
        RtlUpcaseUnicodeString(CurrentTargetName, CurrentTargetName, 0);

        /* Loop while we have alias names */
        CurrentString = ValueData as PWSTR;
        while *CurrentString != UNICODE_NULL {
            /* Get the next alias pointer */
            CurrentAlias = (*QueryContext).CurrentAlias;
            (*QueryContext).CurrentAlias = (*QueryContext).CurrentAlias.add(1);

            /* Get the length of the current string (i.e. the alias name) */
            Length = (wcslen(CurrentString) * size_of::<u16>()) as SIZE_T;

            /* Setup the alias structure */
            (*CurrentAlias).TargetName = CurrentTargetName;
            (*CurrentAlias).Name.Buffer = (*QueryContext).CurrentStringPointer;
            (*CurrentAlias).Name.Length = Length as USHORT;
            (*CurrentAlias).Name.MaximumLength = (Length + size_of::<u16>() as SIZE_T) as USHORT;

            /* Upcase the alias name */
            TempString.Buffer = CurrentString;
            TempString.Length = Length as USHORT;
            RtlUpcaseUnicodeString(
                addr_of_mut!((*CurrentAlias).Name),
                addr_of_mut!(TempString),
                FALSE as u8,
            );

            /* Update the current string pointer */
            (*QueryContext).CurrentStringPointer = (*QueryContext)
                .CurrentStringPointer
                .add((*CurrentAlias).Name.MaximumLength as usize / size_of::<u16>());

            /* Go to the next string */
            CurrentString = CurrentString
                .add((Length as usize + size_of::<u16>()) / size_of::<u16>());
        }
    }

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn NpCompareAliasNames(
    String1: PCUNICODE_STRING,
    String2: PCUNICODE_STRING,
) -> LONG {
    let mut Count: ULONG;
    let mut P1: PWCHAR;
    let mut P2: PWCHAR;

    /* First check if the string sizes match */
    if (*String1).Length != (*String2).Length {
        /* They don't, return positive if the first is longer, negative otherwise */
        return (*String1).Length as LONG - (*String2).Length as LONG;
    }

    /* Now loop all characters */
    Count = (*String1).Length as ULONG / size_of::<u16>() as ULONG;
    P1 = (*String1).Buffer;
    P2 = (*String2).Buffer;
    while Count != 0 {
        /* Check if they don't match */
        if *P1 != *P2 {
            /* Return positive if the first char is greater, negative otherwise */
            return *P1 as LONG - *P2 as LONG;
        }

        /* Go to the next buffer position */
        P1 = P1.add(1);
        P2 = P2.add(1);
        Count -= 1;
    }

    /* All characters matched, return 0 */
    0
}

#[no_mangle]
pub unsafe extern "system" fn NpInitializeAliases() -> NTSTATUS {
    let mut QueryTable: [RTL_QUERY_REGISTRY_TABLE; 2] = zeroed();
    let mut Context: NPFS_QUERY_VALUE_CONTEXT = zeroed();
    let mut Status: NTSTATUS;
    let mut Length: USHORT;
    let mut i: ULONG;
    let mut CurrentAlias: PNPFS_ALIAS;
    let mut AliasPointer: *mut PNPFS_ALIAS;

    // L"Npfs\\Aliases"
    let aliases_path: [u16; 13] = wsz(b"Npfs\\Aliases");

    /* Initialize the query table */
    QueryTable[0].QueryRoutine =
        transmute::<NpReadAliasFn, PRTL_QUERY_REGISTRY_ROUTINE>(NpReadAlias);
    QueryTable[0].Flags = RTL_QUERY_REGISTRY_NOEXPAND;
    QueryTable[0].Name = null_mut();
    QueryTable[0].EntryContext = null_mut();
    QueryTable[0].DefaultType = REG_NONE;
    QueryTable[0].DefaultData = null_mut();
    QueryTable[0].DefaultLength = 0;
    QueryTable[1].QueryRoutine = None;
    QueryTable[1].Flags = 0;
    QueryTable[1].Name = null_mut();

    /* Setup the query context */
    Context.SizeOnly = 1;
    Context.FullSize = 0;
    Context.NumberOfAliases = 0;
    Context.NumberOfEntries = 0;

    /* Query the registry values (calculate length only) */
    Status = RtlQueryRegistryValues(
        RTL_REGISTRY_SERVICES | RTL_REGISTRY_OPTIONAL,
        aliases_path.as_ptr(),
        QueryTable.as_mut_ptr(),
        addr_of_mut!(Context) as PVOID,
        null_mut(),
    );
    if !NT_SUCCESS(Status) {
        if Status == STATUS_OBJECT_NAME_NOT_FOUND {
            return STATUS_SUCCESS;
        }
        return Status;
    }

    /* Check if there is anything */
    if Context.FullSize == 0 {
        /* Nothing to do, return success */
        return STATUS_SUCCESS;
    }

    /* Allocate a structure large enough to hold all the data */
    NpAliases = ExAllocatePoolWithTag(NonPagedPool, Context.FullSize, NPFS_CLIENT_SEC_CTX_TAG); // 'sfpN'
    if NpAliases.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    /* Now setup the actual pointers in the context */
    Context.CurrentTargetName = NpAliases as *mut UNICODE_STRING;
    CurrentAlias =
        Context.CurrentTargetName.add(Context.NumberOfEntries as usize) as PNPFS_ALIAS;
    Context.CurrentAlias = CurrentAlias;
    Context.CurrentStringPointer =
        CurrentAlias.add(Context.NumberOfAliases as usize) as *mut u16;

    /* This time query the real data */
    Context.SizeOnly = FALSE;
    Status = RtlQueryRegistryValues(
        RTL_REGISTRY_SERVICES | RTL_REGISTRY_OPTIONAL,
        aliases_path.as_ptr(),
        QueryTable.as_mut_ptr(),
        addr_of_mut!(Context) as PVOID,
        null_mut(),
    );
    if !NT_SUCCESS(Status) {
        ExFreePoolWithTag(NpAliases, 0);
        NpAliases = null_mut();
        return Status;
    }

    /* Make sure we didn't go past the end of the allocation! */
    // NT_ASSERT((PUCHAR)Context.CurrentStringPointer <= ((PUCHAR)NpAliases + Context.FullSize));
    // (debug-only assert; no-op in this build)

    /* Loop all aliases we got */
    i = 0;
    while i < Context.NumberOfAliases {
        /* Get the length and check what list to use */
        Length = (*CurrentAlias).Name.Length;
        if (Length as usize >= MIN_INDEXED_LENGTH * size_of::<u16>())
            && (Length as usize <= MAX_INDEXED_LENGTH * size_of::<u16>())
        {
            /* For this length range, we use an indexed list */
            AliasPointer = addr_of_mut!(
                NpAliasListByLength[(Length as usize / size_of::<u16>()) - 5]
            );
        } else {
            /* Length is outside of the range, use the default list */
            AliasPointer = addr_of_mut!(NpAliasList);
        }

        /* Loop through all aliases already in the list until we find one that
           is greater than our current alias */
        while !(*AliasPointer).is_null()
            && NpCompareAliasNames(
                addr_of!((*CurrentAlias).Name),
                addr_of!((**AliasPointer).Name),
            ) > 0
        {
            /* Go to the next alias */
            AliasPointer = addr_of_mut!((**AliasPointer).Next);
        }

        /* Insert the alias in the list */
        (*CurrentAlias).Next = *AliasPointer;
        *AliasPointer = CurrentAlias;

        /* Go to the next alias in the array */
        CurrentAlias = CurrentAlias.add(1);

        i += 1;
    }

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn NpFsdDirectoryControl(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    // TRACE("Entered\n"); UNIMPLEMENTED;

    (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_NOT_IMPLEMENTED;
    (*Irp).IoStatus.Information = 0;

    IoCompleteRequest(Irp, IO_NO_INCREMENT);
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn DriverEntry(
    DriverObject: PDRIVER_OBJECT,
    RegistryPath: PUNICODE_STRING,
) -> NTSTATUS {
    let mut DeviceObject: PDEVICE_OBJECT = null_mut();
    let mut DeviceName: UNICODE_STRING = zeroed();
    let mut Status: NTSTATUS;
    let _ = RegistryPath; // UNREFERENCED_PARAMETER(RegistryPath)

    // DPRINT("Next-Generation NPFS-Advanced\n");

    Status = NpInitializeAliases();
    if !NT_SUCCESS(Status) {
        // DPRINT1("Failed to initialize aliases!\n");
        return Status;
    }

    (*DriverObject).MajorFunction[IRP_MJ_CREATE] =
        transmute::<DispatchFn, PDRIVER_DISPATCH>(NpFsdCreate);
    (*DriverObject).MajorFunction[IRP_MJ_CREATE_NAMED_PIPE] =
        transmute::<DispatchFn, PDRIVER_DISPATCH>(NpFsdCreateNamedPipe);
    (*DriverObject).MajorFunction[IRP_MJ_CLOSE] =
        transmute::<DispatchFn, PDRIVER_DISPATCH>(NpFsdClose);
    (*DriverObject).MajorFunction[IRP_MJ_READ] =
        transmute::<DispatchFn, PDRIVER_DISPATCH>(NpFsdRead);
    (*DriverObject).MajorFunction[IRP_MJ_WRITE] =
        transmute::<DispatchFn, PDRIVER_DISPATCH>(NpFsdWrite);
    (*DriverObject).MajorFunction[IRP_MJ_QUERY_INFORMATION] =
        transmute::<DispatchFn, PDRIVER_DISPATCH>(NpFsdQueryInformation);
    (*DriverObject).MajorFunction[IRP_MJ_SET_INFORMATION] =
        transmute::<DispatchFn, PDRIVER_DISPATCH>(NpFsdSetInformation);
    (*DriverObject).MajorFunction[IRP_MJ_QUERY_VOLUME_INFORMATION] =
        transmute::<DispatchFn, PDRIVER_DISPATCH>(NpFsdQueryVolumeInformation);
    (*DriverObject).MajorFunction[IRP_MJ_CLEANUP] =
        transmute::<DispatchFn, PDRIVER_DISPATCH>(NpFsdCleanup);
    (*DriverObject).MajorFunction[IRP_MJ_FLUSH_BUFFERS] =
        transmute::<DispatchFn, PDRIVER_DISPATCH>(NpFsdFlushBuffers);
    (*DriverObject).MajorFunction[IRP_MJ_DIRECTORY_CONTROL] =
        transmute::<DispatchFn, PDRIVER_DISPATCH>(NpFsdDirectoryControl);
    (*DriverObject).MajorFunction[IRP_MJ_FILE_SYSTEM_CONTROL] =
        transmute::<DispatchFn, PDRIVER_DISPATCH>(NpFsdFileSystemControl);
    (*DriverObject).MajorFunction[IRP_MJ_QUERY_SECURITY] =
        transmute::<DispatchFn, PDRIVER_DISPATCH>(NpFsdQuerySecurityInfo);
    (*DriverObject).MajorFunction[IRP_MJ_SET_SECURITY] =
        transmute::<DispatchFn, PDRIVER_DISPATCH>(NpFsdSetSecurityInfo);

    (*DriverObject).DriverUnload = None;

    /* Build the FAST_IO_DISPATCH = { sizeof, NULL, NpFastRead, NpFastWrite } */
    NpFastIoDispatch.SizeOfFastIoDispatch = size_of::<FAST_IO_DISPATCH>() as ULONG;
    NpFastIoDispatch.FastIoCheckIfPossible = None;
    // Transmute straight to the bindgen PFAST_IO_* (Option) slot — it carries the correct
    // per-arch ABI (stdcall on i386, C on amd64/arm64), unlike a hand-written raw "C" type.
    NpFastIoDispatch.FastIoRead = transmute::<FastIoReadFn, PFAST_IO_READ>(NpFastRead);
    NpFastIoDispatch.FastIoWrite = transmute::<FastIoWriteFn, PFAST_IO_WRITE>(NpFastWrite);
    (*DriverObject).FastIoDispatch = addr_of_mut!(NpFastIoDispatch);

    RtlInitUnicodeString(addr_of_mut!(DeviceName), NAMEDPIPE_DEVICE_NAME.as_ptr());
    Status = IoCreateDevice(
        DriverObject,
        size_of::<NP_VCB>() as ULONG,
        addr_of_mut!(DeviceName),
        FILE_DEVICE_NAMED_PIPE,
        0,
        FALSE as u8,
        addr_of_mut!(DeviceObject),
    );
    if !NT_SUCCESS(Status) {
        // DPRINT1("Failed to create named pipe device! (Status %lx)\n", Status);
        return Status;
    }

    /* Initialize the device object */
    NpfsDeviceObject = DeviceObject;
    (*DeviceObject).Flags |= DO_LONG_TERM_REQUESTS;

    /* Initialize the Volume Control Block (VCB) */
    NpVcb = (*DeviceObject).DeviceExtension as PNP_VCB;
    NpInitializeVcb();
    Status = NpCreateRootDcb();
    // ASSERT(Status == STATUS_SUCCESS);
    let _ = Status;
    STATUS_SUCCESS
}

// ---- callback-slot ABI bridges (contract §9) -------------------------------

/// NTAPI dispatch fn item ABI → the bindgen PDRIVER_DISPATCH (Option<extern "C" fn>) slot type.
type DispatchFn = unsafe extern "system" fn(PDEVICE_OBJECT, PIRP) -> NTSTATUS;

/// NpReadAlias's fn-item type → the bindgen PRTL_QUERY_REGISTRY_ROUTINE slot type.
type NpReadAliasFn =
    unsafe extern "system" fn(PWSTR, ULONG, PVOID, ULONG, PVOID, PVOID) -> NTSTATUS;

/// NpFastRead / NpFastWrite fn-item types and the bindgen Option<extern "C" fn> the inner of
/// PFAST_IO_READ / PFAST_IO_WRITE expects. We transmute the fn item to the raw extern "C" fn.
type FastIoReadFn = unsafe extern "system" fn(
    PFILE_OBJECT,
    PLARGE_INTEGER,
    ULONG,
    BOOLEAN,
    ULONG,
    PVOID,
    PIO_STATUS_BLOCK,
    PDEVICE_OBJECT,
) -> BOOLEAN;
type FastIoReadRaw = unsafe extern "C" fn(
    *mut _FILE_OBJECT,
    PLARGE_INTEGER,
    ULONG,
    BOOLEAN,
    ULONG,
    PVOID,
    PIO_STATUS_BLOCK,
    *mut _DEVICE_OBJECT,
) -> BOOLEAN;
type FastIoWriteFn = FastIoReadFn;
type FastIoWriteRaw = FastIoReadRaw;
