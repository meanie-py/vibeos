/*
 *                 Shell basics
 *
 * Copyright 1998 Marcus Meissner
 * Copyright 1998 Juergen Schmied (jsch)  *  <juergen.schmied@metronet.de>
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

#include "precomp.h"

#include "shell32_version.h"

WINE_DEFAULT_DEBUG_CHANNEL(shell);

/*
 * Implemented
 */
EXTERN_C LPWSTR
WINAPI
AddCommasW(DWORD lValue, LPWSTR lpNumber)
{
    WCHAR szValue[MAX_PATH], szSeparator[8 + 1], szGrouping[10 + 1];
    NUMBERFMTW numFormat;

    GetLocaleInfoW(LOCALE_USER_DEFAULT,
                   LOCALE_STHOUSAND,
                   szSeparator,
                   _countof(szSeparator));

    /* Parse LOCALE_SGROUPING into NUMBERFMTW::Grouping.
     * Accumulate the digit groups left-to-right. If there's no trailing ';0'
     * the last group repeats, so multiply by 10. */
    DWORD dwGrouping = 3;
    if (GetLocaleInfoW(LOCALE_USER_DEFAULT,
                       LOCALE_SGROUPING,
                       szGrouping,
                       _countof(szGrouping)))
    {
        PWSTR p = szGrouping;
        dwGrouping = 0;
        while (*p)
        {
            if (*p >= L'1' && *p <= L'9')
                dwGrouping = dwGrouping * 10 + (*p - L'0');
            p++;
        }
        if (p > szGrouping && *(p - 1) != L'0')
            dwGrouping *= 10;
    }
    numFormat.NumDigits = 0;
    numFormat.LeadingZero = 0;
    numFormat.Grouping = dwGrouping;
    numFormat.lpDecimalSep = szSeparator;
    numFormat.lpThousandSep = szSeparator;
    numFormat.NegativeOrder = 0;

    _swprintf(szValue, L"%lu", lValue);

    if (GetNumberFormatW(LOCALE_USER_DEFAULT,
                         0,
                         szValue,
                         &numFormat,
                         lpNumber,
                         MAX_PATH) != 0)
    {
        return lpNumber;
    }

    wcscpy(lpNumber, szValue);
    return lpNumber;
}

/*
 * Implemented
 */
EXTERN_C BOOL
WINAPI
RegenerateUserEnvironment(LPVOID *lpEnvironment, BOOL bUpdateSelf)
{
    HANDLE hUserToken;
    if (!OpenProcessToken(GetCurrentProcess(), TOKEN_READ | TOKEN_WRITE, &hUserToken))
        return FALSE;

    BOOL bResult = CreateEnvironmentBlock(lpEnvironment, hUserToken, TRUE);
    if (!bResult || !lpEnvironment)
    {
        CloseHandle(hUserToken);
        return FALSE;
    }

    if (bUpdateSelf)
    {
        LPWSTR pszz = (LPWSTR)*lpEnvironment;
        if (!pszz)
            return FALSE;

        while (*pszz)
        {
            size_t cch = wcslen(pszz);
            LPWSTR pchEqual = wcschr(pszz, L'=');
            if (pchEqual)
            {
                CStringW strName(pszz, pchEqual - pszz);
                SetEnvironmentVariableW(strName, pchEqual + 1);
            }
            pszz += cch + 1;
        }
    }

    CloseHandle(hUserToken);

    return bResult;
}

/**************************************************************************
 * Default ClassFactory types
 */
typedef HRESULT (CALLBACK *LPFNCREATEINSTANCE)(IUnknown* pUnkOuter, REFIID riid, LPVOID* ppvObject);
HRESULT IDefClF_fnConstructor(LPFNCREATEINSTANCE lpfnCI, PLONG pcRefDll, const IID *riidInst, IClassFactory **theFactory);


/**************************************************************************
 * Default ClassFactory Implementation
 *
 * SHCreateDefClassObject
 *
 * NOTES
 *  Helper function for dlls without their own classfactory.
 *  A generic classfactory is returned.
 *  When the CreateInstance of the cf is called the callback is executed.
 */

class IDefClFImpl :
    public CComObjectRootEx<CComMultiThreadModelNoCS>,
    public IClassFactory
{
private:
    CLSID                    *rclsid;
    LPFNCREATEINSTANCE        lpfnCI;
    const IID                *riidInst;
    LONG                    *pcRefDll;        /* pointer to refcounter in external dll (ugrrr...) */
public:
    IDefClFImpl();
    HRESULT Initialize(LPFNCREATEINSTANCE lpfnCI, PLONG pcRefDll, const IID *riidInstx);

    // IClassFactory
    STDMETHOD(CreateInstance)(IUnknown * pUnkOuter, REFIID riid, LPVOID *ppvObject) override;
    STDMETHOD(LockServer)(BOOL fLock) override;

BEGIN_COM_MAP(IDefClFImpl)
    COM_INTERFACE_ENTRY_IID(IID_IClassFactory, IClassFactory)
END_COM_MAP()
};

IDefClFImpl::IDefClFImpl()
{
    lpfnCI = NULL;
    riidInst = NULL;
    pcRefDll = NULL;
    rclsid = NULL;
}

