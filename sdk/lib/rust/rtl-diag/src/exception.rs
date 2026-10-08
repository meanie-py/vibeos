use crate::shim::*;
use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::driver_ddk::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ddk::{CONTEXT, EXCEPTION_RECORD, PCONTEXT, PCSTR, PEXCEPTION_POINTERS, PEXCEPTION_RECORD, PRTLP_UNHANDLED_EXCEPTION_FILTER, PSZ, RtlCaptureContext, RtlDispatchException, RtlWalkFrameChain, ZwContinue, ZwRaiseException, _EXCEPTION_POINTERS};
use core::ffi::c_void;

extern "system" {
    fn RtlPcToFileHeader(PcValue: *mut c_void, BaseOfImage: *mut *mut c_void) -> *mut c_void;
    fn RtlEncodePointer(Ptr: *mut core::ffi::c_void) -> *mut core::ffi::c_void;
}

// ---- file-scope preamble (translated) ----
pub static mut RtlpUnhandledExceptionFilter: PRTLP_UNHANDLED_EXCEPTION_FILTER = None;

// exception.c guards this `#if !defined(_M_IX86) && !defined(_M_AMD64)`:
// i386 + amd64 provide RtlRaiseException in arch asm/except.c; Rust = arm64.
#[cfg(target_arch = "aarch64")]
#[no_mangle]
pub unsafe extern "system" fn RtlRaiseException(ExceptionRecord: PEXCEPTION_RECORD) {
    let mut Context: CONTEXT = core::mem::zeroed();
    let Status: NTSTATUS;

    RtlCaptureContext(&mut Context);

    (*ExceptionRecord).ExceptionAddress = core::ptr::null_mut(); // _ReturnAddress() N/A in stable Rust (matches ntoskrnl-rust)

    Context.ContextFlags = CONTEXT_FULL;

    if RtlpCheckForActiveDebugger() != 0 {
        Status = ZwRaiseException(ExceptionRecord, &mut Context, 1u8);
    } else {
        if RtlDispatchException(ExceptionRecord, &mut Context) == 0 {
            Status = ZwRaiseException(ExceptionRecord, &mut Context, 0u8);
        } else {
            Status = ZwContinue(&mut Context, 0u8);
        }
    }

    RtlRaiseStatus(Status);
}

