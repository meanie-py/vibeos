/*
 * PROJECT:     shell32
 * LICENSE:     LGPL-2.1-or-later (https://spdx.org/licenses/LGPL-2.1-or-later)
 * PURPOSE:     Recycle Bin virtual folder callback
 * COPYRIGHT:   Copyright 2026 Katayama Hirofumi MZ (katayama.hirofumi.mz@gmail.com)
 */

#pragma once

class CRecycleBinFolderViewCB
    : public CComObjectRootEx<CComMultiThreadModelNoCS>
    , public IShellFolderViewCB
{
    IShellView *m_pShellView; // Not ref-counted!
    /* The folder is Rust-owned (shell32-rust/src/recycle_bin.rs), so this is the interface
     * pointer we were handed and NOT a CRecycleBin*: a static_cast back to the C++ class would
     * apply that class's base-subobject offset to an object Rust laid out. Its one non-virtual
     * method is reached through CRecycleBin_ParseRecycleBinPath_RustBridge. */
    IShellFolder2 *m_pRecycleBin;
    ULONG m_nChangeNotif; // Change notification handle
    CComHeapPtr<ITEMIDLIST> m_pidlParent;
    CComHeapPtr<ITEMIDLIST> m_pidls[2];

    HRESULT RegisterChangeNotify(HWND hwndView);
    HRESULT TranslatePidl(LPITEMIDLIST *ppidlNew, LPCITEMIDLIST pidl);
    void TranslateTwoPIDLs(PIDLIST_ABSOLUTE* pidls);

public:
    CRecycleBinFolderViewCB();
    virtual ~CRecycleBinFolderViewCB();
    void Initialize(IShellFolder2 *pRecycleBin, IShellView *psv, LPCITEMIDLIST pidlParent);

    // IShellFolderViewCB
    STDMETHODIMP MessageSFVCB(UINT uMsg, WPARAM wParam, LPARAM lParam) override;

    DECLARE_NO_REGISTRY()
    DECLARE_NOT_AGGREGATABLE(CRecycleBinFolderViewCB)
    BEGIN_COM_MAP(CRecycleBinFolderViewCB)
        COM_INTERFACE_ENTRY_IID(IID_IShellFolderViewCB, IShellFolderViewCB)
    END_COM_MAP()
};
