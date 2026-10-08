/* fpctrl — ucrtbase's FP control/status family against msvcrt.dll's (arm64, amd64).
 *
 * Until 2026-10-03 arm64 ucrtbase had no body for _clearfp/_control87/_controlfp/
 * _statusfp and shipped forwards into msvcrt.dll; ucrt-rust's fpctrl_arm64.rs replaced
 * them. msvcrt.dll's bodies -- the ones the forward reached -- are the oracle: both DLLs
 * are loaded here and every case runs once through each, from the same FPCR/FPSR state,
 * with the results and the hardware state afterwards compared. amd64 runs the same cases
 * since ucrtbase's MXCSR family moved from sdk/lib/crt's C into ucrt-rust (fpctrl_amd64.rs).
 *
 * The hardware is shared, so each side's effect is read back through the OTHER DLL's
 * _control87(0, 0) as well as its own return value; a body that wrote a different FPCR
 * shows up even when it reports the same word. FP operations only run under the
 * initial (all-masked) control word, so no case can trap.
 */
#include <windows.h>

ULONG __cdecl DbgPrint(const char *Format, ...);
#define SAY(...) DbgPrint("[FPCTRL] " __VA_ARGS__)

typedef unsigned int(__cdecl *ctl_t)(unsigned int, unsigned int);
typedef unsigned int(__cdecl *st_t)(void);
typedef void(__cdecl *setctl_t)(unsigned int, unsigned int);
typedef int(__cdecl *ctls_t)(unsigned int *, unsigned int, unsigned int);

struct side {
    const char *name;
    ctl_t control87, controlfp;
    st_t statusfp, clearfp;
    setctl_t set_controlfp;
    ctls_t controlfp_s;
};

static int g_fail, g_cases;
static unsigned int g_init;
static struct side U, M;

static void check(const char *what, unsigned int a, unsigned int b)
{
    g_cases++;
    if (a != b) {
        g_fail++;
        SAY("DIFF %s ucrtbase=%08x msvcrt=%08x\n", what, a, b);
    }
}

static int load(struct side *s, const char *dll)
{
    HMODULE h = LoadLibraryA(dll);
    s->name = dll;
    if (!h) {
        SAY("FAIL LoadLibrary(%s)\n", dll);
        return 0;
    }
    s->control87 = (ctl_t)(void *)GetProcAddress(h, "_control87");
    s->controlfp = (ctl_t)(void *)GetProcAddress(h, "_controlfp");
    s->statusfp = (st_t)(void *)GetProcAddress(h, "_statusfp");
    s->clearfp = (st_t)(void *)GetProcAddress(h, "_clearfp");
    s->set_controlfp = (setctl_t)(void *)GetProcAddress(h, "_set_controlfp");
    s->controlfp_s = (ctls_t)(void *)GetProcAddress(h, "_controlfp_s");
    if (!s->control87 || !s->controlfp || !s->statusfp || !s->clearfp || !s->set_controlfp ||
        !s->controlfp_s) {
        SAY("FAIL %s lacks an export\n", dll);
        return 0;
    }
    return 1;
}

/* Back to the initial control word with no flags raised, through msvcrt (the oracle). */
static void reset(void)
{
    M.control87(g_init, 0xffffffff);
    M.clearfp();
}

static volatile double g_zero = 0.0, g_one = 1.0, g_big = 1e308, g_tiny = 1e-308, g_three = 3.0;
static volatile double g_sink;

/* Raise a set of status flags, under the all-masked initial control word. */
static void raise_flags(int which)
{
    if (which & 1)
        g_sink = g_one / g_zero; /* divide by zero */
    if (which & 2)
        g_sink = g_zero / g_zero; /* invalid */
    if (which & 4)
        g_sink = g_big * g_big; /* overflow + inexact */
    if (which & 8)
        g_sink = g_tiny / g_big; /* underflow + inexact */
    if (which & 16)
        g_sink = g_one / g_three; /* inexact */
}

