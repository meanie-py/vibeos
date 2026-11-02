use crate::wow64::wow64_cb_automarshal::LONG;
use crate::wow64::wow64_cb_automarshal::LONGLONG;
use crate::wow64::wow64_cb_automarshal::ULONG;
// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_80073828D8A0: [u16; 10] = [0x006Eu16, 0x0074, 0x0064, 0x006C, 0x006C, 0x002E, 0x0064, 0x006C, 0x006C, 0x0000];
pub static RUSTCONV_WSTR_86A7B64C8DDA: [u16; 16] = [
    0x0043u16, 0x003A, 0x005C, 0x0052, 0x0065, 0x0061, 0x0063, 0x0074, 0x004F, 0x0053,
    0x005C, 0x0054, 0x0065, 0x006D, 0x0070, 0x0000,
];
pub static RUSTCONV_WSTR_990385C876A2: [u16; 13] = [
    0x006Bu16, 0x0065, 0x0072, 0x006E, 0x0065, 0x006C, 0x0033, 0x0032, 0x002E, 0x0064,
    0x006C, 0x006C, 0x0000,
];
pub static RUSTCONV_WSTR_A78DD113F020: [u16; 20] = [
    0x005Cu16, 0x0073, 0x0079, 0x0073, 0x0077, 0x006F, 0x0077, 0x0036, 0x0034, 0x005C,
    0x006E, 0x0074, 0x0064, 0x006C, 0x006C, 0x002E, 0x0064, 0x006C, 0x006C, 0x0000,
];
pub static RUSTCONV_WSTR_C554B73F8939: [u16; 5] = [0x005Cu16, 0x003F, 0x003F, 0x005C, 0x0000];
pub static RUSTCONV_WSTR_D1FD64EDDC71: [u16; 13] = [
    0x004Cu16, 0x004F, 0x0043, 0x0041, 0x004C, 0x0041, 0x0050, 0x0050, 0x0044, 0x0041,
    0x0054, 0x0041, 0x0000,
];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
static mut Wow64pSelfBase: PVOID =
    ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void;

// The CPU layer per arch, as in the C's `#ifdef _M_ARM64` (the translation kept only the
// arm64 name and the amd64 machine type, a mixed pair).
#[cfg(target_arch = "aarch64")]
pub static WOW64_CPU_DLL: [::core::ffi::c_ushort; 16] = unsafe {
    ::core::mem::transmute::<[u8; 32], [::core::ffi::c_ushort; 16]>(
        *b"l\0i\0b\0w\0o\0w\x006\x004\0f\0e\0x\0.\0d\0l\0l\0\0\0",
    )
};
#[cfg(target_arch = "x86_64")]
pub static WOW64_CPU_DLL: [::core::ffi::c_ushort; 13] = [
    b'w' as u16, b'o' as u16, b'w' as u16, b'6' as u16, b'4' as u16, b'c' as u16, b'p' as u16,
    b'u' as u16, b'.' as u16, b'd' as u16, b'l' as u16, b'l' as u16, 0,
];

static mut Wow64pWow64Info: WOW64INFO_FEX = _WOW64INFO_FEX {
    NativeSystemPageSize: 0,
    CpuFlags: 0,
    Wow64ExecuteFlags: 0,
    Unknown: 0,
    SectionHandle: 0,
    CrossProcessWorkList: 0,
    NativeMachineType: 0,
    EmulatedMachineType: 0,
};

static mut Wow64pBTCpuResetToConsistentState: Option<
    unsafe extern "C" fn(*mut EXCEPTION_POINTERS) -> LONG,
> = None;

#[no_mangle]
pub static mut Wow64pBTCpuNotifyMap: Option<
    unsafe extern "C" fn(PVOID, PVOID, PVOID, SIZE_T, ULONG, ULONG) -> LONG,
> = None;

#[no_mangle]
pub static mut Wow64pBTCpuSetContext: Option<
    unsafe extern "C" fn(HANDLE, HANDLE, PVOID, PVOID) -> LONG,
> = None;

#[no_mangle]
pub static mut Wow64pBTCpuGetContext: Option<
    unsafe extern "C" fn(HANDLE, HANDLE, PVOID, PVOID) -> LONG,
> = None;

#[no_mangle]
pub static mut Wow64pBTCpuSimulate: Option<unsafe extern "C" fn() -> ()> = None;

#[no_mangle]
pub static mut Wow64pKiUserCallbackDispatcher32: PVOID =
    ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void;

#[no_mangle]
pub static mut Wow64pKiUserExceptionDispatcher32: PVOID =
    ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void;

#[no_mangle]
pub static mut Wow64pBopCode: PVOID =
    ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void;

static mut Wow64pNtdll32Size: SIZE_T = 0;

static mut Wow64pPeb32: ULONG_PTR = 0;

static mut Wow64pBTCpuNotifyMemoryAlloc: Option<
    unsafe extern "C" fn(PVOID, SIZE_T, ULONG, ULONG, BOOL, ULONG) -> (),
> = None;

static mut Wow64pBTCpuNotifyMemoryProtect: Option<
    unsafe extern "C" fn(PVOID, SIZE_T, ULONG, BOOL, ULONG) -> (),
> = None;

static mut Wow64pBTCpuNotifyMemoryFree: Option<
    unsafe extern "C" fn(PVOID, SIZE_T, ULONG, BOOL, ULONG) -> (),
> = None;

static mut Wow64pBTCpuNotifyMemoryDirty: Option<unsafe extern "C" fn(PVOID, SIZE_T) -> ()> = None;

static mut Wow64pBTCpuNotifyUnmapViewOfSection: Option<
    unsafe extern "C" fn(PVOID, BOOL, ULONG) -> (),
> = None;

static mut Wow64pBTCpuNotifyReadFile: Option<
    unsafe extern "C" fn(HANDLE, PVOID, SIZE_T, BOOL, LONG) -> (),
> = None;

static mut Wow64pBTCpuNotifyProcessExecuteFlagsChange: Option<unsafe extern "C" fn(ULONG) -> ()> =
    None;

static mut Wow64pBTCpuFlushInstructionCache2: Option<
    unsafe extern "C" fn(*const ::core::ffi::c_void, SIZE_T) -> (),
> = None;

#[no_mangle]
pub static mut Wow64pKiUserApcDispatcher32: PVOID =
    ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void;

static mut Wow64pLdrInitThunk32: PVOID =
    ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void;

static mut Wow64pBTCpuThreadInit: Option<unsafe extern "C" fn() -> ()> = None;

static mut Wow64pGuestFaults: ULONG = 0;

pub static WOW64_SELF_DLL_NAME: [::core::ffi::c_ushort; 10] = unsafe {
    ::core::mem::transmute::<[u8; 20], [::core::ffi::c_ushort; 10]>(
        *b"w\0o\0w\x006\x004\0.\0d\0l\0l\0\0\0",
    )
};

pub const WOW64_CURRENT_PROCESS: HANDLE = -1isize as HANDLE;

pub const WOW64_CURRENT_THREAD: HANDLE = -2isize as HANDLE;

#[repr(C)]
pub struct WOW64_UNICODE_STRING {
    pub Length: USHORT,
    pub MaximumLength: USHORT,
    pub Buffer: PWSTR,
}
pub type PWOW64_UNICODE_STRING = *mut WOW64_UNICODE_STRING;

#[repr(C)]
pub struct WOW64_ANSI_STRING {
    pub Length: USHORT,
    pub MaximumLength: USHORT,
    pub Buffer: PSTR,
}
pub type PWOW64_ANSI_STRING = *mut WOW64_ANSI_STRING;

#[repr(C)]
pub struct WOW64_OBJECT_ATTRIBUTES {
    pub Length: ULONG,
    pub RootDirectory: HANDLE,
    pub ObjectName: *mut WOW64_UNICODE_STRING,
    pub Attributes: ULONG,
    pub SecurityDescriptor: PVOID,
    pub SecurityQualityOfService: PVOID,
}
pub type PWOW64_OBJECT_ATTRIBUTES = *mut WOW64_OBJECT_ATTRIBUTES;

#[repr(C)]
pub union WOW64_IO_STATUS_BLOCK_U0 {
    pub Status: LONG,
    pub Pointer: PVOID,
}

#[repr(C)]
pub struct WOW64_IO_STATUS_BLOCK {
    pub u0: WOW64_IO_STATUS_BLOCK_U0,
    pub Information: ULONG_PTR,
}
pub type PWOW64_IO_STATUS_BLOCK = *mut WOW64_IO_STATUS_BLOCK;

#[repr(C)]
pub struct WOW64_CLIENT_ID {
    pub UniqueProcess: HANDLE,
    pub UniqueThread: HANDLE,
}
pub type PWOW64_CLIENT_ID = *mut WOW64_CLIENT_ID;

#[repr(C)]
pub struct WOW64_UNICODE_STRING32 {
    pub Length: USHORT,
    pub MaximumLength: USHORT,
    pub Buffer: ULONG, // 32-bit guest pointer
}

#[repr(C)]
pub struct WOW64_RTL_USER_PROCESS_PARAMETERS32 {
    pub MaximumLength: ULONG,                  // +0x00
    pub Length: ULONG,                         // +0x04
    pub Flags: ULONG,                          // +0x08
    pub DebugFlags: ULONG,                     // +0x0c
    pub ConsoleHandle: ULONG,                  // +0x10
    pub ConsoleFlags: ULONG,                   // +0x14
    pub StandardInput: ULONG,                  // +0x18
    pub StandardOutput: ULONG,                 // +0x1c
    pub StandardError: ULONG,                  // +0x20
    pub CurrentDirectoryPath: WOW64_UNICODE_STRING32, // +0x24 CURDIR.DosPath
    pub CurrentDirectoryHandle: ULONG,         // +0x2c CURDIR.Handle
    pub DllPath: WOW64_UNICODE_STRING32,       // +0x30
    pub ImagePathName: WOW64_UNICODE_STRING32, // +0x38
    pub CommandLine: WOW64_UNICODE_STRING32,   // +0x40
    pub Environment: ULONG,                    // +0x48
    pub StartingX: ULONG,
    pub StartingY: ULONG,
    pub CountX: ULONG,
    pub CountY: ULONG,
    pub CountCharsX: ULONG,
    pub CountCharsY: ULONG,
    pub FillAttribute: ULONG,
    pub WindowFlags: ULONG,
    pub ShowWindowFlags: ULONG,
    pub WindowTitle: WOW64_UNICODE_STRING32,   // +0x70
    pub DesktopInfo: WOW64_UNICODE_STRING32,   // +0x78
    pub ShellInfo: WOW64_UNICODE_STRING32,     // +0x80
    pub RuntimeData: WOW64_UNICODE_STRING32,   // +0x88
}

pub const PEB64_PROCESS_PARAMETERS_OFF: usize = 0x20;

pub const PEB32_PROCESS_PARAMETERS_OFF: usize = 0x10;

pub const UPP64_CURDIR_OFF: usize = 0x38; // CURDIR.DosPath (_UNICODE_STRING)

pub const UPP64_CURDIRHANDLE_OFF: usize = 0x48; // CURDIR.Handle

pub const UPP64_DLLPATH_OFF: usize = 0x50;

pub const UPP64_IMAGEPATHNAME_OFF: usize = 0x60;

pub const UPP64_COMMANDLINE_OFF: usize = 0x70;

pub const UPP64_ENVIRONMENT_OFF: usize = 0x80;

pub const UPP64_WINDOWTITLE_OFF: usize = 0xB0;

pub const UPP64_DESKTOPINFO_OFF: usize = 0xC0;

pub const UPP64_CONSOLEHANDLE_OFF: usize = 0x10;

pub const UPP64_CONSOLEFLAGS_OFF: usize = 0x18;

pub const UPP64_STDINPUT_OFF: usize = 0x20;

pub const UPP64_STDOUTPUT_OFF: usize = 0x28;

pub const UPP64_STDERROR_OFF: usize = 0x30;

pub const UPP64_STARTINGX_OFF: usize = 0x88;

pub const UPP64_SHOWWINDOWFLAGS_OFF: usize = 0xA8;

pub const WOW64_MAX_ENVIRONMENT_BYTES: usize = 0x20000;

pub const WOW64B_SVC_ECHO: u32 = 0xF00Du32;

pub const WOW64B_SVC_VERDICT: u32 = 0xD00Eu32;

pub const WOW64B_ECHO_MAGIC: u32 = 0x5CA11EDu32;

pub const WOW64C_SVC_EXIT: u32 = 0xE0D1u32;

pub const WOW64C_HELLO32_SENTINEL: u32 = 0x51DEu32;

pub const WOW64C_STATUS_INVALID_HANDLE: u32 = 0xC0000008u32;

pub const WOW64_HEAP_ZERO_MEMORY: u32 = 0x00000008;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct WOW64_THREAD_SEED {
    pub Eip: ULONG,        // guest thread entry — kernel32's BaseThreadStartupThunk
    pub Esp: ULONG,
    pub Eax: ULONG,        // guest thread routine, per BaseInitializeContext
    pub Ebx: ULONG,        // guest thread parameter
    pub StackBase: ULONG,
    pub StackLimit: ULONG,
    pub DeallocationStack: ULONG, // INITIAL_TEB32.AllocatedStackBase
    // How to release this block. An in-process seed is heap memory; a seed
    // planted by ANOTHER process (CreateRemoteThread) is its own VirtualAlloc.
    pub Remote: ULONG,
}

pub const WOW64_GUEST_FROM64_STACK: usize = 1024 * 1024;

pub const LDR32_PEB_LDR_OFF: usize = 0x0C; // PEB32.Ldr

pub const LDR32_INLOADORDER_OFF: usize = 0x0C; // PEB_LDR_DATA32.InLoadOrderModuleList

pub const LDR32_ENTRY_DLLBASE: usize = 0x18; // LDR_DATA_TABLE_ENTRY32.DllBase

pub const LDR32_ENTRY_SIZEOFIMAGE: usize = 0x20; // .SizeOfImage

pub const LDR32_ENTRY_BASENAME: usize = 0x2C; // .BaseDllName.Length

pub const LDR32_ENTRY_BASENAMEBUF: usize = 0x30; // .BaseDllName.Buffer

pub const TEB32_PEB_OFF: usize = 0x30; // TEB32.ProcessEnvironmentBlock

pub const LDR64_PEB_LDR_OFF: usize = 0x18; // PEB64.Ldr

pub const LDR64_INLOADORDER_OFF: usize = 0x10; // PEB_LDR_DATA.InLoadOrderModuleList

pub const LDR64_ENTRY_DLLBASE: usize = 0x30; // LDR_DATA_TABLE_ENTRY.DllBase

pub const LDR64_ENTRY_BASENAME: usize = 0x58; // .BaseDllName.Length

pub const LDR64_ENTRY_BASENAMEBUF: usize = 0x60; // .BaseDllName.Buffer

pub const LDR64_MAX_MODULES: usize = 512;

pub const WOW64_TEB_TLSSLOTS_OFFSET: usize = 0x1480; // TEB64->TlsSlots[]

pub const WOW64_TLS_WOW64INFO: usize = 10;

pub const WOW64_TLS_CPURESERVED: usize = 1;

pub const WOW64_TLS_INCPUSIMULATION: usize = 2;

pub const WOW64_INITBLOCK_NTDLL_OFF: usize = 72; // SYSTEM_DLL_INIT_BLOCK.ntdll_handle

pub const WOW64_ZEROBITS_BELOW_2G: usize = 0x7FFFFFFF; // mask form

pub const WOW64_TEB_WOWTEBOFFSET: usize = 0x180c; // raw Win10 slot (see MmCreateTeb)

pub const WOW64_MACHINE_I386: u16 = 0x014C;

pub const WOW64_MACHINE_ARM64: u16 = 0xAA64;

pub const WOW64_MACHINE_AMD64: u16 = 0x8664;

#[cfg(target_arch = "aarch64")]
pub const WOW64_NATIVE_MACHINE: u16 = WOW64_MACHINE_ARM64;
#[cfg(target_arch = "x86_64")]
pub const WOW64_NATIVE_MACHINE: u16 = WOW64_MACHINE_AMD64;


pub const WOW64_TEB64_LASTERROR: usize = 0x68;

pub const WOW64_TEB32_LASTERROR: usize = 0x34;

#[repr(C)]
pub struct WOW64_PROCESS_BASIC_INFORMATION {
    pub ExitStatus: LONG,
    pub PebBaseAddress: PVOID,
    pub AffinityMask: ULONG_PTR,
    pub BasePriority: LONG,
    pub UniqueProcessId: ULONG_PTR,
    pub InheritedFromUniqueProcessId: ULONG_PTR,
}

#[repr(C)]
pub struct WOW64_EXCEPTION_RECORD32 {
    pub ExceptionCode: ULONG,
    pub ExceptionFlags: ULONG,
    pub ExceptionRecord: ULONG,          // guest EXCEPTION_RECORD32* (chain)
    pub ExceptionAddress: ULONG,         // guest PVOID
    pub NumberParameters: ULONG,
    pub ExceptionInformation: [ULONG; 15],
}

unsafe extern "C" fn Wow64pTlsSlots() -> *mut PVOID {
    return (NtCurrentTeb() as PBYTE).add(WOW64_TEB_TLSSLOTS_OFFSET as usize) as *mut PVOID;
}

