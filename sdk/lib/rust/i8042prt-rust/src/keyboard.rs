use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::driver_ddk::*;
use ntkrnl_sys::ex::*;
use ntkrnl_sys::interlocked::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ddk::{_WORK_QUEUE_TYPE_DelayedWorkQueue as DelayedWorkQueue, IoAllocateWorkItem, IoFreeWorkItem, IoQueueWorkItem, IoRegisterDeviceInterface, IoSetDeviceInterfaceState, IoStartNextPacket, IoStartPacket, KEYBOARD_ATTRIBUTES, KEYBOARD_INDICATOR_PARAMETERS, KEYBOARD_INPUT_DATA, KeAcquireInterruptSpinLock, KeBugCheck, KeInitializeDpc, KeInsertQueueDpc, KeReleaseInterruptSpinLock, KeStallExecutionProcessor, PDEVICE_OBJECT, PIO_STACK_LOCATION, PIO_WORKITEM, PIRP, PKEYBOARD_ATTRIBUTES, PKEYBOARD_INPUT_DATA, PKINTERRUPT};
// FORCEINLINE accessors (no exported symbol — use the Rust inline impls).
use ntkrnl_sys::inline::tier1::{IoGetCurrentIrpStackLocation, IoMarkIrpPending};
use core::ffi::c_void;
use crate::i8042prt_h::*;
use crate::readwrite::{i8042ReadStatus, i8042ReadKeyboardData};
use crate::misc::ForwardIrpAndForget;
use crate::i8042prt::{i8042StartPacket, i8042PacketIsr};
// SYSDBG_COMMAND is bindgen `_SYSDBG_COMMAND`; KPROCESSOR_MODE is a ddk alias.
use ntkrnl_sys::ddk::{_SYSDBG_COMMAND as SYSDBG_COMMAND, KPROCESSOR_MODE};

// Crash-on-Ctrl-Scroll debug path: these are not in ntkrnl-sys. NTAPI/cdecl per
// ntoskrnl exports (KdSystemDebugControl = NTAPI; DbgBreakPointWithStatus =
// cdecl in ntddk).
extern "system" {
    fn KdSystemDebugControl(
        Command: SYSDBG_COMMAND,
        InputBuffer: PVOID,
        InputBufferLength: ULONG,
        OutputBuffer: PVOID,
        OutputBufferLength: ULONG,
        ReturnLength: *mut ULONG,
        PreviousMode: KPROCESSOR_MODE,
    ) -> NTSTATUS;
}
// DbgBreakPointWithStatus is `@ stdcall` in ntoskrnl.spec → extern "system"
// (i386 decoration _DbgBreakPointWithStatus@4), NOT cdecl.
extern "system" {
    fn DbgBreakPointWithStatus(Status: ULONG);
}
// InterlockedExchange is the _InterlockedExchange intrinsic (no exported
// symbol) — provide it inline as an atomic swap returning the OLD value.
#[inline]
unsafe fn InterlockedExchange(Target: *mut LONG, Value: LONG) -> LONG {
    use core::sync::atomic::{AtomicI32, Ordering};
    (*(Target as *const AtomicI32)).swap(Value, Ordering::SeqCst)
}
// InterlockedExchangePointer (PVOID exchange) — from the ntkrnl-sys intrinsics.
use ntkrnl_sys::interlocked::InterlockedExchangePointer;

extern "system" {
    fn i8042SynchWritePort( DeviceExtension: PPORT_DEVICE_EXTENSION, Port: UCHAR, Value: UCHAR, WaitForAck: BOOLEAN, ) -> NTSTATUS;
}

// ---- file-scope preamble (translated) ----
#[repr(C)]
pub struct _LOCAL_KEYBOARD_INDICATOR_TRANSLATION {
    pub NumberOfIndicatorKeys: USHORT,
    pub IndicatorList: [INDICATOR_LIST; 3],
}
pub type LOCAL_KEYBOARD_INDICATOR_TRANSLATION = _LOCAL_KEYBOARD_INDICATOR_TRANSLATION;
pub type PLOCAL_KEYBOARD_INDICATOR_TRANSLATION = *mut _LOCAL_KEYBOARD_INDICATOR_TRANSLATION;

pub static IndicatorTranslation: LOCAL_KEYBOARD_INDICATOR_TRANSLATION = LOCAL_KEYBOARD_INDICATOR_TRANSLATION {
    NumberOfIndicatorKeys: 3,
    IndicatorList: [
        INDICATOR_LIST { MakeCode: 0x3A, IndicatorFlags: KEYBOARD_CAPS_LOCK_ON as u16 },
        INDICATOR_LIST { MakeCode: 0x45, IndicatorFlags: KEYBOARD_NUM_LOCK_ON as u16 },
        INDICATOR_LIST { MakeCode: 0x46, IndicatorFlags: KEYBOARD_SCROLL_LOCK_ON as u16 },
    ],
};

// i8042SendHookWorkItem is the IO_WORKITEM_ROUTINE defined in i8042prt.c (the
// full IRP round-trip + hook-init lives in i8042prt.rs). keyboard.c references
// it as the IoQueueWorkItem callback; declare it here so the call site below
// resolves to the single canonical `#[no_mangle]` symbol (no duplicate def).
extern "system" {
    fn i8042SendHookWorkItem(DeviceObject: PDEVICE_OBJECT, Context: PVOID);
}

unsafe extern "system" fn i8042KbdQueuePacket(Context: PVOID) {
    let DeviceExtension = Context as PI8042_KEYBOARD_EXTENSION;

    (*DeviceExtension).KeyComplete = 1u8;
    (*DeviceExtension).KeysInBuffer += 1;
    if (*DeviceExtension).KeysInBuffer
        > (*(*DeviceExtension).Common.PortDeviceExtension).Settings.KeyboardDataQueueSize
    {
        (*DeviceExtension).KeysInBuffer -= 1;
    }

    KeInsertQueueDpc(
        core::ptr::addr_of_mut!((*DeviceExtension).DpcKeyboard),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );
}

#[no_mangle]
pub unsafe extern "system" fn i8042SynchWritePortKbd(
    Context: PVOID,
    Value: UCHAR,
    WaitForAck: BOOLEAN,
) -> NTSTATUS {
    i8042SynchWritePort(
        Context as PPORT_DEVICE_EXTENSION,
        0,
        Value,
        WaitForAck,
    )
}

#[no_mangle]
pub unsafe extern "system" fn i8042KbdStartIo(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) {
    let Stack: PIO_STACK_LOCATION = IoGetCurrentIrpStackLocation(Irp);
    let DeviceExtension =
        (*DeviceObject).DeviceExtension as PI8042_KEYBOARD_EXTENSION;
    let PortDeviceExtension: PPORT_DEVICE_EXTENSION =
        (*DeviceExtension).Common.PortDeviceExtension;

    match (*Stack).Parameters.DeviceIoControl.IoControlCode {
        IOCTL_KEYBOARD_SET_INDICATORS => {
            (*PortDeviceExtension).PacketBuffer[0] = KBD_CMD_SET_LEDS as UCHAR;
            (*PortDeviceExtension).PacketBuffer[1] = 0;
            if (*DeviceExtension).KeyboardIndicators.LedFlags as u32
                & KEYBOARD_CAPS_LOCK_ON
                != 0
            {
                (*PortDeviceExtension).PacketBuffer[1] |= KBD_LED_CAPS as UCHAR;
            }

            if (*DeviceExtension).KeyboardIndicators.LedFlags as u32
                & KEYBOARD_NUM_LOCK_ON
                != 0
            {
                (*PortDeviceExtension).PacketBuffer[1] |= KBD_LED_NUM as UCHAR;
            }

            if (*DeviceExtension).KeyboardIndicators.LedFlags as u32
                & KEYBOARD_SCROLL_LOCK_ON
                != 0
            {
                (*PortDeviceExtension).PacketBuffer[1] |= KBD_LED_SCROLL as UCHAR;
            }

            i8042StartPacket(
                PortDeviceExtension,
                core::ptr::addr_of_mut!((*DeviceExtension).Common),
                (*PortDeviceExtension).PacketBuffer.as_mut_ptr(),
                2,
                Irp,
            );
        }
        _ => {}
    }
}

pub(crate) unsafe fn i8042PacketDpc(DeviceExtension: PPORT_DEVICE_EXTENSION) {
    let mut FinishIrp: BOOLEAN = 0u8;
    let Irql: KIRQL;
    let mut Result: NTSTATUS = STATUS_INTERNAL_ERROR;

    if (*DeviceExtension).HighestDIRQLInterrupt.is_null() {
        return;
    }

    Irql = KeAcquireInterruptSpinLock((*DeviceExtension).HighestDIRQLInterrupt);

    if (*DeviceExtension).Packet.State == Idle
        && (*DeviceExtension).PacketComplete != 0
    {
        FinishIrp = 1u8;
        Result = (*DeviceExtension).PacketResult;
        (*DeviceExtension).PacketComplete = 0u8;
    }

    KeReleaseInterruptSpinLock((*DeviceExtension).HighestDIRQLInterrupt, Irql);

    if FinishIrp == 0 {
        return;
    }

    if !(*DeviceExtension).CurrentIrp.is_null() {
        (*(*DeviceExtension).CurrentIrp).IoStatus.__bindgen_anon_1.Status = Result;
        IoCompleteRequest((*DeviceExtension).CurrentIrp, IO_NO_INCREMENT as i8);
        IoStartNextPacket((*DeviceExtension).CurrentIrpDevice, 0u8);
        (*DeviceExtension).CurrentIrp = core::ptr::null_mut();
        (*DeviceExtension).CurrentIrpDevice = core::ptr::null_mut();
    }
}

