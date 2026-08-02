/*
 * fileimpl.c — the OS underneath the file.c reference TU.
 *
 * file.c implements the whole msvcrt stream layer itself (the FILE ring, the
 * `__pioinfo` fd table, text/binary translation, buffering).  What it does NOT
 * implement is the layer below: it calls Win32 for the actual bytes.  This file
 * is that layer, backed by POSIX, so the reference runs on the host over a real
 * filesystem — which is what lets a difftest row compare not just return values
 * and errno but the FILE'S ACTUAL CONTENTS afterwards.
 *
 * Compiled WITHOUT the ref_* rename prelude, like refimpl.c, so these land
 * under the plain Win32 names file.c calls.  That is safe here in a way it
 * would not be for a CRT entry point: msvcrt exports none of these, so there is
 * no code-under-test for the reference to accidentally reach.  The two names
 * that ARE shared environment — GetLastError, WideCharToMultiByte — come from
 * refimpl.c, and refstub/msvcrt.h's import namespace routes them there.
 *
 * ── the one rule that matters here ──────────────────────────────────────
 *
 * A SHIM MUST MODEL THE REAL API'S ERROR BEHAVIOUR, NOT JUST ITS SUCCESS PATH.
 * This is §9.66 in the form that actually bites: a stub that always succeeds
 * makes the caller's failure branch UNREACHABLE, and every difftest row over
 * that branch then reports MATCH while proving nothing.  (`_wctomb_s_l` is the
 * worked example — its insufficient-buffer arm was dead for exactly this
 * reason.)  So every entry point below sets a last-error the caller can see,
 * and the two places where the honest answer depends on the environment rather
 * than on POSIX — is this handle a console, did we inherit an fd table — get an
 * explicit KNOB (ref_set_console_fd, ref_set_startup_handles) instead of a
 * hardcoded "no".  A knob keeps both arms reachable; a hardcoded answer is how
 * a branch quietly stops being tested.
 */
#include <msvcrt.h>

#include <errno.h>
/* ── the errno capture, and why it is not paranoia ───────────────────────
 * msvcrt_crt_consts.h below DISPLACES the host's <errno.h> values, because
 * msvcrt and glibc disagree on two that file.c can produce (EDEADLK 36 vs 35,
 * ENOTEMPTY 41 vs 39) and the reference must use msvcrt's.  But THIS file
 * reads `errno` as set by the HOST's open/read/write, so it must compare
 * against the HOST's numbering.  Capture it here, before the displacement, and
 * the two never get confused.  Without this the ENOTEMPTY arm of
 * host_err_to_win32 would simply never match, and _rmdir's "directory not
 * empty" row would grade against ERROR_ACCESS_DENIED forever. */
enum {
    HOST_EACCES = EACCES,   HOST_EAGAIN = EAGAIN,       HOST_EBADF   = EBADF,
    HOST_EEXIST = EEXIST,   HOST_EINVAL = EINVAL,       HOST_EIO     = EIO,
    HOST_EISDIR = EISDIR,   HOST_EMFILE = EMFILE,       HOST_ENFILE  = ENFILE,
    HOST_ENOENT = ENOENT,   HOST_ENOMEM = ENOMEM,       HOST_ENOSPC  = ENOSPC,
    HOST_ENOTDIR = ENOTDIR, HOST_ENOTEMPTY = ENOTEMPTY, HOST_EPERM   = EPERM,
    HOST_EPIPE  = EPIPE,    HOST_EROFS  = EROFS,        HOST_ESPIPE  = ESPIPE,
    HOST_EXDEV  = EXDEV,    HOST_ENAMETOOLONG = ENAMETOOLONG,
};

/* The types and constants, but deliberately NOT msvcrt_crt_layout.h: that
 * guardian asserts msvcrt's `struct _iobuf` layout, and this file is the layer
 * BELOW the stream layer — it never sees a FILE, so it never claims glibc's
 * guards, so `FILE` here is glibc's and the assertions would (correctly)
 * refuse.  file.c's own TU is where that guardian belongs. */
#include <msvcrt_file_types.h>
#include <msvcrt_crt_consts.h>

#include <fcntl.h>
#include <pthread.h>
#include <stdlib.h>
#include <sys/stat.h>
#include <sys/time.h>
#include <time.h>
#include <unistd.h>

/* The declarations the reference TUs see for the shims defined below.
 *
 * Included HERE, in the file that DEFINES them, so the two can never drift: a
 * shim whose definition stops matching what file.c calls is now a compile error
 * instead of an ABI mismatch surfacing as a wrong answer in an unrelated row.
 * That drift was not hypothetical — until this header existed the reference TU
 * had no declaration for ANY of these and typed all 53 `int(...)`, truncating
 * every returned pointer to 32 bits. See the header for the full story. */
#include <msvcrt_shim_decls.h>
/* The two NLS entry points this file CALLS (path_from_wide, GetTempFileNameW).
 * They were implicit here too — the same tripwire was off for every TU. */
#include <winnls.h>

/* refimpl.c owns the single last-error slot both sides read (see the long note
 * beside its GetLastError).  One writer, one reader — as in the real DLL, where
 * last-error is one TEB slot. */
void msvcrt_ref_set_last_error(DWORD err);
#define SET_ERR(e) msvcrt_ref_set_last_error(e)

/* file.c imports doserrno under the reference namespace (see
 * msvcrt_file_prelude.h); its definition, at the bottom of this file, has to
 * land on the same name.  Declared HERE rather than at the definition because
 * msvcrt_set_errno calls it first: without this the implicit-declaration rule
 * makes it `int()` and truncates the returned pointer to 32 bits.  The
 * harness's -Werror=implicit-function-declaration is what caught that. */
#define __doserrno msvcrt_ref___doserrno
__msvcrt_ulong * CDECL __doserrno(void);

/* ── THE COLLISION RULE FOR THIS FILE ────────────────────────────────────
 *
 * This is the reference's ENVIRONMENT — the layer below the stream layer — and it must
 * never call into the code under test.  But the crate under test ships `#[no_mangle]`
 * definitions of ordinary libc names, and a strong definition beats libc at link time, so
 * any libc call from here whose name msvcrt also exports silently becomes a call INTO the
 * subject.
 *
 * "`strlen` is the only one" was true while file.rs was not in the link.  It stopped being
 * true the moment it was: `MoveFileExW`'s `rename()` became msvcrt's `rename`, which calls
 * `_wrename`, which calls `MoveFileExW` — unbounded recursion, and a stack overflow before
 * the first row ran.  So the rule is now CHECKED BY THE BUILD: build.rs intersects this
 * object's undefined symbols with file.c's export list and refuses the build on a hit.
 * The two escapes below exist because that check has no fix to offer, only a name:
 *
 *   rename  -> renameat(AT_FDCWD, ...)   same operation, name msvcrt does not have
 *   fprintf -> dprintf(2, ...)                          "
 *
 * Declared here rather than by including <stdio.h>, which this file deliberately does not
 * pull (it is the layer below FILE, and claiming glibc's stdio guards is file.c's TU's
 * job). */
extern int renameat(int fromfd, const char *from, int tofd, const char *to);
extern int dprintf(int fd, const char *fmt, ...);

/* Hand-rolled for the reason above — the subject ships `#[no_mangle] strlen`. */
static size_t ref_strlen_local(const char *s)
{
    const char *p = s;

    while (*p) p++;
    return (size_t)(p - s);
}

