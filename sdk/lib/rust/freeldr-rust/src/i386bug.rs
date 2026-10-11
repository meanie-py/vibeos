use ntkrnl_sys::base::*;
use crate::frldr::*;
use ntkrnl_sys::ddk::{PKSPECIAL_REGISTERS, PKTRAP_FRAME, PSTR, va_list};
// freeldr_ddk_imports dropped (not in ddk): File, Unused
use core::ffi::c_void;

extern "C" {
    fn MachVideoGetDisplaySize(Width: *mut ULONG, Height: *mut ULONG, Depth: *mut ULONG);
    fn MachVideoPutChar(Ch: u32, Attr: u32, X: ULONG, Y: ULONG);
    fn MachVideoHideShowTextCursor(Show: BOOLEAN);
    fn MachVideoClearScreen(Attr: UCHAR);
    fn _vsnprintf(buffer: *mut CHAR, count: usize, format: *const CHAR, args: va_list) -> i32;
    fn _snprintf(buffer: *mut CHAR, count: usize, format: *const CHAR, ...) -> i32;
}

// i386bug.c's PrintText() is `static VOID PrintText(const CHAR* Format, ...)`.
// Rust variadic *definitions* need the unstable `c_variadic` feature (not enabled
// crate-wide; lib.rs is out of scope). Faithful equivalent: a file-local macro that
// formats with the C runtime _snprintf then routes through i386PrintText — same
// effect as the C (vsnprintf into a 512 buffer + i386PrintText).
macro_rules! PrintText {
    ($($arg:tt)*) => {{
        let mut __buf: [CHAR; 512] = [0; 512];
        _snprintf(__buf.as_mut_ptr(), core::mem::size_of_val(&__buf), $($arg)*);
        __buf[core::mem::size_of_val(&__buf) - 1] = 0;
        i386PrintText(__buf.as_mut_ptr());
    }};
}

// ---- rustconv dropped file-local content from i386bug.c, restored faithfully ----
// i386bug.c: #define SCREEN_ATTR 0x1F
const SCREEN_ATTR: u32 = 0x1F;
// x86common.h: #define STACKADDR HEX(F000)
const STACKADDR: ULONG_PTR = 0xF000;
// i386bug.c: static ULONG i386_ScreenPosX = 0; static ULONG i386_ScreenPosY = 0;
static mut i386_ScreenPosX: ULONG = 0;
static mut i386_ScreenPosY: ULONG = 0;

#[repr(C)]
struct FRAME {
    Next: *mut FRAME,
    Address: *mut core::ffi::c_void,
}

unsafe fn i386PrintText(pszText: PCHAR) {
    let mut Width: ULONG = 0;
    let mut Unused: ULONG = 0;

    MachVideoGetDisplaySize(
        core::ptr::addr_of_mut!(Width),
        core::ptr::addr_of_mut!(Unused),
        core::ptr::addr_of_mut!(Unused),
    );

    let mut pszText = pszText;
    while *pszText != 0 {
        if *pszText == b'\n' as i8 {
            i386_ScreenPosX = 0;
            i386_ScreenPosY += 1;
            pszText = pszText.offset(1);
            continue;
        }

        MachVideoPutChar(*pszText as u32, SCREEN_ATTR, i386_ScreenPosX, i386_ScreenPosY);
        i386_ScreenPosX += 1;
        if i386_ScreenPosX >= Width {
            i386_ScreenPosX = 0;
            i386_ScreenPosY += 1;
        }

        pszText = pszText.offset(1);
    }
}

unsafe fn PrintTextV(Format: *const CHAR, args: va_list) {
    let mut Buffer: [CHAR; 512] = [0; 512];

    _vsnprintf(Buffer.as_mut_ptr(), core::mem::size_of_val(&Buffer), Format, args);
    Buffer[core::mem::size_of_val(&Buffer) - 1] = 0;

    i386PrintText(Buffer.as_mut_ptr());
}

// (C `PrintText` variadic wrapper replaced by the `PrintText!` macro above;
// `PrintTextV` retained as the va_list worker per never-delete-translation policy.)
#[allow(dead_code)]
unsafe fn _PrintText_va(Format: *const CHAR, args: va_list) {
    PrintTextV(Format, args);
}

