// i8042prt.h — private-header module for the i8042prt.sys Rust translation.
//
// Hand-translated (FREE — no paid API) from drivers/input/i8042prt/i8042prt.h.
// Mirrors the driver-private types (PORT_DEVICE_EXTENSION, FDO_DEVICE_EXTENSION,
// I8042_KEYBOARD_EXTENSION, I8042_MOUSE_EXTENSION, OUTPUT_PACKET, I8042_SETTINGS,
// the enums, the P* pointer typedefs, I8042PRT_TAG, the bit/command #defines)
// that are NOT in ntkrnl-sys. Public DDK member types are re-exported from
// ntkrnl_sys::ddk (bindgen). Pattern copied from arm64input-rust/src/<input_h>.
//
// NEVER use `ddk::*` here (E0659). Selective named imports only.

#![allow(non_camel_case_types, non_upper_case_globals, dead_code)]

use ntkrnl_sys::base::NTSTATUS;
use core::ffi::c_void;

// ---- Public DDK types re-exported for the i8042-private structs/aliases ----
// bindgen names: structs are tag-prefixed with underscore (_FOO); the typedef
// alias (FOO) and pointer alias (PFOO) exist too. All resolve in bindgen `ddk`.
// NOTE: KIRQL is intentionally NOT re-exported here. The hub modules pull it in
// via their own `use ntkrnl_sys::{ke,base}::*` globs; re-exporting it through
// this glob too created an E0659 ambiguity. Header structs below fully-qualify
// the type as `ntkrnl_sys::ddk::KIRQL`.
pub(crate) use ntkrnl_sys::ddk::{
    // primitives / handles
    KSPIN_LOCK,
    KAFFINITY,
    KDPC,
    KTIMER,
    LIST_ENTRY,
    ULARGE_INTEGER,
    UNICODE_STRING,
    KINTERRUPT_MODE,
    PKINTERRUPT,
    PDEVICE_OBJECT,
    PIRP,
    PIO_WORKITEM,
    // kbd/mouse class shared structs — these ARE in bindgen ddk (ntddkbd.h /
    // ntddmou.h pulled transitively). Re-export the bindgen versions so the
    // layout matches the live C i8042prt.sys + cohabiting kbdclass/mouclass
    // exactly. Do NOT hand-roll these — keeps cross-driver ABI in lockstep.
    KEYBOARD_ATTRIBUTES,
    MOUSE_ATTRIBUTES,
    PKEYBOARD_INPUT_DATA,
    PMOUSE_INPUT_DATA,
    KEYBOARD_INDICATOR_PARAMETERS,
};
// CONNECT_DATA (kbdmou.h) is provided by the hand-written driver_ddk module,
// not bindgen — shared with kbdclass/mouclass.
pub(crate) use ntkrnl_sys::driver_ddk::CONNECT_DATA;

// ---------------------------------------------------------------------------
// ntdd8042.h filter-hook structs — INTERNAL_I8042_HOOK_KEYBOARD / _MOUSE.
//
// ntdd8042.h is NOT pulled by ntoskrnl.h, so bindgen does not emit these; they
// are hand-mirrored field-for-field from sdk/include/psdk/ntdd8042.h. All
// members are pointer-width (PVOID + NTAPI function pointers), so a #[repr(C)]
// of raw pointers is layout-exact on every arch.
// ---------------------------------------------------------------------------

// Hook filter-routine function-pointer typedefs (ntdd8042.h). NTAPI = "system".
// Layout: each Option<fn> is exactly pointer-sized, so the repr(C) struct stays
// byte-for-byte identical to the C *mut-PVOID layout on every arch.
pub type PI8042_KEYBOARD_INITIALIZATION_ROUTINE = Option<
    unsafe extern "system" fn(
        InitializationContext: *mut c_void,
        SynchFuncContext: *mut c_void,
        ReadPort: *mut c_void,         // PI8042_SYNCH_READ_PORT
        WritePort: *mut c_void,        // PI8042_SYNCH_WRITE_PORT
        TurnTranslationOn: *mut u8,    // PBOOLEAN
    ) -> NTSTATUS,
>;
pub type PI8042_KEYBOARD_ISR = Option<
    unsafe extern "system" fn(
        Context: *mut c_void,
        CurrentInput: PKEYBOARD_INPUT_DATA,
        CurrentOutput: POUTPUT_PACKET,
        StatusByte: u8,
        Byte: *mut u8,
        ContinueProcessing: *mut u8,
        ScanState: PKEYBOARD_SCAN_STATE,
    ) -> u8,
