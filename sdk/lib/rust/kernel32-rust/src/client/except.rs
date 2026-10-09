use ntdll_sys::nt::*;
use core::ffi::c_void;
use crate::string::nls_subst::NT_SUCCESS;
use crate::k32_externs::{
    BaseSetLastNTError, BaseRunningInServerProcess, HandleToUlong, RtlZeroMemory,
    RtlCopyMemory, CloseHandle, K32_USER_SHARED_DATA,
};
use crate::proc_create::CreateProcessW;

// MEM_IMAGE (== SEC_IMAGE) is not emitted by bindgen.
const EXCEPTION_NESTED_CALL: ULONG = 0x10;
const SEM_NOGPFAULTERRORBOX: ULONG = 0x0002;
// psdk/winbase.h: RTL_SEM_NOGPFAULTERRORBOX is SEM_NOGPFAULTERRORBOX << 4.
const RTL_SEM_NOGPFAULTERRORBOX: ULONG = SEM_NOGPFAULTERRORBOX << 4;
const MEM_IMAGE: ULONG = 0x1000000;
const IMAGE_DIRECTORY_ENTRY_RESOURCE: USHORT = 2;
const EXCEPTION_CONTINUE_SEARCH: LONG = 0;
const EXCEPTION_CONTINUE_EXECUTION: LONG = -1;
// ntstatus.h; not emitted by the substrate.
const STATUS_UNHANDLED_EXCEPTION: NTSTATUS = 0xC0000144_u32 as NTSTATUS;
const STATUS_USER_APC: NTSTATUS = 0x000000C0;
const STATUS_ALERTED: NTSTATUS = 0x00000101;

// SharedUserData->NtSystemRoot: WCHAR[260] at offset 0x30 of KUSER_SHARED_DATA
// (fixed VA 0x7FFE0000). KUSER_SHARED_DATA is not in bindgen; read the field
// directly (offset verified against <ndk/ketypes.h>).
#[inline]
unsafe fn SharedUserData_NtSystemRoot() -> *const u16 {
    (K32_USER_SHARED_DATA + 0x30) as *const u16
}

// wcslen — not in the substrate.
#[inline]
unsafe fn wcslen(mut s: *const u16) -> usize {
    let mut n = 0usize;
    while *s != 0 {
        s = s.add(1);
        n += 1;
    }
    n
}

extern "C" {
    // strsafe.h StringCchPrintfW is FORCEINLINE (no export) → its body is just
    // `_vsnwprintf(dest, cchDest-1, fmt, args)` + NUL-terminate. Call the CRT
    // _snwprintf directly (variadic, cdecl); StringCchPrintfW_hr() wraps the
    // S_OK/STRSAFE_E_INSUFFICIENT_BUFFER result.
    fn _snwprintf(buffer: *mut u16, count: usize, format: *const u16, ...) -> i32;
}
extern "system" {
    // interlckd.h FORCEINLINE wrapper around the InterlockedExchange intrinsic.
    fn InterlockedExchangePointer(
        Target: *mut *mut core::ffi::c_void,
        Value: *mut core::ffi::c_void,
    ) -> *mut core::ffi::c_void;
}

// StringCchPrintfW result mapping (strsafe.h): S_OK on full write, else
// STRSAFE_E_INSUFFICIENT_BUFFER (0x8007007A). except.c only tests hr >= 0.
const STRSAFE_E_INSUFFICIENT_BUFFER: HRESULT = 0x8007_007A_u32 as HRESULT;
#[inline]
unsafe fn strsafe_printf_hr(written: i32, cch_dest: usize, dest: *mut u16) -> HRESULT {
    if cch_dest == 0 {
        return STRSAFE_E_INSUFFICIENT_BUFFER;
    }
    let cch_max = cch_dest - 1;
    if written < 0 || (written as usize) > cch_max {
        // overflow/truncation: NUL-terminate at the end of the buffer
        *dest.add(cch_max) = 0;
        STRSAFE_E_INSUFFICIENT_BUFFER
    } else {
        if (written as usize) == cch_max {
            *dest.add(cch_max) = 0;
        }
        0 // S_OK
    }
}

// const-fn UTF-16 literal helper (same pattern as client/path.rs, dosdev.rs).
const fn to_w<const N: usize>(s: &str) -> [u16; N] {
    let bytes = s.as_bytes();
    let mut out = [0u16; N];
    let mut i = 0;
    while i < bytes.len() {
        out[i] = bytes[i] as u16;
        i += 1;
    }
    out
}

// L"%s\\System32\\" = 12 chars + NUL
static SYSTEM32_FMT_W: [u16; 13] = to_w("%s\\System32\\");
// L"WinSta0\\Default" = 15 chars + NUL
static WINSTA_DEFAULT_W: [u16; 16] = to_w("WinSta0\\Default");

extern "system" {
    fn VirtualQuery(
        lpAddress: *const core::ffi::c_void,
        lpBuffer: *mut MEMORY_BASIC_INFORMATION,
        dwLength: usize,
    ) -> usize;
    fn GetModuleFileNameA(hModule: HMODULE, lpFilename: *mut i8, nSize: DWORD) -> DWORD;
}
extern "C" {
    fn strrchr(_: *const i8, _: i32) -> *mut i8;
}

// ---- file-scope preamble (translated) ----
pub type LPTOP_LEVEL_EXCEPTION_FILTER =
    Option<unsafe extern "system" fn(*mut EXCEPTION_POINTERS) -> i32>;

// Cross-file referenced by active C (dllmain.c initializes it) → keep the C symbol.
#[no_mangle]
pub static mut GlobalTopLevelExceptionFilter: LPTOP_LEVEL_EXCEPTION_FILTER = None;
pub static mut g_dwLastErrorToBreakOn: DWORD = 0;

/*
 * Private helper function to lookup the module name from a given address.
 * The address can point to anywhere within the module.
 */
unsafe fn _module_name_from_addr(
    addr: *const core::ffi::c_void,
    module_start_addr: *mut *mut core::ffi::c_void,
    psz: *mut i8,
    nChars: usize,
    module_name: *mut *mut i8,
) -> *const i8 {
    let mut mbi: MEMORY_BASIC_INFORMATION = core::mem::zeroed();

    if (nChars > MAXDWORD as usize)
        || (VirtualQuery(
            addr,
            &mut mbi as *mut MEMORY_BASIC_INFORMATION,
            core::mem::size_of::<MEMORY_BASIC_INFORMATION>(),
        ) != core::mem::size_of::<MEMORY_BASIC_INFORMATION>())
        || GetModuleFileNameA(mbi.AllocationBase as HMODULE, psz, nChars as DWORD) == 0
    {
        *psz.offset(0) = 0i8;
        *module_name = psz;
        *module_start_addr = core::ptr::null_mut();
    } else {
        let mut s1: *mut i8 = strrchr(psz, b'\\' as i32);
        let s2: *mut i8 = strrchr(psz, b'/' as i32);
        if !s2.is_null() && s1.is_null() {
            s1 = s2;
        } else if !s1.is_null() && !s2.is_null() && (s1 < s2) {
            s1 = s2;
        }

        if s1.is_null() {
            s1 = psz;
        } else {
            s1 = s1.offset(1);
        }

        *module_name = s1;
        *module_start_addr = mbi.AllocationBase as *mut core::ffi::c_void;
    }
    psz
}

unsafe fn _dump_context(pc: PCONTEXT) {
    #[cfg(target_arch = "x86")]
    {
        DbgPrint("CS:EIP %x:%x\n\0".as_ptr() as *const i8, (*pc).SegCs & 0xffff, (*pc).Eip);
        DbgPrint("DS %x ES %x FS %x GS %x\n\0".as_ptr() as *const i8, (*pc).SegDs & 0xffff, (*pc).SegEs & 0xffff,
                 (*pc).SegFs & 0xffff, (*pc).SegGs & 0xfff);
        DbgPrint("EAX: %.8x   EBX: %.8x   ECX: %.8x\n\0".as_ptr() as *const i8, (*pc).Eax, (*pc).Ebx, (*pc).Ecx);
        DbgPrint("EDX: %.8x   EBP: %.8x   ESI: %.8x   ESP: %.8x\n\0".as_ptr() as *const i8, (*pc).Edx,
                 (*pc).Ebp, (*pc).Esi, (*pc).Esp);
        DbgPrint("EDI: %.8x   EFLAGS: %.8x\n\0".as_ptr() as *const i8, (*pc).Edi, (*pc).EFlags);
    }
    #[cfg(target_arch = "x86_64")]
    {
        DbgPrint("CS:RIP %x:%I64x\n\0".as_ptr() as *const i8, ((*pc).SegCs & 0xffff) as u32, (*pc).Rip);
        DbgPrint("DS %x ES %x FS %x GS %x\n\0".as_ptr() as *const i8, ((*pc).SegDs & 0xffff) as u32, ((*pc).SegEs & 0xffff) as u32,
                 ((*pc).SegFs & 0xffff) as u32, ((*pc).SegGs & 0xfff) as u32);
        DbgPrint("RAX: %I64x   RBX: %I64x   RCX: %I64x RDI: %I64x\n\0".as_ptr() as *const i8, (*pc).Rax, (*pc).Rbx, (*pc).Rcx, (*pc).Rdi);
        DbgPrint("RDX: %I64x   RBP: %I64x   RSI: %I64x   RSP: %I64x\n\0".as_ptr() as *const i8, (*pc).Rdx, (*pc).Rbp, (*pc).Rsi, (*pc).Rsp);
        DbgPrint("R8: %I64x   R9: %I64x   R10: %I64x   R11: %I64x\n\0".as_ptr() as *const i8, (*pc).R8, (*pc).R9, (*pc).R10, (*pc).R11);
        DbgPrint("R12: %I64x   R13: %I64x   R14: %I64x   R15: %I64x\n\0".as_ptr() as *const i8, (*pc).R12, (*pc).R13, (*pc).R14, (*pc).R15);
        DbgPrint("EFLAGS: %.8x\n\0".as_ptr() as *const i8, (*pc).EFlags);
    }
    #[cfg(target_arch = "arm")]
    {
        DbgPrint("PC:  %08lx   LR:  %08lx   SP:  %08lx\n\0".as_ptr() as *const i8, (*pc).Pc, (*pc).Lr, (*pc).Sp);
        DbgPrint("R0:  %08lx   R1:  %08lx   R2:  %08lx   R3:  %08lx\n\0".as_ptr() as *const i8, (*pc).R0, (*pc).R1, (*pc).R2, (*pc).R3);
        DbgPrint("R4:  %08lx   R5:  %08lx   R6:  %08lx   R7:  %08lx\n\0".as_ptr() as *const i8, (*pc).R4, (*pc).R5, (*pc).R6, (*pc).R7);
        DbgPrint("R8:  %08lx   R9:  %08lx   R10: %08lx   R11: %08lx\n\0".as_ptr() as *const i8, (*pc).R8, (*pc).R9, (*pc).R10, (*pc).R11);
        DbgPrint("R12: %08lx   CPSR: %08lx  FPSCR: %08lx\n\0".as_ptr() as *const i8, (*pc).R12, (*pc).Cpsr, (*pc).Fpscr);
    }
    #[cfg(target_arch = "aarch64")]
    {
        // The X0..X28/Fp/Lr GPRs live in the CONTEXT's anonymous union/struct.
        let r = &(*pc).__bindgen_anon_1.__bindgen_anon_1;
        DbgPrint("PC:  %I64x   LR:  %I64x   SP:  %I64x   FP:  %I64x\n\0".as_ptr() as *const i8, (*pc).Pc, r.Lr, (*pc).Sp, r.Fp);
        DbgPrint("X0:  %I64x   X1:  %I64x   X2:  %I64x   X3:  %I64x\n\0".as_ptr() as *const i8, r.X0, r.X1, r.X2, r.X3);
        DbgPrint("X4:  %I64x   X5:  %I64x   X6:  %I64x   X7:  %I64x\n\0".as_ptr() as *const i8, r.X4, r.X5, r.X6, r.X7);
        DbgPrint("X8:  %I64x   X9:  %I64x   X10: %I64x   X11: %I64x\n\0".as_ptr() as *const i8, r.X8, r.X9, r.X10, r.X11);
        DbgPrint("X12: %I64x   X13: %I64x   X14: %I64x   X15: %I64x\n\0".as_ptr() as *const i8, r.X12, r.X13, r.X14, r.X15);
        DbgPrint("X16: %I64x   X17: %I64x   X18: %I64x   CPSR: %08lx\n\0".as_ptr() as *const i8, r.X16, r.X17, r.X18, (*pc).Cpsr);
    }
}

