//! Vista+ kernel32 exports that real desktop apps import (Wine dlls/kernelbase semantics):
//! proc/thread attribute lists, Create{Event,Mutex,WaitableTimer}Ex, per-thread error mode,
//! RaiseFailFastException and GetLogicalProcessorInformationEx.
use ntdll_sys::nt::*;
use crate::prelude::*;
use crate::k32_externs::BaseFormatObjectAttributes;
use core::ffi::c_void;

/* ------------------------------------------------------------------------- *
 * Proc/thread attribute lists (Wine layout: header + array of attr/size/value)
 * ------------------------------------------------------------------------- */

#[repr(C)]
#[derive(Clone, Copy)]
struct ProcThreadAttr {
    attr: usize,
    size: usize,
    value: *mut c_void,
}

#[repr(C)]
struct ProcThreadAttrList {
    mask: u32,
    size: u32,
    count: u32,
    unk: usize,
    attrs: [ProcThreadAttr; 1],
}

const PTA_INPUT: usize = 0x0002_0000;
const PTA_THREAD: usize = 0x0001_0000;
const PTA_ADDITIVE: usize = 0x0004_0000;
const PTA_NUMBER: usize = 0x0000_FFFF;

const PTA_PARENT_PROCESS: usize = 0 | PTA_INPUT;
const PTA_HANDLE_LIST: usize = 2 | PTA_INPUT;
const PTA_GROUP_AFFINITY: usize = 3 | PTA_THREAD | PTA_INPUT;
const PTA_PREFERRED_NODE: usize = 4 | PTA_INPUT;
const PTA_IDEAL_PROCESSOR: usize = 5 | PTA_THREAD | PTA_INPUT;
const PTA_UMS_THREAD: usize = 6 | PTA_THREAD | PTA_INPUT;
const PTA_MITIGATION_POLICY: usize = 7 | PTA_INPUT;
const PTA_SECURITY_CAPABILITIES: usize = 9 | PTA_INPUT;
const PTA_PROTECTION_LEVEL: usize = 11 | PTA_INPUT;
const PTA_JOB_LIST: usize = 13 | PTA_INPUT;
const PTA_CHILD_PROCESS_POLICY: usize = 14 | PTA_INPUT;
const PTA_ALL_APP_PACKAGES_POLICY: usize = 15 | PTA_INPUT;
const PTA_WIN32K_FILTER: usize = 16 | PTA_INPUT;
const PTA_DESKTOP_APP_POLICY: usize = 18 | PTA_INPUT;
const PTA_PSEUDOCONSOLE: usize = 22 | PTA_INPUT;
const PTA_MACHINE_TYPE: usize = 25 | PTA_INPUT;

const ERROR_GEN_FAILURE_: u32 = 31;
const ERROR_OBJECT_NAME_EXISTS_: u32 = 698;

#[no_mangle]
pub unsafe extern "system" fn InitializeProcThreadAttributeList(
    lpAttributeList: *mut c_void,
    dwAttributeCount: DWORD,
    dwFlags: DWORD,
    lpSize: *mut usize,
) -> BOOL {
    if lpSize.is_null() || dwFlags != 0 || dwAttributeCount > 27 {
        SetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }
    let needed = core::mem::offset_of!(ProcThreadAttrList, attrs)
        + dwAttributeCount as usize * core::mem::size_of::<ProcThreadAttr>();
    if lpAttributeList.is_null() || *lpSize < needed {
        *lpSize = needed;
        SetLastError(ERROR_INSUFFICIENT_BUFFER);
        return 0;
    }
    let list = lpAttributeList as *mut ProcThreadAttrList;
    (*list).mask = 0;
    (*list).size = dwAttributeCount;
    (*list).count = 0;
    (*list).unk = 0;
    1
}

