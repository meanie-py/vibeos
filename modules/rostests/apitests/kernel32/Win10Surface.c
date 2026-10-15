/*
 * PROJECT:     ReactOS api tests
 * LICENSE:     LGPL-2.0-or-later (https://spdx.org/licenses/LGPL-2.0-or-later)
 * PURPOSE:     Windows 10 surface that apps import: DLL directories, GetProductInfo,
 *              PathCch*, IDN, module enumeration, NtGetNextThread, transactions in the TEB,
 *              UTF-8 conversion, clipboard listeners, InitializeContext, package identity,
 *              WSAPoll/GetAddrInfoEx, iphlpapi tables, shcore DPI, and the setupapi /
 *              cfgmgr32 device property getters. Everything beyond kernel32/user32/gdi32/ntdll
 *              is looked up at run time so the same binary grades Windows 11.
 */
#include "precomp.h"
#include <winuser.h>
#include <wingdi.h>

#ifndef LOAD_LIBRARY_SEARCH_APPLICATION_DIR
#define LOAD_LIBRARY_SEARCH_APPLICATION_DIR 0x00000200
#endif
#ifndef LOAD_LIBRARY_SEARCH_USER_DIRS
#define LOAD_LIBRARY_SEARCH_USER_DIRS       0x00000400
#endif
#ifndef LOAD_LIBRARY_SEARCH_SYSTEM32
#define LOAD_LIBRARY_SEARCH_SYSTEM32        0x00000800
#endif
#ifndef LOAD_LIBRARY_SEARCH_DEFAULT_DIRS
#define LOAD_LIBRARY_SEARCH_DEFAULT_DIRS    0x00001000
#endif
#ifndef LOAD_WITH_ALTERED_SEARCH_PATH
#define LOAD_WITH_ALTERED_SEARCH_PATH       0x00000008
#endif
#ifndef PRODUCT_UNDEFINED
#define PRODUCT_UNDEFINED 0
#endif
#ifndef ERROR_INVALID_FLAGS
#define ERROR_INVALID_FLAGS 1004
#endif
#ifndef WM_CLIPBOARDUPDATE
#define WM_CLIPBOARDUPDATE 0x031D
#endif
#define W10_LIST_MODULES_ALL    3
#define W10_LIST_MODULES_32BIT  1
#define W10_STATUS_NO_MORE_ENTRIES ((NTSTATUS)0x8000001A)
#define W10_APPMODEL_ERROR_NO_PACKAGE 15700
#define W10_HR_INSUFFICIENT_BUFFER ((HRESULT)0x8007007A)

typedef struct { GUID fmtid; ULONG pid; } W10_DEVPROPKEY;
typedef struct { DWORD cbSize; GUID ClassGuid; DWORD DevInst; ULONG_PTR Reserved; } W10_SP_DEVINFO_DATA;

static const W10_DEVPROPKEY W10_Key_InstanceId =
    { { 0x78c34fc8, 0x104a, 0x4aca, { 0x9e, 0xa4, 0x52, 0x4d, 0x52, 0x99, 0x6e, 0x57 } }, 256 };
static const W10_DEVPROPKEY W10_Key_HardwareIds =
    { { 0xa45c254e, 0xdf1c, 0x4efd, { 0x80, 0x20, 0x67, 0xd1, 0x46, 0xa8, 0x50, 0xe0 } }, 3 };

static FARPROC GetK32(LPCSTR Name)
{
    return GetProcAddress(GetModuleHandleW(L"kernel32.dll"), Name);
}

static FARPROC GetNt(LPCSTR Name)
{
    return GetProcAddress(GetModuleHandleW(L"ntdll.dll"), Name);
}

static FARPROC GetDyn(LPCWSTR Dll, LPCSTR Name)
{
    HMODULE Mod = LoadLibraryW(Dll);
    if (!Mod)
    {
        win_skip("%S not available (error %lu)\n", Dll, GetLastError());
        return NULL;
    }
    return GetProcAddress(Mod, Name);
}

/* ---- 2 ---- */
static void Test_GetProductInfo(void)
{
    BOOL (WINAPI *pGetProductInfo)(DWORD, DWORD, DWORD, DWORD, PDWORD) = (void *)GetK32("GetProductInfo");
    DWORD Type;

    if (!pGetProductInfo)
    {
        win_skip("GetProductInfo missing\n");
        return;
    }
    Type = 0xdeadbeef;
    ok(pGetProductInfo(10, 0, 0, 0, &Type), "GetProductInfo(10,0) failed: %lu\n", GetLastError());
    ok(Type != PRODUCT_UNDEFINED, "product type for 10.0 is undefined\n");
    Type = 0xdeadbeef;
    ok(!pGetProductInfo(5, 1, 0, 0, &Type), "GetProductInfo(5,1) succeeded\n");
    ok(Type == PRODUCT_UNDEFINED, "product type for 5.1 is %lx\n", Type);
}

