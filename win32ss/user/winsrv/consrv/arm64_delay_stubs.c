/*
 * Stubs for shell32/ole32 functions that are delay-imported on x86/amd64
 * but cannot be delay-imported on ARM64 (llvm-dlltool limitation).
 * These functions are used only for GUI console clipboard/drag-drop
 * features, which are not exercised during ARM64 boot.
 */

#ifdef _M_ARM64

#include <windows.h>

/* ole32 stubs */
HRESULT WINAPI CoCreateInstance(
    REFCLSID rclsid, LPUNKNOWN pUnkOuter, DWORD dwClsContext,
    REFIID riid, LPVOID *ppv)
{
    (void)rclsid; (void)pUnkOuter; (void)dwClsContext; (void)riid;
    if (ppv) *ppv = NULL;
    return E_NOTIMPL;
}

HRESULT WINAPI CoInitialize(LPVOID pvReserved)
{
    (void)pvReserved;
    return S_OK;
}

void WINAPI CoUninitialize(void)
{
}

/* shell32 stubs */
void WINAPI DragAcceptFiles(HWND hWnd, BOOL fAccept)
{
    (void)hWnd; (void)fAccept;
}

void WINAPI DragFinish(HDROP hDrop)
{
    (void)hDrop;
}

UINT WINAPI DragQueryFileW(HDROP hDrop, UINT iFile, LPWSTR lpszFile, UINT cch)
{
    (void)hDrop; (void)iFile; (void)lpszFile; (void)cch;
    return 0;
}

#endif /* _M_ARM64 */
