/*
 * processimpl.c — the Win32 process layer underneath the process.c reference TU.
 *
 * process.c does not create processes.  It MARSHALS: it turns an `argv` array or a
 * `...` list into one space-separated command line, an `envp` array into a
 * double-NUL-terminated environment block, a bare program name into a full path by
 * walking `PATH` and trying `.com`/`.exe`/`.bat`/`.cmd`, and a `_P_*` mode into a
 * creation-flags word and a wait-or-return decision.  Then it hands all of that to
 * CreateProcessW.  Everything interesting in the file happens before that call.
 *
 * So this shim RECORDS the call rather than performing it.  CreateProcessW copies its
 * arguments into a ledger and returns a preset outcome; the test reads the ledger back
 * and compares the two sides' recordings.  That is the only shape that works here:
 *
 *   * A shim that really spawned would grade the CHILD's behaviour, not the
 *     marshalling, and would do it twice per row against a real scheduler.
 *   * A shim that dropped its arguments on the floor would make every row a
 *     comparison of two -1s.  Which is the §9.73 shape — both sides agree, and the
 *     agreement is about nothing.
 *
 * Compiled WITHOUT the ref_* rename prelude, like fileimpl.c, refimpl.c and
 * timeimpl.c, so these land under the plain Win32 names process.c calls.  Safe for the
 * same reason: msvcrt exports none of them, so there is no code under test for a
 * reference to reach.
 *
 * ── ONE LEDGER, NOT TWO, AND WHY THAT IS NOT §9.73 ──────────────────────────
 *
 * time.c's four exported globals had to be per-side storage because `_tzset` WRITES
 * them and then both sides READ them back — shared storage there makes side B read
 * side A's answer and the two agree for a reason that says nothing.
 *
 * This ledger is a SINK.  Nothing in process.c ever reads it, and the row protocol is
 * reset -> call one side -> snapshot -> reset -> call the other side -> snapshot ->
 * compare the two snapshots.  A single ledger under that protocol cannot leak one
 * side's answer into the other's, and it keeps the recorder's own code identical for
 * both — which is what you want from an instrument.
 *
 * ── the outcome is a KNOB, and both arms are load-bearing ───────────────────
 *
 * `ref_proc_set_create_result` decides whether CreateProcessW succeeds.  Both settings
 * are needed and neither is the "normal" one:
 *
 *   * FAILURE is how every `_exec*` row is graded at all.  `_exec*` passes
 *     `_P_OVERLAY`, and msvcrt_spawn's `_P_OVERLAY` arm is `_exit(0)` — on success the
 *     harness process TERMINATES.  With CreateProcessW failing, the marshalling has
 *     already happened and been recorded, and what comes back is the errno mapping.
 *   * SUCCESS is the only way to reach the `_P_WAIT` / `_P_NOWAIT` / `_P_DETACH`
 *     arms, which is where the returned handle-vs-exit-code split lives.
 *
 * Nothing here is transcribed from Windows.  The fake handles are declared data whose
 * only job is to be distinguishable; the environment (COMSPEC, the module table, the
 * files `msvcrt_search_executable` finds) is set by the test.
 */
#include <msvcrt.h>

#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include <msvcrt_file_types.h>
#include <msvcrt_crt_consts.h>

/* The declarations for what this file DEFINES, included here so a shim whose
 * definition drifts from what process.c calls is a compile error rather than an ABI
 * mismatch surfacing as a wrong answer in an unrelated row.  Same contract as
 * fileimpl.c's include of the same header. */
#include <msvcrt_shim_decls.h>

/* refimpl.c owns the single last-error slot both sides read. */
void msvcrt_ref_set_last_error(DWORD err);
#define SET_ERR(e) msvcrt_ref_set_last_error(e)

