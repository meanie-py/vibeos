//! Shared types + constants for the rtl exception/unwind tail.
//!
//! bindgen drops `#define`s, so the exception-flag / UNW_FLAG / STATUS values
//! are restated here — every value harvested from the in-tree headers
//! (sdk/include/xdk/winnt_old.h, sdk/include/ndk/{amd64,i386}/ketypes.h,
//! sdk/include/ndk/extypes.h, sdk/include/psdk/ntstatus.h). Layout-only locals
//! (SCOPE_TABLE, the i386 + arm64 DISPATCHER_CONTEXT) mirror rtlp.h /
//! winnt_old.h because bindgen emits them opaque (or not at all) there.

use ntkrnl_sys::ddk::*;

/* ===== ExceptionFlags (winnt_old.h) ===== */
pub const EXCEPTION_NONCONTINUABLE: ULONG = 0x01;
pub const EXCEPTION_UNWINDING: ULONG = 0x02;
pub const EXCEPTION_EXIT_UNWIND: ULONG = 0x04;
pub const EXCEPTION_STACK_INVALID: ULONG = 0x08;
pub const EXCEPTION_NESTED_CALL: ULONG = 0x10;
pub const EXCEPTION_TARGET_UNWIND: ULONG = 0x20;
pub const EXCEPTION_COLLIDED_UNWIND: ULONG = 0x40;
pub const EXCEPTION_UNWIND: ULONG = EXCEPTION_UNWINDING
    | EXCEPTION_EXIT_UNWIND
    | EXCEPTION_TARGET_UNWIND
    | EXCEPTION_COLLIDED_UNWIND; // 0x66

#[inline]
pub fn IS_UNWINDING(flags: ULONG) -> bool {
    (flags & EXCEPTION_UNWIND) != 0
}

/* ===== Unwind handler type flags (winnt_old.h) ===== */
pub const UNW_FLAG_NHANDLER: ULONG = 0x0;
pub const UNW_FLAG_EHANDLER: ULONG = 0x1;
pub const UNW_FLAG_UHANDLER: ULONG = 0x2;
pub const UNW_FLAG_CHAININFO: ULONG = 0x4;

/* ===== Filter results (excpt.h) ===== */
pub const EXCEPTION_EXECUTE_HANDLER: LONG = 1;
pub const EXCEPTION_CONTINUE_SEARCH: LONG = 0;
pub const EXCEPTION_CONTINUE_EXECUTION: LONG = -1;

/* ===== NTSTATUS values (ntstatus.h) ===== */
pub const STATUS_NOT_IMPLEMENTED: NTSTATUS = 0xC0000002u32 as NTSTATUS;
pub const STATUS_NONCONTINUABLE_EXCEPTION: NTSTATUS = 0xC0000025u32 as NTSTATUS;
pub const STATUS_INVALID_DISPOSITION: NTSTATUS = 0xC0000026u32 as NTSTATUS;
pub const STATUS_UNWIND: NTSTATUS = 0xC0000027u32 as NTSTATUS;
pub const STATUS_BAD_STACK: NTSTATUS = 0xC0000028u32 as NTSTATUS;
pub const STATUS_INVALID_UNWIND_TARGET: NTSTATUS = 0xC0000029u32 as NTSTATUS;
pub const STATUS_BAD_FUNCTION_TABLE: NTSTATUS = 0xC00000FFu32 as NTSTATUS;
pub const STATUS_LONGJUMP: NTSTATUS = 0x80000026u32 as NTSTATUS;
pub const STATUS_UNWIND_CONSOLIDATE: NTSTATUS = 0x80000029u32 as NTSTATUS;

/* ===== PE ===== */
pub const IMAGE_DIRECTORY_ENTRY_EXCEPTION: USHORT = 3;

