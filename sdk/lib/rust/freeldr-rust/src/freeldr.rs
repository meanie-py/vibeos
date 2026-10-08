use ntkrnl_sys::base::*;
use crate::frldr::*;
use ntkrnl_sys::ddk::{PCCH, PCCHAR, PCSTR, PLDR_DATA_TABLE_ENTRY, wchar_t, wctype_t, wint_t};
use core::ffi::c_void;

// freeldr.c is the BIOS entry-point translation unit: BootMain() is invoked
// by arch/<a>/entry.S as plain cdecl `_BootMain`. The wide-char/math fns are
// deliberate stubs to keep CRT bloat out of FreeLDR.

extern "C" {
    // CRT shim provided by ntoskrnl-side ctype tables / sdk/lib/crt.
    fn _isctype(c: i32, mask: i32) -> i32;
    // printf — exported by tui.rs (#[no_mangle] pub extern "C" fn printf).
    fn printf(fmt: *const i8, ...) -> i32;
    // RtlStringCb is the variadic-printf-family shim (lib/strsafe_shim.c on
    // the C side, since stable Rust can't host c_variadic DEFs).
    fn RtlStringCbPrintfA(pszDest: PCHAR, cbDest: usize, fmt: PCSTR, ...) -> NTSTATUS;
    // BootMain dependencies — implemented across other freeldr-rust modules
    // (settings.rs, debug.rs, machpc.rs/uefisetup.rs, ui.rs/directui.rs/noui.rs,
    // meminit.rs, fs.rs, peloader.rs, custom.rs/uefildr.rs, pchw.rs).
    fn LoadSettings(CmdLine: PCSTR);
    fn DebugInit(DebugString: PCSTR);
    fn MachInit(CmdLine: *const i8);
    fn UiInitialize(ShowUi: BOOLEAN) -> BOOLEAN;
    fn UiMessageBoxCritical(MessageText: PCCH);
    fn MmInitializeMemoryManager() -> BOOLEAN;
    fn FsInit();
    fn PeLdrInitializeModuleList() -> BOOLEAN;
    fn PeLdrLoadBootImage(
        FilePath: PCSTR,
        BaseDllName: PCSTR,
        ImageBase: *mut PVOID,
        DataTableEntry: *mut PLDR_DATA_TABLE_ENTRY,
    ) -> BOOLEAN;
    fn FrLdrCheckCpuCompatibility();
    fn Reboot() -> !;
    // PaToVa/VaToPa is a no-op identity in this freeldr — conversion.rs exports it.
    fn VaToPa(Va: *mut c_void) -> *mut c_void;
    // FrldrBootDrive/Partition: defined in i386 asm (drvmap.S/asm) for BIOS,
    // and in uefidisk.rs (#[no_mangle] pub static mut) for UEFI. Both flavors
    // resolve via this extern.
    static FrldrBootDrive: u8;
    static FrldrBootPartition: ULONG;
}

extern "C" {
    fn TRACE(fmt: *const i8, ...);
    fn WARN(fmt: *const i8, ...);
}

// freeldr.c global: `CCHAR FrLdrBootPath[MAX_PATH] = ""` — MAX_PATH==260.
// Exported (hwdisk.rs etc. consume via `extern { static mut FrLdrBootPath: ... }`).
#[no_mangle]
pub static mut FrLdrBootPath: [CHAR; 260] = [0; 260];

// BootMgrInfo lives in settings.rs (#[no_mangle] pub static mut). Pull it
// in via the crate path; no clash since we don't re-declare extern.
use crate::settings::BootMgrInfo;

// ============================================================================
// BootMain debug-console marker (BIOS i386/amd64 only — bochs port 0xE9).
// UEFIBOOT: void.
// ============================================================================
#[cfg(all(any(target_arch = "x86", target_arch = "x86_64"), not(feature = "uefi")))]
#[inline(always)]
unsafe fn BM_DBGCON(c: u8) {
    core::arch::asm!("out dx, al", in("dx") 0xe9u16, in("al") c,
                     options(nomem, nostack, preserves_flags));
}
#[cfg(any(target_arch = "aarch64", feature = "uefi"))]
#[inline(always)]
unsafe fn BM_DBGCON(_c: u8) {}

// ============================================================================
// File-static helpers (C: `static BOOLEAN LoadRosload`, `static ULONG
// LaunchSecondStageLoader`). Not exported.
// ============================================================================

