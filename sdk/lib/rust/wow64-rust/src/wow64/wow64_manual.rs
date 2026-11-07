use crate::wow64::wow64_cb_automarshal::LONG;
use crate::wow64::wow64_cb_automarshal::LONGLONG;
use crate::wow64::wow64_cb_automarshal::ULONG;
use crate::wow64::wow64_automarshal::Wow64AutoMarshalDispatch;
use crate::wow64::wow64_cb_automarshal::Wow64CbFindMarshal;
use crate::wow64::wow64_pfnmap::Wow64pMapClientProc;
use crate::wow64::wow64_pfnmap::Wow64pPfnMapSetGuest;
use crate::wow64::wow64_pfnmap::Wow64pPfnMapSetPsi;
use crate::wow64::wow64_pfnmap::Wow64pUnmapClientProc;
// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_96A296D224F2: [u16; 1] = [0x0000u16];

use core::ffi::c_void;

extern "system" {
    fn NtQueryValueKey( _: HANDLE, _: PVOID, _: ULONG, _: PVOID, _: ULONG, _: PULONG, ) -> NTSTATUS;
    fn NtUserBuildPropList(_: PVOID, _: PVOID, _: ULONG, _: *mut ULONG) -> NTSTATUS;
    fn NtUserDrawCaptionTemp( _: PVOID, _: PVOID, _: PVOID, _: PVOID, _: PVOID, _: PVOID, _: ULONG, ) -> LONG;
    fn NtUserSetCursorIconData(_: HANDLE, _: PVOID, _: PVOID, _: PVOID) -> LONG;
    fn NtQueryObject(_: HANDLE, _: ULONG, _: PVOID, _: ULONG, _: PULONG) -> NTSTATUS;
}

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
static mut Wow64pCsrBssd32: [::core::ffi::c_uchar; 6856] = [0; 6856];

static mut Wow64pCsrStaticServerData32: [ULONG; 4] = [0; 4];

static mut Wow64pCsrPortDelta: ULONG_PTR = 0;

static mut Wow64pCsrViewLocal: ULONG_PTR = 0;

static mut Wow64pCsrViewSize: ULONG = 0;

static mut Wow64pCsrScratchLocal: ULONG_PTR = 0;

static mut Wow64pCsrScratchSize: ULONG = 0;

pub type NTSTATUS = LONG;

// C declares these `extern` and wow64.c DEFINES them; a second Rust definition here was a
// private copy that stayed None forever (NtContinue returned STATUS_INVALID_PARAMETER and
// every i386 guest died after its loader). Share wow64.rs's.
use crate::wow64::wow64::{Wow64pBTCpuGetContext, Wow64pBTCpuNotifyMap, Wow64pBTCpuSetContext};

#[repr(C)]
#[derive(Clone, Copy)]
pub struct WOW64_UNICODE_STRING32_M {
    pub Length: USHORT,
    pub MaximumLength: USHORT,
    pub Buffer: ULONG,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct WOW64_UNICODE_STRING64_M {
    pub Length: USHORT,
    pub MaximumLength: USHORT,
    pub __pad: ULONG,
    pub Buffer: ULONGLONG,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct WOW64_OBJECT_ATTRIBUTES32_M {
    pub Length: ULONG,
    pub RootDirectory: ULONG,
    pub ObjectName: ULONG,
    pub Attributes: ULONG,
    pub SecurityDescriptor: ULONG,
    pub SecurityQualityOfService: ULONG,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct WOW64_OBJECT_ATTRIBUTES64_M {
    pub Length: ULONG,
    pub __pad0: ULONG,
    pub RootDirectory: ULONGLONG,
    pub ObjectName: ULONGLONG,
    pub Attributes: ULONG,
    pub __pad1: ULONG,
    pub SecurityDescriptor: ULONGLONG,
    pub SecurityQualityOfService: ULONGLONG,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct WOW64_MEMORY_BASIC_INFORMATION32 {
    pub BaseAddress: ULONG,
    pub AllocationBase: ULONG,
    pub AllocationProtect: ULONG,
    pub RegionSize: ULONG,
    pub State: ULONG,
    pub Protect: ULONG,
    pub Type: ULONG,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct WOW64_MEMORY_BASIC_INFORMATION64 {
    pub BaseAddress: ULONGLONG,
    pub AllocationBase: ULONGLONG,
    pub AllocationProtect: ULONG,
    pub __align0: ULONG,
    pub RegionSize: ULONGLONG,
    pub State: ULONG,
    pub Protect: ULONG,
    pub Type: ULONG,
    pub __align1: ULONG,
}

pub const WOW64_MEMORY_BASIC_INFORMATION_CLASS: u32 = 0;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct WOW64_PROCESS_BASIC_INFORMATION32 {
    pub ExitStatus: NTSTATUS,
    pub PebBaseAddress: ULONG,
    pub AffinityMask: ULONG,
    pub BasePriority: LONG,
    pub UniqueProcessId: ULONG,
    pub InheritedFromUniqueProcessId: ULONG,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct WOW64_PROCESS_BASIC_INFORMATION64 {
    pub ExitStatus: NTSTATUS,
    pub __pad: ULONG,
    pub PebBaseAddress: ULONGLONG,
    pub AffinityMask: ULONGLONG,
    pub BasePriority: LONG,
    pub __pad2: ULONG,
    pub UniqueProcessId: ULONGLONG,
    pub InheritedFromUniqueProcessId: ULONGLONG,
}

pub const WOW64_PROCESS_CURRENT: HANDLE = (-1isize) as HANDLE;

#[allow(non_upper_case_globals)]
pub const ProcessBasicInformation: u32 = 0;

#[allow(non_upper_case_globals)]
pub const ProcessWow64Information: u32 = 26;

pub const WOW64_ProcessExecuteFlags: u32 = 0x22;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct WOW64_SYSTEM_BASIC_INFORMATION32 {
    pub Reserved: ULONG,
    pub TimerResolution: ULONG,
    pub PageSize: ULONG,
    pub NumberOfPhysicalPages: ULONG,
    pub LowestPhysicalPageNumber: ULONG,
    pub HighestPhysicalPageNumber: ULONG,
    pub AllocationGranularity: ULONG,
    pub MinimumUserModeAddress: ULONG,
    pub MaximumUserModeAddress: ULONG,
    pub ActiveProcessorsAffinityMask: ULONG,
    pub NumberOfProcessors: UCHAR,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct WOW64_SYSTEM_BASIC_INFORMATION64 {
    pub Reserved: ULONG,
    pub TimerResolution: ULONG,
    pub PageSize: ULONG,
    pub NumberOfPhysicalPages: ULONG,
    pub LowestPhysicalPageNumber: ULONG,
    pub HighestPhysicalPageNumber: ULONG,
    pub AllocationGranularity: ULONG,
    pub __pad: ULONG,
    pub MinimumUserModeAddress: ULONGLONG,
    pub MaximumUserModeAddress: ULONGLONG,
    pub ActiveProcessorsAffinityMask: ULONGLONG,
    pub NumberOfProcessors: UCHAR,
}

#[allow(non_upper_case_globals)]
pub const SystemBasicInformation: u32 = 0;

#[allow(non_upper_case_globals)]
pub const SystemEmulationBasicInformation: u32 = 62;

pub type WOW64_OPEN_FN = Option<unsafe extern "system" fn(PHANDLE, ACCESS_MASK, PVOID) -> NTSTATUS>;
pub type WOW64_OPEN_FN_U32 = Option<unsafe extern "system" fn(PHANDLE, ACCESS_MASK, PVOID, ULONG) -> NTSTATUS>;
pub type WOW64_PORT_FN = Option<unsafe extern "system" fn(PHANDLE, PVOID, ULONG, ULONG, ULONG) -> NTSTATUS>;

pub const WOW64_TEB_WOWTEBOFFSET: u32 = 0x180c;
pub const WOW64_TEB32_ARBUSERPTR: u32 = 0x14;

pub type WOW64_VMLOCK_FN = Option<unsafe extern "system" fn(HANDLE, *mut PVOID, PSIZE_T, ULONG) -> NTSTATUS>;
pub type WOW64_USTR1_FN = Option<unsafe extern "system" fn(PVOID) -> NTSTATUS>;
pub type WOW64_OA1_FN = Option<unsafe extern "system" fn(PVOID) -> NTSTATUS>;
pub type WOW64_AUDIT_CLOSE_FN = Option<unsafe extern "system" fn(PVOID, PVOID, BOOLEAN) -> NTSTATUS>;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct WOW64_IO_STATUS_BLOCK32_M {
    pub Status: ULONG,
    pub Information: ULONG,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct WOW64_IO_STATUS_BLOCK64_M {
    pub Status: ULONGLONG,
    pub Information: ULONGLONG,
}

pub const WOW64_REDIR_MAX: u32 = 260;

pub const WOW64_STATUS_PENDING: NTSTATUS = 0x00000103;

pub const WOW64_STATUS_NOT_IMPLEMENTED: NTSTATUS = 0xC0000002u32 as NTSTATUS;

pub type WOW64_RW_FN = Option<unsafe extern "system" fn(HANDLE, HANDLE, PVOID, PVOID, PVOID, PVOID, ULONG, PLARGE_INTEGER, PULONG) -> NTSTATUS>;

pub const WOW64_TEB_INSTRUMENTATION0: u32 = 5816;

pub const WOW64_FileRenameInformation: u32 = 10;
pub const WOW64_FileLinkInformation: u32 = 11;
pub const WOW64_FileCompletionInformation: u32 = 30;
pub const WOW64_FileMoveClusterInformation: u32 = 31;

pub const WOW64_ZEROBITS_BELOW_2G: ULONG_PTR = 0x7FFFFFFF;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct WOW64_PORT_VIEW32_M {
    pub Length: ULONG,
    pub SectionHandle: ULONG,
    pub SectionOffset: ULONG,
    pub ViewSize: ULONG,
    pub ViewBase: ULONG,
    pub ViewRemoteBase: ULONG,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct WOW64_PORT_VIEW64_M {
    pub Length: ULONG,
    pub __pad0: ULONG,
    pub SectionHandle: ULONGLONG,
    pub SectionOffset: ULONG,
    pub __pad1: ULONG,
    pub ViewSize: ULONGLONG,
    pub ViewBase: ULONGLONG,
    pub ViewRemoteBase: ULONGLONG,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct WOW64_REMOTE_PORT_VIEW32_M {
    pub Length: ULONG,
    pub ViewSize: ULONG,
    pub ViewBase: ULONG,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct WOW64_REMOTE_PORT_VIEW64_M {
    pub Length: ULONG,
    pub __pad0: ULONG,
    pub ViewSize: ULONGLONG,
    pub ViewBase: ULONGLONG,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct WOW64_CSR_CONNECTINFO32_M {
    pub ObjectDirectory: ULONG,
    pub SharedSectionBase: ULONG,
    pub SharedStaticServerData: ULONG,
    pub SharedSectionHeap: ULONG,
    pub DebugFlags: ULONG,
    pub SizeOfPebData: ULONG,
    pub SizeOfTebData: ULONG,
    pub NumberOfServerDllNames: ULONG,
    pub ServerProcessId: ULONG,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct WOW64_CSR_CONNECTINFO64_M {
    pub ObjectDirectory: ULONGLONG,
    pub SharedSectionBase: ULONGLONG,
    pub SharedStaticServerData: ULONGLONG,
    pub SharedSectionHeap: ULONGLONG,
    pub DebugFlags: ULONG,
    pub SizeOfPebData: ULONG,
    pub SizeOfTebData: ULONG,
    pub NumberOfServerDllNames: ULONG,
    pub ServerProcessId: ULONGLONG,
}

pub const WOW64_CSR_SCRATCH_SIZE: u32 = 0x1000;

pub const WOW64_CSR_SERVER_DLL_MAX: usize = 4;

pub const WOW64_BASESRV_SERVERDLL_INDEX: u32 = 1;

pub const WOW64_CSRAPI_BASEP_CREATEPROCESS: u32 = 0x00010000;

pub const WOW64_CSRAPI_BASEP_CREATETHREAD: u32 = 0x00010001;

pub const WOW64_BSSD32_SIZE: usize = 0x1AC8;

pub const WOW64_BSSD_WINDIR32: u32 = 0;
pub const WOW64_BSSD_WINDIR64: u32 = 0;
pub const WOW64_BSSD_SYSDIR32: u32 = 8;
pub const WOW64_BSSD_SYSDIR64: u32 = 16;
pub const WOW64_BSSD_OBJDIR32: u32 = 16;
pub const WOW64_BSSD_OBJDIR64: u32 = 32;
pub const WOW64_BSSD_VERBLK32: u32 = 24;
pub const WOW64_BSSD_VERBLK64: u32 = 48;
pub const WOW64_BSSD_VERBLK_LEN: u32 = 268;
pub const WOW64_BSSD_SYSINFO32: u32 = 292;
pub const WOW64_BSSD_SYSINFO64: u32 = 320;
pub const WOW64_BSSD_TIMEOFDAY32: u32 = 336;
pub const WOW64_BSSD_TIMEOFDAY64: u32 = 384;
pub const WOW64_BSSD_TIMEOFDAY_LEN: u32 = 48;
pub const WOW64_BSSD_INIMAP32: u32 = 384;
pub const WOW64_BSSD_INIMAP64: u32 = 432;
pub const WOW64_BSSD_NLS32: u32 = 388;
pub const WOW64_BSSD_NLS64: u32 = 440;
pub const WOW64_BSSD_NLS_LEN: u32 = 0x1870;
pub const WOW64_BSSD_VDMFLAGS32: u32 = 6644;
pub const WOW64_BSSD_VDMFLAGS64: u32 = 6696;
pub const WOW64_BSSD_SYS32X86_32: u32 = 6648;
pub const WOW64_BSSD_SYS32X86_64: u32 = 6704;
pub const WOW64_BSSD_TSTAIL32: u32 = 6656;
pub const WOW64_BSSD_TSTAIL64: u32 = 6720;
pub const WOW64_BSSD_TSTAIL_LEN: u32 = 200;

pub const WOW64_USER_CEILING32: u64 = 0x7FFEFFFF;

pub const WOW64_SBI_PTRBLK32: u32 = 28;
pub const WOW64_SBI_PTRBLK64: u32 = 32;
pub const WOW64_SBI_NUMPROC32: u32 = 40;
pub const WOW64_SBI_NUMPROC64: u32 = 56;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct WOW64_SECTION_BASIC_INFORMATION64 {
    pub BaseAddress: ULONGLONG,
    pub Attributes: ULONG,
    pub __pad: ULONG,
    pub MaximumSize: ULONGLONG,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct WOW64_SECTION_IMAGE_INFORMATION64 {
    pub TransferAddress: ULONGLONG,
    pub ZeroBits: ULONG,
    pub __pad0: ULONG,
    pub MaximumStackSize: ULONGLONG,
    pub CommittedStackSize: ULONGLONG,
    pub SubSystemType: ULONG,
    pub SubSystemMinorVersion: USHORT,
    pub SubSystemMajorVersion: USHORT,
    pub GpValue: ULONG,
    pub ImageCharacteristics: USHORT,
    pub DllCharacteristics: USHORT,
    pub Machine: USHORT,
    pub ImageContainsCode: BOOLEAN,
    pub ImageFlags: UCHAR,
    pub LoaderFlags: ULONG,
    pub ImageFileSize: ULONG,
    pub CheckSum: ULONG,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct WOW64_SECTION_IMAGE_INFORMATION32 {
    pub TransferAddress: ULONG,
    pub ZeroBits: ULONG,
    pub MaximumStackSize: ULONG,
    pub CommittedStackSize: ULONG,
    pub SubSystemType: ULONG,
    pub SubSystemMinorVersion: USHORT,
    pub SubSystemMajorVersion: USHORT,
    pub GpValue: ULONG,
    pub ImageCharacteristics: USHORT,
    pub DllCharacteristics: USHORT,
    pub Machine: USHORT,
    pub ImageContainsCode: BOOLEAN,
    pub ImageFlags: UCHAR,
    pub LoaderFlags: ULONG,
    pub ImageFileSize: ULONG,
    pub CheckSum: ULONG,
}

pub const WOW64_CTX32_SIZE: u32 = 0x2CC;
pub const WOW64_CTX32_EIP: u32 = 0xB8;
pub const WOW64_CTX32_ESP: u32 = 0xC4;

pub const WOW64_FNID_NUM: usize = 31;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct WOW64_WNDMSG32 {
    pub maxMsgs: ULONG,
    pub abMsgs: ULONG,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct WOW64_WNDMSG64 {
    pub maxMsgs: ULONG,
    pub __pad: ULONG,
    pub abMsgs: ULONGLONG,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct WOW64_SHAREDINFO32 {
    pub psi: ULONG,
    pub aheList: ULONG,
    pub pDispInfo: ULONG,
    pub ulSharedDelta: ULONG,
    pub awmControl: [WOW64_WNDMSG32; WOW64_FNID_NUM],
    pub DefWindowMsgs: WOW64_WNDMSG32,
    pub DefWindowSpecMsgs: WOW64_WNDMSG32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct WOW64_SHAREDINFO64 {
    pub psi: ULONGLONG,
    pub aheList: ULONGLONG,
    pub pDispInfo: ULONGLONG,
    pub ulSharedDelta: ULONGLONG,
    pub awmControl: [WOW64_WNDMSG64; WOW64_FNID_NUM],
    pub DefWindowMsgs: WOW64_WNDMSG64,
    pub DefWindowSpecMsgs: WOW64_WNDMSG64,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct WOW64_USERCONNECT32 {
    pub ulVersion: ULONG,
    pub ulCurrentVersion: ULONG,
    pub dwDispatchCount: ULONG,
    pub siClient: WOW64_SHAREDINFO32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct WOW64_USERCONNECT64 {
    pub ulVersion: ULONG,
    pub ulCurrentVersion: ULONG,
    pub dwDispatchCount: ULONG,
    pub __pad: ULONG,
    pub siClient: WOW64_SHAREDINFO64,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct WOW64_MSG32 {
    pub hwnd: ULONG,
    pub message: ULONG,
    pub wParam: ULONG,
    pub lParam: ULONG,
    pub time: ULONG,
    pub ptx: LONG,
    pub pty: LONG,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct WOW64_MSG64 {
    pub hwnd: ULONGLONG,
    pub message: ULONG,
    pub _pad0: ULONG,
    pub wParam: ULONGLONG,
    pub lParam: ULONGLONG,
    pub time: ULONG,
    pub ptx: LONG,
    pub pty: LONG,
    pub _pad1: ULONG,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct WOW64_LARGE_STRING32 {
    pub Length: ULONG,
    pub MaximumLength: ULONG,
    pub Buffer: ULONG,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct WOW64_LARGE_STRING64 {
    pub Length: ULONG,
    pub MaximumLength: ULONG,
    pub Buffer: ULONGLONG,
}

pub const WOW64_GWL_WNDPROC: i32 = -4;
pub const WOW64_GCL_WNDPROC: i32 = -24;

pub const WOW64_PROPLISTITEM32_CB: u32 = 8;
pub const WOW64_PROPLISTITEM64_CB: u32 = 16;
pub const WOW64_PROPLIST_STACK: usize = 32;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct WOW64_PAINTSTRUCT32 {
    pub hdc: ULONG,
    pub fErase: ULONG,
    pub rc: [LONG; 4],
    pub fRestore: ULONG,
    pub fIncUpdate: ULONG,
    pub rgb: [UCHAR; 32],
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct WOW64_PAINTSTRUCT64 {
    pub hdc: ULONGLONG,
    pub fErase: ULONG,
    pub rc: [LONG; 4],
    pub fRestore: ULONG,
    pub fIncUpdate: ULONG,
    pub rgb: [UCHAR; 32],
    pub _pad: ULONG,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct WOW64_CURSORDATA32 {
    pub lpName: ULONG,
    pub lpModName: ULONG,
    pub rt: USHORT,
    pub dummy: USHORT,
    pub CURSORF_flags: ULONG,
    pub xHotspot: SHORT,
    pub yHotspot: SHORT,
    pub hbmMask: ULONG,
    pub hbmColor: ULONG,
    pub hbmAlpha: ULONG,
    pub rcBounds: [LONG; 4],
    pub hbmUserAlpha: ULONG,
    pub bpp: ULONG,
    pub cx: ULONG,
    pub cy: ULONG,
    pub cpcur: ULONG,
    pub cicur: ULONG,
    pub aspcur: ULONG,
    pub aicur: ULONG,
    pub ajifRate: ULONG,
    pub iicur: ULONG,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct WOW64_CURSORDATA64 {
    pub lpName: ULONGLONG,
    pub lpModName: ULONGLONG,
    pub rt: USHORT,
    pub dummy: USHORT,
    pub CURSORF_flags: ULONG,
    pub xHotspot: SHORT,
    pub yHotspot: SHORT,
    pub hbmMask: ULONGLONG,
    pub hbmColor: ULONGLONG,
    pub hbmAlpha: ULONGLONG,
    pub rcBounds: [LONG; 4],
    pub hbmUserAlpha: ULONGLONG,
    pub bpp: ULONG,
    pub cx: ULONG,
    pub cy: ULONG,
    pub cpcur: ULONG,
    pub cicur: ULONG,
    pub aspcur: ULONGLONG,
    pub aicur: ULONGLONG,
    pub ajifRate: ULONGLONG,
    pub iicur: UINT,
}

pub const WOW64_CURSORF_ACON: u32 = 0x0008;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct WOW64_ICONINFO32_M {
    pub fIcon: ULONG,
    pub xHotspot: ULONG,
    pub yHotspot: ULONG,
    pub hbmMask: ULONG,
    pub hbmColor: ULONG,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct WOW64_ICONINFO64_M {
    pub fIcon: ULONG,
    pub xHotspot: ULONG,
    pub yHotspot: ULONG,
    pub __pad: ULONG,
    pub hbmMask: ULONGLONG,
    pub hbmColor: ULONGLONG,
}

pub const WOW64_GDI_TYPE_MASK: u32 = 0x007f0000;
pub const WOW64_GDI_TYPE_BITMAP: u32 = 0x00050000;
pub const WOW64_GDI_TYPE_BRUSH: u32 = 0x00100000;
pub const WOW64_GDI_TYPE_PEN: u32 = 0x00300000;
pub const WOW64_GDI_TYPE_EXTPEN: u32 = 0x00500000;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct WOW64_BITMAP32_M {
    pub bmType: LONG,
    pub bmWidth: LONG,
    pub bmHeight: LONG,
    pub bmWidthBytes: LONG,
    pub bmPlanes: USHORT,
    pub bmBitsPixel: USHORT,
    pub bmBits: ULONG,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct WOW64_BITMAP64_M {
    pub bmType: LONG,
    pub bmWidth: LONG,
    pub bmHeight: LONG,
    pub bmWidthBytes: LONG,
    pub bmPlanes: USHORT,
    pub bmBitsPixel: USHORT,
    pub bmBits: ULONGLONG,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct WOW64_DIBSECTION32_M {
    pub dsBm: WOW64_BITMAP32_M,
    pub dsBmih: [BYTE; 40],
    pub dsBitfields: [ULONG; 3],
    pub dshSection: ULONG,
    pub dsOffset: ULONG,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct WOW64_DIBSECTION64_M {
    pub dsBm: WOW64_BITMAP64_M,
    pub dsBmih: [BYTE; 40],
    pub dsBitfields: [ULONG; 3],
    pub dshSection: ULONGLONG,
    pub dsOffset: ULONG,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct WOW64_LOGBRUSH32_M {
    pub lbStyle: ULONG,
    pub lbColor: ULONG,
    pub lbHatch: ULONG,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct WOW64_LOGBRUSH64_M {
    pub lbStyle: ULONG,
    pub lbColor: ULONG,
    pub lbHatch: ULONGLONG,
}

pub const WOW64_EXTPEN32_HEAD: u32 = 24;
pub const WOW64_EXTPEN64_HEAD: u32 = 28;

const _: () = assert!(core::mem::size_of::<WOW64_BITMAP32_M>() == 24);
const _: () = assert!(core::mem::size_of::<WOW64_BITMAP64_M>() == 32);
const _: () = assert!(core::mem::size_of::<WOW64_DIBSECTION32_M>() == 84);
const _: () = assert!(core::mem::size_of::<WOW64_DIBSECTION64_M>() == 104);
const _: () = assert!(core::mem::size_of::<WOW64_LOGBRUSH32_M>() == 12);
const _: () = assert!(core::mem::size_of::<WOW64_LOGBRUSH64_M>() == 16);

#[repr(C)]
#[derive(Clone, Copy)]
pub struct WOW64_CLIENT_ID32_M {
    pub UniqueProcess: ULONG,
    pub UniqueThread: ULONG,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct WOW64_CLIENT_ID64_M {
    pub UniqueProcess: ULONGLONG,
    pub UniqueThread: ULONGLONG,
}

pub type WOW64_OPEN_BY_CID_FN = Option<unsafe extern "system" fn(PHANDLE, ACCESS_MASK, PVOID, PVOID) -> NTSTATUS>;

pub const WOW64_STATUS_INVALID_PARAMETER: NTSTATUS = 0xC000000Du32 as NTSTATUS;
pub const WOW64_STATUS_NO_MEMORY: NTSTATUS = 0xC0000017u32 as NTSTATUS;

pub const WOW64_IOCTL_AFD_ACCEPT: ULONG = 0x12010;
pub const WOW64_IOCTL_AFD_RECV: ULONG = 0x12017;
pub const WOW64_IOCTL_AFD_RECV_DATAGRAM: ULONG = 0x1201B;
pub const WOW64_IOCTL_AFD_SEND: ULONG = 0x1201F;
pub const WOW64_IOCTL_AFD_SEND_DATAGRAM: ULONG = 0x12023;
pub const WOW64_IOCTL_AFD_SELECT: ULONG = 0x12024;
pub const WOW64_IOCTL_AFD_GET_TDI_HANDLES: ULONG = 0x12037;
pub const WOW64_IOCTL_AFD_EVENT_SELECT: ULONG = 0x12087;
pub const WOW64_IOCTL_AFD_ENUM_NETWORK_EVENTS: ULONG = 0x1208B;

// pshpack4 in the C (wow64_manual.c's AFD block).
#[repr(C, packed(4))]
#[derive(Clone, Copy)]
pub struct WOW64_AFD_WSABUF32 {
    pub len: ULONG,
    pub buf: ULONG,
}

// pshpack4 in the C (wow64_manual.c's AFD block).
#[repr(C, packed(4))]
#[derive(Clone, Copy)]
pub struct WOW64_AFD_XFER32 {
    pub BufferArray: ULONG,
    pub BufferCount: ULONG,
    pub AfdFlags: ULONG,
    pub TdiFlags: ULONG,
}

// pshpack4 in the C (wow64_manual.c's AFD block).
#[repr(C, packed(4))]
#[derive(Clone, Copy)]
pub struct WOW64_AFD_XFERUDP32 {
    pub BufferArray: ULONG,
    pub BufferCount: ULONG,
    pub AfdFlags: ULONG,
    pub TdiFlags: ULONG,
    pub Address: ULONG,
    pub AddressLength: ULONG,
}

// pshpack4 in the C (wow64_manual.c's AFD block).
#[repr(C, packed(4))]
#[derive(Clone, Copy)]
pub struct WOW64_AFD_SENDUDP32 {
    pub BufferArray: ULONG,
    pub BufferCount: ULONG,
    pub AfdFlags: ULONG,
    pub TdiRequest: [ULONG; 5],
    pub UserDataLength: LONG,
    pub UserData: ULONG,
    pub OptionsLength: LONG,
    pub Options: ULONG,
    pub RemoteAddressLength: LONG,
    pub RemoteAddress: ULONG,
}

// pshpack4 in the C (wow64_manual.c's AFD block).
#[repr(C, packed(4))]
#[derive(Clone, Copy)]
pub struct WOW64_AFD_HANDLE32 {
    pub Handle: ULONG,
    pub Events: ULONG,
    pub Status: LONG,
}

// pshpack4 in the C (wow64_manual.c's AFD block).
#[repr(C, packed(4))]
#[derive(Clone, Copy)]
pub struct WOW64_AFD_POLLHDR32 {
    pub Timeout: LONGLONG,
    pub HandleCount: ULONG,
    pub Exclusive: ULONG,
}

// pshpack4 in the C (wow64_manual.c's AFD block).
#[repr(C, packed(4))]
#[derive(Clone, Copy)]
pub struct WOW64_AFD_EVSEL32 {
    pub EventObject: ULONG,
    pub Events: ULONG,
}

// pshpack4 in the C (wow64_manual.c's AFD block).
#[repr(C, packed(4))]
#[derive(Clone, Copy)]
pub struct WOW64_AFD_ENUMEV32 {
    pub Event: ULONG,
    pub PollEvents: ULONG,
    pub EventStatus: [LONG; 10],
}

// pshpack4 in the C (wow64_manual.c's AFD block).
#[repr(C, packed(4))]
#[derive(Clone, Copy)]
pub struct WOW64_AFD_ACCEPT32 {
    pub UseSAN: ULONG,
    pub SequenceNumber: ULONG,
    pub ListenHandle: ULONG,
}

// pshpack4 in the C (wow64_manual.c's AFD block).
#[repr(C, packed(4))]
#[derive(Clone, Copy)]
pub struct WOW64_AFD_TDIH32 {
    pub TdiAddressHandle: ULONG,
    pub TdiConnectionHandle: ULONG,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct WOW64_AFD_WSABUF64 {
    pub len: ULONG,
    pub _p: ULONG,
    pub buf: ULONGLONG,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct WOW64_AFD_XFER64 {
    pub BufferArray: ULONGLONG,
    pub BufferCount: ULONG,
    pub AfdFlags: ULONG,
    pub TdiFlags: ULONG,
    pub _p: ULONG,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct WOW64_AFD_XFERUDP64 {
    pub BufferArray: ULONGLONG,
    pub BufferCount: ULONG,
    pub AfdFlags: ULONG,
    pub TdiFlags: ULONG,
    pub _p: ULONG,
    pub Address: ULONGLONG,
    pub AddressLength: ULONGLONG,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct WOW64_AFD_SENDUDP64 {
    pub BufferArray: ULONGLONG,
    pub BufferCount: ULONG,
    pub AfdFlags: ULONG,
    pub TdiRequest: [ULONGLONG; 5],
    pub UserDataLength: LONG,
    pub _p0: ULONG,
    pub UserData: ULONGLONG,
    pub OptionsLength: LONG,
    pub _p1: ULONG,
    pub Options: ULONGLONG,
    pub RemoteAddressLength: LONG,
    pub _p2: ULONG,
    pub RemoteAddress: ULONGLONG,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct WOW64_AFD_HANDLE64 {
    pub Handle: ULONGLONG,
    pub Events: ULONG,
    pub Status: LONG,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct WOW64_AFD_POLLHDR64 {
    pub Timeout: LONGLONG,
    pub HandleCount: ULONG,
    pub _p: ULONG,
    pub Exclusive: ULONGLONG,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct WOW64_AFD_EVSEL64 {
    pub EventObject: ULONGLONG,
    pub Events: ULONG,
    pub _p: ULONG,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct WOW64_AFD_ENUMEV64 {
    pub Event: ULONGLONG,
    pub PollEvents: ULONG,
    pub EventStatus: [LONG; 10],
    pub _p: ULONG,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct WOW64_AFD_ACCEPT64 {
    pub UseSAN: ULONG,
    pub SequenceNumber: ULONG,
    pub ListenHandle: ULONGLONG,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct WOW64_AFD_TDIH64 {
    pub TdiAddressHandle: ULONGLONG,
    pub TdiConnectionHandle: ULONGLONG,
}

const _: () = assert!(core::mem::size_of::<WOW64_AFD_WSABUF32>() == 8);
const _: () = assert!(core::mem::size_of::<WOW64_AFD_WSABUF64>() == 16);
const _: () = assert!(core::mem::size_of::<WOW64_AFD_XFER32>() == 16);
const _: () = assert!(core::mem::size_of::<WOW64_AFD_XFER64>() == 24);
const _: () = assert!(core::mem::size_of::<WOW64_AFD_XFERUDP32>() == 24);
const _: () = assert!(core::mem::size_of::<WOW64_AFD_XFERUDP64>() == 40);
const _: () = assert!(core::mem::size_of::<WOW64_AFD_SENDUDP32>() == 56);
const _: () = assert!(core::mem::size_of::<WOW64_AFD_SENDUDP64>() == 104);
const _: () = assert!(core::mem::size_of::<WOW64_AFD_HANDLE32>() == 12);
const _: () = assert!(core::mem::size_of::<WOW64_AFD_HANDLE64>() == 16);
const _: () = assert!(core::mem::size_of::<WOW64_AFD_EVSEL32>() == 8);
const _: () = assert!(core::mem::size_of::<WOW64_AFD_EVSEL64>() == 16);
const _: () = assert!(core::mem::size_of::<WOW64_AFD_ENUMEV32>() == 48);
const _: () = assert!(core::mem::size_of::<WOW64_AFD_ENUMEV64>() == 56);
const _: () = assert!(core::mem::size_of::<WOW64_AFD_ACCEPT32>() == 12);
const _: () = assert!(core::mem::size_of::<WOW64_AFD_ACCEPT64>() == 16);
const _: () = assert!(core::mem::size_of::<WOW64_AFD_TDIH32>() == 8);
const _: () = assert!(core::mem::size_of::<WOW64_AFD_TDIH64>() == 16);
const _: () = assert!(core::mem::size_of::<WOW64_AFD_POLLHDR32>() == 16);
const _: () = assert!(core::mem::size_of::<WOW64_AFD_POLLHDR64>() == 24);

pub const WOW64_AFD_MAX_BUFFERS: usize = 1024;
pub const WOW64_AFD_MAX_HANDLES: usize = 1024;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct WOW64_PORT_MESSAGE32_U1_S1 {
    pub DataLength: SHORT,
    pub TotalLength: SHORT,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub union WOW64_PORT_MESSAGE32_U1 {
    pub s1: WOW64_PORT_MESSAGE32_U1_S1,
    pub Length: ULONG,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct WOW64_PORT_MESSAGE32_U2_S2 {
    pub Type: SHORT,
    pub DataInfoOffset: SHORT,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub union WOW64_PORT_MESSAGE32_U2 {
    pub s2: WOW64_PORT_MESSAGE32_U2_S2,
    pub ZeroInit: ULONG,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct WOW64_PORT_MESSAGE32 {
    pub u1: WOW64_PORT_MESSAGE32_U1,
    pub u2: WOW64_PORT_MESSAGE32_U2,
    pub ClientIdProcess: ULONG,
    pub ClientIdThread: ULONG,
    pub MessageId: ULONG,
    pub ClientViewSize: ULONG,
}

const _: () = assert!(core::mem::size_of::<WOW64_PORT_MESSAGE32>() == 24);

#[repr(C)]
#[derive(Clone, Copy)]
pub struct WOW64_PORT_MESSAGE64_U1_S1 {
    pub DataLength: SHORT,
    pub TotalLength: SHORT,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub union WOW64_PORT_MESSAGE64_U1 {
    pub s1: WOW64_PORT_MESSAGE64_U1_S1,
    pub Length: ULONG,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct WOW64_PORT_MESSAGE64_U2_S2 {
    pub Type: SHORT,
    pub DataInfoOffset: SHORT,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub union WOW64_PORT_MESSAGE64_U2 {
    pub s2: WOW64_PORT_MESSAGE64_U2_S2,
    pub ZeroInit: ULONG,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct WOW64_PORT_MESSAGE64 {
    pub u1: WOW64_PORT_MESSAGE64_U1,
    pub u2: WOW64_PORT_MESSAGE64_U2,
    pub ClientIdProcess: ULONGLONG,
    pub ClientIdThread: ULONGLONG,
    pub MessageId: ULONG,
    pub __pad0: ULONG,
    pub ClientViewSize: ULONGLONG,
}

const _: () = assert!(core::mem::size_of::<WOW64_PORT_MESSAGE64>() == 40);

pub const WOW64_CSR_APINUMBER_OFF32: u32 = 24 + 4;
pub const WOW64_LPC_MAX_DATA32: usize = 256;

pub const WOW64_CSRMSG_BODY32_APINUMBER: u32 = 4;
pub const WOW64_CSRMSG_BODY32_STATUS: u32 = 8;
pub const WOW64_CSRMSG_BODY32_RESERVED: u32 = 12;
pub const WOW64_CSRMSG_BODY32_DATA: u32 = 16;
pub const WOW64_CSRMSG_BODY64_APINUMBER: u32 = 8;
pub const WOW64_CSRMSG_BODY64_STATUS: u32 = 12;
pub const WOW64_CSRMSG_BODY64_RESERVED: u32 = 16;
pub const WOW64_CSRMSG_BODY64_DATA: u32 = 24;

pub const WOW64_BCP32_SIZE: u32 = 140;
pub const WOW64_BCP64_SIZE: u32 = 232;

pub const WOW64_CAPBUF32_HDR: u32 = 16;
pub const WOW64_CAPBUF64_HDR: u32 = 32;
pub const WOW64_CAPBUF32_SIZE: u32 = 0;
pub const WOW64_CAPBUF32_POINTERCOUNT: u32 = 8;
pub const WOW64_CAPBUF64_SIZE: u32 = 0;
pub const WOW64_CAPBUF64_PREV: u32 = 8;
pub const WOW64_CAPBUF64_POINTERCOUNT: u32 = 16;
pub const WOW64_CAPBUF64_BUFFEREND: u32 = 24;

pub const WOW64_CSR_CLIENTCONNECT_PTR32: u32 = 44;
pub const WOW64_CSR_CLIENTCONNECT_PTR64: u32 = 72;

pub type WOW64_IOCTL_FN = Option<unsafe extern "system" fn(HANDLE, HANDLE, PVOID, PVOID, PVOID, ULONG, PVOID, ULONG, PVOID, ULONG) -> NTSTATUS>;

pub const WOW64_MC_FNID_SCROLLBAR: u32 = 0x029A;
pub const WOW64_MC_FNID_MENU: u32 = 0x029C;
pub const WOW64_MC_FNID_DESKTOP: u32 = 0x029D;
pub const WOW64_MC_FNID_DEFWINDOWPROC: u32 = 0x029E;
pub const WOW64_MC_FNID_MESSAGEWND: u32 = 0x029F;
pub const WOW64_MC_FNID_SENDMESSAGE: u32 = 0x02B1;
pub const WOW64_MC_FNID_SENDMESSAGEFF: u32 = 0x02B2;
pub const WOW64_MC_FNID_SENDMESSAGEWTOOPTION: u32 = 0x02B3;

pub const WOW64_MC_WM_CREATE: u32 = 0x0001;
pub const WOW64_MC_WM_COPYDATA: u32 = 0x004A;
pub const WOW64_MC_WM_NCCREATE: u32 = 0x0081;
pub const WOW64_MC_WM_NCCALCSIZE: u32 = 0x0083;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct WOW64_MC_DSM32 {
    pub uFlags: ULONG,
    pub uTimeout: ULONG,
    pub Result: ULONG,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct WOW64_MC_DSM64 {
    pub uFlags: ULONG,
    pub uTimeout: ULONG,
    pub Result: ULONGLONG,
}

pub const WOW64_MC_PAYLOAD64_MAX: usize = 64;

pub const WOW64_ThreadBasicInformation: u32 = 0;
pub const WOW64_ThreadTimes: u32 = 1;
pub const WOW64_ThreadQuerySetWin32StartAddress: u32 = 9;
pub const WOW64_ThreadPerformanceCount: u32 = 11;
pub const WOW64_ThreadAmILastThread: u32 = 12;
pub const WOW64_ThreadPriorityBoost: u32 = 14;
pub const WOW64_ThreadIsIoPending: u32 = 16;

pub const WOW64_TEBSLOT_WOWTEBOFFSET: u32 = 0x180c;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct WOW64_TBI64 {
    pub ExitStatus: ULONG,
    pub Pad0: ULONG,
    pub TebBaseAddress: ULONGLONG,
    pub UniqueProcess: ULONGLONG,
    pub UniqueThread: ULONGLONG,
    pub AffinityMask: ULONGLONG,
    pub Priority: LONG,
    pub BasePriority: LONG,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct WOW64_TBI32 {
    pub ExitStatus: ULONG,
    pub TebBaseAddress: ULONG,
    pub UniqueProcess: ULONG,
    pub UniqueThread: ULONG,
    pub AffinityMask: ULONG,
    pub Priority: LONG,
    pub BasePriority: LONG,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct WOW64_FILE_IO_COMPLETION_INFORMATION32_M {
    pub KeyContext: ULONG,
    pub ApcContext: ULONG,
    pub IoStatusBlock: WOW64_IO_STATUS_BLOCK32_M,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct WOW64_FILE_IO_COMPLETION_INFORMATION64_M {
    pub KeyContext: ULONGLONG,
    pub ApcContext: ULONGLONG,
    pub IoStatusBlock: WOW64_IO_STATUS_BLOCK64_M,
}

pub const WOW64_MON_MAX: usize = 16;

pub const WOW64_INPUT32_CB: u32 = 28;
pub const WOW64_INPUT64_CB: u32 = 40;
pub const WOW64_SENDINPUT_STACK: usize = 64;
pub const WOW64_INPUT_MOUSE: u32 = 0;
pub const WOW64_INPUT_KEYBOARD: u32 = 1;

pub const WOW64_CTX32_SEG_LO: u32 = 0x8C;
pub const WOW64_CTX32_SEG_HI: u32 = 0x98;
pub const WOW64_CTX32_INT_LO: u32 = 0x9C;
pub const WOW64_CTX32_INT_HI: u32 = 0xB0;
pub const WOW64_CTX32_EBX: u32 = 0xA4;
pub const WOW64_CTX32_EAX: u32 = 0xB0;
pub const WOW64_CTX32_CTL_LO: u32 = 0xB4;
pub const WOW64_CTX32_CTL_HI: u32 = 0xC8;
pub const WOW64_CTX32_ARCH: u32 = 0x00010000;
pub const WOW64_CTX32_F_CONTROL: u32 = 0x00000001;
pub const WOW64_CTX32_F_INTEGER: u32 = 0x00000002;
pub const WOW64_CTX32_F_SEGMENTS: u32 = 0x00000004;
pub const WOW64_CTX32_F_UNTRACKED: u32 = 0x00000038;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct WOW64_THREAD_BASIC_INFO {
    pub ExitStatus: NTSTATUS,
    pub TebBaseAddress: PVOID,
    pub UniqueProcess: ULONG_PTR,
    pub UniqueThread: ULONG_PTR,
    pub AffinityMask: ULONG_PTR,
    pub Priority: LONG,
    pub BasePriority: LONG,
}

pub const WOW64_MAXIMUM_WAIT_OBJECTS: u32 = 64;

pub const WOW64_ITEB32_STACKBASE: u32 = 0x08;
pub const WOW64_ITEB32_STACKLIMIT: u32 = 0x0C;
pub const WOW64_ITEB32_ALLOCBASE: u32 = 0x10;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct WOW64_CLIENT_ID {
    pub UniqueProcess: HANDLE,
    pub UniqueThread: HANDLE,
}

pub const WOW64_UPP32_SIZE: u32 = 0x290;
pub const WOW64_UPP64_SIZE: u32 = 0x3f0;
pub const WOW64_UPP32_HDR: u32 = 0x90;
pub const WOW64_PEB32_PARAMS_OFF: u32 = 0x10;
pub const WOW64_PEB64_PARAMS_OFF: u32 = 0x20;
pub const WOW64_PPF_NORMALIZED: u32 = 0x00000001;

#[allow(non_snake_case)]

static mut Wow64Manual_NtLoadKey_unused: () = ();

unsafe extern "C" fn Wow64pMarshalOa(
    mut Oa64: *mut WOW64_OBJECT_ATTRIBUTES64_M,
    mut Uname64: *mut WOW64_UNICODE_STRING64_M,
    mut Oa32Ptr: ULONG,
) {
    let mut Oa32: *mut WOW64_OBJECT_ATTRIBUTES32_M =
        Oa32Ptr as ULONG_PTR as *mut WOW64_OBJECT_ATTRIBUTES32_M;
    (*Oa64).Length = ::core::mem::size_of::<WOW64_OBJECT_ATTRIBUTES64_M>() as ULONG;
    (*Oa64).__pad0 = (0 as ULONG);
    if Oa32.is_null() {
        /* A guest may legally pass NULL where the parameter is _In_opt_, and an
         * ill-behaved one may pass it where it is not. Hand the kernel a valid
         * empty OA rather than dereferencing zero inside the thunk: a bad guest
         * argument must come back as a status code, not as a fault in wow64.dll.
         * Callers whose parameter is genuinely optional use the Opt form below,
         * which passes a real NULL through. */
        (*Oa64).RootDirectory = (0 as ULONGLONG);
        (*Oa64).Attributes = (0 as ULONG);
        (*Oa64).__pad1 = (0 as ULONG);
        (*Oa64).SecurityDescriptor = (0 as ULONGLONG);
        (*Oa64).SecurityQualityOfService = (0 as ULONGLONG);
        (*Oa64).ObjectName = (0 as ULONGLONG);
        return;
    }
    (*Oa64).RootDirectory = (*Oa32).RootDirectory as LONG as LONG_PTR as ULONGLONG; /* handle: sign-ext */
    (*Oa64).Attributes = (*Oa32).Attributes;
    (*Oa64).__pad1 = (0 as ULONG);
    (*Oa64).SecurityDescriptor = (*Oa32).SecurityDescriptor as ULONG_PTR as ULONGLONG;
    (*Oa64).SecurityQualityOfService = (*Oa32).SecurityQualityOfService as ULONG_PTR as ULONGLONG;
    (*Oa64).ObjectName = (0 as ULONGLONG);
    if (*Oa32).ObjectName != 0 {
        let mut N32: *mut WOW64_UNICODE_STRING32_M =
            (*Oa32).ObjectName as ULONG_PTR as *mut WOW64_UNICODE_STRING32_M;
        (*Uname64).Length = (*N32).Length;
        (*Uname64).MaximumLength = (*N32).MaximumLength;
        (*Uname64).__pad = (0 as ULONG);
        (*Uname64).Buffer = (*N32).Buffer as ULONG_PTR as ULONGLONG;
        (*Oa64).ObjectName = Uname64 as ULONG_PTR as ULONGLONG;
    }
}

unsafe extern "C" fn Wow64Manual_NtQueryVirtualMemory(a: *mut ULONG) -> NTSTATUS {
    let mut Process: HANDLE =
        *a.offset(0 as isize) as LONG as LONG_PTR as HANDLE;
    let mut Base: PVOID = *a.offset(1 as isize) as ULONG_PTR as PVOID;
    let mut Class: ULONG = *a.offset(2 as isize);
    let mut Buf32: PVOID = *a.offset(3 as isize) as ULONG_PTR as PVOID;
    let mut Len32: ULONG = *a.offset(4 as isize);
    let mut RetLen32: *mut ULONG =
        *a.offset(5 as isize) as ULONG_PTR as *mut ULONG;

    if Class == WOW64_MEMORY_BASIC_INFORMATION_CLASS as ULONG {
        let mut Mbi64: WOW64_MEMORY_BASIC_INFORMATION64 = core::mem::zeroed();
        let mut Mbi32: *mut WOW64_MEMORY_BASIC_INFORMATION32 =
            Buf32 as *mut WOW64_MEMORY_BASIC_INFORMATION32;
        let mut Ret: SIZE_T = 0 as SIZE_T;
        let mut St: NTSTATUS;

        if (Len32 as usize) < core::mem::size_of::<WOW64_MEMORY_BASIC_INFORMATION32>() {
            if !RetLen32.is_null() {
                *RetLen32 = core::mem::size_of::<WOW64_MEMORY_BASIC_INFORMATION32>() as ULONG;
            }
            return (0xC0000004 as ULONG) as NTSTATUS;
        }
        St = NtQueryVirtualMemory(
            Process,
            Base,
            WOW64_MEMORY_BASIC_INFORMATION_CLASS as ULONG,
            &raw mut Mbi64 as PVOID,
            core::mem::size_of::<WOW64_MEMORY_BASIC_INFORMATION64>() as SIZE_T,
            &raw mut Ret,
        );
        if St >= 0 as NTSTATUS && !Mbi32.is_null() {
            (*Mbi32).BaseAddress = Mbi64.BaseAddress as ULONG;
            (*Mbi32).AllocationBase = Mbi64.AllocationBase as ULONG;
            (*Mbi32).AllocationProtect = Mbi64.AllocationProtect;
            (*Mbi32).RegionSize = Mbi64.RegionSize as ULONG;
            (*Mbi32).State = Mbi64.State;
            (*Mbi32).Protect = Mbi64.Protect;
            (*Mbi32).Type = Mbi64.Type;
            if !RetLen32.is_null() {
                *RetLen32 = core::mem::size_of::<WOW64_MEMORY_BASIC_INFORMATION32>() as ULONG;
            }
        }
        return St;
    }

    {
        let mut Ret_0: SIZE_T = 0 as SIZE_T;
        let St_0: NTSTATUS = NtQueryVirtualMemory(
            Process,
            Base,
            Class,
            Buf32,
            Len32 as SIZE_T,
            &raw mut Ret_0,
        );
        if !RetLen32.is_null() {
            *RetLen32 = Ret_0 as ULONG;
        }
        return St_0;
    }
}

unsafe extern "C" fn Wow64Manual_NtQueryInformationProcess(a: *mut ULONG) -> NTSTATUS {
    let Process: HANDLE = *a.offset(0 as isize) as LONG as LONG_PTR as HANDLE;
    let Class: ULONG = *a.offset(1 as isize);
    let Buf32: PVOID = *a.offset(2 as isize) as ULONG_PTR as PVOID;
    let Len32: ULONG = *a.offset(3 as isize);
    let RetLen32: *mut ULONG = *a.offset(4 as isize) as ULONG_PTR as *mut ULONG;

    if Class == ProcessBasicInformation as ULONG {
        let mut Pbi64: WOW64_PROCESS_BASIC_INFORMATION64 = core::mem::zeroed();
        let Pbi32: *mut WOW64_PROCESS_BASIC_INFORMATION32 = Buf32 as *mut WOW64_PROCESS_BASIC_INFORMATION32;
        let mut Peb32: ULONG_PTR = 0 as ULONG_PTR;
        let mut Ret64: ULONG = 0 as ULONG;
        let mut St: NTSTATUS;
        if (Len32 as usize) < core::mem::size_of::<WOW64_PROCESS_BASIC_INFORMATION32>() as usize {
            if !RetLen32.is_null() {
                *RetLen32 = core::mem::size_of::<WOW64_PROCESS_BASIC_INFORMATION32>() as ULONG;
            }
            return 0xC0000004 as ULONG as NTSTATUS;
        }
        St = NtQueryInformationProcess(
            Process,
            ProcessBasicInformation as ULONG,
            core::ptr::addr_of_mut!(Pbi64) as PVOID,
            core::mem::size_of::<WOW64_PROCESS_BASIC_INFORMATION64>() as ULONG,
            core::ptr::addr_of_mut!(Ret64),
        );
        if St < 0 as NTSTATUS {
            return St;
        }
        /* PebBaseAddress must be the 32-bit PEB — query it explicitly. */
        NtQueryInformationProcess(
            Process,
            ProcessWow64Information as ULONG,
            core::ptr::addr_of_mut!(Peb32) as PVOID,
            core::mem::size_of::<ULONG_PTR>() as ULONG,
            core::ptr::null_mut::<ULONG>(),
        );
        (*Pbi32).ExitStatus = Pbi64.ExitStatus;
        (*Pbi32).PebBaseAddress = Peb32 as ULONG;
        (*Pbi32).AffinityMask = Pbi64.AffinityMask as ULONG;
        (*Pbi32).BasePriority = Pbi64.BasePriority;
        (*Pbi32).UniqueProcessId = Pbi64.UniqueProcessId as ULONG;
        (*Pbi32).InheritedFromUniqueProcessId = Pbi64.InheritedFromUniqueProcessId as ULONG;
        if !RetLen32.is_null() {
            *RetLen32 = core::mem::size_of::<WOW64_PROCESS_BASIC_INFORMATION32>() as ULONG;
        }
        return St;
    }
    if Class == ProcessWow64Information as ULONG {
        /* PEB32 pointer, 32-bit wide to the guest. */
        let mut Peb32: ULONG_PTR = 0 as ULONG_PTR;
        let St: NTSTATUS = NtQueryInformationProcess(
            Process,
            ProcessWow64Information as ULONG,
            core::ptr::addr_of_mut!(Peb32) as PVOID,
            core::mem::size_of::<ULONG_PTR>() as ULONG,
            core::ptr::null_mut::<ULONG>(),
        );
        if St >= 0 as NTSTATUS
            && !Buf32.is_null()
            && (Len32 as usize) >= core::mem::size_of::<ULONG>() as usize
        {
            *(Buf32 as *mut ULONG) = Peb32 as ULONG;
            if !RetLen32.is_null() {
                *RetLen32 = core::mem::size_of::<ULONG>() as ULONG;
            }
        }
        return St;
    }
    /* Scalar / layout-identical classes (ProcessDefaultHardErrorMode,
     * ProcessCookie, ...): pass the guest buffer straight through. Log the
     * class so a pointer-bearing one we haven't special-cased is visible. */
    {
        let mut Ret: ULONG = 0 as ULONG;
        let St: NTSTATUS = NtQueryInformationProcess(Process, Class, Buf32, Len32, core::ptr::addr_of_mut!(Ret));
        if !RetLen32.is_null() {
            *RetLen32 = Ret;
        }
        DbgPrint(
            b"[WOW64]   NtQueryInformationProcess class=%lu passthrough => %08lx (len=%lu)\n\0"
                .as_ptr() as *const i8,
            Class,
            St,
            Len32,
        );
        return St;
    }
}

unsafe extern "C" fn Wow64Manual_NtQuerySystemInformation(a: *mut ULONG) -> NTSTATUS {
    let Class: ULONG = *a.offset(0 as isize);
    let Buf32: PVOID = *a.offset(1 as isize) as ULONG_PTR as PVOID;
    let Len32: ULONG = *a.offset(2 as isize);
    let RetLen32: *mut ULONG = *a.offset(3 as isize) as ULONG_PTR as *mut ULONG;

    if Class == SystemBasicInformation as ULONG || Class == SystemEmulationBasicInformation as ULONG {
        let mut Sbi64: WOW64_SYSTEM_BASIC_INFORMATION64 = core::mem::zeroed();
        let Sbi32: *mut WOW64_SYSTEM_BASIC_INFORMATION32 = Buf32 as *mut WOW64_SYSTEM_BASIC_INFORMATION32;
        let mut Ret: ULONG = 0 as ULONG;
        let mut St: NTSTATUS;
        if Len32 < core::mem::size_of::<WOW64_SYSTEM_BASIC_INFORMATION32>() as ULONG {
            if !RetLen32.is_null() {
                *RetLen32 = core::mem::size_of::<WOW64_SYSTEM_BASIC_INFORMATION32>() as ULONG;
            }
            return 0xC0000004 as ULONG as NTSTATUS;
        }
        St = NtQuerySystemInformation(
            Class,
            core::ptr::addr_of_mut!(Sbi64) as PVOID,
            core::mem::size_of::<WOW64_SYSTEM_BASIC_INFORMATION64>() as ULONG,
            core::ptr::addr_of_mut!(Ret),
        );
        if St < 0 as NTSTATUS {
            return St;
        }
        (*Sbi32).Reserved = Sbi64.Reserved;
        (*Sbi32).TimerResolution = Sbi64.TimerResolution;
        (*Sbi32).PageSize = Sbi64.PageSize;
        (*Sbi32).NumberOfPhysicalPages = Sbi64.NumberOfPhysicalPages;
        (*Sbi32).LowestPhysicalPageNumber = Sbi64.LowestPhysicalPageNumber;
        (*Sbi32).HighestPhysicalPageNumber = Sbi64.HighestPhysicalPageNumber;
        (*Sbi32).AllocationGranularity = Sbi64.AllocationGranularity;
        (*Sbi32).MinimumUserModeAddress = Sbi64.MinimumUserModeAddress as ULONG;
        // Clamp the max user address to the 32-bit guest's 2GB ceiling.
        (*Sbi32).MaximumUserModeAddress =
            if Sbi64.MaximumUserModeAddress > 0x7FFEFFFF {
                0x7FFEFFFF as ULONG
            } else {
                Sbi64.MaximumUserModeAddress as ULONG
            };
        (*Sbi32).ActiveProcessorsAffinityMask = Sbi64.ActiveProcessorsAffinityMask as ULONG;
        (*Sbi32).NumberOfProcessors = Sbi64.NumberOfProcessors;
        if !RetLen32.is_null() {
            *RetLen32 = core::mem::size_of::<WOW64_SYSTEM_BASIC_INFORMATION32>() as ULONG;
        }
        return St;
    }
    if Class == 50 as ULONG {
        // The host answers 8 bytes; the guest asks for its own ULONG_PTR, 4.
        // The value follows the clamp above: a guest's user space ends at 0x7FFEFFFF.
        if Len32 < core::mem::size_of::<ULONG>() as ULONG {
            if !RetLen32.is_null() {
                *RetLen32 = core::mem::size_of::<ULONG>() as ULONG;
            }
            return 0xC0000004 as ULONG as NTSTATUS; // STATUS_INFO_LENGTH_MISMATCH
        }
        if Buf32.is_null() {
            return 0xC0000005 as ULONG as NTSTATUS; // STATUS_ACCESS_VIOLATION
        }
        *(Buf32 as *mut ULONG) = 0x80000000 as ULONG;
        if !RetLen32.is_null() {
            *RetLen32 = core::mem::size_of::<ULONG>() as ULONG;
        }
        return 0 as NTSTATUS;
    }
    {
        let mut Ret: ULONG = 0 as ULONG;
        let St: NTSTATUS = NtQuerySystemInformation(Class, Buf32, Len32, core::ptr::addr_of_mut!(Ret));
        if !RetLen32.is_null() {
            *RetLen32 = Ret;
        }
        DbgPrint(
            b"[WOW64]   NtQuerySystemInformation class=%lu passthrough => %08lx (len=%lu)\n\0"
                .as_ptr() as PCSTR,
            Class,
            St,
            Len32,
        );
        return St;
    }
}

unsafe extern "C" fn Wow64pOpenByOa(mut Fn: WOW64_OPEN_FN, mut a: *mut ULONG) -> NTSTATUS {
    let mut Handle32: *mut ULONG =
        *a.offset(0 as isize) as ULONG_PTR as *mut ULONG;
    let mut Access: ACCESS_MASK = *a.offset(1 as isize) as ACCESS_MASK;
    let mut Oa64: WOW64_OBJECT_ATTRIBUTES64_M = core::mem::zeroed();
    let mut Uname64: WOW64_UNICODE_STRING64_M = core::mem::zeroed();
    let mut Handle: HANDLE = core::ptr::null_mut();
    let mut St: NTSTATUS;

    Wow64pMarshalOa(
        core::ptr::addr_of_mut!(Oa64),
        core::ptr::addr_of_mut!(Uname64),
        *a.offset(2 as isize),
    );
    St = Fn.unwrap_unchecked()(
        core::ptr::addr_of_mut!(Handle),
        Access,
        core::ptr::addr_of_mut!(Oa64) as PVOID,
    );
    if St >= 0 && !Handle32.is_null() {
        *Handle32 = Handle as ULONG_PTR as ULONG;
    }
    return St;
}

unsafe extern "C" fn Wow64pPutHandle(Cell32: ULONG, Value: HANDLE) {
    let Cell: *mut ULONG = Cell32 as usize as *mut ULONG;
    if !Cell.is_null() {
        *Cell = Value as usize as ULONG;
    }
}

unsafe extern "C" fn Wow64pMarshalOaOpt(
    Oa64: *mut WOW64_OBJECT_ATTRIBUTES64_M,
    Uname64: *mut WOW64_UNICODE_STRING64_M,
    Oa32Ptr: ULONG,
) -> PVOID {
    if Oa32Ptr == 0 {
        return core::ptr::null_mut();
    }
    Wow64pMarshalOa(Oa64, Uname64, Oa32Ptr);
    return Oa64 as PVOID;
}

unsafe extern "C" fn Wow64Manual_NtOpenProcessToken(a: *mut ULONG) -> NTSTATUS {
    let mut Token: HANDLE = core::ptr::null_mut();
    let St: NTSTATUS = NtOpenProcessToken(
        (*a.add(0) as LONG as LONG_PTR) as HANDLE,
        *a.add(1) as ACCESS_MASK,
        core::ptr::addr_of_mut!(Token),
    );
    if St >= 0 {
        Wow64pPutHandle(*a.add(2), Token);
    }
    return St;
}

unsafe extern "C" fn Wow64Manual_NtOpenThreadToken(a: *mut ULONG) -> NTSTATUS {
    let mut Token: HANDLE = core::ptr::null_mut();
    let St: NTSTATUS = NtOpenThreadToken(
        (*a.add(0 as usize) as LONG as LONG_PTR) as HANDLE,
        *a.add(1 as usize) as ACCESS_MASK,
        *a.add(2 as usize) as BOOLEAN,
        &raw mut Token,
    );
    if St >= 0 {
        Wow64pPutHandle(*a.add(3 as usize), Token);
    }
    return St;
}

unsafe extern "C" fn Wow64Manual_NtGetNextProcess(a: *mut ULONG) -> NTSTATUS {
    let mut Next: HANDLE = core::ptr::null_mut();
    let mut St: NTSTATUS = NtGetNextProcess(
        *a.add(0) as LONG as LONG_PTR as HANDLE,
        *a.add(1) as ACCESS_MASK,
        *a.add(2),
        *a.add(3),
        core::ptr::addr_of_mut!(Next),
    );
    if St >= 0 {
        Wow64pPutHandle(*a.add(4), Next);
    }
    return St;
}

unsafe extern "C" fn Wow64Manual_NtGetNextThread(a: *mut ULONG) -> NTSTATUS {
    let mut Next: HANDLE = core::ptr::null_mut();
    let mut St: NTSTATUS = NtGetNextThread(
        *a.offset(0 as isize) as LONG as LONG_PTR as HANDLE,
        *a.offset(1 as isize) as LONG as LONG_PTR as HANDLE,
        *a.offset(2 as isize) as ACCESS_MASK,
        *a.offset(3 as isize),
        *a.offset(4 as isize),
        core::ptr::addr_of_mut!(Next),
    );
    if St >= 0 as NTSTATUS {
        Wow64pPutHandle(*a.offset(5 as isize), Next);
    }
    return St;
}

unsafe extern "C" fn Wow64Manual_NtDuplicateObject(a: *mut ULONG) -> NTSTATUS {
    let mut Target: HANDLE = core::ptr::null_mut();
    /* TargetHandle is _Out_opt_ — a DUPLICATE_CLOSE_SOURCE-only call passes
     * NULL, and that NULL has to reach the kernel as NULL. */
    let mut St: NTSTATUS = NtDuplicateObject(
        (*a.add(0)) as LONG as LONG_PTR as HANDLE,
        (*a.add(1)) as LONG as LONG_PTR as HANDLE,
        (*a.add(2)) as LONG as LONG_PTR as HANDLE,
        if *a.add(3) != 0 {
            core::ptr::addr_of_mut!(Target)
        } else {
            core::ptr::null_mut()
        },
        (*a.add(4)) as ACCESS_MASK,
        *a.add(5),
        *a.add(6),
    );
    if St >= 0 {
        Wow64pPutHandle(*a.add(3), Target);
    }
    return St;
}

unsafe extern "C" fn Wow64pOpenByOaU32(Fn: WOW64_OPEN_FN_U32, a: *mut ULONG) -> NTSTATUS {
    let mut Oa64: WOW64_OBJECT_ATTRIBUTES64_M = core::mem::zeroed();
    let mut Uname64: WOW64_UNICODE_STRING64_M = core::mem::zeroed();
    let mut Handle: HANDLE = core::ptr::null_mut();
    let mut St: NTSTATUS = (Fn.unwrap_unchecked())(
        core::ptr::addr_of_mut!(Handle),
        *a.add(1 as usize),
        Wow64pMarshalOaOpt(
            core::ptr::addr_of_mut!(Oa64),
            core::ptr::addr_of_mut!(Uname64),
            *a.add(2 as usize),
        ),
        *a.add(3 as usize),
    );
    if St >= 0 as NTSTATUS {
        Wow64pPutHandle(*a.add(0 as usize), Handle);
    }
    return St;
}

unsafe extern "C" fn Wow64pCreatePort(mut Fn: WOW64_PORT_FN, mut a: *mut ULONG) -> NTSTATUS {
    let mut Oa64: WOW64_OBJECT_ATTRIBUTES64_M = core::mem::zeroed();
    let mut Uname64: WOW64_UNICODE_STRING64_M = core::mem::zeroed();
    let mut Handle: HANDLE = core::ptr::null_mut();
    let mut St: NTSTATUS = Fn.unwrap_unchecked()(
        core::ptr::addr_of_mut!(Handle),
        Wow64pMarshalOaOpt(
            core::ptr::addr_of_mut!(Oa64),
            core::ptr::addr_of_mut!(Uname64),
            *a.offset(1 as isize),
        ),
        *a.offset(2 as isize),
        *a.offset(3 as isize),
        *a.offset(4 as isize),
    );
    if St >= 0 {
        Wow64pPutHandle(*a.offset(0 as isize), Handle);
    }
    return St;
}

unsafe extern "C" fn Wow64Manual_NtDuplicateToken(a: *mut ULONG) -> NTSTATUS {
    let mut Oa64: WOW64_OBJECT_ATTRIBUTES64_M = core::mem::zeroed();
    let mut Uname64: WOW64_UNICODE_STRING64_M = core::mem::zeroed();
    let mut NewToken: HANDLE = core::ptr::null_mut();
    let mut St: NTSTATUS = NtDuplicateToken(
        *a.offset(0 as isize) as LONG as LONG_PTR as HANDLE,
        *a.offset(1 as isize) as ACCESS_MASK,
        (Wow64pMarshalOaOpt(
            core::ptr::addr_of_mut!(Oa64),
            core::ptr::addr_of_mut!(Uname64),
            *a.offset(2 as isize),
        )).cast(),
        *a.offset(3 as isize) as BOOLEAN,
        *a.offset(4 as isize),
        core::ptr::addr_of_mut!(NewToken),
    );
    if St >= 0 as NTSTATUS {
        Wow64pPutHandle(*a.offset(5 as isize), NewToken);
    }
    return St;
}

unsafe extern "C" fn Wow64Manual_NtCreateProcess(a: *mut ULONG) -> NTSTATUS {
    let mut Oa64: WOW64_OBJECT_ATTRIBUTES64_M = core::mem::zeroed();
    let mut Uname64: WOW64_UNICODE_STRING64_M = core::mem::zeroed();
    let mut Process: HANDLE = core::ptr::null_mut();
    let mut St: NTSTATUS = NtCreateProcess(
        core::ptr::addr_of_mut!(Process),
        *a.offset(1 as isize) as ACCESS_MASK,
        (Wow64pMarshalOaOpt(
            core::ptr::addr_of_mut!(Oa64),
            core::ptr::addr_of_mut!(Uname64),
            *a.offset(2 as isize),
        )).cast(),
        (*a.offset(3 as isize) as LONG) as LONG_PTR as HANDLE,
        *a.offset(4 as isize) as BOOLEAN,
        (*a.offset(5 as isize) as LONG) as LONG_PTR as HANDLE,
        (*a.offset(6 as isize) as LONG) as LONG_PTR as HANDLE,
        (*a.offset(7 as isize) as LONG) as LONG_PTR as HANDLE,
    );
    if St >= 0 as NTSTATUS {
        Wow64pPutHandle(*a.offset(0 as isize), Process);
    }
    return St;
}

unsafe extern "C" fn Wow64pPublishRemotePeb32(Process: HANDLE) {
    let mut Teb: PBYTE = NtCurrentTeb() as PBYTE;
    let mut WowTebOffset: LONG = *(Teb.offset(WOW64_TEB_WOWTEBOFFSET as isize) as *mut LONG);
    let mut Peb32: PVOID = core::ptr::null_mut();
    if WowTebOffset == 0 {
        return;
    }
    if NtQueryInformationProcess(
        Process,
        ProcessWow64Information as ULONG,
        core::ptr::addr_of_mut!(Peb32) as PVOID,
        ::core::mem::size_of::<PVOID>() as ULONG,
        core::ptr::null_mut::<ULONG>(),
    ) < 0 as NTSTATUS
    {
        return;
    }
    *(Teb
        .offset(WowTebOffset as isize)
        .offset(WOW64_TEB32_ARBUSERPTR as isize) as *mut ULONG) = Peb32 as ULONG_PTR as ULONG;
}

unsafe extern "C" fn Wow64Manual_NtCreateProcessEx(a: *mut ULONG) -> NTSTATUS {
    let mut Oa64: WOW64_OBJECT_ATTRIBUTES64_M = core::mem::zeroed();
    let mut Uname64: WOW64_UNICODE_STRING64_M = core::mem::zeroed();
    let mut Process: HANDLE = core::ptr::null_mut();
    let mut St: NTSTATUS = NtCreateProcessEx(
        core::ptr::addr_of_mut!(Process),
        *a.offset(1 as isize) as ACCESS_MASK,
        (Wow64pMarshalOaOpt(
            core::ptr::addr_of_mut!(Oa64),
            core::ptr::addr_of_mut!(Uname64),
            *a.offset(2 as isize),
        )).cast(),
        *a.offset(3 as isize) as LONG as LONG_PTR as HANDLE,
        *a.offset(4 as isize),
        *a.offset(5 as isize) as LONG as LONG_PTR as HANDLE,
        *a.offset(6 as isize) as LONG as LONG_PTR as HANDLE,
        *a.offset(7 as isize) as LONG as LONG_PTR as HANDLE,
        *a.offset(8 as isize) as BOOLEAN,
    );
    if St >= 0 as NTSTATUS {
        Wow64pPutHandle(*a.offset(0 as isize), Process);
        Wow64pPublishRemotePeb32(Process);
    }
    return St;
}

unsafe extern "C" fn Wow64Manual_NtCreateProfile(a: *mut ULONG) -> NTSTATUS {
    let mut Profile: HANDLE = core::ptr::null_mut();
    let mut St: NTSTATUS = NtCreateProfile(
        core::ptr::addr_of_mut!(Profile),
        (*a.add(1 as usize)) as LONG as LONG_PTR as HANDLE,
        (*a.add(2 as usize)) as ULONG_PTR as PVOID,
        (*a.add(3 as usize)) as SIZE_T,
        *a.add(4 as usize),
        (*a.add(5 as usize)) as ULONG_PTR as PVOID,
        *a.add(6 as usize),
        *a.add(7 as usize),
        (*a.add(8 as usize)) as ULONG_PTR,
    );
    if St >= 0 as NTSTATUS {
        Wow64pPutHandle(*a.add(0 as usize), Profile);
    }
    return St;
}

unsafe extern "C" fn Wow64pLockUnlockVm(Fn: WOW64_VMLOCK_FN, a: *mut ULONG) -> NTSTATUS {
    let Base32: *mut ULONG = *a.offset(1) as ULONG_PTR as *mut ULONG;
    let Size32: *mut ULONG = *a.offset(2) as ULONG_PTR as *mut ULONG;
    let mut Base: PVOID = if !Base32.is_null() {
        *Base32 as ULONG_PTR as PVOID
    } else {
        core::ptr::null_mut()
    };
    let mut Size: SIZE_T = (if !Size32.is_null() {
        *Size32
    } else {
        0 as ULONG
    }) as SIZE_T;
    let St: NTSTATUS = (Fn.unwrap_unchecked())(
        *a.offset(0) as LONG as LONG_PTR as HANDLE,
        &raw mut Base,
        &raw mut Size,
        *a.offset(3),
    );
    if St >= 0 {
        if !Base32.is_null() {
            *Base32 = Base as ULONG_PTR as ULONG;
        }
        if !Size32.is_null() {
            *Size32 = Size as ULONG;
        }
    }
    return St;
}

unsafe extern "C" fn Wow64pMarshalUstr(U64: *mut WOW64_UNICODE_STRING64_M, U32Ptr: ULONG) -> PVOID {
    let U32: *mut WOW64_UNICODE_STRING32_M = U32Ptr as ULONG_PTR as *mut WOW64_UNICODE_STRING32_M;
    if U32.is_null() {
        return core::ptr::null_mut();
    }
    (*U64).Length = (*U32).Length;
    (*U64).MaximumLength = (*U32).MaximumLength;
    (*U64).__pad = 0 as ULONG;
    (*U64).Buffer = (*U32).Buffer as ULONG_PTR as ULONGLONG;
    return U64 as PVOID;
}

unsafe extern "C" fn Wow64Manual_NtSetValueKey(a: *mut ULONG) -> NTSTATUS {
    let mut Name: WOW64_UNICODE_STRING64_M = core::mem::zeroed();
    return NtSetValueKey(
        (*a.offset(0 as isize) as LONG as LONG_PTR as HANDLE),
        (Wow64pMarshalUstr(core::ptr::addr_of_mut!(Name), *a.offset(1 as isize))).cast(),
        *a.offset(2 as isize),
        *a.offset(3 as isize),
        (*a.offset(4 as isize) as ULONG_PTR as PVOID),
        *a.offset(5 as isize),
    );
}

unsafe extern "C" fn Wow64Manual_NtDeleteValueKey(a: *mut ULONG) -> NTSTATUS {
    let mut Name: WOW64_UNICODE_STRING64_M = core::mem::zeroed();
    return NtDeleteValueKey(
        (*a.offset(0 as isize)) as LONG as LONG_PTR as HANDLE,
        (Wow64pMarshalUstr(
            core::ptr::addr_of_mut!(Name),
            *a.offset(1 as isize),
        )).cast(),
    );
}

unsafe extern "C" fn Wow64Manual_NtRenameKey(a: *mut ULONG) -> NTSTATUS {
    let mut Name: WOW64_UNICODE_STRING64_M = core::mem::zeroed();
    return NtRenameKey(
        (*a.add(0) as LONG as LONG_PTR) as HANDLE,
        (Wow64pMarshalUstr(core::ptr::addr_of_mut!(Name), *a.add(1))).cast(),
    );
}

unsafe extern "C" fn Wow64pCallUstr1(Fn: WOW64_USTR1_FN, a: *mut ULONG) -> NTSTATUS {
    let mut S: WOW64_UNICODE_STRING64_M = WOW64_UNICODE_STRING64_M {
        Length: 0,
        MaximumLength: 0,
        __pad: 0,
        Buffer: 0,
    };
    return (Fn.unwrap_unchecked())(Wow64pMarshalUstr(
        core::ptr::addr_of_mut!(S),
        *a.add(0 as usize),
    ));
}

unsafe extern "C" fn Wow64pCallOa1(Fn: WOW64_OA1_FN, a: *mut ULONG) -> NTSTATUS {
    let mut Oa: WOW64_OBJECT_ATTRIBUTES64_M = core::mem::zeroed();
    let mut Name: WOW64_UNICODE_STRING64_M = core::mem::zeroed();
    return (Fn.unwrap_unchecked())(Wow64pMarshalOaOpt(
        core::ptr::addr_of_mut!(Oa),
        core::ptr::addr_of_mut!(Name),
        *a.offset(0 as isize),
    ));
}

unsafe extern "C" fn Wow64Manual_NtQuerySystemEnvironmentValue(a: *mut ULONG) -> NTSTATUS {
    let mut Name: WOW64_UNICODE_STRING64_M = WOW64_UNICODE_STRING64_M {
        Length: 0,
        MaximumLength: 0,
        __pad: 0,
        Buffer: 0,
    };
    return NtQuerySystemEnvironmentValue(
        (Wow64pMarshalUstr(core::ptr::addr_of_mut!(Name), *a.offset(0 as isize))).cast(),
        (*a.offset(1 as isize) as ULONG_PTR as PVOID).cast(),
        *a.offset(2 as isize),
        *a.offset(3 as isize) as ULONG_PTR as PULONG,
    );
}

unsafe extern "C" fn Wow64Manual_NtSetSystemEnvironmentValue(a: *mut ULONG) -> NTSTATUS {
    let mut Name: WOW64_UNICODE_STRING64_M = WOW64_UNICODE_STRING64_M {
        Length: 0,
        MaximumLength: 0,
        __pad: 0,
        Buffer: 0,
    };
    let mut Value: WOW64_UNICODE_STRING64_M = WOW64_UNICODE_STRING64_M {
        Length: 0,
        MaximumLength: 0,
        __pad: 0,
        Buffer: 0,
    };
    return NtSetSystemEnvironmentValue(
        (Wow64pMarshalUstr(core::ptr::addr_of_mut!(Name), *a.offset(0 as isize))).cast(),
        (Wow64pMarshalUstr(core::ptr::addr_of_mut!(Value), *a.offset(1 as isize))).cast(),
    );
}

unsafe extern "C" fn Wow64Manual_NtQuerySystemEnvironmentValueEx(a: *mut ULONG) -> NTSTATUS {
    let mut Name: WOW64_UNICODE_STRING64_M = WOW64_UNICODE_STRING64_M {
        Length: 0,
        MaximumLength: 0,
        __pad: 0,
        Buffer: 0,
    };
    return NtQuerySystemEnvironmentValueEx(
        (Wow64pMarshalUstr(
            core::ptr::addr_of_mut!(Name),
            *a.offset(0 as isize),
        )).cast(),
        (*a.offset(1 as isize) as ULONG_PTR as PVOID).cast(),
        *a.offset(2 as isize) as ULONG_PTR as PVOID,
        *a.offset(3 as isize) as ULONG_PTR as PULONG,
        *a.offset(4 as isize) as ULONG_PTR as PULONG,
    );
}

unsafe extern "C" fn Wow64Manual_NtSetSystemEnvironmentValueEx(a: *mut ULONG) -> NTSTATUS {
    let mut Name: WOW64_UNICODE_STRING64_M = core::mem::zeroed();
    return NtSetSystemEnvironmentValueEx(
        (Wow64pMarshalUstr(
            core::ptr::addr_of_mut!(Name),
            *a.offset(0 as isize),
        )).cast(),
        (*a.offset(1 as isize) as ULONG_PTR as PVOID).cast(),
        *a.offset(2 as isize) as ULONG_PTR as PVOID,
        *a.offset(3 as isize),
        *a.offset(4 as isize),
    );
}

unsafe extern "C" fn Wow64Manual_NtCreatePagingFile(a: *mut ULONG) -> NTSTATUS {
    let mut Name: WOW64_UNICODE_STRING64_M = WOW64_UNICODE_STRING64_M {
        Length: 0,
        MaximumLength: 0,
        __pad: 0,
        Buffer: 0,
    };
    return NtCreatePagingFile(
        (Wow64pMarshalUstr(
            core::ptr::addr_of_mut!(Name),
            *a.offset(0 as isize),
        )).cast(),
        (*a.offset(1 as isize) as ULONG_PTR as PVOID).cast(),
        (*a.offset(2 as isize) as ULONG_PTR as PVOID).cast(),
        *a.offset(3 as isize),
    );
}

unsafe extern "C" fn Wow64Manual_NtLoadKey(a: *mut ULONG) -> NTSTATUS {
    let mut KeyOa: WOW64_OBJECT_ATTRIBUTES64_M = core::mem::zeroed();
    let mut FileOa: WOW64_OBJECT_ATTRIBUTES64_M = core::mem::zeroed();
    let mut KeyName: WOW64_UNICODE_STRING64_M = core::mem::zeroed();
    let mut FileName: WOW64_UNICODE_STRING64_M = core::mem::zeroed();
    return NtLoadKey(
        (Wow64pMarshalOaOpt(
            core::ptr::addr_of_mut!(KeyOa),
            core::ptr::addr_of_mut!(KeyName),
            *a.add(0 as usize),
        )).cast(),
        (Wow64pMarshalOaOpt(
            core::ptr::addr_of_mut!(FileOa),
            core::ptr::addr_of_mut!(FileName),
            *a.add(1 as usize),
        )).cast(),
    );
}

unsafe extern "C" fn Wow64Manual_NtLoadKey2(a: *mut ULONG) -> NTSTATUS {
    let mut KeyOa: WOW64_OBJECT_ATTRIBUTES64_M = core::mem::zeroed();
    let mut FileOa: WOW64_OBJECT_ATTRIBUTES64_M = core::mem::zeroed();
    let mut KeyName: WOW64_UNICODE_STRING64_M = core::mem::zeroed();
    let mut FileName: WOW64_UNICODE_STRING64_M = core::mem::zeroed();
    return NtLoadKey2(
        (Wow64pMarshalOaOpt(
            core::ptr::addr_of_mut!(KeyOa),
            core::ptr::addr_of_mut!(KeyName),
            *a.offset(0 as isize),
        )).cast(),
        (Wow64pMarshalOaOpt(
            core::ptr::addr_of_mut!(FileOa),
            core::ptr::addr_of_mut!(FileName),
            *a.offset(1 as isize),
        )).cast(),
        *a.offset(2 as isize),
    );
}

unsafe extern "C" fn Wow64Manual_NtLoadKeyEx(a: *mut ULONG) -> NTSTATUS {
    let mut KeyOa: WOW64_OBJECT_ATTRIBUTES64_M = core::mem::zeroed();
    let mut FileOa: WOW64_OBJECT_ATTRIBUTES64_M = core::mem::zeroed();
    let mut KeyName: WOW64_UNICODE_STRING64_M = core::mem::zeroed();
    let mut FileName: WOW64_UNICODE_STRING64_M = core::mem::zeroed();
    return NtLoadKeyEx(
        (Wow64pMarshalOaOpt(
            core::ptr::addr_of_mut!(KeyOa),
            core::ptr::addr_of_mut!(KeyName),
            *a.add(0 as usize),
        )).cast(),
        (Wow64pMarshalOaOpt(
            core::ptr::addr_of_mut!(FileOa),
            core::ptr::addr_of_mut!(FileName),
            *a.add(1 as usize),
        )).cast(),
        *a.add(2 as usize),
        (*a.add(3 as usize) as LONG) as LONG_PTR as HANDLE,
    );
}

unsafe extern "C" fn Wow64Manual_NtReplaceKey(a: *mut ULONG) -> NTSTATUS {
    let mut NewOa: WOW64_OBJECT_ATTRIBUTES64_M = core::mem::zeroed();
    let mut OldOa: WOW64_OBJECT_ATTRIBUTES64_M = core::mem::zeroed();
    let mut NewName: WOW64_UNICODE_STRING64_M = core::mem::zeroed();
    let mut OldName: WOW64_UNICODE_STRING64_M = core::mem::zeroed();
    return NtReplaceKey(
        (Wow64pMarshalOaOpt(
            core::ptr::addr_of_mut!(NewOa),
            core::ptr::addr_of_mut!(NewName),
            *a.offset(0 as isize),
        )).cast(),
        *a.offset(1 as isize) as LONG as LONG_PTR as HANDLE,
        (Wow64pMarshalOaOpt(
            core::ptr::addr_of_mut!(OldOa),
            core::ptr::addr_of_mut!(OldName),
            *a.offset(2 as isize),
        )).cast(),
    );
}

unsafe extern "C" fn Wow64Manual_NtUnloadKey2(a: *mut ULONG) -> NTSTATUS {
    let mut Oa: WOW64_OBJECT_ATTRIBUTES64_M = core::mem::zeroed();
    let mut Name: WOW64_UNICODE_STRING64_M = core::mem::zeroed();
    return NtUnloadKey2(
        (Wow64pMarshalOaOpt(
            core::ptr::addr_of_mut!(Oa),
            core::ptr::addr_of_mut!(Name),
            *a.add(0 as usize),
        )).cast(),
        *a.add(1 as usize),
    );
}

unsafe extern "C" fn Wow64Manual_NtUnloadKeyEx(a: *mut ULONG) -> NTSTATUS {
    let mut Oa: WOW64_OBJECT_ATTRIBUTES64_M = core::mem::zeroed();
    let mut Name: WOW64_UNICODE_STRING64_M = core::mem::zeroed();
    return NtUnloadKeyEx(
        (Wow64pMarshalOaOpt(
            &raw mut Oa,
            &raw mut Name,
            *a.offset(0 as isize),
        )).cast(),
        *a.offset(1 as isize) as LONG as LONG_PTR as HANDLE,
    );
}

unsafe extern "C" fn Wow64Manual_NtQueryOpenSubKeys(a: *mut ULONG) -> NTSTATUS {
    let mut Oa: WOW64_OBJECT_ATTRIBUTES64_M = core::mem::zeroed();
    let mut Name: WOW64_UNICODE_STRING64_M = core::mem::zeroed();
    return NtQueryOpenSubKeys(
        (Wow64pMarshalOaOpt(
            core::ptr::addr_of_mut!(Oa),
            core::ptr::addr_of_mut!(Name),
            *a.add(0),
        )).cast(),
        *a.add(1) as ULONG_PTR as PULONG,
    );
}

unsafe extern "C" fn Wow64Manual_NtQueryOpenSubKeysEx(a: *mut ULONG) -> NTSTATUS {
    let mut Oa: WOW64_OBJECT_ATTRIBUTES64_M = core::mem::zeroed();
    let mut Name: WOW64_UNICODE_STRING64_M = core::mem::zeroed();
    return NtQueryOpenSubKeysEx(
        (Wow64pMarshalOaOpt(
            core::ptr::addr_of_mut!(Oa),
            core::ptr::addr_of_mut!(Name),
            *a.offset(0 as isize),
        )).cast(),
        *a.offset(1 as isize),
        *a.offset(2 as isize) as ULONG_PTR as PVOID,
        *a.offset(3 as isize) as ULONG_PTR as PULONG,
    );
}

unsafe extern "C" fn Wow64Manual_NtAccessCheckAndAuditAlarm(a: *mut ULONG) -> NTSTATUS {
    let mut Sub: WOW64_UNICODE_STRING64_M = core::mem::zeroed();
    let mut TypeName: WOW64_UNICODE_STRING64_M = core::mem::zeroed();
    let mut ObjName: WOW64_UNICODE_STRING64_M = core::mem::zeroed();
    return NtAccessCheckAndAuditAlarm(
        (Wow64pMarshalUstr(
            core::ptr::addr_of_mut!(Sub),
            *a.offset(0 as isize),
        )).cast(),
        *a.offset(1 as isize) as ULONG_PTR as PVOID,
        (Wow64pMarshalUstr(
            core::ptr::addr_of_mut!(TypeName),
            *a.offset(2 as isize),
        )).cast(),
        (Wow64pMarshalUstr(
            core::ptr::addr_of_mut!(ObjName),
            *a.offset(3 as isize),
        )).cast(),
        *a.offset(4 as isize) as ULONG_PTR as PVOID,
        *a.offset(5 as isize) as ACCESS_MASK,
        (*a.offset(6 as isize) as ULONG_PTR as PVOID).cast(),
        *a.offset(7 as isize) as BOOLEAN,
        (*a.offset(8 as isize) as ULONG_PTR as PVOID).cast(),
        (*a.offset(9 as isize) as ULONG_PTR as PVOID).cast(),
        (*a.offset(10 as isize) as ULONG_PTR as PVOID).cast(),
    );
}

unsafe extern "C" fn Wow64Manual_NtOpenObjectAuditAlarm(a: *mut ULONG) -> NTSTATUS {
    let mut Sub: WOW64_UNICODE_STRING64_M = WOW64_UNICODE_STRING64_M {
        Length: 0,
        MaximumLength: 0,
        __pad: 0,
        Buffer: 0,
    };
    let mut TypeName: WOW64_UNICODE_STRING64_M = WOW64_UNICODE_STRING64_M {
        Length: 0,
        MaximumLength: 0,
        __pad: 0,
        Buffer: 0,
    };
    let mut ObjName: WOW64_UNICODE_STRING64_M = WOW64_UNICODE_STRING64_M {
        Length: 0,
        MaximumLength: 0,
        __pad: 0,
        Buffer: 0,
    };
    return NtOpenObjectAuditAlarm(
        (Wow64pMarshalUstr(
            core::ptr::addr_of_mut!(Sub),
            *a.offset(0 as isize),
        )).cast(),
        *a.offset(1 as isize) as ULONG_PTR as PVOID,
        (Wow64pMarshalUstr(
            core::ptr::addr_of_mut!(TypeName),
            *a.offset(2 as isize),
        )).cast(),
        (Wow64pMarshalUstr(
            core::ptr::addr_of_mut!(ObjName),
            *a.offset(3 as isize),
        )).cast(),
        *a.offset(4 as isize) as ULONG_PTR as PVOID,
        *a.offset(5 as isize) as LONG as LONG_PTR as HANDLE,
        *a.offset(6 as isize) as ACCESS_MASK,
        *a.offset(7 as isize) as ACCESS_MASK,
        (*a.offset(8 as isize) as ULONG_PTR as PVOID).cast(),
        *a.offset(9 as isize) as BOOLEAN,
        *a.offset(10 as isize) as BOOLEAN,
        (*a.offset(11 as isize) as ULONG_PTR as PVOID).cast(),
    );
}

unsafe extern "C" fn Wow64pAuditClose(Fn: WOW64_AUDIT_CLOSE_FN, a: *mut ULONG) -> NTSTATUS {
    let mut Sub: WOW64_UNICODE_STRING64_M = core::mem::zeroed();
    return (Fn.unwrap_unchecked())(
        Wow64pMarshalUstr(core::ptr::addr_of_mut!(Sub), *a.add(0 as usize)),
        *a.add(1 as usize) as ULONG_PTR as PVOID,
        *a.add(2 as usize) as BOOLEAN,
    );
}

unsafe extern "C" fn Wow64Manual_NtPrivilegeObjectAuditAlarm(a: *mut ULONG) -> NTSTATUS {
    let mut Sub: WOW64_UNICODE_STRING64_M = core::mem::zeroed();
    return NtPrivilegeObjectAuditAlarm(
        (Wow64pMarshalUstr(core::ptr::addr_of_mut!(Sub), *a.add(0))).cast(),
        *a.add(1) as ULONG_PTR as PVOID,
        *a.add(2) as LONG as LONG_PTR as HANDLE,
        *a.add(3) as ACCESS_MASK,
        (*a.add(4) as ULONG_PTR as PVOID).cast(),
        *a.add(5) as BOOLEAN,
    );
}

unsafe extern "C" fn Wow64Manual_NtPrivilegedServiceAuditAlarm(a: *mut ULONG) -> NTSTATUS {
    let mut Sub: WOW64_UNICODE_STRING64_M = core::mem::zeroed();
    let mut Svc: WOW64_UNICODE_STRING64_M = core::mem::zeroed();
    return NtPrivilegedServiceAuditAlarm(
        (Wow64pMarshalUstr(core::ptr::addr_of_mut!(Sub), *a.add(0 as usize))).cast(),
        (Wow64pMarshalUstr(core::ptr::addr_of_mut!(Svc), *a.add(1 as usize))).cast(),
        *a.add(2 as usize) as LONG as LONG_PTR as HANDLE,
        (*a.add(3 as usize) as ULONG_PTR as PVOID).cast(),
        *a.add(4 as usize) as BOOLEAN,
    );
}

unsafe extern "C" fn Wow64pRedirectPath(
    mut Name64: *mut WOW64_UNICODE_STRING64_M,
    mut Scratch: *mut WCHAR,
) -> BOOLEAN {
    static Sys32: [u16; 11] = [
    0x005Cu16, 0x0073, 0x0079, 0x0073, 0x0074, 0x0065, 0x006D, 0x0033, 0x0032, 0x005C,
    0x0000,
];
    static Wow64: [u16; 11] = [
    0x005Cu16, 0x0073, 0x0079, 0x0073, 0x0077, 0x006F, 0x0077, 0x0036, 0x0034, 0x005C,
    0x0000,
];
    let mut Src: *const WCHAR = (*Name64).Buffer as *const WCHAR;
    let mut Chars: ULONG =
        ((*Name64).Length as usize).wrapping_div(::core::mem::size_of::<WCHAR>() as usize) as ULONG;
    let mut i: ULONG = 0;
    let mut j: ULONG = 0;
    let mut Hit: LONG = -1;

    if Src.is_null() || Chars == 0 {
        return FALSE as BOOLEAN;
    }
    if Chars > WOW64_REDIR_MAX {
        DbgPrint(
            b"[WOW64]   REDIRECT SKIPPED (path %lu chars > %u): '%.*S'\n\0".as_ptr() as PCSTR,
            Chars,
            WOW64_REDIR_MAX as ::core::ffi::c_uint,
            Chars as ::core::ffi::c_int,
            Src,
        );
        return FALSE as BOOLEAN;
    }

    i = 0;
    while i.wrapping_add(10) <= Chars {
        j = 0;
        while j < 10 {
            let mut c: WCHAR = *Src.offset(i.wrapping_add(j) as isize);
            if c as ::core::ffi::c_int >= 'A' as i32 && c as ::core::ffi::c_int <= 'Z' as i32 {
                c = (c as ::core::ffi::c_int + 32) as WCHAR;
            }
            if c as ::core::ffi::c_int != Sys32[j as usize] as ::core::ffi::c_int {
                break;
            }
            j = j.wrapping_add(1);
        }
        if j == 10 {
            Hit = i as LONG;
            break;
        } else {
            i = i.wrapping_add(1);
        }
    }
    if Hit < 0 {
        return FALSE as BOOLEAN;
    }

    i = 0;
    while i < Chars {
        *Scratch.offset(i as isize) = *Src.offset(i as isize);
        i = i.wrapping_add(1);
    }
    j = 0;
    while j < 10 {
        *Scratch.offset((Hit as ULONG).wrapping_add(j) as isize) = Wow64[j as usize];
        j = j.wrapping_add(1);
    }
    (*Name64).Buffer = Scratch as ULONG_PTR as ULONGLONG;
    return TRUE as BOOLEAN;
}

unsafe extern "C" fn Wow64pCopyIosbDown(I64: *mut WOW64_IO_STATUS_BLOCK64_M, Iosb32Ptr: ULONG) {
    let mut I32: *mut WOW64_IO_STATUS_BLOCK32_M =
        Iosb32Ptr as ULONG_PTR as *mut WOW64_IO_STATUS_BLOCK32_M;
    if I32.is_null() {
        return;
    }
    (*I32).Status = (*I64).Status as ULONG;
    (*I32).Information = (*I64).Information as ULONG;
}

unsafe extern "C" fn Wow64Manual_NtFlushVirtualMemory(a: *mut ULONG) -> NTSTATUS {
    let Base32: *mut ULONG = *a.offset(1 as isize) as ULONG_PTR as *mut ULONG;
    let Size32: *mut ULONG = *a.offset(2 as isize) as ULONG_PTR as *mut ULONG;
    let mut Base: PVOID = if !Base32.is_null() {
        *Base32 as ULONG_PTR as PVOID
    } else {
        core::ptr::null_mut()
    };
    let mut Size: SIZE_T = if !Size32.is_null() {
        *Size32 as SIZE_T
    } else {
        0 as SIZE_T
    };
    let mut Iosb: WOW64_IO_STATUS_BLOCK64_M = WOW64_IO_STATUS_BLOCK64_M {
        Status: 0,
        Information: 0,
    };
    let mut St: NTSTATUS;
    Iosb.Status = 0 as ULONGLONG;
    Iosb.Information = 0 as ULONGLONG;
    St = NtFlushVirtualMemory(
        *a.offset(0 as isize) as LONG as LONG_PTR as HANDLE,
        core::ptr::addr_of_mut!(Base),
        core::ptr::addr_of_mut!(Size),
        (core::ptr::addr_of_mut!(Iosb) as PVOID).cast(),
    );
    Wow64pCopyIosbDown(
        core::ptr::addr_of_mut!(Iosb),
        *a.offset(3 as isize),
    );
    if St >= 0 as NTSTATUS {
        if !Base32.is_null() {
            *Base32 = Base as ULONG_PTR as ULONG;
        }
        if !Size32.is_null() {
            *Size32 = Size as ULONG;
        }
    }
    return St;
}

unsafe extern "C" fn Wow64pRefuseAsync(
    mut Event: ULONG,
    mut ApcRoutine: ULONG,
    mut Who: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    if Event == 0 && ApcRoutine == 0 {
        return 0;
    }
    DbgPrint(
        b"[WOW64]   %s: async request (Event=%08lx Apc=%08lx) is not supported under WoW64 yet (needs kernel-side IO_STATUS_BLOCK32) \xE2\x80\x94 refusing\n\0"
            as *const u8 as PCSTR,
        Who,
        Event,
        ApcRoutine,
    );
    return 1;
}

unsafe extern "C" fn Wow64pApcContext(Routine32: ULONG, Context32: ULONG) -> PVOID {
    return (((Routine32 as ULONG_PTR) << 32) | (Context32 as ULONG_PTR)) as PVOID;
}

unsafe extern "C" fn Wow64pQueueIoApc(St: NTSTATUS, Routine32: ULONG, Context32: ULONG, Iosb32: ULONG) -> NTSTATUS {
    let mut Q: NTSTATUS;
    if Routine32 == 0 || St < 0 {
        return St;
    }
    Q = NtQueueApcThread(
        (-(2 as LONG_PTR)) as HANDLE,
        // `(PVOID)Wow64ApcRoutine` in the C, against its own PVOID prototype.
        ::core::mem::transmute::<*const (), _>(Wow64ApcRoutine as *const ()),
        Wow64pApcContext(Routine32, Context32),
        Iosb32 as ULONG_PTR as PVOID,
        core::ptr::null_mut(),
    );
    if Q < 0 {
        DbgPrint(
            b"[WOW64]   completion APC %08lx could not be queued: %08lx\n\0" as *const u8 as PCSTR,
            Routine32,
            Q,
        );
    }
    return St;
}

unsafe extern "C" fn Wow64pFinishIosb(
    mut St: NTSTATUS,
    mut File: HANDLE,
    mut I64: *mut WOW64_IO_STATUS_BLOCK64_M,
    mut Iosb32Ptr: ULONG,
    mut Who: *const ::core::ffi::c_char,
) -> NTSTATUS {
    if St == WOW64_STATUS_PENDING {
        let mut Wait: NTSTATUS = 0;
        DbgPrint(
            b"[WOW64]   %s pended on an async handle \xE2\x80\x94 blocking on it (our host IOSB is stack-resident; we must not outlive it)\n\0"
                .as_ptr() as PCSTR,
            Who,
        );
        Wait = NtWaitForSingleObject(
            File,
            FALSE as BOOLEAN,
            ::core::ptr::null_mut::<LARGE_INTEGER>(),
        );
        if Wait < 0 as NTSTATUS {
            DbgPrint(
                b"[WOW64]   %s: wait failed %08lx \xE2\x80\x94 guest IOSB left untouched\n\0"
                    .as_ptr() as PCSTR,
                Who,
                Wait,
            );
            return Wait;
        }
        St = (*I64).Status as ULONG as NTSTATUS;
    }
    if St < 0 as NTSTATUS {
        (*I64).Status = St as ULONG as ULONGLONG;
    }
    Wow64pCopyIosbDown(I64, Iosb32Ptr);
    return St;
}

unsafe extern "C" fn Wow64pLatchIosb32(Iosb32Ptr: ULONG) {
    *((NtCurrentTeb() as *mut ::core::ffi::c_uchar).offset(WOW64_TEB_INSTRUMENTATION0 as isize)
        as *mut ULONG_PTR) = Iosb32Ptr as ULONG_PTR;
}

unsafe extern "C" fn Wow64pReadWriteFile(mut Fn: WOW64_RW_FN, mut a: *mut ULONG, mut Who: *const ::core::ffi::c_char) -> NTSTATUS {
    let mut File: HANDLE = *a.offset(0 as isize) as LONG as LONG_PTR as HANDLE;
    let mut Event: HANDLE = *a.offset(1 as isize) as LONG as LONG_PTR as HANDLE;
    let mut St: NTSTATUS = 0;
    Wow64pLatchIosb32(*a.offset(4 as isize));
    St = Fn.unwrap_unchecked()(
        File,
        Event,
        if *a.offset(2 as isize) != 0 {
            Wow64ApcRoutine as usize as PVOID
        } else {
            core::ptr::null_mut()
        },
        if *a.offset(2 as isize) != 0 {
            Wow64pApcContext(
                *a.offset(2 as isize),
                *a.offset(3 as isize),
            )
        } else {
            *a.offset(3 as isize) as ULONG_PTR as PVOID
        },
        *a.offset(4 as isize) as ULONG_PTR as PVOID,
        *a.offset(5 as isize) as ULONG_PTR as PVOID,
        *a.offset(6 as isize),
        *a.offset(7 as isize) as ULONG_PTR as PLARGE_INTEGER,
        *a.offset(8 as isize) as ULONG_PTR as PULONG,
    );
    Wow64pLatchIosb32(0 as ULONG);
    return St;
}

unsafe extern "C" fn Wow64Manual_NtReadFile(a: *mut ULONG) -> NTSTATUS {
    let Buffer: PVOID = *a.offset(5 as isize) as ULONG_PTR as PVOID;
    let Length: SIZE_T = *a.offset(6 as isize) as SIZE_T;
    let File: HANDLE = *a.offset(0 as isize) as LONG as LONG_PTR as HANDLE;
    let mut St: NTSTATUS;
    Wow64NotifyReadFileBefore(File, Buffer, Length);
    St = Wow64pReadWriteFile(
        Some(
            core::mem::transmute::<*const (), _>(NtReadFile as *const ()),
        ),
        a,
        b"NtReadFile\0".as_ptr() as *const i8,
    );
    Wow64NotifyReadFileAfter(File, Buffer, Length, St as LONG);
    return St;
}

unsafe extern "C" fn Wow64Manual_NtWriteFile(a: *mut ULONG) -> NTSTATUS {
    return Wow64pReadWriteFile(Some(core::mem::transmute::<*const (), _>(NtWriteFile as *const ())), a, b"NtWriteFile\0".as_ptr() as *const i8);
}

unsafe extern "C" fn Wow64Manual_NtQueryInformationFile(a: *mut ULONG) -> NTSTATUS {
    let File: HANDLE = *a.add(0 as usize) as LONG as LONG_PTR as HANDLE;
    let mut Iosb64: WOW64_IO_STATUS_BLOCK64_M = WOW64_IO_STATUS_BLOCK64_M {
        Status: 0 as ULONGLONG,
        Information: 0 as ULONGLONG,
    };
    let St: NTSTATUS = NtQueryInformationFile(
        File,
        (core::ptr::addr_of_mut!(Iosb64) as PVOID).cast(),
        *a.add(2 as usize) as ULONG_PTR as PVOID,
        *a.add(3 as usize),
        *a.add(4 as usize),
    );
    return Wow64pFinishIosb(
        St,
        File,
        core::ptr::addr_of_mut!(Iosb64),
        *a.add(1 as usize),
        b"NtQueryInformationFile\0".as_ptr() as *const i8,
    );
}

unsafe extern "C" fn Wow64Manual_NtSetInformationFile(a: *mut ULONG) -> NTSTATUS {
    let File: HANDLE =
        *a.offset(0 as isize) as LONG as LONG_PTR as HANDLE;
    let Class: ULONG = *a.offset(4 as isize);
    let mut Iosb64: WOW64_IO_STATUS_BLOCK64_M = WOW64_IO_STATUS_BLOCK64_M {
        Status: 0 as ULONGLONG,
        Information: 0,
    };
    let mut St: NTSTATUS;
    if Class == WOW64_FileRenameInformation as ULONG
        || Class == WOW64_FileLinkInformation as ULONG
        || Class == WOW64_FileCompletionInformation as ULONG
        || Class == WOW64_FileMoveClusterInformation as ULONG
    {
        DbgPrint(
            b"[WOW64]   NtSetInformationFile class=%lu embeds a HANDLE \xE2\x80\x94 no 32->64 marshaller yet, refusing\n\0"
                as *const u8 as PCSTR,
            Class,
        );
        return WOW64_STATUS_NOT_IMPLEMENTED;
    }

    St = NtSetInformationFile(
        File,
        (&raw mut Iosb64 as PVOID).cast(),
        *a.offset(2 as isize) as ULONG_PTR as PVOID,
        *a.offset(3 as isize),
        Class,
    );
    return Wow64pFinishIosb(
        St,
        File,
        &raw mut Iosb64,
        *a.offset(1 as isize),
        b"NtSetInformationFile\0" as *const u8 as *const ::core::ffi::c_char,
    );
}

unsafe extern "C" fn Wow64Manual_NtQueryVolumeInformationFile(a: *mut ULONG) -> NTSTATUS {
    let File: HANDLE = *a.add(0) as LONG as LONG_PTR as HANDLE;
    let mut Iosb64: WOW64_IO_STATUS_BLOCK64_M = WOW64_IO_STATUS_BLOCK64_M {
        Status: 0,
        Information: 0,
    };
    let St: NTSTATUS = NtQueryVolumeInformationFile(
        File,
        (core::ptr::addr_of_mut!(Iosb64) as PVOID).cast(),
        *a.add(2) as ULONG_PTR as PVOID,
        *a.add(3),
        *a.add(4),
    );
    return Wow64pFinishIosb(
        St,
        File,
        core::ptr::addr_of_mut!(Iosb64),
        *a.add(1),
        b"NtQueryVolumeInformationFile\0".as_ptr() as *const i8,
    );
}

unsafe extern "C" fn Wow64Manual_NtFlushBuffersFile(a: *mut ULONG) -> NTSTATUS {
    let File: HANDLE = *a.offset(0 as isize) as LONG as LONG_PTR as HANDLE;
    let mut Iosb64: WOW64_IO_STATUS_BLOCK64_M = WOW64_IO_STATUS_BLOCK64_M {
        Status: 0 as ULONGLONG,
        Information: 0 as ULONGLONG,
    };
    let St: NTSTATUS = NtFlushBuffersFile(File, (core::ptr::addr_of_mut!(Iosb64) as PVOID).cast());
    return Wow64pFinishIosb(
        St,
        File,
        core::ptr::addr_of_mut!(Iosb64),
        *a.offset(1 as isize),
        b"NtFlushBuffersFile\0".as_ptr() as *const i8,
    );
}

unsafe extern "C" fn Wow64Manual_NtQueryDirectoryFile(a: *mut ULONG) -> NTSTATUS {
    let File: HANDLE = *a.offset(0 as isize) as LONG as LONG_PTR as HANDLE;
    let mut Iosb64: WOW64_IO_STATUS_BLOCK64_M = WOW64_IO_STATUS_BLOCK64_M {
        Status: 0,
        Information: 0,
    };
    let mut Mask64: WOW64_UNICODE_STRING64_M = WOW64_UNICODE_STRING64_M {
        Length: 0,
        MaximumLength: 0,
        __pad: 0,
        Buffer: 0,
    };
    let mut Mask64Ptr: PVOID = core::ptr::null_mut();
    let mut St: NTSTATUS;

    if Wow64pRefuseAsync(
        *a.offset(1 as isize),
        *a.offset(2 as isize),
        b"NtQueryDirectoryFile\0".as_ptr() as *const ::core::ffi::c_char,
    ) != 0
    {
        return WOW64_STATUS_NOT_IMPLEMENTED;
    }

    if *a.offset(9 as isize) != 0 {
        let mut M32: *mut WOW64_UNICODE_STRING32_M =
            *a.offset(9 as isize) as ULONG_PTR as *mut WOW64_UNICODE_STRING32_M;
        Mask64.Length = (*M32).Length;
        Mask64.MaximumLength = (*M32).MaximumLength;
        Mask64.__pad = 0;
        Mask64.Buffer = (*M32).Buffer as ULONG_PTR as ULONGLONG;
        Mask64Ptr = core::ptr::addr_of_mut!(Mask64) as PVOID;
    }

    St = NtQueryDirectoryFile(
        File,
        core::ptr::null_mut(),
        None, // NULL ApcRoutine
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(Iosb64) as PIO_STATUS_BLOCK,
        *a.offset(5 as isize) as ULONG_PTR as PVOID,
        *a.offset(6 as isize),
        *a.offset(7 as isize) as FILE_INFORMATION_CLASS,
        *a.offset(8 as isize) as BOOLEAN,
        Mask64Ptr as PUNICODE_STRING,
        *a.offset(10 as isize) as BOOLEAN,
    );
    return Wow64pFinishIosb(
        St,
        File,
        core::ptr::addr_of_mut!(Iosb64),
        *a.offset(4 as isize),
        b"NtQueryDirectoryFile\0".as_ptr() as *const ::core::ffi::c_char,
    );
}

unsafe extern "C" fn Wow64Manual_NtOpenFile(a: *mut ULONG) -> NTSTATUS {
    let mut Handle32: *mut ULONG =
        *a.offset(0 as isize) as ULONG_PTR as *mut ULONG;
    let mut Access: ACCESS_MASK = *a.offset(1 as isize) as ACCESS_MASK;
    let mut Oa64: WOW64_OBJECT_ATTRIBUTES64_M = WOW64_OBJECT_ATTRIBUTES64_M {
        Length: 0,
        __pad0: 0,
        RootDirectory: 0,
        ObjectName: 0,
        Attributes: 0,
        __pad1: 0,
        SecurityDescriptor: 0,
        SecurityQualityOfService: 0,
    };
    let mut Uname64: WOW64_UNICODE_STRING64_M = WOW64_UNICODE_STRING64_M {
        Length: 0,
        MaximumLength: 0,
        __pad: 0,
        Buffer: 0,
    };
    let mut RedirScratch: [WCHAR; (WOW64_REDIR_MAX) as _] = [0; (WOW64_REDIR_MAX) as _];
    let mut Iosb64: WOW64_IO_STATUS_BLOCK64_M = WOW64_IO_STATUS_BLOCK64_M {
        Status: 0 as ULONGLONG,
        Information: 0,
    };
    let mut Handle: HANDLE = core::ptr::null_mut();
    let mut St: NTSTATUS = 0;
    let mut Redirected: BOOLEAN = 0;

    Wow64pMarshalOa(
        core::ptr::addr_of_mut!(Oa64),
        core::ptr::addr_of_mut!(Uname64),
        *a.offset(2 as isize),
    );
    Redirected = Wow64pRedirectPath(
        core::ptr::addr_of_mut!(Uname64),
        core::ptr::addr_of_mut!(RedirScratch) as *mut WCHAR,
    );
    St = NtOpenFile(
        core::ptr::addr_of_mut!(Handle),
        Access,
        (core::ptr::addr_of_mut!(Oa64) as PVOID).cast(),
        (core::ptr::addr_of_mut!(Iosb64) as PVOID).cast(),
        *a.offset(4 as isize),
        *a.offset(5 as isize),
    );
    /* Logged on SUCCESS too. Printing only failures means you can see every
     * path the process could not open and none of the ones it did — so a
     * successful open of the WRONG file (the 64-bit copy, because redirection
     * did not fire) is invisible, which is precisely the case this had to be
     * used to diagnose. */
    if Oa64.ObjectName != 0 {
        DbgPrint(
            b"[WOW64]   NtOpenFile%s '%.*S' => %08lx\n\0" as *const u8 as PCSTR,
            if Redirected != 0 {
                b" [redir]\0" as *const u8 as *const ::core::ffi::c_char
            } else {
                b"\0" as *const u8 as *const ::core::ffi::c_char
            },
            ((Uname64.Length as usize) / core::mem::size_of::<WCHAR>())
                as ::core::ffi::c_int,
            Uname64.Buffer as ULONG_PTR as PCWSTR,
            St,
        );
    }
    Wow64pCopyIosbDown(
        core::ptr::addr_of_mut!(Iosb64),
        *a.offset(3 as isize),
    );
    if St >= 0 && !Handle32.is_null() {
        *Handle32 = Handle as ULONG_PTR as ULONG;
    }
    return St;
}

unsafe extern "C" fn Wow64Manual_NtCreateFile(a: *mut ULONG) -> NTSTATUS {
    let mut Handle32: *mut ULONG =
        *a.add(0 as usize) as ULONG_PTR as *mut ULONG;
    let mut Access: ACCESS_MASK = *a.add(1 as usize) as ACCESS_MASK;
    let mut Oa64: WOW64_OBJECT_ATTRIBUTES64_M = core::mem::zeroed();
    let mut Uname64: WOW64_UNICODE_STRING64_M = core::mem::zeroed();
    let mut RedirScratch: [WCHAR; (WOW64_REDIR_MAX) as _] = [0; (WOW64_REDIR_MAX) as _];
    let mut Iosb64: WOW64_IO_STATUS_BLOCK64_M = core::mem::zeroed();
    let mut AllocSize: PLARGE_INTEGER =
        *a.add(4 as usize) as ULONG_PTR as PLARGE_INTEGER;
    let mut Handle: HANDLE = core::ptr::null_mut();
    let mut St: NTSTATUS = 0;

    Wow64pMarshalOa(
        core::ptr::addr_of_mut!(Oa64),
        core::ptr::addr_of_mut!(Uname64),
        *a.add(2 as usize),
    );
    Wow64pRedirectPath(
        core::ptr::addr_of_mut!(Uname64),
        core::ptr::addr_of_mut!(RedirScratch) as *mut WCHAR,
    );
    St = NtCreateFile(
        core::ptr::addr_of_mut!(Handle),
        Access,
        (core::ptr::addr_of_mut!(Oa64) as PVOID).cast(),
        (core::ptr::addr_of_mut!(Iosb64) as PVOID).cast(),
        AllocSize,
        *a.add(5 as usize),
        *a.add(6 as usize),
        *a.add(7 as usize),
        *a.add(8 as usize),
        *a.add(9 as usize) as ULONG_PTR as PVOID,
        *a.add(10 as usize),
    );
    Wow64pCopyIosbDown(
        core::ptr::addr_of_mut!(Iosb64),
        *a.add(3 as usize),
    );
    if St >= 0 as NTSTATUS && !Handle32.is_null() {
        *Handle32 = Handle as ULONG_PTR as ULONG;
    }
    return St;
}

unsafe extern "C" fn Wow64Manual_NtCreateSection(a: *mut ULONG) -> NTSTATUS {
    let mut Handle32: *mut ULONG = *a.offset(0 as isize) as ULONG_PTR as *mut ULONG;
    let mut Access: ACCESS_MASK = *a.offset(1 as isize) as ACCESS_MASK;
    let mut Oa64: WOW64_OBJECT_ATTRIBUTES64_M = WOW64_OBJECT_ATTRIBUTES64_M {
        Length: 0,
        __pad0: 0,
        RootDirectory: 0,
        ObjectName: 0,
        Attributes: 0,
        __pad1: 0,
        SecurityDescriptor: 0,
        SecurityQualityOfService: 0,
    };
    let mut Uname64: WOW64_UNICODE_STRING64_M = WOW64_UNICODE_STRING64_M {
        Length: 0,
        MaximumLength: 0,
        __pad: 0,
        Buffer: 0,
    };
    let mut Oa64Ptr: PVOID = core::ptr::null_mut();
    let mut MaxSize: PLARGE_INTEGER = *a.offset(3 as isize) as ULONG_PTR as PLARGE_INTEGER;
    let mut Handle: HANDLE = core::ptr::null_mut();
    let mut St: NTSTATUS = 0;

    if *a.offset(2 as isize) != 0 {
        Wow64pMarshalOa(
            core::ptr::addr_of_mut!(Oa64),
            core::ptr::addr_of_mut!(Uname64),
            *a.offset(2 as isize),
        );
        Oa64Ptr = core::ptr::addr_of_mut!(Oa64) as PVOID;
    }
    St = NtCreateSection(
        core::ptr::addr_of_mut!(Handle),
        Access,
        (Oa64Ptr).cast(),
        MaxSize,
        *a.offset(4 as isize),
        *a.offset(5 as isize),
        *a.offset(6 as isize) as LONG as LONG_PTR as HANDLE,
    );
    if St >= 0 && !Handle32.is_null() {
        *Handle32 = Handle as ULONG_PTR as ULONG;
    }
    return St;
}

unsafe extern "C" fn Wow64Manual_NtQueryAttributesFile(a: *mut ULONG) -> NTSTATUS {
    let mut Oa64: WOW64_OBJECT_ATTRIBUTES64_M = WOW64_OBJECT_ATTRIBUTES64_M {
        Length: 0,
        __pad0: 0,
        RootDirectory: 0,
        ObjectName: 0,
        Attributes: 0,
        __pad1: 0,
        SecurityDescriptor: 0,
        SecurityQualityOfService: 0,
    };
    let mut Uname64: WOW64_UNICODE_STRING64_M = WOW64_UNICODE_STRING64_M {
        Length: 0,
        MaximumLength: 0,
        __pad: 0,
        Buffer: 0,
    };
    let mut RedirScratch: [WCHAR; (WOW64_REDIR_MAX) as _] = [0; (WOW64_REDIR_MAX) as _];

    Wow64pMarshalOa(
        &raw mut Oa64,
        &raw mut Uname64,
        *a.offset(0 as isize),
    );
    Wow64pRedirectPath(&raw mut Uname64, &raw mut RedirScratch as *mut WCHAR);
    /* FILE_BASIC_INFORMATION: 4x LARGE_INTEGER + ULONG — width-identical */
    return NtQueryAttributesFile(
        (&raw mut Oa64 as PVOID).cast(),
        (*a.offset(1 as isize) as ULONG_PTR as PVOID).cast(),
    );
}

unsafe extern "C" fn Wow64Manual_NtQueryFullAttributesFile(a: *mut ULONG) -> NTSTATUS {
    let mut Oa64: WOW64_OBJECT_ATTRIBUTES64_M = core::mem::zeroed();
    let mut Uname64: WOW64_UNICODE_STRING64_M = core::mem::zeroed();
    let mut RedirScratch: [WCHAR; (WOW64_REDIR_MAX) as _] = [0; (WOW64_REDIR_MAX) as _];

    Wow64pMarshalOa(
        core::ptr::addr_of_mut!(Oa64),
        core::ptr::addr_of_mut!(Uname64),
        *a.offset(0 as isize),
    );
    Wow64pRedirectPath(
        core::ptr::addr_of_mut!(Uname64),
        core::ptr::addr_of_mut!(RedirScratch) as *mut WCHAR,
    );
    /* FILE_NETWORK_OPEN_INFORMATION: LARGE_INTEGERs + ULONGs — width-identical */
    return NtQueryFullAttributesFile(
        (core::ptr::addr_of_mut!(Oa64) as PVOID).cast(),
        (*a.offset(1 as isize) as ULONG_PTR as PVOID).cast(),
    );
}

unsafe extern "C" fn Wow64Manual_NtAllocateVirtualMemory(a: *mut ULONG) -> NTSTATUS {
    let Process: HANDLE = *a.offset(0 as isize) as LONG as LONG_PTR as HANDLE;
    let Base32: *mut ULONG = *a.offset(1 as isize) as ULONG_PTR as *mut ULONG;
    let mut ZeroBits: ULONG_PTR = *a.offset(2 as isize) as ULONG_PTR;
    let Size32: *mut ULONG = *a.offset(3 as isize) as ULONG_PTR as *mut ULONG;
    let Type: ULONG = *a.offset(4 as isize);
    let Protect: ULONG = *a.offset(5 as isize);
    let mut Base: PVOID = if !Base32.is_null() {
        *Base32 as ULONG_PTR as PVOID
    } else {
        core::ptr::null_mut()
    };
    let mut Size: SIZE_T = (if !Size32.is_null() {
        *Size32
    } else {
        0 as ULONG
    }) as SIZE_T;
    let mut St: NTSTATUS;
    let Self_: BOOLEAN;

    /* Keep guest allocations addressable from 32-bit: when the guest lets the
     * system choose (Base==0), force the result below 2GB (kernel ZeroBits>32
     * mask semantics). */
    if Base.is_null() && ZeroBits == 0 as ULONG_PTR {
        ZeroBits = WOW64_ZEROBITS_BELOW_2G as ULONG_PTR;
    }

    /* Announce the new range to the CPU layer. Without this a guest that
     * generates code into the allocation cannot execute it at all on a JIT
     * host: FEX only decodes addresses inside the executable set it has been
     * told about. Before/After must stay paired around this one call. */
    Self_ = Wow64IsCurrentProcess(Process);
    if Self_ != 0 {
        Wow64NotifyAllocBefore(Base, Size, Type, Protect);
    }
    St = NtAllocateVirtualMemory(
        Process,
        &raw mut Base,
        ZeroBits,
        &raw mut Size,
        Type,
        Protect,
    );
    if Self_ != 0 {
        Wow64NotifyAllocAfter(Base, Size, Type, Protect, St as LONG);
    }
    if St >= 0 as NTSTATUS {
        if !Base32.is_null() {
            *Base32 = Base as ULONG_PTR as ULONG;
        }
        if !Size32.is_null() {
            *Size32 = Size as ULONG;
        }
    }
    return St;
}

unsafe extern "C" fn Wow64Manual_NtFreeVirtualMemory(a: *mut ULONG) -> NTSTATUS {
    let Process: HANDLE = *a.offset(0 as isize) as LONG as LONG_PTR as HANDLE;
    let Base32: *mut ULONG = *a.offset(1 as isize) as ULONG_PTR as *mut ULONG;
    let Size32: *mut ULONG = *a.offset(2 as isize) as ULONG_PTR as *mut ULONG;
    let FreeType: ULONG = *a.offset(3 as isize);
    let mut Base: PVOID = if !Base32.is_null() {
        *Base32 as ULONG_PTR as PVOID
    } else {
        core::ptr::null_mut()
    };
    let mut Size: SIZE_T = if !Size32.is_null() {
        *Size32 as SIZE_T
    } else {
        0 as SIZE_T
    };
    let mut St: NTSTATUS = 0;
    let mut Self_0: BOOLEAN = 0;

    /* A freed range must leave the CPU layer's executable set, or a later
     * allocation reusing the address executes the OLD translation. */
    Self_0 = Wow64IsCurrentProcess(Process);
    if Self_0 != 0 {
        Wow64NotifyFreeBefore(Base, Size, FreeType);
    }
    St = NtFreeVirtualMemory(
        Process,
        core::ptr::addr_of_mut!(Base),
        core::ptr::addr_of_mut!(Size),
        FreeType,
    );
    if Self_0 != 0 {
        Wow64NotifyFreeAfter(Base, Size, FreeType, St as LONG);
    }
    if St >= 0 as NTSTATUS {
        if !Base32.is_null() {
            *Base32 = Base as ULONG_PTR as ULONG;
        }
        if !Size32.is_null() {
            *Size32 = Size as ULONG;
        }
    }
    return St;
}

unsafe extern "C" fn Wow64Manual_NtProtectVirtualMemory(a: *mut ULONG) -> NTSTATUS {
    let mut Process: HANDLE =
        *a.offset(0 as isize) as LONG as LONG_PTR as HANDLE;
    let mut Base32: *mut ULONG =
        *a.offset(1 as isize) as ULONG_PTR as *mut ULONG;
    let mut Size32: *mut ULONG =
        *a.offset(2 as isize) as ULONG_PTR as *mut ULONG;
    let mut NewProtect: ULONG = *a.offset(3 as isize);
    let mut Old32: *mut ULONG =
        *a.offset(4 as isize) as ULONG_PTR as *mut ULONG;
    let mut Base: PVOID = if !Base32.is_null() {
        *Base32 as ULONG_PTR as PVOID
    } else {
        core::ptr::null_mut()
    };
    let mut Size: SIZE_T = (if !Size32.is_null() {
        *Size32
    } else {
        0 as ULONG
    }) as SIZE_T;
    let mut Old: ULONG = 0 as ULONG;
    let mut St: NTSTATUS = 0;
    let mut Self_0: BOOLEAN = 0;

    /* The RW -> RWX transition every runtime code generator makes. This is what
     * puts the range in the CPU layer's executable set and arms its
     * self-modifying-code tracking. */
    Self_0 = Wow64IsCurrentProcess(Process);
    if Self_0 != 0 {
        Wow64NotifyProtectBefore(Base, Size, NewProtect);
    }
    St = NtProtectVirtualMemory(
        Process,
        &raw mut Base,
        &raw mut Size,
        NewProtect,
        &raw mut Old,
    );
    if Self_0 != 0 {
        Wow64NotifyProtectAfter(Base, Size, NewProtect, St as LONG);
    }
    if St >= 0 as NTSTATUS {
        if !Base32.is_null() {
            *Base32 = Base as ULONG_PTR as ULONG;
        }
        if !Size32.is_null() {
            *Size32 = Size as ULONG;
        }
        if !Old32.is_null() {
            *Old32 = Old;
        }
    }
    return St;
}

unsafe extern "C" fn Wow64Manual_NtMapViewOfSection(a: *mut ULONG) -> NTSTATUS {
    let mut Section: HANDLE = *a.offset(0 as isize) as LONG as LONG_PTR as HANDLE;
    let mut Process: HANDLE = *a.offset(1 as isize) as LONG as LONG_PTR as HANDLE;
    let mut Base32: *mut ULONG = *a.offset(2 as isize) as ULONG_PTR as *mut ULONG;
    let mut ZeroBits: ULONG_PTR = *a.offset(3 as isize) as ULONG_PTR;
    let mut CommitSize: SIZE_T = *a.offset(4 as isize) as SIZE_T;
    let mut SectionOffset: PLARGE_INTEGER = *a.offset(5 as isize) as ULONG_PTR as PLARGE_INTEGER;
    let mut Size32: *mut ULONG = *a.offset(6 as isize) as ULONG_PTR as *mut ULONG;
    let mut Inherit: ULONG = *a.offset(7 as isize);
    let mut Type: ULONG = *a.offset(8 as isize);
    let mut Protect: ULONG = *a.offset(9 as isize);
    let mut Base: PVOID = if !Base32.is_null() {
        *Base32 as ULONG_PTR as PVOID
    } else {
        core::ptr::null_mut()
    };
    let mut Size: SIZE_T = if !Size32.is_null() {
        *Size32 as SIZE_T
    } else {
        0 as SIZE_T
    };
    let mut St: NTSTATUS;

    if Base.is_null() && ZeroBits == 0 as ULONG_PTR {
        ZeroBits = WOW64_ZEROBITS_BELOW_2G as ULONG_PTR;
    }

    St = NtMapViewOfSection(
        Section,
        Process,
        core::ptr::addr_of_mut!(Base),
        ZeroBits,
        CommitSize,
        SectionOffset,
        core::ptr::addr_of_mut!(Size),
        Inherit,
        Type,
        Protect,
    );
    if St >= 0 as NTSTATUS {
        if !Base32.is_null() {
            *Base32 = Base as ULONG_PTR as ULONG;
        }
        if !Size32.is_null() {
            *Size32 = Size as ULONG;
        }
        DbgPrint(
            b"[WOW64]   NtMapViewOfSection => %08lx base=%p size=%p\n\0".as_ptr() as PCSTR,
            St,
            Base,
            Size as PVOID,
        );
        /* Tell the CPU layer about a new IMAGE mapping — without this FEX
         * won't translate code from the range. Images only, as Wine's wow64
         * gates it on NtQuerySection(SectionImageInformation): FEX's handler
         * (HandleImageMap) takes the address for a mapped PE and looks up its
         * file and headers, and fed a pagefile-backed DATA view (a 32-bit
         * CreateFileMapping + MapViewOfFile -- spawn32's RemoteBody channel was
         * the first guest to do one) the guest never ran another instruction. */
        let mut ImageInfo: [UCHAR; 0x80 as usize] = [0; 0x80 as usize];
        if NtQuerySection(
            Section,
            1 as ULONG, /* SectionImageInformation */
            core::ptr::addr_of_mut!(ImageInfo) as PVOID,
            core::mem::size_of_val(&ImageInfo) as SIZE_T,
            core::ptr::null_mut(),
        ) >= 0 as NTSTATUS
        {
            if Wow64pBTCpuNotifyMap.is_some() {
                (Wow64pBTCpuNotifyMap.unwrap_unchecked())(
                    core::ptr::null_mut(),
                    Base,
                    core::ptr::null_mut(),
                    Size,
                    Type,
                    Protect,
                );
            }
            /* Guest DLLs carrying their own win32ksys syscall stubs
             * (gdi32/user32) export a per-module Wow64Transition slot that
             * must point at the BOP, same as ntdll32's. */
            Wow64pPatchTransitionSlot(Base, Size);
        }
    } else {
        DbgPrint(
            b"[WOW64] NtMapViewOfSection FAIL %08lx: sec=%p proc=%p base=%p zero=%p commit=%p offp=%p off=%I64x size=%p inherit=%lu type=%08lx prot=%08lx\n\0".as_ptr() as PCSTR,
            St,
            Section,
            Process,
            Base,
            ZeroBits as PVOID,
            CommitSize as PVOID,
            SectionOffset,
            if !SectionOffset.is_null() {
                (*SectionOffset).QuadPart
            } else {
                0 as LONGLONG
            },
            Size as PVOID,
            Inherit,
            Type,
            Protect,
        );
    }
    return St;
}

unsafe extern "C" fn Wow64Manual_NtUnmapViewOfSection(a: *mut ULONG) -> NTSTATUS {
    let Process: HANDLE = *a.offset(0 as isize) as LONG as LONG_PTR as HANDLE;
    let Base: PVOID = *a.offset(1 as isize) as ULONG_PTR as PVOID;
    let Self_0: BOOLEAN = Wow64IsCurrentProcess(Process);
    let mut St: NTSTATUS = 0;

    if Self_0 != 0 {
        Wow64NotifyUnmapBefore(Base);
    }
    St = NtUnmapViewOfSection(Process, Base);
    if Self_0 != 0 {
        Wow64NotifyUnmapAfter(Base, St);
    }
    return St;
}

unsafe extern "C" fn Wow64Manual_NtReadVirtualMemory(a: *mut ULONG) -> NTSTATUS {
    let Process: HANDLE = *a.offset(0 as isize) as LONG as LONG_PTR as HANDLE;
    let Read32: *mut ULONG = *a.offset(4 as isize) as ULONG_PTR as *mut ULONG;
    let mut Read: SIZE_T = 0 as SIZE_T;
    let St: NTSTATUS = NtReadVirtualMemory(
        Process,
        *a.offset(1 as isize) as ULONG_PTR as PVOID,
        *a.offset(2 as isize) as ULONG_PTR as PVOID,
        *a.offset(3 as isize) as SIZE_T,
        core::ptr::addr_of_mut!(Read),
    );
    if !Read32.is_null() {
        *Read32 = Read as ULONG;
    }
    return St;
}

unsafe extern "C" fn Wow64Manual_NtWriteVirtualMemory(a: *mut ULONG) -> NTSTATUS {
    let Process: HANDLE = *a.offset(0 as isize) as LONG as LONG_PTR as HANDLE;
    let Base: PVOID = *a.offset(1 as isize) as ULONG_PTR as PVOID;
    let Size: SIZE_T = *a.offset(3 as isize) as SIZE_T;
    let Written32: *mut ULONG = *a.offset(4 as isize) as ULONG_PTR as *mut ULONG;
    let mut Written: SIZE_T = 0 as SIZE_T;
    let St: NTSTATUS = NtWriteVirtualMemory(
        Process,
        Base,
        *a.offset(2 as isize) as ULONG_PTR as PVOID,
        Size,
        core::ptr::addr_of_mut!(Written),
    );
    if !Written32.is_null() {
        *Written32 = Written as ULONG;
    }
    /* After the fact on purpose: the layer only needs to know the bytes
     * changed, and Written is the range that actually did. */
    if St >= 0 as NTSTATUS && Wow64IsCurrentProcess(Process) as ::core::ffi::c_int != 0 {
        Wow64NotifyMemoryDirty(Base, Written);
    }
    return St;
}

unsafe extern "C" fn Wow64Manual_NtFlushInstructionCache(a: *mut ULONG) -> NTSTATUS {
    let Process: HANDLE = *a.offset(0 as isize) as LONG as LONG_PTR as HANDLE;
    let Base: PVOID = *a.offset(1 as isize) as ULONG_PTR as PVOID;
    let mut Length: SIZE_T = *a.offset(2 as isize) as SIZE_T;

    if Wow64IsCurrentProcess(Process) != 0 {
        Wow64NotifyFlushInstructionCache(Base, Length);
    }
    return NtFlushInstructionCache(Process, Base, Length);
}

unsafe extern "C" fn Wow64Manual_NtSetInformationProcess(a: *mut ULONG) -> NTSTATUS {
    let Process: HANDLE = *a.offset(0 as isize) as LONG as LONG_PTR as HANDLE;
    let Class: ULONG = *a.offset(1 as isize);
    let Buf32: PVOID = *a.offset(2 as isize) as ULONG_PTR as PVOID;
    let Len32: ULONG = *a.offset(3 as isize);
    let mut St: NTSTATUS;

    if Class != 0x22 as ULONG {
        // WOW64_ProcessExecuteFlags
        return NtSetInformationProcess(Process, Class, Buf32, Len32);
    }

    if Len32 as usize != core::mem::size_of::<ULONG>() || Buf32.is_null() {
        return 0xC0000004u32 as NTSTATUS; // STATUS_INFO_LENGTH_MISMATCH
    }

    St = NtSetInformationProcess(Process, Class, Buf32, Len32);
    if St >= 0 as NTSTATUS
        && Wow64IsCurrentProcess(Process) as ::core::ffi::c_int != 0
    {
        Wow64SetExecuteFlags(*(Buf32 as *mut ULONG));
    }
    return St;
}

unsafe extern "C" fn Wow64pCsrNarrowSysInfo(D32: *mut u8, S64: *const u8) {
    let mut i: ULONG = 0;
    i = 0 as ULONG;
    while i < 7 as ULONG {
        Wow64pCsrPut32(
            D32,
            i.wrapping_mul(4 as ULONG),
            Wow64pCsrGet32(
                S64,
                i.wrapping_mul(4 as ULONG),
            ),
        );
        i = i.wrapping_add(1);
    }
    {
        let mut Min: ULONGLONG = *(S64
            .add((WOW64_SBI_PTRBLK64 + 0) as usize)
            as *const ULONGLONG);
        let mut Max: ULONGLONG = *(S64
            .add((WOW64_SBI_PTRBLK64 + 8) as usize)
            as *const ULONGLONG);
        let mut Aff: ULONGLONG = *(S64
            .add((WOW64_SBI_PTRBLK64 + 16) as usize)
            as *const ULONGLONG);

        /* MaximumUserModeAddress is the host's ceiling (0x7FFFFFFEFFFF on
         * amd64). Truncating it hands the guest 0xFFFEFFFF, which reads as a
         * KERNEL address -- clamp to the 32-bit user ceiling instead. */
        if Max > WOW64_USER_CEILING32 as ULONGLONG {
            Max = WOW64_USER_CEILING32 as ULONGLONG;
        }
        if (Min >> 32) != 0 {
            DbgPrint(
                b"[WOW64]   BSSD MinimumUserModeAddress %I64x is above 4GB\n\0".as_ptr()
                    as PCSTR,
                Min,
            );
            Min = 0x10000 as ULONGLONG;
        }
        if (Aff >> 32) != 0 {
            DbgPrint(
                b"[WOW64]   BSSD affinity mask %I64x has processors a 32-bit client cannot address\n\0"
                    .as_ptr() as PCSTR,
                Aff,
            );
        }

        Wow64pCsrPut32(
            D32,
            (WOW64_SBI_PTRBLK32 + 0) as ULONG,
            Min as ULONG,
        );
        Wow64pCsrPut32(
            D32,
            (WOW64_SBI_PTRBLK32 + 4) as ULONG,
            Max as ULONG,
        );
        Wow64pCsrPut32(
            D32,
            (WOW64_SBI_PTRBLK32 + 8) as ULONG,
            Aff as ULONG,
        );
    }
    *D32.add(WOW64_SBI_NUMPROC32 as usize) =
        *S64.add(WOW64_SBI_NUMPROC64 as usize);
}

unsafe extern "C" fn Wow64pCsrNarrowUniStr(D32: *mut u8, O32: ULONG, S64: *const u8, O64: ULONG, Name: *const i8) {
    let Buf: ULONGLONG = *(S64.offset(O64 as isize).offset(8 as isize) as *const ULONGLONG);
    *D32.offset((O32.wrapping_add(0)) as isize) = *S64.offset((O64.wrapping_add(0)) as isize);
    *D32.offset((O32.wrapping_add(1)) as isize) = *S64.offset((O64.wrapping_add(1)) as isize);
    *D32.offset((O32.wrapping_add(2)) as isize) = *S64.offset((O64.wrapping_add(2)) as isize);
    *D32.offset((O32.wrapping_add(3)) as isize) = *S64.offset((O64.wrapping_add(3)) as isize);
    if Buf >> 32 != 0 {
        DbgPrint(
            b"[WOW64]   BSSD %s buffer %I64x is above 4GB \xE2\x80\x94 guest-invisible\n\0".as_ptr() as PCSTR,
            Name,
            Buf,
        );
        Wow64pCsrPut32(D32, O32.wrapping_add(4), 0);
    } else {
        Wow64pCsrPut32(D32, O32.wrapping_add(4), Buf as ULONG);
    }
}

unsafe extern "C" fn Wow64pCsrNarrowBssd(Wide: ULONGLONG) -> ULONG {
    let S: *const u8 = Wide as ULONG_PTR as *const u8;
    let D: *mut u8 = core::ptr::addr_of_mut!(Wow64pCsrBssd32) as *mut u8;
    let mut Ini: ULONGLONG;

    if Wide == 0 {
        return 0;
    }
    if ((D as ULONG_PTR as ULONGLONG) >> 32) != 0 {
        DbgPrint(b"[WOW64]   BSSD shadow above 4GB \xE2\x80\x94 leaving the 64-bit struct\n\0".as_ptr() as PCSTR);
        return Wide as ULONG;
    }

    Wow64pCsrNarrowUniStr(D, WOW64_BSSD_WINDIR32, S, WOW64_BSSD_WINDIR64, b"WindowsDirectory\0".as_ptr() as *const i8);
    Wow64pCsrNarrowUniStr(D, WOW64_BSSD_SYSDIR32, S, WOW64_BSSD_SYSDIR64, b"WindowsSystemDirectory\0".as_ptr() as *const i8);
    Wow64pCsrNarrowUniStr(D, WOW64_BSSD_OBJDIR32, S, WOW64_BSSD_OBJDIR64, b"NamedObjectDirectory\0".as_ptr() as *const i8);
    Wow64pCpy(D.offset(WOW64_BSSD_VERBLK32 as isize), S.offset(WOW64_BSSD_VERBLK64 as isize), WOW64_BSSD_VERBLK_LEN);
    Wow64pCsrNarrowSysInfo(D.offset(WOW64_BSSD_SYSINFO32 as isize), S.offset(WOW64_BSSD_SYSINFO64 as isize));
    Wow64pCpy(D.offset(WOW64_BSSD_TIMEOFDAY32 as isize), S.offset(WOW64_BSSD_TIMEOFDAY64 as isize), WOW64_BSSD_TIMEOFDAY_LEN);

    Ini = *(S.offset(WOW64_BSSD_INIMAP64 as isize) as *const ULONGLONG);
    Wow64pCsrPut32(
        D,
        WOW64_BSSD_INIMAP32,
        if (Ini >> 32) != 0 { 0 } else { Ini as ULONG },
    );

    Wow64pCpy(D.offset(WOW64_BSSD_NLS32 as isize), S.offset(WOW64_BSSD_NLS64 as isize), WOW64_BSSD_NLS_LEN);
    *D.offset((WOW64_BSSD_VDMFLAGS32 + 0) as isize) =
        *S.offset((WOW64_BSSD_VDMFLAGS64 + 0) as isize);
    *D.offset((WOW64_BSSD_VDMFLAGS32 + 1) as isize) =
        *S.offset((WOW64_BSSD_VDMFLAGS64 + 1) as isize);
    Wow64pCsrNarrowUniStr(D, WOW64_BSSD_SYS32X86_32, S, WOW64_BSSD_SYS32X86_64, b"WindowsSys32x86Directory\0".as_ptr() as *const i8);
    Wow64pCpy(D.offset(WOW64_BSSD_TSTAIL32 as isize), S.offset(WOW64_BSSD_TSTAIL64 as isize), WOW64_BSSD_TSTAIL_LEN);

    return D as ULONG_PTR as ULONG;
}

unsafe extern "C" fn Wow64pCsrNarrowStaticServerData(mut Wide: ULONGLONG) -> ULONG {
    let Src: *const ULONGLONG = Wide as *const ULONGLONG;
    let Self_0: ULONG_PTR = core::ptr::addr_of_mut!(
        Wow64pCsrStaticServerData32[0 as usize]
    ) as ULONG_PTR;
    let mut i: ULONG;

    if Wide == 0 {
        return 0 as ULONG;
    }

    /* The narrowed array itself has to be reachable from 32-bit code. */
    if (Self_0 as ULONGLONG) >> 32 != 0 {
        DbgPrint(
            b"[WOW64]   CSR static-server-data shadow at %p is above 4GB \xE2\x80\x94 guest-invisible, leaving the 64-bit array in place\n\0"
                as *const u8 as PCSTR,
            Self_0 as PVOID,
        );
        return Wide as ULONG;
    }

    i = 0 as ULONG;
    while i < WOW64_CSR_SERVER_DLL_MAX as ULONG {
        let E: ULONGLONG = *Src.offset(i as isize);
        if i == WOW64_BASESRV_SERVERDLL_INDEX as ULONG {
            /* The struct behind this entry needs narrowing too. */
            Wow64pCsrStaticServerData32[i as usize] = Wow64pCsrNarrowBssd(E);
        } else if E >> 32 != 0 {
            DbgPrint(
                b"[WOW64]   CSR static server data[%lu]=%I64x is above 4GB \xE2\x80\x94 guest-invisible\n\0"
                    as *const u8 as PCSTR,
                i,
                E,
            );
            Wow64pCsrStaticServerData32[i as usize] = 0 as ULONG;
        } else {
            Wow64pCsrStaticServerData32[i as usize] = E as ULONG;
        }
        i = i.wrapping_add(1);
    }
    return Self_0 as ULONG;
}

unsafe extern "C" fn Wow64pConnectPort(a: *mut ULONG, Secure: ::core::ffi::c_int) -> NTSTATUS {
    let mut Handle32: *mut ULONG =
        *a.offset(0 as isize) as ULONG_PTR as *mut ULONG;
    let mut Name32: *mut WOW64_UNICODE_STRING32_M =
        *a.offset(1 as isize) as ULONG_PTR as *mut WOW64_UNICODE_STRING32_M;
    let mut Name64: WOW64_UNICODE_STRING64_M = core::mem::zeroed();
    let mut Qos: PVOID = *a.offset(2 as isize) as ULONG_PTR as PVOID;
    let mut Cv32: *mut WOW64_PORT_VIEW32_M =
        *a.offset(3 as isize) as ULONG_PTR as *mut WOW64_PORT_VIEW32_M;
    let mut ServerSid: PVOID = if Secure != 0 {
        *a.offset(4 as isize) as ULONG_PTR as PVOID
    } else {
        core::ptr::null_mut()
    };
    let mut Sv32: *mut WOW64_REMOTE_PORT_VIEW32_M = *a.offset(
        (if Secure != 0 { 5 } else { 4 }) as isize,
    ) as ULONG_PTR as *mut WOW64_REMOTE_PORT_VIEW32_M;
    let mut MaxLen32: *mut ULONG = *a.offset(
        (if Secure != 0 { 6 } else { 5 }) as isize,
    ) as ULONG_PTR as *mut ULONG;
    let mut ConnInfo: PVOID = *a.offset(
        (if Secure != 0 { 7 } else { 6 }) as isize,
    ) as ULONG_PTR as PVOID;
    let mut ConnLen32: *mut ULONG = *a.offset(
        (if Secure != 0 { 8 } else { 7 }) as isize,
    ) as ULONG_PTR as *mut ULONG;
    let mut Cv64: WOW64_PORT_VIEW64_M = core::mem::zeroed();
    let mut Cv64Ptr: *mut WOW64_PORT_VIEW64_M = core::ptr::null_mut();
    let mut Sv64: WOW64_REMOTE_PORT_VIEW64_M = core::mem::zeroed();
    let mut Sv64Ptr: *mut WOW64_REMOTE_PORT_VIEW64_M = core::ptr::null_mut();
    let mut Csr64: WOW64_CSR_CONNECTINFO64_M = core::mem::zeroed();
    let mut Csr32: *mut WOW64_CSR_CONNECTINFO32_M = core::ptr::null_mut();
    let mut MaxLen: ULONG = 0 as ULONG;
    let mut ConnLen: ULONG = 0 as ULONG;
    let mut CvReq: ULONG = 0 as ULONG;
    let mut Handle: HANDLE = core::ptr::null_mut();
    let mut St: NTSTATUS = 0 as NTSTATUS;

    if Name32.is_null() {
        return 0xC000000D as ULONG as NTSTATUS;
    }
    Name64.Length = (*Name32).Length;
    Name64.MaximumLength = (*Name32).MaximumLength;
    Name64.__pad = 0 as ULONG;
    Name64.Buffer = (*Name32).Buffer as ULONG_PTR as ULONGLONG;

    /* CSR connect-info thunk: a 0x24-byte blob aimed at an *ApiPort is the
     * 32-bit CSR_API_CONNECTINFO — widen it so 64-bit csrss reads/writes its
     * own layout, then narrow the reply below. */
    if !ConnInfo.is_null()
        && !ConnLen32.is_null()
        && (*ConnLen32 as usize) == core::mem::size_of::<WOW64_CSR_CONNECTINFO32_M>()
        && (Name64.Length as usize)
            >= (7 as usize).wrapping_mul(core::mem::size_of::<WCHAR>())
    {
        let mut N: PCWSTR = Name64.Buffer as PCWSTR;
        let mut NChars: ULONG =
            ((Name64.Length as usize) / core::mem::size_of::<WCHAR>()) as ULONG;
        if *N.offset(NChars.wrapping_sub(7 as ULONG) as isize)
            == 0x0041u16
            && *N.offset(NChars.wrapping_sub(6 as ULONG) as isize)
                == 0x0070u16
            && *N.offset(NChars.wrapping_sub(5 as ULONG) as isize)
                == 0x0069u16
            && *N.offset(NChars.wrapping_sub(4 as ULONG) as isize)
                == 0x0050u16
            && *N.offset(NChars.wrapping_sub(3 as ULONG) as isize)
                == 0x006Fu16
            && *N.offset(NChars.wrapping_sub(2 as ULONG) as isize)
                == 0x0072u16
            && *N.offset(NChars.wrapping_sub(1 as ULONG) as isize)
                == 0x0074u16
        {
            Csr32 = ConnInfo as *mut WOW64_CSR_CONNECTINFO32_M;
            Csr64.ObjectDirectory = (*Csr32).ObjectDirectory as ULONGLONG;
            Csr64.SharedSectionBase = (*Csr32).SharedSectionBase as ULONGLONG;
            Csr64.SharedStaticServerData = (*Csr32).SharedStaticServerData as ULONGLONG;
            Csr64.SharedSectionHeap = (*Csr32).SharedSectionHeap as ULONGLONG;
            Csr64.DebugFlags = (*Csr32).DebugFlags;
            Csr64.SizeOfPebData = (*Csr32).SizeOfPebData;
            Csr64.SizeOfTebData = (*Csr32).SizeOfTebData;
            Csr64.NumberOfServerDllNames = (*Csr32).NumberOfServerDllNames;
            Csr64.ServerProcessId = (*Csr32).ServerProcessId as ULONGLONG;
            ConnInfo = core::ptr::addr_of_mut!(Csr64) as PVOID;
            ConnLen = core::mem::size_of::<WOW64_CSR_CONNECTINFO64_M>() as ULONG;
        }
    }

    if !Cv32.is_null() {
        Cv64.Length = core::mem::size_of::<WOW64_PORT_VIEW64_M>() as ULONG;
        Cv64.__pad1 = 0 as ULONG;
        Cv64.__pad0 = Cv64.__pad1;
        Cv64.SectionHandle = (*Cv32).SectionHandle as LONG as LONG_PTR as ULONGLONG;
        Cv64.SectionOffset = (*Cv32).SectionOffset;
        CvReq = (*Cv32).ViewSize;
        Cv64.ViewSize = CvReq as ULONGLONG;
        Cv64.ViewBase = (*Cv32).ViewBase as ULONGLONG;
        Cv64.ViewRemoteBase = (*Cv32).ViewRemoteBase as ULONGLONG;
        Cv64Ptr = core::ptr::addr_of_mut!(Cv64);
    }
    if !Sv32.is_null() {
        Sv64.Length = core::mem::size_of::<WOW64_REMOTE_PORT_VIEW64_M>() as ULONG;
        Sv64.__pad0 = 0 as ULONG;
        Sv64.ViewSize = (*Sv32).ViewSize as ULONGLONG;
        Sv64.ViewBase = (*Sv32).ViewBase as ULONGLONG;
        Sv64Ptr = core::ptr::addr_of_mut!(Sv64);
    }
    if !ConnLen32.is_null() && Csr32.is_null() {
        ConnLen = *ConnLen32;
    }

    if Secure != 0 {
        St = NtSecureConnectPort(
            core::ptr::addr_of_mut!(Handle),
            (core::ptr::addr_of_mut!(Name64) as PVOID).cast(),
            (Qos).cast(),
            (Cv64Ptr as PVOID).cast(),
            ServerSid,
            (Sv64Ptr as PVOID).cast(),
            if !MaxLen32.is_null() {
                core::ptr::addr_of_mut!(MaxLen)
            } else {
                core::ptr::null_mut::<ULONG>()
            },
            ConnInfo,
            if !ConnLen32.is_null() {
                core::ptr::addr_of_mut!(ConnLen)
            } else {
                core::ptr::null_mut::<ULONG>()
            },
        );
    } else {
        St = NtConnectPort(
            core::ptr::addr_of_mut!(Handle),
            (core::ptr::addr_of_mut!(Name64) as PVOID).cast(),
            (Qos).cast(),
            (Cv64Ptr as PVOID).cast(),
            (Sv64Ptr as PVOID).cast(),
            if !MaxLen32.is_null() {
                core::ptr::addr_of_mut!(MaxLen)
            } else {
                core::ptr::null_mut::<ULONG>()
            },
            ConnInfo,
            if !ConnLen32.is_null() {
                core::ptr::addr_of_mut!(ConnLen)
            } else {
                core::ptr::null_mut::<ULONG>()
            },
        );
    }

    DbgPrint(
        b"[WOW64]   Nt%sConnectPort '%.*S' => %08lx (handle=%p maxlen=%lu)\n\0" as *const u8
            as PCSTR,
        if Secure != 0 {
            b"Secure\0" as *const u8 as *const ::core::ffi::c_char
        } else {
            b"\0" as *const u8 as *const ::core::ffi::c_char
        },
        ((Name64.Length as usize) / core::mem::size_of::<WCHAR>()) as ::core::ffi::c_int,
        Name64.Buffer as PCWSTR,
        St,
        Handle,
        MaxLen,
    );
    /* The client view is where csrlib builds CsrPortHeap, so every capture
     * buffer the guest ever allocates lives inside it. A base above 4GB is
     * invisible to the guest and silently truncates below. */
    if !Cv32.is_null() {
        DbgPrint(
            b"[WOW64]   ClientView: base=%I64x size=%I64x remote=%I64x\n\0" as *const u8 as PCSTR,
            Cv64.ViewBase,
            Cv64.ViewSize,
            Cv64.ViewRemoteBase,
        );
    }
    if !Sv32.is_null() {
        DbgPrint(
            b"[WOW64]   ServerView: base=%I64x size=%I64x\n\0" as *const u8 as PCSTR,
            Sv64.ViewBase,
            Sv64.ViewSize,
        );
    }

    if St >= 0 as NTSTATUS {
        if !Handle32.is_null() {
            *Handle32 = Handle as ULONG_PTR as ULONG;
        }
        if !Cv32.is_null() {
            (*Cv32).ViewSize = Cv64.ViewSize as ULONG;
            (*Cv32).ViewBase = Cv64.ViewBase as ULONG;
            (*Cv32).ViewRemoteBase = Cv64.ViewRemoteBase as ULONG;
        }
        if !Csr32.is_null() && !Cv32.is_null() {
            Wow64pCsrPortDelta = Cv64.ViewRemoteBase.wrapping_sub(Cv64.ViewBase);
            Wow64pCsrViewLocal = Cv64.ViewBase;
            if Cv64.ViewSize
                > ((2 * WOW64_CSR_SCRATCH_SIZE) as ULONGLONG)
            {
                Wow64pCsrViewSize =
                    (Cv64.ViewSize as ULONG).wrapping_sub(WOW64_CSR_SCRATCH_SIZE as ULONG);
                Wow64pCsrScratchLocal =
                    Cv64.ViewBase.wrapping_add(Wow64pCsrViewSize as ULONG_PTR);
                Wow64pCsrScratchSize = WOW64_CSR_SCRATCH_SIZE as ULONG;
                /* The guest's heap must stop short of the tail we keep. */
                (*Cv32).ViewSize = Wow64pCsrViewSize;
                {
                    /* The port view is a RESERVE section -- its pages commit on
                     * demand, so the tail we just claimed is reserved only and
                     * writing it faults. Commit it before anyone uses it. */
                    let mut Sc: PVOID = Wow64pCsrScratchLocal as PVOID;
                    let mut ScSize: SIZE_T = WOW64_CSR_SCRATCH_SIZE as SIZE_T;
                    let mut CSt: NTSTATUS = NtAllocateVirtualMemory(
                        (-(1 as LONG_PTR)) as HANDLE,
                        core::ptr::addr_of_mut!(Sc),
                        0 as ULONG_PTR,
                        core::ptr::addr_of_mut!(ScSize),
                        0x1000 as ULONG, /* MEM_COMMIT */
                        0x04 as ULONG, /* PAGE_READWRITE */
                    );
                    if CSt < 0 as NTSTATUS {
                        Wow64pCsrScratchLocal = 0 as ULONG_PTR;
                        Wow64pCsrScratchSize = 0 as ULONG;
                        (*Cv32).ViewSize = Cv64.ViewSize as ULONG;
                        DbgPrint(
                            b"[WOW64]   CSR scratch commit FAILED %08lx \xE2\x80\x94 capture buffers cannot be widened\n\0"
                                as *const u8 as PCSTR,
                            CSt,
                        );
                    }
                }
            } else {
                /* Too small to share. Say so rather than silently handing the
                 * guest a capture buffer csrss cannot read. */
                Wow64pCsrViewSize = Cv64.ViewSize as ULONG;
                Wow64pCsrScratchLocal = 0 as ULONG_PTR;
                Wow64pCsrScratchSize = 0 as ULONG;
                DbgPrint(
                    b"[WOW64]   CSR port view %I64x too small to reserve scratch \xE2\x80\x94 capture buffers cannot be widened\n\0"
                        as *const u8 as PCSTR,
                    Cv64.ViewSize,
                );
            }
        }
        if !Sv32.is_null() {
            (*Sv32).ViewSize = Sv64.ViewSize as ULONG;
            (*Sv32).ViewBase = Sv64.ViewBase as ULONG;
        }
        if !MaxLen32.is_null() {
            *MaxLen32 = MaxLen;
        }
        if !Csr32.is_null() {
            /* Narrow the CSR reply. The shared-section pointers must be
             * guest-addressable; a >4GB base means csrss's view landed
             * outside the guest's world — report it loudly, the connect
             * "succeeded" but kernel32 would read garbage. */
            if (Csr64.SharedSectionBase | Csr64.SharedStaticServerData | Csr64.SharedSectionHeap)
                >> (32 as ULONG)
                != 0
            {
                DbgPrint(
                    b"[WOW64]   CSR connect: shared section ABOVE 4GB (base=%I64x static=%I64x heap=%I64x) \xE2\x80\x94 guest-invisible!\n\0"
                        as *const u8 as PCSTR,
                    Csr64.SharedSectionBase,
                    Csr64.SharedStaticServerData,
                    Csr64.SharedSectionHeap,
                );
            }
            DbgPrint(
                b"[WOW64]   CSR connectinfo: base=%I64x static=%I64x heap=%I64x pid=%I64x ndlls=%lx\n\0"
                    as *const u8 as PCSTR,
                Csr64.SharedSectionBase,
                Csr64.SharedStaticServerData,
                Csr64.SharedSectionHeap,
                Csr64.ServerProcessId,
                Csr64.NumberOfServerDllNames,
            );
            (*Csr32).ObjectDirectory = Csr64.ObjectDirectory as ULONG;
            (*Csr32).SharedSectionBase = Csr64.SharedSectionBase as ULONG;
            (*Csr32).SharedStaticServerData =
                Wow64pCsrNarrowStaticServerData(Csr64.SharedStaticServerData);
            (*Csr32).SharedSectionHeap = Csr64.SharedSectionHeap as ULONG;
            (*Csr32).DebugFlags = Csr64.DebugFlags;
            (*Csr32).SizeOfPebData = Csr64.SizeOfPebData;
            (*Csr32).SizeOfTebData = Csr64.SizeOfTebData;
            (*Csr32).NumberOfServerDllNames = Csr64.NumberOfServerDllNames;
            (*Csr32).ServerProcessId = Csr64.ServerProcessId as ULONG;
            if !ConnLen32.is_null() {
                *ConnLen32 = core::mem::size_of::<WOW64_CSR_CONNECTINFO32_M>() as ULONG;
            }
        } else if !ConnLen32.is_null() {
            *ConnLen32 = ConnLen;
        }
    }
    return St;
}

unsafe extern "C" fn Wow64Manual_NtSecureConnectPort(a: *mut ULONG) -> NTSTATUS {
    return Wow64pConnectPort(a, 1);
}

unsafe extern "C" fn Wow64Manual_NtConnectPort(a: *mut ULONG) -> NTSTATUS {
    return Wow64pConnectPort(a, 0);
}
unsafe extern "C" fn Wow64Manual_NtCreateEvent(a: *mut ULONG) -> NTSTATUS {
    let Handle32: *mut ULONG = *a.offset(0 as isize) as ULONG_PTR as *mut ULONG;
    let Access: ACCESS_MASK = *a.offset(1 as isize) as ACCESS_MASK;
    let mut Oa64: WOW64_OBJECT_ATTRIBUTES64_M = core::mem::zeroed();
    let mut Uname64: WOW64_UNICODE_STRING64_M = core::mem::zeroed();
    let mut Oa64Ptr: PVOID = core::ptr::null_mut();
    let mut Handle: HANDLE = core::ptr::null_mut();
    let mut St: NTSTATUS;

    if *a.offset(2 as isize) != 0 {
        Wow64pMarshalOa(
            core::ptr::addr_of_mut!(Oa64),
            core::ptr::addr_of_mut!(Uname64),
            *a.offset(2 as isize),
        );
        Oa64Ptr = core::ptr::addr_of_mut!(Oa64) as PVOID;
    }
    St = NtCreateEvent(
        core::ptr::addr_of_mut!(Handle),
        Access,
        (Oa64Ptr).cast(),
        *a.offset(3 as isize),
        *a.offset(4 as isize) as BOOLEAN,
    );
    if St >= (0 as NTSTATUS) && !Handle32.is_null() {
        *Handle32 = Handle as ULONG_PTR as ULONG;
    }
    return St;
}

unsafe extern "C" fn Wow64Manual_NtCreateKeyedEvent(a: *mut ULONG) -> NTSTATUS {
    let Handle32: *mut ULONG =
        *a.offset(0 as isize) as ULONG_PTR as *mut ULONG;
    let Access: ACCESS_MASK = *a.offset(1 as isize) as ACCESS_MASK;
    let mut Oa64: WOW64_OBJECT_ATTRIBUTES64_M = WOW64_OBJECT_ATTRIBUTES64_M {
        Length: 0,
        __pad0: 0,
        RootDirectory: 0,
        ObjectName: 0,
        Attributes: 0,
        __pad1: 0,
        SecurityDescriptor: 0,
        SecurityQualityOfService: 0,
    };
    let mut Uname64: WOW64_UNICODE_STRING64_M = WOW64_UNICODE_STRING64_M {
        Length: 0,
        MaximumLength: 0,
        __pad: 0,
        Buffer: 0,
    };
    let mut Oa64Ptr: PVOID = core::ptr::null_mut();
    let mut Handle: HANDLE = core::ptr::null_mut();
    let mut St: NTSTATUS;

    if *a.offset(2 as isize) != 0 {
        Wow64pMarshalOa(
            core::ptr::addr_of_mut!(Oa64),
            core::ptr::addr_of_mut!(Uname64),
            *a.offset(2 as isize),
        );
        Oa64Ptr = core::ptr::addr_of_mut!(Oa64) as PVOID;
    }
    St = NtCreateKeyedEvent(
        core::ptr::addr_of_mut!(Handle),
        Access,
        (Oa64Ptr).cast(),
        *a.offset(3 as isize),
    );
    if St >= 0 as NTSTATUS && !Handle32.is_null() {
        *Handle32 = Handle as ULONG_PTR as ULONG;
    }
    return St;
}

unsafe extern "C" fn Wow64Manual_NtQuerySection(a: *mut ULONG) -> NTSTATUS {
    #[repr(C)]
    struct SbiOut32 {
        Base: ULONG,
        Attr: ULONG,
        Size: LARGE_INTEGER,
    }

    let Section: HANDLE = (*a.offset(0 as isize)) as LONG as LONG_PTR as HANDLE;
    let Class: ULONG = *a.offset(1 as isize);
    let Buf32: PVOID = (*a.offset(2 as isize)) as ULONG_PTR as PVOID;
    let Len32: ULONG = *a.offset(3 as isize);
    let RetLen32: *mut ULONG = (*a.offset(4 as isize)) as ULONG_PTR as *mut ULONG;
    let mut Ret: SIZE_T = 0 as SIZE_T;
    let mut St: NTSTATUS;

    if Class == 1 as ULONG {
        /* SectionImageInformation */
        let mut Sii64: WOW64_SECTION_IMAGE_INFORMATION64 = core::mem::zeroed();
        let Sii32: *mut WOW64_SECTION_IMAGE_INFORMATION32 = Buf32 as *mut WOW64_SECTION_IMAGE_INFORMATION32;
        if Len32 < core::mem::size_of::<WOW64_SECTION_IMAGE_INFORMATION32>() as ULONG {
            return (0xC0000004 as u32) as NTSTATUS; /* STATUS_INFO_LENGTH_MISMATCH */
        }
        St = NtQuerySection(
            Section,
            Class,
            core::ptr::addr_of_mut!(Sii64) as PVOID,
            core::mem::size_of::<WOW64_SECTION_IMAGE_INFORMATION64>() as SIZE_T,
            core::ptr::addr_of_mut!(Ret),
        );
        if St >= 0 as NTSTATUS {
            (*Sii32).TransferAddress = Sii64.TransferAddress as ULONG;
            (*Sii32).ZeroBits = Sii64.ZeroBits;
            (*Sii32).MaximumStackSize = Sii64.MaximumStackSize as ULONG;
            (*Sii32).CommittedStackSize = Sii64.CommittedStackSize as ULONG;
            (*Sii32).SubSystemType = Sii64.SubSystemType;
            (*Sii32).SubSystemMinorVersion = Sii64.SubSystemMinorVersion;
            (*Sii32).SubSystemMajorVersion = Sii64.SubSystemMajorVersion;
            (*Sii32).GpValue = Sii64.GpValue;
            (*Sii32).ImageCharacteristics = Sii64.ImageCharacteristics;
            (*Sii32).DllCharacteristics = Sii64.DllCharacteristics;
            (*Sii32).Machine = Sii64.Machine;
            (*Sii32).ImageContainsCode = Sii64.ImageContainsCode;
            (*Sii32).ImageFlags = Sii64.ImageFlags;
            (*Sii32).LoaderFlags = Sii64.LoaderFlags;
            (*Sii32).ImageFileSize = Sii64.ImageFileSize;
            (*Sii32).CheckSum = Sii64.CheckSum;
            if !RetLen32.is_null() {
                *RetLen32 = core::mem::size_of::<WOW64_SECTION_IMAGE_INFORMATION32>() as ULONG;
            }
        }
        return St;
    }
    if Class == 0 as ULONG {
        /* SectionBasicInformation */
        let mut Sbi64: WOW64_SECTION_BASIC_INFORMATION64 = core::mem::zeroed();
        let Sbi32: *mut SbiOut32 = Buf32 as *mut SbiOut32;
        if Len32 < core::mem::size_of::<SbiOut32>() as ULONG {
            return (0xC0000004 as u32) as NTSTATUS;
        }
        St = NtQuerySection(
            Section,
            Class,
            core::ptr::addr_of_mut!(Sbi64) as PVOID,
            core::mem::size_of::<WOW64_SECTION_BASIC_INFORMATION64>() as SIZE_T,
            core::ptr::addr_of_mut!(Ret),
        );
        if St >= 0 as NTSTATUS {
            (*Sbi32).Base = Sbi64.BaseAddress as ULONG;
            (*Sbi32).Attr = Sbi64.Attributes;
            (*Sbi32).Size.QuadPart = Sbi64.MaximumSize as LONGLONG;
            if !RetLen32.is_null() {
                *RetLen32 = core::mem::size_of::<SbiOut32>() as ULONG;
            }
        }
        return St;
    }
    DbgPrint(
        b"[WOW64]   NtQuerySection class=%lu unhandled\n\0".as_ptr() as PCSTR,
        Class,
    );
    return (0xC0000003 as u32) as NTSTATUS; /* STATUS_INVALID_INFO_CLASS */
}

unsafe extern "C" fn Wow64Manual_NtQuerySymbolicLinkObject(a: *mut ULONG) -> NTSTATUS {
    let mut Link: HANDLE = *a.offset(0 as isize) as LONG as LONG_PTR as HANDLE;
    let mut Tgt32: *mut WOW64_UNICODE_STRING32_M =
        *a.offset(1 as isize) as ULONG_PTR as *mut WOW64_UNICODE_STRING32_M;
    let mut RetLen32: *mut ULONG = *a.offset(2 as isize) as ULONG_PTR as *mut ULONG;
    let mut Tgt64: WOW64_UNICODE_STRING64_M = core::mem::zeroed();
    let mut RetLen: ULONG = 0;
    let mut St: NTSTATUS = 0;

    if Tgt32.is_null() {
        return 0xC000000D as ULONG as NTSTATUS; /* STATUS_INVALID_PARAMETER */
    }
    Tgt64.Length = (*Tgt32).Length;
    Tgt64.MaximumLength = (*Tgt32).MaximumLength;
    Tgt64.__pad = 0;
    Tgt64.Buffer = (*Tgt32).Buffer as ULONG_PTR as ULONGLONG; /* guest buffer, host writes here */
    St = NtQuerySymbolicLinkObject(
        Link,
        (core::ptr::addr_of_mut!(Tgt64) as PVOID).cast(),
        core::ptr::addr_of_mut!(RetLen) as PULONG,
    );
    (*Tgt32).Length = Tgt64.Length;
    if !RetLen32.is_null() {
        *RetLen32 = RetLen;
    }
    return St;
}

unsafe extern "C" fn Wow64Manual_NtQueueApcThread(a: *mut ULONG) -> NTSTATUS {
    let Thread: HANDLE = *a.add(0) as LONG as LONG_PTR as HANDLE;
    let mut Routine: PVOID = core::ptr::null_mut();

    if *a.add(1) == 0 {
        return NtQueueApcThread(
            Thread,
            None,
            *a.add(2) as ULONG_PTR as PVOID,
            *a.add(3) as ULONG_PTR as PVOID,
            *a.add(4) as ULONG_PTR as PVOID,
        );
    }
    Routine = Wow64pApcRoutineFor(Thread);
    if Routine.is_null() {
        DbgPrint(
            b"[WOW64]   NtQueueApcThread: target thread's process runs no 32-bit code\n\0"
                .as_ptr() as PCSTR,
        );
        return 0xC0000022u32 as NTSTATUS;
    }
    return NtQueueApcThread(
        Thread,
        core::mem::transmute::<*mut c_void, _>(Routine),
        Wow64pApcContext(
            *a.add(1),
            *a.add(2),
        ),
        *a.add(3) as ULONG_PTR as PVOID,
        *a.add(4) as ULONG_PTR as PVOID,
    );
}

unsafe extern "C" fn Wow64Manual_NtSetTimer(a: *mut ULONG) -> NTSTATUS {
    return NtSetTimer(
        *a.offset(0 as isize) as LONG as LONG_PTR as HANDLE,
        *a.offset(1 as isize) as ULONG_PTR as PLARGE_INTEGER,
        if *a.offset(2 as isize) != 0 {
            core::mem::transmute::<*mut c_void, _>(Wow64ApcRoutine as usize as PVOID)
        } else {
            None
        },
        if *a.offset(2 as isize) != 0 {
            Wow64pApcContext(
                *a.offset(2 as isize),
                *a.offset(3 as isize),
            )
        } else {
            *a.offset(3 as isize) as ULONG_PTR as PVOID
        },
        *a.offset(4 as isize) as BOOLEAN,
        *a.offset(5 as isize) as LONG,
        *a.offset(6 as isize) as ULONG_PTR as PBOOLEAN,
    );
}

unsafe extern "C" fn Wow64Manual_NtContinue(a: *mut ULONG) -> NTSTATUS {
    let Ctx32: PVOID = *a.offset(0 as isize) as ULONG_PTR as PVOID;
    /* The end of a guest APC (wow64_callback.c): does not return. */
    Wow64pCompleteGuestApc(
        *a.offset(0 as isize) as ULONG,
        *a.offset(1 as isize) as BOOLEAN,
    );
    if Ctx32.is_null() || Wow64pBTCpuSetContext.is_none() {
        return 0xC000000Du32 as NTSTATUS;
    }
    return (Wow64pBTCpuSetContext.unwrap_unchecked())(
        (-(2 as LONG_PTR)) as HANDLE, /* thread */
        (-(1 as LONG_PTR)) as HANDLE, /* process */
        core::ptr::null_mut(),
        Ctx32,
    ) as NTSTATUS;
}

unsafe extern "C" fn Wow64Manual_NtRaiseException(a: *mut ULONG) -> NTSTATUS {
    let mut Rec32: *const BYTE = *a.offset(0 as isize) as ULONG_PTR as *const BYTE;
    let mut Ctx32: *const BYTE = *a.offset(1 as isize) as ULONG_PTR as *const BYTE;
    let mut FirstChance: ULONG = *a.offset(2 as isize);
    let mut Resume: [ULONG; (WOW64_CTX32_SIZE / 4) as _] = [0; (WOW64_CTX32_SIZE / 4) as _];
    let mut Live: [ULONG; (WOW64_CTX32_SIZE / 4) as _] = [0; (WOW64_CTX32_SIZE / 4) as _];
    let mut Esp: ULONG = 0;
    let mut i: ULONG = 0;

    if Rec32.is_null()
        || Ctx32.is_null()
        || (Rec32 as ULONG_PTR) >= 0x80000000 as ULONG_PTR
        || (Ctx32 as ULONG_PTR) >= 0x80000000 as ULONG_PTR
    {
        return 0xC000000Du32 as NTSTATUS;
    }

    if FirstChance == 0 {
        DbgPrint(
            b"[WOW64] second-chance exception %08lx eip=%08lx \xE2\x80\x94 unhandled in guest, terminating\n\0".as_ptr() as PCSTR,
            *(Rec32 as *const ULONG),
            *(Ctx32.offset(WOW64_CTX32_EIP as isize) as *const ULONG),
        );
        NtTerminateProcess(
            (-(1 as LONG_PTR)) as HANDLE,
            *(Rec32 as *const ULONG) as LONG,
        );
        return 0xC000000Du32 as NTSTATUS; /* not reached */
    }

    if Wow64pKiUserExceptionDispatcher32.is_null()
        || Wow64pBTCpuSetContext.is_none()
        || Wow64pBTCpuGetContext.is_none()
    {
        return 0xC000000Du32 as NTSTATUS;
    }

    /* DBG_PRINTEXCEPTION(_WIDE)_C: natively the kernel's KD consumes these
     * (that's how OutputDebugString "returns" with no debugger). We bypass the
     * kernel, so consume them here the same way — print and resume at the
     * captured context. Dispatching them to guest SEH instead would kill the
     * process: the guest kernel32's __except is DUMMY_PSEH (never registers). */
    {
        let mut Code: ULONG = *(Rec32 as *const ULONG);
        let mut Prm: *const ULONG = Rec32.offset(0x14 as isize) as *const ULONG; /* ExceptionInformation */
        if Code == 0x40010006 as ULONG /* DBG_PRINTEXCEPTION_C */
            || Code == 0x4001000A as ULONG /* DBG_PRINTEXCEPTION_WIDE_C */
        {
            let mut Len: ULONG = *Prm.offset(0 as isize);
            let mut Str: ULONG = *Prm.offset(1 as isize);
            if Str != 0 && Str < 0x80000000 as ULONG && Len != 0 && Len < 512 as ULONG {
                if Code == 0x40010006 as ULONG {
                    DbgPrint(
                        b"[GUEST32] %.*s\0".as_ptr() as PCSTR,
                        Len as core::ffi::c_int,
                        Str as ULONG_PTR as *const core::ffi::c_char,
                    );
                } else {
                    DbgPrint(
                        b"[GUEST32] %.*ws\0".as_ptr() as PCSTR,
                        Len as core::ffi::c_int,
                        Str as ULONG_PTR as *const WCHAR,
                    );
                }
            }
            i = 0 as ULONG;
            while i < (WOW64_CTX32_SIZE / 4) as ULONG {
                Resume[i as usize] = *(Ctx32 as ULONG_PTR as *const ULONG).offset(i as isize);
                i = i.wrapping_add(1);
            }
            return (Wow64pBTCpuSetContext.unwrap_unchecked())(
                (-(2 as LONG_PTR)) as HANDLE,
                (-(1 as LONG_PTR)) as HANDLE,
                NULL,
                &raw mut Resume as *mut ULONG as PVOID,
            ) as NTSTATUS;
        }
    }

    /* Frame placement: below the LIVE guest Esp (the syscall-stub esp from the
     * CPU layer), NOT below the captured Ctx32->Esp — the raiser's frames (and
     * the very Rec32/Ctx32 buffers being copied) live between the two. */
    i = 0 as ULONG;
    while i < (WOW64_CTX32_SIZE / 4) as ULONG {
        Live[i as usize] = 0 as ULONG;
        i = i.wrapping_add(1);
    }
    if (Wow64pBTCpuGetContext.unwrap_unchecked())(
        (-(2 as LONG_PTR)) as HANDLE,
        (-(1 as LONG_PTR)) as HANDLE,
        NULL,
        &raw mut Live as *mut ULONG as PVOID,
    ) < 0 as LONG
    {
        return 0xC000000Du32 as NTSTATUS;
    }
    Esp = Wow64pBuildGuestExceptionFrame(
        (Rec32 as *const core::ffi::c_void).cast(),
        (Ctx32 as *const core::ffi::c_void).cast(),
        Live[(WOW64_CTX32_ESP / 4) as usize].wrapping_sub(0x40 as ULONG),
    );
    if Esp == 0 {
        return 0xC000000Du32 as NTSTATUS;
    }

    if Wow64TraceSyscalls != 0 {
        DbgPrint(
            b"[WOW64] NtRaiseException %08lx eip=%08lx -> 32-bit SEH dispatch (esp=%08lx)\n\0".as_ptr() as PCSTR,
            *(Rec32 as *const ULONG),
            *(Ctx32.offset(WOW64_CTX32_EIP as isize) as *const ULONG),
            Esp,
        );
    }

    i = 0 as ULONG;
    while i < (WOW64_CTX32_SIZE / 4) as ULONG {
        Resume[i as usize] = *(Ctx32 as ULONG_PTR as *const ULONG).offset(i as isize);
        i = i.wrapping_add(1);
    }
    Resume[(WOW64_CTX32_EIP / 4) as usize] =
        Wow64pKiUserExceptionDispatcher32 as ULONG_PTR as ULONG;
    Resume[(WOW64_CTX32_ESP / 4) as usize] = Esp;
    return (Wow64pBTCpuSetContext.unwrap_unchecked())(
        (-(2 as LONG_PTR)) as HANDLE,
        (-(1 as LONG_PTR)) as HANDLE,
        NULL,
        &raw mut Resume as *mut ULONG as PVOID,
    ) as NTSTATUS;
}

unsafe extern "C" fn Wow64Manual_NtGdiOpenDCW(a: *mut ULONG) -> NTSTATUS {
    let Dev32: *mut WOW64_UNICODE_STRING32_M =
        *a.offset(0 as isize) as ULONG_PTR as *mut WOW64_UNICODE_STRING32_M;
    let Pdm: PVOID = *a.offset(1 as isize) as ULONG_PTR as PVOID;
    let Log32: *mut WOW64_UNICODE_STRING32_M =
        *a.offset(2 as isize) as ULONG_PTR as *mut WOW64_UNICODE_STRING32_M;
    let IType: ULONG = *a.offset(3 as isize);
    let BDisplay: BOOL = *a.offset(4 as isize) as BOOL;
    let HSpool: HANDLE = *a.offset(5 as isize) as ULONG_PTR as HANDLE;
    let UMdh32: *mut ULONG =
        *a.offset(6 as isize) as ULONG_PTR as *mut ULONG;
    let mut Dev64: WOW64_UNICODE_STRING64_M = WOW64_UNICODE_STRING64_M {
        Length: 0,
        MaximumLength: 0,
        __pad: 0,
        Buffer: 0,
    };
    let mut Log64: WOW64_UNICODE_STRING64_M = WOW64_UNICODE_STRING64_M {
        Length: 0,
        MaximumLength: 0,
        __pad: 0,
        Buffer: 0,
    };
    let mut Dev64Ptr: PVOID = core::ptr::null_mut();
    let mut Log64Ptr: PVOID = core::ptr::null_mut();
    let mut UMdh64: PVOID = core::ptr::null_mut();
    let mut Hdc: HANDLE;

    if !Dev32.is_null() {
        Dev64.Length = (*Dev32).Length;
        Dev64.MaximumLength = (*Dev32).MaximumLength;
        Dev64.__pad = 0 as ULONG;
        Dev64.Buffer = (*Dev32).Buffer as ULONG_PTR as ULONGLONG;
        Dev64Ptr = core::ptr::addr_of_mut!(Dev64) as PVOID;
    }
    if !Log32.is_null() {
        Log64.Length = (*Log32).Length;
        Log64.MaximumLength = (*Log32).MaximumLength;
        Log64.__pad = 0 as ULONG;
        Log64.Buffer = (*Log32).Buffer as ULONG_PTR as ULONGLONG;
        Log64Ptr = core::ptr::addr_of_mut!(Log64) as PVOID;
    }

    Hdc = (NtGdiOpenDCW(
        (Dev64Ptr).cast(),
        (Pdm).cast(),
        (Log64Ptr).cast(),
        IType,
        BDisplay,
        HSpool,
        if !UMdh32.is_null() {
            core::ptr::addr_of_mut!(UMdh64) as PVOID
        } else {
            core::ptr::null_mut()
        },
    )).cast();
    if !UMdh32.is_null() {
        *UMdh32 = UMdh64 as ULONG_PTR as ULONG;
    }
    DbgPrint(
        b"[WOW64]   NtGdiOpenDCW(dev='%.*S' type=%lu disp=%d) => hdc=%p\n\0" as *const u8 as PCSTR,
        if !Dev64Ptr.is_null() {
            (Dev64.Length as usize / core::mem::size_of::<WCHAR>()) as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        },
        if !Dev64Ptr.is_null() {
            Dev64.Buffer as usize as PCWSTR
        } else {
            RUSTCONV_WSTR_96A296D224F2.as_ptr()
        },
        IType,
        BDisplay,
        Hdc,
    );
    return Hdc as ULONG_PTR as ULONG as NTSTATUS;
}

unsafe extern "C" fn Wow64Manual_NtQueryValueKey(a: *mut ULONG) -> NTSTATUS {
    let mut Key: HANDLE = *a.offset(0 as isize) as LONG as LONG_PTR as HANDLE;
    let mut Name32: *mut WOW64_UNICODE_STRING32_M =
        *a.offset(1 as isize) as ULONG_PTR as *mut WOW64_UNICODE_STRING32_M;
    let mut Class: ULONG = *a.offset(2 as isize);
    let mut Buf: PVOID = *a.offset(3 as isize) as ULONG_PTR as PVOID;
    let mut Len: ULONG = *a.offset(4 as isize);
    let mut RetLen: *mut ULONG =
        *a.offset(5 as isize) as ULONG_PTR as *mut ULONG;
    let mut Name64: WOW64_UNICODE_STRING64_M = WOW64_UNICODE_STRING64_M {
        Length: 0,
        MaximumLength: 0,
        __pad: 0,
        Buffer: 0,
    };
    let mut Name64Ptr: PVOID = core::ptr::null_mut();
    if !Name32.is_null() {
        Name64.Length = (*Name32).Length;
        Name64.MaximumLength = (*Name32).MaximumLength;
        Name64.__pad = 0;
        Name64.Buffer = (*Name32).Buffer as ULONG_PTR as ULONGLONG;
        Name64Ptr = core::ptr::addr_of_mut!(Name64) as PVOID;
    }
    return NtQueryValueKey(Key, Name64Ptr, Class, Buf, Len, RetLen as PULONG);
}

unsafe extern "C" fn Wow64Manual_NtUserProcessConnect(a: *mut ULONG) -> NTSTATUS {
    let mut Process: HANDLE =
        *a.offset(0 as isize) as LONG as LONG_PTR as HANDLE;
    let mut Uc32: *mut WOW64_USERCONNECT32 =
        *a.offset(1 as isize) as ULONG_PTR as *mut WOW64_USERCONNECT32;
    let mut Size32: ULONG = *a.offset(2 as isize);
    let mut Uc64: WOW64_USERCONNECT64 = core::mem::zeroed();
    let mut Status: NTSTATUS;
    let mut i: ULONG;

    if Uc32.is_null() || Size32 as usize != core::mem::size_of::<WOW64_USERCONNECT32>() as usize {
        return 0xC000000Du32 as NTSTATUS; /* STATUS_INVALID_PARAMETER */
    }

    core::ptr::write_bytes(
        core::ptr::addr_of_mut!(Uc64) as *mut u8,
        0,
        core::mem::size_of::<WOW64_USERCONNECT64>(),
    );
    Uc64.ulVersion = (*Uc32).ulVersion;
    Uc64.ulCurrentVersion = (*Uc32).ulCurrentVersion;
    Uc64.dwDispatchCount = (*Uc32).dwDispatchCount;

    Status = NtUserProcessConnect(
        Process,
        (core::ptr::addr_of_mut!(Uc64) as PVOID).cast(),
        core::mem::size_of::<WOW64_USERCONNECT64>() as ULONG,
    );
    if Status >= 0 as NTSTATUS {
        (*Uc32).ulVersion = Uc64.ulVersion;
        (*Uc32).ulCurrentVersion = Uc64.ulCurrentVersion;
        (*Uc32).dwDispatchCount = Uc64.dwDispatchCount;
        if Uc64.siClient.psi > 0xFFFFFFFF as u64
            || Uc64.siClient.aheList > 0xFFFFFFFF as u64
        {
            DbgPrint(
                b"[WOW64] NtUserProcessConnect: SHARED PTRS ABOVE 4GB (psi=%I64x ahe=%I64x) \xE2\x80\x94 kernel low-map missing?!\n\0"
                    .as_ptr() as PCSTR,
                Uc64.siClient.psi,
                Uc64.siClient.aheList,
            );
        }
        (*Uc32).siClient.psi = Uc64.siClient.psi as ULONG;
        Wow64pPfnMapSetPsi(Uc64.siClient.psi);
        (*Uc32).siClient.aheList = Uc64.siClient.aheList as ULONG;
        (*Uc32).siClient.pDispInfo = Uc64.siClient.pDispInfo as ULONG;
        (*Uc32).siClient.ulSharedDelta = Uc64.siClient.ulSharedDelta as ULONG;
        i = 0 as ULONG;
        while i < WOW64_FNID_NUM as ULONG {
            (*Uc32).siClient.awmControl[i as usize].maxMsgs =
                Uc64.siClient.awmControl[i as usize].maxMsgs;
            (*Uc32).siClient.awmControl[i as usize].abMsgs =
                Uc64.siClient.awmControl[i as usize].abMsgs as ULONG;
            i = i.wrapping_add(1);
        }
        (*Uc32).siClient.DefWindowMsgs.maxMsgs = Uc64.siClient.DefWindowMsgs.maxMsgs;
        (*Uc32).siClient.DefWindowMsgs.abMsgs = Uc64.siClient.DefWindowMsgs.abMsgs as ULONG;
        (*Uc32).siClient.DefWindowSpecMsgs.maxMsgs = Uc64.siClient.DefWindowSpecMsgs.maxMsgs;
        (*Uc32).siClient.DefWindowSpecMsgs.abMsgs = Uc64.siClient.DefWindowSpecMsgs.abMsgs as ULONG;
        DbgPrint(
            b"[WOW64]   NtUserProcessConnect: psi=%08lx ahe=%08lx delta=%08lx\n\0".as_ptr()
                as PCSTR,
            (*Uc32).siClient.psi,
            (*Uc32).siClient.aheList,
            (*Uc32).siClient.ulSharedDelta,
        );
    }
    return Status;
}

unsafe extern "C" fn Wow64pMsg32To64(M32: *const WOW64_MSG32, M64: *mut WOW64_MSG64) {
    (*M64).hwnd = (*M32).hwnd as ULONGLONG; (*M64).message = (*M32).message; (*M64)._pad0 = 0 as ULONG;
    (*M64).wParam = (*M32).wParam as ULONGLONG; (*M64).lParam = (*M32).lParam as ULONGLONG;
    (*M64).time = (*M32).time; (*M64).ptx = (*M32).ptx; (*M64).pty = (*M32).pty;
    (*M64)._pad1 = 0 as ULONG;
}

unsafe extern "C" fn Wow64pMsg64To32(M64: *const WOW64_MSG64, M32: *mut WOW64_MSG32) {
    (*M32).hwnd = (*M64).hwnd as ULONG;
    (*M32).message = (*M64).message;
    (*M32).wParam = (*M64).wParam as ULONG;
    (*M32).lParam = (*M64).lParam as ULONG;
    (*M32).time = (*M64).time;
    (*M32).ptx = (*M64).ptx;
    (*M32).pty = (*M64).pty;
}

unsafe extern "C" fn Wow64Manual_NtUserDefSetText(a: *mut ULONG) -> NTSTATUS {
    let mut Ls64: WOW64_LARGE_STRING64 = core::mem::zeroed();
    let mut Ls64Ptr: PVOID = core::ptr::null_mut();
    if *a.add(1 as usize) != 0 {
        let Ls32: *const WOW64_LARGE_STRING32 =
            (*a.add(1 as usize) as ULONG_PTR) as *const WOW64_LARGE_STRING32;
        Ls64.Length = (*Ls32).Length;
        Ls64.MaximumLength = (*Ls32).MaximumLength; /* carries bAnsi in bit 31 */
        Ls64.Buffer = (*Ls32).Buffer as ULONGLONG;
        Ls64Ptr = core::ptr::addr_of_mut!(Ls64) as PVOID;
    }
    /* A NULL WindowText is legal — it clears the title. */
    return (NtUserDefSetText(
        ((*a.add(0 as usize) as ULONG_PTR) as PVOID).cast(),
        (Ls64Ptr).cast(),
    ) as ULONG) as NTSTATUS;
}

unsafe extern "C" fn Wow64Manual_NtUserBuildPropList(a: *mut ULONG) -> NTSTATUS {
    let _shape32: usize = core::mem::size_of::<ITEM32>();
    let _shape64: usize = core::mem::size_of::<ITEM64>();
    let Wnd: PVOID = *a.offset(0 as isize) as LONG as LONG_PTR as PVOID;
    let Out32: *mut ITEM32 =
        *a.offset(1 as isize) as ULONG_PTR as *mut ITEM32;
    let Size32: ULONG = *a.offset(2 as isize);
    let Count32: *mut ULONG =
        *a.offset(3 as isize) as ULONG_PTR as *mut ULONG;
    let mut Stack: [ITEM64; WOW64_PROPLIST_STACK] =
        [ITEM64 { Atom: 0, Data: 0 }; WOW64_PROPLIST_STACK];
    let mut Items: *mut ITEM64 = core::ptr::addr_of_mut!(Stack) as *mut ITEM64;
    let mut Heap: PVOID = core::ptr::null_mut();
    let mut Count: ULONG = 0 as ULONG;
    let mut n: ULONG = 0;
    let mut i: ULONG = 0;
    let mut St: NTSTATUS;

    C_ASSERT!(core::mem::size_of::<ITEM32>() == (WOW64_PROPLISTITEM32_CB as _));
    C_ASSERT!(core::mem::size_of::<ITEM64>() == (WOW64_PROPLISTITEM64_CB as _));

    if Out32.is_null() {
        St = NtUserBuildPropList(
            Wnd,
            core::ptr::null_mut(),
            0 as ULONG,
            core::ptr::addr_of_mut!(Count),
        );
        if St >= 0 as NTSTATUS && !Count32.is_null() {
            *Count32 = Count
                .wrapping_div(WOW64_PROPLISTITEM64_CB as ULONG)
                .wrapping_mul(WOW64_PROPLISTITEM32_CB as ULONG);
        }
        return St;
    }
    if Size32 == 0 as ULONG
        || Size32.wrapping_rem(WOW64_PROPLISTITEM32_CB as ULONG) != 0
    {
        return 0xC000000Du32 as NTSTATUS; /* STATUS_INVALID_PARAMETER, as the kernel says */
    }

    n = Size32.wrapping_div(WOW64_PROPLISTITEM32_CB as ULONG);
    if n > WOW64_PROPLIST_STACK as ULONG {
        let mut Size: SIZE_T = (n as SIZE_T).wrapping_mul(WOW64_PROPLISTITEM64_CB as SIZE_T);
        St = NtAllocateVirtualMemory(
            (-(1 as LONG_PTR)) as HANDLE,
            core::ptr::addr_of_mut!(Heap),
            0 as ULONG_PTR,
            core::ptr::addr_of_mut!(Size),
            (MEM_RESERVE | MEM_COMMIT) as ULONG,
            PAGE_READWRITE as ULONG,
        );
        if St < 0 as NTSTATUS {
            return St;
        }
        Items = Heap as *mut ITEM64;
    }

    St = NtUserBuildPropList(
        Wnd,
        Items as PVOID,
        n.wrapping_mul(WOW64_PROPLISTITEM64_CB as ULONG),
        core::ptr::addr_of_mut!(Count),
    );
    if St >= 0 as NTSTATUS {
        i = 0 as ULONG;
        while i < Count && i < n {
            (*Out32.offset(i as isize)).Atom = (*Items.offset(i as isize)).Atom;
            (*Out32.offset(i as isize)).Data = (*Items.offset(i as isize)).Data as ULONG; /* a handle-sized value */
            i = i.wrapping_add(1);
        }
        if !Count32.is_null() {
            *Count32 = Count;
        }
    }

    if !Heap.is_null() {
        let mut Free: SIZE_T = 0 as SIZE_T;
        NtFreeVirtualMemory(
            (-(1 as LONG_PTR)) as HANDLE,
            core::ptr::addr_of_mut!(Heap),
            core::ptr::addr_of_mut!(Free),
            MEM_RELEASE as ULONG,
        );
    }
    return St;
}

unsafe extern "C" fn Wow64Manual_NtUserSetWindowLong(a: *mut ULONG) -> NTSTATUS {
    let Wnd: PVOID = *a.add(0 as usize) as LONG as LONG_PTR as usize as PVOID; /* handle: sign-extend */

    if (*a.add(1 as usize)) as LONG == WOW64_GWL_WNDPROC {
        return Wow64pMapClientProc(NtUserSetWindowLongPtr(
            (Wnd).cast(),
            *a.add(1 as usize),
            Wow64pUnmapClientProc(*a.add(2 as usize)) as LONG_PTR,
            (*a.add(3 as usize)) as i32,
        ) as ULONGLONG) as NTSTATUS;
    }
    return NtUserSetWindowLong(
        (Wnd).cast(),
        *a.add(1 as usize),
        (*a.add(2 as usize)) as LONG,
        (*a.add(3 as usize)) as i32,
    ) as NTSTATUS;
}

unsafe extern "C" fn Wow64Manual_NtUserSetClassLong(mut a: *mut ULONG) -> NTSTATUS {
    let mut Wnd: PVOID = *a.offset(0 as isize) as LONG as LONG_PTR as PVOID;
    if *a.offset(1 as isize) as LONG == WOW64_GCL_WNDPROC {
        return Wow64pMapClientProc(NtUserSetClassLongPtr(
            (Wnd).cast(),
            *a.offset(1 as isize) as INT,
            Wow64pUnmapClientProc(*a.offset(2 as isize)) as ULONG_PTR,
            (*a.offset(3 as isize)) as i32,
        ) as ULONGLONG) as NTSTATUS;
    }
    return NtUserSetClassLong(
        (Wnd).cast(),
        *a.offset(1 as isize) as INT,
        *a.offset(2 as isize),
        (*a.offset(3 as isize)) as i32,
    ) as NTSTATUS;
}

unsafe extern "C" fn Wow64Manual_NtUserGetCPD(a: *mut ULONG) -> NTSTATUS {
    let Wnd: PVOID = (*a.offset(0 as isize)) as LONG as LONG_PTR as PVOID;

    return (Wow64pMapClientProc(
        (NtUserGetCPD(
            (Wnd).cast(),
            *a.offset(1 as isize),
            (Wow64pUnmapClientProc(*a.offset(2 as isize))) as ULONG_PTR,
        )) as ULONGLONG,
    )) as NTSTATUS;
}

unsafe extern "C" fn Wow64Manual_NtUserRegisterClassExWOW(a: *mut ULONG) -> NTSTATUS {
    let mut W32: *const WC32 =
        *a.add(0 as usize) as ULONG_PTR as *const WC32;
    let mut CN32: *mut WOW64_UNICODE_STRING32_M =
        *a.add(1 as usize) as ULONG_PTR as *mut WOW64_UNICODE_STRING32_M;
    let mut CV32: *mut WOW64_UNICODE_STRING32_M =
        *a.add(2 as usize) as ULONG_PTR as *mut WOW64_UNICODE_STRING32_M;
    let mut M32: *const CMN32 =
        *a.add(3 as usize) as ULONG_PTR as *const CMN32;
    let mut W64: WC64 = core::mem::zeroed();
    let mut CN64: WOW64_UNICODE_STRING64_M = core::mem::zeroed();
    let mut CV64: WOW64_UNICODE_STRING64_M = core::mem::zeroed();
    let mut MenuUs64: WOW64_UNICODE_STRING64_M = core::mem::zeroed();
    let mut M64: CMN64 = core::mem::zeroed();
    let mut CN64p: PVOID = core::ptr::null_mut();
    let mut CV64p: PVOID = core::ptr::null_mut();
    let mut M64p: PVOID = core::ptr::null_mut();

    if W32.is_null() {
        return 0 as NTSTATUS; // ATOM 0 = fail
    }
    W64.cbSize = core::mem::size_of_val(&W64) as ULONG;
    W64.style = (*W32).style;
    // A guest system proc becomes the host's (wow64_pfnmap.c): win32k picks an
    // ANSI window's A proc by comparing the class proc against gpsi.
    W64.lpfnWndProc =
        Wow64pUnmapClientProc((*W32).lpfnWndProc);
    W64.cbClsExtra = (*W32).cbClsExtra;
    W64.cbWndExtra = (*W32).cbWndExtra;
    W64.hInstance = (*W32).hInstance as ULONGLONG;
    W64.hIcon = (*W32).hIcon as ULONGLONG;
    W64.hCursor = (*W32).hCursor as ULONGLONG;
    W64.hbrBackground = (*W32).hbrBackground as ULONGLONG;
    W64.lpszMenuName = (*W32).lpszMenuName as ULONGLONG;
    W64.lpszClassName = (*W32).lpszClassName as ULONGLONG;
    W64.hIconSm = (*W32).hIconSm as ULONGLONG;

    if !CN32.is_null() {
        CN64.Length = (*CN32).Length;
        CN64.MaximumLength = (*CN32).MaximumLength;
        CN64.__pad = 0 as ULONG;
        CN64.Buffer = (*CN32).Buffer as ULONGLONG;
        CN64p = core::ptr::addr_of_mut!(CN64) as PVOID;
    }
    if !CV32.is_null() {
        CV64.Length = (*CV32).Length;
        CV64.MaximumLength = (*CV32).MaximumLength;
        CV64.__pad = 0 as ULONG;
        CV64.Buffer = (*CV32).Buffer as ULONGLONG;
        CV64p = core::ptr::addr_of_mut!(CV64) as PVOID;
    }
    if !M32.is_null() {
        M64.pszClientAnsiMenuName =
            (*M32).pszClientAnsiMenuName as ULONGLONG;
        M64.pwszClientUnicodeMenuName =
            (*M32).pwszClientUnicodeMenuName as ULONGLONG;
        M64.pusMenuName = 0 as ULONGLONG;
        if (*M32).pusMenuName != 0 {
            let mut MU32: *mut WOW64_UNICODE_STRING32_M =
                (*M32).pusMenuName as ULONG_PTR as *mut WOW64_UNICODE_STRING32_M;
            MenuUs64.Length = (*MU32).Length;
            MenuUs64.MaximumLength = (*MU32).MaximumLength;
            MenuUs64.__pad = 0 as ULONG;
            MenuUs64.Buffer = (*MU32).Buffer as ULONGLONG;
            M64.pusMenuName =
                core::ptr::addr_of_mut!(MenuUs64) as ULONG_PTR as ULONGLONG;
        }
        M64p = core::ptr::addr_of_mut!(M64) as PVOID;
    }

    return NtUserRegisterClassExWOW(
        (core::ptr::addr_of_mut!(W64) as PVOID).cast(),
        (CN64p).cast(),
        (CV64p).cast(),
        (M64p).cast(),
        *a.add(4 as usize),
        *a.add(5 as usize),
        (*a.add(6 as usize) as ULONG_PTR as PVOID).cast(),
    ) as NTSTATUS;
}

unsafe extern "C" fn Wow64Manual_NtUserGetAtomName(a: *mut ULONG) -> NTSTATUS {
    let mut N32: *mut WOW64_UNICODE_STRING32_M = *a.offset(1 as isize) as ULONG_PTR as *mut WOW64_UNICODE_STRING32_M;
    let mut N64: WOW64_UNICODE_STRING64_M = WOW64_UNICODE_STRING64_M {
        Length: 0,
        MaximumLength: 0,
        __pad: 0,
        Buffer: 0,
    };
    let mut Ret: ULONG = 0;

    if (N32).is_null() {
        return 0 as NTSTATUS;
    }
    (N64).Length = (*N32).Length;
    (N64).MaximumLength = (*N32).MaximumLength;
    (N64).__pad = 0 as ULONG;
    (N64).Buffer = (*N32).Buffer as ULONG_PTR as ULONGLONG; /* guest buffer, kernel writes here */
    Ret = NtUserGetAtomName(
        (*a.offset(0 as isize) as ULONG_PTR) as ATOM,
        (core::ptr::addr_of_mut!(N64) as PVOID).cast(),
    );
    (*N32).Length = (N64).Length; /* kernel updated the byte count written */
    return Ret as NTSTATUS;
}

unsafe extern "C" fn Wow64Manual_NtUserGetClassName(a: *mut ULONG) -> NTSTATUS {
    let mut N32: *mut WOW64_UNICODE_STRING32_M =
        *a.offset(2 as isize) as ULONG_PTR as *mut WOW64_UNICODE_STRING32_M;
    let mut N64: WOW64_UNICODE_STRING64_M = WOW64_UNICODE_STRING64_M {
        Length: 0,
        MaximumLength: 0,
        __pad: 0,
        Buffer: 0,
    };
    let mut Ret: ULONG = 0;

    if N32.is_null() {
        return 0 as NTSTATUS;
    }
    N64.Length = (*N32).Length;
    N64.MaximumLength = (*N32).MaximumLength;
    N64.__pad = 0 as ULONG;
    N64.Buffer = (*N32).Buffer as ULONG_PTR as ULONGLONG;
    Ret = (NtUserGetClassName(
        (*a.offset(0 as isize) as ULONG_PTR as PVOID).cast(),
        (*a.offset(1 as isize)) as i32,
        (core::ptr::addr_of_mut!(N64) as PVOID).cast(),
    )) as u32;
    (*N32).Length = N64.Length;
    return Ret as NTSTATUS;
}

unsafe extern "C" fn Wow64Manual_NtUserRegisterWindowMessage(a: *mut ULONG) -> NTSTATUS {
    let mut N32: *mut WOW64_UNICODE_STRING32_M =
        *a.offset(0 as isize) as ULONG_PTR as *mut WOW64_UNICODE_STRING32_M;
    let mut N64: WOW64_UNICODE_STRING64_M = WOW64_UNICODE_STRING64_M {
        Length: 0,
        MaximumLength: 0,
        __pad: 0,
        Buffer: 0,
    };

    if N32.is_null() {
        return 0;
    }
    N64.Length = (*N32).Length;
    N64.MaximumLength = (*N32).MaximumLength;
    N64.__pad = 0 as ULONG;
    N64.Buffer = (*N32).Buffer as ULONG_PTR as ULONGLONG; /* guest name buffer (read) */
    return NtUserRegisterWindowMessage((core::ptr::addr_of_mut!(N64) as PVOID).cast()) as NTSTATUS;
}

unsafe extern "C" fn Wow64Manual_NtUserGetKeyboardLayoutName(a: *mut ULONG) -> NTSTATUS {
    let mut N32: *mut WOW64_UNICODE_STRING32_M =
        *a.offset(0 as isize) as ULONG_PTR as *mut WOW64_UNICODE_STRING32_M;
    let mut N64: WOW64_UNICODE_STRING64_M = WOW64_UNICODE_STRING64_M {
        Length: 0,
        MaximumLength: 0,
        __pad: 0,
        Buffer: 0,
    };
    let mut Ret: ULONG = 0;

    if N32.is_null() {
        return 0 as NTSTATUS;
    }
    N64.Length = (*N32).Length;
    N64.MaximumLength = (*N32).MaximumLength;
    N64.__pad = 0 as ULONG;
    N64.Buffer = (*N32).Buffer as ULONG_PTR as ULONGLONG; /* guest buffer, kernel writes here */
    Ret = (NtUserGetKeyboardLayoutName((core::ptr::addr_of_mut!(N64) as PVOID).cast())) as u32;
    (*N32).Length = N64.Length;
    return Ret as NTSTATUS;
}

unsafe extern "C" fn Wow64Manual_NtUserBuildHwndList(a: *mut ULONG) -> NTSTATUS {
    let hDesktop: HANDLE = *a.offset(0 as isize) as ULONG_PTR as HANDLE;
    let hParent: HANDLE = *a.offset(1 as isize) as ULONG_PTR as HANDLE;
    let bChild: BOOLEAN = *a.offset(2 as isize) as BOOLEAN;
    let dwTid: ULONG = *a.offset(3 as isize);
    let cHwnd: ULONG = *a.offset(4 as isize);
    let pList32: *mut ULONG = *a.offset(5 as isize) as ULONG_PTR as *mut ULONG;
    let pNeeded: PULONG = *a.offset(6 as isize) as ULONG_PTR as PULONG;
    let mut Tmp: PVOID = NULL;
    let mut Size: SIZE_T = 0;
    let mut St: NTSTATUS = 0;
    let mut n: ULONG = 0;
    let mut i: ULONG = 0;

    if pList32.is_null() || cHwnd == 0 as ULONG {
        return NtUserBuildHwndList(
            (hDesktop).cast(),
            (hParent).cast(),
            bChild,
            dwTid,
            cHwnd,
            (*a.offset(5 as isize) as ULONG_PTR as PVOID).cast(),
            pNeeded,
        );
    }

    Size = (cHwnd as SIZE_T).wrapping_mul(core::mem::size_of::<PVOID>() as SIZE_T);
    St = NtAllocateVirtualMemory(
        (-(1 as LONG_PTR)) as HANDLE,
        core::ptr::addr_of_mut!(Tmp),
        0 as ULONG_PTR,
        core::ptr::addr_of_mut!(Size),
        (MEM_RESERVE | MEM_COMMIT) as ULONG,
        PAGE_READWRITE as ULONG,
    );
    if St < 0 as NTSTATUS {
        return St;
    }

    St = NtUserBuildHwndList((hDesktop).cast(), (hParent).cast(), bChild, dwTid, cHwnd, (Tmp).cast(), pNeeded);
    if St >= 0 as NTSTATUS {
        n = if !pNeeded.is_null() {
            *pNeeded
        } else {
            0 as ULONG
        };
        if n > cHwnd {
            n = cHwnd;
        }
        i = 0 as ULONG;
        while i < n {
            let elem_ptr = (Tmp as *const u8)
                .add(i as usize * (core::mem::size_of::<ULONGLONG>() as usize)) as *const ULONGLONG;
            *pList32.offset(i as isize) = *elem_ptr as ULONG;
            i = i.wrapping_add(1);
        }
    }

    {
        let mut Free: SIZE_T = 0 as SIZE_T;
        NtFreeVirtualMemory(
            (-(1 as LONG_PTR)) as HANDLE,
            core::ptr::addr_of_mut!(Tmp),
            core::ptr::addr_of_mut!(Free),
            MEM_RELEASE as ULONG,
        );
    }
    return St;
}

unsafe extern "C" fn Wow64Manual_NtUserGetClassInfo(a: *mut ULONG) -> NTSTATUS {
    let mut CN32: *mut WOW64_UNICODE_STRING32_M =
        *a.offset(1 as isize) as ULONG_PTR as *mut WOW64_UNICODE_STRING32_M;
    let mut W32: *mut WC32 =
        *a.offset(2 as isize) as ULONG_PTR as *mut WC32;
    let mut MenuName32: *mut ULONG =
        *a.offset(3 as isize) as ULONG_PTR as *mut ULONG;
    let mut CN64: WOW64_UNICODE_STRING64_M = WOW64_UNICODE_STRING64_M {
        Length: 0,
        MaximumLength: 0,
        __pad: 0,
        Buffer: 0,
    };
    let mut CN64p: PVOID = NULL;
    let mut W64: WC64 = WC64 {
        cbSize: 0,
        style: 0,
        lpfnWndProc: 0,
        cbClsExtra: 0,
        cbWndExtra: 0,
        hInstance: 0,
        hIcon: 0,
        hCursor: 0,
        hbrBackground: 0,
        lpszMenuName: 0,
        lpszClassName: 0,
        hIconSm: 0,
    };
    let mut MenuName64: ULONGLONG = 0 as ULONGLONG;
    let mut Ret: ULONG = 0;

    // C: `if (!W32) return 0;`. The translation had the test inverted, so every
    // WoW64 GetClassInfo(Ex) with a buffer (i.e. every real call) returned FALSE;
    // user32:edit test_initialization then registered a garbage class and crashed.
    if W32.is_null() {
        return 0 as NTSTATUS;
    }
    if !CN32.is_null() {
        /* Buffer may be a raw atom (< 0x10000) — zero-extension keeps it one. */
        CN64.Length = (*CN32).Length;
        CN64.MaximumLength = (*CN32).MaximumLength;
        CN64.__pad = 0 as ULONG;
        CN64.Buffer = (*CN32).Buffer as ULONGLONG;
        CN64p = &raw mut CN64 as PVOID;
    }
    W64.cbSize = core::mem::size_of_val(&W64) as ULONG;
    W64.style = (*W32).style;
    W64.lpfnWndProc = (*W32).lpfnWndProc as ULONGLONG;
    W64.cbClsExtra = (*W32).cbClsExtra;
    W64.cbWndExtra = (*W32).cbWndExtra;
    W64.hInstance = (*W32).hInstance as ULONGLONG;
    W64.hIcon = (*W32).hIcon as ULONGLONG;
    W64.hCursor = (*W32).hCursor as ULONGLONG;
    W64.hbrBackground = (*W32).hbrBackground as ULONGLONG;
    W64.lpszMenuName = (*W32).lpszMenuName as ULONGLONG;
    W64.lpszClassName = (*W32).lpszClassName as ULONGLONG;
    W64.hIconSm = (*W32).hIconSm as ULONGLONG;

    Ret = (NtUserGetClassInfo(
        (*a.offset(0 as isize) as ULONG_PTR as PVOID).cast(),
        (CN64p).cast(),
        (&raw mut W64 as PVOID).cast(),
        (&raw mut MenuName64 as PVOID).cast(),
        (*a.offset(4 as isize)) as i32,
    )) as u32;
    if Ret != 0 {
        /* Narrow back. A class whose members came from 64-bit code (a system
         * class wndproc, a >4GB menu-name mapping) cannot be represented for
         * the guest — trace it loudly instead of silently truncating. */
        if ((W64.lpszMenuName | W64.lpszClassName | MenuName64)
            >> 32)
            != 0
            || ((W64.lpfnWndProc >> 32) != 0
                && (Wow64pMapClientProc(W64.lpfnWndProc)
                    == (W64.lpfnWndProc as ULONG)))
        {
            DbgPrint(
                b"[WOW64] NtUserGetClassInfo: 64-bit-only members (proc=%I64x menu=%I64x cls=%I64x pmenu=%I64x) TRUNCATED\n\0"
                    .as_ptr() as PCSTR,
                W64.lpfnWndProc,
                W64.lpszMenuName,
                W64.lpszClassName,
                MenuName64,
            );
        }
        (*W32).style = W64.style;
        (*W32).lpfnWndProc =
            Wow64pMapClientProc(W64.lpfnWndProc) as ULONG;
        (*W32).cbClsExtra = W64.cbClsExtra;
        (*W32).cbWndExtra = W64.cbWndExtra;
        (*W32).hInstance = W64.hInstance as ULONG;
        (*W32).hIcon = W64.hIcon as ULONG;
        (*W32).hCursor = W64.hCursor as ULONG;
        (*W32).hbrBackground = W64.hbrBackground as ULONG;
        (*W32).lpszMenuName = W64.lpszMenuName as ULONG;
        (*W32).lpszClassName = W64.lpszClassName as ULONG;
        (*W32).hIconSm = W64.hIconSm as ULONG;
        if !MenuName32.is_null() {
            *MenuName32 = MenuName64 as ULONG;
        }
    }
    return Ret as NTSTATUS;
}

unsafe extern "C" fn Wow64Manual_NtUserThunkedMenuItemInfo(a: *mut ULONG) -> NTSTATUS {
    let _Sz32 = core::mem::size_of::<MII32>();
    let _Sz64 = core::mem::size_of::<MII64>();
    let M32: *const MII32 =
        *a.offset(4 as isize) as ULONG_PTR as *const MII32;
    let mut Cap32: *mut WOW64_UNICODE_STRING32_M =
        *a.offset(5 as isize) as ULONG_PTR as *mut WOW64_UNICODE_STRING32_M;
    let mut M64: MII64 = core::mem::zeroed();
    let mut Cap64: WOW64_UNICODE_STRING64_M = core::mem::zeroed();
    let mut M64p: PVOID = core::ptr::null_mut();
    let mut Cap64p: PVOID = core::ptr::null_mut();

    if !M32.is_null() {
        M64.cbSize = core::mem::size_of_val(&M64) as ULONG;
        M64.fMask = (*M32).fMask;
        M64.fType = (*M32).fType;
        M64.fState = (*M32).fState;
        M64.wID = (*M32).wID;
        M64.__pad0 = 0 as ULONG;
        M64.hSubMenu = (*M32).hSubMenu as ULONGLONG;
        M64.hbmpChecked = (*M32).hbmpChecked as ULONGLONG;
        M64.hbmpUnchecked = (*M32).hbmpUnchecked as ULONGLONG;
        M64.dwItemData = (*M32).dwItemData as ULONGLONG;
        M64.dwTypeData = (*M32).dwTypeData as ULONGLONG;
        M64.cch = (*M32).cch;
        M64.__pad1 = 0 as ULONG;
        M64.hbmpItem = (*M32).hbmpItem as ULONGLONG;
        M64p = core::ptr::addr_of_mut!(M64) as PVOID;
    }
    if !Cap32.is_null() {
        Cap64.Length = (*Cap32).Length;
        Cap64.MaximumLength = (*Cap32).MaximumLength;
        Cap64.__pad = 0 as ULONG;
        Cap64.Buffer = (*Cap32).Buffer as ULONGLONG;
        Cap64p = core::ptr::addr_of_mut!(Cap64) as PVOID;
    }
    return NtUserThunkedMenuItemInfo(
        (*a.offset(0 as isize) as ULONG_PTR as PVOID).cast(),
        *a.offset(1 as isize),
        (*a.offset(2 as isize)) as i32,
        (*a.offset(3 as isize)) as i32,
        (M64p).cast(),
        (Cap64p).cast(),
    ) as NTSTATUS;
}

unsafe extern "C" fn Wow64Manual_NtUserCreateWindowEx(a: *mut ULONG) -> NTSTATUS {
    // The C's own block-scope prototype (handles as ULONGLONG), not the SDK's.
    extern "system" {
        fn NtUserCreateWindowEx(
            a0: ULONG, a1: PVOID, a2: PVOID, a3: PVOID, a4: ULONG, a5: LONG, a6: LONG,
            a7: LONG, a8: LONG, a9: ULONGLONG, a10: ULONGLONG, a11: ULONGLONG,
            a12: ULONGLONG, a13: ULONG, a14: PVOID,
        ) -> ULONGLONG;
    }
    let mut Cls64: WOW64_LARGE_STRING64 = core::mem::zeroed();
    let mut Ver64: WOW64_LARGE_STRING64 = core::mem::zeroed();
    let mut Wnd64: WOW64_LARGE_STRING64 = core::mem::zeroed();
    let mut Cls: PVOID = core::ptr::null_mut();
    let mut Ver: PVOID = core::ptr::null_mut();
    let mut Wnd: PVOID = core::ptr::null_mut();

    {
        let mut _v: ULONG = *a.offset(1 as isize);
        if _v != 0 && _v >= 0x10000 {
            let _s32: *const WOW64_LARGE_STRING32 = _v as ULONG_PTR as *const WOW64_LARGE_STRING32;
            Cls64.Length = (*_s32).Length;
            Cls64.MaximumLength = (*_s32).MaximumLength;
            Cls64.Buffer = (*_s32).Buffer as ULONGLONG;
            Cls = core::ptr::addr_of_mut!(Cls64) as PVOID;
        } else if _v != 0 {
            Cls = _v as ULONG_PTR as PVOID;
        }
    }
    {
        let mut _v: ULONG = *a.offset(2 as isize);
        if _v != 0 && _v >= 0x10000 {
            let _s32: *const WOW64_LARGE_STRING32 = _v as ULONG_PTR as *const WOW64_LARGE_STRING32;
            Ver64.Length = (*_s32).Length;
            Ver64.MaximumLength = (*_s32).MaximumLength;
            Ver64.Buffer = (*_s32).Buffer as ULONGLONG;
            Ver = core::ptr::addr_of_mut!(Ver64) as PVOID;
        } else if _v != 0 {
            Ver = _v as ULONG_PTR as PVOID;
        }
    }
    {
        let mut _v: ULONG = *a.offset(3 as isize);
        if _v != 0 && _v >= 0x10000 {
            let _s32: *const WOW64_LARGE_STRING32 = _v as ULONG_PTR as *const WOW64_LARGE_STRING32;
            Wnd64.Length = (*_s32).Length;
            Wnd64.MaximumLength = (*_s32).MaximumLength;
            Wnd64.Buffer = (*_s32).Buffer as ULONGLONG;
            Wnd = core::ptr::addr_of_mut!(Wnd64) as PVOID;
        } else if _v != 0 {
            Wnd = _v as ULONG_PTR as PVOID;
        }
    }

    return (NtUserCreateWindowEx(
        *a.offset(0 as isize),
        Cls,
        Ver,
        Wnd,
        *a.offset(4 as isize),
        *a.offset(5 as isize) as LONG,
        *a.offset(6 as isize) as LONG,
        *a.offset(7 as isize) as LONG,
        *a.offset(8 as isize) as LONG,
        *a.offset(9 as isize) as ULONGLONG,
        *a.offset(10 as isize) as ULONGLONG,
        *a.offset(11 as isize) as ULONGLONG,
        *a.offset(12 as isize) as ULONGLONG,
        *a.offset(13 as isize),
        *a.offset(14 as isize) as ULONG_PTR as PVOID,
    ) as ULONG) as NTSTATUS;
}

unsafe extern "C" fn Wow64Manual_NtUserGetMessage(a: *mut ULONG) -> NTSTATUS {
    let mut M32: *mut WOW64_MSG32 =
        *a.add(0 as usize) as ULONG_PTR as *mut WOW64_MSG32;
    let mut M64: WOW64_MSG64 = core::mem::zeroed();
    let mut Ret: LONG = 0;

    if M32.is_null() {
        return 0;
    }
    Ret = NtUserGetMessage(
        (core::ptr::addr_of_mut!(M64) as PVOID).cast(),
        ((*a.add(1 as usize) as ULONGLONG) as *mut HWND__),
        *a.add(2 as usize),
        *a.add(3 as usize),
    );
    Wow64pMsg64To32(core::ptr::addr_of_mut!(M64), M32);
    return Ret;
}

unsafe extern "C" fn Wow64Manual_NtUserPeekMessage(a: *mut ULONG) -> NTSTATUS {
    let mut M32: *mut WOW64_MSG32 =
        *a.add(0 as usize) as ULONG_PTR as *mut WOW64_MSG32;
    let mut M64: WOW64_MSG64 = core::mem::zeroed();
    let mut Ret: LONG;

    if M32.is_null() {
        return 0;
    }
    Ret = NtUserPeekMessage(
        (core::ptr::addr_of_mut!(M64) as PVOID).cast(),
        ((*a.add(1 as usize) as ULONGLONG) as *mut HWND__),
        *a.add(2 as usize),
        *a.add(3 as usize),
        *a.add(4 as usize),
    );
    if Ret != 0 {
        Wow64pMsg64To32(
            core::ptr::addr_of_mut!(M64),
            M32,
        );
    }
    return Ret as NTSTATUS;
}

unsafe extern "C" fn Wow64Manual_NtUserDispatchMessage(a: *mut ULONG) -> NTSTATUS {
    let mut M32: *mut WOW64_MSG32 =
        *a.add(0 as usize) as ULONG_PTR as *mut WOW64_MSG32;
    let mut M64: WOW64_MSG64 = core::mem::zeroed();

    if M32.is_null() {
        return 0;
    }
    Wow64pMsg32To64(
        M32,
        core::ptr::addr_of_mut!(M64),
    );
    return (NtUserDispatchMessage(
        (core::ptr::addr_of_mut!(M64) as PVOID).cast(),
    ) as ULONG) as NTSTATUS;
}

unsafe extern "C" fn Wow64Manual_NtUserTranslateMessage(a: *mut ULONG) -> NTSTATUS {
    let M32: *mut WOW64_MSG32 =
        *a.add(0 as usize) as ULONG_PTR as *mut WOW64_MSG32;
    let mut M64: WOW64_MSG64 = core::mem::zeroed();

    if M32.is_null() {
        return 0;
    }
    Wow64pMsg32To64(M32, core::ptr::addr_of_mut!(M64));
    return NtUserTranslateMessage(
        (core::ptr::addr_of_mut!(M64) as PVOID).cast(),
        *a.add(1 as usize),
    );
}

unsafe extern "C" fn Wow64Manual_NtUserBeginPaint(a: *mut ULONG) -> NTSTATUS {
    // The C's own block-scope prototype.
    extern "system" {
        fn NtUserBeginPaint(a0: ULONGLONG, a1: PVOID) -> ULONGLONG;
    }
    let mut P32: *mut WOW64_PAINTSTRUCT32 =
        *a.add(1 as usize) as ULONG_PTR as *mut WOW64_PAINTSTRUCT32;
    let mut P64: WOW64_PAINTSTRUCT64 = core::mem::zeroed();
    let mut Hdc: ULONGLONG = 0;
    let mut i: i32;

    if (P32).is_null() {
        return 0 as NTSTATUS;
    }
    Hdc = NtUserBeginPaint(*a.add(0 as usize) as ULONGLONG, core::ptr::addr_of_mut!(P64) as PVOID);
    (*P32).hdc = P64.hdc as ULONG;
    (*P32).fErase = P64.fErase;
    i = 0 as i32;
    while i < 4 as i32 {
        (*P32).rc[i as usize] = P64.rc[i as usize];
        i += 1;
    }
    (*P32).fRestore = P64.fRestore;
    (*P32).fIncUpdate = P64.fIncUpdate;
    i = 0 as i32;
    while i < 32 as i32 {
        (*P32).rgb[i as usize] = P64.rgb[i as usize];
        i += 1;
    }
    return Hdc as ULONG as NTSTATUS;
}

unsafe extern "C" fn Wow64Manual_NtUserEndPaint(a: *mut ULONG) -> NTSTATUS {
    let mut P32: *mut WOW64_PAINTSTRUCT32 =
        *a.offset(1 as isize) as ULONG_PTR as *mut WOW64_PAINTSTRUCT32;
    let mut P64: WOW64_PAINTSTRUCT64 = core::mem::zeroed();
    let mut i: i32 = 0;

    if P32.is_null() {
        return 0;
    }
    P64.hdc = (*P32).hdc as ULONGLONG;
    P64.fErase = (*P32).fErase;
    i = 0;
    while i < 4 {
        P64.rc[i as usize] = (*P32).rc[i as usize];
        i += 1;
    }
    P64.fRestore = (*P32).fRestore;
    P64.fIncUpdate = (*P32).fIncUpdate;
    i = 0;
    while i < 32 {
        P64.rgb[i as usize] = (*P32).rgb[i as usize];
        i += 1;
    }
    P64._pad = 0;
    return NtUserEndPaint(
        ((*a.offset(0 as isize) as ULONGLONG) as *mut HWND__),
        ((core::ptr::addr_of_mut!(P64) as PVOID) as *const tagPAINTSTRUCT),
    ) as NTSTATUS;
}

unsafe extern "C" fn Wow64pUS32to64(
    mut Guest32: ULONG,
    mut Out: *mut WOW64_UNICODE_STRING64_M,
) -> *mut WOW64_UNICODE_STRING64_M {
    let mut S32: *mut WOW64_UNICODE_STRING32_M =
        Guest32 as ULONG_PTR as *mut WOW64_UNICODE_STRING32_M;
    if (S32).is_null() {
        return ::core::ptr::null_mut::<WOW64_UNICODE_STRING64_M>();
    }
    (*Out).Length = (*S32).Length;
    (*Out).MaximumLength = (*S32).MaximumLength;
    (*Out).__pad = 0;
    (*Out).Buffer = (*S32).Buffer as ULONGLONG;
    return Out;
}

unsafe extern "C" fn Wow64Manual_NtUserUnregisterClass(a: *mut ULONG) -> NTSTATUS {
    let mut Name64: WOW64_UNICODE_STRING64_M = core::mem::zeroed();
    return NtUserUnregisterClass(
        (Wow64pUS32to64(*a.offset(0 as isize), core::ptr::addr_of_mut!(Name64)) as PVOID).cast(),
        (*a.offset(1 as isize) as ULONG_PTR as PVOID).cast(),
        (*a.offset(2 as isize) as ULONG_PTR as PVOID).cast(),
    ) as NTSTATUS;
}

unsafe extern "C" fn Wow64Manual_NtUserFindWindowEx(a: *mut ULONG) -> NTSTATUS {
    let mut Cls64: WOW64_UNICODE_STRING64_M = core::mem::zeroed();
    let mut Win64: WOW64_UNICODE_STRING64_M = core::mem::zeroed();
    let Ret: HANDLE = (NtUserFindWindowEx(
        (*a.add(0 as usize) as ULONG_PTR as PVOID).cast(),
        (*a.add(1 as usize) as ULONG_PTR as PVOID).cast(),
        (Wow64pUS32to64(*a.add(2 as usize), core::ptr::addr_of_mut!(Cls64)) as PVOID).cast(),
        (Wow64pUS32to64(*a.add(3 as usize), core::ptr::addr_of_mut!(Win64)) as PVOID).cast(),
        *a.add(4 as usize),
    )).cast();
    return (Ret as ULONG_PTR as ULONG) as NTSTATUS;
}

unsafe extern "C" fn Wow64Manual_NtUserGetWOWClass(a: *mut ULONG) -> NTSTATUS {
    let mut N64: WOW64_UNICODE_STRING64_M = WOW64_UNICODE_STRING64_M {
        Length: 0,
        MaximumLength: 0,
        __pad: 0,
        Buffer: 0,
    };
    let Ret: PVOID = (NtUserGetWOWClass(
        (*a.offset(0 as isize) as ULONG_PTR as PVOID).cast(),
        (Wow64pUS32to64(
            *a.offset(1 as isize),
            &raw mut N64,
        ) as PVOID).cast(),
    )).cast();
    return Ret as ULONG_PTR as ULONG as NTSTATUS;
}

unsafe extern "C" fn Wow64Manual_NtUserSetWindowsHookEx(a: *mut ULONG) -> NTSTATUS {
    let mut N64: WOW64_UNICODE_STRING64_M = core::mem::zeroed();
    let Ret: PVOID = (NtUserSetWindowsHookEx(
        (*a.add(0 as usize) as ULONG_PTR as PVOID).cast(),
        (Wow64pUS32to64(
            *a.add(1 as usize),
            core::ptr::addr_of_mut!(N64),
        ) as PVOID).cast(),
        *a.add(2 as usize),
        *a.add(3 as usize) as INT,
        core::mem::transmute::<*mut c_void, _>(*a.add(4 as usize) as ULONG_PTR as PVOID),
        (*a.add(5 as usize)) as i32,
    )).cast();
    return Ret as ULONG_PTR as ULONG as NTSTATUS;
}

unsafe extern "C" fn Wow64Manual_NtUserSetWinEventHook(a: *mut ULONG) -> NTSTATUS {
    let mut N64: WOW64_UNICODE_STRING64_M = core::mem::zeroed();
    let Ret: PVOID = (NtUserSetWinEventHook(
        *a.offset(0 as isize),
        *a.offset(1 as isize),
        (*a.offset(2 as isize) as ULONG_PTR as PVOID).cast(),
        (Wow64pUS32to64(
            *a.offset(3 as isize),
            core::ptr::addr_of_mut!(N64),
        ) as PVOID).cast(),
        core::mem::transmute::<*mut c_void, _>(*a.offset(4 as isize) as ULONG_PTR as PVOID),
        *a.offset(5 as isize),
        *a.offset(6 as isize),
        *a.offset(7 as isize),
    )).cast();
    return Ret as ULONG_PTR as ULONG as NTSTATUS;
}

unsafe extern "C" fn Wow64Manual_NtUserRegisterUserApiHook(a: *mut ULONG) -> NTSTATUS {
    let mut D64: WOW64_UNICODE_STRING64_M = WOW64_UNICODE_STRING64_M {
        Length: 0,
        MaximumLength: 0,
        __pad: 0,
        Buffer: 0,
    };
    let mut F64: WOW64_UNICODE_STRING64_M = WOW64_UNICODE_STRING64_M {
        Length: 0,
        MaximumLength: 0,
        __pad: 0,
        Buffer: 0,
    };
    return NtUserRegisterUserApiHook(
        (Wow64pUS32to64(
            *a.offset(0 as isize),
            core::ptr::addr_of_mut!(D64),
        ) as PVOID).cast(),
        (Wow64pUS32to64(
            *a.offset(1 as isize),
            core::ptr::addr_of_mut!(F64),
        ) as PVOID).cast(),
        *a.offset(2 as isize),
        *a.offset(3 as isize),
    ) as NTSTATUS;
}

unsafe extern "C" fn Wow64Manual_NtUserDrawCaptionTemp(a: *mut ULONG) -> NTSTATUS {
    let mut S64: WOW64_UNICODE_STRING64_M = WOW64_UNICODE_STRING64_M {
        Length: 0,
        MaximumLength: 0,
        __pad: 0,
        Buffer: 0,
    };
    return NtUserDrawCaptionTemp(
        *a.offset(0 as isize) as ULONG_PTR as PVOID,
        *a.offset(1 as isize) as ULONG_PTR as PVOID,
        *a.offset(2 as isize) as ULONG_PTR as PVOID,
        *a.offset(3 as isize) as ULONG_PTR as PVOID,
        *a.offset(4 as isize) as ULONG_PTR as PVOID,
        Wow64pUS32to64(
            *a.offset(5 as isize),
            core::ptr::addr_of_mut!(S64),
        ) as PVOID,
        *a.offset(6 as isize),
    ) as NTSTATUS;
}

unsafe extern "C" fn Wow64Manual_NtUserLoadKeyboardLayoutEx(a: *mut ULONG) -> NTSTATUS {
    let mut K64: WOW64_UNICODE_STRING64_M = core::mem::zeroed();
    let Ret: PVOID = (NtUserLoadKeyboardLayoutEx(
        *a.offset(0 as isize) as ULONG_PTR as PVOID,
        *a.offset(1 as isize),
        *a.offset(2 as isize) as ULONG_PTR as PVOID,
        (*a.offset(3 as isize) as ULONG_PTR as PVOID).cast(),
        (Wow64pUS32to64(
            *a.offset(4 as isize),
            core::ptr::addr_of_mut!(K64),
        ) as PVOID).cast(),
        *a.offset(5 as isize),
        *a.offset(6 as isize),
    )).cast();
    return Ret as ULONG_PTR as ULONG as NTSTATUS;
}

unsafe extern "C" fn Wow64Manual_NtUserEnumDisplaySettings(a: *mut ULONG) -> NTSTATUS {
    let mut N64: WOW64_UNICODE_STRING64_M = WOW64_UNICODE_STRING64_M {
        Length: 0,
        MaximumLength: 0,
        __pad: 0,
        Buffer: 0,
    };
    /* lpDevMode (a[2]) is DEVMODEW: no pointers, identical 32/64 -> flat pass. */
    return NtUserEnumDisplaySettings(
        (Wow64pUS32to64(
            *a.offset(0 as isize),
            core::ptr::addr_of_mut!(N64),
        ) as PVOID).cast(),
        *a.offset(1 as isize),
        (*a.offset(2 as isize) as ULONG_PTR as PVOID).cast(),
        *a.offset(3 as isize),
    );
}

unsafe extern "C" fn Wow64Manual_NtUserEnumDisplayDevices(a: *mut ULONG) -> NTSTATUS {
    let mut N64: WOW64_UNICODE_STRING64_M = core::mem::zeroed();
    /* lpDisplayDevice (a[2]) is DISPLAY_DEVICEW: no pointers, identical 32/64. */
    return NtUserEnumDisplayDevices(
        (Wow64pUS32to64(*a.offset(0 as isize), core::ptr::addr_of_mut!(N64)) as PVOID).cast(),
        *a.offset(1 as isize),
        (*a.offset(2 as isize) as ULONG_PTR as PVOID).cast(),
        *a.offset(3 as isize),
    ) as NTSTATUS;
}

unsafe extern "C" fn Wow64Manual_NtGdiCreateDIBSection(a: *mut ULONG) -> NTSTATUS {
    let mut pBits32: *mut ULONG =
        *a.offset(8 as isize) as ULONG_PTR as *mut ULONG;
    let mut Bits64: PVOID = core::ptr::null_mut();
    let mut Ret: HANDLE = core::ptr::null_mut();

    Ret = (NtGdiCreateDIBSection(
        (*a.offset(0 as isize) as ULONG_PTR as HANDLE).cast(),
        *a.offset(1 as isize) as LONG as LONG_PTR as HANDLE,
        *a.offset(2 as isize),
        (*a.offset(3 as isize) as ULONG_PTR as PVOID).cast(),
        *a.offset(4 as isize),
        *a.offset(5 as isize),
        *a.offset(6 as isize),
        *a.offset(7 as isize) as ULONG_PTR,
        if !pBits32.is_null() {
            core::ptr::addr_of_mut!(Bits64)
        } else {
            core::ptr::null_mut::<PVOID>()
        },
    )).cast();
    if !Ret.is_null() && !pBits32.is_null() {
        if (Bits64 as ULONG_PTR) >> 32 != 0 {
            DbgPrint(
                b"[WOW64]   NtGdiCreateDIBSection mapped bits at %p (above 4GB) \xE2\x80\x94 unrepresentable to a 32-bit guest; failing the call\n\0"
                    as *const u8 as PCSTR,
                Bits64,
            );
            NtGdiDeleteObjectApp(Ret as PVOID);
            *pBits32 = 0 as ULONG;
            return 0 as NTSTATUS;
        }
        *pBits32 = Bits64 as ULONG_PTR as ULONG;
    }
    return Ret as ULONG_PTR as ULONG as NTSTATUS;
}

unsafe extern "C" fn Wow64Manual_NtUserTranslateAccelerator(a: *mut ULONG) -> NTSTATUS {
    let mut M32: *mut WOW64_MSG32 =
        *a.add(2 as usize) as ULONG_PTR as *mut WOW64_MSG32;
    let mut M64: WOW64_MSG64 = core::mem::zeroed();

    if M32.is_null() {
        return 0 as NTSTATUS;
    }
    Wow64pMsg32To64(M32, &raw mut M64);
    return NtUserTranslateAccelerator(
        (*a.add(0 as usize) as ULONG_PTR as PVOID).cast(),
        (*a.add(1 as usize) as ULONG_PTR as PVOID).cast(),
        (&raw mut M64 as PVOID).cast(),
    ) as NTSTATUS;
}

unsafe extern "C" fn Wow64Manual_NtUserCallMsgFilter(a: *mut ULONG) -> NTSTATUS {
    let mut M32: *mut WOW64_MSG32 =
        *a.offset(0 as isize) as ULONG_PTR as *mut WOW64_MSG32;
    let mut M64: WOW64_MSG64 = core::mem::zeroed();
    let mut Ret: LONG = 0;

    if !M32.is_null() {
    } else {
        return 0 as NTSTATUS;
    }
    Wow64pMsg32To64(M32, core::ptr::addr_of_mut!(M64));
    Ret = NtUserCallMsgFilter(
        (core::ptr::addr_of_mut!(M64) as PVOID).cast(),
        *a.offset(1 as isize) as LONG,
    );
    Wow64pMsg64To32(core::ptr::addr_of_mut!(M64), M32);
    return Ret as NTSTATUS;
}
unsafe extern "C" fn Wow64Manual_NtUserRealInternalGetMessage(a: *mut ULONG) -> NTSTATUS {
    let mut M32: *mut WOW64_MSG32 =
        *a.offset(0 as isize) as ULONG_PTR as *mut WOW64_MSG32;
    let mut M64: WOW64_MSG64 = core::mem::zeroed();
    let mut Ret: LONG = 0;

    if M32.is_null() {
        return 0;
    }
    Ret = NtUserRealInternalGetMessage(
        (core::ptr::addr_of_mut!(M64) as PVOID).cast(),
        (*a.offset(1 as isize) as ULONG_PTR as PVOID).cast(),
        *a.offset(2 as isize),
        *a.offset(3 as isize),
        *a.offset(4 as isize),
        (*a.offset(5 as isize)) as i32,
    );
    if Ret != 0 {
        Wow64pMsg64To32(
            core::ptr::addr_of_mut!(M64),
            M32,
        );
    }
    return Ret as NTSTATUS;
}

unsafe extern "C" fn Wow64Manual_NtUserResolveDesktop(a: *mut ULONG) -> NTSTATUS {
    let mut Path64: WOW64_UNICODE_STRING64_M = core::mem::zeroed();
    let pWinSta32: *mut ULONG =
        *a.offset(3 as isize) as ULONG_PTR as *mut ULONG;
    let mut hWinSta: HANDLE = core::ptr::null_mut();
    let mut Ret: HANDLE = core::ptr::null_mut();
    Ret = (NtUserResolveDesktop(
        *a.offset(0 as isize) as LONG as LONG_PTR as HANDLE,
        (Wow64pUS32to64(
            *a.offset(1 as isize),
            core::ptr::addr_of_mut!(Path64),
        ) as PVOID).cast(),
        (*a.offset(2 as isize)) as i32,
        if !pWinSta32.is_null() {
            (core::ptr::addr_of_mut!(hWinSta) as PVOID).cast()
        } else {
            core::ptr::null_mut()
        },
    )).cast();
    if !pWinSta32.is_null() {
        *pWinSta32 = hWinSta as ULONG_PTR as ULONG;
    }
    return Ret as ULONG_PTR as ULONG as NTSTATUS;
}

unsafe extern "C" fn Wow64Manual_NtUserOpenDesktop(a: *mut ULONG) -> NTSTATUS {
    let mut Oa64: WOW64_OBJECT_ATTRIBUTES64_M = core::mem::zeroed();
    let mut Uname64: WOW64_UNICODE_STRING64_M = core::mem::zeroed();
    let mut Ret: HANDLE;

    if *a.offset(0 as isize) == 0 {
        return NtUserOpenDesktop(
            core::ptr::null_mut(),
            *a.offset(1 as isize),
            *a.offset(2 as isize) as ACCESS_MASK,
        ) as ULONG_PTR as ULONG as NTSTATUS;
    }
    Wow64pMarshalOa(
        &raw mut Oa64,
        &raw mut Uname64,
        *a.offset(0 as isize),
    );
    Ret = (NtUserOpenDesktop(
        (&raw mut Oa64 as PVOID).cast(),
        *a.offset(1 as isize),
        *a.offset(2 as isize) as ACCESS_MASK,
    )).cast();
    return Ret as ULONG_PTR as ULONG as NTSTATUS;
}

unsafe extern "C" fn Wow64Manual_NtUserOpenWindowStation(a: *mut ULONG) -> NTSTATUS {
    let mut Oa64: WOW64_OBJECT_ATTRIBUTES64_M = core::mem::zeroed();
    let mut Uname64: WOW64_UNICODE_STRING64_M = core::mem::zeroed();
    let mut Ret: HANDLE = core::ptr::null_mut();

    if *a.offset(0 as isize) == 0 {
        return NtUserOpenWindowStation(
            core::ptr::null_mut(),
            *a.offset(1 as isize) as ACCESS_MASK,
        ) as ULONG_PTR as ULONG as NTSTATUS;
    }
    Wow64pMarshalOa(
        core::ptr::addr_of_mut!(Oa64),
        core::ptr::addr_of_mut!(Uname64),
        *a.offset(0 as isize),
    );
    Ret = (NtUserOpenWindowStation(
        (core::ptr::addr_of_mut!(Oa64) as PVOID).cast(),
        *a.offset(1 as isize) as ACCESS_MASK,
    )).cast();
    return Ret as ULONG_PTR as ULONG as NTSTATUS;
}

unsafe extern "C" fn Wow64Manual_NtUserCreateWindowStation(a: *mut ULONG) -> NTSTATUS {
    let mut Oa64: WOW64_OBJECT_ATTRIBUTES64_M = core::mem::zeroed();
    let mut Uname64: WOW64_UNICODE_STRING64_M = core::mem::zeroed();
    let mut Ret: HANDLE = core::ptr::null_mut();

    if *a.offset(0 as isize) == 0 {
        return NtUserCreateWindowStation(
            (core::ptr::null_mut() as PVOID).cast(),
            *a.offset(1 as isize) as ACCESS_MASK,
            *a.offset(2 as isize),
            *a.offset(3 as isize),
            *a.offset(4 as isize),
            *a.offset(5 as isize),
            *a.offset(6 as isize),
        ) as ULONG_PTR as ULONG as NTSTATUS;
    }
    Wow64pMarshalOa(
        core::ptr::addr_of_mut!(Oa64),
        core::ptr::addr_of_mut!(Uname64),
        *a.offset(0 as isize),
    );
    Ret = (NtUserCreateWindowStation(
        (core::ptr::addr_of_mut!(Oa64) as PVOID).cast(),
        *a.offset(1 as isize) as ACCESS_MASK,
        *a.offset(2 as isize),
        *a.offset(3 as isize),
        *a.offset(4 as isize),
        *a.offset(5 as isize),
        *a.offset(6 as isize),
    )).cast();
    return Ret as ULONG_PTR as ULONG as NTSTATUS;
}

unsafe extern "C" fn Wow64Manual_NtUserCreateDesktop(a: *mut ULONG) -> NTSTATUS {
    let mut Oa64: WOW64_OBJECT_ATTRIBUTES64_M = core::mem::zeroed();
    let mut Uname64: WOW64_UNICODE_STRING64_M = core::mem::zeroed();
    let mut Device64: WOW64_UNICODE_STRING64_M = core::mem::zeroed();
    let mut Oa: PVOID = core::ptr::null_mut();

    if *a.add(0 as usize) != 0 {
        Wow64pMarshalOa(
            core::ptr::addr_of_mut!(Oa64),
            core::ptr::addr_of_mut!(Uname64),
            *a.add(0 as usize),
        );
        Oa = core::ptr::addr_of_mut!(Oa64) as PVOID;
    }
    return (NtUserCreateDesktop(
        (Oa).cast(),
        (Wow64pUS32to64(
            *a.add(1 as usize),
            core::ptr::addr_of_mut!(Device64),
        ) as PVOID).cast(),
        (*a.add(2 as usize) as ULONG_PTR as PVOID).cast(),
        *a.add(3 as usize),
        *a.add(4 as usize),
    ) as ULONG_PTR as ULONG as NTSTATUS);
}

unsafe extern "C" fn Wow64pTraceCurIcon(What: *const ::core::ffi::c_char, Mod32: ULONG, Rsrc32: ULONG, Param: *mut LONG, Flags: ULONG, Ret: ULONG) {
    if Wow64TraceSyscalls == 0 {
        return;
    }
    let M: *mut WOW64_UNICODE_STRING32_M = Mod32 as ULONG_PTR as *mut WOW64_UNICODE_STRING32_M;
    let R: *mut WOW64_UNICODE_STRING32_M = Rsrc32 as ULONG_PTR as *mut WOW64_UNICODE_STRING32_M;
    DbgPrint(
        b"[WOW64-CURICON] %s mod=%.*S rsrc=%s%.*S%s id=%lx len=%u flags=%lx param=%ld,%ld,%ld ret=%lx\n\0"
            as *const u8 as PCSTR,
        What,
        if !M.is_null() && (*M).Buffer != 0 {
            (*M).Length as ::core::ffi::c_int / 2 as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        },
        if !M.is_null() && (*M).Buffer != 0 {
            (*M).Buffer as ULONG_PTR as *const u16
        } else {
            RUSTCONV_WSTR_96A296D224F2.as_ptr()
        },
        if !R.is_null() && (*R).Buffer >= 0x10000 as ULONG {
            b"\"\0" as *const u8 as *const ::core::ffi::c_char
        } else {
            b"\0" as *const u8 as *const ::core::ffi::c_char
        },
        if !R.is_null() && (*R).Buffer >= 0x10000 as ULONG {
            (*R).Length as ::core::ffi::c_int / 2 as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        },
        if !R.is_null() && (*R).Buffer >= 0x10000 as ULONG {
            (*R).Buffer as ULONG_PTR as *const u16
        } else {
            RUSTCONV_WSTR_96A296D224F2.as_ptr()
        },
        if !R.is_null() && (*R).Buffer >= 0x10000 as ULONG {
            b"\"\0" as *const u8 as *const ::core::ffi::c_char
        } else {
            b"\0" as *const u8 as *const ::core::ffi::c_char
        },
        if !R.is_null() { (*R).Buffer } else { 0 as ULONG },
        if !R.is_null() {
            (*R).Length as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        },
        Flags,
        if !Param.is_null() {
            *Param.offset(0 as isize)
        } else {
            -(1 as ::core::ffi::c_int) as LONG
        },
        if !Param.is_null() {
            *Param.offset(1 as isize)
        } else {
            -(1 as ::core::ffi::c_int) as LONG
        },
        if !Param.is_null() {
            *Param.offset(2 as isize)
        } else {
            -(1 as ::core::ffi::c_int) as LONG
        },
        Ret,
    );
}

unsafe extern "C" fn Wow64Manual_NtUserFindExistingCursorIcon(a: *mut ULONG) -> NTSTATUS {
    let mut Mod64: WOW64_UNICODE_STRING64_M = core::mem::zeroed();
    let mut Rsrc64: WOW64_UNICODE_STRING64_M = core::mem::zeroed();
    let mut Ret: HANDLE = core::ptr::null_mut();
    Ret = (NtUserFindExistingCursorIcon(
        (Wow64pUS32to64(*a.add(0 as usize), core::ptr::addr_of_mut!(Mod64)) as PVOID).cast(),
        (Wow64pUS32to64(*a.add(1 as usize), core::ptr::addr_of_mut!(Rsrc64)) as PVOID).cast(),
        ((*a.add(2 as usize)) as ULONG_PTR as PVOID).cast(),
    )).cast();
    Wow64pTraceCurIcon(
        b"FindExisting\0".as_ptr() as *const ::core::ffi::c_char,
        *a.add(0 as usize),
        *a.add(1 as usize),
        if *a.add(2 as usize) != 0 {
            (*a.add(2 as usize)) as ULONG_PTR as *mut LONG
        } else {
            core::ptr::null_mut::<LONG>()
        },
        0 as ULONG,
        (Ret as ULONG_PTR) as ULONG,
    );
    return ((Ret as ULONG_PTR) as ULONG) as NTSTATUS;
}

unsafe extern "C" fn Wow64Manual_NtUserSetCursorIconData(a: *mut ULONG) -> NTSTATUS {
    let mut Mod64: WOW64_UNICODE_STRING64_M = core::mem::zeroed();
    let mut Rsrc64: WOW64_UNICODE_STRING64_M = core::mem::zeroed();
    let mut C32: *mut WOW64_CURSORDATA32 =
        (*a.add(3 as usize)) as ULONG_PTR as *mut WOW64_CURSORDATA32;
    let mut C64: WOW64_CURSORDATA64 = core::mem::zeroed();
    let mut i: i32 = 0;

    if !C32.is_null() {
        C64.lpName = (*C32).lpName as ULONG_PTR as ULONGLONG;
        C64.lpModName = (*C32).lpModName as ULONG_PTR as ULONGLONG;
        C64.rt = (*C32).rt;
        C64.dummy = (*C32).dummy;
        C64.CURSORF_flags = (*C32).CURSORF_flags;
        C64.xHotspot = (*C32).xHotspot;
        C64.yHotspot = (*C32).yHotspot;
        C64.hbmMask = (*C32).hbmMask as ULONG_PTR as ULONGLONG;
        C64.hbmColor = (*C32).hbmColor as ULONG_PTR as ULONGLONG;
        C64.hbmAlpha = (*C32).hbmAlpha as ULONG_PTR as ULONGLONG;
        i = 0 as i32;
        while i < 4 as i32 {
            C64.rcBounds[i as usize] = (*C32).rcBounds[i as usize];
            i += 1;
        }
        C64.hbmUserAlpha = (*C32).hbmUserAlpha as ULONG_PTR as ULONGLONG;
        C64.bpp = (*C32).bpp;
        C64.cx = (*C32).cx;
        C64.cy = (*C32).cy;
        C64.cpcur = (*C32).cpcur;
        C64.cicur = (*C32).cicur;
        C64.aspcur = (*C32).aspcur as ULONG_PTR as ULONGLONG;
        C64.aicur = (*C32).aicur as ULONG_PTR as ULONGLONG;
        C64.ajifRate = (*C32).ajifRate as ULONG_PTR as ULONGLONG;
        C64.iicur = (*C32).iicur as UINT;
        if (*C32).CURSORF_flags & WOW64_CURSORF_ACON as ULONG != 0 {
            DbgPrint(
                b"[WOW64] NtUserSetCursorIconData: ANIMATED cursor \xE2\x80\x94 frame arrays not re-marshalled (owed)\n\0"
                    .as_ptr() as PCSTR,
            );
        }
    }

    Wow64pTraceCurIcon(
        b"SetData\0".as_ptr() as *const core::ffi::c_char,
        *a.add(1 as usize),
        *a.add(2 as usize),
        core::ptr::null_mut::<LONG>(),
        if (!(C32).is_null()) {
            (*C32).CURSORF_flags
        } else {
            0xffffffff as ULONG
        },
        *a.add(0 as usize),
    );
    return NtUserSetCursorIconData(
        *a.add(0 as usize) as LONG as LONG_PTR as HANDLE,
        Wow64pUS32to64(*a.add(1 as usize), core::ptr::addr_of_mut!(Mod64)) as PVOID,
        Wow64pUS32to64(*a.add(2 as usize), core::ptr::addr_of_mut!(Rsrc64)) as PVOID,
        if (!(C32).is_null()) {
            core::ptr::addr_of_mut!(C64) as PVOID
        } else {
            core::ptr::null_mut()
        },
    ) as NTSTATUS;
}

unsafe extern "C" fn Wow64pUSUp(U32Ptr: ULONG, U64: *mut WOW64_UNICODE_STRING64_M) {
    let U32: *const WOW64_UNICODE_STRING32_M =
        U32Ptr as ULONG_PTR as *const WOW64_UNICODE_STRING32_M;
    (*U64).Length = (*U32).Length;
    (*U64).MaximumLength = (*U32).MaximumLength;
    (*U64).__pad = 0 as ULONG;
    (*U64).Buffer = (*U32).Buffer as ULONGLONG;
}

unsafe extern "C" fn Wow64pUSDown(U64: *const WOW64_UNICODE_STRING64_M, U32Ptr: ULONG) {
    let U32: *mut WOW64_UNICODE_STRING32_M = U32Ptr as ULONG_PTR as *mut WOW64_UNICODE_STRING32_M;
    (*U32).Length = (*U64).Length;
    (*U32).MaximumLength = (*U64).MaximumLength;
    (*U32).Buffer = (*U64).Buffer as ULONG;
}

unsafe extern "C" fn Wow64Manual_NtUserGetIconInfo(a: *mut ULONG) -> NTSTATUS {
    let mut Ii64: WOW64_ICONINFO64_M = core::mem::zeroed();
    let mut Mod64: WOW64_UNICODE_STRING64_M = core::mem::zeroed();
    let mut Res64: WOW64_UNICODE_STRING64_M = core::mem::zeroed();
    let mut Ret: BOOL = 0;
    if *a.add(2 as usize) != 0 {
        Wow64pUSUp(*a.add(2 as usize), core::ptr::addr_of_mut!(Mod64));
    }
    if *a.add(3 as usize) != 0 {
        Wow64pUSUp(*a.add(3 as usize), core::ptr::addr_of_mut!(Res64));
    }
    Ret = NtUserGetIconInfo(
        *a.add(0 as usize) as LONG as LONG_PTR as HANDLE,
        ((if *a.add(1 as usize) != 0 {
            core::ptr::addr_of_mut!(Ii64)
        } else {
            core::ptr::null_mut::<WOW64_ICONINFO64_M>()
        }) as PVOID).cast(),
        ((if *a.add(2 as usize) != 0 {
            core::ptr::addr_of_mut!(Mod64)
        } else {
            core::ptr::null_mut::<WOW64_UNICODE_STRING64_M>()
        }) as PVOID).cast(),
        ((if *a.add(3 as usize) != 0 {
            core::ptr::addr_of_mut!(Res64)
        } else {
            core::ptr::null_mut::<WOW64_UNICODE_STRING64_M>()
        }) as PVOID).cast(),
        (*a.add(4 as usize) as ULONG_PTR as PVOID).cast(),
        *a.add(5 as usize) as BOOL,
    );
    if Ret != 0 && *a.add(1 as usize) != 0 {
        let mut Ii32: *mut WOW64_ICONINFO32_M =
            *a.add(1 as usize) as ULONG_PTR as *mut WOW64_ICONINFO32_M;
        (*Ii32).fIcon = Ii64.fIcon;
        (*Ii32).xHotspot = Ii64.xHotspot;
        (*Ii32).yHotspot = Ii64.yHotspot;
        (*Ii32).hbmMask = Ii64.hbmMask as ULONG;
        (*Ii32).hbmColor = Ii64.hbmColor as ULONG;
    }
    /* A too-small string buffer is answered in the string itself (Length 0,
     * the needed MaximumLength), so the strings come back down whatever Ret
     * says. */
    if *a.add(2 as usize) != 0 {
        Wow64pUSDown(core::ptr::addr_of_mut!(Mod64), *a.add(2 as usize));
    }
    if *a.add(3 as usize) != 0 {
        Wow64pUSDown(core::ptr::addr_of_mut!(Res64), *a.add(3 as usize));
    }
    return Ret as NTSTATUS;
}

unsafe extern "C" fn Wow64pBitmapDown(B64: *const WOW64_BITMAP64_M, B32: *mut WOW64_BITMAP32_M) {
    (*B32).bmType = (*B64).bmType;
    (*B32).bmWidth = (*B64).bmWidth;
    (*B32).bmHeight = (*B64).bmHeight;
    (*B32).bmWidthBytes = (*B64).bmWidthBytes;
    (*B32).bmPlanes = (*B64).bmPlanes;
    (*B32).bmBitsPixel = (*B64).bmBitsPixel;
    (*B32).bmBits = (*B64).bmBits as ULONG;
}

unsafe extern "C" fn Wow64Manual_NtGdiExtGetObjectW(a: *mut ULONG) -> NTSTATUS {
    #[repr(C)]
    union C2RustUnnamed_Host {
        Ds: WOW64_DIBSECTION64_M,
        Lb: WOW64_LOGBRUSH64_M,
        Pen: [BYTE; (WOW64_EXTPEN64_HEAD + (64 * core::mem::size_of::<ULONG>()) as u32) as _],
    }
    let h: HANDLE = *a.offset(0 as isize) as LONG as LONG_PTR as HANDLE;
    let cj: INT = *a.offset(1 as isize) as INT;
    let Out: *mut BYTE = *a.offset(2 as isize) as ULONG_PTR as *mut BYTE;
    let mut Host: C2RustUnnamed_Host = core::mem::zeroed();
    let mut r: INT = 0;
    let mut i: INT = 0;

    match *a.offset(0 as isize) & WOW64_GDI_TYPE_MASK as ULONG {
        WOW64_GDI_TYPE_BITMAP => {
            if Out.is_null() {
                r = NtGdiExtGetObjectW(h, cj, core::ptr::null_mut());
                return (if r as usize == core::mem::size_of::<WOW64_BITMAP64_M>() {
                    core::mem::size_of::<WOW64_BITMAP32_M>()
                } else {
                    r as usize
                }) as NTSTATUS;
            }
            if cj < core::mem::size_of::<WOW64_BITMAP32_M>() as INT {
                return 0 as NTSTATUS;
            }
            r = NtGdiExtGetObjectW(
                h,
                (if cj >= core::mem::size_of::<WOW64_DIBSECTION32_M>() as INT {
                    core::mem::size_of::<WOW64_DIBSECTION64_M>()
                } else {
                    core::mem::size_of::<WOW64_BITMAP64_M>() as usize
                }) as INT,
                &raw mut Host as PVOID,
            );
            if r < core::mem::size_of::<WOW64_BITMAP64_M>() as INT {
                return 0 as NTSTATUS;
            }
            Wow64pBitmapDown(&raw mut Host.Ds.dsBm, Out as *mut WOW64_BITMAP32_M);
            if r < core::mem::size_of::<WOW64_DIBSECTION64_M>() as INT {
                return core::mem::size_of::<WOW64_BITMAP32_M>() as NTSTATUS;
            }
            let mut D32: *mut WOW64_DIBSECTION32_M = Out as *mut WOW64_DIBSECTION32_M;
            i = 0 as INT;
            while i < 40 as INT {
                (*D32).dsBmih[i as usize] = Host.Ds.dsBmih[i as usize];
                i += 1;
            }
            i = 0 as INT;
            while i < 3 as INT {
                (*D32).dsBitfields[i as usize] = Host.Ds.dsBitfields[i as usize];
                i += 1;
            }
            (*D32).dshSection = Host.Ds.dshSection as ULONG;
            (*D32).dsOffset = Host.Ds.dsOffset;
            return core::mem::size_of::<WOW64_DIBSECTION32_M>() as NTSTATUS;
        }
        WOW64_GDI_TYPE_BRUSH => {
            if Out.is_null() {
                r = NtGdiExtGetObjectW(h, cj, core::ptr::null_mut());
                return (if r as usize == core::mem::size_of::<WOW64_LOGBRUSH64_M>() {
                    core::mem::size_of::<WOW64_LOGBRUSH32_M>()
                } else {
                    r as usize
                }) as NTSTATUS;
            }
            if cj < core::mem::size_of::<WOW64_LOGBRUSH32_M>() as INT {
                return 0 as NTSTATUS;
            }
            r = NtGdiExtGetObjectW(h, core::mem::size_of::<WOW64_LOGBRUSH64_M>() as INT, &raw mut Host as PVOID);
            if r < core::mem::size_of::<WOW64_LOGBRUSH64_M>() as INT {
                return 0 as NTSTATUS;
            }
            (*(Out as *mut WOW64_LOGBRUSH32_M)).lbStyle = Host.Lb.lbStyle;
            (*(Out as *mut WOW64_LOGBRUSH32_M)).lbColor = Host.Lb.lbColor;
            (*(Out as *mut WOW64_LOGBRUSH32_M)).lbHatch = Host.Lb.lbHatch as ULONG;
            return core::mem::size_of::<WOW64_LOGBRUSH32_M>() as NTSTATUS;
        }
        WOW64_GDI_TYPE_PEN | WOW64_GDI_TYPE_EXTPEN => {
            let mut cjHost: INT = cj;
            if Out.is_null() {
                r = NtGdiExtGetObjectW(h, cj, core::ptr::null_mut());
                return (if r >= WOW64_EXTPEN64_HEAD as INT {
                    r - 4 as INT
                } else {
                    r
                }) as NTSTATUS;
            }
            if cj >= WOW64_EXTPEN32_HEAD as INT {
                cjHost = cj + 4 as INT;
            }
            if cjHost > core::mem::size_of::<[BYTE; (WOW64_EXTPEN64_HEAD + ((64 * core::mem::size_of::<ULONG>()) as u32)) as _]>() as INT {
                cjHost = core::mem::size_of::<[BYTE; (WOW64_EXTPEN64_HEAD + (64 * core::mem::size_of::<ULONG>()) as u32) as _]>() as INT;
            }
            r = NtGdiExtGetObjectW(h, cjHost, &raw mut Host.Pen as *mut BYTE as PVOID);
            if r < WOW64_EXTPEN64_HEAD as INT {
                i = 0 as INT;
                while i < r {
                    *Out.offset(i as isize) = Host.Pen[i as usize];
                    i += 1;
                }
                return r as NTSTATUS;
            }
            i = 0 as INT;
            while i < 16 as INT {
                *Out.offset(i as isize) = Host.Pen[i as usize];
                i += 1;
            }
            *(Out.offset(16 as isize) as *mut ULONG) =
                *((&raw mut Host.Pen as *mut BYTE).offset(16 as isize) as *mut ULONGLONG) as ULONG;
            i = 24 as INT;
            while i < r {
                *Out.offset((i - 4 as INT) as isize) = Host.Pen[i as usize];
                i += 1;
            }
            return (r - 4 as INT) as NTSTATUS;
        }
        _ => return NtGdiExtGetObjectW(h, cj, Out as PVOID) as NTSTATUS,
    }
}

unsafe extern "C" fn Wow64Manual_NtCallbackReturn(a: *mut ULONG) -> NTSTATUS {
    if Wow64pCompleteUserCallback(
        *a.offset(0 as isize),
        *a.offset(1 as isize),
        *a.offset(2 as isize) as NTSTATUS,
    ) == 0
    {
        DbgPrint(b"[WOW64] NtCallbackReturn with NO callback in flight\n\0".as_ptr() as PCSTR);
        return 0xC0000258 as ULONG as NTSTATUS;
    }
    return 0 as NTSTATUS;
}

unsafe extern "C" fn Wow64Manual_NtCreateSemaphore(a: *mut ULONG) -> NTSTATUS {
    let mut Handle32: *mut ULONG = *a.offset(0 as isize) as ULONG_PTR as *mut ULONG;
    let mut Oa64: WOW64_OBJECT_ATTRIBUTES64_M = core::mem::zeroed();
    let mut Uname64: WOW64_UNICODE_STRING64_M = core::mem::zeroed();
    let mut Oa64Ptr: PVOID = core::ptr::null_mut();
    let mut Handle: HANDLE = core::ptr::null_mut();
    let mut St: NTSTATUS;

    if *a.offset(2 as isize) != 0 {
        Wow64pMarshalOa(
            &raw mut Oa64,
            &raw mut Uname64,
            *a.offset(2 as isize),
        );
        Oa64Ptr = &raw mut Oa64 as PVOID;
    }
    St = NtCreateSemaphore(
        &raw mut Handle,
        *a.offset(1 as isize) as ACCESS_MASK,
        (Oa64Ptr).cast(),
        *a.offset(3 as isize) as LONG,
        *a.offset(4 as isize) as LONG,
    );
    if St >= 0 as NTSTATUS && !Handle32.is_null() {
        *Handle32 = Handle as ULONG_PTR as ULONG;
    }
    return St;
}

unsafe extern "C" fn Wow64Manual_NtCreateMutant(a: *mut ULONG) -> NTSTATUS {
    let Handle32: *mut ULONG =
        *a.offset(0 as isize) as ULONG_PTR as *mut ULONG;
    let mut Oa64: WOW64_OBJECT_ATTRIBUTES64_M = WOW64_OBJECT_ATTRIBUTES64_M {
        Length: 0,
        __pad0: 0,
        RootDirectory: 0,
        ObjectName: 0,
        Attributes: 0,
        __pad1: 0,
        SecurityDescriptor: 0,
        SecurityQualityOfService: 0,
    };
    let mut Uname64: WOW64_UNICODE_STRING64_M = WOW64_UNICODE_STRING64_M {
        Length: 0,
        MaximumLength: 0,
        __pad: 0,
        Buffer: 0,
    };
    let mut Oa64Ptr: PVOID = core::ptr::null_mut();
    let mut Handle: HANDLE = core::ptr::null_mut();
    let mut St: NTSTATUS = 0;
    if *a.offset(2 as isize) != 0 {
        Wow64pMarshalOa(
            &raw mut Oa64,
            &raw mut Uname64,
            *a.offset(2 as isize),
        );
        Oa64Ptr = &raw mut Oa64 as PVOID;
    }
    St = NtCreateMutant(
        &raw mut Handle,
        *a.offset(1 as isize) as ACCESS_MASK,
        (Oa64Ptr).cast(),
        *a.offset(3 as isize) as BOOLEAN,
    );
    if St >= (0 as NTSTATUS) && !Handle32.is_null() {
        *Handle32 = Handle as ULONG_PTR as ULONG;
    }
    return St;
}

unsafe extern "C" fn Wow64Manual_NtCreateKey(a: *mut ULONG) -> NTSTATUS {
    let Handle32: *mut ULONG =
        *a.offset(0 as isize) as ULONG_PTR as *mut ULONG;
    let Disp32: *mut ULONG =
        *a.offset(6 as isize) as ULONG_PTR as *mut ULONG;
    let mut Oa64: WOW64_OBJECT_ATTRIBUTES64_M = core::mem::zeroed();
    let mut Uname64: WOW64_UNICODE_STRING64_M = core::mem::zeroed();
    let mut Class64: WOW64_UNICODE_STRING64_M = core::mem::zeroed();
    let mut Class64Ptr: PVOID = core::ptr::null_mut();
    let mut Handle: HANDLE = core::ptr::null_mut();
    let mut Disposition: ULONG = 0 as ULONG;
    let mut St: NTSTATUS;

    Wow64pMarshalOa(
        core::ptr::addr_of_mut!(Oa64),
        core::ptr::addr_of_mut!(Uname64),
        *a.offset(2 as isize),
    );
    if *a.offset(4 as isize) != 0 {
        let Class32: *const WOW64_UNICODE_STRING32_M =
            *a.offset(4 as isize) as ULONG_PTR
                as *const WOW64_UNICODE_STRING32_M;
        Class64.Length = (*Class32).Length;
        Class64.MaximumLength = (*Class32).MaximumLength;
        Class64.__pad = 0 as ULONG;
        Class64.Buffer = (*Class32).Buffer as ULONGLONG;
        Class64Ptr = core::ptr::addr_of_mut!(Class64) as PVOID;
    }
    St = NtCreateKey(
        core::ptr::addr_of_mut!(Handle),
        *a.offset(1 as isize) as ACCESS_MASK,
        (core::ptr::addr_of_mut!(Oa64) as PVOID).cast(),
        *a.offset(3 as isize),
        (Class64Ptr).cast(),
        *a.offset(5 as isize),
        core::ptr::addr_of_mut!(Disposition),
    );
    if St >= 0 as NTSTATUS && !Handle32.is_null() {
        *Handle32 = Handle as ULONG_PTR as ULONG;
    }
    if !Disp32.is_null() {
        *Disp32 = Disposition;
    }
    return St;
}

unsafe extern "C" fn Wow64Manual_NtCreateSymbolicLinkObject(a: *mut ULONG) -> NTSTATUS {
    let Handle32: *mut ULONG = *a.add(0) as ULONG_PTR as *mut ULONG;
    let mut Oa64: WOW64_OBJECT_ATTRIBUTES64_M = WOW64_OBJECT_ATTRIBUTES64_M {
        Length: 0,
        __pad0: 0,
        RootDirectory: 0,
        ObjectName: 0,
        Attributes: 0,
        __pad1: 0,
        SecurityDescriptor: 0,
        SecurityQualityOfService: 0,
    };
    let mut Uname64: WOW64_UNICODE_STRING64_M = WOW64_UNICODE_STRING64_M {
        Length: 0,
        MaximumLength: 0,
        __pad: 0,
        Buffer: 0,
    };
    let mut Target64: WOW64_UNICODE_STRING64_M = WOW64_UNICODE_STRING64_M {
        Length: 0,
        MaximumLength: 0,
        __pad: 0,
        Buffer: 0,
    };
    let mut Target32: *const WOW64_UNICODE_STRING32_M = core::ptr::null::<WOW64_UNICODE_STRING32_M>();
    let mut Handle: HANDLE = core::ptr::null_mut();
    let mut St: NTSTATUS;

    if *a.add(3) == 0 {
        return ((0xC000000Du32 as i32)) as NTSTATUS;
    }

    Wow64pMarshalOa(
        core::ptr::addr_of_mut!(Oa64),
        core::ptr::addr_of_mut!(Uname64),
        *a.add(2),
    );
    Target32 = *a.add(3) as ULONG_PTR as *const WOW64_UNICODE_STRING32_M;
    Target64.Length = (*Target32).Length;
    Target64.MaximumLength = (*Target32).MaximumLength;
    Target64.__pad = 0;
    Target64.Buffer = (*Target32).Buffer as ULONGLONG;

    St = NtCreateSymbolicLinkObject(
        core::ptr::addr_of_mut!(Handle),
        *a.add(1) as ACCESS_MASK,
        (core::ptr::addr_of_mut!(Oa64) as PVOID).cast(),
        (core::ptr::addr_of_mut!(Target64) as PVOID).cast(),
    );
    if St >= (0) as NTSTATUS && !Handle32.is_null() {
        *Handle32 = Handle as ULONG_PTR as ULONG;
    }
    return St;
}

unsafe extern "C" fn Wow64pOpenByCid(Fn: WOW64_OPEN_BY_CID_FN, a: *mut ULONG) -> NTSTATUS {
    let Handle32: *mut ULONG =
        *a.offset(0 as isize) as ULONG_PTR as *mut ULONG;
    let mut Oa64: WOW64_OBJECT_ATTRIBUTES64_M = core::mem::zeroed();
    let mut Uname64: WOW64_UNICODE_STRING64_M = core::mem::zeroed();
    let mut Cid64: WOW64_CLIENT_ID64_M = core::mem::zeroed();
    let mut Cid64Ptr: PVOID = core::ptr::null_mut();
    let mut Handle: HANDLE = core::ptr::null_mut();
    let mut St: NTSTATUS;

    Wow64pMarshalOa(
        &raw mut Oa64,
        &raw mut Uname64,
        *a.offset(2 as isize),
    );
    if *a.offset(3 as isize) != 0 {
        let Cid32: *const WOW64_CLIENT_ID32_M =
            *a.offset(3 as isize) as ULONG_PTR as *const WOW64_CLIENT_ID32_M;
        Cid64.UniqueProcess = (*Cid32).UniqueProcess as ULONGLONG;
        Cid64.UniqueThread = (*Cid32).UniqueThread as ULONGLONG;
        Cid64Ptr = &raw mut Cid64 as PVOID;
    }
    St = Fn.unwrap_unchecked()(
        &raw mut Handle,
        *a.offset(1 as isize) as ACCESS_MASK,
        &raw mut Oa64 as PVOID,
        Cid64Ptr,
    );
    if St >= 0 as NTSTATUS && !Handle32.is_null() {
        *Handle32 = Handle as ULONG_PTR as ULONG;
    }
    return St;
}

unsafe extern "C" fn Wow64Manual_NtCancelIoFile(a: *mut ULONG) -> NTSTATUS {
    let File: HANDLE = *a.offset(0 as isize) as LONG as LONG_PTR as HANDLE;
    let mut Iosb64: WOW64_IO_STATUS_BLOCK64_M = WOW64_IO_STATUS_BLOCK64_M {
        Status: 0,
        Information: 0,
    };
    let St: NTSTATUS = NtCancelIoFile(File, (core::ptr::addr_of_mut!(Iosb64) as PVOID).cast());
    if St < 0 {
        Iosb64.Status = St as ULONG as ULONGLONG;
    }
    Wow64pCopyIosbDown(core::ptr::addr_of_mut!(Iosb64), *a.offset(1 as isize));
    return St;
}

unsafe extern "C" fn Wow64Manual_NtCancelIoFileEx(a: *mut ULONG) -> NTSTATUS {
    let File: HANDLE = *a.add(0 as usize) as LONG as LONG_PTR as HANDLE;
    let mut Iosb64: WOW64_IO_STATUS_BLOCK64_M = WOW64_IO_STATUS_BLOCK64_M {
        Status: 0 as ULONGLONG,
        Information: 0 as ULONGLONG,
    };
    let St: NTSTATUS = NtCancelIoFileEx(
        File,
        (*a.add(1 as usize) as ULONG_PTR as PVOID).cast(),
        (core::ptr::addr_of_mut!(Iosb64) as PVOID).cast(),
    );
    if St < 0 {
        Iosb64.Status = St as ULONG as ULONGLONG;
    }
    Wow64pCopyIosbDown(
        core::ptr::addr_of_mut!(Iosb64),
        *a.add(2 as usize),
    );
    return St;
}

unsafe extern "C" fn Wow64Manual_NtCancelSynchronousIoFile(a: *mut ULONG) -> NTSTATUS {
    let Thread: HANDLE = *a.offset(0 as isize) as LONG as LONG_PTR as HANDLE;
    let mut Iosb64: WOW64_IO_STATUS_BLOCK64_M = WOW64_IO_STATUS_BLOCK64_M {
        Status: 0,
        Information: 0,
    };
    let St: NTSTATUS = NtCancelSynchronousIoFile(
        Thread,
        (*a.offset(1 as isize) as ULONG_PTR as PVOID).cast(),
        (core::ptr::addr_of_mut!(Iosb64) as PVOID).cast(),
    );
    if St < 0 {
        Iosb64.Status = St as ULONG as ULONGLONG;
    }
    Wow64pCopyIosbDown(
        core::ptr::addr_of_mut!(Iosb64),
        *a.offset(2 as isize),
    );
    return St;
}

unsafe extern "C" fn Wow64Manual_NtLockFile(a: *mut ULONG) -> NTSTATUS {
    let mut File: HANDLE = *a.add(0 as usize) as LONG as LONG_PTR as HANDLE;
    let mut Iosb64: WOW64_IO_STATUS_BLOCK64_M = WOW64_IO_STATUS_BLOCK64_M {
        Status: 0 as ULONGLONG,
        Information: 0 as ULONGLONG,
    };
    let mut St: NTSTATUS;

    if Wow64pRefuseAsync(
        *a.add(1 as usize),
        *a.add(2 as usize),
        b"NtLockFile\0".as_ptr() as *const i8,
    ) != 0
    {
        return WOW64_STATUS_NOT_IMPLEMENTED;
    }
    /* ByteOffset/Length are LARGE_INTEGER* — layout-identical. Key is by value. */
    St = NtLockFile(
        File,
        core::ptr::null_mut(),
        None, // NULL ApcRoutine
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(Iosb64) as PIO_STATUS_BLOCK,
        *a.add(5 as usize) as ULONG_PTR as PLARGE_INTEGER,
        *a.add(6 as usize) as ULONG_PTR as PLARGE_INTEGER,
        *a.add(7 as usize),
        *a.add(8 as usize) as BOOLEAN,
        *a.add(9 as usize) as BOOLEAN,
    );
    return Wow64pFinishIosb(
        St,
        File,
        core::ptr::addr_of_mut!(Iosb64),
        *a.add(4 as usize),
        b"NtLockFile\0".as_ptr() as *const i8,
    );
}

unsafe extern "C" fn Wow64Manual_NtUnlockFile(a: *mut ULONG) -> NTSTATUS {
    let File: HANDLE =
        *a.offset(0 as isize) as LONG as LONG_PTR as HANDLE;
    let mut Iosb64: WOW64_IO_STATUS_BLOCK64_M = WOW64_IO_STATUS_BLOCK64_M {
        Status: 0,
        Information: 0,
    };
    let mut St: NTSTATUS = NtUnlockFile(
        File,
        (core::ptr::addr_of_mut!(Iosb64) as PVOID).cast(),
        *a.offset(2 as isize) as ULONG_PTR as PLARGE_INTEGER,
        *a.offset(3 as isize) as ULONG_PTR as PLARGE_INTEGER,
        *a.offset(4 as isize),
    );
    if St < 0 as NTSTATUS {
        Iosb64.Status = St as ULONG as ULONGLONG;
    }
    Wow64pCopyIosbDown(
        core::ptr::addr_of_mut!(Iosb64),
        *a.offset(1 as isize),
    );
    return St;
}

unsafe extern "C" fn Wow64Manual_NtSetVolumeInformationFile(a: *mut ULONG) -> NTSTATUS {
    let mut File: HANDLE = *a.add(0 as usize) as LONG as LONG_PTR as HANDLE;
    let mut Iosb64: WOW64_IO_STATUS_BLOCK64_M = WOW64_IO_STATUS_BLOCK64_M {
        Status: 0 as ULONGLONG,
        Information: 0 as ULONGLONG,
    };
    let mut St: NTSTATUS = NtSetVolumeInformationFile(
        File,
        (core::ptr::addr_of_mut!(Iosb64) as PVOID).cast(),
        *a.add(2 as usize) as ULONG_PTR as PVOID,
        *a.add(3 as usize),
        *a.add(4 as usize),
    );
    if St < 0 {
        Iosb64.Status = St as ULONG as ULONGLONG;
    }
    Wow64pCopyIosbDown(
        core::ptr::addr_of_mut!(Iosb64),
        *a.add(1 as usize),
    );
    return St;
}

unsafe extern "C" fn Wow64Manual_NtQueryEaFile(a: *mut ULONG) -> NTSTATUS {
    let File: HANDLE = *a.offset(0 as isize) as LONG as LONG_PTR as HANDLE;
    let mut Iosb64: WOW64_IO_STATUS_BLOCK64_M = WOW64_IO_STATUS_BLOCK64_M {
        Status: 0,
        Information: 0,
    };
    let mut St: NTSTATUS = NtQueryEaFile(
        File,
        (core::ptr::addr_of_mut!(Iosb64) as PVOID).cast(),
        (*a.offset(2 as isize)) as ULONG_PTR as PVOID,
        *a.offset(3 as isize),
        (*a.offset(4 as isize)) as BOOLEAN,
        (*a.offset(5 as isize)) as ULONG_PTR as PVOID,
        *a.offset(6 as isize),
        (*a.offset(7 as isize)) as ULONG_PTR as PULONG,
        (*a.offset(8 as isize)) as BOOLEAN,
    );
    if St < 0 {
        Iosb64.Status = St as ULONG as ULONGLONG;
    }
    Wow64pCopyIosbDown(
        core::ptr::addr_of_mut!(Iosb64),
        *a.offset(1 as isize),
    );
    return St;
}

unsafe extern "C" fn Wow64Manual_NtSetEaFile(a: *mut ULONG) -> NTSTATUS {
    let File: HANDLE = (*a.add(0) as LONG) as LONG_PTR as HANDLE;
    let mut Iosb64: WOW64_IO_STATUS_BLOCK64_M = WOW64_IO_STATUS_BLOCK64_M {
        Status: 0,
        Information: 0,
    };
    let St: NTSTATUS = NtSetEaFile(
        File,
        (core::ptr::addr_of_mut!(Iosb64) as PVOID).cast(),
        (*a.add(2) as ULONG_PTR) as PVOID,
        *a.add(3),
    );
    if St < 0 {
        Iosb64.Status = (St as ULONG) as ULONGLONG;
    }
    Wow64pCopyIosbDown(
        core::ptr::addr_of_mut!(Iosb64),
        *a.add(1),
    );
    return St;
}

unsafe extern "C" fn Wow64pIoctlAlloc(Size: SIZE_T) -> PVOID {
    return RtlAllocateHeap(Wow64pProcessHeap(), 0x00000008 as ULONG, Size);
}

unsafe extern "C" fn Wow64pIoctlFree(P: PVOID) {
    if !P.is_null() {
        RtlFreeHeap(Wow64pProcessHeap(), 0, P);
    }
}

unsafe extern "C" fn Wow64Manual_NtQueryObject(a: *mut ULONG) -> NTSTATUS {
    let Obj: HANDLE = *a.add(0 as usize) as LONG as LONG_PTR as HANDLE;
    let Class: ULONG = *a.add(1 as usize);
    let Buf32: *mut UCHAR = *a.add(2 as usize) as ULONG_PTR as *mut UCHAR;
    let Len32: ULONG = *a.add(3 as usize);
    let RetLen32: *mut ULONG = *a.add(4 as usize) as ULONG_PTR as *mut ULONG;
    let Grow: ULONG = (core::mem::size_of::<WOW64_UNICODE_STRING64_M>()
        .wrapping_sub(core::mem::size_of::<WOW64_UNICODE_STRING32_M>())) as ULONG;

    if Class == 0 || Class == 4 {
        return NtQueryObject(Obj, Class, Buf32 as PVOID, Len32, RetLen32 as PULONG);
    }

    if Class == 1 {
        let mut HostLen: ULONG = Len32.wrapping_add(Grow);
        let mut HostRet: ULONG = 0;
        let mut Guest: ULONG = 0;
        let mut Host: *mut UCHAR = Wow64pIoctlAlloc((HostLen) as SIZE_T) as *mut UCHAR;
        let N64: *mut WOW64_UNICODE_STRING64_M = Host as *mut WOW64_UNICODE_STRING64_M;
        let N32: *mut WOW64_UNICODE_STRING32_M = Buf32 as *mut WOW64_UNICODE_STRING32_M;
        let mut St: NTSTATUS = 0;

        if Host.is_null() {
            return (0xC000009A as ULONG) as NTSTATUS;
        }
        St = NtQueryObject(Obj, Class, Host as PVOID, HostLen, core::ptr::addr_of_mut!(HostRet));
        Guest = if HostRet > Grow {
            HostRet.wrapping_sub(Grow)
        } else {
            HostRet
        };
        if St >= 0
            && (Len32 as usize) < core::mem::size_of::<WOW64_UNICODE_STRING32_M>()
        {
            St = (0xC0000004 as ULONG) as NTSTATUS;
        }
        if St >= 0 {
            let mut Bytes: ULONG = if (*N64).Buffer != 0 {
                (*N64).MaximumLength as ULONG
            } else {
                0
            };
            if Bytes as usize
                > (Len32 as usize).wrapping_sub(core::mem::size_of::<WOW64_UNICODE_STRING32_M>())
            {
                Bytes = (Len32 as usize)
                    .wrapping_sub(core::mem::size_of::<WOW64_UNICODE_STRING32_M>()) as ULONG;
            }
            if Bytes != 0 {
                RtlCopyMemory!(
                    Buf32.add(core::mem::size_of::<WOW64_UNICODE_STRING32_M>()) as PVOID,
                    (*N64).Buffer as ULONG_PTR as PVOID,
                    Bytes as usize,
                );
            }
            (*N32).Length = (*N64).Length;
            (*N32).MaximumLength = Bytes as USHORT;
            (*N32).Buffer = if (*N64).Buffer != 0 {
                Buf32.add(core::mem::size_of::<WOW64_UNICODE_STRING32_M>()) as ULONG_PTR as ULONG
            } else {
                0
            };
        }
        if !RetLen32.is_null() {
            *RetLen32 = Guest;
        }
        Wow64pIoctlFree(Host as PVOID);
        return St;
    }

    DbgPrint(
        b"[WOW64]   NtQueryObject class=%lu unhandled\n\0".as_ptr() as *const i8,
        Class,
    );
    return (0xC0000003 as ULONG) as NTSTATUS;
}

unsafe extern "C" fn Wow64pLpcHeader32To64(
    In32: *const WOW64_PORT_MESSAGE32,
    Out64: *mut WOW64_PORT_MESSAGE64,
    DataLength: ULONG,
) {
    (*Out64).u1.s1.DataLength = (*In32).u1.s1.DataLength;
    (*Out64).u1.s1.TotalLength = (core::mem::size_of::<WOW64_PORT_MESSAGE64>() as usize)
        .wrapping_add(DataLength as usize) as SHORT;
    (*Out64).u2.s2.Type = (*In32).u2.s2.Type;
    (*Out64).u2.s2.DataInfoOffset = (*In32).u2.s2.DataInfoOffset;
    (*Out64).ClientIdProcess = (*In32).ClientIdProcess as ULONGLONG;
    (*Out64).ClientIdThread = (*In32).ClientIdThread as ULONGLONG;
    (*Out64).MessageId = (*In32).MessageId;
    (*Out64).__pad0 = 0 as ULONG;
    (*Out64).ClientViewSize = (*In32).ClientViewSize as ULONGLONG;
}

unsafe extern "C" fn Wow64pLpcHeader64To32(In64: *const WOW64_PORT_MESSAGE64, Out32: *mut WOW64_PORT_MESSAGE32, DataLength: ULONG) {
    (*Out32).u1.s1.DataLength = (*In64).u1.s1.DataLength;
    (*Out32).u1.s1.TotalLength = (core::mem::size_of::<WOW64_PORT_MESSAGE32>() as u32).wrapping_add(DataLength) as SHORT;
    (*Out32).u2.s2.Type = (*In64).u2.s2.Type;
    (*Out32).u2.s2.DataInfoOffset = (*In64).u2.s2.DataInfoOffset;
    (*Out32).ClientIdProcess = (*In64).ClientIdProcess as ULONG;
    (*Out32).ClientIdThread = (*In64).ClientIdThread as ULONG;
    (*Out32).MessageId = (*In64).MessageId;
    (*Out32).ClientViewSize = (*In64).ClientViewSize as ULONG;
}

unsafe extern "C" fn Wow64pCpy(mut d: *mut u8, mut s: *const u8, mut n: u32) {
    while n != 0 {
        n = n.wrapping_sub(1);
        *d = *s;
        d = d.offset(1);
        s = s.offset(1);
    }
}

unsafe extern "C" fn Wow64pLpcBadArg(Who: *const i8, Why: *const i8) -> NTSTATUS {
    // DbgPrint("[WOW64] %s REFUSED: %s\n", Who, Why);
    WOW64_STATUS_INVALID_PARAMETER
}

unsafe extern "C" fn Wow64pZeroBytes(mut d: *mut u8, mut n: u32) {
    while n != 0 {
        let fresh18 = n;
        n = n.wrapping_sub(1);
        let _ = fresh18;
        *d = 0 as u8;
        d = d.offset(1);
    }
}

unsafe extern "C" fn Wow64pCsrGet32(P: *const u8, Off: ULONG) -> ULONG {
    let mut V: ULONG = 0;
    Wow64pCpy(core::ptr::addr_of_mut!(V) as *mut u8, P.add(Off as usize), 4);
    return V;
}

unsafe extern "C" fn Wow64pCsrPut32(P: *mut u8, Off: ULONG, V: ULONG) {
    Wow64pCpy(P.add(Off as usize), core::ptr::addr_of!(V) as *const u8, 4);
}

unsafe extern "C" fn Wow64pCsrPut64(P: *mut u8, Off: ULONG, mut V: ULONGLONG) {
    Wow64pCpy(P.add(Off as usize), core::ptr::addr_of_mut!(V) as *const u8, 8);
}

unsafe extern "C" fn Wow64pCsrMapMsgOffset(Api: ULONG, Off32: ULONG, Why: *mut *const i8) -> ULONG {
    if Api == 0 && Off32 == WOW64_CSR_CLIENTCONNECT_PTR32 {
        return WOW64_CSR_CLIENTCONNECT_PTR64;
    }
    *Why = b"no host offset known for this api's pointer slot\0".as_ptr() as *const i8;
    return 0;
}

unsafe extern "C" fn Wow64pCsrWidenCaptureBuffer(Api: ULONG, Cap32: ULONG, Info32: ULONG, InfoOut: *mut ULONGLONG, Off64Out: *mut ULONG, BackTo: *mut *mut u8, BackFrom: *mut *mut u8, BackLen: *mut ULONG, Why: *mut *const i8) -> *mut u8 {
    let mut C32: *const u8 = core::ptr::null();
    let mut C64: *mut u8 = core::ptr::null_mut();
    let mut Local: ULONG_PTR = 0;
    let mut Size32: ULONG = 0;
    let mut Count: ULONG = 0;
    let mut Data32Off: ULONG = 0;
    let mut DataLen: ULONG = 0;
    let mut Total64: ULONG = 0;
    let mut i: ULONG = 0;

    if Wow64pCsrScratchSize == 0 {
        *Why = b"no scratch reserved in the CSR port view\0".as_ptr() as *const i8;
        return core::ptr::null_mut();
    }

    Local = (Cap32 as ULONG_PTR).wrapping_sub(Wow64pCsrPortDelta);
    if Local < Wow64pCsrViewLocal
        || Local >= Wow64pCsrViewLocal.wrapping_add(Wow64pCsrViewSize as ULONG_PTR)
    {
        DbgPrint(
            b"[WOW64] LPC: capture buffer %08lx (local %p) is outside the port view\n\0".as_ptr() as PCSTR,
            Cap32,
            Local as PVOID,
        );
        *Why = b"capture buffer is outside the CSR port view\0".as_ptr() as *const i8;
        return core::ptr::null_mut();
    }
    C32 = Local as *const u8;

    Size32 = Wow64pCsrGet32(C32, WOW64_CAPBUF32_SIZE as ULONG);
    Count = Wow64pCsrGet32(C32, WOW64_CAPBUF32_POINTERCOUNT as ULONG);

    if Count > 64
        || Size32 < (WOW64_CAPBUF32_HDR as ULONG).wrapping_add(Count.wrapping_mul(4 as ULONG))
        || Size32 > 0x10000
        || Local.wrapping_add(Size32 as ULONG_PTR)
            > Wow64pCsrViewLocal.wrapping_add(Wow64pCsrViewSize as ULONG_PTR)
    {
        *Why = b"capture buffer Size/PointerCount out of range\0".as_ptr() as *const i8;
        return core::ptr::null_mut();
    }

    Data32Off = (WOW64_CAPBUF32_HDR as ULONG).wrapping_add(Count.wrapping_mul(4 as ULONG));
    DataLen = Size32.wrapping_sub(Data32Off);
    Total64 = (WOW64_CAPBUF64_HDR as ULONG)
        .wrapping_add(Count.wrapping_mul(8 as ULONG))
        .wrapping_add(DataLen);

    if Total64 > Wow64pCsrScratchSize {
        *Why = b"widened capture buffer does not fit the port-view scratch\0".as_ptr() as *const i8;
        return core::ptr::null_mut();
    }

    if Info32 < Cap32.wrapping_add(Data32Off) || Info32 >= Cap32.wrapping_add(Size32) {
        *Why = b"api data pointer is outside the capture buffer\0".as_ptr() as *const i8;
        return core::ptr::null_mut();
    }

    C64 = Wow64pCsrScratchLocal as *mut u8;

    Wow64pCsrPut32(C64, WOW64_CAPBUF64_SIZE as ULONG, Total64);
    Wow64pCsrPut64(C64, WOW64_CAPBUF64_PREV as ULONG, 0 as ULONGLONG);
    Wow64pCsrPut32(C64, WOW64_CAPBUF64_POINTERCOUNT as ULONG, Count);
    Wow64pCsrPut64(
        C64,
        WOW64_CAPBUF64_BUFFEREND as ULONG,
        (Wow64pCsrScratchLocal
            .wrapping_add(Wow64pCsrPortDelta)
            .wrapping_add(Total64 as ULONG_PTR)) as ULONGLONG,
    );

    i = 0;
    while i < Count {
        let mut Off32: ULONG = Wow64pCsrGet32(
            C32,
            (WOW64_CAPBUF32_HDR as ULONG).wrapping_add(i.wrapping_mul(4 as ULONG)),
        );
        let mut Off64: ULONG = 0;
        if Off32 == 0 {
            Wow64pCsrPut64(
                C64,
                (WOW64_CAPBUF64_HDR as ULONG).wrapping_add(i.wrapping_mul(8 as ULONG)),
                0 as ULONGLONG,
            );
        } else {
            Off64 = Wow64pCsrMapMsgOffset(Api, Off32, Why);
            if Off64 == 0 {
                return core::ptr::null_mut();
            }
            Wow64pCsrPut64(
                C64,
                (WOW64_CAPBUF64_HDR as ULONG).wrapping_add(i.wrapping_mul(8 as ULONG)),
                Off64 as ULONGLONG,
            );
            *Off64Out = Off64;
        }
        i = i.wrapping_add(1);
    }

    Wow64pCpy(
        C64
            .offset(WOW64_CAPBUF64_HDR as isize)
            .offset(Count.wrapping_mul(8 as ULONG) as isize),
        C32.offset(Data32Off as isize),
        DataLen as u32,
    );

    *BackTo = Local.wrapping_add(Data32Off as ULONG_PTR) as *mut u8;
    *BackFrom = C64
        .offset(WOW64_CAPBUF64_HDR as isize)
        .offset(Count.wrapping_mul(8 as ULONG) as isize);
    *BackLen = DataLen;

    *InfoOut = (Wow64pCsrScratchLocal
        .wrapping_add(Wow64pCsrPortDelta)
        .wrapping_add(WOW64_CAPBUF64_HDR as ULONG_PTR)
        .wrapping_add(Count.wrapping_mul(8 as ULONG) as ULONG_PTR)
        .wrapping_add(Info32.wrapping_sub(Cap32.wrapping_add(Data32Off)) as ULONG_PTR)) as ULONGLONG;
    return C64;
}

unsafe extern "C" fn Wow64Manual_NtRequestWaitReplyPort(a: *mut ULONG) -> NTSTATUS {
    let mut Req32: *const WOW64_PORT_MESSAGE32 =
        *a.offset(1 as isize) as ULONG_PTR as *const WOW64_PORT_MESSAGE32;
    let mut Rep32: *mut WOW64_PORT_MESSAGE32 =
        *a.offset(2 as isize) as ULONG_PTR as *mut WOW64_PORT_MESSAGE32;
    let mut Port: HANDLE =
        *a.offset(0 as isize) as LONG as LONG_PTR as HANDLE;
    let mut Req64: *mut WOW64_PORT_MESSAGE64 = ::core::ptr::null_mut();
    let mut Rep64: *mut WOW64_PORT_MESSAGE64 = ::core::ptr::null_mut();
    let mut Cap64: *mut u8 = ::core::ptr::null_mut();
    let mut CapBackTo: *mut u8 = ::core::ptr::null_mut();
    let mut CapBackFrom: *mut u8 = ::core::ptr::null_mut();
    let mut CapBackLen: ULONG = 0 as ULONG;
    let mut Cap32Orig: ULONG = 0 as ULONG;
    let mut Info32Orig: ULONG = 0 as ULONG;
    let mut IsCreateThread: i32 = 0 as i32;
    let mut IsCreateProcess: i32 = 0 as i32;
    let mut DataLength: ULONG = 0;
    let mut RepData: ULONG = 0;
    let mut St: NTSTATUS = 0;

    if Req32.is_null() || Rep32.is_null() {
        return Wow64pLpcBadArg(
            b"NtRequestWaitReplyPort\0".as_ptr() as *const i8,
            b"NULL request or reply buffer\0".as_ptr() as *const i8,
        );
    }

    /* DataLength is a CSHORT: a negative value would widen into a huge ULONG. */
    if (*Req32).u1.s1.DataLength < 0 as SHORT
        || (*Req32).u1.s1.TotalLength < 0 as SHORT
    {
        return Wow64pLpcBadArg(
            b"NtRequestWaitReplyPort\0".as_ptr() as *const i8,
            b"negative DataLength/TotalLength\0".as_ptr() as *const i8,
        );
    }

    DataLength = (*Req32).u1.s1.DataLength as ULONG;
    if DataLength > WOW64_LPC_MAX_DATA32 as ULONG {
        return Wow64pLpcBadArg(
            b"NtRequestWaitReplyPort\0".as_ptr() as *const i8,
            b"DataLength past the guest's own ceiling\0".as_ptr() as *const i8,
        );
    }

    /* The guest's own TotalLength must at least cover its header + data, or the
     * message was malformed before we ever touched it. */
    if ((*Req32).u1.s1.TotalLength as ULONG as usize)
        < (::core::mem::size_of::<WOW64_PORT_MESSAGE32>()).wrapping_add(DataLength as usize)
    {
        return Wow64pLpcBadArg(
            b"NtRequestWaitReplyPort\0".as_ptr() as *const i8,
            b"TotalLength shorter than header + DataLength\0".as_ptr() as *const i8,
        );
    }

    Req64 = Wow64pIoctlAlloc(
        (::core::mem::size_of::<WOW64_PORT_MESSAGE64>() as SIZE_T)
            .wrapping_add(WOW64_LPC_MAX_DATA32 as SIZE_T),
    ) as *mut WOW64_PORT_MESSAGE64;
    Rep64 = Wow64pIoctlAlloc(
        (::core::mem::size_of::<WOW64_PORT_MESSAGE64>() as SIZE_T)
            .wrapping_add(WOW64_LPC_MAX_DATA32 as SIZE_T),
    ) as *mut WOW64_PORT_MESSAGE64;
    if Req64.is_null() || Rep64.is_null() {
        Wow64pIoctlFree(Req64 as PVOID);
        Wow64pIoctlFree(Rep64 as PVOID);
        return WOW64_STATUS_NO_MEMORY;
    }

    {
        let B32: *const u8 =
            (Req32 as *const u8).offset(::core::mem::size_of::<WOW64_PORT_MESSAGE32>() as isize);
        let B64: *mut u8 =
            (Req64 as *mut u8).offset(::core::mem::size_of::<WOW64_PORT_MESSAGE64>() as isize);
        let Api: ULONG = Wow64pCsrGet32(B32, WOW64_CSRMSG_BODY32_APINUMBER);
        let Data32Len: ULONG = if DataLength > WOW64_CSRMSG_BODY32_DATA as ULONG {
            DataLength.wrapping_sub(WOW64_CSRMSG_BODY32_DATA as ULONG)
        } else {
            0 as ULONG
        };

        /* Prologue, explicit and universal across every api. */
        Wow64pCsrPut64(
            B64,
            0,
            Wow64pCsrGet32(B32, 0) as ULONGLONG,
        ); /* CsrCaptureData */
        Wow64pCsrPut32(B64, WOW64_CSRMSG_BODY64_APINUMBER, Api);
        Wow64pCsrPut32(
            B64,
            WOW64_CSRMSG_BODY64_STATUS,
            Wow64pCsrGet32(B32, WOW64_CSRMSG_BODY32_STATUS),
        );
        Wow64pCsrPut32(
            B64,
            WOW64_CSRMSG_BODY64_RESERVED,
            Wow64pCsrGet32(B32, WOW64_CSRMSG_BODY32_RESERVED),
        );

        /* The per-api Data union is NOT width-safe in general. Measured on a
         * real boot, the only api a guest sends here is 0 (CsrSrvClientConnect),
         * whose CSR_CLIENT_CONNECT is {ULONG ServerId, PVOID ConnectionInfo,
         * ULONG ConnectionInfoSize} -- one pointer to widen. Anything else is
         * copied verbatim AND named on serial, so a new api shows up as a loud
         * line rather than as silent corruption. */
        if Api == 0 as ULONG && Data32Len >= 12 as ULONG {
            let Cap32: ULONG = Wow64pCsrGet32(B32, 0);
            Cap32Orig = Cap32;
            let Info32: ULONG = Wow64pCsrGet32(
                B32,
                WOW64_CSRMSG_BODY32_DATA + 4,
            );

            let mut Info64: ULONGLONG = 0 as ULONGLONG;
            let mut Off64Unused: ULONG = 0 as ULONG;
            let mut Why: *const i8 = b"unknown\0".as_ptr() as *const i8;

            Wow64pCsrPut32(
                B64,
                WOW64_CSRMSG_BODY64_DATA + 0,
                Wow64pCsrGet32(B32, WOW64_CSRMSG_BODY32_DATA + 0),
            ); /* ServerId */
            Wow64pCsrPut32(
                B64,
                WOW64_CSRMSG_BODY64_DATA + 16,
                Wow64pCsrGet32(B32, WOW64_CSRMSG_BODY32_DATA + 8),
            ); /* ConnectionInfoSize */
            (*Req64).u1.s1.DataLength = (WOW64_CSRMSG_BODY64_DATA + 20) as SHORT;

            /* csrss reads the capture buffer out of THIS process with host field
             * offsets, so the guest's 16-byte-header copy fails its bounds check
             * and the api answers c000000d. Hand it a host-layout one. */
            if Cap32 != 0 {
                Info32Orig = Info32;
                Cap64 = Wow64pCsrWidenCaptureBuffer(
                    Api,
                    Cap32,
                    Info32,
                    ::core::ptr::addr_of_mut!(Info64),
                    ::core::ptr::addr_of_mut!(Off64Unused),
                    ::core::ptr::addr_of_mut!(CapBackTo),
                    ::core::ptr::addr_of_mut!(CapBackFrom),
                    ::core::ptr::addr_of_mut!(CapBackLen),
                    ::core::ptr::addr_of_mut!(Why),
                );
                if Cap64.is_null() {
                    Wow64pIoctlFree(Req64 as PVOID);
                    Wow64pIoctlFree(Rep64 as PVOID);
                    return Wow64pLpcBadArg(
                        b"NtRequestWaitReplyPort\0".as_ptr() as *const i8,
                        Why,
                    );
                }
                /* csrss dereferences this in ITS address space. */
                Wow64pCsrPut64(
                    B64,
                    0,
                    ((Cap64 as ULONG_PTR).wrapping_add(Wow64pCsrPortDelta)) as ULONGLONG,
                );
                Wow64pCsrPut64(B64, WOW64_CSRMSG_BODY64_DATA + 8, Info64);
            } else {
                Wow64pCsrPut64(
                    B64,
                    WOW64_CSRMSG_BODY64_DATA + 8,
                    Info32 as ULONGLONG,
                );
            }
        } else if Api == WOW64_CSRAPI_BASEP_CREATEPROCESS as ULONG
            && Data32Len >= WOW64_BCP32_SIZE as ULONG
        {
            /* BASE_CREATE_PROCESS -- the message CreateProcess sends basesrv
             * once the child and its initial thread exist. It opens with
             * HANDLE, HANDLE, _CLIENT_ID, i.e. FOUR pointer-width fields in a
             * row, so the verbatim copy handed basesrv a ProcessHandle and a
             * ClientId assembled from the wrong words and it answered
             * c0000008; the guest reported CreateProcessW err 0x6
             * (ERROR_INVALID_HANDLE). Identical failure to BASEP_CREATETHREAD
             * below, one struct larger.
             *
             * Offsets MEASURED, not derived -- offsetof() dumps from the real
             * basemsg.h structs built with the llvm-mingw i686 and x86_64
             * targets. The control on that measurement is sizeof: 140/96 on
             * i386 and 232/160 on amd64, which is exactly what the tree
             * already asserts in ntdll-sys/src/size_check_{i386,amd64}.rs, so
             * a bad transcription would have shown up as a size mismatch
             * rather than as quietly wrong offsets.
             *
             *   field                  i386    amd64
             *   ProcessHandle          0x00    0x00
             *   ThreadHandle           0x04    0x08
             *   ClientId.UniqueProcess 0x08    0x10
             *   ClientId.UniqueThread  0x0c    0x18
             *   CreationFlags          0x10    0x20
             *   VdmBinaryType          0x14    0x24
             *   VdmTask                0x18    0x28
             *   hVDM                   0x1c    0x30
             *   Sxs                    0x20    0x38
             *   PebAddressNative       0x80    0xd8
             *   PebAddressWow64        0x84    0xe0
             *   ProcessorArchitecture  0x88    0xe4
             *   sizeof                 0x8c    0xe8
             *
             * ProcessHandle is NOT purely a handle: kernel32 packs up to two
             * flag bits into its low bits (AddToHandle/RemoveFromHandle) and
             * BaseSrvCreateProcess reads them back out as `Handle & 3` before
             * masking. Zero-extension preserves them, which is the only
             * reason the naive widening is right here.
             *
             * Sxs is the one field NOT marshalled. BaseSrvCreateProcess never
             * reads it -- the SxS work belongs to a different API
             * (BaseSrvSxsCreateActivationContext) -- so widening thirty fields
             * including four UNICODE_STRINGs whose Buffers are guest pointers
             * would buy nothing today. It is ZEROED rather than left as
             * whatever the verbatim copy put there, and the guest is told when
             * it mattered, so the day the server starts reading it this
             * announces itself instead of going quiet. */
            IsCreateProcess = 1 as i32;

            /* Zero first: every byte the server might read is then either a
             * field written below or a deliberate zero, never leftover. */
            Wow64pZeroBytes(
                (B64).offset(WOW64_CSRMSG_BODY64_DATA as isize),
                WOW64_BCP64_SIZE as u32,
            );

            WOW64_BCP_WIDEN!(B32, B64, 0x00, 0x00); /* ProcessHandle (+ flag bits)   */
            WOW64_BCP_WIDEN!(B32, B64, 0x04, 0x08); /* ThreadHandle                  */
            WOW64_BCP_WIDEN!(B32, B64, 0x08, 0x10); /* ClientId.UniqueProcess        */
            WOW64_BCP_WIDEN!(B32, B64, 0x0c, 0x18); /* ClientId.UniqueThread         */
            WOW64_BCP_COPY32!(B32, B64, 0x10, 0x20); /* CreationFlags        (ULONG)  */
            WOW64_BCP_COPY32!(B32, B64, 0x14, 0x24); /* VdmBinaryType        (ULONG)  */
            WOW64_BCP_COPY32!(B32, B64, 0x18, 0x28); /* VdmTask              (ULONG)  */
            WOW64_BCP_WIDEN!(B32, B64, 0x1c, 0x30); /* hVDM                          */
            WOW64_BCP_WIDEN!(B32, B64, 0x80, 0xd8); /* PebAddressNative              */
            WOW64_BCP_COPY32!(B32, B64, 0x84, 0xe0); /* PebAddressWow64      (ULONG)  */
            WOW64_BCP_COPY32!(B32, B64, 0x88, 0xe4); /* ProcessorArchitecture(USHORT) */

            if Wow64pCsrGet32(
                B32,
                WOW64_CSRMSG_BODY32_DATA + 0x20,
            ) != 0 as ULONG
            {
                DbgPrint(
                    b"[WOW64] LPC BasepCreateProcess: Sxs.Flags=%08lx is NOT marshalled (basesrv ignores Sxs today) -- if this starts mattering, widen BASE_SXS_CREATEPROCESS_MSG\n\0"
                        .as_ptr() as *const i8,
                    Wow64pCsrGet32(B32, WOW64_CSRMSG_BODY32_DATA + 0x20),
                );
            }

            (*Req64).u1.s1.DataLength = (WOW64_CSRMSG_BODY64_DATA + WOW64_BCP64_SIZE) as SHORT;
        } else if Api == WOW64_CSRAPI_BASEP_CREATETHREAD as ULONG
            && Data32Len >= 12 as ULONG
        {
            /* BASE_CREATE_THREAD { HANDLE ThreadHandle; _CLIENT_ID ClientId; } --
             * three pointer-width fields in a row, so the verbatim copy handed
             * csrss a ClientId built from the wrong words and it answered
             * c0000001; the guest reported CreateThread err 0x1f. MEASURED:
             *   i386   ThreadHandle=0 UniqueProcess=4  UniqueThread=8  sizeof=12
             *   amd64  ThreadHandle=0 UniqueProcess=8  UniqueThread=16 sizeof=24
             * These are handle/id VALUES, not guest pointers, so zero-extending
             * is the whole conversion. */
            IsCreateThread = 1 as i32;
            Wow64pCsrPut64(
                B64,
                WOW64_CSRMSG_BODY64_DATA + 0,
                Wow64pCsrGet32(B32, WOW64_CSRMSG_BODY32_DATA + 0) as ULONGLONG,
            );
            Wow64pCsrPut64(
                B64,
                WOW64_CSRMSG_BODY64_DATA + 8,
                Wow64pCsrGet32(B32, WOW64_CSRMSG_BODY32_DATA + 4) as ULONGLONG,
            );
            Wow64pCsrPut64(
                B64,
                WOW64_CSRMSG_BODY64_DATA + 16,
                Wow64pCsrGet32(B32, WOW64_CSRMSG_BODY32_DATA + 8) as ULONGLONG,
            );
            (*Req64).u1.s1.DataLength = (WOW64_CSRMSG_BODY64_DATA + 24) as SHORT;
        } else {
            /* A verbatim body copy is CORRECT when the payload holds no
             * pointer-width field, and shouting about those trains the reader
             * to ignore the line that matters. Each entry here was read against
             * its struct, not assumed:
             *   0x00010003 BasepExitProcess            { UINT uExitCode; }
             *   0x0001000c BasepSetProcessShutdownParam{ ULONG Level, Flags; }
             * (reactos/subsys/win/basemsg.h:113, :219) */
            if Api != 0x00010003 as ULONG && Api != 0x0001000C as ULONG {
                DbgPrint(
                    b"[WOW64] LPC: api %08lx has NO payload marshaller \xE2\x80\x94 body copied verbatim, fields past CsrCaptureData may be wrong\n\0"
                        .as_ptr() as *const i8,
                    Api,
                );
            }
            Wow64pCpy(
                (B64).offset(WOW64_CSRMSG_BODY64_DATA as isize),
                (B32).offset(WOW64_CSRMSG_BODY32_DATA as isize),
                Data32Len as u32,
            );
            (*Req64).u1.s1.DataLength =
                (WOW64_CSRMSG_BODY64_DATA as ULONG).wrapping_add(Data32Len) as SHORT;
        }
        (*Req64).u1.s1.TotalLength = ((::core::mem::size_of::<WOW64_PORT_MESSAGE64>() as usize)
            .wrapping_add((*Req64).u1.s1.DataLength as usize)) as SHORT;
    }

    /* The census this rung exists for: which CSR APIs does a guest actually
     * send? ApiNumber encodes server index + API. Counting beats guessing —
     * ~100 message layouts exist and the reachable set is far smaller. */
    if DataLength >= 8 as ULONG {
        let Bl: *const u8 =
            (Req32 as *const u8).offset(::core::mem::size_of::<WOW64_PORT_MESSAGE32>() as isize);
        /* CsrCaptureData decides which arm of CsrValidateMessageBuffer runs:
         * NULL means "same-process only", and a cross-process client is then
         * rejected -- which is the c000000d the api is returning. Print it with
         * the connect args instead of inferring which case this is. */
        DbgPrint(
            b"[WOW64] LPC request: Api=%08lx DataLen=%lu Capture=%08lx ServerId=%08lx Info=%08lx InfoSize=%08lx\n\0"
                .as_ptr() as *const i8,
            Wow64pCsrGet32(Bl, WOW64_CSRMSG_BODY32_APINUMBER),
            DataLength,
            Wow64pCsrGet32(Bl, 0),
            Wow64pCsrGet32(Bl, WOW64_CSRMSG_BODY32_DATA + 0),
            Wow64pCsrGet32(Bl, WOW64_CSRMSG_BODY32_DATA + 4),
            Wow64pCsrGet32(Bl, WOW64_CSRMSG_BODY32_DATA + 8),
        );
    }

    St = NtRequestWaitReplyPort(Port, (Req64 as PVOID).cast(), (Rep64 as PVOID).cast());

    if St >= 0 as NTSTATUS {
        let R64: *const u8 =
            (Rep64 as *const u8).offset(::core::mem::size_of::<WOW64_PORT_MESSAGE64>() as isize);
        let R32: *mut u8 =
            (Rep32 as *mut u8).offset(::core::mem::size_of::<WOW64_PORT_MESSAGE32>() as isize);
        let mut ApiStatus: ULONG = 0;

        RepData = if (*Rep64).u1.s1.DataLength < 0 as SHORT {
            0 as ULONG
        } else {
            (*Rep64).u1.s1.DataLength as ULONG
        };
        if RepData > WOW64_LPC_MAX_DATA32 as ULONG {
            RepData = WOW64_LPC_MAX_DATA32 as ULONG;
        }

        /* Narrow the prologue the same way, or the guest reads its Status out of
         * the wrong word -- the transport status below is NOT the api's answer. */
        /* csrss echoes the message back, so CsrCaptureData in the reply is the
         * SCRATCH buffer we substituted, not the guest's. Handing that back
         * makes the guest compute scratch = reply - CsrPortMemoryDelta and
         * RtlFreeHeap a block its heap never owned. Give it its own pointer. */
        Wow64pCsrPut32(
            R32,
            0,
            if Cap32Orig != 0 {
                Cap32Orig
            } else {
                Wow64pCsrGet32(R64, 0)
            },
        );
        Wow64pCsrPut32(
            R32,
            WOW64_CSRMSG_BODY32_APINUMBER,
            Wow64pCsrGet32(R64, WOW64_CSRMSG_BODY64_APINUMBER),
        );
        ApiStatus = Wow64pCsrGet32(R64, WOW64_CSRMSG_BODY64_STATUS);
        Wow64pCsrPut32(R32, WOW64_CSRMSG_BODY32_STATUS, ApiStatus);
        Wow64pCsrPut32(
            R32,
            WOW64_CSRMSG_BODY32_RESERVED,
            Wow64pCsrGet32(R64, WOW64_CSRMSG_BODY64_RESERVED),
        );
        if RepData > WOW64_CSRMSG_BODY64_DATA as ULONG {
            Wow64pCpy(
                (R32).offset(WOW64_CSRMSG_BODY32_DATA as isize),
                (R64).offset(WOW64_CSRMSG_BODY64_DATA as isize),
                RepData.wrapping_sub(WOW64_CSRMSG_BODY64_DATA as ULONG) as u32,
            );
        }

        if IsCreateProcess != 0 {
            /* basesrv passes &ClientId to CsrCreateProcess by reference, and
             * the verbatim reply copy above laid 64-bit words into a struct the
             * guest reads with 32-bit offsets. kernel32 happens to fill
             * PROCESS_INFORMATION from its OWN local ClientId rather than from
             * the reply, so nothing reads this today -- narrow it anyway, so
             * the guest is not left holding a struct that is wrong in a way
             * only the next reader discovers. */
            WOW64_BCP_NARROW!(R32, R64, 0x00, 0x00); /* ProcessHandle                 */
            WOW64_BCP_NARROW!(R32, R64, 0x04, 0x08); /* ThreadHandle                  */
            WOW64_BCP_NARROW!(R32, R64, 0x08, 0x10); /* ClientId.UniqueProcess        */
            WOW64_BCP_NARROW!(R32, R64, 0x0c, 0x18); /* ClientId.UniqueThread         */
            WOW64_BCP_NARROW!(R32, R64, 0x10, 0x20); /* CreationFlags                 */
            WOW64_BCP_NARROW!(R32, R64, 0x14, 0x24); /* VdmBinaryType                 */
            WOW64_BCP_NARROW!(R32, R64, 0x18, 0x28); /* VdmTask                       */
            WOW64_BCP_NARROW!(R32, R64, 0x1c, 0x30); /* hVDM                          */
        }

        if IsCreateThread != 0 {
            Wow64pCsrPut32(
                R32,
                WOW64_CSRMSG_BODY32_DATA + 0,
                Wow64pCsrGet32(R64, WOW64_CSRMSG_BODY64_DATA + 0),
            );
            Wow64pCsrPut32(
                R32,
                WOW64_CSRMSG_BODY32_DATA + 4,
                Wow64pCsrGet32(R64, WOW64_CSRMSG_BODY64_DATA + 8),
            );
            Wow64pCsrPut32(
                R32,
                WOW64_CSRMSG_BODY32_DATA + 8,
                Wow64pCsrGet32(R64, WOW64_CSRMSG_BODY64_DATA + 16),
            );
        }

        if Cap32Orig != 0 {
            /* The Data union is NOT width-safe, so the verbatim copy above put
             * the 64-bit padding word where the guest reads ConnectionInfo --
             * it then computed NULL - CsrPortMemoryDelta and faulted reading
             * there. Narrow the three fields by hand, the way the request did,
             * and give the guest back its OWN pointers rather than our scratch. */
            Wow64pCsrPut32(
                R32,
                WOW64_CSRMSG_BODY32_DATA + 0,
                Wow64pCsrGet32(R64, WOW64_CSRMSG_BODY64_DATA + 0),
            ); /* ServerId */
            Wow64pCsrPut32(
                R32,
                WOW64_CSRMSG_BODY32_DATA + 4,
                Info32Orig,
            );
            Wow64pCsrPut32(
                R32,
                WOW64_CSRMSG_BODY32_DATA + 8,
                Wow64pCsrGet32(R64, WOW64_CSRMSG_BODY64_DATA + 16),
            ); /* InfoSize */

            if CapBackLen != 0 {
                Wow64pCpy(CapBackTo, CapBackFrom, CapBackLen as u32);
            }
        }

        Wow64pLpcHeader64To32(
            Rep64,
            Rep32,
            (WOW64_CSRMSG_BODY32_DATA as ULONG).wrapping_add(if RepData > WOW64_CSRMSG_BODY64_DATA as ULONG {
                RepData.wrapping_sub(WOW64_CSRMSG_BODY64_DATA as ULONG)
            } else {
                0 as ULONG
            }),
        );

        /* The api's OWN result. A green transport status over a rejected api is
         * exactly the false-green this rung must not report as working. */
        DbgPrint(
            b"[WOW64] LPC reply: api Status=%08lx\n\0".as_ptr() as *const i8,
            ApiStatus,
        );
    }

    DbgPrint(
        b"[WOW64] NtRequestWaitReplyPort => %08lx\n\0".as_ptr() as *const i8,
        St,
    );
    /* Cap64 lives in the port-view scratch, not the ioctl heap — nothing to free. */
    Wow64pIoctlFree(Req64 as PVOID);
    Wow64pIoctlFree(Rep64 as PVOID);
    return St;
}

unsafe extern "C" fn Wow64pIoctlBadArg(Who: *const i8, Why: *const i8) -> NTSTATUS {
    // DbgPrint("[WOW64] %s: malformed 32-bit IOCTL payload: %s\n", Who, Why);
    WOW64_STATUS_INVALID_PARAMETER
}

unsafe extern "C" fn Wow64pIoctlCall(mut Fn: WOW64_IOCTL_FN, mut File: HANDLE, mut Event: HANDLE, mut I64: *mut WOW64_IO_STATUS_BLOCK64_M, mut Code: ULONG, mut In: PVOID, mut InLen: ULONG, mut Out: PVOID, mut OutLen: ULONG, mut Who: *const ::core::ffi::c_char) -> NTSTATUS {
    let mut St: NTSTATUS = (Fn.unwrap_unchecked())(File, Event, core::ptr::null_mut(), core::ptr::null_mut(), (I64).cast(), Code, In, InLen, Out, OutLen);
    let mut Wait: NTSTATUS;

    if St != WOW64_STATUS_PENDING {
        return St;
    }

    Wait = NtWaitForSingleObject(if !Event.is_null() { Event } else { File }, (FALSE) as BOOLEAN, core::ptr::null_mut());
    if Wait < 0 {
        /* The shadow is about to be freed under a request the driver still
         * owns. Nothing here can make that safe, so say so rather than free
         * quietly and corrupt later. */
        DbgPrint(b"[WOW64]   %s: wait on a pended marshalled IOCTL failed %08lx \xE2\x80\x94 the driver may still own our shadow buffer\n\0" as *const u8 as PCSTR, Who, Wait);
        return Wait;
    }
    if !Event.is_null() {
        NtSetEvent(Event, core::ptr::null_mut());
    }
    return (*I64).Status as ULONG as NTSTATUS;
}

unsafe extern "C" fn Wow64pAfdWidenBufs(Buf32Ptr: ULONG, Count: ULONG, Who: *const ::core::ffi::c_char, St: *mut NTSTATUS) -> *mut WOW64_AFD_WSABUF64 {
    let mut B32: *const WOW64_AFD_WSABUF32 = Buf32Ptr as ULONG_PTR as *const WOW64_AFD_WSABUF32;
    let mut B64: *mut WOW64_AFD_WSABUF64 = ::core::ptr::null_mut::<WOW64_AFD_WSABUF64>();
    let mut i: ULONG = 0;

    *St = 0;
    if Count == 0 {
        return ::core::ptr::null_mut::<WOW64_AFD_WSABUF64>(); /* legal: a zero-buffer transfer */
    }
    if Count > WOW64_AFD_MAX_BUFFERS as ULONG {
        *St = Wow64pIoctlBadArg(
            Who,
            b"BufferCount above the marshalling cap\0" as *const u8 as *const ::core::ffi::c_char,
        );
        return ::core::ptr::null_mut::<WOW64_AFD_WSABUF64>();
    }
    if B32.is_null() {
        *St = Wow64pIoctlBadArg(
            Who,
            b"non-zero BufferCount with a NULL BufferArray\0" as *const u8 as *const ::core::ffi::c_char,
        );
        return ::core::ptr::null_mut::<WOW64_AFD_WSABUF64>();
    }
    B64 = Wow64pIoctlAlloc(
        (Count as SIZE_T).wrapping_mul(::core::mem::size_of::<WOW64_AFD_WSABUF64>() as SIZE_T),
    ) as *mut WOW64_AFD_WSABUF64;
    if B64.is_null() {
        *St = WOW64_STATUS_NO_MEMORY;
        return ::core::ptr::null_mut::<WOW64_AFD_WSABUF64>();
    }
    i = 0;
    while i < Count {
        (*B64.offset(i as isize)).len = (*B32.offset(i as isize)).len;
        (*B64.offset(i as isize)).buf = (*B32.offset(i as isize)).buf as ULONG_PTR as ULONGLONG; /* guest VA == host VA */
        i += 1;
    }
    return B64;
}

unsafe extern "C" fn Wow64pIoctlAfdXfer(Fn: WOW64_IOCTL_FN, a: *mut ULONG, Who: *const i8) -> NTSTATUS {
    let In32: *const WOW64_AFD_XFER32 = *a.add(6) as ULONG_PTR as *const WOW64_AFD_XFER32;
    let mut Iosb64: WOW64_IO_STATUS_BLOCK64_M = WOW64_IO_STATUS_BLOCK64_M {
        Status: 0 as ULONGLONG,
        Information: 0,
    };
    let mut B64: *mut WOW64_AFD_WSABUF64 = core::ptr::null_mut();
    let mut In64: *mut WOW64_AFD_XFER64 = core::ptr::null_mut();
    let File: HANDLE = *a.add(0) as LONG as LONG_PTR as HANDLE;
    let Event: HANDLE = *a.add(1) as LONG as LONG_PTR as HANDLE;
    let mut St: NTSTATUS = 0;

    if In32.is_null()
        || ((*a.add(7)) as usize) < core::mem::size_of::<WOW64_AFD_XFER32>() as usize
    {
        return Wow64pIoctlBadArg(Who, b"input shorter than AFD_SEND_INFO/AFD_RECV_INFO\0" as *const u8 as *const i8);
    }

    B64 = Wow64pAfdWidenBufs((*In32).BufferArray, (*In32).BufferCount, Who, core::ptr::addr_of_mut!(St));
    if St != 0 {
        return St;
    }

    In64 = Wow64pIoctlAlloc(104) as *mut WOW64_AFD_XFER64;
    if In64.is_null() {
        Wow64pIoctlFree((B64).cast());
        return WOW64_STATUS_NO_MEMORY;
    }
    (*In64).BufferArray = B64 as ULONG_PTR as ULONGLONG;
    (*In64).BufferCount = (*In32).BufferCount;
    (*In64).AfdFlags = (*In32).AfdFlags;
    (*In64).TdiFlags = (*In32).TdiFlags;

    St = Wow64pIoctlCall(
        Fn,
        File,
        Event,
        core::ptr::addr_of_mut!(Iosb64),
        *a.add(5),
        In64 as PVOID,
        core::mem::size_of::<WOW64_AFD_XFER64>() as ULONG,
        *a.add(8) as ULONG_PTR as PVOID,
        *a.add(9),
        Who,
    );
    Wow64pIoctlFree(In64 as PVOID);
    Wow64pIoctlFree(B64 as PVOID);
    return Wow64pFinishIosb(St, File, core::ptr::addr_of_mut!(Iosb64), *a.add(4), Who);
}

unsafe extern "C" fn Wow64pIoctlAfdRecvFrom(Fn: WOW64_IOCTL_FN, a: *mut ULONG, Who: *const i8) -> NTSTATUS {
    let mut In32: *const WOW64_AFD_XFERUDP32 =
        *a.offset(6 as isize) as ULONG_PTR as *const WOW64_AFD_XFERUDP32;
    let mut Iosb64: WOW64_IO_STATUS_BLOCK64_M = WOW64_IO_STATUS_BLOCK64_M {
        Status: 0,
        Information: 0,
    };
    let mut B64: *mut WOW64_AFD_WSABUF64 = core::ptr::null_mut();
    let mut In64: WOW64_AFD_XFERUDP64 = WOW64_AFD_XFERUDP64 {
        BufferArray: 0,
        BufferCount: 0,
        AfdFlags: 0,
        TdiFlags: 0,
        _p: 0,
        Address: 0,
        AddressLength: 0,
    };
    let mut File: HANDLE =
        *a.offset(0 as isize) as LONG as LONG_PTR as HANDLE;
    let mut Event: HANDLE =
        *a.offset(1 as isize) as LONG as LONG_PTR as HANDLE;
    let mut St: NTSTATUS = 0;

    if In32.is_null()
        || (*a.offset(7 as isize) as usize)
            < core::mem::size_of::<WOW64_AFD_XFERUDP32>() as usize
    {
        return Wow64pIoctlBadArg(
            Who,
            b"input shorter than AFD_RECV_INFO_UDP\0".as_ptr() as *const i8,
        );
    }

    B64 = Wow64pAfdWidenBufs(
        (*In32).BufferArray,
        (*In32).BufferCount,
        Who,
        core::ptr::addr_of_mut!(St),
    );
    if St != 0 {
        return St;
    }

    In64.BufferArray = B64 as ULONG_PTR as ULONGLONG;
    In64.BufferCount = (*In32).BufferCount;
    In64.AfdFlags = (*In32).AfdFlags;
    In64.TdiFlags = (*In32).TdiFlags;
    In64.Address = (*In32).Address as ULONG_PTR as ULONGLONG;
    In64.AddressLength = (*In32).AddressLength as ULONG_PTR as ULONGLONG;

    St = Wow64pIoctlCall(
        Fn,
        File,
        Event,
        core::ptr::addr_of_mut!(Iosb64),
        *a.offset(5 as isize),
        core::ptr::addr_of_mut!(In64) as PVOID,
        core::mem::size_of::<WOW64_AFD_XFERUDP64>() as ULONG,
        *a.offset(8 as isize) as ULONG_PTR as PVOID,
        *a.offset(9 as isize),
        Who,
    );
    Wow64pIoctlFree(B64 as PVOID);
    return Wow64pFinishIosb(
        St,
        File,
        core::ptr::addr_of_mut!(Iosb64),
        *a.offset(4 as isize),
        Who,
    );
}

unsafe extern "C" fn Wow64pIoctlAfdSendTo(Fn: WOW64_IOCTL_FN, a: *mut ULONG, Who: *const i8) -> NTSTATUS {
    let In32: *const WOW64_AFD_SENDUDP32 =
        *a.offset(6 as isize) as ULONG_PTR as *const WOW64_AFD_SENDUDP32;
    let mut Iosb64: WOW64_IO_STATUS_BLOCK64_M = WOW64_IO_STATUS_BLOCK64_M {
        Status: 0,
        Information: 0,
    };
    let mut B64: *mut WOW64_AFD_WSABUF64 = core::ptr::null_mut();
    let mut In64: WOW64_AFD_SENDUDP64 = WOW64_AFD_SENDUDP64 {
        BufferArray: 0,
        BufferCount: 0,
        AfdFlags: 0,
        TdiRequest: [0; 5],
        UserDataLength: 0,
        _p0: 0,
        UserData: 0,
        OptionsLength: 0,
        _p1: 0,
        Options: 0,
        RemoteAddressLength: 0,
        _p2: 0,
        RemoteAddress: 0,
    };
    let File: HANDLE = *a.offset(0 as isize) as LONG as LONG_PTR as HANDLE;
    let Event: HANDLE = *a.offset(1 as isize) as LONG as LONG_PTR as HANDLE;
    let mut St: NTSTATUS = 0;

    if In32.is_null()
        || (*a.offset(7 as isize) as usize)
            < core::mem::size_of::<WOW64_AFD_SENDUDP32>()
    {
        return Wow64pIoctlBadArg(
            Who,
            b"input shorter than AFD_SEND_INFO_UDP\0".as_ptr() as *const i8,
        );
    }
    if (*In32).RemoteAddress == 0 {
        return Wow64pIoctlBadArg(
            Who,
            b"AFD_SEND_INFO_UDP with a NULL RemoteAddress\0".as_ptr() as *const i8,
        );
    }

    B64 = Wow64pAfdWidenBufs(
        (*In32).BufferArray,
        (*In32).BufferCount,
        Who,
        core::ptr::addr_of_mut!(St),
    );
    if St != 0 {
        return St;
    }

    In64.BufferArray = B64 as ULONG_PTR as ULONGLONG;
    In64.BufferCount = (*In32).BufferCount;
    In64.AfdFlags = (*In32).AfdFlags;
    In64.UserDataLength = (*In32).UserDataLength;
    In64.UserData = (*In32).UserData as ULONG_PTR as ULONGLONG;
    In64.OptionsLength = (*In32).OptionsLength;
    In64.Options = (*In32).Options as ULONG_PTR as ULONGLONG;
    In64.RemoteAddressLength = (*In32).RemoteAddressLength;
    In64.RemoteAddress = (*In32).RemoteAddress as ULONG_PTR as ULONGLONG;

    St = Wow64pIoctlCall(
        Fn,
        File,
        Event,
        core::ptr::addr_of_mut!(Iosb64),
        *a.offset(5 as isize),
        core::ptr::addr_of_mut!(In64) as PVOID,
        core::mem::size_of::<WOW64_AFD_SENDUDP64>() as ULONG,
        (*a.offset(8 as isize) as ULONG_PTR) as PVOID,
        *a.offset(9 as isize),
        Who,
    );
    Wow64pIoctlFree(B64 as PVOID);
    return Wow64pFinishIosb(
        St,
        File,
        core::ptr::addr_of_mut!(Iosb64),
        *a.offset(4 as isize),
        Who,
    );
}

unsafe extern "C" fn Wow64pIoctlAfdSelect(Fn: WOW64_IOCTL_FN, a: *mut ULONG, Who: *const i8) -> NTSTATUS {
    let mut In32: *const WOW64_AFD_POLLHDR32 =
        *a.offset(6 as isize) as ULONG_PTR as *const WOW64_AFD_POLLHDR32;
    let mut Out32: *mut WOW64_AFD_POLLHDR32 = (if *a.offset(8 as isize) != 0 {
        *a.offset(8 as isize)
    } else {
        *a.offset(6 as isize)
    }) as ULONG_PTR as *mut WOW64_AFD_POLLHDR32;
    let mut H32: *const WOW64_AFD_HANDLE32 = ::core::ptr::null();
    let mut HOut32: *mut WOW64_AFD_HANDLE32 = ::core::ptr::null_mut();
    let mut Iosb64: WOW64_IO_STATUS_BLOCK64_M = WOW64_IO_STATUS_BLOCK64_M {
        Status: 0,
        Information: 0,
    };
    let mut P64: *mut WOW64_AFD_POLLHDR64 = ::core::ptr::null_mut();
    let mut H64: *mut WOW64_AFD_HANDLE64 = ::core::ptr::null_mut();
    let mut File: HANDLE =
        *a.offset(0 as isize) as LONG as LONG_PTR as HANDLE;
    let mut Event: HANDLE =
        *a.offset(1 as isize) as LONG as LONG_PTR as HANDLE;
    let mut St: NTSTATUS = 0;
    let mut Count: ULONG = 0;
    let mut i: ULONG = 0;
    let mut Need: ULONG = 0;

    if In32.is_null()
        || (*a.offset(7 as isize) as usize)
            < ::core::mem::size_of::<WOW64_AFD_POLLHDR32>() as usize
    {
        return Wow64pIoctlBadArg(
            Who,
            b"input shorter than AFD_POLL_INFO's header\0" as *const u8 as *const i8,
        );
    }
    Count = (*In32).HandleCount;
    if Count > WOW64_AFD_MAX_HANDLES as ULONG {
        return Wow64pIoctlBadArg(
            Who,
            b"HandleCount above the marshalling cap\0" as *const u8 as *const i8,
        );
    }
    Need = (::core::mem::size_of::<WOW64_AFD_POLLHDR32>() as ULONG).wrapping_add(
        Count.wrapping_mul(::core::mem::size_of::<WOW64_AFD_HANDLE32>() as ULONG),
    );
    if *a.offset(7 as isize) < Need {
        return Wow64pIoctlBadArg(
            Who,
            b"input shorter than HandleCount handles\0" as *const u8 as *const i8,
        );
    }
    if (if *a.offset(8 as isize) != 0 {
        *a.offset(9 as isize)
    } else {
        *a.offset(7 as isize)
    }) < Need
    {
        return Wow64pIoctlBadArg(
            Who,
            b"output shorter than HandleCount handles\0" as *const u8 as *const i8,
        );
    }

    P64 = Wow64pIoctlAlloc(
        (::core::mem::size_of::<WOW64_AFD_POLLHDR64>() as SIZE_T).wrapping_add(
            (Count as SIZE_T).wrapping_mul(
                ::core::mem::size_of::<WOW64_AFD_HANDLE64>() as SIZE_T,
            ),
        ),
    ) as *mut WOW64_AFD_POLLHDR64;
    if P64.is_null() {
        return WOW64_STATUS_NO_MEMORY;
    }
    H64 = P64.offset(1 as isize)
        as *mut WOW64_AFD_HANDLE64;
    H32 = In32.offset(1 as isize)
        as *const WOW64_AFD_HANDLE32;

    (*P64).Timeout = (*In32).Timeout;
    (*P64).HandleCount = Count;
    (*P64).Exclusive = (*In32).Exclusive as ULONGLONG;
    i = 0 as ULONG;
    while i < Count {
        (*H64.offset(i as isize)).Handle =
            (*H32.offset(i as isize)).Handle as ULONG_PTR as ULONGLONG;
        (*H64.offset(i as isize)).Events =
            (*H32.offset(i as isize)).Events;
        (*H64.offset(i as isize)).Status =
            (*H32.offset(i as isize)).Status;
        i = i.wrapping_add(1);
    }

    St = Wow64pIoctlCall(
        Fn,
        File,
        Event,
        core::ptr::addr_of_mut!(Iosb64),
        *a.offset(5 as isize),
        P64 as PVOID,
        (::core::mem::size_of::<WOW64_AFD_POLLHDR64>() as usize).wrapping_add(
            (Count as usize).wrapping_mul(::core::mem::size_of::<WOW64_AFD_HANDLE64>() as usize),
        ) as ULONG,
        P64 as PVOID,
        (::core::mem::size_of::<WOW64_AFD_POLLHDR64>() as usize).wrapping_add(
            (Count as usize).wrapping_mul(::core::mem::size_of::<WOW64_AFD_HANDLE64>() as usize),
        ) as ULONG,
        Who,
    );
    if St >= 0 as NTSTATUS && !Out32.is_null() {
        HOut32 = Out32.offset(1 as isize) as *mut WOW64_AFD_HANDLE32;
        (*Out32).HandleCount = (*P64).HandleCount;
        i = 0 as ULONG;
        while i < Count {
            (*HOut32.offset(i as isize)).Handle =
                (*H64.offset(i as isize)).Handle as ULONG;
            (*HOut32.offset(i as isize)).Events =
                (*H64.offset(i as isize)).Events;
            (*HOut32.offset(i as isize)).Status =
                (*H64.offset(i as isize)).Status;
            i = i.wrapping_add(1);
        }
    }
    Wow64pIoctlFree(P64 as PVOID);
    return Wow64pFinishIosb(
        St,
        File,
        core::ptr::addr_of_mut!(Iosb64),
        *a.offset(4 as isize),
        Who,
    );
}

unsafe extern "C" fn Wow64pIoctlAfdEventSelect(Fn: WOW64_IOCTL_FN, a: *mut ULONG, Who: *const i8) -> NTSTATUS {
    let In32: *const WOW64_AFD_EVSEL32 =
        *a.offset(6 as isize) as ULONG_PTR as *const WOW64_AFD_EVSEL32;
    let mut Iosb64: WOW64_IO_STATUS_BLOCK64_M = WOW64_IO_STATUS_BLOCK64_M {
        Status: 0 as ULONGLONG,
        Information: 0 as ULONGLONG,
    };
    let mut In64: WOW64_AFD_EVSEL64 = WOW64_AFD_EVSEL64 {
        EventObject: 0 as ULONGLONG,
        Events: 0,
        _p: 0,
    };
    let File: HANDLE = *a.offset(0 as isize) as LONG as LONG_PTR as HANDLE;
    let Event: HANDLE = *a.offset(1 as isize) as LONG as LONG_PTR as HANDLE;
    let mut St: NTSTATUS;

    if In32.is_null()
        || (*a.offset(7 as isize) as usize)
            < core::mem::size_of::<WOW64_AFD_EVSEL32>() as usize
    {
        return Wow64pIoctlBadArg(
            Who,
            b"input shorter than AFD_EVENT_SELECT_INFO\0".as_ptr() as *const i8,
        );
    }
    In64.EventObject = (*In32).EventObject as LONG as LONG_PTR as ULONGLONG;
    In64.Events = (*In32).Events;

    St = Wow64pIoctlCall(
        Fn,
        File,
        Event,
        core::ptr::addr_of_mut!(Iosb64),
        *a.offset(5 as isize),
        core::ptr::addr_of_mut!(In64) as PVOID,
        core::mem::size_of::<WOW64_AFD_EVSEL64>() as ULONG,
        *a.offset(8 as isize) as ULONG_PTR as PVOID,
        *a.offset(9 as isize),
        Who,
    );
    return Wow64pFinishIosb(
        St,
        File,
        core::ptr::addr_of_mut!(Iosb64),
        *a.offset(4 as isize),
        Who,
    );
}

unsafe extern "C" fn Wow64pIoctlAfdEnumEvents(Fn: WOW64_IOCTL_FN, a: *mut ULONG, Who: *const i8) -> NTSTATUS {
    let In32: *mut WOW64_AFD_ENUMEV32 =
        *a.add(6) as ULONG_PTR as *mut WOW64_AFD_ENUMEV32;
    let mut Iosb64: WOW64_IO_STATUS_BLOCK64_M = WOW64_IO_STATUS_BLOCK64_M {
        Status: 0,
        Information: 0,
    };
    let mut In64: WOW64_AFD_ENUMEV64 = WOW64_AFD_ENUMEV64 {
        Event: 0,
        PollEvents: 0,
        EventStatus: [0; 10],
        _p: 0,
    };
    let File: HANDLE = *a.add(0) as LONG as LONG_PTR as HANDLE;
    let Event: HANDLE = *a.add(1) as LONG as LONG_PTR as HANDLE;
    let mut St: NTSTATUS;
    let mut i: ULONG;

    if In32.is_null()
        || (*a.add(7) as usize)
            < core::mem::size_of::<WOW64_AFD_ENUMEV32>() as usize
    {
        return Wow64pIoctlBadArg(
            Who,
            b"input shorter than AFD_ENUM_NETWORK_EVENTS_INFO\0" as *const u8 as *const i8,
        );
    }
    (*In32).Event = (*In32).Event;
    In64.Event = (*In32).Event as LONG as LONG_PTR as ULONGLONG;

    St = Wow64pIoctlCall(
        Fn,
        File,
        Event,
        core::ptr::addr_of_mut!(Iosb64),
        *a.add(5),
        core::ptr::addr_of_mut!(In64) as PVOID,
        core::mem::size_of::<WOW64_AFD_ENUMEV64>() as ULONG,
        *a.add(8) as ULONG_PTR as PVOID,
        *a.add(9),
        Who,
    );
    if St >= 0 {
        (*In32).PollEvents = In64.PollEvents;
        i = 0;
        while i < 10 {
            (*In32).EventStatus[i as usize] = In64.EventStatus[i as usize];
            i = i.wrapping_add(1);
        }
    }
    return Wow64pFinishIosb(
        St,
        File,
        core::ptr::addr_of_mut!(Iosb64),
        *a.add(4),
        Who,
    );
}

unsafe extern "C" fn Wow64pIoctlAfdAccept(Fn: WOW64_IOCTL_FN, a: *mut ULONG, Who: *const i8) -> NTSTATUS {
    let In32: *const WOW64_AFD_ACCEPT32 =
        *a.offset(6 as isize) as ULONG_PTR as *const WOW64_AFD_ACCEPT32;
    let mut Iosb64: WOW64_IO_STATUS_BLOCK64_M = WOW64_IO_STATUS_BLOCK64_M {
        Status: 0 as ULONGLONG,
        Information: 0 as ULONGLONG,
    };
    let mut In64: WOW64_AFD_ACCEPT64 = WOW64_AFD_ACCEPT64 {
        UseSAN: 0 as ULONG,
        SequenceNumber: 0 as ULONG,
        ListenHandle: 0 as ULONGLONG,
    };
    let File: HANDLE =
        *a.offset(0 as isize) as LONG as LONG_PTR as HANDLE;
    let Event: HANDLE =
        *a.offset(1 as isize) as LONG as LONG_PTR as HANDLE;
    let mut St: NTSTATUS;

    if In32.is_null()
        || (*a.offset(7 as isize) as usize)
            < core::mem::size_of::<WOW64_AFD_ACCEPT32>()
    {
        return Wow64pIoctlBadArg(
            Who,
            b"input shorter than AFD_ACCEPT_DATA\0".as_ptr() as *const i8,
        );
    }
    In64.UseSAN = (*In32).UseSAN;
    In64.SequenceNumber = (*In32).SequenceNumber;
    In64.ListenHandle = (*In32).ListenHandle as LONG as LONG_PTR as ULONGLONG;

    St = Wow64pIoctlCall(
        Fn,
        File,
        Event,
        core::ptr::addr_of_mut!(Iosb64),
        *a.offset(5 as isize),
        core::ptr::addr_of_mut!(In64) as PVOID,
        core::mem::size_of::<WOW64_AFD_ACCEPT64>() as ULONG,
        *a.offset(8 as isize) as ULONG_PTR as PVOID,
        *a.offset(9 as isize),
        Who,
    );
    return Wow64pFinishIosb(
        St,
        File,
        core::ptr::addr_of_mut!(Iosb64),
        *a.offset(4 as isize),
        Who,
    );
}

unsafe extern "C" fn Wow64pIoctlAfdTdiHandles(Fn: WOW64_IOCTL_FN, a: *mut ULONG, Who: *const i8) -> NTSTATUS {
    let Out32: *mut WOW64_AFD_TDIH32 = *a.offset(8 as isize) as ULONG_PTR as *mut WOW64_AFD_TDIH32;
    let mut Iosb64: WOW64_IO_STATUS_BLOCK64_M = WOW64_IO_STATUS_BLOCK64_M { Status: 0, Information: 0 };
    let mut Out64: WOW64_AFD_TDIH64 = WOW64_AFD_TDIH64 { TdiAddressHandle: 0, TdiConnectionHandle: 0 };
    let File: HANDLE = *a.offset(0 as isize) as LONG as LONG_PTR as HANDLE;
    let Event: HANDLE = *a.offset(1 as isize) as LONG as LONG_PTR as HANDLE;
    let mut St: NTSTATUS;

    if Out32.is_null()
        || (*a.offset(9 as isize) as usize) < core::mem::size_of::<WOW64_AFD_TDIH32>() as usize
    {
        return Wow64pIoctlBadArg(
            Who,
            b"output shorter than AFD_TDI_HANDLE_DATA\0".as_ptr() as *const i8,
        );
    }

    St = Wow64pIoctlCall(
        Fn,
        File,
        Event,
        core::ptr::addr_of_mut!(Iosb64),
        *a.offset(5 as isize),
        *a.offset(6 as isize) as ULONG_PTR as PVOID,
        *a.offset(7 as isize),
        core::ptr::addr_of_mut!(Out64) as PVOID,
        core::mem::size_of::<WOW64_AFD_TDIH64>() as ULONG,
        Who,
    );
    if St >= 0 as NTSTATUS {
        (*Out32).TdiAddressHandle = Out64.TdiAddressHandle as ULONG;
        (*Out32).TdiConnectionHandle = Out64.TdiConnectionHandle as ULONG;
    }
    return Wow64pFinishIosb(
        St,
        File,
        core::ptr::addr_of_mut!(Iosb64),
        *a.offset(4 as isize),
        Who,
    );
}

unsafe extern "C" fn Wow64pIoControl(Fn: WOW64_IOCTL_FN, a: *mut ULONG, Who: *const ::core::ffi::c_char) -> NTSTATUS {
    let mut File: HANDLE = *a.add(0 as usize) as LONG as LONG_PTR as HANDLE;
    let mut Event: HANDLE = *a.add(1 as usize) as LONG as LONG_PTR as HANDLE;
    let mut St: NTSTATUS;
    match *a.add(5 as usize) {
        WOW64_IOCTL_AFD_SEND | WOW64_IOCTL_AFD_RECV => {
            return Wow64pQueueIoApc(
                Wow64pIoctlAfdXfer(Fn, a, Who),
                *a.add(2 as usize),
                *a.add(3 as usize),
                *a.add(4 as usize),
            );
        }
        WOW64_IOCTL_AFD_RECV_DATAGRAM => {
            return Wow64pQueueIoApc(
                Wow64pIoctlAfdRecvFrom(Fn, a, Who),
                *a.add(2 as usize),
                *a.add(3 as usize),
                *a.add(4 as usize),
            );
        }
        WOW64_IOCTL_AFD_SELECT => {
            return Wow64pQueueIoApc(
                Wow64pIoctlAfdSelect(Fn, a, Who),
                *a.add(2 as usize),
                *a.add(3 as usize),
                *a.add(4 as usize),
            );
        }
        WOW64_IOCTL_AFD_EVENT_SELECT => {
            return Wow64pQueueIoApc(
                Wow64pIoctlAfdEventSelect(Fn, a, Who),
                *a.add(2 as usize),
                *a.add(3 as usize),
                *a.add(4 as usize),
            );
        }
        WOW64_IOCTL_AFD_ENUM_NETWORK_EVENTS => {
            return Wow64pQueueIoApc(
                Wow64pIoctlAfdEnumEvents(Fn, a, Who),
                *a.add(2 as usize),
                *a.add(3 as usize),
                *a.add(4 as usize),
            );
        }
        WOW64_IOCTL_AFD_ACCEPT => {
            return Wow64pQueueIoApc(
                Wow64pIoctlAfdAccept(Fn, a, Who),
                *a.add(2 as usize),
                *a.add(3 as usize),
                *a.add(4 as usize),
            );
        }
        WOW64_IOCTL_AFD_GET_TDI_HANDLES => {
            return Wow64pQueueIoApc(
                Wow64pIoctlAfdTdiHandles(Fn, a, Who),
                *a.add(2 as usize),
                *a.add(3 as usize),
                *a.add(4 as usize),
            );
        }
        WOW64_IOCTL_AFD_SEND_DATAGRAM => {
            return Wow64pQueueIoApc(
                Wow64pIoctlAfdSendTo(Fn, a, Who),
                *a.add(2 as usize),
                *a.add(3 as usize),
                *a.add(4 as usize),
            );
        }
        _ => {}
    }

    Wow64pLatchIosb32(*a.add(4 as usize));
    St = (Fn.unwrap_unchecked())(
        File,
        Event,
        if *a.add(2 as usize) != 0 {
            Wow64ApcRoutine as PVOID
        } else {
            core::ptr::null_mut()
        },
        if *a.add(2 as usize) != 0 {
            Wow64pApcContext(
                *a.add(2 as usize),
                *a.add(3 as usize),
            )
        } else {
            *a.add(3 as usize) as ULONG_PTR as PVOID
        },
        *a.add(4 as usize) as ULONG_PTR as PVOID,
        *a.add(5 as usize),
        *a.add(6 as usize) as ULONG_PTR as PVOID,
        *a.add(7 as usize),
        *a.add(8 as usize) as ULONG_PTR as PVOID,
        *a.add(9 as usize),
    );
    Wow64pLatchIosb32(0 as ULONG);
    return St;
}

unsafe extern "C" fn Wow64Manual_NtCreateNamedPipeFile(a: *mut ULONG) -> NTSTATUS {
    let Handle32: *mut ULONG =
        *a.add(0 as usize) as ULONG_PTR as *mut ULONG;
    let mut Oa64: WOW64_OBJECT_ATTRIBUTES64_M = core::mem::zeroed();
    let mut Uname64: WOW64_UNICODE_STRING64_M = core::mem::zeroed();
    let mut Iosb64: WOW64_IO_STATUS_BLOCK64_M = WOW64_IO_STATUS_BLOCK64_M {
        Status: 0,
        Information: 0,
    };
    let mut Handle: HANDLE = core::ptr::null_mut();
    let mut St: NTSTATUS;

    Wow64pMarshalOa(
        core::ptr::addr_of_mut!(Oa64),
        core::ptr::addr_of_mut!(Uname64),
        *a.add(2 as usize),
    );
    St = NtCreateNamedPipeFile(
        core::ptr::addr_of_mut!(Handle),
        *a.add(1 as usize) as ACCESS_MASK,
        (core::ptr::addr_of_mut!(Oa64) as PVOID).cast(),
        (core::ptr::addr_of_mut!(Iosb64) as PVOID).cast(),
        *a.add(4 as usize),
        *a.add(5 as usize),
        *a.add(6 as usize),
        *a.add(7 as usize),
        *a.add(8 as usize),
        *a.add(9 as usize),
        *a.add(10 as usize),
        *a.add(11 as usize),
        *a.add(12 as usize),
        *a.add(13 as usize) as ULONG_PTR as PLARGE_INTEGER,
    );
    if St >= 0 as NTSTATUS && !Handle32.is_null() {
        *Handle32 = Handle as ULONG_PTR as ULONG;
    }
    if St < 0 as NTSTATUS {
        Iosb64.Status = St as ULONG as ULONGLONG;
    }
    Wow64pCopyIosbDown(
        core::ptr::addr_of_mut!(Iosb64),
        *a.add(3 as usize),
    );
    return St;
}

unsafe extern "C" fn Wow64Manual_NtCreateMailslotFile(a: *mut ULONG) -> NTSTATUS {
    let mut Handle32: *mut ULONG =
        *a.offset(0 as isize) as ULONG_PTR as *mut ULONG;
    let mut Oa64: WOW64_OBJECT_ATTRIBUTES64_M = core::mem::zeroed();
    let mut Uname64: WOW64_UNICODE_STRING64_M = core::mem::zeroed();
    let mut Iosb64: WOW64_IO_STATUS_BLOCK64_M = core::mem::zeroed();
    let mut Handle: HANDLE = core::ptr::null_mut();
    let mut St: NTSTATUS = 0;

    Wow64pMarshalOa(
        core::ptr::addr_of_mut!(Oa64),
        core::ptr::addr_of_mut!(Uname64),
        *a.offset(2 as isize),
    );
    St = NtCreateMailslotFile(
        core::ptr::addr_of_mut!(Handle),
        *a.offset(1 as isize) as ACCESS_MASK,
        (core::ptr::addr_of_mut!(Oa64) as PVOID).cast(),
        (core::ptr::addr_of_mut!(Iosb64) as PVOID).cast(),
        *a.offset(4 as isize),
        *a.offset(5 as isize),
        *a.offset(6 as isize),
        *a.offset(7 as isize) as ULONG_PTR as PLARGE_INTEGER,
    );
    if St >= (0 as NTSTATUS) && !Handle32.is_null() {
        *Handle32 = Handle as ULONG_PTR as ULONG;
    }
    if St < (0 as NTSTATUS) {
        Iosb64.Status = St as ULONG as ULONGLONG;
    }
    Wow64pCopyIosbDown(
        core::ptr::addr_of_mut!(Iosb64),
        *a.offset(3 as isize),
    );
    return St;
}

unsafe extern "C" fn Wow64pRefuse(Who: *const i8, Why: *const i8) -> NTSTATUS {
    // DbgPrint("[WOW64] %s is not supported under WoW64 yet: %s — refusing\n", Who, Why);
    WOW64_STATUS_NOT_IMPLEMENTED
}

unsafe extern "C" fn Wow64Manual_NtNotifyChangeDirectoryFile(a: *mut ULONG) -> NTSTATUS {
    let _ = a;
    return Wow64pRefuse(
        b"NtNotifyChangeDirectoryFile\0" as *const u8 as *const i8,
        b"it is inherently async (the IOSB outlives the call), which needs the kernel-side IO_STATUS_BLOCK32 fix\0" as *const u8 as *const i8,
    );
}

unsafe extern "C" fn Wow64Manual_NtNotifyChangeKey(a: *mut ULONG) -> NTSTATUS {
    let _ = a;
    return Wow64pRefuse(
        b"NtNotifyChangeKey\0" as *const u8 as *const i8,
        b"inherently async \xE2\x80\x94 same as NtNotifyChangeDirectoryFile\0" as *const u8 as *const i8,
    );
}

unsafe extern "C" fn Wow64Manual_NtNotifyChangeMultipleKeys(a: *mut ULONG) -> NTSTATUS {
    let _ = a;
    return Wow64pRefuse(
        b"NtNotifyChangeMultipleKeys\0" as *const u8 as *const i8,
        b"inherently async \xE2\x80\x94 same as NtNotifyChangeDirectoryFile\0" as *const u8 as *const i8,
    );
}

unsafe extern "C" fn Wow64Manual_NtSetSystemInformation(a: *mut ULONG) -> NTSTATUS {
    let _ = a;
    return Wow64pRefuse(
        b"NtSetSystemInformation\0" as *const u8 as *const i8,
        b"the info buffer's layout is class-dependent and several classes are pointer-bearing; passing one through unchecked would let a guest corrupt kernel state\0" as *const u8 as *const i8,
    );
}

unsafe extern "C" fn Wow64pMcLParamIsPayload(mut FNID: ULONG) -> ::core::ffi::c_int {
    return (FNID == WOW64_MC_FNID_SENDMESSAGE as ULONG
        || FNID == WOW64_MC_FNID_SENDMESSAGEFF as ULONG
        || FNID == WOW64_MC_FNID_SENDMESSAGEWTOOPTION as ULONG
        || FNID == WOW64_MC_FNID_DEFWINDOWPROC as ULONG
        || FNID == WOW64_MC_FNID_SCROLLBAR as ULONG
        || FNID == WOW64_MC_FNID_DESKTOP as ULONG
        || FNID == WOW64_MC_FNID_MENU as ULONG
        || FNID == WOW64_MC_FNID_MESSAGEWND as ULONG) as ::core::ffi::c_int;
}

unsafe extern "C" fn Wow64pMcGoesThroughMsgMemory(FNID: ULONG) -> ::core::ffi::c_int {
    return (FNID == WOW64_MC_FNID_SENDMESSAGE
        || FNID == WOW64_MC_FNID_SENDMESSAGEFF
        || FNID == WOW64_MC_FNID_SENDMESSAGEWTOOPTION) as ::core::ffi::c_int;
}

unsafe extern "C" fn Wow64pMcPayloadNeedsPointee(Msg: ULONG) -> ::core::ffi::c_int {
    return (Msg == WOW64_MC_WM_NCCALCSIZE
        || Msg == WOW64_MC_WM_COPYDATA
        || Msg == WOW64_MC_WM_CREATE
        || Msg == WOW64_MC_WM_NCCREATE) as ::core::ffi::c_int;
}

unsafe extern "C" fn Wow64Manual_NtUserMessageCall(a: *mut ULONG) -> NTSTATUS {
    // The C's own block-scope prototype.
    extern "system" {
        fn NtUserMessageCall(
            a0: PVOID, a1: ULONG_PTR, a2: PVOID, a3: PVOID, a4: PVOID, a5: ULONG_PTR, a6: ULONG_PTR,
        ) -> NTSTATUS;
    }
    let Msg: ULONG = *a.add(1);
    let lp32: ULONG = *a.add(3);
    let ri32: ULONG = *a.add(4);
    let FNID: ULONG = *a.add(5);
    let mut Cbm: *const WOW64_CB_MARSHAL = core::ptr::null::<WOW64_CB_MARSHAL>();
    let mut Payload64: [u8; WOW64_MC_PAYLOAD64_MAX] = [0; WOW64_MC_PAYLOAD64_MAX];
    let mut Dsm64: WOW64_MC_DSM64 = core::mem::zeroed();
    let mut Result64: ULONG_PTR = 0;
    let mut lParamArg: PVOID = *a.add(3) as LONG as LONG_PTR as PVOID;
    let mut ResultArg: PVOID = *a.add(4) as LONG as LONG_PTR as PVOID;
    let mut St: NTSTATUS;
    let mut i: ULONG;

    /* -- (1) lParam payload: guest 32-bit layout -> host 64-bit ------------ */
    if lp32 != 0 && Wow64pMcLParamIsPayload(FNID) != 0
    {
        Cbm = Wow64CbFindMarshal(Msg);
        if !Cbm.is_null()
        {
            if ((*Cbm).Size64 as usize) > core::mem::size_of::<[u8; WOW64_MC_PAYLOAD64_MAX]>()
            {
                /* Generated table outgrew the buffer — a codegen change, not a
                 * guest input. Loud, never a silent truncation. */
                DbgPrint(
                    b"[WOW64] NtUserMessageCall: msg 0x%lx payload %lu > WOW64_MC_PAYLOAD64_MAX %u \xE2\x80\x94 raise it\n\0"
                        as *const u8 as PCSTR,
                    Msg,
                    (*Cbm).Size64 as core::ffi::c_ulong,
                    core::mem::size_of::<[u8; WOW64_MC_PAYLOAD64_MAX]>() as u32,
                );
                return 0;
            }
            /* Up() writes only the fields it knows; padding it skips must not
             * reach win32k as stack garbage. */
            i = 0;
            while i < (*Cbm).Size64 as ULONG
            {
                Payload64[i as usize] = 0 as u8;
                i = i.wrapping_add(1);
            }
            ((*Cbm).Up.unwrap_unchecked())(
                lp32 as ULONG_PTR as *const u8,
                core::ptr::addr_of_mut!(Payload64) as *mut u8,
            );
            lParamArg = core::ptr::addr_of_mut!(Payload64) as *mut u8 as PVOID;
        }
        else if Wow64pMcPayloadNeedsPointee(Msg) != 0
            && Wow64pMcGoesThroughMsgMemory(FNID) != 0
        {
            /* win32k would size the copy from the host struct and read past the
             * end of the guest's buffer. Failing the call loses the message;
             * letting it through can lose the machine. */
            DbgPrint(
                b"[WOW64] NtUserMessageCall: msg 0x%lx (FNID 0x%lx) has a divergent lParam payload with an embedded pointer and no marshaller \xE2\x80\x94 failing the call rather than over-reading the guest buffer\n\0"
                    as *const u8 as PCSTR,
                Msg,
                FNID,
            );
            return 0;
        }
    }

    /* -- (2)/(3) ResultInfo out-slot --------------------------------------- */
    if ri32 != 0
    {
        if FNID == WOW64_MC_FNID_SENDMESSAGE
        {
            ResultArg = core::ptr::addr_of_mut!(Result64) as PVOID;
        }
        else if FNID == WOW64_MC_FNID_SENDMESSAGEFF ||
                 FNID == WOW64_MC_FNID_SENDMESSAGEWTOOPTION
        {
            let d32: *const WOW64_MC_DSM32 = ri32 as ULONG_PTR as *const WOW64_MC_DSM32;
            Dsm64.uFlags = (*d32).uFlags;
            Dsm64.uTimeout = (*d32).uTimeout;
            Dsm64.Result = (*d32).Result as ULONGLONG;
            ResultArg = core::ptr::addr_of_mut!(Dsm64) as PVOID;
        }
    }

    St = NtUserMessageCall(
        *a.add(0) as LONG as LONG_PTR as PVOID,
        *a.add(1) as LONG as LONG_PTR as ULONG_PTR,
        *a.add(2) as LONG as LONG_PTR as PVOID,
        lParamArg,
        ResultArg,
        *a.add(5) as LONG as LONG_PTR as ULONG_PTR,
        *a.add(6) as LONG as LONG_PTR as ULONG_PTR,
    );

    /* -- copy back: host 64-bit -> guest 32-bit ---------------------------- */
    if !Cbm.is_null()
    {
        ((*Cbm).Down.unwrap_unchecked())(
            core::ptr::addr_of_mut!(Payload64) as *mut u8,
            lp32 as ULONG_PTR as *mut u8,
        );
    }

    if ri32 != 0
    {
        if FNID == WOW64_MC_FNID_SENDMESSAGE
        {
            *(ri32 as ULONG_PTR as *mut ULONG) = Result64 as ULONG;
        }
        else if FNID == WOW64_MC_FNID_SENDMESSAGEFF ||
                 FNID == WOW64_MC_FNID_SENDMESSAGEWTOOPTION
        {
            let d32_0: *mut WOW64_MC_DSM32 = ri32 as ULONG_PTR as *mut WOW64_MC_DSM32;
            (*d32_0).uFlags = Dsm64.uFlags;
            (*d32_0).uTimeout = Dsm64.uTimeout;
            (*d32_0).Result = Dsm64.Result as ULONG;
        }
    }

    if Wow64TraceSyscalls != 0
        && ResultArg != *a.add(4) as LONG as LONG_PTR as PVOID
    {
        DbgPrint(
            b"[WOW64] NtUserMessageCall msg=0x%lx fnid=0x%lx result-slot %s\n\0"
                as *const u8 as PCSTR,
            Msg,
            FNID,
            if FNID == WOW64_MC_FNID_SENDMESSAGE
            {
                b"ULONG_PTR 8->4\0" as *const u8 as *const core::ffi::c_char
            }
            else
            {
                b"DOSENDMESSAGE 16->12\0" as *const u8 as *const core::ffi::c_char
            },
        );
    }
    if Wow64TraceSyscalls != 0 && !Cbm.is_null()
    {
        DbgPrint(
            b"[WOW64] NtUserMessageCall msg=0x%lx fnid=0x%lx payload 32<->64 (%u<->%u bytes)\n\0"
                as *const u8 as PCSTR,
            Msg,
            FNID,
            (*Cbm).Size32 as u32,
            (*Cbm).Size64 as u32,
        );
    }
    return St;
}

unsafe extern "C" fn Wow64Manual_NtCreateIoCompletion(a: *mut ULONG) -> NTSTATUS {
    let mut Handle32: *mut ULONG = *a.offset(0 as isize) as ULONG_PTR as *mut ULONG;
    let mut Access: ACCESS_MASK = *a.offset(1 as isize) as ACCESS_MASK;
    let mut Oa64: WOW64_OBJECT_ATTRIBUTES64_M = WOW64_OBJECT_ATTRIBUTES64_M {
        Length: 0,
        __pad0: 0,
        RootDirectory: 0,
        ObjectName: 0,
        Attributes: 0,
        __pad1: 0,
        SecurityDescriptor: 0,
        SecurityQualityOfService: 0,
    };
    let mut Uname64: WOW64_UNICODE_STRING64_M = WOW64_UNICODE_STRING64_M {
        Length: 0,
        MaximumLength: 0,
        __pad: 0,
        Buffer: 0,
    };
    let mut Oa64Ptr: PVOID = core::ptr::null_mut();
    let mut Handle: HANDLE = core::ptr::null_mut();
    let mut St: NTSTATUS;

    if *a.offset(2 as isize) != 0 {
        Wow64pMarshalOa(
            &raw mut Oa64,
            &raw mut Uname64,
            *a.offset(2 as isize),
        );
        Oa64Ptr = &raw mut Oa64 as PVOID;
    }
    St = NtCreateIoCompletion(
        &raw mut Handle,
        Access,
        (Oa64Ptr).cast(),
        *a.offset(3 as isize),
    );
    if St >= 0 && !Handle32.is_null() {
        *Handle32 = Handle as ULONG_PTR as ULONG;
    }
    return St;
}

unsafe extern "C" fn Wow64Manual_NtQueryInformationThread(a: *mut ULONG) -> NTSTATUS {
    let Thread: HANDLE = (*a.offset(0 as isize)) as LONG as LONG_PTR as HANDLE;
    let Class: ULONG = *a.offset(1 as isize);
    let Buf32: PVOID = (*a.offset(2 as isize)) as ULONG_PTR as PVOID;
    let Len32: ULONG = *a.offset(3 as isize);
    let RetLen32: *mut ULONG = (*a.offset(4 as isize)) as ULONG_PTR as *mut ULONG;

    match Class {
        0 => {
            let mut T64: WOW64_TBI64 = core::mem::zeroed();
            let mut T32: *mut WOW64_TBI32 = Buf32 as *mut WOW64_TBI32;
            let mut Ret64: ULONG = 0;
            let mut St: NTSTATUS = 0;
            if (Len32 as usize) < core::mem::size_of::<WOW64_TBI32>() {
                if !RetLen32.is_null() {
                    *RetLen32 = core::mem::size_of::<WOW64_TBI32>() as ULONG;
                }
                return 0xC0000004u32 as NTSTATUS; /* INFO_LENGTH_MISMATCH */
            }
            St = NtQueryInformationThread(
                Thread,
                Class,
                core::ptr::addr_of_mut!(T64) as PVOID,
                core::mem::size_of_val(&T64) as ULONG,
                core::ptr::addr_of_mut!(Ret64),
            );
            if St < 0 {
                return St;
            }
            (*T32).ExitStatus = T64.ExitStatus;
            /* TebBaseAddress must be the 32-bit TEB. Same-process threads:
             * TEB32 = TEB64 + WowTebOffset (the kernel MmCreateTeb slot we
             * already use in Wow64LdrpInitialize). Cross-process handles
             * would need NtReadVirtualMemory; report 0 rather than a 64-bit
             * TEB the guest would misread. */
            (*T32).TebBaseAddress = 0;
            /* TEB64.ClientId.UniqueProcess is at +0x40 (TEB incomplete here) */
            if T64.UniqueProcess
                == *((NtCurrentTeb() as PBYTE).offset(0x40 as isize) as *const ULONGLONG)
                && T64.TebBaseAddress != 0
            {
                let mut WowOff: LONG = *((T64.TebBaseAddress as ULONG_PTR as PBYTE)
                    .offset(WOW64_TEBSLOT_WOWTEBOFFSET as isize) as *const LONG);
                if WowOff != 0 {
                    (*T32).TebBaseAddress =
                        (T64.TebBaseAddress.wrapping_add(WowOff as ULONGLONG)) as ULONG;
                }
            }
            (*T32).UniqueProcess = (T64.UniqueProcess) as ULONG;
            (*T32).UniqueThread = (T64.UniqueThread) as ULONG;
            (*T32).AffinityMask = (T64.AffinityMask) as ULONG;
            (*T32).Priority = T64.Priority;
            (*T32).BasePriority = T64.BasePriority;
            if !RetLen32.is_null() {
                *RetLen32 = core::mem::size_of::<WOW64_TBI32>() as ULONG;
            }
            return St;
        }

        9 => {
            let mut Addr64: ULONGLONG = 0;
            let mut Ret64: ULONG = 0;
            let mut St: NTSTATUS = 0;
            if (Len32 as usize) < core::mem::size_of::<ULONG>() {
                if !RetLen32.is_null() {
                    *RetLen32 = core::mem::size_of::<ULONG>() as ULONG;
                }
                return 0xC0000004u32 as NTSTATUS;
            }
            St = NtQueryInformationThread(
                Thread,
                Class,
                core::ptr::addr_of_mut!(Addr64) as PVOID,
                core::mem::size_of_val(&Addr64) as ULONG,
                core::ptr::addr_of_mut!(Ret64),
            );
            if St < 0 {
                return St;
            }
            *(Buf32 as *mut ULONG) = Addr64 as ULONG;
            if !RetLen32.is_null() {
                *RetLen32 = core::mem::size_of::<ULONG>() as ULONG;
            }
            return St;
        }

        /* Width-identical info classes: LARGE_INTEGERs / ULONGs only. */
        1 | 11 | 12 | 14 | 16 => {
            /* 1: ThreadTimes, 11: ThreadPerformanceCount, 12: ThreadAmILastThread,
             * 14: ThreadPriorityBoost, 16: ThreadIsIoPending */
            return NtQueryInformationThread(Thread, Class, Buf32, Len32, RetLen32 as PULONG);
        }

        _ => {
            /* Fail loud: an unmarshalled class passed through silently would
             * hand the guest 64-bit-laid-out data. */
            DbgPrint(
                b"[WOW64] NtQueryInformationThread class %lu NEEDS MANUAL MARSHALLER \xE2\x80\x94 failing loud\n\0"
                    .as_ptr() as PCSTR,
                Class,
            );
            return 0xC0000003u32 as NTSTATUS; /* INVALID_INFO_CLASS */
        }
    }
}

unsafe extern "C" fn Wow64Manual_NtRemoveIoCompletion(a: *mut ULONG) -> NTSTATUS {
    let Port: HANDLE = *a.offset(0 as isize) as LONG as LONG_PTR as HANDLE;
    let Key32: *mut ULONG = *a.offset(1 as isize) as ULONG_PTR as *mut ULONG;
    let Ctx32: *mut ULONG = *a.offset(2 as isize) as ULONG_PTR as *mut ULONG;
    let mut Key64: PVOID = core::ptr::null_mut();
    let mut Ctx64: PVOID = core::ptr::null_mut();
    let mut Iosb64: WOW64_IO_STATUS_BLOCK64_M = WOW64_IO_STATUS_BLOCK64_M {
        Status: 0,
        Information: 0,
    };
    let mut St: NTSTATUS;

    /* Timeout is a LARGE_INTEGER* — width-identical, guest pointer passes. */
    St = NtRemoveIoCompletion(
        Port,
        core::ptr::addr_of_mut!(Key64),
        core::ptr::addr_of_mut!(Ctx64),
        (core::ptr::addr_of_mut!(Iosb64) as PVOID).cast(),
        *a.offset(4 as isize) as ULONG_PTR as PLARGE_INTEGER,
    );
    if St == 0 {
        if !Key32.is_null() {
            *Key32 = Key64 as ULONG_PTR as ULONG;
        }
        if !Ctx32.is_null() {
            *Ctx32 = Ctx64 as ULONG_PTR as ULONG;
        }
        Wow64pCopyIosbDown(
            core::ptr::addr_of_mut!(Iosb64),
            *a.offset(3 as isize),
        );
    }
    return St;
}

unsafe extern "C" fn Wow64Manual_NtRemoveIoCompletionEx(a: *mut ULONG) -> NTSTATUS {
    let mut Port: HANDLE = *a.offset(0 as isize) as LONG as LONG_PTR as HANDLE;
    let mut Info32: *mut WOW64_FILE_IO_COMPLETION_INFORMATION32_M =
        *a.offset(1 as isize) as ULONG_PTR as *mut WOW64_FILE_IO_COMPLETION_INFORMATION32_M;
    let mut Count: ULONG = *a.offset(2 as isize);
    let mut Removed32: *mut ULONG = *a.offset(3 as isize) as ULONG_PTR as *mut ULONG;
    let mut Info64: [WOW64_FILE_IO_COMPLETION_INFORMATION64_M; 64] =
        core::mem::zeroed();
    let mut Removed: ULONG = 0 as ULONG;
    let mut i: ULONG;
    let mut St: NTSTATUS;

    if Count > 64 as ULONG {
        Count = 64 as ULONG;
    }
    /* Timeout is a LARGE_INTEGER* -- width-identical, guest pointer passes. */
    St = NtRemoveIoCompletionEx(
        Port,
        (core::ptr::addr_of_mut!(Info64) as *mut WOW64_FILE_IO_COMPLETION_INFORMATION64_M as PVOID).cast(),
        Count,
        core::ptr::addr_of_mut!(Removed),
        *a.offset(4 as isize) as ULONG_PTR as PLARGE_INTEGER,
        *a.offset(5 as isize) as BOOLEAN,
    );
    if St == 0 as NTSTATUS {
        i = 0 as ULONG;
        while i < Removed {
            (*Info32.offset(i as isize)).KeyContext = Info64[i as usize].KeyContext as ULONG;
            (*Info32.offset(i as isize)).ApcContext = Info64[i as usize].ApcContext as ULONG;
            (*Info32.offset(i as isize)).IoStatusBlock.Status =
                Info64[i as usize].IoStatusBlock.Status as ULONG;
            (*Info32.offset(i as isize)).IoStatusBlock.Information =
                Info64[i as usize].IoStatusBlock.Information as ULONG;
            i = i.wrapping_add(1);
        }
        if !Removed32.is_null() {
            *Removed32 = Removed;
        }
    }
    return St;
}

unsafe extern "C" fn Wow64Manual_NtUserEnumDisplayMonitors(a: *mut ULONG) -> NTSTATUS {
    let mut Dc: HANDLE = *a.add(0 as usize) as LONG as LONG_PTR as HANDLE;
    let mut Rect: PVOID = *a.add(1 as usize) as ULONG_PTR as PVOID;
    let mut List32: *mut ULONG = *a.add(2 as usize) as ULONG_PTR as *mut ULONG;
    let mut RectList: PVOID = *a.add(3 as usize) as ULONG_PTR as PVOID;
    let mut Want: ULONG = *a.add(4 as usize);
    let mut List64: [ULONGLONG; WOW64_MON_MAX as usize] = [0; WOW64_MON_MAX as usize];
    let mut Got: INT = 0;
    let mut i: ULONG = 0;

    if List32.is_null() || Want == 0 {
        return NtUserEnumDisplayMonitors((Dc).cast(), ((Rect) as *const tagRECT), core::ptr::null_mut(), (RectList).cast(), 0 as ULONG) as NTSTATUS;
    }
    if Want > WOW64_MON_MAX as ULONG {
        DbgPrint(
            b"[WOW64] NtUserEnumDisplayMonitors: %lu monitors > cap %u \xE2\x80\x94 failing loud\n\0".as_ptr() as PCSTR,
            Want,
            WOW64_MON_MAX as ULONG,
        );
        return 0xC0000095 as ULONG as NTSTATUS;
    }
    Got = NtUserEnumDisplayMonitors(
        (Dc).cast(),
        ((Rect) as *const tagRECT),
        (core::ptr::addr_of_mut!(List64) as *mut ULONGLONG as PVOID).cast(),
        (RectList).cast(),
        Want,
    );
    i = 0 as ULONG;
    while Got > 0 as INT && i < Want && i < Got as ULONG {
        *List32.offset(i as isize) = List64[i as usize] as ULONG;
        i = i.wrapping_add(1);
    }
    return Got as NTSTATUS;
}

unsafe extern "C" fn Wow64Manual_NtUserThunkedMenuInfo(a: *mut ULONG) -> NTSTATUS {
    let _ = core::mem::size_of::<MI32>();
    let _ = core::mem::size_of::<MI64>();
    let M32: *const MI32 =
        *a.offset(1 as isize) as ULONG_PTR as *const MI32;
    let mut M64: MI64 = core::mem::zeroed();

    if M32.is_null()
        || (*M32).cbSize as usize
            != core::mem::size_of::<MI32>() as usize
    {
        return NtUserThunkedMenuInfo(
            (*a.offset(0 as isize) as ULONG_PTR as PVOID).cast(),
            ((*a.offset(1 as isize) as ULONG_PTR as PVOID) as *const tagMENUINFO),
        ) as NTSTATUS;
    }

    M64.cbSize = core::mem::size_of_val(&M64) as ULONG;
    M64.fMask = (*M32).fMask;
    M64.dwStyle = (*M32).dwStyle;
    M64.cyMax = (*M32).cyMax;
    M64.hbrBack = (*M32).hbrBack as ULONGLONG;
    M64.dwContextHelpID = (*M32).dwContextHelpID;
    M64.__pad = 0 as ULONG;
    M64.dwMenuData = (*M32).dwMenuData as ULONGLONG;
    return NtUserThunkedMenuInfo(
        (*a.offset(0 as isize) as ULONG_PTR as PVOID).cast(),
        ((core::ptr::addr_of_mut!(M64) as PVOID) as *const tagMENUINFO),
    ) as NTSTATUS;
}

unsafe extern "C" fn Wow64pGet32(p: *const u8) -> u32 {
    let mut v: u32 = 0;
    let mut i: u32 = 0;
    while i < 4 {
        *(core::ptr::addr_of_mut!(v) as *mut u8).offset(i as isize) = *p.offset(i as isize);
        i = i.wrapping_add(1);
    }
    return v;
}

unsafe extern "C" fn Wow64pPut64(p: *mut u8, v: ULONGLONG) {
    let mut i: u32 = 0;
    while i < 8 {
        *p.offset(i as isize) = *(core::ptr::addr_of!(v) as *const u8).offset(i as isize);
        i = i.wrapping_add(1);
    }
}

unsafe extern "C" fn Wow64pInput32To64(
    mut g: *const ::core::ffi::c_uchar,
    mut h: *mut ::core::ffi::c_uchar,
) {
    let mut i: ::core::ffi::c_uint = 0;
    i = 0 as ::core::ffi::c_uint;
    while i < WOW64_INPUT64_CB as ::core::ffi::c_uint {
        *h.offset(i as isize) = 0 as ::core::ffi::c_uchar;
        i = i.wrapping_add(1);
    }
    i = 0 as ::core::ffi::c_uint;
    while i < 4 as ::core::ffi::c_uint {
        *h.offset(i as isize) = *g.offset(i as isize);
        i = i.wrapping_add(1);
    }

    match Wow64pGet32(g) {
        WOW64_INPUT_MOUSE => {
            i = 0 as ::core::ffi::c_uint;
            while i < 20 as ::core::ffi::c_uint {
                *h.offset((8 as ::core::ffi::c_uint).wrapping_add(i) as isize) =
                    *g.offset((4 as ::core::ffi::c_uint).wrapping_add(i) as isize);
                i = i.wrapping_add(1);
            }
            Wow64pPut64(
                h.offset(8 as isize)
                    .offset(24 as isize),
                Wow64pGet32(
                    g.offset(4 as isize)
                        .offset(20 as isize),
                ) as ULONGLONG,
            );
        }
        WOW64_INPUT_KEYBOARD => {
            i = 0 as ::core::ffi::c_uint;
            while i < 12 as ::core::ffi::c_uint {
                *h.offset((8 as ::core::ffi::c_uint).wrapping_add(i) as isize) =
                    *g.offset((4 as ::core::ffi::c_uint).wrapping_add(i) as isize);
                i = i.wrapping_add(1);
            }
            Wow64pPut64(
                h.offset(8 as isize)
                    .offset(16 as isize),
                Wow64pGet32(
                    g.offset(4 as isize)
                        .offset(12 as isize),
                ) as ULONGLONG,
            );
        }
        _ => {
            i = 0 as ::core::ffi::c_uint;
            while i < (WOW64_INPUT32_CB - 4) as ::core::ffi::c_uint {
                *h.offset((8 as ::core::ffi::c_uint).wrapping_add(i) as isize) =
                    *g.offset((4 as ::core::ffi::c_uint).wrapping_add(i) as isize);
                i = i.wrapping_add(1);
            }
        }
    }
}

unsafe extern "C" fn Wow64Manual_NtUserSendInput(a: *mut ULONG) -> NTSTATUS {
    // The C's own block-scope prototype.
    extern "system" {
        fn NtUserSendInput(nInputs: ULONG, pInput: PVOID, cbSize: INT) -> ULONG;
    }
    let nInputs: ULONG = *a.offset(0 as isize);
    let In32: *const u8 = *a.offset(1 as isize) as ULONG_PTR as *const u8;
    let cbGuest: ULONG = *a.offset(2 as isize);
    let mut Stack: [u8; (WOW64_SENDINPUT_STACK as usize) * (WOW64_INPUT64_CB as usize)] = [0u8; (WOW64_SENDINPUT_STACK as usize) * (WOW64_INPUT64_CB as usize)];
    let mut In64: *mut u8 = &raw mut Stack as *mut u8;
    let mut Heap: PVOID = core::ptr::null_mut();
    let mut Ret: ULONG = 0;
    let mut i: ULONG = 0;

    /* Pass a wrong cbSize straight through so the kernel still rejects it:
     * only the guest's own sizeof means "I am a 32-bit INPUT". */
    if In32.is_null() || nInputs == 0 || cbGuest != WOW64_INPUT32_CB as ULONG {
        return NtUserSendInput(
            nInputs,
            (*a.offset(1 as isize) as ULONG_PTR as PVOID).cast(),
            cbGuest as INT,
        ) as NTSTATUS;
    }

    if nInputs > WOW64_SENDINPUT_STACK as ULONG {
        let mut Size: SIZE_T = (nInputs as SIZE_T).wrapping_mul(WOW64_INPUT64_CB as SIZE_T);
        let mut St: NTSTATUS = NtAllocateVirtualMemory(
            (-(1 as LONG_PTR)) as HANDLE,
            &raw mut Heap,
            0 as ULONG_PTR,
            &raw mut Size,
            (MEM_RESERVE | MEM_COMMIT) as ULONG,
            PAGE_READWRITE as ULONG,
        );
        if St < 0 as NTSTATUS {
            return St;
        }
        In64 = Heap as *mut u8;
    }

    i = 0 as ULONG;
    while i < nInputs {
        Wow64pInput32To64(
            In32.offset((i as SIZE_T).wrapping_mul(WOW64_INPUT32_CB as SIZE_T) as isize),
            In64.offset((i as SIZE_T).wrapping_mul(WOW64_INPUT64_CB as SIZE_T) as isize),
        );
        i = i.wrapping_add(1);
    }

    /* IN-only: win32k copies each entry into its own SafeInput and never
     * writes back through pInput, so there is no copy-down. */
    Ret = NtUserSendInput(nInputs, In64 as PVOID, WOW64_INPUT64_CB as INT);

    if !Heap.is_null() {
        let mut Free: SIZE_T = 0 as SIZE_T;
        NtFreeVirtualMemory(
            (-(1 as LONG_PTR)) as HANDLE,
            &raw mut Heap,
            &raw mut Free,
            MEM_RELEASE as ULONG,
        );
    }
    return Ret as NTSTATUS;
}

unsafe extern "C" fn Wow64Manual_NtUserInitializeClientPfnArrays(a: *mut ULONG) -> NTSTATUS {
    static mut Announced: ::core::ffi::c_int = 0;
    Wow64pPfnMapSetGuest(
        *a.offset(0 as isize),
        *a.offset(1 as isize),
    );
    if Announced == 0 {
        Announced = 1 as ::core::ffi::c_int;
        DbgPrint(b"[WOW64] NtUserInitializeClientPfnArrays: INTERCEPTED - guest 32-bit PFNCLIENT arrays are never installed into the GLOBAL gpsi callback table (they would be read at host width and corrupt it for native processes). Guest callbacks go through the WoW64 callback bridge.\n\0".as_ptr() as *const i8);
    }
    return 0 as NTSTATUS;
}

unsafe extern "C" fn Wow64Manual_NtOpenProcessTokenEx(a: *mut ULONG) -> NTSTATUS {
    let mut Handle32: *mut ULONG =
        (*a.offset(3 as isize)) as ULONG_PTR as *mut ULONG;
    let mut Handle: HANDLE = core::ptr::null_mut();
    let mut St: NTSTATUS;

    St = NtOpenProcessTokenEx(
        ((*a.offset(0 as isize)) as LONG) as LONG_PTR as HANDLE,
        (*a.offset(1 as isize)) as ACCESS_MASK,
        *a.offset(2 as isize),
        core::ptr::addr_of_mut!(Handle),
    );
    if St >= 0 && !Handle32.is_null() {
        *Handle32 = Handle as ULONG_PTR as ULONG;
    }
    return St;
}

unsafe extern "C" fn Wow64Manual_NtOpenThreadTokenEx(a: *mut ULONG) -> NTSTATUS {
    let Handle32: *mut ULONG = *a.add(4 as usize) as ULONG_PTR as *mut ULONG;
    let mut Handle: HANDLE = core::ptr::null_mut();
    let mut St: NTSTATUS;
    St = NtOpenThreadTokenEx(
        *a.add(0 as usize) as LONG as LONG_PTR as HANDLE,
        *a.add(1 as usize) as ACCESS_MASK,
        (*a.add(2 as usize) != 0) as BOOLEAN,
        *a.add(3 as usize),
        core::ptr::addr_of_mut!(Handle),
    );
    if St >= 0 as NTSTATUS && !Handle32.is_null() {
        *Handle32 = Handle as ULONG_PTR as ULONG;
    }
    return St;
}

unsafe extern "C" fn Wow64pCtx32Valid(Addr: ULONG) -> core::ffi::c_int {
    return (Addr >= 0x1000 as ULONG
        && Addr < 0x80000000 as ULONG
        && (Addr & 3 as ULONG) == 0 as ULONG
        && (0x80000000 as ULONG).wrapping_sub(Addr) >= WOW64_CTX32_SIZE as ULONG)
        as core::ffi::c_int;
}

unsafe extern "C" fn Wow64pCpuGetContext32(Ctx32: PVOID) -> LONG {
    let mut St: LONG;

    if Wow64pBTCpuGetContext.is_none() {
        return 0xC0000002u32 as LONG;
    }
    St = (Wow64pBTCpuGetContext.unwrap_unchecked())(
        (-(2 as LONG_PTR)) as HANDLE,
        WOW64_PROCESS_CURRENT as HANDLE,
        NULL,
        Ctx32,
    );
    #[cfg(not(target_arch = "aarch64"))] // `#ifndef _M_ARM64` in the C
    if St >= 0 {
        let p = (Ctx32 as PBYTE).add(WOW64_CTX32_ESP as usize) as *mut ULONG;
        core::ptr::write_volatile(p, core::ptr::read_volatile(p).wrapping_add(4));
    }
    return St;
}

unsafe extern "C" fn Wow64pIsCurrentThread(mut Handle32: ULONG) -> ::core::ffi::c_int {
    let mut Self_0: WOW64_THREAD_BASIC_INFO = ::core::mem::zeroed();
    let mut Target: WOW64_THREAD_BASIC_INFO = ::core::mem::zeroed();
    let mut Thread: HANDLE = Handle32 as LONG as LONG_PTR as HANDLE;
    if Thread == -(2 as ::core::ffi::c_int) as LONG_PTR as HANDLE {
        return 1 as ::core::ffi::c_int;
    }
    if NtQueryInformationThread(
        -(2 as ::core::ffi::c_int) as LONG_PTR as HANDLE,
        0 as ULONG,
        &raw mut Self_0 as PVOID,
        ::core::mem::size_of::<WOW64_THREAD_BASIC_INFO>() as ULONG,
        ::core::ptr::null_mut::<ULONG>(),
    ) < 0 as NTSTATUS
        || NtQueryInformationThread(
            Thread,
            0 as ULONG,
            &raw mut Target as PVOID,
            ::core::mem::size_of::<WOW64_THREAD_BASIC_INFO>() as ULONG,
            ::core::ptr::null_mut::<ULONG>(),
        ) < 0 as NTSTATUS
    {
        return 0 as ::core::ffi::c_int;
    }
    return (Self_0.TebBaseAddress == Target.TebBaseAddress) as ::core::ffi::c_int;
}

unsafe extern "C" fn Wow64Manual_NtGetContextThread(a: *mut ULONG) -> NTSTATUS {
    let Ctx32: ULONG = *a.offset(1 as isize);
    let mut Want: ULONG = 0;
    let mut St: LONG = 0;

    if Wow64pCtx32Valid(Ctx32) == 0 {
        return 0xC000000Du32 as NTSTATUS; /* STATUS_INVALID_PARAMETER */
    }
    if Wow64pIsCurrentThread(*a.offset(0 as isize)) == 0 {
        DbgPrint(
            b"[WOW64] NtGetContextThread: REFUSED - cross-thread query (handle %08lx). The CPU layer only exposes the CALLING thread's guest state (task #64).\n\0"
                .as_ptr() as PCSTR,
            *a.offset(0 as isize),
        );
        return 0xC0000002u32 as NTSTATUS; /* STATUS_NOT_IMPLEMENTED */
    }

    Want = core::ptr::read_volatile(Ctx32 as ULONG_PTR as *const ULONG);
    St = Wow64pCpuGetContext32(Ctx32 as ULONG_PTR as PVOID);
    if St < (0 as LONG) {
        return St as NTSTATUS;
    }
    #[cfg(not(target_arch = "aarch64"))] // `#ifndef _M_ARM64` in the C
    if (Want & (WOW64_CTX32_F_UNTRACKED as ULONG)) != 0 {
        DbgPrint(
            b"[WOW64] NtGetContextThread: ContextFlags %08lx asked for FP/Dr/extended state wow64cpu does not maintain; clearing those bits in the reply, fields left untouched.\n\0"
                .as_ptr() as PCSTR,
            Want,
        );
        Want &= !(WOW64_CTX32_F_UNTRACKED as ULONG);
    }
    core::ptr::write_volatile(Ctx32 as ULONG_PTR as *mut ULONG, Want);
    return 0 as NTSTATUS;
}

unsafe extern "C" fn Wow64Manual_NtSetContextThread(a: *mut ULONG) -> NTSTATUS {
    let mut Ctx32: ULONG = *a.add(1 as usize);
    let mut Src: *const BYTE;
    let mut Merged: [BYTE; (WOW64_CTX32_SIZE) as _] = [0; (WOW64_CTX32_SIZE) as _];
    let mut Want: ULONG;
    let mut i: ULONG;
    let mut St: LONG;

    if Wow64pCtx32Valid(Ctx32) == 0 {
        return 0xC000000Du32 as NTSTATUS;
    }
    if Wow64pBTCpuSetContext.is_none() {
        return 0xC0000002u32 as NTSTATUS;
    }
    if Wow64pIsCurrentThread(*a.add(0 as usize)) == 0 {
        DbgPrint(
            b"[WOW64] NtSetContextThread: REFUSED - cross-thread set (handle %08lx). Installing it on the CALLING thread instead would corrupt the wrong thread (task #64).\n\0"
                .as_ptr() as PCSTR,
            *a.add(0 as usize),
        );
        return 0xC0000002u32 as NTSTATUS;
    }

    Src = Ctx32 as ULONG_PTR as *const BYTE;
    Want = core::ptr::read_volatile(Src as *const ULONG);

    /* A partial ContextFlags must leave the unrequested groups ALONE, but
     * BTCpuSetContext installs the whole SegGs..SegSs window unconditionally
     * and flags it as the state to RESUME. So start from the live context and
     * overlay only the groups the guest actually asked for. */
    i = 0 as ULONG;
    while i < WOW64_CTX32_SIZE as ULONG {
        Merged[i as usize] = 0 as BYTE;
        i = i.wrapping_add(1);
    }
    core::ptr::write_unaligned(
        core::ptr::addr_of_mut!(Merged) as *mut BYTE as *mut ULONG,
        Want,
    );
    St = Wow64pCpuGetContext32(core::ptr::addr_of_mut!(Merged) as *mut BYTE as PVOID);
    if St < 0 as LONG {
        return St as NTSTATUS;
    }

    if WOW64_CTX32_WANTS!(Want, WOW64_CTX32_F_SEGMENTS) {
        i = WOW64_CTX32_SEG_LO as ULONG;
        while i <= WOW64_CTX32_SEG_HI as ULONG {
            core::ptr::write_unaligned(
                (core::ptr::addr_of_mut!(Merged) as *mut BYTE).add(i as usize) as *mut ULONG,
                core::ptr::read_unaligned(Src.add(i as usize) as *const ULONG),
            );
            i = i.wrapping_add(4 as ULONG);
        }
    }
    if WOW64_CTX32_WANTS!(Want, WOW64_CTX32_F_INTEGER) {
        i = WOW64_CTX32_INT_LO as ULONG;
        while i <= WOW64_CTX32_INT_HI as ULONG {
            core::ptr::write_unaligned(
                (core::ptr::addr_of_mut!(Merged) as *mut BYTE).add(i as usize) as *mut ULONG,
                core::ptr::read_unaligned(Src.add(i as usize) as *const ULONG),
            );
            i = i.wrapping_add(4 as ULONG);
        }
    } else {
        /* Resuming a replaced context bypasses Wow64CpuDispatch's
         * `Eax = result` fixup, so the return value has to be planted by hand.
         * If the guest asked to set INTEGER state it chose its own Eax, and
         * that wins — same as a real self-directed SetThreadContext. */
        core::ptr::write_unaligned(
            (core::ptr::addr_of_mut!(Merged) as *mut BYTE).add(WOW64_CTX32_EAX as usize)
                as *mut ULONG,
            0 as ULONG,
        ); /* STATUS_SUCCESS */
    }
    if WOW64_CTX32_WANTS!(Want, WOW64_CTX32_F_CONTROL) {
        i = WOW64_CTX32_CTL_LO as ULONG;
        while i <= WOW64_CTX32_CTL_HI as ULONG {
            core::ptr::write_unaligned(
                (core::ptr::addr_of_mut!(Merged) as *mut BYTE).add(i as usize) as *mut ULONG,
                core::ptr::read_unaligned(Src.add(i as usize) as *const ULONG),
            );
            i = i.wrapping_add(4 as ULONG);
        }
    }
#[cfg(not(target_arch = "aarch64"))]
    if (Want & WOW64_CTX32_F_UNTRACKED as ULONG) != 0 {
        DbgPrint(
            b"[WOW64] NtSetContextThread: ContextFlags %08lx asked to set FP/Dr/extended state wow64cpu does not maintain; those groups were IGNORED (control/integer/segments applied).\n\0"
                .as_ptr() as PCSTR,
            Want,
        );
    }

    return (Wow64pBTCpuSetContext.unwrap_unchecked())(
        (-(2 as LONG_PTR)) as HANDLE,
        WOW64_PROCESS_CURRENT as HANDLE,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(Merged) as *mut BYTE as PVOID,
    ) as NTSTATUS;
}

unsafe extern "C" fn Wow64Manual_NtWaitForMultipleObjects(a: *mut ULONG) -> NTSTATUS {
    let Count: ULONG = *a.add(0 as usize);
    let Handles32: *const ULONG = *a.add(1 as usize) as ULONG_PTR as *const ULONG;
    let mut Handles64: [HANDLE; (WOW64_MAXIMUM_WAIT_OBJECTS as _)] = [core::ptr::null_mut(); (WOW64_MAXIMUM_WAIT_OBJECTS as _)];
    let mut i: ULONG;

    if Count == 0 as ULONG || Count > WOW64_MAXIMUM_WAIT_OBJECTS as ULONG {
        return 0xC00000EF as ULONG as NTSTATUS;
    }
    if Handles32.is_null() {
        return 0xC00000F0 as ULONG as NTSTATUS;
    }

    i = 0 as ULONG;
    while i < Count {
        Handles64[i as usize] = *Handles32.add(i as usize) as LONG as LONG_PTR as HANDLE;
        i = i.wrapping_add(1);
    }

    return NtWaitForMultipleObjects(
        Count,
        core::ptr::addr_of_mut!(Handles64) as *mut HANDLE,
        *a.add(2 as usize),
        (*a.add(3 as usize) & 0xFF as ULONG) as BOOLEAN,
        *a.add(4 as usize) as ULONG_PTR as PLARGE_INTEGER,
    );
}

unsafe extern "C" fn Wow64pPublishPeb64Params(
    Process: HANDLE,
    Peb32: ULONG_PTR,
    Peb64: ULONGLONG,
) -> NTSTATUS {
    let mut Hdr32: [u8; (WOW64_UPP32_HDR as _)] = [0; (WOW64_UPP32_HDR as _)];
    let mut Blk64: [u8; (WOW64_UPP64_SIZE) as _] = [0; (WOW64_UPP64_SIZE) as _];
    let mut Upp32: ULONG = 0 as ULONG;
    let mut StrBase: ULONG_PTR;
    let mut Remote: PVOID = core::ptr::null_mut();
    let mut Size: SIZE_T = WOW64_UPP64_SIZE as SIZE_T;
    let mut Done: SIZE_T = 0 as SIZE_T;
    let mut Remote64: ULONGLONG;
    let mut St: NTSTATUS;
    let mut i: u32;

    /* Where the guest put its own block. */
    St = NtReadVirtualMemory(
        Process,
        (Peb32.wrapping_add(WOW64_PEB32_PARAMS_OFF as ULONG_PTR)) as PVOID,
        &raw mut Upp32 as PVOID,
        core::mem::size_of::<ULONG>() as SIZE_T,
        &raw mut Done,
    );
    if St < 0 as NTSTATUS
        || Done != core::mem::size_of::<ULONG>() as SIZE_T
        || Upp32 == 0
    {
        DbgPrint(
            b"[WOW64] PEB64 params: child PEB32->ProcessParameters is empty (st=%08lx) -- nothing to widen\n\0"
                as *const u8 as PCSTR,
            St,
        );
        return 0xC000000D as ULONG as NTSTATUS; /* STATUS_INVALID_PARAMETER */
    }

    St = NtReadVirtualMemory(
        Process,
        (Upp32 as ULONG_PTR) as PVOID,
        &raw mut Hdr32 as *mut u8 as PVOID,
        core::mem::size_of_val(&Hdr32) as SIZE_T,
        &raw mut Done,
    );
    if St < 0 as NTSTATUS
        || Done != core::mem::size_of_val(&Hdr32) as SIZE_T
    {
        return if St < 0 as NTSTATUS {
            St
        } else {
            0xC0000018 as ULONG as NTSTATUS /* CONFLICTING_ADDRESSES */
        };
    }

    /* A denormalized Buffer is an offset from the block base; a normalized one
     * is already absolute. Resolve to absolute either way, once. */
    StrBase = if (*((&raw mut Hdr32 as *mut u8).offset(0x08 as isize) as *mut ULONG)
        & WOW64_PPF_NORMALIZED as ULONG)
        != 0
    {
        0 as ULONG_PTR
    } else {
        Upp32 as ULONG_PTR
    };

    i = 0 as u32;
    while (i as usize) < core::mem::size_of_val(&Blk64) {
        Blk64[i as usize] = 0 as u8;
        i = i.wrapping_add(1);
    }

    /* UPP_U32(0xc, 0xc) -> DebugFlags */
    *((&raw mut Blk64 as *mut u8).offset(0x0c as isize) as *mut ULONG) =
        *((&raw mut Hdr32 as *mut u8).offset(0x0c as isize) as *mut ULONG);

    /* The block describes itself as fully used: the strings it points at live
     * outside it, so there is no arena tail to account for. */
    *((&raw mut Blk64 as *mut u8).offset(0x00 as isize) as *mut ULONG) =
        WOW64_UPP64_SIZE as ULONG; /* MaximumLength */
    *((&raw mut Blk64 as *mut u8).offset(0x04 as isize) as *mut ULONG) =
        WOW64_UPP64_SIZE as ULONG; /* Length        */
    *((&raw mut Blk64 as *mut u8).offset(0x08 as isize) as *mut ULONG) =
        *((&raw mut Hdr32 as *mut u8).offset(0x08 as isize) as *mut ULONG)
            | WOW64_PPF_NORMALIZED as ULONG;

    /* UPP_U32(FACT_0023, FACT_0024) -> DebugFlags (expanded above with the header fields) */
    /* UPP_PTR -> ConsoleHandle */
    *((&raw mut Blk64 as *mut u8).offset(0x10 as isize) as *mut ULONGLONG) =
        *((&raw mut Hdr32 as *mut u8).offset(0x10 as isize) as *mut ULONG)
            as ULONGLONG;
    /* UPP_U32 -> ConsoleFlags */
    *((&raw mut Blk64 as *mut u8).offset(0x18 as isize) as *mut ULONG) =
        *((&raw mut Hdr32 as *mut u8).offset(0x14 as isize) as *mut ULONG);
    /* UPP_PTR -> StandardInput */
    *((&raw mut Blk64 as *mut u8).offset(0x20 as isize) as *mut ULONGLONG) =
        *((&raw mut Hdr32 as *mut u8).offset(0x18 as isize) as *mut ULONG)
            as ULONGLONG;
    /* UPP_PTR -> StandardOutput */
    *((&raw mut Blk64 as *mut u8).offset(0x28 as isize) as *mut ULONGLONG) =
        *((&raw mut Hdr32 as *mut u8).offset(0x1c as isize) as *mut ULONG)
            as ULONGLONG;
    /* UPP_PTR -> StandardError */
    *((&raw mut Blk64 as *mut u8).offset(0x30 as isize) as *mut ULONGLONG) =
        *((&raw mut Hdr32 as *mut u8).offset(0x20 as isize) as *mut ULONG)
            as ULONGLONG;
    /* UPP_STR -> CurDir.DosPath */
    {
        let _b: ULONG =
            *((&raw mut Hdr32 as *mut u8).offset((0x24 as isize) + 4) as *mut ULONG);
        *((&raw mut Blk64 as *mut u8).offset((0x38 as isize) + 0) as *mut USHORT) =
            *((&raw mut Hdr32 as *mut u8).offset((0x24 as isize) + 0) as *mut USHORT);
        *((&raw mut Blk64 as *mut u8).offset((0x38 as isize) + 2) as *mut USHORT) =
            *((&raw mut Hdr32 as *mut u8).offset((0x24 as isize) + 2) as *mut USHORT);
        *((&raw mut Blk64 as *mut u8).offset((0x38 as isize) + 8) as *mut ULONGLONG) =
            if _b != 0 {
                (_b as ULONG_PTR).wrapping_add(StrBase) as ULONGLONG
            } else {
                0u64
            };
    }
    /* UPP_PTR -> CurDir.Handle */
    *((&raw mut Blk64 as *mut u8).offset(0x48 as isize) as *mut ULONGLONG) =
        *((&raw mut Hdr32 as *mut u8).offset(0x2c as isize) as *mut ULONG)
            as ULONGLONG;
    /* UPP_STR -> DllPath */
    {
        let _b: ULONG =
            *((&raw mut Hdr32 as *mut u8).offset((0x30 as isize) + 4) as *mut ULONG);
        *((&raw mut Blk64 as *mut u8).offset((0x50 as isize) + 0) as *mut USHORT) =
            *((&raw mut Hdr32 as *mut u8).offset((0x30 as isize) + 0) as *mut USHORT);
        *((&raw mut Blk64 as *mut u8).offset((0x50 as isize) + 2) as *mut USHORT) =
            *((&raw mut Hdr32 as *mut u8).offset((0x30 as isize) + 2) as *mut USHORT);
        *((&raw mut Blk64 as *mut u8).offset((0x50 as isize) + 8) as *mut ULONGLONG) =
            if _b != 0 {
                (_b as ULONG_PTR).wrapping_add(StrBase) as ULONGLONG
            } else {
                0u64
            };
    }
    /* UPP_STR -> ImagePathName */
    {
        let _b: ULONG =
            *((&raw mut Hdr32 as *mut u8).offset((0x38 as isize) + 4) as *mut ULONG);
        *((&raw mut Blk64 as *mut u8).offset((0x60 as isize) + 0) as *mut USHORT) =
            *((&raw mut Hdr32 as *mut u8).offset((0x38 as isize) + 0) as *mut USHORT);
        *((&raw mut Blk64 as *mut u8).offset((0x60 as isize) + 2) as *mut USHORT) =
            *((&raw mut Hdr32 as *mut u8).offset((0x38 as isize) + 2) as *mut USHORT);
        *((&raw mut Blk64 as *mut u8).offset((0x60 as isize) + 8) as *mut ULONGLONG) =
            if _b != 0 {
                (_b as ULONG_PTR).wrapping_add(StrBase) as ULONGLONG
            } else {
                0u64
            };
    }
    /* UPP_STR -> CommandLine */
    {
        let _b: ULONG =
            *((&raw mut Hdr32 as *mut u8).offset((0x40 as isize) + 4) as *mut ULONG);
        *((&raw mut Blk64 as *mut u8).offset((0x70 as isize) + 0) as *mut USHORT) =
            *((&raw mut Hdr32 as *mut u8).offset((0x40 as isize) + 0) as *mut USHORT);
        *((&raw mut Blk64 as *mut u8).offset((0x70 as isize) + 2) as *mut USHORT) =
            *((&raw mut Hdr32 as *mut u8).offset((0x40 as isize) + 2) as *mut USHORT);
        *((&raw mut Blk64 as *mut u8).offset((0x70 as isize) + 8) as *mut ULONGLONG) =
            if _b != 0 {
                (_b as ULONG_PTR).wrapping_add(StrBase) as ULONGLONG
            } else {
                0u64
            };
    }
    /* UPP_PTR -> Environment */
    *((&raw mut Blk64 as *mut u8).offset(0x80 as isize) as *mut ULONGLONG) =
        *((&raw mut Hdr32 as *mut u8).offset(0x48 as isize) as *mut ULONG)
            as ULONGLONG;
    /* dwX..wShowWindow */
    i = 0 as u32;
    while i < 9 as u32 {
        *((&raw mut Blk64 as *mut u8).offset(
            (0x88 as u32).wrapping_add(i.wrapping_mul(4 as u32))
                as isize,
        ) as *mut ULONG) = *((&raw mut Hdr32 as *mut u8).offset(
            (0x4c as u32).wrapping_add(i.wrapping_mul(4 as u32))
                as isize,
        ) as *mut ULONG);
        i = i.wrapping_add(1);
    }
    /* UPP_STR -> WindowTitle */
    {
        let _b: ULONG =
            *((&raw mut Hdr32 as *mut u8).offset((0x70 as isize) + 4) as *mut ULONG);
        *((&raw mut Blk64 as *mut u8).offset((0xb0 as isize) + 0) as *mut USHORT) =
            *((&raw mut Hdr32 as *mut u8).offset((0x70 as isize) + 0) as *mut USHORT);
        *((&raw mut Blk64 as *mut u8).offset((0xb0 as isize) + 2) as *mut USHORT) =
            *((&raw mut Hdr32 as *mut u8).offset((0x70 as isize) + 2) as *mut USHORT);
        *((&raw mut Blk64 as *mut u8).offset((0xb0 as isize) + 8) as *mut ULONGLONG) =
            if _b != 0 {
                (_b as ULONG_PTR).wrapping_add(StrBase) as ULONGLONG
            } else {
                0u64
            };
    }
    /* UPP_STR -> DesktopInfo */
    {
        let _b: ULONG =
            *((&raw mut Hdr32 as *mut u8).offset((0x78 as isize) + 4) as *mut ULONG);
        *((&raw mut Blk64 as *mut u8).offset((0xc0 as isize) + 0) as *mut USHORT) =
            *((&raw mut Hdr32 as *mut u8).offset((0x78 as isize) + 0) as *mut USHORT);
        *((&raw mut Blk64 as *mut u8).offset((0xc0 as isize) + 2) as *mut USHORT) =
            *((&raw mut Hdr32 as *mut u8).offset((0x78 as isize) + 2) as *mut USHORT);
        *((&raw mut Blk64 as *mut u8).offset((0xc0 as isize) + 8) as *mut ULONGLONG) =
            if _b != 0 {
                (_b as ULONG_PTR).wrapping_add(StrBase) as ULONGLONG
            } else {
                0u64
            };
    }
    /* UPP_STR -> ShellInfo */
    {
        let _b: ULONG =
            *((&raw mut Hdr32 as *mut u8).offset((0x80 as isize) + 4) as *mut ULONG);
        *((&raw mut Blk64 as *mut u8).offset((0xd0 as isize) + 0) as *mut USHORT) =
            *((&raw mut Hdr32 as *mut u8).offset((0x80 as isize) + 0) as *mut USHORT);
        *((&raw mut Blk64 as *mut u8).offset((0xd0 as isize) + 2) as *mut USHORT) =
            *((&raw mut Hdr32 as *mut u8).offset((0x80 as isize) + 2) as *mut USHORT);
        *((&raw mut Blk64 as *mut u8).offset((0xd0 as isize) + 8) as *mut ULONGLONG) =
            if _b != 0 {
                (_b as ULONG_PTR).wrapping_add(StrBase) as ULONGLONG
            } else {
                0u64
            };
    }
    /* UPP_STR -> RuntimeData */
    {
        let _b: ULONG =
            *((&raw mut Hdr32 as *mut u8).offset((0x88 as isize) + 4) as *mut ULONG);
        *((&raw mut Blk64 as *mut u8).offset((0xe0 as isize) + 0) as *mut USHORT) =
            *((&raw mut Hdr32 as *mut u8).offset((0x88 as isize) + 0) as *mut USHORT);
        *((&raw mut Blk64 as *mut u8).offset((0xe0 as isize) + 2) as *mut USHORT) =
            *((&raw mut Hdr32 as *mut u8).offset((0x88 as isize) + 2) as *mut USHORT);
        *((&raw mut Blk64 as *mut u8).offset((0xe0 as isize) + 8) as *mut ULONGLONG) =
            if _b != 0 {
                (_b as ULONG_PTR).wrapping_add(StrBase) as ULONGLONG
            } else {
                0u64
            };
    }

    St = NtAllocateVirtualMemory(
        Process,
        &raw mut Remote,
        0 as ULONG_PTR,
        &raw mut Size,
        (MEM_COMMIT | MEM_RESERVE) as ULONG,
        PAGE_READWRITE as ULONG,
    );
    if St < 0 as NTSTATUS {
        DbgPrint(
            b"[WOW64] PEB64 params: allocate in child failed %08lx\n\0" as *const u8 as PCSTR,
            St,
        );
        return St;
    }

    St = NtWriteVirtualMemory(
        Process,
        Remote,
        &raw mut Blk64 as *mut u8 as PVOID,
        core::mem::size_of_val(&Blk64) as SIZE_T,
        &raw mut Done,
    );
    if St < 0 as NTSTATUS
        || Done != core::mem::size_of_val(&Blk64) as SIZE_T
    {
        return if St < 0 as NTSTATUS {
            St
        } else {
            0x8000000D as ULONG as NTSTATUS /* STATUS_PARTIAL_COPY */
        };
    }

    Remote64 = Remote as ULONG_PTR as ULONGLONG;
    St = NtWriteVirtualMemory(
        Process,
        (Peb64.wrapping_add(WOW64_PEB64_PARAMS_OFF as ULONGLONG)) as ULONG_PTR as PVOID,
        &raw mut Remote64 as PVOID,
        core::mem::size_of::<ULONGLONG>() as SIZE_T,
        &raw mut Done,
    );
    if St < 0 as NTSTATUS
        || Done != core::mem::size_of::<ULONGLONG>() as SIZE_T
    {
        return if St < 0 as NTSTATUS {
            St
        } else {
            0x8000000D as ULONG as NTSTATUS
        };
    }

    DbgPrint(
        b"[WOW64] PEB64->ProcessParameters published at %p (widened from PEB32 block %08lx, strings %s)\n\0"
            as *const u8 as PCSTR,
        Remote,
        Upp32,
        if StrBase != 0 {
            b"denormalized\0" as *const u8 as *const ::core::ffi::c_char
        } else {
            b"normalized\0" as *const u8 as *const ::core::ffi::c_char
        },
    );
    return 0 as NTSTATUS;
}

unsafe extern "C" fn Wow64pCreateForeignThread(
    Process: HANDLE,
    CreateSuspended: BOOLEAN,
    Ctx32: PBYTE,
    ITeb32: PBYTE,
    Handle32: *mut ULONG,
    ClientId32: *mut ULONG,
) -> NTSTATUS {
    let mut Pbi: WOW64_PROCESS_BASIC_INFORMATION64 = core::mem::zeroed();
    let mut TargetPeb32: ULONG_PTR = 0 as ULONG_PTR;
    let mut Ldr: ULONGLONG = (1 as ULONGLONG).wrapping_neg();
    let mut Read: SIZE_T = 0 as SIZE_T;
    let mut Thread: HANDLE = core::ptr::null_mut();
    let mut Cid: WOW64_CLIENT_ID = WOW64_CLIENT_ID {
        UniqueProcess: core::ptr::null_mut(),
        UniqueThread: core::ptr::null_mut(),
    };
    let mut St: NTSTATUS;

    St = NtQueryInformationProcess(
        Process,
        ProcessWow64Information as ULONG,
        core::ptr::addr_of_mut!(TargetPeb32) as PVOID,
        ::core::mem::size_of::<ULONG_PTR>() as ULONG,
        ::core::ptr::null_mut::<ULONG>(),
    );
    if St < 0 || TargetPeb32 == 0 {
        DbgPrint(
            b"[WOW64] NtCreateThread: REFUSED - cross-process into a non-WoW64 target (ProcessHandle=%p st=%08lx) (task #64).\n\0"
                as *const u8 as PCSTR,
            Process,
            St,
        );
        return 0xC0000002 as u32 as NTSTATUS; /* STATUS_NOT_IMPLEMENTED */
    }

    St = NtQueryInformationProcess(
        Process,
        ProcessBasicInformation as ULONG,
        core::ptr::addr_of_mut!(Pbi) as PVOID,
        ::core::mem::size_of::<WOW64_PROCESS_BASIC_INFORMATION64>() as ULONG,
        ::core::ptr::null_mut::<ULONG>(),
    );
    if St >= 0 && Pbi.PebBaseAddress != 0 {
        /* PEB64.Ldr is at +0x18 on every 64-bit host we build for. */
        St = NtReadVirtualMemory(
            Process,
            (Pbi.PebBaseAddress.wrapping_add(0x18 as ULONGLONG) as ULONG_PTR)
                as PVOID,
            core::ptr::addr_of_mut!(Ldr) as PVOID,
            ::core::mem::size_of::<ULONGLONG>() as SIZE_T,
            core::ptr::addr_of_mut!(Read),
        );
        if St < 0
            || Read != ::core::mem::size_of::<ULONGLONG>() as SIZE_T
        {
            Ldr = (1 as ULONGLONG).wrapping_neg(); /* unreadable: treat as "already running" */
        }
    }

    if Ldr != 0 as ULONGLONG {
        /* A RUNNING target: this is CreateRemoteThread; see
         * Wow64CreateForeignGuestThread. */
        if Ctx32.is_null() || ITeb32.is_null() {
            return 0xC000000D as u32 as NTSTATUS; /* STATUS_INVALID_PARAMETER */
        }
        St = Wow64CreateForeignGuestThread(
            Process,
            Pbi.PebBaseAddress,
            *(Ctx32.offset(WOW64_CTX32_EIP as isize) as *mut ULONG),
            *(Ctx32.offset(WOW64_CTX32_ESP as isize) as *mut ULONG),
            *(Ctx32.offset(WOW64_CTX32_EAX as isize) as *mut ULONG),
            *(Ctx32.offset(WOW64_CTX32_EBX as isize) as *mut ULONG),
            *(ITeb32.offset(WOW64_ITEB32_STACKBASE as isize) as *mut ULONG),
            *(ITeb32.offset(WOW64_ITEB32_STACKLIMIT as isize) as *mut ULONG),
            *(ITeb32.offset(WOW64_ITEB32_ALLOCBASE as isize) as *mut ULONG),
            CreateSuspended,
            core::ptr::addr_of_mut!(Thread),
            (core::ptr::addr_of_mut!(Cid)).cast(),
        ) as NTSTATUS;
        if St < 0 {
            return St;
        }
        if !Handle32.is_null() {
            *Handle32 = Thread as ULONG_PTR as ULONG;
        }
        if !ClientId32.is_null() {
            *ClientId32.offset(0 as isize) =
                Cid.UniqueProcess as ULONG_PTR as ULONG;
            *ClientId32.offset(1 as isize) =
                Cid.UniqueThread as ULONG_PTR as ULONG;
        }
        return St;
    }

    /* The child's 64-bit loader runs before its wow64.dll and reads PEB64's
     * process parameters. Widen the block the guest wrote into PEB32 and point
     * PEB64 at it, BEFORE the thread exists. */
    St = Wow64pPublishPeb64Params(Process, TargetPeb32, Pbi.PebBaseAddress);
    if St < 0 {
        DbgPrint(
            b"[WOW64] NtCreateThread: PEB64 params not published %08lx -- refusing rather than starting a child that cannot load\n\0"
                as *const u8 as PCSTR,
            St,
        );
        return St;
    }

    /* StartAddress is never reached; RtlExitUserThread is named so that if
     * that assumption is ever wrong the thread exits instead of running
     * whatever is at the address. */
    St = RtlCreateUserThread(
        Process,
        core::ptr::null_mut(),
        CreateSuspended,
        0 as ULONG,
        0 as SIZE_T,
        0 as SIZE_T,
        core::mem::transmute::<*mut c_void, _>((RtlExitUserThread as usize) as PVOID),
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(Thread),
        (core::ptr::addr_of_mut!(Cid) as PVOID).cast(),
    ) as NTSTATUS;
    if St < 0 {
        DbgPrint(
            b"[WOW64] NtCreateThread: cross-process initial thread failed %08lx\n\0"
                as *const u8 as PCSTR,
            St,
        );
        return St;
    }

    if !Handle32.is_null() {
        *Handle32 = Thread as ULONG_PTR as ULONG;
    }
    if !ClientId32.is_null() {
        *ClientId32.offset(0 as isize) =
            Cid.UniqueProcess as ULONG_PTR as ULONG;
        *ClientId32.offset(1 as isize) =
            Cid.UniqueThread as ULONG_PTR as ULONG;
    }
    DbgPrint(
        b"[WOW64] NtCreateThread: cross-process INITIAL thread pid=%lx tid=%lx suspended=%u (task #64 initial-thread half)\n\0"
            as *const u8 as PCSTR,
        Cid.UniqueProcess as ULONG_PTR as ULONG,
        Cid.UniqueThread as ULONG_PTR as ULONG,
        CreateSuspended as ::core::ffi::c_int,
    );
    return St;
}

unsafe extern "C" fn Wow64Manual_NtTerminateThread(a: *mut ULONG) -> NTSTATUS {
    let mut Thread: HANDLE =
        *a.offset(0 as isize) as LONG as LONG_PTR as HANDLE;
    let mut St: NTSTATUS = 0;
    if !Thread.is_null() && Thread != (-(2 as LONG_PTR)) as HANDLE {
        return NtTerminateThread(
            Thread,
            *a.offset(1 as isize) as NTSTATUS,
        );
    }
    Wow64FreeThreadCpuArea();
    St = NtTerminateThread(
        Thread,
        *a.offset(1 as isize) as NTSTATUS,
    );
    DbgPrint(
        b"[WOW64] NtTerminateThread: self-terminate returned %08lx - forcing thread exit (CPU area already released)\n\0"
            .as_ptr() as *const u8 as PCSTR,
        St,
    );
    RtlExitUserThread(St);
    return St;
}

unsafe extern "C" fn Wow64Manual_NtCreateThread(a: *mut ULONG) -> NTSTATUS {
    let mut Handle32: *mut ULONG = *a.offset(0 as isize) as ULONG_PTR as *mut ULONG;
    let mut ClientId32: *mut ULONG = *a.offset(4 as isize) as ULONG_PTR as *mut ULONG;
    let mut Ctx32: PBYTE = *a.offset(5 as isize) as ULONG_PTR as PBYTE;
    let mut ITeb32: PBYTE = *a.offset(6 as isize) as ULONG_PTR as PBYTE;
    let mut Thread: HANDLE = core::ptr::null_mut();
    let mut Cid: WOW64_CLIENT_ID = WOW64_CLIENT_ID {
        UniqueProcess: core::ptr::null_mut(),
        UniqueThread: core::ptr::null_mut(),
    };
    let mut St: NTSTATUS = 0;

    if (*a.offset(3 as isize) as LONG as LONG_PTR as HANDLE)
        != (-(1 as LONG_PTR)) as HANDLE
    {
        return Wow64pCreateForeignThread(
            (*a.offset(3 as isize) as LONG as LONG_PTR) as HANDLE,
            ((*a.offset(7 as isize)) & (0xFF as ULONG)) as BOOLEAN,
            Ctx32,
            ITeb32,
            Handle32,
            ClientId32,
        );
    }

    if Ctx32.is_null() || ITeb32.is_null() {
        return (0xC000000D as ::core::ffi::c_uint) as NTSTATUS;
    }

    St = Wow64CreateGuestThread(
        *(Ctx32.offset(WOW64_CTX32_EIP as isize) as *mut ULONG),
        *(Ctx32.offset(WOW64_CTX32_ESP as isize) as *mut ULONG),
        *(Ctx32.offset(WOW64_CTX32_EAX as isize) as *mut ULONG),
        *(Ctx32.offset(WOW64_CTX32_EBX as isize) as *mut ULONG),
        *(ITeb32.offset(WOW64_ITEB32_STACKBASE as isize) as *mut ULONG),
        *(ITeb32.offset(WOW64_ITEB32_STACKLIMIT as isize) as *mut ULONG),
        *(ITeb32.offset(WOW64_ITEB32_ALLOCBASE as isize) as *mut ULONG),
        ((*a.offset(7 as isize)) & (0xFF as ULONG)) as BOOLEAN,
        core::ptr::addr_of_mut!(Thread),
        (core::ptr::addr_of_mut!(Cid)).cast(),
    ) as NTSTATUS;

    if St < (0 as NTSTATUS) {
        return St;
    }

    if !Handle32.is_null() {
        *Handle32 = Thread as ULONG_PTR as ULONG;
    }
    if !ClientId32.is_null() {
        *ClientId32.offset(0 as isize) = Cid.UniqueProcess as ULONG_PTR as ULONG;
        *ClientId32.offset(1 as isize) = Cid.UniqueThread as ULONG_PTR as ULONG;
    }

    DbgPrint(
        b"[WOW64] NtCreateThread: guest thread tid=%lx eip=%08lx esp=%08lx routine=%08lx suspended=%lu\n\0"
            .as_ptr() as PCSTR,
        Cid.UniqueThread as ULONG_PTR as ULONG,
        *(Ctx32.offset(WOW64_CTX32_EIP as isize) as *mut ULONG),
        *(Ctx32.offset(WOW64_CTX32_ESP as isize) as *mut ULONG),
        *(Ctx32.offset(WOW64_CTX32_EAX as isize) as *mut ULONG),
        (*a.offset(7 as isize)) & (0xFF as ULONG),
    );
    return St;
}
#[no_mangle]
pub unsafe extern "C" fn Wow64ManualDispatch(Name: *const ::core::ffi::c_char, Arguments: *mut ULONG) -> NTSTATUS {
    /* tiny hand strcmp to avoid a CRT dependency */
    if Wow64pStrEq(Name, b"NtQueryVirtualMemory\0" as *const u8 as *const ::core::ffi::c_char) != 0 {
        return Wow64Manual_NtQueryVirtualMemory(Arguments);
    }
    if Wow64pStrEq(Name, b"NtQueryInformationProcess\0" as *const u8 as *const ::core::ffi::c_char) != 0 {
        return Wow64Manual_NtQueryInformationProcess(Arguments);
    }
    if Wow64pStrEq(Name, b"NtQuerySystemInformation\0" as *const u8 as *const ::core::ffi::c_char) != 0 {
        return Wow64Manual_NtQuerySystemInformation(Arguments);
    }
    if Wow64pStrEq(Name, b"NtOpenKey\0" as *const u8 as *const ::core::ffi::c_char) != 0 {
        return Wow64pOpenByOa(Some(core::mem::transmute::<*const (), _>(NtOpenKey as *const ())), Arguments);
    }
    if Wow64pStrEq(Name, b"NtOpenDirectoryObject\0" as *const u8 as *const ::core::ffi::c_char) != 0 {
        return Wow64pOpenByOa(Some(core::mem::transmute::<*const (), _>(NtOpenDirectoryObject as *const ())), Arguments);
    }
    if Wow64pStrEq(Name, b"NtOpenSymbolicLinkObject\0" as *const u8 as *const ::core::ffi::c_char) != 0 {
        return Wow64pOpenByOa(Some(core::mem::transmute::<*const (), _>(NtOpenSymbolicLinkObject as *const ())), Arguments);
    }
    if Wow64pStrEq(Name, b"NtOpenSection\0" as *const u8 as *const ::core::ffi::c_char) != 0 {
        return Wow64pOpenByOa(Some(core::mem::transmute::<*const (), _>(NtOpenSection as *const ())), Arguments);
    }

    /* The out-cell class — every one of these used to hand the kernel a 4-byte
     * guest cell for a pointer-sized store. Driven off `marshalgen audit`. */
    if Wow64pStrEq(Name, b"NtOpenProcessToken\0" as *const u8 as *const ::core::ffi::c_char) != 0 {
        return Wow64Manual_NtOpenProcessToken(Arguments);
    }
    if Wow64pStrEq(Name, b"NtOpenThreadToken\0" as *const u8 as *const ::core::ffi::c_char) != 0 {
        return Wow64Manual_NtOpenThreadToken(Arguments);
    }
    if Wow64pStrEq(Name, b"NtDuplicateObject\0" as *const u8 as *const ::core::ffi::c_char) != 0 {
        return Wow64Manual_NtDuplicateObject(Arguments);
    }
    if Wow64pStrEq(Name, b"NtGetNextProcess\0" as *const u8 as *const ::core::ffi::c_char) != 0 {
        return Wow64Manual_NtGetNextProcess(Arguments);
    }
    if Wow64pStrEq(Name, b"NtGetNextThread\0" as *const u8 as *const ::core::ffi::c_char) != 0 {
        return Wow64Manual_NtGetNextThread(Arguments);
    }
    if Wow64pStrEq(Name, b"NtDuplicateToken\0" as *const u8 as *const ::core::ffi::c_char) != 0 {
        return Wow64Manual_NtDuplicateToken(Arguments);
    }
    if Wow64pStrEq(Name, b"NtCreateEventPair\0" as *const u8 as *const ::core::ffi::c_char) != 0 {
        return Wow64pOpenByOa(Some(core::mem::transmute::<*const (), _>(NtCreateEventPair as *const ())), Arguments);
    }
    if Wow64pStrEq(Name, b"NtOpenEventPair\0" as *const u8 as *const ::core::ffi::c_char) != 0 {
        return Wow64pOpenByOa(Some(core::mem::transmute::<*const (), _>(NtOpenEventPair as *const ())), Arguments);
    }
    if Wow64pStrEq(Name, b"NtCreateJobObject\0" as *const u8 as *const ::core::ffi::c_char) != 0 {
        return Wow64pOpenByOa(Some(core::mem::transmute::<*const (), _>(NtCreateJobObject as *const ())), Arguments);
    }
    if Wow64pStrEq(Name, b"NtOpenJobObject\0" as *const u8 as *const ::core::ffi::c_char) != 0 {
        return Wow64pOpenByOa(Some(core::mem::transmute::<*const (), _>(NtOpenJobObject as *const ())), Arguments);
    }
    if Wow64pStrEq(Name, b"NtOpenTimer\0" as *const u8 as *const ::core::ffi::c_char) != 0 {
        return Wow64pOpenByOa(Some(core::mem::transmute::<*const (), _>(NtOpenTimer as *const ())), Arguments);
    }
    if Wow64pStrEq(Name, b"NtCreateTimer\0" as *const u8 as *const ::core::ffi::c_char) != 0 {
        return Wow64pOpenByOaU32(Some(core::mem::transmute::<*const (), _>(NtCreateTimer as *const ())), Arguments);
    }
    if Wow64pStrEq(Name, b"NtCreateDebugObject\0" as *const u8 as *const ::core::ffi::c_char) != 0 {
        return Wow64pOpenByOaU32(Some(core::mem::transmute::<*const (), _>(NtCreateDebugObject as *const ())), Arguments);
    }
    if Wow64pStrEq(Name, b"NtCreatePort\0" as *const u8 as *const ::core::ffi::c_char) != 0 {
        return Wow64pCreatePort(Some(core::mem::transmute::<*const (), _>(NtCreatePort as *const ())), Arguments);
    }
    if Wow64pStrEq(Name, b"NtCreateWaitablePort\0" as *const u8 as *const ::core::ffi::c_char) != 0 {
        return Wow64pCreatePort(Some(core::mem::transmute::<*const (), _>(NtCreateWaitablePort as *const ())), Arguments);
    }
    if Wow64pStrEq(Name, b"NtCreateProcess\0" as *const u8 as *const ::core::ffi::c_char) != 0 {
        return Wow64Manual_NtCreateProcess(Arguments);
    }
    if Wow64pStrEq(Name, b"NtCreateProcessEx\0" as *const u8 as *const ::core::ffi::c_char) != 0 {
        return Wow64Manual_NtCreateProcessEx(Arguments);
    }
    if Wow64pStrEq(Name, b"NtCreateProfile\0" as *const u8 as *const ::core::ffi::c_char) != 0 {
        return Wow64Manual_NtCreateProfile(Arguments);
    }
    if Wow64pStrEq(Name, b"NtLockVirtualMemory\0" as *const u8 as *const ::core::ffi::c_char) != 0 {
        return Wow64pLockUnlockVm(Some(NtLockVirtualMemory), Arguments);
    }
    if Wow64pStrEq(Name, b"NtUnlockVirtualMemory\0" as *const u8 as *const ::core::ffi::c_char) != 0 {
        return Wow64pLockUnlockVm(Some(NtUnlockVirtualMemory), Arguments);
    }
    if Wow64pStrEq(Name, b"NtFlushVirtualMemory\0" as *const u8 as *const ::core::ffi::c_char) != 0 {
        return Wow64Manual_NtFlushVirtualMemory(Arguments);
    }

    /* Bare PUNICODE_STRING / POBJECT_ATTRIBUTES args — flat-thunked, the kernel
     * read the string pointer from PAST THE END of the guest's 8-byte struct.
     * NtSetValueKey is the hot one: it BUGCHECKs the box (0x1E) on the garbage
     * pointer, so every RegSetValueEx from a 32-bit guest was a local DoS. */
    if Wow64pStrEq(Name, b"NtSetValueKey\0" as *const u8 as *const ::core::ffi::c_char) != 0 {
        return Wow64Manual_NtSetValueKey(Arguments);
    }
    if Wow64pStrEq(Name, b"NtDeleteValueKey\0" as *const u8 as *const ::core::ffi::c_char) != 0 {
        return Wow64Manual_NtDeleteValueKey(Arguments);
    }
    if Wow64pStrEq(Name, b"NtRenameKey\0" as *const u8 as *const ::core::ffi::c_char) != 0 {
        return Wow64Manual_NtRenameKey(Arguments);
    }
    if Wow64pStrEq(Name, b"NtDeleteFile\0" as *const u8 as *const ::core::ffi::c_char) != 0 {
        return Wow64pCallOa1(Some(core::mem::transmute::<*const (), _>(NtDeleteFile as *const ())), Arguments);
    }
    if Wow64pStrEq(Name, b"NtUnloadKey\0" as *const u8 as *const ::core::ffi::c_char) != 0 {
        return Wow64pCallOa1(Some(core::mem::transmute::<*const (), _>(NtUnloadKey as *const ())), Arguments);
    }
    if Wow64pStrEq(Name, b"NtDisplayString\0" as *const u8 as *const ::core::ffi::c_char) != 0 {
        return Wow64pCallUstr1(Some(core::mem::transmute::<*const (), _>(NtDisplayString as *const ())), Arguments);
    }
    if Wow64pStrEq(Name, b"NtLoadDriver\0" as *const u8 as *const ::core::ffi::c_char) != 0 {
        return Wow64pCallUstr1(Some(core::mem::transmute::<*const (), _>(NtLoadDriver as *const ())), Arguments);
    }
    if Wow64pStrEq(Name, b"NtUnloadDriver\0" as *const u8 as *const ::core::ffi::c_char) != 0 {
        return Wow64pCallUstr1(Some(core::mem::transmute::<*const (), _>(NtUnloadDriver as *const ())), Arguments);
    }
    if Wow64pStrEq(Name, b"NtQuerySystemEnvironmentValue\0" as *const u8 as *const ::core::ffi::c_char) != 0 {
        return Wow64Manual_NtQuerySystemEnvironmentValue(Arguments);
    }
    if Wow64pStrEq(Name, b"NtSetSystemEnvironmentValue\0" as *const u8 as *const ::core::ffi::c_char) != 0 {
        return Wow64Manual_NtSetSystemEnvironmentValue(Arguments);
    }
    if Wow64pStrEq(Name, b"NtQuerySystemEnvironmentValueEx\0" as *const u8 as *const ::core::ffi::c_char) != 0 {
        return Wow64Manual_NtQuerySystemEnvironmentValueEx(Arguments);
    }
    if Wow64pStrEq(Name, b"NtSetSystemEnvironmentValueEx\0" as *const u8 as *const ::core::ffi::c_char) != 0 {
        return Wow64Manual_NtSetSystemEnvironmentValueEx(Arguments);
    }
    if Wow64pStrEq(Name, b"NtCreatePagingFile\0" as *const u8 as *const ::core::ffi::c_char) != 0 {
        return Wow64Manual_NtCreatePagingFile(Arguments);
    }
    if Wow64pStrEq(Name, b"NtLoadKey\0" as *const u8 as *const ::core::ffi::c_char) != 0 {
        return Wow64Manual_NtLoadKey(Arguments);
    }
    if Wow64pStrEq(Name, b"NtLoadKey2\0" as *const u8 as *const ::core::ffi::c_char) != 0 {
        return Wow64Manual_NtLoadKey2(Arguments);
    }
    if Wow64pStrEq(Name, b"NtLoadKeyEx\0" as *const u8 as *const ::core::ffi::c_char) != 0 {
        return Wow64Manual_NtLoadKeyEx(Arguments);
    }
    if Wow64pStrEq(Name, b"NtReplaceKey\0" as *const u8 as *const ::core::ffi::c_char) != 0 {
        return Wow64Manual_NtReplaceKey(Arguments);
    }
    if Wow64pStrEq(Name, b"NtUnloadKey2\0" as *const u8 as *const ::core::ffi::c_char) != 0 {
        return Wow64Manual_NtUnloadKey2(Arguments);
    }
    if Wow64pStrEq(Name, b"NtUnloadKeyEx\0" as *const u8 as *const ::core::ffi::c_char) != 0 {
        return Wow64Manual_NtUnloadKeyEx(Arguments);
    }
    if Wow64pStrEq(Name, b"NtQueryOpenSubKeys\0" as *const u8 as *const ::core::ffi::c_char) != 0 {
        return Wow64Manual_NtQueryOpenSubKeys(Arguments);
    }
    if Wow64pStrEq(Name, b"NtQueryOpenSubKeysEx\0" as *const u8 as *const ::core::ffi::c_char) != 0 {
        return Wow64Manual_NtQueryOpenSubKeysEx(Arguments);
    }
    if Wow64pStrEq(Name, b"NtAccessCheckAndAuditAlarm\0" as *const u8 as *const ::core::ffi::c_char) != 0 {
        return Wow64Manual_NtAccessCheckAndAuditAlarm(Arguments);
    }
    if Wow64pStrEq(Name, b"NtOpenObjectAuditAlarm\0" as *const u8 as *const ::core::ffi::c_char) != 0 {
        return Wow64Manual_NtOpenObjectAuditAlarm(Arguments);
    }
    if Wow64pStrEq(Name, b"NtCloseObjectAuditAlarm\0" as *const u8 as *const ::core::ffi::c_char) != 0 {
        return Wow64pAuditClose(Some(core::mem::transmute::<*const (), _>(NtCloseObjectAuditAlarm as *const ())), Arguments);
    }
    if Wow64pStrEq(Name, b"NtDeleteObjectAuditAlarm\0" as *const u8 as *const ::core::ffi::c_char) != 0 {
        return Wow64pAuditClose(Some(core::mem::transmute::<*const (), _>(NtDeleteObjectAuditAlarm as *const ())), Arguments);
    }
    if Wow64pStrEq(Name, b"NtPrivilegeObjectAuditAlarm\0" as *const u8 as *const ::core::ffi::c_char) != 0 {
        return Wow64Manual_NtPrivilegeObjectAuditAlarm(Arguments);
    }
    if Wow64pStrEq(Name, b"NtPrivilegedServiceAuditAlarm\0" as *const u8 as *const ::core::ffi::c_char) != 0 {
        return Wow64Manual_NtPrivilegedServiceAuditAlarm(Arguments);
    }
    if Wow64pStrEq(Name, b"NtQuerySymbolicLinkObject\0" as *const u8 as *const ::core::ffi::c_char) != 0 {
        return Wow64Manual_NtQuerySymbolicLinkObject(Arguments);
    }
    if Wow64pStrEq(Name, b"NtQueryObject\0" as *const u8 as *const ::core::ffi::c_char) != 0 {
        return Wow64Manual_NtQueryObject(Arguments);
    }
    if Wow64pStrEq(Name, b"NtAllocateVirtualMemory\0" as *const u8 as *const ::core::ffi::c_char) != 0 {
        return Wow64Manual_NtAllocateVirtualMemory(Arguments);
    }
    if Wow64pStrEq(Name, b"NtFreeVirtualMemory\0" as *const u8 as *const ::core::ffi::c_char) != 0 {
        return Wow64Manual_NtFreeVirtualMemory(Arguments);
    }
    if Wow64pStrEq(Name, b"NtProtectVirtualMemory\0" as *const u8 as *const ::core::ffi::c_char) != 0 {
        return Wow64Manual_NtProtectVirtualMemory(Arguments);
    }
    if Wow64pStrEq(Name, b"NtMapViewOfSection\0" as *const u8 as *const ::core::ffi::c_char) != 0 {
        return Wow64Manual_NtMapViewOfSection(Arguments);
    }
    if Wow64pStrEq(Name, b"NtUnmapViewOfSection\0" as *const u8 as *const ::core::ffi::c_char) != 0 {
        return Wow64Manual_NtUnmapViewOfSection(Arguments);
    }
    if Wow64pStrEq(Name, b"NtReadVirtualMemory\0" as *const u8 as *const ::core::ffi::c_char) != 0 {
        return Wow64Manual_NtReadVirtualMemory(Arguments);
    }
    if Wow64pStrEq(Name, b"NtWriteVirtualMemory\0" as *const u8 as *const ::core::ffi::c_char) != 0 {
        return Wow64Manual_NtWriteVirtualMemory(Arguments);
    }
    if Wow64pStrEq(Name, b"NtFlushInstructionCache\0" as *const u8 as *const ::core::ffi::c_char) != 0 {
        return Wow64Manual_NtFlushInstructionCache(Arguments);
    }
    if Wow64pStrEq(Name, b"NtSetInformationProcess\0" as *const u8 as *const ::core::ffi::c_char) != 0 {
        return Wow64Manual_NtSetInformationProcess(Arguments);
    }
    if Wow64pStrEq(Name, b"NtOpenFile\0" as *const u8 as *const ::core::ffi::c_char) != 0 {
        return Wow64Manual_NtOpenFile(Arguments);
    }
    if Wow64pStrEq(Name, b"NtCreateFile\0" as *const u8 as *const ::core::ffi::c_char) != 0 {
        return Wow64Manual_NtCreateFile(Arguments);
    }
    if Wow64pStrEq(Name, b"NtCreateSection\0" as *const u8 as *const ::core::ffi::c_char) != 0 {
        return Wow64Manual_NtCreateSection(Arguments);
    }
    if Wow64pStrEq(Name, b"NtReadFile\0" as *const u8 as *const ::core::ffi::c_char) != 0 {
        return Wow64Manual_NtReadFile(Arguments);
    }
    if Wow64pStrEq(Name, b"NtWriteFile\0" as *const u8 as *const ::core::ffi::c_char) != 0 {
        return Wow64Manual_NtWriteFile(Arguments);
    }
    if Wow64pStrEq(Name, b"NtQueryInformationFile\0" as *const u8 as *const ::core::ffi::c_char) != 0 {
        return Wow64Manual_NtQueryInformationFile(Arguments);
    }
    if Wow64pStrEq(Name, b"NtSetInformationFile\0" as *const u8 as *const ::core::ffi::c_char) != 0 {
        return Wow64Manual_NtSetInformationFile(Arguments);
    }
    if Wow64pStrEq(Name, b"NtQueryVolumeInformationFile\0" as *const u8 as *const ::core::ffi::c_char) != 0 {
        return Wow64Manual_NtQueryVolumeInformationFile(Arguments);
    }
    if Wow64pStrEq(Name, b"NtFlushBuffersFile\0" as *const u8 as *const ::core::ffi::c_char) != 0 {
        return Wow64Manual_NtFlushBuffersFile(Arguments);
    }
    if Wow64pStrEq(Name, b"NtQueryDirectoryFile\0" as *const u8 as *const ::core::ffi::c_char) != 0 {
        return Wow64Manual_NtQueryDirectoryFile(Arguments);
    }
    if Wow64pStrEq(Name, b"NtQueryAttributesFile\0" as *const u8 as *const ::core::ffi::c_char) != 0 {
        return Wow64Manual_NtQueryAttributesFile(Arguments);
    }
    if Wow64pStrEq(Name, b"NtQueryFullAttributesFile\0" as *const u8 as *const ::core::ffi::c_char) != 0 {
        return Wow64Manual_NtQueryFullAttributesFile(Arguments);
    }
    if Wow64pStrEq(Name, b"NtQuerySection\0" as *const u8 as *const ::core::ffi::c_char) != 0 {
        return Wow64Manual_NtQuerySection(Arguments);
    }
    if Wow64pStrEq(Name, b"NtCreateKeyedEvent\0" as *const u8 as *const ::core::ffi::c_char) != 0 {
        return Wow64Manual_NtCreateKeyedEvent(Arguments);
    }
    if Wow64pStrEq(Name, b"NtSecureConnectPort\0" as *const u8 as *const ::core::ffi::c_char) != 0 {
        return Wow64Manual_NtSecureConnectPort(Arguments);
    }
    if Wow64pStrEq(Name, b"NtRequestWaitReplyPort\0" as *const u8 as *const ::core::ffi::c_char) != 0 {
        return Wow64Manual_NtRequestWaitReplyPort(Arguments);
    }
    if Wow64pStrEq(Name, b"NtConnectPort\0" as *const u8 as *const ::core::ffi::c_char) != 0 {
        return Wow64Manual_NtConnectPort(Arguments);
    }
    if Wow64pStrEq(Name, b"NtContinue\0" as *const u8 as *const ::core::ffi::c_char) != 0 {
        return Wow64Manual_NtContinue(Arguments);
    }
    if Wow64pStrEq(Name, b"NtQueueApcThread\0" as *const u8 as *const ::core::ffi::c_char) != 0 {
        return Wow64Manual_NtQueueApcThread(Arguments);
    }
    if Wow64pStrEq(Name, b"NtSetTimer\0" as *const u8 as *const ::core::ffi::c_char) != 0 {
        return Wow64Manual_NtSetTimer(Arguments);
    }
    if Wow64pStrEq(Name, b"NtRaiseException\0" as *const u8 as *const ::core::ffi::c_char) != 0 {
        return Wow64Manual_NtRaiseException(Arguments);
    }
    if Wow64pStrEq(Name, b"NtCreateDirectoryObject\0" as *const u8 as *const ::core::ffi::c_char) != 0 {
        return Wow64pOpenByOa(Some(core::mem::transmute::<*const (), _>(NtCreateDirectoryObject as *const ())), Arguments);
    }
    if Wow64pStrEq(Name, b"NtCreateEvent\0" as *const u8 as *const ::core::ffi::c_char) != 0 {
        return Wow64Manual_NtCreateEvent(Arguments);
    }
    if Wow64pStrEq(Name, b"NtOpenEvent\0" as *const u8 as *const ::core::ffi::c_char) != 0 {
        return Wow64pOpenByOa(Some(core::mem::transmute::<*const (), _>(NtOpenEvent as *const ())), Arguments);
    }
    if Wow64pStrEq(Name, b"NtGdiOpenDCW\0" as *const u8 as *const ::core::ffi::c_char) != 0 {
        return Wow64Manual_NtGdiOpenDCW(Arguments);
    }
    if Wow64pStrEq(Name, b"NtUserProcessConnect\0" as *const u8 as *const ::core::ffi::c_char) != 0 {
        return Wow64Manual_NtUserProcessConnect(Arguments);
    }
    if Wow64pStrEq(Name, b"NtUserMessageCall\0" as *const u8 as *const ::core::ffi::c_char) != 0 {
        return Wow64Manual_NtUserMessageCall(Arguments);
    }
    if Wow64pStrEq(Name, b"NtQueryValueKey\0" as *const u8 as *const ::core::ffi::c_char) != 0 {
        return Wow64Manual_NtQueryValueKey(Arguments);
    }
    if Wow64pStrEq(Name, b"NtUserBuildPropList\0" as *const u8 as *const ::core::ffi::c_char) != 0 {
        return Wow64Manual_NtUserBuildPropList(Arguments);
    }
    if Wow64pStrEq(Name, b"NtUserSetWindowLong\0" as *const u8 as *const ::core::ffi::c_char) != 0 {
        return Wow64Manual_NtUserSetWindowLong(Arguments);
    }
    if Wow64pStrEq(Name, b"NtUserSetClassLong\0" as *const u8 as *const ::core::ffi::c_char) != 0 {
        return Wow64Manual_NtUserSetClassLong(Arguments);
    }
    if Wow64pStrEq(Name, b"NtUserGetCPD\0" as *const u8 as *const ::core::ffi::c_char) != 0 {
        return Wow64Manual_NtUserGetCPD(Arguments);
    }
    if Wow64pStrEq(Name, b"NtUserRegisterClassExWOW\0" as *const u8 as *const ::core::ffi::c_char) != 0 {
        return Wow64Manual_NtUserRegisterClassExWOW(Arguments);
    }
    if Wow64pStrEq(Name, b"NtUserGetClassInfo\0" as *const u8 as *const ::core::ffi::c_char) != 0 {
        return Wow64Manual_NtUserGetClassInfo(Arguments);
    }
    if Wow64pStrEq(Name, b"NtUserGetAtomName\0" as *const u8 as *const ::core::ffi::c_char) != 0 {
        return Wow64Manual_NtUserGetAtomName(Arguments);
    }
    if Wow64pStrEq(Name, b"NtUserGetClassName\0" as *const u8 as *const ::core::ffi::c_char) != 0 {
        return Wow64Manual_NtUserGetClassName(Arguments);
    }
    if Wow64pStrEq(Name, b"NtUserRegisterWindowMessage\0" as *const u8 as *const ::core::ffi::c_char) != 0 {
        return Wow64Manual_NtUserRegisterWindowMessage(Arguments);
    }
    if Wow64pStrEq(Name, b"NtUserGetKeyboardLayoutName\0" as *const u8 as *const ::core::ffi::c_char) != 0 {
        return Wow64Manual_NtUserGetKeyboardLayoutName(Arguments);
    }
    if Wow64pStrEq(Name, b"NtUserUnregisterClass\0" as *const u8 as *const ::core::ffi::c_char) != 0 {
        return Wow64Manual_NtUserUnregisterClass(Arguments);
    }
    if Wow64pStrEq(Name, b"NtUserFindWindowEx\0" as *const u8 as *const ::core::ffi::c_char) != 0 {
        return Wow64Manual_NtUserFindWindowEx(Arguments);
    }
    if Wow64pStrEq(Name, b"NtUserGetWOWClass\0" as *const u8 as *const ::core::ffi::c_char) != 0 {
        return Wow64Manual_NtUserGetWOWClass(Arguments);
    }
    if Wow64pStrEq(Name, b"NtUserSetWindowsHookEx\0" as *const u8 as *const ::core::ffi::c_char) != 0 {
        return Wow64Manual_NtUserSetWindowsHookEx(Arguments);
    }
    if Wow64pStrEq(Name, b"NtUserSetWinEventHook\0" as *const u8 as *const ::core::ffi::c_char) != 0 {
        return Wow64Manual_NtUserSetWinEventHook(Arguments);
    }
    if Wow64pStrEq(Name, b"NtUserRegisterUserApiHook\0" as *const u8 as *const ::core::ffi::c_char) != 0 {
        return Wow64Manual_NtUserRegisterUserApiHook(Arguments);
    }
    if Wow64pStrEq(Name, b"NtUserDrawCaptionTemp\0" as *const u8 as *const ::core::ffi::c_char) != 0 {
        return Wow64Manual_NtUserDrawCaptionTemp(Arguments);
    }
    if Wow64pStrEq(Name, b"NtUserLoadKeyboardLayoutEx\0" as *const u8 as *const ::core::ffi::c_char) != 0 {
        return Wow64Manual_NtUserLoadKeyboardLayoutEx(Arguments);
    }
    if Wow64pStrEq(Name, b"NtUserEnumDisplaySettings\0" as *const u8 as *const ::core::ffi::c_char) != 0 {
        return Wow64Manual_NtUserEnumDisplaySettings(Arguments);
    }
    if Wow64pStrEq(Name, b"NtUserEnumDisplayDevices\0" as *const u8 as *const ::core::ffi::c_char) != 0 {
        return Wow64Manual_NtUserEnumDisplayDevices(Arguments);
    }
    /* E10 (marshalgen audit) */
    if Wow64pStrEq(Name, b"NtGdiCreateDIBSection\0" as *const u8 as *const ::core::ffi::c_char) != 0 {
        return Wow64Manual_NtGdiCreateDIBSection(Arguments);
    }
    if Wow64pStrEq(Name, b"NtUserTranslateAccelerator\0" as *const u8 as *const ::core::ffi::c_char) != 0 {
        return Wow64Manual_NtUserTranslateAccelerator(Arguments);
    }
    if Wow64pStrEq(Name, b"NtUserCallMsgFilter\0" as *const u8 as *const ::core::ffi::c_char) != 0 {
        return Wow64Manual_NtUserCallMsgFilter(Arguments);
    }
    if Wow64pStrEq(Name, b"NtUserRealInternalGetMessage\0" as *const u8 as *const ::core::ffi::c_char) != 0 {
        return Wow64Manual_NtUserRealInternalGetMessage(Arguments);
    }
    if Wow64pStrEq(Name, b"NtUserResolveDesktop\0" as *const u8 as *const ::core::ffi::c_char) != 0 {
        return Wow64Manual_NtUserResolveDesktop(Arguments);
    }
    if Wow64pStrEq(Name, b"NtUserOpenDesktop\0" as *const u8 as *const ::core::ffi::c_char) != 0 {
        return Wow64Manual_NtUserOpenDesktop(Arguments);
    }
    if Wow64pStrEq(Name, b"NtUserOpenWindowStation\0" as *const u8 as *const ::core::ffi::c_char) != 0 {
        return Wow64Manual_NtUserOpenWindowStation(Arguments);
    }
    if Wow64pStrEq(Name, b"NtUserCreateWindowStation\0" as *const u8 as *const ::core::ffi::c_char) != 0 {
        return Wow64Manual_NtUserCreateWindowStation(Arguments);
    }
    if Wow64pStrEq(Name, b"NtUserCreateDesktop\0" as *const u8 as *const ::core::ffi::c_char) != 0 {
        return Wow64Manual_NtUserCreateDesktop(Arguments);
    }
    if Wow64pStrEq(Name, b"NtUserBuildHwndList\0" as *const u8 as *const ::core::ffi::c_char) != 0 {
        return Wow64Manual_NtUserBuildHwndList(Arguments);
    }
    if Wow64pStrEq(Name, b"NtUserThunkedMenuItemInfo\0" as *const u8 as *const ::core::ffi::c_char) != 0 {
        return Wow64Manual_NtUserThunkedMenuItemInfo(Arguments);
    }
    if Wow64pStrEq(Name, b"NtUserCreateWindowEx\0" as *const u8 as *const ::core::ffi::c_char) != 0 {
        return Wow64Manual_NtUserCreateWindowEx(Arguments);
    }
    if Wow64pStrEq(Name, b"NtUserGetMessage\0" as *const u8 as *const ::core::ffi::c_char) != 0 {
        return Wow64Manual_NtUserGetMessage(Arguments);
    }
    if Wow64pStrEq(Name, b"NtUserPeekMessage\0" as *const u8 as *const ::core::ffi::c_char) != 0 {
        return Wow64Manual_NtUserPeekMessage(Arguments);
    }
    if Wow64pStrEq(Name, b"NtUserDispatchMessage\0" as *const u8 as *const ::core::ffi::c_char) != 0 {
        return Wow64Manual_NtUserDispatchMessage(Arguments);
    }
    if Wow64pStrEq(Name, b"NtUserTranslateMessage\0" as *const u8 as *const ::core::ffi::c_char) != 0 {
        return Wow64Manual_NtUserTranslateMessage(Arguments);
    }
    if Wow64pStrEq(Name, b"NtUserBeginPaint\0" as *const u8 as *const ::core::ffi::c_char) != 0 {
        return Wow64Manual_NtUserBeginPaint(Arguments);
    }
    if Wow64pStrEq(Name, b"NtUserEndPaint\0" as *const u8 as *const ::core::ffi::c_char) != 0 {
        return Wow64Manual_NtUserEndPaint(Arguments);
    }
    if Wow64pStrEq(Name, b"NtUserFindExistingCursorIcon\0" as *const u8 as *const ::core::ffi::c_char) != 0 {
        return Wow64Manual_NtUserFindExistingCursorIcon(Arguments);
    }
    if Wow64pStrEq(Name, b"NtUserSetCursorIconData\0" as *const u8 as *const ::core::ffi::c_char) != 0 {
        return Wow64Manual_NtUserSetCursorIconData(Arguments);
    }
    if Wow64pStrEq(Name, b"NtUserGetIconInfo\0" as *const u8 as *const ::core::ffi::c_char) != 0 {
        return Wow64Manual_NtUserGetIconInfo(Arguments);
    }
    if Wow64pStrEq(Name, b"NtGdiExtGetObjectW\0" as *const u8 as *const ::core::ffi::c_char) != 0 {
        return Wow64Manual_NtGdiExtGetObjectW(Arguments);
    }
    if Wow64pStrEq(Name, b"NtUserDefSetText\0" as *const u8 as *const ::core::ffi::c_char) != 0 {
        return Wow64Manual_NtUserDefSetText(Arguments);
    }
    if Wow64pStrEq(Name, b"NtCallbackReturn\0" as *const u8 as *const ::core::ffi::c_char) != 0 {
        return Wow64Manual_NtCallbackReturn(Arguments);
    }

    /* M12: object creators/openers. */
    if Wow64pStrEq(Name, b"NtCreateSemaphore\0" as *const u8 as *const ::core::ffi::c_char) != 0 {
        return Wow64Manual_NtCreateSemaphore(Arguments);
    }
    if Wow64pStrEq(Name, b"NtOpenSemaphore\0" as *const u8 as *const ::core::ffi::c_char) != 0 {
        return Wow64pOpenByOa(Some(core::mem::transmute::<*const (), _>(NtOpenSemaphore as *const ())), Arguments);
    }
    if Wow64pStrEq(Name, b"NtCreateMutant\0" as *const u8 as *const ::core::ffi::c_char) != 0 {
        return Wow64Manual_NtCreateMutant(Arguments);
    }
    if Wow64pStrEq(Name, b"NtOpenMutant\0" as *const u8 as *const ::core::ffi::c_char) != 0 {
        return Wow64pOpenByOa(Some(core::mem::transmute::<*const (), _>(NtOpenMutant as *const ())), Arguments);
    }
    if Wow64pStrEq(Name, b"NtOpenKeyedEvent\0" as *const u8 as *const ::core::ffi::c_char) != 0 {
        return Wow64pOpenByOa(Some(core::mem::transmute::<*const (), _>(NtOpenKeyedEvent as *const ())), Arguments);
    }
    if Wow64pStrEq(Name, b"NtCreateKey\0" as *const u8 as *const ::core::ffi::c_char) != 0 {
        return Wow64Manual_NtCreateKey(Arguments);
    }
    if Wow64pStrEq(Name, b"NtCreateSymbolicLinkObject\0" as *const u8 as *const ::core::ffi::c_char) != 0 {
        return Wow64Manual_NtCreateSymbolicLinkObject(Arguments);
    }
    if Wow64pStrEq(Name, b"NtOpenProcess\0" as *const u8 as *const ::core::ffi::c_char) != 0 {
        return Wow64pOpenByCid(Some(core::mem::transmute::<*const (), _>(NtOpenProcess as *const ())), Arguments);
    }
    if Wow64pStrEq(Name, b"NtOpenThread\0" as *const u8 as *const ::core::ffi::c_char) != 0 {
        return Wow64pOpenByCid(Some(core::mem::transmute::<*const (), _>(NtOpenThread as *const ())), Arguments);
    }
    if Wow64pStrEq(Name, b"NtCreateThread\0" as *const u8 as *const ::core::ffi::c_char) != 0 {
        return Wow64Manual_NtCreateThread(Arguments);
    }
    if Wow64pStrEq(Name, b"NtWaitForMultipleObjects\0" as *const u8 as *const ::core::ffi::c_char) != 0 {
        return Wow64Manual_NtWaitForMultipleObjects(Arguments);
    }
    if Wow64pStrEq(Name, b"NtTerminateThread\0" as *const u8 as *const ::core::ffi::c_char) != 0 {
        return Wow64Manual_NtTerminateThread(Arguments);
    }
    if Wow64pStrEq(Name, b"NtUserInitializeClientPfnArrays\0" as *const u8 as *const ::core::ffi::c_char) != 0 {
        return Wow64Manual_NtUserInitializeClientPfnArrays(Arguments);
    }
    if Wow64pStrEq(Name, b"NtOpenProcessTokenEx\0" as *const u8 as *const ::core::ffi::c_char) != 0 {
        return Wow64Manual_NtOpenProcessTokenEx(Arguments);
    }
    if Wow64pStrEq(Name, b"NtOpenThreadTokenEx\0" as *const u8 as *const ::core::ffi::c_char) != 0 {
        return Wow64Manual_NtOpenThreadTokenEx(Arguments);
    }
    if Wow64pStrEq(Name, b"NtGetContextThread\0" as *const u8 as *const ::core::ffi::c_char) != 0 {
        return Wow64Manual_NtGetContextThread(Arguments);
    }
    if Wow64pStrEq(Name, b"NtSetContextThread\0" as *const u8 as *const ::core::ffi::c_char) != 0 {
        return Wow64Manual_NtSetContextThread(Arguments);
    }

    /* M12: the rest of the _IO_STATUS_BLOCK family. */
    if Wow64pStrEq(Name, b"NtCancelIoFile\0" as *const u8 as *const ::core::ffi::c_char) != 0 {
        return Wow64Manual_NtCancelIoFile(Arguments);
    }
    if Wow64pStrEq(Name, b"NtLockFile\0" as *const u8 as *const ::core::ffi::c_char) != 0 {
        return Wow64Manual_NtLockFile(Arguments);
    }
    if Wow64pStrEq(Name, b"NtUnlockFile\0" as *const u8 as *const ::core::ffi::c_char) != 0 {
        return Wow64Manual_NtUnlockFile(Arguments);
    }
    if Wow64pStrEq(Name, b"NtSetVolumeInformationFile\0" as *const u8 as *const ::core::ffi::c_char) != 0 {
        return Wow64Manual_NtSetVolumeInformationFile(Arguments);
    }
    if Wow64pStrEq(Name, b"NtQueryEaFile\0" as *const u8 as *const ::core::ffi::c_char) != 0 {
        return Wow64Manual_NtQueryEaFile(Arguments);
    }
    if Wow64pStrEq(Name, b"NtSetEaFile\0" as *const u8 as *const ::core::ffi::c_char) != 0 {
        return Wow64Manual_NtSetEaFile(Arguments);
    }
    if Wow64pStrEq(Name, b"NtDeviceIoControlFile\0" as *const u8 as *const ::core::ffi::c_char) != 0 {
        return Wow64pIoControl(
            Some(core::mem::transmute::<*const (), _>(NtDeviceIoControlFile as *const ())),
            Arguments,
            b"NtDeviceIoControlFile\0" as *const u8 as *const ::core::ffi::c_char,
        );
    }
    if Wow64pStrEq(Name, b"NtFsControlFile\0" as *const u8 as *const ::core::ffi::c_char) != 0 {
        return Wow64pIoControl(
            Some(core::mem::transmute::<*const (), _>(NtFsControlFile as *const ())),
            Arguments,
            b"NtFsControlFile\0" as *const u8 as *const ::core::ffi::c_char,
        );
    }
    if Wow64pStrEq(Name, b"NtCreateNamedPipeFile\0" as *const u8 as *const ::core::ffi::c_char) != 0 {
        return Wow64Manual_NtCreateNamedPipeFile(Arguments);
    }
    if Wow64pStrEq(Name, b"NtCreateMailslotFile\0" as *const u8 as *const ::core::ffi::c_char) != 0 {
        return Wow64Manual_NtCreateMailslotFile(Arguments);
    }

    /* M12: loud refusals — reachable, but not correctly doable yet. */
    if Wow64pStrEq(Name, b"NtNotifyChangeDirectoryFile\0" as *const u8 as *const ::core::ffi::c_char) != 0 {
        return Wow64Manual_NtNotifyChangeDirectoryFile(Arguments);
    }
    if Wow64pStrEq(Name, b"NtNotifyChangeKey\0" as *const u8 as *const ::core::ffi::c_char) != 0 {
        return Wow64Manual_NtNotifyChangeKey(Arguments);
    }
    if Wow64pStrEq(Name, b"NtNotifyChangeMultipleKeys\0" as *const u8 as *const ::core::ffi::c_char) != 0 {
        return Wow64Manual_NtNotifyChangeMultipleKeys(Arguments);
    }
    if Wow64pStrEq(Name, b"NtSetSystemInformation\0" as *const u8 as *const ::core::ffi::c_char) != 0 {
        return Wow64Manual_NtSetSystemInformation(Arguments);
    }
    if Wow64pStrEq(Name, b"NtCreateIoCompletion\0" as *const u8 as *const ::core::ffi::c_char) != 0 {
        return Wow64Manual_NtCreateIoCompletion(Arguments);
    }
    if Wow64pStrEq(Name, b"NtOpenIoCompletion\0" as *const u8 as *const ::core::ffi::c_char) != 0 {
        return Wow64pOpenByOa(Some(core::mem::transmute::<*const (), _>(NtOpenIoCompletion as *const ())), Arguments);
    }
    if Wow64pStrEq(Name, b"NtQueryInformationThread\0" as *const u8 as *const ::core::ffi::c_char) != 0 {
        return Wow64Manual_NtQueryInformationThread(Arguments);
    }
    if Wow64pStrEq(Name, b"NtRemoveIoCompletion\0" as *const u8 as *const ::core::ffi::c_char) != 0 {
        return Wow64Manual_NtRemoveIoCompletion(Arguments);
    }
    if Wow64pStrEq(Name, b"NtRemoveIoCompletionEx\0" as *const u8 as *const ::core::ffi::c_char) != 0 {
        return Wow64Manual_NtRemoveIoCompletionEx(Arguments);
    }
    if Wow64pStrEq(Name, b"NtCancelIoFileEx\0" as *const u8 as *const ::core::ffi::c_char) != 0 {
        return Wow64Manual_NtCancelIoFileEx(Arguments);
    }
    if Wow64pStrEq(Name, b"NtCancelSynchronousIoFile\0" as *const u8 as *const ::core::ffi::c_char) != 0 {
        return Wow64Manual_NtCancelSynchronousIoFile(Arguments);
    }
    if Wow64pStrEq(Name, b"NtUserEnumDisplayMonitors\0" as *const u8 as *const ::core::ffi::c_char) != 0 {
        return Wow64Manual_NtUserEnumDisplayMonitors(Arguments);
    }
    if Wow64pStrEq(Name, b"NtUserSendInput\0" as *const u8 as *const ::core::ffi::c_char) != 0 {
        return Wow64Manual_NtUserSendInput(Arguments);
    }
    if Wow64pStrEq(Name, b"NtUserThunkedMenuInfo\0" as *const u8 as *const ::core::ffi::c_char) != 0 {
        return Wow64Manual_NtUserThunkedMenuInfo(Arguments);
    }
    /* M17: fall through to the marshalgen-generated thunks. */
    return Wow64AutoMarshalDispatch(Name, Arguments);
}
#[no_mangle]
pub unsafe extern "C" fn Wow64pStrEq(mut a: *const i8, mut b: *const i8) -> i32 {
    while *a != 0 && *a == *b {
        a = a.add(1);
        b = b.add(1);
    }
    (*a == *b) as i32
}
