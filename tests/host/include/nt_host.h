/*
 * nt_host.h — NT type definitions for host-side (macOS/Linux) compilation
 *
 * This header replaces the entire Windows/ReactOS header chain when compiling
 * NT kernel algorithms natively on the development host. It provides:
 *   - Scalar types (NTSTATUS, ULONG, BOOLEAN, PVOID, etc.)
 *   - String types (UNICODE_STRING, ANSI_STRING)
 *   - List types (LIST_ENTRY with inline macros)
 *   - Splay/AVL tree types and macros
 *   - Generic table types
 *   - Memory macros (RtlCopyMemory -> memcpy)
 *   - Pool allocation stubs (ExAllocatePool -> malloc)
 *   - PSEH replacement via setjmp/signal
 */

#ifndef _NT_HOST_H_
#define _NT_HOST_H_

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stdint.h>
#include <stddef.h>
#include <stdarg.h>
#include <limits.h>
#include <assert.h>
#include <setjmp.h>
#include <signal.h>
#include <sys/mman.h>
#include <unistd.h>

/* ====================================================================
 * Calling conventions and attributes — no-ops on non-Windows
 * ==================================================================== */

#define NTAPI
#define WINAPI
#define CDECL
#define __fastcall
#define FORCEINLINE static inline __attribute__((always_inline))
#define DECLSPEC_NORETURN __attribute__((noreturn))
#define UNREFERENCED_PARAMETER(p) ((void)(p))

/* MSVC-specific types */
#define __int64 long long

/* SAL annotations — no-ops */
#define IN
#define OUT
#define OPTIONAL
#define _In_
#define _Out_
#define _Inout_
#define _In_opt_
#define _Out_opt_
#define _Inout_opt_
#define _In_reads_(s)
#define _In_reads_bytes_(s)
#define _Out_writes_(s)
#define _Out_writes_bytes_(s)
#define _When_(c, a)
#define _Post_invalid_
#define __drv_freesMem(m)
#define __drv_allocatesMem(m)
#define _IRQL_requires_same_
#define _Function_class_(c)
#define _Analysis_assume_(x)
#define _Must_inspect_result_
#define _IRQL_requires_min_(x)
#define _IRQL_requires_max_(x)
#define _Acquires_lock_(x)
#define _Releases_lock_(x)
#define _Global_critical_region_

/* ====================================================================
 * Basic scalar types
 * ==================================================================== */

typedef int32_t  LONG, *PLONG;
typedef uint32_t ULONG, *PULONG;
typedef int16_t  SHORT, *PSHORT;
typedef uint16_t USHORT, *PUSHORT;
typedef int8_t   CHAR;
typedef uint8_t  UCHAR, *PUCHAR, BOOLEAN, *PBOOLEAN;
typedef char     *PCHAR, *PCH, *LPCH, *LPSTR, *PSTR;
typedef const char *PCSTR, *LPCSTR, *PCCH;
typedef uint16_t WCHAR, *PWCHAR, *PWSTR, *LPWSTR;
typedef const uint16_t *PCWSTR, *LPCWSTR, *PCWCH;
typedef UCHAR    CUCHAR, *PCUCHAR;
typedef void     VOID, *PVOID, *LPVOID;
typedef const void *PCVOID, *LPCVOID;
typedef int64_t  LONGLONG, *PLONGLONG;
typedef uint64_t ULONGLONG, *PULONGLONG;

typedef union _LARGE_INTEGER {
    struct { ULONG LowPart; LONG HighPart; };
    LONGLONG QuadPart;
} LARGE_INTEGER, *PLARGE_INTEGER;

typedef union _ULARGE_INTEGER {
    struct { ULONG LowPart; ULONG HighPart; };
    ULONGLONG QuadPart;
} ULARGE_INTEGER, *PULARGE_INTEGER;
typedef uintptr_t ULONG_PTR, *PULONG_PTR, SIZE_T, *PSIZE_T;
typedef intptr_t  LONG_PTR, SSIZE_T;
typedef LONG     CLONG;
typedef int32_t  NTSTATUS, *PNTSTATUS;
typedef ULONG    DWORD;
typedef int      BOOL, INT;
typedef unsigned int UINT;

#define TRUE  1
#define FALSE 0

#define ANYSIZE_ARRAY 1

#ifndef CONST
#define CONST const
#endif

#ifndef ASSERT
#define ASSERT(x) assert(x)
#endif

#ifndef NULL
#define NULL ((void *)0)
#endif

/* ====================================================================
 * NT status codes
 * ==================================================================== */

#define STATUS_SUCCESS              ((NTSTATUS)0x00000000)
#define STATUS_NOT_FOUND            ((NTSTATUS)0xC0000225)
#define STATUS_ACCESS_VIOLATION     ((NTSTATUS)0xC0000005)
#define STATUS_INVALID_PARAMETER    ((NTSTATUS)0xC000000D)
#define STATUS_BUFFER_OVERFLOW      ((NTSTATUS)0x80000005)
#define STATUS_BUFFER_TOO_SMALL     ((NTSTATUS)0xC0000023)
#define STATUS_NO_MEMORY            ((NTSTATUS)0xC0000017)
#define STATUS_UNSUCCESSFUL         ((NTSTATUS)0xC0000001)
#define STATUS_OBJECT_NAME_INVALID  ((NTSTATUS)0xC0000033)
#define STATUS_NO_MORE_ENTRIES      ((NTSTATUS)0x8000001A)
#define STATUS_RANGE_NOT_FOUND      ((NTSTATUS)0xC0000269)
#define STATUS_INSUFFICIENT_RESOURCES ((NTSTATUS)0xC000009A)
#define STATUS_IO_REPARSE_DATA_INVALID ((NTSTATUS)0xC0000278)
#define STATUS_ASSERTION_FAILURE    ((NTSTATUS)0xC0000420)
#define STATUS_TIMEOUT              ((NTSTATUS)0x00000102)
#define STATUS_STACK_OVERFLOW       ((NTSTATUS)0xC00000FD)
#define STATUS_GUARD_PAGE_VIOLATION ((NTSTATUS)0x80000001)
#define RPC_NT_UUID_LOCAL_ONLY      ((NTSTATUS)0x40020056)

#define NT_SUCCESS(Status) ((NTSTATUS)(Status) >= 0)

#define EXCEPTION_EXECUTE_HANDLER 1

/* ====================================================================
 * Utility macros
 * ==================================================================== */

#define MAXULONG    0xFFFFFFFFUL
#define MAXULONG_PTR ((ULONG_PTR)~0ULL)
#define MAXLONG     0x7FFFFFFFL
#define PAGE_SIZE   4096

#ifndef FIELD_OFFSET
#define FIELD_OFFSET(type, field) ((ULONG_PTR)&(((type *)0)->field))
#endif

#ifndef CONTAINING_RECORD
#define CONTAINING_RECORD(address, type, field) \
    ((type *)((PCHAR)(address) - FIELD_OFFSET(type, field)))
#endif

#ifndef RTL_NUMBER_OF
#define RTL_NUMBER_OF(A) (sizeof(A)/sizeof((A)[0]))
#endif

#define UNICODE_NULL ((WCHAR)0)

#define UNIMPLEMENTED do { fprintf(stderr, "UNIMPLEMENTED: %s:%d\n", __FILE__, __LINE__); abort(); } while(0)

#ifndef min
#define min(a, b) ((a) < (b) ? (a) : (b))
#endif
#ifndef max
#define max(a, b) ((a) > (b) ? (a) : (b))
#endif

/* C_ASSERT — compile-time assertion */
#define C_ASSERT(e) _Static_assert(e, #e)

#ifndef RTL_SIZEOF_THROUGH_FIELD
#define RTL_SIZEOF_THROUGH_FIELD(type, field) \
    (FIELD_OFFSET(type, field) + sizeof(((type *)0)->field))
#endif

/* ====================================================================
 * String types
 *
 * Note: Compile with -fshort-wchar so L"..." produces uint16_t arrays
 * matching NT's 16-bit WCHAR.
 * ==================================================================== */

typedef struct _UNICODE_STRING {
    USHORT Length;
    USHORT MaximumLength;
    PWSTR  Buffer;
} UNICODE_STRING, *PUNICODE_STRING;
typedef const UNICODE_STRING *PCUNICODE_STRING;

typedef struct _STRING {
    USHORT Length;
    USHORT MaximumLength;
    PCHAR  Buffer;
} STRING, ANSI_STRING, *PANSI_STRING, *PSTRING;
typedef const STRING *PCSTRING;
typedef const ANSI_STRING *PCANSI_STRING;

#define RTL_CONSTANT_STRING(s) { sizeof(s) - sizeof((s)[0]), sizeof(s), (void*)(s) }

