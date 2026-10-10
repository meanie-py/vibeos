//! pciidex-rust prelude — the single import surface the translated modules use.
//! Bridges the bindgen `types` island and the translated C. Anything bindgen leaves out
//! (CRT, inlines, sibling-C externs, hand-transcribed consts) is declared here ONCE.
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]

pub use crate::types::*;
pub use core::ffi::{c_char, c_int, c_long, c_short, c_uchar, c_uint, c_ushort, c_void};
// C's `size_t`: bindgen spells it `usize` inline, so no island carries the name the C uses.
pub type size_t = usize;
pub use core::ptr::{addr_of, addr_of_mut};

// ================= debug ======================================================================
// reactos/debug.h DPRINT/UNIMPLEMENTED — no-op MACROS, never out-of-line fns (field manual
// §9.65: an out-of-line no-op is an optimization barrier). debug_macros.py rewrites C's bare
// `UNIMPLEMENTED;` to `UNIMPLEMENTED!()`, so the prelude must have it. DPRINT1 PRINTS in this
// DBG=1 tree: it is win32macros-rust's.
#[macro_export] macro_rules! DPRINT  { ($($arg:tt)*) => {{}}; }
#[macro_export] macro_rules! UNIMPLEMENTED { ($($arg:tt)*) => {{}}; }
#[macro_export] macro_rules! UNIMPLEMENTED_ONCE { ($($arg:tt)*) => {{}}; }
pub use crate::{DPRINT, UNIMPLEMENTED, UNIMPLEMENTED_ONCE};
pub use win32macros_rust::DPRINT1;
// The module's own debug.h channels (ERR/WARN/INFO/TRACE, printf-shaped). Macros, never
// out-of-line fns, for the same reason as DPRINT above. debug.h defines DEBUG_WARN and
// DEBUG_ERR under DBG, so in this DBG=1 tree ERR and WARN print, as DPRINT1's "(file:line) "
// format; INFO and TRACE are compiled out upstream too.
#[macro_export] macro_rules! ERR { ($($arg:tt)*) => { ::win32macros_rust::DPRINT1!($($arg)*) }; }
#[macro_export] macro_rules! WARN { ($($arg:tt)*) => { ::win32macros_rust::DPRINT1!($($arg)*) }; }
#[macro_export] macro_rules! INFO { ($($arg:tt)*) => {{}}; }
#[macro_export] macro_rules! TRACE { ($($arg:tt)*) => {{}}; }
pub use crate::{ERR, INFO, TRACE, WARN};

// ================= kernel header helpers ======================================================
// wdm.h/ntddk.h MACROS and MSVC intrinsics the C calls as functions. ntkrnl-sys has portable
// wrappers for them outside the `ddk` glob (ke, inline::tier1, inline::ke, interlocked), so
// they are re-exported here BY NAME -- the C's `_`-prefixed intrinsic spellings included.
// ntkrnl_sys::ke's spin-lock helpers take ITS `PKSPIN_LOCK` (`*mut usize`); a driver's lock
// fields are the island's (`*mut c_ulonglong` on 64-bit) -- the same width, a distinct Rust
// type, so a bare re-export made every call site E0308 (16 in atapi). Thin wrappers over the
// island's type instead.
#[inline]
pub unsafe fn KeAcquireSpinLock(SpinLock: PKSPIN_LOCK, OldIrql: *mut KIRQL) {
    ntkrnl_sys::ke::KeAcquireSpinLock(SpinLock.cast(), OldIrql)
}
#[inline]
pub unsafe fn KeReleaseSpinLock(SpinLock: PKSPIN_LOCK, NewIrql: KIRQL) {
    ntkrnl_sys::ke::KeReleaseSpinLock(SpinLock.cast(), NewIrql)
}
#[inline]
pub unsafe fn KeAcquireSpinLockAtDpcLevel(SpinLock: PKSPIN_LOCK) {
    ntkrnl_sys::ke::KeAcquireSpinLockAtDpcLevel(SpinLock.cast())
}
#[inline]
pub unsafe fn KeReleaseSpinLockFromDpcLevel(SpinLock: PKSPIN_LOCK) {
    ntkrnl_sys::ke::KeReleaseSpinLockFromDpcLevel(SpinLock.cast())
}
#[inline]
pub unsafe fn KeInitializeSpinLock(SpinLock: PKSPIN_LOCK) {
    ntkrnl_sys::inline::ke::KeInitializeSpinLock(SpinLock.cast())
}
pub use ntkrnl_sys::inline::tier1::{
    IoCopyCurrentIrpStackLocationToNext, IoGetCurrentIrpStackLocation, IoGetNextIrpStackLocation,
    IoMarkIrpPending, IoSetCancelRoutine, IoSetCompletionRoutine, IoSetNextIrpStackLocation,
    IoSkipCurrentIrpStackLocation,
};
// KeRaiseIrql/KeLowerIrql are wdm.h MACROS over KfRaiseIrql/KfLowerIrql: x86 has wrappers in
// inline::tier1; amd64 writes CR8 inline (inline::mm); arm64 calls the HAL exports.
#[cfg(target_arch = "x86")]
pub use ntkrnl_sys::inline::tier1::{KeLowerIrql, KeRaiseIrql};
#[cfg(target_arch = "x86_64")]
#[inline]
pub unsafe fn KeRaiseIrql(NewIrql: KIRQL, OldIrql: *mut KIRQL) {
    *OldIrql = ntkrnl_sys::inline::mm::KfRaiseIrql(NewIrql);
}
#[cfg(target_arch = "x86_64")]
#[inline]
pub unsafe fn KeLowerIrql(NewIrql: KIRQL) {
    ntkrnl_sys::inline::mm::KfLowerIrql(NewIrql);
}
#[cfg(target_arch = "aarch64")]
extern "C" {
    fn KfRaiseIrql(NewIrql: KIRQL) -> KIRQL;
    fn KfLowerIrql(NewIrql: KIRQL);
}
#[cfg(target_arch = "aarch64")]
#[inline]
pub unsafe fn KeRaiseIrql(NewIrql: KIRQL, OldIrql: *mut KIRQL) {
    *OldIrql = KfRaiseIrql(NewIrql);
}
#[cfg(target_arch = "aarch64")]
#[inline]
pub unsafe fn KeLowerIrql(NewIrql: KIRQL) {
    KfLowerIrql(NewIrql);
}
// ntbasedef.h: `__assume(e)` is an optimiser promise; `RTL_BITS_OF(t)` is sizeof(t) * 8.
#[inline(always)]
pub unsafe fn __assume(cond: bool) {
    core::hint::assert_unchecked(cond)
}
#[macro_export] macro_rules! RTL_BITS_OF { ($t:ty $(,)?) => { (core::mem::size_of::<$t>() * 8) }; }
pub use crate::RTL_BITS_OF;
pub use ntkrnl_sys::inline::common::InterlockedCompareExchange;
pub use ntkrnl_sys::inline::common::InterlockedCompareExchange as _InterlockedCompareExchange;
pub use ntkrnl_sys::inline::ke::{BitScanForward, BitScanReverse};
pub use ntkrnl_sys::inline::ke::{BitScanForward as _BitScanForward, BitScanReverse as _BitScanReverse};
pub use ntkrnl_sys::interlocked::{
    InterlockedAnd, InterlockedCompareExchangePointer, InterlockedDecrement, InterlockedExchangeAdd,
    InterlockedExchangePointer, InterlockedIncrement, InterlockedOr, InterlockedXor,
};
pub use ntkrnl_sys::interlocked::{
    InterlockedAnd as _InterlockedAnd, InterlockedDecrement as _InterlockedDecrement,
    InterlockedExchangeAdd as _InterlockedExchangeAdd, InterlockedIncrement as _InterlockedIncrement,
    InterlockedOr as _InterlockedOr, InterlockedXor as _InterlockedXor,
};

/// wdm.h: `IoAcquireRemoveLockEx(RemoveLock, Tag, __FILE__, __LINE__, sizeof(IO_REMOVE_LOCK))`.
#[inline]
pub unsafe fn IoAcquireRemoveLock(RemoveLock: PIO_REMOVE_LOCK, Tag: PVOID) -> NTSTATUS {
    IoAcquireRemoveLockEx(
        RemoveLock,
        Tag,
        concat!(file!(), "\0").as_ptr() as PCSTR,
        line!(),
        core::mem::size_of::<IO_REMOVE_LOCK>() as ULONG,
    )
}
/// wdm.h: `IoReleaseRemoveLockEx(RemoveLock, Tag, sizeof(IO_REMOVE_LOCK))`.
#[inline]
pub unsafe fn IoReleaseRemoveLock(RemoveLock: PIO_REMOVE_LOCK, Tag: PVOID) {
    IoReleaseRemoveLockEx(RemoveLock, Tag, core::mem::size_of::<IO_REMOVE_LOCK>() as ULONG)
}
/// wdm.h: `IoReleaseRemoveLockAndWaitEx(RemoveLock, Tag, sizeof(IO_REMOVE_LOCK))`.
#[inline]
pub unsafe fn IoReleaseRemoveLockAndWait(RemoveLock: PIO_REMOVE_LOCK, Tag: PVOID) {
    IoReleaseRemoveLockAndWaitEx(RemoveLock, Tag, core::mem::size_of::<IO_REMOVE_LOCK>() as ULONG)
}

// PAGED_CODE() asserts IRQL < DISPATCH_LEVEL in checked builds and is empty otherwise.
#[macro_export] macro_rules! PAGED_CODE { () => {{}}; }
// NT_VERIFY(exp) is `((exp) ? TRUE : FALSE)` in a free build: the expression is evaluated.
#[macro_export] macro_rules! NT_VERIFY { ($e:expr $(,)?) => { ($e) }; }
// ntbasedef.h: UNREACHABLE is `__builtin_unreachable()`, so reaching it is UB in C too.
#[macro_export] macro_rules! UNREACHABLE {
    () => { core::hint::unreachable_unchecked() }; // nolint: assert-to-ub (__builtin_unreachable)
}
pub use crate::{NT_VERIFY, PAGED_CODE, UNREACHABLE};

// HAL port/register I/O (hal.dll exports on i386/arm64, inline on amd64).
pub use ntkrnl_sys::portio::*;
// C's TRUE/FALSE are ints; in kernel code they land in BOOLEAN (UCHAR) far more than BOOL.
pub const TRUE: BOOLEAN = 1;
pub const FALSE: BOOLEAN = 0;

// C's min()/max() macros (`((a) < (b) ? (a) : (b))`): both operands already share a type in
// a translation that compiles, and the C evaluates the winner twice, which no caller here
// relies on.
#[inline]
pub fn min<T: PartialOrd>(a: T, b: T) -> T {
    if a < b { a } else { b }
}
#[inline]
pub fn max<T: PartialOrd>(a: T, b: T) -> T {
    if a > b { a } else { b }
}

// ================= ata_user.h transfer-mode index macros (hand-ported) =====================
// Function-like SDK macros, so `port macros` (the module's own) does not reach them.
#[macro_export] macro_rules! PIO_MODE { ($n:expr $(,)?) => { ($n) }; }
#[macro_export] macro_rules! SWDMA_MODE { ($n:expr $(,)?) => { (5 + ($n)) }; }
#[macro_export] macro_rules! MWDMA_MODE { ($n:expr $(,)?) => { (8 + ($n)) }; }
#[macro_export] macro_rules! UDMA_MODE { ($n:expr $(,)?) => { (11 + ($n)) }; }
pub use crate::{MWDMA_MODE, PIO_MODE, SWDMA_MODE, UDMA_MODE};
// ata_shared.h `IDE_ACPI_TIMING_MODE_FLAG_UDMA(Drive) (0x01 << (2 * (Drive)))`: an `int` the C
// masks into the ULONG ModeFlags, so the literal takes the flags' type.
#[macro_export] macro_rules! IDE_ACPI_TIMING_MODE_FLAG_UDMA { ($drive:expr $(,)?) => { (0x01 << (2 * ($drive))) }; }
pub use crate::IDE_ACPI_TIMING_MODE_FLAG_UDMA;

// pata.h's ATA_READ/ATA_WRITE take a bare flag and PASTE it: `ChanInfo & CHANNEL_FLAG_##MmioFlag`.
// macro_rules! cannot paste identifiers on stable, so the ported macros take the flag as an
// expression and these names are the pasted results.
pub const MRES_TF: u32 = CHANNEL_FLAG_MRES_TF;
pub const MRES_DMA: u32 = CHANNEL_FLAG_MRES_DMA;
pub const MRES_CTRL: u32 = CHANNEL_FLAG_MRES_CTRL;

