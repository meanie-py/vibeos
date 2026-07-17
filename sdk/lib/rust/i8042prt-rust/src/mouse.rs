use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::driver_ddk::*;
use ntkrnl_sys::ex::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ddk::{_WORK_QUEUE_TYPE_DelayedWorkQueue as DelayedWorkQueue, IoAllocateWorkItem, IoFreeWorkItem, IoQueueWorkItem, KeAcquireInterruptSpinLock, KeCancelTimer, KeInitializeDpc, KeInitializeTimer, KeInsertQueueDpc, KeReleaseInterruptSpinLock, KeSetTimer, KeStallExecutionProcessor, LARGE_INTEGER, MOUSE_ATTRIBUTES, MOUSE_INPUT_DATA, PDEVICE_OBJECT, PIO_STACK_LOCATION, PIO_WORKITEM, PIRP, PKINTERRUPT, PMOUSE_ATTRIBUTES, PMOUSE_INPUT_DATA, ULARGE_INTEGER};
// FORCEINLINE accessors (no exported symbol — use the Rust inline impls).
use ntkrnl_sys::inline::tier1::{IoGetCurrentIrpStackLocation, IoMarkIrpPending};
use core::ffi::c_void;
use crate::i8042prt_h::*;

// pub(crate) siblings (importable by path)
use crate::readwrite::{i8042IsrWritePort, i8042Write, i8042ReadDataWait, i8042ReadStatus, i8042ReadMouseData};
use crate::i8042prt::i8042PacketIsr;

// C does `DeviceExtension->MouseResetState++` on a repr(C) enum. Rust enums
// have no `+=`, so the state-machine increment is expressed as the next
// discriminant. Sound: every reachable state in i8042MouResetIsr advances to a
// defined successor (the C code never increments past the last real state).
#[inline]
unsafe fn next_reset(s: MOUSE_RESET_SUBSTATE) -> MOUSE_RESET_SUBSTATE {
    core::mem::transmute(s as u32 + 1)
}

// #[no_mangle] siblings: link by symbol via extern block.
extern "system" {
    fn i8042SendHookWorkItem(DeviceObject: PDEVICE_OBJECT, Context: PVOID);
    fn ForwardIrpAndForget(DeviceObject: PDEVICE_OBJECT, Irp: PIRP) -> NTSTATUS;
}
// i8042MouHandlePs2pp is cdecl (plain C fn in i8042prt.h).
extern "C" {
    fn i8042MouHandlePs2pp(DeviceExtension: PI8042_MOUSE_EXTENSION, Input: UCHAR);
}

// ---- file-scope preamble (translated) ----
// NOTE: the C had forward declarations `static KDEFERRED_ROUTINE i8042MouDpcRoutine;`
// and `i8042DpcRoutineMouseTimeout;` — these are forward decls of the DPC routine
// FUNCTIONS (defined below), not function-pointer globals. rustconv mistranslated
// them as `static = None` which both (a) collides with the real fn names and
// (b) nulls the routines. Removed; the real functions follow.

#[allow(unpredictable_function_pointer_comparisons)] // FNPTR-CMP(audited): intentional identity compare mirroring the C source; revisit -> core::ptr::fn_addr_eq (see docs/engineering/active-lanes/warnings-burndown-plan.md)
unsafe extern "system" fn i8042MouIsrWritePort(
	Context: PVOID,
	Value: UCHAR)
{
	let DeviceExtension: PI8042_MOUSE_EXTENSION = Context as PI8042_MOUSE_EXTENSION;

	if (*DeviceExtension).MouseHook.IsrWritePort != Some(i8042MouIsrWritePort) {
		((*DeviceExtension).MouseHook.IsrWritePort.unwrap())(
			(*DeviceExtension).MouseHook.CallContext,
			Value);
	} else {
		i8042IsrWritePort((*DeviceExtension).Common.PortDeviceExtension, Value, CTRL_WRITE_MOUSE as UCHAR);
	}
}

unsafe extern "system" fn i8042MouQueuePacket(Context: PVOID) {
    let DeviceExtension = Context as PI8042_MOUSE_EXTENSION;

    (*DeviceExtension).MouseComplete = 1u8;
    (*DeviceExtension).MouseInBuffer += 1;
    if (*DeviceExtension).MouseInBuffer >= (*(*DeviceExtension).Common.PortDeviceExtension).Settings.MouseDataQueueSize {
        (*DeviceExtension).MouseInBuffer -= 1;
    }

    KeInsertQueueDpc(
        core::ptr::addr_of_mut!((*DeviceExtension).DpcMouse),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );
}

// C: plain (non-NTAPI) `VOID i8042MouHandle(...)` — cdecl. Exported so ps2pp's
// `extern "C" { fn i8042MouHandle; }` resolves (i386 `_i8042MouHandle`).
#[no_mangle]
pub unsafe extern "C" fn i8042MouHandle(
    DeviceExtension: PI8042_MOUSE_EXTENSION,
    Output: UCHAR,
) {
    let MouseInput: PMOUSE_INPUT_DATA =
        (*DeviceExtension).MouseBuffer.add((*DeviceExtension).MouseInBuffer as usize);
    let mut Scroll: CHAR;

    match (*DeviceExtension).MouseState {
        _MOUSE_STATE_MouseIdle => {
            if (Output & 8) == 0 {
                return;
            }

            (*MouseInput).__bindgen_anon_1.Buttons = 0;
            (*MouseInput).RawButtons = 0;
            (*MouseInput).Flags = MOUSE_MOVE_RELATIVE as USHORT;

            if (Output & 16) != 0 {
                (*MouseInput).LastX = 1;
            } else {
                (*MouseInput).LastX = 0;
            }
            if (Output & 32) != 0 {
                (*MouseInput).LastY = 1;
            } else {
                (*MouseInput).LastY = 0;
            }

            if (Output & 1) != 0 {
                (*MouseInput).RawButtons |= MOUSE_LEFT_BUTTON_DOWN as u32;
            }
            if (Output & 2) != 0 {
                (*MouseInput).RawButtons |= MOUSE_RIGHT_BUTTON_DOWN as u32;
            }
            if (Output & 4) != 0 {
                (*MouseInput).RawButtons |= MOUSE_MIDDLE_BUTTON_DOWN as u32;
            }

            (*DeviceExtension).MouseState = _MOUSE_STATE_XMovement;
        }

        _MOUSE_STATE_XMovement => {
            if (*MouseInput).LastX != 0 {
                (*MouseInput).LastX = Output as LONG - 256;
            } else {
                (*MouseInput).LastX = Output as LONG;
            }

            (*DeviceExtension).MouseState = _MOUSE_STATE_YMovement;
        }

        _MOUSE_STATE_YMovement => {
            if (*MouseInput).LastY != 0 {
                (*MouseInput).LastY = Output as LONG - 256;
            } else {
                (*MouseInput).LastY = Output as LONG;
            }

            (*MouseInput).LastY = -(*MouseInput).LastY;

            if (*DeviceExtension).MouseType == _MOUSE_TYPE_GenericPS2
                || (*DeviceExtension).MouseType == _MOUSE_TYPE_Ps2pp
            {
                i8042MouHandleButtons(
                    DeviceExtension,
                    (MOUSE_LEFT_BUTTON_DOWN as u32
                        | MOUSE_RIGHT_BUTTON_DOWN as u32
                        | MOUSE_MIDDLE_BUTTON_DOWN as u32) as USHORT,
                );
                (*DeviceExtension).MouseHook.QueueMousePacket.unwrap()(
                    (*DeviceExtension).MouseHook.CallContext,
                );
                (*DeviceExtension).MouseState = _MOUSE_STATE_MouseIdle;
            } else {
                (*DeviceExtension).MouseState = _MOUSE_STATE_ZMovement;
            }
        }

        _MOUSE_STATE_ZMovement => {
            Scroll = (Output & 0x0f) as CHAR;
            if (Scroll & 8) != 0 {
                Scroll = (Scroll as u8 | 0xf0) as CHAR;
            }

            if Scroll != 0 {
                (*MouseInput).RawButtons |= MOUSE_WHEEL as u32;
                (*MouseInput).__bindgen_anon_1.__bindgen_anon_1.ButtonData = (Scroll as i32 * -(WHEEL_DELTA as i32)) as USHORT;
            }

            if (*DeviceExtension).MouseType == _MOUSE_TYPE_IntellimouseExplorer {
                if (Output & 16) != 0 {
                    (*MouseInput).RawButtons |= MOUSE_BUTTON_4_DOWN as u32;
                }
                if (Output & 32) != 0 {
                    (*MouseInput).RawButtons |= MOUSE_BUTTON_5_DOWN as u32;
                }
            }
            i8042MouHandleButtons(
                DeviceExtension,
                (MOUSE_LEFT_BUTTON_DOWN as u32
                    | MOUSE_RIGHT_BUTTON_DOWN as u32
                    | MOUSE_MIDDLE_BUTTON_DOWN as u32
                    | MOUSE_BUTTON_4_DOWN
                    | MOUSE_BUTTON_5_DOWN) as USHORT,
            );
            (*DeviceExtension).MouseHook.QueueMousePacket.unwrap()(
                (*DeviceExtension).MouseHook.CallContext,
            );
            (*DeviceExtension).MouseState = _MOUSE_STATE_MouseIdle;
        }

        _ => {}
    }
}

// C: plain (non-NTAPI) `VOID i8042MouHandleButtons(...)` — cdecl. Exported so
// ps2pp's `extern "C" { fn i8042MouHandleButtons; }` resolves.
#[no_mangle]
pub unsafe extern "C" fn i8042MouHandleButtons(
    DeviceExtension: PI8042_MOUSE_EXTENSION,
    Mask: USHORT,
) {
    let MouseInput: PMOUSE_INPUT_DATA =
        (*DeviceExtension).MouseBuffer.add((*DeviceExtension).MouseInBuffer as usize);
    let NewButtonData: USHORT = (((*MouseInput).RawButtons as u16) & Mask) as USHORT;
    let ButtonDiff: USHORT = (NewButtonData ^ (*DeviceExtension).MouseButtonState) & Mask;

    (*MouseInput).__bindgen_anon_1.__bindgen_anon_1.ButtonFlags |= (NewButtonData & ButtonDiff)
        | ((((!NewButtonData) as u32) << 1) as USHORT & ((ButtonDiff as u32) << 1) as USHORT)
        | (((*MouseInput).RawButtons & 0xfc00) as u16);

    (*DeviceExtension).MouseButtonState =
        ((*DeviceExtension).MouseButtonState & !Mask) | (NewButtonData & Mask);
}

