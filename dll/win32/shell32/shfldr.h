
/*
 *  Virtual Folder
 *  common definitions
 *
 *  Copyright 1997 Marcus Meissner
 *  Copyright 1998, 1999, 2002 Juergen Schmied
 *
 * This library is free software; you can redistribute it and/or
 * modify it under the terms of the GNU Lesser General Public
 * License as published by the Free Software Foundation; either
 * version 2.1 of the License, or (at your option) any later version.
 *
 * This library is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU
 * Lesser General Public License for more details.
 *
 * You should have received a copy of the GNU Lesser General Public
 * License along with this library; if not, write to the Free Software
 * Foundation, Inc., 51 Franklin St, Fifth Floor, Boston, MA 02110-1301, USA
 */

#ifndef _SHFLDR_H_
#define _SHFLDR_H_
#include <ntquery.h> // For PID_STG_*

#define CHARS_IN_GUID 39

typedef struct {
    WORD colnameid; // Column title text resource id passed to LoadString
    WORD colstate;  // SHCOLSTATEF returned by IShellFolder2::GetDefaultColumnState
                    // HACK: SHCOLSTATEF truncated to WORD to reduce .rdata section size
    WORD fmt;       // LVCFMT_*
    WORD cxChar;    // Column width hint
} shvheader;
/* 
 * CFSFolder column indices. CDesktopFolder MUST use the same indices!
 * According to the documentation for IShellFolder2::GetDetailsOf,
 * the first 4 columns for SFGAO_FILESYSTEM items must be Name, Size, Type, Modified date
For Details See:
https://learn.microsoft.com/en-us/windows/win32/api/shobjidl_core/nf-shobjidl_core-ishellfolder2-getdetailsof
 */
#define SHFSF_COL_NAME          0
#define SHFSF_COL_SIZE          1
#define SHFSF_COL_TYPE          2 // SHGFI_TYPENAME
#define SHFSF_COL_MDATE         3 // Modified date
#define SHFSF_COL_FATTS         4 // File attributes
#define SHFSF_COL_COMMENT       5

typedef struct _REQUIREDREGITEM
{
    REFCLSID clsid;
    LPCSTR pszCpl;
    BYTE Order; // According to Geoff Chappell, required items have a fixed sort order
} REQUIREDREGITEM;

typedef struct _REGFOLDERINFO
{
    PIDLTYPE PidlType;
    BYTE Count; // Count of required items
    const REQUIREDREGITEM *Items;
    REFCLSID clsid;
    LPCWSTR pszParsingPath;
    LPCWSTR pszEnumKeyName;
} REGFOLDERINFO;

typedef struct _REGFOLDERINITDATA
{
    IShellFolder *psfOuter;
    const REGFOLDERINFO *pInfo;
} REGFOLDERINITDATA, *PREGFOLDERINITDATA;

/* EXTERN_C because folders/CDesktopFolder.cpp's Rust replacement calls this directly. Without
   it the symbol is C++-mangled (__Z25CRegFolder_CreateInstance...) and no Rust declaration can
   name it; the header is seen by both CRegFolder.cpp's definition and every caller, so the
   linkage flips consistently. Same route the CFSFolder flip took for its two free helpers. */
EXTERN_C HRESULT CRegFolder_CreateInstance(PREGFOLDERINITDATA pInit, LPCITEMIDLIST pidlRoot, REFIID riid, void **ppv);

#define GET_SHGDN_FOR(dwFlags)         ((DWORD)dwFlags & (DWORD)0x0000FF00)
#define GET_SHGDN_RELATION(dwFlags)    ((DWORD)dwFlags & (DWORD)0x000000FF)
#define IS_SHGDN_FOR_PARSING(flags) ( ((flags) & (SHGDN_FORADDRESSBAR | SHGDN_FORPARSING)) == SHGDN_FORPARSING)
#define IS_SHGDN_DESKTOPABSOLUTEPARSING(flags) ( ((flags) & (SHGDN_FORADDRESSBAR | SHGDN_FORPARSING | 0xFF)) == SHGDN_FORPARSING)

static inline SFGAOF 
SHELL_CreateFolderEnumItemAttributeQuery(SHCONTF Flags, BOOL ForRegItem)
{
    SFGAOF query = SFGAO_FOLDER | (ForRegItem ? SFGAO_NONENUMERATED : 0);
    if (!(Flags & SHCONTF_INCLUDEHIDDEN))
        query |= SFGAO_HIDDEN;
    if (!(Flags & SHCONTF_INCLUDESUPERHIDDEN))
        query |= SFGAO_HIDDEN | SFGAO_SYSTEM;
    return query;
}