/* ── the `_P_OVERLAY` terminator ────────────────────────────────────────────
 *
 * msvcrt_spawn's `_P_OVERLAY` arm is `_exit(0)`.  Every `_exec*` row presets
 * CreateProcessW to fail so that arm is never reached — but if one ever did reach it,
 * the harness would exit ZERO having compared nothing and `cargo test` would call that
 * a pass.  Silent-green is the one outcome a differential harness must never produce,
 * so the reference TU's `_exit` is redirected here (see the prelude's stage 8) and
 * this says what happened and leaves with a code nobody mistakes for success.
 *
 * `_Exit`, not `exit`: the reference's FILE ring is half-owned by the code under test,
 * and running atexit handlers over it on the way out of an already-broken run has no
 * upside. */
void ref_process_exit_overlay(int code)
{
    fprintf(stderr,
            "\nmsvcrt-difftest: the process.c reference reached _exit(%d) — a _P_OVERLAY\n"
            "spawn SUCCEEDED. That arm terminates the process, so whatever row did this\n"
            "graded nothing after the CreateProcessW call. Preset the spawn to FAIL for\n"
            "_P_OVERLAY rows (ref_proc_set_create_result(FALSE, ...)).\n",
            code);
    fflush(stderr);
    _Exit(70);
}

/* ── the fake handles ───────────────────────────────────────────────────────
 *
 * Chosen far above any real file descriptor ON PURPOSE.  fileimpl.c models a HANDLE as
 * `fd + 1`, and its CloseHandle turns one back into an fd and calls close(2) — so a
 * process handle that happened to land in fd space would CLOSE ONE OF THE HARNESS'S
 * OWN FILES when process.c disposed of it.  `handle_fd_checked` fcntl-probes first, so
 * a value up here fails that probe and CloseHandle returns FALSE, which is exactly
 * what process.c ignores anyway. */
#define REF_PROC_HPROCESS ((HANDLE)(LONG_PTR)0x70000001)
#define REF_PROC_HTHREAD  ((HANDLE)(LONG_PTR)0x70000002)

/* ── the ledger ─────────────────────────────────────────────────────────────
 *
 * Bounded buffers with an explicit `truncated` flag rather than dynamic ones.  A
 * silently clipped command line would make two different marshallings compare equal —
 * the failure mode this whole file exists to prevent — so the flag is FAIL LOUD: the
 * test asserts it is zero on every row. */
#define REF_PROC_CMDLINE_MAX 8192
#define REF_PROC_ENV_MAX     8192
#define REF_PROC_INHERIT_MAX 1024

typedef struct {
    int      calls;                 /* how many times CreateProcessW was entered */
    int      truncated;             /* any recorded buffer overflowed — see above */

    int      app_null;
    wchar_t  app[MAX_PATH];
    unsigned app_len;

    int      cmdline_null;
    wchar_t  cmdline[REF_PROC_CMDLINE_MAX];
    unsigned cmdline_len;

    /* The environment BLOCK, verbatim including its embedded NULs, because that is
     * what `msvcrt_argvtos(envp, 0)` produces and its terminator rule ("delim ? ... :
     * ...") is one of the two branches in the function.  `env_len` counts elements up
     * to and including the final empty string's NUL. */
    int      env_null;
    wchar_t  env[REF_PROC_ENV_MAX];
    unsigned env_len;

    int      cwd_null;

    DWORD    flags;
    BOOL     inherit;
    int      proc_attr_null, thread_attr_null;

    /* STARTUPINFOW, the fields msvcrt_spawn and _wpopen actually set. */
    DWORD    si_cb;
    DWORD    si_dwFlags;
    WORD     si_cbReserved2;
    int      si_lpReserved2_null;
    unsigned si_inherit_len;
    unsigned char si_inherit[REF_PROC_INHERIT_MAX];
    HANDLE   si_hStdInput, si_hStdOutput, si_hStdError;

    /* LoadLibraryW / GetProcAddress, recorded in the same ledger so one reset clears
     * everything a row can observe. */
    int      loadlibrary_calls;
    int      loadlibrary_null;
    wchar_t  loadlibrary_name[MAX_PATH];
    int      freelibrary_calls;
    HMODULE  freelibrary_arg;
    int      getprocaddr_calls;
    HMODULE  getprocaddr_mod;
    /* `_getdllprocaddr` passes EITHER a name pointer or an ordinal cast to one, and
     * choosing between them is the whole body.  Record the raw pointer value AND, when
     * it looks like a real string, the string — so a row can tell an ordinal 7 from a
     * name at address 7. */
    ULONG_PTR getprocaddr_raw;
    int      getprocaddr_is_name;
    char     getprocaddr_name[256];
} ref_proc_ledger;