#[no_mangle]
pub unsafe extern "system" fn UpdateProcThreadAttribute(
    lpAttributeList: *mut c_void,
    dwFlags: DWORD,
    Attribute: usize,
    lpValue: *mut c_void,
    cbSize: usize,
    lpPreviousValue: *mut c_void,
    lpReturnSize: *mut usize,
) -> BOOL {
    let list = lpAttributeList as *mut ProcThreadAttrList;
    if list.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }
    /* Only PROC_THREAD_ATTRIBUTE_REPLACE_VALUE (1) is defined for dwFlags. */
    if dwFlags & !1u32 != 0 {
        SetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }
    if (*list).count >= (*list).size {
        SetLastError(ERROR_GEN_FAILURE_);
        return 0;
    }
    /* lpPreviousValue / lpReturnSize are reserved and must be NULL. */
    if !lpPreviousValue.is_null() || !lpReturnSize.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }

    let ptr_sz = core::mem::size_of::<*mut c_void>();
    let size_ok = match Attribute & !PTA_ADDITIVE {
        PTA_PARENT_PROCESS => cbSize == ptr_sz,
        PTA_HANDLE_LIST => cbSize != 0 && cbSize % ptr_sz == 0,
        PTA_GROUP_AFFINITY => cbSize == core::mem::size_of::<GROUP_AFFINITY>(),
        PTA_PREFERRED_NODE => cbSize == 2,
        /* PROCESSOR_NUMBER: WORD Group, BYTE Number, BYTE Reserved */
        PTA_IDEAL_PROCESSOR => cbSize == 4,
        PTA_UMS_THREAD => cbSize == 0 || cbSize == ptr_sz,
        PTA_MITIGATION_POLICY => cbSize == 4 || cbSize == 8 || cbSize == 16,
        /* SECURITY_CAPABILITIES: PSID, PSID_AND_ATTRIBUTES, DWORD count, DWORD reserved */
        PTA_SECURITY_CAPABILITIES => cbSize == 2 * ptr_sz + 8,
        PTA_PROTECTION_LEVEL => cbSize == 4,
        PTA_JOB_LIST => cbSize != 0 && cbSize % ptr_sz == 0,
        PTA_CHILD_PROCESS_POLICY | PTA_ALL_APP_PACKAGES_POLICY | PTA_DESKTOP_APP_POLICY => cbSize == 4,
        PTA_WIN32K_FILTER => cbSize == 8,
        PTA_PSEUDOCONSOLE => cbSize == ptr_sz,
        PTA_MACHINE_TYPE => cbSize == 2,
        _ => {
            SetLastError(ERROR_NOT_SUPPORTED);
            return 0;
        }
    };
    if !size_ok {
        SetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }

    let bit = 1u32 << (Attribute & PTA_NUMBER);
    if (*list).mask & bit != 0 {
        SetLastError(ERROR_OBJECT_NAME_EXISTS_);
        return 0;
    }
    (*list).mask |= bit;

    let slot = (&raw mut (*list).attrs as *mut ProcThreadAttr).add((*list).count as usize);
    (*slot).attr = Attribute;
    (*slot).size = cbSize;
    (*slot).value = lpValue;
    (*list).count += 1;
    1
}

#[no_mangle]
pub unsafe extern "system" fn DeleteProcThreadAttributeList(lpAttributeList: *mut c_void) {
    /* The list owns no allocations (values are caller memory); nothing to free. */
    let _ = lpAttributeList;
}

/* ------------------------------------------------------------------------- *
 * Create*Ex synchronization objects
 * ------------------------------------------------------------------------- */

/// Convert an ANSI object name through the TEB static string like the non-Ex A variants.
unsafe fn ansi_name(lpName: LPCSTR) -> Option<LPCWSTR> {
    let UnicodeCache: PUNICODE_STRING = &raw mut (*NtCurrentTeb()).StaticUnicodeString;
    let mut AnsiName: ANSI_STRING = core::mem::zeroed();
    RtlInitAnsiString(&raw mut AnsiName, lpName as PCSZ);
    let Status = RtlAnsiStringToUnicodeString(UnicodeCache, &raw mut AnsiName, 0);
    if NT_SUCCESS(Status) {
        return Some((*UnicodeCache).Buffer as LPCWSTR);
    }
    if Status == STATUS_BUFFER_OVERFLOW {
        SetLastError(ERROR_FILENAME_EXCED_RANGE as DWORD);
    } else {
        BaseSetLastNTError(Status);
    }
    None
}

unsafe fn finish_create(Status: NTSTATUS, Handle: HANDLE) -> HANDLE {
    if NT_SUCCESS(Status) {
        if Status == STATUS_OBJECT_NAME_EXISTS {
            SetLastError(ERROR_ALREADY_EXISTS as DWORD);
        } else {
            SetLastError(ERROR_SUCCESS as DWORD);
        }
        return Handle;
    }
    BaseSetLastNTError(Status);
    core::ptr::null_mut()
}