/* ------------------------------------------------------------------ *
 * host errno -> Win32 error
 *
 * The direction that matters: file.c calls msvcrt_set_errno(GetLastError()),
 * so a wrong code here becomes a wrong errno two steps later, in a row that
 * still "runs".  Only codes reachable from the calls below are listed; an
 * unlisted one lands on ERROR_INVALID_PARAMETER, which msvcrt_set_errno maps
 * to EINVAL — the same fallback the genuine mapper has.
 * ------------------------------------------------------------------ */
static DWORD host_err_to_win32(int e)
{
    if (e == HOST_ENOENT || e == HOST_ENOTDIR) return ERROR_FILE_NOT_FOUND;
    if (e == HOST_EACCES || e == HOST_EPERM || e == HOST_EROFS ||
        e == HOST_EISDIR)                      return ERROR_ACCESS_DENIED;
    if (e == HOST_EEXIST)                      return ERROR_FILE_EXISTS;
    if (e == HOST_EBADF)                       return ERROR_INVALID_HANDLE;
    if (e == HOST_EMFILE || e == HOST_ENFILE)  return ERROR_TOO_MANY_OPEN_FILES;
    if (e == HOST_ENOSPC)                      return ERROR_DISK_FULL;
    if (e == HOST_ENOMEM)                      return ERROR_NOT_ENOUGH_MEMORY;
    if (e == HOST_EPIPE)                       return ERROR_BROKEN_PIPE;
    if (e == HOST_ENOTEMPTY)                   return ERROR_DIR_NOT_EMPTY;
    if (e == HOST_ESPIPE)                      return ERROR_SEEK_ON_DEVICE;
    if (e == HOST_EAGAIN)                      return ERROR_LOCK_VIOLATION;
    if (e == HOST_EXDEV)                       return ERROR_NOT_SUPPORTED;
    if (e == HOST_ENAMETOOLONG)                return ERROR_FILENAME_EXCED_RANGE;
    if (e == HOST_EIO)                         return ERROR_IO_DEVICE;
    if (e == HOST_EINVAL)                      return ERROR_INVALID_PARAMETER;
    return ERROR_INVALID_PARAMETER;
}

static BOOL fail_errno(void)
{
    SET_ERR(host_err_to_win32(errno));
    return FALSE;
}

/* ------------------------------------------------------------------ *
 * HANDLE <-> fd
 *
 * A HANDLE is `fd + 1`, so that fd 0 is distinguishable from NULL and no valid
 * fd can collide with INVALID_HANDLE_VALUE ((HANDLE)-1).  file.c only ever
 * stores handles and hands them back, so nothing depends on the encoding
 * beyond those two reserved values.
 * ------------------------------------------------------------------ */
#define CURRENT_PROCESS_HANDLE ((HANDLE)(LONG_PTR)-2)

static HANDLE fd_to_handle(int fd) { return (HANDLE)(LONG_PTR)(fd + 1); }

static int handle_to_fd(HANDLE h)
{
    LONG_PTR v = (LONG_PTR)h;

    if (h == INVALID_HANDLE_VALUE || h == NULL || v <= 0) return -1;
    return (int)(v - 1);
}

/* An fd that is not open is a bad handle, and saying so is the point: file.c's
 * _close/_write/_read all have an EBADF arm that only exists if this can fail. */
static int handle_fd_checked(HANDLE h)
{
    int fd = handle_to_fd(h);

    if (fd < 0 || fcntl(fd, F_GETFD) == -1) { SET_ERR(ERROR_INVALID_HANDLE); return -1; }
    return fd;
}

/* ------------------------------------------------------------------ *
 * per-fd side tables: delete-on-close, and the console knob
 * ------------------------------------------------------------------ */
#define REF_MAX_FD 4096

static char  ref_delete_on_close[REF_MAX_FD];
static char *ref_close_path[REF_MAX_FD];
static char  ref_is_console[REF_MAX_FD];

/* THE KNOB.  Whether an fd is a console is a property of the environment, not
 * of POSIX, and the difftest must be able to drive BOTH answers: file.c's
 * _write splits at `_isatty(fd) && VerifyConsoleIoHandle(hand)` into a
 * WriteConsoleW path and a WriteFile path, and hardcoding "never a console"
 * would delete the first from every run while all its rows still reported
 * MATCH.  Tests set this per fd; the default is FALSE, so a plain temp file
 * behaves like a plain temp file.
 *
 * ONE CONSTRAINT, learned the hard way and worth keeping: the fd must ALREADY
 * be a character device.  `_isatty` does not call GetFileType — it reads
 * `wxflag & WX_TTY`, which _wsopen_dispatch cached at OPEN time (file.c:2551,
 * :862).  So this knob controls only the second half of
 * `_isatty(fd) && VerifyConsoleIoHandle(hand)`, and a marked ordinary file
 * stays on the WriteFile arm no matter what.  Making GetFileType lie for a
 * marked fd does NOT fix that — the flag was already latched — it just adds
 * dead code.  Tests drive the console arm through /dev/null, which is a real
 * chardev; see tests/file_substrate.rs.
 *
 * Keyed on the HANDLE, not on a descriptor number, and that is not cosmetic:
 * `_open` returns an MSVCRT fd (an index into file.c's own `__pioinfo` table)
 * while this layer speaks POSIX fds, and the two numbering spaces are unrelated
 * — they merely coincide often enough for a test keyed on the wrong one to pass
 * by luck and then fail once the process opens a different number of files.
 * That happened.  Taking the HANDLE removes the ambiguity, because a HANDLE is
 * exactly what VerifyConsoleIoHandle is handed; callers get theirs from
 * `_get_osfhandle(crt_fd)`. */
void ref_set_console_handle(HANDLE h, int on)
{
    int fd = handle_to_fd(h);

    if (fd >= 0 && fd < REF_MAX_FD) ref_is_console[fd] = on ? 1 : 0;
}

/* Reachability, measured rather than argued.  A test that merely writes to a
 * console-marked fd cannot tell WriteConsoleW from WriteFile by its effect —
 * both end in `write(fd, …)`.  This counter is what proves the console arm was
 * the one taken. */
static unsigned long ref_console_writes;

unsigned long ref_console_write_count(void) { return ref_console_writes; }
void          ref_console_write_reset(void) { ref_console_writes = 0; }

/* The other knob: msvcrt_init_io reads GetStartupInfoA's cbReserved2/
 * lpReserved2 to inherit a parent's fd table.  A test process has no such
 * parent, so the default is "nothing inherited" — but that would make the
 * inheritance branch unreachable, so a test can plant a blob here instead. */
static BYTE *ref_startup_reserved2;
static WORD  ref_startup_cb_reserved2;

void ref_set_startup_handles(BYTE *blob, WORD len)
{
    ref_startup_reserved2   = blob;
    ref_startup_cb_reserved2 = len;
}

/* ------------------------------------------------------------------ *
 * wide <-> narrow paths
 *
 * Through refimpl.c's WideCharToMultiByte (shared environment), not the host's
 * wcstombs: the reference must convert exactly as the code under test does.
 * ------------------------------------------------------------------ */
static char *path_from_wide(const wchar_t *w)
{
    int n;
    char *s;

    if (!w) { SET_ERR(ERROR_INVALID_PARAMETER); return NULL; }
    n = WideCharToMultiByte(CP_ACP, 0, w, -1, NULL, 0, NULL, NULL);
    if (n <= 0) { SET_ERR(ERROR_INVALID_PARAMETER); return NULL; }
    if (!(s = malloc(n))) { SET_ERR(ERROR_NOT_ENOUGH_MEMORY); return NULL; }
    WideCharToMultiByte(CP_ACP, 0, w, -1, s, n, NULL, NULL);
    return s;
}

