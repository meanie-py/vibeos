//! The DDK header macros `port macros` refuses, ported once instead of per driver.
//!
//! `port macros` lowers a header macro only when one expansion serves every arch and the
//! body is an expression. These are the ones that fail that test -- the spin-lock family
//! names `KfAcquireSpinLock` (i386 fastcall) or `KeAcquireSpinLockRaiseToDpc`, arm64's
//! `NdisCopyLookaheadData` honours the MAC's copy option, and the buffer-chain macros are
//! statement blocks -- so every NDIS driver met them as unresolved calls. Each macro reaches
//! its argument's fields by NAME, so it works against whichever island the consumer's types
//! come from, and it calls `ntkrnl_sys`'s portable wrappers rather than naming an export.
//! Every macro argument is evaluated once, as the C's single-use spellings are.
#![no_std]
#![allow(non_upper_case_globals)]

// ---- ndis.h: NDIS_SPIN_LOCK is { KSPIN_LOCK SpinLock; KIRQL OldIrql; } -------------------

#[macro_export]
macro_rules! NdisAcquireSpinLock {
    ($lock:expr $(,)?) => {{
        let lock = $lock;
        ntkrnl_sys::ke::KeAcquireSpinLock(
            core::ptr::addr_of_mut!((*lock).SpinLock) as _,
            core::ptr::addr_of_mut!((*lock).OldIrql) as _,
        )
    }};
}

#[macro_export]
macro_rules! NdisReleaseSpinLock {
    ($lock:expr $(,)?) => {{
        let lock = $lock;
        ntkrnl_sys::ke::KeReleaseSpinLock(
            core::ptr::addr_of_mut!((*lock).SpinLock) as _,
            (*lock).OldIrql as _,
        )
    }};
}

#[macro_export]
macro_rules! NdisDprAcquireSpinLock {
    ($lock:expr $(,)?) => {
        ntkrnl_sys::ke::KeAcquireSpinLockAtDpcLevel(core::ptr::addr_of_mut!((*($lock)).SpinLock) as _)
    };
}

#[macro_export]
macro_rules! NdisDprReleaseSpinLock {
    ($lock:expr $(,)?) => {
        ntkrnl_sys::ke::KeReleaseSpinLockFromDpcLevel(
            core::ptr::addr_of_mut!((*($lock)).SpinLock) as _,
        )
    };
}

// ---- ndis.h: buffers are MDLs, chained through `Next`, on a packet's Private.Head/Tail ----

/// `NdisChainBufferAtFront(Packet, Buffer)`: Buffer may itself be a chain; its last link
/// becomes the packet's tail when the packet was empty.
#[macro_export]
macro_rules! NdisChainBufferAtFront {
    ($packet:expr, $buffer:expr $(,)?) => {{
        let packet = $packet;
        let first = $buffer;
        let mut last = first;
        while !(*last).Next.is_null() {
            last = (*last).Next as _;
        }
        if (*packet).Private.Head.is_null() {
            (*packet).Private.Tail = last as _;
        }
        (*last).Next = (*packet).Private.Head as _;
        (*packet).Private.Head = first as _;
        (*packet).Private.ValidCounts = 0 as _;
    }};
}

/// `NdisChainBufferAtBack(Packet, Buffer)`.
#[macro_export]
macro_rules! NdisChainBufferAtBack {
    ($packet:expr, $buffer:expr $(,)?) => {{
        let packet = $packet;
        let first = $buffer;
        let mut last = first;
        while !(*last).Next.is_null() {
            last = (*last).Next as _;
        }
        (*last).Next = core::ptr::null_mut();
        if !(*packet).Private.Head.is_null() {
            (*(*packet).Private.Tail).Next = first as _;
        } else {
            (*packet).Private.Head = first as _;
        }
        (*packet).Private.Tail = last as _;
        (*packet).Private.ValidCounts = 0 as _;
    }};
}

/// `NdisQueryBuffer(Buffer, VirtualAddress OPTIONAL, Length)`.
#[macro_export]
macro_rules! NdisQueryBuffer {
    ($buffer:expr, $va:expr, $length:expr $(,)?) => {{
        let buffer = $buffer;
        let va = $va;
        if !(va as *const core::ffi::c_void).is_null() {
            *(va as *mut *mut core::ffi::c_void) = $crate::MmGetSystemAddressForMdl!(buffer) as _;
        }
        *($length) = (*buffer).ByteCount as _;
    }};
}

/// `NdisGetFirstBufferFromPacketSafe(Packet, &First, &FirstVA, &FirstLength, &TotalLength,
/// Priority)`: the first buffer, its system VA and length, and the whole chain's length; all
/// zero for an empty packet.
#[macro_export]
macro_rules! NdisGetFirstBufferFromPacketSafe {
    ($packet:expr, $first:expr, $first_va:expr, $first_length:expr, $total_length:expr,
     $priority:expr $(,)?) => {{
        let mut buffer = (*($packet)).Private.Head;
        let first_length = $first_length;
        let total_length = $total_length;
        *($first) = buffer as _;
        if !buffer.is_null() {
            *($first_va) = $crate::MmGetSystemAddressForMdlSafe!(buffer, $priority) as _;
            *first_length = (*buffer).ByteCount as _;
            buffer = (*buffer).Next as _;
            *total_length = *first_length as _;
            while !buffer.is_null() {
                *total_length = (*total_length).wrapping_add((*buffer).ByteCount as _);
                buffer = (*buffer).Next as _;
            }
        } else {
            *($first_va) = core::ptr::null_mut();
            *first_length = 0 as _;
            *total_length = 0 as _;
        }
    }};
}