/* ---- 3 ---- */
static void Test_PathCch(void)
{
    HRESULT (WINAPI *pCombine)(PWSTR, SIZE_T, PCWSTR, PCWSTR);
    HRESULT (WINAPI *pRemoveFileSpec)(PWSTR, SIZE_T);
    HRESULT (WINAPI *pSkipRoot)(PCWSTR, PCWSTR *);
    HRESULT (WINAPI *pCombineEx)(PWSTR, SIZE_T, PCWSTR, PCWSTR, ULONG);
    WCHAR Buf[MAX_PATH];
    PCWSTR Rest = NULL;
    HRESULT hr;

    pCombine = (void *)GetDyn(L"api-ms-win-core-path-l1-1-0.dll", "PathCchCombine");
    if (!pCombine)
    {
        win_skip("PathCch API set missing\n");
        return;
    }
    pRemoveFileSpec = (void *)GetProcAddress(GetModuleHandleW(L"api-ms-win-core-path-l1-1-0.dll"), "PathCchRemoveFileSpec");
    pSkipRoot = (void *)GetProcAddress(GetModuleHandleW(L"api-ms-win-core-path-l1-1-0.dll"), "PathCchSkipRoot");
    pCombineEx = (void *)GetProcAddress(GetModuleHandleW(L"api-ms-win-core-path-l1-1-0.dll"), "PathCchCombineEx");

    hr = pCombine(Buf, MAX_PATH, L"C:\\a", L"b\\..\\c");
    ok(hr == S_OK, "PathCchCombine: %08lx\n", hr);
    ok(!lstrcmpW(Buf, L"C:\\a\\c"), "PathCchCombine result %S\n", Buf);

    if (pRemoveFileSpec)
    {
        lstrcpyW(Buf, L"C:\\a\\b.txt");
        hr = pRemoveFileSpec(Buf, MAX_PATH);
        ok(hr == S_OK, "PathCchRemoveFileSpec: %08lx\n", hr);
        ok(!lstrcmpW(Buf, L"C:\\a"), "PathCchRemoveFileSpec result %S\n", Buf);
    }
    else
        win_skip("PathCchRemoveFileSpec missing\n");

    if (pSkipRoot)
    {
        static const WCHAR Path[] = L"C:\\a\\b";
        hr = pSkipRoot(Path, &Rest);
        ok(hr == S_OK, "PathCchSkipRoot: %08lx\n", hr);
        ok(Rest == Path + 3 && !lstrcmpW(Rest, L"a\\b"), "PathCchSkipRoot rest %p (base %p)\n", Rest, Path);
    }
    else
        win_skip("PathCchSkipRoot missing\n");

    if (pCombineEx)
    {
        Buf[0] = 0;
        hr = pCombineEx(Buf, 4, L"C:\\a", L"b", 0);
        ok(hr == W10_HR_INSUFFICIENT_BUFFER, "PathCchCombineEx small buffer: %08lx\n", hr);
    }
    else
        win_skip("PathCchCombineEx missing\n");
}

/* ---- 4 ---- */
static void Test_Idn(void)
{
    int (WINAPI *pToAscii)(DWORD, LPCWSTR, int, LPWSTR, int) = (void *)GetK32("IdnToAscii");
    int (WINAPI *pToUnicode)(DWORD, LPCWSTR, int, LPWSTR, int) = (void *)GetK32("IdnToUnicode");
    WCHAR Buf[64];
    int Len;

    if (!pToAscii || !pToUnicode)
    {
        win_skip("IdnToAscii/IdnToUnicode missing\n");
        return;
    }
    Len = pToAscii(0, L"Test", -1, Buf, 64);
    ok(Len == 5 && !lstrcmpW(Buf, L"Test"), "IdnToAscii(Test): %d %S\n", Len, Buf);
    Len = pToAscii(0, L"Te\x017cst", -1, Buf, 64);
    ok(Len == 13 && !lstrcmpW(Buf, L"xn--test-cbb"), "IdnToAscii(Te\\x17cst): %d %S\n", Len, Buf);
    Len = pToAscii(0, L"Te\x017cst", -1, NULL, 0);
    ok(Len == 13, "IdnToAscii size query: %d\n", Len);
    Len = pToUnicode(0, L"xn--test-cbb", -1, Buf, 64);
    /* Nameprep lowercased the label on the way out, so it decodes lowercase. */
    ok(Len == 6 && !lstrcmpW(Buf, L"te\x017cst"), "IdnToUnicode: %d %S\n", Len, Buf);
    Len = pToUnicode(0, L"xn--test-cbb", -1, NULL, 0);
    ok(Len == 6, "IdnToUnicode size query: %d\n", Len);
}

/* ---- 5 ---- */
static void Test_EnumProcessModulesEx(void)
{
    BOOL (WINAPI *pEnum)(HANDLE, HMODULE *, DWORD, LPDWORD, DWORD) = (void *)GetK32("K32EnumProcessModulesEx");
    HMODULE Mods[256];
    DWORD Needed;
    BOOL Ret;

    if (!pEnum)
    {
        win_skip("K32EnumProcessModulesEx missing\n");
        return;
    }
    Needed = 0;
    Ret = pEnum(GetCurrentProcess(), Mods, sizeof(Mods), &Needed, W10_LIST_MODULES_ALL);
    ok(Ret, "K32EnumProcessModulesEx failed: %lu\n", GetLastError());
    ok(Needed >= sizeof(HMODULE), "needed %lu\n", Needed);
    ok(Mods[0] == GetModuleHandleW(NULL), "first module %p, exe %p\n", Mods[0], GetModuleHandleW(NULL));

    SetLastError(0xdeadbeef);
    Ret = pEnum(GetCurrentProcess(), Mods, sizeof(Mods), &Needed, 4);
    ok(!Ret, "filter 4 accepted\n");
    ok(GetLastError() == ERROR_INVALID_PARAMETER, "filter 4: error %lu\n", GetLastError());

    /* Observation only: what a 32-bit-only listing does for a native process. */
    Needed = 0xdeadbeef;
    SetLastError(0xdeadbeef);
    Ret = pEnum(GetCurrentProcess(), Mods, sizeof(Mods), &Needed, W10_LIST_MODULES_32BIT);
    trace("LIST_MODULES_32BIT: ret=%d needed=%lu error=%lu (sizeof(void*)=%u)\n",
          Ret, Needed, GetLastError(), (unsigned)sizeof(void *));
}