// ================= acpiioct.h method-argument macros (hand-ported) ==========================
// SDK header macros, so `port macros` (the module's own) does not reach them. `Data` lives in
// the anonymous union bindgen names `__bindgen_anon_1`; its offset is the union's.
/// `FIELD_OFFSET(ACPI_METHOD_ARGUMENT, Data) + max(sizeof(ULONG), DataLength)` (a `size_t`).
#[macro_export]
macro_rules! ACPI_METHOD_ARGUMENT_LENGTH {
    ($len:expr $(,)?) => {
        (core::mem::offset_of!(ACPI_METHOD_ARGUMENT, __bindgen_anon_1)
            + if core::mem::size_of::<ULONG>() > ($len) as usize {
                core::mem::size_of::<ULONG>()
            } else {
                ($len) as usize
            })
    };
}
/// `(PACPI_METHOD_ARGUMENT)((PUCHAR)Argument + ACPI_METHOD_ARGUMENT_LENGTH_FROM_ARGUMENT(Argument))`.
#[macro_export]
macro_rules! ACPI_METHOD_NEXT_ARGUMENT {
    ($arg:expr $(,)?) => {
        (($arg) as PUCHAR).add($crate::ACPI_METHOD_ARGUMENT_LENGTH!(
            (*(($arg) as PACPI_METHOD_ARGUMENT)).DataLength
        )) as PACPI_METHOD_ARGUMENT
    };
}
/// Type = BUFFER; DataLength = BuffLength; RtlCopyMemory(&Data[0], BuffData, DataLength).
#[macro_export]
macro_rules! ACPI_METHOD_SET_ARGUMENT_BUFFER {
    ($arg:expr, $data:expr, $len:expr $(,)?) => {{
        let __a: PACPI_METHOD_ARGUMENT = ($arg) as PACPI_METHOD_ARGUMENT;
        (*__a).Type = ACPI_METHOD_ARGUMENT_BUFFER as USHORT;
        (*__a).DataLength = ($len) as USHORT;
        memcpy(
            core::ptr::addr_of_mut!((*__a).__bindgen_anon_1) as *mut c_void,
            ($data) as *const c_void,
            (*__a).DataLength as usize,
        );
    }};
}
pub use crate::{ACPI_METHOD_ARGUMENT_LENGTH, ACPI_METHOD_NEXT_ARGUMENT, ACPI_METHOD_SET_ARGUMENT_BUFFER};


// ================= CRT ========================================================================
// NEVER hand-write memcpy/memset/memmove/memcmp in Rust: LLVM folds any byte-copy loop back into
// a call to the function itself → infinite self-recursion → silent hang. Import them (AGENTS.md).
extern "C" {
    pub fn memcpy(dest: *mut c_void, src: *const c_void, n: usize) -> *mut c_void;
    pub fn memset(dest: *mut c_void, c: c_int, n: usize) -> *mut c_void;
    pub fn memmove(dest: *mut c_void, src: *const c_void, n: usize) -> *mut c_void;
    pub fn memcmp(a: *const c_void, b: *const c_void, n: usize) -> c_int;
}

// bindgen does not emit NULL (a macro) nor NT_SUCCESS (an inline).
pub const NULL: *mut c_void = core::ptr::null_mut();
#[inline(always)]
pub fn NT_SUCCESS(s: NTSTATUS) -> bool {
    s >= 0
}
// Translations reach for the C MACRO forms too (`NT_SUCCESS!(st)`, `CONTAINING_RECORD!(p, T, f)`)
// and a fresh crate otherwise starts with `cannot find macro` in every module that walks a list.
// Address arithmetic, so neither needs an `unsafe` context of its own.
#[macro_export] macro_rules! NT_SUCCESS { ($s:expr) => { (($s) as i32) >= 0 }; }
#[macro_export] macro_rules! CONTAINING_RECORD {
    // An indexed member (`Tail.Overlay.DriverContext[0]`): offsetof the array plus the
    // element stride, as clang folds it -- offset_of! refuses an index.
    ($addr:expr, $ty:ty, $($field:ident).+ [ $idx:expr ] $(,)?) => {{
        const fn __elem<T, const N: usize>(_: *const [T; N]) -> usize { core::mem::size_of::<T>() }
        let slot = core::mem::MaybeUninit::<$ty>::uninit();
        let stride = __elem(unsafe { core::ptr::addr_of!((*slot.as_ptr()).$($field).+) });
        (($addr) as usize - (core::mem::offset_of!($ty, $($field).+) + ($idx) as usize * stride))
            as *mut $ty
    }};
    // C's field argument is a member PATH (`Tail.Overlay.ListEntry`), and the model
    // often leaves a trailing comma after a multi-line call.
    ($addr:expr, $ty:ty, $($field:ident).+ $(,)?) => {
        (($addr) as usize - core::mem::offset_of!($ty, $($field).+)) as *mut $ty
    };
}
pub use crate::{CONTAINING_RECORD, NT_SUCCESS};

// A kernel island's `use ntkrnl_sys::ddk::*;` is private: glob it here.
pub use ntkrnl_sys::ddk::*;

// ================= PnP / resource constants the -sys glob does not carry ===================
// The C switches on these. ntkrnl-sys has them only in `driver_ddk` (hand-written, every
// arch) and, on x86, `hal_ddk` -- neither is reached by the `ddk` glob the island
// re-exports, so a bare name in a match arm would bind everything (§9.112). Values: wdm.h.
pub use ntkrnl_sys::driver_ddk::{
    IRP_MN_CANCEL_REMOVE_DEVICE, IRP_MN_CANCEL_STOP_DEVICE, IRP_MN_DEVICE_USAGE_NOTIFICATION,
    IRP_MN_QUERY_CAPABILITIES, IRP_MN_QUERY_DEVICE_RELATIONS, IRP_MN_QUERY_DEVICE_TEXT,
    IRP_MN_QUERY_ID, IRP_MN_QUERY_INTERFACE, IRP_MN_QUERY_PNP_DEVICE_STATE, IRP_MN_QUERY_POWER,
    IRP_MN_QUERY_REMOVE_DEVICE, IRP_MN_QUERY_RESOURCES, IRP_MN_QUERY_RESOURCE_REQUIREMENTS,
    IRP_MN_QUERY_STOP_DEVICE, IRP_MN_REMOVE_DEVICE, IRP_MN_SET_POWER, IRP_MN_START_DEVICE,
    IRP_MN_STOP_DEVICE, IRP_MN_SURPRISE_REMOVAL,
};
// CM_PARTIAL_RESOURCE_DESCRIPTOR.Type is a UCHAR.
pub const CmResourceTypePort: u8 = 1;
pub const CmResourceTypeInterrupt: u8 = 2;
pub const CmResourceTypeMemory: u8 = 3;

// ==== port resolve: cross-TU definitions (generated) ====
pub use crate::pciidex::ahci_generic::AhciGetControllerProperties;
pub use crate::pciidex::amd::AmdGetControllerProperties;
pub use crate::pciidex::acpi::AtaAcpiGetTaskFile;
pub use crate::pciidex::acpi::AtaAcpiGetTimingMode;
pub use crate::pciidex::acpi::AtaAcpiSetDeviceData;
pub use crate::pciidex::acpi::AtaAcpiSetTimingMode;
pub use crate::pciidex::ahci_io::AtaAhciAllocateSlot;
pub use crate::pciidex::ahci_hw::AtaAhciChannelGetMaximumDeviceCount;
pub use crate::pciidex::ahci_hw::AtaAhciDowngradeInterfaceSpeed;
pub use crate::pciidex::ahci_hw::AtaAhciEnableInterrupts;
pub use crate::pciidex::ahci_hw::AtaAhciEnumerateChannel;
pub use crate::pciidex::ahci_hw::AtaAhciHandleFatalError;
pub use crate::pciidex::ahci_hw::AtaAhciHandlePortStateChange;
pub use crate::pciidex::ahci_io::AtaAhciHbaIsr;
pub use crate::pciidex::ahci_hw::AtaAhciIdentifyDevice;
pub use crate::pciidex::ahci_io::AtaAhciPrepareIo;
pub use crate::pciidex::ahci_io::AtaAhciPreparePrdTable;
pub use crate::pciidex::ahci_hw::AtaAhciResetChannel;
pub use crate::pciidex::ahci_hw::AtaAhciSaveTaskFile;
pub use crate::pciidex::ahci_io::AtaAhciStartIo;
pub use crate::pciidex::ahci_hw::AtaAhciStopDma;
pub use crate::pciidex::fdo::AtaChanEnableInterruptsSync;
pub use crate::pciidex::fdo::AtaCtrlAbortChannel;
pub use crate::pciidex::fdo::AtaCtrlAttachChannel;
pub use crate::pciidex::fdo::AtaCtrlDowngradeInterfaceSpeed;
pub use crate::pciidex::fdo::AtaCtrlGetInitTaskFile;
pub use crate::pciidex::fdo::AtaCtrlPciMapBar;
pub use crate::pciidex::fdo::AtaCtrlSetDeviceData;
pub use crate::pciidex::fdo::AtaCtrlSetTransferMode;
pub use crate::pciidex::pata_generic::AtaSelectTimings;
pub use crate::pciidex::pciidex::AtaSleep;
pub use crate::pciidex::ati::AtiGetControllerProperties;
pub use crate::pciidex::cmd::CmdGetControllerProperties;
pub use crate::pciidex::intel::IntelGetControllerProperties;
pub use crate::pciidex::pata_io::PataAllocateSlot;
pub use crate::pciidex::pata_hw::PataChannelGetMaximumDeviceCount;
pub use crate::pciidex::pata_io::PataChannelIsr;
pub use crate::pciidex::pata_hw::PataEnumerateChannel;
pub use crate::pciidex::pata_hw::PataIdentifyDevice;
pub use crate::pciidex::pata_io::PataLoadTaskFile;
pub use crate::pciidex::pata_io::PataPollingTimerDpc;
pub use crate::pciidex::pata_io::PataPrepareIo;
pub use crate::pciidex::pata_io::PataReadStatus;
pub use crate::pciidex::pata_hw::PataResetChannel;
pub use crate::pciidex::pata_io::PataSaveTaskFile;
pub use crate::pciidex::pata_io::PataStartIo;
pub use crate::pciidex::pctech::PcTechGetControllerProperties;
pub use crate::pciidex::pciidex::PciFindDevice;
pub use crate::pciidex::pata_generic::PciIdeAttachChannel;
pub use crate::pciidex::pata_io::PciIdeChannelIsr;
pub use crate::pciidex::pata_io::PciIdeCheckInterrupt;
pub use crate::pciidex::pata_generic::PciIdeConnectInterrupt;
pub use crate::pciidex::pata_generic::PciIdeCreateChannelData;
pub use crate::pciidex::pata_io::PciIdeDmaStop;
pub use crate::pciidex::pata_generic::PciIdeFreeResources;
pub use crate::pciidex::pata_generic::PciIdeGetChannelState;
pub use crate::pciidex::pata_generic::PciIdeGetControllerProperties;
pub use crate::pciidex::pata_generic::PciIdeInitTaskFileIoResources;
pub use crate::pciidex::pata_generic::PciIdeParseResources;
pub use crate::pciidex::pata_io::PciIdePreparePrdTable;
pub use crate::pciidex::pdo::PciIdeXDispatchPnp;
pub use crate::pciidex::power::PciIdeXDispatchPower;
pub use crate::pciidex::fdo::PciIdeXFdoDispatchPnp;
pub use crate::pciidex::fdo::PciIdeXGetChannelState;
pub use crate::pciidex::pciidex::PciIdeXPdoCompletionRoutine;
pub use crate::pciidex::pdo::PciIdeXPdoRemoveDevice;
pub use crate::pciidex::pciidex::PciIdeXPnpQueryDeviceUsageNotification;
pub use crate::pciidex::pciidex::PciIdeXPnpQueryPnpDeviceState;
pub use crate::pciidex::pciidex::PciIdeXPnpRepeatRequest;
pub use crate::pciidex::pciidex::PciRead;
pub use crate::pciidex::pata_generic::SataSetTransferMode;
pub use crate::pciidex::sil680::Sil680GetControllerProperties;
pub use crate::pciidex::svw_pata::SvwHasUdmaCable;
pub use crate::pciidex::svw_pata::SvwPataGetControllerProperties;
pub use crate::pciidex::svw_sata::SvwSataGetControllerProperties;
pub use crate::pciidex::svw_pata::SvwSetTransferMode;
pub use crate::pciidex::toshiba::ToshibaGetControllerProperties;
pub use crate::pciidex::via::ViaClampTimings;
pub use crate::pciidex::via::ViaGetControllerProperties;
// ==== end cross-TU definitions ====