unsafe fn PrintStackTrace(ExceptionInfo: PEXCEPTION_POINTERS) {
    let mut StartAddr: *mut core::ffi::c_void = core::ptr::null_mut();
    let mut szMod: [i8; 128] = [0; 128];
    let mut szModFile: *mut i8 = core::ptr::null_mut();
    let ExceptionRecord: PEXCEPTION_RECORD = (*ExceptionInfo).ExceptionRecord;
    let ContextRecord: PCONTEXT = (*ExceptionInfo).ContextRecord;

    DbgPrint(b"Unhandled exception\n\0".as_ptr() as *const i8);
    DbgPrint(
        b"ExceptionCode:    %8x\n\0".as_ptr() as *const i8,
        (*ExceptionRecord).ExceptionCode,
    );

    if (*ExceptionRecord).ExceptionCode == STATUS_ACCESS_VIOLATION as u32
        && (*ExceptionRecord).NumberParameters == 2
    {
        DbgPrint(
            b"Faulting Address: %p\n\0".as_ptr() as *const i8,
            (*ExceptionRecord).ExceptionInformation[1] as *mut core::ffi::c_void,
        );
    }

    if (*ExceptionRecord).ExceptionCode == 0x80000100
        && (*ExceptionRecord).NumberParameters == 2
    {
        DbgPrint(
            b"Missing function: %s!%s\n\0".as_ptr() as *const i8,
            (*ExceptionRecord).ExceptionInformation[0] as PSZ,
            (*ExceptionRecord).ExceptionInformation[1] as PSZ,
        );
    }

    _dump_context(ContextRecord);
    _module_name_from_addr(
        (*ExceptionRecord).ExceptionAddress as *const core::ffi::c_void,
        &mut StartAddr,
        szMod.as_mut_ptr(),
        core::mem::size_of::<[i8; 128]>(),
        &mut szModFile,
    );
    DbgPrint(
        b"Address:\n<%s:%x> (%s@%x)\n\0".as_ptr() as *const i8,
        szModFile,
        ((*ExceptionRecord).ExceptionAddress as usize).wrapping_sub(StartAddr as usize),
        szMod.as_mut_ptr(),
        StartAddr,
    );

    #[cfg(target_arch = "x86")]
    {
        DbgPrint(b"Frames:\n\0".as_ptr() as *const i8);

        // _SEH2_TRY (DUMMY_PSEH: handler body is dead code)
        {
            let mut i: UINT = 0;
            let mut Frame: *mut ULONG = (*ContextRecord).Ebp as *mut ULONG;

            while *Frame.offset(1) != 0 && *Frame.offset(1) != 0xdeadbeef && i < 128 {
                if IsBadReadPtr((*Frame.offset(1)) as LPCVOID, 4) != 0 {
                    DbgPrint(
                        b"<%s:%x>\n\0".as_ptr() as *const i8,
                        b"[invalid address]\0".as_ptr() as *const i8,
                        *Frame.offset(1),
                    );
                } else {
                    _module_name_from_addr(
                        (*Frame.offset(1)) as *const core::ffi::c_void,
                        &mut StartAddr,
                        szMod.as_mut_ptr(),
                        core::mem::size_of::<[i8; 128]>(),
                        &mut szModFile,
                    );
                    DbgPrint(
                        b"<%s:%x> (%s@%x)\n\0".as_ptr() as *const i8,
                        szModFile,
                        ((*Frame.offset(1)) as usize).wrapping_sub(StartAddr as usize),
                        szMod.as_mut_ptr(),
                        StartAddr,
                    );
                }

                if IsBadReadPtr(
                    (*Frame.offset(0)) as LPCVOID,
                    (core::mem::size_of::<ULONG>() * 2) as UINT_PTR,
                ) != 0
                {
                    break;
                }

                Frame = (*Frame.offset(0)) as *mut ULONG;
                i += 1;
            }
        }
        // _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER) { DbgPrint("<error dumping...>") } _SEH2_END
    }
}

#[no_mangle]
pub unsafe extern "system" fn BasepCheckForReadOnlyResource(Ptr: *mut core::ffi::c_void) -> LONG {
    let mut Data: *mut core::ffi::c_void;
    let mut Size: ULONG = 0;
    let mut OldProtect: ULONG = 0;
    let mut Size2: SIZE_T;
    let mut mbi: MEMORY_BASIC_INFORMATION = core::mem::zeroed();
    let mut Status: NTSTATUS;
    let mut Ptr = Ptr;
    let mut Ret: LONG = EXCEPTION_CONTINUE_SEARCH;

    /* Check if it was an attempt to write to a read-only image section! */
    Status = NtQueryVirtualMemory(
        NtCurrentProcess(),
        Ptr,
        MemoryBasicInformation,
        &mut mbi as *mut MEMORY_BASIC_INFORMATION as *mut core::ffi::c_void,
        core::mem::size_of::<MEMORY_BASIC_INFORMATION>() as SIZE_T,
        core::ptr::null_mut(),
    );
    if NT_SUCCESS(Status) && mbi.Protect == PAGE_READONLY && mbi.Type == MEM_IMAGE {
        /* Attempt to treat it as a resource section. We need to
           use SEH here because we don't know if it's actually a
           resource mapping */
        // _SEH2_TRY
        {
            Data = RtlImageDirectoryEntryToData(
                mbi.AllocationBase,
                1, // TRUE
                IMAGE_DIRECTORY_ENTRY_RESOURCE,
                &mut Size,
            );

            if !Data.is_null()
                && (Ptr as usize) >= (Data as usize)
                && (Ptr as usize) < (Data as usize) + (Size as usize)
            {
                /* The user tried to write into the resources. Make the page writable... */
                Size2 = 1;
                Status = NtProtectVirtualMemory(
                    NtCurrentProcess(),
                    &mut Ptr,
                    &mut Size2,
                    PAGE_READWRITE,
                    &mut OldProtect,
                );
                if NT_SUCCESS(Status) {
                    Ret = EXCEPTION_CONTINUE_EXECUTION;
                }
            }
        }
        // _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER) { } _SEH2_END
    }

    return Ret;
}

#[no_mangle]
pub unsafe extern "system" fn GetErrorMode() -> u32 {
    let mut Status: i32;
    let mut ErrMode: u32 = 0;

    Status = NtQueryInformationProcess(
        NtCurrentProcess(),
        ProcessDefaultHardErrorMode,
        &mut ErrMode as *mut u32 as *mut core::ffi::c_void,
        core::mem::size_of::<u32>() as u32,
        core::ptr::null_mut(),
    );
    if !NT_SUCCESS(Status) {
        BaseSetLastNTError(Status);
        return 0;
    }

    if (ErrMode & SEM_FAILCRITICALERRORS) != 0 {
        ErrMode &= !SEM_FAILCRITICALERRORS;
    } else {
        ErrMode |= SEM_FAILCRITICALERRORS;
    }

    return ErrMode;
}

/// `BaseProcessStartup`/`BaseThreadStartup`'s
/// `__try { start() } __except (UnhandledExceptionFilter(GetExceptionInformation()))
/// { ExitProcess/ExitThread(GetExceptionCode()) }`, for real.
///
/// The translation kept only the `__try` (DUMMY_PSEH), so no Win32 thread had a
/// top-level handler: `UnhandledExceptionFilter` -- and with it
/// `SetUnhandledExceptionFilter`, AeDebug, `SEM_NOGPFAULTERRORBOX` and the fault box --
/// never ran on any arch, and every unhandled exception went second chance and ended as
/// csrss's anonymous `TerminateProcess(0x80)`. On i386 the thread also ran with an
/// empty `fs:[0]` chain, which `_setjmp3` (ucrtbase_apitest setjmp.c) saw as "no frame".
/// ntdll's `RtlSehTryExcept` is a real frame on every arch (native SEH on amd64/arm64,
/// an `fs:[0]` record on i386).
pub(crate) unsafe fn run_under_top_level_handler<F: FnOnce()>(start: F) -> ! {
    let mut state = ();
    let caught = ntdll_sys::seh::try_except(
        &mut state,
        |_| start(),
        |_, pointers| UnhandledExceptionFilter(pointers as PEXCEPTION_POINTERS),
    );
    // `start` ends in ExitThread, so only a handled exception gets here.
    let code = match caught {
        Err(code) => code as u32,
        Ok(()) => 0,
    };
    /* If we are running inside CSRSS, then just quit this thread */
    if BaseRunningInServerProcess == 0 {
        crate::proc_getters::ExitProcess(code);
    }
    crate::k32_externs::ExitThread(code)
}

