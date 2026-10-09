// pl011.sys — Rust translation of drivers/serial/pl011/pl011.c (minimal PL011
// UART driver for ARM64 QEMU virt). DeepSeek+c2rust translation (Batch 3 driver
// residue), hand-integrated. C kept on disk per the never-delete rule.
//
// IoGetCurrentIrpStackLocation is an INLINE DDK helper (not an ntoskrnl export)
// → ntkrnl_sys::inline::tier1. Dispatch routines are NTAPI (extern "system");
// assigned into MajorFunction via the transmute-to-PDRIVER_DISPATCH bridge
// (i386 nominal extern "system" ≠ extern "stdcall" fn-pointer type).
use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::driver_ddk::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::ddk::{
    IoCreateDevice, IoCreateSymbolicLink, IoDeleteDevice,
    MmMapIoSpace, MmUnmapIoSpace, RtlInitUnicodeString,
    PDEVICE_OBJECT, PDRIVER_OBJECT, PHYSICAL_ADDRESS, PIO_STACK_LOCATION, PIRP,
    PUNICODE_STRING, PDRIVER_DISPATCH, _DEVICE_OBJECT, _IRP, _UNICODE_STRING,
};
// IoCompleteRequest comes from driver_ddk::* above.
use ntkrnl_sys::inline::tier1::IoGetCurrentIrpStackLocation;

// ---- file-scope preamble (translated from pl011.c #defines) ----
// FILE_DEVICE_SERIAL_PORT lives in hal_bindgen (x86-only, configured out on
// arm64) — define locally (winioctl.h value) so pl011 builds on all 3 arches.
const FILE_DEVICE_SERIAL_PORT: u32 = 27;
const UARTDR: u32 = 0x000; // Data Register
const UARTFR: u32 = 0x018; // Flag Register
const UARTCR: u32 = 0x030; // Control Register
const UARTFR_TXFF: u32 = 1 << 5; // TX FIFO Full
const UARTFR_RXFE: u32 = 1 << 4; // RX FIFO Empty
/// QEMU `virt`'s PL011. On a Pi this address is plain RAM, so DriverEntry
/// checks the PrimeCell ID before it touches a register (`is_primecell`).
const PL011_PHYS_ADDR: u64 = 0x09000000;
/// PrimeCell component ID registers, 0xFF0..0xFFC: bytes 0D F0 05 B1 on every
/// PrimeCell, the PL011 included.
const UARTPCELLID0: usize = 0xFF0;
const PRIMECELL_ID: [u32; 4] = [0x0D, 0xF0, 0x05, 0xB1];
const STATUS_NO_SUCH_DEVICE: NTSTATUS = 0xC000_000Eu32 as NTSTATUS;

/// Whether `base` maps a PrimeCell. Read-only, so a mapping of RAM is harmless.
unsafe fn is_primecell(base: PVOID) -> bool {
    (0..4).all(|i| {
        let reg = (base as *const u8).add(UARTPCELLID0 + 4 * i) as *const u32; // nolint: magic-offset (PL011 MMIO register map)
        core::ptr::read_volatile(reg) & 0xFF == PRIMECELL_ID[i]
    })
}
const PL011_REG_SIZE: u32 = 0x1000;

static mut g_UartBase: PVOID = core::ptr::null_mut();

unsafe fn Pl011WriteByte(ch: UCHAR) {
    let dr = (g_UartBase as *mut u8).add(UARTDR as usize) as *mut u32; // nolint: magic-offset (PL011 MMIO register map)
    let fr = (g_UartBase as *mut u8).add(UARTFR as usize) as *mut u32; // nolint: magic-offset (PL011 MMIO register map)
    // Spin-wait until TX FIFO is not full
    while core::ptr::read_volatile(fr) & UARTFR_TXFF != 0 {}
    core::ptr::write_volatile(dr, ch as u32);
}

unsafe fn Pl011RxReady() -> BOOLEAN {
    let fr = (g_UartBase as *mut u8).add(UARTFR as usize) as *mut u32; // nolint: magic-offset (PL011 MMIO register map)
    ((core::ptr::read_volatile(fr) & UARTFR_RXFE) == 0) as BOOLEAN
}

unsafe fn Pl011ReadByte() -> UCHAR {
    let dr = (g_UartBase as *mut u8).add(UARTDR as usize) as *mut u32; // nolint: magic-offset (PL011 MMIO register map)
    (core::ptr::read_volatile(dr) & 0xFF) as UCHAR
}

