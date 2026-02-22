/*
 * ob_rust_shims.c — Non-inline ("outline") implementations of FORCEINLINE/macro
 * functions that the Rust ntoskrnl-rust crate calls via extern declarations.
 *
 * Inside ntoskrnl, many kernel functions are macros or FORCEINLINE (static inline).
 * The C compiler inlines them, producing no linker symbol. Rust extern declarations
 * need real symbols. This file provides them.
 *
 * Technique:
 *   1. #define FORCEINLINE function names to _fi_* BEFORE including headers,
 *      so the static inline definitions use renamed identifiers.
 *   2. Include ntoskrnl.h (which pulls in all internal headers).
 *   3. Create trampoline helpers for macros (KeEnterCriticalRegion etc.).
 *   4. #undef macros and renames.
 *   5. Define real (non-static, non-inline) functions with the original names.
 */

/* === Phase 1: Rename FORCEINLINE static inline functions before headers === */

/* ob_x.h FORCEINLINE functions */
#define ObpAcquireObjectLock              _fi_ObpAcquireObjectLock
#define ObpReleaseObjectLock              _fi_ObpReleaseObjectLock
#define ObpCalloutStart                   _fi_ObpCalloutStart
#define ObpCalloutEnd                     _fi_ObpCalloutEnd
#define ObpDereferenceNameInfo            _fi_ObpDereferenceNameInfo
#define ObpEnterObjectTypeMutex           _fi_ObpEnterObjectTypeMutex
#define ObpLeaveObjectTypeMutex           _fi_ObpLeaveObjectTypeMutex
#define ObpFreeObjectCreateInformation    _fi_ObpFreeObjectCreateInformation
#define ObpInitializeLookupContext        _fi_ObpInitializeLookupContext
#define ObpReferenceNameInfo              _fi_ObpReferenceNameInfo
#define ObpReleaseLookupContext           _fi_ObpReleaseLookupContext
#define ObpReleaseObjectCreateInformation _fi_ObpReleaseObjectCreateInformation
/* Also rename helpers that are only called by renamed functions: */
#define ObpSelectObjectLockSlot           _fi_ObpSelectObjectLockSlot
#define ObpValidateAttributes             _fi_ObpValidateAttributes
#define ObpAcquireObjectLockShared        _fi_ObpAcquireObjectLockShared
#define ObpAcquireDirectoryLockShared     _fi_ObpAcquireDirectoryLockShared
#define ObpAcquireDirectoryLockExclusive  _fi_ObpAcquireDirectoryLockExclusive
#define ObpReleaseDirectoryLock           _fi_ObpReleaseDirectoryLock
#define ObpAcquireLookupContextLock       _fi_ObpAcquireLookupContextLock
#define ObpReleaseLookupContextObject     _fi_ObpReleaseLookupContextObject
#define ObpAllocateObjectCreateInfoBuffer _fi_ObpAllocateObjectCreateInfoBuffer
#define ObpFreeCapturedAttributes         _fi_ObpFreeCapturedAttributes

/* psfuncs.h FORCEINLINE function */
#define PsGetCurrentThread                _fi_PsGetCurrentThread

/* ex.h FORCEINLINE fast reference functions */
#define ExGetObjectFastReference          _fi_ExGetObjectFastReference
#define ExGetCountFastReference           _fi_ExGetCountFastReference
#define ExInitializeFastReference         _fi_ExInitializeFastReference
#define ExAcquireFastReference            _fi_ExAcquireFastReference
#define ExInsertFastReference             _fi_ExInsertFastReference
#define ExReleaseFastReference            _fi_ExReleaseFastReference
#define ExSwapFastReference               _fi_ExSwapFastReference
#define ExCompareSwapFastReference        _fi_ExCompareSwapFastReference

/* ex.h FORCEINLINE push lock functions
 * ExInitializePushLock is a #define macro in ex.h that redirects to
 * _ExInitializePushLock (FORCEINLINE). We rename it here so the ntifs.h
 * prototype (which uses the bare name) gets harmlessly renamed too.
 * The ex.h macro will redefine it to _ExInitializePushLock. */
#define ExInitializePushLock              _fi_ExInitializePushLock
#define ExAcquirePushLockExclusive        _fi_ExAcquirePushLockExclusive
#define ExAcquirePushLockShared           _fi_ExAcquirePushLockShared
#define ExReleasePushLock                 _fi_ExReleasePushLock
#define ExReleasePushLockExclusive        _fi_ExReleasePushLockExclusive
#define ExConvertPushLockSharedToExclusive _fi_ExConvertPushLockSharedToExclusive

