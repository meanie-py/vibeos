/*
 * msvcrt_shim_decls.h — what the reference TUs CALL but do not DEFINE.
 *
 * ── why this file exists ────────────────────────────────────────────────
 *
 * A reference TU (file.c, errno.c) is the genuine msvcrt source compiled for
 * the host.  Everything below it — the Win32 layer, the msvcrt internals that
 * live in TUs we do not compile — is supplied by fileimpl.c / refimpl.c /
 * nlsimpl.c.  Nothing DECLARED them to the reference TU, so all 53 of them fell
 * under C's implicit-declaration rule and were typed `int(...)`.
 *
 * That is not a warning-level problem.  It TRUNCATES EVERY RETURNED POINTER TO
 * 32 BITS.  It survived four waves only by accident: fileimpl.c's `fd_to_handle`
 * hands back small integers, so every truncated `HANDLE` happened to round-trip.
 * The first shim that returned a real `malloc` pointer — `msvcrt_wstrdupa`, which
 * only `freopen` and `_fdopen` call, and which no row exercised until wave 18b —
 * SIGSEGV'd inside `ref_wcschr` on a pointer with its top 16 bits gone.
 *
 * `_wgetenv`, `ref__strdup`, `ref__wcsdup` and `_time64` are the same shape and
 * were still unexercised; wave 18c's tempnam/utime rows would have walked
 * straight into them.
 *
 * ── the second job this file does ───────────────────────────────────────
 *
 * fileimpl.c includes it too.  So a shim whose DEFINITION drifts from what the
 * reference TU calls is now a compile error rather than an ABI mismatch that
 * shows up as a wrong answer in an unrelated row.  That is the whole reason the
 * declarations live in a shared header instead of being pasted into the
 * generated prelude.
 *
 * ── keeping it complete ─────────────────────────────────────────────────
 *
 * `MultiByteToWideChar` / `WideCharToMultiByte` are deliberately NOT here: their
 * siblings already live in winnls.h and one declaration site per function is the
 * point of the exercise.
 *
 * `-Werror=implicit-function-declaration` is what proves this list has no holes,
 * and it is armed for real now: `cc::Build::warnings(false)` emits `-w`, and gcc's
 * `-w` DISARMS `-Werror=` diagnostics entirely, in any order and even with the
 * warning explicitly re-enabled afterwards.  build.rs said "NO -w here, on
 * purpose" in a comment directly above the call that was emitting one.
 */
#ifndef MSVCRT_REF_SHIM_DECLS_H
#define MSVCRT_REF_SHIM_DECLS_H

/* ── the Win32 / NT surface (fileimpl.c, refimpl.c, nlsimpl.c) ───────────
 *
 * `HANDLE`-returning entries are the ones the accident was hiding: they are
 * pointers, and the reference TU was reading 32 bits of them. */
BOOL     WINAPI CloseHandle(HANDLE h);
HANDLE   WINAPI CreateFileW(const wchar_t *name, DWORD access, DWORD share,
                            SECURITY_ATTRIBUTES *sa, DWORD disposition,
                            DWORD flags, HANDLE template_file);
BOOL     WINAPI CreatePipe(HANDLE *read_h, HANDLE *write_h,
                           SECURITY_ATTRIBUTES *sa, DWORD size);
/* ── the process layer (processimpl.c) ───────────────────────────────────
 *
 * `CreateProcessW` is a RECORDER, not a spawner: it copies its arguments into a
 * ledger the test reads back and returns a preset outcome.  That is the only
 * shape a difftest can use here — process.c's whole job is to MARSHAL these
 * arguments, and a shim that actually forked would grade the child's behaviour
 * instead of the marshalling, non-deterministically, twice per row.
 *
 * `WaitForSingleObject` / `GetExitCodeProcess` are presets for the same reason
 * fileimpl.c's console knob is one: msvcrt_spawn's `_P_WAIT` arm and `_cwait`'s
 * ECHILD arm are only reachable if BOTH answers can be produced on demand. */
BOOL     WINAPI CreateProcessW(const wchar_t *app, wchar_t *cmdline,
                               SECURITY_ATTRIBUTES *proc_attr,
                               SECURITY_ATTRIBUTES *thread_attr, BOOL inherit,
                               DWORD flags, void *env, const wchar_t *cwd,
                               STARTUPINFOW *si, PROCESS_INFORMATION *pi);
BOOL     WINAPI FreeLibrary(HMODULE mod);
DWORD    WINAPI GetEnvironmentVariableW(const wchar_t *name, wchar_t *buf,
                                        DWORD size);
BOOL     WINAPI GetExitCodeProcess(HANDLE proc, DWORD *code);
/* Declared `void *`-returning, which is how `_getdllprocaddr` uses it — the PSDK's
 * FARPROC would make that `return GetProcAddress(...)` a function-to-object
 * pointer conversion the host compiler complains about, for no gain here. */
void *   WINAPI GetProcAddress(HMODULE mod, const char *name);
HMODULE  WINAPI LoadLibraryW(const wchar_t *name);
DWORD    WINAPI WaitForSingleObject(HANDLE h, DWORD ms);

void     WINAPI DeleteCriticalSection(CRITICAL_SECTION *cs);
BOOL     WINAPI DeleteFileW(const wchar_t *name);
BOOL     WINAPI DuplicateHandle(HANDLE src_proc, HANDLE src, HANDLE dst_proc,
                                HANDLE *dst, DWORD access, BOOL inherit,
                                DWORD options);