#[no_mangle]
pub unsafe extern "system" fn UnhandledExceptionFilter(
    ExceptionInfo: PEXCEPTION_POINTERS,
) -> i32 {
    // C: `static UNICODE_STRING AeDebugKey = RTL_CONSTANT_STRING(
    //     L"\\Registry\\Machine\\" REGSTR_PATH_AEDEBUG)`. The translation
    // left it EMPTY, so NtOpenKey never found the AeDebug key and the
    // post-mortem debugger settings were never read.
    let mut AeDebugKey: _UNICODE_STRING = core::mem::zeroed();
    RtlInitUnicodeString(
        &mut AeDebugKey,
        w!("\\Registry\\Machine\\Software\\Microsoft\\Windows NT\\CurrentVersion\\AeDebug").as_ptr(),
    );
    static mut IsSecondChance: u8 = 0;

    let mut Status: i32;
    let ExceptionRecord: PEXCEPTION_RECORD = (*ExceptionInfo).ExceptionRecord;
    let mut RealFilter: LPTOP_LEVEL_EXCEPTION_FILTER;
    let mut RetValue: i32;

    let mut DebugPort: *mut core::ffi::c_void = core::ptr::null_mut();
    let mut ErrorParameters: [usize; 4] = [0; 4];
    let mut DebugResponse: u32;
    let mut ErrorResponse: u32 = 0;

    let mut KeyHandle: *mut core::ffi::c_void = core::ptr::null_mut();
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES;
    let mut ValueString: _UNICODE_STRING = core::mem::zeroed();
    let mut Length: u32 = 0;
    let mut Buffer: [u8; core::mem::size_of::<KEY_VALUE_PARTIAL_INFORMATION>() + 260 * 2] =
        [0; core::mem::size_of::<KEY_VALUE_PARTIAL_INFORMATION>() + 260 * 2];
    let PartialInfo: PKEY_VALUE_PARTIAL_INFORMATION = Buffer.as_mut_ptr() as *mut _;
    let mut AeDebugAuto: u8 = 0;
    let mut AeDebugPath: *mut u16 = core::ptr::null_mut();
    let mut AeDebugCmdLine: [u16; 260] = [0; 260];

    let mut Success: i32;
    let mut hr: i32;
    let mut PrependLength: u32;
    let mut hDebugEvent: *mut core::ffi::c_void = core::ptr::null_mut();
    let mut WaitHandles: [*mut core::ffi::c_void; 2] = [core::ptr::null_mut(); 2];
    let mut StartupInfo: STARTUPINFOW = core::mem::zeroed();
    let mut ProcessInfo: PROCESS_INFORMATION = core::mem::zeroed();

    // C: `ExceptionRecord->ExceptionFlags & EXCEPTION_NESTED_CALL`, 0x10.
    // The literal was 0x4 — EXCEPTION_EXIT_UNWIND — so "in case this is a
    // nested exception, just kill the process" terminated on an ordinary exit
    // unwind and let a genuinely nested exception run on.
    if ((*ExceptionRecord).ExceptionFlags & EXCEPTION_NESTED_CALL) != 0 {
        NtTerminateProcess(
            -1isize as *mut core::ffi::c_void,
            (*ExceptionRecord).ExceptionCode as NTSTATUS,
        );
        return 1;
    }

    if ((*ExceptionRecord).ExceptionCode == 0xC0000005u32)
        && ((*ExceptionRecord).NumberParameters >= 2)
    {
        match (*ExceptionRecord).ExceptionInformation[0] {
            1 => {
                RetValue = BasepCheckForReadOnlyResource(
                    (*ExceptionRecord).ExceptionInformation[1] as *mut core::ffi::c_void,
                );
                if RetValue == -1i32 {
                    return -1i32;
                }
            }
            0 => {}
            _ => {}
        }
    }

    Status = NtQueryInformationProcess(
        -1isize as *mut core::ffi::c_void,
        7i32 as u32,
        &mut DebugPort as *mut *mut core::ffi::c_void as *mut core::ffi::c_void,
        core::mem::size_of::<*mut core::ffi::c_void>() as u32,
        core::ptr::null_mut(),
    );
    // Being debugged: pass the exception on to the debugger. The C returns
    // EXCEPTION_CONTINUE_SEARCH (0); this said -1, EXCEPTION_CONTINUE_EXECUTION,
    // which re-ran the faulting instruction forever under a debugger.
    if (Status as i32 >= 0) && (!DebugPort.is_null()) {
        return EXCEPTION_CONTINUE_SEARCH;
    }

    RealFilter = core::mem::transmute::<*mut core::ffi::c_void, LPTOP_LEVEL_EXCEPTION_FILTER>(
        RtlDecodePointer(core::mem::transmute::<LPTOP_LEVEL_EXCEPTION_FILTER, *mut core::ffi::c_void>(
            GlobalTopLevelExceptionFilter,
        )),
    );
    if let Some(filter) = RealFilter {
        RetValue = filter(ExceptionInfo);
        if RetValue != EXCEPTION_CONTINUE_SEARCH {
            return RetValue;
        }
    }

    PrintStackTrace(ExceptionInfo);

    // C: `(GetErrorMode() & SEM_NOGPFAULTERRORBOX) ||
    //     (RtlGetThreadErrorMode() & RTL_SEM_NOGPFAULTERRORBOX)`, 0x2 and 0x20.
    // BOTH were written as 0x80000001, which is neither — bit 0 is
    // SEM_FAILCRITICALERRORS and bit 31 is not defined at all. So the "caller
    // asked us not to pop up a fault box" check consulted the wrong flag in
    // both the Win32 and the NT per-thread error mode.
    if ((GetErrorMode() & SEM_NOGPFAULTERRORBOX) != 0)
        || ((RtlGetThreadErrorMode() & RTL_SEM_NOGPFAULTERRORBOX) != 0)
    {
        return 1;
    }

    ErrorParameters[0] = (*ExceptionRecord).ExceptionCode as usize;
    ErrorParameters[1] = (*ExceptionRecord).ExceptionAddress as usize;

    if (*ExceptionRecord).ExceptionCode == 0xC0000006u32 {
        ErrorParameters[2] = (*ExceptionRecord).ExceptionInformation[2] as usize;
    } else {
        ErrorParameters[2] = (*ExceptionRecord).ExceptionInformation[0] as usize;
    }

    ErrorParameters[3] = (*ExceptionRecord).ExceptionInformation[1] as usize;

    // C: `DebugResponse = OptionOk` (1); this was 0, OptionAbortRetryIgnore.
    DebugResponse = OptionOk as u32;
    AeDebugAuto = 0;

    ObjectAttributes = _OBJECT_ATTRIBUTES {
        Length: core::mem::size_of::<_OBJECT_ATTRIBUTES>() as u32,
        RootDirectory: core::ptr::null_mut(),
        ObjectName: &raw mut AeDebugKey,
        Attributes: 0x40u32,
        SecurityDescriptor: core::ptr::null_mut(),
        SecurityQualityOfService: core::ptr::null_mut(),
    };
    Status = NtOpenKey(
        &mut KeyHandle,
        0x1u32,
        &mut ObjectAttributes,
    );
    if Status as i32 >= 0 {
        // The value names were a Length-2 / Length-16 string over a single
        // temporary 0u16 (an out-of-bounds read, and never "Auto"/"Debugger").
        RtlInitUnicodeString(&mut ValueString, w!("Auto").as_ptr());
        Status = NtQueryValueKey(
            KeyHandle,
            &mut ValueString,
            2u32,
            PartialInfo as *mut core::ffi::c_void,
            core::mem::size_of_val(&Buffer) as u32,
            &mut Length,
        );
        if (Status as i32 >= 0) && ((*PartialInfo).Type == 1u32) {
            AeDebugAuto = if *((*PartialInfo).Data.as_ptr() as *mut u16) == 0x31u16 { 1 } else { 0 };
        } else {
            AeDebugAuto = 0;
        }

        RtlInitUnicodeString(&mut ValueString, w!("Debugger").as_ptr());
        Status = NtQueryValueKey(
            KeyHandle,
            &mut ValueString,
            2u32,
            PartialInfo as *mut core::ffi::c_void,
            core::mem::size_of_val(&Buffer) as u32,
            &mut Length,
        );
        if (Status as i32 >= 0) && ((*PartialInfo).Type == 1u32) {
            AeDebugPath = (*PartialInfo).Data.as_ptr() as *mut u16;
            while !(*AeDebugPath == 0)
                && ((*AeDebugPath == 0x20u16) || (*AeDebugPath == 0x09u16))
            {
                AeDebugPath = AeDebugPath.offset(1);
            }
            if *AeDebugPath != 0 {
                DebugResponse = OptionOkCancel as u32;
            } else {
                AeDebugPath = core::ptr::null_mut();
            }
        } else {
            AeDebugPath = core::ptr::null_mut();
        }

        NtClose(KeyHandle);
    }

    'Quit: {
    if !(!AeDebugPath.is_null() && (AeDebugAuto != 0)) {
        // C: STATUS_UNHANDLED_EXCEPTION | HARDERROR_OVERRIDE_ERRORMODE. This
        // raised 0xC000008E (STATUS_FLOAT_DIVIDE_BY_ZERO) | 0x40000000, so every
        // application-error box reported a float divide by zero; the option
        // offered on a second chance was OptionAbortRetryIgnore, not OptionOk.
        Status = NtRaiseHardError(
            (STATUS_UNHANDLED_EXCEPTION as u32 | HARDERROR_OVERRIDE_ERRORMODE) as NTSTATUS,
            4,
            0,
            ErrorParameters.as_mut_ptr() as PULONG_PTR,
            if IsSecondChance == 0 { DebugResponse } else { OptionOk as u32 },
            &mut ErrorResponse,
        );
    } else {
        Status = 0;
        ErrorResponse = if !AeDebugPath.is_null() {
            ResponseCancel as u32
        } else {
            ResponseOk as u32
        };
    }

    // "Cancel" is the debug choice. This tested 2 (ResponseAbort).
    if ((Status as i32) < 0) || (ErrorResponse != ResponseCancel as u32) || (IsSecondChance != 0) {
        break 'Quit;
    }

    if BaseRunningInServerProcess != 0 {
        IsSecondChance = 1;
        break 'Quit;
    }

    ObjectAttributes = _OBJECT_ATTRIBUTES {
        Length: core::mem::size_of::<_OBJECT_ATTRIBUTES>() as u32,
        RootDirectory: core::ptr::null_mut(),
        ObjectName: core::ptr::null_mut(),
        Attributes: 0x2u32,
        SecurityDescriptor: core::ptr::null_mut(),
        SecurityQualityOfService: core::ptr::null_mut(),
    };
    Status = NtCreateEvent(
        &mut hDebugEvent,
        0x1F0003u32,
        &mut ObjectAttributes,
        0u32,
        0,
    );
    if (Status as i32) < 0 {
        hDebugEvent = core::ptr::null_mut();
    }

    Success = 0;

    Length = wcslen(AeDebugPath) as u32 + 2 * 10 + 1;

    if (*AeDebugPath != 0x22u16)
        && (RtlDetermineDosPathNameType_U(AeDebugPath) == RtlPathTypeRelative)
    {
        PrependLength = wcslen(SharedUserData_NtSystemRoot()) as u32 + 10;
        if PrependLength + Length <= 260 {
            let cch_dest = (PrependLength + 1) as usize;
            let dest = AeDebugCmdLine.as_mut_ptr();
            let n = _snwprintf(
                dest,
                cch_dest.saturating_sub(1),
                SYSTEM32_FMT_W.as_ptr(),
                SharedUserData_NtSystemRoot(),
            );
            hr = strsafe_printf_hr(n, cch_dest, dest);
            Success = if hr as i32 >= 0 { 1 } else { 0 };
        }
    } else {
        PrependLength = 0;
        if Length <= 260 {
            Success = 1;
        }
    }

    if Success != 0 {
        let cch_dest = Length as usize;
        let dest = AeDebugCmdLine.as_mut_ptr().offset(PrependLength as isize);
        let n = _snwprintf(
            dest,
            cch_dest.saturating_sub(1),
            AeDebugPath,
            HandleToUlong((*NtCurrentTeb()).ClientId.UniqueProcess),
            hDebugEvent,
        );
        hr = strsafe_printf_hr(n, cch_dest, dest);
        Success = if hr as i32 >= 0 { 1 } else { 0 };
    }

    if Success != 0 {
        RtlZeroMemory(
            &mut StartupInfo as *mut STARTUPINFOW as *mut core::ffi::c_void,
            core::mem::size_of::<STARTUPINFOW>() as SIZE_T,
        );
        RtlZeroMemory(
            &mut ProcessInfo as *mut PROCESS_INFORMATION as *mut core::ffi::c_void,
            core::mem::size_of::<PROCESS_INFORMATION>() as SIZE_T,
        );

        StartupInfo.cb = core::mem::size_of::<STARTUPINFOW>() as u32;
        StartupInfo.lpDesktop = WINSTA_DEFAULT_W.as_ptr() as *mut u16;

        Success = CreateProcessW(
            core::ptr::null_mut(),
            AeDebugCmdLine.as_mut_ptr(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            1,
            0,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            &mut StartupInfo,
            &mut ProcessInfo,
        );
    }

    if Success != 0 {
        WaitHandles[0] = hDebugEvent;
        WaitHandles[1] = ProcessInfo.hProcess;

        loop {
            // C: WaitAny, looping on STATUS_ALERTED / STATUS_USER_APC. This
            // waited WaitAll (0) and looped on 0xC00000E1/0xC00000E2.
            Status = NtWaitForMultipleObjects(
                2,
                WaitHandles.as_mut_ptr(),
                WaitAny,
                1,
                core::ptr::null_mut(),
            );
            if !((Status == STATUS_ALERTED) || (Status == STATUS_USER_APC)) {
                break;
            }
        }

        if Status as i32 == 0x00000001i32 as i32 {
            Status = NtQueryInformationProcess(
                -1isize as *mut core::ffi::c_void,
                7i32 as u32,
                &mut DebugPort as *mut *mut core::ffi::c_void as *mut core::ffi::c_void,
                core::mem::size_of::<*mut core::ffi::c_void>() as u32,
                core::ptr::null_mut(),
            );
            if ((Status as i32) < 0) || DebugPort.is_null() {
                IsSecondChance = 1;
            }
        }

        CloseHandle(ProcessInfo.hThread);
        CloseHandle(ProcessInfo.hProcess);

        if !hDebugEvent.is_null() {
            NtClose(hDebugEvent);
        }

        return EXCEPTION_CONTINUE_SEARCH;
    }

    if !hDebugEvent.is_null() {
        NtClose(hDebugEvent);
    }

    IsSecondChance = 1;

    } // end 'Quit block — fall through to the Quit code below

    // Quit:
    /* If this is a second chance exception, kill the process */
    if IsSecondChance != 0 {
        NtTerminateProcess(
            -1isize as *mut core::ffi::c_void,
            (*ExceptionRecord).ExceptionCode as NTSTATUS,
        );
    }

    return 1;
}

