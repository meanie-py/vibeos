//! Byte-exact substrate + win32 aliases for the RTL thread-pool cluster.
//!
//! ntkrnl-sys is kernel-oriented; the Wine thread-pool code is user-mode
//! flavored, so the win32 scalar aliases (`BOOL`/`DWORD`/`HMODULE`/…),
//! a few access-mask constants, the `WT_*` flags and the PUBLIC winnt
//! `TP_*` types are mirrored here. The opaque `TP_*` handle types are
//! only ever used as pointers (cast to the file-private structs), so a
//! zero-size opaque struct is sufficient.

use ntkrnl_sys::base::*;          // NTSTATUS, PVOID, ULONG, LONG, BOOLEAN, ULONG_PTR, SIZE_T …
use ntkrnl_sys::ddk::HANDLE;
use ntkrnl_sys::ddk::{PTEB, _TEB, PTHREAD_START_ROUTINE, PHANDLE, RTL_CRITICAL_SECTION};

// ---- win32 scalar aliases not present in ntkrnl-sys ----
pub type BOOL = i32;
pub type DWORD = u32;
pub type INT = i32;
pub type UINT = u32;
pub type LPVOID = PVOID;
pub type HMODULE = PVOID;

// ---- access masks / sentinels (exact C values) ----
// STANDARD_RIGHTS_REQUIRED (0x000F0000) | SYNCHRONIZE (0x00100000) | 0x3
pub const IO_COMPLETION_ALL_ACCESS: ULONG = 0x001F_0003;
pub const EVENT_ALL_ACCESS: ULONG = 0x001F_0003;
pub const INFINITE: ULONG = 0xFFFF_FFFF;
pub const HEAP_ZERO_MEMORY: ULONG = 0x0000_0008;
pub const MAXIMUM_WAIT_OBJECTS: LONG = 64;
pub const MAXLONGLONG: i64 = 0x7FFF_FFFF_FFFF_FFFF;
pub const WAIT_OBJECT_0: ULONG = 0;
pub const WAIT_TIMEOUT: ULONG = 0x0000_0102;
pub const STATUS_TOO_MANY_THREADS: NTSTATUS = 0xC000_0129u32 as i32;

// C `#define CRITICAL_SECTION RTL_CRITICAL_SECTION`
pub type CRITICAL_SECTION = RTL_CRITICAL_SECTION;

// InterlockedExchange (returns OLD value) — not in ntkrnl-sys interlocked.
#[inline(always)]
#[allow(non_snake_case)]
pub unsafe fn InterlockedExchange(target: *mut LONG, value: LONG) -> LONG {
    (*(target as *const core::sync::atomic::AtomicI32))
        .swap(value, core::sync::atomic::Ordering::SeqCst)
}

// ntdll loader refcount hooks for the callback environment's RaceDll
// (NTAPI; absent from the ntkrnl-sys bindgen surface).
extern "system" {
    pub fn LdrAddRefDll(Flags: ULONG, BaseAddress: PVOID) -> NTSTATUS;
    pub fn LdrUnloadDll(BaseAddress: PVOID) -> NTSTATUS;
}
// STATUS_WAIT_0 and INVALID_HANDLE_VALUE come from ntkrnl_sys (constants/
// globals) — do not redefine here (would be ambiguous with the globs).

// ---- WT_* thread-pool flags (winnt) ----
pub const WT_EXECUTEDEFAULT: ULONG = 0x0000_0000;
pub const WT_EXECUTEINIOTHREAD: ULONG = 0x0000_0001;
pub const WT_EXECUTEINUITHREAD: ULONG = 0x0000_0002;
pub const WT_EXECUTEINWAITTHREAD: ULONG = 0x0000_0004;
pub const WT_EXECUTEONLYONCE: ULONG = 0x0000_0008;
pub const WT_EXECUTELONGFUNCTION: ULONG = 0x0000_0010;
pub const WT_EXECUTEINTIMERTHREAD: ULONG = 0x0000_0020;
pub const WT_EXECUTEINPERSISTENTIOTHREAD: ULONG = 0x0000_0040;
pub const WT_EXECUTEINPERSISTENTTHREAD: ULONG = 0x0000_0080;
pub const WT_TRANSFER_IMPERSONATION: ULONG = 0x0000_0100;

