
#include <asm.inc>
#include <ks386.inc>


// cpu 486
.code

EXTERN _check:PROC

PUBLIC _continuePoint
_continuePoint:
 push ss
 push 0
 pushfd
 push cs
 // Eip slot — clang's integrated assembler can't emit the relocation for
 // `push offset sym`, and no register is free here (the whole register file
 // is being recorded). Push a placeholder; patched below once eax is saved.
 push 0
 push ebp

 push eax
 push ecx
 push edx
 push ebx
 push esi
 push edi

 push ds
 push es
 push fs
 push gs

 // Fill the Eip placeholder (5th push -> 11 pushes above it = [esp + 44])
 lea eax, [_continuePoint]
 mov [esp + HEX(02C)], eax

 // TODO: floating point state
 sub esp, 112

 // Debug registers
 sub esp, 24

 push HEX(00010007)

 // Fill the Esp field
 lea eax, [esp + HEX(0CC)]
 lea ecx, [esp + HEX(0C4)]
 mov [ecx], eax

 // Call the function that will compare the current context with the expected one
 cld
 push esp
 call _check

 // check() must not return
 int 3

// EOF
END