#[no_mangle]
pub unsafe extern "system" fn CreateEventExW(
    lpEventAttributes: LPSECURITY_ATTRIBUTES,
    lpName: LPCWSTR,
    dwFlags: DWORD,
    dwDesiredAccess: DWORD,
) -> HANDLE {
    let mut Handle: HANDLE = core::ptr::null_mut();
    let mut ObjectName: _UNICODE_STRING = core::mem::zeroed();
    let mut LocalAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    if !lpName.is_null() {
        RtlInitUnicodeString(&raw mut ObjectName, lpName as PCWSTR);
    }
    let ObjectAttributes = BaseFormatObjectAttributes(
        &raw mut LocalAttributes,
        lpEventAttributes as ntdll_sys::nt::LPSECURITY_ATTRIBUTES,
        if !lpName.is_null() { &raw mut ObjectName } else { core::ptr::null_mut() },
    );
    let Status = NtCreateEvent(
        &raw mut Handle,
        dwDesiredAccess as ACCESS_MASK,
        ObjectAttributes,
        (if dwFlags & 0x1 != 0 {
            _EVENT_TYPE_NotificationEvent as ::core::ffi::c_int
        } else {
            _EVENT_TYPE_SynchronizationEvent as ::core::ffi::c_int
        }) as EVENT_TYPE,
        (dwFlags & 0x2 != 0) as BOOLEAN,
    );
    finish_create(Status, Handle)
}

#[no_mangle]
pub unsafe extern "system" fn CreateEventExA(
    lpEventAttributes: LPSECURITY_ATTRIBUTES,
    lpName: LPCSTR,
    dwFlags: DWORD,
    dwDesiredAccess: DWORD,
) -> HANDLE {
    if lpName.is_null() {
        return CreateEventExW(lpEventAttributes, core::ptr::null(), dwFlags, dwDesiredAccess);
    }
    match ansi_name(lpName) {
        Some(w) => CreateEventExW(lpEventAttributes, w, dwFlags, dwDesiredAccess),
        None => core::ptr::null_mut(),
    }
}

#[no_mangle]
pub unsafe extern "system" fn CreateMutexExW(
    lpMutexAttributes: LPSECURITY_ATTRIBUTES,
    lpName: LPCWSTR,
    dwFlags: DWORD,
    dwDesiredAccess: DWORD,
) -> HANDLE {
    let mut Handle: HANDLE = core::ptr::null_mut();
    let mut ObjectName: _UNICODE_STRING = core::mem::zeroed();
    let mut LocalAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    if !lpName.is_null() {
        RtlInitUnicodeString(&raw mut ObjectName, lpName as PCWSTR);
    }
    let ObjectAttributes = BaseFormatObjectAttributes(
        &raw mut LocalAttributes,
        lpMutexAttributes as ntdll_sys::nt::LPSECURITY_ATTRIBUTES,
        if !lpName.is_null() { &raw mut ObjectName } else { core::ptr::null_mut() },
    );
    let Status = NtCreateMutant(
        &raw mut Handle,
        dwDesiredAccess as ACCESS_MASK,
        ObjectAttributes,
        (dwFlags & 0x1 != 0) as BOOLEAN,
    );
    finish_create(Status, Handle)
}

#[no_mangle]
pub unsafe extern "system" fn CreateMutexExA(
    lpMutexAttributes: LPSECURITY_ATTRIBUTES,
    lpName: LPCSTR,
    dwFlags: DWORD,
    dwDesiredAccess: DWORD,
) -> HANDLE {
    if lpName.is_null() {
        return CreateMutexExW(lpMutexAttributes, core::ptr::null(), dwFlags, dwDesiredAccess);
    }
    match ansi_name(lpName) {
        Some(w) => CreateMutexExW(lpMutexAttributes, w, dwFlags, dwDesiredAccess),
        None => core::ptr::null_mut(),
    }
}

#[no_mangle]
pub unsafe extern "system" fn CreateWaitableTimerExW(
    lpTimerAttributes: LPSECURITY_ATTRIBUTES,
    lpTimerName: LPCWSTR,
    dwFlags: DWORD,
    dwDesiredAccess: DWORD,
) -> HANDLE {
    let mut Handle: HANDLE = core::ptr::null_mut();
    let mut ObjectName: _UNICODE_STRING = core::mem::zeroed();
    let mut LocalAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    if !lpTimerName.is_null() {
        RtlInitUnicodeString(&raw mut ObjectName, lpTimerName as PCWSTR);
    }
    let ObjectAttributes = BaseFormatObjectAttributes(
        &raw mut LocalAttributes,
        lpTimerAttributes as ntdll_sys::nt::LPSECURITY_ATTRIBUTES,
        if !lpTimerName.is_null() { &raw mut ObjectName } else { core::ptr::null_mut() },
    );
    let Status = NtCreateTimer(
        &raw mut Handle,
        dwDesiredAccess as ACCESS_MASK,
        ObjectAttributes,
        if dwFlags & 0x1 != 0 { NotificationTimer } else { SynchronizationTimer } as TIMER_TYPE,
    );
    finish_create(Status, Handle)
}

