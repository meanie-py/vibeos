/* dispprobe — does DispCallFunc marshal arguments and returns correctly on this arch?
 *
 * oleaut32's DispCallFunc is Rust over per-arch global_asm! trampolines transcribed from Wine
 * (upstream's asm thunks.c was declined on the assumption they replace it). Each case calls a
 * known function through DispCallFunc and checks the result: arg counts past every arch's
 * register window (4 on x64, 8 on arm64), int/float interleaving (x64 mirrors slots into both
 * GPRs and XMMs; arm64 has separate x/d banks), 64-bit ints on i386, each return type, and a
 * vtable call with a This pointer.
 * Verdict: "[DISPPROBE] done VERDICT=PASS|FAIL". */
#define COBJMACROS
#include <windows.h>
#include <oleauto.h>

ULONG __cdecl DbgPrint(const char *Format, ...);
#define SAY(...) DbgPrint("[DISPPROBE] " __VA_ARGS__)

static int failures;

static LONG WINAPI sum0(void) { return 42; }
static LONG WINAPI sum3(LONG a, LONG b, LONG c) { return a + 10 * b + 100 * c; }
static LONG WINAPI sum10(LONG a, LONG b, LONG c, LONG d, LONG e, LONG f, LONG g, LONG h, LONG i, LONG j)
{
    return a + 2 * b + 3 * c + 4 * d + 5 * e + 6 * f + 7 * g + 8 * h + 9 * i + 10 * j;
}
static double WINAPI mixd(LONG a, double b, LONG c, double d, float e, LONG f, double g, LONG h, double i,
                          LONG j, double k, float l)
{
    return a + b * 2 + c * 3 + d * 4 + e * 5 + f * 6 + g * 7 + h * 8 + i * 9 + j * 10 + k * 11 + l * 12;
}
static float WINAPI retf(float a, LONG b) { return a * (float)b; }
static LONGLONG WINAPI ret64(LONGLONG a, LONG b, LONGLONG c) { return a * b + c; }
static LONG __cdecl cdecl4(LONG a, LONG b, LONG c, LONG d) { return a - b - c - d; }

/* A one-method object: vtable slot 3 (after the IUnknown three). */
typedef struct Obj { const void *const *vtbl; LONG base; } Obj;
static LONG WINAPI obj_method(Obj *This, LONG x) { return This->base + x; }
static const void *const obj_vtbl[] = { NULL, NULL, NULL, (const void *)obj_method };

static void set_i4(VARIANT *v, LONG x) { VariantInit(v); V_VT(v) = VT_I4; V_I4(v) = x; }
static void set_r8(VARIANT *v, double x) { VariantInit(v); V_VT(v) = VT_R8; V_R8(v) = x; }
static void set_r4(VARIANT *v, float x) { VariantInit(v); V_VT(v) = VT_R4; V_R4(v) = x; }
static void set_i8(VARIANT *v, LONGLONG x) { VariantInit(v); V_VT(v) = VT_I8; V_I8(v) = x; }

static void call(const char *name, void *inst, ULONG_PTR fn, CALLCONV cc, VARTYPE ret, UINT n, VARIANT *args,
                 VARIANT *result)
{
    VARTYPE vts[16];
    VARIANTARG *ptrs[16];
    for (UINT i = 0; i < n; i++) {
        vts[i] = V_VT(&args[i]);
        ptrs[i] = &args[i];
    }
    VariantInit(result);
    HRESULT hr = DispCallFunc(inst, fn, cc, ret, n, vts, ptrs, result);
    if (FAILED(hr)) {
        SAY("FAIL %s: hr=0x%08lx\n", name, hr);
        failures++;
    }
}

#define CHECK(name, cond, fmt, ...) do { \
    if (cond) SAY("ok   %s\n", name); \
    else { SAY("FAIL %s: " fmt "\n", name, __VA_ARGS__); failures++; } } while (0)

int main(void)
{
    VARIANT a[16], r;

    call("sum0", NULL, (ULONG_PTR)sum0, CC_STDCALL, VT_I4, 0, a, &r);
    CHECK("sum0", V_VT(&r) == VT_I4 && V_I4(&r) == 42, "got vt=%u %ld", V_VT(&r), V_I4(&r));

    set_i4(&a[0], 1); set_i4(&a[1], 2); set_i4(&a[2], 3);
    call("sum3", NULL, (ULONG_PTR)sum3, CC_STDCALL, VT_I4, 3, a, &r);
    CHECK("sum3", V_I4(&r) == 321, "got %ld", V_I4(&r));

    for (int i = 0; i < 10; i++) set_i4(&a[i], i + 1);
    call("sum10", NULL, (ULONG_PTR)sum10, CC_STDCALL, VT_I4, 10, a, &r);
    CHECK("sum10", V_I4(&r) == sum10(1, 2, 3, 4, 5, 6, 7, 8, 9, 10), "got %ld", V_I4(&r));

    set_i4(&a[0], 1); set_r8(&a[1], 2.5); set_i4(&a[2], 3); set_r8(&a[3], 4.25); set_r4(&a[4], 5.5f);
    set_i4(&a[5], 6); set_r8(&a[6], 7.125); set_i4(&a[7], 8); set_r8(&a[8], 9.5); set_i4(&a[9], 10);
    set_r8(&a[10], 11.75); set_r4(&a[11], 12.5f);
    call("mixd", NULL, (ULONG_PTR)mixd, CC_STDCALL, VT_R8, 12, a, &r);
    {
        double want = mixd(1, 2.5, 3, 4.25, 5.5f, 6, 7.125, 8, 9.5, 10, 11.75, 12.5f);
        CHECK("mixd", V_VT(&r) == VT_R8 && V_R8(&r) == want, "got %d.%03d want %d.%03d",
              (int)V_R8(&r), (int)((V_R8(&r) - (int)V_R8(&r)) * 1000), (int)want,
              (int)((want - (int)want) * 1000));
    }

    set_r4(&a[0], 1.5f); set_i4(&a[1], 4);
    call("retf", NULL, (ULONG_PTR)retf, CC_STDCALL, VT_R4, 2, a, &r);
    CHECK("retf", V_VT(&r) == VT_R4 && V_R4(&r) == 6.0f, "got vt=%u x1000=%d", V_VT(&r), (int)(V_R4(&r) * 1000));

    set_i8(&a[0], 0x100000001LL); set_i4(&a[1], 3); set_i8(&a[2], 0x7000000000LL);
    call("ret64", NULL, (ULONG_PTR)ret64, CC_STDCALL, VT_I8, 3, a, &r);
    CHECK("ret64", V_VT(&r) == VT_I8 && V_I8(&r) == 0x100000001LL * 3 + 0x7000000000LL, "got 0x%08lx%08lx",
          (ULONG)(V_I8(&r) >> 32), (ULONG)V_I8(&r));

    set_i4(&a[0], 100); set_i4(&a[1], 1); set_i4(&a[2], 2); set_i4(&a[3], 3);
    call("cdecl4", NULL, (ULONG_PTR)cdecl4, CC_CDECL, VT_I4, 4, a, &r);
    CHECK("cdecl4", V_I4(&r) == 94, "got %ld", V_I4(&r));

    {
        Obj o = { obj_vtbl, 1000 };
        set_i4(&a[0], 7);
        call("vtable", &o, 3 * sizeof(void *), CC_STDCALL, VT_I4, 1, a, &r);
        CHECK("vtable", V_I4(&r) == 1007, "got %ld", V_I4(&r));
    }

    SAY("done VERDICT=%s (%d failures)\n", failures ? "FAIL" : "PASS", failures);
    return failures ? 1 : 0;
}
