/* The Win32/CRT/msvcrt type surface `except_i386.c` reaches, transcribed for a 32-bit ELF
 * host.  Force-included by build.rs BEFORE the reference TU, which sees empty stand-ins in
 * place of windef.h / winbase.h / winternl.h / msvcrt.h / cppexcept.h / fpieee.h / setjmp.h /
 * excpt.h / wine's two headers.
 *
 * WHY A TRANSCRIPTION AND NOT THE REAL HEADERS
 * --------------------------------------------
 * The tree's PSDK headers are written for a PE target: `__declspec(dllimport)`, SAL, and
 * `#pragma pack` spellings that gcc rejects and clang only accepts under `-fdeclspec
 * -fms-extensions`, and they drag in the whole winnt.h/winbase.h closure for the eleven
 * structs this TU actually touches.  So the harness transcribes, exactly as the sibling
 * msvcrt-difftest's refstub/ does.
 *
 * WHAT MAKES THE TRANSCRIPTION SAFE
 * ---------------------------------
 * Nothing here is trusted.  `tools/gen_msvcrt_seh_layout.py` compiles the REAL i386 headers
 * with the pinned cross-compiler, reads back every offset and size, and emits
 * `seh_layout.h` — a wall of `_Static_assert`s this file is checked against at every build.
 * A field that moves, or a member this file spelled at the wrong width, is a compile error
 * naming the field.  That is the §9.66 class ("the oracle is not the code under test"), and
 * it is the one this harness would be most exposed to: the whole point of grading SEH is
 * that it walks structures by offset.
 *
 * ONE DELIBERATE DIVERGENCE: `__stdcall` is defined away, so the four `WINAPI` entry points
 * are cdecl on both sides.  That is not a shortcut — Rust's `extern "system"` IS cdecl on
 * `i686-unknown-linux-gnu` (it means stdcall only on 32-bit *Windows*), so defining it away
 * is what keeps the two sides' ABI in agreement.  On the product target both are stdcall,
 * again in agreement.  The convention is therefore never *graded* here; the i386 DLL link
 * is what proves it, and errno.rs's `_SetErrorMode@4` incident is the precedent for that
 * being a real check rather than a formality.
 */

#ifndef MSVCRT_SEH_TYPES_H
#define MSVCRT_SEH_TYPES_H

#include <stddef.h>
/* `uintptr_t` — cppexcept.h's `find_catch_handler` takes one, and in the real build it
 * arrives from the PSDK. 4 bytes here and 4 bytes there. */
#include <stdint.h>

/* `handle_fpieee_flt` calls both, and <string.h> is not reachable from the reference TU (it
 * would drag glibc's declarations into a translation unit that has already been told what
 * every Win32 name means).  Declared here so `-Werror=implicit-function-declaration` has
 * something to be satisfied by — an implicitly-declared `memcpy` returns `int`, which on a
 * pointer-returning call is the truncation class this flag exists to catch. */
void *memset(void *, int, size_t);
void *memcpy(void *, const void *, size_t);

/* ── calling conventions and attributes ────────────────────────────────── */
#define __stdcall
#define __cdecl
#define WINAPI
#define WINAPIV
#define CALLBACK
#define CDECL
#define DECLSPEC_NORETURN __attribute__((noreturn))
#define FORCEINLINE static inline __attribute__((always_inline))
#define DECLSPEC_IMPORT
#define WINE_UNUSED __attribute__((unused))

/* `__stdcall` above is defined AWAY, and this is defined as the real thing, and the two must
 * not be confused — they answer different Rust declarations.
 *
 *   * The seven graded EXPORTS are `extern "system"` in except_i386.rs, and on
 *     `i686-unknown-linux-gnu` that means CDECL ("system" is stdcall only on 32-bit
 *     *Windows*). So `__stdcall` on the reference side has to vanish for the two to agree.
 *   * The Win32 IMPORTS come from msvcrt-sys, which spells them `extern "stdcall"` —
 *     explicitly, and rustc honours that on x86 whatever the OS. So the harness's stand-ins
 *     for them must be stdcall, or the CALLEE-pops contract is broken at every call.
 *
 * Getting the second one wrong is not a link error and not a wrong answer: `OutputDebugStringA`
 * declared cdecl here left four bytes on the stack per call, and the `ret` that followed went
 * to a stack-resident value — a SIGSEGV with the PC inside the stack, nowhere near the cause.
 * Same family as errno.rs's `_SetErrorMode@4`, from the other direction. */