/* ndk ketypes.h FORCEINLINE function */
#define KeGetCurrentPrcb                  _fi_KeGetCurrentPrcb

/* sdk/rtlfuncs.h FORCEINLINE function */
#define RtlInitEmptyUnicodeString         _fi_RtlInitEmptyUnicodeString

/* === Phase 2: Include headers === */
/* Now all FORCEINLINE static functions get renamed _fi_* names. */
/* Macros (KeEnterCriticalRegion, Ex*RundownProtection) are defined normally. */

#include <ntoskrnl.h>
/* === Phase 3: Trampolines for macro-defined functions === */
/* These capture the macro expansion into static inline functions. */

static __inline void _tramp_KeEnterCriticalRegion(void)
{
    KeEnterCriticalRegion();
}

static __inline void _tramp_KeLeaveCriticalRegion(void)
{
    KeLeaveCriticalRegion();
}

static __inline BOOLEAN _tramp_ExAcquireRundownProtection(PEX_RUNDOWN_REF RunRef)
{
    return ExAcquireRundownProtection(RunRef);
}

static __inline void _tramp_ExReleaseRundownProtection(PEX_RUNDOWN_REF RunRef)
{
    ExReleaseRundownProtection(RunRef);
}

static __inline void _tramp_ExWaitForRundownProtectionRelease(PEX_RUNDOWN_REF RunRef)
{
    ExWaitForRundownProtectionRelease(RunRef);
}

static __inline void _tramp_ExRundownCompleted(PEX_RUNDOWN_REF RunRef)
{
    ExRundownCompleted(RunRef);
}

/* ExInitializePushLock is a #define macro → _ExInitializePushLock (FORCEINLINE).
 * Must use trampoline since the macro overwrites Phase 1 renames. */
static __inline void _tramp_ExInitializePushLock(PEX_PUSH_LOCK PushLock)
{
    ExInitializePushLock(PushLock);
}

/* KeQueryInterruptTime is a #define macro on ARM64 and amd64, a real function on i386 */
#if defined(_M_ARM64) || defined(_M_AMD64)
static __inline ULONGLONG _tramp_KeQueryInterruptTime(void)
{
    return KeQueryInterruptTime();
}
#endif

/* === Phase 4: Undef all macros and renames === */

/* Macros from ke_x.h */
#undef KeEnterCriticalRegion
#undef KeLeaveCriticalRegion

/* Macros from ex.h */
#undef ExAcquireRundownProtection
#undef ExReleaseRundownProtection
#undef ExWaitForRundownProtectionRelease
#undef ExRundownCompleted

/* Macros from ex.h (push lock) */
#undef ExInitializePushLock

/* Macros from arm64/ke.h and amd64 wdm.h */
#if defined(_M_ARM64) || defined(_M_AMD64)
#undef KeQueryInterruptTime
#endif

/* Renames from Phase 1 */
#undef ObpAcquireObjectLock
#undef ObpReleaseObjectLock
#undef ObpCalloutStart
#undef ObpCalloutEnd
#undef ObpDereferenceNameInfo
#undef ObpEnterObjectTypeMutex
#undef ObpLeaveObjectTypeMutex
#undef ObpFreeObjectCreateInformation
#undef ObpInitializeLookupContext
#undef ObpReferenceNameInfo
#undef ObpReleaseLookupContext
#undef ObpReleaseObjectCreateInformation
#undef ObpSelectObjectLockSlot
#undef ObpValidateAttributes
#undef ObpAcquireObjectLockShared
#undef ObpAcquireDirectoryLockShared
#undef ObpAcquireDirectoryLockExclusive
#undef ObpReleaseDirectoryLock
#undef ObpAcquireLookupContextLock
#undef ObpReleaseLookupContextObject
#undef ObpAllocateObjectCreateInfoBuffer
#undef ObpFreeCapturedAttributes
#undef PsGetCurrentThread
#undef ExGetObjectFastReference
#undef ExGetCountFastReference
#undef ExInitializeFastReference
#undef ExAcquireFastReference
#undef ExInsertFastReference
#undef ExReleaseFastReference
#undef ExSwapFastReference
#undef ExCompareSwapFastReference
#undef ExAcquirePushLockExclusive
#undef ExAcquirePushLockShared
#undef ExReleasePushLock
#undef ExReleasePushLockExclusive
#undef ExConvertPushLockSharedToExclusive
#undef KeGetCurrentPrcb
#undef RtlInitEmptyUnicodeString