#[no_mangle]
pub unsafe extern "system" fn RaiseException(
    dwExceptionCode: DWORD,
    dwExceptionFlags: DWORD,
    nNumberOfArguments: DWORD,
    lpArguments: *const ULONG_PTR,
) {
    let mut ExceptionRecord: EXCEPTION_RECORD = core::mem::zeroed();

    /* Setup the exception record */
    RtlZeroMemory(
        &mut ExceptionRecord as *mut EXCEPTION_RECORD as *mut core::ffi::c_void,
        core::mem::size_of::<EXCEPTION_RECORD>() as SIZE_T,
    );
    ExceptionRecord.ExceptionCode = dwExceptionCode;
    ExceptionRecord.ExceptionRecord = core::ptr::null_mut();
    ExceptionRecord.ExceptionAddress = RaiseException as *mut core::ffi::c_void;
    ExceptionRecord.ExceptionFlags = dwExceptionFlags & EXCEPTION_NONCONTINUABLE;

    /* Check if we have arguments */
    if lpArguments.is_null() {
        /* We don't */
        ExceptionRecord.NumberParameters = 0;
    } else {
        /* We do, normalize the count */
        let mut nNumberOfArguments = nNumberOfArguments;
        if nNumberOfArguments > EXCEPTION_MAXIMUM_PARAMETERS {
            nNumberOfArguments = EXCEPTION_MAXIMUM_PARAMETERS;
        }

        /* Set the count of parameters and copy them */
        ExceptionRecord.NumberParameters = nNumberOfArguments;
        RtlCopyMemory(
            ExceptionRecord.ExceptionInformation.as_mut_ptr() as *mut core::ffi::c_void,
            lpArguments as *const core::ffi::c_void,
            ((nNumberOfArguments as usize) * core::mem::size_of::<ULONG_PTR>()) as SIZE_T,
        );
    }

    /* Better handling of Delphi Exceptions... a ReactOS Hack */
    if dwExceptionCode == 0xeedface || dwExceptionCode == 0xeedfade {
        DbgPrint(
            "Delphi Exception at address: %p\n\0".as_ptr() as *const i8,
            ExceptionRecord.ExceptionInformation[0],
        );
        DbgPrint(
            "Exception-Object: %p\n\0".as_ptr() as *const i8,
            ExceptionRecord.ExceptionInformation[1],
        );
        DbgPrint(
            "Exception text: %lx\n\0".as_ptr() as *const i8,
            ExceptionRecord.ExceptionInformation[2],
        );
    }

    /* Raise the exception */
    RtlRaiseException(&mut ExceptionRecord as *mut EXCEPTION_RECORD);
}

#[no_mangle]
pub unsafe extern "system" fn SetErrorMode(uMode: u32) -> u32 {
    let PrevErrMode: u32;
    let mut NewMode: u32;

    /* Get the previous mode */
    PrevErrMode = GetErrorMode();
    NewMode = uMode;

    /* Check if failing critical errors was requested */
    if (NewMode & SEM_FAILCRITICALERRORS) != 0 {
        /* Mask it out, since the native API works differently */
        NewMode &= !SEM_FAILCRITICALERRORS;
    } else {
        /* OR it if the caller didn't, due to different native semantics */
        NewMode |= SEM_FAILCRITICALERRORS;
    }

    /* Always keep no alignment faults if they were set */
    NewMode |= PrevErrMode & SEM_NOALIGNMENTFAULTEXCEPT;

    /* Set the new mode */
    NtSetInformationProcess(
        NtCurrentProcess(),
        ProcessDefaultHardErrorMode,
        &NewMode as *const u32 as *mut core::ffi::c_void,
        core::mem::size_of::<u32>() as u32,
    );

    /* Return the previous mode */
    return PrevErrMode;
}

#[no_mangle]
pub unsafe extern "system" fn SetUnhandledExceptionFilter(
    lpTopLevelExceptionFilter: LPTOP_LEVEL_EXCEPTION_FILTER,
) -> LPTOP_LEVEL_EXCEPTION_FILTER {
    let EncodedPointer: *mut core::ffi::c_void;
    let EncodedOldPointer: *mut core::ffi::c_void;

    EncodedPointer = RtlEncodePointer(
        core::mem::transmute::<LPTOP_LEVEL_EXCEPTION_FILTER, *mut core::ffi::c_void>(
            lpTopLevelExceptionFilter,
        ),
    );
    EncodedOldPointer = InterlockedExchangePointer(
        &raw mut GlobalTopLevelExceptionFilter as *mut _ as *mut *mut core::ffi::c_void,
        EncodedPointer,
    );
    return core::mem::transmute::<*mut core::ffi::c_void, LPTOP_LEVEL_EXCEPTION_FILTER>(
        RtlDecodePointer(EncodedOldPointer),
    );
}

// The IsBad*Ptr family. The C probes the range inside _SEH2_TRY and returns TRUE from the
// except block. The first translation kept the probes (`*Current = *Current`, `let _ = *Current`:
// non-volatile, so LLVM deleted them) and dropped the handler, so every non-NULL pointer was
// "good": the sdk delayimp apitests' UnProtect helper asked IsBadWritePtr about an import
// name in .rdata, skipped its VirtualProtect, and the write took the process down.
//
// Divergence from the C: instead of touching each page and catching the fault, ask the memory
// manager. A range is good when every region it covers is committed and grants the access.
// The answer is the same for every page that would fault (NOACCESS, reserved, free,
// read-only for a write), except that a PAGE_GUARD page is reported bad without being touched.
// Windows does report it bad too, but its probe consumes the guard, which is how
// IsBadReadPtr on a thread's stack guard page used to break that stack's growth.

const PAGE_READABLE: ULONG = PAGE_READONLY
    | PAGE_READWRITE
    | PAGE_WRITECOPY
    | PAGE_EXECUTE
    | PAGE_EXECUTE_READ
    | PAGE_EXECUTE_READWRITE
    | PAGE_EXECUTE_WRITECOPY;
const PAGE_WRITABLE: ULONG =
    PAGE_READWRITE | PAGE_WRITECOPY | PAGE_EXECUTE_READWRITE | PAGE_EXECUTE_WRITECOPY;

/// The end (exclusive) of the region holding `address` if that region is committed and grants
/// `access` (a PAGE_READABLE/PAGE_WRITABLE mask), else None. `usize::MAX` stands for a region
/// that ends at the top of the address space.
unsafe fn accessible_region_end(address: ULONG_PTR, access: ULONG) -> Option<ULONG_PTR> {
    let mut mbi: MEMORY_BASIC_INFORMATION = core::mem::zeroed();
    if VirtualQuery(
        address as *const c_void,
        &mut mbi,
        core::mem::size_of::<MEMORY_BASIC_INFORMATION>(),
    ) != core::mem::size_of::<MEMORY_BASIC_INFORMATION>()
    {
        return None;
    }
    if mbi.State != MEM_COMMIT
        || mbi.Protect & (PAGE_GUARD | PAGE_NOACCESS) != 0
        || mbi.Protect & access == 0
    {
        return None;
    }
    let base = mbi.BaseAddress as ULONG_PTR;
    Some(base.checked_add(mbi.RegionSize as ULONG_PTR).unwrap_or(ULONG_PTR::MAX))
}

/// TRUE when any byte of `[first, last]` (inclusive, `last >= first`) is not accessible.
unsafe fn range_is_bad(first: ULONG_PTR, last: ULONG_PTR, access: ULONG) -> BOOL {
    let mut current = first;
    loop {
        match accessible_region_end(current, access) {
            None => return TRUE as BOOL,
            Some(end) if end == ULONG_PTR::MAX || end > last => return FALSE as BOOL,
            Some(end) => current = end,
        }
    }
}

