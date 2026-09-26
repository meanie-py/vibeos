use crate::shim::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::ddk::{KD_SYMBOLS_INFO, NtQueryDebugFilterState, NtSetDebugFilterState, PCCH, PCH, PSTRING, PTEB_ACTIVE_FRAME, RtlImageNtHeader, STRING, va_list};

extern "C" {
    fn strlen(s: PCCH) -> usize;
    fn strncpy(dst: PCH, src: PCCH, n: usize) -> PCH;
    // The C `ntprintf_va_format_a` is a WEAK symbol (non-null only in ntoskrnl;
    // null in ntdll → falls back to _vsnprintf). Stable Rust can't declare weak
    // externs, so that dispatch lives in the C shim (dbgprint_variadic.c) and we
    // call this single non-weak (cdecl) entry point. Weak resolution happens at
    // each consumer's final link, exactly as before.
    fn rtldiag_format_va(dst: PCH, cap: usize, fmt: PCCH, ap: va_list) -> usize;
}
extern "system" {
    // DebugService2 is NTAPI (asm: _DebugService2@12), not cdecl.
    fn DebugService2(Arg1: PVOID, Arg2: PVOID, Service: ULONG) -> ULONG;
}

// ---- file-scope preamble (translated) ----
pub const DBG_PRINTEXCEPTION_C: u32 = 0x40010006;
pub const DBG_STATUS_CONTROL_C: u32 = 1;

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
unsafe fn _DbgPortRead(Port: USHORT) -> UCHAR {
    let Value: UCHAR;
    core::arch::asm!(
        "in al, dx",
        out("al") Value,
        in("dx") Port,
        options(nomem, nostack, preserves_flags)
    );
    Value
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
unsafe fn _DbgPortWrite(Port: USHORT, Value: UCHAR) {
    core::arch::asm!(
        "out dx, al",
        in("dx") Port,
        in("al") Value,
        options(nomem, nostack, preserves_flags),
    );
}

#[no_mangle]
pub unsafe extern "system" fn DebugPrint(
    DebugString: PSTRING,
    ComponentId: ULONG,
    Level: ULONG,
) -> ULONG {
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    {
        let cs: USHORT;
        core::arch::asm!("mov {0:x}, cs", out(reg) cs);
        if (cs & 3) == 0 {
            let lcr = _DbgPortRead(0x3FB);
            if lcr & 0x80 != 0 {
                _DbgPortWrite(0x3FB, lcr & 0x7F);
            }
            let mut i: USHORT = 0;
            while i < (*DebugString).Length {
                while _DbgPortRead(0x3FD) & 0x20 == 0 {}
                _DbgPortWrite(0x3F8, *(*DebugString).Buffer.offset(i as isize) as UCHAR);
                i += 1;
            }
            return STATUS_SUCCESS as ULONG;
        }
    }

    DebugService(
        BREAKPOINT_PRINT,
        (*DebugString).Buffer as *mut core::ffi::c_void,
        UlongToPtr((*DebugString).Length as ULONG),
        UlongToPtr(ComponentId),
        UlongToPtr(Level),
    )
}

#[no_mangle]
pub unsafe extern "system" fn DebugPrompt(Output: PSTRING, Input: PSTRING) -> ULONG {
    DebugService(
        BREAKPOINT_PROMPT,
        (*Output).Buffer as *mut core::ffi::c_void,
        UlongToPtr((*Output).Length as ULONG),
        (*Input).Buffer as *mut core::ffi::c_void,
        UlongToPtr((*Input).MaximumLength as ULONG),
    )
}

#[no_mangle]
pub unsafe extern "system" fn vDbgPrintExWithPrefixInternal(
    Prefix: PCCH,
    ComponentId: ULONG,
    Level: ULONG,
    Format: PCCH,
    ap: va_list,
    HandleBreakpoint: BOOLEAN,
) -> ULONG {

    let Status: NTSTATUS;
    let mut DebugString: STRING = core::mem::zeroed();
    let mut Length: SIZE_T;
    let mut PrefixLength: SIZE_T;

    if (ComponentId != MAXULONG)
        && (NtQueryDebugFilterState(ComponentId, Level)) != (1i32 as NTSTATUS)
    {
        return STATUS_SUCCESS as ULONG;
    }

    if RtlpSetInDbgPrint() != 0 {
        return STATUS_SUCCESS as ULONG;
    }

    // The C is `CHAR Buffer[512];` -- uninitialised, free. The translation
    // wrote `[0; 512]`, which is a 512-byte memset, and declared it at the top
    // so it ran on EVERY DbgPrint -- including the filtered and re-entrant
    // calls that return two lines above without ever touching the buffer. This
    // is one of the hottest paths in the system.
    //
    // It also distorted stack diagnostics: the memset was the first thing to
    // TOUCH the new frame, so a thread crossing a guard page anywhere under a
    // DbgPrint reported the fault against this function instead of against
    // whatever actually consumed the stack. That cost a probe today.
    //
    // NOTE the 512-byte FRAME is faithful to the C and is not the issue --
    // tracing costs the same stack in ReactOS as it does here. Only the memset
    // was ours, and MaybeUninit removes it outright: the C never writes a byte
    // it does not then fill, and neither does this. Only the first `Length`
    // bytes are ever read back, exactly as in the C.
    const DBG_BUFFER_LEN: usize = 512;
    let mut Buffer = core::mem::MaybeUninit::<[CHAR; DBG_BUFFER_LEN]>::uninit();
    let BufferPtr = Buffer.as_mut_ptr() as *mut CHAR;

    PrefixLength = strlen(Prefix);
    if PrefixLength > DBG_BUFFER_LEN {
        PrefixLength = DBG_BUFFER_LEN;
    }

    strncpy(BufferPtr, Prefix, PrefixLength);

    Length = rtldiag_format_va(
        BufferPtr.add(PrefixLength),
        DBG_BUFFER_LEN - PrefixLength,
        Format,
        ap,
    );

    if Length == MAXULONG as SIZE_T {
        *BufferPtr.add(DBG_BUFFER_LEN - 2) = b'\n' as CHAR;
        *BufferPtr.add(DBG_BUFFER_LEN - 1) = b'\0' as CHAR;

        Length = (DBG_BUFFER_LEN - 1) as SIZE_T;
    } else {
        Length += PrefixLength;
    }

    DebugString.Length = Length as USHORT;
    DebugString.Buffer = BufferPtr;

    Status = DebugPrint(&mut DebugString, ComponentId, Level) as NTSTATUS;

    if HandleBreakpoint != 0 {
        if Status == STATUS_BREAKPOINT {
            DbgBreakPointWithStatus(DBG_STATUS_CONTROL_C);
        }
    }

    RtlpClearInDbgPrint();
    Status as ULONG
}

#[no_mangle]
pub unsafe extern "system" fn DbgPrintEmitPreformatted(
    Body: PCCH,
    BodyLength: USHORT,
    ComponentId: ULONG,
    Level: ULONG,
) -> ULONG {
    let Status: NTSTATUS;
    let mut DebugString: STRING = core::mem::zeroed();

    if (ComponentId != MAXULONG)
        && (NtQueryDebugFilterState(ComponentId, Level)) != (1i32 as NTSTATUS)
    {
        return STATUS_SUCCESS as ULONG;
    }

    if RtlpSetInDbgPrint() != 0 {
        return STATUS_SUCCESS as ULONG;
    }

    DebugString.Length = BodyLength;
    DebugString.Buffer = Body as PCHAR;

    Status = DebugPrint(&mut DebugString, ComponentId, Level) as NTSTATUS;

    RtlpClearInDbgPrint();
    Status as ULONG
}

#[no_mangle]
pub unsafe extern "system" fn vDbgPrintExWithPrefix(
    Prefix: PCCH,
    ComponentId: ULONG,
    Level: ULONG,
    Format: PCCH,
    ap: va_list,
) -> ULONG {
    vDbgPrintExWithPrefixInternal(Prefix, ComponentId, Level, Format, ap, 1u8)
}

#[no_mangle]
pub unsafe extern "system" fn vDbgPrintEx(
    ComponentId: ULONG,
    Level: ULONG,
    Format: PCCH,
    ap: va_list,
) -> ULONG {
    vDbgPrintExWithPrefixInternal(
        b"\0".as_ptr() as PCCH,
        ComponentId,
        Level,
        Format,
        ap,
        1u8,
    )
}

// DbgPrint / DbgPrintEx / DbgPrintReturnControlC are C-variadic (`...`) entry
// points. rtl-rust builds on the *stable* toolchain, which cannot DEFINE a
// C-variadic fn (`#![feature(c_variadic)]` is nightly-only). The thin
// `va_start` trampolines therefore live in C (sdk/lib/rtl/dbgprint_variadic.c)
// and forward into the Rust worker `vDbgPrintExWithPrefixInternal` above.
// (Same language-limitation pattern as "i386 SList Pop must be global_asm!".)

#[no_mangle]
pub unsafe extern "system" fn DbgPrompt(
    Prompt: PCCH,
    Response: PCH,
    MaximumResponseLength: ULONG,
) -> ULONG {
    let mut Output: STRING = core::mem::zeroed();
    let mut Input: STRING = core::mem::zeroed();

    Input.MaximumLength = MaximumResponseLength as USHORT;
    Input.Buffer = Response;

    Output.Length = strlen(Prompt) as USHORT;
    Output.Buffer = Prompt as PCHAR;

    DebugPrompt(
        core::ptr::addr_of_mut!(Output),
        core::ptr::addr_of_mut!(Input),
    )
}

#[no_mangle]
pub unsafe extern "system" fn DbgQueryDebugFilterState(
    ComponentId: ULONG,
    Level: ULONG,
) -> NTSTATUS {
    NtQueryDebugFilterState(ComponentId, Level)
}

#[no_mangle]
pub unsafe extern "system" fn DbgSetDebugFilterState(
    ComponentId: ULONG,
    Level: ULONG,
    State: BOOLEAN,
) -> NTSTATUS {
    NtSetDebugFilterState(ComponentId, Level, State)
}

#[no_mangle]
pub unsafe extern "system" fn DbgLoadImageSymbols(
    Name: PSTRING,
    Base: *mut core::ffi::c_void,
    ProcessId: usize,
) {
    let mut SymbolInfo: KD_SYMBOLS_INFO = core::mem::zeroed();

    SymbolInfo.BaseOfDll = Base;
    SymbolInfo.ProcessId = ProcessId as _;

    let NtHeader = RtlImageNtHeader(Base);
    if !NtHeader.is_null() {
        SymbolInfo.CheckSum = (*NtHeader).OptionalHeader.CheckSum;
        SymbolInfo.SizeOfImage = (*NtHeader).OptionalHeader.SizeOfImage;
    } else {
        SymbolInfo.SizeOfImage = 0;
        SymbolInfo.CheckSum = 0;
    }

    DebugService2(
        Name as *mut core::ffi::c_void,
        core::ptr::addr_of_mut!(SymbolInfo) as *mut core::ffi::c_void,
        BREAKPOINT_LOAD_SYMBOLS,
    );
}

#[no_mangle]
pub unsafe extern "system" fn DbgUnLoadImageSymbols(
    Name: PSTRING,
    Base: PVOID,
    ProcessId: ULONG_PTR,
) {
    let mut SymbolInfo: KD_SYMBOLS_INFO = core::mem::zeroed();

    SymbolInfo.BaseOfDll = Base;
    SymbolInfo.ProcessId = ProcessId as _;
    SymbolInfo.SizeOfImage = 0;
    SymbolInfo.CheckSum = 0;

    DebugService2(
        Name as PVOID,
        core::ptr::addr_of_mut!(SymbolInfo) as PVOID,
        BREAKPOINT_UNLOAD_SYMBOLS,
    );
}

#[no_mangle]
pub unsafe extern "system" fn DbgCommandString(Name: PCCH, Command: PCH) {

    let mut NameString: STRING = core::mem::zeroed();
    let mut CommandString: STRING = core::mem::zeroed();

    NameString.Buffer = Name as PCH;
    NameString.Length = strlen(Name) as USHORT;
    CommandString.Buffer = Command as PCH;
    CommandString.Length = strlen(Command as PCCH) as USHORT;

    DebugService2(
        core::ptr::addr_of_mut!(NameString) as PVOID,
        core::ptr::addr_of_mut!(CommandString) as PVOID,
        BREAKPOINT_COMMAND_STRING,
    );
}

#[no_mangle]
pub unsafe extern "system" fn RtlPopFrame(Frame: PTEB_ACTIVE_FRAME) {
    (*NtCurrentTeb()).ActiveFrame = (*Frame).Previous;
}

#[no_mangle]
pub unsafe extern "system" fn RtlPushFrame(Frame: PTEB_ACTIVE_FRAME) {
    (*Frame).Previous = (*NtCurrentTeb()).ActiveFrame;
    (*NtCurrentTeb()).ActiveFrame = Frame;
}

#[no_mangle]
pub unsafe extern "system" fn RtlGetFrame() -> PTEB_ACTIVE_FRAME {
    (*NtCurrentTeb()).ActiveFrame
}
