/*
 * CMenuBand is Rust-owned (sdk/lib/rust/shell32-rust/src/menu_band.rs).
 *
 * OPAQUE-POINTER CONVENTION: every `CMenuBand *` the C++ side holds (the toolbars'
 * m_menuBand, the focus manager's StackEntry::mb, the Push/Pop/_SetChildBand/
 * _SetParentBand parameters) is the Rust object's base pointer, exactly as handed out
 * by the Rust band through the CMenuFocusManager_* bridges and the toolbar Create
 * bridges. It is OPAQUE: never dereference it and never cast it to the retired C++
 * class layout — every operation goes through the extern "C" exports below, which
 * recover the object with a direct cast (no offset adjustment). The retired class
 * declaration (CMenuBand.h) must NOT be included by TUs that talk to the band; the
 * forward declaration below is the whole type.
 *
 * NAMING RULE: export = CMenuBand_ + method name with its leading underscore stripped
 * + _RustExport (method `_ChangeHotItem` -> CMenuBand_ChangeHotItem_RustExport).
 *
 * LIFETIME RULE: where C++ retains a band pointer it balances
 * CMenuBand_AddRef_RustExport / CMenuBand_Release_RustExport at the exact points the
 * old code called AddRef()/Release() (CMenuFocusManager: AddRef in PushMenuBar/
 * PushMenuPopup, Release in PopMenuBar/PopMenuPopup). These route through the same
 * refcount as the band's published IUnknown. The toolbars' m_menuBand stays an
 * unowned back-pointer, as it always was.
 *
 * Signatures mirror CMenuBand.h (line cited per declaration); the Rust definitions
 * live at the bottom of menu_band.rs.
 */
#pragma once

class CMenuBand;        /* opaque: the Rust object base pointer */
class CMenuToolbarBase; /* C++ toolbar companion; passed through untouched */

extern "C"
{
/* IUnknown refcount (same counter as the published interface slots) */
ULONG WINAPI CMenuBand_AddRef_RustExport(CMenuBand *band);
ULONG WINAPI CMenuBand_Release_RustExport(CMenuBand *band);

/* COM methods invoked directly by the companions */
HRESULT WINAPI CMenuBand_GetSite_RustExport(CMenuBand *band, REFIID riid, PVOID *ppvSite);                                                       /* h:116 */
HRESULT WINAPI CMenuBand_GetMenuInfo_RustExport(CMenuBand *band, IShellMenuCallback **ppsmc, UINT *puId, UINT *puIdAncestor, DWORD *pdwFlags);   /* h:155 */
HRESULT WINAPI CMenuBand_IsWindowOwner_RustExport(CMenuBand *band, HWND hWnd);                                                                   /* h:166 */

/* Public non-COM helpers */
BOOL    WINAPI CMenuBand_UseBigIcons_RustExport(CMenuBand *band);                                                                               /* h:207 */
HRESULT WINAPI CMenuBand_AdjustForTheme_RustExport(CMenuBand *band, BOOL bFlatStyle);                                                           /* h:205 */
HRESULT WINAPI CMenuBand_CallCBWithItemId_RustExport(CMenuBand *band, UINT Id, UINT uMsg, WPARAM wParam, LPARAM lParam);                        /* h:184 */
HRESULT WINAPI CMenuBand_CallCBWithItemPidl_RustExport(CMenuBand *band, LPITEMIDLIST pidl, UINT uMsg, WPARAM wParam, LPARAM lParam);            /* h:185 */
HRESULT WINAPI CMenuBand_TrackSubMenu_RustExport(CMenuBand *band, HMENU popup, INT x, INT y, RECT *rcExclude);                                  /* h:186 — C++ took RECT&; must be non-NULL */
HRESULT WINAPI CMenuBand_TrackContextMenu_RustExport(CMenuBand *band, IContextMenu *popup, INT x, INT y);                                       /* h:187 */
HRESULT WINAPI CMenuBand_GetTopLevelWindow_RustExport(CMenuBand *band, HWND *topLevel);                                                         /* h:188 */
HRESULT WINAPI CMenuBand_ChangeHotItem_RustExport(CMenuBand *band, CMenuToolbarBase *tb, INT id, DWORD dwFlags);                                /* h:189 */
HRESULT WINAPI CMenuBand_ChangePopupItem_RustExport(CMenuBand *band, CMenuToolbarBase *tb, INT id);                                             /* h:190 */
HRESULT WINAPI CMenuBand_MenuItemSelect_RustExport(CMenuBand *band, DWORD changeType);                                                          /* h:191 */
HRESULT WINAPI CMenuBand_CancelCurrentPopup_RustExport(CMenuBand *band);                                                                        /* h:192 */
HRESULT WINAPI CMenuBand_OnPopupSubMenu_RustExport(CMenuBand *band, IShellMenu *childShellMenu, POINTL *pAt, RECTL *pExclude, BOOL keyInitiated); /* h:193 */
HRESULT WINAPI CMenuBand_BeforeCancelPopup_RustExport(CMenuBand *band);                                                                         /* h:194 */
HRESULT WINAPI CMenuBand_DisableMouseTrack_RustExport(CMenuBand *band, BOOL bDisable);                                                          /* h:195 */
HRESULT WINAPI CMenuBand_SetChildBand_RustExport(CMenuBand *band, CMenuBand *child);                                                            /* h:196 */
HRESULT WINAPI CMenuBand_SetParentBand_RustExport(CMenuBand *band, CMenuBand *parent);                                                          /* h:197 */
HRESULT WINAPI CMenuBand_IsPopup_RustExport(CMenuBand *band);                                                                                   /* h:198 */
HRESULT WINAPI CMenuBand_IsTracking_RustExport(CMenuBand *band);                                                                                /* h:199 */
HRESULT WINAPI CMenuBand_KillPopupTimers_RustExport(CMenuBand *band);                                                                           /* h:200 */
HRESULT WINAPI CMenuBand_MenuBarMouseDown_RustExport(CMenuBand *band, HWND hwnd, INT item, BOOL isLButton);                                     /* h:201 */
HRESULT WINAPI CMenuBand_MenuBarMouseUp_RustExport(CMenuBand *band, HWND hwnd, INT item, BOOL isLButton);                                       /* h:202 */
HRESULT WINAPI CMenuBand_HasSubMenu_RustExport(CMenuBand *band);                                                                                /* h:203 */
HRESULT WINAPI CMenuBand_KeyboardItemChange_RustExport(CMenuBand *band, DWORD change);                                                          /* h:213 */
}