/* ------------------------------------------------------------------ *
 * critical sections
 *
 * A recursive pthread mutex, allocated LAZILY and published with a CAS.  Lazy
 * because file.c's own MSVCRT_file_cs is STATICALLY initialized
 * (`{ &MSVCRT_file_cs_debug, -1, 0, 0, 0, 0 }`, file.c:305) and never passed to
 * InitializeCriticalSection — so an implementation that only worked on
 * explicitly-initialized sections would deadlock or corrupt on the one section
 * that guards the whole FILE table.
 * ------------------------------------------------------------------ */
static pthread_mutex_t *cs_mutex(CRITICAL_SECTION *cs)
{
    pthread_mutex_t *m, *fresh, *expected = NULL;
    pthread_mutexattr_t attr;

    m = __atomic_load_n((pthread_mutex_t **)&cs->LockSemaphore, __ATOMIC_ACQUIRE);
    if (m) return m;

    if (!(fresh = malloc(sizeof(*fresh)))) return NULL;
    pthread_mutexattr_init(&attr);
    pthread_mutexattr_settype(&attr, PTHREAD_MUTEX_RECURSIVE);
    pthread_mutex_init(fresh, &attr);
    pthread_mutexattr_destroy(&attr);

    if (__atomic_compare_exchange_n((pthread_mutex_t **)&cs->LockSemaphore, &expected,
                                    fresh, 0, __ATOMIC_ACQ_REL, __ATOMIC_ACQUIRE))
        return fresh;

    pthread_mutex_destroy(fresh);
    free(fresh);
    return expected;
}

void WINAPI InitializeCriticalSection(CRITICAL_SECTION *cs)
{
    cs->DebugInfo      = NULL;
    cs->LockCount      = -1;
    cs->RecursionCount = 0;
    cs->OwningThread   = NULL;
    cs->LockSemaphore  = NULL;
    cs->SpinCount      = 0;
    cs_mutex(cs);
}

BOOL WINAPI InitializeCriticalSectionEx(CRITICAL_SECTION *cs, DWORD spin, DWORD flags)
{
    (void)spin; (void)flags;
    InitializeCriticalSection(cs);
    return TRUE;
}

void WINAPI EnterCriticalSection(CRITICAL_SECTION *cs)
{
    pthread_mutex_t *m = cs_mutex(cs);

    if (m) pthread_mutex_lock(m);
    cs->RecursionCount++;
    cs->OwningThread = (HANDLE)(ULONG_PTR)pthread_self();
}

void WINAPI LeaveCriticalSection(CRITICAL_SECTION *cs)
{
    pthread_mutex_t *m = __atomic_load_n((pthread_mutex_t **)&cs->LockSemaphore,
                                         __ATOMIC_ACQUIRE);

    if (--cs->RecursionCount == 0) cs->OwningThread = NULL;
    if (m) pthread_mutex_unlock(m);
}

BOOL WINAPI TryEnterCriticalSection(CRITICAL_SECTION *cs)
{
    pthread_mutex_t *m = cs_mutex(cs);

    if (!m) return FALSE;
    if (pthread_mutex_trylock(m) != 0) return FALSE;
    cs->RecursionCount++;
    cs->OwningThread = (HANDLE)(ULONG_PTR)pthread_self();
    return TRUE;
}

void WINAPI DeleteCriticalSection(CRITICAL_SECTION *cs)
{
    pthread_mutex_t *m = __atomic_exchange_n((pthread_mutex_t **)&cs->LockSemaphore,
                                             NULL, __ATOMIC_ACQ_REL);

    if (m) { pthread_mutex_destroy(m); free(m); }
    cs->DebugInfo      = NULL;
    cs->LockCount      = -1;
    cs->RecursionCount = 0;
    cs->OwningThread   = NULL;
}

/* ------------------------------------------------------------------ *
 * interlocked
 * ------------------------------------------------------------------ */
LONG WINAPI InterlockedCompareExchange(LONG volatile *dst, LONG xchg, LONG cmp)
{
    __atomic_compare_exchange_n(dst, &cmp, xchg, 0, __ATOMIC_SEQ_CST, __ATOMIC_SEQ_CST);
    return cmp;
}

void * WINAPI InterlockedCompareExchangePointer(void * volatile *dst, void *xchg, void *cmp)
{
    __atomic_compare_exchange_n(dst, &cmp, xchg, 0, __ATOMIC_SEQ_CST, __ATOMIC_SEQ_CST);
    return cmp;
}

/* ------------------------------------------------------------------ *
 * process / misc
 * ------------------------------------------------------------------ */
HANDLE WINAPI GetCurrentProcess(void)   { return CURRENT_PROCESS_HANDLE; }
DWORD  WINAPI GetCurrentProcessId(void) { return (DWORD)getpid(); }

void WINAPI Sleep(DWORD ms)
{
    struct timespec ts = { (time_t)(ms / 1000), (long)(ms % 1000) * 1000000L };

    nanosleep(&ts, NULL);
}

void WINAPI GetStartupInfoA(STARTUPINFOA *si)
{
    memset(si, 0, sizeof(*si));
    si->cb           = sizeof(*si);
    si->cbReserved2  = ref_startup_cb_reserved2;
    si->lpReserved2  = ref_startup_reserved2;
}

/* ------------------------------------------------------------------ *
 * std handles
 * ------------------------------------------------------------------ */
static HANDLE ref_std_handles[3];
static char   ref_std_set[3];

static int std_slot(DWORD id)
{
    if (id == STD_INPUT_HANDLE)  return 0;
    if (id == STD_OUTPUT_HANDLE) return 1;
    if (id == STD_ERROR_HANDLE)  return 2;
    return -1;
}

HANDLE WINAPI GetStdHandle(DWORD id)
{
    int slot = std_slot(id);

    if (slot < 0) { SET_ERR(ERROR_INVALID_PARAMETER); return INVALID_HANDLE_VALUE; }
    if (ref_std_set[slot]) return ref_std_handles[slot];
    return fd_to_handle(slot);
}

BOOL WINAPI SetStdHandle(DWORD id, HANDLE h)
{
    int slot = std_slot(id);

    if (slot < 0) { SET_ERR(ERROR_INVALID_PARAMETER); return FALSE; }
    ref_std_handles[slot] = h;
    ref_std_set[slot]     = 1;
    return TRUE;
}

/* Point the three standard handles at the caller's host fds.
 *
 * The default above is `fd_to_handle(slot)` — CRT fd 1 maps to HOST fd 1, the process's
 * real stdout.  That is the honest default for a program, and it makes eleven of file.c's
 * exports ungradeable: `putchar`, `puts`, `_putws` and their kin write to `stdout` with no
 * FILE parameter, so the only way to compare what the two implementations produced is for
 * each side to OWN its stdout.  Called once per side, immediately before that side's
 * `msvcrt_init_io()`, which is the only reader of GetStdHandle — so the two sides come out
 * of init bound to different files while the harness's own fd 1 is never touched.  A
 * `freopen(path, "w", stdout)` would reach the same place by CLOSING host fd 1, and the
 * next `open` would then hand fd 1 to a temp file and silently swallow every `println!`. */
