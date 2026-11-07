#[allow(unused_imports)]
use ntkrnl_sys::ddk::{PWCH, _KSE_DRIVER_IO_CALLBACKS, PDRIVER_INITIALIZE, PDRIVER_STARTIO, PDRIVER_UNLOAD, PDRIVER_ADD_DEVICE, PDRIVER_DISPATCH, PEXTENDED_DRIVER_EXTENSION, PKSE_GET_IO_CALLBACKS, PKSE_SET_COMPLETION_HOOK, _IMAGE_THUNK_DATA64, KPROCESSOR_MODE, ObfReferenceObject};
use super::ksedb::{KseDbGetShimsForDriver, KseDbInitialize};
use super::driverscope::KseDriverScopeInitialize;
use super::ignorepooltag::KseIgnoreFreePoolTagInitialize;
use super::versionlie::KseVersionLieInitialize;
#[allow(unused_imports)]
use ntkrnl_sys::ddk::{_KSE_CALLBACK_ROUTINES, GUID, _LIST_ENTRY, PLIST_ENTRY, KSE_CALLBACK_ROUTINES};
use super::defs::*;
use ntkrnl_sys::driver_ddk::DbgPrint;
use crate::shim_exports::CONTAINING_RECORD;
use crate::ke::ke_defs::HANDLE;
use crate::ke::bug::KiPcToFileHeader;
use crate::ARM3::drvmgmt::MmIsVerifierEnabled;
use crate::ARM3::mdlsup::MmMapLockedPagesSpecifyCache;
use crate::ARM3::mdlsup::MmUnlockPages;
use crate::ARM3::mdlsup::MmUnmapLockedPages;
use crate::ARM3::sysldr::PsLoadedModuleList;
use crate::common_inlines::RtlInitEmptyUnicodeString;
use ntkrnl_sys::ddk::_UNICODE_STRING;
// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
pub static RUSTCONV_WSTR_01D48CD1C188: [u16; 8] = [0x0068u16, 0x0061, 0x006C, 0x002E, 0x0064, 0x006C, 0x006C, 0x0000];
pub static RUSTCONV_WSTR_3BBCA6ED8518: [u16; 13] = [
    0x006Eu16, 0x0074, 0x006F, 0x0073, 0x006B, 0x0072, 0x006E, 0x006C, 0x002E, 0x0065,
    0x0078, 0x0065, 0x0000,
];
pub static RUSTCONV_WSTR_F4B0F3A9DD9A: [u16; 6] = [0x0053u16, 0x0068, 0x0069, 0x006D, 0x0073, 0x0000];
pub static RUSTCONV_WSTR_FB538891C21B: [u16; 73] = [
    0x005Cu16, 0x0052, 0x0065, 0x0067, 0x0069, 0x0073, 0x0074, 0x0072, 0x0079, 0x005C,
    0x004D, 0x0061, 0x0063, 0x0068, 0x0069, 0x006E, 0x0065, 0x005C, 0x0053, 0x0079,
    0x0073, 0x0074, 0x0065, 0x006D, 0x005C, 0x0043, 0x0075, 0x0072, 0x0072, 0x0065,
    0x006E, 0x0074, 0x0043, 0x006F, 0x006E, 0x0074, 0x0072, 0x006F, 0x006C, 0x0053,
    0x0065, 0x0074, 0x005C, 0x0043, 0x006F, 0x006E, 0x0074, 0x0072, 0x006F, 0x006C,
    0x005C, 0x0043, 0x006F, 0x006D, 0x0070, 0x0061, 0x0074, 0x0069, 0x0062, 0x0069,
    0x006C, 0x0069, 0x0074, 0x0079, 0x005C, 0x0044, 0x0072, 0x0069, 0x0076, 0x0065,
    0x0072, 0x005C, 0x0000,
];

use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::ex::*;
use ntkrnl_sys::inline::tier1::*;
use ntkrnl_sys::interlocked::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ddk::{IoAllocateMdl, IoFreeMdl, IoSetCompletionRoutineEx, KSE_DRIVER_IO_CALLBACKS, LDR_DATA_TABLE_ENTRY, LOADER_PARAMETER_EXTENSION, _OBJECT_ATTRIBUTES, PCSTR, PCUNICODE_STRING, PDEVICE_OBJECT, PDRIVER_OBJECT, PIMAGE_IMPORT_BY_NAME, PIMAGE_IMPORT_DESCRIPTOR, PIMAGE_NT_HEADERS, PIMAGE_THUNK_DATA, PIO_COMPLETION_ROUTINE, PIRP, PKEY_VALUE_FULL_INFORMATION, PKSE_DRIVER_IO_CALLBACKS, PKSE_HOOK, PKSE_HOOK_COLLECTION, PKSE_SHIM, PLDR_DATA_TABLE_ENTRY, PLOADER_PARAMETER_BLOCK, PLOADER_PARAMETER_EXTENSION, PMDL, PUNICODE_STRING, RtlAppendUnicodeStringToString, RtlAppendUnicodeToString, RtlEqualUnicodeString, RtlFindExportedRoutineByName, RtlImageDirectoryEntryToData, RtlImageNtHeader, RtlInitUnicodeString, RtlUpcaseUnicodeChar, ZwClose, ZwOpenKey, ZwQueryValueKey};
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
pub static RUSTCONV_C2R_WSTR_F4B0F3A9DD9A: [u16; 6] = [0x0053u16, 0x0068, 0x0069, 0x006D, 0x0073, 0x0000];

static mut ValueName: _UNICODE_STRING = _UNICODE_STRING {
        Length: (::core::mem::size_of::<[::core::ffi::c_ushort; 6]>() as usize)
            .wrapping_sub(::core::mem::size_of::<::core::ffi::c_ushort>() as usize)
            as USHORT,
        MaximumLength: (::core::mem::size_of::<[::core::ffi::c_ushort; 6]>() as usize)
            .wrapping_div(::core::mem::size_of::<::core::ffi::c_char>() as usize)
            as USHORT,
        Buffer: RUSTCONV_C2R_WSTR_F4B0F3A9DD9A.as_ptr() as PWCH,
    };