/* ---- 6 ---- */
static DWORD WINAPI W10_ThreadProc(LPVOID Param)
{
    return 0;
}

static void Test_NtGetNextThread(void)
{
    NTSTATUS (NTAPI *pNext)(HANDLE, HANDLE, ACCESS_MASK, ULONG, ULONG, PHANDLE) =
        (void *)GetNt("NtGetNextThread");
    DWORD Ids[256], Count = 0, i, NewId = 0, Self = GetCurrentThreadId();
    HANDLE Thread, Prev = NULL, Next = NULL;
    BOOL SawSelf = FALSE, SawNew = FALSE;
    NTSTATUS Status;

    if (!pNext)
    {
        win_skip("NtGetNextThread missing\n");
        return;
    }
    Thread = CreateThread(NULL, 0, W10_ThreadProc, NULL, CREATE_SUSPENDED, &NewId);
    ok(Thread != NULL, "CreateThread failed: %lu\n", GetLastError());
    if (!Thread)
        return;

    for (;;)
    {
        Next = NULL;
        Status = pNext(GetCurrentProcess(), Prev, THREAD_QUERY_INFORMATION, 0, 0, &Next);
        if (Prev)
            CloseHandle(Prev);
        if (Status != 0)
            break;
        if (Count < 256)
            Ids[Count++] = GetThreadId(Next);
        Prev = Next;
        if (Count >= 256)
        {
            CloseHandle(Next);
            Status = 0;
            break;
        }
    }
    for (i = 0; i < Count; i++)
    {
        if (Ids[i] == Self) SawSelf = TRUE;
        if (Ids[i] == NewId) SawNew = TRUE;
    }
    ok(Count >= 2, "only %lu threads walked\n", Count);
    ok(SawSelf, "current thread not seen\n");
    ok(SawNew, "new thread %lu not seen\n", NewId);
    ok(Status == W10_STATUS_NO_MORE_ENTRIES, "walk ended with %08lx\n", Status);

    /* The Vista limited right on our own thread. (A walk asking for it hit ACCESS_DENIED
     * on some other thread of this process in our OS, cause not yet established; the walk
     * above asks for THREAD_QUERY_INFORMATION. See modern-windows-compat-roadmap.md.) */
    Next = OpenThread(THREAD_QUERY_LIMITED_INFORMATION, FALSE, Self);
    ok(Next != NULL, "OpenThread(THREAD_QUERY_LIMITED_INFORMATION) failed: %lu\n", GetLastError());
    if (Next)
        CloseHandle(Next);

    ResumeThread(Thread);
    WaitForSingleObject(Thread, 5000);
    CloseHandle(Thread);
}

/* ---- 7 ---- */
static void Test_Transaction(void)
{
    PVOID (NTAPI *pGet)(void) = (void *)GetNt("RtlGetCurrentTransaction");
    BOOLEAN (NTAPI *pSet)(PVOID) = (void *)GetNt("RtlSetCurrentTransaction");
    BOOLEAN Ret;

    if (!pGet || !pSet)
    {
        win_skip("RtlGet/SetCurrentTransaction missing\n");
        return;
    }
    ok(pGet() == NULL, "initial transaction %p\n", pGet());
    Ret = pSet((PVOID)(ULONG_PTR)0x1234);
    ok(Ret == TRUE, "RtlSetCurrentTransaction returned %d\n", Ret);
    ok(pGet() == (PVOID)(ULONG_PTR)0x1234, "transaction %p\n", pGet());
    pSet(NULL);
    ok(pGet() == NULL, "transaction not cleared: %p\n", pGet());
}

/* ---- 8 ---- */
static void Test_NtOpenKeyEx(void)
{
    NTSTATUS (NTAPI *pOpenKeyEx)(PHANDLE, ACCESS_MASK, POBJECT_ATTRIBUTES, ULONG) = (void *)GetNt("NtOpenKeyEx");
    static WCHAR Name[] = L"\\Registry\\Machine\\Software";
    UNICODE_STRING Us;
    OBJECT_ATTRIBUTES Attr;
    HANDLE Key = NULL;
    NTSTATUS Status;

    if (!pOpenKeyEx)
    {
        win_skip("NtOpenKeyEx missing\n");
        return;
    }
    Us.Buffer = Name;
    Us.Length = (USHORT)(lstrlenW(Name) * sizeof(WCHAR));
    Us.MaximumLength = Us.Length + sizeof(WCHAR);
    InitializeObjectAttributes(&Attr, &Us, OBJ_CASE_INSENSITIVE, NULL, NULL);
    Status = pOpenKeyEx(&Key, KEY_READ, &Attr, 0);
    ok(Status == 0, "NtOpenKeyEx: %08lx\n", Status);
    if (Status == 0)
        CloseHandle(Key);
}

