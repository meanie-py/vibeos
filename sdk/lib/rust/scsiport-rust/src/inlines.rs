//! inlines.rs — FORCEINLINE / header-only / intrinsic helpers the LLM rendered as calls.
//! The READ/WRITE_REGISTER_* DDK memory-mapped-register intrinsics (volatile loads/
//! stores), the VerifyIrp*BufferSize scsiport.h FORCEINLINEs, KeRaiseIrql (wraps the
//! exported KfRaiseIrql), MmGetMdlVirtualAddress, and SENSE_BUFFER_SIZE. None are
//! ntoskrnl exports themselves (they're header macros/inlines).
#![allow(non_snake_case, non_upper_case_globals, dead_code, non_camel_case_types)]
use ntkrnl_sys::ddk::{ULONG, KIRQL, PIRP, PVOID};

// FORCEINLINE IRP-stack walker (Tail.Overlay arithmetic) — re-export the tier1 inline.
pub use ntkrnl_sys::inline::tier1::{IoGetCurrentIrpStackLocation, IoGetNextIrpStackLocation,
    IoSkipCurrentIrpStackLocation, IoCopyCurrentIrpStackLocationToNext, IoMarkIrpPending,
    IoSetCompletionRoutine};

// wdm.h FORCEINLINE IoRequestDpc(DevObj, Irp, Context) = KeInsertQueueDpc(&DevObj->Dpc, Irp, Context).
#[inline(always)]
pub unsafe fn IoRequestDpc(
    DeviceObject: ntkrnl_sys::ddk::PDEVICE_OBJECT,
    Irp: ntkrnl_sys::ddk::PIRP,
    Context: PVOID,
) {
    ntkrnl_sys::ddk::KeInsertQueueDpc((&raw mut (*DeviceObject).Dpc).cast(), Irp as PVOID, Context);
}

// ── DDK memory-mapped register I/O (wdm.h READ/WRITE_REGISTER_*) — volatile load/store.
// scsiport's ScsiPortReadRegister*/WriteRegister* export wrappers bottom out here.
#[inline(always)] pub unsafe fn READ_REGISTER_UCHAR(r: *mut u8) -> u8 { core::ptr::read_volatile(r) }
#[inline(always)] pub unsafe fn READ_REGISTER_USHORT(r: *mut u16) -> u16 { core::ptr::read_volatile(r) }
#[inline(always)] pub unsafe fn READ_REGISTER_ULONG(r: *mut u32) -> u32 { core::ptr::read_volatile(r) }
#[inline(always)] pub unsafe fn WRITE_REGISTER_UCHAR(r: *mut u8, v: u8) { core::ptr::write_volatile(r, v) }
#[inline(always)] pub unsafe fn WRITE_REGISTER_USHORT(r: *mut u16, v: u16) { core::ptr::write_volatile(r, v) }
#[inline(always)] pub unsafe fn WRITE_REGISTER_ULONG(r: *mut u32, v: u32) { core::ptr::write_volatile(r, v) }

#[inline(always)] pub unsafe fn READ_REGISTER_BUFFER_UCHAR(r: *mut u8, b: *mut u8, n: ULONG) {
    for i in 0..n as isize { *b.offset(i) = core::ptr::read_volatile(r.offset(i)); }
}
#[inline(always)] pub unsafe fn READ_REGISTER_BUFFER_USHORT(r: *mut u16, b: *mut u16, n: ULONG) {
    for i in 0..n as isize { *b.offset(i) = core::ptr::read_volatile(r.offset(i)); }
}
#[inline(always)] pub unsafe fn READ_REGISTER_BUFFER_ULONG(r: *mut u32, b: *mut u32, n: ULONG) {
    for i in 0..n as isize { *b.offset(i) = core::ptr::read_volatile(r.offset(i)); }
}
#[inline(always)] pub unsafe fn WRITE_REGISTER_BUFFER_UCHAR(r: *mut u8, b: *mut u8, n: ULONG) {
    for i in 0..n as isize { core::ptr::write_volatile(r.offset(i), *b.offset(i)); }
}
#[inline(always)] pub unsafe fn WRITE_REGISTER_BUFFER_USHORT(r: *mut u16, b: *mut u16, n: ULONG) {
    for i in 0..n as isize { core::ptr::write_volatile(r.offset(i), *b.offset(i)); }
}
#[inline(always)] pub unsafe fn WRITE_REGISTER_BUFFER_ULONG(r: *mut u32, b: *mut u32, n: ULONG) {
    for i in 0..n as isize { core::ptr::write_volatile(r.offset(i), *b.offset(i)); }
}