>;
pub type PI8042_MOUSE_ISR = Option<
    unsafe extern "system" fn(
        Context: *mut c_void,
        CurrentInput: PMOUSE_INPUT_DATA,
        CurrentOutput: POUTPUT_PACKET,
        StatusByte: u8,
        Byte: *mut u8,
        ContinueProcessing: *mut u8,
        MouseState: PMOUSE_STATE,
        ResetSubState: PMOUSE_RESET_SUBSTATE,
    ) -> u8,
>;
pub type PI8042_ISR_WRITE_PORT =
    Option<unsafe extern "system" fn(Context: *mut c_void, Value: u8)>;
pub type PI8042_QUEUE_PACKET = Option<unsafe extern "system" fn(Context: *mut c_void)>;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct INTERNAL_I8042_HOOK_KEYBOARD {
    pub Context: *mut c_void,
    pub InitializationRoutine: PI8042_KEYBOARD_INITIALIZATION_ROUTINE,
    pub IsrRoutine: PI8042_KEYBOARD_ISR,
    pub IsrWritePort: PI8042_ISR_WRITE_PORT,
    pub QueueKeyboardPacket: PI8042_QUEUE_PACKET,
    pub CallContext: *mut c_void,
}
pub type PINTERNAL_I8042_HOOK_KEYBOARD = *mut INTERNAL_I8042_HOOK_KEYBOARD;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct INTERNAL_I8042_HOOK_MOUSE {
    pub Context: *mut c_void,
    pub IsrRoutine: PI8042_MOUSE_ISR,
    pub IsrWritePort: PI8042_ISR_WRITE_PORT,
    pub QueueMousePacket: PI8042_QUEUE_PACKET,
    pub CallContext: *mut c_void,
}
pub type PINTERNAL_I8042_HOOK_MOUSE = *mut INTERNAL_I8042_HOOK_MOUSE;

// ---- I8042PRT_TAG ----
// C: #define I8042PRT_TAG '2408'  (4-char literal is big-endian: 0x32343038).
// Emitted via Io/Ex pool-tag APIs that take it as ULONG. The byte order on the
// wire (pool-tag dump) reads "8042" → from_le_bytes(b"8042").
pub const I8042PRT_TAG: u32 = u32::from_le_bytes(*b"8042");

// ---- WHEEL_DELTA / power scancodes ----
pub const WHEEL_DELTA: i32 = 120;
// Power-button scancodes. u32 so they match the ULONG scancode the keyboard
// IRP-completion match scrutinee uses.
pub const KEYBOARD_POWER_CODE: u32 = 0x5E;
pub const KEYBOARD_SLEEP_CODE: u32 = 0x5F;
pub const KEYBOARD_WAKE_CODE: u32 = 0x63;

// ---- PORT_DEVICE_EXTENSION.Flags ----
pub const KEYBOARD_PRESENT: u32 = 0x01;
pub const KEYBOARD_CONNECTED: u32 = 0x02;
pub const KEYBOARD_STARTED: u32 = 0x04;
pub const KEYBOARD_INITIALIZED: u32 = 0x08;
pub const MOUSE_PRESENT: u32 = 0x10;
pub const MOUSE_CONNECTED: u32 = 0x20;
pub const MOUSE_STARTED: u32 = 0x40;
pub const MOUSE_INITIALIZED: u32 = 0x80;

// ---- Controller commands ----
pub const KBD_READ_MODE: u8 = 0x20;
pub const KBD_WRITE_MODE: u8 = 0x60;
pub const MOUSE_ENAB: u8 = 0xA8;
pub const MOUSE_LINE_TEST: u8 = 0xA9;
pub const CTRL_SELF_TEST: u8 = 0xAA;
pub const KBD_CLK_DISABLE: u8 = 0xAD;
pub const KBD_CLK_ENABLE: u8 = 0xAE;
pub const CTRL_WRITE_MOUSE: u8 = 0xD4;

// ---- Keyboard commands ----
pub const KBD_CMD_SET_LEDS: u8 = 0xED;
pub const KBD_CMD_GET_ID: u8 = 0xF2;

// ---- Keyboard responses ----
pub const KBD_SELF_TEST_OK: u8 = 0x55;
pub const KBD_ACK: u8 = 0xFA;
pub const KBD_NACK: u8 = 0xFC;
pub const KBD_RESEND: u8 = 0xFE;

// ---- Controller status register bits ----
pub const KBD_OBF: u8 = 0x01;
pub const KBD_IBF: u8 = 0x02;
pub const MOU_OBF: u8 = 0x20;
pub const KBD_PERR: u8 = 0x80;