static ref_proc_ledger LEDGER;

/* The preset outcomes. */
static BOOL  create_ok        = TRUE;
static DWORD create_error     = ERROR_FILE_NOT_FOUND;
static DWORD create_pid       = 4321;
static DWORD create_tid       = 8765;
static DWORD wait_result      = WAIT_OBJECT_0;
static DWORD wait_error       = ERROR_INVALID_HANDLE;
static BOOL  exitcode_ok      = TRUE;
static DWORD exitcode_value   = 42;
static BOOL  loadlibrary_ok   = TRUE;
static BOOL  freelibrary_ok   = TRUE;
static BOOL  getprocaddr_ok   = TRUE;

/* COMSPEC.  `msvcrt_get_comspec` is the entry point for `_wpopen` and `_wsystem`, and
 * its three arms — variable present, variable absent (the `L"cmd"` fallback), and
 * HeapAlloc failing — are only distinguishable if this can be unset. */
static int     comspec_set;
static wchar_t comspec_value[MAX_PATH];

static unsigned wide_len(const wchar_t *s)
{
    unsigned n = 0;
    while (s[n]) n++;
    return n;
}

/* ── the control surface the tests drive ────────────────────────────────── */

void ref_proc_reset(void)
{
    memset(&LEDGER, 0, sizeof(LEDGER));
}

void ref_proc_set_create_result(BOOL ok, DWORD last_error, DWORD pid, DWORD tid)
{
    create_ok    = ok;
    create_error = last_error;
    create_pid   = pid;
    create_tid   = tid;
}

void ref_proc_set_wait_result(DWORD result, DWORD last_error)
{
    wait_result = result;
    wait_error  = last_error;
}

void ref_proc_set_exit_code(BOOL ok, DWORD value)
{
    exitcode_ok    = ok;
    exitcode_value = value;
}

void ref_proc_set_module_results(BOOL load_ok, BOOL free_ok, BOOL proc_ok)
{
    loadlibrary_ok = load_ok;
    freelibrary_ok = free_ok;
    getprocaddr_ok = proc_ok;
}

/* NULL means "COMSPEC is not set", which is a real environment and the only way to
 * reach `msvcrt_get_comspec`'s `L"cmd"` fallback. */
void ref_proc_set_comspec(const wchar_t *value)
{
    if (!value) { comspec_set = 0; comspec_value[0] = 0; return; }
    comspec_set = 1;
    {
        unsigned n = wide_len(value);
        if (n > MAX_PATH - 1) n = MAX_PATH - 1;
        memcpy(comspec_value, value, n * sizeof(wchar_t));
        comspec_value[n] = 0;
    }
}

/* The ledger, by pointer.  The tests read it through generated accessors rather than a
 * Rust struct copy: a layout the Rust side re-declared would be one more transcription
 * to keep in step, and this struct is harness-internal — nothing in msvcrt has an
 * opinion about it. */