pub(crate) unsafe fn i8042MouInitialize(DeviceExtension: PI8042_MOUSE_EXTENSION) -> NTSTATUS {
    let Status: NTSTATUS;
    let mut Value: UCHAR = 0;

    i8042Write(
        (*DeviceExtension).Common.PortDeviceExtension,
        (*(*DeviceExtension).Common.PortDeviceExtension).ControlPort,
        MOUSE_ENAB as UCHAR,
    );

    if i8042IsrWritePort(
        (*DeviceExtension).Common.PortDeviceExtension,
        MOU_ENAB as UCHAR,
        CTRL_WRITE_MOUSE as UCHAR,
    ) == 0
    {
        return STATUS_IO_DEVICE_ERROR;
    }

    Status = i8042ReadDataWait((*DeviceExtension).Common.PortDeviceExtension, &mut Value);
    if !NT_SUCCESS(Status) {
        return Status;
    }

    if Value == MOUSE_ACK as UCHAR {
        return STATUS_SUCCESS;
    }

    STATUS_IO_DEVICE_ERROR
}

unsafe extern "system" fn i8042MouDpcRoutine(
    Dpc: PKDPC,
    DeferredContext: PVOID,
    SystemArgument1: PVOID,
    SystemArgument2: PVOID,
) {
    let _ = Dpc;
    let _ = SystemArgument1;
    let _ = SystemArgument2;

    let DeviceExtension = DeferredContext as PI8042_MOUSE_EXTENSION;
    let PortDeviceExtension = (*DeviceExtension).Common.PortDeviceExtension;
    let mut MouseTransferred: ULONG = 0;
    let MouseInBufferCopy: ULONG;
    let mut Irql: KIRQL;
    let mut Timeout: LARGE_INTEGER = core::mem::zeroed();

    'switch: {
        match (*DeviceExtension).MouseTimeoutState {
            TimeoutStart => {
                (*DeviceExtension).MouseTimeoutState = NoChange;
                if (*DeviceExtension).MouseTimeoutActive != 0
                    && KeCancelTimer(core::ptr::addr_of_mut!((*DeviceExtension).TimerMouseTimeout)) == 0
                {
                    (*DeviceExtension).MouseTimeoutActive = 0u8;
                    return;
                }

                Timeout.QuadPart = -15000000;

                KeSetTimer(
                    core::ptr::addr_of_mut!((*DeviceExtension).TimerMouseTimeout),
                    Timeout,
                    core::ptr::addr_of_mut!((*DeviceExtension).DpcMouseTimeout),
                );
                (*DeviceExtension).MouseTimeoutActive = 1u8;
                return;
            }
            TimeoutCancel => {
                (*DeviceExtension).MouseTimeoutState = NoChange;
                KeCancelTimer(core::ptr::addr_of_mut!((*DeviceExtension).TimerMouseTimeout));
                (*DeviceExtension).MouseTimeoutActive = 0u8;
                break 'switch;
            }
            _ => {
                break 'switch;
            }
        }
    }

    if (*DeviceExtension).MouseComplete == 0 {
        return;
    }

    Irql = KeAcquireInterruptSpinLock((*PortDeviceExtension).HighestDIRQLInterrupt);

    (*DeviceExtension).MouseComplete = 0u8;
    MouseInBufferCopy = (*DeviceExtension).MouseInBuffer;

    KeReleaseInterruptSpinLock((*PortDeviceExtension).HighestDIRQLInterrupt, Irql);

    if (*DeviceExtension).MouseData.ClassService.is_null() {
        return;
    }

    let ClassService: unsafe extern "system" fn(*mut c_void, *mut c_void, *mut c_void, *mut u32) = core::mem::transmute((*DeviceExtension).MouseData.ClassService);
    (ClassService)(
        (*DeviceExtension).MouseData.ClassDeviceObject as *mut c_void,
        (*DeviceExtension).MouseBuffer as *mut c_void,
        (*DeviceExtension).MouseBuffer.add(MouseInBufferCopy as usize) as *mut c_void,
        &mut MouseTransferred,
    );

    Irql = KeAcquireInterruptSpinLock((*PortDeviceExtension).HighestDIRQLInterrupt);
    (*DeviceExtension).MouseInBuffer -= MouseTransferred;
    if (*DeviceExtension).MouseInBuffer != 0 {
        core::ptr::copy(
            (*DeviceExtension).MouseBuffer.add(MouseTransferred as usize),
            (*DeviceExtension).MouseBuffer,
            (*DeviceExtension).MouseInBuffer as usize,
        );
    }
    KeReleaseInterruptSpinLock((*PortDeviceExtension).HighestDIRQLInterrupt, Irql);
}

unsafe extern "system" fn i8042DpcRoutineMouseTimeout(
    Dpc: PKDPC,
    DeferredContext: PVOID,
    SystemArgument1: PVOID,
    SystemArgument2: PVOID,
) {
    let _ = Dpc;
    let _ = SystemArgument1;
    let _ = SystemArgument2;

    let DeviceExtension: PI8042_MOUSE_EXTENSION = DeferredContext as PI8042_MOUSE_EXTENSION;
    let PortDeviceExtension: PPORT_DEVICE_EXTENSION = (*DeviceExtension).Common.PortDeviceExtension;

    let Irql: KIRQL = KeAcquireInterruptSpinLock((*PortDeviceExtension).HighestDIRQLInterrupt);

    (*PortDeviceExtension).Flags &= !MOUSE_PRESENT;

    KeReleaseInterruptSpinLock((*PortDeviceExtension).HighestDIRQLInterrupt, Irql);
}

#[no_mangle]
pub unsafe extern "system" fn i8042MouInternalDeviceControl(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    let Stack: PIO_STACK_LOCATION = IoGetCurrentIrpStackLocation(Irp);
    (*Irp).IoStatus.Information = 0;
    let DeviceExtension = (*DeviceObject).DeviceExtension as PI8042_MOUSE_EXTENSION;

    let Status: NTSTATUS;

    match (*Stack).Parameters.DeviceIoControl.IoControlCode {
        IOCTL_INTERNAL_MOUSE_CONNECT => {
            let mut WorkItem: PIO_WORKITEM = core::ptr::null_mut();
            let mut WorkItemData: PI8042_HOOK_WORKITEM = core::ptr::null_mut();

            Status = 'block: {
                if (*Stack).Parameters.DeviceIoControl.InputBufferLength
                    != core::mem::size_of::<CONNECT_DATA>() as u32
                {
                    break 'block STATUS_INVALID_PARAMETER;
                }

                (*DeviceExtension).MouseData =
                    *((*Stack).Parameters.DeviceIoControl.Type3InputBuffer as PCONNECT_DATA);

                WorkItem = IoAllocateWorkItem(DeviceObject);
                if WorkItem.is_null() {
                    break 'block STATUS_INSUFFICIENT_RESOURCES;
                }
                WorkItemData = ExAllocatePoolWithTag(
                    NonPagedPool,
                    core::mem::size_of::<I8042_HOOK_WORKITEM>(),
                    I8042PRT_TAG,
                ) as PI8042_HOOK_WORKITEM;
                if WorkItemData.is_null() {
                    break 'block STATUS_NO_MEMORY;
                }
                (*WorkItemData).WorkItem = WorkItem;
                (*WorkItemData).Irp = Irp;

                (*DeviceExtension).Common.Type = Mouse;
                let Size: SIZE_T = (*(*DeviceExtension).Common.PortDeviceExtension)
                    .Settings
                    .MouseDataQueueSize as SIZE_T
                    * core::mem::size_of::<MOUSE_INPUT_DATA>();
                (*DeviceExtension).MouseBuffer =
                    ExAllocatePoolWithTag(NonPagedPool, Size, I8042PRT_TAG) as _;
                if (*DeviceExtension).MouseBuffer.is_null() {
                    break 'block STATUS_NO_MEMORY;
                }
                RtlZeroMemory((*DeviceExtension).MouseBuffer as _, Size);
                (*DeviceExtension).MouseAttributes.InputDataQueueLength =
                    (*(*DeviceExtension).Common.PortDeviceExtension)
                        .Settings
                        .MouseDataQueueSize;
                KeInitializeDpc(
                    core::ptr::addr_of_mut!((*DeviceExtension).DpcMouse),
                    core::mem::transmute::<Option<unsafe extern "system" fn(*mut ntkrnl_sys::ke::KDPC, *mut c_void, *mut c_void, *mut c_void)>, ntkrnl_sys::ddk::PKDEFERRED_ROUTINE>(Some(i8042MouDpcRoutine as unsafe extern "system" fn(*mut ntkrnl_sys::ke::KDPC, *mut c_void, *mut c_void, *mut c_void))),
                    DeviceExtension as _,
                );
                KeInitializeDpc(
                    core::ptr::addr_of_mut!((*DeviceExtension).DpcMouseTimeout),
                    core::mem::transmute::<Option<unsafe extern "system" fn(*mut ntkrnl_sys::ke::KDPC, *mut c_void, *mut c_void, *mut c_void)>, ntkrnl_sys::ddk::PKDEFERRED_ROUTINE>(Some(i8042DpcRoutineMouseTimeout as unsafe extern "system" fn(*mut ntkrnl_sys::ke::KDPC, *mut c_void, *mut c_void, *mut c_void))),
                    DeviceExtension as _,
                );
                KeInitializeTimer(core::ptr::addr_of_mut!(
                    (*DeviceExtension).TimerMouseTimeout
                ));
                (*(*DeviceExtension).Common.PortDeviceExtension).MouseExtension = DeviceExtension;
                (*(*DeviceExtension).Common.PortDeviceExtension).Flags |= MOUSE_CONNECTED;

                IoMarkIrpPending(Irp);
                (*DeviceExtension).MouseState = MouseResetting;
                (*DeviceExtension).MouseResetState = ExpectingReset;
                (*DeviceExtension).MouseHook.IsrWritePort = Some(i8042MouIsrWritePort);
                (*DeviceExtension).MouseHook.QueueMousePacket = Some(i8042MouQueuePacket);
                (*DeviceExtension).MouseHook.CallContext = DeviceExtension as _;
                IoQueueWorkItem(
                    WorkItem,
                    core::mem::transmute::<Option<unsafe extern "system" fn(*mut ntkrnl_sys::ddk::_DEVICE_OBJECT, *mut c_void)>, ntkrnl_sys::ddk::PIO_WORKITEM_ROUTINE>(Some(i8042SendHookWorkItem as unsafe extern "system" fn(*mut ntkrnl_sys::ddk::_DEVICE_OBJECT, *mut c_void))),
                    DelayedWorkQueue,
                    WorkItemData as _,
                );
                return STATUS_PENDING;
            };

            if !(*DeviceExtension).MouseBuffer.is_null() {
                ExFreePoolWithTag((*DeviceExtension).MouseBuffer as _, I8042PRT_TAG);
            }
            if !WorkItem.is_null() {
                IoFreeWorkItem(WorkItem);
            }
            if !WorkItemData.is_null() {
                ExFreePoolWithTag(WorkItemData as _, I8042PRT_TAG);
            }
        }
        IOCTL_INTERNAL_MOUSE_DISCONNECT => {
            Status = STATUS_NOT_IMPLEMENTED;
        }
        IOCTL_INTERNAL_I8042_HOOK_MOUSE => {
            Status = 'block: {
                if (*Stack).Parameters.DeviceIoControl.InputBufferLength
                    < core::mem::size_of::<INTERNAL_I8042_HOOK_MOUSE>() as u32
                {
                    break 'block STATUS_INVALID_PARAMETER;
                }
                let MouseHook = (*Stack).Parameters.DeviceIoControl.Type3InputBuffer
                    as PINTERNAL_I8042_HOOK_MOUSE;

                (*DeviceExtension).MouseHook.Context = (*MouseHook).Context;
                if (*MouseHook).IsrRoutine.is_some() {
                    (*DeviceExtension).MouseHook.IsrRoutine = (*MouseHook).IsrRoutine;
                }

                STATUS_SUCCESS
            };
        }
        IOCTL_INTERNAL_I8042_MOUSE_WRITE_BUFFER => {
            Status = STATUS_NOT_IMPLEMENTED;
        }
        IOCTL_INTERNAL_I8042_MOUSE_START_INFORMATION => {
            Status = STATUS_NOT_IMPLEMENTED;
        }
        IOCTL_MOUSE_QUERY_ATTRIBUTES => {
            Status = 'block: {
                if (*Stack).Parameters.DeviceIoControl.OutputBufferLength
                    < core::mem::size_of::<MOUSE_ATTRIBUTES>() as u32
                {
                    break 'block STATUS_BUFFER_TOO_SMALL;
                }

                *((*Irp).AssociatedIrp.SystemBuffer as PMOUSE_ATTRIBUTES) =
                    (*DeviceExtension).MouseAttributes;
                (*Irp).IoStatus.Information = core::mem::size_of::<MOUSE_ATTRIBUTES>() as _;
                STATUS_SUCCESS
            };
        }
        _ => {
            return ForwardIrpAndForget(DeviceObject, Irp);
        }
    }

    if Status != STATUS_PENDING {
        (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
        IoCompleteRequest(Irp, IO_NO_INCREMENT as i8);
    }
    Status
}

