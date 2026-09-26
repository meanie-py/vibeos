/* INERT since 2026-09-24: not in CMake. This class is Rust now, on the crate's atlwin
 * substrate (sdk/lib/rust/shell32-rust/src/atl_stub_windows.rs). Kept on disk as the
 * reference the port was checked against. */

/* rust_folder_options_stub.cpp — the ONE ATL class dialogs/folder_options.cpp still needs.
 *
 * `dialogs/folder_options.cpp` is otherwise fully translated (shell32-rust/src/dialogs/
 * folder_options.rs) and is out of the CMake source list. What could not come with it is
 * `CMSGlobalFolderOptionsStub`, a `CWindowImpl<>` whose per-instance THUNK and registered
 * window class ATL generates from C++ templates — there is no Rust spelling for either.
 *
 * Same shape as the `CStubWindow32` bridge in utils.cpp, and for the same reason: the class
 * name is load-bearing. `ShowFolderOptionsDialog` calls `FindWindow(L"MSGlobalFolderOptions
 * Stub", NULL)` to raise an already-open Folder Options instead of opening a second one, so a
 * Rust reimplementation that registered the same name would SPLIT that lookup and the dialog
 * would open twice.
 *
 * The pair is create/destroy and the object outlives the call: the ATL thunk must stay alive
 * as long as the window does, so Rust holds it as an opaque pointer and hands it back. On
 * failure nothing is allocated and *ppStub stays NULL.
 *
 * OWED (deliberately not done here): the class has an EMPTY message map, so every message
 * already reaches DefWindowProc and the thunk routes nothing. That makes a ~30-line Rust
 * RegisterClassExW/CreateWindowExW reimplementation plausible — but "plausible" is not
 * "proven equivalent" for the class style, the `(HBRUSH)(bkgnd + 1)` background ATL derives,
 * and CW_USEDEFAULT geometry, and this file is cheap. Retire it against atlwin.h, not against
 * a reading of this comment.
 */

#include "precomp.h"

#define GlobalFolderOptionsClassName _T("MSGlobalFolderOptionsStub")

// CMSGlobalFolderOptionsStub --- The owner window of Folder Options.
// This window hides taskbar button of Folder Options.
class CMSGlobalFolderOptionsStub : public CWindowImpl<CMSGlobalFolderOptionsStub>
{
public:
    DECLARE_WND_CLASS_EX(GlobalFolderOptionsClassName, 0, COLOR_WINDOWTEXT)

    BEGIN_MSG_MAP(CMSGlobalFolderOptionsStub)
    END_MSG_MAP()
};

extern "C" HWND WINAPI
CMSGlobalFolderOptionsStub_Create_RustBridge(DWORD dwStyle, DWORD dwExStyle, void **ppStub)
{
    *ppStub = NULL;
    CMSGlobalFolderOptionsStub *pStub = new CMSGlobalFolderOptionsStub();
    if (!pStub)
        return NULL;

    HWND hWnd = pStub->Create(NULL, NULL, NULL, dwStyle, dwExStyle);
    if (!hWnd)
    {
        delete pStub;
        return NULL;
    }
    *ppStub = pStub;
    return hWnd;
}

extern "C" VOID WINAPI
CMSGlobalFolderOptionsStub_Destroy_RustBridge(void *pStub)
{
    CMSGlobalFolderOptionsStub *p = static_cast<CMSGlobalFolderOptionsStub *>(pStub);
    if (!p)
        return;
    if (p->m_hWnd)
        p->DestroyWindow();
    delete p;
}