int      ref_proc_calls(void)          { return LEDGER.calls; }
int      ref_proc_truncated(void)      { return LEDGER.truncated; }
int      ref_proc_app_null(void)       { return LEDGER.app_null; }
const wchar_t *ref_proc_app(void)      { return LEDGER.app; }
int      ref_proc_cmdline_null(void)   { return LEDGER.cmdline_null; }
const wchar_t *ref_proc_cmdline(void)  { return LEDGER.cmdline; }
int      ref_proc_env_null(void)       { return LEDGER.env_null; }
const wchar_t *ref_proc_env(void)      { return LEDGER.env; }
unsigned ref_proc_env_len(void)        { return LEDGER.env_len; }
int      ref_proc_cwd_null(void)       { return LEDGER.cwd_null; }
DWORD    ref_proc_flags(void)          { return LEDGER.flags; }
BOOL     ref_proc_inherit(void)        { return LEDGER.inherit; }
int      ref_proc_attrs_null(void)     { return LEDGER.proc_attr_null && LEDGER.thread_attr_null; }
DWORD    ref_proc_si_cb(void)          { return LEDGER.si_cb; }
DWORD    ref_proc_si_flags(void)       { return LEDGER.si_dwFlags; }
unsigned ref_proc_si_cbreserved2(void) { return LEDGER.si_cbReserved2; }
int      ref_proc_si_reserved2_null(void) { return LEDGER.si_lpReserved2_null; }
unsigned ref_proc_si_inherit_len(void) { return LEDGER.si_inherit_len; }
const unsigned char *ref_proc_si_inherit(void) { return LEDGER.si_inherit; }
/* The three std handles, as the integers fileimpl.c models a HANDLE with. `_wpopen` is
 * the only caller that sets them, and WHICH of the pipe's two ends goes to stdin vs
 * stdout is the `read_pipe` branch — the one thing that distinguishes "r" from "w". */
long long ref_proc_si_stdin(void)      { return (long long)(LONG_PTR)LEDGER.si_hStdInput; }
long long ref_proc_si_stdout(void)     { return (long long)(LONG_PTR)LEDGER.si_hStdOutput; }
long long ref_proc_si_stderr(void)     { return (long long)(LONG_PTR)LEDGER.si_hStdError; }

int      ref_proc_loadlibrary_calls(void)  { return LEDGER.loadlibrary_calls; }
int      ref_proc_loadlibrary_null(void)   { return LEDGER.loadlibrary_null; }
const wchar_t *ref_proc_loadlibrary_name(void) { return LEDGER.loadlibrary_name; }
int      ref_proc_freelibrary_calls(void)  { return LEDGER.freelibrary_calls; }
long long ref_proc_freelibrary_arg(void)   { return (long long)(LONG_PTR)LEDGER.freelibrary_arg; }
int      ref_proc_getprocaddr_calls(void)  { return LEDGER.getprocaddr_calls; }
long long ref_proc_getprocaddr_mod(void)   { return (long long)(LONG_PTR)LEDGER.getprocaddr_mod; }
unsigned long long ref_proc_getprocaddr_raw(void) { return (unsigned long long)LEDGER.getprocaddr_raw; }
int      ref_proc_getprocaddr_is_name(void) { return LEDGER.getprocaddr_is_name; }
const char *ref_proc_getprocaddr_name(void) { return LEDGER.getprocaddr_name; }

/* The fake handles, so a test can say what `_spawnv(_P_NOWAIT, …)` is expected to
 * return without transcribing the constant on the Rust side. */
long long ref_proc_hprocess(void) { return (long long)(LONG_PTR)REF_PROC_HPROCESS; }
long long ref_proc_hthread(void)  { return (long long)(LONG_PTR)REF_PROC_HTHREAD; }

/* ── the shims ──────────────────────────────────────────────────────────── */

static void record_wide(const wchar_t *src, wchar_t *dst, unsigned cap,
                        int *is_null, unsigned *out_len)
{
    unsigned n;

    if (!src) { *is_null = 1; *out_len = 0; dst[0] = 0; return; }
    *is_null = 0;
    n = wide_len(src);
    if (n > cap - 1) { n = cap - 1; LEDGER.truncated = 1; }
    memcpy(dst, src, n * sizeof(wchar_t));
    dst[n] = 0;
    *out_len = n;
}

/* An environment block is a run of NUL-terminated wide strings ended by an empty one,
 * so `wcslen` stops at the first entry.  Walk to the double NUL instead — that walk IS
 * the shape `msvcrt_argvtos(envp, 0)` is supposed to produce, and a recorder that
 * stopped early would compare only the first variable. */
