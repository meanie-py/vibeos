use ntkrnl_sys::globals::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::ex::*;
use ntkrnl_sys::interlocked::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::ob::*;
use ntkrnl_sys::ps::*;
use ntkrnl_sys::rtl::*;
use crate::cmse::CmpSecurityMethod;
use ntkrnl_sys::ddk::{
    ACCESS_MASK, ANSI_STRING, CMHIVE, CM_KEY_BODY, CM_KEY_NODE, CM_PARSE_CONTEXT,
    CmiCallRegisteredCallbacks, CmpAcquireKcbLockExclusiveByKey, CmpAddToHiveFileList,
    CmpCloseHiveFiles, CmpConstructName, CmpCopyName, CmpCreateKeyControlBlock,
    CmpDelayDerefKeyControlBlock, CmpDestroyHive, CmpDoFlushAll, CmpHiveRootSecurityDescriptor,
    CmpInitCallback, CmpInitCmPrivateAlloc, CmpInitCmPrivateDelayAlloc,
    CmpInitDelayDerefKCBEngine, CmpInitializeCache, CmpInitializeHardwareConfiguration,
    CmpInitializeHive, CmpInitializeMachineDependentConfiguration, CmpNameSize,
    CmpOpenHiveFiles, CmpParseKey, CmpReleaseKcbLock, CmpReleaseKcbLockByKey,
    CmpShutdownWorkers, ERESOURCE, ExAcquireResourceExclusiveLite,
    ExAcquireResourceSharedLite, ExAcquireSharedStarveExclusive, ExInitializePushLock,
    ExInitializeResourceLite, ExIsResourceAcquiredExclusiveLite,
    ExIsResourceAcquiredSharedLite, ExReleaseResourceLite, _GENERIC_MAPPING, HCELL_INDEX,
    KEVENT, KPROCESSOR_MODE, PKSTART_ROUTINE, KeEnterCriticalRegion, KeInitializeEvent,
    KeLeaveCriticalRegion, KeWaitForSingleObject, LIST_ENTRY, NtClose, NtCreateKey,
    NtDeleteValueKey, NtOpenKey, NtQueryValueKey, NtRaiseHardError, NtSetValueKey,
    _OBJECT_ATTRIBUTES, OBJECT_NAME_INFORMATION, OBJECT_TYPE_INITIALIZER, ObCloseHandle,
    ObCreateObject, ObCreateObjectType, ObInsertObject, ObOpenObjectByName,
    ObReferenceObjectByHandle, PBOOT_DRIVER_LIST_ENTRY, PCMHIVE, PCM_KEY_BODY,
    PCM_KEY_CONTROL_BLOCK, PCM_KEY_NODE, PCUNICODE_STRING, PHCELL_INDEX, PHHIVE,
    PKEY_VALUE_FULL_INFORMATION, PLOADER_PARAMETER_BLOCK, PLOADER_PARAMETER_EXTENSION,
    POBJECT_NAME_INFORMATION, POBJECT_TYPE, PSECURITY_DESCRIPTOR, PUNICODE_STRING,
    PsCreateSystemThread, PsGetCurrentProcessId, PsTerminateSystemThread,
    REG_KEY_HANDLE_CLOSE_INFORMATION, REG_POST_OPERATION_INFORMATION,
    RtlAnsiStringToUnicodeString, RtlAppendUnicodeStringToString, RtlCreateUnicodeString,
    RtlCreateUnicodeStringFromAsciiz, RtlFreeUnicodeString, RtlInitAnsiString,
    RtlInitEmptyUnicodeString, RtlInitUnicodeString, RtlIntegerToUnicodeString,
    ZwClose, ZwCreateKey, ZwSetValueKey,
    _UNICODE_STRING, _LIST_ENTRY, _CM_KEY_CONTROL_BLOCK, _CM_KEY_NODE, _CMHIVE, _HHIVE,
    _CM_PARSE_CONTEXT, _CM_KEY_BODY, _OBJECT_TYPE,
    _KWAIT_REASON_Executive, _MODE_KernelMode,
};

use ntkrnl_sys::{opaque_ptr, opaque_ptr_const};
extern "system" {
    fn RtlGetCallersAddress(CallersAddress: *mut PVOID, CallersCaller: *mut PVOID);
    /// ntoskrnl-rust tripwire.rs: this boot's raw fw_cfg tripwire config.
    fn VibeTripwireRead(out: *mut u8, cap: usize) -> usize;
}
use core::ffi::c_void;

// w! macro: convert a string literal to a NUL-terminated UTF-16 array literal.
// Use at compile time so the array has 'static lifetime.
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


const CM_NUMBER_OF_MACHINE_HIVES: usize = 6;

// ============================================================================
// CmGetSystemDriverList — full translation from cmsysini.c lines 1860-1977.
// Walks the SYSTEM hive's CurrentControlSet, calls CmpFindDrivers /
// CmpSortDriverList / CmpResolveDriverDependencies, and returns an array
// of \Registry\Machine\System\CurrentControlSet\Services\<driver> paths,
// terminated by a NULL pointer. Returns NULL on any early-stage failure
// (key open, object reference). On allocation failure inside the hot path
// the function bugchecks (CONFIG_INITIALIZATION_FAILED) — matches C behaviour.
// ============================================================================
#[no_mangle]
pub unsafe extern "system" fn CmGetSystemDriverList() -> *mut PUNICODE_STRING {
    #[inline(never)]
    unsafe fn cmgsdl_trace(s: &[u8]) {
        crate::dbg_print!(b"%s\0", s.as_ptr() as *const i8);
    }
    use ntkrnl_sys::ddk::{
        BOOT_DRIVER_LIST_ENTRY, CmpFindControlSet, CmpFindDrivers,
        CmpFreeDriverList, CmpResolveDriverDependencies, CmpSortDriverList,
        RtlDuplicateUnicodeString, _CM_SERVICE_LOAD_TYPE_SystemLoad,
    };

    const RTL_DUPLICATE_UNICODE_STRING_NULL_TERMINATE_LOCAL: ULONG = 1;

    let mut DriverList: LIST_ENTRY = core::mem::zeroed();
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut Status: NTSTATUS;
    let mut KeyBody: PCM_KEY_BODY = core::ptr::null_mut();
    let Hive: PHHIVE;
    let RootCell: HCELL_INDEX;
    let ControlCell: HCELL_INDEX;
    let mut KeyHandle: HANDLE = core::ptr::null_mut();
    let mut KeyName: _UNICODE_STRING = core::mem::zeroed();
    let mut NextEntry: *mut LIST_ENTRY;
    let mut i: ULONG;
    let mut ServicePath: *mut PUNICODE_STRING = core::ptr::null_mut();
    let Success: BOOLEAN;
    let mut AutoSelect: BOOLEAN = 0u8;
    let mut DriverEntry: *mut BOOT_DRIVER_LIST_ENTRY;

    // Initialize the driver list head.
    InitializeListHead(core::ptr::addr_of_mut!(DriverList));

    // Open the system hive key: \Registry\Machine\System
    // UTF-16 LE bytes, NUL-terminated.
    RtlInitUnicodeString(
        &mut KeyName,
        w!("\\Registry\\Machine\\System").as_ptr() as *const u16,
    );

    ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
    ObjectAttributes.RootDirectory = core::ptr::null_mut();
    ObjectAttributes.ObjectName = &mut KeyName;
    ObjectAttributes.Attributes = OBJ_CASE_INSENSITIVE;
    ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
    ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

    cmgsdl_trace(b"[CMGSDL] NtOpenKey System\r\n\0");
    Status = NtOpenKey(&mut KeyHandle, KEY_READ, &mut ObjectAttributes);
    if !NT_SUCCESS(Status) {
        crate::dbg_print!(b"[CMGSDL] NtOpenKey System FAIL %x\r\n\0", Status);
        return core::ptr::null_mut();
    }

    // Reference the key object to get the root hive/cell to access directly.
    cmgsdl_trace(b"[CMGSDL] ObReferenceObjectByHandle\r\n\0");
    Status = ObReferenceObjectByHandle(
        KeyHandle,
        KEY_QUERY_VALUE,
        CmpKeyObjectType,
        KernelMode as KPROCESSOR_MODE,
        &mut KeyBody as *mut PCM_KEY_BODY as *mut PVOID,
        core::ptr::null_mut(),
    );
    if !NT_SUCCESS(Status) {
        crate::dbg_print!(b"[CMGSDL] ObRef FAIL %x\r\n\0", Status);
        NtClose(KeyHandle);
        return core::ptr::null_mut();
    }

    // Do all this under the registry lock.
    crate::dbg_print!(b"[CMGSDL] CmpLockReg pre: shared=%d excl=%d\r\n\0",
        ExIsResourceAcquiredSharedLite(core::ptr::addr_of_mut!(CmpRegistryLock)) as u32,
        ExIsResourceAcquiredExclusiveLite(core::ptr::addr_of_mut!(CmpRegistryLock)) as u32,
    );
    cmgsdl_trace(b"[CMGSDL] CmpLockRegistryExclusive\r\n\0");
    CmpLockRegistryExclusive();
    cmgsdl_trace(b"[CMGSDL] CmpLockRegistryExclusive done\r\n\0");

    // Get the hive and key cell.
    Hive = (*(*KeyBody).KeyControlBlock).__bindgen_anon_2.__bindgen_anon_1.KeyHive;
    RootCell = (*(*KeyBody).KeyControlBlock).__bindgen_anon_2.__bindgen_anon_1.KeyCell;
    crate::dbg_print!(b"[CMGSDL] Hive=%p RootCell=%x\r\n\0", Hive, RootCell);

    // Open the current control set key. RtlInitUnicodeString of L"Current".
    RtlInitUnicodeString(
        &mut KeyName,
        w!("Current").as_ptr() as *const u16,
    );

    // EndPath label is implemented via a single-iteration loop with `break`s.
    'EndPath: loop {
        cmgsdl_trace(b"[CMGSDL] CmpFindControlSet\r\n\0");
        ControlCell = CmpFindControlSet(
            Hive,
            RootCell,
            &mut KeyName as *mut _UNICODE_STRING as PCUNICODE_STRING,
            core::ptr::addr_of_mut!(AutoSelect),
        );
        if ControlCell == HCELL_NIL {
            cmgsdl_trace(b"[CMGSDL] FindControlSet returned NIL\r\n\0");
            break 'EndPath;
        }

        // Find all system drivers.
        cmgsdl_trace(b"[CMGSDL] CmpFindDrivers\r\n\0");
        Success = CmpFindDrivers(
            Hive,
            ControlCell,
            _CM_SERVICE_LOAD_TYPE_SystemLoad,
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!(DriverList),
        );
        cmgsdl_trace(b"[CMGSDL] CmpFindDrivers done\r\n\0");
        if Success == 0 {
            cmgsdl_trace(b"[CMGSDL] FindDrivers FAIL\r\n\0");
            break 'EndPath;
        }

        // Sort by group/tag.
        cmgsdl_trace(b"[CMGSDL] CmpSortDriverList\r\n\0");
        if CmpSortDriverList(Hive, ControlCell, core::ptr::addr_of_mut!(DriverList)) == 0 {
            cmgsdl_trace(b"[CMGSDL] SortDriverList FAIL\r\n\0");
            break 'EndPath;
        }

        // Remove circular dependencies (cycles) and sort.
        cmgsdl_trace(b"[CMGSDL] CmpResolveDriverDependencies\r\n\0");
        if CmpResolveDriverDependencies(core::ptr::addr_of_mut!(DriverList)) == 0 {
            cmgsdl_trace(b"[CMGSDL] ResolveDriverDependencies FAIL\r\n\0");
            break 'EndPath;
        }

        // Loop the list to count drivers.
        i = 0;
        NextEntry = DriverList.Flink;
        while NextEntry != core::ptr::addr_of_mut!(DriverList) {
            i += 1;
            NextEntry = (*NextEntry).Flink;
        }

        // Allocate the array.
        ServicePath = ExAllocatePoolWithTag(
            NonPagedPool,
            ((i + 1) as usize * core::mem::size_of::<PUNICODE_STRING>()) as _,
            TAG_CM,
        ) as *mut PUNICODE_STRING;
        if ServicePath.is_null() {
            KeBugCheckEx(CONFIG_INITIALIZATION_FAILED, 2, 1, 0, 0);
        }

        // Loop the driver list.
        i = 0;
        NextEntry = DriverList.Flink;
        while NextEntry != core::ptr::addr_of_mut!(DriverList) {
            // CONTAINING_RECORD(NextEntry, BOOT_DRIVER_LIST_ENTRY, Link).
            // Link is the first field of BOOT_DRIVER_LIST_ENTRY, so the offset is 0.
            DriverEntry = NextEntry as *mut BOOT_DRIVER_LIST_ENTRY;

            // Allocate the per-entry UNICODE_STRING.
            *ServicePath.add(i as usize) = ExAllocatePoolWithTag(
                NonPagedPool,
                core::mem::size_of::<_UNICODE_STRING>() as _,
                TAG_CM,
            ) as PUNICODE_STRING;
            if (*ServicePath.add(i as usize)).is_null() {
                KeBugCheckEx(CONFIG_INITIALIZATION_FAILED, 2, 1, 0, 0);
            }

            // Duplicate the registry path.
            Status = RtlDuplicateUnicodeString(
                RTL_DUPLICATE_UNICODE_STRING_NULL_TERMINATE_LOCAL,
                core::ptr::addr_of_mut!((*DriverEntry).RegistryPath)
                    as *const _UNICODE_STRING,
                *ServicePath.add(i as usize),
            );
            if !NT_SUCCESS(Status) {
                KeBugCheckEx(CONFIG_INITIALIZATION_FAILED, 2, 1, 0, 0);
            }

            i += 1;
            NextEntry = (*NextEntry).Flink;
        }

        // Terminate the list with a NULL pointer.
        *ServicePath.add(i as usize) = core::ptr::null_mut();

        break 'EndPath;
    }

    // Free the driver list if we had one.
    if !IsListEmpty(core::ptr::addr_of!(DriverList)) {
        CmpFreeDriverList(Hive, core::ptr::addr_of_mut!(DriverList));
    }

    // Unlock the registry.
    CmpUnlockRegistry();

    // Close the key handle and dereference the object, then return the path.
    ObDereferenceObject(KeyBody as PVOID);
    NtClose(KeyHandle);
    ServicePath
}

// ============================================================================
// Globals owned by this file (cmsysini.c is excluded from C build).
// ============================================================================
// SAFETY: Zero-initialized POD types — KEVENT/ERESOURCE/etc. are populated
// by their respective Initialize* calls during boot.
#[no_mangle] pub static mut CmpKeyObjectType: POBJECT_TYPE = core::ptr::null_mut();
#[no_mangle] pub static mut CmiVolatileHive: PCMHIVE = core::ptr::null_mut();
#[no_mangle] pub static mut CmpHiveListHead: _LIST_ENTRY =
    _LIST_ENTRY { Flink: core::ptr::null_mut(), Blink: core::ptr::null_mut() };
#[no_mangle] pub static mut CmpRegistryLock: ERESOURCE = unsafe { core::mem::zeroed() };
#[no_mangle] pub static mut CmpSelfHealQueueLock: ntkrnl_sys::ddk::_KGUARDED_MUTEX = unsafe { core::mem::zeroed() };
#[no_mangle] pub static mut CmpSelfHealQueueListHead: _LIST_ENTRY =
    _LIST_ENTRY { Flink: core::ptr::null_mut(), Blink: core::ptr::null_mut() };
#[no_mangle] pub static mut CmpLoadWorkerEvent: KEVENT = unsafe { core::mem::zeroed() };
#[no_mangle] pub static mut CmpLoadWorkerIncrement: LONG = 0;
#[no_mangle] pub static mut CmpSystemProcess: PEPROCESS = core::ptr::null_mut();
#[no_mangle] pub static mut CmpFlushOnLockRelease: BOOLEAN = 0;
#[no_mangle] pub static mut CmpSpecialBootCondition: BOOLEAN = 0;
#[no_mangle] pub static mut CmpNoWrite: BOOLEAN = 1;  // C: BOOLEAN CmpNoWrite = TRUE;
#[no_mangle] pub static mut CmpWasSetupBoot: BOOLEAN = 0;
#[no_mangle] pub static mut CmpProfileLoaded: BOOLEAN = 0;
#[no_mangle] pub static mut CmpNoVolatileCreates: BOOLEAN = 0;
#[no_mangle] pub static mut HvShutdownComplete: BOOLEAN = 0;  // C: BOOLEAN HvShutdownComplete = FALSE;
#[no_mangle] pub static mut CmpRegistryLockCaller: PVOID = core::ptr::null_mut();
#[no_mangle] pub static mut CmpRegistryLockCallerCaller: PVOID = core::ptr::null_mut();

// ============================================================================
// Globals owned by cmdata.c (still C; declared as extern statics)
// ============================================================================
extern "C" {
    pub static mut CmSymbolicLinkValueName: _UNICODE_STRING;
    pub static mut CmpLoadOptions: _UNICODE_STRING;
    pub static mut CmpHiveListHeadLock: ntkrnl_sys::ddk::_EX_PUSH_LOCK;
    pub static mut CmpLoadHiveLock: ntkrnl_sys::ddk::_EX_PUSH_LOCK;
    pub static mut CmpVolatileBoot: u32;
    pub static mut CmpShareSystemHives: u8;
    pub static mut CmpMiniNTBoot: u8;
    pub static mut CmpBootType: u32;
    pub static mut CmSelfHeal: u32;
    pub static mut CmpSelfHeal: u8;
    pub static mut CmpRegistryRootHandle: HANDLE;
    pub static mut CmpFlushStarveWriters: u32;
    pub static mut CmpMachineHiveList: [ntkrnl_sys::ddk::_HIVE_LIST_ENTRY; 7];
    pub static mut CmFirstTime: u8;
    pub static mut CmCSDVersionString: _UNICODE_STRING;
    pub static mut CmVersionString: _UNICODE_STRING;
    pub static mut CmNtSpBuildNumber: u32;
    pub static mut ExpInTextModeSetup: u8;
    pub static mut InitIsWinPEMode: u8;
    pub static mut InitSafeBootMode: u32;
    pub static NtBuildLab: i8; // C: extern CHAR NtBuildLab[]; — take addr_of
    pub static mut NtSystemRoot: _UNICODE_STRING;
}

// ============================================================================
// Hive primitives (sdk/lib/cmlib/, still C)
// ============================================================================

extern "system" {
    fn CmCheckRegistry(CmHive: PCMHIVE, Flags: u32) -> u32;
    fn KeQuerySystemTime(CurrentTime: *mut LARGE_INTEGER);
    fn HvInitialize(
        Hive: PHHIVE,
        Operation: u32,
        FileType: u32,
        OpFlags: u32,
        HiveData: PVOID,
        Allocate: PVOID,
        Free: PVOID,
        FileSetSize: PVOID,
        FileWrite: PVOID,
        FileRead: PVOID,
        FileFlush: PVOID,
        Cluster: u32,
        FileName: PCUNICODE_STRING,
    ) -> NTSTATUS;
    fn HvFreeHive(Hive: PHHIVE);
    fn HvFlushHive(Hive: PHHIVE) -> u8;
}

// ============================================================================
// Constants — registry / hive
// ============================================================================

// REG_OPTION_* (from winreg.h)
pub const REG_OPTION_VOLATILE: u32 = 0x00000001;
pub const REG_OPTION_CREATE_LINK: u32 = 0x00000002;
pub const REG_OPTION_NON_VOLATILE: u32 = 0x00000000;
pub const REG_OPTION_BACKUP_RESTORE: u32 = 0x00000004;
pub const REG_OPTION_OPEN_LINK: u32 = 0x00000008;
pub const REG_CREATED_NEW_KEY: u32 = 0x00000001;
pub const REG_OPENED_EXISTING_KEY: u32 = 0x00000002;

// REG_NOTIFY_* (winreg.h)
pub const REG_NOTIFY_CHANGE_NAME: u32 = 0x00000001;
pub const REG_NOTIFY_CHANGE_ATTRIBUTES: u32 = 0x00000002;
pub const REG_NOTIFY_CHANGE_LAST_SET: u32 = 0x00000004;
pub const REG_NOTIFY_CHANGE_SECURITY: u32 = 0x00000008;

// REG_NOTIFY_CLASS values (from cmtypes.h)
pub const RegNtPreKeyHandleClose: u32 = 14; // header value; the hand copy said 30
pub const RegNtPostKeyHandleClose: u32 = 25; // header value; the hand copy said 31

// KEY_* access (winreg.h)
pub const KEY_ALL_ACCESS: u32 = STANDARD_RIGHTS_REQUIRED
    | KEY_QUERY_VALUE
    | KEY_SET_VALUE
    | KEY_CREATE_SUB_KEY
    | KEY_ENUMERATE_SUB_KEYS
    | KEY_NOTIFY
    | KEY_CREATE_LINK;
pub const KEY_WRITE: u32 = STANDARD_RIGHTS_WRITE | KEY_SET_VALUE | KEY_CREATE_SUB_KEY;
pub const KEY_EXECUTE: u32 =
    STANDARD_RIGHTS_READ | KEY_QUERY_VALUE | KEY_ENUMERATE_SUB_KEYS | KEY_NOTIFY;

// KEY_VALUE_INFORMATION_CLASS values
pub const KeyValueBasicInformation: u32 = 0;
pub const KeyValueFullInformation: u32 = 1;
pub const KeyValuePartialInformation: u32 = 2;

// KEY_INFORMATION_CLASS values
pub const KeyBasicInformation: u32 = 0;
pub const KeyNodeInformation: u32 = 1;
pub const KeyFullInformation: u32 = 2;

// HCELL_NIL (cmlib/hivedata.h)
pub const HCELL_NIL: u32 = 0xFFFFFFFF;

// HFILE_TYPE_* (cmlib/hivedata.h)
pub const HFILE_TYPE_PRIMARY: u32 = 0;
pub const HFILE_TYPE_LOG: u32 = 1;
pub const HFILE_TYPE_EXTERNAL: u32 = 2;
pub const HFILE_TYPE_ALTERNATE: u32 = 3;
pub const HFILE_TYPE_MAX: u32 = 4;

// HINIT_* (cmlib/hivedata.h)
pub const HINIT_CREATE: u32 = 0;
pub const HINIT_MEMORY: u32 = 1;
pub const HINIT_FILE: u32 = 2;
pub const HINIT_MEMORY_INPLACE: u32 = 3;
pub const HINIT_FLAT: u32 = 4;
pub const HINIT_MAPFILE: u32 = 5;

// HIVE_* (cmlib/hivedata.h) — one definition, in crate::hiveflags.
pub use crate::hiveflags::{HIVE_NOLAZYFLUSH, HIVE_VOLATILE};

// HBOOT_TYPE_*
pub const HBOOT_TYPE_REGULAR: u32 = 0;
pub const HBOOT_TYPE_SELF_HEAL: u32 = 4;

// HSTORAGE_TYPE
pub const Stable: u32 = 0;
pub const Volatile: u32 = 1;

// CM_CHECK_REGISTRY_* (config/cm.h)
pub const CM_CHECK_REGISTRY_DONT_PURGE_VOLATILES: u32 = 0x00000000;
pub const CM_CHECK_REGISTRY_PURGE_VOLATILES: u32 = 0x00000002;
pub const CM_CHECK_REGISTRY_BOOTLOADER_PURGE_VOLATILES: u32 = 0x00000004;
pub const CM_CHECK_REGISTRY_VALIDATE_HIVE: u32 = 0x00000008;
pub const CM_CHECK_REGISTRY_FIX_HIVE: u32 = 0x00000010;

// CM Pool tags (from config/cm.h)
// C: cmlib.h `#define TAG_CM '  MC'`. A multi-char literal is BIG-endian.
pub const TAG_CM: u32 = u32::from_be_bytes(*b"  MC");
// C: cmlib.h `#define TAG_CMHIVE 'vHMC'` (big-endian multi-char literal).
pub const TAG_CMHIVE: u32 = u32::from_be_bytes(*b"vHMC");
pub const TAG_CMSD: u32 = u32::from_le_bytes(*b"dSmC");    // 'CmSd'
pub const TAG_KCB: u32 = u32::from_be_bytes(*b"bkMC"); // C literal 'bkMC' -- sdk/lib/cmlib/cmlib.h

// CM_KEY_BODY signature
pub const CM_KEY_BODY_TYPE: u32 = 0x6B793032;              // "ky02"

// CM key-control-block flags / counts
pub const CM_KCBS_PER_PAGE: u32 = 16;
pub const CM_DELAYS_PER_PAGE: u32 = 16;
pub const CM_KCB_NO_SUBKEY: u32 = 0x01;
pub const CM_KCB_SUBKEY_ONE: u32 = 0x02;
pub const CM_KCB_SUBKEY_HINT: u32 = 0x04;
pub const CM_KCB_INVALID_CACHED_INFO: u32 = 0x40;
pub const CM_KCB_READ_ONLY_KEY: u32 = 0x80;
pub const CM_KCB_SYM_LINK_FOUND: u32 = 0x08;
pub const KEY_HIVE_ENTRY: u32 = 0x04;
pub const KEY_NO_DELETE: u32 = 0x08;
pub const KEY_COMP_NAME: u32 = 0x20;

// CMP_LOCK_*/_ENLIST_*
pub const CMP_LOCK_KCB_ARRAY_EXCLUSIVE: u32 = 0x1; // ntoskrnl/include/internal/cm.h
pub const CMP_ENLIST_KCB_LOCKED_SHARED: u32 = 0x1; // ntoskrnl/include/internal/cm.h
pub const CMP_ENLIST_KCB_LOCKED_EXCLUSIVE: u32 = 0x2; // ntoskrnl/include/internal/cm.h
pub const CMP_SUBKEY_LEVELS_DEPTH_LIMIT: u32 = 0x20; // ntoskrnl/include/internal/cm.h (32)

// SERVICE_*
pub const SERVICE_ERROR_CRITICAL: u32 = 3;

// UNICODE_NULL
pub const UNICODE_NULL: u16 = 0;

// MaximumClass / MaximumType placeholders for table sizing (not used directly)

// _OBJECT_INFORMATION_CLASS_ObjectNameInformation
pub const _OBJECT_INFORMATION_CLASS_ObjectNameInformation: u32 = 1;

// _WORK_QUEUE_TYPE_DelayedWorkQueue
pub const _WORK_QUEUE_TYPE_DelayedWorkQueue: u32 = 1;

// _SECURITY_IMPERSONATION_LEVEL_SecurityImpersonation
pub const _SECURITY_IMPERSONATION_LEVEL_SecurityImpersonation: u32 = 2;

// SearchSuccess / SearchNeedExclusiveLock (VALUE_SEARCH_RETURN_TYPE)
pub const SearchSuccess: u32 = 0;
pub const SearchNeedExclusiveLock: u32 = 1;
pub const SearchFail: u32 = 2;