// ==== port resolve: tree consensus (generated -- re-run `rustconv port resolve`) ====
pub const ACPI_EVAL_INPUT_BUFFER_SIGNATURE: ULONG = 0x42696541; // pcix-rust/src/fdo.rs
pub const ACPI_METHOD_ARGUMENT_BUFFER: u32 = 2; // atapi-rust/src/types_amd64.rs
pub const AHCI_CAP2_BOH: u32 = 1; // uniata-rust/src/types_amd64.rs
pub const AHCI_CAP_S64A: u32 = 2147483648; // uniata-rust/src/types_amd64.rs
pub const AHCI_CAP_SCLO: u32 = 16777216; // uniata-rust/src/types_amd64.rs
pub const AHCI_CAP_SPM: u32 = 131072; // uniata-rust/src/types_amd64.rs
pub const AHCI_GHC_AE: u32 = 2147483648; // uniata-rust/src/types_amd64.rs
pub const AHCI_GHC_HR: u32 = 1; // uniata-rust/src/types_amd64.rs
pub const AHCI_GHC_IE: u32 = 2; // uniata-rust/src/types_amd64.rs
pub const APC_LEVEL: u32 = 1; // atapi-rust/src/prelude.rs
pub const ATAPI_INT_REASON_COD: UCHAR = 0x01; // freeldr-rust/src/hwide.rs
pub const ATAPI_INT_REASON_IO: UCHAR = 0x02; // freeldr-rust/src/hwide.rs
pub const ATA_CHANNEL_FLAG_IS_AHCI: u32 = 8; // atapi-rust/src/prelude.rs
pub const ATA_CHANNEL_FLAG_IS_EXTERNAL: u32 = 2; // atapi-rust/src/prelude.rs
pub const ATA_CHANNEL_FLAG_NCQ: u32 = 4; // atapi-rust/src/prelude.rs
pub const ATA_CHANNEL_FLAG_PIO_FOR_LBA48_XFER: u32 = 16; // atapi-rust/src/prelude.rs
pub const ATA_CHANNEL_FLAG_PIO_VIA_DMA: u32 = 1; // atapi-rust/src/prelude.rs
pub const ATA_MAX_DEVICE: u32 = 15; // atapi-rust/src/prelude.rs
pub const ATA_MAX_TRANSFER_LENGTH: u32 = 131072; // atapi-rust/src/types_amd64.rs
pub const ATA_SCONTROL: _SATA_SCR_REGISTER = _SATA_SCR_REGISTER_ATA_SCONTROL; // island enum
pub const ATA_SERROR: _SATA_SCR_REGISTER = _SATA_SCR_REGISTER_ATA_SERROR; // island enum
pub const ATA_SSTATUS: _SATA_SCR_REGISTER = _SATA_SCR_REGISTER_ATA_SSTATUS; // island enum
pub const ATA_TIME_BUSY_POLL: ULONG = 500000; // freeldr-rust/src/hwide.rs
pub const ATA_TIME_BUSY_RESET: u32 = 1000000; // legide-rust/src/legide/legide.rs
pub const ATA_TIME_DRQ_ASSERT: ULONG = 15; // freeldr-rust/src/hwide.rs
pub const ATA_TIME_DRQ_CLEAR: u32 = 1000; // legide-rust/src/legide/legide.rs
pub const ATA_TIME_RESET_SELECT: u32 = 200000; // legide-rust/src/legide/legide.rs
pub const AtaAsyncNotificationDetected: _PORT_NOTIFICATION_TYPE = _PORT_NOTIFICATION_TYPE_AtaAsyncNotificationDetected; // island enum
pub const AtaBusChangeDetected: _PORT_NOTIFICATION_TYPE = _PORT_NOTIFICATION_TYPE_AtaBusChangeDetected; // island enum
pub const AtaRequestComplete: _PORT_NOTIFICATION_TYPE = _PORT_NOTIFICATION_TYPE_AtaRequestComplete; // island enum
pub const AtaRequestFailed: _PORT_NOTIFICATION_TYPE = _PORT_NOTIFICATION_TYPE_AtaRequestFailed; // island enum
pub const AtaResetDetected: _PORT_NOTIFICATION_TYPE = _PORT_NOTIFICATION_TYPE_AtaResetDetected; // island enum
pub const BusQueryCompatibleIDs: _BUS_QUERY_ID_TYPE = _BUS_QUERY_ID_TYPE_BusQueryCompatibleIDs; // island enum
pub const BusQueryDeviceID: _BUS_QUERY_ID_TYPE = _BUS_QUERY_ID_TYPE_BusQueryDeviceID; // island enum
pub const BusQueryHardwareIDs: _BUS_QUERY_ID_TYPE = _BUS_QUERY_ID_TYPE_BusQueryHardwareIDs; // island enum
pub const BusQueryInstanceID: _BUS_QUERY_ID_TYPE = _BUS_QUERY_ID_TYPE_BusQueryInstanceID; // island enum
pub const BusRelations: _DEVICE_RELATION_TYPE = _DEVICE_RELATION_TYPE_BusRelations; // island enum
pub const CM_RESOURCE_INTERRUPT_LATCHED: u32 = 1; // atapi-rust/src/types_amd64.rs
pub const CM_RESOURCE_PORT_16_BIT_DECODE: u32 = 16; // atapi-rust/src/types_amd64.rs
pub const CM_RESOURCE_PORT_IO: u32 = 1; // atapi-rust/src/types_amd64.rs
pub const CONN_STATUS_DEV_ATA: _ATA_CONNECTION_STATUS = _ATA_CONNECTION_STATUS_CONN_STATUS_DEV_ATA; // island enum
pub const CONN_STATUS_DEV_ATAPI: _ATA_CONNECTION_STATUS = _ATA_CONNECTION_STATUS_CONN_STATUS_DEV_ATAPI; // island enum
pub const CONN_STATUS_DEV_UNKNOWN: _ATA_CONNECTION_STATUS = _ATA_CONNECTION_STATUS_CONN_STATUS_DEV_UNKNOWN; // island enum
pub const CONN_STATUS_FAILURE: _ATA_CONNECTION_STATUS = _ATA_CONNECTION_STATUS_CONN_STATUS_FAILURE; // island enum
pub const CONN_STATUS_NO_DEVICE: _ATA_CONNECTION_STATUS = _ATA_CONNECTION_STATUS_CONN_STATUS_NO_DEVICE; // island enum
pub const ChannelDisabled: IDE_CHANNEL_STATE = IDE_CHANNEL_STATE_ChannelDisabled; // island enum
pub const ChannelEnabled: IDE_CHANNEL_STATE = IDE_CHANNEL_STATE_ChannelEnabled; // island enum
pub const ChannelStateUnknown: IDE_CHANNEL_STATE = IDE_CHANNEL_STATE_ChannelStateUnknown; // island enum
pub const CmResourceShareDeviceExclusive: u8 = 1; // legide-rust/src/prelude.rs
pub const CmResourceShareShared: _CM_SHARE_DISPOSITION = _CM_SHARE_DISPOSITION_CmResourceShareShared; // island enum
pub const DEVICE_DESCRIPTION_VERSION: u32 = 0; // atapi-rust/src/types_amd64.rs
pub const DEVICE_HAS_CDB_INTERRUPT: u32 = 32; // atapi-rust/src/prelude.rs
pub const DEVICE_IS_ATAPI: u32 = 16; // atapi-rust/src/prelude.rs
pub const DEVICE_IS_NEC_CDR260: u32 = 128; // atapi-rust/src/prelude.rs
pub const DEVICE_NEED_DMA_DIRECTION: u32 = 64; // atapi-rust/src/prelude.rs
pub const DIRECTORY_ALL_ACCESS: u32 = 983055; // atapi-rust/src/prelude.rs
pub const DISPATCH_LEVEL: u32 = 2; // atapi-rust/src/prelude.rs
pub const DO_DEVICE_INITIALIZING: u32 = 128; // atapi-rust/src/prelude.rs
pub const DO_IS_FDO: u32 = 2147483648; // atapi-rust/src/prelude.rs
pub const DPFLTR_PCIIDE_ID: _DPFLTR_TYPE = _DPFLTR_TYPE_DPFLTR_PCIIDE_ID; // island enum
pub const DevicePropertyAddress: DEVICE_REGISTRY_PROPERTY = DEVICE_REGISTRY_PROPERTY_DevicePropertyAddress; // island enum
pub const DeviceTextDescription: _DEVICE_TEXT_TYPE = _DEVICE_TEXT_TYPE_DeviceTextDescription; // island enum
pub const DeviceTextLocationInformation: _DEVICE_TEXT_TYPE = _DEVICE_TEXT_TYPE_DeviceTextLocationInformation; // island enum
pub const DeviceUsageTypeDumpFile: _DEVICE_USAGE_NOTIFICATION_TYPE = _DEVICE_USAGE_NOTIFICATION_TYPE_DeviceUsageTypeDumpFile; // island enum
pub const DeviceUsageTypeHibernation: _DEVICE_USAGE_NOTIFICATION_TYPE = _DEVICE_USAGE_NOTIFICATION_TYPE_DeviceUsageTypeHibernation; // island enum
pub const DeviceUsageTypePaging: _DEVICE_USAGE_NOTIFICATION_TYPE = _DEVICE_USAGE_NOTIFICATION_TYPE_DeviceUsageTypePaging; // island enum
pub const FILE_DEVICE_BUS_EXTENDER: u32 = 42; // atapi-rust/src/types_amd64.rs
pub const FILE_DEVICE_CONTROLLER: u32 = 4; // atapi-rust/src/prelude.rs
pub const FILE_DEVICE_SECURE_OPEN: u32 = 256; // atapi-rust/src/prelude.rs
pub const HbaAhciVersion: _AHCI_HOST_BUS_ADAPTER_REGISTER = _AHCI_HOST_BUS_ADAPTER_REGISTER_HbaAhciVersion; // island enum
pub const HbaBiosHandoffControl: _AHCI_HOST_BUS_ADAPTER_REGISTER = _AHCI_HOST_BUS_ADAPTER_REGISTER_HbaBiosHandoffControl; // island enum
pub const HbaCapabilities: _AHCI_HOST_BUS_ADAPTER_REGISTER = _AHCI_HOST_BUS_ADAPTER_REGISTER_HbaCapabilities; // island enum
pub const HbaCapabilitiesEx: _AHCI_HOST_BUS_ADAPTER_REGISTER = _AHCI_HOST_BUS_ADAPTER_REGISTER_HbaCapabilitiesEx; // island enum
pub const HbaGlobalControl: _AHCI_HOST_BUS_ADAPTER_REGISTER = _AHCI_HOST_BUS_ADAPTER_REGISTER_HbaGlobalControl; // island enum
pub const HbaInterruptStatus: _AHCI_HOST_BUS_ADAPTER_REGISTER = _AHCI_HOST_BUS_ADAPTER_REGISTER_HbaInterruptStatus; // island enum
pub const HbaPortBitmap: _AHCI_HOST_BUS_ADAPTER_REGISTER = _AHCI_HOST_BUS_ADAPTER_REGISTER_HbaPortBitmap; // island enum
pub const IDE_ACPI_TIMING_MODE_FLAG_INDEPENDENT_TIMINGS: u32 = 16; // atapi-rust/src/types_amd64.rs
pub const IDE_ACPI_TIMING_MODE_NOT_SUPPORTED: u32 = 4294967295; // atapi-rust/src/prelude.rs
pub const IDE_COMMAND_ATAPI_IDENTIFY: u32 = 161; // atapi-legacy-rust/src/types_amd64.rs
pub const IDE_COMMAND_ATAPI_PACKET: u32 = 160; // atapi-legacy-rust/src/types_amd64.rs
pub const IDE_COMMAND_IDENTIFY: u32 = 236; // atapi-legacy-rust/src/types_amd64.rs
pub const IDE_DC_ALWAYS: u32 = 8; // atapi-rust/src/types_amd64.rs
pub const IDE_DC_DISABLE_INTERRUPTS: u32 = 2; // atapi-legacy-rust/src/types_amd64.rs
pub const IDE_DC_REENABLE_CONTROLLER: u32 = 0; // atapi-legacy-rust/src/types_amd64.rs
pub const IDE_DC_RESET_CONTROLLER: u32 = 4; // atapi-legacy-rust/src/types_amd64.rs
pub const IDE_DRIVE_SELECT: u32 = 160; // atapi-rust/src/prelude.rs
pub const IDE_FEATURE_DMA: u32 = 1; // atapi-rust/src/types_amd64.rs
pub const IDE_FEATURE_DMADIR: u32 = 4; // atapi-rust/src/types_amd64.rs
pub const IDE_FEATURE_PIO: u32 = 0; // atapi-rust/src/types_amd64.rs
pub const IDE_HIGH_ORDER_BYTE: u32 = 128; // atapi-rust/src/types_amd64.rs
pub const IDE_STATUS_BUSY: u32 = 128; // atapi-legacy-rust/src/types_amd64.rs
pub const IDE_STATUS_DEVICE_FAULT: u32 = 32; // atapi-rust/src/prelude.rs
pub const IDE_STATUS_DRQ: u32 = 8; // atapi-legacy-rust/src/types_amd64.rs
pub const IDE_STATUS_ERROR: u32 = 1; // atapi-legacy-rust/src/types_amd64.rs
pub const IO_NO_INCREMENT: u32 = 0; // atapi-rust/src/prelude.rs
pub const IRP_BUFFERED_IO: u32 = 16; // atapi-rust/src/types_amd64.rs
pub const IRP_INPUT_OPERATION: u32 = 64; // atapi-rust/src/types_amd64.rs
pub const IRP_MJ_DEVICE_CONTROL: u32 = 14; // atapi-rust/src/prelude.rs
pub const IRP_MJ_PNP: u32 = 27; // atapi-rust/src/prelude.rs
pub const IRP_MJ_POWER: u32 = 22; // atapi-rust/src/prelude.rs
pub const IRP_MJ_SYSTEM_CONTROL: u32 = 23; // atapi-rust/src/prelude.rs
pub const InterfaceTypeUndefined: _INTERFACE_TYPE = _INTERFACE_TYPE_InterfaceTypeUndefined; // island enum
pub const IsNEC_98: u32 = 0; // atapi-rust/src/prelude.rs
pub const Isa: _INTERFACE_TYPE = _INTERFACE_TYPE_Isa; // island enum
pub const Latched: _KINTERRUPT_MODE = _KINTERRUPT_MODE_Latched; // island enum
pub const LevelSensitive: _KINTERRUPT_MODE = _KINTERRUPT_MODE_LevelSensitive; // island enum
pub const MAXULONG: u32 = 4294967295; // atapi-rust/src/prelude.rs
pub const MAX_IDE_CHANNEL: u32 = 2; // atapi-rust/src/types_amd64.rs
pub const MAX_IDE_DEVICE: u32 = 2; // atapi-rust/src/prelude.rs
pub const MWDMA_ALL: u32 = 1792; // atapi-rust/src/prelude.rs
pub const MWDMA_MODE2: u32 = 1024; // atapi-rust/src/types_amd64.rs
pub const MmNonCached: MEMORY_CACHING_TYPE = 0; // atapi-rust/src/prelude.rs
pub const NonPagedPool: POOL_TYPE = 0; // atapi-rust/src/prelude.rs
pub const OBJ_CASE_INSENSITIVE: u32 = 64; // atapi-rust/src/prelude.rs
pub const OBJ_KERNEL_HANDLE: u32 = 512; // atapi-rust/src/prelude.rs
pub const OBJ_PERMANENT: u32 = 16; // atapi-rust/src/prelude.rs
pub const PAGE_SHIFT: u32 = 12; // atapi-rust/src/types_amd64.rs
pub const PAGE_SIZE: u32 = 4096; // atapi-rust/src/prelude.rs
pub const PCIBus: _INTERFACE_TYPE = _INTERFACE_TYPE_PCIBus; // island enum
pub const PCIConfiguration: _BUS_DATA_TYPE = _BUS_DATA_TYPE_PCIConfiguration; // island enum
pub const PCIIDEX_GET_CONTROLLER_INTERFACE_SIGNATURE: u32 = 4294963199; // atapi-rust/src/types_amd64.rs
pub const PCIIDEX_INTERFACE_VERSION: u32 = 1; // atapi-rust/src/prelude.rs
pub const PCIIDE_LEGACY_COMMAND_IO_RANGE_LENGTH: u32 = 8; // pciidex-legacy-rust/src/types_amd64.rs
pub const PCIIDE_LEGACY_CONTROL_IO_RANGE_LENGTH: u32 = 1; // pciidex-legacy-rust/src/types_amd64.rs
pub const PCIIDE_LEGACY_PRIMARY_COMMAND_BASE: u32 = 496; // pciidex-legacy-rust/src/types_amd64.rs
pub const PCIIDE_LEGACY_PRIMARY_CONTROL_BASE: u32 = 1014; // pciidex-legacy-rust/src/types_amd64.rs
pub const PCIIDE_LEGACY_PRIMARY_IRQ: u32 = 14; // pciidex-legacy-rust/src/types_amd64.rs
pub const PCIIDE_LEGACY_RESOURCE_COUNT: u32 = 3; // pciidex-legacy-rust/src/types_amd64.rs
pub const PCIIDE_LEGACY_SECONDARY_COMMAND_BASE: u32 = 368; // pciidex-legacy-rust/src/types_amd64.rs
pub const PCIIDE_LEGACY_SECONDARY_CONTROL_BASE: u32 = 886; // pciidex-legacy-rust/src/types_amd64.rs
pub const PCIIDE_LEGACY_SECONDARY_IRQ: u32 = 15; // pciidex-legacy-rust/src/types_amd64.rs
pub const PCIIDE_PROGIF_DMA_CAPABLE: u32 = 128; // pciidex-legacy-rust/src/types_amd64.rs
pub const PCIIDE_PROGIF_PRIMARY_CHANNEL_NATIVE_MODE: u32 = 1; // pciidex-legacy-rust/src/types_amd64.rs
pub const PCIIDE_PROGIF_SECONDARY_CHANNEL_NATIVE_MODE: u32 = 4; // pciidex-legacy-rust/src/types_amd64.rs
pub const PCI_ADDRESS_IO_ADDRESS_MASK: u32 = 4294967292; // atapi-rust/src/types_amd64.rs
pub const PCI_ADDRESS_IO_SPACE: u32 = 1; // atapi-rust/src/types_amd64.rs
pub const PCI_ADDRESS_MEMORY_ADDRESS_MASK: u32 = 4294967280; // atapi-rust/src/types_amd64.rs
pub const PCI_ADDRESS_MEMORY_PREFETCHABLE: u32 = 8; // atapi-rust/src/types_amd64.rs
pub const PCI_ADDRESS_MEMORY_TYPE_MASK: u32 = 6; // atapi-rust/src/types_amd64.rs
pub const PCI_BUS_INTERFACE_STANDARD_VERSION: u32 = 1; // legide-rust/src/types_amd64.rs
pub const PCI_CLASS_MASS_STORAGE_CTLR: u32 = 1; // atapi-rust/src/types_amd64.rs
pub const PCI_ENABLE_BUS_MASTER: u32 = 4; // atapi-rust/src/types_amd64.rs
pub const PCI_INVALID_VENDORID: u32 = 65535; // atapi-rust/src/types_amd64.rs
pub const PCI_MAX_DEVICES: u32 = 32; // atapi-rust/src/types_amd64.rs
pub const PCI_MAX_FUNCTION: u32 = 8; // atapi-rust/src/types_amd64.rs
pub const PCI_SUBCLASS_MSC_AHCI_CTLR: u32 = 6; // atapi-rust/src/types_amd64.rs
pub const PCI_SUBCLASS_MSC_IDE_CTLR: u32 = 1; // atapi-rust/src/types_amd64.rs
pub const PCI_SUBCLASS_MSC_RAID_CTLR: u32 = 4; // atapi-rust/src/types_amd64.rs
pub const PCI_TYPE0_ADDRESSES: u32 = 6; // atapi-rust/src/types_amd64.rs
pub const PCI_TYPE_64BIT: u32 = 4; // atapi-rust/src/types_amd64.rs
pub const PCI_WHICHSPACE_CONFIG: u32 = 0; // atapi-rust/src/types_amd64.rs
pub const PIO_ALL: u32 = 31; // atapi-rust/src/prelude.rs
pub const PNP_DEVICE_NOT_DISABLEABLE: u32 = 32; // atapi-rust/src/prelude.rs
pub const PagedPool: POOL_TYPE = 1; // atapi-rust/src/prelude.rs
pub const PmpPortInfo: _AHCI_PORT_MULTIPLIER_REGISTER = _AHCI_PORT_MULTIPLIER_REGISTER_PmpPortInfo; // island enum
pub const PmpProductId: _AHCI_PORT_MULTIPLIER_REGISTER = _AHCI_PORT_MULTIPLIER_REGISTER_PmpProductId; // island enum
pub const PmpRevisionInfo: _AHCI_PORT_MULTIPLIER_REGISTER = _AHCI_PORT_MULTIPLIER_REGISTER_PmpRevisionInfo; // island enum
pub const PxCmdStatus: _AHCI_PORT_REGISTER = _AHCI_PORT_REGISTER_PxCmdStatus; // island enum
pub const PxCommandIssue: _AHCI_PORT_REGISTER = _AHCI_PORT_REGISTER_PxCommandIssue; // island enum
pub const PxCommandListBaseHigh: _AHCI_PORT_REGISTER = _AHCI_PORT_REGISTER_PxCommandListBaseHigh; // island enum
pub const PxCommandListBaseLow: _AHCI_PORT_REGISTER = _AHCI_PORT_REGISTER_PxCommandListBaseLow; // island enum
pub const PxDeviceSleep: _AHCI_PORT_REGISTER = _AHCI_PORT_REGISTER_PxDeviceSleep; // island enum
pub const PxFisBaseHigh: _AHCI_PORT_REGISTER = _AHCI_PORT_REGISTER_PxFisBaseHigh; // island enum
pub const PxFisBaseLow: _AHCI_PORT_REGISTER = _AHCI_PORT_REGISTER_PxFisBaseLow; // island enum
pub const PxFisSwitchingControl: _AHCI_PORT_REGISTER = _AHCI_PORT_REGISTER_PxFisSwitchingControl; // island enum
pub const PxInterruptEnable: _AHCI_PORT_REGISTER = _AHCI_PORT_REGISTER_PxInterruptEnable; // island enum
pub const PxInterruptStatus: _AHCI_PORT_REGISTER = _AHCI_PORT_REGISTER_PxInterruptStatus; // island enum
pub const PxSataActive: _AHCI_PORT_REGISTER = _AHCI_PORT_REGISTER_PxSataActive; // island enum
pub const PxSataControl: _AHCI_PORT_REGISTER = _AHCI_PORT_REGISTER_PxSataControl; // island enum
pub const PxSataError: _AHCI_PORT_REGISTER = _AHCI_PORT_REGISTER_PxSataError; // island enum
pub const PxSataNotification: _AHCI_PORT_REGISTER = _AHCI_PORT_REGISTER_PxSataNotification; // island enum
pub const PxSataStatus: _AHCI_PORT_REGISTER = _AHCI_PORT_REGISTER_PxSataStatus; // island enum
pub const PxSignature: _AHCI_PORT_REGISTER = _AHCI_PORT_REGISTER_PxSignature; // island enum
pub const PxTaskFileData: _AHCI_PORT_REGISTER = _AHCI_PORT_REGISTER_PxTaskFileData; // island enum
pub const REQUEST_FLAG_DATA_IN: u32 = 64; // atapi-rust/src/prelude.rs
pub const REQUEST_FLAG_DATA_OUT: u32 = 128; // atapi-rust/src/prelude.rs
pub const REQUEST_FLAG_DEVICE_EXCLUSIVE_ACCESS: u32 = 16777216; // atapi-rust/src/types_amd64.rs
pub const REQUEST_FLAG_DMA: u32 = 1; // atapi-rust/src/prelude.rs
pub const REQUEST_FLAG_HAS_TASK_FILE: u32 = 1024; // atapi-rust/src/prelude.rs
pub const REQUEST_FLAG_NCQ: u32 = 4; // atapi-rust/src/prelude.rs
pub const REQUEST_FLAG_POLL: u32 = 2147483648; // atapi-rust/src/prelude.rs
pub const REQUEST_FLAG_PROGRAM_DMA: u32 = 2097152; // atapi-rust/src/prelude.rs
pub const REQUEST_FLAG_SAVE_TASK_FILE: u32 = 512; // atapi-rust/src/prelude.rs
pub const REQUEST_FLAG_SET_AUXILIARY_FIELD: u32 = 4096; // atapi-rust/src/types_amd64.rs
pub const REQUEST_FLAG_SET_ICC_FIELD: u32 = 8192; // atapi-rust/src/types_amd64.rs
pub const SRB_STATUS_BUS_RESET: u32 = 14; // atapi-legacy-rust/src/types_amd64.rs
pub const SRB_STATUS_DATA_OVERRUN: u32 = 18; // atapi-legacy-rust/src/types_amd64.rs
pub const SRB_STATUS_ERROR: u32 = 4; // atapi-legacy-rust/src/types_amd64.rs
pub const SRB_STATUS_PENDING: u32 = 0; // atapi-legacy-rust/src/types_amd64.rs
pub const SRB_STATUS_SELECTION_TIMEOUT: u32 = 10; // atapi-legacy-rust/src/types_amd64.rs
pub const SRB_STATUS_SUCCESS: u32 = 1; // atapi-legacy-rust/src/types_amd64.rs
pub const SRB_STATUS_TIMEOUT: u32 = 9; // atapi-legacy-rust/src/types_amd64.rs
pub const STATUS_BUFFER_OVERFLOW: i32 = 0x80000005_u32 as i32; // atapi-rust/src/prelude.rs
pub const STATUS_DEVICE_BUSY: i32 = 0x80000011u32 as i32; // atapi-rust/src/prelude.rs
pub const STATUS_DEVICE_CONFIGURATION_ERROR: i32 = 0xC0000182u32 as i32; // pciidex-legacy-rust/src/glue.rs
pub const STATUS_INSUFFICIENT_RESOURCES: NTSTATUS = 0xC000_009Au32 as NTSTATUS; // atapi-rust/src/prelude.rs
pub const STATUS_IO_DEVICE_ERROR: i32 = 0xC0000185u32 as i32; // atapi-rust/src/prelude.rs
pub const STATUS_IO_TIMEOUT: i32 = 0xC00000B5u32 as i32; // atapi-rust/src/prelude.rs
pub const STATUS_MORE_PROCESSING_REQUIRED: i32 = 0xC0000016u32 as i32; // atapi-rust/src/prelude.rs
pub const STATUS_NOT_IMPLEMENTED: NTSTATUS = 0xC0000002u32 as NTSTATUS; // atapi-rust/src/prelude.rs
pub const STATUS_NOT_SUPPORTED: i32 = 0xC00000BB_u32 as i32; // atapi-rust/src/prelude.rs
pub const STATUS_NO_MATCH: NTSTATUS = 0xC0000272u32 as i32; // usbaudio-rust/src/glue.rs
pub const STATUS_OBJECT_NAME_NOT_FOUND: NTSTATUS = 0xC0000034u32 as NTSTATUS; // kernel32-rust/src/console/history.rs
pub const STATUS_PENDING: NTSTATUS = 0x00000103; // atapi-rust/src/prelude.rs
pub const STATUS_SUCCESS: i32 = 0; // atapi-rust/src/prelude.rs
pub const STATUS_UNSUCCESSFUL: NTSTATUS = 0xC000_0001u32 as NTSTATUS; // atapi-rust/src/prelude.rs
pub const SWDMA_ALL: u32 = 224; // atapi-rust/src/prelude.rs
pub const SWDMA_MODE2: u32 = 128; // atapi-rust/src/types_amd64.rs
pub const TAG_PCIIDEX: u32 = 0x58656449; // pciidex-legacy-rust/src/glue.rs
pub const TargetDeviceRelation: _DEVICE_RELATION_TYPE = _DEVICE_RELATION_TYPE_TargetDeviceRelation; // island enum
pub const UDMA_ALL: u32 = 260096; // atapi-rust/src/prelude.rs
pub const UDMA_MODE3: u32 = 16384; // atapi-rust/src/types_amd64.rs
pub const UDMA_MODE4: u32 = 32768; // atapi-rust/src/types_amd64.rs
pub const UDMA_MODE5: u32 = 65536; // atapi-rust/src/types_amd64.rs
pub const UDMA_MODE6: u32 = 131072; // atapi-rust/src/types_amd64.rs
extern "C" {
    pub fn _strnicmp(s1: PCSTR, s2: PCSTR, n: usize) -> i32; // freeldr-rust/src/arcsupp.rs
}
extern "system" {
    pub fn PoCallDriver(DeviceObject: PDEVICE_OBJECT, Irp: PIRP) -> NTSTATUS; // hal-rust/src/arm64/pnpdd.rs
    pub fn PoStartNextPowerIrp(Irp: PIRP); // hal-rust/src/arm64/pnpdd.rs
    pub fn RtlAssert( FailedAssertion: PVOID, FileName: PVOID, LineNumber: ULONG, Message: *mut CHAR, ); // mountmgr-rust/src/mountmgr.rs
}
// ==== end port resolve ====
// ntkrnl-sys's ASSERT, not win32macros': DBG=1 here, so the C's ASSERT calls RtlAssert, which
// prints and continues. win32macros' is the release-build no-op.
pub use ntkrnl_sys::ASSERT;
pub use win32macros_rust::{ALIGN_UP, ARRAYSIZE, C_ASSERT, CopyMemory, DBG_UNREFERENCED_LOCAL_VARIABLE, DWLP_DLGPROC, DWLP_MSGRESULT, DWLP_USER, FIELD_OFFSET, GET_X_LPARAM, GET_Y_LPARAM, GetBValue, GetGValue, GetRValue, HIBYTE, HIWORD, IDE_SET_ADVANCE_PIO_MODE, IDE_SET_DEFAULT_PIO_MODE, IDE_SET_MWDMA_MODE, IDE_SET_SWDMA_MODE, IDE_SET_UDMA_MODE, INVALID_HANDLE_VALUE, InitializeObjectAttributes, IsContainerPartition, IsEqualGUID, IsFTPartition, IsRecognizedPartition, LANGIDFROMLCID, LOBYTE, LOWORD, MAKEINTRESOURCEA, MAKEINTRESOURCEW, MAKELANGID, MAKELCID, MAKELONG, MAKEWORD, MoveMemory, PAGE_ROUND_UP, PRIMARYLANGID, PtrToUlong, RGB, RTL_CONSTANT_STRING, RTL_CONTAINS_FIELD, RTL_FIELD_SIZE, RTL_NUMBER_OF, RTL_NUMBER_OF_FIELD, RTL_SIZEOF_THROUGH_FIELD, RtlCopyMemory, RtlFillMemory, RtlGetProcessHeap, RtlMoveMemory, RtlZeroMemory, SORTIDFROMLCID, SRB_STATUS, SUBLANGID, UNICODE_NULL, UNREFERENCED_PARAMETER, UlongToPtr, ZeroMemory, _countof, iswalnum, iswalpha, iswcntrl, iswdigit, iswgraph, iswlower, iswprint, iswpunct, iswspace, iswupper, iswxdigit, wide_bytes};
pub use strsafe_rust::{RtlStringCbPrintfA, RtlStringCbPrintfW, RtlStringCchPrintfA, RtlStringCchPrintfExA, RtlStringCchPrintfExW, RtlStringCchPrintfW, StringCbCatW, StringCbCopyNExW, StringCbCopyW, StringCbPrintfExW, StringCbPrintfW, StringCchCatW, StringCchCopyNExW, StringCchCopyW, StringCchLengthW, StringCchPrintfW, StringCchVPrintfW};
pub use strsafe_rust::ntstrsafe::{RtlStringCatNWorkerW, RtlStringCatWorkerW, RtlStringCbCatNW, RtlStringCbCatW, RtlStringCbCopyNA, RtlStringCbCopyNW, RtlStringCbCopyW, RtlStringCchCatNW, RtlStringCchCatW, RtlStringCchCopyA, RtlStringCchCopyNW, RtlStringCchCopyW, RtlStringCopyNWorkerW, RtlStringCopyWorkerA, RtlStringCopyWorkerW, RtlStringLengthWorkerW};