#[no_mangle]
pub unsafe extern "system" fn CreateWaitableTimerExA(
    lpTimerAttributes: LPSECURITY_ATTRIBUTES,
    lpTimerName: LPCSTR,
    dwFlags: DWORD,
    dwDesiredAccess: DWORD,
) -> HANDLE {
    if lpTimerName.is_null() {
        return CreateWaitableTimerExW(lpTimerAttributes, core::ptr::null(), dwFlags, dwDesiredAccess);
    }
    match ansi_name(lpTimerName) {
        Some(w) => CreateWaitableTimerExW(lpTimerAttributes, w, dwFlags, dwDesiredAccess),
        None => core::ptr::null_mut(),
    }
}

/* ------------------------------------------------------------------------- *
 * Per-thread error mode (TEB->HardErrorMode)
 * ------------------------------------------------------------------------- */

const SEM_VALID_THREAD_MODES: u32 = 0x0001 | 0x0002 | 0x8000;

// The TEB keeps the RTL_SEM_* encoding (rtl-error's RtlSetThreadErrorMode), not the
// SEM_* values callers pass: SEM_FAILCRITICALERRORS 0x1 <-> 0x10,
// SEM_NOGPFAULTERRORBOX 0x2 <-> 0x20, SEM_NOOPENFILEERRORBOX 0x8000 <-> 0x40 (as Wine's
// kernelbase maps them). Storing the SEM_* value raw would put 0x8000 in the TEB.
const SEM_RTL_MAP: [(u32, u32); 3] = [(0x0001, 0x10), (0x0002, 0x20), (0x8000, 0x40)];

extern "system" {
    fn RtlGetThreadErrorMode() -> ULONG;
    fn RtlSetThreadErrorMode(NewMode: ULONG, OldMode: *mut ULONG) -> NTSTATUS;
}

fn rtl_to_sem(rtl: u32) -> u32 {
    SEM_RTL_MAP.iter().filter(|(_, r)| rtl & r != 0).fold(0, |m, (s, _)| m | s)
}

#[no_mangle]
pub unsafe extern "system" fn GetThreadErrorMode() -> DWORD {
    rtl_to_sem(RtlGetThreadErrorMode())
}

#[no_mangle]
pub unsafe extern "system" fn SetThreadErrorMode(dwNewMode: DWORD, lpOldMode: *mut DWORD) -> BOOL {
    if dwNewMode & !SEM_VALID_THREAD_MODES != 0 {
        SetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }
    let rtl = SEM_RTL_MAP.iter().filter(|(s, _)| dwNewMode & s != 0).fold(0, |m, (_, r)| m | r);
    let mut old: ULONG = 0;
    let status = RtlSetThreadErrorMode(rtl, &mut old);
    if status < 0 {
        BaseSetLastNTError(status);
        return 0;
    }
    if !lpOldMode.is_null() {
        *lpOldMode = rtl_to_sem(old);
    }
    1
}

/* ------------------------------------------------------------------------- *
 * RaiseFailFastException
 * ------------------------------------------------------------------------- */

#[no_mangle]
pub unsafe extern "system" fn RaiseFailFastException(
    pExceptionRecord: *mut EXCEPTION_RECORD,
    pContext: *mut c_void,
    dwFlags: DWORD,
) {
    let _ = (pContext, dwFlags);
    let code: u32 = if pExceptionRecord.is_null() {
        0xC000_0602
    } else {
        (*pExceptionRecord).ExceptionCode as u32
    };
    DbgPrint(
        b"RaiseFailFastException: fail-fast, exception code 0x%08lx\n\0".as_ptr() as PCSTR,
        code,
    );
    NtTerminateProcess(NtCurrentProcess(), code as NTSTATUS);
    /* NtTerminateProcess on the current process does not return; never fall out. */
    loop {
        core::hint::spin_loop();
    }
}