// Bugcheck codes
pub const BAD_SYSTEM_CONFIG_INFO: u32 = 0x00000074;
pub const CONFIG_INITIALIZATION_FAILED: u32 = 0x00000067;

// Status codes used by hardrror dispatch
pub const STATUS_RETRY: NTSTATUS = 0xC000022D_u32 as NTSTATUS;
pub const STATUS_ACCOUNT_DISABLED: NTSTATUS = 0xC0000072_u32 as NTSTATUS;
pub const STATUS_ACCOUNT_EXPIRED: NTSTATUS = 0xC0000193_u32 as NTSTATUS;
pub const STATUS_ACCOUNT_RESTRICTION: NTSTATUS = 0xC000006E_u32 as NTSTATUS;
pub const STATUS_NO_SUCH_USER: NTSTATUS = 0xC0000064_u32 as NTSTATUS;
pub const STATUS_WRONG_PASSWORD: NTSTATUS = 0xC000006A_u32 as NTSTATUS;

// ============================================================================
// Macro / inline shims
// ============================================================================


#[inline(always)]
unsafe fn CMP_ASSERT_EXCLUSIVE_REGISTRY_LOCK() {}

#[inline(always)]
unsafe fn CMP_ASSERT_HASH_ENTRY_LOCK(_idx: u32) {}

#[inline(always)]
unsafe fn CMP_ASSERT_KCB_LOCK(_kcb: PCM_KEY_CONTROL_BLOCK) {}

#[inline(always)]
unsafe fn GET_HASH_ENTRY(_table: *mut core::ffi::c_void, _key: u32) -> *mut core::ffi::c_void {
    core::ptr::null_mut()
}

#[inline(always)]
unsafe fn KeBugCheck(code: u32) -> ! {
    KeBugCheckEx(code, 0, 0, 0, 0)
}


// RTL_CONSTANT_STRING: const-eval a UNICODE_STRING from a literal.
#[inline(always)]
unsafe fn RTL_CONSTANT_STRING(_s: *const u16) -> _UNICODE_STRING {
    core::mem::zeroed()
}

// CmpAcquireKcbLockShared shim: in upstream this is a macro that ultimately
// calls CmpAcquireKcbLockSharedByIndex.
extern "C" {
    fn CmpAcquireKcbLockSharedByIndex(Index: u32);
}

#[inline(always)]
unsafe fn kcb_conv_key(Kcb: PCM_KEY_CONTROL_BLOCK) -> u32 {
    (*Kcb).__bindgen_anon_2.__bindgen_anon_1.ConvKey
}

#[inline(always)]
unsafe fn kcb_delete(Kcb: PCM_KEY_CONTROL_BLOCK) -> u32 {
    (*Kcb).__bindgen_anon_1.Delete()
}

// cm_x.h: `CmpAcquireKcbLockSharedByKey(k)` is `...ByIndex(GET_HASH_INDEX(k))`. This local copy
// passed the raw ConvKey hash as the INDEX, so CmpQueryKeyName (NtQueryObject for a key's
// name) did a `lock cmpxchg` at CmpCacheTable[hash]: a 0x1E when that was unmapped, a silent
// write into unrelated kernel memory when it was not, and then released the REAL bucket
// through the shared CmpReleaseKcbLock, which it had never acquired.
#[inline(always)]
unsafe fn CmpAcquireKcbLockShared(Kcb: PCM_KEY_CONTROL_BLOCK) {
    if !Kcb.is_null() {
        CmpAcquireKcbLockSharedByIndex(GET_HASH_INDEX(kcb_conv_key(Kcb)));
    }
}

// EnlistKeyBodyWithKCB / DelistKeyBodyFromKCB / RtlStringCbPrintfW —
// declared elsewhere in the kernel (cmkcbncb.c, rtl/wcstr.c).
extern "system" {
    fn EnlistKeyBodyWithKCB(KeyBody: PCM_KEY_BODY, Flags: u32);
    fn DelistKeyBodyFromKCB(KeyBody: PCM_KEY_BODY, Locked: u8);
}

// ExInitializeWorkItem (deprecated WDM macro replaced by ExInitializeWorkItem
// inline). Provide a tiny shim — not all paths exercise this code.
#[inline(always)]
unsafe fn ExInitializeWorkItem(_item: PVOID, _routine: PVOID, _ctx: PVOID) {}

extern "system" {
    fn HvAllocateCell(Hive: PHHIVE, Size: u32, Type: u32, Nil: u32) -> u32;
    fn ExIsResourceAcquiredShared(Resource: *mut core::ffi::c_void) -> u32;
    fn HvSyncHive(Hive: PHHIVE);
    fn HvWriteAlternateHive(Hive: PHHIVE) -> u8;
}
extern "C" {
    fn HvGetCell(Hive: PHHIVE, Cell: u32) -> *mut core::ffi::c_void;
    fn HvReleaseCell(Hive: PHHIVE, Cell: u32);
    fn CMP_ASSERT_REGISTRY_LOCK_OR_LOADING(Hive: PCMHIVE);
    fn CMP_ASSERT_FLUSH_LOCK(Hive: PCMHIVE);
    fn CMP_ASSERT_REGISTRY_LOCK();
    fn GET_HASH_INDEX(key: ULONG) -> ULONG;
    fn PtrToUlong(ptr: *mut core::ffi::c_void) -> u32;
}

#[no_mangle]
pub unsafe extern "system" fn CmpLinkKeyToHive(
    LinkKeyName: PCWSTR,
    TargetKeyName: PCWSTR,
) -> u8 {
    let LinkKeyName = opaque_ptr_const(LinkKeyName);
    let TargetKeyName = opaque_ptr_const(TargetKeyName);
    let mut Status: NTSTATUS;
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut KeyName: _UNICODE_STRING = core::mem::zeroed();
    let mut LinkKeyHandle: HANDLE = core::ptr::null_mut();
    let mut Disposition: ULONG = 0;

    RtlInitUnicodeString(&mut KeyName, LinkKeyName);
    
    ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
    ObjectAttributes.RootDirectory = core::ptr::null_mut();
    ObjectAttributes.ObjectName = &mut KeyName;
    ObjectAttributes.Attributes = OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE;
    ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
    ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

    Status = ZwCreateKey(
        &mut LinkKeyHandle,
        KEY_CREATE_LINK,
        &mut ObjectAttributes,
        0,
        core::ptr::null_mut(),
        REG_OPTION_VOLATILE | REG_OPTION_CREATE_LINK,
        &mut Disposition,
    );
    
    // The three diagnostics below are upstream's DPRINT1s. They were dropped in
    // translation, which left every one of these failures completely silent —
    // the link key simply never appears and nothing says why.
    if !NT_SUCCESS(Status) {
        crate::dbg_print!(
            b"CM: CmpLinkKeyToHive: couldn't create %S, Status = 0x%lx\r\n\0",
            LinkKeyName,
            Status
        );
        return 0u8;
    }

    if Disposition != REG_CREATED_NEW_KEY {
        crate::dbg_print!(b"CM: CmpLinkKeyToHive: %S already exists!\r\n\0", LinkKeyName);
        ZwClose(LinkKeyHandle);
        return 0u8;
    }

    RtlInitUnicodeString(&mut KeyName, TargetKeyName);
    Status = ZwSetValueKey(
        LinkKeyHandle,
        &raw mut CmSymbolicLinkValueName,
        0,
        REG_LINK,
        KeyName.Buffer as PVOID,
        KeyName.Length as ULONG,
    );

    ObCloseHandle(LinkKeyHandle, 0i8);

    if !NT_SUCCESS(Status) {
        crate::dbg_print!(
            b"CM: CmpLinkKeyToHive: couldn't create symbolic link for %S, Status = 0x%lx\r\n\0",
            TargetKeyName,
            Status
        );
        return 0u8;
    }

    1u8
}

#[no_mangle]
pub unsafe extern "system" fn CmpDeleteKeyObject(DeletedObject: PVOID) {
    let DeletedObject = opaque_ptr(DeletedObject);
    let KeyBody: PCM_KEY_BODY = DeletedObject as PCM_KEY_BODY;
    let mut Kcb: PCM_KEY_CONTROL_BLOCK;
    let mut KeyHandleCloseInfo: REG_KEY_HANDLE_CLOSE_INFORMATION = core::mem::zeroed();
    let mut PostOperationInfo: REG_POST_OPERATION_INFORMATION = core::mem::zeroed();
    let mut Status: NTSTATUS;

    PostOperationInfo.Object = KeyBody as PVOID;
    KeyHandleCloseInfo.Object = KeyBody as PVOID;
    Status = CmiCallRegisteredCallbacks(
        RegNtPreKeyHandleClose,
        core::ptr::addr_of_mut!(KeyHandleCloseInfo) as PVOID,
    );
    if !NT_SUCCESS(Status) {
        PostOperationInfo.Status = Status;
        CmiCallRegisteredCallbacks(
            RegNtPostKeyHandleClose,
            core::ptr::addr_of_mut!(PostOperationInfo) as PVOID,
        );
        return;
    }

    CmpLockRegistry();

    if (*KeyBody).Type == CM_KEY_BODY_TYPE {
        Kcb = (*KeyBody).KeyControlBlock;
        if !Kcb.is_null() {
            DelistKeyBodyFromKCB(KeyBody, (*KeyBody).KcbLocked);

            CmpDelayDerefKeyControlBlock(Kcb);
        }
    }

    CmpUnlockRegistry();

    PostOperationInfo.Status = STATUS_SUCCESS;
    CmiCallRegisteredCallbacks(
        RegNtPostKeyHandleClose,
        core::ptr::addr_of_mut!(PostOperationInfo) as PVOID,
    );
}

#[no_mangle]
pub unsafe extern "system" fn CmpCloseKeyObject(
    Process: ntkrnl_sys::ddk::PEPROCESS,
    Object: PVOID,
    GrantedAccess: ACCESS_MASK,
    ProcessHandleCount: ntkrnl_sys::ddk::ULONG_PTR,
    SystemHandleCount: ntkrnl_sys::ddk::ULONG_PTR,
) {
    let Process = opaque_ptr(Process);
    let Object = opaque_ptr(Object);
    let KeyBody = Object as PCM_KEY_BODY;

    if SystemHandleCount > 1 {
        return;
    }

    if (*KeyBody).Type == CM_KEY_BODY_TYPE {
        if (*KeyBody).NotifyBlock.is_null() {
            return;
        }

        // C is `ASSERT(FALSE)` ("This shouldn't happen yet") — a DEBUG-ONLY
        // marker. The translation turned it into an unconditional KeBugCheck(0),
        // which would kill a release machine on a case NT just walks past, with
        // the meaningless stop code 0 (UNDEFINED_BUG_CODE). Field manual §9.113.
        ntkrnl_sys::ASSERT!(false, "CmpCloseKeyObject: key body still has a NotifyBlock");
    }
}

#[no_mangle]
pub unsafe extern "system" fn CmpQueryKeyName(
    ObjectBody: *mut core::ffi::c_void,
    HasName: u8,
    ObjectNameInfo: POBJECT_NAME_INFORMATION,
    Length: u32,
    ReturnLength: *mut u32,
    PreviousMode: KPROCESSOR_MODE,
) -> i32 {
    let ObjectBody = opaque_ptr(ObjectBody);
    let ObjectNameInfo = opaque_ptr(ObjectNameInfo);
    let ReturnLength = opaque_ptr(ReturnLength);
    let KeyBody = ObjectBody as PCM_KEY_BODY;
    let Kcb = (*KeyBody).KeyControlBlock;

    CmpLockRegistry();

    CmpAcquireKcbLockShared(Kcb);

    if kcb_delete(Kcb) != 0 {
        CmpReleaseKcbLock(Kcb);
        CmpUnlockRegistry();
        return STATUS_KEY_DELETED;
    }

    let KeyName = CmpConstructName(Kcb);

    CmpReleaseKcbLock(Kcb);
    CmpUnlockRegistry();

    if KeyName.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    *ReturnLength = (*KeyName).Length as u32 + core::mem::size_of::<OBJECT_NAME_INFORMATION>() as u32 + core::mem::size_of::<u16>() as u32;

    let mut BytesToCopy = (*KeyName).Length as u32 + core::mem::size_of::<u16>() as u32;

    if (Length <= core::mem::size_of::<OBJECT_NAME_INFORMATION>() as u32) ||
        ((Length < *ReturnLength) && (BytesToCopy < core::mem::size_of::<u16>() as u32)) {
        ExFreePoolWithTag(KeyName as *mut core::ffi::c_void, TAG_CM);
        return STATUS_INFO_LENGTH_MISMATCH;
    }

    let mut Status = STATUS_SUCCESS;

    if Length < *ReturnLength {
        Status = STATUS_INFO_LENGTH_MISMATCH;
        BytesToCopy = Length - core::mem::size_of::<OBJECT_NAME_INFORMATION>() as u32;
    }

    BytesToCopy -= core::mem::size_of::<u16>() as u32;

    (*ObjectNameInfo).Name.Buffer = (ObjectNameInfo.offset(1)) as *mut u16;
    (*ObjectNameInfo).Name.MaximumLength = (*KeyName).Length;
    (*ObjectNameInfo).Name.Length = (*KeyName).Length;

    core::ptr::copy_nonoverlapping(
        (*KeyName).Buffer,
        (*ObjectNameInfo).Name.Buffer,
        BytesToCopy as usize / core::mem::size_of::<u16>(),
    );

    *((*ObjectNameInfo).Name.Buffer.offset((BytesToCopy / core::mem::size_of::<u16>() as u32) as isize)) = 0u16;

    ExFreePoolWithTag(KeyName as *mut core::ffi::c_void, TAG_CM);

    Status
}

unsafe extern "system" fn CmpDeleteKeyObjectCallback(Object: PVOID) {
    CmpDeleteKeyObject(Object);
}

unsafe extern "system" fn CmpQueryKeyNameCallback(
    Object: PVOID,
    HasObjectName: BOOLEAN,
    ObjectNameInfo: POBJECT_NAME_INFORMATION,
    Length: ULONG,
    ReturnLength: PULONG,
    AccessMode: KPROCESSOR_MODE,
) -> NTSTATUS {
    CmpQueryKeyName(Object, HasObjectName, ObjectNameInfo, Length, ReturnLength, AccessMode)
}

#[no_mangle]
pub unsafe extern "system" fn CmpInitHiveFromFile(
    HiveName: PCUNICODE_STRING,
    HiveFlags: u32,
    Hive: *mut PCMHIVE,
    New: PBOOLEAN,
    CheckFlags: u32,
) -> i32 {
    let HiveName = opaque_ptr_const(HiveName);
    let Hive = opaque_ptr(Hive);
    let New = opaque_ptr(New);
    let mut HiveDisposition: u32 = 0;
    let mut LogDisposition: u32 = 0;
    let mut FileHandle: *mut core::ffi::c_void = core::ptr::null_mut();
    let mut LogHandle: *mut core::ffi::c_void = core::ptr::null_mut();
    let mut Status: i32;
    let mut Operation: u32;
    let mut FileType: u32;
    let mut NewHive: PCMHIVE = core::ptr::null_mut();

    *Hive = core::ptr::null_mut();

    Status = CmpOpenHiveFiles(
        HiveName,
        w!(".LOG").as_ptr() as *const u16,
        &mut FileHandle,
        &mut LogHandle,
        &mut HiveDisposition,
        &mut LogDisposition,
        *New,
        0u8,
        1u8,
        core::ptr::null_mut(),
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }

    FileType = if !LogHandle.is_null() {
        1u32
    } else {
        0u32
    };

    if HiveDisposition == 0x00000002 {
        Operation = 0u32;
        *New = 1u8;
    } else {
        Operation = 2u32;
        *New = 0u8;
    }

    if CmpShareSystemHives != 0 {
        FileType = 0u32;
        if !LogHandle.is_null() {
            ZwClose(LogHandle);
            LogHandle = core::ptr::null_mut();
        }
    }

    if HvShutdownComplete != 0 {
        ZwClose(FileHandle);
        if !LogHandle.is_null() {
            ZwClose(LogHandle);
        }
        return 0xC0000189u32 as i32;
    }

    Status = CmpInitializeHive(
        &mut NewHive,
        Operation,
        HiveFlags,
        FileType,
        core::ptr::null_mut(),
        FileHandle,
        LogHandle,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        HiveName,
        CheckFlags,
    );
    if !NT_SUCCESS(Status) {
        ZwClose(FileHandle);
        if !LogHandle.is_null() {
            ZwClose(LogHandle);
        }
        return Status;
    }

    *Hive = NewHive;

    (*NewHive).FileFullPath.Buffer = ExAllocatePoolWithTag(
        PagedPool,
        (*HiveName).Length as usize,
        0x434D5441u32,
    ) as *mut u16;
    if !(*NewHive).FileFullPath.Buffer.is_null() {
        core::ptr::copy_nonoverlapping(
            (*HiveName).Buffer,
            (*NewHive).FileFullPath.Buffer,
            (*HiveName).Length as usize / 2,
        );
        (*NewHive).FileFullPath.Length = (*HiveName).Length;
        (*NewHive).FileFullPath.MaximumLength = (*HiveName).Length;
    }

    STATUS_SUCCESS
}

unsafe fn CmpSetSystemValues(LoaderBlock: PLOADER_PARAMETER_BLOCK) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut KeyHandle: HANDLE = core::ptr::null_mut();
    let mut KeyName: _UNICODE_STRING = core::mem::zeroed();
    let mut ValueName: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };

    RtlInitUnicodeString(
        &mut KeyName,
        w!("\\REGISTRY\\MACHINE\\SYSTEM\\CurrentControlSet\\Control").as_ptr(),
    );
    
    ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as u32;
    ObjectAttributes.RootDirectory = core::ptr::null_mut();
    ObjectAttributes.ObjectName = &mut KeyName;
    ObjectAttributes.Attributes = OBJ_CASE_INSENSITIVE;
    ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
    ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

    Status = NtOpenKey(&mut KeyHandle, KEY_WRITE, &mut ObjectAttributes);
    if !NT_SUCCESS(Status) {
        return Status;
    }

    RtlInitUnicodeString(&mut KeyName, w!("SystemStartOptions").as_ptr());
    Status = NtSetValueKey(
        KeyHandle,
        &mut KeyName,
        0,
        REG_SZ,
        CmpLoadOptions.Buffer as *mut core::ffi::c_void,
        CmpLoadOptions.Length as u32,
    );
    if !NT_SUCCESS(Status) {
        NtClose(KeyHandle);
        return Status;
    }

    RtlInitUnicodeString(&mut KeyName, w!("SystemBootDevice").as_ptr());
    RtlCreateUnicodeStringFromAsciiz(&mut ValueName, (*LoaderBlock).ArcBootDeviceName);
    Status = NtSetValueKey(
        KeyHandle,
        &mut KeyName,
        0,
        REG_SZ,
        ValueName.Buffer as *mut core::ffi::c_void,
        ValueName.Length as u32,
    );

    RtlFreeUnicodeString(&mut ValueName);

    // vibeos (no C counterpart): publish this boot's tripwire config next to
    // SystemStartOptions, so a user-mode harness hook can read per-boot knobs too
    // (winlogon's QUIET). The SYSTEM hive can persist across boots, so a boot
    // without one deletes the value rather than leaving the last boot's behind.
    // Best effort: neither result changes what this function returns.
    {
        let mut Raw = [0u8; 256];
        let mut Wide = [0u16; 257];
        let n = VibeTripwireRead(Raw.as_mut_ptr(), Raw.len());
        for i in 0..n {
            Wide[i] = Raw[i] as u16;
        }
        RtlInitUnicodeString(&mut KeyName, w!("VibeTripwire").as_ptr());
        if n != 0 {
            NtSetValueKey(
                KeyHandle,
                &mut KeyName,
                0,
                REG_SZ,
                Wide.as_mut_ptr() as *mut core::ffi::c_void,
                ((n + 1) * core::mem::size_of::<WCHAR>()) as u32,
            );
        } else {
            NtDeleteValueKey(KeyHandle, &mut KeyName);
        }
    }

    NtClose(KeyHandle);
    Status
}

unsafe fn CmpCreateHardwareProfile(ControlSetHandle: HANDLE) -> NTSTATUS {
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut KeyName: _UNICODE_STRING = core::mem::zeroed();
    let mut ProfilesHandle: HANDLE = core::ptr::null_mut();
    let mut ProfileHandle: HANDLE = core::ptr::null_mut();
    let mut Disposition: ULONG = 0;
    let mut Status: NTSTATUS;

    RtlInitUnicodeString(&mut KeyName, w!("Hardware Profiles").as_ptr());
    ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
    ObjectAttributes.RootDirectory = ControlSetHandle;
    ObjectAttributes.ObjectName = &mut KeyName;
    ObjectAttributes.Attributes = OBJ_CASE_INSENSITIVE;
    ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
    ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();
    
    Status = NtCreateKey(
        &mut ProfilesHandle,
        KEY_ALL_ACCESS,
        &mut ObjectAttributes,
        0,
        core::ptr::null_mut(),
        0,
        &mut Disposition,
    );
    if !NT_SUCCESS(Status) {
        crate::dbg_print!(b"Creating the Hardware Profile key failed\n\0");
        ProfilesHandle = core::ptr::null_mut();
        ProfileHandle = core::ptr::null_mut();
        if !ProfilesHandle.is_null() {
            NtClose(ProfilesHandle);
        }
        if !ProfileHandle.is_null() {
            NtClose(ProfileHandle);
        }
        return Status;
    }

    RtlInitUnicodeString(&mut KeyName, w!("0000").as_ptr());
    ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
    ObjectAttributes.RootDirectory = ProfilesHandle;
    ObjectAttributes.ObjectName = &mut KeyName;
    ObjectAttributes.Attributes = OBJ_CASE_INSENSITIVE;
    ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
    ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();
    
    Status = NtCreateKey(
        &mut ProfileHandle,
        KEY_ALL_ACCESS,
        &mut ObjectAttributes,
        0,
        core::ptr::null_mut(),
        0,
        &mut Disposition,
    );
    if !NT_SUCCESS(Status) {
        crate::dbg_print!(b"Creating the Hardware Profile\\0000 key failed\n\0");
        if !ProfilesHandle.is_null() {
            NtClose(ProfilesHandle);
        }
        if !ProfileHandle.is_null() {
            NtClose(ProfileHandle);
        }
        return Status;
    }

    if !ProfilesHandle.is_null() {
        NtClose(ProfilesHandle);
    }
    if !ProfileHandle.is_null() {
        NtClose(ProfileHandle);
    }

    Status
}

