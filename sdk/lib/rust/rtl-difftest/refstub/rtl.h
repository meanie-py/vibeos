/* Minimal <rtl.h> for isolating ReactOS sdk/lib/rtl/unicode.c on the
 * host. The genuine rtl.h pulls windef/winbase/winreg/objbase + the
 * whole ndk + pseh; none of that is needed to compile unicode.c's
 * string logic. We supply only the counted-string types + the handful
 * of macros/typedefs it textually uses. The NLS/heap link surface
 * (Nls* tables, Rtlp{Allocate,Free}StringMemory, Rtlp{Up,Down}case-
 * UnicodeChar, RtlUpperChar, RtlInt64ToUnicodeString,
 * RtlpDuplicateUnicodeString, strchrW) is stubbed in the build
 * prelude so the whole TU links even though Wave RTL-1's proof
 * functions don't call it. WCHAR is forced to `unsigned short` (host
 * wchar_t is 32-bit) for u16 parity — the cnt-difftest precedent. */
#ifndef RTL_REFSTUB_RTL_H
#define RTL_REFSTUB_RTL_H

#include <stddef.h> /* size_t */
#include <string.h> /* memcpy/memmove/memset/strlen (char) */

/* Calling convention / SAL / annotations → nothing. */
#define NTAPI
#define NTSYSAPI
#define FASTCALL
#define CDECL
#define __cdecl
#define IN
#define OUT
#define OPTIONAL
#define CONST const
#define VOID void
/* SAL — neutralize both the bare and the argument-taking forms. The
 * genuine unicode.c sprinkles these in return-type position; if any
 * is left undefined the parser mis-reads the function header. */
#define _When_(a, b)
#define _In_
#define _In_opt_
#define _Out_
#define _Out_opt_
#define _Inout_
#define _Inout_opt_
#define _In_z_
#define _Inout_z_
#define _Out_z_
#define _In_reads_(x)
#define _In_reads_bytes_(x)
#define _In_reads_z_(x)
#define _In_reads_or_z_(x)
#define _Out_writes_(x)
#define _Out_writes_z_(x)
#define _Out_writes_bytes_(x)
#define _Out_writes_to_(a, b)
#define _Out_writes_bytes_to_(a, b)
#define _Inout_updates_(x)
#define _Field_size_(x)
#define _Field_size_opt_(x)
#define _Reserved_
#define _Const_
#define _Check_return_
#define _Must_inspect_result_
#define _Success_(x)
#define _Return_type_success_(x)
#define _Ret_maybenull_
#define _Ret_range_(a, b)
#define _Post_z_
#define _Post_equal_to_(x)
#define _Pre_maybenull_
#define _Pre_notnull_
#define _At_(a, b)
#define _Analysis_assume_(x)
#define _IRQL_requires_max_(x)
#define _IRQL_requires_same_
#define _Maybe_raises_SEH_exception_
#define _Frees_ptr_opt_
#define _Out_range_(a, b)
#define _Notref_
#define __drv_aliasesMem
#define __drv_allocatesMem(x)
#define __drv_freesMem(x)
#define _Outptr_
#define _Outptr_result_maybenull_
#define _Outptr_result_bytebuffer_(x)
#define __analysis_assume(x) ((void)0)
#define PAGED_CODE()
#define PAGED_CODE_RTL()
#define C_ASSERT(x)

/* Basic NT integer types. WCHAR == unsigned short (host parity). */
typedef int BOOLEAN_INT_UNUSED_;
typedef unsigned char BOOLEAN;
typedef unsigned char UCHAR;
typedef char CHAR;
typedef short SHORT;
typedef unsigned short USHORT;
typedef int LONG;
typedef unsigned int ULONG;
typedef unsigned short WCHAR;
typedef long long LONGLONG;
typedef unsigned long long ULONGLONG;
typedef size_t SIZE_T;
typedef void *PVOID;
typedef CHAR *PCHAR, *PSTR, *LPSTR;
typedef const CHAR *PCSTR, *PCSZ, *LPCSTR;
typedef WCHAR *PWCHAR, *PWSTR, *LPWSTR;
typedef const WCHAR *PCWSTR, *LPCWSTR, *PCWCH;
typedef WCHAR *PWCH;
typedef CHAR *PCH;
typedef const CHAR *PCCH;
typedef BOOLEAN *PBOOLEAN;
typedef UCHAR *PUCHAR;
typedef USHORT *PUSHORT;
typedef SHORT *PSHORT;
typedef ULONG *PULONG;
typedef LONG *PLONG;
typedef int INT, BOOL;
typedef unsigned int UINT;
typedef unsigned char BYTE;
typedef unsigned short WORD;
typedef unsigned int DWORD;
typedef size_t ULONG_PTR, SIZE_T_DUP_;
typedef ptrdiff_t LONG_PTR;
/* NTSTATUS must be 32-bit (like the real kernel's LONG) so the
 * genuine NT_SUCCESS()/NT_ERROR() macros sign-classify 0xC000xxxx
 * error codes correctly. Host `long` is 64-bit, which would make
 * those codes *positive* and NT_SUCCESS(error) wrongly TRUE — same
 * host/target width hazard as the WCHAR→u16 forcing. */