unsafe extern "system" fn i8042PowerWorkItem(
    DeviceObject: PDEVICE_OBJECT,
    Context: *mut core::ffi::c_void,
) {
    let _ = DeviceObject;

    let DeviceExtension = Context as PI8042_KEYBOARD_EXTENSION;

    if (*DeviceExtension).NewCaps != (*DeviceExtension).ReportedCaps {
        let WaitingIrp = InterlockedExchangePointer(
            core::ptr::addr_of_mut!((*DeviceExtension).PowerIrp).cast::<*mut core::ffi::c_void>(),
            core::ptr::null_mut(),
        ) as PIRP;
        if !WaitingIrp.is_null() {
            (*WaitingIrp).IoStatus.__bindgen_anon_1.Status = STATUS_UNSUCCESSFUL;
            (*WaitingIrp).IoStatus.Information = core::mem::size_of::<ULONG>() as _;
            IoCompleteRequest(WaitingIrp, IO_NO_INCREMENT as i8);
        }

        if (*DeviceExtension).PowerInterfaceName.MaximumLength == 0 {
            let Status = IoRegisterDeviceInterface(
                (*DeviceExtension).Common.Pdo,
                core::ptr::addr_of!(GUID_DEVICE_SYS_BUTTON),
                core::ptr::null_mut(),
                core::ptr::addr_of_mut!((*DeviceExtension).PowerInterfaceName),
            );
            if !NT_SUCCESS(Status) {
                (*DeviceExtension).PowerInterfaceName.MaximumLength = 0;
                return;
            }
        } else {
            let Status = IoSetDeviceInterfaceState(
                core::ptr::addr_of_mut!((*DeviceExtension).PowerInterfaceName),
                0u8,
            );
            if !NT_SUCCESS(Status) {
                return;
            }
        }

        let Status = IoSetDeviceInterfaceState(
            core::ptr::addr_of_mut!((*DeviceExtension).PowerInterfaceName),
            1u8,
        );
        if !NT_SUCCESS(Status) {
            return;
        }
    }

    let WaitingIrp = InterlockedExchangePointer(
        core::ptr::addr_of_mut!((*DeviceExtension).PowerIrp).cast::<*mut core::ffi::c_void>(),
        core::ptr::null_mut(),
    ) as PIRP;
    if !WaitingIrp.is_null() {
        let pEvent = (*WaitingIrp).AssociatedIrp.SystemBuffer as PULONG;

        (*WaitingIrp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
        (*WaitingIrp).IoStatus.Information = core::mem::size_of::<ULONG>() as _;
        *pEvent = InterlockedExchange(
            core::ptr::addr_of_mut!((*DeviceExtension).LastPowerKey) as PLONG,
            0,
        ) as ULONG;
        IoCompleteRequest(WaitingIrp, IO_NO_INCREMENT as i8);
    }
}

unsafe fn HandlePowerKeys(DeviceExtension: PI8042_KEYBOARD_EXTENSION) -> BOOLEAN {
    let InputData: PKEYBOARD_INPUT_DATA = (*DeviceExtension).KeyboardBuffer
        .add((*DeviceExtension).KeysInBuffer as usize)
        .offset(-1);

    if ((*InputData).Flags & KEY_E0 as USHORT) == 0 {
        return 0u8;
    }

    let KeyPress: ULONG;
    match (*InputData).MakeCode as u32 {
        KEYBOARD_POWER_CODE => KeyPress = SYS_BUTTON_POWER,
        KEYBOARD_SLEEP_CODE => KeyPress = SYS_BUTTON_SLEEP,
        KEYBOARD_WAKE_CODE => KeyPress = SYS_BUTTON_WAKE,
        _ => return 0u8,
    }

    if ((*InputData).Flags & KEY_BREAK as USHORT) != 0 {
        return 1u8;
    }

    (*DeviceExtension).NewCaps |= KeyPress;

    let target = core::ptr::addr_of_mut!((*DeviceExtension).LastPowerKey) as *mut core::ffi::c_long;
    core::ptr::write_volatile(target, KeyPress as core::ffi::c_long);

    IoQueueWorkItem(
        (*DeviceExtension).PowerWorkItem,
        core::mem::transmute::<Option<unsafe extern "system" fn(*mut ntkrnl_sys::ddk::_DEVICE_OBJECT, *mut c_void)>, ntkrnl_sys::ddk::PIO_WORKITEM_ROUTINE>(Some(i8042PowerWorkItem as unsafe extern "system" fn(*mut ntkrnl_sys::ddk::_DEVICE_OBJECT, *mut c_void))),
        DelayedWorkQueue,
        DeviceExtension as *mut core::ffi::c_void,
    );

    1u8
}

unsafe extern "system" fn i8042KbdDpcRoutine(
    _Dpc: PKDPC,
    DeferredContext: PVOID,
    _SystemArgument1: PVOID,
    _SystemArgument2: PVOID,
) {
    let DeviceExtension: PI8042_KEYBOARD_EXTENSION;
    let PortDeviceExtension: PPORT_DEVICE_EXTENSION;
    let mut KeysTransferred: ULONG = 0;
    let KeysInBufferCopy: ULONG;
    let mut Irql: KIRQL;

    DeviceExtension = DeferredContext as PI8042_KEYBOARD_EXTENSION;
    PortDeviceExtension = (*DeviceExtension).Common.PortDeviceExtension;

    if HandlePowerKeys(DeviceExtension) != 0 {
        (*DeviceExtension).KeyComplete = 0u8;
        return;
    }

    i8042PacketDpc(PortDeviceExtension);
    if (*DeviceExtension).KeyComplete == 0 {
        return;
    }
    if (*PortDeviceExtension).HighestDIRQLInterrupt.is_null() {
        return;
    }

    Irql = KeAcquireInterruptSpinLock((*PortDeviceExtension).HighestDIRQLInterrupt);

    (*DeviceExtension).KeyComplete = 0u8;
    KeysInBufferCopy = (*DeviceExtension).KeysInBuffer;

    KeReleaseInterruptSpinLock((*PortDeviceExtension).HighestDIRQLInterrupt, Irql);

    if (*DeviceExtension).KeyboardData.ClassService.is_null() {
        return;
    }

    // ClassService is a PVOID holding a PSERVICE_CALLBACK_ROUTINE, which is
    // NTAPI — stdcall on i386, so this MUST be extern "system", not extern "C".
    // A cdecl transmute here double-cleans 16 bytes of stack (callee ret 16 +
    // caller add esp,16), skewing ESP so the DPC epilogue rets to 0 → BUGCHECK
    // 0x1E on the first keystroke/packet (field manual §9.38). mouse.rs::
    // i8042MouDpcRoutine is the correct sibling.
    let ClassService: unsafe extern "system" fn(PVOID, PVOID, PVOID, PULONG) =
        core::mem::transmute((*DeviceExtension).KeyboardData.ClassService);
    ClassService(
        (*DeviceExtension).KeyboardData.ClassDeviceObject as PVOID,
        (*DeviceExtension).KeyboardBuffer as PVOID,
        (*DeviceExtension).KeyboardBuffer.add(KeysInBufferCopy as usize) as PVOID,
        &mut KeysTransferred,
    );

    // ClassService runs outside the interrupt lock, so the ISR can append keys
    // meanwhile. Move the untransferred tail down to the start of the buffer, as
    // the mouse DPC does. Upstream's keyboard DPC only decremented the count: the
    // next DPC re-delivered KeyboardBuffer[0] (a key already sent) and dropped the
    // new one. The race showed as one Esc make delivered twice with its break lost,
    // so Ctrl+Esc toggled the start menu open and shut.
    Irql = KeAcquireInterruptSpinLock((*PortDeviceExtension).HighestDIRQLInterrupt);
    (*DeviceExtension).KeysInBuffer -= KeysTransferred;
    if (*DeviceExtension).KeysInBuffer != 0 && KeysTransferred != 0 {
        core::ptr::copy(
            (*DeviceExtension).KeyboardBuffer.add(KeysTransferred as usize),
            (*DeviceExtension).KeyboardBuffer,
            (*DeviceExtension).KeysInBuffer as usize,
        );
    }
    KeReleaseInterruptSpinLock((*PortDeviceExtension).HighestDIRQLInterrupt, Irql);
}

#[no_mangle]
pub unsafe extern "system" fn i8042KbdDeviceControl(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    let Stack: PIO_STACK_LOCATION = IoGetCurrentIrpStackLocation(Irp);
    (*Irp).IoStatus.Information = 0;
    let DeviceExtension = (*DeviceObject).DeviceExtension as PI8042_KEYBOARD_EXTENSION;

    let Status: NTSTATUS;

    match (*Stack).Parameters.DeviceIoControl.IoControlCode {
        IOCTL_GET_SYS_BUTTON_CAPS => {
            if (*Stack).Parameters.DeviceIoControl.OutputBufferLength as usize
                != core::mem::size_of::<ULONG>()
            {
                Status = STATUS_INVALID_PARAMETER;
            } else {
                let pCaps = (*Irp).AssociatedIrp.SystemBuffer as PULONG;
                *pCaps = (*DeviceExtension).NewCaps;
                (*DeviceExtension).ReportedCaps = (*DeviceExtension).NewCaps;
                (*Irp).IoStatus.Information = core::mem::size_of::<ULONG>() as _;
                Status = STATUS_SUCCESS;
            }
        }
        IOCTL_GET_SYS_BUTTON_EVENT => {
            if (*Stack).Parameters.DeviceIoControl.OutputBufferLength as usize
                != core::mem::size_of::<ULONG>()
            {
                Status = STATUS_INVALID_PARAMETER;
            } else {
                let WaitingIrp = InterlockedCompareExchangePointer(
                    core::ptr::addr_of_mut!((*DeviceExtension).PowerIrp) as *mut PVOID,
                    Irp as PVOID,
                    core::ptr::null_mut(),
                );
                if !WaitingIrp.is_null() {
                    let Status = STATUS_INVALID_PARAMETER;
                    (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
                    IoCompleteRequest(Irp, IO_NO_INCREMENT as i8);
                    return Status;
                } else {
                    let PowerKey = {
                        let ptr = core::ptr::addr_of_mut!((*DeviceExtension).LastPowerKey) as *mut LONG;
                        let old = *ptr;
                        *ptr = 0;
                        old as ULONG
                    };
                    if PowerKey != 0 {
                        InterlockedCompareExchangePointer(
                            core::ptr::addr_of_mut!((*DeviceExtension).PowerIrp) as *mut PVOID,
                            core::ptr::null_mut(),
                            Irp as PVOID,
                        );
                        *((*Irp).AssociatedIrp.SystemBuffer as PULONG) = PowerKey;
                        let Status = STATUS_SUCCESS;
                        (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
                        (*Irp).IoStatus.Information = core::mem::size_of::<ULONG>() as _;
                        IoCompleteRequest(Irp, IO_NO_INCREMENT as i8);
                        return Status;
                    } else {
                        let Status = STATUS_PENDING;
                        (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
                        IoMarkIrpPending(Irp);
                        return Status;
                    }
                }
            }
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

unsafe extern "system" fn i8042InitializeKeyboardAttributes(
    DeviceExtension: PI8042_KEYBOARD_EXTENSION,
) {
    let PortDeviceExtension: PPORT_DEVICE_EXTENSION;
    let Settings: PI8042_SETTINGS;
    let KeyboardAttributes: PKEYBOARD_ATTRIBUTES;

    PortDeviceExtension = (*DeviceExtension).Common.PortDeviceExtension;
    Settings = core::ptr::addr_of_mut!((*PortDeviceExtension).Settings);

    KeyboardAttributes = core::ptr::addr_of_mut!((*DeviceExtension).KeyboardAttributes);

    (*KeyboardAttributes).KeyboardIdentifier.Type = (*Settings).OverrideKeyboardType as UCHAR;
    (*KeyboardAttributes).KeyboardIdentifier.Subtype = (*Settings).OverrideKeyboardSubtype as UCHAR;
    (*KeyboardAttributes).NumberOfFunctionKeys = 12; // F1-F12, not 4 (upstream c4517f4b0a3)
    (*KeyboardAttributes).NumberOfIndicators = 3;
    (*KeyboardAttributes).NumberOfKeysTotal = 101;
    (*KeyboardAttributes).InputDataQueueLength = (*Settings).KeyboardDataQueueSize;
    (*KeyboardAttributes).KeyRepeatMinimum.UnitId = 0;
    (*KeyboardAttributes).KeyRepeatMinimum.Rate = (*Settings).SampleRate as USHORT;
    (*KeyboardAttributes).KeyRepeatMinimum.Delay = 0;
    (*KeyboardAttributes).KeyRepeatMinimum.UnitId = 0;
    (*KeyboardAttributes).KeyRepeatMinimum.Rate = (*Settings).SampleRate as USHORT;
    (*KeyboardAttributes).KeyRepeatMinimum.Delay = 0;
}

#[no_mangle]
pub unsafe extern "system" fn i8042KbdInternalDeviceControl(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    let Stack: PIO_STACK_LOCATION;
    let DeviceExtension: PI8042_KEYBOARD_EXTENSION;
    let Status: NTSTATUS;

    Stack = IoGetCurrentIrpStackLocation(Irp);
    (*Irp).IoStatus.Information = 0;
    DeviceExtension = (*DeviceObject).DeviceExtension as PI8042_KEYBOARD_EXTENSION;

    match (*Stack).Parameters.DeviceIoControl.IoControlCode {
        IOCTL_INTERNAL_KEYBOARD_CONNECT => {
            let mut WorkItem: PIO_WORKITEM = core::ptr::null_mut();
            let mut WorkItemData: PI8042_HOOK_WORKITEM = core::ptr::null_mut();
            // Track resources allocated by THIS invocation. The C cleanup frees
            // DeviceExtension->KeyboardBuffer/PowerWorkItem/DebugWorkItem
            // unconditionally, which double-frees a *live* buffer if a second
            // CONNECT arrives after a successful one (already KEYBOARD_CONNECTED).
            // Free only what this call created (and only on failure).
            let buffer_before = (*DeviceExtension).KeyboardBuffer;
            let power_wi_before = (*DeviceExtension).PowerWorkItem;
            let debug_wi_before = (*DeviceExtension).DebugWorkItem;

            'block: {
                if (*Stack).Parameters.DeviceIoControl.InputBufferLength as usize
                    != core::mem::size_of::<CONNECT_DATA>()
                {
                    Status = STATUS_INVALID_PARAMETER;
                    break 'block;
                }

                (*DeviceExtension).KeyboardData =
                    *((*Stack).Parameters.DeviceIoControl.Type3InputBuffer as PCONNECT_DATA);

                WorkItem = IoAllocateWorkItem(DeviceObject);
                if WorkItem.is_null() {
                    Status = STATUS_INSUFFICIENT_RESOURCES;
                    break 'block;
                }
                WorkItemData = ExAllocatePoolWithTag(
                    NonPagedPool,
                    core::mem::size_of::<I8042_HOOK_WORKITEM>(),
                    I8042PRT_TAG,
                ) as PI8042_HOOK_WORKITEM;
                if WorkItemData.is_null() {
                    Status = STATUS_NO_MEMORY;
                    break 'block;
                }
                (*WorkItemData).WorkItem = WorkItem;
                (*WorkItemData).Irp = Irp;

                (*DeviceExtension).Common.Type = Keyboard;
                let Size: SIZE_T = (*(*DeviceExtension).Common.PortDeviceExtension)
                    .Settings
                    .KeyboardDataQueueSize as SIZE_T
                    * core::mem::size_of::<KEYBOARD_INPUT_DATA>() as SIZE_T;
                (*DeviceExtension).KeyboardBuffer =
                    ExAllocatePoolWithTag(NonPagedPool, Size as usize, I8042PRT_TAG) as _;
                if (*DeviceExtension).KeyboardBuffer.is_null() {
                    Status = STATUS_NO_MEMORY;
                    break 'block;
                }
                RtlZeroMemory((*DeviceExtension).KeyboardBuffer as _, Size as usize);
                KeInitializeDpc(
                    core::ptr::addr_of_mut!((*DeviceExtension).DpcKeyboard),
                    core::mem::transmute::<Option<unsafe extern "system" fn(*mut ntkrnl_sys::ke::KDPC, *mut c_void, *mut c_void, *mut c_void)>, ntkrnl_sys::ddk::PKDEFERRED_ROUTINE>(Some(i8042KbdDpcRoutine as unsafe extern "system" fn(*mut ntkrnl_sys::ke::KDPC, *mut c_void, *mut c_void, *mut c_void))),
                    DeviceExtension as _,
                );
                (*DeviceExtension).PowerWorkItem = IoAllocateWorkItem(DeviceObject);
                if (*DeviceExtension).PowerWorkItem.is_null() {
                    Status = STATUS_INSUFFICIENT_RESOURCES;
                    break 'block;
                }
                (*DeviceExtension).DebugWorkItem = IoAllocateWorkItem(DeviceObject);
                if (*DeviceExtension).DebugWorkItem.is_null() {
                    Status = STATUS_INSUFFICIENT_RESOURCES;
                    break 'block;
                }
                (*(*DeviceExtension).Common.PortDeviceExtension).KeyboardExtension =
                    DeviceExtension;
                (*(*DeviceExtension).Common.PortDeviceExtension).Flags |= KEYBOARD_CONNECTED;

                i8042InitializeKeyboardAttributes(DeviceExtension);

                IoMarkIrpPending(Irp);
                (*DeviceExtension).KeyboardHook.QueueKeyboardPacket = Some(i8042KbdQueuePacket);
                (*DeviceExtension).KeyboardHook.CallContext = DeviceExtension as _;
                IoQueueWorkItem(
                    WorkItem,
                    core::mem::transmute::<Option<unsafe extern "system" fn(*mut ntkrnl_sys::ddk::_DEVICE_OBJECT, *mut c_void)>, ntkrnl_sys::ddk::PIO_WORKITEM_ROUTINE>(Some(i8042SendHookWorkItem as unsafe extern "system" fn(*mut ntkrnl_sys::ddk::_DEVICE_OBJECT, *mut c_void))),
                    DelayedWorkQueue,
                    WorkItemData as _,
                );
                Status = STATUS_PENDING;
                break 'block;
            }

            // cleanup — C reaches this label ONLY via `goto cleanup` on the error
            // paths; the success path does `break` out of the switch and SKIPS it.
            // The Rust `'block`/`break 'block` fall-through runs this code for ALL
            // paths, so it must be gated to the failure case. Success == STATUS_PENDING;
            // every error path sets a non-PENDING Status before `break 'block`.
            if Status != STATUS_PENDING {
                // Only free the KeyboardBuffer if THIS call allocated it (i.e. the
                // pointer changed from what it was on entry). Prevents freeing a
                // live buffer left by a previous successful CONNECT.
                if !(*DeviceExtension).KeyboardBuffer.is_null()
                    && (*DeviceExtension).KeyboardBuffer != buffer_before
                {
                    ExFreePoolWithTag((*DeviceExtension).KeyboardBuffer as _, I8042PRT_TAG);
                    (*DeviceExtension).KeyboardBuffer = buffer_before;
                }
                if !(*DeviceExtension).PowerWorkItem.is_null()
                    && (*DeviceExtension).PowerWorkItem != power_wi_before
                {
                    IoFreeWorkItem((*DeviceExtension).PowerWorkItem);
                    (*DeviceExtension).PowerWorkItem = power_wi_before;
                }
                if !(*DeviceExtension).DebugWorkItem.is_null()
                    && (*DeviceExtension).DebugWorkItem != debug_wi_before
                {
                    IoFreeWorkItem((*DeviceExtension).DebugWorkItem);
                    (*DeviceExtension).DebugWorkItem = debug_wi_before;
                }
                if !WorkItem.is_null() {
                    IoFreeWorkItem(WorkItem);
                }
                if !WorkItemData.is_null() {
                    ExFreePoolWithTag(WorkItemData as _, I8042PRT_TAG);
                }
            }
        }
        IOCTL_INTERNAL_KEYBOARD_DISCONNECT => {
            Status = STATUS_NOT_IMPLEMENTED;
        }
        IOCTL_INTERNAL_I8042_HOOK_KEYBOARD => {
            Status = STATUS_SUCCESS;
        }
        IOCTL_KEYBOARD_QUERY_ATTRIBUTES => {
            if ((*Stack).Parameters.DeviceIoControl.OutputBufferLength as usize)
                < core::mem::size_of::<KEYBOARD_ATTRIBUTES>()
            {
                Status = STATUS_BUFFER_TOO_SMALL;
            } else {
                let KeyboardAttributes: PKEYBOARD_ATTRIBUTES =
                    (*Irp).AssociatedIrp.SystemBuffer as PKEYBOARD_ATTRIBUTES;
                *KeyboardAttributes = (*DeviceExtension).KeyboardAttributes;

                (*Irp).IoStatus.Information = core::mem::size_of::<KEYBOARD_ATTRIBUTES>() as _;
                Status = STATUS_SUCCESS;
            }
        }
        IOCTL_KEYBOARD_QUERY_TYPEMATIC => {
            Status = STATUS_NOT_IMPLEMENTED;
        }
        IOCTL_KEYBOARD_SET_TYPEMATIC => {
            Status = STATUS_NOT_IMPLEMENTED;
        }
        IOCTL_KEYBOARD_QUERY_INDICATOR_TRANSLATION => {
            if ((*Stack).Parameters.DeviceIoControl.OutputBufferLength as usize)
                < core::mem::size_of::<LOCAL_KEYBOARD_INDICATOR_TRANSLATION>()
            {
                Status = STATUS_BUFFER_TOO_SMALL;
            } else {
                RtlCopyMemory(
                    (*Irp).AssociatedIrp.SystemBuffer as _,
                    core::ptr::addr_of!(IndicatorTranslation) as _,
                    core::mem::size_of::<LOCAL_KEYBOARD_INDICATOR_TRANSLATION>(),
                );
                (*Irp).IoStatus.Information =
                    core::mem::size_of::<LOCAL_KEYBOARD_INDICATOR_TRANSLATION>() as _;
                Status = STATUS_SUCCESS;
            }
        }
        IOCTL_KEYBOARD_QUERY_INDICATORS => {
            if ((*Stack).Parameters.DeviceIoControl.OutputBufferLength as usize)
                < core::mem::size_of::<KEYBOARD_INDICATOR_PARAMETERS>()
            {
                Status = STATUS_BUFFER_TOO_SMALL;
            } else {
                RtlCopyMemory(
                    (*Irp).AssociatedIrp.SystemBuffer as _,
                    core::ptr::addr_of!((*DeviceExtension).KeyboardIndicators) as _,
                    core::mem::size_of::<KEYBOARD_INDICATOR_PARAMETERS>(),
                );
                (*Irp).IoStatus.Information =
                    core::mem::size_of::<KEYBOARD_INDICATOR_PARAMETERS>() as _;
                Status = STATUS_SUCCESS;
            }
        }
        IOCTL_KEYBOARD_SET_INDICATORS => {
            if ((*Stack).Parameters.DeviceIoControl.InputBufferLength as usize)
                < core::mem::size_of::<KEYBOARD_INDICATOR_PARAMETERS>()
            {
                Status = STATUS_BUFFER_TOO_SMALL;
            } else {
                RtlCopyMemory(
                    core::ptr::addr_of_mut!((*DeviceExtension).KeyboardIndicators) as _,
                    (*Irp).AssociatedIrp.SystemBuffer as _,
                    core::mem::size_of::<KEYBOARD_INDICATOR_PARAMETERS>(),
                );
                Status = STATUS_PENDING;
                IoMarkIrpPending(Irp);
                IoStartPacket(
                    DeviceObject,
                    Irp,
                    core::ptr::null_mut(),
                    None,
                );
            }
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

unsafe fn i8042KbdCallIsrHook(
    DeviceExtension: PI8042_KEYBOARD_EXTENSION,
    Status: UCHAR,
    Input: UCHAR,
    ToReturn: PBOOLEAN,
) -> BOOLEAN {
    let HookReturn: BOOLEAN;
    let mut HookContinue: BOOLEAN = 0u8;
    let mut Input = Input;

    if let Some(IsrRoutine) = (*DeviceExtension).KeyboardHook.IsrRoutine {
        HookReturn = IsrRoutine(
            (*DeviceExtension).KeyboardHook.Context,
            (*DeviceExtension)
                .KeyboardBuffer
                .add((*DeviceExtension).KeysInBuffer as usize),
            core::ptr::addr_of_mut!((*(*DeviceExtension).Common.PortDeviceExtension).Packet),
            Status,
            core::ptr::addr_of_mut!(Input),
            core::ptr::addr_of_mut!(HookContinue),
            core::ptr::addr_of_mut!((*DeviceExtension).KeyboardScanState),
        );

        if HookContinue == 0 {
            *ToReturn = HookReturn;
            return 1u8;
        }
    }
    0u8
}

#[no_mangle]
pub unsafe extern "system" fn i8042KbdInterruptService(
    Interrupt: PKINTERRUPT,
    Context: PVOID,
) -> BOOLEAN {
    let DeviceExtension: PI8042_KEYBOARD_EXTENSION;
    let PortDeviceExtension: PPORT_DEVICE_EXTENSION;
    let InputData: PKEYBOARD_INPUT_DATA;
    let mut Counter: ULONG;
    let mut PortStatus: UCHAR = 0;
    let mut Output: UCHAR = 0;
    let mut ToReturn: BOOLEAN = 0u8;
    let mut Status: NTSTATUS;

    let _ = Interrupt;

    // Delivery-chain boot marker (one-shot): the kbd ISR actually fired.
    {
        static mut g_FirstKbdIsrLogged: LONG = 0;
        let dest = core::ptr::addr_of_mut!(g_FirstKbdIsrLogged);
        if *dest == 0 {
            *dest = 1;
            DbgPrint(b"[I8042] first kbd IRQ fired\n\0".as_ptr() as *const i8);
        }
    }

    DeviceExtension = Context as PI8042_KEYBOARD_EXTENSION;

    PortDeviceExtension = (*DeviceExtension).Common.PortDeviceExtension;
    InputData = (*DeviceExtension)
        .KeyboardBuffer
        .add((*DeviceExtension).KeysInBuffer as usize);
    Counter = (*PortDeviceExtension).Settings.PollStatusIterations;

    while Counter != 0 {
        Status = i8042ReadStatus(PortDeviceExtension, &mut PortStatus);
        if !NT_SUCCESS(Status) {
            return 0u8;
        }
        Status = i8042ReadKeyboardData(PortDeviceExtension, &mut Output);
        if NT_SUCCESS(Status) {
            break;
        }
        KeStallExecutionProcessor(1);
        Counter -= 1;
    }
    if Counter == 0 {
        return 0u8;
    }

    if (*PortDeviceExtension).Settings.CrashOnCtrlScroll != 0 {
        static ScanCodes: [UCHAR; 5] = [0x1d, 0x46, 0xc6, 0x46, 0];

        if Output == ScanCodes[(*DeviceExtension).ComboPosition as usize] {
            (*DeviceExtension).ComboPosition += 1;
            if ScanCodes[(*DeviceExtension).ComboPosition as usize] == 0 {
                KeBugCheck(MANUALLY_INITIATED_CRASH);
            }
        } else if Output == 0xfa {
            /* Ignore ACK */
        } else if Output == ScanCodes[0] {
            (*DeviceExtension).ComboPosition = 1;
        } else {
            (*DeviceExtension).ComboPosition = 0;
        }

        if (*InputData).MakeCode == 0x0F {
            (*DeviceExtension).TabPressed =
                if (*InputData).Flags & KEY_BREAK as USHORT != 0 { 0u8 } else { 1u8 };
        } else if (*DeviceExtension).TabPressed != 0 {
            (*DeviceExtension).TabPressed = 0u8;

            if (*InputData).MakeCode == 0x25 {
                DbgBreakPointWithStatus(DBG_STATUS_SYSRQ);
            } else if (*InputData).MakeCode == 0x30 {
                KeBugCheck(MANUALLY_INITIATED_CRASH);
            } else {
                KdSystemDebugControl(
                    // C multichar literal ' soR' = (' '<<24)|('s'<<16)|('o'<<8)|'R'
                    0x20736f52u32 as SYSDBG_COMMAND,
                    (*InputData).MakeCode as ULONG_PTR as PVOID,
                    0,
                    core::ptr::null_mut(),
                    0,
                    core::ptr::null_mut(),
                    0i32 as KPROCESSOR_MODE,
                );
            }
        }
    }

    if i8042KbdCallIsrHook(DeviceExtension, PortStatus, Output, &mut ToReturn) != 0 {
        return ToReturn;
    }

    /* Only consult the shared packet machine when the in-flight packet
     * targets the KEYBOARD (PacketPort == 0). Mirror of the mouse-ISR gate —
     * see mouse.rs i8042MouInterruptService for the full story (the C calls
     * i8042PacketIsr unconditionally from both ISRs, so a keyboard packet in
     * flight ate the mouse's reset ACKs; that stall killed the PS/2 mouse on
     * every 2026-07-06 boot). */
    if (*PortDeviceExtension).Packet.State != Idle
        && (*PortDeviceExtension).PacketPort == 0
        && i8042PacketIsr(PortDeviceExtension, Output) != 0
    {
        if (*PortDeviceExtension).PacketComplete != 0 {
            KeInsertQueueDpc(
                core::ptr::addr_of_mut!((*DeviceExtension).DpcKeyboard),
                core::ptr::null_mut(),
                core::ptr::null_mut(),
            );
        }
        return 1u8;
    }

    if (*DeviceExtension).KeyboardScanState == Normal {
        match Output {
            0xe0 => {
                (*DeviceExtension).KeyboardScanState = GotE0;
                return 1u8;
            }
            0xe1 => {
                (*DeviceExtension).KeyboardScanState = GotE1;
                return 1u8;
            }
            _ => {}
        }
    }

    (*InputData).Flags = 0;
    match (*DeviceExtension).KeyboardScanState {
        x if x == GotE0 => {
            (*InputData).Flags |= KEY_E0 as USHORT;
        }
        x if x == GotE1 => {
            (*InputData).Flags |= KEY_E1 as USHORT;
        }
        _ => {}
    }
    (*DeviceExtension).KeyboardScanState = Normal;
    if Output & 0x80 != 0 {
        (*InputData).Flags |= KEY_BREAK as USHORT;
    } else {
        (*InputData).Flags |= KEY_MAKE as USHORT;
    }
    (*InputData).MakeCode = (Output & 0x7f) as USHORT;
    (*InputData).Reserved = 0;

    if let Some(QueueKeyboardPacket) = (*DeviceExtension).KeyboardHook.QueueKeyboardPacket {
        QueueKeyboardPacket((*DeviceExtension).KeyboardHook.CallContext);
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
//  * FILE:        drivers/input/i8042prt/keyboard.c
//  * PURPOSE:     Keyboard specific functions
//  * PROGRAMMERS: Copyright Victor Kirhenshtein (sauros@iname.com)
//                 Copyright Jason Filby (jasonfilby@yahoo.com)
//                 Copyright Martijn Vernooij (o112w8r02@sneakemail.com)
//                 Copyright 2006-2007 Herv� Poussineau (hpoussin@reactos.org)
//  */
//
// /* INCLUDES ****************************************************************/
//
// #include "i8042prt.h"
//
// #include <poclass.h>
// #include <ndk/kdfuncs.h>
//
// #include <debug.h>
//
// /* GLOBALS *******************************************************************/
//
// static IO_WORKITEM_ROUTINE i8042PowerWorkItem;
// static KDEFERRED_ROUTINE i8042KbdDpcRoutine;
//
// /* This structure starts with the same layout as KEYBOARD_INDICATOR_TRANSLATION */
// typedef struct _LOCAL_KEYBOARD_INDICATOR_TRANSLATION {
// 	USHORT NumberOfIndicatorKeys;
// 	INDICATOR_LIST IndicatorList[3];
// } LOCAL_KEYBOARD_INDICATOR_TRANSLATION, *PLOCAL_KEYBOARD_INDICATOR_TRANSLATION;
//
// static LOCAL_KEYBOARD_INDICATOR_TRANSLATION IndicatorTranslation = { 3, {
// 	{0x3A, KEYBOARD_CAPS_LOCK_ON},
// 	{0x45, KEYBOARD_NUM_LOCK_ON},
// 	{0x46, KEYBOARD_SCROLL_LOCK_ON}}};
//
// /* FUNCTIONS *****************************************************************/
//
// /*
//  * These functions are callbacks for filter driver custom interrupt
//  * service routines.
//  */
// /*static VOID NTAPI
// i8042KbdIsrWritePort(
// 	IN PVOID Context,
// 	IN UCHAR Value)
// {
// 	PI8042_KEYBOARD_EXTENSION DeviceExtension;
//
// 	DeviceExtension = (PI8042_KEYBOARD_EXTENSION)Context;
//
// 	if (DeviceExtension->KeyboardHook.IsrWritePort)
// 	{
// 		DeviceExtension->KeyboardHook.IsrWritePort(
// 			DeviceExtension->KeyboardHook.CallContext,
// 			Value);
// 	}
// 	else
// 		i8042IsrWritePort(Context, Value, 0);
// }*/
//
// static VOID NTAPI
// i8042KbdQueuePacket(
// 	IN PVOID Context)
// {
// 	PI8042_KEYBOARD_EXTENSION DeviceExtension;
//
// 	DeviceExtension = (PI8042_KEYBOARD_EXTENSION)Context;
//
// 	DeviceExtension->KeyComplete = TRUE;
// 	DeviceExtension->KeysInBuffer++;
// 	if (DeviceExtension->KeysInBuffer > DeviceExtension->Common.PortDeviceExtension->Settings.KeyboardDataQueueSize)
// 	{
// 		WARN_(I8042PRT, "Keyboard buffer overflow\n");
// 		DeviceExtension->KeysInBuffer--;
// 	}
//
// 	TRACE_(I8042PRT, "Irq completes key\n");
// 	KeInsertQueueDpc(&DeviceExtension->DpcKeyboard, NULL, NULL);
// }
//
// /*
//  * These functions are callbacks for filter driver custom
//  * initialization routines.
//  */
// NTSTATUS NTAPI
// i8042SynchWritePortKbd(
// 	IN PVOID Context,
// 	IN UCHAR Value,
// 	IN BOOLEAN WaitForAck)
// {
// 	return i8042SynchWritePort(
// 		(PPORT_DEVICE_EXTENSION)Context,
// 		0,
// 		Value,
// 		WaitForAck);
// }
//
// /*
//  * Process the keyboard internal device requests
//  */
// VOID NTAPI
// i8042KbdStartIo(
// 	IN PDEVICE_OBJECT DeviceObject,
// 	IN PIRP Irp)
// {
// 	PIO_STACK_LOCATION Stack;
// 	PI8042_KEYBOARD_EXTENSION DeviceExtension;
// 	PPORT_DEVICE_EXTENSION PortDeviceExtension;
//
// 	Stack = IoGetCurrentIrpStackLocation(Irp);
// 	DeviceExtension = (PI8042_KEYBOARD_EXTENSION)DeviceObject->DeviceExtension;
// 	PortDeviceExtension = DeviceExtension->Common.PortDeviceExtension;
//
// 	switch (Stack->Parameters.DeviceIoControl.IoControlCode)
// 	{
// 		case IOCTL_KEYBOARD_SET_INDICATORS:
// 		{
// 			TRACE_(I8042PRT, "IOCTL_KEYBOARD_SET_INDICATORS\n");
// 			INFO_(I8042PRT, "Leds: {%s%s%s }\n",
// 				DeviceExtension->KeyboardIndicators.LedFlags & KEYBOARD_CAPS_LOCK_ON ? " CAPSLOCK" : "",
// 				DeviceExtension->KeyboardIndicators.LedFlags & KEYBOARD_NUM_LOCK_ON ? " NUMLOCK" : "",
// 				DeviceExtension->KeyboardIndicators.LedFlags & KEYBOARD_SCROLL_LOCK_ON ? " SCROLLLOCK" : "");
//
// 			PortDeviceExtension->PacketBuffer[0] = KBD_CMD_SET_LEDS;
// 			PortDeviceExtension->PacketBuffer[1] = 0;
// 			if (DeviceExtension->KeyboardIndicators.LedFlags & KEYBOARD_CAPS_LOCK_ON)
// 				PortDeviceExtension->PacketBuffer[1] |= KBD_LED_CAPS;
//
// 			if (DeviceExtension->KeyboardIndicators.LedFlags & KEYBOARD_NUM_LOCK_ON)
// 				PortDeviceExtension->PacketBuffer[1] |= KBD_LED_NUM;
//
// 			if (DeviceExtension->KeyboardIndicators.LedFlags & KEYBOARD_SCROLL_LOCK_ON)
// 				PortDeviceExtension->PacketBuffer[1] |= KBD_LED_SCROLL;
//
// 			i8042StartPacket(
// 				PortDeviceExtension,
// 				&DeviceExtension->Common,
// 				PortDeviceExtension->PacketBuffer,
// 				2,
// 				Irp);
// 			break;
// 		}
// 		default:
// 		{
// 			ERR_(I8042PRT, "Unknown ioctl code 0x%lx\n",
// 				Stack->Parameters.DeviceIoControl.IoControlCode);
// 			ASSERT(FALSE);
// 		}
// 	}
// }
//
// static VOID
// i8042PacketDpc(
// 	IN PPORT_DEVICE_EXTENSION DeviceExtension)
// {
// 	BOOLEAN FinishIrp = FALSE;
// 	KIRQL Irql;
// 	NTSTATUS Result = STATUS_INTERNAL_ERROR; /* Shouldn't happen */
//
// 	/* If the interrupt happens before this is setup, the key
// 	 * was already in the buffer. Too bad! */
// 	if (!DeviceExtension->HighestDIRQLInterrupt)
// 		return;
//
// 	Irql = KeAcquireInterruptSpinLock(DeviceExtension->HighestDIRQLInterrupt);
//
// 	if (DeviceExtension->Packet.State == Idle
// 	 && DeviceExtension->PacketComplete)
// 	{
// 		FinishIrp = TRUE;
// 		Result = DeviceExtension->PacketResult;
// 		DeviceExtension->PacketComplete = FALSE;
// 	}
//
// 	KeReleaseInterruptSpinLock(DeviceExtension->HighestDIRQLInterrupt, Irql);
//
// 	if (!FinishIrp)
// 		return;
//
// 	if (DeviceExtension->CurrentIrp)
// 	{
// 		DeviceExtension->CurrentIrp->IoStatus.Status = Result;
// 		IoCompleteRequest(DeviceExtension->CurrentIrp, IO_NO_INCREMENT);
// 		IoStartNextPacket(DeviceExtension->CurrentIrpDevice, FALSE);
// 		DeviceExtension->CurrentIrp = NULL;
// 		DeviceExtension->CurrentIrpDevice = NULL;
// 	}
// }
//
// static VOID NTAPI
// i8042PowerWorkItem(
// 	IN PDEVICE_OBJECT DeviceObject,
// 	IN PVOID Context)
// {
// 	PI8042_KEYBOARD_EXTENSION DeviceExtension;
// 	PIRP WaitingIrp;
// 	NTSTATUS Status;
//
// 	UNREFERENCED_PARAMETER(DeviceObject);
//
// 	__analysis_assume(Context != NULL);
// 	DeviceExtension = Context;
//
// 	/* See https://learn.microsoft.com/en-us/archive/blogs/doronh/how-ps2-and-hid-keyboads-report-power-button-events */
//
// 	/* Register GUID_DEVICE_SYS_BUTTON interface and report capability */
// 	if (DeviceExtension->NewCaps != DeviceExtension->ReportedCaps)
// 	{
// 		WaitingIrp = InterlockedExchangePointer((PVOID)&DeviceExtension->PowerIrp, NULL);
// 		if (WaitingIrp)
// 		{
// 			/* Cancel the current power irp, as capability changed */
// 			WaitingIrp->IoStatus.Status = STATUS_UNSUCCESSFUL;
// 			WaitingIrp->IoStatus.Information = sizeof(ULONG);
// 			IoCompleteRequest(WaitingIrp, IO_NO_INCREMENT);
// 		}
//
// 		if (DeviceExtension->PowerInterfaceName.MaximumLength == 0)
// 		{
// 			/* We have never registered this interface ; do it */
// 			Status = IoRegisterDeviceInterface(
// 				DeviceExtension->Common.Pdo,
// 				&GUID_DEVICE_SYS_BUTTON,
// 				NULL,
// 				&DeviceExtension->PowerInterfaceName);
// 			if (!NT_SUCCESS(Status))
// 			{
// 				/* We can't do more yet, ignore the keypress... */
// 				WARN_(I8042PRT, "IoRegisterDeviceInterface(GUID_DEVICE_SYS_BUTTON) failed with status 0x%08lx\n",
// 					Status);
// 				DeviceExtension->PowerInterfaceName.MaximumLength = 0;
// 				return;
// 			}
// 		}
// 		else
// 		{
// 			/* Disable the interface. Once activated again, capabilities would be asked again */
// 			Status = IoSetDeviceInterfaceState(
// 				&DeviceExtension->PowerInterfaceName,
// 				FALSE);
// 			if (!NT_SUCCESS(Status))
// 			{
// 				/* Ignore the key press... */
// 				WARN_(I8042PRT, "Disabling interface %wZ failed with status 0x%08lx\n",
// 					&DeviceExtension->PowerInterfaceName, Status);
// 				return;
// 			}
// 		}
// 		/* Enable the interface. This leads to receiving a IOCTL_GET_SYS_BUTTON_CAPS,
// 		 * so we can report new capability */
// 		Status = IoSetDeviceInterfaceState(
// 				&DeviceExtension->PowerInterfaceName,
// 				TRUE);
// 		if (!NT_SUCCESS(Status))
// 		{
// 			/* Ignore the key press... */
// 			WARN_(I8042PRT, "Enabling interface %wZ failed with status 0x%08lx\n",
// 					&DeviceExtension->PowerInterfaceName, Status);
// 			return;
// 		}
// 	}
//
// 	/* Directly complete the IOCTL_GET_SYS_BUTTON_EVENT Irp (if any) */
// 	WaitingIrp = InterlockedExchangePointer((PVOID)&DeviceExtension->PowerIrp, NULL);
// 	if (WaitingIrp)
// 	{
// 		PULONG pEvent = (PULONG)WaitingIrp->AssociatedIrp.SystemBuffer;
//
// 		WaitingIrp->IoStatus.Status = STATUS_SUCCESS;
// 		WaitingIrp->IoStatus.Information = sizeof(ULONG);
// 		*pEvent = InterlockedExchangePointer((PLONG)&DeviceExtension->LastPowerKey, 0);
// 		IoCompleteRequest(WaitingIrp, IO_NO_INCREMENT);
// 	}
// }
//
// /* Return TRUE if it was a power key */
// static BOOLEAN
// HandlePowerKeys(
// 	IN PI8042_KEYBOARD_EXTENSION DeviceExtension)
// {
// 	PKEYBOARD_INPUT_DATA InputData;
// 	ULONG KeyPress;
//
// 	InputData = DeviceExtension->KeyboardBuffer + DeviceExtension->KeysInBuffer - 1;
// 	if (!(InputData->Flags & KEY_E0))
// 		return FALSE;
//
// 	switch (InputData->MakeCode)
// 	{
// 		case KEYBOARD_POWER_CODE:
// 			KeyPress = SYS_BUTTON_POWER;
// 			break;
// 		case KEYBOARD_SLEEP_CODE:
// 			KeyPress = SYS_BUTTON_SLEEP;
// 			break;
// 		case KEYBOARD_WAKE_CODE:
// 			KeyPress = SYS_BUTTON_WAKE;
// 			break;
// 		default:
// 			return FALSE;
// 	}
//
//     if (InputData->Flags & KEY_BREAK)
// 		/* We already took care of the key press */
// 		return TRUE;
//
// 	/* Our work can only be done at passive level, so use a workitem */
// 	DeviceExtension->NewCaps |= KeyPress;
// 	InterlockedExchangePointer((PLONG)&DeviceExtension->LastPowerKey, KeyPress);
// 	IoQueueWorkItem(
// 		DeviceExtension->PowerWorkItem,
// 		&i8042PowerWorkItem,
// 		DelayedWorkQueue,
// 		DeviceExtension);
// 	return TRUE;
// }
//
// static VOID NTAPI
// i8042KbdDpcRoutine(
// 	IN PKDPC Dpc,
// 	IN PVOID DeferredContext,
// 	IN PVOID SystemArgument1,
// 	IN PVOID SystemArgument2)
// {
// 	PI8042_KEYBOARD_EXTENSION DeviceExtension;
// 	PPORT_DEVICE_EXTENSION PortDeviceExtension;
// 	ULONG KeysTransferred = 0;
// 	ULONG KeysInBufferCopy;
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
// 	if (HandlePowerKeys(DeviceExtension))
// 	{
// 		DeviceExtension->KeyComplete = FALSE;
// 		return;
// 	}
//
// 	i8042PacketDpc(PortDeviceExtension);
// 	if (!DeviceExtension->KeyComplete)
// 		return;
// 	/* We got the interrupt as it was being enabled, too bad */
// 	if (!PortDeviceExtension->HighestDIRQLInterrupt)
// 		return;
//
// 	Irql = KeAcquireInterruptSpinLock(PortDeviceExtension->HighestDIRQLInterrupt);
//
// 	DeviceExtension->KeyComplete = FALSE;
// 	KeysInBufferCopy = DeviceExtension->KeysInBuffer;
//
// 	KeReleaseInterruptSpinLock(PortDeviceExtension->HighestDIRQLInterrupt, Irql);
//
// 	TRACE_(I8042PRT, "Send a key\n");
//
// 	if (!DeviceExtension->KeyboardData.ClassService)
// 		return;
//
// 	INFO_(I8042PRT, "Sending %lu key(s)\n", KeysInBufferCopy);
// 	(*(PSERVICE_CALLBACK_ROUTINE)DeviceExtension->KeyboardData.ClassService)(
// 		DeviceExtension->KeyboardData.ClassDeviceObject,
// 		DeviceExtension->KeyboardBuffer,
// 		DeviceExtension->KeyboardBuffer + KeysInBufferCopy,
// 		&KeysTransferred);
//
// 	/* Validate that the callback didn't change the Irql. */
// 	ASSERT(KeGetCurrentIrql() == Irql);
//
// 	Irql = KeAcquireInterruptSpinLock(PortDeviceExtension->HighestDIRQLInterrupt);
// 	DeviceExtension->KeysInBuffer -= KeysTransferred;
// 	KeReleaseInterruptSpinLock(PortDeviceExtension->HighestDIRQLInterrupt, Irql);
// }
//
// /*
//  * Runs the keyboard IOCTL dispatch.
//  */
// NTSTATUS NTAPI
// i8042KbdDeviceControl(
// 	IN PDEVICE_OBJECT DeviceObject,
// 	IN PIRP Irp)
// {
// 	PIO_STACK_LOCATION Stack;
// 	PI8042_KEYBOARD_EXTENSION DeviceExtension;
// 	NTSTATUS Status;
//
// 	Stack = IoGetCurrentIrpStackLocation(Irp);
// 	Irp->IoStatus.Information = 0;
// 	DeviceExtension = (PI8042_KEYBOARD_EXTENSION)DeviceObject->DeviceExtension;
//
// 	switch (Stack->Parameters.DeviceIoControl.IoControlCode)
// 	{
// 		case IOCTL_GET_SYS_BUTTON_CAPS:
// 		{
// 			/* Part of GUID_DEVICE_SYS_BUTTON interface */
// 			PULONG pCaps;
// 			TRACE_(I8042PRT, "IOCTL_GET_SYS_BUTTON_CAPS\n");
//
// 			if (Stack->Parameters.DeviceIoControl.OutputBufferLength != sizeof(ULONG))
// 				Status = STATUS_INVALID_PARAMETER;
// 			else
// 			{
// 				pCaps = (PULONG)Irp->AssociatedIrp.SystemBuffer;
// 				*pCaps = DeviceExtension->NewCaps;
// 				DeviceExtension->ReportedCaps = DeviceExtension->NewCaps;
// 				Irp->IoStatus.Information = sizeof(ULONG);
// 				Status = STATUS_SUCCESS;
// 			}
// 			break;
// 		}
// 		case IOCTL_GET_SYS_BUTTON_EVENT:
// 		{
// 			/* Part of GUID_DEVICE_SYS_BUTTON interface */
// 			PIRP WaitingIrp;
// 			TRACE_(I8042PRT, "IOCTL_GET_SYS_BUTTON_EVENT\n");
//
// 			if (Stack->Parameters.DeviceIoControl.OutputBufferLength != sizeof(ULONG))
// 				Status = STATUS_INVALID_PARAMETER;
// 			else
// 			{
// 				WaitingIrp = InterlockedCompareExchangePointer(
// 					(PVOID)&DeviceExtension->PowerIrp,
// 					Irp,
// 					NULL);
// 				/* Check if an Irp is already pending */
// 				if (WaitingIrp)
// 				{
// 					/* Unable to have a 2nd pending IRP for this IOCTL */
// 					WARN_(I8042PRT, "Unable to pend a second IRP for IOCTL_GET_SYS_BUTTON_EVENT\n");
// 					Status = STATUS_INVALID_PARAMETER;
// 					Irp->IoStatus.Status = Status;
// 					IoCompleteRequest(Irp, IO_NO_INCREMENT);
// 				}
// 				else
// 				{
// 					ULONG PowerKey;
// 					PowerKey = InterlockedExchangePointer((PLONG)&DeviceExtension->LastPowerKey, 0);
// 					if (PowerKey != 0)
// 					{
// 						(VOID)InterlockedCompareExchangePointer((PVOID)&DeviceExtension->PowerIrp, NULL, Irp);
// 						*(PULONG)Irp->AssociatedIrp.SystemBuffer = PowerKey;
// 						Status = STATUS_SUCCESS;
// 						Irp->IoStatus.Status = Status;
// 						Irp->IoStatus.Information = sizeof(ULONG);
// 						IoCompleteRequest(Irp, IO_NO_INCREMENT);
// 					}
// 					else
// 					{
// 						TRACE_(I8042PRT, "Pending IOCTL_GET_SYS_BUTTON_EVENT\n");
// 						Status = STATUS_PENDING;
// 						Irp->IoStatus.Status = Status;
// 						IoMarkIrpPending(Irp);
// 					}
// 				}
// 				return Status;
// 			}
// 			break;
// 		}
// 		default:
// 		{
// 			ERR_(I8042PRT, "IRP_MJ_DEVICE_CONTROL / unknown ioctl code 0x%lx\n",
// 				Stack->Parameters.DeviceIoControl.IoControlCode);
// 			return ForwardIrpAndForget(DeviceObject, Irp);
// 		}
// 	}
//
// 	if (Status != STATUS_PENDING)
// 	{
// 		Irp->IoStatus.Status = Status;
// 		IoCompleteRequest(Irp, IO_NO_INCREMENT);
// 	}
//
// 	return Status;
// }
//
// VOID
// NTAPI
// i8042InitializeKeyboardAttributes(
//     PI8042_KEYBOARD_EXTENSION DeviceExtension)
// {
//     PPORT_DEVICE_EXTENSION PortDeviceExtension;
//     PI8042_SETTINGS Settings;
//     PKEYBOARD_ATTRIBUTES KeyboardAttributes;
//
//     PortDeviceExtension = DeviceExtension->Common.PortDeviceExtension;
//     Settings = &PortDeviceExtension->Settings;
//
//     KeyboardAttributes = &DeviceExtension->KeyboardAttributes;
//
//     KeyboardAttributes->KeyboardIdentifier.Type = (UCHAR)Settings->OverrideKeyboardType;
//     KeyboardAttributes->KeyboardIdentifier.Subtype = (UCHAR)Settings->OverrideKeyboardSubtype;
//     KeyboardAttributes->NumberOfFunctionKeys = 4;
//     KeyboardAttributes->NumberOfIndicators = 3;
//     KeyboardAttributes->NumberOfKeysTotal = 101;
//     KeyboardAttributes->InputDataQueueLength = Settings->KeyboardDataQueueSize;
//     KeyboardAttributes->KeyRepeatMinimum.UnitId = 0;
//     KeyboardAttributes->KeyRepeatMinimum.Rate = (USHORT)Settings->SampleRate;
//     KeyboardAttributes->KeyRepeatMinimum.Delay = 0;
//     KeyboardAttributes->KeyRepeatMinimum.UnitId = 0;
//     KeyboardAttributes->KeyRepeatMinimum.Rate = (USHORT)Settings->SampleRate;
//     KeyboardAttributes->KeyRepeatMinimum.Delay = 0;
// }
//
// /*
//  * Runs the keyboard IOCTL_INTERNAL dispatch.
//  */
// NTSTATUS NTAPI
// i8042KbdInternalDeviceControl(
// 	IN PDEVICE_OBJECT DeviceObject,
// 	IN PIRP Irp)
// {
// 	PIO_STACK_LOCATION Stack;
// 	PI8042_KEYBOARD_EXTENSION DeviceExtension;
// 	NTSTATUS Status;
//
// 	Stack = IoGetCurrentIrpStackLocation(Irp);
// 	Irp->IoStatus.Information = 0;
// 	DeviceExtension = (PI8042_KEYBOARD_EXTENSION)DeviceObject->DeviceExtension;
//
// 	switch (Stack->Parameters.DeviceIoControl.IoControlCode)
// 	{
// 		case IOCTL_INTERNAL_KEYBOARD_CONNECT:
// 		{
// 			SIZE_T Size;
// 			PIO_WORKITEM WorkItem = NULL;
// 			PI8042_HOOK_WORKITEM WorkItemData = NULL;
//
// 			TRACE_(I8042PRT, "IRP_MJ_INTERNAL_DEVICE_CONTROL / IOCTL_INTERNAL_KEYBOARD_CONNECT\n");
// 			if (Stack->Parameters.DeviceIoControl.InputBufferLength != sizeof(CONNECT_DATA))
// 			{
// 				Status = STATUS_INVALID_PARAMETER;
// 				goto cleanup;
// 			}
//
// 			DeviceExtension->KeyboardData =
// 				*((PCONNECT_DATA)Stack->Parameters.DeviceIoControl.Type3InputBuffer);
//
// 			/* Send IOCTL_INTERNAL_I8042_HOOK_KEYBOARD to device stack */
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
// 			DeviceExtension->Common.Type = Keyboard;
// 			Size = DeviceExtension->Common.PortDeviceExtension->Settings.KeyboardDataQueueSize * sizeof(KEYBOARD_INPUT_DATA);
// 			DeviceExtension->KeyboardBuffer = ExAllocatePoolWithTag(
// 				NonPagedPool,
// 				Size,
// 				I8042PRT_TAG);
// 			if (!DeviceExtension->KeyboardBuffer)
// 			{
// 				WARN_(I8042PRT, "ExAllocatePoolWithTag() failed\n");
// 				Status = STATUS_NO_MEMORY;
// 				goto cleanup;
// 			}
// 			RtlZeroMemory(DeviceExtension->KeyboardBuffer, Size);
// 			KeInitializeDpc(
// 				&DeviceExtension->DpcKeyboard,
// 				i8042KbdDpcRoutine,
// 				DeviceExtension);
// 			DeviceExtension->PowerWorkItem = IoAllocateWorkItem(DeviceObject);
// 			if (!DeviceExtension->PowerWorkItem)
// 			{
// 				WARN_(I8042PRT, "IoAllocateWorkItem() failed\n");
// 				Status = STATUS_INSUFFICIENT_RESOURCES;
// 				goto cleanup;
// 			}
// 			DeviceExtension->DebugWorkItem = IoAllocateWorkItem(DeviceObject);
// 			if (!DeviceExtension->DebugWorkItem)
// 			{
// 				WARN_(I8042PRT, "IoAllocateWorkItem() failed\n");
// 				Status = STATUS_INSUFFICIENT_RESOURCES;
// 				goto cleanup;
// 			}
// 			DeviceExtension->Common.PortDeviceExtension->KeyboardExtension = DeviceExtension;
// 			DeviceExtension->Common.PortDeviceExtension->Flags |= KEYBOARD_CONNECTED;
//
//             i8042InitializeKeyboardAttributes(DeviceExtension);
//
// 			IoMarkIrpPending(Irp);
// 			/* FIXME: DeviceExtension->KeyboardHook.IsrWritePort = ; */
// 			DeviceExtension->KeyboardHook.QueueKeyboardPacket = i8042KbdQueuePacket;
// 			DeviceExtension->KeyboardHook.CallContext = DeviceExtension;
// 			IoQueueWorkItem(WorkItem,
// 				i8042SendHookWorkItem,
// 				DelayedWorkQueue,
// 				WorkItemData);
// 			Status = STATUS_PENDING;
// 			break;
//
// cleanup:
// 			if (DeviceExtension->KeyboardBuffer)
// 				ExFreePoolWithTag(DeviceExtension->KeyboardBuffer, I8042PRT_TAG);
// 			if (DeviceExtension->PowerWorkItem)
// 				IoFreeWorkItem(DeviceExtension->PowerWorkItem);
// 			if (DeviceExtension->DebugWorkItem)
// 				IoFreeWorkItem(DeviceExtension->DebugWorkItem);
// 			if (WorkItem)
// 				IoFreeWorkItem(WorkItem);
// 			if (WorkItemData)
// 				ExFreePoolWithTag(WorkItemData, I8042PRT_TAG);
// 			break;
// 		}
// 		case IOCTL_INTERNAL_KEYBOARD_DISCONNECT:
// 		{
// 			TRACE_(I8042PRT, "IRP_MJ_INTERNAL_DEVICE_CONTROL / IOCTL_INTERNAL_KEYBOARD_DISCONNECT\n");
// 			/* MSDN says that operation is to implemented.
// 			 * To implement it, we just have to do:
// 			 * DeviceExtension->KeyboardData.ClassService = NULL;
// 			 */
// 			Status = STATUS_NOT_IMPLEMENTED;
// 			break;
// 		}
// 		case IOCTL_INTERNAL_I8042_HOOK_KEYBOARD:
// 		{
// 			TRACE_(I8042PRT, "IRP_MJ_INTERNAL_DEVICE_CONTROL / IOCTL_INTERNAL_I8042_HOOK_KEYBOARD\n");
// 			/* Nothing to do here */
// 			Status = STATUS_SUCCESS;
// 			break;
// 		}
// 		case IOCTL_KEYBOARD_QUERY_ATTRIBUTES:
// 		{
// 		    PKEYBOARD_ATTRIBUTES KeyboardAttributes;
//
//             /* FIXME: KeyboardAttributes are not initialized anywhere */
// 			TRACE_(I8042PRT, "IRP_MJ_INTERNAL_DEVICE_CONTROL / IOCTL_KEYBOARD_QUERY_ATTRIBUTES\n");
// 			if (Stack->Parameters.DeviceIoControl.OutputBufferLength < sizeof(KEYBOARD_ATTRIBUTES))
// 			{
// 				Status = STATUS_BUFFER_TOO_SMALL;
// 				break;
// 			}
//
//             KeyboardAttributes = Irp->AssociatedIrp.SystemBuffer;
//             *KeyboardAttributes = DeviceExtension->KeyboardAttributes;
//
// 			Irp->IoStatus.Information = sizeof(KEYBOARD_ATTRIBUTES);
// 			Status = STATUS_SUCCESS;
// 			break;
// 		}
// 		case IOCTL_KEYBOARD_QUERY_TYPEMATIC:
// 		{
// 			DPRINT1("IOCTL_KEYBOARD_QUERY_TYPEMATIC not implemented\n");
// 			Status = STATUS_NOT_IMPLEMENTED;
// 			break;
// 		}
// 		case IOCTL_KEYBOARD_SET_TYPEMATIC:
// 		{
// 			DPRINT1("IOCTL_KEYBOARD_SET_TYPEMATIC not implemented\n");
// 			Status = STATUS_NOT_IMPLEMENTED;
// 			break;
// 		}
// 		case IOCTL_KEYBOARD_QUERY_INDICATOR_TRANSLATION:
// 		{
// 			TRACE_(I8042PRT, "IRP_MJ_INTERNAL_DEVICE_CONTROL / IOCTL_KEYBOARD_QUERY_INDICATOR_TRANSLATION\n");
//
// 			/* We should check the UnitID, but it's kind of pointless as
// 			 * all keyboards are supposed to have the same one
// 			 */
// 			if (Stack->Parameters.DeviceIoControl.OutputBufferLength < sizeof(LOCAL_KEYBOARD_INDICATOR_TRANSLATION))
// 			{
// 				Status = STATUS_BUFFER_TOO_SMALL;
// 			}
// 			else
// 			{
// 				RtlCopyMemory(
// 					Irp->AssociatedIrp.SystemBuffer,
// 					&IndicatorTranslation,
// 					sizeof(LOCAL_KEYBOARD_INDICATOR_TRANSLATION));
// 				Irp->IoStatus.Information = sizeof(LOCAL_KEYBOARD_INDICATOR_TRANSLATION);
// 				Status = STATUS_SUCCESS;
// 			}
// 			break;
// 		}
// 		case IOCTL_KEYBOARD_QUERY_INDICATORS:
// 		{
// 			TRACE_(I8042PRT, "IRP_MJ_INTERNAL_DEVICE_CONTROL / IOCTL_KEYBOARD_QUERY_INDICATORS\n");
//
// 			if (Stack->Parameters.DeviceIoControl.OutputBufferLength < sizeof(KEYBOARD_INDICATOR_PARAMETERS))
// 			{
// 				Status = STATUS_BUFFER_TOO_SMALL;
// 			}
// 			else
// 			{
// 				RtlCopyMemory(
// 					Irp->AssociatedIrp.SystemBuffer,
// 					&DeviceExtension->KeyboardIndicators,
// 					sizeof(KEYBOARD_INDICATOR_PARAMETERS));
// 				Irp->IoStatus.Information = sizeof(KEYBOARD_INDICATOR_PARAMETERS);
// 				Status = STATUS_SUCCESS;
// 			}
// 			break;
// 		}
// 		case IOCTL_KEYBOARD_SET_INDICATORS:
// 		{
// 			TRACE_(I8042PRT, "IRP_MJ_INTERNAL_DEVICE_CONTROL / IOCTL_KEYBOARD_SET_INDICATORS\n");
//
// 			if (Stack->Parameters.DeviceIoControl.InputBufferLength < sizeof(KEYBOARD_INDICATOR_PARAMETERS))
// 			{
// 				Status = STATUS_BUFFER_TOO_SMALL;
// 			}
// 			else
// 			{
// 				RtlCopyMemory(
// 					&DeviceExtension->KeyboardIndicators,
// 					Irp->AssociatedIrp.SystemBuffer,
// 					sizeof(KEYBOARD_INDICATOR_PARAMETERS));
// 				Status = STATUS_PENDING;
// 				IoMarkIrpPending(Irp);
// 				IoStartPacket(DeviceObject, Irp, NULL, NULL);
// 			}
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
// /*
//  * Call the customization hook. The ToReturn parameter is about whether
//  * we should go on with the interrupt. The return value is what
//  * we should return (indicating to the system whether someone else
//  * should try to handle the interrupt)
//  */
// static BOOLEAN
// i8042KbdCallIsrHook(
// 	IN PI8042_KEYBOARD_EXTENSION DeviceExtension,
// 	IN UCHAR Status,
// 	IN UCHAR Input,
// 	OUT PBOOLEAN ToReturn)
// {
// 	BOOLEAN HookReturn, HookContinue;
//
// 	HookContinue = FALSE;
//
// 	if (DeviceExtension->KeyboardHook.IsrRoutine)
// 	{
// 		HookReturn = DeviceExtension->KeyboardHook.IsrRoutine(
// 			DeviceExtension->KeyboardHook.Context,
// 			DeviceExtension->KeyboardBuffer + DeviceExtension->KeysInBuffer,
// 			&DeviceExtension->Common.PortDeviceExtension->Packet,
// 			Status,
// 			&Input,
// 			&HookContinue,
// 			&DeviceExtension->KeyboardScanState);
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
// BOOLEAN NTAPI
// i8042KbdInterruptService(
// 	IN PKINTERRUPT Interrupt,
// 	PVOID Context)
// {
// 	PI8042_KEYBOARD_EXTENSION DeviceExtension;
// 	PPORT_DEVICE_EXTENSION PortDeviceExtension;
// 	PKEYBOARD_INPUT_DATA InputData;
// 	ULONG Counter;
// 	UCHAR PortStatus = 0, Output = 0;
// 	BOOLEAN ToReturn = FALSE;
// 	NTSTATUS Status;
//
// 	UNREFERENCED_PARAMETER(Interrupt);
//
// 	__analysis_assume(Context != NULL);
// 	DeviceExtension = Context;
// 	{
// 		static BOOLEAN _i8k_kbd_isr_seen = FALSE;
// 		if (!_i8k_kbd_isr_seen)
// 		{
// 			_i8k_kbd_isr_seen = TRUE;
// 			DPRINT1("[[I8K]] KBD ISR FIRED (first)\n");
// 		}
// 	}
// 	PortDeviceExtension = DeviceExtension->Common.PortDeviceExtension;
// 	InputData = DeviceExtension->KeyboardBuffer + DeviceExtension->KeysInBuffer;
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
// 		Status = i8042ReadKeyboardData(PortDeviceExtension, &Output);
// 		if (NT_SUCCESS(Status))
// 			break;
// 		KeStallExecutionProcessor(1);
// 		Counter--;
// 	}
// 	if (Counter == 0)
// 	{
// 		WARN_(I8042PRT, "Spurious i8042 keyboard interrupt\n");
// 		return FALSE;
// 	}
//
// 	INFO_(I8042PRT, "Got: 0x%02x\n", Output);
//
// 	if (PortDeviceExtension->Settings.CrashOnCtrlScroll)
// 	{
// 		/* Test for CTRL + SCROLL LOCK twice */
// 		static const UCHAR ScanCodes[] = { 0x1d, 0x46, 0xc6, 0x46, 0 };
//
// 		if (Output == ScanCodes[DeviceExtension->ComboPosition])
// 		{
// 			DeviceExtension->ComboPosition++;
// 			if (ScanCodes[DeviceExtension->ComboPosition] == 0)
// 				KeBugCheck(MANUALLY_INITIATED_CRASH);
// 		}
// 		else if (Output == 0xfa)
// 		{
// 		    /* Ignore ACK */
// 		}
// 		else if (Output == ScanCodes[0])
// 			DeviceExtension->ComboPosition = 1;
// 		else
// 			DeviceExtension->ComboPosition = 0;
//
// 		/* Test for TAB + key combination */
// 		if (InputData->MakeCode == 0x0F)
// 			DeviceExtension->TabPressed = !(InputData->Flags & KEY_BREAK);
// 		else if (DeviceExtension->TabPressed)
// 		{
// 			DeviceExtension->TabPressed = FALSE;
//
//             /* Check which action to do */
//             if (InputData->MakeCode == 0x25)
//             {
//                 /* k - Breakpoint */
//                 DbgBreakPointWithStatus(DBG_STATUS_SYSRQ);
//             }
//             else if (InputData->MakeCode == 0x30)
//             {
//                 /* b - Bugcheck */
//                 KeBugCheck(MANUALLY_INITIATED_CRASH);
//             }
//             else
//             {
// 			    /* Send request to the kernel debugger.
// 			     * Unknown requests will be ignored. */
// 			    KdSystemDebugControl(' soR',
// 			                         (PVOID)(ULONG_PTR)InputData->MakeCode,
// 			                         0,
// 			                         NULL,
// 			                         0,
// 			                         NULL,
// 			                         KernelMode);
//             }
// 		}
// 	}
//
// 	if (i8042KbdCallIsrHook(DeviceExtension, PortStatus, Output, &ToReturn))
// 		return ToReturn;
//
// 	if (i8042PacketIsr(PortDeviceExtension, Output))
// 	{
// 		if (PortDeviceExtension->PacketComplete)
// 		{
// 			TRACE_(I8042PRT, "Packet complete\n");
// 			KeInsertQueueDpc(&DeviceExtension->DpcKeyboard, NULL, NULL);
// 		}
// 		TRACE_(I8042PRT, "Irq eaten by packet\n");
// 		return TRUE;
// 	}
//
// 	TRACE_(I8042PRT, "Irq is keyboard input\n");
//
// 	if (DeviceExtension->KeyboardScanState == Normal)
// 	{
// 		switch (Output)
// 		{
// 			case 0xe0:
// 				DeviceExtension->KeyboardScanState = GotE0;
// 				return TRUE;
// 			case 0xe1:
// 				DeviceExtension->KeyboardScanState = GotE1;
// 				return TRUE;
// 			default:
// 				break;
// 		}
// 	}
//
// 	/* Update InputData */
// 	InputData->Flags = 0;
// 	switch (DeviceExtension->KeyboardScanState)
// 	{
// 		case GotE0:
// 			InputData->Flags |= KEY_E0;
// 			break;
// 		case GotE1:
// 			InputData->Flags |= KEY_E1;
// 			break;
// 		default:
// 			break;
// 	}
// 	DeviceExtension->KeyboardScanState = Normal;
// 	if (Output & 0x80)
// 		InputData->Flags |= KEY_BREAK;
// 	else
// 		InputData->Flags |= KEY_MAKE;
// 	InputData->MakeCode = Output & 0x7f;
// 	InputData->Reserved = 0;
//
// 	DeviceExtension->KeyboardHook.QueueKeyboardPacket(DeviceExtension->KeyboardHook.CallContext);
//
// 	return TRUE;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