unsafe fn CmpCreateControlSet(LoaderBlock: PLOADER_PARAMETER_BLOCK) -> NTSTATUS {
    let mut ConfigName: _UNICODE_STRING = core::mem::zeroed();
    let mut SelectName: _UNICODE_STRING = core::mem::zeroed();
    let mut KeyName: _UNICODE_STRING = core::mem::zeroed();
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut ValueInfoBuffer: [CHAR; 128] = [0; 128];
    let mut ValueInfo: PKEY_VALUE_FULL_INFORMATION;
    let mut UnicodeBuffer: [WCHAR; 128] = [0; 128];
    let mut SelectHandle: HANDLE = core::ptr::null_mut();
    let mut KeyHandle: HANDLE = core::ptr::null_mut();
    let mut ConfigHandle: HANDLE = core::ptr::null_mut();
    let mut ProfileHandle: HANDLE = core::ptr::null_mut();
    let mut ParentHandle: HANDLE = core::ptr::null_mut();
    let mut ControlSet: ULONG;
    let HwProfile: ULONG;
    let mut Status: NTSTATUS;
    let mut ResultLength: ULONG = 0;
    let mut Disposition: ULONG = 0;
    let LoaderExtension: PLOADER_PARAMETER_EXTENSION;

    RtlInitUnicodeString(&mut ConfigName, w!("Control\\IDConfigDB").as_ptr());
    RtlInitUnicodeString(&mut SelectName, w!("\\Registry\\Machine\\System\\Select").as_ptr());

    if (*LoaderBlock).RegistryBase == core::ptr::null_mut() {
        RtlInitUnicodeString(&mut KeyName, w!("\\Registry\\Machine\\System\\ControlSet001").as_ptr());
        ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
        ObjectAttributes.RootDirectory = core::ptr::null_mut();
        ObjectAttributes.ObjectName = &mut KeyName;
        ObjectAttributes.Attributes = OBJ_CASE_INSENSITIVE;
        ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
        ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();
        Status = NtCreateKey(&mut KeyHandle, KEY_ALL_ACCESS, &mut ObjectAttributes, 0, core::ptr::null_mut(), 0, &mut Disposition);
        if !NT_SUCCESS(Status) {
            crate::dbg_print!(b"Failed to create ControlSet001 key: 0x%lx\n\0", Status);
            Status = Status;
            KeyHandle = core::ptr::null_mut();
            'Cleanup: loop {
                if !SelectHandle.is_null() { NtClose(SelectHandle); }
                if !KeyHandle.is_null() { NtClose(KeyHandle); }
                if !ConfigHandle.is_null() { NtClose(ConfigHandle); }
                if !ProfileHandle.is_null() { NtClose(ProfileHandle); }
                if !ParentHandle.is_null() { NtClose(ParentHandle); }
                return Status;
            }
        }

        Status = CmpCreateHardwareProfile(KeyHandle);
        if !NT_SUCCESS(Status) {
            crate::dbg_print!(b"Failed to create Hardware profile keys: 0x%lx\n\0", Status);
            'Cleanup: loop {
                if !SelectHandle.is_null() { NtClose(SelectHandle); }
                if !KeyHandle.is_null() { NtClose(KeyHandle); }
                if !ConfigHandle.is_null() { NtClose(ConfigHandle); }
                if !ProfileHandle.is_null() { NtClose(ProfileHandle); }
                if !ParentHandle.is_null() { NtClose(ParentHandle); }
                return Status;
            }
        }

        ControlSet = 1;
    } else {
        ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
        ObjectAttributes.RootDirectory = core::ptr::null_mut();
        ObjectAttributes.ObjectName = &mut SelectName;
        ObjectAttributes.Attributes = OBJ_CASE_INSENSITIVE;
        ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
        ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();
        Status = NtOpenKey(&mut SelectHandle, KEY_READ, &mut ObjectAttributes);
        if !NT_SUCCESS(Status) {
            crate::dbg_print!(b"Failed to open select key: 0x%lx\n\0", Status);
            SelectHandle = core::ptr::null_mut();
            'Cleanup: loop {
                if !SelectHandle.is_null() { NtClose(SelectHandle); }
                if !KeyHandle.is_null() { NtClose(KeyHandle); }
                if !ConfigHandle.is_null() { NtClose(ConfigHandle); }
                if !ProfileHandle.is_null() { NtClose(ProfileHandle); }
                if !ParentHandle.is_null() { NtClose(ParentHandle); }
                return Status;
            }
        }

        RtlInitUnicodeString(&mut KeyName, w!("Current").as_ptr());
        Status = NtQueryValueKey(SelectHandle, &mut KeyName, KeyValueFullInformation, ValueInfoBuffer.as_mut_ptr() as *mut core::ffi::c_void, core::mem::size_of_val(&ValueInfoBuffer) as ULONG, &mut ResultLength);
        if !NT_SUCCESS(Status) {
            crate::dbg_print!(b"Failed to open the Current value: 0x%lx\n\0", Status);
            'Cleanup: loop {
                if !SelectHandle.is_null() { NtClose(SelectHandle); }
                if !KeyHandle.is_null() { NtClose(KeyHandle); }
                if !ConfigHandle.is_null() { NtClose(ConfigHandle); }
                if !ProfileHandle.is_null() { NtClose(ProfileHandle); }
                if !ParentHandle.is_null() { NtClose(ParentHandle); }
                return Status;
            }
        }

        ValueInfo = ValueInfoBuffer.as_mut_ptr() as PKEY_VALUE_FULL_INFORMATION;
        ControlSet = (ValueInfoBuffer.as_ptr().offset((*ValueInfo).DataOffset as isize) as *const ULONG).read_unaligned();
    }

    RtlInitUnicodeString(&mut KeyName, w!("\\Registry\\Machine\\System\\CurrentControlSet").as_ptr());
    ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
    ObjectAttributes.RootDirectory = core::ptr::null_mut();
    ObjectAttributes.ObjectName = &mut KeyName;
    ObjectAttributes.Attributes = OBJ_CASE_INSENSITIVE;
    ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
    ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();
    Status = NtCreateKey(&mut KeyHandle, KEY_CREATE_LINK, &mut ObjectAttributes, 0, core::ptr::null_mut(), REG_OPTION_VOLATILE | REG_OPTION_CREATE_LINK, &mut Disposition);
    if !NT_SUCCESS(Status) {
        KeyHandle = core::ptr::null_mut();
        'Cleanup: loop {
            if !SelectHandle.is_null() { NtClose(SelectHandle); }
            if !KeyHandle.is_null() { NtClose(KeyHandle); }
            if !ConfigHandle.is_null() { NtClose(ConfigHandle); }
            if !ProfileHandle.is_null() { NtClose(ProfileHandle); }
            if !ParentHandle.is_null() { NtClose(ParentHandle); }
            return Status;
        }
    }

    // `\REGISTRY`, not `\Registry`, and the reason is that this string is a symlink
    // TARGET. It gets handed straight back to ObpLookupObjectName as a REPARSE, which
    // matches the leading component in the ROOT OBJECT DIRECTORY against the entry
    // CmpCreateRegistryRoot made — spelled `REGISTRY`. That compare is case-SENSITIVE
    // unless the caller passed OBJ_CASE_INSENSITIVE, because the Key object type is not
    // case-insensitive (ObTypes.c:217 asserts exactly that against WS03). A target is
    // stored verbatim and replayed for whoever opens the link, so it has to be spelled
    // for the worst caller, not the one we happen to have. Only the first component is
    // matched this way; everything after it is CM's own subkey matching, which is
    // case-insensitive regardless — the rest is upper-cased purely to match the other
    // registry paths NT builds.
    Status = crate::rtl_string_cb_printf_w!(UnicodeBuffer.as_mut_ptr(), core::mem::size_of_val(&UnicodeBuffer), crate::wstr!("\\REGISTRY\\MACHINE\\SYSTEM\\CONTROLSET%03ld"), ControlSet);
    if !NT_SUCCESS(Status) {
        'Cleanup: loop {
            if !SelectHandle.is_null() { NtClose(SelectHandle); }
            if !KeyHandle.is_null() { NtClose(KeyHandle); }
            if !ConfigHandle.is_null() { NtClose(ConfigHandle); }
            if !ProfileHandle.is_null() { NtClose(ProfileHandle); }
            if !ParentHandle.is_null() { NtClose(ParentHandle); }
            return Status;
        }
    }

    RtlInitUnicodeString(&mut KeyName, UnicodeBuffer.as_ptr());

    Status = NtSetValueKey(KeyHandle, &raw mut CmSymbolicLinkValueName, 0, REG_LINK, KeyName.Buffer as *mut core::ffi::c_void, KeyName.Length as ULONG);
    if !NT_SUCCESS(Status) {
        'Cleanup: loop {
            if !SelectHandle.is_null() { NtClose(SelectHandle); }
            if !KeyHandle.is_null() { NtClose(KeyHandle); }
            if !ConfigHandle.is_null() { NtClose(ConfigHandle); }
            if !ProfileHandle.is_null() { NtClose(ProfileHandle); }
            if !ParentHandle.is_null() { NtClose(ParentHandle); }
            return Status;
        }
    }

    ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
    ObjectAttributes.RootDirectory = KeyHandle;
    ObjectAttributes.ObjectName = &mut ConfigName;
    ObjectAttributes.Attributes = OBJ_CASE_INSENSITIVE;
    ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
    ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();
    Status = NtOpenKey(&mut ConfigHandle, KEY_READ, &mut ObjectAttributes);

    if !NT_SUCCESS(Status) {
        Status = STATUS_SUCCESS;
        ConfigHandle = core::ptr::null_mut();
        'Cleanup: loop {
            if !SelectHandle.is_null() { NtClose(SelectHandle); }
            if !KeyHandle.is_null() { NtClose(KeyHandle); }
            if !ConfigHandle.is_null() { NtClose(ConfigHandle); }
            if !ProfileHandle.is_null() { NtClose(ProfileHandle); }
            if !ParentHandle.is_null() { NtClose(ParentHandle); }
            return Status;
        }
    }

    let HwProfile: ULONG = if (*LoaderBlock).RegistryBase == core::ptr::null_mut() {
        0
    } else {
        RtlInitUnicodeString(&mut KeyName, w!("CurrentConfig").as_ptr());
        Status = NtQueryValueKey(ConfigHandle, &mut KeyName, KeyValueFullInformation, ValueInfoBuffer.as_mut_ptr() as *mut core::ffi::c_void, core::mem::size_of_val(&ValueInfoBuffer) as ULONG, &mut ResultLength);

        ValueInfo = ValueInfoBuffer.as_mut_ptr() as PKEY_VALUE_FULL_INFORMATION;

        if !NT_SUCCESS(Status) || (*ValueInfo).Type != REG_DWORD {
            Status = STATUS_SUCCESS;
            'Cleanup: loop {
                if !SelectHandle.is_null() { NtClose(SelectHandle); }
                if !KeyHandle.is_null() { NtClose(KeyHandle); }
                if !ConfigHandle.is_null() { NtClose(ConfigHandle); }
                if !ProfileHandle.is_null() { NtClose(ProfileHandle); }
                if !ParentHandle.is_null() { NtClose(ParentHandle); }
                return Status;
            }
        }

        (ValueInfoBuffer.as_ptr().offset((*ValueInfo).DataOffset as isize) as *const ULONG).read_unaligned()
    };

    RtlInitUnicodeString(&mut KeyName, w!("\\Registry\\Machine\\System\\CurrentControlSet\\Hardware Profiles").as_ptr());
    ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
    ObjectAttributes.RootDirectory = core::ptr::null_mut();
    ObjectAttributes.ObjectName = &mut KeyName;
    ObjectAttributes.Attributes = OBJ_CASE_INSENSITIVE;
    ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
    ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();
    Status = NtOpenKey(&mut ParentHandle, KEY_READ, &mut ObjectAttributes);
    if !NT_SUCCESS(Status) {
        Status = STATUS_SUCCESS;
        ParentHandle = core::ptr::null_mut();
        'Cleanup: loop {
            if !SelectHandle.is_null() { NtClose(SelectHandle); }
            if !KeyHandle.is_null() { NtClose(KeyHandle); }
            if !ConfigHandle.is_null() { NtClose(ConfigHandle); }
            if !ProfileHandle.is_null() { NtClose(ProfileHandle); }
            if !ParentHandle.is_null() { NtClose(ParentHandle); }
            return Status;
        }
    }

    crate::rtl_string_cb_printf_w!(UnicodeBuffer.as_mut_ptr(), core::mem::size_of_val(&UnicodeBuffer), crate::wstr!("%04ld"), HwProfile);
    RtlInitUnicodeString(&mut KeyName, UnicodeBuffer.as_ptr());

    ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
    ObjectAttributes.RootDirectory = ParentHandle;
    ObjectAttributes.ObjectName = &mut KeyName;
    ObjectAttributes.Attributes = OBJ_CASE_INSENSITIVE;
    ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
    ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();
    Status = NtOpenKey(&mut ProfileHandle, KEY_READ | KEY_WRITE, &mut ObjectAttributes);
    if !NT_SUCCESS(Status) {
        Status = STATUS_SUCCESS;
        ProfileHandle = core::ptr::null_mut();
        'Cleanup: loop {
            if !SelectHandle.is_null() { NtClose(SelectHandle); }
            if !KeyHandle.is_null() { NtClose(KeyHandle); }
            if !ConfigHandle.is_null() { NtClose(ConfigHandle); }
            if !ProfileHandle.is_null() { NtClose(ProfileHandle); }
            if !ParentHandle.is_null() { NtClose(ParentHandle); }
            return Status;
        }
    }

    LoaderExtension = (*LoaderBlock).Extension;
    if !LoaderExtension.is_null() {
    }

    RtlInitUnicodeString(&mut KeyName, w!("\\Registry\\Machine\\System\\CurrentControlSet\\Hardware Profiles\\Current").as_ptr());
    ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
    ObjectAttributes.RootDirectory = core::ptr::null_mut();
    ObjectAttributes.ObjectName = &mut KeyName;
    ObjectAttributes.Attributes = OBJ_CASE_INSENSITIVE;
    ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
    ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();
    Status = NtCreateKey(&mut KeyHandle, KEY_CREATE_LINK, &mut ObjectAttributes, 0, core::ptr::null_mut(), REG_OPTION_VOLATILE | REG_OPTION_CREATE_LINK, &mut Disposition);
    if NT_SUCCESS(Status) {
        crate::rtl_string_cb_printf_w!(
            UnicodeBuffer.as_mut_ptr(),
            core::mem::size_of_val(&UnicodeBuffer),
            // A symlink target again — see the CONTROLSET%03ld comment above for why the
            // leading component must be `\REGISTRY`.
            crate::wstr!("\\REGISTRY\\MACHINE\\SYSTEM\\CURRENTCONTROLSET\\HARDWARE PROFILES\\%04ld"),
            HwProfile,
        );
        RtlInitUnicodeString(&mut KeyName, UnicodeBuffer.as_ptr());

        Status = NtSetValueKey(KeyHandle, &raw mut CmSymbolicLinkValueName, 0, REG_LINK, KeyName.Buffer as *mut core::ffi::c_void, KeyName.Length as ULONG);
    }

    Status = STATUS_SUCCESS;

    if !SelectHandle.is_null() { NtClose(SelectHandle); }
    if !KeyHandle.is_null() { NtClose(KeyHandle); }
    if !ConfigHandle.is_null() { NtClose(ConfigHandle); }
    if !ProfileHandle.is_null() { NtClose(ProfileHandle); }
    if !ParentHandle.is_null() { NtClose(ParentHandle); }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn CmpLinkHiveToMaster(
    LinkName: PUNICODE_STRING,
    RootDirectory: HANDLE,
    RegistryHive: PCMHIVE,
    Allocate: BOOLEAN,
    SecurityDescriptor: PSECURITY_DESCRIPTOR,
) -> NTSTATUS {
    let LinkName = opaque_ptr(LinkName);
    let RootDirectory = opaque_ptr(RootDirectory);
    let RegistryHive = opaque_ptr(RegistryHive);
    let SecurityDescriptor = opaque_ptr(SecurityDescriptor);
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut Status: NTSTATUS;
    let mut ParseContext: CM_PARSE_CONTEXT = core::mem::zeroed();
    let mut KeyHandle: HANDLE = core::mem::zeroed();
    let mut KeyBody: PCM_KEY_BODY = core::ptr::null_mut();

    ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
    ObjectAttributes.RootDirectory = RootDirectory;
    ObjectAttributes.Attributes = OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE;
    ObjectAttributes.ObjectName = LinkName;
    ObjectAttributes.SecurityDescriptor = SecurityDescriptor as *mut core::ffi::c_void;
    ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

    ParseContext.CreateLink = 1u8;
    ParseContext.CreateOperation = 1u8;
    ParseContext.ChildHive.KeyHive = core::ptr::addr_of_mut!((*RegistryHive).Hive);

    if Allocate != 0 {
        ParseContext.ChildHive.KeyCell = HCELL_NIL;
    } else {
        ParseContext.ChildHive.KeyCell = (*(*RegistryHive).Hive.BaseBlock).RootCell;
    }

    Status = ObOpenObjectByName(
        &mut ObjectAttributes,
        CmpKeyObjectType,
        0i32 as KPROCESSOR_MODE,
        core::ptr::null_mut(),
        KEY_READ | KEY_WRITE,
        &mut ParseContext as *mut CM_PARSE_CONTEXT as PVOID,
        &mut KeyHandle,
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }

    (*RegistryHive).Hive.DirtyFlag = 0u8;

    Status = ObReferenceObjectByHandle(
        KeyHandle,
        0,
        CmpKeyObjectType,
        0i32 as KPROCESSOR_MODE,
        &mut KeyBody as *mut PCM_KEY_BODY as *mut PVOID,
        core::ptr::null_mut(),
    );

    ZwClose(KeyHandle);
    return STATUS_SUCCESS;
}

#[no_mangle]
pub unsafe extern "system" fn CmpInitializeSystemHive(
    LoaderBlock: PLOADER_PARAMETER_BLOCK,
) -> BOOLEAN {
    let LoaderBlock = opaque_ptr(LoaderBlock);
    static mut HIVE_NAME: _UNICODE_STRING = _UNICODE_STRING {
        Length: 14,
        MaximumLength: 16,
        Buffer: w!("SYSTEM").as_ptr() as *mut u16,
    };

    let mut HiveBase: PVOID;
    let mut LoadString: ANSI_STRING = core::mem::zeroed();
    let mut Buffer: PVOID;
    let mut Length: ULONG;
    let mut Status: NTSTATUS;
    let mut KeyName: _UNICODE_STRING = core::mem::zeroed();
    let mut SystemHive: PCMHIVE = core::ptr::null_mut();
    let mut SecurityDescriptor: PSECURITY_DESCRIPTOR;

    RtlInitAnsiString(
        core::ptr::addr_of_mut!(LoadString),
        (*LoaderBlock).LoadOptions,
    );

    Length = LoadString.Length as ULONG * core::mem::size_of::<WCHAR>() as ULONG
        + core::mem::size_of::<WCHAR>() as ULONG;
    Buffer = ExAllocatePoolWithTag(PagedPool, Length as usize, TAG_CM);
    if Buffer.is_null() {
        KeBugCheckEx(
            BAD_SYSTEM_CONFIG_INFO,
            3,
            1,
            LoaderBlock as ULONG_PTR,
            0,
        );
    }

    RtlInitEmptyUnicodeString(
        core::ptr::addr_of_mut!(CmpLoadOptions),
        Buffer as *mut u16,
        Length as USHORT,
    );

    Status = RtlAnsiStringToUnicodeString(
        core::ptr::addr_of_mut!(CmpLoadOptions),
        core::ptr::addr_of_mut!(LoadString),
        0u8,
    );
    if !NT_SUCCESS(Status) {
        return 0u8;
    }

    (*CmpLoadOptions.Buffer.offset(LoadString.Length as isize)) = 0u16;
    CmpLoadOptions.Length += core::mem::size_of::<WCHAR>() as USHORT;

    HiveBase = (*LoaderBlock).RegistryBase;
    ARM64_UART_TRACE!(b"[CM] SysHive: HiveBase=\0".as_ptr());
    if !HiveBase.is_null() {
        ARM64_UART_TRACE!(b"non-null\0".as_ptr());
    } else {
        ARM64_UART_TRACE!(b"NULL\0".as_ptr());
    }
    ARM64_UART_TRACE!(b"\r\n\0".as_ptr());

    ARM64_UART_TRACE!(b"[CM] SysHive: CmpInitializeHive\r\n\0".as_ptr());
    Status = CmpInitializeHive(
        core::ptr::addr_of_mut!(SystemHive),
        if !HiveBase.is_null() {
            HINIT_MEMORY
        } else {
            HINIT_CREATE
        },
        HIVE_NOLAZYFLUSH,
        HFILE_TYPE_ALTERNATE,
        HiveBase,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(HIVE_NAME),
        if !HiveBase.is_null() {
            CM_CHECK_REGISTRY_PURGE_VOLATILES
        } else {
            CM_CHECK_REGISTRY_DONT_PURGE_VOLATILES
        },
    );
    if !NT_SUCCESS(Status) {
        return 0u8;
    }

    ARM64_UART_TRACE!(b"[CM] SysHive: CmpInitializeHive done\r\n\0".as_ptr());

    if RtlCreateUnicodeString(
        core::ptr::addr_of_mut!((*SystemHive).FileFullPath),
        w!("\\SystemRoot\\System32\\Config\\SYSTEM").as_ptr(),
    ) == 0
    {
        return 0u8;
    }

    if !HiveBase.is_null() && CmpShareSystemHives != 0 {
        (*SystemHive).Hive.HiveFlags = HIVE_VOLATILE;
    }

    CmpBootType = (*(*SystemHive).Hive.BaseBlock).BootType;

    if CmSelfHeal == 0 {
        CmpSelfHeal = 0u8;
        if (CmpBootType & HBOOT_TYPE_SELF_HEAL) != 0 {
            KeBugCheckEx(
                BAD_SYSTEM_CONFIG_INFO,
                3,
                3,
                SystemHive as ULONG_PTR,
                0,
            );
        }
    }

    SecurityDescriptor = CmpHiveRootSecurityDescriptor();

    ARM64_UART_TRACE!(b"[CM] SysHive: CmpLinkHiveToMaster\r\n\0".as_ptr());
    RtlInitUnicodeString(
        core::ptr::addr_of_mut!(KeyName),
        w!("\\Registry\\Machine\\SYSTEM").as_ptr(),
    );
    Status = CmpLinkHiveToMaster(
        core::ptr::addr_of_mut!(KeyName),
        core::ptr::null_mut(),
        SystemHive,
        if HiveBase.is_null() { 1u8 } else { 0u8 },
        SecurityDescriptor,
    );

    ExFreePoolWithTag(SecurityDescriptor as PVOID, TAG_CMSD);
    if !NT_SUCCESS(Status) {
        return 0u8;
    }

    (*(*(&raw mut CmpMachineHiveList)).as_mut_ptr().offset(3)).CmHive = SystemHive;

    1u8
}

#[no_mangle]
pub unsafe extern "system" fn CmpCreateObjectTypes() -> NTSTATUS {
    let mut ObjectTypeInitializer: OBJECT_TYPE_INITIALIZER = core::mem::zeroed();
    let mut Name: _UNICODE_STRING = core::mem::zeroed();
    let CmpKeyMapping: _GENERIC_MAPPING = _GENERIC_MAPPING {
        GenericRead: KEY_READ,
        GenericWrite: KEY_WRITE,
        GenericExecute: KEY_EXECUTE,
        GenericAll: KEY_ALL_ACCESS,
    };

    RtlZeroMemory(
        &mut ObjectTypeInitializer as *mut _ as *mut core::ffi::c_void,
        core::mem::size_of::<OBJECT_TYPE_INITIALIZER>(),
    );
    RtlInitUnicodeString(&mut Name, w!("Key").as_ptr());
    ObjectTypeInitializer.Length = core::mem::size_of::<OBJECT_TYPE_INITIALIZER>() as u16;
    ObjectTypeInitializer.DefaultPagedPoolCharge = core::mem::size_of::<CM_KEY_BODY>() as u32;
    ObjectTypeInitializer.GenericMapping = CmpKeyMapping;
    ObjectTypeInitializer.PoolType = PagedPool;
    ObjectTypeInitializer.ValidAccessMask = KEY_ALL_ACCESS;
    ObjectTypeInitializer.UseDefaultObject = 1u8;
    ObjectTypeInitializer.DeleteProcedure = Some(CmpDeleteKeyObjectCallback);
    ObjectTypeInitializer.ParseProcedure = Some(CmpParseKey);
    ObjectTypeInitializer.SecurityProcedure = Some(CmpSecurityMethod);
    ObjectTypeInitializer.QueryNameProcedure = Some(CmpQueryKeyNameCallback);
    ObjectTypeInitializer.CloseProcedure = Some(CmpCloseKeyObject);
    ObjectTypeInitializer.SecurityRequired = 1u8;
    ObjectTypeInitializer.InvalidAttributes = OBJ_EXCLUSIVE | OBJ_PERMANENT;
    // Deliberately NO `CaseInsensitive = TRUE` here. Setting it looks like the fix
    // for mixed-case registry paths and is not: ObTypes.c:217 checks this exact
    // field against WS03 and expects FALSE, so a Key open really is case-sensitive
    // at the object-directory layer. That is why every registry path NT builds is
    // spelled `\REGISTRY\...` — see CmpInitializeHiveList.

    ObCreateObjectType(&mut Name, &mut ObjectTypeInitializer, core::ptr::null_mut(), &raw mut CmpKeyObjectType)
}

#[no_mangle]
pub unsafe extern "system" fn CmpCreateRootNode(
    Hive: PHHIVE,
    Name: PCWSTR,
    Index: PHCELL_INDEX,
) -> u8 {
    let Hive = opaque_ptr(Hive);
    let Name = opaque_ptr_const(Name);
    let Index = opaque_ptr(Index);
    let mut KeyName: _UNICODE_STRING = core::mem::zeroed();
    let KeyCell: PCM_KEY_NODE;

    RtlInitUnicodeString(&mut KeyName, Name);
    *Index = HvAllocateCell(
        Hive,
        core::mem::offset_of!(CM_KEY_NODE, Name) as u32 + CmpNameSize(Hive, &KeyName) as u32,
        0,
        0xFFFFFFFF,
    );
    if *Index == 0xFFFFFFFF {
        return 0u8;
    }

    (*(*Hive).BaseBlock).RootCell = *Index;
    KeyCell = HvGetCell(Hive, *Index) as PCM_KEY_NODE;
    if KeyCell.is_null() {
        return 0u8;
    }

    (*KeyCell).Signature = 0x6B6E;
    (*KeyCell).Flags = (KEY_HIVE_ENTRY | KEY_NO_DELETE) as u16;
    KeQuerySystemTime(core::ptr::addr_of_mut!((*KeyCell).LastWriteTime) as *mut LARGE_INTEGER);
    (*KeyCell).Parent = 0xFFFFFFFF;
    (*KeyCell).SubKeyCounts[0] = 0;
    (*KeyCell).SubKeyCounts[1] = 0;
    (*KeyCell).__bindgen_anon_1.__bindgen_anon_1.SubKeyLists[0] = 0xFFFFFFFF;
    (*KeyCell).__bindgen_anon_1.__bindgen_anon_1.SubKeyLists[1] = 0xFFFFFFFF;
    (*KeyCell).__bindgen_anon_1.__bindgen_anon_1.ValueList.Count = 0;
    (*KeyCell).__bindgen_anon_1.__bindgen_anon_1.ValueList.List = 0xFFFFFFFF;
    (*KeyCell).Security = 0xFFFFFFFF;
    (*KeyCell).Class = 0xFFFFFFFF;
    (*KeyCell).ClassLength = 0;
    (*KeyCell).MaxNameLen = 0;
    (*KeyCell).MaxClassLen = 0;
    (*KeyCell).MaxValueNameLen = 0;
    (*KeyCell).MaxValueDataLen = 0;

    (*KeyCell).NameLength = CmpCopyName(Hive, core::ptr::addr_of_mut!((*KeyCell).Name) as *mut u16, &KeyName);

    if (*KeyCell).NameLength < KeyName.Length {
        (*KeyCell).Flags |= KEY_COMP_NAME as u16;
    }

    HvReleaseCell(Hive, *Index);
    return 1u8;
}

#[no_mangle]
pub unsafe extern "system" fn CmpCreateRegistryRoot() -> u8 {
    let mut KeyName: _UNICODE_STRING = core::mem::zeroed();
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut RootKey: PCM_KEY_BODY = core::ptr::null_mut();
    let mut RootIndex: HCELL_INDEX = 0;
    let mut Status: NTSTATUS;
    let KeyCell: PCM_KEY_NODE;
    let SecurityDescriptor: PSECURITY_DESCRIPTOR;
    let Kcb: PCM_KEY_CONTROL_BLOCK;

    if CmpCreateRootNode(
        &mut (*CmiVolatileHive).Hive,
        w!("REGISTRY").as_ptr(),
        &mut RootIndex,
    ) == 0
    {
        return 0u8;
    }

    RtlInitUnicodeString(&mut KeyName, w!("\\REGISTRY").as_ptr());
    SecurityDescriptor = CmpHiveRootSecurityDescriptor();
    
    ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as u32;
    ObjectAttributes.RootDirectory = core::ptr::null_mut();
    ObjectAttributes.ObjectName = &mut KeyName;
    ObjectAttributes.Attributes = OBJ_CASE_INSENSITIVE;
    ObjectAttributes.SecurityDescriptor = SecurityDescriptor as *mut core::ffi::c_void;
    ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

    Status = ObCreateObject(
        0i8,
        CmpKeyObjectType,
        &mut ObjectAttributes,
        0i8,
        core::ptr::null_mut(),
        core::mem::size_of::<CM_KEY_BODY>() as u32,
        0,
        0,
        &mut RootKey as *mut _ as *mut *mut core::ffi::c_void,
    );
    ExFreePoolWithTag(SecurityDescriptor as *mut core::ffi::c_void, TAG_CMSD);
    if !NT_SUCCESS(Status) {
        return 0u8;
    }

    KeyCell = HvGetCell(&mut (*CmiVolatileHive).Hive, RootIndex) as PCM_KEY_NODE;
    if KeyCell.is_null() {
        ObDereferenceObject(RootKey as *mut core::ffi::c_void);
        return 0u8;
    }

    RtlInitUnicodeString(&mut KeyName, w!("\\REGISTRY").as_ptr());
    Kcb = CmpCreateKeyControlBlock(
        &mut (*CmiVolatileHive).Hive,
        RootIndex,
        KeyCell,
        core::ptr::null_mut(),
        0,
        &mut KeyName,
    );
    if Kcb.is_null() {
        ObDereferenceObject(RootKey as *mut core::ffi::c_void);
        return 0u8;
    }

    (*RootKey).KeyControlBlock = Kcb;
    (*RootKey).Type = CM_KEY_BODY_TYPE;
    (*RootKey).NotifyBlock = core::ptr::null_mut();
    core::ptr::write_volatile(
        core::ptr::addr_of_mut!((*RootKey).ProcessID),
        opaque_ptr(PsGetCurrentProcessId()),
    );
    (*RootKey).KcbLocked = 0u8;

    EnlistKeyBodyWithKCB(RootKey, 0);

    Status = ObInsertObject(
        RootKey as *mut core::ffi::c_void,
        core::ptr::null_mut(),
        KEY_ALL_ACCESS,
        0,
        core::ptr::null_mut(),
        &raw mut CmpRegistryRootHandle,
    );
    if !NT_SUCCESS(Status) {
        return 0u8;
    }

    Status = ObReferenceObjectByHandle(
        CmpRegistryRootHandle,
        KEY_READ,
        core::ptr::null_mut(),
        0i8,
        &mut RootKey as *mut _ as *mut *mut core::ffi::c_void,
        core::ptr::null_mut(),
    );
    if !NT_SUCCESS(Status) {
        ObDereferenceObject(RootKey as *mut core::ffi::c_void);
        return 0u8;
    }

    return 1u8;
}

