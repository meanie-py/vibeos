use crate::misc::{ForwardIrpAndForget, DuplicateUnicodeString};
use crate::kbdclass_h::*;
// Glob the hand-written modules (mutually disjoint) + driver_ddk (deduped vs constants).
use ntkrnl_sys::globals::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ex::*;
use ntkrnl_sys::ob::*;
use ntkrnl_sys::driver_ddk::*;
// FORCEINLINE helpers (bindgen emits extern decls with no symbol) — real impls:
use ntkrnl_sys::inline::ke::KeInitializeSpinLock;
use ntkrnl_sys::inline::tier1::{
    IoGetCurrentIrpStackLocation, IoMarkIrpPending, IoSetCancelRoutine,
    IoSkipCurrentIrpStackLocation,
};
// ddk-ONLY names (pointer aliases, bindgen `_`-structs, Io/Zw/Rtl funcs, enum variants).
use ntkrnl_sys::ddk::{
    CCHAR, PDEVICE_OBJECT, PDRIVER_OBJECT, PIRP, PUNICODE_STRING, PFILE_OBJECT,
    PIO_STACK_LOCATION, _UNICODE_STRING, _OBJECT_ATTRIBUTES, _IO_STATUS_BLOCK,
    IO_STATUS_BLOCK, KEVENT, OBJECT_ATTRIBUTES, KEYBOARD_INPUT_DATA,
    KEY_VALUE_BASIC_INFORMATION, PKEYBOARD_INPUT_DATA, PKEY_VALUE_BASIC_INFORMATION,
    RTL_QUERY_REGISTRY_TABLE,
    IoAcquireCancelSpinLock, IoAllocateDriverObjectExtension,
    IoAttachDeviceToDeviceStackSafe, IoBuildDeviceIoControlRequest, IoCreateDevice,
    IoDeleteDevice, IoDetachDevice, IoForwardIrpSynchronously, IoGetDeviceObjectPointer,
    IoGetDriverObjectExtension, IoRegisterDeviceInterface, IoRegisterDriverReinitialization,
    IoReleaseCancelSpinLock, IoSetDeviceInterfaceState, IoSetStartIoAttributes,
    KeInitializeEvent, KeWaitForSingleObject,
    ZwClose, ZwEnumerateValueKey, ZwOpenFile, ZwOpenKey,
    RtlCopyUnicodeString, RtlAppendUnicodeToString, RtlAppendUnicodeStringToString,
    RtlCreateUnicodeString, RtlQueryRegistryValues, RtlWriteRegistryValue,
    RtlDeleteRegistryValue,
    _EVENT_TYPE_NotificationEvent,
    _KWAIT_REASON_Suspended, _GUID,
};
use core::ffi::c_void;

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

// ---- file-scope preamble (translated) ----
pub const KEYBOARD_BUFFER_SIZE: u32 = 100;

unsafe extern "system" fn DriverUnload(_DriverObject: PDRIVER_OBJECT) {
}