void ref_set_std_fds(int in_fd, int out_fd, int err_fd)
{
    SetStdHandle(STD_INPUT_HANDLE,  fd_to_handle(in_fd));
    SetStdHandle(STD_OUTPUT_HANDLE, fd_to_handle(out_fd));
    SetStdHandle(STD_ERROR_HANDLE,  fd_to_handle(err_fd));
}

/* ------------------------------------------------------------------ *
 * console
 * ------------------------------------------------------------------ */
BOOL WINAPI VerifyConsoleIoHandle(HANDLE h)
{
    int fd = handle_to_fd(h);

    if (fd < 0 || fd >= REF_MAX_FD || !ref_is_console[fd])
    {
        SET_ERR(ERROR_INVALID_HANDLE);
        return FALSE;
    }
    return TRUE;
}

/* A real console renders UTF-16 directly.  The model writes the ACP encoding to
 * the fd, so the bytes are OBSERVABLE to a test — the point being that a row
 * over the console path can be graded at all, rather than vanishing. */
BOOL WINAPI WriteConsoleW(HANDLE h, const void *buf, DWORD len, DWORD *written, void *reserved)
{
    const wchar_t *w = buf;
    int fd = handle_fd_checked(h);
    int n;
    char *mb;
    ssize_t done;

    (void)reserved;
    ref_console_writes++;
    if (fd < 0) return FALSE;
    if (written) *written = 0;
    if (!len) return TRUE;

    n = WideCharToMultiByte(CP_ACP, 0, w, (int)len, NULL, 0, NULL, NULL);
    if (n <= 0) { SET_ERR(ERROR_INVALID_PARAMETER); return FALSE; }
    if (!(mb = malloc(n))) { SET_ERR(ERROR_NOT_ENOUGH_MEMORY); return FALSE; }
    WideCharToMultiByte(CP_ACP, 0, w, (int)len, mb, n, NULL, NULL);

    done = write(fd, mb, (size_t)n);
    free(mb);
    if (done < 0) return fail_errno();
    /* WriteConsoleW counts CHARACTERS written, not bytes. */
    if (written) *written = len;
    return TRUE;
}

/* ------------------------------------------------------------------ *
 * CreateFileW and friends
 * ------------------------------------------------------------------ */
BOOL WINAPI CloseHandle(HANDLE h)
{
    int fd = handle_fd_checked(h);

    if (fd < 0) return FALSE;
    if (fd < REF_MAX_FD && ref_delete_on_close[fd])
    {
        if (ref_close_path[fd]) unlink(ref_close_path[fd]);
        free(ref_close_path[fd]);
        ref_close_path[fd]      = NULL;
        ref_delete_on_close[fd] = 0;
    }
    if (fd < REF_MAX_FD) ref_is_console[fd] = 0;
    if (close(fd) != 0) return fail_errno();
    return TRUE;
}

/* The last dwShareMode CreateFileW was asked for.
 *
 * Windows share modes have no POSIX equivalent, so this shim cannot ACT on the
 * argument — and that would normally make it invisible.  But mapping _SH_DENYRW
 * / _SH_DENYWR / _SH_DENYRD / _SH_DENYNO onto dwShareMode is the ENTIRE reason
 * _fsopen, _wfsopen, _sopen_s and _wsopen_s exist as separate exports: strip the
 * mapping out and every one of them still opens the file, still returns the same
 * descriptor, and still leaves the same bytes on disk.  Recording the argument
 * makes the mapping itself the observation, which is the most this substrate can
 * honestly say about it — the row grades that both sides ASK for the same thing,
 * not that anything enforces it. */
static DWORD ref_last_share_mode;

DWORD ref_get_last_share_mode(void)
{
    return ref_last_share_mode;
}

HANDLE WINAPI CreateFileW(const wchar_t *name, DWORD access, DWORD share,
                          SECURITY_ATTRIBUTES *sa, DWORD disposition,
                          DWORD flags, HANDLE template_file)
{
    char *path = path_from_wide(name);
    int oflags = 0, fd;
    struct stat st;
    int existed;

    ref_last_share_mode = share;
    (void)template_file;
    if (!path) return INVALID_HANDLE_VALUE;

    if ((access & GENERIC_READ) && (access & GENERIC_WRITE)) oflags = O_RDWR;
    else if (access & GENERIC_WRITE)                         oflags = O_WRONLY;
    else                                                     oflags = O_RDONLY;

    existed = stat(path, &st) == 0;

    /* The dispositions differ in their ERROR behaviour, not just in whether
     * they create — and file.c branches on exactly that.  CREATE_NEW must fail
     * ERROR_FILE_EXISTS; OPEN_EXISTING must fail ERROR_FILE_NOT_FOUND; and both
     * OPEN_ALWAYS and CREATE_ALWAYS must report ERROR_ALREADY_EXISTS as the
     * last error even when they SUCCEED, which is how _wsopen_dispatch learns
     * that an _O_CREAT|_O_EXCL open really did create the file. */
    switch (disposition)
    {
    case CREATE_NEW:        oflags |= O_CREAT | O_EXCL;  break;
    case CREATE_ALWAYS:     oflags |= O_CREAT | O_TRUNC; break;
    case OPEN_EXISTING:                                  break;
    case OPEN_ALWAYS:       oflags |= O_CREAT;           break;
    case TRUNCATE_EXISTING: oflags |= O_TRUNC;           break;
    default:
        free(path);
        SET_ERR(ERROR_INVALID_PARAMETER);
        return INVALID_HANDLE_VALUE;
    }

    if (sa && !sa->bInheritHandle) oflags |= O_CLOEXEC;

    fd = open(path, oflags, 0666);
    if (fd < 0)
    {
        free(path);
        fail_errno();
        return INVALID_HANDLE_VALUE;
    }
    if (fd >= REF_MAX_FD)
    {
        close(fd);
        free(path);
        SET_ERR(ERROR_TOO_MANY_OPEN_FILES);
        return INVALID_HANDLE_VALUE;
    }

    /* Win32 applies dwFlagsAndAttributes ONLY when the call actually creates the
     * file, and FILE_ATTRIBUTE_READONLY is the one bit POSIX can express — the
     * same bit, and the same treatment, as SetFileAttributesW below.
     *
     * This is not decoration.  _wsopen_dispatch computes it as
     * `!(pmode & ~MSVCRT_umask & _S_IWRITE)`, so it is the ONLY observable
     * consequence of `_umask`: without it the umask is a variable that is
     * written, read back, and never acted on, and a difftest row over it would
     * report MATCH having proved nothing (the §9.66 shape this file's header
     * rule exists to prevent). */
    if (!existed && (flags & FILE_ATTRIBUTE_READONLY) &&
        (disposition == CREATE_NEW || disposition == CREATE_ALWAYS ||
         disposition == OPEN_ALWAYS))
    {
        struct stat cst;
        if (fstat(fd, &cst) == 0)
            (void)fchmod(fd, cst.st_mode & ~(mode_t)(S_IWUSR | S_IWGRP | S_IWOTH));
    }

    if (flags & FILE_FLAG_DELETE_ON_CLOSE)
    {
        ref_delete_on_close[fd] = 1;
        free(ref_close_path[fd]);
        ref_close_path[fd] = path;
    }
    else free(path);

    if (existed && (disposition == OPEN_ALWAYS || disposition == CREATE_ALWAYS))
        SET_ERR(ERROR_ALREADY_EXISTS);
    else
        SET_ERR(ERROR_SUCCESS);
    return fd_to_handle(fd);
}

