use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::interlocked::*;
use ntkrnl_sys::ob::*;

use ntkrnl_sys::ddk::{
    FAST_MUTEX, LARGE_INTEGER, ULARGE_INTEGER, LUID, UUID,
    PACCESS_TOKEN, SECURITY_SUBJECT_CONTEXT,
    PKEY_VALUE_PARTIAL_INFORMATION,
    _OBJECT_ATTRIBUTES, _UNICODE_STRING, HANDLE,
    ExInitializeFastMutex, ExAcquireFastMutex, ExReleaseFastMutex,
    KeQueryPerformanceCounter,
    ExGetPreviousMode,
    ZwOpenKey, ZwQueryValueKey, ZwSetValueKey, ZwClose,
    SeCaptureSubjectContext, SeReleaseSubjectContext,
    SeQueryAuthenticationIdToken,
    RtlInitUnicodeString, RtlRaiseStatus, PLARGE_INTEGER,
};
use ntkrnl_sys::{opaque_ptr, opaque_ptr_const};

const STATUS_RETRY: NTSTATUS = 0xC000022Du32 as NTSTATUS;

extern "system" {
    fn RtlEqualLuid(L1: *const LUID, L2: *const LUID) -> BOOLEAN;
}
extern "C" {
    fn InterlockedExchangeAdd64(Addend: *mut i64, Value: i64) -> i64;
    fn SeQuerySubjectContextToken(SubjectContext: *mut SECURITY_SUBJECT_CONTEXT) -> PACCESS_TOKEN;
}
extern "system" {
    fn KeQuerySystemTime(CurrentTime: PLARGE_INTEGER);
}

const SEED_BUFFER_SIZE: usize = 6;
const TICKS_PER_CLOCK_TICK: i64 = 1000;
const SECSPERDAY: i64 = 86400;
const TICKSPERSEC: i64 = 10000000;
const SECS_15_OCT_1582_TO_1601: i64 = (17 + 30 + 31 + 365 * 18 + 5) * SECSPERDAY;
const TICKS_15_OCT_1582_TO_1601: u64 = (SECS_15_OCT_1582_TO_1601 as u64) * (TICKSPERSEC as u64);
const TIME_FRAME: i64 = 10000;
const VALUE_BUFFER_SIZE: usize = 20;

#[repr(C)]
struct UUID_CACHED_VALUES_STRUCT {
    Time: u64,
    AllocatedCount: LONG,
    GuidInit: [u8; 8],
}

impl UUID_CACHED_VALUES_STRUCT {
    const fn new() -> Self {
        Self {
            Time: 0,
            AllocatedCount: -1i32,
            GuidInit: [0, 0, 0x80, 0x6E, 0x6F, 0x6E, 0x69, 0x63],
        }
    }

    // GuidInit[] is the union's byte view of { ClockSeqHiAndReserved, ClockSeqLow,
    // NodeId[6] } — see UUID_CACHED_VALUES_STRUCT in ntoskrnl/include/internal/ex.h.
    // HiAndReserved is the FIRST byte: the union exists so GuidInit[0] lands on
    // GUID.Data4[0], which RFC4122 defines as the variant-stamped byte. Transposing
    // these two indices puts the raw sequence low byte in Data4[0] and breaks the
    // (Data4[0] & 0xC0) == 0x80 variant invariant.
    fn ClockSeqHiAndReserved(&self) -> u8 {
        self.GuidInit[0]
    }

    fn set_ClockSeqHiAndReserved(&mut self, value: u8) {
        self.GuidInit[0] = value;
    }

    fn ClockSeqLow(&self) -> u8 {
        self.GuidInit[1]
    }

    fn set_ClockSeqLow(&mut self, value: u8) {
        self.GuidInit[1] = value;
    }

    fn NodeId(&self) -> &[u8; 6] {
        unsafe { &*(self.GuidInit[2..8].as_ptr() as *const [u8; 6]) }
    }

    fn NodeId_mut(&mut self) -> &mut [u8; 6] {
        unsafe { &mut *(self.GuidInit[2..8].as_mut_ptr() as *mut [u8; 6]) }
    }
}

static mut ExpUuidLock: FAST_MUTEX = unsafe { core::mem::zeroed() };
static mut ExpUuidLastTimeAllocated: LARGE_INTEGER = unsafe { core::mem::zeroed() };
static mut ExpUuidSequenceNumber: ULONG = 0;
static mut ExpUuidSequenceNumberValid: BOOLEAN = 0;
static mut ExpUuidSequenceNumberNotSaved: BOOLEAN = 0;
static mut ExpUuidCachedValues: UUID_CACHED_VALUES_STRUCT = UUID_CACHED_VALUES_STRUCT::new();
static mut ExpUuidCacheValid: BOOLEAN = 0;
static mut ExpLuidIncrement: ULONG = 1;
static mut ExpLuid: LARGE_INTEGER = unsafe { core::mem::transmute(0x3e9_u64) };

// CODE_SEG("INIT")
#[no_mangle]
pub unsafe extern "system" fn ExpUuidInitialization() -> BOOLEAN {
    ExInitializeFastMutex(core::ptr::addr_of_mut!(ExpUuidLock));
    ExpUuidSequenceNumberValid = 0;
    KeQuerySystemTime(core::ptr::addr_of_mut!(ExpUuidLastTimeAllocated));
    1
}