// KeRaiseIrql(NewIrql, OldIrql*) macro → *OldIrql = KfRaiseIrql(NewIrql). KfRaiseIrql is
// the real ntoskrnl export (fastcall on i386, cdecl on amd64/arm64).
#[cfg(target_arch = "x86")]
extern "fastcall" { fn KfRaiseIrql(NewIrql: KIRQL) -> KIRQL; }
// amd64: IRQL IS cr8 — the DDK KfRaiseIrql/KeLowerIrql are FORCEINLINE cr8 accessors,
// not exports.
#[cfg(target_arch = "x86_64")]
#[inline(always)]
unsafe fn KfRaiseIrql(NewIrql: KIRQL) -> KIRQL {
    let old: u64;
    core::arch::asm!("mov {}, cr8", out(reg) old, options(nomem, nostack));
    let new = NewIrql as u64;
    core::arch::asm!("mov cr8, {}", in(reg) new, options(nomem, nostack));
    old as KIRQL
}
#[cfg(target_arch = "aarch64")]
extern "C" { fn KfRaiseIrql(NewIrql: KIRQL) -> KIRQL; }
#[inline(always)]
pub unsafe fn KeRaiseIrql(NewIrql: KIRQL, OldIrql: *mut KIRQL) {
    *OldIrql = KfRaiseIrql(NewIrql);
}

// KeLowerIrql — fastcall KfLowerIrql on i386; a plain export elsewhere (the amd64 ddk
// bindgen has it, but the i386/arm64 ones don't — provide it here for all arches).
#[cfg(target_arch = "x86")]
extern "fastcall" { fn KfLowerIrql(NewIrql: KIRQL); }
#[cfg(target_arch = "x86")]
#[inline(always)]
pub unsafe fn KeLowerIrql(NewIrql: KIRQL) { KfLowerIrql(NewIrql) }
#[cfg(target_arch = "x86_64")]
#[inline(always)]
pub unsafe fn KeLowerIrql(NewIrql: KIRQL) {
    let new = NewIrql as u64;
    core::arch::asm!("mov cr8, {}", in(reg) new, options(nomem, nostack));
}
#[cfg(target_arch = "aarch64")]
mod lower_irql { unsafe extern "C" { pub fn KeLowerIrql(NewIrql: super::KIRQL); } }
#[cfg(target_arch = "aarch64")]
pub use lower_irql::KeLowerIrql;

// wdm.h: MmGetMdlVirtualAddress(Mdl) = (PVOID)((PCHAR)Mdl->StartVa + Mdl->ByteOffset).
#[inline(always)]
pub unsafe fn MmGetMdlVirtualAddress(Mdl: *mut ntkrnl_sys::ddk::_MDL) -> PVOID {
    ((*Mdl).StartVa as *mut u8).add((*Mdl).ByteOffset as usize) as PVOID
}

// scsiport.h FORCEINLINE VerifyIrpOutBufferSize / VerifyIrpInBufferSize.
#[inline(always)]
pub unsafe fn VerifyIrpOutBufferSize(Irp: PIRP, Size: usize) -> u8 {
    let io_stack = IoGetCurrentIrpStackLocation(Irp);
    if ((*io_stack).Parameters.DeviceIoControl.OutputBufferLength as usize) < Size {
        (*Irp).IoStatus.Information = Size as _;
        return 0;
    }
    1
}
#[inline(always)]
pub unsafe fn VerifyIrpInBufferSize(Irp: PIRP, Size: usize) -> u8 {
    let io_stack = IoGetCurrentIrpStackLocation(Irp);
    if ((*io_stack).Parameters.DeviceIoControl.InputBufferLength as usize) < Size {
        (*Irp).IoStatus.Information = Size as _;
        return 0;
    }
    1
}

// SENSE_BUFFER_SIZE = sizeof(SENSE_DATA) — fixed 18 bytes by the SCSI spec.
pub const SENSE_BUFFER_SIZE: usize = 18;