static inline void RtlInitUnicodeString(PUNICODE_STRING Dest, PCWSTR Src) {
    if (Src) {
        USHORT len = 0;
        while (Src[len]) len++;
        Dest->Length = len * sizeof(WCHAR);
        Dest->MaximumLength = (len + 1) * sizeof(WCHAR);
        Dest->Buffer = (PWSTR)Src;
    } else {
        Dest->Length = 0;
        Dest->MaximumLength = 0;
        Dest->Buffer = NULL;
    }
}

static inline void RtlInitEmptyUnicodeString(PUNICODE_STRING Dest, PWSTR Buf, USHORT MaxLen) {
    Dest->Length = 0;
    Dest->MaximumLength = MaxLen;
    Dest->Buffer = Buf;
}

static inline void RtlFreeUnicodeString(PUNICODE_STRING Str) {
    if (Str->Buffer) { free(Str->Buffer); Str->Buffer = NULL; }
    Str->Length = 0;
    Str->MaximumLength = 0;
}

static inline void RtlInitAnsiString(PANSI_STRING Dest, PCSTR Src) {
    if (Src) {
        USHORT len = (USHORT)strlen(Src);
        Dest->Length = len;
        Dest->MaximumLength = len + 1;
        Dest->Buffer = (PCHAR)Src;
    } else {
        Dest->Length = 0;
        Dest->MaximumLength = 0;
        Dest->Buffer = NULL;
    }
}

static inline NTSTATUS RtlAnsiStringToUnicodeString(PUNICODE_STRING Dest, PCANSI_STRING Src, BOOLEAN Alloc) {
    USHORT len = Src->Length;
    USHORT byte_len = len * sizeof(WCHAR);
    if (Alloc) {
        Dest->Buffer = (PWSTR)malloc(byte_len + sizeof(WCHAR));
        if (!Dest->Buffer) return STATUS_NO_MEMORY;
        Dest->MaximumLength = byte_len + sizeof(WCHAR);
    }
    for (USHORT i = 0; i < len; i++) {
        Dest->Buffer[i] = (WCHAR)(unsigned char)Src->Buffer[i];
    }
    Dest->Buffer[len] = 0;
    Dest->Length = byte_len;
    return STATUS_SUCCESS;
}

/* RtlUpcaseUnicodeString — simple ASCII-range uppercasing */
static inline NTSTATUS RtlUpcaseUnicodeString(PUNICODE_STRING Dest, PCUNICODE_STRING Src, BOOLEAN Alloc) {
    USHORT len = Src->Length / sizeof(WCHAR);
    if (Alloc) {
        if (len == 0) {
            Dest->Buffer = (PWSTR)malloc(sizeof(WCHAR));
            if (!Dest->Buffer) return STATUS_NO_MEMORY;
            Dest->MaximumLength = 0;
            Dest->Length = 0;
            return STATUS_SUCCESS;
        }
        Dest->Buffer = (PWSTR)malloc(len * sizeof(WCHAR));
        if (!Dest->Buffer) return STATUS_NO_MEMORY;
        Dest->MaximumLength = len * sizeof(WCHAR);
    }
    for (USHORT i = 0; i < len; i++) {
        WCHAR c = Src->Buffer[i];
        if (c >= 'a' && c <= 'z') c -= 32;
        Dest->Buffer[i] = c;
    }
    Dest->Length = Src->Length;
    return STATUS_SUCCESS;
}

/* ====================================================================
 * LIST_ENTRY — doubly-linked list (NT kernel fundamental)
 * ==================================================================== */

typedef struct _LIST_ENTRY {
    struct _LIST_ENTRY *Flink;
    struct _LIST_ENTRY *Blink;
} LIST_ENTRY, *PLIST_ENTRY;

static inline void InitializeListHead(PLIST_ENTRY ListHead) {
    ListHead->Flink = ListHead->Blink = ListHead;
}

static inline BOOLEAN IsListEmpty(PLIST_ENTRY ListHead) {
    return (BOOLEAN)(ListHead->Flink == ListHead);
}

static inline void InsertTailList(PLIST_ENTRY ListHead, PLIST_ENTRY Entry) {
    PLIST_ENTRY Blink = ListHead->Blink;
    Entry->Flink = ListHead;
    Entry->Blink = Blink;
    Blink->Flink = Entry;
    ListHead->Blink = Entry;
}

static inline void InsertHeadList(PLIST_ENTRY ListHead, PLIST_ENTRY Entry) {
    PLIST_ENTRY Flink = ListHead->Flink;
    Entry->Flink = Flink;
    Entry->Blink = ListHead;
    Flink->Blink = Entry;
    ListHead->Flink = Entry;
}

static inline BOOLEAN RemoveEntryList(PLIST_ENTRY Entry) {
    PLIST_ENTRY Flink = Entry->Flink;
    PLIST_ENTRY Blink = Entry->Blink;
    Blink->Flink = Flink;
    Flink->Blink = Blink;
    return (BOOLEAN)(Flink == Blink);
}

static inline PLIST_ENTRY RemoveHeadList(PLIST_ENTRY ListHead) {
    PLIST_ENTRY Entry = ListHead->Flink;
    PLIST_ENTRY Flink = Entry->Flink;
    ListHead->Flink = Flink;
    Flink->Blink = ListHead;
    return Entry;
}

/* ====================================================================
 * SINGLE_LIST_ENTRY — singly-linked list (NT kernel fundamental)
 * ==================================================================== */

typedef struct _SINGLE_LIST_ENTRY {
    struct _SINGLE_LIST_ENTRY *Next;
} SINGLE_LIST_ENTRY, *PSINGLE_LIST_ENTRY;

static inline void PushEntryList(PSINGLE_LIST_ENTRY ListHead, PSINGLE_LIST_ENTRY Entry) {
    Entry->Next = ListHead->Next;
    ListHead->Next = Entry;
}

static inline PSINGLE_LIST_ENTRY PopEntryList(PSINGLE_LIST_ENTRY ListHead) {
    PSINGLE_LIST_ENTRY Entry = ListHead->Next;
    if (Entry) ListHead->Next = Entry->Next;
    return Entry;
}

/* ====================================================================
 * KSPIN_LOCK — spinlock (no-op on single-threaded host)
 * ==================================================================== */

typedef ULONG_PTR KSPIN_LOCK, *PKSPIN_LOCK;

static inline void KeInitializeSpinLock(PKSPIN_LOCK SpinLock) {
    *SpinLock = 0;
}

/* ====================================================================
 * ExInterlocked* list operations — non-concurrent host stubs
 *
 * Defined as macros so tests can #undef them to test the function exports
 * (provided in nls_stub.c). The _impl helpers do the actual work.
 * ==================================================================== */

static inline PLIST_ENTRY _ExInterlockedInsertHeadList_impl(
    PLIST_ENTRY ListHead, PLIST_ENTRY Entry, PKSPIN_LOCK Lock)
{
    (void)Lock;
    PLIST_ENTRY OldFirst = IsListEmpty(ListHead) ? NULL : ListHead->Flink;
    InsertHeadList(ListHead, Entry);
    return OldFirst;
}

static inline PLIST_ENTRY _ExInterlockedInsertTailList_impl(
    PLIST_ENTRY ListHead, PLIST_ENTRY Entry, PKSPIN_LOCK Lock)
{
    (void)Lock;
    PLIST_ENTRY OldLast = IsListEmpty(ListHead) ? NULL : ListHead->Blink;
    InsertTailList(ListHead, Entry);
    return OldLast;
}

static inline PLIST_ENTRY _ExInterlockedRemoveHeadList_impl(
    PLIST_ENTRY ListHead, PKSPIN_LOCK Lock)
{
    (void)Lock;
    if (IsListEmpty(ListHead)) return NULL;
    return RemoveHeadList(ListHead);
}

static inline PSINGLE_LIST_ENTRY _ExInterlockedPushEntryList_impl(
    PSINGLE_LIST_ENTRY ListHead, PSINGLE_LIST_ENTRY Entry, PKSPIN_LOCK Lock)
{
    (void)Lock;
    PSINGLE_LIST_ENTRY OldFirst = ListHead->Next;
    PushEntryList(ListHead, Entry);
    return OldFirst;
}

static inline PSINGLE_LIST_ENTRY _ExInterlockedPopEntryList_impl(
    PSINGLE_LIST_ENTRY ListHead, PKSPIN_LOCK Lock)
{
    (void)Lock;
    return PopEntryList(ListHead);
}

/* Extern function declarations — placed BEFORE macros so they parse without
   expansion. After test #undef's the macros, these declarations are still
   in scope and the linker resolves them from nls_stub.c. */
