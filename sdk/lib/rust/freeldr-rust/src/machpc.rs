#[allow(unused_imports)] use crate::{ERR, TRACE, WARN};
#[allow(dead_code, non_upper_case_globals)] const DbgDefaultChannel: u32 = crate::frldr::DPRINT_HWDETECT; // machpc.c: DBG_DEFAULT_CHANNEL(HWDETECT)
use ntkrnl_sys::base::*;
use crate::frldr::*;  // supplies the ONE correct TAG_HW_RESOURCE_LIST
use ntkrnl_sys::ddk::{CM_PARTIAL_RESOURCE_DESCRIPTOR, CM_PARTIAL_RESOURCE_LIST, KAFFINITY, PCM_DISK_GEOMETRY_DEVICE_DATA, PCM_KEYBOARD_DEVICE_DATA, PCM_PARTIAL_RESOURCE_DESCRIPTOR, PCM_PARTIAL_RESOURCE_LIST, PCM_PNP_BIOS_DEVICE_NODE, PCM_PNP_BIOS_INSTALLATION_CHECK, PCM_SERIAL_DEVICE_DATA, PCONFIGURATION_COMPONENT_DATA, PCSTR};
// freeldr_ddk_imports dropped (not in ddk): BusNumber, PortNumber
use core::ffi::c_void;
pub use ntkrnl_sys::ddk::_CONFIGURATION_CLASS_AdapterClass as AdapterClass;
pub use ntkrnl_sys::ddk::_CM_SHARE_DISPOSITION_CmResourceShareDeviceExclusive as CmResourceShareDeviceExclusive;
pub use ntkrnl_sys::ddk::_CM_SHARE_DISPOSITION_CmResourceShareUndetermined as CmResourceShareUndetermined;
pub use ntkrnl_sys::ddk::_IDENTIFIER_FLAG_ConsoleIn as ConsoleIn;
pub use ntkrnl_sys::ddk::_IDENTIFIER_FLAG_ConsoleOut as ConsoleOut;
pub use ntkrnl_sys::ddk::_CONFIGURATION_CLASS_ControllerClass as ControllerClass;
pub use ntkrnl_sys::ddk::_CONFIGURATION_TYPE_DisplayController as DisplayController;
pub use ntkrnl_sys::ddk::_CONFIGURATION_TYPE_DockingInformation as DockingInformation;
pub use ntkrnl_sys::ddk::_IDENTIFIER_FLAG_Input as Input;
pub use ntkrnl_sys::ddk::_CONFIGURATION_TYPE_KeyboardController as KeyboardController;
pub use ntkrnl_sys::ddk::_CONFIGURATION_TYPE_KeyboardPeripheral as KeyboardPeripheral;
pub use ntkrnl_sys::ddk::_CONFIGURATION_TYPE_MultiFunctionAdapter as MultiFunctionAdapter;
pub use ntkrnl_sys::ddk::_CONFIGURATION_TYPE_ParallelController as ParallelController;
pub use ntkrnl_sys::ddk::_CONFIGURATION_CLASS_PeripheralClass as PeripheralClass;
pub use ntkrnl_sys::ddk::_CONFIGURATION_TYPE_PointerController as PointerController;
pub use ntkrnl_sys::ddk::_CONFIGURATION_TYPE_PointerPeripheral as PointerPeripheral;
pub use ntkrnl_sys::ddk::_CONFIGURATION_TYPE_SerialController as SerialController;
use crate::ui_keystone::{VIDEODISPLAYMODE, TIMEINFO};

// machpc.c dropped content (rustconv drops non-fn).
const MOUSE_TYPE_NONE: i32 = 0;
const MOUSE_TYPE_MICROSOFT: i32 = 1;
const MOUSE_TYPE_LOGITECH: i32 = 2;
const MOUSE_TYPE_WHEELZ: i32 = 3;
const MOUSE_TYPE_MOUSESYSTEMS: i32 = 4;
// arc.h IDENTIFIER_FLAG bit (sibling of ddk Input/ConsoleIn/ConsoleOut).
const Output: i32 = 0x40;

// ddk CM_* device-data types (bindgen names them _CM_*; bare aliases).
pub use ntkrnl_sys::ddk::_CM_DISK_GEOMETRY_DEVICE_DATA as CM_DISK_GEOMETRY_DEVICE_DATA;
pub use ntkrnl_sys::ddk::_CM_KEYBOARD_DEVICE_DATA as CM_KEYBOARD_DEVICE_DATA;
pub use ntkrnl_sys::ddk::_CM_SERIAL_DEVICE_DATA as CM_SERIAL_DEVICE_DATA;
pub use ntkrnl_sys::ddk::_CM_PNP_BIOS_INSTALLATION_CHECK as CM_PNP_BIOS_INSTALLATION_CHECK;
pub use ntkrnl_sys::ddk::_CM_PARTIAL_RESOURCE_LIST as _CM_PARTIAL_RESOURCE_LIST;
pub use ntkrnl_sys::ddk::_CM_PARTIAL_RESOURCE_DESCRIPTOR as _CM_PARTIAL_RESOURCE_DESCRIPTOR;

// machpc.c file-local #define constants (dropped by rustconv).
const MAX_COM_PORTS: ULONG = 4;
const MAX_LPT_PORTS: ULONG = 3;
// I/O-port addresses (used in `IoBase + X` arithmetic / cast to PUCHAR) → ULONG.
const INPORT_REGISTER_CONTROL: ULONG = 0x00;
const INPORT_REGISTER_DATA: ULONG = 0x01;
const INPORT_REGISTER_SIGNATURE: ULONG = 0x02;
// Port data values written/compared as bytes → UCHAR.
const INPORT_REG_MODE: UCHAR = 0x07;
const INPORT_RESET: UCHAR = 0x80;
const INPORT_MODE_BASE: UCHAR = 0x10;
const INPORT_TEST_IRQ: UCHAR = 0x16;
const INPORT_SIGNATURE: UCHAR = 0xDE;
const PIC1_CONTROL_PORT: ULONG = 0x20;
const PIC1_DATA_PORT: ULONG = 0x21;
const PIC2_CONTROL_PORT: ULONG = 0xA0;
const PIC2_DATA_PORT: ULONG = 0xA1;
const CONTROLLER_REGISTER_STATUS: ULONG = 0x64;
const CONTROLLER_REGISTER_CONTROL: ULONG = 0x64;
const CONTROLLER_REGISTER_DATA: ULONG = 0x60;
const CONTROLLER_COMMAND_READ_MODE: UCHAR = 0x20;
const CONTROLLER_COMMAND_WRITE_MODE: UCHAR = 0x60;
const CONTROLLER_COMMAND_GET_VERSION: UCHAR = 0xA1;
const CONTROLLER_COMMAND_MOUSE_DISABLE: UCHAR = 0xA7;
const CONTROLLER_COMMAND_MOUSE_ENABLE: UCHAR = 0xA8;
const CONTROLLER_COMMAND_TEST_MOUSE: UCHAR = 0xA9;
const CONTROLLER_COMMAND_SELF_TEST: UCHAR = 0xAA;
const CONTROLLER_COMMAND_KEYBOARD_TEST: UCHAR = 0xAB;
const CONTROLLER_COMMAND_KEYBOARD_DISABLE: UCHAR = 0xAD;
const CONTROLLER_COMMAND_KEYBOARD_ENABLE: UCHAR = 0xAE;
const CONTROLLER_COMMAND_WRITE_MOUSE_OUTPUT_BUFFER: UCHAR = 0xD3;
const CONTROLLER_COMMAND_WRITE_MOUSE: UCHAR = 0xD4;
const CONTROLLER_STATUS_OUTPUT_BUFFER_FULL: UCHAR = 0x01;
const CONTROLLER_STATUS_INPUT_BUFFER_FULL: UCHAR = 0x02;
const CONTROLLER_STATUS_SELF_TEST: UCHAR = 0x04;
const CONTROLLER_STATUS_COMMAND: UCHAR = 0x08;
const CONTROLLER_STATUS_UNLOCKED: UCHAR = 0x10;
const CONTROLLER_STATUS_MOUSE_OUTPUT_BUFFER_FULL: UCHAR = 0x20;
const CONTROLLER_STATUS_GENERAL_TIMEOUT: UCHAR = 0x40;
const CONTROLLER_STATUS_PARITY_ERROR: UCHAR = 0x80;
const AUX_STATUS_OUTPUT_BUFFER_FULL: UCHAR =
    CONTROLLER_STATUS_OUTPUT_BUFFER_FULL | CONTROLLER_STATUS_MOUSE_OUTPUT_BUFFER_FULL;
const CONTROLLER_TIMEOUT: ULONG = 250;

// pcbios.h _DOCKING_STATE_INFORMATION (dropped by rustconv).
#[repr(C)]
#[derive(Clone, Copy)]
pub struct DOCKING_STATE_INFORMATION {
    pub DockLocationID: ULONG,
    pub SerialNumber: ULONG,
    pub Capabilities: USHORT,
    pub ReturnCode: USHORT,
}
pub type PDOCKING_STATE_INFORMATION = *mut DOCKING_STATE_INFORMATION;

// machpc.c file-local _PNP_DOCK_INFO (dropped by rustconv).
#[repr(C)]
#[derive(Clone, Copy)]
pub struct PNP_DOCK_INFO {
    pub DockLocationID: ULONG,
    pub SerialNumber: ULONG,
    pub Capabilities: USHORT,
}
pub type PPNP_DOCK_INFO = *mut PNP_DOCK_INFO;

// hardware.h GET_SERIAL_PORT function-pointer typedef (dropped by rustconv).
pub type GET_SERIAL_PORT =
    Option<unsafe extern "C" fn(Index: ULONG, Irq: PULONG) -> ULONG>;

// cmtypes.h CM resource constants. Typed to the CM_PARTIAL_RESOURCE_DESCRIPTOR
// field types (Type: UCHAR, Flags: USHORT) so direct assignment is exact.
const CmResourceTypePort: UCHAR = 1;
const CmResourceTypeInterrupt: UCHAR = 2;
const CmResourceTypeDeviceSpecific: UCHAR = 5;
const CM_RESOURCE_PORT_IO: USHORT = 0x0001;
const CM_RESOURCE_INTERRUPT_LATCHED: USHORT = 0x0001;
// i386Idt is arch/i386/i386idt.c's `KIDTENTRY i386Idt[32]`. On amd64 the
// ndk macro-aliases KIDTENTRY -> KIDTENTRY64 (rustconv dropped the alias);
// resolve the element type per-arch so the extern decl type-checks on
// both. (The amd64 freeldr_pe mach path does not reach this symbol.)
#[cfg(target_arch = "x86")]
extern "C" {
    static mut i386Idt: [ntkrnl_sys::ddk::KIDTENTRY; 32];
}
#[cfg(target_arch = "x86_64")]
extern "C" {
    static mut i386Idt: [ntkrnl_sys::ddk::_KIDTENTRY64; 32];
}

// Full machine.h MACHVTBL (37 slots) — machpc.c is the OWNER (defines the
// global + fills it in MachInit). File-local: shadows the keystone's
// deliberately-opaque consumer-side MACHVTBL within this module. #[repr(C)]
// machine.h field order; layout-compatible with the prior C definition so
// consumers (keystone opaque view) are unaffected.
#[repr(C)]
pub struct MACHVTBL {
    pub ConsPutChar: Option<unsafe extern "C" fn(i32)>,
    pub ConsKbHit: Option<unsafe extern "C" fn() -> BOOLEAN>,
    pub ConsGetCh: Option<unsafe extern "C" fn() -> i32>,
    pub VideoClearScreen: Option<unsafe extern "C" fn(UCHAR)>,
    pub VideoSetDisplayMode: Option<unsafe extern "C" fn(PCSTR, BOOLEAN) -> VIDEODISPLAYMODE>,
    pub VideoGetDisplaySize: Option<unsafe extern "C" fn(PULONG, PULONG, PULONG)>,
    pub VideoGetBufferSize: Option<unsafe extern "C" fn() -> ULONG>,
    pub VideoGetFontsFromFirmware: Option<unsafe extern "C" fn(PULONG)>,
    pub VideoSetTextCursorPosition: Option<unsafe extern "C" fn(UCHAR, UCHAR)>,
    pub VideoHideShowTextCursor: Option<unsafe extern "C" fn(BOOLEAN)>,
    pub VideoPutChar: Option<unsafe extern "C" fn(i32, UCHAR, ::core::ffi::c_uint, ::core::ffi::c_uint)>,
    pub VideoCopyOffScreenBufferToVRAM: Option<unsafe extern "C" fn(PVOID)>,
    pub VideoIsPaletteFixed: Option<unsafe extern "C" fn() -> BOOLEAN>,
    pub VideoSetPaletteColor: Option<unsafe extern "C" fn(UCHAR, UCHAR, UCHAR, UCHAR)>,
    pub VideoGetPaletteColor: Option<unsafe extern "C" fn(UCHAR, *mut UCHAR, *mut UCHAR, *mut UCHAR)>,
    pub VideoSync: Option<unsafe extern "C" fn()>,
    pub Beep: Option<unsafe extern "C" fn()>,
    pub PrepareForReactOS: Option<unsafe extern "C" fn()>,
    pub GetMemoryDescriptor: Option<unsafe extern "C" fn(*mut FREELDR_MEMORY_DESCRIPTOR) -> *mut FREELDR_MEMORY_DESCRIPTOR>,
    pub GetMemoryMap: Option<unsafe extern "C" fn(PULONG) -> PFREELDR_MEMORY_DESCRIPTOR>,
    pub GetExtendedBIOSData: Option<unsafe extern "C" fn(PULONG, PULONG)>,
    pub GetFloppyCount: Option<unsafe extern "C" fn() -> UCHAR>,
    pub DiskReadLogicalSectors: Option<unsafe extern "C" fn(UCHAR, ULONGLONG, ULONG, PVOID) -> BOOLEAN>,
    pub DiskGetDriveGeometry: Option<unsafe extern "C" fn(UCHAR, PGEOMETRY) -> BOOLEAN>,
    pub DiskGetCacheableBlockCount: Option<unsafe extern "C" fn(UCHAR) -> ULONG>,
    pub GetTime: Option<unsafe extern "C" fn() -> *mut TIMEINFO>,
    pub GetRelativeTime: Option<unsafe extern "C" fn() -> ULONG>,
    pub InitializeBootDevices: Option<unsafe extern "C" fn() -> BOOLEAN>,
    pub HwDetect: Option<unsafe extern "C" fn(PCSTR) -> PCONFIGURATION_COMPONENT_DATA>,
    pub HwIdle: Option<unsafe extern "C" fn()>,
}
unsafe impl Sync for MACHVTBL {}

// MachVtbl global is DEFINED by C arch/arcemul.c (`MACHVTBL MachVtbl;`,
// still-compiled, smoke-path). machpc.c only FILLS it in MachInit.
// Extern-consume the C-owned global (do NOT #[no_mangle]-define it —
// that dup'd arcemul.c at cold link). Layout matches: machpc's MACHVTBL
// is the faithful machine.h 37-slot struct == arcemul.c's typedef.
extern "C" {
    static mut MachVtbl: MACHVTBL;
}

// machine.h `#define Mach<X>(...) MachVtbl.<X>(...)` macros. rustconv emitted
// these as `extern { fn Mach<X> }` + call sites; no C symbol ever existed.
// Real #[no_mangle] cdecl wrappers dispatching through MachVtbl. Signatures
// exactly match machine.h tagMACHVTBL slot fn-ptr decls.
#[no_mangle]
pub unsafe extern "C" fn MachConsPutChar(Ch: i32) {
    (MachVtbl.ConsPutChar.unwrap())(Ch)
}
#[no_mangle]
pub unsafe extern "C" fn MachConsKbHit() -> BOOLEAN {
    (MachVtbl.ConsKbHit.unwrap())()
}
#[no_mangle]
pub unsafe extern "C" fn MachConsGetCh() -> i32 {
    (MachVtbl.ConsGetCh.unwrap())()
}
#[no_mangle]
pub unsafe extern "C" fn MachVideoClearScreen(Attr: UCHAR) {
    (MachVtbl.VideoClearScreen.unwrap())(Attr)
}
#[no_mangle]
pub unsafe extern "C" fn MachVideoSetDisplayMode(
    DisplayMode: PCSTR,
    Init: BOOLEAN,
) -> VIDEODISPLAYMODE {
    (MachVtbl.VideoSetDisplayMode.unwrap())(DisplayMode, Init)
}
#[no_mangle]
pub unsafe extern "C" fn MachVideoGetBufferSize() -> ULONG {
    (MachVtbl.VideoGetBufferSize.unwrap())()
}
#[no_mangle]
pub unsafe extern "C" fn MachVideoSetTextCursorPosition(X: UCHAR, Y: UCHAR) {
    (MachVtbl.VideoSetTextCursorPosition.unwrap())(X, Y)
}
#[no_mangle]
pub unsafe extern "C" fn MachVideoHideShowTextCursor(Show: BOOLEAN) {
    (MachVtbl.VideoHideShowTextCursor.unwrap())(Show)
}
#[no_mangle]
pub unsafe extern "C" fn MachVideoCopyOffScreenBufferToVRAM(Buffer: PVOID) {
    (MachVtbl.VideoCopyOffScreenBufferToVRAM.unwrap())(Buffer)
}
#[no_mangle]
pub unsafe extern "C" fn MachVideoIsPaletteFixed() -> BOOLEAN {
    (MachVtbl.VideoIsPaletteFixed.unwrap())()
}
#[no_mangle]
pub unsafe extern "C" fn MachVideoSetPaletteColor(
    Color: UCHAR,
    Red: UCHAR,
    Green: UCHAR,
    Blue: UCHAR,
) {
    (MachVtbl.VideoSetPaletteColor.unwrap())(Color, Red, Green, Blue)
}
#[no_mangle]
pub unsafe extern "C" fn MachVideoGetPaletteColor(
    Color: UCHAR,
    Red: *mut UCHAR,
    Green: *mut UCHAR,
    Blue: *mut UCHAR,
) {
    (MachVtbl.VideoGetPaletteColor.unwrap())(Color, Red, Green, Blue)
}
#[no_mangle]
pub unsafe extern "C" fn MachVideoSync() {
    (MachVtbl.VideoSync.unwrap())()
}
#[no_mangle]
pub unsafe extern "C" fn MachBeep() {
    (MachVtbl.Beep.unwrap())()
}
#[no_mangle]
pub unsafe extern "C" fn MachHwIdle() {
    (MachVtbl.HwIdle.unwrap())()
}
#[no_mangle]
pub unsafe extern "C" fn MachVideoGetDisplaySize(
    Width: PULONG,
    Height: PULONG,
    Depth: PULONG,
) {
    (MachVtbl.VideoGetDisplaySize.unwrap())(Width, Height, Depth)
}
#[no_mangle]
pub unsafe extern "C" fn MachGetFloppyCount() -> UCHAR {
    (MachVtbl.GetFloppyCount.unwrap())()
}
#[no_mangle]
pub unsafe extern "C" fn MachDiskReadLogicalSectors(
    DriveNumber: UCHAR,
    SectorNumber: ULONGLONG,
    SectorCount: ULONG,
    Buffer: PVOID,
) -> BOOLEAN {
    (MachVtbl.DiskReadLogicalSectors.unwrap())(DriveNumber, SectorNumber, SectorCount, Buffer)
}
#[no_mangle]
pub unsafe extern "C" fn MachDiskGetDriveGeometry(
    DriveNumber: UCHAR,
    DriveGeometry: PGEOMETRY,
) -> BOOLEAN {
    (MachVtbl.DiskGetDriveGeometry.unwrap())(DriveNumber, DriveGeometry)
}
#[no_mangle]
pub unsafe extern "C" fn MachDiskGetCacheableBlockCount(DriveNumber: UCHAR) -> ULONG {
    (MachVtbl.DiskGetCacheableBlockCount.unwrap())(DriveNumber)
}
#[no_mangle]
pub unsafe extern "C" fn MachVideoPutChar(
    Ch: i32,
    Attr: UCHAR,
    X: ::core::ffi::c_uint,
    Y: ::core::ffi::c_uint,
) {
    (MachVtbl.VideoPutChar.unwrap())(Ch, Attr, X, Y)
}

extern "C" {
    fn DbgPrint(fmt: *const i8, ...) -> i32;
}

