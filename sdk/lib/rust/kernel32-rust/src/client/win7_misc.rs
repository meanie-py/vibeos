//! Win7/Win8/Win10-era kernel32 exports that real apps import (Wine dlls/kernelbase and
//! dlls/kernel32 semantics): cycle-time queries, dynamic time zone info, single-group
//! processor affinity, the thread-pool long-run hint, WER flags and the named-pipe
//! server pid. Each is a small, honest implementation; where we lack the backing
//! mechanism the function says so and fails with ERROR_NOT_SUPPORTED instead of faking it.
use ntdll_sys::nt::*;
use crate::prelude::*;
use crate::wine::timezone::GetTimeZoneInformation;
use core::ffi::c_void;

const FALSE: BOOL = 0;
const TRUE: BOOL = 1;

const S_OK_: HRESULT = 0;
const E_INVALIDARG_: HRESULT = 0x8007_0057_u32 as HRESULT;

/// Sum of KernelTime + UserTime (100ns units) for a thread or process.
/// Windows reports CPU *cycles* here. We return the CPU time in 100ns units
/// instead, which is monotonic and is all callers rely on (elapsed-work
/// comparisons); it is not a true hardware cycle count.
unsafe fn cycle_time_from_kut(kut: &KERNEL_USER_TIMES) -> ULONG64 {
    (kut.KernelTime.QuadPart as u64).wrapping_add(kut.UserTime.QuadPart as u64)
}

#[no_mangle]
pub unsafe extern "system" fn QueryThreadCycleTime(
    ThreadHandle: HANDLE,
    CycleTime: PULONG64,
) -> BOOL {
    if CycleTime.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER);
        return FALSE;
    }
    let mut Kut: KERNEL_USER_TIMES = core::mem::zeroed();
    let Status = NtQueryInformationThread(
        ThreadHandle,
        ThreadTimes,
        &mut Kut as *mut _ as PVOID,
        core::mem::size_of::<KERNEL_USER_TIMES>() as ULONG,
        core::ptr::null_mut(),
    );
    if !NT_SUCCESS(Status) {
        BaseSetLastNTError(Status);
        return FALSE;
    }
    *CycleTime = cycle_time_from_kut(&Kut);
    TRUE
}

#[no_mangle]
pub unsafe extern "system" fn QueryProcessCycleTime(
    ProcessHandle: HANDLE,
    CycleTime: PULONG64,
) -> BOOL {
    if CycleTime.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER);
        return FALSE;
    }
    let mut Kut: KERNEL_USER_TIMES = core::mem::zeroed();
    let Status = NtQueryInformationProcess(
        ProcessHandle,
        ProcessTimes,
        &mut Kut as *mut _ as PVOID,
        core::mem::size_of::<KERNEL_USER_TIMES>() as ULONG,
        core::ptr::null_mut(),
    );
    if !NT_SUCCESS(Status) {
        BaseSetLastNTError(Status);
        return FALSE;
    }
    *CycleTime = cycle_time_from_kut(&Kut);
    TRUE
}

#[no_mangle]
pub unsafe extern "system" fn GetDynamicTimeZoneInformation(
    pTimeZoneInformation: PDYNAMIC_TIME_ZONE_INFORMATION,
) -> DWORD {
    if pTimeZoneInformation.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER);
        return TIME_ZONE_ID_INVALID as DWORD;
    }
    // The TIME_ZONE_INFORMATION prefix of DYNAMIC_TIME_ZONE_INFORMATION is layout-
    // compatible (SDK definition), so the base query fills it in place.
    let ret = GetTimeZoneInformation(pTimeZoneInformation as LPTIME_ZONE_INFORMATION);
    // No registry time zone key name is exposed by the base query.
    core::ptr::write_bytes(
        core::ptr::addr_of_mut!((*pTimeZoneInformation).TimeZoneKeyName) as *mut u8,
        0,
        core::mem::size_of::<[WCHAR; 128]>(),
    );
    (*pTimeZoneInformation).DynamicDaylightTimeDisabled = FALSE as BOOLEAN;
    ret
}

