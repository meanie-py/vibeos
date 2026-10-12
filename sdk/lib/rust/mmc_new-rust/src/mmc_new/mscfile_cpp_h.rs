use crate::prelude::*;
// Deterministic UTF-16 literals from the selected C++ helper closure.
pub static CPP_COM_WSTR_0B3CAA753217: [WCHAR; 6] = [0x0050, 0x006F, 0x0069, 0x006E, 0x0074, 0x0000];
pub static CPP_COM_WSTR_152584348E5E: [WCHAR; 10] = [0x0052, 0x0065, 0x0063, 0x0074, 0x0061, 0x006E, 0x0067, 0x006C, 0x0065, 0x0000];
pub static CPP_COM_WSTR_1CBB7C41CB75: [WCHAR; 4] = [0x0025, 0x006C, 0x0064, 0x0000];
pub static CPP_COM_WSTR_2CEF69E0743E: [WCHAR; 7] = [0x0042, 0x006F, 0x0074, 0x0074, 0x006F, 0x006D, 0x0000];
pub static CPP_COM_WSTR_31B6D31F4B51: [WCHAR; 4] = [0x0032, 0x002E, 0x0030, 0x0000];
pub static CPP_COM_WSTR_3A86FDDAE8D1: [WCHAR; 15] = [0x0043, 0x006F, 0x006E, 0x0073, 0x006F, 0x006C, 0x0065, 0x0056, 0x0065, 0x0072, 0x0073, 0x0069, 0x006F, 0x006E, 0x0000];
pub static CPP_COM_WSTR_3C0007F19A1E: [WCHAR; 12] = [0x0053, 0x0068, 0x006F, 0x0077, 0x0043, 0x006F, 0x006D, 0x006D, 0x0061, 0x006E, 0x0064, 0x0000];
pub static CPP_COM_WSTR_3FB19AA2F69F: [WCHAR; 4] = [0x0054, 0x006F, 0x0070, 0x0000];
pub static CPP_COM_WSTR_464CA43DC6A6: [WCHAR; 5] = [0x004C, 0x0065, 0x0066, 0x0074, 0x0000];
pub static CPP_COM_WSTR_76E480513E2C: [WCHAR; 5] = [0x004E, 0x0061, 0x006D, 0x0065, 0x0000];
pub static CPP_COM_WSTR_7876B2CAE652: [WCHAR; 4] = [0x0033, 0x002E, 0x0030, 0x0000];
pub static CPP_COM_WSTR_831513BFA930: [WCHAR; 16] = [0x004D, 0x004D, 0x0043, 0x005F, 0x0043, 0x006F, 0x006E, 0x0073, 0x006F, 0x006C, 0x0065, 0x0046, 0x0069, 0x006C, 0x0065, 0x0000];
pub static CPP_COM_WSTR_898362848E86: [WCHAR; 16] = [0x0057, 0x0069, 0x006E, 0x0064, 0x006F, 0x0077, 0x0050, 0x006C, 0x0061, 0x0063, 0x0065, 0x006D, 0x0065, 0x006E, 0x0074, 0x0000];
pub static CPP_COM_WSTR_8D4CD219A817: [WCHAR; 2] = [0x0059, 0x0000];
pub static CPP_COM_WSTR_9855CDA098DC: [WCHAR; 2] = [0x0058, 0x0000];
pub static CPP_COM_WSTR_9BAB0882C9E9: [WCHAR; 12] = [0x004D, 0x0061, 0x0078, 0x0050, 0x006F, 0x0073, 0x0069, 0x0074, 0x0069, 0x006F, 0x006E, 0x0000];
pub static CPP_COM_WSTR_BEA2DE21F0EA: [WCHAR; 12] = [0x004D, 0x0069, 0x006E, 0x0050, 0x006F, 0x0073, 0x0069, 0x0074, 0x0069, 0x006F, 0x006E, 0x0000];
pub static CPP_COM_WSTR_C68E11CD808A: [WCHAR; 15] = [0x004E, 0x006F, 0x0072, 0x006D, 0x0061, 0x006C, 0x0050, 0x006F, 0x0073, 0x0069, 0x0074, 0x0069, 0x006F, 0x006E, 0x0000];
pub static CPP_COM_WSTR_D8BA62267C4F: [WCHAR; 6] = [0x0052, 0x0069, 0x0067, 0x0068, 0x0074, 0x0000];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
pub const CLSID_DOMDocument30: GUID = GUID {
    Data1: 0xf5078f32,
    Data2: 0xc551,
    Data3: 0x11d3,
    Data4: [0x89, 0xb9, 0x00, 0x00, 0xf8, 0x1f, 0xe2, 0x21],
};

const E_OUTOFMEMORY: HRESULT = 0x8007000E_u32 as i32; // E_OUTOFMEMORY





