BOOL WINAPI ReadFile(HANDLE h, void *buf, DWORD len, DWORD *read_bytes, void *overlapped)
{
    int fd = handle_fd_checked(h);
    ssize_t n;

    (void)overlapped;
    if (fd < 0) return FALSE;
    n = read(fd, buf, len);
    if (n < 0) { if (read_bytes) *read_bytes = 0; return fail_errno(); }
    if (read_bytes) *read_bytes = (DWORD)n;
    SET_ERR(ERROR_SUCCESS);
    return TRUE;
}

BOOL WINAPI WriteFile(HANDLE h, const void *buf, DWORD len, DWORD *written, void *overlapped)
{
    int fd = handle_fd_checked(h);
    ssize_t n;

    (void)overlapped;
    if (fd < 0) return FALSE;
    n = write(fd, buf, len);
    if (n < 0) { if (written) *written = 0; return fail_errno(); }
    if (written) *written = (DWORD)n;
    SET_ERR(ERROR_SUCCESS);
    return TRUE;
}

DWORD WINAPI SetFilePointer(HANDLE h, LONG low, LONG *high, DWORD method)
{
    int fd = handle_fd_checked(h);
    long long off;
    off_t got;
    int whence;

    if (fd < 0) return INVALID_SET_FILE_POINTER;

    if (method == FILE_BEGIN)        whence = SEEK_SET;
    else if (method == FILE_CURRENT) whence = SEEK_CUR;
    else if (method == FILE_END)     whence = SEEK_END;
    else { SET_ERR(ERROR_INVALID_PARAMETER); return INVALID_SET_FILE_POINTER; }

    /* The low half is SIGNED when there is no high half and UNSIGNED when there
     * is — the API's oldest wart, and file.c's _lseeki64 depends on it. */
    off = high ? (long long)(((unsigned long long)(ULONG)*high << 32) | (ULONG)low)
               : (long long)low;

    got = lseek(fd, (off_t)off, whence);
    if (got == (off_t)-1)
    {
        SET_ERR(errno == HOST_EINVAL ? ERROR_NEGATIVE_SEEK : host_err_to_win32(errno));
        return INVALID_SET_FILE_POINTER;
    }
    if (high) *high = (LONG)((unsigned long long)got >> 32);
    SET_ERR(ERROR_SUCCESS);
    return (DWORD)((unsigned long long)got & 0xFFFFFFFFu);
}

BOOL WINAPI GetFileSizeEx(HANDLE h, LARGE_INTEGER *size)
{
    int fd = handle_fd_checked(h);
    struct stat st;

    if (fd < 0) return FALSE;
    if (fstat(fd, &st) != 0) return fail_errno();
    size->QuadPart = (LONGLONG)st.st_size;
    SET_ERR(ERROR_SUCCESS);
    return TRUE;
}

BOOL WINAPI SetEndOfFile(HANDLE h)
{
    int fd = handle_fd_checked(h);
    off_t pos;

    if (fd < 0) return FALSE;
    if ((pos = lseek(fd, 0, SEEK_CUR)) == (off_t)-1) return fail_errno();
    if (ftruncate(fd, pos) != 0) return fail_errno();
    SET_ERR(ERROR_SUCCESS);
    return TRUE;
}

BOOL WINAPI FlushFileBuffers(HANDLE h)
{
    int fd = handle_fd_checked(h);

    if (fd < 0) return FALSE;
    if (fsync(fd) != 0) return fail_errno();
    SET_ERR(ERROR_SUCCESS);
    return TRUE;
}

DWORD WINAPI GetFileType(HANDLE h)
{
    int fd = handle_to_fd(h);
    struct stat st;

    if (fd < 0 || fstat(fd, &st) != 0) { SET_ERR(ERROR_INVALID_HANDLE); return FILE_TYPE_UNKNOWN; }
    SET_ERR(ERROR_SUCCESS);
    if (S_ISCHR(st.st_mode))  return FILE_TYPE_CHAR;
    if (S_ISFIFO(st.st_mode) || S_ISSOCK(st.st_mode)) return FILE_TYPE_PIPE;
    return FILE_TYPE_DISK;
}

/* ------------------------------------------------------------------ *
 * path-level operations
 * ------------------------------------------------------------------ */
static DWORD attrs_from_stat(const struct stat *st)
{
    DWORD a = 0;

    if (S_ISDIR(st->st_mode)) a |= FILE_ATTRIBUTE_DIRECTORY;
    if (!(st->st_mode & S_IWUSR)) a |= FILE_ATTRIBUTE_READONLY;
    return a ? a : FILE_ATTRIBUTE_NORMAL;
}

DWORD WINAPI GetFileAttributesW(const wchar_t *name)
{
    char *path = path_from_wide(name);
    struct stat st;
    DWORD a;

    if (!path) return INVALID_FILE_ATTRIBUTES;
    if (stat(path, &st) != 0) { free(path); fail_errno(); return INVALID_FILE_ATTRIBUTES; }
    a = attrs_from_stat(&st);
    free(path);
    SET_ERR(ERROR_SUCCESS);
    return a;
}

DWORD WINAPI GetFileAttributesA(const char *name)
{
    struct stat st;

    if (!name) { SET_ERR(ERROR_INVALID_PARAMETER); return INVALID_FILE_ATTRIBUTES; }
    if (stat(name, &st) != 0) { fail_errno(); return INVALID_FILE_ATTRIBUTES; }
    SET_ERR(ERROR_SUCCESS);
    return attrs_from_stat(&st);
}

BOOL WINAPI SetFileAttributesW(const wchar_t *name, DWORD attrs)
{
    char *path = path_from_wide(name);
    struct stat st;
    mode_t mode;

    if (!path) return FALSE;
    if (stat(path, &st) != 0) { free(path); return fail_errno(); }

    /* Only READONLY is expressible on POSIX, and that is the only bit file.c's
     * _wchmod round-trips. */
    mode = st.st_mode & 07777;
    if (attrs & FILE_ATTRIBUTE_READONLY) mode &= ~(mode_t)(S_IWUSR | S_IWGRP | S_IWOTH);
    else                                 mode |= S_IWUSR;

    if (chmod(path, mode) != 0) { free(path); return fail_errno(); }
    free(path);
    SET_ERR(ERROR_SUCCESS);
    return TRUE;
}

/* The 100ns-since-1601 epoch Windows uses; 11644473600 seconds before Unix. */
#define WIN_EPOCH_DELTA 116444736000000000LL

static void filetime_from_unix(FILETIME *ft, time_t sec, long nsec)
{
    long long t = (long long)sec * 10000000LL + nsec / 100 + WIN_EPOCH_DELTA;

    ft->dwLowDateTime  = (DWORD)((unsigned long long)t & 0xFFFFFFFFu);
    ft->dwHighDateTime = (DWORD)((unsigned long long)t >> 32);
}

static long long unix100ns_from_filetime(const FILETIME *ft)
{
    return (long long)(((unsigned long long)ft->dwHighDateTime << 32) | ft->dwLowDateTime)
           - WIN_EPOCH_DELTA;
}