unsafe extern "C" fn Wow64pCurrentTeb32() -> PBYTE {
    let mut Teb: PBYTE = NtCurrentTeb() as PBYTE;
    let mut WowTebOffset: LONG = *(Teb.offset(WOW64_TEB_WOWTEBOFFSET as isize) as *mut LONG);
    return if WowTebOffset != 0 {
        Teb.offset(WowTebOffset as isize)
    } else {
        core::ptr::null_mut::<BYTE>()
    };
}
#[no_mangle]
pub unsafe extern "C" fn Wow64LastErrorToHost() -> ULONG {
    let mut Teb32: PBYTE = Wow64pCurrentTeb32();
    let mut Guest: ULONG = 0;
    if Teb32.is_null() {
        return 0;
    }
    Guest = *(Teb32.offset(WOW64_TEB32_LASTERROR as isize) as *mut ULONG);
    *((NtCurrentTeb() as PBYTE).offset(WOW64_TEB64_LASTERROR as isize) as *mut ULONG) = Guest;
    return Guest;
}
#[no_mangle]
pub unsafe extern "C" fn Wow64LastErrorToGuest() {
    let Teb32: PBYTE = Wow64pCurrentTeb32();

    if Teb32.is_null() {
        return;
    }

    *(Teb32.offset(WOW64_TEB32_LASTERROR as isize) as *mut ULONG) =
        *((NtCurrentTeb() as PBYTE).offset(WOW64_TEB64_LASTERROR as isize) as *mut ULONG);
}
#[no_mangle]
pub unsafe extern "C" fn Wow64LastErrorToGuestIfChanged(WasOnEntry: ULONG) {
    let Teb32: PBYTE = Wow64pCurrentTeb32();
    let mut Host: ULONG = 0;

    if Teb32.is_null() {
        return;
    }

    Host = *((NtCurrentTeb() as PBYTE).offset(WOW64_TEB64_LASTERROR as isize) as *mut ULONG);
    if Host != WasOnEntry {
        *(Teb32.offset(WOW64_TEB32_LASTERROR as isize) as *mut ULONG) = Host;
    }
}
#[no_mangle]
pub unsafe extern "C" fn Wow64IsCurrentProcess(Process: HANDLE) -> BOOLEAN {
    static mut SelfPid: ULONG_PTR = 0; /* 0 until the first non-pseudo-handle query */
    let mut Pbi: WOW64_PROCESS_BASIC_INFORMATION = core::mem::zeroed();

    if Process == WOW64_CURRENT_PROCESS {
        return TRUE as BOOLEAN;
    }
    if Process.is_null() {
        return FALSE as BOOLEAN;
    }

    if SelfPid == 0 {
        if NtQueryInformationProcess(
            WOW64_CURRENT_PROCESS,
            0 as ULONG, /* Basic */
            core::ptr::addr_of_mut!(Pbi) as PVOID,
            core::mem::size_of::<WOW64_PROCESS_BASIC_INFORMATION>() as ULONG,
            core::ptr::null_mut::<ULONG>(),
        ) < 0 as LONG
        {
            return FALSE as BOOLEAN;
        }
        SelfPid = Pbi.UniqueProcessId;
    }
    if NtQueryInformationProcess(
        Process,
        0 as ULONG, /* Basic */
        core::ptr::addr_of_mut!(Pbi) as PVOID,
        core::mem::size_of::<WOW64_PROCESS_BASIC_INFORMATION>() as ULONG,
        core::ptr::null_mut::<ULONG>(),
    ) < 0 as LONG
    {
        return FALSE as BOOLEAN;
    }
    return (Pbi.UniqueProcessId == SelfPid) as core::ffi::c_int as BOOLEAN;
}
#[no_mangle]
pub unsafe extern "C" fn Wow64NotifyAllocBefore(Base: PVOID, Size: SIZE_T, Type: ULONG, Protect: ULONG) {
    if Wow64pBTCpuNotifyMemoryAlloc.is_some() {
        (Wow64pBTCpuNotifyMemoryAlloc.unwrap_unchecked())(Base, Size, Type, Protect, (FALSE as i32), 0);
    }
}
#[no_mangle]
pub unsafe extern "C" fn Wow64NotifyAllocAfter(Base: PVOID, Size: SIZE_T, Type: ULONG, Protect: ULONG, Status: LONG) {
    if Wow64pBTCpuNotifyMemoryAlloc.is_some() {
        (Wow64pBTCpuNotifyMemoryAlloc.unwrap_unchecked())(Base, Size, Type, Protect, (TRUE as i32), Status as ULONG);
    }
}

#[no_mangle]
pub unsafe extern "C" fn Wow64NotifyProtectBefore(Base: PVOID, Size: SIZE_T, NewProtect: ULONG) {
    if Wow64pBTCpuNotifyMemoryProtect.is_some() {
        (Wow64pBTCpuNotifyMemoryProtect.unwrap_unchecked())(Base, Size, NewProtect, (FALSE as i32), 0);
    }
}
#[no_mangle]
pub unsafe extern "C" fn Wow64NotifyProtectAfter(
    Base: PVOID,
    Size: SIZE_T,
    NewProtect: ULONG,
    Status: LONG,
) {
    if Wow64pBTCpuNotifyMemoryProtect.is_some() {
        (Wow64pBTCpuNotifyMemoryProtect.unwrap_unchecked())(Base, Size, NewProtect, (TRUE as i32), Status as ULONG);
    }
}
#[no_mangle]
pub unsafe extern "C" fn Wow64NotifyFreeBefore(Base: PVOID, Size: SIZE_T, FreeType: ULONG) {
    if Wow64pBTCpuNotifyMemoryFree.is_some() {
        (Wow64pBTCpuNotifyMemoryFree.unwrap_unchecked())(Base, Size, FreeType, (FALSE as i32), 0);
    }
}
#[no_mangle]
pub unsafe extern "C" fn Wow64NotifyFreeAfter(Base: PVOID, Size: SIZE_T, FreeType: ULONG, Status: LONG) {
    if Wow64pBTCpuNotifyMemoryFree.is_some() {
        (Wow64pBTCpuNotifyMemoryFree.unwrap_unchecked())(Base, Size, FreeType, (TRUE as i32), (Status) as u32);
    }
}
#[no_mangle]
pub unsafe extern "C" fn Wow64NotifyUnmapBefore(Base: PVOID) {
    if Wow64pBTCpuNotifyUnmapViewOfSection.is_some() {
        (Wow64pBTCpuNotifyUnmapViewOfSection.unwrap_unchecked())(Base, (FALSE as i32), 0);
    }
}
#[no_mangle]
pub unsafe extern "C" fn Wow64NotifyUnmapAfter(Base: PVOID, Status: LONG) {
    if Wow64pBTCpuNotifyUnmapViewOfSection.is_some() {
        (Wow64pBTCpuNotifyUnmapViewOfSection.unwrap_unchecked())(Base, (TRUE as i32), Status as ULONG);
    }
}
#[no_mangle]
pub unsafe extern "C" fn Wow64NotifyReadFileBefore(File: HANDLE, Buffer: PVOID, Length: SIZE_T) {
    if Wow64pBTCpuNotifyReadFile.is_some() {
        (Wow64pBTCpuNotifyReadFile.unwrap_unchecked())(File, Buffer, Length, (FALSE as i32), 0);
    }
}
#[no_mangle]
pub unsafe extern "C" fn Wow64NotifyReadFileAfter(
    File: HANDLE,
    Buffer: PVOID,
    Length: SIZE_T,
    Status: LONG,
) {
    if Wow64pBTCpuNotifyReadFile.is_some() {
        (Wow64pBTCpuNotifyReadFile.unwrap_unchecked())(File, Buffer, Length, (TRUE as i32), Status);
    }
}
#[no_mangle]
pub unsafe extern "C" fn Wow64NotifyMemoryDirty(Base: PVOID, Size: SIZE_T) {
    if !Wow64pBTCpuNotifyMemoryDirty.is_none() {
        (Wow64pBTCpuNotifyMemoryDirty.unwrap_unchecked())(Base, Size);
    }
}
#[no_mangle]
pub unsafe extern "C" fn Wow64NotifyFlushInstructionCache(Base: PVOID, Size: SIZE_T) {
    if !Wow64pBTCpuFlushInstructionCache2.is_none() {
        (Wow64pBTCpuFlushInstructionCache2.unwrap_unchecked())(Base as *const core::ffi::c_void, Size);
    }
}
#[no_mangle]
pub unsafe extern "C" fn Wow64SetExecuteFlags(Flags: ULONG) {
    Wow64pWow64Info.Wow64ExecuteFlags = Flags;
    if Wow64pBTCpuNotifyProcessExecuteFlagsChange.is_some() {
        (Wow64pBTCpuNotifyProcessExecuteFlagsChange.unwrap_unchecked())(Flags);
    }
}
#[no_mangle]
pub unsafe extern "C" fn Wow64pResolveNotifyContract(CpuDll: PVOID) {
    let mut ProcName: WOW64_ANSI_STRING = WOW64_ANSI_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut Wired: ULONG = 0;

    {
        let mut Addr: PVOID = core::ptr::null_mut();
        RtlInitAnsiString(
            (core::ptr::addr_of_mut!(ProcName) as *mut _STRING),
            b"BTCpuNotifyMemoryAlloc\0".as_ptr() as *const i8,
        );
        if LdrGetProcedureAddress(CpuDll, (core::ptr::addr_of_mut!(ProcName) as *mut _STRING), 0, core::ptr::addr_of_mut!(Addr)) >= 0
            && !Addr.is_null()
        {
            *(core::ptr::addr_of_mut!(Wow64pBTCpuNotifyMemoryAlloc) as *mut PVOID) = Addr;
            Wired += 1;
        }
    }
    {
        let mut Addr: PVOID = core::ptr::null_mut();
        RtlInitAnsiString(
            (core::ptr::addr_of_mut!(ProcName) as *mut _STRING),
            b"BTCpuNotifyMemoryProtect\0".as_ptr() as *const i8,
        );
        if LdrGetProcedureAddress(CpuDll, (core::ptr::addr_of_mut!(ProcName) as *mut _STRING), 0, core::ptr::addr_of_mut!(Addr)) >= 0
            && !Addr.is_null()
        {
            *(core::ptr::addr_of_mut!(Wow64pBTCpuNotifyMemoryProtect) as *mut PVOID) = Addr;
            Wired += 1;
        }
    }
    {
        let mut Addr: PVOID = core::ptr::null_mut();
        RtlInitAnsiString(
            (core::ptr::addr_of_mut!(ProcName) as *mut _STRING),
            b"BTCpuNotifyMemoryFree\0".as_ptr() as *const i8,
        );
        if LdrGetProcedureAddress(CpuDll, (core::ptr::addr_of_mut!(ProcName) as *mut _STRING), 0, core::ptr::addr_of_mut!(Addr)) >= 0
            && !Addr.is_null()
        {
            *(core::ptr::addr_of_mut!(Wow64pBTCpuNotifyMemoryFree) as *mut PVOID) = Addr;
            Wired += 1;
        }
    }
    {
        let mut Addr: PVOID = core::ptr::null_mut();
        RtlInitAnsiString(
            (core::ptr::addr_of_mut!(ProcName) as *mut _STRING),
            b"BTCpuNotifyMemoryDirty\0".as_ptr() as *const i8,
        );
        if LdrGetProcedureAddress(CpuDll, (core::ptr::addr_of_mut!(ProcName) as *mut _STRING), 0, core::ptr::addr_of_mut!(Addr)) >= 0
            && !Addr.is_null()
        {
            *(core::ptr::addr_of_mut!(Wow64pBTCpuNotifyMemoryDirty) as *mut PVOID) = Addr;
            Wired += 1;
        }
    }
    {
        let mut Addr: PVOID = core::ptr::null_mut();
        RtlInitAnsiString(
            (core::ptr::addr_of_mut!(ProcName) as *mut _STRING),
            b"BTCpuNotifyUnmapViewOfSection\0".as_ptr() as *const i8,
        );
        if LdrGetProcedureAddress(CpuDll, (core::ptr::addr_of_mut!(ProcName) as *mut _STRING), 0, core::ptr::addr_of_mut!(Addr)) >= 0
            && !Addr.is_null()
        {
            *(core::ptr::addr_of_mut!(Wow64pBTCpuNotifyUnmapViewOfSection) as *mut PVOID) = Addr;
            Wired += 1;
        }
    }
    {
        let mut Addr: PVOID = core::ptr::null_mut();
        RtlInitAnsiString(
            (core::ptr::addr_of_mut!(ProcName) as *mut _STRING),
            b"BTCpuNotifyReadFile\0".as_ptr() as *const i8,
        );
        if LdrGetProcedureAddress(CpuDll, (core::ptr::addr_of_mut!(ProcName) as *mut _STRING), 0, core::ptr::addr_of_mut!(Addr)) >= 0
            && !Addr.is_null()
        {
            *(core::ptr::addr_of_mut!(Wow64pBTCpuNotifyReadFile) as *mut PVOID) = Addr;
            Wired += 1;
        }
    }
    {
        let mut Addr: PVOID = core::ptr::null_mut();
        RtlInitAnsiString(
            (core::ptr::addr_of_mut!(ProcName) as *mut _STRING),
            b"BTCpuNotifyProcessExecuteFlagsChange\0".as_ptr() as *const i8,
        );
        if LdrGetProcedureAddress(CpuDll, (core::ptr::addr_of_mut!(ProcName) as *mut _STRING), 0, core::ptr::addr_of_mut!(Addr)) >= 0
            && !Addr.is_null()
        {
            *(core::ptr::addr_of_mut!(Wow64pBTCpuNotifyProcessExecuteFlagsChange) as *mut PVOID) = Addr;
            Wired += 1;
        }
    }
    {
        let mut Addr: PVOID = core::ptr::null_mut();
        RtlInitAnsiString(
            (core::ptr::addr_of_mut!(ProcName) as *mut _STRING),
            b"BTCpuFlushInstructionCache2\0".as_ptr() as *const i8,
        );
        if LdrGetProcedureAddress(CpuDll, (core::ptr::addr_of_mut!(ProcName) as *mut _STRING), 0, core::ptr::addr_of_mut!(Addr)) >= 0
            && !Addr.is_null()
        {
            *(core::ptr::addr_of_mut!(Wow64pBTCpuFlushInstructionCache2) as *mut PVOID) = Addr;
            Wired += 1;
        }
    }

    /* 0 is a legitimate answer, not a defect: a native CPU layer executes the
     * guest directly and has no translation cache to keep honest. Report the
     * count either way so a JIT layer that silently stopped exporting one of
     * these is visible on serial instead of showing up as stale guest code. */
    DbgPrint(
        b"[FEX-WOW64C] memory-notification contract: %lu/8 entry points wired (0 = native CPU layer, nothing to invalidate)\n\0"
            .as_ptr() as *const i8,
        Wired,
    );
}
#[no_mangle]
pub unsafe extern "C" fn Wow64pProcessHeap() -> PVOID {
    let mut Peb: PBYTE = *((NtCurrentTeb() as PBYTE).add(0x60 as usize) as *mut PBYTE);
    return if !Peb.is_null() {
        *(Peb.add(0x30 as usize) as *mut PVOID)
    } else {
        core::ptr::null_mut()
    };
}

unsafe extern "C" fn Wow64pDumpGuestHeap() {
    let mut Heap: ULONG = 0;
    let mut H: PULONG = core::ptr::null_mut::<ULONG>();
    if Wow64pPeb32 == 0 {
        return;
    }
    Heap = *((Wow64pPeb32 + (0x18 as ULONG_PTR)) as *mut ULONG); /* PEB32->ProcessHeap */
    DbgPrint(
        b"[FEX-WOW64C] PEB32=%p ImageBase[+0x08]=%08lx Ldr[+0x0c]=%08lx ProcParams[+0x10]=%08lx ProcessHeap[+0x18]=%08lx NumHeaps[+0x88]=%08lx\n\0"
            as *const u8 as PCSTR,
        Wow64pPeb32 as PVOID,
        *((Wow64pPeb32 + (0x08 as ULONG_PTR)) as *mut ULONG),
        *((Wow64pPeb32 + (0x0c as ULONG_PTR)) as *mut ULONG),
        *((Wow64pPeb32 + (0x10 as ULONG_PTR)) as *mut ULONG),
        Heap,
        *((Wow64pPeb32 + (0x88 as ULONG_PTR)) as *mut ULONG),
    );
    if Heap == 0 || Heap >= (0x80000000 as ULONG) {
        return;
    }
    H = (Heap as ULONG_PTR) as PULONG;
    DbgPrint(
        b"[FEX-WOW64C] HEAP hdr[0..7]: %08lx %08lx %08lx %08lx %08lx %08lx %08lx %08lx\n\0"
            as *const u8 as PCSTR,
        *H.add((0) as usize),
        *H.add((1) as usize),
        *H.add((2) as usize),
        *H.add((3) as usize),
        *H.add((4) as usize),
        *H.add((5) as usize),
        *H.add((6) as usize),
        *H.add((7) as usize),
    );
    DbgPrint(
        b"[FEX-WOW64C] HEAP Flags[0x44]=%08lx VMThresh[0x60]=%08lx TotalFreeSize[0x54]=%08lx Committed[0x58]=%08lx\n\0"
            as *const u8 as PCSTR,
        *H.add((0x44 / 4) as usize),
        *H.add((0x60 / 4) as usize),
        *H.add((0x54 / 4) as usize),
        *H.add((0x58 / 4) as usize),
    );
    DbgPrint(
        b"[FEX-WOW64C] HEAP FreeListHead[0x1cc]: Flink=%08lx Blink=%08lx (self=%08lx -> EMPTY if Flink==self)\n\0"
            as *const u8 as PCSTR,
        *H.add((0x1cc / 4) as usize),
        *H.add((0x1d0 / 4) as usize),
        Heap.wrapping_add(0x1cc as ULONG),
    );

    /* Guest x86 registers at the fault, from the CPU-area WOW64_CONTEXT (FEX
     * flushes it on the trap). Reveals which near-null store faulted and why. */
    {
        let CpuArea: PBYTE = *(Wow64pTlsSlots()).add((WOW64_TLS_CPURESERVED) as usize) as PBYTE;
        let C: *mut WOW64_CONTEXT =
            CpuArea.add((16) as usize) as *mut WOW64_CONTEXT;
        DbgPrint(
            b"[FEX-WOW64C] guest regs: eip=%08lx eax=%08lx ebx=%08lx ecx=%08lx edx=%08lx esi=%08lx edi=%08lx ebp=%08lx esp=%08lx\n\0"
                as *const u8 as PCSTR,
            (*C).Eip,
            (*C).Eax,
            (*C).Ebx,
            (*C).Ecx,
            (*C).Edx,
            (*C).Esi,
            (*C).Edi,
            (*C).Ebp,
            (*C).Esp,
        );
        if (*C).Ebp != 0 && (*C).Ebp < (0x80000000 as ULONG) {
            DbgPrint(
                b"[FEX-WOW64C] guest [ebp+0x0c]=%08lx (RtlImageNtHeader arg / image base?)\n\0"
                    as *const u8 as PCSTR,
                *((*C).Ebp.wrapping_add(0x0c as ULONG) as ULONG_PTR as *mut ULONG),
            );
        }
    }
}

