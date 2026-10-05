#![no_std]
#![allow(non_upper_case_globals)]
//! pseh-rust — the i386 PSEH/CRT SEH runtime translated to Rust.
//!
//! Faithful translation of `sdk/lib/pseh/i386/seh.s` (the five exported SEH
//! runtime entry points) plus the `sdk/lib/pseh/dummy.c` volatile data symbols.
//!
//! Why this exists: `ntoskrnl.spec` exports `_except_handler2`/`_except_handler3`/
//! `_global_unwind2`/`_local_unwind2`/`_abnormal_termination` (`-arch=i386,arm`)
//! as the driver-facing SEH ABI. Under `USE_DUMMY_PSEH` the kernel's own body
//! never unwinds through them, but they must still be DEFINED so the export
//! table resolves — that is the only reason the kernel kept linking the cmake
//! `libpseh.a`. Translating the runtime to a Rust crate lets the kernel drop
//! that last `libpseh.a` reuse and link it cargo-native.
//!
//! The runtime is irreducible machine code: it walks the `fs:[0]` exception
//! registration chain, accesses the caller frame ebp-relative against the exact
//! `__SEH_prolog` layout, and clears every GPR before calling a filter/handler.
//! None of that survives Rust codegen, so it lives in `global_asm!` transcribed
//! verbatim from seh.s (the `setjmp-rust` precedent). i386-only — amd64/arm64
//! ntoskrnl does not export these symbols. Validated by disassembly-equivalence
//! against the cmake `libpseh.a` (`seh.s` object).
//!
//! The two `_SEH2_Volatile*` globals (`dummy.c`) are genuine Rust. They are
//! needed on ALL three arches: the C cohab archives still linked into the kernel
//! include `pseh2.h`, whose `USE_DUMMY_PSEH` macros reference `_SEH2_Volatile0`
//! (the dead-code guard) and `_SEH2_VolatileExceptionCode` (`_SEH2_GetExceptionCode`).

// DUMMY_PSEH volatile data symbols (verbatim from sdk/lib/pseh/dummy.c).
// `static mut` to land in writable .data exactly like the C `int x = ...;`
// (matching the C ABI; pseh2.h only ever reads them). `#[used]` so they survive
// even when this object is pulled only for the SEH runtime above.
#[used]
#[no_mangle]
pub static mut _SEH2_Volatile0: i32 = 0;
#[used]
#[no_mangle]
pub static mut _SEH2_VolatileExceptionCode: i32 = 0xC000_0005u32 as i32;

