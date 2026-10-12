use crate::prelude::*;
// Deterministic UTF-16 literals from the selected C++ helper closure.
pub static CPP_COM_WSTR_00E356A89DA4: [WCHAR; 14] = [0x0043, 0x006F, 0x006D, 0x0070, 0x006F, 0x006E, 0x0065, 0x006E, 0x0074, 0x0044, 0x0061, 0x0074, 0x0061, 0x0000];
pub static CPP_COM_WSTR_34582C35F59D: [WCHAR; 9] = [0x0049, 0x006D, 0x0061, 0x0067, 0x0065, 0x0049, 0x0064, 0x0078, 0x0000];
pub static CPP_COM_WSTR_4F8320D91E97: [WCHAR; 2] = [0x0030, 0x0000];
pub static CPP_COM_WSTR_578E89C9480F: [WCHAR; 3] = [0x0049, 0x0044, 0x0000];
pub static CPP_COM_WSTR_733FB8CD60A4: [WCHAR; 6] = [0x004E, 0x006F, 0x0064, 0x0065, 0x0073, 0x0000];
pub static CPP_COM_WSTR_76E480513E2C: [WCHAR; 5] = [0x004E, 0x0061, 0x006D, 0x0065, 0x0000];
pub static CPP_COM_WSTR_9FF502938D7D: [WCHAR; 7] = [0x0053, 0x0074, 0x0072, 0x0069, 0x006E, 0x0067, 0x0000];
pub static CPP_COM_WSTR_AFF3436D7481: [WCHAR; 6] = [0x0043, 0x004C, 0x0053, 0x0049, 0x0044, 0x0000];
pub static CPP_COM_WSTR_B1C7484FA096: [WCHAR; 8] = [0x0050, 0x0072, 0x0065, 0x006C, 0x006F, 0x0061, 0x0064, 0x0000];
pub static CPP_COM_WSTR_B520369E2316: [WCHAR; 15] = [0x0043, 0x006F, 0x006D, 0x0070, 0x006F, 0x006E, 0x0065, 0x006E, 0x0074, 0x0044, 0x0061, 0x0074, 0x0061, 0x0073, 0x0000];
pub static CPP_COM_WSTR_BCC77CD05CF3: [WCHAR; 11] = [0x0043, 0x006F, 0x006D, 0x0070, 0x006F, 0x006E, 0x0065, 0x006E, 0x0074, 0x0073, 0x0000];
pub static CPP_COM_WSTR_C1B1DB869951: [WCHAR; 8] = [0x0042, 0x0069, 0x0074, 0x006D, 0x0061, 0x0070, 0x0073, 0x0000];
pub static CPP_COM_WSTR_CA7636CE488C: [WCHAR; 5] = [0x004E, 0x006F, 0x0064, 0x0065, 0x0000];
pub static CPP_COM_WSTR_CD28B1A69262: [WCHAR; 5] = [0x0074, 0x0072, 0x0075, 0x0065, 0x0000];

use core::ffi::c_void;




















