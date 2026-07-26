/*
 * The CRT and Win32 TYPES the OS-coupled msvcrt reference TUs need (file.c
 * first; dir.c / console.c / process.c ride the same substrate).
 *
 * Split from the constants deliberately.  Constants are DERIVED — every value
 * in msvcrt_crt_consts.h is computed by the pinned cross-compiler against
 * msvcrt's real headers, because a wrong constant is silent.  Layouts cannot
 * be derived the same way (a struct definition is not an integer), so they are
 * transcribed here from msvcrt's own headers and then PROVED: the generator
 * harvests every field offset and struct size from the cross-compiler and emits
 * msvcrt_crt_layout.h, a wall of _Static_assert the host compiler checks
 * against these definitions.  A transcription slip is a build error, not a
 * silently misgraded row.
 *
 * Sources, all under dll/win32/msvcrt/include (msvcrt substitutes its own CRT
 * headers for sdk/include/crt — see the generator docstring):
 *   sys/stat.h, sys/utime.h, corecrt.h
 * and the PSDK for the Win32 half.
 *
 * Do not edit a layout here without re-running the generator.
 */
#ifndef MSVCRT_DIFFTEST_FILE_TYPES_H
#define MSVCRT_DIFFTEST_FILE_TYPES_H

/* ── CRT scalar types (dll/win32/msvcrt/include/sys/stat.h, corecrt.h) ──
 * `_dev_t` is gated on _CRTDLL — `unsigned short` with it, `unsigned int`
 * without.  msvcrt does not define it, so `unsigned int` is correct; the
 * layout guardian is what proves that rather than this comment. */
typedef unsigned int   _dev_t;
typedef unsigned short _ino_t;
typedef int            _off_t;
typedef __msvcrt_long  __time32_t;
typedef __int64        __time64_t;

/* Windows base type refstub/msvcrt.h does not carry (nothing needed it until
 * file.c's `__pioinfo` bookkeeping). */
typedef ULONG_PTR DWORD_PTR;

/* glibc's <sys/stat.h> defines st_atime/st_mtime/st_ctime as MACROS
 * (`#define st_atime st_atim.tv_sec`, its POSIX-2008 compatibility shim).  A
 * struct declared before that header is pulled keeps honest field names, but
 * every LATER use of `s.st_atime` in file.c silently expands to
 * `s.st_atim.tv_sec` — a member our structs do not have.  So pull the header
 * now, on purpose, and undo the three macros while nothing depends on them.
 * Pulling it here rather than relying on include order is what makes this
 * robust: it does not matter who else reaches for <sys/stat.h> afterwards. */
#include <sys/stat.h>
#undef st_atime
#undef st_mtime
#undef st_ctime

/* These structs are under `#include <pshpack8.h>` in the real header. */
#pragma pack(push, 8)

struct _stat32 {
  _dev_t st_dev; _ino_t st_ino;
  unsigned short st_mode; short st_nlink, st_uid, st_gid;
  _dev_t st_rdev; _off_t st_size;
  __time32_t st_atime, st_mtime, st_ctime;
};

struct _stat32i64 {
  _dev_t st_dev; _ino_t st_ino;
  unsigned short st_mode; short st_nlink, st_uid, st_gid;
  _dev_t st_rdev; __int64 __attribute__((aligned(8))) st_size;
  __time32_t st_atime, st_mtime, st_ctime;
};

struct _stat64i32 {
  _dev_t st_dev; _ino_t st_ino;
  unsigned short st_mode; short st_nlink, st_uid, st_gid;
  _dev_t st_rdev; _off_t st_size;
  __time64_t st_atime, st_mtime, st_ctime;
};

struct _stat64 {
  _dev_t st_dev; _ino_t st_ino;
  unsigned short st_mode; short st_nlink, st_uid, st_gid;
  _dev_t st_rdev; __int64 __attribute__((aligned(8))) st_size;
  __time64_t st_atime, st_mtime, st_ctime;
};

/* `_stat` / `_stati64` use `time_t`, which is `__time64_t` unless
 * _USE_32BIT_TIME_T — msvcrt does not define it. */
struct _stat {
  _dev_t st_dev; _ino_t st_ino;
  unsigned short st_mode; short st_nlink, st_uid, st_gid;
  _dev_t st_rdev; _off_t st_size;
  __time64_t st_atime, st_mtime, st_ctime;
};

