/*
 * kmt_platform.h — Host shim replacing ReactOS kmt_platform.h
 */
#ifndef _KMTEST_PLATFORM_H_
#define _KMTEST_PLATFORM_H_

#include "nt_host.h"

#ifndef KMT_EMULATE_KERNEL
#define KMT_EMULATE_KERNEL
#endif

/* Note: KMT_USER_MODE is NOT defined here. In the real ReactOS test framework,
 * KMT_EMULATE_KERNEL and KMT_USER_MODE are for different build targets.
 * Our host shim uses KMT_EMULATE_KERNEL mode. Tests that #ifdef KMT_USER_MODE
 * will have those sections skipped (e.g., stack overflow test in RtlException). */

/* ====================================================================
 * Flag macros (NT kernel fundamentals)
 * ==================================================================== */

#ifndef FlagOn
#define FlagOn(x, f)         ((x) & (f))
#endif
#ifndef BooleanFlagOn
#define BooleanFlagOn(x, f)  (((x) & (f)) != 0)
#endif
#ifndef SetFlag
#define SetFlag(x, f)        ((x) |= (f))
#endif
#ifndef ClearFlag
#define ClearFlag(x, f)      ((x) &= ~(f))
#endif

/* signed char type */
typedef signed char SCHAR;

/* ====================================================================
 * FsRtl character classification
 * ==================================================================== */

/* Character type flags */
#define FSRTL_FAT_LEGAL             0x01
#define FSRTL_HPFS_LEGAL            0x02
#define FSRTL_NTFS_LEGAL            0x04
#define FSRTL_WILD_CHARACTER        0x08
#define FSRTL_OLE_LEGAL             0x10
#define FSRTL_NTFS_STREAM_LEGAL     (FSRTL_NTFS_LEGAL | FSRTL_OLE_LEGAL)

/* DOS wildcard special characters */
#define ANSI_DOS_STAR   ('<')
#define ANSI_DOS_QM     ('>')
#define ANSI_DOS_DOT    ('"')
#define DOS_STAR        (L'<')
#define DOS_QM          (L'>')
#define DOS_DOT         (L'"')

/* Legal ANSI character data — raw array for macro use */
extern const UCHAR * const _FsRtlLegalCharData;
#define LEGAL_ANSI_CHARACTER_ARRAY _FsRtlLegalCharData

/*
 * FsRtlLegalAnsiCharacterArray — simulates PE dllimport double indirection.
 * In the actual kernel, FsRtlLegalAnsiCharacterArray is a const UCHAR* exported
 * from ntoskrnl.exe. When imported by a kmtest driver, the import thunk adds
 * one level of indirection, so the test uses (*FsRtlLegalAnsiCharacterArray)[i].
 * We simulate this by providing a pointer-to-pointer.
 */
extern const UCHAR * const * const _FsRtlLegalCharDataPtr;
#define FsRtlLegalAnsiCharacterArray _FsRtlLegalCharDataPtr

/* Character test macros */
#define FsRtlTestAnsiCharacter(C, DEFAULT_RET, WILD_OK, FLAGS) \
    (((SCHAR)(C) < 0) ? DEFAULT_RET : \
        FlagOn(LEGAL_ANSI_CHARACTER_ARRAY[(int)(UCHAR)(C)], \
               (FLAGS) | ((WILD_OK) ? FSRTL_WILD_CHARACTER : 0)))

#define FsRtlIsAnsiCharacterWild(C) \
    FsRtlTestAnsiCharacter((C), FALSE, FALSE, FSRTL_WILD_CHARACTER)

#define FsRtlIsAnsiCharacterLegalFat(C, WILD) \
    FsRtlTestAnsiCharacter((C), TRUE, (WILD), FSRTL_FAT_LEGAL)

#define FsRtlIsAnsiCharacterLegalHpfs(C, WILD) \
    FsRtlTestAnsiCharacter((C), TRUE, (WILD), FSRTL_HPFS_LEGAL)

#define FsRtlIsAnsiCharacterLegalNtfs(C, WILD) \
    FsRtlTestAnsiCharacter((C), TRUE, (WILD), FSRTL_NTFS_LEGAL)