HRESULT IDefClFImpl::Initialize(LPFNCREATEINSTANCE lpfnCIx, PLONG pcRefDllx, const IID *riidInstx)
{
    lpfnCI = lpfnCIx;
    riidInst = riidInstx;
    pcRefDll = pcRefDllx;

    if (pcRefDll)
        InterlockedIncrement(pcRefDll);

    TRACE("(%p)%s\n", this, shdebugstr_guid(riidInst));
    return S_OK;
}

/******************************************************************************
 * IDefClF_fnCreateInstance
 */
HRESULT WINAPI IDefClFImpl::CreateInstance(IUnknown * pUnkOuter, REFIID riid, LPVOID *ppvObject)
{
    TRACE("%p->(%p,%s,%p)\n", this, pUnkOuter, shdebugstr_guid(&riid), ppvObject);

    *ppvObject = NULL;

    if (riidInst == NULL || IsEqualCLSID(riid, *riidInst) || IsEqualCLSID(riid, IID_IUnknown))
    {
        return lpfnCI(pUnkOuter, riid, ppvObject);
    }

    ERR("unknown IID requested %s\n", shdebugstr_guid(&riid));
    return E_NOINTERFACE;
}

/******************************************************************************
 * IDefClF_fnLockServer
 */
HRESULT WINAPI IDefClFImpl::LockServer(BOOL fLock)
{
    TRACE("%p->(0x%x)\n", this, fLock);

    if (fLock)
    {
        if (pcRefDll)
        {
            if (*pcRefDll == LONG_MAX)
                ERR("pcRefDll is pinned, not incrementing\n");
            else
                InterlockedIncrement(pcRefDll);
        }
        _pAtlModule->Lock();
    }
    else
    {
        _pAtlModule->Unlock();
        if (pcRefDll)
        {
            if (*pcRefDll == LONG_MAX)
                ERR("pcRefDll is pinned, not decrementing\n");
            else if (*pcRefDll > 0)
                InterlockedDecrement(pcRefDll);
            else
                ERR("pcRefDll underflow\n");
        }
    }

    return S_OK;
}

/**************************************************************************
 *  IDefClF_fnConstructor
 */

HRESULT IDefClF_fnConstructor(LPFNCREATEINSTANCE lpfnCI, PLONG pcRefDll, const IID *riidInst, IClassFactory **theFactory)
{
    return ShellObjectCreatorInit<IDefClFImpl>(lpfnCI, pcRefDll, riidInst, IID_PPV_ARG(IClassFactory, theFactory));
}

/******************************************************************************
 * SHCreateDefClassObject            [SHELL32.70]
 */
HRESULT WINAPI SHCreateDefClassObject(
    REFIID    riid,
    LPVOID*    ppv,
    LPFNCREATEINSTANCE lpfnCI,    /* [in] create instance callback entry */
    LPDWORD    pcRefDll,        /* [in/out] ref count of the dll */
    REFIID    riidInst)        /* [in] optional interface to the instance */
{
    IClassFactory                *pcf;
    HRESULT                        hResult;

    TRACE("%s %p %p %p %s\n", shdebugstr_guid(&riid), ppv, lpfnCI, pcRefDll, shdebugstr_guid(&riidInst));

    if (!IsEqualCLSID(riid, IID_IClassFactory))
        return E_NOINTERFACE;
    hResult = IDefClF_fnConstructor(lpfnCI, (PLONG)pcRefDll, &riidInst, &pcf);
    if (FAILED(hResult))
        return hResult;
    *ppv = pcf;
    return S_OK;
}

/**************************************************************************
 *  CStartMenuDummy
 */
class CStartMenuDummy :
    public CComCoClass<CStartMenuDummy, &CLSID_StartMenu>,
    public CComObjectRootEx<CComMultiThreadModelNoCS>
{
private:
    CStartMenuDummy();

protected:
    virtual ~CStartMenuDummy();

public:
    DECLARE_REGISTRY_RESOURCEID(IDR_STARTMENU)

    class _CreatorClass
    {
    public:
        static STDMETHODIMP CreateInstance(void *pv, REFIID riid, LPVOID *ppv)
        {
            if (ppv == NULL)
                return E_POINTER;
            *ppv = NULL;
            if (pv != NULL)
                return CLASS_E_NOAGGREGATION;
            return RSHELL_CStartMenu_CreateInstance(riid, ppv);
        }
    };
};

/**************************************************************************
 *  CShell32Module
 */
class CShell32Module : public CComModule
{
public:
    void Term()
    {
        CComCreatorCentralInstance< ATL::CComObject< CDrivesFolder > >::Term();
        CComCreatorCentralInstance< ATL::CComObject< CDesktopFolder > >::Term();
        CComModule::Term();
    }
};

/* Registration remains ATL-owned for translated classes, but activation is Rust-owned. */
extern "C" HRESULT WINAPI CActiveDesktop_CreateInstance(
    IUnknown *pUnkOuter, REFIID riid, LPVOID *ppv);

class CActiveDesktopRustEntry : public CActiveDesktop
{
public:
    class _CreatorClass
    {
    public:
        static HRESULT WINAPI CreateInstance(void *pv, REFIID riid, LPVOID *ppv)
        {
            return CActiveDesktop_CreateInstance(
                static_cast<IUnknown *>(pv), riid, ppv);
        }
    };
};

extern "C" HRESULT WINAPI CDropTargetHelper_CreateInstance(
    IUnknown *pUnkOuter, REFIID riid, LPVOID *ppv);