unsafe fn i8042MouInputTestTimeout(DeviceExtension: PI8042_MOUSE_EXTENSION) {
    let mut Now: ULARGE_INTEGER = core::mem::zeroed();

    if (*DeviceExtension).MouseState == MouseExpectingACK
        || (*DeviceExtension).MouseState == MouseResetting
    {
        return;
    }

    Now.QuadPart = KeQueryInterruptTime();

    if (*DeviceExtension).MouseState != MouseIdle {
        if Now.QuadPart - (*DeviceExtension).MousePacketStartTime.QuadPart
            > (*(*DeviceExtension).Common.PortDeviceExtension).Settings.MouseSynchIn100ns as u64
        {
            (*DeviceExtension).MouseState = MouseIdle;
        }
    }

    if (*DeviceExtension).MouseState == MouseIdle {
        (*DeviceExtension).MousePacketStartTime.QuadPart = Now.QuadPart;
    }
}

unsafe fn i8042MouCallIsrHook(
	DeviceExtension: PI8042_MOUSE_EXTENSION,
	Status: UCHAR,
	mut Input: UCHAR,
	ToReturn: PBOOLEAN,
) -> BOOLEAN {
	let mut HookReturn: BOOLEAN;
	let mut HookContinue: BOOLEAN = 0u8;

	if let Some(IsrRoutine) = (*DeviceExtension).MouseHook.IsrRoutine {
		HookReturn = IsrRoutine(
			(*DeviceExtension).MouseHook.Context,
			(*DeviceExtension).MouseBuffer.add((*DeviceExtension).MouseInBuffer as usize),
			core::ptr::addr_of_mut!((*(*DeviceExtension).Common.PortDeviceExtension).Packet),
			Status,
			core::ptr::addr_of_mut!(Input),
			core::ptr::addr_of_mut!(HookContinue),
			core::ptr::addr_of_mut!((*DeviceExtension).MouseState),
			core::ptr::addr_of_mut!((*DeviceExtension).MouseResetState),
		);

		if HookContinue == 0u8 {
			*ToReturn = HookReturn;
			return 1u8;
		}
	}
	0u8
}

unsafe fn i8042MouResetIsr(
	DeviceExtension: PI8042_MOUSE_EXTENSION,
	Status: UCHAR,
	Value: UCHAR,
) -> BOOLEAN {
	let PortDeviceExtension: PPORT_DEVICE_EXTENSION;
	let mut ToReturn: BOOLEAN = 0u8;

	if i8042MouCallIsrHook(DeviceExtension, Status, Value, core::ptr::addr_of_mut!(ToReturn)) != 0 {
		return ToReturn;
	}

	if (*DeviceExtension).MouseState == MouseIdle {
		if 0xAA == Value {
			(*DeviceExtension).MouseState = MouseResetting;
			(*DeviceExtension).MouseResetState = ExpectingReset;
		} else {
			return 0u8;
		}
	} else if (*DeviceExtension).MouseState != MouseResetting {
		return 0u8;
	}

	(*DeviceExtension).MouseTimeoutState = TimeoutStart;
	PortDeviceExtension = (*DeviceExtension).Common.PortDeviceExtension;

	match (*DeviceExtension).MouseResetState as ULONG {
		x if x == ExpectingReset as ULONG => {
			if MOUSE_ACK == Value as _ {
				return 1u8;
			}
			if 0xAA == Value {
				(*DeviceExtension).MouseResetState = next_reset((*DeviceExtension).MouseResetState);
			} else {
				(*PortDeviceExtension).Flags &= !MOUSE_PRESENT;
				(*DeviceExtension).MouseState = MouseIdle;
			}
			1u8
		}
		x if x == ExpectingResetId as ULONG => {
			if MOUSE_ACK == Value as _ {
				return 1u8;
			}
			if 0x00 == Value {
				(*DeviceExtension).MouseResetState = next_reset((*DeviceExtension).MouseResetState);
				(*DeviceExtension).MouseType = GenericPS2;
				(*DeviceExtension).MouseHook.IsrWritePort.unwrap()((*DeviceExtension).MouseHook.CallContext, 0xF2);
			} else {
				(*PortDeviceExtension).Flags &= !MOUSE_PRESENT;
				(*DeviceExtension).MouseState = MouseIdle;
			}
			1u8
		}
		x if x == ExpectingGetDeviceIdACK as ULONG => {
			if MOUSE_ACK == Value as _ {
				(*DeviceExtension).MouseResetState = next_reset((*DeviceExtension).MouseResetState);
			} else if MOUSE_NACK == Value as _ || MOUSE_ERROR == Value as _ {
				(*DeviceExtension).MouseResetState = next_reset((*DeviceExtension).MouseResetState);
				i8042MouResetIsr(DeviceExtension, Status, 0);
			}
			1u8
		}
		x if x == ExpectingGetDeviceIdValue as ULONG => {
			match Value {
				0x02 => {
					(*DeviceExtension).MouseAttributes.MouseIdentifier = BALLPOINT_I8042_HARDWARE as u16;
				}
				0x03 | 0x04 => {
					(*DeviceExtension).MouseAttributes.MouseIdentifier = WHEELMOUSE_I8042_HARDWARE as u16;
				}
				_ => {
					(*DeviceExtension).MouseAttributes.MouseIdentifier = MOUSE_I8042_HARDWARE as u16;
				}
			}
			(*DeviceExtension).MouseResetState = next_reset((*DeviceExtension).MouseResetState);
			(*DeviceExtension).MouseHook.IsrWritePort.unwrap()((*DeviceExtension).MouseHook.CallContext, 0xE8);
			1u8
		}
		x if x == ExpectingSetResolutionDefaultACK as ULONG => {
			(*DeviceExtension).MouseResetState = next_reset((*DeviceExtension).MouseResetState);
			(*DeviceExtension).MouseHook.IsrWritePort.unwrap()((*DeviceExtension).MouseHook.CallContext, 0x00);
			1u8
		}
		x if x == ExpectingSetResolutionDefaultValueACK as ULONG => {
			(*DeviceExtension).MouseResetState = ExpectingSetScaling1to1ACK;
			(*DeviceExtension).MouseHook.IsrWritePort.unwrap()((*DeviceExtension).MouseHook.CallContext, 0xE6);
			1u8
		}
		x if x == ExpectingSetScaling1to1ACK as ULONG || x == ExpectingSetScaling1to1ACK2 as ULONG => {
			(*DeviceExtension).MouseResetState = next_reset((*DeviceExtension).MouseResetState);
			(*DeviceExtension).MouseHook.IsrWritePort.unwrap()((*DeviceExtension).MouseHook.CallContext, 0xE6);
			1u8
		}
		x if x == ExpectingSetScaling1to1ACK3 as ULONG => {
			(*DeviceExtension).MouseResetState = next_reset((*DeviceExtension).MouseResetState);
			(*DeviceExtension).MouseHook.IsrWritePort.unwrap()((*DeviceExtension).MouseHook.CallContext, 0xE9);
			1u8
		}
		x if x == ExpectingReadMouseStatusACK as ULONG => {
			(*DeviceExtension).MouseResetState = next_reset((*DeviceExtension).MouseResetState);
			1u8
		}
		x if x == ExpectingReadMouseStatusByte1 as ULONG => {
			(*DeviceExtension).MouseLogiBuffer[0] = Value;
			(*DeviceExtension).MouseResetState = next_reset((*DeviceExtension).MouseResetState);
			1u8
		}
		x if x == ExpectingReadMouseStatusByte2 as ULONG => {
			(*DeviceExtension).MouseLogiBuffer[1] = Value;
			(*DeviceExtension).MouseResetState = next_reset((*DeviceExtension).MouseResetState);
			1u8
		}
		x if x == ExpectingReadMouseStatusByte3 as ULONG => {
			(*DeviceExtension).MouseLogiBuffer[2] = Value;
			if (*DeviceExtension).MouseLogiBuffer[1] != 0 {
				(*DeviceExtension).MouseAttributes.NumberOfButtons = (*DeviceExtension).MouseLogiBuffer[1] as _;
				(*DeviceExtension).MouseType = Ps2pp;
				(*DeviceExtension).MouseHook.IsrWritePort.unwrap()((*DeviceExtension).MouseHook.CallContext, 0xF3);
				(*DeviceExtension).MouseResetState = ExpectingSetSamplingRateACK;
				return 1u8;
			}
			(*DeviceExtension).MouseResetState = EnableWheel;
			i8042MouResetIsr(DeviceExtension, Status, Value);
			1u8
		}
		x if x == EnableWheel as ULONG => {
			(*DeviceExtension).MouseHook.IsrWritePort.unwrap()((*DeviceExtension).MouseHook.CallContext, 0xF3);
			(*DeviceExtension).MouseResetState = I8042Reserved1001;
			1u8
		}
		1001 => {
			(*DeviceExtension).MouseHook.IsrWritePort.unwrap()((*DeviceExtension).MouseHook.CallContext, 0xC8);
			(*DeviceExtension).MouseResetState = next_reset((*DeviceExtension).MouseResetState);
			1u8
		}
		1002 | 1004 => {
			(*DeviceExtension).MouseHook.IsrWritePort.unwrap()((*DeviceExtension).MouseHook.CallContext, 0xF3);
			(*DeviceExtension).MouseResetState = next_reset((*DeviceExtension).MouseResetState);
			1u8
		}
		1003 => {
			(*DeviceExtension).MouseHook.IsrWritePort.unwrap()((*DeviceExtension).MouseHook.CallContext, 0x64);
			(*DeviceExtension).MouseResetState = next_reset((*DeviceExtension).MouseResetState);
			1u8
		}
		1005 => {
			(*DeviceExtension).MouseHook.IsrWritePort.unwrap()((*DeviceExtension).MouseHook.CallContext, 0x50);
			(*DeviceExtension).MouseResetState = next_reset((*DeviceExtension).MouseResetState);
			1u8
		}
		1006 => {
			(*DeviceExtension).MouseHook.IsrWritePort.unwrap()((*DeviceExtension).MouseHook.CallContext, 0xF2);
			(*DeviceExtension).MouseResetState = next_reset((*DeviceExtension).MouseResetState);
			1u8
		}
		1007 => {
			(*DeviceExtension).MouseResetState = next_reset((*DeviceExtension).MouseResetState);
			1u8
		}
		1008 => {
			if 0x03 == Value {
				(*DeviceExtension).MouseAttributes.NumberOfButtons = 3;
				(*DeviceExtension).MouseAttributes.MouseIdentifier = WHEELMOUSE_I8042_HARDWARE as u16;
				(*DeviceExtension).MouseType = Intellimouse;
				(*DeviceExtension).MouseResetState = Enable5Buttons;
				i8042MouResetIsr(DeviceExtension, Status, Value);
			} else {
				(*DeviceExtension).MouseHook.IsrWritePort.unwrap()((*DeviceExtension).MouseHook.CallContext, 0xF3);
				(*DeviceExtension).MouseResetState = ExpectingSetSamplingRateACK;
			}
			1u8
		}
		x if x == Enable5Buttons as ULONG => {
			(*DeviceExtension).MouseHook.IsrWritePort.unwrap()((*DeviceExtension).MouseHook.CallContext, 0xF3);
			(*DeviceExtension).MouseResetState = I8042Reserved1021;
			1u8
		}
		1022 | 1024 => {
			(*DeviceExtension).MouseHook.IsrWritePort.unwrap()((*DeviceExtension).MouseHook.CallContext, 0xF3);
			(*DeviceExtension).MouseResetState = next_reset((*DeviceExtension).MouseResetState);
			1u8
		}
		1021 | 1023 => {
			(*DeviceExtension).MouseHook.IsrWritePort.unwrap()((*DeviceExtension).MouseHook.CallContext, 0xC8);
			(*DeviceExtension).MouseResetState = next_reset((*DeviceExtension).MouseResetState);
			1u8
		}
		1025 => {
			(*DeviceExtension).MouseHook.IsrWritePort.unwrap()((*DeviceExtension).MouseHook.CallContext, 0x50);
			(*DeviceExtension).MouseResetState = next_reset((*DeviceExtension).MouseResetState);
			1u8
		}
		1026 => {
			(*DeviceExtension).MouseHook.IsrWritePort.unwrap()((*DeviceExtension).MouseHook.CallContext, 0xF2);
			(*DeviceExtension).MouseResetState = next_reset((*DeviceExtension).MouseResetState);
			1u8
		}
		1027 => {
			if 0x04 == Value {
				(*DeviceExtension).MouseAttributes.NumberOfButtons = 5;
				(*DeviceExtension).MouseAttributes.MouseIdentifier = WHEELMOUSE_I8042_HARDWARE as u16;
				(*DeviceExtension).MouseType = IntellimouseExplorer;
			}
			(*DeviceExtension).MouseHook.IsrWritePort.unwrap()((*DeviceExtension).MouseHook.CallContext, 0xF3);
			(*DeviceExtension).MouseResetState = ExpectingSetSamplingRateACK;
			1u8
		}
		x if x == ExpectingSetSamplingRateACK as ULONG => {
			(*DeviceExtension).MouseHook.IsrWritePort.unwrap()(
				(*DeviceExtension).MouseHook.CallContext,
				(*DeviceExtension).MouseAttributes.SampleRate as UCHAR,
			);
			(*DeviceExtension).MouseResetState = next_reset((*DeviceExtension).MouseResetState);
			1u8
		}
		x if x == ExpectingSetSamplingRateValueACK as ULONG => {
			if MOUSE_NACK == Value as _ {
				(*DeviceExtension).MouseHook.IsrWritePort.unwrap()((*DeviceExtension).MouseHook.CallContext, 0x3C);
				(*DeviceExtension).MouseAttributes.SampleRate = (*PortDeviceExtension).Settings.SampleRate as USHORT;
				(*DeviceExtension).MouseResetState = I8042Reserved1040;
				return 1u8;
			}
			(*DeviceExtension).MouseHook.IsrWritePort.unwrap()((*DeviceExtension).MouseHook.CallContext, 0xE8);
			(*DeviceExtension).MouseResetState = ExpectingFinalResolutionACK;
			1u8
		}
		1040 => {
			(*DeviceExtension).MouseHook.IsrWritePort.unwrap()((*DeviceExtension).MouseHook.CallContext, 0xE8);
			(*DeviceExtension).MouseResetState = ExpectingFinalResolutionACK;
			1u8
		}
		x if x == ExpectingFinalResolutionACK as ULONG => {
			(*DeviceExtension).MouseHook.IsrWritePort.unwrap()(
				(*DeviceExtension).MouseHook.CallContext,
				((*PortDeviceExtension).Settings.MouseResolution & 0xff) as UCHAR,
			);
			(*DeviceExtension).MouseResetState = ExpectingFinalResolutionValueACK;
			1u8
		}
		x if x == ExpectingFinalResolutionValueACK as ULONG => {
			(*DeviceExtension).MouseHook.IsrWritePort.unwrap()((*DeviceExtension).MouseHook.CallContext, 0xF4);
			(*DeviceExtension).MouseResetState = ExpectingEnableACK;
			1u8
		}
		x if x == ExpectingEnableACK as ULONG => {
			(*PortDeviceExtension).Flags |= MOUSE_PRESENT;
			(*DeviceExtension).MouseState = MouseIdle;
			(*DeviceExtension).MouseTimeoutState = TimeoutCancel;
			1u8
		}
		_ => 0u8,
	}
}

