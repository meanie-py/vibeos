/*
 * PROJECT:     ReactOS api tests
 * LICENSE:     LGPL-2.0-or-later (https://spdx.org/licenses/LGPL-2.0-or-later)
 * PURPOSE:     Windows 8/10 exports apps import: CompareObjectHandles,
 *              SetFileInformationByHandle, CreateFile2, the thread/time helpers, and the
 *              WinRT string and DPI calls reached through their API-set names (as apps
 *              reach them). Looked up at run time so the same binary grades Windows 11.
 */
#include "precomp.h"
#include <winuser.h>
#include <wingdi.h>

typedef void *HSTRING_T;
static const GUID IID_IUnknown_T = { 0, 0, 0, { 0xC0, 0, 0, 0, 0, 0, 0, 0x46 } };

/* FILE_INFO_BY_HANDLE_CLASS values and their structs (minwinbase.h), declared here so the
 * test does not depend on which SDK level precomp.h selects. */
enum { CLS_RENAME = 3, CLS_DISPOSITION = 4, CLS_END_OF_FILE = 6 };
typedef struct { LARGE_INTEGER EndOfFile; } EOF_INFO_T;
typedef struct { BOOLEAN DeleteFile; } DISPOSITION_INFO_T;
typedef struct { BOOLEAN ReplaceIfExists; HANDLE RootDirectory; DWORD FileNameLength; WCHAR FileName[1]; } RENAME_INFO_T;

static FARPROC Get(LPCWSTR Dll, LPCSTR Name)
{
    HMODULE Mod = LoadLibraryW(Dll);
    FARPROC Proc = Mod ? GetProcAddress(Mod, Name) : NULL;
    ok(Proc != NULL, "%S!%s missing (module %p)\n", Dll, Name, Mod);
    return Proc;
}

static void Test_CompareObjectHandles(void)
{
    BOOL (WINAPI *pCompare)(HANDLE, HANDLE) = (void *)Get(L"api-ms-win-core-handle-l1-1-0.dll", "CompareObjectHandles");
    HANDLE A, B, Dup;

    if (!pCompare)
        return;
    A = CreateEventW(NULL, TRUE, FALSE, NULL);
    B = CreateEventW(NULL, TRUE, FALSE, NULL);
    DuplicateHandle(GetCurrentProcess(), A, GetCurrentProcess(), &Dup, 0, FALSE, DUPLICATE_SAME_ACCESS);
    ok(Dup != A, "duplicate has the same value\n");
    ok(pCompare(A, Dup) == TRUE, "A vs its duplicate: error %lu\n", GetLastError());
    SetLastError(0xdeadbeef);
    ok(pCompare(A, B) == FALSE, "A vs B compared equal\n");
    ok(GetLastError() == ERROR_NOT_SAME_OBJECT, "A vs B: error %lu\n", GetLastError());
    CloseHandle(Dup);
    CloseHandle(B);
    CloseHandle(A);
}

