	.section .rdata,"dr"
	.p2align 2
	.rva __vibeos_delayimp_head
	.rva __vibeos_delayimp_tail
__dll_name: .asciz "browseui.dll"
	.p2align 1
__hn_0: .short 0
	.asciz "ByName"
	.p2align 1
__hn_1: .short 0
	.asciz "Decorated"
	.p2align 1
__hn_2: .short 0
	.asciz "Forward"
	.p2align 3
__delay_int:
	.rva __hn_0
	.long 0
	.rva __hn_1
	.long 0
	.rva __hn_2
	.long 0
	.long 0x7f
	.long 0x80000000
	.long 0
	.long 0
	.section .data,"dw"
	.p2align 3
__delay_iat:
	.globl __imp_ByName
__imp_ByName:
	.quad __thunk_0
	.globl "__imp_Decorated@8"
"__imp_Decorated@8":
	.quad __thunk_1
	.globl __imp_Forward
__imp_Forward:
	.quad __thunk_2
	.globl __imp_SHOnCWMCommandLine
__imp_SHOnCWMCommandLine:
	.quad __thunk_3
	.quad 0
__delay_hmod:
	.quad 0
	.section .CRT$VBDLYIMPA,"dr",discard,__vibeos_delayimp_head
	.p2align 2
	.globl __vibeos_delayimp_head
__vibeos_delayimp_head:
	.ascii "VIBEOS-DLYIMP-01"
	.section .CRT$VBDLYIMPC,"dr",discard,__vibeos_delayimp_tail
	.p2align 2
	.globl __vibeos_delayimp_tail
__vibeos_delayimp_tail:
	.zero 32
	.section .CRT$VBDLYIMPB,"dr"
	.p2align 2
	.globl ___DELAY_DESC_browseui_dll
___DELAY_DESC_browseui_dll:
	.long 1
	.rva __dll_name
	.rva __delay_hmod
	.rva __delay_iat
	.rva __delay_int
	.long 0
	.long 0
	.long 0
	.section .text,"xr"
	.globl ByName
ByName:
	adrp x16, __imp_ByName
	ldr  x16, [x16, :lo12:__imp_ByName]
	br   x16
