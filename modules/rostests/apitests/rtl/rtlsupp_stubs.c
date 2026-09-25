/*
 * rtl<->host glue for the statically-linked RTL test exe.
 *
 * rtl_unittest links sdk/lib/rtl directly rather than going through ntdll, so
 * it needs the rtlp.h support layer (rtlp.h: "the caller supplies these") the
 * way ldrstubs.c supplies the loader's. Upstream gets them from the `rtl_um`
 * OBJECT library, which compiled dll/ntdll/rtl/libsupp.c -- but that TU is
 * cohab-flipped in this tree (ntdll-rust/src/rtl/libsupp.rs owns the symbols
 * now, inside libntdll_rust.a, which is linked only into ntdll.dll). rtl_um is
 * therefore an empty stub library, and rtl_unittest has not linked since the
 * flip. Nothing noticed, because nothing in this tree builds rostests.
 *
 * These are user-mode definitions, matching ntdll-rust/src/rtl/libsupp.rs
 * one-for-one; they cannot collide with it, because rtl_um's only consumer is
 * this exe (dll/ntdll/CMakeLists.txt says so) and this exe does not link ntdll's
 * Rust.
 */
#include <rtltests.h>

#define NDEBUG
#include <debug.h>

PVOID
NTAPI
RtlpAllocateMemory(SIZE_T Bytes, ULONG Tag)
{
    UNREFERENCED_PARAMETER(Tag);
    return RtlAllocateHeap(NtCurrentPeb()->ProcessHeap, 0, Bytes);
}

VOID
NTAPI
RtlpFreeMemory(PVOID Mem, ULONG Tag)
{
    UNREFERENCED_PARAMETER(Tag);
    RtlFreeHeap(NtCurrentPeb()->ProcessHeap, 0, Mem);
}

/*
 * `RtlGetProcessHeap` is a MACRO in ntrtl.h, so the definition has to undef it
 * first -- otherwise `RtlGetProcessHeap(VOID)` is parsed as an invocation of
 * that macro. rtl.a calls the out-of-line symbol, which nothing then provides.
 */
#undef RtlGetProcessHeap
PVOID
NTAPI
RtlGetProcessHeap(VOID)
{
    return NtCurrentPeb()->ProcessHeap;
}

KPROCESSOR_MODE
NTAPI
RtlpGetMode(VOID)
{
    return UserMode;
}

BOOLEAN
NTAPI
RtlpCheckForActiveDebugger(VOID)
{
    return NtCurrentPeb()->BeingDebugged;
}

BOOLEAN
NTAPI
RtlpSetInDbgPrint(VOID)
{
    if (NtCurrentTeb()->InDbgPrint)
        return TRUE;

    NtCurrentTeb()->InDbgPrint = TRUE;
    return FALSE;
}

VOID
NTAPI
RtlpClearInDbgPrint(VOID)
{
    NtCurrentTeb()->InDbgPrint = FALSE;
}

VOID
NTAPI
RtlpGetStackLimits(PULONG_PTR LowLimit, PULONG_PTR HighLimit)
{
    *LowLimit = (ULONG_PTR)NtCurrentTeb()->NtTib.StackLimit;
    *HighLimit = (ULONG_PTR)NtCurrentTeb()->NtTib.StackBase;
}

VOID
NTAPI
RtlpCheckLogException(
    PEXCEPTION_RECORD ExceptionRecord,
    PCONTEXT ContextRecord,
    PVOID ContextData,
    ULONG Size)
{
    UNREFERENCED_PARAMETER(ExceptionRecord);
    UNREFERENCED_PARAMETER(ContextRecord);
    UNREFERENCED_PARAMETER(ContextData);
    UNREFERENCED_PARAMETER(Size);
}
