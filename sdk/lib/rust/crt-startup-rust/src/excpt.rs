//! startup/crt_handler.c — the gnu top-level exception filter and the
//! WIN64 emulated-.pdata handler registration.

use crate::winapi::*;

extern "C" {
    fn signal(sig: i32, handler: SigHandler) -> SigHandler;
    fn _fpreset();
}

#[no_mangle]
pub static mut __mingw_oldexcpt_handler: LPTOP_LEVEL_EXCEPTION_FILTER = None;

const GCC_MAGIC: DWORD = (b'G' as DWORD) << 16 | (b'C' as DWORD) << 8 | b'C' as DWORD | (1 << 29);

#[no_mangle]
pub unsafe extern "system" fn _gnu_exception_handler(
    exception_data: *mut EXCEPTION_POINTERS,
) -> LONG {
    let mut action = EXCEPTION_CONTINUE_SEARCH;
    let mut reset_fpu = false;
    let rec = (*exception_data).ExceptionRecord;
    let code = (*rec).ExceptionCode;

    #[cfg(not(target_arch = "x86"))]
    {
        if (code & 0x20ff_ffff) == GCC_MAGIC
            && ((*rec).ExceptionFlags & EXCEPTION_NONCONTINUABLE) == 0
        {
            return EXCEPTION_CONTINUE_EXECUTION;
        }
    }
    #[cfg(target_arch = "x86")]
    let _ = GCC_MAGIC;

    match code {
        EXCEPTION_ACCESS_VIOLATION => {
            let old = signal(SIGSEGV, sig_dfl());
            if old == sig_ign() {
                signal(SIGSEGV, sig_ign());
                action = EXCEPTION_CONTINUE_EXECUTION;
            } else if old != sig_dfl() {
                if let Some(f) = old {
                    f(SIGSEGV);
                }
                action = EXCEPTION_CONTINUE_EXECUTION;
            }
        }
        EXCEPTION_ILLEGAL_INSTRUCTION | EXCEPTION_PRIV_INSTRUCTION => {
            let old = signal(SIGILL, sig_dfl());
            if old == sig_ign() {
                signal(SIGILL, sig_ign());
                action = EXCEPTION_CONTINUE_EXECUTION;
            } else if old != sig_dfl() {
                if let Some(f) = old {
                    f(SIGILL);
                }
                action = EXCEPTION_CONTINUE_EXECUTION;
            }
        }
        EXCEPTION_FLT_INVALID_OPERATION
        | EXCEPTION_FLT_DIVIDE_BY_ZERO
        | EXCEPTION_FLT_DENORMAL_OPERAND
        | EXCEPTION_FLT_OVERFLOW
        | EXCEPTION_FLT_UNDERFLOW
        | EXCEPTION_FLT_INEXACT_RESULT => {
            reset_fpu = true;
            handle_fpe(reset_fpu, &mut action);
        }
        EXCEPTION_INT_DIVIDE_BY_ZERO => {
            handle_fpe(reset_fpu, &mut action);
        }
        _ => {}
    }

    // WIN64 also continues on these (matches the C `#ifdef _WIN64`).
    #[cfg(not(target_arch = "x86"))]
    match code {
        EXCEPTION_DATATYPE_MISALIGNMENT
        | EXCEPTION_ARRAY_BOUNDS_EXCEEDED
        | EXCEPTION_FLT_STACK_CHECK
        | EXCEPTION_INT_OVERFLOW
        | EXCEPTION_INVALID_HANDLE => {
            action = EXCEPTION_CONTINUE_EXECUTION;
        }
        _ => {}
    }

    if action == EXCEPTION_CONTINUE_SEARCH {
        if let Some(old) = __mingw_oldexcpt_handler {
            action = old(exception_data);
        }
    }
    action
}

#[inline]
unsafe fn handle_fpe(reset_fpu: bool, action: &mut LONG) {
    let old = signal(SIGFPE, sig_dfl());
    if old == sig_ign() {
        signal(SIGFPE, sig_ign());
        if reset_fpu {
            _fpreset();
        }
        *action = EXCEPTION_CONTINUE_EXECUTION;
    } else if old != sig_dfl() {
        if let Some(f) = old {
            f(SIGFPE);
        }
        *action = EXCEPTION_CONTINUE_EXECUTION;
    }
}

// ---- ARM64: nothing to register (handled by the ABI) ----------------
#[cfg(target_arch = "aarch64")]
#[no_mangle]
pub unsafe extern "C" fn __mingw_init_ehandler() -> i32 {
    1
}

// ---- amd64: emulate a .pdata table over the exec sections -----------
#[cfg(target_arch = "x86_64")]
pub use win64::{__mingw_SEH_error_handler, __mingw_init_ehandler};

#[cfg(target_arch = "x86_64")]
mod win64 {
    use super::*;
    use crate::pesect::{_FindPESectionByName, _FindPESectionExec, _GetPEImageBase};

    #[repr(C, packed)]
    #[derive(Clone, Copy)]
    struct UNWIND_INFO {
        VersionAndFlags: u8,
        PrologSize: u8,
        CountOfUnwindCodes: u8,
        FrameRegisterAndOffset: u8,
        AddressOfExceptionHandler: u32,
    }

