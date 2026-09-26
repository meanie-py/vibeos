use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::macros::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::ex::*;
use ntkrnl_sys::rtl::*;

use ntkrnl_sys::ddk::{
    _ARBITER_INSTANCE, _IO_BUS_TYPE_GUID_LIST, _KEVENT, _UNICODE_STRING,
    _EVENT_TYPE_NotificationEvent,
    PDEVICE_OBJECT, PKEY_VALUE_FULL_INFORMATION, PUNICODE_STRING,
    HANDLE, INTERFACE_TYPE, KPROCESSOR_MODE,
    ExAllocatePool, ExFreePool, ExInitializeFastMutex,
    IoCreateDevice, IoCreateDriver,
    IopUpdateRootKey,
    KeInitializeEvent, KeInitializeSpinLock,
    NtClose, RtlCreateUnicodeString, RtlEqualUnicodeString, RtlInitUnicodeString,
    ZwClose,
};

extern "C" {
    fn IopInitPlugPlayEvents() -> NTSTATUS;
}
extern "system" {
    fn IopGetRegistryValue(Handle: HANDLE, ValueName: PWSTR, Information: *mut PKEY_VALUE_FULL_INFORMATION) -> NTSTATUS;
    fn IopOpenRegistryKeyEx(KeyHandle: *mut HANDLE, ParentKey: HANDLE, Name: PUNICODE_STRING, DesiredAccess: u32) -> NTSTATUS;
    fn IopCreateRegistryKeyEx(Handle: *mut HANDLE, RootHandle: HANDLE, KeyName: PUNICODE_STRING, DesiredAccess: u32, CreateOptions: ULONG, Disposition: *mut ULONG) -> NTSTATUS;
}

const Isa: INTERFACE_TYPE = 1;
const NotificationEvent: u32 = 0x00;
const REG_MULTI_SZ: ULONG = 7;
const REG_SZ: ULONG = 1;
const REG_DWORD: ULONG = 4;
const REG_BINARY: ULONG = 3;
const KEY_READ: u32 = 0x20019;
const KEY_ALL_ACCESS: u32 = 0xF003F;
const REG_OPTION_NON_VOLATILE: ULONG = 0;
const REG_CREATED_NEW_KEY: ULONG = 1;
const FILE_DEVICE_CONTROLLER: ULONG = 0x00000004;
const DO_BUS_ENUMERATED_DEVICE: ULONG = 0x1000;
const DNF_MADEUP: ULONG = 0x00000001;
const DNF_ENUMERATED: ULONG = 0x10;
const DNF_IDS_QUERIED: ULONG = 0x20;
const DNF_NO_RESOURCE_REQUIRED: ULONG = 0x0100;
const PHASE1_INITIALIZATION_FAILED: ULONG = 0x32;

use ntkrnl_sys::ddk::{_DRIVER_OBJECT, _DEVICE_NODE, PDEVICE_NODE, PDRIVER_OBJECT};
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

type DRIVER_OBJECT = _DRIVER_OBJECT;
type DEVICE_NODE = _DEVICE_NODE;

#[no_mangle]
#[used]
pub static mut PiInitGroupOrderTable: PUNICODE_STRING = core::ptr::null_mut();
#[no_mangle]
#[used]
pub static mut PiInitGroupOrderTableCount: USHORT = 0;
static mut PnpDefaultInterfaceType: INTERFACE_TYPE = 0;
#[no_mangle]
#[used]
pub static mut PnPBootDriversLoaded: BOOLEAN = 0;
#[no_mangle]
#[used]
pub static mut PnPBootDriversInitialized: BOOLEAN = 0;

#[no_mangle]
#[used]
pub static mut IopRootBusNumberArbiter: _ARBITER_INSTANCE = unsafe { core::mem::zeroed() };
#[no_mangle]
#[used]
pub static mut IopRootIrqArbiter: _ARBITER_INSTANCE = unsafe { core::mem::zeroed() };
#[no_mangle]
#[used]
pub static mut IopRootDmaArbiter: _ARBITER_INSTANCE = unsafe { core::mem::zeroed() };
#[no_mangle]
#[used]
pub static mut IopRootMemArbiter: _ARBITER_INSTANCE = unsafe { core::mem::zeroed() };
#[no_mangle]
#[used]
pub static mut IopRootPortArbiter: _ARBITER_INSTANCE = unsafe { core::mem::zeroed() };

extern "C" {
    static mut PiEnumerationFinished: _KEVENT;
    static mut IopDeviceTreeLock: KSPIN_LOCK;
    static mut IopDeviceActionLock: KSPIN_LOCK;
    static mut IopDeviceActionRequestList: LIST_ENTRY;
    static mut IopRootDriverObject: PDRIVER_OBJECT;
    static mut IopRootDeviceNode: PDEVICE_NODE;
    static mut PnpBusTypeGuidList: _IO_BUS_TYPE_GUID_LIST;
}

extern "system" {
    fn IopArbPortInitialize() -> NTSTATUS;
    fn IopArbMemInitialize() -> NTSTATUS;
    fn IopArbDmaInitialize() -> NTSTATUS;
    fn IopArbIrqInitialize() -> NTSTATUS;
    fn IopArbBusNumberInitialize() -> NTSTATUS;
    fn PnpRegMultiSzToUnicodeStrings( KeyValueInformation: PKEY_VALUE_FULL_INFORMATION, GroupTable: *mut PUNICODE_STRING, Count: *mut ULONG, ) -> NTSTATUS;
    fn PnpRegSzToString(Buffer: PVOID, DataLength: ULONG, Length: *mut USHORT) -> u8;
}