// ==== port facts: constants evaluated by clang from the C headers (generated -- `rustconv port resolve`) ====
pub const ACPI_EVAL_INPUT_BUFFER_COMPLEX_SIGNATURE: u32 = 0x43696541; // clang, acpi.c
pub const ACPI_EVAL_OUTPUT_BUFFER_SIGNATURE: u32 = 0x426F6541; // clang, acpi.c
pub const AHCI_DELAY_1_SECOND: u32 = 0x64; // clang, ahci_hw.c
pub const AHCI_DELAY_CLO_CLEAR: u32 = 1; // clang, ahci_hw.c
pub const AHCI_DELAY_CR_START_STOP: u32 = 0x32; // clang, ahci_hw.c
pub const AHCI_DELAY_DET_PRESENCE: u32 = 4; // clang, ahci_hw.c
pub const AHCI_DELAY_DET_STABLE: u32 = 0x14; // clang, ahci_hw.c
pub const AHCI_DELAY_FR_START_STOP: u32 = 0x32; // clang, ahci_hw.c
pub const AHCI_DELAY_INTERFACE_CHANGE: u32 = 1; // clang, ahci_hw.c
pub const AHCI_DELAY_PMP_DET_PRESENSE: u32 = 0xA; // clang, ahci_hw.c
pub const AHCI_DELAY_PMP_READY_DRIVE: u32 = 0x28; // clang, ahci_hw.c
pub const AHCI_DELAY_READY_DRIVE: u32 = 0x3E8; // clang, ahci_hw.c
pub const AMD_UDMA_EN: u32 = 0xC0; // clang, amd.c
pub const IOCTL_ACPI_ASYNC_EVAL_METHOD: u32 = 0x32C000; // clang, acpi.c
pub const SIL_REG_SYS_CFG: u32 = 0x8A; // clang, sil680.c
pub const STATUS_ACPI_INVALID_DATA: NTSTATUS = 0xC014000F_u32 as NTSTATUS; // clang, acpi.c
pub const STATUS_DEVICE_HARDWARE_ERROR: NTSTATUS = 0xC0000483_u32 as NTSTATUS; // clang, ahci_generic.c
// ==== end port facts ====