class CDropTargetHelperRustEntry : public CDropTargetHelper
{
public:
    class _CreatorClass
    {
    public:
        static HRESULT WINAPI CreateInstance(void *pv, REFIID riid, LPVOID *ppv)
        {
            return CDropTargetHelper_CreateInstance(
                static_cast<IUnknown *>(pv), riid, ppv);
        }
    };
};

extern "C" HRESULT WINAPI COpenControlPanel_CreateInstance(
    IUnknown *pUnkOuter, REFIID riid, LPVOID *ppv);

class COpenControlPanelRustEntry : public COpenControlPanel
{
public:
    class _CreatorClass
    {
    public:
        static HRESULT WINAPI CreateInstance(void *pv, REFIID riid, LPVOID *ppv)
        {
            return COpenControlPanel_CreateInstance(
                static_cast<IUnknown *>(pv), riid, ppv);
        }
    };
};

extern "C" HRESULT WINAPI CExeDropHandler_CreateInstance(
    IUnknown *pUnkOuter, REFIID riid, LPVOID *ppv);

class CExeDropHandlerRustEntry : public CExeDropHandler
{
public:
    class _CreatorClass
    {
    public:
        static HRESULT WINAPI CreateInstance(void *pv, REFIID riid, LPVOID *ppv)
        {
            return CExeDropHandler_CreateInstance(
                static_cast<IUnknown *>(pv), riid, ppv);
        }
    };
};

extern "C" HRESULT WINAPI CCopyAsPathMenu_CreateInstance(
    IUnknown *pUnkOuter, REFIID riid, LPVOID *ppv);

class CCopyAsPathMenuRustEntry : public CCopyAsPathMenu
{
public:
    class _CreatorClass
    {
    public:
        static HRESULT WINAPI CreateInstance(void *pv, REFIID riid, LPVOID *ppv)
        {
            return CCopyAsPathMenu_CreateInstance(
                static_cast<IUnknown *>(pv), riid, ppv);
        }
    };
};

extern "C" HRESULT WINAPI CShellItem_CreateInstance(
    IUnknown *pUnkOuter, REFIID riid, LPVOID *ppv);

class CShellItemRustEntry :
    public CComObjectRootEx<CComMultiThreadModelNoCS>,
    public CComCoClass<CShellItemRustEntry, &CLSID_ShellItem>
{
public:
    DECLARE_NO_REGISTRY()
    class _CreatorClass
    {
    public:
        static HRESULT WINAPI CreateInstance(void *pv, REFIID riid, LPVOID *ppv)
        {
            return CShellItem_CreateInstance(
                static_cast<IUnknown *>(pv), riid, ppv);
        }
    };
};

extern "C" HRESULT WINAPI CFontsFolder_CreateInstance(
    IUnknown *pUnkOuter, REFIID riid, LPVOID *ppv);

class CFontsFolderRustEntry : public CFontsFolder
{
public:
    class _CreatorClass
    {
    public:
        static HRESULT WINAPI CreateInstance(void *pv, REFIID riid, LPVOID *ppv)
        {
            return CFontsFolder_CreateInstance(
                static_cast<IUnknown *>(pv), riid, ppv);
        }
    };
};

extern "C" HRESULT WINAPI CAdminToolsFolder_CreateInstance(
    IUnknown *pUnkOuter, REFIID riid, LPVOID *ppv);

class CAdminToolsFolderRustEntry : public CAdminToolsFolder
{
public:
    class _CreatorClass
    {
    public:
        static HRESULT WINAPI CreateInstance(void *pv, REFIID riid, LPVOID *ppv)
        {
            return CAdminToolsFolder_CreateInstance(
                static_cast<IUnknown *>(pv), riid, ppv);
        }
    };
};

extern "C" HRESULT WINAPI CMyDocsFolder_CreateInstance(
    IUnknown *pUnkOuter, REFIID riid, LPVOID *ppv);

class CMyDocsFolderRustEntry : public CMyDocsFolder
{
public:
    class _CreatorClass
    {
    public:
        static HRESULT WINAPI CreateInstance(void *pv, REFIID riid, LPVOID *ppv)
        {
            return CMyDocsFolder_CreateInstance(
                static_cast<IUnknown *>(pv), riid, ppv);
        }
    };
};

extern "C" HRESULT WINAPI CNewMenu_CreateInstance(
    IUnknown *pUnkOuter, REFIID riid, LPVOID *ppv);

class CNewMenuRustEntry : public CNewMenu
{
public:
    class _CreatorClass
    {
    public:
        static HRESULT WINAPI CreateInstance(void *pv, REFIID riid, LPVOID *ppv)
        {
            return CNewMenu_CreateInstance(
                static_cast<IUnknown *>(pv), riid, ppv);
        }
    };
};

extern "C" HRESULT WINAPI CSendToMenu_CreateInstance(
    IUnknown *pUnkOuter, REFIID riid, LPVOID *ppv);

class CSendToMenuRustEntry : public CSendToMenu
{
public:
    class _CreatorClass
    {
    public:
        static HRESULT WINAPI CreateInstance(void *pv, REFIID riid, LPVOID *ppv)
        {
            return CSendToMenu_CreateInstance(
                static_cast<IUnknown *>(pv), riid, ppv);
        }
    };
};

extern "C" HRESULT WINAPI CPrinterFolder_CreateInstance(
    IUnknown *pUnkOuter, REFIID riid, LPVOID *ppv);

