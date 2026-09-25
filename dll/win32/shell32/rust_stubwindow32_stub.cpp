/* INERT since 2026-09-24: not in CMake. This class is Rust now, on the crate's atlwin
 * substrate (sdk/lib/rust/shell32-rust/src/atl_stub_windows.rs). Kept on disk as the
 * reference the port was checked against. */

/* rust_stubwindow32_stub.cpp — the ONE ATL class utils.cpp still needs.
 *
 * `utils.cpp` is otherwise fully translated (shell32-rust/src/shell32/utils.rs) and is out of
 * the CMake source list. What could not come with it is `CStubWindow32`, a `CWindowImpl<>`
 * whose per-instance THUNK and registered window class ATL generates from C++ templates.
 *
 * Same shape and same reason as rust_folder_options_stub.cpp: the class name is load-bearing.
 * `FindStubWindow` looks the window up by `CSTUBWINDOW32_CLASSNAME` ("StubWindow32") plus a
 * "StubType" window property, so an already-open property sheet or format-drive dialog is
 * raised instead of a second one being opened. A Rust reimplementation registering the same
 * name would SPLIT that lookup and the dialog would open twice.
 *
 * It must ALSO stay C++ because the class still has live C++ constructors: propsheet.cpp and
 * folders/CDrivesFolder.cpp both build one directly. An ATL `CWindowImpl`'s thunk and its
 * registered class must have exactly ONE owner (shell32-rust/src/atlwin.rs records that rule),
 * so there cannot be a second Rust implementation while those two exist.
 *
 * The pair is create/destroy and the object outlives the call: the ATL thunk must stay alive
 * as long as the window does, so Rust holds it as an opaque pointer and hands it back. On
 * failure nothing is allocated and *ppStub stays NULL.
 *
 * Everything else utils.cpp exported IS Rust now. Of the 16 `*_RustBridge` wrappers that file
 * carried, 14 are published by utils.rs itself as `#[no_mangle]`; only this pair needed
 * rehoming, because only this pair reaches a C++ class.
 */

#include "precomp.h"

WINE_DEFAULT_DEBUG_CHANNEL(shell);

HWND
CStubWindow32::FindStubWindow(UINT Type, LPCWSTR Path)
{
    for (HWND hWnd, hWndAfter = NULL;;)
    {
        hWnd = hWndAfter = FindWindowExW(NULL, hWndAfter, CSTUBWINDOW32_CLASSNAME, Path);
        if (!hWnd || !Path)
            return NULL;
        if (GetPropW(hWnd, GetTypePropName()) == ULongToHandle(Type))
            return hWnd;
    }
}

HRESULT
CStubWindow32::CreateStub(UINT Type, LPCWSTR Path, const POINT *pPt)
{
    if (HWND hWnd = FindStubWindow(Type, Path))
    {
        ::SwitchToThisWindow(::GetLastActivePopup(hWnd), TRUE);
        return HRESULT_FROM_WIN32(ERROR_ALREADY_EXISTS);
    }
    RECT rcPosition = { pPt ? pPt->x : CW_USEDEFAULT, pPt ? pPt->y : CW_USEDEFAULT, 0, 0 };
    DWORD Style = WS_DISABLED | WS_CLIPSIBLINGS | WS_CAPTION;
    DWORD ExStyle = WS_EX_WINDOWEDGE | WS_EX_APPWINDOW;
    if (!Create(NULL, rcPosition, Path, Style, ExStyle))
    {
        ERR("StubWindow32 creation failed\n");
        return E_FAIL;
    }
    ::SetPropW(*this, GetTypePropName(), ULongToHandle(Type));
    return S_OK;
}

extern "C" HRESULT WINAPI
CStubWindow32_CreateStub_RustBridge(
    UINT Type, LPCWSTR Path, const POINT *pPt, HWND *phWnd, void **ppStub)
{
    *phWnd = NULL;
    *ppStub = NULL;
    CStubWindow32 *pStub = new CStubWindow32();
    if (!pStub)
        return E_OUTOFMEMORY;
    HRESULT hr = pStub->CreateStub(Type, Path, pPt);
    if (FAILED(hr))
    {
        delete pStub;
        return hr;
    }
    *phWnd = *pStub;
    *ppStub = pStub;
    return hr;
}

extern "C" void WINAPI
CStubWindow32_DestroyStub_RustBridge(void *pStub)
{
    CStubWindow32 *p = static_cast<CStubWindow32 *>(pStub);
    if (!p)
        return;
    if (p->m_hWnd)
        p->DestroyWindow();
    delete p;
}