// ---- Controller command byte bits ----
pub const CCB_KBD_INT_ENAB: u8 = 0x01;
pub const CCB_MOUSE_INT_ENAB: u8 = 0x02;
pub const CCB_SYSTEM_FLAG: u8 = 0x04;
pub const CCB_KBD_DISAB: u8 = 0x10;
pub const CCB_MOUSE_DISAB: u8 = 0x20;
pub const CCB_TRANSLATE: u8 = 0x40;

// ---- LED bits ----
pub const KBD_LED_SCROLL: u8 = 0x01;
pub const KBD_LED_NUM: u8 = 0x02;
pub const KBD_LED_CAPS: u8 = 0x04;

// ---- Mouse commands ----
pub const MOU_ENAB: u8 = 0xF4;
pub const MOU_CMD_RESET: u8 = 0xFF;

// ---- Mouse responses ----
pub const MOUSE_ACK: u8 = 0xFA;
pub const MOUSE_ERROR: u8 = 0xFC;
pub const MOUSE_NACK: u8 = 0xFE;

// ---- hwhacks.c FLAGS ----
pub const FL_NOLOOP: u32 = 0x01;
pub const FL_INITHACK: u32 = 0x02;

// =====================================================================
// Enums (repr C — 0-based unless an explicit discriminant is given)
// =====================================================================

#[repr(C)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum DEVICE_STATE {
    dsStopped,
    dsStarted,
    dsPaused,
    dsRemoved,
    dsSurpriseRemoved,
}
pub use DEVICE_STATE::*;

#[repr(C)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum MOUSE_TIMEOUT_STATE {
    NoChange,
    TimeoutStart,
    TimeoutCancel,
}
pub use MOUSE_TIMEOUT_STATE::*;
pub type PMOUSE_TIMEOUT_STATE = *mut MOUSE_TIMEOUT_STATE;

#[repr(C)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum I8042_DEVICE_TYPE {
    Unknown,
    Keyboard,
    Mouse,
    PhysicalDeviceObject,
}
pub use I8042_DEVICE_TYPE::*;
pub type PI8042_DEVICE_TYPE = *mut I8042_DEVICE_TYPE;

#[repr(C)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum I8042_MOUSE_TYPE {
    GenericPS2,
    Intellimouse,
    IntellimouseExplorer,
    Ps2pp,
}
pub use I8042_MOUSE_TYPE::*;
pub type PI8042_MOUSE_TYPE = *mut I8042_MOUSE_TYPE;

// MOUSE_STATE — 6 variants, 0-based. (ground truth; ntdd8042.h)
#[repr(C)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum MOUSE_STATE {
    MouseIdle,
    XMovement,
    YMovement,
    ZMovement,
    MouseExpectingACK,
    MouseResetting,
}
pub use MOUSE_STATE::*;
pub type PMOUSE_STATE = *mut MOUSE_STATE;

// MOUSE_RESET_SUBSTATE — 36 sequential (0..=35) then 3 sentinels. (ground truth)
#[repr(C)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum MOUSE_RESET_SUBSTATE {
    ExpectingReset,
    ExpectingResetId,
    ExpectingGetDeviceIdACK,
    ExpectingGetDeviceIdValue,
    ExpectingSetResolutionDefaultACK,
    ExpectingSetResolutionDefaultValueACK,
    ExpectingSetResolutionACK,
    ExpectingSetResolutionValueACK,
    ExpectingSetScaling1to1ACK,
    ExpectingSetScaling1to1ACK2,
    ExpectingSetScaling1to1ACK3,
    ExpectingReadMouseStatusACK,
    ExpectingReadMouseStatusByte1,
    ExpectingReadMouseStatusByte2,
    ExpectingReadMouseStatusByte3,
    StartPnPIdDetection,
    ExpectingLoopSetSamplingRateACK,
    ExpectingLoopSetSamplingRateValueACK,
    ExpectingPnpIdByte1,
    ExpectingPnpIdByte2,
    ExpectingPnpIdByte3,
    ExpectingPnpIdByte4,
    ExpectingPnpIdByte5,
    ExpectingPnpIdByte6,
    ExpectingPnpIdByte7,
    EnableWheel,
    Enable5Buttons,
    ExpectingGetDeviceId2ACK,
    ExpectingGetDeviceId2Value,
    ExpectingSetSamplingRateACK,
    ExpectingSetSamplingRateValueACK,
    ExpectingEnableACK,
    ExpectingFinalResolutionACK,
    ExpectingFinalResolutionValueACK,
    ExpectingGetDeviceIdDetectACK,
    ExpectingGetDeviceIdDetectValue,
    CustomHookStateMinimum = 100,
    CustomHookStateMaximum = 999,
    I8042ReservedMinimum = 1000,
}
pub use MOUSE_RESET_SUBSTATE::*;
pub type PMOUSE_RESET_SUBSTATE = *mut MOUSE_RESET_SUBSTATE;