unsafe fn i386PrintFrames(TrapFrame: PKTRAP_FRAME) {
    let mut Frame: *mut FRAME;

    PrintText!(b"Frames:\n\0".as_ptr() as *const CHAR);
    
    #[cfg(target_arch = "x86")]
    {
        Frame = (*TrapFrame).Ebp as *mut FRAME;
    }
    #[cfg(not(target_arch = "x86"))]
    {
        Frame = (*TrapFrame).TrapFrame as *mut FRAME;
    }

    while !Frame.is_null() && (Frame as usize) < STACKADDR {
        PrintText!(b"%p  \0".as_ptr() as *const CHAR, (*Frame).Address);
        Frame = (*Frame).Next;
    }
}

// i386bug.c FrLdrBugCheckWithMessage (DECLSPEC_NORETURN, C-variadic).
// rustconv dropped the body; Rust stable cannot DEFINE an extern "C"
// variadic (no c_variadic). Faithful non-variadic def: same screen
// path as the C; the `...`-formatted Format is printed literally
// (un-formattable on stable; this is a fatal-halt path — same
// degradation as the keystone BugCheck/TRACE stubs). Callers' cdecl
// vararg call sites are ABI-safe against a non-variadic callee
// (caller cleans the stack on i386 cdecl).
#[no_mangle]
pub unsafe extern "C" fn FrLdrBugCheckWithMessage(
    BugCode: ULONG,
    File: PCHAR,
    Line: ULONG,
    Format: PSTR,
) -> ! {
    MachVideoHideShowTextCursor(0); // FALSE
    MachVideoClearScreen(SCREEN_ATTR as UCHAR);
    i386_ScreenPosX = 0;
    i386_ScreenPosY = 0;

    PrintText!(b"A problem has been detected and FreeLoader boot has been aborted.\n\n\0".as_ptr() as *const CHAR);
    PrintText!(b"BugCheck %ld\n\n\0".as_ptr() as *const CHAR, BugCode);

    if !File.is_null() {
        PrintText!(b"Location: %s:%ld\n\n\0".as_ptr() as *const CHAR, File, Line);
    }
    if !Format.is_null() {
        // varargs un-formattable on stable Rust → emit Format literally.
        i386PrintText(Format as PCHAR);
    }

    loop {
        core::arch::asm!("cli", "hlt", options(nomem, nostack, preserves_flags));
    }
}

// i386bug.c:13 `static const CHAR *i386ExceptionDescriptionText[]`
// (rustconv dropped the non-fn table).
#[cfg(target_arch = "x86")]
static i386ExceptionDescriptionText: [&[u8]; 19] = [
    b"DIVIDE BY ZERO\0",
    b"DEBUG EXCEPTION\0",
    b"NON-MASKABLE INTERRUPT EXCEPTION\0",
    b"BREAKPOINT (INT 3)\0",
    b"OVERFLOW\0",
    b"BOUND EXCEPTION\0",
    b"INVALID OPCODE\0",
    b"FPU NOT AVAILABLE\0",
    b"DOUBLE FAULT\0",
    b"COPROCESSOR SEGMENT OVERRUN\0",
    b"INVALID TSS\0",
    b"SEGMENT NOT PRESENT\0",
    b"STACK EXCEPTION\0",
    b"GENERAL PROTECTION FAULT\0",
    b"PAGE FAULT\0",
    b"Reserved\0",
    b"COPROCESSOR ERROR\0",
    b"ALIGNMENT CHECK\0",
    b"MACHINE CHECK\0",
];