class CPrinterFolderRustEntry :
    public CComObjectRootEx<CComMultiThreadModelNoCS>,
    public CComCoClass<CPrinterFolderRustEntry, &CLSID_Printers>
{
public:
    DECLARE_REGISTRY_RESOURCEID(IDR_PRINTERS)
    class _CreatorClass
    {
    public:
        static HRESULT WINAPI CreateInstance(void *pv, REFIID riid, LPVOID *ppv)
        {
            return CPrinterFolder_CreateInstance(
                static_cast<IUnknown *>(pv), riid, ppv);
        }
    };
};

extern "C" HRESULT WINAPI COpenWithMenu_CreateInstance(
    IUnknown *pUnkOuter, REFIID riid, LPVOID *ppv);

class COpenWithMenuRustEntry : public COpenWithMenu
{
public:
    class _CreatorClass
    {
    public:
        static HRESULT WINAPI CreateInstance(void *pv, REFIID riid, LPVOID *ppv)
        {
            return COpenWithMenu_CreateInstance(
                static_cast<IUnknown *>(pv), riid, ppv);
        }
    };
};

extern "C" HRESULT WINAPI CNetFolder_CreateInstance(
    IUnknown *pUnkOuter, REFIID riid, LPVOID *ppv);

class CNetFolderRustEntry : public CNetFolder
{
public:
    class _CreatorClass
    {
    public:
        static HRESULT WINAPI CreateInstance(void *pv, REFIID riid, LPVOID *ppv)
        {
            return CNetFolder_CreateInstance(
                static_cast<IUnknown *>(pv), riid, ppv);
        }
    };
};

extern "C" HRESULT WINAPI CFolderOptions_CreateInstance(
    IUnknown *pUnkOuter, REFIID riid, LPVOID *ppv);

class CFolderOptionsRustEntry : public CFolderOptions
{
public:
    class _CreatorClass
    {
    public:
        static HRESULT WINAPI CreateInstance(void *pv, REFIID riid, LPVOID *ppv)
        {
            return CFolderOptions_CreateInstance(
                static_cast<IUnknown *>(pv), riid, ppv);
        }
    };
};

extern "C" const CLSID CLSID_RecycleBinCleaner;
extern "C" HRESULT WINAPI CRecycleBinCleaner_CreateInstance(
    IUnknown *pUnkOuter, REFIID riid, LPVOID *ppv);

class CRecycleBinCleanerRustEntry :
    public CComObjectRootEx<CComMultiThreadModelNoCS>,
    public CComCoClass<CRecycleBinCleanerRustEntry, &CLSID_RecycleBinCleaner>
{
public:
    DECLARE_REGISTRY_RESOURCEID(IDR_RECYCLEBINCLEANER)
    class _CreatorClass
    {
    public:
        static HRESULT WINAPI CreateInstance(void *pv, REFIID riid, LPVOID *ppv)
        {
            return CRecycleBinCleaner_CreateInstance(
                static_cast<IUnknown *>(pv), riid, ppv);
        }
    };
};

extern "C" HRESULT WINAPI CRecycleBin_CreateInstance(
    IUnknown *pUnkOuter, REFIID riid, LPVOID *ppv);

class CRecycleBinRustEntry : public CRecycleBin
{
public:
    class _CreatorClass
    {
    public:
        static HRESULT WINAPI CreateInstance(void *pv, REFIID riid, LPVOID *ppv)
        {
            return CRecycleBin_CreateInstance(
                static_cast<IUnknown *>(pv), riid, ppv);
        }
    };
};

extern "C" HRESULT WINAPI CControlPanelFolder_CreateInstance(
    IUnknown *pUnkOuter, REFIID riid, LPVOID *ppv);

class CControlPanelFolderRustEntry : public CControlPanelFolder
{
public:
    class _CreatorClass
    {
    public:
        static HRESULT WINAPI CreateInstance(void *pv, REFIID riid, LPVOID *ppv)
        {
            return CControlPanelFolder_CreateInstance(
                static_cast<IUnknown *>(pv), riid, ppv);
        }
    };
};

extern "C" HRESULT WINAPI CFSFolder_CreateInstance(
    IUnknown *pUnkOuter, REFIID riid, LPVOID *ppv);

class CFSFolderRustEntry : public CFSFolder
{
public:
    class _CreatorClass
    {
    public:
        static HRESULT WINAPI CreateInstance(void *pv, REFIID riid, LPVOID *ppv)
        {
            return CFSFolder_CreateInstance(
                static_cast<IUnknown *>(pv), riid, ppv);
        }
    };
};

extern "C" HRESULT WINAPI CDesktopFolder_CreateInstance(
    IUnknown *pUnkOuter, REFIID riid, LPVOID *ppv);

class CDesktopFolderRustEntry : public CDesktopFolder
{
public:
    class _CreatorClass
    {
    public:
        static HRESULT WINAPI CreateInstance(void *pv, REFIID riid, LPVOID *ppv)
        {
            return CDesktopFolder_CreateInstance(
                static_cast<IUnknown *>(pv), riid, ppv);
        }
    };
};

extern "C" HRESULT WINAPI CDrivesFolder_CreateInstance(
    IUnknown *pUnkOuter, REFIID riid, LPVOID *ppv);

