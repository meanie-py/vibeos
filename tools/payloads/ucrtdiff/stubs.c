/*
 * ucrtdiff link stubs — msvcrt internals the Rust bodies reach that ucrtbase does not
 * export, so no rename rule covers them and nothing on the link line defines them.
 * Rust codegen units bundle many functions, so these objects come in whether or not we
 * grade what pulls them.
 *
 * THEY LIVE IN THEIR OWN TRANSLATION UNIT, and that is the point: declared alongside
 * the real headers they collide, because stdio.h gives fwprintf a signature this would
 * contradict and conio.h defines _cprintf as a static inline (on arm64 that is a
 * same-mangled-name error, on i386 it silently was not). Compiled with NO CRT headers,
 * each definition simply emits the symbol the linker is asking for, with whatever
 * decoration the target uses -- no asm labels, no per-arch spelling.
 *
 * Stubbing beats linking libmsvcrt_static.a, which defines strtol and friends too and
 * would be allowed to answer a GRADED name silently, turning the harness into a
 * comparison of msvcrt against Microsoft while reporting it as our shipping DLL.
 *
 * Nothing here is reachable from any graded family. If that changes, the run says so
 * and stops rather than reporting a number built on a stub.
 */

unsigned long __stdcall GetCurrentProcess(void);
int __stdcall TerminateProcess(void *h, unsigned int code);
unsigned long __cdecl DbgPrint(const char *fmt, ...);

static void unreachable(const char *who)
{
    DbgPrint("[UCRTDIFF] FATAL: a graded body reached %s, which this harness stubs\n", who);
    TerminateProcess((void *)(unsigned long long)GetCurrentProcess(), 2);
}

int __cdecl _cprintf(const char *fmt, ...);
int __cdecl _cprintf(const char *fmt, ...)
{
    (void)fmt;
    unreachable("_cprintf");
    return 0;
}

int __cdecl _scprintf(const char *fmt, ...);
int __cdecl _scprintf(const char *fmt, ...)
{
    (void)fmt;
    unreachable("_scprintf");
    return 0;
}

int __cdecl fwprintf(void *stream, const unsigned short *fmt, ...);
int __cdecl fwprintf(void *stream, const unsigned short *fmt, ...)
{
    (void)stream;
    (void)fmt;
    unreachable("fwprintf");
    return 0;
}

/* i386-only: msvcrt's C++ EH helpers, which live in libmsvcrt_static.a. */
void __cdecl call_copy_ctor(void *a, void *b, void *c, int d);
void __cdecl call_copy_ctor(void *a, void *b, void *c, int d)
{
    (void)a; (void)b; (void)c; (void)d;
    unreachable("call_copy_ctor");
}

void __cdecl call_dtor(void *a, void *b);
void __cdecl call_dtor(void *a, void *b)
{
    (void)a; (void)b;
    unreachable("call_dtor");
}

void __stdcall __wine_RtlUnwind(void *a, void *b, void *c, void *d);
void __stdcall __wine_RtlUnwind(void *a, void *b, void *c, void *d)
{
    (void)a; (void)b; (void)c; (void)d;
    unreachable("__wine_RtlUnwind");
}