// ==== port local types: types the module's .c files define, laid out by c2rust (generated -- `rustconv port resolve`) ====
// AHCI_PORT_REGISTER: c2rust, ahci_hw.c
pub type AHCI_PORT_REGISTER = _AHCI_PORT_REGISTER;
// ATA_CONNECTION_STATUS: c2rust, ahci_hw.c
pub type ATA_CONNECTION_STATUS = _ATA_CONNECTION_STATUS;
// INTEL_CONTROLLER_INFO: c2rust, intel.c
pub type INTEL_CONTROLLER_INFO = _INTEL_CONTROLLER_INFO;
// SATA_SCR_REGISTER: c2rust, svw_sata.c
pub type SATA_SCR_REGISTER = _SATA_SCR_REGISTER;
// _INTEL_CONTROLLER_INFO: c2rust, intel.c
#[derive(Copy, Clone)]
#[repr(C)]
pub struct _INTEL_CONTROLLER_INFO {
    pub DeviceID: USHORT,
    pub Type: USHORT,
}
// ==== end port local types ====

// ==== port externs: C functions declared outside the island -- c2rust signature, clang calling convention (generated -- `rustconv port resolve`) ====
// ==== end port externs ====

// ================= the module's own macros, hand-ported over the generated port ==========
// `port macros` skips a name defined here. These were bought with a width C does not have:
// C evaluates each in `int` (or in the operand's own promoted type) and narrows at the
// assignment, so a Rust port pinned to `u32` mismatched every call site.
//
// intel.c PIIX register fields: `int` results (C's literals and promoted UCHAR operands);
// every intel.rs site combines them as `x as i32 | ...` and narrows after.
macro_rules! PIIX_IDETIM_IE { ($drive:expr $(,)?) => { (0x0002i32 << (4 * ($drive as u32))) }; }
macro_rules! PIIX_IDETIM_PPE { ($drive:expr $(,)?) => { (0x0004i32 << (4 * ($drive as u32))) }; }
macro_rules! PIIX_IDETIM_DTE { ($drive:expr $(,)?) => { (0x0008i32 << (4 * ($drive as u32))) }; }
macro_rules! PIIX_IDETIM_RCT { ($value:expr $(,)?) => { (($value as i32) << 8) }; }
macro_rules! PIIX_IDETIM_ISP { ($value:expr $(,)?) => { (($value as i32) << 12) }; }
macro_rules! PIIX_IDETIM_SETTINGS {
    ($drive:expr, $value:expr $(,)?) => { (($value as i32) << (4 * ($drive as u32))) };
}
macro_rules! PIIX_SIDETIM_RCT {
    ($channel:expr, $value:expr $(,)?) => { (($value as i32) << (($channel as u32) * 4)) };
}
macro_rules! PIIX_SIDETIM_ISP {
    ($channel:expr, $value:expr $(,)?) => { (($value as i32) << ((($channel as u32) * 4) + 2)) };
}
macro_rules! PIIX_UDMACTL_EN {
    ($channel:expr, $drive:expr $(,)?) => { (0x01i32 << (($channel as u32) * 2 + ($drive as u32))) };
}
macro_rules! PIIX_UDMATIM_CT {
    ($drive:expr, $value:expr $(,)?) => { (($value as i32) << (($drive as u32) * 4)) };
}
macro_rules! PIIX_CONFIG_CR { ($channel:expr $(,)?) => { (0x0030i32 << (($channel as u32) * 2)) }; }
macro_rules! PIIX_CONFIG_CLOCK_UDMA66 {
    ($channel:expr, $drive:expr $(,)?) => { (0x0001i32 << ((($channel as u32) * 2) + ($drive as u32))) };
}
macro_rules! PIIX_CONFIG_CLOCK_UDMA100 {
    ($channel:expr, $drive:expr $(,)?) => { (0x1000i32 << ((($channel as u32) * 2) + ($drive as u32))) };
}
pub(crate) use {
    PIIX_CONFIG_CLOCK_UDMA100, PIIX_CONFIG_CLOCK_UDMA66, PIIX_CONFIG_CR, PIIX_IDETIM_DTE,
    PIIX_IDETIM_IE, PIIX_IDETIM_ISP, PIIX_IDETIM_PPE, PIIX_IDETIM_RCT, PIIX_IDETIM_SETTINGS,
    PIIX_SIDETIM_ISP, PIIX_SIDETIM_RCT, PIIX_UDMACTL_EN, PIIX_UDMATIM_CT,
};
// cmd.c interrupt/cable bits: the C stores them straight into a UCHAR, so the literal takes
// the type of the register it lands in (all values fit in 8 bits).
macro_rules! CMD_CFR_INTR { ($channel:expr $(,)?) => { (if ($channel) == 0 { 0x04 } else { 0x10 }) }; }
macro_rules! CMD_MRDMODE_INTR { ($channel:expr $(,)?) => { (0x04 << ($channel)) }; }
macro_rules! CMD_BMIDECSR_CR { ($channel:expr $(,)?) => { (0x01 << ($channel)) }; }
pub(crate) use {CMD_BMIDECSR_CR, CMD_CFR_INTR, CMD_MRDMODE_INTR};
// via.c `GET_TYPE(Flags) ((Flags) & HW_FLAGS_TYPE_MASK)`: the mask is a plain `int` 0x000F in
// the C; every caller passes a ULONG HwFlags word (the model typed the mask `u16`).
macro_rules! GET_TYPE {
    ($flags:expr $(,)?) => { (($flags) & (HW_FLAGS_TYPE_MASK as ULONG)) };
}
pub(crate) use GET_TYPE;
// pciidex.h `IS_PRIMARY_CHANNEL(x) (x->Channel == 0)` is applied to a PDO extension AND to a
// channel's data, whose PATA/AHCI forms embed the common part as an anonymous member. One
// trait, every path.
pub trait IdeChannelNumber {
    unsafe fn channel_number(this: *const Self) -> ULONG;
}
impl IdeChannelNumber for _PDO_DEVICE_EXTENSION {
    unsafe fn channel_number(this: *const Self) -> ULONG {
        (*this).Channel
    }
}
impl IdeChannelNumber for _CHANNEL_DATA_COMMON {
    unsafe fn channel_number(this: *const Self) -> ULONG {
        (*this).Channel
    }
}
impl IdeChannelNumber for _CHANNEL_DATA_PATA {
    unsafe fn channel_number(this: *const Self) -> ULONG {
        (*this).__bindgen_anon_1.Channel
    }
}
impl IdeChannelNumber for _CHANNEL_DATA_AHCI {
    unsafe fn channel_number(this: *const Self) -> ULONG {
        (*this).__bindgen_anon_1.Channel
    }
}
macro_rules! IS_PRIMARY_CHANNEL {
    ($ext:expr $(,)?) => { ($crate::prelude::IdeChannelNumber::channel_number(($ext).cast_const()) == 0) };
}
pub(crate) use IS_PRIMARY_CHANNEL;
// amd.c register offsets: C adds in the operand's own (ULONG) type.
macro_rules! AMD_REG_CONFIG_CR { ($io_base:expr $(,)?) => { (($io_base).wrapping_add(0x02)) }; }
macro_rules! AMD_REG_UDMA {
    ($io_base:expr, $channel:expr $(,)?) => {
        (($io_base).wrapping_add(0x10).wrapping_add(2u32.wrapping_sub(($channel as u32).wrapping_mul(2))))
    };
}
pub(crate) use {AMD_REG_CONFIG_CR, AMD_REG_UDMA};