// TRANSMIT_STATE — Idle=0, SendingBytes. (ntdd8042.h)
#[repr(C)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TRANSMIT_STATE {
    Idle = 0,
    SendingBytes,
}
pub use TRANSMIT_STATE::*;
pub type PTRANSMIT_STATE = *mut TRANSMIT_STATE;

// KEYBOARD_SCAN_STATE — keyboard.c scancode state machine. (i8042prt internal)
#[repr(C)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum KEYBOARD_SCAN_STATE {
    Normal,
    GotE0,
    GotE1,
}
pub use KEYBOARD_SCAN_STATE::*;
pub type PKEYBOARD_SCAN_STATE = *mut KEYBOARD_SCAN_STATE;

// =====================================================================
// OUTPUT_PACKET — ntdd8042.h. Drives the i8042 write state machine.
//   typedef struct _OUTPUT_PACKET {
//       PUCHAR          Bytes;
//       ULONG           CurrentByte;
//       ULONG           ByteCount;
//       TRANSMIT_STATE  State;
//   } OUTPUT_PACKET, *POUTPUT_PACKET;
// =====================================================================
#[repr(C)]
#[derive(Clone, Copy)]
pub struct OUTPUT_PACKET {
    pub Bytes: *mut u8,
    pub CurrentByte: u32,
    pub ByteCount: u32,
    pub State: TRANSMIT_STATE,
}
pub type POUTPUT_PACKET = *mut OUTPUT_PACKET;

// =====================================================================
// I8042_SETTINGS — registry-driven tunables.
// =====================================================================
#[repr(C)]
#[derive(Clone, Copy)]
pub struct I8042_SETTINGS {
    pub KeyboardDataQueueSize: u32,
    pub KeyboardDeviceBaseName: UNICODE_STRING,
    pub MouseDataQueueSize: u32,
    pub MouseResolution: u32,
    pub MouseSynchIn100ns: u32,
    pub NumberOfButtons: u32,
    pub PointerDeviceBaseName: UNICODE_STRING,
    pub PollStatusIterations: u32,
    pub OverrideKeyboardType: u32,
    pub OverrideKeyboardSubtype: u32,
    pub PollingIterations: u32,
    pub PollingIterationsMaximum: u32,
    pub ResendIterations: u32,
    pub SampleRate: u32,
    pub CrashOnCtrlScroll: u32,
}
pub type PI8042_SETTINGS = *mut I8042_SETTINGS;

// =====================================================================
// INTERRUPT_DATA — per-device interrupt parameters.
// =====================================================================
#[repr(C)]
#[derive(Clone, Copy)]
pub struct INTERRUPT_DATA {
    pub Object: PKINTERRUPT,
    pub Vector: u32,
    pub Dirql: ntkrnl_sys::ddk::KIRQL,
    pub InterruptMode: KINTERRUPT_MODE,
    pub ShareInterrupt: u8,
    pub Affinity: KAFFINITY,
}
pub type PINTERRUPT_DATA = *mut INTERRUPT_DATA;

// Forward pointer typedefs (C forward-declares the extension structs).
pub type PI8042_KEYBOARD_EXTENSION = *mut I8042_KEYBOARD_EXTENSION;
pub type PI8042_MOUSE_EXTENSION = *mut I8042_MOUSE_EXTENSION;

// =====================================================================
// PORT_DEVICE_EXTENSION — the shared controller state.
// =====================================================================
#[repr(C)]
pub struct PORT_DEVICE_EXTENSION {
    pub DataPort: *mut u8,    // 0x60
    pub ControlPort: *mut u8, // 0x64
    pub Settings: I8042_SETTINGS,
    pub Flags: u32,

    pub KeyboardExtension: PI8042_KEYBOARD_EXTENSION,
    pub KeyboardInterrupt: INTERRUPT_DATA,
    pub MouseExtension: PI8042_MOUSE_EXTENSION,
    pub MouseInterrupt: INTERRUPT_DATA,
    pub HighestDIRQLInterrupt: PKINTERRUPT,
    pub SpinLock: KSPIN_LOCK,
    pub HighestDirql: ntkrnl_sys::ddk::KIRQL,

    pub Packet: OUTPUT_PACKET,
    pub PacketResends: u32,
    pub PacketComplete: u8,
    pub PacketResult: NTSTATUS,
    pub PacketBuffer: [u8; 16],
    pub PacketPort: u8,

