/* wsprintf -- user32's wsprintfA/W are Rust naked thunks over a va_list engine
 * (user32-wsprintf-rust). This checks the thunks, which no host test can reach: argument
 * counts that cross from registers onto the stack on each ABI (amd64 passes 2 variadic
 * arguments in r8/r9, arm64 6 in x2..x7), mixed int / pointer / __int64 slots, the return
 * value, and the user32 configuration's own rules (`*` printed, `#x` widened, no floats). */
#include <windows.h>
#include <string.h>

ULONG __cdecl DbgPrint(const char *Format, ...);
#define SAY(...) DbgPrint("[WSPRINTF] " __VA_ARGS__)
static int g_fail, g_ok;

static void checkA(int line, const char *got, int ret, const char *want)
{
    if (!strcmp(got, want) && ret == (int)strlen(want)) { g_ok++; return; }
    g_fail++;
    SAY("FAIL line %d: got \"%s\" (ret %d), want \"%s\"\n", line, got, ret, want);
}

static void checkW(int line, const WCHAR *got, int ret, const WCHAR *want)
{
    if (!lstrcmpW(got, want) && ret == lstrlenW(want)) { g_ok++; return; }
    g_fail++;
    SAY("FAIL line %d: got \"%ls\" (ret %d), want \"%ls\"\n", line, got, ret, want);
}

int WINAPI WinMain(HINSTANCE a, HINSTANCE b, LPSTR c, int d)
{
    char buf[512];
    WCHAR wbuf[512];
    int r;
    (void)a; (void)b; (void)c; (void)d;

    r = wsprintfA(buf, "plain");
    checkA(__LINE__, buf, r, "plain");
    r = wsprintfA(buf, "%d", 42);
    checkA(__LINE__, buf, r, "42");
    r = wsprintfA(buf, "%d %d", -1, 2);
    checkA(__LINE__, buf, r, "-1 2");
    /* 11 variadic arguments: past r8/r9 on amd64 and x2..x7 on arm64 */
    r = wsprintfA(buf, "%d %d %d %d %d %d %d %d %d %s %I64d", 1, 2, 3, 4, 5, 6, 7, 8, 9, "str",
                  (__int64)0x123456789);
    checkA(__LINE__, buf, r, "1 2 3 4 5 6 7 8 9 str 4886718345");
    /* an __int64 right at the register/stack boundary on each ABI */
    r = wsprintfA(buf, "%I64x|%d|%I64x", (__int64)0x1122334455667788, 7, (__int64)-1);
    checkA(__LINE__, buf, r, "1122334455667788|7|ffffffffffffffff");
    r = wsprintfA(buf, "%d|%d|%d|%d|%d|%d|%I64d|%s", 1, 2, 3, 4, 5, 6, (__int64)-5, "end");
    checkA(__LINE__, buf, r, "1|2|3|4|5|6|-5|end");
    r = wsprintfA(buf, "%c%c%c|%5s|%-5s|%.2s|%u|%X", 'a', 'b', 'c', "ab", "cd", "xyz", 4000000000u, 0xabcd);
    checkA(__LINE__, buf, r, "abc|   ab|cd   |xy|4000000000|ABCD");
    /* the user32 configuration */
    r = wsprintfA(buf, "%*d", 5, 42);
    checkA(__LINE__, buf, r, "*d");
    r = wsprintfA(buf, "[%#6x]", 0x2a);
    checkA(__LINE__, buf, r, "[    0x2a]");
    r = wsprintfA(buf, "%f", 1.5);
    checkA(__LINE__, buf, r, "f");

    r = wsprintfW(wbuf, L"%d %d %d %d %d %d %d %d %d %s %I64d", 1, 2, 3, 4, 5, 6, 7, 8, 9, L"wide",
                  (__int64)0x123456789);
    checkW(__LINE__, wbuf, r, L"1 2 3 4 5 6 7 8 9 wide 4886718345");
    r = wsprintfW(wbuf, L"%hs|%c|%x", "narrow", L'Z', 0xff);
    checkW(__LINE__, wbuf, r, L"narrow|Z|ff");

    SAY("%d checks, %d failed\n", g_ok + g_fail, g_fail);
    SAY("done %s\n", g_fail ? "FAIL" : "PASS");
    return g_fail != 0;
}