#[no_mangle]
pub unsafe extern "system" fn i8042MouInterruptService(
    Interrupt: PKINTERRUPT,
    Context: PVOID,
) -> BOOLEAN {
    let DeviceExtension: PI8042_MOUSE_EXTENSION;
    let PortDeviceExtension: PPORT_DEVICE_EXTENSION;
    let mut Counter: ULONG;
    let mut Output: UCHAR = 0;
    let mut PortStatus: UCHAR = 0;
    let mut Status: NTSTATUS;

    let _ = Interrupt;

    DeviceExtension = Context as PI8042_MOUSE_EXTENSION;

    PortDeviceExtension = (*DeviceExtension).Common.PortDeviceExtension;
    Counter = (*PortDeviceExtension).Settings.PollStatusIterations;

    while Counter != 0 {
        Status = i8042ReadStatus(PortDeviceExtension, core::ptr::addr_of_mut!(PortStatus));
        if !NT_SUCCESS(Status) {
            return 0u8;
        }
        Status = i8042ReadMouseData(PortDeviceExtension, core::ptr::addr_of_mut!(Output));
        if NT_SUCCESS(Status) {
            break;
        }
        KeStallExecutionProcessor(1);
        Counter -= 1;
    }
    if Counter == 0 {
        return 0u8;
    }

    /* Only consult the shared packet machine when the in-flight packet
     * actually TARGETS THE MOUSE (PacketPort == CTRL_WRITE_MOUSE). The C
     * original calls i8042PacketIsr unconditionally from both ISRs, so a
     * keyboard packet in flight (kbdclass SET_LEDS/typematic init) STEALS the
     * mouse's byte: the aux 0xFA that ACKs a mouse reset step gets consumed
     * as the keyboard packet's ACK, the mouse reset state machine stalls
     * forever (observed 2026-07-06 stuck at wheel-detect state 1001, byte
     * trace in serial), and the PS/2 mouse is dead for the whole session —
     * the timeout DPC just clears MOUSE_PRESENT, there is no retry. This is a
     * latent upstream ReactOS race that our boot timing hits 100%. */
    if (*PortDeviceExtension).Packet.State != Idle
        && (*PortDeviceExtension).PacketPort == CTRL_WRITE_MOUSE as UCHAR
        && i8042PacketIsr(PortDeviceExtension, Output) != 0
    {
        if (*PortDeviceExtension).PacketComplete != 0 {
            KeInsertQueueDpc(
                core::ptr::addr_of_mut!((*DeviceExtension).DpcMouse),
                core::ptr::null_mut(),
                core::ptr::null_mut(),
            );
        }
        return 1u8;
    }

    i8042MouInputTestTimeout(DeviceExtension);

    if i8042MouResetIsr(DeviceExtension, PortStatus, Output) != 0 {
        if NoChange != (*DeviceExtension).MouseTimeoutState {
            KeInsertQueueDpc(
                core::ptr::addr_of_mut!((*DeviceExtension).DpcMouse),
                core::ptr::null_mut(),
                core::ptr::null_mut(),
            );
        }
        return 1u8;
    }

    if (*DeviceExtension).MouseType == Ps2pp {
        i8042MouHandlePs2pp(DeviceExtension, Output);
    } else {
        i8042MouHandle(DeviceExtension, Output);
    }

    1u8
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:     ReactOS i8042 (ps/2 keyboard-mouse controller) driver
//  * LICENSE:     GPL - See COPYING in the top level directory
//  * FILE:        drivers/input/i8042prt/mouse.c
//  * PURPOSE:     Mouse specific functions
//  * PROGRAMMERS: Copyright Victor Kirhenshtein (sauros@iname.com)
//                 Copyright Jason Filby (jasonfilby@yahoo.com)
//                 Copyright Martijn Vernooij (o112w8r02@sneakemail.com)
//                 Copyright 2006-2007 Herv� Poussineau (hpoussin@reactos.org)
//                 Copyright 2008 Colin Finck (mail@colinfinck.de)
//  */
//
// /* INCLUDES ****************************************************************/
//
// #include "i8042prt.h"
//
// #include <debug.h>
//
// /* FUNCTIONS *****************************************************************/
//
// static KDEFERRED_ROUTINE i8042MouDpcRoutine;
// static KDEFERRED_ROUTINE i8042DpcRoutineMouseTimeout;
//
// /*
//  * These functions are callbacks for filter driver custom interrupt
//  * service routines.
//  */
// static VOID NTAPI
// i8042MouIsrWritePort(
// 	IN PVOID Context,
// 	IN UCHAR Value)
// {
// 	PI8042_MOUSE_EXTENSION DeviceExtension;
//
// 	DeviceExtension = (PI8042_MOUSE_EXTENSION)Context;
//
// 	if (DeviceExtension->MouseHook.IsrWritePort != i8042MouIsrWritePort)
// 	{
// 		DeviceExtension->MouseHook.IsrWritePort(
// 			DeviceExtension->MouseHook.CallContext,
// 			Value);
// 	}
// 	else
// 		i8042IsrWritePort(DeviceExtension->Common.PortDeviceExtension, Value, CTRL_WRITE_MOUSE);
// }
//
// static VOID NTAPI
// i8042MouQueuePacket(
// 	IN PVOID Context)
// {
// 	PI8042_MOUSE_EXTENSION DeviceExtension;
//
// 	DeviceExtension = (PI8042_MOUSE_EXTENSION)Context;
//
// 	DeviceExtension->MouseComplete = TRUE;
// 	DeviceExtension->MouseInBuffer++;
// 	if (DeviceExtension->MouseInBuffer >= DeviceExtension->Common.PortDeviceExtension->Settings.MouseDataQueueSize)
// 	{
// 		WARN_(I8042PRT, "Mouse buffer overflow\n");
// 		DeviceExtension->MouseInBuffer--;
// 	}
//
// 	TRACE_(I8042PRT, "Irq completes mouse packet\n");
// 	KeInsertQueueDpc(&DeviceExtension->DpcMouse, NULL, NULL);
// }
//
// VOID
// i8042MouHandle(
// 	IN PI8042_MOUSE_EXTENSION DeviceExtension,
// 	IN UCHAR Output)
// {
// 	PMOUSE_INPUT_DATA MouseInput;
// 	CHAR Scroll;
//
// 	MouseInput = DeviceExtension->MouseBuffer + DeviceExtension->MouseInBuffer;
//
// 	switch (DeviceExtension->MouseState)
// 	{
// 		case MouseIdle:
// 			/* This bit should be 1, if not drop the packet, we
// 			 * might be lucky and get in sync again
// 			 */
// 			if (!(Output & 8)) {
// 				WARN_(I8042PRT, "Bad input, dropping..\n");
// 				return;
// 			}
//
// 			MouseInput->Buttons = 0;
// 			MouseInput->RawButtons = 0;
// 			MouseInput->Flags = MOUSE_MOVE_RELATIVE;
//
// 			/* Note how we ignore the overflow bits, like Windows
// 			 * is said to do. There's no reasonable thing to do
// 			 * anyway.
// 			 */
//
// 			if (Output & 16)
// 				MouseInput->LastX = 1;
// 			else
// 				MouseInput->LastX = 0;
// 			if (Output & 32)
// 				MouseInput->LastY = 1;
// 			else
// 				MouseInput->LastY = 0;
//
// 			if (Output & 1)
// 				MouseInput->RawButtons |= MOUSE_LEFT_BUTTON_DOWN;
// 			if (Output & 2)
// 				MouseInput->RawButtons |= MOUSE_RIGHT_BUTTON_DOWN;
// 			if (Output & 4)
// 				MouseInput->RawButtons |= MOUSE_MIDDLE_BUTTON_DOWN;
//
// 			DeviceExtension->MouseState = XMovement;
// 			break;
//
// 		case XMovement:
// 			if (MouseInput->LastX)
// 				MouseInput->LastX = (LONG) Output - 256;
// 			else
// 				MouseInput->LastX = Output;
//
// 			DeviceExtension->MouseState = YMovement;
// 			break;
//
// 		case YMovement:
// 			if (MouseInput->LastY)
// 				MouseInput->LastY = (LONG)Output - 256;
// 			else
// 				MouseInput->LastY = (LONG)Output;
//
// 			/* Windows wants it the other way around */
// 			MouseInput->LastY = -MouseInput->LastY;
//
// 			if (DeviceExtension->MouseType == GenericPS2 ||
// 			    DeviceExtension->MouseType == Ps2pp)
// 			{
// 				i8042MouHandleButtons(
// 					DeviceExtension,
// 					MOUSE_LEFT_BUTTON_DOWN |
// 					MOUSE_RIGHT_BUTTON_DOWN |
// 					MOUSE_MIDDLE_BUTTON_DOWN);
// 				DeviceExtension->MouseHook.QueueMousePacket(DeviceExtension->MouseHook.CallContext);
// 				DeviceExtension->MouseState = MouseIdle;
// 			}
// 			else
// 			{
// 				DeviceExtension->MouseState = ZMovement;
// 			}
// 			break;
//
// 		case ZMovement:
// 			Scroll = Output & 0x0f;
// 			if (Scroll & 8)
// 				Scroll |= 0xf0;
//
// 			if (Scroll)
// 			{
// 				MouseInput->RawButtons |= MOUSE_WHEEL;
// 				MouseInput->ButtonData = (USHORT)(Scroll * -WHEEL_DELTA);
// 			}
//
// 			if (DeviceExtension->MouseType == IntellimouseExplorer)
// 			{
// 				if (Output & 16)
// 					MouseInput->RawButtons |= MOUSE_BUTTON_4_DOWN;
// 				if (Output & 32)
// 					MouseInput->RawButtons |= MOUSE_BUTTON_5_DOWN;
// 			}
// 			i8042MouHandleButtons(
// 				DeviceExtension,
// 				MOUSE_LEFT_BUTTON_DOWN |
// 				MOUSE_RIGHT_BUTTON_DOWN |
// 				MOUSE_MIDDLE_BUTTON_DOWN |
// 				MOUSE_BUTTON_4_DOWN |
// 				MOUSE_BUTTON_5_DOWN);
// 			DeviceExtension->MouseHook.QueueMousePacket(DeviceExtension->MouseHook.CallContext);
// 			DeviceExtension->MouseState = MouseIdle;
// 			break;
//
// 		default:
// 			ERR_(I8042PRT, "Unexpected state 0x%lx!\n", DeviceExtension->MouseState);
// 			ASSERT(FALSE);
// 	}
// }
//
// /*
//  * Updates ButtonFlags according to RawButtons and a saved state;
//  * Only takes in account the bits that are set in Mask
//  */
// VOID
// i8042MouHandleButtons(
// 	IN PI8042_MOUSE_EXTENSION DeviceExtension,
// 	IN USHORT Mask)
// {
// 	PMOUSE_INPUT_DATA MouseInput;
// 	USHORT NewButtonData;
// 	USHORT ButtonDiff;
//
// 	MouseInput = DeviceExtension->MouseBuffer + DeviceExtension->MouseInBuffer;
// 	NewButtonData = (USHORT)(MouseInput->RawButtons & Mask);
// 	ButtonDiff = (NewButtonData ^ DeviceExtension->MouseButtonState) & Mask;
//
// 	/* Note that the defines are such:
// 	 * MOUSE_LEFT_BUTTON_DOWN 1
// 	 * MOUSE_LEFT_BUTTON_UP   2
// 	 */
// 	MouseInput->ButtonFlags |= (NewButtonData & ButtonDiff) |
// 		(((~(NewButtonData)) << 1) & (ButtonDiff << 1)) |
// 		(MouseInput->RawButtons & 0xfc00);
//
// 	INFO_(I8042PRT, "Left raw/up/down: %u/%u/%u\n",
// 		MouseInput->RawButtons & MOUSE_LEFT_BUTTON_DOWN,
// 		MouseInput->ButtonFlags & MOUSE_LEFT_BUTTON_DOWN,
// 		MouseInput->ButtonFlags & MOUSE_LEFT_BUTTON_UP);
//
// 	DeviceExtension->MouseButtonState =
// 		(DeviceExtension->MouseButtonState & ~Mask) | (NewButtonData & Mask);
// }
//
// /* Does final initializations for the mouse. This method
//  * is called just before connecting the interrupt.
//  */
// NTSTATUS
// i8042MouInitialize(
// 	IN PI8042_MOUSE_EXTENSION DeviceExtension)
// {
// 	NTSTATUS Status;
// 	UCHAR Value;
//
// 	/* Enable the PS/2 mouse port */
// 	i8042Write(DeviceExtension->Common.PortDeviceExtension, DeviceExtension->Common.PortDeviceExtension->ControlPort, MOUSE_ENAB);
//
// 	/* Enable the mouse */
// 	if(!i8042IsrWritePort(DeviceExtension->Common.PortDeviceExtension, MOU_ENAB, CTRL_WRITE_MOUSE))
// 	{
// 		WARN_(I8042PRT, "Failed to enable mouse!\n");
// 		return STATUS_IO_DEVICE_ERROR;
// 	}
//
// 	Status = i8042ReadDataWait(DeviceExtension->Common.PortDeviceExtension, &Value);
// 	if (!NT_SUCCESS(Status))
// 	{
// 		WARN_(I8042PRT, "Failed to read the response of MOU_ENAB, status 0x%08lx\n", Status);
// 		return Status;
// 	}
//
// 	if(Value == MOUSE_ACK)
// 	{
// 		INFO_(I8042PRT, "Mouse was enabled successfully!\n");
// 		return STATUS_SUCCESS;
// 	}
//
// 	WARN_(I8042PRT, "Got 0x%02x instead of 0xFA\n", Value);
// 	return STATUS_IO_DEVICE_ERROR;
// }
//
// static VOID NTAPI
// i8042MouDpcRoutine(
// 	IN PKDPC Dpc,
// 	IN PVOID DeferredContext,
// 	IN PVOID SystemArgument1,
// 	IN PVOID SystemArgument2)
// {
// 	PI8042_MOUSE_EXTENSION DeviceExtension;
// 	PPORT_DEVICE_EXTENSION PortDeviceExtension;
// 	ULONG MouseTransferred = 0;
// 	ULONG MouseInBufferCopy;
// 	KIRQL Irql;
// 	LARGE_INTEGER Timeout;
//
// 	UNREFERENCED_PARAMETER(Dpc);
// 	UNREFERENCED_PARAMETER(SystemArgument1);
// 	UNREFERENCED_PARAMETER(SystemArgument2);
//
// 	__analysis_assume(DeferredContext != NULL);
// 	DeviceExtension = DeferredContext;
// 	PortDeviceExtension = DeviceExtension->Common.PortDeviceExtension;
//
// 	switch (DeviceExtension->MouseTimeoutState)
// 	{
// 		case TimeoutStart:
// 		{
// 			DeviceExtension->MouseTimeoutState = NoChange;
// 			if (DeviceExtension->MouseTimeoutActive &&
// 			    !KeCancelTimer(&DeviceExtension->TimerMouseTimeout))
// 			{
// 				/* The timer fired already, give up */
// 				DeviceExtension->MouseTimeoutActive = FALSE;
// 				return;
// 			}
//
// 			Timeout.QuadPart = -15000000; /* 1.5 seconds, should be enough */
//
// 			KeSetTimer(
// 				&DeviceExtension->TimerMouseTimeout,
// 				Timeout,
// 				&DeviceExtension->DpcMouseTimeout);
// 			DeviceExtension->MouseTimeoutActive = TRUE;
// 			return;
// 		}
//
// 		case TimeoutCancel:
// 		{
// 			DeviceExtension->MouseTimeoutState = NoChange;
// 			KeCancelTimer(&DeviceExtension->TimerMouseTimeout);
// 			DeviceExtension->MouseTimeoutActive = FALSE;
// 		}
//
// 		default:
// 			;/* nothing, don't want a warning */
// 	}
//
// 	/* Should be unlikely */
// 	if (!DeviceExtension->MouseComplete)
// 		return;
//
// 	Irql = KeAcquireInterruptSpinLock(PortDeviceExtension->HighestDIRQLInterrupt);
//
// 	DeviceExtension->MouseComplete = FALSE;
// 	MouseInBufferCopy = DeviceExtension->MouseInBuffer;
//
// 	KeReleaseInterruptSpinLock(PortDeviceExtension->HighestDIRQLInterrupt, Irql);
//
// 	TRACE_(I8042PRT, "Send a mouse packet\n");
//
// 	if (!DeviceExtension->MouseData.ClassService)
// 		return;
//
// 	INFO_(I8042PRT, "Sending %lu mouse move(s)\n", MouseInBufferCopy);
// 	(*(PSERVICE_CALLBACK_ROUTINE)DeviceExtension->MouseData.ClassService)(
// 		DeviceExtension->MouseData.ClassDeviceObject,
// 		DeviceExtension->MouseBuffer,
// 		DeviceExtension->MouseBuffer + MouseInBufferCopy,
// 		&MouseTransferred);
//
// 	Irql = KeAcquireInterruptSpinLock(PortDeviceExtension->HighestDIRQLInterrupt);
// 	DeviceExtension->MouseInBuffer -= MouseTransferred;
// 	if (DeviceExtension->MouseInBuffer)
// 		RtlMoveMemory(
// 			DeviceExtension->MouseBuffer,
// 			DeviceExtension->MouseBuffer + MouseTransferred,
// 			DeviceExtension->MouseInBuffer * sizeof(MOUSE_INPUT_DATA));
// 	KeReleaseInterruptSpinLock(PortDeviceExtension->HighestDIRQLInterrupt, Irql);
// }
//
// /* This timer DPC will be called when the mouse reset times out.
//  * I'll just send the 'disable mouse port' command to the controller
//  * and say the mouse doesn't exist.
//  */
// static VOID NTAPI
// i8042DpcRoutineMouseTimeout(
// 	IN PKDPC Dpc,
// 	IN PVOID DeferredContext,
// 	IN PVOID SystemArgument1,
// 	IN PVOID SystemArgument2)
// {
// 	PI8042_MOUSE_EXTENSION DeviceExtension;
// 	PPORT_DEVICE_EXTENSION PortDeviceExtension;
// 	KIRQL Irql;
//
// 	UNREFERENCED_PARAMETER(Dpc);
// 	UNREFERENCED_PARAMETER(SystemArgument1);
// 	UNREFERENCED_PARAMETER(SystemArgument2);
//
// 	__analysis_assume(DeferredContext != NULL);
// 	DeviceExtension = DeferredContext;
// 	PortDeviceExtension = DeviceExtension->Common.PortDeviceExtension;
//
// 	Irql = KeAcquireInterruptSpinLock(PortDeviceExtension->HighestDIRQLInterrupt);
//
// 	WARN_(I8042PRT, "Mouse initialization timeout! (substate %x)\n",
// 		DeviceExtension->MouseResetState);
//
// 	PortDeviceExtension->Flags &= ~MOUSE_PRESENT;
//
// 	KeReleaseInterruptSpinLock(PortDeviceExtension->HighestDIRQLInterrupt, Irql);
// }
//
// /*
//  * Runs the mouse IOCTL_INTERNAL dispatch.
//  */
// NTSTATUS NTAPI
// i8042MouInternalDeviceControl(
// 	IN PDEVICE_OBJECT DeviceObject,
// 	IN PIRP Irp)
// {
// 	PIO_STACK_LOCATION Stack;
// 	PI8042_MOUSE_EXTENSION DeviceExtension;
// 	NTSTATUS Status;
//
// 	Stack = IoGetCurrentIrpStackLocation(Irp);
// 	Irp->IoStatus.Information = 0;
// 	DeviceExtension = (PI8042_MOUSE_EXTENSION)DeviceObject->DeviceExtension;
//
// 	switch (Stack->Parameters.DeviceIoControl.IoControlCode)
// 	{
// 		case IOCTL_INTERNAL_MOUSE_CONNECT:
// 		{
// 			SIZE_T Size;
// 			PIO_WORKITEM WorkItem = NULL;
// 			PI8042_HOOK_WORKITEM WorkItemData = NULL;
//
// 			TRACE_(I8042PRT, "IRP_MJ_INTERNAL_DEVICE_CONTROL / IOCTL_INTERNAL_MOUSE_CONNECT\n");
// 			if (Stack->Parameters.DeviceIoControl.InputBufferLength != sizeof(CONNECT_DATA))
// 			{
// 				Status = STATUS_INVALID_PARAMETER;
// 				goto cleanup;
// 			}
//
// 			DeviceExtension->MouseData =
// 				*((PCONNECT_DATA)Stack->Parameters.DeviceIoControl.Type3InputBuffer);
//
// 			/* Send IOCTL_INTERNAL_I8042_HOOK_MOUSE to device stack */
// 			WorkItem = IoAllocateWorkItem(DeviceObject);
// 			if (!WorkItem)
// 			{
// 				WARN_(I8042PRT, "IoAllocateWorkItem() failed\n");
// 				Status = STATUS_INSUFFICIENT_RESOURCES;
// 				goto cleanup;
// 			}
// 			WorkItemData = ExAllocatePoolWithTag(
// 				NonPagedPool,
// 				sizeof(I8042_HOOK_WORKITEM),
// 				I8042PRT_TAG);
// 			if (!WorkItemData)
// 			{
// 				WARN_(I8042PRT, "ExAllocatePoolWithTag() failed\n");
// 				Status = STATUS_NO_MEMORY;
// 				goto cleanup;
// 			}
// 			WorkItemData->WorkItem = WorkItem;
// 			WorkItemData->Irp = Irp;
//
// 			/* Initialize extension */
// 			DeviceExtension->Common.Type = Mouse;
// 			Size = DeviceExtension->Common.PortDeviceExtension->Settings.MouseDataQueueSize * sizeof(MOUSE_INPUT_DATA);
// 			DeviceExtension->MouseBuffer = ExAllocatePoolWithTag(
// 				NonPagedPool,
// 				Size,
// 				I8042PRT_TAG);
// 			if (!DeviceExtension->MouseBuffer)
// 			{
// 				WARN_(I8042PRT, "ExAllocatePoolWithTag() failed\n");
// 				Status = STATUS_NO_MEMORY;
// 				goto cleanup;
// 			}
// 			RtlZeroMemory(DeviceExtension->MouseBuffer, Size);
// 			DeviceExtension->MouseAttributes.InputDataQueueLength =
// 				DeviceExtension->Common.PortDeviceExtension->Settings.MouseDataQueueSize;
// 			KeInitializeDpc(
// 				&DeviceExtension->DpcMouse,
// 				i8042MouDpcRoutine,
// 				DeviceExtension);
// 			KeInitializeDpc(
// 				&DeviceExtension->DpcMouseTimeout,
// 				i8042DpcRoutineMouseTimeout,
// 				DeviceExtension);
// 			KeInitializeTimer(&DeviceExtension->TimerMouseTimeout);
// 			DeviceExtension->Common.PortDeviceExtension->MouseExtension = DeviceExtension;
// 			DeviceExtension->Common.PortDeviceExtension->Flags |= MOUSE_CONNECTED;
//
// 			IoMarkIrpPending(Irp);
// 			DeviceExtension->MouseState = MouseResetting;
// 			DeviceExtension->MouseResetState = ExpectingReset;
// 			DeviceExtension->MouseHook.IsrWritePort = i8042MouIsrWritePort;
// 			DeviceExtension->MouseHook.QueueMousePacket = i8042MouQueuePacket;
// 			DeviceExtension->MouseHook.CallContext = DeviceExtension;
// 			IoQueueWorkItem(WorkItem,
// 				i8042SendHookWorkItem,
// 				DelayedWorkQueue,
// 				WorkItemData);
// 			Status = STATUS_PENDING;
// 			break;
//
// cleanup:
// 			if (DeviceExtension->MouseBuffer)
// 				ExFreePoolWithTag(DeviceExtension->MouseBuffer, I8042PRT_TAG);
// 			if (WorkItem)
// 				IoFreeWorkItem(WorkItem);
// 			if (WorkItemData)
// 				ExFreePoolWithTag(WorkItemData, I8042PRT_TAG);
// 			break;
// 		}
// 		case IOCTL_INTERNAL_MOUSE_DISCONNECT:
// 		{
// 			TRACE_(I8042PRT, "IRP_MJ_INTERNAL_DEVICE_CONTROL / IOCTL_INTERNAL_MOUSE_DISCONNECT\n");
// 			/* MSDN says that operation is to implemented.
// 			 * To implement it, we just have to do:
// 			 * DeviceExtension->MouseData.ClassService = NULL;
// 			 */
// 			Status = STATUS_NOT_IMPLEMENTED;
// 			break;
// 		}
// 		case IOCTL_INTERNAL_I8042_HOOK_MOUSE:
// 		{
// 			PINTERNAL_I8042_HOOK_MOUSE MouseHook;
// 			TRACE_(I8042PRT, "IRP_MJ_INTERNAL_DEVICE_CONTROL / IOCTL_INTERNAL_I8042_HOOK_MOUSE\n");
// 			if (Stack->Parameters.DeviceIoControl.InputBufferLength < sizeof(INTERNAL_I8042_HOOK_MOUSE))
// 			{
// 				Status = STATUS_INVALID_PARAMETER;
// 				break;
// 			}
// 			MouseHook = (PINTERNAL_I8042_HOOK_MOUSE)Stack->Parameters.DeviceIoControl.Type3InputBuffer;
//
// 			DeviceExtension->MouseHook.Context = MouseHook->Context;
// 			if (MouseHook->IsrRoutine)
// 				DeviceExtension->MouseHook.IsrRoutine = MouseHook->IsrRoutine;
//
// 			Status = STATUS_SUCCESS;
// 			break;
// 		}
// 		case IOCTL_INTERNAL_I8042_MOUSE_WRITE_BUFFER:
// 		{
// 			DPRINT1("IOCTL_INTERNAL_I8042_MOUSE_WRITE_BUFFER not implemented\n");
// 			Status = STATUS_NOT_IMPLEMENTED;
// 			break;
// 		}
// 		case IOCTL_INTERNAL_I8042_MOUSE_START_INFORMATION:
// 		{
// 			DPRINT1("IOCTL_INTERNAL_I8042_MOUSE_START_INFORMATION not implemented\n");
// 			Status = STATUS_NOT_IMPLEMENTED;
// 			break;
// 		}
// 		case IOCTL_MOUSE_QUERY_ATTRIBUTES:
// 		{
// 			TRACE_(I8042PRT, "IRP_MJ_INTERNAL_DEVICE_CONTROL / IOCTL_MOUSE_QUERY_ATTRIBUTES\n");
// 			if (Stack->Parameters.DeviceIoControl.OutputBufferLength < sizeof(MOUSE_ATTRIBUTES))
// 			{
// 				Status = STATUS_BUFFER_TOO_SMALL;
// 				break;
// 			}
//
// 			*(PMOUSE_ATTRIBUTES) Irp->AssociatedIrp.SystemBuffer = DeviceExtension->MouseAttributes;
// 			Irp->IoStatus.Information = sizeof(MOUSE_ATTRIBUTES);
// 			Status = STATUS_SUCCESS;
// 			break;
// 		}
// 		default:
// 		{
// 			ERR_(I8042PRT, "IRP_MJ_INTERNAL_DEVICE_CONTROL / unknown ioctl code 0x%lx\n",
// 				Stack->Parameters.DeviceIoControl.IoControlCode);
// 			ASSERT(FALSE);
// 			return ForwardIrpAndForget(DeviceObject, Irp);
// 		}
// 	}
//
// 	if (Status != STATUS_PENDING)
// 	{
// 		Irp->IoStatus.Status = Status;
// 		IoCompleteRequest(Irp, IO_NO_INCREMENT);
// 	}
// 	return Status;
// }
//
// /* Test if packets are taking too long to come in. If they do, we
//  * might have gotten out of sync and should just drop what we have.
//  *
//  * If we want to be totally right, we'd also have to keep a count of
//  * errors, and totally reset the mouse after too much of them (can
//  * happen if the user is using a KVM switch and an OS on another port
//  * resets the mouse, or if the user hotplugs the mouse, or if we're just
//  * generally unlucky). Also note the input parsing routine where we
//  * drop invalid input packets.
//  */
// static VOID
// i8042MouInputTestTimeout(
// 	IN PI8042_MOUSE_EXTENSION DeviceExtension)
// {
// 	ULARGE_INTEGER Now;
//
// 	if (DeviceExtension->MouseState == MouseExpectingACK ||
// 	    DeviceExtension->MouseState == MouseResetting)
// 		return;
//
// 	Now.QuadPart = KeQueryInterruptTime();
//
// 	if (DeviceExtension->MouseState != MouseIdle) {
// 		/* Check if the last byte came too long ago */
// 		if (Now.QuadPart - DeviceExtension->MousePacketStartTime.QuadPart >
// 		    DeviceExtension->Common.PortDeviceExtension->Settings.MouseSynchIn100ns)
// 		{
// 			WARN_(I8042PRT, "Mouse input packet timeout\n");
// 			DeviceExtension->MouseState = MouseIdle;
// 		}
// 	}
//
// 	if (DeviceExtension->MouseState == MouseIdle)
// 		DeviceExtension->MousePacketStartTime.QuadPart = Now.QuadPart;
// }
//
// /*
//  * Call the customization hook. The ToReturn parameter is about wether
//  * we should go on with the interrupt. The return value is what
//  * we should return (indicating to the system wether someone else
//  * should try to handle the interrupt)
//  */
// static BOOLEAN
// i8042MouCallIsrHook(
// 	IN PI8042_MOUSE_EXTENSION DeviceExtension,
// 	IN UCHAR Status,
// 	IN UCHAR Input,
// 	OUT PBOOLEAN ToReturn)
// {
// 	BOOLEAN HookReturn, HookContinue;
//
// 	HookContinue = FALSE;
//
// 	if (DeviceExtension->MouseHook.IsrRoutine)
// 	{
// 		HookReturn = DeviceExtension->MouseHook.IsrRoutine(
// 			DeviceExtension->MouseHook.Context,
// 			DeviceExtension->MouseBuffer + DeviceExtension->MouseInBuffer,
// 			&DeviceExtension->Common.PortDeviceExtension->Packet,
// 			Status,
// 			&Input,
// 			&HookContinue,
// 			&DeviceExtension->MouseState,
// 			&DeviceExtension->MouseResetState);
//
// 		if (!HookContinue)
// 		{
// 			*ToReturn = HookReturn;
// 			return TRUE;
// 		}
// 	}
// 	return FALSE;
// }
//
// static BOOLEAN
// i8042MouResetIsr(
// 	IN PI8042_MOUSE_EXTENSION DeviceExtension,
// 	IN UCHAR Status,
// 	IN UCHAR Value)
// {
// 	PPORT_DEVICE_EXTENSION PortDeviceExtension;
// 	BOOLEAN ToReturn = FALSE;
//
// 	if (i8042MouCallIsrHook(DeviceExtension, Status, Value, &ToReturn))
// 		return ToReturn;
//
// 	if (MouseIdle == DeviceExtension->MouseState)
// 	{
// 		/* Magic packet value that indicates a reset */
// 		if (0xAA == Value)
// 		{
// 			WARN_(I8042PRT, "Hot plugged mouse!\n");
// 			DeviceExtension->MouseState = MouseResetting;
// 			DeviceExtension->MouseResetState = ExpectingReset;
// 		}
// 		else
// 			return FALSE;
// 	}
// 	else if (MouseResetting != DeviceExtension->MouseState)
// 		return FALSE;
//
// 	DeviceExtension->MouseTimeoutState = TimeoutStart;
// 	PortDeviceExtension = DeviceExtension->Common.PortDeviceExtension;
//
// 	switch ((ULONG)DeviceExtension->MouseResetState)
// 	{
// 		case ExpectingReset:
// 			if (MOUSE_ACK == Value)
// 			{
// 				WARN_(I8042PRT, "Dropping extra ACK\n");
// 				return TRUE;
// 			}
//
// 			/* First, 0xFF is sent. The mouse is supposed to say AA00 if ok, FC00 if not. */
// 			if (0xAA == Value)
// 			{
// 				DeviceExtension->MouseResetState++;
// 			}
// 			else
// 			{
// 				PortDeviceExtension->Flags &= ~MOUSE_PRESENT;
// 				DeviceExtension->MouseState = MouseIdle;
// 				WARN_(I8042PRT, "Mouse returned bad reset reply: %x (expected aa)\n", Value);
// 			}
// 			return TRUE;
// 		case ExpectingResetId:
// 			if (MOUSE_ACK == Value)
// 			{
// 				WARN_(I8042PRT, "Dropping extra ACK #2\n");
// 				return TRUE;
// 			}
//
// 			if (0x00 == Value)
// 			{
// 				DeviceExtension->MouseResetState++;
// 				DeviceExtension->MouseType = GenericPS2;
// 				DeviceExtension->MouseHook.IsrWritePort(DeviceExtension->MouseHook.CallContext, 0xF2);
// 			}
// 			else
// 			{
// 				PortDeviceExtension->Flags &= ~MOUSE_PRESENT;
// 				DeviceExtension->MouseState = MouseIdle;
// 				WARN_(I8042PRT, "Mouse returned bad reset reply part two: %x (expected 0)\n", Value);
// 			}
// 			return TRUE;
// 		case ExpectingGetDeviceIdACK:
// 			if (MOUSE_ACK == Value)
// 			{
// 				DeviceExtension->MouseResetState++;
// 			}
// 			else if (MOUSE_NACK == Value || MOUSE_ERROR == Value)
// 			{
// 				DeviceExtension->MouseResetState++;
// 				/* Act as if 00 (normal mouse) was received */
// 				WARN_(I8042PRT, "Mouse doesn't support 0xd2, (returns %x, expected %x), faking\n", Value, MOUSE_ACK);
// 				i8042MouResetIsr(DeviceExtension, Status, 0);
// 			}
// 			return TRUE;
// 		case ExpectingGetDeviceIdValue:
// 			switch (Value)
// 			{
// 				case 0x02:
// 					DeviceExtension->MouseAttributes.MouseIdentifier =
// 						BALLPOINT_I8042_HARDWARE;
// 					break;
// 				case 0x03:
// 				case 0x04:
// 					DeviceExtension->MouseAttributes.MouseIdentifier =
// 						WHEELMOUSE_I8042_HARDWARE;
// 					break;
// 				default:
// 					DeviceExtension->MouseAttributes.MouseIdentifier =
// 						MOUSE_I8042_HARDWARE;
// 			}
// 			DeviceExtension->MouseResetState++;
// 			DeviceExtension->MouseHook.IsrWritePort(DeviceExtension->MouseHook.CallContext, 0xE8);
// 			return TRUE;
// 		case ExpectingSetResolutionDefaultACK:
// 			DeviceExtension->MouseResetState++;
// 			DeviceExtension->MouseHook.IsrWritePort(DeviceExtension->MouseHook.CallContext, 0x00);
// 			return TRUE;
// 		case ExpectingSetResolutionDefaultValueACK:
// 			DeviceExtension->MouseResetState = ExpectingSetScaling1to1ACK;
// 			DeviceExtension->MouseHook.IsrWritePort(DeviceExtension->MouseHook.CallContext, 0xE6);
// 			return TRUE;
// 		case ExpectingSetScaling1to1ACK:
// 		case ExpectingSetScaling1to1ACK2:
// 			DeviceExtension->MouseResetState++;
// 			DeviceExtension->MouseHook.IsrWritePort(DeviceExtension->MouseHook.CallContext, 0xE6);
// 			return TRUE;
// 		case ExpectingSetScaling1to1ACK3:
// 			DeviceExtension->MouseResetState++;
// 			DeviceExtension->MouseHook.IsrWritePort(DeviceExtension->MouseHook.CallContext, 0xE9);
// 			return TRUE;
// 		case ExpectingReadMouseStatusACK:
// 			DeviceExtension->MouseResetState++;
// 			return TRUE;
// 		case ExpectingReadMouseStatusByte1:
// 			DeviceExtension->MouseLogiBuffer[0] = Value;
// 			DeviceExtension->MouseResetState++;
// 			return TRUE;
// 		case ExpectingReadMouseStatusByte2:
// 			DeviceExtension->MouseLogiBuffer[1] = Value;
// 			DeviceExtension->MouseResetState++;
// 			return TRUE;
// 		case ExpectingReadMouseStatusByte3:
// 			DeviceExtension->MouseLogiBuffer[2] = Value;
// 			/* Now MouseLogiBuffer is a set of info. If the second
// 			 * byte is 0, the mouse didn't understand the magic
// 			 * code. Otherwise, it it a Logitech and the second byte
// 			 * is the number of buttons, bit 7 of the first byte tells
// 			 * if it understands special E7 commands, the rest is an ID.
// 			 */
// 			if (DeviceExtension->MouseLogiBuffer[1])
// 			{
// 				DeviceExtension->MouseAttributes.NumberOfButtons =
// 					DeviceExtension->MouseLogiBuffer[1];
// 				DeviceExtension->MouseType = Ps2pp;
// 				DeviceExtension->MouseHook.IsrWritePort(DeviceExtension->MouseHook.CallContext, 0xF3);
// 				DeviceExtension->MouseResetState = ExpectingSetSamplingRateACK;
// 				/* TODO: Go through EnableWheel and Enable5Buttons */
// 				return TRUE;
// 			}
// 			DeviceExtension->MouseResetState = EnableWheel;
// 			i8042MouResetIsr(DeviceExtension, Status, Value);
// 			return TRUE;
// 		case EnableWheel:
// 			DeviceExtension->MouseHook.IsrWritePort(DeviceExtension->MouseHook.CallContext, 0xF3);
// 			DeviceExtension->MouseResetState = 1001;
// 			return TRUE;
// 		case 1001:
// 			DeviceExtension->MouseHook.IsrWritePort(DeviceExtension->MouseHook.CallContext, 0xC8);
// 			DeviceExtension->MouseResetState++;
// 			return TRUE;
// 		case 1002:
// 		case 1004:
// 			DeviceExtension->MouseHook.IsrWritePort(DeviceExtension->MouseHook.CallContext, 0xF3);
// 			DeviceExtension->MouseResetState++;
// 			return TRUE;
// 		case 1003:
// 			DeviceExtension->MouseHook.IsrWritePort(DeviceExtension->MouseHook.CallContext, 0x64);
// 			DeviceExtension->MouseResetState++;
// 			return TRUE;
// 		case 1005:
// 			DeviceExtension->MouseHook.IsrWritePort(DeviceExtension->MouseHook.CallContext, 0x50);
// 			DeviceExtension->MouseResetState++;
// 			return TRUE;
// 		case 1006:
// 			DeviceExtension->MouseHook.IsrWritePort(DeviceExtension->MouseHook.CallContext, 0xF2);
// 			DeviceExtension->MouseResetState++;
// 			return TRUE;
// 		case 1007:
// 			/* Ignore ACK */
// 			DeviceExtension->MouseResetState++;
// 			return TRUE;
// 		case 1008:
// 			if (0x03 == Value) {
// 				/* It's either an Intellimouse or Intellimouse Explorer. */
// 				DeviceExtension->MouseAttributes.NumberOfButtons = 3;
// 				DeviceExtension->MouseAttributes.MouseIdentifier =
// 					WHEELMOUSE_I8042_HARDWARE;
// 				DeviceExtension->MouseType = Intellimouse;
// 				DeviceExtension->MouseResetState = Enable5Buttons;
// 				i8042MouResetIsr(DeviceExtension, Status, Value);
// 			}
// 			else
// 			{
// 				/* Just set the default settings and be done */
// 				DeviceExtension->MouseHook.IsrWritePort(DeviceExtension->MouseHook.CallContext, 0xF3);
// 				DeviceExtension->MouseResetState = ExpectingSetSamplingRateACK;
// 			}
// 			return TRUE;
// 		case Enable5Buttons:
// 			DeviceExtension->MouseHook.IsrWritePort(DeviceExtension->MouseHook.CallContext, 0xF3);
// 			DeviceExtension->MouseResetState = 1021;
// 			return TRUE;
// 		case 1022:
// 		case 1024:
// 			DeviceExtension->MouseHook.IsrWritePort(DeviceExtension->MouseHook.CallContext, 0xF3);
// 			DeviceExtension->MouseResetState++;
// 			return TRUE;
// 		case 1021:
// 		case 1023:
// 			DeviceExtension->MouseHook.IsrWritePort(DeviceExtension->MouseHook.CallContext, 0xC8);
// 			DeviceExtension->MouseResetState++;
// 			return TRUE;
// 		case 1025:
// 			DeviceExtension->MouseHook.IsrWritePort(DeviceExtension->MouseHook.CallContext, 0x50);
// 			DeviceExtension->MouseResetState++;
// 			return TRUE;
// 		case 1026:
// 			DeviceExtension->MouseHook.IsrWritePort(DeviceExtension->MouseHook.CallContext, 0xF2);
// 			DeviceExtension->MouseResetState++;
// 			return TRUE;
// 		case 1027:
// 			if (0x04 == Value)
// 			{
// 				DeviceExtension->MouseAttributes.NumberOfButtons = 5;
// 				DeviceExtension->MouseAttributes.MouseIdentifier =
// 					WHEELMOUSE_I8042_HARDWARE;
// 				DeviceExtension->MouseType = IntellimouseExplorer;
// 			}
// 			DeviceExtension->MouseHook.IsrWritePort(DeviceExtension->MouseHook.CallContext, 0xF3);
// 			DeviceExtension->MouseResetState = ExpectingSetSamplingRateACK;
// 			return TRUE;
// 		case ExpectingSetSamplingRateACK:
// 			DeviceExtension->MouseHook.IsrWritePort(
// 				DeviceExtension->MouseHook.CallContext,
// 				(UCHAR)DeviceExtension->MouseAttributes.SampleRate);
// 			DeviceExtension->MouseResetState++;
// 			return TRUE;
// 		case ExpectingSetSamplingRateValueACK:
// 			if (MOUSE_NACK == Value)
// 			{
// 				DeviceExtension->MouseHook.IsrWritePort(DeviceExtension->MouseHook.CallContext, 0x3C);
// 				DeviceExtension->MouseAttributes.SampleRate = (USHORT)PortDeviceExtension->Settings.SampleRate;
// 				DeviceExtension->MouseResetState = 1040;
// 				return TRUE;
// 			}
// 		case 1040:  /* Fallthrough */
// 			DeviceExtension->MouseHook.IsrWritePort(DeviceExtension->MouseHook.CallContext, 0xE8);
// 			DeviceExtension->MouseResetState = ExpectingFinalResolutionACK;
// 			return TRUE;
// 		case ExpectingFinalResolutionACK:
// 			DeviceExtension->MouseHook.IsrWritePort(
// 				DeviceExtension->MouseHook.CallContext,
// 				(UCHAR)(PortDeviceExtension->Settings.MouseResolution & 0xff));
// 			INFO_(I8042PRT, "Mouse resolution %lu\n",
// 				PortDeviceExtension->Settings.MouseResolution);
// 			DeviceExtension->MouseResetState = ExpectingFinalResolutionValueACK;
// 			return TRUE;
// 		case ExpectingFinalResolutionValueACK:
// 			DeviceExtension->MouseHook.IsrWritePort(DeviceExtension->MouseHook.CallContext, 0xF4);
// 			DeviceExtension->MouseResetState = ExpectingEnableACK;
// 			return TRUE;
// 		case ExpectingEnableACK:
// 			PortDeviceExtension->Flags |= MOUSE_PRESENT;
// 			DeviceExtension->MouseState = MouseIdle;
// 			DeviceExtension->MouseTimeoutState = TimeoutCancel;
// 			INFO_(I8042PRT, "Mouse type = %u\n", DeviceExtension->MouseType);
// 			return TRUE;
// 		default:
// 			if (DeviceExtension->MouseResetState < 100 || DeviceExtension->MouseResetState > 999)
// 				ERR_(I8042PRT, "MouseResetState went out of range: %lu\n", DeviceExtension->MouseResetState);
// 			return FALSE;
// 	}
// }
//
// BOOLEAN NTAPI
// i8042MouInterruptService(
// 	IN PKINTERRUPT Interrupt,
// 	PVOID Context)
// {
// 	PI8042_MOUSE_EXTENSION DeviceExtension;
// 	PPORT_DEVICE_EXTENSION PortDeviceExtension;
// 	ULONG Counter;
// 	UCHAR Output = 0, PortStatus = 0;
// 	NTSTATUS Status;
//
// 	UNREFERENCED_PARAMETER(Interrupt);
//
// 	__analysis_assume(Context != NULL);
// 	DeviceExtension = Context;
// 	{
// 		static BOOLEAN _i8k_mou_isr_seen = FALSE;
// 		if (!_i8k_mou_isr_seen)
// 		{
// 			_i8k_mou_isr_seen = TRUE;
// 			DPRINT1("[[I8K]] MOU ISR FIRED (first)\n");
// 		}
// 	}
// 	PortDeviceExtension = DeviceExtension->Common.PortDeviceExtension;
// 	Counter = PortDeviceExtension->Settings.PollStatusIterations;
//
// 	while (Counter)
// 	{
// 		Status = i8042ReadStatus(PortDeviceExtension, &PortStatus);
// 		if (!NT_SUCCESS(Status))
// 		{
// 			WARN_(I8042PRT, "i8042ReadStatus() failed with status 0x%08lx\n", Status);
// 			return FALSE;
// 		}
// 		Status = i8042ReadMouseData(PortDeviceExtension, &Output);
// 		if (NT_SUCCESS(Status))
// 			break;
// 		KeStallExecutionProcessor(1);
// 		Counter--;
// 	}
// 	if (Counter == 0)
// 	{
// 		WARN_(I8042PRT, "Spurious i8042 mouse interrupt\n");
// 		return FALSE;
// 	}
//
// 	INFO_(I8042PRT, "Got: 0x%02x\n", Output);
//
// 	if (i8042PacketIsr(PortDeviceExtension, Output))
// 	{
// 		if (PortDeviceExtension->PacketComplete)
// 		{
// 			TRACE_(I8042PRT, "Packet complete\n");
// 			KeInsertQueueDpc(&DeviceExtension->DpcMouse, NULL, NULL);
// 		}
// 		TRACE_(I8042PRT, "Irq eaten by packet\n");
// 		return TRUE;
// 	}
//
// 	TRACE_(I8042PRT, "Irq is mouse input\n");
//
// 	i8042MouInputTestTimeout(DeviceExtension);
//
// 	if (i8042MouResetIsr(DeviceExtension, PortStatus, Output))
// 	{
// 		TRACE_(I8042PRT, "Handled by ResetIsr or hooked Isr\n");
// 		if (NoChange != DeviceExtension->MouseTimeoutState) {
// 			KeInsertQueueDpc(&DeviceExtension->DpcMouse, NULL, NULL);
// 		}
// 		return TRUE;
// 	}
//
// 	if (DeviceExtension->MouseType == Ps2pp)
// 		i8042MouHandlePs2pp(DeviceExtension, Output);
// 	else
// 		i8042MouHandle(DeviceExtension, Output);
//
// 	return TRUE;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