// pciidex.h's PciRead/PciWrite FORCEINLINEs: the C passes `sizeof(x)` to PciRead/PciWrite's
// ULONG BufferLength, an implicit size_t narrowing the generated port left as `usize`.
// drivers/storage/ide/pciidex/pciidex.h:845: header-inline `PciRead8`
#[allow(dead_code)]
#[inline]
pub unsafe fn PciRead8(
    Controller: PATA_CONTROLLER,
    ConfigDataOffset: ULONG,
) -> UCHAR {
    let mut Result: UCHAR = 0;

    unsafe {
        PciRead(
            Controller,
            &mut Result as *mut _ as PVOID,
            ConfigDataOffset,
            core::mem::size_of::<UCHAR>() as ULONG,
        );
    }

    Result
}
// drivers/storage/ide/pciidex/pciidex.h:857: header-inline `PciRead16`
#[allow(dead_code)]
#[inline]
pub unsafe fn PciRead16(
    Controller: PATA_CONTROLLER,
    ConfigDataOffset: ULONG,
) -> USHORT {
    let mut Result: USHORT = 0;

    unsafe {
        PciRead(
            Controller,
            &mut Result as *mut _ as PVOID,
            ConfigDataOffset,
            core::mem::size_of::<USHORT>() as ULONG,
        );
    }

    Result
}
// drivers/storage/ide/pciidex/pciidex.h:869: header-inline `PciRead32`
#[allow(dead_code)]
#[inline]
pub unsafe fn PciRead32(
    Controller: PATA_CONTROLLER,
    ConfigDataOffset: ULONG,
) -> ULONG {
    let mut Result: ULONG = 0;

    unsafe {
        PciRead(
            Controller,
            &mut Result as *mut _ as PVOID,
            ConfigDataOffset,
            core::mem::size_of::<ULONG>() as ULONG,
        );
    }

    Result
}
// drivers/storage/ide/pciidex/pciidex.h:881: header-inline `PciWrite8`
#[allow(dead_code)]
#[inline]
pub unsafe fn PciWrite8(
    Controller: PATA_CONTROLLER,
    ConfigDataOffset: ULONG,
    Value: UCHAR,
) {
    let mut Value = Value;

    unsafe {
        PciWrite(
            Controller,
            &mut Value as *mut _ as PVOID,
            ConfigDataOffset,
            core::mem::size_of::<UCHAR>() as ULONG,
        );
    }
}
// drivers/storage/ide/pciidex/pciidex.h:891: header-inline `PciWrite16`
#[allow(dead_code)]
#[inline]
pub unsafe fn PciWrite16(
    Controller: PATA_CONTROLLER,
    ConfigDataOffset: ULONG,
    Value: USHORT,
) {
    let mut Value = Value;

    unsafe {
        PciWrite(
            Controller,
            &mut Value as *mut _ as PVOID,
            ConfigDataOffset,
            core::mem::size_of::<USHORT>() as ULONG,
        );
    }
}
// drivers/storage/ide/pciidex/pciidex.h:901: header-inline `PciWrite32`
#[allow(dead_code)]
#[inline]
pub unsafe fn PciWrite32(
    Controller: PATA_CONTROLLER,
    ConfigDataOffset: ULONG,
    Value: ULONG,
) {
    let mut Value = Value;

    unsafe {
        PciWrite(
            Controller,
            &mut Value as *mut _ as PVOID,
            ConfigDataOffset,
            core::mem::size_of::<ULONG>() as ULONG,
        );
    }
}