// mmfuncs.h: MmGetSystemAddressForMdl(Mdl) — MappedSystemVa if already mapped or the
// source is nonpaged pool, else MmMapLockedPages(Mdl, KernelMode).
const MDL_MAPPED_TO_SYSTEM_VA: u16 = 0x0001;
const MDL_SOURCE_IS_NONPAGED_POOL: u16 = 0x0004;
#[inline(always)]
pub unsafe fn MmGetSystemAddressForMdl(Mdl: *mut ntkrnl_sys::ddk::_MDL) -> PVOID {
    if (*Mdl).MdlFlags as u16 & (MDL_MAPPED_TO_SYSTEM_VA | MDL_SOURCE_IS_NONPAGED_POOL) != 0 {
        (*Mdl).MappedSystemVa
    } else {
        ntkrnl_sys::ddk::MmMapLockedPages(Mdl, 0 /* KernelMode */)
    }
}

// xdk/iofuncs.h FORCEINLINE IoAllocateAdapterChannel — dispatch through the DMA_ADAPTER
// operations table (not a kernel export on this SDK surface).
#[inline(always)]
pub unsafe fn IoAllocateAdapterChannel(
    DmaAdapter: ntkrnl_sys::ddk::PDMA_ADAPTER,
    DeviceObject: ntkrnl_sys::ddk::PDEVICE_OBJECT,
    NumberOfMapRegisters: ULONG,
    ExecutionRoutine: ntkrnl_sys::ddk::PDRIVER_CONTROL,
    Context: PVOID,
) -> ntkrnl_sys::ddk::NTSTATUS {
    let alloc = (*(*DmaAdapter).DmaOperations).AllocateAdapterChannel.unwrap();
    alloc(DmaAdapter, DeviceObject, NumberOfMapRegisters, ExecutionRoutine, Context)
}

// ── scsitypes.h FORCEINLINE device-type name tables (PnP ID strings) ──
use crate::types::{INQUIRYDATA, DIRECT_ACCESS_DEVICE, SEQUENTIAL_ACCESS_DEVICE,
    PRINTER_DEVICE, PROCESSOR_DEVICE, WRITE_ONCE_READ_MULTIPLE_DEVICE,
    READ_ONLY_DIRECT_ACCESS_DEVICE, SCANNER_DEVICE, OPTICAL_DEVICE, MEDIUM_CHANGER,
    COMMUNICATION_DEVICE, ARRAY_CONTROLLER_DEVICE, SCSI_ENCLOSURE_DEVICE,
    REDUCED_BLOCK_DEVICE, OPTICAL_CARD_READER_WRITER_DEVICE, BRIDGE_CONTROLLER_DEVICE};

pub unsafe fn GetDeviceType(InquiryData: *mut INQUIRYDATA) -> *const core::ffi::c_char {
    (match (*InquiryData).DeviceType() as u32 {
        DIRECT_ACCESS_DEVICE => b"Disk\0".as_ptr(),
        SEQUENTIAL_ACCESS_DEVICE => b"Sequential\0".as_ptr(),
        PRINTER_DEVICE => b"Printer\0".as_ptr(),
        PROCESSOR_DEVICE => b"Processor\0".as_ptr(),
        WRITE_ONCE_READ_MULTIPLE_DEVICE => b"Worm\0".as_ptr(),
        READ_ONLY_DIRECT_ACCESS_DEVICE => b"CdRom\0".as_ptr(),
        SCANNER_DEVICE => b"Scanner\0".as_ptr(),
        OPTICAL_DEVICE => b"Optical\0".as_ptr(),
        MEDIUM_CHANGER => b"Changer\0".as_ptr(),
        COMMUNICATION_DEVICE => b"Net\0".as_ptr(),
        ARRAY_CONTROLLER_DEVICE => b"Array\0".as_ptr(),
        SCSI_ENCLOSURE_DEVICE => b"Enclosure\0".as_ptr(),
        REDUCED_BLOCK_DEVICE => b"RBC\0".as_ptr(),
        OPTICAL_CARD_READER_WRITER_DEVICE => b"CardReader\0".as_ptr(),
        BRIDGE_CONTROLLER_DEVICE => b"Bridge\0".as_ptr(),
        _ => b"Other\0".as_ptr(),
    }) as *const core::ffi::c_char
}