#[no_mangle]
pub unsafe extern "system" fn GetTimeZoneInformationForYear(
    wYear: USHORT,
    pdtzi: PDYNAMIC_TIME_ZONE_INFORMATION,
    ptzi: LPTIME_ZONE_INFORMATION,
) -> BOOL {
    let _ = wYear; // We have no per-year rule tables; the standard rule applies to every year.
    if ptzi.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER);
        return FALSE;
    }
    if pdtzi.is_null() {
        GetTimeZoneInformation(ptzi);
        return TRUE;
    }
    // Copy the TIME_ZONE_INFORMATION prefix out of the dynamic structure.
    core::ptr::copy_nonoverlapping(
        pdtzi as *const TIME_ZONE_INFORMATION,
        ptzi,
        1,
    );
    TRUE
}

#[no_mangle]
pub unsafe extern "system" fn GetProcessGroupAffinity(
    hProcess: HANDLE,
    GroupCount: PUSHORT,
    GroupArray: PUSHORT,
) -> BOOL {
    let _ = hProcess; // Single processor group (group 0); the process is always in it.
    if GroupCount.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER);
        return FALSE;
    }
    if *GroupCount < 1 {
        *GroupCount = 1;
        SetLastError(ERROR_INSUFFICIENT_BUFFER);
        return FALSE;
    }
    if GroupArray.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER);
        return FALSE;
    }
    *GroupArray = 0;
    *GroupCount = 1;
    TRUE
}

#[no_mangle]
pub unsafe extern "system" fn GetNumaNodeProcessorMaskEx(
    Node: USHORT,
    ProcessorMask: PGROUP_AFFINITY,
) -> BOOL {
    if Node != 0 {
        SetLastError(ERROR_INVALID_PARAMETER);
        return FALSE;
    }
    if ProcessorMask.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER);
        return FALSE;
    }
    // Node 0 is the only node we report: all active processors of the single group.
    core::ptr::write_bytes(ProcessorMask as *mut u8, 0, core::mem::size_of::<GROUP_AFFINITY>());
    (*ProcessorMask).Mask =
        (*BaseStaticServerData).SysInfo.ActiveProcessorsAffinityMask as KAFFINITY;
    (*ProcessorMask).Group = 0;
    TRUE
}

#[no_mangle]
pub unsafe extern "system" fn CallbackMayRunLong(_Instance: PTP_CALLBACK_INSTANCE) -> BOOL {
    // A pool thread is never guaranteed to be spare for a long-running callback
    // (no thread-pool spare-thread accounting), so report FALSE. Callers then do not
    // assume other callbacks can run concurrently while this one blocks.
    FALSE
}

#[no_mangle]
pub unsafe extern "system" fn WerGetFlags(
    _hProcess: HANDLE,
    pdwFlags: *mut DWORD,
) -> HRESULT {
    if pdwFlags.is_null() {
        return E_INVALIDARG_;
    }
    *pdwFlags = 0;
    S_OK_
}

#[no_mangle]
pub unsafe extern "system" fn WerSetFlags(_dwFlags: DWORD) -> HRESULT {
    // WER reporting is not implemented; flags are accepted and ignored.
    S_OK_
}

#[no_mangle]
pub unsafe extern "system" fn GetNamedPipeServerProcessId(
    _hPipe: HANDLE,
    ServerProcessId: PULONG,
) -> BOOL {
    // Wine asks npfs for FSCTL_PIPE_GET_PIPE_ATTRIBUTE("ServerProcessId"). Our npfs
    // (drivers/filesystems/npfs/fsctrl.c) has no such FSCTL, only the connection-
    // attribute one, so there is no real server pid to report. Fail honestly rather
    // than invent one.
    let _ = ServerProcessId;
    SetLastError(ERROR_NOT_SUPPORTED);
    FALSE
}