extern "C" {
    fn READ_PORT_UCHAR(Port: PUCHAR) -> UCHAR;
    fn WRITE_PORT_UCHAR(Port: PUCHAR, Value: UCHAR);
    fn StallExecutionProcessor(Microseconds: ULONG);
    fn RtlStringCbPrintfA(dest: *mut i8, size: usize, fmt: *const i8, ...) -> i32;
    fn FrLdrHeapAlloc(size: usize, tag: u32) -> *mut core::ffi::c_void;
    fn FldrCreateComponentKey( parent: PCONFIGURATION_COMPONENT_DATA, class: u32, type_: u32, flags: u32, key: u32, affinity: u32, identifier: *const i8, resource_list: PCM_PARTIAL_RESOURCE_LIST, size: u32, component_key: *mut PCONFIGURATION_COMPONENT_DATA, );
    fn BiosIsVesaSupported() -> u16;
    fn BiosIsVesaDdcSupported() -> u8;
    fn BiosVesaReadEdid() -> u8;
    fn Int386(interrupt: i32, in_regs: *mut REGS, out_regs: *mut REGS);
    fn INT386_SUCCESS(regs: REGS) -> u8;
    fn Relocator16Boot( Regs: *mut REGS, StackSegment: u16, StackPointer: u16, CodeSegment: u16, CodePointer: u16, );
    fn PcVideoPrepareForReactOS();
    fn DiskStopFloppyMotor();
    // Cross-module pc-hw functions (defined in sibling .rs / external pc C files;
    // machpc.c references them only as MachVtbl fn-pointer slots). machpc.h /
    // pccons.c / pcrtc.c / pcbeep.c canonical cdecl prototypes.
    fn PcConsPutChar(Ch: i32);
    fn PcConsKbHit() -> BOOLEAN;
    fn PcConsGetCh() -> i32;
    fn PcBeep();
    fn PcVideoClearScreen(Attr: UCHAR);
    fn PcVideoSetDisplayMode(DisplayMode: PCSTR, Init: BOOLEAN) -> VIDEODISPLAYMODE;
    fn PcVideoGetDisplaySize(Width: PULONG, Height: PULONG, Depth: PULONG);
    fn PcVideoGetBufferSize() -> ULONG;
    fn PcVideoGetFontsFromFirmware(RomFontPointers: PULONG);
    fn PcVideoSetTextCursorPosition(X: UCHAR, Y: UCHAR);
    fn PcVideoHideShowTextCursor(Show: BOOLEAN);
    fn PcVideoPutChar(Ch: i32, Attr: UCHAR, X: u32, Y: u32);
    fn PcVideoCopyOffScreenBufferToVRAM(Buffer: PVOID);
    fn PcVideoIsPaletteFixed() -> BOOLEAN;
    fn PcVideoSetPaletteColor(Color: UCHAR, Red: UCHAR, Green: UCHAR, Blue: UCHAR);
    fn PcVideoGetPaletteColor(Color: UCHAR, Red: *mut UCHAR, Green: *mut UCHAR, Blue: *mut UCHAR);
    fn PcVideoSync();
    fn PcVideoInit();
    fn PcMemGetMemoryMap(MemoryMapSize: *mut ULONG) -> PFREELDR_MEMORY_DESCRIPTOR;
    fn PcDiskReadLogicalSectors(DriveNumber: UCHAR, SectorNumber: ULONGLONG, SectorCount: ULONG, Buffer: PVOID) -> BOOLEAN;
    fn PcDiskGetDriveGeometry(DriveNumber: UCHAR, DriveGeometry: PGEOMETRY) -> BOOLEAN;
    fn PcDiskGetCacheableBlockCount(DriveNumber: UCHAR) -> ULONG;
    fn PcGetTime() -> *mut TIMEINFO;
    fn HalpCalibrateStallExecution();
    // Platform extern globals (defined in external freeldr C: uefidisk.c /
    // hwdisk.c — UCHAR/ULONG per machpc.h / pchw.c declarations).
    static FrldrBootDrive: UCHAR;
    static FrldrBootPartition: ULONG;
    static PcBiosDiskCount: UCHAR;
    fn memcpy(dest: *mut core::ffi::c_void, src: *const core::ffi::c_void, n: usize) -> *mut core::ffi::c_void;
    fn strcpy(dest: *mut i8, src: *const i8) -> *mut i8;
    fn strlen(s: *const i8) -> usize;
    fn UlongToPtr(ul: ULONG) -> *mut core::ffi::c_void;
    fn DetectBiosDisks(SystemKey: PCONFIGURATION_COMPONENT_DATA, BusKey: PCONFIGURATION_COMPONENT_DATA);
    fn FldrCreateSystemKey(SystemKey: *mut PCONFIGURATION_COMPONENT_DATA, Identifier: PCSTR);
    fn DetectPciBios(SystemKey: PCONFIGURATION_COMPONENT_DATA, BusNumber: *mut ULONG);
    fn DetectApmBios(SystemKey: PCONFIGURATION_COMPONENT_DATA, BusNumber: *mut ULONG);
    fn DetectAcpiBios(SystemKey: PCONFIGURATION_COMPONENT_DATA, BusNumber: *mut ULONG);
    static mut GetHarddiskConfigurationData: *mut core::ffi::c_void;
    static mut FindPciBios: *mut core::ffi::c_void;
    fn PcFindPciBios() -> *mut core::ffi::c_void;
    // pnpbios.c (hardware.h __cdecl prototypes).
    fn PnpBiosSupported() -> ULONG_PTR;
    fn PnpBiosGetDeviceNodeCount(NodeSize: *mut ULONG, NodeCount: *mut ULONG) -> ULONG;
    fn PnpBiosGetDeviceNode(NodeId: *mut UCHAR, NodeBuffer: *mut UCHAR) -> ULONG;
    fn PnpBiosGetDockStationInformation(DockingStationInfo: *mut UCHAR) -> ULONG;
    // machpc.c i386 IRQ-detect asm symbols. HwIrqHandler: __cdecl fn;
    // HwIrqCount: volatile ULONG (both from the IRQ stub asm).
    fn HwIrqHandler();
    static mut HwIrqCount: ULONG;
    // mm.h heap free + crt strncmp + hwdisk.c PcInitializeBootDevices.
    fn FrLdrHeapFree(MemoryPointer: PVOID, Tag: ULONG);
    fn strncmp(s1: *const i8, s2: *const i8, n: usize) -> i32;
    fn PcInitializeBootDevices() -> BOOLEAN;
}

// MSVC _enable/_disable intrinsics (sti/cli) — bus-mouse IRQ probe.
#[inline(always)]
unsafe fn _enable() {
    core::arch::asm!("sti", options(nomem, nostack, preserves_flags));
}
#[inline(always)]
unsafe fn _disable() {
    core::arch::asm!("cli", options(nomem, nostack, preserves_flags));
}

// FREELDR spec marks these as cdecl (`@ cdecl NtLdrGetOption()`); the Rust
// impl in ntldropts.rs is `extern "C"`. The previous `extern "system"` here
// was stdcall on i386 → callee-cleans 8 bytes of args after `ret`, but cdecl
// callee does NOT clean → caller's ESP ends up 8 bytes too low after each
// NtLdrGetOption call. Subsequent ESP-relative local writes overwrote saved
// EBP/RIP, causing the wrapper to `ret` to garbage 0x10006 (rosload PE
// header bytes) on i386.
extern "C" {
    // Returns the option's PCSTR (ntldropts.c). Read as a byte, an option found at an
    // address whose low byte is 0 reads "absent"; only the pointer width is safe.
    fn NtLdrGetOption(Options: PCSTR, Name: PCSTR) -> usize;
    fn NtLdrGetOptionEx(Options: PCSTR, Name: PCSTR, OptionLength: *mut u32) -> PCSTR;
}

// stdcall on i386 — these are stdcall in freeldr.spec / cportlib.
// (CpDoesPortExist was previously also declared in the `extern "C"` block
// above as cdecl, but the actual symbol is `_CpDoesPortExist@4`. Same class
// of ABI mismatch as the NtLdrGetOption bug; was latent because heap
// corruption masked it earlier in boot.)
extern "system" {
    fn CpDoesPortExist(Port: PUCHAR) -> BOOLEAN;
}
unsafe extern "C" {
    fn Rs232PortInUse(Port: PUCHAR) -> BOOLEAN;
}

// NOTE: REGS / BYTEREGS / WORDREGS / DWORDREGS come from the frldr keystone
// (`use crate::frldr::*`) — the full `union REGS` with x/d/w/b views. The
// earlier rustconv-emitted stub `struct REGS { b: REGS_B }` was incomplete
// (missing .d/.w/.x and dl/dh) and shadowed the keystone; removed.

unsafe extern "C" fn PcGetExtendedBIOSData(ExtendedBIOSDataArea: PULONG, ExtendedBIOSDataSize: PULONG) {
    let mut BiosRegs: REGS = core::mem::zeroed();

    BiosRegs.d.eax = 0xC100;
    Int386(0x15, &mut BiosRegs, &mut BiosRegs);
    if INT386_SUCCESS(BiosRegs) != 0 {
        *ExtendedBIOSDataArea = (BiosRegs.w.es as u32) << 4;
        *ExtendedBIOSDataSize = 1024;
    } else {
        WARN!("Int 15h AH=C1h call failed\n");
        *ExtendedBIOSDataArea = 0;
        *ExtendedBIOSDataSize = 0;
    }
}

unsafe fn PcGetHarddiskConfigurationData(DriveNumber: UCHAR, pSize: *mut ULONG) -> PCM_PARTIAL_RESOURCE_LIST {
    let PartialResourceList: PCM_PARTIAL_RESOURCE_LIST;
    let DiskGeometry: PCM_DISK_GEOMETRY_DEVICE_DATA;
    let mut Geometry: GEOMETRY = core::mem::zeroed();
    let Size: ULONG;

    *pSize = 0;

    Size = crate::FIELD_OFFSET!(CM_PARTIAL_RESOURCE_LIST, PartialDescriptors[1]) + core::mem::size_of::<CM_DISK_GEOMETRY_DEVICE_DATA>() as ULONG;
    PartialResourceList = FrLdrHeapAlloc((Size) as usize , TAG_HW_RESOURCE_LIST) as PCM_PARTIAL_RESOURCE_LIST;
    if PartialResourceList.is_null() {
        ERR!("Failed to allocate resource descriptor\n");
        return core::ptr::null_mut();
    }

    RtlZeroMemory(PartialResourceList as *mut core::ffi::c_void, Size as usize);
    (*PartialResourceList).Version = 1;
    (*PartialResourceList).Revision = 1;
    (*PartialResourceList).Count = 1;
    (*PartialResourceList).PartialDescriptors[0].Type = CmResourceTypeDeviceSpecific;
    (*PartialResourceList).PartialDescriptors[0].u.DeviceSpecificData.DataSize = core::mem::size_of::<CM_DISK_GEOMETRY_DEVICE_DATA>() as ULONG;

    DiskGeometry = (core::ptr::addr_of_mut!((*PartialResourceList).PartialDescriptors) as *mut _CM_PARTIAL_RESOURCE_DESCRIPTOR).add(1) as PCM_DISK_GEOMETRY_DEVICE_DATA;
    if PcDiskGetDriveGeometry(DriveNumber, &mut Geometry) != 0 {
        (*DiskGeometry).BytesPerSector = Geometry.BytesPerSector;
        (*DiskGeometry).NumberOfCylinders = Geometry.Cylinders;
        (*DiskGeometry).SectorsPerTrack = Geometry.SectorsPerTrack;
        (*DiskGeometry).NumberOfHeads = Geometry.Heads;
    } else {
        TRACE!("Reading disk geometry failed\n");
        FrLdrHeapFree(PartialResourceList as *mut core::ffi::c_void, TAG_HW_RESOURCE_LIST);
        return core::ptr::null_mut();
    }
    TRACE!("Disk %x: %u Cylinders  %u Heads  %u Sectors  %u Bytes\n", (DriveNumber) as usize, ((*DiskGeometry).NumberOfCylinders) as usize, ((*DiskGeometry).NumberOfHeads) as usize, ((*DiskGeometry).SectorsPerTrack) as usize, ((*DiskGeometry).BytesPerSector) as usize);

    *pSize = Size;
    return PartialResourceList;
}

unsafe fn DetectDockingStation(
    BusKey: PCONFIGURATION_COMPONENT_DATA)
{
    let PartialResourceList: PCM_PARTIAL_RESOURCE_LIST;
    let PartialDescriptor: PCM_PARTIAL_RESOURCE_DESCRIPTOR;
    let mut PeripheralKey: PCONFIGURATION_COMPONENT_DATA = core::ptr::null_mut();
    let DockingState: PDOCKING_STATE_INFORMATION;
    let DockInfo: PPNP_DOCK_INFO;
    let Size: ULONG;
    let Result: ULONG;

    Result = PnpBiosGetDockStationInformation(DiskReadBuffer as *mut UCHAR);

    Size = crate::FIELD_OFFSET!(CM_PARTIAL_RESOURCE_LIST, PartialDescriptors[1]) + core::mem::size_of::<DOCKING_STATE_INFORMATION>() as ULONG;
    PartialResourceList = FrLdrHeapAlloc((Size) as usize , TAG_HW_RESOURCE_LIST) as PCM_PARTIAL_RESOURCE_LIST;
    if PartialResourceList == core::ptr::null_mut()
    {
        ERR!("Failed to allocate resource descriptor\n");
        return;
    }

    RtlZeroMemory(PartialResourceList as *mut core::ffi::c_void, (Size) as usize );
    (*PartialResourceList).Version = 0;
    (*PartialResourceList).Revision = 0;
    (*PartialResourceList).Count = 1;

    PartialDescriptor = core::ptr::addr_of_mut!((*PartialResourceList).PartialDescriptors[0]);
    (*PartialDescriptor).Type = CmResourceTypeDeviceSpecific;
    (*PartialDescriptor).ShareDisposition = (CmResourceShareUndetermined) as u8 ;
    (*PartialDescriptor).Flags = 0;
    (*PartialDescriptor).u.DeviceSpecificData.DataSize = core::mem::size_of::<DOCKING_STATE_INFORMATION>() as ULONG;

    DockingState = PartialDescriptor.add(1) as PDOCKING_STATE_INFORMATION;
    (*DockingState).ReturnCode = (Result) as u16 ;
    if Result == 0
    {
        DockInfo = DiskReadBuffer as PPNP_DOCK_INFO;
        (*DockingState).DockLocationID = (*DockInfo).DockLocationID;
        (*DockingState).SerialNumber = (*DockInfo).SerialNumber;
        (*DockingState).Capabilities = (*DockInfo).Capabilities;
        TRACE!("System docked\n");
        TRACE!("Location: 0x%08lx\n", ((*DockInfo).DockLocationID) as usize);
        TRACE!("Serial: 0x%08lx\n", ((*DockInfo).SerialNumber) as usize);
        TRACE!("Capabilities: 0x%04hx\n", ((*DockInfo).Capabilities) as usize);
    }

    FldrCreateComponentKey(BusKey,
                           PeripheralClass,
                           DockingInformation,
                           0,
                           0,
                           0xFFFFFFFF,
                           b"Docking State Information\0".as_ptr() as *const i8,
                           PartialResourceList,
                           Size,
                           core::ptr::addr_of_mut!(PeripheralKey));
}

unsafe fn DetectPnpBios(SystemKey: PCONFIGURATION_COMPONENT_DATA, BusNumber: *mut ULONG) {
    let PartialResourceList: PCM_PARTIAL_RESOURCE_LIST;
    let mut DeviceNode: PCM_PNP_BIOS_DEVICE_NODE;
    let InstData: PCM_PNP_BIOS_INSTALLATION_CHECK;
    let mut BusKey: PCONFIGURATION_COMPONENT_DATA = core::ptr::null_mut();
    let mut x: ULONG;
    let mut NodeSize: ULONG = 0;
    let mut NodeCount: ULONG = 0;
    let mut NodeNumber: UCHAR;
    let mut FoundNodeCount: ULONG;
    let mut i: i32;
    let mut PnpBufferSize: ULONG;
    let PnpBufferSizeLimit: ULONG;
    let mut Size: ULONG;
    let mut Ptr: *mut u8;

    InstData = PnpBiosSupported() as PCM_PNP_BIOS_INSTALLATION_CHECK;
    if InstData == core::ptr::null_mut() || strncmp((*InstData).Signature.as_ptr() as *const i8, b"$PnP\0".as_ptr() as *const i8, 4) != 0 {
        TRACE!("PnP-BIOS not supported\n");
        return;
    }

    TRACE!("PnP-BIOS supported\n");
    TRACE!("Signature '%c%c%c%c'\n", ((*InstData).Signature[0]) as usize, ((*InstData).Signature[1]) as usize, ((*InstData).Signature[2]) as usize, ((*InstData).Signature[3]) as usize);

    x = PnpBiosGetDeviceNodeCount(core::ptr::addr_of_mut!(NodeSize), core::ptr::addr_of_mut!(NodeCount));
    if x == 0x82 {
        TRACE!("PnP-BIOS function 'Get Number of System Device Nodes' not supported\n");
        return;
    }

    NodeCount &= 0xFF;
    if x != 0 || NodeSize == 0 || NodeCount == 0 {
        ERR!("PnP-BIOS failed to enumerate device nodes\n");
        return;
    }
    TRACE!("MaxNodeSize %u  NodeCount %u\n", (NodeSize) as usize, (NodeCount) as usize);
    TRACE!("Estimated buffer size %u\n", (NodeSize * NodeCount) as usize);

    PnpBufferSizeLimit = core::mem::size_of::<CM_PNP_BIOS_INSTALLATION_CHECK>() as ULONG + (NodeSize * NodeCount);
    Size = crate::FIELD_OFFSET!(CM_PARTIAL_RESOURCE_LIST, PartialDescriptors[1]) + PnpBufferSizeLimit;
    PartialResourceList = FrLdrHeapAlloc((Size) as usize , TAG_HW_RESOURCE_LIST) as PCM_PARTIAL_RESOURCE_LIST;
    if PartialResourceList == core::ptr::null_mut() {
        ERR!("Failed to allocate resource descriptor\n");
        return;
    }

    RtlZeroMemory(PartialResourceList as *mut VOID, (Size) as usize );
    (*PartialResourceList).Version = 1;
    (*PartialResourceList).Revision = 1;
    (*PartialResourceList).Count = 1;
    (*PartialResourceList).PartialDescriptors[0].Type = CmResourceTypeDeviceSpecific;
    (*PartialResourceList).PartialDescriptors[0].ShareDisposition = (CmResourceShareUndetermined) as u8 ;

    Ptr = (*PartialResourceList).PartialDescriptors.as_ptr().add(1) as *mut u8;
    RtlCopyMemory(Ptr as *mut VOID, InstData as *const VOID, core::mem::size_of::<CM_PNP_BIOS_INSTALLATION_CHECK>());
    Ptr = Ptr.add(core::mem::size_of::<CM_PNP_BIOS_INSTALLATION_CHECK>());
    PnpBufferSize = core::mem::size_of::<CM_PNP_BIOS_INSTALLATION_CHECK>() as ULONG;

    FoundNodeCount = 0;
    i = 0;
    while i < 0xFF {
        NodeNumber = i as UCHAR;

        x = PnpBiosGetDeviceNode(core::ptr::addr_of_mut!(NodeNumber), DiskReadBuffer as *mut UCHAR);
        if x == 0 {
            DeviceNode = DiskReadBuffer as PCM_PNP_BIOS_DEVICE_NODE;

            TRACE!("Node: %u  Size %u (0x%x)\n", ((*DeviceNode).Node) as usize, ((*DeviceNode).Size) as usize, ((*DeviceNode).Size) as usize);

            if PnpBufferSize + (((*DeviceNode).Size) as u32) > PnpBufferSizeLimit {
                ERR!("Buffer too small! Ignoring remaining device nodes. (i = %d)\n", (i) as usize);
                break;
            }

            RtlCopyMemory(Ptr as *mut VOID, DeviceNode as *const VOID, (*DeviceNode).Size as usize);

            Ptr = Ptr.add((*DeviceNode).Size as usize);
            PnpBufferSize += ((*DeviceNode).Size) as u32 ;

            FoundNodeCount += 1;
            if FoundNodeCount >= NodeCount {
                break;
            }
        }
        i += 1;
    }

    (*PartialResourceList).PartialDescriptors[0].u.DeviceSpecificData.DataSize = PnpBufferSize;
    Size = crate::FIELD_OFFSET!(CM_PARTIAL_RESOURCE_LIST, PartialDescriptors[1]) + PnpBufferSize;

    TRACE!("Real buffer size: %u\n", (PnpBufferSize) as usize);
    TRACE!("Resource size: %u\n", (Size) as usize);

    FldrCreateComponentKey(SystemKey,
                           AdapterClass,
                           MultiFunctionAdapter,
                           0,
                           0,
                           0xFFFFFFFF,
                           b"PNP BIOS\0".as_ptr() as PCHAR,
                           PartialResourceList,
                           Size,
                           core::ptr::addr_of_mut!(BusKey));

    DetectDockingStation(BusKey);

    *BusNumber += 1;
}

unsafe fn InitializeSerialPort(Port: PUCHAR, LineControl: UCHAR) {
    
    WRITE_PORT_UCHAR(Port.offset(3), 0x80);
    WRITE_PORT_UCHAR(Port, 0x60);
    WRITE_PORT_UCHAR(Port.offset(1), 0);
    WRITE_PORT_UCHAR(Port.offset(3), LineControl);
    WRITE_PORT_UCHAR(Port.offset(1), 0);
    WRITE_PORT_UCHAR(Port.offset(4), 0x09);
    READ_PORT_UCHAR(Port.offset(5));
}

