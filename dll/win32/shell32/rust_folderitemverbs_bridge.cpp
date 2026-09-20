/*
 * PROJECT:     shell32
 * LICENSE:     LGPL-2.1+ (https://spdx.org/licenses/LGPL-2.1+)
 * PURPOSE:     EXTERN_C bridges letting the Rust CFolderItem::Verbs construct the
 *              still-C++ CFolderItemVerbs.
 *
 * CFolder.cpp and CFolderItems.cpp are Rust now; CFolderItemVerbs.cpp is not, because
 * rustconv cannot yet map either of the two things that class needs: its activation is
 * `new CComObject<T>` (the only internal shape the deterministic layer maps is
 * ShellObjectCreator/ShellObjectCreatorInit/_CreatorClass::CreateInstance), and its
 * sibling CFolderItemVerb carries `CComBSTR m_name`, which is not proven scalar/pointer
 * state. So `CFolderItem::Verbs` — which is Rust — still has to build a C++ object.
 * rustconv did the right thing and kept the four operations symbolic rather than
 * inventing a lowering; these are those four symbols. When CFolderItemVerb(s) translate,
 * this file is deleted and the Rust provides them directly under the same names.
 *
 * WHY EVERY ENTRY POINT TRAFFICS IN `FolderItemVerbs*`, NOT `CFolderItemVerbs*`.
 * The C++ this replaces ends with `*ppfic = verbs;`, an IMPLICIT upcast from the class to
 * its COM interface. CFolderItemVerbs inherits CComCoClass + CComObjectRootEx (data, no
 * vptr) before IDispatchImpl<FolderItemVerbs> (which carries the vptr), so that upcast
 * ADJUSTS the pointer by a non-zero offset. The Rust caller has a single value that it
 * both passes back to us and stores into *ppfic, so the value crossing the ABI must be
 * the already-adjusted interface pointer. Each bridge therefore `static_cast`s back down
 * to the class, which reverses the same offset — computed by the compiler from the real
 * layout rather than assumed here. Handing out the raw class pointer instead would store
 * a FolderItemVerbs* that is off by that offset: every vtable call through it lands in
 * the wrong slot, and nothing about it looks wrong at the call site.
 */

#include "precomp.h"
#include "CFolderItemVerbs.h"

EXTERN_C FolderItemVerbs *
CFolderItemVerbs_new(void)
{
    return static_cast<FolderItemVerbs *>(new CComObject<CFolderItemVerbs>());
}

EXTERN_C void
CFolderItemVerbs_delete(FolderItemVerbs *pVerbs)
{
    /* ~CFolderItemVerbs is virtual, so deleting through the class pointer runs the
     * most-derived destructor — this mirrors the C++ `delete verbs;` exactly. */
    delete static_cast<CFolderItemVerbs *>(pVerbs);
}

EXTERN_C HRESULT
CFolderItemVerbs_Init_body(FolderItemVerbs *pVerbs, LPITEMIDLIST idlist)
{
    return static_cast<CFolderItemVerbs *>(pVerbs)->Init(idlist);
}

EXTERN_C ULONG
CFolderItemVerbs_AddRef_body(FolderItemVerbs *pVerbs)
{
    return static_cast<CFolderItemVerbs *>(pVerbs)->AddRef();
}