unsafe fn CmpGetRegistryPath() -> PCWSTR {
    let ConfigPath: PCWSTR;

    if ExpInTextModeSetup == 0 {
        ConfigPath = w!("\\SystemRoot\\System32\\Config\\").as_ptr();
    } else {
        ConfigPath = w!("\\SystemRoot\\").as_ptr();
    }

    crate::dbg_print!(b"CmpGetRegistryPath: ConfigPath = '%S'\n\0", ConfigPath);
    ConfigPath
}

unsafe fn CmpHasAlternateHiveDiverged(
    FileName: PCUNICODE_STRING,
    CmMainmHive: PCMHIVE,
    AlternateHandle: HANDLE,
    Diverged: PBOOLEAN)
{
    let mut Hive: PHHIVE;
    let mut AlternateHive: PHHIVE;
    let mut Status: NTSTATUS;
    let mut CmiAlternateHive: PCMHIVE = core::ptr::null_mut();

    *Diverged = 0u8;

    Status = CmpInitializeHive(
        &mut CmiAlternateHive,
        HINIT_FILE,
        0,
        HFILE_TYPE_PRIMARY,
        core::ptr::null_mut(),
        AlternateHandle,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        FileName,
        CM_CHECK_REGISTRY_DONT_PURGE_VOLATILES);
    if !NT_SUCCESS(Status) {
        crate::dbg_print!(
            b"Failed to initialize the alternate hive to check for diversion (Status 0x%lx)\n\0",
            Status
        );
        *Diverged = 1u8;
        return;
    }

    Hive = core::ptr::addr_of_mut!((*CmMainmHive).Hive);
    AlternateHive = core::ptr::addr_of_mut!((*CmiAlternateHive).Hive);
    if (*(*AlternateHive).BaseBlock).TimeStamp.QuadPart !=
        (*(*Hive).BaseBlock).TimeStamp.QuadPart
    {
        *Diverged = 1u8;
    }

    CmpDestroyHive(CmiAlternateHive);
}

#[no_mangle]
pub unsafe extern "system" fn CmpLoadHiveThread(StartContext: *mut core::ffi::c_void) {
    let StartContext = opaque_ptr(StartContext);
    let mut FileBuffer: [u16; 64] = [0; 64];
    let mut RegBuffer: [u16; 64] = [0; 64];
    let ConfigPath: PCWSTR;
    let mut TempName: _UNICODE_STRING = core::mem::zeroed();
    let mut FileName: _UNICODE_STRING = core::mem::zeroed();
    let mut RegName: _UNICODE_STRING = core::mem::zeroed();
    let i: u32;
    let mut ErrorResponse: u32 = 0;
    let WorkerCount: u32;
    let mut Length: u32;
    let FileStart: u16;
    let mut PrimaryDisposition: u32 = 0;
    let mut SecondaryDisposition: u32 = 0;
    let mut ClusterSize: u32 = 0;
    let mut CmHive: PCMHIVE = core::ptr::null_mut();
    let mut PrimaryHandle: *mut core::ffi::c_void = core::ptr::null_mut();
    let mut AlternateHandle: *mut core::ffi::c_void = core::ptr::null_mut();
    let mut Status: i32 = STATUS_SUCCESS;
    let ErrorParameters: *mut core::ffi::c_void;
    let mut HasDiverged: u8 = 0;

    i = PtrToUlong(StartContext);

    (*(*(&raw mut CmpMachineHiveList)).as_mut_ptr().add(i as usize)).ThreadStarted = 1u8;

    RtlInitEmptyUnicodeString(&mut FileName, FileBuffer.as_mut_ptr(), core::mem::size_of_val(&FileBuffer) as u16);
    RtlInitEmptyUnicodeString(&mut RegName, RegBuffer.as_mut_ptr(), core::mem::size_of_val(&RegBuffer) as u16);

    ConfigPath = CmpGetRegistryPath();
    RtlInitUnicodeString(&mut TempName, ConfigPath);
    RtlAppendUnicodeStringToString(&mut FileName, &mut TempName);
    FileStart = FileName.Length;

    RtlInitUnicodeString(&mut TempName, w!("\\REGISTRY\\").as_ptr());
    RtlAppendUnicodeStringToString(&mut RegName, &mut TempName);

    RtlInitUnicodeString(&mut TempName, (*(*(&raw const CmpMachineHiveList)).as_ptr().add(i as usize)).BaseName);
    RtlAppendUnicodeStringToString(&mut RegName, &mut TempName);

    if *RegName.Buffer.offset((RegName.Length / core::mem::size_of::<u16>() as u16 - 1) as isize) == 0x005C {
        RtlInitUnicodeString(&mut TempName, (*(*(&raw const CmpMachineHiveList)).as_ptr().add(i as usize)).Name);
        RtlAppendUnicodeStringToString(&mut RegName, &mut TempName);
    }

    RtlInitUnicodeString(&mut TempName, (*(*(&raw const CmpMachineHiveList)).as_ptr().add(i as usize)).Name);
    FileName.Length = FileStart;
    RtlAppendUnicodeStringToString(&mut FileName, &mut TempName);
    
    if (*(*(&raw const CmpMachineHiveList)).as_ptr().add(i as usize)).CmHive.is_null() {
        (*(*(&raw mut CmpMachineHiveList)).as_mut_ptr().add(i as usize)).Allocate = 1u8;

        Status = CmpInitHiveFromFile(
            &FileName,
            (*(*(&raw const CmpMachineHiveList)).as_ptr().add(i as usize)).HHiveFlags,
            &mut CmHive,
            core::ptr::addr_of_mut!((*(*(&raw mut CmpMachineHiveList)).as_mut_ptr().add(i as usize)).Allocate),
            CM_CHECK_REGISTRY_PURGE_VOLATILES
        );
        
        if !NT_SUCCESS(Status) ||
            (CmpShareSystemHives == 0 && (*CmHive).FileHandles[HFILE_TYPE_LOG as usize].is_null()) {
            ErrorParameters = core::ptr::addr_of_mut!(FileName) as *mut core::ffi::c_void;
            NtRaiseHardError(
                0xC0000218u32 as i32,
                1,
                1,
                &ErrorParameters as *const _ as *mut _,
                1,
                &mut ErrorResponse
            );
        }

        (*CmHive).Flags = (*(*(&raw const CmpMachineHiveList)).as_ptr().add(i as usize)).CmHiveFlags;
        (*(*(&raw mut CmpMachineHiveList)).as_mut_ptr().add(i as usize)).CmHive2 = CmHive;
    } else {
        CmHive = (*(*(&raw const CmpMachineHiveList)).as_ptr().add(i as usize)).CmHive;
        if ((*CmHive).Hive.HiveFlags & 0x00000001) == 0 {
            Status = CmpOpenHiveFiles(
                &FileName,
                w!(".ALT").as_ptr(),
                &mut PrimaryHandle,
                &mut AlternateHandle,
                &mut PrimaryDisposition,
                &mut SecondaryDisposition,
                1u8,
                1u8,
                0u8,
                &mut ClusterSize
            );
            
            if !NT_SUCCESS(Status) || AlternateHandle.is_null() {
                ErrorParameters = core::ptr::addr_of_mut!(FileName) as *mut core::ffi::c_void;
                NtRaiseHardError(
                    0xC0000218u32 as i32,
                    1,
                    1,
                    &ErrorParameters as *const _ as *mut _,
                    1,
                    &mut ErrorResponse
                );

                KeBugCheckEx(ntkrnl_sys::bugcodes::BAD_SYSTEM_CONFIG_INFO, 9, 0, i as usize, Status as usize);
            }

            (*CmHive).FileHandles[HFILE_TYPE_ALTERNATE as usize] = AlternateHandle;
            (*CmHive).FileHandles[HFILE_TYPE_PRIMARY as usize] = PrimaryHandle;

            // C cmsysini.c: `CmHive->Hive.HiveFlags &= ~HIVE_NOLAZYFLUSH;` (0x2).
            // Clearing 0x80 — not a HiveFlags bit — left NOLAZYFLUSH set, so
            // cmlazy (which reads the right bit) skipped this hive forever.
            (*CmHive).Hive.HiveFlags &= !HIVE_NOLAZYFLUSH;

            Length = (*CmHive).Hive.Storage[0].Length + 0x1000;

            if (*CmHive).Hive.Cluster != ClusterSize {
                // The whole body of this branch upstream is the diagnostic, so
                // dropping it left an `if` with nothing in it.
                crate::dbg_print!(
                    b"FIXME: Support for CmHive->Hive.Cluster (%lu) != ClusterSize (%lu) is unimplemented!\n\0",
                    (*CmHive).Hive.Cluster,
                    ClusterSize
                );
            }

            'Exit: {
            if (*CmHive).Hive.BaseBlock.as_ref().unwrap().BootRecover == 2 {
                crate::dbg_print!(b"FreeLdr recovered the hive (hive 0x%p)\n\0", CmHive);
                RtlSetAllBits(core::ptr::addr_of_mut!((*CmHive).Hive.DirtyVector) as *mut _);
                (*CmHive).Hive.DirtyCount = (*CmHive).Hive.DirtyVector.SizeOfBitMap;
                HvSyncHive(&mut (*CmHive).Hive as PHHIVE);
            } else {
                CmpHasAlternateHiveDiverged(
                    &FileName,
                    CmHive,
                    AlternateHandle,
                    &mut HasDiverged
                );
                
                if HasDiverged != 0 ||
                    PrimaryDisposition == 0x00000002 ||
                    SecondaryDisposition == 0x00000002 {
                    if HvWriteAlternateHive(&mut (*CmHive).Hive as PHHIVE) == 0 {
                        crate::dbg_print!(b"Failed to write to alternate hive\n\0");
                        break 'Exit; // C: goto Exit
                    }
                }
            }

            (*(*(&raw mut CmpMachineHiveList)).as_mut_ptr().add(i as usize)).CmHive2 = CmHive;
            } // 'Exit
        }
    }

    (*(*(&raw mut CmpMachineHiveList)).as_mut_ptr().add(i as usize)).ThreadFinished = 1u8;

    WorkerCount = InterlockedIncrement(&raw mut CmpLoadWorkerIncrement as *mut i32) as u32;
    if WorkerCount == CM_NUMBER_OF_MACHINE_HIVES as u32 {
        KeSetEvent(&raw mut CmpLoadWorkerEvent as *mut _ as *mut core::ffi::c_void, 0, 0u8);
    }

    PsTerminateSystemThread(Status);
}

unsafe extern "system" fn CmpLoadHiveThreadStart(StartContext: PVOID) {
    CmpLoadHiveThread(StartContext);
}

#[no_mangle]
pub unsafe extern "system" fn CmpInitializeHiveList() {
    let mut FileBuffer: [u16; 64] = [0; 64];
    let mut RegBuffer: [u16; 64] = [0; 64];
    let ConfigPath: PCWSTR;
    let mut TempName: _UNICODE_STRING = core::mem::zeroed();
    let mut FileName: _UNICODE_STRING = core::mem::zeroed();
    let mut RegName: _UNICODE_STRING = core::mem::zeroed();
    let mut Thread: HANDLE = core::ptr::null_mut();
    let Status: NTSTATUS;
    let mut i: ULONG;
    let RegStart: USHORT;
    let SecurityDescriptor: PSECURITY_DESCRIPTOR;

    CmpNoWrite = 0u8;

    RtlInitEmptyUnicodeString(
        &mut FileName,
        FileBuffer.as_mut_ptr(),
        core::mem::size_of_val(&FileBuffer) as u16,
    );
    RtlInitEmptyUnicodeString(
        &mut RegName,
        RegBuffer.as_mut_ptr(),
        core::mem::size_of_val(&RegBuffer) as u16,
    );

    ConfigPath = CmpGetRegistryPath();
    RtlInitUnicodeString(&mut TempName, ConfigPath);
    RtlAppendUnicodeStringToString(&mut FileName, &mut TempName);

    RtlInitUnicodeString(&mut TempName, w!("\\REGISTRY\\").as_ptr());
    RtlAppendUnicodeStringToString(&mut RegName, &mut TempName);
    RegStart = RegName.Length;

    KeInitializeEvent(&raw mut CmpLoadWorkerEvent, _EVENT_TYPE_SynchronizationEvent, 0u8);

    CmpSpecialBootCondition = 1u8;

    SecurityDescriptor = CmpHiveRootSecurityDescriptor();

    i = 0;
    while i < CM_NUMBER_OF_MACHINE_HIVES as u32 {
        // C: if (CmpShareSystemHives) CmpMachineHiveList[i].HHiveFlags |= HIVE_VOLATILE;
        // (was mistranslated as `if CmHive2 != null` — CmHive2 is only set later BY the
        // load threads, so MININT hives were never marked volatile and CmpLoadHiveThread
        // spent ~5.5s failing to write CD-backed hives; perf doc "hive-flush stall")
        if CmpShareSystemHives != 0 {
            (*(*(&raw mut CmpMachineHiveList)).as_mut_ptr().add(i as usize)).HHiveFlags |= HIVE_VOLATILE;
        }

        let status = PsCreateSystemThread(
            &mut Thread,
            0x1FFFFF,
            core::ptr::null_mut(),
            0 as HANDLE,
            core::ptr::null_mut(),
            Some(CmpLoadHiveThreadStart),
            i as PVOID,
        );
        if NT_SUCCESS(status) {
            ZwClose(Thread);
        } else {
            KeBugCheckEx(ntkrnl_sys::bugcodes::BAD_SYSTEM_CONFIG_INFO, 9, 3, i as ULONG_PTR, status as ULONG_PTR);
        }

        i += 1;
    }

    KeWaitForSingleObject(
        &raw mut CmpLoadWorkerEvent as *mut _ as PVOID,
        _KWAIT_REASON_Executive,
        _MODE_KernelMode as i8,
        0u8,
        core::ptr::null_mut(),
    );

    CmpSpecialBootCondition = 0u8;

    i = 0;
    while i < CM_NUMBER_OF_MACHINE_HIVES as u32 {
        if (*(*(&raw const CmpMachineHiveList)).as_ptr().add(i as usize)).CmHive == core::ptr::null_mut() {
            RegName.Length = RegStart;
            RtlInitUnicodeString(
                &mut TempName,
                (*(*(&raw const CmpMachineHiveList)).as_ptr().add(i as usize)).BaseName,
            );
            RtlAppendUnicodeStringToString(&mut RegName, &mut TempName);

            if *RegName.Buffer.add((RegName.Length / core::mem::size_of::<u16>() as u16 - 1) as usize) == 0x005C {
                RtlInitUnicodeString(
                    &mut TempName,
                    (*(*(&raw const CmpMachineHiveList)).as_ptr().add(i as usize)).Name,
                );
                RtlAppendUnicodeStringToString(&mut RegName, &mut TempName);
            }

            let status = CmpLinkHiveToMaster(
                &mut RegName,
                core::ptr::null_mut(),
                (*(*(&raw const CmpMachineHiveList)).as_ptr().add(i as usize)).CmHive2,
                (*(*(&raw const CmpMachineHiveList)).as_ptr().add(i as usize)).Allocate,
                SecurityDescriptor,
            );
            if status != STATUS_SUCCESS {
                KeBugCheckEx(ntkrnl_sys::bugcodes::CONFIG_LIST_FAILED, 11, status as ULONG_PTR, i as ULONG_PTR, &mut RegName as *mut _ as ULONG_PTR);
            }

            if (*(*(&raw const CmpMachineHiveList)).as_ptr().add(i as usize)).Allocate != 0 {
            }
        }

        if (*(*(&raw const CmpMachineHiveList)).as_ptr().add(i as usize)).CmHive2 != core::ptr::null_mut() {
            CmpAddToHiveFileList((*(*(&raw const CmpMachineHiveList)).as_ptr().add(i as usize)).CmHive2);
        }

        i += 1;
    }

    ExFreePoolWithTag(SecurityDescriptor as PVOID, 0x53646D43);

    // `\REGISTRY`, upper case, and it is not cosmetic. A symbolic-link key stores
    // its target verbatim and CmpGetSymbolicLink hands that string straight back to
    // ObpLookupObjectName as a REPARSE, which resolves the leading `REGISTRY`
    // component in the ROOT OBJECT DIRECTORY — where the registry root was inserted
    // as `\REGISTRY` (CmpCreateRegistryRoot). That compare is case-SENSITIVE unless
    // the caller passed OBJ_CASE_INSENSITIVE, because the Key object type does not
    // declare itself case-insensitive (ObTypes.c:217 asserts exactly that against
    // WS03, so making the type case-insensitive is not the fix — it fails a test).
    // Upstream spells these `\Registry\...`, so opening one of these links from a
    // caller that omits the attribute returned STATUS_OBJECT_PATH_NOT_FOUND: it hit
    // the symlink, reparsed to `\Registry\User\.Default`, and then failed to find
    // `Registry` next to `REGISTRY`. Everything past the first component is CM's own
    // lookup, which is case-insensitive regardless.
    CmpLinkKeyToHive(
        w!("\\REGISTRY\\MACHINE\\SECURITY\\SAM").as_ptr(),
        w!("\\REGISTRY\\MACHINE\\SAM\\SAM").as_ptr(),
    );

    CmpNoVolatileCreates = 0u8;
    CmpLinkKeyToHive(
        w!("\\REGISTRY\\USER\\S-1-5-18").as_ptr(),
        w!("\\REGISTRY\\USER\\.DEFAULT").as_ptr(),
    );
    CmpNoVolatileCreates = 1u8;
}

#[no_mangle]
pub unsafe extern "system" fn CmInitSystem1() -> BOOLEAN {
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut KeyName: _UNICODE_STRING = core::mem::zeroed();
    let mut KeyHandle: HANDLE = core::ptr::null_mut();
    let mut Status: NTSTATUS;
    let mut HardwareHive: PCMHIVE = core::ptr::null_mut();
    let SecurityDescriptor: PSECURITY_DESCRIPTOR;

    if InitIsWinPEMode != 0 {
        CmpMiniNTBoot = 1u8;
        CmpShareSystemHives = 1u8;
    }

    if CmpVolatileBoot != 0 {
        CmpShareSystemHives = 1u8;
    }

    InitializeListHead(&raw mut CmpHiveListHead as *mut _ as *mut LIST_ENTRY);
    ExInitializePushLock(&raw mut CmpHiveListHeadLock as *mut _ as *mut _);
    ExInitializePushLock(&raw mut CmpLoadHiveLock as *mut _ as *mut _);

    ExInitializeResourceLite(&raw mut CmpRegistryLock);

    CmpInitializeCache();

    CmpInitCmPrivateAlloc();
    CmpInitCmPrivateDelayAlloc();
    CmpInitDelayDerefKCBEngine();

    CmpInitCallback();

    KeInitializeGuardedMutex(&raw mut CmpSelfHealQueueLock as *mut _ as *mut _);
    InitializeListHead(&raw mut CmpSelfHealQueueListHead as *mut _ as *mut LIST_ENTRY);

    CmpSystemProcess = PsGetCurrentProcess();

    ARM64_UART_TRACE!(b"[CM] CmpCreateObjectTypes\r\n\0".as_ptr());
    Status = CmpCreateObjectTypes();
    if !NT_SUCCESS(Status) {
        KeBugCheckEx(CONFIG_INITIALIZATION_FAILED, 1, 1, Status as ULONG_PTR, 0);
    }

    ARM64_UART_TRACE!(b"[CM] CmpInitializeHive (master)\r\n\0".as_ptr());
    Status = CmpInitializeHive(
        &raw mut CmiVolatileHive,
        HINIT_CREATE,
        HIVE_VOLATILE,
        HFILE_TYPE_PRIMARY,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        CM_CHECK_REGISTRY_DONT_PURGE_VOLATILES,
    );
    if !NT_SUCCESS(Status) {
        KeBugCheckEx(CONFIG_INITIALIZATION_FAILED, 1, 2, Status as ULONG_PTR, 0);
    }

    ARM64_UART_TRACE!(b"[CM] CmpCreateRegistryRoot\r\n\0".as_ptr());
    if CmpCreateRegistryRoot() == 0 {
        KeBugCheckEx(CONFIG_INITIALIZATION_FAILED, 1, 3, 0, 0);
    }

    SecurityDescriptor = CmpHiveRootSecurityDescriptor();

    ARM64_UART_TRACE!(b"[CM] NtCreateKey MACHINE\r\n\0".as_ptr());
    RtlInitUnicodeString(&mut KeyName, w!("\\REGISTRY\\MACHINE").as_ptr());
    ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
    ObjectAttributes.RootDirectory = core::ptr::null_mut();
    ObjectAttributes.ObjectName = &mut KeyName;
    ObjectAttributes.Attributes = OBJ_CASE_INSENSITIVE;
    ObjectAttributes.SecurityDescriptor = SecurityDescriptor as *mut core::ffi::c_void;
    ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();
    Status = NtCreateKey(
        &mut KeyHandle,
        KEY_READ | KEY_WRITE,
        &mut ObjectAttributes,
        0,
        core::ptr::null_mut(),
        0,
        core::ptr::null_mut(),
    );
    if !NT_SUCCESS(Status) {
        KeBugCheckEx(CONFIG_INITIALIZATION_FAILED, 1, 5, Status as ULONG_PTR, 0);
    }

    NtClose(KeyHandle);

    ARM64_UART_TRACE!(b"[CM] NtCreateKey USER\r\n\0".as_ptr());
    RtlInitUnicodeString(&mut KeyName, w!("\\REGISTRY\\USER").as_ptr());
    ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
    ObjectAttributes.RootDirectory = core::ptr::null_mut();
    ObjectAttributes.ObjectName = &mut KeyName;
    ObjectAttributes.Attributes = OBJ_CASE_INSENSITIVE;
    ObjectAttributes.SecurityDescriptor = SecurityDescriptor as *mut core::ffi::c_void;
    ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();
    Status = NtCreateKey(
        &mut KeyHandle,
        KEY_READ | KEY_WRITE,
        &mut ObjectAttributes,
        0,
        core::ptr::null_mut(),
        0,
        core::ptr::null_mut(),
    );
    if !NT_SUCCESS(Status) {
        KeBugCheckEx(CONFIG_INITIALIZATION_FAILED, 1, 6, Status as ULONG_PTR, 0);
    }

    NtClose(KeyHandle);

    CmpNoVolatileCreates = 1u8;

    ARM64_UART_TRACE!(b"[CM] CmpInitializeSystemHive\r\n\0".as_ptr());
    if CmpInitializeSystemHive(KeLoaderBlock) == 0 {
        KeBugCheckEx(CONFIG_INITIALIZATION_FAILED, 1, 7, 0, 0);
    }

    ARM64_UART_TRACE!(b"[CM] CmpCreateControlSet\r\n\0".as_ptr());
    Status = CmpCreateControlSet(KeLoaderBlock);
    if !NT_SUCCESS(Status) {
        KeBugCheckEx(CONFIG_INITIALIZATION_FAILED, 1, 8, Status as ULONG_PTR, 0);
    }

    ARM64_UART_TRACE!(b"[CM] CmpInitializeHive (hardware)\r\n\0".as_ptr());
    Status = CmpInitializeHive(
        &mut HardwareHive,
        HINIT_CREATE,
        HIVE_VOLATILE,
        HFILE_TYPE_PRIMARY,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        CM_CHECK_REGISTRY_DONT_PURGE_VOLATILES,
    );
    if !NT_SUCCESS(Status) {
        KeBugCheckEx(CONFIG_INITIALIZATION_FAILED, 1, 11, Status as ULONG_PTR, 0);
    }

    (*(*(&raw mut CmpMachineHiveList)).as_mut_ptr().add(0)).CmHive = HardwareHive;

    RtlInitUnicodeString(&mut KeyName, w!("\\Registry\\Machine\\HARDWARE").as_ptr());
    Status = CmpLinkHiveToMaster(
        &mut KeyName,
        core::ptr::null_mut(),
        HardwareHive,
        1u8,
        SecurityDescriptor,
    );
    if !NT_SUCCESS(Status) {
        KeBugCheckEx(CONFIG_INITIALIZATION_FAILED, 1, 12, Status as ULONG_PTR, 0);
    }

    CmpAddToHiveFileList(HardwareHive);

    ExFreePoolWithTag(SecurityDescriptor as PVOID, TAG_CMSD);

    ARM64_UART_TRACE!(b"[CM] CmpInitializeHardwareConfiguration\r\n\0".as_ptr());
    Status = CmpInitializeHardwareConfiguration(KeLoaderBlock);
    ARM64_UART_TRACE!(b"[CM] CmpInitializeHardwareConfiguration returned\r\n\0".as_ptr());
    if !NT_SUCCESS(Status) {
        KeBugCheckEx(CONFIG_INITIALIZATION_FAILED, 1, 13, Status as ULONG_PTR, 0);
    }

    ARM64_UART_TRACE!(b"[CM] CmpInitializeMachineDependentConfiguration\r\n\0".as_ptr());
    Status = CmpInitializeMachineDependentConfiguration(KeLoaderBlock);
    if !NT_SUCCESS(Status) {
        KeBugCheckEx(CONFIG_INITIALIZATION_FAILED, 1, 14, Status as ULONG_PTR, 0);
    }

    Status = CmpSetSystemValues(KeLoaderBlock);
    if !NT_SUCCESS(Status) {
        KeBugCheckEx(CONFIG_INITIALIZATION_FAILED, 1, 15, Status as ULONG_PTR, 0);
    }

    ExFreePoolWithTag(CmpLoadOptions.Buffer as PVOID, TAG_CM);

    ARM64_UART_TRACE!(b"[CM] CmInitSystem1 done\r\n\0".as_ptr());
    1u8
}