/* ---- 9 ---- */
static void Test_Utf8(void)
{
    NTSTATUS (NTAPI *pToW)(PWSTR, ULONG, PULONG, const char *, ULONG) = (void *)GetNt("RtlUTF8ToUnicodeN");
    NTSTATUS (NTAPI *pToU8)(char *, ULONG, PULONG, PCWSTR, ULONG) = (void *)GetNt("RtlUnicodeToUTF8N");
    WCHAR Wide[16];
    char Narrow[16];
    ULONG Out = 0;
    NTSTATUS Status;

    if (!pToW || !pToU8)
    {
        win_skip("RtlUTF8ToUnicodeN/RtlUnicodeToUTF8N missing\n");
        return;
    }
    ZeroMemory(Wide, sizeof(Wide));
    Status = pToW(Wide, sizeof(Wide), &Out, "h\xc3\xa9", 3);
    ok(Status == 0, "RtlUTF8ToUnicodeN: %08lx\n", Status);
    ok(Out == 4, "wide result size %lu\n", Out);
    ok(Wide[0] == 'h' && Wide[1] == 0xe9, "wide text %04x %04x\n", Wide[0], Wide[1]);

    ZeroMemory(Narrow, sizeof(Narrow));
    Out = 0;
    Status = pToU8(Narrow, sizeof(Narrow), &Out, L"h\xe9", 4);
    ok(Status == 0, "RtlUnicodeToUTF8N: %08lx\n", Status);
    ok(Out == 3, "utf8 result size %lu\n", Out);
    ok(!memcmp(Narrow, "h\xc3\xa9", 3), "utf8 bytes %02x %02x %02x\n",
       (unsigned char)Narrow[0], (unsigned char)Narrow[1], (unsigned char)Narrow[2]);
}

/* ---- 10 ---- */
static WNDPROC W10_OldProc;
static int W10_ClipUpdates;

static LRESULT CALLBACK W10_ClipProc(HWND hwnd, UINT msg, WPARAM wp, LPARAM lp)
{
    if (msg == WM_CLIPBOARDUPDATE)
        W10_ClipUpdates++;
    return CallWindowProcW(W10_OldProc, hwnd, msg, wp, lp);
}

static void Test_ClipboardListener(void)
{
    BOOL (WINAPI *pAdd)(HWND) = (void *)GetProcAddress(GetModuleHandleW(L"user32.dll"), "AddClipboardFormatListener");
    BOOL (WINAPI *pRemove)(HWND) = (void *)GetProcAddress(GetModuleHandleW(L"user32.dll"), "RemoveClipboardFormatListener");
    HWND Wnd;
    HGLOBAL Mem;
    BOOL Ret, Opened;
    DWORD Start;
    LPWSTR Text;

    if (!pAdd || !pRemove)
    {
        win_skip("Add/RemoveClipboardFormatListener missing\n");
        return;
    }
    Wnd = CreateWindowExW(0, L"STATIC", L"w10s", WS_OVERLAPPED, 0, 0, 10, 10, NULL, NULL, GetModuleHandleW(NULL), NULL);
    if (!Wnd)
    {
        skip("cannot create window (error %lu)\n", GetLastError());
        return;
    }
    W10_ClipUpdates = 0;
    W10_OldProc = (WNDPROC)SetWindowLongPtrW(Wnd, GWLP_WNDPROC, (LONG_PTR)W10_ClipProc);

    Ret = pAdd(Wnd);
    ok(Ret, "AddClipboardFormatListener failed: %lu\n", GetLastError());
    SetLastError(0xdeadbeef);
    Ret = pAdd(Wnd);
    trace("second AddClipboardFormatListener: ret=%d error=%lu\n", Ret, GetLastError());

    W10_ClipUpdates = 0;
    Opened = OpenClipboard(Wnd);
    if (!Opened)
        skip("OpenClipboard failed (error %lu)\n", GetLastError());
    else
    {
        EmptyClipboard();
        Mem = GlobalAlloc(GMEM_MOVEABLE, 6 * sizeof(WCHAR));
        Text = Mem ? GlobalLock(Mem) : NULL;
        if (Text)
        {
            lstrcpyW(Text, L"w10s");
            GlobalUnlock(Mem);
            ok(SetClipboardData(CF_UNICODETEXT, Mem) != NULL, "SetClipboardData failed: %lu\n", GetLastError());
        }
        CloseClipboard();

        Start = GetTickCount();
        while (!W10_ClipUpdates && GetTickCount() - Start < 2000)
        {
            MSG Msg;
            while (PeekMessageW(&Msg, NULL, 0, 0, PM_REMOVE))
            {
                TranslateMessage(&Msg);
                DispatchMessageW(&Msg);
            }
            if (!W10_ClipUpdates)
                MsgWaitForMultipleObjects(0, NULL, FALSE, 100, QS_ALLINPUT);
        }
        ok(W10_ClipUpdates > 0, "WM_CLIPBOARDUPDATE never arrived\n");
    }

    Ret = pRemove(Wnd);
    ok(Ret, "RemoveClipboardFormatListener failed: %lu\n", GetLastError());
    SetLastError(0xdeadbeef);
    Ret = pRemove(Wnd);
    trace("second RemoveClipboardFormatListener: ret=%d error=%lu\n", Ret, GetLastError());

    SetLastError(0xdeadbeef);
    Ret = pAdd(NULL);
    ok(!Ret, "AddClipboardFormatListener(NULL) succeeded\n");
    trace("AddClipboardFormatListener(NULL): error=%lu\n", GetLastError());

    DestroyWindow(Wnd);
}

/* ---- 11 ---- */
typedef struct
{
    ULONG cbSize;
    DWORD nFont;
    COORD dwFontSize;
    UINT FontFamily;
    UINT FontWeight;
    WCHAR FaceName[32];
} W10_FONT_INFOEX;