unsafe fn DetectSerialMouse(Port: PUCHAR) -> ULONG {

    let mut Buffer: [CHAR; 4] = [0; 4];
    let mut i: ULONG;
    let mut TimeOut: ULONG;
    let LineControl: UCHAR;

    LineControl = READ_PORT_UCHAR(Port.offset(4));
    WRITE_PORT_UCHAR(Port.offset(4), (LineControl & !0x02) | 0x01);
    StallExecutionProcessor(100000);

    TimeOut = 200;
    while (READ_PORT_UCHAR(Port.offset(5)) & 0x01) != 0 {
        TimeOut -= 1;
        if TimeOut == 0 {
            return (MOUSE_TYPE_NONE) as u32 ;
        }
        READ_PORT_UCHAR(Port);
    }

    WRITE_PORT_UCHAR(Port.offset(4), 0x0b);

    StallExecutionProcessor(10000);

    TimeOut = 20;
    i = 0;
    while i < 4 {
        while (READ_PORT_UCHAR(Port.offset(5)) & 1) == 0 {
            StallExecutionProcessor(100);
            TimeOut -= 1;
            if TimeOut == 0 {
                return (MOUSE_TYPE_NONE) as u32 ;
            }
        }
        Buffer[i as usize] = READ_PORT_UCHAR(Port) as CHAR;
        i += 1;
    }

    TRACE!("Mouse data: %x %x %x %x\n", (Buffer[0]) as usize, (Buffer[1]) as usize, (Buffer[2]) as usize, (Buffer[3]) as usize);

    i = 0;
    while i < 4 {
        if Buffer[i as usize] == b'B' as CHAR {
            return (MOUSE_TYPE_NONE) as u32 ;
        } else if Buffer[i as usize] == b'M' as CHAR {
            if i == 3 {
                return (MOUSE_TYPE_NONE) as u32 ;
            }

            match Buffer[(i + 1) as usize] {
                0x33 => {
                    TRACE!("Microsoft Mouse with 3-buttons detected\n");
                    return (MOUSE_TYPE_LOGITECH) as u32 ;
                }
                0x5a => {
                    TRACE!("Microsoft Wheel Mouse detected\n");
                    return (MOUSE_TYPE_WHEELZ) as u32 ;
                }
                _ => {
                    TRACE!("Microsoft Mouse with 2-buttons detected\n");
                    return (MOUSE_TYPE_MICROSOFT) as u32 ;
                }
            }
        }
        i += 1;
    }

    (MOUSE_TYPE_NONE) as u32
}

unsafe fn GetSerialMousePnpId(Port: PUCHAR, Buffer: *mut i8) -> ULONG {

    let mut TimeOut: ULONG;
    let mut i: ULONG = 0;
    let mut c: i8;
    let x: i8;

    WRITE_PORT_UCHAR(Port.offset(4), 0x09);

    StallExecutionProcessor(10000);

    WRITE_PORT_UCHAR(Port.offset(4), 0x0b);

    StallExecutionProcessor(10000);

    loop {
        TimeOut = 200;
        while ((READ_PORT_UCHAR(Port.offset(5)) & 1) == 0) && (TimeOut > 0) {
            StallExecutionProcessor(1000);
            TimeOut -= 1;
            if TimeOut == 0 {
                return 0;
            }
        }

        c = READ_PORT_UCHAR(Port) as i8;
        if c == 0x08 || c == 0x28 {
            break;
        }
    }

    *Buffer.offset(i as isize) = c;
    i += 1;
    x = c + 1;

    loop {
        TimeOut = 200;
        while ((READ_PORT_UCHAR(Port.offset(5)) & 1) == 0) && (TimeOut > 0) {
            StallExecutionProcessor(1000);
            TimeOut -= 1;
            if TimeOut == 0 {
                return 0;
            }
        }
        c = READ_PORT_UCHAR(Port) as i8;
        *Buffer.offset(i as isize) = c;
        i += 1;
        if c == x {
            break;
        }
        if i >= 256 {
            break;
        }
    }

    i
}

unsafe fn DetectSerialPointerPeripheral(
    ControllerKey: PCONFIGURATION_COMPONENT_DATA,
    Base: PUCHAR,
) {

    let PartialResourceList: PCM_PARTIAL_RESOURCE_LIST;
    let mut Buffer: [i8; 256] = [0; 256];
    let mut Identifier: [CHAR; 256] = [0; 256];
    let mut PeripheralKey: PCONFIGURATION_COMPONENT_DATA = core::ptr::null_mut();
    let MouseType: ULONG;
    let Size: ULONG;
    let Length: ULONG;
    let mut i: ULONG;
    let mut j: ULONG;
    let mut k: ULONG;

    TRACE!("DetectSerialPointerPeripheral()\n");

    Identifier[0] = 0;

    InitializeSerialPort(Base, 2);
    MouseType = DetectSerialMouse(Base);

    if MouseType != 0 {
        Length = GetSerialMousePnpId(Base, Buffer.as_mut_ptr());
        TRACE!("PnP ID length: %u\n", (Length) as usize);

        if Length != 0 {
            if Buffer[0] == 0x08 {
                i = 0;
                while i < Length {
                    Buffer[i as usize] = (Buffer[i as usize] as u8 + 0x20) as i8;
                    i += 1;
                }
            }
            Buffer[Length as usize] = 0;

            TRACE!("PnP ID string: %s\n", (Buffer.as_ptr()) as usize);

            i = 0;
            while i < 7 {
                Identifier[i as usize] = Buffer[(3 + i) as usize];
                i += 1;
            }
            memcpy(
                Identifier.as_mut_ptr().offset(7) as *mut core::ffi::c_void,
                // The C copies L" - " (6 bytes) into this CHAR array, leaving " \0-\0 \0" and
                // cutting the identifier short. The intent is the 3 characters " - ".
                // nolint: narrow-vs-wide (the C is wrong here; this array is CHAR)
                b" - ".as_ptr() as *const core::ffi::c_void,
                3,
            );

            i = 10;
            if Buffer[i as usize] == b'\\' as i8 {
                i += 1;
                j = i;
                while i < Length {
                    if Buffer[i as usize] == b'\\' as i8 {
                        break;
                    }
                    i += 1;
                }
                if i >= Length {
                    i -= 3;
                }
            }

            if Buffer[i as usize] == b'\\' as i8 {
                i += 1;
                j = i;
                while i < Length {
                    if Buffer[i as usize] == b'\\' as i8 {
                        break;
                    }
                    i += 1;
                }

                if i >= Length {
                    i -= 3;
                }
            }

            if Buffer[i as usize] == b'\\' as i8 {
                i += 1;
                j = i;
                while i < Length {
                    if Buffer[i as usize] == b'\\' as i8 {
                        break;
                    }
                    i += 1;
                }
                if Buffer[j as usize] == b'*' as i8 {
                    j += 1;
                }
                if i >= Length {
                    i -= 3;
                }
            }

            if Buffer[i as usize] == b'\\' as i8 {
                i += 1;
                j = i;
                while i < Length {
                    if Buffer[i as usize] == b';' as i8 {
                        break;
                    }
                    i += 1;
                }
                if i >= Length {
                    i -= 3;
                }
                if i > j + 1 {
                    k = 0;
                    while k < i - j {
                        Identifier[(k + 10) as usize] = Buffer[(k + j) as usize];
                        k += 1;
                    }
                    Identifier[(10 + (i - j)) as usize] = 0;
                }
            }

            TRACE!("Identifier string: %s\n", (Identifier.as_ptr()) as usize);
        }

        if Length == 0 || strlen(Identifier.as_ptr()) < 11 {
            if MouseType == 1 {
                strcpy(Identifier.as_mut_ptr(), b"LOGITECH SERIAL MOUSE\0".as_ptr() as *const i8);
            } else if MouseType == 3 {
                strcpy(Identifier.as_mut_ptr(), b"MICROSOFT SERIAL MOUSE WITH WHEEL\0".as_ptr() as *const i8);
            } else {
                strcpy(Identifier.as_mut_ptr(), b"MICROSOFT SERIAL MOUSE\0".as_ptr() as *const i8);
            }
        }

        Size = core::mem::offset_of!(CM_PARTIAL_RESOURCE_LIST, PartialDescriptors) as ULONG;
        PartialResourceList = FrLdrHeapAlloc(Size as usize, TAG_HW_RESOURCE_LIST) as PCM_PARTIAL_RESOURCE_LIST;
        if PartialResourceList == core::ptr::null_mut() {
            ERR!("Failed to allocate resource descriptor\n");
            return;
        }

        RtlZeroMemory(PartialResourceList as *mut core::ffi::c_void, Size as usize);
        (*PartialResourceList).Version = 1;
        (*PartialResourceList).Revision = 1;
        (*PartialResourceList).Count = 0;

        FldrCreateComponentKey(
            ControllerKey,
            PeripheralClass,
            PointerPeripheral,
            Input,
            0,
            0xFFFFFFFF,
            Identifier.as_ptr() as PCHAR,
            PartialResourceList,
            Size,
            &mut PeripheralKey,
        );
    }
}

unsafe extern "C" fn PcGetSerialPort(Index: ULONG, Irq: PULONG) -> ULONG {
    static PcIrq: [ULONG; 4] = [4, 3, 4, 3];
    let BasePtr: PUSHORT = 0x400 as PUSHORT;
    *Irq = PcIrq[Index as usize];
    *BasePtr.offset(Index as isize) as ULONG
}

unsafe fn GetSerialMouseDetectionBitmap(
    Options: PCSTR,
) -> u32 {
    let mut Option: PCSTR;
    let mut c: PCSTR;
    let mut OptionLength: u32 = 0;
    let mut PortBitmap: u32;
    let mut i: u32;

    if NtLdrGetOption(Options, b"FASTDETECT\0".as_ptr() as PCSTR) != 0 {
        return (1 << MAX_COM_PORTS) - 1;
    }

    Option = NtLdrGetOptionEx(Options, b"NOSERIALMICE=\0".as_ptr() as PCSTR, core::ptr::addr_of_mut!(OptionLength));
    if Option.is_null() {
        Option = NtLdrGetOptionEx(Options, b"NOSERIALMICE:\0".as_ptr() as PCSTR, core::ptr::addr_of_mut!(OptionLength));
    }

    if Option.is_null() {
        return 0;
    }

    if OptionLength < (b"NOSERIALMICE=COM9".len() - 1) as u32 {
        return (1 << MAX_COM_PORTS) - 1;
    }

    Option = Option.add((b"NOSERIALMICE=COM".len() - 1) as usize);
    OptionLength -= (b"NOSERIALMICE=COM".len() - 1) as u32;

    PortBitmap = 0;
    c = Option;
    i = 0;
    while i < OptionLength {
        let PortNumber: u8 = ((*c) - ((b'0') as i8)) as u8 ;

        if PortNumber > 0 && PortNumber <= 9 {
            PortBitmap |= 1 << (PortNumber - 1);
        }

        c = c.add(2);
        i += 2;
    }

    return PortBitmap;
}

unsafe fn DetectSerialPorts(
    Options: PCSTR,
    BusKey: PCONFIGURATION_COMPONENT_DATA,
    MachGetSerialPort: GET_SERIAL_PORT,
    Count: ULONG,
) {

    let mut PartialResourceList: PCM_PARTIAL_RESOURCE_LIST;
    let mut PartialDescriptor: PCM_PARTIAL_RESOURCE_DESCRIPTOR;
    let mut SerialDeviceData: PCM_SERIAL_DEVICE_DATA;
    let mut Irq: ULONG = 0;
    let mut Base: ULONG;
    let mut Identifier: [i8; 80] = [0; 80];
    let mut ControllerNumber: ULONG = 0;
    let mut ControllerKey: PCONFIGURATION_COMPONENT_DATA = core::ptr::null_mut();
    let mut i: ULONG;
    let mut Size: ULONG;
    let PortBitmap: ULONG;

    TRACE!("DetectSerialPorts()\n");

    PortBitmap = GetSerialMouseDetectionBitmap(Options);

    i = 0;
    while i < Count {
        Base = (MachGetSerialPort.unwrap())(i, core::ptr::addr_of_mut!(Irq));
        if Base == 0 || CpDoesPortExist(UlongToPtr(Base) as PUCHAR) == 0 {
            i += 1;
            continue;
        }

        TRACE!("Found COM%u port at 0x%x\n", (i + 1) as usize, (Base) as usize);

        RtlStringCbPrintfA(
            Identifier.as_mut_ptr(),
            core::mem::size_of_val(&Identifier),
            b"COM%ld\0".as_ptr() as PCHAR,
            i + 1,
        );

        Size = (core::mem::size_of::<CM_PARTIAL_RESOURCE_LIST>() - core::mem::size_of::<CM_PARTIAL_RESOURCE_DESCRIPTOR>() + 3 * core::mem::size_of::<CM_PARTIAL_RESOURCE_DESCRIPTOR>() + core::mem::size_of::<CM_SERIAL_DEVICE_DATA>()) as ULONG;
        PartialResourceList = FrLdrHeapAlloc(Size as usize, TAG_HW_RESOURCE_LIST) as PCM_PARTIAL_RESOURCE_LIST;
        if PartialResourceList.is_null() {
            ERR!("Failed to allocate resource descriptor! Ignoring remaining serial ports. (i = %lu, Count = %lu)\n", (i) as usize, (Count) as usize);
            break;
        }

        RtlZeroMemory(PartialResourceList as *mut core::ffi::c_void, Size as usize);
        (*PartialResourceList).Version = 1;
        (*PartialResourceList).Revision = 1;
        (*PartialResourceList).Count = 3;

        PartialDescriptor = core::ptr::addr_of_mut!((*PartialResourceList).PartialDescriptors[0]);
        (*PartialDescriptor).Type = CmResourceTypePort;
        (*PartialDescriptor).ShareDisposition = (CmResourceShareDeviceExclusive) as u8 ;
        (*PartialDescriptor).Flags = CM_RESOURCE_PORT_IO;
        (*PartialDescriptor).u.Port.Start.u.LowPart = Base;
        (*PartialDescriptor).u.Port.Start.u.HighPart = 0x0;
        (*PartialDescriptor).u.Port.Length = 8;

        PartialDescriptor = (core::ptr::addr_of_mut!((*PartialResourceList).PartialDescriptors) as *mut _CM_PARTIAL_RESOURCE_DESCRIPTOR).add(1);
        (*PartialDescriptor).Type = CmResourceTypeInterrupt;
        (*PartialDescriptor).ShareDisposition = (CmResourceShareUndetermined) as u8 ;
        (*PartialDescriptor).Flags = CM_RESOURCE_INTERRUPT_LATCHED;
        (*PartialDescriptor).u.Interrupt.Level = Irq;
        (*PartialDescriptor).u.Interrupt.Vector = Irq;
        (*PartialDescriptor).u.Interrupt.Affinity = 0xFFFFFFFF;

        PartialDescriptor = (core::ptr::addr_of_mut!((*PartialResourceList).PartialDescriptors) as *mut _CM_PARTIAL_RESOURCE_DESCRIPTOR).add(2);
        (*PartialDescriptor).Type = CmResourceTypeDeviceSpecific;
        (*PartialDescriptor).ShareDisposition = (CmResourceShareUndetermined) as u8 ;
        (*PartialDescriptor).Flags = 0;
        (*PartialDescriptor).u.DeviceSpecificData.DataSize = core::mem::size_of::<CM_SERIAL_DEVICE_DATA>() as ULONG;

        SerialDeviceData = PartialDescriptor.offset(1) as PCM_SERIAL_DEVICE_DATA;
        (*SerialDeviceData).BaudClock = 1843200;

        FldrCreateComponentKey(
            BusKey,
            ControllerClass,
            SerialController,
            (Output | ((Input) as i32) | ((ConsoleIn) as i32) | ((ConsoleOut) as i32)) as u32 ,
            ControllerNumber,
            0xFFFFFFFF,
            Identifier.as_mut_ptr(),
            PartialResourceList,
            Size,
            core::ptr::addr_of_mut!(ControllerKey),
        );

        if (PortBitmap & (1 << i)) == 0 && Rs232PortInUse(UlongToPtr(Base) as PUCHAR) == 0 {
            DetectSerialPointerPeripheral(ControllerKey, UlongToPtr(Base) as PUCHAR);
        }

        ControllerNumber += 1;
        i += 1;
    }
}

unsafe fn DetectParallelPorts(BusKey: PCONFIGURATION_COMPONENT_DATA) {
    let mut PartialResourceList: PCM_PARTIAL_RESOURCE_LIST;
    let mut PartialDescriptor: PCM_PARTIAL_RESOURCE_DESCRIPTOR;
    let Irq: [u32; 3] = [7, 5, 0xFFFFFFFF];
    let mut Identifier: [i8; 80] = [0; 80];
    let mut ControllerKey: PCONFIGURATION_COMPONENT_DATA = core::ptr::null_mut();
    let mut BasePtr: *mut u16;
    let mut Base: u32;
    let mut ControllerNumber: u32 = 0;
    let mut i: u32;
    let mut Size: u32;

    const MAX_LPT_PORTS: usize = 3;

    TRACE!("DetectParallelPorts() called\n");

    BasePtr = 0x408 as *mut u16;

    i = 0;
    while i < MAX_LPT_PORTS as u32 {
        Base = *BasePtr as u32;
        if Base == 0 {
            BasePtr = BasePtr.offset(1);
            i += 1;
            continue;
        }

        TRACE!("Parallel port %u: %x\n", (ControllerNumber) as usize, (Base) as usize);

        RtlStringCbPrintfA(
            Identifier.as_mut_ptr(),
            core::mem::size_of::<[i8; 80]>(),
            b"PARALLEL%ld\0".as_ptr() as *const i8,
            i + 1,
        );

        Size = core::mem::offset_of!(_CM_PARTIAL_RESOURCE_LIST, PartialDescriptors) as u32
            + core::mem::size_of::<CM_PARTIAL_RESOURCE_DESCRIPTOR>() as u32;
        if Irq[i as usize] != 0xFFFFFFFF {
            Size += core::mem::size_of::<CM_PARTIAL_RESOURCE_DESCRIPTOR>() as u32;
        }

        PartialResourceList = FrLdrHeapAlloc(Size as usize, TAG_HW_RESOURCE_LIST) as PCM_PARTIAL_RESOURCE_LIST;
        if PartialResourceList.is_null() {
            ERR!("Failed to allocate resource descriptor! Ignoring remaining parallel ports. (i = %lu)\n", (i) as usize);
            break;
        }

        core::ptr::write_bytes(PartialResourceList as *mut u8, 0, Size as usize);
        (*PartialResourceList).Version = 1;
        (*PartialResourceList).Revision = 1;
        (*PartialResourceList).Count = if Irq[i as usize] != 0xFFFFFFFF { 2 } else { 1 };

        PartialDescriptor = core::ptr::addr_of_mut!((*PartialResourceList).PartialDescriptors[0]);
        (*PartialDescriptor).Type = 1;
        (*PartialDescriptor).ShareDisposition = (CmResourceShareDeviceExclusive) as u8 ;
        (*PartialDescriptor).Flags = 0x100;
        (*PartialDescriptor).u.Port.Start.u.LowPart = Base;
        (*PartialDescriptor).u.Port.Start.u.HighPart = 0;
        (*PartialDescriptor).u.Port.Length = 3;

        if Irq[i as usize] != 0xFFFFFFFF {
            PartialDescriptor = (core::ptr::addr_of_mut!((*PartialResourceList).PartialDescriptors) as *mut _CM_PARTIAL_RESOURCE_DESCRIPTOR).add(1);
            (*PartialDescriptor).Type = 2;
            (*PartialDescriptor).ShareDisposition = (CmResourceShareUndetermined) as u8 ;
            (*PartialDescriptor).Flags = 0x1;
            (*PartialDescriptor).u.Interrupt.Level = Irq[i as usize];
            (*PartialDescriptor).u.Interrupt.Vector = Irq[i as usize];
            (*PartialDescriptor).u.Interrupt.Affinity = 0xFFFFFFFF;
        }

        FldrCreateComponentKey(
            BusKey,
            ControllerClass,
            ParallelController,
            (Output) as u32 ,
            ControllerNumber,
            0xFFFFFFFF,
            Identifier.as_ptr(),
            PartialResourceList,
            Size,
            core::ptr::addr_of_mut!(ControllerKey),
        );

        ControllerNumber += 1;
        BasePtr = BasePtr.offset(1);
        i += 1;
    }

    TRACE!("DetectParallelPorts() done\n");
}