// i386bug.c:107 `VOID NTAPI i386PrintExceptionText(ULONG, PKTRAP_FRAME,
// PKSPECIAL_REGISTERS)` — called from arch/i386/i386trap.S as the
// stdcall `_i386PrintExceptionText@12` CPU-exception death screen.
// rustconv dropped the body. Faithful translation (i386 `#ifdef
// _M_IX86` branch). NTAPI on i386 = stdcall → `extern "system"` emits
// `_i386PrintExceptionText@12`. UNIQUE (translated unit owns it).
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "system" fn i386PrintExceptionText(
    TrapIndex: ULONG,
    TrapFrame: PKTRAP_FRAME,
    Special: PKSPECIAL_REGISTERS,
) {
    let tf = &*TrapFrame;
    let sp = &*Special;

    MachVideoHideShowTextCursor(0); // FALSE
    MachVideoClearScreen(SCREEN_ATTR as UCHAR);
    i386_ScreenPosX = 0;
    i386_ScreenPosY = 0;

    PrintText!(
        concat!(
            "FreeLdr exception\n",
            "Report this error on the ReactOS Bug Tracker: https://jira.reactos.org\n\n",
            "0x%02lx: Exception %02X: %s\n\n\0"
        ).as_ptr() as *const CHAR,
        TrapIndex,
        TrapIndex,
        i386ExceptionDescriptionText
            [core::cmp::min(TrapIndex as usize, i386ExceptionDescriptionText.len() - 1)]
            .as_ptr() as *const CHAR
    );

    PrintText!(b"EAX: %.8lx        ESP: %.8lx        CR0: %.8lx        DR0: %.8lx\n\0".as_ptr() as *const CHAR,
        tf.Eax, tf.HardwareEsp, sp.Cr0, tf.Dr0);
    PrintText!(b"EBX: %.8lx        EBP: %.8lx        CR1: ????????        DR1: %.8lx\n\0".as_ptr() as *const CHAR,
        tf.Ebx, tf.Ebp, tf.Dr1);
    PrintText!(b"ECX: %.8lx        ESI: %.8lx        CR2: %.8lx        DR2: %.8lx\n\0".as_ptr() as *const CHAR,
        tf.Ecx, tf.Esi, sp.Cr2, tf.Dr2);
    PrintText!(b"EDX: %.8lx        EDI: %.8lx        CR3: %.8lx        DR3: %.8lx\n\0".as_ptr() as *const CHAR,
        tf.Edx, tf.Edi, sp.Cr3, tf.Dr3);
    PrintText!(b"%*s CR4: %.8lx        DR6: %.8lx\n\0".as_ptr() as *const CHAR,
        41, b"\0".as_ptr() as *const CHAR, sp.Cr4, tf.Dr6);
    PrintText!(b"%*s DR7: %.8lx\n\0".as_ptr() as *const CHAR,
        62, b"\0".as_ptr() as *const CHAR, tf.Dr7);
    PrintText!(b" CS: %.4lx            EIP: %.8lx\n\0".as_ptr() as *const CHAR,
        tf.SegCs & 0xffff, tf.Eip);
    PrintText!(b" DS: %.4lx     ERROR CODE: %.8lx\n\0".as_ptr() as *const CHAR,
        tf.SegDs & 0xffff, tf.ErrCode);
    PrintText!(b" ES: %.4lx         EFLAGS: %.8lx\n\0".as_ptr() as *const CHAR,
        tf.SegEs & 0xffff, tf.EFlags);
    PrintText!(b" FS: %.4lx      GDTR Base: %.8lx Limit: %.4x\n\0".as_ptr() as *const CHAR,
        tf.SegFs & 0xffff, sp.Gdtr.Base, sp.Gdtr.Limit as ULONG);
    PrintText!(b" GS: %.4lx      IDTR Base: %.8lx Limit: %.4x\n\0".as_ptr() as *const CHAR,
        tf.SegGs & 0xffff, sp.Idtr.Base, sp.Idtr.Limit as ULONG);
    PrintText!(b" SS: %.4lx           LDTR: %.4lx TR: %.4lx\n\n\0".as_ptr() as *const CHAR,
        tf.HardwareSegSs & 0xffff, sp.Ldtr as ULONG, sp.Tr as ULONG);

    i386PrintFrames(TrapFrame);

    let mut InstructionPointer = tf.Eip as *const u8;
    // Adjust IP for #BP (INT 03) or #OF to point to the offending instruction
    if TrapIndex == 3 || TrapIndex == 4 {
        InstructionPointer = InstructionPointer.offset(-1);
    }

    PrintText!(b"\nInstruction stream: %.2x %.2x %.2x %.2x %.2x %.2x %.2x %.2x\n\0".as_ptr() as *const CHAR,
        *InstructionPointer.offset(0) as ULONG,
        *InstructionPointer.offset(1) as ULONG,
        *InstructionPointer.offset(2) as ULONG,
        *InstructionPointer.offset(3) as ULONG,
        *InstructionPointer.offset(4) as ULONG,
        *InstructionPointer.offset(5) as ULONG,
        *InstructionPointer.offset(6) as ULONG,
        *InstructionPointer.offset(7) as ULONG);
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
//
// #include <freeldr.h>
//
// #include <reactos/buildno.h>
// #include <debug.h>
//
// typedef struct _FRAME
// {
//     struct _FRAME* Next;
//     PVOID Address;
// } FRAME;
//
// static const CHAR *i386ExceptionDescriptionText[] =
// {
//     "DIVIDE BY ZERO",
//     "DEBUG EXCEPTION",
//     "NON-MASKABLE INTERRUPT EXCEPTION",
//     "BREAKPOINT (INT 3)",
//     "OVERFLOW",
//     "BOUND EXCEPTION",
//     "INVALID OPCODE",
//     "FPU NOT AVAILABLE",
//     "DOUBLE FAULT",
//     "COPROCESSOR SEGMENT OVERRUN",
//     "INVALID TSS",
//     "SEGMENT NOT PRESENT",
//     "STACK EXCEPTION",
//     "GENERAL PROTECTION FAULT",
//     "PAGE FAULT",
//     "Reserved",
//     "COPROCESSOR ERROR",
//     "ALIGNMENT CHECK",
//     "MACHINE CHECK"
// };
//
// #define SCREEN_ATTR 0x1F    // Bright white on blue background
//
// /* Used to store the current X and Y position on the screen */
// static ULONG i386_ScreenPosX = 0;
// static ULONG i386_ScreenPosY = 0;
//
// static void
// i386PrintText(CHAR *pszText)
// {
//     ULONG Width, Unused;
//
//     MachVideoGetDisplaySize(&Width, &Unused, &Unused);
//
//     for (; *pszText != ANSI_NULL; ++pszText)
//     {
//         if (*pszText == '\n')
//         {
//             i386_ScreenPosX = 0;
//             ++i386_ScreenPosY;
//             continue;
//         }
//
//         MachVideoPutChar(*pszText, SCREEN_ATTR, i386_ScreenPosX, i386_ScreenPosY);
//         if (++i386_ScreenPosX >= Width)
//         {
//             i386_ScreenPosX = 0;
//             ++i386_ScreenPosY;
//         }
//     // FIXME: Implement vertical screen scrolling if we are at the end of the screen.
//     }
// }
//
// static void
// PrintTextV(const CHAR *Format, va_list args)
// {
//     CHAR Buffer[512];
//
//     _vsnprintf(Buffer, sizeof(Buffer), Format, args);
//     Buffer[sizeof(Buffer) - 1] = ANSI_NULL;
//
//     i386PrintText(Buffer);
// }
//
// static void
// PrintText(const CHAR *Format, ...)
// {
//     va_list argptr;
//
//     va_start(argptr, Format);
//     PrintTextV(Format, argptr);
//     va_end(argptr);
// }
//
// static void
// i386PrintFrames(PKTRAP_FRAME TrapFrame)
// {
//     FRAME* Frame;
//
//     PrintText("Frames:\n");
//     for (Frame =
// #ifdef _M_IX86
//             (FRAME*)TrapFrame->Ebp;
// #else
//             (FRAME*)TrapFrame->TrapFrame;
// #endif
//          Frame != NULL && (ULONG_PTR)Frame < STACKADDR;
//          Frame = Frame->Next)
//     {
//         PrintText("%p  ", Frame->Address);
//     }
// }
//
// void
// NTAPI
// i386PrintExceptionText(ULONG TrapIndex, PKTRAP_FRAME TrapFrame, PKSPECIAL_REGISTERS Special)
// {
//     PUCHAR InstructionPointer;
//
//     MachVideoHideShowTextCursor(FALSE);
//     MachVideoClearScreen(SCREEN_ATTR);
//     i386_ScreenPosX = 0;
//     i386_ScreenPosY = 0;
//
//     PrintText("FreeLdr " KERNEL_VERSION_STR " " KERNEL_VERSION_BUILD_STR "\n"
//               "Report this error on the ReactOS Bug Tracker: https://jira.reactos.org\n\n"
//               "0x%02lx: Exception %02X: %s\n\n",
//               TrapIndex,
//               TrapIndex,
//               i386ExceptionDescriptionText[TrapIndex]);
//
// #ifdef _M_IX86
//     PrintText("EAX: %.8lx        ESP: %.8lx        CR0: %.8lx        DR0: %.8lx\n",
//               TrapFrame->Eax, TrapFrame->HardwareEsp, Special->Cr0, TrapFrame->Dr0);
//     PrintText("EBX: %.8lx        EBP: %.8lx        CR1: ????????        DR1: %.8lx\n",
//               TrapFrame->Ebx, TrapFrame->Ebp, TrapFrame->Dr1);
//     PrintText("ECX: %.8lx        ESI: %.8lx        CR2: %.8lx        DR2: %.8lx\n",
//               TrapFrame->Ecx, TrapFrame->Esi, Special->Cr2, TrapFrame->Dr2);
//     PrintText("EDX: %.8lx        EDI: %.8lx        CR3: %.8lx        DR3: %.8lx\n",
//               TrapFrame->Edx, TrapFrame->Edi, Special->Cr3, TrapFrame->Dr3);
//     PrintText("%*s CR4: %.8lx        DR6: %.8lx\n",
//               41, "", Special->Cr4, TrapFrame->Dr6);
//     PrintText("%*s DR7: %.8lx\n",
//               62, "", TrapFrame->Dr7);
//
//     /* NOTE: Segment registers are intrinsically 16 bits. Even if the x86
//      * KTRAP_FRAME structure stores them as ULONG, only their lower 16 bits
//      * are initialized. We thus cast them to USHORT before display. */
//     PrintText(" CS: %.4lx            EIP: %.8lx\n",
//               (USHORT)TrapFrame->SegCs, TrapFrame->Eip);
//     PrintText(" DS: %.4lx     ERROR CODE: %.8lx\n",
//               (USHORT)TrapFrame->SegDs, TrapFrame->ErrCode);
//     PrintText(" ES: %.4lx         EFLAGS: %.8lx\n",
//               (USHORT)TrapFrame->SegEs, TrapFrame->EFlags);
//     PrintText(" FS: %.4lx      GDTR Base: %.8lx Limit: %.4x\n",
//            // " FS: %.4lx           GDTR: Base %.8lx Limit %.4x\n"
//               (USHORT)TrapFrame->SegFs, Special->Gdtr.Base, Special->Gdtr.Limit);
//     PrintText(" GS: %.4lx      IDTR Base: %.8lx Limit: %.4x\n",
//            // " GS: %.4lx           IDTR: Base %.8lx Limit %.4x\n",
//               (USHORT)TrapFrame->SegGs, Special->Idtr.Base, Special->Idtr.Limit);
//     PrintText(" SS: %.4lx           LDTR: %.4lx TR: %.4lx\n\n",
//               (USHORT)TrapFrame->HardwareSegSs, Special->Ldtr, Special->Tr);
// #else
//     PrintText("RAX: %.8lx        R8:  %.8lx        R12: %.8lx        RSI: %.8lx\n",
//               TrapFrame->Rax, TrapFrame->R8, 0, TrapFrame->Rsi);
//     PrintText("RBX: %.8lx        R9:  %.8lx        R13: %.8lx        RDI: %.8lx\n",
//               TrapFrame->Rbx, TrapFrame->R9, 0, TrapFrame->Rdi);
//     PrintText("RCX: %.8lx        R10: %.8lx        R14: %.8lx        RBP: %.8lx\n",
//               TrapFrame->Rcx, TrapFrame->R10, 0, TrapFrame->Rbp);
//     PrintText("RDX: %.8lx        R11: %.8lx        R15: %.8lx        RSP: %.8lx\n",
//               TrapFrame->Rdx, TrapFrame->R11, 0, TrapFrame->Rsp);
//
//     PrintText(" CS: %.4lx            RIP: %.8lx\n",
//               TrapFrame->SegCs, TrapFrame->Rip);
//     PrintText(" DS: %.4lx     ERROR CODE: %.8lx\n",
//               TrapFrame->SegDs, TrapFrame->ErrorCode);
//     PrintText(" ES: %.4lx         EFLAGS: %.8lx\n",
//               TrapFrame->SegEs, TrapFrame->EFlags);
//     PrintText(" FS: %.4lx      GDTR Base: %.8lx Limit: %.4x\n",
//               TrapFrame->SegFs, Special->Gdtr.Base, Special->Gdtr.Limit);
//     PrintText(" GS: %.4lx      IDTR Base: %.8lx Limit: %.4x\n",
//               TrapFrame->SegGs, Special->Idtr.Base, Special->Idtr.Limit);
//     PrintText(" SS: %.4lx           LDTR: %.4lx TR: %.4lx\n\n",
//               TrapFrame->SegSs, Special->Ldtr, Special->Tr);
// #endif
//
//     /* Display the stack frames */
//     i386PrintFrames(TrapFrame);
//
// #ifdef _M_IX86
//     InstructionPointer = (PUCHAR)TrapFrame->Eip;
// #else
//     InstructionPointer = (PUCHAR)TrapFrame->Rip;
// #endif
//     /* Adjust IP for #BP (INT 03) or #OF to point to the offending instruction */
//     if ((TrapIndex == 3) || (TrapIndex == 4))
//         InstructionPointer--;
//
//     PrintText("\nInstruction stream: %.2x %.2x %.2x %.2x %.2x %.2x %.2x %.2x\n",
//               InstructionPointer[0], InstructionPointer[1],
//               InstructionPointer[2], InstructionPointer[3],
//               InstructionPointer[4], InstructionPointer[5],
//               InstructionPointer[6], InstructionPointer[7]);
// }
//
// DECLSPEC_NORETURN
// VOID
// FrLdrBugCheckWithMessage(
//     ULONG BugCode,
//     PCHAR File,
//     ULONG Line,
//     PSTR Format,
//     ...)
// {
//     va_list argptr;
//
//     MachVideoHideShowTextCursor(FALSE);
//     MachVideoClearScreen(SCREEN_ATTR);
//     i386_ScreenPosX = 0;
//     i386_ScreenPosY = 0;
//
//     PrintText("A problem has been detected and FreeLoader boot has been aborted.\n\n");
//
//     PrintText("%ld: %s\n\n", BugCode, BugCodeStrings[BugCode]);
//
//     if (File)
//     {
//         PrintText("Location: %s:%ld\n\n", File, Line);
//     }
//
//     va_start(argptr, Format);
//     PrintTextV(Format, argptr);
//     va_end(argptr);
//
//     _disable();
//     __halt();
//     for (;;);
// }
//
// static
// DECLSPEC_NORETURN
// void
// FrLdrBugCheckEx(
//     ULONG BugCode,
//     PCHAR File,
//     ULONG Line)
// {
//     MachVideoHideShowTextCursor(FALSE);
//     MachVideoClearScreen(SCREEN_ATTR);
//     i386_ScreenPosX = 0;
//     i386_ScreenPosY = 0;
//
//     PrintText("A problem has been detected and FreeLoader boot has been aborted.\n\n");
//
//     PrintText("%ld: %s\n\n", BugCode, BugCodeStrings[BugCode]);
//
//     if (File)
//     {
//         PrintText("Location: %s:%ld\n\n", File, Line);
//     }
//
//     PrintText("Bug Information:\n    %p\n    %p\n    %p\n    %p\n    %p\n\n",
//               BugCheckInfo[0], BugCheckInfo[1], BugCheckInfo[2], BugCheckInfo[3], BugCheckInfo[4]);
//
//     _disable();
//     __halt();
//     for (;;);
// }
//
// DECLSPEC_NORETURN
// void
// NTAPI
// FrLdrBugCheck(ULONG BugCode)
// {
//     FrLdrBugCheckEx(BugCode, 0, 0);
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