static void Test_ConsoleFontEx(void)
{
    BOOL (WINAPI *pGet)(HANDLE, BOOL, W10_FONT_INFOEX *) = (void *)GetK32("GetCurrentConsoleFontEx");
    HANDLE Out = GetStdHandle(STD_OUTPUT_HANDLE);
    W10_FONT_INFOEX Fi;
    DWORD Mode;
    BOOL Ret;

    if (!pGet)
    {
        win_skip("GetCurrentConsoleFontEx missing\n");
        return;
    }
    ZeroMemory(&Fi, sizeof(Fi));
    Fi.cbSize = sizeof(Fi) - 1;
    SetLastError(0xdeadbeef);
    Ret = pGet(Out, FALSE, &Fi);
    if (!GetConsoleMode(Out, &Mode))
    {
        /* Redirected stdout: the handle check wins over the size check on Windows. */
        trace("stdout is not a console: ret=%d error=%lu\n", Ret, GetLastError());
        ok(!Ret, "bad cbSize accepted on a non-console handle\n");
    }
    else
    {
        ok(!Ret, "bad cbSize accepted\n");
        ok(GetLastError() == ERROR_INVALID_PARAMETER, "bad cbSize: error %lu\n", GetLastError());
    }
    ZeroMemory(&Fi, sizeof(Fi));
    Fi.cbSize = sizeof(Fi);
    SetLastError(0xdeadbeef);
    Ret = pGet(Out, FALSE, &Fi);
    trace("GetCurrentConsoleFontEx: ret=%d error=%lu font=%lu size=%d,%d face=%S\n",
          Ret, GetLastError(), Fi.nFont, Fi.dwFontSize.X, Fi.dwFontSize.Y, Ret ? Fi.FaceName : L"");
}

/* ---- 12 ---- */
static void Test_InitializeContext(void)
{
    BOOL (WINAPI *pInit)(PVOID, DWORD, PCONTEXT *, PDWORD) = (void *)GetK32("InitializeContext");
    DWORD Len = 0;
    PCONTEXT Ctx = NULL;
    PVOID Buf;
    BOOL Ret;

    if (!pInit)
    {
        win_skip("InitializeContext missing\n");
        return;
    }
    SetLastError(0xdeadbeef);
    Ret = pInit(NULL, CONTEXT_FULL, NULL, &Len);
    ok(!Ret, "InitializeContext(NULL) succeeded\n");
    ok(GetLastError() == ERROR_INSUFFICIENT_BUFFER, "error %lu\n", GetLastError());
    ok(Len > 0, "length %lu\n", Len);
    if (Len == 0 || Len > 0x100000)
        return;
    Buf = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, Len);
    Ret = pInit(Buf, CONTEXT_FULL, &Ctx, &Len);
    ok(Ret, "InitializeContext failed: %lu\n", GetLastError());
    if (Ret)
    {
        ok((BYTE *)Ctx >= (BYTE *)Buf && (BYTE *)Ctx < (BYTE *)Buf + Len, "context %p outside %p+%lu\n", Ctx, Buf, Len);
        ok(Ctx->ContextFlags == CONTEXT_FULL, "ContextFlags %08lx\n", Ctx->ContextFlags);
    }
    HeapFree(GetProcessHeap(), 0, Buf);
}

/* ---- 13 ---- */
static void Test_Package(void)
{
    LONG (WINAPI *pPkg)(UINT32 *, PWSTR) = (void *)GetK32("GetCurrentPackageFullName");
    UINT32 Len = 0;
    LONG Ret;

    if (!pPkg)
    {
        win_skip("GetCurrentPackageFullName missing\n");
        return;
    }
    Ret = pPkg(&Len, NULL);
    ok(Ret == W10_APPMODEL_ERROR_NO_PACKAGE, "GetCurrentPackageFullName: %ld\n", Ret);
}

/* ---- 14 ---- */
typedef struct { UINT_PTR fd; SHORT events; SHORT revents; } W10_POLLFD;
typedef struct { USHORT family; USHORT port; ULONG addr; char zero[8]; } W10_SOCKADDR_IN;
typedef struct W10_ADDRINFOEXW
{
    int ai_flags, ai_family, ai_socktype, ai_protocol;
    SIZE_T ai_addrlen;
    PWSTR ai_canonname;
    void *ai_addr;
    void *ai_blob;
    SIZE_T ai_bloblen;
    GUID *ai_provider;
    struct W10_ADDRINFOEXW *ai_next;
} W10_ADDRINFOEXW;