    pub CurrentIrp: PIRP,
    pub CurrentIrpDevice: PDEVICE_OBJECT,
}
pub type PPORT_DEVICE_EXTENSION = *mut PORT_DEVICE_EXTENSION;

// =====================================================================
// I8042_DRIVER_EXTENSION — driver-object extension (one per driver).
// =====================================================================
#[repr(C)]
pub struct I8042_DRIVER_EXTENSION {
    pub RegistryPath: UNICODE_STRING,
    pub Port: PORT_DEVICE_EXTENSION,
    pub DeviceListHead: LIST_ENTRY,
    pub DeviceListLock: KSPIN_LOCK,
}
pub type PI8042_DRIVER_EXTENSION = *mut I8042_DRIVER_EXTENSION;

// =====================================================================
// FDO_DEVICE_EXTENSION — common header for every FDO this driver owns.
// =====================================================================
#[repr(C)]
pub struct FDO_DEVICE_EXTENSION {
    pub Type: I8042_DEVICE_TYPE,
    pub ListEntry: LIST_ENTRY,
    pub Fdo: PDEVICE_OBJECT,
    pub Pdo: PDEVICE_OBJECT,
    pub LowerDevice: PDEVICE_OBJECT,
    pub PnpState: DEVICE_STATE,
    pub PortDeviceExtension: PPORT_DEVICE_EXTENSION,
}
pub type PFDO_DEVICE_EXTENSION = *mut FDO_DEVICE_EXTENSION;

// =====================================================================
// I8042_KEYBOARD_EXTENSION
// =====================================================================
#[repr(C)]
pub struct I8042_KEYBOARD_EXTENSION {
    pub Common: FDO_DEVICE_EXTENSION,
    pub KeyboardData: CONNECT_DATA,
    pub KeyboardHook: INTERNAL_I8042_HOOK_KEYBOARD,
    pub DpcKeyboard: KDPC,

    pub KeyboardAttributes: KEYBOARD_ATTRIBUTES,

    pub KeyboardIndicators: KEYBOARD_INDICATOR_PARAMETERS,

    pub KeyboardScanState: KEYBOARD_SCAN_STATE,
    pub KeyComplete: u8,
    pub KeyboardBuffer: PKEYBOARD_INPUT_DATA,
    pub KeysInBuffer: u32,

    // Power keys items
    pub ReportedCaps: u32,
    pub NewCaps: u32,
    pub LastPowerKey: u32,
    pub PowerInterfaceName: UNICODE_STRING,
    pub PowerWorkItem: PIO_WORKITEM,
    pub PowerIrp: PIRP,

    // Debug items
    pub ComboPosition: u32,
    pub DebugWorkItem: PIO_WORKITEM,
    pub TabPressed: u8,
}

// =====================================================================
// I8042_MOUSE_EXTENSION
// =====================================================================
#[repr(C)]
pub struct I8042_MOUSE_EXTENSION {
    pub Common: FDO_DEVICE_EXTENSION,
    pub MouseData: CONNECT_DATA,
    pub MouseHook: INTERNAL_I8042_HOOK_MOUSE,
    pub DpcMouse: KDPC,

    pub MouseAttributes: MOUSE_ATTRIBUTES,

    pub MouseState: MOUSE_STATE,
    pub MouseComplete: u8,
    pub MouseResetState: MOUSE_RESET_SUBSTATE,
    pub MouseBuffer: PMOUSE_INPUT_DATA,
    pub MouseInBuffer: u32,
    pub MouseButtonState: u16,
    pub MousePacketStartTime: ULARGE_INTEGER,

    pub TimerMouseTimeout: KTIMER,
    pub DpcMouseTimeout: KDPC,
    pub MouseTimeoutState: MOUSE_TIMEOUT_STATE,
    pub MouseTimeoutActive: u8,

    pub MouseLogiBuffer: [u8; 3],
    pub MouseType: I8042_MOUSE_TYPE,
}

// =====================================================================
// I8042_HOOK_WORKITEM
// =====================================================================
#[repr(C)]
#[derive(Clone, Copy)]
pub struct I8042_HOOK_WORKITEM {
    pub WorkItem: PIO_WORKITEM,
    pub Irp: PIRP,
}
pub type PI8042_HOOK_WORKITEM = *mut I8042_HOOK_WORKITEM;

// =====================================================================
// HUB-INTEGRATION ADDITIONS (FREE hand-fix). Constants/types referenced by
// the hub modules (i8042prt/keyboard/mouse/pnp) that are not in ntkrnl-sys.
// Values taken verbatim from the named SDK headers.
// =====================================================================