class CDrivesFolderRustEntry : public CDrivesFolder
{
public:
    class _CreatorClass
    {
    public:
        static HRESULT WINAPI CreateInstance(void *pv, REFIID riid, LPVOID *ppv)
        {
            return CDrivesFolder_CreateInstance(
                static_cast<IUnknown *>(pv), riid, ppv);
        }
    };
};

extern "C" HRESULT WINAPI CQueryAssociations_CreateInstance(
    IUnknown *pUnkOuter, REFIID riid, LPVOID *ppv);

class CQueryAssociationsRustEntry : public CQueryAssociations
{
public:
    class _CreatorClass
    {
    public:
        static HRESULT WINAPI CreateInstance(void *pv, REFIID riid, LPVOID *ppv)
        {
            return CQueryAssociations_CreateInstance(
                static_cast<IUnknown *>(pv), riid, ppv);
        }
    };
};

extern "C" HRESULT WINAPI CUserNotification_CreateInstance(
    IUnknown *pUnkOuter, REFIID riid, LPVOID *ppv);

class CUserNotificationRustEntry : public CUserNotification
{
public:
    class _CreatorClass
    {
    public:
        static HRESULT WINAPI CreateInstance(void *pv, REFIID riid, LPVOID *ppv)
        {
            return CUserNotification_CreateInstance(
                static_cast<IUnknown *>(pv), riid, ppv);
        }
    };
};

/* CShellLibrary.cpp and CUserEventTimer.cpp arrived with the post-fork upstream sync and are
 * now shell32-rust/src/shell32/{CShellLibrary,CUserEventTimer}.rs. Same shape as every entry
 * above: the class is inherited ONLY for the registry/metadata statics its header declares
 * (DECLARE_REGISTRY_RESOURCEID, DECLARE_NOT_AGGREGATABLE) and is never constructed, because
 * _CreatorClass is replaced by the Rust factory. */
extern "C" HRESULT WINAPI CShellLibrary_CreateInstance(
    IUnknown *pUnkOuter, REFIID riid, LPVOID *ppv);

class CShellLibraryRustEntry : public CShellLibrary
{
public:
    class _CreatorClass
    {
    public:
        static HRESULT WINAPI CreateInstance(void *pv, REFIID riid, LPVOID *ppv)
        {
            return CShellLibrary_CreateInstance(
                static_cast<IUnknown *>(pv), riid, ppv);
        }
    };
};

extern "C" HRESULT WINAPI CUserEventTimer_CreateInstance(
    IUnknown *pUnkOuter, REFIID riid, LPVOID *ppv);

class CUserEventTimerRustEntry : public CUserEventTimer
{
public:
    class _CreatorClass
    {
    public:
        static HRESULT WINAPI CreateInstance(void *pv, REFIID riid, LPVOID *ppv)
        {
            return CUserEventTimer_CreateInstance(
                static_cast<IUnknown *>(pv), riid, ppv);
        }
    };
};

extern "C" HRESULT WINAPI CShellLink_CreateInstance(
    IUnknown *pUnkOuter, REFIID riid, LPVOID *ppv);

class CShellLinkRustEntry : public CShellLink
{
public:
    class _CreatorClass
    {
    public:
        static HRESULT WINAPI CreateInstance(void *pv, REFIID riid, LPVOID *ppv)
        {
            return CShellLink_CreateInstance(
                static_cast<IUnknown *>(pv), riid, ppv);
        }
    };
};

/* shellmenu/shellmenu.h (via precomp.h) macro-renames the 2-arg internal creator
 * CMenuBand_CreateInstance(riid, ppv) to RSHELL_CMenuBand_CreateInstance. The Rust factory
 * export (menuband-rust) owns the plain 3-arg name, so drop the macro in this TU before
 * declaring it. (menuband-rust also exports RSHELL_CMenuBand_CreateInstance for the in-tree
 * shellmenu callers.) */
#ifdef CMenuBand_CreateInstance
#undef CMenuBand_CreateInstance
#endif
extern "C" HRESULT WINAPI CMenuBand_CreateInstance(
    IUnknown *pUnkOuter, REFIID riid, LPVOID *ppv);

class CMenuBandRustEntry : public CMenuBand
{
public:
    class _CreatorClass
    {
    public:
        static HRESULT WINAPI CreateInstance(void *pv, REFIID riid, LPVOID *ppv)
        {
            return CMenuBand_CreateInstance(
                static_cast<IUnknown *>(pv), riid, ppv);
        }
    };
};

/* Same macro-rename dance as CMenuBand above: shellmenu.h owns the 2-arg spelling. The
 * 3-arg name is the Rust factory export (menuband-rust merged_folder.rs). */
#ifdef CMergedFolder_CreateInstance
#undef CMergedFolder_CreateInstance
#endif
extern "C" HRESULT WINAPI CMergedFolder_CreateInstance(
    IUnknown *pUnkOuter, REFIID riid, void **ppv);

class CMergedFolderRustEntry : public CMergedFolder
{
public:
    class _CreatorClass
    {
    public:
        static HRESULT WINAPI CreateInstance(void *pv, REFIID riid, LPVOID *ppv)
        {
            return CMergedFolder_CreateInstance(
                static_cast<IUnknown *>(pv), riid, (void**)ppv);
        }
    };
};

extern "C" HRESULT WINAPI CMoveToMenu_CreateInstance(
    IUnknown *pUnkOuter, REFIID riid, LPVOID *ppv);