unsafe extern "C" fn Wow64pLogGuestFault() -> BOOLEAN {
    let N: ULONG = InterlockedIncrement(core::ptr::addr_of_mut!(Wow64pGuestFaults) as *mut LONG) as ULONG;
    return (N <= 16 || (N & 0x3FF) == 0) as ::core::ffi::c_int as BOOLEAN;
}
#[no_mangle]
pub unsafe extern "C" fn Wow64pBuildGuestExceptionFrame(
    Rec32: *const WOW64_EXCEPTION_RECORD32,
    Ctx32: *const WOW64_CONTEXT,
    mut Below: ULONG,
) -> ULONG {
    let mut CtxA: ULONG = 0;
    let mut RecA: ULONG = 0;
    let mut Esp: ULONG = 0;
    let mut i: ULONG = 0;

    Below &= !15 as ULONG;
    if Below < 0x10000 as ULONG || Below >= 0x80000000 as ULONG {
        return 0 as ULONG;
    }
    CtxA = (Below as usize).wrapping_sub(
        (::core::mem::size_of::<WOW64_CONTEXT>() + 15 as usize)
            & !(15 as usize),
    ) as ULONG;
    RecA = (CtxA as usize).wrapping_sub(::core::mem::size_of::<WOW64_EXCEPTION_RECORD32>()) as ULONG;
    Esp = RecA.wrapping_sub(8 as ULONG);
    if Esp < 0x10000 as ULONG {
        return 0 as ULONG;
    }
    i = 0 as ULONG;
    while (i as usize) < ::core::mem::size_of::<WOW64_CONTEXT>() {
        *(CtxA.wrapping_add(i) as ULONG_PTR as PBYTE) = *(Ctx32 as *const BYTE).offset(i as isize);
        i = i.wrapping_add(1);
    }
    i = 0 as ULONG;
    while (i as usize) < ::core::mem::size_of::<WOW64_EXCEPTION_RECORD32>() {
        *(RecA.wrapping_add(i) as ULONG_PTR as PBYTE) = *(Rec32 as *const BYTE).offset(i as isize);
        i = i.wrapping_add(1);
    }
    *(Esp.wrapping_add(0 as ULONG) as ULONG_PTR as *mut ULONG) = RecA;
    *(Esp.wrapping_add(4 as ULONG) as ULONG_PTR as *mut ULONG) = CtxA;
    return Esp;
}

#[cfg(not(target_arch = "aarch64"))] // `#ifndef _M_ARM64` in the C
unsafe extern "system" fn Wow64pPrepareForExceptionHook(R: *mut EXCEPTION_RECORD, Ctx: *mut CONTEXT) {
    let mut CpuArea: PBYTE = core::ptr::null_mut::<BYTE>();
    let mut C: *mut WOW64_CONTEXT = core::ptr::null_mut::<WOW64_CONTEXT>();
    let mut Rec32: WOW64_EXCEPTION_RECORD32 = core::mem::zeroed();
    let mut Info: EXCEPTION_POINTERS = core::mem::zeroed();
    let mut Esp: ULONG = 0;
    let mut i: ULONG = 0;
    let mut N: ULONG = 0;

    if (*Ctx).SegCs as ULONG != 0x23 as ULONG || Wow64pKiUserExceptionDispatcher32.is_null() {
        return; /* not a compat-mode guest fault — let the host path handle it */
    }

    Info.ExceptionRecord = R as PEXCEPTION_RECORD;
    Info.ContextRecord = Ctx as PCONTEXT;
    if Wow64pBTCpuResetToConsistentState.is_some() {
        (Wow64pBTCpuResetToConsistentState.unwrap_unchecked())(&raw mut Info); /* flush live guest -> CPU area */
    }

    CpuArea = (*Wow64pTlsSlots().offset(WOW64_TLS_CPURESERVED as isize)) as PBYTE;
    if CpuArea.is_null() {
        /* Same hazard as Wow64pVeh: no guest state means we cannot build a
         * 32-bit exception frame, and reading C-> would fault inside the
         * handler. Let the host death path report the real fault. */
        DbgPrint(
            b"[FEX-WOW64C] guest fault with no CPU area on this thread \xE2\x80\x94 host path\n\0"
                as *const u8 as PCSTR,
        );
        return;
    }
    C = CpuArea.offset(16 as isize) as *mut WOW64_CONTEXT;

    /* DBG_PRINTEXCEPTION(_WIDE)_C from a guest OutputDebugString: consume it
     * like the kernel's KD does (guest kernel32 __except is DUMMY_PSEH), print
     * and let the host dispatcher's DUMMY_PSEH swallow-and-continue path resume
     * at the faulting Eip — do NOT redirect into guest SEH. */
    if (*R).ExceptionCode as ULONG == 0x40010006 as ULONG
        || (*R).ExceptionCode as ULONG == 0x4001000A as ULONG
    {
        return;
    }

    Rec32.ExceptionCode = (*R).ExceptionCode as ULONG;
    Rec32.ExceptionFlags = (*R).ExceptionFlags as ULONG;
    Rec32.ExceptionRecord = 0 as ULONG; /* no chain conversion */
    Rec32.ExceptionAddress = (*C).Eip as ULONG;
    N = (if (*R).NumberParameters > 15 as DWORD {
        15 as DWORD
    } else {
        (*R).NumberParameters
    }) as ULONG;
    Rec32.NumberParameters = N;
    i = 0 as ULONG;
    while i < 15 as ULONG {
        Rec32.ExceptionInformation[i as usize] = if i < N {
            (*R).ExceptionInformation[i as usize] as ULONG
        } else {
            0 as ULONG
        };
        i = i.wrapping_add(1);
    }

    /* Frame goes on the GUEST stack, below the guest's faulting Esp. */
    Esp = Wow64pBuildGuestExceptionFrame(
        &raw mut Rec32,
        C,
        ((*C).Esp as ULONG).wrapping_sub(0x10 as ULONG),
    );
    if Esp == 0 {
        DbgPrint(
            b"[FEX-WOW64C] guest fault %08lx eip=%08lx: guest-stack frame build FAILED (esp=%08lx) \xE2\x80\x94 host death path\n\0"
                as *const u8 as PCSTR,
            Rec32.ExceptionCode,
            (*C).Eip,
            (*C).Esp,
        );
        return;
    }

    if Wow64pLogGuestFault() != 0 {
        DbgPrint(
            b"[FEX-WOW64C] guest fault %08lx eip=%08lx -> 32-bit SEH dispatch (esp=%08lx) #%lu\n\0"
                as *const u8 as PCSTR,
            Rec32.ExceptionCode,
            (*C).Eip,
            Esp,
            Wow64pGuestFaults,
        );
        if Rec32.ExceptionCode == 0xc0000005 as ULONG {
            DbgPrint(
                b"[FEX-WOW64C]   AV rw=%lx addr=%08lx | eax=%08lx ebx=%08lx ecx=%08lx edx=%08lx esi=%08lx edi=%08lx ebp=%08lx\n\0"
                    as *const u8 as PCSTR,
                Rec32.ExceptionInformation[0 as usize],
                Rec32.ExceptionInformation[1 as usize],
                (*C).Eax,
                (*C).Ebx,
                (*C).Ecx,
                (*C).Edx,
                (*C).Esi,
                (*C).Edi,
                (*C).Ebp,
            );
        }
    }
    (*Ctx).Rip = Wow64pKiUserExceptionDispatcher32 as ULONG_PTR as DWORD64;
    (*Ctx).Rsp = Esp as DWORD64;
    (*Ctx).EFlags &= !(0x400 as DWORD); /* cld — fault may have been mid-rep-movs */
    NtContinue(Ctx, FALSE as BOOLEAN); /* resume in the guest dispatcher; never returns */
}
#[cfg(target_arch = "aarch64")] // `#ifdef _M_ARM64` in the C
unsafe extern "C" fn Wow64pRedirectGuestFault(Info: *mut EXCEPTION_POINTERS) -> BOOLEAN {
    let mut CpuArea: PBYTE = (*Wow64pTlsSlots().offset(WOW64_TLS_CPURESERVED as isize)) as PBYTE;
    let mut R: *mut EXCEPTION_RECORD = (*Info).ExceptionRecord as *mut EXCEPTION_RECORD;
    let mut ExAddr: ULONG_PTR = (*R).ExceptionAddress as ULONG_PTR;
    let mut C: *mut WOW64_CONTEXT = core::ptr::null_mut::<WOW64_CONTEXT>();
    let mut Rec32: WOW64_EXCEPTION_RECORD32 = core::mem::zeroed();
    let mut Esp: ULONG = 0;
    let mut i: ULONG = 0;
    let mut N: ULONG = 0;

    /* Same hazard as the breakpoint path below: a VEH runs on ANY thread, and
     * a thread with no guest state has no CPU area to read. */
    if CpuArea.is_null() || Wow64pKiUserExceptionDispatcher32.is_null() {
        return (FALSE) as BOOLEAN;
    }
    C = CpuArea.offset(16 as isize) as *mut WOW64_CONTEXT;

    /* Ours only if the CPU layer has already restated the fault in guest
     * terms. A host-side fault it declined to touch still carries a host
     * ExceptionAddress, which cannot be both below 2GB and equal to the
     * guest's Eip. */
    if ExAddr >= 0x80000000 || ExAddr != ((*C).Eip as ULONG_PTR) {
        return (FALSE) as BOOLEAN;
    }

    /* DBG_PRINTEXCEPTION_C from a guest OutputDebugString is a message, not a
     * fault — consume it exactly like the amd64 hook does. */
    if ((*R).ExceptionCode as ULONG) == 0x40010006 || ((*R).ExceptionCode as ULONG) == 0x4001000A {
        return (FALSE) as BOOLEAN;
    }

    Rec32.ExceptionCode = (*R).ExceptionCode as ULONG;
    Rec32.ExceptionFlags = (*R).ExceptionFlags as ULONG;
    Rec32.ExceptionRecord = 0; /* no chain conversion */
    Rec32.ExceptionAddress = (*C).Eip as ULONG;
    N = (if (*R).NumberParameters > 15 {
        15
    } else {
        (*R).NumberParameters
    }) as ULONG;
    Rec32.NumberParameters = N;
    i = 0;
    while i < 15 {
        Rec32.ExceptionInformation[i as usize] = if i < N {
            (*R).ExceptionInformation[i as usize] as ULONG
        } else {
            0
        };
        i = i.wrapping_add(1);
    }

    /* Frame goes on the GUEST stack, below the guest's faulting Esp. */
    Esp = Wow64pBuildGuestExceptionFrame(
        core::ptr::addr_of_mut!(Rec32),
        C,
        ((*C).Esp as ULONG).wrapping_sub(0x10),
    );
    if Esp == 0 {
        DbgPrint(
            b"[FEX-WOW64C] guest fault %08lx eip=%08lx: guest-stack frame build FAILED (esp=%08lx) \xE2\x80\x94 host death path\n\0"
                as *const u8 as PCSTR,
            Rec32.ExceptionCode,
            (*C).Eip,
            (*C).Esp,
        );
        return (FALSE) as BOOLEAN;
    }

    if Wow64pLogGuestFault() != 0 {
        DbgPrint(
            b"[FEX-WOW64C] guest fault %08lx eip=%08lx -> 32-bit SEH dispatch (esp=%08lx) #%lu\n\0"
                as *const u8 as PCSTR,
            Rec32.ExceptionCode,
            (*C).Eip,
            Esp,
            Wow64pGuestFaults,
        );
        if Rec32.ExceptionCode == 0xc0000005 {
            DbgPrint(
                b"[FEX-WOW64C]   AV rw=%lx addr=%08lx | eax=%08lx ebx=%08lx ecx=%08lx edx=%08lx esi=%08lx edi=%08lx ebp=%08lx\n\0"
                    as *const u8 as PCSTR,
                Rec32.ExceptionInformation[0 as usize],
                Rec32.ExceptionInformation[1 as usize],
                (*C).Eax,
                (*C).Ebx,
                (*C).Ecx,
                (*C).Edx,
                (*C).Esi,
                (*C).Edi,
                (*C).Ebp,
            );
        }
    }

    (*C).Eip = (Wow64pKiUserExceptionDispatcher32 as ULONG_PTR) as ULONG as DWORD;
    (*C).Esp = Esp as DWORD;
    (*C).EFlags &= !(0x400 as DWORD); /* cld — the fault may have been mid-rep-movs */

    /* Publish through the CPU layer rather than trusting the raw CPU-area
     * write: the JIT keeps its own copy of the register file and reloads it
     * from here on simulate re-entry (the same reason the NtContinue
     * marshaller goes through BTCpuSetContext). Passing the CPU area itself is
     * explicitly supported — FEX copies the input before flushing. */
    if Wow64pBTCpuSetContext.is_some() {
        (Wow64pBTCpuSetContext.unwrap_unchecked())(
            WOW64_CURRENT_THREAD,
            WOW64_CURRENT_PROCESS,
            core::ptr::null_mut(),
            (C).cast(),
        );
    }
    return (TRUE) as BOOLEAN;
}

unsafe extern "system" fn Wow64pVeh(Info: *mut EXCEPTION_POINTERS) -> LONG {
    if Wow64pBTCpuResetToConsistentState.is_some() {
        (Wow64pBTCpuResetToConsistentState.unwrap_unchecked())(Info); /* NtContinues if handled */
    }

    /* Guest DbgPrint forwarding: ntdll32's DebugService does `int 2d` with
     * eax=1 (BREAKPOINT_PRINT), ecx=buffer, edx=length; FEX surfaces it as a
     * breakpoint exception. Print the guest's message to serial. */
    if (*(*Info).ExceptionRecord).ExceptionCode == ((0x80000003u32 as LONG)) as u32 {
        let mut CpuArea: PBYTE =
            (*Wow64pTlsSlots().offset(WOW64_TLS_CPURESERVED as isize)) as PBYTE;
        let mut C: *mut WOW64_CONTEXT =
            CpuArea.offset(16 as isize) as *mut WOW64_CONTEXT;
        /* A VEH runs on ANY thread, including one with no guest state. Decline. */
        if CpuArea.is_null() {
            DbgPrint(
                b"[FEX-WOW64C] VEH: breakpoint with no CPU area on this thread \xE2\x80\x94 declining\n\0"
                    .as_ptr() as PCSTR,
            );
            return EXCEPTION_CONTINUE_SEARCH as LONG;
        }
        if (*C).Eax == 1
            && (*C).Ecx != 0
            && (*C).Ecx < 0x80000000
            && (*C).Edx < 512
        {
            DbgPrint(
                b"[GUEST32] %.*s\0".as_ptr() as PCSTR,
                (*C).Edx as i32,
                (*C).Ecx as ULONG_PTR as *const i8,
            );
        }
        /* A DebugService `int 2d` -- PRINT, PROMPT, LOAD/UNLOAD_SYMBOLS, COMMAND_STRING (1..5):
         * complete every one and resume the caller. */
        if (*(*Info).ExceptionRecord).NumberParameters >= 1
            && (*(*Info).ExceptionRecord).ExceptionInformation[0 as usize]
                >= 1
            && (*(*Info).ExceptionRecord).ExceptionInformation[0 as usize]
                <= 5
            && ((*(*Info).ExceptionRecord).ExceptionAddress as ULONG_PTR)
                == ((*C).Eip as ULONG_PTR)
            && Wow64pBTCpuSetContext.is_some()
        {
            (*C).Eax = 0; /* STATUS_SUCCESS */
            (Wow64pBTCpuSetContext.unwrap_unchecked())(
                WOW64_CURRENT_THREAD,
                WOW64_CURRENT_PROCESS,
                core::ptr::null_mut(),
                (C).cast(),
            );
            return EXCEPTION_CONTINUE_EXECUTION as LONG;
        }
        /* Breakpoints keep the pre-E18 behaviour deliberately. */
        return EXCEPTION_CONTINUE_SEARCH as LONG;
    }

    /* E18: hand a guest fault to the guest's 32-bit SEH. */
    #[cfg(target_arch = "aarch64")]
    if Wow64pRedirectGuestFault(Info) != 0 {
        return EXCEPTION_CONTINUE_EXECUTION as LONG;
    }

    DbgPrint(
        b"[FEX-WOW64C] VEH: unhandled code=%08lx addr=%p pc=%p\n\0".as_ptr() as PCSTR,
        (*(*Info).ExceptionRecord).ExceptionCode,
        (*(*Info).ExceptionRecord).ExceptionAddress,
        WOW64_HOST_PC!((*Info).ContextRecord),
    );
    Wow64pDumpGuestHeap();
    return EXCEPTION_CONTINUE_SEARCH as LONG;
}

unsafe extern "C" fn Wow64pAllocBelow2G(mut Size: SIZE_T, mut Protect: ULONG, What: PCSTR) -> PVOID {
    let mut Base: PVOID = core::ptr::null_mut();
    let mut Status: LONG = NtAllocateVirtualMemory(WOW64_CURRENT_PROCESS, core::ptr::addr_of_mut!(Base),
                                          (WOW64_ZEROBITS_BELOW_2G) as ULONG_PTR, core::ptr::addr_of_mut!(Size),
                                          (MEM_RESERVE | MEM_COMMIT) as ULONG, Protect);
    if Status < 0 || (Base as ULONG_PTR) >= 0x80000000
    {
        DbgPrint(b"[FEX-WOW64C] %s: below-2GB alloc FAILED (status=%08lx base=%p)\n\0".as_ptr() as PCSTR,
                 What, Status, Base);
        return core::ptr::null_mut();
    }
    return Base;
}

