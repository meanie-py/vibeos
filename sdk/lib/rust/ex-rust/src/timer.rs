use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::ob::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::probe::*;

use ntkrnl_sys::ddk::{
    _ETIMER, _ETHREAD, _KAPC, _KDPC, _KTHREAD, _KTIMER,
    ACCESS_MASK, ETIMER, ExGetPreviousMode,
    KPROCESSOR_MODE, KeCancelTimer,
    KeFlushQueuedDpcs, KeInitializeSpinLock, KeInsertQueueApc,
    KeReadStateTimer, KeRemoveQueueApc,
    KeRemoveQueueDpc, LARGE_INTEGER, PETHREAD, PETIMER, PHANDLE,
    PKAPC, PKNORMAL_ROUTINE, PLARGE_INTEGER, POBJECT_ATTRIBUTES,
    POBJECT_TYPE, PTIMER_APC_ROUTINE, PTIMER_BASIC_INFORMATION,
    PsGetCurrentThread, RtlInitUnicodeString, TIMER_BASIC_INFORMATION,
    _GENERIC_MAPPING, INFORMATION_CLASS_INFO, OBJECT_TYPE_INITIALIZER,
    ObCreateObjectType, ObDereferenceObjectEx, ObReferenceObjectByHandle,
    ObReferenceObjectSafe, _TIMER_TYPE_NotificationTimer,
    _TIMER_TYPE_SynchronizationTimer, _KWAIT_REASON_Executive,
};
use ntkrnl_sys::{opaque_ptr, opaque_ptr_const};
// KDEFERRED_ROUTINE, KeAcquireSpinLockAtDpcLevel, KeReleaseSpinLockFromDpcLevel
// come from ntkrnl_sys::ke::* (portable wrappers for i386 fastcall variants)

#[no_mangle]
pub static mut ExTimerType: POBJECT_TYPE = core::ptr::null_mut();

const TIMER_QUERY_STATE: ACCESS_MASK = 0x0001;
const TIMER_MODIFY_STATE: ACCESS_MASK = 0x0002;
const TIMER_ALL_ACCESS: ACCESS_MASK = STANDARD_RIGHTS_REQUIRED | SYNCHRONIZE | TIMER_QUERY_STATE | TIMER_MODIFY_STATE;
const ICIF_QUERY: ULONG = 0x0001;
const STATUS_TIMER_RESUME_IGNORED: NTSTATUS = 0x40000025u32 as NTSTATUS;

const TIMER_TYPE_NotificationTimer: u32 = _TIMER_TYPE_NotificationTimer;
const TIMER_TYPE_SynchronizationTimer: u32 = _TIMER_TYPE_SynchronizationTimer;

// ExpWakeListLock: KSPIN_LOCK — must match pointer width for KeAcquireSpinLockAtDpcLevel
#[cfg(target_pointer_width = "64")]
static mut ExpWakeListLock: u64 = 0;
#[cfg(target_pointer_width = "32")]
static mut ExpWakeListLock: u32 = 0;
static mut ExpWakeList: LIST_ENTRY = LIST_ENTRY {
    Flink: core::ptr::null_mut(),
    Blink: core::ptr::null_mut(),
};

static mut ExpTimerMapping: _GENERIC_MAPPING = _GENERIC_MAPPING {
    GenericRead: STANDARD_RIGHTS_READ | TIMER_QUERY_STATE,
    GenericWrite: STANDARD_RIGHTS_WRITE | TIMER_MODIFY_STATE,
    GenericExecute: STANDARD_RIGHTS_EXECUTE | SYNCHRONIZE,
    GenericAll: TIMER_ALL_ACCESS,
};

static ExTimerInfoClass: [INFORMATION_CLASS_INFO; 1] = [
    INFORMATION_CLASS_INFO {
        RequiredSizeQUERY: core::mem::size_of::<TIMER_BASIC_INFORMATION>() as USHORT,
        AlignmentQUERY: core::mem::align_of::<TIMER_BASIC_INFORMATION>() as UCHAR,
        RequiredSizeSET: 0,
        AlignmentSET: 0,
        Flags: ICIF_QUERY as USHORT,
    },
];

extern "system" {
    fn KeInitializeDpc(Dpc: *mut _KDPC, DeferredRoutine: KDEFERRED_ROUTINE, DeferredContext: PVOID);
    fn KeInitializeTimerEx(Timer: *mut _KTIMER, Type: u32);
    fn ObCreateObject( ProbeMode: KPROCESSOR_MODE, ObjectType: POBJECT_TYPE, ObjectAttributes: POBJECT_ATTRIBUTES, AccessMode: KPROCESSOR_MODE, Reserved: PVOID, ObjectSize: ULONG, PagedPoolCharge: ULONG, NonPagedPoolCharge: ULONG, Object: *mut PVOID, ) -> NTSTATUS;
    fn ObInsertObject( Object: PVOID, AccessState: PVOID, DesiredAccess: ACCESS_MASK, AdditionalReferences: ULONG, ReferencedObject: *mut PVOID, Handle: PHANDLE, ) -> NTSTATUS;
    fn ObOpenObjectByName( ObjectAttributes: POBJECT_ATTRIBUTES, ObjectType: POBJECT_TYPE, AccessMode: KPROCESSOR_MODE, AccessState: PVOID, DesiredAccess: ACCESS_MASK, ParseContext: PVOID, Handle: PHANDLE, ) -> NTSTATUS;
    fn KeInitializeApc( Apc: *mut _KAPC, Thread: *mut _KTHREAD, Environment: u8, KernelRoutine: unsafe extern "C" fn( *mut _KAPC, *mut PKNORMAL_ROUTINE, *mut PVOID, *mut PVOID, *mut PVOID, ), RundownRoutine: PVOID, NormalRoutine: PKNORMAL_ROUTINE, Mode: KPROCESSOR_MODE, Context: PVOID, );
    fn KeSetTimerEx( Timer: *mut _KTIMER, DueTime: LARGE_INTEGER, Period: LONG, Dpc: *mut _KDPC, ) -> BOOLEAN;
}
extern "C" {
    fn ProbeForWriteBoolean(Address: PBOOLEAN);
    fn ProbeForWriteHandle(Address: PHANDLE);
    fn ProbeForReadLargeInteger(Address: PLARGE_INTEGER) -> LARGE_INTEGER;
    fn DefaultQueryInfoBufferCheck( Class: u32, InfoClassArray: *const INFORMATION_CLASS_INFO, InfoClassArrayEntries: ULONG, Flags: ULONG, Buffer: PVOID, Length: ULONG, ReturnLength: *mut ULONG, ReturnLengthPtr: *mut ULONG, PreviousMode: KPROCESSOR_MODE, ) -> NTSTATUS;
}
// NTAPI (stdcall on i386) — defs in shim_exports/common_inlines are extern "system"
extern "system" {
    fn InsertTailList(ListHead: PLIST_ENTRY, Entry: PLIST_ENTRY);
}

const IO_NO_INCREMENT: u8 = 0;
const CurrentApcEnvironment: u8 = 0x02;
const ICIF_PROBE_READ_WRITE: ULONG = 0x00000003;