// ---- public opaque TP handle types (winnt_old.h) ----
macro_rules! opaque {
    ($name:ident, $p:ident) => {
        #[repr(C)]
        pub struct $name {
            _opaque: [u8; 0],
        }
        pub type $p = *mut $name;
    };
}
opaque!(TP_POOL, PTP_POOL);
opaque!(TP_WORK, PTP_WORK);
opaque!(TP_TIMER, PTP_TIMER);
opaque!(TP_WAIT, PTP_WAIT);
opaque!(TP_IO, PTP_IO);
opaque!(TP_CLEANUP_GROUP, PTP_CLEANUP_GROUP);
opaque!(TP_CALLBACK_INSTANCE, PTP_CALLBACK_INSTANCE);

pub type TP_WAIT_RESULT = DWORD;
pub type TP_VERSION = DWORD;
pub type PTP_VERSION = *mut DWORD;

// enum _TP_CALLBACK_PRIORITY (C `int`)
pub type TP_CALLBACK_PRIORITY = i32;
pub const TP_CALLBACK_PRIORITY_HIGH: TP_CALLBACK_PRIORITY = 0;
pub const TP_CALLBACK_PRIORITY_NORMAL: TP_CALLBACK_PRIORITY = 1;
pub const TP_CALLBACK_PRIORITY_LOW: TP_CALLBACK_PRIORITY = 2;
pub const TP_CALLBACK_PRIORITY_INVALID: TP_CALLBACK_PRIORITY = 3;
pub const TP_CALLBACK_PRIORITY_COUNT: TP_CALLBACK_PRIORITY = 3;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct TP_POOL_STACK_INFORMATION {
    pub StackReserve: SIZE_T,
    pub StackCommit: SIZE_T,
}
pub type PTP_POOL_STACK_INFORMATION = *mut TP_POOL_STACK_INFORMATION;
const _: () = assert!(core::mem::size_of::<TP_POOL_STACK_INFORMATION>() == 2 * core::mem::size_of::<usize>());

// ---- TP callback fn-ptr typedefs (NTAPI = extern "system") ----
pub type PTP_SIMPLE_CALLBACK =
    Option<unsafe extern "system" fn(Instance: PTP_CALLBACK_INSTANCE, Context: PVOID)>;
pub type PTP_WORK_CALLBACK =
    Option<unsafe extern "system" fn(Instance: PTP_CALLBACK_INSTANCE, Context: PVOID, Work: PTP_WORK)>;
pub type PTP_TIMER_CALLBACK =
    Option<unsafe extern "system" fn(Instance: PTP_CALLBACK_INSTANCE, Context: PVOID, Timer: PTP_TIMER)>;
pub type PTP_WAIT_CALLBACK = Option<
    unsafe extern "system" fn(
        Instance: PTP_CALLBACK_INSTANCE,
        Context: PVOID,
        Wait: PTP_WAIT,
        WaitResult: TP_WAIT_RESULT,
    ),
>;
pub type PTP_CLEANUP_GROUP_CANCEL_CALLBACK =
    Option<unsafe extern "system" fn(ObjectContext: PVOID, CleanupContext: PVOID)>;
// void (CALLBACK *)(PTP_CALLBACK_INSTANCE,void*,void*,IO_STATUS_BLOCK*,PTP_IO)
pub type PTP_IO_CALLBACK = Option<
    unsafe extern "system" fn(
        Instance: PTP_CALLBACK_INSTANCE,
        Context: PVOID,
        ApcContext: PVOID,
        IoSB: *mut core::ffi::c_void,
        Io: PTP_IO,
    ),