BOOL WINAPI GetFileAttributesExW(const wchar_t *name, int level,
                                 WIN32_FILE_ATTRIBUTE_DATA *data)
{
    char *path;
    struct stat st;

    if (level != GetFileExInfoStandard) { SET_ERR(ERROR_INVALID_PARAMETER); return FALSE; }
    if (!(path = path_from_wide(name))) return FALSE;
    if (stat(path, &st) != 0) { free(path); return fail_errno(); }
    free(path);

    memset(data, 0, sizeof(*data));
    data->dwFileAttributes = attrs_from_stat(&st);
    filetime_from_unix(&data->ftCreationTime,   st.st_ctim.tv_sec, st.st_ctim.tv_nsec);
    filetime_from_unix(&data->ftLastAccessTime, st.st_atim.tv_sec, st.st_atim.tv_nsec);
    filetime_from_unix(&data->ftLastWriteTime,  st.st_mtim.tv_sec, st.st_mtim.tv_nsec);
    data->nFileSizeLow  = (DWORD)((unsigned long long)st.st_size & 0xFFFFFFFFu);
    data->nFileSizeHigh = (DWORD)((unsigned long long)st.st_size >> 32);
    SET_ERR(ERROR_SUCCESS);
    return TRUE;
}

BOOL WINAPI SetFileTime(HANDLE h, const FILETIME *create, const FILETIME *access,
                        const FILETIME *write)
{
    int fd = handle_fd_checked(h);
    struct timespec ts[2];

    (void)create;   /* POSIX has no creation time to set. */
    if (fd < 0) return FALSE;

    if (access)
    {
        long long t = unix100ns_from_filetime(access);
        ts[0].tv_sec = (time_t)(t / 10000000LL);
        ts[0].tv_nsec = (long)((t % 10000000LL) * 100);
    }
    else ts[0].tv_nsec = UTIME_OMIT;

    if (write)
    {
        long long t = unix100ns_from_filetime(write);
        ts[1].tv_sec = (time_t)(t / 10000000LL);
        ts[1].tv_nsec = (long)((t % 10000000LL) * 100);
    }
    else ts[1].tv_nsec = UTIME_OMIT;

    if (futimens(fd, ts) != 0) return fail_errno();
    SET_ERR(ERROR_SUCCESS);
    return TRUE;
}

BOOL WINAPI DeleteFileW(const wchar_t *name)
{
    char *path = path_from_wide(name);

    if (!path) return FALSE;
    if (unlink(path) != 0) { free(path); return fail_errno(); }
    free(path);
    SET_ERR(ERROR_SUCCESS);
    return TRUE;
}

BOOL WINAPI MoveFileExW(const wchar_t *from, const wchar_t *to, DWORD flags)
{
    char *a = path_from_wide(from), *b = NULL;
    int ok;

    /* flags is deliberately unused: file.c's only call passes
     * MOVEFILE_COPY_ALLOWED, so the MOVEFILE_REPLACE_EXISTING arm is not
     * reachable from the code under test and modelling it would mean adding a
     * constant nothing derives a behaviour from.  If a caller ever passes it,
     * this shim will refuse the rename rather than silently clobber — wrong,
     * but wrong LOUDLY, which is the trade this file takes everywhere. */
    (void)flags;
    if (!a) return FALSE;
    if (!(b = path_from_wide(to))) { free(a); return FALSE; }

    /* Win32 rename does NOT clobber by default; POSIX rename always does.
     * file.c's _wrename relies on the failure. */
    if (access(b, F_OK) == 0) { free(a); free(b); SET_ERR(ERROR_ALREADY_EXISTS); return FALSE; }

    /* renameat, NOT rename — see the collision note at the top of this file.  msvcrt
     * exports a `rename`, file.rs defines it, and a strong definition beats libc, so a
     * plain `rename()` here becomes MoveFileExW -> msvcrt rename -> _wrename ->
     * MoveFileExW: unbounded recursion, and it is the reference's environment calling
     * into the code under test.  `renameat` is the same POSIX operation under a name
     * msvcrt does not have. */
    ok = renameat(AT_FDCWD, a, AT_FDCWD, b) == 0;
    free(a); free(b);
    if (!ok) return fail_errno();
    SET_ERR(ERROR_SUCCESS);
    return TRUE;
}

DWORD WINAPI GetTempFileNameW(const wchar_t *dir, const wchar_t *prefix,
                              UINT unique, wchar_t *out)
{
    char tmpl[MAX_PATH * 4];
    char *d = NULL, *p = NULL;
    int fd, n;

    if (!(d = path_from_wide(dir))) return 0;
    if (prefix && !(p = path_from_wide(prefix))) { free(d); return 0; }

    n = snprintf(tmpl, sizeof(tmpl), "%s%s%.3sXXXXXX", d,
                 (d[0] && d[ref_strlen_local(d) - 1] == '/') ? "" : "/", p ? p : "tmp");
    free(d); free(p);
    if (n < 0 || (size_t)n >= sizeof(tmpl)) { SET_ERR(ERROR_FILENAME_EXCED_RANGE); return 0; }

    /* uUnique == 0 means "pick one and CREATE the file", which is what file.c's
     * tmpnam path depends on — the name must be reserved, not merely proposed. */
    if ((fd = mkstemp(tmpl)) < 0) { fail_errno(); return 0; }
    close(fd);
    if (unique) unlink(tmpl);

    /* Through the same converter as every other path in this file, not a raw
     * byte widen: the two must agree, or a non-ASCII TMPDIR would produce a name
     * this shim can create but path_from_wide cannot re-open. */
    if (!MultiByteToWideChar(CP_ACP, 0, tmpl, -1, out, MAX_PATH))
    {
        SET_ERR(ERROR_FILENAME_EXCED_RANGE);
        return 0;
    }
    SET_ERR(ERROR_SUCCESS);
    /* Win32 returns the unique number it used. file.c only tests non-zero. */
    return unique ? unique : 1;
}

/* ------------------------------------------------------------------ *
 * pipes, handle duplication, byte-range locks
 * ------------------------------------------------------------------ */
BOOL WINAPI CreatePipe(HANDLE *read_h, HANDLE *write_h, SECURITY_ATTRIBUTES *sa, DWORD size)
{
    int fds[2];

    (void)size;
    if (pipe(fds) != 0) return fail_errno();
    if (sa && !sa->bInheritHandle)
    {
        fcntl(fds[0], F_SETFD, FD_CLOEXEC);
        fcntl(fds[1], F_SETFD, FD_CLOEXEC);
    }
    *read_h  = fd_to_handle(fds[0]);
    *write_h = fd_to_handle(fds[1]);
    SET_ERR(ERROR_SUCCESS);
    return TRUE;
}

BOOL WINAPI DuplicateHandle(HANDLE src_proc, HANDLE src, HANDLE dst_proc, HANDLE *dst,
                            DWORD access, BOOL inherit, DWORD options)
{
    int fd, dup_fd;

    (void)src_proc; (void)dst_proc; (void)access; (void)options;
    if ((fd = handle_fd_checked(src)) < 0) return FALSE;

    dup_fd = inherit ? dup(fd) : fcntl(fd, F_DUPFD_CLOEXEC, 0);
    if (dup_fd < 0) return fail_errno();
    if (dup_fd >= REF_MAX_FD) { close(dup_fd); SET_ERR(ERROR_TOO_MANY_OPEN_FILES); return FALSE; }

    *dst = fd_to_handle(dup_fd);
    SET_ERR(ERROR_SUCCESS);
    return TRUE;
}

static BOOL range_lock(HANDLE h, DWORD off_low, DWORD off_high,
                       DWORD len_low, DWORD len_high, int type)
{
    int fd = handle_fd_checked(h);
    struct flock fl;

    if (fd < 0) return FALSE;
    memset(&fl, 0, sizeof(fl));
    fl.l_type   = type;
    fl.l_whence = SEEK_SET;
    fl.l_start  = (off_t)(((unsigned long long)off_high << 32) | off_low);
    fl.l_len    = (off_t)(((unsigned long long)len_high << 32) | len_low);

    if (fcntl(fd, F_SETLK, &fl) != 0)
    {
        SET_ERR(ERROR_LOCK_VIOLATION);
        return FALSE;
    }
    SET_ERR(ERROR_SUCCESS);
    return TRUE;
}