unsafe fn DetectKeyboardDevice() -> u8 {

    let mut Status: u8 = 0;
    let mut Scancode: u8;
    let mut Loops: u32;
    let mut Result: u8 = 1u8;

    WRITE_PORT_UCHAR(0x60 as *mut u8, 0xF2);

    Loops = 0;
    while Loops < 100 {
        StallExecutionProcessor(10000);
        Status = READ_PORT_UCHAR(0x64 as *mut u8);
        if (Status & 0x01) != 0 {
            break;
        }
        Loops += 1;
    }

    if (Status & 0x01) == 0 {
        Result = 0u8;
    }

    Scancode = READ_PORT_UCHAR(0x60 as *mut u8);
    if Scancode != 0xFA {
        Result = 0u8;
    }

    StallExecutionProcessor(10000);

    Status = READ_PORT_UCHAR(0x64 as *mut u8);
    if (Status & 0x01) == 0 {
        return Result;
    }

    Scancode = READ_PORT_UCHAR(0x60 as *mut u8);
    if Scancode != 0xAB {
        Result = 0u8;
    }

    StallExecutionProcessor(10000);

    Status = READ_PORT_UCHAR(0x64 as *mut u8);
    if (Status & 0x01) == 0 {
        Result = 0u8;
    }

    Scancode = READ_PORT_UCHAR(0x60 as *mut u8);
    if Scancode != 0x41 {
        Result = 0u8;
    }

    Result
}

unsafe fn DetectKeyboardPeripheral(ControllerKey: PCONFIGURATION_COMPONENT_DATA) {
    let PartialResourceList: PCM_PARTIAL_RESOURCE_LIST;
    let PartialDescriptor: PCM_PARTIAL_RESOURCE_DESCRIPTOR;
    let KeyboardData: PCM_KEYBOARD_DEVICE_DATA;
    let mut PeripheralKey: PCONFIGURATION_COMPONENT_DATA = core::ptr::null_mut();
    let Size: u32;
    let mut Regs: REGS = core::mem::zeroed();

    {
        Size = crate::FIELD_OFFSET!(CM_PARTIAL_RESOURCE_LIST, PartialDescriptors[1]) as u32 + core::mem::size_of::<CM_KEYBOARD_DEVICE_DATA>() as u32;
        PartialResourceList = FrLdrHeapAlloc((Size) as usize , TAG_HW_RESOURCE_LIST) as PCM_PARTIAL_RESOURCE_LIST;
        if PartialResourceList.is_null() {
            ERR!("Failed to allocate resource descriptor\n");
            return;
        }

        core::ptr::write_bytes(PartialResourceList as *mut u8, 0, Size as usize);
        (*PartialResourceList).Version = 1;
        (*PartialResourceList).Revision = 1;
        (*PartialResourceList).Count = 1;

        PartialDescriptor = &mut (*PartialResourceList).PartialDescriptors[0] as *mut _;
        (*PartialDescriptor).Type = CmResourceTypeDeviceSpecific;
        (*PartialDescriptor).ShareDisposition = (CmResourceShareUndetermined) as u8 ;
        (*PartialDescriptor).u.DeviceSpecificData.DataSize = core::mem::size_of::<CM_KEYBOARD_DEVICE_DATA>() as u32;

        Regs.b.ah = 0x02;
        Int386(0x16, &mut Regs as *mut _, &mut Regs as *mut _);

        KeyboardData = PartialDescriptor.offset(1) as PCM_KEYBOARD_DEVICE_DATA;
        (*KeyboardData).Version = 1;
        (*KeyboardData).Revision = 1;
        (*KeyboardData).Type = 4;
        (*KeyboardData).Subtype = 0;
        (*KeyboardData).KeyboardFlags = (Regs.b.al) as u16 ;

        FldrCreateComponentKey(
            ControllerKey,
            PeripheralClass,
            KeyboardPeripheral,
            Input | ConsoleIn,
            0,
            0xFFFFFFFF,
            b"PCAT_ENHANCED\0".as_ptr() as PCHAR,
            PartialResourceList,
            Size,
            &mut PeripheralKey as *mut _,
        );
    }
}

unsafe fn DetectKeyboardController(BusKey: PCONFIGURATION_COMPONENT_DATA) {
    let PartialResourceList: PCM_PARTIAL_RESOURCE_LIST;
    let mut PartialDescriptor: PCM_PARTIAL_RESOURCE_DESCRIPTOR;
    let mut ControllerKey: PCONFIGURATION_COMPONENT_DATA = core::ptr::null_mut();
    let Size: u32;

    Size = crate::FIELD_OFFSET!(CM_PARTIAL_RESOURCE_LIST, PartialDescriptors[3]) as u32;
    PartialResourceList = FrLdrHeapAlloc((Size) as usize , TAG_HW_RESOURCE_LIST) as PCM_PARTIAL_RESOURCE_LIST;
    if PartialResourceList.is_null() {
        ERR!("Failed to allocate resource descriptor\n");
        return;
    }

    RtlZeroMemory(PartialResourceList as *mut core::ffi::c_void, Size as usize);
    (*PartialResourceList).Version = 1;
    (*PartialResourceList).Revision = 1;
    (*PartialResourceList).Count = 3;

    PartialDescriptor = core::ptr::addr_of_mut!((*PartialResourceList).PartialDescriptors[0]);
    (*PartialDescriptor).Type = CmResourceTypeInterrupt;
    (*PartialDescriptor).ShareDisposition = (CmResourceShareUndetermined) as u8 ;
    (*PartialDescriptor).Flags = CM_RESOURCE_INTERRUPT_LATCHED;
    (*PartialDescriptor).u.Interrupt.Level = 1;
    (*PartialDescriptor).u.Interrupt.Vector = 1;
    (*PartialDescriptor).u.Interrupt.Affinity = 0xFFFFFFFF;

    PartialDescriptor = (core::ptr::addr_of_mut!((*PartialResourceList).PartialDescriptors) as *mut _CM_PARTIAL_RESOURCE_DESCRIPTOR).add(1);
    (*PartialDescriptor).Type = CmResourceTypePort;
    (*PartialDescriptor).ShareDisposition = (CmResourceShareDeviceExclusive) as u8 ;
    (*PartialDescriptor).Flags = CM_RESOURCE_PORT_IO;
    (*PartialDescriptor).u.Port.Start.u.LowPart = 0x60;
    (*PartialDescriptor).u.Port.Start.u.HighPart = 0x0;
    (*PartialDescriptor).u.Port.Length = 1;

    PartialDescriptor = (core::ptr::addr_of_mut!((*PartialResourceList).PartialDescriptors) as *mut _CM_PARTIAL_RESOURCE_DESCRIPTOR).add(2);
    (*PartialDescriptor).Type = CmResourceTypePort;
    (*PartialDescriptor).ShareDisposition = (CmResourceShareDeviceExclusive) as u8 ;
    (*PartialDescriptor).Flags = CM_RESOURCE_PORT_IO;
    (*PartialDescriptor).u.Port.Start.u.LowPart = 0x64;
    (*PartialDescriptor).u.Port.Start.u.HighPart = 0x0;
    (*PartialDescriptor).u.Port.Length = 1;

    FldrCreateComponentKey(
        BusKey,
        ControllerClass,
        KeyboardController,
        Input | ConsoleIn,
        0,
        0xFFFFFFFF,
        core::ptr::null_mut(),
        PartialResourceList,
        Size,
        core::ptr::addr_of_mut!(ControllerKey),
    );

    DetectKeyboardPeripheral(ControllerKey);
}

unsafe fn PS2ControllerWait() {
    let mut Timeout: u32;
    let mut Status: u8;

    Timeout = 0;
    while Timeout < CONTROLLER_TIMEOUT {
        Status = READ_PORT_UCHAR(CONTROLLER_REGISTER_STATUS as *mut u8);
        if (Status & CONTROLLER_STATUS_INPUT_BUFFER_FULL) == 0 {
            return;
        }

        StallExecutionProcessor(1000);
        Timeout += 1;
    }
}

unsafe fn DetectPS2AuxPort() -> u8 {
    return 1u8;
}

unsafe fn DetectPS2AuxDevice() -> u8 {
    let mut Scancode: u8;
    let mut Status: u8;
    let mut Loops: u32;
    let mut Result: u8 = 1u8;

    PS2ControllerWait();
    WRITE_PORT_UCHAR(CONTROLLER_REGISTER_CONTROL as *mut u8,
                     CONTROLLER_COMMAND_WRITE_MOUSE);
    PS2ControllerWait();

    WRITE_PORT_UCHAR(CONTROLLER_REGISTER_DATA as *mut u8, 0xF2);

    Loops = 0;
    while Loops < 100 {
        StallExecutionProcessor(10000);
        Status = READ_PORT_UCHAR(CONTROLLER_REGISTER_STATUS as *mut u8);
        if (Status & CONTROLLER_STATUS_OUTPUT_BUFFER_FULL) != 0 {
            break;
        }
        Loops += 1;
    }

    Status = READ_PORT_UCHAR(CONTROLLER_REGISTER_STATUS as *mut u8);
    if (Status & CONTROLLER_STATUS_MOUSE_OUTPUT_BUFFER_FULL) == 0 {
        Result = 0u8;
    }

    Scancode = READ_PORT_UCHAR(CONTROLLER_REGISTER_DATA as *mut u8);
    if Scancode != 0xFA {
        Result = 0u8;
    }

    StallExecutionProcessor(10000);

    Status = READ_PORT_UCHAR(CONTROLLER_REGISTER_STATUS as *mut u8);
    if (Status & CONTROLLER_STATUS_MOUSE_OUTPUT_BUFFER_FULL) == 0 {
        Result = 0u8;
    }

    Scancode = READ_PORT_UCHAR(CONTROLLER_REGISTER_DATA as *mut u8);
    if Scancode != 0x00 {
        Result = 0u8;
    }

    Result
}

unsafe fn DetectPS2Mouse(BusKey: PCONFIGURATION_COMPONENT_DATA) {

    let mut PartialResourceList: PCM_PARTIAL_RESOURCE_LIST;
    let mut ControllerKey: PCONFIGURATION_COMPONENT_DATA = core::ptr::null_mut();
    let mut PeripheralKey: PCONFIGURATION_COMPONENT_DATA = core::ptr::null_mut();
    let mut Size: u32;

    if DetectPS2AuxPort() != 0 {
        TRACE!("Detected PS2 port\n");

        // C: Size = FIELD_OFFSET(CM_PARTIAL_RESOURCE_LIST, PartialDescriptors[1]).
        // rustconv emitted a `[u8; 20]` literal-size stand-in for
        // CM_PARTIAL_RESOURCE_DESCRIPTOR; correct on amd64 (20B) but
        // 4 bytes too BIG on i386 (16B). Use FIELD_OFFSET! for both arches.
        Size = crate::FIELD_OFFSET!(CM_PARTIAL_RESOURCE_LIST, PartialDescriptors[1]);
        PartialResourceList = FrLdrHeapAlloc(Size as usize, TAG_HW_RESOURCE_LIST) as PCM_PARTIAL_RESOURCE_LIST;
        if PartialResourceList.is_null() {
            ERR!("Failed to allocate resource descriptor\n");
            return;
        }

        // C: RtlZeroMemory(PartialResourceList, Size). Byte-count, not
        // element-count (see [[feedback_write_bytes_element_count]]).
        core::ptr::write_bytes(PartialResourceList as *mut u8, 0, Size as usize);
        (*PartialResourceList).Version = 1;
        (*PartialResourceList).Revision = 1;
        (*PartialResourceList).Count = 1;

        (*PartialResourceList).PartialDescriptors[0].Type = 2u8;
        (*PartialResourceList).PartialDescriptors[0].ShareDisposition = (CmResourceShareUndetermined) as u8 ;
        (*PartialResourceList).PartialDescriptors[0].Flags = 1u16;
        (*PartialResourceList).PartialDescriptors[0].u.Interrupt.Level = 12;
        (*PartialResourceList).PartialDescriptors[0].u.Interrupt.Vector = 12;
        (*PartialResourceList).PartialDescriptors[0].u.Interrupt.Affinity = 0xFFFFFFFF;

        FldrCreateComponentKey(
            BusKey,
            ControllerClass,
            PointerController,
            Input,
            0,
            0xFFFFFFFF,
            core::ptr::null(),
            PartialResourceList,
            Size,
            core::ptr::addr_of_mut!(ControllerKey),
        );

        if DetectPS2AuxDevice() != 0 {
            TRACE!("Detected PS2 mouse\n");

            // C: Size = FIELD_OFFSET(CM_PARTIAL_RESOURCE_LIST, PartialDescriptors).
            Size = crate::FIELD_OFFSET!(CM_PARTIAL_RESOURCE_LIST, PartialDescriptors);
            PartialResourceList = FrLdrHeapAlloc(Size as usize, TAG_HW_RESOURCE_LIST) as PCM_PARTIAL_RESOURCE_LIST;
            if PartialResourceList.is_null() {
                ERR!("Failed to allocate resource descriptor\n");
                return;
            }

            // C: RtlZeroMemory(PartialResourceList, Size). Byte-count form.
            core::ptr::write_bytes(PartialResourceList as *mut u8, 0, Size as usize);
            (*PartialResourceList).Version = 1;
            (*PartialResourceList).Revision = 1;
            (*PartialResourceList).Count = 0;

            FldrCreateComponentKey(
                ControllerKey,
                ControllerClass,
                PointerPeripheral,
                Input,
                0,
                0xFFFFFFFF,
                b"MICROSOFT PS2 MOUSE\0".as_ptr() as *const i8,
                PartialResourceList,
                Size,
                core::ptr::addr_of_mut!(PeripheralKey) as *mut PCONFIGURATION_COMPONENT_DATA,
            );
        }
    }
}

unsafe fn CreateBusMousePeripheralKey(
    BusKey: PCONFIGURATION_COMPONENT_DATA,
    IoBase: u32,
    Irq: u32,
)
{
    let mut PartialResourceList: PCM_PARTIAL_RESOURCE_LIST;
    let mut PartialDescriptor: PCM_PARTIAL_RESOURCE_DESCRIPTOR;
    let mut ControllerKey: PCONFIGURATION_COMPONENT_DATA = core::ptr::null_mut();
    let mut PeripheralKey: PCONFIGURATION_COMPONENT_DATA = core::ptr::null_mut();
    let mut Size: u32;

    const CM_RESOURCE_PORT_IO: u16 = 1;
    const CM_RESOURCE_INTERRUPT_LATCHED: u16 = 1;
    const CmResourceTypePort: u8 = 1;
    const CmResourceTypeInterrupt: u8 = 2;

    // C: Size = FIELD_OFFSET(CM_PARTIAL_RESOURCE_LIST, PartialDescriptors[2]).
    // rustconv used `sizeof(PCM_*)` (pointer size 4/8) instead of struct
    // size (12), under-allocating → heap overflow when writing 2 descs.
    Size = crate::FIELD_OFFSET!(CM_PARTIAL_RESOURCE_LIST, PartialDescriptors[2]);

    PartialResourceList = FrLdrHeapAlloc(Size as usize, TAG_HW_RESOURCE_LIST) as PCM_PARTIAL_RESOURCE_LIST;
    if PartialResourceList.is_null()
    {
        ERR!("Failed to allocate resource descriptor\n");
        return;
    }

    RtlZeroMemory(PartialResourceList as *mut core::ffi::c_void, Size as usize);
    (*PartialResourceList).Version = 1;
    (*PartialResourceList).Revision = 1;
    (*PartialResourceList).Count = 2;

    PartialDescriptor = core::ptr::addr_of_mut!((*PartialResourceList).PartialDescriptors[0]);
    (*PartialDescriptor).Type = CmResourceTypePort;
    (*PartialDescriptor).ShareDisposition = (CmResourceShareDeviceExclusive) as u8 ;
    (*PartialDescriptor).Flags = CM_RESOURCE_PORT_IO;
    (*PartialDescriptor).u.Port.Start.u.LowPart = IoBase;
    (*PartialDescriptor).u.Port.Start.u.HighPart = 0;
    (*PartialDescriptor).u.Port.Length = 4;

    PartialDescriptor = (core::ptr::addr_of_mut!((*PartialResourceList).PartialDescriptors) as *mut _CM_PARTIAL_RESOURCE_DESCRIPTOR).add(1);
    (*PartialDescriptor).Type = CmResourceTypeInterrupt;
    (*PartialDescriptor).ShareDisposition = (CmResourceShareUndetermined) as u8 ;
    (*PartialDescriptor).Flags = CM_RESOURCE_INTERRUPT_LATCHED;
    (*PartialDescriptor).u.Interrupt.Level = Irq;
    (*PartialDescriptor).u.Interrupt.Vector = Irq;
    (*PartialDescriptor).u.Interrupt.Affinity = !0 as KAFFINITY;

    FldrCreateComponentKey(BusKey,
                           ControllerClass,
                           PointerController,
                           Input,
                           0,
                           0xFFFFFFFF,
                           core::ptr::null(),
                           PartialResourceList,
                           Size,
                           core::ptr::addr_of_mut!(ControllerKey));

    // C: Size = FIELD_OFFSET(CM_PARTIAL_RESOURCE_LIST, PartialDescriptors).
    // rustconv used `sizeof(LIST) - sizeof(PCM_*)`, over-allocating slightly
    // (sizeof(LIST) includes one inline DESC). Fixed to the canonical form.
    Size = crate::FIELD_OFFSET!(CM_PARTIAL_RESOURCE_LIST, PartialDescriptors);

    PartialResourceList = FrLdrHeapAlloc(Size as usize, TAG_HW_RESOURCE_LIST) as PCM_PARTIAL_RESOURCE_LIST;
    if PartialResourceList.is_null()
    {
        ERR!("Failed to allocate resource descriptor\n");
        return;
    }

    RtlZeroMemory(PartialResourceList as *mut core::ffi::c_void, Size as usize);
    (*PartialResourceList).Version = 1;
    (*PartialResourceList).Revision = 1;
    (*PartialResourceList).Count = 0;

    FldrCreateComponentKey(ControllerKey,
                           ControllerClass,
                           PointerPeripheral,
                           Input,
                           0,
                           0xFFFFFFFF,
                           b"MICROSOFT INPORT MOUSE\0".as_ptr() as *const i8,
                           PartialResourceList,
                           Size,
                           core::ptr::addr_of_mut!(PeripheralKey));
}

unsafe fn DetectBusMouseTestIrq(
    IoBase: ULONG,
    Irq: ULONG,
) -> ULONG {
    let mut OldOffset: USHORT;
    let mut OldExtendedOffset: USHORT;
    let Vector: ULONG;
    let mut i: ULONG;

    HwIrqCount = 0;

    WRITE_PORT_UCHAR((IoBase + INPORT_REGISTER_CONTROL) as PUCHAR, INPORT_RESET);
    WRITE_PORT_UCHAR((IoBase + INPORT_REGISTER_CONTROL) as PUCHAR, INPORT_REG_MODE);

    Vector = Irq + 8;

    OldOffset = 0;
    OldExtendedOffset = 0;
    // i386Idt has the 16-bit real-mode IDT entry layout (.Offset /
    // .ExtendedOffset). That layout is i386-only; on amd64 the ndk macro-
    // aliases KIDTENTRY -> KIDTENTRY64 (different fields) and this PC-AT
    // bus-mouse 16-bit-handler IDT hook is not part of the amd64
    // freeldr_pe mach path. Keep the i386 logic byte-identical; no-op the
    // IDT hook on amd64.
    #[cfg(target_arch = "x86")]
    {
        OldOffset = i386Idt[Vector as usize].Offset;
        OldExtendedOffset = i386Idt[Vector as usize].ExtendedOffset;

        i386Idt[Vector as usize].Offset = (((HwIrqHandler as ULONG) & 0xFFFF) as u16);
        i386Idt[Vector as usize].ExtendedOffset = (((HwIrqHandler as ULONG) >> 16) as u16);
    }

    WRITE_PORT_UCHAR(PIC1_DATA_PORT as PUCHAR, !(1 << Irq) as UCHAR);

    _enable();

    i = 0;
    while i < 15 {
        WRITE_PORT_UCHAR((IoBase + INPORT_REGISTER_DATA) as PUCHAR, INPORT_MODE_BASE);
        WRITE_PORT_UCHAR((IoBase + INPORT_REGISTER_DATA) as PUCHAR, INPORT_TEST_IRQ);
        i += 1;
    }

    WRITE_PORT_UCHAR((IoBase + INPORT_REGISTER_DATA) as PUCHAR, 0);

    _disable();

    #[cfg(target_arch = "x86")]
    {
        i386Idt[Vector as usize].Offset = OldOffset;
        i386Idt[Vector as usize].ExtendedOffset = OldExtendedOffset;
    }
    #[cfg(not(target_arch = "x86"))]
    {
        let _ = (OldOffset, OldExtendedOffset);
    }

    if HwIrqCount != 0 { Irq } else { 0 }
}

unsafe fn DetectBusMouseIrq(
    IoBase: ULONG,
) -> ULONG {
    let Mask1: UCHAR;
    let Mask2: UCHAR;
    let mut Irq: ULONG;
    let mut Result: ULONG = 0;

    Mask1 = READ_PORT_UCHAR(PIC1_DATA_PORT as PUCHAR);
    Mask2 = READ_PORT_UCHAR(PIC2_DATA_PORT as PUCHAR);

    WRITE_PORT_UCHAR(PIC2_DATA_PORT as PUCHAR, 0xFF);

    Irq = 5;
    loop {
        Result = DetectBusMouseTestIrq(IoBase, Irq);
        if Result != 0 {
            break;
        }
        if Irq == 3 {
            break;
        }
        Irq -= 1;
    }

    WRITE_PORT_UCHAR(PIC1_DATA_PORT as PUCHAR, Mask1);
    WRITE_PORT_UCHAR(PIC2_DATA_PORT as PUCHAR, Mask2);

    Result
}

