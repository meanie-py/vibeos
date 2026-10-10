#[allow(unused_imports)] use crate::{ERR, TRACE};
#[allow(dead_code, non_upper_case_globals)] const DbgDefaultChannel: u32 = crate::frldr::DPRINT_HWDETECT; // pchw.c: DBG_DEFAULT_CHANNEL(HWDETECT)
use ntkrnl_sys::base::*;
use crate::frldr::*;
use ntkrnl_sys::ddk::{CM_INT13_DRIVE_PARAMETER, CM_PARTIAL_RESOURCE_LIST, PCM_FLOPPY_DEVICE_DATA, PCM_INT13_DRIVE_PARAMETER, PCM_PARTIAL_RESOURCE_DESCRIPTOR, PCM_PARTIAL_RESOURCE_LIST, PCONFIGURATION_COMPONENT_DATA, PCSTR};
use core::ffi::c_void;
pub use ntkrnl_sys::ddk::_CM_SHARE_DISPOSITION_CmResourceShareDeviceExclusive as CmResourceShareDeviceExclusive;
pub use ntkrnl_sys::ddk::_CM_SHARE_DISPOSITION_CmResourceShareUndetermined as CmResourceShareUndetermined;
pub use ntkrnl_sys::ddk::_CONFIGURATION_CLASS_ControllerClass as ControllerClass;
pub use ntkrnl_sys::ddk::_CONFIGURATION_TYPE_DiskController as DiskController;
pub use ntkrnl_sys::ddk::_CONFIGURATION_TYPE_DiskPeripheral as DiskPeripheral;
pub use ntkrnl_sys::ddk::_CONFIGURATION_TYPE_FloppyDiskPeripheral as FloppyDiskPeripheral;
pub use ntkrnl_sys::ddk::_IDENTIFIER_FLAG_Input as Input;
pub use ntkrnl_sys::ddk::_CONFIGURATION_CLASS_PeripheralClass as PeripheralClass;

// pchw.c dropped content (rustconv drops non-fn).
const CLOCK_TICK_RATE: ULONG = 1193182;
const HZ: ULONG = 100;
const LATCH: ULONG = CLOCK_TICK_RATE / HZ;
static mut delay_count: ::core::ffi::c_uint = 1;
const MILLISEC: ULONG = 10;
const PRECISION: ULONG = 8;

// pchw.c:41 `GET_HARDDISK_CONFIG_DATA GetHarddiskConfigurationData = NULL;`
// — C global function pointer, owned by pchw (canonical). Assigned by
// machpc.rs DetectHardware (`= PcGetHarddiskConfigurationData as *mut c_void`).
// Layout-compatible: Option<fn> is pointer-sized with NULL niche, matching
// machpc.rs's `extern static mut *mut c_void` view.
pub type GET_HARDDISK_CONFIG_DATA =
    Option<unsafe extern "C" fn(DriveNumber: UCHAR, pSize: *mut ULONG) -> PCM_PARTIAL_RESOURCE_LIST>;
#[no_mangle]
pub static mut GetHarddiskConfigurationData: GET_HARDDISK_CONFIG_DATA = None;

// arc.h IDENTIFIER_FLAG bit (sibling of ddk Input/ConsoleIn/ConsoleOut).
const Output: i32 = 0x40;

// debug.h enum _FRLDR_BUGCHECK_CODES (dropped by rustconv).
const TEST_BUGCHECK: ULONG = 0;
const MISSING_HARDWARE_REQUIREMENTS: ULONG = 1;
const FREELDR_IMAGE_CORRUPTION: ULONG = 2;
const MEMORY_INIT_FAILURE: ULONG = 3;

// ddk CM structs (bindgen names them _CM_*; bare aliases for the underscore
// names rustconv emitted).
pub use ntkrnl_sys::ddk::_CM_PARTIAL_RESOURCE_LIST as _CM_PARTIAL_RESOURCE_LIST;
pub use ntkrnl_sys::ddk::_CM_PARTIAL_RESOURCE_DESCRIPTOR as _CM_PARTIAL_RESOURCE_DESCRIPTOR;
pub use ntkrnl_sys::ddk::_CM_FLOPPY_DEVICE_DATA as _CM_FLOPPY_DEVICE_DATA;

extern "C" {
    // hwdisk.c external freeldr global (UCHAR per pchw.c declaration).
    static PcBiosDiskCount: UCHAR;
    fn DiskResetController(DriveNumber: UCHAR) -> BOOLEAN;
    fn FldrCreateComponentKey( Parent: PCONFIGURATION_COMPONENT_DATA, Class: u32, Type: u32, Flags: u32, Key: ULONG, Affinity: ULONG, Identifier: PCHAR, Data: PCM_PARTIAL_RESOURCE_LIST, Size: ULONG, ComponentKey: *mut PCONFIGURATION_COMPONENT_DATA, ) -> BOOLEAN;
    fn FrLdrHeapAlloc(MemorySize: usize, Tag: ULONG) -> PVOID;
    fn RtlStringCbPrintfA(pszDest: PCHAR, cbDest: usize, pszFormat: PCHAR, ...) -> LONG;
    #[link_name = "MachGetFloppyCount"]
    fn MachGetFloppyCount__c() -> UCHAR;
    fn FldrSetConfigurationData( ComponentKey: PCONFIGURATION_COMPONENT_DATA, ResourceList: PCM_PARTIAL_RESOURCE_LIST, Size: ULONG, );
    fn GetHarddiskIdentifier(DriveNumber: UCHAR) -> PCSTR;
    fn MachDiskGetDriveGeometry(DriveNumber: UCHAR, Geometry: *mut GEOMETRY) -> BOOLEAN;
}