#[no_mangle]
pub unsafe extern "system" fn ExTimerRundown() {
    let Thread = PsGetCurrentThread() as *mut _ETHREAD;
    let mut OldIrql: KIRQL = 0;
    let mut CurrentEntry: PLIST_ENTRY;
    let mut Timer: PETIMER;
    let mut DerefsToDo: ULONG;

    // [guard] KeAcquireSpinLock(addr_of_mut!((*Thread).ActiveTimerListLock), &mut OldIrql);
    let mut active_lock_guard = unsafe { ntkrnl_sys::guards::SpinLockGuard::acquire(
        core::ptr::addr_of_mut!((*Thread).ActiveTimerListLock) as ntkrnl_sys::ke::PKSPIN_LOCK
    ) };

    CurrentEntry = (*Thread).ActiveTimerListHead.Flink;
    while CurrentEntry != core::ptr::addr_of_mut!((*Thread).ActiveTimerListHead) {
        Timer = (CurrentEntry as usize - core::mem::offset_of!(_ETIMER, ActiveTimerListEntry)) as PETIMER;

        ObReferenceObject(Timer as PVOID);
        DerefsToDo = 1;

        // [guard-drop] KeReleaseSpinLock — guard Drop handles this
        drop(active_lock_guard);

        // [guard] KeAcquireSpinLock(addr_of_mut!((*Timer).Lock), &mut OldIrql);
        let timer_lock_guard = unsafe { ntkrnl_sys::guards::SpinLockGuard::acquire(
            core::ptr::addr_of_mut!((*Timer).Lock) as ntkrnl_sys::ke::PKSPIN_LOCK
        ) };

        // [guard] KeAcquireSpinLockAtDpcLevel(addr_of_mut!((*Thread).ActiveTimerListLock));
        let active_at_dpc_guard = unsafe { ntkrnl_sys::guards::SpinLockAtDpcGuard::acquire(
            core::ptr::addr_of_mut!((*Thread).ActiveTimerListLock) as ntkrnl_sys::ke::PKSPIN_LOCK
        ) };

        if (*Timer).ApcAssociated != 0
            && core::ptr::addr_of_mut!((*Thread).Tcb) == (*Timer).TimerApc.Thread
        {
            RemoveEntryList(core::ptr::addr_of_mut!((*Timer).ActiveTimerListEntry));
            (*Timer).ApcAssociated = 0;

            KeCancelTimer(core::ptr::addr_of_mut!((*Timer).KeTimer));
            KeRemoveQueueDpc(core::ptr::addr_of_mut!((*Timer).TimerDpc));
            if KeRemoveQueueApc(core::ptr::addr_of_mut!((*Timer).TimerApc)) != 0 {
                DerefsToDo += 1;
            }

            DerefsToDo += 1;
        }

        // [guard-drop] KeReleaseSpinLockFromDpcLevel — guard Drop handles this
        drop(active_at_dpc_guard);

        // [guard-drop] KeReleaseSpinLock — guard Drop handles this
        drop(timer_lock_guard);

        ObDereferenceObjectEx(Timer as PVOID, DerefsToDo as LONG);

        // [guard] KeAcquireSpinLock(addr_of_mut!((*Thread).ActiveTimerListLock), &mut OldIrql);
        active_lock_guard = unsafe { ntkrnl_sys::guards::SpinLockGuard::acquire(
            core::ptr::addr_of_mut!((*Thread).ActiveTimerListLock) as ntkrnl_sys::ke::PKSPIN_LOCK
        ) };

        CurrentEntry = (*Thread).ActiveTimerListHead.Flink;
    }

    // [guard-drop] KeReleaseSpinLock — guard Drop handles this
    drop(active_lock_guard);
}

// On i386, bindgen's OB_DELETE_METHOD is `extern "stdcall"` (not `extern "system"`).
// On amd64/arm64, it's `extern "C"`. Both match "system" at ABI level, but Rust
// treats them as distinct types. Use cfg to match the exact convention bindgen expects.
unsafe extern "system" fn ExpDeleteTimer(ObjectBody: PVOID) {
    ExpDeleteTimerInner(ObjectBody);
}
#[inline(always)]
unsafe fn ExpDeleteTimerInner(ObjectBody: PVOID) {
    let mut OldIrql: KIRQL = 0;
    let Timer = ObjectBody as PETIMER;

    if !(*Timer).WakeTimerListEntry.Flink.is_null() {
        // [guard] KeAcquireSpinLock(addr_of_mut!(ExpWakeListLock), &mut OldIrql);
        let wake_list_lock_guard = unsafe { ntkrnl_sys::guards::SpinLockGuard::acquire(
            core::ptr::addr_of_mut!(ExpWakeListLock) as *mut _ as ntkrnl_sys::ke::PKSPIN_LOCK
        ) };

        if !(*Timer).WakeTimerListEntry.Flink.is_null() {
            RemoveEntryList(core::ptr::addr_of_mut!((*Timer).WakeTimerListEntry));
            (*Timer).WakeTimerListEntry.Flink = core::ptr::null_mut();
        }

        // [guard-drop] KeReleaseSpinLock — guard Drop handles this
        drop(wake_list_lock_guard);
    }

    KeCancelTimer(core::ptr::addr_of_mut!((*Timer).KeTimer));
    KeFlushQueuedDpcs();
}

unsafe extern "system" fn ExpTimerDpcRoutine(
    Dpc: PKDPC,
    DeferredContext: PVOID,
    SystemArgument1: PVOID,
    SystemArgument2: PVOID,
) {
    let Timer = DeferredContext as PETIMER;
    let mut Inserted: BOOLEAN = 0;

    if ObReferenceObjectSafe(Timer as PVOID) == 0 {
        return;
    }

    // [guard] KeAcquireSpinLockAtDpcLevel(addr_of_mut!((*Timer).Lock));
    let timer_lock_guard = unsafe { ntkrnl_sys::guards::SpinLockAtDpcGuard::acquire(
        core::ptr::addr_of_mut!((*Timer).Lock) as ntkrnl_sys::ke::PKSPIN_LOCK
    ) };

    if (*Timer).ApcAssociated != 0 {
        Inserted = KeInsertQueueApc(
            core::ptr::addr_of_mut!((*Timer).TimerApc),
            SystemArgument1,
            SystemArgument2,
            IO_NO_INCREMENT as i32,
        );
    }

    // [guard-drop] KeReleaseSpinLockFromDpcLevel — guard Drop handles this
    drop(timer_lock_guard);

    if Inserted == 0 {
        ObDereferenceObject(Timer as PVOID);
    }
}

unsafe extern "C" fn ExpTimerApcKernelRoutine(
    Apc: *mut _KAPC,
    NormalRoutine: *mut PKNORMAL_ROUTINE,
    NormalContext: *mut PVOID,
    SystemArgument1: *mut PVOID,
    SystemArguemnt2: *mut PVOID,
) {
    let Timer: PETIMER;
    let mut OldIrql: KIRQL = 0;
    let mut DerefsToDo: ULONG = 1;
    let Thread = PsGetCurrentThread() as *mut _ETHREAD;

    Timer = (Apc as usize - core::mem::offset_of!(_ETIMER, TimerApc)) as PETIMER;

    // [guard] KeAcquireSpinLock(addr_of_mut!((*Timer).Lock), &mut OldIrql);
    let timer_lock_guard = unsafe { ntkrnl_sys::guards::SpinLockGuard::acquire(
        core::ptr::addr_of_mut!((*Timer).Lock) as ntkrnl_sys::ke::PKSPIN_LOCK
    ) };

    // [guard] KeAcquireSpinLockAtDpcLevel(addr_of_mut!((*Thread).ActiveTimerListLock));
    let active_list_lock_guard = unsafe { ntkrnl_sys::guards::SpinLockAtDpcGuard::acquire(
        core::ptr::addr_of_mut!((*Thread).ActiveTimerListLock) as ntkrnl_sys::ke::PKSPIN_LOCK
    ) };

    if (*Timer).ApcAssociated != 0
        && core::ptr::addr_of_mut!((*Thread).Tcb) == (*Timer).TimerApc.Thread
    {
        if (*Timer).Period == 0 {
            RemoveEntryList(core::ptr::addr_of_mut!((*Timer).ActiveTimerListEntry));

            (*Timer).ApcAssociated = 0;
            DerefsToDo += 1;
        }
    } else {
        *NormalRoutine = None;
    }

    // [guard-drop] KeReleaseSpinLockFromDpcLevel — guard Drop handles this
    drop(active_list_lock_guard);
    // [guard-drop] KeReleaseSpinLock — guard Drop handles this
    drop(timer_lock_guard);

    ObDereferenceObjectEx(Timer as PVOID, DerefsToDo as LONG);
}