// ---- ntdd8042.h: keyboard scan-state E0/E1 prefix flags ----
pub const KEY_E0: u32 = 0x00000020; // ntdd8042.h:103
pub const KEY_E1: u32 = 0x00000040; // ntdd8042.h:104

// ---- ntdd8042.h: filter-hook IOCTLs ----
// CTL_CODE(DeviceType, Function, Method, Access) =
//   (DeviceType<<16)|(Access<<14)|(Function<<2)|Method
// KEYBOARD: FILE_DEVICE_KEYBOARD=0x0b, fn=0x0FF0, METHOD_NEITHER=3, ANY=0
// MOUSE:    FILE_DEVICE_MOUSE=0x0f,    fn=0x0FF0, METHOD_NEITHER=3, ANY=0
pub const IOCTL_INTERNAL_I8042_HOOK_KEYBOARD: u32 = 0x000B_3FC3;
pub const IOCTL_INTERNAL_I8042_HOOK_MOUSE: u32 = 0x000F_3FC3;

// ---- ntdd8042.h: i8042 mouse internal IOCTLs ----
// MOUSE=0x0f, METHOD_NEITHER=3, ANY=0; fn 0x0FF3 / 0x0FF1
pub const IOCTL_INTERNAL_I8042_MOUSE_START_INFORMATION: u32 = 0x000F_3FCF;
pub const IOCTL_INTERNAL_I8042_MOUSE_WRITE_BUFFER: u32 = 0x000F_3FC7;

// ---- kbdmou.h: internal CONNECT/DISCONNECT IOCTLs (METHOD_NEITHER=3) ----
pub const IOCTL_INTERNAL_KEYBOARD_CONNECT: u32 = 0x000B_0203;
pub const IOCTL_INTERNAL_KEYBOARD_DISCONNECT: u32 = 0x000B_0403;
pub const IOCTL_INTERNAL_MOUSE_CONNECT: u32 = 0x000F_0203;
pub const IOCTL_INTERNAL_MOUSE_DISCONNECT: u32 = 0x000F_0403;

// ---- ntddkbd.h: keyboard class IOCTLs (METHOD_BUFFERED=0, ANY=0) ----
pub const IOCTL_KEYBOARD_QUERY_ATTRIBUTES: u32 = 0x000B_0000;
pub const IOCTL_KEYBOARD_SET_TYPEMATIC: u32 = 0x000B_0004;
pub const IOCTL_KEYBOARD_SET_INDICATORS: u32 = 0x000B_0008;
pub const IOCTL_KEYBOARD_QUERY_TYPEMATIC: u32 = 0x000B_0020;
pub const IOCTL_KEYBOARD_QUERY_INDICATORS: u32 = 0x000B_0040;
pub const IOCTL_KEYBOARD_QUERY_INDICATOR_TRANSLATION: u32 = 0x000B_0080;

// ---- ntddmou.h: mouse class IOCTLs ----
pub const IOCTL_MOUSE_QUERY_ATTRIBUTES: u32 = 0x000F_0000;

// ---- poclass.h: system button IOCTLs (BATTERY=0x29, BUFFERED=0, READ=1) ----
pub const IOCTL_GET_SYS_BUTTON_CAPS: u32 = 0x0029_4140;
pub const IOCTL_GET_SYS_BUTTON_EVENT: u32 = 0x0029_4144;

// ---- iotypes.h: PnP IRP minor function codes (MinorFunction is UCHAR) ----
pub const IRP_MN_QUERY_CAPABILITIES: u8 = 0x09;
pub const IRP_MN_FILTER_RESOURCE_REQUIREMENTS: u8 = 0x0D;
pub const IRP_MN_QUERY_PNP_DEVICE_STATE: u8 = 0x14;

// ---- cmtypes.h: CM_RESOURCE_TYPE values (resource descriptor Type field) ----
pub const CmResourceTypePort: u32 = 1;
pub const CmResourceTypeInterrupt: u32 = 2;

// ---- ntddkbd.h: keyboard indicator (LED) flags ----
pub const KEYBOARD_SCROLL_LOCK_ON: u32 = 1; // ntddkbd.h:105
pub const KEYBOARD_NUM_LOCK_ON: u32 = 2; // ntddkbd.h:103
pub const KEYBOARD_CAPS_LOCK_ON: u32 = 4; // ntddkbd.h:104

// ---- ntddkbd.h: INDICATOR_LIST ----
#[repr(C)]
#[derive(Clone, Copy)]
pub struct INDICATOR_LIST {
    pub MakeCode: u16,
    pub IndicatorFlags: u16,
}
pub type PINDICATOR_LIST = *mut INDICATOR_LIST;