__thunk_0:
	sub  sp, sp, #0xe0
	stp  x0, x1, [sp, #0x00]
	stp  x2, x3, [sp, #0x10]
	stp  x4, x5, [sp, #0x20]
	stp  x6, x7, [sp, #0x30]
	str  x8, [sp, #0x40]
	stp  q0, q1, [sp, #0x50]
	stp  q2, q3, [sp, #0x70]
	stp  q4, q5, [sp, #0x90]
	stp  q6, q7, [sp, #0xb0]
	str  x30, [sp, #0xd0]
	adrp x0, ___DELAY_DESC_browseui_dll
	add  x0, x0, :lo12:___DELAY_DESC_browseui_dll
	adrp x1, __imp_ByName
	add  x1, x1, :lo12:__imp_ByName
	bl   __delayLoadHelper2
	mov  x16, x0
	ldr  x30, [sp, #0xd0]
	ldp  x0, x1, [sp, #0x00]
	ldp  x2, x3, [sp, #0x10]
	ldp  x4, x5, [sp, #0x20]
	ldp  x6, x7, [sp, #0x30]
	ldr  x8, [sp, #0x40]
	ldp  q0, q1, [sp, #0x50]
	ldp  q2, q3, [sp, #0x70]
	ldp  q4, q5, [sp, #0x90]
	ldp  q6, q7, [sp, #0xb0]
	add  sp, sp, #0xe0
	br   x16
	.globl "Decorated@8"
"Decorated@8":
	adrp x16, "__imp_Decorated@8"
	ldr  x16, [x16, :lo12:"__imp_Decorated@8"]
	br   x16
__thunk_1:
	sub  sp, sp, #0xe0
	stp  x0, x1, [sp, #0x00]
	stp  x2, x3, [sp, #0x10]
	stp  x4, x5, [sp, #0x20]
	stp  x6, x7, [sp, #0x30]
	str  x8, [sp, #0x40]
	stp  q0, q1, [sp, #0x50]
	stp  q2, q3, [sp, #0x70]
	stp  q4, q5, [sp, #0x90]
	stp  q6, q7, [sp, #0xb0]
	str  x30, [sp, #0xd0]
	adrp x0, ___DELAY_DESC_browseui_dll
	add  x0, x0, :lo12:___DELAY_DESC_browseui_dll
	adrp x1, "__imp_Decorated@8"
	add  x1, x1, :lo12:"__imp_Decorated@8"
	bl   __delayLoadHelper2
	mov  x16, x0
	ldr  x30, [sp, #0xd0]
	ldp  x0, x1, [sp, #0x00]
	ldp  x2, x3, [sp, #0x10]
	ldp  x4, x5, [sp, #0x20]
	ldp  x6, x7, [sp, #0x30]
	ldr  x8, [sp, #0x40]
	ldp  q0, q1, [sp, #0x50]
	ldp  q2, q3, [sp, #0x70]
	ldp  q4, q5, [sp, #0x90]
	ldp  q6, q7, [sp, #0xb0]
	add  sp, sp, #0xe0
	br   x16
	.globl Forward
Forward:
	adrp x16, __imp_Forward
	ldr  x16, [x16, :lo12:__imp_Forward]
	br   x16
__thunk_2:
	sub  sp, sp, #0xe0
	stp  x0, x1, [sp, #0x00]
	stp  x2, x3, [sp, #0x10]
	stp  x4, x5, [sp, #0x20]
	stp  x6, x7, [sp, #0x30]
	str  x8, [sp, #0x40]
	stp  q0, q1, [sp, #0x50]
	stp  q2, q3, [sp, #0x70]
	stp  q4, q5, [sp, #0x90]
	stp  q6, q7, [sp, #0xb0]
	str  x30, [sp, #0xd0]
	adrp x0, ___DELAY_DESC_browseui_dll
	add  x0, x0, :lo12:___DELAY_DESC_browseui_dll
	adrp x1, __imp_Forward
	add  x1, x1, :lo12:__imp_Forward
	bl   __delayLoadHelper2
	mov  x16, x0
	ldr  x30, [sp, #0xd0]
	ldp  x0, x1, [sp, #0x00]
	ldp  x2, x3, [sp, #0x10]
	ldp  x4, x5, [sp, #0x20]
	ldp  x6, x7, [sp, #0x30]
	ldr  x8, [sp, #0x40]
	ldp  q0, q1, [sp, #0x50]
	ldp  q2, q3, [sp, #0x70]
	ldp  q4, q5, [sp, #0x90]
	ldp  q6, q7, [sp, #0xb0]
	add  sp, sp, #0xe0
	br   x16
	.globl SHOnCWMCommandLine
SHOnCWMCommandLine:
	adrp x16, __imp_SHOnCWMCommandLine
	ldr  x16, [x16, :lo12:__imp_SHOnCWMCommandLine]
	br   x16
__thunk_3:
	sub  sp, sp, #0xe0
	stp  x0, x1, [sp, #0x00]
	stp  x2, x3, [sp, #0x10]
	stp  x4, x5, [sp, #0x20]
	stp  x6, x7, [sp, #0x30]
	str  x8, [sp, #0x40]
	stp  q0, q1, [sp, #0x50]
	stp  q2, q3, [sp, #0x70]
	stp  q4, q5, [sp, #0x90]
	stp  q6, q7, [sp, #0xb0]
	str  x30, [sp, #0xd0]
	adrp x0, ___DELAY_DESC_browseui_dll
	add  x0, x0, :lo12:___DELAY_DESC_browseui_dll
	adrp x1, __imp_SHOnCWMCommandLine
	add  x1, x1, :lo12:__imp_SHOnCWMCommandLine
	bl   __delayLoadHelper2
	mov  x16, x0
	ldr  x30, [sp, #0xd0]
	ldp  x0, x1, [sp, #0x00]
	ldp  x2, x3, [sp, #0x10]
	ldp  x4, x5, [sp, #0x20]
	ldp  x6, x7, [sp, #0x30]
	ldr  x8, [sp, #0x40]
	ldp  q0, q1, [sp, #0x50]
	ldp  q2, q3, [sp, #0x70]
	ldp  q4, q5, [sp, #0x90]
	ldp  q6, q7, [sp, #0xb0]
	add  sp, sp, #0xe0
	br   x16
	/* NOTE: 1 DATA export(s) not delay-bound: Data */