void     WINAPI EnterCriticalSection(CRITICAL_SECTION *cs);
BOOL     WINAPI FlushFileBuffers(HANDLE h);
HANDLE   WINAPI GetCurrentProcess(void);
DWORD    WINAPI GetCurrentProcessId(void);
DWORD    WINAPI GetFileAttributesA(const char *name);
BOOL     WINAPI GetFileAttributesExW(const wchar_t *name, int level,
                                     WIN32_FILE_ATTRIBUTE_DATA *data);
DWORD    WINAPI GetFileAttributesW(const wchar_t *name);
BOOL     WINAPI GetFileSizeEx(HANDLE h, LARGE_INTEGER *size);
DWORD    WINAPI GetFileType(HANDLE h);
void     WINAPI GetStartupInfoA(STARTUPINFOA *si);
HANDLE   WINAPI GetStdHandle(DWORD id);
DWORD    WINAPI GetTempFileNameW(const wchar_t *dir, const wchar_t *prefix,
                                 UINT unique, wchar_t *out);
void     WINAPI InitializeCriticalSection(CRITICAL_SECTION *cs);
BOOL     WINAPI InitializeCriticalSectionEx(CRITICAL_SECTION *cs, DWORD spin,
                                            DWORD flags);
LONG     WINAPI InterlockedCompareExchange(LONG volatile *dst, LONG xchg,
                                           LONG cmp);
void *   WINAPI InterlockedCompareExchangePointer(void * volatile *dst,
                                                  void *xchg, void *cmp);
void     WINAPI LeaveCriticalSection(CRITICAL_SECTION *cs);
BOOL     WINAPI LockFile(HANDLE h, DWORD ol, DWORD oh, DWORD ll, DWORD lh);
BOOL     WINAPI MoveFileExW(const wchar_t *from, const wchar_t *to, DWORD flags);
NTSTATUS WINAPI NtQueryInformationFile(HANDLE h, IO_STATUS_BLOCK *iosb,
                                       void *info, ULONG len, int class);
BOOL     WINAPI ReadFile(HANDLE h, void *buf, DWORD len, DWORD *read_bytes,
                         void *overlapped);
BOOLEAN  WINAPI RtlTimeToSecondsSince1970(const LARGE_INTEGER *time,
                                          ULONG *seconds);
BOOL     WINAPI SetEndOfFile(HANDLE h);
UINT     WINAPI SetErrorMode(UINT mode);
BOOL     WINAPI SetFileAttributesW(const wchar_t *name, DWORD attrs);
DWORD    WINAPI SetFilePointer(HANDLE h, LONG low, LONG *high, DWORD method);
BOOL     WINAPI SetFileTime(HANDLE h, const FILETIME *create,
                            const FILETIME *access, const FILETIME *write);
BOOL     WINAPI SetStdHandle(DWORD id, HANDLE h);
void     WINAPI Sleep(DWORD ms);
BOOL     WINAPI TryEnterCriticalSection(CRITICAL_SECTION *cs);
BOOL     WINAPI UnlockFile(HANDLE h, DWORD ol, DWORD oh, DWORD ll, DWORD lh);
BOOL     WINAPI VerifyConsoleIoHandle(HANDLE h);
BOOL     WINAPI WriteConsoleW(HANDLE h, const void *buf, DWORD len,
                              DWORD *written, void *reserved);
BOOL     WINAPI WriteFile(HANDLE h, const void *buf, DWORD len, DWORD *written,
                          void *overlapped);
LPWSTR   WINAPI lstrcatW(LPWSTR dst, LPCWSTR src);
LPWSTR   WINAPI lstrcpyW(LPWSTR dst, LPCWSTR src);

/* ── msvcrt internals from TUs that are not references (fileimpl.c) ──────
 *
 * `msvcrt_wstrdupa` is the one that crashed. It is spelled exactly as
 * dll/win32/msvcrt/msvcrt.h spells it — no CDECL — because that header is what
 * the SHIPPING build compiles file.c against, and a difference here would be a
 * difference between the reference and the thing it grades. */
int          CDECL _get_fmode(int *mode);
int          CDECL _getdrive(void);
/* `ref_`-prefixed, unlike its neighbours, and the reason is the code under test:
 * msvcrt EXPORTS `_time64`, so once time.rs is in the link the plain name belongs to
 * the SUBJECT and a second strong definition here is a duplicate-symbol failure. The
 * file.c preludes map `_time64` onto this name (stage 4.5); fileimpl.c defines it. */
__time64_t   CDECL ref__time64(__time64_t *t);
wchar_t *    CDECL _wgetenv(const wchar_t *name);
wchar_t *          msvcrt_wstrdupa(const char *str);

/* ── what one reference TU calls in ANOTHER ──────────────────────────────
 *
 * Spelled in their RENAMED form on purpose. This header is included after the
 * rename wall has run, so writing `_strdup` here would be rewritten to
 * `ref__strdup` anyway — but writing the renamed name directly is what makes it
 * obvious that these resolve to the sibling reference TU rather than to glibc,
 * which is the entire point of the wall. `ref__strdup` and `ref__wcsdup` return
 * `malloc` pointers and were being truncated exactly like `msvcrt_wstrdupa`;
 * `file.c` reaches them from `tmpfile` and `_wtempnam`, both ungraded until now,
 * which is why nothing had crashed on them yet. */
char *    CDECL ref__strdup(const char *str);
wchar_t * CDECL ref__wcsdup(const wchar_t *str);
int       CDECL ref__wcsnicmp(const wchar_t *str1, const wchar_t *str2, size_t n);
int       CDECL ref__write(int fd, const void *buf, unsigned int count);
void            msvcrt_ref_msvcrt_set_errno(int err);

#endif /* MSVCRT_REF_SHIM_DECLS_H */