pub unsafe fn GetGenericType(InquiryData: *mut INQUIRYDATA) -> *const core::ffi::c_char {
    (match (*InquiryData).DeviceType() as u32 {
        DIRECT_ACCESS_DEVICE => b"GenDisk\0".as_ptr(),
        PRINTER_DEVICE => b"GenPrinter\0".as_ptr(),
        WRITE_ONCE_READ_MULTIPLE_DEVICE => b"GenWorm\0".as_ptr(),
        READ_ONLY_DIRECT_ACCESS_DEVICE => b"GenCdRom\0".as_ptr(),
        SCANNER_DEVICE => b"GenScanner\0".as_ptr(),
        OPTICAL_DEVICE => b"GenOptical\0".as_ptr(),
        MEDIUM_CHANGER => b"ScsiChanger\0".as_ptr(),
        COMMUNICATION_DEVICE => b"ScsiNet\0".as_ptr(),
        ARRAY_CONTROLLER_DEVICE => b"ScsiArray\0".as_ptr(),
        SCSI_ENCLOSURE_DEVICE => b"ScsiEnclosure\0".as_ptr(),
        REDUCED_BLOCK_DEVICE => b"ScsiRBC\0".as_ptr(),
        OPTICAL_CARD_READER_WRITER_DEVICE => b"ScsiCardReader\0".as_ptr(),
        BRIDGE_CONTROLLER_DEVICE => b"ScsiBridge\0".as_ptr(),
        _ => b"ScsiOther\0".as_ptr(),
    }) as *const core::ffi::c_char
}

pub unsafe fn GetPeripheralTypeW(InquiryData: *mut INQUIRYDATA) -> *const u16 {
    macro_rules! w {
        ($s:literal) => {{
            const N: usize = $s.len() + 1;
            static W: [u16; N] = {
                let mut a = [0u16; N];
                let mut i = 0;
                while i < $s.len() { a[i] = $s.as_bytes()[i] as u16; i += 1; }
                a
            };
            W.as_ptr()
        }};
    }
    match (*InquiryData).DeviceType() as u32 {
        DIRECT_ACCESS_DEVICE => w!("DiskPeripheral"),
        SEQUENTIAL_ACCESS_DEVICE => w!("TapePeripheral"),
        PRINTER_DEVICE => w!("PrinterPeripheral"),
        WRITE_ONCE_READ_MULTIPLE_DEVICE => w!("WormPeripheral"),
        READ_ONLY_DIRECT_ACCESS_DEVICE => w!("CdRomPeripheral"),
        SCANNER_DEVICE => w!("ScannerPeripheral"),
        OPTICAL_DEVICE => w!("OpticalDiskPeripheral"),
        MEDIUM_CHANGER => w!("MediumChangerPeripheral"),
        COMMUNICATION_DEVICE => w!("CommunicationsPeripheral"),
        ARRAY_CONTROLLER_DEVICE => w!("ArrayPeripheral"),
        SCSI_ENCLOSURE_DEVICE => w!("EnclosurePeripheral"),
        REDUCED_BLOCK_DEVICE => w!("RBCPeripheral"),
        OPTICAL_CARD_READER_WRITER_DEVICE => w!("CardReaderPeripheral"),
        BRIDGE_CONTROLLER_DEVICE => w!("BridgePeripheral"),
        _ => w!("OtherPeripheral"),
    }
}