impl MscFile {
    pub unsafe fn VariantFromString(&mut self, wszValue: PCWSTR, Variant: &mut VARIANT) -> HRESULT {
        let this_: *mut Self = self;
        let mut hr: HRESULT = S_OK;
        let bstr: BSTR = SysAllocString(wszValue);
        'CleanUp: {
            if bstr.is_null() {
                hr = E_OUTOFMEMORY;
                break 'CleanUp;
            }
    
            V_VT!(core::ptr::addr_of_mut!(*Variant)) = (VT_BSTR as u16);
            V_BSTR!(core::ptr::addr_of_mut!(*Variant)) = bstr;
        }
        return hr;
    }
    pub unsafe fn __ctor_0(&mut self, pszFileName: PCWSTR) {
        let this_: *mut Self = self;
        {
            (*this_).m_pDocument = core::ptr::null_mut();
            VariantInit(core::ptr::addr_of_mut!((*this_).m_varFileName));
            (*this_).VariantFromString(pszFileName, &mut (*this_).m_varFileName);
        }
    }
    pub unsafe fn __dtor__body(&mut self) {
        let this_: *mut Self = self;
        VariantClear(core::ptr::addr_of_mut!((*this_).m_varFileName));
        SAFE_RELEASE!((*this_).m_pDocument);
    }
    pub unsafe fn CreateAndInitDOM(&mut self) -> HRESULT {
        let this_: *mut Self = self;
        let mut hr: HRESULT = CoCreateInstance(&CLSID_DOMDocument30,
                                      core::ptr::null_mut(),
                                      CLSCTX_INPROC_SERVER,
                                      &IID_IXMLDOMDocument,
                                      core::ptr::addr_of_mut!((*this_).m_pDocument) as *mut *mut core::ffi::c_void);
        if hr >= 0
        {
            (*(*this_).m_pDocument).put_async(VARIANT_FALSE);
            (*(*this_).m_pDocument).put_validateOnParse(VARIANT_FALSE);
            (*(*this_).m_pDocument).put_resolveExternals(VARIANT_FALSE);
            (*(*this_).m_pDocument).put_preserveWhiteSpace(VARIANT_TRUE);
        }
        return hr;
    }
    pub unsafe fn CreateElement(&mut self, wszName: PCWSTR, ppElement: *mut *mut IXMLDOMElement) -> HRESULT {
        let this_: *mut Self = self;
        let mut hr: HRESULT = S_OK;
        *ppElement = core::ptr::null_mut();
    
        let bstrName: BSTR = SysAllocString(wszName);
        'CleanUp: {
            if bstrName.is_null() {
                hr = E_OUTOFMEMORY;
                break 'CleanUp;
            }
            hr = (*(*this_).m_pDocument).createElement(bstrName, ppElement);
            // CHK_HR(hr)
            if hr < 0 {
                break 'CleanUp;
            }
        }
        // CleanUp:
        SysFreeString(bstrName);
        return hr;
    }
    pub unsafe fn AppendChildToParent(&mut self, pChild: *mut IXMLDOMNode, pParent: *mut IXMLDOMNode) -> HRESULT {
        let this_: *mut Self = self;
        let mut hr: HRESULT = S_OK;
        let mut pChildOut: *mut IXMLDOMNode = core::ptr::null_mut();
        'CleanUp: {
            if pParent.is_null() {
                hr = (*(*this_).m_pDocument).appendChild(pChild, core::ptr::addr_of_mut!(pChildOut));
                if hr < 0 {
                    break 'CleanUp;
                }
            } else {
                hr = (*pParent).appendChild(pChild, core::ptr::addr_of_mut!(pChildOut));
                if hr < 0 {
                    break 'CleanUp;
                }
            }
        }
        SAFE_RELEASE!(pChildOut);
        return hr;
    }
    pub unsafe fn CreateAndAddPINode(&mut self, wszTarget: PCWSTR, wszData: PCWSTR) -> HRESULT {
        let this_: *mut Self = self;
        let mut hr: HRESULT = S_OK;
        let mut pPI: *mut IXMLDOMProcessingInstruction = core::ptr::null_mut();
    
        let bstrTarget: BSTR = SysAllocString(wszTarget);
        let bstrData: BSTR = SysAllocString(wszData);
        'CleanUp: {
            if !(!bstrTarget.is_null() && !bstrData.is_null()) {
                hr = E_OUTOFMEMORY;
                break 'CleanUp;
            }
    
            hr = (*(*this_).m_pDocument).createProcessingInstruction(bstrTarget, bstrData, core::ptr::addr_of_mut!(pPI));
            if hr < 0 {
                break 'CleanUp;
            }
            hr = (*this_).AppendChildToParent(pPI as *mut IXMLDOMNode, (*this_).m_pDocument as *mut IXMLDOMNode);
            if hr < 0 {
                break 'CleanUp;
            }
        }
    
        // CleanUp:
        SAFE_RELEASE!(pPI);
        SysFreeString(bstrTarget);
        SysFreeString(bstrData);
        hr
    }
    pub unsafe fn CreateAndAddCommentNode(&mut self, wszComment: PCWSTR) -> HRESULT {
        let this_: *mut Self = self;
        let mut hr: HRESULT = S_OK;
        let mut pComment: *mut IXMLDOMComment = core::ptr::null_mut();
    
        let bstrComment: BSTR = SysAllocString(wszComment);
        'CleanUp: {
            if bstrComment.is_null() {
                hr = E_OUTOFMEMORY;
                break 'CleanUp;
            }
    
            hr = (*(*this_).m_pDocument).createComment(bstrComment, core::ptr::addr_of_mut!(pComment));
            if hr < 0 {
                break 'CleanUp;
            }
            hr = (*this_).AppendChildToParent(pComment as *mut IXMLDOMNode, (*this_).m_pDocument as *mut IXMLDOMNode);
            if hr < 0 {
                break 'CleanUp;
            }
        }
    
        // CleanUp:
        SAFE_RELEASE!(pComment);
        SysFreeString(bstrComment);
        return hr;
    }
    pub unsafe fn CreateAndAddAttributeNode(&mut self, wszName: PCWSTR, wszValue: PCWSTR, pParent: *mut IXMLDOMElement) -> HRESULT {
        let this_: *mut Self = self;
        let mut hr: HRESULT = S_OK;
        let mut pAttribute: *mut IXMLDOMAttribute = core::ptr::null_mut();
        let mut pAttributeOut: *mut IXMLDOMAttribute = core::ptr::null_mut(); // Out param that is not used
    
        let mut bstrName: BSTR = core::ptr::null_mut();
        let mut varValue: VARIANT = core::mem::zeroed();
        VariantInit(core::ptr::addr_of_mut!(varValue));
    
        bstrName = SysAllocString(wszName);
        'CleanUp: {
            // CHK_ALLOC(bstrName)
            if bstrName.is_null() {
                hr = E_OUTOFMEMORY;
                break 'CleanUp;
            }
            // CHK_HR((*this_).VariantFromString(wszValue, varValue))
            hr = (*this_).VariantFromString(wszValue, &mut varValue);
            if hr < 0 {
                break 'CleanUp;
            }
    
            // CHK_HR((*this_).m_pDocument->createAttribute(bstrName, &pAttribute))
            hr = (*(*this_).m_pDocument).createAttribute(bstrName, core::ptr::addr_of_mut!(pAttribute));
            if hr < 0 {
                break 'CleanUp;
            }
            // CHK_HR(pAttribute->put_value(varValue))
            hr = (*pAttribute).put_value(varValue);
            if hr < 0 {
                break 'CleanUp;
            }
            // CHK_HR(pParent->setAttributeNode(pAttribute, &pAttributeOut))
            hr = (*pParent).setAttributeNode(pAttribute, core::ptr::addr_of_mut!(pAttributeOut));
            if hr < 0 {
                break 'CleanUp;
            }
        }
        // CleanUp:
        SAFE_RELEASE!(pAttribute);
        SAFE_RELEASE!(pAttributeOut);
        SysFreeString(bstrName);
        VariantClear(core::ptr::addr_of_mut!(varValue));
        return hr;
    }
    pub unsafe fn CreateAndAddTextNode(&mut self, wszText: PCWSTR, pParent: *mut IXMLDOMNode) -> HRESULT {
        let this_: *mut Self = self;
        let mut hr: HRESULT = S_OK;
        let mut pText: *mut IXMLDOMText = core::ptr::null_mut();
    
        let bstrText: BSTR = SysAllocString(wszText);
        'CleanUp: {
            if bstrText.is_null() {
                hr = E_OUTOFMEMORY;
                break 'CleanUp;
            }
    
            hr = (*(*this_).m_pDocument).createTextNode(bstrText, core::ptr::addr_of_mut!(pText));
            // CHK_HR(hr)
            if hr < 0 {
                break 'CleanUp;
            }
    
            hr = (*this_).AppendChildToParent(pText as *mut IXMLDOMNode, pParent);
            // CHK_HR(hr)
            if hr < 0 {
                break 'CleanUp;
            }
        }
    
        SAFE_RELEASE!(pText);
        SysFreeString(bstrText);
        return hr;
    }
    pub unsafe fn CreateAndAddCDATANode(&mut self, wszCDATA: PCWSTR, pParent: *mut IXMLDOMNode) -> HRESULT {
        let this_: *mut Self = self;
        let mut hr: HRESULT = S_OK;
        let mut pCDATA: *mut IXMLDOMCDATASection = core::ptr::null_mut();
    
        let bstrCDATA: BSTR = SysAllocString(wszCDATA);
        'CleanUp: {
            if bstrCDATA.is_null() {
                hr = E_OUTOFMEMORY;
                break 'CleanUp;
            }
            hr = (*(*this_).m_pDocument).createCDATASection(bstrCDATA, core::ptr::addr_of_mut!(pCDATA));
            // CHK_HR(hr)
            if hr < 0 {
                break 'CleanUp;
            }
            hr = (*this_).AppendChildToParent(pCDATA as *mut IXMLDOMNode, pParent);
            // CHK_HR(hr)
            if hr < 0 {
                break 'CleanUp;
            }
        }
    
        SAFE_RELEASE!(pCDATA);
        SysFreeString(bstrCDATA);
        return hr;
    }
    pub unsafe fn CreateAndAddElementNode(&mut self, wszName: PCWSTR, pParent: *mut IXMLDOMNode, ppElement: *mut *mut IXMLDOMElement) -> HRESULT {
        let this_: *mut Self = self;
        let mut hr: HRESULT = S_OK;
        let mut pElement: *mut IXMLDOMElement = core::ptr::null_mut();
        'CleanUp: {
            hr = (*this_).CreateElement(wszName, core::ptr::addr_of_mut!(pElement));
            if hr < 0 {
                break 'CleanUp;
            }
            hr = (*this_).AppendChildToParent(pElement as *mut IXMLDOMNode, pParent);
            if hr < 0 {
                break 'CleanUp;
            }
        }
        // CleanUp:
        if !ppElement.is_null() {
            *ppElement = pElement;
        } else {
            SAFE_RELEASE!(pElement);
        }
        hr
    }
    pub unsafe fn SaveWindowPlacement(&mut self, pWindow: *mut CWindow, pParentNode: *mut IXMLDOMElement) -> HRESULT {
        let this_: *mut Self = self;
        let mut pWindowPlacement: *mut IXMLDOMElement = core::ptr::null_mut();
        let mut pPointElement: *mut IXMLDOMElement = core::ptr::null_mut();
        let mut pRectangleElement: *mut IXMLDOMElement = core::ptr::null_mut();
        let mut wndpl: WINDOWPLACEMENT = core::mem::zeroed();
        let mut szBuffer: [u16; 32] = [0; 32];
        let mut hr: HRESULT = S_OK;
    
        'CleanUp: {
            (*pWindow).GetWindowPlacement(core::ptr::addr_of_mut!(wndpl));
    
            hr = (*this_).CreateAndAddElementNode(CPP_COM_WSTR_898362848E86.as_ptr(), pParentNode as *mut IXMLDOMNode, core::ptr::addr_of_mut!(pWindowPlacement));
            if hr < 0 {
                break 'CleanUp;
            }
            hr = (*this_).CreateAndAddAttributeNode(CPP_COM_WSTR_3C0007F19A1E.as_ptr(), ShowCmdToString((wndpl.showCmd) as i32), pWindowPlacement);
            if hr < 0 {
                break 'CleanUp;
            }
    
            hr = (*this_).CreateAndAddElementNode(CPP_COM_WSTR_0B3CAA753217.as_ptr(), pWindowPlacement as *mut IXMLDOMNode, core::ptr::addr_of_mut!(pPointElement));
            if hr < 0 {
                break 'CleanUp;
            }
            hr = (*this_).CreateAndAddAttributeNode(CPP_COM_WSTR_76E480513E2C.as_ptr(), CPP_COM_WSTR_BEA2DE21F0EA.as_ptr(), pPointElement);
            if hr < 0 {
                break 'CleanUp;
            }
            _swprintf(szBuffer.as_mut_ptr(), CPP_COM_WSTR_1CBB7C41CB75.as_ptr(), wndpl.ptMinPosition.x);
            hr = (*this_).CreateAndAddAttributeNode(CPP_COM_WSTR_9855CDA098DC.as_ptr(), szBuffer.as_ptr(), pPointElement);
            if hr < 0 {
                break 'CleanUp;
            }
            _swprintf(szBuffer.as_mut_ptr(), CPP_COM_WSTR_1CBB7C41CB75.as_ptr(), wndpl.ptMinPosition.y);
            hr = (*this_).CreateAndAddAttributeNode(CPP_COM_WSTR_8D4CD219A817.as_ptr(), szBuffer.as_ptr(), pPointElement);
            if hr < 0 {
                break 'CleanUp;
            }
            SAFE_RELEASE!(pPointElement);
    
            hr = (*this_).CreateAndAddElementNode(CPP_COM_WSTR_0B3CAA753217.as_ptr(), pWindowPlacement as *mut IXMLDOMNode, core::ptr::addr_of_mut!(pPointElement));
            if hr < 0 {
                break 'CleanUp;
            }
            hr = (*this_).CreateAndAddAttributeNode(CPP_COM_WSTR_76E480513E2C.as_ptr(), CPP_COM_WSTR_9BAB0882C9E9.as_ptr(), pPointElement);
            if hr < 0 {
                break 'CleanUp;
            }
            _swprintf(szBuffer.as_mut_ptr(), CPP_COM_WSTR_1CBB7C41CB75.as_ptr(), wndpl.ptMaxPosition.x);
            hr = (*this_).CreateAndAddAttributeNode(CPP_COM_WSTR_9855CDA098DC.as_ptr(), szBuffer.as_ptr(), pPointElement);
            if hr < 0 {
                break 'CleanUp;
            }
            _swprintf(szBuffer.as_mut_ptr(), CPP_COM_WSTR_1CBB7C41CB75.as_ptr(), wndpl.ptMaxPosition.y);
            hr = (*this_).CreateAndAddAttributeNode(CPP_COM_WSTR_8D4CD219A817.as_ptr(), szBuffer.as_ptr(), pPointElement);
            if hr < 0 {
                break 'CleanUp;
            }
            SAFE_RELEASE!(pPointElement);
    
            /* <Rectangle Name="NormalPosition" Top="0" Bottom="508" Left="0" Right="1186"/> */
            hr = (*this_).CreateAndAddElementNode(CPP_COM_WSTR_152584348E5E.as_ptr(), pWindowPlacement as *mut IXMLDOMNode, core::ptr::addr_of_mut!(pRectangleElement));
            if hr < 0 {
                break 'CleanUp;
            }
            hr = (*this_).CreateAndAddAttributeNode(CPP_COM_WSTR_76E480513E2C.as_ptr(), CPP_COM_WSTR_C68E11CD808A.as_ptr(), pRectangleElement);
            if hr < 0 {
                break 'CleanUp;
            }
            _swprintf(szBuffer.as_mut_ptr(), CPP_COM_WSTR_1CBB7C41CB75.as_ptr(), wndpl.rcNormalPosition.top);
            hr = (*this_).CreateAndAddAttributeNode(CPP_COM_WSTR_3FB19AA2F69F.as_ptr(), szBuffer.as_ptr(), pRectangleElement);
            if hr < 0 {
                break 'CleanUp;
            }
            _swprintf(szBuffer.as_mut_ptr(), CPP_COM_WSTR_1CBB7C41CB75.as_ptr(), wndpl.rcNormalPosition.bottom);
            hr = (*this_).CreateAndAddAttributeNode(CPP_COM_WSTR_2CEF69E0743E.as_ptr(), szBuffer.as_ptr(), pRectangleElement);
            if hr < 0 {
                break 'CleanUp;
            }
            _swprintf(szBuffer.as_mut_ptr(), CPP_COM_WSTR_1CBB7C41CB75.as_ptr(), wndpl.rcNormalPosition.left);
            hr = (*this_).CreateAndAddAttributeNode(CPP_COM_WSTR_464CA43DC6A6.as_ptr(), szBuffer.as_ptr(), pRectangleElement);
            if hr < 0 {
                break 'CleanUp;
            }
            _swprintf(szBuffer.as_mut_ptr(), CPP_COM_WSTR_1CBB7C41CB75.as_ptr(), wndpl.rcNormalPosition.right);
            hr = (*this_).CreateAndAddAttributeNode(CPP_COM_WSTR_D8BA62267C4F.as_ptr(), szBuffer.as_ptr(), pRectangleElement);
            if hr < 0 {
                break 'CleanUp;
            }
            SAFE_RELEASE!(pRectangleElement);
        }
    
        // CleanUp:
        SAFE_RELEASE!(pWindowPlacement);
    
        return hr;
    }
    pub unsafe fn ParseWindowPlacement(&mut self, pWindow: *mut CWindow, pParentElement: *mut IXMLDOMElement) -> HRESULT {
        let this_: *mut Self = self;
        let mut pWindowPlacementElement: *mut IXMLDOMElement = core::ptr::null_mut();
        let mut pChildList: *mut IXMLDOMNodeList = core::ptr::null_mut();
        let mut pNextElement: *mut IXMLDOMNode = core::mem::zeroed();
        let mut NodeType: DOMNodeType = core::mem::zeroed();
        let mut NodeName: BSTR = core::ptr::null_mut();
        let mut ShowCommand: VARIANT = core::mem::zeroed();
        let mut NameValue: VARIANT = core::mem::zeroed();
        let mut wndpl: WINDOWPLACEMENT = core::mem::zeroed();
        let mut hr: HRESULT = S_OK;
    
        ZeroMemory!(core::ptr::addr_of_mut!(wndpl) as *mut core::ffi::c_void, core::mem::size_of::<WINDOWPLACEMENT>());
        wndpl.length = core::mem::size_of::<WINDOWPLACEMENT>() as u32;
        wndpl.flags = 0;
    
        hr = (*this_).GetElement(pParentElement, CPP_COM_WSTR_898362848E86.as_ptr() as LPWSTR, core::ptr::addr_of_mut!(pWindowPlacementElement));
        if hr != S_OK {
            // goto done
            // (handled below by the labeled block)
        }
    
        'done: {
            if hr != S_OK {
                break 'done;
            }
    
            VariantInit(core::ptr::addr_of_mut!(ShowCommand));
            hr = (*this_).GetAttribute(pWindowPlacementElement, CPP_COM_WSTR_3C0007F19A1E.as_ptr() as LPWSTR, core::ptr::addr_of_mut!(ShowCommand));
            if hr != S_OK {
                break 'done;
            }
    
            wndpl.showCmd = (ShowCmdFromString(V_BSTR!(core::ptr::addr_of_mut!(ShowCommand)))) as u32;
    
            VariantClear(core::ptr::addr_of_mut!(ShowCommand));
    
            hr = (*pWindowPlacementElement).get_childNodes(core::ptr::addr_of_mut!(pChildList));
            if hr < 0 {
                break 'done;
            }
    
            loop {
                hr = (*pChildList).nextNode(core::ptr::addr_of_mut!(pNextElement));
                if hr != S_OK {
                    break;
                }
    
                hr = (*pNextElement).get_nodeType(core::ptr::addr_of_mut!(NodeType));
                if hr == S_OK {
                    if NodeType == NODE_ELEMENT {
                        NodeName = core::ptr::null_mut();
                        hr = (*pNextElement).get_baseName(core::ptr::addr_of_mut!(NodeName));
                        if hr == S_OK {
                            VariantInit(core::ptr::addr_of_mut!(NameValue));
    
                            hr = (*this_).GetAttribute(pNextElement as *mut IXMLDOMElement, CPP_COM_WSTR_76E480513E2C.as_ptr() as LPWSTR, core::ptr::addr_of_mut!(NameValue));
                            if hr == S_OK {
                                if _wcsicmp(NodeName, CPP_COM_WSTR_0B3CAA753217.as_ptr() as PCWSTR) == 0 {
                                    let mut PosX: VARIANT = core::mem::zeroed();
                                    let mut PosY: VARIANT = core::mem::zeroed();
                                    VariantInit(core::ptr::addr_of_mut!(PosX));
                                    VariantInit(core::ptr::addr_of_mut!(PosY));
                                    hr = (*this_).GetAttribute(pNextElement as *mut IXMLDOMElement, CPP_COM_WSTR_9855CDA098DC.as_ptr() as LPWSTR, core::ptr::addr_of_mut!(PosX));
                                    hr = (*this_).GetAttribute(pNextElement as *mut IXMLDOMElement, CPP_COM_WSTR_8D4CD219A817.as_ptr() as LPWSTR, core::ptr::addr_of_mut!(PosY));
    
                                    if _wcsicmp(V_BSTR!(core::ptr::addr_of_mut!(NameValue)), CPP_COM_WSTR_BEA2DE21F0EA.as_ptr() as PCWSTR) == 0 {
                                        wndpl.ptMinPosition.x = _wtol(V_BSTR!(core::ptr::addr_of_mut!(PosX)));
                                        wndpl.ptMinPosition.y = _wtol(V_BSTR!(core::ptr::addr_of_mut!(PosY)));
                                    } else if _wcsicmp(V_BSTR!(core::ptr::addr_of_mut!(NameValue)), CPP_COM_WSTR_9BAB0882C9E9.as_ptr() as PCWSTR) == 0 {
                                        wndpl.ptMaxPosition.x = _wtol(V_BSTR!(core::ptr::addr_of_mut!(PosX)));
                                        wndpl.ptMaxPosition.y = _wtol(V_BSTR!(core::ptr::addr_of_mut!(PosY)));
                                    }
                                    VariantClear(core::ptr::addr_of_mut!(PosX));
                                    VariantClear(core::ptr::addr_of_mut!(PosY));
                                } else if _wcsicmp(NodeName, CPP_COM_WSTR_152584348E5E.as_ptr() as PCWSTR) == 0 {
                                    let mut PosTop: VARIANT = core::mem::zeroed();
                                    let mut PosBottom: VARIANT = core::mem::zeroed();
                                    let mut PosLeft: VARIANT = core::mem::zeroed();
                                    let mut PosRight: VARIANT = core::mem::zeroed();
                                    VariantInit(core::ptr::addr_of_mut!(PosTop));
                                    VariantInit(core::ptr::addr_of_mut!(PosBottom));
                                    VariantInit(core::ptr::addr_of_mut!(PosLeft));
                                    VariantInit(core::ptr::addr_of_mut!(PosRight));
                                    hr = (*this_).GetAttribute(pNextElement as *mut IXMLDOMElement, CPP_COM_WSTR_3FB19AA2F69F.as_ptr() as LPWSTR, core::ptr::addr_of_mut!(PosTop));
                                    hr = (*this_).GetAttribute(pNextElement as *mut IXMLDOMElement, CPP_COM_WSTR_2CEF69E0743E.as_ptr() as LPWSTR, core::ptr::addr_of_mut!(PosBottom));
                                    hr = (*this_).GetAttribute(pNextElement as *mut IXMLDOMElement, CPP_COM_WSTR_464CA43DC6A6.as_ptr() as LPWSTR, core::ptr::addr_of_mut!(PosLeft));
                                    hr = (*this_).GetAttribute(pNextElement as *mut IXMLDOMElement, CPP_COM_WSTR_D8BA62267C4F.as_ptr() as LPWSTR, core::ptr::addr_of_mut!(PosRight));
                                    if _wcsicmp(V_BSTR!(core::ptr::addr_of_mut!(NameValue)), CPP_COM_WSTR_C68E11CD808A.as_ptr() as PCWSTR) == 0 {
                                        wndpl.rcNormalPosition.top = _wtol(V_BSTR!(core::ptr::addr_of_mut!(PosTop)));
                                        wndpl.rcNormalPosition.bottom = _wtol(V_BSTR!(core::ptr::addr_of_mut!(PosBottom)));
                                        wndpl.rcNormalPosition.left = _wtol(V_BSTR!(core::ptr::addr_of_mut!(PosLeft)));
                                        wndpl.rcNormalPosition.right = _wtol(V_BSTR!(core::ptr::addr_of_mut!(PosRight)));
                                    }
                                    VariantClear(core::ptr::addr_of_mut!(PosTop));
                                    VariantClear(core::ptr::addr_of_mut!(PosBottom));
                                    VariantClear(core::ptr::addr_of_mut!(PosLeft));
                                    VariantClear(core::ptr::addr_of_mut!(PosRight));
                                }
    
                                VariantClear(core::ptr::addr_of_mut!(NameValue));
                            }
    
                            SysFreeString(NodeName);
                            NodeName = core::ptr::null_mut();
                        }
                    }
                }
    
                (*pNextElement).Release();
            }
    
            (*pWindow).SetWindowPlacement(core::ptr::addr_of!(wndpl));
        }
    
        // done:
        if !pChildList.is_null() {
            (*pChildList).Release();
        }
    
        if !pWindowPlacementElement.is_null() {
            (*pWindowPlacementElement).Release();
        }
    
        return hr;
    }
    pub unsafe fn LoadDOM(&mut self) -> HRESULT {
        let this_: *mut Self = self;
        let mut pXMLErr: *mut IXMLDOMParseError = core::ptr::null_mut();
        let mut bstrErr: BSTR = core::ptr::null_mut();
        let mut varStatus: VARIANT_BOOL = 0;
        let mut hr: HRESULT = S_OK;
    
        hr = (*(*this_).m_pDocument).load((*this_).m_varFileName, core::ptr::addr_of_mut!(varStatus));
        'CleanUp: {
            if varStatus != VARIANT_TRUE
            {
                // Failed to load xml, get last parsing error
                hr = (*(*this_).m_pDocument).get_parseError(core::ptr::addr_of_mut!(pXMLErr));
                if hr < 0 {
                    break 'CleanUp;
                }
                hr = (*pXMLErr).get_reason(core::ptr::addr_of_mut!(bstrErr));
                if hr < 0 {
                    break 'CleanUp;
                }
            }
        }
    
        // CleanUp:
        SAFE_RELEASE!(pXMLErr);
        SysFreeString(bstrErr);
    
        return hr;
    }
    pub unsafe fn SaveDOM(&mut self) -> HRESULT {
        let this_: *mut Self = self;
        return (*(*this_).m_pDocument).save((*this_).m_varFileName);
    }
    pub unsafe fn CheckMscFile(&mut self, ppRootElement: *mut *mut IXMLDOMElement) -> HRESULT {
        let this_: *mut Self = self;
        let mut pRootElement: *mut IXMLDOMElement = core::ptr::null_mut();
        let mut rootName: BSTR = core::ptr::null_mut();
        let mut version: VARIANT = core::mem::zeroed();
        let mut hr: HRESULT = S_OK;
    
        *ppRootElement = core::ptr::null_mut();
    
        VariantInit(core::ptr::addr_of_mut!(version));
    
        'done: {
            hr = (*(*this_).m_pDocument).get_documentElement(core::ptr::addr_of_mut!(pRootElement));
            if hr < 0 {
                break 'done;
            }
    
            hr = (*pRootElement).get_baseName(core::ptr::addr_of_mut!(rootName));
            if hr < 0 {
                break 'done;
            }
    
            if _wcsicmp(rootName, CPP_COM_WSTR_831513BFA930.as_ptr()) != 0 {
                hr = S_FALSE;
                break 'done;
            }
    
            hr = (*this_).GetAttribute(
                pRootElement,
                CPP_COM_WSTR_3A86FDDAE8D1.as_ptr() as LPWSTR,
                core::ptr::addr_of_mut!(version),
            );
            if hr < 0 {
                break 'done;
            }
    
            if V_VT!(core::ptr::addr_of_mut!(version)) == VT_NULL {
                hr = S_FALSE;
                break 'done;
            }
    
            if wcscmp(V_BSTR!(core::ptr::addr_of_mut!(version)), CPP_COM_WSTR_31B6D31F4B51.as_ptr()) != 0
                && wcscmp(V_BSTR!(core::ptr::addr_of_mut!(version)), CPP_COM_WSTR_7876B2CAE652.as_ptr()) != 0
            {
                hr = S_FALSE;
                break 'done;
            }
    
            *ppRootElement = pRootElement;
        }
    
        // done:
        VariantClear(core::ptr::addr_of_mut!(version));
    
        if !rootName.is_null() {
            SysFreeString(rootName);
        }
    
        if (hr != S_OK) && !pRootElement.is_null() {
            (*pRootElement).Release();
        }
    
        hr
    }
    pub unsafe fn GetAttribute(&mut self, pElement: *mut IXMLDOMElement, pszAttributeName: LPWSTR, pAttributeValue: *mut VARIANT) -> HRESULT {
        let this_: *mut Self = self;
        let mut bstrAttribute: BSTR = core::ptr::null_mut();
        let mut hr: HRESULT = S_OK;
    
        'done: {
            bstrAttribute = SysAllocString(pszAttributeName);
            if bstrAttribute.is_null() {
                hr = E_OUTOFMEMORY;
                break 'done;
            }
    
            hr = (*pElement).getAttribute(bstrAttribute, pAttributeValue);
        }
    
        if !bstrAttribute.is_null() {
            SysFreeString(bstrAttribute);
        }
    
        return hr;
    }
    pub unsafe fn GetElement(&mut self, pParentElement: *mut IXMLDOMElement, pszElementName: LPWSTR, ppElement: *mut *mut IXMLDOMElement) -> HRESULT {
        let this_: *mut Self = self;
        let mut pChildList: *mut IXMLDOMNodeList = core::ptr::null_mut();
        let mut nextItem: *mut IXMLDOMNode = core::ptr::null_mut();
        let mut NodeType: DOMNodeType = core::mem::zeroed();
        let mut NodeName: BSTR = core::ptr::null_mut();
        let mut hr: HRESULT = S_OK;
    
        *ppElement = core::ptr::null_mut();
    
        'done: {
            hr = (*pParentElement).get_childNodes(core::ptr::addr_of_mut!(pChildList));
            if hr < 0 {
                break 'done;
            }
    
            loop {
                hr = (*pChildList).nextNode(core::ptr::addr_of_mut!(nextItem));
                if hr != S_OK {
                    break;
                }
    
                hr = (*nextItem).get_nodeType(core::ptr::addr_of_mut!(NodeType));
                if hr == S_OK {
                    if NodeType == NODE_ELEMENT {
                        NodeName = core::ptr::null_mut();
                        hr = (*nextItem).get_baseName(core::ptr::addr_of_mut!(NodeName));
                        if hr == S_OK {
                            if _wcsicmp(NodeName, pszElementName) == 0 {
                                *ppElement = nextItem as *mut IXMLDOMElement;
                                break;
                            }
                            SysFreeString(NodeName);
                            NodeName = core::ptr::null_mut();
                        }
                    }
                }
    
                (*nextItem).Release();
            }
        }
    
        if !NodeName.is_null() {
            SysFreeString(NodeName);
        }
    
        if !pChildList.is_null() {
            (*pChildList).Release();
        }
    
        hr
    }
    pub unsafe fn ReleaseNode(&mut self, mut pNode: *mut IXMLDOMNode) {
        let this_: *mut Self = self;
        if !pNode.is_null() {
            (*pNode).Release();
            pNode = core::ptr::null_mut();
        }
    }
}