unsafe extern "C" fn Wow64pMapNtdll32() -> PVOID {
    static mut PathBuf: [WCHAR; 280] = [0; 280];
    let mut SystemRoot: *const WCHAR = (0x7FFE0000 as PBYTE)
        .offset(0x30 as isize)
        as *const WCHAR;
    let mut P: *mut WCHAR = core::ptr::addr_of_mut!(PathBuf) as *mut WCHAR;
    let mut S: *const WCHAR = core::ptr::null::<WCHAR>();
    let mut Path: WOW64_UNICODE_STRING = core::mem::zeroed();
    let mut Oa: WOW64_OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut Iosb: WOW64_IO_STATUS_BLOCK = core::mem::zeroed();
    let mut File: HANDLE = core::ptr::null_mut();
    let mut Section: HANDLE = core::ptr::null_mut();
    let mut Base: PVOID = core::ptr::null_mut();
    let mut ViewSize: SIZE_T = 0 as SIZE_T;
    let mut Status: LONG;

    S = RUSTCONV_WSTR_C554B73F8939.as_ptr();
    while *S != 0 {
        let fresh23 = S;
        S = S.offset(1);
        let fresh24 = P;
        P = P.offset(1);
        *fresh24 = *fresh23;
    }
    S = SystemRoot;
    while *S != 0
        && ((P.offset_from(core::ptr::addr_of_mut!(PathBuf) as *mut WCHAR)) as isize)
            < (260 as isize)
    {
        let fresh25 = S;
        S = S.offset(1);
        let fresh26 = P;
        P = P.offset(1);
        *fresh26 = *fresh25;
    }
    S = RUSTCONV_WSTR_A78DD113F020.as_ptr();
    while *S != 0 {
        let fresh27 = S;
        S = S.offset(1);
        let fresh28 = P;
        P = P.offset(1);
        *fresh28 = *fresh27;
    }
    *P = 0 as WCHAR;
    RtlInitUnicodeString(
        (core::ptr::addr_of_mut!(Path) as *mut _UNICODE_STRING),
        core::ptr::addr_of_mut!(PathBuf) as *mut WCHAR as PCWSTR,
    );

    Oa.Length = core::mem::size_of::<WOW64_OBJECT_ATTRIBUTES>() as ULONG;
    Oa.RootDirectory = core::ptr::null_mut();
    Oa.ObjectName = core::ptr::addr_of_mut!(Path);
    Oa.Attributes = 0x40 as ULONG; /* OBJ_CASE_INSENSITIVE */
    Oa.SecurityDescriptor = core::ptr::null_mut();
    Oa.SecurityQualityOfService = core::ptr::null_mut();

    Status = NtOpenFile(
        core::ptr::addr_of_mut!(File),
        GENERIC_READ as ACCESS_MASK
            | GENERIC_EXECUTE as ACCESS_MASK
            | SYNCHRONIZE as ACCESS_MASK,
        ((core::ptr::addr_of_mut!(Oa)) as *mut _OBJECT_ATTRIBUTES),
        ((core::ptr::addr_of_mut!(Iosb)) as *mut _IO_STATUS_BLOCK),
        FILE_SHARE_READ as ULONG,
        0x60 as ULONG, /* FILE_NON_DIRECTORY_FILE|FILE_SYNCHRONOUS_IO_NONALERT */
    );
    if Status < (0 as LONG)
    {
        DbgPrint(
            b"[FEX-WOW64C] NtOpenFile(%S) FAILED %08lx\n\0" as *const u8 as PCSTR,
            core::ptr::addr_of_mut!(PathBuf) as *mut WCHAR,
            Status,
        );
        return core::ptr::null_mut();
    }

    Status = NtCreateSection(
        core::ptr::addr_of_mut!(Section),
        (SECTION_MAP_READ as ACCESS_MASK)
            | (SECTION_MAP_EXECUTE as ACCESS_MASK)
            | (SECTION_QUERY as ACCESS_MASK),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        PAGE_EXECUTE as ULONG,
        SEC_IMAGE as ULONG,
        File,
    );
    NtClose(File);
    if Status < (0 as LONG)
    {
        DbgPrint(
            b"[FEX-WOW64C] NtCreateSection(ntdll32) FAILED %08lx\n\0" as *const u8 as PCSTR,
            Status,
        );
        return core::ptr::null_mut();
    }

    Status = NtMapViewOfSection(
        Section,
        WOW64_CURRENT_PROCESS,
        core::ptr::addr_of_mut!(Base),
        0 as ULONG_PTR,
        0 as SIZE_T,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(ViewSize),
        1 as ULONG, /* ViewShare */
        0 as ULONG,
        PAGE_EXECUTE as ULONG,
    );
    NtClose(Section);
    if Status < (0 as LONG)
    {
        DbgPrint(
            b"[FEX-WOW64C] NtMapViewOfSection(ntdll32) FAILED %08lx\n\0" as *const u8 as PCSTR,
            Status,
        );
        return core::ptr::null_mut();
    }
    Wow64pNtdll32Size = ViewSize;
    DbgPrint(
        b"[FEX-WOW64C] ntdll32 mapped at %p size=%zx\n\0" as *const u8 as PCSTR,
        Base,
        ViewSize,
    );
    return Base;
}

unsafe extern "C" fn Wow64pFindExport32(
    Base: PVOID,
    ImgSize: SIZE_T,
    Name: *const ::core::ffi::c_char,
) -> PVOID {
    let mut B: PBYTE = Base as PBYTE;
    let mut Nt: PIMAGE_NT_HEADERS32 =
        B.offset((*(B as PIMAGE_DOS_HEADER)).e_lfanew as isize) as PIMAGE_NT_HEADERS32;
    let mut ExpRva: ULONG = (*Nt).OptionalHeader.DataDirectory[0].VirtualAddress as ULONG;
    let mut Exp: PIMAGE_EXPORT_DIRECTORY = ::core::ptr::null_mut();
    let mut NameRvas: PULONG = ::core::ptr::null_mut();
    let mut FuncRvas: PULONG = ::core::ptr::null_mut();
    let mut Ordinals: PUSHORT = ::core::ptr::null_mut();
    let mut i: ULONG;

    /* Every RVA below is range checked before it is dereferenced. */
    if ExpRva == 0
        || (ExpRva as usize) > (ImgSize as usize)
        || (ImgSize as usize) < ::core::mem::size_of::<_IMAGE_EXPORT_DIRECTORY>()
        || (ExpRva as usize)
            > (ImgSize as usize).wrapping_sub(::core::mem::size_of::<_IMAGE_EXPORT_DIRECTORY>())
    {
        return NULL;
    }
    Exp = B.offset(ExpRva as isize) as PIMAGE_EXPORT_DIRECTORY;

    if ((*Exp).NumberOfNames as usize)
        > (ImgSize as usize).wrapping_div(::core::mem::size_of::<ULONG>())
    {
        return NULL;
    }
    if ((*Exp).AddressOfNames as usize) > (ImgSize as usize)
        || ((*Exp).AddressOfNameOrdinals as usize) > (ImgSize as usize)
        || ((*Exp).AddressOfFunctions as usize) > (ImgSize as usize)
    {
        return NULL;
    }
    if ((*Exp).NumberOfNames as usize).wrapping_mul(::core::mem::size_of::<ULONG>())
        > (ImgSize as usize).wrapping_sub((*Exp).AddressOfNames as usize)
        || ((*Exp).NumberOfNames as usize).wrapping_mul(::core::mem::size_of::<USHORT>())
            > (ImgSize as usize).wrapping_sub((*Exp).AddressOfNameOrdinals as usize)
    {
        return NULL;
    }

    NameRvas = B.offset((*Exp).AddressOfNames as isize) as PULONG;
    Ordinals = B.offset((*Exp).AddressOfNameOrdinals as isize) as PUSHORT;
    FuncRvas = B.offset((*Exp).AddressOfFunctions as isize) as PULONG;
    i = 0;
    while i < (*Exp).NumberOfNames {
        let mut N: *const ::core::ffi::c_char = ::core::ptr::null();
        let mut M: *const ::core::ffi::c_char = Name;
        let mut Left: SIZE_T;

        if (*NameRvas.offset(i as isize) as usize) >= (ImgSize as usize) {
            return NULL;
        }
        N = B.offset(*NameRvas.offset(i as isize) as isize) as *const ::core::ffi::c_char;
        Left = ((ImgSize as usize).wrapping_sub(*NameRvas.offset(i as isize) as usize)) as _;
        /* Bounded comparison: stop at the end of the view. */
        while Left != 0
            && *N as ::core::ffi::c_int != 0
            && *N as ::core::ffi::c_int == *M as ::core::ffi::c_int
        {
            N = N.offset(1);
            M = M.offset(1);
            Left = Left.wrapping_sub(1);
        }
        if Left != 0 && *N as ::core::ffi::c_int == *M as ::core::ffi::c_int {
            let mut Ord: ULONG = *Ordinals.offset(i as isize) as ULONG;
            if Ord >= (*Exp).NumberOfFunctions
                || ((*Exp).AddressOfFunctions as usize).wrapping_add(
                    (Ord as usize)
                        .wrapping_add(1)
                        .wrapping_mul(::core::mem::size_of::<ULONG>()),
                ) > (ImgSize as usize)
            {
                return NULL;
            }
            if (*FuncRvas.offset(Ord as isize) as usize) >= (ImgSize as usize) {
                return NULL;
            }
            return B.offset(*FuncRvas.offset(Ord as isize) as isize) as PVOID;
        }
        i = i.wrapping_add(1);
    }
    return NULL;
}
#[no_mangle]
pub unsafe extern "C" fn Wow64pPatchTransitionSlot(Base: PVOID, Size: SIZE_T) {
    let mut B: PBYTE = Base as PBYTE;
    let mut Nt: PIMAGE_NT_HEADERS32 = core::ptr::null_mut();
    let mut Slot: PULONG = core::ptr::null_mut();
    let mut ImgSize: SIZE_T = 0;

    if Wow64pBopCode.is_null() || Base.is_null() || Size < 0x1000 {
        return;
    }
    if (*(B as PIMAGE_DOS_HEADER)).e_magic != (IMAGE_DOS_SIGNATURE as u16) {
        return;
    }
    Nt = B.offset((*(B as PIMAGE_DOS_HEADER)).e_lfanew as isize) as PIMAGE_NT_HEADERS32;
    if (Nt as usize).wrapping_sub(B as usize)
        > (Size.wrapping_sub((core::mem::size_of::<_IMAGE_NT_HEADERS32>() as usize) as _)) as _
        || (*Nt).Signature != IMAGE_NT_SIGNATURE
    {
        return;
    }

    /* RVA arithmetic is only meaningful for an IMAGE-mapped section; a data
     * view (ViewSize != SizeOfImage page-rounded) is declined here. */
    ImgSize = (*Nt).OptionalHeader.SizeOfImage as SIZE_T;
    if ImgSize < 0x1000
        || (ImgSize.wrapping_add(0xFFF) & !(0xFFF as SIZE_T))
            != (Size.wrapping_add(0xFFF) & !(0xFFF as SIZE_T))
    {
        return;
    }
    if ImgSize > Size {
        return;
    }

    Slot = Wow64pFindExport32(
        Base,
        ImgSize,
        b"Wow64Transition\0".as_ptr() as *const i8,
    ) as PULONG;
    if !Slot.is_null() {
        *Slot = Wow64pBopCode as ULONG_PTR as ULONG;
        DbgPrint(
            b"[FEX-WOW64C] Wow64Transition slot %p (module %p) patched -> BOP %p\n\0".as_ptr()
                as PCSTR,
            Slot,
            Base,
            Wow64pBopCode,
        );
    }

    #[cfg(target_arch = "x86_64")]
    {
        /* win32k client stubs dispatch through _KiIndirectSystemCall; rewrite
         * the dispatcher in place to jump to this process's BOP. amd64-only. */
        if !Wow64pBopCode.is_null() {
            static mut PAT: [u8; 7] = [
                0xBA,
                0x00,
                0x03,
                0xFE,
                0x7F,
                0xFF,
                0x22,
            ];
            /* ImgSize, not Size: the authoritative extent is the PE header's. */
            let Scan: SIZE_T = if ImgSize > 0x400000 {
                0x400000
            } else {
                ImgSize
            };
            let End: PBYTE = B.wrapping_add((Scan.wrapping_sub((7 as usize) as _)) as _);
            let mut P: PBYTE = B;
            while P <= End {
                let mut k: ULONG = 0;
                while (k as usize) < (7 as usize) && *P.add(k as usize) == PAT[k as usize] {
                    k += 1;
                }
                if (k as usize) == (7 as usize) {
                    {
                        let mut Pg: PVOID = P as PVOID;
                        let mut Sz: SIZE_T = (7 as usize) as _;
                        let mut Old: ULONG = 0;
                        NtProtectVirtualMemory(
                            WOW64_CURRENT_PROCESS,
                            core::ptr::addr_of_mut!(Pg),
                            core::ptr::addr_of_mut!(Sz),
                            PAGE_EXECUTE_READWRITE,
                            core::ptr::addr_of_mut!(Old),
                        );
                        /* mov edx, BOP */
                        *(P.add(1 as usize) as *mut ULONG) =
                            Wow64pBopCode as ULONG_PTR as ULONG;
                        /* jmp edx */
                        *P.add(5 as usize) = 0xFF;
                        *P.add(6 as usize) = 0xE2;
                        NtProtectVirtualMemory(
                            WOW64_CURRENT_PROCESS,
                            core::ptr::addr_of_mut!(Pg),
                            core::ptr::addr_of_mut!(Sz),
                            Old,
                            core::ptr::addr_of_mut!(Old),
                        );
                        NtFlushInstructionCache(WOW64_CURRENT_PROCESS, (P).cast(), (7) as SIZE_T);
                        DbgPrint(
                            b"[FEX-WOW64C] win32k KiIndirectSystemCall @ %p (module %p) -> BOP %p\n\0"
                                .as_ptr() as PCSTR,
                            P,
                            Base,
                            Wow64pBopCode,
                        );
                    }
                    break; /* one dispatcher per module */
                }
                P = P.add(1);
            }
        }
    }
}

unsafe extern "C" fn Wow64pCopyUstr32(Dst32: *mut WOW64_UNICODE_STRING32, Src: PBYTE, Arena: *mut PBYTE) {
    let Length: USHORT = *(Src.add(0 as usize) as *mut USHORT);
    let Buffer: PWSTR = *(Src.add(8 as usize) as *mut PWSTR); /* +8: after Length/Max/pad in UNICODE_STRING64 */
    (*Dst32).Length = Length;
    if Length == 0 || Buffer.is_null() {
        (*Dst32).MaximumLength = (if Length != 0 {
            (Length as usize).wrapping_add(core::mem::size_of::<WCHAR>() as usize) as USHORT as core::ffi::c_int
        } else {
            0 as core::ffi::c_int
        }) as USHORT;
        (*Dst32).Buffer = 0 as ULONG;
        return;
    }
    (*Dst32).MaximumLength =
        (Length as usize).wrapping_add(core::mem::size_of::<WCHAR>() as usize) as USHORT;
    (*Dst32).Buffer = *Arena as ULONG_PTR as ULONG;
    let mut i: USHORT = 0 as USHORT;
    while (i as core::ffi::c_int) < Length as core::ffi::c_int {
        *(*Arena).add(i as usize) = *(Buffer as PBYTE).add(i as usize);
        i = i.wrapping_add(1);
    }
    *(*Arena).add(Length as usize) = 0 as BYTE;
    *(*Arena).add(Length as usize + 1 as usize) = 0 as BYTE;
    *Arena = (*Arena).add(
        ((Length as usize)
            .wrapping_add(core::mem::size_of::<WCHAR>() as usize)
            .wrapping_add(3 as usize)
            & !(3 as usize)),
    );
}

unsafe extern "C" fn Wow64pCopyEnvironment32(Upp32: *mut WOW64_RTL_USER_PROCESS_PARAMETERS32, Upp64: PBYTE) {
    let mut Env64: PWSTR = *(Upp64.add(UPP64_ENVIRONMENT_OFF) as *mut PWSTR);
    let mut EnvBytes: SIZE_T = 0;
    let mut i: SIZE_T = 0;
    let mut Env32: PBYTE = core::ptr::null_mut();

    if Env64.is_null() {
        DbgPrint(
            b"[FEX-WOW64C] WARN: PEB64 params carry no Environment \xE2\x80\x94 guest gets an empty (but terminated) block\n\0"
                as *const u8 as PCSTR,
        );
        EnvBytes = (2 as usize).wrapping_mul(core::mem::size_of::<WCHAR>() as usize) as SIZE_T;
    } else {
        /* Find the double-NUL that ends the block (inclusive). */
        let MaxChars: SIZE_T = (WOW64_MAX_ENVIRONMENT_BYTES as SIZE_T).wrapping_div(core::mem::size_of::<WCHAR>() as SIZE_T);
        let mut n: SIZE_T = 0 as SIZE_T;
        while n.wrapping_add(1 as SIZE_T) < MaxChars
            && !(*Env64.add(n as usize) == 0
                && *Env64.add(n.wrapping_add(1 as SIZE_T) as usize) == 0)
        {
            n = n.wrapping_add(1);
        }
        if n.wrapping_add(1 as SIZE_T) >= MaxChars {
            DbgPrint(
                b"[FEX-WOW64C] WARN: Environment not double-NUL terminated within %u bytes \xE2\x80\x94 guest gets an empty block\n\0"
                    as *const u8 as PCSTR,
                WOW64_MAX_ENVIRONMENT_BYTES,
            );
            Env64 = core::ptr::null_mut();
            EnvBytes = (2 as usize).wrapping_mul(core::mem::size_of::<WCHAR>() as usize) as SIZE_T;
        } else {
            EnvBytes = (n as usize)
                .wrapping_add(2 as usize)
                .wrapping_mul(core::mem::size_of::<WCHAR>() as usize) as SIZE_T;
        }
    }

    Env32 = Wow64pAllocBelow2G(EnvBytes, PAGE_READWRITE, b"environment32\0" as *const u8 as *const core::ffi::c_char) as PBYTE;
    if Env32.is_null() {
        /* Leave Environment NULL rather than pointing at nothing: the guest's
         * first env read would AV either way, but a loud line beats a mystery. */
        DbgPrint(
            b"[FEX-WOW64C] WARN: environment32 alloc failed \xE2\x80\x94 guest env is NULL\n\0"
                as *const u8 as PCSTR,
        );
        return;
    }
    if !Env64.is_null() {
        i = 0 as SIZE_T;
        while i < EnvBytes {
            *Env32.add(i as usize) = *(Env64 as PBYTE).add(i as usize);
            i = i.wrapping_add(1);
        }
    }
    /* else: the alloc is already zero-filled — an empty, terminated block. */

    (*Upp32).Environment = Env32 as ULONG_PTR as ULONG;
    DbgPrint(
        b"[FEX-WOW64C] environment32 at %p (%u bytes)\n\0" as *const u8 as PCSTR,
        Env32,
        EnvBytes as ULONG,
    );
}