extern PLIST_ENTRY ExInterlockedInsertHeadList(PLIST_ENTRY, PLIST_ENTRY, PKSPIN_LOCK);
extern PLIST_ENTRY ExInterlockedInsertTailList(PLIST_ENTRY, PLIST_ENTRY, PKSPIN_LOCK);
extern PLIST_ENTRY ExInterlockedRemoveHeadList(PLIST_ENTRY, PKSPIN_LOCK);
extern PSINGLE_LIST_ENTRY ExInterlockedPushEntryList(PSINGLE_LIST_ENTRY, PSINGLE_LIST_ENTRY, PKSPIN_LOCK);
extern PSINGLE_LIST_ENTRY ExInterlockedPopEntryList(PSINGLE_LIST_ENTRY, PKSPIN_LOCK);

/* Macros: when active, use inline impls. After #undef, calls fall through
   to the extern function declarations above (resolved by nls_stub.c). */
#define ExInterlockedInsertHeadList(h,e,l) _ExInterlockedInsertHeadList_impl(h,e,l)
#define ExInterlockedInsertTailList(h,e,l) _ExInterlockedInsertTailList_impl(h,e,l)
#define ExInterlockedRemoveHeadList(h,l)   _ExInterlockedRemoveHeadList_impl(h,l)
#define ExInterlockedPushEntryList(h,e,l)  _ExInterlockedPushEntryList_impl(h,e,l)
#define ExInterlockedPopEntryList(h,l)     _ExInterlockedPopEntryList_impl(h,l)

/* ====================================================================
 * RTL_SPLAY_LINKS — splay tree node
 * ==================================================================== */

typedef struct _RTL_SPLAY_LINKS {
    struct _RTL_SPLAY_LINKS *Parent;
    struct _RTL_SPLAY_LINKS *LeftChild;
    struct _RTL_SPLAY_LINKS *RightChild;
} RTL_SPLAY_LINKS, *PRTL_SPLAY_LINKS;

#define RtlParent(Links)     ((PRTL_SPLAY_LINKS)(Links))->Parent
#define RtlLeftChild(Links)  ((PRTL_SPLAY_LINKS)(Links))->LeftChild
#define RtlRightChild(Links) ((PRTL_SPLAY_LINKS)(Links))->RightChild

#define RtlIsRoot(Links)       (RtlParent(Links) == (PRTL_SPLAY_LINKS)(Links))
#define RtlIsLeftChild(Links)  (RtlLeftChild(RtlParent(Links)) == (PRTL_SPLAY_LINKS)(Links))
#define RtlIsRightChild(Links) (RtlRightChild(RtlParent(Links)) == (PRTL_SPLAY_LINKS)(Links))

#define RtlInitializeSplayLinks(Links) {           \
    PRTL_SPLAY_LINKS _SplayLinks;                  \
    _SplayLinks = (PRTL_SPLAY_LINKS)(Links);       \
    _SplayLinks->Parent = _SplayLinks;              \
    _SplayLinks->LeftChild = NULL;                  \
    _SplayLinks->RightChild = NULL;                 \
}

#define RtlInsertAsLeftChild(ParentLinks, ChildLinks) { \
    PRTL_SPLAY_LINKS _SplayParent;                      \
    PRTL_SPLAY_LINKS _SplayChild;                       \
    _SplayParent = (PRTL_SPLAY_LINKS)(ParentLinks);     \
    _SplayChild = (PRTL_SPLAY_LINKS)(ChildLinks);       \
    _SplayParent->LeftChild = _SplayChild;               \
    _SplayChild->Parent = _SplayParent;                  \
}

#define RtlInsertAsRightChild(ParentLinks, ChildLinks) { \
    PRTL_SPLAY_LINKS _SplayParent;                       \
    PRTL_SPLAY_LINKS _SplayChild;                        \
    _SplayParent = (PRTL_SPLAY_LINKS)(ParentLinks);      \
    _SplayChild = (PRTL_SPLAY_LINKS)(ChildLinks);        \
    _SplayParent->RightChild = _SplayChild;               \
    _SplayChild->Parent = _SplayParent;                   \
}

/* Splay tree function declarations */
PRTL_SPLAY_LINKS NTAPI RtlSplay(PRTL_SPLAY_LINKS Links);
PRTL_SPLAY_LINKS NTAPI RtlDelete(PRTL_SPLAY_LINKS Links);
PRTL_SPLAY_LINKS NTAPI RtlRealSuccessor(PRTL_SPLAY_LINKS Links);
PRTL_SPLAY_LINKS NTAPI RtlRealPredecessor(PRTL_SPLAY_LINKS Links);
PRTL_SPLAY_LINKS NTAPI RtlSubtreeSuccessor(IN PRTL_SPLAY_LINKS Links);
PRTL_SPLAY_LINKS NTAPI RtlSubtreePredecessor(IN PRTL_SPLAY_LINKS Links);
VOID NTAPI RtlDeleteNoSplay(PRTL_SPLAY_LINKS Links, PRTL_SPLAY_LINKS *Root);

/* ====================================================================
 * RTL_BALANCED_LINKS — AVL tree node
 * ==================================================================== */

typedef struct _RTL_BALANCED_LINKS {
    struct _RTL_BALANCED_LINKS *Parent;
    struct _RTL_BALANCED_LINKS *LeftChild;
    struct _RTL_BALANCED_LINKS *RightChild;
    CHAR Balance;
    UCHAR Reserved[3];
} RTL_BALANCED_LINKS, *PRTL_BALANCED_LINKS;

/* ====================================================================
 * Generic table types (splay-based and AVL-based)
 * ==================================================================== */

typedef enum _TABLE_SEARCH_RESULT {
    TableEmptyTree,
    TableFoundNode,
    TableInsertAsLeft,
    TableInsertAsRight
} TABLE_SEARCH_RESULT;

typedef enum _RTL_GENERIC_COMPARE_RESULTS {
    GenericLessThan,
    GenericGreaterThan,
    GenericEqual
} RTL_GENERIC_COMPARE_RESULTS;

struct _RTL_GENERIC_TABLE;
struct _RTL_AVL_TABLE;

/* Splay-based generic table */
typedef RTL_GENERIC_COMPARE_RESULTS (NTAPI RTL_GENERIC_COMPARE_ROUTINE)(
    struct _RTL_GENERIC_TABLE *Table, PVOID FirstStruct, PVOID SecondStruct);
typedef RTL_GENERIC_COMPARE_ROUTINE *PRTL_GENERIC_COMPARE_ROUTINE;

typedef PVOID (NTAPI RTL_GENERIC_ALLOCATE_ROUTINE)(
    struct _RTL_GENERIC_TABLE *Table, CLONG ByteSize);
typedef RTL_GENERIC_ALLOCATE_ROUTINE *PRTL_GENERIC_ALLOCATE_ROUTINE;

typedef VOID (NTAPI RTL_GENERIC_FREE_ROUTINE)(
    struct _RTL_GENERIC_TABLE *Table, PVOID Buffer);
typedef RTL_GENERIC_FREE_ROUTINE *PRTL_GENERIC_FREE_ROUTINE;

typedef struct _RTL_GENERIC_TABLE {
    PRTL_SPLAY_LINKS TableRoot;
    LIST_ENTRY InsertOrderList;
    PLIST_ENTRY OrderedPointer;
    ULONG WhichOrderedElement;
    ULONG NumberGenericTableElements;
    PRTL_GENERIC_COMPARE_ROUTINE CompareRoutine;
    PRTL_GENERIC_ALLOCATE_ROUTINE AllocateRoutine;
    PRTL_GENERIC_FREE_ROUTINE FreeRoutine;
    PVOID TableContext;
} RTL_GENERIC_TABLE, *PRTL_GENERIC_TABLE;

/* AVL-based generic table */
typedef RTL_GENERIC_COMPARE_RESULTS (NTAPI RTL_AVL_COMPARE_ROUTINE)(
    struct _RTL_AVL_TABLE *Table, PVOID FirstStruct, PVOID SecondStruct);
typedef RTL_AVL_COMPARE_ROUTINE *PRTL_AVL_COMPARE_ROUTINE;

typedef PVOID (NTAPI RTL_AVL_ALLOCATE_ROUTINE)(
    struct _RTL_AVL_TABLE *Table, CLONG ByteSize);
typedef RTL_AVL_ALLOCATE_ROUTINE *PRTL_AVL_ALLOCATE_ROUTINE;

typedef VOID (NTAPI RTL_AVL_FREE_ROUTINE)(
    struct _RTL_AVL_TABLE *Table, PVOID Buffer);
typedef RTL_AVL_FREE_ROUTINE *PRTL_AVL_FREE_ROUTINE;