// ── wdm FORCEINLINEs / macros / intrinsics the LLM left as extern calls ──
// ExAllocatePoolZero = ExAllocatePoolWithTag + RtlZeroMemory (wdm.h FORCEINLINE).
#[inline(always)]
pub unsafe fn ExAllocatePoolZero(PoolType: ntkrnl_sys::ddk::POOL_TYPE, NumberOfBytes: u32, Tag: u32) -> PVOID {
    let p = ntkrnl_sys::ddk::ExAllocatePoolWithTag(PoolType, NumberOfBytes as ntkrnl_sys::ddk::SIZE_T, Tag);
    if !p.is_null() { core::ptr::write_bytes(p as *mut u8, 0, NumberOfBytes as usize); }
    p
}
// KeInitializeSpinLock — FORCEINLINE: *SpinLock = 0.
#[inline(always)]
pub unsafe fn KeInitializeSpinLock(SpinLock: *mut usize) { core::ptr::write_volatile(SpinLock, 0); }
// IoInitializeDpcRequest — FORCEINLINE: KeInitializeDpc(&DevObj->Dpc, DpcRoutine, DevObj).
#[inline(always)]
pub unsafe fn IoInitializeDpcRequest(DeviceObject: ntkrnl_sys::ddk::PDEVICE_OBJECT, DpcRoutine: PVOID) {
    ntkrnl_sys::ddk::KeInitializeDpc(
        (&raw mut (*DeviceObject).Dpc).cast(),
        Some(core::mem::transmute(DpcRoutine)),
        DeviceObject as PVOID,
    );
}
// KeAcquireSpinLockRaiseToDpc — i386: the spinlock entry points live in HAL as
// fastcall KfAcquireSpinLock; elsewhere a plain ntoskrnl export.
#[cfg(target_arch = "x86")]
extern "fastcall" { fn KfAcquireSpinLock(SpinLock: *mut usize) -> KIRQL; }
#[cfg(target_arch = "x86")]
#[inline(always)]
pub unsafe fn KeAcquireSpinLockRaiseToDpc(SpinLock: *mut usize) -> KIRQL { KfAcquireSpinLock(SpinLock) }
#[cfg(not(target_arch = "x86"))]
mod acq_dpc { unsafe extern "C" { pub fn KeAcquireSpinLockRaiseToDpc(SpinLock: *mut usize) -> super::KIRQL; } }
#[cfg(not(target_arch = "x86"))]
pub use acq_dpc::KeAcquireSpinLockRaiseToDpc;
// __debugbreak intrinsic.
#[inline(always)]
pub unsafe fn __debugbreak() {
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    core::arch::asm!("int3");
    #[cfg(target_arch = "aarch64")]
    core::arch::asm!("brk #0xf000");
}
// ALIGN_UP / ALIGN_UP_POINTER (umtypes/wdm macros) — callers use INT64 (8) alignment.
#[inline(always)]
pub unsafe fn ALIGN_UP(value: u32, alignment: u32) -> u32 {
    (value + alignment - 1) & !(alignment - 1)
}
#[inline(always)]
pub unsafe fn ALIGN_UP_POINTER(ptr: *mut core::ffi::c_void, alignment: u32) -> *mut core::ffi::c_void {
    (((ptr as usize) + alignment as usize - 1) & !(alignment as usize - 1)) as *mut core::ffi::c_void
}