BOOL WINAPI LockFile(HANDLE h, DWORD ol, DWORD oh, DWORD ll, DWORD lh)
{
    return range_lock(h, ol, oh, ll, lh, F_WRLCK);
}

BOOL WINAPI UnlockFile(HANDLE h, DWORD ol, DWORD oh, DWORD ll, DWORD lh)
{
    return range_lock(h, ol, oh, ll, lh, F_UNLCK);
}

/* ------------------------------------------------------------------ *
 * the NT layer file.c reaches for directly
 * ------------------------------------------------------------------ */
BOOLEAN WINAPI RtlTimeToSecondsSince1970(const LARGE_INTEGER *time, ULONG *seconds)
{
    long long secs = (time->QuadPart - WIN_EPOCH_DELTA) / 10000000LL;

    if (secs < 0 || secs > 0xFFFFFFFFLL) return FALSE;
    *seconds = (ULONG)secs;
    return TRUE;
}

#define REF_STATUS_SUCCESS           ((NTSTATUS)0x00000000)
#define REF_STATUS_INVALID_HANDLE    ((NTSTATUS)0xC0000008)
#define REF_STATUS_INVALID_PARAMETER ((NTSTATUS)0xC000000D)

NTSTATUS WINAPI NtQueryInformationFile(HANDLE h, IO_STATUS_BLOCK *iosb, void *info,
                                       ULONG len, int class)
{
    int fd = handle_to_fd(h);
    struct stat st;

    if (fd < 0 || fstat(fd, &st) != 0) return REF_STATUS_INVALID_HANDLE;

    if (class == FileBasicInformation)
    {
        FILE_BASIC_INFORMATION *bi = info;

        if (len < sizeof(*bi)) return REF_STATUS_INVALID_PARAMETER;
        memset(bi, 0, sizeof(*bi));
        bi->CreationTime.QuadPart   = (long long)st.st_ctim.tv_sec * 10000000LL
                                      + st.st_ctim.tv_nsec / 100 + WIN_EPOCH_DELTA;
        bi->LastAccessTime.QuadPart = (long long)st.st_atim.tv_sec * 10000000LL
                                      + st.st_atim.tv_nsec / 100 + WIN_EPOCH_DELTA;
        bi->LastWriteTime.QuadPart  = (long long)st.st_mtim.tv_sec * 10000000LL
                                      + st.st_mtim.tv_nsec / 100 + WIN_EPOCH_DELTA;
        bi->ChangeTime              = bi->LastWriteTime;
        bi->FileAttributes          = attrs_from_stat(&st);
        if (iosb) iosb->Information = sizeof(*bi);
        return REF_STATUS_SUCCESS;
    }

    if (class == FileStandardInformation)
    {
        FILE_STANDARD_INFORMATION *si = info;

        if (len < sizeof(*si)) return REF_STATUS_INVALID_PARAMETER;
        memset(si, 0, sizeof(*si));
        si->AllocationSize.QuadPart = (LONGLONG)st.st_blocks * 512;
        si->EndOfFile.QuadPart      = (LONGLONG)st.st_size;
        si->NumberOfLinks           = (ULONG)st.st_nlink;
        si->Directory               = S_ISDIR(st.st_mode) ? 1 : 0;
        if (iosb) iosb->Information = sizeof(*si);
        return REF_STATUS_SUCCESS;
    }

    return REF_STATUS_INVALID_PARAMETER;
}

/* ------------------------------------------------------------------ *
 * the msvcrt internals file.c imports from TUs that are not references
 *
 * Each is either genuine msvcrt logic TRANSCRIBED (msvcrt_set_errno,
 * msvcrt_wstrdupa — both pure functions, both with derived constants so a
 * transcription slip cannot be silent) or process state a test process has to
 * own anyway (_commode, _fmode, the lock table).
 * ------------------------------------------------------------------ */
unsigned int MSVCRT__commode;
static int   ref_fmode;

int CDECL _get_fmode(int *mode)
{
    if (!mode) { *_errno() = EINVAL; return EINVAL; }
    *mode = ref_fmode;
    return 0;
}

/* file.c: `buf->st_dev = buf->st_rdev = _getdrive() - 1`, so 3 gives st_dev 2,
 * which is what msvcrt reports for a file on C:. */
int CDECL _getdrive(void) { return 3; }

__time64_t CDECL _time64(__time64_t *t)
{
    __time64_t now = (__time64_t)time(NULL);

    if (t) *t = now;
    return now;
}

/* The real _wgetenv returns a pointer into msvcrt's wide environ block, which
 * this harness has no reason to maintain; a thread-local buffer has the same
 * lifetime from any caller's point of view (valid until the next call on the
 * same thread) and file.c uses it only for L"TMP" in the tmpnam path. */
wchar_t * CDECL _wgetenv(const wchar_t *name)
{
    static __thread wchar_t value[4096];
    char narrow[1024];
    const char *got;
    size_t i;

    for (i = 0; name[i] && i < sizeof(narrow) - 1; i++) narrow[i] = (char)name[i];
    narrow[i] = 0;
    if (!(got = getenv(narrow))) return NULL;
    for (i = 0; got[i] && i < sizeof(value) / sizeof(value[0]) - 1; i++)
        value[i] = (wchar_t)(unsigned char)got[i];
    value[i] = 0;
    return value;
}

/* msvcrt's global lock table (mtdll.h).  file.c takes _STREAM_LOCKS + n around
 * the FILE ring and _EXIT_LOCK1 for _fcloseall.
 *
 * Sized from the real header rather than rounded up, so that an out-of-range
 * lock number is a HARD FAILURE below rather than a silently skipped lock —
 * which would look like the code simply not needing mutual exclusion, and would
 * surface much later as a corrupted FILE ring under parallel tests. */
#define REF_STREAM_LOCKS 28 /* mtdll.h: _STREAM_LOCKS */
#define REF_IOB_ENTRIES  20 /* mtdll.h: _IOB_ENTRIES  */
#define REF_TOTAL_LOCKS  (REF_STREAM_LOCKS + REF_IOB_ENTRIES)
static pthread_mutex_t ref_locks[REF_TOTAL_LOCKS];
static pthread_once_t  ref_locks_once = PTHREAD_ONCE_INIT;

static void ref_locks_init(void)
{
    pthread_mutexattr_t attr;
    int i;

    pthread_mutexattr_init(&attr);
    pthread_mutexattr_settype(&attr, PTHREAD_MUTEX_RECURSIVE);
    for (i = 0; i < REF_TOTAL_LOCKS; i++) pthread_mutex_init(&ref_locks[i], &attr);
    pthread_mutexattr_destroy(&attr);
}

static pthread_mutex_t *ref_lock_slot(int locknum)
{
    pthread_once(&ref_locks_once, ref_locks_init);
    if (locknum < 0 || locknum >= REF_TOTAL_LOCKS)
    {
        /* Loud on purpose.  Skipping the lock would leave the reference running
         * unsynchronised and look like it worked. */
        /* dprintf, not fprintf: msvcrt exports `fprintf` too (the varargs shim), and a
         * diagnostic that routes through the code under test is the one call that must
         * not.  See the collision note at the top of this file. */
        dprintf(2, "fileimpl: _lock(%d) out of range [0,%d) — mtdll.h moved\n",
                locknum, REF_TOTAL_LOCKS);
        abort();
    }
    return &ref_locks[locknum];
}