>;
// VOID (CALLBACK *PRTL_OVERLAPPED_COMPLETION_ROUTINE)(DWORD,DWORD,LPVOID)
pub type PRTL_OVERLAPPED_COMPLETION_ROUTINE =
    Option<unsafe extern "system" fn(DWORD, DWORD, LPVOID)>;

// The old-API worker / wait-or-timer callbacks. These cross the C↔Rust
// boundary (RtlQueueWorkItem is called from C; RtlCreateTimer's Callback
// comes from C callers), so they MUST be NTAPI = `extern "system"`
// (stdcall on i386). ntkrnl-sys/ddk's bindgen types are `extern "C"`,
// which is wrong for i386 — define the faithful versions here and use
// these instead of the ddk imports across the cluster.
//   VOID (NTAPI *WORKERCALLBACKFUNC)(PVOID)
pub type WORKERCALLBACKFUNC = Option<unsafe extern "system" fn(Context: PVOID)>;
//   #define PRTL_WORK_ITEM_ROUTINE WORKERCALLBACKFUNC (threadpool.c)
pub type PRTL_WORK_ITEM_ROUTINE = WORKERCALLBACKFUNC;
//   VOID (NTAPI *RTL_WAITORTIMERCALLBACKFUNC)(PVOID, BOOLEAN)
pub type WAITORTIMERCALLBACKFUNC = Option<unsafe extern "system" fn(PVOID, BOOLEAN)>;
pub type RTL_WAITORTIMERCALLBACKFUNC = WAITORTIMERCALLBACKFUNC;

// The pluggable pool-thread start/exit hooks (set via RtlSetThreadPoolStartFunc
// by SMSS/CSRSS). NTAPI = extern "system" (ddk's bindgen makes them extern "C",
// wrong for i386). The first arg keeps ddk's PTHREAD_START_ROUTINE for ABI
// consistency with RtlCreateUserThread.
//   NTSTATUS (NTAPI *PRTL_START_POOL_THREAD)(PTHREAD_START_ROUTINE, PVOID, PHANDLE)
pub type PRTL_START_POOL_THREAD = Option<
    unsafe extern "system" fn(PTHREAD_START_ROUTINE, PVOID, PHANDLE) -> NTSTATUS,
>;
//   NTSTATUS (NTAPI *PRTL_EXIT_POOL_THREAD)(NTSTATUS)
pub type PRTL_EXIT_POOL_THREAD = Option<unsafe extern "system" fn(NTSTATUS) -> NTSTATUS>;