#define FsRtlIsAnsiCharacterLegalNtfsStream(C, WILD_OK) \
    FsRtlTestAnsiCharacter((C), TRUE, (WILD_OK), FSRTL_NTFS_STREAM_LEGAL)

#define FsRtlIsAnsiCharacterLegal(C, FLAGS) \
    FsRtlTestAnsiCharacter((C), TRUE, FALSE, (FLAGS))

#define FsRtlIsUnicodeCharacterWild(C) \
    ((((C) >= 0x40) ? FALSE : \
    FlagOn(LEGAL_ANSI_CHARACTER_ARRAY[(C)], FSRTL_WILD_CHARACTER)))

/* ====================================================================
 * FsRtl tunnel function declarations for tests
 * ==================================================================== */

VOID NTAPI FsRtlInitializeTunnelCache(IN PTUNNEL Cache);
VOID NTAPI FsRtlAddToTunnelCache(IN PTUNNEL Cache, IN ULONGLONG DirectoryKey,
    IN PUNICODE_STRING ShortName, IN PUNICODE_STRING LongName,
    IN BOOLEAN KeyByShortName, IN ULONG DataLength, IN PVOID Data);
BOOLEAN NTAPI FsRtlFindInTunnelCache(IN PTUNNEL Cache, IN ULONGLONG DirectoryKey,
    IN PUNICODE_STRING Name, OUT PUNICODE_STRING ShortName, OUT PUNICODE_STRING LongName,
    IN OUT PULONG DataLength, OUT PVOID Data);
VOID NTAPI FsRtlDeleteKeyFromTunnelCache(IN PTUNNEL Cache, IN ULONGLONG DirectoryKey);
VOID NTAPI FsRtlDeleteTunnelCache(IN PTUNNEL Cache);

/* ====================================================================
 * FsRtlRemoveDotsFromPath declaration
 * ==================================================================== */

NTSTATUS NTAPI FsRtlRemoveDotsFromPath(IN PWSTR OriginalString,
    IN USHORT PathLength, OUT USHORT *NewLength);

/* ====================================================================
 * FsRtl function declarations for tests
 * ==================================================================== */

VOID NTAPI FsRtlDissectDbcs(
    IN ANSI_STRING Path,
    OUT PANSI_STRING FirstName,
    OUT PANSI_STRING RemainingName);

VOID NTAPI FsRtlDissectName(
    IN UNICODE_STRING Path,
    OUT PUNICODE_STRING FirstName,
    OUT PUNICODE_STRING RemainingName);

BOOLEAN NTAPI FsRtlDoesDbcsContainWildCards(
    IN PANSI_STRING Name);

BOOLEAN NTAPI FsRtlDoesNameContainWildCards(
    IN PUNICODE_STRING Name);

BOOLEAN NTAPI FsRtlIsDbcsInExpression(
    IN PANSI_STRING Expression,
    IN PANSI_STRING Name);

BOOLEAN NTAPI FsRtlIsNameInExpression(
    IN PUNICODE_STRING Expression,
    IN PUNICODE_STRING Name,
    IN BOOLEAN IgnoreCase,
    IN PWCHAR UpcaseTable OPTIONAL);

BOOLEAN NTAPI FsRtlIsFatDbcsLegal(
    IN ANSI_STRING DbcsName,
    IN BOOLEAN WildCardsPermissible,
    IN BOOLEAN PathNamePermissible,
    IN BOOLEAN LeadingBackslashPermissible);

BOOLEAN NTAPI FsRtlIsHpfsDbcsLegal(
    IN ANSI_STRING DbcsName,
    IN BOOLEAN WildCardsPermissible,
    IN BOOLEAN PathNamePermissible,
    IN BOOLEAN LeadingBackslashPermissible);

/* ====================================================================
 * FsRtl MCB function declarations
 * ==================================================================== */

VOID NTAPI FsRtlInitializeLargeMcb(IN PLARGE_MCB Mcb, IN POOL_TYPE PoolType);
VOID NTAPI FsRtlUninitializeLargeMcb(IN PLARGE_MCB Mcb);