// exception.c guards this `#if !defined(_M_IX86)`: i386 provides RtlRaiseStatus
// in i386/except.c. Rust = amd64 + arm64.
/// Raises logged so far this boot. Every raise used to print five lines; with Rust's
/// kernel SEH most raises are CAUGHT (an FS driver's raise model, an in-page error), and
/// Wine's storage32 test_locking drove ~4000 of them -- 24k serial lines that alone ran
/// the test past its timeout under TCG. The first RAISE_LOG_FULL are logged in full (the
/// unhandled one that matters is almost always among the first), then 1 in
/// RAISE_LOG_SAMPLE with the running count, so a storm stays visible without drowning
/// the log.
#[cfg(not(target_arch = "x86"))]
static RAISE_LOG_COUNT: core::sync::atomic::AtomicU32 = core::sync::atomic::AtomicU32::new(0);
#[cfg(not(target_arch = "x86"))]
const RAISE_LOG_FULL: u32 = 64;
#[cfg(not(target_arch = "x86"))]
const RAISE_LOG_SAMPLE: u32 = 256;

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
// The C ends the same way (exception.c: "If we returned, raise a status"): the tail call
// is reached only when ZwRaiseException itself FAILS, and re-raises with that failure
// status. rustc cannot know ZwRaiseException does not return, so it sees every path end
// in a self-call; the recursion is real only on a broken raise, as it is in the C.
#[allow(unconditional_recursion)]
pub unsafe extern "system" fn RtlRaiseStatus(mut Status: NTSTATUS) {
    let mut ExceptionRecord: EXCEPTION_RECORD = core::mem::zeroed();
    let mut Context: CONTEXT = core::mem::zeroed();
    let raise_no =
        RAISE_LOG_COUNT.fetch_add(1, core::sync::atomic::Ordering::Relaxed);
    let verbose = raise_no < RAISE_LOG_FULL || raise_no % RAISE_LOG_SAMPLE == 0;

    // _ReturnAddress(): kernel targets build with frame-pointer=always, so the
    // return address is recoverable per-arch. Capture FIRST, before any call.
    // Setting ExceptionAddress=NULL (the old stub) blinded the whole diagnosis
    // chain: an unhandled raise bugchecked 0x1E with P2=0 and no raiser named
    // (2026-07-06, the post-shell-ready C00000D8 hunt).
    // amd64: LLVM's kernel prologue is `lea rbp, [rsp+0x80]` (NOT a classic
    // [saved-rbp][ret] frame), so [rbp+8] is garbage. Scan the stack upward for
    // the first kernel-text-looking qword instead — that's the raise site (or
    // a frame or two above it; symbolicate offline against the banner's bases).
    #[cfg(target_arch = "x86_64")]
    let return_address: usize = {
        let rsp: usize;
        core::arch::asm!("mov {}, rsp", out(reg) rsp, options(nomem, nostack, preserves_flags));
        // Print the first four text-like qwords: the scan starts inside this
        // fn's own (uninitialized) frame, so candidate #1 can be a stale local;
        // four candidates make the raiser unambiguous.
        // The kernel-text window below (0xFFFFF800.. ..0xFFFFF8FF..) also
        // contains kernel STACKS — amd64 kernel stacks sit at 0xFFFFF880`xxxxxxxx,
        // the same range as driver text. So a raw range test happily reports a
        // saved stack POINTER as the raise site, which is how a 2026-07-24 E13
        // triage got "RtlRaiseStatus(0xC0000005) from fffff88004880500" and spent
        // a while trying to symbolicate a stack address as win32k code. Reject
        // anything within 1MB of the current rsp: kernel stacks are 16-24KB, so
        // that window is comfortably inside the stack and cannot swallow a real
        // driver text address.
        let near_stack = |v: usize| -> bool {
            let d = if v > rsp { v - rsp } else { rsp - v };
            d < 0x10_0000
        };
        let mut found: usize = 0;
        let mut printed = 0u32;
        let mut i = 0usize;
        while i < 0x400 && printed < 4 {
            let v = *((rsp + i * 8) as *const usize);
            if v >= 0xFFFF_F800_0000_0000 && v < 0xFFFF_F8FF_FFFF_FFFF && !near_stack(v) {
                if found == 0 {
                    found = v;
                }
                if verbose {
                    DbgPrint(
                        b"[RAISE]   stack[+0x%03lX] = %p\n\0".as_ptr() as PCSTR,
                        (i * 8) as u32,
                        v as *const core::ffi::c_void,
                    );
                }
                printed += 1;
            }
            i += 1;
        }
        found
    };
    #[cfg(target_arch = "aarch64")]
    let return_address: usize = {
        let lr: usize;
        core::arch::asm!("mov {}, x30", out(reg) lr, options(nomem, nostack, preserves_flags));
        lr
    };

    // With DUMMY_PSEH every kernel-mode software raise is fatal (dead except
    // handlers) — name the raiser BEFORE the inevitable 0x1E.
    if verbose {
        DbgPrint(
            b"[RAISE] RtlRaiseStatus(0x%08lX) from %p (raise #%lu)\n\0".as_ptr() as PCSTR,
            Status as u32,
            return_address as *const core::ffi::c_void,
            raise_no + 1,
        );
    }

    // The stack-scan guess above names a stack or data address as often as code (a
    // storage32 FILE_CORRUPT storm reported `KiP0BootStackData`). A real unwind is
    // exact; KeBugCheck already walks this way, so it is safe here. Verbose raises only.
    #[cfg(target_arch = "x86_64")]
    if verbose {
        let mut frames: [*mut core::ffi::c_void; 6] = [core::ptr::null_mut(); 6];
        let n = RtlCaptureStackBackTrace(1, 6, frames.as_mut_ptr(), core::ptr::null_mut());
        let mut k = 0usize;
        while k < n as usize {
            // Image base + RVA + SizeOfImage: the boot log names no driver load bases, and
            // the image size is what identifies WHICH .sys a frame is in.
            let mut base: *mut c_void = core::ptr::null_mut();
            RtlPcToFileHeader(frames[k], &mut base);
            let mut size: u32 = 0;
            if !base.is_null() {
                let nt = ntkrnl_sys::ddk::RtlImageNtHeader(base);
                if !nt.is_null() {
                    size = (*nt).OptionalHeader.SizeOfImage;
                }
            }
            DbgPrint(
                b"[RAISE]   frame %lu = %p (image %p +0x%lX, size 0x%lX)\n\0".as_ptr() as PCSTR,
                k as u32,
                frames[k],
                base,
                (frames[k] as usize).wrapping_sub(base as usize) as u32,
                size,
            );
            k += 1;
        }
    }

    RtlCaptureContext(&mut Context);

    ExceptionRecord.ExceptionAddress = return_address as *mut core::ffi::c_void;
    ExceptionRecord.ExceptionCode = Status;
    ExceptionRecord.ExceptionRecord = core::ptr::null_mut();
    ExceptionRecord.NumberParameters = 0;
    ExceptionRecord.ExceptionFlags = EXCEPTION_NONCONTINUABLE;

    Context.ContextFlags = CONTEXT_FULL;

    if RtlpCheckForActiveDebugger() != 0 {
        ZwRaiseException(&mut ExceptionRecord, &mut Context, 1u8);
    } else {
        RtlDispatchException(&mut ExceptionRecord, &mut Context);

        Status = ZwRaiseException(&mut ExceptionRecord, &mut Context, 0u8);
    }

    RtlRaiseStatus(Status);
}