// The i386 SEH runtime. Transcribed verbatim from sdk/lib/pseh/i386/seh.s; the
// MASM `<asm.inc>`/`<ks386.inc>` symbolic offsets become `.set` constants (the
// EXCEPTION_* values are from the generated ks386.inc: FLAGS=4, POINTERS
// record=0 / context=4; the SEH_FRAME_* layout is from seh_prolog.s). The five
// PUBLIC entry points are emitted with their decorated i386 object names
// (`__name`, two underscores = the cdecl-decorated `_name` the .spec exports);
// `_unwind_handler` stays file-local. External `_RtlUnwind@16` is the kernel's
// own export.
#[cfg(target_arch = "x86")]
core::arch::global_asm!(
    ".intel_syntax noprefix",
    ".text",

    ".set DISPOSITION_DISMISS, 0",
    ".set DISPOSITION_CONTINUE_SEARCH, 1",
    ".set DISPOSITION_COLLIDED_UNWIND, 3",
    ".set EXCEPTION_EXIT_UNWIND, 4",
    ".set EXCEPTION_UNWINDING, 2",
    ".set EXCEPTION_RECORD_EXCEPTION_FLAGS, 4",
    ".set EXCEPTION_POINTERS_EXCEPTION_RECORD, 0",
    ".set EXCEPTION_POINTERS_CONTEXT_RECORD, 4",
    // SEH_FRAME_* — see seh_prolog.s.
    ".set SEH_FRAME_NewEsp, 0",
    ".set SEH_FRAME_unused, 4",
    ".set SEH_FRAME_PreviousRecord, 8",
    ".set SEH_FRAME_Handler, 12",
    ".set SEH_FRAME_SEHTable, 16",
    ".set SEH_FRAME_Disable, 20",
    ".set SEH_FRAME_OriginalEbp, 24",
    ".set SEH_FRAME_Size, 28",
    // __except_handler3 param offsets (local equates in seh.s).
    ".set PARAM_ExceptionRecord, 8",
    ".set PARAM_RegistrationFrame, 12",
    ".set PARAM_Context, 16",

    ".globl __global_unwind2",
    ".globl __local_unwind2",
    ".globl __abnormal_termination",
    ".globl __except_handler2",
    ".globl __except_handler3",

    // ---- _unwind_handler (file-local) -----------------------------------
    "_unwind_handler:",
    "    mov ecx, [esp+4]",
    "    test dword ptr [ecx+4], EXCEPTION_EXIT_UNWIND + EXCEPTION_UNWINDING",
    "    mov eax, DISPOSITION_CONTINUE_SEARCH",
    "    jz unwind_handler_return",
    "    mov eax, [esp+20]",
    "    push ebp",
    "    mov ebp, [eax+16]",
    "    mov edx, [eax+40]",
    "    push edx",
    "    mov edx, [eax+36]",
    "    push edx",
    "    call __local_unwind2",
    "    add esp, 8",
    "    pop ebp",
    "    mov eax, [esp+8]",
    "    mov edx, [esp+16]",
    "    mov [edx], eax",
    "    mov eax, DISPOSITION_COLLIDED_UNWIND",
    "unwind_handler_return:",
    "    ret",

    // ---- __global_unwind2 -----------------------------------------------
    "__global_unwind2:",
    "    push ebp",
    "    mov ebp, esp",
    "    push ebx",
    "    push esi",
    "    push edi",
    "    push ebp",
    "    push 0",
    "    push 0",
    // seh.s: `push glu_return` (push the return address as imm32). LLVM's COFF
    // i386 backend rejects a symbol relocation on the push-imm32 encoding
    // ("unsupported relocation type"), but accepts it on mov-imm32 (b8 +DIR32),
    // so materialize the address in a dead scratch reg first. eax is caller-
    // saved and unused here (RtlUnwind clobbers it; reloaded after the call).
    "    mov eax, offset glu_return",
    "    push eax",
    "    push dword ptr [ebp+8]",
    "    call _RtlUnwind@16",
    "glu_return:",
    "    pop ebp",
    "    pop edi",
    "    pop esi",
    "    pop ebx",
    "    mov esp, ebp",
    "    pop ebp",
    "    ret",

    // ---- __abnormal_termination -----------------------------------------
    "__abnormal_termination:",
    "    xor eax, eax",
    "    mov ecx, dword ptr fs:[0]",
    "    cmp dword ptr [ecx+4], offset _unwind_handler",
    "    jne ab_return",
    "    mov edx, [ecx+12]",
    "    mov edx, [edx+12]",
    "    cmp [ecx+8], edx",
    "    jne ab_return",
    "    mov eax, 1",
    "ab_return:",
    "    ret",

    // ---- __local_unwind2 ------------------------------------------------
    "__local_unwind2:",
    "    push ebx",
    "    push esi",
    "    push edi",
    "    mov eax, [esp+16]",
    "    push ebp",
    "    push eax",
    "    push -2",
    // seh.s: `push _unwind_handler` (push the handler address). Same LLVM
    // push-imm32-symbol limitation as glu_return above → mov+push. eax already
    // consumed by `push eax` above and is reloaded at unwind_loop, so it is dead.
    "    mov eax, offset _unwind_handler",
    "    push eax",
    "    push dword ptr fs:[0]",
    "    mov dword ptr fs:[0], esp",
    "unwind_loop:",
    "    mov eax, [esp+36]",
    "    mov ebx, [eax+8]",
    "    mov esi, [eax+12]",
    "    cmp esi, -1",
    "    je unwind_return",
    "    cmp dword ptr [esp+40], -1",
    "    je unwind_ok",
    "    cmp esi, [esp+40]",
    "    jbe unwind_return",
    "unwind_ok:",
    "    lea esi, [esi+esi*2]",
    "    mov ecx, [ebx+esi*4]",
    "    mov [esp+8], ecx",
    "    mov [eax+12], ecx",
    "    cmp dword ptr [ebx+esi*4+4], 0",
    "    jnz NLG_Return2",
    "    call dword ptr [ebx+esi*4+8]",
    "NLG_Return2:",
    "    jmp unwind_loop",
    "unwind_return:",
    "    pop dword ptr fs:[0]",
    "    add esp, 16",
    "    pop edi",
    "    pop esi",
    "    pop ebx",
    "    ret",

    // ---- __except_handler2 ----------------------------------------------
    "__except_handler2:",
    "    push ebp",
    "    mov ebp, esp",
    "    sub esp, 8",
    "    push ebx",
    "    push esi",
    "    push edi",
    "    push ebp",
    "    cld",
    "    mov ebx, [ebp+12]",
    "    mov eax, [ebp+8]",
    "    test dword ptr [eax+4], EXCEPTION_EXIT_UNWIND + EXCEPTION_UNWINDING",
    "    jnz except_unwind2",
    "    mov [ebp-8], eax",
    "    mov eax, [ebp+16]",
    "    mov [ebp-4], eax",
    "    lea eax, [ebp-8]",
    "    mov [ebx+20], eax",
    "    mov esi, [ebx+12]",
    "    mov edi, [ebx+8]",
    "except_loop2:",
    "    cmp esi, -1",
    "    je except_search2",
    "    lea ecx, [esi+esi*2]",
    "    cmp dword ptr [edi+ecx*4+4], 0",
    "    jz except_continue2",
    "    push esi",
    "    push ebp",
    "    mov ebp, [ebx+16]",
    "    call dword ptr [edi+ecx*4+4]",
    "    pop ebp",
    "    pop esi",
    "    mov ebx, [ebp+12]",
    "    or eax, eax",
    "    jz except_continue2",
    "    js except_dismiss2",
    "    mov edi, [ebx+8]",
    "    push ebx",
    "    call __global_unwind2",
    "    add esp, 4",
    "    mov ebp, [ebx+16]",
    "    push esi",
    "    push ebx",
    "    call __local_unwind2",
    "    add esp, 8",
    "    lea ecx, [esi+esi*2]",
    "    mov eax, [edi+ecx*4]",
    "    mov [ebx+12], eax",
    "    call dword ptr [edi+ecx*4+8]",
    "except_continue2:",
    "    mov edi, [ebx+8]",
    "    lea ecx, [esi+esi*2]",
    "    mov esi, [edi+ecx*4]",
    "    jmp except_loop2",
    "except_dismiss2:",
    "    mov eax, DISPOSITION_DISMISS",
    "    jmp except_return2",
    "except_search2:",
    "    mov eax, DISPOSITION_CONTINUE_SEARCH",
    "    jmp except_return2",
    "except_unwind2:",
    "    push ebp",
    "    mov ebp, [ebx+16]",
    "    push -1",
    "    push ebx",
    "    call __local_unwind2",
    "    add esp, 8",
    "    pop ebp",
    "    mov eax, DISPOSITION_CONTINUE_SEARCH",
    "except_return2:",
    "    pop ebp",
    "    pop edi",
    "    pop esi",
    "    pop ebx",
    "    mov esp, ebp",
    "    pop ebp",
    "    ret",

    // ---- __except_handler3 ----------------------------------------------
    "__except_handler3:",
    "    push ebp",
    "    mov ebp, esp",
    "    sub esp, 8",
    "    push ebx",
    "    push esi",
    "    push edi",
    "    push ebp",
    "    cld",
    "    mov ebx, [ebp+PARAM_RegistrationFrame]",
    "    mov eax, [ebp+PARAM_ExceptionRecord]",
    "    test dword ptr [eax+EXCEPTION_RECORD_EXCEPTION_FLAGS], EXCEPTION_EXIT_UNWIND + EXCEPTION_UNWINDING",
    "    jnz except_unwind3",
    "    mov [ebp-8+EXCEPTION_POINTERS_EXCEPTION_RECORD], eax",
    "    mov eax, [ebp+PARAM_Context]",
    "    mov [ebp-8+EXCEPTION_POINTERS_CONTEXT_RECORD], eax",
    "    lea eax, [ebp-8]",
    "    mov [ebx-SEH_FRAME_PreviousRecord+SEH_FRAME_unused], eax",
    "    mov esi, [ebx-SEH_FRAME_PreviousRecord+SEH_FRAME_Disable]",
    "    mov edi, [ebx-SEH_FRAME_PreviousRecord+SEH_FRAME_SEHTable]",
    "except_loop3:",
    "    cmp esi, -1",
    "    je except_search3",
    "    lea ecx, [esi+esi*2]",
    "    mov eax, [edi+ecx*4+4]",
    "    test eax, eax",
    "    jz except_continue3",
    "    push esi",
    "    push ebp",
    "    lea ebp, [ebx-SEH_FRAME_PreviousRecord+SEH_FRAME_OriginalEbp]",
    "    xor ebx, ebx",
    "    xor ecx, ecx",
    "    xor edx, edx",
    "    xor esi, esi",
    "    xor edi, edi",
    "    call eax",
    "    pop ebp",
    "    pop esi",
    "    mov ebx, [ebp+PARAM_RegistrationFrame]",
    "    test eax, eax",
    "    jz except_continue3",
    "    js except_dismiss3",
    "    mov edi, [ebx-SEH_FRAME_PreviousRecord+SEH_FRAME_SEHTable]",
    "    push ebx",
    "    call __global_unwind2",
    "    add esp, 4",
    "    lea ebp, [ebx-SEH_FRAME_PreviousRecord+SEH_FRAME_OriginalEbp]",
    "    push esi",
    "    push ebx",
    "    call __local_unwind2",
    "    add esp, 8",
    "    lea ecx, [esi+esi*2]",
    "    mov eax, [edi+ecx*4]",
    "    mov [ebx-SEH_FRAME_PreviousRecord+SEH_FRAME_Disable], eax",
    "    mov eax, [edi+ecx*4+8]",
    "    xor ebx, ebx",
    "    xor ecx, ecx",
    "    xor edx, edx",
    "    xor esi, esi",
    "    xor edi, edi",
    "    call eax",
    "except_continue3:",
    "    mov edi, [ebx-SEH_FRAME_PreviousRecord+SEH_FRAME_SEHTable]",
    "    lea ecx, [esi+esi*2]",
    "    mov esi, [edi+ecx*4]",
    "    jmp except_loop3",
    "except_dismiss3:",
    "    mov eax, DISPOSITION_DISMISS",
    "    jmp except_return3",
    "except_unwind3:",
    "    push ebp",
    "    lea ebp, [ebx-SEH_FRAME_PreviousRecord+SEH_FRAME_OriginalEbp]",
    "    push -1",
    "    push ebx",
    "    call __local_unwind2",
    "    add esp, 8",
    "    pop ebp",
    "except_search3:",
    "    mov eax, DISPOSITION_CONTINUE_SEARCH",
    "except_return3:",
    "    pop ebp",
    "    pop edi",
    "    pop esi",
    "    pop ebx",
    "    mov esp, ebp",
    "    pop ebp",
    "    ret",
);
