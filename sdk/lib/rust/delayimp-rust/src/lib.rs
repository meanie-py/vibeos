//! `__delayLoadHelper2` and its two hook variables: sdk/lib/delayimp's delayimp.c,
//! pfnDliNotifyHook2.c and pfnDliFailureHook2.c, the runtime half of delay-loaded imports.
//!
//! The linker's delay-import thunks call `__delayLoadHelper2(descriptor, &IAT[i])` the first
//! time a delay-imported function is called; it loads the DLL (once per descriptor),
//! resolves the name or ordinal, patches the IAT slot and returns the address. The two hook
//! pointers let a module observe or override each step (MSVC's documented contract, which
//! base/setup/reactos and the sdk_apitest delayimp test exercise). A module that defines its
//! own hook does so in its own object, which links ahead of this archive, so the hook's
//! member here is never pulled (see hooks/).
#![no_std]
#![allow(non_upper_case_globals, non_snake_case, non_camel_case_types)]

use core::ffi::{c_char, c_void};
use core::ptr::{addr_of, null, null_mut};

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    panic_common::panic_common(env!("CARGO_PKG_NAME"), info)
}

/// core's i386 `.eh_frame` names a personality routine; nothing here unwinds, so it is
/// never called (the pseh-rust / setjmp-rust pattern).
#[no_mangle]
pub extern "C" fn rust_eh_personality() {}

type Farproc = Option<unsafe extern "system" fn() -> isize>;
type Hmodule = *mut c_void;

/// `ImgDelayDescr`: every field a DWORD (RVAs since DELAYLOAD_VERSION 0x200).
#[repr(C)]
pub struct ImgDelayDescr {
    grAttrs: u32,
    rvaDLLName: u32,
    rvaHmod: u32,
    rvaIAT: u32,
    rvaINT: u32,
    rvaBoundIAT: u32,
    rvaUnloadIAT: u32,
    dwTimeStamp: u32,
}

/// `DelayLoadProc`: a BOOL then the name/ordinal union.
#[repr(C)]
pub struct DelayLoadProc {
    fImportByName: i32,
    /// `szProcName` (LPCSTR) or `dwOrdinal` (DWORD) -- one pointer-width union.
    szProcName: *const c_char,
}

#[repr(C)]
pub struct DelayLoadInfo {
    cb: u32,
    pidd: *const ImgDelayDescr,
    ppfn: *mut Farproc,
    szDll: *const c_char,
    dlp: DelayLoadProc,
    hmodCur: Hmodule,
    pfnCur: Farproc,
    dwLastError: u32,
}

/// `PfnDliHook`: `FARPROC (WINAPI *)(unsigned dliNotify, PDelayLoadInfo)`.
pub type PfnDliHook = Option<unsafe extern "system" fn(u32, *mut DelayLoadInfo) -> Farproc>;

// The hook slots are defined in two crates of their own (hooks/notify, hooks/failure), so
// each is a separate archive member that a module defining its own hook never pulls; read
// here by symbol name. `extern crate` keeps both in the staticlib.
extern crate delayimp_failure_hook;
extern crate delayimp_notify_hook;
extern "C" {
    static mut __pfnDliNotifyHook2: PfnDliHook;
    static mut __pfnDliFailureHook2: PfnDliHook;
}

const dliStartProcessing: u32 = 0;
const dliNotePreLoadLibrary: u32 = 1;
const dliNotePreGetProcAddress: u32 = 2;
const dliFailLoadLib: u32 = 3;
const dliFailGetProc: u32 = 4;
const dliNoteEndProcessing: u32 = 5;

const ERROR_SUCCESS: u32 = 0;
const ERROR_MOD_NOT_FOUND: u32 = 126;
const ERROR_PROC_NOT_FOUND: u32 = 127;
/// `VcppException(ERROR_SEVERITY_ERROR, err)`: severity | FACILITY_VISUALCPP (0x6d) << 16 | err.
const fn vcpp_exception(err: u32) -> u32 {
    0xC000_0000 | (0x6d << 16) | err
}

/// IMAGE_ORDINAL_FLAG32/64: the top bit of an IMAGE_THUNK_DATA says "by ordinal".
const IMAGE_ORDINAL_FLAG: usize = 1 << (usize::BITS - 1);

extern "C" {
    static __ImageBase: u8;
}
extern "system" {
    fn LoadLibraryA(name: *const c_char) -> Hmodule;
    fn GetProcAddress(module: Hmodule, name: *const c_char) -> Farproc;
    fn GetLastError() -> u32;
    fn RaiseException(code: u32, flags: u32, nargs: u32, args: *const usize);
}

