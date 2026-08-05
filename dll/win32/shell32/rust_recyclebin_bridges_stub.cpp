/* rust_recyclebin_bridges_stub.cpp — the three bridges folders/CRecycleBin.cpp could not take
 * with it when it flipped to Rust (shell32-rust/src/folders/CRecycleBin.rs).
 *
 * The other two bridges that block lived beside — CRecycleBinEnum_CreateInstance_RustBridge and
 * CRecycleBinItemContextMenu_CreateInstance_RustBridge — are gone from C++ entirely: both
 * classes are now Rust (folders/CRecycleBinEnum.rs, folders/CRecycleBinItemContextMenu.rs) and
 * the Rust TU slice publishes those two symbols itself. What is left here is what still has a
 * C++ implementation on the other side of the call:
 *
 *   - CRecyclerDropTarget_CreateInstance is defined in droptargets/CRecyclerDropTarget.cpp,
 *     which is still C++, and is declared in folders/CRecycleBin.h WITHOUT EXTERN_C, so its
 *     symbol is C++-mangled and Rust cannot name it. A one-line forwarder is cheaper and more
 *     honest than changing the linkage of a function whose only caller is this forwarder.
 *
 *   - CRecycleBinFolderViewCB is a C++ class in a TU that CANNOT be bought: the deterministic
 *     scaffold preflight refuses it twice over (no OBJECT_ENTRY activation, and a
 *     `CComHeapPtr<ITEMIDLIST> m_pidls[2]` whose lifetime has no mapping). Its construction
 *     goes through ShellObjectCreator<T>, an ATL template — there is no Rust spelling for
 *     either the template or the class, so both the create and the Initialize call stay here.
 *
 * ONE BEHAVIOURAL FIX came with the move, and it is the reason Initialize's signature changed.
 * The old bridge did `static_cast<CRecycleBin *>(psf)` to turn the IShellFolder2 pointer back
 * into a CRecycleBin*, then called a C++ method on it. CRecycleBin has been Rust-owned since
 * recycle_bin.rs landed, so that cast applied the C++ class's base-subobject offset to an
 * object laid out by Rust, and the method it reached was the DEAD C++ copy in CRecycleBin.cpp
 * — which is exactly the copy that has now left. It linked, and it was already wrong. The
 * callback now holds the IShellFolder2 it was actually given and calls the Rust implementation
 * through CRecycleBin_ParseRecycleBinPath_RustBridge, which does its own offset arithmetic
 * against the Rust layout. No cast, no second implementation.
 */

#include "precomp.h"

WINE_DEFAULT_DEBUG_CHANNEL(shell);

extern "C" HRESULT WINAPI
CRecyclerDropTarget_CreateInstance_RustBridge(REFIID riid, LPVOID *ppvOut)
{
    return CRecyclerDropTarget_CreateInstance(riid, ppvOut);
}

extern "C" HRESULT WINAPI
CRecycleBinFolderViewCB_CreateInstance_RustBridge(
    IShellFolder2 *psf, IShellFolderViewCB **ppvcb)
{
    UNREFERENCED_PARAMETER(psf);
    CComPtr<CRecycleBinFolderViewCB> sfviewcb;
    HRESULT hr = ShellObjectCreator(sfviewcb);
    if (FAILED(hr))
        return hr;
    // Transfer the (AddRef'd) reference to the caller; Rust releases it.
    *ppvcb = static_cast<IShellFolderViewCB *>(sfviewcb.Detach());
    return S_OK;
}

extern "C" HRESULT WINAPI
CRecycleBinFolderViewCB_Initialize_RustBridge(
    IShellFolderViewCB *pvcb, IShellFolder2 *psf, IShellView *psv, LPCITEMIDLIST pidl)
{
    CRecycleBinFolderViewCB *pViewCB = static_cast<CRecycleBinFolderViewCB *>(pvcb);
    pViewCB->Initialize(psf, psv, pidl);
    return S_OK;
}