/* === Phase 5: Real (non-static, non-inline) functions === */

/* --- ke_x.h macros --- */

void KeEnterCriticalRegion(void)
{
    _tramp_KeEnterCriticalRegion();
}

void KeLeaveCriticalRegion(void)
{
    _tramp_KeLeaveCriticalRegion();
}

/* --- ex.h rundown protection macros → FORCEINLINE --- */

BOOLEAN ExAcquireRundownProtection(PEX_RUNDOWN_REF RunRef)
{
    return _tramp_ExAcquireRundownProtection(RunRef);
}

void ExReleaseRundownProtection(PEX_RUNDOWN_REF RunRef)
{
    _tramp_ExReleaseRundownProtection(RunRef);
}

void ExWaitForRundownProtectionRelease(PEX_RUNDOWN_REF RunRef)
{
    _tramp_ExWaitForRundownProtectionRelease(RunRef);
}

void ExRundownCompleted(PEX_RUNDOWN_REF RunRef)
{
    _tramp_ExRundownCompleted(RunRef);
}

/* --- psfuncs.h --- */

PETHREAD NTAPI PsGetCurrentThread(void)
{
    return _fi_PsGetCurrentThread();
}

/* --- ob_x.h FORCEINLINE functions --- */

void ObpAcquireObjectLock(POBJECT_HEADER ObjectHeader)
{
    _fi_ObpAcquireObjectLock(ObjectHeader);
}

void ObpReleaseObjectLock(POBJECT_HEADER ObjectHeader)
{
    _fi_ObpReleaseObjectLock(ObjectHeader);
}

void ObpCalloutStart(PKIRQL CalloutIrql)
{
    _fi_ObpCalloutStart(CalloutIrql);
}

void ObpCalloutEnd(KIRQL CalloutIrql, PCHAR Procedure,
                   POBJECT_TYPE ObjectType, PVOID Object)
{
    _fi_ObpCalloutEnd(CalloutIrql, Procedure, ObjectType, Object);
}

POBJECT_HEADER_NAME_INFO ObpReferenceNameInfo(POBJECT_HEADER ObjectHeader)
{
    return _fi_ObpReferenceNameInfo(ObjectHeader);
}

void ObpDereferenceNameInfo(POBJECT_HEADER_NAME_INFO HeaderNameInfo)
{
    _fi_ObpDereferenceNameInfo(HeaderNameInfo);
}

void ObpEnterObjectTypeMutex(POBJECT_TYPE ObjectType)
{
    _fi_ObpEnterObjectTypeMutex(ObjectType);
}

void ObpLeaveObjectTypeMutex(POBJECT_TYPE ObjectType)
{
    _fi_ObpLeaveObjectTypeMutex(ObjectType);
}

void ObpInitializeLookupContext(POBP_LOOKUP_CONTEXT Context)
{
    _fi_ObpInitializeLookupContext(Context);
}

void ObpReleaseLookupContext(POBP_LOOKUP_CONTEXT Context)
{
    _fi_ObpReleaseLookupContext(Context);
}

void ObpReleaseObjectCreateInformation(POBJECT_CREATE_INFORMATION ObjectCreateInfo)
{
    _fi_ObpReleaseObjectCreateInformation(ObjectCreateInfo);
}

void ObpFreeObjectCreateInformation(POBJECT_CREATE_INFORMATION ObjectCreateInfo)
{
    _fi_ObpFreeObjectCreateInformation(ObjectCreateInfo);
}

/* --- ob_x.h FORCEINLINE functions (directory/lookup/misc) --- */

void ObpAcquireDirectoryLockShared(POBJECT_DIRECTORY Directory,
                                   POBP_LOOKUP_CONTEXT Context)
{
    _fi_ObpAcquireDirectoryLockShared(Directory, Context);
}

void ObpAcquireDirectoryLockExclusive(POBJECT_DIRECTORY Directory,
                                      POBP_LOOKUP_CONTEXT Context)
{
    _fi_ObpAcquireDirectoryLockExclusive(Directory, Context);
}

void ObpReleaseDirectoryLock(POBJECT_DIRECTORY Directory,
                             POBP_LOOKUP_CONTEXT Context)
{
    _fi_ObpReleaseDirectoryLock(Directory, Context);
}

void ObpAcquireLookupContextLock(POBP_LOOKUP_CONTEXT Context,
                                 POBJECT_DIRECTORY Directory)
{
    _fi_ObpAcquireLookupContextLock(Context, Directory);
}

void ObpReleaseLookupContextObject(POBP_LOOKUP_CONTEXT Context)
{
    _fi_ObpReleaseLookupContextObject(Context);
}

