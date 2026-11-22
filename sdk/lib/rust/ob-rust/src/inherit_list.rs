//! PROC_THREAD_ATTRIBUTE_HANDLE_LIST: inherit only the listed handles.
//!
//! vibeos addition; ReactOS has no equivalent. Windows carries the list in
//! NtCreateUserProcess's attribute list. kernel32 here creates processes with
//! NtCreateProcessEx, which has no slot for one, so kernel32 stages the list on the creating
//! thread first (`ObVibeStageInheritList`, reached through a private NtSetInformationThread
//! class) and clears it afterwards. While a list is staged, ObInitProcess inherits from the
//! current process only the handle-table entries it names. Windows 11 measures as follows
//! (tools/payloads/hostapps/inhlist.win11.txt): only listed handles reach the child, and a
//! non-inheritable, repeated or `bInheritHandles=FALSE` list fails CreateProcess with
//! ERROR_INVALID_PARAMETER. kernel32 enforces those failures; this module only filters.
//!
//! Entries are matched by handle-table entry address rather than handle value, because
//! ExDupHandleTable's callback is given entries, not handles.
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::ex::{ExAllocatePoolWithTag, ExFreePoolWithTag};
use ntkrnl_sys::ke::{KeAcquireSpinLock, KeReleaseSpinLock, KSPIN_LOCK};
use ntkrnl_sys::mm::PagedPool;
use ntkrnl_sys::ps::PsGetCurrentProcess;
use ntkrnl_sys::ddk::{
    _EPROCESS, ExMapHandleToPointer, ExUnlockHandleTableEntry, KeEnterCriticalRegion,
    KeLeaveCriticalRegion, KPROCESSOR_MODE, HANDLE, PETHREAD, PHANDLE_TABLE, PHANDLE_TABLE_ENTRY,
};

extern "system" {
    fn PsGetCurrentThread() -> PETHREAD;
}

/// Windows caps nothing visible here; this bounds a kernel allocation from a user length.
const INHERIT_LIST_MAX_HANDLES: usize = 4096;
const INHERIT_LIST_SLOTS: usize = 16;
const TAG_OB_INHERIT_LIST: u32 = u32::from_le_bytes(*b"ObIL");

#[derive(Clone, Copy)]
struct Slot {
    thread: PETHREAD,
    entries: *mut PHANDLE_TABLE_ENTRY,
    count: usize,
    /// Set by ObInitProcess for the duration of its ExDupHandleTable call, so that a staged
    /// list filters that one inheritance and nothing else the thread does.
    active: bool,
}
const EMPTY: Slot = Slot { thread: core::ptr::null_mut(), entries: core::ptr::null_mut(), count: 0, active: false };

static mut LOCK: KSPIN_LOCK = 0;
static mut TABLE: [Slot; INHERIT_LIST_SLOTS] = [EMPTY; INHERIT_LIST_SLOTS];

unsafe fn take_slot(thread: PETHREAD) -> Option<Slot> {
    let mut irql = 0;
    KeAcquireSpinLock(core::ptr::addr_of_mut!(LOCK), &mut irql);
    let table = &mut *core::ptr::addr_of_mut!(TABLE);
    let found = table.iter_mut().find(|s| s.thread == thread).map(|s| core::mem::replace(s, EMPTY));
    KeReleaseSpinLock(core::ptr::addr_of_mut!(LOCK), irql);
    found
}

unsafe fn free_slot(slot: Slot) {
    if !slot.entries.is_null() {
        ExFreePoolWithTag(slot.entries as PVOID, TAG_OB_INHERIT_LIST);
    }
}

