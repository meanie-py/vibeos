use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::ex::*;
use ntkrnl_sys::ob::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::mm::*;

use ntkrnl_sys::ddk::{
    _CALLBACK_OBJECT, _CALLBACK_REGISTRATION, _EX_CALLBACK, _EX_CALLBACK_ROUTINE_BLOCK,
    _EX_FAST_REF, _EX_PUSH_LOCK, _KEVENT, _OBJECT_ATTRIBUTES, _OBJECT_TYPE_INITIALIZER,
    _UNICODE_STRING, _GENERIC_MAPPING, HANDLE, ACCESS_MASK,
    PCALLBACK_OBJECT, PCALLBACK_FUNCTION, PEX_CALLBACK_FUNCTION,
    ExAcquireFastReference, ExCompareSwapFastReference, ExGetCountFastReference,
    ExGetObjectFastReference, ExInitializeFastReference, ExInitializeRundownProtection,
    ExInsertFastReference, ExReleaseFastReference, ExReleaseRundownProtection,
    ExWaitForRundownProtectionRelease, ExfAcquireRundownProtectionEx,
    ExfReleaseRundownProtectionEx, ExWaitOnPushLock, KeEnterCriticalRegion,
    KeInitializeEvent, KeInitializeSpinLock, KeLeaveCriticalRegion, KeWaitForSingleObject,
    NtClose, NtCreateDirectoryObject, ObCreateObject, ObCreateObjectType, ObInsertObject,
    ObOpenObjectByName, ObReferenceObjectByHandle, ZwClose,
    RtlInitUnicodeString,
    POBJECT_TYPE, PHANDLE,
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


extern "C" {
    fn CONTAINING_RECORD( address: PVOID, type_size: usize, field_offset: usize, ) -> PVOID;
    fn InitializeObjectAttributes( p: *mut _OBJECT_ATTRIBUTES, n: *mut _UNICODE_STRING, a: ULONG, r: HANDLE, s: PVOID, );
    static SePublicDefaultSd: PVOID;
}

const TAG_CALLBACK_ROUTINE_BLOCK: ULONG = 0x62726243; // 'cbCk'
const TAG_CALLBACK_REGISTRATION: ULONG = 0x65524243; // 'cbCr'

// ReactOS uses 7 for ALL architectures (3 low bits), matching C #define.
const MAX_FAST_REFS: ULONG = 7;

const CALLBACK_MODIFY_STATE: ACCESS_MASK = 0x0001;
const CALLBACK_ALL_ACCESS: ACCESS_MASK = STANDARD_RIGHTS_REQUIRED | SYNCHRONIZE | CALLBACK_MODIFY_STATE;

#[repr(C)]
struct SYSTEM_CALLBACKS {
    CallbackObject: *mut PCALLBACK_OBJECT,
    Name: *const u16,
}

static mut ExpCallbackMapping: _GENERIC_MAPPING = _GENERIC_MAPPING {
    GenericRead: STANDARD_RIGHTS_READ,
    GenericWrite: STANDARD_RIGHTS_WRITE | CALLBACK_MODIFY_STATE,
    GenericExecute: STANDARD_RIGHTS_EXECUTE | SYNCHRONIZE,
    GenericAll: CALLBACK_ALL_ACCESS,
};

#[no_mangle]
pub static mut SetSystemTimeCallback: PCALLBACK_OBJECT = core::ptr::null_mut();
#[no_mangle]
pub static mut SetSystemStateCallback: PCALLBACK_OBJECT = core::ptr::null_mut();
#[no_mangle]
pub static mut PowerStateCallback: PCALLBACK_OBJECT = core::ptr::null_mut();

// Wide string literals for callback names (UTF-16LE)
static CALLBACK_NAME_SET_SYSTEM_TIME: [u16; 24] = [
    b'\\' as u16, b'C' as u16, b'a' as u16, b'l' as u16, b'l' as u16, b'b' as u16, b'a' as u16,
    b'c' as u16, b'k' as u16, b'\\' as u16, b'S' as u16, b'e' as u16, b't' as u16, b'S' as u16,
    b'y' as u16, b's' as u16, b't' as u16, b'e' as u16, b'm' as u16, b'T' as u16, b'i' as u16,
    b'm' as u16, b'e' as u16, 0u16,
]; // L"\\Callback\\SetSystemTime"
static CALLBACK_NAME_SET_SYSTEM_STATE: [u16; 25] = [
    b'\\' as u16, b'C' as u16, b'a' as u16, b'l' as u16, b'l' as u16, b'b' as u16, b'a' as u16,
    b'c' as u16, b'k' as u16, b'\\' as u16, b'S' as u16, b'e' as u16, b't' as u16, b'S' as u16,
    b'y' as u16, b's' as u16, b't' as u16, b'e' as u16, b'm' as u16, b'S' as u16, b't' as u16,
    b'a' as u16, b't' as u16, b'e' as u16, 0u16,
]; // L"\\Callback\\SetSystemState"
static CALLBACK_NAME_POWER_STATE: [u16; 21] = [
    b'\\' as u16, b'C' as u16, b'a' as u16, b'l' as u16, b'l' as u16, b'b' as u16, b'a' as u16,
    b'c' as u16, b'k' as u16, b'\\' as u16, b'P' as u16, b'o' as u16, b'w' as u16, b'e' as u16,
    b'r' as u16, b'S' as u16, b't' as u16, b'a' as u16, b't' as u16, b'e' as u16, 0u16,
]; // L"\\Callback\\PowerState"

static mut ExpInitializeCallback: [SYSTEM_CALLBACKS; 4] = {
    [
        SYSTEM_CALLBACKS {
            CallbackObject: core::ptr::addr_of_mut!(SetSystemTimeCallback),
            Name: CALLBACK_NAME_SET_SYSTEM_TIME.as_ptr(),
        },
        SYSTEM_CALLBACKS {
            CallbackObject: core::ptr::addr_of_mut!(SetSystemStateCallback),
            Name: CALLBACK_NAME_SET_SYSTEM_STATE.as_ptr(),
        },
        SYSTEM_CALLBACKS {
            CallbackObject: core::ptr::addr_of_mut!(PowerStateCallback),
            Name: CALLBACK_NAME_POWER_STATE.as_ptr(),
        },
        SYSTEM_CALLBACKS {
            CallbackObject: core::ptr::null_mut(),
            Name: core::ptr::null(),
        },
    ]
};

#[no_mangle]
pub static mut ExCallbackObjectType: POBJECT_TYPE = core::ptr::null_mut();
static mut ExpCallbackEvent: _KEVENT = unsafe { core::mem::zeroed() };
static mut ExpCallBackFlush: _EX_PUSH_LOCK = unsafe { core::mem::zeroed() };

#[no_mangle]
pub unsafe extern "system" fn ExInitializeCallBack(Callback: *mut _EX_CALLBACK) {
    let Callback = opaque_ptr(Callback);
    ExInitializeFastReference(
        core::ptr::addr_of_mut!((*Callback).RoutineBlock),
        core::ptr::null_mut(),
    );
}

#[no_mangle]
pub unsafe extern "system" fn ExAllocateCallBack(
    Function: PEX_CALLBACK_FUNCTION,
    Context: PVOID,
) -> *mut _EX_CALLBACK_ROUTINE_BLOCK {
    let Context = opaque_ptr(Context);
    let CallbackBlock = ExAllocatePoolWithTag(
        PagedPool,
        core::mem::size_of::<_EX_CALLBACK_ROUTINE_BLOCK>(),
        TAG_CALLBACK_ROUTINE_BLOCK,
    ) as *mut _EX_CALLBACK_ROUTINE_BLOCK;

    if !CallbackBlock.is_null() {
        (*CallbackBlock).Function = Function;
        (*CallbackBlock).Context = Context;
        ExInitializeRundownProtection(core::ptr::addr_of_mut!(
            (*CallbackBlock).RundownProtect
        ));
    }

    CallbackBlock
}

#[no_mangle]
pub unsafe extern "system" fn ExFreeCallBack(CallbackBlock: *mut _EX_CALLBACK_ROUTINE_BLOCK) {
    let CallbackBlock = opaque_ptr(CallbackBlock);
    ExFreePoolWithTag(CallbackBlock as PVOID, TAG_CALLBACK_ROUTINE_BLOCK);
}

#[no_mangle]
pub unsafe extern "system" fn ExWaitForCallBacks(CallbackBlock: *mut _EX_CALLBACK_ROUTINE_BLOCK) {
    let CallbackBlock = opaque_ptr(CallbackBlock);
    ExWaitForRundownProtectionRelease(core::ptr::addr_of_mut!(
        (*CallbackBlock).RundownProtect
    ));
}

#[no_mangle]
pub unsafe extern "system" fn ExGetCallBackBlockRoutine(
    CallbackBlock: *mut _EX_CALLBACK_ROUTINE_BLOCK,
) -> PEX_CALLBACK_FUNCTION {
    let CallbackBlock = opaque_ptr(CallbackBlock);
    (*CallbackBlock).Function
}

#[no_mangle]
pub unsafe extern "system" fn ExGetCallBackBlockContext(
    CallbackBlock: *mut _EX_CALLBACK_ROUTINE_BLOCK,
) -> PVOID {
    let CallbackBlock = opaque_ptr(CallbackBlock);
    (*CallbackBlock).Context
}

#[no_mangle]
pub unsafe extern "system" fn ExDereferenceCallBackBlock(
    CallBack: *mut _EX_CALLBACK,
    CallbackBlock: *mut _EX_CALLBACK_ROUTINE_BLOCK,
) {
    let CallBack = opaque_ptr(CallBack);
    let CallbackBlock = opaque_ptr(CallbackBlock);
    if ExReleaseFastReference(
        core::ptr::addr_of_mut!((*CallBack).RoutineBlock),
        CallbackBlock as PVOID,
    ) == 0
    {
        ExReleaseRundownProtection(core::ptr::addr_of_mut!((*CallbackBlock).RundownProtect));
    }
}

#[no_mangle]
pub unsafe extern "system" fn ExReferenceCallBackBlock(
    CallBack: *mut _EX_CALLBACK,
) -> *mut _EX_CALLBACK_ROUTINE_BLOCK {
    let CallBack = opaque_ptr(CallBack);
    let OldValue = ExAcquireFastReference(core::ptr::addr_of_mut!((*CallBack).RoutineBlock));
    let Count = ExGetCountFastReference(OldValue);

    if ExGetObjectFastReference(OldValue).is_null() {
        return core::ptr::null_mut();
    }

    if Count == 0 {
        return core::ptr::null_mut();
    }

    let CallbackBlock = ExGetObjectFastReference(OldValue) as *mut _EX_CALLBACK_ROUTINE_BLOCK;

    if Count == 1 {
        if ExfAcquireRundownProtectionEx(
            core::ptr::addr_of_mut!((*CallbackBlock).RundownProtect),
            MAX_FAST_REFS,
        ) != 0
        {
            if ExInsertFastReference(
                core::ptr::addr_of_mut!((*CallBack).RoutineBlock),
                CallbackBlock as PVOID,
            ) == 0
            {
                ExfReleaseRundownProtectionEx(
                    core::ptr::addr_of_mut!((*CallbackBlock).RundownProtect),
                    MAX_FAST_REFS,
                );
            }
        }
    }

    CallbackBlock
}

#[no_mangle]
pub unsafe extern "system" fn ExCompareExchangeCallBack(
    CallBack: *mut _EX_CALLBACK,
    NewBlock: *mut _EX_CALLBACK_ROUTINE_BLOCK,
    OldBlock: *mut _EX_CALLBACK_ROUTINE_BLOCK,
) -> BOOLEAN {
    let CallBack = opaque_ptr(CallBack);
    let NewBlock = opaque_ptr(NewBlock);
    let OldBlock = opaque_ptr(OldBlock);
    if !NewBlock.is_null() {
        if ExfAcquireRundownProtectionEx(
            core::ptr::addr_of_mut!((*NewBlock).RundownProtect),
            MAX_FAST_REFS + 1,
        ) == 0
        {
            return 0u8;
        }
    }

    let OldValue = ExCompareSwapFastReference(
        core::ptr::addr_of_mut!((*CallBack).RoutineBlock),
        NewBlock as PVOID,
        OldBlock as PVOID,
    );

    let CallbackBlock = ExGetObjectFastReference(OldValue) as *mut _EX_CALLBACK_ROUTINE_BLOCK;
    let Count = ExGetCountFastReference(OldValue) as ULONG;

    if CallbackBlock == OldBlock {
        if !CallbackBlock.is_null() {
            KeEnterCriticalRegion();
            ExWaitOnPushLock(core::ptr::addr_of_mut!(ExpCallBackFlush));
            KeLeaveCriticalRegion();
            ExfReleaseRundownProtectionEx(
                core::ptr::addr_of_mut!((*CallbackBlock).RundownProtect),
                Count + 1,
            );
        }
        1u8
    } else {
        if !NewBlock.is_null() {
            ExfReleaseRundownProtectionEx(
                core::ptr::addr_of_mut!((*NewBlock).RundownProtect),
                MAX_FAST_REFS + 1,
            );
        }
        0u8
    }
}

unsafe extern "system" fn ExpDeleteCallback(Object: PVOID) {
    ntkrnl_sys::ASSERT!(IsListEmpty(core::ptr::addr_of_mut!(
        (*(Object as *mut _CALLBACK_OBJECT)).RegisteredCallbacks
    )));
}

// CODE_SEG("INIT")
#[no_mangle]
pub unsafe extern "system" fn ExpInitializeCallbacks() -> BOOLEAN {
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut DirName: _UNICODE_STRING = core::mem::zeroed();
    let mut CallbackName: _UNICODE_STRING = core::mem::zeroed();
    let mut Name: _UNICODE_STRING = core::mem::zeroed();
    let mut ObjectTypeInitializer: _OBJECT_TYPE_INITIALIZER = core::mem::zeroed();
    let mut DirectoryHandle: HANDLE = core::ptr::null_mut();

    (*core::ptr::addr_of_mut!(ExpCallBackFlush)).__bindgen_anon_1.Value = 0;

    core::ptr::write_bytes(
        core::ptr::addr_of_mut!(ObjectTypeInitializer) as *mut u8,
        0,
        core::mem::size_of::<_OBJECT_TYPE_INITIALIZER>(),
    );
    RtlInitUnicodeString(
        core::ptr::addr_of_mut!(Name),
        w!("Callback").as_ptr() as *const u16,
    );
    ObjectTypeInitializer.Length = core::mem::size_of::<_OBJECT_TYPE_INITIALIZER>() as u16;
    ObjectTypeInitializer.InvalidAttributes = OBJ_OPENLINK;
    ObjectTypeInitializer.GenericMapping = ExpCallbackMapping;
    ObjectTypeInitializer.PoolType = NonPagedPool;
    ObjectTypeInitializer.DeleteProcedure = Some(abi_cast!(ExpDeleteCallback));
    ObjectTypeInitializer.ValidAccessMask = CALLBACK_ALL_ACCESS;
    let Status = ObCreateObjectType(
        core::ptr::addr_of_mut!(Name),
        core::ptr::addr_of_mut!(ObjectTypeInitializer),
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(ExCallbackObjectType),
    );
    if Status < 0 {
        return 0u8;
    }

    RtlInitUnicodeString(
        core::ptr::addr_of_mut!(DirName),
        w!("\\Callback").as_ptr() as *const u16,
    );
    InitializeObjectAttributes(
        core::ptr::addr_of_mut!(ObjectAttributes),
        core::ptr::addr_of_mut!(DirName),
        OBJ_CASE_INSENSITIVE | OBJ_PERMANENT,
        core::ptr::null_mut(),
        SePublicDefaultSd,
    );

    let Status = NtCreateDirectoryObject(
        core::ptr::addr_of_mut!(DirectoryHandle),
        DIRECTORY_ALL_ACCESS,
        core::ptr::addr_of_mut!(ObjectAttributes),
    );
    if Status < 0 {
        return 0u8;
    }

    NtClose(DirectoryHandle);

    KeInitializeEvent(
        core::ptr::addr_of_mut!(ExpCallbackEvent),
        ntkrnl_sys::ddk::_EVENT_TYPE_NotificationEvent as _,
        0u8,
    );

    let mut i = 0;
    while !(*(*(&raw const ExpInitializeCallback)).get_unchecked(i)).CallbackObject.is_null() {
        RtlInitUnicodeString(
            core::ptr::addr_of_mut!(CallbackName),
            (*(*(&raw const ExpInitializeCallback)).get_unchecked(i)).Name,
        );

        InitializeObjectAttributes(
            core::ptr::addr_of_mut!(ObjectAttributes),
            core::ptr::addr_of_mut!(CallbackName),
            OBJ_PERMANENT | OBJ_CASE_INSENSITIVE,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        );

        let Status = ExCreateCallback(
            (*(*(&raw const ExpInitializeCallback)).get_unchecked(i)).CallbackObject,
            core::ptr::addr_of_mut!(ObjectAttributes),
            1u8,
            1u8,
        );
        if Status < 0 {
            return 0u8;
        }
        i += 1;
    }

    1u8
}

#[no_mangle]
pub unsafe extern "system" fn ExCreateCallback(
    CallbackObject: *mut PCALLBACK_OBJECT,
    ObjectAttributes: *mut _OBJECT_ATTRIBUTES,
    Create: BOOLEAN,
    AllowMultipleCallbacks: BOOLEAN,
) -> NTSTATUS {
    let CallbackObject = opaque_ptr(CallbackObject);
    let ObjectAttributes = opaque_ptr(ObjectAttributes);
    let mut Callback: *mut _CALLBACK_OBJECT = core::ptr::null_mut();
    let mut Status: NTSTATUS;
    let mut Handle: HANDLE = core::ptr::null_mut();

    if !(*ObjectAttributes).ObjectName.is_null() {
        Status = ObOpenObjectByName(
            ObjectAttributes,
            ExCallbackObjectType,
            0i8,
            core::ptr::null_mut(),
            0,
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!(Handle),
        );
    } else {
        Status = STATUS_UNSUCCESSFUL;
    }

    if Status < 0 && Create != 0 {
        Status = ObCreateObject(
            0i8,
            ExCallbackObjectType,
            ObjectAttributes,
            0i8,
            core::ptr::null_mut(),
            core::mem::size_of::<_CALLBACK_OBJECT>() as ULONG,
            0,
            0,
            core::ptr::addr_of_mut!(Callback) as *mut PVOID,
        );
        if Status >= 0 {
            (*Callback).Signature = 0x6C6C6143;
            KeInitializeSpinLock(core::ptr::addr_of_mut!((*Callback).Lock));
            InitializeListHead(core::ptr::addr_of_mut!((*Callback).RegisteredCallbacks));
            (*Callback).AllowMultipleCallbacks = AllowMultipleCallbacks;

            Status = ObInsertObject(
                Callback as PVOID,
                core::ptr::null_mut(),
                FILE_READ_DATA,
                0,
                core::ptr::null_mut(),
                core::ptr::addr_of_mut!(Handle),
            );
        }
    }

    if Status >= 0 {
        Status = ObReferenceObjectByHandle(
            Handle,
            0,
            ExCallbackObjectType,
            0i8,
            core::ptr::addr_of_mut!(Callback) as *mut PVOID,
            core::ptr::null_mut(),
        );

        ZwClose(Handle);
    }

    if Status >= 0 {
        *CallbackObject = Callback;
    }
    Status
}

#[no_mangle]
pub unsafe extern "system" fn ExNotifyCallback(
    CallbackObject: PCALLBACK_OBJECT,
    Argument1: PVOID,
    Argument2: PVOID,
) {
    let Argument1 = opaque_ptr(Argument1);
    let Argument2 = opaque_ptr(Argument2);
    let mut RegisteredCallbacks: PLIST_ENTRY;
    let mut CallbackRegistration: *mut _CALLBACK_REGISTRATION;
    let mut OldIrql: KIRQL = 0;

    if CallbackObject.is_null()
        || IsListEmpty(core::ptr::addr_of_mut!((*CallbackObject).RegisteredCallbacks))
    {
        return;
    }

    // [guard] KeAcquireSpinLock(addr_of_mut!((*CallbackObject).Lock), addr_of_mut!(OldIrql));
    let mut spin_guard_0 = unsafe { ntkrnl_sys::guards::SpinLockGuard::acquire(
        core::ptr::addr_of_mut!((*CallbackObject).Lock) as ntkrnl_sys::ke::PKSPIN_LOCK
    ) };

    RegisteredCallbacks = (*CallbackObject).RegisteredCallbacks.Flink;
    while RegisteredCallbacks != core::ptr::addr_of_mut!((*CallbackObject).RegisteredCallbacks) {
        CallbackRegistration = CONTAINING_RECORD(
            RegisteredCallbacks as PVOID,
            core::mem::size_of::<_CALLBACK_REGISTRATION>(),
            core::mem::offset_of!(_CALLBACK_REGISTRATION, Link),
        ) as *mut _CALLBACK_REGISTRATION;

        if (*CallbackRegistration).UnregisterWaiting == 0 {
            (*CallbackRegistration).Busy += 1;

            // [guard-drop] KeReleaseSpinLock — guard Drop handles this
            drop(spin_guard_0);

            ((*CallbackRegistration).CallbackFunction.unwrap())(
                (*CallbackRegistration).CallbackContext,
                Argument1,
                Argument2,
            );

            // [guard] KeAcquireSpinLock(addr_of_mut!((*CallbackObject).Lock), addr_of_mut!(OldIrql));
            spin_guard_0 = unsafe { ntkrnl_sys::guards::SpinLockGuard::acquire(
                core::ptr::addr_of_mut!((*CallbackObject).Lock) as ntkrnl_sys::ke::PKSPIN_LOCK
            ) };

            (*CallbackRegistration).Busy -= 1;

            if (*CallbackRegistration).UnregisterWaiting != 0
                && (*CallbackRegistration).Busy == 0
            {
                KeSetEvent(
                    core::ptr::addr_of_mut!(ExpCallbackEvent) as PVOID,
                    0,
                    0u8,
                );
            }
        }

        RegisteredCallbacks = (*RegisteredCallbacks).Flink;
    }

    // [guard-drop] KeReleaseSpinLock — guard Drop handles this
    drop(spin_guard_0);
}

#[no_mangle]
pub unsafe extern "system" fn ExRegisterCallback(
    CallbackObject: PCALLBACK_OBJECT,
    CallbackFunction: PCALLBACK_FUNCTION,
    CallbackContext: PVOID,
) -> PVOID {
    let CallbackContext = opaque_ptr(CallbackContext);
    let mut CallbackRegistration: *mut _CALLBACK_REGISTRATION = core::ptr::null_mut();
    let mut OldIrql: KIRQL = 0;

    ObReferenceObject(CallbackObject as PVOID);

    CallbackRegistration = ExAllocatePoolWithTag(
        NonPagedPool,
        core::mem::size_of::<_CALLBACK_REGISTRATION>(),
        TAG_CALLBACK_REGISTRATION,
    ) as *mut _CALLBACK_REGISTRATION;

    if CallbackRegistration.is_null() {
        ObDereferenceObject(CallbackObject as PVOID);
        return core::ptr::null_mut();
    }

    (*CallbackRegistration).CallbackObject = CallbackObject;
    (*CallbackRegistration).CallbackFunction = CallbackFunction;
    (*CallbackRegistration).CallbackContext = CallbackContext;
    (*CallbackRegistration).Busy = 0;
    (*CallbackRegistration).UnregisterWaiting = 0u8;

    // [guard] KeAcquireSpinLock(addr_of_mut!((*CallbackObject).Lock), addr_of_mut!(OldIrql));
    let spin_guard = unsafe { ntkrnl_sys::guards::SpinLockGuard::acquire(
        core::ptr::addr_of_mut!((*CallbackObject).Lock) as ntkrnl_sys::ke::PKSPIN_LOCK
    ) };

    if (*CallbackObject).AllowMultipleCallbacks != 0
        || IsListEmpty(core::ptr::addr_of_mut!((*CallbackObject).RegisteredCallbacks))
    {
        InsertTailList(
            core::ptr::addr_of_mut!((*CallbackObject).RegisteredCallbacks),
            core::ptr::addr_of_mut!((*CallbackRegistration).Link),
        );

        // [guard-drop] KeReleaseSpinLock — guard Drop handles this
        drop(spin_guard);
    } else {
        // [guard-drop] KeReleaseSpinLock — guard Drop handles this
        drop(spin_guard);

        ExFreePoolWithTag(CallbackRegistration as PVOID, TAG_CALLBACK_REGISTRATION);
        CallbackRegistration = core::ptr::null_mut();

        ObDereferenceObject(CallbackObject as PVOID);
    }

    CallbackRegistration as PVOID
}

#[no_mangle]
pub unsafe extern "system" fn ExUnregisterCallback(CallbackRegistrationHandle: PVOID) {
    let CallbackRegistrationHandle = opaque_ptr(CallbackRegistrationHandle);
    let CallbackRegistration = CallbackRegistrationHandle as *mut _CALLBACK_REGISTRATION;
    let CallbackObject = (*CallbackRegistration).CallbackObject;
    let mut OldIrql: KIRQL = 0;

    // [guard] KeAcquireSpinLock(addr_of_mut!((*CallbackObject).Lock), addr_of_mut!(OldIrql));
    let mut spin_guard_0 = unsafe { ntkrnl_sys::guards::SpinLockGuard::acquire(
        core::ptr::addr_of_mut!((*CallbackObject).Lock) as ntkrnl_sys::ke::PKSPIN_LOCK
    ) };

    while (*CallbackRegistration).Busy != 0 {
        (*CallbackRegistration).UnregisterWaiting = 1u8;

        // [guard-drop] KeReleaseSpinLock — guard Drop handles this
        drop(spin_guard_0);

        KeClearEvent(core::ptr::addr_of_mut!(ExpCallbackEvent) as PVOID);

        KeWaitForSingleObject(
            core::ptr::addr_of_mut!(ExpCallbackEvent) as PVOID,
            0u32,
            0i8,
            0u8,
            core::ptr::null_mut(),
        );

        // [guard] KeAcquireSpinLock(addr_of_mut!((*CallbackObject).Lock), addr_of_mut!(OldIrql));
        spin_guard_0 = unsafe { ntkrnl_sys::guards::SpinLockGuard::acquire(
            core::ptr::addr_of_mut!((*CallbackObject).Lock) as ntkrnl_sys::ke::PKSPIN_LOCK
        ) };
    }

    RemoveEntryList(core::ptr::addr_of_mut!((*CallbackRegistration).Link));

    // [guard-drop] KeReleaseSpinLock — guard Drop handles this
    drop(spin_guard_0);

    ExFreePoolWithTag(CallbackRegistration as PVOID, TAG_CALLBACK_REGISTRATION);

    ObDereferenceObject(CallbackObject as PVOID);
}

// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         ReactOS Kernel
//  * LICENSE:         GPL - See COPYING in the top level directory
//  * FILE:            ntoskrnl/ex/callback.c
//  * PURPOSE:         Executive callbacks
//  * PROGRAMMERS:     Alex Ionescu (alex.ionescu@reactos.org)
//  */
//
// /* INCLUDES ******************************************************************/
//
// #include <ntoskrnl.h>
// #define NDEBUG
// #include <debug.h>
//
// /* TYPES *********************************************************************/
//
// /* Mapping for Callback Object */
// _GENERIC_MAPPING ExpCallbackMapping =
// {
//     STANDARD_RIGHTS_READ,
//     STANDARD_RIGHTS_WRITE | CALLBACK_MODIFY_STATE,
//     STANDARD_RIGHTS_EXECUTE | SYNCHRONIZE,
//     CALLBACK_ALL_ACCESS
// };
//
// /* Kernel Default Callbacks */
// PCALLBACK_OBJECT SetSystemTimeCallback;
// PCALLBACK_OBJECT SetSystemStateCallback;
// PCALLBACK_OBJECT PowerStateCallback;
// SYSTEM_CALLBACKS ExpInitializeCallback[] =
// {
//    {&SetSystemTimeCallback, L"\\Callback\\SetSystemTime"},
//    {&SetSystemStateCallback, L"\\Callback\\SetSystemState"},
//    {&PowerStateCallback, L"\\Callback\\PowerState"},
//    {NULL, NULL}
// };
//
// POBJECT_TYPE ExCallbackObjectType;
// KEVENT ExpCallbackEvent;
// EX_PUSH_LOCK ExpCallBackFlush;
//
// /* PRIVATE FUNCTIONS *********************************************************/
//
// VOID
// NTAPI
// ExInitializeCallBack(IN OUT PEX_CALLBACK Callback)
// {
//     /* Initialize the fast reference */
//     ExInitializeFastReference(&Callback->RoutineBlock, NULL);
// }
//
// PEX_CALLBACK_ROUTINE_BLOCK
// NTAPI
// ExAllocateCallBack(IN PEX_CALLBACK_FUNCTION Function,
//                    IN PVOID Context)
// {
//     PEX_CALLBACK_ROUTINE_BLOCK CallbackBlock;
//
//     /* Allocate a callback */
//     CallbackBlock = ExAllocatePoolWithTag(PagedPool,
//                                           sizeof(EX_CALLBACK_ROUTINE_BLOCK),
//                                           TAG_CALLBACK_ROUTINE_BLOCK);
//     if (CallbackBlock)
//     {
//         /* Initialize it */
//         CallbackBlock->Function = Function;
//         CallbackBlock->Context = Context;
//         ExInitializeRundownProtection(&CallbackBlock->RundownProtect);
//     }
//
//     /* Return it */
//     return CallbackBlock;
// }
//
// VOID
// NTAPI
// ExFreeCallBack(IN PEX_CALLBACK_ROUTINE_BLOCK CallbackBlock)
// {
//     /* Just free it from memory */
//     ExFreePoolWithTag(CallbackBlock, TAG_CALLBACK_ROUTINE_BLOCK);
// }
//
// VOID
// NTAPI
// ExWaitForCallBacks(IN PEX_CALLBACK_ROUTINE_BLOCK CallbackBlock)
// {
//     /* Wait on the rundown */
//     ExWaitForRundownProtectionRelease(&CallbackBlock->RundownProtect);
// }
//
// PEX_CALLBACK_FUNCTION
// NTAPI
// ExGetCallBackBlockRoutine(IN PEX_CALLBACK_ROUTINE_BLOCK CallbackBlock)
// {
//     /* Return the function */
//     return CallbackBlock->Function;
// }
//
// PVOID
// NTAPI
// ExGetCallBackBlockContext(IN PEX_CALLBACK_ROUTINE_BLOCK CallbackBlock)
// {
//     /* Return the context */
//     return CallbackBlock->Context;
// }
//
// VOID
// NTAPI
// ExDereferenceCallBackBlock(IN OUT PEX_CALLBACK CallBack,
//                            IN PEX_CALLBACK_ROUTINE_BLOCK CallbackBlock)
// {
//     /* Release a fast reference */
//     if (!ExReleaseFastReference(&CallBack->RoutineBlock, CallbackBlock))
//     {
//         /* Take slow path */
//         ExReleaseRundownProtection(&CallbackBlock->RundownProtect);
//     }
// }
//
// PEX_CALLBACK_ROUTINE_BLOCK
// NTAPI
// ExReferenceCallBackBlock(IN OUT PEX_CALLBACK CallBack)
// {
//     EX_FAST_REF OldValue;
//     ULONG_PTR Count;
//     PEX_CALLBACK_ROUTINE_BLOCK CallbackBlock;
//
//     /* Acquire a reference */
//     OldValue = ExAcquireFastReference(&CallBack->RoutineBlock);
//     Count = ExGetCountFastReference(OldValue);
//
//     /* Fail if there isn't any object */
//     if (!ExGetObjectFastReference(OldValue)) return NULL;
//
//     /* Check if we don't have a reference */
//     if (!Count)
//     {
//         /* FIXME: Race */
//         DPRINT1("Unhandled callback race condition\n");
//         ASSERT(FALSE);
//         return NULL;
//     }
//
//     /* Get the callback block */
//     CallbackBlock = ExGetObjectFastReference(OldValue);
//
//     /* Check if this is the last reference */
//     if (Count == 1)
//     {
//         /* Acquire rundown protection */
//         if (ExfAcquireRundownProtectionEx(&CallbackBlock->RundownProtect,
//                                           MAX_FAST_REFS))
//         {
//             /* Insert references */
//             if (!ExInsertFastReference(&CallBack->RoutineBlock, CallbackBlock))
//             {
//                 /* Backdown the rundown acquire */
//                 ExfReleaseRundownProtectionEx(&CallbackBlock->RundownProtect,
//                                               MAX_FAST_REFS);
//             }
//         }
//     }
//
//     /* Return the callback block */
//     return CallbackBlock;
// }
//
// BOOLEAN
// NTAPI
// ExCompareExchangeCallBack(IN OUT PEX_CALLBACK CallBack,
//                           IN PEX_CALLBACK_ROUTINE_BLOCK NewBlock,
//                           IN PEX_CALLBACK_ROUTINE_BLOCK OldBlock)
// {
//     EX_FAST_REF OldValue;
//     PEX_CALLBACK_ROUTINE_BLOCK CallbackBlock;
//     ULONG Count;
//
//     /* Check that we have a new block */
//     if (NewBlock)
//     {
//         /* Acquire rundown */
//         if (!ExfAcquireRundownProtectionEx(&NewBlock->RundownProtect,
//                                            MAX_FAST_REFS + 1))
//         {
//             /* This should never happen */
//             ASSERTMSG("Callback block is already undergoing rundown\n", FALSE);
//             return FALSE;
//         }
//     }
//
//     /* Do the swap */
//     OldValue = ExCompareSwapFastReference(&CallBack->RoutineBlock,
//                                           NewBlock,
//                                           OldBlock);
//
//     /* Get the routine block */
//     CallbackBlock = ExGetObjectFastReference(OldValue);
//     Count = ExGetCountFastReference(OldValue);
//
//     /* Make sure the swap worked */
//     if (CallbackBlock == OldBlock)
//     {
//         /* Make sure we replaced a valid pointer */
//         if (CallbackBlock)
//         {
//             /* Acquire the flush lock and immediately release it */
//             KeEnterCriticalRegion();
//             ExWaitOnPushLock(&ExpCallBackFlush);
//
//             /* Release rundown protection */
//             KeLeaveCriticalRegion();
//             ExfReleaseRundownProtectionEx(&CallbackBlock->RundownProtect,
//                                           Count + 1);
//         }
//
//         /* Compare worked */
//         return TRUE;
//     }
//     else
//     {
//         /* It failed, check if we had a block */
//         if (NewBlock)
//         {
//             /* We did, remove the references that we had added */
//             ExfReleaseRundownProtectionEx(&NewBlock->RundownProtect,
//                                           MAX_FAST_REFS + 1);
//         }
//
//         /* Return failure */
//         return FALSE;
//     }
// }
//
// VOID
// NTAPI
// ExpDeleteCallback(IN PVOID Object)
// {
//     /* Sanity check */
//     ASSERT(IsListEmpty(&((PCALLBACK_OBJECT)Object)->RegisteredCallbacks));
// }
//
// /*++
//  * @name ExpInitializeCallbacks
//  *
//  * Creates the Callback Object as a valid Object Type in the Kernel.
//  * Internal function, subject to further review
//  *
//  * @return TRUE if the Callback Object Type was successfully created.
//  *
//  * @remarks None
//  *
//  *--*/
// CODE_SEG("INIT")
// BOOLEAN
// NTAPI
// ExpInitializeCallbacks(VOID)
// {
//     _OBJECT_ATTRIBUTES ObjectAttributes;
//     NTSTATUS Status;
//     _UNICODE_STRING DirName = RTL_CONSTANT_STRING(L"\\Callback");
//     _UNICODE_STRING CallbackName;
//     _UNICODE_STRING Name;
//     OBJECT_TYPE_INITIALIZER ObjectTypeInitializer;
//     HANDLE DirectoryHandle;
//     ULONG i;
//
//     /* Setup lightweight callback lock */
//     ExpCallBackFlush.Value = 0;
//
//     /* Initialize the Callback Object type  */
//     RtlZeroMemory(&ObjectTypeInitializer, sizeof(ObjectTypeInitializer));
//     RtlInitUnicodeString(&Name, L"Callback");
//     ObjectTypeInitializer.Length = sizeof(ObjectTypeInitializer);
//     ObjectTypeInitializer.InvalidAttributes = OBJ_OPENLINK;
//     ObjectTypeInitializer.GenericMapping = ExpCallbackMapping;
//     ObjectTypeInitializer.PoolType = NonPagedPool;
//     ObjectTypeInitializer.DeleteProcedure = ExpDeleteCallback;
//     ObjectTypeInitializer.ValidAccessMask = CALLBACK_ALL_ACCESS;
//     Status = ObCreateObjectType(&Name,
//                                 &ObjectTypeInitializer,
//                                 NULL,
//                                 &ExCallbackObjectType);
//     if (!NT_SUCCESS(Status)) return FALSE;
//
//     /* Initialize the Object */
//     InitializeObjectAttributes(&ObjectAttributes,
//                                &DirName,
//                                OBJ_CASE_INSENSITIVE | OBJ_PERMANENT,
//                                NULL,
//                                SePublicDefaultSd);
//
//     /* Create the Object Directory */
//     Status = NtCreateDirectoryObject(&DirectoryHandle,
//                                      DIRECTORY_ALL_ACCESS,
//                                      &ObjectAttributes);
//     if (!NT_SUCCESS(Status)) return FALSE;
//
//     /* Close Handle... */
//     NtClose(DirectoryHandle);
//
//     /* Initialize Event used when unregistering */
//     KeInitializeEvent(&ExpCallbackEvent, NotificationEvent, 0);
//
//     /* Default NT Kernel Callbacks. */
//     for (i = 0; ExpInitializeCallback[i].CallbackObject; i++)
//     {
//         /* Create the name from the structure */
//         RtlInitUnicodeString(&CallbackName, ExpInitializeCallback[i].Name);
//
//         /* Initialize the Object Attributes Structure */
//         InitializeObjectAttributes(&ObjectAttributes,
//                                    &CallbackName,
//                                    OBJ_PERMANENT | OBJ_CASE_INSENSITIVE,
//                                    NULL,
//                                    NULL);
//
//         /* Create the Callback Object */
//         Status = ExCreateCallback(ExpInitializeCallback[i].CallbackObject,
//                                   &ObjectAttributes,
//                                   TRUE,
//                                   TRUE);
//         if (!NT_SUCCESS(Status)) return FALSE;
//     }
//
//     /* Everything successful */
//     return TRUE;
// }
//
// /* PUBLIC FUNCTIONS **********************************************************/
//
// /*++
//  * @name ExCreateCallback
//  * @implemented
//  *
//  * Opens or creates a Callback Object. Creates only if Create is true.
//  * Allows multiple Callback Functions to be registered only if
//  * AllowMultipleCallbacks is true.
//  * See: https://web.archive.org/web/20081230235552/http://www.osronline.com/DDKx/kmarch/k102_967m.htm
//  *      https://www.osronline.com/article.cfm%5eid=24.htm
//  *
//  * @param CallbackObject
//  *        Pointer that will receive the Callback Object.
//  *
//  * @param CallbackName
//  *        Name of Callback
//  *
//  * @param Create
//  *        Determines if the object will be created if it doesn't exit
//  *
//  * @param AllowMultipleCallbacks
//  *        Determines if more then one registered callback function
//  *        can be attached to this Callback Object.
//  *
//  * @return STATUS_SUCESS if not failed.
//  *
//  * @remarks Must be called at IRQL = PASSIVE_LEVEL
//  *
//  *--*/
// NTSTATUS
// NTAPI
// ExCreateCallback(OUT PCALLBACK_OBJECT *CallbackObject,
//                  IN POBJECT_ATTRIBUTES ObjectAttributes,
//                  IN BOOLEAN Create,
//                  IN BOOLEAN AllowMultipleCallbacks)
// {
//     PCALLBACK_OBJECT Callback = NULL;
//     NTSTATUS Status;
//     HANDLE Handle = NULL;
//     PAGED_CODE();
//
//     /* Open a handle to the callback if it exists */
//     if (ObjectAttributes->ObjectName)
//     {
//         /* Open the handle */
//         Status = ObOpenObjectByName(ObjectAttributes,
//                                     ExCallbackObjectType,
//                                     KernelMode,
//                                     NULL,
//                                     0,
//                                     NULL,
//                                     &Handle);
//     }
//     else
//     {
//         /* Otherwise, fail */
//         Status = STATUS_UNSUCCESSFUL;
//     }
//
//     /* We weren't able to open it...should we create it? */
//     if (!(NT_SUCCESS(Status)) && (Create))
//     {
//         /* Create the object */
//         Status = ObCreateObject(KernelMode,
//                                 ExCallbackObjectType,
//                                 ObjectAttributes,
//                                 KernelMode,
//                                 NULL,
//                                 sizeof(CALLBACK_OBJECT),
//                                 0,
//                                 0,
//                                 (PVOID *)&Callback);
//         if (NT_SUCCESS(Status))
//         {
//             /* Set it up */
//             Callback->Signature = 'llaC';
//             KeInitializeSpinLock(&Callback->Lock);
//             InitializeListHead(&Callback->RegisteredCallbacks);
//             Callback->AllowMultipleCallbacks = AllowMultipleCallbacks;
//
//             /* Insert the object into the object namespace */
//             Status = ObInsertObject(Callback,
//                                     NULL,
//                                     FILE_READ_DATA,
//                                     0,
//                                     NULL,
//                                     &Handle);
//         }
//     }
//
//     /* Check if we have success until here */
//     if (NT_SUCCESS(Status))
//     {
//         /* Get a pointer to the new object from the handle we just got */
//         Status = ObReferenceObjectByHandle(Handle,
//                                            0,
//                                            ExCallbackObjectType,
//                                            KernelMode,
//                                            (PVOID *)&Callback,
//                                            NULL);
//
//         /* Close the Handle, since we now have the pointer */
//         ZwClose(Handle);
//     }
//
//     /* Everything went fine, so return a pointer to the Object */
//     if (NT_SUCCESS(Status))
//     {
//         *CallbackObject = Callback;
//     }
//     return Status;
// }
//
// /*++
//  * @name ExNotifyCallback
//  * @implemented
//  *
//  * Calls a function pointer (a registered callback)
//  * See: https://web.archive.org/web/20090106214158/http://www.osronline.com/DDKx/kmarch/k102_2f5e.htm
//  *      http://vmsone.com/~decuslib/vmssig/vmslt99b/nt/wdm-callback.txt (DEAD_LINK)
//  *
//  * @param CallbackObject
//  *        Which callback to call
//  *
//  * @param Argument1
//  *        Pointer/data to send to callback function
//  *
//  * @param Argument2
//  *        Pointer/data to send to callback function
//  *
//  * @return None
//  *
//  * @remarks None
//  *
//  *--*/
// VOID
// NTAPI
// ExNotifyCallback(IN PCALLBACK_OBJECT CallbackObject,
//                  IN PVOID Argument1,
//                  IN PVOID Argument2)
// {
//     PLIST_ENTRY RegisteredCallbacks;
//     PCALLBACK_REGISTRATION CallbackRegistration;
//     KIRQL OldIrql;
//
//     /* Check if we don't have an object or registrations */
//     if (!(CallbackObject) ||
//         (IsListEmpty(&CallbackObject->RegisteredCallbacks)))
//     {
//         /* Don't notify */
//         return;
//     }
//
//     /* Acquire the Lock */
//     KeAcquireSpinLock(&CallbackObject->Lock, &OldIrql);
//
//     /* Enumerate through all the registered functions */
//     for (RegisteredCallbacks = CallbackObject->RegisteredCallbacks.Flink;
//          RegisteredCallbacks != &CallbackObject->RegisteredCallbacks;
//          RegisteredCallbacks = RegisteredCallbacks->Flink)
//     {
//         /* Get a pointer to a Callback Registration from the List Entries */
//         CallbackRegistration = CONTAINING_RECORD(RegisteredCallbacks,
//                                                  CALLBACK_REGISTRATION,
//                                                  Link);
//
//         /* Don't bother doing notification if it's pending to be deleted */
//         if (!CallbackRegistration->UnregisterWaiting)
//         {
//             /* Mark the Callback in use, so it won't get deleted */
//             CallbackRegistration->Busy += 1;
//
//             /* Release the Spinlock before making the call */
//             KeReleaseSpinLock(&CallbackObject->Lock, OldIrql);
//
//             /* Call the Registered Function */
//             CallbackRegistration->CallbackFunction(CallbackRegistration->
//                                                    CallbackContext,
//                                                    Argument1,
//                                                    Argument2);
//
//             /* Get SpinLock back */
//             KeAcquireSpinLock(&CallbackObject->Lock, &OldIrql);
//
//             /* We are not in use anymore */
//             CallbackRegistration->Busy -= 1;
//
//             /* Check if removal is pending and we're not active */
//             if ((CallbackRegistration->UnregisterWaiting) &&
//                 !(CallbackRegistration->Busy))
//             {
//                 /* Signal the callback event */
//                 KeSetEvent(&ExpCallbackEvent, 0, FALSE);
//             }
//         }
//     }
//
//     /* Release the Callback Object */
//     KeReleaseSpinLock(&CallbackObject->Lock, OldIrql);
// }
//
// /*++
//  * @name ExRegisterCallback
//  * @implemented
//  *
//  * Allows a function to associate a callback pointer (Function) to
//  * a created Callback object
//  * See: DDK, OSR, links in ExNotifyCallback
//  *
//  * @param CallbackObject
//  *        The Object Created with ExCreateCallBack
//  *
//  * @param CallBackFunction
//  *        Pointer to the function to be called back
//  *
//  * @param CallBackContext
//  *        Block of memory that can contain user-data which will be
//  *        passed on to the callback
//  *
//  * @return A handle to a Callback Registration Structure (MSDN Documentation)
//  *
//  * @remarks None
//  *
//  *--*/
// PVOID
// NTAPI
// ExRegisterCallback(IN PCALLBACK_OBJECT CallbackObject,
//                    IN PCALLBACK_FUNCTION CallbackFunction,
//                    IN PVOID CallbackContext)
// {
//     PCALLBACK_REGISTRATION CallbackRegistration = NULL;
//     KIRQL OldIrql;
//
//     /* Sanity checks */
//     ASSERT(CallbackFunction);
//     ASSERT_IRQL_LESS_OR_EQUAL(DISPATCH_LEVEL);
//
//     /* Create reference to Callback Object */
//     ObReferenceObject(CallbackObject);
//
//     /* Allocate memory for the structure */
//     CallbackRegistration = ExAllocatePoolWithTag(NonPagedPool,
//                                                  sizeof(CALLBACK_REGISTRATION),
//                                                  TAG_CALLBACK_REGISTRATION);
//     if (!CallbackRegistration)
//     {
//         /* Dereference and fail */
//         ObDereferenceObject (CallbackObject);
//         return NULL;
//     }
//
//     /* Create Callback Registration */
//     CallbackRegistration->CallbackObject = CallbackObject;
//     CallbackRegistration->CallbackFunction = CallbackFunction;
//     CallbackRegistration->CallbackContext = CallbackContext;
//     CallbackRegistration->Busy = 0;
//     CallbackRegistration->UnregisterWaiting = FALSE;
//
//     /* Acquire SpinLock */
//     KeAcquireSpinLock(&CallbackObject->Lock, &OldIrql);
//
//     /* Check if 1) No Callbacks registered or 2) Multiple Callbacks allowed */
//     if ((CallbackObject->AllowMultipleCallbacks) ||
//         (IsListEmpty(&CallbackObject->RegisteredCallbacks)))
//     {
//         /* Register the callback */
//         InsertTailList(&CallbackObject->RegisteredCallbacks,
//                        &CallbackRegistration->Link);
//
//         /* Release SpinLock */
//         KeReleaseSpinLock(&CallbackObject->Lock, OldIrql);
//     }
//     else
//     {
//         /* Release SpinLock */
//         KeReleaseSpinLock(&CallbackObject->Lock, OldIrql);
//
//         /* Free the registration */
//         ExFreePoolWithTag(CallbackRegistration, TAG_CALLBACK_REGISTRATION);
//         CallbackRegistration = NULL;
//
//         /* Dereference the object */
//         ObDereferenceObject(CallbackObject);
//     }
//
//     /* Return handle to Registration Object */
//     return (PVOID)CallbackRegistration;
// }
//
// /*++
//  * @name ExUnregisterCallback
//  * @implemented
//  *
//  * Deregisters a CallBack
//  * See: DDK, OSR, links in ExNotifyCallback
//  *
//  * @param CallbackRegistration
//  *        Callback Registration Handle
//  *
//  * @return None
//  *
//  * @remarks None
//  *
//  *--*/
// VOID
// NTAPI
// ExUnregisterCallback(IN PVOID CallbackRegistrationHandle)
// {
//     PCALLBACK_REGISTRATION CallbackRegistration;
//     PCALLBACK_OBJECT CallbackObject;
//     KIRQL OldIrql;
//     ASSERT_IRQL_LESS_OR_EQUAL(DISPATCH_LEVEL);
//
//     /* Convert Handle to valid Structure Pointer */
//     CallbackRegistration = (PCALLBACK_REGISTRATION)CallbackRegistrationHandle;
//
//     /* Get the Callback Object */
//     CallbackObject = CallbackRegistration->CallbackObject;
//
//     /* Lock the Object */
//     KeAcquireSpinLock (&CallbackObject->Lock, &OldIrql);
//
//     /* We can't Delete the Callback if it's in use */
//     while (CallbackRegistration->Busy)
//     {
//         /* Let everyone else know we're unregistering */
//         CallbackRegistration->UnregisterWaiting = TRUE;
//
//         /* We are going to wait for the event, so the Lock isn't necessary */
//         KeReleaseSpinLock(&CallbackObject->Lock, OldIrql);
//
//         /* Make sure the event is cleared */
//         KeClearEvent(&ExpCallbackEvent);
//
//         /* Wait for the Event */
//         KeWaitForSingleObject(&ExpCallbackEvent,
//                               Executive,
//                               KernelMode,
//                               FALSE,
//                               NULL);
//
//         /* We need the Lock again */
//         KeAcquireSpinLock(&CallbackObject->Lock, &OldIrql);
//     }
//
//     /* Remove the Callback */
//     RemoveEntryList(&CallbackRegistration->Link);
//
//     /* It's now safe to release the lock */
//     KeReleaseSpinLock(&CallbackObject->Lock, OldIrql);
//
//     /* Delete this registration */
//     ExFreePoolWithTag(CallbackRegistration, TAG_CALLBACK_REGISTRATION);
//
//     /* Remove the reference */
//     ObDereferenceObject(CallbackObject);
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
