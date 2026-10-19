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
	.p2align 2
__delay_int:
	.rva __hn_0
	.rva __hn_1
	.rva __hn_2
	.long 0x8000007f
	.long 0
	.section .data,"dw"
	.p2align 2
__delay_iat:
	.globl __imp__ByName
__imp__ByName:
	.long __thunk_0
	.globl "__imp__Decorated@8"
"__imp__Decorated@8":
	.long __thunk_1
	.globl __imp__Forward
__imp__Forward:
	.long __thunk_2
	.globl __imp__SHOnCWMCommandLine
__imp__SHOnCWMCommandLine:
	.long __thunk_3
	.long 0
__delay_hmod:
	.long 0
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
	.globl _ByName
_ByName:
	jmp  *__imp__ByName
__thunk_0:
	pushl $__imp__ByName
	pushl $___DELAY_DESC_browseui_dll
	calll ___delayLoadHelper2@8
	jmpl  *%eax
	.globl "_Decorated@8"
"_Decorated@8":
	jmp  *"__imp__Decorated@8"
__thunk_1:
	pushl $"__imp__Decorated@8"
	pushl $___DELAY_DESC_browseui_dll
	calll ___delayLoadHelper2@8
	jmpl  *%eax
	.globl _Forward
_Forward:
	jmp  *__imp__Forward
__thunk_2:
	pushl $__imp__Forward
	pushl $___DELAY_DESC_browseui_dll
	calll ___delayLoadHelper2@8
	jmpl  *%eax
	.globl _SHOnCWMCommandLine
_SHOnCWMCommandLine:
	jmp  *__imp__SHOnCWMCommandLine
__thunk_3:
	pushl $__imp__SHOnCWMCommandLine
	pushl $___DELAY_DESC_browseui_dll
	calll ___delayLoadHelper2@8
	jmpl  *%eax
	/* NOTE: 1 DATA export(s) not delay-bound: Data */