/// Stage (or, with `length` 0, clear) the current thread's handle list. `handles` is
/// `length` bytes of HANDLE values in the caller's address space.
#[no_mangle]
pub unsafe extern "system" fn ObVibeStageInheritList(
    handles: *const HANDLE,
    length: ULONG,
    mode: KPROCESSOR_MODE,
) -> NTSTATUS {
    let thread = PsGetCurrentThread();
    if let Some(old) = take_slot(thread) {
        free_slot(old);
    }
    let length = length as usize;
    if length == 0 {
        return STATUS_SUCCESS;
    }
    let width = core::mem::size_of::<HANDLE>();
    if length % width != 0 || length / width > INHERIT_LIST_MAX_HANDLES {
        return STATUS_INVALID_PARAMETER;
    }
    let count = length / width;
    let captured = ExAllocatePoolWithTag(PagedPool, length, TAG_OB_INHERIT_LIST) as *mut HANDLE;
    if captured.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }
    if let Err(status) = ntkrnl_sys::probe::safe_copy_from_user(
        captured as *mut core::ffi::c_void,
        handles as *const core::ffi::c_void,
        length,
        width,
        mode,
    ) {
        ExFreePoolWithTag(captured as PVOID, TAG_OB_INHERIT_LIST);
        return status;
    }

    // Resolve in place: a HANDLE and a PHANDLE_TABLE_ENTRY are both pointer-sized.
    let entries = captured as *mut PHANDLE_TABLE_ENTRY;
    let table = (*(PsGetCurrentProcess() as *mut _EPROCESS)).ObjectTable as PHANDLE_TABLE;
    KeEnterCriticalRegion();
    for i in 0..count {
        let entry = ExMapHandleToPointer(table, *captured.add(i));
        if entry.is_null() {
            KeLeaveCriticalRegion();
            ExFreePoolWithTag(captured as PVOID, TAG_OB_INHERIT_LIST);
            return STATUS_INVALID_HANDLE;
        }
        ExUnlockHandleTableEntry(table, entry);
        *entries.add(i) = entry;
    }
    KeLeaveCriticalRegion();

    let mut irql = 0;
    KeAcquireSpinLock(core::ptr::addr_of_mut!(LOCK), &mut irql);
    let table = &mut *core::ptr::addr_of_mut!(TABLE);
    let free = table.iter_mut().find(|s| s.thread.is_null());
    let status = match free {
        Some(s) => {
            *s = Slot { thread, entries, count, active: false };
            STATUS_SUCCESS
        }
        None => STATUS_INSUFFICIENT_RESOURCES,
    };
    KeReleaseSpinLock(core::ptr::addr_of_mut!(LOCK), irql);
    if status != STATUS_SUCCESS {
        ExFreePoolWithTag(captured as PVOID, TAG_OB_INHERIT_LIST);
    }
    status
}

/// ObInitProcess: start or stop filtering the inheritance this thread is about to perform.
/// Returns whether a list is staged. Stopping also discards the list: it applies to one
/// process creation.
pub(crate) unsafe fn set_active(on: bool) -> bool {
    let thread = PsGetCurrentThread();
    if !on {
        if let Some(old) = take_slot(thread) {
            free_slot(old);
        }
        return false;
    }
    let mut irql = 0;
    KeAcquireSpinLock(core::ptr::addr_of_mut!(LOCK), &mut irql);
    let table = &mut *core::ptr::addr_of_mut!(TABLE);
    let staged = match table.iter_mut().find(|s| s.thread == thread) {
        Some(s) => {
            s.active = true;
            true
        }
        None => false,
    };
    KeReleaseSpinLock(core::ptr::addr_of_mut!(LOCK), irql);
    staged
}

/// ObpDuplicateHandleCallback: `None` when no list is filtering this thread's inheritance,
/// otherwise whether the parent's `entry` is on the list.
pub(crate) unsafe fn allows(entry: PHANDLE_TABLE_ENTRY) -> Option<bool> {
    let thread = PsGetCurrentThread();
    let mut irql = 0;
    KeAcquireSpinLock(core::ptr::addr_of_mut!(LOCK), &mut irql);
    let table = &*core::ptr::addr_of!(TABLE);
    let verdict = table
        .iter()
        .find(|s| s.thread == thread && s.active)
        .map(|s| (0..s.count).any(|i| *s.entries.add(i) == entry));
    KeReleaseSpinLock(core::ptr::addr_of_mut!(LOCK), irql);
    verdict
}