/* ------------------------------------------------------------------------- *
 * GetLogicalProcessorInformationEx
 *
 * ntdll-sys has no NtQuerySystemInformationEx and the kernel has no
 * SystemLogicalProcessorInformationEx class, so the answer is synthesized in user
 * mode from the legacy SystemLogicalProcessorInformation records (single group 0).
 * ------------------------------------------------------------------------- */

struct Emitter {
    buf: *mut u8,
    cap: usize,
    pos: usize,
}

impl Emitter {
    /// Append one record `size` bytes long with its header filled in; returns the
    /// (zeroed) record if it fits, else None (the size is still counted).
    unsafe fn begin(&mut self, rel: u32, size: usize) -> Option<*mut SYSTEM_LOGICAL_PROCESSOR_INFORMATION_EX> {
        let at = self.pos;
        self.pos += size;
        if self.buf.is_null() || at + size > self.cap {
            return None;
        }
        let p = self.buf.add(at);
        core::ptr::write_bytes(p, 0, size);
        let r = p as *mut SYSTEM_LOGICAL_PROCESSOR_INFORMATION_EX;
        core::ptr::addr_of_mut!((*r).Relationship).write_unaligned(rel as _);
        core::ptr::addr_of_mut!((*r).Size).write_unaligned(size as ULONG);
        Some(r)
    }
}

const HDR: usize = core::mem::size_of::<ULONG>() * 2;

fn ga(mask: usize) -> GROUP_AFFINITY {
    GROUP_AFFINITY { Mask: mask as _, Group: 0, Reserved: [0; 3] }
}