static void record_env(const wchar_t *src)
{
    unsigned n = 0;

    if (!src) { LEDGER.env_null = 1; LEDGER.env_len = 0; return; }
    LEDGER.env_null = 0;
    while (n < REF_PROC_ENV_MAX - 1)
    {
        wchar_t c = src[n];
        LEDGER.env[n] = c;
        n++;
        if (!c && (n == 1 || !src[n - 2])) break;   /* the empty final string */
    }
    if (n >= REF_PROC_ENV_MAX - 1) LEDGER.truncated = 1;
    LEDGER.env[n] = 0;
    LEDGER.env_len = n;
}

BOOL WINAPI CreateProcessW(const wchar_t *app, wchar_t *cmdline,
                           SECURITY_ATTRIBUTES *proc_attr,
                           SECURITY_ATTRIBUTES *thread_attr, BOOL inherit,
                           DWORD flags, void *env, const wchar_t *cwd,
                           STARTUPINFOW *si, PROCESS_INFORMATION *pi)
{
    LEDGER.calls++;
    record_wide(app, LEDGER.app, MAX_PATH, &LEDGER.app_null, &LEDGER.app_len);
    record_wide(cmdline, LEDGER.cmdline, REF_PROC_CMDLINE_MAX, &LEDGER.cmdline_null,
                &LEDGER.cmdline_len);
    record_env((const wchar_t *)env);
    LEDGER.cwd_null         = cwd == NULL;
    LEDGER.flags            = flags;
    LEDGER.inherit          = inherit;
    LEDGER.proc_attr_null   = proc_attr == NULL;
    LEDGER.thread_attr_null = thread_attr == NULL;

    if (si)
    {
        LEDGER.si_cb              = si->cb;
        LEDGER.si_dwFlags         = si->dwFlags;
        LEDGER.si_cbReserved2     = si->cbReserved2;
        LEDGER.si_lpReserved2_null = si->lpReserved2 == NULL;
        LEDGER.si_hStdInput       = si->hStdInput;
        LEDGER.si_hStdOutput      = si->hStdOutput;
        LEDGER.si_hStdError       = si->hStdError;
        /* The CRT's fd-inherit block, byte for byte.  file.c BUILDS it
         * (`msvcrt_create_io_inherit_block`) and process.c only carries it, but the
         * carrying is the part that can go wrong: a spawn that passed a NULL block, or
         * the wrong length, loses the parent's open descriptors in the child. */
        if (si->lpReserved2 && si->cbReserved2)
        {
            unsigned n = si->cbReserved2;
            if (n > REF_PROC_INHERIT_MAX) { n = REF_PROC_INHERIT_MAX; LEDGER.truncated = 1; }
            memcpy(LEDGER.si_inherit, si->lpReserved2, n);
            LEDGER.si_inherit_len = n;
        }
    }

    if (!create_ok) { SET_ERR(create_error); return FALSE; }
    if (pi)
    {
        pi->hProcess     = REF_PROC_HPROCESS;
        pi->hThread      = REF_PROC_HTHREAD;
        pi->dwProcessId  = create_pid;
        pi->dwThreadId   = create_tid;
    }
    SET_ERR(ERROR_SUCCESS);
    return TRUE;
}

DWORD WINAPI WaitForSingleObject(HANDLE h, DWORD ms)
{
    (void)h; (void)ms;
    if (wait_result == WAIT_FAILED) SET_ERR(wait_error);
    else                            SET_ERR(ERROR_SUCCESS);
    return wait_result;
}

BOOL WINAPI GetExitCodeProcess(HANDLE proc, DWORD *code)
{
    (void)proc;
    if (!exitcode_ok) { SET_ERR(wait_error); return FALSE; }
    if (code) *code = exitcode_value;
    SET_ERR(ERROR_SUCCESS);
    return TRUE;
}

/* Windows' contract, modelled exactly, because `msvcrt_get_comspec` depends on all
 * three of its answers: with a NULL/short buffer it returns the size INCLUDING the
 * terminator, on success the count EXCLUDING it, and 0 when the variable is unset.
 * A shim that returned the same number in the first two cases would make the function
 * allocate one wchar_t too few and the copy would truncate — silently, and identically
 * on both sides. */
