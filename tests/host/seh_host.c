/*
 * seh_host.c — PSEH replacement for host-side testing
 *
 * Uses setjmp/longjmp + POSIX signals to catch access violations
 * (SIGSEGV/SIGBUS) and map them to STATUS_ACCESS_VIOLATION.
 *
 * Supports nested TRY/EXCEPT/FINALLY blocks via a per-thread
 * jmp_buf pointer chain (each _SEH2_TRY pushes a local jmp_buf).
 *
 * Uses sigaltstack so the signal handler can survive stack overflow.
 */

#include "include/nt_host.h"

__thread jmp_buf *_seh_current_jmpbuf = NULL;
__thread volatile NTSTATUS _seh_code = 0;

static struct sigaction _seh_old_sigsegv;
static struct sigaction _seh_old_sigbus;
static int _seh_altstack_initialized = 0;

void _seh_signal_handler(int sig) {
    _seh_code = STATUS_ACCESS_VIOLATION;
    if (_seh_current_jmpbuf) {
        longjmp(*_seh_current_jmpbuf, 1);
    }
    /* No handler installed — re-raise with default */
    signal(sig, SIG_DFL);
    raise(sig);
}

void _seh_install(void) {
    /* Set up alternate signal stack (once) so we can handle stack overflow */
    if (!_seh_altstack_initialized) {
        stack_t ss;
        ss.ss_sp = malloc(SIGSTKSZ);
        ss.ss_size = SIGSTKSZ;
        ss.ss_flags = 0;
        if (ss.ss_sp) {
            sigaltstack(&ss, NULL);
        }
        _seh_altstack_initialized = 1;
    }

    struct sigaction sa;
    sa.sa_handler = _seh_signal_handler;
    sigemptyset(&sa.sa_mask);
    sa.sa_flags = SA_NODEFER | SA_ONSTACK;  /* NODEFER: allow longjmp; ONSTACK: survive stack overflow */
    sigaction(SIGSEGV, &sa, &_seh_old_sigsegv);
    sigaction(SIGBUS, &sa, &_seh_old_sigbus);
}

void _seh_uninstall(void) {
    sigaction(SIGSEGV, &_seh_old_sigsegv, NULL);
    sigaction(SIGBUS, &_seh_old_sigbus, NULL);
}

/*
 * ExRaiseStatus — software exception via longjmp.
 * Non-inline to ensure longjmp works correctly across stack frames.
 */
void _ExRaiseStatus(NTSTATUS Status) {
    _seh_code = Status;
    if (_seh_current_jmpbuf) {
        longjmp(*_seh_current_jmpbuf, 1);
    }
    fprintf(stderr, "Unhandled ExRaiseStatus: 0x%08x\n", (unsigned)Status);
    abort();
}