unsafe extern "C" fn Wow64pBuildProcessParams32(mut Peb32: ULONG_PTR) {
    let mut Peb64: PBYTE = (NtCurrentTeb() as PBYTE).offset(0 as isize);
    let mut Upp64: PBYTE = core::ptr::null_mut::<BYTE>();
    let mut Upp32: *mut WOW64_RTL_USER_PROCESS_PARAMETERS32 =
        core::ptr::null_mut::<WOW64_RTL_USER_PROCESS_PARAMETERS32>();
    let mut Arena: PBYTE = core::ptr::null_mut::<BYTE>();
    let mut Block: PBYTE = core::ptr::null_mut::<BYTE>();
    let mut Size: SIZE_T = 0x2000 as SIZE_T;

    Peb64 = *((NtCurrentTeb() as PBYTE).offset(0x60 as isize) as *mut PBYTE);
    Upp64 = *(Peb64.offset(PEB64_PROCESS_PARAMETERS_OFF as isize) as *mut PBYTE);
    if Upp64.is_null() {
        DbgPrint(
            b"[FEX-WOW64C] WARN: PEB64->ProcessParameters NULL \xE2\x80\x94 32-bit params left empty\n\0"
                as *const u8 as PCSTR,
        );
        return;
    }

    Block = Wow64pAllocBelow2G(
        Size,
        PAGE_READWRITE as ULONG,
        b"proc params32\0" as *const u8 as *const ::core::ffi::c_char,
    ) as PBYTE;
    if Block.is_null() {
        NtTerminateProcess(WOW64_CURRENT_PROCESS, 0xBAD as LONG);
    }
    Upp32 = Block as *mut WOW64_RTL_USER_PROCESS_PARAMETERS32;
    Arena = Block.offset(core::mem::size_of::<WOW64_RTL_USER_PROCESS_PARAMETERS32>() as usize as isize);
    Arena = ((Arena as ULONG_PTR).wrapping_add(3 as ULONG_PTR)
        & !(3 as ULONG_PTR)) as PBYTE;

    (*Upp32).MaximumLength = Size as ULONG;
    (*Upp32).Length = Size as ULONG;
    (*Upp32).Flags = 0x6001 as ULONG;
    (*Upp32).ConsoleHandle =
        *(Upp64.offset(UPP64_CONSOLEHANDLE_OFF as isize) as *mut ULONGLONG) as ULONG;
    (*Upp32).ConsoleFlags = *(Upp64.offset(UPP64_CONSOLEFLAGS_OFF as isize) as *mut ULONG);
    (*Upp32).StandardInput =
        *(Upp64.offset(UPP64_STDINPUT_OFF as isize) as *mut ULONGLONG) as ULONG;
    (*Upp32).StandardOutput =
        *(Upp64.offset(UPP64_STDOUTPUT_OFF as isize) as *mut ULONGLONG) as ULONG;
    (*Upp32).StandardError =
        *(Upp64.offset(UPP64_STDERROR_OFF as isize) as *mut ULONGLONG) as ULONG;

    {
        let mut i: ULONG = 0;
        let Src: *const ULONG = Upp64.offset(UPP64_STARTINGX_OFF as isize) as *const ULONG;
        let Dst: *mut ULONG = core::ptr::addr_of_mut!((*Upp32).StartingX);
        i = 0 as ULONG;
        while i
            <= ((UPP64_SHOWWINDOWFLAGS_OFF - UPP64_STARTINGX_OFF) / 4) as ULONG
        {
            *Dst.offset(i as isize) = *Src.offset(i as isize);
            i = i.wrapping_add(1);
        }
    }

    Wow64pCopyUstr32(
        core::ptr::addr_of_mut!((*Upp32).CurrentDirectoryPath),
        Upp64.offset(UPP64_CURDIR_OFF as isize),
        core::ptr::addr_of_mut!(Arena),
    );
    (*Upp32).CurrentDirectoryHandle =
        *(Upp64.offset(UPP64_CURDIRHANDLE_OFF as isize) as *mut ULONGLONG) as ULONG;
    Wow64pCopyUstr32(
        core::ptr::addr_of_mut!((*Upp32).DllPath),
        Upp64.offset(UPP64_DLLPATH_OFF as isize),
        core::ptr::addr_of_mut!(Arena),
    );
    Wow64pCopyUstr32(
        core::ptr::addr_of_mut!((*Upp32).ImagePathName),
        Upp64.offset(UPP64_IMAGEPATHNAME_OFF as isize),
        core::ptr::addr_of_mut!(Arena),
    );
    Wow64pCopyUstr32(
        core::ptr::addr_of_mut!((*Upp32).CommandLine),
        Upp64.offset(UPP64_COMMANDLINE_OFF as isize),
        core::ptr::addr_of_mut!(Arena),
    );
    Wow64pCopyUstr32(
        core::ptr::addr_of_mut!((*Upp32).WindowTitle),
        Upp64.offset(UPP64_WINDOWTITLE_OFF as isize),
        core::ptr::addr_of_mut!(Arena),
    );
    Wow64pCopyUstr32(
        core::ptr::addr_of_mut!((*Upp32).DesktopInfo),
        Upp64.offset(UPP64_DESKTOPINFO_OFF as isize),
        core::ptr::addr_of_mut!(Arena),
    );
    Wow64pCopyEnvironment32(Upp32, Upp64);

    if (*Upp32).DllPath.Length as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
        static DefPath: [u16; 40] = [
    0x0058u16, 0x003A, 0x005C, 0x0072, 0x0065, 0x0061, 0x0063, 0x0074, 0x006F, 0x0073,
    0x005C, 0x0073, 0x0079, 0x0073, 0x0077, 0x006F, 0x0077, 0x0036, 0x0034, 0x003B,
    0x0058, 0x003A, 0x005C, 0x0072, 0x0065, 0x0061, 0x0063, 0x0074, 0x006F, 0x0073,
    0x005C, 0x0073, 0x0079, 0x0073, 0x0074, 0x0065, 0x006D, 0x0033, 0x0032, 0x0000,
];
        let Len: USHORT = (core::mem::size_of::<[u16; 40]>() - core::mem::size_of::<u16>()) as USHORT;
        (*Upp32).DllPath.Length = Len;
        (*Upp32).DllPath.MaximumLength =
            (Len as usize).wrapping_add(core::mem::size_of::<u16>()) as USHORT;
        (*Upp32).DllPath.Buffer = Arena as ULONG_PTR as ULONG;
        let mut i_0: USHORT = 0 as USHORT;
        while (i_0 as usize) < core::mem::size_of::<[u16; 40]>() {
            *Arena.offset(i_0 as isize) =
                *(core::ptr::addr_of!(DefPath) as *const u8).offset(i_0 as isize);
            i_0 = i_0.wrapping_add(1);
        }
        Arena = Arena.offset(
            ((core::mem::size_of::<[u16; 40]>() + 3 as usize)
                & !(3 as usize)) as isize,
        );
    }

    *((Peb32 as PBYTE).offset(PEB32_PROCESS_PARAMETERS_OFF as isize) as *mut ULONG) =
        Upp32 as ULONG_PTR as ULONG;
    DbgPrint(
        b"[FEX-WOW64C] PEB32->ProcessParameters built at %p (image='%.*S')\n\0" as *const u8
            as PCSTR,
        Upp32,
        (*Upp32).ImagePathName.Length as ::core::ffi::c_int / 2 as ::core::ffi::c_int,
        (*Upp32).ImagePathName.Buffer as ULONG_PTR as PWSTR,
    );
}

#[no_mangle]
pub unsafe extern "system" fn Wow64SystemServiceEx(ServiceNumber: ULONG, Arguments: PULONG) -> ULONG {
    match ServiceNumber {
        WOW64B_SVC_ECHO => {
            DbgPrint(
                b"[FEX-WOW64B] ECHO service: args %p = %08lx %08lx %08lx\n\0" as *const u8 as PCSTR,
                Arguments,
                *Arguments.offset(0 as isize),
                *Arguments.offset(1 as isize),
                *Arguments.offset(2 as isize),
            );
            return WOW64B_ECHO_MAGIC;
        }
        WOW64B_SVC_VERDICT => {
            if *Arguments.offset(0 as isize) == WOW64B_ECHO_MAGIC {
                DbgPrint(
                    b"[FEX-WOW64B] guest syscall round-trip PASS (echo magic returned by guest)\n\0" as *const u8 as PCSTR,
                );
                NtTerminateProcess(WOW64_CURRENT_PROCESS, 0x600D);
            }
            DbgPrint(
                b"[FEX-WOW64B] guest syscall round-trip FAIL (guest passed %08lx, want %08x)\n\0" as *const u8 as PCSTR,
                *Arguments.offset(0 as isize),
                WOW64B_ECHO_MAGIC,
            );
            NtTerminateProcess(WOW64_CURRENT_PROCESS, 0xBAD);
            return 0; /* unreachable */
        }
        WOW64C_SVC_EXIT => {
            DbgPrint(
                b"[FEX-WOW64C] guest entry returned %08lx (want %08x), NtClose(NULL) via ntdll32 stub => %08lx (want %08x)\n\0" as *const u8 as PCSTR,
                *Arguments.offset(1 as isize),
                WOW64C_HELLO32_SENTINEL,
                *Arguments.offset(0 as isize),
                WOW64C_STATUS_INVALID_HANDLE,
            );
            if *Arguments.offset(1 as isize) == WOW64C_HELLO32_SENTINEL
                && *Arguments.offset(0 as isize) == WOW64C_STATUS_INVALID_HANDLE
            {
                DbgPrint(
                    b"[FEX-WOW64C] M2 PASS: real i386 image code executed under FEX in a WoW64 process\n\0" as *const u8 as PCSTR,
                );
                DbgPrint(
                    b"[FEX-WOW64C] M3a PASS: real ntdll32 syscall stub round-tripped Wow64Transition -> BOP -> thunk table\n\0" as *const u8 as PCSTR,
                );
                DbgPrint(
                    b"[FEX-WOW64C] M3b PASS: guest ntdll32 LdrInitializeThunk ran the 32-bit loader, then handed off to the image entry\n\0" as *const u8 as PCSTR,
                );
                NtTerminateProcess(WOW64_CURRENT_PROCESS, 0x600D);
            }
            NtTerminateProcess(WOW64_CURRENT_PROCESS, 0xBAD);
            return 0; /* unreachable */
        }
        _ => {
            /* Phase D: generated thunk table (wow64_thunks.c), indexed by the
             * real service number the guest's ntdll32 stub put in EAX. */
            return Wow64DispatchSystemService(ServiceNumber, Arguments) as ULONG;
        }
    }
}

unsafe extern "C" fn Wow64pInstallCpuArea() -> *mut WOW64_CONTEXT {
    let mut Buf: PBYTE = core::ptr::null_mut::<BYTE>();
    let mut CpuArea: PBYTE = core::ptr::null_mut::<BYTE>();
    Buf = RtlAllocateHeap(
        Wow64pProcessHeap(),
        WOW64_HEAP_ZERO_MEMORY as ULONG,
        (core::mem::size_of::<PVOID>() as SIZE_T)
            .wrapping_add(16 as SIZE_T)
            .wrapping_add(16 as SIZE_T)
            .wrapping_add(core::mem::size_of::<WOW64_CONTEXT>() as SIZE_T),
    ) as PBYTE;
    if Buf.is_null() {
        DbgPrint(
            b"[FEX-WOW64C] FAIL: per-thread CPU area allocation failed\n\0".as_ptr() as PCSTR,
        );
        return core::ptr::null_mut::<WOW64_CONTEXT>();
    }
    CpuArea = (((Buf as usize)
        .wrapping_add(core::mem::size_of::<PVOID>() as usize)
        .wrapping_add(15 as usize))
        & !(15 as usize)) as PBYTE;
    *(CpuArea as *mut PVOID).offset(-(1 as isize)) = Buf as PVOID;
    *(CpuArea.offset(2 as isize) as *mut USHORT) =
        WOW64_MACHINE_I386 as USHORT; /* WOW64_CPURESERVED.Machine */
    *Wow64pTlsSlots().offset(WOW64_TLS_CPURESERVED as isize) = CpuArea as PVOID;
    return CpuArea.offset(16 as isize) as *mut WOW64_CONTEXT;
}
#[no_mangle]
pub unsafe extern "C" fn Wow64FreeThreadCpuArea() {
    let CpuArea: PBYTE = *Wow64pTlsSlots().offset(WOW64_TLS_CPURESERVED as isize) as PBYTE;
    if CpuArea.is_null() {
        return;
    }
    *Wow64pTlsSlots().offset(WOW64_TLS_CPURESERVED as isize) = core::ptr::null_mut();
    RtlFreeHeap(Wow64pProcessHeap(), 0 as ULONG, *(CpuArea as *mut PVOID).offset(-(1 as isize)));
}

unsafe extern "C" fn Wow64pInitStartContext(Ctx: *mut WOW64_CONTEXT, Eip: ULONG, Esp: ULONG) {
    let mut XSave: PBYTE = core::ptr::null_mut();
    let mut i: ULONG;

    i = 0 as ULONG;
    while (i as usize) < core::mem::size_of::<WOW64_CONTEXT>() {
        *(Ctx as PBYTE).offset(i as isize) = 0 as BYTE;
        i = i.wrapping_add(1);
    }

    (*Ctx).ContextFlags = (WOW64_CONTEXT_FULL | WOW64_CONTEXT_EXTENDED_REGISTERS) as DWORD;
    (*Ctx).Eip = Eip;
    (*Ctx).Esp = Esp;
    (*Ctx).EFlags = 0x202 as DWORD;
    (*Ctx).SegCs = 0x23 as DWORD;
    (*Ctx).SegSs = 0x2B as DWORD;
    (*Ctx).SegDs = 0x2B as DWORD;
    (*Ctx).SegEs = 0x2B as DWORD;
    (*Ctx).SegFs = 0x53 as DWORD;
    (*Ctx).SegGs = 0x2B as DWORD;
    XSave = core::ptr::addr_of_mut!((*Ctx).ExtendedRegisters) as PBYTE;
    *(XSave.offset(0x00 as isize) as *mut USHORT) = 0x027F as USHORT;
    *(XSave.offset(0x18 as isize) as *mut ULONG) = 0x00001F80 as ULONG;
}

unsafe extern "C" fn Wow64pThreadInit(
    Start: *const WOW64_CONTEXT,
    StackBase: ULONG,
    StackLimit: ULONG,
    DeallocationStack: ULONG,
    What: *const i8,
) {
    let mut Teb32: PBYTE = Wow64pCurrentTeb32();
    let mut Ctx: *mut WOW64_CONTEXT = core::ptr::null_mut();

    if Teb32.is_null() {
        DbgPrint(
            b"[FEX-WOW64C] FAIL: %s has no TEB32 (WowTebOffset=0)\n\0".as_ptr() as PCSTR,
            What,
        );
        return;
    }

    /* The WOW64INFO *data* is process-wide, but the TLS slot pointing at it
     * is per-thread — the CPU layer reads it on whichever thread it runs. */
    *Wow64pTlsSlots().offset(WOW64_TLS_WOW64INFO as isize) =
        core::ptr::addr_of_mut!(Wow64pWow64Info) as PVOID;

    Ctx = Wow64pInstallCpuArea();
    if Ctx.is_null() {
        return;
    }
    *Ctx = *Start;

    *(Teb32.offset(0x04 as isize) as *mut ULONG) = StackBase; /* NtTib.StackBase (high) */
    *(Teb32.offset(0x08 as isize) as *mut ULONG) = StackLimit; /* NtTib.StackLimit (low) */

    /* And the BOTTOM of the reservation, which is a different fact from
     * StackLimit: StackLimit is only the COMMITTED floor and walks down as the
     * stack grows, so it cannot say how far this stack is allowed to grow.
     * MiCheckForUserStackOverflow needs that bound to extend a guest stack at
     * all -- with it zero the kernel has no 32-bit descriptor to match a guest
     * fault against, rejects every guest stack-growth fault, and the guest dies
     * on its first page of growth (measured: eventvwr's worker needed the page
     * at 0x1fae000 while TEB64 covered [0x1fb0000, 0x20b0000)). Displacement
     * asserted against the bindgen _TEB32 in ntkrnl-sys::wow64. */
    *(Teb32.offset(0xE0C as isize) as *mut ULONG) = DeallocationStack; /* TEB32.DeallocationStack */

    Wow64pBTCpuThreadInit.unwrap_unchecked()();

    /* From here the thread has guest state, so a guest APC can run on it
     * (Wow64ApcRoutine defers one that arrives earlier). Windows' name for the
     * slot; it is reserved in PEB->TlsBitmap with the rest (ldrinit.rs). */
    *Wow64pTlsSlots().offset(WOW64_TLS_INCPUSIMULATION as isize) =
        1 as PVOID;

    DbgPrint(
        b"[FEX-WOW64C] %s: entering guest eip=%08lx esp=%08lx stack=%08lx..%08lx dealloc=%08lx\n\0".as_ptr() as PCSTR,
        What,
        (*Ctx).Eip,
        (*Ctx).Esp,
        StackLimit,
        StackBase,
        DeallocationStack,
    );
    Wow64pBTCpuSimulate.unwrap_unchecked()();
}

unsafe extern "C" fn Wow64pBuildLoaderApcFrame(mut FrameTop: PBYTE, RunCtx: *const WOW64_CONTEXT, mut SystemArgument1: PVOID) -> ULONG {
    let CAS: ULONG = (((core::mem::size_of::<WOW64_CONTEXT>() as ULONG).wrapping_add(3 as ULONG)) & !(3 as ULONG)) as ULONG;
    let mut E: ULONG = FrameTop.offset(-((16 as ULONG).wrapping_add(CAS as ULONG).wrapping_add(8 as ULONG) as isize)) as ULONG_PTR as ULONG;
    let mut i: ULONG;

    E &= !(15 as ULONG);
    *(E.wrapping_add(0 as ULONG) as ULONG_PTR as *mut ULONG) = Wow64pLdrInitThunk32 as ULONG_PTR as ULONG;
    *(E.wrapping_add(4 as ULONG) as ULONG_PTR as *mut ULONG) = E.wrapping_add(16 as ULONG); /* NormalContext = &CONTEXT */
    *(E.wrapping_add(8 as ULONG) as ULONG_PTR as *mut ULONG) = SystemArgument1 as ULONG_PTR as ULONG;
    *(E.wrapping_add(12 as ULONG) as ULONG_PTR as *mut ULONG) = 0 as ULONG;
    i = 0 as ULONG;
    while (i as usize) < core::mem::size_of::<WOW64_CONTEXT>() {
        *(E.wrapping_add(16 as ULONG).wrapping_add(i) as ULONG_PTR as PBYTE) = *(RunCtx as *const BYTE).offset(i as isize);
        i = i.wrapping_add(1);
    }
    return E;
}