// cdecl functions (no NTAPI in C declarations)
extern "C" {
    fn PipAllocateDeviceNode(Pdo: PDEVICE_OBJECT) -> PDEVICE_NODE;
    fn PnpRootInitializeDevExtension();
    fn PiSetDevNodeState(DeviceNode: PDEVICE_NODE, State: u32) -> u32;
    fn PiQueueDeviceAction( DeviceObject: PDEVICE_OBJECT, Action: u32, Callback: PVOID, Context: PVOID, );
}

extern "system" {
    fn PnpRootDriverEntry(DriverObject: PDRIVER_OBJECT, RegistryPath: PUNICODE_STRING) -> NTSTATUS;
}

const DRIVER_ROOT_NAME: &[u8] = b"\\Driver\\\0";
const REGSTR_KEY_ENUM: &[u16] = &[b'E' as u16, b'n' as u16, b'u' as u16, b'm' as u16, 0];
const REGSTR_KEY_ROOTENUM: &[u16] = &[b'R' as u16, b'o' as u16, b'o' as u16, b't' as u16, 0];
const REGSTR_VAL_ROOT_DEVNODE: &[u16] = &[b'H' as u16, b'T' as u16, b'R' as u16, b'E' as u16, b'E' as u16, b'\\' as u16, b'R' as u16, b'O' as u16, b'O' as u16, b'T' as u16, b'\\' as u16, b'0' as u16, 0];

const PiActionEnumRootDevices: u32 = 1;
const DeviceNodeStarted: u32 = 0x308;

#[no_mangle]
pub unsafe extern "system" fn IopDetermineDefaultInterfaceType() -> INTERFACE_TYPE {
    Isa
}

