.text






.globl LdrInitializeThunk
LdrInitializeThunk:
    mov x29, #0
    mov x0, x9
    b LdrpInit
.globl KiUserApcDispatcher
KiUserApcDispatcher:

    mov x9, sp


    blr x3


    mov x0, sp
    mov w1, #1
    bl NtContinue


    brk #0xF000







.globl KiUserExceptionDispatcher
KiUserExceptionDispatcher:
    stp x29, x30, [sp, #-32]!
    mov x29, sp
    stp x19, x20, [sp, #16]
    mov x19, x0
    mov x20, x1

    bl RtlDispatchException

    cbz w0, .LRaiseException

    mov x0, x20
    mov w1, #0
    bl NtContinue
    brk #0xF003

.LRaiseException:
    mov x0, x19
    mov x1, x20
    mov w2, #0
    bl NtRaiseException
    brk #0xF003