unsafe fn ExpUuidLoadSequenceNumber(Sequence: *mut ULONG) -> NTSTATUS {
    let mut ValueBuffer: [u8; VALUE_BUFFER_SIZE] = [0; VALUE_BUFFER_SIZE];
    let mut KeyName: _UNICODE_STRING = core::mem::zeroed();
    let mut ValueName: _UNICODE_STRING = core::mem::zeroed();
    let mut KeyHandle: HANDLE = core::ptr::null_mut();
    let mut ValueLength: ULONG = 0;

    static KEY_PATH: [u16; 41] = [
        92, 82, 101, 103, 105, 115, 116, 114, 121, 92, 77, 97, 99, 104, 105, 110, 101, 92,
        83, 111, 102, 116, 119, 97, 114, 101, 92, 77, 105, 99, 114, 111, 115, 111, 102, 116,
        92, 82, 112, 99, 0,
    ];
    static VALUE_NAME: [u16; 19] = [
        85, 117, 105, 100, 83, 101, 113, 117, 101, 110, 99, 101, 78, 117, 109, 98, 101, 114, 0,
    ];

    RtlInitUnicodeString(
        core::ptr::addr_of_mut!(KeyName),
        KEY_PATH.as_ptr(),
    );
    RtlInitUnicodeString(
        core::ptr::addr_of_mut!(ValueName),
        VALUE_NAME.as_ptr(),
    );

    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
    ObjectAttributes.RootDirectory = core::ptr::null_mut();
    ObjectAttributes.ObjectName = core::ptr::addr_of_mut!(KeyName);
    ObjectAttributes.Attributes = OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE;
    ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
    ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

    let Status = ZwOpenKey(
        core::ptr::addr_of_mut!(KeyHandle),
        GENERIC_READ,
        core::ptr::addr_of_mut!(ObjectAttributes),
    );
    if Status < 0 {
        return Status;
    }

    let ValueInfo = ValueBuffer.as_mut_ptr() as PKEY_VALUE_PARTIAL_INFORMATION;
    let Status = ZwQueryValueKey(
        KeyHandle,
        core::ptr::addr_of_mut!(ValueName),
        ntkrnl_sys::ddk::_KEY_VALUE_INFORMATION_CLASS_KeyValuePartialInformation as _,
        ValueBuffer.as_mut_ptr() as PVOID,
        VALUE_BUFFER_SIZE as ULONG,
        core::ptr::addr_of_mut!(ValueLength),
    );
    ZwClose(KeyHandle);
    if Status < 0 {
        return Status;
    }

    if (*ValueInfo).Type != 4 || (*ValueInfo).DataLength != 4 {
        return STATUS_UNSUCCESSFUL;
    }

    *Sequence = *((*ValueInfo).Data.as_ptr() as *const ULONG);

    STATUS_SUCCESS
}

unsafe fn ExpUuidSaveSequenceNumber(Sequence: *const ULONG) -> NTSTATUS {
    let mut KeyName: _UNICODE_STRING = core::mem::zeroed();
    let mut ValueName: _UNICODE_STRING = core::mem::zeroed();
    let mut KeyHandle: HANDLE = core::ptr::null_mut();

    static KEY_PATH2: [u16; 41] = [
        92, 82, 101, 103, 105, 115, 116, 114, 121, 92, 77, 97, 99, 104, 105, 110, 101, 92,
        83, 111, 102, 116, 119, 97, 114, 101, 92, 77, 105, 99, 114, 111, 115, 111, 102, 116,
        92, 82, 112, 99, 0,
    ];
    static VALUE_NAME2: [u16; 19] = [
        85, 117, 105, 100, 83, 101, 113, 117, 101, 110, 99, 101, 78, 117, 109, 98, 101, 114, 0,
    ];

    RtlInitUnicodeString(
        core::ptr::addr_of_mut!(KeyName),
        KEY_PATH2.as_ptr(),
    );
    RtlInitUnicodeString(
        core::ptr::addr_of_mut!(ValueName),
        VALUE_NAME2.as_ptr(),
    );

    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
    ObjectAttributes.RootDirectory = core::ptr::null_mut();
    ObjectAttributes.ObjectName = core::ptr::addr_of_mut!(KeyName);
    ObjectAttributes.Attributes = OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE;
    ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
    ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

    let Status = ZwOpenKey(
        core::ptr::addr_of_mut!(KeyHandle),
        GENERIC_READ | GENERIC_WRITE,
        core::ptr::addr_of_mut!(ObjectAttributes),
    );
    if Status < 0 {
        return Status;
    }

    let Status = ZwSetValueKey(
        KeyHandle,
        core::ptr::addr_of_mut!(ValueName),
        0,
        4,
        Sequence as PVOID,
        core::mem::size_of::<ULONG>() as ULONG,
    );
    ZwClose(KeyHandle);
    if Status < 0 {
        return Status;
    }

    Status
}

unsafe fn ExpUuidSaveSequenceNumberIf() {
    if ExpUuidSequenceNumberNotSaved == 1 {
        let Status = ExpUuidSaveSequenceNumber(core::ptr::addr_of!(ExpUuidSequenceNumber));
        if Status >= 0 {
            ExpUuidSequenceNumberNotSaved = 0;
        }
    }
}