#[no_mangle]
pub unsafe extern "system" fn IopInitializeArbiters() -> NTSTATUS {
    let mut Status: NTSTATUS;

    Status = IopArbPortInitialize();
    if Status < 0 {
        return Status;
    }

    Status = IopArbMemInitialize();
    if Status < 0 {
        return Status;
    }

    Status = IopArbDmaInitialize();
    if Status < 0 {
        return Status;
    }

    Status = IopArbIrqInitialize();
    if Status < 0 {
        return Status;
    }

    Status = IopArbBusNumberInitialize();
    if Status < 0 {
        return Status;
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn PiInitCacheGroupInformation() -> NTSTATUS {
    let mut KeyHandle: HANDLE = core::ptr::null_mut();
    let mut Status: NTSTATUS;
    let mut KeyValueInformation: PKEY_VALUE_FULL_INFORMATION = core::ptr::null_mut();
    let mut GroupTable: PUNICODE_STRING = core::ptr::null_mut();
    let mut Count: ULONG = 0;
    let mut GroupString: _UNICODE_STRING = core::mem::zeroed();

    RtlInitUnicodeString(
        &mut GroupString,
        b"\\\0R\0e\0g\0i\0s\0t\0r\0y\0\\\0M\0a\0c\0h\0i\0n\0e\0\\\0S\0y\0s\0t\0e\0m\0\\\0C\0u\0r\0r\0e\0n\0t\0C\0o\0n\0t\0r\0o\0l\0S\0e\0t\0\\\0C\0o\0n\0t\0r\0o\0l\0\\\0S\0e\0r\0v\0i\0c\0e\0G\0r\0o\0u\0p\0O\0r\0d\0e\0r\0\0\0"
            .as_ptr() as *mut u16,
    );

    Status = IopOpenRegistryKeyEx(&mut KeyHandle, core::ptr::null_mut(), &mut GroupString, KEY_READ);
    if Status >= 0 {
        Status = IopGetRegistryValue(KeyHandle, w!("List").as_ptr() as *mut u16, &mut KeyValueInformation);
        ZwClose(KeyHandle);

        if Status >= 0 {
            // Use volatile reads to prevent LLVM from optimizing away the
            // checks (cross-CGU captures(none) attribute propagation bug)
            let kvi_ptr = core::ptr::read_volatile(&KeyValueInformation);
            let kvi_type = core::ptr::read_volatile(core::ptr::addr_of!((*kvi_ptr).Type));
            let kvi_datalen = core::ptr::read_volatile(core::ptr::addr_of!((*kvi_ptr).DataLength));
            let cond1 = (kvi_type == REG_MULTI_SZ) as ULONG;
            let cond2 = (kvi_datalen != 0) as ULONG;
            if kvi_type == REG_MULTI_SZ && kvi_datalen != 0 {
                Status = PnpRegMultiSzToUnicodeStrings(
                    KeyValueInformation,
                    &mut GroupTable,
                    &mut Count,
                );

                PiInitGroupOrderTable = GroupTable;
                PiInitGroupOrderTableCount = Count as USHORT;
            } else {
                Status = STATUS_UNSUCCESSFUL;
            }

            ExFreePool(KeyValueInformation as PVOID);
        } else {
        }
    } else {
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn PpInitGetGroupOrderIndex(ServiceHandle: HANDLE) -> USHORT {
    let ServiceHandle = opaque_ptr(ServiceHandle);
    let mut Status: NTSTATUS;
    let mut KeyValueInformation: PKEY_VALUE_FULL_INFORMATION = core::ptr::null_mut();
    let mut i: USHORT;
    let mut Buffer: PVOID;
    let mut Group: _UNICODE_STRING = core::mem::zeroed();

    if PiInitGroupOrderTable.is_null() {
        return u16::MAX;
    }

    if ServiceHandle.is_null() {
        return PiInitGroupOrderTableCount + 1;
    }

    Status = IopGetRegistryValue(ServiceHandle, w!("Group").as_ptr() as *mut u16, &mut KeyValueInformation);
    if Status < 0 {
        return PiInitGroupOrderTableCount;
    }

    Buffer = (KeyValueInformation as ULONG_PTR + (*KeyValueInformation).DataOffset as ULONG_PTR) as PVOID;
    PnpRegSzToString(Buffer, (*KeyValueInformation).DataLength, &mut Group.Length);
    Group.MaximumLength = (*KeyValueInformation).DataLength as USHORT;
    Group.Buffer = Buffer as *mut u16;

    i = 0;
    while i < PiInitGroupOrderTableCount {
        if RtlEqualUnicodeString(&mut Group, PiInitGroupOrderTable.add(i as usize), 1u8) != 0 {
            break;
        }
        i += 1;
    }

    ExFreePool(KeyValueInformation as PVOID);
    i
}

#[no_mangle]
pub unsafe extern "system" fn PipGetDriverTagPriority(ServiceHandle: HANDLE) -> USHORT {
    let ServiceHandle = opaque_ptr(ServiceHandle);
    let mut Status: NTSTATUS;
    let mut KeyHandle: HANDLE = core::ptr::null_mut();
    let mut KeyValueInformation: PKEY_VALUE_FULL_INFORMATION = core::ptr::null_mut();
    let mut KeyValueInformationTag: PKEY_VALUE_FULL_INFORMATION = core::ptr::null_mut();
    let mut KeyValueInformationGroupOrderList: PKEY_VALUE_FULL_INFORMATION = core::ptr::null_mut();
    let mut Buffer: PVOID;
    let mut Group: _UNICODE_STRING = core::mem::zeroed();
    let mut GroupOrder: *mut ULONG;
    let mut Count: ULONG;
    let mut Tag: ULONG = 0;
    let mut i: USHORT = u16::MAX;
    let mut GroupString: _UNICODE_STRING = core::mem::zeroed();

    RtlInitUnicodeString(
        &mut GroupString,
        b"\\\0R\0e\0g\0i\0s\0t\0r\0y\0\\\0M\0a\0c\0h\0i\0n\0e\0\\\0S\0y\0s\0t\0e\0m\0\\\0C\0u\0r\0r\0e\0n\0t\0C\0o\0n\0t\0r\0o\0l\0S\0e\0t\0\\\0C\0o\0n\0t\0r\0o\0l\0\\\0S\0e\0r\0v\0i\0c\0e\0G\0r\0o\0u\0p\0O\0r\0d\0e\0r\0\0\0"
            .as_ptr() as *mut u16,
    );

    Status = IopOpenRegistryKeyEx(&mut KeyHandle, core::ptr::null_mut(), &mut GroupString, KEY_READ);
    if Status < 0 {
        return u16::MAX;
    }

    Status = IopGetRegistryValue(ServiceHandle, w!("Group").as_ptr() as *mut u16, &mut KeyValueInformation);
    if Status < 0 {
        if !KeyHandle.is_null() {
            NtClose(KeyHandle);
        }
        return u16::MAX;
    }

    if (*KeyValueInformation).Type == REG_SZ && (*KeyValueInformation).DataLength != 0 {
        Buffer = (KeyValueInformation as ULONG_PTR + (*KeyValueInformation).DataOffset as ULONG_PTR) as PVOID;
        PnpRegSzToString(Buffer, (*KeyValueInformation).DataLength, &mut Group.Length);
        Group.MaximumLength = (*KeyValueInformation).DataLength as USHORT;
        Group.Buffer = Buffer as *mut u16;
    }

    Status = IopGetRegistryValue(ServiceHandle, w!("Tag").as_ptr() as *mut u16, &mut KeyValueInformationTag);
    if Status < 0 {
        if !KeyValueInformation.is_null() {
            ExFreePool(KeyValueInformation as PVOID);
        }
        if !KeyHandle.is_null() {
            NtClose(KeyHandle);
        }
        return u16::MAX;
    }

    if (*KeyValueInformationTag).Type == REG_DWORD && (*KeyValueInformationTag).DataLength != 0 {
        Tag = *(((KeyValueInformationTag as ULONG_PTR) + (*KeyValueInformationTag).DataOffset as ULONG_PTR)
            as *mut ULONG);
    }

    ExFreePool(KeyValueInformationTag as PVOID);

    Status = IopGetRegistryValue(KeyHandle, Group.Buffer, &mut KeyValueInformationGroupOrderList);

    if !KeyValueInformation.is_null() {
        ExFreePool(KeyValueInformation as PVOID);
    }
    if !KeyHandle.is_null() {
        NtClose(KeyHandle);
    }
    if Status < 0 {
        return u16::MAX;
    }

    if (*KeyValueInformationGroupOrderList).Type == REG_BINARY
        && (*KeyValueInformationGroupOrderList).DataLength != 0
    {
        GroupOrder = ((KeyValueInformationGroupOrderList as ULONG_PTR)
            + (*KeyValueInformationGroupOrderList).DataOffset as ULONG_PTR)
            as *mut ULONG;

        Count = *GroupOrder;

        GroupOrder = GroupOrder.add(1);
        i = 1;
        while i <= Count as USHORT {
            if Tag == *GroupOrder {
                break;
            }

            GroupOrder = GroupOrder.add(1);
            i += 1;
        }
    }

    ExFreePool(KeyValueInformationGroupOrderList as PVOID);
    i
}

#[no_mangle]
pub unsafe extern "system" fn IopInitializePlugPlayServices() -> NTSTATUS {
    #[inline(never)]
    unsafe fn pnp_trace(s: &[u8]) {
        crate::dbg_print!(b"%s\0", s.as_ptr() as *const i8);
    }
    #[inline(never)]
    unsafe fn pnp_trace_s(s: &[u8], v: i32) {
        crate::dbg_print!(s, v);
    }
    let mut Status: NTSTATUS;
    let mut Disposition: ULONG = 0;
    let mut KeyHandle: HANDLE = core::ptr::null_mut();
    let mut EnumHandle: HANDLE = core::ptr::null_mut();
    let mut ParentHandle: HANDLE = core::ptr::null_mut();
    let mut TreeHandle: HANDLE = core::ptr::null_mut();
    let mut ControlHandle: HANDLE = core::ptr::null_mut();
    let mut KeyName: _UNICODE_STRING = core::mem::zeroed();
    let mut PnpManagerDriverName: _UNICODE_STRING = core::mem::zeroed();
    let mut Pdo: PDEVICE_OBJECT = core::ptr::null_mut();

    KeInitializeSpinLock(core::ptr::addr_of_mut!(IopDeviceTreeLock) as *mut _);
    KeInitializeSpinLock(core::ptr::addr_of_mut!(IopDeviceActionLock) as *mut _);
    InitializeListHead(&raw mut IopDeviceActionRequestList);
    KeInitializeEvent(&raw mut PiEnumerationFinished, _EVENT_TYPE_NotificationEvent, 1u8);

    PnpDefaultInterfaceType = IopDetermineDefaultInterfaceType();

    pnp_trace(b"[PNP] IopInitializeArbiters\r\n\0");
    Status = IopInitializeArbiters();
    if Status < 0 {
        pnp_trace_s(b"[PNP] IopInitializeArbiters FAIL %x\r\n\0", Status);
        return Status;
    }

    pnp_trace(b"[PNP] PiInitCacheGroupInformation\r\n\0");
    Status = PiInitCacheGroupInformation();
    if Status < 0 {
        pnp_trace_s(b"[PNP] PiInitCacheGroupInformation FAIL %x\r\n\0", Status);
        return Status;
    }

    RtlInitUnicodeString(
        &mut KeyName,
        w!("\\REGISTRY\\MACHINE\\SYSTEM\\CURRENTCONTROLSET").as_ptr() as *mut u16,
    );
    pnp_trace(b"[PNP] open CurrentControlSet\r\n\0");
    Status = IopOpenRegistryKeyEx(&mut KeyHandle, core::ptr::null_mut(), &mut KeyName, KEY_ALL_ACCESS);
    if Status < 0 {
        pnp_trace_s(b"[PNP] open CCS FAIL %x\r\n\0", Status);
        return Status;
    }

    RtlInitUnicodeString(&mut KeyName, w!("Control").as_ptr() as *mut u16);
    pnp_trace(b"[PNP] create Control\r\n\0");
    Status = IopCreateRegistryKeyEx(
        &mut ControlHandle,
        KeyHandle,
        &mut KeyName,
        KEY_ALL_ACCESS,
        REG_OPTION_NON_VOLATILE,
        &mut Disposition,
    );
    if Status < 0 {
        pnp_trace_s(b"[PNP] create Control FAIL %x\r\n\0", Status);
        return Status;
    }

    if Disposition == REG_CREATED_NEW_KEY {
        let mut DeviceClassesHandle: HANDLE = core::ptr::null_mut();

        RtlInitUnicodeString(&mut KeyName, w!("DeviceClasses").as_ptr() as *mut u16);
        Status = IopCreateRegistryKeyEx(
            &mut DeviceClassesHandle,
            ControlHandle,
            &mut KeyName,
            KEY_ALL_ACCESS,
            REG_OPTION_NON_VOLATILE,
            &mut Disposition,
        );
        if Status < 0 {
            return Status;
        }

        ZwClose(DeviceClassesHandle);
    }

    ZwClose(ControlHandle);

    RtlInitUnicodeString(&mut KeyName, REGSTR_KEY_ENUM.as_ptr() as *mut u16);
    pnp_trace(b"[PNP] create Enum\r\n\0");
    Status = IopCreateRegistryKeyEx(
        &mut EnumHandle,
        KeyHandle,
        &mut KeyName,
        KEY_ALL_ACCESS,
        REG_OPTION_NON_VOLATILE,
        &mut Disposition,
    );
    if Status < 0 {
        pnp_trace_s(b"[PNP] create Enum FAIL %x\r\n\0", Status);
        return Status;
    }

    ParentHandle = EnumHandle;
    RtlInitUnicodeString(&mut KeyName, REGSTR_KEY_ROOTENUM.as_ptr() as *mut u16);
    pnp_trace(b"[PNP] create Root\r\n\0");
    Status = IopCreateRegistryKeyEx(
        &mut EnumHandle,
        ParentHandle,
        &mut KeyName,
        KEY_ALL_ACCESS,
        REG_OPTION_NON_VOLATILE,
        &mut Disposition,
    );
    NtClose(ParentHandle);
    if Status < 0 {
        pnp_trace_s(b"[PNP] create Root FAIL %x\r\n\0", Status);
        return Status;
    }
    NtClose(EnumHandle);

    RtlInitUnicodeString(
        &mut KeyName,
        w!("\\REGISTRY\\MACHINE\\SYSTEM\\CURRENTCONTROLSET\\ENUM").as_ptr() as *mut u16,
    );
    Status = IopOpenRegistryKeyEx(&mut EnumHandle, core::ptr::null_mut(), &mut KeyName, KEY_ALL_ACCESS);
    if Status >= 0 {
        RtlInitUnicodeString(&mut KeyName, REGSTR_VAL_ROOT_DEVNODE.as_ptr() as *mut u16);
        Status = IopCreateRegistryKeyEx(
            &mut TreeHandle,
            EnumHandle,
            &mut KeyName,
            KEY_ALL_ACCESS,
            REG_OPTION_NON_VOLATILE,
            core::ptr::null_mut(),
        );
        NtClose(EnumHandle);
        if Status >= 0 {
            NtClose(TreeHandle);
        }
    }

    // L"\\Driver\\PnpManager"
    static PNP_MANAGER_NAME: [u16; 19] = [
        b'\\' as u16, b'D' as u16, b'r' as u16, b'i' as u16, b'v' as u16, b'e' as u16, b'r' as u16,
        b'\\' as u16, b'P' as u16, b'n' as u16, b'p' as u16, b'M' as u16, b'a' as u16, b'n' as u16,
        b'a' as u16, b'g' as u16, b'e' as u16, b'r' as u16, 0,
    ];
    RtlInitUnicodeString(
        &mut PnpManagerDriverName,
        PNP_MANAGER_NAME.as_ptr() as *mut u16,
    );
    Status = IoCreateDriver(&mut PnpManagerDriverName, core::mem::transmute(PnpRootDriverEntry as usize));
    if Status < 0 {
        KeBugCheckEx(PHASE1_INITIALIZATION_FAILED, Status as ULONG_PTR, 0, 0, 0);
    }

    Status = IoCreateDevice(
        IopRootDriverObject,
        0,
        core::ptr::null_mut(),
        FILE_DEVICE_CONTROLLER,
        0,
        0u8,
        &mut Pdo,
    );
    if Status < 0 {
        KeBugCheckEx(PHASE1_INITIALIZATION_FAILED, Status as ULONG_PTR, 0, 0, 0);
    }

    (*Pdo).Flags |= DO_BUS_ENUMERATED_DEVICE;

    IopRootDeviceNode = PipAllocateDeviceNode(Pdo);

    (*IopRootDeviceNode).Flags |= DNF_MADEUP | DNF_ENUMERATED | DNF_IDS_QUERIED | DNF_NO_RESOURCE_REQUIRED;

    if RtlCreateUnicodeString(
        core::ptr::addr_of_mut!((*IopRootDeviceNode).InstancePath) as *mut _UNICODE_STRING,
        REGSTR_VAL_ROOT_DEVNODE.as_ptr() as *mut u16,
    ) == 0
    {
        KeBugCheckEx(PHASE1_INITIALIZATION_FAILED, Status as ULONG_PTR, 0, 0, 0);
    }

    PnpRootInitializeDevExtension();

    PiSetDevNodeState(IopRootDeviceNode, DeviceNodeStarted);

    Status = IopInitPlugPlayEvents();
    if Status < 0 {
        return Status;
    }

    // Upstream a7f658e3221: a global struct plus an 8-slot tagged GUID buffer.
    let List = core::ptr::addr_of_mut!(PnpBusTypeGuidList);
    ExInitializeFastMutex(core::ptr::addr_of_mut!((*List).Lock));
    (*List).AllocatedCount = 8;
    (*List).GuidCount = 0;
    (*List).Guids = ExAllocatePoolWithTag(
        PagedPool,
        ((*List).AllocatedCount as usize * core::mem::size_of::<ntkrnl_sys::ddk::GUID>()) as _,
        crate::pnpmgr::TAG_PNP_GUIDS,
    ) as *mut ntkrnl_sys::ddk::GUID;
    if (*List).Guids.is_null() {
        ntkrnl_sys::driver_ddk::DbgPrint(b"Failed to allocate PnP GUID buffer!\n\0".as_ptr() as *const i8);
        KeBugCheckEx(PHASE1_INITIALIZATION_FAILED, STATUS_NO_MEMORY as ULONG_PTR, 0, 0, 0);
    }
    core::ptr::write_bytes((*List).Guids, 0, (*List).AllocatedCount as usize);

    PiQueueDeviceAction(Pdo, PiActionEnumRootDevices, core::ptr::null_mut(), core::ptr::null_mut());

    Status = IopUpdateRootKey();
    if Status < 0 {
        return Status;
    }

    NtClose(KeyHandle);

    PiQueueDeviceAction(Pdo, PiActionEnumRootDevices, core::ptr::null_mut(), core::ptr::null_mut());

    STATUS_SUCCESS
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         ReactOS Kernel
//  * LICENSE:         BSD - See COPYING.ARM in the top level directory
//  * FILE:            ntoskrnl/io/pnpmgr/pnpinit.c
//  * PURPOSE:         PnP Initialization Code
//  * PROGRAMMERS:     ReactOS Portable Systems Group
//  */
//
// /* INCLUDES *******************************************************************/
//
// #include <ntoskrnl.h>
// #define NDEBUG
// #include <debug.h>
//
//
// /* GLOBALS ********************************************************************/
//
// PUNICODE_STRING PiInitGroupOrderTable;
// USHORT PiInitGroupOrderTableCount;
// INTERFACE_TYPE PnpDefaultInterfaceType;
// BOOLEAN PnPBootDriversLoaded = FALSE;
// BOOLEAN PnPBootDriversInitialized = FALSE;
//
// ARBITER_INSTANCE IopRootBusNumberArbiter;
// ARBITER_INSTANCE IopRootIrqArbiter;
// ARBITER_INSTANCE IopRootDmaArbiter;
// ARBITER_INSTANCE IopRootMemArbiter;
// ARBITER_INSTANCE IopRootPortArbiter;
//
// extern KEVENT PiEnumerationFinished;
//
// NTSTATUS NTAPI IopArbPortInitialize(VOID);
// NTSTATUS NTAPI IopArbMemInitialize(VOID);
// NTSTATUS NTAPI IopArbDmaInitialize(VOID);
// NTSTATUS NTAPI IopArbIrqInitialize(VOID);
// NTSTATUS NTAPI IopArbBusNumberInitialize(VOID);
//
// /* FUNCTIONS ******************************************************************/
//
// INTERFACE_TYPE
// NTAPI
// IopDetermineDefaultInterfaceType(VOID)
// {
//     /* FIXME: ReactOS doesn't support MicroChannel yet */
//     return Isa;
// }
//
// NTSTATUS
// NTAPI
// IopInitializeArbiters(VOID)
// {
//     NTSTATUS Status;
//
//     Status = IopArbPortInitialize();
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("IopArbPortInitialize() return %X\n", Status);
//         return Status;
//     }
//
//     Status = IopArbMemInitialize();
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("IopArbMemInitialize() return %X\n", Status);
//         return Status;
//     }
//
//     Status = IopArbDmaInitialize();
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("IopArbDmaInitialize() return %X\n", Status);
//         return Status;
//     }
//
//     Status = IopArbIrqInitialize();
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("IopArbIrqInitialize() return %X\n", Status);
//         return Status;
//     }
//
//     Status = IopArbBusNumberInitialize();
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("IopArbBusNumberInitialize() return %X\n", Status);
//     }
//
//     return Status;
// }
//
//
// CODE_SEG("INIT")
// NTSTATUS
// NTAPI
// PiInitCacheGroupInformation(VOID)
// {
//     HANDLE KeyHandle;
//     NTSTATUS Status;
//     PKEY_VALUE_FULL_INFORMATION KeyValueInformation;
//     PUNICODE_STRING GroupTable;
//     ULONG Count;
//     UNICODE_STRING GroupString =
//         RTL_CONSTANT_STRING(L"\\Registry\\Machine\\System\\CurrentControlSet"
//                             L"\\Control\\ServiceGroupOrder");
//
//     /* Open the registry key */
//     Status = IopOpenRegistryKeyEx(&KeyHandle,
//                                   NULL,
//                                   &GroupString,
//                                   KEY_READ);
//     if (NT_SUCCESS(Status))
//     {
//         /* Get the list */
//         Status = IopGetRegistryValue(KeyHandle, L"List", &KeyValueInformation);
//         ZwClose(KeyHandle);
//
//         /* Make sure we got it */
//         if (NT_SUCCESS(Status))
//         {
//             /* Make sure it's valid */
//             if ((KeyValueInformation->Type == REG_MULTI_SZ) &&
//                 (KeyValueInformation->DataLength))
//             {
//                 /* Convert it to unicode strings */
//                 Status = PnpRegMultiSzToUnicodeStrings(KeyValueInformation,
//                                                        &GroupTable,
//                                                        &Count);
//
//                 /* Cache it for later */
//                 PiInitGroupOrderTable = GroupTable;
//                 PiInitGroupOrderTableCount = (USHORT)Count;
//             }
//             else
//             {
//                 /* Fail */
//                 Status = STATUS_UNSUCCESSFUL;
//             }
//
//             /* Free the information */
//             ExFreePool(KeyValueInformation);
//         }
//     }
//
//     /* Return status */
//     return Status;
// }
//
// USHORT
// NTAPI
// PpInitGetGroupOrderIndex(IN HANDLE ServiceHandle)
// {
//     NTSTATUS Status;
//     PKEY_VALUE_FULL_INFORMATION KeyValueInformation;
//     USHORT i;
//     PVOID Buffer;
//     UNICODE_STRING Group;
//     PAGED_CODE();
//
//     /* Make sure we have a cache */
//     if (!PiInitGroupOrderTable) return -1;
//
//     /* If we don't have a handle, the rest is easy -- return the count */
//     if (!ServiceHandle) return PiInitGroupOrderTableCount + 1;
//
//     /* Otherwise, get the group value */
//     Status = IopGetRegistryValue(ServiceHandle, L"Group", &KeyValueInformation);
//     if (!NT_SUCCESS(Status)) return PiInitGroupOrderTableCount;
//
//     /* Make sure we have a valid string */
//     ASSERT(KeyValueInformation->Type == REG_SZ);
//     ASSERT(KeyValueInformation->DataLength);
//
//     /* Convert to unicode string */
//     Buffer = (PVOID)((ULONG_PTR)KeyValueInformation + KeyValueInformation->DataOffset);
//     PnpRegSzToString(Buffer, KeyValueInformation->DataLength, &Group.Length);
//     Group.MaximumLength = (USHORT)KeyValueInformation->DataLength;
//     Group.Buffer = Buffer;
//
//     /* Loop the groups */
//     for (i = 0; i < PiInitGroupOrderTableCount; i++)
//     {
//         /* Try to find a match */
//         if (RtlEqualUnicodeString(&Group, &PiInitGroupOrderTable[i], TRUE)) break;
//     }
//
//     /* We're done */
//     ExFreePool(KeyValueInformation);
//     return i;
// }
//
// USHORT
// NTAPI
// PipGetDriverTagPriority(IN HANDLE ServiceHandle)
// {
//     NTSTATUS Status;
//     HANDLE KeyHandle = NULL;
//     PKEY_VALUE_FULL_INFORMATION KeyValueInformation = NULL;
//     PKEY_VALUE_FULL_INFORMATION KeyValueInformationTag;
//     PKEY_VALUE_FULL_INFORMATION KeyValueInformationGroupOrderList;
//     PVOID Buffer;
//     UNICODE_STRING Group;
//     PULONG GroupOrder;
//     ULONG Count, Tag = 0;
//     USHORT i = -1;
//     UNICODE_STRING GroupString =
//     RTL_CONSTANT_STRING(L"\\Registry\\Machine\\System\\CurrentControlSet"
//                         L"\\Control\\ServiceGroupOrder");
//
//     /* Open the key */
//     Status = IopOpenRegistryKeyEx(&KeyHandle, NULL, &GroupString, KEY_READ);
//     if (!NT_SUCCESS(Status)) goto Quickie;
//
//     /* Read the group */
//     Status = IopGetRegistryValue(ServiceHandle, L"Group", &KeyValueInformation);
//     if (!NT_SUCCESS(Status)) goto Quickie;
//
//     /* Make sure we have a group */
//     if ((KeyValueInformation->Type == REG_SZ) &&
//         (KeyValueInformation->DataLength))
//     {
//         /* Convert to unicode string */
//         Buffer = (PVOID)((ULONG_PTR)KeyValueInformation + KeyValueInformation->DataOffset);
//         PnpRegSzToString(Buffer, KeyValueInformation->DataLength, &Group.Length);
//         Group.MaximumLength = (USHORT)KeyValueInformation->DataLength;
//         Group.Buffer = Buffer;
//     }
//
//     /* Now read the tag */
//     Status = IopGetRegistryValue(ServiceHandle, L"Tag", &KeyValueInformationTag);
//     if (!NT_SUCCESS(Status)) goto Quickie;
//
//     /* Make sure we have a tag */
//     if ((KeyValueInformationTag->Type == REG_DWORD) &&
//         (KeyValueInformationTag->DataLength))
//     {
//         /* Read it */
//         Tag = *(PULONG)((ULONG_PTR)KeyValueInformationTag +
//                         KeyValueInformationTag->DataOffset);
//     }
//
//     /* We can get rid of this now */
//     ExFreePool(KeyValueInformationTag);
//
//     /* Now let's read the group's tag order */
//     Status = IopGetRegistryValue(KeyHandle,
//                                  Group.Buffer,
//                                  &KeyValueInformationGroupOrderList);
//
//     /* We can get rid of this now */
// Quickie:
//     if (KeyValueInformation) ExFreePool(KeyValueInformation);
//     if (KeyHandle) NtClose(KeyHandle);
//     if (!NT_SUCCESS(Status)) return -1;
//
//     /* We're on the success path -- validate the tag order*/
//     if ((KeyValueInformationGroupOrderList->Type == REG_BINARY) &&
//         (KeyValueInformationGroupOrderList->DataLength))
//     {
//         /* Get the order array */
//         GroupOrder = (PULONG)((ULONG_PTR)KeyValueInformationGroupOrderList +
//                               KeyValueInformationGroupOrderList->DataOffset);
//
//         /* Get the count */
//         Count = *GroupOrder;
//         ASSERT(((Count + 1) * sizeof(ULONG)) <=
//                KeyValueInformationGroupOrderList->DataLength);
//
//         /* Now loop each tag */
//         GroupOrder++;
//         for (i = 1; i <= Count; i++)
//         {
//             /* If we found it, we're out */
//             if (Tag == *GroupOrder) break;
//
//             /* Try the next one */
//             GroupOrder++;
//         }
//     }
//
//     /* Last buffer to free */
//     ExFreePool(KeyValueInformationGroupOrderList);
//     return i;
// }
//
// CODE_SEG("INIT")
// NTSTATUS
// NTAPI
// IopInitializePlugPlayServices(VOID)
// {
//     NTSTATUS Status;
//     ULONG Disposition;
//     HANDLE KeyHandle, EnumHandle, ParentHandle, TreeHandle, ControlHandle;
//     UNICODE_STRING KeyName = RTL_CONSTANT_STRING(L"\\REGISTRY\\MACHINE\\SYSTEM\\CURRENTCONTROLSET");
//     UNICODE_STRING PnpManagerDriverName = RTL_CONSTANT_STRING(DRIVER_ROOT_NAME L"PnpManager");
//     PDEVICE_OBJECT Pdo;
//
//     /* Initialize locks and such */
//     ARM64_UART_TRACE("[PNP] init locks\r\n");
//     KeInitializeSpinLock(&IopDeviceTreeLock);
//     KeInitializeSpinLock(&IopDeviceActionLock);
//     InitializeListHead(&IopDeviceActionRequestList);
//     KeInitializeEvent(&PiEnumerationFinished, NotificationEvent, TRUE);
//
//     /* Get the default interface */
//     ARM64_UART_TRACE("[PNP] IopDetermineDefaultInterfaceType\r\n");
//     PnpDefaultInterfaceType = IopDetermineDefaultInterfaceType();
//
//     /* Initialize arbiters */
//     ARM64_UART_TRACE("[PNP] IopInitializeArbiters\r\n");
//     Status = IopInitializeArbiters();
//     if (!NT_SUCCESS(Status)) return Status;
//
//     /* Setup the group cache */
//     ARM64_UART_TRACE("[PNP] PiInitCacheGroupInformation\r\n");
//     Status = PiInitCacheGroupInformation();
//     if (!NT_SUCCESS(Status)) return Status;
//
//     /* Open the current control set */
//     ARM64_UART_TRACE("[PNP] open CurrentControlSet\r\n");
//     Status = IopOpenRegistryKeyEx(&KeyHandle,
//                                   NULL,
//                                   &KeyName,
//                                   KEY_ALL_ACCESS);
//     if (!NT_SUCCESS(Status)) return Status;
//
//     /* Create the control key */
//     RtlInitUnicodeString(&KeyName, L"Control");
//     Status = IopCreateRegistryKeyEx(&ControlHandle,
//                                     KeyHandle,
//                                     &KeyName,
//                                     KEY_ALL_ACCESS,
//                                     REG_OPTION_NON_VOLATILE,
//                                     &Disposition);
//     if (!NT_SUCCESS(Status)) return Status;
//
//     /* Check if it's a new key */
//     if (Disposition == REG_CREATED_NEW_KEY)
//     {
//         HANDLE DeviceClassesHandle;
//
//         /* Create the device classes key */
//         RtlInitUnicodeString(&KeyName, L"DeviceClasses");
//         Status = IopCreateRegistryKeyEx(&DeviceClassesHandle,
//                                         ControlHandle,
//                                         &KeyName,
//                                         KEY_ALL_ACCESS,
//                                         REG_OPTION_NON_VOLATILE,
//                                         &Disposition);
//         if (!NT_SUCCESS(Status)) return Status;
//
//         ZwClose(DeviceClassesHandle);
//     }
//
//     ZwClose(ControlHandle);
//
//     /* Create the enum key */
//     RtlInitUnicodeString(&KeyName, REGSTR_KEY_ENUM);
//     Status = IopCreateRegistryKeyEx(&EnumHandle,
//                                     KeyHandle,
//                                     &KeyName,
//                                     KEY_ALL_ACCESS,
//                                     REG_OPTION_NON_VOLATILE,
//                                     &Disposition);
//     if (!NT_SUCCESS(Status)) return Status;
//
//     /* Check if it's a new key */
//     if (Disposition == REG_CREATED_NEW_KEY)
//     {
//         /* FIXME: DACLs */
//     }
//
//     /* Create the root key */
//     ParentHandle = EnumHandle;
//     RtlInitUnicodeString(&KeyName, REGSTR_KEY_ROOTENUM);
//     Status = IopCreateRegistryKeyEx(&EnumHandle,
//                                     ParentHandle,
//                                     &KeyName,
//                                     KEY_ALL_ACCESS,
//                                     REG_OPTION_NON_VOLATILE,
//                                     &Disposition);
//     NtClose(ParentHandle);
//     if (!NT_SUCCESS(Status)) return Status;
//     NtClose(EnumHandle);
//
//     /* Open the root key now */
//     RtlInitUnicodeString(&KeyName, L"\\REGISTRY\\MACHINE\\SYSTEM\\CURRENTCONTROLSET\\ENUM");
//     Status = IopOpenRegistryKeyEx(&EnumHandle,
//                                   NULL,
//                                   &KeyName,
//                                   KEY_ALL_ACCESS);
//     if (NT_SUCCESS(Status))
//     {
//         /* Create the root dev node */
//         RtlInitUnicodeString(&KeyName, REGSTR_VAL_ROOT_DEVNODE);
//         Status = IopCreateRegistryKeyEx(&TreeHandle,
//                                         EnumHandle,
//                                         &KeyName,
//                                         KEY_ALL_ACCESS,
//                                         REG_OPTION_NON_VOLATILE,
//                                         NULL);
//         NtClose(EnumHandle);
//         if (NT_SUCCESS(Status)) NtClose(TreeHandle);
//     }
//
//     /* Create the root driver */
//     ARM64_UART_TRACE("[PNP] IoCreateDriver PnpManager\r\n");
//     Status = IoCreateDriver(&PnpManagerDriverName, PnpRootDriverEntry);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("IoCreateDriverObject() failed\n");
//         KeBugCheckEx(PHASE1_INITIALIZATION_FAILED, Status, 0, 0, 0);
//     }
//
//     /* Create the root PDO */
//     ARM64_UART_TRACE("[PNP] IoCreateDevice root PDO\r\n");
//     Status = IoCreateDevice(IopRootDriverObject,
//                             0,
//                             NULL,
//                             FILE_DEVICE_CONTROLLER,
//                             0,
//                             FALSE,
//                             &Pdo);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("IoCreateDevice() failed\n");
//         KeBugCheckEx(PHASE1_INITIALIZATION_FAILED, Status, 0, 0, 0);
//     }
//
//     /* This is a bus enumerated device */
//     Pdo->Flags |= DO_BUS_ENUMERATED_DEVICE;
//
//     /* Create the root device node */
//     IopRootDeviceNode = PipAllocateDeviceNode(Pdo);
//
//     /* Set flags */
//     IopRootDeviceNode->Flags |= DNF_MADEUP | DNF_ENUMERATED |
//                                 DNF_IDS_QUERIED | DNF_NO_RESOURCE_REQUIRED;
//
//     /* Create instance path */
//     if (!RtlCreateUnicodeString(&IopRootDeviceNode->InstancePath, REGSTR_VAL_ROOT_DEVNODE))
//     {
//         DPRINT1("RtlCreateUnicodeString() failed\n");
//         KeBugCheckEx(PHASE1_INITIALIZATION_FAILED, Status, 0, 0, 0);
//     }
//
//     PnpRootInitializeDevExtension();
//
//     PiSetDevNodeState(IopRootDeviceNode, DeviceNodeStarted);
//
//     /* Initialize PnP-Event notification support */
//     ARM64_UART_TRACE("[PNP] IopInitPlugPlayEvents\r\n");
//     Status = IopInitPlugPlayEvents();
//     if (!NT_SUCCESS(Status)) return Status;
//
//     /* Initialize the Bus Type GUID List */
//     PnpBusTypeGuidList = ExAllocatePool(PagedPool, sizeof(IO_BUS_TYPE_GUID_LIST));
//     RtlZeroMemory(PnpBusTypeGuidList, sizeof(IO_BUS_TYPE_GUID_LIST));
//     ExInitializeFastMutex(&PnpBusTypeGuidList->Lock);
//
//     /* Initialize PnP root relations (this is a syncronous operation) */
//     ARM64_UART_TRACE("[PNP] PiQueueDeviceAction(1)\r\n");
//     PiQueueDeviceAction(Pdo, PiActionEnumRootDevices, NULL, NULL);
//     ARM64_UART_TRACE("[PNP] PiQueueDeviceAction(1) done\r\n");
//
//     /* Launch the firmware mapper */
//     ARM64_UART_TRACE("[PNP] IopUpdateRootKey\r\n");
//     Status = IopUpdateRootKey();
//     if (!NT_SUCCESS(Status)) return Status;
//
//     /* Close the handle to the control set */
//     NtClose(KeyHandle);
//
//     /* Initialize PnP root relations (this is a syncronous operation) */
//     ARM64_UART_TRACE("[PNP] PiQueueDeviceAction(2)\r\n");
//     PiQueueDeviceAction(Pdo, PiActionEnumRootDevices, NULL, NULL);
//     ARM64_UART_TRACE("[PNP] PiQueueDeviceAction(2) done\r\n");
//
//     /* We made it */
//     ARM64_UART_TRACE("[PNP] IopInitializePlugPlayServices done\r\n");
//     return STATUS_SUCCESS;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