unsafe extern "system" fn Wow64pGuestThreadStart(Parameter: PVOID) -> ULONG {
    let mut Seed: WOW64_THREAD_SEED = *(Parameter as *mut WOW64_THREAD_SEED);
    let mut RunCtx: WOW64_CONTEXT = core::mem::zeroed();
    let mut StartCtx: WOW64_CONTEXT = core::mem::zeroed();
    let mut ApcEsp: ULONG;

    if Seed.Remote != 0 {
        let mut Base: PVOID = Parameter;
        let mut Zero: SIZE_T = 0;
        NtFreeVirtualMemory(
            WOW64_CURRENT_PROCESS,
            core::ptr::addr_of_mut!(Base),
            core::ptr::addr_of_mut!(Zero),
            MEM_RELEASE,
        );
    } else {
        RtlFreeHeap(Wow64pProcessHeap(), 0, Parameter);
    }

    /* What the thread runs once the 32-bit loader has initialized it: the
     * entry kernel32 asked for, on the stack kernel32 allocated for it. */
    Wow64pInitStartContext(core::ptr::addr_of_mut!(RunCtx), Seed.Eip, Seed.Esp);
    RunCtx.Eax = Seed.Eax;
    RunCtx.Ebx = Seed.Ebx;

    /* Route through the guest loader exactly like the initial thread does, so
     * the new thread gets LdrpInitializeThread — TLS slot allocation and
     * DLL_THREAD_ATTACH for every loaded 32-bit DLL. Entering RunCtx directly
     * would skip both, and a guest that uses TLS would read a slot array that
     * was never allocated. The frame goes at the top of the guest's OWN stack
     * (kernel32's BaseCreateStack allocated it before the syscall); the loader
     * unwinds off it before NtContinue-ing to RunCtx. */
    ApcEsp = Wow64pBuildLoaderApcFrame(
        (Seed.Esp as ULONG_PTR) as PBYTE,
        core::ptr::addr_of_mut!(RunCtx),
        core::ptr::null_mut(),
    );
    Wow64pInitStartContext(
        core::ptr::addr_of_mut!(StartCtx),
        (Wow64pKiUserApcDispatcher32 as ULONG_PTR) as ULONG,
        ApcEsp,
    );

    Wow64pThreadInit(
        core::ptr::addr_of_mut!(StartCtx),
        Seed.StackBase,
        Seed.StackLimit,
        Seed.DeallocationStack,
        b"guest thread\0".as_ptr() as *const i8,
    );

    /* Only reached if the entry failed outright — kill this thread, not the
     * process: the rest of the guest is still healthy. */
    DbgPrint(b"[FEX-WOW64C] guest thread: entry failed, exiting thread\n\0".as_ptr() as PCSTR);
    RtlExitUserThread(((0xC0000001u32 as i32)) as LONG);
    return 0;
}
#[no_mangle]
pub unsafe extern "C" fn Wow64CreateGuestThread(
    Eip: ULONG,
    Esp: ULONG,
    Eax: ULONG,
    Ebx: ULONG,
    StackBase: ULONG,
    StackLimit: ULONG,
    DeallocationStack: ULONG,
    CreateSuspended: BOOLEAN,
    ThreadHandle: *mut HANDLE,
    ClientId: *mut WOW64_CLIENT_ID,
) -> LONG {
    let mut Seed: *mut WOW64_THREAD_SEED = core::ptr::null_mut::<WOW64_THREAD_SEED>();
    let mut Status: LONG;

    if Wow64pBTCpuThreadInit.is_none() || Wow64pBTCpuSimulate.is_none() {
        DbgPrint(b"[FEX-WOW64C] Wow64CreateGuestThread: no CPU layer\n\0" as *const u8 as PCSTR);
        return 0xC0000001u32 as LONG; /* STATUS_UNSUCCESSFUL */
    }

    Seed = RtlAllocateHeap(
        Wow64pProcessHeap(),
        WOW64_HEAP_ZERO_MEMORY as ULONG,
        core::mem::size_of::<WOW64_THREAD_SEED>() as SIZE_T,
    ) as *mut WOW64_THREAD_SEED;
    if Seed.is_null() {
        return 0xC0000017u32 as LONG; /* STATUS_NO_MEMORY */
    }

    (*Seed).Eip = Eip;
    (*Seed).Esp = Esp;
    (*Seed).Eax = Eax;
    (*Seed).Ebx = Ebx;
    (*Seed).StackBase = StackBase;
    (*Seed).StackLimit = StackLimit;
    (*Seed).DeallocationStack = DeallocationStack;

    Status = RtlCreateUserThread(
        WOW64_CURRENT_PROCESS as HANDLE,
        core::ptr::null_mut(),
        CreateSuspended,
        0 as ULONG,
        0 as SIZE_T,
        0 as SIZE_T,
        core::mem::transmute::<*mut c_void, _>(Wow64pGuestThreadStart as usize as PVOID),
        Seed as PVOID,
        ThreadHandle,
        ((ClientId) as *mut _CLIENT_ID),
    );
    if Status < 0 as LONG {
        DbgPrint(
            b"[FEX-WOW64C] Wow64CreateGuestThread: RtlCreateUserThread %08lx\n\0" as *const u8 as PCSTR,
            Status,
        );
        RtlFreeHeap(Wow64pProcessHeap(), 0 as ULONG, Seed as PVOID);
    }
    return Status;
}
#[no_mangle]
pub unsafe extern "C" fn Wow64pGuestModule32(
    mut Name: *const WCHAR,
    mut ImgSize: *mut SIZE_T,
) -> PVOID {
    let mut Teb32: PBYTE = Wow64pCurrentTeb32();
    let mut Peb32: ULONG = 0;
    let mut Ldr: ULONG = 0;
    let mut Head: ULONG = 0;
    let mut Cur: ULONG = 0;
    let mut Guard: ::core::ffi::c_uint = 0;
    let mut NameLen: ::core::ffi::c_uint = 0 as ::core::ffi::c_uint;
    let mut i: ::core::ffi::c_uint = 0;
    while *Name.offset(NameLen as isize) != 0 {
        NameLen = NameLen.wrapping_add(1);
    }
    if Teb32.is_null() {
        return core::ptr::null_mut();
    }
    Peb32 = *(Teb32.offset(TEB32_PEB_OFF as isize) as *mut ULONG);
    if Peb32 == 0 {
        return core::ptr::null_mut();
    }
    Ldr = *(Peb32.wrapping_add(LDR32_PEB_LDR_OFF as ULONG) as ULONG_PTR as *mut ULONG);
    if Ldr == 0 {
        return core::ptr::null_mut();
    }
    Head = Ldr.wrapping_add(LDR32_INLOADORDER_OFF as ULONG);
    Cur = *(Head as ULONG_PTR as *mut ULONG);
    Guard = 0 as ::core::ffi::c_uint;
    while Cur != 0 && Cur != Head && Guard < 512 as ::core::ffi::c_uint {
        let mut ByteLen: USHORT =
            *(Cur.wrapping_add(LDR32_ENTRY_BASENAME as ULONG) as ULONG_PTR as *mut USHORT);
        let mut Buf: *const WCHAR = *(Cur.wrapping_add(LDR32_ENTRY_BASENAMEBUF as ULONG)
            as ULONG_PTR as *mut ULONG) as ULONG_PTR
            as *const WCHAR;
        if !(Buf.is_null()
            || (ByteLen as usize).wrapping_div(::core::mem::size_of::<WCHAR>() as usize)
                != NameLen as usize)
        {
            i = 0 as ::core::ffi::c_uint;
            while i < NameLen {
                if Wow64pLowerW(*Buf.offset(i as isize)) as ::core::ffi::c_int
                    != Wow64pLowerW(*Name.offset(i as isize)) as ::core::ffi::c_int
                {
                    break;
                }
                i = i.wrapping_add(1);
            }
            if i == NameLen {
                *ImgSize = *(Cur.wrapping_add(LDR32_ENTRY_SIZEOFIMAGE as ULONG) as ULONG_PTR
                    as *mut ULONG) as SIZE_T;
                return *(Cur.wrapping_add(LDR32_ENTRY_DLLBASE as ULONG) as ULONG_PTR
                    as *mut ULONG) as ULONG_PTR as PVOID;
            }
        }
        Cur = *(Cur as ULONG_PTR as *mut ULONG);
        Guard = Guard.wrapping_add(1);
    }
    return core::ptr::null_mut();
}

#[no_mangle]
pub unsafe extern "system" fn Wow64StartGuestThreadFrom64(Start: ULONG_PTR, Param: ULONG_PTR, Win32Thread: BOOLEAN) {
    let mut RunCtx: WOW64_CONTEXT = core::mem::zeroed::<WOW64_CONTEXT>();
    let mut StartCtx: WOW64_CONTEXT = core::mem::zeroed::<WOW64_CONTEXT>();
    let mut Module: PVOID = core::ptr::null_mut();
    let mut Entry: PVOID = core::ptr::null_mut();
    let mut ImgSize: SIZE_T = 0 as SIZE_T;
    let mut Stack: PBYTE = core::ptr::null_mut();
    let mut Top: ULONG = 0;
    let mut Esp: ULONG = 0;
    let mut ApcEsp: ULONG = 0;

    if Start > 0xFFFFFFFF || Param > 0xFFFFFFFF
    {
        /* LdrpDivertWow64Thread only asks for a start that is not native
         * code; one that is not 32-bit either is nobody's. */
        DbgPrint(b"[FEX-WOW64C] thread from 64-bit: start %p param %p not 32-bit\n\0".as_ptr() as PCSTR,
                 Start as PVOID,
                 Param as PVOID);
        RtlExitUserThread(0xC000007B as ULONG as LONG); /* STATUS_INVALID_IMAGE_FORMAT */
    }
    if Wow64pBTCpuThreadInit.is_none() || Wow64pBTCpuSimulate.is_none() || Wow64pLdrInitThunk32.is_null()
    {
        DbgPrint(b"[FEX-WOW64C] thread from 64-bit: no CPU layer\n\0".as_ptr() as PCSTR);
        RtlExitUserThread(0xC0000001 as ULONG as LONG);
    }

    Module = Wow64pGuestModule32(if Win32Thread != 0 { RUSTCONV_WSTR_990385C876A2.as_ptr() } else { RUSTCONV_WSTR_80073828D8A0.as_ptr() }, core::ptr::addr_of_mut!(ImgSize));
    Entry = if !Module.is_null() {
        Wow64pFindExport32(Module, ImgSize,
                                        if Win32Thread != 0 { b"BaseThreadStart\0".as_ptr() as *const i8 } else { b"RtlExitUserThread\0".as_ptr() as *const i8 })
    } else {
        core::ptr::null_mut()
    };
    if Entry.is_null()
    {
        DbgPrint(b"[FEX-WOW64C] thread from 64-bit: 32-bit %s has no %s\n\0".as_ptr() as PCSTR,
                 if Win32Thread != 0 { b"kernel32\0".as_ptr() as *const i8 } else { b"ntdll\0".as_ptr() as *const i8 },
                 if Win32Thread != 0 { b"BaseThreadStart\0".as_ptr() as *const i8 } else { b"RtlExitUserThread\0".as_ptr() as *const i8 });
        RtlExitUserThread(0xC0000139 as ULONG as LONG); /* STATUS_ENTRYPOINT_NOT_FOUND */
    }

    Stack = Wow64pAllocBelow2G(WOW64_GUEST_FROM64_STACK as SIZE_T, PAGE_READWRITE as ULONG,
                                      b"thread from 64-bit: stack\0".as_ptr() as *const i8) as PBYTE;
    if Stack.is_null()
    {
        RtlExitUserThread(0xC0000017 as ULONG as LONG); /* STATUS_NO_MEMORY */
    }
    Top = (Stack as ULONG_PTR as ULONG).wrapping_add(WOW64_GUEST_FROM64_STACK as ULONG);

    /* The routine's frame. stdcall: [esp] return address, then the args. The
     * loader APC frame goes below it (Wow64pBuildLoaderApcFrame). */
    if Win32Thread != 0
    {
        /* BaseThreadStartup(Start, Param) never returns: ExitThread. */
        Esp = Top.wrapping_sub(0x100 as ULONG) & !(15 as ULONG);
        *(Esp as ULONG_PTR as *mut ULONG).offset(0 as isize) = 0 as ULONG;
        *(Esp as ULONG_PTR as *mut ULONG).offset(1 as isize) = Start as ULONG;
        *(Esp as ULONG_PTR as *mut ULONG).offset(2 as isize) = Param as ULONG;
        Wow64pInitStartContext(core::ptr::addr_of_mut!(RunCtx), Entry as ULONG_PTR as ULONG, Esp);
    }
    else
    {
        /* Start(Param), returning its exit code into RtlExitUserThread. */
        Esp = Top.wrapping_sub(0x100 as ULONG) & !(15 as ULONG);
        *(Esp as ULONG_PTR as *mut ULONG).offset(0 as isize) = Entry as ULONG_PTR as ULONG;
        *(Esp as ULONG_PTR as *mut ULONG).offset(1 as isize) = Param as ULONG;
        Wow64pInitStartContext(core::ptr::addr_of_mut!(RunCtx), Start as ULONG, Esp);
    }

    ApcEsp = Wow64pBuildLoaderApcFrame(Esp.wrapping_sub(16 as ULONG) as ULONG_PTR as PBYTE,
                                    core::ptr::addr_of_mut!(RunCtx),
                                    core::ptr::null_mut());
    Wow64pInitStartContext(core::ptr::addr_of_mut!(StartCtx),
                           Wow64pKiUserApcDispatcher32 as ULONG_PTR as ULONG, ApcEsp);

    DbgPrint(b"[FEX-WOW64C] thread from 64-bit: %s start=%08lx param=%08lx stack=%p\n\0".as_ptr() as PCSTR,
             if Win32Thread != 0 { b"Win32\0".as_ptr() as *const i8 } else { b"Rtl\0".as_ptr() as *const i8 },
             Start as ULONG,
             Param as ULONG,
             Stack);
    Wow64pThreadInit(core::ptr::addr_of_mut!(StartCtx), Top, Stack as ULONG_PTR as ULONG,
                     Stack as ULONG_PTR as ULONG, b"thread from 64-bit\0".as_ptr() as *const i8);

    DbgPrint(b"[FEX-WOW64C] thread from 64-bit: entry failed, exiting thread\n\0".as_ptr() as PCSTR);
    RtlExitUserThread(0xC0000001 as ULONG as LONG);
}

unsafe extern "C" fn Wow64pLowerW(c: WCHAR) -> WCHAR {
    return if c >= 0x0041u16 && c <= 0x005Au16 {
        (c - 0x0041u16 + 0x0061u16) as WCHAR
    } else {
        c
    };
}

unsafe extern "C" fn Wow64pRemoteModuleBase(Process: HANDLE, Peb64: ULONGLONG, Name: *const WCHAR) -> ULONGLONG {
    let mut Ldr: ULONGLONG = 0 as ULONGLONG;
    let mut Head: ULONGLONG = 0;
    let mut Cur: ULONGLONG = 0 as ULONGLONG;
    let mut Done: SIZE_T = 0 as SIZE_T;
    let mut Guard: core::ffi::c_uint;
    let mut NameLen: core::ffi::c_uint = 0 as core::ffi::c_uint;

    while *Name.add(NameLen as usize) != 0 {
        NameLen = NameLen.wrapping_add(1);
    }

    if NtReadVirtualMemory(
        Process,
        (Peb64.wrapping_add(LDR64_PEB_LDR_OFF as ULONGLONG)) as usize as PVOID,
        core::ptr::addr_of_mut!(Ldr) as PVOID,
        core::mem::size_of::<ULONGLONG>() as SIZE_T,
        core::ptr::addr_of_mut!(Done),
    ) < 0 as LONG || Ldr == 0
    {
        return 0 as ULONGLONG;
    }

    Head = Ldr.wrapping_add(LDR64_INLOADORDER_OFF as ULONGLONG);
    if NtReadVirtualMemory(
        Process,
        Head as usize as PVOID,
        core::ptr::addr_of_mut!(Cur) as PVOID,
        core::mem::size_of::<ULONGLONG>() as SIZE_T,
        core::ptr::addr_of_mut!(Done),
    ) < 0 as LONG
    {
        return 0 as ULONGLONG;
    }

    Guard = 0 as core::ffi::c_uint;
    while Cur != 0 && Cur != Head && Guard < LDR64_MAX_MODULES as core::ffi::c_uint {
        /* InLoadOrderLinks sits at offset 0, so the link IS the entry. */
        let mut DllBase: ULONGLONG = 0 as ULONGLONG;
        let mut NameBuf: ULONGLONG = 0 as ULONGLONG;
        let mut ByteLen: USHORT = 0 as USHORT;
        let mut Buf: [WCHAR; 64 as usize] = [0; 64 as usize];
        let mut Chars: core::ffi::c_uint;
        let mut i: core::ffi::c_uint;
        let mut Match: core::ffi::c_int;

        if NtReadVirtualMemory(
            Process,
            (Cur.wrapping_add(LDR64_ENTRY_DLLBASE as ULONGLONG)) as usize as PVOID,
            core::ptr::addr_of_mut!(DllBase) as PVOID,
            core::mem::size_of::<ULONGLONG>() as SIZE_T,
            core::ptr::addr_of_mut!(Done),
        ) >= 0 as LONG
            && NtReadVirtualMemory(
                Process,
                (Cur.wrapping_add(LDR64_ENTRY_BASENAME as ULONGLONG)) as usize as PVOID,
                core::ptr::addr_of_mut!(ByteLen) as PVOID,
                core::mem::size_of::<USHORT>() as SIZE_T,
                core::ptr::addr_of_mut!(Done),
            ) >= 0 as LONG
            && NtReadVirtualMemory(
                Process,
                (Cur.wrapping_add(LDR64_ENTRY_BASENAMEBUF as ULONGLONG)) as usize as PVOID,
                core::ptr::addr_of_mut!(NameBuf) as PVOID,
                core::mem::size_of::<ULONGLONG>() as SIZE_T,
                core::ptr::addr_of_mut!(Done),
            ) >= 0 as LONG
            && NameBuf != 0
            && ByteLen != 0
            && (ByteLen as usize / core::mem::size_of::<WCHAR>()) == NameLen as usize
            && (NameLen as usize)
                < (core::mem::size_of_val(&Buf) / core::mem::size_of_val(&Buf[0 as usize]))
            && NtReadVirtualMemory(
                Process,
                NameBuf as usize as PVOID,
                Buf.as_mut_ptr() as PVOID,
                ByteLen as SIZE_T,
                core::ptr::addr_of_mut!(Done),
            ) >= 0 as LONG
        {
            Chars = (ByteLen as usize / core::mem::size_of::<WCHAR>()) as core::ffi::c_uint;
            Match = 1 as core::ffi::c_int;
            i = 0 as core::ffi::c_uint;
            while i < Chars {
                if Wow64pLowerW(Buf[i as usize])
                    != Wow64pLowerW(*Name.add(i as usize))
                {
                    Match = 0 as core::ffi::c_int;
                    break;
                }
                i = i.wrapping_add(1);
            }
            if Match != 0 {
                return DllBase;
            }
        }

        if NtReadVirtualMemory(
            Process,
            Cur as usize as PVOID,
            core::ptr::addr_of_mut!(Cur) as PVOID,
            core::mem::size_of::<ULONGLONG>() as SIZE_T,
            core::ptr::addr_of_mut!(Done),
        ) < 0 as LONG
        {
            break;
        }
        Guard = Guard.wrapping_add(1);
    }
    return 0 as ULONGLONG;
}
#[no_mangle]
pub unsafe extern "C" fn Wow64pApcRoutineFor(Thread: HANDLE) -> PVOID {
    let mut Tbi: [ULONGLONG; 6] = [0; 6];
    let mut Pbi: [ULONGLONG; 6] = [0; 6];
    let mut Me: [ULONGLONG; 6] = [0; 6];
    let mut Oa: WOW64_OBJECT_ATTRIBUTES = WOW64_OBJECT_ATTRIBUTES {
        Length: core::mem::size_of::<WOW64_OBJECT_ATTRIBUTES>() as ULONG,
        RootDirectory: core::ptr::null_mut(),
        ObjectName: core::ptr::null_mut(),
        Attributes: 0,
        SecurityDescriptor: core::ptr::null_mut(),
        SecurityQualityOfService: core::ptr::null_mut(),
    };
    let mut Cid: WOW64_CLIENT_ID = WOW64_CLIENT_ID {
        UniqueProcess: core::ptr::null_mut(),
        UniqueThread: core::ptr::null_mut(),
    };
    let mut Process: HANDLE = core::ptr::null_mut();
    let mut TheirBase: ULONGLONG = 0 as ULONGLONG;

    if Thread == (-(2 as LONG_PTR)) as HANDLE {
        return (Wow64ApcRoutine as usize) as PVOID;
    }
    if NtQueryInformationThread(
        Thread,
        0 as ULONG, /* ThreadBasicInformation */
        &raw mut Tbi as *mut ULONGLONG as PVOID,
        core::mem::size_of::<[ULONGLONG; 6]>() as ULONG,
        core::ptr::null_mut(),
    ) < 0 as LONG
        || NtQueryInformationProcess(
            WOW64_CURRENT_PROCESS,
            0 as ULONG, /* ProcessBasicInformation */
            &raw mut Me as *mut ULONGLONG as PVOID,
            core::mem::size_of::<[ULONGLONG; 6]>() as ULONG,
            core::ptr::null_mut(),
        ) < 0 as LONG
        || Tbi[2 as usize] == Me[4 as usize]
    {
        /* ours (or a bad handle: the kernel says so) */
        return (Wow64ApcRoutine as usize) as PVOID;
    }

    Cid.UniqueProcess = Tbi[2 as usize] as HANDLE;
    Cid.UniqueThread = core::ptr::null_mut();
    if NtOpenProcess(
        &raw mut Process,
        0x0410 as ACCESS_MASK, /* QUERY_INFORMATION | VM_READ */
        ((&raw mut Oa) as *mut _OBJECT_ATTRIBUTES),
        ((&raw mut Cid) as *mut _CLIENT_ID),
    ) < 0 as LONG
    {
        return core::ptr::null_mut();
    }
    if !Wow64pSelfBase.is_null()
        && NtQueryInformationProcess(
            Process,
            0 as ULONG,
            &raw mut Pbi as *mut ULONGLONG as PVOID,
            core::mem::size_of::<[ULONGLONG; 6]>() as ULONG,
            core::ptr::null_mut(),
        ) >= 0 as LONG
    {
        TheirBase = Wow64pRemoteModuleBase(
            Process,
            Pbi[1 as usize],
            (WOW64_SELF_DLL_NAME).as_ptr(),
        );
    }
    NtClose(Process);
    if TheirBase == 0 {
        return core::ptr::null_mut();
    }
    return ((Wow64ApcRoutine as usize as LONGLONG)
        + (TheirBase as LONGLONG - Wow64pSelfBase as ULONG_PTR as LONGLONG))
        as ULONG_PTR as PVOID;
}