class CMoveToMenuRustEntry :
    public CComObjectRootEx<CComMultiThreadModelNoCS>,
    public CComCoClass<CMoveToMenuRustEntry, &CLSID_MoveToMenu>
{
public:
    DECLARE_REGISTRY_RESOURCEID(IDR_MOVETOMENU)
    class _CreatorClass
    {
    public:
        static HRESULT WINAPI CreateInstance(void *pv, REFIID riid, LPVOID *ppv)
        {
            return CMoveToMenu_CreateInstance(
                static_cast<IUnknown *>(pv), riid, ppv);
        }
    };
};

extern "C" HRESULT WINAPI CCopyToMenu_CreateInstance(
    IUnknown *pUnkOuter, REFIID riid, LPVOID *ppv);

class CCopyToMenuRustEntry :
    public CComObjectRootEx<CComMultiThreadModelNoCS>,
    public CComCoClass<CCopyToMenuRustEntry, &CLSID_CopyToMenu>
{
public:
    DECLARE_REGISTRY_RESOURCEID(IDR_COPYTOMENU)
    class _CreatorClass
    {
    public:
        static HRESULT WINAPI CreateInstance(void *pv, REFIID riid, LPVOID *ppv)
        {
            return CCopyToMenu_CreateInstance(
                static_cast<IUnknown *>(pv), riid, ppv);
        }
    };
};

extern "C" HRESULT WINAPI CDrvDefExt_CreateInstance(
    IUnknown *pUnkOuter, REFIID riid, LPVOID *ppv);

class CDrvDefExtRustEntry : public CDrvDefExt
{
public:
    class _CreatorClass
    {
    public:
        static HRESULT WINAPI CreateInstance(void *pv, REFIID riid, LPVOID *ppv)
        {
            return CDrvDefExt_CreateInstance(
                static_cast<IUnknown *>(pv), riid, ppv);
        }
    };
};

extern "C" HRESULT WINAPI CStartMenuDummy_CreateInstance(
    void *pUnkOuter, REFIID riid, LPVOID *ppv);

class CStartMenuDummyRustEntry : public CStartMenuDummy
{
public:
    class _CreatorClass
    {
    public:
        static HRESULT WINAPI CreateInstance(void *pv, REFIID riid, LPVOID *ppv)
        {
            return CStartMenuDummy_CreateInstance(pv, riid, ppv);
        }
    };
};

extern "C" HRESULT WINAPI CShellDispatch_CreateInstance(
    IUnknown *pUnkOuter, REFIID riid, LPVOID *ppv);

class CShellDispatchRustEntry : public CShellDispatch
{
public:
    class _CreatorClass
    {
    public:
        static HRESULT WINAPI CreateInstance(void *pv, REFIID riid, LPVOID *ppv)
        {
            return CShellDispatch_CreateInstance(
                static_cast<IUnknown *>(pv), riid, ppv);
        }
    };
};

extern "C" HRESULT WINAPI CFileDefExt_CreateInstance(
    IUnknown *pUnkOuter, REFIID riid, LPVOID *ppv);

class CFileDefExtRustEntry : public CFileDefExt
{
public:
    class _CreatorClass
    {
    public:
        static HRESULT WINAPI CreateInstance(void *pv, REFIID riid, LPVOID *ppv)
        {
            return CFileDefExt_CreateInstance(
                static_cast<IUnknown *>(pv), riid, ppv);
        }
    };
};

#undef CMenuSite_CreateInstance
extern "C" HRESULT WINAPI CMenuSite_CreateInstance(
    IUnknown *pUnkOuter, REFIID riid, LPVOID *ppv);

class CMenuSiteRustEntry : public CMenuSite
{
public:
    class _CreatorClass
    {
    public:
        static HRESULT WINAPI CreateInstance(void *pv, REFIID riid, LPVOID *ppv)
        {
            return CMenuSite_CreateInstance(
                static_cast<IUnknown *>(pv), riid, ppv);
        }
    };
};

#undef CMenuDeskBar_CreateInstance
extern "C" HRESULT WINAPI CMenuDeskBar_CreateInstance(
    IUnknown *pUnkOuter, REFIID riid, LPVOID *ppv);

class CMenuDeskBarRustEntry : public CMenuDeskBar
{
public:
    class _CreatorClass
    {
    public:
        static HRESULT WINAPI CreateInstance(void *pv, REFIID riid, LPVOID *ppv)
        {
            return CMenuDeskBar_CreateInstance(
                static_cast<IUnknown *>(pv), riid, ppv);
        }
    };
};


