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

// ---- tdikrnl.h: the TDI request builders ------------------------------------------------
//
// Each fills the NEXT stack location of an IRP for an IRP_MJ_INTERNAL_DEVICE_CONTROL to a
// transport. `port macros` refuses them as statement blocks. The request structs
// (TDI_REQUEST_KERNEL_*) and minor codes (TDI_*) are named unqualified, so they resolve
// in the consumer's own island, as the NDIS macros' fields do.

/// `TdiBuildBaseIrp`: major/minor, device and file object, and the completion routine --
/// invoked on success, error and cancel when there is one, cleared when there is not.
#[macro_export]
macro_rules! TdiBuildBaseIrp {
    ($irp:expr, $dev:expr, $file:expr, $routine:expr, $context:expr, $sp:expr, $minor:expr $(,)?) => {{
        let (irp, sp) = ($irp, $sp);
        (*sp).MajorFunction = ntkrnl_sys::driver_ddk::IRP_MJ_INTERNAL_DEVICE_CONTROL as _;
        (*sp).MinorFunction = ($minor) as _;
        (*sp).DeviceObject = ($dev) as _;
        (*sp).FileObject = ($file) as _;
        let routine: ntkrnl_sys::ddk::PIO_COMPLETION_ROUTINE = $routine;
        if routine.is_some() {
            ntkrnl_sys::inline::tier1::IoSetCompletionRoutine(irp, routine, ($context) as _, 1, 1, 1);
        } else {
            ntkrnl_sys::inline::tier1::IoSetCompletionRoutine(
                irp,
                None,
                core::ptr::null_mut(),
                0,
                0,
                0,
            );
        }
    }};
}

/// The shared prologue: the next stack location, its base fields, and the request
/// overlaid on its `Parameters`.
#[doc(hidden)]
#[macro_export]
macro_rules! __tdi_request {
    ($irp:expr, $dev:expr, $file:expr, $routine:expr, $context:expr, $minor:expr, $req:ty) => {{
        let irp = $irp;
        let sp = ntkrnl_sys::inline::tier1::IoGetNextIrpStackLocation(irp);
        $crate::TdiBuildBaseIrp!(irp, $dev, $file, $routine, $context, sp, $minor);
        (irp, core::ptr::addr_of_mut!((*sp).Parameters) as *mut $req)
    }};
}

#[macro_export]
macro_rules! TdiBuildAccept {
    ($irp:expr, $dev:expr, $file:expr, $routine:expr, $context:expr, $request_info:expr,
     $return_info:expr $(,)?) => {{
        let (_, r) = $crate::__tdi_request!($irp, $dev, $file, $routine, $context, TDI_ACCEPT,
            TDI_REQUEST_KERNEL_ACCEPT);
        (*r).RequestConnectionInformation = ($request_info) as _;
        (*r).ReturnConnectionInformation = ($return_info) as _;
    }};
}

#[macro_export]
macro_rules! TdiBuildAction {
    ($irp:expr, $dev:expr, $file:expr, $routine:expr, $context:expr, $mdl:expr $(,)?) => {{
        let irp = $irp;
        let sp = ntkrnl_sys::inline::tier1::IoGetNextIrpStackLocation(irp);
        $crate::TdiBuildBaseIrp!(irp, $dev, $file, $routine, $context, sp, TDI_ACTION);
        (*irp).MdlAddress = ($mdl) as _;
    }};
}

#[macro_export]
macro_rules! TdiBuildAssociateAddress {
    ($irp:expr, $dev:expr, $file:expr, $routine:expr, $context:expr, $handle:expr $(,)?) => {{
        let (_, r) = $crate::__tdi_request!($irp, $dev, $file, $routine, $context,
            TDI_ASSOCIATE_ADDRESS, TDI_REQUEST_KERNEL_ASSOCIATE);
        (*r).AddressHandle = ($handle) as _;
    }};
}

#[macro_export]
macro_rules! TdiBuildConnect {
    ($irp:expr, $dev:expr, $file:expr, $routine:expr, $context:expr, $time:expr,
     $request_info:expr, $return_info:expr $(,)?) => {{
        let (_, r) = $crate::__tdi_request!($irp, $dev, $file, $routine, $context, TDI_CONNECT,
            TDI_REQUEST_KERNEL);
        (*r).RequestConnectionInformation = ($request_info) as _;
        (*r).ReturnConnectionInformation = ($return_info) as _;
        (*r).RequestSpecific = ($time) as _;
    }};
}

#[macro_export]
macro_rules! TdiBuildDisassociateAddress {
    ($irp:expr, $dev:expr, $file:expr, $routine:expr, $context:expr $(,)?) => {{
        let irp = $irp;
        let sp = ntkrnl_sys::inline::tier1::IoGetNextIrpStackLocation(irp);
        $crate::TdiBuildBaseIrp!(irp, $dev, $file, $routine, $context, sp,
            TDI_DISASSOCIATE_ADDRESS);
    }};
}

#[macro_export]
macro_rules! TdiBuildDisconnect {
    ($irp:expr, $dev:expr, $file:expr, $routine:expr, $context:expr, $time:expr, $flags:expr,
     $request_info:expr, $return_info:expr $(,)?) => {{
        let (_, r) = $crate::__tdi_request!($irp, $dev, $file, $routine, $context,
            TDI_DISCONNECT, TDI_REQUEST_KERNEL);
        (*r).RequestConnectionInformation = ($request_info) as _;
        (*r).ReturnConnectionInformation = ($return_info) as _;
        (*r).RequestSpecific = ($time) as _;
        (*r).RequestFlags = ($flags) as _;
    }};
}