#[no_mangle]
pub unsafe extern "system" fn CmpLockRegistryExclusive() {
    KeEnterCriticalRegion();
    ExAcquireResourceExclusiveLite(core::ptr::addr_of_mut!(CmpRegistryLock), 1u8);
    
    RtlGetCallersAddress(
        core::ptr::addr_of_mut!(CmpRegistryLockCaller),
        core::ptr::addr_of_mut!(CmpRegistryLockCallerCaller)
    );
}

#[no_mangle]
pub unsafe extern "system" fn CmpLockRegistry() {
    KeEnterCriticalRegion();

    if CmpFlushStarveWriters != 0 {
        ExAcquireSharedStarveExclusive(core::ptr::addr_of_mut!(CmpRegistryLock), 1u8);
    } else {
        ExAcquireResourceSharedLite(core::ptr::addr_of_mut!(CmpRegistryLock), 1u8);
    }
    static mut LOCK_COUNT: u32 = 0;
    LOCK_COUNT += 1;
    if LOCK_COUNT < 100 {
        let mut caller: PVOID = core::ptr::null_mut();
        let mut caller2: PVOID = core::ptr::null_mut();
        RtlGetCallersAddress(&mut caller, &mut caller2);
        crate::dbg_print!(b"[LOCK] Lock #%d caller=%p\r\n\0", LOCK_COUNT, caller);
    }
}

#[no_mangle]
pub unsafe extern "system" fn CmpTestRegistryLock() -> u8 {
    if ExIsResourceAcquiredSharedLite(core::ptr::addr_of_mut!(CmpRegistryLock)) != 0 {
        1u8
    } else {
        0u8
    }
}

#[no_mangle]
pub unsafe extern "system" fn CmpTestRegistryLockExclusive() -> u8 {
    if ExIsResourceAcquiredExclusiveLite(core::ptr::addr_of_mut!(CmpRegistryLock)) == 0 {
        0u8
    } else {
        1u8
    }
}

#[no_mangle]
pub unsafe extern "system" fn CmpLockHiveFlusherExclusive(Hive: PCMHIVE) {
    let Hive = opaque_ptr(Hive);
    
    CMP_ASSERT_REGISTRY_LOCK_OR_LOADING(Hive);
    ExAcquireResourceExclusiveLite((*Hive).FlusherLock, 1u8);
}

#[no_mangle]
pub unsafe extern "system" fn CmpLockHiveFlusherShared(Hive: PCMHIVE) {
    let Hive = opaque_ptr(Hive);
    CMP_ASSERT_REGISTRY_LOCK_OR_LOADING(Hive);
    ExAcquireResourceSharedLite((*Hive).FlusherLock, 1u8);
}

#[no_mangle]
pub unsafe extern "system" fn CmpUnlockHiveFlusher(Hive: PCMHIVE) {
    let Hive = opaque_ptr(Hive);

    CMP_ASSERT_REGISTRY_LOCK_OR_LOADING(Hive);
    CMP_ASSERT_FLUSH_LOCK(Hive);

    ExReleaseResourceLite((*Hive).FlusherLock);
}

#[no_mangle]
pub unsafe extern "system" fn CmpTestHiveFlusherLockShared(Hive: PCMHIVE) -> u8 {
    let Hive = opaque_ptr(Hive);
    if ExIsResourceAcquiredSharedLite((*Hive).FlusherLock) == 0 {
        0u8
    } else {
        1u8
    }
}

#[no_mangle]
pub unsafe extern "system" fn CmpTestHiveFlusherLockExclusive(Hive: PCMHIVE) -> u8 {
    let Hive = opaque_ptr(Hive);
    if ExIsResourceAcquiredExclusiveLite((*Hive).FlusherLock) == 0 {
        0u8
    } else {
        1u8
    }
}

#[no_mangle]
pub unsafe extern "system" fn CmpUnlockRegistry() {
    CMP_ASSERT_REGISTRY_LOCK();

    if CmpFlushOnLockRelease != 0 {
        CMP_ASSERT_EXCLUSIVE_REGISTRY_LOCK();

        CmpDoFlushAll(1u8);
        CmpFlushOnLockRelease = 0u8;
    }
    static mut UNLOCK_COUNT: u32 = 0;
    UNLOCK_COUNT += 1;
    if UNLOCK_COUNT < 100 {
        let mut caller: PVOID = core::ptr::null_mut();
        let mut caller2: PVOID = core::ptr::null_mut();
        RtlGetCallersAddress(&mut caller, &mut caller2);
        crate::dbg_print!(b"[UNLK] Unlock #%d caller=%p\r\n\0", UNLOCK_COUNT, caller);
    }

    ExReleaseResourceLite(core::ptr::addr_of_mut!(CmpRegistryLock));
    KeLeaveCriticalRegion();
}

#[no_mangle]
pub unsafe extern "system" fn CmpAcquireTwoKcbLocksExclusiveByKey(
    ConvKey1: ULONG,
    ConvKey2: ULONG,
) {
    let Index1: ULONG;
    let Index2: ULONG;

    CMP_ASSERT_REGISTRY_LOCK();

    Index1 = GET_HASH_INDEX(ConvKey1);
    Index2 = GET_HASH_INDEX(ConvKey2);

    if Index1 < Index2 {
        CmpAcquireKcbLockExclusiveByKey(ConvKey1);
        CmpAcquireKcbLockExclusiveByKey(ConvKey2);
    } else {
        CmpAcquireKcbLockExclusiveByKey(ConvKey2);
        if Index1 != Index2 {
            CmpAcquireKcbLockExclusiveByKey(ConvKey1);
        }
    }
}

#[no_mangle]
pub unsafe extern "system" fn CmpReleaseTwoKcbLockByKey(
    ConvKey1: u32,
    ConvKey2: u32,
) {
    let Index1: u32;
    let Index2: u32;

    CMP_ASSERT_REGISTRY_LOCK();

    Index1 = GET_HASH_INDEX(ConvKey1);
    Index2 = GET_HASH_INDEX(ConvKey2);

    if Index1 < Index2 {
        CmpReleaseKcbLockByKey(ConvKey2);
        CmpReleaseKcbLockByKey(ConvKey1);
    } else {
        if Index1 != Index2 {
            CmpReleaseKcbLockByKey(ConvKey1);
        }
        CmpReleaseKcbLockByKey(ConvKey2);
    }
}

#[no_mangle]
pub unsafe extern "system" fn CmShutdownSystem() {
    let mut ListEntry: PLIST_ENTRY;
    let mut Hive: PCMHIVE;

    if CmFirstTime == 0 {
        CmpShutdownWorkers();
    }

    CmpLockRegistryExclusive();
    CmpDoFlushAll(1u8);

    ListEntry = (*core::ptr::addr_of_mut!(CmpHiveListHead)).Flink;
    while ListEntry != core::ptr::addr_of_mut!(CmpHiveListHead) {
        Hive = (ListEntry as *mut u8).offset(-(core::mem::offset_of!(CMHIVE, HiveList) as isize)) as PCMHIVE;

        CmpCloseHiveFiles(Hive);

        ListEntry = (*ListEntry).Flink;
    }

    HvShutdownComplete = 1u8;

    CmpUnlockRegistry();
}