unsafe fn LoadRosload(
    RosloadPath: PCSTR,
    ImageBase: *mut PVOID,
    DataTableEntry: *mut PLDR_DATA_TABLE_ENTRY,
) -> BOOLEAN {
    let mut FullPath: [CHAR; 260] = [0; 260];

    RtlStringCbPrintfA(
        FullPath.as_mut_ptr(),
        core::mem::size_of_val(&FullPath),
        b"%s\\%s\0".as_ptr() as PCSTR,
        FrLdrBootPath.as_ptr(),
        RosloadPath,
    );

    TRACE(b"Loading second stage loader '%s'\n\0".as_ptr() as *const i8, FullPath.as_ptr());

    let Success = PeLdrLoadBootImage(
        FullPath.as_ptr() as PCSTR,
        b"scsiport.sys\0".as_ptr() as PCSTR,
        ImageBase,
        DataTableEntry,
    );
    if Success == 0 {
        WARN(b"Failed to load second stage loader '%s'\n\0".as_ptr() as *const i8, FullPath.as_ptr());
        return FALSE;
    }
    TRUE
}

unsafe fn LaunchSecondStageLoader() -> ULONG {
    let mut RosloadDTE: PLDR_DATA_TABLE_ENTRY = core::ptr::null_mut();
    let mut ImageBase: PVOID = core::ptr::null_mut();

    if LoadRosload(
        b"rosload.exe\0".as_ptr() as PCSTR,
        &mut ImageBase,
        &mut RosloadDTE,
    ) == FALSE
    {
        if LoadRosload(
            b"loader\\rosload.exe\0".as_ptr() as PCSTR,
            &mut ImageBase,
            &mut RosloadDTE,
        ) == FALSE
        {
            // ENOENT == 2 in errno-style; _ARC_CODES variant is 14.
            // freeldr.c uses ENOENT from arc.h → arc-codes "no file" = 14.
            return 14 as ULONG;
        }
    }

    printf(b"Launching rosload.exe...\n\0".as_ptr() as *const i8);
    let entry_va = VaToPa((*RosloadDTE).EntryPoint);
    // rosload's entry (RunLoader) returns VOID: reading a LONG back took whatever was left
    // in eax/rax, and a nonzero leftover printed "Unable to load second stage loader".
    // Upstream e87a74f1312.
    let EntryPoint: extern "C" fn() = core::mem::transmute(entry_va);
    EntryPoint();
    0 // ESUCCESS
}

// ============================================================================
// BootMain — THE BIOS entry point invoked by arch/<a>/entry.S as `_BootMain`
// (cdecl). The UEFI build's entry is UefiMain/EfiMain (different TU); BootMain
// is dead-code on UEFI but harmless to link.
// ============================================================================
#[no_mangle]
pub unsafe extern "C" fn BootMain(CmdLine: PCCH) {
    BM_DBGCON(b'm');
    LoadSettings(CmdLine as PCSTR);
    BM_DBGCON(b's');

    DebugInit(BootMgrInfo.DebugString);
    BM_DBGCON(b'd');

    MachInit(CmdLine);
    BM_DBGCON(b'M');

    TRACE(b"BootMain() called.\n\0".as_ptr() as *const i8);

    #[cfg(all(any(target_arch = "x86", target_arch = "x86_64"), not(feature = "uefi")))]
    {
        BM_DBGCON(b'K');
        FrLdrCheckCpuCompatibility();
        BM_DBGCON(b'k');
    }

    BM_DBGCON(b'u');
    if UiInitialize(FALSE) == FALSE {
        BM_DBGCON(b'U');
        UiMessageBoxCritical(b"Unable to initialize UI.\0".as_ptr() as PCCH);
        Reboot();
    }
    BM_DBGCON(b'o');

    BM_DBGCON(b'z');
    if MmInitializeMemoryManager() == FALSE {
        BM_DBGCON(b'Z');
        UiMessageBoxCritical(b"Unable to initialize memory manager.\0".as_ptr() as PCCH);
        Reboot();
    }
    BM_DBGCON(b'Y');

    BM_DBGCON(b'y');
    FsInit();
    BM_DBGCON(b'I');

    if PeLdrInitializeModuleList() == FALSE {
        UiMessageBoxCritical(b"Unable to initialize module list.\0".as_ptr() as PCCH);
        Reboot();
    }

    // MachInitializeBootDevices() — macro `MachVtbl.InitializeBootDevices()`
    // in C; resolved via the shared frldr.rs slot-27 helper.
    if MachInitializeBootDevices() == FALSE {
        UiMessageBoxCritical(b"Error when detecting hardware.\0".as_ptr() as PCCH);
        Reboot();
    }

    // ESUCCESS == 0 (arc-codes).
    if LaunchSecondStageLoader() != 0 {
        UiMessageBoxCritical(b"Unable to load second stage loader.\0".as_ptr() as PCCH);
    }

    Reboot();
}