#[no_mangle]
pub unsafe extern "system" fn GetLogicalProcessorInformationEx(
    RelationshipType: u32,
    Buffer: *mut c_void,
    ReturnedLength: *mut DWORD,
) -> BOOL {
    if ReturnedLength.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }
    const ALL: u32 = 0xffff;
    if RelationshipType > 4 && RelationshipType != ALL {
        SetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }

    /* Fetch the legacy records, growing the scratch buffer as needed. */
    let heap = RtlGetProcessHeap();
    let mut cap: usize = 4096;
    let mut legacy: *mut u8;
    let legacy_len: ULONG;
    loop {
        legacy = RtlAllocateHeap(heap, 0, cap as SIZE_T) as *mut u8;
        if legacy.is_null() {
            SetLastError(ERROR_NOT_ENOUGH_MEMORY as DWORD);
            return 0;
        }
        let mut need: ULONG = 0;
        let st = NtQuerySystemInformation(
            SystemLogicalProcessorInformation,
            legacy as *mut c_void,
            cap as ULONG,
            &mut need,
        );
        if NT_SUCCESS(st) {
            legacy_len = need;
            break;
        }
        RtlFreeHeap(heap, 0, legacy as PVOID);
        if st == STATUS_INFO_LENGTH_MISMATCH || st == STATUS_BUFFER_TOO_SMALL {
            cap = if (need as usize) > cap { need as usize } else { cap * 2 };
            if cap > 0x100000 {
                SetLastError(ERROR_INVALID_PARAMETER);
                return 0;
            }
            continue;
        }
        BaseSetLastNTError(st);
        return 0;
    }

    let n = legacy_len as usize / core::mem::size_of::<SYSTEM_LOGICAL_PROCESSOR_INFORMATION>();
    let recs = legacy as *const SYSTEM_LOGICAL_PROCESSOR_INFORMATION;

    /* Union of all core masks; fall back to any mask, then CPU 0. */
    let mut all_mask: usize = 0;
    let mut have_pkg = false;
    let mut have_numa = false;
    for i in 0..n {
        let r = &*recs.add(i);
        if r.Relationship == 0 {
            all_mask |= r.ProcessorMask as usize;
        }
        if r.Relationship == 3 {
            have_pkg = true;
        }
        if r.Relationship == 1 {
            have_numa = true;
        }
    }
    if all_mask == 0 {
        for i in 0..n {
            all_mask |= (*recs.add(i)).ProcessorMask as usize;
        }
    }
    if all_mask == 0 {
        all_mask = 1;
    }

    let mut em = Emitter { buf: Buffer as *mut u8, cap: *ReturnedLength as usize, pos: 0 };
    let want = |rel: u32| RelationshipType == ALL || RelationshipType == rel;

    let proc_size = HDR + core::mem::size_of::<PROCESSOR_RELATIONSHIP>();
    let numa_size = HDR + core::mem::size_of::<NUMA_NODE_RELATIONSHIP>();
    let cache_size = HDR + core::mem::size_of::<CACHE_RELATIONSHIP>();
    let group_size = HDR + core::mem::size_of::<GROUP_RELATIONSHIP>();

    macro_rules! emit_proc {
        ($rel:expr, $flags:expr, $mask:expr) => {
            if let Some(r) = em.begin($rel, proc_size) {
                let p = &raw mut (*r).__bindgen_anon_1.Processor;
                core::ptr::addr_of_mut!((*p).Flags).write_unaligned($flags);
                core::ptr::addr_of_mut!((*p).GroupCount).write_unaligned(1);
                core::ptr::addr_of_mut!((*p).GroupMask).write_unaligned([ga($mask)]);
            }
        };
    }

    if want(0) {
        for i in 0..n {
            let r = &*recs.add(i);
            if r.Relationship == 0 {
                emit_proc!(0, r.__bindgen_anon_1.ProcessorCore.Flags, r.ProcessorMask as usize);
            }
        }
    }
    if want(1) {
        let mut any = false;
        for i in 0..n {
            let r = &*recs.add(i);
            if r.Relationship == 1 {
                any = true;
                if let Some(o) = em.begin(1, numa_size) {
                    let p = &raw mut (*o).__bindgen_anon_1.NumaNode;
                    core::ptr::addr_of_mut!((*p).NodeNumber)
                        .write_unaligned(r.__bindgen_anon_1.NumaNode.NodeNumber);
                    core::ptr::addr_of_mut!((*p).GroupMask).write_unaligned(ga(r.ProcessorMask as usize));
                }
            }
        }
        if !any && !have_numa {
            if let Some(o) = em.begin(1, numa_size) {
                let p = &raw mut (*o).__bindgen_anon_1.NumaNode;
                core::ptr::addr_of_mut!((*p).GroupMask).write_unaligned(ga(all_mask));
            }
        }
    }
    if want(2) {
        for i in 0..n {
            let r = &*recs.add(i);
            if r.Relationship == 2 {
                if let Some(o) = em.begin(2, cache_size) {
                    let c = r.__bindgen_anon_1.Cache;
                    let p = &raw mut (*o).__bindgen_anon_1.Cache;
                    core::ptr::addr_of_mut!((*p).Level).write_unaligned(c.Level);
                    core::ptr::addr_of_mut!((*p).Associativity).write_unaligned(c.Associativity);
                    core::ptr::addr_of_mut!((*p).LineSize).write_unaligned(c.LineSize);
                    core::ptr::addr_of_mut!((*p).CacheSize).write_unaligned(c.Size);
                    core::ptr::addr_of_mut!((*p).Type).write_unaligned(c.Type);
                    core::ptr::addr_of_mut!((*p).GroupMask).write_unaligned(ga(r.ProcessorMask as usize));
                }
            }
        }
    }
    if want(3) {
        let mut any = false;
        for i in 0..n {
            let r = &*recs.add(i);
            if r.Relationship == 3 {
                any = true;
                emit_proc!(3, 0, r.ProcessorMask as usize);
            }
        }
        if !any && !have_pkg {
            emit_proc!(3, 0, all_mask);
        }
    }
    if want(4) {
        if let Some(o) = em.begin(4, group_size) {
            let g = &raw mut (*o).__bindgen_anon_1.Group;
            let cnt = all_mask.count_ones() as u8;
            core::ptr::addr_of_mut!((*g).MaximumGroupCount).write_unaligned(1);
            core::ptr::addr_of_mut!((*g).ActiveGroupCount).write_unaligned(1);
            let mut gi: PROCESSOR_GROUP_INFO = core::mem::zeroed();
            gi.MaximumProcessorCount = cnt;
            gi.ActiveProcessorCount = cnt;
            gi.ActiveProcessorMask = all_mask as _;
            core::ptr::addr_of_mut!((*g).GroupInfo).write_unaligned([gi]);
        }
    }

    RtlFreeHeap(heap, 0, legacy as PVOID);

    let total = em.pos;
    let provided = *ReturnedLength as usize;
    *ReturnedLength = total as DWORD;
    if Buffer.is_null() || total > provided {
        SetLastError(ERROR_INSUFFICIENT_BUFFER);
        return 0;
    }
    1
}
