/*
 * ntoskrnl.h — Host shim replacing the kernel master header
 *
 * Used when compiling kernel-mode source files (ntoskrnl/fsrtl)
 * for host-side testing.
 */
#ifndef _NTOSKRNL_HOST_H_
#define _NTOSKRNL_HOST_H_

#include "nt_host.h"

/* PAGED_CODE — IRQL assertion, no-op on host */
#define PAGED_CODE() ((void)0)

/* CODE_SEG — section placement, no-op on host */
#define CODE_SEG(s)

/* DBCS lead byte detection — always FALSE on host (ASCII only) */
#define FsRtlIsLeadDbcsCharacter(c) (FALSE)

/* NLS globals — not using DBCS on host */
extern BOOLEAN NlsMbOemCodePageTag;

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

/* signed char type used by FsRtlTestAnsiCharacter */
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

/* Legal ANSI character array — 128 entries indexed by character code.
 * Kernel source uses single-pointer direct access. */
extern const UCHAR * const _FsRtlLegalCharData;
#define LEGAL_ANSI_CHARACTER_ARRAY _FsRtlLegalCharData
#define FsRtlLegalAnsiCharacterArray _FsRtlLegalCharData

/* Character test macro */
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
 * Pool tags and constants for FsRtl
 * ==================================================================== */

#define IFS_POOL_TAG 'SF2R'
#define MAXIMUM_PAIR_COUNT 15

/* ====================================================================
 * FsRtl MCB function declarations
 * ==================================================================== */

VOID NTAPI FsRtlInitializeLargeMcbs(VOID);

VOID NTAPI FsRtlInitializeBaseMcb(IN PBASE_MCB Mcb, IN POOL_TYPE PoolType);
VOID NTAPI FsRtlInitializeLargeMcb(IN PLARGE_MCB Mcb, IN POOL_TYPE PoolType);

VOID NTAPI FsRtlUninitializeBaseMcb(IN PBASE_MCB Mcb);
VOID NTAPI FsRtlUninitializeLargeMcb(IN PLARGE_MCB Mcb);

BOOLEAN NTAPI FsRtlAddBaseMcbEntry(IN PBASE_MCB Mcb, IN LONGLONG Vbn, IN LONGLONG Lbn, IN LONGLONG SectorCount);
BOOLEAN NTAPI FsRtlAddLargeMcbEntry(IN PLARGE_MCB Mcb, IN LONGLONG Vbn, IN LONGLONG Lbn, IN LONGLONG SectorCount);

BOOLEAN NTAPI FsRtlRemoveBaseMcbEntry(IN PBASE_MCB Mcb, IN LONGLONG Vbn, IN LONGLONG SectorCount);
VOID NTAPI FsRtlRemoveLargeMcbEntry(IN PLARGE_MCB Mcb, IN LONGLONG Vbn, IN LONGLONG SectorCount);

BOOLEAN NTAPI FsRtlGetNextBaseMcbEntry(IN PBASE_MCB Mcb, IN ULONG RunIndex,
    OUT PLONGLONG Vbn, OUT PLONGLONG Lbn, OUT PLONGLONG SectorCount);
BOOLEAN NTAPI FsRtlGetNextLargeMcbEntry(IN PLARGE_MCB Mcb, IN ULONG RunIndex,
    OUT PLONGLONG Vbn, OUT PLONGLONG Lbn, OUT PLONGLONG SectorCount);

BOOLEAN NTAPI FsRtlLookupBaseMcbEntry(IN PBASE_MCB Mcb, IN LONGLONG Vbn,
    OUT PLONGLONG Lbn OPTIONAL, OUT PLONGLONG SectorCountFromLbn OPTIONAL,
    OUT PLONGLONG StartingLbn OPTIONAL, OUT PLONGLONG SectorCountFromStartingLbn OPTIONAL,
    OUT PULONG Index OPTIONAL);