#[no_mangle]
pub unsafe extern "system" fn CmpSetVersionData() {
    let mut Status: NTSTATUS;
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut KeyName: _UNICODE_STRING = core::mem::zeroed();
    let mut ValueName: _UNICODE_STRING = core::mem::zeroed();
    let mut ValueData: _UNICODE_STRING = core::mem::zeroed();
    let mut TempString: ANSI_STRING = core::mem::zeroed();
    let mut SoftwareKeyHandle: HANDLE = core::ptr::null_mut();
    let mut MicrosoftKeyHandle: HANDLE = core::ptr::null_mut();
    let mut WindowsNtKeyHandle: HANDLE = core::ptr::null_mut();
    let mut CurrentVersionKeyHandle: HANDLE = core::ptr::null_mut();
    let mut Buffer: [u16; 128] = [0; 128];

    RtlInitUnicodeString(&mut KeyName, w!("\\REGISTRY\\MACHINE\\SOFTWARE").as_ptr());
    ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as u32;
    ObjectAttributes.RootDirectory = core::ptr::null_mut();
    ObjectAttributes.ObjectName = &mut KeyName;
    ObjectAttributes.Attributes = 0x00000040 | 0x00000200;
    ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
    ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();
    
    Status = NtCreateKey(
        &mut SoftwareKeyHandle,
        0x0004,
        &mut ObjectAttributes,
        0,
        core::ptr::null_mut(),
        0,
        core::ptr::null_mut()
    );
    if !NT_SUCCESS(Status) {
        crate::dbg_print!(
            b"Failed to create key %wZ (Status: %08lx)\n\0",
            core::ptr::addr_of!(KeyName),
            Status
        );
        return;
    }

    RtlInitUnicodeString(&mut KeyName, w!("Microsoft").as_ptr());
    ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as u32;
    ObjectAttributes.RootDirectory = SoftwareKeyHandle;
    ObjectAttributes.ObjectName = &mut KeyName;
    ObjectAttributes.Attributes = 0x00000040 | 0x00000200;
    ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
    ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();
    
    Status = NtCreateKey(
        &mut MicrosoftKeyHandle,
        0x0004,
        &mut ObjectAttributes,
        0,
        core::ptr::null_mut(),
        0,
        core::ptr::null_mut()
    );
    if !NT_SUCCESS(Status) {
        crate::dbg_print!(
            b"Failed to create key %wZ (Status: %08lx)\n\0",
            core::ptr::addr_of!(KeyName),
            Status
        );
        NtClose(SoftwareKeyHandle);
        return;
    }

    RtlInitUnicodeString(&mut KeyName, w!("Windows NT").as_ptr());
    ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as u32;
    ObjectAttributes.RootDirectory = MicrosoftKeyHandle;
    ObjectAttributes.ObjectName = &mut KeyName;
    ObjectAttributes.Attributes = 0x00000040 | 0x00000200;
    ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
    ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();
    
    Status = NtCreateKey(
        &mut WindowsNtKeyHandle,
        0x0004,
        &mut ObjectAttributes,
        0,
        core::ptr::null_mut(),
        0,
        core::ptr::null_mut()
    );
    if !NT_SUCCESS(Status) {
        crate::dbg_print!(
            b"Failed to create key %wZ (Status: %08lx)\n\0",
            core::ptr::addr_of!(KeyName),
            Status
        );
        NtClose(MicrosoftKeyHandle);
        NtClose(SoftwareKeyHandle);
        return;
    }

    RtlInitUnicodeString(&mut KeyName, w!("CurrentVersion").as_ptr());
    ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as u32;
    ObjectAttributes.RootDirectory = WindowsNtKeyHandle;
    ObjectAttributes.ObjectName = &mut KeyName;
    ObjectAttributes.Attributes = 0x00000040 | 0x00000200;
    ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
    ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();
    
    Status = NtCreateKey(
        &mut CurrentVersionKeyHandle,
        0x0004 | 0x0002,
        &mut ObjectAttributes,
        0,
        core::ptr::null_mut(),
        0,
        core::ptr::null_mut()
    );
    if !NT_SUCCESS(Status) {
        crate::dbg_print!(
            b"Failed to create key %wZ (Status: %08lx)\n\0",
            core::ptr::addr_of!(KeyName),
            Status
        );
        NtClose(WindowsNtKeyHandle);
        NtClose(MicrosoftKeyHandle);
        NtClose(SoftwareKeyHandle);
        return;
    }

    RtlInitUnicodeString(&mut ValueName, w!("CurrentVersion").as_ptr());
    NtSetValueKey(
        CurrentVersionKeyHandle,
        &mut ValueName,
        0,
        1,
        CmVersionString.Buffer as *mut core::ffi::c_void,
        CmVersionString.Length as u32 + core::mem::size_of::<u16>() as u32
    );

    RtlInitUnicodeString(&mut ValueName, w!("CurrentBuildNumber").as_ptr());
    RtlInitEmptyUnicodeString(&mut ValueData, Buffer.as_mut_ptr(), core::mem::size_of_val(&Buffer) as u16);
    RtlIntegerToUnicodeString(NtBuildNumber & 0xFFFF, 10, &mut ValueData);
    NtSetValueKey(
        CurrentVersionKeyHandle,
        &mut ValueName,
        0,
        1,
        ValueData.Buffer as *mut core::ffi::c_void,
        ValueData.Length as u32 + core::mem::size_of::<u16>() as u32
    );

    RtlInitUnicodeString(&mut ValueName, w!("BuildLab").as_ptr());
    RtlInitAnsiString(&mut TempString, core::ptr::addr_of!(NtBuildLab) as *const i8);
    Status = RtlAnsiStringToUnicodeString(&mut ValueData, &mut TempString, 0u8);
    if NT_SUCCESS(Status) {
        NtSetValueKey(
            CurrentVersionKeyHandle,
            &mut ValueName,
            0,
            1,
            ValueData.Buffer as *mut core::ffi::c_void,
            ValueData.Length as u32 + core::mem::size_of::<u16>() as u32
        );
    }

    RtlInitUnicodeString(&mut ValueName, w!("CurrentType").as_ptr());
    let cpu_kind: *const u16 = {
        #[cfg(feature = "smp")] { w!("Multiprocessor").as_ptr() }
        #[cfg(not(feature = "smp"))] { w!("Uniprocessor").as_ptr() }
    };
    let build_kind: *const u16 = {
        #[cfg(DBG)] { w!("Checked").as_ptr() }
        #[cfg(not(DBG))] { w!("Free").as_ptr() }
    };
    crate::rtl_string_cb_printf_w!(
        Buffer.as_mut_ptr(),
        core::mem::size_of_val(&Buffer),
        crate::wstr!("%s %s"),
        cpu_kind,
        build_kind,
    );
    RtlInitUnicodeString(&mut ValueData, Buffer.as_ptr());
    NtSetValueKey(
        CurrentVersionKeyHandle,
        &mut ValueName,
        0,
        1,
        ValueData.Buffer as *mut core::ffi::c_void,
        ValueData.Length as u32 + core::mem::size_of::<u16>() as u32
    );

    RtlInitUnicodeString(&mut ValueName, w!("CSDVersion").as_ptr());
    if CmCSDVersionString.Length != 0 {
        NtSetValueKey(
            CurrentVersionKeyHandle,
            &mut ValueName,
            0,
            1,
            CmCSDVersionString.Buffer as *mut core::ffi::c_void,
            CmCSDVersionString.Length as u32 + core::mem::size_of::<u16>() as u32
        );
    } else {
        NtDeleteValueKey(CurrentVersionKeyHandle, &mut ValueName);
    }

    RtlInitUnicodeString(&mut ValueName, w!("CSDBuildNumber").as_ptr());
    if CmNtSpBuildNumber != 0 {
        RtlInitEmptyUnicodeString(&mut ValueData, Buffer.as_mut_ptr(), core::mem::size_of_val(&Buffer) as u16);
        RtlIntegerToUnicodeString(CmNtSpBuildNumber, 10, &mut ValueData);
        NtSetValueKey(
            CurrentVersionKeyHandle,
            &mut ValueName,
            0,
            1,
            ValueData.Buffer as *mut core::ffi::c_void,
            ValueData.Length as u32 + core::mem::size_of::<u16>() as u32
        );
    } else {
        NtDeleteValueKey(CurrentVersionKeyHandle, &mut ValueName);
    }

    RtlInitUnicodeString(&mut ValueName, w!("SystemRoot").as_ptr());
    NtSetValueKey(
        CurrentVersionKeyHandle,
        &mut ValueName,
        0,
        1,
        NtSystemRoot.Buffer as *mut core::ffi::c_void,
        NtSystemRoot.Length as u32 + core::mem::size_of::<u16>() as u32
    );

    if !CurrentVersionKeyHandle.is_null() {
        NtClose(CurrentVersionKeyHandle);
    }
    if !WindowsNtKeyHandle.is_null() {
        NtClose(WindowsNtKeyHandle);
    }
    if !MicrosoftKeyHandle.is_null() {
        NtClose(MicrosoftKeyHandle);
    }
    if !SoftwareKeyHandle.is_null() {
        NtClose(SoftwareKeyHandle);
    }
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         ReactOS Kernel
//  * LICENSE:         BSD - See COPYING.ARM in the top level directory
//  * PURPOSE:         Configuration Manager - System Initialization Code
//  * PROGRAMMERS:     ReactOS Portable Systems Group
//  *                  Alex Ionescu (alex.ionescu@reactos.org)
//  */
//
// /* INCLUDES *******************************************************************/
//
// #include "ntoskrnl.h"
// #define NDEBUG
// #include "debug.h"
//
//
// POBJECT_TYPE CmpKeyObjectType;
// PCMHIVE CmiVolatileHive;
// LIST_ENTRY CmpHiveListHead;
// ERESOURCE CmpRegistryLock;
// KGUARDED_MUTEX CmpSelfHealQueueLock;
// LIST_ENTRY CmpSelfHealQueueListHead;
// KEVENT CmpLoadWorkerEvent;
// LONG CmpLoadWorkerIncrement;
// PEPROCESS CmpSystemProcess;
// PVOID CmpRegistryLockCallerCaller, CmpRegistryLockCaller;
// BOOLEAN CmpFlushOnLockRelease;
// BOOLEAN CmpSpecialBootCondition;
//
// /* Disable registry hive writes, until the IO subsystem is initialized
//  * and disk access is enabled (when the SM signals so after AUTOCHK) */
// BOOLEAN CmpNoWrite = TRUE;
//
// BOOLEAN CmpWasSetupBoot;
// BOOLEAN CmpProfileLoaded;
// BOOLEAN CmpNoVolatileCreates;
// ULONG CmpTraceLevel = 0;
// BOOLEAN HvShutdownComplete = FALSE;
//
// extern LONG CmpFlushStarveWriters;
// extern BOOLEAN CmFirstTime;
//
// /* FUNCTIONS ******************************************************************/
//
// BOOLEAN
// NTAPI
// CmpLinkKeyToHive(
//     _In_z_ PCWSTR LinkKeyName,
//     _In_z_ PCWSTR TargetKeyName)
// {
//     NTSTATUS Status;
//     OBJECT_ATTRIBUTES ObjectAttributes;
//     UNICODE_STRING KeyName;
//     HANDLE LinkKeyHandle;
//     ULONG Disposition;
//
//     PAGED_CODE();
//
//     /* Initialize the object attributes */
//     RtlInitUnicodeString(&KeyName, LinkKeyName);
//     InitializeObjectAttributes(&ObjectAttributes,
//                                &KeyName,
//                                OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE,
//                                NULL,
//                                NULL);
//
//     /* Create the link key */
//     Status = ZwCreateKey(&LinkKeyHandle,
//                          KEY_CREATE_LINK,
//                          &ObjectAttributes,
//                          0,
//                          NULL,
//                          REG_OPTION_VOLATILE | REG_OPTION_CREATE_LINK,
//                          &Disposition);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("CM: CmpLinkKeyToHive: couldn't create %S, Status = 0x%lx\n",
//                 LinkKeyName, Status);
//         return FALSE;
//     }
//
//     /* Check if the new key was actually created */
//     if (Disposition != REG_CREATED_NEW_KEY)
//     {
//         DPRINT1("CM: CmpLinkKeyToHive: %S already exists!\n", LinkKeyName);
//         ZwClose(LinkKeyHandle);
//         return FALSE;
//     }
//
//     /* Set the target key name as link target */
//     RtlInitUnicodeString(&KeyName, TargetKeyName);
//     Status = ZwSetValueKey(LinkKeyHandle,
//                            &mut CmSymbolicLinkValueName,
//                            0,
//                            REG_LINK,
//                            KeyName.Buffer,
//                            KeyName.Length);
//
//     /* Close the link key handle */
//     ObCloseHandle(LinkKeyHandle, KernelMode);
//
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("CM: CmpLinkKeyToHive: couldn't create symbolic link for %S, Status = 0x%lx\n",
//                 TargetKeyName, Status);
//         return FALSE;
//     }
//
//     return TRUE;
// }
//
// VOID
// NTAPI
// CmpDeleteKeyObject(PVOID DeletedObject)
// {
//     PCM_KEY_BODY KeyBody = (PCM_KEY_BODY)DeletedObject;
//     PCM_KEY_CONTROL_BLOCK Kcb;
//     REG_KEY_HANDLE_CLOSE_INFORMATION KeyHandleCloseInfo;
//     REG_POST_OPERATION_INFORMATION PostOperationInfo;
//     NTSTATUS Status;
//     PAGED_CODE();
//
//     /* First off, prepare the handle close information callback */
//     PostOperationInfo.Object = KeyBody;
//     KeyHandleCloseInfo.Object = KeyBody;
//     Status = CmiCallRegisteredCallbacks(RegNtPreKeyHandleClose,
//                                         &KeyHandleCloseInfo);
//     if (!NT_SUCCESS(Status))
//     {
//         /* If we failed, notify the post routine */
//         PostOperationInfo.Status = Status;
//         CmiCallRegisteredCallbacks(RegNtPostKeyHandleClose, &PostOperationInfo);
//         return;
//     }
//
//     /* Acquire hive lock */
//     CmpLockRegistry();
//
//     /* Make sure this is a valid key body */
//     if (KeyBody->Type == CM_KEY_BODY_TYPE)
//     {
//         /* Get the KCB */
//         Kcb = KeyBody->KeyControlBlock;
//         if (Kcb)
//         {
//             /* Delist the key */
//             DelistKeyBodyFromKCB(KeyBody, KeyBody->KcbLocked);
//
//             /* Dereference the KCB */
//             CmpDelayDerefKeyControlBlock(Kcb);
//         }
//     }
//
//     /* Release the registry lock */
//     CmpUnlockRegistry();
//
//     /* Do the post callback */
//     PostOperationInfo.Status = STATUS_SUCCESS;
//     CmiCallRegisteredCallbacks(RegNtPostKeyHandleClose, &PostOperationInfo);
// }
//
// VOID
// NTAPI
// CmpCloseKeyObject(IN PEPROCESS Process OPTIONAL,
//                   IN PVOID Object,
//                   IN ACCESS_MASK GrantedAccess,
//                   IN ULONG ProcessHandleCount,
//                   IN ULONG SystemHandleCount)
// {
//     PCM_KEY_BODY KeyBody = (PCM_KEY_BODY)Object;
//     PAGED_CODE();
//
//     /* Don't do anything if we're not the last handle */
//     if (SystemHandleCount > 1) return;
//
//     /* Make sure we're a valid key body */
//     if (KeyBody->Type == CM_KEY_BODY_TYPE)
//     {
//         /* Don't do anything if we don't have a notify block */
//         if (!KeyBody->NotifyBlock) return;
//
//         /* This shouldn't happen yet */
//         ASSERT(FALSE);
//     }
// }
//
// NTSTATUS
// NTAPI
// CmpQueryKeyName(IN PVOID ObjectBody,
//                 IN BOOLEAN HasName,
//                 IN OUT POBJECT_NAME_INFORMATION ObjectNameInfo,
//                 IN ULONG Length,
//                 OUT PULONG ReturnLength,
//                 IN KPROCESSOR_MODE PreviousMode)
// {
//     PUNICODE_STRING KeyName;
//     ULONG BytesToCopy;
//     NTSTATUS Status = STATUS_SUCCESS;
//     PCM_KEY_BODY KeyBody = (PCM_KEY_BODY)ObjectBody;
//     PCM_KEY_CONTROL_BLOCK Kcb = KeyBody->KeyControlBlock;
//
//     /* Acquire hive lock */
//     CmpLockRegistry();
//
//     /* Lock KCB shared */
//     CmpAcquireKcbLockShared(Kcb);
//
//     /* Check if it's a deleted block */
//     if (Kcb->Delete)
//     {
//         /* Release the locks */
//         CmpReleaseKcbLock(Kcb);
//         CmpUnlockRegistry();
//
//         /* Let the caller know it's deleted */
//         return STATUS_KEY_DELETED;
//     }
//
//     /* Get the name */
//     KeyName = CmpConstructName(Kcb);
//
//     /* Release the locks */
//     CmpReleaseKcbLock(Kcb);
//     CmpUnlockRegistry();
//
//     /* Check if we got the name */
//     if (!KeyName) return STATUS_INSUFFICIENT_RESOURCES;
//
//     /* Set the returned length */
//     *ReturnLength = KeyName->Length + sizeof(OBJECT_NAME_INFORMATION) + sizeof(WCHAR);
//
//     /* Calculate amount of bytes to copy into the buffer */
//     BytesToCopy = KeyName->Length + sizeof(WCHAR);
//
//     /* Check if the provided buffer is too small to fit even anything */
//     if ((Length <= sizeof(OBJECT_NAME_INFORMATION)) ||
//         ((Length < *ReturnLength) && (BytesToCopy < sizeof(WCHAR))))
//     {
//         /* Free the buffer allocated by CmpConstructName */
//         ExFreePoolWithTag(KeyName, TAG_CM);
//
//         /* Return buffer length failure without writing anything there because nothing fits */
//         return STATUS_INFO_LENGTH_MISMATCH;
//     }
//
//     /* Check if the provided buffer can be partially written */
//     if (Length < *ReturnLength)
//     {
//         /* Yes, indicate so in the return status */
//         Status = STATUS_INFO_LENGTH_MISMATCH;
//
//         /* Calculate amount of bytes which the provided buffer could handle */
//         BytesToCopy = Length - sizeof(OBJECT_NAME_INFORMATION);
//     }
//
//     /* Remove the null termination character from the size */
//     BytesToCopy -= sizeof(WCHAR);
//
//     /* Fill in the result */
//     _SEH2_TRY
//     {
//         /* Return data to user */
//         ObjectNameInfo->Name.Buffer = (PWCHAR)(ObjectNameInfo + 1);
//         ObjectNameInfo->Name.MaximumLength = KeyName->Length;
//         ObjectNameInfo->Name.Length = KeyName->Length;
//
//         /* Copy string content*/
//         RtlCopyMemory(ObjectNameInfo->Name.Buffer,
//                       KeyName->Buffer,
//                       BytesToCopy);
//
//         /* Null terminate it */
//         ObjectNameInfo->Name.Buffer[BytesToCopy / sizeof(WCHAR)] = UNICODE_NULL;
//     }
//     _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//     {
//         /* Get the status */
//         Status = _SEH2_GetExceptionCode();
//     }
//     _SEH2_END;
//
//     /* Free the buffer allocated by CmpConstructName */
//     ExFreePoolWithTag(KeyName, TAG_CM);
//
//     /* Return status */
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// CmpInitHiveFromFile(IN PCUNICODE_STRING HiveName,
//                     IN ULONG HiveFlags,
//                     OUT PCMHIVE *Hive,
//                     IN OUT PBOOLEAN New,
//                     IN ULONG CheckFlags)
// {
//     ULONG HiveDisposition, LogDisposition;
//     HANDLE FileHandle = NULL, LogHandle = NULL;
//     NTSTATUS Status;
//     ULONG Operation, FileType;
//     PCMHIVE NewHive;
//     PAGED_CODE();
//
//     /* Assume failure */
//     *Hive = NULL;
//
//     /* Open or create the hive files */
//     Status = CmpOpenHiveFiles(HiveName,
//                               L".LOG",
//                               &FileHandle,
//                               &LogHandle,
//                               &HiveDisposition,
//                               &LogDisposition,
//                               *New,
//                               FALSE,
//                               TRUE,
//                               NULL);
//     if (!NT_SUCCESS(Status)) return Status;
//
//     /* Check if we have a log handle */
//     FileType = (LogHandle) ? HFILE_TYPE_LOG : HFILE_TYPE_PRIMARY;
//
//     /* Check if we created or opened the hive */
//     if (HiveDisposition == FILE_CREATED)
//     {
//         /* Do a create operation */
//         Operation = HINIT_CREATE;
//         *New = TRUE;
//     }
//     else
//     {
//         /* Open it as a file */
//         Operation = HINIT_FILE;
//         *New = FALSE;
//     }
//
//     /* Check if the system hives are opened in shared mode */
//     if (CmpShareSystemHives)
//     {
//         /* Then force using the primary hive */
//         FileType = HFILE_TYPE_PRIMARY;
//         if (LogHandle)
//         {
//             /* Get rid of the log handle */
//             ZwClose(LogHandle);
//             LogHandle = NULL;
//         }
//     }
//
//     /* Check if we're too late */
//     if (HvShutdownComplete)
//     {
//         /* Fail */
//         ZwClose(FileHandle);
//         if (LogHandle) ZwClose(LogHandle);
//         return STATUS_TOO_LATE;
//     }
//
//     /* Initialize the hive */
//     Status = CmpInitializeHive(&NewHive,
//                                Operation,
//                                HiveFlags,
//                                FileType,
//                                NULL,
//                                FileHandle,
//                                LogHandle,
//                                NULL,
//                                NULL,
//                                HiveName,
//                                CheckFlags);
//     if (!NT_SUCCESS(Status))
//     {
//         /* Fail */
//         ZwClose(FileHandle);
//         if (LogHandle) ZwClose(LogHandle);
//         return Status;
//     }
//
//     /* Success, return hive */
//     *Hive = NewHive;
//
//     /* Duplicate the hive name */
//     NewHive->FileFullPath.Buffer = ExAllocatePoolWithTag(PagedPool,
//                                                          HiveName->Length,
//                                                          TAG_CM);
//     if (NewHive->FileFullPath.Buffer)
//     {
//         /* Copy the string */
//         RtlCopyMemory(NewHive->FileFullPath.Buffer,
//                       HiveName->Buffer,
//                       HiveName->Length);
//         NewHive->FileFullPath.Length = HiveName->Length;
//         NewHive->FileFullPath.MaximumLength = HiveName->Length;
//     }
//
//     /* Return success */
//     return STATUS_SUCCESS;
// }
//
// CODE_SEG("INIT")
// NTSTATUS
// NTAPI
// CmpSetSystemValues(IN PLOADER_PARAMETER_BLOCK LoaderBlock)
// {
//     NTSTATUS Status;
//     OBJECT_ATTRIBUTES ObjectAttributes;
//     HANDLE KeyHandle;
//     UNICODE_STRING KeyName, ValueName = { 0, 0, NULL };
//
//     ASSERT(LoaderBlock != NULL);
//
//     /* Setup attributes for loader options */
//     RtlInitUnicodeString(&KeyName,
//                          L"\\REGISTRY\\MACHINE\\SYSTEM\\CurrentControlSet\\"
//                          L"Control");
//     InitializeObjectAttributes(&ObjectAttributes,
//                                &KeyName,
//                                OBJ_CASE_INSENSITIVE,
//                                NULL,
//                                NULL);
//     Status = NtOpenKey(&KeyHandle, KEY_WRITE, &ObjectAttributes);
//     if (!NT_SUCCESS(Status))
//         return Status;
//
//     /* Setup the value for the system start options */
//     RtlInitUnicodeString(&KeyName, L"SystemStartOptions");
//     Status = NtSetValueKey(KeyHandle,
//                            &KeyName,
//                            0,
//                            REG_SZ,
//                            CmpLoadOptions.Buffer,
//                            CmpLoadOptions.Length);
//     if (!NT_SUCCESS(Status))
//         goto Quit;
//
//     /* Setup the value for the system boot device in ARC format */
//     RtlInitUnicodeString(&KeyName, L"SystemBootDevice");
//     RtlCreateUnicodeStringFromAsciiz(&ValueName, LoaderBlock->ArcBootDeviceName);
//     Status = NtSetValueKey(KeyHandle,
//                            &KeyName,
//                            0,
//                            REG_SZ,
//                            ValueName.Buffer,
//                            ValueName.Length);
//
//     /* Free the temporary string */
//     RtlFreeUnicodeString(&ValueName);
//
// Quit:
//     /* Close the key and return */
//     NtClose(KeyHandle);
//     return Status;
// }
//
// static
// CODE_SEG("INIT")
// NTSTATUS
// CmpCreateHardwareProfile(HANDLE ControlSetHandle)
// {
//     OBJECT_ATTRIBUTES ObjectAttributes;
//     UNICODE_STRING KeyName;
//     HANDLE ProfilesHandle = NULL;
//     HANDLE ProfileHandle = NULL;
//     ULONG Disposition;
//     NTSTATUS Status;
//
//     DPRINT("CmpCreateHardwareProfile()\n");
//
//     /* Create the Hardware Profiles key */
//     RtlInitUnicodeString(&KeyName, L"Hardware Profiles");
//     InitializeObjectAttributes(&ObjectAttributes,
//                                &KeyName,
//                                OBJ_CASE_INSENSITIVE,
//                                ControlSetHandle,
//                                NULL);
//     Status = NtCreateKey(&ProfilesHandle,
//                          KEY_ALL_ACCESS,
//                          &ObjectAttributes,
//                          0,
//                          NULL,
//                          0,
//                          &Disposition);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("Creating the Hardware Profile key failed\n");
//         goto done;
//     }
//
//     /* Sanity check */
//     ASSERT(Disposition == REG_CREATED_NEW_KEY);
//
//     /* Create the 0000 key */
//     RtlInitUnicodeString(&KeyName, L"0000");
//     InitializeObjectAttributes(&ObjectAttributes,
//                                &KeyName,
//                                OBJ_CASE_INSENSITIVE,
//                                ProfilesHandle,
//                                NULL);
//     Status = NtCreateKey(&ProfileHandle,
//                          KEY_ALL_ACCESS,
//                          &ObjectAttributes,
//                          0,
//                          NULL,
//                          0,
//                          &Disposition);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("Creating the Hardware Profile\\0000 key failed\n");
//         goto done;
//     }
//
//     /* Sanity check */
//     ASSERT(Disposition == REG_CREATED_NEW_KEY);
//
// done:
//     if (ProfilesHandle)
//         NtClose(ProfilesHandle);
//
//     if (ProfileHandle)
//         NtClose(ProfileHandle);
//
//     DPRINT("CmpCreateHardwareProfile() done\n");
//
//     return Status;
// }
//
// CODE_SEG("INIT")
// NTSTATUS
// NTAPI
// CmpCreateControlSet(IN PLOADER_PARAMETER_BLOCK LoaderBlock)
// {
//     UNICODE_STRING ConfigName = RTL_CONSTANT_STRING(L"Control\\IDConfigDB");
//     UNICODE_STRING SelectName =
//         RTL_CONSTANT_STRING(L"\\Registry\\Machine\\System\\Select");
//     UNICODE_STRING KeyName;
//     OBJECT_ATTRIBUTES ObjectAttributes;
//     CHAR ValueInfoBuffer[128];
//     PKEY_VALUE_FULL_INFORMATION ValueInfo;
//     WCHAR UnicodeBuffer[128];
//     HANDLE SelectHandle = NULL;
//     HANDLE KeyHandle = NULL;
//     HANDLE ConfigHandle = NULL;
//     HANDLE ProfileHandle = NULL;
//     HANDLE ParentHandle = NULL;
//     ULONG ControlSet, HwProfile;
//     NTSTATUS Status;
//     ULONG ResultLength, Disposition;
//     PLOADER_PARAMETER_EXTENSION LoaderExtension;
//     PAGED_CODE();
//
//     /* ReactOS Hack: Hard-code current to 001 for SetupLdr */
//     if (LoaderBlock->RegistryBase == NULL)
//     {
//         /* Build the ControlSet001 key */
//         RtlInitUnicodeString(&KeyName,
//                              L"\\Registry\\Machine\\System\\ControlSet001");
//         InitializeObjectAttributes(&ObjectAttributes,
//                                    &KeyName,
//                                    OBJ_CASE_INSENSITIVE,
//                                    NULL,
//                                    NULL);
//         Status = NtCreateKey(&KeyHandle,
//                              KEY_ALL_ACCESS,
//                              &ObjectAttributes,
//                              0,
//                              NULL,
//                              0,
//                              &Disposition);
//         if (!NT_SUCCESS(Status))
//         {
//             DPRINT1("Failed to create ControlSet001 key: 0x%lx\n", Status);
//             goto Cleanup;
//         }
//
//         /* Create the Hardware Profile keys */
//         Status = CmpCreateHardwareProfile(KeyHandle);
//         if (!NT_SUCCESS(Status))
//         {
//             DPRINT1("Failed to create Hardware profile keys: 0x%lx\n", Status);
//             goto Cleanup;
//         }
//
//         /* Use hard-coded setting */
//         ControlSet = 1;
//     }
//     else
//     {
//         /* Open the select key */
//         InitializeObjectAttributes(&ObjectAttributes,
//                                    &SelectName,
//                                    OBJ_CASE_INSENSITIVE,
//                                    NULL,
//                                    NULL);
//         Status = NtOpenKey(&SelectHandle, KEY_READ, &ObjectAttributes);
//         if (!NT_SUCCESS(Status))
//         {
//             DPRINT1("Failed to open select key: 0x%lx\n", Status);
//             goto Cleanup;
//         }
//
//         /* Open the current value */
//         RtlInitUnicodeString(&KeyName, L"Current");
//         Status = NtQueryValueKey(SelectHandle,
//                                  &KeyName,
//                                  KeyValueFullInformation,
//                                  ValueInfoBuffer,
//                                  sizeof(ValueInfoBuffer),
//                                  &ResultLength);
//         if (!NT_SUCCESS(Status))
//         {
//             DPRINT1("Failed to open the Current value: 0x%lx\n", Status);
//             goto Cleanup;
//         }
//
//         /* Get the actual value pointer, and get the control set ID */
//         ValueInfo = (PKEY_VALUE_FULL_INFORMATION)ValueInfoBuffer;
//         ControlSet = *(PULONG)((PUCHAR)ValueInfo + ValueInfo->DataOffset);
//     }
//
//     /* Create the current control set key */
//     RtlInitUnicodeString(&KeyName,
//                          L"\\Registry\\Machine\\System\\CurrentControlSet");
//     InitializeObjectAttributes(&ObjectAttributes,
//                                &KeyName,
//                                OBJ_CASE_INSENSITIVE,
//                                NULL,
//                                NULL);
//     Status = NtCreateKey(&KeyHandle,
//                          KEY_CREATE_LINK,
//                          &ObjectAttributes,
//                          0,
//                          NULL,
//                          REG_OPTION_VOLATILE | REG_OPTION_CREATE_LINK,
//                          &Disposition);
//     if (!NT_SUCCESS(Status))
//         goto Cleanup;
//
//     /* Sanity check */
//     ASSERT(Disposition == REG_CREATED_NEW_KEY);
//
//     /* Initialize the target link name */
//     Status = RtlStringCbPrintfW(UnicodeBuffer, sizeof(UnicodeBuffer),
//                                 L"\\Registry\\Machine\\System\\ControlSet%03ld",
//                                 ControlSet);
//     if (!NT_SUCCESS(Status))
//         goto Cleanup;
//
//     RtlInitUnicodeString(&KeyName, UnicodeBuffer);
//
//     /* Set the value */
//     Status = NtSetValueKey(KeyHandle,
//                            &mut CmSymbolicLinkValueName,
//                            0,
//                            REG_LINK,
//                            KeyName.Buffer,
//                            KeyName.Length);
//     if (!NT_SUCCESS(Status))
//         goto Cleanup;
//
//     /* Get the configuration database key */
//     InitializeObjectAttributes(&ObjectAttributes,
//                                &ConfigName,
//                                OBJ_CASE_INSENSITIVE,
//                                KeyHandle,
//                                NULL);
//     Status = NtOpenKey(&ConfigHandle, KEY_READ, &ObjectAttributes);
//
//     /* Check if we don't have one */
//     if (!NT_SUCCESS(Status))
//     {
//         /* Cleanup and exit */
//         Status = STATUS_SUCCESS;
//         goto Cleanup;
//     }
//
//     /* ReactOS Hack: Hard-code current to 001 for SetupLdr */
//     if (LoaderBlock->RegistryBase == NULL)
//     {
//         HwProfile = 0;
//     }
//     else
//     {
//         /* Now get the current config */
//         RtlInitUnicodeString(&KeyName, L"CurrentConfig");
//         Status = NtQueryValueKey(ConfigHandle,
//                                  &KeyName,
//                                  KeyValueFullInformation,
//                                  ValueInfoBuffer,
//                                  sizeof(ValueInfoBuffer),
//                                  &ResultLength);
//
//         /* Set pointer to buffer */
//         ValueInfo = (PKEY_VALUE_FULL_INFORMATION)ValueInfoBuffer;
//
//         /* Check if we failed or got a non DWORD-value */
//         if (!NT_SUCCESS(Status) || (ValueInfo->Type != REG_DWORD))
//         {
//             Status = STATUS_SUCCESS;
//             goto Cleanup;
//         }
//
//         /* Get the hadware profile */
//         HwProfile = *(PULONG)((PUCHAR)ValueInfo + ValueInfo->DataOffset);
//     }
//
//     /* Open the hardware profile key */
//     RtlInitUnicodeString(&KeyName,
//                          L"\\Registry\\Machine\\System\\CurrentControlSet"
//                          L"\\Hardware Profiles");
//     InitializeObjectAttributes(&ObjectAttributes,
//                                &KeyName,
//                                OBJ_CASE_INSENSITIVE,
//                                NULL,
//                                NULL);
//     Status = NtOpenKey(&ParentHandle, KEY_READ, &ObjectAttributes);
//     if (!NT_SUCCESS(Status))
//     {
//         /* Exit and clean up */
//         Status = STATUS_SUCCESS;
//         goto Cleanup;
//     }
//
//     /* Build the profile name */
//     RtlStringCbPrintfW(UnicodeBuffer, sizeof(UnicodeBuffer),
//                        L"%04ld", HwProfile);
//     RtlInitUnicodeString(&KeyName, UnicodeBuffer);
//
//     /* Open the associated key */
//     InitializeObjectAttributes(&ObjectAttributes,
//                                &KeyName,
//                                OBJ_CASE_INSENSITIVE,
//                                ParentHandle,
//                                NULL);
//     Status = NtOpenKey(&ProfileHandle,
//                        KEY_READ | KEY_WRITE,
//                        &ObjectAttributes);
//     if (!NT_SUCCESS(Status))
//     {
//         /* Cleanup and exit */
//         Status = STATUS_SUCCESS;
//         goto Cleanup;
//     }
//
//     /* Check if we have a loader block extension */
//     LoaderExtension = LoaderBlock->Extension;
//     if (LoaderExtension)
//     {
//         DPRINT("ReactOS doesn't support NTLDR Profiles yet!\n");
//     }
//
//     /* Create the current hardware profile key */
//     RtlInitUnicodeString(&KeyName,
//                          L"\\Registry\\Machine\\System\\CurrentControlSet\\"
//                          L"Hardware Profiles\\Current");
//     InitializeObjectAttributes(&ObjectAttributes,
//                                &KeyName,
//                                OBJ_CASE_INSENSITIVE,
//                                NULL,
//                                NULL);
//     Status = NtCreateKey(&KeyHandle,
//                          KEY_CREATE_LINK,
//                          &ObjectAttributes,
//                          0,
//                          NULL,
//                          REG_OPTION_VOLATILE | REG_OPTION_CREATE_LINK,
//                          &Disposition);
//     if (NT_SUCCESS(Status))
//     {
//         /* Sanity check */
//         ASSERT(Disposition == REG_CREATED_NEW_KEY);
//
//         /* Create the profile name */
//         RtlStringCbPrintfW(UnicodeBuffer, sizeof(UnicodeBuffer),
//                            L"\\Registry\\Machine\\System\\CurrentControlSet\\"
//                            L"Hardware Profiles\\%04ld",
//                            HwProfile);
//         RtlInitUnicodeString(&KeyName, UnicodeBuffer);
//
//         /* Set it */
//         Status = NtSetValueKey(KeyHandle,
//                                &mut CmSymbolicLinkValueName,
//                                0,
//                                REG_LINK,
//                                KeyName.Buffer,
//                                KeyName.Length);
//     }
//
//     Status = STATUS_SUCCESS;
//
// Cleanup:
//     /* Close every opened handle */
//     if (SelectHandle) NtClose(SelectHandle);
//     if (KeyHandle) NtClose(KeyHandle);
//     if (ConfigHandle) NtClose(ConfigHandle);
//     if (ProfileHandle) NtClose(ProfileHandle);
//     if (ParentHandle) NtClose(ParentHandle);
//
//     DPRINT("CmpCreateControlSet() done\n");
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// CmpLinkHiveToMaster(IN PUNICODE_STRING LinkName,
//                     IN HANDLE RootDirectory,
//                     IN PCMHIVE RegistryHive,
//                     IN BOOLEAN Allocate,
//                     IN PSECURITY_DESCRIPTOR SecurityDescriptor)
// {
//     OBJECT_ATTRIBUTES ObjectAttributes;
//     NTSTATUS Status;
//     CM_PARSE_CONTEXT ParseContext = {0};
//     HANDLE KeyHandle;
//     PCM_KEY_BODY KeyBody;
//     PAGED_CODE();
//
//     /* Setup the object attributes */
//     InitializeObjectAttributes(&ObjectAttributes,
//                                LinkName,
//                                OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE,
//                                RootDirectory,
//                                SecurityDescriptor);
//
//     /* Setup the parse context */
//     ParseContext.CreateLink = TRUE;
//     ParseContext.CreateOperation = TRUE;
//     ParseContext.ChildHive.KeyHive = &RegistryHive->Hive;
//
//     /* Check if we have a root keycell or if we need to create it */
//     if (Allocate)
//     {
//         /* Create it */
//         ParseContext.ChildHive.KeyCell = HCELL_NIL;
//     }
//     else
//     {
//         /* We have one */
//         ParseContext.ChildHive.KeyCell = RegistryHive->Hive.BaseBlock->RootCell;
//     }
//
//     /* Create the link node */
//     Status = ObOpenObjectByName(&ObjectAttributes,
//                                 CmpKeyObjectType,
//                                 KernelMode,
//                                 NULL,
//                                 KEY_READ | KEY_WRITE,
//                                 (PVOID)&ParseContext,
//                                 &KeyHandle);
//     if (!NT_SUCCESS(Status)) return Status;
//
//     /* Mark the hive as clean */
//     RegistryHive->Hive.DirtyFlag = FALSE;
//
//     /* ReactOS Hack: Keep alive */
//     Status = ObReferenceObjectByHandle(KeyHandle,
//                                        0,
//                                        CmpKeyObjectType,
//                                        KernelMode,
//                                        (PVOID*)&KeyBody,
//                                        NULL);
//     ASSERT(NT_SUCCESS(Status));
//
//     /* Close the extra handle */
//     ZwClose(KeyHandle);
//     return STATUS_SUCCESS;
// }
//
// CODE_SEG("INIT")
// BOOLEAN
// NTAPI
// CmpInitializeSystemHive(IN PLOADER_PARAMETER_BLOCK LoaderBlock)
// {
//     static const UNICODE_STRING HiveName = RTL_CONSTANT_STRING(L"SYSTEM");
//     PVOID HiveBase;
//     ANSI_STRING LoadString;
//     PVOID Buffer;
//     ULONG Length;
//     NTSTATUS Status;
//     UNICODE_STRING KeyName;
//     PCMHIVE SystemHive = NULL;
//     PSECURITY_DESCRIPTOR SecurityDescriptor;
//
//     PAGED_CODE();
//
//     /* Setup the ansi string */
//     RtlInitAnsiString(&LoadString, LoaderBlock->LoadOptions);
//
//     /* Allocate the unicode buffer */
//     Length = LoadString.Length * sizeof(WCHAR) + sizeof(UNICODE_NULL);
//     Buffer = ExAllocatePoolWithTag(PagedPool, Length, TAG_CM);
//     if (!Buffer)
//     {
//         /* Fail */
//         KeBugCheckEx(BAD_SYSTEM_CONFIG_INFO, 3, 1, (ULONG_PTR)LoaderBlock, 0);
//     }
//
//     /* Setup the unicode string */
//     RtlInitEmptyUnicodeString(&CmpLoadOptions, Buffer, (USHORT)Length);
//
//     /* Add the load options and null-terminate */
//     Status = RtlAnsiStringToUnicodeString(&CmpLoadOptions, &LoadString, FALSE);
//     if (!NT_SUCCESS(Status))
//     {
//         return FALSE;
//     }
//
//     CmpLoadOptions.Buffer[LoadString.Length] = UNICODE_NULL;
//     CmpLoadOptions.Length += sizeof(WCHAR);
//
//     /* Get the System Hive base address */
//     HiveBase = LoaderBlock->RegistryBase;
//     ARM64_UART_TRACE("[CM] SysHive: HiveBase=");
//     if (HiveBase) ARM64_UART_TRACE("non-null"); else ARM64_UART_TRACE("NULL");
//     ARM64_UART_TRACE("\r\n");
//
//     ARM64_UART_TRACE("[CM] SysHive: CmpInitializeHive\r\n");
//     Status = CmpInitializeHive(&SystemHive,
//                                HiveBase ? HINIT_MEMORY : HINIT_CREATE,
//                                HIVE_NOLAZYFLUSH,
//                                HFILE_TYPE_ALTERNATE,
//                                HiveBase,
//                                NULL,
//                                NULL,
//                                NULL,
//                                NULL,
//                                &HiveName,
//                                HiveBase ? CM_CHECK_REGISTRY_PURGE_VOLATILES : CM_CHECK_REGISTRY_DONT_PURGE_VOLATILES);
//     if (!NT_SUCCESS(Status))
//     {
//         return FALSE;
//     }
//
//     ARM64_UART_TRACE("[CM] SysHive: CmpInitializeHive done\r\n");
//
//     /* Set the hive filename */
//     if (!RtlCreateUnicodeString(&SystemHive->FileFullPath, L"\\SystemRoot\\System32\\Config\\SYSTEM"))
//         return FALSE;
//
//     /* Load the system hive as volatile, if opened in shared mode */
//     if (HiveBase && CmpShareSystemHives)
//         SystemHive->Hive.HiveFlags = HIVE_VOLATILE;
//
//     /* Save the boot type */
//     CmpBootType = SystemHive->Hive.BaseBlock->BootType;
//
//     /* Are we in self-healing mode? */
//     if (!CmSelfHeal)
//     {
//         /* Disable self-healing internally and check if boot type wanted it */
//         CmpSelfHeal = FALSE;
//         if (CmpBootType & HBOOT_TYPE_SELF_HEAL)
//         {
//             /* We're disabled, so bugcheck */
//             KeBugCheckEx(BAD_SYSTEM_CONFIG_INFO,
//                          3,
//                          3,
//                          (ULONG_PTR)SystemHive,
//                          0);
//         }
//     }
//
//     /* Create the default security descriptor */
//     SecurityDescriptor = CmpHiveRootSecurityDescriptor();
//
//     /* Attach it to the system key */
//     /* Let CmpLinkHiveToMaster allocate a new hive if we got none from the LoaderBlock. */
//     ARM64_UART_TRACE("[CM] SysHive: CmpLinkHiveToMaster\r\n");
//     RtlInitUnicodeString(&KeyName, L"\\Registry\\Machine\\SYSTEM");
//     Status = CmpLinkHiveToMaster(&KeyName,
//                                  NULL,
//                                  SystemHive,
//                                  !HiveBase,
//                                  SecurityDescriptor);
//
//     /* Free the security descriptor */
//     ExFreePoolWithTag(SecurityDescriptor, TAG_CMSD);
//     if (!NT_SUCCESS(Status)) return FALSE;
//
//     /* Add the hive to the hive list */
//     CmpMachineHiveList[3].CmHive = SystemHive;
//
//     /* Success! */
//     return TRUE;
// }
//
// CODE_SEG("INIT")
// NTSTATUS
// NTAPI
// CmpCreateObjectTypes(VOID)
// {
//     OBJECT_TYPE_INITIALIZER ObjectTypeInitializer;
//     UNICODE_STRING Name;
//     GENERIC_MAPPING CmpKeyMapping = {KEY_READ,
//                                      KEY_WRITE,
//                                      KEY_EXECUTE,
//                                      KEY_ALL_ACCESS};
//     PAGED_CODE();
//
//     /* Initialize the Key object type */
//     RtlZeroMemory(&ObjectTypeInitializer, sizeof(ObjectTypeInitializer));
//     RtlInitUnicodeString(&Name, L"Key");
//     ObjectTypeInitializer.Length = sizeof(ObjectTypeInitializer);
//     ObjectTypeInitializer.DefaultPagedPoolCharge = sizeof(CM_KEY_BODY);
//     ObjectTypeInitializer.GenericMapping = CmpKeyMapping;
//     ObjectTypeInitializer.PoolType = PagedPool;
//     ObjectTypeInitializer.ValidAccessMask = KEY_ALL_ACCESS;
//     ObjectTypeInitializer.UseDefaultObject = TRUE;
//     ObjectTypeInitializer.DeleteProcedure = CmpDeleteKeyObject;
//     ObjectTypeInitializer.ParseProcedure = CmpParseKey;
//     ObjectTypeInitializer.SecurityProcedure = CmpSecurityMethod;
//     ObjectTypeInitializer.QueryNameProcedure = CmpQueryKeyName;
//     ObjectTypeInitializer.CloseProcedure = CmpCloseKeyObject;
//     ObjectTypeInitializer.SecurityRequired = TRUE;
//     ObjectTypeInitializer.InvalidAttributes = OBJ_EXCLUSIVE | OBJ_PERMANENT;
//
//     /* Create it */
//     return ObCreateObjectType(&Name, &ObjectTypeInitializer, NULL, &CmpKeyObjectType);
// }
//
// CODE_SEG("INIT")
// BOOLEAN
// NTAPI
// CmpCreateRootNode(IN PHHIVE Hive,
//                   IN PCWSTR Name,
//                   OUT PHCELL_INDEX Index)
// {
//     UNICODE_STRING KeyName;
//     PCM_KEY_NODE KeyCell;
//     PAGED_CODE();
//
//     /* Initialize the node name and allocate it */
//     RtlInitUnicodeString(&KeyName, Name);
//     *Index = HvAllocateCell(Hive,
//                             FIELD_OFFSET(CM_KEY_NODE, Name) +
//                             CmpNameSize(Hive, &KeyName),
//                             Stable,
//                             HCELL_NIL);
//     if (*Index == HCELL_NIL) return FALSE;
//
//     /* Set the cell index and get the data */
//     Hive->BaseBlock->RootCell = *Index;
//     KeyCell = (PCM_KEY_NODE)HvGetCell(Hive, *Index);
//     if (!KeyCell) return FALSE;
//
//     /* Setup the cell */
//     KeyCell->Signature = CM_KEY_NODE_SIGNATURE;
//     KeyCell->Flags = KEY_HIVE_ENTRY | KEY_NO_DELETE;
//     KeQuerySystemTime(&KeyCell->LastWriteTime);
//     KeyCell->Parent = HCELL_NIL;
//     KeyCell->SubKeyCounts[Stable] = 0;
//     KeyCell->SubKeyCounts[Volatile] = 0;
//     KeyCell->SubKeyLists[Stable] = HCELL_NIL;
//     KeyCell->SubKeyLists[Volatile] = HCELL_NIL;
//     KeyCell->ValueList.Count = 0;
//     KeyCell->ValueList.List = HCELL_NIL;
//     KeyCell->Security = HCELL_NIL;
//     KeyCell->Class = HCELL_NIL;
//     KeyCell->ClassLength = 0;
//     KeyCell->MaxNameLen = 0;
//     KeyCell->MaxClassLen = 0;
//     KeyCell->MaxValueNameLen = 0;
//     KeyCell->MaxValueDataLen = 0;
//
//     /* Copy the name (this will also set the length) */
//     KeyCell->NameLength = CmpCopyName(Hive, KeyCell->Name, &KeyName);
//
//     /* Check if the name was compressed and set the flag if so */
//     if (KeyCell->NameLength < KeyName.Length)
//         KeyCell->Flags |= KEY_COMP_NAME;
//
//     /* Return success */
//     HvReleaseCell(Hive, *Index);
//     return TRUE;
// }
//
// CODE_SEG("INIT")
// BOOLEAN
// NTAPI
// CmpCreateRegistryRoot(VOID)
// {
//     UNICODE_STRING KeyName;
//     OBJECT_ATTRIBUTES ObjectAttributes;
//     PCM_KEY_BODY RootKey;
//     HCELL_INDEX RootIndex;
//     NTSTATUS Status;
//     PCM_KEY_NODE KeyCell;
//     PSECURITY_DESCRIPTOR SecurityDescriptor;
//     PCM_KEY_CONTROL_BLOCK Kcb;
//     PAGED_CODE();
//
//     /* Setup the root node */
//     if (!CmpCreateRootNode(&CmiVolatileHive->Hive, L"REGISTRY", &RootIndex))
//     {
//         /* We failed */
//         return FALSE;
//     }
//
//     /* Create '\Registry' key. */
//     RtlInitUnicodeString(&KeyName, L"\\REGISTRY");
//     SecurityDescriptor = CmpHiveRootSecurityDescriptor();
//     InitializeObjectAttributes(&ObjectAttributes,
//                                &KeyName,
//                                OBJ_CASE_INSENSITIVE,
//                                NULL,
//                                SecurityDescriptor);
//     Status = ObCreateObject(KernelMode,
//                             CmpKeyObjectType,
//                             &ObjectAttributes,
//                             KernelMode,
//                             NULL,
//                             sizeof(CM_KEY_BODY),
//                             0,
//                             0,
//                             (PVOID*)&RootKey);
//     ExFreePoolWithTag(SecurityDescriptor, TAG_CMSD);
//     if (!NT_SUCCESS(Status)) return FALSE;
//
//     /* Sanity check, and get the key cell */
//     ASSERT((&CmiVolatileHive->Hive)->ReleaseCellRoutine == NULL);
//     KeyCell = (PCM_KEY_NODE)HvGetCell(&CmiVolatileHive->Hive, RootIndex);
//     if (!KeyCell)
//     {
//         ObDereferenceObject(RootKey);
//         return FALSE;
//     }
//
//     /* Create the KCB */
//     RtlInitUnicodeString(&KeyName, L"\\REGISTRY");
//     Kcb = CmpCreateKeyControlBlock(&CmiVolatileHive->Hive,
//                                    RootIndex,
//                                    KeyCell,
//                                    NULL,
//                                    0,
//                                    &KeyName);
//     if (!Kcb)
//     {
//         ObDereferenceObject(RootKey);
//         return FALSE;
//     }
//
//     /* Initialize the object */
//     RootKey->KeyControlBlock = Kcb;
//     RootKey->Type = CM_KEY_BODY_TYPE;
//     RootKey->NotifyBlock = NULL;
//     RootKey->ProcessID = PsGetCurrentProcessId();
//     RootKey->KcbLocked = FALSE;
//
//     /* Link with KCB */
//     EnlistKeyBodyWithKCB(RootKey, 0);
//
//     /* Insert the key into the namespace */
//     Status = ObInsertObject(RootKey,
//                             NULL,
//                             KEY_ALL_ACCESS,
//                             0,
//                             NULL,
//                             &CmpRegistryRootHandle);
//     if (!NT_SUCCESS(Status))
//     {
//         return FALSE;
//     }
//
//     /* Reference the key again so that we never lose it */
//     Status = ObReferenceObjectByHandle(CmpRegistryRootHandle,
//                                        KEY_READ,
//                                        NULL,
//                                        KernelMode,
//                                        (PVOID*)&RootKey,
//                                        NULL);
//     if (!NT_SUCCESS(Status))
//     {
//         ObDereferenceObject(RootKey);
//         return FALSE;
//     }
//
//     /* Completely successful */
//     return TRUE;
// }
//
// static PCWSTR
// CmpGetRegistryPath(VOID)
// {
//     PCWSTR ConfigPath;
//
//     /* Check if we are booted in setup */
//     if (!ExpInTextModeSetup)
//     {
//         ConfigPath = L"\\SystemRoot\\System32\\Config\\";
//     }
//     else
//     {
//         ConfigPath = L"\\SystemRoot\\";
//     }
//
//     DPRINT1("CmpGetRegistryPath: ConfigPath = '%S'\n", ConfigPath);
//
//     return ConfigPath;
// }
//
// /**
//  * @brief
//  * Checks if the primary and alternate backing hive are
//  * the same, by determining the time stamp of both hives.
//  *
//  * @param[in] FileName
//  * A pointer to a string containing the file name of the
//  * primary hive.
//  *
//  * @param[in] CmMainmHive
//  * A pointer to a CM hive descriptor associated with the
//  * primary hive.
//  *
//  * @param[in] AlternateHandle
//  * A handle to a file that represents the alternate hive.
//  *
//  * @param[in] Diverged
//  * A pointer to a boolean value, if both hives are the same
//  * it returns TRUE. Otherwise it returns FALSE.
//  */
// static
// VOID
// CmpHasAlternateHiveDiverged(
//     _In_ PCUNICODE_STRING FileName,
//     _In_ PCMHIVE CmMainmHive,
//     _In_ HANDLE AlternateHandle,
//     _Out_ PBOOLEAN Diverged)
// {
//     PHHIVE Hive, AlternateHive;
//     NTSTATUS Status;
//     PCMHIVE CmiAlternateHive;
//
//     /* Assume it has not diverged */
//     *Diverged = FALSE;
//
//     /* Initialize the SYSTEM alternate hive */
//     Status = CmpInitializeHive(&CmiAlternateHive,
//                                HINIT_FILE,
//                                0,
//                                HFILE_TYPE_PRIMARY,
//                                NULL,
//                                AlternateHandle,
//                                NULL,
//                                NULL,
//                                NULL,
//                                FileName,
//                                CM_CHECK_REGISTRY_DONT_PURGE_VOLATILES);
//     if (!NT_SUCCESS(Status))
//     {
//         /* Assume it has diverged... */
//         DPRINT1("Failed to initialize the alternate hive to check for diversion (Status 0x%lx)\n", Status);
//         *Diverged = TRUE;
//         return;
//     }
//
//     /*
//      * Check the timestamp of both hives. If they do not match they
//      * have diverged, the kernel has to synchronize the both hives.
//      */
//     Hive = &CmMainmHive->Hive;
//     AlternateHive = &CmiAlternateHive->Hive;
//     if (AlternateHive->BaseBlock->TimeStamp.QuadPart !=
//         Hive->BaseBlock->TimeStamp.QuadPart)
//     {
//         *Diverged = TRUE;
//     }
//
//     CmpDestroyHive(CmiAlternateHive);
// }
//
// _Function_class_(KSTART_ROUTINE)
// VOID
// NTAPI
// CmpLoadHiveThread(IN PVOID StartContext)
// {
//     WCHAR FileBuffer[64], RegBuffer[64];
//     PCWSTR ConfigPath;
//     UNICODE_STRING TempName, FileName, RegName;
//     ULONG i, ErrorResponse, WorkerCount, Length;
//     USHORT FileStart;
//     ULONG PrimaryDisposition, SecondaryDisposition, ClusterSize;
//     PCMHIVE CmHive;
//     HANDLE PrimaryHandle = NULL, AlternateHandle = NULL;
//     NTSTATUS Status = STATUS_SUCCESS;
//     PVOID ErrorParameters;
//     BOOLEAN HasDiverged;
//     PAGED_CODE();
//
//     /* Get the hive index, make sure it makes sense */
//     i = PtrToUlong(StartContext);
//     ASSERT(CmpMachineHiveList[i].Name != NULL);
//
//     /* We were started */
//     CmpMachineHiveList[i].ThreadStarted = TRUE;
//
//     /* Build the file name and registry name strings */
//     RtlInitEmptyUnicodeString(&FileName, FileBuffer, sizeof(FileBuffer));
//     RtlInitEmptyUnicodeString(&RegName, RegBuffer, sizeof(RegBuffer));
//
//     /* Now build the system root path */
//     ConfigPath = CmpGetRegistryPath();
//     RtlInitUnicodeString(&TempName, ConfigPath);
//     RtlAppendUnicodeStringToString(&FileName, &TempName);
//     FileStart = FileName.Length;
//
//     /* And build the registry root path */
//     RtlInitUnicodeString(&TempName, L"\\REGISTRY\\");
//     RtlAppendUnicodeStringToString(&RegName, &TempName);
//
//     /* Build the base name */
//     RtlInitUnicodeString(&TempName, CmpMachineHiveList[i].BaseName);
//     RtlAppendUnicodeStringToString(&RegName, &TempName);
//
//     /* Check if this is a child of the root */
//     if (RegName.Buffer[RegName.Length / sizeof(WCHAR) - 1] == OBJ_NAME_PATH_SEPARATOR)
//     {
//         /* Then setup the whole name */
//         RtlInitUnicodeString(&TempName, CmpMachineHiveList[i].Name);
//         RtlAppendUnicodeStringToString(&RegName, &TempName);
//     }
//
//     /* Now add the rest of the file name */
//     RtlInitUnicodeString(&TempName, CmpMachineHiveList[i].Name);
//     FileName.Length = FileStart;
//     RtlAppendUnicodeStringToString(&FileName, &TempName);
//     if (!CmpMachineHiveList[i].CmHive)
//     {
//         /* We need to allocate a new hive structure */
//         CmpMachineHiveList[i].Allocate = TRUE;
//
//         /* Load the hive file */
//         Status = CmpInitHiveFromFile(&FileName,
//                                      CmpMachineHiveList[i].HHiveFlags,
//                                      &CmHive,
//                                      &CmpMachineHiveList[i].Allocate,
//                                      CM_CHECK_REGISTRY_PURGE_VOLATILES);
//         if (!NT_SUCCESS(Status) ||
//             (!CmpShareSystemHives && !CmHive->FileHandles[HFILE_TYPE_LOG]))
//         {
//             /*
//              * We failed, or could not get a log file (unless
//              * the hive is shared), raise a hard error.
//              */
//             ErrorParameters = &FileName;
//             NtRaiseHardError(STATUS_CANNOT_LOAD_REGISTRY_FILE,
//                              1,
//                              1,
//                              (PULONG_PTR)&ErrorParameters,
//                              OptionOk,
//                              &ErrorResponse);
//         }
//
//         /* Set the hive flags and newly allocated hive pointer */
//         CmHive->Flags = CmpMachineHiveList[i].CmHiveFlags;
//         CmpMachineHiveList[i].CmHive2 = CmHive;
//     }
//     else
//     {
//         /* We already have a hive, is it volatile? */
//         CmHive = CmpMachineHiveList[i].CmHive;
//         if (!(CmHive->Hive.HiveFlags & HIVE_VOLATILE))
//         {
//             /* It's now, open the hive file and log */
//             Status = CmpOpenHiveFiles(&FileName,
//                                       L".ALT",
//                                       &PrimaryHandle,
//                                       &AlternateHandle,
//                                       &PrimaryDisposition,
//                                       &SecondaryDisposition,
//                                       TRUE,
//                                       TRUE,
//                                       FALSE,
//                                       &ClusterSize);
//             if (!NT_SUCCESS(Status) || !AlternateHandle)
//             {
//                 /* Couldn't open the hive or its alternate file, raise a hard error */
//                 ErrorParameters = &FileName;
//                 NtRaiseHardError(STATUS_CANNOT_LOAD_REGISTRY_FILE,
//                                  1,
//                                  1,
//                                  (PULONG_PTR)&ErrorParameters,
//                                  OptionOk,
//                                  &ErrorResponse);
//
//                 /* And bugcheck for posterity's sake */
//                 KeBugCheckEx(BAD_SYSTEM_CONFIG_INFO, 9, 0, i, Status);
//             }
//
//             /* Save the file handles. This should remove our sync hacks */
//             /*
//              * FIXME: Any hive that relies on the alternate hive for recovery purposes
//              * will only get an alternate hive. As a result, the LOG file would never
//              * get synced each time a write is done to the hive. In the future it would
//              * be best to adapt the code so that a primary hive can use a LOG and ALT
//              * hives at the same time.
//              */
//             CmHive->FileHandles[HFILE_TYPE_ALTERNATE] = AlternateHandle;
//             CmHive->FileHandles[HFILE_TYPE_PRIMARY] = PrimaryHandle;
//
//             /* Allow lazy flushing since the handles are there -- remove sync hacks */
//             //ASSERT(CmHive->Hive.HiveFlags & HIVE_NOLAZYFLUSH);
//             CmHive->Hive.HiveFlags &= ~HIVE_NOLAZYFLUSH;
//
//             /* Get the real size of the hive */
//             Length = CmHive->Hive.Storage[Stable].Length + HBLOCK_SIZE;
//
//             /* Check if the cluster size doesn't match */
//             if (CmHive->Hive.Cluster != ClusterSize)
//             {
//                 DPRINT1("FIXME: Support for CmHive->Hive.Cluster (%lu) != ClusterSize (%lu) is unimplemented!\n",
//                         CmHive->Hive.Cluster, ClusterSize);
//             }
//
//             /* Set the file size */
//             DPRINT("FIXME: Should set file size: %lu\n", Length);
//             //if (!CmpFileSetSize((PHHIVE)CmHive, HFILE_TYPE_PRIMARY, Length, Length))
//             //{
//                 /* This shouldn't fail */
//                 //ASSERT(FALSE);
//             //}
//
//             /* FreeLdr has recovered the hive with a log, we must do a flush */
//             if (CmHive->Hive.BaseBlock->BootRecover == HBOOT_BOOT_RECOVERED_BY_HIVE_LOG)
//             {
//                 DPRINT1("FreeLdr recovered the hive (hive 0x%p)\n", CmHive);
//                 RtlSetAllBits(&CmHive->Hive.DirtyVector);
//                 CmHive->Hive.DirtyCount = CmHive->Hive.DirtyVector.SizeOfBitMap;
//                 HvSyncHive((PHHIVE)CmHive);
//             }
//             else
//             {
//                 /*
//                  * Check whether the both primary and alternate hives are the same,
//                  * or that the primary or alternate were created for the first time.
//                  * Do a write against the alternate hive in these cases.
//                  */
//                 CmpHasAlternateHiveDiverged(&FileName,
//                                             CmHive,
//                                             AlternateHandle,
//                                             &HasDiverged);
//                 if (HasDiverged ||
//                     PrimaryDisposition == FILE_CREATED ||
//                     SecondaryDisposition == FILE_CREATED)
//                 {
//                     if (!HvWriteAlternateHive((PHHIVE)CmHive))
//                     {
//                         DPRINT1("Failed to write to alternate hive\n");
//                         goto Exit;
//                     }
//                 }
//             }
//
//             /* Finally, set our allocated hive to the same hive we've had */
//             CmpMachineHiveList[i].CmHive2 = CmHive;
//             ASSERT(CmpMachineHiveList[i].CmHive == CmpMachineHiveList[i].CmHive2);
//         }
//     }
//
// Exit:
//     /* We're done */
//     CmpMachineHiveList[i].ThreadFinished = TRUE;
//
//     /* Check if we're the last worker */
//     WorkerCount = InterlockedIncrement(&CmpLoadWorkerIncrement);
//     if (WorkerCount == CM_NUMBER_OF_MACHINE_HIVES)
//     {
//         /* Signal the event */
//         KeSetEvent(&CmpLoadWorkerEvent, 0, FALSE);
//     }
//
//     /* Kill the thread */
//     PsTerminateSystemThread(Status);
// }
//
// VOID
// NTAPI
// CmpInitializeHiveList(VOID)
// {
//     WCHAR FileBuffer[64], RegBuffer[64];
//     PCWSTR ConfigPath;
//     UNICODE_STRING TempName, FileName, RegName;
//     HANDLE Thread;
//     NTSTATUS Status;
//     ULONG i;
//     USHORT RegStart;
//     PSECURITY_DESCRIPTOR SecurityDescriptor;
//
//     PAGED_CODE();
//
//     /* Reenable hive writes now */
//     CmpNoWrite = FALSE;
//
//     /* Build the file name and registry name strings */
//     RtlInitEmptyUnicodeString(&FileName, FileBuffer, sizeof(FileBuffer));
//     RtlInitEmptyUnicodeString(&RegName, RegBuffer, sizeof(RegBuffer));
//
//     /* Now build the system root path */
//     ConfigPath = CmpGetRegistryPath();
//     RtlInitUnicodeString(&TempName, ConfigPath);
//     RtlAppendUnicodeStringToString(&FileName, &TempName);
//
//     /* And build the registry root path */
//     RtlInitUnicodeString(&TempName, L"\\REGISTRY\\");
//     RtlAppendUnicodeStringToString(&RegName, &TempName);
//     RegStart = RegName.Length;
//
//     /* Setup the event to synchronize workers */
//     KeInitializeEvent(&CmpLoadWorkerEvent, SynchronizationEvent, FALSE);
//
//     /* Enter special boot condition */
//     CmpSpecialBootCondition = TRUE;
//
//     /* Create the SD for the root hives */
//     SecurityDescriptor = CmpHiveRootSecurityDescriptor();
//
//     /* Loop every hive we care about */
//     for (i = 0; i < CM_NUMBER_OF_MACHINE_HIVES; i++)
//     {
//         /* Make sure the list is set up */
//         ASSERT(CmpMachineHiveList[i].Name != NULL);
//
//         /* Load this root hive as volatile, if opened in shared mode */
//         if (CmpShareSystemHives)
//             CmpMachineHiveList[i].HHiveFlags |= HIVE_VOLATILE;
//
//         /* Create a thread to handle this hive */
//         Status = PsCreateSystemThread(&Thread,
//                                       THREAD_ALL_ACCESS,
//                                       NULL,
//                                       0,
//                                       NULL,
//                                       CmpLoadHiveThread,
//                                       UlongToPtr(i));
//         if (NT_SUCCESS(Status))
//         {
//             /* We don't care about the handle -- the thread self-terminates */
//             ZwClose(Thread);
//         }
//         else
//         {
//             /* Can't imagine this happening */
//             KeBugCheckEx(BAD_SYSTEM_CONFIG_INFO, 9, 3, i, Status);
//         }
//     }
//
//     /* Make sure we've reached the end of the list */
//     ASSERT(CmpMachineHiveList[i].Name == NULL);
//
//     /* Wait for hive loading to finish */
//     KeWaitForSingleObject(&CmpLoadWorkerEvent,
//                           Executive,
//                           KernelMode,
//                           FALSE,
//                           NULL);
//
//     /* Exit the special boot condition and make sure all workers completed */
//     CmpSpecialBootCondition = FALSE;
//     ASSERT(CmpLoadWorkerIncrement == CM_NUMBER_OF_MACHINE_HIVES);
//
//     /* Loop hives again */
//     for (i = 0; i < CM_NUMBER_OF_MACHINE_HIVES; i++)
//     {
//         /* Make sure the thread ran and finished */
//         ASSERT(CmpMachineHiveList[i].ThreadFinished == TRUE);
//         ASSERT(CmpMachineHiveList[i].ThreadStarted == TRUE);
//
//         /* Check if this was a new hive */
//         if (!CmpMachineHiveList[i].CmHive)
//         {
//             /* Make sure we allocated something */
//             ASSERT(CmpMachineHiveList[i].CmHive2 != NULL);
//
//             /* Build the base name */
//             RegName.Length = RegStart;
//             RtlInitUnicodeString(&TempName, CmpMachineHiveList[i].BaseName);
//             RtlAppendUnicodeStringToString(&RegName, &TempName);
//
//             /* Check if this is a child of the root */
//             if (RegName.Buffer[RegName.Length / sizeof(WCHAR) - 1] == OBJ_NAME_PATH_SEPARATOR)
//             {
//                 /* Then setup the whole name */
//                 RtlInitUnicodeString(&TempName, CmpMachineHiveList[i].Name);
//                 RtlAppendUnicodeStringToString(&RegName, &TempName);
//             }
//
//             /* Now link the hive to its master */
//             Status = CmpLinkHiveToMaster(&RegName,
//                                          NULL,
//                                          CmpMachineHiveList[i].CmHive2,
//                                          CmpMachineHiveList[i].Allocate,
//                                          SecurityDescriptor);
//             if (Status != STATUS_SUCCESS)
//             {
//                 /* Linking needs to work */
//                 KeBugCheckEx(CONFIG_LIST_FAILED, 11, Status, i, (ULONG_PTR)&RegName);
//             }
//
//             /* Check if we had to allocate a new hive */
//             if (CmpMachineHiveList[i].Allocate)
//             {
//                 /* Sync the new hive */
//                 //HvSyncHive((PHHIVE)(CmpMachineHiveList[i].CmHive2));
//             }
//         }
//
//         /* Check if we created a new hive */
//         if (CmpMachineHiveList[i].CmHive2)
//         {
//             /* Add to HiveList key */
//             CmpAddToHiveFileList(CmpMachineHiveList[i].CmHive2);
//         }
//     }
//
//     /* Get rid of the SD */
//     ExFreePoolWithTag(SecurityDescriptor, TAG_CMSD);
//
//     /* Link SECURITY to SAM */
//     CmpLinkKeyToHive(L"\\Registry\\Machine\\Security\\SAM",
//                      L"\\Registry\\Machine\\SAM\\SAM");
//
//     /* Link S-1-5-18 to .Default */
//     CmpNoVolatileCreates = FALSE;
//     CmpLinkKeyToHive(L"\\Registry\\User\\S-1-5-18",
//                      L"\\Registry\\User\\.Default");
//     CmpNoVolatileCreates = TRUE;
// }
//
// CODE_SEG("INIT")
// BOOLEAN
// NTAPI
// CmInitSystem1(VOID)
// {
//     OBJECT_ATTRIBUTES ObjectAttributes;
//     UNICODE_STRING KeyName;
//     HANDLE KeyHandle;
//     NTSTATUS Status;
//     PCMHIVE HardwareHive;
//     PSECURITY_DESCRIPTOR SecurityDescriptor;
//     PAGED_CODE();
//
//     /* Check if this is PE-boot */
//     if (InitIsWinPEMode)
//     {
//         /* Set the registry in PE mode and load the system hives in shared mode */
//         CmpMiniNTBoot = TRUE;
//         CmpShareSystemHives = TRUE;
//     }
//     /* If we are in volatile boot mode, ALL hives without exception
//      * (system hives and others) will be loaded in shared mode */
//     if (CmpVolatileBoot)
//         CmpShareSystemHives = TRUE;
//
//     /* Initialize the hive list and lock */
//     InitializeListHead(&CmpHiveListHead);
//     ExInitializePushLock(&CmpHiveListHeadLock);
//     ExInitializePushLock(&CmpLoadHiveLock);
//
//     /* Initialize registry lock */
//     ExInitializeResourceLite(&CmpRegistryLock);
//
//     /* Initialize the cache */
//     CmpInitializeCache();
//
//     /* Initialize allocation and delayed dereferencing */
//     CmpInitCmPrivateAlloc();
//     CmpInitCmPrivateDelayAlloc();
//     CmpInitDelayDerefKCBEngine();
//
//     /* Initialize callbacks */
//     CmpInitCallback();
//
//     /* Initialize self healing */
//     KeInitializeGuardedMutex(&CmpSelfHealQueueLock);
//     InitializeListHead(&CmpSelfHealQueueListHead);
//
//     /* Save the current process and lock the registry */
//     CmpSystemProcess = PsGetCurrentProcess();
//
//     /* Create the key object types */
//     ARM64_UART_TRACE("[CM] CmpCreateObjectTypes\r\n");
//     Status = CmpCreateObjectTypes();
//     if (!NT_SUCCESS(Status))
//     {
//         /* Bugcheck */
//         KeBugCheckEx(CONFIG_INITIALIZATION_FAILED, 1, 1, Status, 0);
//     }
//
//     /* Build the master hive */
//     ARM64_UART_TRACE("[CM] CmpInitializeHive (master)\r\n");
//     Status = CmpInitializeHive(&CmiVolatileHive,
//                                HINIT_CREATE,
//                                HIVE_VOLATILE,
//                                HFILE_TYPE_PRIMARY,
//                                NULL,
//                                NULL,
//                                NULL,
//                                NULL,
//                                NULL,
//                                NULL,
//                                CM_CHECK_REGISTRY_DONT_PURGE_VOLATILES);
//     if (!NT_SUCCESS(Status))
//     {
//         /* Bugcheck */
//         KeBugCheckEx(CONFIG_INITIALIZATION_FAILED, 1, 2, Status, 0);
//     }
//
//     /* Create the \REGISTRY key node */
//     ARM64_UART_TRACE("[CM] CmpCreateRegistryRoot\r\n");
//     if (!CmpCreateRegistryRoot())
//     {
//         /* Bugcheck */
//         KeBugCheckEx(CONFIG_INITIALIZATION_FAILED, 1, 3, 0, 0);
//     }
//
//     /* Create the default security descriptor */
//     SecurityDescriptor = CmpHiveRootSecurityDescriptor();
//
//     /* Create '\Registry\Machine' key */
//     ARM64_UART_TRACE("[CM] NtCreateKey MACHINE\r\n");
//     RtlInitUnicodeString(&KeyName, L"\\REGISTRY\\MACHINE");
//     InitializeObjectAttributes(&ObjectAttributes,
//                                &KeyName,
//                                OBJ_CASE_INSENSITIVE,
//                                NULL,
//                                SecurityDescriptor);
//     Status = NtCreateKey(&KeyHandle,
//                          KEY_READ | KEY_WRITE,
//                          &ObjectAttributes,
//                          0,
//                          NULL,
//                          0,
//                          NULL);
//     if (!NT_SUCCESS(Status))
//     {
//         /* Bugcheck */
//         KeBugCheckEx(CONFIG_INITIALIZATION_FAILED, 1, 5, Status, 0);
//     }
//
//     /* Close the handle */
//     NtClose(KeyHandle);
//
//     /* Create '\Registry\User' key */
//     ARM64_UART_TRACE("[CM] NtCreateKey USER\r\n");
//     RtlInitUnicodeString(&KeyName, L"\\REGISTRY\\USER");
//     InitializeObjectAttributes(&ObjectAttributes,
//                                &KeyName,
//                                OBJ_CASE_INSENSITIVE,
//                                NULL,
//                                SecurityDescriptor);
//     Status = NtCreateKey(&KeyHandle,
//                          KEY_READ | KEY_WRITE,
//                          &ObjectAttributes,
//                          0,
//                          NULL,
//                          0,
//                          NULL);
//     if (!NT_SUCCESS(Status))
//     {
//         /* Bugcheck */
//         KeBugCheckEx(CONFIG_INITIALIZATION_FAILED, 1, 6, Status, 0);
//     }
//
//     /* Close the handle */
//     NtClose(KeyHandle);
//
//     /* After this point, do not allow creating keys in the master hive */
//     CmpNoVolatileCreates = TRUE;
//
//     /* Initialize the system hive */
//     ARM64_UART_TRACE("[CM] CmpInitializeSystemHive\r\n");
//     if (!CmpInitializeSystemHive(KeLoaderBlock))
//     {
//         /* Bugcheck */
//         KeBugCheckEx(CONFIG_INITIALIZATION_FAILED, 1, 7, 0, 0);
//     }
//
//     /* Create the 'CurrentControlSet' link */
//     ARM64_UART_TRACE("[CM] CmpCreateControlSet\r\n");
//     Status = CmpCreateControlSet(KeLoaderBlock);
//     if (!NT_SUCCESS(Status))
//     {
//         /* Bugcheck */
//         KeBugCheckEx(CONFIG_INITIALIZATION_FAILED, 1, 8, Status, 0);
//     }
//
//     /* Create the hardware hive */
//     ARM64_UART_TRACE("[CM] CmpInitializeHive (hardware)\r\n");
//     Status = CmpInitializeHive(&HardwareHive,
//                                HINIT_CREATE,
//                                HIVE_VOLATILE,
//                                HFILE_TYPE_PRIMARY,
//                                NULL,
//                                NULL,
//                                NULL,
//                                NULL,
//                                NULL,
//                                NULL,
//                                CM_CHECK_REGISTRY_DONT_PURGE_VOLATILES);
//     if (!NT_SUCCESS(Status))
//     {
//         /* Bugcheck */
//         KeBugCheckEx(CONFIG_INITIALIZATION_FAILED, 1, 11, Status, 0);
//     }
//
//     /* Add the hive to the hive list */
//     CmpMachineHiveList[0].CmHive = HardwareHive;
//
//     /* Attach it to the machine key */
//     RtlInitUnicodeString(&KeyName, L"\\Registry\\Machine\\HARDWARE");
//     Status = CmpLinkHiveToMaster(&KeyName,
//                                  NULL,
//                                  HardwareHive,
//                                  TRUE,
//                                  SecurityDescriptor);
//     if (!NT_SUCCESS(Status))
//     {
//         /* Bugcheck */
//         KeBugCheckEx(CONFIG_INITIALIZATION_FAILED, 1, 12, Status, 0);
//     }
//
//     /* Add to HiveList key */
//     CmpAddToHiveFileList(HardwareHive);
//
//     /* Free the security descriptor */
//     ExFreePoolWithTag(SecurityDescriptor, TAG_CMSD);
//
//     /* Fill out the Hardware key with the ARC Data from the Loader */
//     ARM64_UART_TRACE("[CM] CmpInitializeHardwareConfiguration\r\n");
//     Status = CmpInitializeHardwareConfiguration(KeLoaderBlock);
//     ARM64_UART_TRACE("[CM] CmpInitializeHardwareConfiguration returned\r\n");
//     if (!NT_SUCCESS(Status))
//     {
//         /* Bugcheck */
//         KeBugCheckEx(CONFIG_INITIALIZATION_FAILED, 1, 13, Status, 0);
//     }
//
//     /* Initialize machine-dependent information into the registry */
//     ARM64_UART_TRACE("[CM] CmpInitializeMachineDependentConfiguration\r\n");
//     Status = CmpInitializeMachineDependentConfiguration(KeLoaderBlock);
//     if (!NT_SUCCESS(Status))
//     {
//         /* Bugcheck */
//         KeBugCheckEx(CONFIG_INITIALIZATION_FAILED, 1, 14, Status, 0);
//     }
//
//     /* Initialize volatile registry settings */
//     Status = CmpSetSystemValues(KeLoaderBlock);
//     if (!NT_SUCCESS(Status))
//     {
//         /* Bugcheck */
//         KeBugCheckEx(CONFIG_INITIALIZATION_FAILED, 1, 15, Status, 0);
//     }
//
//     /* Free the load options */
//     ExFreePoolWithTag(CmpLoadOptions.Buffer, TAG_CM);
//
//     /* If we got here, all went well */
//     ARM64_UART_TRACE("[CM] CmInitSystem1 done\r\n");
//     return TRUE;
// }
//
// CODE_SEG("INIT")
// PUNICODE_STRING*
// NTAPI
// CmGetSystemDriverList(VOID)
// {
//     LIST_ENTRY DriverList;
//     OBJECT_ATTRIBUTES ObjectAttributes;
//     NTSTATUS Status;
//     PCM_KEY_BODY KeyBody;
//     PHHIVE Hive;
//     HCELL_INDEX RootCell, ControlCell;
//     HANDLE KeyHandle;
//     UNICODE_STRING KeyName;
//     PLIST_ENTRY NextEntry;
//     ULONG i;
//     PUNICODE_STRING* ServicePath = NULL;
//     BOOLEAN Success, AutoSelect;
//     PBOOT_DRIVER_LIST_ENTRY DriverEntry;
//     PAGED_CODE();
//
//     /* Initialize the driver list */
//     InitializeListHead(&DriverList);
//
//     /* Open the system hive key */
//     RtlInitUnicodeString(&KeyName, L"\\Registry\\Machine\\System");
//     InitializeObjectAttributes(&ObjectAttributes,
//                                &KeyName,
//                                OBJ_CASE_INSENSITIVE,
//                                NULL,
//                                NULL);
//     Status = NtOpenKey(&KeyHandle, KEY_READ, &ObjectAttributes);
//     if (!NT_SUCCESS(Status)) return NULL;
//
//     /* Reference the key object to get the root hive/cell to access directly */
//     Status = ObReferenceObjectByHandle(KeyHandle,
//                                        KEY_QUERY_VALUE,
//                                        CmpKeyObjectType,
//                                        KernelMode,
//                                        (PVOID*)&KeyBody,
//                                        NULL);
//     if (!NT_SUCCESS(Status))
//     {
//         /* Fail */
//         NtClose(KeyHandle);
//         return NULL;
//     }
//
//     /* Do all this under the registry lock */
//     CmpLockRegistryExclusive();
//
//     /* Get the hive and key cell */
//     Hive = KeyBody->KeyControlBlock->KeyHive;
//     RootCell = KeyBody->KeyControlBlock->KeyCell;
//
//     /* Open the current control set key */
//     RtlInitUnicodeString(&KeyName, L"Current");
//     ControlCell = CmpFindControlSet(Hive, RootCell, &KeyName, &AutoSelect);
//     if (ControlCell == HCELL_NIL) goto EndPath;
//
//     /* Find all system drivers */
//     Success = CmpFindDrivers(Hive, ControlCell, SystemLoad, NULL, &DriverList);
//     if (!Success) goto EndPath;
//
//     /* Sort by group/tag */
//     if (!CmpSortDriverList(Hive, ControlCell, &DriverList)) goto EndPath;
//
//     /* Remove circular dependencies (cycles) and sort */
//     if (!CmpResolveDriverDependencies(&DriverList)) goto EndPath;
//
//     /* Loop the list to count drivers */
//     for (i = 0, NextEntry = DriverList.Flink;
//          NextEntry != &DriverList;
//          i++, NextEntry = NextEntry->Flink);
//
//     /* Allocate the array */
//     ServicePath = ExAllocatePool(NonPagedPool, (i + 1) * sizeof(PUNICODE_STRING));
//     if (!ServicePath) KeBugCheckEx(CONFIG_INITIALIZATION_FAILED, 2, 1, 0, 0);
//
//     /* Loop the driver list */
//     for (i = 0, NextEntry = DriverList.Flink;
//          NextEntry != &DriverList;
//          i++, NextEntry = NextEntry->Flink)
//     {
//         /* Get the entry */
//         DriverEntry = CONTAINING_RECORD(NextEntry, BOOT_DRIVER_LIST_ENTRY, Link);
//
//         /* Allocate the path for the caller */
//         ServicePath[i] = ExAllocatePool(NonPagedPool, sizeof(UNICODE_STRING));
//         if (!ServicePath[i])
//         {
//             KeBugCheckEx(CONFIG_INITIALIZATION_FAILED, 2, 1, 0, 0);
//         }
//
//         /* Duplicate the registry path */
//         Status = RtlDuplicateUnicodeString(RTL_DUPLICATE_UNICODE_STRING_NULL_TERMINATE,
//                                            &DriverEntry->RegistryPath,
//                                            ServicePath[i]);
//         if (!NT_SUCCESS(Status))
//         {
//             KeBugCheckEx(CONFIG_INITIALIZATION_FAILED, 2, 1, 0, 0);
//         }
//     }
//
//     /* Terminate the list */
//     ServicePath[i] = NULL;
//
// EndPath:
//     /* Free the driver list if we had one */
//     if (!IsListEmpty(&DriverList)) CmpFreeDriverList(Hive, &DriverList);
//
//     /* Unlock the registry */
//     CmpUnlockRegistry();
//
//     /* Close the key handle and dereference the object, then return the path */
//     ObDereferenceObject(KeyBody);
//     NtClose(KeyHandle);
//     return ServicePath;
// }
//
// VOID
// NTAPI
// CmpLockRegistryExclusive(VOID)
// {
//     /* Enter a critical region and lock the registry */
//     KeEnterCriticalRegion();
//     ExAcquireResourceExclusiveLite(&CmpRegistryLock, TRUE);
//
//     /* Sanity check */
//     ASSERT(CmpFlushStarveWriters == 0);
//     RtlGetCallersAddress(&CmpRegistryLockCaller, &CmpRegistryLockCallerCaller);
// }
//
// VOID
// NTAPI
// CmpLockRegistry(VOID)
// {
//     /* Enter a critical region */
//     KeEnterCriticalRegion();
//
//     /* Check if we have to starve writers */
//     if (CmpFlushStarveWriters)
//     {
//         /* Starve exlusive waiters */
//         ExAcquireSharedStarveExclusive(&CmpRegistryLock, TRUE);
//     }
//     else
//     {
//         /* Just grab the lock */
//         ExAcquireResourceSharedLite(&CmpRegistryLock, TRUE);
//     }
// }
//
// BOOLEAN
// NTAPI
// CmpTestRegistryLock(VOID)
// {
//     /* Test the lock */
//     return !ExIsResourceAcquiredSharedLite(&CmpRegistryLock) ? FALSE : TRUE;
// }
//
// BOOLEAN
// NTAPI
// CmpTestRegistryLockExclusive(VOID)
// {
//     /* Test the lock */
//     return !ExIsResourceAcquiredExclusiveLite(&CmpRegistryLock) ? FALSE : TRUE;
// }
//
// VOID
// NTAPI
// CmpLockHiveFlusherExclusive(IN PCMHIVE Hive)
// {
//     /* Lock the flusher. We should already be in a critical section */
//     CMP_ASSERT_REGISTRY_LOCK_OR_LOADING(Hive);
//     ASSERT((ExIsResourceAcquiredShared(Hive->FlusherLock) == 0) &&
//            (ExIsResourceAcquiredExclusiveLite(Hive->FlusherLock) == 0));
//     ExAcquireResourceExclusiveLite(Hive->FlusherLock, TRUE);
// }
//
// VOID
// NTAPI
// CmpLockHiveFlusherShared(IN PCMHIVE Hive)
// {
//     /* Lock the flusher. We should already be in a critical section */
//     CMP_ASSERT_REGISTRY_LOCK_OR_LOADING(Hive);
//     ASSERT((ExIsResourceAcquiredShared(Hive->FlusherLock) == 0) &&
//            (ExIsResourceAcquiredExclusiveLite(Hive->FlusherLock) == 0));
//     ExAcquireResourceSharedLite(Hive->FlusherLock, TRUE);
// }
//
// VOID
// NTAPI
// CmpUnlockHiveFlusher(IN PCMHIVE Hive)
// {
//     /* Sanity check */
//     CMP_ASSERT_REGISTRY_LOCK_OR_LOADING(Hive);
//     CMP_ASSERT_FLUSH_LOCK(Hive);
//
//     /* Release the lock */
//     ExReleaseResourceLite(Hive->FlusherLock);
// }
//
// BOOLEAN
// NTAPI
// CmpTestHiveFlusherLockShared(IN PCMHIVE Hive)
// {
//     /* Test the lock */
//     return !ExIsResourceAcquiredSharedLite(Hive->FlusherLock) ? FALSE : TRUE;
// }
//
// BOOLEAN
// NTAPI
// CmpTestHiveFlusherLockExclusive(IN PCMHIVE Hive)
// {
//     /* Test the lock */
//     return !ExIsResourceAcquiredExclusiveLite(Hive->FlusherLock) ? FALSE : TRUE;
// }
//
// VOID
// NTAPI
// CmpUnlockRegistry(VOID)
// {
//     /* Sanity check */
//     CMP_ASSERT_REGISTRY_LOCK();
//
//     /* Check if we should flush the registry */
//     if (CmpFlushOnLockRelease)
//     {
//         /* The registry should be exclusively locked for this */
//         CMP_ASSERT_EXCLUSIVE_REGISTRY_LOCK();
//
//         /* Flush the registry */
//         CmpDoFlushAll(TRUE);
//         CmpFlushOnLockRelease = FALSE;
//     }
//
//     /* Release the lock and leave the critical region */
//     ExReleaseResourceLite(&CmpRegistryLock);
//     KeLeaveCriticalRegion();
// }
//
// VOID
// NTAPI
// CmpAcquireTwoKcbLocksExclusiveByKey(IN ULONG ConvKey1,
//                                     IN ULONG ConvKey2)
// {
//     ULONG Index1, Index2;
//
//     /* Sanity check */
//     CMP_ASSERT_REGISTRY_LOCK();
//
//     /* Get hash indexes */
//     Index1 = GET_HASH_INDEX(ConvKey1);
//     Index2 = GET_HASH_INDEX(ConvKey2);
//
//     /* See which one is highest */
//     if (Index1 < Index2)
//     {
//         /* Grab them in the proper order */
//         CmpAcquireKcbLockExclusiveByKey(ConvKey1);
//         CmpAcquireKcbLockExclusiveByKey(ConvKey2);
//     }
//     else
//     {
//         /* Grab the second one first, then the first */
//         CmpAcquireKcbLockExclusiveByKey(ConvKey2);
//         if (Index1 != Index2) CmpAcquireKcbLockExclusiveByKey(ConvKey1);
//     }
// }
//
// VOID
// NTAPI
// CmpReleaseTwoKcbLockByKey(IN ULONG ConvKey1,
//                           IN ULONG ConvKey2)
// {
//     ULONG Index1, Index2;
//
//     /* Sanity check */
//     CMP_ASSERT_REGISTRY_LOCK();
//
//     /* Get hash indexes */
//     Index1 = GET_HASH_INDEX(ConvKey1);
//     Index2 = GET_HASH_INDEX(ConvKey2);
//     ASSERT((GET_HASH_ENTRY(CmpCacheTable, ConvKey2)->Owner == KeGetCurrentThread()) ||
//            CmpTestRegistryLockExclusive());
//
//     /* See which one is highest */
//     if (Index1 < Index2)
//     {
//         /* Grab them in the proper order */
//         ASSERT((GET_HASH_ENTRY(CmpCacheTable, ConvKey1)->Owner == KeGetCurrentThread()) ||
//                CmpTestRegistryLockExclusive());
//         CmpReleaseKcbLockByKey(ConvKey2);
//         CmpReleaseKcbLockByKey(ConvKey1);
//     }
//     else
//     {
//         /* Release the first one first, then the second */
//         if (Index1 != Index2)
//         {
//             ASSERT((GET_HASH_ENTRY(CmpCacheTable, ConvKey1)->Owner == KeGetCurrentThread()) ||
//                    CmpTestRegistryLockExclusive());
//             CmpReleaseKcbLockByKey(ConvKey1);
//         }
//         CmpReleaseKcbLockByKey(ConvKey2);
//     }
// }
//
// VOID
// NTAPI
// CmShutdownSystem(VOID)
// {
//     PLIST_ENTRY ListEntry;
//     PCMHIVE Hive;
//
//     /* Kill the workers */
//     if (!CmFirstTime) CmpShutdownWorkers();
//
//     /* Flush all hives */
//     CmpLockRegistryExclusive();
//     CmpDoFlushAll(TRUE);
//
//     /* Close all hive files */
//     ListEntry = CmpHiveListHead.Flink;
//     while (ListEntry != &CmpHiveListHead)
//     {
//         Hive = CONTAINING_RECORD(ListEntry, CMHIVE, HiveList);
//
//         CmpCloseHiveFiles(Hive);
//
//         ListEntry = ListEntry->Flink;
//     }
//
//     /*
//      * As we flushed all the hives on the disk,
//      * tell the system we do not want any further
//      * registry flushing or syncing at this point
//      * since we are shutting down the registry anyway.
//      */
//     HvShutdownComplete = TRUE;
//
//     CmpUnlockRegistry();
// }
//
// VOID
// NTAPI
// CmpSetVersionData(VOID)
// {
//     NTSTATUS Status;
//     OBJECT_ATTRIBUTES ObjectAttributes;
//     UNICODE_STRING KeyName;
//     UNICODE_STRING ValueName;
//     UNICODE_STRING ValueData;
//     ANSI_STRING TempString;
//     HANDLE SoftwareKeyHandle = NULL;
//     HANDLE MicrosoftKeyHandle = NULL;
//     HANDLE WindowsNtKeyHandle = NULL;
//     HANDLE CurrentVersionKeyHandle = NULL;
//     WCHAR Buffer[128]; // Buffer large enough to contain a full ULONG in decimal
//                        // representation, and the full 'CurrentType' string.
//
//     /*
//      * Open the 'HKLM\Software\Microsoft\Windows NT\CurrentVersion' key
//      * (create the intermediate subkeys if needed).
//      */
//
//     RtlInitUnicodeString(&KeyName, L"\\REGISTRY\\MACHINE\\SOFTWARE");
//     InitializeObjectAttributes(&ObjectAttributes,
//                                &KeyName,
//                                OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE,
//                                NULL,
//                                NULL);
//     Status = NtCreateKey(&SoftwareKeyHandle,
//                          KEY_CREATE_SUB_KEY,
//                          &ObjectAttributes,
//                          0,
//                          NULL,
//                          0,
//                          NULL);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("Failed to create key %wZ (Status: %08lx)\n", &KeyName, Status);
//         return;
//     }
//
//     RtlInitUnicodeString(&KeyName, L"Microsoft");
//     InitializeObjectAttributes(&ObjectAttributes,
//                                &KeyName,
//                                OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE,
//                                SoftwareKeyHandle,
//                                NULL);
//     Status = NtCreateKey(&MicrosoftKeyHandle,
//                          KEY_CREATE_SUB_KEY,
//                          &ObjectAttributes,
//                          0,
//                          NULL,
//                          0,
//                          NULL);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("Failed to create key %wZ (Status: %08lx)\n", &KeyName, Status);
//         goto Quit;
//     }
//
//     RtlInitUnicodeString(&KeyName, L"Windows NT");
//     InitializeObjectAttributes(&ObjectAttributes,
//                                &KeyName,
//                                OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE,
//                                MicrosoftKeyHandle,
//                                NULL);
//     Status = NtCreateKey(&WindowsNtKeyHandle,
//                          KEY_CREATE_SUB_KEY,
//                          &ObjectAttributes,
//                          0,
//                          NULL,
//                          0,
//                          NULL);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("Failed to create key %wZ (Status: %08lx)\n", &KeyName, Status);
//         goto Quit;
//     }
//
//     RtlInitUnicodeString(&KeyName, L"CurrentVersion");
//     InitializeObjectAttributes(&ObjectAttributes,
//                                &KeyName,
//                                OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE,
//                                WindowsNtKeyHandle,
//                                NULL);
//     Status = NtCreateKey(&CurrentVersionKeyHandle,
//                          KEY_CREATE_SUB_KEY | KEY_SET_VALUE,
//                          &ObjectAttributes,
//                          0,
//                          NULL,
//                          0,
//                          NULL);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("Failed to create key %wZ (Status: %08lx)\n", &KeyName, Status);
//         goto Quit;
//     }
//
//     /* Set the 'CurrentVersion' value */
//     RtlInitUnicodeString(&ValueName, L"CurrentVersion");
//     NtSetValueKey(CurrentVersionKeyHandle,
//                   &ValueName,
//                   0,
//                   REG_SZ,
//                   CmVersionString.Buffer,
//                   CmVersionString.Length + sizeof(WCHAR));
//
//     /* Set the 'CurrentBuildNumber' value */
//     RtlInitUnicodeString(&ValueName, L"CurrentBuildNumber");
//     RtlInitEmptyUnicodeString(&ValueData, Buffer, sizeof(Buffer));
//     RtlIntegerToUnicodeString(NtBuildNumber & 0xFFFF, 10, &ValueData);
//     NtSetValueKey(CurrentVersionKeyHandle,
//                   &ValueName,
//                   0,
//                   REG_SZ,
//                   ValueData.Buffer,
//                   ValueData.Length + sizeof(WCHAR));
//
//     /* Set the 'BuildLab' value */
//     RtlInitUnicodeString(&ValueName, L"BuildLab");
//     RtlInitAnsiString(&TempString, NtBuildLab);
//     Status = RtlAnsiStringToUnicodeString(&ValueData, &TempString, FALSE);
//     if (NT_SUCCESS(Status))
//     {
//         NtSetValueKey(CurrentVersionKeyHandle,
//                       &ValueName,
//                       0,
//                       REG_SZ,
//                       ValueData.Buffer,
//                       ValueData.Length + sizeof(WCHAR));
//     }
//
//     /* Set the 'CurrentType' value */
//     RtlInitUnicodeString(&ValueName, L"CurrentType");
//     RtlStringCbPrintfW(Buffer, sizeof(Buffer),
//                        L"%s %s",
// #ifdef CONFIG_SMP
//                        L"Multiprocessor"
// #else
//                        L"Uniprocessor"
// #endif
//                        ,
// #if (DBG == 1)
//                        L"Checked"
// #else
//                        L"Free"
// #endif
//                        );
//     RtlInitUnicodeString(&ValueData, Buffer);
//     NtSetValueKey(CurrentVersionKeyHandle,
//                   &ValueName,
//                   0,
//                   REG_SZ,
//                   ValueData.Buffer,
//                   ValueData.Length + sizeof(WCHAR));
//
//     /* Set the 'CSDVersion' value */
//     RtlInitUnicodeString(&ValueName, L"CSDVersion");
//     if (CmCSDVersionString.Length != 0)
//     {
//         NtSetValueKey(CurrentVersionKeyHandle,
//                       &ValueName,
//                       0,
//                       REG_SZ,
//                       CmCSDVersionString.Buffer,
//                       CmCSDVersionString.Length + sizeof(WCHAR));
//     }
//     else
//     {
//         NtDeleteValueKey(CurrentVersionKeyHandle, &ValueName);
//     }
//
//     /* Set the 'CSDBuildNumber' value */
//     RtlInitUnicodeString(&ValueName, L"CSDBuildNumber");
//     if (CmNtSpBuildNumber != 0)
//     {
//         RtlInitEmptyUnicodeString(&ValueData, Buffer, sizeof(Buffer));
//         RtlIntegerToUnicodeString(CmNtSpBuildNumber, 10, &ValueData);
//         NtSetValueKey(CurrentVersionKeyHandle,
//                       &ValueName,
//                       0,
//                       REG_SZ,
//                       ValueData.Buffer,
//                       ValueData.Length + sizeof(WCHAR));
//     }
//     else
//     {
//         NtDeleteValueKey(CurrentVersionKeyHandle, &ValueName);
//     }
//
//     /* Set the 'SystemRoot' value */
//     RtlInitUnicodeString(&ValueName, L"SystemRoot");
//     NtSetValueKey(CurrentVersionKeyHandle,
//                   &ValueName,
//                   0,
//                   REG_SZ,
//                   NtSystemRoot.Buffer,
//                   NtSystemRoot.Length + sizeof(WCHAR));
//
// Quit:
//     /* Close the keys */
//     if (CurrentVersionKeyHandle != NULL)
//         NtClose(CurrentVersionKeyHandle);
//
//     if (WindowsNtKeyHandle != NULL)
//         NtClose(WindowsNtKeyHandle);
//
//     if (MicrosoftKeyHandle != NULL)
//         NtClose(MicrosoftKeyHandle);
//
//     if (SoftwareKeyHandle != NULL)
//         NtClose(SoftwareKeyHandle);
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