static mut KseEngine: KSE_ENGINE = _KSE_ENGINE {
    DisableFlags: KSE_DISABLE_NONE,
    State: KSE_STATE_UNINITIALIZED,
    Flags: KSE_FLAGS_NONE,
    ProvidersListHead: _LIST_ENTRY {
        Flink: ::core::ptr::null::<_LIST_ENTRY>() as *mut _LIST_ENTRY,
        Blink: ::core::ptr::null::<_LIST_ENTRY>() as *mut _LIST_ENTRY,
    },
    ShimmedDriversListHead: _LIST_ENTRY {
        Flink: ::core::ptr::null::<_LIST_ENTRY>() as *mut _LIST_ENTRY,
        Blink: ::core::ptr::null::<_LIST_ENTRY>() as *mut _LIST_ENTRY,
    },
    KseCallbackRoutines: _KSE_CALLBACK_ROUTINES {
        KseGetIoCallbacksRoutine: None,
        KseSetCompletionHookRoutine: None,
    },
    DeviceInfoCache: ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void,
    HardwareIdCache: ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void,
    ShimmedDriverHint: ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void,
};

pub const KSE_MAX_SHIMS_PER_DRIVER: i32 = 10;

#[repr(C)]
pub struct _KSE_PROVIDER {
    pub ProviderList: _LIST_ENTRY,
    pub Shim: PKSE_SHIM,
}
pub type KSE_PROVIDER = _KSE_PROVIDER;
pub type PKSE_PROVIDER = *mut _KSE_PROVIDER;

pub type KSE_DISABLE_FLAGS = i32;
pub const KSE_DISABLE_NONE: i32 = 0;
pub const KSE_DISABLE_DRIVER_SHIMS: i32 = 1 << 0;
pub const KSE_DISABLE_DEVICE_SHIMS: i32 = 1 << 1;

pub type KSE_ENGINE_STATE = i32;
pub const KSE_STATE_UNINITIALIZED: i32 = 0;
pub const KSE_STATE_INITIALIZING: i32 = 1;
pub const KSE_STATE_INITIALIZED: i32 = 2;

pub type KSE_ENGINE_FLAGS = i32;
pub const KSE_FLAGS_NONE: i32 = 0;
pub const KSE_FLAGS_GROUPPOLICY_OK: i32 = 1 << 1;
pub const KSE_FLAGS_DRV_SHIMS_ACTIVE: i32 = 1 << 11;
pub const KSE_FLAGS_DEV_SHIMS_ACTIVE: i32 = 1 << 12;

const _: () = assert!(KSE_FLAGS_GROUPPOLICY_OK == 2);
const _: () = assert!(KSE_FLAGS_DRV_SHIMS_ACTIVE == 0x800);
const _: () = assert!(KSE_FLAGS_DEV_SHIMS_ACTIVE == 0x1000);