EXTERN_C SHCONTF
SHELL_GetDefaultFolderEnumSHCONTF();

EXTERN_C BOOL
SHELL_IncludeItemInFolderEnum(IShellFolder *pSF, PCUITEMID_CHILD pidl, SFGAOF Query, SHCONTF Flags);

static inline HRESULT
MakeSCID(SHCOLUMNID &scid, REFCLSID fmtid, UINT pid)
{
    scid.fmtid = fmtid;
    scid.pid = pid;
    return S_OK;
}

EXTERN_C HRESULT
SHELL_MapSCIDToColumn(IShellFolder2 *pSF, const SHCOLUMNID *pscid);
HRESULT
SHELL_GetDetailsOfAsStringVariant(IShellFolder *pSF, PCUITEMID_CHILD pidl, UINT Column, VARIANT *pVar);
// EXTERN_C: this exact overload (IShellFolder2*) is exported unmangled by Rust
// (shell32-rust/src/shell32/shlfolder.rs, #[no_mangle] extern "C" fn). Do not mangle it.
EXTERN_C HRESULT
SHELL_GetDetailsOfAsStringVariant(IShellFolder2 *pSF, PCUITEMID_CHILD pidl, UINT Column, VARIANT *pVar);
EXTERN_C HRESULT
SHELL_GetDetailsOfColumnAsVariant(IShellFolder2 *pSF, PCUITEMID_CHILD pidl, UINT Column, VARTYPE vt, VARIANT *pVar);
EXTERN_C HRESULT
SH32_GetDetailsOfPKeyAsVariant(IShellFolder2 *pSF, PCUITEMID_CHILD pidl, const SHCOLUMNID *pscid, VARIANT *pVar, BOOL UseFsColMap);

HRESULT
SHELL_QueryInfoTipAlloc(_In_ IShellFolder* pSF, _In_ DWORD dwFlags, _In_ LPCITEMIDLIST pidl, _Out_ PWSTR *ppTip);

// EXTERN_C: exported unmangled by Rust (shell32-rust/src/shell32/shlfolder.rs).
EXTERN_C HRESULT
SHELL_CreateAbsolutePidl(IShellFolder *pSF, PCUIDLIST_RELATIVE pidlChild, PIDLIST_ABSOLUTE *ppPidl);

EXTERN_C HRESULT
Shell_NextElement(
    _Inout_ LPWSTR *ppch,
    _Out_ LPWSTR pszOut,
    _In_ INT cchOut,
    _In_ BOOL bValidate);

EXTERN_C HRESULT SHELL32_ParseNextElement (IShellFolder2 * psf, HWND hwndOwner, LPBC pbc, LPITEMIDLIST * pidlInOut,
                  LPOLESTR szNext, DWORD * pEaten, DWORD * pdwAttributes);

EXTERN_C HRESULT SHELL32_GetDisplayNameOfChild (IShellFolder2 * psf, LPCITEMIDLIST pidl, DWORD dwFlags, LPSTRRET strRet);

HRESULT SHELL32_GetFSItemAttributes(IShellFolder * psf, LPCITEMIDLIST pidl, LPDWORD pdwAttributes);

EXTERN_C HRESULT WINAPI SHELL32_CompareDetails(IShellFolder2* isf, LPARAM lParam, LPCITEMIDLIST pidl1, LPCITEMIDLIST pidl2);

EXTERN_C HRESULT SHELL32_CompareChildren(IShellFolder2* psf, LPARAM lParam, LPCITEMIDLIST pidl1, LPCITEMIDLIST pidl2);

EXTERN_C HRESULT SHELL32_CoCreateInitSF (LPCITEMIDLIST pidlRoot, PERSIST_FOLDER_TARGET_INFO* ppfti,
                LPCITEMIDLIST pidlChild, const GUID* clsid, REFIID riid, LPVOID *ppvOut);

/* Was a second overload of SHELL32_CoCreateInitSF. EXTERN_C maps one name to one symbol and
 * therefore cannot carry an overload set, so the csidl form needs a name of its own now that
 * the definition is Rust. Fills a PERSIST_FOLDER_TARGET_INFO from csidl and delegates. */
EXTERN_C HRESULT SHELL32_CoCreateInitSFWithCsidl (LPCITEMIDLIST pidlRoot, const GUID* clsid,
                                int csidl, REFIID riid, LPVOID *ppvOut);

EXTERN_C HRESULT SHELL32_BindToSF (LPCITEMIDLIST pidlRoot, PERSIST_FOLDER_TARGET_INFO* ppfti,
                LPCITEMIDLIST pidl, const GUID* clsid, REFIID riid, LPVOID *ppvOut);