static void Test_Winsock(void)
{
    HMODULE Ws2 = LoadLibraryW(L"ws2_32.dll");
    int (WINAPI *pStartup)(WORD, void *);
    int (WINAPI *pCleanup)(void);
    UINT_PTR (WINAPI *pSocket)(int, int, int);
    int (WINAPI *pBind)(UINT_PTR, const void *, int);
    int (WINAPI *pClose)(UINT_PTR);
    int (WINAPI *pPoll)(W10_POLLFD *, ULONG, INT);
    INT (WINAPI *pGetAddrInfoEx)(PCWSTR, PCWSTR, DWORD, GUID *, const W10_ADDRINFOEXW *,
                                 W10_ADDRINFOEXW **, void *, void *, void *, HANDLE *);
    void (WINAPI *pFreeAddrInfoEx)(W10_ADDRINFOEXW *);
    BYTE WsaData[1024];
    W10_SOCKADDR_IN Addr;
    W10_POLLFD Pfd;
    W10_ADDRINFOEXW Hints, *Res = NULL;
    UINT_PTR Sock;
    int Ret;

    if (!Ws2)
    {
        win_skip("ws2_32 missing\n");
        return;
    }
    pStartup = (void *)GetProcAddress(Ws2, "WSAStartup");
    pCleanup = (void *)GetProcAddress(Ws2, "WSACleanup");
    pSocket = (void *)GetProcAddress(Ws2, "socket");
    pBind = (void *)GetProcAddress(Ws2, "bind");
    pClose = (void *)GetProcAddress(Ws2, "closesocket");
    pPoll = (void *)GetProcAddress(Ws2, "WSAPoll");
    pGetAddrInfoEx = (void *)GetProcAddress(Ws2, "GetAddrInfoExW");
    pFreeAddrInfoEx = (void *)GetProcAddress(Ws2, "FreeAddrInfoExW");
    if (!pStartup || !pCleanup || !pSocket || !pBind || !pClose || !pPoll || !pGetAddrInfoEx || !pFreeAddrInfoEx)
    {
        win_skip("ws2_32 exports missing\n");
        return;
    }
    Ret = pStartup(MAKEWORD(2, 2), WsaData);
    ok(Ret == 0, "WSAStartup: %d\n", Ret);
    if (Ret)
        return;

    Sock = pSocket(2 /* AF_INET */, 2 /* SOCK_DGRAM */, 17 /* IPPROTO_UDP */);
    ok(Sock != (UINT_PTR)-1, "socket failed\n");
    if (Sock != (UINT_PTR)-1)
    {
        ZeroMemory(&Addr, sizeof(Addr));
        Addr.family = 2;
        Addr.addr = 0x0100007f; /* 127.0.0.1 in network order, little-endian host */
        ok(pBind(Sock, &Addr, sizeof(Addr)) == 0, "bind failed\n");

        Pfd.fd = Sock; Pfd.events = 0x0010 /* POLLWRNORM */; Pfd.revents = 0;
        Ret = pPoll(&Pfd, 1, 1000);
        ok(Ret == 1, "WSAPoll(write): %d\n", Ret);
        ok(Pfd.revents & 0x0010, "revents %04x\n", Pfd.revents);

        Pfd.fd = Sock; Pfd.events = 0x0100 /* POLLRDNORM */; Pfd.revents = 0;
        Ret = pPoll(&Pfd, 1, 0);
        ok(Ret == 0, "WSAPoll(read): %d revents %04x\n", Ret, Pfd.revents);
        pClose(Sock);
    }

    ZeroMemory(&Hints, sizeof(Hints));
    Hints.ai_family = 2;
    Ret = pGetAddrInfoEx(L"localhost", NULL, 12 /* NS_DNS */, NULL, &Hints, &Res, NULL, NULL, NULL, NULL);
    ok(Ret == 0, "GetAddrInfoExW: %d\n", Ret);
    ok(Res != NULL, "no result\n");
    if (Res)
        pFreeAddrInfoEx(Res);
    pCleanup();
}

/* ---- 15 ---- */
static void Test_IpHelper(void)
{
    ULONG (WINAPI *pFwd)(USHORT, ULONG **) = (void *)GetDyn(L"iphlpapi.dll", "GetIpForwardTable2");
    ULONG (WINAPI *pUni)(USHORT, ULONG **);
    void (WINAPI *pFree)(PVOID);
    HMODULE Mod = GetModuleHandleW(L"iphlpapi.dll");
    ULONG *Table = NULL;
    ULONG Ret;

    if (!pFwd)
        return;
    pUni = (void *)GetProcAddress(Mod, "GetUnicastIpAddressTable");
    pFree = (void *)GetProcAddress(Mod, "FreeMibTable");
    if (!pUni || !pFree)
    {
        win_skip("iphlpapi table exports missing\n");
        return;
    }
    Ret = pFwd(2, &Table);
    ok(Ret == NO_ERROR, "GetIpForwardTable2: %lu\n", Ret);
    if (Ret == NO_ERROR && Table)
    {
        ok(Table[0] > 0, "forward table empty\n");
        pFree(Table);
    }
    Table = NULL;
    Ret = pUni(2, &Table);
    ok(Ret == NO_ERROR, "GetUnicastIpAddressTable: %lu\n", Ret);
    if (Ret == NO_ERROR && Table)
    {
        ok(Table[0] >= 1, "no unicast addresses\n");
        pFree(Table);
    }
}

/* ---- 16 ---- */
static void Test_ShcoreDpi(void)
{
    HRESULT (WINAPI *pGetDpi)(HMONITOR, int, UINT *, UINT *) = (void *)GetDyn(L"shcore.dll", "GetDpiForMonitor");
    HMONITOR Mon;
    POINT Pt = { 0, 0 };
    UINT X = 0, Y = 0;
    HRESULT hr;

    if (!pGetDpi)
        return;
    Mon = MonitorFromPoint(Pt, MONITOR_DEFAULTTOPRIMARY);
    hr = pGetDpi(Mon, 0 /* MDT_EFFECTIVE_DPI */, &X, &Y);
    ok(hr == S_OK, "GetDpiForMonitor: %08lx\n", hr);
    ok(X > 0 && Y > 0, "dpi %u x %u\n", X, Y);
}