unsafe fn DetectBusMouse(
    BusKey: PCONFIGURATION_COMPONENT_DATA,
)
{
    let mut IoBase: ULONG;
    let mut Irq: ULONG;
    let mut Signature1: ULONG;
    let mut Signature2: ULONG;
    let mut Signature3: ULONG;

    IoBase = 0x23C;
    loop {
        Signature1 = READ_PORT_UCHAR((IoBase + INPORT_REGISTER_SIGNATURE) as PUCHAR) as ULONG;
        Signature2 = READ_PORT_UCHAR((IoBase + INPORT_REGISTER_SIGNATURE) as PUCHAR) as ULONG;
        if Signature1 == Signature2 {
            if IoBase < 0x230 {
                break;
            }
            IoBase -= 4;
            continue;
        }
        if Signature1 != ((INPORT_SIGNATURE) as u32) && Signature2 != ((INPORT_SIGNATURE) as u32) {
            if IoBase < 0x230 {
                break;
            }
            IoBase -= 4;
            continue;
        }

        Signature3 = READ_PORT_UCHAR((IoBase + INPORT_REGISTER_SIGNATURE) as PUCHAR) as ULONG;
        if Signature1 != Signature3 {
            if IoBase < 0x230 {
                break;
            }
            IoBase -= 4;
            continue;
        }

        Irq = DetectBusMouseIrq(IoBase);
        if Irq == 0 {
            if IoBase < 0x230 {
                break;
            }
            IoBase -= 4;
            continue;
        }

        CreateBusMousePeripheralKey(BusKey, IoBase, Irq);
        break;
    }
}

/// The DisplayController node for a VBE linear framebuffer: one memory range
/// plus CM_FRAMEBUF_DEVICE_DATA, byte-for-byte the layout uefihw.rs builds for GOP.
unsafe fn CreateFramebufferDisplayKey(
    BusKey: PCONFIGURATION_COMPONENT_DATA,
    Fb: &crate::pcvideo::PcVbeFramebuffer,
) {
    const CmResourceTypeMemory: u8 = 3;
    const CM_RESOURCE_MEMORY_READ_WRITE: USHORT = 0x0000;
    let mut ControllerKey: PCONFIGURATION_COMPONENT_DATA = core::ptr::null_mut();
    let Size: u32 = crate::FIELD_OFFSET!(CM_PARTIAL_RESOURCE_LIST, PartialDescriptors[2])
        + core::mem::size_of::<CM_FRAMEBUF_DEVICE_DATA>() as u32;
    let PartialResourceList =
        FrLdrHeapAlloc(Size as usize, TAG_HW_RESOURCE_LIST) as PCM_PARTIAL_RESOURCE_LIST;
    if PartialResourceList.is_null() {
        ERR!("Failed to allocate resource descriptor\n");
        return;
    }
    core::ptr::write_bytes(PartialResourceList as *mut u8, 0, Size as usize);
    (*PartialResourceList).Version = 1;
    (*PartialResourceList).Revision = 2;
    (*PartialResourceList).Count = 2;

    let mut PartialDescriptor: PCM_PARTIAL_RESOURCE_DESCRIPTOR =
        core::ptr::addr_of_mut!((*PartialResourceList).PartialDescriptors[0]);
    (*PartialDescriptor).Type = CmResourceTypeMemory;
    (*PartialDescriptor).ShareDisposition = CmResourceShareDeviceExclusive as u8;
    (*PartialDescriptor).Flags = CM_RESOURCE_MEMORY_READ_WRITE;
    (*PartialDescriptor).u.Memory.Start.QuadPart = Fb.Base as i64;
    (*PartialDescriptor).u.Memory.Length = Fb.Size;

    // flex array: element [1] is past the [1]-sized field — pointer-step it.
    PartialDescriptor = PartialDescriptor.add(1);
    (*PartialDescriptor).Type = CmResourceTypeDeviceSpecific;
    (*PartialDescriptor).ShareDisposition = CmResourceShareUndetermined as u8;
    (*PartialDescriptor).Flags = 0;
    (*PartialDescriptor).u.DeviceSpecificData.DataSize =
        core::mem::size_of::<CM_FRAMEBUF_DEVICE_DATA>() as u32;
    core::ptr::write_unaligned(PartialDescriptor.add(1) as *mut CM_FRAMEBUF_DEVICE_DATA, Fb.Data);

    FldrCreateComponentKey(
        BusKey,
        ControllerClass,
        DisplayController,
        (Output | ((ConsoleOut) as i32)) as u32,
        0,
        0xFFFFFFFF,
        b"VBE Framebuffer\0".as_ptr() as PCSTR,
        PartialResourceList,
        Size,
        core::ptr::addr_of_mut!(ControllerKey),
    );
}

unsafe fn DetectDisplayController(BusKey: PCONFIGURATION_COMPONENT_DATA) {
    let Identifier: PCSTR;
    let mut ControllerKey: PCONFIGURATION_COMPONENT_DATA = core::ptr::null_mut();
    let VesaVersion: u16;

    VesaVersion = BiosIsVesaSupported();
    if VesaVersion != 0 {
        TRACE!("VESA version %c.%c\n", (((VesaVersion >> 8) as u8 + b'0') as i32) as usize, (((VesaVersion & 0xFF) as u8 + b'0') as i32) as usize);
    } else {
        TRACE!("VESA not supported\n");
    }

    // A VBE linear framebuffer, described the way the UEFI loader describes GOP
    // (uefihw.rs), is what the kernel's framebuffer display path needs.
    if let Some(fb) = crate::pcvideo::PcVideoSelectVbeFramebuffer(VesaVersion) {
        CreateFramebufferDisplayKey(BusKey, &fb);
        return;
    }

    if VesaVersion >= 0x0200 {
        Identifier = b"VBE Display\0".as_ptr() as PCSTR;
    } else {
        Identifier = b"VGA Display\0".as_ptr() as PCSTR;
    }

    FldrCreateComponentKey(
        BusKey,
        ControllerClass,
        DisplayController,
        (Output | ((ConsoleOut) as i32)) as u32 ,
        0,
        0xFFFFFFFF,
        Identifier,
        core::ptr::null_mut(),
        0,
        core::ptr::addr_of_mut!(ControllerKey),
    );

    if VesaVersion != 0 {
        if BiosIsVesaDdcSupported() != 0 {
            TRACE!("VESA/DDC supported!\n");
            if BiosVesaReadEdid() != 0 {
                TRACE!("EDID data read successfully!\n");
            }
        }
    }
}

unsafe fn DetectIsaBios(
    Options: PCSTR,
    SystemKey: PCONFIGURATION_COMPONENT_DATA,
    BusNumber: *mut ULONG,
)
{

    let PartialResourceList: PCM_PARTIAL_RESOURCE_LIST;
    let mut BusKey: PCONFIGURATION_COMPONENT_DATA = core::ptr::null_mut();
    let Size: ULONG;

    Size = crate::FIELD_OFFSET!(CM_PARTIAL_RESOURCE_LIST, PartialDescriptors) as ULONG;
    PartialResourceList = FrLdrHeapAlloc(Size as usize, TAG_HW_RESOURCE_LIST) as PCM_PARTIAL_RESOURCE_LIST;
    if PartialResourceList.is_null()
    {
        ERR!("Failed to allocate resource descriptor\n");
        return;
    }

    RtlZeroMemory(PartialResourceList as *mut core::ffi::c_void, Size as usize);
    (*PartialResourceList).Version = 1;
    (*PartialResourceList).Revision = 1;
    (*PartialResourceList).Count = 0;

    FldrCreateComponentKey(SystemKey,
                           AdapterClass,
                           MultiFunctionAdapter,
                           0,
                           0,
                           0xFFFFFFFF,
                           b"ISA\0".as_ptr() as PCSTR,
                           PartialResourceList,
                           Size,
                           core::ptr::addr_of_mut!(BusKey));

    (*BusNumber) = (*BusNumber).wrapping_add(1);

    DetectBiosDisks(SystemKey, BusKey);
    DetectSerialPorts(Options, BusKey, Some(PcGetSerialPort), MAX_COM_PORTS);
    DetectParallelPorts(BusKey);
    DetectKeyboardController(BusKey);
    DetectPS2Mouse(BusKey);
    #[cfg(target_arch = "x86")]
    DetectBusMouse(BusKey);
    DetectDisplayController(BusKey);
}

unsafe extern "C" fn PcGetFloppyCount() -> u8 {

    let Data: u8;

    WRITE_PORT_UCHAR(0x70 as *mut u8, 0x10);
    Data = READ_PORT_UCHAR(0x71 as *mut u8);

    (if Data & 0xF0 != 0 { 1 } else { 0 }) + (if Data & 0x0F != 0 { 1 } else { 0 })
}

#[cfg(not(feature = "uefi"))]
#[no_mangle]
pub unsafe extern "C" fn PcHwDetect(Options: PCSTR) -> PCONFIGURATION_COMPONENT_DATA {

    let mut SystemKey: PCONFIGURATION_COMPONENT_DATA = core::ptr::null_mut();
    let mut BusNumber: ULONG = 0;

    TRACE!("DetectHardware()\n");

    FldrCreateSystemKey(core::ptr::addr_of_mut!(SystemKey), b"AT/AT COMPATIBLE\0".as_ptr() as PCSTR);

    GetHarddiskConfigurationData = PcGetHarddiskConfigurationData as *mut core::ffi::c_void;
    FindPciBios = PcFindPciBios as *mut core::ffi::c_void;

    DetectPciBios(SystemKey, core::ptr::addr_of_mut!(BusNumber));
    DetectApmBios(SystemKey, core::ptr::addr_of_mut!(BusNumber));
    DetectPnpBios(SystemKey, core::ptr::addr_of_mut!(BusNumber));
    DetectIsaBios(Options, SystemKey, core::ptr::addr_of_mut!(BusNumber));
    DetectAcpiBios(SystemKey, core::ptr::addr_of_mut!(BusNumber));

    TRACE!("DetectHardware() Done\n");
    return SystemKey;
}

#[no_mangle]
pub unsafe extern "C" fn PcHwIdle() {
    let mut Regs: REGS = core::mem::zeroed();

    Regs.b.ah = 0x53;
    Regs.b.al = 0x05;

    Int386(0x15, &mut Regs, &mut Regs);

    if INT386_SUCCESS(Regs) != 0 {
        return;
    }
}

#[no_mangle]
pub unsafe extern "C" fn ChainLoadBiosBootSectorCode(
    BootDrive: UCHAR,
    BootPartition: ULONG,
) {
    let mut Regs: REGS = core::mem::zeroed();

    Regs.b.dl = if BootDrive != 0 { BootDrive } else { FrldrBootDrive };
    Regs.b.dh = if BootPartition != 0 { BootPartition as UCHAR } else { (FrldrBootPartition) as u8  };

    Relocator16Boot(
        &mut Regs,
        0x0000,
        0x7C00,
        0x0000,
        0x7C00,
    );
}

// BIOS-only: uefildr's MachInit is uefisetup.c (UEFI-active C). Gated
// out of the `uefi` archive to avoid the per-binary-owner dup.
#[cfg(not(feature = "uefi"))]
#[inline(always)]
unsafe fn _vga_dbg(slot: u32, c: u8) {
    let vga = 0xB8000 as *mut u16;
    // top-row debug strip: each slot is 1 char, attribute 0x0F (white-on-black).
    core::ptr::write_volatile(vga.offset(slot as isize), (c as u16) | 0x0F00);
    // Also emit to QEMU debugcon (port 0xe9) for off-screen capture.
    core::arch::asm!("out dx, al", in("dx") 0xe9u16, in("al") c, options(nostack, preserves_flags));
}

#[cfg(not(feature = "uefi"))]
#[no_mangle]
pub unsafe extern "C" fn MachInit(_CmdLine: *const i8) {
    _vga_dbg(0, b'A');
    RtlZeroMemory(&mut MachVtbl as *mut _ as *mut core::ffi::c_void, core::mem::size_of_val(&MachVtbl));
    _vga_dbg(1, b'B');
    MachVtbl.ConsPutChar = Some(PcConsPutChar);
    MachVtbl.ConsKbHit = Some(PcConsKbHit);
    MachVtbl.ConsGetCh = Some(PcConsGetCh);
    MachVtbl.VideoClearScreen = Some(PcVideoClearScreen);
    MachVtbl.VideoSetDisplayMode = Some(PcVideoSetDisplayMode);
    MachVtbl.VideoGetDisplaySize = Some(PcVideoGetDisplaySize);
    MachVtbl.VideoGetBufferSize = Some(PcVideoGetBufferSize);
    MachVtbl.VideoGetFontsFromFirmware = Some(PcVideoGetFontsFromFirmware);
    MachVtbl.VideoSetTextCursorPosition = Some(PcVideoSetTextCursorPosition);
    MachVtbl.VideoHideShowTextCursor = Some(PcVideoHideShowTextCursor);
    MachVtbl.VideoPutChar = Some(PcVideoPutChar);
    MachVtbl.VideoCopyOffScreenBufferToVRAM = Some(PcVideoCopyOffScreenBufferToVRAM);
    MachVtbl.VideoIsPaletteFixed = Some(PcVideoIsPaletteFixed);
    MachVtbl.VideoSetPaletteColor = Some(PcVideoSetPaletteColor);
    MachVtbl.VideoGetPaletteColor = Some(PcVideoGetPaletteColor);
    MachVtbl.VideoSync = Some(PcVideoSync);
    MachVtbl.Beep = Some(PcBeep);
    MachVtbl.PrepareForReactOS = Some(PcPrepareForReactOS);
    MachVtbl.GetMemoryMap = Some(PcMemGetMemoryMap);
    MachVtbl.GetExtendedBIOSData = Some(PcGetExtendedBIOSData);
    MachVtbl.GetFloppyCount = Some(PcGetFloppyCount);
    MachVtbl.DiskReadLogicalSectors = Some(PcDiskReadLogicalSectors);
    MachVtbl.DiskGetDriveGeometry = Some(PcDiskGetDriveGeometry);
    MachVtbl.DiskGetCacheableBlockCount = Some(PcDiskGetCacheableBlockCount);
    MachVtbl.GetTime = Some(PcGetTime);
    MachVtbl.InitializeBootDevices = Some(PcInitializeBootDevices);
    MachVtbl.HwDetect = Some(PcHwDetect);
    MachVtbl.HwIdle = Some(PcHwIdle);
    _vga_dbg(2, b'C');

    HalpCalibrateStallExecution();
    _vga_dbg(3, b'D');
    PcVideoInit();
    _vga_dbg(4, b'E');
}