/// An expression: the IRP, from IoBuildDeviceIoControlRequest with no buffers.
#[macro_export]
macro_rules! TdiBuildInternalDeviceControlIrp {
    ($sub:expr, $dev:expr, $file:expr, $event:expr, $iosb:expr $(,)?) => {{
        let _ = $file; // the C takes it and never uses it
        IoBuildDeviceIoControlRequest(
            ($sub) as _,
            ($dev) as _,
            core::ptr::null_mut(),
            0,
            core::ptr::null_mut(),
            0,
            1,
            ($event) as _,
            ($iosb) as _,
        )
    }};
}

#[macro_export]
macro_rules! TdiBuildListen {
    ($irp:expr, $dev:expr, $file:expr, $routine:expr, $context:expr, $flags:expr,
     $request_info:expr, $return_info:expr $(,)?) => {{
        let (_, r) = $crate::__tdi_request!($irp, $dev, $file, $routine, $context, TDI_LISTEN,
            TDI_REQUEST_KERNEL);
        (*r).RequestConnectionInformation = ($request_info) as _;
        (*r).ReturnConnectionInformation = ($return_info) as _;
        (*r).RequestFlags = ($flags) as _;
    }};
}

#[macro_export]
macro_rules! TdiBuildQueryInformation {
    ($irp:expr, $dev:expr, $file:expr, $routine:expr, $context:expr, $qtype:expr,
     $mdl:expr $(,)?) => {{
        let (irp, r) = $crate::__tdi_request!($irp, $dev, $file, $routine, $context,
            TDI_QUERY_INFORMATION, TDI_REQUEST_KERNEL_QUERY_INFORMATION);
        (*r).RequestConnectionInformation = core::ptr::null_mut();
        (*r).QueryType = ($qtype) as _;
        (*irp).MdlAddress = ($mdl) as _;
    }};
}

#[macro_export]
macro_rules! TdiBuildReceive {
    ($irp:expr, $dev:expr, $file:expr, $routine:expr, $context:expr, $mdl:expr, $flags:expr,
     $len:expr $(,)?) => {{
        let (irp, r) = $crate::__tdi_request!($irp, $dev, $file, $routine, $context,
            TDI_RECEIVE, TDI_REQUEST_KERNEL_RECEIVE);
        (*r).ReceiveFlags = ($flags) as _;
        (*r).ReceiveLength = ($len) as _;
        (*irp).MdlAddress = ($mdl) as _;
    }};
}

#[macro_export]
macro_rules! TdiBuildReceiveDatagram {
    ($irp:expr, $dev:expr, $file:expr, $routine:expr, $context:expr, $mdl:expr, $len:expr,
     $receive_info:expr, $return_info:expr, $flags:expr $(,)?) => {{
        let (irp, r) = $crate::__tdi_request!($irp, $dev, $file, $routine, $context,
            TDI_RECEIVE_DATAGRAM, TDI_REQUEST_KERNEL_RECEIVEDG);
        (*r).ReceiveDatagramInformation = ($receive_info) as _;
        (*r).ReturnDatagramInformation = ($return_info) as _;
        (*r).ReceiveLength = ($len) as _;
        (*r).ReceiveFlags = ($flags) as _;
        (*irp).MdlAddress = ($mdl) as _;
    }};
}

#[macro_export]
macro_rules! TdiBuildSend {
    ($irp:expr, $dev:expr, $file:expr, $routine:expr, $context:expr, $mdl:expr, $flags:expr,
     $len:expr $(,)?) => {{
        let (irp, r) = $crate::__tdi_request!($irp, $dev, $file, $routine, $context, TDI_SEND,
            TDI_REQUEST_KERNEL_SEND);
        (*r).SendFlags = ($flags) as _;
        (*r).SendLength = ($len) as _;
        (*irp).MdlAddress = ($mdl) as _;
    }};
}

#[macro_export]
macro_rules! TdiBuildSendDatagram {
    ($irp:expr, $dev:expr, $file:expr, $routine:expr, $context:expr, $mdl:expr, $len:expr,
     $send_info:expr $(,)?) => {{
        let (irp, r) = $crate::__tdi_request!($irp, $dev, $file, $routine, $context,
            TDI_SEND_DATAGRAM, TDI_REQUEST_KERNEL_SENDDG);
        (*r).SendDatagramInformation = ($send_info) as _;
        (*r).SendLength = ($len) as _;
        (*irp).MdlAddress = ($mdl) as _;
    }};
}

#[macro_export]
macro_rules! TdiBuildSetEventHandler {
    ($irp:expr, $dev:expr, $file:expr, $routine:expr, $context:expr, $event_type:expr,
     $handler:expr, $event_context:expr $(,)?) => {{
        let (_, r) = $crate::__tdi_request!($irp, $dev, $file, $routine, $context,
            TDI_SET_EVENT_HANDLER, TDI_REQUEST_KERNEL_SET_EVENT);
        (*r).EventType = ($event_type) as _;
        (*r).EventHandler = ($handler) as _;
        (*r).EventContext = ($event_context) as _;
    }};
}

#[macro_export]
macro_rules! TdiBuildSetInformation {
    ($irp:expr, $dev:expr, $file:expr, $routine:expr, $context:expr, $stype:expr,
     $mdl:expr $(,)?) => {{
        let (irp, r) = $crate::__tdi_request!($irp, $dev, $file, $routine, $context,
            TDI_SET_INFORMATION, TDI_REQUEST_KERNEL_SET_INFORMATION);
        (*r).RequestConnectionInformation = core::ptr::null_mut();
        (*r).SetType = ($stype) as _;
        (*irp).MdlAddress = ($mdl) as _;
    }};
}