#define SEH_WINAPI __attribute__((stdcall))

/* ── scalars ──────────────────────────────────────────────────────────── */
typedef unsigned char BYTE;
typedef unsigned short WORD;
typedef unsigned long DWORD;
typedef long LONG;
typedef unsigned long ULONG;
typedef int INT;
typedef unsigned int UINT;
typedef int BOOL;
typedef void *PVOID;
typedef void *LPVOID;
typedef void *HANDLE;
typedef void *HMODULE;
typedef unsigned short WCHAR;
typedef unsigned long ULONG_PTR;
typedef long LONG_PTR;
typedef unsigned long DWORD_PTR;
typedef unsigned long __msvcrt_ulong;
typedef long long LONGLONG;
typedef unsigned long long ULONGLONG;

#ifndef TRUE
#define TRUE 1
#endif
#ifndef FALSE
#define FALSE 0
#endif
#ifndef NULL
#define NULL ((void *)0)
#endif

/* ── exception dispositions and flags (winnt.h) ────────────────────────── */
typedef enum _EXCEPTION_DISPOSITION
{
    ExceptionContinueExecution,
    ExceptionContinueSearch,
    ExceptionNestedException,
    ExceptionCollidedUnwind
} EXCEPTION_DISPOSITION;

#define EXCEPTION_NONCONTINUABLE  0x01
#define EXCEPTION_UNWINDING       0x02
#define EXCEPTION_EXIT_UNWIND     0x04
#define EXCEPTION_STACK_INVALID   0x08
#define EXCEPTION_NESTED_CALL     0x10
#define EXCEPTION_TARGET_UNWIND   0x20
#define EXCEPTION_COLLIDED_UNWIND 0x40

#define EXCEPTION_MAXIMUM_PARAMETERS 15

#define EXCEPTION_EXECUTE_HANDLER    1
#define EXCEPTION_CONTINUE_SEARCH    0
#define EXCEPTION_CONTINUE_EXECUTION (-1)

#define STATUS_FLOAT_DIVIDE_BY_ZERO      ((DWORD)0xc000008eL)
#define STATUS_FLOAT_INVALID_OPERATION   ((DWORD)0xc0000090L)
#define STATUS_FLOAT_OVERFLOW            ((DWORD)0xc0000091L)
#define STATUS_FLOAT_UNDERFLOW           ((DWORD)0xc0000093L)
#define STATUS_FLOAT_INEXACT_RESULT      ((DWORD)0xc000008fL)

typedef struct _EXCEPTION_RECORD
{
    DWORD ExceptionCode;
    DWORD ExceptionFlags;
    struct _EXCEPTION_RECORD *ExceptionRecord;
    PVOID ExceptionAddress;
    DWORD NumberParameters;
    ULONG_PTR ExceptionInformation[EXCEPTION_MAXIMUM_PARAMETERS];
} EXCEPTION_RECORD, *PEXCEPTION_RECORD;

/* i386's CONTEXT.  `handle_fpieee_flt` reads FloatSave field by field, so this is the one
 * struct here whose exact shape is load-bearing rather than incidental. */
#define SIZE_OF_80387_REGISTERS 80
#define MAXIMUM_SUPPORTED_EXTENSION 512

typedef struct _FLOATING_SAVE_AREA
{
    DWORD ControlWord;
    DWORD StatusWord;
    DWORD TagWord;
    DWORD ErrorOffset;
    DWORD ErrorSelector;
    DWORD DataOffset;
    DWORD DataSelector;
    BYTE RegisterArea[SIZE_OF_80387_REGISTERS];
    DWORD Cr0NpxState;
} FLOATING_SAVE_AREA;