#[no_mangle]
pub unsafe extern "C" fn PcPrepareForReactOS() {
    
    PcVideoPrepareForReactOS();
    DiskStopFloppyMotor();
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  *  FreeLoader
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
// #include <cportlib/cportlib.h>
//
// #include "../ntldr/ntldropts.h"
//
// #include <debug.h>
// DBG_DEFAULT_CHANNEL(HWDETECT);
//
//
// /* Maximum number of COM and LPT ports */
// #define MAX_COM_PORTS   4
// #define MAX_LPT_PORTS   3
//
// /* No Mouse */
// #define MOUSE_TYPE_NONE            0
// /* Microsoft Mouse with 2 buttons */
// #define MOUSE_TYPE_MICROSOFT       1
// /* Logitech Mouse with 3 buttons */
// #define MOUSE_TYPE_LOGITECH        2
// /* Microsoft Wheel Mouse (aka Z Mouse) */
// #define MOUSE_TYPE_WHEELZ          3
// /* Mouse Systems Mouse */
// #define MOUSE_TYPE_MOUSESYSTEMS    4
//
// #define INPORT_REGISTER_CONTROL    0x00
// #define INPORT_REGISTER_DATA       0x01
// #define INPORT_REGISTER_SIGNATURE  0x02
//
// #define INPORT_REG_MODE            0x07
// #define INPORT_RESET               0x80
// #define INPORT_MODE_BASE           0x10
// #define INPORT_TEST_IRQ            0x16
// #define INPORT_SIGNATURE           0xDE
//
// #define PIC1_CONTROL_PORT          0x20
// #define PIC1_DATA_PORT             0x21
// #define PIC2_CONTROL_PORT          0xA0
// #define PIC2_DATA_PORT             0xA1
//
// /* PS2 stuff */
//
// /* Controller registers. */
// #define CONTROLLER_REGISTER_STATUS                      0x64
// #define CONTROLLER_REGISTER_CONTROL                     0x64
// #define CONTROLLER_REGISTER_DATA                        0x60
//
// /* Controller commands. */
// #define CONTROLLER_COMMAND_READ_MODE                    0x20
// #define CONTROLLER_COMMAND_WRITE_MODE                   0x60
// #define CONTROLLER_COMMAND_GET_VERSION                  0xA1
// #define CONTROLLER_COMMAND_MOUSE_DISABLE                0xA7
// #define CONTROLLER_COMMAND_MOUSE_ENABLE                 0xA8
// #define CONTROLLER_COMMAND_TEST_MOUSE                   0xA9
// #define CONTROLLER_COMMAND_SELF_TEST                    0xAA
// #define CONTROLLER_COMMAND_KEYBOARD_TEST                0xAB
// #define CONTROLLER_COMMAND_KEYBOARD_DISABLE             0xAD
// #define CONTROLLER_COMMAND_KEYBOARD_ENABLE              0xAE
// #define CONTROLLER_COMMAND_WRITE_MOUSE_OUTPUT_BUFFER    0xD3
// #define CONTROLLER_COMMAND_WRITE_MOUSE                  0xD4
//
// /* Controller status */
// #define CONTROLLER_STATUS_OUTPUT_BUFFER_FULL            0x01
// #define CONTROLLER_STATUS_INPUT_BUFFER_FULL             0x02
// #define CONTROLLER_STATUS_SELF_TEST                     0x04
// #define CONTROLLER_STATUS_COMMAND                       0x08
// #define CONTROLLER_STATUS_UNLOCKED                      0x10
// #define CONTROLLER_STATUS_MOUSE_OUTPUT_BUFFER_FULL      0x20
// #define CONTROLLER_STATUS_GENERAL_TIMEOUT               0x40
// #define CONTROLLER_STATUS_PARITY_ERROR                  0x80
// #define AUX_STATUS_OUTPUT_BUFFER_FULL                   (CONTROLLER_STATUS_OUTPUT_BUFFER_FULL | \
//                                                          CONTROLLER_STATUS_MOUSE_OUTPUT_BUFFER_FULL)
//
// /* Timeout in ms for sending to keyboard controller. */
// #define CONTROLLER_TIMEOUT                              250
//
// #include <pshpack1.h>
// typedef struct _PNP_DOCK_INFO
// {
//     ULONG DockLocationID;
//     ULONG SerialNumber;
//     USHORT Capabilities;
// } PNP_DOCK_INFO, *PPNP_DOCK_INFO;
// #include <poppack.h>
//
// /* FIXME: Abstract things better so we don't need to place define here */
//
// VOID
// PcGetExtendedBIOSData(PULONG ExtendedBIOSDataArea, PULONG ExtendedBIOSDataSize)
// {
//     REGS BiosRegs;
//
//     /* Get address and size of the extended BIOS data area */
//     BiosRegs.d.eax = 0xC100;
//     Int386(0x15, &BiosRegs, &BiosRegs);
//     if (INT386_SUCCESS(BiosRegs))
//     {
//         *ExtendedBIOSDataArea = BiosRegs.w.es << 4;
//         *ExtendedBIOSDataSize = 1024;
//     }
//     else
//     {
//         WARN("Int 15h AH=C1h call failed\n");
//         *ExtendedBIOSDataArea = 0;
//         *ExtendedBIOSDataSize = 0;
//     }
// }
//
// // NOTE: Similar to machxbox.c!XboxGetHarddiskConfigurationData(),
// // but with extended geometry support.
// static
// PCM_PARTIAL_RESOURCE_LIST
// PcGetHarddiskConfigurationData(UCHAR DriveNumber, ULONG* pSize)
// {
//     PCM_PARTIAL_RESOURCE_LIST PartialResourceList;
//     PCM_DISK_GEOMETRY_DEVICE_DATA DiskGeometry;
//     GEOMETRY Geometry;
//     ULONG Size;
//
//     /* Initialize returned size */
//     *pSize = 0;
//
//     /* Set 'Configuration Data' value */
//     Size = crate::FIELD_OFFSET!(CM_PARTIAL_RESOURCE_LIST, PartialDescriptors[1]) + sizeof(*DiskGeometry);
//     PartialResourceList = FrLdrHeapAlloc(Size, TAG_HW_RESOURCE_LIST);
//     if (PartialResourceList == NULL)
//     {
//         ERR("Failed to allocate resource descriptor\n");
//         return NULL;
//     }
//
//     RtlZeroMemory(PartialResourceList, Size);
//     PartialResourceList->Version = 1;
//     PartialResourceList->Revision = 1;
//     PartialResourceList->Count = 1;
//     PartialResourceList->PartialDescriptors[0].Type =
//         CmResourceTypeDeviceSpecific;
// //  PartialResourceList->PartialDescriptors[0].ShareDisposition =
// //  PartialResourceList->PartialDescriptors[0].Flags =
//     PartialResourceList->PartialDescriptors[0].u.DeviceSpecificData.DataSize =
//         sizeof(*DiskGeometry);
//
//     /* Get the disk geometry */
//     DiskGeometry = (PCM_DISK_GEOMETRY_DEVICE_DATA)&PartialResourceList->PartialDescriptors[1];
//     if (PcDiskGetDriveGeometry(DriveNumber, &Geometry))
//     {
//         DiskGeometry->BytesPerSector = Geometry.BytesPerSector;
//         DiskGeometry->NumberOfCylinders = Geometry.Cylinders;
//         DiskGeometry->SectorsPerTrack = Geometry.SectorsPerTrack;
//         DiskGeometry->NumberOfHeads = Geometry.Heads;
//     }
//     else
//     {
//         TRACE("Reading disk geometry failed\n");
//         FrLdrHeapFree(PartialResourceList, TAG_HW_RESOURCE_LIST);
//         return NULL;
//     }
//     TRACE("Disk %x: %u Cylinders  %u Heads  %u Sectors  %u Bytes\n", (//           DriveNumber) as usize, (//           DiskGeometry->NumberOfCylinders) as usize, (//           DiskGeometry->NumberOfHeads) as usize, (//           DiskGeometry->SectorsPerTrack) as usize, (//           DiskGeometry->BytesPerSector) as usize);
//
//     /* Return configuration data */
//     *pSize = Size;
//     return PartialResourceList;
// }
//
// static
// VOID
// DetectDockingStation(
//     _Inout_ PCONFIGURATION_COMPONENT_DATA BusKey)
// {
//     PCM_PARTIAL_RESOURCE_LIST PartialResourceList;
//     PCM_PARTIAL_RESOURCE_DESCRIPTOR PartialDescriptor;
//     PCONFIGURATION_COMPONENT_DATA PeripheralKey;
//     PDOCKING_STATE_INFORMATION DockingState;
//     PPNP_DOCK_INFO DockInfo;
//     ULONG Size, Result;
//
//     Result = PnpBiosGetDockStationInformation(DiskReadBuffer);
//
//     /* Build full device descriptor */
//     Size = crate::FIELD_OFFSET!(CM_PARTIAL_RESOURCE_LIST, PartialDescriptors[1]) + sizeof(*DockingState);
//     PartialResourceList = FrLdrHeapAlloc(Size, TAG_HW_RESOURCE_LIST);
//     if (PartialResourceList == NULL)
//     {
//         ERR("Failed to allocate resource descriptor\n");
//         return;
//     }
//
//     /* Initialize resource descriptor */
//     RtlZeroMemory(PartialResourceList, Size);
//     PartialResourceList->Version = 0;
//     PartialResourceList->Revision = 0;
//     PartialResourceList->Count = 1;
//
//     /* Set device specific data */
//     PartialDescriptor = &PartialResourceList->PartialDescriptors[0];
//     PartialDescriptor->Type = CmResourceTypeDeviceSpecific;
//     PartialDescriptor->ShareDisposition = CmResourceShareUndetermined;
//     PartialDescriptor->Flags = 0;
//     PartialDescriptor->u.DeviceSpecificData.DataSize = sizeof(*DockingState);
//
//     DockingState = (PDOCKING_STATE_INFORMATION)(PartialDescriptor + 1);
//     DockingState->ReturnCode = Result;
//     if (Result == 0)
//     {
//         DockInfo = (PPNP_DOCK_INFO)DiskReadBuffer;
//         DockingState->DockLocationID = DockInfo->DockLocationID;
//         DockingState->SerialNumber = DockInfo->SerialNumber;
//         DockingState->Capabilities = DockInfo->Capabilities;
//         TRACE("System docked\n");
//         TRACE("Location: 0x%08lx\n", (DockInfo->DockLocationID) as usize);
//         TRACE("Serial: 0x%08lx\n", (DockInfo->SerialNumber) as usize);
//         TRACE("Capabilities: 0x%04hx\n", (DockInfo->Capabilities) as usize);
//     }
//
//     /* Create controller key */
//     FldrCreateComponentKey(BusKey,
//                            PeripheralClass,
//                            DockingInformation,
//                            0,
//                            0,
//                            0xFFFFFFFF,
//                            "Docking State Information",
//                            PartialResourceList,
//                            Size,
//                            &PeripheralKey);
// }
//
// static
// VOID
// DetectPnpBios(PCONFIGURATION_COMPONENT_DATA SystemKey, ULONG *BusNumber)
// {
//     PCM_PARTIAL_RESOURCE_LIST PartialResourceList;
//     PCM_PNP_BIOS_DEVICE_NODE DeviceNode;
//     PCM_PNP_BIOS_INSTALLATION_CHECK InstData;
//     PCONFIGURATION_COMPONENT_DATA BusKey;
//     ULONG x;
//     ULONG NodeSize = 0;
//     ULONG NodeCount = 0;
//     UCHAR NodeNumber;
//     ULONG FoundNodeCount;
//     int i;
//     ULONG PnpBufferSize;
//     ULONG PnpBufferSizeLimit;
//     ULONG Size;
//     char* Ptr;
//
//     InstData = (PCM_PNP_BIOS_INSTALLATION_CHECK)PnpBiosSupported();
//     if (InstData == NULL || strncmp((CHAR*)InstData->Signature, "$PnP", 4))
//     {
//         TRACE("PnP-BIOS not supported\n");
//         return;
//     }
//
//     TRACE("PnP-BIOS supported\n");
//     TRACE("Signature '%c%c%c%c'\n", (//           InstData->Signature[0]) as usize, (InstData->Signature[1]) as usize, (//           InstData->Signature[2]) as usize, (InstData->Signature[3]) as usize);
//
//     x = PnpBiosGetDeviceNodeCount(&NodeSize, &NodeCount);
//     if (x == 0x82)
//     {
//         TRACE("PnP-BIOS function 'Get Number of System Device Nodes' not supported\n");
//         return;
//     }
//
//     NodeCount &= 0xFF; // needed since some fscked up BIOSes return wrong info (e.g. Mac Virtual PC)
//     // e.g. look: https://web.archive.org/web/20080329010332/http://my.execpc.com/~geezer/osd/pnp/pnp16.c
//     if (x != 0 || NodeSize == 0 || NodeCount == 0)
//     {
//         ERR("PnP-BIOS failed to enumerate device nodes\n");
//         return;
//     }
//     TRACE("MaxNodeSize %u  NodeCount %u\n", (NodeSize) as usize, (NodeCount) as usize);
//     TRACE("Estimated buffer size %u\n", (NodeSize * NodeCount) as usize);
//
//     /* Set 'Configuration Data' value */
//     PnpBufferSizeLimit = sizeof(*InstData) + (NodeSize * NodeCount);
//     Size = crate::FIELD_OFFSET!(CM_PARTIAL_RESOURCE_LIST, PartialDescriptors[1]) + PnpBufferSizeLimit;
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
//     PartialResourceList->PartialDescriptors[0].Type =
//         CmResourceTypeDeviceSpecific;
//     PartialResourceList->PartialDescriptors[0].ShareDisposition =
//         CmResourceShareUndetermined;
//
//     /* Set installation check data */
//     Ptr = (char*)&PartialResourceList->PartialDescriptors[1];
//     RtlCopyMemory(Ptr, InstData, sizeof(*InstData));
//     Ptr += sizeof(*InstData);
//     PnpBufferSize = sizeof(*InstData);
//
//     /* Copy device nodes */
//     FoundNodeCount = 0;
//     for (i = 0; i < 0xFF; i++)
//     {
//         NodeNumber = (UCHAR)i;
//
//         x = PnpBiosGetDeviceNode(&NodeNumber, DiskReadBuffer);
//         if (x == 0)
//         {
//             DeviceNode = (PCM_PNP_BIOS_DEVICE_NODE)DiskReadBuffer;
//
//             TRACE("Node: %u  Size %u (0x%x)\n", (//                   DeviceNode->Node) as usize, (//                   DeviceNode->Size) as usize, (//                   DeviceNode->Size) as usize);
//
//             if (PnpBufferSize + DeviceNode->Size > PnpBufferSizeLimit)
//             {
//                 ERR("Buffer too small! Ignoring remaining device nodes. (i = %d)\n", (i) as usize);
//                 break;
//             }
//
//             RtlCopyMemory(Ptr, DeviceNode, DeviceNode->Size);
//
//             Ptr += DeviceNode->Size;
//             PnpBufferSize += DeviceNode->Size;
//
//             FoundNodeCount++;
//             if (FoundNodeCount >= NodeCount)
//                 break;
//         }
//     }
//
//     /* Set real data size */
//     PartialResourceList->PartialDescriptors[0].u.DeviceSpecificData.DataSize =
//         PnpBufferSize;
//     Size = crate::FIELD_OFFSET!(CM_PARTIAL_RESOURCE_LIST, PartialDescriptors[1]) + PnpBufferSize;
//
//     TRACE("Real buffer size: %u\n", (PnpBufferSize) as usize);
//     TRACE("Resource size: %u\n", (Size) as usize);
//
//     /* Create component key */
//     FldrCreateComponentKey(SystemKey,
//                            AdapterClass,
//                            MultiFunctionAdapter,
//                            0,
//                            0,
//                            0xFFFFFFFF,
//                            "PNP BIOS",
//                            PartialResourceList,
//                            Size,
//                            &BusKey);
//
//     DetectDockingStation(BusKey);
//
//     (*BusNumber)++;
// }
//
//
// static
// VOID
// InitializeSerialPort(PUCHAR Port,
//                      UCHAR LineControl)
// {
//     WRITE_PORT_UCHAR(Port + 3, 0x80);  /* set DLAB on   */
//     WRITE_PORT_UCHAR(Port,     0x60);  /* speed LO byte */
//     WRITE_PORT_UCHAR(Port + 1, 0);     /* speed HI byte */
//     WRITE_PORT_UCHAR(Port + 3, LineControl);
//     WRITE_PORT_UCHAR(Port + 1, 0);     /* set comm and DLAB to 0 */
//     WRITE_PORT_UCHAR(Port + 4, 0x09);  /* DR int enable */
//     READ_PORT_UCHAR(Port + 5);  /* clear error bits */
// }
//
// static
// ULONG
// DetectSerialMouse(PUCHAR Port)
// {
//     CHAR Buffer[4];
//     ULONG i;
//     ULONG TimeOut;
//     UCHAR LineControl;
//
//     /* Shutdown mouse or something like that */
//     LineControl = READ_PORT_UCHAR(Port + 4);
//     WRITE_PORT_UCHAR(Port + 4, (LineControl & ~0x02) | 0x01);
//     StallExecutionProcessor(100000);
//
//     /*
//      * Clear buffer
//      * Maybe there is no serial port although BIOS reported one (this
//      * is the case on Apple hardware), or the serial port is misbehaving,
//      * therefore we must give up after some time.
//      */
//     TimeOut = 200;
//     while (READ_PORT_UCHAR(Port + 5) & 0x01)
//     {
//         if (--TimeOut == 0)
//             return MOUSE_TYPE_NONE;
//         READ_PORT_UCHAR(Port);
//     }
//
//     /*
//      * Send modem control with 'Data Terminal Ready', 'Request To Send' and
//      * 'Output Line 2' message. This enables mouse to identify.
//      */
//     WRITE_PORT_UCHAR(Port + 4, 0x0b);
//
//     /* Wait 10 milliseconds for the mouse getting ready */
//     StallExecutionProcessor(10000);
//
//     /* Read first four bytes, which contains Microsoft Mouse signs */
//     TimeOut = 20;
//     for (i = 0; i < 4; i++)
//     {
//         while ((READ_PORT_UCHAR(Port + 5) & 1) == 0)
//         {
//             StallExecutionProcessor(100);
//             --TimeOut;
//             if (TimeOut == 0)
//                 return MOUSE_TYPE_NONE;
//         }
//         Buffer[i] = READ_PORT_UCHAR(Port);
//     }
//
//     TRACE("Mouse data: %x %x %x %x\n", (//           Buffer[0]) as usize, (Buffer[1]) as usize, (Buffer[2]) as usize, (Buffer[3]) as usize);
//
//     /* Check that four bytes for signs */
//     for (i = 0; i < 4; ++i)
//     {
//         if (Buffer[i] == 'B')
//         {
//             /* Sign for Microsoft Ballpoint */
// //      DbgPrint("Microsoft Ballpoint device detected\n");
// //      DbgPrint("THIS DEVICE IS NOT SUPPORTED, YET\n");
//             return MOUSE_TYPE_NONE;
//         }
//         else if (Buffer[i] == 'M')
//         {
//             /* Sign for Microsoft Mouse protocol followed by button specifier */
//             if (i == 3)
//             {
//                 /* Overflow Error */
//                 return MOUSE_TYPE_NONE;
//             }
//
//             switch (Buffer[i + 1])
//             {
//             case '3':
//                 TRACE("Microsoft Mouse with 3-buttons detected\n");
//                 return MOUSE_TYPE_LOGITECH;
//
//             case 'Z':
//                 TRACE("Microsoft Wheel Mouse detected\n");
//                 return MOUSE_TYPE_WHEELZ;
//
//                 /* case '2': */
//             default:
//                 TRACE("Microsoft Mouse with 2-buttons detected\n");
//                 return MOUSE_TYPE_MICROSOFT;
//             }
//         }
//     }
//
//     return MOUSE_TYPE_NONE;
// }
//
// static ULONG
// GetSerialMousePnpId(PUCHAR Port, char *Buffer)
// {
//     ULONG TimeOut;
//     ULONG i = 0;
//     char c;
//     char x;
//
//     WRITE_PORT_UCHAR(Port + 4, 0x09);
//
//     /* Wait 10 milliseconds for the mouse getting ready */
//     StallExecutionProcessor(10000);
//
//     WRITE_PORT_UCHAR(Port + 4, 0x0b);
//
//     StallExecutionProcessor(10000);
//
//     for (;;)
//     {
//         TimeOut = 200;
//         while (((READ_PORT_UCHAR(Port + 5) & 1) == 0) && (TimeOut > 0))
//         {
//             StallExecutionProcessor(1000);
//             --TimeOut;
//             if (TimeOut == 0)
//             {
//                 return 0;
//             }
//         }
//
//         c = READ_PORT_UCHAR(Port);
//         if (c == 0x08 || c == 0x28)
//             break;
//     }
//
//     Buffer[i++] = c;
//     x = c + 1;
//
//     for (;;)
//     {
//         TimeOut = 200;
//         while (((READ_PORT_UCHAR(Port + 5) & 1) == 0) && (TimeOut > 0))
//         {
//             StallExecutionProcessor(1000);
//             --TimeOut;
//             if (TimeOut == 0)
//                 return 0;
//         }
//         c = READ_PORT_UCHAR(Port);
//         Buffer[i++] = c;
//         if (c == x)
//             break;
//         if (i >= 256)
//             break;
//     }
//
//     return i;
// }
//
// static
// VOID
// DetectSerialPointerPeripheral(PCONFIGURATION_COMPONENT_DATA ControllerKey,
//                               PUCHAR Base)
// {
//     PCM_PARTIAL_RESOURCE_LIST PartialResourceList;
//     char Buffer[256];
//     CHAR Identifier[256];
//     PCONFIGURATION_COMPONENT_DATA PeripheralKey;
//     ULONG MouseType;
//     ULONG Size, Length;
//     ULONG i;
//     ULONG j;
//     ULONG k;
//
//     TRACE("DetectSerialPointerPeripheral()\n");
//
//     Identifier[0] = 0;
//
//     InitializeSerialPort(Base, 2);
//     MouseType = DetectSerialMouse(Base);
//
//     if (MouseType != MOUSE_TYPE_NONE)
//     {
//         Length = GetSerialMousePnpId(Base, Buffer);
//         TRACE( "PnP ID length: %u\n", (Length) as usize);
//
//         if (Length != 0)
//         {
//             /* Convert PnP sting to ASCII */
//             if (Buffer[0] == 0x08)
//             {
//                 for (i = 0; i < Length; i++)
//                     Buffer[i] += 0x20;
//             }
//             Buffer[Length] = 0;
//
//             TRACE("PnP ID string: %s\n", (Buffer) as usize);
//
//             /* Copy PnpId string */
//             for (i = 0; i < 7; i++)
//             {
//                 Identifier[i] = Buffer[3 + i];
//             }
//             memcpy(&Identifier[7],
//                    L" - ",
//                    3 * sizeof(WCHAR));
//
//             /* Skip device serial number */
//             i = 10;
//             if (Buffer[i] == '\\')
//             {
//                 for (j = ++i; i < Length; ++i)
//                 {
//                     if (Buffer[i] == '\\')
//                         break;
//                 }
//                 if (i >= Length)
//                     i -= 3;
//             }
//
//             /* Skip PnP class */
//             if (Buffer[i] == '\\')
//             {
//                 for (j = ++i; i < Length; ++i)
//                 {
//                     if (Buffer[i] == '\\')
//                         break;
//                 }
//
//                 if (i >= Length)
//                     i -= 3;
//             }
//
//             /* Skip compatible PnP Id */
//             if (Buffer[i] == '\\')
//             {
//                 for (j = ++i; i < Length; ++i)
//                 {
//                     if (Buffer[i] == '\\')
//                         break;
//                 }
//                 if (Buffer[j] == '*')
//                     ++j;
//                 if (i >= Length)
//                     i -= 3;
//             }
//
//             /* Get product description */
//             if (Buffer[i] == '\\')
//             {
//                 for (j = ++i; i < Length; ++i)
//                 {
//                     if (Buffer[i] == ';')
//                         break;
//                 }
//                 if (i >= Length)
//                     i -= 3;
//                 if (i > j + 1)
//                 {
//                     for (k = 0; k < i - j; k++)
//                     {
//                         Identifier[k + 10] = Buffer[k + j];
//                     }
//                     Identifier[10 + (i - j)] = 0;
//                 }
//             }
//
//             TRACE("Identifier string: %s\n", (Identifier) as usize);
//         }
//
//         if (Length == 0 || strlen(Identifier) < 11)
//         {
//             switch (MouseType)
//             {
//             case MOUSE_TYPE_LOGITECH:
//                 strcpy(Identifier, "LOGITECH SERIAL MOUSE");
//                 break;
//
//             case MOUSE_TYPE_WHEELZ:
//                 strcpy(Identifier, "MICROSOFT SERIAL MOUSE WITH WHEEL");
//                 break;
//
//             case MOUSE_TYPE_MICROSOFT:
//             default:
//                 strcpy(Identifier, "MICROSOFT SERIAL MOUSE");
//                 break;
//             }
//         }
//
//         /* Set 'Configuration Data' value */
//         Size = crate::FIELD_OFFSET!(CM_PARTIAL_RESOURCE_LIST, PartialDescriptors);
//         PartialResourceList = FrLdrHeapAlloc(Size, TAG_HW_RESOURCE_LIST);
//         if (PartialResourceList == NULL)
//         {
//             ERR("Failed to allocate resource descriptor\n");
//             return;
//         }
//
//         RtlZeroMemory(PartialResourceList, Size);
//         PartialResourceList->Version = 1;
//         PartialResourceList->Revision = 1;
//         PartialResourceList->Count = 0;
//
//         /* Create 'PointerPeripheral' key */
//         FldrCreateComponentKey(ControllerKey,
//                                PeripheralClass,
//                                PointerPeripheral,
//                                Input,
//                                0,
//                                0xFFFFFFFF,
//                                Identifier,
//                                PartialResourceList,
//                                Size,
//                                &PeripheralKey);
//     }
// }
//
// /* FIXME: Abstract things better so we don't need to place define here */
// static
// ULONG
// PcGetSerialPort(ULONG Index, PULONG Irq)
// {
//     static const ULONG PcIrq[MAX_COM_PORTS] = {4, 3, 4, 3};
//     PUSHORT BasePtr;
//
//     /*
//      * The BIOS data area 0x400 holds the address of the first valid COM port.
//      * Each COM port address is stored in a 2-byte field.
//      * Infos at: https://web.archive.org/web/20240119203029/http://www.bioscentral.com/misc/bda.htm
//      */
//     BasePtr = (PUSHORT)0x400;
//     *Irq = PcIrq[Index];
//
//     return (ULONG) *(BasePtr + Index);
// }
//
// /*
//  * Parse the serial mouse detection options.
//  * Format: /FASTDETECT
//  * or: /NOSERIALMICE=COM[0-9],[0-9],[0-9]...
//  * or: /NOSERIALMICE:COM[0-9]...
//  * If we have /FASTDETECT, then nothing can be detected.
//  */
// static
// ULONG
// GetSerialMouseDetectionBitmap(
//     _In_opt_ PCSTR Options)
// {
//     PCSTR Option, c;
//     ULONG OptionLength, PortBitmap, i;
//
//     if (NtLdrGetOption(Options, "FASTDETECT"))
//         return (1 << MAX_COM_PORTS) - 1;
//
//     Option = NtLdrGetOptionEx(Options, "NOSERIALMICE=", &OptionLength);
//     if (!Option)
//         Option = NtLdrGetOptionEx(Options, "NOSERIALMICE:", &OptionLength);
//
//     if (!Option)
//         return 0;
//
//     /* Invalid port list */
//     if (OptionLength < (sizeof("NOSERIALMICE=COM9") - 1))
//         return (1 << MAX_COM_PORTS) - 1;
//
//     /* Move to the port list */
//     Option += sizeof("NOSERIALMICE=COM") - 1;
//     OptionLength -= sizeof("NOSERIALMICE=COM") - 1;
//
//     PortBitmap = 0;
//     c = Option;
//     for (i = 0; i < OptionLength; i += 2)
//     {
//         UCHAR PortNumber = *c - '0';
//
//         if (PortNumber > 0 && PortNumber <= 9)
//             PortBitmap |= 1 << (PortNumber - 1);
//
//         c += 2;
//     }
//
//     return PortBitmap;
// }
//
// VOID
// DetectSerialPorts(
//     _In_opt_ PCSTR Options,
//     _Inout_ PCONFIGURATION_COMPONENT_DATA BusKey,
//     _In_ GET_SERIAL_PORT MachGetSerialPort,
//     _In_ ULONG Count)
// {
//     PCM_PARTIAL_RESOURCE_LIST PartialResourceList;
//     PCM_PARTIAL_RESOURCE_DESCRIPTOR PartialDescriptor;
//     PCM_SERIAL_DEVICE_DATA SerialDeviceData;
//     ULONG Irq;
//     ULONG Base;
//     CHAR Identifier[80];
//     ULONG ControllerNumber = 0;
//     PCONFIGURATION_COMPONENT_DATA ControllerKey;
//     ULONG i;
//     ULONG Size;
//     ULONG PortBitmap;
//
//     TRACE("DetectSerialPorts()\n");
//
//     PortBitmap = GetSerialMouseDetectionBitmap(Options);
//
//     for (i = 0; i < Count; i++)
//     {
//         Base = MachGetSerialPort(i, &Irq);
//         if ((Base == 0) || !CpDoesPortExist(UlongToPtr(Base)))
//             continue;
//
//         TRACE("Found COM%u port at 0x%x\n", (i + 1) as usize, (Base) as usize);
//
//         /* Set 'Identifier' value */
//         RtlStringCbPrintfA(Identifier, sizeof(Identifier), "COM%ld", i + 1);
//
//         /* Build full device descriptor */
//         Size = crate::FIELD_OFFSET!(CM_PARTIAL_RESOURCE_LIST, PartialDescriptors[3]) + sizeof(*SerialDeviceData);
//         PartialResourceList = FrLdrHeapAlloc(Size, TAG_HW_RESOURCE_LIST);
//         if (PartialResourceList == NULL)
//         {
//             ERR("Failed to allocate resource descriptor! Ignoring remaining serial ports. (i = %lu, Count = %lu)\n", (//                 i) as usize, (Count) as usize);
//             break;
//         }
//
//         /* Initialize resource descriptor */
//         RtlZeroMemory(PartialResourceList, Size);
//         PartialResourceList->Version = 1;
//         PartialResourceList->Revision = 1;
//         PartialResourceList->Count = 3;
//
//         /* Set IO Port */
//         PartialDescriptor = &PartialResourceList->PartialDescriptors[0];
//         PartialDescriptor->Type = CmResourceTypePort;
//         PartialDescriptor->ShareDisposition = CmResourceShareDeviceExclusive;
//         PartialDescriptor->Flags = CM_RESOURCE_PORT_IO;
//         PartialDescriptor->u.Port.Start.LowPart = Base;
//         PartialDescriptor->u.Port.Start.HighPart = 0x0;
//         PartialDescriptor->u.Port.Length = 8;
//
//         /* Set Interrupt */
//         PartialDescriptor = &PartialResourceList->PartialDescriptors[1];
//         PartialDescriptor->Type = CmResourceTypeInterrupt;
//         PartialDescriptor->ShareDisposition = CmResourceShareUndetermined;
//         PartialDescriptor->Flags = CM_RESOURCE_INTERRUPT_LATCHED;
//         PartialDescriptor->u.Interrupt.Level = Irq;
//         PartialDescriptor->u.Interrupt.Vector = Irq;
//         PartialDescriptor->u.Interrupt.Affinity = 0xFFFFFFFF;
//
//         /* Set serial data (device specific) */
//         PartialDescriptor = &PartialResourceList->PartialDescriptors[2];
//         PartialDescriptor->Type = CmResourceTypeDeviceSpecific;
//         PartialDescriptor->ShareDisposition = CmResourceShareUndetermined;
//         PartialDescriptor->Flags = 0;
//         PartialDescriptor->u.DeviceSpecificData.DataSize = sizeof(*SerialDeviceData);
//
//         SerialDeviceData = (PCM_SERIAL_DEVICE_DATA)(PartialDescriptor + 1);
//         SerialDeviceData->BaudClock = 1843200; /* UART Clock frequency (Hertz) */
//
//         /* Create controller key */
//         FldrCreateComponentKey(BusKey,
//                                ControllerClass,
//                                SerialController,
//                                Output | Input | ConsoleIn | ConsoleOut,
//                                ControllerNumber,
//                                0xFFFFFFFF,
//                                Identifier,
//                                PartialResourceList,
//                                Size,
//                                &ControllerKey);
//
//         if (!(PortBitmap & (1 << i)) && !Rs232PortInUse(UlongToPtr(Base)))
//         {
//             /* Detect serial mouse */
//             DetectSerialPointerPeripheral(ControllerKey, UlongToPtr(Base));
//         }
//
//         ControllerNumber++;
//     }
// }
//
// /* FIXME: Abstract things better so we don't need to place define here */
//
// static VOID
// DetectParallelPorts(PCONFIGURATION_COMPONENT_DATA BusKey)
// {
//     PCM_PARTIAL_RESOURCE_LIST PartialResourceList;
//     PCM_PARTIAL_RESOURCE_DESCRIPTOR PartialDescriptor;
//     ULONG Irq[MAX_LPT_PORTS] = {7, 5, (ULONG)-1};
//     CHAR Identifier[80];
//     PCONFIGURATION_COMPONENT_DATA ControllerKey;
//     PUSHORT BasePtr;
//     ULONG Base;
//     ULONG ControllerNumber = 0;
//     ULONG i;
//     ULONG Size;
//
//     TRACE("DetectParallelPorts() called\n");
//
//     /*
//      * The BIOS data area 0x408 holds the address of the first valid LPT port.
//      * Each LPT port address is stored in a 2-byte field.
//      * Infos at: https://web.archive.org/web/20240119203029/http://www.bioscentral.com/misc/bda.htm
//      */
//     BasePtr = (PUSHORT)0x408;
//
//     for (i = 0; i < MAX_LPT_PORTS; i++, BasePtr++)
//     {
//         Base = (ULONG) * BasePtr;
//         if (Base == 0)
//             continue;
//
//         TRACE("Parallel port %u: %x\n", (ControllerNumber) as usize, (Base) as usize);
//
//         /* Set 'Identifier' value */
//         RtlStringCbPrintfA(Identifier, sizeof(Identifier), "PARALLEL%ld", i + 1);
//
//         /* Build full device descriptor */
//         Size = crate::FIELD_OFFSET!(CM_PARTIAL_RESOURCE_LIST, PartialDescriptors[1]);
//         if (Irq[i] != (ULONG)-1)
//             Size += sizeof(CM_PARTIAL_RESOURCE_DESCRIPTOR);
//
//         PartialResourceList = FrLdrHeapAlloc(Size, TAG_HW_RESOURCE_LIST);
//         if (PartialResourceList == NULL)
//         {
//             ERR("Failed to allocate resource descriptor! Ignoring remaining parallel ports. (i = %lu)\n", (i) as usize);
//             break;
//         }
//
//         /* Initialize resource descriptor */
//         RtlZeroMemory(PartialResourceList, Size);
//         PartialResourceList->Version = 1;
//         PartialResourceList->Revision = 1;
//         PartialResourceList->Count = (Irq[i] != (ULONG)-1) ? 2 : 1;
//
//         /* Set IO Port */
//         PartialDescriptor = &PartialResourceList->PartialDescriptors[0];
//         PartialDescriptor->Type = CmResourceTypePort;
//         PartialDescriptor->ShareDisposition = CmResourceShareDeviceExclusive;
//         PartialDescriptor->Flags = CM_RESOURCE_PORT_IO;
//         PartialDescriptor->u.Port.Start.LowPart = Base;
//         PartialDescriptor->u.Port.Start.HighPart = 0x0;
//         PartialDescriptor->u.Port.Length = 3;
//
//         /* Set Interrupt */
//         if (Irq[i] != (ULONG)-1)
//         {
//             PartialDescriptor = &PartialResourceList->PartialDescriptors[1];
//             PartialDescriptor->Type = CmResourceTypeInterrupt;
//             PartialDescriptor->ShareDisposition = CmResourceShareUndetermined;
//             PartialDescriptor->Flags = CM_RESOURCE_INTERRUPT_LATCHED;
//             PartialDescriptor->u.Interrupt.Level = Irq[i];
//             PartialDescriptor->u.Interrupt.Vector = Irq[i];
//             PartialDescriptor->u.Interrupt.Affinity = 0xFFFFFFFF;
//         }
//
//         /* Create controller key */
//         FldrCreateComponentKey(BusKey,
//                                ControllerClass,
//                                ParallelController,
//                                Output,
//                                ControllerNumber,
//                                0xFFFFFFFF,
//                                Identifier,
//                                PartialResourceList,
//                                Size,
//                                &ControllerKey);
//
//         ControllerNumber++;
//     }
//
//     TRACE("DetectParallelPorts() done\n");
// }
//
// // static
// BOOLEAN
// DetectKeyboardDevice(VOID)
// {
//     UCHAR Status;
//     UCHAR Scancode;
//     ULONG Loops;
//     BOOLEAN Result = TRUE;
//
//     /* Identify device */
//     WRITE_PORT_UCHAR((PUCHAR)CONTROLLER_REGISTER_DATA, 0xF2);
//
//     /* Wait for reply */
//     for (Loops = 0; Loops < 100; Loops++)
//     {
//         StallExecutionProcessor(10000);
//         Status = READ_PORT_UCHAR((PUCHAR)CONTROLLER_REGISTER_STATUS);
//         if ((Status & CONTROLLER_STATUS_OUTPUT_BUFFER_FULL) != 0)
//             break;
//     }
//
//     if ((Status & CONTROLLER_STATUS_OUTPUT_BUFFER_FULL) == 0)
//     {
//         /* PC/XT keyboard or no keyboard */
//         Result = FALSE;
//     }
//
//     Scancode = READ_PORT_UCHAR((PUCHAR)CONTROLLER_REGISTER_DATA);
//     if (Scancode != 0xFA)
//     {
//         /* No ACK received */
//         Result = FALSE;
//     }
//
//     StallExecutionProcessor(10000);
//
//     Status = READ_PORT_UCHAR((PUCHAR)CONTROLLER_REGISTER_STATUS);
//     if ((Status & CONTROLLER_STATUS_OUTPUT_BUFFER_FULL) == 0)
//     {
//         /* Found AT keyboard */
//         return Result;
//     }
//
//     Scancode = READ_PORT_UCHAR((PUCHAR)CONTROLLER_REGISTER_DATA);
//     if (Scancode != 0xAB)
//     {
//         /* No 0xAB received */
//         Result = FALSE;
//     }
//
//     StallExecutionProcessor(10000);
//
//     Status = READ_PORT_UCHAR((PUCHAR)CONTROLLER_REGISTER_STATUS);
//     if ((Status & CONTROLLER_STATUS_OUTPUT_BUFFER_FULL) == 0)
//     {
//         /* No byte in buffer */
//         Result = FALSE;
//     }
//
//     Scancode = READ_PORT_UCHAR((PUCHAR)CONTROLLER_REGISTER_DATA);
//     if (Scancode != 0x41)
//     {
//         /* No 0x41 received */
//         Result = FALSE;
//     }
//
//     /* Found MF-II keyboard */
//     return Result;
// }
//
// static VOID
// DetectKeyboardPeripheral(PCONFIGURATION_COMPONENT_DATA ControllerKey)
// {
//     PCM_PARTIAL_RESOURCE_LIST PartialResourceList;
//     PCM_PARTIAL_RESOURCE_DESCRIPTOR PartialDescriptor;
//     PCM_KEYBOARD_DEVICE_DATA KeyboardData;
//     PCONFIGURATION_COMPONENT_DATA PeripheralKey;
//     ULONG Size;
//     REGS Regs;
//
//     /* HACK: don't call DetectKeyboardDevice() as it fails in Qemu 0.8.2
//     if (DetectKeyboardDevice()) */
//     {
//         /* Set 'Configuration Data' value */
//         Size = crate::FIELD_OFFSET!(CM_PARTIAL_RESOURCE_LIST, PartialDescriptors[1]) + sizeof(*KeyboardData);
//         PartialResourceList = FrLdrHeapAlloc(Size, TAG_HW_RESOURCE_LIST);
//         if (PartialResourceList == NULL)
//         {
//             ERR("Failed to allocate resource descriptor\n");
//             return;
//         }
//
//         /* Initialize resource descriptor */
//         RtlZeroMemory(PartialResourceList, Size);
//         PartialResourceList->Version = 1;
//         PartialResourceList->Revision = 1;
//         PartialResourceList->Count = 1;
//
//         PartialDescriptor = &PartialResourceList->PartialDescriptors[0];
//         PartialDescriptor->Type = CmResourceTypeDeviceSpecific;
//         PartialDescriptor->ShareDisposition = CmResourceShareUndetermined;
//         PartialDescriptor->u.DeviceSpecificData.DataSize = sizeof(*KeyboardData);
//
//         /* Int 16h AH=02h
//          * KEYBOARD - GET SHIFT FLAGS
//          *
//          * Return:
//          * AL - shift flags
//          */
//         Regs.b.ah = 0x02;
//         Int386(0x16, &Regs, &Regs);
//
//         KeyboardData = (PCM_KEYBOARD_DEVICE_DATA)(PartialDescriptor + 1);
//         KeyboardData->Version = 1;
//         KeyboardData->Revision = 1;
//         KeyboardData->Type = 4;
//         KeyboardData->Subtype = 0;
//         KeyboardData->KeyboardFlags = Regs.b.al;
//
//         /* Create controller key */
//         FldrCreateComponentKey(ControllerKey,
//                                PeripheralClass,
//                                KeyboardPeripheral,
//                                Input | ConsoleIn,
//                                0,
//                                0xFFFFFFFF,
//                                "PCAT_ENHANCED",
//                                PartialResourceList,
//                                Size,
//                                &PeripheralKey);
//     }
// }
//
// static
// VOID
// DetectKeyboardController(PCONFIGURATION_COMPONENT_DATA BusKey)
// {
//     PCM_PARTIAL_RESOURCE_LIST PartialResourceList;
//     PCM_PARTIAL_RESOURCE_DESCRIPTOR PartialDescriptor;
//     PCONFIGURATION_COMPONENT_DATA ControllerKey;
//     ULONG Size;
//
//     /* Set 'Configuration Data' value */
//     Size = crate::FIELD_OFFSET!(CM_PARTIAL_RESOURCE_LIST, PartialDescriptors[3]);
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
//     PartialResourceList->Count = 3;
//
//     /* Set Interrupt */
//     PartialDescriptor = &PartialResourceList->PartialDescriptors[0];
//     PartialDescriptor->Type = CmResourceTypeInterrupt;
//     PartialDescriptor->ShareDisposition = CmResourceShareUndetermined;
//     PartialDescriptor->Flags = CM_RESOURCE_INTERRUPT_LATCHED;
//     PartialDescriptor->u.Interrupt.Level = 1;
//     PartialDescriptor->u.Interrupt.Vector = 1;
//     PartialDescriptor->u.Interrupt.Affinity = 0xFFFFFFFF;
//
//     /* Set IO Port 0x60 */
//     PartialDescriptor = &PartialResourceList->PartialDescriptors[1];
//     PartialDescriptor->Type = CmResourceTypePort;
//     PartialDescriptor->ShareDisposition = CmResourceShareDeviceExclusive;
//     PartialDescriptor->Flags = CM_RESOURCE_PORT_IO;
//     PartialDescriptor->u.Port.Start.LowPart = 0x60;
//     PartialDescriptor->u.Port.Start.HighPart = 0x0;
//     PartialDescriptor->u.Port.Length = 1;
//
//     /* Set IO Port 0x64 */
//     PartialDescriptor = &PartialResourceList->PartialDescriptors[2];
//     PartialDescriptor->Type = CmResourceTypePort;
//     PartialDescriptor->ShareDisposition = CmResourceShareDeviceExclusive;
//     PartialDescriptor->Flags = CM_RESOURCE_PORT_IO;
//     PartialDescriptor->u.Port.Start.LowPart = 0x64;
//     PartialDescriptor->u.Port.Start.HighPart = 0x0;
//     PartialDescriptor->u.Port.Length = 1;
//
//     /* Create controller key */
//     FldrCreateComponentKey(BusKey,
//                            ControllerClass,
//                            KeyboardController,
//                            Input | ConsoleIn,
//                            0,
//                            0xFFFFFFFF,
//                            NULL,
//                            PartialResourceList,
//                            Size,
//                            &ControllerKey);
//
//     DetectKeyboardPeripheral(ControllerKey);
// }
//
// static
// VOID
// PS2ControllerWait(VOID)
// {
//     ULONG Timeout;
//     UCHAR Status;
//
//     for (Timeout = 0; Timeout < CONTROLLER_TIMEOUT; Timeout++)
//     {
//         Status = READ_PORT_UCHAR((PUCHAR)CONTROLLER_REGISTER_STATUS);
//         if ((Status & CONTROLLER_STATUS_INPUT_BUFFER_FULL) == 0)
//             return;
//
//         /* Sleep for one millisecond */
//         StallExecutionProcessor(1000);
//     }
// }
//
// static
// BOOLEAN
// DetectPS2AuxPort(VOID)
// {
// #if 1
//     /* Current detection is too unreliable. Just do as if
//      * the PS/2 aux port is always present
//      */
//     return TRUE;
// #else
//     ULONG Loops;
//     UCHAR Status;
//
//     /* Put the value 0x5A in the output buffer using the
//      * "WriteAuxiliary Device Output Buffer" command (0xD3).
//      * Poll the Status Register for a while to see if the value really turns up
//      * in the Data Register. If the KEYBOARD_STATUS_MOUSE_OBF bit is also set
//      * to 1 in the Status Register, we assume this controller has an
//      *  Auxiliary Port (a.k.a. Mouse Port).
//      */
//     PS2ControllerWait();
//     WRITE_PORT_UCHAR((PUCHAR)CONTROLLER_REGISTER_CONTROL,
//                      CONTROLLER_COMMAND_WRITE_MOUSE_OUTPUT_BUFFER);
//     PS2ControllerWait();
//
//     /* 0x5A is a random dummy value */
//     WRITE_PORT_UCHAR((PUCHAR)CONTROLLER_REGISTER_DATA,
//                      0x5A);
//
//     for (Loops = 0; Loops < 10; Loops++)
//     {
//         StallExecutionProcessor(10000);
//         Status = READ_PORT_UCHAR((PUCHAR)CONTROLLER_REGISTER_STATUS);
//         if ((Status & CONTROLLER_STATUS_OUTPUT_BUFFER_FULL) != 0)
//             break;
//     }
//
//     READ_PORT_UCHAR((PUCHAR)CONTROLLER_REGISTER_DATA);
//
//     return (Status & CONTROLLER_STATUS_MOUSE_OUTPUT_BUFFER_FULL);
// #endif
// }
//
// static
// BOOLEAN
// DetectPS2AuxDevice(VOID)
// {
//     UCHAR Scancode;
//     UCHAR Status;
//     ULONG Loops;
//     BOOLEAN Result = TRUE;
//
//     PS2ControllerWait();
//     WRITE_PORT_UCHAR((PUCHAR)CONTROLLER_REGISTER_CONTROL,
//                      CONTROLLER_COMMAND_WRITE_MOUSE);
//     PS2ControllerWait();
//
//     /* Identify device */
//     WRITE_PORT_UCHAR((PUCHAR)CONTROLLER_REGISTER_DATA, 0xF2);
//
//     /* Wait for reply */
//     for (Loops = 0; Loops < 100; Loops++)
//     {
//         StallExecutionProcessor(10000);
//         Status = READ_PORT_UCHAR((PUCHAR)CONTROLLER_REGISTER_STATUS);
//         if ((Status & CONTROLLER_STATUS_OUTPUT_BUFFER_FULL) != 0)
//             break;
//     }
//
//     Status = READ_PORT_UCHAR((PUCHAR)CONTROLLER_REGISTER_STATUS);
//     if ((Status & CONTROLLER_STATUS_MOUSE_OUTPUT_BUFFER_FULL) == 0)
//         Result = FALSE;
//
//     Scancode = READ_PORT_UCHAR((PUCHAR)CONTROLLER_REGISTER_DATA);
//     if (Scancode != 0xFA)
//         Result = FALSE;
//
//     StallExecutionProcessor(10000);
//
//     Status = READ_PORT_UCHAR((PUCHAR)CONTROLLER_REGISTER_STATUS);
//     if ((Status & CONTROLLER_STATUS_MOUSE_OUTPUT_BUFFER_FULL) == 0)
//         Result = FALSE;
//
//     Scancode = READ_PORT_UCHAR((PUCHAR)CONTROLLER_REGISTER_DATA);
//     if (Scancode != 0x00)
//         Result = FALSE;
//
//     return Result;
// }
//
// // FIXME: Missing: DetectPS2Peripheral!! (for corresponding 'PointerPeripheral')
//
// static
// VOID
// DetectPS2Mouse(PCONFIGURATION_COMPONENT_DATA BusKey)
// {
//     PCM_PARTIAL_RESOURCE_LIST PartialResourceList;
//     PCONFIGURATION_COMPONENT_DATA ControllerKey;
//     PCONFIGURATION_COMPONENT_DATA PeripheralKey;
//     ULONG Size;
//
//     if (DetectPS2AuxPort())
//     {
//         TRACE("Detected PS2 port\n");
//
//         Size = crate::FIELD_OFFSET!(CM_PARTIAL_RESOURCE_LIST, PartialDescriptors[1]);
//         PartialResourceList = FrLdrHeapAlloc(Size, TAG_HW_RESOURCE_LIST);
//         if (PartialResourceList == NULL)
//         {
//             ERR("Failed to allocate resource descriptor\n");
//             return;
//         }
//
//         /* Initialize resource descriptor */
//         RtlZeroMemory(PartialResourceList, Size);
//         PartialResourceList->Version = 1;
//         PartialResourceList->Revision = 1;
//         PartialResourceList->Count = 1;
//
//         /* Set Interrupt */
//         PartialResourceList->PartialDescriptors[0].Type = CmResourceTypeInterrupt;
//         PartialResourceList->PartialDescriptors[0].ShareDisposition = CmResourceShareUndetermined;
//         PartialResourceList->PartialDescriptors[0].Flags = CM_RESOURCE_INTERRUPT_LATCHED;
//         PartialResourceList->PartialDescriptors[0].u.Interrupt.Level = 12;
//         PartialResourceList->PartialDescriptors[0].u.Interrupt.Vector = 12;
//         PartialResourceList->PartialDescriptors[0].u.Interrupt.Affinity = 0xFFFFFFFF;
//
//         /* Create controller key */
//         FldrCreateComponentKey(BusKey,
//                                ControllerClass,
//                                PointerController,
//                                Input,
//                                0,
//                                0xFFFFFFFF,
//                                NULL,
//                                PartialResourceList,
//                                Size,
//                                &ControllerKey);
//
//         if (DetectPS2AuxDevice())
//         {
//             TRACE("Detected PS2 mouse\n");
//
//             /* Initialize resource descriptor */
//             Size = crate::FIELD_OFFSET!(CM_PARTIAL_RESOURCE_LIST, PartialDescriptors);
//             PartialResourceList = FrLdrHeapAlloc(Size, TAG_HW_RESOURCE_LIST);
//             if (PartialResourceList == NULL)
//             {
//                 ERR("Failed to allocate resource descriptor\n");
//                 return;
//             }
//
//             RtlZeroMemory(PartialResourceList, Size);
//             PartialResourceList->Version = 1;
//             PartialResourceList->Revision = 1;
//             PartialResourceList->Count = 0;
//
//             /* Create peripheral key */
//             FldrCreateComponentKey(ControllerKey,
//                                    ControllerClass,
//                                    PointerPeripheral,
//                                    Input,
//                                    0,
//                                    0xFFFFFFFF,
//                                    "MICROSOFT PS2 MOUSE",
//                                    PartialResourceList,
//                                    Size,
//                                    &PeripheralKey);
//         }
//     }
// }
//
// #if defined(_M_IX86)
// static VOID
// CreateBusMousePeripheralKey(
//     _Inout_ PCONFIGURATION_COMPONENT_DATA BusKey,
//     _In_ ULONG IoBase,
//     _In_ ULONG Irq)
// {
//     PCM_PARTIAL_RESOURCE_LIST PartialResourceList;
//     PCM_PARTIAL_RESOURCE_DESCRIPTOR PartialDescriptor;
//     PCONFIGURATION_COMPONENT_DATA ControllerKey;
//     PCONFIGURATION_COMPONENT_DATA PeripheralKey;
//     ULONG Size;
//
//     /* Set 'Configuration Data' value */
//     Size = crate::FIELD_OFFSET!(CM_PARTIAL_RESOURCE_LIST, PartialDescriptors[2]);
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
//     PartialResourceList->Count = 2;
//
//     /* Set IO Port */
//     PartialDescriptor = &PartialResourceList->PartialDescriptors[0];
//     PartialDescriptor->Type = CmResourceTypePort;
//     PartialDescriptor->ShareDisposition = CmResourceShareDeviceExclusive;
//     PartialDescriptor->Flags = CM_RESOURCE_PORT_IO;
//     PartialDescriptor->u.Port.Start.LowPart = IoBase;
//     PartialDescriptor->u.Port.Start.HighPart = 0;
//     PartialDescriptor->u.Port.Length = 4;
//
//     /* Set Interrupt */
//     PartialDescriptor = &PartialResourceList->PartialDescriptors[1];
//     PartialDescriptor->Type = CmResourceTypeInterrupt;
//     PartialDescriptor->ShareDisposition = CmResourceShareUndetermined;
//     PartialDescriptor->Flags = CM_RESOURCE_INTERRUPT_LATCHED;
//     PartialDescriptor->u.Interrupt.Level = Irq;
//     PartialDescriptor->u.Interrupt.Vector = Irq;
//     PartialDescriptor->u.Interrupt.Affinity = (KAFFINITY)-1;
//
//     /* Create controller key */
//     FldrCreateComponentKey(BusKey,
//                            ControllerClass,
//                            PointerController,
//                            Input,
//                            0,
//                            0xFFFFFFFF,
//                            NULL,
//                            PartialResourceList,
//                            Size,
//                            &ControllerKey);
//
//     /* Set 'Configuration Data' value */
//     Size = crate::FIELD_OFFSET!(CM_PARTIAL_RESOURCE_LIST, PartialDescriptors);
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
//     PartialResourceList->Count = 0;
//
//     /* Create peripheral key */
//     FldrCreateComponentKey(ControllerKey,
//                            ControllerClass,
//                            PointerPeripheral,
//                            Input,
//                            0,
//                            0xFFFFFFFF,
//                            "MICROSOFT INPORT MOUSE",
//                            PartialResourceList,
//                            Size,
//                            &PeripheralKey);
// }
//
// extern KIDTENTRY DECLSPEC_ALIGN(4) i386Idt[32];
// VOID __cdecl HwIrqHandler(VOID);
// extern volatile ULONG HwIrqCount;
//
// static ULONG
// DetectBusMouseTestIrq(
//     _In_ ULONG IoBase,
//     _In_ ULONG Irq)
// {
//     USHORT OldOffset, OldExtendedOffset;
//     ULONG Vector, i;
//
//     HwIrqCount = 0;
//
//     /* Reset the device */
//     WRITE_PORT_UCHAR((PUCHAR)IoBase + INPORT_REGISTER_CONTROL, INPORT_RESET);
//     WRITE_PORT_UCHAR((PUCHAR)IoBase + INPORT_REGISTER_CONTROL, INPORT_REG_MODE);
//
//     Vector = Irq + 8;
//
//     /* Save the old interrupt vector and replace it by ours */
//     OldOffset = i386Idt[Vector].Offset;
//     OldExtendedOffset = i386Idt[Vector].ExtendedOffset;
//
//     i386Idt[Vector].Offset = (ULONG)HwIrqHandler & 0xFFFF;
//     i386Idt[Vector].ExtendedOffset = (ULONG)HwIrqHandler >> 16;
//
//     /* Enable the requested IRQ on the master PIC */
//     WRITE_PORT_UCHAR((PUCHAR)PIC1_DATA_PORT, ~(1 << Irq));
//
//     _enable();
//
//     /* Configure the device to generate interrupts */
//     for (i = 0; i < 15; i++)
//     {
//         WRITE_PORT_UCHAR((PUCHAR)IoBase + INPORT_REGISTER_DATA, INPORT_MODE_BASE);
//         WRITE_PORT_UCHAR((PUCHAR)IoBase + INPORT_REGISTER_DATA, INPORT_TEST_IRQ);
//     }
//
//     /* Disable the device */
//     WRITE_PORT_UCHAR((PUCHAR)IoBase + INPORT_REGISTER_DATA, 0);
//
//     _disable();
//
//     i386Idt[Vector].Offset = OldOffset;
//     i386Idt[Vector].ExtendedOffset = OldExtendedOffset;
//
//     return (HwIrqCount != 0) ? Irq : 0;
// }
//
// static ULONG
// DetectBusMouseIrq(
//     _In_ ULONG IoBase)
// {
//     UCHAR Mask1, Mask2;
//     ULONG Irq, Result;
//
//     /* Save the current interrupt mask */
//     Mask1 = READ_PORT_UCHAR(PIC1_DATA_PORT);
//     Mask2 = READ_PORT_UCHAR(PIC2_DATA_PORT);
//
//     /* Mask the interrupts on the slave PIC */
//     WRITE_PORT_UCHAR(PIC2_DATA_PORT, 0xFF);
//
//     /* Process IRQ detection: IRQ 5, 4, 3 */
//     for (Irq = 5; Irq >= 3; Irq--)
//     {
//         Result = DetectBusMouseTestIrq(IoBase, Irq);
//         if (Result != 0)
//             break;
//     }
//
//     /* Restore the mask */
//     WRITE_PORT_UCHAR(PIC1_DATA_PORT, Mask1);
//     WRITE_PORT_UCHAR(PIC2_DATA_PORT, Mask2);
//
//     return Result;
// }
//
// static VOID
// DetectBusMouse(
//     _Inout_ PCONFIGURATION_COMPONENT_DATA BusKey)
// {
//     ULONG IoBase, Irq, Signature1, Signature2, Signature3;
//
//     /*
//      * The bus mouse lives at one of these addresses: 0x230, 0x234, 0x238, 0x23C.
//      * The 0x23C port is the most common I/O setting.
//      */
//     for (IoBase = 0x23C; IoBase >= 0x230; IoBase -= 4)
//     {
//         Signature1 = READ_PORT_UCHAR((PUCHAR)IoBase + INPORT_REGISTER_SIGNATURE);
//         Signature2 = READ_PORT_UCHAR((PUCHAR)IoBase + INPORT_REGISTER_SIGNATURE);
//         if (Signature1 == Signature2)
//             continue;
//         if (Signature1 != INPORT_SIGNATURE && Signature2 != INPORT_SIGNATURE)
//             continue;
//
//         Signature3 = READ_PORT_UCHAR((PUCHAR)IoBase + INPORT_REGISTER_SIGNATURE);
//         if (Signature1 != Signature3)
//             continue;
//
//         Irq = DetectBusMouseIrq(IoBase);
//         if (Irq == 0)
//             continue;
//
//         CreateBusMousePeripheralKey(BusKey, IoBase, Irq);
//         break;
//     }
// }
// #endif /* _M_IX86 */
//
// // Implemented in pcvesa.c, returns the VESA version
// USHORT  BiosIsVesaSupported(VOID);
// BOOLEAN BiosIsVesaDdcSupported(VOID);
// BOOLEAN BiosVesaReadEdid(VOID);
//
// static VOID
// DetectDisplayController(PCONFIGURATION_COMPONENT_DATA BusKey)
// {
//     PCSTR Identifier;
//     PCONFIGURATION_COMPONENT_DATA ControllerKey;
//     USHORT VesaVersion;
//
//     /* FIXME: Set 'ComponentInformation' value */
//
//     VesaVersion = BiosIsVesaSupported();
//     if (VesaVersion != 0)
//     {
//         TRACE("VESA version %c.%c\n", (//               (VesaVersion >> 8) + '0') as usize, (//               (VesaVersion & 0xFF) + '0') as usize);
//     }
//     else
//     {
//         TRACE("VESA not supported\n");
//     }
//
//     if (VesaVersion >= 0x0200)
//         Identifier = "VBE Display";
//     else
//         Identifier = "VGA Display";
//
//     FldrCreateComponentKey(BusKey,
//                            ControllerClass,
//                            DisplayController,
//                            Output | ConsoleOut,
//                            0,
//                            0xFFFFFFFF,
//                            Identifier,
//                            NULL,
//                            0,
//                            &ControllerKey);
//
//     /* FIXME: Add display peripheral (monitor) data */
//     if (VesaVersion != 0)
//     {
//         if (BiosIsVesaDdcSupported())
//         {
//             TRACE("VESA/DDC supported!\n");
//             if (BiosVesaReadEdid())
//             {
//                 TRACE("EDID data read successfully!\n");
//             }
//         }
//     }
// }
//
// static
// VOID
// DetectIsaBios(
//     _In_opt_ PCSTR Options,
//     _Inout_ PCONFIGURATION_COMPONENT_DATA SystemKey,
//     _Out_ ULONG *BusNumber)
// {
//     PCM_PARTIAL_RESOURCE_LIST PartialResourceList;
//     PCONFIGURATION_COMPONENT_DATA BusKey;
//     ULONG Size;
//
//     /* Set 'Configuration Data' value */
//     Size = crate::FIELD_OFFSET!(CM_PARTIAL_RESOURCE_LIST, PartialDescriptors);
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
//     PartialResourceList->Count = 0;
//
//     /* Create new bus key */
//     FldrCreateComponentKey(SystemKey,
//                            AdapterClass,
//                            MultiFunctionAdapter,
//                            0,
//                            0,
//                            0xFFFFFFFF,
//                            "ISA",
//                            PartialResourceList,
//                            Size,
//                            &BusKey);
//
//     /* Increment bus number */
//     (*BusNumber)++;
//
//     /* Detect ISA/BIOS devices */
//     DetectBiosDisks(SystemKey, BusKey);
//     DetectSerialPorts(Options, BusKey, PcGetSerialPort, MAX_COM_PORTS);
//     DetectParallelPorts(BusKey);
//     DetectKeyboardController(BusKey);
//     DetectPS2Mouse(BusKey);
// #if defined(_M_IX86)
//     DetectBusMouse(BusKey);
// #endif
//     DetectDisplayController(BusKey);
//
//     /* FIXME: Detect more ISA devices */
// }
//
// static
// UCHAR
// PcGetFloppyCount(VOID)
// {
//     UCHAR Data;
//
//     WRITE_PORT_UCHAR((PUCHAR)0x70, 0x10);
//     Data = READ_PORT_UCHAR((PUCHAR)0x71);
//
//     return ((Data & 0xF0) ? 1 : 0) + ((Data & 0x0F) ? 1 : 0);
// }
//
// PCONFIGURATION_COMPONENT_DATA
// PcHwDetect(
//     _In_opt_ PCSTR Options)
// {
//     PCONFIGURATION_COMPONENT_DATA SystemKey;
//     ULONG BusNumber = 0;
//
//     TRACE("DetectHardware()\n");
//
//     /* Create the 'System' key */
//     // TODO: Discover and set the other machine types
//     FldrCreateSystemKey(&SystemKey, "AT/AT COMPATIBLE");
//
//     GetHarddiskConfigurationData = PcGetHarddiskConfigurationData;
//     FindPciBios = PcFindPciBios;
//
//     /* Detect buses */
//     DetectPciBios(SystemKey, &BusNumber);
//     DetectApmBios(SystemKey, &BusNumber);
//     DetectPnpBios(SystemKey, &BusNumber);
//     DetectIsaBios(Options, SystemKey, &BusNumber); // TODO: Detect first EISA or MCA, before ISA
//     DetectAcpiBios(SystemKey, &BusNumber);
//
//     // TODO: Collect the ROM blocks from 0xC0000 to 0xF0000 and append their
//     // CM_ROM_BLOCK data into the 'System' key's configuration data.
//
//     TRACE("DetectHardware() Done\n");
//     return SystemKey;
// }
//
// VOID
// PcHwIdle(VOID)
// {
//     REGS Regs;
//
//     /* Select APM 1.0+ function */
//     Regs.b.ah = 0x53;
//
//     /* Function 05h: CPU idle */
//     Regs.b.al = 0x05;
//
//     /* Call INT 15h */
//     Int386(0x15, &Regs, &Regs);
//
//     /* Check if successfull (CF set on error) */
//     if (INT386_SUCCESS(Regs))
//         return;
//
//     /*
//      * No futher processing here.
//      * Optionally implement HLT instruction handling.
//      */
// }
//
// VOID __cdecl ChainLoadBiosBootSectorCode(
//     IN UCHAR BootDrive OPTIONAL,
//     IN ULONG BootPartition OPTIONAL)
// {
//     REGS Regs;
//
//     RtlZeroMemory(&Regs, sizeof(Regs));
//
//     /* Set the boot drive and the boot partition */
//     Regs.b.dl = (UCHAR)(BootDrive ? BootDrive : FrldrBootDrive);
//     Regs.b.dh = (UCHAR)(BootPartition ? BootPartition : FrldrBootPartition);
//
//     /*
//      * Don't stop the floppy drive motor when we are just booting a bootsector,
//      * a drive, or a partition. If we were to stop the floppy motor, the BIOS
//      * wouldn't be informed and if the next read is to a floppy then the BIOS
//      * will still think the motor is on and this will result in a read error.
//      */
//     // DiskStopFloppyMotor();
//
//     Relocator16Boot(&Regs,
//                     /* Stack segment:pointer */
//                     0x0000, 0x7C00,
//                     /* Code segment:pointer */
//                     0x0000, 0x7C00);
// }
//
//
//
// /******************************************************************************/
//
// /* FIXME: Abstract things better so we don't need to place define here */
// VOID
// MachInit(const char *CmdLine)
// {
//     /* Setup vtbl */
//     RtlZeroMemory(&MachVtbl, sizeof(MachVtbl));
//     MachVtbl.ConsPutChar = PcConsPutChar;
//     MachVtbl.ConsKbHit = PcConsKbHit;
//     MachVtbl.ConsGetCh = PcConsGetCh;
//     MachVtbl.VideoClearScreen = PcVideoClearScreen;
//     MachVtbl.VideoSetDisplayMode = PcVideoSetDisplayMode;
//     MachVtbl.VideoGetDisplaySize = PcVideoGetDisplaySize;
//     MachVtbl.VideoGetBufferSize = PcVideoGetBufferSize;
//     MachVtbl.VideoGetFontsFromFirmware = PcVideoGetFontsFromFirmware;
//     MachVtbl.VideoSetTextCursorPosition = PcVideoSetTextCursorPosition;
//     MachVtbl.VideoHideShowTextCursor = PcVideoHideShowTextCursor;
//     MachVtbl.VideoPutChar = PcVideoPutChar;
//     MachVtbl.VideoCopyOffScreenBufferToVRAM = PcVideoCopyOffScreenBufferToVRAM;
//     MachVtbl.VideoIsPaletteFixed = PcVideoIsPaletteFixed;
//     MachVtbl.VideoSetPaletteColor = PcVideoSetPaletteColor;
//     MachVtbl.VideoGetPaletteColor = PcVideoGetPaletteColor;
//     MachVtbl.VideoSync = PcVideoSync;
//     MachVtbl.Beep = PcBeep;
//     MachVtbl.PrepareForReactOS = PcPrepareForReactOS;
//     MachVtbl.GetMemoryMap = PcMemGetMemoryMap;
//     MachVtbl.GetExtendedBIOSData = PcGetExtendedBIOSData;
//     MachVtbl.GetFloppyCount = PcGetFloppyCount;
//     MachVtbl.DiskReadLogicalSectors = PcDiskReadLogicalSectors;
//     MachVtbl.DiskGetDriveGeometry = PcDiskGetDriveGeometry;
//     MachVtbl.DiskGetCacheableBlockCount = PcDiskGetCacheableBlockCount;
//     MachVtbl.GetTime = PcGetTime;
//     MachVtbl.InitializeBootDevices = PcInitializeBootDevices;
//     MachVtbl.HwDetect = PcHwDetect;
//     MachVtbl.HwIdle = PcHwIdle;
//
//     HalpCalibrateStallExecution();
//     PcVideoInit();
// }
//
// VOID
// PcPrepareForReactOS(VOID)
// {
//     /* Prepare video and turn off the floppy motor */
//     PcVideoPrepareForReactOS();
//     DiskStopFloppyMotor();
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
