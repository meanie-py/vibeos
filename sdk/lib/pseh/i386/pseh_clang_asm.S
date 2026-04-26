/*
 * PROJECT:     ReactOS system libraries
 * LICENSE:     GNU GPL - See COPYING in the top level directory
 * PURPOSE:     Setjmp/longjmp PSEH support for i386 Clang (llvm-mingw)
 * PROGRAMMER:  ReactOS ARM64 port
 *
 * Frame layout (_PSEH_CLANG_FRAME):
 *   +0   Prev           (EXCEPTION_REGISTRATION_RECORD.Next)
 *   +4   Handler        (EXCEPTION_REGISTRATION_RECORD.Handler)
 *   +8   SavedEbp
 *   +12  SavedEsp
 *   +16  SavedEip
 *   +20  SavedEbx
 *   +24  SavedEsi
 *   +28  SavedEdi
 *   +32  ExceptionCode
 *   +36  ExceptionRecordCopy  (80 bytes = sizeof(EXCEPTION_RECORD))
 *   +116 ExceptionPointersCopy (8 bytes: .ExceptionRecord, .ContextRecord)
 */

.text

/*
 * int __cdecl _PsehSetjmp(_PSEH_CLANG_FRAME *frame)
 *
 * Saves callee-saved registers + return address into frame.
 * Returns 0 on first call, nonzero on longjmp return.
 * Declared __attribute__((returns_twice)) in the header.
 */
.global __PsehSetjmp
__PsehSetjmp:
    movl    4(%esp), %ecx           /* ecx = frame */
    movl    %ebp, 8(%ecx)           /* SavedEbp */
    leal    4(%esp), %eax           /* caller's ESP (skip ret addr pushed by call) */
    movl    %eax, 12(%ecx)          /* SavedEsp */
    movl    (%esp), %eax            /* return address */
    movl    %eax, 16(%ecx)          /* SavedEip */
    movl    %ebx, 20(%ecx)          /* SavedEbx */
    movl    %esi, 24(%ecx)          /* SavedEsi */
    movl    %edi, 28(%ecx)          /* SavedEdi */
    xorl    %eax, %eax              /* return 0 */
    ret

/*
 * void __cdecl __declspec(noreturn) _PsehLongjmp(_PSEH_CLANG_FRAME *frame, int value)
 *
 * Restores registers saved by _PsehSetjmp and jumps back.
 */
.global __PsehLongjmp
__PsehLongjmp:
    movl    4(%esp), %ecx           /* ecx = frame */
    movl    8(%esp), %eax           /* eax = value (nonzero) */
    movl    20(%ecx), %ebx          /* restore EBX */
    movl    24(%ecx), %esi          /* restore ESI */
    movl    28(%ecx), %edi          /* restore EDI */
    movl    8(%ecx), %ebp           /* restore EBP */
    movl    12(%ecx), %esp          /* restore ESP */
    jmp     *16(%ecx)               /* jump to SavedEip (return address of _PsehSetjmp call) */

/*
 * void __cdecl _PsehPushFrame(_PSEH_CLANG_FRAME *frame)
 *
 * Pushes frame onto fs:[0] SEH chain.
 */
.global __PsehPushFrame
__PsehPushFrame:
    movl    4(%esp), %ecx           /* ecx = frame */
    movl    %fs:0, %eax             /* eax = current head */
    movl    %eax, 0(%ecx)           /* frame->Prev = old head */
    movl    %ecx, %fs:0             /* fs:[0] = frame */
    ret

/*
 * void __cdecl _PsehCallRtlUnwind(_PSEH_CLANG_FRAME *frame)
 *
 * Calls RtlUnwind@16 with callee-saved register protection.
 * RtlUnwind may clobber non-volatiles during unwind callbacks.
 */
.global __PsehCallRtlUnwind
__PsehCallRtlUnwind:
    pushl   %ebp
    movl    %esp, %ebp
    pushl   %edi
    pushl   %esi
    pushl   %ebx

    pushl   $0                      /* ReturnValue */
    pushl   $0                      /* ExceptionRecord */
    pushl   $0                      /* TargetIp */
    pushl   8(%ebp)                 /* TargetFrame = frame */
    call    _RtlUnwind@16

    popl    %ebx
    popl    %esi
    popl    %edi
    popl    %ebp
    ret