typedef struct _CONTEXT
{
    DWORD ContextFlags;
    DWORD Dr0, Dr1, Dr2, Dr3, Dr6, Dr7;
    FLOATING_SAVE_AREA FloatSave;
    DWORD SegGs, SegFs, SegEs, SegDs;
    DWORD Edi, Esi, Ebx, Edx, Ecx, Eax;
    DWORD Ebp, Eip, SegCs, EFlags, Esp, SegSs;
    BYTE ExtendedRegisters[MAXIMUM_SUPPORTED_EXTENSION];
} CONTEXT, *PCONTEXT;

typedef struct _EXCEPTION_POINTERS
{
    PEXCEPTION_RECORD ExceptionRecord;
    PCONTEXT ContextRecord;
} EXCEPTION_POINTERS, *PEXCEPTION_POINTERS;

struct _EXCEPTION_REGISTRATION_RECORD;
typedef EXCEPTION_DISPOSITION (*PEXCEPTION_ROUTINE)(
    struct _EXCEPTION_RECORD *, PVOID, struct _CONTEXT *, PVOID);

typedef struct _EXCEPTION_REGISTRATION_RECORD
{
    struct _EXCEPTION_REGISTRATION_RECORD *Prev;
    PEXCEPTION_ROUTINE Handler;
} EXCEPTION_REGISTRATION_RECORD, *PEXCEPTION_REGISTRATION_RECORD;

/* ── the TEB, as far as fs:0 and fs:[0x18] ─────────────────────────────── */
typedef struct _NT_TIB
{
    struct _EXCEPTION_REGISTRATION_RECORD *ExceptionList;
    PVOID StackBase;
    PVOID StackLimit;
    PVOID SubSystemTib;
    union
    {
        PVOID FiberData;
        DWORD Version;
    };
    PVOID ArbitraryUserPointer;
    struct _NT_TIB *Self;
} NT_TIB;

typedef struct _TEB
{
    NT_TIB Tib;
    PVOID EnvironmentPointer;
    /* the harness never reads past here; seh_layout.h pins Tib's offset and shape */
} TEB;

/* ── setjmp.h's MS `_JUMP_BUFFER` (i386) ───────────────────────────────── */
typedef struct __JUMP_BUFFER
{
    unsigned long Ebp;
    unsigned long Ebx;
    unsigned long Edi;
    unsigned long Esi;
    unsigned long Esp;
    unsigned long Eip;
    unsigned long Registration;
    unsigned long TryLevel;
    unsigned long Cookie;
    unsigned long UnwindFunc;
    unsigned long UnwindData[6];
} _JUMP_BUFFER;

/* ── fpieee.h ──────────────────────────────────────────────────────────── */
#define _FpCodeUnspecified 0
#define _FpCodeAdd         1
#define _FpCodeSubtract    2
#define _FpCodeMultiply    3
#define _FpCodeDivide      4

#define _FpFormatFp32   0
#define _FpFormatFp64   1
#define _FpFormatFp80   2

typedef float _FP32;
typedef double _FP64;

typedef struct
{
    unsigned short W[5];
} _FP80;

/* `DECLSPEC_ALIGN(16)` in the real header, and that alignment is the whole reason
 * `_FPIEEE_VALUE` is 32 bytes rather than 24 — drop it and every Operand/Result offset in
 * the record shifts.  seh_layout.h is what caught this. */
typedef struct __attribute__((aligned(16)))
{
    unsigned long W[4];
} _FP128;

/* The union comes FIRST and the two bit-fields follow it; the real header is written that
 * way and the offsets depend on it. */
typedef struct
{
    union
    {
        _FP32 Fp32Value;
        _FP64 Fp64Value;
        _FP80 Fp80Value;
        _FP128 Fp128Value;
        short I16Value;
        int I32Value;
        unsigned short U16Value;
        unsigned int U32Value;
        char *StringValue;
        int CompareValue;
        long long Q64Value;
    } Value;
    unsigned int OperandValid : 1;
    unsigned int Format : 4;
} _FPIEEE_VALUE;

typedef struct
{
    unsigned int Inexact : 1;
    unsigned int Underflow : 1;
    unsigned int Overflow : 1;
    unsigned int ZeroDivide : 1;
    unsigned int InvalidOperation : 1;
} _FPIEEE_EXCEPTION_FLAGS;