struct _stati64 {
  _dev_t st_dev; _ino_t st_ino;
  unsigned short st_mode; short st_nlink, st_uid, st_gid;
  _dev_t st_rdev; __int64 __attribute__((aligned(8))) st_size;
  __time64_t st_atime, st_mtime, st_ctime;
};

struct __utimbuf32 { __time32_t actime, modtime; };
struct __utimbuf64 { __time64_t actime, modtime; };

#pragma pack(pop)

/* ── Win32 (psdk) ───────────────────────────────────────────────────── */

typedef union _LARGE_INTEGER {
  struct { DWORD LowPart; LONG HighPart; } u;
  struct { DWORD LowPart; LONG HighPart; };
  LONGLONG QuadPart;
} LARGE_INTEGER, *PLARGE_INTEGER;

typedef struct _FILETIME {
  DWORD dwLowDateTime, dwHighDateTime;
} FILETIME, *LPFILETIME;

typedef struct _SECURITY_ATTRIBUTES {
  DWORD nLength; void *lpSecurityDescriptor; BOOL bInheritHandle;
} SECURITY_ATTRIBUTES, *LPSECURITY_ATTRIBUTES;

typedef struct _WIN32_FILE_ATTRIBUTE_DATA {
  DWORD dwFileAttributes;
  FILETIME ftCreationTime, ftLastAccessTime, ftLastWriteTime;
  DWORD nFileSizeHigh, nFileSizeLow;
} WIN32_FILE_ATTRIBUTE_DATA, *LPWIN32_FILE_ATTRIBUTE_DATA;

typedef struct _STARTUPINFOA {
  DWORD cb; char *lpReserved, *lpDesktop, *lpTitle;
  DWORD dwX, dwY, dwXSize, dwYSize, dwXCountChars, dwYCountChars,
        dwFillAttribute, dwFlags;
  WORD wShowWindow, cbReserved2; BYTE *lpReserved2;
  HANDLE hStdInput, hStdOutput, hStdError;
} STARTUPINFOA, *LPSTARTUPINFOA;

/* Verbatim from sdk/include/ndk/rtltypes.h.  Note `Spare` is ULONG_PTR[1], not
 * DWORD[2] — file.c's MSVCRT_file_cs_debug stores a string ADDRESS in Spare[0],
 * so a 32-bit element makes that initializer a truncating conversion the host
 * compiler rejects outright ("not computable at load time").  The real header
 * also C_ASSERTs sizeof == 0x30 on Win64; the layout guardian re-derives it. */
typedef struct _RTL_CRITICAL_SECTION_DEBUG {
  WORD Type, CreatorBackTraceIndex;
  struct _RTL_CRITICAL_SECTION *CriticalSection;
  struct { void *Flink, *Blink; } ProcessLocksList;
  DWORD EntryCount, ContentionCount;
  union {
    ULONG_PTR WineDebugString;
    ULONG_PTR Spare[1];
    struct { DWORD Flags; WORD CreatorBackTraceIndexHigh, SpareWORD; };
  };
} RTL_CRITICAL_SECTION_DEBUG, CRITICAL_SECTION_DEBUG,
  *PRTL_CRITICAL_SECTION_DEBUG;

typedef struct _RTL_CRITICAL_SECTION {
  PRTL_CRITICAL_SECTION_DEBUG DebugInfo;
  LONG LockCount, RecursionCount;
  HANDLE OwningThread, LockSemaphore;
  ULONG_PTR SpinCount;
} RTL_CRITICAL_SECTION, CRITICAL_SECTION, *LPCRITICAL_SECTION;

typedef unsigned char BOOLEAN;
typedef LONG NTSTATUS;

typedef struct _IO_STATUS_BLOCK {
  union { NTSTATUS Status; void *Pointer; };
  ULONG_PTR Information;
} IO_STATUS_BLOCK, *PIO_STATUS_BLOCK;

typedef struct _FILE_BASIC_INFORMATION {
  LARGE_INTEGER CreationTime, LastAccessTime, LastWriteTime, ChangeTime;
  ULONG FileAttributes;
} FILE_BASIC_INFORMATION;

typedef struct _FILE_STANDARD_INFORMATION {
  LARGE_INTEGER AllocationSize, EndOfFile;
  ULONG NumberOfLinks;
  BOOLEAN DeletePending, Directory;
} FILE_STANDARD_INFORMATION;

#endif /* MSVCRT_DIFFTEST_FILE_TYPES_H */
