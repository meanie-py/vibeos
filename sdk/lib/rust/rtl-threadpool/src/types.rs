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
use ntkrnl_sys::ddk::{PTEB, _TEB};

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
pub const STATUS_WAIT_0: NTSTATUS = 0;
pub const MAXIMUM_WAIT_OBJECTS: LONG = 64;
pub const INVALID_HANDLE_VALUE: HANDLE = !0usize as HANDLE; // (HANDLE)-1

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
pub union TP_CALLBACK_ENVIRON_V3_u {
    pub Flags: DWORD,
    pub s: DWORD, // bitfield { LongFunction:1; Persistent:1; Private:30; }
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

// keep the import live across arches that don't read _TEB by name
const _: () = { let _ = core::mem::size_of::<_TEB>; };