DWORD WINAPI GetEnvironmentVariableW(const wchar_t *name, wchar_t *buf, DWORD size)
{
    static const wchar_t comspec_name[] = { 'C','O','M','S','P','E','C',0 };
    unsigned n, i;

    if (!name) { SET_ERR(ERROR_INVALID_PARAMETER); return 0; }
    for (i = 0; comspec_name[i] || name[i]; i++)
        if (name[i] != comspec_name[i]) { SET_ERR(ERROR_ENVVAR_NOT_FOUND); return 0; }
    if (!comspec_set) { SET_ERR(ERROR_ENVVAR_NOT_FOUND); return 0; }

    n = wide_len(comspec_value);
    if (!buf || size < n + 1) { SET_ERR(ERROR_SUCCESS); return n + 1; }
    memcpy(buf, comspec_value, n * sizeof(wchar_t));
    buf[n] = 0;
    SET_ERR(ERROR_SUCCESS);
    return n;
}

/* ── the module table ───────────────────────────────────────────────────────
 *
 * `_loaddll` / `_unloaddll` / `_getdllprocaddr` do no work of their own beyond a UTF-8
 * conversion and an ordinal-vs-name decision, so the table is one fake module.  What
 * is graded is which of those calls happen and with what — recorded above. */
#define REF_PROC_HMODULE ((HMODULE)(LONG_PTR)0x70001000)
#define REF_PROC_PROC    ((void *)(LONG_PTR)0x70002000)

HMODULE WINAPI LoadLibraryW(const wchar_t *name)
{
    unsigned dummy_len;

    LEDGER.loadlibrary_calls++;
    record_wide(name, LEDGER.loadlibrary_name, MAX_PATH, &LEDGER.loadlibrary_null,
                &dummy_len);
    if (!loadlibrary_ok) { SET_ERR(ERROR_FILE_NOT_FOUND); return NULL; }
    SET_ERR(ERROR_SUCCESS);
    return REF_PROC_HMODULE;
}

BOOL WINAPI FreeLibrary(HMODULE mod)
{
    LEDGER.freelibrary_calls++;
    LEDGER.freelibrary_arg = mod;
    if (!freelibrary_ok) { SET_ERR(ERROR_INVALID_HANDLE); return FALSE; }
    SET_ERR(ERROR_SUCCESS);
    return TRUE;
}

void * WINAPI GetProcAddress(HMODULE mod, const char *name)
{
    ULONG_PTR raw = (ULONG_PTR)name;

    LEDGER.getprocaddr_calls++;
    LEDGER.getprocaddr_mod = mod;
    LEDGER.getprocaddr_raw = raw;
    /* An ORDINAL arrives here as a small integer cast to a pointer; a NAME arrives as a
     * real address.  Distinguishing them the way the real GetProcAddress does — by
     * HIWORD — is what lets a row tell "ordinal 7" from "the name at address 7", and
     * that distinction IS `_getdllprocaddr`'s body. */
    if (raw > 0xFFFF)
    {
        unsigned i = 0;
        LEDGER.getprocaddr_is_name = 1;
        while (i < sizeof(LEDGER.getprocaddr_name) - 1 && name[i])
        {
            LEDGER.getprocaddr_name[i] = name[i];
            i++;
        }
        LEDGER.getprocaddr_name[i] = 0;
    }
    else LEDGER.getprocaddr_is_name = 0;

    if (!getprocaddr_ok) { SET_ERR(ERROR_PROC_NOT_FOUND); return NULL; }
    SET_ERR(ERROR_SUCCESS);
    return REF_PROC_PROC;
}

long long ref_proc_hmodule(void) { return (long long)(LONG_PTR)REF_PROC_HMODULE; }
long long ref_proc_procaddr(void) { return (long long)(LONG_PTR)REF_PROC_PROC; }