typedef NTSTATUS (NTAPI RTL_AVL_MATCH_FUNCTION)(
    struct _RTL_AVL_TABLE *Table, PVOID UserData, PVOID MatchData);
typedef RTL_AVL_MATCH_FUNCTION *PRTL_AVL_MATCH_FUNCTION;

typedef struct _RTL_AVL_TABLE {
    RTL_BALANCED_LINKS BalancedRoot;
    PVOID OrderedPointer;
    ULONG WhichOrderedElement;
    ULONG NumberGenericTableElements;
    ULONG DepthOfTree;
    PRTL_BALANCED_LINKS RestartKey;
    ULONG DeleteCount;
    PRTL_AVL_COMPARE_ROUTINE CompareRoutine;
    PRTL_AVL_ALLOCATE_ROUTINE AllocateRoutine;
    PRTL_AVL_FREE_ROUTINE FreeRoutine;
    PVOID TableContext;
} RTL_AVL_TABLE, *PRTL_AVL_TABLE;

/* Splay-based generic table functions */
VOID NTAPI RtlInitializeGenericTable(IN PRTL_GENERIC_TABLE Table,
    IN PRTL_GENERIC_COMPARE_ROUTINE CompareRoutine,
    IN PRTL_GENERIC_ALLOCATE_ROUTINE AllocateRoutine,
    IN PRTL_GENERIC_FREE_ROUTINE FreeRoutine,
    IN PVOID TableContext);
PVOID NTAPI RtlInsertElementGenericTable(IN PRTL_GENERIC_TABLE Table,
    IN PVOID Buffer, IN ULONG BufferSize, OUT PBOOLEAN NewElement OPTIONAL);
PVOID NTAPI RtlInsertElementGenericTableFull(IN PRTL_GENERIC_TABLE Table,
    IN PVOID Buffer, IN ULONG BufferSize, OUT PBOOLEAN NewElement OPTIONAL,
    IN PVOID NodeOrParent, IN TABLE_SEARCH_RESULT SearchResult);
BOOLEAN NTAPI RtlIsGenericTableEmpty(IN PRTL_GENERIC_TABLE Table);
ULONG NTAPI RtlNumberGenericTableElements(IN PRTL_GENERIC_TABLE Table);
PVOID NTAPI RtlLookupElementGenericTable(IN PRTL_GENERIC_TABLE Table, IN PVOID Buffer);
PVOID NTAPI RtlLookupElementGenericTableFull(IN PRTL_GENERIC_TABLE Table,
    IN PVOID Buffer, OUT PVOID *NodeOrParent, OUT TABLE_SEARCH_RESULT *SearchResult);
BOOLEAN NTAPI RtlDeleteElementGenericTable(IN PRTL_GENERIC_TABLE Table, IN PVOID Buffer);
PVOID NTAPI RtlEnumerateGenericTable(IN PRTL_GENERIC_TABLE Table, IN BOOLEAN Restart);
PVOID NTAPI RtlEnumerateGenericTableWithoutSplaying(IN PRTL_GENERIC_TABLE Table, IN OUT PVOID *RestartKey);
PVOID NTAPI RtlGetElementGenericTable(IN PRTL_GENERIC_TABLE Table, IN ULONG I);

/* AVL-based generic table functions */
VOID NTAPI RtlInitializeGenericTableAvl(IN OUT PRTL_AVL_TABLE Table,
    IN PRTL_AVL_COMPARE_ROUTINE CompareRoutine,
    IN PRTL_AVL_ALLOCATE_ROUTINE AllocateRoutine,
    IN PRTL_AVL_FREE_ROUTINE FreeRoutine,
    IN PVOID TableContext);
PVOID NTAPI RtlInsertElementGenericTableAvl(IN PRTL_AVL_TABLE Table,
    IN PVOID Buffer, IN ULONG BufferSize, OUT PBOOLEAN NewElement OPTIONAL);
BOOLEAN NTAPI RtlIsGenericTableEmptyAvl(IN PRTL_AVL_TABLE Table);
ULONG NTAPI RtlNumberGenericTableElementsAvl(IN PRTL_AVL_TABLE Table);
PVOID NTAPI RtlLookupElementGenericTableAvl(IN PRTL_AVL_TABLE Table, IN PVOID Buffer);
BOOLEAN NTAPI RtlDeleteElementGenericTableAvl(IN PRTL_AVL_TABLE Table, IN PVOID Buffer);
PVOID NTAPI RtlEnumerateGenericTableAvl(IN PRTL_AVL_TABLE Table, IN BOOLEAN Restart);
PVOID NTAPI RtlEnumerateGenericTableWithoutSplayingAvl(IN PRTL_AVL_TABLE Table, IN OUT PVOID *RestartKey);
PVOID NTAPI RtlGetElementGenericTableAvl(IN PRTL_AVL_TABLE Table, IN ULONG I);
PVOID NTAPI RtlEnumerateGenericTableLikeADirectory(IN PRTL_AVL_TABLE Table,
    IN PRTL_AVL_MATCH_FUNCTION MatchFunction, IN PVOID MatchData,
    IN ULONG NextFlag, IN OUT PVOID *RestartKey, IN OUT PULONG DeleteCount,
    IN OUT PVOID Buffer);

/* ====================================================================
 * RTL_RANGE_LIST — range list data structures
 * ==================================================================== */

typedef struct _RTL_RANGE_LIST {
    LIST_ENTRY ListHead;
    ULONG Flags;
    ULONG Count;
    ULONG Stamp;
} RTL_RANGE_LIST, *PRTL_RANGE_LIST;

typedef struct _RTL_RANGE {
    ULONGLONG Start;
    ULONGLONG End;
    PVOID UserData;
    PVOID Owner;
    UCHAR Attributes;
    UCHAR Flags;
} RTL_RANGE, *PRTL_RANGE;

typedef struct _RANGE_LIST_ITERATOR {
    PLIST_ENTRY RangeListHead;
    PLIST_ENTRY MergedHead;
    PVOID Current;
    ULONG Stamp;
} RTL_RANGE_LIST_ITERATOR, *PRTL_RANGE_LIST_ITERATOR;

#define RTL_RANGE_LIST_ADD_IF_CONFLICT  0x00000001
#define RTL_RANGE_LIST_ADD_SHARED       0x00000002

#define RTL_RANGE_SHARED                0x01

typedef BOOLEAN (NTAPI *PRTL_CONFLICT_RANGE_CALLBACK)(PVOID Context, struct _RTL_RANGE *Range);

/* Range list function declarations */
VOID NTAPI RtlInitializeRangeList(PRTL_RANGE_LIST RangeList);
NTSTATUS NTAPI RtlAddRange(PRTL_RANGE_LIST RangeList, ULONGLONG Start, ULONGLONG End,
    UCHAR Attributes, ULONG Flags, PVOID UserData, PVOID Owner);
NTSTATUS NTAPI RtlDeleteRange(PRTL_RANGE_LIST RangeList, ULONGLONG Start, ULONGLONG End, PVOID Owner);
VOID NTAPI RtlFreeRangeList(PRTL_RANGE_LIST RangeList);
NTSTATUS NTAPI RtlGetFirstRange(PRTL_RANGE_LIST RangeList, PRTL_RANGE_LIST_ITERATOR Iterator, PRTL_RANGE *Range);
NTSTATUS NTAPI RtlGetNextRange(PRTL_RANGE_LIST_ITERATOR Iterator, PRTL_RANGE *Range, BOOLEAN MoveForwards);
NTSTATUS NTAPI RtlIsRangeAvailable(PRTL_RANGE_LIST RangeList, ULONGLONG Start, ULONGLONG End,
    ULONG Flags, UCHAR AttributeAvailableMask, PVOID Context, PRTL_CONFLICT_RANGE_CALLBACK Callback,
    PBOOLEAN Available);

/* ====================================================================
 * Memory functions
 * ==================================================================== */

/* Use memmove, not memcpy: Windows kernel memcpy is a simple forward copy
   that handles overlapping regions when dest < src. macOS memcpy may use
   SIMD/backwards copy that breaks this. The RtlMemory test relies on this. */
#define RtlCopyMemory(d,s,n)    memmove(d,s,n)
#define RtlMoveMemory(d,s,n)    memmove(d,s,n)
#define RtlFillMemory(d,n,v)    memset(d,v,n)
#define RtlZeroMemory(d,n)      memset(d,0,n)
#define RtlEqualMemory(a,b,n)   (!memcmp(a,b,n))
#define RtlSecureZeroMemory(d,n) memset(d,0,n)
#define RtlCopyBytes             RtlCopyMemory
#define RtlZeroBytes             RtlZeroMemory
#define RtlFillBytes             RtlFillMemory
#define RtlCopyMemoryNonTemporal RtlCopyMemory
#define RtlPrefetchMemoryNonTemporal(s,l) ((void)0)