static void Test_FileInfoByHandle(void)
{
    BOOL (WINAPI *pSet)(HANDLE, int, LPVOID, DWORD) =
        (void *)Get(L"kernel32.dll", "SetFileInformationByHandle");
    HANDLE (WINAPI *pCreateFile2)(LPCWSTR, DWORD, DWORD, DWORD, void *) =
        (void *)Get(L"kernel32.dll", "CreateFile2");
    WCHAR Dir[MAX_PATH], Path[MAX_PATH], Path2[MAX_PATH];
    EOF_INFO_T Eof;
    DISPOSITION_INFO_T Disp;
    struct { RENAME_INFO_T Info; WCHAR Rest[MAX_PATH]; } Ren;
    LARGE_INTEGER Size;
    HANDLE File;

    if (!pSet || !pCreateFile2)
        return;
    GetTempPathW(MAX_PATH, Dir);
    /* The live CD's temp directory is read-only; the sweep's working directory is not. */
    if (!GetTempFileNameW(Dir, L"w8s", 0, Path) && !GetTempFileNameW(L".", L"w8s", 0, Path))
    {
        skip("no writable temp directory (error %lu)\n", GetLastError());
        return;
    }
    lstrcpyW(Path2, Path);
    lstrcatW(Path2, L".renamed");
    DeleteFileW(Path2);

    File = pCreateFile2(Path, GENERIC_READ | GENERIC_WRITE | DELETE, 0, CREATE_ALWAYS, NULL);
    ok(File != INVALID_HANDLE_VALUE, "CreateFile2: error %lu\n", GetLastError());
    if (File == INVALID_HANDLE_VALUE)
        return;

    Eof.EndOfFile.QuadPart = 4096;
    ok(pSet(File, CLS_END_OF_FILE, &Eof, sizeof(Eof)), "EndOfFile: error %lu\n", GetLastError());
    ok(GetFileSizeEx(File, &Size) && Size.QuadPart == 4096, "size %I64d\n", Size.QuadPart);

    ZeroMemory(&Ren, sizeof(Ren));
    Ren.Info.ReplaceIfExists = TRUE;
    Ren.Info.FileNameLength = lstrlenW(Path2) * sizeof(WCHAR);
    lstrcpyW(Ren.Info.FileName, Path2);
    ok(pSet(File, CLS_RENAME, &Ren, sizeof(Ren)), "Rename: error %lu\n", GetLastError());
    ok(GetFileAttributesW(Path2) != INVALID_FILE_ATTRIBUTES, "renamed file missing\n");
    ok(GetFileAttributesW(Path) == INVALID_FILE_ATTRIBUTES, "old name still exists\n");

    Disp.DeleteFile = TRUE;
    ok(pSet(File, CLS_DISPOSITION, &Disp, sizeof(Disp)), "Disposition: error %lu\n", GetLastError());
    SetLastError(0xdeadbeef);
    ok(!pSet(File, 0x7fff, &Disp, sizeof(Disp)), "bogus class accepted\n");
    ok(GetLastError() == ERROR_INVALID_PARAMETER, "bogus class: error %lu\n", GetLastError());
    CloseHandle(File);
    ok(GetFileAttributesW(Path2) == INVALID_FILE_ATTRIBUTES, "delete-on-close did not delete\n");
}

static void Test_ThreadAndTime(void)
{
    VOID (WINAPI *pLimits)(PULONG_PTR, PULONG_PTR) = (void *)Get(L"kernel32.dll", "GetCurrentThreadStackLimits");
    BOOL (WINAPI *pUnbiased)(PULONGLONG) = (void *)Get(L"kernel32.dll", "QueryUnbiasedInterruptTime");
    WORD (WINAPI *pGroups)(void) = (void *)Get(L"kernel32.dll", "GetMaximumProcessorGroupCount");
    ULONG_PTR Low = 0, High = 0;
    ULONGLONG T1 = 0, T2 = 0;

    if (pLimits)
    {
        pLimits(&Low, &High);
        ok(Low < (ULONG_PTR)&Low && (ULONG_PTR)&Low < High, "stack %Ix..%Ix does not hold %p\n", Low, High, &Low);
    }
    if (pUnbiased)
    {
        ok(pUnbiased(&T1), "QueryUnbiasedInterruptTime failed\n");
        Sleep(20);
        pUnbiased(&T2);
        ok(T1 != 0 && T2 > T1, "unbiased time %I64u -> %I64u\n", T1, T2);
    }
    if (pGroups)
        ok(pGroups() >= 1, "GetMaximumProcessorGroupCount %u\n", pGroups());
}