/// `NdisCopyLookaheadData(Destination, Source, Length, MacOptions)`: a plain copy on x86 and
/// amd64; elsewhere a byte-at-a-time copy unless the MAC set NDIS_MAC_OPTION_COPY_LOOKAHEAD_DATA
/// (0x1), because the lookahead may sit in device memory a wide access must not touch. The
/// byte loop is volatile so LLVM cannot fold it back into a wide memcpy.
#[macro_export]
macro_rules! NdisCopyLookaheadData {
    ($dest:expr, $src:expr, $length:expr, $options:expr $(,)?) => {{
        let dest = ($dest) as *mut u8;
        let src = ($src) as *const u8;
        let length = ($length) as usize;
        let options = ($options) as u32;
        if cfg!(any(target_arch = "x86", target_arch = "x86_64")) || options & 0x1 != 0 {
            core::ptr::copy_nonoverlapping(src, dest, length);
        } else {
            let mut i = 0usize;
            while i < length {
                core::ptr::write_volatile(dest.add(i), core::ptr::read_volatile(src.add(i)));
                i += 1;
            }
        }
    }};
}

/// ndis.h: `#define PROTOCOL_RESERVED_SIZE_IN_PACKET (4 * sizeof(PVOID))`.
pub const PROTOCOL_RESERVED_SIZE_IN_PACKET: usize = 4 * core::mem::size_of::<*mut u8>();

/// ndis.h: `#define NdisInterruptLevelSensitive LevelSensitive` / `NdisInterruptLatched
/// Latched`, KINTERRUPT_MODE's enumerators (bindgen's `c_uint` on every arch).
pub const NdisInterruptLevelSensitive: u32 = 0;
pub const NdisInterruptLatched: u32 = 1;

// ---- mmfuncs.h ---------------------------------------------------------------------------

/// `MmGetSystemAddressForMdl(Mdl)`: the mapped VA when the MDL is already mapped to system
/// space or describes nonpaged pool (MDL_MAPPED_TO_SYSTEM_VA 0x1 | MDL_SOURCE_IS_NONPAGED_POOL
/// 0x4), else map it for KernelMode (0).
#[macro_export]
macro_rules! MmGetSystemAddressForMdl {
    ($mdl:expr $(,)?) => {{
        let mdl = $mdl;
        if ((*mdl).MdlFlags as u32) & (0x0001 | 0x0004) != 0 {
            (*mdl).MappedSystemVa as *mut core::ffi::c_void
        } else {
            ntkrnl_sys::ddk::MmMapLockedPages(mdl as _, 0 as _) as *mut core::ffi::c_void
        }
    }};
}

/// `MmGetSystemAddressForMdlSafe(Mdl, Priority)`: as above, but mapping with
/// MmMapLockedPagesSpecifyCache(Mdl, KernelMode, MmCached (1), NULL, FALSE, Priority), which
/// returns NULL instead of bugchecking when the system is out of PTEs.
#[macro_export]
macro_rules! MmGetSystemAddressForMdlSafe {
    ($mdl:expr, $priority:expr $(,)?) => {{
        let mdl = $mdl;
        if ((*mdl).MdlFlags as u32) & (0x0001 | 0x0004) != 0 {
            (*mdl).MappedSystemVa as *mut core::ffi::c_void
        } else {
            ntkrnl_sys::ddk::MmMapLockedPagesSpecifyCache(
                mdl as _,
                0 as _,
                1 as _,
                core::ptr::null_mut(),
                0 as _,
                ($priority) as _,
            ) as *mut core::ffi::c_void
        }
    }};
}

// ---- wdm.h -------------------------------------------------------------------------------

/// `Int32x32To64(a, b)`: `(LONGLONG)(LONG)a * (LONGLONG)(LONG)b`; i386's `__emul` is the same
/// signed 32x32->64 product.
#[macro_export]
macro_rules! Int32x32To64 {
    ($a:expr, $b:expr $(,)?) => {
        ((($a) as i32 as i64) * (($b) as i32 as i64))
    };
}

/// `KeQueryTickCount(PLARGE_INTEGER)`: the 64-bit headers read KUSER_SHARED_DATA.TickCount
/// (0xFFFFF78000000000 + 0x320); i386's inline reads the exported KeTickCount, and ntoskrnl
/// exports the same as `KeQueryTickCount` (stdcall), which this calls.
#[macro_export]
macro_rules! KeQueryTickCount {
    ($count:expr $(,)?) => {{
        let count = ($count) as *mut u64;
        #[cfg(not(target_arch = "x86"))] // nolint: cfg-vs-c (the header's own per-arch arms)
        {
            *count = core::ptr::read_volatile((0xFFFF_F780_0000_0000u64 + 0x320) as usize as *const u64);
        }
        #[cfg(target_arch = "x86")] // nolint: cfg-vs-c (the header's own per-arch arms)
        {
            extern "stdcall" {
                fn KeQueryTickCount(CurrentCount: *mut u64);
            }
            KeQueryTickCount(count);
        }
    }};
}

// ---- ntdef.h -----------------------------------------------------------------------------

/// `RTL_CONSTANT_OBJECT_ATTRIBUTES(n, a)`, a static initialiser: names the consumer's own
/// `OBJECT_ATTRIBUTES`.
#[macro_export]
macro_rules! RTL_CONSTANT_OBJECT_ATTRIBUTES {
    ($name:expr, $attributes:expr $(,)?) => {
        OBJECT_ATTRIBUTES {
            Length: core::mem::size_of::<OBJECT_ATTRIBUTES>() as _,
            RootDirectory: core::ptr::null_mut(),
            ObjectName: ($name) as _,
            Attributes: ($attributes) as _,
            SecurityDescriptor: core::ptr::null_mut(),
            SecurityQualityOfService: core::ptr::null_mut(),
        }
    };
}