BOOLEAN NTAPI FsRtlAddLargeMcbEntry(IN PLARGE_MCB Mcb, IN LONGLONG Vbn, IN LONGLONG Lbn, IN LONGLONG SectorCount);
VOID NTAPI FsRtlRemoveLargeMcbEntry(IN PLARGE_MCB Mcb, IN LONGLONG Vbn, IN LONGLONG SectorCount);

BOOLEAN NTAPI FsRtlGetNextLargeMcbEntry(IN PLARGE_MCB Mcb, IN ULONG RunIndex,
    OUT PLONGLONG Vbn, OUT PLONGLONG Lbn, OUT PLONGLONG SectorCount);
BOOLEAN NTAPI FsRtlLookupLargeMcbEntry(IN PLARGE_MCB Mcb, IN LONGLONG Vbn,
    OUT PLONGLONG Lbn OPTIONAL, OUT PLONGLONG SectorCountFromLbn OPTIONAL,
    OUT PLONGLONG StartingLbn OPTIONAL, OUT PLONGLONG SectorCountFromStartingLbn OPTIONAL,
    OUT PULONG Index OPTIONAL);

BOOLEAN NTAPI FsRtlLookupLastLargeMcbEntry(IN PLARGE_MCB Mcb, OUT PLONGLONG Vbn, OUT PLONGLONG Lbn);
BOOLEAN NTAPI FsRtlLookupLastLargeMcbEntryAndIndex(IN PLARGE_MCB Mcb,
    OUT PLONGLONG LargeVbn, OUT PLONGLONG LargeLbn, OUT PULONG Index);

ULONG NTAPI FsRtlNumberOfRunsInLargeMcb(IN PLARGE_MCB Mcb);

VOID NTAPI FsRtlResetLargeMcb(IN PLARGE_MCB Mcb, IN BOOLEAN SelfSynchronized);
BOOLEAN NTAPI FsRtlSplitLargeMcb(IN PLARGE_MCB Mcb, IN LONGLONG Vbn, IN LONGLONG Amount);
VOID NTAPI FsRtlTruncateLargeMcb(IN PLARGE_MCB Mcb, IN LONGLONG Vbn);

/* String conversion stub: UNICODE_STRING -> ANSI_STRING */
static inline NTSTATUS RtlUnicodeStringToAnsiString(PANSI_STRING Dest, PCUNICODE_STRING Src, BOOLEAN Alloc) {
    USHORT wlen = Src->Length / sizeof(WCHAR);
    if (Alloc) {
        Dest->Buffer = (PCHAR)malloc(wlen + 1);
        if (!Dest->Buffer) return STATUS_NO_MEMORY;
        Dest->MaximumLength = wlen + 1;
    }
    for (USHORT i = 0; i < wlen; i++) {
        WCHAR c = Src->Buffer[i];
        Dest->Buffer[i] = (CHAR)(c > 0xFF ? '?' : (UCHAR)c);
    }
    Dest->Buffer[wlen] = 0;
    Dest->Length = wlen;
    return STATUS_SUCCESS;
}

static inline void RtlFreeAnsiString(PANSI_STRING Str) {
    if (Str->Buffer) { free(Str->Buffer); Str->Buffer = NULL; }
    Str->Length = 0;
    Str->MaximumLength = 0;
}

/* ====================================================================
 * RtlFindCharInUnicodeString — search for character in string
 * Implemented in nls_stub.c (not inline — must crash on NULL for SEH tests)
 * ==================================================================== */

#define RTL_FIND_CHAR_IN_UNICODE_STRING_START_AT_END        1
#define RTL_FIND_CHAR_IN_UNICODE_STRING_COMPLEMENT_CHAR_SET 2
#define RTL_FIND_CHAR_IN_UNICODE_STRING_CASE_INSENSITIVE    4

NTSTATUS NTAPI RtlFindCharInUnicodeString(
    ULONG Flags, PCUNICODE_STRING SearchString,
    PCUNICODE_STRING MatchString, PUSHORT Position);

/* ====================================================================
 * Safe string functions (ntstrsafe.h substitute)
 * Implemented in nls_stub.c
 * ==================================================================== */

NTSTATUS RtlStringCbPrintfA(char *pszDest, SIZE_T cbDest, const char *pszFormat, ...);

#endif