typedef int NTSTATUS;
typedef NTSTATUS *PNTSTATUS;
#define HASH_STRING_ALGORITHM_DEFAULT 0
#define HASH_STRING_ALGORITHM_X65599 1
#define HASH_STRING_ALGORITHM_INVALID 0xffffffff
/* RtlIsTextUnicode flags (winnls) — values per Win32; only need the
 * non-RTL-1 RtlIsTextUnicode to compile. */
#define IS_TEXT_UNICODE_ASCII16 0x0001
#define IS_TEXT_UNICODE_REVERSE_ASCII16 0x0010
#define IS_TEXT_UNICODE_STATISTICS 0x0002
#define IS_TEXT_UNICODE_REVERSE_STATISTICS 0x0020
#define IS_TEXT_UNICODE_CONTROLS 0x0004
#define IS_TEXT_UNICODE_REVERSE_CONTROLS 0x0040
#define IS_TEXT_UNICODE_SIGNATURE 0x0008
#define IS_TEXT_UNICODE_REVERSE_SIGNATURE 0x0080
#define IS_TEXT_UNICODE_ILLEGAL_CHARS 0x0100
#define IS_TEXT_UNICODE_ODD_LENGTH 0x0200
#define IS_TEXT_UNICODE_DBCS_LEADBYTE 0x0400
#define IS_TEXT_UNICODE_NULL_BYTES 0x1000
#define IS_TEXT_UNICODE_UNICODE_MASK 0x000F
#define IS_TEXT_UNICODE_REVERSE_MASK 0x00F0
#define IS_TEXT_UNICODE_NOT_UNICODE_MASK 0x0F00
#define IS_TEXT_UNICODE_NOT_ASCII_MASK 0xF000
/* pool tags: the genuine values are multichar literals; the harness
 * RtlpAllocateStringMemory stub ignores the tag, so 0 is fine. */
#define TAG_USTR 0
#define TAG_ASTR 0
#define TAG_OSTR 0

typedef union _LARGE_INTEGER {
    struct {
        ULONG LowPart;
        LONG HighPart;
    };
    LONGLONG QuadPart;
} LARGE_INTEGER, *PLARGE_INTEGER;

typedef struct _STRING {
    USHORT Length;
    USHORT MaximumLength;
    PCHAR Buffer;
} STRING, *PSTRING, ANSI_STRING, *PANSI_STRING, OEM_STRING, *POEM_STRING;
typedef const STRING *PCSTRING, *PCANSI_STRING, *PCOEM_STRING;

typedef struct _UNICODE_STRING {
    USHORT Length;
    USHORT MaximumLength;
    PWSTR Buffer;
} UNICODE_STRING, *PUNICODE_STRING;
typedef const UNICODE_STRING *PCUNICODE_STRING;

/* Genuine layouts (ndk/rtltypes.h) — a non-RTL-1 fn
 * (RtlMultiAppendUnicodeStringBuffer) touches the members; it's never
 * exercised by Wave RTL-1 but must compile. */
typedef struct _RTL_BUFFER {
    PUCHAR Buffer;
    PUCHAR StaticBuffer;
    SIZE_T Size;
    SIZE_T StaticSize;
    SIZE_T ReservedForAllocatedSize;
    PVOID ReservedForIMalloc;
} RTL_BUFFER, *PRTL_BUFFER;

typedef struct _RTL_UNICODE_STRING_BUFFER {
    UNICODE_STRING String;
    RTL_BUFFER ByteBuffer;
    UCHAR MinimumStaticBufferForTerminalNul[2];
} RTL_UNICODE_STRING_BUFFER, *PRTL_UNICODE_STRING_BUFFER;

#define RTL_SKIP_BUFFER_COPY 0x00000001

#define RTL_DUPLICATE_UNICODE_STRING_NULL_TERMINATE 1
#define RTL_DUPLICATE_UNICODE_STRING_ALLOCATE_NULL_STRING 2
#define RTL_FIND_CHAR_IN_UNICODE_STRING_START_AT_END 1
#define RTL_FIND_CHAR_IN_UNICODE_STRING_COMPLEMENT_CHAR_SET 2
#define RTL_FIND_CHAR_IN_UNICODE_STRING_CASE_INSENSITIVE 4
#define MAX_COMPUTERNAME_LENGTH 15