#[no_mangle]
pub unsafe extern "system" fn RtlCaptureStackBackTrace(
    mut FramesToSkip: u32,
    FramesToCapture: u32,
    BackTrace: *mut *mut core::ffi::c_void,
    BackTraceHash: *mut u32,
) -> u16 {
    let mut Frames: [*mut core::ffi::c_void; 2 * 64] = [core::ptr::null_mut(); 2 * 64];
    let FrameCount: u32;
    let mut Hash: u32 = 0;
    let mut i: u32;

    /* i386 ONLY: a NULL BackTrace has nowhere to store frames. The C reference stored into
     * it unguarded and relied on the caller's SEH to absorb the fault -- RtlStack's
     * TestStackWalk4 deliberately calls RtlCaptureStackBackTrace(0, 5, NULL, NULL) inside
     * _SEH2_TRY. That is the CORRECT behaviour and it still works on amd64/arm64, where the
     * AV is raised and caught (baseline RtlStackKM = pass=35 fail=11); do NOT guard there,
     * or the WS03 branch expecting STATUS_ACCESS_VIOLATION flips to a failure.
     *
     * On i386 that handler is DEAD under USE_DUMMY_PSEH (task #18: kernel-mode frame-based
     * SEH does not catch a driver __except), so the NULL store AVs -> BUGCHECK 0x1E, which
     * ABORTED THE ENTIRE kmtest sweep at test 52 and left 22 tests unevaluated. Bail
     * explicitly here, mirroring the BackTraceHash null-check below and the same fix already
     * applied to RtlWalkFrameChain(NULL, ...). RETIRE THIS cfg WHEN TASK #18 LANDS -- then
     * i386 catches the AV like the other two arches and the guard becomes a divergence.
     * See field manual 9.105. */
    #[cfg(target_arch = "x86")]
    if BackTrace.is_null() {
        return 0;
    }

    /* Skip a frame for the caller */
    FramesToSkip += 1;

    /* Don't go past the limit */
    if (FramesToCapture + FramesToSkip) >= 128 {
        return 0;
    }

    /* Do the back trace */
    FrameCount = RtlWalkFrameChain(Frames.as_mut_ptr(), FramesToCapture + FramesToSkip, 0);

    /* Make sure we're not skipping all of them */
    if FrameCount <= FramesToSkip {
        return 0;
    }

    /* Loop all the frames */
    i = 0;
    while i < FramesToCapture {
        /* Don't go past the limit */
        if (FramesToSkip + i) >= FrameCount {
            break;
        }

        /* Save this entry and hash it */
        *BackTrace.offset(i as isize) = Frames[(FramesToSkip + i) as usize];
        Hash += PtrToUlong(*BackTrace.offset(i as isize));

        i += 1;
    }

    /* Write the hash */
    if !BackTraceHash.is_null() {
        core::ptr::write_volatile(BackTraceHash, Hash);
    }

    /* Clear the other entries and return count */
    RtlFillMemoryUlong(Frames.as_mut_ptr() as *mut core::ffi::c_void, 128, 0);
    i as u16
}