static inline SIZE_T RtlCompareMemory(const void *Source1, const void *Source2, SIZE_T Length) {
    const unsigned char *s1 = (const unsigned char *)Source1;
    const unsigned char *s2 = (const unsigned char *)Source2;
    SIZE_T i;
    for (i = 0; i < Length; i++) {
        if (s1[i] != s2[i]) break;
    }
    return i;
}

static inline void RtlFillMemoryUlong(PVOID Dest, SIZE_T Length, ULONG Pattern) {
    PULONG d = (PULONG)Dest;
    SIZE_T count = Length / sizeof(ULONG);
    for (SIZE_T i = 0; i < count; i++) d[i] = Pattern;
}

static inline SIZE_T RtlCompareMemoryUlong(PVOID Source, SIZE_T Length, ULONG Pattern) {
    SIZE_T count = Length / sizeof(ULONG);
    PULONG s = (PULONG)Source;
    for (SIZE_T i = 0; i < count; i++) {
        if (s[i] != Pattern) return i * sizeof(ULONG);
    }
    return count * sizeof(ULONG);
}

/* ====================================================================
 * Pool allocation stubs
 * ==================================================================== */

typedef enum _POOL_TYPE {
    NonPagedPool,
    PagedPool,
    NonPagedPoolMustSucceed,
    DontUseThisType,
    NonPagedPoolCacheAligned,
    PagedPoolCacheAligned
} POOL_TYPE;

#define POOL_RAISE_IF_ALLOCATION_FAILURE 16

#define ExAllocatePool(type, size)               malloc(size)
#define ExAllocatePoolWithTag(type, size, tag)    malloc(size)
#define ExFreePool(p)                            free(p)
#define ExFreePoolWithTag(p, tag)                free(p)

/* ====================================================================
 * Lookaside lists — simple malloc/free wrappers on host
 * ==================================================================== */

typedef struct _PAGED_LOOKASIDE_LIST {
    SIZE_T Size;
} PAGED_LOOKASIDE_LIST, *PPAGED_LOOKASIDE_LIST;

typedef struct _NPAGED_LOOKASIDE_LIST {
    SIZE_T Size;
} NPAGED_LOOKASIDE_LIST, *PNPAGED_LOOKASIDE_LIST;

typedef PVOID (NTAPI *PALLOCATE_FUNCTION)(POOL_TYPE PoolType, SIZE_T NumberOfBytes, ULONG Tag);
typedef VOID (NTAPI *PFREE_FUNCTION)(PVOID Buffer);

static inline void ExInitializePagedLookasideList(
    PPAGED_LOOKASIDE_LIST Lookaside,
    PALLOCATE_FUNCTION Allocate,
    PFREE_FUNCTION Free,
    ULONG Flags,
    SIZE_T Size,
    ULONG Tag,
    USHORT Depth)
{
    (void)Allocate; (void)Free; (void)Flags; (void)Tag; (void)Depth;
    Lookaside->Size = Size;
}

static inline PVOID ExAllocateFromPagedLookasideList(PPAGED_LOOKASIDE_LIST Lookaside) {
    return malloc(Lookaside->Size);
}

static inline void ExFreeToPagedLookasideList(PPAGED_LOOKASIDE_LIST Lookaside, PVOID Entry) {
    (void)Lookaside;
    free(Entry);
}

static inline void ExInitializeNPagedLookasideList(
    PNPAGED_LOOKASIDE_LIST Lookaside,
    PALLOCATE_FUNCTION Allocate,
    PFREE_FUNCTION Free,
    ULONG Flags,
    SIZE_T Size,
    ULONG Tag,
    USHORT Depth)
{
    (void)Allocate; (void)Free; (void)Flags; (void)Tag; (void)Depth;
    Lookaside->Size = Size;
}

static inline PVOID ExAllocateFromNPagedLookasideList(PNPAGED_LOOKASIDE_LIST Lookaside) {
    return malloc(Lookaside->Size);
}

static inline void ExFreeToNPagedLookasideList(PNPAGED_LOOKASIDE_LIST Lookaside, PVOID Entry) {
    (void)Lookaside;
    free(Entry);
}

/* ====================================================================
 * KGUARDED_MUTEX — no-op on single-threaded host
 * ==================================================================== */

typedef struct _KGUARDED_MUTEX {
    LONG Count;
} KGUARDED_MUTEX, *PKGUARDED_MUTEX;

static inline void KeInitializeGuardedMutex(PKGUARDED_MUTEX Mutex) {
    Mutex->Count = 0;
}

static inline void KeAcquireGuardedMutex(PKGUARDED_MUTEX Mutex) {
    (void)Mutex;
}

static inline void KeReleaseGuardedMutex(PKGUARDED_MUTEX Mutex) {
    (void)Mutex;
}

/* ====================================================================
 * HANDLE type
 * ==================================================================== */

typedef PVOID HANDLE, *PHANDLE;

/* ====================================================================
 * FAST_MUTEX — no-op on single-threaded host
 * ==================================================================== */

typedef struct _FAST_MUTEX {
    volatile LONG Count;
} FAST_MUTEX, *PFAST_MUTEX;

static inline void ExInitializeFastMutex(PFAST_MUTEX Mutex) {
    Mutex->Count = 1;
}

static inline void ExAcquireFastMutex(PFAST_MUTEX Mutex) {
    (void)Mutex;
}

static inline void ExReleaseFastMutex(PFAST_MUTEX Mutex) {
    (void)Mutex;
}

/* ====================================================================
 * TUNNEL — tunnel cache for filesystem drivers
 * ==================================================================== */

typedef struct _TUNNEL {
    FAST_MUTEX Mutex;
    PRTL_SPLAY_LINKS Cache;
    LIST_ENTRY TimerQueue;
    USHORT NumEntries;
} TUNNEL, *PTUNNEL;

/* ====================================================================
 * OBJECT_ATTRIBUTES — for ZwOpenKey stub
 * ==================================================================== */

#define OBJ_CASE_INSENSITIVE 0x00000040
#define OBJ_KERNEL_HANDLE    0x00000200
#define KEY_READ 0x20019

typedef struct _OBJECT_ATTRIBUTES {
    ULONG Length;
    HANDLE RootDirectory;
    PUNICODE_STRING ObjectName;
    ULONG Attributes;
    PVOID SecurityDescriptor;
    PVOID SecurityQualityOfService;
} OBJECT_ATTRIBUTES, *POBJECT_ATTRIBUTES;

#define InitializeObjectAttributes(p, n, a, r, s) do { \
    (p)->Length = sizeof(OBJECT_ATTRIBUTES); \
    (p)->RootDirectory = (r); \
    (p)->Attributes = (a); \
    (p)->ObjectName = (n); \
    (p)->SecurityDescriptor = (s); \
    (p)->SecurityQualityOfService = NULL; \
} while(0)

/* ====================================================================
 * Registry stubs — always fail (no registry on host)
 * ==================================================================== */

typedef enum _KEY_VALUE_INFORMATION_CLASS {
    KeyValueBasicInformation,
    KeyValueFullInformation,
    KeyValuePartialInformation
} KEY_VALUE_INFORMATION_CLASS;

typedef struct _KEY_VALUE_FULL_INFORMATION {
    ULONG TitleIndex;
    ULONG Type;
    ULONG DataOffset;
    ULONG DataLength;
    ULONG NameLength;
    WCHAR Name[1];
} KEY_VALUE_FULL_INFORMATION, *PKEY_VALUE_FULL_INFORMATION;

static inline NTSTATUS ZwOpenKey(PHANDLE KeyHandle, ULONG DesiredAccess, POBJECT_ATTRIBUTES ObjectAttributes) {
    (void)KeyHandle; (void)DesiredAccess; (void)ObjectAttributes;
    return STATUS_UNSUCCESSFUL;
}

static inline NTSTATUS ZwQueryValueKey(HANDLE KeyHandle, PUNICODE_STRING ValueName,
    KEY_VALUE_INFORMATION_CLASS InfoClass, PVOID Info, ULONG Length, PULONG ResultLength) {
    (void)KeyHandle; (void)ValueName; (void)InfoClass; (void)Info; (void)Length; (void)ResultLength;
    return STATUS_UNSUCCESSFUL;
}

static inline NTSTATUS ZwClose(HANDLE Handle) {
    (void)Handle;
    return STATUS_SUCCESS;
}

/* ====================================================================
 * Kernel query stubs
 * ==================================================================== */

static inline BOOLEAN MmIsThisAnNtAsSystem(void) { return FALSE; }