#[no_mangle]
pub unsafe extern "system" fn IsBadReadPtr(
    lp: LPCVOID,
    ucb: UINT_PTR,
) -> BOOL {
    /* Quick cases */
    if ucb == 0 {
        return FALSE as BOOL;
    }
    if lp.is_null() {
        return TRUE as BOOL;
    }

    /* Another quick failure case: the range wraps */
    let first = lp as ULONG_PTR;
    let Some(last) = first.checked_add(ucb as ULONG_PTR - 1) else {
        return TRUE as BOOL;
    };

    range_is_bad(first, last, PAGE_READABLE)
}

#[no_mangle]
pub unsafe extern "system" fn IsBadHugeReadPtr(
    lp: LPCVOID,
    ucb: UINT_PTR,
) -> BOOL {
    return IsBadReadPtr(lp, ucb);
}

#[no_mangle]
pub unsafe extern "system" fn IsBadCodePtr(
    lpfn: FARPROC,
) -> BOOL {
    /* Executing has the same privileges as reading */
    return IsBadReadPtr(
        core::mem::transmute::<FARPROC, LPVOID>(lpfn),
        1,
    );
}

#[no_mangle]
pub unsafe extern "system" fn IsBadWritePtr(
    lp: LPVOID,
    ucb: UINT_PTR,
) -> BOOL {
    /* Quick cases */
    if ucb == 0 {
        return FALSE as BOOL;
    }
    if lp.is_null() {
        return TRUE as BOOL;
    }

    /* Another quick failure case: the range wraps */
    let first = lp as ULONG_PTR;
    let Some(last) = first.checked_add(ucb as ULONG_PTR - 1) else {
        return TRUE as BOOL;
    };

    range_is_bad(first, last, PAGE_WRITABLE)
}

#[no_mangle]
pub unsafe extern "system" fn IsBadHugeWritePtr(
    lp: LPVOID,
    ucb: UINT_PTR,
) -> BOOL {
    return IsBadWritePtr(lp, ucb);
}

/// IsBadStringPtrA/W: read characters until a NUL or `max_chars` of them, and report TRUE if
/// any byte read lies outside a readable region. Each region is queried once, before the
/// first character that reaches into it is read.
unsafe fn string_is_bad<T: Copy + PartialEq + Default>(s: *const T, max_chars: UINT_PTR) -> BOOL {
    /* Quick cases */
    if max_chars == 0 {
        return FALSE as BOOL;
    }
    if s.is_null() {
        return TRUE as BOOL;
    }

    let size = core::mem::size_of::<T>() as ULONG_PTR;
    // Bytes [current, readable_end) are known readable.
    let mut readable_end: ULONG_PTR = s as ULONG_PTR;
    let mut current = s;
    for _ in 0..max_chars {
        let first = current as ULONG_PTR;
        let Some(last) = first.checked_add(size - 1) else {
            return TRUE as BOOL;
        };
        // A character may straddle a region boundary; keep extending until it is covered.
        while readable_end <= last {
            match accessible_region_end(readable_end.max(first), PAGE_READABLE) {
                None => return TRUE as BOOL,
                Some(end) => readable_end = end,
            }
            if readable_end == ULONG_PTR::MAX {
                break;
            }
        }
        if current.read_unaligned() == T::default() {
            break;
        }
        current = current.add(1);
    }
    FALSE as BOOL
}

#[no_mangle]
pub unsafe extern "system" fn IsBadStringPtrW(
    lpsz: LPCWSTR,
    ucchMax: UINT_PTR,
) -> BOOL {
    string_is_bad::<WCHAR>(lpsz, ucchMax)
}

#[no_mangle]
pub unsafe extern "system" fn IsBadStringPtrA(
    lpsz: LPCSTR,
    ucchMax: UINT_PTR,
) -> BOOL {
    string_is_bad::<CHAR>(lpsz, ucchMax)
}

#[no_mangle]
pub unsafe extern "system" fn SetLastError(dwErrCode: DWORD) {
    if (g_dwLastErrorToBreakOn != 0) && (g_dwLastErrorToBreakOn == dwErrCode) {
        DbgBreakPoint();
    }
    if (*NtCurrentTeb()).LastErrorValue != dwErrCode {
        (*NtCurrentTeb()).LastErrorValue = dwErrCode;
    }
}