unsafe fn _module_name_from_addr(addr: *const core::ffi::c_void, module_start_addr: *mut *mut core::ffi::c_void, psz: *mut core::ffi::c_char, nChars: usize) -> *const core::ffi::c_char {
    *psz.offset(0) = 0;
    *module_start_addr = core::ptr::null_mut();
    psz
}

unsafe fn _dump_context(pc: PCONTEXT) {
    #[cfg(target_arch = "x86")]
    {
        DbgPrint(b"CS:EIP %x:%x\n\0".as_ptr() as PCSTR, ((*pc).SegCs & 0xffff) as u32, (*pc).Eip);
        DbgPrint(b"DS %x ES %x FS %x GS %x\n\0".as_ptr() as PCSTR, ((*pc).SegDs & 0xffff) as u32, ((*pc).SegEs & 0xffff) as u32, ((*pc).SegFs & 0xffff) as u32, ((*pc).SegGs & 0xfff) as u32);
        DbgPrint(b"EAX: %.8x   EBX: %.8x   ECX: %.8x\n\0".as_ptr() as PCSTR, (*pc).Eax, (*pc).Ebx, (*pc).Ecx);
        DbgPrint(b"EDX: %.8x   EBP: %.8x   ESI: %.8x   ESP: %.8x\n\0".as_ptr() as PCSTR, (*pc).Edx, (*pc).Ebp, (*pc).Esi, (*pc).Esp);
        DbgPrint(b"EDI: %.8x   EFLAGS: %.8x\n\0".as_ptr() as PCSTR, (*pc).Edi, (*pc).EFlags);
    }
    #[cfg(target_arch = "x86_64")]
    {
        DbgPrint(b"CS:RIP %x:%I64x\n\0".as_ptr() as PCSTR, ((*pc).SegCs & 0xffff) as u32, (*pc).Rip);
        DbgPrint(b"DS %x ES %x FS %x GS %x\n\0".as_ptr() as PCSTR, ((*pc).SegDs & 0xffff) as u32, ((*pc).SegEs & 0xffff) as u32, ((*pc).SegFs & 0xffff) as u32, ((*pc).SegGs & 0xfff) as u32);
        DbgPrint(b"RAX: %I64x   RBX: %I64x   RCX: %I64x RDI: %I64x\n\0".as_ptr() as PCSTR, (*pc).Rax, (*pc).Rbx, (*pc).Rcx, (*pc).Rdi);
        DbgPrint(b"RDX: %I64x   RBP: %I64x   RSI: %I64x   RSP: %I64x\n\0".as_ptr() as PCSTR, (*pc).Rdx, (*pc).Rbp, (*pc).Rsi, (*pc).Rsp);
        DbgPrint(b"R8: %I64x   R9: %I64x   R10: %I64x   R11: %I64x\n\0".as_ptr() as PCSTR, (*pc).R8, (*pc).R9, (*pc).R10, (*pc).R11);
        DbgPrint(b"R12: %I64x   R13: %I64x   R14: %I64x   R15: %I64x\n\0".as_ptr() as PCSTR, (*pc).R12, (*pc).R13, (*pc).R14, (*pc).R15);
        DbgPrint(b"EFLAGS: %.8x\n\0".as_ptr() as PCSTR, (*pc).EFlags);
    }
}