#[inline(always)]
unsafe fn from_rva<T>(rva: u32) -> *mut T {
    addr_of!(__ImageBase).cast::<u8>().wrapping_add(rva as usize) as *mut T
}

/// `FARPROC WINAPI __delayLoadHelper2(PCImgDelayDescr pidd, PImgThunkData pIATEntry)`.
#[no_mangle]
pub unsafe extern "system" fn __delayLoadHelper2(pidd: *const ImgDelayDescr, pIATEntry: *mut usize) -> Farproc {
    let pIAT: *mut usize = from_rva((*pidd).rvaIAT);
    let pINT: *const usize = from_rva((*pidd).rvaINT);
    let phMod: *mut Hmodule = from_rva((*pidd).rvaHmod);
    let index = pIATEntry.offset_from(pIAT) as usize;

    let mut dli = DelayLoadInfo {
        cb: core::mem::size_of::<DelayLoadInfo>() as u32,
        pidd,
        ppfn: pIAT.add(index) as *mut Farproc,
        szDll: from_rva((*pidd).rvaDLLName),
        dlp: DelayLoadProc { fImportByName: 0, szProcName: null() },
        hmodCur: null_mut(),
        pfnCur: None,
        dwLastError: 0,
    };
    let thunk = *pINT.add(index);
    dli.dlp.fImportByName = (thunk & IMAGE_ORDINAL_FLAG == 0) as i32;
    dli.dlp.szProcName = if dli.dlp.fImportByName != 0 {
        // u1.AddressOfData is the RVA of an IMAGE_IMPORT_BY_NAME { WORD Hint; CHAR Name[]; }.
        from_rva::<u8>(thunk as u32).add(2) as *const c_char
    } else {
        // IMAGE_ORDINAL: the low 16 bits, in the union's DWORD (GetProcAddress takes it as is).
        (thunk & 0xffff) as *const c_char
    };

    if let Some(notify) = __pfnDliNotifyHook2 {
        dli.pfnCur = notify(dliStartProcessing, &mut dli);
        if dli.pfnCur.is_some() {
            *pIAT.add(index) = dli.pfnCur.map_or(0, |f| f as usize);
            if let Some(notify) = __pfnDliNotifyHook2 {
                notify(dliNoteEndProcessing, &mut dli);
            }
            return dli.pfnCur;
        }
    }

    dli.hmodCur = *phMod;
    if dli.hmodCur.is_null() {
        if let Some(notify) = __pfnDliNotifyHook2 {
            dli.hmodCur = notify(dliNotePreLoadLibrary, &mut dli).map_or(null_mut(), |f| f as Hmodule);
        }
        if dli.hmodCur.is_null() {
            dli.hmodCur = LoadLibraryA(dli.szDll);
            if dli.hmodCur.is_null() {
                dli.dwLastError = GetLastError();
                if let Some(fail) = __pfnDliFailureHook2 {
                    dli.hmodCur = fail(dliFailLoadLib, &mut dli).map_or(null_mut(), |f| f as Hmodule);
                }
                if dli.hmodCur.is_null() {
                    let args = [addr_of!(dli) as usize];
                    RaiseException(vcpp_exception(ERROR_MOD_NOT_FOUND), 0, 1, args.as_ptr());
                    // If we survive the exception, we are expected to use pfnCur directly.
                    return dli.pfnCur;
                }
            }
        }
        *phMod = dli.hmodCur;
    }

    dli.dwLastError = ERROR_SUCCESS;
    if let Some(notify) = __pfnDliNotifyHook2 {
        dli.pfnCur = notify(dliNotePreGetProcAddress, &mut dli);
    }
    if dli.pfnCur.is_none() {
        // dli.dlp.szProcName may also be an ordinal; GetProcAddress takes either.
        dli.pfnCur = GetProcAddress(dli.hmodCur, dli.dlp.szProcName);
        if dli.pfnCur.is_none() {
            dli.dwLastError = GetLastError();
            if let Some(fail) = __pfnDliFailureHook2 {
                dli.pfnCur = fail(dliFailGetProc, &mut dli);
            }
            if dli.pfnCur.is_none() {
                let args = [addr_of!(dli) as usize];
                RaiseException(vcpp_exception(ERROR_PROC_NOT_FOUND), 0, 1, args.as_ptr());
            }
        }
    }

    *pIAT.add(index) = dli.pfnCur.map_or(0, |f| f as usize);
    dli.dwLastError = ERROR_SUCCESS;
    if let Some(notify) = __pfnDliNotifyHook2 {
        notify(dliNoteEndProcessing, &mut dli);
    }
    dli.pfnCur
}