/* ---- 17 ---- */
static void Test_SetupApiProperty(void)
{
    HMODULE Mod = LoadLibraryW(L"setupapi.dll");
    HANDLE (WINAPI *pClassDevs)(const GUID *, PCWSTR, HWND, DWORD);
    BOOL (WINAPI *pEnum)(HANDLE, DWORD, W10_SP_DEVINFO_DATA *);
    BOOL (WINAPI *pInstId)(HANDLE, W10_SP_DEVINFO_DATA *, PWSTR, DWORD, PDWORD);
    BOOL (WINAPI *pProp)(HANDLE, W10_SP_DEVINFO_DATA *, const W10_DEVPROPKEY *, ULONG *, PBYTE, DWORD, PDWORD, DWORD);
    BOOL (WINAPI *pDestroy)(HANDLE);
    HANDLE Set;
    W10_SP_DEVINFO_DATA Dev;
    WCHAR Id[512], Prop[512];
    ULONG Type;
    DWORD Req;
    BOOL Ret;

    if (!Mod)
    {
        win_skip("setupapi missing\n");
        return;
    }
    pClassDevs = (void *)GetProcAddress(Mod, "SetupDiGetClassDevsW");
    pEnum = (void *)GetProcAddress(Mod, "SetupDiEnumDeviceInfo");
    pInstId = (void *)GetProcAddress(Mod, "SetupDiGetDeviceInstanceIdW");
    pProp = (void *)GetProcAddress(Mod, "SetupDiGetDevicePropertyW");
    pDestroy = (void *)GetProcAddress(Mod, "SetupDiDestroyDeviceInfoList");
    if (!pClassDevs || !pEnum || !pInstId || !pProp || !pDestroy)
    {
        win_skip("setupapi exports missing\n");
        return;
    }
    Set = pClassDevs(NULL, NULL, NULL, 0x4 /* DIGCF_ALLCLASSES */ | 0x2 /* DIGCF_PRESENT */);
    ok(Set != INVALID_HANDLE_VALUE, "SetupDiGetClassDevsW failed: %lu\n", GetLastError());
    if (Set == INVALID_HANDLE_VALUE)
        return;
    ZeroMemory(&Dev, sizeof(Dev));
    Dev.cbSize = sizeof(Dev);
    if (!pEnum(Set, 0, &Dev))
    {
        skip("no devices (error %lu)\n", GetLastError());
        pDestroy(Set);
        return;
    }
    ok(pInstId(Set, &Dev, Id, 512, NULL), "SetupDiGetDeviceInstanceIdW failed: %lu\n", GetLastError());

    Type = 0; Req = 0;
    ZeroMemory(Prop, sizeof(Prop));
    Ret = pProp(Set, &Dev, &W10_Key_InstanceId, &Type, (PBYTE)Prop, sizeof(Prop), &Req, 0);
    ok(Ret, "InstanceId property failed: %lu\n", GetLastError());
    ok(Type == 0x12 /* DEVPROP_TYPE_STRING */, "type %lx\n", Type);
    ok(!lstrcmpW(Prop, Id), "property %S, instance id %S\n", Prop, Id);

    Req = 0;
    SetLastError(0xdeadbeef);
    Ret = pProp(Set, &Dev, &W10_Key_InstanceId, &Type, (PBYTE)Prop, 2, &Req, 0);
    ok(!Ret, "2-byte buffer succeeded\n");
    ok(GetLastError() == ERROR_INSUFFICIENT_BUFFER, "small buffer: error %lu\n", GetLastError());
    ok(Req == (lstrlenW(Id) + 1) * sizeof(WCHAR), "required %lu, expected %u\n", Req, (lstrlenW(Id) + 1) * (unsigned)sizeof(WCHAR));

    SetLastError(0xdeadbeef);
    Ret = pProp(Set, &Dev, &W10_Key_InstanceId, &Type, (PBYTE)Prop, sizeof(Prop), &Req, 1);
    ok(!Ret, "Flags=1 succeeded\n");
    ok(GetLastError() == ERROR_INVALID_FLAGS, "Flags=1: error %lu\n", GetLastError());

    Type = 0;
    if (pProp(Set, &Dev, &W10_Key_HardwareIds, &Type, (PBYTE)Prop, sizeof(Prop), &Req, 0))
        ok(Type == 0x2012 /* DEVPROP_TYPE_STRING_LIST */, "HardwareIds type %lx\n", Type);
    else
        trace("HardwareIds not present on device 0 (error %lu)\n", GetLastError());
    pDestroy(Set);
}