/* ===== EXCEPTION_DISPOSITION values (bindgen enum-const aliases) ===== */
pub type EXCEPTION_DISPOSITION = _EXCEPTION_DISPOSITION;
pub const ExceptionContinueExecution: EXCEPTION_DISPOSITION = 0;
pub const ExceptionContinueSearch: EXCEPTION_DISPOSITION = 1;
pub const ExceptionNestedException: EXCEPTION_DISPOSITION = 2;
pub const ExceptionCollidedUnwind: EXCEPTION_DISPOSITION = 3;

/* ===== MODE (extypes.h) ===== */
pub const KernelMode: KPROCESSOR_MODE = 0;
pub const UserMode: KPROCESSOR_MODE = 1;

/* ===== SCOPE_TABLE (winnt_old.h, amd64/arm64 __C_specific_handler data) ===== */
#[repr(C)]
#[derive(Copy, Clone)]
pub struct SCOPE_RECORD {
    pub BeginAddress: ULONG,
    pub EndAddress: ULONG,
    pub HandlerAddress: ULONG,
    pub JumpTarget: ULONG,
}

#[repr(C)]
pub struct SCOPE_TABLE {
    pub Count: ULONG,
    pub ScopeRecord: [SCOPE_RECORD; 1], // flexible array
}
pub type PSCOPE_TABLE = *mut SCOPE_TABLE;

/* ===== i386 DISPATCHER_CONTEXT (rtlp.h) ===== */
#[cfg(target_arch = "x86")]
#[repr(C)]
pub struct DISPATCHER_CONTEXT {
    pub RegistrationPointer: PEXCEPTION_REGISTRATION_RECORD,
}

/* ===== arm64 DISPATCHER_CONTEXT (winnt_old.h ARM64 branch; bindgen emits it
 * opaque, so it is restated here field-for-field) ===== */
#[cfg(target_arch = "aarch64")]
#[repr(C)]
pub struct DISPATCHER_CONTEXT {
    pub ControlPc: ULONG_PTR,
    pub ImageBase: ULONG_PTR,
    pub FunctionEntry: PRUNTIME_FUNCTION,
    pub EstablisherFrame: ULONG_PTR,
    pub TargetPc: ULONG_PTR,
    pub ContextRecord: PCONTEXT,
    pub LanguageHandler: PEXCEPTION_ROUTINE,
    pub HandlerData: PVOID,
    pub HistoryTable: *mut _UNWIND_HISTORY_TABLE,
    pub ScopeIndex: ULONG,
    pub ControlPcIsUnwound: BOOLEAN,
    pub NonVolatileRegisters: *mut UCHAR,
}

/* amd64 uses the bindgen _DISPATCHER_CONTEXT directly. */
#[cfg(target_arch = "x86_64")]
pub type DISPATCHER_CONTEXT = _DISPATCHER_CONTEXT;

pub type PDISPATCHER_CONTEXT = *mut DISPATCHER_CONTEXT;

/* ===== EXCEPTION_CHAIN_END (i386 SEH list terminator, ndk) ===== */
#[cfg(target_arch = "x86")]
pub const EXCEPTION_CHAIN_END: PEXCEPTION_REGISTRATION_RECORD =
    usize::MAX as PEXCEPTION_REGISTRATION_RECORD; // ((PEXCEPTION_REGISTRATION_RECORD)-1)

/* ===== CONTEXT flag values (winnt_old.h) ===== */
#[cfg(target_arch = "x86_64")]
pub const CONTEXT_AMD64: ULONG = 0x0010_0000;
#[cfg(target_arch = "x86_64")]
pub const CONTEXT_CONTROL: ULONG = CONTEXT_AMD64 | 0x1;
#[cfg(target_arch = "x86_64")]
pub const CONTEXT_INTEGER: ULONG = CONTEXT_AMD64 | 0x2;
#[cfg(target_arch = "x86_64")]
pub const CONTEXT_SEGMENTS: ULONG = CONTEXT_AMD64 | 0x4;
#[cfg(target_arch = "x86_64")]
pub const CONTEXT_FLOATING_POINT: ULONG = CONTEXT_AMD64 | 0x8;