BOOLEAN NTAPI FsRtlLookupLargeMcbEntry(IN PLARGE_MCB Mcb, IN LONGLONG Vbn,
    OUT PLONGLONG Lbn OPTIONAL, OUT PLONGLONG SectorCountFromLbn OPTIONAL,
    OUT PLONGLONG StartingLbn OPTIONAL, OUT PLONGLONG SectorCountFromStartingLbn OPTIONAL,
    OUT PULONG Index OPTIONAL);

BOOLEAN NTAPI FsRtlLookupLastBaseMcbEntry(IN PBASE_MCB Mcb, OUT PLONGLONG Vbn, OUT PLONGLONG Lbn);
BOOLEAN NTAPI FsRtlLookupLastLargeMcbEntry(IN PLARGE_MCB Mcb, OUT PLONGLONG Vbn, OUT PLONGLONG Lbn);

BOOLEAN NTAPI FsRtlLookupLastBaseMcbEntryAndIndex(IN PBASE_MCB Mcb,
    IN OUT PLONGLONG LargeVbn, IN OUT PLONGLONG LargeLbn, IN OUT PULONG Index);
BOOLEAN NTAPI FsRtlLookupLastLargeMcbEntryAndIndex(IN PLARGE_MCB Mcb,
    OUT PLONGLONG LargeVbn, OUT PLONGLONG LargeLbn, OUT PULONG Index);

ULONG NTAPI FsRtlNumberOfRunsInBaseMcb(IN PBASE_MCB Mcb);
ULONG NTAPI FsRtlNumberOfRunsInLargeMcb(IN PLARGE_MCB Mcb);

VOID NTAPI FsRtlResetBaseMcb(IN PBASE_MCB Mcb);
VOID NTAPI FsRtlResetLargeMcb(IN PLARGE_MCB Mcb, IN BOOLEAN SelfSynchronized);

BOOLEAN NTAPI FsRtlSplitBaseMcb(IN PBASE_MCB Mcb, IN LONGLONG Vbn, IN LONGLONG Amount);
BOOLEAN NTAPI FsRtlSplitLargeMcb(IN PLARGE_MCB Mcb, IN LONGLONG Vbn, IN LONGLONG Amount);

VOID NTAPI FsRtlTruncateBaseMcb(IN PBASE_MCB Mcb, IN LONGLONG Vbn);
VOID NTAPI FsRtlTruncateLargeMcb(IN PLARGE_MCB Mcb, IN LONGLONG Vbn);

/* ====================================================================
 * FsRtl tunnel function declarations
 * ==================================================================== */

VOID NTAPI FsRtlInitializeTunnels(VOID);
VOID NTAPI FsRtlInitializeTunnelCache(IN PTUNNEL Cache);
VOID NTAPI FsRtlAddToTunnelCache(IN PTUNNEL Cache, IN ULONGLONG DirectoryKey,
    IN PUNICODE_STRING ShortName, IN PUNICODE_STRING LongName,
    IN BOOLEAN KeyByShortName, IN ULONG DataLength, IN PVOID Data);
BOOLEAN NTAPI FsRtlFindInTunnelCache(IN PTUNNEL Cache, IN ULONGLONG DirectoryKey,
    IN PUNICODE_STRING Name, OUT PUNICODE_STRING ShortName, OUT PUNICODE_STRING LongName,
    IN OUT PULONG DataLength, OUT PVOID Data);
VOID NTAPI FsRtlDeleteKeyFromTunnelCache(IN PTUNNEL Cache, IN ULONGLONG DirectoryKey);
VOID NTAPI FsRtlDeleteTunnelCache(IN PTUNNEL Cache);

/* NTKERNELAPI — empty on host */
#define NTKERNELAPI

/* ====================================================================
 * FsRtl name/expression function declarations
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

BOOLEAN NTAPI FsRtlAreNamesEqual(
    IN PCUNICODE_STRING Name1,
    IN PCUNICODE_STRING Name2,
    IN BOOLEAN IgnoreCase,
    IN PCWCH UpcaseTable OPTIONAL);

#endif
