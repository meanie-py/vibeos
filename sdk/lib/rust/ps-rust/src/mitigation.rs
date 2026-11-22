//! Process mitigation policies: the dynamic-code policy (ACG), which Chrome's sandbox turns on
//! for its own browser process (ProhibitDynamicCode + AllowThreadOptOut, so V8's JIT thread
//! can opt back out). Every other policy keeps kernel32's answer: a request that turns a
//! restriction on is refused (ERROR_NOT_SUPPORTED), never pretended.
//!
//! Not in the C: ReactOS has no ProcessMitigationPolicy class. The layout is Windows':
//! PROCESS_MITIGATION_POLICY_INFORMATION { PROCESS_MITIGATION_POLICY Policy; ULONG Flags },
//! classes ProcessMitigationPolicy (52) and ThreadDynamicCodePolicyInfo (42).
//!
//! Enforcement (ARM3 virtual.rs / section.rs, through `PsVibeDynamicCodeBlocked`): while a
//! process has ProhibitDynamicCode on, its own threads (except one that opted out) may not
//! allocate executable memory, add execute to non-image memory, or map a non-image section
//! executable. As on Windows, ACG restricts a process only on its own address space.
//! Divergence: Windows relocates images in the kernel; our ntdll loader relocates in user
//! mode by making image pages writable for a moment, so re-protecting IMAGE pages is
//! allowed. No new executable memory can appear either way.

use ntkrnl_sys::base::STATUS_SUCCESS;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::ddk::*;
use ntkrnl_sys::ob::*;
use ntkrnl_sys::probe::*;
use ntkrnl_sys::ps::PsGetCurrentProcess;

extern "C" {
    static PsProcessType: POBJECT_TYPE;
    static PsThreadType: POBJECT_TYPE;
}

pub const PROCESS_MITIGATION_POLICY_CLASS: u32 = 52; // ProcessMitigationPolicy
pub const THREAD_DYNAMIC_CODE_POLICY_CLASS: u32 = 42; // ThreadDynamicCodePolicyInfo
const PROCESS_DYNAMIC_CODE_POLICY: i32 = 2;

// PROCESS_MITIGATION_DYNAMIC_CODE_POLICY flags.
const PROHIBIT_DYNAMIC_CODE: u32 = 0x1;
const ALLOW_THREAD_OPT_OUT: u32 = 0x2;
const ALLOW_REMOTE_DOWNGRADE: u32 = 0x4;
// AuditProhibitDynamicCode (0x8) would need an audit log instead of a refusal: not offered.
const KNOWN: u32 = PROHIBIT_DYNAMIC_CODE | ALLOW_THREAD_OPT_OUT | ALLOW_REMOTE_DOWNGRADE;

/// THREAD_DYNAMIC_CODE_ALLOW: this thread may generate code despite the process policy.
/// Kept in a bit of ETHREAD.SameThreadPassiveFlags no other code uses (bits 0-5 are taken),
/// which only the thread itself writes, at passive level: the opt-out is same-thread only.
const ETHREAD_DYNAMIC_CODE_OPT_OUT: u32 = 0x8000_0000;

const STATUS_DYNAMIC_CODE_BLOCKED: NTSTATUS = 0xC000_0604_u32 as NTSTATUS;

#[repr(C)]
#[derive(Copy, Clone)]
struct PolicyInformation {
    policy: i32,
    flags: u32,
}

/// The policy word lives in EPROCESS.Spares[0], unused by everything else (ndk pstypes.h).
#[inline]
unsafe fn policy_word(process: *mut _EPROCESS) -> *mut u32 {
    core::ptr::addr_of_mut!((*process).Spares[0])
}

#[inline]
unsafe fn thread_flags(thread: *mut _ETHREAD) -> *mut u32 {
    core::ptr::addr_of_mut!((*thread).__bindgen_anon_8.SameThreadPassiveFlags)
}

/// Is dynamic code forbidden for a change the CURRENT thread makes to `process`'s memory?
/// Called by ARM3 for allocations, protection changes and section views that ask for execute.
#[no_mangle]
pub unsafe extern "C" fn PsVibeDynamicCodeBlocked(process: PVOID) -> u8 {
    let current = PsGetCurrentProcess() as *mut _EPROCESS;
    if process as *mut _EPROCESS != current {
        return 0;
    }
    let word = core::ptr::read_volatile(policy_word(current));
    if word & PROHIBIT_DYNAMIC_CODE == 0 {
        return 0;
    }
    let thread = PsGetCurrentThread() as *mut _ETHREAD;
    if word & ALLOW_THREAD_OPT_OUT != 0
        && core::ptr::read_volatile(thread_flags(thread)) & ETHREAD_DYNAMIC_CODE_OPT_OUT != 0
    {
        return 0;
    }
    1
}

unsafe fn reference_process(handle: HANDLE, access: ACCESS_MASK, mode: KPROCESSOR_MODE) -> Result<*mut _EPROCESS, NTSTATUS> {
    let mut process: PVOID = core::ptr::null_mut();
    let status = ObReferenceObjectByHandle(handle, access, PsProcessType, mode, &mut process, core::ptr::null_mut());
    if status < 0 {
        Err(status)
    } else {
        Ok(process as *mut _EPROCESS)
    }
}