BEGIN_OBJECT_MAP(ObjectMap)
    OBJECT_ENTRY(CLSID_ActiveDesktop, CActiveDesktopRustEntry)
    OBJECT_ENTRY(CLSID_ShellFSFolder, CFSFolderRustEntry)
    OBJECT_ENTRY(CLSID_MyComputer, CDrivesFolderRustEntry)
    OBJECT_ENTRY(CLSID_ShellDesktop, CDesktopFolderRustEntry)
    OBJECT_ENTRY(CLSID_ShellFileDefExt, CFileDefExtRustEntry)
    OBJECT_ENTRY(CLSID_ShellDrvDefExt, CDrvDefExtRustEntry)
    OBJECT_ENTRY(CLSID_ShellItem, CShellItemRustEntry)
    OBJECT_ENTRY(CLSID_ShellLink, CShellLinkRustEntry)
    OBJECT_ENTRY(CLSID_Shell, CShellDispatchRustEntry)
    OBJECT_ENTRY(CLSID_ShellLibrary, CShellLibraryRustEntry)
    OBJECT_ENTRY(CLSID_DragDropHelper, CDropTargetHelperRustEntry)
    OBJECT_ENTRY(CLSID_ControlPanel, CControlPanelFolderRustEntry)
    OBJECT_ENTRY(CLSID_OpenControlPanel, COpenControlPanelRustEntry)
    OBJECT_ENTRY(CLSID_MyDocuments, CMyDocsFolderRustEntry)
    OBJECT_ENTRY(CLSID_NetworkPlaces, CNetFolderRustEntry)
    OBJECT_ENTRY(CLSID_FontsFolderShortcut, CFontsFolderRustEntry)
    OBJECT_ENTRY(CLSID_Printers, CPrinterFolderRustEntry)
    OBJECT_ENTRY(CLSID_AdminFolderShortcut, CAdminToolsFolderRustEntry)
    OBJECT_ENTRY(CLSID_ShellFldSetExt, CFolderOptionsRustEntry)
    OBJECT_ENTRY(CLSID_RecycleBinCleaner, CRecycleBinCleanerRustEntry)
    OBJECT_ENTRY(CLSID_RecycleBin, CRecycleBinRustEntry)
    OBJECT_ENTRY(CLSID_OpenWithMenu, COpenWithMenuRustEntry)
    OBJECT_ENTRY(CLSID_NewMenu, CNewMenuRustEntry)
    OBJECT_ENTRY(CLSID_SendToMenu, CSendToMenuRustEntry)
    OBJECT_ENTRY(CLSID_CopyAsPathMenu, CCopyAsPathMenuRustEntry)
    OBJECT_ENTRY(CLSID_CopyToMenu, CCopyToMenuRustEntry)
    OBJECT_ENTRY(CLSID_MoveToMenu, CMoveToMenuRustEntry)
    OBJECT_ENTRY(CLSID_StartMenu, CStartMenuDummyRustEntry)
    OBJECT_ENTRY(CLSID_MenuBandSite, CMenuSiteRustEntry)
    OBJECT_ENTRY(CLSID_MenuBand, CMenuBandRustEntry)
    OBJECT_ENTRY(CLSID_MenuDeskBar, CMenuDeskBarRustEntry)
    OBJECT_ENTRY(CLSID_MergedFolder, CMergedFolderRustEntry)
    OBJECT_ENTRY(CLSID_ExeDropHandler, CExeDropHandlerRustEntry)
    OBJECT_ENTRY(CLSID_QueryAssociations, CQueryAssociationsRustEntry)
    OBJECT_ENTRY(CLSID_UserNotification, CUserNotificationRustEntry)
    OBJECT_ENTRY(CLSID_UserEventTimer, CUserEventTimerRustEntry)
END_OBJECT_MAP()

CShell32Module  gModule;

extern "C" void WINAPI Shell32RustObjectLock()
{
    gModule.Lock();
}

extern "C" void WINAPI Shell32RustObjectUnlock()
{
    gModule.Unlock();
}

/* menuband-rust (libmenuband_rust.a) is linked into both shell32.dll and rshell.dll and
 * locks its host DLL's COM module around each CMenuBand instance. shell32's copy forwards
 * to gModule; rshell provides its own in base/shell/rshell/misc.cpp. */
extern "C" void WINAPI MenuBandRustModuleLock()
{
    gModule.Lock();
}

extern "C" void WINAPI MenuBandRustModuleUnlock()
{
    gModule.Unlock();
}

extern "C" HRESULT WINAPI Shell32RustCoCreateInitSF(
    LPCITEMIDLIST pidlRoot, const GUID *clsid, INT csidl,
    REFIID riid, LPVOID *ppvOut)
{
    return SHELL32_CoCreateInitSFWithCsidl(pidlRoot, clsid, csidl, riid, ppvOut);
}

extern "C" INT WINAPI Shell32RustMapContextMenuVerbToCmdId(
    LPCMINVOKECOMMANDINFO pInfo, const CMVERBMAP *pMap)
{
    return SHELL_MapContextMenuVerbToCmdId(pInfo, pMap);
}

extern "C" HRESULT WINAPI Shell32RustGetCommandStringImpl(
    SIZE_T CmdId, UINT uFlags, LPSTR Buf, UINT cchBuf, const CMVERBMAP *pMap)
{
    return SHELL_GetCommandStringImpl(CmdId, uFlags, Buf, cchBuf, pMap);
}


/***********************************************************************
 * DllGetVersion [SHELL32.@]
 *
 * Retrieves version information of the 'SHELL32.DLL'
 *
 * PARAMS
 *     pdvi [O] pointer to version information structure.
 *
 * RETURNS
 *     Success: S_OK
 *     Failure: E_INVALIDARG
 *
 * NOTES
 *     Returns version of a shell32.dll from IE4.01 SP1.
 */