// CODE_SEG("INIT")
#[no_mangle]
pub unsafe extern "system" fn ExpInitializeTimerImplementation() -> BOOLEAN {
    let mut ObjectTypeInitializer: OBJECT_TYPE_INITIALIZER = core::mem::zeroed();
    let mut Name: ntkrnl_sys::ddk::_UNICODE_STRING = core::mem::zeroed();
    let Status: NTSTATUS;

    core::ptr::write_bytes(
        &mut ObjectTypeInitializer as *mut OBJECT_TYPE_INITIALIZER as *mut u8,
        0,
        core::mem::size_of::<OBJECT_TYPE_INITIALIZER>(),
    );
    RtlInitUnicodeString(
        &mut Name,
        [
            'T' as u16, 'i' as u16, 'm' as u16, 'e' as u16, 'r' as u16, 0,
        ]
        .as_ptr(),
    );
    ObjectTypeInitializer.Length = core::mem::size_of::<OBJECT_TYPE_INITIALIZER>() as USHORT;
    ObjectTypeInitializer.InvalidAttributes = OBJ_OPENLINK;
    ObjectTypeInitializer.DefaultNonPagedPoolCharge = core::mem::size_of::<ETIMER>() as ULONG;
    ObjectTypeInitializer.GenericMapping = ExpTimerMapping;
    ObjectTypeInitializer.PoolType = NonPagedPool as u32;
    ObjectTypeInitializer.ValidAccessMask = TIMER_ALL_ACCESS;
    ObjectTypeInitializer.DeleteProcedure = Some(ExpDeleteTimer);
    Status = ObCreateObjectType(&mut Name, &mut ObjectTypeInitializer, core::ptr::null_mut(), &raw mut ExTimerType);
    if Status < 0 {
        return 0;
    }

    KeInitializeSpinLock(core::ptr::addr_of_mut!(ExpWakeListLock));
    InitializeListHead(core::ptr::addr_of_mut!(ExpWakeList));
    1
}