typedef struct _GENERATE_NAME_CONTEXT {
    USHORT Checksum;
    BOOLEAN CheckSumInserted;
    UCHAR NameLength;
    WCHAR NameBuffer[8];
    ULONG ExtensionLength;
    WCHAR ExtensionBuffer[4];
    ULONG LastIndexValue;
} GENERATE_NAME_CONTEXT, *PGENERATE_NAME_CONTEXT;

typedef struct _GUID {
    ULONG Data1;
    USHORT Data2;
    USHORT Data3;
    UCHAR Data4[8];
} GUID, *PGUID, *LPGUID;
typedef const GUID *LPCGUID, *REFGUID;

#define TRUE 1
#define FALSE 0
#ifndef NULL
#define NULL ((void *)0)
#endif
#define MAXUSHORT 0xffff
#define MAXULONG 0xffffffff
#define UNICODE_NULL ((WCHAR)0)
#define ANSI_NULL ((CHAR)0)
#define LOBYTE(w) ((UCHAR)((w) & 0xff))
#define HIBYTE(w) ((UCHAR)(((w) >> 8) & 0xff))
#ifndef min
#define min(a, b) (((a) < (b)) ? (a) : (b))
#endif
#ifndef max
#define max(a, b) (((a) > (b)) ? (a) : (b))
#endif

#define NT_SUCCESS(s) (((NTSTATUS)(s)) >= 0)
#define NT_ERROR(s) ((((ULONG)(s)) >> 30) == 3)

#define STATUS_SUCCESS ((NTSTATUS)0x00000000L)
#define STATUS_BUFFER_OVERFLOW ((NTSTATUS)0x80000005L)
#define STATUS_UNMAPPABLE_CHARACTER ((NTSTATUS)0xC0000162L)
#define STATUS_NOT_IMPLEMENTED ((NTSTATUS)0xC0000002L)
#define STATUS_ACCESS_VIOLATION ((NTSTATUS)0xC0000005L)
#define STATUS_INVALID_PARAMETER ((NTSTATUS)0xC000000DL)
#define STATUS_INVALID_PARAMETER_1 ((NTSTATUS)0xC00000EFL)
#define STATUS_INVALID_PARAMETER_2 ((NTSTATUS)0xC00000F0L)
#define STATUS_NO_MEMORY ((NTSTATUS)0xC0000017L)
#define STATUS_BUFFER_TOO_SMALL ((NTSTATUS)0xC0000023L)
#define STATUS_NOT_FOUND ((NTSTATUS)0xC0000225L)
#define STATUS_NAME_TOO_LONG ((NTSTATUS)0xC0000106L)
#define STATUS_INVALID_COMPUTER_NAME ((NTSTATUS)0xC0000122L)
#define STATUS_SOME_NOT_MAPPED ((NTSTATUS)0x00000107L)
#define STATUS_INVALID_PARAMETER_4 ((NTSTATUS)0xC00000F2L)
#define STATUS_INVALID_PARAMETER_5 ((NTSTATUS)0xC00000F3L)

/* unicode.c uses these three as the byte-move primitives. */
#define RtlCopyMemory(d, s, n) memcpy((d), (s), (n))
#define RtlMoveMemory(d, s, n) memmove((d), (s), (n))
#define RtlZeroMemory(d, n) memset((d), 0, (n))
#define RtlFillMemory(d, n, v) memset((d), (v), (n))

/* host wcslen wants 32-bit wchar_t; unicode.c's WCHAR is u16 here. */
static __inline SIZE_T rtlref_wcslen(const WCHAR *s) {
    SIZE_T n = 0;
    while (s[n]) n++;
    return n;
}
#define wcslen rtlref_wcslen

/* Size macros (ndk/rtlfuncs.h) — single-byte CP form (NLS_MB_*=0 so
 * the genuine ternaries never reach the Rtlx* path). Wave RTL-3. */
#define NLS_MB_CODE_PAGE_TAG 0
#define NLS_MB_OEM_CODE_PAGE_TAG 0
#define RtlUnicodeStringToAnsiSize(S) (((S)->Length + sizeof(WCHAR)) / sizeof(WCHAR))
#define RtlAnsiStringToUnicodeSize(S) (((S)->Length + sizeof(CHAR)) * sizeof(WCHAR))
#define RtlUnicodeStringToOemSize(S) (((S)->Length + sizeof(WCHAR)) / sizeof(WCHAR))
#define RtlOemStringToUnicodeSize(S) (((S)->Length + sizeof(CHAR)) * sizeof(WCHAR))
#define RtlUnicodeStringToCountedOemSize(S) ((ULONG)(RtlUnicodeStringToOemSize(S) - sizeof(CHAR)))
#define RtlOemStringToCountedUnicodeSize(S) ((ULONG)(RtlOemStringToUnicodeSize(S) - sizeof(WCHAR)))

#endif