// ============================================================================
// wctomb / mbtowc / iswctype — stubs that bypass the CRT's 144 KB wctype
// table. All public in C (no `static`); export as cdecl.
//
// These and pow/log/log10 below are DELIBERATE second definitions: cnt-rust
// (linked after this crate) defines the real ones. In C, freeldr.c's object
// always beat the libcntpr archive member; here freeldr-rust's archive comes
// first on the uefildr/rosload link line, which is what keeps them winning.
// ============================================================================

// nolint: duplicate-symbol (overrides cnt-rust's wctomb on purpose, as freeldr.c overrides libcntpr)
#[no_mangle]
pub unsafe extern "C" fn wctomb(mbchar: *mut i8, wchar: wchar_t) -> i32 {
    *mbchar = wchar as i8;
    1
}

// nolint: duplicate-symbol (overrides cnt-rust's mbtowc on purpose, as freeldr.c overrides libcntpr)
#[no_mangle]
pub unsafe extern "C" fn mbtowc(
    wchar: *mut wchar_t,
    mbchar: *const i8,
    _count: usize,
) -> i32 {
    *wchar = *mbchar as wchar_t;
    1
}

// The wctype table is 144 KB, too much for poor freeldr (freeldr.c). The link
// builds cnt-rust-lib with `freeldr`, which leaves cnt-ctype's iswctype out.
// `(char)wc` as in the C: `_isctype` indexes a 256-entry table.
// nolint: duplicate-symbol (freeldr links build cnt-rust-lib with `freeldr`, whose `no-iswctype` leaves cnt-ctype's out; link_guard.py enforces it)
#[no_mangle]
pub unsafe extern "C" fn iswctype(wc: wint_t, wctypeFlags: wctype_t) -> i32 {
    _isctype(wc as u8 as i8 as i32, wctypeFlags as i32)
}

// ============================================================================
// pow / log / log10 — deliberately-broken stubs (CRT pulls in heavy math
// tables otherwise). __debugbreak() if ever called.
// ============================================================================

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
macro_rules! debugbreak { () => { core::arch::asm!("int3", options(nomem, nostack, preserves_flags)); } }
#[cfg(target_arch = "aarch64")]
macro_rules! debugbreak { () => { core::arch::asm!("brk #0xF000", options(nomem, nostack, preserves_flags)); } }

// nolint: duplicate-symbol (overrides cnt-rust's pow on purpose, as freeldr.c overrides libcntpr)
#[no_mangle]
pub unsafe extern "C" fn pow(_x: f64, _y: f64) -> f64 {
    debugbreak!();
    0.0
}

// nolint: duplicate-symbol (overrides cnt-rust's log on purpose, as freeldr.c overrides libcntpr)
#[no_mangle]
pub unsafe extern "C" fn log(_x: f64) -> f64 {
    debugbreak!();
    0.0
}

// nolint: duplicate-symbol (overrides cnt-rust's log10 on purpose, as freeldr.c overrides libcntpr)
#[no_mangle]
pub unsafe extern "C" fn log10(_x: f64) -> f64 {
    debugbreak!();
    0.0
}

// ============================================================================
// FrLdr* accessors — public, cdecl. Used by ntldr / pchw paths.
// ============================================================================

#[no_mangle]
pub unsafe extern "C" fn FrLdrGetBootPath() -> PCCHAR {
    FrLdrBootPath.as_ptr() as PCCHAR
}

#[no_mangle]
pub unsafe extern "C" fn FrldrGetBootDrive() -> UCHAR {
    FrldrBootDrive
}

#[no_mangle]
pub unsafe extern "C" fn FrldrGetBootPartition() -> ULONG {
    FrldrBootPartition
}
