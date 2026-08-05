use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
pub static mut hDllInstance: HINSTANCE = core::ptr::null_mut();
pub const SIDN_LOOKUPSUCCEEDED: u32 = 0x101;

#[repr(C)]
pub struct _SIDLOOKUPNOTIFYINFO {
    pub nmh: NMHDR,
    pub Sid: PSID,
    pub SidRequestResult: PSIDREQRESULT,
}
pub type SIDLOOKUPNOTIFYINFO = _SIDLOOKUPNOTIFYINFO;
pub type PSIDLOOKUPNOTIFYINFO = *mut _SIDLOOKUPNOTIFYINFO;
unsafe extern "C" fn AceHeaderToSID(AceHeader: PACE_HEADER) -> PSID {
    let mut Sid: PSID = core::ptr::null_mut();
    match (*AceHeader).AceType as UINT {
        ACCESS_ALLOWED_ACE_TYPE => {
            Sid = core::ptr::addr_of_mut!((*(AceHeader as PACCESS_ALLOWED_ACE)).SidStart) as PSID;
        }
        ACCESS_ALLOWED_OBJECT_ACE_TYPE => {
            Sid = core::ptr::addr_of_mut!((*(AceHeader as PACCESS_ALLOWED_OBJECT_ACE)).SidStart) as PSID;
        }
        ACCESS_DENIED_ACE_TYPE => {
            Sid = core::ptr::addr_of_mut!((*(AceHeader as PACCESS_DENIED_ACE)).SidStart) as PSID;
        }
        SYSTEM_AUDIT_OBJECT_ACE_TYPE => {
            Sid = core::ptr::addr_of_mut!((*(AceHeader as PACCESS_DENIED_OBJECT_ACE)).SidStart) as PSID;
        }
        _ => {}
    }

    Sid
}
unsafe extern "C" fn DestroySecurityPage(sp: PSECURITY_PAGE) {
    DestroySidCacheMgr((*sp).SidCacheMgr);

    if (*sp).OwnerSid != core::ptr::null_mut() {
        LocalFree((*sp).OwnerSid as HLOCAL);
    }

    HeapFree(
        GetProcessHeap(),
        0,
        sp as LPVOID,
    );

    CoUninitialize();
}
unsafe extern "C" fn FreePrincipalsList(
    sp: PSECURITY_PAGE,
    PrincipalsListHead: *mut PPRINCIPAL_LISTITEM,
) {
    let mut CurItem: PPRINCIPAL_LISTITEM;
    let mut NextItem: PPRINCIPAL_LISTITEM;
    let mut AceEntry: PACE_ENTRY;
    let mut NextAceEntry: PACE_ENTRY;

    CurItem = *PrincipalsListHead;
    while !CurItem.is_null() {
        AceEntry = (*CurItem).ACEs;
        while !AceEntry.is_null() {
            NextAceEntry = (*AceEntry).Next;
            HeapFree(
                GetProcessHeap(),
                0,
                AceEntry as LPVOID,
            );
            AceEntry = NextAceEntry;
        }

        if !(*CurItem).SidReqResult.is_null() {
            DereferenceSidReqResult(
                (*sp).SidCacheMgr,
                (*CurItem).SidReqResult,
            );
        }

        if !(*CurItem).DisplayString.is_null() {
            LocalFree((*CurItem).DisplayString as HLOCAL);
        }

        NextItem = (*CurItem).Next;
        HeapFree(
            GetProcessHeap(),
            0,
            CurItem as LPVOID,
        );
        CurItem = NextItem;
    }

    *PrincipalsListHead = core::ptr::null_mut();
}
unsafe extern "C" fn AddAceToPrincipal(
    Principal: PPRINCIPAL_LISTITEM,
    AceHeader: PACE_HEADER,
) -> PACE_ENTRY {
    let mut AceEntry: PACE_ENTRY;
    let mut AceLink: *mut PACE_ENTRY;

    AceEntry = HeapAlloc(
        GetProcessHeap(),
        0,
        (core::mem::size_of::<ACE_ENTRY>() as usize).wrapping_add((*AceHeader).AceSize as usize) as SIZE_T,
    ) as PACE_ENTRY;

    if !AceEntry.is_null() {
        (*AceEntry).Next = core::ptr::null_mut();

        CopyMemory!(
            AceEntry.offset(1) as *mut core::ffi::c_void,
            AceHeader as *const core::ffi::c_void,
            (*AceHeader).AceSize as usize,
        );

        AceLink = core::ptr::addr_of_mut!((*Principal).ACEs);
        while !(*AceLink).is_null() {
            AceLink = core::ptr::addr_of_mut!((*(*AceLink)).Next);
        }
        *AceLink = AceEntry;
    }

    AceEntry
}
unsafe extern "C" fn FindSidInPrincipalsListAddAce(
    PrincipalsListHead: PPRINCIPAL_LISTITEM,
    Sid: PSID,
    AceHeader: PACE_HEADER,
) -> PPRINCIPAL_LISTITEM {
    let mut CurItem = PrincipalsListHead;

    while !CurItem.is_null() {
        if EqualSid(
            CurItem.offset(1) as PSID,
            Sid,
        ) != 0
        {
            if !AddAceToPrincipal(CurItem, AceHeader).is_null() {
                return CurItem;
            }

            break;
        }
        CurItem = (*CurItem).Next;
    }

    core::ptr::null_mut()
}
unsafe extern "C" fn SidLookupCompletion(
    SidCacheMgr: HANDLE,
    Sid: PSID,
    SidRequestResult: PSIDREQRESULT,
    Context: PVOID,
) {
    let sp = Context as PSECURITY_PAGE;

    if !SidRequestResult.is_null() {
        let mut LookupInfo: SIDLOOKUPNOTIFYINFO = core::mem::zeroed();

        LookupInfo.nmh.hwndFrom = (*sp).hWnd;
        LookupInfo.nmh.idFrom = 0;
        LookupInfo.nmh.code = SIDN_LOOKUPSUCCEEDED;
        LookupInfo.Sid = Sid;
        LookupInfo.SidRequestResult = SidRequestResult;

        SendMessage(
            (*sp).hWnd,
            (WM_NOTIFY) as u32,
            LookupInfo.nmh.idFrom as WPARAM,
            core::ptr::addr_of_mut!(LookupInfo.nmh) as LPARAM,
        );
    }
}
unsafe extern "C" fn AddPrincipalToList(
    sp: PSECURITY_PAGE,
    Sid: PSID,
    AceHeader: PACE_HEADER,
    LookupDeferred: *mut BOOL,
) -> PPRINCIPAL_LISTITEM {
    let mut PrincipalListItem: PPRINCIPAL_LISTITEM = core::ptr::null_mut();
    let mut PrincipalLink: *mut PPRINCIPAL_LISTITEM;
    let mut AceEntry: PACE_ENTRY;
    let mut Deferred: BOOL = (FALSE) as i32;

    if FindSidInPrincipalsListAddAce((*sp).PrincipalsListHead, Sid, AceHeader).is_null() {
        let mut SidLength: DWORD;

        PrincipalLink = core::ptr::addr_of_mut!((*sp).PrincipalsListHead);
        while !(*PrincipalLink).is_null() {
            PrincipalLink = core::ptr::addr_of_mut!((**PrincipalLink).Next);
        }

        SidLength = GetLengthSid(Sid);

        PrincipalListItem = HeapAlloc(
            GetProcessHeap(),
            0,
            ((core::mem::size_of::<PRINCIPAL_LISTITEM>() as DWORD)
                .wrapping_add(SidLength)) as u64,
        ) as PPRINCIPAL_LISTITEM;
        if !PrincipalListItem.is_null() {
            (*PrincipalListItem).DisplayString = core::ptr::null_mut();
            (*PrincipalListItem).SidReqResult = core::ptr::null_mut();

            CopySid(
                SidLength,
                PrincipalListItem.offset(1) as PSID,
                Sid,
            );

            AceEntry = HeapAlloc(
                GetProcessHeap(),
                0,
                ((core::mem::size_of::<ACE_ENTRY>() as DWORD)
                    .wrapping_add((*AceHeader).AceSize as DWORD)) as u64,
            ) as PACE_ENTRY;
            if !AceEntry.is_null() {
                (*AceEntry).Next = core::ptr::null_mut();
                CopyMemory!(
                    AceEntry.offset(1) as *mut core::ffi::c_void,
                    AceHeader as *const core::ffi::c_void,
                    (*AceHeader).AceSize as usize,
                );

                (*PrincipalListItem).ACEs = AceEntry;

                (*PrincipalListItem).Next = core::ptr::null_mut();

                *PrincipalLink = PrincipalListItem;

                Deferred = if LookupSidCache(
                    (*sp).SidCacheMgr,
                    Sid,
                    SidLookupCompletion,
                    sp as PVOID,
                ) != 0
                {
                    0
                } else {
                    1
                };
            } else {
                HeapFree(
                    GetProcessHeap(),
                    0,
                    PrincipalListItem as LPVOID,
                );
                PrincipalListItem = core::ptr::null_mut();
            }
        }
    }

    if !PrincipalListItem.is_null() && !LookupDeferred.is_null() {
        *LookupDeferred = Deferred;
    }

    PrincipalListItem
}
unsafe extern "C" fn GetDisplayStringFromSidRequestResult(SidReqResult: PSIDREQRESULT) -> LPWSTR {
    let mut lpDisplayString: LPWSTR = core::ptr::null_mut();

    if (*SidReqResult).SidNameUse == SidTypeUser ||
       (*SidReqResult).SidNameUse == SidTypeGroup {
        LoadAndFormatString(
            hDllInstance,
            (IDS_USERDOMAINFORMAT) as u32,
            &mut lpDisplayString,
            (*SidReqResult).AccountName,
            (*SidReqResult).DomainName,
            (*SidReqResult).AccountName,
        );
    } else {
        LoadAndFormatString(
            hDllInstance,
            (IDS_USERFORMAT) as u32,
            &mut lpDisplayString,
            (*SidReqResult).AccountName,
        );
    }

    lpDisplayString
}
unsafe extern "C" fn GetPrincipalDisplayString(PrincipalListItem: PPRINCIPAL_LISTITEM) -> LPWSTR {
    let mut lpDisplayString: LPWSTR = core::ptr::null_mut();

    if (*PrincipalListItem).SidReqResult != core::ptr::null_mut() {
        lpDisplayString = GetDisplayStringFromSidRequestResult((*PrincipalListItem).SidReqResult);
    } else {
        ConvertSidToStringSidW(
            PrincipalListItem.offset(1) as PSID,
            &mut lpDisplayString,
        );
    }

    lpDisplayString
}
unsafe extern "C" fn GetPrincipalAccountNameString(
    PrincipalListItem: PPRINCIPAL_LISTITEM,
) -> LPWSTR {
    let mut lpDisplayString: LPWSTR = core::ptr::null_mut();

    if (*PrincipalListItem).SidReqResult != core::ptr::null_mut() {
        LoadAndFormatString(
            hDllInstance,
            (IDS_USERFORMAT) as u32,
            &mut lpDisplayString,
            (*(*PrincipalListItem).SidReqResult).AccountName,
        );
    } else {
        ConvertSidToStringSid(
            PrincipalListItem.offset(1) as PSID,
            &mut lpDisplayString,
        );
    }

    lpDisplayString
}
unsafe extern "C" fn CreatePrincipalListItem(
    li: *mut LVITEM,
    sp: PSECURITY_PAGE,
    PrincipalListItem: PPRINCIPAL_LISTITEM,
    Index: INT,
    Selected: BOOL,
) {
    let mut ImageIndex = 2;

    if !(*PrincipalListItem).SidReqResult.is_null() {
        match (*(*PrincipalListItem).SidReqResult).SidNameUse {
            SidTypeUser => {
                ImageIndex = 0;
            }
            SidTypeWellKnownGroup | SidTypeGroup => {
                ImageIndex = 1;
            }
            _ => {}
        }
    }

    (*li).mask = (LVIF_IMAGE | LVIF_PARAM | LVIF_STATE | LVIF_TEXT) as u32;
    (*li).iItem = Index;
    (*li).iSubItem = 0;
    (*li).state = if Selected != 0 { (LVIS_SELECTED) as u32 } else { 0 };
    (*li).stateMask = (LVIS_SELECTED) as u32;
    (*li).pszText = (*PrincipalListItem).DisplayString;
    (*li).iImage = ImageIndex;
    (*li).lParam = PrincipalListItem as LPARAM;
}
unsafe extern "C" fn AddPrincipalListEntry(
    sp: PSECURITY_PAGE,
    PrincipalListItem: PPRINCIPAL_LISTITEM,
    Index: INT,
    Selected: BOOL,
) -> INT {
    let mut li: LVITEM = core::mem::zeroed();
    let Ret: INT;

    if (*PrincipalListItem).DisplayString != core::ptr::null_mut() {
        LocalFree((*PrincipalListItem).DisplayString as HLOCAL);
    }
    (*PrincipalListItem).DisplayString = GetPrincipalDisplayString(PrincipalListItem);

    CreatePrincipalListItem(
        &mut li,
        sp,
        PrincipalListItem,
        Index,
        Selected,
    );

    Ret = ListView_InsertItem!((*sp).hWndPrincipalsList, &mut li);

    return Ret;
}
#[cfg(target_arch = "x86")]
unsafe extern "stdcall" fn PrincipalCompare(
    lParam1: LPARAM,
    lParam2: LPARAM,
    lParamSort: LPARAM,
) -> i32 {
    let Item1 = lParam1 as PPRINCIPAL_LISTITEM;
    let Item2 = lParam2 as PPRINCIPAL_LISTITEM;

    if (*Item1).DisplayString != core::ptr::null_mut() && (*Item2).DisplayString != core::ptr::null_mut() {
        return wcscmp((*Item1).DisplayString, (*Item2).DisplayString);
    }

    return 0;
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn PrincipalCompare(
    lParam1: LPARAM,
    lParam2: LPARAM,
    lParamSort: LPARAM,
) -> i32 {
    let Item1 = lParam1 as PPRINCIPAL_LISTITEM;
    let Item2 = lParam2 as PPRINCIPAL_LISTITEM;

    if (*Item1).DisplayString != core::ptr::null_mut() && (*Item2).DisplayString != core::ptr::null_mut() {
        return wcscmp((*Item1).DisplayString, (*Item2).DisplayString);
    }

    return 0;
}
unsafe extern "C" fn UpdatePrincipalListItem(
    sp: PSECURITY_PAGE,
    PrincipalIndex: INT,
    PrincipalListItem: PPRINCIPAL_LISTITEM,
    SidReqResult: PSIDREQRESULT,
) {
    let mut li: LVITEM = core::mem::zeroed();

    if (*PrincipalListItem).SidReqResult != core::ptr::null_mut() {
        DereferenceSidReqResult(
            (*sp).SidCacheMgr,
            (*PrincipalListItem).SidReqResult,
        );
    }

    ReferenceSidReqResult((*sp).SidCacheMgr, SidReqResult);
    (*PrincipalListItem).SidReqResult = SidReqResult;

    if (*PrincipalListItem).DisplayString != core::ptr::null_mut() {
        LocalFree((*PrincipalListItem).DisplayString as HLOCAL);
    }
    (*PrincipalListItem).DisplayString = GetPrincipalDisplayString(PrincipalListItem);

    CreatePrincipalListItem(
        &mut li,
        sp,
        PrincipalListItem,
        PrincipalIndex,
        (FALSE) as i32,
    );

    li.mask &= (!(LVIF_STATE | LVIF_PARAM)) as u32;

    ListView_SetItem!((*sp).hWndPrincipalsList, &mut li);

    ListView_SortItems!(
        (*sp).hWndPrincipalsList,
        PrincipalCompare,
        sp as LPARAM,
    );
}
unsafe extern "C" fn ReloadPrincipalsList(sp: PSECURITY_PAGE) {
    let mut SecurityDescriptor: PSECURITY_DESCRIPTOR = core::ptr::null_mut();
    let mut DaclPresent: BOOL = 0;
    let mut DaclDefaulted: BOOL = 0;
    let mut OwnerDefaulted: BOOL = 0;
    let mut Dacl: PACL = core::ptr::null_mut();
    let mut OwnerSid: PSID = core::ptr::null_mut();
    let mut OwnerSidString: LPTSTR;
    let mut SidLen: DWORD;
    let mut hRet: HRESULT;

    FreePrincipalsList(sp, core::ptr::addr_of_mut!((*sp).PrincipalsListHead));

    hRet = (*((*(*sp).psi).lpVtbl)).GetSecurity.unwrap_unchecked()(
        (*sp).psi,
        DACL_SECURITY_INFORMATION | OWNER_SECURITY_INFORMATION,
        core::ptr::addr_of_mut!(SecurityDescriptor),
        (FALSE) as i32,
    );

    if ((hRet) >= 0) != 0 && SecurityDescriptor != core::ptr::null_mut() {
        if GetSecurityDescriptorOwner(
            SecurityDescriptor,
            core::ptr::addr_of_mut!(OwnerSid),
            core::ptr::addr_of_mut!(OwnerDefaulted),
        ) != 0
        {
            (*sp).OwnerDefaulted = OwnerDefaulted;
            if (*sp).OwnerSid != core::ptr::null_mut() {
                LocalFree((*sp).OwnerSid as HLOCAL);
                (*sp).OwnerSid = core::ptr::null_mut();
            }

            SidLen = GetLengthSid(OwnerSid);
            if SidLen == 0 {
                SetDlgItemText((*sp).hWnd, IDC_OWNER, core::ptr::null_mut());
            } else {
                (*sp).OwnerSid = LocalAlloc(LMEM_FIXED, (SidLen) as u64) as PSID;
                if (*sp).OwnerSid != core::ptr::null_mut() {
                    if CopySid(SidLen, (*sp).OwnerSid, OwnerSid) != 0 {
                        if LookupSidCache(
                            (*sp).SidCacheMgr,
                            (*sp).OwnerSid,
                            SidLookupCompletion,
                            sp as LPVOID,
                        ) == 0
                        {
                            if ConvertSidToStringSid(
                                (*sp).OwnerSid,
                                (core::ptr::addr_of_mut!(OwnerSidString)).cast(),
                            ) != 0
                            {
                                SetDlgItemText(
                                    (*sp).hWnd,
                                    IDC_OWNER,
                                    OwnerSidString,
                                );
                                LocalFree(OwnerSidString as HLOCAL);
                            } else {
                                SetDlgItemText((*sp).hWnd, IDC_OWNER, core::ptr::null_mut());
                            }
                        }
                    } else {
                        SetDlgItemText((*sp).hWnd, IDC_OWNER, core::ptr::null_mut());
                    }
                } else {
                    SetDlgItemText((*sp).hWnd, IDC_OWNER, core::ptr::null_mut());
                }
            }
        } else {
            SetDlgItemText((*sp).hWnd, IDC_OWNER, core::ptr::null_mut());
        }

        if GetSecurityDescriptorDacl(
            SecurityDescriptor,
            core::ptr::addr_of_mut!(DaclPresent),
            core::ptr::addr_of_mut!(Dacl),
            core::ptr::addr_of_mut!(DaclDefaulted),
        ) != 0
            && DaclPresent != 0
            && Dacl != core::ptr::null_mut()
        {
            let mut Sid: PSID;
            let mut AceHeader: PACE_HEADER;
            let mut AceIndex: ULONG;

            AceIndex = 0;
            while AceIndex < (*Dacl).AceCount as ULONG {
                if GetAce(
                    Dacl,
                    AceIndex,
                    core::ptr::addr_of_mut!(AceHeader) as *mut LPVOID,
                ) != 0
                    && AceHeader != core::ptr::null_mut()
                {
                    let mut LookupDeferred: BOOL = 0;
                    let mut PrincipalListItem: PPRINCIPAL_LISTITEM;

                    Sid = AceHeaderToSID(AceHeader);

                    PrincipalListItem = AddPrincipalToList(
                        sp,
                        Sid,
                        AceHeader,
                        core::ptr::addr_of_mut!(LookupDeferred),
                    );

                    if PrincipalListItem != core::ptr::null_mut() && LookupDeferred != 0 {
                        AddPrincipalListEntry(
                            sp,
                            PrincipalListItem,
                            -1,
                            (FALSE) as i32,
                        );
                    }
                }
                AceIndex += 1;
            }
        }
        LocalFree(SecurityDescriptor as HLOCAL);
    }
}
unsafe extern "C" fn UpdateControlStates(sp: PSECURITY_PAGE) {
    let Selected: PPRINCIPAL_LISTITEM = ListViewGetSelectedItemData((*sp).hWndPrincipalsList) as PPRINCIPAL_LISTITEM;

    EnableWindow(
        (*sp).hBtnRemove,
        if !Selected.is_null() { 1 } else { 0 },
    );
    EnableWindow(
        (*sp).hAceCheckList,
        if !Selected.is_null() { 1 } else { 0 },
    );

    if !Selected.is_null() {
        let mut szLabel: LPWSTR;
        let szDisplayString: LPWSTR;

        szDisplayString = GetPrincipalAccountNameString(Selected);
        if LoadAndFormatString(
            hDllInstance,
            (IDS_PERMISSIONS_FOR) as u32,
            core::ptr::addr_of_mut!(szLabel) as *mut LPVOID as *mut *mut u16,
            szDisplayString,
        ) != 0
        {
            SetWindowText((*sp).hPermissionsForLabel, szLabel);

            LocalFree(szLabel as HLOCAL);
        }

        LocalFree(szDisplayString as HLOCAL);
    } else {
        let mut szPermissions: [u16; 255] = [0; 255];

        if LoadString(
            hDllInstance,
            (IDS_PERMISSIONS) as u32,
            szPermissions.as_mut_ptr(),
            ((core::mem::size_of_val(&szPermissions) / core::mem::size_of::<u16>()) as u32) as i32,
        ) != 0
        {
            SetWindowText((*sp).hPermissionsForLabel, szPermissions.as_ptr());
        }

        SendMessage(
            (*sp).hAceCheckList,
            (CLM_CLEARCHECKBOXES) as u32,
            0,
            0,
        );
    }
}
unsafe extern "C" fn UpdatePrincipalInfo(
    sp: PSECURITY_PAGE,
    LookupInfo: PSIDLOOKUPNOTIFYINFO,
) {
    let mut CurItem: PPRINCIPAL_LISTITEM;
    let mut DisplayName: LPWSTR;

    if (*sp).OwnerSid != core::ptr::null_mut() &&
        EqualSid((*sp).OwnerSid, (*LookupInfo).Sid) != 0
    {
        if (*LookupInfo).SidRequestResult != core::ptr::null_mut() {
            DisplayName = GetDisplayStringFromSidRequestResult((*LookupInfo).SidRequestResult);
        } else if ConvertSidToStringSidW((*LookupInfo).Sid, &mut DisplayName) == 0 {
            DisplayName = core::ptr::null_mut();
        }

        if DisplayName != core::ptr::null_mut() {
            SetDlgItemTextW((*sp).hWnd, IDC_OWNER, DisplayName);

            LocalFree(DisplayName as HLOCAL);
        }
    }

    CurItem = (*sp).PrincipalsListHead;
    while CurItem != core::ptr::null_mut() {
        if EqualSid(
            CurItem.offset(1) as PSID,
            (*LookupInfo).Sid,
        ) != 0
        {
            let mut PrincipalIndex: INT;
            let mut lvfi: LVFINDINFO = core::mem::zeroed();

            lvfi.flags = (LVFI_PARAM) as u32;
            lvfi.lParam = CurItem as LPARAM;
            PrincipalIndex = ListView_FindItem!(
                (*sp).hWndPrincipalsList,
                -1 as INT,
                &mut lvfi,
            );

            if PrincipalIndex != -1 {
                UpdatePrincipalListItem(
                    sp,
                    PrincipalIndex,
                    CurItem,
                    (*LookupInfo).SidRequestResult,
                );

                if ListViewGetSelectedItemData((*sp).hWndPrincipalsList) == CurItem as LPARAM {
                    UpdateControlStates(sp);
                }
            } else {
                AddPrincipalListEntry(
                    sp,
                    CurItem,
                    -1,
                    (FALSE) as i32,
                );
            }
            break;
        }
        CurItem = (*CurItem).Next;
    }
}
#[cfg(target_arch = "x86")]
unsafe extern "stdcall" fn SecurityPageCallback(
    hwnd: HWND,
    uMsg: u32,
    ppsp: *mut PROPSHEETPAGEW,
) -> u32 {
    let sp = (*ppsp).lParam as PSECURITY_PAGE;

    match (uMsg) as INT {
        PSPCB_CREATE => {
            return TRUE;
        }
        PSPCB_RELEASE => {
            DestroySecurityPage(sp);
            return FALSE;
        }
        _ => {}
    }

    FALSE
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn SecurityPageCallback(
    hwnd: HWND,
    uMsg: u32,
    ppsp: *mut PROPSHEETPAGEW,
) -> u32 {
    let sp = (*ppsp).lParam as PSECURITY_PAGE;

    match (uMsg) as INT {
        PSPCB_CREATE => {
            return TRUE;
        }
        PSPCB_RELEASE => {
            DestroySecurityPage(sp);
            return FALSE;
        }
        _ => {}
    }

    FALSE
}
unsafe extern "C" fn SetAceCheckListColumns(
    hAceCheckList: HWND,
    Button: u32,
    hLabel: HWND,
) {
    let mut pt: POINT = core::mem::zeroed();
    let mut rcLabel: RECT = core::mem::zeroed();

    GetWindowRect(hLabel, &mut rcLabel);
    pt.y = 0;
    pt.x = (rcLabel.right - rcLabel.left) / 2;
    MapWindowPoints(
        hLabel,
        hAceCheckList,
        &mut pt,
        1,
    );

    SendMessage(
        hAceCheckList,
        (CLM_SETCHECKBOXCOLUMN) as u32,
        Button as WPARAM,
        pt.x as LPARAM,
    );
}
unsafe extern "C" fn LoadPermissionsList(
    sp: *mut _SECURITY_PAGE,
    GuidObjectType: *mut GUID,
    dwFlags: DWORD,
    DefaultAccess: *mut SI_ACCESS,
) {
    let mut hRet: HRESULT;
    let mut AccessList: PSI_ACCESS = core::ptr::null_mut();
    let mut nAccessList: ULONG = 0;
    let mut DefaultAccessIndex: ULONG = 0;
    let mut szSpecialPermissions: [WCHAR; 255] = [0; 255];
    let mut SpecialPermissionsPresent: BOOLEAN = (FALSE) as u8;
    let mut SpecialPermissionsMask: ACCESS_MASK = 0;

    SendMessage(
        (*sp).hAceCheckList,
        (CLM_CLEAR) as u32,
        0,
        0,
    );

    hRet = (*(*(*sp).psi).lpVtbl).GetAccessRights.unwrap_unchecked()(
        (*sp).psi,
        GuidObjectType,
        dwFlags,
        &mut AccessList,
        &mut nAccessList,
        &mut DefaultAccessIndex,
    );

    if ((hRet) >= 0) && nAccessList != 0 {
        let mut NameStr: LPCWSTR;
        let mut CurAccess: PSI_ACCESS;
        let mut NameBuffer: [WCHAR; (MAX_PATH) as usize] = [0; (MAX_PATH) as usize];

        if !DefaultAccess.is_null() {
            core::ptr::copy_nonoverlapping(AccessList.add(DefaultAccessIndex as usize), DefaultAccess, 1);
        }

        let LastAccess = AccessList.add(nAccessList as usize);
        CurAccess = AccessList.add(0 as usize);
        while CurAccess != LastAccess {
            if (*CurAccess).dwFlags & dwFlags != 0 {
                if IS_INTRESOURCE!((*CurAccess).pszName) {
                    if LoadString(
                        (*sp).ObjectInfo.hInstance,
                        (*CurAccess).pszName as UINT,
                        NameBuffer.as_mut_ptr(),
                        ((core::mem::size_of_val(&NameBuffer) / core::mem::size_of::<WCHAR>()) as DWORD) as i32,
                    ) == 0
                    {
                        LoadString(
                            hDllInstance,
                            (IDS_UNKNOWN) as u32,
                            NameBuffer.as_mut_ptr(),
                            (core::mem::size_of_val(&NameBuffer) / core::mem::size_of::<WCHAR>()) as i32,
                        );
                    }
                    NameStr = NameBuffer.as_ptr();
                } else {
                    NameStr = (*CurAccess).pszName;
                }

                SendMessage(
                    (*sp).hAceCheckList,
                    (CLM_ADDITEM) as u32,
                    (*CurAccess).mask as WPARAM,
                    NameStr as LPARAM,
                );
            } else if (*CurAccess).dwFlags & (SI_ACCESS_SPECIFIC) as u32 != 0 {
                SpecialPermissionsPresent = (TRUE) as u8;
                SpecialPermissionsMask |= (*CurAccess).mask;
            }
            CurAccess = CurAccess.add(1);
        }
    }

    if SpecialPermissionsPresent != 0
        && LoadString(
            hDllInstance,
            (IDS_SPECIAL_PERMISSIONS) as u32,
            szSpecialPermissions.as_mut_ptr(),
            (core::mem::size_of_val(&szSpecialPermissions) / core::mem::size_of::<WCHAR>()) as i32,
        ) != 0
    {
        (*sp).SpecialPermCheckIndex = SendMessage(
            (*sp).hAceCheckList,
            (CLM_ADDITEM) as u32,
            SpecialPermissionsMask as WPARAM,
            szSpecialPermissions.as_ptr() as LPARAM,
        ) as INT;

        if (*sp).SpecialPermCheckIndex != -1 {
            SendMessage(
                (*sp).hAceCheckList,
                (CLM_SETITEMSTATE) as u32,
                (*sp).SpecialPermCheckIndex as WPARAM,
                (CIS_ALLOWDISABLED | CIS_DENYDISABLED | CIS_NONE) as i64,
            );
        }
    }
}
unsafe extern "C" fn ResizeControls(
    sp: *mut SECURITY_PAGE,
    Width: i32,
    Height: i32,
) {
    let mut hWndAllow: HWND;
    let mut hWndDeny: HWND;
    let mut hWndOwnerEdit: HWND;
    let mut rcControl: RECT = core::mem::zeroed();
    let mut rcControl2: RECT = core::mem::zeroed();
    let mut rcControl3: RECT = core::mem::zeroed();
    let mut rcWnd: RECT = core::mem::zeroed();
    let mut cxWidth: i32;
    let mut cxEdge: i32;
    let mut btnSpacing: i32;
    let mut pt: POINT = core::mem::zeroed();
    let mut pt2: POINT = core::mem::zeroed();
    let mut dwp: HDWP;
    let mut nControls: i32 = 8;
    let mut lvc: LVCOLUMN = core::mem::zeroed();

    hWndAllow = GetDlgItem(
        (*sp).hWnd,
        IDC_LABEL_ALLOW,
    );
    hWndDeny = GetDlgItem(
        (*sp).hWnd,
        IDC_LABEL_DENY,
    );

    GetWindowRect(
        (*sp).hWnd,
        &mut rcWnd,
    );

    cxEdge = GetSystemMetrics(SM_CXEDGE);

    pt.x = 0;
    pt.y = 0;
    MapWindowPoints(
        (*sp).hWndPrincipalsList,
        (*sp).hWnd,
        &mut pt,
        1,
    );
    cxWidth = Width - (2 * pt.x);

    if (*sp).ObjectInfo.dwFlags & (SI_ADVANCED) as u32 != 0 {
        nControls += 2;
    }

    dwp = BeginDeferWindowPos(nControls);
    if dwp != core::ptr::null_mut() {
        hWndOwnerEdit = GetDlgItem(
            (*sp).hWnd,
            IDC_OWNER,
        );
        GetWindowRect(
            hWndOwnerEdit,
            &mut rcControl,
        );
        pt2.x = 0;
        pt2.y = 0;
        MapWindowPoints(
            hWndOwnerEdit,
            (*sp).hWnd,
            &mut pt2,
            1,
        );
        dwp = DeferWindowPos(
            dwp,
            hWndOwnerEdit,
            core::ptr::null_mut(),
            0,
            0,
            Width - pt.x - pt2.x,
            rcControl.bottom - rcControl.top,
            (SWP_NOMOVE | SWP_NOZORDER) as u32,
        );
        if dwp == core::ptr::null_mut() {
            return;
        }

        GetWindowRect(
            (*sp).hWndPrincipalsList,
            &mut rcControl,
        );
        dwp = DeferWindowPos(
            dwp,
            (*sp).hWndPrincipalsList,
            core::ptr::null_mut(),
            0,
            0,
            cxWidth,
            rcControl.bottom - rcControl.top,
            (SWP_NOMOVE | SWP_NOZORDER) as u32,
        );
        if dwp == core::ptr::null_mut() {
            return;
        }

        GetWindowRect(
            (*sp).hBtnAdd,
            &mut rcControl,
        );
        GetWindowRect(
            (*sp).hBtnRemove,
            &mut rcControl2,
        );
        btnSpacing = rcControl2.left - rcControl.right;
        pt2.x = 0;
        pt2.y = 0;
        MapWindowPoints(
            (*sp).hBtnAdd,
            (*sp).hWnd,
            &mut pt2,
            1,
        );
        dwp = DeferWindowPos(
            dwp,
            (*sp).hBtnAdd,
            core::ptr::null_mut(),
            pt.x + cxWidth - (rcControl2.right - rcControl2.left) -
                (rcControl.right - rcControl.left) -
                btnSpacing - cxEdge,
            pt2.y,
            0,
            0,
            (SWP_NOSIZE | SWP_NOZORDER) as u32,
        );
        if dwp == core::ptr::null_mut() {
            return;
        }

        pt2.x = 0;
        pt2.y = 0;
        MapWindowPoints(
            (*sp).hBtnRemove,
            (*sp).hWnd,
            &mut pt2,
            1,
        );
        dwp = DeferWindowPos(
            dwp,
            (*sp).hBtnRemove,
            core::ptr::null_mut(),
            pt.x + cxWidth - (rcControl2.right - rcControl2.left) - cxEdge,
            pt2.y,
            0,
            0,
            (SWP_NOSIZE | SWP_NOZORDER) as u32,
        );
        if dwp == core::ptr::null_mut() {
            return;
        }

        GetWindowRect(
            hWndAllow,
            &mut rcControl,
        );
        GetWindowRect(
            hWndDeny,
            &mut rcControl2,
        );
        GetWindowRect(
            (*sp).hPermissionsForLabel,
            &mut rcControl3,
        );
        pt2.x = 0;
        pt2.y = 0;
        MapWindowPoints(
            (*sp).hPermissionsForLabel,
            (*sp).hWnd,
            &mut pt2,
            1,
        );
        dwp = DeferWindowPos(
            dwp,
            (*sp).hPermissionsForLabel,
            core::ptr::null_mut(),
            0,
            0,
            cxWidth - (rcControl2.right - rcControl2.left) -
                (rcControl.right - rcControl.left) -
                (2 * btnSpacing) - cxEdge,
            rcControl3.bottom - rcControl3.top,
            (SWP_NOMOVE | SWP_NOZORDER) as u32,
        );
        if dwp == core::ptr::null_mut() {
            return;
        }

        pt2.x = 0;
        pt2.y = 0;
        MapWindowPoints(
            hWndAllow,
            (*sp).hWnd,
            &mut pt2,
            1,
        );
        dwp = DeferWindowPos(
            dwp,
            hWndAllow,
            core::ptr::null_mut(),
            cxWidth - (rcControl2.right - rcControl2.left) -
                (rcControl.right - rcControl.left) -
                btnSpacing - cxEdge,
            pt2.y,
            0,
            0,
            (SWP_NOSIZE | SWP_NOZORDER) as u32,
        );
        if dwp == core::ptr::null_mut() {
            return;
        }

        pt2.x = 0;
        pt2.y = 0;
        MapWindowPoints(
            hWndDeny,
            (*sp).hWnd,
            &mut pt2,
            1,
        );
        dwp = DeferWindowPos(
            dwp,
            hWndDeny,
            core::ptr::null_mut(),
            cxWidth - (rcControl2.right - rcControl2.left) - cxEdge,
            pt2.y,
            0,
            0,
            (SWP_NOSIZE | SWP_NOZORDER) as u32,
        );
        if dwp == core::ptr::null_mut() {
            return;
        }

        GetWindowRect(
            (*sp).hAceCheckList,
            &mut rcControl,
        );
        GetWindowRect(
            (*sp).hBtnAdvanced,
            &mut rcControl2,
        );
        GetWindowRect(
            GetDlgItem(
                (*sp).hWnd,
                IDC_LABEL_ADVANCED,
            ),
            &mut rcControl3,
        );
        dwp = DeferWindowPos(
            dwp,
            (*sp).hAceCheckList,
            core::ptr::null_mut(),
            0,
            0,
            cxWidth,
            if (*sp).ObjectInfo.dwFlags & (SI_ADVANCED) as u32 != 0 {
                Height - (rcControl.top - rcWnd.top) -
                    (rcControl3.bottom - rcControl3.top) - pt.x - btnSpacing
            } else {
                Height - (rcControl.top - rcWnd.top) - pt.x
            },
            (SWP_NOMOVE | SWP_NOZORDER) as u32,
        );
        if dwp == core::ptr::null_mut() {
            return;
        }

        if (*sp).ObjectInfo.dwFlags & (SI_ADVANCED) as u32 != 0 {
            dwp = DeferWindowPos(
                dwp,
                GetDlgItem(
                    (*sp).hWnd,
                    IDC_LABEL_ADVANCED,
                ),
                core::ptr::null_mut(),
                pt.x,
                Height - (rcControl3.bottom - rcControl3.top) - pt.x,
                cxWidth - (rcControl2.right - rcControl2.left) - cxEdge,
                rcControl3.bottom - rcControl3.top,
                (SWP_NOZORDER) as u32,
            );
            if dwp == core::ptr::null_mut() {
                return;
            }

            dwp = DeferWindowPos(
                dwp,
                (*sp).hBtnAdvanced,
                core::ptr::null_mut(),
                cxWidth - (rcControl2.right - rcControl2.left) + pt.x,
                Height - (rcControl2.bottom - rcControl2.top) - pt.x,
                0,
                0,
                (SWP_NOSIZE | SWP_NOZORDER) as u32,
            );
            if dwp == core::ptr::null_mut() {
                return;
            }
        }

        EndDeferWindowPos(dwp);
    }

    GetClientRect(
        (*sp).hWndPrincipalsList,
        &mut rcControl,
    );
    lvc.mask = (LVCF_WIDTH) as u32;
    lvc.cx = rcControl.right;
    ListView_SetColumn!(
        (*sp).hWndPrincipalsList,
        0,
        &lvc,
    );

    SetAceCheckListColumns(
        (*sp).hAceCheckList,
        (CLB_ALLOW) as u32,
        hWndAllow,
    );
    SetAceCheckListColumns(
        (*sp).hAceCheckList,
        (CLB_DENY) as u32,
        hWndDeny,
    );
}
unsafe extern "C" fn BuildDefaultPrincipalAce(
    sp: *mut SECURITY_PAGE,
    pSid: PSID,
) -> PACE_HEADER {
    let mut Ace: PACCESS_ALLOWED_ACE;
    let mut SidLen: DWORD;
    let mut AceSize: WORD;

    SidLen = GetLengthSid(pSid);
    AceSize = (FIELD_OFFSET!(ACCESS_ALLOWED_ACE, SidStart) + (SidLen as usize) as i32) as WORD;
    Ace = HeapAlloc(
        GetProcessHeap(),
        0,
        AceSize as usize as SIZE_T,
    ) as PACCESS_ALLOWED_ACE;
    if !Ace.is_null() {
        (*Ace).Header.AceType = (ACCESS_ALLOWED_ACE_TYPE) as u8;
        (*Ace).Header.AceFlags = 0;
        (*Ace).Header.AceSize = AceSize;
        (*Ace).Mask = (*sp).DefaultAccess.mask;

        if CopySid(
            SidLen,
            core::ptr::addr_of_mut!((*Ace).SidStart) as PSID,
            pSid,
        ) != 0
        {
            return core::ptr::addr_of_mut!((*Ace).Header);
        }

        HeapFree(
            GetProcessHeap(),
            0,
            Ace as LPVOID,
        );
    }

    core::ptr::null_mut()
}
unsafe extern "C" fn AddSelectedPrincipal(
    pDsObjectPicker: *mut core::ffi::c_void,
    hwndParent: HWND,
    pSid: PSID,
    Context: PVOID,
) -> BOOL {
    let sp = Context as PSECURITY_PAGE;

    let AceHeader = BuildDefaultPrincipalAce(sp, pSid);
    if AceHeader != core::ptr::null_mut() {
        let mut LookupDeferred: BOOL = 0;

        let PrincipalListItem = AddPrincipalToList(
            sp,
            pSid,
            AceHeader,
            &mut LookupDeferred,
        );

        if PrincipalListItem != core::ptr::null_mut() && LookupDeferred != 0 {
            AddPrincipalListEntry(
                sp,
                PrincipalListItem,
                -1,
                (FALSE) as i32,
            );
        }

        HeapFree(
            GetProcessHeap(),
            0,
            (AceHeader).cast(),
        );
    }

    (TRUE) as i32
}
#[cfg(target_arch = "x86")]
unsafe extern "stdcall" fn SecurityPageProc(
    hwndDlg: HWND,
    uMsg: u32,
    wParam: WPARAM,
    lParam: LPARAM,
) -> INT_PTR {
    let mut sp: PSECURITY_PAGE;
    let mut Ret: INT_PTR = FALSE;

    sp = GetWindowLongPtr(hwndDlg, DWLP_USER) as PSECURITY_PAGE;
    if !sp.is_null() || uMsg == WM_INITDIALOG {
        match (uMsg) as INT {
            WM_NOTIFY => {
                let pnmh = lParam as *mut NMHDR;

                if (*pnmh).hwndFrom == (*sp).hWndPrincipalsList {
                    match ((*pnmh).code) as INT {
                        LVN_ITEMCHANGED => {
                            let pnmv = lParam as LPNMLISTVIEW;

                            if ((*pnmv).uChanged & LVIF_STATE) != 0 &&
                                (((*pnmv).uOldState & (LVIS_FOCUSED | LVIS_SELECTED)) != 0 ||
                                 ((*pnmv).uNewState & (LVIS_FOCUSED | LVIS_SELECTED)) != 0) {
                                UpdateControlStates(sp);
                            }
                        }
                        _ => {}
                    }
                } else if (*pnmh).hwndFrom == (*sp).hAceCheckList {
                    match ((*pnmh).code) as INT {
                        CLN_CHANGINGITEMCHECKBOX => {
                            let pcicb = lParam as PNMCHANGEITEMCHECKBOX;

                            if (*pcicb).Checked != 0 {
                                (*pcicb).NewState &= !((if (*pcicb).CheckBox != CLB_DENY { CIS_DENY } else { CIS_ALLOW }));
                            }
                        }
                        _ => {}
                    }
                } else if (*pnmh).hwndFrom == (*sp).hWnd {
                    match ((*pnmh).code) as INT {
                        SIDN_LOOKUPSUCCEEDED => {
                            let LookupInfo = ((lParam as usize).wrapping_sub(core::mem::offset_of!(SIDLOOKUPNOTIFYINFO, nmh))) as PSIDLOOKUPNOTIFYINFO;

                            UpdatePrincipalInfo(sp, LookupInfo);
                        }
                        _ => {}
                    }
                }
            }

            WM_COMMAND => {
                match (LOWORD!(wParam)) as INT {
                    IDC_ADD_PRINCIPAL => {
                        let mut hRet: HRESULT;

                        hRet = InitializeObjectPicker((*sp).ServerName,
                                                      core::ptr::addr_of_mut!((*sp).ObjectInfo),
                                                      core::ptr::addr_of_mut!((*sp).pDsObjectPicker));
                        if hRet >= 0 {
                            hRet = InvokeObjectPickerDialog((*sp).pDsObjectPicker,
                                                            hwndDlg,
                                                            AddSelectedPrincipal,
                                                            sp as LPVOID);
                            if hRet < 0 {
                                MessageBox(hwndDlg, w!("InvokeObjectPickerDialog failed!\n") as LPCWSTR, core::ptr::null_mut(), 0);
                            }

                            FreeObjectPicker((*sp).pDsObjectPicker);
                        } else {
                            MessageBox(hwndDlg, w!("InitializeObjectPicker failed!\n") as LPCWSTR, core::ptr::null_mut(), 0);
                        }
                    }

                    IDC_REMOVE_PRINCIPAL => {
                        let SelectedPrincipal: PPRINCIPAL_LISTITEM;

                        SelectedPrincipal = ListViewGetSelectedItemData((*sp).hWndPrincipalsList) as PPRINCIPAL_LISTITEM;
                        if !SelectedPrincipal.is_null() {
                        }
                    }
                    _ => {}
                }
            }

            WM_SIZE => {
                ResizeControls(sp,
                               LOWORD!(lParam) as i32,
                               HIWORD!(lParam) as i32);
            }

            WM_INITDIALOG => {
                sp = (*(lParam as LPPROPSHEETPAGE)).lParam as PSECURITY_PAGE;
                if !sp.is_null() {
                    let mut lvc: LV_COLUMN = core::mem::zeroed();
                    let mut rcLvClient: RECT = core::mem::zeroed();

                    (*sp).hWnd = hwndDlg;
                    (*sp).hWndPrincipalsList = GetDlgItem(hwndDlg, IDC_PRINCIPALS);
                    (*sp).hBtnAdd = GetDlgItem(hwndDlg, IDC_ADD_PRINCIPAL);
                    (*sp).hBtnRemove = GetDlgItem(hwndDlg, IDC_REMOVE_PRINCIPAL);
                    (*sp).hBtnAdvanced = GetDlgItem(hwndDlg, IDC_ADVANCED);
                    (*sp).hAceCheckList = GetDlgItem(hwndDlg, IDC_ACE_CHECKLIST);
                    (*sp).hPermissionsForLabel = GetDlgItem(hwndDlg, IDC_LABEL_PERMISSIONS_FOR);

                    (*sp).SpecialPermCheckIndex = -1;

                    SetWindowLongPtr(hwndDlg, DWLP_USER, sp as DWORD_PTR);

                    ListView_SetExtendedListViewStyleEx!((*sp).hWndPrincipalsList,
                                                              LVS_EX_FULLROWSELECT,
                                                              LVS_EX_FULLROWSELECT);

                    (*sp).hiPrincipals = ImageList_Create(16,
                                                        16,
                                                        ILC_COLOR32 | ILC_MASK,
                                                        0,
                                                        3);
                    if !(*sp).hiPrincipals.is_null() {
                        let hbmImages: HBITMAP;

                        hbmImages = LoadBitmap(hDllInstance,
                                               MAKEINTRESOURCE!(IDB_USRGRPIMAGES));
                        if !hbmImages.is_null() {
                            ImageList_AddMasked((*sp).hiPrincipals,
                                                hbmImages,
                                                RGB!(255,
                                                    0,
                                                    255));

                            DeleteObject((hbmImages).cast());
                        }
                    }

                    if !(*sp).hiPrincipals.is_null() {
                        ListView_SetImageList!((*sp).hWndPrincipalsList,
                                                    (*sp).hiPrincipals,
                                                    LVSIL_SMALL);
                    }

                    GetClientRect((*sp).hWndPrincipalsList,
                                  core::ptr::addr_of_mut!(rcLvClient));

                    lvc.mask = LVCF_FMT | LVCF_WIDTH;
                    lvc.fmt = LVCFMT_LEFT;
                    lvc.cx = rcLvClient.right;
                    ListView_InsertColumn!((*sp).hWndPrincipalsList,
                                                0,
                                                core::ptr::addr_of_mut!(lvc));

                    ReloadPrincipalsList(sp);

                    ListViewSelectItem((*sp).hWndPrincipalsList,
                                       0);

                    SetAceCheckListColumns((*sp).hAceCheckList,
                                           CLB_ALLOW,
                                           GetDlgItem(hwndDlg, IDC_LABEL_ALLOW));
                    SetAceCheckListColumns((*sp).hAceCheckList,
                                           CLB_DENY,
                                           GetDlgItem(hwndDlg, IDC_LABEL_DENY));

                    LoadPermissionsList(sp,
                                        core::ptr::null_mut(),
                                        SI_ACCESS_GENERAL |
                                        (if ((*sp).ObjectInfo.dwFlags & SI_CONTAINER) != 0 { SI_ACCESS_CONTAINER } else { 0 }),
                                        core::ptr::addr_of_mut!((*sp).DefaultAccess));

                    if ((*sp).ObjectInfo.dwFlags & SI_ADVANCED) != 0 {
                        (*sp).ObjectInfo.dwFlags |= SI_EDIT_PERMS;
                    } else {
                        ShowWindow((*sp).hBtnAdvanced,
                                   SW_HIDE);
                        ShowWindow(GetDlgItem(hwndDlg, IDC_LABEL_ADVANCED),
                                   SW_HIDE);
                    }

                    SendMessage((*sp).hAceCheckList,
                                CLM_ENABLEQUICKSEARCH,
                                TRUE,
                                0);

                    UpdateControlStates(sp);
                }

                Ret = TRUE;
            }
            _ => {}
        }
    }
    Ret
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn SecurityPageProc(
    hwndDlg: HWND,
    uMsg: u32,
    wParam: WPARAM,
    lParam: LPARAM,
) -> INT_PTR {
    let mut sp: PSECURITY_PAGE;
    let mut Ret: INT_PTR = (FALSE) as i64;

    sp = GetWindowLongPtr(hwndDlg, DWLP_USER) as PSECURITY_PAGE;
    if !sp.is_null() || uMsg == (WM_INITDIALOG) as u32 {
        match (uMsg) as INT {
            WM_NOTIFY => {
                let pnmh = lParam as *mut NMHDR;

                if (*pnmh).hwndFrom == (*sp).hWndPrincipalsList {
                    match ((*pnmh).code) as INT {
                        LVN_ITEMCHANGED => {
                            let pnmv = lParam as LPNMLISTVIEW;

                            if ((*pnmv).uChanged & (LVIF_STATE) as u32) != 0 &&
                                (((*pnmv).uOldState & ((LVIS_FOCUSED | LVIS_SELECTED)) as u32) != 0 ||
                                 ((*pnmv).uNewState & ((LVIS_FOCUSED | LVIS_SELECTED)) as u32) != 0) {
                                UpdateControlStates(sp);
                            }
                        }
                        _ => {}
                    }
                } else if (*pnmh).hwndFrom == (*sp).hAceCheckList {
                    match ((*pnmh).code) as INT {
                        CLN_CHANGINGITEMCHECKBOX => {
                            let pcicb = lParam as PNMCHANGEITEMCHECKBOX;

                            if (*pcicb).Checked != 0 {
                                (*pcicb).NewState &= (!((if (*pcicb).CheckBox != (CLB_DENY) as u32 { CIS_DENY } else { CIS_ALLOW }))) as u32;
                            }
                        }
                        _ => {}
                    }
                } else if (*pnmh).hwndFrom == (*sp).hWnd {
                    match ((*pnmh).code) as INT {
                        SIDN_LOOKUPSUCCEEDED => {
                            let LookupInfo = ((lParam as usize).wrapping_sub(core::mem::offset_of!(SIDLOOKUPNOTIFYINFO, nmh))) as PSIDLOOKUPNOTIFYINFO;

                            UpdatePrincipalInfo(sp, LookupInfo);
                        }
                        _ => {}
                    }
                }
            }

            WM_COMMAND => {
                match (LOWORD!(wParam)) as INT {
                    IDC_ADD_PRINCIPAL => {
                        let mut hRet: HRESULT;

                        hRet = InitializeObjectPicker((*sp).ServerName,
                                                      core::ptr::addr_of_mut!((*sp).ObjectInfo),
                                                      core::ptr::addr_of_mut!((*sp).pDsObjectPicker));
                        if hRet >= 0 {
                            hRet = InvokeObjectPickerDialog((*sp).pDsObjectPicker,
                                                            hwndDlg,
                                                            AddSelectedPrincipal,
                                                            sp as LPVOID);
                            if hRet < 0 {
                                MessageBox(hwndDlg, w!("InvokeObjectPickerDialog failed!\n") as LPCWSTR, core::ptr::null_mut(), 0);
                            }

                            FreeObjectPicker((*sp).pDsObjectPicker);
                        } else {
                            MessageBox(hwndDlg, w!("InitializeObjectPicker failed!\n") as LPCWSTR, core::ptr::null_mut(), 0);
                        }
                    }

                    IDC_REMOVE_PRINCIPAL => {
                        let SelectedPrincipal: PPRINCIPAL_LISTITEM;

                        SelectedPrincipal = ListViewGetSelectedItemData((*sp).hWndPrincipalsList) as PPRINCIPAL_LISTITEM;
                        if !SelectedPrincipal.is_null() {
                        }
                    }
                    _ => {}
                }
            }

            WM_SIZE => {
                ResizeControls(sp,
                               LOWORD!(lParam) as i32,
                               HIWORD!(lParam) as i32);
            }

            WM_INITDIALOG => {
                sp = (*(lParam as LPPROPSHEETPAGE)).lParam as PSECURITY_PAGE;
                if !sp.is_null() {
                    let mut lvc: LV_COLUMN = core::mem::zeroed();
                    let mut rcLvClient: RECT = core::mem::zeroed();

                    (*sp).hWnd = hwndDlg;
                    (*sp).hWndPrincipalsList = GetDlgItem(hwndDlg, IDC_PRINCIPALS);
                    (*sp).hBtnAdd = GetDlgItem(hwndDlg, IDC_ADD_PRINCIPAL);
                    (*sp).hBtnRemove = GetDlgItem(hwndDlg, IDC_REMOVE_PRINCIPAL);
                    (*sp).hBtnAdvanced = GetDlgItem(hwndDlg, IDC_ADVANCED);
                    (*sp).hAceCheckList = GetDlgItem(hwndDlg, IDC_ACE_CHECKLIST);
                    (*sp).hPermissionsForLabel = GetDlgItem(hwndDlg, IDC_LABEL_PERMISSIONS_FOR);

                    (*sp).SpecialPermCheckIndex = -1;

                    SetWindowLongPtr(hwndDlg, DWLP_USER, (sp as DWORD_PTR) as i64);

                    ListView_SetExtendedListViewStyleEx!((*sp).hWndPrincipalsList,
                                                              LVS_EX_FULLROWSELECT,
                                                              LVS_EX_FULLROWSELECT);

                    (*sp).hiPrincipals = ImageList_Create(16,
                                                        16,
                                                        (ILC_COLOR32 | ILC_MASK) as u32,
                                                        0,
                                                        3);
                    if !(*sp).hiPrincipals.is_null() {
                        let hbmImages: HBITMAP;

                        hbmImages = LoadBitmap(hDllInstance,
                                               MAKEINTRESOURCE!(IDB_USRGRPIMAGES));
                        if !hbmImages.is_null() {
                            ImageList_AddMasked((*sp).hiPrincipals,
                                                hbmImages,
                                                RGB!(255,
                                                    0,
                                                    255));

                            DeleteObject((hbmImages).cast());
                        }
                    }

                    if !(*sp).hiPrincipals.is_null() {
                        ListView_SetImageList!((*sp).hWndPrincipalsList,
                                                    (*sp).hiPrincipals,
                                                    LVSIL_SMALL);
                    }

                    GetClientRect((*sp).hWndPrincipalsList,
                                  core::ptr::addr_of_mut!(rcLvClient));

                    lvc.mask = (LVCF_FMT | LVCF_WIDTH) as u32;
                    lvc.fmt = LVCFMT_LEFT;
                    lvc.cx = rcLvClient.right;
                    ListView_InsertColumn!((*sp).hWndPrincipalsList,
                                                0,
                                                core::ptr::addr_of_mut!(lvc));

                    ReloadPrincipalsList(sp);

                    ListViewSelectItem((*sp).hWndPrincipalsList,
                                       0);

                    SetAceCheckListColumns((*sp).hAceCheckList,
                                           (CLB_ALLOW) as u32,
                                           GetDlgItem(hwndDlg, IDC_LABEL_ALLOW));
                    SetAceCheckListColumns((*sp).hAceCheckList,
                                           (CLB_DENY) as u32,
                                           GetDlgItem(hwndDlg, IDC_LABEL_DENY));

                    LoadPermissionsList(sp,
                                        core::ptr::null_mut(),
                                        (SI_ACCESS_GENERAL |
                                        (if ((*sp).ObjectInfo.dwFlags & (SI_CONTAINER) as u32) != 0 { SI_ACCESS_CONTAINER } else { 0 })) as u32,
                                        core::ptr::addr_of_mut!((*sp).DefaultAccess));

                    if ((*sp).ObjectInfo.dwFlags & (SI_ADVANCED) as u32) != 0 {
                        (*sp).ObjectInfo.dwFlags |= (SI_EDIT_PERMS) as u32;
                    } else {
                        ShowWindow((*sp).hBtnAdvanced,
                                   SW_HIDE);
                        ShowWindow(GetDlgItem(hwndDlg, IDC_LABEL_ADVANCED),
                                   SW_HIDE);
                    }

                    SendMessage((*sp).hAceCheckList,
                                CLM_ENABLEQUICKSEARCH,
                                TRUE,
                                0);

                    UpdateControlStates(sp);
                }

                Ret = (TRUE) as i64;
            }
            _ => {}
        }
    }
    Ret
}

#[no_mangle]
pub unsafe extern "system" fn CreateSecurityPage(psi: LPSECURITYINFO) -> HPROPSHEETPAGE {
    let mut psp: PROPSHEETPAGE = core::mem::zeroed();
    let mut sPage: PSECURITY_PAGE;
    let mut ObjectInfo: SI_OBJECT_INFO = core::mem::zeroed();
    let mut SidCacheMgr: HANDLE;
    let mut SystemName: LPCWSTR = core::ptr::null();
    let mut hRet: HRESULT;

    if psi.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER);

        // DPRINT removed
        return core::ptr::null_mut();
    }

    hRet = ((*(*psi).lpVtbl).GetObjectInformation.unwrap_unchecked())(psi, &mut ObjectInfo);

    if hRet < 0 {
        SetLastError((hRet) as u32);

        // DPRINT removed
        return core::ptr::null_mut();
    }

    if (ObjectInfo.dwFlags & (SI_SERVER_IS_DC) as u32) != 0
        && !ObjectInfo.pszServerName.is_null()
        && *ObjectInfo.pszServerName.offset(0) != 0
    {
        SystemName = ObjectInfo.pszServerName;
    }

    SidCacheMgr = CreateSidCacheMgr(GetProcessHeap(), SystemName);
    if SidCacheMgr.is_null() {
        // DPRINT removed
        return core::ptr::null_mut();
    }

    hRet = CoInitialize(core::ptr::null_mut());
    if hRet < 0 {
        DestroySidCacheMgr(SidCacheMgr);
        // DPRINT removed
        return core::ptr::null_mut();
    }

    sPage = HeapAlloc(
        GetProcessHeap(),
        HEAP_ZERO_MEMORY,
        core::mem::size_of::<SECURITY_PAGE>() as SIZE_T,
    ) as PSECURITY_PAGE;
    if sPage.is_null() {
        DestroySidCacheMgr(SidCacheMgr);
        CoUninitialize();

        SetLastError(ERROR_NOT_ENOUGH_MEMORY);

        // DPRINT removed
        return core::ptr::null_mut();
    }

    ZeroMemory!(
        sPage as *mut core::ffi::c_void,
        core::mem::size_of_val(&*sPage),
    );

    (*sPage).psi = psi;
    (*sPage).ObjectInfo = ObjectInfo;
    (*sPage).ServerName = SystemName;
    (*sPage).SidCacheMgr = SidCacheMgr;

    psp.dwSize = core::mem::size_of::<PROPSHEETPAGE>() as u32;
    psp.dwFlags = (PSP_USECALLBACK) as u32;
    psp.hInstance = hDllInstance;
    *psp.__bindgen_anon_1.pszTemplate.as_mut() = MAKEINTRESOURCE!(IDD_SECPAGE);
    psp.pfnDlgProc = Some(SecurityPageProc);
    psp.lParam = sPage as LPARAM;
    psp.pfnCallback = Some(SecurityPageCallback);

    if (ObjectInfo.dwFlags & (SI_PAGE_TITLE) as u32) != 0 {
        psp.pszTitle = ObjectInfo.pszPageTitle;

        if !psp.pszTitle.is_null() {
            psp.dwFlags |= (PSP_USETITLE) as u32;
        }
    } else {
        psp.pszTitle = core::ptr::null();
    }

    CreatePropertySheetPage(&psp)
}

#[no_mangle]
pub unsafe extern "system" fn EditSecurity(
    hwndOwner: HWND,
    psi: LPSECURITYINFO,
) -> BOOL {
    let mut hRet: HRESULT;
    let mut ObjectInfo: SI_OBJECT_INFO = core::mem::zeroed();
    let mut psh: PROPSHEETHEADER;
    let mut hPages: [HPROPSHEETPAGE; 1] = core::mem::zeroed();
    let mut lpCaption: LPWSTR = core::ptr::null_mut();
    let Ret: BOOL;

    if psi.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER);

        // DPRINT removed
        return (FALSE) as i32;
    }

    hRet = ((*(*psi).lpVtbl).GetObjectInformation.unwrap_unchecked())(
        psi,
        &mut ObjectInfo,
    );

    if hRet < 0 {
        SetLastError((hRet) as u32);

        // DPRINT removed
        return (FALSE) as i32;
    }

    hPages[0] = CreateSecurityPage(psi);
    if hPages[0].is_null() {
        // DPRINT removed
        return (FALSE) as i32;
    }

    psh.dwSize = core::mem::size_of::<PROPSHEETHEADER>() as u32;
    psh.dwFlags = (PSH_DEFAULT) as u32;
    psh.hwndParent = hwndOwner;
    psh.hInstance = hDllInstance;

    if LoadAndFormatString(
        hDllInstance,
        (IDS_PSP_TITLE) as u32,
        &mut lpCaption,
        ObjectInfo.pszObjectName,
    ) != 0
    {
        psh.pszCaption = lpCaption;
    } else {
        psh.pszCaption = ObjectInfo.pszObjectName;
    }

    psh.nPages = (core::mem::size_of_val(&hPages) / core::mem::size_of::<HPROPSHEETPAGE>()) as u32;
    *psh.__bindgen_anon_2.nStartPage.as_mut() = 0;
    *psh.__bindgen_anon_3.phpage.as_mut() = hPages.as_mut_ptr();

    Ret = if PropertySheet(&mut psh) != -1 { 1 } else { 0 };

    if !lpCaption.is_null() {
        LocalFree(lpCaption as HLOCAL);
    }

    Ret
}