#[cfg(target_arch = "x86")]
pub const CONTEXT_i386: ULONG = 0x0001_0000;
#[cfg(target_arch = "x86")]
pub const CONTEXT_CONTROL: ULONG = CONTEXT_i386 | 0x1;
#[cfg(target_arch = "x86")]
pub const CONTEXT_INTEGER: ULONG = CONTEXT_i386 | 0x2;
#[cfg(target_arch = "x86")]
pub const CONTEXT_SEGMENTS: ULONG = CONTEXT_i386 | 0x4;

#[cfg(target_arch = "aarch64")]
pub const CONTEXT_ARM64: ULONG = 0x0040_0000;
#[cfg(target_arch = "aarch64")]
pub const CONTEXT_CONTROL: ULONG = CONTEXT_ARM64 | 0x1;
#[cfg(target_arch = "aarch64")]
pub const CONTEXT_INTEGER: ULONG = CONTEXT_ARM64 | 0x2;

/* ===== amd64 GDT/EFLAGS/MXCSR (ndk/amd64/ketypes.h) ===== */
#[cfg(target_arch = "x86_64")]
pub mod ke64 {
    pub const KGDT64_R0_CODE: u16 = 0x0010;
    pub const KGDT64_R0_DATA: u16 = 0x0018;
    pub const KGDT64_R3_DATA: u16 = 0x0028;
    pub const KGDT64_R3_CODE: u16 = 0x0030;
    pub const KGDT64_R3_CMTEB: u16 = 0x0050;
    pub const RPL_MASK: u16 = 0x0003;
    pub const EFLAGS_INTERRUPT_MASK: u32 = 0x200;
    pub const INITIAL_MXCSR: u32 = 0x1f80;
}

/* ===== i386 GDT/EFLAGS (ndk/i386/ketypes.h) ===== */
#[cfg(target_arch = "x86")]
pub mod ke32 {
    pub const KGDT_R3_CODE: u32 = 0x18;
    pub const KGDT_R3_DATA: u32 = 0x20;
    pub const KGDT_R3_TEB: u32 = 0x38;
    pub const EFLAGS_INTERRUPT_MASK: u32 = 0x200;
}

/* ===== Per-consumer libsupp helpers (ntoskrnl/rtl/libsupp.c or
 * dll/ntdll/rtl/libsupp.c provide these — external by design) ===== */
extern "system" {
    pub fn RtlpGetStackLimits(LowLimit: *mut ULONG_PTR, HighLimit: *mut ULONG_PTR);
    pub fn RtlpGetMode() -> KPROCESSOR_MODE;
    pub fn RtlpCheckLogException(
        ExceptionRecord: PEXCEPTION_RECORD,
        ContextRecord: PCONTEXT,
        ContextData: PVOID,
        Size: ULONG,
    );
    pub fn RtlpAllocateMemory(Bytes: SIZE_T, Tag: ULONG) -> PVOID;
    pub fn RtlpFreeMemory(Mem: PVOID, Tag: ULONG);
    /// rtl-diag (exception.rs / arch asm)
    pub fn RtlRaiseException(ExceptionRecord: PEXCEPTION_RECORD);
    /// DECLSPEC_NORETURN — raises and never returns.
    pub fn RtlRaiseStatus(Status: NTSTATUS) -> !;
    /// rtl-diag image.rs (translated image.c)
    pub fn RtlPcToFileHeader(PcValue: PVOID, BaseOfImage: *mut PVOID) -> PVOID;
}

/* C runtime */
extern "C" {
    pub fn wcslen(s: *const u16) -> usize;
}

/// `__debugbreak()`
#[inline(always)]
pub unsafe fn debugbreak() {
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    core::arch::asm!("int3");
    #[cfg(target_arch = "aarch64")]
    core::arch::asm!("brk #0xf000");
}