void ObpAcquireObjectLockShared(POBJECT_HEADER ObjectHeader)
{
    _fi_ObpAcquireObjectLockShared(ObjectHeader);
}

PVOID ObpAllocateObjectCreateInfoBuffer(PP_NPAGED_LOOKASIDE_NUMBER Type)
{

    return _fi_ObpAllocateObjectCreateInfoBuffer(Type);
}

void ObpFreeCapturedAttributes(PVOID Buffer,
                               PP_NPAGED_LOOKASIDE_NUMBER Type)
{
    _fi_ObpFreeCapturedAttributes(Buffer, Type);
}

ULONG ObpSelectObjectLockSlot(POBJECT_HEADER ObjectHeader)
{
    return _fi_ObpSelectObjectLockSlot(ObjectHeader);
}

ULONG NTAPI ObpValidateAttributes(ULONG Attributes,
                                   KPROCESSOR_MODE PreviousMode)
{
    return _fi_ObpValidateAttributes(Attributes, PreviousMode);
}

/* --- ex.h FORCEINLINE fast reference functions --- */

PVOID ExGetObjectFastReference(EX_FAST_REF FastRef)
{
    return _fi_ExGetObjectFastReference(FastRef);
}

ULONG ExGetCountFastReference(EX_FAST_REF FastRef)
{
    return _fi_ExGetCountFastReference(FastRef);
}

void ExInitializeFastReference(PEX_FAST_REF FastRef, PVOID Object)
{
    _fi_ExInitializeFastReference(FastRef, Object);
}

EX_FAST_REF ExAcquireFastReference(PEX_FAST_REF FastRef)
{
    return _fi_ExAcquireFastReference(FastRef);
}

BOOLEAN ExInsertFastReference(PEX_FAST_REF FastRef, PVOID Object)
{
    return _fi_ExInsertFastReference(FastRef, Object);
}

BOOLEAN ExReleaseFastReference(PEX_FAST_REF FastRef, PVOID Object)
{
    return _fi_ExReleaseFastReference(FastRef, Object);
}

EX_FAST_REF ExSwapFastReference(PEX_FAST_REF FastRef, PVOID Object)
{
    return _fi_ExSwapFastReference(FastRef, Object);
}

EX_FAST_REF ExCompareSwapFastReference(PEX_FAST_REF FastRef,
                                        PVOID Object,
                                        PVOID OldObject)
{
    return _fi_ExCompareSwapFastReference(FastRef, Object, OldObject);
}

/* --- ex.h push lock functions --- */

VOID NTAPI ExInitializePushLock(PEX_PUSH_LOCK PushLock)
{
    _tramp_ExInitializePushLock(PushLock);
}

void ExAcquirePushLockExclusive(PEX_PUSH_LOCK PushLock)
{
    _fi_ExAcquirePushLockExclusive(PushLock);
}

void ExAcquirePushLockShared(PEX_PUSH_LOCK PushLock)
{
    _fi_ExAcquirePushLockShared(PushLock);
}

void ExReleasePushLock(PEX_PUSH_LOCK PushLock)
{
    _fi_ExReleasePushLock(PushLock);
}

void ExReleasePushLockExclusive(PEX_PUSH_LOCK PushLock)
{
    _fi_ExReleasePushLockExclusive(PushLock);
}

BOOLEAN ExConvertPushLockSharedToExclusive(PEX_PUSH_LOCK PushLock)
{
    return _fi_ExConvertPushLockSharedToExclusive(PushLock);
}

/* --- ke FORCEINLINE functions --- */

PKPRCB NTAPI KeGetCurrentPrcb(void)
{
    return _fi_KeGetCurrentPrcb();
}

/* KeQueryInterruptTime: macro on ARM64 and amd64, real function on i386 */
#if defined(_M_ARM64) || defined(_M_AMD64)
ULONGLONG NTAPI KeQueryInterruptTime(void)
{
    return _tramp_KeQueryInterruptTime();
}
#endif

/* --- rtl FORCEINLINE functions --- */

void RtlInitEmptyUnicodeString(PUNICODE_STRING UnicodeString,
                               PWCHAR Buffer,
                               USHORT BufferSize)
{
    _fi_RtlInitEmptyUnicodeString(UnicodeString, Buffer, BufferSize);
}

/* ObpSetHandleAttributes — was in obhandle.c, now in Rust.
 * The Rust version defines it locally for ExChangeHandle callback usage.
 * No external C callers, so no shim needed. */