// machpc.rs returns UCHAR; a ULONG decl read 24 bits nobody wrote.
#[allow(non_snake_case)]
unsafe fn MachGetFloppyCount() -> ULONG {
    MachGetFloppyCount__c() as ULONG
}
extern "C" {
    fn READ_PORT_UCHAR(Port: *mut u8) -> u8;
    fn WRITE_PORT_UCHAR(Port: *mut u8, Value: u8);
    fn FrLdrBugCheckWithMessage( BugCheckCode: ULONG, File: PCHAR, Line: ULONG, Format: PCHAR, ... );
    fn __cpuid(CpuInfo: *mut INT, InfoType: INT);
}

unsafe fn __StallExecutionProcessor(Loops: ULONG) {
    let mut i: u32 = 0;
    while i < Loops {
        core::ptr::read_volatile(&i);
        i = i.wrapping_add(1);
    }
}

// BIOS-only: uefildr's StallExecutionProcessor comes from UEFI-active C.
// Gated out of the `uefi` archive (per-binary-owner dup).
#[cfg(not(feature = "uefi"))]
#[no_mangle]
pub unsafe extern "C" fn StallExecutionProcessor(Microseconds: ULONG) {
    let LoopCount: ULONGLONG = ((delay_count as ULONGLONG) * (Microseconds as ULONGLONG)) / 1000u64;
    __StallExecutionProcessor(LoopCount as ULONG);
}

unsafe fn Read8254Timer() -> u32 {

    let mut Count: u32;

    WRITE_PORT_UCHAR(0x43 as *mut u8, 0x00);
    Count = READ_PORT_UCHAR(0x40 as *mut u8) as u32;
    Count |= (READ_PORT_UCHAR(0x40 as *mut u8) as u32) << 8;

    return Count;
}