// ---- kbdmou.h: PSERVICE_CALLBACK_ROUTINE (VOID NTAPI(PVOID×4)) ----
pub type PSERVICE_CALLBACK_ROUTINE = Option<
    unsafe extern "system" fn(
        NormalContext: *mut c_void,
        SystemArgument1: *mut c_void,
        SystemArgument2: *mut c_void,
        SystemArgument3: *mut c_void,
    ),
>;

// ---- ntddclas.h: system power-button flags ----
pub const SYS_BUTTON_POWER: u32 = 0x00000001; // ntddclas.h:23
pub const SYS_BUTTON_SLEEP: u32 = 0x00000002; // ntddclas.h:24
pub const SYS_BUTTON_WAKE: u32 = 0x00000004; // ntddclas.h:25

// ---- ntddmou.h: mouse indicator / button / wheel flags (LastX/ButtonFlags) ----
pub const MOUSE_MOVE_RELATIVE: u16 = 0x0000; // ntddmou.h (MOUSE_INPUT_DATA.Flags)
// RawButtons bits — OR'd into MOUSE_INPUT_DATA.RawButtons (ULONG). u32 to match.
pub const MOUSE_BUTTON_4_DOWN: u32 = 0x0040; // ntddmou.h
pub const MOUSE_BUTTON_5_DOWN: u32 = 0x0100; // ntddmou.h
pub const MOUSE_WHEEL: u32 = 0x0400; // ntddmou.h

// ---- ntddmou.h: i8042 hardware-present identifier flags ----
pub const BALLPOINT_I8042_HARDWARE: u32 = 0x20; // BALLPOINT_I8042_HARDWARE_PRESENT
pub const WHEELMOUSE_I8042_HARDWARE: u32 = 0x40; // WHEELMOUSE_I8042_HARDWARE_PRESENT
pub const MOUSE_I8042_HARDWARE: u32 = 0x80; // MOUSE_I8042_HARDWARE_PRESENT

// ---- ntddk.h / kdfuncs.h: crash + debug-control ----
pub const MANUALLY_INITIATED_CRASH: u32 = 0xDEADDEAD; // ntddk.h:5612
pub const DBG_STATUS_SYSRQ: u32 = 0x02; // kdfuncs.h:74

// ---- winnt.h: registry access mask (NT5 value) ----
pub const KEY_ALL_ACCESS: u32 = 0xF003F;

// ---- wdm.h: latched interrupt resource flag ----
pub const CM_RESOURCE_INTERRUPT_LATCHED: u8 = 0x01;

// ---- ULongToPtr intrinsic ----
#[inline]
pub unsafe fn ULongToPtr(u: u32) -> *mut c_void {
    u as usize as *mut c_void
}

// ---- poclass.h: GUID_DEVICE_SYS_BUTTON ----
// DEFINE_GUID(GUID_DEVICE_SYS_BUTTON, 0x4AFA3D53, 0x74A7, 0x11d0,
//             0xbe,0x5e,0x00,0xA0,0xC9,0x06,0x28,0x57)
pub static GUID_DEVICE_SYS_BUTTON: ntkrnl_sys::ddk::GUID = ntkrnl_sys::ddk::GUID {
    Data1: 0x4AFA3D53,
    Data2: 0x74A7,
    Data3: 0x11d0,
    Data4: [0xbe, 0x5e, 0x00, 0xA0, 0xC9, 0x06, 0x28, 0x57],
};

// =====================================================================
// NTSTATUS codes used by the hubs that are not in ntkrnl-sys constants.
// Values from ntstatus.h.
// =====================================================================
pub const STATUS_IO_TIMEOUT: NTSTATUS = 0xC00000B5u32 as NTSTATUS;
pub const STATUS_IO_DEVICE_ERROR: NTSTATUS = 0xC0000185u32 as NTSTATUS;
pub const STATUS_INTERNAL_ERROR: NTSTATUS = 0xC00000E5u32 as NTSTATUS;
pub const STATUS_UNEXPECTED_IO_ERROR: NTSTATUS = 0xC00000E9u32 as NTSTATUS;
pub const STATUS_DEVICE_BUSY: NTSTATUS = 0x80000011u32 as NTSTATUS;
pub const STATUS_ABANDONED: NTSTATUS = 0x00000080u32 as NTSTATUS;