STDAPI DllGetVersion(DLLVERSIONINFO *pdvi)
{
    /* FIXME: shouldn't these values come from the version resource? */
    if (pdvi->cbSize == sizeof(DLLVERSIONINFO) ||
        pdvi->cbSize == sizeof(DLLVERSIONINFO2))
    {
        pdvi->dwMajorVersion = WINE_FILEVERSION_MAJOR;
        pdvi->dwMinorVersion = WINE_FILEVERSION_MINOR;
        pdvi->dwBuildNumber = WINE_FILEVERSION_BUILD;
        pdvi->dwPlatformID = WINE_FILEVERSION_PLATFORMID;
        if (pdvi->cbSize == sizeof(DLLVERSIONINFO2))
        {
            DLLVERSIONINFO2 *pdvi2 = (DLLVERSIONINFO2 *)pdvi;

            pdvi2->dwFlags = 0;
            pdvi2->ullVersion = MAKEDLLVERULL(WINE_FILEVERSION_MAJOR,
                                              WINE_FILEVERSION_MINOR,
                                              WINE_FILEVERSION_BUILD,
                                              WINE_FILEVERSION_PLATFORMID);
        }
        TRACE("%u.%u.%u.%u\n",
              pdvi->dwMajorVersion, pdvi->dwMinorVersion,
              pdvi->dwBuildNumber, pdvi->dwPlatformID);
        return S_OK;
    }
    else
    {
        WARN("wrong DLLVERSIONINFO size from app\n");
        return E_INVALIDARG;
    }
}

/*************************************************************************
 * global variables of the shell32.dll
 * all are once per process
 *
 */
HINSTANCE   shell32_hInstance;

/*************************************************************************
 * SHELL32 DllMain
 *
 * NOTES
 *  calling oleinitialize here breaks sone apps.
 */
STDAPI_(BOOL) DllMain(HINSTANCE hInstance, DWORD dwReason, LPVOID fImpLoad)
{
    TRACE("%p 0x%x %p\n", hInstance, dwReason, fImpLoad);
    if (dwReason == DLL_PROCESS_ATTACH)
    {
        shell32_hInstance = hInstance;
        gModule.Init(ObjectMap, hInstance, &LIBID_Shell32);

        DisableThreadLibraryCalls (hInstance);

        /* get full path to this DLL for IExtractIconW_fnGetIconLocation() */
        GetModuleFileNameW(hInstance, swShell32Name, MAX_PATH);
        swShell32Name[MAX_PATH - 1] = '\0';

        /* Initialize comctl32 */
        INITCOMMONCONTROLSEX InitCtrls;
        InitCtrls.dwSize = sizeof(INITCOMMONCONTROLSEX);
        InitCtrls.dwICC = ICC_WIN95_CLASSES | ICC_DATE_CLASSES | ICC_USEREX_CLASSES;
        InitCommonControlsEx(&InitCtrls);

        /* Bad idea, initialization in DllMain! */
        InitChangeNotifications();
    }
    else if (dwReason == DLL_PROCESS_DETACH)
    {
        shell32_hInstance = NULL;
        SIC_Destroy();
        FreeChangeNotifications();
        gModule.Term();
    }
    return TRUE;
}

/***********************************************************************
 *              DllCanUnloadNow (SHELL32.@)
 */
STDAPI DllCanUnloadNow()
{
    return gModule.DllCanUnloadNow();
}

/*************************************************************************
 *              DllGetClassObject     [SHELL32.@]
 *              SHDllGetClassObject   [SHELL32.128]
 */
STDAPI DllGetClassObject(REFCLSID rclsid, REFIID riid, LPVOID *ppv)
{
    HRESULT                                hResult;

    TRACE("CLSID:%s,IID:%s\n", shdebugstr_guid(&rclsid), shdebugstr_guid(&riid));

    hResult = gModule.DllGetClassObject(rclsid, riid, ppv);
    TRACE("-- pointer to class factory: %p\n", *ppv);
    return hResult;
}

static HRESULT UpdateRegistryFromResource(BOOL Register)
{
    static const BYTE resid[] =
    {
        IDR_FOLDEROPTIONS,
        IDR_EXPLORER,
        IDR_SYSTEMFILEASSOC,
    };
    HRESULT hr = S_OK;
    for (SIZE_T i = 0; i < _countof(resid) && SUCCEEDED(hr); ++i)
        hr = gModule.UpdateRegistryFromResource(resid[i], Register, NULL);
    return hr;
}

/***********************************************************************
 *              DllRegisterServer (SHELL32.@)
 */
STDAPI DllRegisterServer()
{
    HRESULT hr;

    hr = gModule.DllRegisterServer(TRUE);
    if (FAILED(hr))
        return hr;

    if (FAILED(hr = UpdateRegistryFromResource(TRUE)))
        return hr;

    hr = SHELL_RegisterShellFolders();
    if (FAILED(hr))
        return hr;

    return S_OK;
}

/***********************************************************************
 *              DllUnregisterServer (SHELL32.@)
 */
STDAPI DllUnregisterServer()
{
    HRESULT hr;

    hr = gModule.DllUnregisterServer(TRUE);
    if (FAILED(hr))
        return hr;

    if (FAILED(hr = UpdateRegistryFromResource(FALSE)))
        return hr;

    return S_OK;
}

/*************************************************************************
 * DllInstall         [SHELL32.@]
 *
 * PARAMETERS
 *
 *    BOOL bInstall - TRUE for install, FALSE for uninstall
 *    LPCWSTR pszCmdLine - command line (unused by shell32?)
 */

HRESULT WINAPI DllInstall(BOOL bInstall, LPCWSTR cmdline)
{
    FIXME("%s %s: stub\n", bInstall ? "TRUE":"FALSE", debugstr_w(cmdline));
    return S_OK;        /* indicate success */
}