static inline void KeQuerySystemTime(PLARGE_INTEGER CurrentTime) {
    /* Incrementing counter — sufficient for tunnel cache age logic */
    static LONGLONG _kqst_counter = 1000000000LL;
    CurrentTime->QuadPart = _kqst_counter++;
}

/* ====================================================================
 * POOL_COLD_ALLOCATION flag
 * ==================================================================== */

#define POOL_COLD_ALLOCATION 256

/* ====================================================================
 * Unicode string utility functions
 * ==================================================================== */

static inline LONG RtlCompareUnicodeString(PCUNICODE_STRING String1, PCUNICODE_STRING String2, BOOLEAN CaseInsensitive) {
    USHORT len1 = String1->Length / sizeof(WCHAR);
    USHORT len2 = String2->Length / sizeof(WCHAR);
    USHORT minlen = len1 < len2 ? len1 : len2;
    for (USHORT i = 0; i < minlen; i++) {
        WCHAR c1 = String1->Buffer[i];
        WCHAR c2 = String2->Buffer[i];
        if (CaseInsensitive) {
            if (c1 >= 'a' && c1 <= 'z') c1 -= 32;
            if (c2 >= 'a' && c2 <= 'z') c2 -= 32;
        }
        if (c1 != c2) return (LONG)c1 - (LONG)c2;
    }
    return (LONG)len1 - (LONG)len2;
}

static inline void RtlCopyUnicodeString(PUNICODE_STRING Dest, PCUNICODE_STRING Src) {
    USHORT len = Src->Length;
    if (len > Dest->MaximumLength) len = Dest->MaximumLength;
    if (len) RtlCopyMemory(Dest->Buffer, Src->Buffer, len);
    Dest->Length = len;
    if (len < Dest->MaximumLength) {
        Dest->Buffer[len / sizeof(WCHAR)] = UNICODE_NULL;
    }
}

static inline NTSTATUS RtlAppendUnicodeToString(PUNICODE_STRING Dest, PCWSTR Src) {
    if (!Src) return STATUS_SUCCESS;
    USHORT srclen = 0;
    while (Src[srclen]) srclen++;
    USHORT srcbytes = srclen * sizeof(WCHAR);
    if (Dest->Length + srcbytes > Dest->MaximumLength) return STATUS_BUFFER_TOO_SMALL;
    RtlCopyMemory((PCHAR)Dest->Buffer + Dest->Length, Src, srcbytes);
    Dest->Length += srcbytes;
    if (Dest->Length < Dest->MaximumLength) {
        Dest->Buffer[Dest->Length / sizeof(WCHAR)] = UNICODE_NULL;
    }
    return STATUS_SUCCESS;
}

/* ====================================================================
 * 16-bit wcscpy/wcslen — macOS wcscpy uses 32-bit wchar_t, incompatible
 * with -fshort-wchar. Provide our own 16-bit versions.
 * ==================================================================== */

static inline WCHAR *_nt_wcscpy(WCHAR *dest, const WCHAR *src) {
    WCHAR *d = dest;
    while ((*d++ = *src++) != 0);
    return dest;
}

static inline int _nt_wcscmp(const WCHAR *s1, const WCHAR *s2) {
    while (*s1 && *s1 == *s2) { s1++; s2++; }
    return (int)*s1 - (int)*s2;
}

#define wcscpy _nt_wcscpy
#define wcscmp _nt_wcscmp

/* ====================================================================
 * MCB types — Mapped Control Block for filesystem drivers
 * ==================================================================== */

typedef struct _BASE_MCB {
    ULONG MaximumPairCount;
    ULONG PairCount;
    USHORT PoolType;
    USHORT Flags;
    PVOID Mapping;
} BASE_MCB, *PBASE_MCB;

typedef struct _LARGE_MCB {
    PKGUARDED_MUTEX GuardedMutex;
    BASE_MCB BaseMcb;
} LARGE_MCB, *PLARGE_MCB;

/* IRQL — tracked state on host (functions in nls_stub.c) */
typedef unsigned char KIRQL, *PKIRQL;
#define LOW_LEVEL       0
#define PASSIVE_LEVEL   0
#define APC_LEVEL       1
#define DISPATCH_LEVEL  2
#define CMCI_LEVEL      5
#define PROFILE_LEVEL   27
#define CLOCK1_LEVEL    28
#define CLOCK2_LEVEL    28
#define CLOCK_LEVEL     CLOCK2_LEVEL
#define IPI_LEVEL       29
#define POWER_LEVEL     30
#define HIGH_LEVEL      31

extern void KeRaiseIrql(KIRQL NewIrql, PKIRQL OldIrql);
extern void KeLowerIrql(KIRQL NewIrql);
extern KIRQL KeGetCurrentIrql(void);
extern KIRQL KeRaiseIrqlToDpcLevel(void);
extern KIRQL KeRaiseIrqlToSynchLevel(void);

/* KmtIsMultiProcessorBuild — FALSE on host */
extern BOOLEAN KmtIsMultiProcessorBuild;

/* ====================================================================
 * Dispatcher object types — common to events, timers, mutants
 * ==================================================================== */

typedef short CSHORT, *PCSHORT;
typedef LONG KPRIORITY;

typedef struct _DISPATCHER_HEADER {
    union {
        struct {
            UCHAR Type;
            union { UCHAR Abandoned; UCHAR Absolute; };
            union { UCHAR Size; UCHAR Hand; };
            union { UCHAR Inserted; UCHAR DpcActive; };
        };
        volatile LONG Lock;
    };
    LONG SignalState;
    LIST_ENTRY WaitListHead;
} DISPATCHER_HEADER;

/* Wait block — used in dispatcher wait lists */
typedef struct _KWAIT_BLOCK {
    LIST_ENTRY WaitListEntry;
    struct _KTHREAD *Thread;
    PVOID Object;
    struct _KWAIT_BLOCK *NextWaitBlock;
    USHORT WaitKey;
    UCHAR WaitType;
    volatile UCHAR BlockState;
} KWAIT_BLOCK, *PKWAIT_BLOCK;

/* ====================================================================
 * KTHREAD — thread structure for APC, mutex, event, and timer testing
 * ==================================================================== */

typedef struct _KTHREAD {
    SHORT KernelApcDisable;
    SHORT SpecialApcDisable;
    LIST_ENTRY MutantListHead;
    BOOLEAN WaitNext;
    KIRQL WaitIrql;
    PVOID Win32Thread;
    KWAIT_BLOCK WaitBlock[4];
} KTHREAD, *PKTHREAD;

extern KTHREAD _host_current_thread;
#define KeGetCurrentThread() ((PKTHREAD)&_host_current_thread)
extern void _host_init_thread(void);

/* APC functions */
extern BOOLEAN NTAPI KeAreApcsDisabled(VOID);
extern void NTAPI KeEnterCriticalRegion(VOID);
extern void NTAPI KeLeaveCriticalRegion(VOID);

/* Guarded region functions */
extern BOOLEAN NTAPI KeAreAllApcsDisabled(VOID);
extern void NTAPI KeEnterGuardedRegion(VOID);
extern void NTAPI KeLeaveGuardedRegion(VOID);

/* KmtGetSystemRoutineAddress — dynamic function lookup on host */
extern PVOID KmtGetSystemRoutineAddress(PCWSTR Name);

/* RtlUpcaseUnicodeChar / RtlIsValidOemCharacter */
extern WCHAR NTAPI RtlUpcaseUnicodeChar(WCHAR Source);
extern BOOLEAN NTAPI RtlIsValidOemCharacter(PWCHAR Char);

/* ====================================================================
 * KEVENT — kernel event object
 * ==================================================================== */

typedef enum _EVENT_TYPE {
    NotificationEvent = 0,
    SynchronizationEvent = 1
} EVENT_TYPE;

typedef struct _KEVENT {
    DISPATCHER_HEADER Header;
} KEVENT, *PKEVENT, *PRKEVENT;

extern VOID NTAPI KeInitializeEvent(PKEVENT Event, EVENT_TYPE Type, BOOLEAN State);
extern LONG NTAPI KeSetEvent(PKEVENT Event, KPRIORITY Increment, BOOLEAN Wait);
extern LONG NTAPI KeResetEvent(PKEVENT Event);
extern VOID NTAPI KeClearEvent(PKEVENT Event);
extern LONG NTAPI KePulseEvent(PKEVENT Event, KPRIORITY Increment, BOOLEAN Wait);
extern LONG NTAPI KeReadStateEvent(PKEVENT Event);

/* ====================================================================
 * KTIMER — kernel timer object
 * ==================================================================== */

typedef enum _TIMER_TYPE {
    NotificationTimer = 0,
    SynchronizationTimer = 1
} TIMER_TYPE;

#define TimerNotificationObject     8
#define TimerSynchronizationObject  9