extern "C"
BOOL HCR_RegOpenClassIDKey(REFIID riid, HKEY *hkey);

/* EXTERN_C: the class is Rust-owned (shell32-rust/src/shell32/CDefViewBckgrndMenu.rs) and
 * publishes this factory as an unmangled cdecl symbol, so the still-C++ CDefView.cpp needs the
 * declaration to name it rather than a C++-mangled one. Arity and order match the C exactly. */
EXTERN_C HRESULT CDefViewBckgrndMenu_CreateInstance(IShellFolder* psf, REFIID riid, void **ppv);

EXTERN_C HRESULT SH_GetApidlFromDataObject(IDataObject *pDataObject, PIDLIST_ABSOLUTE* ppidlfolder, PUITEMID_CHILD **apidlItems, UINT *pcidl);

static __inline int SHELL32_GUIDToStringA (REFGUID guid, LPSTR str)
{
    return sprintf(str, "{%08X-%04X-%04X-%02X%02X-%02X%02X%02X%02X%02X%02X}",
            guid.Data1, guid.Data2, guid.Data3,
            guid.Data4[0], guid.Data4[1], guid.Data4[2], guid.Data4[3],
            guid.Data4[4], guid.Data4[5], guid.Data4[6], guid.Data4[7]);
}

static __inline int SHELL32_GUIDToStringW (REFGUID guid, LPWSTR str)
{
    static const WCHAR fmtW[] =
     { '{','%','0','8','l','X','-','%','0','4','X','-','%','0','4','X','-',
     '%','0','2','X','%','0','2','X','-',
     '%','0','2','X','%','0','2','X','%','0','2','X','%','0','2','X',
     '%','0','2','X','%','0','2','X','}',0 };
    return _swprintf(str, fmtW,
            guid.Data1, guid.Data2, guid.Data3,
            guid.Data4[0], guid.Data4[1], guid.Data4[2], guid.Data4[3],
            guid.Data4[4], guid.Data4[5], guid.Data4[6], guid.Data4[7]);
}

/* EXTERN_C: both are now defined in Rust (shell32-rust/src/cfs_folder_tu.rs), which is where the
 * rest of folders/CFSFolder.cpp went. Plain __cdecl — no WINAPI on either declaration. Without
 * this the C++ callers (CDefView.cpp, CRecycleBin.cpp) emit a mangled name nothing defines. */
EXTERN_C void SHELL_FS_ProcessDisplayFilename(LPWSTR szPath, DWORD dwFlags);
EXTERN_C BOOL SHELL_FS_HideExtension(LPCWSTR pwszPath);

static inline BOOL IsIllegalFsFileName(PCWSTR Name)
{
    return StrIsNullOrEmpty(Name) || StrPBrkW(Name, INVALID_FILETITLE_CHARACTERSW);
}

EXTERN_C void CloseRegKeyArray(HKEY* array, UINT cKeys);
EXTERN_C LSTATUS AddClassKeyToArray(const WCHAR* szClass, HKEY* array, UINT* cKeys);
EXTERN_C LSTATUS AddClsidKeyToArray(REFCLSID clsid, HKEY* array, UINT* cKeys);
EXTERN_C void AddFSClassKeysToArray(UINT cidl, PCUITEMID_CHILD_ARRAY apidl, HKEY* array, UINT* cKeys);
EXTERN_C void AddPidlClassKeysToArray(LPCITEMIDLIST pidl, HKEY* array, UINT* cKeys);

#ifdef __cplusplus

struct CRegKeyHandleArray
{
    HKEY hKeys[16];
    UINT cKeys;

    CRegKeyHandleArray() : cKeys(0) {}
    ~CRegKeyHandleArray() { CloseRegKeyArray(hKeys, cKeys); }
    operator HKEY*() { return hKeys; }
    operator UINT*() { return &cKeys; }
    operator UINT() { return cKeys; }
    HKEY& operator [](SIZE_T i) { return hKeys[i]; }
};

HRESULT inline SHSetStrRet(LPSTRRET pStrRet, DWORD resId)
{
    return SHSetStrRet(pStrRet, shell32_hInstance, resId);
}

#endif

// EXTERN_C: the definition is Rust now (shell32-rust/src/dialogs/general.rs), which emits the
// plain symbol. Without this the C++ callers reference a mangled name that nothing defines.
EXTERN_C UINT SHELL_GetIconUnderlineFlags();

#endif /* _SHFLDR_H_ */
