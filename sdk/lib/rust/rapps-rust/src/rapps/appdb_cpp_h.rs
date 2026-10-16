use crate::prelude::*;
use crate::rapps::appinfo_h_h::IsInstalledEnum;
// Deterministic UTF-16 literals from the selected C++ helper closure.
pub static CPP_COM_WSTR_03259284055C: [WCHAR; 6] = [0x002A, 0x002E, 0x0074, 0x006D, 0x0070, 0x0000];
pub static CPP_COM_WSTR_075DF3C8587C: [WCHAR; 6] = [0x002A, 0x002E, 0x0069, 0x0063, 0x006F, 0x0000];
pub static CPP_COM_WSTR_3602DD16BFBA: [WCHAR; 16] = [0x0053, 0x0079, 0x0073, 0x0074, 0x0065, 0x006D, 0x0043, 0x006F, 0x006D, 0x0070, 0x006F, 0x006E, 0x0065, 0x006E, 0x0074, 0x0000];
pub static CPP_COM_WSTR_4ACCF377958C: [WCHAR; 12] = [0x0073, 0x0063, 0x0072, 0x0065, 0x0065, 0x006E, 0x0073, 0x0068, 0x006F, 0x0074, 0x0073, 0x0000];
pub static CPP_COM_WSTR_8FBE47979CB1: [WCHAR; 14] = [0x0050, 0x0061, 0x0072, 0x0065, 0x006E, 0x0074, 0x004B, 0x0065, 0x0079, 0x004E, 0x0061, 0x006D, 0x0065, 0x0000];
pub static CPP_COM_WSTR_BF93837DC388: [WCHAR; 6] = [0x002A, 0x002E, 0x0074, 0x0078, 0x0074, 0x0000];
pub static CPP_COM_WSTR_FAB65EAA8D3C: [WCHAR; 6] = [0x0069, 0x0063, 0x006F, 0x006E, 0x0073, 0x0000];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
pub const HKEY_CURRENT_USER: usize = 0x8000_0001;
pub const HKEY_LOCAL_MACHINE: usize = 0x8000_0002;

pub static mut g_RootKeyEnum: [HKEY; 3] = [
    HKEY_CURRENT_USER as HKEY,
    HKEY_LOCAL_MACHINE as HKEY,
    HKEY_LOCAL_MACHINE as HKEY,
];

pub static g_RegSamEnum: [REGSAM; 3] = [0, KEY_WOW64_32KEY, KEY_WOW64_64KEY];




















unsafe extern "C" fn GetRootKeyInfo(Index: u32, RegSam: *mut REGSAM) -> HKEY {
    C_ASSERT!(core::mem::size_of::<[HKEY; 3]>() / core::mem::size_of::<HKEY>() == _countof!(g_RegSamEnum) as usize);
    if (Index as usize) < _countof!(g_RootKeyEnum) as usize {
        *RegSam = g_RegSamEnum[Index as usize];
        return g_RootKeyEnum[Index as usize];
    }
    return core::ptr::null_mut();
}

unsafe extern "C" fn ClearList(list: &mut CAtlList<*mut CAppInfo>) {
    let mut InfoListPosition = list.GetHeadPosition();
    while !InfoListPosition.is_null() {
        let Info: *mut CAppInfo = *list.GetNext(&mut InfoListPosition);
        CAppInfo__delete(Info);
    }
    list.RemoveAll();
}

unsafe extern "C" fn DeleteWithWildcard(Dir: &CPathW, Filter: LPCWSTR) {
    let mut hFind: HANDLE = INVALID_HANDLE_VALUE;
    let mut FindFileData: WIN32_FIND_DATAW = core::mem::zeroed();

    let mut DirWithFilter: CPathW = Dir.clone();
    DirWithFilter += Filter;

    hFind = FindFirstFileW(DirWithFilter.GetString(), core::ptr::addr_of_mut!(FindFileData).cast());

    if hFind == INVALID_HANDLE_VALUE {
        return;
    }

    loop {
        let mut szTmp: CPathW = Dir.clone();
        szTmp += FindFileData.cFileName.as_ptr();

        if (FindFileData.dwFileAttributes & FILE_ATTRIBUTE_DIRECTORY) == 0 {
            DeleteFileW(szTmp.GetString());
        }

        if FindNextFileW(hFind, core::ptr::addr_of_mut!(FindFileData).cast()) == 0 {
            break;
        }
    }
    FindClose(hFind);
}