#[no_mangle]
pub unsafe extern "system" fn NtCancelTimer(
    TimerHandle: HANDLE,
    CurrentState: PBOOLEAN,
) -> NTSTATUS {
    let TimerHandle = opaque_ptr(TimerHandle);
    let CurrentState = opaque_ptr(CurrentState);
    let mut Timer: PETIMER = core::ptr::null_mut();
    let PreviousMode = ExGetPreviousMode();
    let mut State: BOOLEAN;
    let mut OldIrql: KIRQL = 0;
    let mut TimerThread: PETHREAD;
    let mut DerefsToDo: ULONG = 1;
    let mut Status: NTSTATUS;

    if !CurrentState.is_null() && PreviousMode != 0i8 {
        if let Err(e) = probe_user_buffer_write(
            CurrentState as *const core::ffi::c_void,
            core::mem::size_of::<BOOLEAN>(),
            core::mem::align_of::<BOOLEAN>(),
            PreviousMode,
        ) {
            return e;
        }
    }

    Status = ObReferenceObjectByHandle(
        TimerHandle as PVOID,
        TIMER_MODIFY_STATE,
        ExTimerType,
        PreviousMode,
        &mut Timer as *mut PETIMER as *mut PVOID,
        core::ptr::null_mut(),
    );
    if Status >= 0 {
        KeAcquireSpinLock(core::ptr::addr_of_mut!((*Timer).Lock) as *mut usize, &mut OldIrql);

        if (*Timer).ApcAssociated != 0 {
            TimerThread = ((*Timer).TimerApc.Thread as usize - core::mem::offset_of!(_ETHREAD, Tcb))
                as PETHREAD;

            KeAcquireSpinLockAtDpcLevel(core::ptr::addr_of_mut!((*TimerThread).ActiveTimerListLock) as *mut usize);

            RemoveEntryList(core::ptr::addr_of_mut!((*Timer).ActiveTimerListEntry));
            (*Timer).ApcAssociated = 0;

            KeReleaseSpinLockFromDpcLevel(core::ptr::addr_of_mut!(
                (*TimerThread).ActiveTimerListLock
            ) as *mut usize);

            KeCancelTimer(core::ptr::addr_of_mut!((*Timer).KeTimer));
            KeRemoveQueueDpc(core::ptr::addr_of_mut!((*Timer).TimerDpc));
            if KeRemoveQueueApc(core::ptr::addr_of_mut!((*Timer).TimerApc)) != 0 {
                DerefsToDo += 1;
            }
            DerefsToDo += 1;
        } else {
            KeCancelTimer(core::ptr::addr_of_mut!((*Timer).KeTimer));
        }

        if !(*Timer).WakeTimerListEntry.Flink.is_null() {
            KeAcquireSpinLockAtDpcLevel(core::ptr::addr_of_mut!(ExpWakeListLock) as *mut usize);

            if !(*Timer).WakeTimerListEntry.Flink.is_null() {
                RemoveEntryList(core::ptr::addr_of_mut!((*Timer).WakeTimerListEntry));
                (*Timer).WakeTimerListEntry.Flink = core::ptr::null_mut();
            }

            KeReleaseSpinLockFromDpcLevel(core::ptr::addr_of_mut!(ExpWakeListLock) as *mut usize);
        }

        KeReleaseSpinLock(core::ptr::addr_of_mut!((*Timer).Lock) as *mut usize, OldIrql);

        State = KeReadStateTimer(core::ptr::addr_of_mut!((*Timer).KeTimer));

        ObDereferenceObjectEx(Timer as PVOID, DerefsToDo as LONG);

        if !CurrentState.is_null() {
            write_user_nofail(CurrentState, State, PreviousMode);
        }
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn NtCreateTimer(
    TimerHandle: PHANDLE,
    DesiredAccess: ACCESS_MASK,
    ObjectAttributes: POBJECT_ATTRIBUTES,
    TimerType: u32,
) -> NTSTATUS {
    let TimerHandle = opaque_ptr(TimerHandle);
    let ObjectAttributes = opaque_ptr(ObjectAttributes);
    let mut Timer: PETIMER = core::ptr::null_mut();
    let mut hTimer: HANDLE = core::ptr::null_mut();
    let PreviousMode = ExGetPreviousMode();
    let mut Status: NTSTATUS;

    if TimerType != TIMER_TYPE_NotificationTimer && TimerType != TIMER_TYPE_SynchronizationTimer {
        return STATUS_INVALID_PARAMETER_4;
    }

    if PreviousMode != 0i8 {
        if let Err(e) = probe_user_buffer_write(
            TimerHandle as *const core::ffi::c_void,
            core::mem::size_of::<HANDLE>(),
            core::mem::align_of::<HANDLE>(),
            PreviousMode,
        ) {
            return e;
        }
    }

    Status = ObCreateObject(
        PreviousMode,
        ExTimerType,
        ObjectAttributes,
        PreviousMode,
        core::ptr::null_mut(),
        core::mem::size_of::<ETIMER>() as ULONG,
        0,
        0,
        &mut Timer as *mut PETIMER as *mut PVOID,
    );
    if Status >= 0 {
        KeInitializeDpc(
            core::ptr::addr_of_mut!((*Timer).TimerDpc),
            Some(ExpTimerDpcRoutine),
            Timer as PVOID,
        );

        KeInitializeTimerEx(
            core::ptr::addr_of_mut!((*Timer).KeTimer),
            TimerType,
        );

        KeInitializeSpinLock(core::ptr::addr_of_mut!((*Timer).Lock));
        (*Timer).ApcAssociated = 0;
        (*Timer).WakeTimer = 0;
        (*Timer).WakeTimerListEntry.Flink = core::ptr::null_mut();

        Status = ObInsertObject(
            Timer as PVOID,
            core::ptr::null_mut(),
            DesiredAccess,
            0,
            core::ptr::null_mut(),
            &mut hTimer,
        );

        if Status >= 0 {
            write_user_nofail(TimerHandle, hTimer, PreviousMode);
        }
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn NtOpenTimer(
    TimerHandle: PHANDLE,
    DesiredAccess: ACCESS_MASK,
    ObjectAttributes: POBJECT_ATTRIBUTES,
) -> NTSTATUS {
    let TimerHandle = opaque_ptr(TimerHandle);
    let ObjectAttributes = opaque_ptr(ObjectAttributes);
    let mut hTimer: HANDLE = core::ptr::null_mut();
    let PreviousMode = ExGetPreviousMode();
    let mut Status: NTSTATUS;

    if PreviousMode != 0i8 {
        if let Err(e) = probe_user_buffer_write(
            TimerHandle as *const core::ffi::c_void,
            core::mem::size_of::<HANDLE>(),
            core::mem::align_of::<HANDLE>(),
            PreviousMode,
        ) {
            return e;
        }
    }

    Status = ObOpenObjectByName(
        ObjectAttributes,
        ExTimerType,
        PreviousMode,
        core::ptr::null_mut(),
        DesiredAccess,
        core::ptr::null_mut(),
        &mut hTimer,
    );
    if Status >= 0 {
        write_user_nofail(TimerHandle, hTimer, PreviousMode);
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn NtQueryTimer(
    TimerHandle: HANDLE,
    TimerInformationClass: u32,
    TimerInformation: PVOID,
    TimerInformationLength: ULONG,
    ReturnLength: *mut ULONG,
) -> NTSTATUS {
    let TimerHandle = opaque_ptr(TimerHandle);
    let TimerInformation = opaque_ptr(TimerInformation);
    let ReturnLength = opaque_ptr(ReturnLength);
    let mut Timer: PETIMER = core::ptr::null_mut();
    let PreviousMode = ExGetPreviousMode();
    let mut Status: NTSTATUS;
    let BasicInfo = TimerInformation as PTIMER_BASIC_INFORMATION;

    Status = DefaultQueryInfoBufferCheck(
        TimerInformationClass,
        ExTimerInfoClass.as_ptr(),
        (core::mem::size_of_val(&ExTimerInfoClass) / core::mem::size_of::<INFORMATION_CLASS_INFO>())
            as ULONG,
        ICIF_PROBE_READ_WRITE,
        TimerInformation,
        TimerInformationLength,
        ReturnLength,
        core::ptr::null_mut(),
        PreviousMode,
    );
    if Status < 0 {
        return Status;
    }

    Status = ObReferenceObjectByHandle(
        TimerHandle as PVOID,
        TIMER_QUERY_STATE,
        ExTimerType,
        PreviousMode,
        &mut Timer as *mut PETIMER as *mut PVOID,
        core::ptr::null_mut(),
    );
    if Status >= 0 {
        let result = (|| -> Result<(), NTSTATUS> {
            let remaining_time = (*Timer).KeTimer.DueTime.QuadPart as i64
                - KeQueryInterruptTime() as i64;
            write_user(
                core::ptr::addr_of_mut!((*BasicInfo).TimeRemaining),
                LARGE_INTEGER {
                    QuadPart: remaining_time,
                },
                PreviousMode,
            )?;

            write_user(
                core::ptr::addr_of_mut!((*BasicInfo).SignalState),
                KeReadStateTimer(core::ptr::addr_of_mut!((*Timer).KeTimer)),
                PreviousMode,
            )?;

            if !ReturnLength.is_null() {
                write_user(
                    ReturnLength,
                    core::mem::size_of::<TIMER_BASIC_INFORMATION>() as ULONG,
                    PreviousMode,
                )?;
            }
            Ok(())
        })();

        if let Err(e) = result {
            Status = e;
        }

        ObDereferenceObject(Timer as PVOID);
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn NtSetTimer(
    TimerHandle: HANDLE,
    DueTime: PLARGE_INTEGER,
    TimerApcRoutine: PTIMER_APC_ROUTINE,
    TimerContext: PVOID,
    WakeTimer: BOOLEAN,
    Period: LONG,
    PreviousState: PBOOLEAN,
) -> NTSTATUS {
    let TimerHandle = opaque_ptr(TimerHandle);
    let DueTime = opaque_ptr(DueTime);
    let TimerContext = opaque_ptr(TimerContext);
    let PreviousState = opaque_ptr(PreviousState);
    let mut Timer: PETIMER = core::ptr::null_mut();
    let mut OldIrql: KIRQL = 0;
    let mut State: BOOLEAN;
    let PreviousMode = ExGetPreviousMode();
    let Thread = PsGetCurrentThread() as *mut _ETHREAD;
    let mut TimerDueTime: LARGE_INTEGER = unsafe { core::mem::zeroed() };
    let mut TimerThread: PETHREAD;
    let mut DerefsToDo: ULONG = 1;
    let mut Status: NTSTATUS = STATUS_SUCCESS;

    if Period < 0 {
        return STATUS_INVALID_PARAMETER_6;
    }

    if PreviousMode != 0i8 {
        let result = (|| -> Result<(), NTSTATUS> {
            TimerDueTime = probe_read_user::<LARGE_INTEGER>(DueTime, PreviousMode)?;

            if !PreviousState.is_null() {
                probe_user_buffer_write(
                    PreviousState as *const core::ffi::c_void,
                    core::mem::size_of::<BOOLEAN>(),
                    core::mem::align_of::<BOOLEAN>(),
                    PreviousMode,
                )?;
            }
            Ok(())
        })();

        if let Err(e) = result {
            return e;
        }
    } else {
        TimerDueTime = match ntkrnl_sys::probe::read_back(DueTime) { Ok(v) => v, Err(s) => return s };
    }

    Status = ObReferenceObjectByHandle(
        TimerHandle as PVOID,
        TIMER_MODIFY_STATE,
        ExTimerType,
        PreviousMode,
        &mut Timer as *mut PETIMER as *mut PVOID,
        core::ptr::null_mut(),
    );

    if Status >= 0 && WakeTimer != 0 {
        Status = STATUS_TIMER_RESUME_IGNORED;
    }

    if Status >= 0 {
        // [guard] KeAcquireSpinLock(addr_of_mut!((*Timer).Lock), &mut OldIrql);
        let timer_lock_guard = unsafe { ntkrnl_sys::guards::SpinLockGuard::acquire(
            core::ptr::addr_of_mut!((*Timer).Lock) as ntkrnl_sys::ke::PKSPIN_LOCK
        ) };

        if (*Timer).ApcAssociated != 0 {
            TimerThread = ((*Timer).TimerApc.Thread as usize - core::mem::offset_of!(_ETHREAD, Tcb))
                as PETHREAD;

            // [guard] KeAcquireSpinLockAtDpcLevel(addr_of_mut!((*TimerThread).ActiveTimerListLock));
            let thread_active_lock_guard = unsafe { ntkrnl_sys::guards::SpinLockAtDpcGuard::acquire(
                core::ptr::addr_of_mut!((*TimerThread).ActiveTimerListLock) as ntkrnl_sys::ke::PKSPIN_LOCK
            ) };

            RemoveEntryList(core::ptr::addr_of_mut!((*Timer).ActiveTimerListEntry));
            (*Timer).ApcAssociated = 0;

            // [guard-drop] KeReleaseSpinLockFromDpcLevel — guard Drop handles this
            drop(thread_active_lock_guard);

            KeCancelTimer(core::ptr::addr_of_mut!((*Timer).KeTimer));
            KeRemoveQueueDpc(core::ptr::addr_of_mut!((*Timer).TimerDpc));
            if KeRemoveQueueApc(core::ptr::addr_of_mut!((*Timer).TimerApc)) != 0 {
                DerefsToDo += 1;
            }
            DerefsToDo += 1;
        } else {
            KeCancelTimer(core::ptr::addr_of_mut!((*Timer).KeTimer));
        }

        State = KeReadStateTimer(core::ptr::addr_of_mut!((*Timer).KeTimer));

        (*Timer).WakeTimer = WakeTimer;
        // [guard] KeAcquireSpinLockAtDpcLevel(addr_of_mut!(ExpWakeListLock));
        let wake_list_lock_guard = unsafe { ntkrnl_sys::guards::SpinLockAtDpcGuard::acquire(
            core::ptr::addr_of_mut!(ExpWakeListLock) as *mut _ as ntkrnl_sys::ke::PKSPIN_LOCK
        ) };
        if WakeTimer != 0 && (*Timer).WakeTimerListEntry.Flink.is_null() {
            InsertTailList(
                core::ptr::addr_of_mut!(ExpWakeList),
                core::ptr::addr_of_mut!((*Timer).WakeTimerListEntry),
            );
        } else if WakeTimer == 0 && !(*Timer).WakeTimerListEntry.Flink.is_null() {
            RemoveEntryList(core::ptr::addr_of_mut!((*Timer).WakeTimerListEntry));
            (*Timer).WakeTimerListEntry.Flink = core::ptr::null_mut();
        }
        // [guard-drop] KeReleaseSpinLockFromDpcLevel — guard Drop handles this
        drop(wake_list_lock_guard);

        (*Timer).Period = Period;
        if TimerApcRoutine.is_some() {
            KeInitializeApc(
                core::ptr::addr_of_mut!((*Timer).TimerApc),
                core::ptr::addr_of_mut!((*Thread).Tcb),
                CurrentApcEnvironment,
                ExpTimerApcKernelRoutine,
                core::ptr::null_mut(),
                core::mem::transmute::<PTIMER_APC_ROUTINE, PKNORMAL_ROUTINE>(TimerApcRoutine),
                PreviousMode,
                TimerContext,
            );

            // [guard] KeAcquireSpinLockAtDpcLevel(addr_of_mut!((*Thread).ActiveTimerListLock));
            let thread_active_lock_guard = unsafe { ntkrnl_sys::guards::SpinLockAtDpcGuard::acquire(
                core::ptr::addr_of_mut!((*Thread).ActiveTimerListLock) as ntkrnl_sys::ke::PKSPIN_LOCK
            ) };
            InsertTailList(
                core::ptr::addr_of_mut!((*Thread).ActiveTimerListHead),
                core::ptr::addr_of_mut!((*Timer).ActiveTimerListEntry),
            );
            (*Timer).ApcAssociated = 1;
            // [guard-drop] KeReleaseSpinLockFromDpcLevel — guard Drop handles this
            drop(thread_active_lock_guard);

            DerefsToDo -= 1;
        }

        KeSetTimerEx(
            core::ptr::addr_of_mut!((*Timer).KeTimer),
            TimerDueTime,
            Period,
            if TimerApcRoutine.is_some() {
                core::ptr::addr_of_mut!((*Timer).TimerDpc)
            } else {
                core::ptr::null_mut()
            },
        );

        // [guard-drop] KeReleaseSpinLock — guard Drop handles this
        drop(timer_lock_guard);

        if DerefsToDo != 0 {
            ObDereferenceObjectEx(Timer as PVOID, DerefsToDo as LONG);
        }

        if !PreviousState.is_null() {
            write_user_nofail(PreviousState, State, PreviousMode);
        }
    }

    Status
}

#[macro_export]
macro_rules! offset_of {
    ($type:ty, $field:ident) => {{
        let dummy = core::mem::MaybeUninit::<$type>::uninit();
        let dummy_ptr = dummy.as_ptr();
        let field_ptr = unsafe { core::ptr::addr_of!((*dummy_ptr).$field) };
        (field_ptr as usize) - (dummy_ptr as usize)
    }};
}

// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         ReactOS Kernel
//  * LICENSE:         GPL - See COPYING in the top level directory
//  * FILE:            ntoskrnl/ex/timer.c
//  * PURPOSE:         Executive Timer Implementation
//  * PROGRAMMERS:     Alex Ionescu (alex.ionescu@reactos.org)
//  */
//
// /* INCLUDES ******************************************************************/
//
// #include <ntoskrnl.h>
// #define NDEBUG
// #include <debug.h>
//
// /* GLOBALS *******************************************************************/
//
// /* Timer Object Type */
// POBJECT_TYPE ExTimerType = NULL;
//
// KSPIN_LOCK ExpWakeListLock;
// LIST_ENTRY ExpWakeList;
//
// /* Timer Mapping */
// static _GENERIC_MAPPING ExpTimerMapping =
// {
//     STANDARD_RIGHTS_READ    | TIMER_QUERY_STATE,
//     STANDARD_RIGHTS_WRITE   | TIMER_MODIFY_STATE,
//     STANDARD_RIGHTS_EXECUTE | SYNCHRONIZE,
//     TIMER_ALL_ACCESS
// };
//
// /* Timer Information Classes */
// static const INFORMATION_CLASS_INFO ExTimerInfoClass[] =
// {
//     /* TimerBasicInformation */
//     IQS_SAME(TIMER_BASIC_INFORMATION, ULONG, ICIF_QUERY),
// };
//
// /* PRIVATE FUNCTIONS *********************************************************/
//
// VOID
// NTAPI
// ExTimerRundown(VOID)
// {
//     PETHREAD Thread = PsGetCurrentThread();
//     KIRQL OldIrql;
//     PLIST_ENTRY CurrentEntry;
//     PETIMER Timer;
//     ULONG DerefsToDo;
//
//     /* Lock the Thread's Active Timer List and loop it */
//     KeAcquireSpinLock(&Thread->ActiveTimerListLock, &OldIrql);
//     CurrentEntry = Thread->ActiveTimerListHead.Flink;
//     while (CurrentEntry != &Thread->ActiveTimerListHead)
//     {
//         /* Get the timer */
//         Timer = CONTAINING_RECORD(CurrentEntry, ETIMER, ActiveTimerListEntry);
//
//         /* Reference it */
//         ObReferenceObject(Timer);
//         DerefsToDo = 1;
//
//         /* Unlock the list */
//         KeReleaseSpinLock(&Thread->ActiveTimerListLock, OldIrql);
//
//         /* Lock the Timer */
//         KeAcquireSpinLock(&Timer->Lock, &OldIrql);
//
//         /* Lock the list again */
//         KeAcquireSpinLockAtDpcLevel(&Thread->ActiveTimerListLock);
//
//         /* Make sure that the timer is valid */
//         if ((Timer->ApcAssociated) && (&Thread->Tcb == Timer->TimerApc.Thread))
//         {
//             /* Remove it from the list */
//             RemoveEntryList(&Timer->ActiveTimerListEntry);
//             Timer->ApcAssociated = FALSE;
//
//             /* Cancel the timer and remove its DPC and APC */
//             KeCancelTimer(&Timer->KeTimer);
//             KeRemoveQueueDpc(&Timer->TimerDpc);
//             if (KeRemoveQueueApc(&Timer->TimerApc)) DerefsToDo++;
//
//             /* Add another dereference to do */
//             DerefsToDo++;
//         }
//
//         /* Unlock the list */
//         KeReleaseSpinLockFromDpcLevel(&Thread->ActiveTimerListLock);
//
//         /* Unlock the Timer */
//         KeReleaseSpinLock(&Timer->Lock, OldIrql);
//
//         /* Dereference it */
//         ObDereferenceObjectEx(Timer, DerefsToDo);
//
//         /* Loop again */
//         KeAcquireSpinLock(&Thread->ActiveTimerListLock, &OldIrql);
//         CurrentEntry = Thread->ActiveTimerListHead.Flink;
//     }
//
//     /* Release lock and return */
//     KeReleaseSpinLock(&Thread->ActiveTimerListLock, OldIrql);
// }
//
// VOID
// NTAPI
// ExpDeleteTimer(IN PVOID ObjectBody)
// {
//     KIRQL OldIrql;
//     PETIMER Timer = ObjectBody;
//
//     /* Check if it has a Wait List */
//     if (Timer->WakeTimerListEntry.Flink)
//     {
//         /* Lock the Wake List */
//         KeAcquireSpinLock(&ExpWakeListLock, &OldIrql);
//
//         /* Check again, since it might've changed before we locked */
//         if (Timer->WakeTimerListEntry.Flink)
//         {
//             /* Remove it from the Wait List */
//             RemoveEntryList(&Timer->WakeTimerListEntry);
//             Timer->WakeTimerListEntry.Flink = NULL;
//         }
//
//         /* Release the Wake List */
//         KeReleaseSpinLock(&ExpWakeListLock, OldIrql);
//     }
//
//     /* Tell the Kernel to cancel the Timer and flush all queued DPCs */
//     KeCancelTimer(&Timer->KeTimer);
//     KeFlushQueuedDpcs();
// }
//
// _Function_class_(KDEFERRED_ROUTINE)
// VOID
// NTAPI
// ExpTimerDpcRoutine(IN PKDPC Dpc,
//                    IN PVOID DeferredContext,
//                    IN PVOID SystemArgument1,
//                    IN PVOID SystemArgument2)
// {
//     PETIMER Timer = DeferredContext;
//     BOOLEAN Inserted = FALSE;
//
//     /* Reference the timer */
//     if (!ObReferenceObjectSafe(Timer)) return;
//
//     /* Lock the Timer */
//     KeAcquireSpinLockAtDpcLevel(&Timer->Lock);
//
//     /* Check if the timer is associated */
//     if (Timer->ApcAssociated)
//     {
//         /* Queue the APC */
//         Inserted = KeInsertQueueApc(&Timer->TimerApc,
//                                     SystemArgument1,
//                                     SystemArgument2,
//                                     IO_NO_INCREMENT);
//     }
//
//     /* Release the Timer */
//     KeReleaseSpinLockFromDpcLevel(&Timer->Lock);
//
//     /* Dereference it if we couldn't queue the APC */
//     if (!Inserted) ObDereferenceObject(Timer);
// }
//
// VOID
// NTAPI
// ExpTimerApcKernelRoutine(IN PKAPC Apc,
//                          IN OUT PKNORMAL_ROUTINE* NormalRoutine,
//                          IN OUT PVOID* NormalContext,
//                          IN OUT PVOID* SystemArgument1,
//                          IN OUT PVOID* SystemArguemnt2)
// {
//     PETIMER Timer;
//     KIRQL OldIrql;
//     ULONG DerefsToDo = 1;
//     PETHREAD Thread = PsGetCurrentThread();
//
//     /* We need to find out which Timer we are */
//     Timer = CONTAINING_RECORD(Apc, ETIMER, TimerApc);
//
//     /* Lock the Timer */
//     KeAcquireSpinLock(&Timer->Lock, &OldIrql);
//
//     /* Lock the Thread's Active Timer List*/
//     KeAcquireSpinLockAtDpcLevel(&Thread->ActiveTimerListLock);
//
//     /* Make sure that the Timer is valid, and that it belongs to this thread */
//     if ((Timer->ApcAssociated) && (&Thread->Tcb == Timer->TimerApc.Thread))
//     {
//         /* Check if it's not periodic */
//         if (!Timer->Period)
//         {
//             /* Remove it from the Active Timers List */
//             RemoveEntryList(&Timer->ActiveTimerListEntry);
//
//             /* Disable it */
//             Timer->ApcAssociated = FALSE;
//             DerefsToDo++;
//         }
//     }
//     else
//     {
//         /* Clear the normal routine */
//         *NormalRoutine = NULL;
//     }
//
//     /* Release locks */
//     KeReleaseSpinLockFromDpcLevel(&Thread->ActiveTimerListLock);
//     KeReleaseSpinLock(&Timer->Lock, OldIrql);
//
//     /* Dereference as needed */
//     ObDereferenceObjectEx(Timer, DerefsToDo);
// }
//
// CODE_SEG("INIT")
// BOOLEAN
// NTAPI
// ExpInitializeTimerImplementation(VOID)
// {
//     OBJECT_TYPE_INITIALIZER ObjectTypeInitializer;
//     _UNICODE_STRING Name;
//     NTSTATUS Status;
//
//     /* Create the Timer Object Type */
//     RtlZeroMemory(&ObjectTypeInitializer, sizeof(ObjectTypeInitializer));
//     RtlInitUnicodeString(&Name, L"Timer");
//     ObjectTypeInitializer.Length = sizeof(ObjectTypeInitializer);
//     ObjectTypeInitializer.InvalidAttributes = OBJ_OPENLINK;
//     ObjectTypeInitializer.DefaultNonPagedPoolCharge = sizeof(ETIMER);
//     ObjectTypeInitializer.GenericMapping = ExpTimerMapping;
//     ObjectTypeInitializer.PoolType = NonPagedPool;
//     ObjectTypeInitializer.ValidAccessMask = TIMER_ALL_ACCESS;
//     ObjectTypeInitializer.DeleteProcedure = ExpDeleteTimer;
//     Status = ObCreateObjectType(&Name, &ObjectTypeInitializer, NULL, &ExTimerType);
//     if (!NT_SUCCESS(Status)) return FALSE;
//
//     /* Initialize the Wait List and Lock */
//     KeInitializeSpinLock(&ExpWakeListLock);
//     InitializeListHead(&ExpWakeList);
//     return TRUE;
// }
//
// /* PUBLIC FUNCTIONS **********************************************************/
//
// NTSTATUS
// NTAPI
// NtCancelTimer(IN HANDLE TimerHandle,
//               OUT PBOOLEAN CurrentState OPTIONAL)
// {
//     PETIMER Timer;
//     KPROCESSOR_MODE PreviousMode = ExGetPreviousMode();
//     BOOLEAN State;
//     KIRQL OldIrql;
//     PETHREAD TimerThread;
//     ULONG DerefsToDo = 1;
//     NTSTATUS Status;
//     PAGED_CODE();
//
//     /* Check if we need to probe */
//     if ((CurrentState) && (PreviousMode != KernelMode))
//     {
//         _SEH2_TRY
//         {
//             /* Make sure the pointer is valid */
//             ProbeForWriteBoolean(CurrentState);
//         }
//         _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//         {
//             /* Return the exception code */
//             _SEH2_YIELD(return _SEH2_GetExceptionCode());
//         }
//         _SEH2_END;
//     }
//
//     /* Get the Timer Object */
//     Status = ObReferenceObjectByHandle(TimerHandle,
//                                        TIMER_MODIFY_STATE,
//                                        ExTimerType,
//                                        PreviousMode,
//                                        (PVOID*)&Timer,
//                                        NULL);
//     if (NT_SUCCESS(Status))
//     {
//         /* Lock the Timer */
//         KeAcquireSpinLock(&Timer->Lock, &OldIrql);
//
//         /* Check if it's enabled */
//         if (Timer->ApcAssociated)
//         {
//             /* Get the Thread. */
//             TimerThread = CONTAINING_RECORD(Timer->TimerApc.Thread,
//                                             _ETHREAD,
//                                             Tcb);
//
//             /* Lock its active list */
//             KeAcquireSpinLockAtDpcLevel(&TimerThread->ActiveTimerListLock);
//
//             /* Remove it */
//             RemoveEntryList(&Timer->ActiveTimerListEntry);
//             Timer->ApcAssociated = FALSE;
//
//             /* Unlock the list */
//             KeReleaseSpinLockFromDpcLevel(&TimerThread->ActiveTimerListLock);
//
//             /* Cancel the Timer */
//             KeCancelTimer(&Timer->KeTimer);
//             KeRemoveQueueDpc(&Timer->TimerDpc);
//             if (KeRemoveQueueApc(&Timer->TimerApc)) DerefsToDo++;
//             DerefsToDo++;
//         }
//         else
//         {
//             /* If timer was disabled, we still need to cancel it */
//             KeCancelTimer(&Timer->KeTimer);
//         }
//
//         /* Handle a Wake Timer */
//         if (Timer->WakeTimerListEntry.Flink)
//         {
//             /* Lock the Wake List */
//             KeAcquireSpinLockAtDpcLevel(&ExpWakeListLock);
//
//             /* Check again, since it might've changed before we locked */
//             if (Timer->WakeTimerListEntry.Flink)
//             {
//                 /* Remove it from the Wait List */
//                 RemoveEntryList(&Timer->WakeTimerListEntry);
//                 Timer->WakeTimerListEntry.Flink = NULL;
//             }
//
//             /* Release the Wake List */
//             KeReleaseSpinLockFromDpcLevel(&ExpWakeListLock);
//         }
//
//         /* Unlock the Timer */
//         KeReleaseSpinLock(&Timer->Lock, OldIrql);
//
//         /* Read the old State */
//         State = KeReadStateTimer(&Timer->KeTimer);
//
//         /* Dereference the Object */
//         ObDereferenceObjectEx(Timer, DerefsToDo);
//
//         /* Check if caller wants the state */
//         if (CurrentState)
//         {
//             _SEH2_TRY
//             {
//                 /* Return the Timer State */
//                 *CurrentState = State;
//             }
//             _SEH2_EXCEPT(ExSystemExceptionFilter())
//             {
//                 /* Do nothing */
//                 (void)0;
//             }
//             _SEH2_END;
//         }
//     }
//
//     /* Return to Caller */
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// NtCreateTimer(OUT PHANDLE TimerHandle,
//               IN ACCESS_MASK DesiredAccess,
//               IN POBJECT_ATTRIBUTES ObjectAttributes OPTIONAL,
//               IN TIMER_TYPE TimerType)
// {
//     PETIMER Timer;
//     HANDLE hTimer;
//     KPROCESSOR_MODE PreviousMode = ExGetPreviousMode();
//     NTSTATUS Status;
//     PAGED_CODE();
//
//     /* Check for correct timer type */
//     if ((TimerType != NotificationTimer) &&
//         (TimerType != SynchronizationTimer))
//     {
//         /* Fail */
//         return STATUS_INVALID_PARAMETER_4;
//     }
//
//     /* Check if we need to probe */
//     if (PreviousMode != KernelMode)
//     {
//         _SEH2_TRY
//         {
//             /* Make sure the pointer is valid */
//             ProbeForWriteHandle(TimerHandle);
//         }
//         _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//         {
//             /* Return the exception code */
//             _SEH2_YIELD(return _SEH2_GetExceptionCode());
//         }
//         _SEH2_END;
//     }
//
//     /* Create the Object */
//     Status = ObCreateObject(PreviousMode,
//                             ExTimerType,
//                             ObjectAttributes,
//                             PreviousMode,
//                             NULL,
//                             sizeof(ETIMER),
//                             0,
//                             0,
//                             (PVOID*)&Timer);
//     if (NT_SUCCESS(Status))
//     {
//         /* Initialize the DPC */
//         KeInitializeDpc(&Timer->TimerDpc, ExpTimerDpcRoutine, Timer);
//
//         /* Initialize the Kernel Timer */
//         KeInitializeTimerEx(&Timer->KeTimer, TimerType);
//
//         /* Initialize the timer fields */
//         KeInitializeSpinLock(&Timer->Lock);
//         Timer->ApcAssociated = FALSE;
//         Timer->WakeTimer = FALSE;
//         Timer->WakeTimerListEntry.Flink = NULL;
//
//         /* Insert the Timer */
//         Status = ObInsertObject((PVOID)Timer,
//                                 NULL,
//                                 DesiredAccess,
//                                 0,
//                                 NULL,
//                                 &hTimer);
//
//         /* Check for success */
//         if (NT_SUCCESS(Status))
//         {
//             /* Enter SEH */
//             _SEH2_TRY
//             {
//                 /* Return the Timer Handle */
//                 *TimerHandle = hTimer;
//             }
//             _SEH2_EXCEPT(ExSystemExceptionFilter())
//             {
//                 /* Do nothing */
//                 (void)0;
//             }
//             _SEH2_END;
//         }
//     }
//
//     /* Return to Caller */
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// NtOpenTimer(OUT PHANDLE TimerHandle,
//             IN ACCESS_MASK DesiredAccess,
//             IN POBJECT_ATTRIBUTES ObjectAttributes)
// {
//     HANDLE hTimer;
//     KPROCESSOR_MODE PreviousMode = ExGetPreviousMode();
//     NTSTATUS Status;
//     PAGED_CODE();
//
//     /* Check Parameter Validity */
//     if (PreviousMode != KernelMode)
//     {
//         _SEH2_TRY
//         {
//             /* Make sure the pointer is valid */
//             ProbeForWriteHandle(TimerHandle);
//         }
//         _SEH2_EXCEPT(ExSystemExceptionFilter())
//         {
//             /* Return the exception code */
//             _SEH2_YIELD(return _SEH2_GetExceptionCode());
//         }
//         _SEH2_END;
//     }
//
//     /* Open the Timer */
//     Status = ObOpenObjectByName(ObjectAttributes,
//                                 ExTimerType,
//                                 PreviousMode,
//                                 NULL,
//                                 DesiredAccess,
//                                 NULL,
//                                 &hTimer);
//     if (NT_SUCCESS(Status))
//     {
//         /* Enter SEH */
//         _SEH2_TRY
//         {
//             /* Return the Timer Handle */
//             *TimerHandle = hTimer;
//         }
//         _SEH2_EXCEPT(ExSystemExceptionFilter())
//         {
//             /* Do nothing */
//             (void)0;
//         }
//         _SEH2_END;
//     }
//
//     /* Return to Caller */
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// NtQueryTimer(IN HANDLE TimerHandle,
//              IN TIMER_INFORMATION_CLASS TimerInformationClass,
//              OUT PVOID TimerInformation,
//              IN ULONG TimerInformationLength,
//              OUT PULONG ReturnLength OPTIONAL)
// {
//     PETIMER Timer;
//     KPROCESSOR_MODE PreviousMode = ExGetPreviousMode();
//     NTSTATUS Status;
//     PTIMER_BASIC_INFORMATION BasicInfo = TimerInformation;
//     PAGED_CODE();
//
//     /* Check Validity */
//     Status = DefaultQueryInfoBufferCheck(TimerInformationClass,
//                                          ExTimerInfoClass,
//                                          sizeof(ExTimerInfoClass) /
//                                          sizeof(ExTimerInfoClass[0]),
//                                          ICIF_PROBE_READ_WRITE,
//                                          TimerInformation,
//                                          TimerInformationLength,
//                                          ReturnLength,
//                                          NULL,
//                                          PreviousMode);
//     if (!NT_SUCCESS(Status)) return Status;
//
//     /* Get the Timer Object */
//     Status = ObReferenceObjectByHandle(TimerHandle,
//                                        TIMER_QUERY_STATE,
//                                        ExTimerType,
//                                        PreviousMode,
//                                        (PVOID*)&Timer,
//                                        NULL);
//     if (NT_SUCCESS(Status))
//     {
//         /* Return the Basic Information */
//         _SEH2_TRY
//         {
//             /* Return the remaining time, corrected */
//             BasicInfo->TimeRemaining.QuadPart = Timer->
//                                                 KeTimer.DueTime.QuadPart -
//                                                 KeQueryInterruptTime();
//
//             /* Return the current state */
//             BasicInfo->SignalState = KeReadStateTimer(&Timer->KeTimer);
//
//             /* Return the buffer length if requested */
//             if (ReturnLength) *ReturnLength = sizeof(TIMER_BASIC_INFORMATION);
//         }
//         _SEH2_EXCEPT(ExSystemExceptionFilter())
//         {
//             /* Get the exception code */
//             Status = _SEH2_GetExceptionCode();
//         }
//         _SEH2_END;
//
//         /* Dereference Object */
//         ObDereferenceObject(Timer);
//     }
//
//     /* Return Status */
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// NtSetTimer(IN HANDLE TimerHandle,
//            IN PLARGE_INTEGER DueTime,
//            IN PTIMER_APC_ROUTINE TimerApcRoutine OPTIONAL,
//            IN PVOID TimerContext OPTIONAL,
//            IN BOOLEAN WakeTimer,
//            IN LONG Period OPTIONAL,
//            OUT PBOOLEAN PreviousState OPTIONAL)
// {
//     PETIMER Timer;
//     KIRQL OldIrql;
//     BOOLEAN State;
//     KPROCESSOR_MODE PreviousMode = ExGetPreviousMode();
//     PETHREAD Thread = PsGetCurrentThread();
//     LARGE_INTEGER TimerDueTime;
//     PETHREAD TimerThread;
//     ULONG DerefsToDo = 1;
//     NTSTATUS Status = STATUS_SUCCESS;
//     PAGED_CODE();
//
//     /* Check for a valid Period */
//     if (Period < 0) return STATUS_INVALID_PARAMETER_6;
//
//     /* Check if we need to probe */
//     if (PreviousMode != KernelMode)
//     {
//         _SEH2_TRY
//         {
//             /* Probe and capture the due time */
//             TimerDueTime = ProbeForReadLargeInteger(DueTime);
//
//             /* Probe the state pointer if one was passed */
//             if (PreviousState) ProbeForWriteBoolean(PreviousState);
//         }
//         _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//         {
//             /* Return the exception code */
//             _SEH2_YIELD(return _SEH2_GetExceptionCode());
//         }
//         _SEH2_END;
//     }
//     else
//     {
//         /* Capture the time directly */
//         TimerDueTime = *DueTime;
//     }
//
//     /* Get the Timer Object */
//     Status = ObReferenceObjectByHandle(TimerHandle,
//                                        TIMER_MODIFY_STATE,
//                                        ExTimerType,
//                                        PreviousMode,
//                                        (PVOID*)&Timer,
//                                        NULL);
//
//     /*
//      * Tell the user we don't support Wake Timers...
//      * when we have the ability to use/detect the Power Management
//      * functionality required to support them, make this check dependent
//      * on the actual PM capabilities
//      */
//     if (NT_SUCCESS(Status) && WakeTimer)
//     {
//         Status = STATUS_TIMER_RESUME_IGNORED;
//     }
//
//     /* Check status */
//     if (NT_SUCCESS(Status))
//     {
//         /* Lock the Timer */
//         KeAcquireSpinLock(&Timer->Lock, &OldIrql);
//
//         /* Cancel Running Timer */
//         if (Timer->ApcAssociated)
//         {
//             /* Get the Thread. */
//             TimerThread = CONTAINING_RECORD(Timer->TimerApc.Thread,
//                                             _ETHREAD,
//                                             Tcb);
//
//             /* Lock its active list */
//             KeAcquireSpinLockAtDpcLevel(&TimerThread->ActiveTimerListLock);
//
//             /* Remove it */
//             RemoveEntryList(&Timer->ActiveTimerListEntry);
//             Timer->ApcAssociated = FALSE;
//
//             /* Unlock the list */
//             KeReleaseSpinLockFromDpcLevel(&TimerThread->ActiveTimerListLock);
//
//             /* Cancel the Timer */
//             KeCancelTimer(&Timer->KeTimer);
//             KeRemoveQueueDpc(&Timer->TimerDpc);
//             if (KeRemoveQueueApc(&Timer->TimerApc)) DerefsToDo++;
//             DerefsToDo++;
//         }
//         else
//         {
//             /* If timer was disabled, we still need to cancel it */
//             KeCancelTimer(&Timer->KeTimer);
//         }
//
//         /* Read the State */
//         State = KeReadStateTimer(&Timer->KeTimer);
//
//         /* Handle Wake Timers */
//         Timer->WakeTimer = WakeTimer;
//         KeAcquireSpinLockAtDpcLevel(&ExpWakeListLock);
//         if ((WakeTimer) && !(Timer->WakeTimerListEntry.Flink))
//         {
//             /* Insert it into the list */
//             InsertTailList(&ExpWakeList, &Timer->WakeTimerListEntry);
//         }
//         else if (!(WakeTimer) && (Timer->WakeTimerListEntry.Flink))
//         {
//             /* Remove it from the list */
//             RemoveEntryList(&Timer->WakeTimerListEntry);
//             Timer->WakeTimerListEntry.Flink = NULL;
//         }
//         KeReleaseSpinLockFromDpcLevel(&ExpWakeListLock);
//
//         /* Set up the APC Routine if specified */
//         Timer->Period = Period;
//         if (TimerApcRoutine)
//         {
//             /* Initialize the APC */
//             KeInitializeApc(&Timer->TimerApc,
//                             &Thread->Tcb,
//                             CurrentApcEnvironment,
//                             ExpTimerApcKernelRoutine,
//                             (PKRUNDOWN_ROUTINE)NULL,
//                             (PKNORMAL_ROUTINE)TimerApcRoutine,
//                             PreviousMode,
//                             TimerContext);
//
//             /* Lock the Thread's Active List and Insert */
//             KeAcquireSpinLockAtDpcLevel(&Thread->ActiveTimerListLock);
//             InsertTailList(&Thread->ActiveTimerListHead,
//                            &Timer->ActiveTimerListEntry);
//             Timer->ApcAssociated = TRUE;
//             KeReleaseSpinLockFromDpcLevel(&Thread->ActiveTimerListLock);
//
//             /* One less dereference to do */
//             DerefsToDo--;
//          }
//
//         /* Enable and Set the Timer */
//         KeSetTimerEx(&Timer->KeTimer,
//                      TimerDueTime,
//                      Period,
//                      TimerApcRoutine ? &Timer->TimerDpc : NULL);
//
//         /* Unlock the Timer */
//         KeReleaseSpinLock(&Timer->Lock, OldIrql);
//
//         /* Dereference if it was previously enabled */
//         if (DerefsToDo) ObDereferenceObjectEx(Timer, DerefsToDo);
//
//         /* Check if we need to return the State */
//         if (PreviousState)
//         {
//             /* Enter SEH */
//             _SEH2_TRY
//             {
//                 /* Return the Timer State */
//                 *PreviousState = State;
//             }
//             _SEH2_EXCEPT(ExSystemExceptionFilter())
//             {
//                 /* Do nothing */
//                 (void)0;
//             }
//             _SEH2_END;
//         }
//     }
//
//     /* Return to Caller */
//     return Status;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