unsafe extern "system" fn ClassCreate(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    if (*((*DeviceObject).DeviceExtension as PCOMMON_DEVICE_EXTENSION)).IsClassDO == 0 {
        return ForwardIrpAndForget(DeviceObject, Irp);
    }

    (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
    (*Irp).IoStatus.Information = 0;
    IoCompleteRequest(Irp, IO_NO_INCREMENT as CCHAR);
    STATUS_SUCCESS
}

unsafe extern "system" fn ClassClose(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    if ((*((*DeviceObject).DeviceExtension as PCOMMON_DEVICE_EXTENSION))).IsClassDO == 0 {
        return ForwardIrpAndForget(DeviceObject, Irp);
    }

    (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
    (*Irp).IoStatus.Information = 0;
    IoCompleteRequest(Irp, IO_NO_INCREMENT as CCHAR);
    STATUS_SUCCESS
}

unsafe extern "system" fn ClassCleanup(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    if (*((*DeviceObject).DeviceExtension as PCOMMON_DEVICE_EXTENSION)).IsClassDO == 0 {
        return ForwardIrpAndForget(DeviceObject, Irp);
    }

    (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
    (*Irp).IoStatus.Information = 0;
    IoCompleteRequest(Irp, IO_NO_INCREMENT as CCHAR);
    STATUS_SUCCESS
}

unsafe extern "system" fn ClassRead(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    let DeviceExtension = (*DeviceObject).DeviceExtension as PCLASS_DEVICE_EXTENSION;
    let mut OldIrql: KIRQL = 0;
    let Status: NTSTATUS;

    if ((*DeviceObject).DeviceExtension as PCOMMON_DEVICE_EXTENSION).as_ref().is_none() {
    }
    if (*((*DeviceObject).DeviceExtension as PCOMMON_DEVICE_EXTENSION)).IsClassDO == 0 {
        return ForwardIrpAndForget(DeviceObject, Irp);
    }

    if ((*IoGetCurrentIrpStackLocation(Irp)).Parameters.Read.Length as usize)
        < core::mem::size_of::<KEYBOARD_INPUT_DATA>()
    {
        (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_BUFFER_TOO_SMALL;
        (*Irp).IoStatus.Information = 0;
        IoCompleteRequest(Irp, IO_NO_INCREMENT as CCHAR);

        return STATUS_BUFFER_TOO_SMALL;
    }

    KeAcquireSpinLock(
        core::ptr::addr_of_mut!((*DeviceExtension).SpinLock),
        &mut OldIrql,
    );
    Status = HandleReadIrp(DeviceObject, Irp, 0u8);
    KeReleaseSpinLock(core::ptr::addr_of_mut!((*DeviceExtension).SpinLock), OldIrql);
    Status
}

unsafe extern "system" fn ClassDeviceControl(
	DeviceObject: PDEVICE_OBJECT,
	Irp: PIRP,
) -> NTSTATUS {
	let mut Status: NTSTATUS = STATUS_NOT_SUPPORTED;

	if (*((*DeviceObject).DeviceExtension as PCOMMON_DEVICE_EXTENSION)).IsClassDO == 0 {
		return ForwardIrpAndForget(DeviceObject, Irp);
	}

	let IoControlCode = (*IoGetCurrentIrpStackLocation(Irp))
		.Parameters
		.DeviceIoControl
		.IoControlCode;

	'block: {
		match IoControlCode {
			IOCTL_KEYBOARD_QUERY_ATTRIBUTES
			| IOCTL_KEYBOARD_QUERY_INDICATOR_TRANSLATION
			| IOCTL_KEYBOARD_QUERY_INDICATORS
			| IOCTL_KEYBOARD_QUERY_TYPEMATIC => {
				let Head: PLIST_ENTRY = core::ptr::addr_of_mut!(
					(*((*DeviceObject).DeviceExtension as PCLASS_DEVICE_EXTENSION)).ListHead
				);
				if (*Head).Flink != Head {
					let DevExt: PPORT_DEVICE_EXTENSION = ((*Head).Flink as *mut u8)
						.offset(-(core::mem::offset_of!(PORT_DEVICE_EXTENSION, ListEntry) as isize))
						as PPORT_DEVICE_EXTENSION;
					(*IoGetCurrentIrpStackLocation(Irp)).MajorFunction =
						IRP_MJ_INTERNAL_DEVICE_CONTROL as u8;
					IoSkipCurrentIrpStackLocation(Irp);
					return IoCallDriver((*DevExt).DeviceObject, Irp);
				}
			}
			IOCTL_KEYBOARD_SET_INDICATORS | IOCTL_KEYBOARD_SET_TYPEMATIC => {
				let Head: PLIST_ENTRY = core::ptr::addr_of_mut!(
					(*((*DeviceObject).DeviceExtension as PCLASS_DEVICE_EXTENSION)).ListHead
				);
				let mut Entry: PLIST_ENTRY = (*Head).Flink;
				Status = STATUS_SUCCESS;
				while Entry != Head {
					let DevExt: PPORT_DEVICE_EXTENSION = (Entry as *mut u8)
						.offset(-(core::mem::offset_of!(PORT_DEVICE_EXTENSION, ListEntry) as isize))
						as PPORT_DEVICE_EXTENSION;

					(*IoGetCurrentIrpStackLocation(Irp)).MajorFunction =
						IRP_MJ_INTERNAL_DEVICE_CONTROL as u8;

					if IoForwardIrpSynchronously((*DevExt).LowerDevice, Irp) != 0 {
						if !NT_SUCCESS((*Irp).IoStatus.__bindgen_anon_1.Status) {
							Status = (*Irp).IoStatus.__bindgen_anon_1.Status;
						}
					} else {
						Status = STATUS_UNSUCCESSFUL;
					}

					Entry = (*Entry).Flink;
				}
			}
			_ => {
				break 'block;
			}
		}
	}

	(*Irp).IoStatus.__bindgen_anon_1.Status = Status;
	(*Irp).IoStatus.Information = 0;
	IoCompleteRequest(Irp, IO_NO_INCREMENT as CCHAR);

	Status
}

unsafe extern "system" fn ClassPower(
	DeviceObject: PDEVICE_OBJECT,
	Irp: PIRP) -> NTSTATUS
{
	let DeviceExtension = (*DeviceObject).DeviceExtension as PPORT_DEVICE_EXTENSION;
	if (*DeviceExtension).Common.IsClassDO == 0
	{
		PoStartNextPowerIrp(Irp);
		IoSkipCurrentIrpStackLocation(Irp);
		return PoCallDriver((*DeviceExtension).LowerDevice, Irp);
	}

	match (*IoGetCurrentIrpStackLocation(Irp)).MinorFunction
	{
		IRP_MN_SET_POWER | IRP_MN_QUERY_POWER => {
			(*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
		}
		_ => {}
	}
	let Status = (*Irp).IoStatus.__bindgen_anon_1.Status;
	PoStartNextPowerIrp(Irp);
	IoCompleteRequest(Irp, IO_NO_INCREMENT as CCHAR);
	Status
}

unsafe fn ReadRegistryEntries(
	RegistryPath: PUNICODE_STRING,
	DriverExtension: PCLASS_DRIVER_EXTENSION,
) -> NTSTATUS {
	let mut ParametersRegistryKey: _UNICODE_STRING = core::mem::zeroed();
	let mut Parameters: [RTL_QUERY_REGISTRY_TABLE; 4] = core::mem::zeroed();
	let mut Status: NTSTATUS;

	let mut DefaultConnectMultiplePorts: ULONG = 1;
	let mut DefaultDataQueueSize: ULONG = 0x64;
	let DefaultDeviceBaseName: PCWSTR = {
		static NAME: [u16; 14] = [
			b'K' as u16, b'e' as u16, b'y' as u16, b'b' as u16, b'o' as u16,
			b'a' as u16, b'r' as u16, b'd' as u16, b'C' as u16, b'l' as u16,
			b'a' as u16, b's' as u16, b's' as u16, 0,
		];
		NAME.as_ptr()
	};

	ParametersRegistryKey.Length = 0;
	ParametersRegistryKey.MaximumLength = ((*RegistryPath).Length as usize
		+ core::mem::size_of::<[u16; 12]>()
		+ core::mem::size_of::<u16>()) as u16;
	ParametersRegistryKey.Buffer = ExAllocatePoolWithTag(
		PagedPool,
		ParametersRegistryKey.MaximumLength as usize,
		CLASS_TAG,
	) as *mut u16;
	if ParametersRegistryKey.Buffer.is_null() {
		return STATUS_NO_MEMORY;
	}
	RtlCopyUnicodeString(&mut ParametersRegistryKey, RegistryPath);

	let parameters_str: [u16; 12] = [
		b'\\' as u16, b'P' as u16, b'a' as u16, b'r' as u16, b'a' as u16,
		b'm' as u16, b'e' as u16, b't' as u16, b'e' as u16, b'r' as u16,
		b's' as u16, 0,
	];
	RtlAppendUnicodeToString(&mut ParametersRegistryKey, parameters_str.as_ptr());
	*ParametersRegistryKey
		.Buffer
		.add(ParametersRegistryKey.Length as usize / core::mem::size_of::<u16>()) = 0;

	RtlZeroMemory(
		Parameters.as_mut_ptr() as *mut _,
		core::mem::size_of::<[RTL_QUERY_REGISTRY_TABLE; 4]>() as _,
	);

	let name0: [u16; 21] = [
		b'C' as u16, b'o' as u16, b'n' as u16, b'n' as u16, b'e' as u16,
		b'c' as u16, b't' as u16, b'M' as u16, b'u' as u16, b'l' as u16,
		b't' as u16, b'i' as u16, b'p' as u16, b'l' as u16, b'e' as u16,
		b'P' as u16, b'o' as u16, b'r' as u16, b't' as u16, b's' as u16, 0,
	];
	Parameters[0].Flags = RTL_QUERY_REGISTRY_DIRECT;
	Parameters[0].Name = name0.as_ptr() as *mut _;
	Parameters[0].EntryContext =
		core::ptr::addr_of_mut!((*DriverExtension).ConnectMultiplePorts) as *mut _;
	Parameters[0].DefaultType = REG_DWORD;
	Parameters[0].DefaultData = &mut DefaultConnectMultiplePorts as *mut _ as *mut _;
	Parameters[0].DefaultLength = core::mem::size_of::<ULONG>() as u32;

	let name1: [u16; 22] = [
		b'K' as u16, b'e' as u16, b'y' as u16, b'b' as u16, b'o' as u16,
		b'a' as u16, b'r' as u16, b'd' as u16, b'D' as u16, b'a' as u16,
		b't' as u16, b'a' as u16, b'Q' as u16, b'u' as u16, b'e' as u16,
		b'u' as u16, b'e' as u16, b'S' as u16, b'i' as u16, b'z' as u16,
		b'e' as u16, 0,
	];
	Parameters[1].Flags = RTL_QUERY_REGISTRY_DIRECT;
	Parameters[1].Name = name1.as_ptr() as *mut _;
	Parameters[1].EntryContext =
		core::ptr::addr_of_mut!((*DriverExtension).DataQueueSize) as *mut _;
	Parameters[1].DefaultType = REG_DWORD;
	Parameters[1].DefaultData = &mut DefaultDataQueueSize as *mut _ as *mut _;
	Parameters[1].DefaultLength = core::mem::size_of::<ULONG>() as u32;

	let name2: [u16; 23] = [
		b'K' as u16, b'e' as u16, b'y' as u16, b'b' as u16, b'o' as u16,
		b'a' as u16, b'r' as u16, b'd' as u16, b'D' as u16, b'e' as u16,
		b'v' as u16, b'i' as u16, b'c' as u16, b'e' as u16, b'B' as u16,
		b'a' as u16, b's' as u16, b'e' as u16, b'N' as u16, b'a' as u16,
		b'm' as u16, b'e' as u16, 0,
	];
	Parameters[2].Flags = RTL_QUERY_REGISTRY_DIRECT;
	Parameters[2].Name = name2.as_ptr() as *mut _;
	Parameters[2].EntryContext =
		core::ptr::addr_of_mut!((*DriverExtension).DeviceBaseName) as *mut _;
	Parameters[2].DefaultType = REG_SZ;
	Parameters[2].DefaultData = DefaultDeviceBaseName as *mut _;
	Parameters[2].DefaultLength = 0;

	Status = RtlQueryRegistryValues(
		(RTL_REGISTRY_ABSOLUTE | RTL_REGISTRY_OPTIONAL) as u32,
		ParametersRegistryKey.Buffer,
		Parameters.as_mut_ptr(),
		core::ptr::null_mut(),
		core::ptr::null_mut(),
	);

	if NT_SUCCESS(Status) {
		if (*DriverExtension).ConnectMultiplePorts != 0
			&& (*DriverExtension).ConnectMultiplePorts != 1
		{
			(*DriverExtension).ConnectMultiplePorts = DefaultConnectMultiplePorts;
		}
		if (*DriverExtension).DataQueueSize == 0 {
			(*DriverExtension).DataQueueSize = DefaultDataQueueSize;
		}
	} else if Status == STATUS_OBJECT_NAME_NOT_FOUND {
		(*DriverExtension).ConnectMultiplePorts = DefaultConnectMultiplePorts;
		(*DriverExtension).DataQueueSize = DefaultDataQueueSize;
		if RtlCreateUnicodeString(
			core::ptr::addr_of_mut!((*DriverExtension).DeviceBaseName),
			DefaultDeviceBaseName,
		) != 0
		{
			Status = STATUS_SUCCESS;
		} else {
			Status = STATUS_NO_MEMORY;
		}
	}

	ExFreePoolWithTag(ParametersRegistryKey.Buffer as *mut _, CLASS_TAG);
	Status
}

unsafe fn CreateClassDeviceObject(
    DriverObject: PDRIVER_OBJECT,
    ClassDO: *mut PDEVICE_OBJECT,
) -> NTSTATUS {
    let DriverExtension: PCLASS_DRIVER_EXTENSION;
    let mut DeviceId: ULONG = 0;
    let PrefixLength: ULONG;
    let mut DeviceNameU: _UNICODE_STRING = core::mem::zeroed();
    let DeviceIdW: PWSTR;
    let mut Fdo: PDEVICE_OBJECT = core::ptr::null_mut();
    let DeviceExtension: PCLASS_DEVICE_EXTENSION;
    let mut Status: NTSTATUS;

    DriverExtension = IoGetDriverObjectExtension(DriverObject, DriverObject as *mut _) as PCLASS_DRIVER_EXTENSION;
    DeviceNameU.Length = 0;
    DeviceNameU.MaximumLength = (wcslen(w!("\\Device\\").as_ptr() as PWSTR) as USHORT)
        .wrapping_mul(core::mem::size_of::<u16>() as USHORT)
        + (*DriverExtension).DeviceBaseName.Length
        + (4 * core::mem::size_of::<u16>()) as USHORT
        + core::mem::size_of::<u16>() as USHORT;
    DeviceNameU.Buffer = ExAllocatePoolWithTag(PagedPool, DeviceNameU.MaximumLength as usize, CLASS_TAG) as PWSTR;
    if DeviceNameU.Buffer.is_null() {
        return STATUS_NO_MEMORY;
    }

    Status = 'block: {
        let DeviceTag: [u16; 9] = [0x5c, 0x44, 0x65, 0x76, 0x69, 0x63, 0x65, 0x5c, 0];
        let s = RtlAppendUnicodeToString(&mut DeviceNameU, DeviceTag.as_ptr() as PWSTR);
        if !NT_SUCCESS(s) {
            break 'block s;
        }
        let s = RtlAppendUnicodeStringToString(&mut DeviceNameU, &mut (*DriverExtension).DeviceBaseName);
        if !NT_SUCCESS(s) {
            break 'block s;
        }
        PrefixLength = DeviceNameU.MaximumLength as ULONG
            - (4 * core::mem::size_of::<u16>()) as ULONG
            - core::mem::size_of::<u16>() as ULONG;
        DeviceIdW = DeviceNameU.Buffer.add(PrefixLength as usize / core::mem::size_of::<u16>());
        let mut result: NTSTATUS = STATUS_TOO_MANY_NAMES;
        while DeviceId < 9999 {
            let fmt: [u16; 4] = [0x25, 0x6c, 0x75, 0]; /* "%lu" */
            DeviceNameU.Length = (PrefixLength
                + (swprintf(DeviceIdW, fmt.as_ptr() as PWSTR, DeviceId) as ULONG)
                    * core::mem::size_of::<u16>() as ULONG) as USHORT;
            let s = IoCreateDevice(
                DriverObject,
                core::mem::size_of::<CLASS_DEVICE_EXTENSION>() as ULONG,
                &mut DeviceNameU,
                FILE_DEVICE_KEYBOARD,
                FILE_DEVICE_SECURE_OPEN,
                0u8,
                &mut Fdo,
            );
            if NT_SUCCESS(s) {
                result = s;
                break 'block result;
            } else if s != STATUS_OBJECT_NAME_COLLISION {
                result = s;
                break 'block result;
            }
            DeviceId += 1;
        }
        result
    };

    if !NT_SUCCESS(Status) {
        ExFreePoolWithTag(DeviceNameU.Buffer as *mut _, CLASS_TAG);
        return Status;
    }

    DeviceExtension = (*Fdo).DeviceExtension as PCLASS_DEVICE_EXTENSION;
    RtlZeroMemory(DeviceExtension as *mut _, core::mem::size_of::<CLASS_DEVICE_EXTENSION>());
    (*DeviceExtension).Common.IsClassDO = 1u8;
    (*DeviceExtension).DriverExtension = DriverExtension;
    InitializeListHead(core::ptr::addr_of_mut!((*DeviceExtension).ListHead));
    KeInitializeSpinLock(core::ptr::addr_of_mut!((*DeviceExtension).ListSpinLock));
    KeInitializeSpinLock(core::ptr::addr_of_mut!((*DeviceExtension).SpinLock));
    (*DeviceExtension).InputCount = 0;
    (*DeviceExtension).PortData = ExAllocatePoolWithTag(
        NonPagedPool,
        (*(*DeviceExtension).DriverExtension).DataQueueSize as usize
            * core::mem::size_of::<KEYBOARD_INPUT_DATA>(),
        CLASS_TAG,
    ) as *mut KEYBOARD_INPUT_DATA;
    if (*DeviceExtension).PortData.is_null() {
        ExFreePoolWithTag(DeviceNameU.Buffer as *mut _, CLASS_TAG);
        return STATUS_NO_MEMORY;
    }
    (*DeviceExtension).DeviceName = DeviceNameU.Buffer;
    (*Fdo).Flags |= DO_POWER_PAGABLE;
    (*Fdo).Flags |= DO_BUFFERED_IO;
    (*Fdo).Flags &= !DO_DEVICE_INITIALIZING;

    RtlWriteRegistryValue(
        RTL_REGISTRY_DEVICEMAP,
        (*DriverExtension).DeviceBaseName.Buffer,
        (*DeviceExtension).DeviceName as *mut _,
        REG_SZ,
        (*DriverExtension).RegistryPath.Buffer as *mut _,
        (*DriverExtension).RegistryPath.MaximumLength as ULONG,
    );

    if !ClassDO.is_null() {
        *ClassDO = Fdo;
    }

    STATUS_SUCCESS
}

unsafe fn FillEntries(
    ClassDeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
    DataStart: PKEYBOARD_INPUT_DATA,
    NumberOfEntries: SIZE_T,
) -> NTSTATUS {
    let mut Status: NTSTATUS = STATUS_SUCCESS;

    if (*ClassDeviceObject).Flags & DO_BUFFERED_IO != 0 {
        RtlCopyMemory(
            (*Irp).AssociatedIrp.SystemBuffer,
            DataStart as *const _ as *mut _,
            NumberOfEntries * core::mem::size_of::<KEYBOARD_INPUT_DATA>(),
        );
    } else if (*ClassDeviceObject).Flags & DO_DIRECT_IO != 0 {
        let DestAddress = MmGetSystemAddressForMdlSafe((*Irp).MdlAddress, NormalPagePriority as ULONG);
        if !DestAddress.is_null() {
            RtlCopyMemory(
                DestAddress,
                DataStart as *const _ as *mut _,
                NumberOfEntries * core::mem::size_of::<KEYBOARD_INPUT_DATA>(),
            );
        } else {
            Status = STATUS_UNSUCCESSFUL;
        }
    } else {
        RtlCopyMemory(
            (*Irp).UserBuffer,
            DataStart as *const _ as *mut _,
            NumberOfEntries * core::mem::size_of::<KEYBOARD_INPUT_DATA>(),
        );
    }

    Status
}

unsafe extern "system" fn ClassCallback(
    ClassDeviceObject: PDEVICE_OBJECT,
    DataStart: PKEYBOARD_INPUT_DATA,
    DataEnd: PKEYBOARD_INPUT_DATA,
    ConsumedCount: PULONG,
) -> BOOLEAN {
    let ClassDeviceExtension = (*ClassDeviceObject).DeviceExtension as PCLASS_DEVICE_EXTENSION;
    let mut OldIrql: KIRQL = 0;
    let InputCount: SIZE_T =
        (DataEnd as usize - DataStart as usize) / core::mem::size_of::<KEYBOARD_INPUT_DATA>();
    let ReadSize: SIZE_T;

    KeAcquireSpinLock(
        core::ptr::addr_of_mut!((*ClassDeviceExtension).SpinLock),
        &mut OldIrql,
    );
    if InputCount > 0 {
        static mut g_FirstKbdDeliveredLogged: LONG = 0;
        if {
            let dest = core::ptr::addr_of_mut!(g_FirstKbdDeliveredLogged);
            let old = *dest;
            if old == 0 {
                *dest = 1;
            }
            old
        } == 0
        {
            DbgPrint(
                b"[KBDCLASS] first key delivered sc=0x%x flags=0x%x cnt=%lu\n\0".as_ptr()
                    as *const i8,
                (*DataStart).MakeCode as u32,
                (*DataStart).Flags as u32,
                InputCount as ULONG,
            );
        }

        if (*ClassDeviceExtension).InputCount + InputCount
            > (*(*ClassDeviceExtension).DriverExtension).DataQueueSize as SIZE_T
        {
            ReadSize = (*(*ClassDeviceExtension).DriverExtension).DataQueueSize as SIZE_T
                - (*ClassDeviceExtension).InputCount;
        } else {
            ReadSize = InputCount;
        }

        RtlCopyMemory(
            (*ClassDeviceExtension).PortData.add((*ClassDeviceExtension).InputCount as usize) as *mut core::ffi::c_void,
            DataStart as *const core::ffi::c_void,
            core::mem::size_of::<KEYBOARD_INPUT_DATA>() * ReadSize as usize,
        );

        (*ClassDeviceExtension).InputCount += ReadSize;

        *ConsumedCount += ReadSize as ULONG;

        if !(*ClassDeviceExtension).PendingIrp.is_null() {
            HandleReadIrp(ClassDeviceObject, (*ClassDeviceExtension).PendingIrp, 0u8);
        }
    }
    KeReleaseSpinLock(
        core::ptr::addr_of_mut!((*ClassDeviceExtension).SpinLock),
        OldIrql,
    );

    1u8
}

unsafe fn ConnectPortDriver(
	PortDO: PDEVICE_OBJECT,
	ClassDO: PDEVICE_OBJECT,
) -> NTSTATUS {
	let mut Event: KEVENT = core::mem::zeroed();
	let Irp: PIRP;
	let mut IoStatus: _IO_STATUS_BLOCK = core::mem::zeroed();
	let mut ConnectData: CONNECT_DATA = core::mem::zeroed();
	let mut Status: NTSTATUS;

	KeInitializeEvent(&mut Event, _EVENT_TYPE_NotificationEvent, 0u8);

	ConnectData.ClassDeviceObject = ClassDO;
	ConnectData.ClassService = ClassCallback as *const () as *mut c_void;

	Irp = IoBuildDeviceIoControlRequest(
		IOCTL_INTERNAL_KEYBOARD_CONNECT,
		PortDO,
		core::ptr::addr_of_mut!(ConnectData) as *mut core::ffi::c_void,
		core::mem::size_of::<CONNECT_DATA>() as u32,
		core::ptr::null_mut(),
		0,
		1u8,
		&mut Event,
		&mut IoStatus,
	);
	if Irp.is_null() {
		return STATUS_INSUFFICIENT_RESOURCES;
	}

	Status = IoCallDriver(PortDO, Irp);

	if Status == STATUS_PENDING {
		KeWaitForSingleObject(
			&mut Event as *mut _ as *mut core::ffi::c_void,
			_KWAIT_REASON_Suspended,
			0i32 as i8,
			0u8,
			core::ptr::null_mut(),
		);
	} else {
		IoStatus.__bindgen_anon_1.Status = Status;
	}

	if NT_SUCCESS(IoStatus.__bindgen_anon_1.Status) {
		ObReferenceObject(PortDO as *mut core::ffi::c_void);
		ExInterlockedInsertTailList(
			core::ptr::addr_of_mut!(
				(*((*ClassDO).DeviceExtension as PCLASS_DEVICE_EXTENSION)).ListHead
			),
			core::ptr::addr_of_mut!(
				(*((*PortDO).DeviceExtension as PPORT_DEVICE_EXTENSION)).ListEntry
			),
			core::ptr::addr_of_mut!(
				(*((*ClassDO).DeviceExtension as PCLASS_DEVICE_EXTENSION)).ListSpinLock
			) as *mut _,
		);
		if (*ClassDO).StackSize <= (*PortDO).StackSize {
			(*ClassDO).StackSize = (*PortDO).StackSize + 1;
		}
	}

	IoStatus.__bindgen_anon_1.Status
}

unsafe fn DestroyPortDriver(PortDO: PDEVICE_OBJECT) {
    let DeviceExtension: PPORT_DEVICE_EXTENSION;
    let ClassDeviceExtension: PCLASS_DEVICE_EXTENSION;
    let DriverExtension: PCLASS_DRIVER_EXTENSION;
    let mut Event: KEVENT = core::mem::zeroed();
    let Irp: PIRP;
    let mut IoStatus: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut OldIrql: KIRQL = 0;
    let Status: NTSTATUS;

    DeviceExtension = (*PortDO).DeviceExtension as PPORT_DEVICE_EXTENSION;
    ClassDeviceExtension = (*(*DeviceExtension).ClassDO).DeviceExtension as PCLASS_DEVICE_EXTENSION;
    DriverExtension = IoGetDriverObjectExtension(
        (*PortDO).DriverObject as *mut _,
        (*PortDO).DriverObject as *mut _,
    ) as PCLASS_DRIVER_EXTENSION;

    /* Send IOCTL_INTERNAL_*_DISCONNECT */
    KeInitializeEvent(&mut Event, _EVENT_TYPE_NotificationEvent, 0u8);
    Irp = IoBuildDeviceIoControlRequest(
        IOCTL_INTERNAL_KEYBOARD_DISCONNECT,
        PortDO,
        core::ptr::null_mut(),
        0,
        core::ptr::null_mut(),
        0,
        1u8,
        &mut Event,
        &mut IoStatus,
    );
    if !Irp.is_null() {
        Status = IoCallDriver(PortDO, Irp);
        if Status == STATUS_PENDING {
            KeWaitForSingleObject(
                &mut Event as *mut _ as *mut _,
                _KWAIT_REASON_Suspended,
                0i32 as i8,
                0u8,
                core::ptr::null_mut(),
            );
        }
    }

    /* Remove from ClassDeviceExtension->ListHead list */
    KeAcquireSpinLock(&mut (*ClassDeviceExtension).ListSpinLock, &mut OldIrql);
    RemoveEntryList(&mut (*DeviceExtension).ListEntry);
    KeReleaseSpinLock(&mut (*ClassDeviceExtension).ListSpinLock, OldIrql);

    /* Remove entry from HKEY_LOCAL_MACHINE\HARDWARE\DEVICEMAP\[DeviceBaseName] */
    RtlDeleteRegistryValue(
        RTL_REGISTRY_DEVICEMAP,
        (*DriverExtension).DeviceBaseName.Buffer,
        (*ClassDeviceExtension).DeviceName,
    );

    if !(*DeviceExtension).LowerDevice.is_null() {
        IoDetachDevice((*DeviceExtension).LowerDevice);
    }
    ObDereferenceObject(PortDO as *mut _);

    if (*DriverExtension).ConnectMultiplePorts == 0 && !(*DeviceExtension).ClassDO.is_null() {
        ExFreePoolWithTag((*ClassDeviceExtension).PortData as *mut _, CLASS_TAG);
        ExFreePoolWithTag((*ClassDeviceExtension).DeviceName as *mut _, CLASS_TAG);
        IoDeleteDevice((*DeviceExtension).ClassDO);
    }

    IoDeleteDevice(PortDO);
}

unsafe extern "system" fn ClassAddDevice(
	DriverObject: PDRIVER_OBJECT,
	Pdo: PDEVICE_OBJECT,
) -> NTSTATUS {
	let DriverExtension: PCLASS_DRIVER_EXTENSION;
	let mut Fdo: PDEVICE_OBJECT = core::ptr::null_mut();
	let mut DeviceExtension: PPORT_DEVICE_EXTENSION = core::ptr::null_mut();
	let mut Status: NTSTATUS;

	DriverExtension = IoGetDriverObjectExtension(DriverObject, DriverObject as *mut _) as PCLASS_DRIVER_EXTENSION;

	if Pdo.is_null() {
		/* We may get a NULL Pdo at the first call as we're a legacy driver. Ignore it */
		return STATUS_SUCCESS;
	}

	'cleanup: {
		/* Create new device object */
		Status = IoCreateDevice(
			DriverObject,
			core::mem::size_of::<PORT_DEVICE_EXTENSION>() as ULONG,
			core::ptr::null_mut(),
			(*Pdo).DeviceType,
			if (*Pdo).Characteristics & FILE_DEVICE_SECURE_OPEN != 0 { FILE_DEVICE_SECURE_OPEN } else { 0 },
			0u8,
			&mut Fdo,
		);
		if !NT_SUCCESS(Status) {
			break 'cleanup;
		}
		IoSetStartIoAttributes(Fdo, 1u8, 1u8);

		DeviceExtension = (*Fdo).DeviceExtension as PPORT_DEVICE_EXTENSION;
		RtlZeroMemory(DeviceExtension as *mut _, core::mem::size_of::<PORT_DEVICE_EXTENSION>());
		(*DeviceExtension).Common.IsClassDO = 0u8;
		(*DeviceExtension).DeviceObject = Fdo;
		(*DeviceExtension).PnpState = dsStopped;
		Status = IoAttachDeviceToDeviceStackSafe(Fdo, Pdo, &mut (*DeviceExtension).LowerDevice);
		if !NT_SUCCESS(Status) {
			break 'cleanup;
		}
		if (*(*DeviceExtension).LowerDevice).Flags & DO_POWER_PAGABLE != 0 {
			(*Fdo).Flags |= DO_POWER_PAGABLE;
		}
		if (*(*DeviceExtension).LowerDevice).Flags & DO_BUFFERED_IO != 0 {
			(*Fdo).Flags |= DO_BUFFERED_IO;
		}
		if (*(*DeviceExtension).LowerDevice).Flags & DO_DIRECT_IO != 0 {
			(*Fdo).Flags |= DO_DIRECT_IO;
		}

		if (*DriverExtension).ConnectMultiplePorts != 0 {
			(*DeviceExtension).ClassDO = (*DriverExtension).MainClassDeviceObject;
		} else {
			/* We need a new class device object for this Fdo */
			Status = CreateClassDeviceObject(
				DriverObject,
				&mut (*DeviceExtension).ClassDO,
			);
			if !NT_SUCCESS(Status) {
				break 'cleanup;
			}
		}
		Status = ConnectPortDriver(Fdo, (*DeviceExtension).ClassDO);
		if !NT_SUCCESS(Status) {
			break 'cleanup;
		}
		(*Fdo).Flags &= !DO_DEVICE_INITIALIZING;

		/* Register interface ; ignore the error (if any) as having
		 * a registered interface is not so important... */
		Status = IoRegisterDeviceInterface(
			Pdo,
			core::ptr::addr_of!(GUID_DEVINTERFACE_KEYBOARD) as *const _GUID,
			core::ptr::null_mut(),
			&mut (*DeviceExtension).InterfaceName,
		);
		if !NT_SUCCESS(Status) {
			(*DeviceExtension).InterfaceName.Length = 0;
		}

		return STATUS_SUCCESS;
	}

	if !Fdo.is_null() {
		DestroyPortDriver(Fdo);
	}
	Status
}

unsafe extern "system" fn ClassCancelRoutine(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) {
    let ClassDeviceExtension =
        (*DeviceObject).DeviceExtension as PCLASS_DEVICE_EXTENSION;
    let mut OldIrql: KIRQL = 0;
    let mut wasQueued: u8 = 0u8;

    IoReleaseCancelSpinLock((*Irp).CancelIrql);

    KeAcquireSpinLock(
        core::ptr::addr_of_mut!((*ClassDeviceExtension).SpinLock),
        &mut OldIrql,
    );

    if (*ClassDeviceExtension).PendingIrp == Irp {
        (*ClassDeviceExtension).PendingIrp = core::ptr::null_mut();
        wasQueued = 1u8;
    }
    KeReleaseSpinLock(
        core::ptr::addr_of_mut!((*ClassDeviceExtension).SpinLock),
        OldIrql,
    );

    if wasQueued != 0 {
        (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_CANCELLED;
        (*Irp).IoStatus.Information = 0;
        IoCompleteRequest(Irp, IO_NO_INCREMENT as CCHAR);
    }
}

unsafe fn HandleReadIrp(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
    IsInStartIo: BOOLEAN,
) -> NTSTATUS {
    let DeviceExtension = (*DeviceObject).DeviceExtension as PCLASS_DEVICE_EXTENSION;
    let mut Status: NTSTATUS;
    let mut OldIrql: KIRQL = 0;

    if (*DeviceExtension).InputCount > 0 {
        let NumberOfEntries: SIZE_T;

        let Available = (*DeviceExtension).InputCount;
        let Requested = (*IoGetCurrentIrpStackLocation(Irp)).Parameters.Read.Length as SIZE_T
            / core::mem::size_of::<KEYBOARD_INPUT_DATA>();
        NumberOfEntries = if Available < Requested { Available } else { Requested };

        Status = FillEntries(
            DeviceObject,
            Irp,
            (*DeviceExtension).PortData,
            NumberOfEntries,
        );

        if NT_SUCCESS(Status) {
            if (*DeviceExtension).InputCount > NumberOfEntries {
                core::ptr::copy(
                    (*DeviceExtension).PortData.add(NumberOfEntries),
                    (*DeviceExtension).PortData,
                    ((*DeviceExtension).InputCount - NumberOfEntries)
                        * core::mem::size_of::<KEYBOARD_INPUT_DATA>(),
                );
            }

            (*DeviceExtension).InputCount -= NumberOfEntries;

            (*Irp).IoStatus.Information =
                (NumberOfEntries * core::mem::size_of::<KEYBOARD_INPUT_DATA>()) as _;
        }

        (*Irp).IoStatus.__bindgen_anon_1.Status = Status;

        IoSetCancelRoutine(Irp, None);
        IoCompleteRequest(Irp, IO_KEYBOARD_INCREMENT as CCHAR);
        (*DeviceExtension).PendingIrp = core::ptr::null_mut();
    } else {
        IoAcquireCancelSpinLock(&mut OldIrql);
        if (*Irp).Cancel != 0 {
            (*DeviceExtension).PendingIrp = core::ptr::null_mut();
            Status = STATUS_CANCELLED;
        } else {
            IoMarkIrpPending(Irp);
            (*DeviceExtension).PendingIrp = Irp;
            IoSetCancelRoutine(Irp, Some(core::mem::transmute(ClassCancelRoutine as *const ())));
            Status = STATUS_PENDING;
        }
        IoReleaseCancelSpinLock(OldIrql);
    }
    Status
}

unsafe extern "system" fn ClassPnp(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    let DeviceExtension: PPORT_DEVICE_EXTENSION = (*DeviceObject).DeviceExtension as PPORT_DEVICE_EXTENSION;
    let IrpSp: PIO_STACK_LOCATION = IoGetCurrentIrpStackLocation(Irp);
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut Iosb: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut Status: NTSTATUS;

    match (*IrpSp).MinorFunction {
        IRP_MN_START_DEVICE => {
            if IoForwardIrpSynchronously((*DeviceExtension).LowerDevice, Irp) != 0 {
                Status = (*Irp).IoStatus.__bindgen_anon_1.Status;
            } else {
                Status = STATUS_UNSUCCESSFUL;
            }

            if NT_SUCCESS(Status) {
                ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as u32;
                ObjectAttributes.RootDirectory = core::ptr::null_mut();
                ObjectAttributes.Attributes = OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE;
                ObjectAttributes.ObjectName = core::ptr::addr_of_mut!((*DeviceExtension).InterfaceName);
                ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
                ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

                Status = ZwOpenFile(
                    core::ptr::addr_of_mut!((*DeviceExtension).FileHandle),
                    FILE_READ_DATA,
                    &mut ObjectAttributes,
                    &mut Iosb,
                    0,
                    0,
                );
                if !NT_SUCCESS(Status) {
                    (*DeviceExtension).FileHandle = core::ptr::null_mut();
                }
            } else {
                (*DeviceExtension).FileHandle = core::ptr::null_mut();
            }
            if (*DeviceExtension).InterfaceName.Length != 0 {
                IoSetDeviceInterfaceState(core::ptr::addr_of_mut!((*DeviceExtension).InterfaceName), 1u8);
            }
            (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
            IoCompleteRequest(Irp, IO_NO_INCREMENT as CCHAR);
            return Status;
        }

        IRP_MN_STOP_DEVICE => {
            if !(*DeviceExtension).FileHandle.is_null() {
                ZwClose((*DeviceExtension).FileHandle);
                (*DeviceExtension).FileHandle = core::ptr::null_mut();
            }
            Status = STATUS_SUCCESS;
        }

        IRP_MN_REMOVE_DEVICE => {
            if (*DeviceExtension).InterfaceName.Length != 0 {
                IoSetDeviceInterfaceState(core::ptr::addr_of_mut!((*DeviceExtension).InterfaceName), 0u8);
            }
            if !(*DeviceExtension).FileHandle.is_null() {
                ZwClose((*DeviceExtension).FileHandle);
                (*DeviceExtension).FileHandle = core::ptr::null_mut();
            }
            IoSkipCurrentIrpStackLocation(Irp);
            Status = IoCallDriver((*DeviceExtension).LowerDevice, Irp);
            DestroyPortDriver(DeviceObject);
            return Status;
        }

        _ => {
            Status = (*Irp).IoStatus.__bindgen_anon_1.Status;
        }
    }

    (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
    if NT_SUCCESS(Status) || Status == STATUS_NOT_SUPPORTED {
        IoSkipCurrentIrpStackLocation(Irp);
        IoCallDriver((*DeviceExtension).LowerDevice, Irp)
    } else {
        IoCompleteRequest(Irp, IO_NO_INCREMENT as CCHAR);
        Status
    }
}

unsafe extern "system" fn ClassStartIo(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) {
    let DeviceExtension = (*DeviceObject).DeviceExtension as PCLASS_DEVICE_EXTENSION;
    let mut OldIrql: KIRQL = 0;

    KeAcquireSpinLock(core::ptr::addr_of_mut!((*DeviceExtension).SpinLock), &mut OldIrql);
    HandleReadIrp(DeviceObject, Irp, 1u8);
    KeReleaseSpinLock(core::ptr::addr_of_mut!((*DeviceExtension).SpinLock), OldIrql);
}

unsafe extern "system" fn SearchForLegacyDrivers(
	DriverObject: PDRIVER_OBJECT,
	Context: *mut core::ffi::c_void,
	Count: u32,
) {
	let mut DeviceMapKeyU: _UNICODE_STRING = {
		let s = b"\\REGISTRY\\MACHINE\\HARDWARE\\DEVICEMAP\0";
		const WIDE: [u16; 37] = {
			const SRC: &[u8] = b"\\REGISTRY\\MACHINE\\HARDWARE\\DEVICEMAP\0";
			let mut arr = [0u16; 37];
			let mut i = 0;
			while i < SRC.len() {
				arr[i] = SRC[i] as u16;
				i += 1;
			}
			arr
		};
		let wide: &'static [u16] = &WIDE;
		let _ = s;
		_UNICODE_STRING {
			Length: ((wide.len() - 1) * 2) as u16,
			MaximumLength: (wide.len() * 2) as u16,
			Buffer: wide.as_ptr() as *mut u16,
		}
	};

	let DriverExtension: PCLASS_DRIVER_EXTENSION;
	let mut PortBaseName: _UNICODE_STRING = _UNICODE_STRING { Length: 0, MaximumLength: 0, Buffer: core::ptr::null_mut() };
	let mut KeyValueInformation: PKEY_VALUE_BASIC_INFORMATION = core::ptr::null_mut();
	let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
	let mut hDeviceMapKey: *mut core::ffi::c_void = (-1isize) as *mut core::ffi::c_void;
	let mut hPortKey: *mut core::ffi::c_void = (-1isize) as *mut core::ffi::c_void;
	let mut Index_local: u32 = 0;
	let Size: u32;
	let mut ResultLength: u32 = 0;
	let mut Status: NTSTATUS;

	'cleanup: {
		if Count != 1 {
			return;
		}
		DriverExtension = Context as PCLASS_DRIVER_EXTENSION;

		/* Create port base name, by replacing Class by Port at the end of the class base name */
		Status = DuplicateUnicodeString(
			RTL_DUPLICATE_UNICODE_STRING_NULL_TERMINATE,
			core::ptr::addr_of_mut!((*DriverExtension).DeviceBaseName),
			core::ptr::addr_of_mut!(PortBaseName),
		);
		if !NT_SUCCESS(Status) {
			break 'cleanup;
		}
		PortBaseName.Length -= (("Class".len() + 1) * 2 - 2) as u16;
		{
			let wide: &'static [u16] = &[
				'P' as u16, 'o' as u16, 'r' as u16, 't' as u16, 0u16,
			];
			let mut PortStr = _UNICODE_STRING {
				Length: 8,
				MaximumLength: 10,
				Buffer: wide.as_ptr() as *mut u16,
			};
			let _ = &mut PortStr;
			RtlAppendUnicodeToString(core::ptr::addr_of_mut!(PortBaseName), wide.as_ptr());
		}

		/* Allocate memory */
		Size = (core::mem::size_of::<KEY_VALUE_BASIC_INFORMATION>() + 260) as u32;
		KeyValueInformation = ExAllocatePoolWithTag(PagedPool, Size as usize, CLASS_TAG) as PKEY_VALUE_BASIC_INFORMATION;
		if KeyValueInformation.is_null() {
			Status = STATUS_NO_MEMORY;
			break 'cleanup;
		}

		/* Open HKEY_LOCAL_MACHINE\HARDWARE\DEVICEMAP */
		ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as u32;
		ObjectAttributes.RootDirectory = core::ptr::null_mut();
		ObjectAttributes.Attributes = OBJ_KERNEL_HANDLE | OBJ_CASE_INSENSITIVE;
		ObjectAttributes.ObjectName = core::ptr::addr_of_mut!(DeviceMapKeyU);
		ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
		ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();
		Status = ZwOpenKey(core::ptr::addr_of_mut!(hDeviceMapKey), 0, core::ptr::addr_of_mut!(ObjectAttributes));
		if Status == STATUS_OBJECT_NAME_NOT_FOUND {
			Status = STATUS_SUCCESS;
			break 'cleanup;
		} else if !NT_SUCCESS(Status) {
			break 'cleanup;
		}

		/* Open sub key */
		ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as u32;
		ObjectAttributes.RootDirectory = hDeviceMapKey;
		ObjectAttributes.Attributes = OBJ_KERNEL_HANDLE | OBJ_CASE_INSENSITIVE;
		ObjectAttributes.ObjectName = core::ptr::addr_of_mut!(PortBaseName);
		ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
		ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();
		Status = ZwOpenKey(core::ptr::addr_of_mut!(hPortKey), KEY_QUERY_VALUE, core::ptr::addr_of_mut!(ObjectAttributes));
		if Status == STATUS_OBJECT_NAME_NOT_FOUND {
			Status = STATUS_SUCCESS;
			break 'cleanup;
		} else if !NT_SUCCESS(Status) {
			break 'cleanup;
		}

		/* Read each value name */
		while ZwEnumerateValueKey(
			hPortKey,
			{ let i = Index_local; Index_local += 1; i },
			KeyValueBasicInformation,
			KeyValueInformation as *mut core::ffi::c_void,
			Size,
			core::ptr::addr_of_mut!(ResultLength),
		) == STATUS_SUCCESS
		{
			let mut PortName: _UNICODE_STRING = core::mem::zeroed();
			let mut PortDeviceObject: PDEVICE_OBJECT = core::ptr::null_mut();
			let mut FileObject: PFILE_OBJECT = core::ptr::null_mut();

			PortName.Length = (*KeyValueInformation).NameLength as u16;
			PortName.MaximumLength = (*KeyValueInformation).NameLength as u16;
			PortName.Buffer = core::ptr::addr_of_mut!((*KeyValueInformation).Name[0]);

			/* Open the device object pointer */
			Status = IoGetDeviceObjectPointer(
				core::ptr::addr_of_mut!(PortName),
				FILE_READ_ATTRIBUTES,
				core::ptr::addr_of_mut!(FileObject),
				core::ptr::addr_of_mut!(PortDeviceObject),
			);
			if !NT_SUCCESS(Status) {
				continue;
			}

			Status = ClassAddDevice(DriverObject, PortDeviceObject);
			let _ = Status;

			ObDereferenceObject(FileObject as *mut core::ffi::c_void);
		}
	}

	if !KeyValueInformation.is_null() {
		ExFreePoolWithTag(KeyValueInformation as *mut core::ffi::c_void, CLASS_TAG);
	}
	if hDeviceMapKey != (-1isize) as *mut core::ffi::c_void {
		ZwClose(hDeviceMapKey);
	}
	if hPortKey != (-1isize) as *mut core::ffi::c_void {
		ZwClose(hPortKey);
	}
}

#[no_mangle]
pub unsafe extern "system" fn DriverEntry(
    DriverObject: PDRIVER_OBJECT,
    RegistryPath: PUNICODE_STRING,
) -> NTSTATUS {
    let mut DriverExtension: PCLASS_DRIVER_EXTENSION = core::ptr::null_mut();
    let mut Status: NTSTATUS;

    Status = IoAllocateDriverObjectExtension(
        DriverObject,
        DriverObject as *mut _,
        core::mem::size_of::<CLASS_DRIVER_EXTENSION>() as ULONG,
        &mut DriverExtension as *mut PCLASS_DRIVER_EXTENSION as *mut PVOID,
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }
    RtlZeroMemory(
        DriverExtension as PVOID,
        core::mem::size_of::<CLASS_DRIVER_EXTENSION>() as SIZE_T,
    );

    Status = DuplicateUnicodeString(
        RTL_DUPLICATE_UNICODE_STRING_NULL_TERMINATE,
        RegistryPath,
        core::ptr::addr_of_mut!((*DriverExtension).RegistryPath),
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }

    Status = ReadRegistryEntries(RegistryPath, DriverExtension);
    if !NT_SUCCESS(Status) {
        return Status;
    }

    if (*DriverExtension).ConnectMultiplePorts == 1 {
        Status = CreateClassDeviceObject(
            DriverObject,
            core::ptr::addr_of_mut!((*DriverExtension).MainClassDeviceObject),
        );
        if !NT_SUCCESS(Status) {
            return Status;
        }
    }

    (*(*DriverObject).DriverExtension).AddDevice = Some(core::mem::transmute(ClassAddDevice as *const ()));
    (*DriverObject).DriverUnload = Some(core::mem::transmute(DriverUnload as *const ()));

    (*DriverObject).MajorFunction[IRP_MJ_CREATE as usize] = Some(core::mem::transmute(ClassCreate as *const ()));
    (*DriverObject).MajorFunction[IRP_MJ_CLOSE as usize] = Some(core::mem::transmute(ClassClose as *const ()));
    (*DriverObject).MajorFunction[IRP_MJ_CLEANUP as usize] = Some(core::mem::transmute(ClassCleanup as *const ()));
    (*DriverObject).MajorFunction[IRP_MJ_READ as usize] = Some(core::mem::transmute(ClassRead as *const ()));
    (*DriverObject).MajorFunction[IRP_MJ_POWER as usize] = Some(core::mem::transmute(ClassPower as *const ()));
    (*DriverObject).MajorFunction[IRP_MJ_PNP as usize] = Some(core::mem::transmute(ClassPnp as *const ()));
    (*DriverObject).MajorFunction[IRP_MJ_DEVICE_CONTROL as usize] = Some(core::mem::transmute(ClassDeviceControl as *const ()));
    (*DriverObject).MajorFunction[IRP_MJ_INTERNAL_DEVICE_CONTROL as usize] =
        Some(core::mem::transmute(ForwardIrpAndForget as *const ()));
    (*DriverObject).DriverStartIo = Some(core::mem::transmute(ClassStartIo as *const ()));

    IoRegisterDriverReinitialization(
        DriverObject,
        Some(core::mem::transmute(SearchForLegacyDrivers as *const ())),
        DriverExtension as PVOID,
    );

    STATUS_SUCCESS
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:       See COPYING in the top level directory
//  * PROJECT:         ReactOS Keyboard class driver
//  * FILE:            drivers/kbdclass/kbdclass.c
//  * PURPOSE:         Keyboard class driver
//  *
//  * PROGRAMMERS:     Hervé Poussineau (hpoussin@reactos.org)
//  */
//
// #include "kbdclass.h"
//
// #include <stdio.h>
// #include <pseh/pseh2.h>
// #include <kbdmou.h>
// #include <debug.h>
//
// static DRIVER_UNLOAD DriverUnload;
// static DRIVER_DISPATCH ClassCreate;
// static DRIVER_DISPATCH ClassClose;
// static DRIVER_DISPATCH ClassCleanup;
// static DRIVER_DISPATCH ClassRead;
// static DRIVER_DISPATCH ClassDeviceControl;
// static DRIVER_DISPATCH ClassPower;
// static DRIVER_ADD_DEVICE ClassAddDevice;
// static DRIVER_STARTIO ClassStartIo;
// static DRIVER_CANCEL ClassCancelRoutine;
// static NTSTATUS
// HandleReadIrp(
// 	IN PDEVICE_OBJECT DeviceObject,
// 	IN PIRP Irp,
// 	BOOLEAN IsInStartIo);
//
// static VOID NTAPI
// DriverUnload(IN PDRIVER_OBJECT DriverObject)
// {
// 	// nothing to do here yet
// }
//
// static NTSTATUS NTAPI
// ClassCreate(
// 	IN PDEVICE_OBJECT DeviceObject,
// 	IN PIRP Irp)
// {
// 	TRACE_(CLASS_NAME, "IRP_MJ_CREATE\n");
//
// 	if (!((PCOMMON_DEVICE_EXTENSION)DeviceObject->DeviceExtension)->IsClassDO)
// 		return ForwardIrpAndForget(DeviceObject, Irp);
//
// 	/* FIXME: open all associated Port devices */
// 	Irp->IoStatus.Status = STATUS_SUCCESS;
// 	Irp->IoStatus.Information = 0;
// 	IoCompleteRequest(Irp, IO_NO_INCREMENT);
// 	return STATUS_SUCCESS;
// }
//
// static NTSTATUS NTAPI
// ClassClose(
// 	IN PDEVICE_OBJECT DeviceObject,
// 	IN PIRP Irp)
// {
// 	TRACE_(CLASS_NAME, "IRP_MJ_CLOSE\n");
//
// 	if (!((PCOMMON_DEVICE_EXTENSION)DeviceObject->DeviceExtension)->IsClassDO)
// 		return ForwardIrpAndForget(DeviceObject, Irp);
//
// 	/* FIXME: close all associated Port devices */
// 	Irp->IoStatus.Status = STATUS_SUCCESS;
// 	Irp->IoStatus.Information = 0;
// 	IoCompleteRequest(Irp, IO_NO_INCREMENT);
// 	return STATUS_SUCCESS;
// }
//
// static NTSTATUS NTAPI
// ClassCleanup(
// 	IN PDEVICE_OBJECT DeviceObject,
// 	IN PIRP Irp)
// {
// 	TRACE_(CLASS_NAME, "IRP_MJ_CLEANUP\n");
//
// 	if (!((PCOMMON_DEVICE_EXTENSION)DeviceObject->DeviceExtension)->IsClassDO)
// 		return ForwardIrpAndForget(DeviceObject, Irp);
//
// 	/* FIXME: cleanup all associated Port devices */
// 	Irp->IoStatus.Status = STATUS_SUCCESS;
// 	Irp->IoStatus.Information = 0;
// 	IoCompleteRequest(Irp, IO_NO_INCREMENT);
// 	return STATUS_SUCCESS;
// }
//
// static NTSTATUS NTAPI
// ClassRead(
// 	IN PDEVICE_OBJECT DeviceObject,
// 	IN PIRP Irp)
// {
// 	PCLASS_DEVICE_EXTENSION DeviceExtension = DeviceObject->DeviceExtension;
// 	KIRQL OldIrql;
// 	NTSTATUS Status;
//
// 	TRACE_(CLASS_NAME, "IRP_MJ_READ\n");
//
// 	ASSERT(DeviceExtension->Common.IsClassDO);
//
// 	if (!((PCOMMON_DEVICE_EXTENSION)DeviceObject->DeviceExtension)->IsClassDO)
// 		return ForwardIrpAndForget(DeviceObject, Irp);
//
// 	if (IoGetCurrentIrpStackLocation(Irp)->Parameters.Read.Length < sizeof(KEYBOARD_INPUT_DATA))
// 	{
// 		Irp->IoStatus.Status = STATUS_BUFFER_TOO_SMALL;
// 		Irp->IoStatus.Information = 0;
// 		IoCompleteRequest(Irp, IO_NO_INCREMENT);
//
// 		return STATUS_BUFFER_TOO_SMALL;
// 	}
//
// 	KeAcquireSpinLock(&DeviceExtension->SpinLock, &OldIrql);
// 	Status = HandleReadIrp(DeviceObject, Irp, FALSE);
// 	KeReleaseSpinLock(&DeviceExtension->SpinLock, OldIrql);
// 	return Status;
// }
//
// static NTSTATUS NTAPI
// ClassDeviceControl(
// 	IN PDEVICE_OBJECT DeviceObject,
// 	IN PIRP Irp)
// {
// 	//PCLASS_DEVICE_EXTENSION DeviceExtension;
// 	NTSTATUS Status = STATUS_NOT_SUPPORTED;
//
// 	TRACE_(CLASS_NAME, "IRP_MJ_DEVICE_CONTROL\n");
//
// 	if (!((PCOMMON_DEVICE_EXTENSION)DeviceObject->DeviceExtension)->IsClassDO)
// 		return ForwardIrpAndForget(DeviceObject, Irp);
//
// 	//DeviceExtension = (PCLASS_DEVICE_EXTENSION)DeviceObject->DeviceExtension;
//
// 	switch (IoGetCurrentIrpStackLocation(Irp)->Parameters.DeviceIoControl.IoControlCode)
// 	{
// 		case IOCTL_KEYBOARD_QUERY_ATTRIBUTES:
// 		case IOCTL_KEYBOARD_QUERY_INDICATOR_TRANSLATION:
// 		case IOCTL_KEYBOARD_QUERY_INDICATORS:
// 		case IOCTL_KEYBOARD_QUERY_TYPEMATIC:
// 		{
// 			/* FIXME: We hope that all devices will return the same result.
// 			 * Ask only the first one */
// 			PLIST_ENTRY Head = &((PCLASS_DEVICE_EXTENSION)DeviceObject->DeviceExtension)->ListHead;
// 			if (Head->Flink != Head)
// 			{
// 				/* We have at least one device */
// 				PPORT_DEVICE_EXTENSION DevExt = CONTAINING_RECORD(Head->Flink, PORT_DEVICE_EXTENSION, ListEntry);
// 				IoGetCurrentIrpStackLocation(Irp)->MajorFunction = IRP_MJ_INTERNAL_DEVICE_CONTROL;
// 				IoSkipCurrentIrpStackLocation(Irp);
// 				return IoCallDriver(DevExt->DeviceObject, Irp);
// 			}
// 			break;
// 		}
// 		case IOCTL_KEYBOARD_SET_INDICATORS:
// 		case IOCTL_KEYBOARD_SET_TYPEMATIC: /* not in MSDN, would seem logical */
// 		{
// 			/* Send it to all associated Port devices */
// 			PLIST_ENTRY Head = &((PCLASS_DEVICE_EXTENSION)DeviceObject->DeviceExtension)->ListHead;
// 			PLIST_ENTRY Entry = Head->Flink;
// 			Status = STATUS_SUCCESS;
// 			while (Entry != Head)
// 			{
// 				PPORT_DEVICE_EXTENSION DevExt = CONTAINING_RECORD(Entry, PORT_DEVICE_EXTENSION, ListEntry);
//
// 				IoGetCurrentIrpStackLocation(Irp)->MajorFunction = IRP_MJ_INTERNAL_DEVICE_CONTROL;
//
// 				if (IoForwardIrpSynchronously(DevExt->LowerDevice, Irp))
// 				{
// 					if (!NT_SUCCESS(Irp->IoStatus.Status))
// 					{
// 						Status = Irp->IoStatus.Status;
// 					}
// 				}
// 				else
// 				{
// 					Status = STATUS_UNSUCCESSFUL;
// 				}
// 				
// 				Entry = Entry->Flink;
// 			}
// 			break;
// 		}
// 		default:
// 			WARN_(CLASS_NAME, "IRP_MJ_DEVICE_CONTROL / unknown I/O control code 0x%lx\n",
// 				IoGetCurrentIrpStackLocation(Irp)->Parameters.DeviceIoControl.IoControlCode);
// 			ASSERT(FALSE);
// 			break;
// 	}
//
// 	Irp->IoStatus.Status = Status;
// 	Irp->IoStatus.Information = 0;
// 	IoCompleteRequest(Irp, IO_NO_INCREMENT);
//
// 	return Status;
// }
//
// static NTSTATUS NTAPI
// ClassPower(
// 	IN PDEVICE_OBJECT DeviceObject,
// 	IN PIRP Irp)
// {
// 	NTSTATUS Status;
// 	PPORT_DEVICE_EXTENSION DeviceExtension;
//
// 	DeviceExtension = DeviceObject->DeviceExtension;
// 	if (!DeviceExtension->Common.IsClassDO)
// 	{
// 		/* Forward port DO IRPs to lower device */
// 		PoStartNextPowerIrp(Irp);
// 		IoSkipCurrentIrpStackLocation(Irp);
// 		return PoCallDriver(DeviceExtension->LowerDevice, Irp);
// 	}
//
// 	switch (IoGetCurrentIrpStackLocation(Irp)->MinorFunction)
// 	{
// 		case IRP_MN_SET_POWER:
// 		case IRP_MN_QUERY_POWER:
// 			Irp->IoStatus.Status = STATUS_SUCCESS;
// 			break;
// 	}
// 	Status = Irp->IoStatus.Status;
// 	PoStartNextPowerIrp(Irp);
// 	IoCompleteRequest(Irp, IO_NO_INCREMENT);
// 	return Status;
// }
//
// static NTSTATUS
// ReadRegistryEntries(
// 	IN PUNICODE_STRING RegistryPath,
// 	IN PCLASS_DRIVER_EXTENSION DriverExtension)
// {
// 	UNICODE_STRING ParametersRegistryKey;
// 	RTL_QUERY_REGISTRY_TABLE Parameters[4];
// 	NTSTATUS Status;
//
// 	/* HACK: We don't support multiple devices with this disabled */
// 	ULONG DefaultConnectMultiplePorts = 1;
// 	ULONG DefaultDataQueueSize = 0x64;
// 	PCWSTR DefaultDeviceBaseName = L"KeyboardClass";
//
// 	ParametersRegistryKey.Length = 0;
// 	ParametersRegistryKey.MaximumLength = RegistryPath->Length + sizeof(L"\\Parameters") + sizeof(UNICODE_NULL);
// 	ParametersRegistryKey.Buffer = ExAllocatePoolWithTag(PagedPool, ParametersRegistryKey.MaximumLength, CLASS_TAG);
// 	if (!ParametersRegistryKey.Buffer)
// 	{
// 		WARN_(CLASS_NAME, "ExAllocatePoolWithTag() failed\n");
// 		return STATUS_NO_MEMORY;
// 	}
// 	RtlCopyUnicodeString(&ParametersRegistryKey, RegistryPath);
// 	RtlAppendUnicodeToString(&ParametersRegistryKey, L"\\Parameters");
// 	ParametersRegistryKey.Buffer[ParametersRegistryKey.Length / sizeof(WCHAR)] = UNICODE_NULL;
//
// 	RtlZeroMemory(Parameters, sizeof(Parameters));
//
// 	Parameters[0].Flags = RTL_QUERY_REGISTRY_DIRECT;
// 	Parameters[0].Name = L"ConnectMultiplePorts";
// 	Parameters[0].EntryContext = &DriverExtension->ConnectMultiplePorts;
// 	Parameters[0].DefaultType = REG_DWORD;
// 	Parameters[0].DefaultData = &DefaultConnectMultiplePorts;
// 	Parameters[0].DefaultLength = sizeof(ULONG);
//
// 	Parameters[1].Flags = RTL_QUERY_REGISTRY_DIRECT;
// 	Parameters[1].Name = L"KeyboardDataQueueSize";
// 	Parameters[1].EntryContext = &DriverExtension->DataQueueSize;
// 	Parameters[1].DefaultType = REG_DWORD;
// 	Parameters[1].DefaultData = &DefaultDataQueueSize;
// 	Parameters[1].DefaultLength = sizeof(ULONG);
//
// 	Parameters[2].Flags = RTL_QUERY_REGISTRY_DIRECT;
// 	Parameters[2].Name = L"KeyboardDeviceBaseName";
// 	Parameters[2].EntryContext = &DriverExtension->DeviceBaseName;
// 	Parameters[2].DefaultType = REG_SZ;
// 	Parameters[2].DefaultData = (PVOID)DefaultDeviceBaseName;
// 	Parameters[2].DefaultLength = 0;
//
// 	Status = RtlQueryRegistryValues(
// 		RTL_REGISTRY_ABSOLUTE | RTL_REGISTRY_OPTIONAL,
// 		ParametersRegistryKey.Buffer,
// 		Parameters,
// 		NULL,
// 		NULL);
//
// 	if (NT_SUCCESS(Status))
// 	{
// 		/* Check values */
// 		if (DriverExtension->ConnectMultiplePorts != 0
// 			&& DriverExtension->ConnectMultiplePorts != 1)
// 		{
// 			DriverExtension->ConnectMultiplePorts = DefaultConnectMultiplePorts;
// 		}
// 		if (DriverExtension->DataQueueSize == 0)
// 		{
// 			DriverExtension->DataQueueSize = DefaultDataQueueSize;
// 		}
// 	}
// 	else if (Status == STATUS_OBJECT_NAME_NOT_FOUND)
// 	{
// 		/* Registry path doesn't exist. Set defaults */
// 		DriverExtension->ConnectMultiplePorts = DefaultConnectMultiplePorts;
// 		DriverExtension->DataQueueSize = DefaultDataQueueSize;
// 		if (RtlCreateUnicodeString(&DriverExtension->DeviceBaseName, DefaultDeviceBaseName))
// 			Status = STATUS_SUCCESS;
// 		else
// 			Status = STATUS_NO_MEMORY;
// 	}
//
// 	ExFreePoolWithTag(ParametersRegistryKey.Buffer, CLASS_TAG);
// 	return Status;
// }
//
// static NTSTATUS
// CreateClassDeviceObject(
// 	IN PDRIVER_OBJECT DriverObject,
// 	OUT PDEVICE_OBJECT *ClassDO OPTIONAL)
// {
// 	PCLASS_DRIVER_EXTENSION DriverExtension;
// 	ULONG DeviceId = 0;
// 	ULONG PrefixLength;
// 	UNICODE_STRING DeviceNameU;
// 	PWSTR DeviceIdW = NULL; /* Pointer into DeviceNameU.Buffer */
// 	PDEVICE_OBJECT Fdo;
// 	PCLASS_DEVICE_EXTENSION DeviceExtension;
// 	NTSTATUS Status;
//
// 	TRACE_(CLASS_NAME, "CreateClassDeviceObject(0x%p)\n", DriverObject);
//
// 	/* Create new device object */
// 	DriverExtension = IoGetDriverObjectExtension(DriverObject, DriverObject);
// 	DeviceNameU.Length = 0;
// 	DeviceNameU.MaximumLength =
// 		(USHORT)wcslen(L"\\Device\\") * sizeof(WCHAR) /* "\Device\" */
// 		+ DriverExtension->DeviceBaseName.Length /* "KeyboardClass" */
// 		+ 4 * sizeof(WCHAR)                      /* Id between 0 and 9999 */
// 		+ sizeof(UNICODE_NULL);                  /* Final NULL char */
// 	DeviceNameU.Buffer = ExAllocatePoolWithTag(PagedPool, DeviceNameU.MaximumLength, CLASS_TAG);
// 	if (!DeviceNameU.Buffer)
// 	{
// 		WARN_(CLASS_NAME, "ExAllocatePoolWithTag() failed\n");
// 		return STATUS_NO_MEMORY;
// 	}
// 	Status = RtlAppendUnicodeToString(&DeviceNameU, L"\\Device\\");
// 	if (!NT_SUCCESS(Status))
// 	{
// 		WARN_(CLASS_NAME, "RtlAppendUnicodeToString() failed with status 0x%08lx\n", Status);
// 		goto cleanup;
// 	}
// 	Status = RtlAppendUnicodeStringToString(&DeviceNameU, &DriverExtension->DeviceBaseName);
// 	if (!NT_SUCCESS(Status))
// 	{
// 		WARN_(CLASS_NAME, "RtlAppendUnicodeStringToString() failed with status 0x%08lx\n", Status);
// 		goto cleanup;
// 	}
// 	PrefixLength = DeviceNameU.MaximumLength - 4 * sizeof(WCHAR) - sizeof(UNICODE_NULL);
// 	DeviceIdW = &DeviceNameU.Buffer[PrefixLength / sizeof(WCHAR)];
// 	while (DeviceId < 9999)
// 	{
// 		DeviceNameU.Length = (USHORT)(PrefixLength + swprintf(DeviceIdW, L"%lu", DeviceId) * sizeof(WCHAR));
// 		Status = IoCreateDevice(
// 			DriverObject,
// 			sizeof(CLASS_DEVICE_EXTENSION),
// 			&DeviceNameU,
// 			FILE_DEVICE_KEYBOARD,
// 			FILE_DEVICE_SECURE_OPEN,
// 			FALSE,
// 			&Fdo);
// 		if (NT_SUCCESS(Status))
// 			goto cleanup;
// 		else if (Status != STATUS_OBJECT_NAME_COLLISION)
// 		{
// 			WARN_(CLASS_NAME, "IoCreateDevice() failed with status 0x%08lx\n", Status);
// 			goto cleanup;
// 		}
// 		DeviceId++;
// 	}
// 	WARN_(CLASS_NAME, "Too many devices starting with '\\Device\\%wZ'\n", &DriverExtension->DeviceBaseName);
// 	Status = STATUS_TOO_MANY_NAMES;
// cleanup:
// 	if (!NT_SUCCESS(Status))
// 	{
// 		ExFreePoolWithTag(DeviceNameU.Buffer, CLASS_TAG);
// 		return Status;
// 	}
//
// 	DeviceExtension = (PCLASS_DEVICE_EXTENSION)Fdo->DeviceExtension;
// 	RtlZeroMemory(DeviceExtension, sizeof(CLASS_DEVICE_EXTENSION));
// 	DeviceExtension->Common.IsClassDO = TRUE;
// 	DeviceExtension->DriverExtension = DriverExtension;
// 	InitializeListHead(&DeviceExtension->ListHead);
// 	KeInitializeSpinLock(&DeviceExtension->ListSpinLock);
// 	KeInitializeSpinLock(&DeviceExtension->SpinLock);
// 	DeviceExtension->InputCount = 0;
// 	DeviceExtension->PortData = ExAllocatePoolWithTag(NonPagedPool, DeviceExtension->DriverExtension->DataQueueSize * sizeof(KEYBOARD_INPUT_DATA), CLASS_TAG);
// 	if (!DeviceExtension->PortData)
// 	{
// 		ExFreePoolWithTag(DeviceNameU.Buffer, CLASS_TAG);
// 		return STATUS_NO_MEMORY;
// 	}
// 	DeviceExtension->DeviceName = DeviceNameU.Buffer;
// 	Fdo->Flags |= DO_POWER_PAGABLE;
// 	Fdo->Flags |= DO_BUFFERED_IO; /* FIXME: Why is it needed for 1st stage setup? */
// 	Fdo->Flags &= ~DO_DEVICE_INITIALIZING;
//
// 	/* Add entry entry to HKEY_LOCAL_MACHINE\HARDWARE\DEVICEMAP\[DeviceBaseName] */
// 	RtlWriteRegistryValue(
// 		RTL_REGISTRY_DEVICEMAP,
// 		DriverExtension->DeviceBaseName.Buffer,
// 		DeviceExtension->DeviceName,
// 		REG_SZ,
// 		DriverExtension->RegistryPath.Buffer,
// 		DriverExtension->RegistryPath.MaximumLength);
//
// 	if (ClassDO)
// 		*ClassDO = Fdo;
//
// 	return STATUS_SUCCESS;
// }
//
// static NTSTATUS
// FillEntries(
// 	IN PDEVICE_OBJECT ClassDeviceObject,
// 	IN PIRP Irp,
// 	IN PKEYBOARD_INPUT_DATA DataStart,
// 	IN SIZE_T NumberOfEntries)
// {
// 	NTSTATUS Status = STATUS_SUCCESS;
//
// 	if (ClassDeviceObject->Flags & DO_BUFFERED_IO)
// 	{
// 		RtlCopyMemory(
// 			Irp->AssociatedIrp.SystemBuffer,
// 			DataStart,
// 			NumberOfEntries * sizeof(KEYBOARD_INPUT_DATA));
// 	}
// 	else if (ClassDeviceObject->Flags & DO_DIRECT_IO)
// 	{
// 		PVOID DestAddress = MmGetSystemAddressForMdlSafe(Irp->MdlAddress, NormalPagePriority);
// 		if (DestAddress)
// 		{
// 			RtlCopyMemory(
// 				DestAddress,
// 				DataStart,
// 				NumberOfEntries * sizeof(KEYBOARD_INPUT_DATA));
// 		}
// 		else
// 			Status = STATUS_UNSUCCESSFUL;
// 	}
// 	else
// 	{
// 		_SEH2_TRY
// 		{
// 			RtlCopyMemory(
// 				Irp->UserBuffer,
// 				DataStart,
// 				NumberOfEntries * sizeof(KEYBOARD_INPUT_DATA));
// 		}
// 		_SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
// 		{
// 			Status = _SEH2_GetExceptionCode();
// 		}
// 		_SEH2_END;
// 	}
//
// 	return Status;
// }
//
// static BOOLEAN NTAPI
// ClassCallback(
// 	IN PDEVICE_OBJECT ClassDeviceObject,
// 	IN OUT PKEYBOARD_INPUT_DATA DataStart,
// 	IN PKEYBOARD_INPUT_DATA DataEnd,
// 	IN OUT PULONG ConsumedCount)
// {
// 	PCLASS_DEVICE_EXTENSION ClassDeviceExtension = ClassDeviceObject->DeviceExtension;
// 	KIRQL OldIrql;
// 	SIZE_T InputCount = DataEnd - DataStart;
// 	SIZE_T ReadSize;
//
// 	TRACE_(CLASS_NAME, "ClassCallback()\n");
//
// 	ASSERT(ClassDeviceExtension->Common.IsClassDO);
//
// 	KeAcquireSpinLock(&ClassDeviceExtension->SpinLock, &OldIrql);
// 	if (InputCount > 0)
// 	{
// 		/* Smoke delivery oracle: prove a real keystroke reached the
// 		 * keyboard class driver (i8042prt/usbkbd -> kbdclass), not just
// 		 * "win32k didn't crash". One-shot (InterlockedCompareExchange) so
// 		 * the post-`shell ready` input/mouse storm can't spam serial. The
// 		 * smoke test asserts this marker on i386/amd64 — boot markers
// 		 * structurally cannot, since the raw-input path only runs once a
// 		 * key actually arrives after the desktop is up. */
// 		static volatile LONG g_FirstKbdDeliveredLogged = 0;
// 		if (InterlockedCompareExchange(&g_FirstKbdDeliveredLogged, 1, 0) == 0)
// 			DbgPrint("[KBDCLASS] first key delivered sc=0x%x flags=0x%x cnt=%lu\n",
// 			         DataStart->MakeCode, DataStart->Flags, (ULONG)InputCount);
//
// 		if (ClassDeviceExtension->InputCount + InputCount > ClassDeviceExtension->DriverExtension->DataQueueSize)
// 		{
// 			/*
// 			 * We're exceeding the buffer, and data will be thrown away...
// 			 * FIXME: What could we do, as we are at DISPATCH_LEVEL?
// 			 */
// 			ReadSize = ClassDeviceExtension->DriverExtension->DataQueueSize - ClassDeviceExtension->InputCount;
// 		}
// 		else
// 			ReadSize = InputCount;
//
// 		/*
// 		 * Move the input data from the port data queue to our class data
// 		 * queue.
// 		 */
// 		RtlCopyMemory(
// 			&ClassDeviceExtension->PortData[ClassDeviceExtension->InputCount],
// 			(PCHAR)DataStart,
// 			sizeof(KEYBOARD_INPUT_DATA) * ReadSize);
//
// 		/* Move the counter up */
// 		ClassDeviceExtension->InputCount += ReadSize;
//
// 		(*ConsumedCount) += (ULONG)ReadSize;
//
// 		/* Complete pending IRP (if any) */
// 		if (ClassDeviceExtension->PendingIrp)
// 			HandleReadIrp(ClassDeviceObject, ClassDeviceExtension->PendingIrp, FALSE);
// 	}
// 	KeReleaseSpinLock(&ClassDeviceExtension->SpinLock, OldIrql);
//
// 	TRACE_(CLASS_NAME, "Leaving ClassCallback()\n");
// 	return TRUE;
// }
//
// /* Send IOCTL_INTERNAL_*_CONNECT to port */
// static NTSTATUS
// ConnectPortDriver(
// 	IN PDEVICE_OBJECT PortDO,
// 	IN PDEVICE_OBJECT ClassDO)
// {
// 	KEVENT Event;
// 	PIRP Irp;
// 	IO_STATUS_BLOCK IoStatus;
// 	CONNECT_DATA ConnectData;
// 	NTSTATUS Status;
//
// 	TRACE_(CLASS_NAME, "Connecting PortDO %p to ClassDO %p\n", PortDO, ClassDO);
//
// 	KeInitializeEvent(&Event, NotificationEvent, FALSE);
//
// 	ConnectData.ClassDeviceObject = ClassDO;
// 	ConnectData.ClassService      = ClassCallback;
//
// 	Irp = IoBuildDeviceIoControlRequest(
// 		IOCTL_INTERNAL_KEYBOARD_CONNECT,
// 		PortDO,
// 		&ConnectData, sizeof(CONNECT_DATA),
// 		NULL, 0,
// 		TRUE, &Event, &IoStatus);
// 	if (!Irp)
// 		return STATUS_INSUFFICIENT_RESOURCES;
//
// 	Status = IoCallDriver(PortDO, Irp);
//
// 	if (Status == STATUS_PENDING)
// 		KeWaitForSingleObject(&Event, Suspended, KernelMode, FALSE, NULL);
// 	else
// 		IoStatus.Status = Status;
//
// 	if (NT_SUCCESS(IoStatus.Status))
// 	{
// 		ObReferenceObject(PortDO);
// 		ExInterlockedInsertTailList(
// 			&((PCLASS_DEVICE_EXTENSION)ClassDO->DeviceExtension)->ListHead,
// 			&((PPORT_DEVICE_EXTENSION)PortDO->DeviceExtension)->ListEntry,
// 			&((PCLASS_DEVICE_EXTENSION)ClassDO->DeviceExtension)->ListSpinLock);
// 		if (ClassDO->StackSize <= PortDO->StackSize)
// 		{
// 			/* Increase the stack size, in case we have to
// 			 * forward some IRPs to the port device object
// 			 */
// 			ClassDO->StackSize = PortDO->StackSize + 1;
// 		}
// 	}
//
// 	return IoStatus.Status;
// }
//
// /* Send IOCTL_INTERNAL_*_DISCONNECT to port + destroy the Port DO */
// static VOID
// DestroyPortDriver(
// 	IN PDEVICE_OBJECT PortDO)
// {
// 	PPORT_DEVICE_EXTENSION DeviceExtension;
// 	PCLASS_DEVICE_EXTENSION ClassDeviceExtension;
// 	PCLASS_DRIVER_EXTENSION DriverExtension;
// 	KEVENT Event;
// 	PIRP Irp;
// 	IO_STATUS_BLOCK IoStatus;
// 	KIRQL OldIrql;
// 	NTSTATUS Status;
//
// 	TRACE_(CLASS_NAME, "Destroying PortDO %p\n", PortDO);
//
// 	DeviceExtension = (PPORT_DEVICE_EXTENSION)PortDO->DeviceExtension;
// 	ClassDeviceExtension = DeviceExtension->ClassDO->DeviceExtension;
// 	DriverExtension = IoGetDriverObjectExtension(PortDO->DriverObject, PortDO->DriverObject);
//
// 	/* Send IOCTL_INTERNAL_*_DISCONNECT */
// 	KeInitializeEvent(&Event, NotificationEvent, FALSE);
// 	Irp = IoBuildDeviceIoControlRequest(
// 		IOCTL_INTERNAL_KEYBOARD_DISCONNECT,
// 		PortDO,
// 		NULL, 0,
// 		NULL, 0,
// 		TRUE, &Event, &IoStatus);
// 	if (Irp)
// 	{
// 		Status = IoCallDriver(PortDO, Irp);
// 		if (Status == STATUS_PENDING)
// 			KeWaitForSingleObject(&Event, Suspended, KernelMode, FALSE, NULL);
// 	}
//
// 	/* Remove from ClassDeviceExtension->ListHead list */
// 	KeAcquireSpinLock(&ClassDeviceExtension->ListSpinLock, &OldIrql);
// 	RemoveEntryList(&DeviceExtension->ListEntry);
// 	KeReleaseSpinLock(&ClassDeviceExtension->ListSpinLock, OldIrql);
//
// 	/* Remove entry from HKEY_LOCAL_MACHINE\HARDWARE\DEVICEMAP\[DeviceBaseName] */
// 	RtlDeleteRegistryValue(
// 		RTL_REGISTRY_DEVICEMAP,
// 		DriverExtension->DeviceBaseName.Buffer,
// 		ClassDeviceExtension->DeviceName);
//
// 	if (DeviceExtension->LowerDevice)
// 		IoDetachDevice(DeviceExtension->LowerDevice);
// 	ObDereferenceObject(PortDO);
//
// 	if (!DriverExtension->ConnectMultiplePorts && DeviceExtension->ClassDO)
// 	{
// 		ExFreePoolWithTag(ClassDeviceExtension->PortData, CLASS_TAG);
// 		ExFreePoolWithTag((PVOID)ClassDeviceExtension->DeviceName, CLASS_TAG);
// 		IoDeleteDevice(DeviceExtension->ClassDO);
// 	}
//
// 	IoDeleteDevice(PortDO);
// }
//
// static NTSTATUS NTAPI
// ClassAddDevice(
// 	IN PDRIVER_OBJECT DriverObject,
// 	IN PDEVICE_OBJECT Pdo)
// {
// 	PCLASS_DRIVER_EXTENSION DriverExtension;
// 	PDEVICE_OBJECT Fdo = NULL;
// 	PPORT_DEVICE_EXTENSION DeviceExtension = NULL;
// 	NTSTATUS Status;
//
// 	TRACE_(CLASS_NAME, "ClassAddDevice called. Pdo = 0x%p\n", Pdo);
//
// 	DriverExtension = IoGetDriverObjectExtension(DriverObject, DriverObject);
//
// 	if (Pdo == NULL)
// 		/* We may get a NULL Pdo at the first call as we're a legacy driver. Ignore it */
// 		return STATUS_SUCCESS;
//
// 	/* Create new device object */
// 	Status = IoCreateDevice(
// 		DriverObject,
// 		sizeof(PORT_DEVICE_EXTENSION),
// 		NULL,
// 		Pdo->DeviceType,
// 		Pdo->Characteristics & FILE_DEVICE_SECURE_OPEN ? FILE_DEVICE_SECURE_OPEN : 0,
// 		FALSE,
// 		&Fdo);
// 	if (!NT_SUCCESS(Status))
// 	{
// 		WARN_(CLASS_NAME, "IoCreateDevice() failed with status 0x%08lx\n", Status);
// 		goto cleanup;
// 	}
// 	IoSetStartIoAttributes(Fdo, TRUE, TRUE);
//
// 	DeviceExtension = (PPORT_DEVICE_EXTENSION)Fdo->DeviceExtension;
// 	RtlZeroMemory(DeviceExtension, sizeof(PORT_DEVICE_EXTENSION));
// 	DeviceExtension->Common.IsClassDO = FALSE;
// 	DeviceExtension->DeviceObject = Fdo;
// 	DeviceExtension->PnpState = dsStopped;
// 	Status = IoAttachDeviceToDeviceStackSafe(Fdo, Pdo, &DeviceExtension->LowerDevice);
// 	if (!NT_SUCCESS(Status))
// 	{
// 		WARN_(CLASS_NAME, "IoAttachDeviceToDeviceStackSafe() failed with status 0x%08lx\n", Status);
// 		goto cleanup;
// 	}
// 	if (DeviceExtension->LowerDevice->Flags & DO_POWER_PAGABLE)
// 		Fdo->Flags |= DO_POWER_PAGABLE;
// 	if (DeviceExtension->LowerDevice->Flags & DO_BUFFERED_IO)
// 		Fdo->Flags |= DO_BUFFERED_IO;
// 	if (DeviceExtension->LowerDevice->Flags & DO_DIRECT_IO)
// 		Fdo->Flags |= DO_DIRECT_IO;
//
// 	if (DriverExtension->ConnectMultiplePorts)
// 		DeviceExtension->ClassDO = DriverExtension->MainClassDeviceObject;
// 	else
// 	{
// 		/* We need a new class device object for this Fdo */
// 		Status = CreateClassDeviceObject(
// 			DriverObject,
// 			&DeviceExtension->ClassDO);
// 		if (!NT_SUCCESS(Status))
// 		{
// 			WARN_(CLASS_NAME, "CreateClassDeviceObject() failed with status 0x%08lx\n", Status);
// 			goto cleanup;
// 		}
// 	}
// 	Status = ConnectPortDriver(Fdo, DeviceExtension->ClassDO);
// 	if (!NT_SUCCESS(Status))
// 	{
// 		WARN_(CLASS_NAME, "ConnectPortDriver() failed with status 0x%08lx\n", Status);
// 		goto cleanup;
// 	}
// 	Fdo->Flags &= ~DO_DEVICE_INITIALIZING;
//
// 	/* Register interface ; ignore the error (if any) as having
// 	 * a registered interface is not so important... */
// 	Status = IoRegisterDeviceInterface(
// 		Pdo,
// 		core::ptr::addr_of!(GUID_DEVINTERFACE_KEYBOARD) as *const _GUID,
// 		NULL,
// 		&DeviceExtension->InterfaceName);
// 	if (!NT_SUCCESS(Status))
// 		DeviceExtension->InterfaceName.Length = 0;
//
// 	return STATUS_SUCCESS;
//
// cleanup:
// 	if (Fdo)
// 		DestroyPortDriver(Fdo);
// 	return Status;
// }
//
// static VOID NTAPI
// ClassCancelRoutine(
// 	IN PDEVICE_OBJECT DeviceObject,
// 	IN PIRP Irp)
// {
// 	PCLASS_DEVICE_EXTENSION ClassDeviceExtension = DeviceObject->DeviceExtension;
// 	KIRQL OldIrql;
// 	BOOLEAN wasQueued = FALSE;
//
// 	TRACE_(CLASS_NAME, "ClassCancelRoutine(DeviceObject %p, Irp %p)\n", DeviceObject, Irp);
//
// 	ASSERT(ClassDeviceExtension->Common.IsClassDO);
//
// 	IoReleaseCancelSpinLock(Irp->CancelIrql);
//
// 	KeAcquireSpinLock(&ClassDeviceExtension->SpinLock, &OldIrql);
//
// 	if (ClassDeviceExtension->PendingIrp == Irp)
// 	{
// 		ClassDeviceExtension->PendingIrp = NULL;
// 		wasQueued = TRUE;
// 	}
// 	KeReleaseSpinLock(&ClassDeviceExtension->SpinLock, OldIrql);
//
// 	if (wasQueued)
// 	{
// 		Irp->IoStatus.Status = STATUS_CANCELLED;
// 		Irp->IoStatus.Information = 0;
// 		IoCompleteRequest(Irp, IO_NO_INCREMENT);
// 	}
// 	else
// 	{
// 		DPRINT1("Cancelled IRP is not pending. Race condition?\n");
// 	}
// }
//
// static NTSTATUS
// HandleReadIrp(
// 	IN PDEVICE_OBJECT DeviceObject,
// 	IN PIRP Irp,
// 	BOOLEAN IsInStartIo)
// {
// 	PCLASS_DEVICE_EXTENSION DeviceExtension = DeviceObject->DeviceExtension;
// 	NTSTATUS Status;
// 	KIRQL OldIrql;
//
// 	TRACE_(CLASS_NAME, "HandleReadIrp(DeviceObject %p, Irp %p)\n", DeviceObject, Irp);
//
// 	ASSERT(DeviceExtension->Common.IsClassDO);
//
// 	if (DeviceExtension->InputCount > 0)
// 	{
// 		SIZE_T NumberOfEntries;
//
// 		NumberOfEntries = MIN(
// 			DeviceExtension->InputCount,
// 			IoGetCurrentIrpStackLocation(Irp)->Parameters.Read.Length / sizeof(KEYBOARD_INPUT_DATA));
//
// 		Status = FillEntries(
// 			DeviceObject,
// 			Irp,
// 			DeviceExtension->PortData,
// 			NumberOfEntries);
//
// 		if (NT_SUCCESS(Status))
// 		{
// 			if (DeviceExtension->InputCount > NumberOfEntries)
// 			{
// 				RtlMoveMemory(
// 					&DeviceExtension->PortData[0],
// 					&DeviceExtension->PortData[NumberOfEntries],
// 					(DeviceExtension->InputCount - NumberOfEntries) * sizeof(KEYBOARD_INPUT_DATA));
// 			}
//
// 			DeviceExtension->InputCount -= NumberOfEntries;
//
// 			Irp->IoStatus.Information = NumberOfEntries * sizeof(KEYBOARD_INPUT_DATA);
// 		}
//
// 		/* Go to next packet and complete this request */
// 		Irp->IoStatus.Status = Status;
//
// 		(VOID)IoSetCancelRoutine(Irp, NULL);
// 		IoCompleteRequest(Irp, IO_KEYBOARD_INCREMENT);
// 		DeviceExtension->PendingIrp = NULL;
// 	}
// 	else
// 	{
// 		IoAcquireCancelSpinLock(&OldIrql);
// 		if (Irp->Cancel)
// 		{
// 			DeviceExtension->PendingIrp = NULL;
// 			Status = STATUS_CANCELLED;
// 		}
// 		else
// 		{
// 			IoMarkIrpPending(Irp);
// 			DeviceExtension->PendingIrp = Irp;
// 			(VOID)IoSetCancelRoutine(Irp, ClassCancelRoutine);
// 			Status = STATUS_PENDING;
// 		}
// 		IoReleaseCancelSpinLock(OldIrql);
// 	}
// 	return Status;
// }
//
// static NTSTATUS NTAPI
// ClassPnp(
// 	IN PDEVICE_OBJECT DeviceObject,
// 	IN PIRP Irp)
// {
// 	PPORT_DEVICE_EXTENSION DeviceExtension = DeviceObject->DeviceExtension;
// 	PIO_STACK_LOCATION IrpSp = IoGetCurrentIrpStackLocation(Irp);
// 	OBJECT_ATTRIBUTES ObjectAttributes;
// 	IO_STATUS_BLOCK Iosb;
// 	NTSTATUS Status;
//
// 	switch (IrpSp->MinorFunction)
// 	{
// 		case IRP_MN_START_DEVICE:
// 			if (IoForwardIrpSynchronously(DeviceExtension->LowerDevice, Irp))
// 			{
// 				Status = Irp->IoStatus.Status;
// 			}
// 			else
// 			{
// 				Status = STATUS_UNSUCCESSFUL;
// 			}
//
// 			if (NT_SUCCESS(Status))
// 			{
// 				InitializeObjectAttributes(&ObjectAttributes,
// 										   &DeviceExtension->InterfaceName,
// 										   OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE,
// 										   NULL,
// 										   NULL);
//
// 				Status = ZwOpenFile(&DeviceExtension->FileHandle,
// 									FILE_READ_DATA,
// 									&ObjectAttributes,
// 									&Iosb,
// 									0,
// 									0);
// 				if (!NT_SUCCESS(Status))
// 					DeviceExtension->FileHandle = NULL;
// 			}
// 			else
// 				DeviceExtension->FileHandle = NULL;
// 			if (DeviceExtension->InterfaceName.Length != 0)
// 				IoSetDeviceInterfaceState(&DeviceExtension->InterfaceName, TRUE);
// 			Irp->IoStatus.Status = Status;
// 			IoCompleteRequest(Irp, IO_NO_INCREMENT);
// 			return Status;
//
// 		case IRP_MN_STOP_DEVICE:
// 			if (DeviceExtension->FileHandle)
// 			{
// 				ZwClose(DeviceExtension->FileHandle);
// 				DeviceExtension->FileHandle = NULL;
// 			}
// 			Status = STATUS_SUCCESS;
// 			break;
//
//         case IRP_MN_REMOVE_DEVICE:
// 			if (DeviceExtension->InterfaceName.Length != 0)
// 				IoSetDeviceInterfaceState(&DeviceExtension->InterfaceName, FALSE);
//             if (DeviceExtension->FileHandle)
// 			{
// 				ZwClose(DeviceExtension->FileHandle);
// 				DeviceExtension->FileHandle = NULL;
// 			}
//             IoSkipCurrentIrpStackLocation(Irp);
// 		    Status = IoCallDriver(DeviceExtension->LowerDevice, Irp);
//             DestroyPortDriver(DeviceObject);
// 			return Status;
//
// 		default:
// 			Status = Irp->IoStatus.Status;
// 			break;
// 	}
//
// 	Irp->IoStatus.Status = Status;
// 	if (NT_SUCCESS(Status) || Status == STATUS_NOT_SUPPORTED)
// 	{
// 		IoSkipCurrentIrpStackLocation(Irp);
// 		return IoCallDriver(DeviceExtension->LowerDevice, Irp);
// 	}
// 	else
// 	{
// 		IoCompleteRequest(Irp, IO_NO_INCREMENT);
// 		return Status;
// 	}
// }
//
// static VOID NTAPI
// ClassStartIo(
// 	IN PDEVICE_OBJECT DeviceObject,
// 	IN PIRP Irp)
// {
// 	PCLASS_DEVICE_EXTENSION DeviceExtension = DeviceObject->DeviceExtension;
// 	KIRQL OldIrql;
//
// 	TRACE_(CLASS_NAME, "ClassStartIo(DeviceObject %p, Irp %p)\n", DeviceObject, Irp);
//
// 	ASSERT(DeviceExtension->Common.IsClassDO);
//
// 	KeAcquireSpinLock(&DeviceExtension->SpinLock, &OldIrql);
// 	HandleReadIrp(DeviceObject, Irp, TRUE);
// 	KeReleaseSpinLock(&DeviceExtension->SpinLock, OldIrql);
// }
//
// static VOID NTAPI
// SearchForLegacyDrivers(
// 	IN PDRIVER_OBJECT DriverObject,
// 	IN PVOID Context, /* PCLASS_DRIVER_EXTENSION */
// 	IN ULONG Count)
// {
// 	UNICODE_STRING DeviceMapKeyU = RTL_CONSTANT_STRING(L"\\REGISTRY\\MACHINE\\HARDWARE\\DEVICEMAP");
// 	PCLASS_DRIVER_EXTENSION DriverExtension;
// 	UNICODE_STRING PortBaseName = { 0, 0, NULL };
// 	PKEY_VALUE_BASIC_INFORMATION KeyValueInformation = NULL;
// 	OBJECT_ATTRIBUTES ObjectAttributes;
// 	HANDLE hDeviceMapKey = (HANDLE)-1;
// 	HANDLE hPortKey = (HANDLE)-1;
// 	ULONG Index = 0;
// 	ULONG Size, ResultLength;
// 	NTSTATUS Status;
//
// 	TRACE_(CLASS_NAME, "SearchForLegacyDrivers(%p %p %lu)\n",
// 		DriverObject, Context, Count);
//
// 	if (Count != 1)
// 		return;
// 	DriverExtension = (PCLASS_DRIVER_EXTENSION)Context;
//
// 	/* Create port base name, by replacing Class by Port at the end of the class base name */
// 	Status = DuplicateUnicodeString(
// 		RTL_DUPLICATE_UNICODE_STRING_NULL_TERMINATE,
// 		&DriverExtension->DeviceBaseName,
// 		&PortBaseName);
// 	if (!NT_SUCCESS(Status))
// 	{
// 		WARN_(CLASS_NAME, "DuplicateUnicodeString() failed with status 0x%08lx\n", Status);
// 		goto cleanup;
// 	}
// 	PortBaseName.Length -= (sizeof(L"Class") - sizeof(UNICODE_NULL));
// 	RtlAppendUnicodeToString(&PortBaseName, L"Port");
//
// 	/* Allocate memory */
// 	Size = sizeof(KEY_VALUE_BASIC_INFORMATION) + MAX_PATH;
// 	KeyValueInformation = ExAllocatePoolWithTag(PagedPool, Size, CLASS_TAG);
// 	if (!KeyValueInformation)
// 	{
// 		WARN_(CLASS_NAME, "ExAllocatePoolWithTag() failed\n");
// 		Status = STATUS_NO_MEMORY;
// 		goto cleanup;
// 	}
//
// 	/* Open HKEY_LOCAL_MACHINE\HARDWARE\DEVICEMAP */
// 	InitializeObjectAttributes(&ObjectAttributes, &DeviceMapKeyU, OBJ_KERNEL_HANDLE | OBJ_CASE_INSENSITIVE, NULL, NULL);
// 	Status = ZwOpenKey(&hDeviceMapKey, 0, &ObjectAttributes);
// 	if (Status == STATUS_OBJECT_NAME_NOT_FOUND)
// 	{
// 		INFO_(CLASS_NAME, "HKLM\\HARDWARE\\DEVICEMAP is non-existent\n");
// 		Status = STATUS_SUCCESS;
// 		goto cleanup;
// 	}
// 	else if (!NT_SUCCESS(Status))
// 	{
// 		WARN_(CLASS_NAME, "ZwOpenKey() failed with status 0x%08lx\n", Status);
// 		goto cleanup;
// 	}
//
// 	/* Open sub key */
// 	InitializeObjectAttributes(&ObjectAttributes, &PortBaseName, OBJ_KERNEL_HANDLE | OBJ_CASE_INSENSITIVE, hDeviceMapKey, NULL);
// 	Status = ZwOpenKey(&hPortKey, KEY_QUERY_VALUE, &ObjectAttributes);
// 	if (Status == STATUS_OBJECT_NAME_NOT_FOUND)
// 	{
// 		INFO_(CLASS_NAME, "HKLM\\HARDWARE\\DEVICEMAP\\%wZ is non-existent\n", &PortBaseName);
// 		Status = STATUS_SUCCESS;
// 		goto cleanup;
// 	}
// 	else if (!NT_SUCCESS(Status))
// 	{
// 		WARN_(CLASS_NAME, "ZwOpenKey() failed with status 0x%08lx\n", Status);
// 		goto cleanup;
// 	}
//
// 	/* Read each value name */
// 	while (ZwEnumerateValueKey(hPortKey, Index++, KeyValueBasicInformation, KeyValueInformation, Size, &ResultLength) == STATUS_SUCCESS)
// 	{
// 		UNICODE_STRING PortName;
// 		PDEVICE_OBJECT PortDeviceObject = NULL;
// 		PFILE_OBJECT FileObject = NULL;
//
// 		PortName.Length = PortName.MaximumLength = (USHORT)KeyValueInformation->NameLength;
// 		PortName.Buffer = KeyValueInformation->Name;
//
// 		/* Open the device object pointer */
// 		Status = IoGetDeviceObjectPointer(&PortName, FILE_READ_ATTRIBUTES, &FileObject, &PortDeviceObject);
// 		if (!NT_SUCCESS(Status))
// 		{
// 			WARN_(CLASS_NAME, "IoGetDeviceObjectPointer(%wZ) failed with status 0x%08lx\n", &PortName, Status);
// 			continue;
// 		}
// 		INFO_(CLASS_NAME, "Legacy driver found\n");
//
// 		Status = ClassAddDevice(DriverObject, PortDeviceObject);
// 		if (!NT_SUCCESS(Status))
// 		{
// 			/* FIXME: Log the error */
// 			WARN_(CLASS_NAME, "ClassAddDevice() failed with status 0x%08lx\n", Status);
// 		}
//
// 		ObDereferenceObject(FileObject);
// 	}
//
// cleanup:
// 	if (KeyValueInformation != NULL)
// 		ExFreePoolWithTag(KeyValueInformation, CLASS_TAG);
// 	if (hDeviceMapKey != (HANDLE)-1)
// 		ZwClose(hDeviceMapKey);
// 	if (hPortKey != (HANDLE)-1)
// 		ZwClose(hPortKey);
// }
//
// /*
//  * Standard DriverEntry method.
//  */
// NTSTATUS NTAPI
// DriverEntry(
// 	IN PDRIVER_OBJECT DriverObject,
// 	IN PUNICODE_STRING RegistryPath)
// {
// 	PCLASS_DRIVER_EXTENSION DriverExtension;
// 	NTSTATUS Status;
//
// 	Status = IoAllocateDriverObjectExtension(
// 		DriverObject,
// 		DriverObject,
// 		sizeof(CLASS_DRIVER_EXTENSION),
// 		(PVOID*)&DriverExtension);
// 	if (!NT_SUCCESS(Status))
// 	{
// 		WARN_(CLASS_NAME, "IoAllocateDriverObjectExtension() failed with status 0x%08lx\n", Status);
// 		return Status;
// 	}
// 	RtlZeroMemory(DriverExtension, sizeof(CLASS_DRIVER_EXTENSION));
//
// 	Status = DuplicateUnicodeString(
// 		RTL_DUPLICATE_UNICODE_STRING_NULL_TERMINATE,
// 		RegistryPath,
// 		&DriverExtension->RegistryPath);
// 	if (!NT_SUCCESS(Status))
// 	{
// 		WARN_(CLASS_NAME, "DuplicateUnicodeString() failed with status 0x%08lx\n", Status);
// 		return Status;
// 	}
//
// 	Status = ReadRegistryEntries(RegistryPath, DriverExtension);
// 	if (!NT_SUCCESS(Status))
// 	{
// 		WARN_(CLASS_NAME, "ReadRegistryEntries() failed with status 0x%08lx\n", Status);
// 		return Status;
// 	}
//
// 	if (DriverExtension->ConnectMultiplePorts == 1)
// 	{
// 		Status = CreateClassDeviceObject(
// 			DriverObject,
// 			&DriverExtension->MainClassDeviceObject);
// 		if (!NT_SUCCESS(Status))
// 		{
// 			WARN_(CLASS_NAME, "CreateClassDeviceObject() failed with status 0x%08lx\n", Status);
// 			return Status;
// 		}
// 	}
//
// 	DriverObject->DriverExtension->AddDevice = ClassAddDevice;
// 	DriverObject->DriverUnload = DriverUnload;
//
// 	DriverObject->MajorFunction[IRP_MJ_CREATE]         = ClassCreate;
// 	DriverObject->MajorFunction[IRP_MJ_CLOSE]          = ClassClose;
// 	DriverObject->MajorFunction[IRP_MJ_CLEANUP]        = ClassCleanup;
// 	DriverObject->MajorFunction[IRP_MJ_READ]           = ClassRead;
// 	DriverObject->MajorFunction[IRP_MJ_POWER]          = ClassPower;
// 	DriverObject->MajorFunction[IRP_MJ_PNP]            = ClassPnp;
// 	DriverObject->MajorFunction[IRP_MJ_DEVICE_CONTROL] = ClassDeviceControl;
// 	DriverObject->MajorFunction[IRP_MJ_INTERNAL_DEVICE_CONTROL] = ForwardIrpAndForget;
// 	DriverObject->DriverStartIo                        = ClassStartIo;
//
// 	/* We will detect the legacy devices later */
// 	IoRegisterDriverReinitialization(
// 		DriverObject,
// 		SearchForLegacyDrivers,
// 		DriverExtension);
//
// 	return STATUS_SUCCESS;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