static void Test_WinRtString(void)
{
    HRESULT (WINAPI *pCreate)(LPCWSTR, UINT32, HSTRING_T *) =
        (void *)Get(L"api-ms-win-core-winrt-string-l1-1-0.dll", "WindowsCreateString");
    HRESULT (WINAPI *pDelete)(HSTRING_T) =
        (void *)Get(L"api-ms-win-core-winrt-string-l1-1-0.dll", "WindowsDeleteString");
    HRESULT (WINAPI *pConcat)(HSTRING_T, HSTRING_T, HSTRING_T *) =
        (void *)Get(L"api-ms-win-core-winrt-string-l1-1-0.dll", "WindowsConcatString");
    HRESULT (WINAPI *pCompare)(HSTRING_T, HSTRING_T, INT32 *) =
        (void *)Get(L"api-ms-win-core-winrt-string-l1-1-0.dll", "WindowsCompareStringOrdinal");
    UINT32 (WINAPI *pLen)(HSTRING_T) =
        (void *)Get(L"api-ms-win-core-winrt-string-l1-1-0.dll", "WindowsGetStringLen");
    LPCWSTR (WINAPI *pRaw)(HSTRING_T, UINT32 *) =
        (void *)Get(L"api-ms-win-core-winrt-string-l1-1-0.dll", "WindowsGetStringRawBuffer");
    HRESULT (WINAPI *pRoInit)(int) = (void *)Get(L"api-ms-win-core-winrt-l1-1-0.dll", "RoInitialize");
    HRESULT (WINAPI *pFactory)(HSTRING_T, const GUID *, void **) =
        (void *)Get(L"api-ms-win-core-winrt-l1-1-0.dll", "RoGetActivationFactory");
    VOID (WINAPI *pRoUninit)(void) = (void *)Get(L"api-ms-win-core-winrt-l1-1-0.dll", "RoUninitialize");
    HSTRING_T A = NULL, B = NULL, C = NULL, Bogus = NULL;
    INT32 Order = 99;
    UINT32 Len = 0;
    void *Factory = (void *)1;
    HRESULT hr;

    if (!pCreate || !pDelete || !pConcat || !pCompare || !pLen || !pRaw)
        return;
    ok(pCreate(L"vibe", 4, &A) == S_OK && A, "create A\n");
    ok(pCreate(L"os", 2, &B) == S_OK && B, "create B\n");
    ok(pConcat(A, B, &C) == S_OK, "concat\n");
    ok(pLen(C) == 6, "concat length %u\n", pLen(C));
    ok(!lstrcmpW(pRaw(C, &Len), L"vibeos") && Len == 6, "concat text %S\n", pRaw(C, &Len));
    ok(pCompare(A, B, &Order) == S_OK && Order > 0, "compare vibe/os: %d\n", Order);
    ok(pLen(NULL) == 0 && pRaw(NULL, &Len) && Len == 0, "NULL is the empty string\n");
    ok(pCreate(L"", 0, &Bogus) == S_OK && Bogus == NULL, "empty string is NULL\n");

    if (pRoInit && pFactory && pRoUninit)
    {
        hr = pRoInit(1 /* RO_INIT_MULTITHREADED */);
        ok(hr == S_OK || hr == S_FALSE, "RoInitialize: %08lx\n", hr);
        pCreate(L"VibeOS.No.Such.Class", 20, &Bogus);
        hr = pFactory(Bogus, &IID_IUnknown_T, &Factory);
        ok(hr == REGDB_E_CLASSNOTREG, "unknown class: %08lx\n", hr);
        ok(Factory == NULL, "factory %p\n", Factory);
        pDelete(Bogus);
        pRoUninit();
    }
    pDelete(C);
    pDelete(B);
    pDelete(A);
}

static void Test_Dpi(void)
{
    HRESULT (WINAPI *pGetDpi)(HMONITOR, int, UINT *, UINT *) =
        (void *)Get(L"api-ms-win-shcore-scaling-l1-1-1.dll", "GetDpiForMonitor");
    HMONITOR Mon = MonitorFromPoint((POINT){0, 0}, MONITOR_DEFAULTTOPRIMARY);
    UINT X = 0, Y = 0;
    HDC Dc;

    if (!pGetDpi)
        return;
    Dc = GetDC(NULL);
    ok(pGetDpi(Mon, 0 /* MDT_EFFECTIVE_DPI */, &X, &Y) == S_OK, "GetDpiForMonitor failed\n");
    ok(X == (UINT)GetDeviceCaps(Dc, LOGPIXELSX) && Y == (UINT)GetDeviceCaps(Dc, LOGPIXELSY),
       "dpi %u x %u\n", X, Y);
    ReleaseDC(NULL, Dc);
    /* Windows 11's answers (not E_INVALIDARG, which is Wine's). */
    ok(pGetDpi(Mon, 3, &X, &Y) == HRESULT_FROM_WIN32(ERROR_BAD_ARGUMENTS), "bad type: %08lx\n", pGetDpi(Mon, 3, &X, &Y));
    ok(pGetDpi(Mon, 0, NULL, &Y) == HRESULT_FROM_WIN32(ERROR_INVALID_ADDRESS), "NULL out: %08lx\n", pGetDpi(Mon, 0, NULL, &Y));
}

START_TEST(Win8Surface)
{
    Test_CompareObjectHandles();
    Test_FileInfoByHandle();
    Test_ThreadAndTime();
    Test_WinRtString();
    Test_Dpi();
}