impl CAppDB {
    pub unsafe fn EnumerateFiles(&mut self) -> BOOL {
        let this_: *mut Self = self;
        ClearList(&mut (*this_).m_Available);

        let mut AppsPath: CPathW = (*this_).m_BasePath.clone();
        AppsPath += RAPPS_DATABASE_SUBDIR.as_ptr();
        let mut WildcardPath: CPathW = AppsPath.clone();
        WildcardPath += CPP_COM_WSTR_BF93837DC388.as_ptr();

        let mut FindFileData: WIN32_FIND_DATAW = core::mem::zeroed();
        let mut hFind: HANDLE = FindFirstFileW(WildcardPath.GetString(), core::ptr::addr_of_mut!(FindFileData).cast());
        if hFind == INVALID_HANDLE_VALUE {
            return (FALSE) as BOOL;
        }

        loop {
            let mut szPkgName: CStringW = CStringW::from_ptr(FindFileData.cFileName.as_ptr());
            PathRemoveExtensionW(szPkgName.GetBuffer((MAX_PATH as i32)));
            szPkgName.ReleaseBuffer();

            // static upcast of a possibly-NULL CAvailableApplicationInfo*: CAppInfo is its first base
            let mut Info: *mut CAppInfo = (*this_).FindAvailableByPackageName(&szPkgName) as *mut CAppInfo;
            ATLASSERT!(Info == core::ptr::null_mut());
            if Info.is_null() {
                Info = CAppDB::CreateAvailableAppInstance(&szPkgName, AppsPath.GetString()) as *mut CAppInfo;
                if !Info.is_null() {
                    (*this_).m_Available.AddTail(Info);
                }
            }

            if FindNextFileW(hFind, core::ptr::addr_of_mut!(FindFileData).cast()) == 0 {
                break;
            }
        }

        FindClose(hFind);
        return (TRUE) as BOOL;
    }
    pub unsafe fn EnumerateRegistry(List: *mut CAtlList<*mut CAppInfo>, SearchOnly: LPCWSTR) -> *mut CInstalledApplicationInfo {
        ATLASSERT!(!List.is_null() || !SearchOnly.is_null());
        let mut wowsam: REGSAM = 0;
        let mut hRootKey: HKEY = core::ptr::null_mut();
        let mut RootEnum: CEnumInstalledRootKey = CEnumInstalledRootKey__make_0();
        loop {
            hRootKey = RootEnum.Get(&mut wowsam);
            if hRootKey.is_null() {
                break;
            }
            'body: {
                let mut hKey: CRegKey = CRegKey__make_0();
                if hKey.Open(hRootKey, UNINSTALL_SUBKEY, KEY_READ | wowsam) != (ERROR_SUCCESS as i32) {
                    break 'body;
                }
                let mut Index: DWORD = 0;
                loop {
                    let mut szKeyName: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
                    let mut dwSize: DWORD = _countof!(szKeyName);
                    if hKey.EnumKey(Index, szKeyName.as_mut_ptr(), &mut dwSize, core::ptr::null_mut()) != (ERROR_SUCCESS as i32) {
                        break;
                    }
                    if !List.is_null() || StrCmpIW(SearchOnly, szKeyName.as_ptr()) == 0 {
                        let mut Info: *mut CInstalledApplicationInfo;
                        Info = CAppDB::CreateInstalledAppByRegistryKey__0(szKeyName.as_ptr(), (hKey).op_to_HKEY(), RootEnum.GetKeyIndex());
                        if !Info.is_null() {
                            if !List.is_null() {
                                (*List).AddTail(core::ptr::addr_of_mut!((*Info).__base0));
                            } else {
                                return Info;
                            }
                        }
                    }
                    Index = Index.wrapping_add(1);
                }
            }
            RootEnum.Next();
        }
        core::ptr::null_mut()
    }
    pub unsafe fn CreateInstalledAppByRegistryKey__0(KeyName: LPCWSTR, hKeyParent: HKEY, KeyIndex: UINT) -> *mut CInstalledApplicationInfo {
        let mut hSubKey: CRegKey = CRegKey__make_0();
        if hSubKey.Open(hKeyParent, KeyName, KEY_READ) != (ERROR_SUCCESS as i32) {
            return core::ptr::null_mut();
        }
        let mut value: DWORD = 0;
        let mut size: DWORD = 0;

        size = core::mem::size_of::<DWORD>() as DWORD;
        if RegQueryValueExW(hSubKey.op_to_HKEY(), CPP_COM_WSTR_3602DD16BFBA.as_ptr(), core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::addr_of_mut!(value) as LPBYTE, core::ptr::addr_of_mut!(size)) == 0 && value == 1
        {
            // Ignore system components
            return core::ptr::null_mut();
        }

        size = 0;
        let bIsUpdate: BOOL = (RegQueryValueExW(hSubKey.op_to_HKEY(), CPP_COM_WSTR_8FBE47979CB1.as_ptr(), core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::addr_of_mut!(size)) == 0) as BOOL;

        let cat: AppsCategories = if bIsUpdate != 0 { ENUM_UPDATES } else { ENUM_INSTALLED_APPLICATIONS };
        let mut pInfo: *mut CInstalledApplicationInfo;
        pInfo = CInstalledApplicationInfo__new_0(hSubKey.Detach(), &CStringW::from_ptr(KeyName), cat, KeyIndex);
        if !pInfo.is_null() && (*pInfo).Valid() != 0
        {
            return pInfo;
        }
        CInstalledApplicationInfo__delete(pInfo);
        return core::ptr::null_mut();
    }
    pub unsafe fn __ctor_0(&mut self, path: &CStringW) {
        let this_: *mut Self = self;
        (*this_).m_BasePath = CPathW::from_cstring(path);
        {
            (*this_).m_BasePath.Canonicalize();
        }
    }
    pub unsafe fn GetDefaultPath() -> CStringW {
        let mut path = CStringW::new();
        GetStorageDirectory(core::ptr::addr_of_mut!(path));
        return path;
    }
    pub unsafe fn GetApps(&mut self, List: &mut CAtlList<*mut CAppInfo>, Type: AppsCategories) {
        let this_: *mut Self = self;
        let UseInstalled: BOOL = IsInstalledEnum((Type) as i32);
        let list: &CAtlList<*mut CAppInfo> = if UseInstalled != 0 {
            &(*this_).m_Installed
        } else {
            &(*this_).m_Available
        };
        let IncludeAll: bool = if UseInstalled != 0 {
            Type == ENUM_ALL_INSTALLED
        } else {
            Type == ENUM_ALL_AVAILABLE
        };

        let mut CurrentListPosition: POSITION = list.GetHeadPosition();
        while !CurrentListPosition.is_null() {
            let Info: *mut CAppInfo = *list.GetNext(&mut CurrentListPosition);

            if IncludeAll || Type == (*Info).iCategory {
                List.AddTail(Info);
            }
        }
    }
    pub unsafe fn FindAvailableByPackageName(&mut self, name: &CStringW) -> *mut CAvailableApplicationInfo {
        let this_: *mut Self = self;
        let mut CurrentListPosition = (*this_).m_Available.GetHeadPosition();
        while !CurrentListPosition.is_null() {
            let Info: *mut CAppInfo = *(*this_).m_Available.GetNext(&mut CurrentListPosition);
            if (*Info).szIdentifier.Compare(name.GetString()) == 0 {
                return Info as *mut CAvailableApplicationInfo;
            }
        }
        return core::ptr::null_mut();
    }
    pub unsafe fn UpdateAvailable(&mut self) {
        let this_: *mut Self = self;
        if CreateDirectoryW((*this_).m_BasePath.GetString(), core::ptr::null_mut()) == 0
            && GetLastError() != ERROR_ALREADY_EXISTS
        {
            return;
        }

        if (*this_).EnumerateFiles() != 0 {
            return;
        }

        DownloadApplicationsDB(
            if SettingsInfo.bUseSource != 0 { (SettingsInfo.szSourceURL).as_ptr() } else { APPLICATION_DATABASE_URL.as_ptr() },
            (SettingsInfo.bUseSource == 0) as i32,
        );

        let mut AppsPath: CPathW = (*this_).m_BasePath.clone();
        AppsPath += RAPPS_DATABASE_SUBDIR.as_ptr();
        if ExtractFilesFromCab__0(
            &CStringW::from_ptr(APPLICATION_DATABASE_NAME.as_ptr()),
            &(*this_).m_BasePath.m_strPath,
            &AppsPath.m_strPath,
            None,
            core::ptr::null_mut(),
        ) == 0
        {
            return;
        }

        let mut CabFile: CPathW = (*this_).m_BasePath.clone();
        CabFile += APPLICATION_DATABASE_NAME.as_ptr();
        DeleteFileW(CabFile.GetString());

        (*this_).EnumerateFiles();
    }
    pub unsafe fn UpdateInstalled(&mut self) {
        let this_: *mut Self = self;
        // Remove all old entries
        ClearList(&mut (*this_).m_Installed);

        CAppDB::EnumerateRegistry(&mut (*this_).m_Installed, core::ptr::null_mut());
    }
    pub unsafe fn RemoveCached(&mut self) {
        let this_: *mut Self = self;
        // Delete icons
        let mut AppsPath: CPathW = (*this_).m_BasePath.clone();
        AppsPath += RAPPS_DATABASE_SUBDIR.as_ptr();
        let mut IconPath: CPathW = AppsPath.clone();
        IconPath += CPP_COM_WSTR_FAB65EAA8D3C.as_ptr();
        DeleteWithWildcard(&IconPath, CPP_COM_WSTR_075DF3C8587C.as_ptr());

        // Delete leftover screenshots
        let mut ScrnshotFolder: CPathW = AppsPath.clone();
        ScrnshotFolder += CPP_COM_WSTR_4ACCF377958C.as_ptr();
        DeleteWithWildcard(&ScrnshotFolder, CPP_COM_WSTR_03259284055C.as_ptr());

        // Delete data base files (*.txt)
        DeleteWithWildcard(&AppsPath, CPP_COM_WSTR_BF93837DC388.as_ptr());

        RemoveDirectoryW(IconPath.GetString());
        RemoveDirectoryW(ScrnshotFolder.GetString());
        RemoveDirectoryW(AppsPath.GetString());
        RemoveDirectoryW((*this_).m_BasePath.GetString());
    }
    pub unsafe fn RemoveInstalledAppFromRegistry(Info: *const CAppInfo) -> DWORD {
        // Validate that this is actually an installed app / update
        ATLASSERT!((*Info).iCategory == ENUM_INSTALLED_APPLICATIONS || (*Info).iCategory == ENUM_UPDATES);
        if (*Info).iCategory != ENUM_INSTALLED_APPLICATIONS && (*Info).iCategory != ENUM_UPDATES {
            return ERROR_INVALID_PARAMETER;
        }

        let InstalledInfo: *const CInstalledApplicationInfo = Info as *const CInstalledApplicationInfo;

        let mut Name: CStringW = CStringW::from_ptr((*InstalledInfo).__base0.szIdentifier.GetString());
        let mut wowsam: REGSAM = 0;
        let hRoot: HKEY = GetRootKeyInfo((*InstalledInfo).m_KeyInfo, &mut wowsam);
        ATLASSERT!(hRoot);
        if hRoot.is_null() {
            return ERROR_OPEN_FAILED;
        }

        let mut Uninstall: CRegKey = CRegKey__make_0();
        let mut err: LSTATUS = Uninstall.Open(hRoot, UNINSTALL_SUBKEY, KEY_READ | KEY_WRITE | wowsam);
        if err == (ERROR_SUCCESS as i32) {
            err = Uninstall.RecurseDeleteKey(Name.GetString());
        }
        return err as DWORD;
    }
    pub unsafe fn CreateInstalledAppByRegistryKey__1(Name: LPCWSTR) -> *mut CInstalledApplicationInfo {
        return CAppDB::EnumerateRegistry(core::ptr::null_mut(), Name);
    }
    pub unsafe fn CreateInstalledAppInstance(KeyName: LPCWSTR, User: BOOL, WowSam: REGSAM) -> *mut CInstalledApplicationInfo {
        let hRootKey: HKEY = if User != 0 { HKEY_CURRENT_USER as HKEY } else { HKEY_LOCAL_MACHINE as HKEY };
        let KeyIndex: UINT = if User != 0 { (0) } else { if (WowSam & KEY_WOW64_64KEY) != 0 { 2 } else { 1 } };
        let mut hKey: CRegKey = CRegKey__make_0();
        if hKey.Open(hRootKey, UNINSTALL_SUBKEY, KEY_READ | WowSam) == (ERROR_SUCCESS as i32) {
            return CAppDB::CreateInstalledAppByRegistryKey__0(KeyName, hKey.op_to_HKEY(), KeyIndex);
        }
        return core::ptr::null_mut();
    }
    pub unsafe fn CreateAvailableAppInstance(PkgName: &CStringW, DBPath: PCWSTR) -> *mut CAvailableApplicationInfo {
        let mut pAppInfo: *mut CAvailableApplicationInfo;
        let mut AppsPath: CPathW = if !DBPath.is_null() {
            CPathW::from_ptr(DBPath)
        } else {
            let mut tmp = CPathW::from_cstring(&CAppDB::GetDefaultPath());
            tmp += RAPPS_DATABASE_SUBDIR.as_ptr();
            tmp
        };
        let mut ManifestPath: CPathW = AppsPath.clone();
        let mut ManifestName: CStringW = PkgName.clone();
        ManifestName += MANIFEST_DOTEXT;
        ManifestPath += &ManifestName;
        let Parser: *mut CConfigParser = CConfigParser__new_0(&ManifestPath.m_strPath);
        let mut Cat: i32 = 0;
        if (((*Parser).GetInt(&CStringW::from_ptr(DB_CATEGORY.as_ptr()), &mut Cat)) == 0) {
            Cat = (ENUM_INVALID) as i32;
        }

        pAppInfo = CAvailableApplicationInfo__new_0(Parser, PkgName, Cat as AppsCategories, &AppsPath);
        if (*pAppInfo).Valid() != 0 {
            return pAppInfo;
        }
        CAvailableApplicationInfo__delete(pAppInfo);
        return core::ptr::null_mut();
    }
}

impl CEnumInstalledRootKey {
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;
        (*this_).m_Index = 0;
        {
            // We don't want to display duplicate entries on systems without WoW64 keys support.
            // When ROS starts supporting the WOW REGSAM flags,
            // this code can be changed back to IsSystem64Bit() ? 3 : 2.
            (*this_).m_Count = 2 + ((IsSameRegKey__0(HKEY_LOCAL_MACHINE as HKEY, UNINSTALL_SUBKEY, KEY_WOW64_64KEY,
                                                            UNINSTALL_SUBKEY, KEY_WOW64_32KEY) == 0) as UINT);
        }
    }
    pub unsafe fn Get(&mut self, RegSam: &mut REGSAM) -> HKEY {
        let this_: *mut Self = self;
        return if (*this_).m_Index < (*this_).m_Count { GetRootKeyInfo((*this_).m_Index, RegSam) } else { core::ptr::null_mut() };
    }
    pub unsafe fn Next(&mut self) {
        let this_: *mut Self = self;
        (*this_).m_Index += 1;
    }
    pub unsafe fn GetKeyIndex(&mut self) -> UINT {
        let this_: *mut Self = self;
        (*this_).m_Index
    }
}