impl CSnapin {
    pub unsafe fn RemoveAllSubnodes(&mut self) {
        let this_: *mut Self = self;
        let mut SubNode: *mut CSnapin;
        let mut pos: POSITION = (*this_).m_SubNodes.GetHeadPosition();
        while pos != core::ptr::null_mut() {
            SubNode = *(*this_).m_SubNodes.GetNext(&mut pos);
            if !SubNode.is_null() {
                CSnapin__delete(SubNode);
            }
        }
        (*this_).m_SubNodes.RemoveAll();
    }
    pub unsafe fn __ctor_0(&mut self, CacheEntry: *mut CSnapinCacheEntry, displayName: PWSTR) {
        let this_: *mut Self = self;
        (*this_).m_CacheEntry = CacheEntry;
        {
            if !displayName.is_null() {
                (*this_).m_DisplayName = CAtlString::from_ptr(displayName);
            } else {
                (*this_).m_DisplayName = (*(*this_).m_CacheEntry).Name().clone();
            }
        }
    }
    pub unsafe fn __dtor__body(&mut self) {
        let this_: *mut Self = self;
        (*this_).RemoveAllSubnodes();
    }
    pub unsafe fn Name(&mut self) -> &CAtlString {
        let this_: *mut Self = self;
        return (*(*this_).m_CacheEntry).Name();
    }
    pub unsafe fn Provider(&mut self) -> &CAtlString {
        let this_: *mut Self = self;
        return (*(*this_).m_CacheEntry).Provider();
    }
    pub unsafe fn Description(&mut self) -> &CAtlString {
        let this_: *mut Self = self;
        return (*(*this_).m_CacheEntry).Description();
    }
    pub unsafe fn Version(&mut self) -> &CAtlString {
        let this_: *mut Self = self;
        return (*(*this_).m_CacheEntry).Version();
    }
    pub unsafe fn DisplayName(&mut self) -> &CAtlString {
        let this_: *mut Self = self;
        return &(*this_).m_DisplayName;
    }
    pub unsafe fn SetDisplayName(&mut self, pszName: LPWSTR) {
        let this_: *mut Self = self;
        (*this_).m_DisplayName = CAtlString::from_ptr(pszName);
    }
    pub unsafe fn CacheEntry(&mut self) -> *mut CSnapinCacheEntry {
        let this_: *mut Self = self;
        (*this_).m_CacheEntry
    }
    pub unsafe fn OnAdd(&mut self, console: *mut IConsole) {
        let this_: *mut Self = self;}
    pub unsafe fn OnAccept(&mut self, console: *mut IConsole) {
        let this_: *mut Self = self;}
    pub unsafe fn SaveNode(&mut self, mscFile: *mut MscFile, pParentNode: *mut IXMLDOMElement) {
        let this_: *mut Self = self;
        let mut pNodeElement: *mut IXMLDOMElement = core::ptr::null_mut();
        let mut pNodesElement: *mut IXMLDOMElement = core::ptr::null_mut();
        let mut pStringElement: *mut IXMLDOMElement = core::ptr::null_mut();
        let mut pBitmapsElement: *mut IXMLDOMElement = core::ptr::null_mut();
        let mut pComponentDatasElement: *mut IXMLDOMElement = core::ptr::null_mut();
        let mut pComponentDataElement: *mut IXMLDOMElement = core::ptr::null_mut();
        let mut pComponentsElement: *mut IXMLDOMElement = core::ptr::null_mut();
        let mut pos: POSITION;
        let mut SubNode: *mut CSnapin;
        let mut hr: HRESULT = S_OK;
    
        /* <Node ID="3" ImageIdx="0" CLSID="{C96401CC-0E17-11D3-885B-00C04F72C717}" Preload="true"> */
        'CleanUp: {
            hr = (*mscFile).CreateAndAddElementNode(CPP_COM_WSTR_CA7636CE488C.as_ptr(), (pParentNode).cast(), core::ptr::addr_of_mut!(pNodeElement));
            if hr < 0 {
                break 'CleanUp;
            }
            hr = (*mscFile).CreateAndAddAttributeNode(CPP_COM_WSTR_578E89C9480F.as_ptr(), CPP_COM_WSTR_4F8320D91E97.as_ptr(), pNodeElement);
            if hr < 0 {
                break 'CleanUp;
            } /* FIXME */
            hr = (*mscFile).CreateAndAddAttributeNode(CPP_COM_WSTR_34582C35F59D.as_ptr(), CPP_COM_WSTR_4F8320D91E97.as_ptr(), pNodeElement);
            if hr < 0 {
                break 'CleanUp;
            } /* FIXME */
            hr = (*mscFile).CreateAndAddAttributeNode(CPP_COM_WSTR_AFF3436D7481.as_ptr(), (*(*this_).CacheEntry()).GuidString().GetString(), pNodeElement);
            if hr < 0 {
                break 'CleanUp;
            }
            hr = (*mscFile).CreateAndAddAttributeNode(CPP_COM_WSTR_B1C7484FA096.as_ptr(), CPP_COM_WSTR_CD28B1A69262.as_ptr(), pNodeElement);
            if hr < 0 {
                break 'CleanUp;
            } /* FIXME */
    
            /* <Nodes> */
            hr = (*mscFile).CreateAndAddElementNode(CPP_COM_WSTR_733FB8CD60A4.as_ptr(), (pNodeElement).cast(), core::ptr::addr_of_mut!(pNodesElement));
            if hr < 0 {
                break 'CleanUp;
            }
            pos = (*this_).m_SubNodes.GetHeadPosition();
            while !pos.is_null() {
                SubNode = *(*this_).m_SubNodes.GetNext(&mut pos);
                if !SubNode.is_null() {
                    (*SubNode).SaveNode(mscFile, pNodesElement);
                }
            }
            SAFE_RELEASE!(pNodesElement); /* </Nodes> */
    
            /* <String Name="Name" ID="2"/> */
            hr = (*mscFile).CreateAndAddElementNode(CPP_COM_WSTR_9FF502938D7D.as_ptr(), (pNodeElement).cast(), core::ptr::addr_of_mut!(pStringElement));
            if hr < 0 {
                break 'CleanUp;
            }
            hr = (*mscFile).CreateAndAddAttributeNode(CPP_COM_WSTR_76E480513E2C.as_ptr(), (*this_).m_DisplayName.GetString(), pStringElement);
            if hr < 0 {
                break 'CleanUp;
            } /* FIXME */
            SAFE_RELEASE!(pStringElement); /* </String> */
    
            /* <Bitmaps> */
            hr = (*mscFile).CreateAndAddElementNode(CPP_COM_WSTR_C1B1DB869951.as_ptr(), (pNodeElement).cast(), core::ptr::addr_of_mut!(pBitmapsElement));
            if hr < 0 {
                break 'CleanUp;
            }
            /*
                    <BinaryData Name="Small" BinaryRefIndex="3"/>
                    <BinaryData Name="Large" BinaryRefIndex="4"/>
            */
            SAFE_RELEASE!(pBitmapsElement); /* </Bitmaps> */
    
            /* <ComponentDatas> */
            hr = (*mscFile).CreateAndAddElementNode(CPP_COM_WSTR_B520369E2316.as_ptr(), (pNodeElement).cast(), core::ptr::addr_of_mut!(pComponentDatasElement));
            if hr < 0 {
                break 'CleanUp;
            }
            /* <ComponentData> */
            hr = (*mscFile).CreateAndAddElementNode(CPP_COM_WSTR_00E356A89DA4.as_ptr(), (pComponentDatasElement).cast(), core::ptr::addr_of_mut!(pComponentDataElement));
            if hr < 0 {
                break 'CleanUp;
            }
            /*
                    <GUID Name="Snapin">{C96401CC-0E17-11D3-885B-00C04F72C717}</GUID>
                    <Stream BinaryRefIndex="5"/>
            */
            SAFE_RELEASE!(pComponentDataElement); /* </ComponentData> */
            SAFE_RELEASE!(pComponentDatasElement); /* </ComponentDatas> */
    
            /* <Components /> */
            hr = (*mscFile).CreateAndAddElementNode(CPP_COM_WSTR_BCC77CD05CF3.as_ptr(), (pNodeElement).cast(), core::ptr::addr_of_mut!(pComponentsElement));
            if hr < 0 {
                break 'CleanUp;
            }
            SAFE_RELEASE!(pComponentsElement); /* </Components> */
        }
    
        /* CleanUp: */
        SAFE_RELEASE!(pNodeElement); /* </Node> */
    }
}