/* Which slot the last _lock / _unlock took.
 *
 * `_lock_file` has two arms and NOTHING ELSE distinguishes them: for a FILE inside the
 * _iob array it takes global lock `_STREAM_LOCKS + (file - _iob)`, and for any other FILE
 * it enters that FILE's own CRITICAL_SECTION.  The critical-section arm leaves a mark a
 * row can read (RecursionCount, in the FILE's own memory, per side).  The _lock arm
 * leaves NONE — a recursive pthread mutex taken and released is invisible — so without
 * this recorder a row could not tell "took the right global lock" from "took the wrong
 * one" from "took none at all", and the arm that guards the whole std-stream ring would
 * be ungradeable.
 *
 * ONE definition serving both sides, like ref_get_last_share_mode: read it immediately
 * after each side's call, never once at the end. */
static int ref_last_lock_num   = -1;
static int ref_last_unlock_num = -1;

int ref_get_last_lock(void)   { return ref_last_lock_num; }
int ref_get_last_unlock(void) { return ref_last_unlock_num; }
void ref_reset_last_locks(void) { ref_last_lock_num = ref_last_unlock_num = -1; }

void CDECL _lock(int locknum)
{
    ref_last_lock_num = locknum;
    pthread_mutex_lock(ref_lock_slot(locknum));
}

void CDECL _unlock(int locknum)
{
    ref_last_unlock_num = locknum;
    pthread_mutex_unlock(ref_lock_slot(locknum));
}

/* VERBATIM from dll/win32/msvcrt/data.c, which is not a reference TU. */
wchar_t *msvcrt_wstrdupa(const char *str)
{
    const unsigned int len = ref_strlen_local(str) + 1;
    wchar_t *wstr = malloc(len * sizeof(wchar_t));

    if (!wstr) return NULL;
    MultiByteToWideChar(CP_ACP, MB_PRECOMPOSED, str, len, wstr, len);
    return wstr;
}

/* ── errno.c's Win32-error -> errno mapper, transcribed ──────────────────
 *
 * errno.c is not (yet) a reference TU, and file.c calls this from ~60 places —
 * it IS the errno behaviour a difftest row grades.  So the table is transcribed
 * case-for-case from dll/win32/msvcrt/errno.c, and every constant on both sides
 * of it is DERIVED by the pinned cross-compiler (see gen_msvcrt_difftest_consts
 * — that is also why EDEADLK is msvcrt's 36 here and not glibc's 35).
 *
 * A missing case would not be a compile error: it would fall through to
 * `default: EINVAL` and look like an ordinary mapping.  The generator's
 * allowlist is what keeps the case labels honest.
 *
 * Owed: compiling errno.c as reference TU #6 would delete this transcription
 * outright and grade 18 more exports.  It is deferred only because errno.c also
 * owns _errno and _invalid_parameter, which refimpl.c currently supplies as
 * ENVIRONMENT with a test-visible call counter — swapping those is a change to
 * every existing test's plumbing, not a change to file.c.
 */
/* The Win32 -> errno MAPPING and the reference's errno store USED to live here — a
 * hand-written transcription of msvcrt's 42-case switch, called by BOTH sides because
 * neither had anything better.  That made the single most exercised mapping in the file
 * suite the one thing no row could grade: shared substrate is invisible to a differential
 * test, both sides are wrong identically, and every row reports MATCH (§9.73).  file.c
 * stores through it 54 times, and so does file.rs.
 *
 * Gone now.  errno.c is reference TU #7, so `msvcrt_set_errno` and `_errno` are msvcrt's
 * GENUINE ones (renamed into the msvcrt_ref_* import namespace by refstub/msvcrt.h, which
 * is exactly the socket the hand-written pair used to fill), while the subject keeps
 * Rust's.  The mapping is finally a difference the harness can see.  `__doserrno` went
 * with them: errno.c returns `&msvcrt_get_thread_data()->thread_doserrno`, so the slot
 * this file used to invent is now where msvcrt actually keeps it. */

/* ── the errno values the substrate test asserts ─────────────────────────
 * Exported from here, where the DERIVED msvcrt_crt_consts.h is in scope,
 * because a Rust test cannot include a C header.  Transcribing them into the
 * test instead would re-open the msvcrt-vs-glibc divergence this file exists to
 * close, and would put two more magic numbers in a tree that has a lint
 * (scan_const_contradictions) specifically to notice them.  It noticed. */
const int ref_msvcrt_ENOENT = ENOENT;
const int ref_msvcrt_EBADF = EBADF;

/* ── the reference's doserrno window ─────────────────────────────────────
 * The twin of refimpl.c's ref_errno_get/ref_errno_set, and it did not exist before
 * errno.c became a reference TU: the slot was a file-static here, so nothing outside
 * this file could read it and `msvcrt_set_errno`'s doserrno half was ungradeable even
 * in principle.  It reads through the accessor rather than a stored pointer so it lands
 * on whatever __doserrno() returns — which is now msvcrt's own
 * `&msvcrt_get_thread_data()->thread_doserrno`.  subj_doserrno_get/set are the
 * subject-side pair (subjectimpl.c); the two together are what lets a row assert that
 * both sides mapped one ERROR_* code to the same errno AND kept the same doserrno. */
unsigned int ref_doserrno_get(void)           { return *__doserrno(); }
void         ref_doserrno_set(unsigned int v) { *__doserrno() = v; }



/* ── the three Win32 entry points errno.c needs ──────────────────────────
 * errno.c's real external surface is 15 symbols (llvm-nm on the shipped object).
 * Twelve already resolve: MultiByteToWideChar from nlsimpl.c, msvcrt_get_thread_data
 * from refimpl.c, _write from file.c, strcpy/strlen/wcslen/wcstombs from the union
 * TUs, malloc/free/_vsnprintf from the host, and ros_dbg_log/wine_dbgstr_wn drop out
 * because refstub/wine/debug.h no-ops every channel macro.  These three are what is
 * left.  They live here rather than in refimpl.c for the reason at the top of this
 * file: this is the reference's OS layer, and these are OS calls.
 *
 * SetErrorMode is the only one with state, and modelling it as a plain read-modify-write
 * is not a shortcut — it IS the contract `_seterrormode` exposes.  Process-wide rather
 * than __thread, matching Win32: SEM_* is a process attribute.  The row that grades
 * _seterrormode compares the value each side hands back, so the store has to be real. */
static UINT ref_error_mode;

UINT WINAPI SetErrorMode(UINT mode)
{
    UINT previous = ref_error_mode;
    ref_error_mode = mode;
    return previous;
}

/* lstrcpyW / lstrcatW — kernel32's UNCHECKED wide copies, which is why they are two
 * lines rather than a call to wcscpy/wcscat.  Routing them through the CRT would send
 * the reference through cnt-rust's #[no_mangle] wcscpy, i.e. through the code under
 * test (the collision rule at the top of this file).  __wcserror is the only caller. */
LPWSTR WINAPI lstrcpyW(LPWSTR dst, LPCWSTR src)
{
    LPWSTR out = dst;
    while ((*out++ = *src++) != 0) {}
    return dst;
}

LPWSTR WINAPI lstrcatW(LPWSTR dst, LPCWSTR src)
{
    LPWSTR end = dst;
    while (*end) end++;
    while ((*end++ = *src++) != 0) {}
    return dst;
}