#[no_mangle]
pub unsafe extern "C" fn Wow64CreateForeignGuestThread(
    Process: HANDLE,
    Peb64: ULONGLONG,
    Eip: ULONG,
    Esp: ULONG,
    Eax: ULONG,
    Ebx: ULONG,
    StackBase: ULONG,
    StackLimit: ULONG,
    DeallocationStack: ULONG,
    CreateSuspended: BOOLEAN,
    ThreadHandle: *mut HANDLE,
    ClientId: *mut WOW64_CLIENT_ID,
) -> LONG {
    let mut Seed: WOW64_THREAD_SEED = core::mem::zeroed();
    let mut TheirBase: ULONGLONG;
    let mut Delta: LONGLONG;
    let mut Remote: PVOID = core::ptr::null_mut();
    let mut RemoteStart: PVOID;
    let mut Size: SIZE_T = core::mem::size_of::<WOW64_THREAD_SEED>() as SIZE_T;
    let mut Done: SIZE_T = 0;
    let mut Zero: SIZE_T = 0;
    let mut Status: LONG;

    if Wow64pSelfBase.is_null() {
        return 0xC0000001 as ULONG as LONG; /* STATUS_UNSUCCESSFUL */
    }

    TheirBase = Wow64pRemoteModuleBase(Process, Peb64, (WOW64_SELF_DLL_NAME).as_ptr());
    if TheirBase == 0 {
        /* Not a WoW64 process, or one whose wow64.dll has not loaded yet --
         * either way there is nothing over there to run the guest side, and
         * refusing beats starting a thread that cannot be initialised. */
        DbgPrint(
            b"[FEX-WOW64C] foreign thread: target has no wow64.dll loaded\n\0".as_ptr() as PCSTR,
        );
        return 0xC0000002 as ULONG as LONG; /* STATUS_NOT_IMPLEMENTED */
    }
    Delta = (TheirBase as LONGLONG).wrapping_sub(Wow64pSelfBase as ULONG_PTR as LONGLONG);
    RemoteStart = (((Wow64pGuestThreadStart as usize as ULONG_PTR) as LONGLONG) + Delta)
        as ULONG_PTR as PVOID;

    Seed.Eip = Eip;
    Seed.Esp = Esp;
    Seed.Eax = Eax;
    Seed.Ebx = Ebx;
    Seed.StackBase = StackBase;
    Seed.StackLimit = StackLimit;
    Seed.DeallocationStack = DeallocationStack;
    Seed.Remote = 1;

    Status = NtAllocateVirtualMemory(
        Process,
        &raw mut Remote,
        0,
        &raw mut Size,
        (MEM_COMMIT | MEM_RESERVE) as ULONG,
        PAGE_READWRITE as ULONG,
    );
    if Status < 0 {
        DbgPrint(
            b"[FEX-WOW64C] foreign thread: seed allocate %08lx\n\0".as_ptr() as PCSTR,
            Status,
        );
        return Status;
    }

    Status = NtWriteVirtualMemory(
        Process,
        Remote,
        &raw mut Seed as PVOID,
        core::mem::size_of::<WOW64_THREAD_SEED>() as SIZE_T,
        &raw mut Done,
    );
    if Status < 0 || Done != core::mem::size_of::<WOW64_THREAD_SEED>() as SIZE_T {
        NtFreeVirtualMemory(
            Process,
            &raw mut Remote,
            &raw mut Zero,
            MEM_RELEASE as ULONG,
        );
        return if Status < 0 {
            Status
        } else {
            0x8000000D as ULONG as LONG
        };
    }

    Status = RtlCreateUserThread(
        Process,
        core::ptr::null_mut(),
        CreateSuspended,
        0 as ULONG,
        0 as SIZE_T,
        0 as SIZE_T,
        core::mem::transmute::<*mut c_void, _>(RemoteStart),
        Remote,
        ThreadHandle as PHANDLE,
        ((ClientId) as *mut _CLIENT_ID),
    );
    if Status < 0 {
        NtFreeVirtualMemory(
            Process,
            &raw mut Remote,
            &raw mut Zero,
            MEM_RELEASE as ULONG,
        );
        DbgPrint(
            b"[FEX-WOW64C] foreign thread: RtlCreateUserThread %08lx\n\0".as_ptr() as PCSTR,
            Status,
        );
        return Status;
    }

    DbgPrint(
        b"[FEX-WOW64C] foreign guest thread tid=%lx eip=%08lx esp=%08lx seed=%p start=%p (wow64.dll delta %s)\n\0"
            .as_ptr() as PCSTR,
        (*ClientId).UniqueThread as ULONG_PTR as ULONG,
        Eip,
        Esp,
        Remote,
        RemoteStart,
        if Delta != 0 {
            b"NONZERO - relocated\0".as_ptr() as *const i8
        } else {
            b"0\0".as_ptr() as *const i8
        },
    );
    return Status;
}