// ── PORT I/O (READ/WRITE_PORT_*) ──
// i386: real HAL stdcall exports. amd64: DDK FORCEINLINE in/out instructions — NOT
// exports — so inline asm here. arm64: MMIO function exports from hal (extern "C").
#[cfg(target_arch = "x86")]
mod port_io {
    unsafe extern "system" {
        pub fn READ_PORT_UCHAR(Port: *mut u8) -> u8;
        pub fn READ_PORT_USHORT(Port: *mut u16) -> u16;
        pub fn READ_PORT_ULONG(Port: *mut u32) -> u32;
        pub fn READ_PORT_BUFFER_UCHAR(Port: *mut u8, Buffer: *mut u8, Count: u32);
        pub fn READ_PORT_BUFFER_USHORT(Port: *mut u16, Buffer: *mut u16, Count: u32);
        pub fn READ_PORT_BUFFER_ULONG(Port: *mut u32, Buffer: *mut u32, Count: u32);
        pub fn WRITE_PORT_UCHAR(Port: *mut u8, Value: u8);
        pub fn WRITE_PORT_USHORT(Port: *mut u16, Value: u16);
        pub fn WRITE_PORT_ULONG(Port: *mut u32, Value: u32);
        pub fn WRITE_PORT_BUFFER_UCHAR(Port: *mut u8, Buffer: *const u8, Count: u32);
        pub fn WRITE_PORT_BUFFER_USHORT(Port: *mut u16, Buffer: *const u16, Count: u32);
        pub fn WRITE_PORT_BUFFER_ULONG(Port: *mut u32, Buffer: *const u32, Count: u32);
    }
}
#[cfg(target_arch = "aarch64")]
mod port_io {
    unsafe extern "system" {
        pub fn READ_PORT_UCHAR(Port: *mut u8) -> u8;
        pub fn READ_PORT_USHORT(Port: *mut u16) -> u16;
        pub fn READ_PORT_ULONG(Port: *mut u32) -> u32;
        pub fn READ_PORT_BUFFER_UCHAR(Port: *mut u8, Buffer: *mut u8, Count: u32);
        pub fn READ_PORT_BUFFER_USHORT(Port: *mut u16, Buffer: *mut u16, Count: u32);
        pub fn READ_PORT_BUFFER_ULONG(Port: *mut u32, Buffer: *mut u32, Count: u32);
        pub fn WRITE_PORT_UCHAR(Port: *mut u8, Value: u8);
        pub fn WRITE_PORT_USHORT(Port: *mut u16, Value: u16);
        pub fn WRITE_PORT_ULONG(Port: *mut u32, Value: u32);
        pub fn WRITE_PORT_BUFFER_UCHAR(Port: *mut u8, Buffer: *const u8, Count: u32);
        pub fn WRITE_PORT_BUFFER_USHORT(Port: *mut u16, Buffer: *const u16, Count: u32);
        pub fn WRITE_PORT_BUFFER_ULONG(Port: *mut u32, Buffer: *const u32, Count: u32);
    }
}
#[cfg(target_arch = "x86_64")]
mod port_io {
    #[inline(always)]
    pub unsafe fn READ_PORT_UCHAR(Port: *mut u8) -> u8 {
        let v: u8;
        core::arch::asm!("in al, dx", in("dx") Port as u16, out("al") v, options(nomem, nostack));
        v
    }
    #[inline(always)]
    pub unsafe fn READ_PORT_USHORT(Port: *mut u16) -> u16 {
        let v: u16;
        core::arch::asm!("in ax, dx", in("dx") Port as u16, out("ax") v, options(nomem, nostack));
        v
    }
    #[inline(always)]
    pub unsafe fn READ_PORT_ULONG(Port: *mut u32) -> u32 {
        let v: u32;
        core::arch::asm!("in eax, dx", in("dx") Port as u16, out("eax") v, options(nomem, nostack));
        v
    }
    #[inline(always)]
    pub unsafe fn READ_PORT_BUFFER_UCHAR(Port: *mut u8, Buffer: *mut u8, Count: u32) {
        for i in 0..Count as isize { *Buffer.offset(i) = READ_PORT_UCHAR(Port); }
    }
    #[inline(always)]
    pub unsafe fn READ_PORT_BUFFER_USHORT(Port: *mut u16, Buffer: *mut u16, Count: u32) {
        for i in 0..Count as isize { *Buffer.offset(i) = READ_PORT_USHORT(Port); }
    }
    #[inline(always)]
    pub unsafe fn READ_PORT_BUFFER_ULONG(Port: *mut u32, Buffer: *mut u32, Count: u32) {
        for i in 0..Count as isize { *Buffer.offset(i) = READ_PORT_ULONG(Port); }
    }
    #[inline(always)]
    pub unsafe fn WRITE_PORT_UCHAR(Port: *mut u8, Value: u8) {
        core::arch::asm!("out dx, al", in("dx") Port as u16, in("al") Value, options(nomem, nostack));
    }
    #[inline(always)]
    pub unsafe fn WRITE_PORT_USHORT(Port: *mut u16, Value: u16) {
        core::arch::asm!("out dx, ax", in("dx") Port as u16, in("ax") Value, options(nomem, nostack));
    }
    #[inline(always)]
    pub unsafe fn WRITE_PORT_ULONG(Port: *mut u32, Value: u32) {
        core::arch::asm!("out dx, eax", in("dx") Port as u16, in("eax") Value, options(nomem, nostack));
    }
    #[inline(always)]
    pub unsafe fn WRITE_PORT_BUFFER_UCHAR(Port: *mut u8, Buffer: *const u8, Count: u32) {
        for i in 0..Count as isize { WRITE_PORT_UCHAR(Port, *Buffer.offset(i)); }
    }
    #[inline(always)]
    pub unsafe fn WRITE_PORT_BUFFER_USHORT(Port: *mut u16, Buffer: *const u16, Count: u32) {
        for i in 0..Count as isize { WRITE_PORT_USHORT(Port, *Buffer.offset(i)); }
    }
    #[inline(always)]
    pub unsafe fn WRITE_PORT_BUFFER_ULONG(Port: *mut u32, Buffer: *const u32, Count: u32) {
        for i in 0..Count as isize { WRITE_PORT_ULONG(Port, *Buffer.offset(i)); }
    }
}
pub use port_io::*;
