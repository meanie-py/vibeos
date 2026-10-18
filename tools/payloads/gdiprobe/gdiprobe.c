/* gdiprobe — is a memory DC's world transform identity through dwrite's render-target setup?
 *
 * dwrite's bitmap render target does CreateCompatibleDC, SetGraphicsMode(GM_ADVANCED), selects
 * a 32bpp top-down DIB section (or, at zero width, a 1x1 1bpp bitmap), and expects
 * GetWorldTransform on that DC to read identity. font.c:1397 reads something else here.
 * Each step prints the transform's raw float bits, so a wrong value is visible, not just "bad".
 * Verdict: "[GDIPROBE] done VERDICT=PASS|FAIL". */
#define _WIN32_WINNT 0x0601
#include <windows.h>

ULONG __cdecl DbgPrint(const char *Format, ...);

/* DbgPrint for the guest's serial; stdout too, for `vibeos-cli winref run` on Windows. */
static void SAY(const char *Format, ...)
{
    char line[512];
    va_list args;
    DWORD n;
    HANDLE out = GetStdHandle(STD_OUTPUT_HANDLE);
    int len = lstrlenA(lstrcpyA(line, "[GDIPROBE] "));
    va_start(args, Format);
    wvsprintfA(line + len, Format, args);
    va_end(args);
    DbgPrint("%s", line);
    if (out && out != INVALID_HANDLE_VALUE)
        WriteFile(out, line, lstrlenA(line), &n, NULL);
}

static int failures;

static DWORD bits(FLOAT f)
{
    DWORD d;
    CopyMemory(&d, &f, sizeof(d));
    return d;
}

static void check(const char *step, HDC hdc)
{
    XFORM x;
    BOOL ok;
    memset(&x, 0xcc, sizeof(x));
    ok = GetWorldTransform(hdc, &x);
    SAY("%s: ret=%d mode=%d M11=%08lx M12=%08lx M21=%08lx M22=%08lx Dx=%08lx Dy=%08lx\n", step, ok,
        GetGraphicsMode(hdc), bits(x.eM11), bits(x.eM12), bits(x.eM21), bits(x.eM22), bits(x.eDx), bits(x.eDy));
    if (!ok || x.eM11 != 1.0f || x.eM22 != 1.0f || x.eM12 != 0.0f || x.eM21 != 0.0f || x.eDx != 0.0f || x.eDy != 0.0f)
    {
        SAY("FAIL %s: not identity\n", step);
        failures++;
    }
}

static HBITMAP dib(HDC hdc, int width, int height)
{
    char buf[sizeof(BITMAPINFOHEADER) + 256 * sizeof(RGBQUAD)];
    BITMAPINFO *bmi = (BITMAPINFO *)buf;
    void *bits;
    memset(buf, 0, sizeof(buf));
    bmi->bmiHeader.biSize = sizeof(bmi->bmiHeader);
    bmi->bmiHeader.biWidth = width;
    bmi->bmiHeader.biHeight = -height;
    bmi->bmiHeader.biBitCount = 32;
    bmi->bmiHeader.biPlanes = 1;
    bmi->bmiHeader.biCompression = BI_RGB;
    return CreateDIBSection(hdc, bmi, DIB_RGB_COLORS, &bits, NULL, 0);
}

int main(void)
{
    HDC screen = CreateCompatibleDC(0), hdc;
    HBITMAP hbm, old;
    XFORM ident = {1.0f, 0.0f, 0.0f, 1.0f, 0.0f, 0.0f};

    check("CreateCompatibleDC(0)", screen);
    hdc = CreateCompatibleDC(screen);
    check("CreateCompatibleDC(memdc)", hdc);
    SetGraphicsMode(hdc, GM_ADVANCED);
    check("GM_ADVANCED", hdc);
    hbm = dib(hdc, 10, 5);
    old = SelectObject(hdc, hbm);
    check("select 10x5 DIB", hdc);
    DeleteObject(SelectObject(hdc, dib(hdc, 5, 5)));
    check("select 5x5 DIB", hdc);
    hbm = dib(hdc, 0, 5);
    SAY("CreateDIBSection(0x5) = %p\n", hbm);
    if (!hbm)
        hbm = CreateBitmap(1, 1, 1, 1, NULL);
    DeleteObject(SelectObject(hdc, hbm));
    check("select fallback bitmap", hdc);
    SetWorldTransform(hdc, &ident);
    check("SetWorldTransform(identity)", hdc);
    SelectObject(hdc, old);
    DeleteObject(hbm);
    DeleteDC(hdc);
    DeleteDC(screen);
    SAY("done VERDICT=%s (%d failures)\n", failures ? "FAIL" : "PASS", failures);
    return failures;
}