// =====================================================================
// Prefixed enum-variant aliases. rustconv emitted some enum variants with the
// bindgen-style `_ENUM_Variant` spelling; the enums here are hand-written so
// the plain variant is in scope via the `pub use ENUM::*` lines above. Provide
// the prefixed spelling as aliases so those call sites resolve unchanged.
pub const _MOUSE_STATE_MouseIdle: MOUSE_STATE = MOUSE_STATE::MouseIdle;
pub const _MOUSE_STATE_XMovement: MOUSE_STATE = MOUSE_STATE::XMovement;
pub const _MOUSE_STATE_YMovement: MOUSE_STATE = MOUSE_STATE::YMovement;
pub const _MOUSE_STATE_ZMovement: MOUSE_STATE = MOUSE_STATE::ZMovement;
pub const _MOUSE_STATE_MouseExpectingACK: MOUSE_STATE = MOUSE_STATE::MouseExpectingACK;
pub const _MOUSE_STATE_MouseResetting: MOUSE_STATE = MOUSE_STATE::MouseResetting;

pub const _MOUSE_TYPE_GenericPS2: I8042_MOUSE_TYPE = I8042_MOUSE_TYPE::GenericPS2;
pub const _MOUSE_TYPE_Intellimouse: I8042_MOUSE_TYPE = I8042_MOUSE_TYPE::Intellimouse;
pub const _MOUSE_TYPE_IntellimouseExplorer: I8042_MOUSE_TYPE = I8042_MOUSE_TYPE::IntellimouseExplorer;
pub const _MOUSE_TYPE_Ps2pp: I8042_MOUSE_TYPE = I8042_MOUSE_TYPE::Ps2pp;

pub const _MOUSE_TIMEOUT_STATE_Idle: TRANSMIT_STATE = TRANSMIT_STATE::Idle;
pub const _MOUSE_TIMEOUT_STATE_SendingBytes: TRANSMIT_STATE = TRANSMIT_STATE::SendingBytes;

// ============================================================================
// i386 size cross-check vs the C build (ground truth from szprobe against the
// real kernel build flags). A mismatch here = pool corruption at IoCreateDevice
// (the device extension is allocated/zeroed at the Rust size; if any embedded
// DDK type or field layout diverges from C the totals drift). Guarded to i386
// because that is the arch the cohab flip boots; extend per-arch as needed.
// ============================================================================
#[cfg(target_arch = "x86")]
const _: () = {
    assert!(core::mem::size_of::<FDO_DEVICE_EXTENSION>() == 0x20);
    assert!(core::mem::size_of::<I8042_KEYBOARD_EXTENSION>() == 0xb8);
    assert!(core::mem::size_of::<I8042_MOUSE_EXTENSION>() == 0xe0);
    assert!(core::mem::size_of::<PORT_DEVICE_EXTENSION>() == 0xcc);
    assert!(core::mem::size_of::<I8042_DRIVER_EXTENSION>() == 0xe0);
    assert!(core::mem::size_of::<I8042_SETTINGS>() == 0x44);
    assert!(core::mem::size_of::<OUTPUT_PACKET>() == 0x10);
    assert!(core::mem::size_of::<INTERRUPT_DATA>() == 0x18);
    assert!(core::mem::size_of::<INTERNAL_I8042_HOOK_KEYBOARD>() == 0x18);
    assert!(core::mem::size_of::<INTERNAL_I8042_HOOK_MOUSE>() == 0x14);
};

// Connect-path type sizes vs C (i386 ground truth). KeyboardBuffer/MouseBuffer
// allocations = QueueSize * sizeof(INPUT_DATA); a mismatch here mis-sizes the
// pool block and the RtlZeroMemory/ISR writes overrun it (BAD_POOL_HEADER 0x19).
#[cfg(target_arch = "x86")]
const _: () = {
    assert!(core::mem::size_of::<ntkrnl_sys::ddk::KEYBOARD_INPUT_DATA>() == 12);
    assert!(core::mem::size_of::<ntkrnl_sys::ddk::MOUSE_INPUT_DATA>() == 24);
    assert!(core::mem::size_of::<CONNECT_DATA>() == 8);
    assert!(core::mem::size_of::<KEYBOARD_ATTRIBUTES>() == 28);
    assert!(core::mem::size_of::<MOUSE_ATTRIBUTES>() == 12);
    assert!(core::mem::size_of::<KEYBOARD_INDICATOR_PARAMETERS>() == 4);
    assert!(core::mem::size_of::<INTERNAL_I8042_HOOK_KEYBOARD>() == 24);
    assert!(core::mem::size_of::<INTERNAL_I8042_HOOK_MOUSE>() == 20);
    assert!(core::mem::size_of::<I8042_HOOK_WORKITEM>() == 8);
};