struct _KDPC;

typedef struct _KTIMER {
    DISPATCHER_HEADER Header;
    ULARGE_INTEGER DueTime;
    LIST_ENTRY TimerListEntry;
    struct _KDPC *Dpc;
    ULONG Processor;
    ULONG Period;
} KTIMER, *PKTIMER;

extern VOID NTAPI KeInitializeTimerEx(PKTIMER Timer, TIMER_TYPE Type);
extern LONG NTAPI KeReadStateTimer(PKTIMER Timer);

/* ====================================================================
 * KMUTANT / KMUTEX — kernel mutex/mutant objects
 * ==================================================================== */

#define MutantObject 2

typedef struct _KMUTANT {
    DISPATCHER_HEADER Header;
    LIST_ENTRY MutantListEntry;
    struct _KTHREAD *OwnerThread;
    BOOLEAN Abandoned;
    UCHAR ApcDisable;
} KMUTANT, *PKMUTANT, *PRKMUTANT;

typedef KMUTANT KMUTEX, *PKMUTEX, *PRKMUTEX;

extern VOID NTAPI KeInitializeMutant(PKMUTANT Mutant, BOOLEAN InitialOwner);
extern LONG NTAPI KeReleaseMutant(PKMUTANT Mutant, KPRIORITY Increment,
                                   BOOLEAN Abandoned, BOOLEAN Wait);
extern VOID NTAPI KeInitializeMutex(PKMUTEX Mutex, ULONG Level);
extern LONG NTAPI KeReleaseMutex(PKMUTEX Mutex, BOOLEAN Wait);

/* ====================================================================
 * KeWaitForSingleObject — uncontended fast path for mutant/mutex
 * ==================================================================== */

typedef enum _KWAIT_REASON { Executive = 0 } KWAIT_REASON;
#define KernelMode 0
#define UserMode   1
#define STATUS_MUTANT_LIMIT_EXCEEDED ((NTSTATUS)0xC0000191)
#define STATUS_MUTANT_NOT_OWNED      ((NTSTATUS)0xC0000046)

extern NTSTATUS NTAPI KeWaitForSingleObject(
    PVOID Object, KWAIT_REASON WaitReason, LONG WaitMode,
    BOOLEAN Alertable, PLARGE_INTEGER Timeout);

/* ====================================================================
 * SLIST_HEADER — sequenced singly-linked list (host LP64 layout)
 * ==================================================================== */

typedef SINGLE_LIST_ENTRY SLIST_ENTRY, *PSLIST_ENTRY;

typedef union _SLIST_HEADER {
    struct {
        SINGLE_LIST_ENTRY Next;
        USHORT Depth;
        USHORT Sequence;
    };
} SLIST_HEADER, *PSLIST_HEADER;

static inline void InitializeSListHead(PSLIST_HEADER ListHead) {
    memset(ListHead, 0, sizeof(SLIST_HEADER));
}
#define ExInitializeSListHead InitializeSListHead

static inline USHORT ExQueryDepthSList(PSLIST_HEADER ListHead) {
    return ListHead->Depth;
}

extern PSLIST_ENTRY ExInterlockedPushEntrySList(
    PSLIST_HEADER ListHead, PSLIST_ENTRY ListEntry, PKSPIN_LOCK Lock);
extern PSLIST_ENTRY ExInterlockedPopEntrySList(
    PSLIST_HEADER ListHead, PKSPIN_LOCK Lock);
extern PSLIST_ENTRY ExInterlockedFlushSList(PSLIST_HEADER ListHead);

/* _WIN32_WINNT version constants */
#define _WIN32_WINNT_NT4    0x0400
#define _WIN32_WINNT_WIN2K  0x0500
#define _WIN32_WINNT_WINXP  0x0501
#define _WIN32_WINNT_WS03   0x0502
#define _WIN32_WINNT_VISTA  0x0600
#define _WIN32_WINNT_WIN7   0x0601
#define _WIN32_WINNT_WIN8   0x0602

/* ====================================================================
 * KDEVICE_QUEUE — device queue for sorted IRP processing
 * ==================================================================== */

typedef struct _KDEVICE_QUEUE_ENTRY {
    LIST_ENTRY DeviceListEntry;
    ULONG SortKey;
    BOOLEAN Inserted;
} KDEVICE_QUEUE_ENTRY, *PKDEVICE_QUEUE_ENTRY;

typedef struct _KDEVICE_QUEUE {
    CSHORT Type;
    CSHORT Size;
    LIST_ENTRY DeviceListHead;
    KSPIN_LOCK Lock;
    BOOLEAN Busy;
} KDEVICE_QUEUE, *PKDEVICE_QUEUE;

/* DeviceQueueObject — KOBJECTS enum value */
#define DeviceQueueObject 20

/* NT_ASSERT — maps to ASSERT on host */
#define NT_ASSERT(x) ASSERT(x)

/* ASSERT_DEVICE_QUEUE — validates device queue type tag */
#define ASSERT_DEVICE_QUEUE(Object) \
    NT_ASSERT((Object)->Type == DeviceQueueObject)

/* KLOCK_QUEUE_HANDLE — queued spinlock handle (no-op on host) */
typedef struct _KSPIN_LOCK_QUEUE {
    struct _KSPIN_LOCK_QUEUE *volatile Next;
    PKSPIN_LOCK volatile Lock;
} KSPIN_LOCK_QUEUE, *PKSPIN_LOCK_QUEUE;

typedef struct _KLOCK_QUEUE_HANDLE {
    KSPIN_LOCK_QUEUE LockQueue;
    KIRQL OldIrql;
} KLOCK_QUEUE_HANDLE, *PKLOCK_QUEUE_HANDLE;

/* KiAcquireDeviceQueueLock / KiReleaseDeviceQueueLock — no-op on host */
static inline void KiAcquireDeviceQueueLock(PKDEVICE_QUEUE DeviceQueue, PKLOCK_QUEUE_HANDLE DeviceLock) {
    (void)DeviceQueue; (void)DeviceLock;
}
static inline void KiReleaseDeviceQueueLock(PKLOCK_QUEUE_HANDLE DeviceLock) {
    (void)DeviceLock;
}

/* Device queue function declarations */
VOID NTAPI KeInitializeDeviceQueue(PKDEVICE_QUEUE DeviceQueue);
BOOLEAN NTAPI KeInsertDeviceQueue(PKDEVICE_QUEUE DeviceQueue, PKDEVICE_QUEUE_ENTRY DeviceQueueEntry);
BOOLEAN NTAPI KeInsertByKeyDeviceQueue(PKDEVICE_QUEUE DeviceQueue, PKDEVICE_QUEUE_ENTRY DeviceQueueEntry, ULONG SortKey);
PKDEVICE_QUEUE_ENTRY NTAPI KeRemoveDeviceQueue(PKDEVICE_QUEUE DeviceQueue);
PKDEVICE_QUEUE_ENTRY NTAPI KeRemoveByKeyDeviceQueue(PKDEVICE_QUEUE DeviceQueue, ULONG SortKey);
BOOLEAN NTAPI KeRemoveEntryDeviceQueue(PKDEVICE_QUEUE DeviceQueue, PKDEVICE_QUEUE_ENTRY DeviceQueueEntry);

/* ExRaiseStatus — software exception via longjmp (implemented in seh_host.c) */
void _ExRaiseStatus(NTSTATUS Status);
#define ExRaiseStatus(s) _ExRaiseStatus(s)
#define RtlRaiseStatus(s) _ExRaiseStatus(s)

/* ====================================================================
 * Interlocked operations — using GCC/Clang __sync builtins
 *
 * Defined as macros (not static inline) so they override any prior
 * extern declarations in test source files (e.g., ExInterlocked.c
 * declares them as __declspec(dllimport) before including our headers).
 * ==================================================================== */

#define MAXLONG  0x7fffffff
#define MINLONG  ((LONG)0x80000000)

/* --- Interlocked macros wrapping __sync builtins --- */

#define InterlockedCompareExchange(dest, exch, comp) \
    ((LONG)__sync_val_compare_and_swap((volatile LONG *)(dest), (LONG)(comp), (LONG)(exch)))
#define InterlockedCompareExchangeAcquire(dest, exch, comp) \
    InterlockedCompareExchange(dest, exch, comp)
#define InterlockedCompareExchangeRelease(dest, exch, comp) \
    InterlockedCompareExchange(dest, exch, comp)
#define InterlockedCompareExchangePointer(dest, exch, comp) \
    ((PVOID)__sync_val_compare_and_swap((void *volatile *)(dest), (void *)(comp), (void *)(exch)))

#define InterlockedExchange(dest, val) \
    ((LONG)__sync_lock_test_and_set((volatile LONG *)(dest), (LONG)(val)))