static const unsigned int k_masks[] = {
    0, 0xffffffff, 0x0008001f /* _MCW_EM */, 0x00000300 /* _MCW_RC */,
    0x00030000 /* _MCW_PC */, 0x03000000 /* _MCW_DN */, 0x00080000 /* _EM_DENORMAL */,
    0x00000010, 0x00000008, 0x00000004, 0x00000002, 0x00000001, 0x0008031f,
};
static const unsigned int k_vals[] = {
    0, 0xffffffff, 0x0008001f, 0x00000100, 0x00000200, 0x00000300, 0x00000010,
    0x00000001, 0x00080000, 0x0008021f, 0x00000315,
};

static void control_cases(const char *fn, int which)
{
    char what[96];
    for (unsigned i = 0; i < sizeof k_vals / sizeof *k_vals; i++)
        for (unsigned j = 0; j < sizeof k_masks / sizeof *k_masks; j++) {
            unsigned int v = k_vals[i], m = k_masks[j], ru, rm, hu, hm;
            reset();
            ru = which ? U.controlfp(v, m) : U.control87(v, m);
            hu = M.control87(0, 0);
            reset();
            rm = which ? M.controlfp(v, m) : M.control87(v, m);
            hm = U.control87(0, 0);
            wsprintfA(what, "%s(%08x,%08x)", fn, v, m);
            check(what, ru, rm);
            wsprintfA(what, "%s(%08x,%08x).after", fn, v, m);
            check(what, hu, hm);
        }
    reset();
}

static void status_cases(void)
{
    char what[64];
    for (int w = 0; w < 32; w++) {
        unsigned int su, sm, cu, cm, au, am;
        reset();
        raise_flags(w);
        su = U.statusfp();
        cu = U.clearfp();
        au = M.statusfp();
        reset();
        raise_flags(w);
        sm = M.statusfp();
        cm = M.clearfp();
        am = U.statusfp();
        wsprintfA(what, "_statusfp(raise %02x)", w);
        check(what, su, sm);
        wsprintfA(what, "_clearfp(raise %02x)", w);
        check(what, cu, cm);
        wsprintfA(what, "_clearfp(raise %02x).after", w);
        check(what, au, am);
    }
    reset();
}

static void misc_cases(void)
{
    unsigned int cu = 0x1234, cm = 0x1234;
    int ru, rm;
    reset();
    U.set_controlfp(0x00000200, 0x00000300);
    cu = M.control87(0, 0);
    reset();
    M.set_controlfp(0x00000200, 0x00000300);
    cm = U.control87(0, 0);
    check("_set_controlfp(RC_UP).after", cu, cm);

    /* _controlfp_s: ucrtbase's is sdk/lib/crt's C over _controlfp, so this grades the
     * new body through a caller as well. (No out-of-range request: both sides route
     * that to the invalid-parameter handler, which ends the process.) */
    reset();
    ru = U.controlfp_s(&cu, 0x00000100, 0x00000300);
    reset();
    rm = M.controlfp_s(&cm, 0x00000100, 0x00000300);
    check("_controlfp_s(RC_DOWN).ret", (unsigned)ru, (unsigned)rm);
    check("_controlfp_s(RC_DOWN).cur", cu, cm);
    reset();
}

int __cdecl main(void)
{
    if (!load(&U, "ucrtbase.dll") || !load(&M, "msvcrt.dll")) {
        SAY("done FAIL (load)\n");
        return 1;
    }
    if ((void *)U.control87 == (void *)M.control87) {
        /* The forward this payload exists to retire: both names resolve to one body. */
        SAY("FAIL ucrtbase!_control87 IS msvcrt!_control87 -- still forwarded\n");
        g_fail++;
    }
    g_init = M.control87(0, 0);
    SAY("initial control word %08x\n", g_init);
    control_cases("_control87", 0);
    control_cases("_controlfp", 1);
    status_cases();
    misc_cases();
    SAY("%d cases, %d diffs\n", g_cases, g_fail);
    SAY("done %s\n", g_fail ? "FAIL" : "PASS");
    return g_fail != 0;
}