#[no_mangle]
pub unsafe extern "system" fn DllMain(
    hinstDLL: HINSTANCE,
    dwReason: u32,
    lpvReserved: LPVOID,
) -> i32 {
    match dwReason {
        DLL_PROCESS_ATTACH => {
            hDllInstance = hinstDLL;

            DisableThreadLibraryCalls(hinstDLL);

            if RegisterCheckListControl(hinstDLL) == 0 {
                // DPRINT removed
                return (FALSE) as i32;
            }
        }

        DLL_PROCESS_DETACH => {
            UnregisterCheckListControl(hinstDLL);
        }

        _ => {}
    }

    return (TRUE) as i32;
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * ReactOS Access Control List Editor
//  * Copyright (C) 2004-2005 ReactOS Team
//  *
//  * This library is free software; you can redistribute it and/or
//  * modify it under the terms of the GNU Lesser General Public
//  * License as published by the Free Software Foundation; either
//  * version 2.1 of the License, or (at your option) any later version.
//  *
//  * This library is distributed in the hope that it will be useful,
//  * but WITHOUT ANY WARRANTY; without even the implied warranty of
//  * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU
//  * Lesser General Public License for more details.
//  *
//  * You should have received a copy of the GNU Lesser General Public
//  * License along with this library; if not, write to the Free Software
//  * Foundation, Inc., 51 Franklin Street, Fifth Floor, Boston, MA  02110-1301  USA
//  */
// /*
//  * PROJECT:         ReactOS Access Control List Editor
//  * FILE:            lib/aclui/aclui.c
//  * PURPOSE:         Access Control List Editor
//  * PROGRAMMER:      Thomas Weidenmueller <w3seek@reactos.com>
//  *
//  * UPDATE HISTORY:
//  *      08/10/2004  Created
//  */
//
// #include "precomp.h"
//
// #include <sddl.h>
//
// #define NDEBUG
// #include <debug.h>
//
// #include "resource.h"
//
// HINSTANCE hDllInstance;
//
// #define SIDN_LOOKUPSUCCEEDED    (0x101)
// typedef struct _SIDLOOKUPNOTIFYINFO
// {
//     NMHDR nmh;
//     PSID Sid;
//     PSIDREQRESULT SidRequestResult;
// } SIDLOOKUPNOTIFYINFO, *PSIDLOOKUPNOTIFYINFO;
//
// static PSID
// AceHeaderToSID( PACE_HEADER AceHeader)
// {
//     PSID Sid = NULL;
//     switch (AceHeader->AceType)
//     {
//         case ACCESS_ALLOWED_ACE_TYPE:
//             Sid = (PSID)&((PACCESS_ALLOWED_ACE)AceHeader)->SidStart;
//             break;
//         case ACCESS_ALLOWED_OBJECT_ACE_TYPE:
//             Sid = (PSID)&((PACCESS_ALLOWED_OBJECT_ACE)AceHeader)->SidStart;
//             break;
//         case ACCESS_DENIED_ACE_TYPE:
//             Sid = (PSID)&((PACCESS_DENIED_ACE)AceHeader)->SidStart;
//             break;
//         case SYSTEM_AUDIT_OBJECT_ACE_TYPE:
//             Sid = (PSID)&((PACCESS_DENIED_OBJECT_ACE)AceHeader)->SidStart;
//             break;
//     }
//
//     return Sid;
// }
//
// static VOID
// DestroySecurityPage( PSECURITY_PAGE sp)
// {
//     DestroySidCacheMgr(sp->SidCacheMgr);
//
//     if (sp->OwnerSid != NULL)
//         LocalFree((HLOCAL)sp->OwnerSid);
//
//     HeapFree(GetProcessHeap(),
//              0,
//              sp);
//
//     CoUninitialize();
// }
//
// static VOID
// FreePrincipalsList( PSECURITY_PAGE sp,
//                     PPRINCIPAL_LISTITEM *PrincipalsListHead)
// {
//     PPRINCIPAL_LISTITEM CurItem, NextItem;
//     PACE_ENTRY AceEntry, NextAceEntry;
//
//     CurItem = *PrincipalsListHead;
//     while (CurItem != NULL)
//     {
//         /* Free all ACEs */
//         AceEntry = CurItem->ACEs;
//         while (AceEntry != NULL)
//         {
//             NextAceEntry = AceEntry->Next;
//             HeapFree(GetProcessHeap(),
//                      0,
//                      AceEntry);
//             AceEntry = NextAceEntry;
//         }
//
//         /* free the SID string if present */
//         if (CurItem->SidReqResult != NULL)
//         {
//             DereferenceSidReqResult(sp->SidCacheMgr,
//                                     CurItem->SidReqResult);
//         }
//
//         if (CurItem->DisplayString != NULL)
//         {
//             LocalFree((HLOCAL)CurItem->DisplayString);
//         }
//
//         /* free the ACE list item */
//         NextItem = CurItem->Next;
//         HeapFree(GetProcessHeap(),
//                  0,
//                  CurItem);
//         CurItem = NextItem;
//     }
//
//     *PrincipalsListHead = NULL;
// }
//
// static PACE_ENTRY
// AddAceToPrincipal( PPRINCIPAL_LISTITEM Principal,
//                    PACE_HEADER AceHeader)
// {
//     PACE_ENTRY AceEntry, *AceLink;
//
//     AceEntry = HeapAlloc(GetProcessHeap(),
//                          0,
//                          sizeof(ACE_ENTRY) + AceHeader->AceSize);
//     if (AceEntry != NULL)
//     {
//         AceEntry->Next = NULL;
//
//         /* copy the ACE */
//         CopyMemory!(AceEntry + 1,
//                    AceHeader,
//                    AceHeader->AceSize);
//
//         /* append it to the list */
//         AceLink = &Principal->ACEs;
//         while (*AceLink != NULL)
//         {
//             AceLink = &(*AceLink)->Next;
//         }
//         *AceLink = AceEntry;
//     }
//
//     return AceEntry;
// }
//
// static PPRINCIPAL_LISTITEM
// FindSidInPrincipalsListAddAce( PPRINCIPAL_LISTITEM PrincipalsListHead,
//                                PSID Sid,
//                                PACE_HEADER AceHeader)
// {
//     PPRINCIPAL_LISTITEM CurItem;
//
//     for (CurItem = PrincipalsListHead;
//          CurItem != NULL;
//          CurItem = CurItem->Next)
//     {
//         if (EqualSid((PSID)(CurItem + 1),
//                      Sid))
//         {
//             if (AddAceToPrincipal(CurItem,
//                                   AceHeader) != NULL)
//             {
//                 return CurItem;
//             }
//
//             /* unable to add the ACE to the principal */
//             break;
//         }
//     }
//
//     return NULL;
// }
//
// static VOID
// SidLookupCompletion( HANDLE SidCacheMgr,
//                      PSID Sid,
//                      PSIDREQRESULT SidRequestResult,
//                      PVOID Context)
// {
//     PSECURITY_PAGE sp = (PSECURITY_PAGE)Context;
//
//     /* NOTE: this routine may be executed in a different thread
//              than the GUI! */
//
//     if (SidRequestResult != NULL)
//     {
//         SIDLOOKUPNOTIFYINFO LookupInfo;
//
//         LookupInfo.nmh.hwndFrom = sp->hWnd;
//         LookupInfo.nmh.idFrom = 0;
//         LookupInfo.nmh.code = SIDN_LOOKUPSUCCEEDED;
//         LookupInfo.Sid = Sid;
//         LookupInfo.SidRequestResult = SidRequestResult;
//
//         /* notify the page that the sid lookup succeeded */
//         SendMessage(sp->hWnd,
//                     WM_NOTIFY,
//                     (WPARAM)LookupInfo.nmh.idFrom,
//                     (LPARAM)&LookupInfo.nmh);
//     }
// }
//
// static PPRINCIPAL_LISTITEM
// AddPrincipalToList( PSECURITY_PAGE sp,
//                     PSID Sid,
//                     PACE_HEADER AceHeader,
//                     BOOL *LookupDeferred  )
// {
//     PPRINCIPAL_LISTITEM PrincipalListItem = NULL, *PrincipalLink;
//     PACE_ENTRY AceEntry;
//     BOOL Deferred = FALSE;
//
//     if (!FindSidInPrincipalsListAddAce(sp->PrincipalsListHead,
//                                        Sid,
//                                        AceHeader))
//     {
//         DWORD SidLength;
//
//         PrincipalLink = &sp->PrincipalsListHead;
//         while (*PrincipalLink != NULL)
//         {
//             PrincipalLink = &(*PrincipalLink)->Next;
//         }
//
//         SidLength = GetLengthSid(Sid);
//
//         /* allocate the principal */
//         PrincipalListItem = HeapAlloc(GetProcessHeap(),
//                                       0,
//                                       sizeof(PRINCIPAL_LISTITEM) + SidLength);
//         if (PrincipalListItem != NULL)
//         {
//             PrincipalListItem->DisplayString = NULL;
//             PrincipalListItem->SidReqResult = NULL;
//
//             CopySid(SidLength,
//                     (PSID)(PrincipalListItem + 1),
//                     Sid);
//
//             /* allocate some memory for the ACE and copy it */
//             AceEntry = HeapAlloc(GetProcessHeap(),
//                                  0,
//                                  sizeof(ACE_ENTRY) + AceHeader->AceSize);
//             if (AceEntry != NULL)
//             {
//                 AceEntry->Next = NULL;
//                 CopyMemory!(AceEntry + 1,
//                            AceHeader,
//                            AceHeader->AceSize);
//
//                 /* add the ACE to the list */
//                 PrincipalListItem->ACEs = AceEntry;
//
//                 PrincipalListItem->Next = NULL;
//
//                 /* append item to the principals list */
//                 *PrincipalLink = PrincipalListItem;
//
//                 /* lookup the SID now */
//                 Deferred = !LookupSidCache(sp->SidCacheMgr,
//                                            Sid,
//                                            SidLookupCompletion,
//                                            sp);
//             }
//             else
//             {
//                 HeapFree(GetProcessHeap(),
//                          0,
//                          PrincipalListItem);
//                 PrincipalListItem = NULL;
//             }
//         }
//     }
//
//     if (PrincipalListItem != NULL && LookupDeferred != NULL)
//     {
//         *LookupDeferred = Deferred;
//     }
//
//     return PrincipalListItem;
// }
//
// static LPWSTR
// GetDisplayStringFromSidRequestResult( PSIDREQRESULT SidReqResult)
// {
//     LPWSTR lpDisplayString = NULL;
//
//     if (SidReqResult->SidNameUse == SidTypeUser ||
//         SidReqResult->SidNameUse == SidTypeGroup)
//     {
//         LoadAndFormatString(hDllInstance,
//                             IDS_USERDOMAINFORMAT,
//                             &lpDisplayString,
//                             SidReqResult->AccountName,
//                             SidReqResult->DomainName,
//                             SidReqResult->AccountName);
//     }
//     else
//     {
//         LoadAndFormatString(hDllInstance,
//                             IDS_USERFORMAT,
//                             &lpDisplayString,
//                             SidReqResult->AccountName);
//     }
//
//     return lpDisplayString;
// }
//
// static LPWSTR
// GetPrincipalDisplayString( PPRINCIPAL_LISTITEM PrincipalListItem)
// {
//     LPWSTR lpDisplayString = NULL;
//
//     if (PrincipalListItem->SidReqResult != NULL)
//     {
//         lpDisplayString = GetDisplayStringFromSidRequestResult(PrincipalListItem->SidReqResult);
//     }
//     else
//     {
//         ConvertSidToStringSidW((PSID)(PrincipalListItem + 1),
//                                &lpDisplayString);
//     }
//
//     return lpDisplayString;
// }
//
// static LPWSTR
// GetPrincipalAccountNameString( PPRINCIPAL_LISTITEM PrincipalListItem)
// {
//     LPWSTR lpDisplayString = NULL;
//
//     if (PrincipalListItem->SidReqResult != NULL)
//     {
//         LoadAndFormatString(hDllInstance,
//                             IDS_USERFORMAT,
//                             &lpDisplayString,
//                             PrincipalListItem->SidReqResult->AccountName);
//     }
//     else
//     {
//         ConvertSidToStringSid((PSID)(PrincipalListItem + 1),
//                               &lpDisplayString);
//     }
//
//     return lpDisplayString;
// }
//
// static VOID
// CreatePrincipalListItem( LVITEM *li,
//                          PSECURITY_PAGE sp,
//                          PPRINCIPAL_LISTITEM PrincipalListItem,
//                          INT Index,
//                          BOOL Selected)
// {
//     INT ImageIndex = 2;
//
//     if (PrincipalListItem->SidReqResult != NULL)
//     {
//         switch (PrincipalListItem->SidReqResult->SidNameUse)
//         {
//             case SidTypeUser:
//                 ImageIndex = 0;
//                 break;
//             case SidTypeWellKnownGroup:
//             case SidTypeGroup:
//                 ImageIndex = 1;
//                 break;
//             default:
//                 break;
//         }
//     }
//
//     li->mask = LVIF_IMAGE | LVIF_PARAM | LVIF_STATE | LVIF_TEXT;
//     li->iItem = Index;
//     li->iSubItem = 0;
//     li->state = (Selected ? LVIS_SELECTED : 0);
//     li->stateMask = LVIS_SELECTED;
//     li->pszText = PrincipalListItem->DisplayString;
//     li->iImage = ImageIndex;
//     li->lParam = (LPARAM)PrincipalListItem;
// }
//
// static INT
// AddPrincipalListEntry( PSECURITY_PAGE sp,
//                        PPRINCIPAL_LISTITEM PrincipalListItem,
//                        INT Index,
//                        BOOL Selected)
// {
//     LVITEM li;
//     INT Ret;
//
//     if (PrincipalListItem->DisplayString != NULL)
//     {
//         LocalFree((HLOCAL)PrincipalListItem->DisplayString);
//     }
//     PrincipalListItem->DisplayString = GetPrincipalDisplayString(PrincipalListItem);
//
//     CreatePrincipalListItem(&li,
//                             sp,
//                             PrincipalListItem,
//                             Index,
//                             Selected);
//
//     Ret = ListView_InsertItem!(sp->hWndPrincipalsList,
//                               &li);
//
//     return Ret;
// }
//
// static int __stdcall
// PrincipalCompare( LPARAM lParam1,
//                   LPARAM lParam2,
//                   LPARAM lParamSort)
// {
//     PPRINCIPAL_LISTITEM Item1 = (PPRINCIPAL_LISTITEM)lParam1;
//     PPRINCIPAL_LISTITEM Item2 = (PPRINCIPAL_LISTITEM)lParam2;
//
//     if (Item1->DisplayString != NULL && Item2->DisplayString != NULL)
//     {
//         return wcscmp(Item1->DisplayString,
//                       Item2->DisplayString);
//     }
//
//     return 0;
// }
//
// static VOID
// UpdatePrincipalListItem( PSECURITY_PAGE sp,
//                          INT PrincipalIndex,
//                          PPRINCIPAL_LISTITEM PrincipalListItem,
//                          PSIDREQRESULT SidReqResult)
// {
//     LVITEM li;
//
//     /* replace the request result structure */
//     if (PrincipalListItem->SidReqResult != NULL)
//     {
//         DereferenceSidReqResult(sp->SidCacheMgr,
//                                 PrincipalListItem->SidReqResult);
//     }
//
//     ReferenceSidReqResult(sp->SidCacheMgr,
//                           SidReqResult);
//     PrincipalListItem->SidReqResult = SidReqResult;
//
//     /* update the display string */
//     if (PrincipalListItem->DisplayString != NULL)
//     {
//         LocalFree((HLOCAL)PrincipalListItem->DisplayString);
//     }
//     PrincipalListItem->DisplayString = GetPrincipalDisplayString(PrincipalListItem);
//
//     /* update the list item */
//     CreatePrincipalListItem(&li,
//                             sp,
//                             PrincipalListItem,
//                             PrincipalIndex,
//                             FALSE);
//
//     /* don't change the list item state */
//     li.mask &= ~(LVIF_STATE | LVIF_PARAM);
//
//     (void)ListView_SetItem!(sp->hWndPrincipalsList,
//                            &li);
//
//     /* sort the principals list view again */
//     (void)ListView_SortItems!(sp->hWndPrincipalsList,
//                              PrincipalCompare,
//                              (LPARAM)sp);
// }
//
// static VOID
// ReloadPrincipalsList( PSECURITY_PAGE sp)
// {
//     PSECURITY_DESCRIPTOR SecurityDescriptor;
//     BOOL DaclPresent, DaclDefaulted, OwnerDefaulted;
//     PACL Dacl = NULL;
//     PSID OwnerSid = NULL;
//     LPTSTR OwnerSidString;
//     DWORD SidLen;
//     HRESULT hRet;
//
//     /* delete the cached ACL */
//     FreePrincipalsList(sp,
//                        &sp->PrincipalsListHead);
//
//     /* query the ACL */
//     hRet = sp->psi->lpVtbl->GetSecurity(sp->psi,
//                                         DACL_SECURITY_INFORMATION | OWNER_SECURITY_INFORMATION,
//                                         &SecurityDescriptor,
//                                         FALSE);
//     if (SUCCEEDED!(hRet) && SecurityDescriptor != NULL)
//     {
//         if (GetSecurityDescriptorOwner(SecurityDescriptor,
//                                        &OwnerSid,
//                                        &OwnerDefaulted))
//         {
//             sp->OwnerDefaulted = OwnerDefaulted;
//             if (sp->OwnerSid != NULL)
//             {
//                 LocalFree((HLOCAL)sp->OwnerSid);
//                 sp->OwnerSid = NULL;
//             }
//
//             SidLen = GetLengthSid(OwnerSid);
//             if (SidLen == 0)
//                 goto ClearOwner;
//
//             sp->OwnerSid = (PSID)LocalAlloc(LMEM_FIXED,
//                                             SidLen);
//             if (sp->OwnerSid != NULL)
//             {
//                 if (CopySid(SidLen,
//                             sp->OwnerSid,
//                             OwnerSid))
//                 {
//                     /* Lookup the SID now */
//                     if (!LookupSidCache(sp->SidCacheMgr,
//                                         sp->OwnerSid,
//                                         SidLookupCompletion,
//                                         sp))
//                     {
//                         /* Lookup was deferred */
//                         if (ConvertSidToStringSid(sp->OwnerSid,
//                                                   &OwnerSidString))
//                         {
//                             SetDlgItemText(sp->hWnd,
//                                            IDC_OWNER,
//                                            OwnerSidString);
//                             LocalFree((HLOCAL)OwnerSidString);
//                         }
//                         else
//                             goto ClearOwner;
//                     }
//                 }
//                 else
//                     goto ClearOwner;
//             }
//             else
//                 goto ClearOwner;
//         }
//         else
//         {
// ClearOwner:
//             SetDlgItemText(sp->hWnd,
//                            IDC_OWNER,
//                            NULL);
//         }
//
//         if (GetSecurityDescriptorDacl(SecurityDescriptor,
//                                       &DaclPresent,
//                                       &Dacl,
//                                       &DaclDefaulted) &&
//             DaclPresent && Dacl != NULL)
//         {
//             PSID Sid;
//             PACE_HEADER AceHeader;
//             ULONG AceIndex;
//
//             for (AceIndex = 0;
//                  AceIndex < Dacl->AceCount;
//                  AceIndex++)
//             {
//                 if (GetAce(Dacl,
//                            AceIndex,
//                            (LPVOID*)&AceHeader) &&
//                     AceHeader != NULL)
//                 {
//                     BOOL LookupDeferred;
//                     PPRINCIPAL_LISTITEM PrincipalListItem;
//
//                     Sid = AceHeaderToSID(AceHeader);
//
//                     PrincipalListItem = AddPrincipalToList(sp,
//                                                            Sid,
//                                                            AceHeader,
//                                                            &LookupDeferred);
//
//                     if (PrincipalListItem != NULL && LookupDeferred)
//                     {
//                         AddPrincipalListEntry(sp,
//                                               PrincipalListItem,
//                                               -1,
//                                               FALSE);
//                     }
//                 }
//             }
//         }
//         LocalFree((HLOCAL)SecurityDescriptor);
//     }
// }
//
// static VOID
// UpdateControlStates( PSECURITY_PAGE sp)
// {
//     PPRINCIPAL_LISTITEM Selected = (PPRINCIPAL_LISTITEM)ListViewGetSelectedItemData(sp->hWndPrincipalsList);
//
//     EnableWindow(sp->hBtnRemove,
//                  Selected != NULL);
//     EnableWindow(sp->hAceCheckList,
//                  Selected != NULL);
//
//     if (Selected != NULL)
//     {
//         LPWSTR szLabel;
//         LPWSTR szDisplayString;
//
//         szDisplayString = GetPrincipalAccountNameString(Selected);
//         if (LoadAndFormatString(hDllInstance,
//                                 IDS_PERMISSIONS_FOR,
//                                 &szLabel,
//                                 szDisplayString))
//         {
//             SetWindowText(sp->hPermissionsForLabel,
//                           szLabel);
//
//             LocalFree((HLOCAL)szLabel);
//         }
//
//         LocalFree((HLOCAL)szDisplayString);
//
//         /* FIXME - update the checkboxes */
//     }
//     else
//     {
//         WCHAR szPermissions[255];
//
//         if (LoadString(hDllInstance,
//                        IDS_PERMISSIONS,
//                        szPermissions,
//                        sizeof(szPermissions) / sizeof(szPermissions[0])))
//         {
//             SetWindowText(sp->hPermissionsForLabel,
//                           szPermissions);
//         }
//
//         SendMessage(sp->hAceCheckList,
//                     CLM_CLEARCHECKBOXES,
//                     0,
//                     0);
//     }
// }
//
// static void
// UpdatePrincipalInfo( PSECURITY_PAGE sp,
//                      PSIDLOOKUPNOTIFYINFO LookupInfo)
// {
//     PPRINCIPAL_LISTITEM CurItem;
//     LPWSTR DisplayName;
//
//     if (sp->OwnerSid != NULL &&
//         EqualSid(sp->OwnerSid,
//                  LookupInfo->Sid))
//     {
//         if (LookupInfo->SidRequestResult != NULL)
//             DisplayName = GetDisplayStringFromSidRequestResult(LookupInfo->SidRequestResult);
//         else if (!ConvertSidToStringSidW(LookupInfo->Sid,
//                                          &DisplayName))
//         {
//             DisplayName = NULL;
//         }
//
//         if (DisplayName != NULL)
//         {
//             SetDlgItemTextW(sp->hWnd,
//                             IDC_OWNER,
//                             DisplayName);
//
//             LocalFree((HLOCAL)DisplayName);
//         }
//     }
//
//     for (CurItem = sp->PrincipalsListHead;
//          CurItem != NULL;
//          CurItem = CurItem->Next)
//     {
//         if (EqualSid((PSID)(CurItem + 1),
//                      LookupInfo->Sid))
//         {
//             INT PrincipalIndex;
//             LVFINDINFO lvfi;
//
//             /* find the principal in the list */
//             lvfi.flags = LVFI_PARAM;
//             lvfi.lParam = (LPARAM)CurItem;
//             PrincipalIndex = ListView_FindItem!(sp->hWndPrincipalsList,
//                                                -1,
//                                                &lvfi);
//
//             if (PrincipalIndex != -1)
//             {
//                 /* update the principal in the list view control */
//                 UpdatePrincipalListItem(sp,
//                                         PrincipalIndex,
//                                         CurItem,
//                                         LookupInfo->SidRequestResult);
//
//                 if (ListViewGetSelectedItemData(sp->hWndPrincipalsList) == (LPARAM)CurItem)
//                 {
//                     UpdateControlStates(sp);
//                 }
//             }
//             else
//             {
//                 AddPrincipalListEntry(sp,
//                                       CurItem,
//                                       -1,
//                                       FALSE);
//             }
//             break;
//         }
//     }
// }
//
// static UINT __stdcall
// SecurityPageCallback( HWND hwnd,
//                       UINT uMsg,
//                       LPPROPSHEETPAGE ppsp)
// {
//     PSECURITY_PAGE sp = (PSECURITY_PAGE)ppsp->lParam;
//
//     switch (uMsg)
//     {
//         case PSPCB_CREATE:
//         {
//             return TRUE;
//         }
//         case PSPCB_RELEASE:
//         {
//             DestroySecurityPage(sp);
//             return FALSE;
//         }
//     }
//
//     return FALSE;
// }
//
// static VOID
// SetAceCheckListColumns( HWND hAceCheckList,
//                         UINT Button,
//                         HWND hLabel)
// {
//     POINT pt;
//     RECT rcLabel;
//
//     GetWindowRect(hLabel,
//                   &rcLabel);
//     pt.y = 0;
//     pt.x = (rcLabel.right - rcLabel.left) / 2;
//     MapWindowPoints(hLabel,
//                     hAceCheckList,
//                     &pt,
//                     1);
//
//     SendMessage(hAceCheckList,
//                 CLM_SETCHECKBOXCOLUMN,
//                 Button,
//                 pt.x);
// }
//
// static VOID
// LoadPermissionsList( PSECURITY_PAGE sp,
//                      GUID *GuidObjectType,
//                      DWORD dwFlags,
//                      SI_ACCESS *DefaultAccess)
// {
//     HRESULT hRet;
//     PSI_ACCESS AccessList;
//     ULONG nAccessList, DefaultAccessIndex;
//     WCHAR szSpecialPermissions[255];
//     BOOLEAN SpecialPermissionsPresent = FALSE;
//     ACCESS_MASK SpecialPermissionsMask = 0;
//
//     /* clear the permissions list */
//
//     SendMessage(sp->hAceCheckList,
//                 CLM_CLEAR,
//                 0,
//                 0);
//
//     /* query the access rights from the server */
//     hRet = sp->psi->lpVtbl->GetAccessRights(sp->psi,
//                                             GuidObjectType,
//                                             dwFlags, /* FIXME */
//                                             &AccessList,
//                                             &nAccessList,
//                                             &DefaultAccessIndex);
//     if (SUCCEEDED!(hRet) && nAccessList != 0)
//     {
//         LPCWSTR NameStr;
//         PSI_ACCESS CurAccess, LastAccess;
//         WCHAR NameBuffer[MAX_PATH];
//
//         /* save the default access rights to be used when adding ACEs later */
//         if (DefaultAccess != NULL)
//         {
//             *DefaultAccess = AccessList[DefaultAccessIndex];
//         }
//
//         LastAccess = AccessList + nAccessList;
//         for (CurAccess = &AccessList[0];
//              CurAccess != LastAccess;
//              CurAccess++)
//         {
//             if (CurAccess->dwFlags & dwFlags)
//             {
//                 /* get the permission name, load it from a string table if necessary */
//                 if (IS_INTRESOURCE!(CurAccess->pszName))
//                 {
//                     if (!LoadString(sp->ObjectInfo.hInstance,
//                                     (UINT)((ULONG_PTR)CurAccess->pszName),
//                                     NameBuffer,
//                                     sizeof(NameBuffer) / sizeof(NameBuffer[0])))
//                     {
//                         LoadString(hDllInstance,
//                                    IDS_UNKNOWN,
//                                    NameBuffer,
//                                    sizeof(NameBuffer) / sizeof(NameBuffer[0]));
//                     }
//                     NameStr = NameBuffer;
//                 }
//                 else
//                 {
//                     NameStr = CurAccess->pszName;
//                 }
//
//                 SendMessage(sp->hAceCheckList,
//                             CLM_ADDITEM,
//                             (WPARAM)CurAccess->mask,
//                             (LPARAM)NameStr);
//             }
//             else if (CurAccess->dwFlags & SI_ACCESS_SPECIFIC)
//             {
//                 SpecialPermissionsPresent = TRUE;
//                 SpecialPermissionsMask |= CurAccess->mask;
//             }
//         }
//     }
//
//     /* add the special permissions check item in case the specific access rights
//        aren't displayed */
//     if (SpecialPermissionsPresent &&
//         LoadString(hDllInstance,
//                    IDS_SPECIAL_PERMISSIONS,
//                    szSpecialPermissions,
//                    sizeof(szSpecialPermissions) / sizeof(szSpecialPermissions[0])))
//     {
//         /* add the special permissions check item */
//         sp->SpecialPermCheckIndex = (INT)SendMessage(sp->hAceCheckList,
//                                                      CLM_ADDITEM,
//                                                      (WPARAM)SpecialPermissionsMask,
//                                                      (LPARAM)szSpecialPermissions);
//         if (sp->SpecialPermCheckIndex != -1)
//         {
//             SendMessage(sp->hAceCheckList,
//                         CLM_SETITEMSTATE,
//                         (WPARAM)sp->SpecialPermCheckIndex,
//                         CIS_ALLOWDISABLED | CIS_DENYDISABLED | CIS_NONE);
//         }
//     }
// }
//
// static VOID
// ResizeControls( PSECURITY_PAGE sp,
//                 INT Width,
//                 INT Height)
// {
//     HWND hWndAllow, hWndDeny, hWndOwnerEdit;
//     RECT rcControl, rcControl2, rcControl3, rcWnd;
//     INT cxWidth, cxEdge, btnSpacing;
//     POINT pt, pt2;
//     HDWP dwp;
//     INT nControls = 8;
//     LVCOLUMN lvc;
//
//     hWndAllow = GetDlgItem(sp->hWnd,
//                            IDC_LABEL_ALLOW);
//     hWndDeny = GetDlgItem(sp->hWnd,
//                           IDC_LABEL_DENY);
//
//     GetWindowRect(sp->hWnd,
//                   &rcWnd);
//
//     cxEdge = GetSystemMetrics(SM_CXEDGE);
//
//     /* use the left margin of the principal list view control for all control
//        margins */
//     pt.x = 0;
//     pt.y = 0;
//     MapWindowPoints(sp->hWndPrincipalsList,
//                     sp->hWnd,
//                     &pt,
//                     1);
//     cxWidth = Width - (2 * pt.x);
//
//     if (sp->ObjectInfo.dwFlags & SI_ADVANCED)
//     {
//         nControls += 2;
//     }
//
//     if ((dwp = BeginDeferWindowPos(nControls)))
//     {
//         /* resize the owner edit field */
//         hWndOwnerEdit = GetDlgItem(sp->hWnd,
//                                    IDC_OWNER);
//         GetWindowRect(hWndOwnerEdit,
//                       &rcControl);
//         pt2.x = 0;
//         pt2.y = 0;
//         MapWindowPoints(hWndOwnerEdit,
//                         sp->hWnd,
//                         &pt2,
//                         1);
//         if (!(dwp = DeferWindowPos(dwp,
//                                    hWndOwnerEdit,
//                                    NULL,
//                                    0,
//                                    0,
//                                    Width - pt.x - pt2.x,
//                                    rcControl.bottom - rcControl.top,
//                                    SWP_NOMOVE | SWP_NOZORDER)))
//         {
//             goto EndDeferWnds;
//         }
//
//         /* resize the Principal list view */
//         GetWindowRect(sp->hWndPrincipalsList,
//                       &rcControl);
//         if (!(dwp = DeferWindowPos(dwp,
//                                    sp->hWndPrincipalsList,
//                                    NULL,
//                                    0,
//                                    0,
//                                    cxWidth,
//                                    rcControl.bottom - rcControl.top,
//                                    SWP_NOMOVE | SWP_NOZORDER)))
//         {
//             goto EndDeferWnds;
//         }
//
//         /* move the Add Principal button */
//         GetWindowRect(sp->hBtnAdd,
//                       &rcControl);
//         GetWindowRect(sp->hBtnRemove,
//                       &rcControl2);
//         btnSpacing = rcControl2.left - rcControl.right;
//         pt2.x = 0;
//         pt2.y = 0;
//         MapWindowPoints(sp->hBtnAdd,
//                         sp->hWnd,
//                         &pt2,
//                         1);
//         if (!(dwp = DeferWindowPos(dwp,
//                                    sp->hBtnAdd,
//                                    NULL,
//                                    pt.x + cxWidth - (rcControl2.right - rcControl2.left) -
//                                        (rcControl.right - rcControl.left) -
//                                        btnSpacing - cxEdge,
//                                    pt2.y,
//                                    0,
//                                    0,
//                                    SWP_NOSIZE | SWP_NOZORDER)))
//         {
//             goto EndDeferWnds;
//         }
//
//         /* move the Delete Principal button */
//         pt2.x = 0;
//         pt2.y = 0;
//         MapWindowPoints(sp->hBtnRemove,
//                         sp->hWnd,
//                         &pt2,
//                         1);
//         if (!(dwp = DeferWindowPos(dwp,
//                                    sp->hBtnRemove,
//                                    NULL,
//                                    pt.x + cxWidth - (rcControl2.right - rcControl2.left) - cxEdge,
//                                    pt2.y,
//                                    0,
//                                    0,
//                                    SWP_NOSIZE | SWP_NOZORDER)))
//         {
//             goto EndDeferWnds;
//         }
//
//         /* move the Permissions For label */
//         GetWindowRect(hWndAllow,
//                       &rcControl);
//         GetWindowRect(hWndDeny,
//                       &rcControl2);
//         GetWindowRect(sp->hPermissionsForLabel,
//                       &rcControl3);
//         pt2.x = 0;
//         pt2.y = 0;
//         MapWindowPoints(sp->hPermissionsForLabel,
//                         sp->hWnd,
//                         &pt2,
//                         1);
//         if (!(dwp = DeferWindowPos(dwp,
//                                    sp->hPermissionsForLabel,
//                                    NULL,
//                                    0,
//                                    0,
//                                    cxWidth - (rcControl2.right - rcControl2.left) -
//                                        (rcControl.right - rcControl.left) -
//                                        (2 * btnSpacing) - cxEdge,
//                                    rcControl3.bottom - rcControl3.top,
//                                    SWP_NOMOVE | SWP_NOZORDER)))
//         {
//             goto EndDeferWnds;
//         }
//
//         /* move the Allow label */
//         pt2.x = 0;
//         pt2.y = 0;
//         MapWindowPoints(hWndAllow,
//                         sp->hWnd,
//                         &pt2,
//                         1);
//         if (!(dwp = DeferWindowPos(dwp,
//                                    hWndAllow,
//                                    NULL,
//                                    cxWidth - (rcControl2.right - rcControl2.left) -
//                                        (rcControl.right - rcControl.left) -
//                                        btnSpacing - cxEdge,
//                                    pt2.y,
//                                    0,
//                                    0,
//                                    SWP_NOSIZE | SWP_NOZORDER)))
//         {
//             goto EndDeferWnds;
//         }
//
//         /* move the Deny label */
//         pt2.x = 0;
//         pt2.y = 0;
//         MapWindowPoints(hWndDeny,
//                         sp->hWnd,
//                         &pt2,
//                         1);
//         if (!(dwp = DeferWindowPos(dwp,
//                                    hWndDeny,
//                                    NULL,
//                                    cxWidth - (rcControl2.right - rcControl2.left) - cxEdge,
//                                    pt2.y,
//                                    0,
//                                    0,
//                                    SWP_NOSIZE | SWP_NOZORDER)))
//         {
//             goto EndDeferWnds;
//         }
//
//         /* resize the Permissions check list box */
//         GetWindowRect(sp->hAceCheckList,
//                       &rcControl);
//         GetWindowRect(sp->hBtnAdvanced,
//                       &rcControl2);
//         GetWindowRect(GetDlgItem(sp->hWnd,
//                                  IDC_LABEL_ADVANCED),
//                       &rcControl3);
//         if (!(dwp = DeferWindowPos(dwp,
//                                    sp->hAceCheckList,
//                                    NULL,
//                                    0,
//                                    0,
//                                    cxWidth,
//                                    ((sp->ObjectInfo.dwFlags & SI_ADVANCED) ?
//                                        Height - (rcControl.top - rcWnd.top) -
//                                            (rcControl3.bottom - rcControl3.top) - pt.x - btnSpacing :
//                                        Height - (rcControl.top - rcWnd.top) - pt.x),
//                                    SWP_NOMOVE | SWP_NOZORDER)))
//         {
//             goto EndDeferWnds;
//         }
//
//         if (sp->ObjectInfo.dwFlags & SI_ADVANCED)
//         {
//             /* move and resize the Advanced label */
//             if (!(dwp = DeferWindowPos(dwp,
//                                        GetDlgItem(sp->hWnd,
//                                                   IDC_LABEL_ADVANCED),
//                                        NULL,
//                                        pt.x,
//                                        Height - (rcControl3.bottom - rcControl3.top) - pt.x,
//                                        cxWidth - (rcControl2.right - rcControl2.left) - cxEdge,
//                                        rcControl3.bottom - rcControl3.top,
//                                        SWP_NOZORDER)))
//             {
//                 goto EndDeferWnds;
//             }
//
//             /* move and resize the Advanced button */
//             if (!(dwp = DeferWindowPos(dwp,
//                                        sp->hBtnAdvanced,
//                                        NULL,
//                                        cxWidth - (rcControl2.right - rcControl2.left) + pt.x,
//                                        Height - (rcControl2.bottom - rcControl2.top) - pt.x,
//                                        0,
//                                        0,
//                                        SWP_NOSIZE | SWP_NOZORDER)))
//             {
//                 goto EndDeferWnds;
//             }
//         }
//
//         EndDeferWindowPos(dwp);
//     }
//
// EndDeferWnds:
//     /* update the width of the principal list view column */
//     GetClientRect(sp->hWndPrincipalsList,
//                   &rcControl);
//     lvc.mask = LVCF_WIDTH;
//     lvc.cx = rcControl.right;
//     (void)ListView_SetColumn!(sp->hWndPrincipalsList,
//                              0,
//                              &lvc);
//
//     /* calculate the columns of the allow/deny checkboxes */
//     SetAceCheckListColumns(sp->hAceCheckList,
//                            CLB_ALLOW,
//                            hWndAllow);
//     SetAceCheckListColumns(sp->hAceCheckList,
//                            CLB_DENY,
//                            hWndDeny);
// }
//
// static PACE_HEADER
// BuildDefaultPrincipalAce( PSECURITY_PAGE sp,
//                           PSID pSid)
// {
//     PACCESS_ALLOWED_ACE Ace;
//     DWORD SidLen;
//     WORD AceSize;
//
//     SidLen = GetLengthSid(pSid);
//     AceSize = FIELD_OFFSET!(ACCESS_ALLOWED_ACE,
//                            SidStart) + (WORD)SidLen;
//     Ace = HeapAlloc(GetProcessHeap(),
//                     0,
//                     AceSize);
//     if (Ace != NULL)
//     {
//         Ace->Header.AceType = ACCESS_ALLOWED_ACE_TYPE;
//         Ace->Header.AceFlags = 0; /* FIXME */
//         Ace->Header.AceSize = AceSize;
//         Ace->Mask = sp->DefaultAccess.mask;
//
//         if (CopySid(SidLen,
//                     (PSID)&Ace->SidStart,
//                     pSid))
//         {
//             return &Ace->Header;
//         }
//
//         HeapFree(GetProcessHeap(),
//                  0,
//                  Ace);
//     }
//
//     return NULL;
// }
//
// static BOOL
// AddSelectedPrincipal( IDsObjectPicker *pDsObjectPicker,
//                       HWND hwndParent  ,
//                       PSID pSid,
//                       PVOID Context  )
// {
//     PACE_HEADER AceHeader;
//     PSECURITY_PAGE sp = (PSECURITY_PAGE)Context;
//
//     AceHeader = BuildDefaultPrincipalAce(sp,
//                                          pSid);
//     if (AceHeader != NULL)
//     {
//         PPRINCIPAL_LISTITEM PrincipalListItem;
//         BOOL LookupDeferred;
//
//         PrincipalListItem = AddPrincipalToList(sp,
//                                                pSid,
//                                                AceHeader,
//                                                &LookupDeferred);
//
//         if (PrincipalListItem != NULL && LookupDeferred)
//         {
//             AddPrincipalListEntry(sp,
//                                   PrincipalListItem,
//                                   -1,
//                                   FALSE);
//         }
//
//         HeapFree(GetProcessHeap(),
//                  0,
//                  AceHeader);
//     }
//
//     return TRUE;
// }
//
// static INT_PTR __stdcall
// SecurityPageProc( HWND hwndDlg,
//                   UINT uMsg,
//                   WPARAM wParam,
//                   LPARAM lParam)
// {
//     PSECURITY_PAGE sp;
//     INT_PTR Ret = FALSE;
//
//     sp = (PSECURITY_PAGE)GetWindowLongPtr(hwndDlg, DWLP_USER);
//     if (sp != NULL || uMsg == WM_INITDIALOG)
//     {
//         switch (uMsg)
//         {
//             case WM_NOTIFY:
//             {
//                 NMHDR *pnmh = (NMHDR*)lParam;
//
//                 if (pnmh->hwndFrom == sp->hWndPrincipalsList)
//                 {
//                     switch (pnmh->code)
//                     {
//                         case LVN_ITEMCHANGED:
//                         {
//                             LPNMLISTVIEW pnmv = (LPNMLISTVIEW)lParam;
//
//                             if ((pnmv->uChanged & LVIF_STATE) &&
//                                 ((pnmv->uOldState & (LVIS_FOCUSED | LVIS_SELECTED)) ||
//                                  (pnmv->uNewState & (LVIS_FOCUSED | LVIS_SELECTED))))
//                             {
//                                 UpdateControlStates(sp);
//                             }
//                             break;
//                         }
//                     }
//                 }
//                 else if (pnmh->hwndFrom == sp->hAceCheckList)
//                 {
//                     switch (pnmh->code)
//                     {
//                         case CLN_CHANGINGITEMCHECKBOX:
//                         {
//                             PNMCHANGEITEMCHECKBOX pcicb = (PNMCHANGEITEMCHECKBOX)lParam;
//
//                             /* make sure only one of both checkboxes is only checked
//                                at the same time */
//                             if (pcicb->Checked)
//                             {
//                                 pcicb->NewState &= ~((pcicb->CheckBox != CLB_DENY) ? CIS_DENY : CIS_ALLOW);
//                             }
//                             break;
//                         }
//                     }
//                 }
//                 else if (pnmh->hwndFrom == sp->hWnd)
//                 {
//                     switch(pnmh->code)
//                     {
//                         case SIDN_LOOKUPSUCCEEDED:
//                         {
//                             PSIDLOOKUPNOTIFYINFO LookupInfo = CONTAINING_RECORD(lParam,
//                                                                                 SIDLOOKUPNOTIFYINFO,
//                                                                                 nmh);
//
//                             /* a SID lookup succeeded, update the information */
//                             UpdatePrincipalInfo(sp,
//                                                 LookupInfo);
//                             break;
//                         }
//                     }
//                 }
//                 break;
//             }
//
//             case WM_COMMAND:
//             {
//                 switch (LOWORD!(wParam))
//                 {
//                     case IDC_ADD_PRINCIPAL:
//                     {
//                         HRESULT hRet;
//
//                         hRet = InitializeObjectPicker(sp->ServerName,
//                                                       &sp->ObjectInfo,
//                                                       &sp->pDsObjectPicker);
//                         if (SUCCEEDED!(hRet))
//                         {
//                             hRet = InvokeObjectPickerDialog(sp->pDsObjectPicker,
//                                                             hwndDlg,
//                                                             AddSelectedPrincipal,
//                                                             sp);
//                             if (FAILED!(hRet))
//                             {
//                                 MessageBox(hwndDlg, L"InvokeObjectPickerDialog failed!\n", NULL, 0);
//                             }
//
//                             /* delete the instance */
//                             FreeObjectPicker(sp->pDsObjectPicker);
//                         }
//                         else
//                         {
//                             MessageBox(hwndDlg, L"InitializeObjectPicker failed!\n", NULL, 0);
//                         }
//                         break;
//                     }
//
//                     case IDC_REMOVE_PRINCIPAL:
//                     {
//                         PPRINCIPAL_LISTITEM SelectedPrincipal;
//
//                         SelectedPrincipal = (PPRINCIPAL_LISTITEM)ListViewGetSelectedItemData(sp->hWndPrincipalsList);
//                         if (SelectedPrincipal != NULL)
//                         {
//                             /* FIXME */
//                         }
//                         break;
//                     }
//                 }
//                 break;
//             }
//
//             case WM_SIZE:
//             {
//                 ResizeControls(sp,
//                                (INT)LOWORD!(lParam),
//                                (INT)HIWORD!(lParam));
//                 break;
//             }
//
//             case WM_INITDIALOG:
//             {
//                 sp = (PSECURITY_PAGE)((LPPROPSHEETPAGE)lParam)->lParam;
//                 if(sp != NULL)
//                 {
//                     LV_COLUMN lvc;
//                     RECT rcLvClient;
//
//                     sp->hWnd = hwndDlg;
//                     sp->hWndPrincipalsList = GetDlgItem(hwndDlg, IDC_PRINCIPALS);
//                     sp->hBtnAdd = GetDlgItem(hwndDlg, IDC_ADD_PRINCIPAL);
//                     sp->hBtnRemove = GetDlgItem(hwndDlg, IDC_REMOVE_PRINCIPAL);
//                     sp->hBtnAdvanced = GetDlgItem(hwndDlg, IDC_ADVANCED);
//                     sp->hAceCheckList = GetDlgItem(hwndDlg, IDC_ACE_CHECKLIST);
//                     sp->hPermissionsForLabel = GetDlgItem(hwndDlg, IDC_LABEL_PERMISSIONS_FOR);
//
//                     sp->SpecialPermCheckIndex = -1;
//
//                     /* save the pointer to the structure */
//                     SetWindowLongPtr(hwndDlg, DWLP_USER, (DWORD_PTR)sp);
//
//                     (void)ListView_SetExtendedListViewStyleEx!(sp->hWndPrincipalsList,
//                                                               LVS_EX_FULLROWSELECT,
//                                                               LVS_EX_FULLROWSELECT);
//
//                     sp->hiPrincipals = ImageList_Create(16,
//                                                         16,
//                                                         ILC_COLOR32 | ILC_MASK,
//                                                         0,
//                                                         3);
//                     if (sp->hiPrincipals != NULL)
//                     {
//                         HBITMAP hbmImages;
//
//                         hbmImages = LoadBitmap(hDllInstance,
//                                                MAKEINTRESOURCE!(IDB_USRGRPIMAGES));
//                         if (hbmImages != NULL)
//                         {
//                             ImageList_AddMasked(sp->hiPrincipals,
//                                                 hbmImages,
//                                                 RGB!(255,
//                                                     0,
//                                                     255));
//
//                             DeleteObject(hbmImages);
//                         }
//                     }
//
//                     /* setup the listview control */
//                     if (sp->hiPrincipals != NULL)
//                     {
//                         (void)ListView_SetImageList!(sp->hWndPrincipalsList,
//                                                     sp->hiPrincipals,
//                                                     LVSIL_SMALL);
//                     }
//
//                     GetClientRect(sp->hWndPrincipalsList,
//                                   &rcLvClient);
//
//                     /* add a column to the list view */
//                     lvc.mask = LVCF_FMT | LVCF_WIDTH;
//                     lvc.fmt = LVCFMT_LEFT;
//                     lvc.cx = rcLvClient.right;
//                     (void)ListView_InsertColumn!(sp->hWndPrincipalsList,
//                                                 0,
//                                                 &lvc);
//
//                     ReloadPrincipalsList(sp);
//
//                     ListViewSelectItem(sp->hWndPrincipalsList,
//                                        0);
//
//                     /* calculate the columns of the allow/deny checkboxes */
//                     SetAceCheckListColumns(sp->hAceCheckList,
//                                            CLB_ALLOW,
//                                            GetDlgItem(hwndDlg, IDC_LABEL_ALLOW));
//                     SetAceCheckListColumns(sp->hAceCheckList,
//                                            CLB_DENY,
//                                            GetDlgItem(hwndDlg, IDC_LABEL_DENY));
//
//                     LoadPermissionsList(sp,
//                                         NULL,
//                                         SI_ACCESS_GENERAL |
//                                         ((sp->ObjectInfo.dwFlags & SI_CONTAINER) ? SI_ACCESS_CONTAINER : 0),
//                                         &sp->DefaultAccess);
//
//                     /* hide controls in case the flags aren't present */
//                     if (sp->ObjectInfo.dwFlags & SI_ADVANCED)
//                     {
//                         /* editing the permissions is least the user can do when
//                            the advanced button is showed */
//                         sp->ObjectInfo.dwFlags |= SI_EDIT_PERMS;
//                     }
//                     else
//                     {
//                         ShowWindow(sp->hBtnAdvanced,
//                                    SW_HIDE);
//                         ShowWindow(GetDlgItem(hwndDlg, IDC_LABEL_ADVANCED),
//                                    SW_HIDE);
//                     }
//
//                     /* enable quicksearch for the permissions checklist control */
//                     SendMessage(sp->hAceCheckList,
//                                 CLM_ENABLEQUICKSEARCH,
//                                 TRUE,
//                                 0);
//
//                     UpdateControlStates(sp);
//                 }
//
//                 Ret = TRUE;
//                 break;
//             }
//         }
//     }
//     return Ret;
// }
//
//
// /*
//  * CreateSecurityPage							EXPORTED
//  *
//  * @implemented
//  */
// HPROPSHEETPAGE
// WINAPI
// CreateSecurityPage( LPSECURITYINFO psi)
// {
//     PROPSHEETPAGE psp = {0};
//     PSECURITY_PAGE sPage;
//     SI_OBJECT_INFO ObjectInfo = {0};
//     HANDLE SidCacheMgr;
//     LPCWSTR SystemName = NULL;
//     HRESULT hRet;
//
//     if (psi == NULL)
//     {
//         SetLastError(ERROR_INVALID_PARAMETER);
//
//         DPRINT("No ISecurityInformation class passed!\n");
//         return NULL;
//     }
//
//     /* get the object information from the server. Zero the structure before
//        because some applications seem to return SUCCESS but only seem to set the
//        fields they care about. */
//     hRet = psi->lpVtbl->GetObjectInformation(psi,
//                                              &ObjectInfo);
//
//     if (FAILED!(hRet))
//     {
//         SetLastError(hRet);
//
//         DPRINT("CreateSecurityPage() failed! Failed to query the object information!\n");
//         return NULL;
//     }
//
//     if ((ObjectInfo.dwFlags & SI_SERVER_IS_DC) &&
//          ObjectInfo.pszServerName != NULL &&
//          ObjectInfo.pszServerName[0] != '\0')
//     {
//         SystemName = ObjectInfo.pszServerName;
//     }
//
//     SidCacheMgr = CreateSidCacheMgr(GetProcessHeap(),
//                                     SystemName);
//     if (SidCacheMgr == NULL)
//     {
//         DPRINT("Creating the SID cache failed!\n");
//         return NULL;
//     }
//
//     hRet = CoInitialize(NULL);
//     if (FAILED!(hRet))
//     {
//         DestroySidCacheMgr(SidCacheMgr);
//         DPRINT("CoInitialize failed!\n");
//         return NULL;
//     }
//
//     sPage = HeapAlloc(GetProcessHeap(),
//                       HEAP_ZERO_MEMORY,
//                       sizeof(SECURITY_PAGE));
//     if (sPage == NULL)
//     {
//         DestroySidCacheMgr(SidCacheMgr);
//         CoUninitialize();
//
//         SetLastError(ERROR_NOT_ENOUGH_MEMORY);
//
//         DPRINT("Not enough memory to allocate a SECURITY_PAGE!\n");
//         return NULL;
//     }
//
//     ZeroMemory!(sPage,
//                sizeof(*sPage));
//
//     sPage->psi = psi;
//     sPage->ObjectInfo = ObjectInfo;
//     sPage->ServerName = SystemName;
//     sPage->SidCacheMgr = SidCacheMgr;
//
//     psp.dwSize = sizeof(PROPSHEETPAGE);
//     psp.dwFlags = PSP_USECALLBACK;
//     psp.hInstance = hDllInstance;
//     *psp.__bindgen_anon_1.pszTemplate.as_mut() = MAKEINTRESOURCE!(IDD_SECPAGE);
//     psp.pfnDlgProc = SecurityPageProc;
//     psp.lParam = (LPARAM)sPage;
//     psp.pfnCallback = SecurityPageCallback;
//
//     if (ObjectInfo.dwFlags & SI_PAGE_TITLE)
//     {
//         psp.pszTitle = ObjectInfo.pszPageTitle;
//
//         if (psp.pszTitle != NULL)
//         {
//             psp.dwFlags |= PSP_USETITLE;
//         }
//     }
//     else
//     {
//         psp.pszTitle = NULL;
//     }
//
//     /* NOTE: the SECURITY_PAGE structure will be freed by the property page
//              callback! */
//
//     return CreatePropertySheetPage(&psp);
// }
//
//
// /*
//  * EditSecurity								EXPORTED
//  *
//  * @implemented
//  */
// BOOL
// WINAPI
// EditSecurity( HWND hwndOwner,
//               LPSECURITYINFO psi)
// {
//     HRESULT hRet;
//     SI_OBJECT_INFO ObjectInfo = {0};
//     PROPSHEETHEADER psh;
//     HPROPSHEETPAGE hPages[1];
//     LPWSTR lpCaption = NULL;
//     BOOL Ret;
//
//     if (psi == NULL)
//     {
//         SetLastError(ERROR_INVALID_PARAMETER);
//
//         DPRINT("No ISecurityInformation class passed!\n");
//         return FALSE;
//     }
//
//     /* get the object information from the server. Zero the structure before
//        because some applications seem to return SUCCESS but only seem to set the
//        fields they care about. */
//     hRet = psi->lpVtbl->GetObjectInformation(psi,
//                                              &ObjectInfo);
//
//     if (FAILED!(hRet))
//     {
//         SetLastError(hRet);
//
//         DPRINT("GetObjectInformation() failed!\n");
//         return FALSE;
//     }
//
//     /* create the page */
//     hPages[0] = CreateSecurityPage(psi);
//     if (hPages[0] == NULL)
//     {
//         DPRINT("CreateSecurityPage(), couldn't create property sheet!\n");
//         return FALSE;
//     }
//
//     psh.dwSize = sizeof(PROPSHEETHEADER);
//     psh.dwFlags = PSH_DEFAULT;
//     psh.hwndParent = hwndOwner;
//     psh.hInstance = hDllInstance;
//
//     /* Set the page title to the object name, make sure the format string
//        has "%1" NOT "%s" because it uses FormatMessage() to automatically
//        allocate the right amount of memory. */
//     if (LoadAndFormatString(hDllInstance,
//                             IDS_PSP_TITLE,
//                             &lpCaption,
//                             ObjectInfo.pszObjectName))
//     {
//         psh.pszCaption = lpCaption;
//     }
//     else
//     {
//         psh.pszCaption = ObjectInfo.pszObjectName;
//     }
//
//     psh.nPages = sizeof(hPages) / sizeof(HPROPSHEETPAGE);
//     *psh.__bindgen_anon_2.nStartPage.as_mut() = 0;
//     psh.__bindgen_anon_3.phpage = hPages;
//
//     Ret = (PropertySheet(&psh) != -1);
//
//     if (lpCaption != NULL)
//     {
//         LocalFree((HLOCAL)lpCaption);
//     }
//
//     return Ret;
// }
//
// BOOL
// WINAPI
// DllMain( HINSTANCE hinstDLL,
//          DWORD dwReason,
//          LPVOID lpvReserved)
// {
//     switch (dwReason)
//     {
//         case DLL_PROCESS_ATTACH:
//             hDllInstance = hinstDLL;
//
//             DisableThreadLibraryCalls(hinstDLL);
//
//             if (!RegisterCheckListControl(hinstDLL))
//             {
//                 DPRINT("Registering the CHECKLIST_ACLUI class failed!\n");
//                 return FALSE;
//             }
//             break;
//
//         case DLL_PROCESS_DETACH:
//             UnregisterCheckListControl(hinstDLL);
//             break;
//     }
//
//     return TRUE;
// }
//
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