unsafe fn ExpAllocateUuids(
    Time: *mut ULARGE_INTEGER,
    Range: *mut ULONG,
    Sequence: *mut ULONG,
) -> NTSTATUS {
    let mut Counter: LARGE_INTEGER = core::mem::zeroed();
    let mut Frequency: LARGE_INTEGER = core::mem::zeroed();
    let mut CurrentTime: LARGE_INTEGER = core::mem::zeroed();
    let mut TimeDiff: LARGE_INTEGER = core::mem::zeroed();
    let mut ClockDiff: LARGE_INTEGER = core::mem::zeroed();

    if ExpUuidSequenceNumberValid == 0 {
        let Status = ExpUuidLoadSequenceNumber(core::ptr::addr_of_mut!(ExpUuidSequenceNumber));
        if Status >= 0 {
            ExpUuidSequenceNumber = ExpUuidSequenceNumber.wrapping_add(1);
        } else {
            Counter = KeQueryPerformanceCounter(core::ptr::addr_of_mut!(Frequency));
            ExpUuidSequenceNumber ^= (&Status as *const _ as usize) as ULONG;
            ExpUuidSequenceNumber ^= (Sequence as usize) as ULONG;
            ExpUuidSequenceNumber ^= Counter.u.LowPart as ULONG;
            ExpUuidSequenceNumber ^= Counter.u.HighPart as ULONG;
        }

        ExpUuidSequenceNumberValid = 1;
        ExpUuidSequenceNumberNotSaved = 1;
    }

    KeQuerySystemTime(core::ptr::addr_of_mut!(CurrentTime));
    TimeDiff.QuadPart = CurrentTime.QuadPart - ExpUuidLastTimeAllocated.QuadPart;

    if TimeDiff.QuadPart < 0 {
        ExpUuidSequenceNumber = ExpUuidSequenceNumber.wrapping_add(1);
        TimeDiff.QuadPart = 2 * TIME_FRAME;
        ExpUuidSequenceNumberNotSaved = 1;
        ExpUuidLastTimeAllocated.QuadPart = CurrentTime.QuadPart - 2 * TIME_FRAME;
    }

    if TimeDiff.QuadPart == 0 {
        return STATUS_RETRY;
    }

    if TimeDiff.u.HighPart > 0 || TimeDiff.QuadPart > TICKS_PER_CLOCK_TICK * TIME_FRAME {
        TimeDiff.QuadPart = TICKS_PER_CLOCK_TICK * TIME_FRAME;
    }

    if TimeDiff.u.HighPart < 0 || TimeDiff.QuadPart <= TIME_FRAME {
        *Range = TimeDiff.QuadPart as ULONG;
        ClockDiff.QuadPart = 0;
    } else {
        *Range = TIME_FRAME as ULONG;
        ClockDiff.QuadPart = TimeDiff.QuadPart - TIME_FRAME;
        ClockDiff.u.HighPart -= 1;
    }

    (*Time).QuadPart = (CurrentTime.QuadPart - (*Range as i64) - ClockDiff.QuadPart) as u64;
    ExpUuidLastTimeAllocated.QuadPart = CurrentTime.QuadPart - ClockDiff.QuadPart;
    *Sequence = ExpUuidSequenceNumber;

    STATUS_SUCCESS
}

unsafe fn ExpUuidGetValues(CachedValues: *mut UUID_CACHED_VALUES_STRUCT) -> NTSTATUS {
    let mut Time: ULARGE_INTEGER = core::mem::zeroed();
    let mut Range: ULONG = 0;
    let mut Sequence: ULONG = 0;

    let Status = ExpAllocateUuids(
        core::ptr::addr_of_mut!(Time),
        core::ptr::addr_of_mut!(Range),
        core::ptr::addr_of_mut!(Sequence),
    );
    if Status == STATUS_RETRY {
        return Status;
    }

    if Status < 0 {
        return STATUS_NO_MEMORY;
    }

    (*CachedValues).set_ClockSeqLow(Sequence as u8);
    (*CachedValues).set_ClockSeqHiAndReserved(((Sequence & 0x3F00) >> 8) as u8);
    let hi = (*CachedValues).ClockSeqHiAndReserved() | 0x80;
    (*CachedValues).set_ClockSeqHiAndReserved(hi);
    (*CachedValues).AllocatedCount = Range as LONG;

    Time.QuadPart = Time.QuadPart.wrapping_add(TICKS_15_OCT_1582_TO_1601);
    (*CachedValues).Time = Time.QuadPart.wrapping_add((Range as u64).wrapping_sub(1));

    STATUS_SUCCESS
}

// CODE_SEG("INIT")
#[no_mangle]
pub unsafe extern "system" fn ExLuidInitialization() -> BOOLEAN {
    1
}

#[no_mangle]
pub unsafe extern "system" fn ExAllocateLocallyUniqueId(LocallyUniqueId: *mut LUID) {
    let LocallyUniqueId = opaque_ptr(LocallyUniqueId);
    *(LocallyUniqueId as *mut i64) = InterlockedExchangeAdd64(
        core::ptr::addr_of_mut!(ExpLuid.QuadPart),
        ExpLuidIncrement as i64,
    );
}