// ==== port macros: the module's own C macros (generated -- `rustconv port macros`) ====
// drivers/storage/ide/pciidex/ahci.h:527: header-inline `AHCI_HBA_READ`
#[allow(dead_code)]
#[inline]
pub unsafe fn AHCI_HBA_READ(
    HbaIoBase: PVOID,
    Register: AHCI_HOST_BUS_ADAPTER_REGISTER,
) -> ULONG {
    READ_REGISTER_ULONG(
        ((HbaIoBase as ULONG_PTR).wrapping_add(Register as ULONG_PTR)) as PULONG
    )
}
// drivers/storage/ide/pciidex/ahci.h:536: header-inline `AHCI_HBA_WRITE`
#[allow(dead_code)]
#[inline]
pub unsafe fn AHCI_HBA_WRITE(
    HbaIoBase: PVOID,
    Register: AHCI_HOST_BUS_ADAPTER_REGISTER,
    Value: ULONG,
) {
    WRITE_REGISTER_ULONG(
        ((HbaIoBase as ULONG_PTR).wrapping_add(Register as ULONG_PTR)) as PULONG,
        Value,
    )
}
// drivers/storage/ide/pciidex/ahci.h:524: #define AHCI_PORT_BASE(HbaIoBase, PortNumber) (PULONG)((ULONG_PTR)(HbaIoBase) + (PortNumber) * 0x80 + 0x100)
#[allow(unused_macros)]
macro_rules! AHCI_PORT_BASE {
    ($hba_io_base:expr, $port_number:expr $(,)?) => {
        (((($hba_io_base as ULONG_PTR).wrapping_add(($port_number as ULONG_PTR).wrapping_mul(0x80))).wrapping_add(0x100)) as PULONG)
    };
}
#[allow(unused_imports)]
pub(crate) use AHCI_PORT_BASE;
// drivers/storage/ide/pciidex/ahci.h:546: header-inline `AHCI_PORT_READ`
#[allow(dead_code)]
#[inline]
pub unsafe fn AHCI_PORT_READ(
    PortIoBase: PVOID,
    Register: AHCI_PORT_REGISTER,
) -> ULONG {
    READ_REGISTER_ULONG(
        ((PortIoBase as ULONG_PTR).wrapping_add(Register as ULONG_PTR)) as PULONG
    )
}
// drivers/storage/ide/pciidex/ahci.h:555: header-inline `AHCI_PORT_WRITE`
#[allow(dead_code)]
#[inline]
pub unsafe fn AHCI_PORT_WRITE(
    PortIoBase: PVOID,
    Register: AHCI_PORT_REGISTER,
    Value: ULONG,
) {
    WRITE_REGISTER_ULONG(
        ((PortIoBase as ULONG_PTR).wrapping_add(Register as ULONG_PTR)) as PULONG,
        Value,
    )
}
// drivers/storage/ide/pciidex/ahci.h:183: #define AHCI_PXCMD_CCS(Value) (((Value) & AHCI_PXCMD_CCS_MASK) >> AHCI_PXCMD_CCS_SHIFT)
#[allow(unused_macros)]
macro_rules! AHCI_PXCMD_CCS {
    ($value:expr $(,)?) => {
        ((($value) & AHCI_PXCMD_CCS_MASK) >> AHCI_PXCMD_CCS_SHIFT)
    };
}
#[allow(unused_imports)]
pub(crate) use AHCI_PXCMD_CCS;
// drivers/storage/ide/pciidex/chipset/amd.c:65: #define AMD_CONFIG_PREFETCH(Channel) (0xC0 >> ((Channel) * 2))
#[allow(unused_macros)]
macro_rules! AMD_CONFIG_PREFETCH {
    ($channel:expr $(,)?) => {
        (0xC0u32 >> (($channel as u32) * 2))
    };
}
#[allow(unused_imports)]
pub(crate) use AMD_CONFIG_PREFETCH;
// drivers/storage/ide/pciidex/chipset/amd.c:62: #define AMD_REG_ADDRESS_SETUP(IoBase) ((IoBase) + 0x0C)
#[allow(unused_macros)]
macro_rules! AMD_REG_ADDRESS_SETUP {
    ($io_base:expr $(,)?) => {
        (($io_base).wrapping_add(0x0C))
    };
}
#[allow(unused_imports)]
pub(crate) use AMD_REG_ADDRESS_SETUP;
// drivers/storage/ide/pciidex/chipset/amd.c:59: #define AMD_REG_CONFIG_PREFETCH(IoBase) ((IoBase) + 0x01)
#[allow(unused_macros)]
macro_rules! AMD_REG_CONFIG_PREFETCH {
    ($io_base:expr $(,)?) => {
        (($io_base).wrapping_add(0x01))
    };
}
#[allow(unused_imports)]
pub(crate) use AMD_REG_CONFIG_PREFETCH;
// drivers/storage/ide/pciidex/chipset/amd.c:61: #define AMD_REG_TIMING_CTRL(IoBase) ((IoBase) + 0x08)
#[allow(unused_macros)]
macro_rules! AMD_REG_TIMING_CTRL {
    ($io_base:expr $(,)?) => {
        (($io_base).wrapping_add(0x08))
    };
}
#[allow(unused_imports)]
pub(crate) use AMD_REG_TIMING_CTRL;
// drivers/storage/ide/pciidex/chipset/amd.c:68: #define AMD_UDMA_TIME(x) (x)
#[allow(unused_macros)]
macro_rules! AMD_UDMA_TIME {
    ($x:expr $(,)?) => {
        ($x)
    };
}
#[allow(unused_imports)]
pub(crate) use AMD_UDMA_TIME;
// drivers/storage/ide/pciidex/pciidex.h:34: #define ASSUME(cond) do { ASSERT(cond); __assume(cond); } while (0)
#[allow(unused_macros)]
macro_rules! ASSUME {
    ($cond:expr $(,)?) => {
        {
            ASSERT!($cond);
            __assume($cond);
        }
    };
}
#[allow(unused_imports)]
pub(crate) use ASSUME;
// drivers/storage/ide/pciidex/pata.h:32: #define ATA_IO_WAIT() KeStallExecutionProcessor(1)
#[allow(unused_macros)]
macro_rules! ATA_IO_WAIT {
    () => {
        KeStallExecutionProcessor(1)
    };
}
#[allow(unused_imports)]
pub(crate) use ATA_IO_WAIT;
// drivers/storage/ide/pciidex/pata.h:260: #define ATA_READ(Port, Ctx, MmioFlag) AtaReadPortUchar(Port, (Ctx)->ChanInfo & CHANNEL_FLAG_##MmioFlag)
#[allow(unused_macros)]
macro_rules! ATA_READ {
    ($port:expr, $ctx:expr, $mmio_flag:expr $(,)?) => {
        AtaReadPortUchar($port, (*$ctx).__bindgen_anon_1.ChanInfo & $mmio_flag)
    };
}
#[allow(unused_imports)]
pub(crate) use ATA_READ;
// drivers/storage/ide/pciidex/pata.h:248: #define ATA_READ_BLOCK_16(Port, Buffer, Count, Ctx, MmioFlag) AtaReadBlock16(Port, Buffer, Count, (Ctx)->ChanInfo & CHANNEL_FLAG_##MmioFlag)
#[allow(unused_macros)]
macro_rules! ATA_READ_BLOCK_16 {
    ($port:expr, $buffer:expr, $count:expr, $ctx:expr, $mmio_flag:expr $(,)?) => {
        AtaReadBlock16($port, $buffer, $count, (*$ctx).__bindgen_anon_1.ChanInfo & $mmio_flag)
    };
}
#[allow(unused_imports)]
pub(crate) use ATA_READ_BLOCK_16;
// drivers/storage/ide/pciidex/pata.h:254: #define ATA_READ_BLOCK_32(Port, Buffer, Count, Ctx, MmioFlag) AtaReadBlock32(Port, Buffer, Count, (Ctx)->ChanInfo & CHANNEL_FLAG_##MmioFlag)
#[allow(unused_macros)]
macro_rules! ATA_READ_BLOCK_32 {
    ($port:expr, $buffer:expr, $count:expr, $ctx:expr, $mmio_flag:expr $(,)?) => {
        AtaReadBlock32($port, $buffer, $count, (*$ctx).__bindgen_anon_1.ChanInfo & $mmio_flag)
    };
}
#[allow(unused_imports)]
pub(crate) use ATA_READ_BLOCK_32;
// drivers/storage/ide/pciidex/pata.h:269: header-inline `ATA_SELECT_DEVICE`
#[allow(dead_code)]
#[inline]
pub unsafe fn ATA_SELECT_DEVICE(
    ChanData: PCHANNEL_DATA_PATA,
    DeviceNumber: UCHAR,
    DeviceSelect: UCHAR,
) {
    #[cfg(target_arch = "x86")]
    {
        if ((*ChanData).__bindgen_anon_1.ChanInfo & CHANNEL_FLAG_CBUS) != 0 &&
           ((*ChanData).LastAtaBankId != DeviceNumber)
        {
            let mut AtaBank: UCHAR;

            (*ChanData).LastAtaBankId = DeviceNumber;

            AtaBank = DeviceNumber >> 1;

            if ((*ChanData).__bindgen_anon_1.ChanInfo & CHANNEL_FLAG_IO32) != 0
            {
                AtaBank |= PC98_ATA_BANK_32BIT_PORT as UCHAR;
            }

            WRITE_PORT_UCHAR(PC98_ATA_BANK as PUCHAR, AtaBank);
        }
    }

    ATA_WRITE!((*ChanData).Regs.Device, DeviceSelect, ChanData, CHANNEL_FLAG_MRES_TF);
    ATA_IO_WAIT!();
}
// drivers/storage/ide/pciidex/pata.h:299: header-inline `ATA_WAIT`
#[allow(dead_code)]
#[inline]
pub unsafe fn ATA_WAIT(
    ChanData: PCHANNEL_DATA_PATA,
    Timeout: ULONG,
    Mask: UCHAR,
    Value: UCHAR,
) -> UCHAR {
    let mut IdeStatus: UCHAR = 0;
    let mut i: ULONG = 0;

    while i < Timeout {
        IdeStatus = unsafe { ((*ChanData).ReadStatus.unwrap_unchecked())(ChanData) };
        if (IdeStatus & Mask) == Value {
            break;
        }
        if IdeStatus == 0xFF {
            break;
        }
        KeStallExecutionProcessor(10);
        i = i.wrapping_add(1);
    }

    IdeStatus
}
// drivers/storage/ide/pciidex/pata.h:263: #define ATA_WRITE(Port, Value, Ctx, MmioFlag) AtaWritePortUchar(Port, Value, (Ctx)->ChanInfo & CHANNEL_FLAG_##MmioFlag)
#[allow(unused_macros)]
macro_rules! ATA_WRITE {
    ($port:expr, $value:expr, $ctx:expr, $mmio_flag:expr $(,)?) => {
        AtaWritePortUchar($port, $value, (*$ctx).__bindgen_anon_1.ChanInfo & $mmio_flag)
    };
}
#[allow(unused_imports)]
pub(crate) use ATA_WRITE;
// drivers/storage/ide/pciidex/pata.h:251: #define ATA_WRITE_BLOCK_16(Port, Buffer, Count, Ctx, MmioFlag) AtaWriteBlock16(Port, Buffer, Count, (Ctx)->ChanInfo & CHANNEL_FLAG_##MmioFlag)
#[allow(unused_macros)]
macro_rules! ATA_WRITE_BLOCK_16 {
    ($port:expr, $buffer:expr, $count:expr, $ctx:expr, $mmio_flag:expr $(,)?) => {
        AtaWriteBlock16($port, $buffer, $count, (*$ctx).__bindgen_anon_1.ChanInfo & $mmio_flag)
    };
}
#[allow(unused_imports)]
pub(crate) use ATA_WRITE_BLOCK_16;
// drivers/storage/ide/pciidex/pata.h:257: #define ATA_WRITE_BLOCK_32(Port, Buffer, Count, Ctx, MmioFlag) AtaWriteBlock32(Port, Buffer, Count, (Ctx)->ChanInfo & CHANNEL_FLAG_##MmioFlag)
#[allow(unused_macros)]
macro_rules! ATA_WRITE_BLOCK_32 {
    ($port:expr, $buffer:expr, $count:expr, $ctx:expr, $mmio_flag:expr $(,)?) => {
        AtaWriteBlock32($port, $buffer, $count, (*$ctx).__bindgen_anon_1.ChanInfo & $mmio_flag)
    };
}
#[allow(unused_imports)]
pub(crate) use ATA_WRITE_BLOCK_32;
// drivers/storage/ide/pciidex/pata.h:266: #define ATA_WRITE_ULONG(Port, Value, Ctx, MmioFlag) AtaWritePortUlong(Port, Value, (Ctx)->ChanInfo & CHANNEL_FLAG_##MmioFlag)
#[allow(unused_macros)]
macro_rules! ATA_WRITE_ULONG {
    ($port:expr, $value:expr, $ctx:expr, $mmio_flag:expr $(,)?) => {
        AtaWritePortUlong($port, $value, (*$ctx).__bindgen_anon_1.ChanInfo & $mmio_flag)
    };
}
#[allow(unused_imports)]
pub(crate) use ATA_WRITE_ULONG;
// drivers/storage/ide/pciidex/pata.h:234: header-inline `CLAMP_TIMING`
#[allow(dead_code)]
#[inline]
pub unsafe fn CLAMP_TIMING(
    Value: ATATIM,
    Minimum: ATATIM,
    Maximum: ATATIM,
) -> ATATIM {
    if Value < Minimum {
        return Minimum;
    }
    if Value > Maximum {
        return Maximum;
    }
    Value
}
// drivers/storage/ide/pciidex/chipset/cmd.c:48: #define CMD_CNTRL_CHAN_EN(Channel) (0x04 << (Channel))
#[allow(unused_macros)]
macro_rules! CMD_CNTRL_CHAN_EN {
    ($channel:expr $(,)?) => {
        (0x04u32 << ($channel as u32))
    };
}
#[allow(unused_imports)]
pub(crate) use CMD_CNTRL_CHAN_EN;
// drivers/storage/ide/pciidex/chipset/cmd.c:44: #define CMD_REG_UDIDETCR(Channel) (((Channel) == 0) ? 0x73 : 0x7B)
#[allow(unused_macros)]
macro_rules! CMD_REG_UDIDETCR {
    ($channel:expr $(,)?) => {
        (if ($channel) == 0 { 0x73u32 } else { 0x7Bu32 })
    };
}
#[allow(unused_imports)]
pub(crate) use CMD_REG_UDIDETCR;
// drivers/storage/ide/pciidex/chipset/cmd.c:59: #define CMD_UDIDETCR_CLEAR(Drive) (((Drive) == 0) ? 0x35 : 0xCA)
#[allow(unused_macros)]
macro_rules! CMD_UDIDETCR_CLEAR {
    ($drive:expr $(,)?) => {
        (if ($drive) == 0 { 0x35u32 } else { 0xCAu32 })
    };
}
#[allow(unused_imports)]
pub(crate) use CMD_UDIDETCR_CLEAR;
// drivers/storage/ide/pciidex/chipset/cmd.c:60: #define CMD_UDIDETCR_EN(Drive) (0x01 << (Drive))
#[allow(unused_macros)]
macro_rules! CMD_UDIDETCR_EN {
    ($drive:expr $(,)?) => {
        (0x01u32 << ($drive as u32))
    };
}
#[allow(unused_imports)]
pub(crate) use CMD_UDIDETCR_EN;
// drivers/storage/ide/pciidex/pciidex.h:825: header-inline `CountSetBits`
#[allow(dead_code)]
#[inline]
pub unsafe fn CountSetBits(x: ULONG) -> ULONG {
    let mut x = x;

    x = x.wrapping_sub((x >> 1) & 0x55555555u32);
    x = (x & 0x33333333u32) + ((x >> 2) & 0x33333333u32);

    (((x + (x >> 4)) & 0x0F0F0F0Fu32).wrapping_mul(0x01010101u32)) >> 24
}
// drivers/storage/ide/pciidex/pciidex.h:56: #define DEV_NUMBER(Device) ((Device)->TransportFlags & DEVICE_NUMBER_MASK)
#[allow(unused_macros)]
macro_rules! DEV_NUMBER {
    ($device:expr $(,)?) => {
        ((*$device).TransportFlags & DEVICE_NUMBER_MASK)
    };
}
#[allow(unused_imports)]
pub(crate) use DEV_NUMBER;
// drivers/storage/ide/pciidex/pciidex.h:40: #define IS_FDO(p) ((((PCOMMON_DEVICE_EXTENSION)(p))->Flags & DO_IS_FDO) != 0)
#[allow(unused_macros)]
macro_rules! IS_FDO {
    ($p:expr $(,)?) => {
        (((*(($p) as PCOMMON_DEVICE_EXTENSION)).Flags & DO_IS_FDO) != 0)
    };
}
#[allow(unused_imports)]
pub(crate) use IS_FDO;
// drivers/storage/ide/pciidex/pciidex.h:836: header-inline `IsPowerOfTwo`
#[allow(dead_code)]
#[inline]
pub unsafe fn IsPowerOfTwo(x: ULONG) -> BOOLEAN {
    (x != 0 && ((x & (x.wrapping_sub(1))) == 0)) as BOOLEAN
}
// drivers/storage/ide/pciidex/pata.h:45: #define MWDMA_MODES(MinMode, MaxMode) (NUM_TO_BITMAP(MWDMA_MODE((MaxMode) + 1)) & ~NUM_TO_BITMAP(MWDMA_MODE(MinMode)))
#[allow(unused_macros)]
macro_rules! MWDMA_MODES {
    ($min_mode:expr, $max_mode:expr $(,)?) => {
        (NUM_TO_BITMAP!(MWDMA_MODE!(($max_mode) + 1)) & (!NUM_TO_BITMAP!(MWDMA_MODE!($min_mode))))
    };
}
#[allow(unused_imports)]
pub(crate) use MWDMA_MODES;
// drivers/storage/ide/pciidex/pata.h:34: #define NUM_TO_BITMAP(num) (0xFFFFFFFF >> (RTL_BITS_OF(ULONG) - (num)))
#[allow(unused_macros)]
macro_rules! NUM_TO_BITMAP {
    ($num:expr $(,)?) => {
        (0xFFFFFFFFu32 >> ((RTL_BITS_OF!(ULONG) as u32).wrapping_sub($num as u32)))
    };
}
#[allow(unused_imports)]
pub(crate) use NUM_TO_BITMAP;
// drivers/storage/ide/pciidex/chipset/intel.c:163: #define PIIX_REG_IDETIM(Channel) (0x40 + ((Channel) * 2))
#[allow(unused_macros)]
macro_rules! PIIX_REG_IDETIM {
    ($Channel:expr $(,)?) => {
        (0x40 + (($Channel) * 2))
    };
}
#[allow(unused_imports)]
pub(crate) use PIIX_REG_IDETIM;
// drivers/storage/ide/pciidex/chipset/intel.c:206: #define PIIX_REG_UDMATIM(Channel) (0x4A + (Channel))
#[allow(unused_macros)]
macro_rules! PIIX_REG_UDMATIM {
    ($Channel:expr $(,)?) => {
        (0x4A + ($Channel))
    };
}
#[allow(unused_imports)]
pub(crate) use PIIX_REG_UDMATIM;
// drivers/storage/ide/pciidex/chipset/intel.c:187: #define PIIX_SIDETIM_ISPRCT_MASK(Channel) (0x0F << ((Channel) * 4))
#[allow(unused_macros)]
macro_rules! PIIX_SIDETIM_ISPRCT_MASK {
    ($channel:expr $(,)?) => {
        (0x0Fu32 << (($channel as u32) * 4))
    };
}
#[allow(unused_imports)]
pub(crate) use PIIX_SIDETIM_ISPRCT_MASK;
// drivers/storage/ide/pciidex/chipset/intel.c:196: #define PIIX_UDMACTL_EN_MASK(Channel) (0x03 << ((Channel) * 2))
#[allow(unused_macros)]
macro_rules! PIIX_UDMACTL_EN_MASK {
    ($channel:expr $(,)?) => {
        (0x03u32 << (($channel as u32) * 2))
    };
}
#[allow(unused_imports)]
pub(crate) use PIIX_UDMACTL_EN_MASK;
// drivers/storage/ide/pciidex/chipset/intel.c:241: #define SCH_DTIM_MDM(Mode) ((Mode) << 8)
#[allow(unused_macros)]
macro_rules! SCH_DTIM_MDM {
    ($mode:expr $(,)?) => {
        (($mode as u32) << 8)
    };
}
#[allow(unused_imports)]
pub(crate) use SCH_DTIM_MDM;
// drivers/storage/ide/pciidex/chipset/intel.c:240: #define SCH_DTIM_PM(Mode) (Mode)
#[allow(unused_macros)]
macro_rules! SCH_DTIM_PM {
    ($mode:expr $(,)?) => {
        ($mode)
    };
}
#[allow(unused_imports)]
pub(crate) use SCH_DTIM_PM;
// drivers/storage/ide/pciidex/chipset/intel.c:242: #define SCH_DTIM_UDM(Mode) ((Mode) << 16)
#[allow(unused_macros)]
macro_rules! SCH_DTIM_UDM {
    ($mode:expr $(,)?) => {
        (($mode as u32) << 16)
    };
}
#[allow(unused_imports)]
pub(crate) use SCH_DTIM_UDM;
// drivers/storage/ide/pciidex/chipset/intel.c:232: #define SCH_REG_DTIM(Drive) (0x80 + (Drive) * 4)
#[allow(unused_macros)]
macro_rules! SCH_REG_DTIM {
    ($drive:expr $(,)?) => {
        (0x80u32 + ($drive as u32) * 4)
    };
}
#[allow(unused_imports)]
pub(crate) use SCH_REG_DTIM;
// drivers/storage/ide/pciidex/chipset/sil680.c:18: #define SIL_REG_CFG(Channel) (0xA0 + ((Channel) << 4))
#[allow(unused_macros)]
macro_rules! SIL_REG_CFG {
    ($channel:expr $(,)?) => {
        (0xA0u32 + (($channel as u32) << 4))
    };
}
#[allow(unused_imports)]
pub(crate) use SIL_REG_CFG;
// drivers/storage/ide/pciidex/chipset/sil680.c:22: #define SIL_REG_DMA_TIMING(Channel, Drive) (0xA8 + ((Channel) << 4) + ((Drive) << 1))
#[allow(unused_macros)]
macro_rules! SIL_REG_DMA_TIMING {
    ($channel:expr, $drive:expr $(,)?) => {
        (0xA8u32 + (($channel as u32) << 4) + (($drive as u32) << 1))
    };
}
#[allow(unused_imports)]
pub(crate) use SIL_REG_DMA_TIMING;
// drivers/storage/ide/pciidex/chipset/sil680.c:21: #define SIL_REG_PIO_TIMING(Channel, Drive) (0xA4 + ((Channel) << 4) + ((Drive) << 1))
#[allow(unused_macros)]
macro_rules! SIL_REG_PIO_TIMING {
    ($channel:expr, $drive:expr $(,)?) => {
        (0xA4u32 + (($channel as u32) << 4) + (($drive as u32) << 1))
    };
}
#[allow(unused_imports)]
pub(crate) use SIL_REG_PIO_TIMING;
// drivers/storage/ide/pciidex/chipset/sil680.c:19: #define SIL_REG_STATUS(Channel) (0xA1 + ((Channel) << 4))
#[allow(unused_macros)]
macro_rules! SIL_REG_STATUS {
    ($channel:expr $(,)?) => {
        (0xA1u32 + (($channel as u32) << 4))
    };
}
#[allow(unused_imports)]
pub(crate) use SIL_REG_STATUS;
// drivers/storage/ide/pciidex/chipset/sil680.c:23: #define SIL_REG_UDMA_TIMING(Channel, Drive) (0xAC + ((Channel) << 4) + ((Drive) << 1))
#[allow(unused_macros)]
macro_rules! SIL_REG_UDMA_TIMING {
    ($channel:expr, $drive:expr $(,)?) => {
        (0xACu32 + (($channel as u32) << 4) + (($drive as u32) << 1))
    };
}
#[allow(unused_imports)]
pub(crate) use SIL_REG_UDMA_TIMING;
// drivers/storage/ide/pciidex/chipset/sil680.c:16: #define SIL_REG_XFER_MODE(Channel) (0x80 + ((Channel) << 2))
#[allow(unused_macros)]
macro_rules! SIL_REG_XFER_MODE {
    ($channel:expr $(,)?) => {
        (0x80u32 + (($channel as u32) << 2))
    };
}
#[allow(unused_imports)]
pub(crate) use SIL_REG_XFER_MODE;
// drivers/storage/ide/pciidex/pata.h:49: #define UDMA_MODES(MinMode, MaxMode) (NUM_TO_BITMAP(UDMA_MODE((MaxMode) + 1)) & ~NUM_TO_BITMAP(UDMA_MODE(MinMode)))
#[allow(unused_macros)]
macro_rules! UDMA_MODES {
    ($min_mode:expr, $max_mode:expr $(,)?) => {
        (NUM_TO_BITMAP!(UDMA_MODE!(($max_mode) + 1)) & (!NUM_TO_BITMAP!(UDMA_MODE!($min_mode))))
    };
}
#[allow(unused_imports)]
pub(crate) use UDMA_MODES;
// drivers/storage/ide/pciidex/chipset/via.c:75: #define VIA_REG_PORT_TIMING(Channel) (0x4F - (Channel))
#[allow(unused_macros)]
macro_rules! VIA_REG_PORT_TIMING {
    ($channel:expr $(,)?) => {
        (0x4Fu32 - ($channel as u32))
    };
}
#[allow(unused_imports)]
pub(crate) use VIA_REG_PORT_TIMING;
// drivers/storage/ide/pciidex/chipset/via.c:93: #define VIA_SINGLE_CHAN_UDMA_CABLE_PRESENT(Controller) ((PciRead16(Controller, 0x50) & 0x1010) != 0)
#[allow(unused_macros)]
macro_rules! VIA_SINGLE_CHAN_UDMA_CABLE_PRESENT {
    ($controller:expr $(,)?) => {
        ((unsafe { PciRead16($controller, 0x50) } & 0x1010) != 0)
    };
}
#[allow(unused_imports)]
pub(crate) use VIA_SINGLE_CHAN_UDMA_CABLE_PRESENT;
// drivers/storage/ide/pciidex/chipset/via.c:90: #define VIA_UDMA_CABLE_PRESENT(Reg32, Channel) (((Reg32) & (0x10100000 >> ((Channel) * 16))) != 0)
#[allow(unused_macros)]
macro_rules! VIA_UDMA_CABLE_PRESENT {
    ($reg32:expr, $channel:expr $(,)?) => {
        ((($reg32) & (0x10100000u32 >> (($channel as u32) * 16))) != 0)
    };
}
#[allow(unused_imports)]
pub(crate) use VIA_UDMA_CABLE_PRESENT;
// drivers/storage/ide/pciidex/chipset/via.c:87: #define VIA_UDMA_CLOCK_ENABLED(Reg32, Channel) (((Reg32) & (0x00080000 >> ((Channel) * 16))) != 0)
#[allow(unused_macros)]
macro_rules! VIA_UDMA_CLOCK_ENABLED {
    ($reg32:expr, $channel:expr $(,)?) => {
        ((($reg32) & (0x00080000u32 >> (($channel as u32) * 16))) != 0)
    };
}
#[allow(unused_imports)]
pub(crate) use VIA_UDMA_CLOCK_ENABLED;
// drivers/storage/ide/pciidex/chipset/via.c:103: #define VT6421A_REG_TIMING_CTRL(Drive) (0xAB - (Drive))
#[allow(unused_macros)]
macro_rules! VT6421A_REG_TIMING_CTRL {
    ($drive:expr $(,)?) => {
        (0xABu32 - ($drive as u32))
    };
}
#[allow(unused_imports)]
pub(crate) use VT6421A_REG_TIMING_CTRL;
// drivers/storage/ide/pciidex/chipset/via.c:104: #define VT6421A_REG_UDMA_CTRL(Drive) (0xB3 - (Drive))
#[allow(unused_macros)]
macro_rules! VT6421A_REG_UDMA_CTRL {
    ($drive:expr $(,)?) => {
        (0xB3u32 - ($drive as u32))
    };
}
#[allow(unused_imports)]
pub(crate) use VT6421A_REG_UDMA_CTRL;
// drivers/storage/ide/pciidex/chipset/via.c:97: #define VT6421A_UDMA_CABLE_PRESENT(Controller) ((PciRead16(Controller, 0xB2) & 0x1010) == 0)
#[allow(unused_macros)]
macro_rules! VT6421A_UDMA_CABLE_PRESENT {
    ($controller:expr $(,)?) => {
        ((unsafe { PciRead16($controller, 0xB2) } & 0x1010) == 0)
    };
}
#[allow(unused_imports)]
pub(crate) use VT6421A_UDMA_CABLE_PRESENT;
// ==== end port macros ====