#define InterlockedExchangePointer(dest, val) \
    ((PVOID)__sync_lock_test_and_set((void *volatile *)(dest), (void *)(val)))

#define InterlockedExchangeAdd(dest, val) \
    ((LONG)__sync_fetch_and_add((volatile LONG *)(dest), (LONG)(val)))

#define InterlockedIncrement(p) \
    ((LONG)__sync_add_and_fetch((volatile LONG *)(p), 1))
#define InterlockedDecrement(p) \
    ((LONG)__sync_sub_and_fetch((volatile LONG *)(p), 1))
#define InterlockedIncrementAcquire(p) InterlockedIncrement(p)
#define InterlockedIncrementRelease(p) InterlockedIncrement(p)
#define InterlockedDecrementAcquire(p) InterlockedDecrement(p)
#define InterlockedDecrementRelease(p) InterlockedDecrement(p)

#define InterlockedAnd(dest, val) \
    ((LONG)__sync_fetch_and_and((volatile LONG *)(dest), (LONG)(val)))
#define InterlockedOr(dest, val) \
    ((LONG)__sync_fetch_and_or((volatile LONG *)(dest), (LONG)(val)))
#define InterlockedXor(dest, val) \
    ((LONG)__sync_fetch_and_xor((volatile LONG *)(dest), (LONG)(val)))

#define InterlockedAdd64(dest, val) \
    ((LONGLONG)__sync_add_and_fetch((volatile LONGLONG *)(dest), (LONGLONG)(val)))
#define InterlockedXor64(dest, val) \
    ((LONGLONG)__sync_fetch_and_xor((volatile LONGLONG *)(dest), (LONGLONG)(val)))

/* --- Ex* Interlocked with spinlock (spinlock ignored on single-threaded host) --- */

static inline LONGLONG _nt_ExInterlockedCompareExchange64(volatile LONGLONG *dest,
    LONGLONG *exch, LONGLONG *comp, PKSPIN_LOCK lock) {
    (void)lock;
    return __sync_val_compare_and_swap(dest, *comp, *exch);
}
#define ExInterlockedCompareExchange64(dest, exch, comp, lock) \
    _nt_ExInterlockedCompareExchange64((volatile LONGLONG *)(dest), (LONGLONG *)(exch), (LONGLONG *)(comp), (PKSPIN_LOCK)(lock))

static inline ULONG _nt_ExInterlockedExchangeUlong(ULONG *dest, ULONG val, PKSPIN_LOCK lock) {
    (void)lock;
    return __sync_lock_test_and_set(dest, val);
}
#define ExInterlockedExchangeUlong(dest, val, lock) \
    _nt_ExInterlockedExchangeUlong((ULONG *)(dest), (ULONG)(val), (PKSPIN_LOCK)(lock))

/* ExInterlockedAddUlong: provided as function (not macro) because
 * ExInterlocked.c does #undef ExInterlockedAddUlong then calls it.
 * On ARM64 there's no conflicting extern declaration (#ifdef _X86_). */
static inline ULONG ExInterlockedAddUlong(ULONG *dest, ULONG val, PKSPIN_LOCK lock) {
    (void)lock;
    return __sync_fetch_and_add(dest, val);
}

static inline LARGE_INTEGER _nt_ExInterlockedAddLargeInteger(volatile LARGE_INTEGER *addend,
    LARGE_INTEGER increment, PKSPIN_LOCK lock) {
    (void)lock;
    LARGE_INTEGER ret;
    ret.QuadPart = __sync_fetch_and_add(&addend->QuadPart, increment.QuadPart);
    return ret;
}
#define ExInterlockedAddLargeInteger(addend, incr, lock) \
    _nt_ExInterlockedAddLargeInteger((volatile LARGE_INTEGER *)(addend), (incr), (PKSPIN_LOCK)(lock))

#define ExInterlockedAddLargeStatistic(Addend, Increment) \
    (VOID)InterlockedAdd64(&(Addend)->QuadPart, (LONGLONG)(Increment))

/* --- INTERLOCKED_RESULT enum (for ExInterlockedIncrement/DecrementLong) --- */
#define RESULT_ZERO     0
#define RESULT_NEGATIVE 1
#define RESULT_POSITIVE 2
typedef enum _INTERLOCKED_RESULT {
    ResultNegative = RESULT_NEGATIVE,
    ResultZero = RESULT_ZERO,
    ResultPositive = RESULT_POSITIVE
} INTERLOCKED_RESULT;

/* ====================================================================
 * PSEH replacement — setjmp/signal-based exception handling
 *
 * Supports nested TRY/EXCEPT/FINALLY via per-thread jmp_buf pointer.
 * Each _SEH2_TRY pushes a local jmp_buf; _SEH2_END pops it.
 * Catches SIGSEGV/SIGBUS (→ STATUS_ACCESS_VIOLATION) and
 * ExRaiseStatus (→ any status code).
 * ==================================================================== */

extern __thread jmp_buf *_seh_current_jmpbuf;
extern __thread volatile NTSTATUS _seh_code;

void _seh_signal_handler(int sig);
void _seh_install(void);
void _seh_uninstall(void);

#define _SEH2_TRY \
    do { \
        jmp_buf _local_jmpbuf; \
        jmp_buf *_saved_jmpbuf = _seh_current_jmpbuf; \
        volatile int _local_caught = 0; \
        volatile NTSTATUS _local_code = STATUS_SUCCESS; \
        volatile int _local_is_finally = 0; \
        _seh_current_jmpbuf = &_local_jmpbuf; \
        _seh_install(); \
        if (setjmp(_local_jmpbuf) == 0) {

#define _SEH2_EXCEPT(filter) \
        } else { \
            _local_caught = 1; \
            _local_code = _seh_code; \
        } \
        _seh_uninstall(); \
        _seh_current_jmpbuf = _saved_jmpbuf; \
        if (_local_caught) {

#define _SEH2_FINALLY \
        } else { \
            _local_caught = 1; \
            _local_code = _seh_code; \
        } \
        _seh_uninstall(); \
        _seh_current_jmpbuf = _saved_jmpbuf; \
        _local_is_finally = 1; \
        {

#define _SEH2_END \
        } \
        if (_local_is_finally && _local_caught) { \
            _seh_code = _local_code; \
            if (_seh_current_jmpbuf) { \
                longjmp(*_seh_current_jmpbuf, 1); \
            } \
            fprintf(stderr, "Unhandled exception: 0x%08x\n", (unsigned)_local_code); \
            abort(); \
        } \
    } while(0)

#define _SEH2_GetExceptionCode() _local_code
#define _SEH2_YIELD(x) x
#define _SEH2_VOLATILE volatile

/* ====================================================================
 * UUID / GUID type
 * ==================================================================== */

typedef struct _GUID {
    ULONG Data1;
    USHORT Data2;
    USHORT Data3;
    UCHAR Data4[8];
} GUID, *PGUID, UUID, *PUUID;

/* ExUuidCreate — generate a v1-format UUID with DCE variant */
static inline NTSTATUS ExUuidCreate(UUID *Uuid) {
    static ULONGLONG _uuid_counter = 0x0001020304050607ULL;
    ULONGLONG ts = ++_uuid_counter;
    Uuid->Data1 = (ULONG)(ts & 0xFFFFFFFF);
    Uuid->Data2 = (USHORT)((ts >> 32) & 0xFFFF);
    Uuid->Data3 = (USHORT)(0x1000 | ((ts >> 48) & 0x0FFF)); /* version 1 */
    Uuid->Data4[0] = (UCHAR)(0x80 | (ts & 0x3F));           /* variant DCE */
    for (int i = 1; i < 8; i++) Uuid->Data4[i] = (UCHAR)(ts >> (i * 5));
    return STATUS_SUCCESS;
}

/* ====================================================================
 * alloca compatibility
 * ==================================================================== */

#include <alloca.h>
#ifndef _alloca
#define _alloca alloca
#endif

/* ====================================================================
 * Miscellaneous kernel stubs
 * ==================================================================== */

#define KmtIsCheckedBuild FALSE
static inline BOOLEAN KmtAreInterruptsEnabled(void) { return TRUE; }
static inline void KmtSetIrql(KIRQL NewIrql) { KeLowerIrql(NewIrql); }

/* SharedUserData stub for GetNTVersion() macro */
typedef struct _KUSER_SHARED_DATA {
    ULONG NtMajorVersion;
    ULONG NtMinorVersion;
} KUSER_SHARED_DATA;

static KUSER_SHARED_DATA _SharedUserData = { 5, 2 }; /* NT 5.2 = WS2003 */
#define SharedUserData (&_SharedUserData)

#endif /* _NT_HOST_H_ */