#[no_mangle]
pub unsafe extern "system" fn Wow64LdrpInitialize(Context: PVOID) {
    let mut Peb32: ULONG_PTR = 0;
    let mut Teb32: PBYTE = Wow64pCurrentTeb32();
    let mut ImageBase32: ULONG = 0;
    let mut ImageBase: PBYTE = core::ptr::null_mut();
    let mut Nt32: PIMAGE_NT_HEADERS32 = core::ptr::null_mut();
    let mut EntryRva: ULONG = 0;
    let mut GuestStackSize: ULONG = 0;
    let mut Ntdll32: PVOID = core::ptr::null_mut();
    let mut InitBlock: PVOID = core::ptr::null_mut();
    let mut Fex: PVOID = core::ptr::null_mut();
    let mut Bop: PVOID = core::ptr::null_mut();
    let mut ExitStub: PBYTE = core::ptr::null_mut();
    let mut GuestStack: PBYTE = core::ptr::null_mut();
    let mut ApcStack: PBYTE = core::ptr::null_mut();
    let mut KiUserApcDispatcher32: PVOID = core::ptr::null_mut();
    let mut LdrInitThunk32: PVOID = core::ptr::null_mut();
    let mut GuestEsp: ULONG = 0;
    let mut ApcEsp: ULONG = 0;
    let mut DllName: WOW64_UNICODE_STRING = core::mem::zeroed();
    let mut ProcName: WOW64_ANSI_STRING = core::mem::zeroed();
    let mut BTCpuProcessInit: Option<unsafe extern "C" fn()> = None;
    let mut BTCpuThreadInit: Option<unsafe extern "C" fn()> = None;
    let mut BTCpuGetBopCode: Option<unsafe extern "C" fn() -> PVOID> = None;
    let mut BTCpuSimulate: Option<unsafe extern "C" fn()> = None;
    let mut BTCpuNotifyMapViewOfSection: Option<
        unsafe extern "C" fn(PVOID, PVOID, PVOID, SIZE_T, ULONG, ULONG) -> LONG,
    > = None;
    let mut Status: LONG = 0;

    DbgPrint(b"[FEX-WOW64C] Wow64LdrpInitialize: 64-bit loader handed the thread over (Context=%p)\n\0".as_ptr() as PCSTR, Context);

    /* Register the crash oracle FIRST: libwow64fex.dll (and its kernel32
     * dependency) load below, before any BTCpu call — a fault there must be
     * dumped, not swallowed into a silent process exit. */
    RtlAddVectoredExceptionHandler(1, Some(Wow64pVeh));

    /* -- M1 chain verification (kernel bootstrap sanity) ------------------ */
    Status = NtQueryInformationProcess(
        WOW64_CURRENT_PROCESS,
        26,
        core::ptr::addr_of_mut!(Peb32) as PVOID,
        ::core::mem::size_of::<ULONG_PTR>() as ULONG,
        core::ptr::null_mut(),
    );
    if Status < 0
        || Peb32 == 0
        || Teb32.is_null()
        || *(Teb32.offset(0x30 as isize) as *mut ULONG) != Peb32 as ULONG
    {
        DbgPrint(
            b"[FEX-WOW64C] FAIL: PEB32/TEB32 chain inconsistent (PEB32=%p TEB32=%p st=%08lx)\n\0".as_ptr() as PCSTR,
            Peb32 as PVOID,
            Teb32,
            Status,
        );
        NtTerminateProcess(WOW64_CURRENT_PROCESS, 0xBAD);
    }

    Wow64pPeb32 = Peb32; /* for the VEH heap diagnostic */
    ImageBase32 = *((Peb32 as PBYTE).offset(0x08 as isize) as *mut ULONG);
    ImageBase = ImageBase32 as ULONG_PTR as PBYTE;
    Nt32 = ImageBase.offset((*(ImageBase as PIMAGE_DOS_HEADER)).e_lfanew as isize) as PIMAGE_NT_HEADERS32;
    EntryRva = (*Nt32).OptionalHeader.AddressOfEntryPoint as ULONG;
    DbgPrint(
        b"[FEX-WOW64C] PEB32=%p TEB32=%p image=%08lx entry=+%lx\n\0".as_ptr() as PCSTR,
        Peb32 as PVOID,
        Teb32,
        ImageBase32,
        EntryRva,
    );

    /* -- 32-bit ntdll + LdrSystemDllInitBlock ----------------------------- */
    Ntdll32 = Wow64pMapNtdll32();
    if Ntdll32.is_null() {
        NtTerminateProcess(WOW64_CURRENT_PROCESS, 0xBAD);
    }

    {
        let mut NtdllName: WOW64_UNICODE_STRING = core::mem::zeroed();
        let mut NtdllBase: PVOID = core::ptr::null_mut();
        RtlInitUnicodeString((core::ptr::addr_of_mut!(NtdllName) as *mut _UNICODE_STRING), RUSTCONV_WSTR_80073828D8A0.as_ptr());
        LdrGetDllHandle(core::ptr::null_mut(), core::ptr::null_mut(), (core::ptr::addr_of_mut!(NtdllName) as *mut _UNICODE_STRING), core::ptr::addr_of_mut!(NtdllBase));
        RtlInitAnsiString((core::ptr::addr_of_mut!(ProcName) as *mut _STRING), b"LdrSystemDllInitBlock\0".as_ptr() as PCSTR);
        InitBlock = core::ptr::null_mut();
        if NtdllBase.is_null()
            || LdrGetProcedureAddress(NtdllBase, (core::ptr::addr_of_mut!(ProcName) as *mut _STRING), 0, core::ptr::addr_of_mut!(InitBlock)) < 0
            || InitBlock.is_null()
        {
            DbgPrint(b"[FEX-WOW64C] FAIL: LdrSystemDllInitBlock unresolved\n\0".as_ptr() as PCSTR);
            NtTerminateProcess(WOW64_CURRENT_PROCESS, 0xBAD);
        }
        *((InitBlock as PBYTE).offset(WOW64_INITBLOCK_NTDLL_OFF as isize) as *mut ULONGLONG) = Ntdll32 as ULONG_PTR as ULONGLONG;
    }

    /* -- WOW64INFO + environment landmine ---------------------------------- */
    Wow64pWow64Info.NativeSystemPageSize = 0x1000 as ULONG;
    Wow64pWow64Info.NativeMachineType = WOW64_NATIVE_MACHINE as USHORT;
    Wow64pWow64Info.EmulatedMachineType = WOW64_MACHINE_I386 as USHORT;
    *Wow64pTlsSlots().offset(WOW64_TLS_WOW64INFO as isize) = core::ptr::addr_of_mut!(Wow64pWow64Info) as PVOID;

    {
        let mut EnvName: WOW64_UNICODE_STRING = core::mem::zeroed();
        let mut EnvValue: WOW64_UNICODE_STRING = core::mem::zeroed();
        RtlInitUnicodeString((core::ptr::addr_of_mut!(EnvName) as *mut _UNICODE_STRING), RUSTCONV_WSTR_D1FD64EDDC71.as_ptr());
        RtlInitUnicodeString((core::ptr::addr_of_mut!(EnvValue) as *mut _UNICODE_STRING), RUSTCONV_WSTR_86A7B64C8DDA.as_ptr());
        RtlSetEnvironmentVariable(core::ptr::null_mut(), ((core::ptr::addr_of_mut!(EnvName)) as *mut _UNICODE_STRING), ((core::ptr::addr_of_mut!(EnvValue)) as *mut _UNICODE_STRING));
    }

    /* -- Load the CPU layer ------------------------------------------------ */
    DbgPrint(b"[FEX-WOW64C] InitBlock.ntdll_handle set; loading CPU layer...\n\0".as_ptr() as PCSTR);
    RtlInitUnicodeString((core::ptr::addr_of_mut!(DllName) as *mut _UNICODE_STRING), (WOW64_CPU_DLL).as_ptr());
    Fex = core::ptr::null_mut();
    Status = LdrLoadDll(core::ptr::null_mut(), core::ptr::null_mut(), (core::ptr::addr_of_mut!(DllName) as *mut _UNICODE_STRING), core::ptr::addr_of_mut!(Fex));
    if Status < 0 || Fex.is_null() {
        DbgPrint(b"[FEX-WOW64C] FAIL: CPU-layer DLL load %08lx\n\0".as_ptr() as PCSTR, Status);
        NtTerminateProcess(WOW64_CURRENT_PROCESS, 0xBAD);
    }

    /* WOW64_RESOLVE(BTCpuProcessInit) */
    {
        let mut Addr: PVOID = core::ptr::null_mut();
        RtlInitAnsiString((core::ptr::addr_of_mut!(ProcName) as *mut _STRING), b"BTCpuProcessInit\0".as_ptr() as PCSTR);
        if LdrGetProcedureAddress(Fex, (core::ptr::addr_of_mut!(ProcName) as *mut _STRING), 0, core::ptr::addr_of_mut!(Addr)) < 0 || Addr.is_null() {
            DbgPrint(b"[FEX-WOW64C] FAIL: BTCpuProcessInit unresolved\n\0".as_ptr() as PCSTR);
            NtTerminateProcess(WOW64_CURRENT_PROCESS, 0xBAD);
        }
        BTCpuProcessInit = Some(core::mem::transmute(Addr));
    }
    /* WOW64_RESOLVE(BTCpuThreadInit) */
    {
        let mut Addr: PVOID = core::ptr::null_mut();
        RtlInitAnsiString((core::ptr::addr_of_mut!(ProcName) as *mut _STRING), b"BTCpuThreadInit\0".as_ptr() as PCSTR);
        if LdrGetProcedureAddress(Fex, (core::ptr::addr_of_mut!(ProcName) as *mut _STRING), 0, core::ptr::addr_of_mut!(Addr)) < 0 || Addr.is_null() {
            DbgPrint(b"[FEX-WOW64C] FAIL: BTCpuThreadInit unresolved\n\0".as_ptr() as PCSTR);
            NtTerminateProcess(WOW64_CURRENT_PROCESS, 0xBAD);
        }
        BTCpuThreadInit = Some(core::mem::transmute(Addr));
    }
    /* WOW64_RESOLVE(BTCpuGetBopCode) */
    {
        let mut Addr: PVOID = core::ptr::null_mut();
        RtlInitAnsiString((core::ptr::addr_of_mut!(ProcName) as *mut _STRING), b"BTCpuGetBopCode\0".as_ptr() as PCSTR);
        if LdrGetProcedureAddress(Fex, (core::ptr::addr_of_mut!(ProcName) as *mut _STRING), 0, core::ptr::addr_of_mut!(Addr)) < 0 || Addr.is_null() {
            DbgPrint(b"[FEX-WOW64C] FAIL: BTCpuGetBopCode unresolved\n\0".as_ptr() as PCSTR);
            NtTerminateProcess(WOW64_CURRENT_PROCESS, 0xBAD);
        }
        BTCpuGetBopCode = Some(core::mem::transmute(Addr));
    }
    /* WOW64_RESOLVE(BTCpuSimulate) */
    {
        let mut Addr: PVOID = core::ptr::null_mut();
        RtlInitAnsiString((core::ptr::addr_of_mut!(ProcName) as *mut _STRING), b"BTCpuSimulate\0".as_ptr() as PCSTR);
        if LdrGetProcedureAddress(Fex, (core::ptr::addr_of_mut!(ProcName) as *mut _STRING), 0, core::ptr::addr_of_mut!(Addr)) < 0 || Addr.is_null() {
            DbgPrint(b"[FEX-WOW64C] FAIL: BTCpuSimulate unresolved\n\0".as_ptr() as PCSTR);
            NtTerminateProcess(WOW64_CURRENT_PROCESS, 0xBAD);
        }
        BTCpuSimulate = Some(core::mem::transmute(Addr));
    }
    /* WOW64_RESOLVE(BTCpuNotifyMapViewOfSection) */
    {
        let mut Addr: PVOID = core::ptr::null_mut();
        RtlInitAnsiString((core::ptr::addr_of_mut!(ProcName) as *mut _STRING), b"BTCpuNotifyMapViewOfSection\0".as_ptr() as PCSTR);
        if LdrGetProcedureAddress(Fex, (core::ptr::addr_of_mut!(ProcName) as *mut _STRING), 0, core::ptr::addr_of_mut!(Addr)) < 0 || Addr.is_null() {
            DbgPrint(b"[FEX-WOW64C] FAIL: BTCpuNotifyMapViewOfSection unresolved\n\0".as_ptr() as PCSTR);
            NtTerminateProcess(WOW64_CURRENT_PROCESS, 0xBAD);
        }
        BTCpuNotifyMapViewOfSection = Some(core::mem::transmute(Addr));
    }
    Wow64pBTCpuNotifyMap = BTCpuNotifyMapViewOfSection;
    {
        let mut Addr: PVOID = core::ptr::null_mut();
        RtlInitAnsiString((core::ptr::addr_of_mut!(ProcName) as *mut _STRING), b"BTCpuSetContext\0".as_ptr() as PCSTR);
        LdrGetProcedureAddress(Fex, (core::ptr::addr_of_mut!(ProcName) as *mut _STRING), 0, core::ptr::addr_of_mut!(Addr));
        Wow64pBTCpuSetContext = Some(core::mem::transmute::<
            PVOID,
            unsafe extern "C" fn(HANDLE, HANDLE, PVOID, PVOID) -> LONG,
        >(Addr));
    }
    {
        let mut Addr: PVOID = core::ptr::null_mut();
        RtlInitAnsiString((core::ptr::addr_of_mut!(ProcName) as *mut _STRING), b"BTCpuGetContext\0".as_ptr() as PCSTR);
        LdrGetProcedureAddress(Fex, (core::ptr::addr_of_mut!(ProcName) as *mut _STRING), 0, core::ptr::addr_of_mut!(Addr));
        Wow64pBTCpuGetContext = Some(core::mem::transmute::<
            PVOID,
            unsafe extern "C" fn(HANDLE, HANDLE, PVOID, PVOID) -> LONG,
        >(Addr));
    }
    Wow64pBTCpuSimulate = BTCpuSimulate;
    Wow64pBTCpuThreadInit = BTCpuThreadInit; /* #64: the per-thread entry needs it */
    {
        let mut Addr: PVOID = core::ptr::null_mut();
        RtlInitAnsiString((core::ptr::addr_of_mut!(ProcName) as *mut _STRING), b"BTCpuResetToConsistentState\0".as_ptr() as PCSTR);
        LdrGetProcedureAddress(Fex, (core::ptr::addr_of_mut!(ProcName) as *mut _STRING), 0, core::ptr::addr_of_mut!(Addr));
        Wow64pBTCpuResetToConsistentState = Some(core::mem::transmute::<
            PVOID,
            unsafe extern "C" fn(*mut EXCEPTION_POINTERS) -> LONG,
        >(Addr));
    }

    DbgPrint(
        b"[FEX-WOW64C] CPU layer %ws loaded @ %p; calling BTCpuProcessInit...\n\0".as_ptr() as PCSTR,
        WOW64_CPU_DLL.as_ptr(),
        Fex,
    );
    /* FEX discovers the process address-space layout during ProcessInit.
     * Allocate every raw guest page first; pages created afterward are not
     * visible until the full memory-notification contract is wired. */
    ExitStub = Wow64pAllocBelow2G(0x1000, PAGE_EXECUTE_READWRITE, b"exit stub\0".as_ptr() as PCSTR) as PBYTE;

    /*
     * The guest's stack must be the size its IMAGE asked for, not a fixed 64K.
     * (See original C commentary: NOTE this COMMITS the whole reserve.)
     */
    GuestStackSize = (*Nt32).OptionalHeader.SizeOfStackReserve as ULONG;
    if GuestStackSize < 0x10000 {
        GuestStackSize = 0x10000; /* never smaller than the old fixed size */
    }
    if GuestStackSize > 0x1000000 {
        GuestStackSize = 0x1000000; /* sanity cap: 16MB */
    }
    GuestStackSize = GuestStackSize.wrapping_add(0xFFFF) & !(0xFFFF);
    GuestStack = Wow64pAllocBelow2G(GuestStackSize as SIZE_T, PAGE_READWRITE, b"guest run stack\0".as_ptr() as PCSTR) as PBYTE;
    ApcStack = Wow64pAllocBelow2G(0x40000, PAGE_READWRITE, b"guest loader stack\0".as_ptr() as PCSTR) as PBYTE;
    if ExitStub.is_null() || GuestStack.is_null() || ApcStack.is_null() {
        NtTerminateProcess(WOW64_CURRENT_PROCESS, 0xBAD);
    }

    /* Build PEB32->ProcessParameters (down-converted from PEB64) so the 32-bit
     * loader has valid ImagePathName/CommandLine to run LdrpInitializeProcess. */
    Wow64pBuildProcessParams32(Peb32);

    BTCpuProcessInit.unwrap_unchecked()();
    /* Only now: the CPU layer built its address-space model inside ProcessInit,
     * so a notification delivered before this point has nothing to land in. */
    Wow64pResolveNotifyContract(Fex);
    Bop = BTCpuGetBopCode.unwrap_unchecked()();
    Wow64pBopCode = Bop;
    DbgPrint(
        b"[FEX-WOW64C] BTCpuProcessInit OK \xE2\x80\x94 BOP=%p CpuFlags=%lx\n\0".as_ptr() as PCSTR,
        Bop,
        Wow64pWow64Info.CpuFlags,
    );
    if (Bop as ULONG_PTR) >= 0x80000000 {
        DbgPrint(b"[FEX-WOW64C] FAIL: BOP above 2GB\n\0".as_ptr() as PCSTR);
        NtTerminateProcess(WOW64_CURRENT_PROCESS, 0xBAD);
    }

    /* -- Publish the BOP into the guest ntdll32's Wow64Transition slot ------ */
    {
        let mut Slot: PULONG = Wow64pFindExport32(Ntdll32, Wow64pNtdll32Size, b"Wow64Transition\0".as_ptr() as *const ::core::ffi::c_char) as PULONG;
        let mut Page: PVOID = core::ptr::null_mut();
        let mut Size: SIZE_T = ::core::mem::size_of::<ULONG>() as SIZE_T;
        let mut OldProt: ULONG = 0;
        if Slot.is_null() {
            DbgPrint(b"[FEX-WOW64C] FAIL: ntdll32 has no Wow64Transition export (stale syswow64 stage?)\n\0".as_ptr() as PCSTR);
            NtTerminateProcess(WOW64_CURRENT_PROCESS, 0xBAD);
        }
        Page = Slot as PVOID;
        NtProtectVirtualMemory(
            WOW64_CURRENT_PROCESS,
            core::ptr::addr_of_mut!(Page),
            core::ptr::addr_of_mut!(Size),
            PAGE_READWRITE,
            core::ptr::addr_of_mut!(OldProt),
        ); /* COW data — best effort */
        *Slot = Bop as ULONG_PTR as ULONG;
        DbgPrint(b"[FEX-WOW64C] Wow64Transition slot %p patched -> BOP %p\n\0".as_ptr() as PCSTR, Slot, Bop);
    }

    /* -- Register the guest images with FEX ------------------------------- */
    BTCpuNotifyMapViewOfSection.unwrap_unchecked()(
        core::ptr::null_mut(),
        ImageBase32 as ULONG_PTR as PVOID,
        core::ptr::null_mut(),
        0,
        0,
        PAGE_EXECUTE_READ,
    );
    BTCpuNotifyMapViewOfSection.unwrap_unchecked()(
        core::ptr::null_mut(),
        Ntdll32,
        core::ptr::null_mut(),
        0,
        0,
        PAGE_EXECUTE_READ,
    );
    DbgPrint(
        b"[FEX-WOW64C] BTCpuNotifyMapViewOfSection: image=%08lx ntdll32=%p registered\n\0".as_ptr() as PCSTR,
        ImageBase32,
        Ntdll32,
    );

    /* -- Resolve the guest ntdll32 loader entrypoints (M3b) ---------------- */
    KiUserApcDispatcher32 = Wow64pFindExport32(Ntdll32, Wow64pNtdll32Size, b"KiUserApcDispatcher\0".as_ptr() as *const ::core::ffi::c_char);
    LdrInitThunk32 = Wow64pFindExport32(Ntdll32, Wow64pNtdll32Size, b"LdrInitializeThunk\0".as_ptr() as *const ::core::ffi::c_char);
    /* #64: hoist them out of this frame — Wow64pGuestThreadStart runs later. */
    Wow64pKiUserApcDispatcher32 = KiUserApcDispatcher32;
    Wow64pLdrInitThunk32 = LdrInitThunk32;
    /* M8: kernel->user callbacks re-enter the guest through its own 32-bit
     * callback dispatcher; the 64-bit PEB table points at our bridge. */
    Wow64pKiUserCallbackDispatcher32 = Wow64pFindExport32(Ntdll32, Wow64pNtdll32Size, b"KiUserCallbackDispatcher\0".as_ptr() as *const ::core::ffi::c_char);
    /* E5: guest SEH entry — non-fatal if absent. */
    Wow64pKiUserExceptionDispatcher32 = Wow64pFindExport32(Ntdll32, Wow64pNtdll32Size, b"KiUserExceptionDispatcher\0".as_ptr() as *const ::core::ffi::c_char);
    if Wow64pKiUserExceptionDispatcher32.is_null() {
        DbgPrint(b"[FEX-WOW64C] WARN: ntdll32 has no KiUserExceptionDispatcher export \xE2\x80\x94 guest SEH dispatch disabled\n\0".as_ptr() as PCSTR);
    }
    /* E5: install the KiUserExceptionDispatcher hook in the HOST ntdll. */
    #[cfg(not(target_arch = "aarch64"))] // `#ifndef _M_ARM64` in the C
    {
        let mut NtdllName: WOW64_UNICODE_STRING = core::mem::zeroed();
        let mut Ntdll64: PVOID = core::ptr::null_mut();
        let mut Slot: *mut PVOID = core::ptr::null_mut();
        RtlInitUnicodeString((core::ptr::addr_of_mut!(NtdllName) as *mut _UNICODE_STRING), RUSTCONV_WSTR_80073828D8A0.as_ptr());
        LdrGetDllHandle(core::ptr::null_mut(), core::ptr::null_mut(), (core::ptr::addr_of_mut!(NtdllName) as *mut _UNICODE_STRING), core::ptr::addr_of_mut!(Ntdll64));
        RtlInitAnsiString((core::ptr::addr_of_mut!(ProcName) as *mut _STRING), b"Wow64PrepareForException\0".as_ptr() as PCSTR);
        if !Ntdll64.is_null()
            && LdrGetProcedureAddress(Ntdll64, (core::ptr::addr_of_mut!(ProcName) as *mut _STRING), 0, core::ptr::addr_of_mut!(Slot) as *mut PVOID) >= 0
            && !Slot.is_null()
        {
            *Slot = (Wow64pPrepareForExceptionHook as usize) as PVOID;
            DbgPrint(
                b"[FEX-WOW64C] Wow64PrepareForException hook installed @ %p (slot %p)\n\0".as_ptr() as PCSTR,
                (Wow64pPrepareForExceptionHook as usize) as PVOID,
                Slot as PVOID,
            );
        } else {
            DbgPrint(b"[FEX-WOW64C] WARN: Wow64PrepareForException slot unresolved \xe2\x80\x94 guest SEH dispatch disabled\n\0".as_ptr() as PCSTR);
        }
    }
    Wow64pInstallUserCallbackTable();
    if KiUserApcDispatcher32.is_null() || LdrInitThunk32.is_null() {
        DbgPrint(
            b"[FEX-WOW64C] FAIL: ntdll32 KiUserApcDispatcher=%p LdrInitializeThunk=%p\n\0".as_ptr() as PCSTR,
            KiUserApcDispatcher32,
            LdrInitThunk32,
        );
        NtTerminateProcess(WOW64_CURRENT_PROCESS, 0xBAD);
    }

    /* -- Guest exit stub + stack ------------------------------------------ */
    {
        let mut NtClose32: PVOID = Wow64pFindExport32(Ntdll32, Wow64pNtdll32Size, b"NtClose\0".as_ptr() as *const ::core::ffi::c_char);
        let mut P: PBYTE = ExitStub;
        let mut BopCall: PBYTE = core::ptr::null_mut();
        let mut Rel: ULONG = 0;
        if NtClose32.is_null() {
            DbgPrint(b"[FEX-WOW64C] FAIL: ntdll32 has no NtClose export\n\0".as_ptr() as PCSTR);
            NtTerminateProcess(WOW64_CURRENT_PROCESS, 0xBAD);
        }
        *P = 0x50 as BYTE; /* push eax */
        P = P.offset(1);
        *P = 0x6A as BYTE; /* push 0 */
        P = P.offset(1);
        *P = 0x00 as BYTE;
        P = P.offset(1);
        *P = 0xBA as BYTE; /* mov edx */
        P = P.offset(1);
        *(P as *mut ULONG) = NtClose32 as ULONG_PTR as ULONG;
        P = P.offset(4 as isize);
        *P = 0xFF as BYTE; /* call edx */
        P = P.offset(1);
        *P = 0xD2 as BYTE;
        P = P.offset(1);
        *P = 0x50 as BYTE; /* push eax */
        P = P.offset(1);
        *P = 0xB8 as BYTE; /* mov eax */
        P = P.offset(1);
        *(P as *mut ULONG) = WOW64C_SVC_EXIT as ULONG;
        P = P.offset(4 as isize);
        *P = 0xE8 as BYTE; /* call rel */
        P = P.offset(1);
        BopCall = P;
        P = P.offset(4 as isize);
        *P = 0xF4 as BYTE; /* hlt */
        P = P.offset(1);
        Rel = P.offset_from(BopCall.offset(4 as isize)) as ULONG;
        *(BopCall as *mut ULONG) = Rel;
        *P = 0xBA as BYTE; /* mov edx */
        P = P.offset(1);
        *(P as *mut ULONG) = Bop as ULONG_PTR as ULONG;
        P = P.offset(4 as isize);
        *P = 0xFF as BYTE; /* call edx */
        P = P.offset(1);
        *P = 0xD2 as BYTE;
        P = P.offset(1);
        *P = 0xC3 as BYTE; /* ret */
        P = P.offset(1);
    }

    GuestEsp = (GuestStack as ULONG_PTR as ULONG)
        .wrapping_add(GuestStackSize)
        .wrapping_sub(64);
    GuestEsp = GuestEsp.wrapping_sub(4);
    *(GuestEsp as ULONG_PTR as *mut ULONG) = ExitStub as ULONG_PTR as ULONG; /* ret addr */

    /* -- Inner "run" CONTEXT32 -------------------------------------------- */
    {
        let mut RunCtx: WOW64_CONTEXT = core::mem::zeroed();
        let mut StartCtx: WOW64_CONTEXT = core::mem::zeroed();

        Wow64pInitStartContext(core::ptr::addr_of_mut!(RunCtx), ImageBase32.wrapping_add(EntryRva), GuestEsp);

        /* The loader APC frame goes on the dedicated loader stack for the
         * INITIAL thread. */
        ApcEsp = Wow64pBuildLoaderApcFrame(
            ApcStack.offset(0x40000 as isize).offset(-(0x100 as isize)),
            core::ptr::addr_of_mut!(RunCtx),
            Ntdll32,
        );
        Wow64pInitStartContext(
            core::ptr::addr_of_mut!(StartCtx),
            KiUserApcDispatcher32 as ULONG_PTR as ULONG,
            ApcEsp,
        );

        DbgPrint(
            b"[FEX-WOW64C] entering BTCpuSimulate via loader (apcDisp=%08lx esp=%08lx LdrInit=%p runEip=%08lx exit=%p)\n\0".as_ptr() as PCSTR,
            StartCtx.Eip,
            StartCtx.Esp,
            LdrInitThunk32,
            ImageBase32.wrapping_add(EntryRva),
            ExitStub,
        );

        /* The guest image entry runs on GuestStack — those are the bounds
         * i386 RtlDispatchException validates fs:[0] SEH records against. */
        Wow64pThreadInit(
            core::ptr::addr_of_mut!(StartCtx),
            (GuestStack as ULONG_PTR as ULONG).wrapping_add(GuestStackSize),
            GuestStack as ULONG_PTR as ULONG,
            GuestStack as ULONG_PTR as ULONG,
            b"initial thread\0".as_ptr() as *const ::core::ffi::c_char,
        );
    }

    DbgPrint(b"[FEX-WOW64C] FAIL: BTCpuSimulate returned\n\0".as_ptr() as PCSTR);
    NtTerminateProcess(WOW64_CURRENT_PROCESS, 0xBAD);
}

#[no_mangle]
pub unsafe extern "system" fn Wow64ProcessPendingCrossProcessItems() {}

#[no_mangle]
pub unsafe extern "system" fn DllMain(hDll: HINSTANCE, dwReason: DWORD, lpReserved: LPVOID) -> BOOL {
    UNREFERENCED_PARAMETER!(lpReserved);

    if dwReason == DLL_PROCESS_ATTACH {
        Wow64pSelfBase = hDll as PVOID;
        DbgPrint(b"[WOW64] wow64.dll loaded\n\0".as_ptr() as PCSTR);
    }
    return (TRUE) as BOOL;
}