#[no_mangle]
pub unsafe extern "system" fn GetLastError() -> DWORD {
    (*NtCurrentTeb()).LastErrorValue
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:       See COPYING in the top level directory
//  * PROJECT:         ReactOS system libraries
//  * FILE:            dll/win32/kernel32/client/except.c
//  * PURPOSE:         Exception functions
//  * PROGRAMMER:      Ariadne (ariadne@xs4all.nl)
//  *                  Modified from WINE [ Onno Hovers, (onno@stack.urc.tue.nl) ]
//  * UPDATE HISTORY:
//  *                  Created 01/11/98
//  */
//
// /* INCLUDES *******************************************************************/
//
// #include <k32.h>
// #include <strsafe.h>
//
// #define NDEBUG
// #include <debug.h>
//
// /*
//  * Private helper function to lookup the module name from a given address.
//  * The address can point to anywhere within the module.
//  */
// static const char*
// _module_name_from_addr(const void* addr, void **module_start_addr,
//                        char* psz, size_t nChars, char** module_name)
// {
//     MEMORY_BASIC_INFORMATION mbi;
//
//     if ((nChars > MAXDWORD) ||
//         (VirtualQuery(addr, &mbi, sizeof(mbi)) != sizeof(mbi)) ||
//         !GetModuleFileNameA((HMODULE)mbi.AllocationBase, psz, (DWORD)nChars))
//     {
//         psz[0] = '\0';
//         *module_name = psz;
//         *module_start_addr = 0;
//     }
//     else
//     {
//         char* s1 = strrchr(psz, '\\'), *s2 = strrchr(psz, '/');
//         if (s2 && !s1)
//             s1 = s2;
//         else if (s1 && s2 && s1 < s2)
//             s1 = s2;
//
//         if (!s1)
//             s1 = psz;
//         else
//             s1++;
//
//         *module_name = s1;
//         *module_start_addr = (void *)mbi.AllocationBase;
//     }
//     return psz;
// }
//
//
// static VOID
// _dump_context(PCONTEXT pc)
// {
// #ifdef _M_IX86
//     /*
//      * Print out the CPU registers
//      */
//     DbgPrint("CS:EIP %x:%x\n", pc->SegCs&0xffff, pc->Eip );
//     DbgPrint("DS %x ES %x FS %x GS %x\n", pc->SegDs&0xffff, pc->SegEs&0xffff,
//              pc->SegFs&0xffff, pc->SegGs&0xfff);
//     DbgPrint("EAX: %.8x   EBX: %.8x   ECX: %.8x\n", pc->Eax, pc->Ebx, pc->Ecx);
//     DbgPrint("EDX: %.8x   EBP: %.8x   ESI: %.8x   ESP: %.8x\n", pc->Edx,
//              pc->Ebp, pc->Esi, pc->Esp);
//     DbgPrint("EDI: %.8x   EFLAGS: %.8x\n", pc->Edi, pc->EFlags);
// #elif defined(_M_AMD64)
//     DbgPrint("CS:RIP %x:%I64x\n", pc->SegCs&0xffff, pc->Rip );
//     DbgPrint("DS %x ES %x FS %x GS %x\n", pc->SegDs&0xffff, pc->SegEs&0xffff,
//              pc->SegFs&0xffff, pc->SegGs&0xfff);
//     DbgPrint("RAX: %I64x   RBX: %I64x   RCX: %I64x RDI: %I64x\n", pc->Rax, pc->Rbx, pc->Rcx, pc->Rdi);
//     DbgPrint("RDX: %I64x   RBP: %I64x   RSI: %I64x   RSP: %I64x\n", pc->Rdx, pc->Rbp, pc->Rsi, pc->Rsp);
//     DbgPrint("R8: %I64x   R9: %I64x   R10: %I64x   R11: %I64x\n", pc->R8, pc->R9, pc->R10, pc->R11);
//     DbgPrint("R12: %I64x   R13: %I64x   R14: %I64x   R15: %I64x\n", pc->R12, pc->R13, pc->R14, pc->R15);
//     DbgPrint("EFLAGS: %.8x\n", pc->EFlags);
// #elif defined(_M_ARM)
//     DbgPrint("PC:  %08lx   LR:  %08lx   SP:  %08lx\n", pc->Pc, pc->Lr, pc->Sp);
//     DbgPrint("R0:  %08lx   R1:  %08lx   R2:  %08lx   R3:  %08lx\n", pc->R0, pc->R1, pc->R2, pc->R3);
//     DbgPrint("R4:  %08lx   R5:  %08lx   R6:  %08lx   R7:  %08lx\n", pc->R4, pc->R5, pc->R6, pc->R7);
//     DbgPrint("R8:  %08lx   R9:  %08lx   R10: %08lx   R11: %08lx\n", pc->R8, pc->R9, pc->R10, pc->R11);
//     DbgPrint("R12: %08lx   CPSR: %08lx  FPSCR: %08lx\n", pc->R12, pc->Cpsr, pc->Fpscr);
// #elif defined(_M_ARM64)
//     DbgPrint("PC:  %I64x   LR:  %I64x   SP:  %I64x   FP:  %I64x\n", pc->Pc, pc->Lr, pc->Sp, pc->Fp);
//     DbgPrint("X0:  %I64x   X1:  %I64x   X2:  %I64x   X3:  %I64x\n", pc->X0, pc->X1, pc->X2, pc->X3);
//     DbgPrint("X4:  %I64x   X5:  %I64x   X6:  %I64x   X7:  %I64x\n", pc->X4, pc->X5, pc->X6, pc->X7);
//     DbgPrint("X8:  %I64x   X9:  %I64x   X10: %I64x   X11: %I64x\n", pc->X8, pc->X9, pc->X10, pc->X11);
//     DbgPrint("X12: %I64x   X13: %I64x   X14: %I64x   X15: %I64x\n", pc->X12, pc->X13, pc->X14, pc->X15);
//     DbgPrint("X16: %I64x   X17: %I64x   X18: %I64x   CPSR: %08lx\n", pc->X16, pc->X17, pc->X18, pc->Cpsr);
// #else
//     #error "Unknown architecture"
// #endif
// }
//
// static VOID
// PrintStackTrace(IN PEXCEPTION_POINTERS ExceptionInfo)
// {
//     PVOID StartAddr;
//     CHAR szMod[128] = "", *szModFile;
//     PEXCEPTION_RECORD ExceptionRecord = ExceptionInfo->ExceptionRecord;
//     PCONTEXT ContextRecord = ExceptionInfo->ContextRecord;
//
//     /* Print a stack trace */
//     DbgPrint("Unhandled exception\n");
//     DbgPrint("ExceptionCode:    %8x\n", ExceptionRecord->ExceptionCode);
//
//     if (ExceptionRecord->ExceptionCode == STATUS_ACCESS_VIOLATION &&
//         ExceptionRecord->NumberParameters == 2)
//     {
//         DbgPrint("Faulting Address: %p\n", (PVOID)ExceptionRecord->ExceptionInformation[1]);
//     }
//
//     /* Trace the wine special error and show the modulename and functionname */
//     if (ExceptionRecord->ExceptionCode == 0x80000100 /* EXCEPTION_WINE_STUB */ &&
//         ExceptionRecord->NumberParameters == 2)
//     {
//         DbgPrint("Missing function: %s!%s\n", (PSZ)ExceptionRecord->ExceptionInformation[0], (PSZ)ExceptionRecord->ExceptionInformation[1]);
//     }
//
//     _dump_context(ContextRecord);
//     _module_name_from_addr(ExceptionRecord->ExceptionAddress, &StartAddr, szMod, sizeof(szMod), &szModFile);
//     DbgPrint("Address:\n<%s:%x> (%s@%x)\n",
//              szModFile,
//              (ULONG_PTR)ExceptionRecord->ExceptionAddress - (ULONG_PTR)StartAddr,
//              szMod,
//              StartAddr);
// #ifdef _M_IX86
//     DbgPrint("Frames:\n");
//
//     _SEH2_TRY
//     {
//         UINT i;
//         PULONG Frame = (PULONG)ContextRecord->Ebp;
//
//         for (i = 0; Frame[1] != 0 && Frame[1] != 0xdeadbeef && i < 128; i++)
//         {
//             if (IsBadReadPtr((PVOID)Frame[1], 4))
//             {
//                 DbgPrint("<%s:%x>\n", "[invalid address]", Frame[1]);
//             }
//             else
//             {
//                 _module_name_from_addr((const void*)Frame[1], &StartAddr,
//                                        szMod, sizeof(szMod), &szModFile);
//                 DbgPrint("<%s:%x> (%s@%x)\n",
//                          szModFile,
//                          (ULONG_PTR)Frame[1] - (ULONG_PTR)StartAddr,
//                          szMod,
//                          StartAddr);
//             }
//
//             if (IsBadReadPtr((PVOID)Frame[0], sizeof(*Frame) * 2))
//                 break;
//
//             Frame = (PULONG)Frame[0];
//         }
//     }
//     _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//     {
//         DbgPrint("<error dumping stack trace: 0x%x>\n", _SEH2_GetExceptionCode());
//     }
//     _SEH2_END;
// #endif
// }
//
// /* GLOBALS ********************************************************************/
//
// LPTOP_LEVEL_EXCEPTION_FILTER GlobalTopLevelExceptionFilter;
// DWORD g_dwLastErrorToBreakOn;
//
// /* FUNCTIONS ******************************************************************/
//
// LONG
// WINAPI
// BasepCheckForReadOnlyResource(IN PVOID Ptr)
// {
//     PVOID Data;
//     ULONG Size, OldProtect;
//     SIZE_T Size2;
//     MEMORY_BASIC_INFORMATION mbi;
//     NTSTATUS Status;
//     LONG Ret = EXCEPTION_CONTINUE_SEARCH;
//
//     /* Check if it was an attempt to write to a read-only image section! */
//     Status = NtQueryVirtualMemory(NtCurrentProcess(),
//                                   Ptr,
//                                   MemoryBasicInformation,
//                                   &mbi,
//                                   sizeof(mbi),
//                                   NULL);
//     if (NT_SUCCESS(Status) &&
//         mbi.Protect == PAGE_READONLY && mbi.Type == MEM_IMAGE)
//     {
//         /* Attempt to treat it as a resource section. We need to
//            use SEH here because we don't know if it's actually a
//            resource mapping */
//         _SEH2_TRY
//         {
//             Data = RtlImageDirectoryEntryToData(mbi.AllocationBase,
//                                                 TRUE,
//                                                 IMAGE_DIRECTORY_ENTRY_RESOURCE,
//                                                 &Size);
//
//             if (Data != NULL &&
//                 (ULONG_PTR)Ptr >= (ULONG_PTR)Data &&
//                 (ULONG_PTR)Ptr < (ULONG_PTR)Data + Size)
//             {
//                 /* The user tried to write into the resources. Make the page
//                    writable... */
//                 Size2 = 1;
//                 Status = NtProtectVirtualMemory(NtCurrentProcess(),
//                                                 &Ptr,
//                                                 &Size2,
//                                                 PAGE_READWRITE,
//                                                 &OldProtect);
//                 if (NT_SUCCESS(Status))
//                 {
//                     Ret = EXCEPTION_CONTINUE_EXECUTION;
//                 }
//             }
//         }
//         _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//         {
//         }
//         _SEH2_END;
//     }
//
//     return Ret;
// }
//
// UINT
// WINAPI
// GetErrorMode(VOID)
// {
//     NTSTATUS Status;
//     UINT ErrMode;
//
//     /* Query the current setting */
//     Status = NtQueryInformationProcess(NtCurrentProcess(),
//                                        ProcessDefaultHardErrorMode,
//                                        &ErrMode,
//                                        sizeof(ErrMode),
//                                        NULL);
//     if (!NT_SUCCESS(Status))
//     {
//         /* Fail if we couldn't query */
//         BaseSetLastNTError(Status);
//         return 0;
//     }
//
//     /* Check if NOT failing critical errors was requested */
//     if (ErrMode & SEM_FAILCRITICALERRORS)
//     {
//         /* Mask it out, since the native API works differently */
//         ErrMode &= ~SEM_FAILCRITICALERRORS;
//     }
//     else
//     {
//         /* OR it if the caller didn't, due to different native semantics */
//         ErrMode |= SEM_FAILCRITICALERRORS;
//     }
//
//     /* Return the mode */
//     return ErrMode;
// }
//
// /*
//  * @implemented
//  */
// LONG
// WINAPI
// UnhandledExceptionFilter(IN PEXCEPTION_POINTERS ExceptionInfo)
// {
//     static UNICODE_STRING AeDebugKey =
//         RTL_CONSTANT_STRING(L"\\Registry\\Machine\\" REGSTR_PATH_AEDEBUG);
//
//     static BOOLEAN IsSecondChance = FALSE;
//
//     /* Exception data */
//     NTSTATUS Status;
//     PEXCEPTION_RECORD ExceptionRecord = ExceptionInfo->ExceptionRecord;
//     LPTOP_LEVEL_EXCEPTION_FILTER RealFilter;
//     LONG RetValue;
//
//     /* Debugger and hard error parameters */
//     HANDLE DebugPort = NULL;
//     ULONG_PTR ErrorParameters[4];
//     ULONG DebugResponse, ErrorResponse;
//
//     /* Post-Mortem "Auto-Execute" (AE) Debugger registry data */
//     HANDLE KeyHandle;
//     OBJECT_ATTRIBUTES ObjectAttributes;
//     UNICODE_STRING ValueString;
//     ULONG Length;
//     UCHAR Buffer[sizeof(KEY_VALUE_PARTIAL_INFORMATION) + MAX_PATH * sizeof(WCHAR)];
//     PKEY_VALUE_PARTIAL_INFORMATION PartialInfo = (PVOID)Buffer;
//     BOOLEAN AeDebugAuto = FALSE;
//     PWCHAR AeDebugPath = NULL;
//     WCHAR AeDebugCmdLine[MAX_PATH];
//
//     /* Debugger process data */
//     BOOL Success;
//     HRESULT hr;
//     ULONG PrependLength;
//     HANDLE hDebugEvent;
//     HANDLE WaitHandles[2];
//     STARTUPINFOW StartupInfo;
//     PROCESS_INFORMATION ProcessInfo;
//
//     /* In case this is a nested exception, just kill the process */
//     if (ExceptionRecord->ExceptionFlags & EXCEPTION_NESTED_CALL)
//     {
//         NtTerminateProcess(NtCurrentProcess(), ExceptionRecord->ExceptionCode);
//         return EXCEPTION_EXECUTE_HANDLER;
//     }
//
//     if ((ExceptionRecord->ExceptionCode == STATUS_ACCESS_VIOLATION) &&
//         (ExceptionRecord->NumberParameters >= 2))
//     {
//         switch (ExceptionRecord->ExceptionInformation[0])
//         {
//             case EXCEPTION_WRITE_FAULT:
//             {
//                 /*
//                  * Change the protection on some write attempts,
//                  * some InstallShield setups have this bug.
//                  */
//                 RetValue = BasepCheckForReadOnlyResource(
//                     (PVOID)ExceptionRecord->ExceptionInformation[1]);
//                 if (RetValue == EXCEPTION_CONTINUE_EXECUTION)
//                     return EXCEPTION_CONTINUE_EXECUTION;
//                 break;
//             }
//
//             case EXCEPTION_EXECUTE_FAULT:
//                 /* FIXME */
//                 break;
//         }
//     }
//
//     /* If the process is being debugged, pass the exception to the debugger */
//     Status = NtQueryInformationProcess(NtCurrentProcess(),
//                                        ProcessDebugPort,
//                                        &DebugPort,
//                                        sizeof(DebugPort),
//                                        NULL);
//     if (NT_SUCCESS(Status) && DebugPort)
//     {
//         DPRINT("Passing exception to debugger\n");
//         return EXCEPTION_CONTINUE_SEARCH;
//     }
//
//     /* No debugger present, let's continue... */
//
//     RealFilter = RtlDecodePointer(GlobalTopLevelExceptionFilter);
//     if (RealFilter)
//     {
//         RetValue = RealFilter(ExceptionInfo);
//         if (RetValue != EXCEPTION_CONTINUE_SEARCH)
//             return RetValue;
//     }
//
//     /* ReactOS-specific: DPRINT a stack trace */
//     PrintStackTrace(ExceptionInfo);
//
//     /*
//      * Now pop up an error if needed. Check both the process-wide (Win32)
//      * and per-thread error-mode flags (NT).
//      */
//     if ((GetErrorMode() & SEM_NOGPFAULTERRORBOX) ||
//         (RtlGetThreadErrorMode() & RTL_SEM_NOGPFAULTERRORBOX))
//     {
//         /* Do not display the pop-up error box, just transfer control to the exception handler */
//         return EXCEPTION_EXECUTE_HANDLER;
//     }
//
//     /* Save exception code and address */
//     ErrorParameters[0] = (ULONG_PTR)ExceptionRecord->ExceptionCode;
//     ErrorParameters[1] = (ULONG_PTR)ExceptionRecord->ExceptionAddress;
//
//     if (ExceptionRecord->ExceptionCode == STATUS_IN_PAGE_ERROR)
//     {
//         /*
//          * Get the underlying status code that resulted in the exception,
//          * and just forget about the type of operation (read/write).
//          */
//         ErrorParameters[2] = ExceptionRecord->ExceptionInformation[2];
//     }
//     else // if (ExceptionRecord->ExceptionCode == STATUS_ACCESS_VIOLATION) or others...
//     {
//         /* Get the type of operation that caused the access violation */
//         ErrorParameters[2] = ExceptionRecord->ExceptionInformation[0];
//     }
//
//     /* Save faulting address */
//     ErrorParameters[3] = ExceptionRecord->ExceptionInformation[1];
//
//     /*
//      * Prepare the hard error dialog: default to only OK
//      * in case we do not have any debugger at our disposal.
//      */
//     DebugResponse = OptionOk;
//     AeDebugAuto = FALSE;
//
//     /*
//      * Retrieve Post-Mortem Debugger settings from the registry, under:
//      * HKEY_LOCAL_MACHINE\SOFTWARE\Microsoft\Windows NT\CurrentVersion\AeDebug
//      * (REGSTR_PATH_AEDEBUG).
//      */
//
//     InitializeObjectAttributes(&ObjectAttributes,
//                                &AeDebugKey,
//                                OBJ_CASE_INSENSITIVE,
//                                NULL,
//                                NULL);
//     Status = NtOpenKey(&KeyHandle, KEY_QUERY_VALUE, &ObjectAttributes);
//     if (NT_SUCCESS(Status))
//     {
//         /*
//          * Read the 'Auto' REG_SZ value:
//          * "0" (or any other value): Prompt the user for starting the debugger.
//          * "1": Start the debugger without prompting.
//          */
//         RtlInitUnicodeString(&ValueString, REGSTR_VAL_AEDEBUG_AUTO);
//         Status = NtQueryValueKey(KeyHandle,
//                                  &ValueString,
//                                  KeyValuePartialInformation,
//                                  PartialInfo,
//                                  sizeof(Buffer),
//                                  &Length);
//         if (NT_SUCCESS(Status) && (PartialInfo->Type == REG_SZ))
//         {
//             AeDebugAuto = (*(PWCHAR)PartialInfo->Data == L'1');
//         }
//         else
//         {
//             AeDebugAuto = FALSE;
//         }
//
//         /*
//          * Read and store the 'Debugger' REG_SZ value. Its usual format is:
//          *    C:\dbgtools\ntsd.exe -p %ld -e %ld -g
//          * with the first and second parameters being respectively
//          * the process ID and a notification event handle.
//          */
//         RtlInitUnicodeString(&ValueString, REGSTR_VAL_AEDEBUG_DEBUGGER);
//         Status = NtQueryValueKey(KeyHandle,
//                                  &ValueString,
//                                  KeyValuePartialInformation,
//                                  PartialInfo,
//                                  sizeof(Buffer),
//                                  &Length);
//         if (NT_SUCCESS(Status) && (PartialInfo->Type == REG_SZ))
//         {
//             /* We hope the string is NULL-terminated */
//             AeDebugPath = (PWCHAR)PartialInfo->Data;
//
//             /* Skip any prepended whitespace */
//             while (  *AeDebugPath &&
//                    ((*AeDebugPath == L' ') ||
//                     (*AeDebugPath == L'\t')) ) // iswspace(*AeDebugPath)
//             {
//                 ++AeDebugPath;
//             }
//
//             if (*AeDebugPath)
//             {
//                 /* We have a debugger path, we can prompt the user to debug the program */
//                 DebugResponse = OptionOkCancel;
//             }
//             else
//             {
//                 /* We actually do not have anything, reset the pointer */
//                 AeDebugPath = NULL;
//             }
//         }
//         else
//         {
//             AeDebugPath = NULL;
//         }
//
//         NtClose(KeyHandle);
//     }
//
//     // TODO: Start a ReactOS Fault Reporter (unimplemented!)
//     //
//     // For now we are doing the "old way" (aka Win2k), that is also the fallback
//     // case for Windows XP/2003 in case it does not find faultrep.dll to display
//     // the nice "Application Error" dialog box: We use a hard error to communicate
//     // the problem and prompt the user to continue debugging the application or
//     // to terminate it.
//     //
//     // Since Windows XP/2003, we have the ReportFault API available.
//     // See http://www.clausbrod.de/twiki/pub/Blog/DefinePrivatePublic20070616/reportfault.cpp
//     // and https://learn.microsoft.com/en-us/windows/win32/wer/using-wer
//     // and the legacy ReportFault API: https://learn.microsoft.com/en-us/windows/win32/api/errorrep/nf-errorrep-reportfault
//     //
//     // NOTE: Starting Vista+, the fault API is constituted of the WerXXX functions.
//     //
//     // Also see Vostokov's book "Memory Dump Analysis Anthology Collector's Edition, Volume 1"
//     // at: https://books.google.fr/books?id=9w2x6NHljg4C&pg=PA115&lpg=PA115
//
//     if (!(AeDebugPath && AeDebugAuto))
//     {
//         /*
//          * Either there is no debugger specified, or the debugger should
//          * not start automatically: display the hard error no matter what.
//          * For a first chance exception, allow continuing or debugging;
//          * for a second chance exception, just debug or kill the process.
//          */
//         Status = NtRaiseHardError(STATUS_UNHANDLED_EXCEPTION | HARDERROR_OVERRIDE_ERRORMODE,
//                                   4,
//                                   0,
//                                   ErrorParameters,
//                                   (!IsSecondChance ? DebugResponse : OptionOk),
//                                   &ErrorResponse);
//     }
//     else
//     {
//         Status = STATUS_SUCCESS;
//         ErrorResponse = (AeDebugPath ? ResponseCancel : ResponseOk);
//     }
//
//     /*
//      * If the user has chosen not to debug the process, or
//      * if this is a second chance exception, kill the process.
//      */
//     if (!NT_SUCCESS(Status) || (ErrorResponse != ResponseCancel) || IsSecondChance)
//         break 'Quit;
//
//     /* If the exception comes from a CSR Server, kill it (this will lead to ReactOS shutdown) */
//     if (BaseRunningInServerProcess)
//     {
//         IsSecondChance = TRUE;
//         break 'Quit;
//     }
//
//     /*
//      * Attach a debugger to this process. The debugger process is given
//      * the process ID of the current process to be debugged, as well as
//      * a notification event handle. After being spawned, the debugger
//      * initializes and attaches to the process by calling DebugActiveProcess.
//      * When the debugger is ready, it signals the notification event,
//      * so that we can give it control over the process being debugged,
//      * by passing it the exception.
//      *
//      * See https://learn.microsoft.com/en-us/previous-versions/ms809754(v=msdn.10)
//      * and http://www.debuginfo.com/articles/ntsdwatson.html
//      * and https://sourceware.org/ml/gdb-patches/2012-08/msg00893.html
//      * for more details.
//      */
//
//     /* Create an inheritable notification debug event for the debugger */
//     InitializeObjectAttributes(&ObjectAttributes,
//                                NULL,
//                                OBJ_INHERIT,
//                                NULL,
//                                NULL);
//     Status = NtCreateEvent(&hDebugEvent,
//                            EVENT_ALL_ACCESS,
//                            &ObjectAttributes,
//                            NotificationEvent,
//                            FALSE);
//     if (!NT_SUCCESS(Status))
//         hDebugEvent = NULL;
//
//     /* Build the debugger command line */
//
//     Success = FALSE;
//
//     /*
//      * We will add two longs (process ID and event handle) to the command
//      * line. The biggest 32-bit unsigned int (0xFFFFFFFF == 4.294.967.295)
//      * takes 10 decimal digits. We then count the terminating NULL.
//      */
//     Length = (ULONG)wcslen(AeDebugPath) + 2*10 + 1;
//
//     /* Check whether the debugger path may be a relative path */
//     if ((*AeDebugPath != L'"') &&
//         (RtlDetermineDosPathNameType_U(AeDebugPath) == RtlPathTypeRelative))
//     {
//         /* Relative path, prepend SystemRoot\System32 */
//         PrependLength = (ULONG)wcslen(SharedUserData->NtSystemRoot) + 10 /* == wcslen(L"\\System32\\") */;
//         if (PrependLength + Length <= ARRAYSIZE(AeDebugCmdLine))
//         {
//             hr = StringCchPrintfW(AeDebugCmdLine,
//                                   PrependLength + 1,
//                                   L"%s\\System32\\",
//                                   SharedUserData->NtSystemRoot);
//             Success = SUCCEEDED(hr);
//         }
//     }
//     else
//     {
//         /* Full path */
//         PrependLength = 0;
//         if (Length <= ARRAYSIZE(AeDebugCmdLine))
//             Success = TRUE;
//     }
//
//     /* Format the command line */
//     if (Success)
//     {
//         hr = StringCchPrintfW(&AeDebugCmdLine[PrependLength],
//                               Length,
//                               AeDebugPath,
//                               HandleToUlong(NtCurrentTeb()->ClientId.UniqueProcess), // GetCurrentProcessId()
//                               hDebugEvent);
//         Success = SUCCEEDED(hr);
//     }
//
//     /* Start the debugger */
//     if (Success)
//     {
//         DPRINT1("\nStarting debugger: '%S'\n", AeDebugCmdLine);
//
//         RtlZeroMemory(&StartupInfo, sizeof(StartupInfo));
//         RtlZeroMemory(&ProcessInfo, sizeof(ProcessInfo));
//
//         StartupInfo.cb = sizeof(StartupInfo);
//         StartupInfo.lpDesktop = L"WinSta0\\Default";
//
//         Success = CreateProcessW(NULL,
//                                  AeDebugCmdLine,
//                                  NULL, NULL,
//                                  TRUE, 0,
//                                  NULL, NULL,
//                                  &StartupInfo, &ProcessInfo);
//     }
//
//     if (Success)
//     {
//         WaitHandles[0] = hDebugEvent;
//         WaitHandles[1] = ProcessInfo.hProcess;
//
//         /* Loop until the debugger gets ready or terminates unexpectedly */
//         do
//         {
//             /* Alertable wait */
//             Status = NtWaitForMultipleObjects(ARRAYSIZE(WaitHandles),
//                                               WaitHandles,
//                                               WaitAny,
//                                               TRUE, NULL);
//         } while ((Status == STATUS_ALERTED) || (Status == STATUS_USER_APC));
//
//         /*
//          * The debugger terminated unexpectedly and we cannot attach to it.
//          * Kill the process being debugged.
//          */
//         if (Status == STATUS_WAIT_1)
//         {
//             /* Be sure there is no other debugger attached */
//             Status = NtQueryInformationProcess(NtCurrentProcess(),
//                                                ProcessDebugPort,
//                                                &DebugPort,
//                                                sizeof(DebugPort),
//                                                NULL);
//             if (!NT_SUCCESS(Status) || !DebugPort)
//             {
//                 /* No debugger is attached, kill the process at next round */
//                 IsSecondChance = TRUE;
//             }
//         }
//
//         CloseHandle(ProcessInfo.hThread);
//         CloseHandle(ProcessInfo.hProcess);
//
//         if (hDebugEvent)
//             NtClose(hDebugEvent);
//
//         return EXCEPTION_CONTINUE_SEARCH;
//     }
//
//     /* We failed starting the debugger, close the event handle and kill the process */
//
//     if (hDebugEvent)
//         NtClose(hDebugEvent);
//
//     IsSecondChance = TRUE;
//
//
// Quit:
//     /* If this is a second chance exception, kill the process */
//     if (IsSecondChance)
//         NtTerminateProcess(NtCurrentProcess(), ExceptionRecord->ExceptionCode);
//
//     /* Otherwise allow handling exceptions in first chance */
//
//     /*
//      * Returning EXCEPTION_EXECUTE_HANDLER means that the code in
//      * the __except block will be executed. Normally this will end up in a
//      * Terminate process.
//      */
//
//     return EXCEPTION_EXECUTE_HANDLER;
// }
//
// /*
//  * @implemented
//  */
// VOID
// WINAPI
// RaiseException(
//      DWORD dwExceptionCode,
//      DWORD dwExceptionFlags,
//      DWORD nNumberOfArguments,
//      const ULONG_PTR *lpArguments)
// {
//     EXCEPTION_RECORD ExceptionRecord;
//
//     /* Setup the exception record */
//     RtlZeroMemory(&ExceptionRecord, sizeof(ExceptionRecord));
//     ExceptionRecord.ExceptionCode = dwExceptionCode;
//     ExceptionRecord.ExceptionRecord = NULL;
//     ExceptionRecord.ExceptionAddress = (PVOID)RaiseException;
//     ExceptionRecord.ExceptionFlags = dwExceptionFlags & EXCEPTION_NONCONTINUABLE;
//
//     /* Check if we have arguments */
//     if (!lpArguments)
//     {
//         /* We don't */
//         ExceptionRecord.NumberParameters = 0;
//     }
//     else
//     {
//         /* We do, normalize the count */
//         if (nNumberOfArguments > EXCEPTION_MAXIMUM_PARAMETERS)
//             nNumberOfArguments = EXCEPTION_MAXIMUM_PARAMETERS;
//
//         /* Set the count of parameters and copy them */
//         ExceptionRecord.NumberParameters = nNumberOfArguments;
//         RtlCopyMemory(ExceptionRecord.ExceptionInformation,
//                       lpArguments,
//                       nNumberOfArguments * sizeof(ULONG_PTR));
//     }
//
//     /* Better handling of Delphi Exceptions... a ReactOS Hack */
//     if (dwExceptionCode == 0xeedface || dwExceptionCode == 0xeedfade)
//     {
//         DPRINT1("Delphi Exception at address: %p\n", ExceptionRecord.ExceptionInformation[0]);
//         DPRINT1("Exception-Object: %p\n", ExceptionRecord.ExceptionInformation[1]);
//         DPRINT1("Exception text: %lx\n", ExceptionRecord.ExceptionInformation[2]);
//     }
//
//     /* Raise the exception */
//     RtlRaiseException(&ExceptionRecord);
// }
//
// /*
//  * @implemented
//  */
// UINT
// WINAPI
// SetErrorMode(IN UINT uMode)
// {
//     UINT PrevErrMode, NewMode;
//
//     /* Get the previous mode */
//     PrevErrMode = GetErrorMode();
//     NewMode = uMode;
//
//     /* Check if failing critical errors was requested */
//     if (NewMode & SEM_FAILCRITICALERRORS)
//     {
//         /* Mask it out, since the native API works differently */
//         NewMode &= ~SEM_FAILCRITICALERRORS;
//     }
//     else
//     {
//         /* OR it if the caller didn't, due to different native semantics */
//         NewMode |= SEM_FAILCRITICALERRORS;
//     }
//
//     /* Always keep no alignment faults if they were set */
//     NewMode |= (PrevErrMode & SEM_NOALIGNMENTFAULTEXCEPT);
//
//     /* Set the new mode */
//     NtSetInformationProcess(NtCurrentProcess(),
//                             ProcessDefaultHardErrorMode,
//                             (PVOID)&NewMode,
//                             sizeof(NewMode));
//
//     /* Return the previous mode */
//     return PrevErrMode;
// }
//
// /*
//  * @implemented
//  */
// LPTOP_LEVEL_EXCEPTION_FILTER
// WINAPI
// DECLSPEC_HOTPATCH
// SetUnhandledExceptionFilter(IN LPTOP_LEVEL_EXCEPTION_FILTER lpTopLevelExceptionFilter)
// {
//     PVOID EncodedPointer, EncodedOldPointer;
//
//     EncodedPointer = RtlEncodePointer(lpTopLevelExceptionFilter);
//     EncodedOldPointer = InterlockedExchangePointer((PVOID*)&GlobalTopLevelExceptionFilter,
//                                             EncodedPointer);
//     return RtlDecodePointer(EncodedOldPointer);
// }
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// IsBadReadPtr(IN LPCVOID lp,
//              IN UINT_PTR ucb)
// {
//     ULONG PageSize;
//     BOOLEAN Result = FALSE;
//     volatile CHAR *Current;
//     PCHAR Last;
//
//     /* Quick cases */
//     if (!ucb) return FALSE;
//     if (!lp) return TRUE;
//
//     /* Get the page size */
//     PageSize = BaseStaticServerData->SysInfo.PageSize;
//
//     /* Calculate start and end */
//     Current = (volatile CHAR*)lp;
//     Last = (PCHAR)((ULONG_PTR)lp + ucb - 1);
//
//     /* Another quick failure case */
//     if (Last < Current) return TRUE;
//
//     /* Enter SEH */
//     _SEH2_TRY
//     {
//         /* Do an initial probe */
//         *Current;
//
//         /* Align the addresses */
//         Current = (volatile CHAR *)ALIGN_DOWN_POINTER_BY(Current, PageSize);
//         Last = (PCHAR)ALIGN_DOWN_POINTER_BY(Last, PageSize);
//
//         /* Probe the entire range */
//         while (Current != Last)
//         {
//             Current += PageSize;
//             *Current;
//         }
//     }
//     _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//     {
//         /* We hit an exception, so return true */
//         Result = TRUE;
//     }
//     _SEH2_END
//
//     /* Return exception status */
//     return Result;
// }
//
// /*
//  * @implemented
//  */
// BOOL
// NTAPI
// IsBadHugeReadPtr(LPCVOID lp,
//                  UINT_PTR ucb)
// {
//     /* Implementation is the same on 32-bit */
//     return IsBadReadPtr(lp, ucb);
// }
//
// /*
//  * @implemented
//  */
// BOOL
// NTAPI
// IsBadCodePtr(FARPROC lpfn)
// {
//     /* Executing has the same privileges as reading */
//     return IsBadReadPtr((LPVOID)lpfn, 1);
// }
//
// /*
//  * @implemented
//  */
// BOOL
// NTAPI
// IsBadWritePtr(IN LPVOID lp,
//               IN UINT_PTR ucb)
// {
//     ULONG PageSize;
//     BOOLEAN Result = FALSE;
//     volatile CHAR *Current;
//     PCHAR Last;
//
//     /* Quick cases */
//     if (!ucb) return FALSE;
//     if (!lp) return TRUE;
//
//     /* Get the page size */
//     PageSize = BaseStaticServerData->SysInfo.PageSize;
//
//     /* Calculate start and end */
//     Current = (volatile CHAR*)lp;
//     Last = (PCHAR)((ULONG_PTR)lp + ucb - 1);
//
//     /* Another quick failure case */
//     if (Last < Current) return TRUE;
//
//     /* Enter SEH */
//     _SEH2_TRY
//     {
//         /* Do an initial probe */
//         *Current = *Current;
//
//         /* Align the addresses */
//         Current = (volatile CHAR *)ALIGN_DOWN_POINTER_BY(Current, PageSize);
//         Last = (PCHAR)ALIGN_DOWN_POINTER_BY(Last, PageSize);
//
//         /* Probe the entire range */
//         while (Current != Last)
//         {
//             Current += PageSize;
//             *Current = *Current;
//         }
//     }
//     _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//     {
//         /* We hit an exception, so return true */
//         Result = TRUE;
//     }
//     _SEH2_END
//
//     /* Return exception status */
//     return Result;
// }
//
// /*
//  * @implemented
//  */
// BOOL
// NTAPI
// IsBadHugeWritePtr(IN LPVOID lp,
//                   IN UINT_PTR ucb)
// {
//     /* Implementation is the same on 32-bit */
//     return IsBadWritePtr(lp, ucb);
// }
//
// /*
//  * @implemented
//  */
// BOOL
// NTAPI
// IsBadStringPtrW(IN LPCWSTR lpsz,
//                 IN UINT_PTR ucchMax)
// {
//     BOOLEAN Result = FALSE;
//     volatile WCHAR *Current;
//     PWCHAR Last;
//     WCHAR Char;
//
//     /* Quick cases */
//     if (!ucchMax) return FALSE;
//     if (!lpsz) return TRUE;
//
//     /* Calculate start and end */
//     Current = (volatile WCHAR*)lpsz;
//     Last = (PWCHAR)((ULONG_PTR)lpsz + (ucchMax * 2) - 2);
//
//     /* Enter SEH */
//     _SEH2_TRY
//     {
//         /* Probe the entire range */
//         Char = *Current++;
//         while ((Char) && (Current != Last)) Char = *Current++;
//     }
//     _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//     {
//         /* We hit an exception, so return true */
//         Result = TRUE;
//     }
//     _SEH2_END
//
//     /* Return exception status */
//     return Result;
// }
//
// /*
//  * @implemented
//  */
// BOOL
// NTAPI
// IsBadStringPtrA(IN LPCSTR lpsz,
//                 IN UINT_PTR ucchMax)
// {
//     BOOLEAN Result = FALSE;
//     volatile CHAR *Current;
//     PCHAR Last;
//     CHAR Char;
//
//     /* Quick cases */
//     if (!ucchMax) return FALSE;
//     if (!lpsz) return TRUE;
//
//     /* Calculate start and end */
//     Current = (volatile CHAR*)lpsz;
//     Last = (PCHAR)((ULONG_PTR)lpsz + ucchMax - 1);
//
//     /* Enter SEH */
//     _SEH2_TRY
//     {
//         /* Probe the entire range */
//         Char = *Current++;
//         while ((Char) && (Current != Last)) Char = *Current++;
//     }
//     _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//     {
//         /* We hit an exception, so return true */
//         Result = TRUE;
//     }
//     _SEH2_END
//
//     /* Return exception status */
//     return Result;
// }
//
// /*
//  * @implemented
//  */
// VOID
// WINAPI
// SetLastError(IN DWORD dwErrCode)
// {
//     /* Break if a debugger requested checking for this error code */
//     if ((g_dwLastErrorToBreakOn) && (g_dwLastErrorToBreakOn == dwErrCode)) DbgBreakPoint();
//
//     /* Set last error if it's a new error */
//     if (NtCurrentTeb()->LastErrorValue != dwErrCode) NtCurrentTeb()->LastErrorValue = dwErrCode;
// }
//
// /*
//  * @implemented
//  */
// DWORD
// WINAPI
// GetLastError(VOID)
// {
//     /* Return the current value */
//     return NtCurrentTeb()->LastErrorValue;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