// ==== port resolve: substrate helpers (generated) ====
pub use ntkrnl_sys::mm::ALIGN_UP_BY;
pub use ntkrnl_sys::inline::common::BYTES_TO_PAGES;
pub use ntkrnl_sys::driver_ddk::DbgPrint;
pub use ntkrnl_sys::inline::mm::ExAllocatePoolUninitialized;
pub use ntkrnl_sys::inline::mm::ExAllocatePoolZero;
pub use ntkrnl_sys::ex::ExFreePoolWithTag;
pub use ntkrnl_sys::inline::ex::ExInitializeFastMutex;
pub use ntkrnl_sys::inline::common::HalGetInterruptTranslator;
pub use ntkrnl_sys::inline::common::InitializeListHead;
pub use ntkrnl_sys::inline::common::InsertTailList;
pub use ntkrnl_sys::ddk::IoAcquireRemoveLockEx;
pub use ntkrnl_sys::inline::common::IoAdjustPagingPathCount;
pub use ntkrnl_sys::driver_ddk::IoCallDriver;
pub use ntkrnl_sys::driver_ddk::IoCompleteRequest;
pub use ntkrnl_sys::inline::common::IoInitializeRemoveLock;
pub use ntkrnl_sys::ddk::IoReleaseRemoveLockAndWaitEx;
pub use ntkrnl_sys::ddk::IoReleaseRemoveLockEx;
pub use ntkrnl_sys::inline::common::IsEqualGUIDAligned;
pub use ntkrnl_sys::inline::common::KeFlushIoBuffers;
pub use ntkrnl_sys::ke::KeGetCurrentIrql;
pub use ntkrnl_sys::ke::KeSetEvent;
pub use ntkrnl_sys::ddk::KeStallExecutionProcessor;
pub use ntkrnl_sys::ob::ObDereferenceObject;
pub use ntkrnl_sys::inline::tier1::ObReferenceObject;
pub use ntkrnl_sys::inline::common::PCI_MULTIFUNCTION_DEVICE;
pub use ntkrnl_sys::portio::READ_PORT_BUFFER_ULONG;
pub use ntkrnl_sys::portio::READ_PORT_BUFFER_USHORT;
pub use ntkrnl_sys::portio::READ_PORT_UCHAR;
pub use ntkrnl_sys::portio::READ_PORT_ULONG;
pub use ntkrnl_sys::portio::READ_REGISTER_BUFFER_ULONG;
pub use ntkrnl_sys::portio::READ_REGISTER_BUFFER_USHORT;
pub use ntkrnl_sys::portio::READ_REGISTER_UCHAR;
pub use ntkrnl_sys::portio::READ_REGISTER_ULONG;
pub use ntkrnl_sys::portio::READ_REGISTER_USHORT;
pub use ntkrnl_sys::inline::common::RemoveEntryList;
pub use ntkrnl_sys::inline::common::RtlULongAdd;
pub use ntkrnl_sys::portio::WRITE_PORT_BUFFER_ULONG;
pub use ntkrnl_sys::portio::WRITE_PORT_BUFFER_USHORT;
pub use ntkrnl_sys::portio::WRITE_PORT_UCHAR;
pub use ntkrnl_sys::portio::WRITE_PORT_ULONG;
pub use ntkrnl_sys::portio::WRITE_REGISTER_BUFFER_ULONG;
pub use ntkrnl_sys::portio::WRITE_REGISTER_BUFFER_USHORT;
pub use ntkrnl_sys::portio::WRITE_REGISTER_UCHAR;
pub use ntkrnl_sys::portio::WRITE_REGISTER_ULONG;
pub use ntkrnl_sys::portio::WRITE_REGISTER_USHORT;
// ==== end substrate helpers ====