#[no_mangle]
pub unsafe extern "system" fn DriverEntry(
    DriverObject: PDRIVER_OBJECT,
    RegistryPath: PUNICODE_STRING,
) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let mut DeviceObject: PDEVICE_OBJECT = core::ptr::null_mut();
    let mut DeviceName: _UNICODE_STRING = core::mem::zeroed();
    let mut SymlinkName: _UNICODE_STRING = core::mem::zeroed();
    let mut PhysAddr: PHYSICAL_ADDRESS = core::mem::zeroed();

    let _ = RegistryPath; // UNREFERENCED_PARAMETER

    DbgPrint(b"[PL011] DriverEntry\n\0".as_ptr() as *const i8);

    // Map the PL011 registers
    PhysAddr.QuadPart = PL011_PHYS_ADDR as i64;
    g_UartBase = MmMapIoSpace(PhysAddr, PL011_REG_SIZE as ntkrnl_sys::ddk::SIZE_T, MmNonCached);
    if g_UartBase.is_null() {
        DbgPrint(b"[PL011] MmMapIoSpace FAILED\n\0".as_ptr() as *const i8);
        return STATUS_INSUFFICIENT_RESOURCES;
    }
    DbgPrint(b"[PL011] UART mapped at %p\n\0".as_ptr() as *const i8, g_UartBase);

    // Divergence from the C, which assumed QEMU: on a Pi the fixed address is
    // RAM, and the UARTCR read-modify-write below would store into it on every
    // boot. Nothing opens this device today, so a board without a PL011 there
    // simply gets no \Device\ArmSerial.
    if !is_primecell(g_UartBase) {
        DbgPrint(b"[PL011] no PrimeCell at %llx; not a QEMU virt board, no device\n\0".as_ptr() as *const i8, PL011_PHYS_ADDR);
        MmUnmapIoSpace(g_UartBase, PL011_REG_SIZE as ntkrnl_sys::ddk::SIZE_T);
        g_UartBase = core::ptr::null_mut();
        return STATUS_NO_SUCH_DEVICE;
    }

    // Ensure UART is enabled for TX and RX
    let cr = (g_UartBase as *mut u8).add(UARTCR as usize) as *mut u32; // nolint: magic-offset (PL011 MMIO register map)
    let mut crVal: u32 = core::ptr::read_volatile(cr);
    DbgPrint(
        b"[PL011] UARTCR = 0x%08lx (UARTEN=%d TXE=%d RXE=%d)\n\0".as_ptr() as *const i8,
        crVal,
        crVal & 1u32,
        (crVal >> 8) & 1u32,
        (crVal >> 9) & 1u32,
    );
    if (crVal & (1u32 << 9)) == 0 {
        crVal |= (1u32 << 9) | (1u32 << 0); // RXE + UARTEN
        core::ptr::write_volatile(cr, crVal);
        DbgPrint(
            b"[PL011] Enabled RXE: UARTCR = 0x%08lx\n\0".as_ptr() as *const i8,
            core::ptr::read_volatile(cr),
        );
    }

    // Create device object — \Device\ArmSerial
    const DEVICE_NAME_STR: &[u16] = &[
        b'\\' as u16, b'D' as u16, b'e' as u16, b'v' as u16, b'i' as u16, b'c' as u16, b'e' as u16,
        b'\\' as u16, b'A' as u16, b'r' as u16, b'm' as u16, b'S' as u16, b'e' as u16, b'r' as u16,
        b'i' as u16, b'a' as u16, b'l' as u16, 0u16,
    ];
    RtlInitUnicodeString(&raw mut DeviceName, DEVICE_NAME_STR.as_ptr());
    Status = IoCreateDevice(
        DriverObject,
        0u32,
        &raw mut DeviceName,
        FILE_DEVICE_SERIAL_PORT,
        0u32,
        0u8, // FALSE
        &raw mut DeviceObject,
    );
    if Status < 0 {
        DbgPrint(b"[PL011] IoCreateDevice failed: 0x%lx\n\0".as_ptr() as *const i8, Status);
        MmUnmapIoSpace(g_UartBase, PL011_REG_SIZE as ntkrnl_sys::ddk::SIZE_T);
        g_UartBase = core::ptr::null_mut();
        return Status;
    }

    (*DeviceObject).Flags |= DO_BUFFERED_IO;
    (*DeviceObject).Flags &= !DO_DEVICE_INITIALIZING;

    // Create symbolic link \DosDevices\COM1
    const SYMLINK_NAME_STR: &[u16] = &[
        b'\\' as u16, b'D' as u16, b'o' as u16, b's' as u16, b'D' as u16, b'e' as u16, b'v' as u16,
        b'i' as u16, b'c' as u16, b'e' as u16, b's' as u16, b'\\' as u16, b'C' as u16, b'O' as u16,
        b'M' as u16, b'1' as u16, 0u16,
    ];
    RtlInitUnicodeString(&raw mut SymlinkName, SYMLINK_NAME_STR.as_ptr());
    Status = IoCreateSymbolicLink(&raw mut SymlinkName, &raw mut DeviceName);
    if Status < 0 {
        DbgPrint(b"[PL011] IoCreateSymbolicLink failed: 0x%lx\n\0".as_ptr() as *const i8, Status);
        IoDeleteDevice(DeviceObject);
        MmUnmapIoSpace(g_UartBase, PL011_REG_SIZE as ntkrnl_sys::ddk::SIZE_T);
        g_UartBase = core::ptr::null_mut();
        return Status;
    }

    // Set up dispatch routines (NTAPI → PDRIVER_DISPATCH via transmute bridge)
    type DispatchFn = Option<unsafe extern "system" fn(*mut _DEVICE_OBJECT, *mut _IRP) -> NTSTATUS>;
    (*DriverObject).MajorFunction[IRP_MJ_CREATE as usize] =
        core::mem::transmute::<DispatchFn, PDRIVER_DISPATCH>(Some(Pl011Create));
    (*DriverObject).MajorFunction[IRP_MJ_CLOSE as usize] =
        core::mem::transmute::<DispatchFn, PDRIVER_DISPATCH>(Some(Pl011Close));
    (*DriverObject).MajorFunction[IRP_MJ_READ as usize] =
        core::mem::transmute::<DispatchFn, PDRIVER_DISPATCH>(Some(Pl011Read));
    (*DriverObject).MajorFunction[IRP_MJ_WRITE as usize] =
        core::mem::transmute::<DispatchFn, PDRIVER_DISPATCH>(Some(Pl011Write));

    DbgPrint(b"[PL011] Driver loaded OK\n\0".as_ptr() as *const i8);
    STATUS_SUCCESS
}