#[repr(C)]
pub struct _KSE_ENGINE {
    pub DisableFlags: KSE_DISABLE_FLAGS,
    pub State: KSE_ENGINE_STATE,
    pub Flags: KSE_ENGINE_FLAGS,
    pub ProvidersListHead: _LIST_ENTRY,
    pub ShimmedDriversListHead: _LIST_ENTRY,
    pub KseCallbackRoutines: KSE_CALLBACK_ROUTINES,
    pub DeviceInfoCache: PVOID,
    pub HardwareIdCache: PVOID,
    pub ShimmedDriverHint: PVOID,
}
pub type KSE_ENGINE = _KSE_ENGINE;
pub type PKSE_ENGINE = *mut _KSE_ENGINE;
#[no_mangle]
pub unsafe extern "system" fn KsepPoolAllocateNonPaged(NumberOfBytes: SIZE_T) -> PVOID {
    let Ptr: PVOID = ExAllocatePoolWithTag(NonPagedPool, NumberOfBytes, TAG_KSE_CALLBACKS);
    if !Ptr.is_null() {
        return Ptr;
    }

    DbgPrint(
        b"KSE: Unable to allocated %u bytes\n\0".as_ptr() as PCSTR,
        NumberOfBytes,
    );
    core::ptr::null_mut()
}
#[no_mangle]
pub unsafe extern "system" fn KsepFindShim(Name: PCUNICODE_STRING) -> PKSE_SHIM {
    let mut ProviderEntry: PKSE_PROVIDER;
    let mut ListEntry: PLIST_ENTRY;
    let mut ShimName: _UNICODE_STRING = core::mem::zeroed();
    ListEntry = KseEngine.ProvidersListHead.Flink;
    while ListEntry != core::ptr::addr_of_mut!(KseEngine.ProvidersListHead) {
        ProviderEntry = ((ListEntry as ULONG_PTR).wrapping_sub(
            core::mem::offset_of!(KSE_PROVIDER, ProviderList) as ULONG_PTR,
        )) as PKSE_PROVIDER;
        if (*(*ProviderEntry).Shim).ShimName.is_null() {
            ListEntry = (*ListEntry).Flink;
            continue;
        }

        RtlInitUnicodeString(
            (core::ptr::addr_of_mut!(ShimName)).cast(),
            (*(*ProviderEntry).Shim).ShimName as PCWSTR,
        );
        if RtlEqualUnicodeString(
            core::ptr::addr_of_mut!(ShimName),
            Name,
            TRUE as BOOLEAN,
        ) != 0
        {
            return (*ProviderEntry).Shim;
        }
        ListEntry = (*ListEntry).Flink;
    }

    core::ptr::null_mut()
}
#[no_mangle]
pub unsafe extern "system" fn KsepAddShimByName(
    Name: PCUNICODE_STRING,
    Shims: *mut PKSE_SHIM,
    NumberOfShims: PULONG,
) {
    let mut Shim: PKSE_SHIM;
    let mut n: ULONG;

    Shim = KsepFindShim(Name);
    if Shim.is_null() {
        DPRINT1!(b"KSE: Shim %wZ not found\n\0".as_ptr() as PCSTR, Name);
        return;
    }

    n = 0;
    while n < *NumberOfShims {
        if *Shims.offset(n as isize) == Shim {
            return;
        }
        n = n.wrapping_add(1);
    }

    if *NumberOfShims >= KSE_MAX_SHIMS_PER_DRIVER as ULONG {
        DPRINT1!(b"KSE: Too many shims, ignoring %wZ\n\0".as_ptr() as PCSTR, Name);
        return;
    }

    *Shims.offset(*NumberOfShims as isize) = Shim;
    *NumberOfShims = (*NumberOfShims).wrapping_add(1);
}
unsafe extern "system" fn KsepMultiSzToShims(
    KeyValueInformation: PKEY_VALUE_FULL_INFORMATION,
    Shims: *mut PKSE_SHIM,
    NumberOfShims: PULONG,
) {
    let mut Current: PWCHAR;
    let mut End: PWCHAR;
    let mut ShimName: _UNICODE_STRING = core::mem::zeroed();
    let mut Length: SIZE_T;

    Current = ((KeyValueInformation as ULONG_PTR).wrapping_add((*KeyValueInformation).DataOffset as ULONG_PTR)) as PWCHAR;
    End = ((Current as ULONG_PTR).wrapping_add((*KeyValueInformation).DataLength as ULONG_PTR)) as PWCHAR;

    while Current < End && *Current != 0 {
        Length = 0 as SIZE_T;
        while Current.offset(Length as isize) < End && *Current.offset(Length as isize) != 0 {
            Length += 1;
        }

        ShimName.Buffer = Current;
        ShimName.Length = (Length * core::mem::size_of::<WCHAR>() as SIZE_T) as USHORT;
        ShimName.MaximumLength = ShimName.Length;

        KsepAddShimByName(core::ptr::addr_of_mut!(ShimName) as PCUNICODE_STRING, Shims, NumberOfShims);

        Current = Current.offset((Length + 1 as SIZE_T) as isize);
    }
}
unsafe extern "system" fn KsepGetShimsFromRegistry(
    LdrEntry: PLDR_DATA_TABLE_ENTRY,
    Shims: *mut PKSE_SHIM,
    NumberOfShims: PULONG,
) -> NTSTATUS {
    let mut KeyNameBuffer: [WCHAR; 256] = [0; 256];
    let mut KeyName: _UNICODE_STRING = core::mem::zeroed();
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut KeyValueInformation: PKEY_VALUE_FULL_INFORMATION = core::ptr::null_mut();
    let mut KeyValueSize: ULONG = 0x800 as ULONG;
    let mut KeyHandle: HANDLE = core::ptr::null_mut();
    let mut Status: NTSTATUS;

    RtlInitEmptyUnicodeString(
        &mut KeyName,
        KeyNameBuffer.as_mut_ptr(),
        core::mem::size_of_val(&KeyNameBuffer) as USHORT,
    );
    Status = RtlAppendUnicodeToString(&mut KeyName, RUSTCONV_WSTR_FB538891C21B.as_ptr());
    if NT_SUCCESS(Status) {
        Status = RtlAppendUnicodeStringToString(&mut KeyName, core::ptr::addr_of_mut!((*LdrEntry).BaseDllName));
    }
    if !NT_SUCCESS(Status) {
        return Status;
    }

    ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
    ObjectAttributes.RootDirectory = core::ptr::null_mut();
    ObjectAttributes.ObjectName = &mut KeyName;
    ObjectAttributes.Attributes = (OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE) as _;
    ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
    ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

    Status = ZwOpenKey(&mut KeyHandle, KEY_READ as _, &mut ObjectAttributes);
    if !NT_SUCCESS(Status) {
        return Status;
    }

    KeyValueInformation = ExAllocatePoolWithTag(
        PagedPool,
        KeyValueSize as SIZE_T,
        TAG_KSE_REGISTRY as ULONG,
    ) as PKEY_VALUE_FULL_INFORMATION;
    if KeyValueInformation.is_null() {
        ZwClose(KeyHandle);
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    Status = ZwQueryValueKey(
        KeyHandle,
        core::ptr::addr_of!(ValueName) as PUNICODE_STRING,
        KeyValueFullInformation as _,
        KeyValueInformation as PVOID,
        KeyValueSize,
        &mut KeyValueSize,
    );
    ZwClose(KeyHandle);

    if NT_SUCCESS(Status) {
        if (*KeyValueInformation).Type == REG_MULTI_SZ as ULONG
            && (*KeyValueInformation).DataLength != 0
        {
            KsepMultiSzToShims(KeyValueInformation, Shims, NumberOfShims);
        } else {
            Status = STATUS_OBJECT_TYPE_MISMATCH;
        }
    }

    ExFreePoolWithTag(KeyValueInformation as PVOID, TAG_KSE_REGISTRY as ULONG);
    return Status;
}
unsafe extern "system" fn KsepGetShimsForDriver(
    LdrEntry: PLDR_DATA_TABLE_ENTRY,
    Shims: *mut PKSE_SHIM,
    NumberOfShims: PULONG,
) -> NTSTATUS {
    *NumberOfShims = 0;

    KseDbGetShimsForDriver(LdrEntry, Shims, NumberOfShims);
    KsepGetShimsFromRegistry(LdrEntry, Shims, NumberOfShims);

    if *NumberOfShims != 0 {
        STATUS_SUCCESS
    } else {
        STATUS_NOT_FOUND
    }
}
unsafe extern "system" fn KsepGetShimCallbacksForDriver(
    LdrEntry: PLDR_DATA_TABLE_ENTRY,
    pIoCallbacks: PKSE_DRIVER_IO_CALLBACKS,
) -> NTSTATUS {
    let mut Shims: [PKSE_SHIM; (KSE_MAX_SHIMS_PER_DRIVER as _)] = [core::ptr::null_mut(); (KSE_MAX_SHIMS_PER_DRIVER as _)];
    let mut NumberOfShims: ULONG = 0;
    let mut n: ULONG;
    let mut HookCollection: PKSE_HOOK_COLLECTION;
    let mut Hook: PKSE_HOOK;
    let mut Status: NTSTATUS;

    core::ptr::write_bytes(
        pIoCallbacks as *mut u8,
        0,
        core::mem::size_of::<_KSE_DRIVER_IO_CALLBACKS>(),
    );

    Status = KsepGetShimsForDriver(LdrEntry, Shims.as_mut_ptr(), &mut NumberOfShims);
    if !NT_SUCCESS(Status) {
        DPRINT!(
            b"KSE: No Shim found for %wZ\n\0".as_ptr() as PCHAR,
            &mut (*LdrEntry).BaseDllName,
        );
        return Status;
    }

    n = 0;
    while n < NumberOfShims {
        HookCollection = (*Shims[n as usize]).HookCollectionsArray;
        while (*HookCollection).Type != KseCollectionInvalid as ULONG {
            if (*HookCollection).Type != KseCollectionCallback as ULONG {
                HookCollection = HookCollection.add(1);
                continue;
            }

            Hook = (*HookCollection).HookArray;
            while (*Hook).Type != KseHookInvalid as ULONG {
                if (*Hook).Type != KseHookIRPCallback as ULONG {
                    Hook = Hook.add(1);
                    continue;
                }

                if (*Hook).__bindgen_anon_1.CallbackId == KseHookCallbackDriverInit as ULONG {
                    (*pIoCallbacks).DriverInit =
                        core::mem::transmute::<PVOID, PDRIVER_INITIALIZE>((*Hook).HookFunction);
                } else if (*Hook).__bindgen_anon_1.CallbackId
                    == KseHookCallbackDriverStartIo as ULONG
                {
                    (*pIoCallbacks).DriverStartIo =
                        core::mem::transmute::<PVOID, PDRIVER_STARTIO>((*Hook).HookFunction);
                } else if (*Hook).__bindgen_anon_1.CallbackId
                    == KseHookCallbackDriverUnload as ULONG
                {
                    (*pIoCallbacks).DriverUnload =
                        core::mem::transmute::<PVOID, PDRIVER_UNLOAD>((*Hook).HookFunction);
                } else if (*Hook).__bindgen_anon_1.CallbackId == KseHookCallbackAddDevice as ULONG {
                    (*pIoCallbacks).AddDevice =
                        core::mem::transmute::<PVOID, PDRIVER_ADD_DEVICE>((*Hook).HookFunction);
                } else if (*Hook).__bindgen_anon_1.CallbackId >= KseHookCallbackMajorFunction as ULONG
                    && (*Hook).__bindgen_anon_1.CallbackId
                        <= (KseHookCallbackMajorFunction as ULONG)
                            .wrapping_add(IRP_MJ_MAXIMUM_FUNCTION as ULONG)
                {
                    (*pIoCallbacks).MajorFunction[(*Hook)
                        .__bindgen_anon_1
                        .CallbackId
                        .wrapping_sub(KseHookCallbackMajorFunction as ULONG)
                        as usize] =
                        core::mem::transmute::<PVOID, PDRIVER_DISPATCH>((*Hook).HookFunction);
                }

                Hook = Hook.add(1);
            }

            HookCollection = HookCollection.add(1);
        }
        n = n.wrapping_add(1);
    }
    return STATUS_SUCCESS;
}
#[no_mangle]
pub unsafe extern "system" fn KseShimDriverIoCallbacks(DriverObject: PDRIVER_OBJECT) -> NTSTATUS {
    let mut IoCallbacks: KSE_DRIVER_IO_CALLBACKS = core::mem::zeroed();
    let mut KseCallbacks: PKSE_DRIVER_IO_CALLBACKS = core::ptr::null_mut();
    let mut LdrEntry: PLDR_DATA_TABLE_ENTRY = core::ptr::null_mut();
    let mut InKernel: BOOLEAN = 0;
    let mut i: ULONG;
    let mut Status: NTSTATUS;

    if KseEngine.State as ::core::ffi::c_uint != (KSE_STATE_INITIALIZED) as ::core::ffi::c_uint {
        return STATUS_SUCCESS;
    }

    if KiPcToFileHeader(
        (*DriverObject).DriverStart,
        &raw mut LdrEntry,
        FALSE as BOOLEAN,
        &raw mut InKernel,
    )
    .is_null()
    {
        return STATUS_NOT_FOUND;
    }

    Status = KsepGetShimCallbacksForDriver(LdrEntry, &raw mut IoCallbacks);
    if !NT_SUCCESS(Status) {
        return Status;
    }

    KseCallbacks = KsepPoolAllocateNonPaged(::core::mem::size_of::<KSE_DRIVER_IO_CALLBACKS>() as SIZE_T)
        as PKSE_DRIVER_IO_CALLBACKS;
    if KseCallbacks.is_null() {
        return STATUS_NO_MEMORY;
    }

    core::ptr::write_bytes(
        KseCallbacks as *mut u8,
        0,
        ::core::mem::size_of::<KSE_DRIVER_IO_CALLBACKS>(),
    );

    if (*DriverObject).DriverInit.is_some() && IoCallbacks.DriverInit.is_some() {
        (*KseCallbacks).DriverInit = (*DriverObject).DriverInit;
        (*DriverObject).DriverInit = IoCallbacks.DriverInit;
    }
    if (*DriverObject).DriverStartIo.is_some() && IoCallbacks.DriverStartIo.is_some() {
        (*KseCallbacks).DriverStartIo = (*DriverObject).DriverStartIo;
        (*DriverObject).DriverStartIo = IoCallbacks.DriverStartIo;
    }
    if (*DriverObject).DriverUnload.is_some() && IoCallbacks.DriverUnload.is_some() {
        (*KseCallbacks).DriverUnload = (*DriverObject).DriverUnload;
        (*DriverObject).DriverUnload = IoCallbacks.DriverUnload;
    }
    if (*(*DriverObject).DriverExtension).AddDevice.is_some() && IoCallbacks.AddDevice.is_some() {
        (*KseCallbacks).AddDevice = (*(*DriverObject).DriverExtension).AddDevice;
        (*(*DriverObject).DriverExtension).AddDevice = IoCallbacks.AddDevice;
    }

    i = 0 as ULONG;
    while i <= IRP_MJ_MAXIMUM_FUNCTION as ULONG {
        if (*DriverObject).MajorFunction[i as usize].is_some()
            && IoCallbacks.MajorFunction[i as usize].is_some()
        {
            (*KseCallbacks).MajorFunction[i as usize] = (*DriverObject).MajorFunction[i as usize];
            (*DriverObject).MajorFunction[i as usize] = IoCallbacks.MajorFunction[i as usize];
        }
        i = i.wrapping_add(1);
    }

    (*IoGetDrvObjExtension(DriverObject)).KseCallbacks = KseCallbacks as PVOID;

    return STATUS_SUCCESS;
}
unsafe extern "system" fn KseShimDatabaseBootInitialize(LoaderExtension: PLOADER_PARAMETER_EXTENSION) -> NTSTATUS {
    let mut Status: NTSTATUS;

    Status = KseDbInitialize((*LoaderExtension).DrvDBImage, (*LoaderExtension).DrvDBSize);
    if !NT_SUCCESS(Status) {
        DPRINT1!(
            b"KSE: Unable to load the driver database (0x%08lx)\n\0".as_ptr() as PCSTR,
            Status,
        );
    }

    return STATUS_SUCCESS;
}

unsafe extern "system" fn KsepMatchInitMachineInfo() -> NTSTATUS {
    STATUS_SUCCESS
}
unsafe extern "system" fn KseGetIoCallbacks(
    DriverObject: PDRIVER_OBJECT,
) -> PKSE_DRIVER_IO_CALLBACKS {
    (*(IoGetDrvObjExtension(DriverObject) as PEXTENDED_DRIVER_EXTENSION)).KseCallbacks as PKSE_DRIVER_IO_CALLBACKS
}
unsafe extern "system" fn KseSetCompletionHook(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
    CompletionRoutine: PIO_COMPLETION_ROUTINE,
    Context: PVOID,
) -> NTSTATUS {
    return IoSetCompletionRoutineEx(DeviceObject, Irp, CompletionRoutine, Context, TRUE, TRUE, TRUE);
}

#[no_mangle]
pub unsafe extern "system" fn KseInitialize(BootPhase: ULONG, LoaderBlock: PLOADER_PARAMETER_BLOCK) -> NTSTATUS {
    let mut LoaderExtension: PLOADER_PARAMETER_EXTENSION = core::ptr::null_mut();
    let mut Status: NTSTATUS = STATUS_SUCCESS;
    let mut Flags: ULONG = 0;

    if BootPhase == 0 {
        if InitSafeBootMode != 0 ||
            NT_SUCCESS(MmIsVerifierEnabled(&mut Flags))
        {
            DPRINT!("KSE: Not initializing (verifier or safe boot)\n");
            return STATUS_SUCCESS;
        }

        LoaderExtension = (*LoaderBlock).Extension;
        if LoaderExtension.is_null() ||
            ((*LoaderExtension).Size as ULONG) < (core::mem::offset_of!(LOADER_PARAMETER_EXTENSION, NetworkLoaderBlock) as ULONG) ||
            (*LoaderExtension).DrvDBImage.is_null()
        {
            DPRINT!("KSE: Not initializing (invalid DB)\n");
            return STATUS_SUCCESS;
        }

        KseEngine.State = KSE_STATE_INITIALIZED;
        InitializeListHead(&mut KseEngine.ProvidersListHead);
        InitializeListHead(&mut KseEngine.ShimmedDriversListHead);
        KseEngine.KseCallbackRoutines.KseGetIoCallbacksRoutine = Some(
            KseGetIoCallbacks as unsafe extern "system" fn(PDRIVER_OBJECT) -> PKSE_DRIVER_IO_CALLBACKS,
        ) as PKSE_GET_IO_CALLBACKS;
        KseEngine.KseCallbackRoutines.KseSetCompletionHookRoutine = Some(
            KseSetCompletionHook as unsafe extern "system" fn(
                PDEVICE_OBJECT,
                PIRP,
                PIO_COMPLETION_ROUTINE,
                PVOID,
            ) -> NTSTATUS,
        ) as PKSE_SET_COMPLETION_HOOK;
        Status = KseShimDatabaseBootInitialize(LoaderExtension);
        if NT_SUCCESS(Status) {
            Status = KsepMatchInitMachineInfo();
            if NT_SUCCESS(Status) {
                Status = KseDriverScopeInitialize();
            }
        }
        KseEngine.State = KSE_STATE_INITIALIZING;
    } else if BootPhase == 1 {
        if KseEngine.State != KSE_STATE_INITIALIZING {
            return STATUS_SUCCESS;
        }

        KseEngine.State = KSE_STATE_INITIALIZED;
        KseIgnoreFreePoolTagInitialize();
        KseVersionLieInitialize();
        Status = STATUS_SUCCESS;
    }

    DPRINT1!(b"KseInitialize(BootPhase %ld) => %08lx\n\0".as_ptr() as PCSTR, BootPhase, Status);
    return Status;
}

#[no_mangle]
pub unsafe extern "system" fn KseRegisterShim(Shim: PKSE_SHIM, Ignored: PVOID, Flags: ULONG) -> NTSTATUS {
    return KseRegisterShimEx(Shim, Ignored, Flags, core::ptr::null_mut());
}

#[no_mangle]
pub unsafe extern "system" fn KseUnregisterShim(
    Shim: PKSE_SHIM,
    Unknown1: PVOID,
    Unknown2: PVOID,
) -> NTSTATUS {
    // UNIMPLEMENTED;
    STATUS_NOT_SUPPORTED
}
unsafe extern "system" fn KsepIsImportFromModule(mut ImportName: PCSTR, mut ModuleName: PCWSTR) -> BOOLEAN {
    while *ImportName != 0 && *ModuleName != 0 {
        if RtlUpcaseUnicodeChar(*ImportName as UCHAR as WCHAR) != RtlUpcaseUnicodeChar(*ModuleName) {
            return FALSE;
        }
        ImportName = ImportName.offset(1);
        ModuleName = ModuleName.offset(1);
    }
    (*ImportName == ANSI_NULL && *ModuleName == UNICODE_NULL) as BOOLEAN
}
unsafe extern "system" fn KsepFindExportingModule(
    HookCollection: PKSE_HOOK_COLLECTION,
    ModuleName: *mut PCWSTR,
) -> PLDR_DATA_TABLE_ENTRY {
    let mut ListEntry: PLIST_ENTRY;
    let mut LdrEntry: PLDR_DATA_TABLE_ENTRY;
    let mut ExportDriverName: _UNICODE_STRING = core::mem::zeroed();

    match (*HookCollection).Type {
        0 => {
            *ModuleName = RUSTCONV_WSTR_3BBCA6ED8518.as_ptr();
            ListEntry = PsLoadedModuleList.Flink as PLIST_ENTRY;
            return ldr_from_load_order(ListEntry);
        }
        1 => {
            *ModuleName = RUSTCONV_WSTR_01D48CD1C188.as_ptr();
            ListEntry = (*PsLoadedModuleList.Flink).Flink as PLIST_ENTRY;
            return ldr_from_load_order(ListEntry);
        }
        2 => {
            if (*HookCollection).ExportDriverName.is_null() {
                return core::ptr::null_mut();
            }
            *ModuleName = (*HookCollection).ExportDriverName as PCWSTR;
            RtlInitUnicodeString(
                (&raw mut ExportDriverName).cast(),
                (*HookCollection).ExportDriverName as PCWSTR,
            );
            ListEntry = PsLoadedModuleList.Flink as PLIST_ENTRY;
            while ListEntry != (&raw mut PsLoadedModuleList) as PLIST_ENTRY {
                LdrEntry = ldr_from_load_order(ListEntry);
                if RtlEqualUnicodeString(
                    &raw mut (*LdrEntry).BaseDllName,
                    &raw mut ExportDriverName,
                    TRUE as BOOLEAN,
                ) != 0
                {
                    return LdrEntry;
                }
                ListEntry = (*ListEntry).Flink as PLIST_ENTRY;
            }
            return core::ptr::null_mut();
        }
        _ => return core::ptr::null_mut(),
    }
}
unsafe extern "system" fn KsepWriteImportEntry(
    Thunk: PIMAGE_THUNK_DATA,
    NewFunction: PVOID,
) -> NTSTATUS {
    let mut Mdl: PMDL;
    let mut MappedThunk: PIMAGE_THUNK_DATA;
    let mut Status: NTSTATUS = STATUS_SUCCESS;

    Mdl = IoAllocateMdl(
        Thunk as PVOID,
        ::core::mem::size_of::<_IMAGE_THUNK_DATA64>() as ULONG,
        FALSE,
        FALSE,
        core::ptr::null_mut(),
    );
    if Mdl.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    // C: MmProbeAndLockPages inside _SEH2_TRY, Status = the exception code. The
    // non-raising export returns that status instead.
    Status = ntkrnl_sys::probe::MmTryProbeAndLockPages(Mdl, (0i32) as KPROCESSOR_MODE, IoReadAccess);

    if !NT_SUCCESS(Status) {
        IoFreeMdl(Mdl);
        return Status;
    }

    MappedThunk = MmMapLockedPagesSpecifyCache(
        Mdl,
        (0i32) as KPROCESSOR_MODE,
        MmCached,
        core::ptr::null_mut(),
        (FALSE) as ULONG,
        NormalPagePriority,
    ) as PIMAGE_THUNK_DATA;
    if !MappedThunk.is_null() {
        InterlockedExchangePointer(
            core::ptr::addr_of_mut!((*MappedThunk).u1.Function) as *mut PVOID as *mut *mut ::core::ffi::c_void,
            NewFunction as *mut ::core::ffi::c_void,
        );
        MmUnmapLockedPages(MappedThunk as PVOID, Mdl);
    } else {
        Status = STATUS_INSUFFICIENT_RESOURCES;
    }

    MmUnlockPages(Mdl);
    IoFreeMdl(Mdl);
    return Status;
}
unsafe extern "system" fn KsepHookImports(
    LdrEntry: PLDR_DATA_TABLE_ENTRY,
    HookCollection: PKSE_HOOK_COLLECTION,
) {
    let DllBase: PUCHAR = (*LdrEntry).DllBase as PUCHAR;
    let mut ImportDescriptor: PIMAGE_IMPORT_DESCRIPTOR = core::ptr::null_mut();
    let mut OriginalThunk: PIMAGE_THUNK_DATA = core::ptr::null_mut();
    let mut FirstThunk: PIMAGE_THUNK_DATA = core::ptr::null_mut();
    let mut ImportByName: PIMAGE_IMPORT_BY_NAME = core::ptr::null_mut();
    let mut ExportEntry: PLDR_DATA_TABLE_ENTRY = core::ptr::null_mut();
    let mut ModuleName: PCWSTR = core::ptr::null();
    let mut Hook: PKSE_HOOK = core::ptr::null_mut();
    let mut Match: BOOLEAN;
    let mut Size: ULONG = 0;
    let mut Status: NTSTATUS;

    ExportEntry = KsepFindExportingModule(HookCollection, &raw mut ModuleName);
    if ExportEntry.is_null() {
        DPRINT1!(
            b"KSE: Exporting module for collection type %lu not found\n\0".as_ptr() as PCSTR,
            (*HookCollection).Type,
        );
        return;
    }

    Hook = (*HookCollection).HookArray;
    while (*Hook).Type != KseHookInvalid as ULONG {
        if (*Hook).Type != KseHookFunction as ULONG || !(*Hook).OriginalFunction.is_null() {
            Hook = Hook.offset(1);
            continue;
        }
        (*Hook).OriginalFunction = RtlFindExportedRoutineByName(
            (*ExportEntry).DllBase,
            (*Hook).__bindgen_anon_1.FunctionName as PCSTR,
        );
        if (*Hook).OriginalFunction.is_null() {
            DPRINT1!(
                b"KSE: %s not exported by %S\n\0".as_ptr() as PCSTR,
                (*Hook).__bindgen_anon_1.FunctionName,
                ModuleName,
            );
        }
        Hook = Hook.offset(1);
    }

    ImportDescriptor = RtlImageDirectoryEntryToData(
        DllBase as PVOID,
        TRUE,
        IMAGE_DIRECTORY_ENTRY_IMPORT,
        &raw mut Size,
    ) as PIMAGE_IMPORT_DESCRIPTOR;
    if ImportDescriptor.is_null() {
        return;
    }

    while (*ImportDescriptor).Name != 0 && (*ImportDescriptor).FirstThunk != 0 {
        if (*ImportDescriptor).Name >= (*LdrEntry).SizeOfImage
            || KsepIsImportFromModule(
                DllBase.offset((*ImportDescriptor).Name as isize) as PCSTR,
                ModuleName,
            ) == 0
        {
            ImportDescriptor = ImportDescriptor.offset(1);
            continue;
        }

        FirstThunk = DllBase.offset((*ImportDescriptor).FirstThunk as isize) as PIMAGE_THUNK_DATA;
        OriginalThunk = if (*ImportDescriptor).__bindgen_anon_1.OriginalFirstThunk != 0 {
            DllBase.offset((*ImportDescriptor).__bindgen_anon_1.OriginalFirstThunk as isize)
                as PIMAGE_THUNK_DATA
        } else {
            core::ptr::null_mut()
        };

        while (*FirstThunk).u1.Function != 0 {
            ImportByName = core::ptr::null_mut();
            if !OriginalThunk.is_null()
                && !image_snap_by_ordinal((*OriginalThunk).u1.Ordinal as u64)
                && ((*OriginalThunk).u1.AddressOfData as ULONGLONG) < (*LdrEntry).SizeOfImage as ULONGLONG
            {
                ImportByName = DllBase.offset((*OriginalThunk).u1.AddressOfData as isize)
                    as PIMAGE_IMPORT_BY_NAME;
            }

            Hook = (*HookCollection).HookArray;
            while (*Hook).Type != KseHookInvalid as ULONG {
                if (*Hook).Type != KseHookFunction as ULONG
                    || (*Hook).OriginalFunction.is_null()
                {
                    Hook = Hook.offset(1);
                    continue;
                }

                if !ImportByName.is_null() {
                    Match = (strcmp(
                        core::ptr::addr_of_mut!((*ImportByName).Name).cast::<::core::ffi::c_char>(),
                        (*Hook).__bindgen_anon_1.FunctionName as *const ::core::ffi::c_char,
                    ) == 0) as BOOLEAN;
                } else {
                    Match = ((*FirstThunk).u1.Function == ((*Hook).OriginalFunction as ULONG_PTR) as _)
                        as BOOLEAN;
                }

                if Match == 0 {
                    Hook = Hook.offset(1);
                    continue;
                }

                Status = KsepWriteImportEntry(FirstThunk, (*Hook).HookFunction);
                if NT_SUCCESS(Status) {
                    DPRINT1!(
                        b"KSE: Hooked %S!%s in %wZ\n\0".as_ptr() as PCSTR,
                        ModuleName,
                        (*Hook).__bindgen_anon_1.FunctionName,
                        core::ptr::addr_of_mut!((*LdrEntry).BaseDllName),
                    );
                } else {
                    DPRINT1!(
                        b"KSE: Unable to hook %S!%s in %wZ (0x%08lx)\n\0".as_ptr() as PCSTR,
                        ModuleName,
                        (*Hook).__bindgen_anon_1.FunctionName,
                        core::ptr::addr_of_mut!((*LdrEntry).BaseDllName),
                        Status,
                    );
                }
                break;
            }

            FirstThunk = FirstThunk.offset(1);
            OriginalThunk = if !OriginalThunk.is_null() {
                OriginalThunk.offset(1 as isize)
            } else {
                core::ptr::null_mut()
            };
        }

        ImportDescriptor = ImportDescriptor.offset(1);
    }
}
unsafe extern "system" fn KsepApplyShimToDriver(
    Shim: PKSE_SHIM,
    LdrEntry: PLDR_DATA_TABLE_ENTRY,
) {
    let mut HookCollection: PKSE_HOOK_COLLECTION = (*Shim).HookCollectionsArray;
    while (*HookCollection).Type != KseCollectionInvalid as ULONG {
        if (*HookCollection).Type == KseCollectionCallback as ULONG {
            HookCollection = HookCollection.offset(1);
            continue;
        }
        KsepHookImports(LdrEntry, HookCollection);
        HookCollection = HookCollection.offset(1);
    }
}

#[no_mangle]
pub unsafe extern "system" fn KseDriverLoadImage(LdrEntry: PLDR_DATA_TABLE_ENTRY) -> NTSTATUS {
    let mut Shims: [PKSE_SHIM; (KSE_MAX_SHIMS_PER_DRIVER as _)] =
        [core::ptr::null_mut(); (KSE_MAX_SHIMS_PER_DRIVER as _)];
    let mut NumberOfShims: ULONG = 0;
    let mut n: ULONG;
    let mut Status: NTSTATUS;

    if KseEngine.State as ULONG != KSE_STATE_INITIALIZED as ULONG {
        return STATUS_SUCCESS as NTSTATUS;
    }

    Status = KsepGetShimsForDriver(LdrEntry, Shims.as_mut_ptr(), &raw mut NumberOfShims);
    if !NT_SUCCESS(Status) {
        DbgPrint(
            b"(%s:%d) KSE: No Shim found for %wZ\n\0".as_ptr() as PCSTR,
            b"kse.c\0".as_ptr() as *const core::ffi::c_char,
            0 as core::ffi::c_int,
            &raw mut (*LdrEntry).BaseDllName,
        );
        return STATUS_SUCCESS as NTSTATUS;
    }

    n = 0 as ULONG;
    while n < NumberOfShims {
        DbgPrint(
            b"(%s:%d) KSE: Applying shim %S (0x%08lx) to %wZ\n\0".as_ptr() as PCSTR,
            b"kse.c\0".as_ptr() as *const core::ffi::c_char,
            0 as core::ffi::c_int,
            (*Shims[n as usize]).ShimName,
            (*(*Shims[n as usize]).ShimGuid).Data1,
            &raw mut (*LdrEntry).BaseDllName,
        );

        KsepApplyShimToDriver(Shims[n as usize], LdrEntry);

        if let Some(notify) = (*Shims[n as usize]).ShimmedDriverTargetedNotification {
            let NtHeaders: PIMAGE_NT_HEADERS = RtlImageNtHeader((*LdrEntry).DllBase);
            notify(
                &raw mut (*LdrEntry).BaseDllName,
                (*LdrEntry).DllBase,
                (*LdrEntry).SizeOfImage,
                if !NtHeaders.is_null() {
                    (*NtHeaders).FileHeader.TimeDateStamp
                } else {
                    0 as ULONG
                },
                (*LdrEntry).__bindgen_anon_1.__bindgen_anon_1.CheckSum,
            );
        }

        n = n.wrapping_add(1);
    }

    return STATUS_SUCCESS as NTSTATUS;
}

// ---------------------------------------------------------------------------
// KseRegisterShimEx — hand-written. The C asks KiPcToFileHeader(_ReturnAddress())
// for the module that called it, and refuses any hook whose function lies
// outside that module. Stable Rust has no _ReturnAddress, and c2rust cannot
// lower it, so the export is a per-arch trampoline that hands its own return
// address to the Rust body as a fifth argument. KseRegisterShim calls through
// the same symbol, so (as in the C) built-in shims register as ntoskrnl.
// ---------------------------------------------------------------------------

extern "system" {
    fn KseRegisterShimEx(Shim: PKSE_SHIM, Ignored: PVOID, Flags: ULONG, DriverObject: PVOID) -> NTSTATUS;
}


#[cfg(target_arch = "x86")]
core::arch::global_asm!(
    ".globl _KseRegisterShimEx@16",
    "_KseRegisterShimEx@16:",
    // entry: [esp] = return address, [esp+4..+16] = Shim, Ignored, Flags, DriverObject
    "push dword ptr [esp]",       // 5th argument: the caller
    "push dword ptr [esp+20]",    // DriverObject
    "push dword ptr [esp+20]",    // Flags
    "push dword ptr [esp+20]",    // Ignored
    "push dword ptr [esp+20]",    // Shim
    "call {body}",                // stdcall @20 pops its five arguments
    "ret 16",
    body = sym KsepRegisterShimExFromCaller,
);

#[cfg(target_arch = "x86_64")]
core::arch::global_asm!(
    ".globl KseRegisterShimEx",
    "KseRegisterShimEx:",
    "mov rax, [rsp]",             // the caller
    "sub rsp, 0x38",              // home space + 5th argument, 16-byte aligned at the call
    "mov [rsp+0x20], rax",
    "call {body}",
    "add rsp, 0x38",
    "ret",
    body = sym KsepRegisterShimExFromCaller,
);

#[cfg(target_arch = "aarch64")]
core::arch::global_asm!(
    ".globl KseRegisterShimEx",
    "KseRegisterShimEx:",
    "stp x29, x30, [sp, #-16]!",
    "mov x29, sp",
    "mov x4, x30",                // the caller
    "bl {body}",
    "ldp x29, x30, [sp], #16",
    "ret",
    body = sym KsepRegisterShimExFromCaller,
);

unsafe extern "system" fn KsepRegisterShimExFromCaller(
    Shim: PKSE_SHIM,
    _Ignored: PVOID,
    _Flags: ULONG,
    DriverObject: PVOID,
    Caller: PVOID,
) -> NTSTATUS {
    if Shim.is_null() || (*Shim).ShimGuid.is_null() {
        DPRINT1!(b"KSE: Unable to register empty shim or shim without guid\n\0".as_ptr() as PCSTR);
        return STATUS_INVALID_PARAMETER;
    }

    if KseEngine.State != KSE_STATE_INITIALIZED {
        DPRINT1!(
            b"KSE: Unable to register shim 0x%08x before engine is initialized\n\0".as_ptr() as PCSTR,
            (*(*Shim).ShimGuid).Data1,
        );
        return STATUS_UNSUCCESSFUL;
    }

    // Search the loaded module associated to the caller.
    let mut DataTableEntry: PLDR_DATA_TABLE_ENTRY = core::ptr::null_mut();
    let mut InKernel: BOOLEAN = 0;
    if KiPcToFileHeader(Caller, &mut DataTableEntry, FALSE as BOOLEAN, &mut InKernel).is_null() {
        return STATUS_NOT_FOUND;
    }

    // Check that every shim function belongs to the caller.
    let base = (*DataTableEntry).DllBase as ULONG_PTR;
    let size = (*DataTableEntry).SizeOfImage as ULONG_PTR;
    let mut HookCollection = (*Shim).HookCollectionsArray;
    while (*HookCollection).Type != KseCollectionInvalid as ULONG {
        let mut Hook = (*HookCollection).HookArray;
        while (*Hook).Type != KseHookInvalid as ULONG {
            let f = (*Hook).HookFunction as ULONG_PTR;
            if f < base || f - base >= size {
                return STATUS_UNSUCCESSFUL;
            }
            Hook = Hook.add(1);
        }
        HookCollection = HookCollection.add(1);
    }

    // Is the shim already registered?
    let head = core::ptr::addr_of_mut!(KseEngine.ProvidersListHead);
    let mut ListEntry = (*head).Flink;
    while ListEntry != head {
        let Provider = ListEntry as PKSE_PROVIDER; // ProviderList is the first field
        let a = &*(*(*Provider).Shim).ShimGuid;
        let b = &*(*Shim).ShimGuid;
        if a.Data1 == b.Data1 && a.Data2 == b.Data2 && a.Data3 == b.Data3 && a.Data4 == b.Data4 {
            DPRINT1!(b"KSE: GUID already registered\n\0".as_ptr() as PCSTR);
            return STATUS_OBJECT_NAME_COLLISION;
        }
        ListEntry = (*ListEntry).Flink;
    }

    // Allocate a new entry for this shim.
    let Provider = ExAllocatePoolWithTag(PagedPool, core::mem::size_of::<KSE_PROVIDER>() as SIZE_T, TAG_KSE_PROVIDER)
        as PKSE_PROVIDER;
    if Provider.is_null() {
        return STATUS_NO_MEMORY;
    }
    (*Provider).Shim = Shim;
    let entry = core::ptr::addr_of_mut!((*Provider).ProviderList);
    // InsertTailList
    let blink = (*head).Blink;
    (*entry).Flink = head;
    (*entry).Blink = blink;
    (*blink).Flink = entry;
    (*head).Blink = entry;

    // Now, update the shim.
    (*Shim).KseCallbackRoutines = core::ptr::addr_of_mut!(KseEngine.KseCallbackRoutines);

    if !DriverObject.is_null() {
        ObfReferenceObject(DriverObject);
    }

    DPRINT1!(b"KSE: Registered shim %S\n\0".as_ptr() as PCSTR, (*Shim).ShimName);
    STATUS_SUCCESS
}