/// NtSetInformationProcess(ProcessMitigationPolicy). Only the current process may set its
/// own policy (SetProcessMitigationPolicy has no process handle), and once ProhibitDynamicCode
/// is on it stays on.
pub unsafe fn set_process_mitigation(handle: HANDLE, buffer: PVOID, length: ULONG, mode: KPROCESSOR_MODE) -> NTSTATUS {
    if length as usize != core::mem::size_of::<PolicyInformation>() {
        return STATUS_INFO_LENGTH_MISMATCH;
    }
    let info = match probe_read_user::<PolicyInformation>(buffer as *const PolicyInformation, mode) {
        Ok(v) => v,
        Err(s) => return s,
    };
    let process = match reference_process(handle, PROCESS_SET_INFORMATION, mode) {
        Ok(p) => p,
        Err(s) => return s,
    };
    let status = if process != PsGetCurrentProcess() as *mut _EPROCESS {
        STATUS_INVALID_PARAMETER
    } else if info.policy != PROCESS_DYNAMIC_CODE_POLICY {
        // The other policies are restrictions nothing here enforces: turning one on is
        // refused, a request that turns nothing on matches what a query reports.
        if info.flags == 0 { STATUS_SUCCESS } else { STATUS_NOT_SUPPORTED }
    } else if info.flags & !KNOWN != 0 {
        STATUS_NOT_SUPPORTED
    } else {
        let word = policy_word(process);
        let old = core::ptr::read_volatile(word);
        if old & PROHIBIT_DYNAMIC_CODE != 0 && info.flags & PROHIBIT_DYNAMIC_CODE == 0 {
            STATUS_ACCESS_DENIED // a process cannot lift ACG from itself
        } else {
            let atomic = &*(word as *const core::sync::atomic::AtomicU32);
            atomic.fetch_or(info.flags, core::sync::atomic::Ordering::SeqCst);
            STATUS_SUCCESS
        }
    };
    ObDereferenceObject(process as PVOID);
    status
}

/// NtQueryInformationProcess(ProcessMitigationPolicy): the caller names the policy in the
/// buffer; the dynamic-code policy reports its flags, every other policy reports none on.
pub unsafe fn query_process_mitigation(handle: HANDLE, buffer: PVOID, length: ULONG, ret: PULONG, mode: KPROCESSOR_MODE) -> NTSTATUS {
    if length as usize != core::mem::size_of::<PolicyInformation>() {
        return STATUS_INFO_LENGTH_MISMATCH;
    }
    let mut info = match probe_read_user::<PolicyInformation>(buffer as *const PolicyInformation, mode) {
        Ok(v) => v,
        Err(s) => return s,
    };
    let process = match reference_process(handle, PROCESS_QUERY_LIMITED_INFORMATION, mode) {
        Ok(p) => p,
        Err(s) => return s,
    };
    info.flags = if info.policy == PROCESS_DYNAMIC_CODE_POLICY {
        core::ptr::read_volatile(policy_word(process)) & KNOWN
    } else {
        0
    };
    ObDereferenceObject(process as PVOID);
    if let Err(s) = probe_write_user::<PolicyInformation>(buffer as *mut PolicyInformation, info, mode) {
        return s;
    }
    if !ret.is_null() {
        if let Err(s) = probe_write_user::<ULONG>(ret, length, mode) {
            return s;
        }
    }
    STATUS_SUCCESS
}

/// NtSetInformationThread(ThreadDynamicCodePolicyInfo): a ULONG, 1 = this thread may generate
/// code. Same-thread only, and only when the process policy allows thread opt-out.
pub unsafe fn set_thread_dynamic_code(handle: HANDLE, buffer: PVOID, length: ULONG, mode: KPROCESSOR_MODE) -> NTSTATUS {
    if length as usize != core::mem::size_of::<ULONG>() {
        return STATUS_INFO_LENGTH_MISMATCH;
    }
    let value = match probe_read_user::<ULONG>(buffer as *const ULONG, mode) {
        Ok(v) => v,
        Err(s) => return s,
    };
    let mut thread: PVOID = core::ptr::null_mut();
    let status = ObReferenceObjectByHandle(handle, THREAD_SET_INFORMATION, PsThreadType, mode, &mut thread, core::ptr::null_mut());
    if status < 0 {
        return status;
    }
    let current = PsGetCurrentThread() as *mut _ETHREAD;
    let status = if thread as *mut _ETHREAD != current {
        STATUS_INVALID_PARAMETER
    } else if value > 1 {
        STATUS_INVALID_PARAMETER
    } else if core::ptr::read_volatile(policy_word(PsGetCurrentProcess() as *mut _EPROCESS)) & ALLOW_THREAD_OPT_OUT == 0 {
        STATUS_ACCESS_DENIED
    } else {
        let flags = thread_flags(current);
        let v = core::ptr::read_volatile(flags);
        core::ptr::write_volatile(
            flags,
            if value != 0 { v | ETHREAD_DYNAMIC_CODE_OPT_OUT } else { v & !ETHREAD_DYNAMIC_CODE_OPT_OUT },
        );
        STATUS_SUCCESS
    };
    ObDereferenceObject(thread);
    status
}