typedef struct
{
    unsigned int RoundingMode : 2;
    unsigned int Precision : 3;
    unsigned int Operation : 12;
    _FPIEEE_EXCEPTION_FLAGS Cause;
    _FPIEEE_EXCEPTION_FLAGS Enable;
    _FPIEEE_EXCEPTION_FLAGS Status;
    _FPIEEE_VALUE Operand1;
    _FPIEEE_VALUE Operand2;
    _FPIEEE_VALUE Result;
} _FPIEEE_RECORD, *PFPIEEE_RECORD;

/* ── cppexcept.h / cxx.h, cut to what except_i386.c names ──────────────── */
#define CXX_FRAME_MAGIC_VC6 0x19930520
#define CXX_FRAME_MAGIC_VC7 0x19930521
#define CXX_FRAME_MAGIC_VC8 0x19930522
#define CXX_EXCEPTION       0xe06d7363

#define FUNC_DESCR_SYNCHRONOUS 1
#define FUNC_DESCR_NOEXCEPT    4

typedef struct
{
    UINT ip;
    int state;
} ipmap_info;

typedef struct
{
    UINT flags;
    const void *type_info;
    int offset;
    void *(*handler)(void);
} catchblock_info;

typedef struct
{
    int start_level;
    int end_level;
    int catch_level;
    unsigned int catchblock_count;
    const catchblock_info *catchblock;
} tryblock_info;

typedef struct
{
    int prev;
    void *(*handler)(void);
} unwind_info;

typedef struct
{
    UINT magic : 29;
    UINT bbt_flags : 3;
    UINT unwind_count;
    const unwind_info *unwind_table;
    UINT tryblock_count;
    const tryblock_info *tryblock;
    UINT ipmap_count;
    const ipmap_info *ipmap;
    const void *expect_list;
    UINT flags;
} cxx_function_descr;

typedef struct
{
    UINT count;
    const void *info[5];
} cxx_type_info_table;

typedef struct
{
    UINT flags;
    void *destructor;
    void *custom_handler;
    const cxx_type_info_table *type_info_table;
} cxx_exception_type;

typedef struct _frame_info
{
    void *object;
    struct _frame_info *next;
} frame_info;

typedef struct
{
    frame_info frame_info;
    EXCEPTION_RECORD *rec;
    CONTEXT *context;
} cxx_frame_info;

/* ── msvcrt.h's thread_data_t ──────────────────────────────────────────── */
typedef void(__cdecl *_se_translator_function)(unsigned int, struct _EXCEPTION_POINTERS *);
typedef void(__cdecl *terminate_function)(void);
typedef void(__cdecl *unexpected_function)(void);

#define LOCALE_NAME_MAX_LENGTH 85

struct __thread_data
{
    DWORD tid;
    HANDLE handle;
    int thread_errno;
    __msvcrt_ulong thread_doserrno;
    int unk1;
    unsigned int random_seed;
    char *strtok_next;
    WCHAR *wcstok_next;
    unsigned char *mbstok_next;
    char *strerror_buffer;
    WCHAR *wcserror_buffer;
    char *tmpnam_buffer;
    WCHAR *wtmpnam_buffer;
    void *unk2[2];
    char *asctime_buffer;
    WCHAR *wasctime_buffer;
    void *time_buffer;
    char *efcvt_buffer;
    int unk3[2];
    void *unk4[3];
    EXCEPTION_POINTERS *xcptinfo;
    int fpecode;
    void *mbcinfo;
    void *locinfo;
    int locale_flags;
    int unk5[1];
    terminate_function terminate_handler;
    unexpected_function unexpected_handler;
    _se_translator_function se_translator;
    void *unk6;
    EXCEPTION_RECORD *exc_record;
    CONTEXT *ctx_record;
    int processing_throw;
    frame_info *frame_info_head;
    void *unk8[6];
    BOOL cached_sname_match;
    WCHAR cached_sname[LOCALE_NAME_MAX_LENGTH];
    int unk9[2];
    DWORD cached_cp;
    char cached_locale[131];
    void *unk10[100];
};

typedef struct __thread_data thread_data_t;

#endif /* MSVCRT_SEH_TYPES_H */