unsafe fn PrintStackTrace(ExceptionInfo: *mut _EXCEPTION_POINTERS) {
    let mut StartAddr: *mut core::ffi::c_void = core::ptr::null_mut();
    let mut szMod: [core::ffi::c_char; 128] = [0; 128];
    let ExceptionRecord: PEXCEPTION_RECORD = (*ExceptionInfo).ExceptionRecord;
    let ContextRecord: PCONTEXT = (*ExceptionInfo).ContextRecord;

    DbgPrint(b"Unhandled exception\n\0".as_ptr() as PCSTR);
    DbgPrint(b"ExceptionCode:    %8x\n\0".as_ptr() as PCSTR, (*ExceptionRecord).ExceptionCode);

    if (*ExceptionRecord).ExceptionCode as i32 == STATUS_ACCESS_VIOLATION &&
        (*ExceptionRecord).NumberParameters == 2
    {
        DbgPrint(b"Faulting Address: %8x\n\0".as_ptr() as PCSTR, (*ExceptionRecord).ExceptionInformation[1]);
    }

    if (*ExceptionRecord).ExceptionCode == 0x80000100u32 as i32 &&
        (*ExceptionRecord).NumberParameters == 2
    {
        DbgPrint(
            b"Missing function: %s!%s\n\0".as_ptr() as PCSTR,
            (*ExceptionRecord).ExceptionInformation[0] as PSZ,
            (*ExceptionRecord).ExceptionInformation[1] as PSZ,
        );
    }

    _dump_context(ContextRecord);
    _module_name_from_addr(
        (*ExceptionRecord).ExceptionAddress,
        core::ptr::addr_of_mut!(StartAddr),
        szMod.as_mut_ptr(),
        core::mem::size_of::<[core::ffi::c_char; 128]>(),
    );
    DbgPrint(
        b"Address:\n   %8x+%-8x   %s\n\0".as_ptr() as PCSTR,
        StartAddr,
        ((*ExceptionRecord).ExceptionAddress as usize) - (StartAddr as usize),
        szMod.as_ptr(),
    );

    #[cfg(target_arch = "x86")]
    {
        DbgPrint(b"Frames:\n\0".as_ptr() as PCSTR);

        let mut i: u32 = 0;
        let mut Frame: *mut u32 = (*ContextRecord).Ebp as *mut u32;

        while *Frame.offset(1) != 0 && *Frame.offset(1) != 0xdeadbeef && i < 128 {
            if *Frame.offset(1) == 0 {
                DbgPrint(
                    b"   %8x%9s   %s\n\0".as_ptr() as PCSTR,
                    *Frame.offset(1),
                    b"<invalid address>\0".as_ptr() as PCSTR,
                    b" \0".as_ptr() as PCSTR,
                );
            } else {
                _module_name_from_addr(
                    *Frame.offset(1) as *const core::ffi::c_void,
                    core::ptr::addr_of_mut!(StartAddr),
                    szMod.as_mut_ptr(),
                    core::mem::size_of::<[core::ffi::c_char; 128]>(),
                );
                DbgPrint(
                    b"   %8x+%-8x   %s\n\0".as_ptr() as PCSTR,
                    StartAddr,
                    (*Frame.offset(1) as usize) - (StartAddr as usize),
                    szMod.as_ptr(),
                );
            }

            if *Frame.offset(0) == 0 {
                break;
            }

            Frame = *Frame.offset(0) as *mut u32;
            i += 1;
        }
    }
}

#[no_mangle]
pub unsafe extern "system" fn RtlUnhandledExceptionFilter(
    ExceptionInfo: PEXCEPTION_POINTERS,
) -> LONG {
    RtlUnhandledExceptionFilter2(ExceptionInfo, b"\0".as_ptr() as PCSTR)
}
#[no_mangle]
pub unsafe extern "system" fn RtlUnhandledExceptionFilter2(
    ExceptionInfo: PEXCEPTION_POINTERS,
    Function: PCSTR,
) -> LONG {
    PrintStackTrace(ExceptionInfo);

    if (*(*ExceptionInfo).ExceptionRecord).ExceptionCode == STATUS_POSSIBLE_DEADLOCK {
        return EXCEPTION_CONTINUE_EXECUTION;
    }
    EXCEPTION_CONTINUE_SEARCH
}

#[no_mangle]
pub unsafe extern "system" fn RtlSetUnhandledExceptionFilter(
    TopLevelExceptionFilter: PRTLP_UNHANDLED_EXCEPTION_FILTER,
) {

    RtlpUnhandledExceptionFilter = core::mem::transmute(RtlEncodePointer(
        core::mem::transmute::<_, *mut core::ffi::c_void>(TopLevelExceptionFilter),
    ));
}