#[no_mangle]
pub unsafe extern "system" fn NtAllocateLocallyUniqueId(LocallyUniqueId: *mut LUID) -> NTSTATUS {
    let LocallyUniqueId = opaque_ptr(LocallyUniqueId);
    let PreviousMode = ExGetPreviousMode();
    if PreviousMode != 0 {
        // C: _SEH2_TRY { ProbeForWrite } whose handler returns the exception code.
        if let Err(status) = ntkrnl_sys::probe::probe_user_buffer_write(
            LocallyUniqueId as PVOID,
            core::mem::size_of::<LUID>(),
            core::mem::align_of::<ULONG>(),
            1,
        ) {
            return status;
        }
    }

    // Divergence from the C, which hands the user pointer straight to
    // ExAllocateLocallyUniqueId and stores OUTSIDE its __try: an unmap after the
    // probe would fault there. Allocate into a local and copy it out fault-tolerantly.
    let mut luid: LUID = core::mem::zeroed();
    ExAllocateLocallyUniqueId(&mut luid);
    if let Err(status) = ntkrnl_sys::probe::write_back(LocallyUniqueId, luid) {
        return status;
    }
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn ExUuidCreate(Uuid: *mut UUID) -> NTSTATUS {
    let Uuid = opaque_ptr(Uuid);
    let mut Status: NTSTATUS = STATUS_SUCCESS;

    loop {
        let mut Time: LARGE_INTEGER;
        let mut Valid: BOOLEAN;
        let mut AllocatedCount: LONG;

        loop {
            Time.QuadPart = ExpUuidCachedValues.Time as i64;

            core::ptr::copy_nonoverlapping(
                (*(&raw const ExpUuidCachedValues)).GuidInit.as_ptr(),
                (*Uuid).Data4.as_mut_ptr(),
                8,
            );

            Valid = ExpUuidCacheValid;
            AllocatedCount = InterlockedDecrement(
                core::ptr::addr_of_mut!(ExpUuidCachedValues.AllocatedCount) as *mut LONG
            );

            if Time.QuadPart == ExpUuidCachedValues.Time as i64 {
                break;
            }
        }

        if AllocatedCount >= 0 {
            if Valid == 0 {
                Status = 0x40020056u32 as NTSTATUS;
            }

            Time.QuadPart -= AllocatedCount as i64;
            (*Uuid).Data1 = Time.u.LowPart;
            (*Uuid).Data2 = Time.u.HighPart as u16;
            (*Uuid).Data3 = ((Time.u.HighPart >> 16) & 0x0FFF) as u16 | 0x1000;

            return Status;
        }

        ExAcquireFastMutex(core::ptr::addr_of_mut!(ExpUuidLock));
        if Time.QuadPart == ExpUuidCachedValues.Time as i64 {
            Status = ExpUuidGetValues(core::ptr::addr_of_mut!(ExpUuidCachedValues));
            if Status != STATUS_SUCCESS {
                ExReleaseFastMutex(core::ptr::addr_of_mut!(ExpUuidLock));
                return Status;
            }

            ExpUuidSaveSequenceNumberIf();
        }
        ExReleaseFastMutex(core::ptr::addr_of_mut!(ExpUuidLock));
    }
}

#[no_mangle]
pub unsafe extern "system" fn NtAllocateUuids(
    Time: *mut ULARGE_INTEGER,
    Range: *mut ULONG,
    Sequence: *mut ULONG,
    Seed: *mut u8,
) -> NTSTATUS {
    let Time = opaque_ptr(Time);
    let Range = opaque_ptr(Range);
    let Sequence = opaque_ptr(Sequence);
    let Seed = opaque_ptr(Seed);
    let mut IntTime: ULARGE_INTEGER = core::mem::zeroed();
    let mut IntRange: ULONG = 0;
    let mut IntSequence: ULONG = 0;

    let PreviousMode = ExGetPreviousMode();
    if PreviousMode != 0 {
        // C: _SEH2_TRY { four ProbeForWrites } whose handler returns the code.
        let probes: [(PVOID, usize, usize); 4] = [
            (Time as PVOID, core::mem::size_of::<ULARGE_INTEGER>(), core::mem::align_of::<ULONG>()),
            (Range as PVOID, core::mem::size_of::<ULONG>(), core::mem::align_of::<ULONG>()),
            (Sequence as PVOID, core::mem::size_of::<ULONG>(), core::mem::align_of::<ULONG>()),
            (Seed as PVOID, SEED_BUFFER_SIZE, core::mem::align_of::<u8>()),
        ];
        for (address, length, alignment) in probes {
            if let Err(status) = ntkrnl_sys::probe::probe_user_buffer(address, length, alignment, 1) {
                return status;
            }
        }
    }

    ExAcquireFastMutex(core::ptr::addr_of_mut!(ExpUuidLock));

    let Status = ExpAllocateUuids(
        core::ptr::addr_of_mut!(IntTime),
        core::ptr::addr_of_mut!(IntRange),
        core::ptr::addr_of_mut!(IntSequence),
    );
    if Status < 0 {
        ExReleaseFastMutex(core::ptr::addr_of_mut!(ExpUuidLock));
        return Status;
    }

    ExpUuidSaveSequenceNumberIf();
    ExReleaseFastMutex(core::ptr::addr_of_mut!(ExpUuidLock));

    // C: the write-back is under _SEH2_TRY, whose handler returns the code. The
    // pointers were probed above (user mode) or are trusted (kernel mode); the
    // writes themselves are fault-tolerant.
    let write_back = || -> Result<(), NTSTATUS> {
        ntkrnl_sys::probe::write_user(core::ptr::addr_of_mut!((*Time).QuadPart), IntTime.QuadPart, 0)?;
        ntkrnl_sys::probe::write_user(Range, IntRange, 0)?;
        ntkrnl_sys::probe::write_user(Sequence, IntSequence, 0)?;
        ntkrnl_sys::probe::safe_copy_raw(
            Seed.cast(),
            (*(&raw const ExpUuidCachedValues)).NodeId().as_ptr().cast(),
            SEED_BUFFER_SIZE,
        )
    };
    match write_back() {
        Ok(()) => STATUS_SUCCESS,
        Err(status) => status,
    }
}

#[no_mangle]
pub unsafe extern "system" fn NtSetUuidSeed(Seed: *const u8) -> NTSTATUS {
    let mut GotContext: BOOLEAN = 0;
    let mut SubjectContext: SECURITY_SUBJECT_CONTEXT = core::mem::zeroed();
    let mut CallerLuid: LUID = core::mem::zeroed();
    let SystemLuid = LUID { LowPart: 0x3e7, HighPart: 0 };

    // C: one _SEH2_TRY whose handler stores the exception code in Status. Its
    // RtlRaiseStatus calls are how it reaches that handler; with no SEH they are
    // `Err` returns (raising would be an unhandled exception).
    let result = (|| -> Result<(), NTSTATUS> {
        SeCaptureSubjectContext(core::ptr::addr_of_mut!(SubjectContext));
        GotContext = 1;

        let Token = SeQuerySubjectContextToken(core::ptr::addr_of_mut!(SubjectContext));
        let Status = SeQueryAuthenticationIdToken(Token, core::ptr::addr_of_mut!(CallerLuid));
        if Status < 0 {
            return Err(Status);
        }

        if RtlEqualLuid(
            core::ptr::addr_of!(CallerLuid),
            core::ptr::addr_of!(SystemLuid),
        ) == 0
        {
            return Err(STATUS_ACCESS_DENIED);
        }

        let node_id = (*(&raw mut ExpUuidCachedValues)).NodeId_mut().as_mut_ptr();
        ntkrnl_sys::probe::safe_copy_from_user(
            node_id.cast(),
            Seed.cast(),
            SEED_BUFFER_SIZE,
            core::mem::align_of::<u8>(),
            1,
        )?;

        // The C re-reads *Seed here; the byte just captured is the same one.
        ExpUuidCacheValid = (!((*node_id >> 7) & 1)) & 1;

        Ok(())
    })();
    let Status = match result {
        Ok(()) => STATUS_SUCCESS,
        Err(status) => status,
    };

    if GotContext != 0 {
        SeReleaseSubjectContext(core::ptr::addr_of_mut!(SubjectContext));
    }

    Status
}

// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:       See COPYING in the top level directory
//  * PROJECT:         ReactOS kernel
//  * FILE:            ntoskrnl/ex/uuid.c
//  * PURPOSE:         UUID generator
//  * PROGRAMMERS:     Eric Kohl
//  *                  Thomas Weidenmueller
//  *                  Pierre Schweitzer
//  */
//
// /* INCLUDES *****************************************************************/
//
// #include <ntoskrnl.h>
// #define NDEBUG
// #include <debug.h>
//
// #define SEED_BUFFER_SIZE 6
//
// /* Number of 100ns ticks per clock tick. To be safe, assume that the clock
//    resolution is at least 1000 * 100 * (1/1000000) = 1/10 of a second */
// #define TICKS_PER_CLOCK_TICK 1000
// #define SECSPERDAY  86400
// #define TICKSPERSEC 10000000
//
// /* UUID system time starts at October 15, 1582 */
// #define SECS_15_OCT_1582_TO_1601  ((17 + 30 + 31 + 365 * 18 + 5) * SECSPERDAY)
// #define TICKS_15_OCT_1582_TO_1601 ((ULONGLONG)SECS_15_OCT_1582_TO_1601 * TICKSPERSEC)
//
// /* 10000 in 100-ns model = 0.1 microsecond */
// #define TIME_FRAME 10000
//
//
// /* GLOBALS ****************************************************************/
//
// FAST_MUTEX ExpUuidLock;
// LARGE_INTEGER ExpUuidLastTimeAllocated;
// ULONG ExpUuidSequenceNumber = 0;
// BOOLEAN ExpUuidSequenceNumberValid;
// BOOLEAN ExpUuidSequenceNumberNotSaved = FALSE;
// UUID_CACHED_VALUES_STRUCT ExpUuidCachedValues = {0ULL, 0xFFFFFFFF, {{0, 0, {0x80, 0x6E, 0x6F, 0x6E, 0x69, 0x63}}}};
// BOOLEAN ExpUuidCacheValid = FALSE;
// ULONG ExpLuidIncrement = 1;
// LARGE_INTEGER ExpLuid = {{0x3e9, 0x0}};
//
// /* FUNCTIONS ****************************************************************/
//
// /*
//  * @implemented
//  */
// CODE_SEG("INIT")
// BOOLEAN
// NTAPI
// ExpUuidInitialization(VOID)
// {
//     ExInitializeFastMutex(&ExpUuidLock);
//
//     ExpUuidSequenceNumberValid = FALSE;
//     KeQuerySystemTime(&ExpUuidLastTimeAllocated);
//
//     return TRUE;
// }
//
//
// /*
//  * @implemented
//  */
// #define VALUE_BUFFER_SIZE 20
// static NTSTATUS
// ExpUuidLoadSequenceNumber(PULONG Sequence)
// {
//     UCHAR ValueBuffer[VALUE_BUFFER_SIZE];
//     PKEY_VALUE_PARTIAL_INFORMATION ValueInfo;
//     _OBJECT_ATTRIBUTES ObjectAttributes;
//     _UNICODE_STRING KeyName, ValueName;
//     HANDLE KeyHandle;
//     ULONG ValueLength;
//     NTSTATUS Status;
//
//     PAGED_CODE();
//
//     RtlInitUnicodeString(&KeyName, L"\\Registry\\Machine\\Software\\Microsoft\\Rpc");
//     RtlInitUnicodeString(&ValueName, L"UuidSequenceNumber");
//
//     InitializeObjectAttributes(&ObjectAttributes,
//                                &KeyName,
//                                OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE,
//                                NULL,
//                                NULL);
//     Status = ZwOpenKey(&KeyHandle, GENERIC_READ, &ObjectAttributes);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT("ZwOpenKey() failed (Status %lx)\n", Status);
//         return Status;
//     }
//
//     ValueInfo = (PKEY_VALUE_PARTIAL_INFORMATION)ValueBuffer;
//     Status = ZwQueryValueKey(KeyHandle,
//                              &ValueName,
//                              KeyValuePartialInformation,
//                              ValueBuffer,
//                              VALUE_BUFFER_SIZE,
//                              &ValueLength);
//     ZwClose(KeyHandle);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT("ZwQueryValueKey() failed (Status %lx)\n", Status);
//         return Status;
//     }
//
//     if (ValueInfo->Type != REG_DWORD || ValueInfo->DataLength != sizeof(DWORD))
//     {
//         return STATUS_UNSUCCESSFUL;
//     }
//
//     *Sequence = *((PULONG)ValueInfo->Data);
//
//     DPRINT("Loaded sequence %lx\n", *Sequence);
//
//     return STATUS_SUCCESS;
// }
// #undef VALUE_BUFFER_SIZE
//
// /*
//  * @implemented
//  */
// static NTSTATUS
// ExpUuidSaveSequenceNumber(PULONG Sequence)
// {
//     _OBJECT_ATTRIBUTES ObjectAttributes;
//     _UNICODE_STRING KeyName, ValueName;
//     HANDLE KeyHandle;
//     NTSTATUS Status;
//
//     PAGED_CODE();
//
//     RtlInitUnicodeString(&KeyName, L"\\Registry\\Machine\\Software\\Microsoft\\Rpc");
//     RtlInitUnicodeString(&ValueName, L"UuidSequenceNumber");
//
//     InitializeObjectAttributes(&ObjectAttributes,
//                                &KeyName,
//                                OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE,
//                                NULL,
//                                NULL);
//     Status = ZwOpenKey(&KeyHandle,
//                        GENERIC_READ | GENERIC_WRITE,
//                        &ObjectAttributes);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT("ZwOpenKey() failed (Status %lx)\n", Status);
//         return Status;
//     }
//
//     Status = ZwSetValueKey(KeyHandle,
//                            &ValueName,
//                            0,
//                            REG_DWORD,
//                            Sequence,
//                            sizeof(ULONG));
//     ZwClose(KeyHandle);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT("ZwSetValueKey() failed (Status %lx)\n", Status);
//     }
//
//     return Status;
// }
//
// /*
//  * @implemented
//  * Warning! This function must be called
//  * with ExpUuidLock held!
//  */
// static VOID
// ExpUuidSaveSequenceNumberIf(VOID)
// {
//     NTSTATUS Status;
//
//     PAGED_CODE();
//
//     /* Only save sequence if it has to */
//     if (ExpUuidSequenceNumberNotSaved == TRUE)
//     {
//         Status = ExpUuidSaveSequenceNumber(&ExpUuidSequenceNumber);
//         if (NT_SUCCESS(Status))
//         {
//             ExpUuidSequenceNumberNotSaved = FALSE;
//         }
//     }
// }
//
// /*
//  * @implemented
//  * Warning! This function must be called
//  * with ExpUuidLock held!
//  */
// static NTSTATUS
// ExpAllocateUuids(PULARGE_INTEGER Time,
//                  PULONG Range,
//                  PULONG Sequence)
// {
//     NTSTATUS Status;
//     LARGE_INTEGER Counter, Frequency, CurrentTime, TimeDiff, ClockDiff;
//
//     PAGED_CODE();
//
//     /* Initialize sequence number */
//     if (!ExpUuidSequenceNumberValid)
//     {
//         /* Try to load sequence number */
//         Status = ExpUuidLoadSequenceNumber(&ExpUuidSequenceNumber);
//         if (NT_SUCCESS(Status))
//         {
//             ++ExpUuidSequenceNumber;
//         }
//         else
//         {
//             /* If we cannot, generate a "true" random */
//             Counter = KeQueryPerformanceCounter(&Frequency);
//             ExpUuidSequenceNumber ^= (ULONG_PTR)&Status ^ (ULONG_PTR)Sequence ^ Counter.LowPart ^ Counter.HighPart;
//         }
//
//         /* It's valid and to be saved */
//         ExpUuidSequenceNumberValid = TRUE;
//         ExpUuidSequenceNumberNotSaved = TRUE;
//     }
//
//     KeQuerySystemTime(&CurrentTime);
//     TimeDiff.QuadPart = CurrentTime.QuadPart - ExpUuidLastTimeAllocated.QuadPart;
//     /* If time went backwards, change sequence (see RFC example) */
//     if (TimeDiff.QuadPart < 0)
//     {
//         ++ExpUuidSequenceNumber;
//         TimeDiff.QuadPart = 2 * TIME_FRAME;
//
//         /* It's to be saved */
//         ExpUuidSequenceNumberNotSaved = TRUE;
//         ExpUuidLastTimeAllocated.QuadPart = CurrentTime.QuadPart - 2 * TIME_FRAME;
//     }
//
//     if (TimeDiff.QuadPart == 0)
//     {
//         return STATUS_RETRY;
//     }
//
//     /* If time diff > 0.1ms, squash it to reduce it to keep our clock resolution */
//     if (TimeDiff.HighPart > 0 || TimeDiff.QuadPart > TICKS_PER_CLOCK_TICK * TIME_FRAME)
//     {
//         TimeDiff.QuadPart = TICKS_PER_CLOCK_TICK * TIME_FRAME;
//     }
//
//     if (TimeDiff.HighPart < 0 || TimeDiff.QuadPart <= TIME_FRAME)
//     {
//         *Range = TimeDiff.QuadPart;
//         ClockDiff.QuadPart = 0LL;
//     }
//     else
//     {
//         *Range = TIME_FRAME;
//         ClockDiff.QuadPart = TimeDiff.QuadPart - TIME_FRAME;
//         --ClockDiff.HighPart;
//     }
//
//     Time->QuadPart = CurrentTime.QuadPart - *Range - ClockDiff.QuadPart;
//     ExpUuidLastTimeAllocated.QuadPart = CurrentTime.QuadPart - ClockDiff.QuadPart;
//     *Sequence = ExpUuidSequenceNumber;
//
//     return STATUS_SUCCESS;
// }
//
// /*
//  * @implemented
//  * Warning! This function must be called
//  * with ExpUuidLock held!
//  */
// static NTSTATUS
// ExpUuidGetValues(PUUID_CACHED_VALUES_STRUCT CachedValues)
// {
//     NTSTATUS Status;
//     ULARGE_INTEGER Time;
//     ULONG Range;
//     ULONG Sequence;
//
//     PAGED_CODE();
//
//     /* Allocate UUIDs */
//     Status = ExpAllocateUuids(&Time, &Range, &Sequence);
//     if (Status == STATUS_RETRY)
//     {
//         return Status;
//     }
//
//     if (!NT_SUCCESS(Status))
//     {
//         return STATUS_NO_MEMORY;
//     }
//
//     /* We need at least one UUID */
//     ASSERT(Range != 0);
//
//     /* Set up our internal cache
//      * See format_uuid_v1 in RFC4122 for magic values
//      */
//     CachedValues->ClockSeqLow = Sequence;
//     CachedValues->ClockSeqHiAndReserved = (Sequence & 0x3F00) >> 8;
//     CachedValues->ClockSeqHiAndReserved |= 0x80;
//     CachedValues->AllocatedCount = Range;
//
//     /*
//      * Time is relative to UUID time
//      * And we set last time range for all the possibly
//      * returnable UUID
//      */
//     Time.QuadPart += TICKS_15_OCT_1582_TO_1601;
//     CachedValues->Time = Time.QuadPart + (Range - 1);
//
//     return STATUS_SUCCESS;
// }
//
// /*
//  * @implemented
//  */
// CODE_SEG("INIT")
// BOOLEAN
// NTAPI
// ExLuidInitialization(VOID)
// {
//     return TRUE;
// }
//
// /*
//  * @implemented
//  */
// VOID
// NTAPI
// ExAllocateLocallyUniqueId(OUT LUID *LocallyUniqueId)
// {
//     /* Atomically increment the luid */
//     *(LONG64*)LocallyUniqueId = InterlockedExchangeAdd64(&ExpLuid.QuadPart,
//                                                          ExpLuidIncrement);
// }
//
//
// /*
//  * @implemented
//  */
// NTSTATUS
// NTAPI
// NtAllocateLocallyUniqueId(OUT LUID *LocallyUniqueId)
// {
//     KPROCESSOR_MODE PreviousMode;
//     PAGED_CODE();
//
//     /* Probe if user mode */
//     PreviousMode = ExGetPreviousMode();
//     if (PreviousMode != KernelMode)
//     {
//         _SEH2_TRY
//         {
//             ProbeForWrite(LocallyUniqueId,
//                           sizeof(LUID),
//                           sizeof(ULONG));
//         }
//         _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//         {
//             _SEH2_YIELD(return _SEH2_GetExceptionCode());
//         }
//         _SEH2_END;
//     }
//
//     /* Do the allocation */
//     ExAllocateLocallyUniqueId(LocallyUniqueId);
//     return STATUS_SUCCESS;
// }
//
// /*
//  * @implemented
//  */
// NTSTATUS
// NTAPI
// ExUuidCreate(OUT UUID *Uuid)
// {
//     NTSTATUS Status;
//     LONG AllocatedCount;
//     LARGE_INTEGER Time;
//     BOOLEAN Valid;
//
//     PAGED_CODE();
//
//     Status = STATUS_SUCCESS;
//     /* Loop until we have an UUID to return */
//     while (TRUE)
//     {
//         /* Try to gather node values */
//         do
//         {
//             Time.QuadPart = ExpUuidCachedValues.Time;
//
//             RtlCopyMemory(Uuid->Data4,
//                           ExpUuidCachedValues.GuidInit,
//                           sizeof(Uuid->Data4));
//
//             Valid = ExpUuidCacheValid;
//             AllocatedCount = InterlockedDecrement(&ExpUuidCachedValues.AllocatedCount);
//         }
//         /* Loop till we can do it without being disturbed */
//         while (Time.QuadPart != ExpUuidCachedValues.Time);
//
//         /* We have more than an allocated UUID left, that's OK to return! */
//         if (AllocatedCount >= 0)
//         {
//             break;
//         }
//
//         /*
//          * Here, we're out of UUIDs, we need to allocate more
//          * We need to be alone to do it, so lock the mutex
//          */
//         ExAcquireFastMutex(&ExpUuidLock);
//         if (Time.QuadPart == ExpUuidCachedValues.Time)
//         {
//             /* If allocation fails, bail out! */
//             Status = ExpUuidGetValues(&ExpUuidCachedValues);
//             if (Status != STATUS_SUCCESS)
//             {
//                 ExReleaseFastMutex(&ExpUuidLock);
//                 return Status;
//             }
//
//             /* Save our current sequence if changed */
//             ExpUuidSaveSequenceNumberIf();
//         }
//         ExReleaseFastMutex(&ExpUuidLock);
//     }
//
//     /*
//      * Once here, we've got an UUID to return
//      * But, if our init wasn't sane, then, make
//      * sure it's only used locally
//      */
//     if (!Valid)
//     {
//         Status = RPC_NT_UUID_LOCAL_ONLY;
//     }
//
//     /* Set our timestamp - see RFC4211 */
//     Time.QuadPart -= AllocatedCount;
//     Uuid->Data1 = Time.LowPart;
//     Uuid->Data2 = Time.HighPart;
//     /* We also set the bit for GUIDv1 */
//     Uuid->Data3 = ((Time.HighPart >> 16) & 0x0FFF) | 0x1000;
//
//     return Status;
// }
//
// /*
//  * @implemented
//  */
// NTSTATUS
// NTAPI
// NtAllocateUuids(OUT PULARGE_INTEGER Time,
//                 OUT PULONG Range,
//                 OUT PULONG Sequence,
//                 OUT PUCHAR Seed)
// {
//     ULARGE_INTEGER IntTime;
//     ULONG IntRange, IntSequence;
//     NTSTATUS Status;
//     KPROCESSOR_MODE PreviousMode;
//
//     PAGED_CODE();
//
//     /* Probe if user mode */
//     PreviousMode = ExGetPreviousMode();
//     if (PreviousMode != KernelMode)
//     {
//         _SEH2_TRY
//         {
//             ProbeForWrite(Time,
//                           sizeof(ULARGE_INTEGER),
//                           sizeof(ULONG));
//
//             ProbeForWrite(Range,
//                           sizeof(ULONG),
//                           sizeof(ULONG));
//
//             ProbeForWrite(Sequence,
//                           sizeof(ULONG),
//                           sizeof(ULONG));
//
//             ProbeForWrite(Seed,
//                           SEED_BUFFER_SIZE,
//                           sizeof(UCHAR));
//         }
//         _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//         {
//             _SEH2_YIELD(return _SEH2_GetExceptionCode());
//         }
//         _SEH2_END;
//     }
//
//     /* During allocation we must be alone */
//     ExAcquireFastMutex(&ExpUuidLock);
//
//     Status = ExpAllocateUuids(&IntTime,
//                               &IntRange,
//                               &IntSequence);
//     if (!NT_SUCCESS(Status))
//     {
//         ExReleaseFastMutex(&ExpUuidLock);
//         return Status;
//     }
//
//     /* If sequence number was changed, save it */
//     ExpUuidSaveSequenceNumberIf();
//
//     /* Allocation done, so we can release */
//     ExReleaseFastMutex(&ExpUuidLock);
//
//     /* Write back UUIDs to caller */
//     _SEH2_TRY
//     {
//         Time->QuadPart = IntTime.QuadPart;
//         *Range = IntRange;
//         *Sequence = IntSequence;
//
//         RtlCopyMemory(Seed,
//                       &ExpUuidCachedValues.NodeId[0],
//                       SEED_BUFFER_SIZE);
//
//         Status = STATUS_SUCCESS;
//     }
//     _SEH2_EXCEPT(ExSystemExceptionFilter())
//     {
//         Status = _SEH2_GetExceptionCode();
//     }
//     _SEH2_END;
//
//     return Status;
// }
//
//
// /*
//  * @implemented
//  */
// NTSTATUS
// NTAPI
// NtSetUuidSeed(IN PUCHAR Seed)
// {
//     NTSTATUS Status;
//     BOOLEAN GotContext;
//     PACCESS_TOKEN Token;
//     SECURITY_SUBJECT_CONTEXT SubjectContext;
//     LUID CallerLuid, SystemLuid = SYSTEM_LUID;
//
//     PAGED_CODE();
//
//     /* Should only be done by umode */
//     ASSERT(KeGetPreviousMode() != KernelMode);
//
//     /* No context to release */
//     GotContext = FALSE;
//     _SEH2_TRY
//     {
//         /* Get our caller context and remember to release it */
//         SeCaptureSubjectContext(&SubjectContext);
//         GotContext = TRUE;
//
//         /* Get caller access token and its associated ID */
//         Token = SeQuerySubjectContextToken(&SubjectContext);
//         Status = SeQueryAuthenticationIdToken(Token, &CallerLuid);
//         if (!NT_SUCCESS(Status))
//         {
//             RtlRaiseStatus(Status);
//         }
//
//         /* This call is only allowed for SYSTEM */
//         if (!RtlEqualLuid(&CallerLuid, &SystemLuid))
//         {
//             RtlRaiseStatus(STATUS_ACCESS_DENIED);
//         }
//
//         /* Check for buffer validity and then copy it to our seed */
//         ProbeForRead(Seed, SEED_BUFFER_SIZE, sizeof(UCHAR));
//         RtlCopyMemory(&ExpUuidCachedValues.NodeId[0], Seed, SEED_BUFFER_SIZE);
//
//         /*
//          * According to RFC 4122, UUID seed is based on MAC addresses
//          * If it is randomly set, then, it must have its multicast be set
//          * to be valid and avoid collisions
//          * Reflect it here
//          */
//         ExpUuidCacheValid = ~(*Seed >> 7) & 1;
//
//         Status = STATUS_SUCCESS;
//     }
//     _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//     {
//         Status = _SEH2_GetExceptionCode();
//     }
//     _SEH2_END;
//
//     /* Release context if required */
//     if (GotContext)
//     {
//         SeReleaseSubjectContext(&SubjectContext);
//     }
//
//     return Status;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