// struct _TP_CALLBACK_ENVIRON_V3 (winnt_old.h). The bitfield union is
// flattened to its `Flags` word (the LongFunction/Persistent/Private
// bits are read via masks, matching bindgen's anon-union treatment).
#[repr(C)]
pub struct TP_CALLBACK_ENVIRON_V3 {
    pub Version: TP_VERSION,
    pub Pool: PTP_POOL,
    pub CleanupGroup: PTP_CLEANUP_GROUP,
    pub CleanupGroupCancelCallback: PTP_CLEANUP_GROUP_CANCEL_CALLBACK,
    pub RaceDll: PVOID,
    pub ActivationContext: PVOID, // struct _ACTIVATION_CONTEXT *
    pub FinalizationCallback: PTP_SIMPLE_CALLBACK,
    pub u: TP_CALLBACK_ENVIRON_V3_u,
    pub CallbackPriority: TP_CALLBACK_PRIORITY,
    pub Size: DWORD,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct TP_CALLBACK_ENVIRON_V3_u_s(pub DWORD);
impl TP_CALLBACK_ENVIRON_V3_u_s {
    // bitfield { DWORD LongFunction:1; Persistent:1; Private:30; } (LE)
    #[inline]
    pub fn LongFunction(&self) -> ULONG {
        self.0 & 0x1
    }
    #[inline]
    pub fn set_LongFunction(&mut self, v: ULONG) {
        self.0 = (self.0 & !0x1) | (v & 0x1);
    }
    #[inline]
    pub fn Persistent(&self) -> ULONG {
        (self.0 >> 1) & 0x1
    }
    #[inline]
    pub fn set_Persistent(&mut self, v: ULONG) {
        self.0 = (self.0 & !0x2) | ((v & 0x1) << 1);
    }
}

#[repr(C)]
pub union TP_CALLBACK_ENVIRON_V3_u {
    pub Flags: DWORD,
    pub s: TP_CALLBACK_ENVIRON_V3_u_s,
}
pub type TP_CALLBACK_ENVIRON = TP_CALLBACK_ENVIRON_V3;
pub type PTP_CALLBACK_ENVIRON = *mut TP_CALLBACK_ENVIRON_V3;

// ---- RtlGetProcessHeap() macro: NtCurrentPeb()->ProcessHeap ----
// Crate-private (NOT #[no_mangle]) — ntdll-rust already exports the real
// `RtlGetProcessHeap` symbol; this is the inline-macro equivalent so the
// translated bodies can call it without a symbol collision.
#[cfg(target_arch = "x86")]
#[inline(always)]
pub unsafe fn NtCurrentTeb() -> PTEB {
    let teb: PTEB;
    core::arch::asm!("mov {}, fs:[0x18]", out(reg) teb, options(nostack, pure, readonly));
    teb
}
#[cfg(target_arch = "x86_64")]
#[inline(always)]
pub unsafe fn NtCurrentTeb() -> PTEB {
    let teb: PTEB;
    core::arch::asm!("mov {}, gs:[0x30]", out(reg) teb, options(nostack, pure, readonly));
    teb
}
#[cfg(target_arch = "aarch64")]
#[inline(always)]
pub unsafe fn NtCurrentTeb() -> PTEB {
    let teb: PTEB;
    core::arch::asm!("mov {}, x18", out(reg) teb, options(nostack, pure, nomem));
    teb
}

// RtlReAllocateHeap is a real exported RTL symbol but absent from the
// ntkrnl-sys bindgen surface — declare it (NTAPI = extern "system").
extern "system" {
    pub fn RtlReAllocateHeap(
        HeapHandle: PVOID,
        Flags: ULONG,
        BaseAddress: PVOID,
        Size: SIZE_T,
    ) -> PVOID;
}

#[inline(always)]
#[allow(non_snake_case)]
pub unsafe fn RtlGetProcessHeap() -> PVOID {
    (*(*NtCurrentTeb()).ProcessEnvironmentBlock).ProcessHeap as PVOID
}
// C `#define GetProcessHeap() RtlGetProcessHeap()`
#[inline(always)]
#[allow(non_snake_case)]
pub unsafe fn GetProcessHeap() -> PVOID {
    RtlGetProcessHeap()
}

// threadpool.c win32 #defines:
//   GetCurrentProcess() => NtCurrentProcess()
//   GetCurrentThread()  => NtCurrentThread()
//   GetCurrentThreadId()=> HandleToULong(NtCurrentTeb()->ClientId.UniqueThread)
#[inline(always)]
#[allow(non_snake_case)]
pub unsafe fn GetCurrentProcess() -> HANDLE {
    ntkrnl_sys::macros::NtCurrentProcess()
}
#[inline(always)]
#[allow(non_snake_case)]
pub unsafe fn GetCurrentThread() -> HANDLE {
    ntkrnl_sys::macros::NtCurrentThread()
}
#[inline(always)]
#[allow(non_snake_case)]
pub unsafe fn GetCurrentThreadId() -> DWORD {
    (*NtCurrentTeb()).ClientId.UniqueThread as usize as DWORD
}

// keep the import live across arches that don't read _TEB by name
const _: () = { let _ = core::mem::size_of::<_TEB>; };
