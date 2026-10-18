#[allow(unused_imports)] use crate::{TRACE, WARN};
#[allow(dead_code, non_upper_case_globals)] const DbgDefaultChannel: u32 = crate::frldr::DPRINT_WARNING; // freeldr.c: DBG_DEFAULT_CHANNEL(WARNING)
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

    TRACE!("Loading second stage loader '%s'\n", FullPath.as_ptr());

    let Success = PeLdrLoadBootImage(
        FullPath.as_ptr() as PCSTR,
        b"scsiport.sys\0".as_ptr() as PCSTR,
        ImageBase,
        DataTableEntry,
    );
    if Success == 0 {
        WARN!("Failed to load second stage loader '%s'\n", FullPath.as_ptr());
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

    TRACE!("BootMain() called.\n");

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

/// TRUE in uefildr.efi, FALSE in the BIOS freeldr.sys. rosload is one binary for both and
/// asks, to set the loader block's `BootViaEFI`: winldr.c's `#ifdef UEFIBOOT` never fires
/// in rosload, so without this nothing after the loader can tell the firmware apart.
#[no_mangle]
pub unsafe extern "C" fn FrLdrIsUefiBoot() -> BOOLEAN {
    cfg!(feature = "uefi") as BOOLEAN
}

/// A valid SMBIOS entry point at `p`: the anchor (`_SM3_` for 3.x, `_SM_` for 2.1), a
/// length that fits, and a byte sum of zero over it.
unsafe fn smbios_entry_point_valid(p: *const u8) -> bool {
    let len = if core::slice::from_raw_parts(p, 5) == b"_SM3_" {
        *p.add(6) as usize
    } else if core::slice::from_raw_parts(p, 4) == b"_SM_" {
        *p.add(5) as usize
    } else {
        return false;
    };
    (16..=32).contains(&len)
        && core::slice::from_raw_parts(p, len).iter().fold(0u8, |s, b| s.wrapping_add(*b)) == 0
}

/// Physical address of the firmware's SMBIOS entry point, or 0. rosload asks, to hand
/// the kernel `LOADER_PARAMETER_EXTENSION.SMBiosEPSHeader`: after boot the kernel's view
/// of the BIOS F-segment holds no tables on the smoke VMs, and under UEFI (all of arm64)
/// SMBIOS is only in the EFI configuration table, which nothing else passes on -- so
/// GetSystemFirmwareTable('RSMB') found nothing.
#[no_mangle]
pub unsafe extern "C" fn FrLdrFindSmbiosEntryPoint() -> ULONG_PTR {
    #[cfg(feature = "uefi")]
    {
        use crate::frldr::{EFI_GUID, EFI_SYSTEM_TABLE};
        extern "C" {
            static mut GlobalSystemTable: *mut EFI_SYSTEM_TABLE;
        }
        // SMBIOS3_TABLE_GUID first: a 3.x entry point is the only one arm64 firmware has.
        const GUIDS: [EFI_GUID; 2] = [
            EFI_GUID { Data1: 0xf2fd1544, Data2: 0x9794, Data3: 0x4a2c,
                       Data4: [0x99, 0x2e, 0xe5, 0xbb, 0xcf, 0x20, 0xe3, 0x94] },
            EFI_GUID { Data1: 0xeb9d2d31, Data2: 0x2d88, Data3: 0x11d3,
                       Data4: [0x9a, 0x16, 0x00, 0x90, 0x27, 0x3f, 0xc1, 0x4d] },
        ];
        let st = GlobalSystemTable;
        for want in &GUIDS {
            for i in 0..(*st).NumberOfTableEntries as usize {
                let e = (*st).ConfigurationTable.add(i);
                let g = &(*e).VendorGuid;
                if g.Data1 == want.Data1 && g.Data2 == want.Data2 && g.Data3 == want.Data3
                    && g.Data4 == want.Data4
                {
                    let p = (*e).VendorTable as *const u8;
                    if !p.is_null() && smbios_entry_point_valid(p) {
                        return p as ULONG_PTR;
                    }
                }
            }
        }
        0
    }
    #[cfg(all(not(feature = "uefi"), any(target_arch = "x86", target_arch = "x86_64")))]
    {
        // The spec's legacy search: 16-byte boundaries in 0xF0000-0xFFFFF, which the
        // loader still sees as the BIOS left it.
        let mut a = 0xF0000usize;
        while a <= 0x100000 - 32 {
            if smbios_entry_point_valid(a as *const u8) {
                return a as ULONG_PTR;
            }
            a += 16;
        }
        0
    }
    #[cfg(all(not(feature = "uefi"), not(any(target_arch = "x86", target_arch = "x86_64"))))]
    {
        0
    }
}