unsafe fn WaitFor8254Wraparound() {
    let mut CurCount: u32;
    let mut PrevCount: u32 = !0;
    let mut Delta: i32;

    CurCount = Read8254Timer();

    loop {
        PrevCount = CurCount;
        CurCount = Read8254Timer();
        Delta = CurCount.wrapping_sub(PrevCount) as i32;

        if !(Delta < 300) {
            break;
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn HalpCalibrateStallExecution() {

    let mut i: u32;
    let mut calib_bit: u32;
    let mut CurCount: u32;

    WRITE_PORT_UCHAR(0x43 as *mut u8, 0x34);
    WRITE_PORT_UCHAR(0x40 as *mut u8, (LATCH & 0xff) as u8);
    WRITE_PORT_UCHAR(0x40 as *mut u8, (LATCH >> 8) as u8);

    delay_count = 1;

    loop {
        delay_count <<= 1;

        WaitFor8254Wraparound();

        __StallExecutionProcessor(delay_count);

        CurCount = Read8254Timer();

        if !(CurCount > LATCH / 2) {
            break;
        }
    }

    delay_count >>= 1;

    calib_bit = delay_count;

    i = 0;
    while i < PRECISION {
        calib_bit >>= 1;

        if calib_bit == 0 {
            break;
        }

        delay_count |= calib_bit;

        WaitFor8254Wraparound();

        __StallExecutionProcessor(delay_count);

        CurCount = Read8254Timer();

        if CurCount <= LATCH / 2 {
            delay_count &= !calib_bit;
        }

        i += 1;
    }

    delay_count /= MILLISEC / 2;
}

unsafe fn GetFloppyType(DriveNumber: UCHAR) -> UCHAR {

    let Data: UCHAR;

    WRITE_PORT_UCHAR(0x70 as PUCHAR, 0x10);
    Data = READ_PORT_UCHAR(0x71 as PUCHAR);

    if DriveNumber == 0 {
        return Data >> 4;
    } else if DriveNumber == 1 {
        return Data & 0x0F;
    }

    return 0;
}

unsafe fn GetInt1eTable() -> PVOID {
    let SegPtr: PUSHORT = 0x7A as PUSHORT;
    let OfsPtr: PUSHORT = 0x78 as PUSHORT;
    
    (((((*SegPtr) as ULONG) << 4) + (*OfsPtr) as ULONG) as ULONG_PTR) as PVOID
}

unsafe fn DetectBiosFloppyPeripheral(ControllerKey: PCONFIGURATION_COMPONENT_DATA) {

    let mut PartialResourceList: PCM_PARTIAL_RESOURCE_LIST;
    let mut PartialDescriptor: PCM_PARTIAL_RESOURCE_DESCRIPTOR;
    let mut FloppyData: PCM_FLOPPY_DEVICE_DATA;
    let mut Identifier: [CHAR; 20] = [0; 20];
    let mut PeripheralKey: PCONFIGURATION_COMPONENT_DATA = core::ptr::null_mut();
    let mut Size: ULONG;
    let mut FloppyNumber: UCHAR;
    let _FloppyType: UCHAR;
    let MaxDensity: [ULONG; 6] = [0, 360, 1200, 720, 1440, 2880];
    let mut Ptr: PUCHAR;

    FloppyNumber = 0;
    while FloppyNumber < 2 {
        let FloppyType = GetFloppyType(FloppyNumber);

        if (FloppyType > 5) || (FloppyType == 0) {
            FloppyNumber += 1;
            continue;
        }

        if DiskResetController(FloppyNumber) == 0 {
            FloppyNumber += 1;
            continue;
        }

        Ptr = GetInt1eTable() as PUCHAR;

        RtlStringCbPrintfA(
            Identifier.as_mut_ptr(),
            core::mem::size_of_val(&Identifier),
            b"FLOPPY%d\0".as_ptr() as PCHAR,
            (FloppyNumber + 1) as i32,
        );

        Size = core::mem::offset_of!(_CM_PARTIAL_RESOURCE_LIST, PartialDescriptors) as ULONG
            + core::mem::size_of::<_CM_PARTIAL_RESOURCE_DESCRIPTOR>() as ULONG
            + core::mem::size_of::<_CM_FLOPPY_DEVICE_DATA>() as ULONG;
        PartialResourceList = FrLdrHeapAlloc(Size as usize, TAG_HW_RESOURCE_LIST) as PCM_PARTIAL_RESOURCE_LIST;
        if PartialResourceList == core::ptr::null_mut() {
            ERR!("Failed to allocate resource descriptor! Ignoring remaining floppy peripherals. (FloppyNumber = %u)\n", (FloppyNumber) as usize);
            return;
        }

        // C: RtlZeroMemory(PartialResourceList, Size).
        // rustconv emitted `write_bytes(typed_ptr, 0, 1)` which zeroes
        // sizeof(*typed_ptr) bytes — bigger than the trailing-flex-array
        // alloc, smashing the next heap block. Use byte-count form.
        core::ptr::write_bytes(PartialResourceList as *mut u8, 0, Size as usize);
        (*PartialResourceList).Version = 1;
        (*PartialResourceList).Revision = 1;
        (*PartialResourceList).Count = 1;

        PartialDescriptor = core::ptr::addr_of_mut!((*PartialResourceList).PartialDescriptors[0]);
        (*PartialDescriptor).Type = 8;
        (*PartialDescriptor).ShareDisposition = CmResourceShareUndetermined as u8;
        (*PartialDescriptor).u.DeviceSpecificData.DataSize = core::mem::size_of::<_CM_FLOPPY_DEVICE_DATA>() as ULONG;

        FloppyData = PartialDescriptor.offset(1) as PCM_FLOPPY_DEVICE_DATA;
        (*FloppyData).Version = 2;
        (*FloppyData).Revision = 0;
        (*FloppyData).MaxDensity = MaxDensity[FloppyType as usize];
        (*FloppyData).MountDensity = 0;
        core::ptr::copy_nonoverlapping(
            Ptr,
            core::ptr::addr_of_mut!((*FloppyData).StepRateHeadUnloadTime) as PUCHAR,
            11,
        );
        (*FloppyData).MaximumTrackValue = if FloppyType == 1 { 39 } else { 79 };
        (*FloppyData).DataTransferRate = 0;

        FldrCreateComponentKey(
            ControllerKey,
            PeripheralClass,
            FloppyDiskPeripheral,
            Input | ((Output) as u32),
            FloppyNumber as ULONG,
            0xFFFFFFFF,
            Identifier.as_mut_ptr(),
            PartialResourceList,
            Size,
            core::ptr::addr_of_mut!(PeripheralKey),
        );

        FloppyNumber += 1;
    }
}

unsafe fn DetectBiosFloppyController(BusKey: PCONFIGURATION_COMPONENT_DATA) -> PCONFIGURATION_COMPONENT_DATA {

    let mut ControllerKey: PCONFIGURATION_COMPONENT_DATA = core::ptr::null_mut();
    let PartialResourceList: PCM_PARTIAL_RESOURCE_LIST;
    let mut PartialDescriptor: PCM_PARTIAL_RESOURCE_DESCRIPTOR;
    let Size: ULONG;
    let FloppyCount: ULONG;

    FloppyCount = MachGetFloppyCount();
    TRACE!("Floppy count: %u\n", (FloppyCount) as usize);

    // C: Size = FIELD_OFFSET(CM_PARTIAL_RESOURCE_LIST, PartialDescriptors[3]).
    // rustconv emitted `sizeof(CM_PARTIAL_RESOURCE_LIST) + 2 * sizeof(PCM_PARTIAL_RESOURCE_DESCRIPTOR)`
    // where `PCM_…` is a *pointer* (4 bytes on i386), under-allocating by
    // 2*sizeof(descriptor) - 2*sizeof(ptr). The under-alloc's RtlZeroMemory +
    // descriptor writes overflowed into the next free block's header,
    // corrupting the default-heap free list right after rosload UiInit.
    Size = crate::FIELD_OFFSET!(CM_PARTIAL_RESOURCE_LIST, PartialDescriptors[3]);
    PartialResourceList = FrLdrHeapAlloc(Size as usize, TAG_HW_RESOURCE_LIST) as PCM_PARTIAL_RESOURCE_LIST;
    if PartialResourceList == core::ptr::null_mut() {
        ERR!("Failed to allocate resource descriptor\n");
        return core::ptr::null_mut();
    }

    RtlZeroMemory(PartialResourceList as *mut core::ffi::c_void, (Size) as usize );
    (*PartialResourceList).Version = 1;
    (*PartialResourceList).Revision = 1;
    (*PartialResourceList).Count = 3;

    PartialDescriptor = core::ptr::addr_of_mut!((*PartialResourceList).PartialDescriptors[0]);
    (*PartialDescriptor).Type = 1;
    (*PartialDescriptor).ShareDisposition = CmResourceShareDeviceExclusive as u8;
    (*PartialDescriptor).Flags = 0x01;
    (*PartialDescriptor).u.Port.Start.__bindgen_anon_1.LowPart = 0x03F0;
    (*PartialDescriptor).u.Port.Start.__bindgen_anon_1.HighPart = 0x0;
    (*PartialDescriptor).u.Port.Length = 8;

    PartialDescriptor = (core::ptr::addr_of_mut!((*PartialResourceList).PartialDescriptors) as *mut _CM_PARTIAL_RESOURCE_DESCRIPTOR).add(1);
    (*PartialDescriptor).Type = 2;
    (*PartialDescriptor).ShareDisposition = CmResourceShareUndetermined as u8;
    (*PartialDescriptor).Flags = 0x01;
    (*PartialDescriptor).u.Interrupt.Level = 6;
    (*PartialDescriptor).u.Interrupt.Vector = 6;
    (*PartialDescriptor).u.Interrupt.Affinity = 0xFFFFFFFF;

    PartialDescriptor = (core::ptr::addr_of_mut!((*PartialResourceList).PartialDescriptors) as *mut _CM_PARTIAL_RESOURCE_DESCRIPTOR).add(2);
    (*PartialDescriptor).Type = 3;
    (*PartialDescriptor).ShareDisposition = CmResourceShareUndetermined as u8;
    (*PartialDescriptor).Flags = 0;
    (*PartialDescriptor).u.Dma.Channel = 2;
    (*PartialDescriptor).u.Dma.Port = 0;

    FldrCreateComponentKey(
        BusKey,
        ControllerClass as u32,
        DiskController as u32,
        Output as u32 | Input as u32,
        0x0,
        0xFFFFFFFF,
        core::ptr::null_mut(),
        PartialResourceList,
        Size,
        core::ptr::addr_of_mut!(ControllerKey),
    );

    if FloppyCount != 0 {
        DetectBiosFloppyPeripheral(ControllerKey);
    }

    ControllerKey
}

#[no_mangle]
pub unsafe extern "C" fn DetectBiosDisks(
    SystemKey: PCONFIGURATION_COMPONENT_DATA,
    BusKey: PCONFIGURATION_COMPONENT_DATA,
) {

    let ControllerKey: PCONFIGURATION_COMPONENT_DATA;
    let mut DiskKey: PCONFIGURATION_COMPONENT_DATA = core::ptr::null_mut();
    let mut PartialResourceList: PCM_PARTIAL_RESOURCE_LIST;
    let Int13Drives: PCM_INT13_DRIVE_PARAMETER;
    let mut Geometry: GEOMETRY = core::mem::zeroed();
    let DiskCount: UCHAR;
    let mut DriveNumber: UCHAR;
    let mut i: USHORT;
    let mut Size: ULONG;

    DiskCount = PcBiosDiskCount;

    ControllerKey = DetectBiosFloppyController(BusKey);
    if ControllerKey.is_null() {
        ERR!("Failed to detect BIOS disk controller\n");
        return;
    }

    Size = crate::FIELD_OFFSET!(CM_PARTIAL_RESOURCE_LIST, PartialDescriptors[1])
        + core::mem::size_of::<CM_INT13_DRIVE_PARAMETER>() as ULONG * DiskCount as ULONG;
    
    PartialResourceList = FrLdrHeapAlloc((Size) as usize , TAG_HW_RESOURCE_LIST) as PCM_PARTIAL_RESOURCE_LIST;
    if PartialResourceList.is_null() {
        ERR!("Failed to allocate resource descriptor\n");
        return;
    }

    RtlZeroMemory(PartialResourceList as *mut core::ffi::c_void, (Size) as usize );
    (*PartialResourceList).Version = 1;
    (*PartialResourceList).Revision = 1;
    (*PartialResourceList).Count = 1;
    (*PartialResourceList).PartialDescriptors[0].Type = CmResourceTypeDeviceSpecific;
    (*PartialResourceList).PartialDescriptors[0].ShareDisposition = 0;
    (*PartialResourceList).PartialDescriptors[0].Flags = 0;
    (*PartialResourceList).PartialDescriptors[0].u.DeviceSpecificData.DataSize =
        core::mem::size_of::<CM_INT13_DRIVE_PARAMETER>() as ULONG * DiskCount as ULONG;

    Int13Drives = (core::ptr::addr_of_mut!((*PartialResourceList).PartialDescriptors) as *mut _CM_PARTIAL_RESOURCE_DESCRIPTOR).add(1) as PCM_INT13_DRIVE_PARAMETER;
    i = 0;
    while i < DiskCount as USHORT {
        DriveNumber = 0x80 + i as UCHAR;

        if MachDiskGetDriveGeometry(DriveNumber, core::ptr::addr_of_mut!(Geometry)) != 0 {
            (*Int13Drives.offset(i as isize)).DriveSelect = (DriveNumber) as u16 ;
            (*Int13Drives.offset(i as isize)).MaxCylinders = Geometry.Cylinders - 1;
            (*Int13Drives.offset(i as isize)).SectorsPerTrack = Geometry.SectorsPerTrack as USHORT;
            (*Int13Drives.offset(i as isize)).MaxHeads = (Geometry.Heads - 1) as USHORT;
            (*Int13Drives.offset(i as isize)).NumberDrives = (DiskCount) as u16 ;

            TRACE!("Disk %x: %u Cylinders  %u Heads  %u Sectors  %u Bytes\n", (DriveNumber) as usize, (Geometry.Cylinders - 1) as usize, (Geometry.Heads - 1) as usize, (Geometry.SectorsPerTrack) as usize, (Geometry.BytesPerSector) as usize);
        }
        i += 1;
    }

    FldrSetConfigurationData(SystemKey, PartialResourceList, Size);

    i = 0;
    while i < DiskCount as USHORT {
        let Identifier: PCSTR;

        DriveNumber = 0x80 + i as UCHAR;

        // read_volatile: machpc.rs initializes GetHarddiskConfigurationData at
        // runtime via an extern `*mut c_void` view (different type than this
        // crate's Option<fn>). LLVM cannot see that cross-CGU write, so without
        // the volatile load it folds the static's value to its compile-time
        // initializer (None), proves unwrap() panics, and emits `unreachable`
        // for every caller — folding DetectBiosDisks to a ud2 stub that
        // triple-faulted PcHwDetect before any serial output.
        let gh = core::ptr::read_volatile(core::ptr::addr_of!(GetHarddiskConfigurationData));
        PartialResourceList = (gh.unwrap())(DriveNumber, core::ptr::addr_of_mut!(Size));
        Identifier = GetHarddiskIdentifier(DriveNumber);

        FldrCreateComponentKey(
            ControllerKey,
            PeripheralClass,
            DiskPeripheral,
            (Output | ((Input) as i32)) as u32,
            i as ULONG,
            0xFFFFFFFF,
            Identifier as PCHAR,
            PartialResourceList,
            Size,
            core::ptr::addr_of_mut!(DiskKey),
        );
        i += 1;
    }
}

#[no_mangle]
pub unsafe extern "C" fn FrLdrCheckCpuCompatibility() {
    dbgcon(b'F'); dbgcon(b'C'); dbgcon(b'C');
    let mut CpuInformation: [INT; 4] = [-1; 4];
    let NumberOfIds: ULONG;

    __cpuid(CpuInformation.as_mut_ptr(), 0);

    NumberOfIds = CpuInformation[0] as ULONG;

    if NumberOfIds == 0 {
        FrLdrBugCheckWithMessage(
            MISSING_HARDWARE_REQUIREMENTS,
            b"pchw.c\0".as_ptr() as PCHAR,
            24,
            b"ReactOS requires the CPUID instruction to return more than one supported ID.\n\n\0".as_ptr() as PCHAR,
        );
    }

    if NumberOfIds == 1 {
        let ProcessorFamily: INT;

        __cpuid(CpuInformation.as_mut_ptr(), 1);

        ProcessorFamily = (CpuInformation[0] >> 8) & 0xF;

        if ProcessorFamily < 5 {
            FrLdrBugCheckWithMessage(
                MISSING_HARDWARE_REQUIREMENTS,
                b"pchw.c\0".as_ptr() as PCHAR,
                44,
                b"Processor is too old (family %u < 5)\nReactOS requires a Pentium-level processor or newer.\0".as_ptr() as PCHAR,
                ProcessorFamily,
            );
        }
    }
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  *  FreeLoader
//  *
//  *  Copyright (C) 2003, 2004  Eric Kohl
//  *  Copyright (C) 2009  Hervé Poussineau
//  *
//  *  This program is free software; you can redistribute it and/or modify
//  *  it under the terms of the GNU General Public License as published by
//  *  the Free Software Foundation; either version 2 of the License, or
//  *  (at your option) any later version.
//  *
//  *  This program is distributed in the hope that it will be useful,
//  *  but WITHOUT ANY WARRANTY; without even the implied warranty of
//  *  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
//  *  GNU General Public License for more details.
//  *
//  *  You should have received a copy of the GNU General Public License along
//  *  with this program; if not, write to the Free Software Foundation, Inc.,
//  *  51 Franklin Street, Fifth Floor, Boston, MA 02110-1301 USA.
//  */
//
// #include <freeldr.h>
//
// #include <debug.h>
// DBG_DEFAULT_CHANNEL(HWDETECT);
//
// #define MILLISEC     (10)
// #define PRECISION    (8)
//
// #define CLOCK_TICK_RATE 1193182
//
// #define HZ (100)
// #define LATCH (CLOCK_TICK_RATE / HZ)
//
// static unsigned int delay_count = 1;
//
// /* Used for BIOS disks pre-enumeration performed when detecting the boot devices in InitializeBootDevices() */
// extern UCHAR PcBiosDiskCount;
//
// /* This function is slightly different in its PC and XBOX versions */
// GET_HARDDISK_CONFIG_DATA GetHarddiskConfigurationData = NULL;
//
// PCHAR
// GetHarddiskIdentifier(UCHAR DriveNumber);
//
// /* FUNCTIONS *****************************************************************/
//
// static
// VOID
// __StallExecutionProcessor(ULONG Loops)
// {
//     register volatile unsigned int i;
//     for (i = 0; i < Loops; i++);
// }
//
// VOID StallExecutionProcessor(ULONG Microseconds)
// {
//     ULONGLONG LoopCount = ((ULONGLONG)delay_count * (ULONGLONG)Microseconds) / 1000ULL;
//     __StallExecutionProcessor((ULONG)LoopCount);
// }
//
// static
// ULONG
// Read8254Timer(VOID)
// {
//     ULONG Count;
//
//     WRITE_PORT_UCHAR((PUCHAR)0x43, 0x00);
//     Count = READ_PORT_UCHAR((PUCHAR)0x40);
//     Count |= READ_PORT_UCHAR((PUCHAR)0x40) << 8;
//
//     return Count;
// }
//
// static
// VOID
// WaitFor8254Wraparound(VOID)
// {
//     ULONG CurCount;
//     ULONG PrevCount = ~0;
//     LONG Delta;
//
//     CurCount = Read8254Timer();
//
//     do
//     {
//         PrevCount = CurCount;
//         CurCount = Read8254Timer();
//         Delta = CurCount - PrevCount;
//
//         /*
//          * This limit for delta seems arbitrary, but it isn't, it's
//          * slightly above the level of error a buggy Mercury/Neptune
//          * chipset timer can cause.
//          */
//     }
//     while (Delta < 300);
// }
//
// VOID
// HalpCalibrateStallExecution(VOID)
// {
//     ULONG i;
//     ULONG calib_bit;
//     ULONG CurCount;
//
//     /* Initialise timer interrupt with MILLISECOND ms interval        */
//     WRITE_PORT_UCHAR((PUCHAR)0x43, 0x34);  /* binary, mode 2, LSB/MSB, ch 0 */
//     WRITE_PORT_UCHAR((PUCHAR)0x40, LATCH & 0xff); /* LSB */
//     WRITE_PORT_UCHAR((PUCHAR)0x40, LATCH >> 8); /* MSB */
//
//     /* Stage 1:  Coarse calibration                                   */
//
//     delay_count = 1;
//
//     do
//     {
//         /* Next delay count to try */
//         delay_count <<= 1;
//
//         WaitFor8254Wraparound();
//
//         /* Do the delay */
//         __StallExecutionProcessor(delay_count);
//
//         CurCount = Read8254Timer();
//     }
//     while (CurCount > LATCH / 2);
//
//     /* Get bottom value for delay */
//     delay_count >>= 1;
//
//     /* Stage 2:  Fine calibration                                     */
//
//     /* Which bit are we going to test */
//     calib_bit = delay_count;
//
//     for (i = 0; i < PRECISION; i++)
//     {
//         /* Next bit to calibrate */
//         calib_bit >>= 1;
//
//         /* If we have done all bits, stop */
//         if (!calib_bit) break;
//
//         /* Set the bit in delay_count */
//         delay_count |= calib_bit;
//
//         WaitFor8254Wraparound();
//
//         /* Do the delay */
//         __StallExecutionProcessor(delay_count);
//
//         CurCount = Read8254Timer();
//         /* If a tick has passed, turn the calibrated bit back off */
//         if (CurCount <= LATCH / 2)
//             delay_count &= ~calib_bit;
//     }
//
//     /* We're finished:  Do the finishing touches */
//
//     /* Calculate delay_count for 1ms */
//     delay_count /= (MILLISEC / 2);
// }
//
//
// static
// UCHAR
// GetFloppyType(UCHAR DriveNumber)
// {
//     UCHAR Data;
//
//     WRITE_PORT_UCHAR((PUCHAR)0x70, 0x10);
//     Data = READ_PORT_UCHAR((PUCHAR)0x71);
//
//     if (DriveNumber == 0)
//         return Data >> 4;
//     else if (DriveNumber == 1)
//         return Data & 0x0F;
//
//     return 0;
// }
//
// static
// PVOID
// GetInt1eTable(VOID)
// {
//     PUSHORT SegPtr = (PUSHORT)0x7A;
//     PUSHORT OfsPtr = (PUSHORT)0x78;
//
//     return (PVOID)((ULONG_PTR)(((ULONG)(*SegPtr)) << 4) + (ULONG)(*OfsPtr));
// }
//
// static
// VOID
// DetectBiosFloppyPeripheral(PCONFIGURATION_COMPONENT_DATA ControllerKey)
// {
//     PCM_PARTIAL_RESOURCE_LIST PartialResourceList;
//     PCM_PARTIAL_RESOURCE_DESCRIPTOR PartialDescriptor;
//     PCM_FLOPPY_DEVICE_DATA FloppyData;
//     CHAR Identifier[20];
//     PCONFIGURATION_COMPONENT_DATA PeripheralKey;
//     ULONG Size;
//     UCHAR FloppyNumber;
//     UCHAR FloppyType;
//     ULONG MaxDensity[6] = {0, 360, 1200, 720, 1440, 2880};
//     PUCHAR Ptr;
//
//     for (FloppyNumber = 0; FloppyNumber < 2; FloppyNumber++)
//     {
//         FloppyType = GetFloppyType(FloppyNumber);
//
//         if ((FloppyType > 5) || (FloppyType == 0))
//             continue;
//
//         if (!DiskResetController(FloppyNumber))
//             continue;
//
//         Ptr = GetInt1eTable();
//
//         /* Set 'Identifier' value */
//         RtlStringCbPrintfA(Identifier, sizeof(Identifier), "FLOPPY%d", FloppyNumber + 1);
//
//         Size = crate::FIELD_OFFSET!(CM_PARTIAL_RESOURCE_LIST, PartialDescriptors[1]) + sizeof(*FloppyData);
//         PartialResourceList = FrLdrHeapAlloc(Size, TAG_HW_RESOURCE_LIST);
//         if (PartialResourceList == NULL)
//         {
//             ERR("Failed to allocate resource descriptor! Ignoring remaining floppy peripherals. (FloppyNumber = %u)\n", (//                 FloppyNumber) as usize);
//             return;
//         }
//
//         RtlZeroMemory(PartialResourceList, Size);
//         PartialResourceList->Version = 1;
//         PartialResourceList->Revision = 1;
//         PartialResourceList->Count = 1;
//
//         PartialDescriptor = &PartialResourceList->PartialDescriptors[0];
//         PartialDescriptor->Type = CmResourceTypeDeviceSpecific;
//         PartialDescriptor->ShareDisposition = CmResourceShareUndetermined;
//         PartialDescriptor->u.DeviceSpecificData.DataSize = sizeof(*FloppyData);
//
//         FloppyData = (PCM_FLOPPY_DEVICE_DATA)(PartialDescriptor + 1);
//         FloppyData->Version = 2;
//         FloppyData->Revision = 0;
//         FloppyData->MaxDensity = MaxDensity[FloppyType];
//         FloppyData->MountDensity = 0;
//         RtlCopyMemory(&FloppyData->StepRateHeadUnloadTime, Ptr, 11);
//         FloppyData->MaximumTrackValue = (FloppyType == 1) ? 39 : 79;
//         FloppyData->DataTransferRate = 0;
//
//         FldrCreateComponentKey(ControllerKey,
//                                PeripheralClass,
//                                FloppyDiskPeripheral,
//                                Input | Output,
//                                FloppyNumber,
//                                0xFFFFFFFF,
//                                Identifier,
//                                PartialResourceList,
//                                Size,
//                                &PeripheralKey);
//     }
// }
//
// static
// PCONFIGURATION_COMPONENT_DATA
// DetectBiosFloppyController(PCONFIGURATION_COMPONENT_DATA BusKey)
// {
//     PCONFIGURATION_COMPONENT_DATA ControllerKey;
//     PCM_PARTIAL_RESOURCE_LIST PartialResourceList;
//     PCM_PARTIAL_RESOURCE_DESCRIPTOR PartialDescriptor;
//     ULONG Size;
//     ULONG FloppyCount;
//
//     FloppyCount = MachGetFloppyCount();
//     TRACE("Floppy count: %u\n", (FloppyCount) as usize);
//
//     /* Always create a BIOS disk controller, no matter if we have floppy drives or not */
//     Size = crate::FIELD_OFFSET!(CM_PARTIAL_RESOURCE_LIST, PartialDescriptors[3]);
//     PartialResourceList = FrLdrHeapAlloc(Size, TAG_HW_RESOURCE_LIST);
//     if (PartialResourceList == NULL)
//     {
//         ERR("Failed to allocate resource descriptor\n");
//         return NULL;
//     }
//
//     /* Initialize resource descriptor */
//     RtlZeroMemory(PartialResourceList, Size);
//     PartialResourceList->Version = 1;
//     PartialResourceList->Revision = 1;
//     PartialResourceList->Count = 3;
//
//     /* Set IO Port */
//     PartialDescriptor = &PartialResourceList->PartialDescriptors[0];
//     PartialDescriptor->Type = CmResourceTypePort;
//     PartialDescriptor->ShareDisposition = CmResourceShareDeviceExclusive;
//     PartialDescriptor->Flags = CM_RESOURCE_PORT_IO;
//     PartialDescriptor->u.Port.Start.LowPart = 0x03F0;
//     PartialDescriptor->u.Port.Start.HighPart = 0x0;
//     PartialDescriptor->u.Port.Length = 8;
//
//     /* Set Interrupt */
//     PartialDescriptor = &PartialResourceList->PartialDescriptors[1];
//     PartialDescriptor->Type = CmResourceTypeInterrupt;
//     PartialDescriptor->ShareDisposition = CmResourceShareUndetermined;
//     PartialDescriptor->Flags = CM_RESOURCE_INTERRUPT_LATCHED;
//     PartialDescriptor->u.Interrupt.Level = 6;
//     PartialDescriptor->u.Interrupt.Vector = 6;
//     PartialDescriptor->u.Interrupt.Affinity = 0xFFFFFFFF;
//
//     /* Set DMA channel */
//     PartialDescriptor = &PartialResourceList->PartialDescriptors[2];
//     PartialDescriptor->Type = CmResourceTypeDma;
//     PartialDescriptor->ShareDisposition = CmResourceShareUndetermined;
//     PartialDescriptor->Flags = 0;
//     PartialDescriptor->u.Dma.Channel = 2;
//     PartialDescriptor->u.Dma.Port = 0;
//
//     /* Create floppy disk controller */
//     FldrCreateComponentKey(BusKey,
//                            ControllerClass,
//                            DiskController,
//                            Output | Input,
//                            0x0,
//                            0xFFFFFFFF,
//                            NULL,
//                            PartialResourceList,
//                            Size,
//                            &ControllerKey);
//
//     if (FloppyCount)
//         DetectBiosFloppyPeripheral(ControllerKey);
//
//     return ControllerKey;
// }
//
// VOID
// DetectBiosDisks(PCONFIGURATION_COMPONENT_DATA SystemKey,
//                 PCONFIGURATION_COMPONENT_DATA BusKey)
// {
//     PCONFIGURATION_COMPONENT_DATA ControllerKey, DiskKey;
//     PCM_PARTIAL_RESOURCE_LIST PartialResourceList;
//     PCM_INT13_DRIVE_PARAMETER Int13Drives;
//     GEOMETRY Geometry;
//     UCHAR DiskCount, DriveNumber;
//     USHORT i;
//     ULONG Size;
//
//     /* The pre-enumeration of the BIOS disks was already done in InitializeBootDevices() */
//     DiskCount = PcBiosDiskCount;
//
//     /* Use the floppy disk controller as our controller */
//     ControllerKey = DetectBiosFloppyController(BusKey);
//     if (!ControllerKey)
//     {
//         ERR("Failed to detect BIOS disk controller\n");
//         return;
//     }
//
//     /* Allocate resource descriptor */
//     Size = crate::FIELD_OFFSET!(CM_PARTIAL_RESOURCE_LIST, PartialDescriptors[1]) +
//            sizeof(CM_INT13_DRIVE_PARAMETER) * DiskCount;
//     PartialResourceList = FrLdrHeapAlloc(Size, TAG_HW_RESOURCE_LIST);
//     if (PartialResourceList == NULL)
//     {
//         ERR("Failed to allocate resource descriptor\n");
//         return;
//     }
//
//     /* Initialize resource descriptor */
//     RtlZeroMemory(PartialResourceList, Size);
//     PartialResourceList->Version = 1;
//     PartialResourceList->Revision = 1;
//     PartialResourceList->Count = 1;
//     PartialResourceList->PartialDescriptors[0].Type = CmResourceTypeDeviceSpecific;
//     PartialResourceList->PartialDescriptors[0].ShareDisposition = 0;
//     PartialResourceList->PartialDescriptors[0].Flags = 0;
//     PartialResourceList->PartialDescriptors[0].u.DeviceSpecificData.DataSize =
//         sizeof(CM_INT13_DRIVE_PARAMETER) * DiskCount;
//
//     /* Get harddisk Int13 geometry data */
//     Int13Drives = (PCM_INT13_DRIVE_PARAMETER)&PartialResourceList->PartialDescriptors[1];
//     for (i = 0; i < DiskCount; i++)
//     {
//         DriveNumber = 0x80 + i;
//
//         if (MachDiskGetDriveGeometry(DriveNumber, &Geometry))
//         {
//             Int13Drives[i].DriveSelect = DriveNumber;
//             Int13Drives[i].MaxCylinders = Geometry.Cylinders - 1;
//             Int13Drives[i].SectorsPerTrack = (USHORT)Geometry.SectorsPerTrack;
//             Int13Drives[i].MaxHeads = (USHORT)Geometry.Heads - 1;
//             Int13Drives[i].NumberDrives = DiskCount;
//
//             TRACE("Disk %x: %u Cylinders  %u Heads  %u Sectors  %u Bytes\n", (//                   DriveNumber) as usize, (//                   Geometry.Cylinders - 1) as usize, (//                   Geometry.Heads - 1) as usize, (//                   Geometry.SectorsPerTrack) as usize, (//                   Geometry.BytesPerSector) as usize);
//         }
//     }
//
//     /* Update the 'System' key's configuration data with BIOS INT13h information */
//     FldrSetConfigurationData(SystemKey, PartialResourceList, Size);
//
//     /* Create and fill subkey for each harddisk */
//     for (i = 0; i < DiskCount; i++)
//     {
//         PCSTR Identifier;
//
//         DriveNumber = 0x80 + i;
//
//         /* Get disk values */
//         PartialResourceList = GetHarddiskConfigurationData(DriveNumber, &Size);
//         Identifier = GetHarddiskIdentifier(DriveNumber);
//
//         /* Create disk key */
//         FldrCreateComponentKey(ControllerKey,
//                                PeripheralClass,
//                                DiskPeripheral,
//                                Output | Input,
//                                i,
//                                0xFFFFFFFF,
//                                Identifier,
//                                PartialResourceList,
//                                Size,
//                                &DiskKey);
//     }
// }
//
// VOID
// FrLdrCheckCpuCompatibility(VOID)
// {
//     INT CpuInformation[4] = {-1};
//     ULONG NumberOfIds;
//
//     /* Check if the processor first supports ID 1 */
//     __cpuid(CpuInformation, 0);
//
//     NumberOfIds = CpuInformation[0];
//
//     if (NumberOfIds == 0)
//     {
//         FrLdrBugCheckWithMessage(MISSING_HARDWARE_REQUIREMENTS,
//                                  __FILE__,
//                                  __LINE__,
//                                  "ReactOS requires the CPUID instruction to return "
//                                  "more than one supported ID.\n\n");
//     }
//
//     /* NumberOfIds will be greater than 1 if the processor is new enough */
//     if (NumberOfIds == 1)
//     {
//         INT ProcessorFamily;
//
//         /* Get information */
//         __cpuid(CpuInformation, 1);
//
//         ProcessorFamily = (CpuInformation[0] >> 8) & 0xF;
//
//         /* If it's Family 4 or lower, bugcheck */
//         if (ProcessorFamily < 5)
//         {
//             FrLdrBugCheckWithMessage(MISSING_HARDWARE_REQUIREMENTS,
//                                      __FILE__,
//                                      __LINE__,
//                                      "Processor is too old (family %u < 5)\n"
//                                      "ReactOS requires a Pentium-level processor or newer.",
//                                      ProcessorFamily);
//         }
//     }
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