/* ---- 18 ---- */
static void Test_CfgMgrProperty(void)
{
    HMODULE Mod = LoadLibraryW(L"cfgmgr32.dll");
    ULONG (WINAPI *pLocate)(DWORD *, PWSTR, ULONG);
    ULONG (WINAPI *pGetId)(DWORD, PWSTR, ULONG, ULONG);
    ULONG (WINAPI *pGetProp)(DWORD, const W10_DEVPROPKEY *, ULONG *, PVOID, ULONG *, ULONG);
    DWORD Dn = 0;
    WCHAR Id[512], Prop[512];
    ULONG Type = 0, Size = sizeof(Prop), Cr;

    if (!Mod)
    {
        win_skip("cfgmgr32 missing\n");
        return;
    }
    pLocate = (void *)GetProcAddress(Mod, "CM_Locate_DevNodeW");
    pGetId = (void *)GetProcAddress(Mod, "CM_Get_Device_IDW");
    pGetProp = (void *)GetProcAddress(Mod, "CM_Get_DevNode_PropertyW");
    if (!pLocate || !pGetId || !pGetProp)
    {
        win_skip("cfgmgr32 exports missing\n");
        return;
    }
    Cr = pLocate(&Dn, NULL, 0 /* CM_LOCATE_DEVNODE_NORMAL */);
    ok(Cr == 0, "CM_Locate_DevNodeW: %lx\n", Cr);
    if (Cr)
        return;
    Cr = pGetId(Dn, Id, 512, 0);
    ok(Cr == 0, "CM_Get_Device_IDW: %lx\n", Cr);
    ZeroMemory(Prop, sizeof(Prop));
    Cr = pGetProp(Dn, &W10_Key_InstanceId, &Type, Prop, &Size, 0);
    ok(Cr == 0, "CM_Get_DevNode_PropertyW: %lx\n", Cr);
    ok(Type == 0x12, "type %lx\n", Type);
    ok(!lstrcmpiW(Prop, Id), "property %S, id %S\n", Prop, Id);
}

/* ---- 1 (last: changes process state) ---- */
static void Test_DllDirectories(void)
{
    PVOID (WINAPI *pAdd)(PCWSTR) = (void *)GetK32("AddDllDirectory");
    BOOL (WINAPI *pRemove)(PVOID) = (void *)GetK32("RemoveDllDirectory");
    BOOL (WINAPI *pSetDefault)(DWORD) = (void *)GetK32("SetDefaultDllDirectories");
    WCHAR Dir[MAX_PATH + 32];
    PVOID Cookie;
    HMODULE Mod;
    BOOL Ret;

    if (!pAdd || !pRemove || !pSetDefault)
    {
        win_skip("DLL directory APIs missing\n");
        return;
    }
    SetLastError(0xdeadbeef);
    Cookie = pAdd(L".\\.");
    ok(Cookie == NULL, "relative directory accepted\n");
    ok(GetLastError() == ERROR_INVALID_PARAMETER, "relative: error %lu\n", GetLastError());

    GetWindowsDirectoryW(Dir, MAX_PATH);
    lstrcatW(Dir, L"\\nosuchdir_w10s");
    SetLastError(0xdeadbeef);
    Cookie = pAdd(Dir);
    ok(Cookie == NULL, "missing directory accepted\n");
    ok(GetLastError() == ERROR_FILE_NOT_FOUND, "missing: error %lu\n", GetLastError());

    GetSystemDirectoryW(Dir, MAX_PATH);
    Cookie = pAdd(Dir);
    ok(Cookie != NULL, "AddDllDirectory(system): error %lu\n", GetLastError());
    if (Cookie)
        ok(pRemove(Cookie), "RemoveDllDirectory: error %lu\n", GetLastError());

    SetLastError(0xdeadbeef);
    Ret = pSetDefault(0);
    ok(!Ret, "SetDefaultDllDirectories(0) succeeded\n");
    ok(GetLastError() == ERROR_INVALID_PARAMETER, "SetDefaultDllDirectories(0): error %lu\n", GetLastError());

    if (GetModuleHandleW(L"wtsapi32.dll"))
    {
        skip("wtsapi32.dll already loaded\n");
        return;
    }
    SetLastError(0xdeadbeef);
    Mod = LoadLibraryExW(L"wtsapi32.dll", NULL, LOAD_WITH_ALTERED_SEARCH_PATH | LOAD_LIBRARY_SEARCH_SYSTEM32);
    ok(Mod == NULL, "altered + search flags accepted\n");
    ok(GetLastError() == ERROR_INVALID_PARAMETER, "altered + search: error %lu\n", GetLastError());
    if (Mod) FreeLibrary(Mod);

    Mod = LoadLibraryExW(L"wtsapi32.dll", NULL, LOAD_LIBRARY_SEARCH_APPLICATION_DIR);
    ok(Mod == NULL, "wtsapi32.dll found in the application directory\n");
    if (Mod) FreeLibrary(Mod);

    ok(pSetDefault(LOAD_LIBRARY_SEARCH_USER_DIRS), "SetDefaultDllDirectories(USER_DIRS): %lu\n", GetLastError());
    Mod = LoadLibraryW(L"wtsapi32.dll");
    ok(Mod == NULL, "wtsapi32.dll loaded with only user dirs\n");
    if (Mod) FreeLibrary(Mod);

    ok(pSetDefault(LOAD_LIBRARY_SEARCH_SYSTEM32), "SetDefaultDllDirectories(SYSTEM32): %lu\n", GetLastError());
    Mod = LoadLibraryW(L"wtsapi32.dll");
    ok(Mod != NULL, "wtsapi32.dll not loaded with system32: error %lu\n", GetLastError());
    if (Mod) FreeLibrary(Mod);

    pSetDefault(LOAD_LIBRARY_SEARCH_DEFAULT_DIRS);
}

START_TEST(Win10Surface)
{
    Test_GetProductInfo();
    Test_PathCch();
    Test_Idn();
    Test_EnumProcessModulesEx();
    Test_NtGetNextThread();
    Test_Transaction();
    Test_NtOpenKeyEx();
    Test_Utf8();
    Test_ClipboardListener();
    Test_ConsoleFontEx();
    Test_InitializeContext();
    Test_Package();
    Test_Winsock();
    Test_IpHelper();
    Test_ShcoreDpi();
    Test_SetupApiProperty();
    Test_CfgMgrProperty();
    Test_DllDirectories();
}
