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
	jmp  *__imp_ByName(%rip)
__thunk_0:
	subq $0x88, %rsp
	movq %rcx,0x20(%rsp)
	movq %rdx,0x28(%rsp)
	movq %r8,0x30(%rsp)
	movq %r9,0x38(%rsp)
	movaps %xmm0,0x40(%rsp)
	movaps %xmm1,0x50(%rsp)
	movaps %xmm2,0x60(%rsp)
	movaps %xmm3,0x70(%rsp)
	leaq __imp_ByName(%rip), %rdx
	leaq ___DELAY_DESC_browseui_dll(%rip), %rcx
	callq __delayLoadHelper2
	movq %rax, %r11
	movaps 0x70(%rsp),%xmm3
	movaps 0x60(%rsp),%xmm2
	movaps 0x50(%rsp),%xmm1
	movaps 0x40(%rsp),%xmm0
	movq 0x38(%rsp),%r9
	movq 0x30(%rsp),%r8
	movq 0x28(%rsp),%rdx
	movq 0x20(%rsp),%rcx
	addq $0x88, %rsp
	jmpq *%r11
	.globl "Decorated@8"
"Decorated@8":
	jmp  *"__imp_Decorated@8"(%rip)
__thunk_1:
	subq $0x88, %rsp
	movq %rcx,0x20(%rsp)
	movq %rdx,0x28(%rsp)
	movq %r8,0x30(%rsp)
	movq %r9,0x38(%rsp)
	movaps %xmm0,0x40(%rsp)
	movaps %xmm1,0x50(%rsp)
	movaps %xmm2,0x60(%rsp)
	movaps %xmm3,0x70(%rsp)
	leaq "__imp_Decorated@8"(%rip), %rdx
	leaq ___DELAY_DESC_browseui_dll(%rip), %rcx
	callq __delayLoadHelper2
	movq %rax, %r11
	movaps 0x70(%rsp),%xmm3
	movaps 0x60(%rsp),%xmm2
	movaps 0x50(%rsp),%xmm1
	movaps 0x40(%rsp),%xmm0
	movq 0x38(%rsp),%r9
	movq 0x30(%rsp),%r8
	movq 0x28(%rsp),%rdx
	movq 0x20(%rsp),%rcx
	addq $0x88, %rsp
	jmpq *%r11
	.globl Forward
Forward:
	jmp  *__imp_Forward(%rip)
__thunk_2:
	subq $0x88, %rsp
	movq %rcx,0x20(%rsp)
	movq %rdx,0x28(%rsp)
	movq %r8,0x30(%rsp)
	movq %r9,0x38(%rsp)
	movaps %xmm0,0x40(%rsp)
	movaps %xmm1,0x50(%rsp)
	movaps %xmm2,0x60(%rsp)
	movaps %xmm3,0x70(%rsp)
	leaq __imp_Forward(%rip), %rdx
	leaq ___DELAY_DESC_browseui_dll(%rip), %rcx
	callq __delayLoadHelper2
	movq %rax, %r11
	movaps 0x70(%rsp),%xmm3
	movaps 0x60(%rsp),%xmm2
	movaps 0x50(%rsp),%xmm1
	movaps 0x40(%rsp),%xmm0
	movq 0x38(%rsp),%r9
	movq 0x30(%rsp),%r8
	movq 0x28(%rsp),%rdx
	movq 0x20(%rsp),%rcx
	addq $0x88, %rsp
	jmpq *%r11
	.globl SHOnCWMCommandLine
SHOnCWMCommandLine:
	jmp  *__imp_SHOnCWMCommandLine(%rip)
__thunk_3:
	subq $0x88, %rsp
	movq %rcx,0x20(%rsp)
	movq %rdx,0x28(%rsp)
	movq %r8,0x30(%rsp)
	movq %r9,0x38(%rsp)
	movaps %xmm0,0x40(%rsp)
	movaps %xmm1,0x50(%rsp)
	movaps %xmm2,0x60(%rsp)
	movaps %xmm3,0x70(%rsp)
	leaq __imp_SHOnCWMCommandLine(%rip), %rdx
	leaq ___DELAY_DESC_browseui_dll(%rip), %rcx
	callq __delayLoadHelper2
	movq %rax, %r11
	movaps 0x70(%rsp),%xmm3
	movaps 0x60(%rsp),%xmm2
	movaps 0x50(%rsp),%xmm1
	movaps 0x40(%rsp),%xmm0
	movq 0x38(%rsp),%r9
	movq 0x30(%rsp),%r8
	movq 0x28(%rsp),%rdx
	movq 0x20(%rsp),%rcx
	addq $0x88, %rsp
	jmpq *%r11
	/* NOTE: 1 DATA export(s) not delay-bound: Data */