    const MAX_PDATA_ENTRIES: usize = 32;
    static mut EMU_PDATA: [RUNTIME_FUNCTION; MAX_PDATA_ENTRIES] = [RUNTIME_FUNCTION {
        BeginAddress: 0,
        EndAddress: 0,
        UnwindData: 0,
    }; MAX_PDATA_ENTRIES];
    static mut EMU_XDATA: [UNWIND_INFO; MAX_PDATA_ENTRIES] = [UNWIND_INFO {
        VersionAndFlags: 0,
        PrologSize: 0,
        CountOfUnwindCodes: 0,
        FrameRegisterAndOffset: 0,
        AddressOfExceptionHandler: 0,
    }; MAX_PDATA_ENTRIES];

    #[no_mangle]
    pub unsafe extern "C" fn __mingw_init_ehandler() -> i32 {
        static mut WAS_HERE: i32 = 0;
        let image_base = _GetPEImageBase();
        if WAS_HERE != 0 || image_base.is_null() {
            return WAS_HERE;
        }
        WAS_HERE = 1;
        if !_FindPESectionByName(b".pdata\0".as_ptr()).is_null() {
            return 1;
        }

        let mut e = 0usize;
        while e < MAX_PDATA_ENTRIES {
            let sec = _FindPESectionExec(e);
            if sec.is_null() {
                break;
            }
            EMU_XDATA[e].VersionAndFlags = 9; // UNW_FLAG_EHANDLER | UNW_VERSION
            EMU_XDATA[e].AddressOfExceptionHandler =
                ((__mingw_SEH_error_handler as *const u8).offset_from(image_base)) as u32;
            EMU_PDATA[e].BeginAddress = (*sec).VirtualAddress;
            EMU_PDATA[e].EndAddress = (*sec).VirtualAddress + (*sec).VirtualSize;
            EMU_PDATA[e].UnwindData = (core::ptr::addr_of!(EMU_XDATA[e]) as *const u8)
                .offset_from(image_base) as u32;
            e += 1;
        }
        if e != 0 {
            RtlAddFunctionTable(EMU_PDATA.as_mut_ptr(), e as DWORD, image_base as u64);
        }
        1
    }

    #[no_mangle]
    pub unsafe extern "C" fn __mingw_SEH_error_handler(
        ExceptionRecord: *mut EXCEPTION_RECORD,
        _EstablisherFrame: PVOID,
        _ContextRecord: PVOID,
        _DispatcherContext: PVOID,
    ) -> i32 {
        // EXCEPTION_DISPOSITION: 1 = ContinueSearch, 0 = ContinueExecution,
        // 4 = ExecuteHandler (matching the C's literal codes).
        let mut action = 1i32;
        let code = (*ExceptionRecord).ExceptionCode;
        match code {
            EXCEPTION_ACCESS_VIOLATION => {
                let old = signal(SIGSEGV, sig_dfl());
                if old == sig_ign() {
                    signal(SIGSEGV, sig_ign());
                    action = 0;
                } else if old != sig_dfl() {
                    if let Some(f) = old {
                        f(SIGSEGV);
                    }
                    action = 0;
                } else {
                    action = 4;
                }
            }
            EXCEPTION_ILLEGAL_INSTRUCTION | EXCEPTION_PRIV_INSTRUCTION => {
                let old = signal(SIGILL, sig_dfl());
                if old == sig_ign() {
                    signal(SIGILL, sig_ign());
                    action = 0;
                } else if old != sig_dfl() {
                    if let Some(f) = old {
                        f(SIGILL);
                    }
                    action = 0;
                } else {
                    action = 4;
                }
            }
            EXCEPTION_FLT_INVALID_OPERATION
            | EXCEPTION_FLT_DIVIDE_BY_ZERO
            | EXCEPTION_FLT_DENORMAL_OPERAND
            | EXCEPTION_FLT_OVERFLOW
            | EXCEPTION_FLT_UNDERFLOW
            | EXCEPTION_FLT_INEXACT_RESULT => {
                seh_fpe(true, &mut action);
            }
            EXCEPTION_INT_DIVIDE_BY_ZERO => {
                seh_fpe(false, &mut action);
            }
            EXCEPTION_DATATYPE_MISALIGNMENT
            | EXCEPTION_ARRAY_BOUNDS_EXCEEDED
            | EXCEPTION_FLT_STACK_CHECK
            | EXCEPTION_INT_OVERFLOW
            | EXCEPTION_INVALID_HANDLE => {
                action = 0;
            }
            _ => {}
        }
        action
    }

    #[inline]
    unsafe fn seh_fpe(reset_fpu: bool, action: &mut i32) {
        let old = signal(SIGFPE, sig_dfl());
        if old == sig_ign() {
            signal(SIGFPE, sig_ign());
            if reset_fpu {
                _fpreset();
            }
            *action = 0;
        } else if old != sig_dfl() {
            if let Some(f) = old {
                f(SIGFPE);
            }
            *action = 0;
        }
    }
}