unsafe extern "system" fn Pl011Create(DeviceObject: PDEVICE_OBJECT, Irp: PIRP) -> NTSTATUS {
    let _ = DeviceObject;
    (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
    (*Irp).IoStatus.Information = 0;
    IoCompleteRequest(Irp, IO_NO_INCREMENT as i8);
    STATUS_SUCCESS
}

unsafe extern "system" fn Pl011Close(DeviceObject: PDEVICE_OBJECT, Irp: PIRP) -> NTSTATUS {
    let _ = DeviceObject;
    (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
    (*Irp).IoStatus.Information = 0;
    IoCompleteRequest(Irp, IO_NO_INCREMENT as i8);
    STATUS_SUCCESS
}

unsafe extern "system" fn Pl011Write(DeviceObject: PDEVICE_OBJECT, Irp: PIRP) -> NTSTATUS {
    let _ = DeviceObject;
    let IrpSp: PIO_STACK_LOCATION = IoGetCurrentIrpStackLocation(Irp);
    let Buffer: PUCHAR = (*Irp).AssociatedIrp.SystemBuffer as PUCHAR;
    let Length: ULONG = (*IrpSp).Parameters.Write.Length;

    let mut i: ULONG = 0;
    while i < Length {
        if *Buffer.add(i as usize) as i32 == b'\n' as i32 {
            Pl011WriteByte(b'\r' as UCHAR);
        }
        Pl011WriteByte(*Buffer.add(i as usize));
        i = i.wrapping_add(1);
    }

    (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
    (*Irp).IoStatus.Information = Length as ntkrnl_sys::ddk::ULONG_PTR;
    IoCompleteRequest(Irp, IO_NO_INCREMENT as i8);
    STATUS_SUCCESS
}

unsafe extern "system" fn Pl011Read(DeviceObject: PDEVICE_OBJECT, Irp: PIRP) -> NTSTATUS {
    let _ = DeviceObject;
    let IrpSp: PIO_STACK_LOCATION = IoGetCurrentIrpStackLocation(Irp);
    let Buffer: PUCHAR = (*Irp).AssociatedIrp.SystemBuffer as PUCHAR;
    let MaxLength: ULONG = (*IrpSp).Parameters.Read.Length;
    let mut BytesRead: ULONG = 0;

    if MaxLength == 0 {
        (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
        (*Irp).IoStatus.Information = 0;
        IoCompleteRequest(Irp, IO_NO_INCREMENT as i8);
        return STATUS_SUCCESS;
    }

    // Pure non-blocking read (see C: ARM64 has no timer IRQ / preemption, so never
    // busy-wait here — return what the FIFO has and let the caller poll+sleep).
    while BytesRead < MaxLength {
        if Pl011RxReady() == 0 {
            break;
        }
        let ch: UCHAR = Pl011ReadByte();
        if ch as i32 == 0x08 || ch as i32 == 0x7F {
            // backspace: erase one char (BS Space BS)
            if BytesRead > 0 {
                BytesRead = BytesRead.wrapping_sub(1);
                Pl011WriteByte(0x08);
                Pl011WriteByte(b' ');
                Pl011WriteByte(0x08);
            }
            continue;
        }
        if ch as i32 == b'\r' as i32 {
            Pl011WriteByte(b'\r');
            Pl011WriteByte(b'\n');
            *Buffer.add(BytesRead as usize) = b'\r';
            BytesRead = BytesRead.wrapping_add(1);
            break; // line complete
        } else {
            Pl011WriteByte(ch);
            *Buffer.add(BytesRead as usize) = ch;
            BytesRead = BytesRead.wrapping_add(1);
        }
    }

    (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
    (*Irp).IoStatus.Information = BytesRead as ntkrnl_sys::ddk::ULONG_PTR;
    IoCompleteRequest(Irp, IO_NO_INCREMENT as i8);
    STATUS_SUCCESS
}
